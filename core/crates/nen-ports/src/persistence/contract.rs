//! Shared contract kit for every artifact store adapter (`docs/architecture.md`
//! → "`HttpClient` ve `Persistence` de contract test kiti alır").
//!
//! The kit exercises what ADR-0017 makes true of *any* store, filesystem or
//! not: content round-trips, the same content always lands at the same
//! address, different content does not, and an address nothing was written to
//! reads as missing rather than as something else. Whether a second write
//! leaves a second copy on disk is a filesystem-level question the adapter's
//! own tests answer; it is not observable through this port.

use super::{ArtifactRecord, ArtifactStore, ArtifactStoreError, ContentAddress};
use crate::translation::TranslationProviderIdentity;
use nen_domain::source::LanguageTag;
use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractViolation {
    /// A stored artifact did not read back exactly as it was written.
    RoundTrip,
    /// Writing the same content twice produced two different addresses.
    AddressStability,
    /// Two different records landed at the same address.
    ContentSensitivity,
    /// An address nothing was written to did not read as missing.
    MissingNotFound,
    /// The store refused an operation the contract requires it to accept.
    Unavailable,
}

/// Builds the kit's fixture record. `marker` varies the translated text, so
/// two calls with different markers are two genuinely different artifacts.
///
/// `expect` here follows `playback::fake`'s precedent: the inputs are
/// literals this function owns, so a failure is a defect in the kit itself
/// and a panic is the right report.
fn record(marker: &str) -> ArtifactRecord {
    let cues = (0..3u32)
        .map(|index| {
            let start = index * 1_000;
            Cue::new(
                CueId::new(index + 1),
                TimeSpan::new(start, start + 900).expect("a valid span"),
                vec![format!("line {index} {marker}")],
            )
        })
        .collect();
    let document = SubtitleDocument::new(cues);
    let webvtt = format!("WEBVTT\n\n1\n00:00:00.000 --> 00:00:00.900\n{marker}\n");

    ArtifactRecord {
        source_fingerprint: [1u8; 32],
        timeline_fingerprint: [2u8; 32],
        source_language: LanguageTag::parse("en").expect("a valid tag"),
        target_language: LanguageTag::parse("tr").expect("a valid tag"),
        provider: TranslationProviderIdentity::new("contract", "kit").expect("a valid identity"),
        pipeline_version: 1,
        block_layout_version: 1,
        glossary: None,
        media_hash: None,
        created_at_unix_ms: 1_700_000_000_000,
        document,
        webvtt,
    }
}

pub fn check(store: &dyn ArtifactStore) -> Result<(), Vec<ContractViolation>> {
    let mut violations = Vec::new();

    // An address nothing was written to reads as missing — checked first, so
    // a store that answers every address the same way is caught before its
    // own writes can mask it.
    let absent = ContentAddress::from_bytes([0u8; 32]);
    match store.get(absent) {
        Err(ArtifactStoreError::NotFound) => {}
        _ => violations.push(ContractViolation::MissingNotFound),
    }
    if store.contains(absent) != Ok(false) {
        violations.push(ContractViolation::MissingNotFound);
    }

    let first = record("alpha");
    let Ok(address) = store.put(&first) else {
        violations.push(ContractViolation::Unavailable);
        return Err(violations);
    };

    match store.get(address) {
        Ok(read_back) if read_back == first => {}
        Ok(_) => violations.push(ContractViolation::RoundTrip),
        Err(_) => violations.push(ContractViolation::Unavailable),
    }
    if store.contains(address) != Ok(true) {
        violations.push(ContractViolation::RoundTrip);
    }

    match store.put(&first) {
        Ok(again) if again == address => {}
        Ok(_) => violations.push(ContractViolation::AddressStability),
        Err(_) => violations.push(ContractViolation::Unavailable),
    }

    let second = record("beta");
    match store.put(&second) {
        Ok(other) if other == address => violations.push(ContractViolation::ContentSensitivity),
        Ok(other) => match store.get(other) {
            Ok(read_back) if read_back == second => {}
            Ok(_) => violations.push(ContractViolation::RoundTrip),
            Err(_) => violations.push(ContractViolation::Unavailable),
        },
        Err(_) => violations.push(ContractViolation::Unavailable),
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}
