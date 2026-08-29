//! What the session adds to the bridge: something that pulls, and a lifetime.
//!
//! [`ShellEngineBridge`](nen_app::playback::ShellEngineBridge) is judged in
//! `playback_bridge.rs` — it forwards correctly and the shared kit says so.
//! This file is about the two things it structurally cannot do, both decided in
//! ADR-0033: pulling from the engine when nobody asked, and stopping in an
//! order that leaves no thread behind.
//!
//! The engine here is scripted rather than realistic. It reports exactly what a
//! test asks it to report and counts how often it was pulled, because those two
//! numbers are the whole subject.

use nen_app::session::PlaybackSession;
use nen_ports::playback::{
    Capability, EventQueue, Operation, PlaybackError, PlaybackEvent, PlaybackState,
    TrackDescriptor, TrackId, TrackKind,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// An engine that reports what it is told to and remembers what happened to it.
#[derive(Default)]
struct ScriptedEngine {
    /// What the engine is holding for its next `drain_events` — the unbounded
    /// buffer every foreign adapter has.
    reported: Mutex<Vec<PlaybackEvent>>,
    /// How many times anyone pulled.
    pulls: AtomicUsize,
    shutdowns: AtomicUsize,
}

impl ScriptedEngine {
    fn report(&self, events: impl IntoIterator<Item = PlaybackEvent>) {
        self.reported
            .lock()
            .expect("the fake is not poisoned")
            .extend(events);
    }

    fn buffered(&self) -> usize {
        self.reported
            .lock()
            .expect("the fake is not poisoned")
            .len()
    }

    fn pulls(&self) -> usize {
        self.pulls.load(Ordering::Acquire)
    }

    fn shutdowns(&self) -> usize {
        self.shutdowns.load(Ordering::Acquire)
    }
}

impl nen_app::playback::ShellEngine for ScriptedEngine {
    fn capabilities(&self) -> Vec<Capability> {
        Capability::ALL.into_iter().collect()
    }

    fn load(&self, _locator: String) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn play(&self) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn pause(&self) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn stop(&self) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn seek(&self, _to_ms: u64) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn position_ms(&self) -> Result<u64, PlaybackError> {
        Ok(0)
    }

    fn duration_ms(&self) -> Result<Option<u64>, PlaybackError> {
        Ok(Some(1_000))
    }

    fn state(&self) -> PlaybackState {
        PlaybackState::Ready
    }

    fn tracks(&self, _kind: TrackKind) -> Result<Vec<TrackDescriptor>, PlaybackError> {
        Ok(Vec::new())
    }

    fn select_track(&self, _kind: TrackKind, _track: Option<TrackId>) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn selected_track(&self, _kind: TrackKind) -> Result<Option<TrackId>, PlaybackError> {
        Ok(None)
    }

    fn drain_events(&self) -> Vec<PlaybackEvent> {
        self.pulls.fetch_add(1, Ordering::AcqRel);
        std::mem::take(&mut *self.reported.lock().expect("the fake is not poisoned"))
    }

    fn shutdown(&self) -> Result<(), PlaybackError> {
        self.shutdowns.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }

    fn set_rate(&self, _rate: f32) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn set_volume(&self, _volume: f32) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn extract_text(&self, _track: TrackId) -> Result<String, PlaybackError> {
        Ok(String::new())
    }

    fn inject_subtitle(&self, _webvtt: String) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn rendered_subtitle_text(&self) -> Result<Option<String>, PlaybackError> {
        Ok(None)
    }

    fn set_subtitle_bottom_inset(&self, _fraction: f32) -> Result<(), PlaybackError> {
        Ok(())
    }
}

fn engine() -> Arc<ScriptedEngine> {
    Arc::new(ScriptedEngine::default())
}

/// A burst of events that are never allowed to be dropped silently.
fn critical_burst(count: usize) -> Vec<PlaybackEvent> {
    (0..count).map(|_| PlaybackEvent::TracksChanged).collect()
}

fn positions(count: u64) -> Vec<PlaybackEvent> {
    (0..count)
        .map(|at| PlaybackEvent::PositionChanged {
            position: Duration::from_millis(at),
        })
        .collect()
}

/// Waits for something the pump thread is supposed to do.
///
/// Generous on purpose: the number being measured is "did it happen at all",
/// never "how fast", so the timeout is a failure detector rather than a
/// deadline the test is trying to meet.
fn wait_until(condition: impl Fn() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if condition() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    condition()
}

const FAST_PUMP: Duration = Duration::from_millis(1);

#[test]
fn the_pump_pulls_without_the_shell_asking() {
    let engine = engine();
    let session = PlaybackSession::with_pump_interval(Arc::clone(&engine) as Arc<_>, FAST_PUMP);

    engine.report(critical_burst(3));

    assert!(
        wait_until(|| engine.buffered() == 0),
        "the pump never emptied the engine's own buffer"
    );
    assert!(engine.pulls() > 0);

    // And what it pulled is in the queue, waiting for a shell that has not
    // asked yet.
    assert_eq!(session.drain_events().len(), 3);
}

#[test]
fn an_unwatched_engine_buffers_without_limit() {
    // The negative control for the decision above: the queue's 64-slot limit
    // is not, on its own, a limit on anything. With nothing pulling, the
    // backlog simply lives on the engine's side and grows.
    let engine = engine();
    let _session = PlaybackSession::without_pump(Arc::clone(&engine) as Arc<_>);

    engine.report(critical_burst(500));

    assert_eq!(engine.buffered(), 500);
    assert!(engine.buffered() > EventQueue::DEFAULT_CAPACITY);
}

#[test]
fn a_pulled_flood_collapses_into_one_loss_marker() {
    let engine = engine();
    let session = PlaybackSession::without_pump(Arc::clone(&engine) as Arc<_>);

    engine.report(critical_burst(500));
    session.pump_once();

    let delivered = session.drain_events();
    assert!(
        delivered.len() <= EventQueue::DEFAULT_CAPACITY,
        "delivered {} events, queue holds {}",
        delivered.len(),
        EventQueue::DEFAULT_CAPACITY
    );
    let dropped = match delivered.first() {
        Some(PlaybackEvent::EventsLost { dropped }) => *dropped,
        other => panic!("expected a loss marker first, got {other:?}"),
    };
    assert!(dropped > 0, "the marker claims nothing was lost");
    assert_eq!(
        dropped as usize + delivered.len() - 1,
        500,
        "every reported event is either delivered or counted as lost"
    );
}

#[test]
fn positions_never_occupy_more_than_one_slot() {
    let engine = engine();
    let session = PlaybackSession::without_pump(Arc::clone(&engine) as Arc<_>);

    engine.report(positions(1_000));
    session.pump_once();

    let delivered = session.drain_events();
    assert_eq!(
        delivered,
        vec![PlaybackEvent::PositionChanged {
            position: Duration::from_millis(999)
        }],
        "a thousand positions are one position: the newest"
    );
}

#[test]
fn critical_events_keep_the_order_they_happened_in() {
    let engine = engine();
    let session = PlaybackSession::without_pump(Arc::clone(&engine) as Arc<_>);

    let script = vec![
        PlaybackEvent::StateChanged {
            state: PlaybackState::Ready,
        },
        PlaybackEvent::SeekCompleted {
            position: Duration::from_millis(42),
        },
        PlaybackEvent::TracksChanged,
        PlaybackEvent::StateChanged {
            state: PlaybackState::Playing,
        },
        PlaybackEvent::EndReached,
    ];
    engine.report(script.clone());
    session.pump_once();

    assert_eq!(session.drain_events(), script);
}

#[test]
fn every_command_is_refused_after_shutdown() {
    let engine = engine();
    let session = PlaybackSession::without_pump(Arc::clone(&engine) as Arc<_>);
    session.shutdown().expect("shutdown succeeds");

    let refusals: Vec<(Operation, PlaybackError)> = vec![
        (
            Operation::Load,
            session.load("fixture".to_owned()).unwrap_err(),
        ),
        (Operation::Play, session.play().unwrap_err()),
        (Operation::Pause, session.pause().unwrap_err()),
        (Operation::Stop, session.stop().unwrap_err()),
        (
            Operation::Seek,
            session.seek(Duration::from_secs(1)).unwrap_err(),
        ),
        (Operation::Position, session.position().unwrap_err()),
        (Operation::Duration, session.duration().unwrap_err()),
        (Operation::State, session.state().unwrap_err()),
        (
            Operation::Tracks,
            session.tracks(TrackKind::Subtitle).unwrap_err(),
        ),
        (
            Operation::SelectTrack,
            session.select_track(TrackKind::Subtitle, None).unwrap_err(),
        ),
        (
            Operation::SelectTrack,
            session.selected_track(TrackKind::Audio).unwrap_err(),
        ),
        (Operation::SetRate, session.set_rate(1.5).unwrap_err()),
        (Operation::SetVolume, session.set_volume(0.5).unwrap_err()),
    ];

    for (operation, error) in refusals {
        assert_eq!(
            error,
            PlaybackError::ShutDown { operation },
            "{operation} was not refused as shut down"
        );
    }
}

#[test]
fn the_queue_still_answers_after_shutdown() {
    // The events a medium produced before it ended already happened; a shell
    // drawing its last frame needs them.
    let engine = engine();
    let session = PlaybackSession::without_pump(Arc::clone(&engine) as Arc<_>);

    engine.report([PlaybackEvent::EndReached]);
    session.shutdown().expect("shutdown succeeds");

    assert_eq!(session.drain_events(), vec![PlaybackEvent::EndReached]);
}

#[test]
fn shutdown_stops_the_pump() {
    let engine = engine();
    let session = PlaybackSession::with_pump_interval(Arc::clone(&engine) as Arc<_>, FAST_PUMP);
    assert!(wait_until(|| engine.pulls() > 0), "the pump never ran");

    session.shutdown().expect("shutdown succeeds");
    let settled = engine.pulls();

    std::thread::sleep(FAST_PUMP * 50);
    assert_eq!(
        engine.pulls(),
        settled,
        "something still pulled after shutdown joined the pump"
    );
}

#[test]
fn shutdown_is_idempotent_and_reaches_the_engine_once() {
    let engine = engine();
    let session = PlaybackSession::without_pump(Arc::clone(&engine) as Arc<_>);

    session.shutdown().expect("shutdown succeeds");
    session
        .shutdown()
        .expect("a second shutdown is not an error");

    assert_eq!(engine.shutdowns(), 1);
}

#[test]
fn dropping_the_session_stops_the_pump_and_the_engine() {
    let engine = engine();
    {
        let _session =
            PlaybackSession::with_pump_interval(Arc::clone(&engine) as Arc<_>, FAST_PUMP);
        assert!(wait_until(|| engine.pulls() > 0), "the pump never ran");
    }
    let settled = engine.pulls();

    std::thread::sleep(FAST_PUMP * 50);
    assert_eq!(engine.pulls(), settled, "a dropped session left a thread");
    assert_eq!(engine.shutdowns(), 1, "a dropped session left an engine");
}
