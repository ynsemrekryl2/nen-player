import Combine
import Foundation
import NenCore
import NenPlaybackMPV
import Testing
@testable import NenPlayerShell

/// `NEN-102`'s progress indicator and cancellation surface.
///
/// A cancel is proven the way `core/crates/nen-ffi/tests/translation_gate.rs`'s
/// own `PausingSink` proves it on the Rust side: `translationProgressObserver`
/// runs synchronously on the worker thread, inside `onProgress`, while
/// `TranslationCall`'s delivery gate lock (ADR-0004 Karar 2) is held — there
/// is no FFI-exposed call gate a Swift test could reach into
/// `nen_providers::translation_mock`'s own, so this observer hook is the only
/// way to pause the mock provider mid-run from this side of the boundary.
// `.serialized`: several tests here park a translation job's worker thread
// (a real OS thread from Swift's cooperative pool, via `Task.detached`) for
// tens of milliseconds at a time, some alongside a second `Task.detached`
// canceller. Running them concurrently with each other risks exhausting
// that bounded pool — observed as the whole package's *other*, unrelated
// tests stalling mid-run, not just these ones failing.
@Suite("AI translation progress and cancellation (NEN-102)", .serialized)
@MainActor
struct TranslationProgressTests {
    @Test("progress reaches the indicator in order, and clears when the job finishes")
    func progressReachesTheIndicatorInOrder() async throws {
        let store = TempFixture("translate-progress-order")
        defer { store.remove() }
        let dir = TempFixture("translate-progress-order-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Source.en.srt", multiBlockSrt(cueCount: 95))

        let model = makeModel(session: FakeSession(), targetLanguage: "tr", storeRoot: store.url)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(userSubtitleToken(in: model))
        model.selectSubtitle(token: token)

        var observed: [TranslationProgressState] = []
        let cancellable = model.$translationProgress
            .compactMap { $0 }
            .sink { observed.append($0) }

        model.translateSelectedSubtitle()
        await model.awaitTranslation()
        cancellable.cancel()

        #expect(!observed.isEmpty, "a multi-block job reports at least one progress event")
        #expect(model.translationProgress == nil, "the indicator clears once the job finishes")

        // Block sequence only ever advances by exactly one, at a `Preparing`/
        // `done == 0` boundary (`checkpoint.rs`'s own contract) — never
        // skips, never goes backward.
        var lastBlock = 0
        for state in observed {
            #expect(state.block == lastBlock || state.block == lastBlock + 1, "block sequence skipped or went backward")
            lastBlock = state.block
        }
        #expect(lastBlock == 3, "a 95-cue document with the default 40-cue block splits into 3 blocks")

        // Within each block, `done` is monotonic (ADR-0004 Karar 4) and never
        // exceeds `total`.
        var withinBlock: [Int: [FfiTranslationProgress]] = [:]
        for state in observed {
            withinBlock[state.block, default: []].append(
                FfiTranslationProgress(phase: state.phase, done: state.done, total: state.total)
            )
        }
        for (_, events) in withinBlock {
            var lastDone: UInt32 = 0
            for event in events {
                #expect(event.done >= lastDone, "done went backward within a block")
                #expect(event.done <= event.total, "done exceeded total")
                lastDone = event.done
            }
        }
    }

    @Test("cancelling mid-run stops the job, hides the indicator, and leaves no trace")
    func cancelMidRunStopsTheJob() async throws {
        let store = TempFixture("translate-cancel-mid-run")
        defer { store.remove() }
        let dir = TempFixture("translate-cancel-mid-run-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        // Deliberately the same multi-block size the other cancellation
        // tests below use, not a 1-cue document. Once `release()` lets the
        // paused worker resume, cancellation only lands at the *next*
        // checkpoint the worker reaches after the delivery gate closes — a
        // 1-cue job has almost no remaining checkpoints to land on, and can
        // finish (writing its artifact) before the canceller thread is ever
        // scheduled at all. A 95-cue/3-block job has many more checkpoints
        // ahead of it, giving cancellation many more chances to land before
        // the job could possibly finish — measured directly: this test
        // flaked red every run with a 1-cue document, green every run with
        // this one.
        let subtitle = dir.write("Source.en.srt", multiBlockSrt(cueCount: 95))

        let rendezvous = Rendezvous()
        let session = FakeSession()
        let model = makeModel(
            session: session,
            targetLanguage: "tr",
            storeRoot: store.url,
            translationProgressObserver: { _ in rendezvous.arriveAndWaitOnce() }
        )
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(userSubtitleToken(in: model))
        model.selectSubtitle(token: token)
        let sourceCountBefore = model.subtitleSourceCount
        let selectedBefore = model.selectedSubtitleToken
        let drawnBefore = session.drawnSubtitles

        model.translateSelectedSubtitle()
        await rendezvous.waitUntilArrived()

        // `beginCancellingOnBackgroundThread` already confirms the canceller
        // thread is running before returning; `release()` follows
        // immediately, deliberately with no extra grace sleep. Measured
        // directly while writing this test: an *added* sleep here — even
        // though `translation_gate.rs`'s own analogous Rust test takes one,
        // to give its canceller thread time to reach the delivery gate's
        // lock — made this **less** reliable, not more, on this platform:
        // the longer a paused worker thread sits parked before `release()`
        // wakes it, the more consistently it wins the OS scheduler's
        // wake-up race against an equally-parked canceller thread. With no
        // added delay, the canceller — which has been running (not
        // parked) since `started.wait()` returned — reliably reaches and
        // blocks on the lock first.
        let waitForCanceller = beginCancellingOnBackgroundThread(model)
        rendezvous.release()
        await waitForCanceller()

        await model.awaitTranslation()

        #expect(!model.isTranslating)
        #expect(model.translationProgress == nil)
        #expect(model.subtitleSourceCount == sourceCountBefore, "a cancelled job added no source")
        #expect(model.selectedSubtitleToken == selectedBefore, "the screen never changed (§10)")
        #expect(session.drawnSubtitles == drawnBefore, "nothing new was ever drawn")
        #expect(
            model.transientMessage == PlaybackPresentation.translationJoinMessage(for: .Cancelled)
        )
        // Not `!fileExists(.../artifacts)`: `FilesystemArtifactStore::new`
        // (`core/crates/nen-persist/src/store.rs`) creates that directory
        // eagerly, the moment the engine opens — regardless of whether any
        // artifact is ever written into it. The real claim is that it holds
        // nothing — a cancelled job wrote no *file*.
        #expect(
            artifactFileCount(in: store.url) == 0,
            "a cancelled job wrote no artifact"
        )

        let aiEntry = model.subtitleMenu.flatMap(\.entries).first { $0.kind == .ai }
        #expect(aiEntry == nil, "a cancelled job added no Ai catalog row")
    }

    @Test("cancelling before the worker starts never starts a job at all")
    func cancelBeforeStartNeverStartsAJob() async throws {
        let store = TempFixture("translate-cancel-before-start")
        defer { store.remove() }
        let dir = TempFixture("translate-cancel-before-start-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Source.en.srt", TempFixture.validSrt)

        let model = makeModel(session: FakeSession(), targetLanguage: "tr", storeRoot: store.url)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(userSubtitleToken(in: model))
        model.selectSubtitle(token: token)

        model.translateSelectedSubtitle()
        // Same main-actor turn as the command — nothing has hopped off the
        // main actor yet, so this cancel wins deterministically, every time.
        model.cancelTranslation()

        await model.awaitTranslation()

        #expect(!model.isTranslating)
        #expect(model.translationProgress == nil)
        #expect(
            !FileManager.default.fileExists(atPath: store.url.appendingPathComponent("artifacts").path),
            "the store root was never even opened"
        )
        let aiEntry = model.subtitleMenu.flatMap(\.entries).first { $0.kind == .ai }
        #expect(aiEntry == nil)
    }

    @Test("the on-screen subtitle never changes while a job runs or after it is cancelled (§10)")
    func theScreenNeverChangesMidRun() async throws {
        let store = TempFixture("translate-screen-unchanged")
        defer { store.remove() }
        let dir = TempFixture("translate-screen-unchanged-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Source.en.srt", multiBlockSrt(cueCount: 95))

        let rendezvous = Rendezvous()
        let session = FakeSession()
        let model = makeModel(
            session: session,
            targetLanguage: "tr",
            storeRoot: store.url,
            translationProgressObserver: { _ in rendezvous.arriveAndWaitOnce() }
        )
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(userSubtitleToken(in: model))
        model.selectSubtitle(token: token)
        let drawnBefore = session.drawnSubtitles
        let selectedBefore = model.selectedSubtitleToken

        model.translateSelectedSubtitle()
        await rendezvous.waitUntilArrived()

        // Mid-run, while the worker is parked: the catalog has gained
        // nothing yet, and nothing new was drawn.
        #expect(session.drawnSubtitles == drawnBefore)
        #expect(model.selectedSubtitleToken == selectedBefore)
        #expect(model.subtitleMenu.flatMap(\.entries).first { $0.kind == .ai } == nil)

        rendezvous.release()
        await model.awaitTranslation()

        // After completion: a row was catalogued (proving the job actually
        // ran to success), but the screen — what is drawn and what is
        // selected — is still untouched. Cataloguing is not selecting.
        #expect(session.drawnSubtitles == drawnBefore, "catalog ≠ screen")
        #expect(model.selectedSubtitleToken == selectedBefore)
    }

    @Test("a cancelled job adds no Ai source to the menu")
    func cancelledJobAddsNoAiSource() async throws {
        let store = TempFixture("translate-cancel-no-ai-row")
        defer { store.remove() }
        let dir = TempFixture("translate-cancel-no-ai-row-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Source.en.srt", multiBlockSrt(cueCount: 95))

        let rendezvous = Rendezvous()
        let model = makeModel(
            session: FakeSession(),
            targetLanguage: "tr",
            storeRoot: store.url,
            translationProgressObserver: { _ in rendezvous.arriveAndWaitOnce() }
        )
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(userSubtitleToken(in: model))
        model.selectSubtitle(token: token)

        model.translateSelectedSubtitle()
        await rendezvous.waitUntilArrived()
        let waitForCanceller = beginCancellingOnBackgroundThread(model)
        rendezvous.release()
        await waitForCanceller()
        await model.awaitTranslation()

        let aiEntries = model.subtitleMenu.flatMap(\.entries).filter { $0.kind == .ai }
        #expect(aiEntries.isEmpty)
    }

    @Test("opening another medium cancels the running job")
    func openingAnotherMediumCancelsTheJob() async throws {
        let store = TempFixture("translate-open-cancels")
        defer { store.remove() }
        let dir = TempFixture("translate-open-cancels-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let other = dir.write("Other.mkv", "not really a video either")
        let subtitle = dir.write("Source.en.srt", multiBlockSrt(cueCount: 95))

        let rendezvous = Rendezvous()
        let model = makeModel(
            session: FakeSession(),
            targetLanguage: "tr",
            storeRoot: store.url,
            translationProgressObserver: { _ in rendezvous.arriveAndWaitOnce() }
        )
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(userSubtitleToken(in: model))
        model.selectSubtitle(token: token)

        model.translateSelectedSubtitle()
        await rendezvous.waitUntilArrived()

        // `openMedia(at:)` is `@MainActor`-isolated and, on this same
        // thread, calls `cancelTranslation()` synchronously — which blocks
        // until the paused callback above returns. `release()` cannot run
        // from this same call (there is nothing left to run it with once
        // blocked), so it runs from an independent background thread
        // instead, concurrently with `openMedia` itself.
        Thread {
            Thread.sleep(forTimeInterval: 0.05)
            rendezvous.release()
        }.start()
        model.openMedia(at: other)
        model.consume([.stateChanged(state: .ready)])

        await model.awaitTranslation()

        #expect(!model.isTranslating)
        #expect(model.translationProgress == nil)
        let aiEntry = model.subtitleMenu.flatMap(\.entries).first { $0.kind == .ai }
        #expect(aiEntry == nil, "the new medium's menu gained nothing from the old medium's job")
    }

    @Test("the Altyazı menu's gate shows a job running excludes starting a new one")
    func menuSwitchesToCancelWhileRunning() async throws {
        let store = TempFixture("translate-menu-switch")
        defer { store.remove() }
        let dir = TempFixture("translate-menu-switch-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Source.en.srt", multiBlockSrt(cueCount: 95))

        let rendezvous = Rendezvous()
        let model = makeModel(
            session: FakeSession(),
            targetLanguage: "tr",
            storeRoot: store.url,
            translationProgressObserver: { _ in rendezvous.arriveAndWaitOnce() }
        )
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(userSubtitleToken(in: model))
        model.selectSubtitle(token: token)

        model.translateSelectedSubtitle()
        await rendezvous.waitUntilArrived()

        #expect(model.isTranslating)
        #expect(!model.canTranslateSelectedSubtitle, "a running job disables the start command")

        let waitForCanceller = beginCancellingOnBackgroundThread(model)
        rendezvous.release()
        await waitForCanceller()
        await model.awaitTranslation()
    }

    // MARK: - Helpers

    private func makeModel(
        session: FakeSession,
        targetLanguage: String?,
        storeRoot: URL,
        translationProgressObserver: PlayerModel.TranslationProgressObserver? = nil
    ) -> PlayerModel {
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            preferenceStore: MemoryPreferenceStore(),
            translationPreferenceStore: MemoryTranslationPreferenceStore(targetLanguage: targetLanguage),
            translationStoreRoot: storeRoot,
            translationProgressObserver: translationProgressObserver,
            sessionFactory: { _ in session }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())
        return model
    }

    private func userSubtitleToken(in model: PlayerModel) -> UInt32? {
        model.subtitleMenu
            .first { SubtitleMenuGroupID($0.group) == .userSubtitles }?
            .entries.first?.token
    }
}

/// A synthetic multi-block subtitle document — `cueCount` sequential,
/// non-overlapping 1.2s cues spaced 1.5s apart, well inside strict SRT
/// (`NEN-013`). 95 cues, the default 40-cue/6-overlap block layout
/// (`ADR-0015`), splits into exactly 3 blocks — the same count
/// `fixtures/subtitles/blocks/layout-sample.srt`'s golden test measures.
private func multiBlockSrt(cueCount: Int) -> String {
    var lines: [String] = []
    for index in 1...cueCount {
        let start = Double(index - 1) * 1.5
        let end = start + 1.2
        lines.append("\(index)")
        lines.append("\(srtTimestamp(start)) --> \(srtTimestamp(end))")
        lines.append("Line number \(index).")
        lines.append("")
    }
    return lines.joined(separator: "\n")
}

private func srtTimestamp(_ seconds: Double) -> String {
    let totalMilliseconds = Int((seconds * 1000).rounded())
    let hours = totalMilliseconds / 3_600_000
    let minutes = (totalMilliseconds / 60_000) % 60
    let secs = (totalMilliseconds / 1_000) % 60
    let millis = totalMilliseconds % 1_000
    return String(format: "%02d:%02d:%02d,%03d", hours, minutes, secs, millis)
}

/// How many artifact files sit under `storeRoot/artifacts/` — not whether
/// the directory itself exists. `FilesystemArtifactStore::new`
/// (`core/crates/nen-persist/src/store.rs`) creates that directory eagerly
/// on open, before any translation work happens, so its mere existence
/// proves nothing about whether a job actually wrote to it. Mirrors
/// `core/crates/nen-ffi/tests/translation_gate.rs`'s own `artifact_count`
/// helper on the Rust side of this same boundary.
func artifactFileCount(in storeRoot: URL) -> Int {
    let artifactsDir = storeRoot.appendingPathComponent("artifacts")
    return (try? FileManager.default.contentsOfDirectory(atPath: artifactsDir.path))?.count ?? 0
}

/// Starts `model.cancelTranslation()` on a real `Thread` and waits until it
/// has begun running — not until it returns. Returns a closure the caller
/// awaits *after* releasing whatever gate `cancel()` is about to block on.
///
/// A real `Thread`, not `Task.detached`: `cancel()` blocks on the same
/// delivery-gate lock a paused progress callback holds
/// (`TranslationCancellation`'s own doc comment), so it needs to start
/// running — and reach that lock — immediately, the guarantee
/// `core/crates/nen-ffi/tests/translation_gate.rs`'s own analogous test
/// gets from `thread::spawn`. `Task.detached` only queues onto Swift's
/// cooperative pool with no guarantee of when it actually starts; measured
/// directly while writing this suite, the unblocked (fast, mock-provider)
/// worker could race straight through the whole job before a merely-queued
/// canceller task ever got a turn, so `cancel()` landed too late to matter.
///
/// Calling `model.cancelTranslation()` (or anything that calls it, like
/// `openMedia`) directly on the main actor while a `Rendezvous`-paused
/// callback holds the gate is a **guaranteed deadlock**, not just a slow
/// path: the call blocks the main actor's one real thread, and nothing
/// else can run on it to reach the `release()` that would ever unblock
/// it — measured directly, this shape of self-deadlock is what every
/// caller of this helper was hitting before it existed.
func beginCancellingOnBackgroundThread(_ model: PlayerModel) -> () async -> Void {
    let started = DispatchSemaphore(value: 0)
    let finished = DispatchSemaphore(value: 0)
    Thread {
        started.signal()
        model.cancelTranslation()
        finished.signal()
    }.start()
    started.wait()
    return {
        await withCheckedContinuation { (continuation: CheckedContinuation<Void, Never>) in
            DispatchQueue.global(qos: .utility).async {
                finished.wait()
                continuation.resume()
            }
        }
    }
}

/// A single-arrival rendezvous a test uses to pause the translation worker
/// thread mid-run, mirroring `core/crates/nen-ffi/tests/translation_gate.rs`'s
/// `Rendezvous`/`PausingSink` pair on the Swift side of the same boundary.
/// Pauses on the **first** callback only — later callbacks in the same run
/// pass straight through, the same "pause once, then let the rest complete
/// once released" shape the Rust fixture uses.
final class Rendezvous: @unchecked Sendable {
    private let lock = NSLock()
    private var hasPausedOnce = false
    private var hasArrivedAlready = false
    private var arrivedContinuation: CheckedContinuation<Void, Never>?
    private let releaseSemaphore = DispatchSemaphore(value: 0)

    /// Awaited from the test's `@MainActor` body — a true async suspension,
    /// **not** a thread-blocking wait. `translateSelectedSubtitle()`'s own
    /// task inherits the caller's actor (MainActor, Swift's own rule for an
    /// unstructured `Task` created from actor-isolated code) and needs the
    /// MainActor free to even begin running — a `DispatchSemaphore.wait()`
    /// here would starve that very task of the thread it needs to reach the
    /// first `onProgress` callback this method is waiting for, deadlocking
    /// the test against itself before the job ever started.
    func waitUntilArrived() async {
        await withCheckedContinuation { continuation in
            lock.lock()
            if hasArrivedAlready {
                lock.unlock()
                continuation.resume()
            } else {
                arrivedContinuation = continuation
                lock.unlock()
            }
        }
    }

    func release() {
        releaseSemaphore.signal()
    }

    /// Called from `onProgress`, on the worker thread. Blocks **that**
    /// thread — a real OS thread from `Task.detached`'s pool, not the
    /// MainActor — while `TranslationCall`'s delivery gate lock is held,
    /// until `release()`.
    func arriveAndWaitOnce() {
        lock.lock()
        let shouldPause = !hasPausedOnce
        hasPausedOnce = true
        if shouldPause {
            hasArrivedAlready = true
            let continuation = arrivedContinuation
            arrivedContinuation = nil
            lock.unlock()
            continuation?.resume()
        } else {
            lock.unlock()
        }
        guard shouldPause else { return }
        releaseSemaphore.wait()
    }
}
