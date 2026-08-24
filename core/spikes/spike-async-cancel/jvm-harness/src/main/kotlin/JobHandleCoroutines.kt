// NEN-011 — task metninin "suspend fonksiyon davranışı, coroutine iptali ile
// JobHandle.cancel() uyumu" maddesinin uygulaması.
//
// **Önemli bulgu (kanıt kaydına girer):** Rust tarafı (`spike-async-cancel`,
// NEN-009) `async fn` DEĞİL — düz bir OS thread + `ProgressSink` foreign
// callback trait + bloklayan `JobHandle.join()`. UniFFI'ın Kotlin bindgen'i
// bunu gerçek bir `suspend fun` olarak ÜRETMEZ (yalnız Rust `async fn`
// export'ları `Async.kt` şablonuyla suspend'e dönüşür — bkz.
// uniffi_bindgen kaynağındaki `bindings/kotlin/templates/Async.kt`).
// Üretilen `join()` düz, bloklayan bir metod olarak kalır.
//
// Bu sarmalayıcı üretilen API'nin üstüne ince bir `suspend fun` katmanı
// koyuyor: `join()` ayrı bir thread'de çalışırken, gerçek bir
// `kotlinx.coroutines` iptali (`Job.cancel()`) `invokeOnCancellation` ile
// gerçek Rust `JobHandle.cancel()`'ı tetikliyor — yani Swift'in `Task`
// iptalinin ölçtüğü şeyin birebir Kotlin karşılığı, ama üretilen bindingdeki
// değil, harness'taki bir sarmalayıcı üzerinden.

import kotlinx.coroutines.suspendCancellableCoroutine
import uniffi.spike_async_cancel.JobHandle
import uniffi.spike_async_cancel.JobOutcome
import uniffi.spike_async_cancel.JobParams
import uniffi.spike_async_cancel.ProgressSink
import uniffi.spike_async_cancel.startJob
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

/**
 * `join()`'i bloklamadan bekler; bu coroutine iptal edilirse gerçek Rust
 * `cancel()`'ı çağırır (delivery gate'i kapatır), sonra worker'ın gerçekten
 * bitmesini arka planda bekler.
 */
suspend fun JobHandle.joinCancellable(): JobOutcome = suspendCancellableCoroutine { cont ->
    cont.invokeOnCancellation { this.cancel() }

    val thread =
        Thread {
            try {
                val outcome = this.join()
                if (cont.isActive) cont.resume(outcome)
            } catch (t: Throwable) {
                if (cont.isActive) cont.resumeWithException(t)
            }
        }
    thread.isDaemon = true
    thread.name = "spike-async-cancel-join"
    thread.start()
}

/** `start_job` + `joinCancellable`'ın birleşimi — ölçüm ve testlerin kullandığı üst seviye API. */
suspend fun runJobCancellable(params: JobParams, sink: ProgressSink): JobOutcome {
    val handle = startJob(params, sink)
    return handle.joinCancellable()
}
