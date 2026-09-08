//! Strict, local validation of an untrusted translated block (NEN-091,
//! ADR-0016).
//!
//! Provider DTOs contain only cue IDs and translated text. The source
//! document remains authoritative for timing, and a [`ValidatedBlock`] is
//! produced only when every expected output cue has one trustworthy,
//! non-empty translation. A failed inspection may retain acceptable cues for
//! the in-crate repair pipeline, but never exposes them as a validated block.

use crate::blocks::TranslationBlock;
use nen_domain::subtitle::{CueId, SubtitleDocument, TimeSpan};
use nen_ports::translation::{TranslatedCue, TranslationResponse};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// One translated cue whose text passed local validation and whose timing was
/// copied from the source document.
#[derive(Clone, PartialEq, Eq)]
pub struct ValidatedCue {
    cue_id: CueId,
    span: TimeSpan,
    text: String,
}

impl ValidatedCue {
    pub const fn cue_id(&self) -> CueId {
        self.cue_id
    }

    pub const fn span(&self) -> TimeSpan {
        self.span
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl fmt::Debug for ValidatedCue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidatedCue")
            .field("cue_id", &self.cue_id)
            .field("span", &self.span)
            .field("text_len", &self.text.chars().count())
            .finish()
    }
}

/// A completely validated block, normalized to the source block's output
/// order. This type cannot be constructed outside this module.
#[derive(Clone, PartialEq, Eq)]
pub struct ValidatedBlock {
    block_index: usize,
    cues: Vec<ValidatedCue>,
}

impl ValidatedBlock {
    pub const fn block_index(&self) -> usize {
        self.block_index
    }

    pub fn cues(&self) -> &[ValidatedCue] {
        &self.cues
    }
}

impl fmt::Debug for ValidatedBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidatedBlock")
            .field("block_index", &self.block_index)
            .field("cue_count", &self.cues.len())
            .field("cues", &self.cues)
            .finish()
    }
}

/// A structural reason why an untrusted response cannot be accepted as-is.
/// Variants intentionally contain no translated text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockViolation {
    CountMismatch { expected: usize, received: usize },
    MissingCue { cue_id: CueId },
    DuplicateCue { cue_id: CueId },
    UnknownCue { cue_id: CueId },
    EmptyText { cue_id: CueId },
}

/// The validation report consumed by the repair pipeline.
pub struct BlockValidationError {
    block_index: usize,
    violations: Vec<BlockViolation>,
    repair_cue_ids: Vec<CueId>,
    accepted_cues: Vec<ValidatedCue>,
}

impl BlockValidationError {
    pub const fn block_index(&self) -> usize {
        self.block_index
    }

    pub fn violations(&self) -> &[BlockViolation] {
        &self.violations
    }

    /// Expected cue IDs that need a targeted repair, in source block order.
    pub fn repair_cue_ids(&self) -> &[CueId] {
        &self.repair_cue_ids
    }

    /// Partial cues are intentionally crate-private. NEN-092 may merge them
    /// into a repair attempt, but callers can never publish them as a block.
    #[allow(dead_code)]
    pub(crate) fn accepted_cues(&self) -> &[ValidatedCue] {
        &self.accepted_cues
    }
}

impl fmt::Debug for BlockValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockValidationError")
            .field("block_index", &self.block_index)
            .field("violations", &self.violations)
            .field("repair_cue_ids", &self.repair_cue_ids)
            .field("accepted_cue_count", &self.accepted_cues.len())
            .finish()
    }
}

impl fmt::Display for BlockValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "translation block {} failed validation ({} violations)",
            self.block_index,
            self.violations.len()
        )
    }
}

impl std::error::Error for BlockValidationError {}

struct ExpectedCue {
    cue_id: CueId,
    span: TimeSpan,
}

/// Validate and normalize one provider response against a block's expected
/// output set. The provider response is borrowed so a caller can inspect it
/// again while preparing a repair, but only a successful result is a
/// [`ValidatedBlock`].
pub fn validate_block(
    document: &SubtitleDocument,
    block: &TranslationBlock,
    response: &TranslationResponse,
) -> Result<ValidatedBlock, BlockValidationError> {
    let expected = expected_cues(document, block);
    let expected_ids: BTreeSet<CueId> = expected.iter().map(|cue| cue.cue_id).collect();
    let mut by_id: BTreeMap<CueId, Vec<&TranslatedCue>> = BTreeMap::new();

    for cue in &response.cues {
        by_id.entry(cue.cue_id).or_default().push(cue);
    }

    let mut violations = Vec::new();
    if expected.len() != response.cues.len() {
        violations.push(BlockViolation::CountMismatch {
            expected: expected.len(),
            received: response.cues.len(),
        });
    }

    for cue_id in by_id
        .keys()
        .copied()
        .filter(|id| !expected_ids.contains(id))
    {
        violations.push(BlockViolation::UnknownCue { cue_id });
    }

    let mut repair_cue_ids = Vec::new();
    let mut accepted_by_id = BTreeMap::new();

    for expected_cue in &expected {
        match by_id.get(&expected_cue.cue_id) {
            None => {
                violations.push(BlockViolation::MissingCue {
                    cue_id: expected_cue.cue_id,
                });
                repair_cue_ids.push(expected_cue.cue_id);
            }
            Some(cues) if cues.len() != 1 => {
                violations.push(BlockViolation::DuplicateCue {
                    cue_id: expected_cue.cue_id,
                });
                repair_cue_ids.push(expected_cue.cue_id);
            }
            Some(cues) => {
                let Some(cue) = cues.first().copied() else {
                    continue;
                };
                let text = cue.text.trim();
                if text.is_empty() {
                    violations.push(BlockViolation::EmptyText {
                        cue_id: expected_cue.cue_id,
                    });
                    repair_cue_ids.push(expected_cue.cue_id);
                } else {
                    accepted_by_id.insert(expected_cue.cue_id, text.to_owned());
                }
            }
        }
    }

    let accepted_cues = expected
        .iter()
        .filter_map(|expected_cue| {
            accepted_by_id
                .remove(&expected_cue.cue_id)
                .map(|text| ValidatedCue {
                    cue_id: expected_cue.cue_id,
                    span: expected_cue.span,
                    text,
                })
        })
        .collect::<Vec<_>>();

    if violations.is_empty() {
        Ok(ValidatedBlock {
            block_index: block.index(),
            cues: accepted_cues,
        })
    } else {
        Err(BlockValidationError {
            block_index: block.index(),
            violations,
            repair_cue_ids,
            accepted_cues,
        })
    }
}

fn expected_cues(document: &SubtitleDocument, block: &TranslationBlock) -> Vec<ExpectedCue> {
    block
        .output_positions()
        .filter_map(|position| document.cues().get(position))
        .map(|cue| ExpectedCue {
            cue_id: cue.id(),
            span: cue.span(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{BlockLayout, BlockLayoutConfig};
    use nen_domain::subtitle::{Cue, CueId, TimeSpan};

    fn document() -> SubtitleDocument {
        SubtitleDocument::new(
            (1..=3)
                .map(|id| {
                    Cue::new(
                        CueId::new(id),
                        TimeSpan::new(id * 1_000, id * 1_000 + 900).expect("valid span"),
                        vec![format!("Source {id}")],
                    )
                })
                .collect(),
        )
    }

    fn response(cues: Vec<(u32, &str)>) -> TranslationResponse {
        TranslationResponse {
            cues: cues
                .into_iter()
                .map(|(cue_id, text)| TranslatedCue {
                    cue_id: CueId::new(cue_id),
                    text: text.into(),
                })
                .collect(),
        }
    }

    fn block(document: &SubtitleDocument) -> TranslationBlock {
        let layout = BlockLayout::of(document, BlockLayoutConfig::new(30, 1).expect("config"))
            .expect("layout");
        layout.blocks()[0].clone()
    }

    #[test]
    fn valid_response_is_normalized_and_copies_source_spans() {
        let document = document();
        let block = block(&document);
        let result = validate_block(
            &document,
            &block,
            &response(vec![(3, "  Üç  "), (1, " Bir "), (2, "İki")]),
        )
        .expect("valid response");

        assert_eq!(
            result
                .cues()
                .iter()
                .map(ValidatedCue::cue_id)
                .collect::<Vec<_>>(),
            vec![CueId::new(1), CueId::new(2), CueId::new(3)]
        );
        assert_eq!(result.cues()[0].text(), "Bir");
        assert_eq!(result.cues()[1].span(), document.cues()[1].span());
        assert_eq!(result.cues()[2].span(), document.cues()[2].span());
    }

    #[test]
    fn missing_and_unknown_ids_produce_a_deterministic_repair_set() {
        let document = document();
        let block = block(&document);
        let error = validate_block(
            &document,
            &block,
            &response(vec![(1, "one"), (99, "extra")]),
        )
        .expect_err("invalid response");

        assert_eq!(error.repair_cue_ids(), &[CueId::new(2), CueId::new(3)]);
        assert!(error.violations().contains(&BlockViolation::CountMismatch {
            expected: 3,
            received: 2
        }));
        assert!(error.violations().contains(&BlockViolation::UnknownCue {
            cue_id: CueId::new(99)
        }));
        assert!(error.violations().contains(&BlockViolation::MissingCue {
            cue_id: CueId::new(2)
        }));
    }

    #[test]
    fn duplicate_and_empty_ids_are_not_silently_repaired() {
        let document = document();
        let block = block(&document);
        let error = validate_block(
            &document,
            &block,
            &response(vec![(1, "one"), (1, "another"), (2, "  "), (3, "three")]),
        )
        .expect_err("invalid response");

        assert_eq!(error.repair_cue_ids(), &[CueId::new(1), CueId::new(2)]);
        assert!(error.violations().contains(&BlockViolation::DuplicateCue {
            cue_id: CueId::new(1)
        }));
        assert!(error.violations().contains(&BlockViolation::EmptyText {
            cue_id: CueId::new(2)
        }));
    }
}
