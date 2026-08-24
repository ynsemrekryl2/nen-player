import Foundation
import Testing

import SpikeCore

// NEN-029 — event ordering, seek/seek-complete ordering (A and B), and
// reentrancy. Baseline numbers (4 Hz/60 Hz sweep) live in
// SpikeReverseFFI/main.swift; this file only proves ordering properties.

final class SeqRecordingObserver: PlaybackObserver, @unchecked Sendable {
    private let lock = NSLock()
    private var seqs: [UInt32] = []

    private func record(_ seq: UInt32) {
        lock.lock(); seqs.append(seq); lock.unlock()
    }

    func onPosition(tick: PositionTick) { record(tick.seq) }
    func onStateChanged(sessionId: UInt64, seq: UInt32, state: PlaybackState) { record(seq) }
    func onTrackSnapshot(snapshot: TrackSnapshot) { record(snapshot.seq) }
    func onSeekCompleted(sessionId: UInt64, seq: UInt32, targetMs: UInt64) { record(seq) }
    func onError(error: EngineError) {}

    var recorded: [UInt32] {
        lock.lock(); defer { lock.unlock() }
        return seqs
    }
}

final class SeekOutcomeObserver: PlaybackObserver, @unchecked Sendable {
    private let lock = NSLock()
    private var completions: [UInt64] = []
    private var errorCount = 0
    let outcomeReceived = DispatchSemaphore(value: 0)

    func onPosition(tick: PositionTick) {}
    func onStateChanged(sessionId: UInt64, seq: UInt32, state: PlaybackState) {}
    func onTrackSnapshot(snapshot: TrackSnapshot) {}
    func onSeekCompleted(sessionId: UInt64, seq: UInt32, targetMs: UInt64) {
        lock.lock(); completions.append(targetMs); lock.unlock()
        outcomeReceived.signal()
    }
    func onError(error: EngineError) {
        lock.lock(); errorCount += 1; lock.unlock()
        outcomeReceived.signal()
    }

    var snapshotState: (completions: [UInt64], errors: Int) {
        lock.lock(); defer { lock.unlock() }
        return (completions, errorCount)
    }
}

/// Reentrant `seek()` from inside `onPosition`, on the callback's own
/// thread. `ReverseEngine::seek` only takes its own `seek_threads` lock —
/// never the delivery gate's `sink` mutex a callback runs under — so this
/// is safe by construction, not by luck (see lib.rs's `deliver`/`cancel`
/// doc comment for the contrasting *unsafe* case, `cancel()`, which is not
/// exercised live here — see the module doc below for why).
final class ReentrantSeekObserver: PlaybackObserver, @unchecked Sendable {
    weak var engineRef: ReverseEngine?
    private let lock = NSLock()
    private var seekTriggered = false
    private var completions: [UInt64] = []
    let firstPosition = DispatchSemaphore(value: 0)
    let seekCompleted = DispatchSemaphore(value: 0)

    func onPosition(tick: PositionTick) {
        lock.lock()
        let shouldTrigger = !seekTriggered
        seekTriggered = true
        lock.unlock()
        if shouldTrigger, let engine = engineRef {
            engine.seek(targetMs: 7_777)
        }
        firstPosition.signal()
    }
    func onStateChanged(sessionId: UInt64, seq: UInt32, state: PlaybackState) {}
    func onTrackSnapshot(snapshot: TrackSnapshot) {}
    func onSeekCompleted(sessionId: UInt64, seq: UInt32, targetMs: UInt64) {
        lock.lock(); completions.append(targetMs); lock.unlock()
        seekCompleted.signal()
    }
    func onError(error: EngineError) {}

    var completed: [UInt64] {
        lock.lock(); defer { lock.unlock() }
        return completions
    }
}

@Suite("NEN-029 ordering and reentrancy")
struct ReverseFFIOrderingTests {

    /// **A** — global `seq`, stamped by Rust immediately before every
    /// callback kind while the delivery gate is held, arrives strictly
    /// increasing across *all* kinds (position, state, track, seek), not
    /// just within one.
    @Test("A: seq strictly increases across all callback kinds")
    func aEventOrderingIsStrictlyIncreasing() {
        let observer = SeqRecordingObserver()
        let params = TickParams(
            sessionId: 1, hz: 300.0, runDurationMs: 600, mediaDurationMs: 600_000, seekLatencyMs: 5)
        let engine = ReverseEngine.start(params: params, observer: observer)
        engine.publishTrackSnapshot(audioTrack: 1, subtitleTrack: 2)
        engine.seek(targetMs: 3_000)

        while engine.isRunning() {
            Thread.sleep(forTimeInterval: 0.02)
        }
        Thread.sleep(forTimeInterval: 0.1) // let the seek's own short-lived thread land too
        engine.shutdown()

        let seqs = observer.recorded
        #expect(seqs.count > 3, "expected multiple callback kinds to have fired")
        for i in 1..<seqs.count {
            #expect(seqs[i - 1] < seqs[i], "seq must be strictly increasing: \(seqs)")
        }
    }

    /// **A** — `seek(target)` within the simulated media length eventually
    /// (not immediately — it's async) delivers `on_seek_completed` for that
    /// exact target, never an error.
    @Test("A: seek() within range completes with the requested target")
    func aSeekWithinRangeCompletesWithTarget() {
        let observer = SeekOutcomeObserver()
        let params = TickParams(
            sessionId: 1, hz: 50.0, runDurationMs: 3_000, mediaDurationMs: 600_000, seekLatencyMs: 10)
        let engine = ReverseEngine.start(params: params, observer: observer)
        engine.seek(targetMs: 12_345)

        let signaled = observer.outcomeReceived.wait(timeout: .now() + 2) == .success
        engine.shutdown()

        #expect(signaled, "seek outcome did not arrive within timeout")
        #expect(observer.snapshotState.completions == [12_345])
        #expect(observer.snapshotState.errors == 0)
    }

    /// **B** — `report_seek_started` then `report_seek_completed`, called in
    /// that order, preserve arrival order (no reordering across the
    /// boundary — expected, since B has no async hop at all).
    @Test("B: seek-started then seek-completed preserve arrival order")
    func bSeekOrderingIsPreserved() {
        let session = ForwardSession(sessionId: 3)
        try? session.reportSeekStarted(seq: 0, targetMs: 5_000)
        try? session.reportSeekCompleted(seq: 1, targetMs: 5_000)
        session.shutdown()

        #expect(session.arrivalSeqs() == [0, 1])
    }

    /// Reentrancy — not one of I1–I5, a baseline observation (DoD: "callback
    /// içinden core'a çağrı güvenli mi"). Triggering a new `seek()` from
    /// *inside* `onPosition`, on the callback's own thread, must not be
    /// blocked by the callback that triggered it — proven with a bounded
    /// wait, not an unbounded one.
    ///
    /// **What is deliberately NOT exercised here:** a same-thread `cancel()`
    /// call from inside a callback. `cancel()` takes the exact `sink` mutex
    /// `deliver()` already holds while invoking the callback — that combination
    /// self-deadlocks the tick thread by construction (see lib.rs's
    /// `deliver`/`cancel` doc comments for the proof by inspection). Actually
    /// triggering that deadlock in this suite would wedge an OS thread and
    /// permanently leak `live_engines()`'s count for every later test in this
    /// binary — a worse outcome than the analytical proof already gives. The
    /// finding is real and worth recording in the evidence write-up; it is
    /// not worth an executed test that corrupts shared process state.
    @Test("reentrancy: seek() from inside onPosition is safe (does not touch the delivery-gate mutex)")
    func reentrantSeekFromCallbackDoesNotDeadlock() {
        let observer = ReentrantSeekObserver()
        let params = TickParams(
            sessionId: 1, hz: 100.0, runDurationMs: 3_000, mediaDurationMs: 600_000, seekLatencyMs: 10)
        let engine = ReverseEngine.start(params: params, observer: observer)
        observer.engineRef = engine

        let firstArrived = observer.firstPosition.wait(timeout: .now() + 2) == .success
        #expect(firstArrived, "expected the first onPosition callback")
        let seekArrived = observer.seekCompleted.wait(timeout: .now() + 2) == .success
        #expect(
            seekArrived,
            "reentrant seek() should complete like any other seek — it must not be blocked by the callback that triggered it")
        #expect(observer.completed == [7_777])

        engine.shutdown()
    }
}
