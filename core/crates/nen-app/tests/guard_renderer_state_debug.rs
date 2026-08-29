//! K23 guard for what the renderer remembers (NEN-027).
//!
//! [`RendererState`] holds the document being drawn, which is dialogue
//! (K23 #4, `security-policy.md` §1 #4). It is the one field on the session
//! that carries the user's own text, so what it prints is checked here with a
//! deliberately derived twin standing next to it.

use nen_app::renderer::RendererState;
use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};

const PRIVATE_DIALOGUE: &str = "Bunu kimseye söyleme.";

fn document() -> SubtitleDocument {
    SubtitleDocument::new(vec![Cue::new(
        CueId::new(1),
        TimeSpan::new(1_000, 2_000).expect("a valid span"),
        vec![PRIVATE_DIALOGUE.to_string()],
    )])
}

/// Puts a document into the state the only way anything can: by drawing it.
fn showing() -> RendererState {
    use nen_ports::playback::fake::FakeEngine;
    use nen_ports::playback::{MediaSource, PlaybackEngine};
    use nen_ports::renderer::SubtitleRenderer;

    let mut engine = FakeEngine::full();
    engine
        .load(&MediaSource::new("fixtures/media/contract-clip.mkv"))
        .expect("load");
    let mut state = RendererState::new();
    nen_app::renderer::EngineNativeRenderer::new(&mut engine, &mut state)
        .show(&document())
        .expect("shown");
    state
}

#[test]
fn the_renderer_state_never_prints_the_document() {
    let state = showing();
    assert!(state.is_showing());

    let printed = format!("{state:?}");
    assert!(
        !printed.contains(PRIVATE_DIALOGUE),
        "dialogue leaked: {printed}"
    );
    // What is left still says what a reader needs: that something is showing,
    // and how much of it.
    assert!(printed.contains("showing: true"), "{printed}");
    assert!(printed.contains("cue_count"), "{printed}");
}

#[test]
fn a_derived_state_really_does_leak() {
    // The negative control, shaped like the mistake this state invites:
    // keeping the lines rather than the document, so they can be answered
    // without rebuilding an index. A derive prints them.
    #[derive(Debug)]
    #[allow(dead_code)] // read only by the derive, which is the point
    struct DerivedTwin {
        showing: bool,
        lines: Vec<String>,
    }

    let twin = DerivedTwin {
        showing: true,
        lines: vec![PRIVATE_DIALOGUE.to_string()],
    };
    let printed = format!("{twin:?}");
    assert!(
        printed.contains(PRIVATE_DIALOGUE),
        "the control does not leak, so the guard above proves nothing: {printed}"
    );
}

#[test]
fn forgetting_leaves_nothing_to_print_and_nothing_to_answer() {
    let mut state = showing();
    state.forget();
    assert!(!state.is_showing());
    assert_eq!(state.expected_at(1_500), None);
    assert!(format!("{state:?}").contains("showing: false"));
}
