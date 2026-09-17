use nen_app::identity::{
    search_opensubtitles_candidates, search_opensubtitles_candidates_for_media,
    search_opensubtitles_candidates_with_evidence, CandidateSearchMethod, ProviderCandidateError,
    ProviderCandidateOutcome,
};
use nen_app::ports::credentials::{
    ApiKey, CredentialKind, InMemoryCredentialStore, SecureCredentialStore,
};
use nen_app::ports::http::{
    HttpClient, HttpError, HttpHeader, HttpMethod, HttpRequest, HttpResponse,
};
use nen_app::ports::identity::MediaHash;
use nen_app::ports::subtitle_candidates::SubtitleCandidate;
use nen_app::subtitles::SubtitleLibrary;
use nen_domain::source::{LanguageTag, SubtitlePreferences};
use nen_identity::evidence::{HandoffMetadata, MediaEvidence};
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
    if let Ok(ProviderCandidateOutcome::Candidates { candidates, .. }) = result.clone() {
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

    assert_eq!(
        outcome,
        ProviderCandidateOutcome::Candidates {
            candidates: Vec::new(),
            attempted: vec![
                CandidateSearchMethod::Hash,
                CandidateSearchMethod::VerifiedIdentity
            ],
            found_by: None,
        }
    );
    let requests = client.requests.lock().expect("request lock");
    assert_eq!(requests.len(), 2);
    assert!(requests[0].url.contains("moviehash="));
    assert!(requests[1].url.contains("query=Synthetic%20Film"));
}

fn one_candidate_response(hash_match: bool) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: Vec::new(),
        body: format!(
            r#"{{"data":[{{"id":"public-1","attributes":{{"moviehash_match":{hash_match},"language":"tr","files":[{{"file_id":9001}}]}}}}]}}"#
        )
        .into_bytes(),
    }
}

fn synthetic_identity() -> nen_app::ports::identity::VerifiedMediaIdentity {
    nen_app::ports::identity::VerifiedMediaIdentity {
        title: "Synthetic Film".into(),
        year: Some(2020),
        season: None,
        episode: None,
    }
}

#[test]
fn a_hash_hit_reports_hash_and_never_asks_by_identity() {
    let credentials = InMemoryCredentialStore::new();
    credentials
        .set(
            CredentialKind::OpenSubtitles,
            ApiKey::new("fixture-key").expect("key"),
        )
        .expect("credential store");
    let client = RecordingClient {
        requests: Mutex::new(Vec::new()),
        responses: Mutex::new(vec![one_candidate_response(true)]),
    };
    let outcome = search_opensubtitles_candidates(
        Some(HASH),
        Some(synthetic_identity()),
        vec![language("tr")],
        &credentials,
        &client,
    )
    .expect("hash search");

    let ProviderCandidateOutcome::Candidates {
        candidates,
        attempted,
        found_by,
    } = outcome
    else {
        panic!("expected candidates");
    };
    assert_eq!(candidates.len(), 1);
    assert_eq!(attempted, vec![CandidateSearchMethod::Hash]);
    assert_eq!(found_by, Some(CandidateSearchMethod::Hash));
    assert_eq!(client.requests.lock().expect("request lock").len(), 1);
}

#[test]
fn an_identity_fallback_hit_reports_both_attempts_and_the_identity_finder() {
    let credentials = InMemoryCredentialStore::new();
    credentials
        .set(
            CredentialKind::OpenSubtitles,
            ApiKey::new("fixture-key").expect("key"),
        )
        .expect("credential store");
    let client = RecordingClient {
        requests: Mutex::new(Vec::new()),
        responses: Mutex::new(vec![empty_response(), one_candidate_response(false)]),
    };
    let outcome = search_opensubtitles_candidates(
        Some(HASH),
        Some(synthetic_identity()),
        vec![language("tr")],
        &credentials,
        &client,
    )
    .expect("fallback search");

    let ProviderCandidateOutcome::Candidates {
        candidates,
        attempted,
        found_by,
    } = outcome
    else {
        panic!("expected candidates");
    };
    assert_eq!(candidates.len(), 1);
    assert_eq!(
        attempted,
        vec![
            CandidateSearchMethod::Hash,
            CandidateSearchMethod::VerifiedIdentity
        ]
    );
    assert_eq!(found_by, Some(CandidateSearchMethod::VerifiedIdentity));
    assert_eq!(client.requests.lock().expect("request lock").len(), 2);
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

#[test]
fn a_filename_movie_is_a_parsed_fallback_without_release_suffix_leakage() {
    let credentials = InMemoryCredentialStore::new();
    credentials
        .set(
            CredentialKind::OpenSubtitles,
            ApiKey::new("fixture-key").expect("key"),
        )
        .expect("credential store");
    let client = RecordingClient {
        requests: Mutex::new(Vec::new()),
        responses: Mutex::new(vec![one_candidate_response(false)]),
    };
    let evidence =
        MediaEvidence::for_local_file("/library/Private.Movie.2024.1080p.WEB-DL.x264.mkv");
    let outcome = search_opensubtitles_candidates_with_evidence(
        None,
        None,
        Some(&evidence),
        vec![language("tr")],
        &credentials,
        &client,
    )
    .expect("parsed fallback search");

    let ProviderCandidateOutcome::Candidates {
        attempted,
        found_by,
        ..
    } = outcome
    else {
        panic!("expected candidates");
    };
    assert_eq!(attempted, vec![CandidateSearchMethod::ParsedIdentity]);
    assert_eq!(found_by, Some(CandidateSearchMethod::ParsedIdentity));
    let requests = client.requests.lock().expect("request lock");
    assert_eq!(requests.len(), 1);
    assert!(requests[0].url.contains("query=Private%20Movie"));
    assert!(requests[0].url.contains("year=2024"));
    assert!(!requests[0].url.contains("1080p"));
    assert!(!requests[0].url.contains("WEB-DL"));
}

#[test]
fn a_filename_series_carries_season_and_episode_but_not_release_metadata() {
    let credentials = InMemoryCredentialStore::new();
    credentials
        .set(
            CredentialKind::OpenSubtitles,
            ApiKey::new("fixture-key").expect("key"),
        )
        .expect("credential store");
    let client = RecordingClient {
        requests: Mutex::new(Vec::new()),
        responses: Mutex::new(vec![one_candidate_response(false)]),
    };
    let evidence = MediaEvidence::for_local_file("/library/Example.Show.S02E03.720p.mkv");
    let outcome = search_opensubtitles_candidates_with_evidence(
        None,
        None,
        Some(&evidence),
        vec![language("en")],
        &credentials,
        &client,
    )
    .expect("parsed series search");

    let ProviderCandidateOutcome::Candidates { attempted, .. } = outcome else {
        panic!("expected candidates");
    };
    assert_eq!(attempted, vec![CandidateSearchMethod::ParsedIdentity]);
    let requests = client.requests.lock().expect("request lock");
    assert!(requests[0].url.contains("query=Example%20Show"));
    assert!(requests[0].url.contains("season_number=2"));
    assert!(requests[0].url.contains("episode_number=3"));
    assert!(!requests[0].url.contains("720p"));
}

#[test]
fn canonical_sidecar_identity_beats_a_conflicting_filename_query() {
    let credentials = InMemoryCredentialStore::new();
    credentials
        .set(
            CredentialKind::OpenSubtitles,
            ApiKey::new("fixture-key").expect("key"),
        )
        .expect("credential store");
    let client = RecordingClient {
        requests: Mutex::new(Vec::new()),
        responses: Mutex::new(vec![one_candidate_response(false)]),
    };
    let evidence = MediaEvidence::for_local_file("/library/Wrong.Movie.1999.1080p.mkv").with_nfo(
        nen_identity::nfo::parse("<movie><uniqueid type=\"imdb\">tt1375666</uniqueid></movie>"),
    );
    let outcome = search_opensubtitles_candidates_with_evidence(
        None,
        None,
        Some(&evidence),
        vec![language("en")],
        &credentials,
        &client,
    )
    .expect("canonical search");

    let ProviderCandidateOutcome::Candidates {
        attempted,
        found_by,
        ..
    } = outcome
    else {
        panic!("expected candidates");
    };
    assert_eq!(attempted, vec![CandidateSearchMethod::CanonicalIdentity]);
    assert_eq!(found_by, Some(CandidateSearchMethod::CanonicalIdentity));
    let request = &client.requests.lock().expect("request lock")[0];
    assert!(request.url.contains("imdb_id=tt1375666"));
    assert!(!request.url.contains("Wrong"));
    assert!(!request.url.contains("1999"));
}

#[test]
fn handoff_parent_identity_is_preferred_for_series_search() {
    let credentials = InMemoryCredentialStore::new();
    credentials
        .set(
            CredentialKind::OpenSubtitles,
            ApiKey::new("fixture-key").expect("key"),
        )
        .expect("credential store");
    let client = RecordingClient {
        requests: Mutex::new(Vec::new()),
        responses: Mutex::new(vec![one_candidate_response(false)]),
    };
    let evidence = MediaEvidence::for_local_file("/library/Wrong.Show.S01E02.mkv").with_handoff(
        HandoffMetadata {
            parent_imdb_id: Some("tt7654321".into()),
            ..HandoffMetadata::default()
        },
    );
    let outcome = search_opensubtitles_candidates_with_evidence(
        None,
        None,
        Some(&evidence),
        vec![language("en")],
        &credentials,
        &client,
    )
    .expect("parent identity search");

    let ProviderCandidateOutcome::Candidates { attempted, .. } = outcome else {
        panic!("expected candidates");
    };
    assert_eq!(attempted, vec![CandidateSearchMethod::CanonicalIdentity]);
    let request = &client.requests.lock().expect("request lock")[0];
    assert!(request.url.contains("parent_imdb_id=tt7654321"));
    assert!(!request.url.contains("Wrong"));
}

#[test]
fn remote_declared_name_is_used_without_query_or_host_data() {
    let credentials = InMemoryCredentialStore::new();
    credentials
        .set(
            CredentialKind::OpenSubtitles,
            ApiKey::new("fixture-key").expect("key"),
        )
        .expect("credential store");
    let client = RecordingClient {
        requests: Mutex::new(Vec::new()),
        responses: Mutex::new(vec![
            HttpResponse {
                status_code: 200,
                headers: vec![
                    HttpHeader {
                        name: "Content-Disposition".into(),
                        value: "attachment; filename=Remote.Movie.2024.mkv".into(),
                    },
                    HttpHeader {
                        name: "Accept-Ranges".into(),
                        value: "none".into(),
                    },
                ],
                body: Vec::new(),
            },
            one_candidate_response(false),
        ]),
    };
    let outcome = search_opensubtitles_candidates_for_media(
        "https://private.example/opaque/stream.mkv?token=SECRET",
        None,
        None,
        vec![language("tr")],
        &credentials,
        &client,
    )
    .expect("remote parsed search");

    let ProviderCandidateOutcome::Candidates { attempted, .. } = outcome else {
        panic!("expected candidates");
    };
    assert_eq!(attempted, vec![CandidateSearchMethod::ParsedIdentity]);
    let requests = client.requests.lock().expect("request lock");
    assert_eq!(requests[0].method, HttpMethod::Head);
    assert!(requests[1].url.contains("query=Remote%20Movie"));
    assert!(requests[1].url.contains("year=2024"));
    assert!(!requests[1].url.contains("SECRET"));
    assert!(!requests[1].url.contains("private.example"));
}
