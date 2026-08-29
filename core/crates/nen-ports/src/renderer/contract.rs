//! The shared contract kit every `SubtitleRenderer` adapter must pass.
//!
//! ADR-0011 Karar 4's rule, applied to a second port: **the scenarios are
//! data, not code.** A scenario is a list of steps, each an [`Action`] paired
//! with the [`Outcome`] the contract requires — no closures, no
//! adapter-specific assertions. The engine-native adapter and M7's overlay run
//! this same list, so "both renderers behave the same" is a fact rather than a
//! hope.
//!
//! `docs/testing-strategy.md` names the evidence this produces: "aynı kitin
//! fake + gerçek adapter'da geçmesi".
//!
//! # What this kit deliberately does not check
//!
//! **That the right cue is on screen at a given moment.** A renderer does not
//! own the playhead, so a kit at this level cannot move time and would have to
//! invent a hook for every adapter to seek with. That question is asked one
//! layer down, where the playhead lives — the playback contract's "an injected
//! document is drawn at its own moment" scenario — and again against a real
//! engine in NEN-027's parity test. What is left here is what a renderer can
//! answer on its own: showing, replacing, clearing, and refusing.

use super::capability::{Capabilities, Capability};
use super::error::RenderError;
use super::surface::SubtitleRenderer;
use nen_domain::subtitle::SubtitleDocument;

/// One port operation, as data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Show,
    Clear,
    RenderedText,
}

/// The variant of a [`RenderError`], without its payload.
///
/// Scenarios assert the *kind* of refusal, not a surface's private numbers: a
/// `SurfaceFailure` code is meaningless across adapters, and a scenario that
/// pinned one could only ever pass on a single renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Unsupported,
    Unavailable,
    NoMedia,
    ShutDown,
    ReentrantCall,
    SurfaceFailure,
}

impl ErrorKind {
    fn of(error: &RenderError) -> Self {
        match error {
            RenderError::Unsupported { .. } => Self::Unsupported,
            RenderError::Unavailable { .. } => Self::Unavailable,
            RenderError::NoMedia { .. } => Self::NoMedia,
            RenderError::ShutDown { .. } => Self::ShutDown,
            RenderError::ReentrantCall { .. } => Self::ReentrantCall,
            RenderError::SurfaceFailure { .. } => Self::SurfaceFailure,
        }
    }
}

/// What the contract requires an action to produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Succeeded; the returned value is not what this step is about.
    Ok,
    Error(ErrorKind),
    /// Something is on screen.
    ///
    /// **What** is on screen is never inspected: it is dialogue (K23 #4), and
    /// whether it is the *right* dialogue is checked where the playhead is
    /// known, not here.
    Drawing,
    /// Nothing is on screen.
    Blank,
}

/// One step of a scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    pub action: Action,
    pub expect: Outcome,
}

impl Step {
    pub fn new(action: Action, expect: Outcome) -> Self {
        Self { action, expect }
    }
}

/// When a scenario applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applicability {
    /// Base behaviour — every adapter, always.
    Always,
    /// Only for renderers that declare the capability.
    WithCapability(Capability),
    /// Only for renderers that do not — this is where the typed refusal is
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    pub name: &'static str,
    pub applies: Applicability,
    pub steps: Vec<Step>,
}

/// A step that did not produce what the contract requires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub scenario: &'static str,
    pub step: usize,
    pub action: Action,
    pub detail: String,
}

impl Failure {
    /// One rendered line, ready to cross a boundary that has no Rust types.
    pub fn render(&self) -> String {
        format!(
            "{} — step {} ({:?}): {}",
            self.scenario, self.step, self.action, self.detail
        )
    }
}

/// What a runner needs besides the renderer: a document it can show.
///
/// **The surface's playhead must already be inside the document's first cue.**
/// A renderer cannot move time, so the kit cannot put it there; a caller that
/// leaves the surface outside every cue will see the `Drawing` step fail, and
/// correctly — a renderer showing a document that is on screen nowhere is not
/// something this kit can distinguish from one that failed to draw.
pub struct RenderInputs {
    pub document: SubtitleDocument,
}

impl RenderInputs {
    pub fn new(document: SubtitleDocument) -> Self {
        Self { document }
    }
}

/// Every scenario in the kit.
pub fn scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "show: a document is accepted",
            applies: Applicability::Always,
            steps: vec![Step::new(Action::Show, Outcome::Ok)],
        },
        Scenario {
            name: "show: the second document replaces the first, it does not stack",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Show, Outcome::Ok),
                Step::new(Action::Show, Outcome::Ok),
            ],
        },
        Scenario {
            name: "clear: accepted with nothing showing",
            applies: Applicability::Always,
            steps: vec![Step::new(Action::Clear, Outcome::Ok)],
        },
        Scenario {
            name: "clear: accepted twice in a row",
            applies: Applicability::Always,
            steps: vec![
                Step::new(Action::Show, Outcome::Ok),
                Step::new(Action::Clear, Outcome::Ok),
                Step::new(Action::Clear, Outcome::Ok),
            ],
        },
        Scenario {
            name: "rendered text: refused when the capability is absent",
            applies: Applicability::WithoutCapability(Capability::ObservedText),
            steps: vec![
                Step::new(Action::Show, Outcome::Ok),
                Step::new(Action::RenderedText, Outcome::Error(ErrorKind::Unsupported)),
            ],
        },
        Scenario {
            name: "rendered text: nothing is drawn before a document is shown",
            applies: Applicability::WithCapability(Capability::ObservedText),
            steps: vec![Step::new(Action::RenderedText, Outcome::Blank)],
        },
        Scenario {
            name: "rendered text: a shown document is on screen",
            applies: Applicability::WithCapability(Capability::ObservedText),
            steps: vec![
                Step::new(Action::Show, Outcome::Ok),
                Step::new(Action::RenderedText, Outcome::Drawing),
            ],
        },
        Scenario {
            name: "rendered text: clearing takes it off screen",
            applies: Applicability::WithCapability(Capability::ObservedText),
            steps: vec![
                Step::new(Action::Show, Outcome::Ok),
                Step::new(Action::Clear, Outcome::Ok),
                Step::new(Action::RenderedText, Outcome::Blank),
            ],
        },
    ]
}

/// How many scenarios apply to a renderer with the given capabilities.
///
/// Lets a caller assert that a run was not vacuous — a kit that skipped
/// everything would otherwise "pass".
pub fn applicable_count(capabilities: Capabilities) -> usize {
    scenarios()
        .iter()
        .filter(|scenario| scenario.applies.applies_to(capabilities))
        .count()
}

/// Runs one scenario, returning the first failing step.
///
/// Stops at the first failure: later steps assume earlier ones succeeded, so
/// continuing would report consequences rather than causes.
pub fn run_scenario<R: SubtitleRenderer>(
    renderer: &mut R,
    scenario: &Scenario,
    inputs: &RenderInputs,
) -> Option<Failure> {
    for (index, step) in scenario.steps.iter().enumerate() {
        if let Err(detail) = run_step(renderer, step, inputs) {
            return Some(Failure {
                scenario: scenario.name,
                step: index,
                action: step.action,
                detail,
            });
        }
    }
    None
}

/// Runs every applicable scenario against a freshly built renderer each time.
///
/// Rebuilt per scenario so one cannot leave state behind that makes the next
/// pass — or fail — for the wrong reason.
pub fn run_all<R: SubtitleRenderer>(build: impl Fn() -> R, inputs: &RenderInputs) -> Vec<Failure> {
    let mut failures = Vec::new();
    for scenario in scenarios() {
        let mut renderer = build();
        if !scenario.applies.applies_to(renderer.capabilities()) {
            continue;
        }
        if let Some(failure) = run_scenario(&mut renderer, &scenario, inputs) {
            failures.push(failure);
        }
    }
    failures
}

fn run_step<R: SubtitleRenderer>(
    renderer: &mut R,
    step: &Step,
    inputs: &RenderInputs,
) -> Result<(), String> {
    let expect = step.expect;
    match step.action {
        Action::Show => check_unit(renderer.show(&inputs.document), expect),
        Action::Clear => check_unit(renderer.clear(), expect),
        Action::RenderedText => match (renderer.rendered_text(), expect) {
            (Ok(Some(_)), Outcome::Drawing) | (Ok(None), Outcome::Blank) => Ok(()),
            (Ok(Some(_)), Outcome::Blank) => Err("something is on screen".to_string()),
            (Ok(None), Outcome::Drawing) => Err("nothing is on screen".to_string()),
            (result, expect) => check_unit(result, expect),
        },
    }
}

fn check_unit<T>(result: Result<T, RenderError>, expect: Outcome) -> Result<(), String> {
    match (result, expect) {
        (Ok(_), Outcome::Ok) => Ok(()),
        (Ok(_), Outcome::Error(kind)) => Err(format!("succeeded, expected {kind:?}")),
        (Err(error), Outcome::Error(kind)) => {
            let actual = ErrorKind::of(&error);
            if actual == kind {
                Ok(())
            } else {
                Err(format!("failed with {actual:?}, expected {kind:?}"))
            }
        }
        (Err(error), Outcome::Ok) => Err(format!("failed with {error:?}, expected success")),
        (_, other) => Err(format!("this action cannot satisfy {other:?}")),
    }
}
