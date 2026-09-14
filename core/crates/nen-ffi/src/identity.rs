//! Verified media identity's single FFI boundary (NEN-120, ADR-0040).
//!
//! The app layer owns evidence, credential lookup and the OpenSubtitles
//! adapter. This module only translates the bounded result to the shell;
//! media hashes, API keys, URLs and provider payloads never appear in the
//! returned records or errors.

use crate::credentials::FfiSecureCredentialStore;
use crate::remote_evidence::{adapt_http_client, ForeignHttpClient};
use nen_app::identity::{ProviderIdentityError, ProviderIdentityOutcome};
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

/// A provider lookup's non-error result. `NoCredential` is deliberately a
/// status, not an exception: playback and local identity evidence work
/// without an OpenSubtitles key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiIdentityLookupStatus {
    Match,
    NoMatch,
    Ambiguous,
    NoCredential,
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
