// NEN-029 ölçüm harness'ı — ÜRÜN KODU DEĞİLDİR (CLAUDE.md kural 7).
//
//   SpikeReverseFFI
//
// A (core-owned, reverse callback) ile B (shell-owned, forward call)
// arasında ölçülü bir A/B karşılaştırması: 4 Hz ve 60 Hz'de per-call maliyet,
// A için ayrıca MainActor hop maliyeti ve callback'in hangi thread'e
// düştüğü, ardından A'ya özgü bir "backgrounding" proxy deneyi.
//
// Invariant'ların (I1/I4) pass/fail kanıtı burada DEĞİL,
// Tests/SpikeReverseFFITests/'te — bu dosya yalnız baseline sayıları
// raporlar (M1-core-spike.md → "Baseline ölçümleri").
//
// MainActor'a zamanlanmış işin (A'nın hop ölçümü) düzgün akabilmesi için
// tüm ölçüm işi bir arka plan kuyruğunda çalışır; asıl process ana thread'i
// yalnızca `dispatchMain()` ile dispatch main queue'yu pompalar — gerçek bir
// uygulamanın UI thread'inin hiç senkron bloklanmaması gibi. Ölçüm mantığını
// senkron `Thread.sleep` ile ana thread'de yazıp MainActor işini kendi
// bloğumuzla ac/dolaylı olarak açlığa düşürmek, hop maliyetini "gerçek hop
// maliyeti" değil "ana thread ne zaman müsait oldu" yapardı.

import Darwin
import Foundation
import SpikeCore

// MARK: - Ölçüm araçları (spike-async-cancel/apple-harness/main.swift ile aynı desen)

@inline(__always)
func nowNanos() -> UInt64 {
    clock_gettime_nsec_np(CLOCK_UPTIME_RAW)
}

func percentile(_ sorted: [UInt64], _ p: Int) -> UInt64 {
    guard !sorted.isEmpty else { return 0 }
    let rank = max(1, (p * sorted.count + 99) / 100)
    return sorted[min(rank, sorted.count) - 1]
}

func us(_ nanos: UInt64) -> String {
    String(format: "%.2f µs", Double(nanos) / 1_000)
}

// MARK: - Direction A — MeasuringObserver

/// Records position-arrival thread identity and a MainActor-hop cost sample
/// per tick (Task { @MainActor in ... }, timed from receipt to execution).
/// State-change/track/seek/error callbacks are counted but not hop-measured
/// — the DoD's 4 Hz/60 Hz comparison is specifically about position updates.
final class MeasuringObserver: PlaybackObserver, @unchecked Sendable {
    private let lock = NSLock()
    private var hopCostsNs: [UInt64] = []
    private var onMainCount = 0
    private var offMainCount = 0
    private var positionCount = 0
    let firstPosition = DispatchSemaphore(value: 0)
    private var signaled = false

    func onPosition(tick: PositionTick) {
        let t0 = nowNanos()
        let onMain = Thread.isMainThread
        lock.lock()
        positionCount += 1
        if onMain { onMainCount += 1 } else { offMainCount += 1 }
        let shouldSignal = !signaled
        signaled = true
        lock.unlock()
        if shouldSignal { firstPosition.signal() }

        Task { @MainActor in
            let t1 = nowNanos()
            self.recordHop(t1 &- t0)
        }
    }

    private func recordHop(_ ns: UInt64) {
        lock.lock(); hopCostsNs.append(ns); lock.unlock()
    }

    func onStateChanged(sessionId: UInt64, seq: UInt32, state: PlaybackState) {}
    func onTrackSnapshot(snapshot: TrackSnapshot) {}
    func onSeekCompleted(sessionId: UInt64, seq: UInt32, targetMs: UInt64) {}
    func onError(error: EngineError) {}

    var snapshot: (positionCount: Int, onMain: Int, offMain: Int, hopCosts: [UInt64]) {
        lock.lock(); defer { lock.unlock() }
        return (positionCount, onMainCount, offMainCount, hopCostsNs)
    }
}

/// A second observer, deliberately `@MainActor`-isolated, checking what
/// Swift 6 strict concurrency actually requires for a synchronous foreign
/// protocol conformance — a real, previously unmeasured toolchain fact
/// (not previously exercised by any M1 spike). If this type did not compile,
/// that failure would itself be the finding; it does compile, and the
/// pattern it needed (`nonisolated` methods reaching back in via
/// `MainActor.assumeIsolated`) is the finding instead.
@MainActor
final class MainActorObserver: PlaybackObserver {
    private(set) var isolatedPositionCount = 0

    nonisolated func onPosition(tick: PositionTick) {
        MainActor.assumeIsolated {
            self.isolatedPositionCount += 1
        }
    }
    nonisolated func onStateChanged(sessionId: UInt64, seq: UInt32, state: PlaybackState) {}
    nonisolated func onTrackSnapshot(snapshot: TrackSnapshot) {}
    nonisolated func onSeekCompleted(sessionId: UInt64, seq: UInt32, targetMs: UInt64) {}
    nonisolated func onError(error: EngineError) {}
}

struct ASample {
    let hz: Double
    let sampleCount: Int
    let costSorted: [UInt64]
    let hopSorted: [UInt64]
    let onMain: Int
    let offMain: Int
}

func measureDirectionA(hz: Double, runDurationMs: UInt64, hopSettleMs: UInt64) -> ASample {
    let observer = MeasuringObserver()
    let params = TickParams(
        sessionId: 1, hz: hz, runDurationMs: runDurationMs, mediaDurationMs: 600_000, seekLatencyMs: 5)
    let engine = ReverseEngine.start(params: params, observer: observer)

    observer.firstPosition.wait()
    while engine.isRunning() {
        Thread.sleep(forTimeInterval: 0.02)
    }
    // Let any still-queued MainActor hop tasks drain before reading them.
    Thread.sleep(forTimeInterval: Double(hopSettleMs) / 1000)

    let costSamples = engine.callCostSamplesNs()
    let snap = observer.snapshot
    engine.shutdown()

    return ASample(
        hz: hz, sampleCount: snap.positionCount, costSorted: costSamples.sorted(),
        hopSorted: snap.hopCosts.sorted(), onMain: snap.onMain, offMain: snap.offMain)
}

// MARK: - Direction B — forward-call timer

struct BSample {
    let hz: Double
    let sampleCount: Int
    let costSorted: [UInt64]
}

func measureDirectionB(hz: Double, runDurationMs: UInt64) -> BSample {
    let session = ForwardSession(sessionId: 2)
    let lock = NSLock()
    var costs: [UInt64] = []
    var seq: UInt32 = 0

    let queue = DispatchQueue(label: "spike.reverse-ffi.forward-timer")
    let timer = DispatchSource.makeTimerSource(queue: queue)
    let deadline = DispatchTime.now() + .milliseconds(Int(runDurationMs))
    let done = DispatchSemaphore(value: 0)

    timer.schedule(deadline: .now(), repeating: 1.0 / hz)
    timer.setEventHandler {
        if DispatchTime.now() > deadline {
            timer.cancel()
            done.signal()
            return
        }
        let thisSeq = seq
        seq += 1
        let t0 = nowNanos()
        try? session.reportPosition(seq: thisSeq, positionMs: UInt64(thisSeq) * UInt64(1000.0 / hz))
        let elapsed = nowNanos() &- t0
        lock.lock(); costs.append(elapsed); lock.unlock()
    }
    timer.resume()
    done.wait()
    session.shutdown()

    let sorted = costs.sorted()
    return BSample(hz: hz, sampleCount: sorted.count, costSorted: sorted)
}

// MARK: - Backgrounding proxy (A only — see module doc)

/// Suspends the Swift-side callback-draining queue while Rust's tick thread
/// keeps calling `onPosition`, then resumes and reports backlog/coalescing
/// behavior. A CLI harness has no real `UIApplication`/`NSApplication`
/// lifecycle to hook — this is a documented proxy, not a real OS-lifecycle
/// test. Only meaningful for A: B's "producer" is the shell's own timer,
/// which would simply stop calling in a real backgrounding scenario, so
/// there is no analogous backlog risk to probe on that side.
final class QueuedObserver: PlaybackObserver, @unchecked Sendable {
    let queue = DispatchQueue(label: "spike.reverse-ffi.background-proxy")
    private let lock = NSLock()
    private var drained = 0
    private var maxDepth = 0
    private var pending = 0

    func onPosition(tick: PositionTick) {
        lock.lock(); pending += 1; maxDepth = max(maxDepth, pending); lock.unlock()
        queue.async { [self] in
            lock.lock(); pending -= 1; drained += 1; lock.unlock()
        }
    }
    func onStateChanged(sessionId: UInt64, seq: UInt32, state: PlaybackState) {}
    func onTrackSnapshot(snapshot: TrackSnapshot) {}
    func onSeekCompleted(sessionId: UInt64, seq: UInt32, targetMs: UInt64) {}
    func onError(error: EngineError) {}

    var stats: (drained: Int, maxDepth: Int, pendingNow: Int) {
        lock.lock(); defer { lock.unlock() }
        return (drained, maxDepth, pending)
    }
}

func measureBackgroundingProxy() -> String {
    let observer = QueuedObserver()
    let params = TickParams(sessionId: 3, hz: 30.0, runDurationMs: 6_000, mediaDurationMs: 600_000, seekLatencyMs: 5)
    let engine = ReverseEngine.start(params: params, observer: observer)

    Thread.sleep(forTimeInterval: 0.3) // let a few ticks land normally first
    observer.queue.suspend()
    let suspendedFor = 2.0
    Thread.sleep(forTimeInterval: suspendedFor) // Rust keeps ticking — deliver() only enqueues, never blocks on the suspended queue
    observer.queue.resume()

    while engine.isRunning() {
        Thread.sleep(forTimeInterval: 0.05)
    }
    Thread.sleep(forTimeInterval: 0.3) // drain settle
    let stats = observer.stats
    engine.shutdown()

    let expectedDuringSuspend = Int(suspendedFor * 30.0)
    return "suspended \(suspendedFor)s at 30 Hz (~\(expectedDuringSuspend) ticks expected) → "
        + "max backlog depth \(stats.maxDepth), drained \(stats.drained) total, "
        + "pending after 0.3s settle: \(stats.pendingNow)"
}

// MARK: - Run

func runAll() {
    print("## Bağlam")
    print("")
    print("- cihaz: \(ProcessInfo.processInfo.processorCount) çekirdek")
    print("- OS: \(ProcessInfo.processInfo.operatingSystemVersionString)")
    print("- tarih: \(Date())")
    print("")

    let sweeps: [(hz: Double, aDurationMs: UInt64, bDurationMs: UInt64)] = [
        (4.0, 30_000, 30_000),
        (60.0, 5_000, 5_000),
    ]

    print("## Yaklaşım — A (core-owned, reverse callback)")
    print("")
    print("- her hz için tek koşu; süre sample sayısını belirliyor (baseline, kontrollü N değil)")
    print("")
    print("| hz | pozisyon sample | per-call maliyet p50/p95/max | MainActor hop p50/p95/max | callback thread'i |")
    print("|---|---|---|---|---|")
    var aSamples: [ASample] = []
    for sweep in sweeps {
        let sample = measureDirectionA(hz: sweep.hz, runDurationMs: sweep.aDurationMs, hopSettleMs: 300)
        aSamples.append(sample)
        let threadNote = sample.onMain == 0
            ? "hep ana thread DIŞINDA (\(sample.offMain)/\(sample.offMain + sample.onMain))"
            : "\(sample.onMain)/\(sample.offMain + sample.onMain) ana thread'de — beklenmedik"
        print(
            "| \(Int(sweep.hz)) | \(sample.sampleCount) | "
                + "\(us(percentile(sample.costSorted, 50)))/\(us(percentile(sample.costSorted, 95)))/\(us(sample.costSorted.last ?? 0)) | "
                + "\(us(percentile(sample.hopSorted, 50)))/\(us(percentile(sample.hopSorted, 95)))/\(us(sample.hopSorted.last ?? 0)) | "
                + "\(threadNote) |")
    }

    print("")
    print("## Yaklaşım — B (shell-owned, forward call)")
    print("")
    print("- Swift kendi `DispatchSourceTimer`'ıyla sürüyor; ters çağrı yok, dolayısıyla ölçülecek bir thread-hop de yok")
    print("")
    print("| hz | pozisyon sample | per-call maliyet (Swift ölçümü) p50/p95/max |")
    print("|---|---|---|")
    var bSamples: [BSample] = []
    for sweep in sweeps {
        let sample = measureDirectionB(hz: sweep.hz, runDurationMs: sweep.bDurationMs)
        bSamples.append(sample)
        print(
            "| \(Int(sweep.hz)) | \(sample.sampleCount) | "
                + "\(us(percentile(sample.costSorted, 50)))/\(us(percentile(sample.costSorted, 95)))/\(us(sample.costSorted.last ?? 0)) |")
    }

    print("")
    print("**Not:** A'nın MainActor-hop sütunu B'de yok çünkü B'de ters çağrı hiç yok —")
    print("Swift her zaman çağıran taraf. Bu asimetri (A her event için thread-hop vergisi")
    print("öder, B hiç ödemez) karşılaştırmanın baş bulgusu; sayısal tablo bunun kanıtı.")

    print("")
    print("## Yüksek frekanslı FFI trafiği — 60 Hz'de tahmini toplam maliyet/sn")
    print("")
    if let a60 = aSamples.first(where: { $0.hz == 60.0 }), let b60 = bSamples.first(where: { $0.hz == 60.0 }) {
        // Per-call cost is in nanoseconds; ×60 events/sec, then ÷1_000_000 to
        // report milliseconds of FFI-attributable cost per second of runtime.
        let aTotalMs = (Double(percentile(a60.costSorted, 50)) + Double(percentile(a60.hopSorted, 50))) * 60.0 / 1_000_000.0
        let bTotalMs = Double(percentile(b60.costSorted, 50)) * 60.0 / 1_000_000.0
        print("- A: ~\(String(format: "%.3f", aTotalMs)) ms/sn (call + hop maliyeti × 60)")
        print("- B: ~\(String(format: "%.3f", bTotalMs)) ms/sn (call maliyeti × 60, hop yok)")
    }

    print("")
    print("## Backgrounding proxy (yalnız A — bkz. modül dokümanı, gerçek OS lifecycle DEĞİL)")
    print("")
    print("- \(measureBackgroundingProxy())")

    print("")
    print("## MainActor-izoleli observer uygunluğu (araç zinciri bulgusu)")
    print("")
    print("- `@MainActor final class MainActorObserver: PlaybackObserver` DERLENİYOR —")
    print("  ama yalnız her protokol metodu `nonisolated` işaretlenip MainActor'a ait")
    print("  duruma dokunmak için `MainActor.assumeIsolated { ... }` kullanılınca.")
    print("  Swift 6 strict concurrency, senkron bir foreign-trait metodunu doğrudan")
    print("  actor-isolated bırakmıyor — bu spike'tan önce ölçülmemiş bir araç zinciri gerçeği.")
    let mainActorObserver = MainActorObserver()
    let miniParams = TickParams(sessionId: 4, hz: 20.0, runDurationMs: 200, mediaDurationMs: 60_000, seekLatencyMs: 5)
    let miniEngine = ReverseEngine.start(params: miniParams, observer: mainActorObserver)
    while miniEngine.isRunning() {
        Thread.sleep(forTimeInterval: 0.02)
    }
    miniEngine.shutdown()

    print("")
    print("## Kaynak bağlamı")
    print("")
    print("| Ölçüm | Değer |")
    print("|---|---|")
    print("| live_engines() (koşu sonrası) | \(liveEngines()) |")
    print("| live_forward_sessions() (koşu sonrası) | \(liveForwardSessions()) |")
}

DispatchQueue.global(qos: .userInitiated).async {
    runAll()
    exit(0)
}
dispatchMain()
