//! What the engine tells the core, and how that reaches the platform shell.
//!
//! ADR-0026 settled the direction: the Rust core owns the session and the
//! platform shell is called back. ADR-0011 Karar 1 settled what that delivery
//! must survive — NEN-029 measured a producer that keeps ticking while its
//! consumer is suspended (a Swift drain queue was held for 2 s and the Rust
//! side kept producing), so a delivery model that assumes a ready consumer is
//! measurably wrong.
//!
//! The answer here is a **bounded queue with per-class coalescing**:
//!
//! - [`DeliveryClass::Coalescing`] — at most one instance waits.
//!   [`PlaybackEvent::PositionChanged`] and
//!   [`PlaybackEvent::VideoGeometryChanged`] are the two, and dropping their
//!   intermediate instances loses nothing: both describe an absolute current
//!   value, so the newest one is the true one.
//! - [`DeliveryClass::Critical`] — order preserved, never silently dropped.
//!   `buffering → ready` is a sequence; dropping the first leaves the consumer
//!   in a state that never existed.
//!
//! When critical events would overflow anyway, the queue does not lose them
//! quietly: it collapses to a single [`PlaybackEvent::EventsLost`] carrying how
//! many went, and the consumer is required to resync. **The stream a consumer
//! sees is therefore either complete or says that it is not.**
//!
//! ADR-0011 Karar 2 lives here too: [`deliver_all`] marks the thread while a
//! callback runs, and [`guard_reentrancy`] turns a synchronous port call made
//! from inside that callback into [`PlaybackError::ReentrantCall`] instead of
//! the self-deadlock NEN-029 found.

use super::error::{Operation, PlaybackError};
use std::cell::Cell;
use std::collections::VecDeque;
use std::fmt;
use std::mem::discriminant;
use std::time::Duration;

/// Where playback is, as a whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlaybackState {
    /// Nothing loaded yet.
    Idle,
    /// Loading, or refilling after a seek — not able to present frames.
    Buffering,
    /// Loaded and able to play, but not playing.
    Ready,
    Playing,
    Paused,
    /// Reached the end of the medium.
    Ended,
    /// Failed; the accompanying [`PlaybackEvent::Failed`] carries why.
    Failed,
}

impl PlaybackState {
    /// Stable lowercase name. Safe to log.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Buffering => "buffering",
            Self::Ready => "ready",
            Self::Playing => "playing",
            Self::Paused => "paused",
            Self::Ended => "ended",
            Self::Failed => "failed",
        }
    }

    /// Whether media is loaded in this state.
    pub const fn has_media(self) -> bool {
        !matches!(self, Self::Idle)
    }
}

impl fmt::Display for PlaybackState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How an event must be treated when the consumer is behind (ADR-0011 Karar 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryClass {
    /// Only the newest instance is kept.
    Coalescing,
    /// Kept in order; never dropped without saying so.
    Critical,
}

/// Something the engine reports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackEvent {
    /// Playback advanced. Coalescing — see the module note.
    PositionChanged {
        position: Duration,
    },
    StateChanged {
        state: PlaybackState,
    },
    /// A seek finished, at the position actually reached.
    ///
    /// Separate from `PositionChanged` because it answers "did my seek land?",
    /// which a coalesced position cannot: the position that matters is the one
    /// belonging to *this* seek, not the newest one.
    SeekCompleted {
        position: Duration,
    },
    /// The track list changed and must be re-read.
    TracksChanged,
    /// The video's display size changed and must be re-read.
    ///
    /// **Carries no payload, on purpose** (ADR-0038 Karar 2). The consumer has
    /// to re-read after `EventsLost` anyway, so a value in the event would be a
    /// second source for the same truth — and the one that can go stale. What
    /// the event says is "ask again", and asking is the only way to learn the
    /// answer.
    ///
    /// Coalescing for the reason a position is: only the newest size matters.
    /// Measured on libmpv, one load produces two `MPV_EVENT_VIDEO_RECONFIG`
    /// (`evidence/M3/NEN-068-measurement.md`), and the intermediate one says
    /// nothing the final one does not.
    VideoGeometryChanged,
    EndReached,
    Failed {
        error: PlaybackError,
    },
    /// Delivery could not keep up and `dropped` critical events were lost.
    ///
    /// The consumer must resync — re-read state, position and tracks — rather
    /// than assume its view is current.
    EventsLost {
        dropped: u32,
    },
}

impl PlaybackEvent {
    pub const fn delivery_class(&self) -> DeliveryClass {
        match self {
            Self::PositionChanged { .. } | Self::VideoGeometryChanged => DeliveryClass::Coalescing,
            Self::StateChanged { .. }
            | Self::SeekCompleted { .. }
            | Self::TracksChanged
            | Self::EndReached
            | Self::Failed { .. }
            | Self::EventsLost { .. } => DeliveryClass::Critical,
        }
    }

    /// Whether a newer event replaces this one in the queue.
    ///
    /// Only ever true between two coalescing events of the same variant.
    fn is_replaced_by(&self, newer: &PlaybackEvent) -> bool {
        newer.delivery_class() == DeliveryClass::Coalescing
            && discriminant(self) == discriminant(newer)
    }
}

/// A bounded, coalescing queue of pending events.
///
/// Holds no lock and does no I/O: the adapter pushes what the engine reports,
/// and delivery happens separately in [`deliver_all`]. Keeping the two apart is
/// what makes the reentrancy rule enforceable in one place rather than in every
/// adapter.
#[derive(Debug, Clone)]
pub struct EventQueue {
    capacity: usize,
    events: VecDeque<PlaybackEvent>,
}

impl EventQueue {
    /// The capacity used when nothing else is specified.
    ///
    /// Sized for a consumer that is briefly behind (a backgrounded app catching
    /// up), not for one that is gone: critical events arrive at the speed of
    /// user actions and state transitions, so this holds a long burst of them
    /// while a coalescing event never occupies more than one slot.
    pub const DEFAULT_CAPACITY: usize = 64;

    /// Creates a queue. A capacity below 2 is raised to 2 — an overflow marker
    /// plus the event that caused it must both fit, or the queue could not
    /// report its own overflow.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(2),
            events: VecDeque::new(),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Enqueues an event, applying the delivery rules.
    ///
    /// A coalescing event replaces its pending twin **and moves to the back**:
    /// the queue's order then still reflects the order things were produced in,
    /// with the stale value gone rather than the fresh one held back.
    pub fn push(&mut self, event: PlaybackEvent) {
        if event.delivery_class() == DeliveryClass::Coalescing {
            self.events
                .retain(|pending| !pending.is_replaced_by(&event));
        }
        if self.events.len() >= self.capacity {
            self.collapse_to_loss_marker();
        }
        self.events.push_back(event);
    }

    /// Replaces the whole backlog with one [`PlaybackEvent::EventsLost`].
    ///
    /// Coalescing events are not counted as losses — losing an intermediate
    /// position is by design, and counting it would make the consumer resync
    /// over something that was never a loss. An existing marker's count is
    /// carried over so repeated overflows accumulate instead of resetting.
    fn collapse_to_loss_marker(&mut self) {
        let mut dropped: u32 = 0;
        for event in self.events.drain(..) {
            match event {
                PlaybackEvent::EventsLost { dropped: earlier } => {
                    dropped = dropped.saturating_add(earlier);
                }
                other if other.delivery_class() == DeliveryClass::Critical => {
                    dropped = dropped.saturating_add(1);
                }
                _ => {}
            }
        }
        self.events.push_back(PlaybackEvent::EventsLost { dropped });
    }

    pub fn pop(&mut self) -> Option<PlaybackEvent> {
        self.events.pop_front()
    }

    /// Removes and returns everything pending, in order.
    pub fn drain(&mut self) -> Vec<PlaybackEvent> {
        self.events.drain(..).collect()
    }

    /// The pending events, without removing them. For tests and diagnostics.
    pub fn pending(&self) -> &VecDeque<PlaybackEvent> {
        &self.events
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CAPACITY)
    }
}

/// Receives delivered events. Implemented by the platform shell.
pub trait EventSink {
    fn deliver(&self, event: PlaybackEvent);
}

thread_local! {
    static IN_CALLBACK: Cell<bool> = const { Cell::new(false) };
}

/// Marks the current thread as being inside an event callback.
///
/// Restores the previous mark on drop, so nesting behaves and an unwinding
/// callback cannot leave the thread permanently marked.
#[derive(Debug)]
pub struct CallbackScope {
    previous: bool,
}

impl CallbackScope {
    pub fn enter() -> Self {
        let previous = IN_CALLBACK.with(|flag| flag.replace(true));
        Self { previous }
    }
}

impl Drop for CallbackScope {
    fn drop(&mut self) {
        IN_CALLBACK.with(|flag| flag.set(self.previous));
    }
}

/// Whether this thread is currently inside an event callback.
pub fn in_callback() -> bool {
    IN_CALLBACK.with(Cell::get)
}

/// The check every port operation performs first (ADR-0011 Karar 2).
///
/// Returns [`PlaybackError::ReentrantCall`] when called from inside a callback
/// on the same thread. The caller is expected to defer the operation — to the
/// next run-loop turn, or to another thread — rather than to retry immediately.
pub fn guard_reentrancy(operation: Operation) -> Result<(), PlaybackError> {
    if in_callback() {
        Err(PlaybackError::ReentrantCall { operation })
    } else {
        Ok(())
    }
}

/// Delivers everything pending to the sink, with the reentrancy mark set.
///
/// The mark is what makes Karar 2 a property of the **port** rather than of
/// each adapter: an adapter pushes to the queue and never calls the sink
/// itself, so no adapter can forget to set it.
pub fn deliver_all(queue: &mut EventQueue, sink: &dyn EventSink) {
    let pending = queue.drain();
    if pending.is_empty() {
        return;
    }
    let _scope = CallbackScope::enter();
    for event in pending {
        sink.deliver(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn position(secs: u64) -> PlaybackEvent {
        PlaybackEvent::PositionChanged {
            position: Duration::from_secs(secs),
        }
    }

    fn state(state: PlaybackState) -> PlaybackEvent {
        PlaybackEvent::StateChanged { state }
    }

    #[derive(Default)]
    struct RecordingSink {
        seen: Mutex<Vec<PlaybackEvent>>,
    }

    impl EventSink for RecordingSink {
        fn deliver(&self, event: PlaybackEvent) {
            self.seen.lock().expect("sink lock").push(event);
        }
    }

    #[test]
    fn position_events_coalesce_to_the_newest_value() {
        let mut queue = EventQueue::default();
        queue.push(position(1));
        queue.push(position(2));
        queue.push(position(3));
        assert_eq!(queue.drain(), vec![position(3)]);
    }

    #[test]
    fn critical_events_keep_their_order_and_are_never_coalesced() {
        let mut queue = EventQueue::default();
        queue.push(state(PlaybackState::Buffering));
        queue.push(state(PlaybackState::Ready));
        queue.push(state(PlaybackState::Playing));
        assert_eq!(
            queue.drain(),
            vec![
                state(PlaybackState::Buffering),
                state(PlaybackState::Ready),
                state(PlaybackState::Playing),
            ]
        );
    }

    #[test]
    fn a_coalesced_event_moves_behind_the_critical_events_produced_after_it() {
        // Produced: pos(1), paused, pos(2). The stale position is gone, and
        // what remains is in production order — not the position frozen in the
        // slot where it first arrived.
        let mut queue = EventQueue::default();
        queue.push(position(1));
        queue.push(state(PlaybackState::Paused));
        queue.push(position(2));
        assert_eq!(
            queue.drain(),
            vec![state(PlaybackState::Paused), position(2)]
        );
    }

    #[test]
    fn geometry_events_coalesce_among_themselves_only() {
        // One load produces two reconfigurations on the real engine, and the
        // first says nothing the second does not. What must not happen is the
        // two coalescing classes collapsing into each other: a position is not
        // a geometry.
        let mut queue = EventQueue::default();
        queue.push(PlaybackEvent::VideoGeometryChanged);
        queue.push(position(4));
        queue.push(PlaybackEvent::VideoGeometryChanged);
        assert_eq!(
            queue.drain(),
            vec![position(4), PlaybackEvent::VideoGeometryChanged]
        );
    }

    #[test]
    fn a_dropped_geometry_event_is_not_counted_as_a_loss() {
        // Losing an intermediate geometry is by design, exactly as with a
        // position: counting it would send the consumer resyncing over nothing.
        let mut queue = EventQueue::new(2);
        queue.push(PlaybackEvent::VideoGeometryChanged);
        queue.push(PlaybackEvent::TracksChanged);
        queue.push(PlaybackEvent::EndReached);
        assert_eq!(queue.drain()[0], PlaybackEvent::EventsLost { dropped: 1 });
    }

    #[test]
    fn seek_completed_does_not_coalesce_with_position() {
        // It answers "did *my* seek land", which the newest position cannot.
        let mut queue = EventQueue::default();
        let seek = PlaybackEvent::SeekCompleted {
            position: Duration::from_secs(10),
        };
        queue.push(seek);
        queue.push(position(11));
        queue.push(position(12));
        assert_eq!(queue.drain(), vec![seek, position(12)]);
    }

    #[test]
    fn overflow_reports_the_loss_instead_of_dropping_silently() {
        let mut queue = EventQueue::new(4);
        for _ in 0..4 {
            queue.push(PlaybackEvent::TracksChanged);
        }
        queue.push(state(PlaybackState::Ended));

        assert_eq!(
            queue.drain(),
            vec![
                PlaybackEvent::EventsLost { dropped: 4 },
                state(PlaybackState::Ended),
            ]
        );
    }

    #[test]
    fn repeated_overflow_accumulates_the_loss_count() {
        let mut queue = EventQueue::new(2);
        // Fill, overflow, fill, overflow again.
        for _ in 0..6 {
            queue.push(PlaybackEvent::TracksChanged);
        }
        let drained = queue.drain();
        let PlaybackEvent::EventsLost { dropped } = drained[0] else {
            panic!("expected a loss marker first, got {drained:?}");
        };
        // Nothing may be forgotten: every critical event that went is counted.
        let delivered = drained.len() - 1;
        assert_eq!(dropped as usize + delivered, 6);
    }

    #[test]
    fn coalesced_losses_are_not_counted_as_lost_events() {
        // Losing an intermediate position is by design, not a loss to report;
        // counting it would send the consumer resyncing over nothing.
        let mut queue = EventQueue::new(2);
        queue.push(position(1));
        queue.push(PlaybackEvent::TracksChanged);
        queue.push(PlaybackEvent::EndReached);
        let drained = queue.drain();
        assert_eq!(drained[0], PlaybackEvent::EventsLost { dropped: 1 });
    }

    #[test]
    fn capacity_below_two_is_raised_so_overflow_can_be_reported() {
        let mut queue = EventQueue::new(0);
        assert_eq!(queue.capacity(), 2);
        queue.push(PlaybackEvent::TracksChanged);
        queue.push(PlaybackEvent::EndReached);
        queue.push(PlaybackEvent::TracksChanged);
        let drained = queue.drain();
        assert_eq!(drained[0], PlaybackEvent::EventsLost { dropped: 2 });
        assert_eq!(drained[1], PlaybackEvent::TracksChanged);
    }

    #[test]
    fn delivery_marks_the_thread_and_clears_it_afterwards() {
        struct ReentrantSink;
        impl EventSink for ReentrantSink {
            fn deliver(&self, _event: PlaybackEvent) {
                assert!(in_callback(), "the mark must be set during delivery");
                assert_eq!(
                    guard_reentrancy(Operation::Play),
                    Err(PlaybackError::ReentrantCall {
                        operation: Operation::Play
                    })
                );
            }
        }

        let mut queue = EventQueue::default();
        queue.push(PlaybackEvent::EndReached);
        assert!(!in_callback());
        deliver_all(&mut queue, &ReentrantSink);
        assert!(!in_callback(), "the mark must be cleared after delivery");
        assert!(guard_reentrancy(Operation::Play).is_ok());
    }

    #[test]
    fn delivery_preserves_order_and_empties_the_queue() {
        let sink = RecordingSink::default();
        let mut queue = EventQueue::default();
        queue.push(state(PlaybackState::Ready));
        queue.push(position(5));
        queue.push(PlaybackEvent::EndReached);

        deliver_all(&mut queue, &sink);

        assert!(queue.is_empty());
        assert_eq!(
            *sink.seen.lock().expect("sink lock"),
            vec![
                state(PlaybackState::Ready),
                position(5),
                PlaybackEvent::EndReached,
            ]
        );
    }

    #[test]
    fn an_empty_queue_never_enters_a_callback_scope() {
        struct NeverCalled;
        impl EventSink for NeverCalled {
            fn deliver(&self, _event: PlaybackEvent) {
                panic!("nothing was queued");
            }
        }
        let mut queue = EventQueue::default();
        deliver_all(&mut queue, &NeverCalled);
        assert!(!in_callback());
    }

    #[test]
    fn a_panicking_callback_does_not_leave_the_thread_marked() {
        struct PanickingSink;
        impl EventSink for PanickingSink {
            fn deliver(&self, _event: PlaybackEvent) {
                panic!("callback blew up");
            }
        }
        let mut queue = EventQueue::default();
        queue.push(PlaybackEvent::EndReached);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            deliver_all(&mut queue, &PanickingSink);
        }));
        assert!(result.is_err());
        assert!(
            !in_callback(),
            "an unwinding callback must not strand the mark"
        );
    }

    #[test]
    fn nested_scopes_restore_the_previous_mark() {
        assert!(!in_callback());
        {
            let _outer = CallbackScope::enter();
            assert!(in_callback());
            {
                let _inner = CallbackScope::enter();
                assert!(in_callback());
            }
            assert!(in_callback(), "the outer scope is still active");
        }
        assert!(!in_callback());
    }
}
