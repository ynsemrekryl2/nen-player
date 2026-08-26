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

    @Test("shutdown clears stale state and window resume opens pending media")
    func shutdownThenResumeOpensPendingMedia() {
        let first = FakeSession()
        let second = FakeSession()
        var sessions = [first, second]
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in sessions.removeFirst() }
        )
        let surface = MPVVideoView.makePlaybackSurface()
        model.attach(to: surface)
        model.openMedia(at: URL(fileURLWithPath: "/fixtures/media/first.mkv"))
        model.consume([.stateChanged(state: .ready), .stateChanged(state: .playing)])

        model.shutdown()

        #expect(model.mediaName == nil)
        #expect(model.playbackState == .idle)
        #expect(model.positionMilliseconds == 0)
        #expect(model.durationMilliseconds == nil)

        let next = URL(fileURLWithPath: "/fixtures/media/second.mkv")
        model.openMedia(at: next)
        #expect(second.loadedLocators.isEmpty)

        model.resume()

        #expect(first.shutdownCount == 1)
        #expect(second.loadedLocators == [next.path])
        #expect(model.mediaName == "second.mkv")
    }

    @Test("window resume after shutdown restarts event polling")
    func resumeRestartsPolling() async throws {
        let first = FakeSession()
        let second = FakeSession()
        var sessions = [first, second]
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            pollIntervalNanoseconds: 1_000_000,
            managesCursor: false,
            sessionFactory: { _ in sessions.removeFirst() }
        )
        let surface = MPVVideoView.makePlaybackSurface()
        model.attach(to: surface)
        model.shutdown()

        let next = URL(fileURLWithPath: "/fixtures/media/second.mkv")
        model.openMedia(at: next)
        second.events = [.stateChanged(state: .ready)]
        model.resume()

        let deadline = Date().addingTimeInterval(0.5)
        while second.playCount == 0, Date() < deadline {
            try await Task.sleep(nanoseconds: 1_000_000)
        }

        #expect(second.playCount == 1)
        #expect(model.playbackState == .ready)
        model.shutdown()
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

    // MARK: - Transient error class (ADR-0031 Karar 1)

    @Test("resynchronizing without media says nothing")
    func resyncWithoutMediaIsSilent() {
        let fixture = FakeSession()
        // Exactly the shipped chain: a session exists from `attach`, but no
        // media is loaded, so the engine refuses the position read.
        fixture.errors[.position] = .NotLoaded
        fixture.currentState = .idle
        let model = makeModel(session: fixture)

        model.applicationBecameActive()

        #expect(model.transientMessage == nil)
        #expect(model.fatalMessage == nil)
        #expect(model.mediaName == nil)
    }

    @Test("EventsLost resynchronization stays silent when the engine refuses")
    func eventsLostResyncIsSilent() {
        let fixture = FakeSession()
        fixture.errors[.position] = .NotLoaded
        let model = makeModel(session: fixture)

        model.consume([.eventsLost(dropped: 4)])

        #expect(model.transientMessage == nil)
        #expect(model.fatalMessage == nil)
    }

    @Test("a refused seek is reported transiently, not fatally")
    func refusedSeekIsTransient() {
        let fixture = FakeSession()
        let model = makePlayingModel(session: fixture)
        fixture.errors[.seek] = .NotLoaded

        model.seekRelative(seconds: 10)

        #expect(model.transientMessage == "Önce bir medya açın.")
        #expect(model.fatalMessage == nil)
        #expect(model.playbackState != .failed)
    }

    @Test("a refused volume change is reported transiently, not fatally")
    func refusedVolumeIsTransient() {
        let fixture = FakeSession()
        let model = makePlayingModel(session: fixture)
        fixture.errors[.volume] = .Unsupported(capability: .volume)

        model.adjustVolume(by: -0.1)

        #expect(model.transientMessage == "Bu işlem desteklenmiyor.")
        #expect(model.fatalMessage == nil)
        #expect(model.volume == 1)
    }

    @Test("a refused play command is reported transiently, not fatally")
    func refusedPlaybackToggleIsTransient() {
        let fixture = FakeSession()
        let model = makePlayingModel(session: fixture)
        fixture.errors[.play] = .ReentrantCall

        model.togglePlayback()

        #expect(model.transientMessage == "İşlem şu anda tamamlanamadı.")
        #expect(model.fatalMessage == nil)
        #expect(model.playbackState != .failed)
    }

    @Test("transient copy carries no path, engine name, or numeric code")
    func transientCopyIsClosed() {
        let engineNames = ["mpv", "libmpv", "MPV", "AVFoundation", "ffmpeg"]
        for error in Self.everyPlaybackError {
            let message = PlaybackPresentation.errorMessage(for: error)
            #expect(!message.contains("/"), "path separator in: \(message)")
            let carriesDigits = message.rangeOfCharacter(from: .decimalDigits) != nil
            #expect(!carriesDigits, "numeric code in: \(message)")
            for name in engineNames {
                #expect(!message.contains(name), "engine name in: \(message)")
            }
            // `FfiPlaybackError` reflects itself in `errorDescription`; the
            // shell must never fall through to that.
            #expect(message != String(reflecting: error))
            #expect(message != error.localizedDescription)
        }
    }

    @Test("every playback error variant has its own Turkish sentence")
    func everyErrorVariantHasDistinctCopy() {
        let messages = Self.everyPlaybackError.map(PlaybackPresentation.errorMessage(for:))
        #expect(Set(messages).count == messages.count)
        for message in messages {
            #expect(!message.isEmpty)
            #expect(message.hasSuffix("."))
        }
        #expect(PlaybackPresentation.errorMessage(for: CocoaError(.fileNoSuchFile))
            == "İşlem tamamlanamadı.")
    }

    @Test("a transient message clears itself")
    func transientMessageExpires() async throws {
        let fixture = FakeSession()
        let model = makePlayingModel(
            session: fixture,
            transientMessageDurationNanoseconds: 20_000_000
        )
        fixture.errors[.seek] = .NotLoaded

        model.seekRelative(seconds: 10)
        #expect(model.transientMessage != nil)

        try await Task.sleep(nanoseconds: 300_000_000)
        #expect(model.transientMessage == nil)
    }

    /// Every `FfiPlaybackError` variant, including all four load failures.
    /// Payload values are deliberately distinctive so the negative test would
    /// catch them if they ever leaked into user-facing copy.
    private static let everyPlaybackError: [FfiPlaybackError] = [
        .Unsupported(capability: .volume),
        .ReentrantCall,
        .NotLoaded,
        .ShutDown,
        .UnknownTrack(kind: .subtitle),
        .RateOutOfRange(requested: 7.5, min: 0.25, max: 4),
        .LoadFailed(reason: .notFound),
        .LoadFailed(reason: .unreadable),
        .LoadFailed(reason: .unsupportedFormat),
        .LoadFailed(reason: .networkUnavailable),
        .EngineFailure(code: 4242),
    ]

    private func makeModel(
        session: FakeSession,
        store: MemoryRecentStore = MemoryRecentStore(),
        transientMessageDurationNanoseconds: UInt64 = 3_000_000_000
    ) -> PlayerModel {
        let model = PlayerModel(
            recentStore: store,
            startsPolling: false,
            transientMessageDurationNanoseconds: transientMessageDurationNanoseconds,
            managesCursor: false,
            sessionFactory: { _ in session }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())
        return model
    }

    /// Loads media and settles it into `Ready`, so refusal tests start from a
    /// model that genuinely has media.
    private func makePlayingModel(
        session: FakeSession,
        transientMessageDurationNanoseconds: UInt64 = 3_000_000_000
    ) -> PlayerModel {
        let model = makeModel(
            session: session,
            transientMessageDurationNanoseconds: transientMessageDurationNanoseconds
        )
        model.openMedia(at: URL(fileURLWithPath: "/fixtures/media/contract-clip.mkv"))
        model.consume([.stateChanged(state: .ready)])
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

/// The session calls a test can make fail, so the shell's refusal paths run.
private enum FakeSessionCall: Hashable {
    case load, play, pause, stop, seek, position, duration, state, tracks, volume
}

private final class FakeSession: PlaybackSessionClient {
    /// Errors keyed by call: every listed call throws instead of succeeding.
    var errors: [FakeSessionCall: FfiPlaybackError] = [:]

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
    var shutdownCount = 0

    private func refuse(_ call: FakeSessionCall) throws {
        if let error = errors[call] { throw error }
    }

    func load(locator: String) throws {
        try refuse(.load)
        loadedLocators.append(locator)
    }
    func play() throws {
        try refuse(.play)
        playCount += 1
        currentState = .playing
    }
    func pause() throws {
        try refuse(.pause)
        pauseCount += 1
        currentState = .paused
    }
    func stop() throws {
        try refuse(.stop)
        currentState = .idle
    }
    func seek(toMs: UInt64) throws {
        try refuse(.seek)
        seekTargets.append(toMs)
        currentPosition = toMs
    }
    func positionMs() throws -> UInt64 {
        try refuse(.position)
        return currentPosition
    }
    func durationMs() throws -> UInt64? {
        try refuse(.duration)
        return currentDuration
    }
    func state() throws -> FfiPlaybackState {
        try refuse(.state)
        return currentState
    }
    func tracks(kind: FfiTrackKind) throws -> [FfiTrackDescriptor] {
        try refuse(.tracks)
        requestedTrackKinds.append(kind)
        return []
    }
    func setVolume(volume: Float) throws {
        try refuse(.volume)
        volumes.append(volume)
    }
    func drainEvents() -> [FfiSessionEvent] {
        defer { events.removeAll() }
        return events
    }
    func shutdown() throws { shutdownCount += 1 }
}
