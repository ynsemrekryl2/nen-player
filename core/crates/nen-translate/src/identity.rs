//! Cache identity: one deterministic key over every component
//! `docs/product-spec.md` §11 names, so that "prompt/schema/pipeline
//! semantiği değişince uyumsuz cache kullanılmamalıdır" holds mechanically
//! rather than by convention (ADR-0018).
//!
//! [`CacheIdentity::of`] does not decide where the identity is stored or
//! compared — that is `NEN-098`'s (an artifact metadata index) and, for
//! `S9`'s "menüde yalnız en yeni" projection, its consumer's job. This module
//! only computes the value: same [`CacheIdentityInput`] in, same
//! [`CacheIdentity`] out, always, on every machine — and a different value on
//! any single component changing.
//!
//! The encoding mirrors [`nen_subtitle::fingerprint`] (ADR-0007's pattern,
//! reused per ADR-0018 Karar 1): an explicit, fixed-order byte encoding
//! hashed with BLAKE3, not `serde_json` or any other general-purpose
//! serialization whose field order is a derive detail rather than a
//! contract.

use std::fmt;

use nen_domain::source::LanguageTag;
use nen_ports::identity::MediaHash;
use nen_ports::translation::TranslationProviderIdentity;
use nen_subtitle::fingerprint::SourceFingerprint;

use crate::artifact::GlossaryIdentity;
use crate::blocks::BlockLayoutConfig;
use crate::versions::PipelineVersions;

/// Every component `docs/product-spec.md` §11 lists, borrowed rather than
/// owned — a caller already holds each of these (a
/// [`crate::checkpoint::TranslationPlan`], a [`crate::artifact::ArtifactMetadata`],
/// a [`crate::blocks::BlockLayout`]) and this type does not ask for a second
/// copy.
///
/// `media_context` (ADR-0018 Karar 5) is [`MediaHash`] alone, not the full
/// evidence layer — title, year and file path do not enter the identity.
pub struct CacheIdentityInput<'a> {
    pub source_fingerprint: SourceFingerprint,
    pub source_language: &'a LanguageTag,
    pub target_language: &'a LanguageTag,
    pub provider: &'a TranslationProviderIdentity,
    pub media_hash: Option<MediaHash>,
    pub glossary: &'a GlossaryIdentity,
    pub block_layout: BlockLayoutConfig,
}

/// Hand-written per `docs/security-policy.md` §1 #8: `provider` carries a
/// provider/model identity and `glossary` may carry a user-chosen name,
/// neither of which is loggable. Only shape survives.
impl fmt::Debug for CacheIdentityInput<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CacheIdentityInput")
            .field("source_language", &self.source_language)
            .field("target_language", &self.target_language)
            .field("has_media_hash", &self.media_hash.is_some())
            .field(
                "has_glossary",
                &!matches!(self.glossary, GlossaryIdentity::None),
            )
            .field("block_size", &self.block_layout.block_size())
            .field("overlap", &self.block_layout.overlap())
            .finish()
    }
}

/// BLAKE3 digest of a [`CacheIdentityInput`] encoded against
/// [`PipelineVersions::CURRENT`] (ADR-0018 Karar 1/2).
///
/// Two artifacts with the same identity were built from the same source
/// text+timing, the same language pair, the same provider/model, the same
/// media, the same glossary, the same block layout, and the same pipeline/
/// prompt/schema/block-layout/session semantics — so one is always a valid
/// substitute for the other. Any one of those differing yields a different
/// identity, so an old artifact is simply never looked up again once the
/// component it depended on changes.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheIdentity([u8; 32]);

impl CacheIdentity {
    /// Computes the identity for `input` under today's pipeline semantics
    /// ([`PipelineVersions::CURRENT`]). There is deliberately no variant that
    /// takes an explicit [`PipelineVersions`] outside this crate's own
    /// tests — a caller cannot ask for yesterday's identity, only today's.
    pub fn of(input: &CacheIdentityInput<'_>) -> Self {
        Self::encode(input, &PipelineVersions::CURRENT)
    }

    #[cfg(test)]
    pub(crate) fn of_with_versions(
        input: &CacheIdentityInput<'_>,
        versions: &PipelineVersions,
    ) -> Self {
        Self::encode(input, versions)
    }

    fn encode(input: &CacheIdentityInput<'_>, versions: &PipelineVersions) -> Self {
        let mut bytes = Vec::new();

        // 1. Source fingerprint — already a fixed-width 32-byte digest.
        bytes.extend_from_slice(input.source_fingerprint.as_bytes());
        // 2-3. Source / target language.
        push_str(&mut bytes, input.source_language.as_str());
        push_str(&mut bytes, input.target_language.as_str());
        // 4. Provider/model — two separate length-prefixed fields, not one
        // concatenated string, so `("ab", "c")` and `("a", "bc")` cannot
        // collide (same reasoning as `SourceFingerprint`'s per-line prefix).
        push_str(&mut bytes, input.provider.provider());
        push_str(&mut bytes, input.provider.model());
        // 5. Media context (ADR-0018 Karar 5): MediaHash only.
        push_option(&mut bytes, input.media_hash, |bytes, hash| {
            bytes.extend_from_slice(hash.as_bytes());
        });
        // 6. Glossary (ADR-0018 Karar 3): coded as a 0- or 1-element
        // collection. "No glossary" and "empty glossary" are the same
        // representation on purpose — there is no third state to encode.
        match input.glossary {
            GlossaryIdentity::None => push_u32(&mut bytes, 0),
            GlossaryIdentity::Named(name) => {
                push_u32(&mut bytes, 1);
                push_str(&mut bytes, name);
            }
        }
        // 7-8. Block size / overlap.
        push_u32(&mut bytes, input.block_layout.block_size());
        push_u32(&mut bytes, input.block_layout.overlap());
        // 9-13. Pipeline / prompt / schema / block-layout / session versions.
        push_u32(&mut bytes, versions.pipeline as usize);
        push_u32(&mut bytes, versions.prompt as usize);
        push_u32(&mut bytes, versions.schema as usize);
        push_u32(&mut bytes, versions.block_layout as usize);
        push_u32(&mut bytes, versions.translation_session as usize);

        Self(*blake3::hash(&bytes).as_bytes())
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Lowercase hex, 64 characters. Named rather than `Display` on purpose —
    /// see this type's `Debug`/`Display` docs.
    pub fn to_hex(&self) -> String {
        let mut out = String::with_capacity(64);
        for byte in self.0 {
            out.push(hex_digit(byte >> 4));
            out.push(hex_digit(byte & 0x0f));
        }
        out
    }
}

/// Matches `nen_ports::persistence::ContentAddress::to_hex`'s convention —
/// this module cannot see that private helper, so the four-line encoder is
/// duplicated rather than the two crates growing a shared dependency for it.
const fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'a' + nibble - 10) as char,
    }
}

/// K23 #8: a cache identity is derived from a media hash and (in the general
/// case) other sensitive components, and it will eventually double as a
/// lookup/file key (`NEN-098`) — the same rationale
/// [`nen_ports::persistence::ContentAddress`] already documents for a content
/// address. `Debug`/`Display` print `<redacted>`; the real hex is reachable
/// only through the explicitly named [`CacheIdentity::to_hex`]/
/// [`CacheIdentity::as_bytes`].
impl fmt::Debug for CacheIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CacheIdentity(<redacted>)")
    }
}

impl fmt::Display for CacheIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

/// Encodes `value` as a `u32` LE length prefix followed by its UTF-8 bytes.
fn push_str(bytes: &mut Vec<u8>, value: &str) {
    push_u32(bytes, value.len());
    bytes.extend_from_slice(value.as_bytes());
}

/// Encodes `value` as `u32` LE, saturating rather than panicking — this crate
/// denies `unwrap`/`expect` outside tests and nothing realistic here
/// approaches `u32::MAX` (`nen_subtitle::fingerprint`'s same convention).
fn push_u32(bytes: &mut Vec<u8>, value: usize) {
    let value = u32::try_from(value).unwrap_or(u32::MAX);
    bytes.extend_from_slice(&value.to_le_bytes());
}

/// Encodes an optional value as one presence byte, followed by `write` on
/// `bytes` only when `value` is `Some`.
fn push_option<T>(bytes: &mut Vec<u8>, value: Option<T>, write: impl FnOnce(&mut Vec<u8>, T)) {
    match value {
        Some(value) => {
            bytes.push(1);
            write(bytes, value);
        }
        None => bytes.push(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};

    fn source_fingerprint() -> SourceFingerprint {
        let document = SubtitleDocument::new(vec![Cue::new(
            CueId::new(1),
            TimeSpan::new(0, 1_000).expect("valid span"),
            vec!["Hello".to_owned()],
        )]);
        SourceFingerprint::of(&document)
    }

    fn language(tag: &str) -> LanguageTag {
        LanguageTag::parse(tag).expect("valid language")
    }

    fn provider(provider: &str, model: &str) -> TranslationProviderIdentity {
        TranslationProviderIdentity::new(provider, model).expect("valid identity")
    }

    fn block_layout(block_size: usize, overlap: usize) -> BlockLayoutConfig {
        BlockLayoutConfig::new(block_size, overlap).expect("valid config")
    }

    /// Builds a fresh, independently-owned baseline input every time it is
    /// called — used both to prove determinism (two independently built
    /// inputs with the same values yield the same identity) and as the
    /// starting point every negative test mutates exactly one field of.
    fn baseline() -> (
        LanguageTag,
        LanguageTag,
        TranslationProviderIdentity,
        GlossaryIdentity,
    ) {
        (
            language("en"),
            language("tr"),
            provider("test", "echo"),
            GlossaryIdentity::none(),
        )
    }

    fn input<'a>(
        source_language: &'a LanguageTag,
        target_language: &'a LanguageTag,
        provider: &'a TranslationProviderIdentity,
        glossary: &'a GlossaryIdentity,
    ) -> CacheIdentityInput<'a> {
        CacheIdentityInput {
            source_fingerprint: source_fingerprint(),
            source_language,
            target_language,
            provider,
            media_hash: None,
            glossary,
            block_layout: block_layout(40, 6),
        }
    }

    #[test]
    fn same_input_produces_the_same_identity_twice() {
        let (source_language, target_language, provider, glossary) = baseline();
        let a = CacheIdentity::of(&input(
            &source_language,
            &target_language,
            &provider,
            &glossary,
        ));
        let b = CacheIdentity::of(&input(
            &source_language,
            &target_language,
            &provider,
            &glossary,
        ));
        assert_eq!(a, b);
    }

    #[test]
    fn independently_built_equal_inputs_produce_the_same_identity() {
        // Same values, but every component built from its own fresh call —
        // proves the identity is a function of the *values*, not of sharing
        // one in-memory instance (no accidental reliance on pointer/instance
        // identity anywhere in the encoding).
        let a = CacheIdentity::of(&input(
            &language("en"),
            &language("tr"),
            &provider("test", "echo"),
            &GlossaryIdentity::none(),
        ));
        let b = CacheIdentity::of(&input(
            &language("en"),
            &language("tr"),
            &provider("test", "echo"),
            &GlossaryIdentity::none(),
        ));
        assert_eq!(a, b);
    }

    #[test]
    fn same_media_hash_value_from_two_builds_produces_the_same_identity() {
        // Exercises the `Some(MediaHash)` path (the two tests above only
        // cover `input()`'s `None` default) — the "no unnecessary
        // invalidation" DoD line held against the one component the other
        // tests do not touch.
        let (source_language, target_language, provider, glossary) = baseline();
        let mut a = input(&source_language, &target_language, &provider, &glossary);
        a.media_hash = Some(MediaHash::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]));
        let mut b = input(&source_language, &target_language, &provider, &glossary);
        b.media_hash = Some(MediaHash::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]));
        assert_eq!(CacheIdentity::of(&a), CacheIdentity::of(&b));
    }

    #[test]
    fn to_hex_is_lowercase_and_64_characters() {
        let (source_language, target_language, provider, glossary) = baseline();
        let hex = CacheIdentity::of(&input(
            &source_language,
            &target_language,
            &provider,
            &glossary,
        ))
        .to_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    // --- Version-field negative tests (ADR-0018 Karar 4) ---
    //
    // These five live here rather than in `tests/cache_identity_negative.rs`
    // because they need `of_with_versions`, which is `pub(crate)` on
    // purpose (module docs on `CacheIdentity::of`): a real caller only ever
    // gets today's identity, never an arbitrary one. Each test simulates a
    // version bump — the same input encoded once under a "before" set and
    // once under an "after" set that differs in exactly one field — and
    // proves a `HashMap` lookup keyed on the "before" identity misses under
    // "after", the same way `assert_invalidates` does in the external file.

    fn assert_bump_invalidates(before: PipelineVersions, after: PipelineVersions) {
        let (source_language, target_language, provider, glossary) = baseline();
        let fixed_input = input(&source_language, &target_language, &provider, &glossary);

        let old = CacheIdentity::of_with_versions(&fixed_input, &before);
        let new = CacheIdentity::of_with_versions(&fixed_input, &after);
        assert_ne!(old, new, "version bump did not change the identity");

        let mut index: HashMap<CacheIdentity, &'static str> = HashMap::new();
        index.insert(old, "stale-artifact");
        assert!(
            !index.contains_key(&new),
            "a lookup under the bumped identity found the pre-bump artifact"
        );
    }

    #[test]
    fn pipeline_version_bump_invalidates() {
        let before = PipelineVersions::CURRENT;
        let after = PipelineVersions {
            pipeline: before.pipeline + 1,
            ..before
        };
        assert_bump_invalidates(before, after);
    }

    #[test]
    fn prompt_version_bump_invalidates() {
        let before = PipelineVersions::CURRENT;
        let after = PipelineVersions {
            prompt: before.prompt + 1,
            ..before
        };
        assert_bump_invalidates(before, after);
    }

    #[test]
    fn schema_version_bump_invalidates() {
        let before = PipelineVersions::CURRENT;
        let after = PipelineVersions {
            schema: before.schema + 1,
            ..before
        };
        assert_bump_invalidates(before, after);
    }

    #[test]
    fn block_layout_version_bump_invalidates() {
        let before = PipelineVersions::CURRENT;
        let after = PipelineVersions {
            block_layout: before.block_layout + 1,
            ..before
        };
        assert_bump_invalidates(before, after);
    }

    #[test]
    fn translation_session_version_bump_invalidates() {
        let before = PipelineVersions::CURRENT;
        let after = PipelineVersions {
            translation_session: before.translation_session + 1,
            ..before
        };
        assert_bump_invalidates(before, after);
    }
}
