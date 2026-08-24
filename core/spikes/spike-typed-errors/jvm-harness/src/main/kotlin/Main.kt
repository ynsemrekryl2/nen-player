// NEN-011 ölçüm harness'ı — ÜRÜN KODU DEĞİLDİR (CLAUDE.md kural 7).
//
// `core/spikes/spike-typed-errors/apple-harness/Sources/SpikeTypedErrors/main.swift`'in
// (NEN-010) Kotlin/JVM karşılığı — eşleme maliyeti: throw → catch → when
// round-trip. Invariant'ların (I3) pass/fail kanıtı burada DEĞİL,
// src/test/kotlin/TypedErrorTests.kt'te.
//
// **Kotlin-özgü isimlendirme bulgusu (kanıt kaydına girer):** uniffi_bindgen
// 0.32'nin Kotlin backend'i, adı "Error" ile biten bir `uniffi::Error`
// tipini otomatik olarak "Exception" ile değiştiriyor (bkz.
// `bindings/kotlin/gen_kotlin/mod.rs::convert_error_suffix`, kaynak
// incelemesiyle doğrulandı) — yani Rust'taki `AppError`, Swift'te `AppError`
// kalırken Kotlin'de **`AppException`** olarak üretiliyor. Bu, Swift'in
// PascalCase/camelCase asimetrisinden (NEN-010 kanıt kaydı) AYRI, yalnız
// Kotlin backend'ine özgü ikinci bir isimlendirme sapması.

import java.time.Instant
import java.util.Locale
import uniffi.spike_typed_errors.AppException
import uniffi.spike_typed_errors.triggerParseError

// Locale.ROOT: JVM varsayılan locale'i (bu makinede tr_TR) ondalık ayıracı
// olarak virgül kullanıyor — Swift `String(format:)` her zaman nokta
// kullandığından, karşılaştırılabilirlik için burada da sabitleniyor.
fun us(nanos: Long): String = String.format(Locale.ROOT, "%.2f µs", nanos / 1_000.0)

fun percentile(sorted: LongArray, p: Int): Long {
    if (sorted.isEmpty()) return 0
    val rank = maxOf(1, (p * sorted.size + 99) / 100)
    return sorted[minOf(rank, sorted.size) - 1]
}

/**
 * Eşleme maliyeti: throw → catch → when round-trip. `triggerParseError` en
 * pahalı varyant (iki alan + nullable String); Swift eşdeğeriyle aynı
 * gerekçe — ölçülen şey "hangi varyant" değil FFI + when mekanizmasının
 * sabit maliyeti.
 */
fun measureMappingCost(repeats: Int): LongArray {
    val samples = LongArray(repeats)
    var written = 0
    for (i in 0 until repeats) {
        val start = System.nanoTime()
        try {
            triggerParseError("/dev/null", i.toUInt())
        } catch (e: AppException.Parse) {
            @Suppress("UNUSED_EXPRESSION")
            e.extension to e.line
        }
        samples[written] = System.nanoTime() - start
        written += 1
    }
    return samples
}

fun main() {
    println("## Bağlam")
    println()
    println("- cihaz: ${Runtime.getRuntime().availableProcessors()} çekirdek")
    println("- JVM: ${System.getProperty("java.vendor")} ${System.getProperty("java.version")}")
    println("- tarih: ${Instant.now()}")
    println()

    val variantCount = 5
    println("## Varyant sayısı")
    println()
    println("`AppException` (Rust: `AppError`): $variantCount varyant (Parse, Network, Cancelled, CapabilityUnavailable, Validation).")
    println()

    val repeats = 10_000
    val samples = measureMappingCost(repeats).sortedArray()

    println("## Eşleme maliyeti — throw → catch → when round-trip")
    println()
    println("- repeats: $repeats")
    println()
    println("| p50 | p95 | max |")
    println("|---|---|---|")
    println("| ${us(percentile(samples, 50))} | ${us(percentile(samples, 95))} | ${us(samples.last())} |")
}
