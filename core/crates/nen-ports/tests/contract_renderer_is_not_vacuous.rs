//! Proves the renderer contract kit actually catches things.
//!
//! A kit is only worth what it rejects. Each renderer below is broken in one
//! specific direction the kit tolerates elsewhere, and the kit must go red for
//! each — otherwise a scenario is decoration.

use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_ports::renderer::contract::{run_all, RenderInputs};
use nen_ports::renderer::error::Operation;
use nen_ports::renderer::{Capabilities, Capability, RenderError, SubtitleRenderer};

fn inputs() -> RenderInputs {
    RenderInputs::new(SubtitleDocument::new(vec![Cue::new(
        CueId::new(1),
        TimeSpan::new(1_000, 2_000).expect("a valid span"),
        vec!["contract".to_string()],
    )]))
}

/// Declares `ObservedText` and refuses to answer it anyway.
struct Liar;

impl SubtitleRenderer for Liar {
    fn capabilities(&self) -> Capabilities {
        Capabilities::ALL
    }
    fn show(&mut self, _: &SubtitleDocument) -> Result<(), RenderError> {
        Ok(())
    }
    fn clear(&mut self) -> Result<(), RenderError> {
        Ok(())
    }
    // The default impl refuses — which is correct only for a renderer that
    // does *not* declare the capability.
}

/// Draws, but never stops drawing.
struct NeverClears {
    showing: bool,
}

impl SubtitleRenderer for NeverClears {
    fn capabilities(&self) -> Capabilities {
        Capabilities::ALL
    }
    fn show(&mut self, _: &SubtitleDocument) -> Result<(), RenderError> {
        self.showing = true;
        Ok(())
    }
    fn clear(&mut self) -> Result<(), RenderError> {
        // The silent no-op `docs/architecture.md` forbids: it reports success
        // and leaves the subtitle on screen.
        Ok(())
    }
    fn rendered_text(&self) -> Result<Option<String>, RenderError> {
        Ok(self.showing.then(|| "still here".to_string()))
    }
}

/// Refuses to show anything at all.
struct Refuses;

impl SubtitleRenderer for Refuses {
    fn capabilities(&self) -> Capabilities {
        Capabilities::NONE
    }
    fn show(&mut self, _: &SubtitleDocument) -> Result<(), RenderError> {
        Err(RenderError::Unavailable {
            operation: Operation::Show,
        })
    }
    fn clear(&mut self) -> Result<(), RenderError> {
        Ok(())
    }
}

/// Draws nothing even after being shown a document.
struct DrawsNothing;

impl SubtitleRenderer for DrawsNothing {
    fn capabilities(&self) -> Capabilities {
        Capabilities::new([Capability::ObservedText])
    }
    fn show(&mut self, _: &SubtitleDocument) -> Result<(), RenderError> {
        Ok(())
    }
    fn clear(&mut self) -> Result<(), RenderError> {
        Ok(())
    }
    fn rendered_text(&self) -> Result<Option<String>, RenderError> {
        Ok(None)
    }
}

#[test]
fn a_declared_capability_that_refuses_is_caught() {
    assert!(!run_all(|| Liar, &inputs()).is_empty());
}

#[test]
fn a_clear_that_does_nothing_is_caught() {
    assert!(!run_all(|| NeverClears { showing: false }, &inputs()).is_empty());
}

#[test]
fn a_renderer_that_cannot_show_is_caught() {
    assert!(!run_all(|| Refuses, &inputs()).is_empty());
}

#[test]
fn a_renderer_that_draws_nothing_is_caught() {
    assert!(!run_all(|| DrawsNothing, &inputs()).is_empty());
}
