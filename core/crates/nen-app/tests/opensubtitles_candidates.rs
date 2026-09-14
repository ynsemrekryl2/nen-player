use nen_app::identity::{
    search_opensubtitles_candidates, ProviderCandidateError, ProviderCandidateOutcome,
};
use nen_app::ports::credentials::{
    ApiKey, CredentialKind, InMemoryCredentialStore, SecureCredentialStore,
};
use nen_app::ports::http::{HttpClient, HttpError, HttpRequest, HttpResponse};
use nen_app::ports::identity::MediaHash;
use nen_app::ports::subtitle_candidates::SubtitleCandidate;
use nen_app::subtitles::SubtitleLibrary;
use nen_domain::source::{LanguageTag, SubtitlePreferences};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

const HASH: MediaHash = MediaHash::from_bytes([0, 1, 2, 3, 4, 5, 6, 7]);

fn language(value: &str) -> LanguageTag {
    LanguageTag::parse(value).expect("language")
}

fn candidate(id: &str, file_id: u64, language_code: &str) -> SubtitleCandidate {
    SubtitleCandidate {
        public_id: id.into(),
        private_file_id: file_id,
        language: language(language_code),
        release_name: Some("Fixture release".into()),
        hearing_impaired: true,
        ai_translated: false,
    }
}

#[test]
fn candidates_are_catalogued_in_preference_groups_without_documents() {
    let mut library = SubtitleLibrary::new();
    let tokens = library.add_opensubtitles(vec![
        candidate("public-en", 7001, "en"),
        candidate("public-tr", 7002, "tr"),
    ]);

    assert_eq!(tokens, [1, 2]);
    assert_eq!(library.catalog().len(), 2);
    assert_eq!(library.opensubtitles_file_id(tokens[0]), Some(7001));
    assert_eq!(library.opensubtitles_file_id(tokens[1]), Some(7002));
    assert!(tokens
        .iter()
        .all(|token| library.document_of(*token).is_none()));
    assert!(tokens.iter().all(|token| library.is_token_usable(*token)));

    let sections = library.menu(&SubtitlePreferences::new(Some(language("tr")), None));
    assert!(sections.iter().any(|section| {
        section
            .entries
            .iter()
            .any(|entry| entry.kind == nen_domain::source::SubtitleSourceKind::OpenSubtitles)
    }));
    let entries: Vec<_> = sections
        .iter()
        .flat_map(|section| section.entries.iter())
        .filter(|entry| entry.kind == nen_domain::source::SubtitleSourceKind::OpenSubtitles)
        .collect();
    assert!(entries[0].hearing_impaired);
    assert!(!entries[0].ai_translated);
    assert_eq!(entries[0].label, "Fixture release");
}

#[test]
fn same_public_id_upserts_without_losing_token_or_document_free_state() {
    let mut library = SubtitleLibrary::new();
    let first = library.add_opensubtitles(vec![candidate("same-public", 7001, "en")])[0];
    let second = library.add_opensubtitles(vec![candidate("same-public", 7002, "tr")])[0];

    assert_eq!(first, second);
    assert_eq!(library.catalog().len(), 1);
    assert_eq!(library.opensubtitles_file_id(first), Some(7002));
    assert!(library.document_of(first).is_none());
}

struct CountingClient {
    sends: AtomicUsize,
    response: HttpResponse,
}

struct RecordingClient {
    requests: Mutex<Vec<HttpRequest>>,
    responses: Mutex<Vec<HttpResponse>>,
}

impl HttpClient for RecordingClient {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        self.requests.lock().expect("request lock").push(request);
        Ok(self.responses.lock().expect("response lock").remove(0))
    }
}

impl HttpClient for CountingClient {
    fn send(&self, _request: HttpRequest) -> Result<HttpResponse, HttpError> {
        self.sends.fetch_add(1, Ordering::SeqCst);
        Ok(self.response.clone())
    }
}

fn empty_response() -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: Vec::new(),
        body: br#"{"data":[]}"#.to_vec(),
    }
}

#[test]
fn missing_key_returns_without_a_provider_send() {
    let credentials = InMemoryCredentialStore::new();
    let client = CountingClient {
        sends: AtomicUsize::new(0),
        response: empty_response(),
    };
    let outcome = search_opensubtitles_candidates(
        Some(HASH),
        None,
        vec![language("en")],
        &credentials,
        &client,
    )
    .expect("keyless search is ordinary");

    assert_eq!(outcome, ProviderCandidateOutcome::NoCredential);
    assert_eq!(client.sends.load(Ordering::SeqCst), 0);
}

#[test]
fn malformed_provider_response_is_typed_and_does_not_mutate_catalog() {
    let credentials = InMemoryCredentialStore::new();
    credentials
        .set(
            CredentialKind::OpenSubtitles,
            ApiKey::new("fixture-key").expect("key"),
        )
        .expect("credential store");
    let client = CountingClient {
        sends: AtomicUsize::new(0),
        response: HttpResponse {
            status_code: 200,
            headers: Vec::new(),
            body: b"{".to_vec(),
        },
    };
    let result =
        search_opensubtitles_candidates(Some(HASH), None, Vec::new(), &credentials, &client);

    assert_eq!(
        result,
        Err(ProviderCandidateError::Provider(
            nen_app::ports::subtitle_candidates::SubtitleCandidateSearchError::InvalidResponse
        ))
    );
    let mut library = SubtitleLibrary::new();
    if let Ok(ProviderCandidateOutcome::Candidates(candidates)) = result.clone() {
        library.add_opensubtitles(candidates);
    }
    assert_eq!(library.catalog().len(), 0);
    assert_eq!(client.sends.load(Ordering::SeqCst), 1);
}

#[test]
fn an_exact_hash_miss_falls_back_to_verified_identity_in_order() {
    let credentials = InMemoryCredentialStore::new();
    credentials
        .set(
            CredentialKind::OpenSubtitles,
            ApiKey::new("fixture-key").expect("key"),
        )
        .expect("credential store");
    let client = RecordingClient {
        requests: Mutex::new(Vec::new()),
        responses: Mutex::new(vec![empty_response(), empty_response()]),
    };
    let outcome = search_opensubtitles_candidates(
        Some(HASH),
        Some(nen_app::ports::identity::VerifiedMediaIdentity {
            title: "Synthetic Film".into(),
            year: Some(2020),
            season: None,
            episode: None,
        }),
        vec![language("tr")],
        &credentials,
        &client,
    )
    .expect("fallback search");

    assert_eq!(outcome, ProviderCandidateOutcome::Candidates(Vec::new()));
    let requests = client.requests.lock().expect("request lock");
    assert_eq!(requests.len(), 2);
    assert!(requests[0].url.contains("moviehash="));
    assert!(requests[1].url.contains("query=Synthetic%20Film"));
}

#[test]
fn missing_search_identity_returns_before_credential_or_http_access() {
    let credentials = InMemoryCredentialStore::new();
    let client = CountingClient {
        sends: AtomicUsize::new(0),
        response: empty_response(),
    };
    let result = search_opensubtitles_candidates(None, None, Vec::new(), &credentials, &client);

    assert_eq!(result, Ok(ProviderCandidateOutcome::NoIdentity));
    assert_eq!(client.sends.load(Ordering::SeqCst), 0);
}
