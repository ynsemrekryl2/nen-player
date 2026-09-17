//! Provider-neutral media identity lookup port (NEN-033, ADR-0040).
//!
//! The port carries only bounded, typed values. Hashes, credentials and media
//! titles are sensitive and therefore have hand-written redacted formatting.

use std::fmt;

/// The eight-byte OpenSubtitles-compatible hash sent to a provider.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MediaHash([u8; 8]);

impl MediaHash {
    pub const fn from_bytes(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 8] {
        &self.0
    }
}

impl fmt::Debug for MediaHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MediaHash(<redacted>)")
    }
}

impl fmt::Display for MediaHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

/// Identity fields returned only after an exact provider match.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct VerifiedMediaIdentity {
    pub title: String,
    pub year: Option<u16>,
    pub season: Option<u16>,
    pub episode: Option<u16>,
}

impl fmt::Debug for VerifiedMediaIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VerifiedMediaIdentity")
            .field("title", &"<present>")
            .field("year", &self.year.is_some())
            .field("season", &self.season.is_some())
            .field("episode", &self.episode.is_some())
            .finish()
    }
}

/// A provider-canonical identifier collected from a trusted media declaration
/// such as a handoff or `.nfo` sidecar. It is never a subtitle or download id.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CanonicalMediaIdentity {
    pub imdb_id: Option<String>,
    pub parent_imdb_id: Option<String>,
}

impl fmt::Debug for CanonicalMediaIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CanonicalMediaIdentity")
            .field("imdb_id", &self.imdb_id.as_ref().map(|_| "<present>"))
            .field(
                "parent_imdb_id",
                &self.parent_imdb_id.as_ref().map(|_| "<present>"),
            )
            .finish()
    }
}

/// A bounded title/year/season/episode provider query derived from weak or
/// non-canonical evidence such as a release filename. It must remain distinct
/// from [`VerifiedMediaIdentity`] so a parsed name can never be treated as an
/// exact media identity.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ParsedMediaIdentity {
    pub title: String,
    pub year: Option<u16>,
    pub season: Option<u16>,
    pub episode: Option<u16>,
}

impl fmt::Debug for ParsedMediaIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParsedMediaIdentity")
            .field("title", &"<present>")
            .field("year", &self.year.is_some())
            .field("season", &self.season.is_some())
            .field("episode", &self.episode.is_some())
            .finish()
    }
}

/// The provider's bounded answer to one hash lookup.
#[derive(Clone, PartialEq, Eq)]
pub enum IdentityLookup {
    Match(VerifiedMediaIdentity),
    NoMatch,
    Ambiguous,
}

impl fmt::Debug for IdentityLookup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IdentityLookup")
            .field(&match self {
                Self::Match(_) => "match",
                Self::NoMatch => "no_match",
                Self::Ambiguous => "ambiguous",
            })
            .finish()
    }
}

/// Provider failures contain no response, URL, credential or media payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityLookupError {
    InvalidCredential,
    Transport,
    HttpStatus,
    InvalidResponse,
    ResponseTooLarge,
    RedirectRejected,
}

impl fmt::Display for IdentityLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidCredential => "invalid provider credential",
            Self::Transport => "provider transport failed",
            Self::HttpStatus => "provider request failed",
            Self::InvalidResponse => "provider response was invalid",
            Self::ResponseTooLarge => "provider response was too large",
            Self::RedirectRejected => "provider redirect was rejected",
        })
    }
}

impl std::error::Error for IdentityLookupError {}

/// Resolves an exact media hash without prescribing a provider implementation.
pub trait MediaIdentityLookup: Send + Sync {
    fn lookup_by_hash(&self, hash: MediaHash) -> Result<IdentityLookup, IdentityLookupError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_debug_and_errors_carry_no_provider_payload() {
        let identity = VerifiedMediaIdentity {
            title: "Private Filename.mkv".into(),
            year: Some(2020),
            season: None,
            episode: None,
        };
        assert!(!format!("{identity:?}").contains("Private Filename"));
        assert!(!format!("{:?}", IdentityLookup::Match(identity)).contains("Private Filename"));
        let canonical = CanonicalMediaIdentity {
            imdb_id: Some("tt1375666".into()),
            parent_imdb_id: Some("tt7654321".into()),
        };
        let parsed = ParsedMediaIdentity {
            title: "Private Filename".into(),
            year: Some(2020),
            season: None,
            episode: None,
        };
        assert!(!format!("{canonical:?}").contains("1375666"));
        assert!(!format!("{parsed:?}").contains("Private Filename"));
        assert_eq!(
            IdentityLookupError::InvalidResponse.to_string(),
            "provider response was invalid"
        );
        assert!(!format!("{:?}", IdentityLookupError::InvalidResponse).contains("Filename"));
    }
}
