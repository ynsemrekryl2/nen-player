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
    /// Changes for every medium that reaches the playback session, even when
    /// two different files share the same basename. In-window presentation
    /// state uses this to discard UI that belongs to the previous medium.
    @Published private(set) var mediaPresentationRevision: UInt64 = 0
    /// How many subtitle sources are known for the medium being played.
    @Published public private(set) var subtitleSourceCount: UInt32 = 0
    /// §8's menu, re-derived by the core every time the catalog moves.
    @Published public private(set) var subtitleMenu: [FfiMenuSection] = []
    /// Which heading column two is showing. Bound to the heading's identity,
    /// never to its row index — column one grows while the menu is open
    /// (ADR-0031 Karar 4.2).
    @Published public private(set) var browsedSubtitleGroup: SubtitleMenuGroupID = .closed
    /// The row being shown, or `nil` for `Kapalı`.
    @Published public private(set) var selectedSubtitleToken: UInt32?
    /// Whether the sidecar scan is still running (ADR-0031 Karar 4).
    @Published public private(set) var isScanningSubtitles = false

    public var hasMedia: Bool { mediaName != nil && fatalMessage == nil }
    public var isPlaying: Bool { playbackState == .playing }
    public var windowTitle: String { mediaName ?? "Nen Player" }
    public var displayedPositionMilliseconds: UInt64 {
        seekPreviewMilliseconds ?? positionMilliseconds
    }
    /// Whether the catalog holds anything at all — what tells the two empty
    /// states apart.
    public var hasAnySubtitleSource: Bool { subtitleSourceCount > 0 }
    /// The rows under the heading column two is showing.
    public var browsedSubtitleEntries: [FfiMenuEntry] {
        subtitleMenu
            .first { SubtitleMenuGroupID($0.group) == browsedSubtitleGroup }?
            .entries ?? []
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
    /// The language the menu hoists and auto-selection looks for.
    ///
    /// Injected rather than read from `Locale` at the point of use: read
    /// statically, every menu test would depend on the language of the machine
    /// running it — green on a Turkish laptop and red on an English one, for a
    /// reason that has nothing to do with the code.
    private let preferredSubtitleLanguage: String?
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
    private var sidecarScanTask: Task<Void, Never>?
    /// ADR-0031 Karar 4.3: automatic selection runs **once**, at the start.
    /// A source discovered later never re-triggers it, however well it matches.
    private var hasAutoSelected = false
    private var controlsTask: Task<Void, Never>?
    private(set) var controlsPinned = false
    private var transientTask: Task<Void, Never>?
    private var applicationActive = true
    private var playWhenReady = false
    private weak var videoView: MPVVideoView?
    private var accessedURL: URL?
    private var hasSecurityScope = false
    private var cursorHidden = false
    /// The share of the surface the chrome was last reported to cover.
    private var subtitleBottomInset: Float = 0

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
        preferredSubtitleLanguage: String? = Locale.preferredLanguages.first,
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
        self.preferredSubtitleLanguage = preferredSubtitleLanguage
        if startsPolling {
            startPolling()
        }
    }

    public func attach(to videoView: MPVVideoView) {
        self.videoView = videoView
        guard session == nil else { return }
        do {
            session = try sessionFactory(videoView)
            // A new session starts with the subtitle at the bottom, and the
            // chrome the last one was told about is still on screen. Nothing
            // reports geometry again until the layout changes, so the value is
            // carried over rather than waited for.
            if subtitleBottomInset != 0 {
                try? session?.setSubtitleBottomInset(fraction: subtitleBottomInset)
            }
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
        mediaPresentationRevision &+= 1

        mediaName = url.lastPathComponent
        fatalMessage = nil
        transientMessage = nil
        resetSubtitleCatalog()
        startSidecarScan(besides: url)
        playbackState = .buffering
        positionMilliseconds = 0
        durationMilliseconds = nil
        seekPreviewMilliseconds = nil
        releaseSeekGuard()
        controlsVisible = true
        playWhenReady = true
        setControlsPinned(false)

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

    /// Forgets the previous medium's subtitles.
    private func resetSubtitleCatalog() {
        sidecarScanTask?.cancel()
        sidecarScanTask = nil
        subtitles.clear()
        selectedSubtitleToken = nil
        browsedSubtitleGroup = .closed
        hasAutoSelected = false
        isScanningSubtitles = false
        refreshSubtitleMenu()
    }

    /// Looks for a sidecar next to a medium being opened — **without the
    /// medium waiting for it** (ADR-0031 Karar 4).
    ///
    /// The scan touches the filesystem, so it runs off the main actor; the
    /// menu is openable throughout and fills in when the answer lands. M3's
    /// scan is one `stat` and one parse, but the phase is real rather than
    /// decorative: a shell that did this inline would make "tarama sürüyor" a
    /// state that never happens, and the rule it exists to protect
    /// (a growing list must not move the user's selection) untested.
    ///
    /// **Silent, whatever it finds** (ADR-0031 Karar 5). A scan is not the
    /// user's action; telling them that a file they never mentioned was refused
    /// would be noise at the exact moment they asked to watch something — and
    /// it would confirm that the refused file exists.
    private func startSidecarScan(besides url: URL) {
        let library = subtitles
        let path = url.path
        isScanningSubtitles = true
        sidecarScanTask = Task { [weak self] in
            _ = await Task.detached(priority: .utility) {
                library.addSidecarFor(mediaPath: path)
            }.value
            guard !Task.isCancelled, let self else { return }
            self.isScanningSubtitles = false
            self.refreshSubtitleMenu()
        }
    }

    /// Waits for the sidecar scan to land.
    ///
    /// For tests only, and internal so it cannot become a way for the UI to
    /// wait — the whole point of ADR-0031 Karar 4 is that nothing does.
    func awaitSidecarScan() async {
        await sidecarScanTask?.value
    }

    /// Catalogues the medium's own tracks and, once, opens the preferred one.
    ///
    /// Driven by the engine reaching `ready`, which is when the tracks exist.
    /// Calling it again is harmless: the catalog upserts and the tokens hold,
    /// so a resync re-reads without renumbering anything.
    private func catalogEmbeddedTracks() {
        guard let session, let tracks = try? session.tracks(kind: .subtitle) else { return }
        subtitles.addEmbedded(tracks: tracks)
        refreshSubtitleMenu()
        applyAutoSelectionIfNeeded()
    }

    /// Re-derives the menu. Cheap, and the only way the shell learns the list
    /// changed — the core owns grouping, order and which headings exist at all.
    private func refreshSubtitleMenu() {
        subtitleMenu = subtitles.menu(
            primary: preferredSubtitleLanguage,
            secondary: nil
        )
        subtitleSourceCount = subtitles.sourceCount()
        // Only when the heading is *gone* — a new medium. A heading that merely
        // moved down a row because the scan landed must not reset anything.
        if !subtitleMenu.contains(where: { SubtitleMenuGroupID($0.group) == browsedSubtitleGroup }) {
            browsedSubtitleGroup = .closed
        }
    }

    private func applyAutoSelectionIfNeeded() {
        guard !hasAutoSelected else { return }
        hasAutoSelected = true
        guard let token = subtitles.autoSelection(
            primary: preferredSubtitleLanguage,
            secondary: nil
        ) else { return }
        selectSubtitle(token: token)
    }

    // MARK: - Menu actions

    /// Points column two at a heading. Shows nothing, changes nothing.
    public func browseSubtitleGroup(_ group: SubtitleMenuGroupID) {
        guard group != .closed else {
            turnSubtitlesOff()
            return
        }
        browsedSubtitleGroup = group
    }

    /// §8's `Kapalı`: the subtitle goes away and column two says so.
    public func turnSubtitlesOff() {
        guard accepted({ try $0.hideSubtitle() }) else { return }
        selectedSubtitleToken = nil
        browsedSubtitleGroup = .closed
    }

    /// Shows a row.
    ///
    /// **The shell does not know how.** Whether the row is one of the medium's
    /// own tracks or a file the user loaded — and therefore whether it is
    /// selected or drawn — is the core session's decision (ADR-0013 Karar 3),
    /// so this method hands over a token and reacts to what came back. A row
    /// that cannot be shown, because the menu shrank underneath the click or
    /// because it is marked broken, changes nothing and says nothing: that is
    /// an outcome, not a failure (ADR-0031 Karar 5).
    public func selectSubtitle(token: UInt32) {
        var shown = true
        guard accepted({ shown = try $0.showSubtitle(library: self.subtitles, token: token) == .shown })
        else { return }
        guard shown else { return }
        selectedSubtitleToken = token
        // Column one's highlight is always what column two is showing, and the
        // row that is showing has to be reachable from it. Without this, a
        // subtitle opened by automatic selection leaves the menu pointing at
        // `Kapalı` — and column two then says "Altyazılar kapalı." over a
        // subtitle that is on screen. Measured on the real .app.
        if let group = groupContaining(token) {
            browsedSubtitleGroup = group
        }
    }

    private func groupContaining(_ token: UInt32) -> SubtitleMenuGroupID? {
        subtitleMenu
            .first { $0.entries.contains { $0.token == token } }
            .map { SubtitleMenuGroupID($0.group) }
    }

    /// Runs a subtitle command against the session, if there is one.
    ///
    /// Returns whether the caller may go on. With no session yet there is
    /// nothing to refuse, so the model's own state still moves — the menu is
    /// usable before a medium is open. A refusal is shown to the user and
    /// stops the caller, which is what keeps the highlight off a subtitle that
    /// never appeared.
    private func accepted(_ body: (any PlaybackSessionClient) throws -> Void) -> Bool {
        guard let session else { return true }
        do {
            try body(session)
            return true
        } catch {
            presentTransient(PlaybackPresentation.errorMessage(for: error))
            return false
        }
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
        if isPlaying, !controlsPinned {
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

    /// Tells the core how much of the surface this shell's own chrome covers,
    /// so the subtitle is not drawn underneath it (ADR-0037).
    ///
    /// `fraction` is a share of the surface **height** and comes from the
    /// view's own geometry — the only place that knows how tall the transport
    /// bar is and how tall the surface under it is. `0` when the chrome is
    /// hidden, which is where the player spends most of a session.
    ///
    /// Deduplicated because geometry reports arrive on every layout pass while
    /// this value changes twice per hide cycle.
    ///
    /// A refusal is swallowed on purpose. An inset outside the accepted range
    /// is this shell's own layout bug and never the user's doing; ADR-0031
    /// Karar 1 has no class for it, and a notification would tell the user
    /// about a state that is not theirs.
    public func setSubtitleBottomInset(_ fraction: Double) {
        guard fraction.isFinite else { return }
        let inset = Float(fraction)
        guard inset != subtitleBottomInset else { return }
        subtitleBottomInset = inset
        try? session?.setSubtitleBottomInset(fraction: inset)
    }

    /// Keeps the complete player chrome and cursor visible while an in-window
    /// control surface needs them. Releasing the pin restores the ordinary
    /// playback-driven hide cycle rather than hiding immediately.
    public func setControlsPinned(_ pinned: Bool) {
        controlsPinned = pinned
        showControls()
        if !pinned, isPlaying {
            scheduleControlsHide()
        }
    }

    public func shutdown() {
        pollTask?.cancel()
        pollTask = nil
        sidecarScanTask?.cancel()
        sidecarScanTask = nil
        controlsTask?.cancel()
        controlsTask = nil
        controlsPinned = false
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
        subtitles.clear()
        subtitleMenu = []
        subtitleSourceCount = 0
        selectedSubtitleToken = nil
        browsedSubtitleGroup = .closed
        hasAutoSelected = false
        isScanningSubtitles = false
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
                catalogEmbeddedTracks()
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
        guard isPlaying, !controlsPinned else {
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
        // The tracks exist the moment the file is loaded, which is what `ready`
        // means here. Cataloguing before play keeps ADR-0031 Karar 4's promise
        // literal: the menu holds `Kapalı` plus the embedded tracks from the
        // first frame, with no scan having finished.
        if state == .ready {
            catalogEmbeddedTracks()
        }
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
            catalogEmbeddedTracks()
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
        guard isPlaying, !controlsPinned else {
            showControls()
            return
        }
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
        controlsPinned = false
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
