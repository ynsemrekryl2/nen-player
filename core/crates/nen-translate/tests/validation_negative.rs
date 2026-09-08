//! Negative and redaction coverage for NEN-091 strict block validation.

use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_ports::translation::{TranslatedCue, TranslationResponse};
use nen_translate::blocks::{BlockLayout, BlockLayoutConfig};
use nen_translate::validation::{validate_block, BlockViolation};

const SENTINEL: &str = "PRIVATE_TRANSLATED_DIALOGUE_SENTINEL";

fn document() -> SubtitleDocument {
    SubtitleDocument::new(
        (1..=3)
            .map(|id| {
                Cue::new(
                    CueId::new(id),
                    TimeSpan::new(id * 1_000, id * 1_000 + 500).expect("valid span"),
                    vec![format!("Source {id}")],
                )
            })
            .collect(),
    )
}

fn block(document: &SubtitleDocument) -> nen_translate::blocks::TranslationBlock {
    BlockLayout::of(document, BlockLayoutConfig::new(30, 1).expect("config"))
        .expect("layout")
        .blocks()[0]
        .clone()
}

fn response(cues: &[(u32, &str)]) -> TranslationResponse {
    TranslationResponse {
        cues: cues
            .iter()
            .map(|(cue_id, text)| TranslatedCue {
                cue_id: CueId::new(*cue_id),
                text: (*text).into(),
            })
            .collect(),
    }
}

#[test]
fn an_extra_unknown_cue_is_rejected_without_a_repair_target() {
    let document = document();
    let error = validate_block(
        &document,
        &block(&document),
        &response(&[(1, "one"), (2, "two"), (3, "three"), (99, "extra")]),
    )
    .expect_err("unknown cue must be rejected");

    assert!(error.repair_cue_ids().is_empty());
    assert!(error.violations().contains(&BlockViolation::UnknownCue {
        cue_id: CueId::new(99)
    }));
}

#[test]
fn a_changed_cue_id_is_unknown_and_missing() {
    let document = document();
    let error = validate_block(
        &document,
        &block(&document),
        &response(&[(1, "one"), (20, "changed"), (3, "three")]),
    )
    .expect_err("changed cue id must be rejected");

    assert_eq!(error.repair_cue_ids(), &[CueId::new(2)]);
    assert!(error.violations().contains(&BlockViolation::UnknownCue {
        cue_id: CueId::new(20)
    }));
    assert!(error.violations().contains(&BlockViolation::MissingCue {
        cue_id: CueId::new(2)
    }));
}

#[test]
fn failure_and_success_debug_surfaces_hide_translated_text() {
    let document = document();
    let block = block(&document);
    let invalid_response = response(&[(1, SENTINEL), (2, " "), (3, "three")]);
    let error = validate_block(&document, &block, &invalid_response).expect_err("empty cue");
    let error_debug = format!("{error:?}");
    let error_display = format!("{error}");
    assert!(!error_debug.contains(SENTINEL), "{error_debug}");
    assert!(!error_display.contains(SENTINEL), "{error_display}");

    let valid = validate_block(
        &document,
        &block,
        &response(&[(1, SENTINEL), (2, "two"), (3, "three")]),
    )
    .expect("valid response");
    assert!(!format!("{valid:?}").contains(SENTINEL));
    assert!(!format!("{:?}", valid.cues()[0]).contains(SENTINEL));
}

#[test]
fn the_guard_sentinel_would_be_visible_in_a_derived_debug_twin() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct LeakyTranslatedCue {
        text: String,
    }

    let output = format!(
        "{:?}",
        LeakyTranslatedCue {
            text: SENTINEL.into(),
        }
    );
    assert!(output.contains(SENTINEL));
}
