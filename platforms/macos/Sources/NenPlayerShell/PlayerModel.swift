import AppKit
import Combine
import Foundation
import NenCore
import NenPlaybackMPV
import NenRemoteEvidenceHTTP
import OSLog
import UniformTypeIdentifiers

@MainActor
public final class PlayerModel: ObservableObject {
    public typealias SessionFactory = @MainActor (MPVVideoView) throws -> any PlaybackSessionClient
    /// Collects optional evidence for a handoff without becoming part of the
    /// playback critical path. The closure is injected so platform tests can
    /// prove the call and its failure policy without making a network request.
    public typealias HandoffEvidenceCollector = @Sendable (URL) throws -> Void
    /// Runs synchronously on the translation job's worker thread for every
    /// progress callback, before it reaches `translationProgress` (`NEN-102`).
    /// Test-only: there is no FFI-exposed way to pause the mock provider
    /// mid-block, so a test pauses here instead, the same rendezvous shape
    /// `core/crates/nen-ffi/tests/translation_gate.rs`'s `PausingSink` uses.
    public typealias TranslationProgressObserver = @Sendable (FfiTranslationProgress) -> Void

    @Published public private(set) var mediaName: String?
    @Published public private(set) var recentMedia: [RecentMediaEntry] = []
    @Published public private(set) var playbackState: FfiPlaybackState = .idle
    @Published public private(set) var positionMilliseconds: UInt64 = 0
    @Published public private(set) var durationMilliseconds: UInt64?
    @Published public private(set) var volume: Float = 1
    @Published public private(set) var playbackRate: Float = 1
    @Published public private(set) var fatalMessage: String?
    @Published public private(set) var transientMessage: String?
    @Published public private(set) var controlsVisible = true
    /// Mirrors the player window's `NSWindow` full-screen style mask
    /// (NEN-047). Has no effect on playback itself — it exists so `Esc` can
    /// be scoped to full screen only in `PlayerCommands`, and so that scoping
    /// is testable without reading AppKit window state.
    @Published public private(set) var isFullScreen = false
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
    /// The AI translation target language (§9, `NEN-101`), a plain primary
    /// subtag on the same terms as `subtitlePreferences` — published so the
    /// Settings picker and the "Altyazı" command stay in the same frame.
    @Published public private(set) var translationTargetLanguage: String?
    /// Whether a translation job started by `translateSelectedSubtitle()` is
    /// still running. The command's own gate — `NEN-102` adds progress and
    /// cancellation on top, not a second flag.
    @Published public private(set) var isTranslating = false
    /// The running job's latest progress, or `nil` when no job is running
    /// (`NEN-102`). `PlayerRootView` shows this in the same transient-pill
    /// slot the command's own start/finish messages already use — no new
    /// permanent chrome.
    @Published public private(set) var translationProgress: TranslationProgressState?
    /// The display size of the picture being played, or `nil` when there is no
    /// picture (ADR-0038).
    ///
    /// Read from the session rather than kept from an event: the event carries
    /// no value on purpose, so this is the only copy and it cannot disagree
    /// with the engine. `nil` is a state — audio-only media, a medium that
    /// failed, nothing open — and it is what leaves the window free to resize.
    @Published public private(set) var videoGeometry: FfiVideoGeometry?

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

    /// The one gate `translateSelectedSubtitle()` and the "Altyazı" command
    /// share — no source, a source already in the target language, a
    /// non-translatable or broken row, no target language chosen, or a job
    /// already running all disable it (§9, ADR-0031 Karar 4.3's family: a
    /// selection never fires a translation on its own, only this explicit
    /// command does).
    public var canTranslateSelectedSubtitle: Bool {
        guard !isTranslating, let target = translationTargetLanguage else { return false }
        guard let token = selectedSubtitleToken, let entry = subtitleMenuEntry(for: token) else {
            return false
        }
        guard entry.defect == nil, entry.translatable, let sourceLanguage = entry.language else {
            return false
        }
        return Self.primarySubtag(of: sourceLanguage) != Self.primarySubtag(of: target)
    }
    public var durationText: String {
        PlaybackPresentation.duration(
            position: displayedPositionMilliseconds,
            total: durationMilliseconds,
            showsRemaining: showsRemainingTime
        )
    }
    public var elapsedTimeText: String {
        PlaybackPresentation.elapsed(position: displayedPositionMilliseconds)
    }
    public var trailingTimeText: String {
        PlaybackPresentation.trailingDuration(
            position: displayedPositionMilliseconds,
            total: durationMilliseconds,
            showsRemaining: showsRemainingTime
        )
    }

    /// The closed set exposed by NEN-067's in-player rate panel.
    public static let playbackRateOptions: [Float] = [0.5, 0.75, 1, 1.5, 2]

    /// Configures the subtitle picker as an explicit user-selection surface.
    ///
    /// `NSOpenPanel` resolves a selected symlink/alias before handing the URL
    /// to the application. Keep that choice explicit rather than relying on
    /// AppKit's default, while the filesystem gates still reject symlinks on
    /// paths the application discovers or receives directly.
    static func configureSubtitleFilePanel(_ panel: NSOpenPanel) {
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.canChooseFiles = true
        panel.allowedContentTypes = [.init(filenameExtension: "srt")].compactMap { $0 }
        panel.prompt = "Yükle"
        panel.resolvesAliases = true
    }

    /// The artifact store's default root (`NEN-101`) — the application's own
    /// Application Support directory, on the same terms as `ADR-0034`'s
    /// sandbox-free distribution: no security-scoped bookmark, because
    /// nothing outside the app's own directory is being touched.
    public static func defaultTranslationStoreRoot() -> URL {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? FileManager.default.temporaryDirectory
        return base.appendingPathComponent("NenPlayer", isDirectory: true)
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
    private let handoffEvidenceCollector: HandoffEvidenceCollector
    /// Where the two preferred languages live (NEN-037).
    ///
    /// Injected rather than read from `Locale` at the point of use: read
    /// statically, every menu test would depend on the language of the machine
    /// running it — green on a Turkish laptop and red on an English one, for a
    /// reason that has nothing to do with the code.
    private let preferenceStore: any SubtitlePreferenceStoring
    /// The languages the menu hoists and auto-selection looks for.
    ///
    /// Published so the Settings scene's pickers and the menu stay in the
    /// same frame — a picker that only wrote to the store would need its own
    /// mechanism to notice a change made elsewhere.
    @Published public private(set) var subtitlePreferences: SubtitleLanguagePreferences
    /// Where the AI translation target language lives (`NEN-101`), on the
    /// same "injected, not read from `Locale` at the point of use" terms as
    /// `preferenceStore`.
    private let translationPreferenceStore: any TranslationPreferenceStoring
    /// The artifact store's root directory (`ADR-0034`'s sandbox-free
    /// application — no security-scoped bookmark needed for a location the
    /// app itself owns). Not created until the first translation command:
    /// `FfiTranslationEngine(storeRoot:)` is only constructed inside
    /// `translateSelectedSubtitle()`.
    private let translationStoreRoot: URL
    /// The catalog and the documents behind it, owned by the core (NEN-025).
    ///
    /// Not behind a protocol, unlike `PlaybackSessionClient`: that one exists
    /// because a real playback session needs a real libmpv process. This one
    /// needs a filesystem and nothing else, so the tests drive the real thing
    /// and prove the real gates. `FfiTranslationEngine`/`FfiTranslationJob`
    /// follow the same reasoning — a filesystem and the deterministic mock
    /// provider, no process to fake.
    private let subtitles = FfiSubtitleLibrary()
    private var session: (any PlaybackSessionClient)?
    /// A medium (and its handoff start position, if any) that arrived before
    /// `attach(to:)` gave it something to open into — the ordinary case for a
    /// handoff that starts the app cold (NEN-080's `HandoffCoordinator`).
    private var pendingOpen: (url: URL, startPositionMs: UInt64?)?
    /// The handoff start position for the medium currently loading, applied
    /// once `apply(_:)` sees `.ready` (NEN-081). `⌘O` and every other opener
    /// leave this `nil`.
    private var pendingHandoffStartPositionMs: UInt64?
    private var pollTask: Task<Void, Never>?
    private var sidecarScanTask: Task<Void, Never>?
    private var handoffEvidenceTask: Task<Void, Never>?
    private var translationTask: Task<Void, Never>?
    /// Tracks the running job for `cancelTranslation()` (`NEN-102`). One
    /// instance reused for the model's lifetime — `begin()` resets it per
    /// job, `isTranslating`'s own gate guarantees only one job at a time.
    private let translationCancellation = TranslationCancellation()
    /// Test-only hook into every progress callback (`NEN-102`). `nil` in
    /// production — see `TranslationProgressObserver`'s own doc comment.
    private let translationProgressObserver: TranslationProgressObserver?
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
        preferenceStore: any SubtitlePreferenceStoring = UserDefaultsSubtitlePreferenceStore(),
        translationPreferenceStore: any TranslationPreferenceStoring = UserDefaultsTranslationPreferenceStore(),
        translationStoreRoot: URL = PlayerModel.defaultTranslationStoreRoot(),
        translationProgressObserver: TranslationProgressObserver? = nil,
        handoffEvidenceCollector: HandoffEvidenceCollector? = nil,
        sessionFactory: @escaping SessionFactory = { view in
            let engine = try MPVPlaybackEngine(videoView: view)
            return FfiPlaybackSession(engine: engine)
        }
    ) {
        self.recentStore = recentStore
        self.recentMedia = recentStore.entries
        self.sessionFactory = sessionFactory
        self.shouldPoll = startsPolling
        self.pollIntervalNanoseconds = pollIntervalNanoseconds
        self.controlsHideDelayNanoseconds = controlsHideDelayNanoseconds
        self.transientMessageDurationNanoseconds = transientMessageDurationNanoseconds
        self.seekGuardTimeoutNanoseconds = seekGuardTimeoutNanoseconds
        self.now = now
        self.managesCursor = managesCursor
        self.preferenceStore = preferenceStore
        self.subtitlePreferences = preferenceStore.preferences
        self.translationPreferenceStore = translationPreferenceStore
        self.translationTargetLanguage = translationPreferenceStore.targetLanguage
        self.translationStoreRoot = translationStoreRoot
        self.translationProgressObserver = translationProgressObserver
        if let handoffEvidenceCollector {
            self.handoffEvidenceCollector = handoffEvidenceCollector
        } else {
            let client = URLSessionRemoteEvidenceClient()
            self.handoffEvidenceCollector = { url in
                _ = try collectRemoteEvidence(client: client, url: url.absoluteString)
            }
        }
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
            if let pendingOpen {
                self.pendingOpen = nil
                openMedia(at: pendingOpen.url, startPositionMs: pendingOpen.startPositionMs)
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

    /// Opens a medium, whether it came from `⌘O`, a drop, the recents list,
    /// or a handoff from another application (NEN-080).
    ///
    /// Accepts a local file or a remote `http`/`https` URL — the same two
    /// shapes `nen_app::handoff` resolves a locator to. Everything else
    /// (`⌘O`'s own panel, drag-and-drop) already only ever hands over a file
    /// URL, so the remote branch is reached only from a handoff.
    ///
    /// `startPositionMs` is a handoff's business only (NEN-081); `⌘O`, the
    /// recents list and drag-and-drop all leave it `nil`. It is not applied
    /// here — the medium's duration is not knowable until it has loaded, so
    /// it waits in `pendingHandoffStartPositionMs` for `apply(_:)`'s `.ready`
    /// branch, the same event ADR-0042's own deferred seek answers to.
    public func openMedia(at url: URL, startPositionMs: UInt64? = nil) {
        guard url.isFileURL || Self.isSupportedRemoteURL(url) else {
            presentTransient(PlaybackPresentation.unsupportedMediaSourceMessage)
            return
        }
        guard let session else {
            pendingOpen = (url, startPositionMs)
            return
        }

        releaseSecurityScope()
        if url.isFileURL {
            hasSecurityScope = url.startAccessingSecurityScopedResource()
            accessedURL = url
        }
        mediaPresentationRevision &+= 1
        // A translation job started against the outgoing medium must not
        // keep running unwatched, spending a real provider's credit on a
        // result the revision guard below would discard anyway (user
        // decision, NEN-102: switching media cancels the running job).
        cancelTranslation()

        mediaName = url.lastPathComponent
        fatalMessage = nil
        transientMessage = nil
        resetSubtitleCatalog()
        // Every open replaces whatever position a previous, still-loading
        // medium was carrying — there is only ever one medium in flight.
        pendingHandoffStartPositionMs = startPositionMs
        if url.isFileURL {
            // A remote locator's `.path` is a URL path component, not a
            // filesystem path — there is no directory beside it to list.
            startSidecarScan(besides: url)
        }
        playbackState = .buffering
        positionMilliseconds = 0
        durationMilliseconds = nil
        // The outgoing medium's size says nothing about the incoming one's, and
        // an aspect lock left over from it would shape the window for a picture
        // that is no longer there. Cleared before the load, not after: the
        // incoming size arrives with its own announcement.
        videoGeometry = nil
        seekPreviewMilliseconds = nil
        releaseSeekGuard()
        controlsVisible = true
        playWhenReady = true
        setControlsPinned(false)

        do {
            // A remote locator is never written to history — its query string
            // can carry a token (NEN-042 → YAPILMAYACAK). The store itself
            // enforces this; the call stays unconditional so a file handed
            // over by another application gets the same recents entry `⌘O`
            // would give it.
            try recentStore.save(url)
            recentMedia = recentStore.entries
        } catch {
            presentTransient(PlaybackPresentation.recentMediaSaveFailedMessage)
        }

        do {
            try session.load(locator: url.isFileURL ? url.path : url.absoluteString)
        } catch {
            playWhenReady = false
            presentFatal(error)
        }
    }

    /// `http`/`https` only — the same scheme gate
    /// `nen_app::remote_evidence::validate_url` enforces on the core side of a
    /// handoff. Kept narrow rather than "not a file URL", so a scheme this
    /// player has no engine support for does not reach `session.load`.
    private static func isSupportedRemoteURL(_ url: URL) -> Bool {
        guard let scheme = url.scheme?.lowercased() else { return false }
        return scheme == "http" || scheme == "https"
    }

    /// Applies what `HandoffIntake` decided a launch input meant (NEN-080).
    ///
    /// `.none` does nothing — an ordinary launch (no positional argument) is
    /// not a failure to react to. `.medium` opens exactly the way `⌘O` opens
    /// one, carrying its start position along (NEN-081); `.rejected` uses the
    /// same transient surface `openMedia(at:)` already shows for a source it
    /// refuses on its own.
    public func handleHandoff(_ outcome: HandoffOutcome) {
        switch outcome {
        case .none:
            break
        case let .medium(url, startPositionMs):
            startHandoffEvidenceCollection(for: url)
            openMedia(at: url, startPositionMs: startPositionMs)
        case .rejected:
            presentTransient(HandoffIntake.rejectionMessage)
        }
    }

    /// Starts optional remote evidence collection after the handoff has been
    /// accepted. The playback load is deliberately issued by the caller
    /// without waiting for this task; a missing header, unknown identity or a
    /// typed HTTP failure must never turn into a playback failure (NEN-082).
    private func startHandoffEvidenceCollection(for url: URL) {
        guard Self.isSupportedRemoteURL(url) else { return }
        handoffEvidenceTask?.cancel()
        let collector = handoffEvidenceCollector
        handoffEvidenceTask = Task {
            await Task.detached(priority: .utility) {
                do {
                    try collector(url)
                } catch {
                    // Evidence is optional. The typed error is intentionally
                    // contained here so playback remains independent of the
                    // remote metadata request (NEN-082).
                }
            }.value
        }
    }

    /// Waits for the optional evidence task. This is internal so platform
    /// tests can prove that the call happened without making playback await it.
    func awaitHandoffEvidence() async {
        await handoffEvidenceTask?.value
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
        Self.configureSubtitleFilePanel(panel)
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
        // The catalog changed, so the menu is re-derived: a counter moving on
        // its own would leave the row the user just picked off the list they
        // are about to open (NEN-028).
        refreshSubtitleMenu()
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
                library.addSidecarsFor(mediaPath: path)
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
            primary: subtitlePreferences.primary,
            secondary: subtitlePreferences.secondary
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
            primary: subtitlePreferences.primary,
            secondary: subtitlePreferences.secondary
        ) else { return }
        selectSubtitle(token: token)
    }

    /// Changes the two preferred languages (NEN-037's Settings scene).
    ///
    /// Re-sorts the menu on the spot — Karar 4's whole point is that the
    /// preferred languages sit above the rest. It does **not** touch
    /// `hasAutoSelected` or the subtitle already on screen: ADR-0031
    /// Karar 4.3 runs auto-selection once, at the start of a medium, and a
    /// preference changed mid-playback must not silently swap what the user
    /// is reading.
    public func updateSubtitlePreferences(_ preferences: SubtitleLanguagePreferences) {
        let normalized = preferences.normalized()
        preferenceStore.save(normalized)
        subtitlePreferences = normalized
        refreshSubtitleMenu()
    }

    // MARK: - Translation (§9, NEN-101)

    /// Changes the AI translation target language (`NEN-101`'s Settings row).
    ///
    /// Same shape as `updateSubtitlePreferences`: writes through to the
    /// store and does not touch anything already on screen or already
    /// selected — only `translateSelectedSubtitle()`'s own explicit command
    /// starts a job.
    public func updateTranslationTargetLanguage(_ code: String?) {
        translationPreferenceStore.save(code)
        translationTargetLanguage = code
    }

    /// §9's explicit command: "AI ile <hedef dile> çevir."
    ///
    /// Starts a job, blocks on its result off the main actor (the same
    /// `Task.detached` shape as `startSidecarScan`), and on success catalogues
    /// the outcome — but **never selects it**: the row the user is reading
    /// stays exactly what it was (ADR-0031 Karar 4.3's family, §9's "zorla AI
    /// çıktısına geçilmez"). `NEN-102` adds progress (`translationProgress`)
    /// and cancellation (`cancelTranslation()`) on top of this same job —
    /// the progress relay only ever narrates the job, it never mutates what
    /// is drawn on screen (§10's progressive-publication ban).
    public func translateSelectedSubtitle() {
        guard canTranslateSelectedSubtitle,
            let target = translationTargetLanguage,
            let token = selectedSubtitleToken
        else { return }

        isTranslating = true
        translationProgress = nil
        translationCancellation.begin()
        let library = subtitles
        let storeRoot = translationStoreRoot
        let cancellation = translationCancellation
        let observer = translationProgressObserver
        let session = self.session
        // A medium switch must not let a job started against the previous
        // medium's catalog silently add a row to the new one's menu — the
        // same revision guard `mediaPresentationRevision`'s own doc comment
        // describes for in-window presentation state. `openMedia` also
        // cancels the job outright (NEN-102 user decision); this guard is
        // the second, independent line of defence `NEN-101` already had.
        let revision = mediaPresentationRevision

        let (stream, progressContinuation) = AsyncStream.makeStream(of: FfiTranslationProgress.self)
        let relay = TranslationProgressRelay(continuation: progressContinuation, observer: observer)

        translationTask?.cancel()
        translationTask = Task { [weak self] in
            async let outcome: TranslationJoinOutcome = Task.detached(priority: .utility) {
                defer { progressContinuation.finish() }
                // A cancel issued in the same main-actor turn as the command
                // (or won by a race arriving before the store even opens)
                // must never reach the artifact store or a provider at all —
                // `translationCancellation`'s own doc comment.
                guard !cancellation.isCancelled else {
                    return .joinFailed(.Cancelled)
                }
                // `NEN-044`: an embedded row has no document until something
                // asks the engine to extract one. This is that ask, run here
                // — off the main actor, before the store or the engine is
                // touched — so a selected embedded track no longer refuses
                // with `NoDocument` for having never been read. A row that
                // already has a document (a user file, an AI translation, or
                // an embedded row a previous call already extracted) returns
                // `.ready` immediately without decoding anything again.
                guard let session else {
                    return .startFailed(.Unusable)
                }
                do {
                    _ = try session.prepareEmbeddedDocument(library: library, token: token)
                } catch let prepareError as FfiEmbeddedDocumentError {
                    return .prepareFailed(prepareError)
                } catch {
                    return .prepareFailed(.EngineFailure)
                }
                do {
                    // `FfiTranslationEngine` opens the root with `canonicalize`
                    // (`nen-persist`'s `FilesystemArtifactStore::new`), which
                    // fails if the root itself does not exist yet — unlike its
                    // own `artifacts/` subdirectory, the root is this shell's
                    // to create, on the first command a fresh install ever
                    // runs (measured on the real .app, NEN-101).
                    guard (try? FileManager.default.createDirectory(
                        at: storeRoot,
                        withIntermediateDirectories: true
                    )) != nil else {
                        return .startFailed(.StoreUnavailable)
                    }
                    let engine = try FfiTranslationEngine(storeRoot: storeRoot.path)
                    let job = try engine.start(
                        library: library,
                        token: token,
                        targetLanguage: target,
                        sink: relay
                    )
                    cancellation.adopt(job)
                    do {
                        _ = try job.join()
                        return .succeeded(job)
                    } catch let joinError as FfiTranslationError {
                        return .joinFailed(joinError)
                    } catch {
                        return .joinFailed(.Failed)
                    }
                } catch let startError as FfiTranslationStartError {
                    return .startFailed(startError)
                } catch {
                    return .startFailed(.Unusable)
                }
            }.value

            // Counts blocks by their own boundary — the first progress event
            // of each block is `Preparing` with `done == 0`
            // (`checkpoint.rs`'s own contract) — rather than trusting a
            // counter the provider itself could get wrong.
            var currentBlock = 0
            for await progress in stream {
                guard !Task.isCancelled, let self else { break }
                if progress.phase == .preparing, progress.done == 0 {
                    currentBlock += 1
                } else if currentBlock == 0 {
                    currentBlock = 1
                }
                guard self.mediaPresentationRevision == revision else { continue }
                self.translationProgress = TranslationProgressState(
                    phase: progress.phase,
                    block: currentBlock,
                    done: progress.done,
                    total: progress.total
                )
            }

            let result = await outcome
            guard !Task.isCancelled, let self else { return }
            self.isTranslating = false
            self.translationProgress = nil
            guard self.mediaPresentationRevision == revision else { return }
            switch result {
            case let .succeeded(job):
                _ = job.catalogInto(library: library)
                self.refreshSubtitleMenu()
            case let .prepareFailed(error):
                self.presentTransient(PlaybackPresentation.prepareEmbeddedDocumentMessage(for: error))
            case let .startFailed(error):
                self.presentTransient(PlaybackPresentation.translationStartMessage(for: error))
            case let .joinFailed(error):
                self.presentTransient(PlaybackPresentation.translationJoinMessage(for: error))
            }
        }
    }

    /// §9's explicit cancellation counterpart, `NEN-102`.
    ///
    /// `nonisolated`, deliberately: cancellation must be requestable without
    /// waiting for the main actor, from the transient pill's `İptal` button,
    /// the `Altyazı` menu's replacement command, or a test that is holding
    /// the worker thread inside a progress callback. Idempotent — safe to
    /// call with no job running, before a job has started, or after one has
    /// already finished (`FfiTranslationJob.cancel()`'s own doc comment).
    /// UI state (`isTranslating`, `translationProgress`) is not touched
    /// here; it is cleared once, on the main actor, when
    /// `translateSelectedSubtitle()`'s own task observes the job's outcome —
    /// the single writer `openMedia`'s revision guard already relies on.
    public nonisolated func cancelTranslation() {
        translationCancellation.cancel()
    }

    /// Waits for the running translation job. For tests only, and internal
    /// on the same terms as `awaitSidecarScan`.
    func awaitTranslation() async {
        await translationTask?.value
    }

    private func subtitleMenuEntry(for token: UInt32) -> FfiMenuEntry? {
        subtitleMenu
            .first { $0.entries.contains { $0.token == token } }?
            .entries
            .first { $0.token == token }
    }

    /// Reduces a BCP-47-ish tag to its primary subtag — `en` for `en-us` —
    /// the same granularity `nen_domain::source::LanguageTag::primary_tag`
    /// groups on (ADR-0030), applied on the shell side the way
    /// `UserDefaultsSubtitlePreferenceStore`'s seed already does.
    private static func primarySubtag(of tag: String) -> String {
        String(tag.split(separator: "-", maxSplits: 1).first ?? Substring(tag)).lowercased()
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

    public func openRecentMedia(_ id: RecentMediaEntry.ID) {
        do {
            guard let url = try recentStore.resolve(id) else {
                recentStore.remove(id)
                recentMedia = recentStore.entries
                presentTransient(PlaybackPresentation.recentMediaUnavailableMessage)
                return
            }
            openMedia(at: url)
        } catch {
            recentStore.remove(id)
            recentMedia = recentStore.entries
            presentTransient(PlaybackPresentation.recentMediaUnavailableMessage)
        }
    }

    /// The command behind the File menu's "Son Açılanları Temizle" — a no-op
    /// on an already-empty list.
    public func clearRecentMedia() {
        recentStore.clear()
        recentMedia = recentStore.entries
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
        // A keyboard seek moves the playhead with no on-screen sign of it
        // once the transport has faded — echo it the same way pointer
        // movement does (NEN-047).
        pointerMoved()
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

    public func setPlaybackRate(_ rate: Float) {
        guard Self.playbackRateOptions.contains(rate) else {
            presentTransient("Bu oynatma hızı kullanılamıyor.")
            return
        }
        guard let session else { return }
        do {
            try session.setRate(rate: rate)
            playbackRate = rate
        } catch {
            presentTransient(PlaybackPresentation.errorMessage(for: error))
        }
    }

    public func adjustVolume(by delta: Float) {
        setVolume(volume + delta)
        // Same reasoning as `seekRelative`: a keyboard volume nudge is
        // otherwise silent on screen once the transport has faded (NEN-047).
        pointerMoved()
    }

    public func toggleDurationMode() {
        showsRemainingTime.toggle()
    }

    /// Mirrors the player window's full-screen style mask so `PlayerCommands`
    /// can scope `Esc` to full screen only (NEN-047).
    public func setFullScreen(_ isFullScreen: Bool) {
        self.isFullScreen = isFullScreen
    }

    public func pointerMoved() {
        showControls()
        if isPlaying, !controlsPinned {
            scheduleControlsHide()
        }
    }

    public func pointerLeft() {
        guard isPlaying, !controlsPinned else {
            showControls()
            return
        }

        // Leaving the player is an immediate boundary: do not leave the
        // delayed hide task alive to race a later pointer event, and do not
        // hide the system cursor once it has left our surface. The view's
        // animation on `controlsVisible` keeps the existing 0.24 s fade.
        controlsTask?.cancel()
        controlsTask = nil
        controlsVisible = false
        showCursorIfNeeded()
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
        handoffEvidenceTask?.cancel()
        handoffEvidenceTask = nil
        translationTask?.cancel()
        translationTask = nil
        cancelTranslation()
        isTranslating = false
        translationProgress = nil
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
        videoGeometry = nil
        seekPreviewMilliseconds = nil
        releaseSeekGuard()
        // The session it targeted is gone, so nothing is left to seek
        // (ADR-0042 Karar 4's own reasoning, carried over — NEN-081).
        pendingHandoffStartPositionMs = nil
        fatalMessage = nil
        transientMessage = nil
        playWhenReady = false
        controlsVisible = true
        playbackRate = 1
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
            case .videoGeometryChanged:
                refreshVideoGeometry()
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
        if state == .failed || state == .idle {
            // Nothing is left to apply a still-pending handoff position to —
            // the load it targeted either failed or was stopped before it
            // answered (ADR-0042 Karar 4's own reasoning, carried over).
            pendingHandoffStartPositionMs = nil
        }
        // The tracks exist the moment the file is loaded, which is what `ready`
        // means here. Cataloguing before play keeps ADR-0031 Karar 4's promise
        // literal: the menu holds `Kapalı` plus the embedded tracks from the
        // first frame, with no scan having finished.
        if state == .ready {
            catalogEmbeddedTracks()
            // Every medium that opens gets one redraw, whether or not it has a
            // picture — because the one that has none would otherwise get no
            // redraw at all. mpv asks for a frame only when it has produced
            // one, so an audio-only medium leaves the previous medium's last
            // frame standing on the surface (NEN-069).
            //
            // Unconditional rather than guarded by `videoGeometry == nil`:
            // this property is cleared before every load and re-read only when
            // VIDEO_RECONFIG says to (ADR-0038 Karar 2), and that event has
            // not arrived yet — so here it reads `nil` for a medium with a
            // picture exactly as it does for one without, and the guard would
            // pin nothing. One redraw per load costs a single render of the
            // frame mpv already holds.
            videoView?.redrawWithoutNewFrame()
            // Before `playWhenReady` turns into `session.play()` below — a
            // handoff position is where the medium starts, not somewhere it
            // jumps to after a frame of playing from the beginning (NEN-081).
            applyHandoffStartPosition()
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

    /// Asks the session for the size the engine is actually showing.
    ///
    /// Swallows a refusal rather than reporting it: a session that cannot
    /// answer is one that is shutting down or holds no medium, and the honest
    /// consequence is the same either way — no size, so no aspect lock. The
    /// user has nothing to do about it and ADR-0031 Karar 1 has no class for
    /// it.
    private func refreshVideoGeometry() {
        guard let session else {
            videoGeometry = nil
            return
        }
        videoGeometry = (try? session.videoGeometry()) ?? nil
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
            // The announcement may be among the events that were dropped, and
            // it is the only thing that would ever have told the shell to look
            // (ADR-0038 Karar 2). Re-reading here is what keeps the window's
            // aspect lock true after a loss instead of frozen at the last size
            // that happened to get through.
            videoGeometry = try session.videoGeometry()
            _ = try session.tracks(kind: .audio)
            catalogEmbeddedTracks()
        } catch {
            Self.logger.debug("Playback resynchronization found no readable state")
        }
    }

    /// Issues a seek and starts the guard NEN-053 depends on.
    ///
    /// `drainingEvents` is `false` only for NEN-081's handoff start position:
    /// that call runs from inside `apply(_:)`, itself reached from inside
    /// `consume(_:)`'s loop over one batch of events — draining again there
    /// would feed a second, overlapping batch into the same loop before it
    /// has finished the first. Every other caller (`seekRelative`,
    /// `commitSeek`) drains immediately: NEN-055 measured the answer as
    /// already queued by the time the command returns.
    ///
    /// `presentsErrorOnFailure` is `false` for the same handoff path: a
    /// rejected handoff position is not the user's own action, so ADR-0031
    /// Karar 1 has no class for it — the medium simply keeps playing from
    /// wherever it already is.
    private func seek(
        to milliseconds: UInt64,
        drainingEvents: Bool = true,
        presentsErrorOnFailure: Bool = true
    ) {
        guard let session, hasMedia else { return }
        do {
            try session.seek(toMs: milliseconds)
            positionMilliseconds = milliseconds
            pendingSeekCount += 1
            seekGuardExpiry = now() + seekGuardTimeoutNanoseconds
            if drainingEvents {
                // Safe only because of the guard above: what the queue is
                // holding at this instant is the position from *before* the
                // seek, which is exactly what NEN-053 measured and now
                // refuses.
                drainSessionEvents()
            }
        } catch {
            // A refused seek is owed no answer, so it must not leave a guard
            // behind: the position that keeps arriving is the true one.
            releaseSeekGuard()
            if presentsErrorOnFailure {
                presentTransient(PlaybackPresentation.errorMessage(for: error))
            }
        }
    }

    /// Applies the position a handoff carried, once the medium it targets has
    /// finished loading (NEN-081).
    ///
    /// Called from `apply(_:)`'s `.ready` branch rather than riding
    /// ADR-0042's own deferred-seek mechanism at `load` time: the duration
    /// this needs to judge the position against is not knowable until the
    /// medium has loaded, since mpv itself only learns it then (see
    /// ADR-0043's Notlar).
    private func applyHandoffStartPosition() {
        guard let target = pendingHandoffStartPositionMs else { return }
        pendingHandoffStartPositionMs = nil
        // A duration this session cannot read (a live stream, or a session
        // that is already gone) has nothing to exceed — the position is
        // trusted rather than dropped for a question the medium cannot
        // answer. A duration it *can* read and the position meets or passes
        // is exactly ADR-0043 Karar 2's "süreyi aşan değer sessizce düşer":
        // the medium keeps playing from where the load already put it,
        // no error surface.
        if let duration = try? session?.durationMs(), target >= duration {
            return
        }
        seek(to: target, drainingEvents: false, presentsErrorOnFailure: false)
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
        // Nothing is being shown, so nothing constrains the window: the fatal
        // state is one of the three ADR-0038 leaves free to resize.
        videoGeometry = nil
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
