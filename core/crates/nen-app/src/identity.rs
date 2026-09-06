//! Application orchestration for provider-backed media identity (NEN-033).

use nen_identity::evidence::MediaEvidence;
use nen_ports::identity::{IdentityLookup, IdentityLookupError, MediaHash, MediaIdentityLookup};

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

#[cfg(test)]
mod tests {
    use super::*;
    use nen_identity::os_hash;
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
}
