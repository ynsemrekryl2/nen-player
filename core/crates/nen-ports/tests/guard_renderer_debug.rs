//! K23 guard for the renderer port (NEN-027).
//!
//! A renderer holds the one thing this codebase must never log: the dialogue
//! it is drawing (K23 #4, `security-policy.md` §1 #4). Every type on this path
//! is printed here and searched for it.
//!
//! The derived twin is the point. A `Debug` written by hand is only a promise;
//! the twin shows what the derive would have printed instead, so this file
//! fails the day someone replaces an impl with `#[derive(Debug)]`.

use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_ports::renderer::error::Operation;
use nen_ports::renderer::{Capabilities, FakeRenderer, RenderError, SubtitleRenderer};

const PRIVATE_DIALOGUE: &str = "Bunu kimseye söyleme.";

fn document() -> SubtitleDocument {
    SubtitleDocument::new(vec![Cue::new(
        CueId::new(1),
        TimeSpan::new(1_000, 2_000).expect("a valid span"),
        vec![PRIVATE_DIALOGUE.to_string()],
    )])
}

#[test]
fn a_renderer_never_prints_what_it_is_drawing() {
    let mut renderer = FakeRenderer::full();
    renderer.show(&document()).expect("shown");

    let printed = format!("{renderer:?}");
    assert!(
        !printed.contains(PRIVATE_DIALOGUE),
        "dialogue leaked: {printed}"
    );
    // What is left is still useful: a reader can see that something is showing
    // and how much of it, which is what a log is for.
    assert!(printed.contains("showing_cues"), "{printed}");
    assert!(printed.contains("capabilities"), "{printed}");
}

#[test]
fn a_derived_renderer_really_does_leak() {
    // The negative control, shaped like the mistake a renderer actually makes:
    // caching the line it last drew, so it can be asked without going back to
    // the surface. That cache is dialogue, and a derive prints it.
    #[derive(Debug)]
    #[allow(dead_code)] // read only by the derive, which is the point
    struct DerivedTwin {
        capabilities: Capabilities,
        last_drawn: Option<String>,
    }

    let twin = DerivedTwin {
        capabilities: Capabilities::ALL,
        last_drawn: Some(PRIVATE_DIALOGUE.to_string()),
    };
    let printed = format!("{twin:?}");
    assert!(
        printed.contains(PRIVATE_DIALOGUE),
        "the control does not leak, so the guard above proves nothing: {printed}"
    );
}

#[test]
fn no_render_error_variant_carries_private_data() {
    // Constructed rather than asserted about: every variant is a bounded enum
    // or a number, so there is nothing a foreign language could re-print.
    for error in [
        RenderError::Unavailable {
            operation: Operation::Show,
        },
        RenderError::NoMedia {
            operation: Operation::Show,
        },
        RenderError::ShutDown {
            operation: Operation::Clear,
        },
        RenderError::ReentrantCall {
            operation: Operation::Show,
        },
        RenderError::SurfaceFailure {
            operation: Operation::Show,
            code: -42,
        },
    ] {
        let printed = format!("{error:?}");
        assert!(
            !printed.contains('/'),
            "a path could appear here: {printed}"
        );
        assert!(
            !printed.contains(PRIVATE_DIALOGUE),
            "dialogue could appear here: {printed}"
        );
    }
}
