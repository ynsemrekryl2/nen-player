//! NEN-029 — playback/renderer session ownership: **A** (core-owned, Rust
//! reverse-calls into a platform observer) vs. **B** (shell-owned, the
//! platform pushes plain forward calls into core). Feeds ADR-0026.
//!
//! **This is not product code** (CLAUDE.md rule 7). It opens its own throwaway
//! FFI gate, which ADR-0028 permits for spikes; ADR-0006's "`nen-ffi` is the
//! single gate" rule stays binding for `core/crates/*`. Only **fake** engines
//! live here — no libmpv/AVPlayer/Media3 (task's own "YAPILMAYACAK").
//!
//! ## Direction A — [`ReverseEngine`]
//!
//! A fake engine runs on its own OS thread and calls a foreign
//! [`PlaybackObserver`] repeatedly — position ticks at a configurable Hz,
//! state transitions, seek completion, typed errors. This generalizes
//! NEN-009's delivery-gate pattern (`Mutex<Option<Arc<dyn Trait>>>`, callback
//! invoked *while holding the lock*, `cancel()` takes the same lock to clear
//! it) from "one job, a handful of callbacks" to "one session, a continuous
//! stream" — the gate is what gives Invariant I1 (no callback after
//! `cancel()` returns) for free, exactly as in NEN-009.
//!
//! ## Direction B — [`ForwardSession`]
//!
//! B is **not** a reverse-callback mechanism at all — that asymmetry is the
//! spike's central finding. In the real B world the shell owns the engine
//! and its clock; core is a passive receiver. So B is modeled as plain
//! forward methods with no observer registration and no reverse thread-hop:
//! the *Swift* harness drives its own timer and calls in, timing on its own
//! side. A pays a per-event thread-hop tax; B structurally cannot, because
//! Swift is always the caller.
//!
//! ## Timing discipline
//!
//! Every call is timed on whichever side **initiates** it: Rust times its
//! own `Instant::now()` around each `observer.on_*()` call for A, Swift
//! times its own clock around each `report_*()` call for B. Rust and Swift
//! timestamps are never correlated against each other — that would require
//! the two clocks to be a provably shared domain, which this spike does not
//! need to solve.

use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

uniffi::setup_scaffolding!();

/// How many [`ReverseEngine`]s currently have a live tick thread. Read by
/// [`live_engines`] — Invariant I4's leak check for Direction A.
static LIVE_ENGINES: AtomicUsize = AtomicUsize::new(0);

/// How many [`ForwardSession`]s have not yet been shut down. Read by
/// [`live_forward_sessions`] — Direction B's analogous leak check (no
/// thread involved; this tracks object lifetime, not thread liveness).
static LIVE_FORWARD_SESSIONS: AtomicUsize = AtomicUsize::new(0);

/// Cap on retained call-cost samples / arrival-seq log per instance — this
/// is a spike diagnostic, not a re-run of NEN-008's "no full lists over FFI"
/// finding, but there is no reason to let a long sweep grow unboundedly.
const SAMPLE_CAP: usize = 20_000;

/// Coarse playback state. Typed, not a free string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum PlaybackState {
    Idle,
    Buffering,
    Playing,
    Paused,
    Ended,
    Failed,
}

/// One position notification. K23 is not in play here — these are synthetic
/// counters, not derived from any real media path or user content — so a
/// plain derive is enough (contrast [`EngineError`] below, which exists
/// specifically to prove exhaustive, string-parse-free typed-error transfer,
/// same discipline NEN-010 established).
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Record)]
pub struct PositionTick {
    pub session_id: u64,
    pub seq: u32,
    pub position_ms: u64,
}

/// One track-selection snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Record)]
pub struct TrackSnapshot {
    pub session_id: u64,
    pub seq: u32,
    pub audio_track: i32,
    pub subtitle_track: i32,
}

/// A closed set of fake engine failure reasons — never a free string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum FailureCode {
    DecoderInitFailed,
    UnsupportedCodec,
    IoStalled,
}

/// Rich `uniffi::Error`: both variants cross to Swift as typed data. No
/// field here is K23-sensitive (fake session ids, fake ms offsets, a closed
/// failure-code enum) — hand-written `Debug`/`Display` is for the same
/// "exhaustive switch, never a message string" discipline NEN-010 proved,
/// not for redaction.
#[derive(uniffi::Error)]
pub enum EngineError {
    /// A [`ReverseEngine::seek`] target past the simulated media length.
    SeekOutOfRange { requested_ms: u64, duration_ms: u64 },
    /// A generic fake-engine failure, closed-set reason.
    EngineFailure { code: FailureCode },
}

impl fmt::Debug for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::SeekOutOfRange {
                requested_ms,
                duration_ms,
            } => f
                .debug_struct("SeekOutOfRange")
                .field("requested_ms", requested_ms)
                .field("duration_ms", duration_ms)
                .finish(),
            EngineError::EngineFailure { code } => {
                f.debug_struct("EngineFailure").field("code", code).finish()
            }
        }
    }
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for EngineError {}

/// Implemented by the foreign side (Swift here). `foreign`-only: nothing on
/// the Rust side ever implements this — this trait *is* the reverse
/// callback being measured. One global `seq`, stamped immediately before
/// each call while the delivery gate is held, lets the harness assert
/// strict arrival ordering across *all* callback kinds, not just within one.
#[uniffi::export(foreign)]
pub trait PlaybackObserver: Send + Sync {
    fn on_position(&self, tick: PositionTick);
    fn on_state_changed(&self, session_id: u64, seq: u32, state: PlaybackState);
    fn on_track_snapshot(&self, snapshot: TrackSnapshot);
    fn on_seek_completed(&self, session_id: u64, seq: u32, target_ms: u64);
    fn on_error(&self, error: EngineError);
}

/// Knobs the Swift harness sweeps: `hz` relates directly to the DoD's 4 Hz /
/// 60 Hz position-update comparison; `media_duration_ms` gives `seek` a
/// simulated bound to exceed (exercising [`EngineError::SeekOutOfRange`]).
#[derive(Clone, Copy, uniffi::Record)]
pub struct TickParams {
    pub session_id: u64,
    pub hz: f64,
    pub run_duration_ms: u64,
    pub media_duration_ms: u64,
    pub seek_latency_ms: u64,
}

type ObserverSink = Arc<Mutex<Option<Arc<dyn PlaybackObserver>>>>;

/// Runs the delivery gate under the sink lock and reports whether the
/// action was actually delivered (`false` = gate already closed, i.e.
/// cancelled/shut down). Cost is measured only around the callback
/// invocation itself, on the Rust (caller) side — Direction A's timing
/// discipline, see module doc.
fn deliver<F>(sink: &ObserverSink, cost_samples: &Mutex<Vec<u64>>, action: F) -> bool
where
    F: FnOnce(&dyn PlaybackObserver),
{
    let guard = sink.lock().expect("sink mutex poisoned");
    let observer = match guard.as_deref() {
        Some(observer) => observer,
        None => return false,
    };
    let start = Instant::now();
    action(observer);
    let elapsed_ns = start.elapsed().as_nanos() as u64;
    drop(guard);

    let mut samples = cost_samples.lock().expect("cost-sample mutex poisoned");
    if samples.len() < SAMPLE_CAP {
        samples.push(elapsed_ns);
    }
    true
}

/// Runs the worker body on its own OS thread. Always decrements
/// [`LIVE_ENGINES`] on the way out — including the panic-unwind path — via
/// `Drop`, so Invariant I4 holds even if a callback panics.
struct LiveEngineGuard;

impl LiveEngineGuard {
    fn new() -> Self {
        LIVE_ENGINES.fetch_add(1, Ordering::SeqCst);
        Self
    }
}

impl Drop for LiveEngineGuard {
    fn drop(&mut self) {
        LIVE_ENGINES.fetch_sub(1, Ordering::SeqCst);
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_tick_thread(
    session_id: u64,
    hz: f64,
    run_duration_ms: u64,
    sink: ObserverSink,
    stop: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    seq: Arc<AtomicU32>,
    cost_samples: Arc<Mutex<Vec<u64>>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let _live = LiveEngineGuard::new();
        running.store(true, Ordering::Release);

        let interval = Duration::from_secs_f64(1.0 / hz.max(0.001));
        let step_ms = ((interval.as_secs_f64() * 1000.0).round() as u64).max(1);
        let end = Instant::now() + Duration::from_millis(run_duration_ms);
        let mut position_ms: u64 = 0;

        deliver(&sink, &cost_samples, |o| {
            o.on_state_changed(
                session_id,
                seq.fetch_add(1, Ordering::SeqCst),
                PlaybackState::Buffering,
            );
        });
        deliver(&sink, &cost_samples, |o| {
            o.on_state_changed(
                session_id,
                seq.fetch_add(1, Ordering::SeqCst),
                PlaybackState::Playing,
            );
        });

        while Instant::now() < end && !stop.load(Ordering::Acquire) {
            thread::sleep(interval);
            position_ms += step_ms;
            let delivered = deliver(&sink, &cost_samples, |o| {
                o.on_position(PositionTick {
                    session_id,
                    seq: seq.fetch_add(1, Ordering::SeqCst),
                    position_ms,
                });
            });
            if !delivered {
                // Gate closed (cancel()) — stop producing; a real engine
                // would keep decoding, but nothing is left to notify.
                break;
            }
        }

        deliver(&sink, &cost_samples, |o| {
            o.on_state_changed(
                session_id,
                seq.fetch_add(1, Ordering::SeqCst),
                PlaybackState::Ended,
            );
        });

        running.store(false, Ordering::Release);
    })
}

/// Direction A: a fake, core-owned playback session that reverse-calls a
/// [`PlaybackObserver`]. `cancel()` and `shutdown()` are deliberately
/// distinct: `cancel()` only closes the delivery gate (Invariant I1's
/// concern — same mechanism as NEN-009's `JobHandle::cancel()`); `shutdown()`
/// additionally stops and joins every thread this instance owns (Invariant
/// I4's concern). A cancelled-but-not-shut-down engine is a valid, testable
/// intermediate state — it mirrors "cancel() returned but the OS thread is
/// still winding down," which is exactly the gap I1 exists to make safe.
#[derive(uniffi::Object)]
pub struct ReverseEngine {
    session_id: u64,
    sink: ObserverSink,
    stop: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    seq: Arc<AtomicU32>,
    media_duration_ms: u64,
    seek_latency_ms: u64,
    cost_samples: Arc<Mutex<Vec<u64>>>,
    tick_thread: Mutex<Option<JoinHandle<()>>>,
    // Detached per-seek threads (each simulates async completion latency).
    // shutdown() drains and joins every one of these — so repeated
    // start/seek/shutdown cycles are leak-free by construction, not by
    // accident of a short sleep outliving the test.
    seek_threads: Mutex<Vec<JoinHandle<()>>>,
}

#[uniffi::export]
impl ReverseEngine {
    #[uniffi::constructor]
    pub fn start(params: TickParams, observer: Arc<dyn PlaybackObserver>) -> Arc<Self> {
        let sink: ObserverSink = Arc::new(Mutex::new(Some(observer)));
        let stop = Arc::new(AtomicBool::new(false));
        let running = Arc::new(AtomicBool::new(false));
        let seq = Arc::new(AtomicU32::new(0));
        let cost_samples = Arc::new(Mutex::new(Vec::new()));

        let tick_thread = spawn_tick_thread(
            params.session_id,
            params.hz,
            params.run_duration_ms,
            Arc::clone(&sink),
            Arc::clone(&stop),
            Arc::clone(&running),
            Arc::clone(&seq),
            Arc::clone(&cost_samples),
        );

        Arc::new(Self {
            session_id: params.session_id,
            sink,
            stop,
            running,
            seq,
            media_duration_ms: params.media_duration_ms,
            seek_latency_ms: params.seek_latency_ms,
            cost_samples,
            tick_thread: Mutex::new(Some(tick_thread)),
            seek_threads: Mutex::new(Vec::new()),
        })
    }

    /// Simulates an async seek: after `seek_latency_ms`, delivers either
    /// `on_seek_completed` or (if `target_ms` exceeds the simulated media
    /// length) `on_error(SeekOutOfRange)`. Runs on its own short-lived
    /// thread so the caller is never blocked — the real shape a UI seek bar
    /// needs.
    pub fn seek(&self, target_ms: u64) {
        let sink = Arc::clone(&self.sink);
        let cost_samples = Arc::clone(&self.cost_samples);
        let seq = Arc::clone(&self.seq);
        let session_id = self.session_id;
        let media_duration_ms = self.media_duration_ms;
        let latency = Duration::from_millis(self.seek_latency_ms);

        let handle = thread::spawn(move || {
            thread::sleep(latency);
            if target_ms > media_duration_ms {
                deliver(&sink, &cost_samples, |o| {
                    o.on_error(EngineError::SeekOutOfRange {
                        requested_ms: target_ms,
                        duration_ms: media_duration_ms,
                    });
                });
            } else {
                deliver(&sink, &cost_samples, |o| {
                    o.on_seek_completed(session_id, seq.fetch_add(1, Ordering::SeqCst), target_ms);
                });
            }
        });

        let mut threads = self
            .seek_threads
            .lock()
            .expect("seek-threads mutex poisoned");
        threads.retain(|h| !h.is_finished());
        threads.push(handle);
    }

    /// Delivers a track-snapshot notification immediately (synchronous —
    /// unlike `seek`, there's no async step to simulate).
    pub fn publish_track_snapshot(&self, audio_track: i32, subtitle_track: i32) {
        let session_id = self.session_id;
        deliver(&self.sink, &self.cost_samples, |o| {
            o.on_track_snapshot(TrackSnapshot {
                session_id,
                seq: self.seq.fetch_add(1, Ordering::SeqCst),
                audio_track,
                subtitle_track,
            });
        });
    }

    /// Delivers a typed engine failure immediately — Direction A's half of
    /// the typed-error-transfer DoD item (Direction B's half is
    /// [`ForwardSession::report_failure`]).
    pub fn trigger_error(&self, code: FailureCode) {
        deliver(&self.sink, &self.cost_samples, |o| {
            o.on_error(EngineError::EngineFailure { code });
        });
    }

    /// Closes the delivery gate. Blocks only as long as it takes to acquire
    /// the sink lock — if a callback is mid-flight, `cancel()` waits for it
    /// to return, then clears the sink. Same mechanism as NEN-009's
    /// `JobHandle::cancel()`; this is what makes Invariant I1 structural
    /// rather than a race.
    pub fn cancel(&self) {
        let mut guard = self.sink.lock().expect("sink mutex poisoned");
        *guard = None;
    }

    /// Stops the tick thread and joins every thread this instance owns
    /// (tick thread + any in-flight seek threads), also closing the gate as
    /// a courtesy if `cancel()` was never called. Idempotent-safe to call
    /// once; a second call is a no-op (both `Option::take()`s are already
    /// empty).
    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::Release);
        {
            let mut guard = self.sink.lock().expect("sink mutex poisoned");
            *guard = None;
        }
        if let Some(handle) = self
            .tick_thread
            .lock()
            .expect("tick-thread mutex poisoned")
            .take()
        {
            let _ = handle.join();
        }
        let seek_threads: Vec<JoinHandle<()>> = std::mem::take(
            &mut *self
                .seek_threads
                .lock()
                .expect("seek-threads mutex poisoned"),
        );
        for handle in seek_threads {
            let _ = handle.join();
        }
    }

    /// `true` until the tick thread has run to completion (naturally,
    /// cancelled, or shut down). Does not block.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    /// Per-callback delivery cost in nanoseconds, measured on the Rust
    /// (caller) side — the raw data behind the 4 Hz / 60 Hz baseline
    /// tables. Bounded by [`SAMPLE_CAP`].
    pub fn call_cost_samples_ns(&self) -> Vec<u64> {
        self.cost_samples
            .lock()
            .expect("cost-sample mutex poisoned")
            .clone()
    }
}

/// How many [`ReverseEngine`]s currently have a live tick thread. Used by
/// the Swift harness to prove Invariant I4 for Direction A: after N
/// repeated start+shutdown cycles, this must return to 0.
#[uniffi::export]
pub fn live_engines() -> u64 {
    LIVE_ENGINES.load(Ordering::SeqCst) as u64
}

/// Direction B: a fake shell-owned session. No thread, no observer
/// registration — the platform shell (represented by the Swift harness's
/// own timer) calls plain forward methods, timed on the Swift side. There
/// is no reverse thread-hop to measure here; that asymmetry versus
/// Direction A is itself the headline comparative finding (see ADR-0026
/// draft).
#[derive(uniffi::Object)]
pub struct ForwardSession {
    #[allow(dead_code)]
    session_id: u64,
    arrival_seqs: Mutex<Vec<u32>>,
    shut_down: AtomicBool,
}

#[uniffi::export]
impl ForwardSession {
    #[uniffi::constructor]
    pub fn new(session_id: u64) -> Arc<Self> {
        LIVE_FORWARD_SESSIONS.fetch_add(1, Ordering::SeqCst);
        Arc::new(Self {
            session_id,
            arrival_seqs: Mutex::new(Vec::new()),
            shut_down: AtomicBool::new(false),
        })
    }

    fn record(&self, seq: u32) -> Result<(), EngineError> {
        if self.shut_down.load(Ordering::Acquire) {
            // A forward call arriving after shutdown fails gracefully with a
            // typed error rather than panicking or silently no-op'ing — the
            // B-side analogue of "does late delivery misbehave," though this
            // is not Invariant I1 (B has no reverse callback to be "late").
            return Err(EngineError::EngineFailure {
                code: FailureCode::IoStalled,
            });
        }
        let mut seqs = self
            .arrival_seqs
            .lock()
            .expect("arrival-seq mutex poisoned");
        if seqs.len() < SAMPLE_CAP {
            seqs.push(seq);
        }
        Ok(())
    }

    pub fn report_position(&self, seq: u32, _position_ms: u64) -> Result<(), EngineError> {
        self.record(seq)
    }

    pub fn report_state(&self, seq: u32, _state: PlaybackState) -> Result<(), EngineError> {
        self.record(seq)
    }

    pub fn report_track_snapshot(&self, snapshot: TrackSnapshot) -> Result<(), EngineError> {
        self.record(snapshot.seq)
    }

    pub fn report_seek_started(&self, seq: u32, _target_ms: u64) -> Result<(), EngineError> {
        self.record(seq)
    }

    pub fn report_seek_completed(&self, seq: u32, _target_ms: u64) -> Result<(), EngineError> {
        self.record(seq)
    }

    /// Direction B's half of the typed-error-transfer DoD item — a plain
    /// `Result` return, no reverse callback involved.
    pub fn report_failure(&self, code: FailureCode) -> Result<(), EngineError> {
        Err(EngineError::EngineFailure { code })
    }

    /// Recorded arrival order, for the Swift harness's ordering assertions.
    pub fn arrival_seqs(&self) -> Vec<u32> {
        self.arrival_seqs
            .lock()
            .expect("arrival-seq mutex poisoned")
            .clone()
    }

    pub fn is_shut_down(&self) -> bool {
        self.shut_down.load(Ordering::Acquire)
    }

    /// Idempotent — a second call is a no-op (`swap` only decrements once).
    pub fn shutdown(&self) {
        if !self.shut_down.swap(true, Ordering::AcqRel) {
            LIVE_FORWARD_SESSIONS.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

/// How many [`ForwardSession`]s have not yet been shut down — Direction B's
/// leak check. No thread is involved on this side; this is an object-
/// lifetime counter, not a thread-liveness one.
#[uniffi::export]
pub fn live_forward_sessions() -> u64 {
    LIVE_FORWARD_SESSIONS.load(Ordering::SeqCst) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cross-test isolation for the global `LIVE_ENGINES` /
    /// `LIVE_FORWARD_SESSIONS` counters. Unlike NEN-009's near-instant
    /// CPU-bound busy loops, this spike's ticks run on real wall-clock
    /// sleeps (Hz-based), so parallel `cargo test` threads reliably overlap
    /// and can make a shared counter read >0 because a *different*,
    /// still-running test's engine is live — not because this test leaked.
    /// Every test that creates a `ReverseEngine` or `ForwardSession` takes
    /// this lock first, serializing them within this crate's test binary.
    static TEST_SERIAL: Mutex<()> = Mutex::new(());

    #[derive(Default)]
    struct RecordingObserver {
        positions: Mutex<Vec<PositionTick>>,
        states: Mutex<Vec<PlaybackState>>,
        seek_completions: Mutex<Vec<u64>>,
        errors: Mutex<Vec<String>>,
        seqs: Mutex<Vec<u32>>,
    }

    impl RecordingObserver {
        fn position_count(&self) -> usize {
            self.positions.lock().unwrap().len()
        }
    }

    impl PlaybackObserver for RecordingObserver {
        fn on_position(&self, tick: PositionTick) {
            self.seqs.lock().unwrap().push(tick.seq);
            self.positions.lock().unwrap().push(tick);
        }
        fn on_state_changed(&self, _session_id: u64, seq: u32, state: PlaybackState) {
            self.seqs.lock().unwrap().push(seq);
            self.states.lock().unwrap().push(state);
        }
        fn on_track_snapshot(&self, snapshot: TrackSnapshot) {
            self.seqs.lock().unwrap().push(snapshot.seq);
        }
        fn on_seek_completed(&self, _session_id: u64, seq: u32, target_ms: u64) {
            self.seqs.lock().unwrap().push(seq);
            self.seek_completions.lock().unwrap().push(target_ms);
        }
        fn on_error(&self, error: EngineError) {
            self.errors.lock().unwrap().push(format!("{error:?}"));
        }
    }

    fn params(hz: f64, run_duration_ms: u64) -> TickParams {
        TickParams {
            session_id: 1,
            hz,
            run_duration_ms,
            media_duration_ms: 60_000,
            seek_latency_ms: 5,
        }
    }

    #[test]
    fn engine_runs_and_shutdown_stops_it_cleanly() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let observer = Arc::new(RecordingObserver::default());
        let engine = ReverseEngine::start(params(200.0, 5_000), observer.clone());
        thread::sleep(Duration::from_millis(50));
        engine.shutdown();

        assert!(!engine.is_running());
        assert!(
            observer.position_count() > 0,
            "expected at least one tick before shutdown"
        );
        assert_eq!(live_engines(), 0);
    }

    #[test]
    fn cancel_closes_the_gate_before_shutdown() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let observer = Arc::new(RecordingObserver::default());
        let engine = ReverseEngine::start(params(500.0, 5_000), observer.clone());
        thread::sleep(Duration::from_millis(30));
        engine.cancel();
        let after_cancel = observer.position_count();
        thread::sleep(Duration::from_millis(30));
        let settled = observer.position_count();
        engine.shutdown();

        assert_eq!(
            after_cancel, settled,
            "no delivery should land after cancel() returns"
        );
        assert_eq!(live_engines(), 0);
    }

    #[test]
    fn seek_within_range_completes_with_target() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let observer = Arc::new(RecordingObserver::default());
        let engine = ReverseEngine::start(params(50.0, 2_000), observer.clone());
        engine.seek(10_000);
        thread::sleep(Duration::from_millis(60));
        engine.shutdown();

        assert_eq!(
            observer.seek_completions.lock().unwrap().as_slice(),
            &[10_000]
        );
        assert!(observer.errors.lock().unwrap().is_empty());
    }

    #[test]
    fn seek_out_of_range_delivers_typed_error() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let observer = Arc::new(RecordingObserver::default());
        let engine = ReverseEngine::start(params(50.0, 2_000), observer.clone());
        engine.seek(999_999);
        thread::sleep(Duration::from_millis(60));
        engine.shutdown();

        assert!(observer.seek_completions.lock().unwrap().is_empty());
        let errors = observer.errors.lock().unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("SeekOutOfRange"));
    }

    #[test]
    fn trigger_error_delivers_engine_failure() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let observer = Arc::new(RecordingObserver::default());
        let engine = ReverseEngine::start(params(50.0, 2_000), observer.clone());
        engine.trigger_error(FailureCode::UnsupportedCodec);
        engine.shutdown();

        let errors = observer.errors.lock().unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("EngineFailure"));
        assert!(errors[0].contains("UnsupportedCodec"));
    }

    #[test]
    fn seq_is_strictly_increasing_across_callback_kinds() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let observer = Arc::new(RecordingObserver::default());
        let engine = ReverseEngine::start(params(200.0, 1_800), observer.clone());
        engine.publish_track_snapshot(1, 2);
        engine.seek(5_000);
        thread::sleep(Duration::from_millis(80));
        engine.shutdown();

        let seqs = observer.seqs.lock().unwrap().clone();
        assert!(
            seqs.len() > 3,
            "expected multiple callback kinds to have fired"
        );
        for window in seqs.windows(2) {
            assert!(
                window[0] < window[1],
                "seq must be strictly increasing: {seqs:?}"
            );
        }
    }

    #[test]
    fn repeated_start_shutdown_cycles_leave_zero_live_engines() {
        let _serial = TEST_SERIAL.lock().unwrap();
        for _ in 0..50 {
            let observer = Arc::new(RecordingObserver::default());
            let engine = ReverseEngine::start(params(1_000.0, 20), observer);
            engine.seek(1_000);
            thread::sleep(Duration::from_millis(5));
            engine.shutdown();
        }
        assert_eq!(live_engines(), 0);
    }

    #[test]
    fn forward_session_records_arrival_order() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let session = ForwardSession::new(7);
        session.report_state(0, PlaybackState::Buffering).unwrap();
        session.report_position(1, 100).unwrap();
        session.report_seek_started(2, 5_000).unwrap();
        session.report_seek_completed(3, 5_000).unwrap();
        session.shutdown();

        assert_eq!(session.arrival_seqs(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn forward_calls_after_shutdown_return_typed_error_not_panic() {
        let _serial = TEST_SERIAL.lock().unwrap();
        let session = ForwardSession::new(9);
        session.shutdown();
        let result = session.report_position(0, 0);

        assert!(matches!(
            result,
            Err(EngineError::EngineFailure {
                code: FailureCode::IoStalled
            })
        ));
    }

    #[test]
    fn repeated_new_shutdown_cycles_leave_zero_live_sessions() {
        let _serial = TEST_SERIAL.lock().unwrap();
        for _ in 0..50 {
            let session = ForwardSession::new(1);
            session.report_failure(FailureCode::DecoderInitFailed).ok();
            session.shutdown();
            session.shutdown(); // idempotent — must not double-decrement
        }
        assert_eq!(live_forward_sessions(), 0);
    }

    #[test]
    fn engine_error_variants_are_distinguishable_without_string_parsing() {
        fn classify(e: &EngineError) -> &'static str {
            match e {
                EngineError::SeekOutOfRange { .. } => "seek_out_of_range",
                EngineError::EngineFailure { .. } => "engine_failure",
            }
        }
        assert_eq!(
            classify(&EngineError::SeekOutOfRange {
                requested_ms: 1,
                duration_ms: 0
            }),
            "seek_out_of_range"
        );
        assert_eq!(
            classify(&EngineError::EngineFailure {
                code: FailureCode::IoStalled
            }),
            "engine_failure"
        );
    }
}
