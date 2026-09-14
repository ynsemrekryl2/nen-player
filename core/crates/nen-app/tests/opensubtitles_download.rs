use nen_app::ports::credentials::{
    ApiKey, CredentialKind, InMemoryCredentialStore, SecureCredentialStore,
};
use nen_app::ports::http::{HttpClient, HttpError, HttpHeader, HttpRequest, HttpResponse};
use nen_app::ports::subtitle_candidates::SubtitleCandidate;
use nen_app::subtitles::{DownloadRefusal, SubtitleLibrary};
use nen_domain::source::LanguageTag;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const DOWNLOAD_ENDPOINT: &str = "https://api.opensubtitles.com/api/v1/download";
const LINK: &str = "https://dl.opensubtitles.com/subtitles/fixture";

fn candidate() -> SubtitleCandidate {
    SubtitleCandidate {
        public_id: "public-candidate".into(),
        private_file_id: 7001,
        language: LanguageTag::parse("en").expect("language"),
        release_name: Some("Fixture release".into()),
        hearing_impaired: false,
        ai_translated: false,
    }
}

fn metadata() -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![HttpHeader {
            name: "Content-Type".into(),
            value: "application/json".into(),
        }],
        body: include_bytes!("../../../../fixtures/providers/opensubtitles/download-link.json")
            .to_vec(),
    }
}

fn text(body: &[u8]) -> HttpResponse {
    HttpResponse {
        status_code: 200,
        headers: vec![HttpHeader {
            name: "Content-Type".into(),
            value: "text/plain; charset=utf-8".into(),
        }],
        body: body.to_vec(),
    }
}

fn valid_srt() -> &'static [u8] {
    b"1\n00:00:00,000 --> 00:00:01,000\nFixture subtitle\n"
}

struct RecordingClient {
    responses: Mutex<Vec<Result<HttpResponse, HttpError>>>,
    requests: Mutex<Vec<HttpRequest>>,
}

impl RecordingClient {
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

impl HttpClient for RecordingClient {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        self.requests.lock().expect("request lock").push(request);
        self.responses.lock().expect("response lock").remove(0)
    }
}

struct RevisionChangingClient {
    responses: Mutex<Vec<Result<HttpResponse, HttpError>>>,
    calls: AtomicUsize,
    revision: Arc<AtomicU64>,
}

impl HttpClient for RevisionChangingClient {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        assert!(request.url == DOWNLOAD_ENDPOINT || request.url == LINK);
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        let response = self.responses.lock().expect("response lock").remove(0);
        if call == 1 {
            self.revision.store(2, Ordering::Release);
        }
        response
    }
}

fn credentials() -> InMemoryCredentialStore {
    let store = InMemoryCredentialStore::new();
    store
        .set(
            CredentialKind::OpenSubtitles,
            ApiKey::new("fixture-key").expect("key"),
        )
        .expect("credential store");
    store
}

fn library_with_candidate() -> (SubtitleLibrary, u32) {
    let mut library = SubtitleLibrary::new();
    let token = library.add_opensubtitles(vec![candidate()])[0];
    (library, token)
}

#[test]
fn selected_candidate_attaches_a_document_and_second_selection_is_idempotent() {
    let (mut library, token) = library_with_candidate();
    let store = credentials();
    let client = RecordingClient::new(vec![Ok(metadata()), Ok(text(valid_srt()))]);

    library
        .download_opensubtitles(token, &store, &client)
        .expect("selected subtitle");
    assert!(library.document_of(token).is_some());
    assert!(library.is_token_usable(token));

    library
        .download_opensubtitles(token, &store, &client)
        .expect("idempotent selection");
    assert_eq!(client.request_count(), 2);
}

#[test]
fn missing_credential_is_typed_and_does_not_send_or_attach() {
    let (mut library, token) = library_with_candidate();
    let store = InMemoryCredentialStore::new();
    let client = RecordingClient::new(Vec::new());

    assert_eq!(
        library.download_opensubtitles(token, &store, &client),
        Err(DownloadRefusal::MissingCredential)
    );
    assert_eq!(client.request_count(), 0);
    assert!(library.document_of(token).is_none());
}

#[test]
fn malformed_srt_never_mutates_the_catalogue() {
    let (mut library, token) = library_with_candidate();
    let store = credentials();
    let client = RecordingClient::new(vec![Ok(metadata()), Ok(text(b"not an srt"))]);
    assert_eq!(
        library.download_opensubtitles(token, &store, &client),
        Err(DownloadRefusal::MalformedSubtitle)
    );
    assert_eq!(client.request_count(), 2);
    assert!(library.document_of(token).is_none());
    assert!(library.is_token_usable(token));
}

#[test]
fn invalid_encoding_never_mutates_the_catalogue() {
    let (mut library, token) = library_with_candidate();
    let store = credentials();
    let client = RecordingClient::new(vec![Ok(metadata()), Ok(text(&[0xff, 0xfe, 0xfd]))]);
    assert_eq!(
        library.download_opensubtitles(token, &store, &client),
        Err(DownloadRefusal::InvalidEncoding)
    );
    assert_eq!(client.request_count(), 2);
    assert!(library.document_of(token).is_none());
    assert!(library.is_token_usable(token));
}

#[test]
fn a_changed_media_revision_discards_a_late_document() {
    let (mut library, token) = library_with_candidate();
    let store = credentials();
    let revision = Arc::new(AtomicU64::new(1));
    let client = RevisionChangingClient {
        responses: Mutex::new(vec![Ok(metadata()), Ok(text(valid_srt()))]),
        calls: AtomicUsize::new(0),
        revision: Arc::clone(&revision),
    };

    assert_eq!(
        library.download_opensubtitles_if_current(token, 1, &revision, &store, &client,),
        Err(DownloadRefusal::Cancelled)
    );
    assert_eq!(client.calls.load(Ordering::SeqCst), 2);
    assert!(library.document_of(token).is_none());
}

#[test]
fn stale_or_non_provider_selection_is_rejected_without_network() {
    let (mut library, _token) = library_with_candidate();
    let store = credentials();
    let client = RecordingClient::new(Vec::new());
    assert_eq!(
        library.download_opensubtitles(99, &store, &client),
        Err(DownloadRefusal::NotOpenSubtitles)
    );
    assert_eq!(client.request_count(), 0);
}

#[test]
fn refusal_debug_is_payload_free() {
    let output = format!(
        "{:?} {:?} {:?}",
        DownloadRefusal::Provider(
            nen_app::ports::subtitle_download::SubtitleDownloadError::InvalidResponse
        ),
        DownloadRefusal::InvalidEncoding,
        DownloadRefusal::MalformedSubtitle
    );
    assert!(!output.contains("https://dl.opensubtitles.com"));
    assert!(!output.contains("fixture-key"));
    assert!(!output.contains("7001"));
}
