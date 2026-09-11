import Foundation
import NenCore

/// A translation job's progress, as shown on screen (`NEN-102`).
///
/// Deliberately **not** a document-wide percentage: `FfiTranslationProgress`
/// is block-local (`core/crates/nen-translate/src/checkpoint.rs` forks a
/// fresh `TranslationCall` at every block boundary and resets `done` to `0`
/// there), and the FFI boundary does not carry the document's total cue
/// count — the shell has nothing honest to divide by yet. A document-wide
/// percentage is `NEN-107`'s job, planned to replace this type's `block`/
/// `fraction` pair once the FFI surface carries a document-wide total.
public struct TranslationProgressState: Equatable, Sendable {
    public let phase: FfiTranslationPhase
    /// 1-based — the first block reported is block 1, never block 0.
    public let block: Int
    /// This block's own `done` (ADR-0004 Karar 4's monotonic counter,
    /// reset at each block boundary — not a running total).
    public let done: UInt32
    /// This block's own cue count.
    public let total: UInt32

    /// This block's own completion fraction, not the document's.
    public var fraction: Double {
        total == 0 ? 0 : Double(done) / Double(total)
    }
}

/// Carries progress callbacks from a translation job's worker thread
/// (`nen-ffi`'s `TranslationCall`) to an `AsyncStream` a main-actor consumer
/// can iterate in the order they were reported (`NEN-102`).
///
/// `AsyncStream` preserves yield order; a bare `Task { @MainActor in … }`
/// spawned per callback would not — nothing guarantees the scheduler runs
/// per-`Task` closures in creation order, and `PlayerRootView`'s indicator
/// would then occasionally show a stale block or a `done` that briefly
/// jumps backward and forward.
///
/// **Never calls back into the job this progress belongs to.**
/// `onProgress` runs while `TranslationCall` holds its delivery gate's lock
/// (`core/crates/nen-ffi/src/translation.rs`'s own warning on
/// `ForeignTranslationProgressSink`); this type only yields to a
/// continuation, nothing more.
final class TranslationProgressRelay: ForeignTranslationProgressSink, Sendable {
    private let continuation: AsyncStream<FfiTranslationProgress>.Continuation
    /// Runs synchronously, on the worker thread, before the callback
    /// yields — test-only (`PlayerModel`'s `translationProgressObserver`
    /// injection point). A test observer can block here to pause the
    /// worker mid-run, the same rendezvous shape
    /// `core/crates/nen-ffi/tests/translation_gate.rs`'s `PausingSink`
    /// uses on the Rust side, since there is no FFI-exposed call gate for
    /// a Swift test to reach into `nen_providers::translation_mock`'s own.
    private let observer: (@Sendable (FfiTranslationProgress) -> Void)?

    init(
        continuation: AsyncStream<FfiTranslationProgress>.Continuation,
        observer: (@Sendable (FfiTranslationProgress) -> Void)? = nil
    ) {
        self.continuation = continuation
        self.observer = observer
    }

    func onProgress(progress: FfiTranslationProgress) {
        observer?(progress)
        continuation.yield(progress)
    }
}

/// Tracks a translation job across the boundary between the main actor
/// (where cancellation is requested) and the detached worker task (where
/// the job is created and joined) — `NEN-102`'s cancellation surface.
///
/// Deliberately not an `actor`: `PlayerModel.cancelTranslation()` must be
/// callable synchronously, without an `await`, so a test can cancel a job
/// while the worker thread is parked inside a progress callback without
/// itself needing to hop onto the main actor first.
///
/// One instance is reused for the model's whole lifetime; `begin()` resets
/// it for each new job. `translateSelectedSubtitle()`'s own re-entrancy gate
/// (`isTranslating`) already guarantees only one job is ever tracked at a
/// time, so this type does not need to guard against overlapping jobs.
final class TranslationCancellation: @unchecked Sendable {
    private let lock = NSLock()
    private var job: FfiTranslationJob?
    private var cancelRequested = false

    /// Starts tracking a new job. Called synchronously on the main actor,
    /// before the detached task that will eventually call `adopt(_:)` is
    /// even spawned.
    func begin() {
        lock.lock()
        job = nil
        cancelRequested = false
        lock.unlock()
    }

    /// Whether `cancel()` arrived before a job existed to hand it to.
    ///
    /// The detached task checks this before doing any FFI work at all — a
    /// cancel issued in the same main-actor turn as the command never opens
    /// the artifact store or starts a job; there is nothing racy left to
    /// clean up.
    var isCancelled: Bool {
        lock.lock()
        defer { lock.unlock() }
        return cancelRequested
    }

    /// Hands the running job to this tracker.
    ///
    /// If `cancel()` already arrived — a race between the `isCancelled`
    /// check above and the job actually starting — the job is cancelled
    /// immediately here rather than left to run to completion unwatched.
    func adopt(_ job: FfiTranslationJob) {
        lock.lock()
        self.job = job
        let alreadyCancelled = cancelRequested
        lock.unlock()
        if alreadyCancelled {
            job.cancel()
        }
    }

    /// Requests cancellation.
    ///
    /// Safe to call before a job exists (the request is remembered for
    /// `adopt(_:)`) or after the job already finished
    /// (`FfiTranslationJob.cancel()` is idempotent, its own doc comment).
    /// The lock is released before calling into the FFI job: `cancel()`
    /// blocks until an in-flight progress callback returns (ADR-0004 Karar
    /// 2), and that callback runs on the worker thread, not this one —
    /// holding the lock across that wait would only cost a caller nothing
    /// blocks on today.
    func cancel() {
        lock.lock()
        cancelRequested = true
        let job = self.job
        lock.unlock()
        job?.cancel()
    }
}

/// How a translation job that finished or failed reports back to the main
/// actor. Carries the finished `FfiTranslationJob` rather than its summary
/// so `catalogInto` can be called only after re-checking the medium has not
/// changed underneath it — `FfiTranslationJob`, `FfiTranslationSummary` and
/// every error type here are all `Sendable` (generated bindings), so nothing
/// here needs a manual `@unchecked`.
enum TranslationJoinOutcome: Sendable {
    case succeeded(FfiTranslationJob)
    /// `prepare_embedded_document` (`NEN-044`) refused before a job could
    /// even be prepared — the engine was never asked to start.
    case prepareFailed(FfiEmbeddedDocumentError)
    case startFailed(FfiTranslationStartError)
    case joinFailed(FfiTranslationError)
}
