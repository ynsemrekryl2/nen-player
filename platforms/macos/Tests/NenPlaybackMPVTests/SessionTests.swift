import Foundation
import NenCore
import Testing

@testable import NenPlaybackMPV

/// NEN-045's boundary evidence: the shell drives real playback **through the
/// core**, not around it.
///
/// The Rust tests in `core/crates/nen-app/tests/playback_session.rs` already
/// judge the session's own behaviour — the pump, the bounded queue, the
/// shutdown order — against a scripted engine. Nothing of that is repeated
/// here. What only this side can show is that a real libmpv engine, handed over
/// and then let go of, is driven by a Rust-owned session whose pump calls back
/// into Swift on its own thread.
struct SessionTests {
    static func fixturePath(_ name: String) -> String {
        ContractTests.fixturePath(name)
    }

    /// Builds a session over a real engine.
    ///
    /// The engine is deliberately not kept: ADR-0033 Karar 1 says the shell
    /// hands it over and talks to the session afterwards. A test that held on
    /// to it would be demonstrating the shape this task exists to remove.
    private func session() throws -> FfiPlaybackSession {
        FfiPlaybackSession(engine: try MPVPlaybackEngine())
    }

    @Test func theSessionPlaysTheClipAndReportsItMoving() throws {
        let session = try self.session()
        defer { try? session.shutdown() }

        try session.load(locator: Self.fixturePath("contract-clip.mkv"))
        try settle(session, until: .ready)
        try session.play()

        // Position events are the pump's work: nothing in this test asks the
        // engine for anything between `play` and the drain below.
        var positions: [UInt64] = []
        var states: [FfiPlaybackState] = []
        try waitUntil("playback reports two distinct positions") {
            for event in session.drainEvents() {
                switch event {
                case .positionChanged(let positionMs): positions.append(positionMs)
                case .stateChanged(let state): states.append(state)
                default: break
                }
            }
            return Set(positions).count >= 2
        }

        #expect(states.contains(.playing), "never reported playing: \(states)")
        #expect(positions.last! > positions.first!, "position did not advance")
        #expect(try session.durationMs() == 30_008)
    }

    @Test func aSeekIsAnsweredThroughTheSession() throws {
        let session = try self.session()
        defer { try? session.shutdown() }

        try session.load(locator: Self.fixturePath("contract-clip.mkv"))
        try settle(session, until: .ready)
        _ = session.drainEvents()

        try session.seek(toMs: 12_000)

        var landed: UInt64?
        try waitUntil("the seek is answered") {
            for event in session.drainEvents() {
                if case .seekCompleted(let positionMs) = event { landed = positionMs }
            }
            return landed != nil
        }
        #expect(landed! > 11_900 && landed! < 12_100, "seek landed at \(landed!) ms")
    }

    @Test func trackMetadataSurvivesTheRoundTrip() throws {
        let session = try self.session()
        defer { try? session.shutdown() }

        try session.load(locator: Self.fixturePath("contract-clip.mkv"))
        try settle(session, until: .ready)

        let subtitles = try session.tracks(kind: .subtitle)
        #expect(subtitles.count == 2)
        // The container writes ISO 639-2; the core hands back the canonical tag
        // (ADR-0032). That the shell sees `en` rather than `eng` is the proof
        // the descriptor came back **through the core**.
        #expect(subtitles.contains { $0.language == "en" }, "\(subtitles.map(\.language))")

        try session.selectTrack(kind: .subtitle, track: subtitles[0].id)
        #expect(try session.selectedTrack(kind: .subtitle) == subtitles[0].id)
        try session.selectTrack(kind: .subtitle, track: nil)
        #expect(try session.selectedTrack(kind: .subtitle) == nil)
    }

    @Test func aBrokenFileFailsThroughTheSessionWithoutCrashing() throws {
        let session = try self.session()
        defer { try? session.shutdown() }

        try session.load(locator: Self.fixturePath("broken-clip.mkv"))
        try settle(session, until: .failed)

        #expect(throws: FfiPlaybackError.self) { try session.positionMs() }
    }

    @Test func everythingIsRefusedAfterShutdown() throws {
        let session = try self.session()
        try session.load(locator: Self.fixturePath("contract-clip.mkv"))
        try settle(session, until: .ready)

        try session.shutdown()
        // A shell cannot always know whether its own teardown already ran.
        try session.shutdown()

        #expect(throws: FfiPlaybackError.self) { try session.play() }
        #expect(throws: FfiPlaybackError.self) { try session.positionMs() }
        #expect(throws: FfiPlaybackError.self) { try session.state() }
    }

    // MARK: - Waiting

    private func settle(
        _ session: FfiPlaybackSession,
        until wanted: FfiPlaybackState,
        timeout: TimeInterval = 5
    ) throws {
        try waitUntil("state \(wanted)", timeout: timeout) {
            (try? session.state()) == wanted
        }
    }

    private func waitUntil(
        _ what: String,
        timeout: TimeInterval = 5,
        _ condition: () -> Bool
    ) throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if condition() { return }
            Thread.sleep(forTimeInterval: 0.005)
        }
        Issue.record("never happened: \(what)")
    }
}
