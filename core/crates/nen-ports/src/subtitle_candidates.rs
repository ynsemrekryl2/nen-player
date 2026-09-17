//! Provider-neutral OpenSubtitles candidate search port (NEN-121, ADR-0021).
//!
//! Search returns metadata only. The public subtitle id, language, display
//! metadata and closed badge flags may be projected into the catalog; the
//! private file id is retained only by the application/provider composition
//! and has a redacted `Debug` surface.

use crate::identity::{
    CanonicalMediaIdentity, MediaHash, ParsedMediaIdentity, VerifiedMediaIdentity,
};
use nen_domain::source::LanguageTag;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

pub const MAX_CANDIDATES: usize = 16;
pub const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
pub const MAX_RELEASE_NAME_CHARS: usize = 256;

#[derive(Clone, PartialEq, Eq)]
pub enum SubtitleCandidateSearchQuery {
    Hash(MediaHash),
    VerifiedIdentity(VerifiedMediaIdentity),
    CanonicalIdentity(CanonicalMediaIdentity),
    ParsedIdentity(ParsedMediaIdentity),
}

impl SubtitleCandidateSearchQuery {
    pub fn by_hash(hash: MediaHash) -> Self {
        Self::Hash(hash)
    }

    pub fn by_verified_identity(identity: VerifiedMediaIdentity) -> Self {
        Self::VerifiedIdentity(identity)
    }

    pub fn by_canonical_identity(identity: CanonicalMediaIdentity) -> Self {
        Self::CanonicalIdentity(identity)
    }

    pub fn by_parsed_identity(identity: ParsedMediaIdentity) -> Self {
        Self::ParsedIdentity(identity)
    }
}

impl From<VerifiedMediaIdentity> for SubtitleCandidateSearchQuery {
    fn from(value: VerifiedMediaIdentity) -> Self {
        Self::VerifiedIdentity(value)
    }
}

impl fmt::Debug for SubtitleCandidateSearchQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SubtitleCandidateSearchQuery")
            .field(match self {
                Self::Hash(_) => &"hash",
                Self::VerifiedIdentity(_) => &"verified_identity",
                Self::CanonicalIdentity(_) => &"canonical_identity",
                Self::ParsedIdentity(_) => &"parsed_identity",
            })
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SubtitleCandidateSearchRequest {
    pub query: SubtitleCandidateSearchQuery,
    pub languages: Vec<LanguageTag>,
}

impl SubtitleCandidateSearchRequest {
    pub fn new(query: SubtitleCandidateSearchQuery) -> Self {
        Self {
            query,
            languages: Vec::new(),
        }
    }

    pub fn with_languages(mut self, languages: Vec<LanguageTag>) -> Self {
        self.languages = languages;
        self
    }
}

impl fmt::Debug for SubtitleCandidateSearchRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubtitleCandidateSearchRequest")
            .field("query", &self.query)
            .field("language_count", &self.languages.len())
            .finish()
    }
}

/// Search metadata plus the private file id needed by the later download
/// task. This type never crosses the FFI boundary.
#[derive(Clone, PartialEq, Eq)]
pub struct SubtitleCandidate {
    pub public_id: String,
    pub private_file_id: u64,
    pub language: LanguageTag,
    pub release_name: Option<String>,
    pub hearing_impaired: bool,
    pub ai_translated: bool,
}

impl fmt::Debug for SubtitleCandidate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubtitleCandidate")
            .field("public_id", &"<opaque>")
            .field("private_file_id", &"<redacted>")
            .field("language", &self.language)
            .field(
                "release_name",
                &self.release_name.as_ref().map(|_| "<present>"),
            )
            .field("hearing_impaired", &self.hearing_impaired)
            .field("ai_translated", &self.ai_translated)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtitleCandidateSearchError {
    InvalidRequest,
    Transport,
    HttpStatus,
    InvalidResponse,
    ResponseTooLarge,
    RedirectRejected,
}

impl fmt::Display for SubtitleCandidateSearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidRequest => "invalid candidate search request",
            Self::Transport => "candidate provider transport failed",
            Self::HttpStatus => "candidate provider request failed",
            Self::InvalidResponse => "candidate provider response was invalid",
            Self::ResponseTooLarge => "candidate provider response was too large",
            Self::RedirectRejected => "candidate provider redirect was rejected",
        })
    }
}

impl std::error::Error for SubtitleCandidateSearchError {}

pub trait SubtitleCandidateSearch: Send + Sync {
    fn search(
        &self,
        request: &SubtitleCandidateSearchRequest,
    ) -> Result<Vec<SubtitleCandidate>, SubtitleCandidateSearchError>;
}

pub struct FakeSubtitleCandidateSearch {
    answer: Result<Vec<SubtitleCandidate>, SubtitleCandidateSearchError>,
    calls: AtomicUsize,
}

impl FakeSubtitleCandidateSearch {
    pub fn new(answer: Result<Vec<SubtitleCandidate>, SubtitleCandidateSearchError>) -> Self {
        Self {
            answer,
            calls: AtomicUsize::new(0),
        }
    }

    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }
}

impl SubtitleCandidateSearch for FakeSubtitleCandidateSearch {
    fn search(
        &self,
        _request: &SubtitleCandidateSearchRequest,
    ) -> Result<Vec<SubtitleCandidate>, SubtitleCandidateSearchError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.answer.clone()
    }
}

pub mod contract {
    use super::{SubtitleCandidateSearch, SubtitleCandidateSearchRequest, MAX_CANDIDATES};
    use std::collections::HashSet;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ContractViolation {
        CallFailed,
        TooManyCandidates,
        DuplicatePublicId,
        NonDeterministic,
    }

    pub fn check(
        search: &dyn SubtitleCandidateSearch,
        request: &SubtitleCandidateSearchRequest,
    ) -> Result<(), Vec<ContractViolation>> {
        let first = match search.search(request) {
            Ok(value) => value,
            Err(_) => return Err(vec![ContractViolation::CallFailed]),
        };
        let second = match search.search(request) {
            Ok(value) => value,
            Err(_) => return Err(vec![ContractViolation::CallFailed]),
        };

        let mut failures = Vec::new();
        if first.len() > MAX_CANDIDATES {
            failures.push(ContractViolation::TooManyCandidates);
        }
        let ids = first
            .iter()
            .map(|candidate| &candidate.public_id)
            .collect::<HashSet<_>>();
        if ids.len() != first.len() {
            failures.push(ContractViolation::DuplicatePublicId);
        }
        if first != second {
            failures.push(ContractViolation::NonDeterministic);
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: &str) -> SubtitleCandidate {
        SubtitleCandidate {
            public_id: id.into(),
            private_file_id: 7,
            language: LanguageTag::parse("en").expect("language"),
            release_name: Some("WEB-DL".into()),
            hearing_impaired: false,
            ai_translated: false,
        }
    }

    #[test]
    fn fake_passes_the_shared_contract() {
        let fake = FakeSubtitleCandidateSearch::new(Ok(vec![candidate("public")]));
        let request = SubtitleCandidateSearchRequest::new(SubtitleCandidateSearchQuery::by_hash(
            MediaHash::from_bytes([0; 8]),
        ));
        contract::check(&fake, &request).expect("contract passes");
        assert_eq!(fake.calls(), 2);
    }

    #[test]
    fn debug_hides_private_and_display_values() {
        let candidate = candidate("public-sentinel");
        let printed = format!("{candidate:?}");
        assert!(!printed.contains("public-sentinel"));
        assert!(!printed.contains("WEB-DL"));
        assert!(!printed.contains('7'));
    }
}
