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

    // MARK: - Transport feedback (NEN-055)

    @Test("the icon turns on the click, not on the next poll tick")
    func togglingReadsWhatTheCommandAlreadyProduced() {
        let fixture = FakeSession()
        fixture.currentDuration = 30_000
        let model = makePlayingModel(session: fixture)
        // `Ready` starts playback on its own, and that transition is queued too.
        model.consume(fixture.drainEvents())
        #expect(model.isPlaying)

        model.togglePlayback()

        // Nothing polled: `startsPolling` is false and no event was handed in.
        #expect(model.playbackState == .paused)
        #expect(fixture.pauseCount == 1)

        model.togglePlayback()

        #expect(model.isPlaying)
        #expect(fixture.playCount == 2)
    }

    // MARK: - The seek guard (NEN-053)
    //
    // mpv keeps reporting `time-pos` from before a seek until it has served it,
    // and those reports reach the shell through two stages of polling. Applied
    // blindly they undo a seek that already landed — the knob snapping home
    // before jumping to where it was released.

    @Test("a position report from before a seek does not move the knob back")
    func stalePositionDoesNotUndoASeek() {
        let fixture = FakeSession()
        fixture.currentPosition = 4_000
        fixture.currentDuration = 30_000
        let model = makePlayingModel(session: fixture)

        model.previewSeek(to: 20_000)
        model.commitSeek()
        #expect(model.displayedPositionMilliseconds == 20_000)

        // What mpv was reporting while the core had not served the seek yet.
        model.consume([.positionChanged(positionMs: 4_040), .positionChanged(positionMs: 4_080)])

        #expect(model.positionMilliseconds == 20_000)
        #expect(model.seekPreviewMilliseconds == nil)
    }

    @Test("the seek's own answer is what lets position reports through again")
    func seekCompletedReleasesTheGuard() {
        let fixture = FakeSession()
        fixture.currentPosition = 4_000
        fixture.currentDuration = 30_000
        let model = makePlayingModel(session: fixture)

        model.seekRelative(seconds: 10)
        model.consume([.positionChanged(positionMs: 4_040)])
        #expect(model.positionMilliseconds == 14_000)

        model.consume([.seekCompleted(positionMs: 14_000)])
        model.consume([.positionChanged(positionMs: 14_040)])

        #expect(model.positionMilliseconds == 14_040)
    }

    @Test("two seeks in a row stay guarded until both are answered")
    func backToBackSeeksNeedBothAnswers() {
        let fixture = FakeSession()
        fixture.currentPosition = 4_000
        fixture.currentDuration = 30_000
        let model = makePlayingModel(session: fixture)

        model.seekRelative(seconds: 5)
        model.seekRelative(seconds: 5)
        #expect(fixture.seekTargets == [9_000, 14_000])

        // mpv merges seeks it cannot serve one by one and answers all of them
        // from a single restart, so the first answer must not open the gate.
        model.consume([.seekCompleted(positionMs: 14_000)])
        model.consume([.positionChanged(positionMs: 9_040)])
        #expect(model.positionMilliseconds == 14_000)

        model.consume([.seekCompleted(positionMs: 14_000)])
        model.consume([.positionChanged(positionMs: 14_040)])
        #expect(model.positionMilliseconds == 14_040)
    }

    @Test("an answer that never arrives does not freeze the position forever")
    func theGuardExpires() {
        let fixture = FakeSession()
        fixture.currentPosition = 4_000
        fixture.currentDuration = 30_000
        let clock = TestClock()
        let model = makePlayingModel(
            session: fixture,
            seekGuardTimeoutNanoseconds: 1_000,
            clock: clock
        )

        model.seekRelative(seconds: 10)
        model.consume([.positionChanged(positionMs: 4_040)])
        #expect(model.positionMilliseconds == 14_000)

        clock.nanoseconds += 1_001
        model.consume([.positionChanged(positionMs: 14_500)])

        #expect(model.positionMilliseconds == 14_500)
    }

    @Test("a refused seek leaves no guard behind")
    func aRefusedSeekDoesNotGuard() {
        let fixture = FakeSession()
        fixture.currentPosition = 4_000
        fixture.currentDuration = 30_000
        let model = makePlayingModel(session: fixture)
        fixture.errors[.seek] = .NotLoaded

        model.seekRelative(seconds: 10)
        model.consume([.positionChanged(positionMs: 4_040)])

        // Nothing was ever seeked to, so the position that keeps arriving is
        // the true one and must be shown.
        #expect(model.positionMilliseconds == 4_040)
    }

    @Test("EventsLost drops the guard along with the events")
    func resynchronizationDropsTheGuard() {
        let fixture = FakeSession()
        fixture.currentPosition = 4_000
        fixture.currentDuration = 30_000
        let model = makePlayingModel(session: fixture)

        model.seekRelative(seconds: 10)
        // The answer may be among what was dropped, so waiting for it would
        // wait forever; the engine is asked instead.
        fixture.currentPosition = 14_000
        model.consume([.eventsLost(dropped: 3)])
        model.consume([.positionChanged(positionMs: 14_040)])

        #expect(model.positionMilliseconds == 14_040)
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

    // MARK: - Subtitle loading and sidecar discovery (NEN-025)
    //
    // These drive the real `FfiSubtitleLibrary` against real files, because the
    // thing under test is the shell's *reaction* to a real gate verdict. The
    // gates themselves are proved in `nen-app`; what is proved here is
    // ADR-0031 Karar 5's split — who gets told, and who does not.

    @Test("a sidecar beside the medium is picked up when it opens")
    func sidecarIsDiscoveredOnOpen() async {
        let dir = TempFixture("discovered")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        dir.write("Film.srt", TempFixture.validSrt)

        let model = makeModel(session: FakeSession())
        model.openMedia(at: media)
        await model.awaitSidecarScan()

        #expect(model.subtitleSourceCount == 1)
        #expect(model.transientMessage == nil)
    }

    @Test("a file the user picked and had refused produces a notification")
    func explicitRefusalIsAnnounced() {
        let dir = TempFixture("explicit-refusal")
        defer { dir.remove() }
        let real = dir.write("real.srt", TempFixture.validSrt)
        let link = dir.symlink("Chosen.srt", to: real)

        let model = makePlayingModel(session: FakeSession())
        model.loadSubtitleFile(at: link)

        #expect(model.transientMessage == "Bu bir kısayol; altyazı olarak açılamıyor.")
        #expect(model.subtitleSourceCount == 0)
    }

    @Test("the same refusal found by a scan says nothing at all")
    func scannedRefusalIsSilent() {
        // The negative control for the test above: identical file, identical
        // verdict, different discoverer. Only the announcement differs.
        let dir = TempFixture("scanned-refusal")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let real = dir.write("real.srt", TempFixture.validSrt)
        _ = dir.symlink("Film.srt", to: real)

        let model = makeModel(session: FakeSession())
        model.openMedia(at: media)

        #expect(model.transientMessage == nil)
        #expect(model.subtitleSourceCount == 0)
    }

    @Test("a broken subtitle is catalogued quietly rather than interrupting playback")
    func brokenSubtitleDoesNotInterrupt() {
        // ADR-0031 Karar 5: its surface is the menu, not the transport. The
        // spine of the product is that a subtitle problem never stops playback.
        let dir = TempFixture("broken")
        defer { dir.remove() }
        let broken = dir.write("Broken.srt", "this is not a timecode")

        let session = FakeSession()
        let model = makePlayingModel(session: session)
        model.loadSubtitleFile(at: broken)

        #expect(model.transientMessage == nil)
        #expect(model.subtitleSourceCount == 1)
        // Playback was neither failed nor interrupted by the broken file.
        #expect(model.fatalMessage == nil)
        #expect(model.playbackState != .failed)
        #expect(session.pauseCount == 0)
        #expect(session.shutdownCount == 0)
    }

    @Test("loading one file twice leaves one source")
    func loadingTwiceLeavesOneSource() {
        let dir = TempFixture("twice")
        defer { dir.remove() }
        let subtitle = dir.write("Film.srt", TempFixture.validSrt)

        let model = makePlayingModel(session: FakeSession())
        model.loadSubtitleFile(at: subtitle)
        model.loadSubtitleFile(at: subtitle)

        #expect(model.subtitleSourceCount == 1)
    }

    @Test("a new medium does not inherit the previous one's subtitles")
    func openingAnotherMediumClearsTheCatalog() async {
        let dir = TempFixture("cleared")
        defer { dir.remove() }
        let first = dir.write("First.mkv", "not really a video")
        dir.write("First.srt", TempFixture.validSrt)
        let second = dir.write("Second.mkv", "not really a video")

        let model = makeModel(session: FakeSession())
        model.openMedia(at: first)
        // The scan is a phase of its own now (NEN-026): the medium never waits
        // for it, so the count only exists once it has landed.
        await model.awaitSidecarScan()
        #expect(model.subtitleSourceCount == 1)

        model.openMedia(at: second)
        await model.awaitSidecarScan()
        #expect(model.subtitleSourceCount == 0)
    }

    @Test("asking for a subtitle before a medium is refused, and says why")
    func subtitleWithoutMediaIsRefused() {
        let model = makeModel(session: FakeSession())
        model.chooseSubtitleFile()

        #expect(model.transientMessage == "Önce bir medya açın.")
        #expect(model.subtitleSourceCount == 0)
    }

    private func makeModel(
        session: FakeSession,
        store: MemoryRecentStore = MemoryRecentStore(),
        transientMessageDurationNanoseconds: UInt64 = 3_000_000_000,
        seekGuardTimeoutNanoseconds: UInt64 = 1_500_000_000,
        clock: TestClock = TestClock()
    ) -> PlayerModel {
        let model = PlayerModel(
            recentStore: store,
            startsPolling: false,
            transientMessageDurationNanoseconds: transientMessageDurationNanoseconds,
            seekGuardTimeoutNanoseconds: seekGuardTimeoutNanoseconds,
            now: { clock.nanoseconds },
            managesCursor: false,
            // Pinned rather than inherited from the machine: automatic subtitle
            // selection reads this, and a suite whose result depends on the
            // laptop's system language is not a suite.
            preferredSubtitleLanguage: nil,
            sessionFactory: { _ in session }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())
        return model
    }

    /// Loads media and settles it into `Ready`, so refusal tests start from a
    /// model that genuinely has media.
    private func makePlayingModel(
        session: FakeSession,
        transientMessageDurationNanoseconds: UInt64 = 3_000_000_000,
        seekGuardTimeoutNanoseconds: UInt64 = 1_500_000_000,
        clock: TestClock = TestClock()
    ) -> PlayerModel {
        let model = makeModel(
            session: session,
            transientMessageDurationNanoseconds: transientMessageDurationNanoseconds,
            seekGuardTimeoutNanoseconds: seekGuardTimeoutNanoseconds,
            clock: clock
        )
        model.openMedia(at: URL(fileURLWithPath: "/fixtures/media/contract-clip.mkv"))
        model.consume([.stateChanged(state: .ready)])
        return model
    }
}
