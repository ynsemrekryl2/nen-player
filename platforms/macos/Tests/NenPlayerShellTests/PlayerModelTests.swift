import AppKit
import Foundation
import NenCore
import NenPlaybackMPV
import Testing
@testable import NenPlayerShell

@Suite("macOS player shell")
@MainActor
struct PlayerModelTests {
    @Test("opening media waits for Ready, then plays without discovery")
    func opensAndPlaysWhenReady() {
        let fixture = FakeSession()
        let store = MemoryRecentStore()
        let model = makeModel(session: fixture, store: store)
        let url = URL(fileURLWithPath: "/fixtures/media/contract-clip.mkv")

        model.openMedia(at: url)
        #expect(fixture.loadedLocators == [url.path])
        #expect(fixture.playCount == 0)
        #expect(model.windowTitle == "contract-clip.mkv")

        model.consume([.stateChanged(state: .ready)])
        #expect(fixture.playCount == 1)
        #expect(model.mediaName == "contract-clip.mkv")
    }

    @Test("transport commands clamp seeks and volume")
    func transportCommands() {
        let fixture = FakeSession()
        fixture.currentPosition = 4_000
        fixture.currentDuration = 30_000
        let model = makeModel(session: fixture)
        model.openMedia(at: URL(fileURLWithPath: "/fixtures/media/contract-clip.mkv"))
        model.consume([.stateChanged(state: .ready), .stateChanged(state: .paused)])

        model.seekRelative(seconds: -5)
        model.seekRelative(seconds: 30)
        model.setVolume(2)

        #expect(fixture.seekTargets == [0, 30_000])
        #expect(fixture.volumes == [1])
        #expect(model.volume == 1)
    }

    @Test("EventsLost silently refreshes state, position, duration, and tracks")
    func eventsLostResynchronizesEverything() {
        let fixture = FakeSession()
        fixture.currentState = .paused
        fixture.currentPosition = 12_000
        fixture.currentDuration = 30_008
        let model = makeModel(session: fixture)
        model.openMedia(at: URL(fileURLWithPath: "/fixtures/media/contract-clip.mkv"))

        model.consume([.eventsLost(dropped: 7)])

        #expect(model.playbackState == .paused)
        #expect(model.positionMilliseconds == 12_000)
        #expect(model.durationMilliseconds == 30_008)
        #expect(fixture.requestedTrackKinds == [.audio, .subtitle])
        #expect(model.transientMessage == nil)
    }

    @Test("fatal copy contains neither a path nor engine internals")
    func fatalCopyIsClosedAndRecoverable() {
        let fixture = FakeSession()
        let model = makeModel(session: fixture)
        let first = URL(fileURLWithPath: "/private/library/secret-name.mkv")
        model.openMedia(at: first)

        model.consume([.failed(error: .EngineFailure(code: -13))])
        #expect(model.fatalMessage == "Medya oynatılamadı.")
        #expect(model.windowTitle == "secret-name.mkv")
        #expect(model.fatalMessage?.contains("/private") == false)
        #expect(model.fatalMessage?.localizedCaseInsensitiveContains("mpv") == false)
        #expect(model.fatalMessage?.contains("13") == false)

        model.openMedia(at: URL(fileURLWithPath: "/fixtures/media/contract-clip.mkv"))
        #expect(model.fatalMessage == nil)
    }

    @Test("controls hide only while playing")
    func controlsVisibilityFollowsPlayback() {
        let fixture = FakeSession()
        let model = makeModel(session: fixture)
        model.openMedia(at: URL(fileURLWithPath: "/fixtures/media/contract-clip.mkv"))

        model.consume([.stateChanged(state: .playing)])
        model.hideControlsNow()
        #expect(model.controlsVisible == false)

        model.consume([.stateChanged(state: .paused)])
        model.hideControlsNow()
        #expect(model.controlsVisible)
    }

    @Test("the one stored bookmark reopens in the same model")
    func recentMediaReopens() {
        let fixture = FakeSession()
        let store = MemoryRecentStore()
        let url = URL(fileURLWithPath: "/fixtures/media/contract-clip.mkv")
        store.url = url
        let model = makeModel(session: fixture, store: store)

        model.openRecentMedia()

        #expect(fixture.loadedLocators == [url.path])
        #expect(model.recentMediaName == "contract-clip.mkv")
    }

    @Test("duration toggles between elapsed and remaining")
    func durationPresentation() {
        #expect(PlaybackPresentation.time(milliseconds: 65_000) == "01:05")
        #expect(PlaybackPresentation.time(milliseconds: 3_665_000) == "01:01:05")
        #expect(
            PlaybackPresentation.duration(position: 5_000, total: 65_000, showsRemaining: false)
                == "00:05 / 01:05"
        )
        #expect(
            PlaybackPresentation.duration(position: 5_000, total: 65_000, showsRemaining: true)
                == "−01:00 / 01:05"
        )
    }

    private func makeModel(
        session: FakeSession,
        store: MemoryRecentStore = MemoryRecentStore()
    ) -> PlayerModel {
        let model = PlayerModel(
            recentStore: store,
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in session }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())
        return model
    }
}

private final class MemoryRecentStore: RecentMediaStoring {
    var url: URL?
    var displayName: String? { url?.lastPathComponent }

    func save(_ url: URL) throws {
        self.url = url
    }

    func resolve() throws -> URL? {
        url
    }

    func clear() {
        url = nil
    }
}

private final class FakeSession: PlaybackSessionClient {
    var loadedLocators: [String] = []
    var playCount = 0
    var pauseCount = 0
    var seekTargets: [UInt64] = []
    var volumes: [Float] = []
    var currentPosition: UInt64 = 0
    var currentDuration: UInt64? = 30_008
    var currentState: FfiPlaybackState = .ready
    var requestedTrackKinds: [FfiTrackKind] = []
    var events: [FfiSessionEvent] = []

    func load(locator: String) throws { loadedLocators.append(locator) }
    func play() throws { playCount += 1; currentState = .playing }
    func pause() throws { pauseCount += 1; currentState = .paused }
    func stop() throws { currentState = .idle }
    func seek(toMs: UInt64) throws { seekTargets.append(toMs); currentPosition = toMs }
    func positionMs() throws -> UInt64 { currentPosition }
    func durationMs() throws -> UInt64? { currentDuration }
    func state() throws -> FfiPlaybackState { currentState }
    func tracks(kind: FfiTrackKind) throws -> [FfiTrackDescriptor] {
        requestedTrackKinds.append(kind)
        return []
    }
    func setVolume(volume: Float) throws { volumes.append(volume) }
    func drainEvents() -> [FfiSessionEvent] {
        defer { events.removeAll() }
        return events
    }
    func shutdown() throws {}
}
