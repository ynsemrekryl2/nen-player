//! K23 guard for `NEN-098`'s index types (`docs/security-policy.md` §1).
//!
//! [`CacheKey`] doubles as a cache lookup key over sensitive components
//! (ADR-0018 — source fingerprint, media hash, provider/model identity) and
//! [`ArtifactIndexEntry`] carries one of those keys plus a raw source
//! fingerprint (K23 #8) — neither may reach a `Debug`/`Display` surface.
//!
//! Same technique as `nen-persist`'s `guard_persist_debug.rs` (which already
//! covers [`ContentAddress`]): a distinctive byte pattern proves the guard is
//! not vacuous, and a `#[derive(Debug)]` twin of the same payload is shown to
//! leak it.

use nen_domain::source::LanguageTag;
use nen_ports::persistence::{ArtifactIndexEntry, CacheKey, ContentAddress};

/// A byte pattern distinctive enough that its hex encoding cannot appear by
/// accident.
const CACHE_KEY_BYTES: [u8; 32] = [0x5a; 32];
const SOURCE_FINGERPRINT_BYTES: [u8; 32] = [0x6b; 32];
const ADDRESS_BYTES: [u8; 32] = [0x7c; 32];

#[test]
fn no_cache_key_debug_or_display_output_leaks_its_own_hex() {
    let key = CacheKey::from_bytes(CACHE_KEY_BYTES);
    let hex = key.to_hex();

    // Not vacuous: the hex really is derivable from this key.
    assert_eq!(hex.len(), 64);
    assert!(hex.starts_with("5a5a"));

    let debug = format!("{key:?}");
    let display = format!("{key}");
    assert!(
        !debug.contains(&hex) && !debug.contains("5a5a"),
        "CacheKey::Debug leaked its own hex — {debug}"
    );
    assert!(
        !display.contains(&hex) && !display.contains("5a5a"),
        "CacheKey::Display leaked its own hex — {display}"
    );
}

fn entry_with_sentinel() -> ArtifactIndexEntry {
    ArtifactIndexEntry {
        address: ContentAddress::from_bytes(ADDRESS_BYTES),
        cache_identity: CacheKey::from_bytes(CACHE_KEY_BYTES),
        source_fingerprint: SOURCE_FINGERPRINT_BYTES,
        target_language: LanguageTag::parse("tr").expect("a valid tag"),
        created_at_unix_ms: 1_700_000_000_000,
    }
}

#[test]
fn no_index_entry_debug_output_leaks_a_hash_or_fingerprint() {
    let entry = entry_with_sentinel();
    let debug = format!("{entry:?}");

    assert!(
        !debug.contains("5a5a") && !debug.contains("6b6b") && !debug.contains("7c7c"),
        "ArtifactIndexEntry::Debug leaked a hash or fingerprint — {debug}"
    );
    // Ordinary, non-sensitive fields still print — the guard is not blind to
    // its own scope.
    assert!(
        debug.contains("tr"),
        "unrelated fields must still print — {debug}"
    );
}

#[test]
fn a_derived_debug_really_would_leak_the_index_entrys_hashes() {
    // The hand-written impl is what does the redaction above; a derive over
    // the same payload does not.
    #[derive(Debug)]
    #[allow(dead_code)]
    struct DerivedTwin {
        address: [u8; 32],
        cache_identity: [u8; 32],
        source_fingerprint: [u8; 32],
    }

    let entry = entry_with_sentinel();
    let twin = DerivedTwin {
        address: *entry.address.as_bytes(),
        cache_identity: *entry.cache_identity.as_bytes(),
        source_fingerprint: entry.source_fingerprint,
    };

    let debug = format!("{twin:?}");
    assert!(
        debug.contains("124, 124"), // 0x7c
        "the derived twin hid the address digest — this control proves nothing: {debug}"
    );
    assert!(
        debug.contains("90, 90"), // 0x5a
        "the derived twin hid the cache identity digest — this control proves nothing: {debug}"
    );
    assert!(
        debug.contains("107, 107"), // 0x6b
        "the derived twin hid the fingerprint digest — this control proves nothing: {debug}"
    );
}
