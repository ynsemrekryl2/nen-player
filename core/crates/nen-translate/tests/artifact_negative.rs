//! Negative tests for artifact assembly (NEN-094 DoD).
//!
//! Two things must never produce a [`nen_translate::artifact::ValidatedSubtitleArtifact`]:
//! a translation run that never fully checkpointed, and a checkpointed run
//! whose cues do not actually belong to the document [`assemble`] is asked
//! to pair it with.

mod support;

use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_ports::translation::{
    TranslatedCue, TranslationCall, TranslationProvider, TranslationProviderError,
    TranslationProviderIdentity, TranslationRequest, TranslationResponse,
};
use nen_translate::artifact::ArtifactError;
use nen_translate::blocks::{BlockLayout, BlockLayoutConfig};
use nen_translate::checkpoint::{translate_checkpointed, BlockCheckpoints};

fn document(cue_count: u32) -> SubtitleDocument {
    SubtitleDocument::new(
        (1..=cue_count)
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

/// Same cue count and timing as [`document`], but every `CueId` is shifted
/// by 1000 — the same "shape" as a translation of `document`, but not
/// actually of it.
fn document_with_renumbered_cue_ids(cue_count: u32) -> SubtitleDocument {
    SubtitleDocument::new(
        (1..=cue_count)
            .map(|id| {
                Cue::new(
                    CueId::new(id + 1_000),
                    TimeSpan::new(id * 1_000, id * 1_000 + 500).expect("valid span"),
                    vec![format!("Source {id}")],
                )
            })
            .collect(),
    )
}

/// Same cue count and `CueId`s as [`document`], but every span is shifted by
/// 250ms — same identity, different timing.
fn document_with_shifted_spans(cue_count: u32) -> SubtitleDocument {
    SubtitleDocument::new(
        (1..=cue_count)
            .map(|id| {
                Cue::new(
                    CueId::new(id),
                    TimeSpan::new(id * 1_000 + 250, id * 1_000 + 750).expect("valid span"),
                    vec![format!("Source {id}")],
                )
            })
            .collect(),
    )
}

fn layout(document: &SubtitleDocument) -> BlockLayout {
    layout_with_config(document, 30, 1)
}

fn layout_with_config(
    document: &SubtitleDocument,
    block_size: usize,
    overlap: usize,
) -> BlockLayout {
    BlockLayout::of(
        document,
        BlockLayoutConfig::new(block_size, overlap).expect("config"),
    )
    .expect("layout")
}

#[test]
fn a_run_that_never_fully_checkpointed_cannot_reach_assembly() {
    // Cancel the gate after the first block lands, mirroring
    // checkpoint::tests::interrupted_run_resumes_from_its_checkpoints. There
    // is no `assemble` call in this test at all: the point is that
    // `into_completed` — the only door to a `CompletedBlocks`, which
    // `assemble` requires — refuses to open. A partial run is unrepresentable
    // as artifact input, not merely rejected by a runtime check.
    struct CancelAfterFirstBlock {
        served: std::sync::atomic::AtomicUsize,
    }

    impl TranslationProvider for CancelAfterFirstBlock {
        fn identity(&self) -> TranslationProviderIdentity {
            TranslationProviderIdentity::new("test", "cancel-after-first").expect("identity")
        }

        fn translate(
            &self,
            request: &TranslationRequest,
            call: &TranslationCall,
        ) -> Result<TranslationResponse, TranslationProviderError> {
            let served = self
                .served
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let cues = request
                .output_cue_ids
                .iter()
                .map(|cue_id| TranslatedCue {
                    cue_id: *cue_id,
                    text: format!("translated {}", cue_id.get()),
                })
                .collect();
            if served > 0 {
                call.cancel();
            }
            call.finish(TranslationResponse { cues })
        }
    }

    let document = document(95);
    let layout = layout(&document);
    assert!(layout.blocks().len() >= 2, "fixture needs multiple blocks");
    let mut checkpoints = BlockCheckpoints::for_layout(&layout);
    let provider = CancelAfterFirstBlock {
        served: std::sync::atomic::AtomicUsize::new(0),
    };

    let error = translate_checkpointed(
        &provider,
        &document,
        &layout,
        &support::plan(),
        &TranslationCall::without_progress(),
        &mut checkpoints,
    )
    .expect_err("second block's commit is cancelled");
    assert!(
        format!("{error}").contains("cancelled"),
        "expected the run to report cancellation, got {error}"
    );
    assert!(!checkpoints.is_complete());

    let incomplete = checkpoints
        .into_completed()
        .expect_err("a partial run must not become CompletedBlocks");
    assert!(incomplete.checkpointed() < incomplete.total());
    // No `CompletedBlocks` exists past this point, so there is nothing to
    // hand `assemble` — the type system, not a runtime check, is what closed
    // this door.
}

#[test]
fn a_completed_run_paired_with_a_document_of_different_cue_ids_is_rejected() {
    let source = document(3);
    let layout = layout(&source);
    let completed = support::build_completed(&source, &layout);

    let mismatched = document_with_renumbered_cue_ids(3);
    let error = support::try_build_artifact(&mismatched, &layout, &completed)
        .expect_err("cue ids do not match the paired document");
    assert!(
        matches!(error, ArtifactError::CueMismatch { .. }),
        "expected CueMismatch, got {error:?}"
    );
}

#[test]
fn a_completed_run_paired_with_a_document_of_shifted_timing_is_rejected() {
    let source = document(3);
    let layout = layout(&source);
    let completed = support::build_completed(&source, &layout);

    let mismatched = document_with_shifted_spans(3);
    let error = support::try_build_artifact(&mismatched, &layout, &completed)
        .expect_err("timing does not match the paired document");
    assert!(
        matches!(error, ArtifactError::SpanMismatch { .. }),
        "expected SpanMismatch, got {error:?}"
    );
}

#[test]
fn a_layout_built_for_a_shorter_document_is_rejected() {
    let source = document(3);
    let layout = layout(&source);
    let completed = support::build_completed(&source, &layout);

    let shorter = document(2);
    let error = support::try_build_artifact(&shorter, &layout, &completed)
        .expect_err("layout was not built for this document's length");
    assert!(
        matches!(error, ArtifactError::LayoutMismatch { .. }),
        "expected LayoutMismatch, got {error:?}"
    );
}

#[test]
fn a_layout_built_for_a_shorter_document_than_a_longer_one_is_rejected() {
    // The layout's own blocks only cover positions 0..3, so without the
    // up-front cue-count check this document's extra, un-covered cue would
    // simply never be looked at during assembly — this is the scenario that
    // actually discriminates the top-of-function `LayoutMismatch` check from
    // the per-cue bounds check inside the loop (see NEN-094 kanıt kaydı).
    let source = document(3);
    let layout = layout(&source);
    let completed = support::build_completed(&source, &layout);

    let longer = document(4);
    let error = support::try_build_artifact(&longer, &layout, &completed)
        .expect_err("layout was not built for this document's length");
    assert!(
        matches!(error, ArtifactError::LayoutMismatch { .. }),
        "expected LayoutMismatch, got {error:?}"
    );
}

#[test]
fn a_completed_run_paired_with_a_differently_shaped_layout_is_rejected() {
    // Same document, same cue count — so the up-front `LayoutMismatch` check
    // does not fire — but two different block-size configs, which split it
    // into a different number of blocks. Pairing a run completed against one
    // layout with the *other* layout is the scenario `BlockCount` exists for.
    let source = document(90);
    let narrow_layout = layout_with_config(&source, 30, 1);
    let wide_layout = layout_with_config(&source, 60, 1);
    assert_ne!(
        narrow_layout.blocks().len(),
        wide_layout.blocks().len(),
        "fixture must exercise genuinely different block counts"
    );

    let completed = support::build_completed(&source, &narrow_layout);
    let error = support::try_build_artifact(&source, &wide_layout, &completed)
        .expect_err("completed was checkpointed against a differently-shaped layout");
    assert!(
        matches!(error, ArtifactError::BlockCount { .. }),
        "expected BlockCount, got {error:?}"
    );
}

#[test]
fn a_completed_run_paired_with_a_layout_of_the_same_block_count_but_different_windows_is_rejected()
{
    // Same document, same block *count* (3) for both configs, but a
    // different overlap shifts each block's own output window size — the
    // scenario `BlockCueCount` exists for, distinct from `BlockCount` above.
    let source = document(60);
    let layout_a = layout_with_config(&source, 30, 1);
    let layout_b = layout_with_config(&source, 30, 14);
    assert_eq!(layout_a.blocks().len(), layout_b.blocks().len());
    assert_ne!(
        layout_a.blocks()[0].output_positions().len(),
        layout_b.blocks()[0].output_positions().len(),
        "fixture must exercise genuinely different per-block window sizes"
    );

    let completed = support::build_completed(&source, &layout_a);
    let error = support::try_build_artifact(&source, &layout_b, &completed)
        .expect_err("completed's blocks do not match layout_b's own output windows");
    assert!(
        matches!(error, ArtifactError::BlockCueCount { .. }),
        "expected BlockCueCount, got {error:?}"
    );
}
