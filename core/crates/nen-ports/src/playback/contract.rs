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

use super::capability::{Capabilities, Capability};
use super::engine::PlaybackEngine;
use super::error::PlaybackError;
use super::event::{CallbackScope, PlaybackEvent, PlaybackState};
use super::media::MediaSource;
use super::track::{TrackId, TrackKind};
use nen_domain::subtitle::SubtitleDocument;
use std::fmt;
use std::time::Duration;

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
        track: Option<TrackId>,
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
        track: TrackId,
    },
    InjectSubtitle,
    Shutdown,
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
    PositionMs(u64),
    /// `None` asserts "this medium reports no duration" (a live stream).
    DurationMs(Option<u64>),
    TrackCount(usize),
    SelectedTrack(Option<TrackId>),
    /// Exactly these event shapes, in this order.
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
                Step::new(Action::State, Outcome::State(PlaybackState::Ready)),
                Step::new(Action::Play, Outcome::Ok),
                Step::new(Action::State, Outcome::State(PlaybackState::Playing)),
                Step::new(Action::Pause, Outcome::Ok),
                Step::new(Action::State, Outcome::State(PlaybackState::Paused)),
            ],
        },
        Scenario {
            name: "loading reports buffering before ready, in order",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
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
                Step::new(Action::DrainEvents, Outcome::Ok),
                Step::new(Action::Seek { to_ms: 5_000 }, Outcome::Ok),
                Step::new(Action::Position, Outcome::PositionMs(5_000)),
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
                Step::new(Action::Seek { to_ms: 10_000 }, Outcome::Ok),
                Step::new(Action::SeekRelative { delta_ms: 2_000 }, Outcome::Ok),
                Step::new(Action::Position, Outcome::PositionMs(12_000)),
                Step::new(Action::SeekRelative { delta_ms: -50_000 }, Outcome::Ok),
                Step::new(Action::Position, Outcome::PositionMs(0)),
            ],
        },
        Scenario {
            name: "seeking past the end ends playback instead of failing",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::new(Action::Play, Outcome::Ok),
                Step::new(Action::DrainEvents, Outcome::Ok),
                Step::new(
                    Action::Seek {
                        to_ms: u64::from(u32::MAX),
                    },
                    Outcome::Ok,
                ),
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
                Step::new(
                    Action::Tracks {
                        kind: TrackKind::Subtitle,
                    },
                    Outcome::TrackCount(2),
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
                        track: Some(TrackId(1)),
                    },
                    Outcome::Ok,
                ),
                Step::new(
                    Action::SelectedTrack {
                        kind: TrackKind::Subtitle,
                    },
                    Outcome::SelectedTrack(Some(TrackId(1))),
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
                Step::new(
                    Action::SelectTrack {
                        kind: TrackKind::Subtitle,
                        track: Some(TrackId(9_999)),
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
                Step::new(
                    Action::SelectTrack {
                        kind: TrackKind::Audio,
                        track: Some(TrackId(1)),
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
                Step::new(Action::Play, Outcome::Ok),
                Step::new(Action::Stop, Outcome::Ok),
                Step::new(Action::State, Outcome::State(PlaybackState::Idle)),
                Step::new(Action::Play, Outcome::Error(ErrorKind::NotLoaded)),
                Step::new(Action::Load, Outcome::Ok),
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
                Step::new(Action::Duration, Outcome::DurationMs(Some(120_000))),
            ],
        },
        Scenario {
            name: "position events coalesce while critical events keep their order",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::new(Action::DrainEvents, Outcome::Ok),
                Step::new(Action::Play, Outcome::Ok),
                Step::new(Action::Seek { to_ms: 1_000 }, Outcome::Ok),
                Step::new(Action::Seek { to_ms: 2_000 }, Outcome::Ok),
                // Two seeks each produced a position update; only the newest
                // position survives, while both completions do.
                Step::new(
                    Action::DrainEvents,
                    Outcome::Events(vec![
                        EventShape::StateChanged(PlaybackState::Playing),
                        EventShape::SeekCompleted,
                        EventShape::SeekCompleted,
                        EventShape::PositionChanged,
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
        steps: std::iter::once(Step::new(Action::Load, Outcome::Ok))
            .chain(actions.into_iter().map(|action| {
                Step::from_callback(action, Outcome::Error(ErrorKind::ReentrantCall))
            }))
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
                Step::new(Action::SetRate { rate: 1.5 }, Outcome::Ok),
            ],
        },
        Scenario {
            name: "volume: refused when the capability is absent",
            applies: Applicability::WithoutCapability(Capability::Volume),
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
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
                Step::new(Action::SetVolume { volume: 0.5 }, Outcome::Ok),
            ],
        },
        Scenario {
            name: "text extraction: refused when the capability is absent",
            applies: Applicability::WithoutCapability(Capability::EmbeddedTextExtraction),
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::new(
                    Action::ExtractText { track: TrackId(0) },
                    Outcome::Error(ErrorKind::Unsupported),
                ),
            ],
        },
        Scenario {
            name: "text extraction: returns text when the capability is present",
            applies: Applicability::WithCapability(Capability::EmbeddedTextExtraction),
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::new(
                    Action::ExtractText { track: TrackId(0) },
                    Outcome::NonEmptyText,
                ),
            ],
        },
        Scenario {
            name: "text extraction: an unknown track is a typed error, not empty text",
            applies: Applicability::WithCapability(Capability::EmbeddedTextExtraction),
            steps: vec![
                Step::new(Action::Load, Outcome::Ok),
                Step::new(
                    Action::ExtractText {
                        track: TrackId(9_999),
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
                Step::new(Action::InjectSubtitle, Outcome::Ok),
            ],
        },
    ]
}

/// What a runner needs besides the engine: a medium it can actually load, and
/// a document it can inject.
///
/// Supplied by the caller because a fake medium and a real one differ — this is
/// the only place an adapter's own fixtures enter the kit, and it is data, not
/// behaviour.
pub struct ContractInputs {
    pub media: MediaSource,
    pub document: SubtitleDocument,
}

impl ContractInputs {
    pub fn new(media: MediaSource, document: SubtitleDocument) -> Self {
        Self { media, document }
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
    for (index, step) in scenario.steps.iter().enumerate() {
        let _scope = step.inside_callback.then(CallbackScope::enter);
        if let Err(detail) = run_step(engine, step, inputs) {
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
        Action::SelectTrack { kind, track } => check_unit(engine.select_track(kind, track), expect),
        Action::Position => match (engine.position(), expect) {
            (Ok(position), Outcome::PositionMs(expected)) => {
                let actual = position.as_millis() as u64;
                if actual == *expected {
                    Ok(())
                } else {
                    Err(format!("position was {actual} ms, expected {expected} ms"))
                }
            }
            (result, expect) => check_unit(result.map(|_| ()), expect),
        },
        Action::Duration => match (engine.duration(), expect) {
            (Ok(duration), Outcome::DurationMs(expected)) => {
                let actual = duration.map(|d| d.as_millis() as u64);
                if actual == *expected {
                    Ok(())
                } else {
                    Err(format!("duration was {actual:?}, expected {expected:?}"))
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
            (Ok(tracks), Outcome::TrackCount(expected)) => {
                if tracks.len() == *expected {
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
                if selected == *expected {
                    Ok(())
                } else {
                    Err(format!("selected {selected:?}, expected {expected:?}"))
                }
            }
            (result, expect) => check_unit(result.map(|_| ()), expect),
        },
        Action::ExtractText { track } => match (engine.extract_text(track), expect) {
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
        Action::DrainEvents => {
            let drained = engine.events().drain();
            match expect {
                Outcome::Ok => Ok(()),
                Outcome::Events(expected) => {
                    let actual: Vec<EventShape> = drained.iter().map(EventShape::of).collect();
                    if actual == *expected {
                        Ok(())
                    } else {
                        Err(format!("events were {actual:?}, expected {expected:?}"))
                    }
                }
                other => Err(format!("draining events cannot satisfy {other:?}")),
            }
        }
    }
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
