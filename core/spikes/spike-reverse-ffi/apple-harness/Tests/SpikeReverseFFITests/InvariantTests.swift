import Foundation
import Testing

import SpikeCore

// NEN-029 — Invariant I1 (Direction A only) and I4 (both directions), plus a
// Direction-A-specific object-lifetime check. XCTest DEĞİL swift-testing:
// bu makinede tam Xcode yok (bkz. spike-async-cancel/CoreBridgeTests.swift).
//
// Ölçüm (4 Hz / 60 Hz baseline) burada değil, SpikeReverseFFI çalıştırılabilirinde
// — bu dosya yalnız pass/fail invariant'ları kanıtlıyor.

/// Every callback bumps `afterCancelCount` once `markCancelRequested()` has
/// been called — mirroring `spike-async-cancel`'s `RecordingSink`.
final class RecordingObserver: PlaybackObserver, @unchecked Sendable {
    private let lock = NSLock()
    private var afterCancelCount = 0
    private var cancelRequested = false
    let firstPosition = DispatchSemaphore(value: 0)
    private var signaled = false

    private func noteCallback() {
        lock.lock()
        if cancelRequested { afterCancelCount += 1 }
        lock.unlock()
    }

    func onPosition(tick: PositionTick) {
        lock.lock()
        if cancelRequested { afterCancelCount += 1 }
        let shouldSignal = !signaled
        signaled = true
        lock.unlock()
        if shouldSignal { firstPosition.signal() }
    }
    func onStateChanged(sessionId: UInt64, seq: UInt32, state: PlaybackState) { noteCallback() }
    func onTrackSnapshot(snapshot: TrackSnapshot) { noteCallback() }
    func onSeekCompleted(sessionId: UInt64, seq: UInt32, targetMs: UInt64) { noteCallback() }
    func onError(error: EngineError) { noteCallback() }

    func markCancelRequested() {
        lock.lock(); cancelRequested = true; lock.unlock()
    }

    var afterCancel: Int {
        lock.lock(); defer { lock.unlock() }
        return afterCancelCount
    }
}

@Suite("NEN-029 invariants")
struct ReverseFFIInvariantTests {

    /// **I1** — `cancel()` döndükten sonra hiçbir callback gelmez.
    ///
    /// `markCancelRequested()` kasıtlı olarak `cancel()` **döndükten sonra**
    /// çağrılıyor, önce değil — NEN-009'un tam aynı metodolojisi (bkz.
    /// `spike-async-cancel/apple-harness/Tests/.../InvariantTests.swift` ve
    /// bu spike'ın `lib.rs` modül dokümanı). İnvariant'ın kesin sınırı budur:
    /// "cancel() çağrıldı" değil, "cancel() döndü".
    @Test("cancel() returns → zero further callbacks, ever")
    func noCallbackAfterCancelReturns() {
        let observer = RecordingObserver()
        let params = TickParams(
            sessionId: 1, hz: 500.0, runDurationMs: 10_000, mediaDurationMs: 600_000, seekLatencyMs: 5)
        let engine = ReverseEngine.start(params: params, observer: observer)

        let firstArrived = observer.firstPosition.wait(timeout: .now() + 3) == .success
        #expect(firstArrived, "expected at least one tick before cancelling")

        engine.cancel()
        observer.markCancelRequested()
        // Gate is closed; give the tick thread a moment to observe it on
        // its next loop iteration and stop attempting deliveries, then join.
        Thread.sleep(forTimeInterval: 0.2)
        engine.shutdown()

        #expect(observer.afterCancel == 0, "callback delivered after cancel() returned")
    }

    /// **I4** (Direction A) — tekrarlı start+shutdown (50 kez) sonrası
    /// `live_engines()` sızıntısız 0'a dönüyor.
    @Test("50 start+shutdown cycles leave zero live engines")
    func repeatedEngineCyclesLeaveNoLiveEngines() {
        for _ in 0..<50 {
            let observer = RecordingObserver()
            let params = TickParams(
                sessionId: 1, hz: 200.0, runDurationMs: 30, mediaDurationMs: 60_000, seekLatencyMs: 2)
            let engine = ReverseEngine.start(params: params, observer: observer)
            engine.seek(targetMs: 1_000)
            Thread.sleep(forTimeInterval: 0.01)
            engine.shutdown()
        }
        #expect(liveEngines() == 0)
    }

    /// **I4** (Direction B) — tekrarlı `new`+`shutdown` (50 kez) sonrası
    /// `live_forward_sessions()` sızıntısız 0'a dönüyor. B'de thread yok; bu
    /// bir obje-ömrü sayacıdır, thread-canlılığı değil.
    @Test("50 new+shutdown cycles leave zero live forward sessions")
    func repeatedForwardSessionCyclesLeaveNoLiveSessions() {
        for _ in 0..<50 {
            let session = ForwardSession(sessionId: 2)
            try? session.reportFailure(code: .decoderInitFailed)
            session.shutdown()
            session.shutdown() // idempotent — must not double-decrement
        }
        #expect(liveForwardSessions() == 0)
    }

    /// Callback-triggered object release timing (DoD: "callback sonrası
    /// object release doğru anda mı oluyor"). `LifetimeTrackedObserver`
    /// counts its own live instances in `init`/`deinit`; the count must
    /// return to 0 after N start→shutdown cycles, proving the generated FFI
    /// glue releases its hold on the Swift observer at `shutdown()` (which
    /// clears the Rust-side `Arc<dyn PlaybackObserver>`), not later.
    @Test("observer object is released after engine shutdown, not retained by generated glue")
    func observerIsReleasedAfterShutdown() {
        for _ in 0..<20 {
            autoreleasepool {
                let observer = LifetimeTrackedObserver()
                let params = TickParams(
                    sessionId: 5, hz: 200.0, runDurationMs: 20, mediaDurationMs: 60_000, seekLatencyMs: 2)
                let engine = ReverseEngine.start(params: params, observer: observer)
                Thread.sleep(forTimeInterval: 0.01)
                engine.shutdown()
            }
        }
        #expect(
            LifetimeTrackedObserver.liveCount == 0,
            "observer instances leaked past shutdown: \(LifetimeTrackedObserver.liveCount)")
    }
}

final class LifetimeTrackedObserver: PlaybackObserver, @unchecked Sendable {
    static let lock = NSLock()
    // Manually synchronized by `lock` above, not by the compiler — same
    // opt-out spike-async-cancel's `@unchecked Sendable` classes use for
    // their own NSLock-guarded state.
    nonisolated(unsafe) static var liveCount = 0

    init() {
        Self.lock.lock(); Self.liveCount += 1; Self.lock.unlock()
    }
    deinit {
        Self.lock.lock(); Self.liveCount -= 1; Self.lock.unlock()
    }

    func onPosition(tick: PositionTick) {}
    func onStateChanged(sessionId: UInt64, seq: UInt32, state: PlaybackState) {}
    func onTrackSnapshot(snapshot: TrackSnapshot) {}
    func onSeekCompleted(sessionId: UInt64, seq: UInt32, targetMs: UInt64) {}
    func onError(error: EngineError) {}
}
