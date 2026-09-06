import Foundation
import NenCore
import Testing

@testable import NenPlaybackMPV

/// NEN-022's evidence: the real libmpv adapter against the **same** contract
/// kit the fake passes.
///
/// `docs/milestones/M3-macos-slice.md` states it as an exit criterion —
/// "gerçek libmpv adapter'ı, fake adapter ile **aynı** contract kitini
/// geçmelidir" — and ADR-0011 Karar 4 is what makes "the same" literal: the
/// scenarios are data in `nen-ports`, driven from here through FFI. There is no
/// second copy of the kit in this file, and there must never be one.
struct ContractTests {
    // MARK: - Fixture

    /// The repo root, derived from this file's own path.
    ///
    /// Not a bundle resource: the fixture is shared with the Rust tests and the
    /// generation command in `fixtures/media/`, and copying it into a bundle
    /// would make "the same clip" quietly stop being the same clip.
    static let repositoryRoot: URL = {
        var url = URL(fileURLWithPath: #filePath)
        for _ in 0..<5 { url = url.deletingLastPathComponent() }
        return url
    }()

    static func fixturePath(_ name: String) -> String {
        repositoryRoot.appendingPathComponent("fixtures/media/\(name)").path
    }

    /// What `fixtures/media/contract-clip.mkv` is.
    ///
    /// Every number here was measured, not guessed — see
    /// `fixtures/media/contract-clip.ffmpeg.txt`. The ids are **ff-index**
    /// values, because the port's ids are one space across kinds while mpv
    /// numbers tracks per kind: audio and subtitle would both start at 1.
    static var fixture: FfiContractFixture {
        FfiContractFixture(
            locator: fixturePath("contract-clip.mkv"),
            durationMs: 30_008,
            audioTrackCount: 2,
            subtitleTrackCount: 2,
            audioTrack: 1,
            subtitleTrack: 3,
            unknownTrack: 9_999,
            // Measured: `seek 5 absolute+exact` lands on 5.000000 s exactly on
            // this clip. The margin is for media this adapter has not met — a
            // sparse-keyframe or VFR source — not for slack it needs here.
            seekToleranceMs: 100,
            settleTimeoutMs: 5_000,
            // The clip's real display size. Declaring it is what makes the
            // kit's geometry step demand the positive branch — announced, then
            // readable — instead of the audio-only one.
            videoGeometry: FfiVideoGeometry(width: 160, height: 90)
        )
    }

    // MARK: - The kit

    @Test func theRealAdapterPassesTheSharedContractKit() throws {
        let report = runPlaybackContract(factory: MPVEngineFactory(), fixture: Self.fixture)

        #expect(report.failures.isEmpty, "\(report.failures.joined(separator: "\n"))")

        // A run that applied nothing would report no failures and look like a
        // pass. The expected count comes from the kit itself, so adding a
        // scenario tightens this automatically instead of dating it.
        let expected = applicableScenarioCount(capabilities: MPVPlaybackEngine.declaredCapabilities)
        #expect(report.applied == expected)
        #expect(report.applied >= 15, "only \(report.applied) scenarios applied")
    }

    // MARK: - DoD, on the real clip

    @Test func playPauseAndSeekOnTheLocalClip() throws {
        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        try engine.load(locator: Self.fixturePath("contract-clip.mkv"))
        try settle(engine, until: .ready)

        try engine.play()
        #expect(engine.state() == .playing)

        try engine.pause()
        #expect(engine.state() == .paused)

        try engine.seek(toMs: 12_000)
        try settle(engine) { _ in
            (try? engine.positionMs()).map { $0 > 11_000 && $0 < 13_000 } ?? false
        }
        let landed = try engine.positionMs()
        #expect(landed > 11_900 && landed < 12_100, "seek landed at \(landed) ms")

        #expect(try engine.durationMs() == 30_008)
        #expect(try engine.tracks(kind: .audio).count == 2)
        #expect(try engine.tracks(kind: .subtitle).count == 2)
    }

    @Test func seekingPastTheDurationEndsPlaybackInsteadOfFailing() throws {
        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        try engine.load(locator: Self.fixturePath("contract-clip.mkv"))
        try settle(engine, until: .ready)
        try engine.play()

        // Well past a 30 s clip. The port is explicit that this is not an
        // error: it ends playback.
        try engine.seek(toMs: 9_999_999)
        try settle(engine, until: .ended)

        #expect(engine.state() == .ended)
        // `keep-open` is why the medium is still there to answer: the `Ended`
        // state has media loaded, so duration and tracks still work.
        #expect(try engine.durationMs() == 30_008)
    }

    // MARK: - Display geometry (ADR-0038)

    @Test func theDisplaySizeIsAnnouncedAndThenReadable() throws {
        // Both halves of ADR-0038 Karar 1 and 2 against the real engine. The
        // announcement matters as much as the value: without it the shell is
        // never told to look, and the window keeps the previous medium's shape.
        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        // Idle, `None` would mean "this medium has no video" — and there is no
        // medium at all, so the honest answer is a refusal.
        #expect(throws: FfiPlaybackError.self) { try engine.videoGeometry() }

        try engine.load(locator: Self.fixturePath("contract-clip.mkv"))
        try settle(engine, until: .ready)

        #expect(
            try drainedShapes(engine, contain: .videoGeometryChanged),
            "the engine never announced the display size"
        )
        #expect(try waitForGeometry(engine) == FfiVideoGeometry(width: 160, height: 90))
    }

    @Test func anAnamorphicMediumReportsItsDisplaySizeNotItsStoredSize() throws {
        // ADR-0038 Karar 3, against the one fixture where the two differ:
        // `anamorphic-clip.mkv` stores 720x576 and displays 1024x576. An
        // adapter that handed over the stored frame would open a 5:4 window
        // for a 16:9 picture, and every other fixture in this repository is
        // square-pixel, so nothing else in the suite could tell.
        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        try engine.load(locator: Self.fixturePath("anamorphic-clip.mkv"))
        try settle(engine, until: .ready)

        let geometry = try waitForGeometry(engine)
        #expect(geometry == FfiVideoGeometry(width: 1_024, height: 576))
        #expect(geometry != FfiVideoGeometry(width: 720, height: 576))
    }

    @Test func aLoadingMediumDoesNotExposeTheOutgoingDisplaySize() throws {
        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }
        try engine.load(locator: Self.fixturePath("contract-clip.mkv"))
        try settle(engine, until: .ready)
        let outgoing = try waitForGeometry(engine)
        #expect(outgoing == FfiVideoGeometry(width: 160, height: 90))

        // Hold the adapter at load's first step while the real VO still has
        // the old picture. Racing an actual load would miss this short window.
        try engine.mutate { engine.phase = .loading }
        #expect(try engine.videoGeometry() == nil)
        try engine.mutate { engine.phase = .loaded }
        #expect(try engine.videoGeometry() == outgoing, "the old VO size must really exist")
    }

    @Test func successiveMediaReportTheirOwnDisplaySize() throws {
        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }
        var outgoing: FfiVideoGeometry?
        for (name, geometry) in [
            ("contract-clip.mkv", FfiVideoGeometry(width: 160, height: 90)),
            ("aspect-4x3-clip.mkv", FfiVideoGeometry(width: 160, height: 120)),
            ("anamorphic-clip.mkv", FfiVideoGeometry(width: 1_024, height: 576))
        ] {
            try engine.load(locator: Self.fixturePath(name))
            try settle(engine, until: .ready)
            // What the previous medium reported, not what this one should:
            // handing the wanted size to the wait would make the expectation
            // below assert its own premise.
            let seen = try waitForGeometry(engine, after: outgoing)
            #expect(seen == geometry, "loaded \(name), saw \(String(describing: seen))")
            outgoing = seen
        }
        try engine.load(locator: Self.fixturePath("audio-only-clip.mka"))
        try settle(engine, until: .ready)
        #expect(try engine.videoGeometry() == nil)
    }

    @Test func aMediumWithNoVideoAnswersNothingAndAnnouncesNothing() throws {
        // `None` is a state, not a failure. Measured: libmpv emits no video
        // reconfiguration at all for an audio-only medium
        // (`evidence/M3/NEN-068-measurement.md`), so an announcement here would
        // send the shell asking after a size that does not exist.
        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        try engine.load(locator: Self.fixturePath("audio-only-clip.mka"))
        try settle(engine, until: .ready)
        // Long enough for a reconfiguration to have arrived if one were coming;
        // the positive test above sees its own within the settle.
        Thread.sleep(forTimeInterval: 0.3)

        #expect(try engine.videoGeometry() == nil)
        #expect(
            // A short window: the wait above already gave a reconfiguration
            // every chance to arrive, and a negative assertion must not spend
            // the positive path's patience proving nothing happened.
            try !drainedShapes(engine, contain: .videoGeometryChanged, timeout: 0.2),
            "a medium with no picture announced a display size"
        )
    }

    /// Waits for the size to resolve, the way the shared kit's own step does.
    ///
    /// Measured: the size is still unreadable at the first of the two
    /// reconfigurations a load produces, so a single read the instant after the
    /// event reads before the answer exists.
    ///
    /// `outgoing` is the size the **previous** medium reported on this engine,
    /// and passing it is what makes a second load safe to wait for. Between
    /// `loadfile` and the new picture's reconfiguration, `video-out-params`
    /// still describes the medium that is leaving —
    /// `aLoadingMediumDoesNotExposeTheOutgoingDisplaySize` pins that on
    /// purpose — so a wait that takes the first non-nil answer can take the old
    /// one. Under a loaded machine that window is wide enough to lose: measured
    /// at 2/5 green in the parallel package, failing in ~0,2 s rather than at
    /// the timeout, with the wait returning exactly the outgoing size
    /// (NEN-049).
    ///
    /// Only the outgoing value is excluded, never the wanted one: a wait told
    /// what to expect would answer the caller's own question. A third size
    /// still resolves, and still fails the caller's expectation.
    ///
    /// Two successive media that genuinely share a display size would spend the
    /// whole timeout here before answering correctly. No fixture pair does
    /// today; one that did would need the load's own reconfiguration to
    /// separate them, not a value comparison.
    private func waitForGeometry(
        _ engine: MPVPlaybackEngine,
        after outgoing: FfiVideoGeometry? = nil,
        timeout: TimeInterval = 5
    ) throws -> FfiVideoGeometry? {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if let geometry = try engine.videoGeometry(), geometry != outgoing { return geometry }
            Thread.sleep(forTimeInterval: 0.01)
        }
        return try engine.videoGeometry()
    }

    /// Whether the engine has reported an event of this shape by now.
    ///
    /// Drains rather than samples once: the adapter reports on its own thread,
    /// so what is pending at any single instant is a race.
    private func drainedShapes(
        _ engine: MPVPlaybackEngine,
        contain wanted: FfiPlaybackEvent,
        timeout: TimeInterval = 3
    ) throws -> Bool {
        let deadline = Date().addingTimeInterval(timeout)
        var seen: [FfiPlaybackEvent] = []
        repeat {
            seen.append(contentsOf: engine.drainEvents())
            if seen.contains(wanted) { return true }
            Thread.sleep(forTimeInterval: 0.01)
        } while Date() < deadline
        return false
    }

    @Test func aFileThatIsNotMediaFailsWithoutCrashing() throws {
        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        // Valid extension, bytes that are not a container.
        try engine.load(locator: Self.fixturePath("broken-clip.mkv"))
        try settle(engine, until: .failed)

        #expect(engine.state() == .failed)
        #expect(throws: FfiPlaybackError.self) { try engine.positionMs() }
    }

    @Test func aMissingFileFailsWithoutCrashing() throws {
        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        try engine.load(locator: "/nen-does-not-exist/missing.mkv")
        try settle(engine, until: .failed)

        #expect(engine.state() == .failed)
    }

    @Test func shutdownIsIdempotentAndRefusesEverythingAfterwards() throws {
        let engine = try MPVPlaybackEngine()
        try engine.load(locator: Self.fixturePath("contract-clip.mkv"))
        try settle(engine, until: .ready)

        try engine.shutdown()
        // A shell cannot always know whether its own teardown already ran.
        try engine.shutdown()

        #expect(throws: FfiPlaybackError.self) { try engine.play() }
        #expect(throws: FfiPlaybackError.self) { try engine.positionMs() }
        #expect(engine.state() == .idle)
    }

    // MARK: - Settling

    /// Waits for a state, the way the kit's own `Settle` step does.
    private func settle(
        _ engine: MPVPlaybackEngine,
        until wanted: FfiPlaybackState,
        timeout: TimeInterval = 5
    ) throws {
        try settle(engine, timeout: timeout) { $0 == wanted }
    }

    private func settle(
        _ engine: MPVPlaybackEngine,
        timeout: TimeInterval = 5,
        until predicate: (FfiPlaybackState) -> Bool
    ) throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if predicate(engine.state()) { return }
            Thread.sleep(forTimeInterval: 0.005)
        }
        Issue.record("never settled; stuck at \(engine.state())")
    }
}
