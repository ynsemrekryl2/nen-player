//! Versioned wire format for incomplete translation resume snapshots.

use nen_domain::subtitle::CueId;
use nen_ports::persistence::{CacheKey, ResumeBlock, ResumeCue, ResumeRecord, ResumeStoreError};
use serde::{Deserialize, Serialize};

const FORMAT_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct WireRecord {
    format_version: u32,
    cache_key: String,
    total_blocks: u32,
    blocks: Vec<WireBlock>,
}

#[derive(Serialize, Deserialize)]
struct WireBlock {
    block_index: u32,
    cues: Vec<WireCue>,
}

#[derive(Serialize, Deserialize)]
struct WireCue {
    cue_id: u32,
    text: String,
}

pub(crate) fn serialize(record: &ResumeRecord) -> Result<Vec<u8>, ResumeStoreError> {
    let wire = WireRecord {
        format_version: FORMAT_VERSION,
        cache_key: record.cache_key.to_hex(),
        total_blocks: record.total_blocks,
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
    Ok(ResumeRecord {
        cache_key,
        total_blocks: wire.total_blocks,
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
