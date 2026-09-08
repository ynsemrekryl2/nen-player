//! Security guard for whole-document context and block layout (K23 #4,
//! `docs/security-policy.md`).
//!
//! A [`ContextTerm`] is dialogue text pulled straight out of the document,
//! which makes `DocumentContext`'s `Debug` the obvious place for it to leak
//! into a log. This proves it does not, and that the guard is not blind: the
//! sentinel is engineered to actually become a kept term (repeated, mid
//! sentence), so the assertion below is meaningful rather than vacuous.
//!
//! `BlockLayout`/`TranslationBlock`/`BlockLayoutError` never carry cue text
//! at all (only positions and counts), so their guard here is a structural
//! sanity check rather than a discriminating one.

mod support;

use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_translate::blocks::{BlockLayout, BlockLayoutConfig, BlockLayoutError};
use nen_translate::context::DocumentContext;

/// Text that must never reach the context's output. Long and distinctive so
/// a partial leak is caught as surely as a whole one, and shaped like a
/// context-extraction candidate (capitalized, alphanumeric) so it is not
/// filtered out before it ever reaches a `ContextTerm`.
const SENTINEL: &str = "Zzqxvunlogged";

fn document_with_sentinel() -> SubtitleDocument {
    let lines = [
        format!("Hello {SENTINEL} dialogue."),
        format!("More {SENTINEL} dialogue."),
        format!("Even more {SENTINEL} dialogue."),
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
fn the_sentinel_actually_becomes_a_kept_term() {
    let document = document_with_sentinel();
    let context = DocumentContext::of(&document);
    let names: Vec<&str> = context.terms().iter().map(|t| t.term()).collect();
    assert_eq!(
        names,
        vec![SENTINEL],
        "guard is vacuous unless the sentinel is actually kept"
    );
}

#[test]
fn no_context_debug_output_leaks_the_term_text() {
    let document = document_with_sentinel();
    let context = DocumentContext::of(&document);

    let context_debug = format!("{context:?}");
    assert!(
        !context_debug.contains(SENTINEL),
        "DocumentContext::Debug leaked term text — {context_debug}"
    );

    for term in context.terms() {
        let term_debug = format!("{term:?}");
        assert!(
            !term_debug.contains(SENTINEL),
            "ContextTerm::Debug leaked term text — {term_debug}"
        );
    }
}

#[test]
fn no_block_layout_debug_or_display_output_leaks_document_text() {
    let document = document_with_sentinel();
    let layout = BlockLayout::of(&document, BlockLayoutConfig::default())
        .expect("small fixture fits in one block");

    let layout_debug = format!("{layout:?}");
    assert!(!layout_debug.contains(SENTINEL), "{layout_debug}");

    for block in layout.blocks() {
        let block_debug = format!("{block:?}");
        assert!(!block_debug.contains(SENTINEL), "{block_debug}");
    }

    let error = BlockLayoutError::BlockSizeOutOfRange { got: 61 };
    let error_debug = format!("{error:?}");
    let error_display = format!("{error}");
    assert!(!error_debug.contains(SENTINEL), "{error_debug}");
    assert!(!error_display.contains(SENTINEL), "{error_display}");
}
