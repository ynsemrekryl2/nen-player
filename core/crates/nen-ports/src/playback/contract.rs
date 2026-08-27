//! The shared contract kit every `PlaybackEngine` adapter must pass.
//!
//! ADR-0011 Karar 4: **the scenarios are data, not code.** A scenario is a list
//! of steps, each an [`Action`] paired with the [`Outcome`] the contract
//! requires — no closures, no adapter-specific assertions. That is what lets
//! M3's exit criterion hold literally: NEN-022 drives *this* list through FFI
//! against the real libmpv adapter instead of writing a second kit that drifts
//! from this one.
//!
//! `docs/testing-strategy.md` names the evidence this produces: "aynı kitin
//! fake + gerçek adapter'da geçmesi".
//!
//! A scenario declares when it applies ([`Applicability`]), so one list covers
//! engines with different capability sets: the capability-gated behaviour is
//! checked on engines that declare it, and the typed refusal is checked on
//! engines that do not.
//!
//! # Why nothing here is a number an adapter chose
//!
//! NEN-022 measured what a real engine does, and two habits of the first draft
//! turned out to be the fake's biography rather than the contract:
//!
//! - **Durations, track counts and track ids were literals** (`120_000`,
//!   `TrackCount(2)`, `TrackId(1)`) copied from [`super::fake`]. A real
//!   fixture would have had to be exactly 120 s with exactly those ids — which
//!   is the kit shaping itself around one adapter, the thing ADR-0011 Karar 4
//!   exists to prevent. They now live in [`ContractInputs`], and a step names
//!   what it *means* ([`TrackRef::Known`], [`Outcome::FixtureDuration`])
//!   instead of a number that means nothing on its own.
//! - **Every assertion was instantaneous and exact.** A real engine loads
//!   asynchronously and reports a stream richer than the one the fake emits.
//!   [`Action::Settle`] waits for a state instead of assuming it,
//!   [`Outcome::PositionNear`] allows a measured tolerance, and
//!   [`Outcome::Events`] matches a **subsequence** so an engine may say more
//!   than the contract requires — but never less, and never a `Failed` or
//!   `EventsLost` the scenario did not ask for.
//!
//! Loosening a kit is how a kit stops catching things, so
//! `tests/contract_kit_is_not_vacuous.rs` breaks a fake in each of the
//! directions this file now tolerates and proves the kit still goes red.

use super::capability::{Capabilities, Capability};
use super::engine::PlaybackEngine;
use super::error::PlaybackError;
use super::event::{CallbackScope, PlaybackEvent, PlaybackState};
use super::media::MediaSource;
use super::track::{TrackId, TrackKind};
use nen_domain::subtitle::SubtitleDocument;
use std::fmt;
use std::time::{Duration, Instant};

/// Which track a step means, without naming the adapter's numbers.
///
/// `SelectTrack(Subtitle, Known(Subtitle))` reads as the contract intends it —
/// "select a subtitle track this medium really has" — and stays true whatever
/// ids the engine hands out. The runner resolves it from [`ContractInputs`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackRef {
    /// An id belonging to a track of this kind that the fixture really has.
    Known(TrackKind),
    /// An id no track of any kind has.
    Unknown,
}

impl TrackRef {
    fn resolve(self, inputs: &ContractInputs) -> TrackId {
        match self {
            Self::Known(TrackKind::Audio) => inputs.audio_track,
            Self::Known(TrackKind::Subtitle) => inputs.subtitle_track,
            Self::Unknown => inputs.unknown_track,
        }
    }
}

/// One port operation, as data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Load,
    Play,
    Pause,
    Stop,
    Seek {
        to_ms: u64,
    },
    SeekRelative {
        delta_ms: i64,
    },
    Position,
    Duration,
    State,
    Tracks {
        kind: TrackKind,
    },
    SelectTrack {
        kind: TrackKind,
        track: Option<TrackRef>,
    },
    SelectedTrack {
        kind: TrackKind,
    },
    SetRate {
        rate: f32,
    },
    SetVolume {
        volume: f32,
    },
    ExtractText {
        track: TrackRef,
    },
    InjectSubtitle,
    Shutdown,
    /// Waits until a given event shape has been reported, or gives up.
    ///
    /// A seek is not finished when `seek` returns — that is what
    /// [`PlaybackEvent::SeekCompleted`](super::event::PlaybackEvent::SeekCompleted)
    /// is *for*. Reading the position straight after the command reads the old
    /// one, and draining straight after finds an empty queue: NEN-022 measured
    /// both against libmpv. A scenario therefore waits for the answer before
    /// asking about its consequences.
    AwaitEvent {
        shape: EventShape,
    },
    /// Waits for a `SeekCompleted` and checks **which position it carries**.
    ///
    /// [`Action::AwaitEvent`] can only ask whether an answer arrived.
    /// NEN-051 measured what that misses: an adapter answered a seek with the
    /// position the medium held *before* the seek was served — `0 ms` — and the
    /// scenario still passed, because the `Position` step that followed asked
    /// the engine directly and by then the seek had landed. The event was
    /// wrong; nothing looked at it.
    ///
    /// Judged against [`ContractInputs::seek_tolerance_ms`], the same margin
    /// [`Outcome::PositionNear`] uses. The last landing is the one checked: a
    /// scenario drains before each seek it means to judge.
    AwaitSeekLanding {
        near_ms: u64,
    },
    /// Waits until the engine reaches a state, or gives up.
    ///
    /// The fake reaches its states inside the call that causes them; a real
    /// engine does not — `loadfile` returns long before the medium is ready.
    /// Asserting the state directly would therefore test timing, not
    /// behaviour. A scenario says "settle, *then* look".
    ///
    /// Failing to settle is a failure: an engine that never reaches the state
    /// is exactly what this step is here to catch.
    Settle {
        until: PlaybackState,
    },
    /// Takes everything pending off the event queue, so the next
    /// [`Outcome::Events`] describes only what happened after this point.
    DrainEvents,
}

/// The variant of a [`PlaybackError`], without its payload.
///
/// Scenarios assert the *kind* of refusal, not an engine's private numbers: an
/// `EngineFailure` code is meaningless across adapters, and a scenario that
/// pinned one could only ever pass on a single engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Unsupported,
    ReentrantCall,
    NotLoaded,
    ShutDown,
    UnknownTrack,
    RateOutOfRange,
    LoadFailed,
    EngineFailure,
}

impl ErrorKind {
    fn of(error: &PlaybackError) -> Self {
        match error {
            PlaybackError::Unsupported { .. } => Self::Unsupported,
            PlaybackError::ReentrantCall { .. } => Self::ReentrantCall,
            PlaybackError::NotLoaded { .. } => Self::NotLoaded,
            PlaybackError::ShutDown { .. } => Self::ShutDown,
            PlaybackError::UnknownTrack { .. } => Self::UnknownTrack,
            PlaybackError::RateOutOfRange { .. } => Self::RateOutOfRange,
            PlaybackError::LoadFailed { .. } => Self::LoadFailed,
            PlaybackError::EngineFailure { .. } => Self::EngineFailure,
        }
    }
}

/// The shape of an event, without values that legitimately differ per engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventShape {
    PositionChanged,
    StateChanged(PlaybackState),
    SeekCompleted,
    TracksChanged,
    EndReached,
    Failed,
    EventsLost,
}

impl EventShape {
    fn of(event: &PlaybackEvent) -> Self {
        match event {
            PlaybackEvent::PositionChanged { .. } => Self::PositionChanged,
            PlaybackEvent::StateChanged { state } => Self::StateChanged(*state),
            PlaybackEvent::SeekCompleted { .. } => Self::SeekCompleted,
            PlaybackEvent::TracksChanged => Self::TracksChanged,
            PlaybackEvent::EndReached => Self::EndReached,
            PlaybackEvent::Failed { .. } => Self::Failed,
            PlaybackEvent::EventsLost { .. } => Self::EventsLost,
        }
    }
}

/// What the contract requires an action to produce.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// Succeeded; the returned value is not what this step is about.
    Ok,
    Error(ErrorKind),
    State(PlaybackState),
    /// Within [`ContractInputs::seek_tolerance_ms`] of this position.
    ///
    /// Not exact equality: a real engine seeks in the medium's own units and
    /// lands *near* the request. The tolerance is the adapter's, declared with
    /// its fixture, so an engine cannot widen it to hide a bad seek — and the
    /// fake keeps its tolerance at zero, so the fake stays exact.
    PositionNear(u64),
    /// The duration the fixture declares ([`ContractInputs::duration_ms`]);
    /// `None` there means "this medium reports no duration" (a live stream).
    FixtureDuration,
    /// As many tracks of this kind as the fixture declares.
    FixtureTrackCount(TrackKind),
    SelectedTrack(Option<TrackRef>),
    /// These event shapes, in this order, **as a subsequence**, arriving
    /// within the fixture's settle timeout.
    ///
    /// An engine may report more than the contract requires — a real one emits
    /// reconfiguration events around every seek — but never less and never out
    /// of order. Two shapes it may not add unasked: `Failed` and `EventsLost`
    /// are refused unless the scenario lists them, so "extras are allowed"
    /// can never swallow a failure.
    ///
    /// **Waiting, not sampling.** Events arrive on the engine's own thread, so
    /// a single drain the instant after a command sees whatever happens to have
    /// landed. The runner keeps draining until the expectation is satisfied or
    /// the timeout runs out; what it accumulates since the last
    /// `DrainEvents`/`Outcome::Ok` is what gets matched.
    ///
    /// Exact queue behaviour (coalescing, overflow, resync) is not asserted
    /// here: it belongs to [`super::event::EventQueue`], which both adapters
    /// share, and is proven exactly in `tests/event_ordering.rs`.
    Events(Vec<EventShape>),
    /// Text came back and is non-empty. The text itself is subtitle dialogue
    /// (K23 #4) and is never compared or printed here.
    NonEmptyText,
}

/// One step of a scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub action: Action,
    pub expect: Outcome,
    /// Run this action while the thread is marked as being inside an event
    /// callback (ADR-0011 Karar 2).
    ///
    /// The mark is exactly what `super::event::deliver_all` sets around real
    /// delivery, so a step with this set reproduces the reentrant call without
    /// needing a live engine to emit an event first.
    pub inside_callback: bool,
}

impl Step {
    pub fn new(action: Action, expect: Outcome) -> Self {
        Self {
            action,
            expect,
            inside_callback: false,
        }
    }

    pub fn from_callback(action: Action, expect: Outcome) -> Self {
        Self {
            action,
            expect,
            inside_callback: true,
        }
    }

    /// Wait for a state before asserting anything about it.
    pub fn settle(until: PlaybackState) -> Self {
        Self::new(Action::Settle { until }, Outcome::Ok)
    }

    /// Wait for an event before asserting anything about its consequences.
    pub fn awaits(shape: EventShape) -> Self {
        Self::new(Action::AwaitEvent { shape }, Outcome::Ok)
    }

    /// Wait for a seek's answer and require it to name the right position.
    pub fn awaits_seek_landing(near_ms: u64) -> Self {
        Self::new(Action::AwaitSeekLanding { near_ms }, Outcome::Ok)
    }
}

/// When a scenario applies to an engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applicability {
    /// Base behaviour — every adapter, always.
    Always,
    /// Only for engines that declare the capability.
    WithCapability(Capability),
    /// Only for engines that do not — this is where the typed refusal is
    /// proven.
    WithoutCapability(Capability),
}

impl Applicability {
    pub fn applies_to(self, capabilities: Capabilities) -> bool {
        match self {
            Self::Always => true,
            Self::WithCapability(capability) => capabilities.contains(capability),
            Self::WithoutCapability(capability) => !capabilities.contains(capability),
        }
    }
}

/// A named sequence of steps.
#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    pub name: &'static str,
    pub applies: Applicability,
    pub steps: Vec<Step>,
}

/// A step that did not produce what the contract requires.
#[derive(Debug, Clone, PartialEq)]
pub struct Failure {
    pub scenario: &'static str,
    pub step: usize,
    pub action: Action,
    pub expected: Outcome,
    pub detail: String,
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: step {} ({:?}) expected {:?} — {}",
            self.scenario, self.step, self.action, self.expected, self.detail
        )
    }
}

/// The one list. Adding a scenario here subjects **every** adapter to it.
pub fn scenarios() -> Vec<Scenario> {
    let mut all = vec![
        Scenario {
            name: "an engine starts idle and refuses what needs media",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::State, Outcome::State(PlaybackState::Idle)),
                Step::new(Action::Play, Outcome::Error(ErrorKind::NotLoaded)),
                Step::new(Action::Pause, Outcome::Error(ErrorKind::NotLoaded)),
                Step::new(
                    Action::Seek { to_ms: 0 },
                    Outcome::Error(ErrorKind::NotLoaded),
                ),
                Step::new(Action::Position, Outcome::Error(ErrorKind::NotLoaded)),
                Step::new(Action::Duration, Outcome::Error(ErrorKind::NotLoaded)),
                Step::new(
                    Action::Tracks {
                        kind: TrackKind::Subtitle,
                    },
                    Outcome::Error(ErrorKind::NotLoaded),
                ),
            ],
        },
        Scenario {
            name: "load then play reaches the playing state",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(Action::State, Outcome::State(PlaybackState::Ready)),
                Step::new(Action::Play, Outcome::Ok),
                Step::settle(PlaybackState::Playing),
                Step::new(Action::State, Outcome::State(PlaybackState::Playing)),
                Step::new(Action::Pause, Outcome::Ok),
                Step::settle(PlaybackState::Paused),
                Step::new(Action::State, Outcome::State(PlaybackState::Paused)),
            ],
        },
        Scenario {
            name: "loading reports buffering before ready, in order",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(
                    Action::DrainEvents,
                    Outcome::Events(vec![
                        EventShape::StateChanged(PlaybackState::Buffering),
                        EventShape::StateChanged(PlaybackState::Ready),
                        EventShape::TracksChanged,
                    ]),
                ),
            ],
        },
        Scenario {
            name: "a seek reports completion, not just a new position",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(Action::DrainEvents, Outcome::Ok),
                Step::new(Action::Seek { to_ms: 5_000 }, Outcome::Ok),
                // The answer must name where *this* seek landed, not merely
                // exist: the `Position` step below would pass on its own even
                // if the event carried a stale position (NEN-051).
                Step::awaits_seek_landing(5_000),
                Step::new(Action::Position, Outcome::PositionNear(5_000)),
                Step::new(
                    Action::DrainEvents,
                    Outcome::Events(vec![EventShape::SeekCompleted, EventShape::PositionChanged]),
                ),
            ],
        },
        Scenario {
            name: "relative seek moves from the current position and clamps at zero",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(Action::Seek { to_ms: 10_000 }, Outcome::Ok),
                Step::awaits_seek_landing(10_000),
                // Relative seek is `position` + `seek` (ADR-0011 Karar 3), so
                // it can only be right if the seek before it has landed.
                Step::new(Action::DrainEvents, Outcome::Ok),
                Step::new(Action::SeekRelative { delta_ms: 2_000 }, Outcome::Ok),
                Step::awaits_seek_landing(12_000),
                Step::new(Action::Position, Outcome::PositionNear(12_000)),
                Step::new(Action::DrainEvents, Outcome::Ok),
                Step::new(Action::SeekRelative { delta_ms: -50_000 }, Outcome::Ok),
                Step::awaits_seek_landing(0),
                Step::new(Action::Position, Outcome::PositionNear(0)),
            ],
        },
        Scenario {
            name: "seeking past the end ends playback instead of failing",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(Action::Play, Outcome::Ok),
                Step::new(Action::DrainEvents, Outcome::Ok),
                Step::new(
                    Action::Seek {
                        to_ms: u64::from(u32::MAX),
                    },
                    Outcome::Ok,
                ),
                Step::settle(PlaybackState::Ended),
                Step::new(Action::State, Outcome::State(PlaybackState::Ended)),
                Step::new(
                    Action::DrainEvents,
                    Outcome::Events(vec![
                        EventShape::SeekCompleted,
                        EventShape::PositionChanged,
                        EventShape::StateChanged(PlaybackState::Ended),
                        EventShape::EndReached,
                    ]),
                ),
            ],
        },
        Scenario {
            name: "tracks are enumerated and selectable, and can be turned off",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(
                    Action::Tracks {
                        kind: TrackKind::Subtitle,
                    },
                    Outcome::FixtureTrackCount(TrackKind::Subtitle),
                ),
                Step::new(
                    Action::Tracks {
                        kind: TrackKind::Audio,
                    },
                    Outcome::FixtureTrackCount(TrackKind::Audio),
                ),
                Step::new(
                    Action::SelectedTrack {
                        kind: TrackKind::Subtitle,
                    },
                    Outcome::SelectedTrack(None),
                ),
                Step::new(
                    Action::SelectTrack {
                        kind: TrackKind::Subtitle,
                        track: Some(TrackRef::Known(TrackKind::Subtitle)),
                    },
                    Outcome::Ok,
                ),
                Step::new(
                    Action::SelectedTrack {
                        kind: TrackKind::Subtitle,
                    },
                    Outcome::SelectedTrack(Some(TrackRef::Known(TrackKind::Subtitle))),
                ),
                // `Kapalı` (§8) is selecting nothing, not an error.
                Step::new(
                    Action::SelectTrack {
                        kind: TrackKind::Subtitle,
                        track: None,
                    },
                    Outcome::Ok,
                ),
                Step::new(
                    Action::SelectedTrack {
                        kind: TrackKind::Subtitle,
                    },
                    Outcome::SelectedTrack(None),
                ),
            ],
        },
        Scenario {
            name: "selecting a track that does not exist is a typed error",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(
                    Action::SelectTrack {
                        kind: TrackKind::Subtitle,
                        track: Some(TrackRef::Unknown),
                    },
                    Outcome::Error(ErrorKind::UnknownTrack),
                ),
                // The failed selection changed nothing.
                Step::new(
                    Action::SelectedTrack {
                        kind: TrackKind::Subtitle,
                    },
                    Outcome::SelectedTrack(None),
                ),
            ],
        },
        Scenario {
            name: "an audio id is not a subtitle id",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                // A subtitle track's id, offered as an audio one. Ids are one
                // space across kinds; an engine that numbers per kind must map
                // them into one, or this refuses to hold.
                Step::new(
                    Action::SelectTrack {
                        kind: TrackKind::Audio,
                        track: Some(TrackRef::Known(TrackKind::Subtitle)),
                    },
                    Outcome::Error(ErrorKind::UnknownTrack),
                ),
            ],
        },
        Scenario {
            name: "stop returns the engine to idle and it can load again",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(Action::Play, Outcome::Ok),
                Step::new(Action::Stop, Outcome::Ok),
                Step::settle(PlaybackState::Idle),
                Step::new(Action::State, Outcome::State(PlaybackState::Idle)),
                Step::new(Action::Play, Outcome::Error(ErrorKind::NotLoaded)),
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(Action::State, Outcome::State(PlaybackState::Ready)),
            ],
        },
        Scenario {
            name: "shutdown is idempotent and refuses everything afterwards",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::new(Action::Shutdown, Outcome::Ok),
                // A shell cannot always know whether its teardown already ran.
                Step::new(Action::Shutdown, Outcome::Ok),
                Step::new(Action::Play, Outcome::Error(ErrorKind::ShutDown)),
                Step::new(Action::Load, Outcome::Error(ErrorKind::ShutDown)),
                Step::new(
                    Action::Seek { to_ms: 0 },
                    Outcome::Error(ErrorKind::ShutDown),
                ),
                Step::new(Action::Position, Outcome::Error(ErrorKind::ShutDown)),
            ],
        },
        Scenario {
            name: "duration is reported once media is loaded",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(Action::Duration, Outcome::FixtureDuration),
            ],
        },
        Scenario {
            name: "position events coalesce while critical events keep their order",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(Action::DrainEvents, Outcome::Ok),
                Step::new(Action::Play, Outcome::Ok),
                Step::new(Action::Seek { to_ms: 1_000 }, Outcome::Ok),
                Step::new(Action::Seek { to_ms: 2_000 }, Outcome::Ok),
                Step::settle(PlaybackState::Playing),
                // Both completions survive and keep their order: a seek's
                // answer is never coalesced away. That the *positions* between
                // them collapse is a property of the shared `EventQueue` and
                // is proven exactly in `tests/event_ordering.rs`.
                Step::new(
                    Action::DrainEvents,
                    Outcome::Events(vec![
                        EventShape::StateChanged(PlaybackState::Playing),
                        EventShape::SeekCompleted,
                        EventShape::SeekCompleted,
                    ]),
                ),
            ],
        },
    ];

    all.extend(reentrancy_scenarios());
    all.extend(capability_scenarios());
    all
}

/// One scenario per operation, proving Karar 2 holds for all of them.
fn reentrancy_scenarios() -> Vec<Scenario> {
    let actions = [
        Action::Play,
        Action::Pause,
        Action::Stop,
        Action::Seek { to_ms: 0 },
        Action::SeekRelative { delta_ms: 1_000 },
        Action::SelectTrack {
            kind: TrackKind::Subtitle,
            track: None,
        },
        Action::Shutdown,
    ];
    vec![Scenario {
        name: "no operation may be called synchronously from inside a callback",
        applies: Applicability::Always,
        steps: [
            Step::new(Action::Load, Outcome::Ok),
            Step::settle(PlaybackState::Ready),
        ]
        .into_iter()
        .chain(
            actions.into_iter().map(|action| {
                Step::from_callback(action, Outcome::Error(ErrorKind::ReentrantCall))
            }),
        )
        // The engine is untouched: the refusals really refused.
        .chain(std::iter::once(Step::new(
            Action::State,
            Outcome::State(PlaybackState::Ready),
        )))
        .collect(),
    }]
}

/// For each capability: the behaviour when it is declared, and the typed
/// refusal when it is not.
fn capability_scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "rate: refused when the capability is absent",
            applies: Applicability::WithoutCapability(Capability::PlaybackRate),
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(
                    Action::SetRate { rate: 1.5 },
                    Outcome::Error(ErrorKind::Unsupported),
                ),
            ],
        },
        Scenario {
            name: "rate: accepted when the capability is present",
            applies: Applicability::WithCapability(Capability::PlaybackRate),
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(Action::SetRate { rate: 1.5 }, Outcome::Ok),
            ],
        },
        Scenario {
            name: "volume: refused when the capability is absent",
            applies: Applicability::WithoutCapability(Capability::Volume),
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(
                    Action::SetVolume { volume: 0.5 },
                    Outcome::Error(ErrorKind::Unsupported),
                ),
            ],
        },
        Scenario {
            name: "volume: accepted when the capability is present",
            applies: Applicability::WithCapability(Capability::Volume),
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(Action::SetVolume { volume: 0.5 }, Outcome::Ok),
            ],
        },
        Scenario {
            name: "text extraction: refused when the capability is absent",
            applies: Applicability::WithoutCapability(Capability::EmbeddedTextExtraction),
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(
                    Action::ExtractText {
                        track: TrackRef::Known(TrackKind::Subtitle),
                    },
                    Outcome::Error(ErrorKind::Unsupported),
                ),
            ],
        },
        Scenario {
            name: "text extraction: returns text when the capability is present",
            applies: Applicability::WithCapability(Capability::EmbeddedTextExtraction),
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(
                    Action::ExtractText {
                        track: TrackRef::Known(TrackKind::Subtitle),
                    },
                    Outcome::NonEmptyText,
                ),
            ],
        },
        Scenario {
            name: "text extraction: an unknown track is a typed error, not empty text",
            applies: Applicability::WithCapability(Capability::EmbeddedTextExtraction),
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(
                    Action::ExtractText {
                        track: TrackRef::Unknown,
                    },
                    Outcome::Error(ErrorKind::UnknownTrack),
                ),
            ],
        },
        Scenario {
            name: "subtitle injection: refused when the capability is absent",
            applies: Applicability::WithoutCapability(Capability::ExternalSubtitleInjection),
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(
                    Action::InjectSubtitle,
                    Outcome::Error(ErrorKind::Unsupported),
                ),
            ],
        },
        Scenario {
            name: "subtitle injection: accepted when the capability is present",
            applies: Applicability::WithCapability(Capability::ExternalSubtitleInjection),
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::settle(PlaybackState::Ready),
                Step::new(Action::InjectSubtitle, Outcome::Ok),
            ],
        },
    ]
}

/// What a runner needs besides the engine: a medium it can actually load, a
/// document it can inject, and what that medium is.
///
/// **This is the only place an adapter's own numbers enter the kit**, and they
/// are data, not behaviour. Nothing in [`scenarios`] names a duration, a count
/// or an id; every step that needs one names its *role* and the runner looks
/// it up here.
pub struct ContractInputs {
    pub media: MediaSource,
    pub document: SubtitleDocument,
    /// The medium's duration, or `None` for a live stream.
    pub duration_ms: Option<u64>,
    /// How many audio tracks the medium has.
    pub audio_track_count: usize,
    /// How many subtitle tracks the medium has.
    pub subtitle_track_count: usize,
    /// An audio track the medium really has.
    pub audio_track: TrackId,
    /// A subtitle track the medium really has.
    pub subtitle_track: TrackId,
    /// An id no track of **any** kind has.
    ///
    /// Ids are one space across kinds — that is what lets the contract prove an
    /// audio id is not silently accepted as a subtitle one — so an adapter
    /// whose engine numbers tracks per kind must map them into one space.
    pub unknown_track: TrackId,
    /// How far a seek may land from where it was asked to.
    ///
    /// Zero for an engine that is exact (the fake is). A real engine declares
    /// what it measured; see `platforms/macos`.
    pub seek_tolerance_ms: u64,
    /// How long [`Action::Settle`] waits before calling it a failure.
    pub settle_timeout_ms: u64,
}

impl ContractInputs {
    /// Inputs for an engine that answers instantly and exactly.
    ///
    /// The defaults are the strict ones: zero tolerance, and a settle timeout
    /// that only matters to an engine that needs one.
    pub fn new(media: MediaSource, document: SubtitleDocument) -> Self {
        Self {
            media,
            document,
            duration_ms: Some(0),
            audio_track_count: 0,
            subtitle_track_count: 0,
            audio_track: TrackId(0),
            subtitle_track: TrackId(0),
            unknown_track: TrackId(u32::MAX),
            seek_tolerance_ms: 0,
            settle_timeout_ms: 5_000,
        }
    }

    pub fn with_duration_ms(mut self, duration_ms: Option<u64>) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    pub fn with_track_counts(mut self, audio: usize, subtitle: usize) -> Self {
        self.audio_track_count = audio;
        self.subtitle_track_count = subtitle;
        self
    }

    pub fn with_tracks(mut self, audio: TrackId, subtitle: TrackId, unknown: TrackId) -> Self {
        self.audio_track = audio;
        self.subtitle_track = subtitle;
        self.unknown_track = unknown;
        self
    }

    pub fn with_seek_tolerance_ms(mut self, tolerance_ms: u64) -> Self {
        self.seek_tolerance_ms = tolerance_ms;
        self
    }

    pub fn with_settle_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.settle_timeout_ms = timeout_ms;
        self
    }

    fn track_count(&self, kind: TrackKind) -> usize {
        match kind {
            TrackKind::Audio => self.audio_track_count,
            TrackKind::Subtitle => self.subtitle_track_count,
        }
    }
}

/// Runs one scenario against an engine, returning the first failing step.
///
/// Stops at the first failure: later steps assume earlier ones succeeded, so
/// continuing would report consequences rather than causes.
pub fn run_scenario<E: PlaybackEngine>(
    engine: &mut E,
    scenario: &Scenario,
    inputs: &ContractInputs,
) -> Option<Failure> {
    // What the engine has reported since the last reset. Kept across steps
    // because an event may land while a later step is still being set up: a
    // buffer that only ever held one drain would lose it and blame the engine.
    let mut seen = Seen::default();
    for (index, step) in scenario.steps.iter().enumerate() {
        let _scope = step.inside_callback.then(CallbackScope::enter);
        if let Err(detail) = run_step(engine, step, inputs, &mut seen) {
            return Some(Failure {
                scenario: scenario.name,
                step: index,
                action: step.action,
                expected: step.expect.clone(),
                detail,
            });
        }
    }
    None
}

/// Runs every applicable scenario against a freshly built engine.
///
/// The engine is rebuilt per scenario so one scenario cannot leave state behind
/// that makes the next one pass — or fail — for the wrong reason.
pub fn run_all<E: PlaybackEngine>(
    mut build: impl FnMut() -> E,
    inputs: &ContractInputs,
) -> Vec<Failure> {
    let mut failures = Vec::new();
    for scenario in scenarios() {
        let mut engine = build();
        if !scenario.applies.applies_to(engine.capabilities()) {
            continue;
        }
        if let Some(failure) = run_scenario(&mut engine, &scenario, inputs) {
            failures.push(failure);
        }
    }
    failures
}

/// How many scenarios apply to an engine with the given capabilities.
///
/// Lets a caller assert that a run was not vacuous — a kit that skipped
/// everything would otherwise "pass".
pub fn applicable_count(capabilities: Capabilities) -> usize {
    scenarios()
        .iter()
        .filter(|scenario| scenario.applies.applies_to(capabilities))
        .count()
}

fn run_step<E: PlaybackEngine>(
    engine: &mut E,
    step: &Step,
    inputs: &ContractInputs,
    seen: &mut Seen,
) -> Result<(), String> {
    let expect = &step.expect;
    match step.action {
        Action::Load => check_unit(engine.load(&inputs.media), expect),
        Action::Play => check_unit(engine.play(), expect),
        Action::Pause => check_unit(engine.pause(), expect),
        Action::Stop => check_unit(engine.stop(), expect),
        Action::Seek { to_ms } => check_unit(engine.seek(Duration::from_millis(to_ms)), expect),
        Action::SeekRelative { delta_ms } => check_unit(engine.seek_relative(delta_ms), expect),
        Action::Shutdown => check_unit(engine.shutdown(), expect),
        Action::SetRate { rate } => check_unit(engine.set_rate(rate), expect),
        Action::SetVolume { volume } => check_unit(engine.set_volume(volume), expect),
        Action::InjectSubtitle => check_unit(engine.inject_subtitle(&inputs.document), expect),
        Action::SelectTrack { kind, track } => check_unit(
            engine.select_track(kind, track.map(|reference| reference.resolve(inputs))),
            expect,
        ),
        Action::Settle { until } => settle(engine, until, inputs),
        Action::AwaitEvent { shape } => {
            await_events(engine, std::slice::from_ref(&shape), inputs, seen)
        }
        Action::AwaitSeekLanding { near_ms } => {
            await_events(engine, &[EventShape::SeekCompleted], inputs, seen)?;
            let Some(landed) = seen.seek_landings.last().copied() else {
                return Err("a SeekCompleted arrived but carried no position to check".to_string());
            };
            let drift = landed.abs_diff(near_ms);
            if drift <= inputs.seek_tolerance_ms {
                Ok(())
            } else {
                Err(format!(
                    "the seek was answered at {landed} ms, expected {near_ms} ms \
                     ({drift} ms away, tolerance {} ms)",
                    inputs.seek_tolerance_ms
                ))
            }
        }
        Action::Position => match (engine.position(), expect) {
            (Ok(position), Outcome::PositionNear(expected)) => {
                let actual = position.as_millis() as u64;
                let drift = actual.abs_diff(*expected);
                if drift <= inputs.seek_tolerance_ms {
                    Ok(())
                } else {
                    Err(format!(
                        "position was {actual} ms, expected {expected} ms \
                         ({drift} ms away, tolerance {} ms)",
                        inputs.seek_tolerance_ms
                    ))
                }
            }
            (result, expect) => check_unit(result.map(|_| ()), expect),
        },
        Action::Duration => match (engine.duration(), expect) {
            (Ok(duration), Outcome::FixtureDuration) => {
                let actual = duration.map(|d| d.as_millis() as u64);
                let expected = inputs.duration_ms;
                match (actual, expected) {
                    (Some(actual), Some(expected))
                        if actual.abs_diff(expected) <= inputs.seek_tolerance_ms =>
                    {
                        Ok(())
                    }
                    (None, None) => Ok(()),
                    _ => Err(format!("duration was {actual:?}, expected {expected:?}")),
                }
            }
            (result, expect) => check_unit(result.map(|_| ()), expect),
        },
        Action::State => match expect {
            Outcome::State(expected) => {
                let actual = engine.state();
                if actual == *expected {
                    Ok(())
                } else {
                    Err(format!("state was {actual}, expected {expected}"))
                }
            }
            other => Err(format!("state cannot satisfy {other:?}")),
        },
        Action::Tracks { kind } => match (engine.tracks(kind), expect) {
            (Ok(tracks), Outcome::FixtureTrackCount(of_kind)) => {
                let expected = inputs.track_count(*of_kind);
                if tracks.len() == expected {
                    Ok(())
                } else {
                    Err(format!(
                        "{} {kind} tracks, expected {expected}",
                        tracks.len()
                    ))
                }
            }
            (result, expect) => check_unit(result.map(|_| ()), expect),
        },
        Action::SelectedTrack { kind } => match (engine.selected_track(kind), expect) {
            (Ok(selected), Outcome::SelectedTrack(expected)) => {
                let expected = expected.map(|reference| reference.resolve(inputs));
                if selected == expected {
                    Ok(())
                } else {
                    Err(format!("selected {selected:?}, expected {expected:?}"))
                }
            }
            (result, expect) => check_unit(result.map(|_| ()), expect),
        },
        Action::ExtractText { track } => match (engine.extract_text(track.resolve(inputs)), expect)
        {
            (Ok(text), Outcome::NonEmptyText) => {
                if text.is_empty() {
                    // Never print the text itself: it is dialogue (K23 #4).
                    Err("extracted text was empty".to_string())
                } else {
                    Ok(())
                }
            }
            (result, expect) => check_unit(result.map(|_| ()), expect),
        },
        Action::DrainEvents => match expect {
            // A bare drain is a reset: everything before this point stops
            // counting, so the next expectation describes only what follows.
            Outcome::Ok => {
                collect(engine, seen);
                seen.clear();
                Ok(())
            }
            Outcome::Events(expected) => await_events(engine, expected, inputs, seen),
            other => Err(format!("draining events cannot satisfy {other:?}")),
        },
    }
}

/// Moves whatever the engine has pending into the running buffer.
/// What the kit has watched go past, in both the forms it judges.
///
/// [`EventShape`] deliberately drops payloads that differ legitimately per
/// engine, which is right for order and for kind — and blind for exactly one
/// value. A seek's landing position is *not* engine-private: the fixture
/// already states how far off it may be
/// ([`ContractInputs::seek_tolerance_ms`]), so the kit can and must judge it.
/// NEN-051 is why it now does — an adapter answered a seek with `0 ms` and
/// every shape-level assertion still passed.
#[derive(Debug, Default)]
struct Seen {
    shapes: Vec<EventShape>,
    /// The position carried by every `SeekCompleted`, in order.
    seek_landings: Vec<u64>,
}

impl Seen {
    fn clear(&mut self) {
        self.shapes.clear();
        self.seek_landings.clear();
    }
}

fn collect<E: PlaybackEngine>(engine: &mut E, seen: &mut Seen) {
    for event in engine.events().drain().iter() {
        if let PlaybackEvent::SeekCompleted { position } = event {
            seen.seek_landings.push(position.as_millis() as u64);
        }
        seen.shapes.push(EventShape::of(event));
    }
}

/// Polls until `expected` is satisfied, or the fixture's timeout runs out.
fn await_events<E: PlaybackEngine>(
    engine: &mut E,
    expected: &[EventShape],
    inputs: &ContractInputs,
    seen: &mut Seen,
) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_millis(inputs.settle_timeout_ms);
    loop {
        collect(engine, seen);
        match check_events(&seen.shapes, expected) {
            Ok(()) => return Ok(()),
            // An unasked failure is not going to become acceptable by waiting.
            Err(detail) if detail.contains("unasked") => return Err(detail),
            Err(detail) => {
                if Instant::now() >= deadline {
                    return Err(format!("{detail} (waited {} ms)", inputs.settle_timeout_ms));
                }
            }
        }
        std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
    }
}

/// Polls until the engine reaches `until`, or the fixture's timeout runs out.
///
/// Polling rather than waiting on an event: `state()` is the port's own answer
/// and every adapter has it, whereas the event that *causes* a state is the
/// adapter's business. An engine already in the state costs one call — which is
/// what the fake pays.
fn settle<E: PlaybackEngine>(
    engine: &mut E,
    until: PlaybackState,
    inputs: &ContractInputs,
) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_millis(inputs.settle_timeout_ms);
    loop {
        let state = engine.state();
        if state == until {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "never reached {until} within {} ms (stuck at {state})",
                inputs.settle_timeout_ms
            ));
        }
        std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
    }
}

/// How long between two `state()` polls while settling.
const POLL_INTERVAL_MS: u64 = 5;

/// Matches `expected` as a **subsequence** of `actual`, in order.
///
/// Extras are allowed because a real engine reports more than the contract
/// requires — but two shapes are refused when unasked. A `Failed` or an
/// `EventsLost` the scenario did not list is a genuine defect, and the whole
/// point of allowing extras is lost if it can hide one.
fn check_events(actual: &[EventShape], expected: &[EventShape]) -> Result<(), String> {
    for unexpected in [EventShape::Failed, EventShape::EventsLost] {
        if actual.contains(&unexpected) && !expected.contains(&unexpected) {
            return Err(format!(
                "an unasked {unexpected:?} was reported; events were {actual:?}"
            ));
        }
    }
    let mut remaining = actual.iter();
    for shape in expected {
        if !remaining.any(|candidate| candidate == shape) {
            return Err(format!(
                "{shape:?} never arrived (in order); events were {actual:?}, \
                 expected at least {expected:?}"
            ));
        }
    }
    Ok(())
}

fn check_unit<T>(result: Result<T, PlaybackError>, expect: &Outcome) -> Result<(), String> {
    match (result, expect) {
        (Ok(_), Outcome::Ok) => Ok(()),
        (Ok(_), Outcome::Error(kind)) => Err(format!("succeeded, expected {kind:?}")),
        (Err(error), Outcome::Error(kind)) => {
            let actual = ErrorKind::of(&error);
            if actual == *kind {
                Ok(())
            } else {
                Err(format!("failed with {actual:?}, expected {kind:?}"))
            }
        }
        (Err(error), Outcome::Ok) => Err(format!("failed with {error:?}, expected success")),
        (_, other) => Err(format!("this action cannot satisfy {other:?}")),
    }
}
