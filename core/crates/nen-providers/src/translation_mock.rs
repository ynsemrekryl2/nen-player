//! Deterministic, network-free translation provider used by M5.

use nen_ports::translation::{
    TranslatedCue, TranslationCall, TranslationProgress, TranslationProgressPhase,
    TranslationProvider, TranslationProviderError, TranslationProviderIdentity, TranslationRequest,
    TranslationResponse,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

const PROVIDER_ID: &str = "nen-mock";
const MODEL_ID: &str = "deterministic-v1";

#[derive(Default)]
pub struct MockCallGate {
    state: Mutex<MockCallGateState>,
    changed: Condvar,
}

#[derive(Default)]
struct MockCallGateState {
    arrived: bool,
    released: bool,
}

impl MockCallGate {
    pub fn wait_until_arrived(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !state.arrived {
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    pub fn release(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.released = true;
        self.changed.notify_all();
    }

    fn arrive_and_wait(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.arrived = true;
        self.changed.notify_all();
        while !state.released {
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }
}

#[derive(Default)]
pub struct MockTranslationProvider {
    calls: AtomicUsize,
    call_gate: Option<Arc<MockCallGate>>,
}

impl MockTranslationProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_call_gate(call_gate: Arc<MockCallGate>) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            call_gate: Some(call_gate),
        }
    }

    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }
}

impl TranslationProvider for MockTranslationProvider {
    fn identity(&self) -> TranslationProviderIdentity {
        TranslationProviderIdentity::new(PROVIDER_ID, MODEL_ID)
            .expect("static mock identity is valid")
    }

    fn translate(
        &self,
        request: &TranslationRequest,
        call: &TranslationCall,
    ) -> Result<TranslationResponse, TranslationProviderError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        call.checkpoint()?;
        let total = u32::try_from(request.output_cue_ids.len())
            .map_err(|_| TranslationProviderError::Permanent)?;
        if total == 0 {
            return Err(TranslationProviderError::Permanent);
        }
        call.progress(TranslationProgress {
            phase: TranslationProgressPhase::Preparing,
            done: 0,
            total,
        })?;

        if let Some(gate) = self.call_gate.as_ref() {
            gate.arrive_and_wait();
        }

        let mut cues = Vec::with_capacity(request.output_cue_ids.len());
        for (position, cue_id) in request.output_cue_ids.iter().copied().enumerate() {
            call.checkpoint()?;
            let source = request
                .context_cues
                .iter()
                .find(|cue| cue.cue_id == cue_id)
                .ok_or(TranslationProviderError::Permanent)?;
            cues.push(TranslatedCue {
                cue_id,
                text: format!("[{}] {}", request.target_language.as_str(), source.text),
            });
            call.progress(TranslationProgress {
                phase: TranslationProgressPhase::Translating,
                done: u32::try_from(position + 1)
                    .map_err(|_| TranslationProviderError::Permanent)?,
                total,
            })?;
        }

        call.progress(TranslationProgress {
            phase: TranslationProgressPhase::Finalizing,
            done: total,
            total,
        })?;
        call.finish(TranslationResponse { cues })
    }
}
