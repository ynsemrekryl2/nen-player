import Cmpv
import AppKit
import Foundation
import NenCore

/// The macOS `PlaybackEngine` adapter (ADR-0012 Karar 1 and 2).
///
/// The core owns the session and calls in (ADR-0026 yön A); this object is what
/// it calls. Everything it reports goes back through ``drainEvents()`` and into
/// the core's own queue, so the delivery rules of ADR-0011 Karar 1 — coalescing,
/// overflow, `EventsLost` — are applied once, in Rust, for every platform.
///
/// # What this adapter does not do
///
/// - **No window ownership.** The shell may hand the adapter an
///   ``MPVVideoView``, but creating and retaining the window stays in
///   `NenPlayerShell`. Tests use the headless initializer and keep `vo=null` /
///   `ao=null`.
/// - **No text extraction, no external subtitles.** Both are capabilities and
///   both are declared absent, so the contract checks the typed refusal
///   instead. They arrive with NEN-044 and NEN-027.
/// - **No classifying.** The adapter reports what the container says — the
///   codec — and never decides what it means. Whether a subtitle codec carries
///   text is answered once, in `nen-ports`, for every platform (NEN-023).
///
/// # Threading
///
/// `mpv_wait_event` may only be called from one thread, so exactly one does:
/// ``pump``. Everything it learns lands in ``state`` under ``lock``. Commands
/// and property reads are called from whatever thread the core is on, which
/// libmpv allows.
public final class MPVPlaybackEngine: ForeignPlaybackEngine, @unchecked Sendable {
    /// Where the medium is in its life, as far as this adapter knows.
    enum Phase {
        case idle
        case loading
        case loaded
        case failed
    }

    struct Track {
        let ffIndex: UInt32
        /// mpv's own id, which is **per kind** and 1-based — so an audio track
        /// and a subtitle track both answer to `1`. The port's ids are one
        /// space across kinds (that is what lets the contract prove an audio id
        /// is not accepted as a subtitle one), so `ffIndex` is what leaves this
        /// file and `mpvId` never does.
        let mpvId: Int64
        let kind: FfiTrackKind
        let language: String?
        let codec: String
        let isDefault: Bool
        /// What the container calls this track.
        ///
        /// **Never logged** (K23 #8): a title is regularly a release name or a
        /// private filename. It crosses to the core, which holds it behind a
        /// guarded `Debug`, and `RedactionTests` proves this adapter says
        /// nothing about it.
        let title: String?
    }

    let handle: OpaquePointer
    let videoView: MPVVideoView?
    let lock = NSLock()
    /// Signalled once the pump thread has left its loop, so teardown can wait
    /// for it before freeing the handle it is blocked on.
    let pumpFinished = DispatchSemaphore(value: 0)

    // Everything below is guarded by `lock`.
    var phase: Phase = .idle
    var started = false
    var pendingSeeks = 0
    /// Whether mpv has actually *started* a seek that has not been answered yet.
    ///
    /// `pendingSeeks` counts what callers are owed; this says whether the core
    /// has begun serving any of it. The two are not the same instant, and
    /// NEN-051 measured what lives in the gap: `loadfile` produces a
    /// `playback-restart` of its own, delivered *after* `file-loaded` — that is,
    /// after `state()` already answers `Ready`. A shell that seeks the moment it
    /// sees `Ready` therefore has `pendingSeeks == 1` when the **load's** restart
    /// arrives, and without this flag that restart answers the seek with
    /// whatever `time-pos` happens to be — `0 ms` when the core has not served
    /// the seek yet.
    ///
    /// `MPV_EVENT_SEEK` is the marker that closes the gap: mpv emits it when a
    /// seek begins, on the same queue and before that seek's own
    /// `playback-restart`. A restart seen while this is `false` cannot be
    /// answering a seek, because no seek had started.
    var seekInFlight = false
    var stopRequested = false
    var shutDown = false
    /// The last `eof-reached` value seen, so a repeat does not re-announce the
    /// end and a rewind can announce it again.
    var atEndOfFile = false
    /// The loaded medium's tracks, in the adapter's own model.
    var trackList: [Track] = []
    var pending: [FfiPlaybackEvent] = []

    /// Creates either a headless contract-test engine or an engine embedded in
    /// the shell's AppKit video view. The shell owns the view and the adapter
    /// only gives its opaque address to libmpv; no path or media identity is
    /// retained here.
    public init(videoView: MPVVideoView? = nil) throws {
        guard let created = mpv_create() else {
            throw FfiPlaybackError.EngineFailure(code: MPV_ERROR_GENERIC.rawValue)
        }
        handle = created
        self.videoView = videoView

        // `config=no` and `terminal=no`: a user's ~/.config/mpv must not be able
        // to change what the contract measures, and this process owns no tty.
        // `pause=yes`: `load` must not start playback — the port says so and
        // M3's first exit criterion depends on the distinction.
        // `keep-open=yes`: at EOF the medium stays loaded, so `position`,
        // `duration` and `tracks` still answer in the `Ended` state.
        // `sid=no`: subtitle selection is the core's decision (§8, the menu's
        // `Kapalı` entry), never the engine's own default.
        var options = [
            ("config", "no"), ("terminal", "no"),
            ("idle", "yes"), ("pause", "yes"),
            ("keep-open", "yes"), ("sid", "no")
        ]
        if videoView == nil {
            options.append(("vo", "null"))
            options.append(("ao", "null"))
        }
        for (name, value) in options {
            try check(mpv_set_option_string(handle, name, value))
        }
        try check(mpv_initialize(handle))
        if let videoView {
            try check(mpv_set_option_string(handle, "vo", "libmpv"))
            try MainActor.assumeIsolated {
                try check(videoView.attach(to: handle))
            }
        }
        try check(mpv_observe_property(handle, 0, "time-pos", MPV_FORMAT_DOUBLE))
        // Observed rather than polled at the moment of a seek: measured against
        // the contract fixture, mpv sets `eof-reached` *after* the seek's
        // `playback-restart`, so checking it there reported "not ended" for a
        // medium that had just ended. Observing it means the end is announced
        // when it actually happens, whatever the ordering.
        try check(mpv_observe_property(handle, 0, "eof-reached", MPV_FORMAT_FLAG))

        let thread = Thread { [weak self] in self?.runEventLoop() }
        thread.name = "nen.playback.events"
        thread.start()
    }

    deinit {
        // `shutdown()` is idempotent and the contract calls it, but a shell
        // that drops the engine without calling it must not leak an mpv core.
        shutdownOnce()
    }

    // MARK: - ForeignPlaybackEngine

    public func capabilities() -> [FfiCapability] {
        // Rate and volume are real here. Text extraction (NEN-044) and external
        // subtitle injection (NEN-027) are not implemented yet, and declaring a
        // capability this engine does not have would make the contract check
        // the wrong half — the kit verifies the typed refusal for whatever is
        // absent, which is exactly the behaviour a caller gets today.
        Self.declaredCapabilities
    }

    public func load(locator: String) throws {
        try mutate(requireLoaded: false) {
            phase = .loading
            started = false
            stopRequested = false
            atEndOfFile = false
            seekInFlight = false
            trackList = []
            pending.append(.stateChanged(state: .buffering))
        }
        try command(["loadfile", locator])
    }

    public func play() throws {
        try mutate { started = true }
        try setFlag("pause", false)
        try mutate { pending.append(.stateChanged(state: .playing)) }
    }

    public func pause() throws {
        try setFlag("pause", true)
        try mutate { pending.append(.stateChanged(state: .paused)) }
    }

    public func stop() throws {
        try mutate { stopRequested = true }
        try command(["stop"])
        try mutate {
            phase = .idle
            started = false
            seekInFlight = false
            trackList = []
            pending.append(.stateChanged(state: .idle))
        }
    }

    public func seek(toMs: UInt64) throws {
        try mutate { pendingSeeks += 1 }
        // `absolute+exact` rather than plain `absolute`: without it mpv lands on
        // the nearest keyframe, which on a sparse-keyframe medium is seconds
        // away from what was asked. Measured on the contract fixture, exact
        // seeking lands on the requested millisecond.
        do {
            try command(["seek", String(format: "%.3f", Double(toMs) / 1000.0), "absolute+exact"])
        } catch {
            try? mutate { pendingSeeks = max(0, pendingSeeks - 1) }
            throw error
        }
    }

    public func positionMs() throws -> UInt64 {
        try requireMedia()
        let seconds = try double("time-pos")
        guard seconds.isFinite, seconds > 0 else { return 0 }
        return UInt64((seconds * 1000).rounded())
    }

    public func durationMs() throws -> UInt64? {
        try requireMedia()
        // A live stream reports no duration; that is `None`, not a failure.
        guard let seconds = try? double("duration"), seconds.isFinite, seconds > 0 else {
            return nil
        }
        return UInt64((seconds * 1000).rounded())
    }

    public func state() -> FfiPlaybackState {
        lock.lock()
        defer { lock.unlock() }
        if shutDown { return .idle }
        switch phase {
        case .idle: return .idle
        case .failed: return .failed
        case .loading: return .buffering
        case .loaded:
            if atEndOfFile { return .ended }
            // `Ready` and `Paused` are both "loaded, not playing". What
            // separates them is whether playback ever started — the port's own
            // wording: Ready is "able to play, but not playing".
            if !started { return .ready }
            return pausedUnlocked() ? .paused : .playing
        }
    }

    public func tracks(kind: FfiTrackKind) throws -> [FfiTrackDescriptor] {
        try requireMedia()
        lock.lock()
        defer { lock.unlock() }
        return trackList.filter { $0.kind == kind }.map {
            FfiTrackDescriptor(
                id: $0.ffIndex,
                kind: $0.kind,
                language: $0.language,
                codec: $0.codec,
                isDefault: $0.isDefault,
                title: $0.title
            )
        }
    }

    public func selectTrack(kind: FfiTrackKind, track: UInt32?) throws {
        try requireMedia()
        let property = kind == .audio ? "aid" : "sid"
        guard let wanted = track else {
            try setString(property, "no")
            return
        }
        lock.lock()
        let match = trackList.first { $0.ffIndex == wanted && $0.kind == kind }
        lock.unlock()
        guard let match else {
            // Either no such id at all, or the id belongs to the other kind.
            // The port makes no distinction, and neither should the message.
            throw FfiPlaybackError.UnknownTrack(kind: kind)
        }
        try setString(property, String(match.mpvId))
    }

    public func selectedTrack(kind: FfiTrackKind) throws -> UInt32? {
        try requireMedia()
        let property = kind == .audio ? "aid" : "sid"
        guard let raw = try? string(property), raw != "no", let mpvId = Int64(raw) else {
            return nil
        }
        lock.lock()
        defer { lock.unlock() }
        return trackList.first { $0.kind == kind && $0.mpvId == mpvId }?.ffIndex
    }

    public func drainEvents() -> [FfiPlaybackEvent] {
        lock.lock()
        defer { lock.unlock() }
        let drained = pending
        pending.removeAll(keepingCapacity: true)
        return drained
    }

    public func shutdown() throws {
        shutdownOnce()
    }

    public func setRate(rate: Float) throws {
        // Lifecycle before validation: a shut-down engine refuses everything,
        // and answering `RateOutOfRange` there would describe the argument
        // instead of the engine's actual state. The fake checks in the same
        // order, and the contract compares the two.
        try requireLive()
        guard Self.rateRange.contains(Double(rate)) else {
            throw FfiPlaybackError.RateOutOfRange(
                requested: rate,
                min: Float(Self.rateRange.lowerBound),
                max: Float(Self.rateRange.upperBound)
            )
        }
        try setDouble("speed", Double(rate))
    }

    public func setVolume(volume: Float) throws {
        try requireLive()
        let wanted = Double(max(0, min(1, volume))) * 100
        // Set at the device, not in the filter chain (NEN-054).
        //
        // mpv's software `volume` is applied *before* its audio buffer, whose
        // default is 200 ms (`--audio-buffer`, and mpv's own manual names the
        // consequence: a larger buffer "may make soft-volume ... react
        // slower"). Samples already in that buffer keep the old level, which is
        // what a user hears as the volume arriving late. `ao-volume` is applied
        // by the audio output itself, past the buffer.
        //
        // It exists only while an audio output does — mpv: "available only if
        // mpv audio output is currently active" — so the software volume stays
        // as the fallback for an engine that is idle or headless. The two must
        // never both hold a level, or they multiply: whenever the device takes
        // the level, the filter chain is put back to unity.
        do {
            try setDouble("ao-volume", wanted)
        } catch {
            try setDouble("volume", wanted)
            return
        }
        try setDouble("volume", 100)
    }

    public func extractText(track _: UInt32) throws -> String {
        // Declared absent above, so the core refuses before reaching here. The
        // typed refusal is repeated rather than trapped: a capability set and
        // an implementation that disagree is a bug the contract should see.
        throw FfiPlaybackError.Unsupported(capability: .embeddedTextExtraction)
    }

    public func injectSubtitle(webvtt _: String) throws {
        throw FfiPlaybackError.Unsupported(capability: .externalSubtitleInjection)
    }

    /// What `speed` this engine accepts. mpv goes wider; this is the range the
    /// adapter is willing to promise.
    static let rateRange: ClosedRange<Double> = 0.25...4.0
}

/// Builds a fresh engine per scenario.
///
/// The kit rebuilds the engine for every scenario so one cannot leave state
/// behind that makes the next pass — or fail — for the wrong reason. Across the
/// boundary that needs an object rather than a closure.
public final class MPVEngineFactory: ForeignEngineFactory, @unchecked Sendable {
    public init() {}

    public func build() -> ForeignPlaybackEngine {
        // The factory cannot report a failure: `build` has no error channel,
        // because a kit that could not build an engine has nothing to test. An
        // engine that failed to start refuses every call instead, which is a
        // result the contract can actually judge.
        (try? MPVPlaybackEngine()) ?? DeadEngine()
    }
}

/// What a shell gets when the engine could not start at all.
///
/// Not a silent no-op: every operation returns the typed refusal
/// `docs/architecture.md` requires, so a caller learns immediately rather than
/// watching nothing happen.
final class DeadEngine: ForeignPlaybackEngine, @unchecked Sendable {
    func capabilities() -> [FfiCapability] { [] }
    func load(locator _: String) throws { throw FfiPlaybackError.LoadFailed(reason: .unreadable) }
    func play() throws { throw FfiPlaybackError.NotLoaded }
    func pause() throws { throw FfiPlaybackError.NotLoaded }
    func stop() throws { throw FfiPlaybackError.NotLoaded }
    func seek(toMs _: UInt64) throws { throw FfiPlaybackError.NotLoaded }
    func positionMs() throws -> UInt64 { throw FfiPlaybackError.NotLoaded }
    func durationMs() throws -> UInt64? { throw FfiPlaybackError.NotLoaded }
    func state() -> FfiPlaybackState { .failed }
    func tracks(kind _: FfiTrackKind) throws -> [FfiTrackDescriptor] {
        throw FfiPlaybackError.NotLoaded
    }
    func selectTrack(kind _: FfiTrackKind, track _: UInt32?) throws {
        throw FfiPlaybackError.NotLoaded
    }
    func selectedTrack(kind _: FfiTrackKind) throws -> UInt32? {
        throw FfiPlaybackError.NotLoaded
    }
    func drainEvents() -> [FfiPlaybackEvent] { [] }
    func shutdown() throws {}
    func setRate(rate _: Float) throws { throw FfiPlaybackError.NotLoaded }
    func setVolume(volume _: Float) throws { throw FfiPlaybackError.NotLoaded }
    func extractText(track _: UInt32) throws -> String { throw FfiPlaybackError.NotLoaded }
    func injectSubtitle(webvtt _: String) throws { throw FfiPlaybackError.NotLoaded }
}

extension MPVPlaybackEngine {
    /// The capability set this adapter declares, as a value a test can compare
    /// against without instantiating an engine.
    public static let declaredCapabilities: [FfiCapability] = [.playbackRate, .volume]
}
