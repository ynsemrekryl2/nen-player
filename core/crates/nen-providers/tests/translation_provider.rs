use nen_domain::source::LanguageTag;
use nen_domain::subtitle::CueId;
use nen_ports::translation::contract::{self, ContractViolation};
use nen_ports::translation::{
    TranslatedCue, TranslationCall, TranslationCue, TranslationProgress, TranslationProgressPhase,
    TranslationProgressSink, TranslationProvider, TranslationProviderError,
    TranslationProviderIdentity, TranslationRequest, TranslationResponse,
};
use nen_providers::translation_mock::{MockCallGate, MockTranslationProvider};
use std::sync::{Arc, Mutex};
use std::thread;

fn request() -> TranslationRequest {
    TranslationRequest {
        source_language: LanguageTag::parse("en").expect("source language"),
        target_language: LanguageTag::parse("tr").expect("target language"),
        context_cues: vec![
            TranslationCue {
                cue_id: CueId::new(10),
                text: "First private line".into(),
            },
            TranslationCue {
                cue_id: CueId::new(20),
                text: "Second private line".into(),
            },
        ],
        output_cue_ids: vec![CueId::new(10), CueId::new(20)],
        context_terms: vec!["PrivateName".into()],
    }
}

#[derive(Default)]
struct RecordingSink {
    updates: Mutex<Vec<TranslationProgress>>,
}

impl TranslationProgressSink for RecordingSink {
    fn on_progress(&self, progress: TranslationProgress) {
        self.updates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(progress);
    }
}

#[test]
fn mock_passes_the_shared_contract_kit() {
    contract::check(&MockTranslationProvider::new(), &request()).expect("contract passes");
}

#[test]
fn same_request_has_the_same_identity_response_and_progress() {
    fn run(
        provider: &MockTranslationProvider,
        request: &TranslationRequest,
    ) -> (TranslationResponse, Vec<TranslationProgress>) {
        let sink = Arc::new(RecordingSink::default());
        let response = provider
            .translate(request, &TranslationCall::new(sink.clone()))
            .expect("translation");
        let updates = sink
            .updates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        (response, updates)
    }

    let provider = MockTranslationProvider::new();
    let input = request();
    let first_identity = provider.identity();
    let first = run(&provider, &input);
    let second_identity = provider.identity();
    let second = run(&provider, &input);

    assert_eq!(first_identity, second_identity);
    assert_eq!(first, second);
    assert_eq!(provider.calls(), 2);
}

#[test]
fn cancelling_an_in_flight_call_delivers_no_late_result_or_progress() {
    let gate = Arc::new(MockCallGate::default());
    let provider = Arc::new(MockTranslationProvider::with_call_gate(gate.clone()));
    let input = request();
    let sink = Arc::new(RecordingSink::default());
    let call = TranslationCall::new(sink.clone());
    let worker_provider = provider.clone();
    let worker_call = call.clone();
    let worker = thread::spawn(move || worker_provider.translate(&input, &worker_call));

    gate.wait_until_arrived();
    let before_cancel = sink
        .updates
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .len();
    call.cancel();
    gate.release();

    assert_eq!(
        worker.join().expect("worker joins"),
        Err(TranslationProviderError::Cancelled)
    );
    assert_eq!(
        sink.updates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len(),
        before_cancel
    );
}

struct DefectProvider;

impl TranslationProvider for DefectProvider {
    fn identity(&self) -> TranslationProviderIdentity {
        TranslationProviderIdentity::new("defect", "missing-cue").expect("identity")
    }

    fn translate(
        &self,
        request: &TranslationRequest,
        call: &TranslationCall,
    ) -> Result<TranslationResponse, TranslationProviderError> {
        call.checkpoint()?;
        let total = u32::try_from(request.output_cue_ids.len())
            .map_err(|_| TranslationProviderError::Permanent)?;
        call.progress(TranslationProgress {
            phase: TranslationProgressPhase::Preparing,
            done: 0,
            total,
        })?;
        call.progress(TranslationProgress {
            phase: TranslationProgressPhase::Finalizing,
            done: total,
            total,
        })?;
        call.finish(TranslationResponse {
            cues: vec![TranslatedCue {
                cue_id: request.output_cue_ids[0],
                text: "incomplete".into(),
            }],
        })
    }
}

#[test]
fn contract_kit_goes_red_for_a_missing_cue_defect() {
    let violations = contract::check(&DefectProvider, &request()).expect_err("defect is caught");
    assert!(violations.contains(&ContractViolation::ResponseCueIds));
}

#[test]
fn request_response_and_errors_never_print_provider_payloads() {
    let secret = "First private line";
    let input = request();
    let identity =
        TranslationProviderIdentity::new(secret, secret).expect("non-empty sentinel identity");
    let response = MockTranslationProvider::new()
        .translate(&input, &TranslationCall::without_progress())
        .expect("translation");

    assert!(!format!("{identity:?}").contains(secret));
    assert!(!format!("{input:?}").contains(secret));
    assert!(!format!("{response:?}").contains(secret));
    assert!(!format!("{:?}", response.cues[0]).contains(secret));
    for error in [
        TranslationProviderError::Cancelled,
        TranslationProviderError::Transient,
        TranslationProviderError::Permanent,
    ] {
        assert!(!format!("{error:?} {error}").contains(secret));
    }
}
