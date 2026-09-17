import Foundation
import NenCore

/// A translation job's document-wide progress, as shown on screen (`NEN-107`).
///
/// The provider still reports block-local progress behind the core delivery
/// gate. `nen-translate` maps each block's offset into the document total and
/// clamps repair/retry attempts to a shared high-water mark before this FFI
/// value reaches the shell.
public struct TranslationProgressState: Equatable, Sendable {
    public let phase: FfiTranslationPhase
    /// Number of translated cues across the whole document.
    public let done: UInt32
    /// Total cues in the whole document.
    public let total: UInt32

    /// Completion fraction for the whole document.
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
/// would then occasionally show a stale count or a `done` that briefly jumps
/// backward and forward.
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
    private var preparationCancellation: (generation: UInt64, handler: () -> Void)?
    private var generation: UInt64 = 0

    /// Starts tracking a new job. Called synchronously on the main actor,
    /// before the detached task that will eventually call `adopt(_:)` is
    /// even spawned.
    func begin() {
        lock.lock()
        job = nil
        cancelRequested = false
        generation &+= 1
        preparationCancellation = nil
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

    /// Registers the cancellation hook for the preparation phase, before a
    /// translation job exists. A registration that races with `cancel()` is
    /// invoked synchronously, so a remote read cannot miss the user's request.
    @discardableResult
    func registerPreparationCancellation(_ handler: @escaping () -> Void) -> UInt64 {
        lock.lock()
        let registrationGeneration = generation
        if cancelRequested {
            lock.unlock()
            handler()
        } else {
            preparationCancellation = (registrationGeneration, handler)
            lock.unlock()
        }
        return registrationGeneration
    }

    /// Removes only the registration belonging to the preparation call that
    /// installed it. This keeps a late completion from clearing a newer job's
    /// cancellation hook if the lifecycle ever permits overlap.
    func clearPreparationCancellation(_ generation: UInt64) {
        lock.lock()
        if preparationCancellation?.generation == generation {
            preparationCancellation = nil
        }
        lock.unlock()
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
        let preparation = preparationCancellation?.handler
        lock.unlock()
        preparation?()
        job?.cancel()
    }
}

/// How a translation job that finished or failed reports back to the main
/// actor. Carries the finished `FfiTranslationJob` rather than its summary
/// so `catalogInto` can be called only after re-checking the medium has not
/// changed underneath it — `FfiTranslationJob`, `FfiTranslationSummary` and
/// every error type here are all `Sendable` (generated bindings), so nothing
/// here needs a manual `@unchecked`.
extension FfiTokenUsage {
    /// No provider call was ever made — every early-cancellation exit above
    /// `FfiTranslationEngine.start` returns this rather than an ambiguous
    /// zero built by hand at each call site.
    static let zero = FfiTokenUsage(
        inputTokens: 0,
        cachedInputTokens: 0,
        outputTokens: 0,
        costUsd: nil
    )

    /// Combines two usages the way `nen_ports::translation::TokenUsage`
    /// does on the Rust side: counts add, and `costUsd` stays `nil` only
    /// when neither side reported one (`NEN-139`'s session-wide total).
    func adding(_ other: FfiTokenUsage) -> FfiTokenUsage {
        let combinedCost: Double? = switch (costUsd, other.costUsd) {
        case let (.some(a), .some(b)): a + b
        case let (.some(a), .none): a
        case let (.none, .some(b)): b
        case (.none, .none): nil
        }
        return FfiTokenUsage(
            inputTokens: inputTokens + other.inputTokens,
            cachedInputTokens: cachedInputTokens + other.cachedInputTokens,
            outputTokens: outputTokens + other.outputTokens,
            costUsd: combinedCost
        )
    }
}

enum TranslationJoinOutcome: Sendable {
    case succeeded(FfiTranslationJob)
    /// `prepare_embedded_document` (`NEN-044`) refused before a job could
    /// even be prepared — the engine was never asked to start.
    case prepareFailed(FfiEmbeddedDocumentError)
    case startFailed(FfiTranslationStartError)
    /// `usage` is whatever `FfiTranslationJob.totalUsage()` read at the
    /// moment of failure (`NEN-138`/`NEN-139`) — zero when no job ever
    /// started (every early-cancellation site above), otherwise the tokens
    /// its provider calls already billed before the failure or the
    /// cancellation, which are not un-billed by either.
    case joinFailed(FfiTranslationError, usage: FfiTokenUsage)
}
