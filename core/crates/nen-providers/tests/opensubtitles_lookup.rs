use nen_ports::http::{
    FakeHttpClient, HttpClient, HttpError, HttpHeader, HttpRequest, HttpResponse,
};
use nen_ports::identity::{IdentityLookup, MediaHash, MediaIdentityLookup, VerifiedMediaIdentity};
use nen_providers::opensubtitles::{OpenSubtitlesApiKey, OpenSubtitlesIdentityLookup};
use std::sync::Mutex;

const ENDPOINT: &str = "https://api.opensubtitles.com/api/v1/subtitles?moviehash=0001020304050607&moviehash_match=only";
const USER_AGENT: &str = "Nen Player/0.1";
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;

fn hash() -> MediaHash {
    MediaHash::from_bytes([0, 1, 2, 3, 4, 5, 6, 7])
}

fn response(body: &str) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: Vec::new(),
        body: body.as_bytes().to_vec(),
    }
}

fn lookup(body: &str) -> Result<IdentityLookup, nen_ports::identity::IdentityLookupError> {
    let request = HttpRequest::get(
        ENDPOINT,
        vec![
            HttpHeader {
                name: "Api-Key".into(),
                value: "fixture-key".into(),
            },
            HttpHeader {
                name: "User-Agent".into(),
                value: USER_AGENT.into(),
            },
            HttpHeader {
                name: "Accept".into(),
                value: "application/json".into(),
            },
        ],
        MAX_RESPONSE_BYTES,
    );
    let client = FakeHttpClient::new(vec![(request, Ok(response(body)))]);
    let key = OpenSubtitlesApiKey::new("fixture-key").expect("valid key");
    OpenSubtitlesIdentityLookup::new(&client, key).lookup_by_hash(hash())
}

#[test]
fn recorded_movie_fixture_becomes_a_match() {
    let result = lookup(include_str!(
        "../../../../fixtures/providers/opensubtitles/exact-movie.json"
    ))
    .expect("lookup");
    assert_eq!(
        result,
        IdentityLookup::Match(VerifiedMediaIdentity {
            title: "Synthetic Film".into(),
            year: Some(2020),
            season: None,
            episode: None,
        })
    );
}

#[test]
fn recorded_episode_fixture_preserves_episode_coordinates() {
    let result = lookup(include_str!(
        "../../../../fixtures/providers/opensubtitles/exact-episode.json"
    ))
    .expect("lookup");
    assert_eq!(
        result,
        IdentityLookup::Match(VerifiedMediaIdentity {
            title: "Synthetic Show".into(),
            year: Some(2021),
            season: Some(2),
            episode: Some(7),
        })
    );
}

#[test]
fn recorded_no_match_and_ambiguous_fixtures_are_typed() {
    assert_eq!(
        lookup(include_str!(
            "../../../../fixtures/providers/opensubtitles/no-match.json"
        ))
        .expect("lookup"),
        IdentityLookup::NoMatch
    );
    assert_eq!(
        lookup(include_str!(
            "../../../../fixtures/providers/opensubtitles/ambiguous.json"
        ))
        .expect("lookup"),
        IdentityLookup::Ambiguous
    );
}

#[test]
fn provider_payloads_and_credentials_never_reach_debug_output() {
    let key = OpenSubtitlesApiKey::new("fixture-secret").expect("valid key");
    let request = HttpRequest::get(
        "https://api.opensubtitles.com/api/v1/subtitles?moviehash=deadbeefdeadbeef",
        vec![HttpHeader {
            name: "Api-Key".into(),
            value: "fixture-secret".into(),
        }],
        MAX_RESPONSE_BYTES,
    );
    let response = response(
        r#"{"data":[{"attributes":{"moviehash_match":true,"feature_details":{"title":"Private Filename.mkv","year":2020},"files":[{"file_id":987654,"file_name":"Private Filename.mkv"}]}}]}"#,
    );
    assert!(!format!("{key:?}").contains("fixture-secret"));
    assert!(!format!("{request:?}").contains("deadbeefdeadbeef"));
    assert!(!format!("{request:?}").contains("fixture-secret"));
    assert!(!format!("{response:?}").contains("Private Filename.mkv"));
    assert!(!format!("{response:?}").contains("987654"));
}

struct SequenceClient {
    responses: Mutex<Vec<Result<HttpResponse, HttpError>>>,
    requests: Mutex<Vec<HttpRequest>>,
}

impl SequenceClient {
    fn new(responses: Vec<Result<HttpResponse, HttpError>>) -> Self {
        Self {
            responses: Mutex::new(responses),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn request_count(&self) -> usize {
        self.requests.lock().expect("request lock").len()
    }
}

impl HttpClient for SequenceClient {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        self.requests.lock().expect("request lock").push(request);
        self.responses.lock().expect("response lock").remove(0)
    }
}

fn sequence_lookup(
    client: &SequenceClient,
) -> Result<IdentityLookup, nen_ports::identity::IdentityLookupError> {
    let key = OpenSubtitlesApiKey::new("fixture-key").expect("valid key");
    OpenSubtitlesIdentityLookup::new(client, key).lookup_by_hash(hash())
}

fn status_response(status_code: u16, headers: Vec<HttpHeader>, body: Vec<u8>) -> HttpResponse {
    HttpResponse {
        status_code,
        headers,
        body,
    }
}

#[test]
fn malformed_json_is_a_payload_free_typed_error() {
    let client = SequenceClient::new(vec![Ok(status_response(200, Vec::new(), b"{".to_vec()))]);
    assert_eq!(
        sequence_lookup(&client),
        Err(nen_ports::identity::IdentityLookupError::InvalidResponse)
    );
}

#[test]
fn oversized_json_is_rejected_before_parsing() {
    let client = SequenceClient::new(vec![Ok(status_response(
        200,
        Vec::new(),
        vec![b'x'; MAX_RESPONSE_BYTES + 1],
    ))]);
    assert_eq!(
        sequence_lookup(&client),
        Err(nen_ports::identity::IdentityLookupError::ResponseTooLarge)
    );
}

#[test]
fn non_success_and_transport_failures_are_typed() {
    let transport = SequenceClient::new(vec![Err(HttpError::Transport)]);
    assert_eq!(
        sequence_lookup(&transport),
        Err(nen_ports::identity::IdentityLookupError::Transport)
    );

    let status = SequenceClient::new(vec![Ok(status_response(429, Vec::new(), Vec::new()))]);
    assert_eq!(
        sequence_lookup(&status),
        Err(nen_ports::identity::IdentityLookupError::HttpStatus)
    );
}

#[test]
fn only_https_approved_hosts_and_bounded_redirects_are_followed() {
    let redirected = SequenceClient::new(vec![
        Ok(status_response(
            302,
            vec![HttpHeader {
                name: "Location".into(),
                value: "https://vip-api.opensubtitles.com/api/v1/subtitles?moviehash=0001020304050607&moviehash_match=only".into(),
            }],
            Vec::new(),
        )),
        Ok(response(r#"{"data":[]}"#)),
    ]);
    assert_eq!(sequence_lookup(&redirected), Ok(IdentityLookup::NoMatch));
    assert_eq!(redirected.request_count(), 2);

    let insecure = SequenceClient::new(vec![Ok(status_response(
        302,
        vec![HttpHeader {
            name: "Location".into(),
            value: "http://api.opensubtitles.com/api/v1/subtitles".into(),
        }],
        Vec::new(),
    ))]);
    assert_eq!(
        sequence_lookup(&insecure),
        Err(nen_ports::identity::IdentityLookupError::RedirectRejected)
    );

    let foreign_host = SequenceClient::new(vec![Ok(status_response(
        302,
        vec![HttpHeader {
            name: "Location".into(),
            value: "https://evil.example/api/v1/subtitles".into(),
        }],
        Vec::new(),
    ))]);
    assert_eq!(
        sequence_lookup(&foreign_host),
        Err(nen_ports::identity::IdentityLookupError::RedirectRejected)
    );
}

#[test]
fn redirect_limit_is_enforced() {
    let responses = (0..6)
        .map(|_| {
            Ok(status_response(
                302,
                vec![HttpHeader {
                    name: "Location".into(),
                    value: ENDPOINT.into(),
                }],
                Vec::new(),
            ))
        })
        .collect();
    let client = SequenceClient::new(responses);
    assert_eq!(
        sequence_lookup(&client),
        Err(nen_ports::identity::IdentityLookupError::RedirectRejected)
    );
    assert_eq!(client.request_count(), 6);
}
