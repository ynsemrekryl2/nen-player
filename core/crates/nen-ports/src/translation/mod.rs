//! Provider-neutral translation port (NEN-090, ADR-0004).
//!
//! The port is deliberately synchronous and runtime-free. The caller owns the
//! worker and job lifecycle; adapters receive a shared [`TranslationCall`]
//! that gates progress and the returned result against cancellation.

pub mod contract;

use nen_domain::source::LanguageTag;
use nen_domain::subtitle::CueId;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

/// Cache-visible contract versions shared by pipeline and provider adapters
/// (ADR-0048). Keeping them at the port boundary prevents wire/pipeline drift.
pub const PROMPT_VERSION: u32 = 2;
pub const SCHEMA_VERSION: u32 = 2;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TranslationProviderIdentity {
    provider: String,
    model: String,
}

impl TranslationProviderIdentity {
    pub fn new(provider: &str, model: &str) -> Result<Self, TranslationProviderError> {
        if provider.trim().is_empty() || model.trim().is_empty() {
            return Err(TranslationProviderError::Permanent);
        }
        Ok(Self {
            provider: provider.to_owned(),
            model: model.to_owned(),
        })
    }

    pub fn provider(&self) -> &str {
        &self.provider
    }

    pub fn model(&self) -> &str {
        &self.model
    }
}

impl fmt::Debug for TranslationProviderIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TranslationProviderIdentity")
            .field("provider_len", &self.provider.chars().count())
            .field("model_len", &self.model.chars().count())
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct TranslationCue {
    pub cue_id: CueId,
    pub text: String,
}

impl fmt::Debug for TranslationCue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TranslationCue")
            .field("cue_id", &self.cue_id)
            .field("text_len", &self.text.chars().count())
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct AnalysisCharacter {
    pub name: String,
    pub description: String,
}

impl fmt::Debug for AnalysisCharacter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnalysisCharacter")
            .field("name_len", &self.name.chars().count())
            .field("description_len", &self.description.chars().count())
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct AnalysisGlossaryEntry {
    pub source: String,
    pub target: String,
    pub note: String,
}

impl fmt::Debug for AnalysisGlossaryEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnalysisGlossaryEntry")
            .field("source_len", &self.source.chars().count())
            .field("target_len", &self.target.chars().count())
            .field("note_len", &self.note.chars().count())
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DocumentAnalysis {
    pub summary: String,
    pub characters: Vec<AnalysisCharacter>,
    pub glossary: Vec<AnalysisGlossaryEntry>,
}

impl DocumentAnalysis {
    /// Re-validates provider and disk data before it can become block context.
    pub fn validate(&self) -> Result<(), DocumentAnalysisValidationError> {
        if self.summary.trim().is_empty() {
            return Err(DocumentAnalysisValidationError::EmptySummary);
        }
        if self.characters.iter().any(|character| {
            character.name.trim().is_empty() || character.description.trim().is_empty()
        }) {
            return Err(DocumentAnalysisValidationError::EmptyCharacterField);
        }
        if self.glossary.iter().any(|entry| {
            entry.source.trim().is_empty()
                || entry.target.trim().is_empty()
                || entry.note.trim().is_empty()
        }) {
            return Err(DocumentAnalysisValidationError::EmptyGlossaryField);
        }
        Ok(())
    }
}

impl fmt::Debug for DocumentAnalysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DocumentAnalysis")
            .field("summary_len", &self.summary.chars().count())
            .field("character_count", &self.characters.len())
            .field("glossary_count", &self.glossary.len())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentAnalysisValidationError {
    EmptySummary,
    EmptyCharacterField,
    EmptyGlossaryField,
}

impl fmt::Display for DocumentAnalysisValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptySummary => "document analysis summary is empty",
            Self::EmptyCharacterField => "document analysis character field is empty",
            Self::EmptyGlossaryField => "document analysis glossary field is empty",
        })
    }
}

impl std::error::Error for DocumentAnalysisValidationError {}

#[derive(Clone, PartialEq, Eq)]
pub struct DocumentAnalysisRequest {
    pub source_language: LanguageTag,
    pub target_language: LanguageTag,
    pub transcript: Vec<TranslationCue>,
    pub context_terms: Vec<String>,
}

impl fmt::Debug for DocumentAnalysisRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DocumentAnalysisRequest")
            .field("source_language", &self.source_language)
            .field("target_language", &self.target_language)
            .field("transcript_cue_count", &self.transcript.len())
            .field("context_term_count", &self.context_terms.len())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationMode {
    Initial,
    TargetedRepair,
    FullRetry,
}

#[derive(Clone, PartialEq, Eq)]
pub struct TranslationRequest {
    pub source_language: LanguageTag,
    pub target_language: LanguageTag,
    pub context_cues: Vec<TranslationCue>,
    pub output_cue_ids: Vec<CueId>,
    pub analysis: DocumentAnalysis,
    pub block_index: u32,
    pub block_count: u32,
    pub mode: TranslationMode,
}

impl fmt::Debug for TranslationRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TranslationRequest")
            .field("source_language", &self.source_language)
            .field("target_language", &self.target_language)
            .field("context_cue_count", &self.context_cues.len())
            .field("output_cue_count", &self.output_cue_ids.len())
            .field("analysis", &self.analysis)
            .field("block_index", &self.block_index)
            .field("block_count", &self.block_count)
            .field("mode", &self.mode)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct TranslatedCue {
    pub cue_id: CueId,
    pub text: String,
}

impl fmt::Debug for TranslatedCue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TranslatedCue")
            .field("cue_id", &self.cue_id)
            .field("text_len", &self.text.chars().count())
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct TranslationResponse {
    pub cues: Vec<TranslatedCue>,
}

impl fmt::Debug for TranslationResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TranslationResponse")
            .field("cue_count", &self.cues.len())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TranslationProgressPhase {
    Preparing,
    Translating,
    Finalizing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TranslationProgress {
    pub phase: TranslationProgressPhase,
    /// Provider-local when supplied to [`TranslationCall::progress`]. A
    /// document-scoped call maps this to the document-wide offset before it
    /// reaches the sink.
    pub done: u32,
    /// Provider-local when supplied to [`TranslationCall::progress`]. A
    /// document-scoped call replaces this with the document cue count for the
    /// sink-facing value.
    pub total: u32,
}

pub trait TranslationProgressSink: Send + Sync {
    fn on_progress(&self, progress: TranslationProgress);
}

#[derive(Clone, Copy)]
struct DocumentProgressScope {
    offset: u32,
    total: u32,
}

#[derive(Default)]
struct DocumentProgressState {
    last: Option<TranslationProgress>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationProviderError {
    Cancelled,
    Transient,
    Permanent,
    ResponseTooLarge,
}

impl fmt::Display for TranslationProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cancelled => "translation cancelled",
            Self::Transient => "translation provider temporarily unavailable",
            Self::Permanent => "translation provider failed",
            Self::ResponseTooLarge => "translation provider response was too large",
        })
    }
}

impl std::error::Error for TranslationProviderError {}

struct DeliveryState {
    cancelled: bool,
    sink: Option<Arc<dyn TranslationProgressSink>>,
}

#[derive(Clone)]
pub struct TranslationCall {
    state: Arc<Mutex<DeliveryState>>,
    last_progress: Arc<Mutex<Option<TranslationProgress>>>,
    document_progress: Arc<Mutex<DocumentProgressState>>,
    document_scope: Option<DocumentProgressScope>,
}

impl TranslationCall {
    pub fn new(sink: Arc<dyn TranslationProgressSink>) -> Self {
        Self::from_sink(Some(sink))
    }

    pub fn without_progress() -> Self {
        Self::from_sink(None)
    }

    fn from_sink(sink: Option<Arc<dyn TranslationProgressSink>>) -> Self {
        Self {
            state: Arc::new(Mutex::new(DeliveryState {
                cancelled: false,
                sink,
            })),
            last_progress: Arc::new(Mutex::new(None)),
            document_progress: Arc::new(Mutex::new(DocumentProgressState::default())),
            document_scope: None,
        }
    }

    /// Create a retry call that shares cancellation and progress delivery but
    /// starts a fresh per-provider progress sequence. A repair request may
    /// contain fewer cue IDs than the original request, so its provider-facing
    /// progress total legitimately differs from the preceding attempt. When
    /// this call is document-scoped, its sink-facing high-water mark remains
    /// shared with the retry.
    pub fn fork(&self) -> Self {
        Self {
            state: self.state.clone(),
            last_progress: Arc::new(Mutex::new(None)),
            document_progress: self.document_progress.clone(),
            document_scope: self.document_scope,
        }
    }

    /// Start a fresh provider progress sequence that reports into one
    /// document-wide counter. `offset` is the number of output cues belonging
    /// to preceding blocks; `total` is the whole document's output cue count.
    /// The provider still reports its own block-local `done`/`total`, while a
    /// sink attached to this scoped call receives the mapped document values.
    pub fn with_document_progress(
        &self,
        offset: u32,
        total: u32,
    ) -> Result<Self, TranslationProviderError> {
        if offset > total {
            return Err(TranslationProviderError::Permanent);
        }
        let mut scoped = self.fork();
        scoped.document_scope = Some(DocumentProgressScope { offset, total });
        Ok(scoped)
    }

    pub fn cancel(&self) {
        let mut state = self.lock_fail_closed();
        state.cancelled = true;
        state.sink = None;
    }

    pub fn is_cancelled(&self) -> bool {
        self.lock_fail_closed().cancelled
    }

    pub fn checkpoint(&self) -> Result<(), TranslationProviderError> {
        if self.lock_fail_closed().cancelled {
            Err(TranslationProviderError::Cancelled)
        } else {
            Ok(())
        }
    }

    pub fn progress(&self, progress: TranslationProgress) -> Result<(), TranslationProviderError> {
        let state = self.lock_fail_closed();
        if state.cancelled {
            return Err(TranslationProviderError::Cancelled);
        }
        let mut last_progress = self
            .last_progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if progress.done > progress.total
            || last_progress.is_some_and(|previous| {
                progress.total != previous.total
                    || progress.phase < previous.phase
                    || (progress.phase == previous.phase && progress.done < previous.done)
            })
        {
            return Err(TranslationProviderError::Permanent);
        }
        // Validate and remember the provider's local sequence first. The
        // document-wide mapping below deliberately does not relax this
        // provider contract; it only changes what a scoped sink receives.
        let delivered = self.map_document_progress(progress)?;
        *last_progress = Some(progress);
        if let (Some(sink), Some(delivered)) = (state.sink.as_ref(), delivered) {
            sink.on_progress(delivered);
        }
        Ok(())
    }

    pub fn finish(
        &self,
        response: TranslationResponse,
    ) -> Result<TranslationResponse, TranslationProviderError> {
        let state = self.lock_fail_closed();
        if state.cancelled {
            Err(TranslationProviderError::Cancelled)
        } else {
            Ok(response)
        }
    }

    /// Run `f` under the same delivery gate that guards progress and result
    /// delivery (ADR-0004 Karar 5). If the gate is closed, `f` never runs and
    /// [`TranslationProviderError::Cancelled`] is returned. Otherwise the gate
    /// lock is held for the duration of `f`, so a concurrent `cancel()` blocks
    /// until `f` returns, and once `cancel()` has returned no `commit` can run.
    ///
    /// `f` must not call back into this `TranslationCall` (the gate lock is
    /// not reentrant and doing so will deadlock).
    pub fn commit<T>(&self, f: impl FnOnce() -> T) -> Result<T, TranslationProviderError> {
        let state = self.lock_fail_closed();
        if state.cancelled {
            Err(TranslationProviderError::Cancelled)
        } else {
            Ok(f())
        }
    }

    fn lock_fail_closed(&self) -> MutexGuard<'_, DeliveryState> {
        match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => {
                let mut state = poisoned.into_inner();
                state.cancelled = true;
                state.sink = None;
                state
            }
        }
    }

    fn map_document_progress(
        &self,
        progress: TranslationProgress,
    ) -> Result<Option<TranslationProgress>, TranslationProviderError> {
        let Some(scope) = self.document_scope else {
            return Ok(Some(progress));
        };
        let candidate = scope
            .offset
            .checked_add(progress.done)
            .ok_or(TranslationProviderError::Permanent)?;
        if candidate > scope.total {
            return Err(TranslationProviderError::Permanent);
        }

        let mut state = self
            .document_progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state
            .last
            .is_some_and(|previous| previous.total != scope.total)
        {
            return Err(TranslationProviderError::Permanent);
        }
        let done = state
            .last
            .map_or(candidate, |previous| previous.done.max(candidate));
        let delivered = TranslationProgress {
            phase: progress.phase,
            done,
            total: scope.total,
        };
        if state.last == Some(delivered) {
            return Ok(None);
        }
        state.last = Some(delivered);
        Ok(Some(delivered))
    }
}

impl fmt::Debug for TranslationCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TranslationCall")
            .field("cancelled", &self.is_cancelled())
            .finish_non_exhaustive()
    }
}

pub trait TranslationProvider: Send + Sync {
    fn identity(&self) -> TranslationProviderIdentity;

    fn analyze_document(
        &self,
        request: &DocumentAnalysisRequest,
        call: &TranslationCall,
    ) -> Result<DocumentAnalysis, TranslationProviderError>;

    fn translate(
        &self,
        request: &TranslationRequest,
        call: &TranslationCall,
    ) -> Result<TranslationResponse, TranslationProviderError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopSink;

    impl TranslationProgressSink for NoopSink {
        fn on_progress(&self, _progress: TranslationProgress) {}
    }

    #[test]
    fn cancellation_closes_progress_and_result_delivery() {
        let call = TranslationCall::new(Arc::new(NoopSink));
        call.cancel();
        assert_eq!(call.checkpoint(), Err(TranslationProviderError::Cancelled));
        assert_eq!(
            call.progress(TranslationProgress {
                phase: TranslationProgressPhase::Preparing,
                done: 0,
                total: 1,
            }),
            Err(TranslationProviderError::Cancelled)
        );
        assert_eq!(
            call.finish(TranslationResponse { cues: Vec::new() }),
            Err(TranslationProviderError::Cancelled)
        );
    }

    #[test]
    fn progress_rejects_regression_and_out_of_bounds_counts() {
        let call = TranslationCall::without_progress();
        call.progress(TranslationProgress {
            phase: TranslationProgressPhase::Translating,
            done: 1,
            total: 2,
        })
        .expect("first progress");
        assert_eq!(
            call.progress(TranslationProgress {
                phase: TranslationProgressPhase::Translating,
                done: 0,
                total: 2,
            }),
            Err(TranslationProviderError::Permanent)
        );
        assert_eq!(
            TranslationCall::without_progress().progress(TranslationProgress {
                phase: TranslationProgressPhase::Preparing,
                done: 2,
                total: 1,
            }),
            Err(TranslationProviderError::Permanent)
        );
    }

    #[test]
    fn fork_resets_progress_sequence_but_shares_cancellation() {
        let call = TranslationCall::without_progress();
        call.progress(TranslationProgress {
            phase: TranslationProgressPhase::Finalizing,
            done: 3,
            total: 3,
        })
        .expect("initial call completes");

        let retry = call.fork();
        retry
            .progress(TranslationProgress {
                phase: TranslationProgressPhase::Preparing,
                done: 0,
                total: 1,
            })
            .expect("retry starts a fresh sequence");

        call.cancel();
        assert_eq!(retry.checkpoint(), Err(TranslationProviderError::Cancelled));
        assert_eq!(
            retry.progress(TranslationProgress {
                phase: TranslationProgressPhase::Finalizing,
                done: 1,
                total: 1,
            }),
            Err(TranslationProviderError::Cancelled)
        );
    }

    #[test]
    fn document_progress_maps_offsets_and_clamps_retry_regressions() {
        #[derive(Default)]
        struct RecordingSink {
            updates: Mutex<Vec<TranslationProgress>>,
        }

        impl TranslationProgressSink for RecordingSink {
            fn on_progress(&self, progress: TranslationProgress) {
                self.updates.lock().expect("lock").push(progress);
            }
        }

        let sink = Arc::new(RecordingSink::default());
        let call = TranslationCall::new(sink.clone());
        let scoped = call
            .with_document_progress(4, 10)
            .expect("valid document progress scope");
        scoped
            .progress(TranslationProgress {
                phase: TranslationProgressPhase::Translating,
                done: 3,
                total: 3,
            })
            .expect("initial block progress");
        scoped
            .fork()
            .progress(TranslationProgress {
                phase: TranslationProgressPhase::Preparing,
                done: 0,
                total: 1,
            })
            .expect("retry starts a fresh provider sequence");

        let updates = sink.updates.lock().expect("lock").clone();
        assert_eq!(updates[0].done, 7, "offset is added to local progress");
        assert_eq!(updates[0].total, 10, "sink receives the document total");
        assert_eq!(updates[1].done, 7, "retry cannot lower document progress");
        assert_eq!(updates[1].total, 10);
    }

    #[test]
    fn sensitive_values_are_shape_only_in_debug_output() {
        let secret = "PRIVATE SUBTITLE DIALOGUE";
        let cue = TranslationCue {
            cue_id: CueId::new(7),
            text: secret.into(),
        };
        let request = TranslationRequest {
            source_language: LanguageTag::parse("en").expect("language"),
            target_language: LanguageTag::parse("tr").expect("language"),
            context_cues: vec![cue.clone()],
            output_cue_ids: vec![cue.cue_id],
            analysis: DocumentAnalysis {
                summary: secret.into(),
                characters: vec![AnalysisCharacter {
                    name: secret.into(),
                    description: secret.into(),
                }],
                glossary: vec![AnalysisGlossaryEntry {
                    source: secret.into(),
                    target: secret.into(),
                    note: secret.into(),
                }],
            },
            block_index: 1,
            block_count: 1,
            mode: TranslationMode::Initial,
        };
        let response = TranslationResponse {
            cues: vec![TranslatedCue {
                cue_id: cue.cue_id,
                text: secret.into(),
            }],
        };

        for output in [
            format!("{cue:?}"),
            format!("{request:?}"),
            format!("{response:?}"),
        ] {
            assert!(!output.contains(secret));
        }
    }

    #[test]
    fn document_analysis_rejects_every_empty_required_text_field() {
        let valid = DocumentAnalysis {
            summary: "summary".into(),
            characters: vec![AnalysisCharacter {
                name: "name".into(),
                description: "description".into(),
            }],
            glossary: vec![AnalysisGlossaryEntry {
                source: "source".into(),
                target: "target".into(),
                note: "note".into(),
            }],
        };
        assert_eq!(valid.validate(), Ok(()));

        let mut empty_summary = valid.clone();
        empty_summary.summary = " \n ".into();
        let mut empty_character = valid.clone();
        empty_character.characters[0].description.clear();
        let mut empty_glossary = valid;
        empty_glossary.glossary[0].target.clear();

        assert_eq!(
            empty_summary.validate(),
            Err(DocumentAnalysisValidationError::EmptySummary)
        );
        assert_eq!(
            empty_character.validate(),
            Err(DocumentAnalysisValidationError::EmptyCharacterField)
        );
        assert_eq!(
            empty_glossary.validate(),
            Err(DocumentAnalysisValidationError::EmptyGlossaryField)
        );
    }

    #[test]
    fn derived_debug_would_really_expose_the_guard_sentinel() {
        #[derive(Debug)]
        #[allow(dead_code)] // read only by the derive, which is the point
        struct LeakyCue {
            text: String,
        }

        let secret = "PRIVATE SUBTITLE DIALOGUE";
        let output = format!(
            "{:?}",
            LeakyCue {
                text: secret.into()
            }
        );
        assert!(output.contains(secret));
    }

    #[test]
    fn a_panicking_progress_sink_poison_closes_the_gate() {
        struct PanickingSink;

        impl TranslationProgressSink for PanickingSink {
            fn on_progress(&self, _progress: TranslationProgress) {
                panic!("intentional callback panic");
            }
        }

        let call = TranslationCall::new(Arc::new(PanickingSink));
        let delivery = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = call.progress(TranslationProgress {
                phase: TranslationProgressPhase::Preparing,
                done: 0,
                total: 1,
            });
        }));
        assert!(delivery.is_err());
        assert_eq!(call.checkpoint(), Err(TranslationProviderError::Cancelled));
    }

    #[test]
    fn commit_after_cancel_never_runs_the_closure() {
        let call = TranslationCall::without_progress();
        call.cancel();

        let mut ran = false;
        let result = call.commit(|| {
            ran = true;
        });

        assert_eq!(result, Err(TranslationProviderError::Cancelled));
        assert!(!ran, "commit closure must not run once the gate is closed");
    }

    #[test]
    fn cancel_waits_for_an_in_flight_commit_and_blocks_every_commit_after() {
        use std::sync::mpsc;
        use std::sync::Mutex as StdMutex;
        use std::thread;
        use std::time::Duration;

        let call = TranslationCall::without_progress();
        let (commit_started_tx, commit_started_rx) = mpsc::channel::<()>();
        let (release_commit_tx, release_commit_rx) = mpsc::channel::<()>();
        // Independent of `TranslationCall`'s own gate lock, so recording an
        // event never itself contends on the lock under test.
        let events = Arc::new(StdMutex::new(Vec::<&'static str>::new()));

        let committer_call = call.clone();
        let committer_events = events.clone();
        let committer = thread::spawn(move || {
            committer_call.commit(|| {
                committer_events.lock().unwrap().push("commit_start");
                commit_started_tx.send(()).expect("signal commit start");
                release_commit_rx
                    .recv_timeout(Duration::from_secs(5))
                    .expect("released after cancel() has started waiting");
                committer_events.lock().unwrap().push("commit_end");
            })
        });

        commit_started_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("commit reported it started");

        // `cancel()` must block on the gate lock the running commit holds,
        // so it cannot record `cancel_done` until the commit above does.
        let canceller_call = call.clone();
        let canceller_events = events.clone();
        let canceller = thread::spawn(move || {
            canceller_call.cancel();
            canceller_events.lock().unwrap().push("cancel_done");
        });

        // Give the canceller a chance to reach (and block on) the lock
        // before the commit is allowed to finish.
        thread::sleep(Duration::from_millis(50));
        release_commit_tx.send(()).expect("release the commit");

        committer
            .join()
            .expect("committer thread")
            .expect("commit succeeded");
        canceller.join().expect("canceller thread");

        let recorded = events.lock().unwrap().clone();
        assert_eq!(recorded, vec!["commit_start", "commit_end", "cancel_done"]);

        assert!(call.is_cancelled());
        assert_eq!(call.commit(|| ()), Err(TranslationProviderError::Cancelled));
    }
}
