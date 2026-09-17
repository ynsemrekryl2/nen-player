use nen_ports::http::{HttpClient, HttpError, HttpHeader, HttpMethod, HttpRequest, HttpResponse};
use nen_ports::subtitle_download::{
    SubtitleDownloadError, SubtitleDownloadRequest, SubtitleDownloader, MAX_DOWNLOAD_BYTES,
    MAX_DOWNLOAD_METADATA_BYTES, MAX_DOWNLOAD_REDIRECTS,
};
use nen_providers::opensubtitles::{OpenSubtitlesApiKey, OpenSubtitlesDownloader};
use std::sync::Mutex;

const DOWNLOAD_ENDPOINT: &str = "https://api.opensubtitles.com/api/v1/download";
const LINK: &str = "https://dl.opensubtitles.com/subtitles/fixture";
const WWW_LINK: &str = "https://www.opensubtitles.com/download/fixture/subfile/fixture.srt";
const KEY: &str = "fixture-key";

fn response(status_code: u16, headers: Vec<HttpHeader>, body: Vec<u8>) -> HttpResponse {
    HttpResponse {
        status_code,
        headers,
        body,
    }
}

fn text_response(body: &[u8]) -> HttpResponse {
    response(
        200,
        vec![HttpHeader {
            name: "Content-Type".into(),
            value: "text/plain; charset=utf-8".into(),
        }],
        body.to_vec(),
    )
}

fn srt() -> Vec<u8> {
    b"1\n00:00:00,000 --> 00:00:01,000\nFixture subtitle\n".to_vec()
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

fn downloader<'a>(client: &'a dyn HttpClient) -> OpenSubtitlesDownloader<'a> {
    OpenSubtitlesDownloader::new(
        client,
        OpenSubtitlesApiKey::new(KEY).expect("valid fixture key"),
    )
}

fn metadata(link: &str) -> HttpResponse {
    response(
        200,
        vec![HttpHeader {
            name: "Content-Type".into(),
            value: "application/json".into(),
        }],
        format!(r#"{{"link":"{link}"}}"#).into_bytes(),
    )
}

fn request_is_metadata(request: &HttpRequest) {
    assert_eq!(request.method, HttpMethod::Post);
    assert_eq!(request.url, DOWNLOAD_ENDPOINT);
    assert_eq!(request.max_body_bytes, MAX_DOWNLOAD_METADATA_BYTES);
    assert_eq!(request.timeout_ms, Some(15_000));
    assert_eq!(
        request.body.as_deref(),
        Some(br#"{"file_id":7001}"#.as_slice())
    );
    assert!(request
        .headers
        .iter()
        .any(|header| header.name == "Api-Key" && header.value == KEY));
    assert!(request
        .headers
        .iter()
        .any(|header| header.name == "Content-Type" && header.value == "application/json"));
}

#[test]
fn selection_exchanges_a_private_id_then_gets_bounded_plaintext_without_reusing_key() {
    let client = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into(), LINK.into()],
        vec![
            Ok(response(
                200,
                vec![HttpHeader {
                    name: "Content-Type".into(),
                    value: "application/json".into(),
                }],
                include_bytes!("../../../../fixtures/providers/opensubtitles/download-link.json")
                    .to_vec(),
            )),
            Ok(text_response(&srt())),
        ],
    );
    let downloaded = downloader(&client)
        .download(SubtitleDownloadRequest::new(7001))
        .expect("download");

    assert_eq!(downloaded.bytes(), srt().as_slice());
    let requests = client.requests();
    assert_eq!(requests.len(), 2);
    request_is_metadata(&requests[0]);
    assert_eq!(requests[1].method, HttpMethod::Get);
    assert_eq!(requests[1].max_body_bytes, MAX_DOWNLOAD_BYTES);
    assert!(requests[1]
        .headers
        .iter()
        .all(|header| header.name != "Api-Key"));
}

#[test]
fn final_quota_download_uses_the_valid_link_before_the_remaining_count_reaches_zero() {
    let client = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into(), LINK.into()],
        vec![
            Ok(response(
                200,
                vec![HttpHeader {
                    name: "Content-Type".into(),
                    value: "application/json".into(),
                }],
                include_bytes!(
                    "../../../../fixtures/providers/opensubtitles/download-link-final-quota.json"
                )
                .to_vec(),
            )),
            Ok(text_response(&srt())),
        ],
    );

    let downloaded = downloader(&client)
        .download(SubtitleDownloadRequest::new(7001))
        .expect("the final allowed download");

    assert_eq!(downloaded.bytes(), srt().as_slice());
    assert_eq!(client.requests().len(), 2);
}

#[test]
fn official_www_download_link_is_accepted_without_reusing_key() {
    let client = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into(), WWW_LINK.into()],
        vec![Ok(metadata(WWW_LINK)), Ok(text_response(&srt()))],
    );

    let downloaded = downloader(&client)
        .download(SubtitleDownloadRequest::new(7001))
        .expect("official www download");

    assert_eq!(downloaded.bytes(), srt().as_slice());
    let requests = client.requests();
    assert_eq!(requests.len(), 2);
    assert!(requests[1]
        .headers
        .iter()
        .all(|header| header.name != "Api-Key"));
}

#[test]
fn redirect_to_official_www_download_link_is_followed() {
    let client = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into(), LINK.into(), WWW_LINK.into()],
        vec![
            Ok(metadata(LINK)),
            Ok(response(
                302,
                vec![HttpHeader {
                    name: "Location".into(),
                    value: WWW_LINK.into(),
                }],
                Vec::new(),
            )),
            Ok(text_response(&srt())),
        ],
    );

    let downloaded = downloader(&client)
        .download(SubtitleDownloadRequest::new(7001))
        .expect("official redirect");

    assert_eq!(downloaded.bytes(), srt().as_slice());
    assert_eq!(client.requests().len(), 3);
}

#[test]
fn an_unapproved_link_host_never_receives_a_get() {
    let client = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into()],
        vec![Ok(metadata("https://evil.example/subtitles/fixture"))],
    );
    assert_eq!(
        downloader(&client).download(SubtitleDownloadRequest::new(7001)),
        Err(SubtitleDownloadError::RedirectRejected)
    );
    assert_eq!(client.requests().len(), 1);
}

#[test]
fn an_http_link_is_rejected_before_get() {
    let client = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into()],
        vec![Ok(metadata(
            "http://dl.opensubtitles.com/subtitles/fixture",
        ))],
    );
    assert_eq!(
        downloader(&client).download(SubtitleDownloadRequest::new(7001)),
        Err(SubtitleDownloadError::RedirectRejected)
    );
    assert_eq!(client.requests().len(), 1);
}

#[test]
fn www_host_is_limited_to_download_paths() {
    let client = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into()],
        vec![Ok(metadata(
            "https://www.opensubtitles.com/subtitles/fixture.srt",
        ))],
    );
    assert_eq!(
        downloader(&client).download(SubtitleDownloadRequest::new(7001)),
        Err(SubtitleDownloadError::RedirectRejected)
    );
    assert_eq!(client.requests().len(), 1);
}

#[test]
fn a_www_lookalike_host_is_rejected_before_get() {
    let client = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into()],
        vec![Ok(metadata(
            "https://www.opensubtitles.com.evil.example/download/fixture.srt",
        ))],
    );
    assert_eq!(
        downloader(&client).download(SubtitleDownloadRequest::new(7001)),
        Err(SubtitleDownloadError::RedirectRejected)
    );
    assert_eq!(client.requests().len(), 1);
}

#[test]
fn an_http_www_link_is_rejected_before_get() {
    let client = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into()],
        vec![Ok(metadata(
            "http://www.opensubtitles.com/download/fixture.srt",
        ))],
    );
    assert_eq!(
        downloader(&client).download(SubtitleDownloadRequest::new(7001)),
        Err(SubtitleDownloadError::RedirectRejected)
    );
    assert_eq!(client.requests().len(), 1);
}

#[test]
fn every_redirect_hop_stays_on_an_approved_host() {
    let client = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into(), LINK.into()],
        vec![
            Ok(metadata(LINK)),
            Ok(response(
                302,
                vec![HttpHeader {
                    name: "Location".into(),
                    value: "https://evil.example/subtitles/fixture".into(),
                }],
                Vec::new(),
            )),
        ],
    );
    assert_eq!(
        downloader(&client).download(SubtitleDownloadRequest::new(7001)),
        Err(SubtitleDownloadError::RedirectRejected)
    );
    assert_eq!(client.requests().len(), 2);
}

#[test]
fn redirect_chain_is_capped_at_five_hops() {
    let mut expected = vec![DOWNLOAD_ENDPOINT.to_owned(), LINK.to_owned()];
    let mut responses = vec![Ok(metadata(LINK))];
    for index in 0..=MAX_DOWNLOAD_REDIRECTS {
        let next = format!("https://dl.opensubtitles.com/subtitles/r{index}");
        responses.push(Ok(response(
            302,
            vec![HttpHeader {
                name: "Location".into(),
                value: next.clone(),
            }],
            Vec::new(),
        )));
        expected.push(next);
    }
    let client = SequenceClient::new(expected, responses);

    assert_eq!(
        downloader(&client).download(SubtitleDownloadRequest::new(7001)),
        Err(SubtitleDownloadError::RedirectRejected)
    );
    assert_eq!(
        client.requests().len(),
        usize::from(MAX_DOWNLOAD_REDIRECTS) + 2
    );
}

#[test]
fn content_length_budget_is_enforced_before_accepting_the_body() {
    let oversized_length = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into(), LINK.into()],
        vec![
            Ok(metadata(LINK)),
            Ok(response(
                200,
                vec![HttpHeader {
                    name: "Content-Length".into(),
                    value: (MAX_DOWNLOAD_BYTES as u64 + 1).to_string(),
                }],
                srt(),
            )),
        ],
    );
    assert_eq!(
        downloader(&oversized_length).download(SubtitleDownloadRequest::new(7001)),
        Err(SubtitleDownloadError::ContentTooLarge)
    );
}

#[test]
fn actual_body_budget_is_enforced() {
    let oversized_body = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into(), LINK.into()],
        vec![
            Ok(metadata(LINK)),
            Ok(text_response(&vec![b'x'; MAX_DOWNLOAD_BYTES + 1])),
        ],
    );
    assert_eq!(
        downloader(&oversized_body).download(SubtitleDownloadRequest::new(7001)),
        Err(SubtitleDownloadError::ContentTooLarge)
    );
}

#[test]
fn unexpected_content_type_is_rejected_before_app_attach() {
    let wrong_type = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into(), LINK.into()],
        vec![
            Ok(metadata(LINK)),
            Ok(response(
                200,
                vec![HttpHeader {
                    name: "Content-Type".into(),
                    value: "application/zip".into(),
                }],
                srt(),
            )),
        ],
    );
    assert_eq!(
        downloader(&wrong_type).download(SubtitleDownloadRequest::new(7001)),
        Err(SubtitleDownloadError::UnexpectedContentType)
    );
}

#[test]
fn archive_magic_is_rejected_before_app_attach() {
    let archive = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into(), LINK.into()],
        vec![
            Ok(metadata(LINK)),
            Ok(text_response(b"PK\x03\x04not subtitle")),
        ],
    );
    assert_eq!(
        downloader(&archive).download(SubtitleDownloadRequest::new(7001)),
        Err(SubtitleDownloadError::ArchiveRejected)
    );
}

#[test]
fn quota_without_a_link_is_typed_and_does_not_make_a_second_request() {
    let client = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into()],
        vec![Ok(response(
            200,
            Vec::new(),
            include_bytes!("../../../../fixtures/providers/opensubtitles/quota.json").to_vec(),
        ))],
    );
    assert_eq!(
        downloader(&client).download(SubtitleDownloadRequest::new(7001)),
        Err(SubtitleDownloadError::QuotaExhausted)
    );
    assert_eq!(client.requests().len(), 1);
}

#[test]
fn provider_failures_and_private_debug_values_are_payload_free() {
    let client = SequenceClient::new(
        vec![DOWNLOAD_ENDPOINT.into()],
        vec![Err(HttpError::Transport)],
    );
    assert_eq!(
        downloader(&client).download(SubtitleDownloadRequest::new(7001)),
        Err(SubtitleDownloadError::Transport)
    );
    let debug = format!(
        "{:?} {:?}",
        SubtitleDownloadRequest::new(884_422),
        SubtitleDownloadError::InvalidResponse
    );
    assert!(!debug.contains("884422"));
    assert!(!debug.contains(KEY));
}
