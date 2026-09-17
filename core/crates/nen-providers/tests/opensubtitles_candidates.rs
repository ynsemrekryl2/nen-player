use nen_ports::http::{HttpClient, HttpError, HttpHeader, HttpMethod, HttpRequest, HttpResponse};
use nen_ports::identity::{
    CanonicalMediaIdentity, MediaHash, ParsedMediaIdentity, VerifiedMediaIdentity,
};
use nen_ports::subtitle_candidates::{
    contract, SubtitleCandidateSearch, SubtitleCandidateSearchError, SubtitleCandidateSearchQuery,
    SubtitleCandidateSearchRequest, MAX_RESPONSE_BYTES,
};
use nen_providers::opensubtitles::{OpenSubtitlesApiKey, OpenSubtitlesCandidateSearch};
use std::sync::Mutex;

const HASH_ENDPOINT: &str = "https://api.opensubtitles.com/api/v1/subtitles?moviehash=0001020304050607&moviehash_match=only";
const HASH_LANG_ENDPOINT: &str = "https://api.opensubtitles.com/api/v1/subtitles?moviehash=0001020304050607&moviehash_match=only&languages=en,tr";
const IDENTITY_ENDPOINT: &str =
    "https://api.opensubtitles.com/api/v1/subtitles?query=Synthetic%20Film&year=2020&languages=tr";
const CANONICAL_ENDPOINT: &str =
    "https://api.opensubtitles.com/api/v1/subtitles?imdb_id=tt1375666&languages=en";
const PARSED_ENDPOINT: &str = "https://api.opensubtitles.com/api/v1/subtitles?query=Example%20Show&season_number=2&episode_number=3&languages=en";

fn hash() -> MediaHash {
    MediaHash::from_bytes([0, 1, 2, 3, 4, 5, 6, 7])
}

fn response(body: &[u8]) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: Vec::new(),
        body: body.to_vec(),
    }
}

struct SequenceClient {
    expected: Mutex<Vec<String>>,
    responses: Mutex<Vec<Result<HttpResponse, HttpError>>>,
    requests: Mutex<Vec<HttpRequest>>,
}

impl SequenceClient {
    fn new(expected: Vec<String>, responses: Vec<Result<HttpResponse, HttpError>>) -> Self {
        Self {
            expected: Mutex::new(expected),
            responses: Mutex::new(responses),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn requests(&self) -> Vec<HttpRequest> {
        self.requests.lock().expect("request lock").clone()
    }
}

impl HttpClient for SequenceClient {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        let expected = self.expected.lock().expect("expected lock").remove(0);
        assert_eq!(request.url, expected);
        self.requests.lock().expect("request lock").push(request);
        self.responses.lock().expect("response lock").remove(0)
    }
}

fn search<'a>(client: &'a dyn HttpClient) -> OpenSubtitlesCandidateSearch<'a> {
    OpenSubtitlesCandidateSearch::new(
        client,
        OpenSubtitlesApiKey::new("fixture-key").expect("valid key"),
    )
}

#[test]
fn fixture_catalogues_bounded_metadata_and_never_calls_download() {
    let client = SequenceClient::new(
        vec![HASH_LANG_ENDPOINT.into()],
        vec![Ok(response(include_bytes!(
            "../../../../fixtures/providers/opensubtitles/candidates.json"
        )))],
    );
    let query = SubtitleCandidateSearchRequest::new(SubtitleCandidateSearchQuery::by_hash(hash()))
        .with_languages(vec![
            nen_domain::source::LanguageTag::parse("en").expect("language"),
            nen_domain::source::LanguageTag::parse("tr-TR").expect("language"),
        ]);
    let candidates = search(&client).search(&query).expect("candidates");

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].public_id, "candidate-en");
    assert_eq!(candidates[0].private_file_id, 7001);
    assert!(candidates[0].hearing_impaired);
    assert_eq!(candidates[1].language.as_str(), "tr");
    assert!(candidates[1].ai_translated);
    assert_eq!(
        candidates[1].release_name.as_deref(),
        Some("BluRay release")
    );
    let requests = client.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, HttpMethod::Get);
    assert!(requests[0].url.contains("/subtitles?"));
    assert!(!requests[0].url.contains("/download"));
}

#[test]
fn real_adapter_passes_the_shared_candidate_contract() {
    let fixture = response(include_bytes!(
        "../../../../fixtures/providers/opensubtitles/candidates.json"
    ));
    let client = SequenceClient::new(
        vec![HASH_ENDPOINT.into(), HASH_ENDPOINT.into()],
        vec![Ok(fixture.clone()), Ok(fixture)],
    );
    let query = SubtitleCandidateSearchRequest::new(SubtitleCandidateSearchQuery::by_hash(hash()));
    contract::check(&search(&client), &query).expect("candidate contract");
}

#[test]
fn verified_identity_search_uses_bounded_identity_coordinates() {
    let client = SequenceClient::new(
        vec![IDENTITY_ENDPOINT.into()],
        vec![Ok(response(br#"{"data":[]}"#))],
    );
    let query = SubtitleCandidateSearchRequest::new(
        SubtitleCandidateSearchQuery::by_verified_identity(VerifiedMediaIdentity {
            title: "Synthetic Film".into(),
            year: Some(2020),
            season: None,
            episode: None,
        }),
    )
    .with_languages(vec![
        nen_domain::source::LanguageTag::parse("tr").expect("language")
    ]);
    assert!(search(&client).search(&query).expect("search").is_empty());
}

#[test]
fn canonical_identity_search_uses_imdb_without_title_or_filename_data() {
    let client = SequenceClient::new(
        vec![CANONICAL_ENDPOINT.into()],
        vec![Ok(response(br#"{"data":[]}"#))],
    );
    let query = SubtitleCandidateSearchRequest::new(
        SubtitleCandidateSearchQuery::by_canonical_identity(CanonicalMediaIdentity {
            imdb_id: Some("tt1375666".into()),
            parent_imdb_id: None,
        }),
    )
    .with_languages(vec![
        nen_domain::source::LanguageTag::parse("en").expect("language")
    ]);
    assert!(search(&client).search(&query).expect("search").is_empty());
}

#[test]
fn parsed_identity_search_carries_episode_coordinates() {
    let client = SequenceClient::new(
        vec![PARSED_ENDPOINT.into()],
        vec![Ok(response(br#"{"data":[]}"#))],
    );
    let query = SubtitleCandidateSearchRequest::new(
        SubtitleCandidateSearchQuery::by_parsed_identity(ParsedMediaIdentity {
            title: "Example Show".into(),
            year: None,
            season: Some(2),
            episode: Some(3),
        }),
    )
    .with_languages(vec![
        nen_domain::source::LanguageTag::parse("en").expect("language")
    ]);
    assert!(search(&client).search(&query).expect("search").is_empty());
}

#[test]
fn malformed_canonical_identity_is_rejected_before_http() {
    let client = SequenceClient::new(Vec::new(), Vec::new());
    let query = SubtitleCandidateSearchRequest::new(
        SubtitleCandidateSearchQuery::by_canonical_identity(CanonicalMediaIdentity {
            imdb_id: Some("private-id".into()),
            parent_imdb_id: None,
        }),
    );
    assert_eq!(
        search(&client).search(&query),
        Err(SubtitleCandidateSearchError::InvalidRequest)
    );
    assert!(client.requests().is_empty());
}

#[test]
fn malformed_and_oversized_responses_are_typed_errors() {
    let malformed = SequenceClient::new(vec![HASH_ENDPOINT.into()], vec![Ok(response(b"{"))]);
    let query = SubtitleCandidateSearchRequest::new(SubtitleCandidateSearchQuery::by_hash(hash()));
    assert_eq!(
        search(&malformed).search(&query),
        Err(SubtitleCandidateSearchError::InvalidResponse)
    );

    let oversized = SequenceClient::new(
        vec![HASH_ENDPOINT.into()],
        vec![Ok(response(&vec![b'x'; MAX_RESPONSE_BYTES + 1]))],
    );
    assert_eq!(
        search(&oversized).search(&query),
        Err(SubtitleCandidateSearchError::ResponseTooLarge)
    );
}

#[test]
fn response_is_capped_and_release_metadata_is_sanitized() {
    let mut body = String::from(r#"{"data":["#);
    for index in 0..20 {
        if index != 0 {
            body.push(',');
        }
        body.push_str(&format!(
            r#"{{"id":"candidate-{index}","attributes":{{"language":"en","release":"release / {index}","moviehash_match":true,"files":[{{"file_id":{}}}]}}}}"#,
            index + 1
        ));
    }
    body.push_str("]}");
    let client = SequenceClient::new(
        vec![HASH_ENDPOINT.into()],
        vec![Ok(response(body.as_bytes()))],
    );
    let query = SubtitleCandidateSearchRequest::new(SubtitleCandidateSearchQuery::by_hash(hash()));
    let candidates = search(&client).search(&query).expect("candidates");

    assert_eq!(candidates.len(), 16);
    assert_eq!(candidates[0].release_name.as_deref(), Some("release   0"));
    assert!(!candidates[0]
        .release_name
        .as_deref()
        .unwrap_or_default()
        .contains('/'));
}

#[test]
fn redirect_to_an_unapproved_provider_host_is_rejected() {
    let client = SequenceClient::new(
        vec![HASH_ENDPOINT.into()],
        vec![Ok(HttpResponse {
            status_code: 302,
            headers: vec![HttpHeader {
                name: "Location".into(),
                value: "https://evil.example/subtitles".into(),
            }],
            body: Vec::new(),
        })],
    );
    let query = SubtitleCandidateSearchRequest::new(SubtitleCandidateSearchQuery::by_hash(hash()));
    assert_eq!(
        search(&client).search(&query),
        Err(SubtitleCandidateSearchError::RedirectRejected)
    );
}
