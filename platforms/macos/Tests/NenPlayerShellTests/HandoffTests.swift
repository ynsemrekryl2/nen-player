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
