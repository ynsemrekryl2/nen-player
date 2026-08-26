import AppKit
import Combine
import Foundation
import NenCore
import NenPlaybackMPV
import OSLog
import UniformTypeIdentifiers

@MainActor
public final class PlayerModel: ObservableObject {
    public typealias SessionFactory = @MainActor (MPVVideoView) throws -> any PlaybackSessionClient

    @Published public private(set) var mediaName: String?
    @Published public private(set) var recentMediaName: String?
    @Published public private(set) var playbackState: FfiPlaybackState = .idle
    @Published public private(set) var positionMilliseconds: UInt64 = 0
    @Published public private(set) var durationMilliseconds: UInt64?
    @Published public private(set) var volume: Float = 1
    @Published public private(set) var fatalMessage: String?
    @Published public private(set) var transientMessage: String?
    @Published public private(set) var controlsVisible = true
    @Published public private(set) var showsRemainingTime = false
    @Published public private(set) var seekPreviewMilliseconds: UInt64?
    /// How many subtitle sources are known for the medium being played.
    ///
    /// The menu itself is NEN-026's; this is what NEN-025 can honestly publish
    /// — enough for the shell to prove the catalog is wired up, and nothing
    /// that would commit the menu to a shape before it is designed.
    @Published public private(set) var subtitleSourceCount: UInt32 = 0

    public var hasMedia: Bool { mediaName != nil && fatalMessage == nil }
    public var isPlaying: Bool { playbackState == .playing }
    public var windowTitle: String { mediaName ?? "Nen Player" }
    public var displayedPositionMilliseconds: UInt64 {
        seekPreviewMilliseconds ?? positionMilliseconds
    }
    public var durationText: String {
        PlaybackPresentation.duration(
            position: displayedPositionMilliseconds,
            total: durationMilliseconds,
            showsRemaining: showsRemainingTime
        )
    }

    private static let logger = Logger(subsystem: "player.nen.macos", category: "playback")

    private let recentStore: any RecentMediaStoring
    private let sessionFactory: SessionFactory
    private let shouldPoll: Bool
    private let pollIntervalNanoseconds: UInt64
    private let controlsHideDelayNanoseconds: UInt64
    private let transientMessageDurationNanoseconds: UInt64
    /// How long a seek may keep `positionChanged` out before the guard gives up.
    ///
    /// The guard is normally released by the seek's own answer. An answer that
    /// never arrives must not freeze the displayed position for the rest of the
    /// session, so it expires as well.
    private let seekGuardTimeoutNanoseconds: UInt64
    /// A monotonic clock, injectable so a test can reach the expiry without waiting.
    private let now: () -> UInt64
    private let managesCursor: Bool
    /// The catalog and the documents behind it, owned by the core (NEN-025).
    ///
    /// Not behind a protocol, unlike `PlaybackSessionClient`: that one exists
    /// because a real playback session needs a real libmpv process. This one
    /// needs a filesystem and nothing else, so the tests drive the real thing
    /// and prove the real gates.
    private let subtitles = FfiSubtitleLibrary()
    private var session: (any PlaybackSessionClient)?
    private var pendingURL: URL?
    private var pollTask: Task<Void, Never>?
    private var controlsTask: Task<Void, Never>?
    private var transientTask: Task<Void, Never>?
    private var applicationActive = true
    private var playWhenReady = false
    private weak var videoView: MPVVideoView?
    private var accessedURL: URL?
    private var hasSecurityScope = false
    private var cursorHidden = false
    /// Seeks issued and not yet answered.
    ///
    /// A count rather than a flag: mpv merges seeks it cannot serve one by one
    /// and answers all of them from a single `playback-restart`, so the shell is
    /// owed one `seekCompleted` per request and must stay guarded until the last
    /// of them lands.
    private var pendingSeekCount = 0
    /// When the guard stops being believed, whatever the count says.
    private var seekGuardExpiry: UInt64 = 0

    public init(
        recentStore: any RecentMediaStoring = UserDefaultsRecentMediaStore(),
        startsPolling: Bool = true,
        pollIntervalNanoseconds: UInt64 = 50_000_000,
        controlsHideDelayNanoseconds: UInt64 = 2_500_000_000,
        transientMessageDurationNanoseconds: UInt64 = 3_000_000_000,
        seekGuardTimeoutNanoseconds: UInt64 = 1_500_000_000,
        now: @escaping () -> UInt64 = { DispatchTime.now().uptimeNanoseconds },
        managesCursor: Bool = true,
        sessionFactory: @escaping SessionFactory = { view in
            let engine = try MPVPlaybackEngine(videoView: view)
            return FfiPlaybackSession(engine: engine)
        }
    ) {
        self.recentStore = recentStore
        self.recentMediaName = recentStore.displayName
        self.sessionFactory = sessionFactory
        self.shouldPoll = startsPolling
        self.pollIntervalNanoseconds = pollIntervalNanoseconds
        self.controlsHideDelayNanoseconds = controlsHideDelayNanoseconds
        self.transientMessageDurationNanoseconds = transientMessageDurationNanoseconds
        self.seekGuardTimeoutNanoseconds = seekGuardTimeoutNanoseconds
        self.now = now
        self.managesCursor = managesCursor
        if startsPolling {
            startPolling()
        }
    }

    public func attach(to videoView: MPVVideoView) {
        self.videoView = videoView
        guard session == nil else { return }
        do {
            session = try sessionFactory(videoView)
            if shouldPoll {
                startPolling()
            }
            if let pendingURL {
                self.pendingURL = nil
                openMedia(at: pendingURL)
            }
        } catch {
            presentFatal(error)
        }
    }

    public func resume() {
        guard let videoView else { return }
        attach(to: videoView)
    }

    public func chooseMedia() {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.canChooseFiles = true
        panel.prompt = "Aç"
        if panel.runModal() == .OK, let url = panel.url {
            openMedia(at: url)
        }
    }

    public func openMedia(at url: URL) {
        guard url.isFileURL else {
            presentTransient("Bu medya kaynağı açılamıyor.")
            return
        }
        guard let session else {
            pendingURL = url
            return
        }

        releaseSecurityScope()
        hasSecurityScope = url.startAccessingSecurityScopedResource()
        accessedURL = url

        mediaName = url.lastPathComponent
        fatalMessage = nil
        transientMessage = nil
        discoverSidecar(besides: url)
        playbackState = .buffering
        positionMilliseconds = 0
        durationMilliseconds = nil
        seekPreviewMilliseconds = nil
        releaseSeekGuard()
        controlsVisible = true
        playWhenReady = true

        do {
            try recentStore.save(url)
            recentMediaName = url.lastPathComponent
        } catch {
            presentTransient("Son açılan medya kaydedilemedi.")
        }

        do {
            try session.load(locator: url.path)
        } catch {
            playWhenReady = false
            presentFatal(error)
        }
    }

    /// Asks the user for a subtitle file and loads it.
    ///
    /// A medium first: a subtitle catalog belongs to something being played,
    /// and an entry with nothing to attach to would be a row the user cannot
    /// act on.
    public func chooseSubtitleFile() {
        guard hasMedia else {
            presentTransient("Önce bir medya açın.")
            return
        }
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.canChooseFiles = true
        panel.allowedContentTypes = [.init(filenameExtension: "srt")].compactMap { $0 }
        panel.prompt = "Yükle"
        if panel.runModal() == .OK, let url = panel.url {
            loadSubtitleFile(at: url)
        }
    }

    /// Loads a subtitle file the user picked.
    ///
    /// A refusal is the user's own action being turned down, so it earns the
    /// *geçici* notification of ADR-0031 Karar 1. A file that was catalogued
    /// and marked broken earns nothing here: its surface is the menu
    /// (Karar 5), and interrupting playback for it would break the product's
    /// spine — "altyazı sorunu playback'i durdurmaz".
    public func loadSubtitleFile(at url: URL) {
        let outcome = subtitles.addFile(path: url.path)
        subtitleSourceCount = subtitles.sourceCount()
        if case let .rejected(reason) = outcome {
            presentTransient(PlaybackPresentation.subtitleRejectionMessage(for: reason))
        }
    }

    /// Looks for a sidecar next to a medium being opened.
    ///
    /// **Silent, whatever it finds** (ADR-0031 Karar 5). A scan is not the
    /// user's action; telling them that a file they never mentioned was refused
    /// would be noise at the exact moment they asked to watch something — and
    /// it would confirm that the refused file exists.
    private func discoverSidecar(besides url: URL) {
        subtitles.clear()
        _ = subtitles.addSidecarFor(mediaPath: url.path)
        subtitleSourceCount = subtitles.sourceCount()
    }

    public func openRecentMedia() {
        do {
            guard let url = try recentStore.resolve() else {
                recentStore.clear()
                recentMediaName = nil
                presentTransient("Son açılan medya artık kullanılamıyor.")
                return
            }
            openMedia(at: url)
        } catch {
            recentStore.clear()
            recentMediaName = nil
            presentTransient("Son açılan medya artık kullanılamıyor.")
        }
    }

    public func togglePlayback() {
        guard let session, hasMedia else { return }
        do {
            switch playbackState {
            case .playing, .buffering:
                playWhenReady = false
                try session.pause()
            case .ended:
                try session.seek(toMs: 0)
                try session.play()
            case .idle, .ready, .paused, .failed:
                try session.play()
            }
            // The engine reports the transition before the command returns, and
            // the bridge pulls after every command, so what the click produced
            // is already in the queue by now — measured at 50–180 us (NEN-055).
            // Waiting for the next poll tick would sit on an answer the shell
            // is already holding.
            drainSessionEvents()
        } catch {
            presentTransient(PlaybackPresentation.errorMessage(for: error))
        }
    }

    public func seekRelative(seconds: Int64) {
        let delta = seconds * 1_000
        let current = Int64(clamping: displayedPositionMilliseconds)
        let upper = durationMilliseconds.map { Int64(clamping: $0) } ?? Int64.max
        seek(to: UInt64(max(0, min(upper, current + delta))))
    }

    public func previewSeek(to milliseconds: UInt64) {
        seekPreviewMilliseconds = min(milliseconds, durationMilliseconds ?? milliseconds)
        pointerMoved()
    }

    public func commitSeek() {
        guard let target = seekPreviewMilliseconds else { return }
        // The seek first, the preview second. Dropping the preview before the
        // position moves would let `displayedPositionMilliseconds` fall back to
        // the pre-drag playhead for the length of one turn — the knob jumping
        // home the instant it is released. If the seek is refused the preview
        // still clears, which is honest: the position did not change.
        seek(to: target)
        seekPreviewMilliseconds = nil
    }

    public func setVolume(_ value: Float) {
        let clamped = min(1, max(0, value))
        guard let session else {
            volume = clamped
            return
        }
        do {
            try session.setVolume(volume: clamped)
            volume = clamped
        } catch {
            presentTransient(PlaybackPresentation.errorMessage(for: error))
        }
    }

    public func adjustVolume(by delta: Float) {
        setVolume(volume + delta)
    }

    public func toggleDurationMode() {
        showsRemainingTime.toggle()
    }

    public func pointerMoved() {
        showControls()
        if isPlaying {
            scheduleControlsHide()
        }
    }

    public func pointerLeft() {
        showControls()
    }

    public func applicationResignedActive() {
        applicationActive = false
        showControls()
    }

    public func applicationBecameActive() {
        applicationActive = true
        drainSessionEvents()
        resynchronize()
    }

    public func shutdown() {
        pollTask?.cancel()
        pollTask = nil
        controlsTask?.cancel()
        controlsTask = nil
        transientTask?.cancel()
        transientTask = nil
        showCursorIfNeeded()
        try? session?.shutdown()
        session = nil
        releaseSecurityScope()
        mediaName = nil
        playbackState = .idle
        positionMilliseconds = 0
        durationMilliseconds = nil
        seekPreviewMilliseconds = nil
        releaseSeekGuard()
        fatalMessage = nil
        transientMessage = nil
        playWhenReady = false
        controlsVisible = true
    }

    func consume(_ events: [FfiSessionEvent]) {
        for event in events {
            switch event {
            case let .positionChanged(positionMs):
                // The queue can still be holding the position from *before* a
                // seek at the moment the shell looks, and a drag is what makes
                // that the normal case rather than a rare one: AppKit runs a
                // nested tracking loop while the knob is held, so nothing
                // drains until it is released — and then the poll wakes in the
                // same runloop turn as the release, microseconds after the seek
                // command, before mpv's own new `time-pos` has crossed.
                //
                // Measured against real libmpv (NEN-053): applied blindly, that
                // one report snaps the knob back to the pre-drag playhead, and
                // the seek's answer moves it to the target 8 ms later. Until a
                // seek is answered, its target is what the shell knows.
                guard !seekIsInFlight else { break }
                positionMilliseconds = positionMs
            case let .seekCompleted(positionMs):
                positionMilliseconds = positionMs
                pendingSeekCount = max(0, pendingSeekCount - 1)
                if pendingSeekCount == 0 {
                    seekGuardExpiry = 0
                }
            case let .stateChanged(state):
                apply(state)
            case .tracksChanged:
                break
            case .endReached:
                apply(.ended)
            case let .failed(error):
                playWhenReady = false
                apply(.failed)
                presentFatal(error)
            case let .eventsLost(dropped):
                Self.logger.notice("Playback event queue resynchronized after dropping \(dropped, privacy: .public) events")
                resynchronize()
            }
        }
    }

    func hideControlsNow() {
        guard isPlaying else {
            showControls()
            return
        }
        controlsVisible = false
        if managesCursor, !cursorHidden {
            NSCursor.hide()
            cursorHidden = true
        }
    }

    private func startPolling() {
        guard pollTask == nil else { return }
        pollTask = Task { [weak self] in
            while !Task.isCancelled {
                guard let self else { return }
                try? await Task.sleep(nanoseconds: self.pollIntervalNanoseconds)
                if self.applicationActive {
                    self.drainSessionEvents()
                }
            }
        }
    }

    private func drainSessionEvents() {
        guard let session else { return }
        consume(session.drainEvents())
    }

    private func apply(_ state: FfiPlaybackState) {
        playbackState = state
        if state == .ready, playWhenReady, let session {
            playWhenReady = false
            do {
                try session.play()
            } catch {
                presentFatal(error)
                return
            }
        }
        if state == .ready || state == .playing || state == .paused || state == .ended {
            refreshPositionAndDuration()
        }
        if state == .playing {
            scheduleControlsHide()
        } else {
            showControls()
        }
    }

    private func refreshPositionAndDuration() {
        guard let session else { return }
        // Asking the engine is not automatically fresher than the queue:
        // measured, mpv moves `time-pos` to the target as soon as it takes the
        // seek command — but not before, and this call can land in that gap.
        if !seekIsInFlight, let position = try? session.positionMs() {
            positionMilliseconds = position
        }
        if let duration = try? session.durationMs() {
            durationMilliseconds = duration
        }
    }

    private func resynchronize() {
        guard let session else { return }
        // Whatever the shell was waiting for may be among the events that were
        // dropped, so the guard is dropped with them and the engine is asked.
        releaseSeekGuard()
        do {
            apply(try session.state())
            positionMilliseconds = try session.positionMs()
            durationMilliseconds = try session.durationMs()
            _ = try session.tracks(kind: .audio)
            _ = try session.tracks(kind: .subtitle)
        } catch {
            Self.logger.debug("Playback resynchronization found no readable state")
        }
    }

    private func seek(to milliseconds: UInt64) {
        guard let session, hasMedia else { return }
        do {
            try session.seek(toMs: milliseconds)
            positionMilliseconds = milliseconds
            pendingSeekCount += 1
            seekGuardExpiry = now() + seekGuardTimeoutNanoseconds
            // Safe only because of the guard above: what the queue is holding
            // at this instant is the position from *before* the seek, which is
            // exactly what NEN-053 measured and now refuses.
            drainSessionEvents()
        } catch {
            // A refused seek is owed no answer, so it must not leave a guard
            // behind: the position that keeps arriving is the true one.
            releaseSeekGuard()
            presentTransient(PlaybackPresentation.errorMessage(for: error))
        }
    }

    /// Whether a position report may still be describing where the medium was
    /// *before* a seek this shell is waiting on.
    private var seekIsInFlight: Bool {
        pendingSeekCount > 0 && now() < seekGuardExpiry
    }

    private func releaseSeekGuard() {
        pendingSeekCount = 0
        seekGuardExpiry = 0
    }

    private func scheduleControlsHide() {
        controlsTask?.cancel()
        controlsTask = Task { [weak self] in
            guard let self else { return }
            try? await Task.sleep(nanoseconds: self.controlsHideDelayNanoseconds)
            guard !Task.isCancelled else { return }
            self.hideControlsNow()
        }
    }

    private func showControls() {
        controlsTask?.cancel()
        controlsVisible = true
        showCursorIfNeeded()
    }

    private func showCursorIfNeeded() {
        if managesCursor, cursorHidden {
            NSCursor.unhide()
            cursorHidden = false
        }
    }

    private func presentFatal(_ error: Error) {
        playbackState = .failed
        fatalMessage = PlaybackPresentation.errorMessage(for: error)
        showControls()
    }

    private func presentTransient(_ message: String) {
        transientTask?.cancel()
        transientMessage = message
        let lifetime = transientMessageDurationNanoseconds
        transientTask = Task { [weak self] in
            try? await Task.sleep(nanoseconds: lifetime)
            guard !Task.isCancelled else { return }
            self?.transientMessage = nil
        }
    }

    private func releaseSecurityScope() {
        if hasSecurityScope {
            accessedURL?.stopAccessingSecurityScopedResource()
        }
        accessedURL = nil
        hasSecurityScope = false
    }
}
