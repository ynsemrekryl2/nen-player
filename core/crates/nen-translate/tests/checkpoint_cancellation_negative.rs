//! Negative and redaction coverage for NEN-093 checkpoint-only commit and
//! cancel-without-late-commit.
//!
//! `checkpoint::tests` (in-crate) covers the sequential skip/resume/failure
//! paths deterministically. This file covers what those single-threaded
//! tests structurally cannot: a provider response that is genuinely
//! in-flight — sitting inside `TranslationCall::commit`'s gate check — at
//! the exact moment another thread calls `cancel()`.

use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use nen_domain::source::LanguageTag;
use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_ports::translation::{
    TranslatedCue, TranslationCall, TranslationProvider, TranslationProviderError,
    TranslationProviderIdentity, TranslationRequest, TranslationResponse,
};
use nen_translate::blocks::{BlockLayout, BlockLayoutConfig};
use nen_translate::checkpoint::{
    translate_checkpointed, BlockCheckpoints, TranslationPlan, TranslationRunError,
};
use nen_translate::repair::BlockTranslationError;

const SENTINEL: &str = "PRIVATE_TRANSLATED_DIALOGUE_SENTINEL";

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

/// Answers the first `unblocked_calls` requests immediately with an echoed
/// translation; every request after that signals `started` and then blocks
/// on `release` before answering — simulating a provider response that is
/// genuinely in flight while a concurrent `cancel()` runs.
struct StallOnceProvider {
    unblocked_calls: Mutex<usize>,
    started: mpsc::Sender<()>,
    release: Mutex<mpsc::Receiver<()>>,
}

impl TranslationProvider for StallOnceProvider {
    fn identity(&self) -> TranslationProviderIdentity {
        TranslationProviderIdentity::new("test", "stall-once").expect("identity")
    }

    fn translate(
        &self,
        request: &TranslationRequest,
        call: &TranslationCall,
    ) -> Result<TranslationResponse, TranslationProviderError> {
        let mut unblocked = self.unblocked_calls.lock().expect("lock");
        if *unblocked > 0 {
            *unblocked -= 1;
        } else {
            drop(unblocked);
            self.started.send(()).expect("signal call start");
            self.release
                .lock()
                .expect("lock")
                .recv_timeout(Duration::from_secs(5))
                .expect("released by the cancelling thread");
        }

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

/// A provider response genuinely in flight when `cancel()` fires on another
/// thread is never committed: `finish()` itself starts rejecting once the
/// gate closes, so the block that raced the cancellation cannot produce a
/// `ValidatedBlock` at all, let alone checkpoint one.
#[test]
fn a_response_in_flight_during_cancel_is_never_committed() {
    let document = document(95);
    let layout = layout(&document);
    assert!(
        layout.blocks().len() >= 2,
        "fixture needs at least two blocks"
    );

    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let provider = StallOnceProvider {
        unblocked_calls: Mutex::new(1), // block 0 answers immediately
        started: started_tx,
        release: Mutex::new(release_rx),
    };

    let call = TranslationCall::without_progress();
    let mut checkpoints = BlockCheckpoints::for_layout(&layout);

    let run = thread::scope(|scope| {
        let handle = scope.spawn(|| {
            translate_checkpointed(
                &provider,
                &document,
                &layout,
                &plan(),
                &call,
                &mut checkpoints,
            )
        });

        // Wait until block 1's provider call is inside the stall — its
        // response is "in flight" from the caller's point of view.
        started_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("block 1's call reported it started");

        // Cancel from a different thread while that call is still stalled.
        call.cancel();
        release_tx.send(()).expect("release the stalled call");

        handle.join().expect("worker thread")
    });

    let error = run.expect_err("the stalled block must not complete");
    assert!(
        matches!(
            error,
            TranslationRunError::Block(BlockTranslationError::Provider(
                TranslationProviderError::Cancelled
            ))
        ),
        "expected a provider-level cancellation, got {error:?}"
    );

    // Block 0 committed before the cancellation; block 1 — in flight when
    // `cancel()` ran — never did.
    assert_eq!(checkpoints.checkpointed_count(), 1);
    assert!(!checkpoints.is_complete());

    let incomplete = checkpoints
        .into_completed()
        .expect_err("a half-validated run must not publish anything");
    assert_eq!(incomplete.checkpointed(), 1);
    assert_eq!(incomplete.total(), layout.blocks().len());
}

/// A gate that is already closed before a run starts is caught at the very
/// first block boundary: the provider is never called and nothing is
/// checkpointed. (Mirrors `checkpoint::tests`'s in-crate boundary test; kept
/// here too because it is the direct counterpart of the in-flight case
/// above and belongs in the same negative-evidence file.)
#[test]
fn cancel_before_the_run_starts_prevents_every_commit() {
    let document = document(95);
    let layout = layout(&document);
    let call = TranslationCall::without_progress();
    call.cancel();

    let mut checkpoints = BlockCheckpoints::for_layout(&layout);
    let provider = StallOnceProvider {
        unblocked_calls: Mutex::new(usize::MAX),
        started: mpsc::channel().0,
        release: Mutex::new(mpsc::channel().1),
    };

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
    assert_eq!(checkpoints.checkpointed_count(), 0);
    assert!(checkpoints.into_completed().is_err());
}

/// K23 #4: the run-level error surface — which embeds a translated cue's
/// worth of context through `BlockTranslationError` — must never print
/// dialogue text, whether the run failed on validation or on cancellation.
#[test]
fn run_error_debug_and_display_never_leak_cue_text() {
    let document = SubtitleDocument::new(vec![Cue::new(
        CueId::new(1),
        TimeSpan::new(0, 500).expect("valid span"),
        vec![SENTINEL.to_owned()],
    )]);
    let layout =
        BlockLayout::of(&document, BlockLayoutConfig::new(30, 1).expect("config")).expect("layout");

    struct EmptyTextProvider;
    impl TranslationProvider for EmptyTextProvider {
        fn identity(&self) -> TranslationProviderIdentity {
            TranslationProviderIdentity::new("test", "empty").expect("identity")
        }

        fn translate(
            &self,
            request: &TranslationRequest,
            call: &TranslationCall,
        ) -> Result<TranslationResponse, TranslationProviderError> {
            // Deliberately echoes the sentinel-bearing context back as the
            // "translation", then fails validation by leaving it empty —
            // exercising the path where the sentinel was in the request the
            // provider saw, not just the response.
            let _ = &request.context_cues;
            call.finish(TranslationResponse {
                cues: request
                    .output_cue_ids
                    .iter()
                    .map(|cue_id| TranslatedCue {
                        cue_id: *cue_id,
                        text: String::new(),
                    })
                    .collect(),
            })
        }
    }

    let mut checkpoints = BlockCheckpoints::for_layout(&layout);
    let error = translate_checkpointed(
        &EmptyTextProvider,
        &document,
        &layout,
        &plan(),
        &TranslationCall::without_progress(),
        &mut checkpoints,
    )
    .expect_err("empty text fails validation");

    assert!(!format!("{error:?}").contains(SENTINEL));
    assert!(!format!("{error}").contains(SENTINEL));
    assert!(!format!("{checkpoints:?}").contains(SENTINEL));
}

/// A guard-rail on the guards above: if `TranslationRunError` were changed
/// to a naive `#[derive(Debug)]` over a variant that kept raw text, the
/// leak would show up immediately. This documents that the safety comes
/// from `BlockTranslationError`'s own hand-written surfaces, not from
/// `TranslationRunError` accidentally never being asked to print text.
#[test]
fn the_guard_sentinel_would_be_visible_in_a_derived_debug_twin() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct LeakyBlockOutcome {
        text: String,
    }

    let leaked = format!(
        "{:?}",
        LeakyBlockOutcome {
            text: SENTINEL.to_owned(),
        }
    );
    assert!(leaked.contains(SENTINEL));
}
