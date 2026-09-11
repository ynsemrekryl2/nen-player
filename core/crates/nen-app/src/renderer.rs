//! The engine-native subtitle renderer (ADR-0013 Karar 1 and 2).
//!
//! The port is `nen_ports::renderer`; this is its first adapter. It draws
//! nothing itself — it hands the document to the playback engine and lets the
//! engine draw, which is what product-spec §14 allows once engine-native
//! support is known to be reliable. NEN-027 measured that it is
//! (`evidence/M3/NEN-027-injection-measurement.md`).
//!
//! # Why the adapter is here and not in a platform
//!
//! Because there is nothing platform-specific left to write. The engine
//! already accepts a document
//! ([`PlaybackEngine::inject_subtitle`](nen_ports::playback::PlaybackEngine::inject_subtitle))
//! and already reports what it draws, both behind capabilities. An adapter in
//! Swift and again in Kotlin would be the same twenty lines, twice, with two
//! chances to differ. Here it is written once and every platform that
//! implements the two engine methods gets a renderer for free.
//!
//! # What the state is for
//!
//! [`RendererState`] keeps the document that was shown. Not as a cache of the
//! screen — the engine owns the screen — but because the core is the only
//! place that can answer **what should be on screen** at a moment, which is
//! how "the right cue is displayed" gets checked against something. That
//! answer is `nen_subtitle::CueIndex`'s (NEN-017), asked here and decided
//! nowhere else.

use nen_domain::subtitle::SubtitleDocument;
use nen_ports::playback::{Capability as EngineCapability, PlaybackEngine, PlaybackError};
use nen_ports::playback::{Operation as EngineOperation, TrackKind};
use nen_ports::renderer::error::Operation;
use nen_ports::renderer::{
    Capabilities, Capability, RenderError, SubtitleRenderer, MAX_BOTTOM_INSET,
};
use nen_subtitle::index::CueIndex;
use std::fmt;

/// What the renderer remembers between calls.
///
/// Lives with the session rather than inside the adapter because the adapter
/// borrows the engine for the length of one call and cannot outlive it.
#[derive(Default)]
pub struct RendererState {
    showing: Option<SubtitleDocument>,
    bottom_inset: f32,
}

impl RendererState {
    pub fn new() -> Self {
        Self::default()
    }

    /// The text that should be on screen at a moment, if any.
    ///
    /// `None` when nothing is showing or the moment falls in a gap — a gap is
    /// not an error, and the empty answer is the correct one to compare
    /// against an engine drawing nothing.
    ///
    /// Cues active at the same millisecond are joined in document order, on
    /// one line each: overlapping cues are legitimate SRT (NEN-013) and a
    /// renderer that dropped one would answer a question nobody asked.
    ///
    /// **Security:** the result is dialogue (K23 #4). Displayable and
    /// comparable, never loggable.
    pub fn expected_at(&self, at_ms: u32) -> Option<String> {
        let document = self.showing.as_ref()?;
        let index = CueIndex::build(document);
        let lines: Vec<String> = index
            .active_cues(at_ms)
            .into_iter()
            .map(|cue| cue.lines().join("\n"))
            .collect();
        (!lines.is_empty()).then(|| lines.join("\n"))
    }

    /// Whether a document is showing at all.
    pub fn is_showing(&self) -> bool {
        self.showing.is_some()
    }

    /// The share of the surface the shell last said its chrome covers.
    ///
    /// Kept across [`Self::forget`] on purpose: the chrome does not move when
    /// the medium changes, so a new document arrives on a surface that is
    /// covered exactly as much as the old one was.
    pub fn bottom_inset(&self) -> f32 {
        self.bottom_inset
    }

    /// Forgets the document. Called when a new medium is loaded: the old
    /// document's timeline says nothing about the new medium's moments.
    pub fn forget(&mut self) {
        self.showing = None;
    }
}

impl fmt::Debug for RendererState {
    /// Prints whether something is showing and how big it is — never the cues.
    ///
    /// The document is dialogue (K23 #4). `tests/guard_renderer_state_debug.rs`
    /// holds this line with a deliberately derived twin.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RendererState")
            .field("showing", &self.showing.is_some())
            .field(
                "cue_count",
                &self.showing.as_ref().map(SubtitleDocument::len),
            )
            .field("bottom_inset", &self.bottom_inset)
            .finish()
    }
}

/// A renderer that draws by handing the document to the engine.
///
/// Borrows both halves for one call: the engine belongs to the session and the
/// state belongs next to it, so this type is built where it is used and never
/// stored.
pub struct EngineNativeRenderer<'a> {
    engine: &'a mut dyn PlaybackEngine,
    state: &'a mut RendererState,
}

impl<'a> EngineNativeRenderer<'a> {
    pub fn new(engine: &'a mut dyn PlaybackEngine, state: &'a mut RendererState) -> Self {
        Self { engine, state }
    }
}

impl SubtitleRenderer for EngineNativeRenderer<'_> {
    /// What this renderer can do is what the engine under it can do.
    ///
    /// Derived rather than declared: an adapter that announced `ObservedText`
    /// over an engine that cannot report would be a promise the pairing cannot
    /// keep, and the contract kit would catch it — after shipping a renderer
    /// that lies.
    fn capabilities(&self) -> Capabilities {
        let mut capabilities = Capabilities::NONE;
        if self
            .engine
            .capabilities()
            .contains(EngineCapability::RenderedTextObservation)
        {
            capabilities = capabilities.with(Capability::ObservedText);
        }
        capabilities
    }

    fn show(&mut self, document: &SubtitleDocument) -> Result<(), RenderError> {
        self.engine
            .inject_subtitle(document)
            .map_err(|error| render_error(error, Operation::Show))?;
        self.state.showing = Some(document.clone());
        Ok(())
    }

    /// Deselects the engine's subtitle, whatever it was drawing.
    ///
    /// Deliberately not "remove the document we injected": §8's `Kapalı` means
    /// *nothing* on screen, and the thing on screen may be an embedded track
    /// this renderer never touched.
    /// Refuses an impossible inset before the engine ever hears about it.
    ///
    /// The range check lives here rather than in the engine because the unit
    /// is this port's (ADR-0037 Karar 2): an engine is handed a value that is
    /// already in range and has nothing left to judge.
    fn set_bottom_inset(&mut self, fraction: f32) -> Result<(), RenderError> {
        if !(0.0..=MAX_BOTTOM_INSET).contains(&fraction) {
            return Err(RenderError::InsetOutOfRange {
                operation: Operation::SetBottomInset,
            });
        }
        self.engine
            .set_subtitle_bottom_inset(fraction)
            .map_err(|error| render_error(error, Operation::SetBottomInset))?;
        self.state.bottom_inset = fraction;
        Ok(())
    }

    fn clear(&mut self) -> Result<(), RenderError> {
        self.engine
            .select_track(TrackKind::Subtitle, None)
            .map_err(|error| render_error(error, Operation::Clear))?;
        self.state.forget();
        Ok(())
    }

    fn rendered_text(&self) -> Result<Option<String>, RenderError> {
        self.engine
            .rendered_subtitle_text()
            .map_err(|error| render_error(error, Operation::RenderedText))
    }
}

/// Translates the engine's refusal into the renderer's.
///
/// The two ports have their own error vocabularies on purpose — a renderer
/// that spoke `PlaybackError` would be a renderer that assumed an engine — so
/// the delegating adapter is where the two meet. Written out rather than
/// derived so each mapping can be argued: an engine that does not accept a
/// document does not make the renderer *incapable of answering something*, it
/// makes this pairing unable to draw at all.
fn render_error(error: PlaybackError, operation: Operation) -> RenderError {
    match error {
        PlaybackError::Unsupported {
            capability: EngineCapability::RenderedTextObservation,
            ..
        } => RenderError::Unsupported {
            operation,
            capability: Capability::ObservedText,
        },
        PlaybackError::Unsupported { .. } => RenderError::Unavailable { operation },
        PlaybackError::NotLoaded { .. } => RenderError::NoMedia { operation },
        PlaybackError::ShutDown { .. } => RenderError::ShutDown { operation },
        PlaybackError::ReentrantCall { .. } => RenderError::ReentrantCall { operation },
        PlaybackError::EngineFailure { code } => RenderError::SurfaceFailure { operation, code },
        // Nothing else can reach a render call — there is no track id, no rate
        // and no locator on this path — so they collapse into the honest
        // catch-all rather than into a variant that would describe them wrongly.
        // `InsetOutOfRange` is here rather than mapped straight through
        // because it cannot reach this function: the range is checked above,
        // before the engine is called, so an engine answering it would be
        // answering a question nobody asked.
        PlaybackError::UnknownTrack { .. }
        | PlaybackError::TrackCarriesNoText
        | PlaybackError::RateOutOfRange { .. }
        | PlaybackError::InsetOutOfRange { .. }
        | PlaybackError::LoadFailed { .. } => RenderError::SurfaceFailure { operation, code: 0 },
    }
}

/// Translates the renderer's refusal back into the one the shell already
/// understands.
///
/// The shell's error vocabulary is playback's: ADR-0031's three error classes,
/// the FFI enum and the transport bar's message all speak it. Giving the shell
/// a second vocabulary for the same three classes would buy nothing and cost a
/// translation on every platform, so the translation happens once, here.
pub(crate) fn playback_error(error: RenderError) -> PlaybackError {
    let operation = match error.operation() {
        Operation::Show => EngineOperation::InjectSubtitle,
        Operation::Clear => EngineOperation::SelectTrack,
        Operation::RenderedText => EngineOperation::RenderedText,
        Operation::SetBottomInset => EngineOperation::SetSubtitleBottomInset,
    };
    match error {
        RenderError::Unsupported { .. } => PlaybackError::Unsupported {
            operation,
            capability: EngineCapability::RenderedTextObservation,
        },
        RenderError::Unavailable { .. } => PlaybackError::Unsupported {
            operation,
            capability: EngineCapability::ExternalSubtitleInjection,
        },
        RenderError::NoMedia { .. } => PlaybackError::NotLoaded { operation },
        RenderError::ShutDown { .. } => PlaybackError::ShutDown { operation },
        RenderError::ReentrantCall { .. } => PlaybackError::ReentrantCall { operation },
        RenderError::InsetOutOfRange { .. } => PlaybackError::InsetOutOfRange { operation },
        RenderError::SurfaceFailure { code, .. } => PlaybackError::EngineFailure { code },
    }
}
