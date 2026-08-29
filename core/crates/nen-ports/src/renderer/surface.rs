//! The `SubtitleRenderer` port itself.
//!
//! `docs/product-spec.md` §14 asks for two things and this trait is both of
//! them: subtitle drawing is **its own port**, and the UI does not know which
//! implementation is behind it. ADR-0013 chose the first adapter — the engine
//! draws, and the core hands it the document — without letting that choice
//! reach a single call site.
//!
//! The port is deliberately small. It says what a renderer must do (show a
//! document, take it off screen), what only some can do (say what is on screen
//! right now), and nothing at all about *how* — no styling, no positioning, no
//! sync offset. Those are M7's questions and would be answered here by
//! guessing.
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
