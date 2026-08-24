// NEN-011 — typed error mapping, Kotlin/JVM tarafı.
// `apple-harness/Tests/SpikeTypedErrorsTests/TypedErrorTests.swift`'in
// (NEN-010) birebir karşılığı.
//
// **Kotlin-özgü isimlendirme bulguları (kanıt kaydına girer, kaynak
// incelemesiyle doğrulandı — bkz. `Main.kt` modül dokümanı):**
// 1. `AppError` → Kotlin'de `AppException` (Error→Exception son ek değişimi,
//    yalnız Kotlin backend'inde; Swift `AppError` kalıyor).
// 2. Düz `uniffi::Enum` varyantları (`Capability`, `ValidationField`) Kotlin'de
//    SCREAMING_SNAKE_CASE üretiliyor (`enum_variant_name` →
//    `to_shouty_snake_case()`), Swift'te ise camelCase (`.localAsr`) idi.
//    Swift zaten PascalCase-error/camelCase-enum asimetrisini bulmuştu; Kotlin
//    üçüncü bir kuralı (SCREAMING_SNAKE_CASE) ekliyor — üç binding boyunca üç
//    farklı isimlendirme kuralı.

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.fail
import uniffi.spike_typed_errors.AppException
import uniffi.spike_typed_errors.Capability
import uniffi.spike_typed_errors.ValidationField
import uniffi.spike_typed_errors.triggerCancelled
import uniffi.spike_typed_errors.triggerCapabilityUnavailable
import uniffi.spike_typed_errors.triggerNetworkError
import uniffi.spike_typed_errors.triggerParseError
import uniffi.spike_typed_errors.triggerValidationError

private const val PRIVATE_PATH = "/Users/alice/Movies/Really Personal/S01E01.mkv"
private const val MEDIA_URL_WITH_TOKEN =
    "https://media.example.com/stream/season1/ep01.mkv?token=eyJhbGciOiJIUzI1NiJ9.super-secret-payload"

class TypedErrorTests {

    /** DoD #1 — her varyant Kotlin tarafında `when` ile, mesaj string'i parse
     *  edilmeden ayrıştırılabiliyor. `else` dalı YOK: derleyici sealed class
     *  exhaustiveness'i zorluyor (Swift'in `default:`'sız `switch`'inin
     *  karşılığı). */
    @Test
    fun everyVariantSwitchesWithoutStringParsing() {
        fun classify(error: AppException): String =
            when (error) {
                is AppException.Parse -> "parse:${error.extension ?: "none"}:${error.line}"
                is AppException.Network -> "network:${error.host}:${error.status}"
                is AppException.Cancelled -> "cancelled"
                is AppException.CapabilityUnavailable -> "capability:${error.capability}"
                is AppException.Validation -> "validation:${error.field}"
            }

        val parse =
            try {
                triggerParseError(PRIVATE_PATH, 7u)
                fail("expected triggerParseError to throw")
            } catch (e: AppException.Parse) {
                e
            }
        assertEquals("parse:mkv:7", classify(parse))

        val network =
            try {
                triggerNetworkError(MEDIA_URL_WITH_TOKEN, 503u)
                fail("expected triggerNetworkError to throw")
            } catch (e: AppException.Network) {
                e
            }
        // media.example.com bu spike'ın allowlist'inde değil.
        assertEquals("network:<redacted-host>:503", classify(network))

        val allowlisted =
            try {
                triggerNetworkError("https://api.opensubtitles.com/v1/x", 200u)
                fail("expected triggerNetworkError to throw")
            } catch (e: AppException.Network) {
                e
            }
        assertEquals("network:api.opensubtitles.com:200", classify(allowlisted))

        try {
            triggerCancelled()
            fail("expected triggerCancelled to throw")
        } catch (e: AppException.Cancelled) {
            assertEquals("cancelled", classify(e))
        }

        val capability =
            try {
                triggerCapabilityUnavailable(Capability.CLOUD_TRANSLATION)
                fail("expected triggerCapabilityUnavailable to throw")
            } catch (e: AppException.CapabilityUnavailable) {
                e
            }
        assertEquals("capability:CLOUD_TRANSLATION", classify(capability))

        val validation =
            try {
                triggerValidationError(ValidationField.MEDIA_DURATION)
                fail("expected triggerValidationError to throw")
            } catch (e: AppException.Validation) {
                e
            }
        assertEquals("validation:MEDIA_DURATION", classify(validation))
    }

    /** DoD #2 — payload'lı bir varyantın Kotlin'in KENDİ varsayılan
     *  `toString()`'i yasaklı desen içermiyor. Rust'ın elle yazılmış `Debug`'ı
     *  burada devrede değil (bkz. `lib.rs` modül dokümanı) — FFI sınırını
     *  gerçekten geçmiş bir hatayı Kotlin'in kendi basımıyla kontrol ediyoruz. */
    @Test
    fun defaultKotlinPrintingHasNoForbiddenPattern() {
        val forbidden = listOf(PRIVATE_PATH, MEDIA_URL_WITH_TOKEN, "media.example.com")

        fun assertSafe(error: AppException, label: String) {
            val printed = error.toString()
            for (pattern in forbidden) {
                assertFalse(
                    printed.contains(pattern),
                    "$label's default Kotlin printing leaked $pattern: $printed")
            }
        }

        try {
            triggerParseError(PRIVATE_PATH, 7u)
        } catch (e: AppException.Parse) {
            assertSafe(e, "parse")
        }

        try {
            triggerNetworkError(MEDIA_URL_WITH_TOKEN, 503u)
        } catch (e: AppException.Network) {
            assertSafe(e, "network")
        }
    }
}
