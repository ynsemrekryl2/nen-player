import Foundation
import Testing

import SpikeCore

// NEN-029 — typed error transfer, both directions. Same discipline
// NEN-010 established: exhaustive `switch`/pattern match, never a message
// string. No field on `EngineError` is K23-sensitive (see lib.rs doc), so
// this is purely about the mechanism, not redaction.

final class ErrorRecordingObserver: PlaybackObserver, @unchecked Sendable {
    private let lock = NSLock()
    private var errors: [EngineError] = []
    let errorReceived = DispatchSemaphore(value: 0)

    func onPosition(tick: PositionTick) {}
    func onStateChanged(sessionId: UInt64, seq: UInt32, state: PlaybackState) {}
    func onTrackSnapshot(snapshot: TrackSnapshot) {}
    func onSeekCompleted(sessionId: UInt64, seq: UInt32, targetMs: UInt64) {}
    func onError(error: EngineError) {
        lock.lock(); errors.append(error); lock.unlock()
        errorReceived.signal()
    }

    var recorded: [EngineError] {
        lock.lock(); defer { lock.unlock() }
        return errors
    }
}

func classifyEngineError(_ error: EngineError) -> String {
    switch error {
    case .SeekOutOfRange: return "seek_out_of_range"
    case .EngineFailure: return "engine_failure"
    }
}

@Suite("NEN-029 typed error transfer")
struct ReverseFFITypedErrorTests {

    /// **A** — `trigger_error` delivers `.EngineFailure` through the reverse
    /// callback, switchable without ever parsing a message string.
    @Test("A: trigger_error delivers EngineFailure, switchable without string parsing")
    func aEngineFailureIsTypedAndSwitchable() {
        let observer = ErrorRecordingObserver()
        let params = TickParams(
            sessionId: 1, hz: 50.0, runDurationMs: 500, mediaDurationMs: 60_000, seekLatencyMs: 5)
        let engine = ReverseEngine.start(params: params, observer: observer)
        engine.triggerError(code: .unsupportedCodec)
        let arrived = observer.errorReceived.wait(timeout: .now() + 2) == .success
        engine.shutdown()

        #expect(arrived)
        #expect(observer.recorded.count == 1)
        #expect(classifyEngineError(observer.recorded[0]) == "engine_failure")
        if case let .EngineFailure(code) = observer.recorded[0] {
            #expect(code == .unsupportedCodec)
        } else {
            Issue.record("expected .EngineFailure, got \(observer.recorded[0])")
        }
    }

    /// **A** — `seek()` past the simulated media length delivers
    /// `.SeekOutOfRange` (never `on_seek_completed`), carrying the exact
    /// requested/duration values as typed fields.
    @Test("A: seek() past media duration delivers SeekOutOfRange, not on_seek_completed")
    func aSeekPastDurationIsTypedError() {
        let observer = ErrorRecordingObserver()
        let params = TickParams(
            sessionId: 1, hz: 50.0, runDurationMs: 1_000, mediaDurationMs: 10_000, seekLatencyMs: 10)
        let engine = ReverseEngine.start(params: params, observer: observer)
        engine.seek(targetMs: 999_999)
        let arrived = observer.errorReceived.wait(timeout: .now() + 2) == .success
        engine.shutdown()

        #expect(arrived)
        #expect(observer.recorded.count == 1)
        if case let .SeekOutOfRange(requestedMs, durationMs) = observer.recorded[0] {
            #expect(requestedMs == 999_999)
            #expect(durationMs == 10_000)
        } else {
            Issue.record("expected .SeekOutOfRange, got \(observer.recorded[0])")
        }
    }

    /// **B** — `report_failure` is a plain forward call returning a typed,
    /// switchable `EngineError` via Swift's `throws` — no reverse callback
    /// involved at all.
    @Test("B: report_failure throws a typed, switchable EngineError")
    func bReportFailureThrowsTypedError() {
        let session = ForwardSession(sessionId: 9)
        do {
            try session.reportFailure(code: .ioStalled)
            Issue.record("expected reportFailure to throw")
        } catch let error as EngineError {
            #expect(classifyEngineError(error) == "engine_failure")
            if case let .EngineFailure(code) = error {
                #expect(code == .ioStalled)
            } else {
                Issue.record("expected .EngineFailure, got \(error)")
            }
        } catch {
            Issue.record("expected EngineError, got \(error)")
        }
        session.shutdown()
    }

    /// **B** — a forward call arriving after `shutdown()` fails gracefully
    /// with a typed error rather than crashing or silently no-op'ing. Not
    /// Invariant I1 (B has no reverse callback to be "late") — a distinct,
    /// smaller B-side guarantee the DoD's scoping note calls for explicitly.
    @Test("B: forward call after shutdown throws typed error instead of crashing")
    func bForwardCallAfterShutdownThrowsTypedError() {
        let session = ForwardSession(sessionId: 10)
        session.shutdown()
        do {
            try session.reportPosition(seq: 0, positionMs: 0)
            Issue.record("expected reportPosition to throw after shutdown")
        } catch let error as EngineError {
            #expect(classifyEngineError(error) == "engine_failure")
        } catch {
            Issue.record("expected EngineError, got \(error)")
        }
    }
}
