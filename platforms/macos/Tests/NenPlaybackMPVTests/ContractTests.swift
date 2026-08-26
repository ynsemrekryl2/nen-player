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
            settleTimeoutMs: 5_000
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
