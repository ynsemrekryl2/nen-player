use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use nen_domain::source::LanguageTag;
use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_ports::persistence::{CacheKey, ResumeRecord, ResumeStore, ResumeStoreError};
use nen_ports::translation::{
    AnalysisCharacter, AnalysisGlossaryEntry, DocumentAnalysis, DocumentAnalysisRequest,
    TranslatedCue, TranslationCall, TranslationMode, TranslationProvider, TranslationProviderError,
    TranslationProviderIdentity, TranslationRequest, TranslationResponse,
};
use nen_translate::blocks::{BlockLayout, BlockLayoutConfig};
use nen_translate::checkpoint::{translate_resumable, ResumableTranslationError, TranslationPlan};
use nen_translate::repair::BlockTranslationError;

fn document(cue_count: u32) -> SubtitleDocument {
    SubtitleDocument::new(
        (1..=cue_count)
            .map(|id| {
                Cue::new(
                    CueId::new(id),
                    TimeSpan::new(id * 1_000, id * 1_000 + 500).expect("valid span"),
                    vec![format!("Source dialogue {id}")],
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
        source_language: LanguageTag::parse("en").expect("source"),
        target_language: LanguageTag::parse("tr").expect("target"),
        context_terms: vec!["RecurringName".into()],
    }
}

fn analysis(label: &str) -> DocumentAnalysis {
    DocumentAnalysis {
        summary: format!("Whole-document summary {label}"),
        characters: vec![AnalysisCharacter {
            name: "Speaker".into(),
            description: "Uses an informal register".into(),
        }],
        glossary: vec![AnalysisGlossaryEntry {
            source: "RecurringName".into(),
            target: "YinelenenAd".into(),
            note: "Keep this rendering consistent".into(),
        }],
    }
}

#[derive(Default)]
struct MemoryResumeStore {
    record: Mutex<Option<ResumeRecord>>,
}

impl MemoryResumeStore {
    fn snapshot(&self) -> Option<ResumeRecord> {
        self.record.lock().expect("resume lock").clone()
    }
}

impl ResumeStore for MemoryResumeStore {
    fn load(&self, key: CacheKey) -> Result<Option<ResumeRecord>, ResumeStoreError> {
        Ok(self
            .record
            .lock()
            .expect("resume lock")
            .clone()
            .filter(|record| record.cache_key == key))
    }

    fn save(&self, record: &ResumeRecord) -> Result<(), ResumeStoreError> {
        *self.record.lock().expect("resume lock") = Some(record.clone());
        Ok(())
    }

    fn delete(&self, key: CacheKey) -> Result<(), ResumeStoreError> {
        let mut record = self.record.lock().expect("resume lock");
        if record
            .as_ref()
            .is_some_and(|record| record.cache_key == key)
        {
            *record = None;
        }
        Ok(())
    }
}

struct RecordingProvider {
    analysis_result: Result<DocumentAnalysis, TranslationProviderError>,
    fail_translation_at: Option<usize>,
    analysis_calls: AtomicUsize,
    translation_calls: AtomicUsize,
    analysis_requests: Mutex<Vec<DocumentAnalysisRequest>>,
    translation_requests: Mutex<Vec<TranslationRequest>>,
    events: Mutex<Vec<&'static str>>,
}

impl RecordingProvider {
    fn new(analysis_result: Result<DocumentAnalysis, TranslationProviderError>) -> Self {
        Self {
            analysis_result,
            fail_translation_at: None,
            analysis_calls: AtomicUsize::new(0),
            translation_calls: AtomicUsize::new(0),
            analysis_requests: Mutex::new(Vec::new()),
            translation_requests: Mutex::new(Vec::new()),
            events: Mutex::new(Vec::new()),
        }
    }

    fn failing_translation_at(mut self, index: usize) -> Self {
        self.fail_translation_at = Some(index);
        self
    }
}

impl TranslationProvider for RecordingProvider {
    fn identity(&self) -> TranslationProviderIdentity {
        TranslationProviderIdentity::new("test", "two-stage").expect("identity")
    }

    fn analyze_document(
        &self,
        request: &DocumentAnalysisRequest,
        call: &TranslationCall,
    ) -> Result<DocumentAnalysis, TranslationProviderError> {
        call.checkpoint()?;
        self.analysis_calls.fetch_add(1, Ordering::SeqCst);
        self.analysis_requests
            .lock()
            .expect("analysis requests")
            .push(request.clone());
        self.events.lock().expect("events").push("analysis");
        self.analysis_result.clone()
    }

    fn translate(
        &self,
        request: &TranslationRequest,
        call: &TranslationCall,
    ) -> Result<TranslationResponse, TranslationProviderError> {
        call.checkpoint()?;
        let call_index = self.translation_calls.fetch_add(1, Ordering::SeqCst);
        self.translation_requests
            .lock()
            .expect("translation requests")
            .push(request.clone());
        self.events.lock().expect("events").push("block");
        if self.fail_translation_at == Some(call_index) {
            return Err(TranslationProviderError::Cancelled);
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

#[test]
fn a_fresh_run_calls_analysis_once_before_blocks_and_reuses_it_byte_for_byte() {
    let document = document(65);
    let layout = layout(&document);
    let expected_analysis = analysis("fresh");
    let provider = RecordingProvider::new(Ok(expected_analysis.clone()));
    let store = MemoryResumeStore::default();
    let key = CacheKey::from_bytes([0x31; 32]);

    translate_resumable(
        &provider,
        &document,
        &layout,
        &plan(),
        &TranslationCall::without_progress(),
        key,
        &store,
    )
    .expect("run completes");

    assert_eq!(provider.analysis_calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        provider.translation_calls.load(Ordering::SeqCst),
        layout.blocks().len()
    );
    assert_eq!(provider.events.lock().expect("events")[0], "analysis");

    let analysis_requests = provider
        .analysis_requests
        .lock()
        .expect("analysis requests");
    assert_eq!(analysis_requests.len(), 1);
    assert_eq!(analysis_requests[0].transcript.len(), document.len());
    assert_eq!(analysis_requests[0].transcript[0].cue_id, CueId::new(1));
    assert_eq!(analysis_requests[0].transcript[0].text, "Source dialogue 1");
    assert_eq!(analysis_requests[0].context_terms, vec!["RecurringName"]);
    drop(analysis_requests);

    let requests = provider
        .translation_requests
        .lock()
        .expect("translation requests");
    for (index, request) in requests.iter().enumerate() {
        assert_eq!(request.analysis, expected_analysis);
        assert_eq!(request.mode, TranslationMode::Initial);
        assert_eq!(
            request.block_index,
            u32::try_from(index + 1).expect("small")
        );
        assert_eq!(
            request.block_count,
            u32::try_from(layout.blocks().len()).expect("small")
        );
    }

    let persisted = store.snapshot().expect("resume snapshot");
    assert_eq!(persisted.analysis, expected_analysis);
    assert_eq!(persisted.blocks.len(), layout.blocks().len());
}

#[test]
fn restart_reuses_persisted_analysis_and_skips_the_checkpointed_prefix() {
    let document = document(65);
    let layout = layout(&document);
    let persisted_analysis = analysis("persisted");
    let store = MemoryResumeStore::default();
    let key = CacheKey::from_bytes([0x42; 32]);
    let first = RecordingProvider::new(Ok(persisted_analysis.clone())).failing_translation_at(1);

    let error = translate_resumable(
        &first,
        &document,
        &layout,
        &plan(),
        &TranslationCall::without_progress(),
        key,
        &store,
    )
    .expect_err("second block is interrupted");
    assert_eq!(
        error,
        ResumableTranslationError::Block(BlockTranslationError::Provider(
            TranslationProviderError::Cancelled
        ))
    );
    let interrupted = store.snapshot().expect("interrupted snapshot");
    assert_eq!(interrupted.analysis, persisted_analysis);
    assert_eq!(interrupted.blocks.len(), 1);

    let resumed = RecordingProvider::new(Ok(analysis("must-not-be-used")));
    translate_resumable(
        &resumed,
        &document,
        &layout,
        &plan(),
        &TranslationCall::without_progress(),
        key,
        &store,
    )
    .expect("resumed run completes");

    assert_eq!(resumed.analysis_calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        resumed.translation_calls.load(Ordering::SeqCst),
        layout.blocks().len() - 1
    );
    assert!(resumed
        .translation_requests
        .lock()
        .expect("translation requests")
        .iter()
        .all(|request| request.analysis == persisted_analysis));
}

#[test]
fn analysis_failure_or_invalid_output_starts_no_block_and_persists_nothing() {
    let document = document(3);
    let layout = layout(&document);

    for (index, analysis_result) in [
        Err(TranslationProviderError::Permanent),
        Ok(DocumentAnalysis {
            summary: "   ".into(),
            characters: Vec::new(),
            glossary: Vec::new(),
        }),
    ]
    .into_iter()
    .enumerate()
    {
        let provider = RecordingProvider::new(analysis_result);
        let store = MemoryResumeStore::default();
        let error = translate_resumable(
            &provider,
            &document,
            &layout,
            &plan(),
            &TranslationCall::without_progress(),
            CacheKey::from_bytes([u8::try_from(index + 1).expect("small"); 32]),
            &store,
        )
        .expect_err("analysis must stop the run");

        assert!(matches!(
            error,
            ResumableTranslationError::Analysis(TranslationProviderError::Permanent)
        ));
        assert_eq!(provider.translation_calls.load(Ordering::SeqCst), 0);
        assert!(store.snapshot().is_none());
    }
}
