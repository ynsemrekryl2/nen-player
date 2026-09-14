//! Application orchestration for provider-backed media identity (NEN-033).

use nen_identity::evidence::MediaEvidence;
use nen_identity::os_hash::OsHash;
use nen_ports::credentials::{CredentialKind, CredentialStoreError, SecureCredentialStore};
use nen_ports::http::HttpClient;
use nen_ports::identity::{IdentityLookup, IdentityLookupError, MediaHash, MediaIdentityLookup};
use nen_providers::opensubtitles::{OpenSubtitlesApiKey, OpenSubtitlesIdentityLookup};

/// The evidence-level answer to an optional provider identity lookup.
///
/// `NoCredential` is an ordinary outcome rather than an error: identity is
/// supplementary evidence and the player must remain useful without a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderIdentityOutcome {
    Match(nen_ports::identity::VerifiedMediaIdentity),
    NoMatch,
    Ambiguous,
    NoCredential,
}

/// Why an identity lookup could not complete. All variants are payload-free;
/// provider responses, URLs and credential values never leave their adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderIdentityError {
    CredentialStore(CredentialStoreError),
    Provider(IdentityLookupError),
    RemoteEvidence(crate::remote_evidence::RemoteEvidenceError),
}

impl std::fmt::Display for ProviderIdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CredentialStore(error) => error.fmt(f),
            Self::Provider(error) => error.fmt(f),
            Self::RemoteEvidence(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ProviderIdentityError {}

/// Computes the provider-compatible media hash from two bounded windows.
/// Reading the windows remains the platform adapter's job; this keeps the
/// NEN-018 algorithm in its owning crate while allowing the FFI composition
/// root to pass the result into cache identity.
pub fn media_hash_from_windows(file_size: u64, head: &[u8], tail: &[u8]) -> Option<MediaHash> {
    nen_identity::os_hash::of(file_size, head, tail)
        .ok()
        .map(|hash| MediaHash::from_bytes(*hash.as_bytes()))
}

/// Applies an exact provider match to evidence while preserving local fallback
/// resolution for `NoMatch`, `Ambiguous` and typed provider failures.
pub fn apply_provider_identity(
    evidence: MediaEvidence,
    lookup: &dyn MediaIdentityLookup,
) -> (MediaEvidence, Result<IdentityLookup, IdentityLookupError>) {
    let Some(hash) = evidence.os_hash() else {
        return (evidence, Ok(IdentityLookup::NoMatch));
    };

    let request = MediaHash::from_bytes(*hash.as_bytes());
    let result = lookup.lookup_by_hash(request);
    if let Ok(IdentityLookup::Match(identity)) = &result {
        let identity = identity.clone();
        let evidence = evidence.with_verified_identity(
            identity.title,
            identity.year,
            identity.season,
            identity.episode,
        );
        return (evidence, result);
    }
    (evidence, result)
}

/// Runs an already-composed lookup only when the OpenSubtitles credential is
/// present. This generic seam keeps the credential gate testable with the
/// deterministic `FakeMediaIdentityLookup` without contacting a provider.
pub fn apply_provider_identity_with_credentials(
    evidence: MediaEvidence,
    credentials: &dyn SecureCredentialStore,
    lookup: &dyn MediaIdentityLookup,
) -> Result<(MediaEvidence, ProviderIdentityOutcome), ProviderIdentityError> {
    if evidence.os_hash().is_none() {
        return Ok((evidence, ProviderIdentityOutcome::NoMatch));
    }
    if credentials
        .get(CredentialKind::OpenSubtitles)
        .map_err(ProviderIdentityError::CredentialStore)?
        .is_none()
    {
        return Ok((evidence, ProviderIdentityOutcome::NoCredential));
    }
    finish_provider_identity(evidence, lookup)
}

/// Composes the real OpenSubtitles lookup for a local hash. A missing hash is
/// returned before credential or network access, so an unidentifiable medium
/// produces no provider request.
pub fn lookup_opensubtitles_by_hash(
    hash: Option<MediaHash>,
    credentials: &dyn SecureCredentialStore,
    http: &dyn HttpClient,
) -> Result<(MediaEvidence, ProviderIdentityOutcome), ProviderIdentityError> {
    let Some(hash) = hash else {
        return Ok((MediaEvidence::default(), ProviderIdentityOutcome::NoMatch));
    };

    let evidence = MediaEvidence::default().with_os_hash(OsHash::from_bytes(*hash.as_bytes()));
    let Some(key) = credentials
        .get(CredentialKind::OpenSubtitles)
        .map_err(ProviderIdentityError::CredentialStore)?
    else {
        return Ok((evidence, ProviderIdentityOutcome::NoCredential));
    };
    let key = OpenSubtitlesApiKey::new(key.expose()).map_err(ProviderIdentityError::Provider)?;
    let lookup = OpenSubtitlesIdentityLookup::new(http, key);
    finish_provider_identity(evidence, &lookup)
}

/// Collects bounded remote evidence and then performs the same exact-hash
/// lookup. Credential presence is checked first so a keyless install does not
/// issue even the optional remote evidence requests.
pub fn lookup_opensubtitles_for_remote_url(
    url: &str,
    credentials: &dyn SecureCredentialStore,
    http: &dyn HttpClient,
) -> Result<(MediaEvidence, ProviderIdentityOutcome), ProviderIdentityError> {
    let Some(key) = credentials
        .get(CredentialKind::OpenSubtitles)
        .map_err(ProviderIdentityError::CredentialStore)?
    else {
        return Ok((
            MediaEvidence::for_remote_url(url),
            ProviderIdentityOutcome::NoCredential,
        ));
    };
    let evidence = crate::remote_evidence::collect_remote_evidence(http, url)
        .map_err(ProviderIdentityError::RemoteEvidence)?;
    let key = OpenSubtitlesApiKey::new(key.expose()).map_err(ProviderIdentityError::Provider)?;
    let lookup = OpenSubtitlesIdentityLookup::new(http, key);
    finish_provider_identity(evidence, &lookup)
}

fn finish_provider_identity(
    evidence: MediaEvidence,
    lookup: &dyn MediaIdentityLookup,
) -> Result<(MediaEvidence, ProviderIdentityOutcome), ProviderIdentityError> {
    let (evidence, result) = apply_provider_identity(evidence, lookup);
    let outcome = match result {
        Ok(IdentityLookup::Match(identity)) => ProviderIdentityOutcome::Match(identity),
        Ok(IdentityLookup::NoMatch) => ProviderIdentityOutcome::NoMatch,
        Ok(IdentityLookup::Ambiguous) => ProviderIdentityOutcome::Ambiguous,
        Err(error) => return Err(ProviderIdentityError::Provider(error)),
    };
    Ok((evidence, outcome))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nen_identity::os_hash;
    use nen_ports::credentials::{ApiKey, InMemoryCredentialStore};
    use nen_ports::identity::{IdentityLookupError, VerifiedMediaIdentity};
    use nen_providers::opensubtitles::FakeMediaIdentityLookup;

    fn evidence() -> MediaEvidence {
        let contents = vec![7_u8; os_hash::MIN_FILE_BYTES as usize];
        let hash = os_hash::of_bytes(&contents).expect("hash");
        MediaEvidence::for_local_file("/media/filename.1999.mkv").with_os_hash(hash)
    }

    #[test]
    fn exact_match_becomes_the_strongest_evidence_layer() {
        let provider =
            FakeMediaIdentityLookup::new(Ok(IdentityLookup::Match(VerifiedMediaIdentity {
                title: "Verified Film".into(),
                year: Some(2010),
                season: None,
                episode: None,
            })));
        let (evidence, result) = apply_provider_identity(evidence(), &provider);
        assert!(matches!(result, Ok(IdentityLookup::Match(_))));
        assert_eq!(evidence.resolve().title.as_deref(), Some("Verified Film"));
        assert_eq!(provider.calls(), 1);
    }

    #[test]
    fn no_match_keeps_the_existing_local_fallback() {
        let provider = FakeMediaIdentityLookup::new(Ok(IdentityLookup::NoMatch));
        let (evidence, result) = apply_provider_identity(evidence(), &provider);
        assert_eq!(result, Ok(IdentityLookup::NoMatch));
        assert_eq!(evidence.resolve().title.as_deref(), Some("filename"));
    }

    #[test]
    fn missing_hash_does_not_call_the_provider() {
        let provider =
            FakeMediaIdentityLookup::new(Ok(IdentityLookup::Match(VerifiedMediaIdentity {
                title: "Never used".into(),
                year: None,
                season: None,
                episode: None,
            })));
        let evidence = MediaEvidence::for_local_file("/media/filename.1999.mkv");
        let (evidence, result) = apply_provider_identity(evidence, &provider);
        assert_eq!(result, Ok(IdentityLookup::NoMatch));
        assert_eq!(provider.calls(), 0);
        assert_eq!(evidence.resolve().title.as_deref(), Some("filename"));
    }

    #[test]
    fn provider_error_keeps_local_fallback() {
        let provider = FakeMediaIdentityLookup::new(Err(IdentityLookupError::Transport));
        let (evidence, result) = apply_provider_identity(evidence(), &provider);
        assert_eq!(result, Err(IdentityLookupError::Transport));
        assert_eq!(evidence.resolve().title.as_deref(), Some("filename"));
    }

    #[test]
    fn credentialed_fake_match_updates_evidence() {
        let credentials = InMemoryCredentialStore::new();
        credentials
            .set(
                CredentialKind::OpenSubtitles,
                ApiKey::new("fixture-key").expect("fixture key"),
            )
            .expect("credential store");
        let provider =
            FakeMediaIdentityLookup::new(Ok(IdentityLookup::Match(VerifiedMediaIdentity {
                title: "Verified Film".into(),
                year: Some(2010),
                season: None,
                episode: None,
            })));

        let (evidence, outcome) =
            apply_provider_identity_with_credentials(evidence(), &credentials, &provider)
                .expect("identity lookup");

        assert_eq!(
            outcome,
            ProviderIdentityOutcome::Match(VerifiedMediaIdentity {
                title: "Verified Film".into(),
                year: Some(2010),
                season: None,
                episode: None,
            })
        );
        assert_eq!(evidence.resolve().title.as_deref(), Some("Verified Film"));
        assert_eq!(provider.calls(), 1);
    }

    #[test]
    fn missing_credential_does_not_call_lookup() {
        let credentials = InMemoryCredentialStore::new();
        let provider =
            FakeMediaIdentityLookup::new(Ok(IdentityLookup::Match(VerifiedMediaIdentity {
                title: "Never used".into(),
                year: None,
                season: None,
                episode: None,
            })));

        let (evidence, outcome) =
            apply_provider_identity_with_credentials(evidence(), &credentials, &provider)
                .expect("missing credential is an ordinary outcome");

        assert_eq!(outcome, ProviderIdentityOutcome::NoCredential);
        assert_eq!(provider.calls(), 0);
        assert_eq!(evidence.resolve().title.as_deref(), Some("filename"));
    }

    #[test]
    fn missing_hash_does_not_read_or_call_provider() {
        let credentials = InMemoryCredentialStore::new();
        let provider = FakeMediaIdentityLookup::new(Ok(IdentityLookup::NoMatch));
        let evidence = MediaEvidence::for_local_file("/media/filename.mkv");

        let (_, outcome) =
            apply_provider_identity_with_credentials(evidence, &credentials, &provider)
                .expect("missing hash is an ordinary outcome");

        assert_eq!(outcome, ProviderIdentityOutcome::NoMatch);
        assert_eq!(provider.calls(), 0);
    }
}
