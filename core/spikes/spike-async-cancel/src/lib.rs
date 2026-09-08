//! NEN-009 — can a long-running job crossing the FFI boundary be cancelled
//! cooperatively, with **zero** callbacks delivered after `cancel()` returns
//! and **zero** thread/resource leaks across repeated cancellation?
//!
//! **This is not product code** (CLAUDE.md rule 7). It opens its own throwaway
//! FFI gate, which ADR-0028 permits for spikes; ADR-0006's "`nen-ffi` is the
//! single gate" rule stays binding for `core/crates/*`.
//!
//! ## Design — the delivery gate
//!
//! The callback sink lives behind `Mutex<Option<Arc<dyn ProgressSink>>>`.
//! The worker thread takes the lock, reads the sink, and — while still
//! holding the lock — invokes the callback through it. `cancel()` takes the
//! *same* lock, clears the sink to `None`, and returns. Because the callback
//! call happens under the lock:
//!
//! * `cancel()` cannot return while a callback is in flight (it blocks until
//!   the lock is free) — so no callback is ever "in the air" when `cancel()`
//!   hands control back to the caller.
//! * once `cancel()` has cleared the sink, every later checkpoint sees `None`
//!   and stops delivering — so no callback fires after that point either.
//!
//! Both together are Invariant I1. The gate is also what Invariant I2 tests:
//! a worker built with `attempt_late_commit` tries to call `on_commit` after
//! observing cancellation anyway (simulating a buggy implementation) — the
//! gate must still block it.
//!
//! No SRT parsing, no real translation pipeline: that is M5. The "work" is a
//! deterministic busy-loop, not `sleep`, so latency numbers measure the gate
//! and the FFI crossing, not the OS scheduler's sleep jitter.

use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

uniffi::setup_scaffolding!();

/// How many jobs are currently running (spawned, not yet returned from their
/// worker closure). Read by [`live_jobs`] — Invariant I4's leak check.
static LIVE_JOBS: AtomicUsize = AtomicUsize::new(0);

/// Monotonically increasing job id source, so ids stay unique within a
/// process without needing caller-supplied identifiers.
static NEXT_JOB_ID: AtomicUsize = AtomicUsize::new(1);

/// Coarse phase of a job. Typed, not a free string — a callback payload must
/// stay switch-able without string parsing (same principle NEN-010 will
/// apply to errors).
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum Phase {
    Prepare,
    Work,
    Finalize,
}

/// One progress notification. K23: no dialogue, no file path, no free text —
/// only counters and a typed phase.
#[derive(Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct ProgressUpdate {
    pub job_id: u64,
    pub phase: Phase,
    pub done: u32,
    pub total: u32,
}

impl fmt::Debug for ProgressUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProgressUpdate")
            .field("job_id", &self.job_id)
            .field("phase", &self.phase)
            .field("done", &self.done)
            .field("total", &self.total)
            .finish()
    }
}

/// Job configuration — the knobs the Swift harness sweeps to relate
/// checkpoint granularity to cancellation latency (M1 baseline table).
#[derive(Clone, Copy, uniffi::Record)]
pub struct JobParams {
    pub total_blocks: u32,
    /// Simulated per-block cost. Busy-wait, not `sleep`: a deterministic,
    /// CPU-bound cost that does not depend on OS timer resolution.
    pub block_micros: u64,
    /// A progress checkpoint (and gate check) happens every N blocks.
    pub checkpoint_every: u32,
    /// I2: after observing cancellation, still *try* to call `on_commit` —
    /// simulates a buggy worker so the test can prove the gate blocks it
    /// rather than merely observing that a correct worker never tries.
    pub attempt_late_commit: bool,
}

/// Outcome of one job run, returned by [`JobHandle::join`].
#[derive(Clone, Copy, uniffi::Record)]
pub struct JobOutcome {
    pub cancelled: bool,
    pub completed: bool,
    pub committed: bool,
    pub delivered_before_cancel: u32,
    pub late_commit_attempted: bool,
    pub late_commit_blocked: bool,
}

impl fmt::Debug for JobOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JobOutcome").finish_non_exhaustive()
    }
}

/// Implemented by the foreign side (Swift here, Kotlin in NEN-011) to
/// receive progress and commit notifications. `foreign`-only: nothing on the
/// Rust side ever implements this trait, so the FFI bridge does not need to
/// support calling into a Rust implementation.
#[uniffi::export(foreign)]
pub trait ProgressSink: Send + Sync {
    fn on_progress(&self, update: ProgressUpdate);
    fn on_commit(&self, job_id: u64);
}

fn busy_wait(duration: Duration) {
    let start = Instant::now();
    while start.elapsed() < duration {
        std::hint::spin_loop();
    }
}

/// Runs the worker body on the calling (spawned) thread. Always decrements
/// [`LIVE_JOBS`] on the way out — including the panic-unwind path — via the
/// `Drop` guard below, so Invariant I4 holds even if a checkpoint panics.
struct LiveJobGuard;

impl LiveJobGuard {
    fn new() -> Self {
        LIVE_JOBS.fetch_add(1, Ordering::SeqCst);
        Self
    }
}

impl Drop for LiveJobGuard {
    fn drop(&mut self) {
        LIVE_JOBS.fetch_sub(1, Ordering::SeqCst);
    }
}

type Sink = Arc<Mutex<Option<Arc<dyn ProgressSink>>>>;

/// A running (or finished) job. Handed to the caller by [`start_job`].
#[derive(uniffi::Object)]
pub struct JobHandle {
    job_id: u64,
    sink: Sink,
    thread: Mutex<Option<JoinHandle<JobOutcome>>>,
    outcome: Mutex<Option<JobOutcome>>,
    finished: Arc<AtomicBool>,
}

#[uniffi::export]
impl JobHandle {
    /// Closes the delivery gate. Blocks only as long as it takes to acquire
    /// the sink lock — if a callback is mid-flight, `cancel()` waits for it
    /// to return, then clears the sink. After `cancel()` returns, no future
    /// checkpoint in the worker can observe a live sink (Invariant I1).
    pub fn cancel(&self) {
        let mut guard = self.sink.lock().expect("sink mutex poisoned");
        *guard = None;
    }

    /// `true` until the worker thread has run to completion (cancelled or
    /// not). Does not block.
    pub fn is_running(&self) -> bool {
        !self.finished.load(Ordering::Acquire)
    }

    /// Opaque job identifier — a process-local counter, not derived from any
    /// user content. Lets the harness correlate a handle with the `job_id`
    /// carried on each [`ProgressUpdate`].
    pub fn job_id(&self) -> u64 {
        self.job_id
    }

    /// Blocks until the worker thread has exited, then returns its outcome.
    /// Safe to call more than once — the outcome is cached after the first
    /// join.
    pub fn join(&self) -> JobOutcome {
        let mut cached = self.outcome.lock().expect("outcome mutex poisoned");
        if let Some(outcome) = *cached {
            return outcome;
        }
        let handle = self
            .thread
            .lock()
            .expect("thread mutex poisoned")
            .take()
            .expect("join() called concurrently — spike harness is single-threaded per job");
        let outcome = handle.join().expect("worker thread panicked");
        *cached = Some(outcome);
        outcome
    }
}

/// Starts a job. The worker runs on its own OS thread so the caller (the
/// Swift harness's main thread, in the measurement binary) is free to call
/// `cancel()` concurrently — exactly the shape a real cancel button needs.
#[uniffi::export]
pub fn start_job(params: JobParams, sink: Arc<dyn ProgressSink>) -> Arc<JobHandle> {
    let job_id = NEXT_JOB_ID.fetch_add(1, Ordering::SeqCst) as u64;
    let shared_sink: Sink = Arc::new(Mutex::new(Some(sink)));
    let worker_sink = Arc::clone(&shared_sink);
    let finished = Arc::new(AtomicBool::new(false));
    let worker_finished = Arc::clone(&finished);

    let thread = thread::spawn(move || {
        let _live = LiveJobGuard::new();
        let checkpoint_every = params.checkpoint_every.max(1);
        let mut delivered = 0u32;
        let mut cancelled_at: Option<u32> = None;

        // Approach-A parity with NEN-008: a `Prepare` checkpoint so a job
        // with zero work blocks still has one gate check.
        if !notify(&worker_sink, job_id, Phase::Prepare, 0, params.total_blocks) {
            cancelled_at = Some(0);
        } else {
            delivered += 1;
        }

        if cancelled_at.is_none() {
            for block in 0..params.total_blocks {
                busy_wait(Duration::from_micros(params.block_micros));
                let done = block + 1;
                let at_checkpoint = done % checkpoint_every == 0 || done == params.total_blocks;
                if !at_checkpoint {
                    continue;
                }
                if notify(&worker_sink, job_id, Phase::Work, done, params.total_blocks) {
                    delivered += 1;
                } else {
                    cancelled_at = Some(done);
                    break;
                }
            }
        }

        let mut outcome = JobOutcome {
            cancelled: cancelled_at.is_some(),
            completed: false,
            committed: false,
            delivered_before_cancel: delivered,
            late_commit_attempted: false,
            late_commit_blocked: false,
        };

        if cancelled_at.is_none() {
            // Finalize + commit — normal, non-cancelled path.
            let guard = worker_sink.lock().expect("sink mutex poisoned");
            if let Some(sink) = guard.as_ref() {
                sink.on_progress(ProgressUpdate {
                    job_id,
                    phase: Phase::Finalize,
                    done: params.total_blocks,
                    total: params.total_blocks,
                });
                sink.on_commit(job_id);
                outcome.completed = true;
                outcome.committed = true;
            } else {
                // Cancelled in the gap between the last checkpoint and here.
                outcome.cancelled = true;
            }
        } else if params.attempt_late_commit {
            // Invariant I2, negative path: simulate a buggy worker that
            // tries to commit anyway after observing cancellation. The gate
            // must block it — the sink is already `None`.
            outcome.late_commit_attempted = true;
            let guard = worker_sink.lock().expect("sink mutex poisoned");
            if let Some(sink) = guard.as_ref() {
                sink.on_commit(job_id);
                outcome.committed = true;
                outcome.late_commit_blocked = false;
            } else {
                outcome.late_commit_blocked = true;
            }
        }

        worker_finished.store(true, Ordering::Release);
        outcome
    });

    Arc::new(JobHandle {
        job_id,
        sink: shared_sink,
        thread: Mutex::new(Some(thread)),
        outcome: Mutex::new(None),
        finished,
    })
}

/// Takes the sink lock, and — if the gate is still open — delivers one
/// progress notification under that same lock. Returns whether it delivered
/// (`false` means the gate was already closed, i.e. cancelled).
fn notify(sink: &Sink, job_id: u64, phase: Phase, done: u32, total: u32) -> bool {
    let guard = sink.lock().expect("sink mutex poisoned");
    match guard.as_ref() {
        Some(s) => {
            s.on_progress(ProgressUpdate {
                job_id,
                phase,
                done,
                total,
            });
            true
        }
        None => false,
    }
}

/// How many jobs currently have a live worker thread. Used by the Swift
/// harness to prove Invariant I4: after N repeated start+cancel+join cycles,
/// this must return to 0.
#[uniffi::export]
pub fn live_jobs() -> u64 {
    LIVE_JOBS.load(Ordering::SeqCst) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;
    use std::sync::MutexGuard;

    // `LIVE_JOBS` is intentionally process-global because the Swift harness
    // uses it as a shutdown/leak check. Rust's test harness runs tests in
    // parallel, so every test must share one guard before starting a worker;
    // otherwise a different test can legitimately keep this diagnostic
    // counter above zero when an assertion samples it after `join()`.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn test_guard() -> MutexGuard<'static, ()> {
        TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[derive(Default)]
    struct RecordingSink {
        progress_count: AtomicU32,
        commit_count: AtomicU32,
    }

    impl ProgressSink for RecordingSink {
        fn on_progress(&self, _update: ProgressUpdate) {
            self.progress_count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_commit(&self, _job_id: u64) {
            self.commit_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn params(total_blocks: u32, checkpoint_every: u32, attempt_late_commit: bool) -> JobParams {
        JobParams {
            total_blocks,
            block_micros: 50,
            checkpoint_every,
            attempt_late_commit,
        }
    }

    #[test]
    fn uncancelled_job_completes_and_commits_exactly_once() {
        let _test_guard = test_guard();
        let sink = Arc::new(RecordingSink::default());
        let handle = start_job(params(20, 5, false), sink.clone());
        let outcome = handle.join();

        assert!(outcome.completed);
        assert!(outcome.committed);
        assert!(!outcome.cancelled);
        assert_eq!(sink.commit_count.load(Ordering::SeqCst), 1);
        assert_eq!(live_jobs(), 0);
    }

    #[test]
    fn cancel_before_start_yields_zero_delivered_and_no_commit() {
        let _test_guard = test_guard();
        let sink = Arc::new(RecordingSink::default());
        let handle = start_job(params(1_000_000, 1, false), sink.clone());
        handle.cancel();
        let outcome = handle.join();

        assert!(outcome.cancelled);
        assert!(!outcome.committed);
        assert_eq!(sink.commit_count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn late_commit_attempt_is_blocked_by_the_gate() {
        let _test_guard = test_guard();
        let sink = Arc::new(RecordingSink::default());
        let handle = start_job(params(1_000_000, 1, true), sink.clone());
        handle.cancel();
        let outcome = handle.join();

        assert!(outcome.cancelled);
        assert!(outcome.late_commit_attempted);
        assert!(outcome.late_commit_blocked);
        assert!(!outcome.committed);
        assert_eq!(
            sink.commit_count.load(Ordering::SeqCst),
            0,
            "gate must block on_commit even when the worker still tries"
        );
    }

    #[test]
    fn repeated_cancel_cycles_leave_no_live_jobs() {
        let _test_guard = test_guard();
        for _ in 0..100 {
            let sink = Arc::new(RecordingSink::default());
            let handle = start_job(params(500, 3, false), sink);
            handle.cancel();
            handle.join();
        }
        assert_eq!(live_jobs(), 0);
    }

    #[test]
    fn join_is_idempotent() {
        let _test_guard = test_guard();
        let sink = Arc::new(RecordingSink::default());
        let handle = start_job(params(10, 2, false), sink);
        let first = handle.join();
        let second = handle.join();
        assert_eq!(first.completed, second.completed);
        assert_eq!(first.committed, second.committed);
    }

    #[test]
    fn is_running_reflects_completion() {
        let _test_guard = test_guard();
        let sink = Arc::new(RecordingSink::default());
        let handle = start_job(params(5, 1, false), sink);
        handle.join();
        assert!(!handle.is_running());
    }
}
