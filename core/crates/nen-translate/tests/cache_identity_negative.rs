//! Negative coverage for `nen_translate::identity` (ADR-0018, `NEN-097`).
//!
//! One test per encoded component: change exactly that component and prove
//! two things at once — the identity itself differs, and a lookup keyed on
//! the old identity in a `HashMap` (standing in for the metadata index
//! `NEN-098` will build) misses. That second assertion is the DoD's "eski
//! artifact eşleşmiyor" line made concrete rather than left implicit in an
//! `assert_ne!`.
//!
//! Version-field components (pipeline/prompt/schema/block-layout/
//! translation-session) are covered separately, inside
//! `nen_translate::identity`'s own `#[cfg(test)]` module — they need the
//! crate-private `CacheIdentity::of_with_versions` to construct an
//! alternate [`PipelineVersions`], which an external test crate cannot see.
//! This file covers every component reachable through the public API:
//! source fingerprint, source language, target language, provider, model,
//! media hash, glossary, block size and overlap.

use std::collections::HashMap;

use nen_domain::source::LanguageTag;
use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_ports::identity::MediaHash;
use nen_ports::translation::TranslationProviderIdentity;
use nen_subtitle::fingerprint::SourceFingerprint;
use nen_translate::artifact::GlossaryIdentity;
use nen_translate::blocks::BlockLayoutConfig;
use nen_translate::identity::{CacheIdentity, CacheIdentityInput};

fn document(text: &str) -> SubtitleDocument {
    SubtitleDocument::new(vec![Cue::new(
        CueId::new(1),
        TimeSpan::new(0, 1_000).expect("valid span"),
        vec![text.to_owned()],
    )])
}

fn fingerprint(text: &str) -> SourceFingerprint {
    SourceFingerprint::of(&document(text))
}

fn language(tag: &str) -> LanguageTag {
    LanguageTag::parse(tag).expect("valid language")
}

fn provider(provider: &str, model: &str) -> TranslationProviderIdentity {
    TranslationProviderIdentity::new(provider, model).expect("valid identity")
}

fn layout(block_size: usize, overlap: usize) -> BlockLayoutConfig {
    BlockLayoutConfig::new(block_size, overlap).expect("valid config")
}

/// Every field at a fixed, arbitrary baseline value — each test below builds
/// its own copy and changes exactly one field before computing the identity.
struct Baseline {
    source_fingerprint: SourceFingerprint,
    source_language: LanguageTag,
    target_language: LanguageTag,
    provider: TranslationProviderIdentity,
    media_hash: Option<MediaHash>,
    glossary: GlossaryIdentity,
    block_layout: BlockLayoutConfig,
}

impl Baseline {
    fn new() -> Self {
        Self {
            source_fingerprint: fingerprint("Hello"),
            source_language: language("en"),
            target_language: language("tr"),
            provider: provider("openai", "gpt-baseline"),
            media_hash: Some(MediaHash::from_bytes([1, 2, 3, 4, 5, 6, 7, 8])),
            glossary: GlossaryIdentity::named("hunter-x-hunter"),
            block_layout: layout(40, 6),
        }
    }

    fn identity(&self) -> CacheIdentity {
        CacheIdentity::of(&CacheIdentityInput {
            source_fingerprint: self.source_fingerprint,
            source_language: &self.source_language,
            target_language: &self.target_language,
            provider: &self.provider,
            media_hash: self.media_hash,
            glossary: &self.glossary,
            block_layout: self.block_layout,
        })
    }
}

/// Asserts the DoD's two claims for one mutation: the identity changed, and
/// an artifact indexed under `old` is unreachable through `new`.
fn assert_invalidates(old: CacheIdentity, new: CacheIdentity) {
    assert_ne!(old, new, "mutated component did not change the identity");

    let mut index: HashMap<CacheIdentity, &'static str> = HashMap::new();
    index.insert(old, "stale-artifact");
    assert!(
        !index.contains_key(&new),
        "a lookup under the new identity found the old artifact"
    );
}

#[test]
fn source_fingerprint_change_invalidates() {
    let old = Baseline::new().identity();
    let mut mutated = Baseline::new();
    mutated.source_fingerprint = fingerprint("Goodbye");
    assert_invalidates(old, mutated.identity());
}

#[test]
fn source_language_change_invalidates() {
    let old = Baseline::new().identity();
    let mut mutated = Baseline::new();
    mutated.source_language = language("fr");
    assert_invalidates(old, mutated.identity());
}

#[test]
fn target_language_change_invalidates() {
    let old = Baseline::new().identity();
    let mut mutated = Baseline::new();
    mutated.target_language = language("de");
    assert_invalidates(old, mutated.identity());
}

#[test]
fn provider_name_change_invalidates() {
    let old = Baseline::new().identity();
    let mut mutated = Baseline::new();
    mutated.provider = provider("anthropic", "gpt-baseline");
    assert_invalidates(old, mutated.identity());
}

#[test]
fn provider_model_change_invalidates() {
    let old = Baseline::new().identity();
    let mut mutated = Baseline::new();
    mutated.provider = provider("openai", "gpt-newer");
    assert_invalidates(old, mutated.identity());
}

#[test]
fn media_hash_none_to_some_invalidates() {
    let mut old = Baseline::new();
    old.media_hash = None;
    let old = old.identity();
    let mutated = Baseline::new(); // media_hash: Some(...)
    assert_invalidates(old, mutated.identity());
}

#[test]
fn media_hash_different_value_invalidates() {
    let old = Baseline::new().identity();
    let mut mutated = Baseline::new();
    mutated.media_hash = Some(MediaHash::from_bytes([9, 9, 9, 9, 9, 9, 9, 9]));
    assert_invalidates(old, mutated.identity());
}

#[test]
fn glossary_none_to_named_invalidates() {
    let mut old = Baseline::new();
    old.glossary = GlossaryIdentity::none();
    let old = old.identity();
    let mutated = Baseline::new(); // glossary: Named("hunter-x-hunter")
    assert_invalidates(old, mutated.identity());
}

#[test]
fn glossary_different_name_invalidates() {
    let old = Baseline::new().identity();
    let mut mutated = Baseline::new();
    mutated.glossary = GlossaryIdentity::named("jujutsu-kaisen");
    assert_invalidates(old, mutated.identity());
}

#[test]
fn block_size_change_invalidates() {
    let old = Baseline::new().identity();
    let mut mutated = Baseline::new();
    mutated.block_layout = layout(50, 6);
    assert_invalidates(old, mutated.identity());
}

#[test]
fn overlap_change_invalidates() {
    let old = Baseline::new().identity();
    let mut mutated = Baseline::new();
    mutated.block_layout = layout(40, 8);
    assert_invalidates(old, mutated.identity());
}

/// Not a negative test on its own — a companion sanity check that the
/// baseline used by every test above is internally reproducible, so a
/// failure above is attributable to the one field each test changed and not
/// to `Baseline` itself being non-deterministic.
#[test]
fn two_unmutated_baselines_share_an_identity() {
    assert_eq!(Baseline::new().identity(), Baseline::new().identity());
}
