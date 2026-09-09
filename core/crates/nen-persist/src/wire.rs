//! The on-disk shape of one artifact file and its conversion to and from
//! [`ArtifactRecord`] (ADR-0017 Karar 2: metadata, normalized cues and
//! WebVTT together in a single canonical file).
//!
//! `serde` lives only in this module. `nen-domain` and `nen-ports` types stay
//! unaware of serialization: the wire structs below are plain data, and every
//! conversion back into a domain type goes through that type's own public
//! constructor, so a hand-edited or corrupted file cannot produce a domain
//! value the domain itself would refuse to build.
//!
//! Field order matters. `serde_json` serializes a struct's fields in
//! declaration order, which is what makes the bytes — and therefore the
//! content address — deterministic for a given record. Nothing here may
//! become a `HashMap`.

use nen_domain::source::LanguageTag;
use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_ports::identity::MediaHash;
use nen_ports::persistence::{ArtifactRecord, ArtifactStoreError, CacheKey};
use nen_ports::translation::TranslationProviderIdentity;
use serde::{Deserialize, Serialize};

/// Version of the *file format* below — not ADR-0018's `schema version`,
/// which is about a provider's structured-output contract and is
/// `NEN-097`'s subject. This exists so a future format change is refused
/// (`ArtifactStoreError::Corrupt`) rather than misread as the current one.
///
/// `2` (`NEN-098`) added `cache_identity`; a `1`-tagged file predates cache
/// identity and is refused rather than read back with a fabricated key.
const FORMAT_VERSION: u32 = 2;

#[derive(Serialize, Deserialize)]
struct WireRecord {
    format: u32,
    source_fingerprint: String,
    timeline_fingerprint: String,
    source_language: String,
    target_language: String,
    provider: String,
    model: String,
    pipeline_version: u32,
    block_layout_version: u32,
    glossary: Option<String>,
    media_hash: Option<String>,
    created_at_unix_ms: u64,
    cues: Vec<WireCue>,
    webvtt: String,
    cache_identity: String,
}

#[derive(Serialize, Deserialize)]
struct WireCue {
    id: u32,
    start_ms: u32,
    end_ms: u32,
    lines: Vec<String>,
}

/// Serializes a record into the exact bytes that get hashed and stored.
pub(crate) fn serialize(record: &ArtifactRecord) -> Result<Vec<u8>, ArtifactStoreError> {
    let wire = WireRecord {
        format: FORMAT_VERSION,
        source_fingerprint: to_hex(&record.source_fingerprint),
        timeline_fingerprint: to_hex(&record.timeline_fingerprint),
        source_language: record.source_language.as_str().to_owned(),
        target_language: record.target_language.as_str().to_owned(),
        provider: record.provider.provider().to_owned(),
        model: record.provider.model().to_owned(),
        pipeline_version: record.pipeline_version,
        block_layout_version: record.block_layout_version,
        glossary: record.glossary.clone(),
        media_hash: record.media_hash.map(|hash| to_hex(hash.as_bytes())),
        created_at_unix_ms: record.created_at_unix_ms,
        cues: record
            .document
            .cues()
            .iter()
            .map(|cue| WireCue {
                id: cue.id().get(),
                start_ms: cue.span().start_ms(),
                end_ms: cue.span().end_ms(),
                lines: cue.lines().to_vec(),
            })
            .collect(),
        webvtt: record.webvtt.clone(),
        cache_identity: to_hex(record.cache_identity.as_bytes()),
    };

    serde_json::to_vec(&wire).map_err(|_| ArtifactStoreError::Corrupt)
}

/// Parses stored bytes back into a record. Every failure — malformed JSON, an
/// unknown format, a language tag or time span the domain refuses — is the
/// same typed, payload-free [`ArtifactStoreError::Corrupt`].
pub(crate) fn deserialize(bytes: &[u8]) -> Result<ArtifactRecord, ArtifactStoreError> {
    let wire: WireRecord =
        serde_json::from_slice(bytes).map_err(|_| ArtifactStoreError::Corrupt)?;
    if wire.format != FORMAT_VERSION {
        return Err(ArtifactStoreError::Corrupt);
    }

    let mut cues = Vec::with_capacity(wire.cues.len());
    for cue in wire.cues {
        let span =
            TimeSpan::new(cue.start_ms, cue.end_ms).map_err(|_| ArtifactStoreError::Corrupt)?;
        cues.push(Cue::new(CueId::new(cue.id), span, cue.lines));
    }

    Ok(ArtifactRecord {
        source_fingerprint: from_hex_32(&wire.source_fingerprint)?,
        timeline_fingerprint: from_hex_32(&wire.timeline_fingerprint)?,
        source_language: LanguageTag::parse(&wire.source_language)
            .map_err(|_| ArtifactStoreError::Corrupt)?,
        target_language: LanguageTag::parse(&wire.target_language)
            .map_err(|_| ArtifactStoreError::Corrupt)?,
        provider: TranslationProviderIdentity::new(&wire.provider, &wire.model)
            .map_err(|_| ArtifactStoreError::Corrupt)?,
        pipeline_version: wire.pipeline_version,
        block_layout_version: wire.block_layout_version,
        glossary: wire.glossary,
        media_hash: wire
            .media_hash
            .as_deref()
            .map(from_hex_8)
            .transpose()?
            .map(MediaHash::from_bytes),
        created_at_unix_ms: wire.created_at_unix_ms,
        document: SubtitleDocument::new(cues),
        webvtt: wire.webvtt,
        cache_identity: CacheKey::from_bytes(from_hex_32(&wire.cache_identity)?),
    })
}

fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(hex_digit(byte >> 4));
        out.push(hex_digit(byte & 0x0f));
    }
    out
}

const fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'a' + nibble - 10) as char,
    }
}

fn from_hex_32(value: &str) -> Result<[u8; 32], ArtifactStoreError> {
    let mut bytes = [0u8; 32];
    decode_hex(value, &mut bytes)?;
    Ok(bytes)
}

fn from_hex_8(value: &str) -> Result<[u8; 8], ArtifactStoreError> {
    let mut bytes = [0u8; 8];
    decode_hex(value, &mut bytes)?;
    Ok(bytes)
}

fn decode_hex(value: &str, out: &mut [u8]) -> Result<(), ArtifactStoreError> {
    if value.len() != out.len() * 2 {
        return Err(ArtifactStoreError::Corrupt);
    }
    for (index, byte) in out.iter_mut().enumerate() {
        let pair = value
            .get(index * 2..index * 2 + 2)
            .ok_or(ArtifactStoreError::Corrupt)?;
        let mut decoded = 0u8;
        for digit in pair.bytes() {
            let nibble = match digit {
                b'0'..=b'9' => digit - b'0',
                b'a'..=b'f' => digit - b'a' + 10,
                _ => return Err(ArtifactStoreError::Corrupt),
            };
            decoded = (decoded << 4) | nibble;
        }
        *byte = decoded;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nen_domain::subtitle::{Cue, CueId, TimeSpan};

    fn record() -> ArtifactRecord {
        ArtifactRecord {
            source_fingerprint: [7u8; 32],
            timeline_fingerprint: [9u8; 32],
            source_language: LanguageTag::parse("en").expect("a valid tag"),
            target_language: LanguageTag::parse("tr").expect("a valid tag"),
            provider: TranslationProviderIdentity::new("mock", "echo-1").expect("an identity"),
            pipeline_version: 1,
            block_layout_version: 1,
            glossary: None,
            media_hash: None,
            created_at_unix_ms: 1_700_000_000_000,
            document: SubtitleDocument::new(vec![Cue::new(
                CueId::new(1),
                TimeSpan::new(0, 900).expect("a valid span"),
                vec!["satır".to_owned()],
            )]),
            webvtt: "WEBVTT\n\n".to_owned(),
            cache_identity: CacheKey::from_bytes([3u8; 32]),
        }
    }

    /// The bytes must be a pure function of the record: the content address
    /// means nothing otherwise.
    #[test]
    fn serialization_is_deterministic() {
        let record = record();
        let first = serialize(&record).expect("serializes");
        for _ in 0..8 {
            assert_eq!(serialize(&record).expect("serializes"), first);
        }
    }

    #[test]
    fn a_record_round_trips_through_the_wire_format() {
        let record = record();
        let bytes = serialize(&record).expect("serializes");
        assert_eq!(deserialize(&bytes).expect("deserializes"), record);
    }

    /// A file written by a future format version is refused, not misread as
    /// this one.
    #[test]
    fn an_unknown_format_version_is_refused() {
        let bytes = serialize(&record()).expect("serializes");
        let text = String::from_utf8(bytes).expect("utf-8");
        let bumped = text.replacen(
            &format!("\"format\":{FORMAT_VERSION}"),
            &format!("\"format\":{}", FORMAT_VERSION + 1),
            1,
        );
        assert_ne!(bumped, text, "the fixture must actually change the version");

        assert_eq!(
            deserialize(bumped.as_bytes()),
            Err(ArtifactStoreError::Corrupt)
        );
    }

    /// Every malformed shape reports the same typed, payload-free reason.
    #[test]
    fn malformed_bytes_are_refused() {
        let text = String::from_utf8(serialize(&record()).expect("serializes")).expect("utf-8");

        let cases = [
            // Not JSON at all.
            "not json".to_owned(),
            // Truncated — what a non-atomic partial write would leave.
            text[..text.len() / 2].to_owned(),
            // A language tag the domain refuses.
            text.replacen("\"source_language\":\"en\"", "\"source_language\":\"\"", 1),
            // A time span the domain refuses (end before start).
            text.replacen("\"end_ms\":900", "\"end_ms\":0", 1),
            // A provider identity the port refuses.
            text.replacen("\"model\":\"echo-1\"", "\"model\":\"\"", 1),
            // A fingerprint that is not a 32-byte digest.
            text.replacen(&to_hex(&[7u8; 32]), "abcd", 1),
        ];

        for (index, case) in cases.iter().enumerate() {
            assert_ne!(*case, text, "case {index} did not modify the fixture");
            assert_eq!(
                deserialize(case.as_bytes()),
                Err(ArtifactStoreError::Corrupt),
                "case {index} was accepted"
            );
        }
    }
}
