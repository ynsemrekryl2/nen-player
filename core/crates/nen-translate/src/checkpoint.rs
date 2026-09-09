//! Blocks-in-sequence orchestration with checkpoint-only commit (NEN-093,
//! `docs/product-spec.md` §10, ADR-0004 Karar 2/5).
//!
//! [`translate_checkpointed`] drives [`repair::translate_block_with_repair`]
//! over every block of a [`BlockLayout`] in order. A block is committed into
//! a [`BlockCheckpoints`] only after it has fully passed local validation,
//! and the commit itself runs through [`TranslationCall::commit`] — the same
//! delivery gate that guards progress and result delivery — so a `cancel()`
//! call either happens before a block's commit (nothing from that block is
//! kept) or waits for an in-flight commit to finish and then blocks every
//! later one. There is no interleaving where a late commit slips through.
//!
//! Already-checkpointed blocks are skipped without a provider call, so a
//! resumed run — same [`BlockCheckpoints`], same layout — never re-translates
//! work it already has. [`BlockCheckpoints::into_completed`] is the only way
//! to obtain a [`CompletedBlocks`], and it refuses unless every block in the
//! layout is checkpointed: a partially translated document cannot be turned
//! into something a caller could publish.
//!
//! Each block's provider work runs under its own [`TranslationCall::fork`]
//! (NEN-106): a provider only ever sees one block's request, so it reports
//! that block's own cue count as its progress `total`, which legitimately
//! differs from block to block. A fork shares the run's single cancellation
//! gate and progress sink but starts a fresh per-provider progress
//! sequence, so [`TranslationCall::progress`]'s monotonic-`total` rule
//! (ADR-0004 Karar 4) applies within a block, never across blocks.

use crate::blocks::{BlockLayout, TranslationBlock};
use crate::repair::{self, BlockTranslationError};
use crate::validation::ValidatedBlock;
use nen_domain::source::LanguageTag;
use nen_domain::subtitle::SubtitleDocument;
use nen_ports::translation::{
    TranslationCall, TranslationCue, TranslationProvider, TranslationProviderError,
    TranslationRequest,
};
use std::fmt;

/// The language pair and document-wide context every block of one
/// translation run shares. Carries no per-cue text of its own, but
/// `context_terms` is subtitle-derived (K23 #4), so `Debug` prints only a
/// count — never the terms themselves.
#[derive(Clone, PartialEq, Eq)]
pub struct TranslationPlan {
    pub source_language: LanguageTag,
    pub target_language: LanguageTag,
    pub context_terms: Vec<String>,
}

impl fmt::Debug for TranslationPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TranslationPlan")
            .field("source_language", &self.source_language)
            .field("target_language", &self.target_language)
            .field("context_term_count", &self.context_terms.len())
            .finish()
    }
}

/// Per-block checkpoint slots for one [`BlockLayout`]. Only a fully
/// validated [`ValidatedBlock`] can occupy a slot, and only
/// [`TranslationCall::commit`] can fill one — see the module docs for why
/// that closes the late-commit gap.
pub struct BlockCheckpoints {
    slots: Vec<Option<ValidatedBlock>>,
}

impl BlockCheckpoints {
    /// One empty slot per block in `layout`, indexed by
    /// [`TranslationBlock::index`].
    pub fn for_layout(layout: &BlockLayout) -> Self {
        Self {
            slots: (0..layout.blocks().len()).map(|_| None).collect(),
        }
    }

    pub fn is_checkpointed(&self, block_index: usize) -> bool {
        self.slots
            .get(block_index)
            .is_some_and(|slot| slot.is_some())
    }

    pub fn checkpointed_count(&self) -> usize {
        self.slots.iter().filter(|slot| slot.is_some()).count()
    }

    pub fn total_blocks(&self) -> usize {
        self.slots.len()
    }

    pub fn is_complete(&self) -> bool {
        self.checkpointed_count() == self.total_blocks()
    }

    /// Commit one validated block under `call`'s delivery gate. Returns
    /// [`TranslationProviderError::Cancelled`] if the gate is already closed,
    /// or if it closes while this call is waiting for the gate lock a
    /// concurrent `cancel()` holds — either way, nothing is written.
    fn commit(
        &mut self,
        call: &TranslationCall,
        block: ValidatedBlock,
    ) -> Result<(), TranslationProviderError> {
        let index = block.block_index();
        let slot = self
            .slots
            .get_mut(index)
            .ok_or(TranslationProviderError::Permanent)?;
        call.commit(|| {
            *slot = Some(block);
        })
    }

    /// Consume this checkpoint set. Succeeds only when every block in the
    /// layout is checkpointed — the type-level enforcement of "no partial
    /// document" (`docs/product-spec.md` §10).
    pub fn into_completed(self) -> Result<CompletedBlocks, IncompleteRun> {
        let total = self.slots.len();
        let checkpointed = self.slots.iter().filter(|slot| slot.is_some()).count();
        if checkpointed != total {
            return Err(IncompleteRun {
                checkpointed,
                total,
            });
        }
        Ok(CompletedBlocks {
            blocks: self.slots.into_iter().flatten().collect(),
        })
    }
}

impl fmt::Debug for BlockCheckpoints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockCheckpoints")
            .field("checkpointed", &self.checkpointed_count())
            .field("total_blocks", &self.total_blocks())
            .finish()
    }
}

/// [`BlockCheckpoints::into_completed`] was called before every block was
/// checkpointed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IncompleteRun {
    checkpointed: usize,
    total: usize,
}

impl IncompleteRun {
    pub const fn checkpointed(&self) -> usize {
        self.checkpointed
    }

    pub const fn total(&self) -> usize {
        self.total
    }
}

impl fmt::Display for IncompleteRun {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "translation run incomplete ({}/{} blocks checkpointed)",
            self.checkpointed, self.total
        )
    }
}

impl std::error::Error for IncompleteRun {}

/// Every block of a document, fully validated and checkpointed, in block
/// order. Can only be produced by [`BlockCheckpoints::into_completed`].
pub struct CompletedBlocks {
    blocks: Vec<ValidatedBlock>,
}

impl CompletedBlocks {
    pub fn blocks(&self) -> &[ValidatedBlock] {
        &self.blocks
    }
}

impl fmt::Debug for CompletedBlocks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompletedBlocks")
            .field("block_count", &self.blocks.len())
            .finish()
    }
}

/// Why a checkpointed run stopped before every block was committed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationRunError {
    /// The delivery gate was already closed, or closed while this run was
    /// waiting on it. Whatever was checkpointed before the cancellation
    /// stands; nothing more will be committed by this call.
    Cancelled,
    /// One block exhausted its repair budget or the provider itself failed.
    Block(BlockTranslationError),
}

impl fmt::Display for TranslationRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => f.write_str("translation run cancelled"),
            Self::Block(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for TranslationRunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Cancelled => None,
            Self::Block(error) => Some(error),
        }
    }
}

/// Translate every not-yet-checkpointed block of `layout` in order and
/// commit each as it validates.
///
/// A block already present in `checkpoints` (from a prior, interrupted run)
/// is skipped without calling `provider` at all — resuming never re-does
/// finished work. The gate is checked at each block boundary before that
/// block's provider work starts (ADR-0004 Karar 3); a closed gate stops the
/// run immediately and leaves every later block un-checkpointed.
pub fn translate_checkpointed(
    provider: &dyn TranslationProvider,
    document: &SubtitleDocument,
    layout: &BlockLayout,
    plan: &TranslationPlan,
    call: &TranslationCall,
    checkpoints: &mut BlockCheckpoints,
) -> Result<(), TranslationRunError> {
    for block in layout.blocks() {
        if checkpoints.is_checkpointed(block.index()) {
            continue;
        }

        call.checkpoint()
            .map_err(|_error| TranslationRunError::Cancelled)?;

        let request = request_for_block(document, block, plan);
        // Each block gets its own forked progress sequence: a provider only
        // sees its own block's request and honestly reports that block's own
        // cue count as `total`, which legitimately differs block to block.
        // `TranslationCall::progress` rightly rejects a changed `total`
        // within one sequence, so blocks must not share one (NEN-106). The
        // fork shares cancellation and progress delivery — only the
        // per-provider progress sequence starts fresh.
        let validated =
            repair::translate_block_with_repair(provider, document, block, &request, &call.fork())
                .map_err(TranslationRunError::Block)?;

        checkpoints
            .commit(call, validated)
            .map_err(|_error| TranslationRunError::Cancelled)?;
    }
    Ok(())
}

fn request_for_block(
    document: &SubtitleDocument,
    block: &TranslationBlock,
    plan: &TranslationPlan,
) -> TranslationRequest {
    let cues = document.cues();
    let context_cues = block
        .context_positions()
        .filter_map(|position| cues.get(position))
        .map(|cue| TranslationCue {
            cue_id: cue.id(),
            text: cue.lines().join("\n"),
        })
        .collect();
    TranslationRequest {
        source_language: plan.source_language.clone(),
        target_language: plan.target_language.clone(),
        context_cues,
        output_cue_ids: block.output_cue_ids(document),
        context_terms: plan.context_terms.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::BlockLayoutConfig;
    use nen_domain::subtitle::{Cue, CueId, TimeSpan};
    use nen_ports::translation::{
        TranslatedCue, TranslationProgress, TranslationProgressPhase, TranslationProgressSink,
        TranslationProviderIdentity, TranslationResponse,
    };
    use std::sync::{Arc, Mutex};

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

    fn layout(document: &SubtitleDocument) -> BlockLayout {
        BlockLayout::of(document, BlockLayoutConfig::new(30, 1).expect("config")).expect("layout")
    }

    fn plan() -> TranslationPlan {
        TranslationPlan {
            source_language: LanguageTag::parse("en").expect("language"),
            target_language: LanguageTag::parse("tr").expect("language"),
            context_terms: Vec::new(),
        }
    }

    /// Always returns a translation identical (as text) to the source cue
    /// text, so every block validates on the first attempt.
    #[derive(Clone)]
    struct EchoProvider {
        calls: Arc<Mutex<usize>>,
    }

    impl EchoProvider {
        fn new() -> Self {
            Self {
                calls: Arc::new(Mutex::new(0)),
            }
        }

        fn call_count(&self) -> usize {
            *self.calls.lock().expect("lock")
        }
    }

    impl TranslationProvider for EchoProvider {
        fn identity(&self) -> TranslationProviderIdentity {
            TranslationProviderIdentity::new("test", "echo").expect("identity")
        }

        fn translate(
            &self,
            request: &TranslationRequest,
            call: &TranslationCall,
        ) -> Result<TranslationResponse, TranslationProviderError> {
            *self.calls.lock().expect("lock") += 1;
            let cues = request
                .output_cue_ids
                .iter()
                .map(|cue_id| TranslatedCue {
                    cue_id: *cue_id,
                    text: format!("translated {}", cue_id.get()),
                })
                .collect();
            call.finish(TranslationResponse { cues })
        }
    }

    /// Fails validation (empty text) for every requested cue, every call.
    struct AlwaysInvalidProvider;

    impl TranslationProvider for AlwaysInvalidProvider {
        fn identity(&self) -> TranslationProviderIdentity {
            TranslationProviderIdentity::new("test", "invalid").expect("identity")
        }

        fn translate(
            &self,
            request: &TranslationRequest,
            call: &TranslationCall,
        ) -> Result<TranslationResponse, TranslationProviderError> {
            let cues = request
                .output_cue_ids
                .iter()
                .map(|cue_id| TranslatedCue {
                    cue_id: *cue_id,
                    text: String::new(),
                })
                .collect();
            call.finish(TranslationResponse { cues })
        }
    }

    #[test]
    fn a_failed_block_is_never_checkpointed() {
        let document = document(3);
        let layout = layout(&document);
        let mut checkpoints = BlockCheckpoints::for_layout(&layout);

        let error = translate_checkpointed(
            &AlwaysInvalidProvider,
            &document,
            &layout,
            &plan(),
            &TranslationCall::without_progress(),
            &mut checkpoints,
        )
        .expect_err("validation must fail");

        assert!(matches!(error, TranslationRunError::Block(_)));
        assert_eq!(checkpoints.checkpointed_count(), 0);
        assert!(!checkpoints.is_complete());
    }

    #[test]
    fn a_resumed_run_never_re_translates_checkpointed_blocks() {
        // 3-block document so the run can be interrupted after block 0.
        let document = document(95);
        let layout = layout(&document);
        assert!(layout.blocks().len() >= 2, "fixture needs multiple blocks");

        let mut checkpoints = BlockCheckpoints::for_layout(&layout);
        let provider = EchoProvider::new();
        let call = TranslationCall::without_progress();

        // First run: manually stop after the first block by only feeding a
        // single-block "layout view" via a pre-populated checkpoint set is
        // not available, so instead cancel after we observe block 0 land by
        // running to completion once, recording the call count, then
        // resuming with a fresh provider/call and checking for zero calls.
        translate_checkpointed(
            &provider,
            &document,
            &layout,
            &plan(),
            &call,
            &mut checkpoints,
        )
        .expect("first run completes");
        let first_run_calls = provider.call_count();
        assert!(first_run_calls > 0);
        assert!(checkpoints.is_complete());

        let resumed_provider = EchoProvider::new();
        let resumed_call = TranslationCall::without_progress();
        translate_checkpointed(
            &resumed_provider,
            &document,
            &layout,
            &plan(),
            &resumed_call,
            &mut checkpoints,
        )
        .expect("resumed run is a no-op");

        assert_eq!(
            resumed_provider.call_count(),
            0,
            "every block was already checkpointed"
        );
    }

    #[test]
    fn interrupted_run_resumes_from_its_checkpoints_without_retranslating_them() {
        let document = document(95);
        let layout = layout(&document);
        assert!(layout.blocks().len() >= 3, "fixture needs several blocks");

        let mut checkpoints = BlockCheckpoints::for_layout(&layout);

        // Cancel the gate right after the first block's commit by wrapping
        // the provider so it cancels the call once it has served the second
        // block's request (i.e. after block 0 checkpoints).
        struct CancelAfterNCalls {
            inner: EchoProvider,
            remaining_before_cancel: Mutex<usize>,
        }

        impl TranslationProvider for CancelAfterNCalls {
            fn identity(&self) -> TranslationProviderIdentity {
                self.inner.identity()
            }

            fn translate(
                &self,
                request: &TranslationRequest,
                call: &TranslationCall,
            ) -> Result<TranslationResponse, TranslationProviderError> {
                let response = self.inner.translate(request, call)?;
                let mut remaining = self.remaining_before_cancel.lock().expect("lock");
                if *remaining == 0 {
                    call.cancel();
                } else {
                    *remaining -= 1;
                }
                Ok(response)
            }
        }

        // Let the first call (block 0) through untouched; cancel once the
        // second call (block 1) has already produced — but not yet
        // committed — its response, so block 0's commit is unaffected and
        // block 1's is the one the gate rejects.
        let provider = CancelAfterNCalls {
            inner: EchoProvider::new(),
            remaining_before_cancel: Mutex::new(1),
        };
        let call = TranslationCall::without_progress();

        let error = translate_checkpointed(
            &provider,
            &document,
            &layout,
            &plan(),
            &call,
            &mut checkpoints,
        )
        .expect_err("run is cancelled mid-way");
        assert_eq!(error, TranslationRunError::Cancelled);
        assert_eq!(checkpoints.checkpointed_count(), 1);
        assert!(!checkpoints.is_complete());

        // Resume with a fresh call/provider: only the remaining blocks are
        // translated, the already-checkpointed one is skipped.
        let resumed_provider = EchoProvider::new();
        let resumed_call = TranslationCall::without_progress();
        translate_checkpointed(
            &resumed_provider,
            &document,
            &layout,
            &plan(),
            &resumed_call,
            &mut checkpoints,
        )
        .expect("resumed run completes");

        assert!(checkpoints.is_complete());
        assert_eq!(
            resumed_provider.call_count(),
            layout.blocks().len() - 1,
            "only the un-checkpointed blocks are re-requested"
        );
    }

    #[test]
    fn a_gate_closed_before_the_run_starts_is_checked_at_the_block_boundary() {
        // Cancel before `translate_checkpointed` is ever called: the loop's
        // own `call.checkpoint()` boundary check must catch this without
        // ever reaching the provider.
        let document = document(95);
        let layout = layout(&document);
        let mut checkpoints = BlockCheckpoints::for_layout(&layout);
        let provider = EchoProvider::new();
        let call = TranslationCall::without_progress();
        call.cancel();

        let error = translate_checkpointed(
            &provider,
            &document,
            &layout,
            &plan(),
            &call,
            &mut checkpoints,
        )
        .expect_err("gate is already closed");

        assert_eq!(error, TranslationRunError::Cancelled);
        assert_eq!(provider.call_count(), 0, "provider must not be called");
        assert_eq!(checkpoints.checkpointed_count(), 0);
    }

    #[test]
    fn incomplete_checkpoints_refuse_to_become_completed_blocks() {
        let document = document(95);
        let layout = layout(&document);
        let checkpoints = BlockCheckpoints::for_layout(&layout);

        let error = checkpoints
            .into_completed()
            .expect_err("no block was checkpointed");
        assert_eq!(error.checkpointed(), 0);
        assert_eq!(error.total(), layout.blocks().len());
    }

    #[test]
    fn complete_checkpoints_become_completed_blocks_in_order() {
        let document = document(3);
        let layout = layout(&document);
        let mut checkpoints = BlockCheckpoints::for_layout(&layout);
        let provider = EchoProvider::new();

        translate_checkpointed(
            &provider,
            &document,
            &layout,
            &plan(),
            &TranslationCall::without_progress(),
            &mut checkpoints,
        )
        .expect("run completes");

        let completed = checkpoints.into_completed().expect("all checkpointed");
        let indices: Vec<usize> = completed.blocks().iter().map(|b| b.block_index()).collect();
        let sorted = {
            let mut sorted = indices.clone();
            sorted.sort_unstable();
            sorted
        };
        assert_eq!(indices, sorted, "blocks are kept in block order");
    }

    /// Reports progress honestly for its own request (`total` = the
    /// request's own output cue count) and echoes each cue back, so it
    /// validates on the first attempt — the shape of `MockTranslationProvider`
    /// that `NEN-106` measured as failing on a second block.
    struct ProgressReportingProvider;

    impl TranslationProvider for ProgressReportingProvider {
        fn identity(&self) -> TranslationProviderIdentity {
            TranslationProviderIdentity::new("test", "progress").expect("identity")
        }

        fn translate(
            &self,
            request: &TranslationRequest,
            call: &TranslationCall,
        ) -> Result<TranslationResponse, TranslationProviderError> {
            let total = u32::try_from(request.output_cue_ids.len()).expect("small fixture");
            call.progress(TranslationProgress {
                phase: TranslationProgressPhase::Preparing,
                done: 0,
                total,
            })?;
            let mut cues = Vec::with_capacity(request.output_cue_ids.len());
            for (position, cue_id) in request.output_cue_ids.iter().copied().enumerate() {
                cues.push(TranslatedCue {
                    cue_id,
                    text: format!("translated {}", cue_id.get()),
                });
                call.progress(TranslationProgress {
                    phase: TranslationProgressPhase::Translating,
                    done: u32::try_from(position + 1).expect("small fixture"),
                    total,
                })?;
            }
            call.finish(TranslationResponse { cues })
        }
    }

    /// Records every delivered `TranslationProgress`, in delivery order.
    #[derive(Default)]
    struct RecordingSink {
        events: Mutex<Vec<TranslationProgress>>,
    }

    impl TranslationProgressSink for RecordingSink {
        fn on_progress(&self, progress: TranslationProgress) {
            self.events.lock().expect("lock").push(progress);
        }
    }

    impl RecordingSink {
        fn events(&self) -> Vec<TranslationProgress> {
            self.events.lock().expect("lock").clone()
        }
    }

    /// `NEN-106`: a provider that honestly reports its own block's cue count
    /// as `total` must not have its second block rejected as a `Permanent`
    /// provider error just because the first block's `total` differed.
    #[test]
    fn a_provider_reporting_its_own_block_total_completes_a_multi_block_run() {
        let document = document(95);
        let layout = layout(&document);
        assert!(layout.blocks().len() >= 2, "fixture needs multiple blocks");

        let mut checkpoints = BlockCheckpoints::for_layout(&layout);
        let sink = Arc::new(RecordingSink::default());
        let call = TranslationCall::new(sink.clone());

        translate_checkpointed(
            &ProgressReportingProvider,
            &document,
            &layout,
            &plan(),
            &call,
            &mut checkpoints,
        )
        .expect("a provider that honestly reports its own block total must not be rejected");

        assert!(checkpoints.is_complete());
    }

    /// Each block's progress sequence starts fresh at its own `total` and is
    /// monotonic within itself — no event carries over a previous block's
    /// `total` or `done`.
    #[test]
    fn each_blocks_progress_sequence_is_independently_monotonic() {
        let document = document(95);
        let layout = layout(&document);
        let block_totals: Vec<u32> = layout
            .blocks()
            .iter()
            .map(|block| {
                u32::try_from(block.output_cue_ids(&document).len()).expect("small fixture")
            })
            .collect();
        assert!(block_totals.len() >= 2, "fixture needs multiple blocks");

        let mut checkpoints = BlockCheckpoints::for_layout(&layout);
        let sink = Arc::new(RecordingSink::default());
        let call = TranslationCall::new(sink.clone());

        translate_checkpointed(
            &ProgressReportingProvider,
            &document,
            &layout,
            &plan(),
            &call,
            &mut checkpoints,
        )
        .expect("run completes");

        let events = sink.events();
        // Split the flat event stream back into per-block runs by watching
        // `done` reset to 0 (`Preparing`) at each block boundary, and check
        // each run's own total matches that block's own cue count, and its
        // `done` values are monotonically non-decreasing within the run.
        let mut block_index = 0usize;
        let mut previous_done: Option<u32> = None;
        for event in &events {
            if event.phase == TranslationProgressPhase::Preparing {
                if previous_done.is_some() {
                    block_index += 1;
                }
                previous_done = None;
            }
            assert_eq!(
                event.total, block_totals[block_index],
                "block {block_index}'s progress must carry its own total"
            );
            if let Some(previous) = previous_done {
                assert!(
                    event.done >= previous,
                    "done must not regress within a block"
                );
            }
            previous_done = Some(event.done);
        }
        assert_eq!(
            block_index + 1,
            block_totals.len(),
            "every block must have reported its own progress run"
        );
    }

    /// The gate closing mid-run still stops the run at the block boundary
    /// (ADR-0004 Karar 2/5) even though each block now runs its own forked
    /// progress sequence — cancellation is shared state, not per-fork.
    /// Mirrors `interrupted_run_resumes_from_its_checkpoints_without_retranslating_them`
    /// but through a provider that also reports progress, so a forked
    /// progress sequence is actually exercised on both the committed and the
    /// cancelled block.
    #[test]
    fn cancellation_still_stops_the_run_when_blocks_fork_their_progress() {
        let document = document(95);
        let layout = layout(&document);
        assert!(layout.blocks().len() >= 2, "fixture needs multiple blocks");

        struct CancelAfterNCalls {
            remaining_before_cancel: Mutex<usize>,
        }

        impl TranslationProvider for CancelAfterNCalls {
            fn identity(&self) -> TranslationProviderIdentity {
                TranslationProviderIdentity::new("test", "cancel-after-n").expect("identity")
            }

            fn translate(
                &self,
                request: &TranslationRequest,
                call: &TranslationCall,
            ) -> Result<TranslationResponse, TranslationProviderError> {
                let response = ProgressReportingProvider.translate(request, call)?;
                let mut remaining = self.remaining_before_cancel.lock().expect("lock");
                if *remaining == 0 {
                    call.cancel();
                } else {
                    *remaining -= 1;
                }
                Ok(response)
            }
        }

        let mut checkpoints = BlockCheckpoints::for_layout(&layout);
        let provider = CancelAfterNCalls {
            remaining_before_cancel: Mutex::new(1),
        };
        let sink = Arc::new(RecordingSink::default());
        let call = TranslationCall::new(sink);

        let error = translate_checkpointed(
            &provider,
            &document,
            &layout,
            &plan(),
            &call,
            &mut checkpoints,
        )
        .expect_err("run is cancelled mid-way");

        assert_eq!(error, TranslationRunError::Cancelled);
        assert_eq!(checkpoints.checkpointed_count(), 1);
        assert!(!checkpoints.is_complete());
    }
}
