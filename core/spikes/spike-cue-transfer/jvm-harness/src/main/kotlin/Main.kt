// NEN-011 ölçüm harness'ı — ÜRÜN KODU DEĞİLDİR (CLAUDE.md kural 7).
//
//   SpikeCueTransfer full     <count>
//   SpikeCueTransfer windowed <count>
//
// `core/spikes/spike-cue-transfer/apple-harness/Sources/SpikeCueTransfer/main.swift`'in
// (NEN-008) Kotlin/JVM karşılığı — aynı iki yaklaşım, aynı ölçüm mantığı,
// karşılaştırılabilir kalması için birebir aynı tablo biçimi.
//
// Tek çağrıda bir yaklaşım ölçülür: peak heap'i tek process'te iki yaklaşım
// birbirini kirletir (Swift'in "peak RSS" ölçümüyle aynı gerekçe).
//
// **Metodolojik sapma (NEN-011 kanıt kaydına girer):** Swift `mach_task_basic_info`
// ile gerçek process RSS okuyordu. JVM'de bunun temiz bir karşılığı yok — GC
// ve JIT warm-up araya giriyor. Burada `MemoryPoolMXBean.peakUsage` (yalnız
// HEAP havuzları) kullanılıyor: JVM'in kendi tuttuğu, süreç boyunca sıfırlanana
// kadar büyüyen bir yüksek-su-işareti. RSS değil heap kullanımı ölçüyor — bug
// değil, iki runtime'ın bellek modelleri farklı olduğu için beklenen bir
// sapma.

import java.lang.management.ManagementFactory
import java.lang.management.MemoryType
import java.util.Locale
import kotlin.math.min
import uniffi.spike_cue_transfer.CueHandle
import uniffi.spike_cue_transfer.allCues
import uniffi.spike_cue_transfer.payloadBytes

// MARK: - Ölçüm araçları

fun percentile(sorted: LongArray, p: Int): Long {
    if (sorted.isEmpty()) return 0
    val rank = maxOf(1, (p * sorted.size + 99) / 100)
    return sorted[min(rank, sorted.size) - 1]
}

// Locale.ROOT: JVM varsayılan locale'i ondalık ayıracı olarak virgül
// kullanabiliyor (bu makinede tr_TR) — Swift `String(format:)` her zaman
// nokta kullanıyordu, sayıların karşılaştırılabilir kalması için burada da
// sabitleniyor.
fun ms(nanos: Long): String = String.format(Locale.ROOT, "%.3f ms", nanos / 1_000_000.0)
fun us(nanos: Long): String = String.format(Locale.ROOT, "%.2f µs", nanos / 1_000.0)
fun mib(bytes: Long): String = String.format(Locale.ROOT, "%.1f MiB", bytes / (1024.0 * 1024.0))

/** Yalnız HEAP havuzlarının toplam peak kullanımı — modül dokümanındaki not. */
fun peakHeapBytes(): Long =
    ManagementFactory.getMemoryPoolMXBeans()
        .filter { it.type == MemoryType.HEAP }
        .sumOf { it.peakUsage?.used ?: 0L }

fun currentHeapBytes(): Long =
    ManagementFactory.getMemoryMXBean().heapMemoryUsage.used

/** Deterministik ama sıralı olmayan konumlar — Swift `Positions` ile aynı LCG. */
class Positions {
    private var state: ULong = 0x2545_F491_4F6C_DD1DuL

    fun next(bound: ULong): ULong {
        state = state * 6_364_136_223_846_793_005uL + 1_442_695_040_888_963_407uL
        return if (bound == 0uL) 0uL else (state shr 33) % bound
    }
}

/** Optimizer'ın ölçülen çağrıyı silmesini engelleyen ve verinin gerçekten
 *  geldiğini kanıtlayan toplam. */
var checksum: ULong = 0uL

fun absorb(cues: List<uniffi.spike_cue_transfer.SpikeCue>) {
    for (cue in cues) {
        checksum += cue.endMs + cue.text.toByteArray(Charsets.UTF_8).size.toULong()
    }
}

// MARK: - Argümanlar

fun main(args: Array<String>) {
    if (args.size != 2) {
        System.err.println("kullanım: SpikeCueTransfer <full|windowed> <count>")
        kotlin.system.exitProcess(2)
    }
    val approach = args[0]
    val count = args[1].toUInt()

    val windowSize = 40u
    val randomAccessSamples = 10_000
    val fullListRepeats = 20

    val payload = payloadBytes(count)
    val baselineHeap = currentHeapBytes()

    // MARK: - Yaklaşım A — tam listeyi FFI'dan geçir

    if (approach == "full") {
        val samples = LongArray(fullListRepeats)
        var firstCall = 0L
        var received = 0

        for (repetition in 0 until fullListRepeats) {
            val start = System.nanoTime()
            val cues = allCues(count)
            val elapsed = System.nanoTime() - start

            samples[repetition] = elapsed
            if (repetition == 0) firstCall = elapsed
            received = cues.size
            absorb(cues)
        }

        val peakHeap = peakHeapBytes()
        val sorted = samples.sortedArray()

        println("## Yaklaşım A — tam liste (`allCues(count)`)")
        println()
        println("| Ölçüm | Değer |")
        println("|---|---|")
        println("| alınan cue | $received |")
        println("| payload (Rust tarafı) | ${mib(payload.toLong())} |")
        println("| ilk çağrı | ${ms(firstCall)} |")
        println("| p50 | ${ms(percentile(sorted, 50))} |")
        println("| p95 | ${ms(percentile(sorted, 95))} |")
        println("| min / max | ${ms(sorted.first())} / ${ms(sorted.last())} |")
        println("| **en uzun tek thread bloğu** | **${ms(sorted.last())}** |")
        println("| toplam ($fullListRepeats tekrar) | ${ms(samples.sum())} |")
        println("| peak heap | ${mib(peakHeap)} (başlangıç ${mib(baselineHeap)}) |")
        println()
        println("checksum: $checksum")
        return
    }

    // MARK: - Yaklaşım B — pencere / handle

    if (approach == "windowed") {
        val handleStart = System.nanoTime()
        val handle = CueHandle(count)
        val handleElapsed = System.nanoTime() - handleStart

        val afterHandleHeap = currentHeapBytes()
        val bound = if (count > windowSize) (count - windowSize).toULong() else 0uL
        val positions = Positions()

        // 1. Rastgele pencere erişimi.
        val windowSamples = LongArray(randomAccessSamples)
        for (i in 0 until randomAccessSamples) {
            val start = positions.next(bound).toUInt()
            val began = System.nanoTime()
            val window = handle.cues(start, windowSize)
            windowSamples[i] = System.nanoTime() - began
            absorb(window)
        }

        // 2. activeCue — playback sırasında her position güncellemesinde sorulan şey.
        val timelineMs = count.toULong() * 3_000uL
        val activeSamples = LongArray(randomAccessSamples)
        var found = 0
        for (i in 0 until randomAccessSamples) {
            val at = positions.next(timelineMs)
            val began = System.nanoTime()
            val cue = handle.activeCue(at)
            activeSamples[i] = System.nanoTime() - began
            if (cue != null) {
                found += 1
                checksum += cue.endMs
            }
        }

        // 3. Dokümanın TAMAMINI pencere pencere gezmek.
        val sweepStart = System.nanoTime()
        var swept = 0
        var cursor = 0u
        while (cursor < count) {
            val window = handle.cues(cursor, windowSize)
            swept += window.size
            absorb(window)
            cursor += windowSize
        }
        val sweepElapsed = System.nanoTime() - sweepStart

        val peakHeap = peakHeapBytes()
        val sortedWindows = windowSamples.sortedArray()
        val sortedActive = activeSamples.sortedArray()

        println("## Yaklaşım B — pencere / handle (`CueHandle`)")
        println()
        println("| Ölçüm | Değer |")
        println("|---|---|")
        println("| cue sayısı (handle) | ${handle.count()} |")
        println("| payload (Rust tarafı) | ${mib(payload.toLong())} |")
        println("| handle kurulumu | ${ms(handleElapsed)} |")
        println("| pencere $windowSize cue — p50 | ${us(percentile(sortedWindows, 50))} |")
        println("| pencere $windowSize cue — p95 | ${us(percentile(sortedWindows, 95))} |")
        println("| pencere — max | ${us(sortedWindows.last())} |")
        println("| `activeCue(atMs)` — p50 | ${us(percentile(sortedActive, 50))} |")
        println("| `activeCue(atMs)` — p95 | ${us(percentile(sortedActive, 95))} |")
        println("| `activeCue(atMs)` — max | ${us(sortedActive.last())} |")
        println(
            "| **en uzun tek thread bloğu** | **${ms(maxOf(sortedWindows.last(), sortedActive.last()))}** |")
        println(
            "| tüm dokümanı pencereyle gezmek | ${ms(sweepElapsed)} ($swept cue, ${(count + windowSize - 1u) / windowSize} çağrı) |")
        println("| heap handle kurulduktan sonra | ${mib(afterHandleHeap)} (başlangıç ${mib(baselineHeap)}) |")
        println("| peak heap | ${mib(peakHeap)} |")
        println()
        println("checksum: $checksum (bulunan cue: $found / $randomAccessSamples)")
        handle.destroy()
        return
    }

    System.err.println("bilinmeyen yaklaşım: $approach (full | windowed)")
    kotlin.system.exitProcess(2)
}
