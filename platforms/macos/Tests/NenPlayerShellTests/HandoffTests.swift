import AppKit
import Foundation
import NenCore
import NenPlaybackMPV
import Testing
@testable import NenPlayerShell

/// `HandoffIntake` — the pure translation from what the core decided a
/// launch input meant into what `PlayerModel` can act on (NEN-080).
@Suite("handoff intake")
struct HandoffIntakeTests {
    @Test("a plain launch with no positional argument is not a failure")
    func plainLaunchIsNotAHandoff() {
        #expect(HandoffIntake.fromArgv(["/Applications/Nen Player.app/Contents/MacOS/NenPlayer"]) == .none)
    }

    @Test("an argv locator becomes a local medium to open")
    func argvLocatorBecomesAMedium() {
        let outcome = HandoffIntake.fromArgv([
            "/Applications/Nen Player.app/Contents/MacOS/NenPlayer",
            "--no-terminal",
            "/Users/x/Film.mkv",
        ])
        #expect(outcome == .medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: nil))
    }

    @Test("an unsupported argv scheme is a rejection the user is shown")
    func unsupportedArgvSchemeIsRejected() {
        let outcome = HandoffIntake.fromArgv([
            "/Applications/Nen Player.app/Contents/MacOS/NenPlayer",
            "ftp://example.test/a.mkv",
        ])
        #expect(outcome == .rejected(PlaybackPresentation.unsupportedMediaSourceMessage))
    }

    @Test("an opened document's file URL becomes a local medium")
    func openedDocumentBecomesAMedium() {
        let outcome = HandoffIntake.fromURL("file:///Users/x/Film.mkv")
        #expect(outcome == .medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: nil))
    }

    @Test("the nenplayer scheme carries a remote URL and its time fragment")
    func schemeCarriesRemoteURLAndFragment() {
        let outcome = HandoffIntake.fromURL("nenplayer://https://example.test/a/Film.mkv#t=90")
        #expect(outcome == .medium(URL(string: "https://example.test/a/Film.mkv")!, startPositionMs: 90_000))
    }

    @Test("a malformed opened-document URL is a rejection, not a crash")
    func malformedOpenedDocumentIsRejected() {
        #expect(HandoffIntake.fromURL("not a url at all") == .rejected(PlaybackPresentation.unsupportedMediaSourceMessage))
    }
}

/// `HandoffCoordinator` — reconciling a handoff with `PlayerModel` not
/// existing yet, and with AppKit's own two entry points *not* firing in
/// launch order.
@Suite("handoff coordinator")
@MainActor
struct HandoffCoordinatorTests {
    @Test("a handoff queued before the model exists plays once attached")
    func queuedHandoffPlaysOnceAttached() {
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        let coordinator = HandoffCoordinator()

        coordinator.deliver(.medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: nil))
        #expect(fixture.loadedLocators.isEmpty)

        coordinator.attach(model)
        model.attach(to: MPVVideoView.makePlaybackSurface())

        #expect(fixture.loadedLocators == ["/Users/x/Film.mkv"])
    }

    @Test("a real handoff queued first is not erased by a later ordinary launch")
    func realHandoffSurvivesALaterNone() {
        // The bug this type exists to prevent: measured on a real cold
        // open-with launch, `application(_:open:)` (a real handoff) fires
        // BEFORE `applicationDidFinishLaunching` (whose argv reads as `.none`
        // for that same launch, since `open` does not forward the file as a
        // positional argument). Without this ordering rule the `.none`
        // silently overwrote the medium and the app opened empty.
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        let coordinator = HandoffCoordinator()

        coordinator.deliver(.medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: nil))
        coordinator.deliver(.none)
        coordinator.attach(model)
        model.attach(to: MPVVideoView.makePlaybackSurface())

        #expect(fixture.loadedLocators == ["/Users/x/Film.mkv"])
    }

    @Test("a second real handoff queued before attach replaces the first")
    func secondRealHandoffBeforeAttachWins() {
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        let coordinator = HandoffCoordinator()

        coordinator.deliver(.medium(URL(fileURLWithPath: "/Users/x/First.mkv"), startPositionMs: nil))
        coordinator.deliver(.medium(URL(fileURLWithPath: "/Users/x/Second.mkv"), startPositionMs: nil))
        coordinator.attach(model)
        model.attach(to: MPVVideoView.makePlaybackSurface())

        #expect(fixture.loadedLocators == ["/Users/x/Second.mkv"])
    }

    @Test("attaching with nothing queued does not touch the model")
    func attachWithNothingQueuedDoesNothing() {
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        let coordinator = HandoffCoordinator()

        coordinator.deliver(.none)
        coordinator.attach(model)
        model.attach(to: MPVVideoView.makePlaybackSurface())

        #expect(fixture.loadedLocators.isEmpty)
        #expect(model.mediaName == nil)
    }

    @Test("a handoff arriving after attach reaches the model directly")
    func handoffAfterAttachGoesStraightThrough() {
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())
        let coordinator = HandoffCoordinator()
        coordinator.attach(model)

        coordinator.deliver(.medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: nil))

        #expect(fixture.loadedLocators == ["/Users/x/Film.mkv"])
    }
}

/// `PlayerModel.handleHandoff` — what NEN-080's DoD actually asks for: the
/// medium plays "`⌘O` ile açılmış gibi aynı yoldan", whether the app was
/// already running or the handoff is what started it.
@Suite("handoff reaches the player")
@MainActor
struct PlayerModelHandoffTests {
    @Test("a remote handoff collects optional evidence without delaying playback")
    func remoteHandoffCollectsEvidenceOffThePlaybackPath() async {
        let fixture = FakeSession()
        let recorder = HandoffEvidenceRecorder()
        let remote = URL(string: "https://media.invalid/movies/Film.2010/stream.mkv")!
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            handoffEvidenceCollector: { url in try recorder.collect(url) },
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.handleHandoff(.medium(remote, startPositionMs: nil))

        // The load is synchronous and observable immediately; evidence is not
        // allowed to become a prerequisite for playback.
        #expect(fixture.loadedLocators == [remote.absoluteString])
        await model.awaitHandoffEvidence()
        #expect(recorder.urls == [remote])
    }

    @Test("a typed evidence failure leaves the handoff playable")
    func evidenceFailureDoesNotBecomeAPlaybackFailure() async {
        let fixture = FakeSession()
        let recorder = HandoffEvidenceRecorder()
        recorder.fails = true
        let first = URL(string: "https://media.invalid/opaque/0")!
        let second = URL(string: "https://media.invalid/opaque/1")!
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            handoffEvidenceCollector: { url in try recorder.collect(url) },
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.handleHandoff(.medium(first, startPositionMs: nil))
        await model.awaitHandoffEvidence()
        model.handleHandoff(.medium(second, startPositionMs: nil))
        await model.awaitHandoffEvidence()

        #expect(fixture.loadedLocators == [first.absoluteString, second.absoluteString])
        #expect(model.fatalMessage == nil)
        #expect(model.transientMessage == nil)
    }

    @Test("local handoffs and ordinary opens do not collect remote evidence")
    func onlyRemoteHandoffsCollectEvidence() async {
        let fixture = FakeSession()
        let recorder = HandoffEvidenceRecorder()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            handoffEvidenceCollector: { url in try recorder.collect(url) },
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.handleHandoff(.medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: nil))
        model.openMedia(at: URL(string: "https://media.invalid/ordinary.mkv")!)
        await model.awaitHandoffEvidence()

        #expect(recorder.urls.isEmpty)
    }

    @Test("a handoff arriving while the app is already open plays immediately")
    func handoffWhileOpenPlaysImmediately() {
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.handleHandoff(.medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: nil))

        #expect(fixture.loadedLocators == ["/Users/x/Film.mkv"])
        #expect(model.mediaName == "Film.mkv")
    }

    @Test("a handoff that starts the app is not lost — it plays once the surface attaches")
    func handoffBeforeLaunchIsQueuedUntilAttach() {
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )

        // No `attach(to:)` yet — this is the state a cold launch is in the
        // moment argv is read, before the video surface exists.
        model.handleHandoff(.medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: nil))
        #expect(fixture.loadedLocators.isEmpty)

        model.attach(to: MPVVideoView.makePlaybackSurface())

        #expect(fixture.loadedLocators == ["/Users/x/Film.mkv"])
        #expect(model.mediaName == "Film.mkv")
    }

    @Test("a remote handoff locator loads by URL, not by filesystem path")
    func remoteHandoffLoadsByURL() {
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())
        let url = URL(string: "https://example.test/a/Film.mkv")!

        model.handleHandoff(.medium(url, startPositionMs: 90_000))

        #expect(fixture.loadedLocators == ["https://example.test/a/Film.mkv"])
        #expect(model.mediaName == "Film.mkv")
    }

    @Test("a rejected handoff shows the same message a refused source shows")
    func rejectedHandoffShowsTheOrdinaryMessage() {
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.handleHandoff(.rejected(PlaybackPresentation.unsupportedMediaSourceMessage))

        #expect(model.transientMessage == PlaybackPresentation.unsupportedMediaSourceMessage)
        #expect(fixture.loadedLocators.isEmpty)
    }

    @Test("an ordinary launch with no handoff does nothing")
    func noHandoffDoesNothing() {
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.handleHandoff(.none)

        #expect(fixture.loadedLocators.isEmpty)
        #expect(model.mediaName == nil)
        #expect(model.transientMessage == nil)
    }

    @Test("openMedia itself accepts a remote http/https URL, not only a file URL")
    func openMediaAcceptsRemoteURL() {
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.openMedia(at: URL(string: "http://example.test/a/Film.mkv")!)

        #expect(fixture.loadedLocators == ["http://example.test/a/Film.mkv"])
        #expect(model.transientMessage == nil)
    }

    @Test("openMedia still refuses a scheme neither a file nor http/https")
    func openMediaRefusesUnsupportedScheme() {
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.openMedia(at: URL(string: "ftp://example.test/a.mkv")!)

        #expect(fixture.loadedLocators.isEmpty)
        #expect(model.transientMessage == PlaybackPresentation.unsupportedMediaSourceMessage)
    }

    @Test("a second handoff while a medium is already playing opens the new one, ⌘O's own way")
    func secondHandoffWhileAlreadyOpenReplacesTheMedium() {
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())
        model.openMedia(at: URL(fileURLWithPath: "/Users/x/First.mkv"))
        model.consume([.stateChanged(state: .ready)])

        model.handleHandoff(.medium(URL(fileURLWithPath: "/Users/x/Second.mkv"), startPositionMs: nil))

        #expect(fixture.loadedLocators == ["/Users/x/First.mkv", "/Users/x/Second.mkv"])
        #expect(model.mediaName == "Second.mkv")
    }
}


/// The start position a handoff carries, applied once the medium it targets
/// has loaded (NEN-081). ADR-0043 Karar 2 and its Notlar: the position rides
/// through `PlayerModel`, not `PlaybackSessionClient`'s own load call — the
/// duration it must be judged against is unknowable before then.
@Suite("handoff start position")
@MainActor
struct HandoffStartPositionTests {
    @Test("a handoff's start position seeks before playback begins")
    func startPositionSeeksBeforePlay() {
        let fixture = FakeSession()
        fixture.currentDuration = 30_008
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.handleHandoff(.medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: 12_000))
        model.consume([.stateChanged(state: .ready)])

        #expect(fixture.seekTargets == [12_000])
        #expect(fixture.callOrder == [.seek, .play])
    }

    @Test("a handoff without a start position never seeks")
    func noStartPositionNeverSeeks() {
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.handleHandoff(.medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: nil))
        model.consume([.stateChanged(state: .ready)])

        #expect(fixture.seekTargets.isEmpty)
        #expect(fixture.callOrder == [.play])
    }

    @Test("a start position at or past the medium's duration is dropped, no error shown")
    func startPositionPastDurationIsDropped() {
        let fixture = FakeSession()
        fixture.currentDuration = 30_008
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.handleHandoff(.medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: 999_999))
        model.consume([.stateChanged(state: .ready)])

        #expect(fixture.seekTargets.isEmpty)
        #expect(model.transientMessage == nil)
        #expect(model.mediaName == "Film.mkv")
    }

    @Test("a start position exactly at the duration is dropped too")
    func startPositionAtDurationIsDropped() {
        let fixture = FakeSession()
        fixture.currentDuration = 30_008
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.handleHandoff(.medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: 30_008))
        model.consume([.stateChanged(state: .ready)])

        #expect(fixture.seekTargets.isEmpty)
    }

    @Test("a start position is applied even when the medium's duration cannot be read")
    func startPositionAppliesWhenDurationIsUnreadable() {
        let fixture = FakeSession()
        // A live stream — ADR-0038 Karar 1's own `None` for "no duration".
        fixture.currentDuration = nil
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.handleHandoff(.medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: 999_999))
        model.consume([.stateChanged(state: .ready)])

        #expect(fixture.seekTargets == [999_999])
    }

    @Test("opening media the ordinary way never carries a start position")
    func ordinaryOpenNeverSeeksForAStartPosition() {
        let fixture = FakeSession()
        fixture.currentDuration = 30_008
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.openMedia(at: URL(fileURLWithPath: "/Users/x/Film.mkv"))
        model.consume([.stateChanged(state: .ready)])

        #expect(fixture.seekTargets.isEmpty)
    }

    @Test("a load that fails drops the handoff position it was carrying")
    func failedLoadDropsTheHandoffPosition() {
        let fixture = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.handleHandoff(.medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: 12_000))
        // The load itself failed — nothing this position could still land on.
        model.consume([.failed(error: FfiPlaybackError.EngineFailure(code: -1))])
        // A later, unrelated `ready` (e.g. from whatever the user opens next)
        // must not replay a position that belonged to the failed load.
        model.consume([.stateChanged(state: .ready)])

        #expect(fixture.seekTargets.isEmpty)
    }

    @Test("a second handoff's start position replaces the first, still-loading one")
    func secondHandoffReplacesThePendingPosition() {
        let fixture = FakeSession()
        fixture.currentDuration = 30_008
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.handleHandoff(.medium(URL(fileURLWithPath: "/Users/x/First.mkv"), startPositionMs: 5_000))
        model.handleHandoff(.medium(URL(fileURLWithPath: "/Users/x/Second.mkv"), startPositionMs: 15_000))
        model.consume([.stateChanged(state: .ready)])

        #expect(fixture.seekTargets == [15_000])
    }

    @Test("a start position that arrives before the surface attaches is still applied once it does")
    func startPositionSurvivesTheColdLaunchQueue() {
        let fixture = FakeSession()
        fixture.currentDuration = 30_008
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )

        // No `attach(to:)` yet — the state a cold, handoff-driven launch is in
        // the moment the position is read.
        model.handleHandoff(.medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: 12_000))
        #expect(fixture.loadedLocators.isEmpty)

        model.attach(to: MPVVideoView.makePlaybackSurface())
        model.consume([.stateChanged(state: .ready)])

        #expect(fixture.seekTargets == [12_000])
    }

    @Test("a start position the engine refuses shows no error and does not block playback")
    func refusedStartPositionShowsNoError() {
        let fixture = FakeSession()
        fixture.currentDuration = 30_008
        fixture.errors[.seek] = .EngineFailure(code: -12)
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in fixture }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.handleHandoff(.medium(URL(fileURLWithPath: "/Users/x/Film.mkv"), startPositionMs: 12_000))
        model.consume([.stateChanged(state: .ready)])

        #expect(model.transientMessage == nil)
        #expect(fixture.callOrder == [.play])
    }
}


/// The same claim as `HandoffStartPositionTests`, against the real libmpv
/// adapter rather than `FakeSession` — the DoD's own demand: `NEN-052`'s
/// measured load window (2.5–12 ms) is exactly what a handoff's position
/// races, and a fake engine cannot prove a race was actually won.
///
/// No prior shell test drives `PlayerModel` through its real
/// `sessionFactory` default; `PicturelessSurfaceTests.windowedSurface()`
/// establishes that a real engine attaches to a real, windowed
/// `MPVVideoView` in this test process, so the same shape is reused here.
@Suite("handoff start position — real libmpv")
@MainActor
struct HandoffStartPositionRealEngineTests {
    /// `PicturelessSurfaceTests.windowedSurface()`'s reasoning applies here
    /// too: a view outside a window never becomes drawable, and the product
    /// always attaches the real engine to a windowed surface.
    private func windowedSurface() -> (NSWindow, MPVVideoView) {
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 640, height: 360),
            styleMask: [.borderless],
            backing: .buffered,
            defer: false
        )
        let view = MPVVideoView.makePlaybackSurface()
        view.frame = NSRect(x: 0, y: 0, width: 640, height: 360)
        window.contentView?.addSubview(view)
        return (window, view)
    }

    /// Both real-engine claims in one test, one medium reopened rather than
    /// two separate real mpv processes: `swift test` parallelizes across
    /// suites, and every extra concurrent real engine + windowed surface adds
    /// to the same main-actor timing contention `NEN-049`/`NEN-066` already
    /// documented in this suite's neighbours — one is enough to prove both
    /// claims against the real adapter.
    @Test("a handoff's start position lands on the real engine, and a past-duration one is dropped")
    func startPositionAgainstTheRealEngine() async throws {
        let (window, view) = windowedSurface()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            pollIntervalNanoseconds: 1_000_000,
            managesCursor: false
        )
        model.attach(to: view)
        let fixture = URL(fileURLWithPath: SubtitleSafeAreaTests.fixturePath("contract-clip.mkv"))

        // A position within the fixture's own measured 30_008 ms duration.
        model.handleHandoff(.medium(fixture, startPositionMs: 12_000))

        // `NEN-052`/`NEN-049`'s own lesson, applied to this shell's own
        // position property: "ready" is not "landed", so the assertion waits
        // for the position itself rather than reading it once.
        let landingDeadline = Date().addingTimeInterval(10)
        while !(model.positionMilliseconds > 11_900 && model.positionMilliseconds < 12_100),
            Date() < landingDeadline
        {
            try await Task.sleep(nanoseconds: 2_000_000)
        }
        #expect(
            model.positionMilliseconds > 11_900 && model.positionMilliseconds < 12_100,
            "landed at \(model.positionMilliseconds) ms"
        )

        // Reopening the same fixture with a position past its duration —
        // dropped silently, the medium plays from the start.
        model.handleHandoff(.medium(fixture, startPositionMs: 999_000))
        let reopenDeadline = Date().addingTimeInterval(10)
        while model.playbackState != .playing, Date() < reopenDeadline {
            try await Task.sleep(nanoseconds: 2_000_000)
        }
        #expect(model.playbackState == .playing)
        #expect(model.transientMessage == nil)
        // Never seeked past the fixture's own end: the position stayed near
        // the medium's start, nowhere near the requested (and refused) 999 s.
        #expect(model.positionMilliseconds < 5_000, "position \(model.positionMilliseconds) ms — should have opened near the start")

        model.shutdown()
        _ = window.isVisible
    }
}
