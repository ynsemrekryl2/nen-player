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
use crate::renderer::{playback_error, EngineNativeRenderer, RendererState};
use crate::subtitles::SubtitleLibrary;
use nen_ports::playback::{
    MediaSource, Operation, PlaybackEngine, PlaybackError, PlaybackEvent, PlaybackState,
    TrackDescriptor, TrackId, TrackKind,
};
use nen_ports::renderer::{RenderError, SubtitleRenderer};
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

/// What showing a menu row did.
///
/// Not a `Result`: a row that cannot be shown is not an error the shell has to
/// handle as one, on exactly the reasoning
/// [`AddOutcome`](crate::subtitles::AddOutcome) is built on. A stale token, or
/// one naming a source marked broken, is an **outcome** with a defined
/// behaviour — do nothing — while the engine refusing is a genuine failure the
/// user is told about. Collapsing the two would put a shrunken menu on the
/// same path as a dead engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowOutcome {
    /// On screen now.
    Shown,
    /// Nothing happened: no such row, or the row is marked unusable.
    Unusable,
}

/// One medium, one engine, one event queue.
///
/// Every command forwards to the port through [`ShellEngineBridge`], which is
/// where the queue, the coalescing rules and the reentrancy guard live. This
/// type adds exactly two things the bridge cannot have: something that pulls
/// on its own, and a lifetime.
pub struct PlaybackSession {
    engine: Arc<Mutex<ShellEngineBridge>>,
    /// What the renderer remembers between calls (ADR-0013 Karar 2).
    ///
    /// Beside the engine rather than inside it: the engine draws, but only
    /// this side knows *which document* it was given, and that is what makes
    /// "the right cue is on screen" a question with an answer.
    renderer: Mutex<RendererState>,
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
            renderer: Mutex::new(RendererState::new()),
            pump: Mutex::new(None),
            shut_down: AtomicBool::new(false),
        }
    }

    pub fn load(&self, locator: String) -> Result<(), PlaybackError> {
        // Forgotten before the command, not after: the outgoing medium's
        // document says nothing about the incoming one's moments, and a load
        // that fails must not leave the old answer standing either.
        lock(&self.renderer).forget();
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

    /// Shows the subtitle a menu row names (ADR-0013 Karar 3).
    ///
    /// **The one place that decides how a row reaches the screen.** An
    /// embedded row is the engine's own track and is selected; a user file is
    /// a document and is drawn by the renderer. The shell asks for a row and
    /// is told what happened — it does not learn which kind the row was, and
    /// the dialogue never leaves this side.
    ///
    /// Either way exactly one subtitle ends up on screen: selecting a track
    /// replaces whatever was drawn, and showing a document replaces whatever
    /// was selected.
    pub fn show_source(
        &self,
        library: &SubtitleLibrary,
        token: u32,
    ) -> Result<ShowOutcome, PlaybackError> {
        if !library.is_token_usable(token) {
            return Ok(ShowOutcome::Unusable);
        }
        if let Some(track) = library.embedded_track_of(token) {
            self.command(Operation::SelectTrack, |engine| {
                engine.select_track(TrackKind::Subtitle, Some(track))
            })?;
            // The engine is drawing its own track now, so nothing this side
            // showed is on screen any more.
            lock(&self.renderer).forget();
            return Ok(ShowOutcome::Shown);
        }
        let Some(document) = library.document_of(token) else {
            // Catalogued, not broken, not a track and with no document behind
            // it. Nothing exists to draw, and inventing a failure for it would
            // tell the user about a state that is not theirs.
            return Ok(ShowOutcome::Unusable);
        };
        self.render(|renderer| renderer.show(document))?;
        Ok(ShowOutcome::Shown)
    }

    /// Declares the share of the surface the shell's own chrome covers, so
    /// the subtitle stays out of it (ADR-0037).
    ///
    /// Called whenever the chrome appears or disappears, not once at startup:
    /// the transport bar is on screen for a fraction of a session and the
    /// subtitle belongs at the bottom the rest of the time. `0.0` is the
    /// ordinary value.
    ///
    /// Refuses anything outside `0.0..=0.5` instead of clamping — a clamped
    /// inset is indistinguishable from an honoured one (ADR-0037 Karar 2).
    pub fn set_subtitle_bottom_inset(&self, fraction: f32) -> Result<(), PlaybackError> {
        self.render(|renderer| renderer.set_bottom_inset(fraction))
    }

    /// §8's `Kapalı`: nothing on screen, whatever was drawing it.
    pub fn hide_subtitle(&self) -> Result<(), PlaybackError> {
        self.render(|renderer| renderer.clear())
    }

    /// The text that **should** be on screen at a moment, from the document
    /// the renderer was given.
    ///
    /// `nen_subtitle::CueIndex`'s answer (NEN-017), and nobody else's.
    /// `None` when nothing is showing or the moment falls in a gap.
    ///
    /// **Security:** dialogue (K23 #4). Comparable and displayable, never
    /// loggable.
    pub fn expected_subtitle_text(&self, at_ms: u64) -> Option<String> {
        let at_ms = u32::try_from(at_ms).unwrap_or(u32::MAX);
        lock(&self.renderer).expected_at(at_ms)
    }

    /// The text that **is** on screen, as the engine reports it.
    ///
    /// The other half of the pair: comparing the two is how NEN-027 proves the
    /// cue after a seek is the right one instead of assuming it. Needs the
    /// engine to declare
    /// [`RenderedTextObservation`](nen_ports::playback::Capability::RenderedTextObservation);
    /// without it the answer is a typed refusal, not a guess.
    pub fn rendered_subtitle_text(&self) -> Result<Option<String>, PlaybackError> {
        self.render(|renderer| renderer.rendered_text())
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

    /// Runs one renderer operation with the engine and the state in hand.
    ///
    /// The lock order is engine-then-state everywhere, which is what keeps two
    /// callers from meeting in the middle; nothing takes them the other way
    /// round.
    fn render<T>(
        &self,
        body: impl FnOnce(&mut EngineNativeRenderer<'_>) -> Result<T, RenderError>,
    ) -> Result<T, PlaybackError> {
        if self.shut_down.load(Ordering::Acquire) {
            return Err(PlaybackError::ShutDown {
                operation: Operation::InjectSubtitle,
            });
        }
        let mut engine = lock(&self.engine);
        let mut state = lock(&self.renderer);
        let mut renderer = EngineNativeRenderer::new(&mut *engine, &mut state);
        body(&mut renderer).map_err(playback_error)
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
