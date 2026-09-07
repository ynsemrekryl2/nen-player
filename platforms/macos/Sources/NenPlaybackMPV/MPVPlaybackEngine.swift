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
/// - **No text extraction.** It is a capability and it is declared absent, so
///   the contract checks the typed refusal instead. It arrives with NEN-044.
/// - **No drawing decisions.** External subtitles are drawn by mpv itself
///   (ADR-0013 Karar 1), but *which* document reaches this adapter, and when,
///   is the core session's answer. Nothing here consults a catalog, and
///   nothing here opens a file: the document arrives as text.
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
    /// A seek issued while `phase == .loading`, held here instead of being
    /// sent to mpv — which refuses it (measured `MPV_ERROR_COMMAND`,
    /// `evidence/M3/NEN-052-measurement.md`) — and applied once
    /// `FILE_LOADED` arrives (ADR-0042). `pendingSeeks` is still incremented
    /// when a seek is deferred: the caller is owed a `SeekCompleted` exactly
    /// as if the command had been sent.
    ///
    /// Only the latest deferred target survives if more than one arrives in
    /// the window — the same trade `pendingSeeks` already makes for seeks
    /// mpv merges into one `playback-restart` (Karar 3): every caller is
    /// still answered, but only one position is actually reached.
    var deferredSeekMs: UInt64?
    var stopRequested = false
    /// The playlist entry id of the medium this adapter is currently loading or
    /// playing, as mpv numbered it.
    ///
    /// mpv ends the outgoing file when `loadfile` replaces it, and that end is
    /// indistinguishable by reason code from a genuine load failure: NEN-058
    /// measured both as `reason=STOP, error=0`. What separates them is *which
    /// entry* ended — the outgoing one or the one just asked for — and mpv says
    /// so in `playlist_entry_id`.
    ///
    /// Set to `noEntry` before each `loadfile` is issued, so an end that arrives
    /// while no id is known belongs to no load this adapter is waiting on. That
    /// ordering is structural rather than a race won by timing.
    var currentEntryId: Int64 = MPVPlaybackEngine.noEntry
    var shutDown = false
    /// The last `eof-reached` value seen, so a repeat does not re-announce the
    /// end and a rewind can announce it again.
    var atEndOfFile = false
    /// The loaded medium's tracks, in the adapter's own model.
    ///
    /// Only the medium's own. A document this adapter injected is a track as
    /// far as mpv is concerned, and it is deliberately not here — see
    /// ``reloadTracksUnlocked()``.
    var trackList: [Track] = []
    /// mpv's id for the document this adapter injected, or ``noTrack``.
    ///
    /// Kept because `sub-add` appends: injecting a second document without
    /// removing the first leaves both loaded, and the medium collects one more
    /// every time the user picks another subtitle.
    var injectedSubtitleId: Int64 = MPVPlaybackEngine.noTrack
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
        // `sid=no` and `sub-auto=no`: subtitles are the core's decision (§8,
        // the menu's `Kapalı` entry), never the engine's own.
        //
        // Both halves are needed and NEN-058 measured why. `sid=no` only stops
        // mpv from *showing* a subtitle; with mpv's default `sub-auto=exact` the
        // engine still **opens** the `.srt` sitting next to the medium, and it
        // showed up as a third subtitle track on a fixture that has two.
        // A file opened that way is a source no catalog ever saw, no menu ever
        // listed (ADR-0031 Karar 4/5), and none of NEN-025's file gates ever
        // examined — the engine would be loading subtitles behind the core's back.
        var options = [
            ("config", "no"), ("terminal", "no"),
            ("idle", "yes"), ("pause", "yes"),
            ("keep-open", "yes"), ("sid", "no"), ("sub-auto", "no")
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
        // Rate, volume, external subtitle injection and rendered-text
        // observation are real here; text extraction (NEN-044) is not. A
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
            // mpv drops external subtitles with the outgoing file, so the id
            // this adapter is holding stops meaning anything at that moment.
            injectedSubtitleId = Self.noTrack
            // Cleared *before* the command: from here until mpv answers, no
            // entry id is this load's, so the outgoing file's end cannot be
            // mistaken for this one's failure.
            currentEntryId = Self.noEntry
            // A deferred seek was waiting on the medium this adapter is
            // about to replace — the same reasoning ADR-0042 Karar 4 applies
            // to a load that fails applies here too, since that load's own
            // `FILE_LOADED` is now never coming.
            dropDeferredSeekUnlocked()
            pending.append(.stateChanged(state: .buffering))
        }
        let entry = try loadFile(locator)
        try mutate(requireLoaded: false) { currentEntryId = entry }
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
            injectedSubtitleId = Self.noTrack
            // Nothing is left to apply it to (ADR-0042 Karar 4).
            dropDeferredSeekUnlocked()
            pending.append(.stateChanged(state: .idle))
        }
    }

    public func seek(toMs: UInt64) throws {
        try requireMedia()
        var deferred = false
        lock.lock()
        pendingSeeks += 1
        if phase == .loading {
            // ADR-0042: the medium is still opening, and mpv's own `seek`
            // command refuses in this window — measured `MPV_ERROR_COMMAND`
            // (`evidence/M3/NEN-052-measurement.md`), a window of 2.5–12 ms
            // on the contract fixture, i.e. one a real seek routinely lands
            // in rather than rarely. The request is not lost: it is held and
            // applied at `FILE_LOADED`, the same event that answers the
            // load's own restart.
            deferredSeekMs = toMs
            deferred = true
        }
        lock.unlock()
        guard !deferred else { return }
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

    /// The display size of the picture, or `nil` when there is none
    /// (ADR-0038 Karar 1 and 3).
    ///
    /// `video-out-params/dw` and `/dh` are what mpv hands out **after** VO
    /// filters and rotation, which is exactly the number ADR-0038 Karar 3 asks
    /// for: the adapter reports the engine's own answer rather than deriving
    /// one. `video-params/dw`/`dh` is the fallback for the moment before a
    /// video output exists; measured on this engine the two agree once the
    /// file is open (`evidence/M3/NEN-068-measurement.md`).
    ///
    /// **The correction is real, not theoretical.** Measured on
    /// `fixtures/media/anamorphic-clip.mkv` — 720x576 stored, SAR 64:45 — both
    /// properties answer `1024x576`. An adapter that reported the stored frame
    /// size would open a 5:4 window for a 16:9 picture.
    ///
    /// `nil` for audio-only media: neither property is readable there, and the
    /// port says that is a state and not a failure. A zero from either is
    /// treated the same way — an engine that does not know yet must not be
    /// taken for one that knows the answer is zero.
    public func videoGeometry() throws -> FfiVideoGeometry? {
        try requireMedia()
        // During loadfile, video-out-params can still describe the outgoing
        // picture. Do not consume the new medium's initial window sizing with
        // that stale size; FILE_LOADED and VIDEO_RECONFIG will make it readable.
        lock.lock()
        let loaded = phase == .loaded
        lock.unlock()
        guard loaded else { return nil }
        guard let size = displaySize("video-out-params") ?? displaySize("video-params") else {
            return nil
        }
        return size
    }

    private func displaySize(_ prefix: String) -> FfiVideoGeometry? {
        guard let width = try? int("\(prefix)/dw"),
              let height = try? int("\(prefix)/dh"),
              width > 0, height > 0
        else { return nil }
        return FfiVideoGeometry(width: UInt32(width), height: UInt32(height))
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

    /// Hands mpv a document to draw, over `memory://` (ADR-0013 Karar 4).
    ///
    /// **The dialogue never touches the filesystem.** `sub-add` takes a URL and
    /// the obvious one would be a temp file; measured on libmpv 2.5.0,
    /// `memory://` is accepted for subtitles, so the user's own text stays in
    /// this process — no file to protect, no file to clean up, and nothing left
    /// behind by a crash (`evidence/M3/NEN-027-injection-measurement.md`).
    ///
    /// `select` is part of the command rather than a following `sid` write: mpv
    /// numbers the new track itself, so asking it to select what it just added
    /// avoids having to guess the id before reading it back.
    ///
    /// **Security (K23 #4):** `webvtt` is dialogue. It is passed to mpv and
    /// dropped; it is never logged, never stored and never included in an error.
    public func injectSubtitle(webvtt: String) throws {
        try requireMedia()
        // Before, not after: `sub-add` appends, so a second document without
        // this leaves the first one loaded for the rest of the medium.
        removeInjectedSubtitle()
        try command(["sub-add", "memory://" + webvtt, "select"])
        // `select` made it the current subtitle, so mpv's own answer is the id.
        if let raw = try? string("sid"), let id = Int64(raw) {
            lock.lock()
            injectedSubtitleId = id
            lock.unlock()
        }
    }

    /// Keeps the subtitle out of the bottom `inset` of the surface
    /// (ADR-0037 Karar 5).
    ///
    /// `sub-pos` is a percentage of the surface height with `100` at the
    /// bottom, so `(1 - inset) x 100` lifts the line by exactly the share the
    /// shell says its chrome covers. Measured on this engine, at a 360 pt
    /// surface: `sub-pos = 80` moved the band up 69 pt (20.0%) and `65` moved
    /// it 120.5 pt (33.5%), identically for an embedded track and an injected
    /// document (`evidence/M3/NEN-066-measurement.md`).
    ///
    /// mpv's own bottom margin is left alone, and after the lift it becomes
    /// the gap between the subtitle and whatever the shell put there.
    ///
    /// `sub-margin-y` was measured and not used: it needs `sub-use-margins`
    /// and, on the ASS side, `sub-ass-force-margins`, while `sub-pos` gave the
    /// same linear result for both kinds of subtitle with neither.
    ///
    /// The value arrives already inside `0.0...0.5` — the renderer port
    /// refuses anything else before the engine is called — so this rounds
    /// rather than judges.
    public func setSubtitleBottomInset(fraction: Float) throws {
        let position = min(100, max(0, Int(((1 - Double(fraction)) * 100).rounded())))
        try setString("sub-pos", String(position))
    }

    /// What mpv is drawing right now, if anything.
    ///
    /// `sub-text` is what reached the screen, not what was asked for — which is
    /// the whole reason this exists (ADR-0013 Karar 2). mpv answers with an
    /// empty string in a gap between cues; that is `nil` here, because "nothing
    /// is on screen" is a state and not an empty line of dialogue.
    ///
    /// **Security (K23 #4):** dialogue again. It crosses to the core and is
    /// never logged here.
    public func renderedSubtitleText() throws -> String? {
        try requireMedia()
        guard let text = try? string("sub-text"), !text.isEmpty else { return nil }
        return text
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
    func videoGeometry() throws -> FfiVideoGeometry? { throw FfiPlaybackError.NotLoaded }
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
    func renderedSubtitleText() throws -> String? { throw FfiPlaybackError.NotLoaded }
    func setSubtitleBottomInset(fraction _: Float) throws { throw FfiPlaybackError.NotLoaded }
}

extension MPVPlaybackEngine {
    /// The capability set this adapter declares, as a value a test can compare
    /// against without instantiating an engine.
    public static let declaredCapabilities: [FfiCapability] = [
        .externalSubtitleInjection, .renderedTextObservation, .playbackRate, .volume
    ]

    /// No playlist entry. mpv numbers its entries from 1, so no real entry can
    /// collide with this.
    static let noEntry: Int64 = 0

    /// No injected subtitle. mpv numbers subtitle tracks from 1 as well.
    static let noTrack: Int64 = 0
}
