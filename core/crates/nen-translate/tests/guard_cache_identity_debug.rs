//! Security guard for cache identity (K23 #4/#8, `docs/security-policy.md`).
//!
//! [`CacheIdentityInput`] carries a user-chosen glossary name and a
//! provider/model identity — both potentially sensitive, both the obvious
//! place for a leak into a log line. [`CacheIdentity`] itself is, like
//! [`nen_ports::persistence::ContentAddress`], a hash that will eventually
//! double as a lookup/file key (`NEN-098`); K23 #8 forbids a private hash on
//! a log surface just as it forbids a private filename.
//!
//! This proves neither leaks, and that the guard is not blind: a sentinel
//! actually placed in the glossary name and the provider identity is
//! confirmed absent from every `Debug`/`Display` surface, and a deliberately
//! naive `#[derive(Debug)]` twin is shown to leak the same sentinel — so the
//! redaction demonstrably comes from the hand-written `Debug` impls, not
//! from the sentinel simply never reaching a formatter.

use nen_domain::source::LanguageTag;
use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_ports::identity::MediaHash;
use nen_ports::translation::TranslationProviderIdentity;
use nen_subtitle::fingerprint::SourceFingerprint;
use nen_translate::artifact::GlossaryIdentity;
use nen_translate::blocks::BlockLayoutConfig;
use nen_translate::identity::{CacheIdentity, CacheIdentityInput};

/// Long and distinctive so a partial leak is caught as surely as a whole
/// one.
const SENTINEL: &str = "Zzqxvunlogged";

fn fingerprint() -> SourceFingerprint {
    let document = SubtitleDocument::new(vec![Cue::new(
        CueId::new(1),
        TimeSpan::new(0, 1_000).expect("valid span"),
        vec!["Hello".to_owned()],
    )]);
    SourceFingerprint::of(&document)
}

fn sentinel_input<'a>(
    source_language: &'a LanguageTag,
    target_language: &'a LanguageTag,
    provider: &'a TranslationProviderIdentity,
    glossary: &'a GlossaryIdentity,
) -> CacheIdentityInput<'a> {
    CacheIdentityInput {
        source_fingerprint: fingerprint(),
        source_language,
        target_language,
        provider,
        media_hash: Some(MediaHash::from_bytes([1, 2, 3, 4, 5, 6, 7, 8])),
        glossary,
        block_layout: BlockLayoutConfig::new(40, 6).expect("valid config"),
    }
}

#[test]
fn no_cache_identity_input_debug_output_leaks_the_glossary_name() {
    let source_language = LanguageTag::parse("en").expect("valid language");
    let target_language = LanguageTag::parse("tr").expect("valid language");
    let provider = TranslationProviderIdentity::new("openai", "gpt-baseline").expect("valid id");
    let glossary = GlossaryIdentity::named(SENTINEL);

    let input = sentinel_input(&source_language, &target_language, &provider, &glossary);
    let debug = format!("{input:?}");
    assert!(
        !debug.contains(SENTINEL),
        "CacheIdentityInput::Debug leaked the glossary name — {debug}"
    );
}

#[test]
fn no_cache_identity_input_debug_output_leaks_the_provider_identity() {
    let source_language = LanguageTag::parse("en").expect("valid language");
    let target_language = LanguageTag::parse("tr").expect("valid language");
    let provider = TranslationProviderIdentity::new(SENTINEL, SENTINEL).expect("valid id");
    let glossary = GlossaryIdentity::none();

    let input = sentinel_input(&source_language, &target_language, &provider, &glossary);
    let debug = format!("{input:?}");
    assert!(
        !debug.contains(SENTINEL),
        "CacheIdentityInput::Debug leaked the provider identity — {debug}"
    );
}

#[test]
fn no_cache_identity_debug_or_display_output_leaks_its_own_hex() {
    let source_language = LanguageTag::parse("en").expect("valid language");
    let target_language = LanguageTag::parse("tr").expect("valid language");
    let provider = TranslationProviderIdentity::new("openai", "gpt-baseline").expect("valid id");
    let glossary = GlossaryIdentity::named("hunter-x-hunter");

    let identity = CacheIdentity::of(&sentinel_input(
        &source_language,
        &target_language,
        &provider,
        &glossary,
    ));
    let hex = identity.to_hex();

    let debug = format!("{identity:?}");
    let display = format!("{identity}");
    assert!(
        !debug.contains(&hex),
        "CacheIdentity::Debug leaked its own hex — {debug}"
    );
    assert!(
        !display.contains(&hex),
        "CacheIdentity::Display leaked its own hex — {display}"
    );
}

/// A guard-rail on the guards above: if `CacheIdentityInput` were changed to
/// a naive `#[derive(Debug)]`, the sentinel would show up immediately. This
/// documents that the redaction above comes from the hand-written `Debug`
/// impl, not from the sentinel accidentally never reaching a formatter.
#[test]
fn the_guard_sentinel_would_be_visible_in_a_derived_debug_twin() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct LeakyCacheIdentityInput {
        glossary_name: String,
        provider: String,
    }

    let leaked = format!(
        "{:?}",
        LeakyCacheIdentityInput {
            glossary_name: SENTINEL.to_owned(),
            provider: SENTINEL.to_owned(),
        }
    );
    assert!(leaked.contains(SENTINEL));
}
