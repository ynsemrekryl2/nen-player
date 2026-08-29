//! A deterministic in-memory `SubtitleRenderer`, and the reference for what
//! the contract requires.
//!
//! Two jobs, the same two [`playback::fake`](crate::playback::fake) has: when
//! the kit and an adapter disagree about what a rule means, this is what the
//! rule means; and it lets everything above the port be tested without a
//! screen.
//!
//! It draws nothing, because nothing here has pixels. What it models is the
//! *state* a renderer has: which document is showing, and whether anything is
//! on screen at all.

use super::capability::{Capabilities, Capability};
use super::error::{Operation, RenderError};
use super::surface::SubtitleRenderer;
use nen_domain::subtitle::SubtitleDocument;
use std::fmt;

/// A fake renderer whose capability set is chosen by the caller.
pub struct FakeRenderer {
    capabilities: Capabilities,
    showing: Option<SubtitleDocument>,
}

impl FakeRenderer {
    /// A renderer declaring every optional capability.
    pub fn full() -> Self {
        Self::new(Capabilities::ALL)
    }

    /// A renderer that only draws.
    ///
    /// The one that proves the other half of the capability rule: the gated
    /// operation must come back as a typed refusal rather than a panic or a
    /// no-op.
    pub fn minimal() -> Self {
        Self::new(Capabilities::NONE)
    }

    pub fn new(capabilities: Capabilities) -> Self {
        Self {
            capabilities,
            showing: None,
        }
    }

    /// How many cues the showing document carries, if one is showing.
    /// Test-facing; not part of the port.
    pub fn showing_cues(&self) -> Option<usize> {
        self.showing.as_ref().map(SubtitleDocument::len)
    }
}

impl SubtitleRenderer for FakeRenderer {
    fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    fn show(&mut self, document: &SubtitleDocument) -> Result<(), RenderError> {
        self.showing = Some(document.clone());
        Ok(())
    }

    fn clear(&mut self) -> Result<(), RenderError> {
        self.showing = None;
        Ok(())
    }

    fn rendered_text(&self) -> Result<Option<String>, RenderError> {
        if !self.capabilities.contains(Capability::ObservedText) {
            return Err(RenderError::Unsupported {
                operation: Operation::RenderedText,
                capability: Capability::ObservedText,
            });
        }
        // The fake has no playhead, so it has an imaginary one, frozen inside
        // the first cue. That is the contract kit's precondition made concrete:
        // a surface positioned inside the document draws something, and one
        // showing nothing draws nothing.
        Ok(self
            .showing
            .as_ref()
            .and_then(|document| document.cues().first())
            .map(|cue| cue.lines().join("\n")))
    }
}

impl fmt::Debug for FakeRenderer {
    /// Prints what is safe: the capabilities and whether something is showing.
    ///
    /// Never the document. It is dialogue (K23 #4), and a fake that leaked it
    /// in a test log would leak it just as thoroughly as product code.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FakeRenderer")
            .field("capabilities", &self.capabilities)
            .field("showing_cues", &self.showing_cues())
            .finish()
    }
}
