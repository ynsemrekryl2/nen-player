use nen_domain::source::LanguageTag;
use nen_domain::subtitle::CueId;
use nen_ports::credentials::ApiKey;
use nen_ports::http::{
    FakeHttpClient, HttpClient, HttpError, HttpHeader, HttpMethod, HttpRequest, HttpResponse,
};
use nen_ports::translation::contract;
use nen_ports::translation::{
    AnalysisCharacter, AnalysisGlossaryEntry, DocumentAnalysis, DocumentAnalysisRequest,
    TranslationCall, TranslationCue, TranslationMode, TranslationProvider,
    TranslationProviderError, TranslationRequest,
};
use nen_providers::openai::{
    OpenAiTranslationProvider, RetrySleeper, MAX_PROVIDER_BODY_BYTES, OPENAI_RESPONSES_ENDPOINT,
    PROVIDER_TIMEOUT_MS,
};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const FIXTURE_KEY: &str = "fixture-api-key";

fn request() -> TranslationRequest {
    TranslationRequest {
        source_language: LanguageTag::parse("en").expect("source language"),
        target_language: LanguageTag::parse("tr").expect("target language"),
        context_cues: vec![
            TranslationCue {
                cue_id: CueId::new(10),
                text: "First fixture line".into(),
            },
            TranslationCue {
                cue_id: CueId::new(20),
                text: "Second fixture line".into(),
            },
        ],
        output_cue_ids: vec![CueId::new(10), CueId::new(20)],
        analysis: fixture_analysis(),
        block_index: 1,
        block_count: 1,
        mode: TranslationMode::Initial,
    }
}

fn fixture_analysis() -> DocumentAnalysis {
    DocumentAnalysis {
        summary: "Two speakers discuss a fixture.".into(),
        characters: vec![AnalysisCharacter {
            name: "PrivateName".into(),
            description: "A concise speaker".into(),
        }],
        glossary: vec![AnalysisGlossaryEntry {
            source: "fixture".into(),
            target: "örnek".into(),
            note: "Keep the technical meaning".into(),
        }],
    }
}

fn analysis_request() -> DocumentAnalysisRequest {
    DocumentAnalysisRequest {
        source_language: LanguageTag::parse("en").expect("source language"),
        target_language: LanguageTag::parse("tr").expect("target language"),
        transcript: request().context_cues,
        context_terms: vec!["PrivateName".into()],
    }
}

fn response(status_code: u16, body: impl Into<Vec<u8>>) -> HttpResponse {
    HttpResponse {
        status_code,
        headers: Vec::new(),
        body: body.into(),
    }
}

fn response_with_headers(
    status_code: u16,
    headers: Vec<HttpHeader>,
    body: impl Into<Vec<u8>>,
) -> HttpResponse {
    HttpResponse {
        status_code,
        headers,
        body: body.into(),
    }
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

    fn first_request(&self) -> HttpRequest {
        self.requests
            .lock()
            .expect("request lock")
            .first()
            .cloned()
            .expect("one request")
    }
}

impl HttpClient for SequenceClient {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        self.requests.lock().expect("request lock").push(request);
        self.responses
            .lock()
            .expect("response lock")
            .pop()
            .unwrap_or(Err(HttpError::Transport))
    }
}

#[derive(Default)]
struct RecordingSleeper {
    delays: Mutex<Vec<Duration>>,
    cancel_on_sleep: bool,
}

impl RecordingSleeper {
    fn cancelling() -> Self {
        Self {
            delays: Mutex::new(Vec::new()),
            cancel_on_sleep: true,
        }
    }

    fn delays(&self) -> Vec<Duration> {
        self.delays.lock().expect("delay lock").clone()
    }
}

impl RetrySleeper for RecordingSleeper {
    fn sleep(&self, duration: Duration, call: &TranslationCall) {
        self.delays.lock().expect("delay lock").push(duration);
        if self.cancel_on_sleep {
            call.cancel();
        }
    }
}

fn provider<'a>(
    client: &'a SequenceClient,
    sleeper: Arc<RecordingSleeper>,
) -> OpenAiTranslationProvider<'a> {
    OpenAiTranslationProvider::with_endpoint_and_retry_sleeper(
        client,
        ApiKey::new(FIXTURE_KEY).expect("fixture key"),
        "gpt-5.6-luna",
        OPENAI_RESPONSES_ENDPOINT,
        sleeper,
    )
    .expect("provider")
}

fn translate_fixture(
    client: &SequenceClient,
    sleeper: Arc<RecordingSleeper>,
) -> Result<nen_ports::translation::TranslationResponse, TranslationProviderError> {
    provider(client, sleeper).translate(&request(), &TranslationCall::without_progress())
}

#[test]
fn openai_passes_the_shared_contract_kit() {
    let capture_client = SequenceClient::new(vec![Ok(response(
        200,
        include_str!("../../../../fixtures/providers/openai/response-success.json"),
    ))]);
    OpenAiTranslationProvider::new(
        &capture_client,
        ApiKey::new(FIXTURE_KEY).expect("fixture key"),
        "gpt-5.6-luna",
    )
    .expect("provider")
    .translate(&request(), &TranslationCall::without_progress())
    .expect("capture request");
    let expected_request = capture_client.first_request();
    let client = FakeHttpClient::new(vec![(
        expected_request,
        Ok(response(
            200,
            include_str!("../../../../fixtures/providers/openai/response-success.json"),
        )),
    )]);
    let provider = OpenAiTranslationProvider::new(
        &client,
        ApiKey::new(FIXTURE_KEY).expect("fixture key"),
        "gpt-5.6-luna",
    )
    .expect("provider");
    contract::check(&provider, &request()).expect("contract passes");
}

#[test]
fn request_matches_golden_and_uses_the_approved_transport_shape() {
    let client = SequenceClient::new(vec![Ok(response(
        200,
        include_str!("../../../../fixtures/providers/openai/response-success.json"),
    ))]);
    let sleeper = Arc::new(RecordingSleeper::default());
    provider(&client, sleeper)
        .translate(&request(), &TranslationCall::without_progress())
        .expect("translation");

    let sent = client.first_request();
    assert!(sent.method == HttpMethod::Post, "provider uses POST");
    assert!(
        sent.url == OPENAI_RESPONSES_ENDPOINT,
        "provider uses approved host"
    );
    assert!(
        sent.timeout_ms == Some(PROVIDER_TIMEOUT_MS),
        "provider timeout is bounded"
    );
    assert!(
        sent.max_body_bytes == MAX_PROVIDER_BODY_BYTES,
        "body limit is bounded"
    );
    assert!(
        sent.headers
            .iter()
            .any(|header| header.name.eq_ignore_ascii_case("content-type")
                && header.value == "application/json"),
        "JSON content type is present"
    );
    assert!(
        sent.headers
            .iter()
            .any(|header| header.name.eq_ignore_ascii_case("authorization")
                && header.value.starts_with("Bearer ")),
        "authorization header is present"
    );

    let actual: Value =
        serde_json::from_slice(sent.body.as_ref().expect("request body")).expect("request JSON");
    let expected: Value = serde_json::from_str(include_str!(
        "../../../../fixtures/providers/openai/request-success.golden"
    ))
    .expect("golden JSON");
    assert!(actual == expected, "request JSON differs from golden");
}

#[test]
fn analysis_request_matches_golden_and_maps_the_validated_result() {
    let client = SequenceClient::new(vec![Ok(response(
        200,
        include_str!("../../../../fixtures/providers/openai/response-analysis-success.json"),
    ))]);
    let actual_analysis = provider(&client, Arc::new(RecordingSleeper::default()))
        .analyze_document(&analysis_request(), &TranslationCall::without_progress())
        .expect("analysis");
    assert_eq!(actual_analysis, fixture_analysis());

    let sent = client.first_request();
    let actual: Value =
        serde_json::from_slice(sent.body.as_ref().expect("request body")).expect("request JSON");
    let expected: Value = serde_json::from_str(include_str!(
        "../../../../fixtures/providers/openai/request-analysis.golden"
    ))
    .expect("golden JSON");
    assert_eq!(actual, expected, "analysis request differs from golden");

    let input: Value = serde_json::from_str(actual["input"].as_str().expect("input JSON"))
        .expect("embedded input JSON");
    for forbidden in [
        "title",
        "path",
        "basename",
        "url",
        "mediaHash",
        "startTime",
        "endTime",
    ] {
        assert!(
            input.get(forbidden).is_none(),
            "forbidden field {forbidden}"
        );
    }
}

#[test]
fn targeted_and_full_retry_requests_match_their_dynamic_schema_goldens() {
    let mut targeted = request();
    targeted.output_cue_ids = vec![CueId::new(20)];
    targeted.mode = TranslationMode::TargetedRepair;
    let mut full = request();
    full.mode = TranslationMode::FullRetry;

    for (request, golden) in [
        (
            targeted,
            include_str!("../../../../fixtures/providers/openai/request-targeted-repair.golden"),
        ),
        (
            full,
            include_str!("../../../../fixtures/providers/openai/request-full-retry.golden"),
        ),
    ] {
        let client = SequenceClient::new(vec![Ok(response(
            200,
            include_str!("../../../../fixtures/providers/openai/response-success.json"),
        ))]);
        provider(&client, Arc::new(RecordingSleeper::default()))
            .translate(&request, &TranslationCall::without_progress())
            .expect("translation request");
        let actual: Value =
            serde_json::from_slice(client.first_request().body.as_ref().expect("request body"))
                .expect("request JSON");
        let expected: Value = serde_json::from_str(golden).expect("golden JSON");
        assert_eq!(actual, expected, "mode request differs from golden");
    }
}

#[test]
fn invalid_empty_refused_or_oversized_analysis_is_a_permanent_local_failure() {
    for body in [
        r#"{"output_text":"{\"summary\":\"   \",\"characters\":[],\"glossary\":[]}"}"#,
        r#"{"output_text":"{\"summary\":\"valid\",\"characters\":[],\"glossary\":[],\"extra\":true}"}"#,
        include_str!("../../../../fixtures/providers/openai/response-refusal.json"),
    ] {
        let client = SequenceClient::new(vec![Ok(response(200, body))]);
        assert_eq!(
            provider(&client, Arc::new(RecordingSleeper::default()))
                .analyze_document(&analysis_request(), &TranslationCall::without_progress()),
            Err(TranslationProviderError::Permanent)
        );
        assert_eq!(client.request_count(), 1);
    }

    let oversized_response = SequenceClient::new(vec![Ok(response(
        200,
        vec![b'x'; MAX_PROVIDER_BODY_BYTES + 1],
    ))]);
    assert_eq!(
        provider(&oversized_response, Arc::new(RecordingSleeper::default()))
            .analyze_document(&analysis_request(), &TranslationCall::without_progress()),
        Err(TranslationProviderError::Permanent)
    );
    assert_eq!(oversized_response.request_count(), 1);

    let mut oversized = analysis_request();
    oversized.transcript[0].text = "x".repeat(MAX_PROVIDER_BODY_BYTES);
    let client = SequenceClient::new(Vec::new());
    assert_eq!(
        provider(&client, Arc::new(RecordingSleeper::default()))
            .analyze_document(&oversized, &TranslationCall::without_progress()),
        Err(TranslationProviderError::Permanent)
    );
    assert_eq!(client.request_count(), 0, "oversized analysis is not sent");
}

#[test]
fn success_response_is_untrusted_json_mapped_to_shared_cues() {
    let client = SequenceClient::new(vec![Ok(response(
        200,
        include_str!("../../../../fixtures/providers/openai/response-success.json"),
    ))]);
    let result =
        translate_fixture(&client, Arc::new(RecordingSleeper::default())).expect("translation");
    assert!(result.cues.len() == 2, "two cues are mapped");
    assert!(
        result.cues[0].cue_id == CueId::new(10),
        "first cue id is mapped"
    );
    assert!(
        result.cues[1].cue_id == CueId::new(20),
        "second cue id is mapped"
    );
    assert!(
        result.cues[0].text == "İlk fixture satırı",
        "first cue text is mapped"
    );
}

#[test]
fn missing_and_duplicate_cues_are_passed_to_the_local_validator() {
    let missing = SequenceClient::new(vec![Ok(response(
        200,
        include_str!("../../../../fixtures/providers/openai/response-missing-cue.json"),
    ))]);
    let missing_result = translate_fixture(&missing, Arc::new(RecordingSleeper::default()))
        .expect("provider parses missing cue response");
    assert!(
        missing_result.cues.len() == 1,
        "missing cue is not repaired remotely"
    );

    let duplicate = SequenceClient::new(vec![Ok(response(
        200,
        include_str!("../../../../fixtures/providers/openai/response-duplicate-id.json"),
    ))]);
    let duplicate_result = translate_fixture(&duplicate, Arc::new(RecordingSleeper::default()))
        .expect("provider parses duplicate cue response");
    assert!(
        duplicate_result.cues.len() == 2,
        "duplicate cue is not repaired remotely"
    );
    assert!(
        duplicate_result
            .cues
            .iter()
            .all(|cue| cue.cue_id == CueId::new(10)),
        "duplicate identity remains visible to local validation"
    );
}

#[test]
fn malformed_json_and_refusal_are_permanent_failures() {
    for fixture in [
        include_str!("../../../../fixtures/providers/openai/response-malformed.json"),
        include_str!("../../../../fixtures/providers/openai/response-refusal.json"),
    ] {
        let client = SequenceClient::new(vec![Ok(response(200, fixture))]);
        let result = translate_fixture(&client, Arc::new(RecordingSleeper::default()));
        assert!(
            result == Err(TranslationProviderError::Permanent),
            "invalid provider output is permanent"
        );
        assert!(client.request_count() == 1, "invalid output is not retried");
    }
}

#[test]
fn credential_client_and_unknown_http_failures_are_permanent() {
    for status_code in [400, 401, 403, 404] {
        let client = SequenceClient::new(vec![Ok(response(status_code, b"{}"))]);
        let result = translate_fixture(&client, Arc::new(RecordingSleeper::default()));
        assert!(
            result == Err(TranslationProviderError::Permanent),
            "non-retryable status is permanent"
        );
        assert!(
            client.request_count() == 1,
            "non-retryable status is not retried"
        );
    }
}

#[test]
fn rate_limit_and_server_failures_retry_with_bounded_backoff() {
    let client = SequenceClient::new(vec![
        Ok(response(
            200,
            include_str!("../../../../fixtures/providers/openai/response-success.json"),
        )),
        Ok(response(
            500,
            include_str!("../../../../fixtures/providers/openai/response-500.json"),
        )),
        Ok(response(
            429,
            include_str!("../../../../fixtures/providers/openai/response-429.json"),
        )),
    ]);
    let sleeper = Arc::new(RecordingSleeper::default());
    provider(&client, sleeper.clone())
        .translate(&request(), &TranslationCall::without_progress())
        .expect("retry succeeds");
    assert!(
        client.request_count() == 3,
        "two transient failures are retried"
    );
    assert!(
        sleeper.delays() == vec![Duration::from_millis(500), Duration::from_secs(2)],
        "fallback delays are bounded"
    );
}

#[test]
fn retry_after_is_clamped_and_invalid_values_use_fallback() {
    let client = SequenceClient::new(vec![
        Ok(response(
            200,
            include_str!("../../../../fixtures/providers/openai/response-success.json"),
        )),
        Ok(response_with_headers(
            500,
            vec![HttpHeader {
                name: "Retry-After".into(),
                value: "not-a-duration".into(),
            }],
            include_str!("../../../../fixtures/providers/openai/response-500.json"),
        )),
        Ok(response_with_headers(
            429,
            vec![HttpHeader {
                name: "Retry-After".into(),
                value: "20".into(),
            }],
            include_str!("../../../../fixtures/providers/openai/response-429.json"),
        )),
    ]);
    let sleeper = Arc::new(RecordingSleeper::default());
    provider(&client, sleeper.clone())
        .translate(&request(), &TranslationCall::without_progress())
        .expect("retry succeeds");
    assert!(
        sleeper.delays() == vec![Duration::from_secs(10), Duration::from_secs(2)],
        "retry-after is bounded and invalid retry-after falls back"
    );
}

#[test]
fn transport_failures_retry_and_exhaustion_is_transient() {
    let success = || {
        Ok(response(
            200,
            include_str!("../../../../fixtures/providers/openai/response-success.json"),
        ))
    };
    let client = SequenceClient::new(vec![success(), Err(HttpError::Transport)]);
    let sleeper = Arc::new(RecordingSleeper::default());
    provider(&client, sleeper.clone())
        .translate(&request(), &TranslationCall::without_progress())
        .expect("transport retry succeeds");
    assert!(client.request_count() == 2, "transport failure is retried");
    assert!(
        sleeper.delays() == vec![Duration::from_millis(500)],
        "transport fallback is bounded"
    );

    let exhausted = SequenceClient::new(vec![
        Err(HttpError::Transport),
        Err(HttpError::Transport),
        Err(HttpError::Transport),
    ]);
    let result = provider(&exhausted, Arc::new(RecordingSleeper::default()))
        .translate(&request(), &TranslationCall::without_progress());
    assert!(
        result == Err(TranslationProviderError::Transient),
        "retries are capped"
    );
    assert!(
        exhausted.request_count() == 3,
        "retry count is capped at two retries"
    );
}

#[test]
fn cancellation_during_backoff_prevents_the_next_send() {
    let client = SequenceClient::new(vec![Ok(response(429, b"{}"))]);
    let sleeper = Arc::new(RecordingSleeper::cancelling());
    let result = provider(&client, sleeper.clone())
        .translate(&request(), &TranslationCall::without_progress());
    assert!(
        result == Err(TranslationProviderError::Cancelled),
        "cancelled retry stops"
    );
    assert!(
        client.request_count() == 1,
        "cancellation prevents the next send"
    );
    assert!(
        sleeper.delays() == vec![Duration::from_millis(500)],
        "backoff was entered once"
    );
}

#[test]
fn request_and_response_limits_fail_without_leaking_payloads() {
    let oversized_response = SequenceClient::new(vec![Ok(response(
        200,
        vec![b'x'; MAX_PROVIDER_BODY_BYTES + 1],
    ))]);
    let result = provider(&oversized_response, Arc::new(RecordingSleeper::default()))
        .translate(&request(), &TranslationCall::without_progress());
    assert!(
        result == Err(TranslationProviderError::Permanent),
        "oversized response is permanent"
    );
    assert!(
        oversized_response.request_count() == 1,
        "oversized response is not retried"
    );

    let oversized_request = TranslationRequest {
        context_cues: vec![TranslationCue {
            cue_id: CueId::new(10),
            text: "x".repeat(MAX_PROVIDER_BODY_BYTES),
        }],
        ..request()
    };
    let client = SequenceClient::new(Vec::new());
    let result = provider(&client, Arc::new(RecordingSleeper::default()))
        .translate(&oversized_request, &TranslationCall::without_progress());
    assert!(
        result == Err(TranslationProviderError::Permanent),
        "oversized request is permanent"
    );
    assert!(
        client.request_count() == 0,
        "oversized request is rejected before send"
    );
}

#[test]
fn unapproved_endpoint_is_rejected_before_send() {
    let client = SequenceClient::new(Vec::new());
    let provider = OpenAiTranslationProvider::with_endpoint_and_retry_sleeper(
        &client,
        ApiKey::new(FIXTURE_KEY).expect("fixture key"),
        "gpt-5.6-luna",
        "https://evil.example/v1/responses",
        Arc::new(RecordingSleeper::default()),
    )
    .expect("test provider");
    let result = provider.translate(&request(), &TranslationCall::without_progress());
    assert!(
        result == Err(TranslationProviderError::Permanent),
        "unapproved endpoint is permanent"
    );
    assert!(
        client.request_count() == 0,
        "unapproved endpoint is never sent"
    );
}

#[test]
fn sensitive_provider_surfaces_are_shape_only() {
    let secret = "PRIVATE FIXTURE SECRET";
    let dialogue = "PRIVATE FIXTURE DIALOGUE";
    let raw_response = format!(r#"{{"output_text":"{dialogue}"}}"#);
    let request = TranslationRequest {
        context_cues: vec![TranslationCue {
            cue_id: CueId::new(10),
            text: dialogue.into(),
        }],
        analysis: DocumentAnalysis {
            summary: secret.into(),
            ..fixture_analysis()
        },
        ..request()
    };
    let provider_client = SequenceClient::new(Vec::new());
    let provider = provider(&provider_client, Arc::new(RecordingSleeper::default()));
    let http_request = HttpRequest::post_json(
        "https://private.example/response?token=PRIVATE_FIXTURE_TOKEN",
        vec![HttpHeader {
            name: "Authorization".into(),
            value: secret.into(),
        }],
        raw_response.as_bytes().to_vec(),
        MAX_PROVIDER_BODY_BYTES,
        PROVIDER_TIMEOUT_MS,
    );
    let http_response = response(200, raw_response.as_bytes());
    let api_key = ApiKey::new(secret).expect("key");

    for output in [
        format!("{provider:?}"),
        format!("{request:?}"),
        format!("{http_request:?}"),
        format!("{http_response:?}"),
        format!("{api_key:?}"),
    ] {
        assert!(!output.contains(secret), "secret does not reach Debug");
        assert!(!output.contains(dialogue), "dialogue does not reach Debug");
        assert!(
            !output.contains("PRIVATE_FIXTURE_TOKEN"),
            "token does not reach Debug"
        );
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    struct LeakyTwin {
        api_key: String,
        dialogue: String,
        raw_response: String,
    }
    let leaky = format!(
        "{:?}",
        LeakyTwin {
            api_key: secret.into(),
            dialogue: dialogue.into(),
            raw_response,
        }
    );
    assert!(
        leaky.contains(secret),
        "negative guard twin exposes sentinel"
    );
}
