//! The playback session the platform shell drives (ADR-0033).
//!
//! [`ADR-0026`] put the session in the core and [`ADR-0012`] Karar 2 put the
//! macOS engine in Swift, so the shell reaches playback through *this* object
//! and never through the engine it handed over. What that buys is written in
//! ADR-0026: subtitle sync and translation triggering are decided once, here,
//! instead of once per platform.
//!
//! [`ADR-0012`]: ../../../../docs/adr/0012-macos-playback-engine.md
//! [`ADR-0026`]: ../../../../docs/adr/0026-playback-renderer-ownership.md
//!
//! # Why the session owns a pump (ADR-0033 Karar 2)
//!
//! [`EventQueue`](nen_ports::playback::EventQueue) is bounded and turns an
//! overflow into `EventsLost`, which is what lets a consumer that fell behind
//! learn that it did (ADR-0011 Karar 1). But the queue only bounds what has
//! already been pulled out of the engine: a foreign adapter buffers on **its**
//! side until someone calls `drain_events`, and that buffer has no limit.
//!
//! So with nobody pulling, the 64-slot limit bounds nothing — the backlog is
//! simply somewhere else, and a backgrounded app comes back to a thousand real
//! events instead of one honest `EventsLost`. The pump is what closes that:
//! it pulls whether or not the shell is looking, so accumulation always happens
//! in the queue, where the rules are.
//!
//! # Why delivery is pull (ADR-0033 Karar 3)
//!
//! The shell calls [`PlaybackSession::drain_events`] when it is ready. Pushing
//! instead — a foreign `EventSink` — would move the same unbounded backlog into
//! the platform's dispatch queue, which has neither a limit nor coalescing.
//! A suspended consumer accumulates nothing here.
//!
//! Because nothing is ever called *back*, ADR-0011 Karar 2's forbidden state
//! cannot arise on this path. The guard still runs on every command: the ban
//! belongs to the port, and `nen-ports`' own sink path is still there for a
//! future consumer.

use crate::playback::{ShellEngine, ShellEngineBridge};
use nen_ports::playback::{
    MediaSource, Operation, PlaybackEngine, PlaybackError, PlaybackEvent, PlaybackState,
    TrackDescriptor, TrackId, TrackKind,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

/// How often the pump pulls from the engine.
///
/// Roughly 30 Hz. Pulling faster only refreshes a value that is already
/// coalescing — `PositionChanged` never occupies more than one slot — while
/// pulling slower delays critical events, which are the ones the UI reacts to.
pub const PUMP_INTERVAL: Duration = Duration::from_millis(33);

/// Takes a lock without caring whether a previous holder panicked.
///
/// A foreign engine that panics mid-call would poison the mutex, and refusing
/// to look at the queue afterwards helps nobody: the events already in it are
/// still true, and the session's own state is a queue plus a flag.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What the pump thread waits on, so stopping it does not take an interval.
#[derive(Default)]
struct PumpSignal {
    stop: Mutex<bool>,
    wake: Condvar,
}

struct Pump {
    signal: Arc<PumpSignal>,
    handle: JoinHandle<()>,
}

/// One medium, one engine, one event queue.
///
/// Every command forwards to the port through [`ShellEngineBridge`], which is
/// where the queue, the coalescing rules and the reentrancy guard live. This
/// type adds exactly two things the bridge cannot have: something that pulls
/// on its own, and a lifetime.
pub struct PlaybackSession {
    engine: Arc<Mutex<ShellEngineBridge>>,
    pump: Mutex<Option<Pump>>,
    shut_down: AtomicBool,
}

impl PlaybackSession {
    /// The session a shell builds: pumping at [`PUMP_INTERVAL`].
    pub fn new(engine: Arc<dyn ShellEngine>) -> Self {
        Self::with_pump_interval(engine, PUMP_INTERVAL)
    }

    /// The same session at a chosen interval.
    ///
    /// Exists so a test can watch the pump do its work without waiting on the
    /// production cadence. The interval is **not** a platform knob — ADR-0033
    /// Karar 2 keeps it in the core precisely so a platform cannot quietly
    /// choose a slower one.
    pub fn with_pump_interval(engine: Arc<dyn ShellEngine>, interval: Duration) -> Self {
        let session = Self::idle(engine);
        *lock(&session.pump) = Some(spawn_pump(Arc::clone(&session.engine), interval));
        session
    }

    /// A session that pulls only when told to, via [`Self::pump_once`].
    ///
    /// **Not a supported way to run a player.** It exists so the negative
    /// control can measure what the pump is worth: the same flood, with and
    /// without something pulling in the background.
    pub fn without_pump(engine: Arc<dyn ShellEngine>) -> Self {
        Self::idle(engine)
    }

    fn idle(engine: Arc<dyn ShellEngine>) -> Self {
        Self {
            engine: Arc::new(Mutex::new(ShellEngineBridge::new(engine))),
            pump: Mutex::new(None),
            shut_down: AtomicBool::new(false),
        }
    }

    pub fn load(&self, locator: String) -> Result<(), PlaybackError> {
        self.command(Operation::Load, |engine| {
            engine.load(&MediaSource::new(locator))
        })
    }

    pub fn play(&self) -> Result<(), PlaybackError> {
        self.command(Operation::Play, PlaybackEngine::play)
    }

    pub fn pause(&self) -> Result<(), PlaybackError> {
        self.command(Operation::Pause, PlaybackEngine::pause)
    }

    pub fn stop(&self) -> Result<(), PlaybackError> {
        self.command(Operation::Stop, PlaybackEngine::stop)
    }

    pub fn seek(&self, to: Duration) -> Result<(), PlaybackError> {
        self.command(Operation::Seek, |engine| engine.seek(to))
    }

    pub fn position(&self) -> Result<Duration, PlaybackError> {
        self.command(Operation::Position, |engine| engine.position())
    }

    pub fn duration(&self) -> Result<Option<Duration>, PlaybackError> {
        self.command(Operation::Duration, |engine| engine.duration())
    }

    /// Where playback is.
    ///
    /// The port answers this infallibly; the session cannot, because a shut
    /// down session has no engine to ask and inventing `Idle` would be a
    /// confident wrong answer to a question the shell asks in order to draw
    /// something.
    pub fn state(&self) -> Result<PlaybackState, PlaybackError> {
        self.command(Operation::State, |engine| Ok(engine.state()))
    }

    pub fn tracks(&self, kind: TrackKind) -> Result<Vec<TrackDescriptor>, PlaybackError> {
        self.command(Operation::Tracks, |engine| engine.tracks(kind))
    }

    pub fn select_track(
        &self,
        kind: TrackKind,
        track: Option<TrackId>,
    ) -> Result<(), PlaybackError> {
        self.command(Operation::SelectTrack, |engine| {
            engine.select_track(kind, track)
        })
    }

    pub fn selected_track(&self, kind: TrackKind) -> Result<Option<TrackId>, PlaybackError> {
        self.command(Operation::SelectTrack, |engine| engine.selected_track(kind))
    }

    pub fn set_rate(&self, rate: f32) -> Result<(), PlaybackError> {
        self.command(Operation::SetRate, |engine| engine.set_rate(rate))
    }

    pub fn set_volume(&self, volume: f32) -> Result<(), PlaybackError> {
        self.command(Operation::SetVolume, |engine| engine.set_volume(volume))
    }

    /// Everything the queue holds, in order, emptying it.
    ///
    /// Still answers after shutdown: the last events a medium produced —
    /// including the state change that ended it — are exactly what the shell
    /// needs to draw its final frame, and refusing to hand them over would
    /// lose facts that already happened.
    pub fn drain_events(&self) -> Vec<PlaybackEvent> {
        lock(&self.engine).events().drain()
    }

    /// Pulls once, on the caller's thread.
    ///
    /// What the pump does every interval, exposed so a test can do it at a
    /// known moment instead of racing a thread.
    pub fn pump_once(&self) {
        lock(&self.engine).events();
    }

    /// Stops the pump, then the engine. Idempotent.
    ///
    /// Order matters and is the whole reason this is not two calls: the pump
    /// touches the engine every interval, so it has to be stopped **and
    /// joined** before the engine is told to go away.
    pub fn shutdown(&self) -> Result<(), PlaybackError> {
        if self.shut_down.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        self.stop_pump();
        lock(&self.engine).shutdown()
    }

    fn command<T>(
        &self,
        operation: Operation,
        body: impl FnOnce(&mut ShellEngineBridge) -> Result<T, PlaybackError>,
    ) -> Result<T, PlaybackError> {
        if self.shut_down.load(Ordering::Acquire) {
            return Err(PlaybackError::ShutDown { operation });
        }
        body(&mut lock(&self.engine))
    }

    fn stop_pump(&self) {
        let Some(pump) = lock(&self.pump).take() else {
            return;
        };
        *lock(&pump.signal.stop) = true;
        pump.signal.wake.notify_all();
        // A pump thread that panicked is already gone; the join tells us so and
        // there is nothing further to do about it here.
        let _ = pump.handle.join();
    }
}

impl Drop for PlaybackSession {
    /// A shell that drops the session without calling `shutdown` must not leave
    /// a thread pulling on an engine nobody owns.
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

fn spawn_pump(engine: Arc<Mutex<ShellEngineBridge>>, interval: Duration) -> Pump {
    let signal = Arc::new(PumpSignal::default());
    let theirs = Arc::clone(&signal);
    let handle = std::thread::Builder::new()
        .name("nen.playback.pump".to_owned())
        .spawn(move || loop {
            // `events()` is what pulls: the bridge drains the foreign engine
            // into the shared queue before handing the queue over. The pump
            // wants the pulling, not the queue.
            lock(&engine).events();

            let mut stop = lock(&theirs.stop);
            if *stop {
                return;
            }
            let (guard, _) = theirs
                .wake
                .wait_timeout(stop, interval)
                .unwrap_or_else(PoisonError::into_inner);
            stop = guard;
            if *stop {
                return;
            }
        })
        .expect("the pump thread can be spawned");
    Pump { signal, handle }
}
