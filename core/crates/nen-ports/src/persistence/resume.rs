//! Persistence port for resumable translation checkpoints (NEN-105).

use nen_domain::subtitle::CueId;
use std::fmt;

use super::CacheKey;

/// A defensive upper bound for one resume snapshot read from untrusted disk.
pub const MAX_RESUME_BYTES: usize = 16 * 1024 * 1024;

/// One persisted translated cue. Text is private subtitle dialogue (K23 #4).
#[derive(Clone, PartialEq, Eq)]
pub struct ResumeCue {
    pub cue_id: CueId,
    pub text: String,
}

impl fmt::Debug for ResumeCue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResumeCue")
            .field("cue_id", &self.cue_id)
            .field("text_len", &self.text.chars().count())
            .finish()
    }
}

/// One fully validated block as stored between process runs.
#[derive(Clone, PartialEq, Eq)]
pub struct ResumeBlock {
    pub block_index: u32,
    pub cues: Vec<ResumeCue>,
}

impl fmt::Debug for ResumeBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResumeBlock")
            .field("block_index", &self.block_index)
            .field("cue_count", &self.cues.len())
            .finish()
    }
}

/// A complete snapshot of the validated prefix of one translation run.
#[derive(Clone, PartialEq, Eq)]
pub struct ResumeRecord {
    pub cache_key: CacheKey,
    pub total_blocks: u32,
    pub blocks: Vec<ResumeBlock>,
}

impl fmt::Debug for ResumeRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResumeRecord")
            .field("cache_key", &self.cache_key)
            .field("total_blocks", &self.total_blocks)
            .field("checkpointed_blocks", &self.blocks.len())
            .finish()
    }
}

/// Why the separate resume area refused an operation. No variant carries a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeStoreError {
    Io,
    Corrupt,
    OutsideRoot,
    TooLarge,
}

impl fmt::Display for ResumeStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Io => "the resume store could not complete a filesystem operation",
            Self::Corrupt => "the stored resume record is not well-formed",
            Self::OutsideRoot => "the resume address resolves outside the store root",
            Self::TooLarge => "the stored resume record is larger than the read bound",
        })
    }
}

impl std::error::Error for ResumeStoreError {}

/// Separate persistence for incomplete, never-publishable translation state.
pub trait ResumeStore: Send + Sync {
    fn load(&self, key: CacheKey) -> Result<Option<ResumeRecord>, ResumeStoreError>;
    fn save(&self, record: &ResumeRecord) -> Result<(), ResumeStoreError>;
    fn delete(&self, key: CacheKey) -> Result<(), ResumeStoreError>;
}
