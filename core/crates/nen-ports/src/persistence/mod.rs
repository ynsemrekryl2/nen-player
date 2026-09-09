//! Persistence port for validated translation artifacts (NEN-096, ADR-0017).
//!
//! ADR-0017 decides the storage model: the filesystem, no database; one
//! artifact is **one canonical file** holding its metadata, its normalized
//! cues and its WebVTT together, addressed by the `blake3` hash of that
//! file's own full content. This module carries the two halves that decision
//! needs at the port boundary — [`ContentAddress`], the address, and
//! [`ArtifactRecord`], what lives at it — plus the [`ArtifactStore`] trait an
//! adapter implements.
//!
//! The record lives here rather than in the adapter because ADR-0006 confines
//! `nen-persist` to `nen-domain` + `nen-ports`: it cannot see
//! `nen_translate::artifact::ValidatedSubtitleArtifact`. `nen-translate`
//! projects a validated artifact into an [`ArtifactRecord`]
//! (`ValidatedSubtitleArtifact::to_record`) and the store persists that.
//!
//! **The projection is deliberately one-way.** Reading returns an
//! [`ArtifactRecord`], never a `ValidatedSubtitleArtifact` — NEN-094's
//! invariant that only `assemble` can produce a validated artifact stays
//! true at the type level, and bytes read back off a disk this process does
//! not own can never impersonate one.
//!
//! Nothing here carries cache identity semantics: that is ADR-0018's subject
//! and `NEN-097`'s task, and ADR-0018 is not `accepted` yet (Kural 4). The
//! record's fields are exactly the projection of the getters
//! `ValidatedSubtitleArtifact` already exposes today.

pub mod contract;

use nen_domain::source::LanguageTag;
use nen_domain::subtitle::SubtitleDocument;
use std::fmt;

use crate::identity::MediaHash;
use crate::translation::TranslationProviderIdentity;

/// The largest artifact file a store will read back. A translated subtitle
/// document is text: a three-hour film's WebVTT plus its cue list stays far
/// below this, so the bound only exists to keep a corrupt or hostile file
/// from being loaded whole. `http::MAX_RESPONSE_BYTES` is the precedent.
pub const MAX_ARTIFACT_BYTES: usize = 16 * 1024 * 1024;

/// Length of a [`ContentAddress`] in hex characters.
const ADDRESS_HEX_LEN: usize = 64;

/// The content address of a stored artifact: the `blake3` digest of the
/// artifact file's full content (ADR-0017 Karar 2).
///
/// `Debug` and `Display` print `<redacted>`, following [`MediaHash`]. That is
/// not caution for its own sake: the address *is* the artifact's file name,
/// and K23 #3/#8 (`docs/security-policy.md`) forbid a private file name or
/// hash on a log surface. The real hex is reachable only through the
/// explicitly named [`ContentAddress::to_hex`] / [`ContentAddress::as_bytes`],
/// so a stray `{}` in a log line cannot leak it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentAddress([u8; 32]);

impl ContentAddress {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Parses exactly 64 lowercase hex characters and nothing else.
    ///
    /// This is the first of the two layers ADR-0017 Karar 4 asks for: a
    /// value that could name anything but a digest — `..`, a path separator,
    /// a percent escape, an uppercase digit, a short or long string — never
    /// becomes a `ContentAddress`, so it can never reach a path at all. The
    /// adapter's canonical-root comparison is the second layer.
    pub fn from_hex(value: &str) -> Result<Self, ContentAddressError> {
        if value.len() != ADDRESS_HEX_LEN {
            return Err(ContentAddressError::Length);
        }
        let mut bytes = [0u8; 32];
        for (index, byte) in bytes.iter_mut().enumerate() {
            let pair = value
                .get(index * 2..index * 2 + 2)
                .ok_or(ContentAddressError::Length)?;
            let mut value = 0u8;
            for digit in pair.bytes() {
                let nibble = match digit {
                    b'0'..=b'9' => digit - b'0',
                    b'a'..=b'f' => digit - b'a' + 10,
                    _ => return Err(ContentAddressError::NotHex),
                };
                value = (value << 4) | nibble;
            }
            *byte = value;
        }
        Ok(Self(bytes))
    }

    /// Lowercase hex, 64 characters. Named rather than `Display` on purpose —
    /// see the type's own documentation.
    pub fn to_hex(&self) -> String {
        let mut out = String::with_capacity(ADDRESS_HEX_LEN);
        for byte in self.0 {
            out.push(hex_digit(byte >> 4));
            out.push(hex_digit(byte & 0x0f));
        }
        out
    }
}

const fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'a' + nibble - 10) as char,
    }
}

impl fmt::Debug for ContentAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ContentAddress(<redacted>)")
    }
}

impl fmt::Display for ContentAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

/// Why a string could not become a [`ContentAddress`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentAddressError {
    /// Not exactly [`ADDRESS_HEX_LEN`] characters long.
    Length,
    /// Contains something other than a lowercase hex digit.
    NotHex,
}

impl fmt::Display for ContentAddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Length => "content address is not 64 characters long",
            Self::NotHex => "content address is not lowercase hex",
        })
    }
}

impl std::error::Error for ContentAddressError {}

/// One stored artifact: everything ADR-0017 Karar 2 keeps in a single file.
///
/// Every field is the projection of a getter
/// `nen_translate::artifact::ValidatedSubtitleArtifact` already exposes. The
/// fingerprints arrive as raw digests because `nen-subtitle`'s
/// `SourceFingerprint`/`TimelineFingerprint` live above this crate; a
/// consumer that wants the typed form rebuilds it from these bytes.
#[derive(Clone, PartialEq, Eq)]
pub struct ArtifactRecord {
    pub source_fingerprint: [u8; 32],
    pub timeline_fingerprint: [u8; 32],
    pub source_language: LanguageTag,
    pub target_language: LanguageTag,
    pub provider: TranslationProviderIdentity,
    pub pipeline_version: u32,
    pub block_layout_version: u32,
    /// `None` is `GlossaryIdentity::None`; M5 has no glossary write surface.
    pub glossary: Option<String>,
    pub media_hash: Option<MediaHash>,
    pub created_at_unix_ms: u64,
    /// The translated document: source cue IDs and timings, translated text.
    pub document: SubtitleDocument,
    pub webvtt: String,
}

impl fmt::Debug for ArtifactRecord {
    /// Shape only: the document and the WebVTT are translated dialogue
    /// (K23 #4), and the fingerprints are private hashes of the user's own
    /// media (K23 #8).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArtifactRecord")
            .field("source_fingerprint", &"<redacted>")
            .field("timeline_fingerprint", &"<redacted>")
            .field("source_language", &self.source_language)
            .field("target_language", &self.target_language)
            .field("provider", &self.provider)
            .field("pipeline_version", &self.pipeline_version)
            .field("block_layout_version", &self.block_layout_version)
            .field("glossary", &self.glossary.is_some())
            .field("media_hash", &self.media_hash)
            .field("created_at_unix_ms", &self.created_at_unix_ms)
            .field("cue_count", &self.document.len())
            .field("webvtt_len", &self.webvtt.len())
            .finish()
    }
}

/// Why a store refused a read or a write.
///
/// Flat, `Copy` and payload-free, like every other port error in this crate:
/// **no variant carries a path, a file name or an address** (K23 #3). A
/// caller that needs to know *which* artifact failed already holds its
/// [`ContentAddress`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactStoreError {
    /// No artifact is stored at this address.
    NotFound,
    /// The underlying filesystem refused the operation.
    Io,
    /// Bytes were read but are not a well-formed artifact, or do not hash to
    /// the address they were read from.
    Corrupt,
    /// The address resolved outside the injected store root (ADR-0017
    /// Karar 4).
    OutsideRoot,
    /// The stored file exceeds [`MAX_ARTIFACT_BYTES`].
    TooLarge,
}

impl fmt::Display for ArtifactStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NotFound => "no artifact is stored at that address",
            Self::Io => "the artifact store could not complete a filesystem operation",
            Self::Corrupt => "the stored artifact is not well-formed",
            Self::OutsideRoot => "the address resolves outside the store root",
            Self::TooLarge => "the stored artifact is larger than the read bound",
        })
    }
}

impl std::error::Error for ArtifactStoreError {}

/// A content-addressed store for validated translation artifacts.
///
/// Synchronous and runtime-free, like every port in this crate. `put` is
/// required to be atomic: no reader may ever observe a partially written
/// artifact at an address, whatever happens mid-write (ADR-0017 Karar 3).
pub trait ArtifactStore: Send + Sync {
    /// Writes `record` and returns the address its content hashes to.
    /// Writing the same content twice keeps one copy and returns the same
    /// address both times.
    fn put(&self, record: &ArtifactRecord) -> Result<ContentAddress, ArtifactStoreError>;

    /// Reads back the artifact stored at `address`.
    fn get(&self, address: ContentAddress) -> Result<ArtifactRecord, ArtifactStoreError>;

    /// Whether an artifact is stored at `address`.
    fn contains(&self, address: ContentAddress) -> Result<bool, ArtifactStoreError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips() {
        let address = ContentAddress::from_bytes([0xab; 32]);
        let hex = address.to_hex();
        assert_eq!(hex.len(), ADDRESS_HEX_LEN);
        assert_eq!(ContentAddress::from_hex(&hex), Ok(address));
    }

    #[test]
    fn hex_covers_every_nibble() {
        let mut bytes = [0u8; 32];
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = (index as u8) * 8;
        }
        let address = ContentAddress::from_bytes(bytes);
        assert_eq!(ContentAddress::from_hex(&address.to_hex()), Ok(address));
    }

    #[test]
    fn a_traversal_string_is_not_an_address() {
        for candidate in [
            "../../etc/passwd",
            "..",
            "/etc/passwd",
            "%2e%2e%2f",
            "..\\windows",
        ] {
            assert!(
                ContentAddress::from_hex(candidate).is_err(),
                "traversal candidate parsed as an address: {candidate}"
            );
        }
    }

    #[test]
    fn only_exactly_64_lowercase_hex_characters_parse() {
        let valid = "0".repeat(ADDRESS_HEX_LEN);
        assert!(ContentAddress::from_hex(&valid).is_ok());

        assert_eq!(
            ContentAddress::from_hex(&"0".repeat(ADDRESS_HEX_LEN - 1)),
            Err(ContentAddressError::Length)
        );
        assert_eq!(
            ContentAddress::from_hex(&"0".repeat(ADDRESS_HEX_LEN + 1)),
            Err(ContentAddressError::Length)
        );
        assert_eq!(
            ContentAddress::from_hex(""),
            Err(ContentAddressError::Length)
        );

        // Uppercase is rejected: one artifact, one spelling of its address.
        let uppercase = format!("{}A", "0".repeat(ADDRESS_HEX_LEN - 1));
        assert_eq!(
            ContentAddress::from_hex(&uppercase),
            Err(ContentAddressError::NotHex)
        );

        // A separator inside an otherwise correctly sized string.
        let separator = format!("{}/", "0".repeat(ADDRESS_HEX_LEN - 1));
        assert_eq!(
            ContentAddress::from_hex(&separator),
            Err(ContentAddressError::NotHex)
        );

        // Multi-byte characters must not slip past the length check.
        let multibyte = format!("{}é", "0".repeat(ADDRESS_HEX_LEN - 2));
        assert!(ContentAddress::from_hex(&multibyte).is_err());
    }
}
