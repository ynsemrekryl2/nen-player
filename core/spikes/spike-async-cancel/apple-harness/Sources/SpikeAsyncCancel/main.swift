// NEN-009 ölçüm harness'ı — ÜRÜN KODU DEĞİLDİR (CLAUDE.md kural 7).
//
//   SpikeAsyncCancel
//
// Checkpoint aralığı ile cancellation latency arasındaki ilişkiyi ölçer:
// her sweep değeri için bir iş başlatılır, ilk ilerleme bildirimi gelir
// gelmez iptal edilir, `cancel()` çağrısından `join()` dönüşüne kadar geçen
// süre örneklenir. Bu, gerçek "kullanıcı iptal düğmesine bastı" senaryosunun
// ölçtüğü şeydir — Rust ile Swift arasında saat senkronizasyonu gerekmez,
// çünkü hem başlangıç hem bitiş aynı (Swift) thread'de ölçülüyor.
//
// Invariant'ların (I1/I2/I4) pass/fail kanıtı burada DEĞİL,
// Tests/SpikeAsyncCancelTests/InvariantTests.swift'te — bu dosya yalnız
// baseline sayıları raporlar (M1-core-spike.md → "Baseline ölçümleri").

import Darwin
import Foundation
import SpikeCore

// MARK: - Ölçüm araçları (spike-cue-transfer/apple-harness/main.swift ile aynı desen)

@inline(__always)
func nowNanos() -> UInt64 {
    clock_gettime_nsec_np(CLOCK_UPTIME_RAW)
}

func percentile(_ sorted: [UInt64], _ p: Int) -> UInt64 {
    guard !sorted.isEmpty else { return 0 }
    let rank = max(1, (p * sorted.count + 99) / 100)
    return sorted[min(rank, sorted.count) - 1]
}

func ms(_ nanos: UInt64) -> String {
    String(format: "%.3f ms", Double(nanos) / 1_000_000)
}

func mib(_ bytes: UInt64) -> String {
    String(format: "%.2f MiB", Double(bytes) / (1024 * 1024))
}

func residentSize() -> (current: UInt64, peak: UInt64) {
    var info = mach_task_basic_info()
    var count = mach_msg_type_number_t(
        MemoryLayout<mach_task_basic_info>.size / MemoryLayout<natural_t>.size)

    let result = withUnsafeMutablePointer(to: &info) { pointer in
        pointer.withMemoryRebound(to: integer_t.self, capacity: Int(count)) { rebound in
            task_info(mach_task_self_, task_flavor_t(MACH_TASK_BASIC_INFO), rebound, &count)
        }
    }

    guard result == KERN_SUCCESS else { return (0, 0) }
    return (info.resident_size, info.resident_size_max)
}

/// Process'in şu anki thread sayısı — I4'ün "OS thread sızmıyor" iddiasının
/// bağlamsal kanıtı (sert eşik değil, `main.swift` yalnız raporlar; pass/fail
/// olan `live_jobs()` invariant testinde).
func threadCount() -> Int {
    var list: thread_act_array_t?
    var count: mach_msg_type_number_t = 0
    let result = task_threads(mach_task_self_, &list, &count)
    guard result == KERN_SUCCESS, let list else { return -1 }
    defer {
        vm_deallocate(
            mach_task_self_, vm_address_t(bitPattern: list), vm_size_t(count) * vm_size_t(MemoryLayout<thread_t>.stride))
    }
    return Int(count)
}

// MARK: - Sink

/// Her ilerleme çağrısını sayar; `cancelRequested` sonrası gelen her çağrıyı
/// ayrıca sayar (beklenen: her zaman 0). İlk çağrıda semaphore'u tetikler —
/// "iş gerçekten başladı" beklemesi için `sleep` yerine bu kullanılıyor.
final class MeasuringSink: ProgressSink, @unchecked Sendable {
    let firstProgress = DispatchSemaphore(value: 0)
    private let lock = NSLock()
    private var signaled = false
    private var cancelRequested = false
    private(set) var afterCancelCount = 0

    func onProgress(update: ProgressUpdate) {
        lock.lock()
        if cancelRequested { afterCancelCount += 1 }
        let shouldSignal = !signaled
        signaled = true
        lock.unlock()
        if shouldSignal { firstProgress.signal() }
    }

    func onCommit(jobId: UInt64) {
        lock.lock()
        if cancelRequested { afterCancelCount += 1 }
        lock.unlock()
    }

    func markCancelRequested() {
        lock.lock(); cancelRequested = true; lock.unlock()
    }
}

// MARK: - Sweep

/// Bir checkpoint aralığı için `repeats` kere ölç: iş başlar, ilk ilerleme
/// bildirimini bekle, hemen iptal et, `cancel()`→`join()` süresini örnekle.
///
/// `markCancelRequested()` burada — InvariantTests.swift'in tersine —
/// `cancel()` ÇAĞRILMADAN ÖNCE işaretleniyor: bu fonksiyonun ölçtüğü şey
/// I1'in kapı garantisi değil, "kullanıcı iptale karar verdi" ile "Swift'in
/// `cancel()`'ı fiilen çağırması" arasındaki dispatch penceresi. O pencerede
/// gerçek bir callback kaçabilir — kapı henüz kapanmadığı için bu meşru,
/// I1'i ihlal etmez. I1'in kendisi (cancel() DÖNDÜKTEN sonra sıfır callback)
/// InvariantTests.swift'te ayrıca ve airtight biçimde kanıtlanıyor.
func measure(checkpointEvery: UInt32, blockMicros: UInt64, repeats: Int) -> (samples: [UInt64], afterCancelTotal: Int) {
    var samples: [UInt64] = []
    samples.reserveCapacity(repeats)
    var afterCancelTotal = 0

    for _ in 0..<repeats {
        let sink = MeasuringSink()
        let params = JobParams(
            totalBlocks: 5_000_000, blockMicros: blockMicros, checkpointEvery: checkpointEvery,
            attemptLateCommit: false)
        let handle = startJob(params: params, sink: sink)

        sink.firstProgress.wait()
        sink.markCancelRequested()
        let start = nowNanos()
        handle.cancel()
        _ = handle.join()
        samples.append(nowNanos() - start)
        afterCancelTotal += sink.afterCancelCount // dispatch penceresi kaçağı — I1 değil
    }

    return (samples, afterCancelTotal)
}

// MARK: - Ölçüm bağlamı

print("## Bağlam")
print("")
print("- cihaz: \(ProcessInfo.processInfo.processorCount) çekirdek")
print("- OS: \(ProcessInfo.processInfo.operatingSystemVersionString)")
print("- tarih: \(Date())")
print("")

let baselineRss = residentSize().current
let baselineThreads = threadCount()

// blockMicros sabit tutulup checkpointEvery süpürülüyor — DoD'un istediği
// "checkpoint aralığı ile latency arasındaki ilişki".
let blockMicros: UInt64 = 20
let sweep: [UInt32] = [1, 5, 20, 100]
let repeats = 200

print("## Yaklaşım — checkpoint aralığı × cancellation latency")
print("")
print("- block_micros: \(blockMicros) µs (sabit)")
print("- repeats: \(repeats) (her checkpoint değeri için)")
print("")
print("| checkpoint_every | ~aralık | p50 | p95 | max | dispatch penceresinde kaçan callback |")
print("|---|---|---|---|---|---|")

var grandAfterCancelTotal = 0
for checkpointEvery in sweep {
    let (samples, afterCancelTotal) = measure(
        checkpointEvery: checkpointEvery, blockMicros: blockMicros, repeats: repeats)
    grandAfterCancelTotal += afterCancelTotal
    let sorted = samples.sorted()
    let interval = ms(UInt64(checkpointEvery) * blockMicros * 1_000)
    print(
        "| \(checkpointEvery) | \(interval) | \(ms(percentile(sorted, 50))) | "
            + "\(ms(percentile(sorted, 95))) | \(ms(sorted.last ?? 0)) | \(afterCancelTotal) |")
}

print("")
print(
    "**Not:** yukarıdaki sayaç I1'in kendisi DEĞİL — “iptale karar verildi” ile "
    + "“cancel() fiilen çağrıldı” arasındaki dispatch penceresinde kaçan callback "
    + "sayısı (\(sweep.count * repeats) çalıştırma toplamı: \(grandAfterCancelTotal)). "
    + "I1'in kendisi (cancel() DÖNDÜKTEN sonra sıfır callback — kapı garantisi) "
    + "InvariantTests.swift'te ayrıca ve deterministik biçimde kanıtlanıyor.")

let rss = residentSize()
let threadsAfter = threadCount()
print("")
print("## Kaynak bağlamı")
print("")
print("| Ölçüm | Değer |")
print("|---|---|")
print("| live_jobs() (sweep sonrası) | \(liveJobs()) |")
print("| thread sayısı — başlangıç → bitiş | \(baselineThreads) → \(threadsAfter) |")
print("| RSS — başlangıç → şu an | \(mib(baselineRss)) → \(mib(rss.current)) |")
print("| peak RSS | \(mib(rss.peak)) |")
