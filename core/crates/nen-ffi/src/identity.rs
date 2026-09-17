//! Verified media identity's single FFI boundary (NEN-120, ADR-0040).
//!
//! The app layer owns evidence, credential lookup and the OpenSubtitles
//! adapter. This module only translates the bounded result to the shell;
//! media hashes, API keys, URLs and provider payloads never appear in the
//! returned records or errors.

use crate::credentials::FfiSecureCredentialStore;
use crate::remote_evidence::{adapt_http_client, ForeignHttpClient};
use crate::subtitles::FfiSubtitleLibrary;
use nen_app::domain::source::LanguageTag;
use nen_app::identity::{
    CandidateSearchMethod, ProviderCandidateError, ProviderCandidateOutcome, ProviderIdentityError,
    ProviderIdentityOutcome,
};
use nen_app::ports::identity::{MediaHash, VerifiedMediaIdentity};
use std::fmt;
use std::sync::Arc;

/// The identity fields that may be shown by a later shell task. No provider
/// identifiers or transport metadata cross this record.
#[derive(Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiVerifiedMediaIdentity {
    pub title: String,
    pub year: Option<u16>,
    pub season: Option<u16>,
    pub episode: Option<u16>,
}

impl fmt::Debug for FfiVerifiedMediaIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FfiVerifiedMediaIdentity")
            .field("title", &"<present>")
            .field("year", &self.year.is_some())
            .field("season", &self.season.is_some())
            .field("episode", &self.episode.is_some())
            .finish()
    }
}

impl From<VerifiedMediaIdentity> for FfiVerifiedMediaIdentity {
    fn from(value: VerifiedMediaIdentity) -> Self {
        Self {
            title: value.title,
            year: value.year,
            season: value.season,
            episode: value.episode,
        }
    }
}

impl From<FfiVerifiedMediaIdentity> for VerifiedMediaIdentity {
    fn from(value: FfiVerifiedMediaIdentity) -> Self {
        Self {
            title: value.title,
            year: value.year,
            season: value.season,
            episode: value.episode,
        }
    }
}

/// A provider lookup's non-error result. `NoCredential` is deliberately a
/// status, not an exception: playback and local identity evidence work
/// without an OpenSubtitles key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiIdentityLookupStatus {
    Match,
    NoMatch,
    Ambiguous,
    NoCredential,
    /// No hash could be derived for the medium, so the provider was never
    /// asked (NEN-131 shows this apart from a provider miss).
    NoHash,
}

/// The bounded identity result consumed by the shell.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiIdentityLookupResult {
    pub status: FfiIdentityLookupStatus,
    pub identity: Option<FfiVerifiedMediaIdentity>,
}

/// Flat, payload-free failures for the optional identity worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Error)]
pub enum FfiIdentityLookupError {
    InvalidMediaHash,
    CredentialStoreUnavailable,
    CredentialStoreDenied,
    CredentialStoreCorrupt,
    ProviderInvalidCredential,
    ProviderTransport,
    ProviderHttpStatus,
    ProviderInvalidResponse,
    ProviderResponseTooLarge,
    ProviderRedirectRejected,
    RemoteInvalidUrl,
    RemoteInsecureRedirect,
    RemoteMissingRedirectLocation,
    RemoteRedirectLimitExceeded,
    RemoteHttpStatus,
    RemoteInvalidResponse,
    RemoteResponseTooLarge,
    RemoteTransport,
}

impl fmt::Display for FfiIdentityLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidMediaHash => "invalid_media_hash",
            Self::CredentialStoreUnavailable => "credential_store_unavailable",
            Self::CredentialStoreDenied => "credential_store_denied",
            Self::CredentialStoreCorrupt => "credential_store_corrupt",
            Self::ProviderInvalidCredential => "provider_invalid_credential",
            Self::ProviderTransport => "provider_transport",
            Self::ProviderHttpStatus => "provider_http_status",
            Self::ProviderInvalidResponse => "provider_invalid_response",
            Self::ProviderResponseTooLarge => "provider_response_too_large",
            Self::ProviderRedirectRejected => "provider_redirect_rejected",
            Self::RemoteInvalidUrl => "remote_invalid_url",
            Self::RemoteInsecureRedirect => "remote_insecure_redirect",
            Self::RemoteMissingRedirectLocation => "remote_missing_redirect_location",
            Self::RemoteRedirectLimitExceeded => "remote_redirect_limit_exceeded",
            Self::RemoteHttpStatus => "remote_http_status",
            Self::RemoteInvalidResponse => "remote_invalid_response",
            Self::RemoteResponseTooLarge => "remote_response_too_large",
            Self::RemoteTransport => "remote_transport",
        })
    }
}

impl std::error::Error for FfiIdentityLookupError {}

/// The optional candidate catalog's ordinary outcome. An empty catalog is
/// represented by `Cataloged`; missing credentials and missing identity stay
/// silent states rather than shell errors (ADR-0021).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiCandidateSearchStatus {
    Cataloged,
    NoCredential,
    NoIdentity,
}

/// Which ADR-0021 query a candidate search issued — a method name only,
/// never the hash or identity it carried.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiCandidateSearchMethod {
    Hash,
    CanonicalIdentity,
    VerifiedIdentity,
    ParsedIdentity,
}

/// What a candidate search did and what it found, for the shell's event log
/// (NEN-131). Counts and method names only: candidates themselves live in the
/// receiving library, private file ids never leave the Rust side.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiCandidateSearchReport {
    pub status: FfiCandidateSearchStatus,
    pub candidate_count: u32,
    pub attempted: Vec<FfiCandidateSearchMethod>,
    pub found_by: Option<FfiCandidateSearchMethod>,
}

impl From<CandidateSearchMethod> for FfiCandidateSearchMethod {
    fn from(value: CandidateSearchMethod) -> Self {
        match value {
            CandidateSearchMethod::Hash => Self::Hash,
            CandidateSearchMethod::CanonicalIdentity => Self::CanonicalIdentity,
            CandidateSearchMethod::VerifiedIdentity => Self::VerifiedIdentity,
            CandidateSearchMethod::ParsedIdentity => Self::ParsedIdentity,
        }
    }
}

/// Flat failures from the optional OpenSubtitles metadata search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Error)]
pub enum FfiCandidateSearchError {
    CredentialStoreUnavailable,
    CredentialStoreDenied,
    CredentialStoreCorrupt,
    InvalidRequest,
    Transport,
    HttpStatus,
    InvalidResponse,
    ResponseTooLarge,
    RedirectRejected,
    RemoteInvalidUrl,
    RemoteInsecureRedirect,
    RemoteMissingRedirectLocation,
    RemoteRedirectLimitExceeded,
    RemoteHttpStatus,
    RemoteInvalidResponse,
    RemoteResponseTooLarge,
    RemoteTransport,
}

impl fmt::Display for FfiCandidateSearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::CredentialStoreUnavailable => "credential_store_unavailable",
            Self::CredentialStoreDenied => "credential_store_denied",
            Self::CredentialStoreCorrupt => "credential_store_corrupt",
            Self::InvalidRequest => "invalid_request",
            Self::Transport => "transport",
            Self::HttpStatus => "http_status",
            Self::InvalidResponse => "invalid_response",
            Self::ResponseTooLarge => "response_too_large",
            Self::RedirectRejected => "redirect_rejected",
            Self::RemoteInvalidUrl => "remote_invalid_url",
            Self::RemoteInsecureRedirect => "remote_insecure_redirect",
            Self::RemoteMissingRedirectLocation => "remote_missing_redirect_location",
            Self::RemoteRedirectLimitExceeded => "remote_redirect_limit_exceeded",
            Self::RemoteHttpStatus => "remote_http_status",
            Self::RemoteInvalidResponse => "remote_invalid_response",
            Self::RemoteResponseTooLarge => "remote_response_too_large",
            Self::RemoteTransport => "remote_transport",
        })
    }
}

impl std::error::Error for FfiCandidateSearchError {}

impl From<ProviderCandidateError> for FfiCandidateSearchError {
    fn from(value: ProviderCandidateError) -> Self {
        match value {
            ProviderCandidateError::CredentialStore(error) => match error {
                nen_app::ports::credentials::CredentialStoreError::Unavailable => {
                    Self::CredentialStoreUnavailable
                }
                nen_app::ports::credentials::CredentialStoreError::Denied => {
                    Self::CredentialStoreDenied
                }
                nen_app::ports::credentials::CredentialStoreError::Corrupt => {
                    Self::CredentialStoreCorrupt
                }
            },
            ProviderCandidateError::Provider(error) => match error {
                nen_app::ports::subtitle_candidates::SubtitleCandidateSearchError::InvalidRequest => {
                    Self::InvalidRequest
                }
                nen_app::ports::subtitle_candidates::SubtitleCandidateSearchError::Transport => {
                    Self::Transport
                }
                nen_app::ports::subtitle_candidates::SubtitleCandidateSearchError::HttpStatus => {
                    Self::HttpStatus
                }
                nen_app::ports::subtitle_candidates::SubtitleCandidateSearchError::InvalidResponse => {
                    Self::InvalidResponse
                }
                nen_app::ports::subtitle_candidates::SubtitleCandidateSearchError::ResponseTooLarge => {
                    Self::ResponseTooLarge
                }
                nen_app::ports::subtitle_candidates::SubtitleCandidateSearchError::RedirectRejected => {
                    Self::RedirectRejected
                }
            },
            ProviderCandidateError::RemoteEvidence(error) => match error {
                nen_app::remote_evidence::RemoteEvidenceError::InvalidUrl => {
                    Self::RemoteInvalidUrl
                }
                nen_app::remote_evidence::RemoteEvidenceError::InsecureRedirect => {
                    Self::RemoteInsecureRedirect
                }
                nen_app::remote_evidence::RemoteEvidenceError::MissingRedirectLocation => {
                    Self::RemoteMissingRedirectLocation
                }
                nen_app::remote_evidence::RemoteEvidenceError::RedirectLimitExceeded => {
                    Self::RemoteRedirectLimitExceeded
                }
                nen_app::remote_evidence::RemoteEvidenceError::HttpStatus => {
                    Self::RemoteHttpStatus
                }
                nen_app::remote_evidence::RemoteEvidenceError::InvalidResponse => {
                    Self::RemoteInvalidResponse
                }
                nen_app::remote_evidence::RemoteEvidenceError::ResponseTooLarge => {
                    Self::RemoteResponseTooLarge
                }
                nen_app::remote_evidence::RemoteEvidenceError::Transport => Self::RemoteTransport,
            },
        }
    }
}

impl From<ProviderIdentityError> for FfiIdentityLookupError {
    fn from(value: ProviderIdentityError) -> Self {
        match value {
            ProviderIdentityError::CredentialStore(error) => match error {
                nen_app::ports::credentials::CredentialStoreError::Unavailable => {
                    Self::CredentialStoreUnavailable
                }
                nen_app::ports::credentials::CredentialStoreError::Denied => {
                    Self::CredentialStoreDenied
                }
                nen_app::ports::credentials::CredentialStoreError::Corrupt => {
                    Self::CredentialStoreCorrupt
                }
            },
            ProviderIdentityError::Provider(error) => match error {
                nen_app::ports::identity::IdentityLookupError::InvalidCredential => {
                    Self::ProviderInvalidCredential
                }
                nen_app::ports::identity::IdentityLookupError::Transport => Self::ProviderTransport,
                nen_app::ports::identity::IdentityLookupError::HttpStatus => {
                    Self::ProviderHttpStatus
                }
                nen_app::ports::identity::IdentityLookupError::InvalidResponse => {
                    Self::ProviderInvalidResponse
                }
                nen_app::ports::identity::IdentityLookupError::ResponseTooLarge => {
                    Self::ProviderResponseTooLarge
                }
                nen_app::ports::identity::IdentityLookupError::RedirectRejected => {
                    Self::ProviderRedirectRejected
                }
            },
            ProviderIdentityError::RemoteEvidence(error) => match error {
                nen_app::remote_evidence::RemoteEvidenceError::InvalidUrl => Self::RemoteInvalidUrl,
                nen_app::remote_evidence::RemoteEvidenceError::InsecureRedirect => {
                    Self::RemoteInsecureRedirect
                }
                nen_app::remote_evidence::RemoteEvidenceError::MissingRedirectLocation => {
                    Self::RemoteMissingRedirectLocation
                }
                nen_app::remote_evidence::RemoteEvidenceError::RedirectLimitExceeded => {
                    Self::RemoteRedirectLimitExceeded
                }
                nen_app::remote_evidence::RemoteEvidenceError::HttpStatus => Self::RemoteHttpStatus,
                nen_app::remote_evidence::RemoteEvidenceError::InvalidResponse => {
                    Self::RemoteInvalidResponse
                }
                nen_app::remote_evidence::RemoteEvidenceError::ResponseTooLarge => {
                    Self::RemoteResponseTooLarge
                }
                nen_app::remote_evidence::RemoteEvidenceError::Transport => Self::RemoteTransport,
            },
        }
    }
}

fn result(outcome: ProviderIdentityOutcome) -> FfiIdentityLookupResult {
    match outcome {
        ProviderIdentityOutcome::Match(identity) => FfiIdentityLookupResult {
            status: FfiIdentityLookupStatus::Match,
            identity: Some(identity.into()),
        },
        ProviderIdentityOutcome::NoMatch => FfiIdentityLookupResult {
            status: FfiIdentityLookupStatus::NoMatch,
            identity: None,
        },
        ProviderIdentityOutcome::Ambiguous => FfiIdentityLookupResult {
            status: FfiIdentityLookupStatus::Ambiguous,
            identity: None,
        },
        ProviderIdentityOutcome::NoCredential => FfiIdentityLookupResult {
            status: FfiIdentityLookupStatus::NoCredential,
            identity: None,
        },
        ProviderIdentityOutcome::NoHash => FfiIdentityLookupResult {
            status: FfiIdentityLookupStatus::NoHash,
            identity: None,
        },
    }
}

/// Looks up a hash already computed by the platform's bounded window reader.
/// `None` returns before credential or HTTP access.
#[uniffi::export]
pub fn lookup_verified_identity_by_hash(
    media_hash: Option<Vec<u8>>,
    credential_store: Arc<FfiSecureCredentialStore>,
    http_client: Arc<dyn ForeignHttpClient>,
) -> Result<FfiIdentityLookupResult, FfiIdentityLookupError> {
    let media_hash = media_hash
        .map(|bytes| {
            bytes
                .try_into()
                .map(MediaHash::from_bytes)
                .map_err(|_| FfiIdentityLookupError::InvalidMediaHash)
        })
        .transpose()?;
    let credentials =
        credential_store.as_ref() as &dyn nen_app::ports::credentials::SecureCredentialStore;
    let http = adapt_http_client(http_client);
    let (_, outcome) =
        nen_app::identity::lookup_opensubtitles_by_hash(media_hash, credentials, http.as_ref())
            .map_err(FfiIdentityLookupError::from)?;
    Ok(result(outcome))
}

/// Collects bounded remote evidence and performs the same optional identity
/// lookup without returning the media hash or URL to the shell.
#[uniffi::export]
pub fn lookup_verified_remote_identity(
    url: String,
    credential_store: Arc<FfiSecureCredentialStore>,
    http_client: Arc<dyn ForeignHttpClient>,
) -> Result<FfiIdentityLookupResult, FfiIdentityLookupError> {
    let credentials =
        credential_store.as_ref() as &dyn nen_app::ports::credentials::SecureCredentialStore;
    let http = adapt_http_client(http_client);
    let (_, outcome) =
        nen_app::identity::lookup_opensubtitles_for_remote_url(&url, credentials, http.as_ref())
            .map_err(FfiIdentityLookupError::from)?;
    Ok(result(outcome))
}

/// Searches and catalogs OpenSubtitles metadata into the receiving library.
/// The library may be a worker snapshot; the shell decides when that snapshot
/// is safe to merge into the live medium. Private provider file ids remain in
/// the Rust application library and never appear in this API.
#[uniffi::export]
pub fn search_opensubtitles_candidates(
    media_hash: Option<Vec<u8>>,
    identity: Option<FfiVerifiedMediaIdentity>,
    languages: Vec<String>,
    credential_store: Arc<FfiSecureCredentialStore>,
    http_client: Arc<dyn ForeignHttpClient>,
    library: Arc<FfiSubtitleLibrary>,
) -> Result<FfiCandidateSearchReport, FfiCandidateSearchError> {
    let media_hash = media_hash
        .map(|bytes| {
            bytes
                .try_into()
                .map(MediaHash::from_bytes)
                .map_err(|_| FfiCandidateSearchError::InvalidRequest)
        })
        .transpose()?;
    let identity = identity.map(Into::into);
    let languages = languages
        .into_iter()
        .filter_map(|tag| LanguageTag::parse(&tag).ok())
        .collect();
    let credentials =
        credential_store.as_ref() as &dyn nen_app::ports::credentials::SecureCredentialStore;
    let http = adapt_http_client(http_client);
    let outcome = nen_app::identity::search_opensubtitles_candidates(
        media_hash,
        identity,
        languages,
        credentials,
        http.as_ref(),
    )
    .map_err(FfiCandidateSearchError::from)?;

    let silent = |status| FfiCandidateSearchReport {
        status,
        candidate_count: 0,
        attempted: Vec::new(),
        found_by: None,
    };
    match outcome {
        ProviderCandidateOutcome::Candidates {
            candidates,
            attempted,
            found_by,
        } => {
            let candidate_count = u32::try_from(candidates.len()).unwrap_or(u32::MAX);
            library.with_mut(|library| library.add_opensubtitles(candidates));
            Ok(FfiCandidateSearchReport {
                status: FfiCandidateSearchStatus::Cataloged,
                candidate_count,
                attempted: attempted.into_iter().map(Into::into).collect(),
                found_by: found_by.map(Into::into),
            })
        }
        ProviderCandidateOutcome::NoCredential => {
            Ok(silent(FfiCandidateSearchStatus::NoCredential))
        }
        ProviderCandidateOutcome::NoIdentity => Ok(silent(FfiCandidateSearchStatus::NoIdentity)),
    }
}

/// Searches candidates after the core has collected the media locator's
/// filename, URL path, redirect and Content-Disposition evidence. The raw
/// locator is consumed inside the core and never appears in the returned
/// report or its errors.
#[uniffi::export]
pub fn search_opensubtitles_candidates_for_media(
    media_locator: String,
    media_hash: Option<Vec<u8>>,
    identity: Option<FfiVerifiedMediaIdentity>,
    languages: Vec<String>,
    credential_store: Arc<FfiSecureCredentialStore>,
    http_client: Arc<dyn ForeignHttpClient>,
    library: Arc<FfiSubtitleLibrary>,
) -> Result<FfiCandidateSearchReport, FfiCandidateSearchError> {
    let media_hash = media_hash
        .map(|bytes| {
            bytes
                .try_into()
                .map(MediaHash::from_bytes)
                .map_err(|_| FfiCandidateSearchError::InvalidRequest)
        })
        .transpose()?;
    let identity = identity.map(Into::into);
    let languages = languages
        .into_iter()
        .filter_map(|tag| LanguageTag::parse(&tag).ok())
        .collect();
    let credentials =
        credential_store.as_ref() as &dyn nen_app::ports::credentials::SecureCredentialStore;
    let http = adapt_http_client(http_client);
    let outcome = nen_app::identity::search_opensubtitles_candidates_for_media(
        &media_locator,
        media_hash,
        identity,
        languages,
        credentials,
        http.as_ref(),
    )
    .map_err(FfiCandidateSearchError::from)?;

    let silent = |status| FfiCandidateSearchReport {
        status,
        candidate_count: 0,
        attempted: Vec::new(),
        found_by: None,
    };
    match outcome {
        ProviderCandidateOutcome::Candidates {
            candidates,
            attempted,
            found_by,
        } => {
            let candidate_count = u32::try_from(candidates.len()).unwrap_or(u32::MAX);
            library.with_mut(|library| library.add_opensubtitles(candidates));
            Ok(FfiCandidateSearchReport {
                status: FfiCandidateSearchStatus::Cataloged,
                candidate_count,
                attempted: attempted.into_iter().map(Into::into).collect(),
                found_by: found_by.map(Into::into),
            })
        }
        ProviderCandidateOutcome::NoCredential => {
            Ok(silent(FfiCandidateSearchStatus::NoCredential))
        }
        ProviderCandidateOutcome::NoIdentity => Ok(silent(FfiCandidateSearchStatus::NoIdentity)),
    }
}
