//! Application orchestration for provider-backed media identity (NEN-033).

use nen_identity::evidence::MediaEvidence;
use nen_identity::os_hash::OsHash;
use nen_ports::credentials::{CredentialKind, CredentialStoreError, SecureCredentialStore};
use nen_ports::filename_normalization::{
    FilenameNormalization, FilenameNormalizationError as ProviderFilenameNormalizationError,
    FilenameNormalizationRequest, FilenameNormalizationResult, FilenameNormalizer,
};
use nen_ports::http::HttpClient;
use nen_ports::identity::{
    CanonicalMediaIdentity, IdentityLookup, IdentityLookupError, MediaHash, MediaIdentityLookup,
    ParsedMediaIdentity, VerifiedMediaIdentity,
};
use nen_ports::subtitle_candidates::{
    SubtitleCandidate, SubtitleCandidateSearch, SubtitleCandidateSearchError,
    SubtitleCandidateSearchQuery, SubtitleCandidateSearchRequest,
};
use nen_providers::opensubtitles::{
    OpenSubtitlesApiKey, OpenSubtitlesCandidateSearch, OpenSubtitlesIdentityLookup,
};

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
    /// No provider-compatible hash could be derived, so no lookup was even
    /// attempted. Distinct from `NoMatch` — the provider was never asked
    /// (NEN-131 reports both to the user separately).
    NoHash,
}

/// Why an identity lookup could not complete. All variants are payload-free;
/// provider responses, URLs and credential values never leave their adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderIdentityError {
    CredentialStore(CredentialStoreError),
    Provider(IdentityLookupError),
    RemoteEvidence(crate::remote_evidence::RemoteEvidenceError),
}

/// Which ADR-0021 query a candidate search issued. Carried on the outcome so
/// the shell can say what was tried and what answered (NEN-131); it names a
/// method only, never the hash or the identity behind it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateSearchMethod {
    Hash,
    CanonicalIdentity,
    VerifiedIdentity,
    ParsedIdentity,
}

/// The optional OpenSubtitles candidate catalog result. An empty candidate
/// list is a successful search with no subtitles; `NoCredential` and
/// `NoIdentity` are ordinary keyless/unidentifiable states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderCandidateOutcome {
    Candidates {
        candidates: Vec<SubtitleCandidate>,
        /// Every query issued, in order.
        attempted: Vec<CandidateSearchMethod>,
        /// The query whose answer is `candidates`, or `None` when every
        /// attempted query came back empty.
        found_by: Option<CandidateSearchMethod>,
    },
    NoCredential,
    NoIdentity,
}

/// Why the optional candidate catalog could not complete. Provider payloads,
/// URLs and credential values remain inside their adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderCandidateError {
    CredentialStore(CredentialStoreError),
    Provider(SubtitleCandidateSearchError),
    RemoteEvidence(crate::remote_evidence::RemoteEvidenceError),
}

/// The explicit, media-scoped consent gate required by ADR-0046.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilenameNormalizationPermission {
    Denied,
    Granted,
}

/// AI normalization remains a suggestion. In particular, `Suggestion` does
/// not alter `MediaEvidence`, `MediaIdentity` or automatic selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilenameNormalizationOutcome {
    NotNeeded,
    NoPermission,
    NoFilename,
    Unknown,
    Suggestion(FilenameNormalization),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilenameNormalizationError {
    CredentialStore(CredentialStoreError),
    MissingCredential,
    Provider(ProviderFilenameNormalizationError),
    ProviderUnavailable,
}

impl std::fmt::Display for FilenameNormalizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CredentialStore(error) => error.fmt(f),
            Self::MissingCredential => f.write_str("filename normalization credential is missing"),
            Self::Provider(error) => error.fmt(f),
            Self::ProviderUnavailable => f.write_str("filename normalization provider unavailable"),
        }
    }
}

/// Composes the selected real translation provider's HTTP and credential
/// path for filename normalization. The provider is constructed only after
/// the app has checked that deterministic identity is unusable and consent is
/// granted, so a denied request does not even read secure storage.
pub fn normalize_filename_with_selected_provider(
    evidence: MediaEvidence,
    permission: FilenameNormalizationPermission,
    choice: &crate::translation::ProviderChoice,
    credentials: &dyn SecureCredentialStore,
    http: &dyn HttpClient,
) -> Result<(MediaEvidence, FilenameNormalizationOutcome), FilenameNormalizationError> {
    if evidence.resolve().is_usable() {
        return Ok((evidence, FilenameNormalizationOutcome::NotNeeded));
    }
    if permission != FilenameNormalizationPermission::Granted {
        return Ok((evidence, FilenameNormalizationOutcome::NoPermission));
    }
    if evidence.ai_filename_stem().is_none() {
        return Ok((evidence, FilenameNormalizationOutcome::NoFilename));
    }

    match choice {
        crate::translation::ProviderChoice::Mock => normalize_filename_if_unknown(
            evidence,
            permission,
            &nen_providers::filename_normalization::FakeFilenameNormalizer::new(Ok(
                FilenameNormalizationResult::Unknown,
            )),
        ),
        crate::translation::ProviderChoice::OpenAi { model } => {
            let key = credentials
                .get(CredentialKind::OpenAi)
                .map_err(FilenameNormalizationError::CredentialStore)?
                .ok_or(FilenameNormalizationError::MissingCredential)?;
            let normalizer =
                nen_providers::openai::OpenAiTranslationProvider::new(http, key, model)
                    .map_err(|_| FilenameNormalizationError::ProviderUnavailable)?;
            normalize_filename_if_unknown(evidence, permission, &normalizer)
        }
        crate::translation::ProviderChoice::OpenRouter { model } => {
            let key = credentials
                .get(CredentialKind::OpenRouter)
                .map_err(FilenameNormalizationError::CredentialStore)?
                .ok_or(FilenameNormalizationError::MissingCredential)?;
            let normalizer =
                nen_providers::openrouter::OpenRouterTranslationProvider::new(http, key, model)
                    .map_err(|_| FilenameNormalizationError::ProviderUnavailable)?;
            normalize_filename_if_unknown(evidence, permission, &normalizer)
        }
    }
}

impl std::error::Error for FilenameNormalizationError {}

/// Runs the AI fallback only after deterministic evidence is unusable and the
/// user has explicitly granted consent for this media.
pub fn normalize_filename_if_unknown(
    evidence: MediaEvidence,
    permission: FilenameNormalizationPermission,
    normalizer: &dyn FilenameNormalizer,
) -> Result<(MediaEvidence, FilenameNormalizationOutcome), FilenameNormalizationError> {
    if evidence.resolve().is_usable() {
        return Ok((evidence, FilenameNormalizationOutcome::NotNeeded));
    }
    let Some(stem) = evidence.ai_filename_stem() else {
        return Ok((evidence, FilenameNormalizationOutcome::NoFilename));
    };
    if permission != FilenameNormalizationPermission::Granted {
        return Ok((evidence, FilenameNormalizationOutcome::NoPermission));
    }
    let request = FilenameNormalizationRequest::new(stem.as_str())
        .map_err(FilenameNormalizationError::Provider)?;
    let result = normalizer
        .normalize(&request)
        .map_err(FilenameNormalizationError::Provider)?;
    let outcome = match result {
        FilenameNormalizationResult::Normalized(suggestion) => {
            FilenameNormalizationOutcome::Suggestion(suggestion)
        }
        FilenameNormalizationResult::Unknown => FilenameNormalizationOutcome::Unknown,
    };
    Ok((evidence, outcome))
}

impl std::fmt::Display for ProviderCandidateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CredentialStore(error) => error.fmt(f),
            Self::Provider(error) => error.fmt(f),
            Self::RemoteEvidence(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ProviderCandidateError {}

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
        return Ok((evidence, ProviderIdentityOutcome::NoHash));
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
        return Ok((MediaEvidence::default(), ProviderIdentityOutcome::NoHash));
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

/// Searches OpenSubtitles metadata using the ADR-0049 order. Exact hash wins;
/// a canonical IMDb/parent IMDb declaration follows; an exact hash identity
/// and finally the parsed title coordinates are fallback queries. A missing
/// hash, identity and evidence returns before credential or HTTP access.
/// Search never downloads or attaches a subtitle document.
pub fn search_opensubtitles_candidates(
    hash: Option<MediaHash>,
    identity: Option<VerifiedMediaIdentity>,
    languages: Vec<nen_domain::source::LanguageTag>,
    credentials: &dyn SecureCredentialStore,
    http: &dyn HttpClient,
) -> Result<ProviderCandidateOutcome, ProviderCandidateError> {
    search_opensubtitles_candidates_with_evidence(
        hash,
        identity,
        None,
        languages,
        credentials,
        http,
    )
}

/// Searches with all locally collected evidence already available. The
/// evidence resolver supplies the canonical-id and parsed fallbacks without
/// allowing a lower filename layer to replace a stronger declaration.
pub fn search_opensubtitles_candidates_with_evidence(
    hash: Option<MediaHash>,
    identity: Option<VerifiedMediaIdentity>,
    evidence: Option<&MediaEvidence>,
    languages: Vec<nen_domain::source::LanguageTag>,
    credentials: &dyn SecureCredentialStore,
    http: &dyn HttpClient,
) -> Result<ProviderCandidateOutcome, ProviderCandidateError> {
    let canonical = evidence.and_then(canonical_provider_identity);
    let parsed = evidence.and_then(parsed_provider_identity);
    search_opensubtitles_candidates_with_inputs(
        hash,
        identity,
        canonical,
        parsed,
        languages,
        credentials,
        http,
    )
}

/// Collects the same bounded evidence used for remote identity lookup, then
/// searches the provider with its fan-in result. Local paths stay entirely
/// offline apart from the eventual provider search.
pub fn search_opensubtitles_candidates_for_media(
    locator: &str,
    hash: Option<MediaHash>,
    identity: Option<VerifiedMediaIdentity>,
    languages: Vec<nen_domain::source::LanguageTag>,
    credentials: &dyn SecureCredentialStore,
    http: &dyn HttpClient,
) -> Result<ProviderCandidateOutcome, ProviderCandidateError> {
    if is_remote_locator(locator) {
        // Do not issue remote evidence requests on a keyless install. This is
        // the same credential-before-network rule as the existing identity
        // path, and keeps a filename from becoming an accidental probe.
        if credentials
            .get(CredentialKind::OpenSubtitles)
            .map_err(ProviderCandidateError::CredentialStore)?
            .is_none()
        {
            return Ok(ProviderCandidateOutcome::NoCredential);
        }
        let evidence = crate::remote_evidence::collect_remote_evidence(http, locator)
            .map_err(ProviderCandidateError::RemoteEvidence)?;
        return search_opensubtitles_candidates_with_evidence(
            hash,
            identity,
            Some(&evidence),
            languages,
            credentials,
            http,
        );
    }

    let evidence = MediaEvidence::for_local_file(locator);
    search_opensubtitles_candidates_with_evidence(
        hash,
        identity,
        Some(&evidence),
        languages,
        credentials,
        http,
    )
}

fn search_opensubtitles_candidates_with_inputs(
    hash: Option<MediaHash>,
    identity: Option<VerifiedMediaIdentity>,
    canonical: Option<CanonicalMediaIdentity>,
    parsed: Option<ParsedMediaIdentity>,
    languages: Vec<nen_domain::source::LanguageTag>,
    credentials: &dyn SecureCredentialStore,
    http: &dyn HttpClient,
) -> Result<ProviderCandidateOutcome, ProviderCandidateError> {
    if hash.is_none() && identity.is_none() && canonical.is_none() && parsed.is_none() {
        return Ok(ProviderCandidateOutcome::NoIdentity);
    }
    let Some(key) = credentials
        .get(CredentialKind::OpenSubtitles)
        .map_err(ProviderCandidateError::CredentialStore)?
    else {
        return Ok(ProviderCandidateOutcome::NoCredential);
    };
    let key = OpenSubtitlesApiKey::new(key.expose()).map_err(|error| {
        ProviderCandidateError::Provider(match error {
            IdentityLookupError::InvalidCredential => SubtitleCandidateSearchError::InvalidRequest,
            IdentityLookupError::Transport => SubtitleCandidateSearchError::Transport,
            IdentityLookupError::HttpStatus => SubtitleCandidateSearchError::HttpStatus,
            IdentityLookupError::InvalidResponse => SubtitleCandidateSearchError::InvalidResponse,
            IdentityLookupError::ResponseTooLarge => SubtitleCandidateSearchError::ResponseTooLarge,
            IdentityLookupError::RedirectRejected => SubtitleCandidateSearchError::RedirectRejected,
        })
    })?;
    let search = OpenSubtitlesCandidateSearch::new(http, key);

    let mut attempted = Vec::with_capacity(4);
    if let Some(hash) = hash {
        let request =
            SubtitleCandidateSearchRequest::new(SubtitleCandidateSearchQuery::by_hash(hash))
                .with_languages(languages.clone());
        attempted.push(CandidateSearchMethod::Hash);
        let candidates = search
            .search(&request)
            .map_err(ProviderCandidateError::Provider)?;
        if !candidates.is_empty() || (identity.is_none() && canonical.is_none() && parsed.is_none())
        {
            let found_by = (!candidates.is_empty()).then_some(CandidateSearchMethod::Hash);
            return Ok(ProviderCandidateOutcome::Candidates {
                candidates,
                attempted,
                found_by,
            });
        }
    }

    if let Some(canonical) = canonical {
        let request = SubtitleCandidateSearchRequest::new(
            SubtitleCandidateSearchQuery::by_canonical_identity(canonical),
        )
        .with_languages(languages.clone());
        attempted.push(CandidateSearchMethod::CanonicalIdentity);
        let candidates = search
            .search(&request)
            .map_err(ProviderCandidateError::Provider)?;
        if !candidates.is_empty() {
            return Ok(ProviderCandidateOutcome::Candidates {
                candidates,
                attempted,
                found_by: Some(CandidateSearchMethod::CanonicalIdentity),
            });
        }
    }

    if let Some(identity) = identity {
        let request = SubtitleCandidateSearchRequest::new(
            SubtitleCandidateSearchQuery::by_verified_identity(identity),
        )
        .with_languages(languages.clone());
        attempted.push(CandidateSearchMethod::VerifiedIdentity);
        let candidates = search
            .search(&request)
            .map_err(ProviderCandidateError::Provider)?;
        if !candidates.is_empty() {
            return Ok(ProviderCandidateOutcome::Candidates {
                candidates,
                attempted,
                found_by: Some(CandidateSearchMethod::VerifiedIdentity),
            });
        }
    }

    if let Some(parsed) = parsed {
        let request = SubtitleCandidateSearchRequest::new(
            SubtitleCandidateSearchQuery::by_parsed_identity(parsed),
        )
        .with_languages(languages);
        attempted.push(CandidateSearchMethod::ParsedIdentity);
        let candidates = search
            .search(&request)
            .map_err(ProviderCandidateError::Provider)?;
        if !candidates.is_empty() {
            return Ok(ProviderCandidateOutcome::Candidates {
                candidates,
                attempted,
                found_by: Some(CandidateSearchMethod::ParsedIdentity),
            });
        }
    }

    Ok(ProviderCandidateOutcome::Candidates {
        candidates: Vec::new(),
        attempted,
        found_by: None,
    })
}

fn canonical_provider_identity(evidence: &MediaEvidence) -> Option<CanonicalMediaIdentity> {
    let identity = CanonicalMediaIdentity {
        imdb_id: evidence.canonical_imdb_id().map(str::to_owned),
        parent_imdb_id: evidence.canonical_parent_imdb_id().map(str::to_owned),
    };
    (identity.imdb_id.is_some() || identity.parent_imdb_id.is_some()).then_some(identity)
}

fn parsed_provider_identity(evidence: &MediaEvidence) -> Option<ParsedMediaIdentity> {
    let parsed = evidence.resolve();
    Some(ParsedMediaIdentity {
        title: parsed.title?,
        year: parsed.year,
        season: parsed.season,
        episode: parsed.episode,
    })
}

fn is_remote_locator(locator: &str) -> bool {
    locator
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
        || locator
            .get(..8)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"))
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
    use nen_ports::http::{HttpError, HttpRequest, HttpResponse};
    use nen_ports::identity::{IdentityLookupError, VerifiedMediaIdentity};
    use nen_providers::opensubtitles::FakeMediaIdentityLookup;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    struct RecordingHttpClient {
        calls: AtomicUsize,
        requests: Mutex<Vec<HttpRequest>>,
        responses: Mutex<Vec<HttpResponse>>,
    }

    impl RecordingHttpClient {
        fn new() -> Self {
            Self::with_responses(vec![HttpResponse {
                status_code: 200,
                headers: Vec::new(),
                body: br#"{"output_text":"{\"title\":\"Provider Film\",\"year\":2024,\"season\":null,\"episode\":null}"}"#
                    .to_vec(),
            }])
        }

        fn with_responses(responses: Vec<HttpResponse>) -> Self {
            Self {
                calls: AtomicUsize::new(0),
                requests: Mutex::new(Vec::new()),
                responses: Mutex::new(responses),
            }
        }
    }

    impl HttpClient for RecordingHttpClient {
        fn send(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            self.requests
                .lock()
                .expect("recording client lock")
                .push(request);
            let mut responses = self.responses.lock().expect("recording response lock");
            if responses.is_empty() {
                Err(HttpError::Transport)
            } else {
                Ok(responses.remove(0))
            }
        }
    }

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

        assert_eq!(outcome, ProviderIdentityOutcome::NoHash);
        assert_eq!(provider.calls(), 0);
    }

    #[test]
    fn unknown_deterministic_identity_gets_an_untrusted_ai_suggestion() {
        let normalizer = nen_providers::filename_normalization::FakeFilenameNormalizer::new(Ok(
            FilenameNormalizationResult::Normalized(
                FilenameNormalization::new("Private Film".into(), Some(2024), None, None)
                    .expect("fixture result"),
            ),
        ));
        let evidence = MediaEvidence::for_local_file("/media/private-film.mkv");

        let (evidence, outcome) = normalize_filename_if_unknown(
            evidence,
            FilenameNormalizationPermission::Granted,
            &normalizer,
        )
        .expect("normalization");

        assert_eq!(normalizer.calls(), 1);
        assert_eq!(evidence.resolve().title, None);
        let FilenameNormalizationOutcome::Suggestion(suggestion) = outcome else {
            panic!("expected a suggestion")
        };
        assert_eq!(suggestion.title, "Private Film");
        assert_eq!(suggestion.year, Some(2024));
    }

    #[test]
    fn deterministic_identity_skips_ai_provider() {
        let normalizer = nen_providers::filename_normalization::FakeFilenameNormalizer::new(Ok(
            FilenameNormalizationResult::Normalized(
                FilenameNormalization::new("Never used".into(), Some(2024), None, None)
                    .expect("fixture result"),
            ),
        ));
        let evidence = MediaEvidence::for_local_file("/media/Film.2020.mkv");

        let (_, outcome) = normalize_filename_if_unknown(
            evidence,
            FilenameNormalizationPermission::Granted,
            &normalizer,
        )
        .expect("normalization");

        assert_eq!(outcome, FilenameNormalizationOutcome::NotNeeded);
        assert_eq!(normalizer.calls(), 0);
    }

    #[test]
    fn denied_ai_filename_permission_never_calls_provider() {
        let normalizer = nen_providers::filename_normalization::FakeFilenameNormalizer::new(Ok(
            FilenameNormalizationResult::Normalized(
                FilenameNormalization::new("Never used".into(), Some(2024), None, None)
                    .expect("fixture result"),
            ),
        ));
        let evidence = MediaEvidence::for_local_file("/media/private-film.mkv");

        let (_, outcome) = normalize_filename_if_unknown(
            evidence,
            FilenameNormalizationPermission::Denied,
            &normalizer,
        )
        .expect("denied permission is an ordinary outcome");

        assert_eq!(outcome, FilenameNormalizationOutcome::NoPermission);
        assert_eq!(normalizer.calls(), 0);
    }

    #[test]
    fn ai_filename_and_provider_result_stay_out_of_debug_surfaces() {
        let filename = "PRIVATE_FILENAME_SENTINEL";
        let response = "PRIVATE_PROVIDER_RESPONSE_SENTINEL";
        let normalizer = nen_providers::filename_normalization::FakeFilenameNormalizer::new(Ok(
            FilenameNormalizationResult::Normalized(
                FilenameNormalization::new(response.into(), Some(2024), None, None)
                    .expect("fixture result"),
            ),
        ));
        let evidence = MediaEvidence::for_local_file(&format!("/media/{filename}.mkv"));
        let (_, outcome) = normalize_filename_if_unknown(
            evidence.clone(),
            FilenameNormalizationPermission::Granted,
            &normalizer,
        )
        .expect("normalization");

        let printed = format!("{evidence:?} {outcome:?}");
        assert!(!printed.contains(filename));
        assert!(!printed.contains(response));
    }

    #[test]
    fn selected_openai_provider_uses_filename_only_request_and_credential_path() {
        let credentials = InMemoryCredentialStore::new();
        credentials
            .set(
                CredentialKind::OpenAi,
                ApiKey::new("fixture-openai-key").expect("credential"),
            )
            .expect("credential store");
        let http = RecordingHttpClient::new();
        let evidence = MediaEvidence::for_local_file("/private/library/Provider.Film.mkv");

        let (_, outcome) = normalize_filename_with_selected_provider(
            evidence,
            FilenameNormalizationPermission::Granted,
            &crate::translation::ProviderChoice::OpenAi {
                model: "gpt-5.6-luna".into(),
            },
            &credentials,
            &http,
        )
        .expect("provider normalization");

        let FilenameNormalizationOutcome::Suggestion(suggestion) = outcome else {
            panic!("expected provider suggestion")
        };
        assert_eq!(suggestion.title, "Provider Film");
        assert_eq!(suggestion.year, Some(2024));
        assert_eq!(http.calls.load(Ordering::Relaxed), 1);
        let request = http
            .requests
            .lock()
            .expect("recording client lock")
            .first()
            .cloned()
            .expect("provider request");
        let body = String::from_utf8(request.body.expect("request body")).expect("utf8");
        assert!(body.contains("Provider.Film"));
        assert!(!body.contains("/private/library"));
        assert!(!body.contains(".mkv"));
        assert!(!body.contains("fixture-openai-key"));
    }

    #[test]
    fn selected_provider_missing_credential_fails_before_http() {
        let credentials = InMemoryCredentialStore::new();
        let http = RecordingHttpClient::new();

        let result = normalize_filename_with_selected_provider(
            MediaEvidence::for_local_file("/media/private-film.mkv"),
            FilenameNormalizationPermission::Granted,
            &crate::translation::ProviderChoice::OpenAi {
                model: "gpt-5.6-luna".into(),
            },
            &credentials,
            &http,
        );

        assert_eq!(result, Err(FilenameNormalizationError::MissingCredential));
        assert_eq!(http.calls.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn selected_openrouter_provider_preflights_once_then_normalizes() {
        let credentials = InMemoryCredentialStore::new();
        credentials
            .set(
                CredentialKind::OpenRouter,
                ApiKey::new("fixture-openrouter-key").expect("credential"),
            )
            .expect("credential store");
        let http = RecordingHttpClient::with_responses(vec![
            HttpResponse {
                status_code: 200,
                headers: Vec::new(),
                body: br#"{"data":{"supported_parameters":["structured_outputs"]}}"#.to_vec(),
            },
            HttpResponse {
                status_code: 200,
                headers: Vec::new(),
                body: br#"{"choices":[{"message":{"content":"{\"title\":\"Router Film\",\"year\":2023,\"season\":null,\"episode\":null}"}}]}"#.to_vec(),
            },
        ]);

        let (_, outcome) = normalize_filename_with_selected_provider(
            MediaEvidence::for_local_file("/private/library/Router.Film.mkv"),
            FilenameNormalizationPermission::Granted,
            &crate::translation::ProviderChoice::OpenRouter {
                model: "openai/gpt-5.6-luna".into(),
            },
            &credentials,
            &http,
        )
        .expect("provider normalization");

        let FilenameNormalizationOutcome::Suggestion(suggestion) = outcome else {
            panic!("expected provider suggestion")
        };
        assert_eq!(suggestion.title, "Router Film");
        assert_eq!(suggestion.year, Some(2023));
        assert_eq!(http.calls.load(Ordering::Relaxed), 2);
        let requests = http.requests.lock().expect("recording client lock");
        let body = String::from_utf8(
            requests
                .get(1)
                .expect("normalization request")
                .body
                .clone()
                .expect("request body"),
        )
        .expect("utf8");
        assert!(body.contains("Router.Film"));
        assert!(!body.contains(".mkv"));
        assert!(!body.contains("/private/library"));
    }
}
