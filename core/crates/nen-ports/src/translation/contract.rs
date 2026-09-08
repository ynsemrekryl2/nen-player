//! Shared contract kit for every translation provider adapter.

use super::{
    TranslationCall, TranslationProgress, TranslationProgressPhase, TranslationProgressSink,
    TranslationProvider, TranslationProviderError, TranslationRequest,
};
use nen_domain::subtitle::CueId;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractViolation {
    CallFailed,
    ResponseCueIds,
    MissingProgress,
    ProgressBoundaries,
    Cancellation,
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

pub fn check(
    provider: &dyn TranslationProvider,
    request: &TranslationRequest,
) -> Result<(), Vec<ContractViolation>> {
    let mut violations = Vec::new();
    let sink = Arc::new(RecordingSink::default());
    let call = TranslationCall::new(sink.clone());

    match provider.translate(request, &call) {
        Ok(response) => {
            // Ordering is deliberately not part of the provider contract:
            // NEN-091 owns normalization of an otherwise valid response.
            let mut returned: Vec<CueId> = response.cues.iter().map(|cue| cue.cue_id).collect();
            let mut expected = request.output_cue_ids.clone();
            returned.sort_unstable();
            expected.sort_unstable();
            if returned != expected {
                violations.push(ContractViolation::ResponseCueIds);
            }
        }
        Err(_) => violations.push(ContractViolation::CallFailed),
    }

    let updates = sink
        .updates
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if updates.is_empty() {
        violations.push(ContractViolation::MissingProgress);
    } else if updates
        .first()
        .is_some_and(|first| first.phase != TranslationProgressPhase::Preparing || first.done != 0)
        || updates.last().is_some_and(|last| {
            last.phase != TranslationProgressPhase::Finalizing || last.done != last.total
        })
    {
        violations.push(ContractViolation::ProgressBoundaries);
    }
    drop(updates);

    let cancelled = TranslationCall::without_progress();
    cancelled.cancel();
    if provider.translate(request, &cancelled) != Err(TranslationProviderError::Cancelled) {
        violations.push(ContractViolation::Cancellation);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}
