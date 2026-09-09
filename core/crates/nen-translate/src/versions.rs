//! Every hand-bumped pipeline version constant, collected in one place
//! (ADR-0018 Karar 4).
//!
//! Each constant is bumped by hand exactly when the thing it names changes
//! meaning, never derived: `PIPELINE_VERSION` when [`crate::artifact::assemble`]'s
//! own step order or meaning changes, `PROMPT_VERSION` when the prompt text
//! sent to a provider changes (M6 — no prompt exists yet, so this cannot bump
//! today), `SCHEMA_VERSION` when the shape of data sent to/from a provider
//! changes, `BLOCK_LAYOUT_VERSION` when ADR-0015's block-boundary/overlap
//! rule changes, and `TRANSLATION_SESSION_VERSION` when the shape of a
//! source/target language-pair representation changes.
//!
//! [`crate::identity::CacheIdentity::of`] reads [`PipelineVersions::CURRENT`]
//! as five of its components — a bump to any constant here makes every cache
//! entry keyed on the old value unreachable, which is the entire point.

/// Bumped by hand whenever the artifact-assembly pipeline's own step order or
/// meaning changes (not the block layout, provider schema or prompt — those
/// get their own versions here). Re-exported at [`crate::artifact::PIPELINE_VERSION`]
/// for the call sites that existed before this module did (NEN-094).
pub const PIPELINE_VERSION: u32 = 1;

/// Bumped by hand whenever the prompt text sent to a translation provider
/// changes. M5 has no real provider (mock only), so no prompt exists yet and
/// this cannot bump today — it exists so `NEN-097`'s cache identity has
/// somewhere to read a real value from once M6 adds one.
pub const PROMPT_VERSION: u32 = 1;

/// Bumped by hand whenever the shape of data sent to or received from a
/// translation provider changes (`nen_ports::translation`'s request/response
/// DTOs).
pub const SCHEMA_VERSION: u32 = 1;

/// Bumped by hand whenever ADR-0015 Karar 1/2's block-boundary or
/// overlap-split rule changes. Re-exported at
/// [`crate::blocks::BLOCK_LAYOUT_VERSION`] for the call sites that existed
/// before this module did (NEN-089).
pub const BLOCK_LAYOUT_VERSION: u32 = 1;

/// Bumped by hand whenever the shape of a source/target language-pair
/// representation changes (`crate::checkpoint::TranslationPlan`).
pub const TRANSLATION_SESSION_VERSION: u32 = 1;

/// Every version constant above, bundled so [`crate::identity::CacheIdentity`]
/// takes one parameter instead of five and a test can construct an
/// alternate set without touching the real constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipelineVersions {
    pub pipeline: u32,
    pub prompt: u32,
    pub schema: u32,
    pub block_layout: u32,
    pub translation_session: u32,
}

impl PipelineVersions {
    /// The version set every real call site uses. Tests that need to prove a
    /// version bump changes the cache identity build their own
    /// [`PipelineVersions`] instead of mutating these constants.
    pub const CURRENT: Self = Self {
        pipeline: PIPELINE_VERSION,
        prompt: PROMPT_VERSION,
        schema: SCHEMA_VERSION,
        block_layout: BLOCK_LAYOUT_VERSION,
        translation_session: TRANSLATION_SESSION_VERSION,
    };
}
