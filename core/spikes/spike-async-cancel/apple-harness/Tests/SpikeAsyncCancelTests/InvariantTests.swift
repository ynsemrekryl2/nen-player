import Foundation
import Testing

import SpikeCore

// NEN-009 — Invariant'lar I1, I2, I4 (M1-core-spike.md). XCTest DEĞİL
// swift-testing: bu makinede tam Xcode yok (bkz. CoreBridgeTests.swift).
//
// Ölçüm (checkpoint aralığı × cancellation latency baseline) burada değil,
// SpikeAsyncCancel çalıştırılabilirinde — bu dosya yalnız pass/fail
// invariant'ları kanıtlıyor.

/// `ProgressSink` implementasyonu — bir kilit altında sayaç tutuyor.
/// `cancelRequested` `true` olduktan sonra gelen HER çağrı `afterCancelCount`'a
/// sayılır; bu, gerçek "cancel() döndükten sonra callback var mı" sorusunu
/// cevaplıyor (yalnız tek bir örnek değil).
final class RecordingSink: ProgressSink, @unchecked Sendable {
    private let lock = NSLock()
    private var progressCount = 0
    private var commitCount = 0
    private var afterCancelCount = 0
    private var cancelRequested = false

    func onProgress(update: ProgressUpdate) {
        lock.lock()
        progressCount += 1
        if cancelRequested { afterCancelCount += 1 }
        lock.unlock()
    }

    func onCommit(jobId: UInt64) {
        lock.lock()
        commitCount += 1
        if cancelRequested { afterCancelCount += 1 }
        lock.unlock()
    }

    func markCancelRequested() {
        lock.lock(); cancelRequested = true; lock.unlock()
    }

    var counts: (progress: Int, commit: Int, afterCancel: Int) {
        lock.lock(); defer { lock.unlock() }
        return (progressCount, commitCount, afterCancelCount)
    }
}

/// `onProgress` çağrıldığında bir kere sinyal veren yardımcı — "işin gerçekten
/// başladığını" beklemek için `sleep` yerine kullanılıyor.
final class SignalOnFirstProgress: ProgressSink, @unchecked Sendable {
    let semaphore = DispatchSemaphore(value: 0)
    private let lock = NSLock()
    private var signaled = false

    func onProgress(update: ProgressUpdate) {
        lock.lock()
        let shouldSignal = !signaled
        signaled = true
        lock.unlock()
        if shouldSignal { semaphore.signal() }
    }

    func onCommit(jobId: UInt64) {}
}

@Suite("NEN-009 invariants")
struct AsyncCancelInvariantTests {

    /// **I1** — cancel() döndükten sonra hiçbir callback gelmez.
    ///
    /// Beş saniyelik (varsayılan parametrelerle) bir işi, ilk gerçek ilerleme
    /// bildirimini aldıktan hemen sonra iptal ediyoruz — yani iş henüz
    /// sürerken. Kapı tasarımı gereği `cancel()` geri döndüğünde artık
    /// gönderilecek hiçbir callback olmamalı.
    ///
    /// `markCancelRequested()` kasıtlı olarak `cancel()` **döndükten sonra**
    /// çağrılıyor, önce değil: invariant'ın kesin sınırı budur ("cancel()
    /// çağrıldı" değil, "cancel() döndü" — bkz. `lib.rs` modül dokümanı).
    /// Sınırı öne çekmek (cancel() çağrılmadan önce işaretlemek) testi
    /// gereksiz kırılgan yapardı: `SpikeAsyncCancel` ölçüm harness'ı bunu
    /// deneyip `--debug` build'inde 800 koşudan 1'inde sıfırdan farklı bulmuştu
    /// — kapıda değil, "kullanıcı iptale karar verdi" ile "Swift'in `cancel()`'ı
    /// gerçekten çağırması" arasındaki dispatch gecikmesinde. Bu ayrı, daha
    /// gevşek bir ölçüm; `main.swift`'in bağlam raporunda ayrıca görünür.
    @Test("cancel() returns → zero further callbacks, ever")
    func noCallbackAfterCancelReturns() {
        let sink = RecordingSink()
        let starter = SignalOnFirstProgress()
        // Aynı job'a iki sink veremeyiz (start_job tek sink alıyor); bu yüzden
        // "ilk ilerleme geldi" sinyalini ayrı, ufak bir job ile alıp asıl işi
        // ardından başlatmak yerine — burada tek sink kullanıp onProgress
        // içinde hem sayıyoruz hem de ilk çağrıda semaphore'u tetikliyoruz.
        let combined = CombinedSink(recorder: sink, starter: starter)

        let params = JobParams(
            totalBlocks: 5_000_000, blockMicros: 1, checkpointEvery: 50,
            attemptLateCommit: false)
        let handle = startJob(params: params, sink: combined)

        starter.semaphore.wait()
        handle.cancel()
        sink.markCancelRequested()
        let outcome = handle.join()

        #expect(outcome.cancelled)
        let counts = sink.counts
        #expect(counts.afterCancel == 0, "callback delivered after cancel() returned")
    }

    /// **I2** (negatif) — iptal sonrası commit denemesi engelleniyor.
    ///
    /// `attemptLateCommit: true`, iptal edilmiş bir worker'ın yine de
    /// `onCommit` çağırmayı DENEDİĞİ senaryoyu simüle ediyor (kasıtlı hatalı
    /// implementasyon). Kapı bunu engellemeli: `onCommit` hiç çağrılmamalı.
    @Test("cancelled job's late commit attempt is blocked")
    func lateCommitAttemptIsBlocked() {
        let sink = RecordingSink()
        let params = JobParams(
            totalBlocks: 10_000_000, blockMicros: 1, checkpointEvery: 1,
            attemptLateCommit: true)
        let handle = startJob(params: params, sink: sink)

        // Hiç beklemeden iptal — worker'ın ilk checkpoint'ten önce ya da
        // sonra yakalanması fark etmez, ikisi de aynı kapıdan geçer.
        sink.markCancelRequested()
        handle.cancel()
        let outcome = handle.join()

        #expect(outcome.cancelled)
        #expect(outcome.lateCommitAttempted)
        #expect(outcome.lateCommitBlocked)
        #expect(!outcome.committed)
        #expect(sink.counts.commit == 0, "onCommit must never fire for a blocked late commit")
    }

    /// **I4** — tekrarlı iptal (100 kez) sonrası kaynak sızıntısı yok.
    ///
    /// Rust tarafındaki `live_jobs()` her `join()` sonrası 0'a dönmeli;
    /// aksi hâlde worker thread'i beklenenden uzun yaşıyor demektir.
    @Test("100 start+cancel+join cycles leave zero live jobs")
    func repeatedCancelLeavesNoLiveJobs() {
        for _ in 0..<100 {
            let sink = RecordingSink()
            let params = JobParams(
                totalBlocks: 2_000, blockMicros: 5, checkpointEvery: 4,
                attemptLateCommit: false)
            let handle = startJob(params: params, sink: sink)
            sink.markCancelRequested()
            handle.cancel()
            _ = handle.join()
        }

        #expect(liveJobs() == 0)
    }
}

/// I1 testinin tek-sink kısıtına yardımcı: hem sayıyor hem "ilk ilerleme
/// geldi" sinyalini veriyor.
final class CombinedSink: ProgressSink, @unchecked Sendable {
    let recorder: RecordingSink
    let starter: SignalOnFirstProgress

    init(recorder: RecordingSink, starter: SignalOnFirstProgress) {
        self.recorder = recorder
        self.starter = starter
    }

    func onProgress(update: ProgressUpdate) {
        recorder.onProgress(update: update)
        starter.onProgress(update: update)
    }

    func onCommit(jobId: UInt64) {
        recorder.onCommit(jobId: jobId)
        starter.onCommit(jobId: jobId)
    }
}
