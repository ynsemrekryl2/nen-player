//! Versioned wire format for incomplete translation resume snapshots.

use nen_domain::subtitle::CueId;
use nen_ports::persistence::{CacheKey, ResumeBlock, ResumeCue, ResumeRecord, ResumeStoreError};
use nen_ports::translation::{AnalysisCharacter, AnalysisGlossaryEntry, DocumentAnalysis};
use serde::{Deserialize, Serialize};

const FORMAT_VERSION: u32 = 2;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRecord {
    format_version: u32,
    cache_key: String,
    total_blocks: u32,
    analysis: WireAnalysis,
    blocks: Vec<WireBlock>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireAnalysis {
    summary: String,
    characters: Vec<WireCharacter>,
    glossary: Vec<WireGlossaryEntry>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireCharacter {
    name: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireGlossaryEntry {
    source: String,
    target: String,
    note: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireBlock {
    block_index: u32,
    cues: Vec<WireCue>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireCue {
    cue_id: u32,
    text: String,
}

pub(crate) fn serialize(record: &ResumeRecord) -> Result<Vec<u8>, ResumeStoreError> {
    record
        .analysis
        .validate()
        .map_err(|_| ResumeStoreError::Corrupt)?;
    let wire = WireRecord {
        format_version: FORMAT_VERSION,
        cache_key: record.cache_key.to_hex(),
        total_blocks: record.total_blocks,
        analysis: WireAnalysis {
            summary: record.analysis.summary.clone(),
            characters: record
                .analysis
                .characters
                .iter()
                .map(|character| WireCharacter {
                    name: character.name.clone(),
                    description: character.description.clone(),
                })
                .collect(),
            glossary: record
                .analysis
                .glossary
                .iter()
                .map(|entry| WireGlossaryEntry {
                    source: entry.source.clone(),
                    target: entry.target.clone(),
                    note: entry.note.clone(),
                })
                .collect(),
        },
        blocks: record
            .blocks
            .iter()
            .map(|block| WireBlock {
                block_index: block.block_index,
                cues: block
                    .cues
                    .iter()
                    .map(|cue| WireCue {
                        cue_id: cue.cue_id.get(),
                        text: cue.text.clone(),
                    })
                    .collect(),
            })
            .collect(),
    };
    serde_json::to_vec(&wire).map_err(|_| ResumeStoreError::Corrupt)
}

pub(crate) fn deserialize(bytes: &[u8]) -> Result<ResumeRecord, ResumeStoreError> {
    let wire: WireRecord = serde_json::from_slice(bytes).map_err(|_| ResumeStoreError::Corrupt)?;
    if wire.format_version != FORMAT_VERSION {
        return Err(ResumeStoreError::Corrupt);
    }
    let cache_key = CacheKey::from_hex(&wire.cache_key).map_err(|_| ResumeStoreError::Corrupt)?;
    let analysis = DocumentAnalysis {
        summary: wire.analysis.summary,
        characters: wire
            .analysis
            .characters
            .into_iter()
            .map(|character| AnalysisCharacter {
                name: character.name,
                description: character.description,
            })
            .collect(),
        glossary: wire
            .analysis
            .glossary
            .into_iter()
            .map(|entry| AnalysisGlossaryEntry {
                source: entry.source,
                target: entry.target,
                note: entry.note,
            })
            .collect(),
    };
    analysis.validate().map_err(|_| ResumeStoreError::Corrupt)?;
    Ok(ResumeRecord {
        cache_key,
        total_blocks: wire.total_blocks,
        analysis,
        blocks: wire
            .blocks
            .into_iter()
            .map(|block| ResumeBlock {
                block_index: block.block_index,
                cues: block
                    .cues
                    .into_iter()
                    .map(|cue| ResumeCue {
                        cue_id: CueId::new(cue.cue_id),
                        text: cue.text,
                    })
                    .collect(),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> ResumeRecord {
        ResumeRecord {
            cache_key: CacheKey::from_bytes([0x22; 32]),
            total_blocks: 2,
            analysis: DocumentAnalysis {
                summary: "Whole document summary".into(),
                characters: vec![AnalysisCharacter {
                    name: "Speaker".into(),
                    description: "Uses an informal register".into(),
                }],
                glossary: vec![AnalysisGlossaryEntry {
                    source: "The Hand".into(),
                    target: "El".into(),
                    note: "Recurring title".into(),
                }],
            },
            blocks: vec![ResumeBlock {
                block_index: 0,
                cues: vec![ResumeCue {
                    cue_id: CueId::new(1),
                    text: "Translated cue".into(),
                }],
            }],
        }
    }

    #[test]
    fn version_two_round_trips_the_validated_analysis_and_prefix() {
        let expected = record();
        let bytes = serialize(&expected).expect("serialize");
        assert_eq!(deserialize(&bytes).expect("deserialize"), expected);
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("JSON");
        assert_eq!(json["format_version"], FORMAT_VERSION);
        assert_eq!(json["analysis"]["summary"], "Whole document summary");
    }

    #[test]
    fn old_unknown_or_invalid_analysis_records_are_corrupt() {
        let old_v1 = format!(
            r#"{{"format_version":1,"cache_key":"{}","total_blocks":0,"blocks":[]}}"#,
            CacheKey::from_bytes([0x22; 32]).to_hex()
        );
        assert_eq!(
            deserialize(old_v1.as_bytes()),
            Err(ResumeStoreError::Corrupt)
        );

        let mut json: serde_json::Value =
            serde_json::from_slice(&serialize(&record()).expect("serialize")).expect("JSON");
        json["analysis"]["summary"] = serde_json::Value::String("  ".into());
        assert_eq!(
            deserialize(&serde_json::to_vec(&json).expect("JSON")),
            Err(ResumeStoreError::Corrupt)
        );

        json["analysis"]["summary"] = serde_json::Value::String("valid".into());
        json["analysis"]["unexpected"] = serde_json::Value::Bool(true);
        assert_eq!(
            deserialize(&serde_json::to_vec(&json).expect("JSON")),
            Err(ResumeStoreError::Corrupt)
        );
    }
}
