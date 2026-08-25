//! NEN-021 DoD #4: the event stream's ordering guarantee, driven through a real
//! engine rather than through the queue alone.
//!
//! ADR-0011 Karar 1 in one sentence: **a consumer's view is either complete or
//! says that it is not.** The scenario that forces the question is the one
//! NEN-029 measured — the Swift consumer was suspended for 2 s while the Rust
//! producer kept ticking. These tests reproduce that shape: nobody drains, the
//! engine keeps reporting, and the contract still has to hold.

use nen_ports::playback::event::{deliver_all, EventQueue, EventSink};
use nen_ports::playback::fake::{fake_inputs, FakeEngine};
use nen_ports::playback::{PlaybackEngine, PlaybackEvent, PlaybackState};
use std::sync::Mutex;
use std::time::Duration;

#[derive(Default)]
struct RecordingSink {
    seen: Mutex<Vec<PlaybackEvent>>,
}

impl RecordingSink {
    fn taken(&self) -> Vec<PlaybackEvent> {
        std::mem::take(&mut *self.seen.lock().expect("sink lock"))
    }
}

impl EventSink for RecordingSink {
    fn deliver(&self, event: PlaybackEvent) {
        self.seen.lock().expect("sink lock").push(event);
    }
}

fn loaded() -> FakeEngine {
    let mut engine = FakeEngine::full();
    engine.load(&fake_inputs().media).expect("load");
    engine.events().drain();
    engine
}

#[test]
fn state_transitions_reach_the_consumer_in_the_order_they_happened() {
    let engine = &mut loaded();
    let sink = RecordingSink::default();

    engine.play().expect("play");
    engine.pause().expect("pause");
    engine.play().expect("play");
    deliver_all(engine.events(), &sink);

    assert_eq!(
        sink.taken(),
        vec![
            PlaybackEvent::StateChanged {
                state: PlaybackState::Playing
            },
            PlaybackEvent::StateChanged {
                state: PlaybackState::Paused
            },
            PlaybackEvent::StateChanged {
                state: PlaybackState::Playing
            },
        ]
    );
}

#[test]
fn a_suspended_consumer_loses_positions_but_no_state_transitions() {
    // The NEN-029 shape: the producer keeps going while nobody drains.
    let engine = &mut loaded();

    for step in 1..=200u64 {
        engine
            .seek(Duration::from_millis(step * 100))
            .expect("seek");
    }

    let pending = engine.events().drain();
    let positions = pending
        .iter()
        .filter(|event| matches!(event, PlaybackEvent::PositionChanged { .. }))
        .count();
    assert_eq!(positions, 1, "only the newest position may survive");

    let last = pending
        .iter()
        .rev()
        .find_map(|event| match event {
            PlaybackEvent::PositionChanged { position } => Some(*position),
            _ => None,
        })
        .expect("a position survived");
    assert_eq!(
        last,
        Duration::from_millis(20_000),
        "the surviving position must be the newest, not the oldest"
    );
}

#[test]
fn overflow_tells_the_consumer_instead_of_lying_to_it() {
    // A queue small enough to overflow within one burst of critical events.
    let mut queue = EventQueue::new(8);
    for _ in 0..64 {
        queue.push(PlaybackEvent::TracksChanged);
    }

    let sink = RecordingSink::default();
    deliver_all(&mut queue, &sink);
    let seen = sink.taken();

    let PlaybackEvent::EventsLost { dropped } = seen[0] else {
        panic!("overflow must announce itself first, got {seen:?}");
    };
    // Nothing vanished unaccounted for: what was dropped plus what arrived is
    // everything that was produced.
    assert_eq!(dropped as usize + (seen.len() - 1), 64);
    assert!(
        !seen[1..]
            .iter()
            .any(|event| matches!(event, PlaybackEvent::EventsLost { .. })),
        "only one marker per overflow burst: {seen:?}"
    );
}

#[test]
fn after_a_loss_the_engine_can_still_be_resynced() {
    // The consumer's obligation after `EventsLost` is to re-read state — and
    // the engine must be able to answer, because the loss was in delivery, not
    // in the engine.
    let engine = &mut loaded();
    engine.play().expect("play");
    engine.seek(Duration::from_millis(30_000)).expect("seek");

    // Simulate a consumer that missed everything.
    engine.events().drain();

    assert_eq!(engine.state(), PlaybackState::Playing);
    assert_eq!(
        engine.position().expect("position"),
        Duration::from_millis(30_000)
    );
    assert_eq!(
        engine.duration().expect("duration"),
        Some(Duration::from_millis(120_000))
    );
}

#[test]
fn seek_completion_survives_a_burst_of_positions() {
    // SeekCompleted answers "did *my* seek land". Coalescing must never take
    // it, however many positions follow.
    let engine = &mut loaded();

    engine.seek(Duration::from_millis(1_000)).expect("seek");
    engine.seek(Duration::from_millis(2_000)).expect("seek");
    engine.seek(Duration::from_millis(3_000)).expect("seek");

    let pending = engine.events().drain();
    let completions = pending
        .iter()
        .filter(|event| matches!(event, PlaybackEvent::SeekCompleted { .. }))
        .count();
    assert_eq!(completions, 3, "every seek must report its own completion");
}

#[test]
fn reaching_the_end_reports_the_state_before_the_end_marker() {
    let engine = &mut loaded();
    engine.play().expect("play");
    engine.events().drain();

    engine.seek(Duration::from_secs(9_999)).expect("seek");

    let shapes: Vec<PlaybackEvent> = engine.events().drain();
    let ended_at = shapes
        .iter()
        .position(|event| {
            matches!(
                event,
                PlaybackEvent::StateChanged {
                    state: PlaybackState::Ended
                }
            )
        })
        .expect("an ended transition");
    let marker_at = shapes
        .iter()
        .position(|event| matches!(event, PlaybackEvent::EndReached))
        .expect("an end marker");
    assert!(
        ended_at < marker_at,
        "the state must be current before the marker arrives: {shapes:?}"
    );
}
