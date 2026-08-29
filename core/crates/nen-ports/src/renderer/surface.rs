//! The `SubtitleRenderer` port itself.
//!
//! `docs/product-spec.md` §14 asks for two things and this trait is both of
//! them: subtitle drawing is **its own port**, and the UI does not know which
//! implementation is behind it. ADR-0013 chose the first adapter — the engine
//! draws, and the core hands it the document — without letting that choice
//! reach a single call site.
//!
//! The port is deliberately small. It says what a renderer must do (show a
//! document, take it off screen, keep clear of the part of the surface the
//! shell has covered), what only some can do (say what is on screen right
//! now), and nothing at all about *how* — no styling, no font, no sync offset.
//! Those are M7's questions and would be answered here by guessing.
//!
//! The one thing that looks like positioning is not (ADR-0037): a bottom inset
//! says which part of the surface is **not visible**, which is a fact about
//! the shell's layout rather than a preference about how a subtitle should
//! look. NEN-066 measured what its absence costs — libmpv drew every line
//! correctly and the transport bar covered all of them.
//!
//! # What this port does not own
//!
//! - **The playhead.** A renderer draws what a surface is playing; it does not
//!   move time. That is [`PlaybackEngine`](crate::playback::PlaybackEngine)'s.
//! - **Which cue belongs to a moment.** `nen_subtitle::CueIndex` answers that
//!   once, for every platform, and a renderer that re-decided it would be a
//!   second opinion next to the tests that pin the first.
//! - **Which source is showing.** The catalog and the session know; a renderer
//!   is handed a document and never asks where it came from.

use super::capability::{Capabilities, Capability};
use super::error::{Operation, RenderError};
use nen_domain::subtitle::SubtitleDocument;

/// The largest share of the surface a shell may declare covered.
///
/// A shell whose own chrome hides more than half the surface is describing a
/// mistake, not a layout: what it asks for would put the subtitle above the
/// middle of the picture. Measured need is far below it — NEN-066's transport
/// bar covers `134 / 360 = 0.372` of the shortest window the player allows.
pub const MAX_BOTTOM_INSET: f32 = 0.5;

/// Something that can put a subtitle document on screen.
///
/// Implemented once per drawing strategy rather than once per platform: M3's
/// adapter delegates to the playback engine (ADR-0013 Karar 1), and M7's
/// overlay will draw itself. Both wear this trait, so the session that calls
/// them changes for neither.
pub trait SubtitleRenderer {
    /// The optional capabilities this renderer provides.
    ///
    /// Must be constant for the lifetime of the renderer: a caller builds
    /// behaviour from it, and a set that changed underneath would make that
    /// behaviour lie.
    fn capabilities(&self) -> Capabilities;

    /// Draws this document, replacing whatever was showing.
    ///
    /// Showing a second document is not an error and does not stack: the last
    /// one wins. That is the whole reason `show` takes a document rather than
    /// adding one — two subtitles on screen at once is a state the product
    /// never wants and therefore a state this port cannot express.
    ///
    /// **Security:** the document is dialogue (K23 #4). It may be drawn; it
    /// may never be logged, and no error returned here may name any part of it.
    fn show(&mut self, document: &SubtitleDocument) -> Result<(), RenderError>;

    /// Declares that the bottom `fraction` of the surface is covered by
    /// something else, and keeps the subtitle out of it (ADR-0037).
    ///
    /// `fraction` is a share of the **surface height** — never points, pixels
    /// or dp. The shell is the only place that knows how tall its own chrome
    /// is and the only place that knows the surface it sits on; a unit of
    /// length would make the core ask for a scale factor it has no use for.
    ///
    /// Values outside `0.0..=`[`MAX_BOTTOM_INSET`] are refused with
    /// [`RenderError::InsetOutOfRange`] rather than clamped: a clamped inset
    /// is indistinguishable from an honoured one, and the caller would go on
    /// believing the subtitle is clear of its chrome.
    ///
    /// This is **not** styling. A renderer is told which part of the surface
    /// is not visible, not where a subtitle should look best — position, size
    /// and style stay M7's questions and stay out of this port.
    ///
    /// Base rather than a capability (ADR-0037 Karar 4): a renderer that
    /// cannot be told where not to draw would put the line under the shell's
    /// own controls, which is a defect and not a degraded mode.
    fn set_bottom_inset(&mut self, fraction: f32) -> Result<(), RenderError>;

    /// Takes whatever is showing off screen.
    ///
    /// Not an error when nothing is showing — §8's `Kapalı` is reachable from
    /// any state, including the one it is already in.
    fn clear(&mut self) -> Result<(), RenderError>;

    /// The subtitle text on screen right now, if any.
    ///
    /// Needs [`Capability::ObservedText`]. `None` means nothing is drawn at
    /// this moment; a gap between cues is not an error.
    ///
    /// What comes back is what was **drawn**, not what was asked for. That is
    /// the point of the method: it is the only way a caller can check the cue
    /// on screen against the document it came from instead of trusting that
    /// the two agree.
    ///
    /// **Security:** dialogue again (K23 #4). Displayable and comparable,
    /// never loggable.
    fn rendered_text(&self) -> Result<Option<String>, RenderError> {
        Err(RenderError::Unsupported {
            operation: Operation::RenderedText,
            capability: Capability::ObservedText,
        })
    }
}
