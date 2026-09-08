//! Security guard for the assembled artifact (K23 #4, `docs/security-policy.md`).
//!
//! A [`nen_translate::artifact::ValidatedSubtitleArtifact`] carries translated
//! dialogue in its `translated_document()`/`webvtt()`, which makes its
//! `Debug` the obvious place for that text to leak into a log. This proves it
//! does not, and that the guard is not blind: the sentinel is engineered to
//! actually reach the artifact's WebVTT output, so the assertion is
//! meaningful rather than vacuous.
//!
//! [`nen_translate::artifact::ArtifactError`] never carries cue text at all
//! (only positions, counts and `CueId`s), so its guard here is a structural
//! sanity check, matching `guard_context_debug.rs`'s treatment of
//! `BlockLayoutError`.

mod support;

use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_translate::artifact::ArtifactError;

/// Text that must never reach `Debug`/`Display` output. Long and distinctive
/// so a partial leak is caught as surely as a whole one.
const SENTINEL: &str = "Zzqxvunlogged";

fn document_with_sentinel() -> SubtitleDocument {
    let lines = [
        format!("Hello {SENTINEL} dialogue."),
        format!("More {SENTINEL} dialogue."),
    ];
    let cues = lines
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            let start = (i as u32) * 1_000;
            Cue::new(
                CueId::new(i as u32 + 1),
                TimeSpan::new(start, start + 900).expect("well-formed span"),
                vec![line],
            )
        })
        .collect();
    SubtitleDocument::new(cues)
}

#[test]
fn no_artifact_debug_output_leaks_dialogue_text() {
    let document = document_with_sentinel();
    let artifact = support::build_artifact(&document);

    // Guard is not vacuous: the sentinel really is in the artifact's own
    // payload (via EchoTranslationProvider, which echoes source text back).
    assert!(
        artifact.webvtt().contains(SENTINEL),
        "fixture does not exercise the guard — sentinel never reached the artifact"
    );

    let debug = format!("{artifact:?}");
    assert!(
        !debug.contains(SENTINEL),
        "ValidatedSubtitleArtifact::Debug leaked dialogue text — {debug}"
    );
}

#[test]
fn no_artifact_error_debug_or_display_output_leaks_dialogue_text() {
    let cue_id = CueId::new(1);
    let errors = [
        ArtifactError::LayoutMismatch {
            expected: 3,
            received: 2,
        },
        ArtifactError::BlockCount {
            expected: 2,
            received: 1,
        },
        ArtifactError::BlockCueCount {
            block_index: 0,
            expected: 3,
            received: 2,
        },
        ArtifactError::CueMismatch {
            position: 0,
            expected: cue_id,
            received: CueId::new(2),
        },
        ArtifactError::SpanMismatch { cue_id },
        ArtifactError::TimelineMismatch,
    ];

    for error in errors {
        let debug = format!("{error:?}");
        let display = format!("{error}");
        assert!(!debug.contains(SENTINEL), "{debug}");
        assert!(!display.contains(SENTINEL), "{display}");
    }
}
