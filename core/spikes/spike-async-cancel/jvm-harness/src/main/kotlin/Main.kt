// NEN-011 ölçüm harness'ı — ÜRÜN KODU DEĞİLDİR (CLAUDE.md kural 7).
//
// `core/spikes/spike-async-cancel/apple-harness/Sources/SpikeAsyncCancel/main.swift`'in
// (NEN-009) Kotlin/JVM karşılığı — checkpoint aralığı ile cancellation
// latency arasındaki ilişkiyi ölçer. Invariant'ların (I1/I2/I4) pass/fail
// kanıtı burada DEĞİL, src/test/kotlin/InvariantTests.kt'te.

import java.lang.management.ManagementFactory
import java.lang.management.MemoryType
import java.time.Instant
import java.util.Locale
import java.util.concurrent.Semaphore
import java.util.concurrent.locks.ReentrantLock
import kotlin.concurrent.withLock
import uniffi.spike_async_cancel.JobParams
import uniffi.spike_async_cancel.ProgressSink
import uniffi.spike_async_cancel.ProgressUpdate
import uniffi.spike_async_cancel.liveJobs
import uniffi.spike_async_cancel.startJob

// Locale.ROOT: JVM varsayılan locale'i (bu makinede tr_TR) ondalık ayıracı
// olarak virgül kullanıyor — Swift `String(format:)` her zaman nokta
// kullandığından, karşılaştırılabilirlik için burada da sabitleniyor.
fun ms(nanos: Long): String = String.format(Locale.ROOT, "%.3f ms", nanos / 1_000_000.0)

fun percentile(sorted: LongArray, p: Int): Long {
    if (sorted.isEmpty()) return 0
    val rank = maxOf(1, (p * sorted.size + 99) / 100)
    return sorted[minOf(rank, sorted.size) - 1]
}

fun peakHeapBytes(): Long =
    ManagementFactory.getMemoryPoolMXBeans()
        .filter { it.type == MemoryType.HEAP }
        .sumOf { it.peakUsage?.used ?: 0L }

fun currentHeapBytes(): Long = ManagementFactory.getMemoryMXBean().heapMemoryUsage.used

fun threadCount(): Int = Thread.activeCount()

/** Swift `MeasuringSink` ile aynı desen: her çağrıyı sayar, `cancelRequested`
 *  sonrası geleni ayrıca sayar (beklenen: her zaman 0), ilk çağrıda semaforu
 *  tetikler. */
class MeasuringSink : ProgressSink {
    val firstProgress = Semaphore(0)
    private val lock = ReentrantLock()
    private var signaled = false
    private var cancelRequested = false
    var afterCancelCount = 0
        private set

    override fun onProgress(update: ProgressUpdate) {
        lock.withLock {
            if (cancelRequested) afterCancelCount += 1
            val shouldSignal = !signaled
            signaled = true
            if (shouldSignal) firstProgress.release()
        }
    }

    override fun onCommit(jobId: ULong) {
        lock.withLock { if (cancelRequested) afterCancelCount += 1 }
    }

    fun markCancelRequested() {
        lock.withLock { cancelRequested = true }
    }
}

/**
 * Bir checkpoint aralığı için `repeats` kere ölç: iş başlar, ilk ilerleme
 * bildirimini bekle, hemen iptal et, `cancel()`→`join()` süresini örnekle.
 *
 * Swift eşdeğeriyle aynı not: `markCancelRequested()` burada `cancel()`
 * ÇAĞRILMADAN ÖNCE işaretleniyor — ölçülen şey I1'in kapı garantisi değil,
 * "iptale karar verildi" ile "fiilen cancel() çağrıldı" arasındaki dispatch
 * penceresi. I1'in kendisi InvariantTests.kt'te ayrıca kanıtlanıyor.
 */
fun measure(checkpointEvery: UInt, blockMicros: ULong, repeats: Int): Pair<LongArray, Int> {
    val samples = LongArray(repeats)
    var afterCancelTotal = 0

    for (i in 0 until repeats) {
        val sink = MeasuringSink()
        val params =
            JobParams(
                totalBlocks = 5_000_000u, blockMicros = blockMicros, checkpointEvery = checkpointEvery,
                attemptLateCommit = false)
        val handle = startJob(params, sink)

        sink.firstProgress.acquire()
        sink.markCancelRequested()
        val start = System.nanoTime()
        handle.cancel()
        handle.join()
        samples[i] = System.nanoTime() - start
        afterCancelTotal += sink.afterCancelCount
        handle.destroy()
    }

    return samples to afterCancelTotal
}

fun main() {
    println("## Bağlam")
    println()
    println("- cihaz: ${Runtime.getRuntime().availableProcessors()} çekirdek")
    println("- JVM: ${System.getProperty("java.vendor")} ${System.getProperty("java.version")}")
    println("- tarih: ${Instant.now()}")
    println()

    val baselineHeap = currentHeapBytes()
    val baselineThreads = threadCount()

    val blockMicros = 20uL
    val sweep = listOf(1u, 5u, 20u, 100u)
    val repeats = 200

    println("## Yaklaşım — checkpoint aralığı × cancellation latency")
    println()
    println("- block_micros: $blockMicros µs (sabit)")
    println("- repeats: $repeats (her checkpoint değeri için)")
    println()
    println("| checkpoint_every | ~aralık | p50 | p95 | max | dispatch penceresinde kaçan callback |")
    println("|---|---|---|---|---|---|")

    var grandAfterCancelTotal = 0
    for (checkpointEvery in sweep) {
        val (samples, afterCancelTotal) = measure(checkpointEvery, blockMicros, repeats)
        grandAfterCancelTotal += afterCancelTotal
        val sorted = samples.sortedArray()
        val interval = ms((checkpointEvery.toLong() * blockMicros.toLong()) * 1_000)
        println(
            "| $checkpointEvery | $interval | ${ms(percentile(sorted, 50))} | " +
                "${ms(percentile(sorted, 95))} | ${ms(sorted.last())} | $afterCancelTotal |")
    }

    println()
    println(
        "**Not:** yukarıdaki sayaç I1'in kendisi DEĞİL — dispatch penceresinde kaçan " +
            "callback sayısı (${sweep.size * repeats} çalıştırma toplamı: $grandAfterCancelTotal). " +
            "I1'in kendisi (cancel() DÖNDÜKTEN sonra sıfır callback) InvariantTests.kt'te " +
            "ayrıca ve deterministik biçimde kanıtlanıyor.")

    val peakHeap = peakHeapBytes()
    val threadsAfter = threadCount()
    println()
    println("## Kaynak bağlamı")
    println()
    println("| Ölçüm | Değer |")
    println("|---|---|")
    println("| live_jobs() (sweep sonrası) | ${liveJobs()} |")
    println("| thread sayısı — başlangıç → bitiş | $baselineThreads → $threadsAfter |")
    fun mib(bytes: Long) = String.format(Locale.ROOT, "%.2f", bytes / 1048576.0)
    println("| heap — başlangıç → şu an | ${mib(baselineHeap)} MiB → ${mib(currentHeapBytes())} MiB |")
    println("| peak heap | ${mib(peakHeap)} MiB |")
}
