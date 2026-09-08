//! Bounded block repair for untrusted translation responses (NEN-092).
//!
//! The initial provider response is validated locally. Missing, duplicated or
//! empty expected cue IDs are eligible for at most two targeted repairs. If
//! validation still fails (or no targeted set can be derived), one clean
//! full-block retry is attempted. Only a completely validated block can leave
//! this module.

use crate::blocks::TranslationBlock;
use crate::validation::{validate_block, BlockViolation, ValidatedBlock};
use nen_domain::subtitle::SubtitleDocument;
use nen_ports::translation::{
    TranslatedCue, TranslationCall, TranslationProvider, TranslationProviderError,
    TranslationRequest, TranslationResponse,
};
use std::fmt;

/// Maximum number of targeted repairs after the initial provider call.
pub const MAX_TARGETED_REPAIRS: usize = 2;

/// Maximum number of clean full-block retries after targeted repairs.
pub const MAX_FULL_BLOCK_RETRIES: usize = 1;

/// Failure of a block translation attempt.
///
/// This type intentionally contains no translated text or partial cue list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockTranslationError {
    Provider(TranslationProviderError),
    ValidationBudgetExhausted {
        block_index: usize,
        targeted_repairs: usize,
        full_block_retries: usize,
        violations: Vec<BlockViolation>,
    },
}

impl fmt::Display for BlockTranslationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provider(error) => error.fmt(f),
            Self::ValidationBudgetExhausted {
                block_index,
                targeted_repairs,
                full_block_retries,
                violations,
            } => write!(
                f,
                "translation block {block_index} exhausted validation budget ({targeted_repairs} targeted repairs, {full_block_retries} full-block retries, {count} violations)",
                count = violations.len()
            ),
        }
    }
}

impl std::error::Error for BlockTranslationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Provider(error) => Some(error),
            Self::ValidationBudgetExhausted { .. } => None,
        }
    }
}

impl BlockTranslationError {
    pub const fn block_index(&self) -> Option<usize> {
        match self {
            Self::Provider(_) => None,
            Self::ValidationBudgetExhausted { block_index, .. } => Some(*block_index),
        }
    }

    pub const fn targeted_repairs(&self) -> Option<usize> {
        match self {
            Self::Provider(_) => None,
            Self::ValidationBudgetExhausted {
                targeted_repairs, ..
            } => Some(*targeted_repairs),
        }
    }

    pub const fn full_block_retries(&self) -> Option<usize> {
        match self {
            Self::Provider(_) => None,
            Self::ValidationBudgetExhausted {
                full_block_retries, ..
            } => Some(*full_block_retries),
        }
    }

    pub fn violations(&self) -> Option<&[BlockViolation]> {
        match self {
            Self::Provider(_) => None,
            Self::ValidationBudgetExhausted { violations, .. } => Some(violations),
        }
    }
}

/// Translate and locally validate one block with the fixed repair budget.
pub fn translate_block_with_repair(
    provider: &dyn TranslationProvider,
    document: &SubtitleDocument,
    block: &TranslationBlock,
    request: &TranslationRequest,
    call: &TranslationCall,
) -> Result<ValidatedBlock, BlockTranslationError> {
    let response = provider
        .translate(request, call)
        .map_err(BlockTranslationError::Provider)?;

    let mut error = match validate_block(document, block, &response) {
        Ok(validated) => return Ok(validated),
        Err(error) => error,
    };

    let mut targeted_repairs = 0usize;
    while targeted_repairs < MAX_TARGETED_REPAIRS {
        let repair_ids = error.repair_cue_ids();
        if repair_ids.is_empty() {
            break;
        }

        let repair_request = request_for_ids(request, repair_ids);
        let repair_response = provider
            .translate(&repair_request, &call.fork())
            .map_err(BlockTranslationError::Provider)?;
        let merged = merge_accepted_and_response(error.accepted_cues(), &repair_response);
        targeted_repairs += 1;

        error = match validate_block(document, block, &merged) {
            Ok(validated) => return Ok(validated),
            Err(error) => error,
        };
    }

    let full_block_retries = MAX_FULL_BLOCK_RETRIES;
    let retry_response = provider
        .translate(request, &call.fork())
        .map_err(BlockTranslationError::Provider)?;
    match validate_block(document, block, &retry_response) {
        Ok(validated) => Ok(validated),
        Err(final_error) => Err(BlockTranslationError::ValidationBudgetExhausted {
            block_index: block.index(),
            targeted_repairs,
            full_block_retries,
            violations: final_error.violations().to_vec(),
        }),
    }
}

fn request_for_ids(
    request: &TranslationRequest,
    cue_ids: &[nen_domain::subtitle::CueId],
) -> TranslationRequest {
    let mut repair_request = request.clone();
    repair_request.output_cue_ids = cue_ids.to_vec();
    repair_request
}

fn merge_accepted_and_response(
    accepted: &[crate::validation::ValidatedCue],
    response: &TranslationResponse,
) -> TranslationResponse {
    let mut cues = accepted
        .iter()
        .map(|cue| TranslatedCue {
            cue_id: cue.cue_id(),
            text: cue.text().to_owned(),
        })
        .collect::<Vec<_>>();
    cues.extend(response.cues.iter().cloned());
    TranslationResponse { cues }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{BlockLayout, BlockLayoutConfig};
    use nen_domain::source::LanguageTag;
    use nen_domain::subtitle::{Cue, CueId, TimeSpan};
    use nen_ports::translation::{
        TranslationCue, TranslationProgress, TranslationProgressPhase, TranslationProviderIdentity,
    };
    use std::sync::{Arc, Mutex};

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

    fn block(document: &SubtitleDocument) -> TranslationBlock {
        BlockLayout::of(document, BlockLayoutConfig::new(30, 1).expect("config"))
            .expect("layout")
            .blocks()[0]
            .clone()
    }

    fn request() -> TranslationRequest {
        TranslationRequest {
            source_language: LanguageTag::parse("en").expect("language"),
            target_language: LanguageTag::parse("tr").expect("language"),
            context_cues: vec![
                TranslationCue {
                    cue_id: CueId::new(1),
                    text: "Source 1".into(),
                },
                TranslationCue {
                    cue_id: CueId::new(2),
                    text: "Source 2".into(),
                },
                TranslationCue {
                    cue_id: CueId::new(3),
                    text: "Source 3".into(),
                },
            ],
            output_cue_ids: vec![CueId::new(1), CueId::new(2), CueId::new(3)],
            context_terms: vec!["Name".into()],
        }
    }

    #[derive(Clone)]
    struct ScriptedProvider {
        responses: Arc<Mutex<Vec<TranslationResponse>>>,
        requests: Arc<Mutex<Vec<TranslationRequest>>>,
        calls: Arc<Mutex<usize>>,
        provider_error: Option<TranslationProviderError>,
    }

    impl ScriptedProvider {
        fn new(responses: Vec<TranslationResponse>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses)),
                requests: Arc::new(Mutex::new(Vec::new())),
                calls: Arc::new(Mutex::new(0)),
                provider_error: None,
            }
        }

        fn always_invalid() -> Self {
            Self::new(vec![
                TranslationResponse {
                    cues: vec![
                        TranslatedCue {
                            cue_id: CueId::new(1),
                            text: "one".into(),
                        },
                        TranslatedCue {
                            cue_id: CueId::new(3),
                            text: "three".into(),
                        },
                    ],
                },
                TranslationResponse { cues: Vec::new() },
            ])
        }

        fn call_count(&self) -> usize {
            *self.calls.lock().unwrap()
        }

        fn requested_ids(&self) -> Vec<Vec<CueId>> {
            self.requests
                .lock()
                .unwrap()
                .iter()
                .map(|request| request.output_cue_ids.clone())
                .collect()
        }

        fn requests(&self) -> Vec<TranslationRequest> {
            self.requests.lock().unwrap().clone()
        }
    }

    impl TranslationProvider for ScriptedProvider {
        fn identity(&self) -> TranslationProviderIdentity {
            TranslationProviderIdentity::new("test", "scripted").expect("identity")
        }

        fn translate(
            &self,
            request: &TranslationRequest,
            call: &TranslationCall,
        ) -> Result<TranslationResponse, TranslationProviderError> {
            *self.calls.lock().unwrap() += 1;
            self.requests.lock().unwrap().push(request.clone());
            if let Some(error) = self.provider_error {
                return Err(error);
            }
            call.progress(TranslationProgress {
                phase: TranslationProgressPhase::Preparing,
                done: 0,
                total: request.output_cue_ids.len() as u32,
            })?;
            let response = {
                let mut responses = self.responses.lock().unwrap();
                if responses.len() > 1 {
                    responses.remove(0)
                } else {
                    responses
                        .last()
                        .cloned()
                        .unwrap_or(TranslationResponse { cues: Vec::new() })
                }
            };
            call.progress(TranslationProgress {
                phase: TranslationProgressPhase::Finalizing,
                done: request.output_cue_ids.len() as u32,
                total: request.output_cue_ids.len() as u32,
            })?;
            call.finish(response)
        }
    }

    fn valid_response() -> TranslationResponse {
        TranslationResponse {
            cues: vec![
                TranslatedCue {
                    cue_id: CueId::new(1),
                    text: "one".into(),
                },
                TranslatedCue {
                    cue_id: CueId::new(2),
                    text: "two".into(),
                },
                TranslatedCue {
                    cue_id: CueId::new(3),
                    text: "three".into(),
                },
            ],
        }
    }

    #[test]
    fn valid_initial_response_uses_no_repair() {
        let provider = ScriptedProvider::new(vec![valid_response()]);
        let result = translate_block_with_repair(
            &provider,
            &document(),
            &block(&document()),
            &request(),
            &TranslationCall::without_progress(),
        )
        .expect("valid block");
        assert_eq!(result.cues().len(), 3);
        assert_eq!(provider.call_count(), 1);
        assert_eq!(
            provider.requested_ids(),
            vec![vec![CueId::new(1), CueId::new(2), CueId::new(3)]]
        );
    }

    #[test]
    fn targeted_repair_requests_only_the_missing_cue_and_merges_it() {
        let provider = ScriptedProvider::new(vec![
            TranslationResponse {
                cues: vec![
                    TranslatedCue {
                        cue_id: CueId::new(1),
                        text: "one".into(),
                    },
                    TranslatedCue {
                        cue_id: CueId::new(3),
                        text: "three".into(),
                    },
                ],
            },
            TranslationResponse {
                cues: vec![TranslatedCue {
                    cue_id: CueId::new(2),
                    text: "two".into(),
                }],
            },
        ]);
        let result = translate_block_with_repair(
            &provider,
            &document(),
            &block(&document()),
            &request(),
            &TranslationCall::without_progress(),
        )
        .expect("repair succeeds");
        assert_eq!(result.cues().len(), 3);
        assert_eq!(provider.call_count(), 2);
        assert_eq!(provider.requested_ids()[1], vec![CueId::new(2)]);
        let targeted = &provider.requests()[1];
        assert_eq!(targeted.source_language, request().source_language);
        assert_eq!(targeted.target_language, request().target_language);
        assert_eq!(targeted.context_cues, request().context_cues);
        assert_eq!(targeted.context_terms, request().context_terms);
    }

    #[test]
    fn unknown_only_response_skips_targeted_repair_and_uses_full_retry() {
        let provider = ScriptedProvider::new(vec![
            TranslationResponse {
                cues: vec![TranslatedCue {
                    cue_id: CueId::new(99),
                    text: "extra".into(),
                }],
            },
            valid_response(),
        ]);
        translate_block_with_repair(
            &provider,
            &document(),
            &block(&document()),
            &request(),
            &TranslationCall::without_progress(),
        )
        .expect("full retry succeeds");
        assert_eq!(provider.call_count(), 2);
        assert_eq!(
            provider.requested_ids(),
            vec![
                vec![CueId::new(1), CueId::new(2), CueId::new(3)],
                vec![CueId::new(1), CueId::new(2), CueId::new(3)],
            ]
        );
    }

    #[test]
    fn exhausted_budget_is_bounded_and_never_returns_partial_output() {
        let provider = ScriptedProvider::always_invalid();
        let error = translate_block_with_repair(
            &provider,
            &document(),
            &block(&document()),
            &request(),
            &TranslationCall::without_progress(),
        )
        .expect_err("invalid provider must exhaust budget");
        assert_eq!(provider.call_count(), 4);
        assert_eq!(
            provider.requested_ids(),
            vec![
                vec![CueId::new(1), CueId::new(2), CueId::new(3)],
                vec![CueId::new(2)],
                vec![CueId::new(2)],
                vec![CueId::new(1), CueId::new(2), CueId::new(3)],
            ]
        );
        assert_eq!(error.targeted_repairs(), Some(2));
        assert_eq!(error.full_block_retries(), Some(1));
        assert!(error.violations().is_some());
    }

    #[test]
    fn provider_failure_is_not_retried_as_validation() {
        let mut provider = ScriptedProvider::new(Vec::new());
        provider.provider_error = Some(TranslationProviderError::Transient);
        let error = translate_block_with_repair(
            &provider,
            &document(),
            &block(&document()),
            &request(),
            &TranslationCall::without_progress(),
        )
        .expect_err("provider error");
        assert_eq!(
            error,
            BlockTranslationError::Provider(TranslationProviderError::Transient)
        );
        assert_eq!(provider.call_count(), 1);
    }

    #[test]
    fn repair_error_surfaces_never_leak_translated_text() {
        let provider = ScriptedProvider::always_invalid();
        let error = translate_block_with_repair(
            &provider,
            &document(),
            &block(&document()),
            &request(),
            &TranslationCall::without_progress(),
        )
        .expect_err("invalid provider");
        let secret = "unknown";
        assert!(!format!("{error:?}").contains(secret));
        assert!(!format!("{error}").contains(secret));

        let mut request = request();
        request.context_cues[0].text = secret.into();
        let response = TranslationResponse {
            cues: vec![TranslatedCue {
                cue_id: CueId::new(1),
                text: secret.into(),
            }],
        };
        assert!(!format!("{request:?}").contains(secret));
        assert!(!format!("{response:?}").contains(secret));
    }
}
