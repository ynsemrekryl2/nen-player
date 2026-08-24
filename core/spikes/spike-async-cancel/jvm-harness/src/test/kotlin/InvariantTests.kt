// NEN-011 — Invariant'lar I1, I2, I4 (M1-core-spike.md), Kotlin/JVM tarafı.
// `apple-harness/Tests/SpikeAsyncCancelTests/InvariantTests.swift`'in
// (NEN-009) birebir karşılığı.
//
// I1 burada — Swift'in `Task` iptalinin aksine — gerçek bir
// `kotlinx.coroutines` `Job.cancel()` ile tetikleniyor: `joinCancellable()`
// sarmalayıcısı `invokeOnCancellation`'da Rust `JobHandle.cancel()`'ı
// çağırıyor (bkz. JobHandleCoroutines.kt modül dokümanı — bu, task metninin
// "coroutine iptali ile JobHandle.cancel() uyumu" maddesinin kanıtı).

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import java.util.concurrent.Semaphore
import java.util.concurrent.locks.ReentrantLock
import kotlin.concurrent.withLock
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import uniffi.spike_async_cancel.JobParams
import uniffi.spike_async_cancel.ProgressSink
import uniffi.spike_async_cancel.ProgressUpdate
import uniffi.spike_async_cancel.liveJobs
import uniffi.spike_async_cancel.startJob

/** `cancelRequested` `true` olduktan sonra gelen HER çağrı `afterCancelCount`'a
 *  sayılır — Swift `RecordingSink` ile aynı desen. */
class RecordingSink : ProgressSink {
    private val lock = ReentrantLock()
    private var progressCount = 0
    private var commitCount = 0
    var afterCancelCount = 0
        private set
    private var cancelRequested = false

    override fun onProgress(update: ProgressUpdate) {
        lock.withLock {
            progressCount += 1
            if (cancelRequested) afterCancelCount += 1
        }
    }

    override fun onCommit(jobId: ULong) {
        lock.withLock {
            commitCount += 1
            if (cancelRequested) afterCancelCount += 1
        }
    }

    fun markCancelRequested() {
        lock.withLock { cancelRequested = true }
    }

    fun commitCountSnapshot(): Int = lock.withLock { commitCount }
}

/** `onProgress` çağrıldığında bir kere sinyal veren yardımcı — Swift
 *  `SignalOnFirstProgress` ile aynı: "işin gerçekten başladığını" beklemek
 *  için `sleep` yerine kullanılıyor. Aynı anda hem sayıp hem sinyal vermek
 *  için `RecordingSink`'i sarmalıyor (aynı job'a iki sink veremeyiz). */
class CombinedSink(private val recorder: RecordingSink) : ProgressSink {
    val firstProgress = Semaphore(0)
    private val lock = ReentrantLock()
    private var signaled = false

    override fun onProgress(update: ProgressUpdate) {
        recorder.onProgress(update)
        lock.withLock {
            val shouldSignal = !signaled
            signaled = true
            if (shouldSignal) firstProgress.release()
        }
    }

    override fun onCommit(jobId: ULong) {
        recorder.onCommit(jobId)
    }
}

class AsyncCancelInvariantTests {

    /**
     * I1 — kotlinx coroutine `cancel()` çağrıldıktan (ve `cancelAndJoin()`
     * tamamlandıktan) sonra hiçbir callback gelmez.
     *
     * `markCancelRequested()` kasıtlı olarak coroutine `cancel()` ÇAĞRILDIKTAN
     * SONRA işaretleniyor — Swift eşdeğerindeki gerekçenin aynısı: invariant'ın
     * kesin sınırı "iptal isteği" değil "delivery gate kapandı" anıdır.
     */
    @Test
    fun noCallbackAfterCoroutineCancelCompletes() = runBlocking {
        val recorder = RecordingSink()
        val combined = CombinedSink(recorder)

        val params =
            JobParams(
                totalBlocks = 5_000_000u, blockMicros = 1u, checkpointEvery = 50u,
                attemptLateCommit = false)

        val scope = CoroutineScope(Dispatchers.Default)
        val handle = startJob(params, combined)
        val job =
            scope.launch {
                handle.joinCancellable()
            }

        combined.firstProgress.acquire()
        job.cancelAndJoin()
        recorder.markCancelRequested()
        // handle.cancel() joinCancellable()'ın invokeOnCancellation'ında zaten
        // çağrıldı; worker'ın gerçekten bitmesini burada da bekleyelim.
        handle.join()

        assertEquals(0, recorder.afterCancelCount, "callback delivered after coroutine cancel completed")
        handle.destroy()
    }

    /**
     * I2 (negatif) — iptal sonrası commit denemesi engelleniyor. Coroutine
     * gerekmiyor: doğrudan `JobHandle.cancel()` ile aynı kapı test ediliyor.
     */
    @Test
    fun lateCommitAttemptIsBlocked() {
        val sink = RecordingSink()
        val params =
            JobParams(
                totalBlocks = 10_000_000u, blockMicros = 1u, checkpointEvery = 1u,
                attemptLateCommit = true)
        val handle = startJob(params, sink)

        sink.markCancelRequested()
        handle.cancel()
        val outcome = handle.join()

        assertTrue(outcome.cancelled)
        assertTrue(outcome.lateCommitAttempted)
        assertTrue(outcome.lateCommitBlocked)
        assertTrue(!outcome.committed)
        assertEquals(0, sink.commitCountSnapshot(), "onCommit must never fire for a blocked late commit")
        handle.destroy()
    }

    /** I4 — tekrarlı iptal (100 kez) sonrası kaynak sızıntısı yok. */
    @Test
    fun repeatedCancelLeavesNoLiveJobs() {
        repeat(100) {
            val sink = RecordingSink()
            val params =
                JobParams(
                    totalBlocks = 2_000u, blockMicros = 5u, checkpointEvery = 4u,
                    attemptLateCommit = false)
            val handle = startJob(params, sink)
            sink.markCancelRequested()
            handle.cancel()
            handle.join()
            handle.destroy()
        }

        assertEquals(0uL, liveJobs())
    }
}
