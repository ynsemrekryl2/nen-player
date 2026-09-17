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
use nen_providers::openrouter::{
    Clock, OpenRouterPreflightError, OpenRouterTranslationProvider, RetrySleeper,
    MAX_PROVIDER_BODY_BYTES, OPENROUTER_CHAT_ENDPOINT, OPENROUTER_MODEL_ENDPOINT_PREFIX,
    PROVIDER_TIMEOUT_MS,
};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const FIXTURE_KEY: &str = "fixture-openrouter-key";
const MODEL: &str = "openai/gpt-5.6-luna";

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

    fn method_count(&self, method: HttpMethod) -> usize {
        self.requests
            .lock()
            .expect("request lock")
            .iter()
            .filter(|request| request.method == method)
            .count()
    }

    fn requests(&self) -> Vec<HttpRequest> {
        self.requests.lock().expect("request lock").clone()
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
}

impl RecordingSleeper {
    fn delays(&self) -> Vec<Duration> {
        self.delays.lock().expect("delay lock").clone()
    }
}

impl RetrySleeper for RecordingSleeper {
    fn sleep(&self, duration: Duration, _call: &TranslationCall) {
        self.delays.lock().expect("delay lock").push(duration);
    }
}

struct TestClock {
    now: Mutex<u64>,
}

impl TestClock {
    fn new(now: u64) -> Self {
        Self {
            now: Mutex::new(now),
        }
    }

    fn advance(&self, seconds: u64) {
        *self.now.lock().expect("clock lock") += seconds;
    }
}

impl Clock for TestClock {
    fn now_seconds(&self) -> u64 {
        *self.now.lock().expect("clock lock")
    }
}

fn provider<'a>(
    client: &'a SequenceClient,
    sleeper: Arc<RecordingSleeper>,
    clock: Arc<TestClock>,
) -> OpenRouterTranslationProvider<'a> {
    OpenRouterTranslationProvider::with_endpoint_and_retry_sleeper(
        client,
        ApiKey::new(FIXTURE_KEY).expect("fixture key"),
        MODEL,
        OPENROUTER_CHAT_ENDPOINT,
        &format!("{OPENROUTER_MODEL_ENDPOINT_PREFIX}{MODEL}"),
        sleeper,
        clock,
    )
    .expect("provider")
}

fn supported() -> Result<HttpResponse, HttpError> {
    Ok(response(
        200,
        include_str!("../../../../fixtures/providers/openrouter/capability-supported.json"),
    ))
}

fn chat_success() -> Result<HttpResponse, HttpError> {
    Ok(response(
        200,
        include_str!("../../../../fixtures/providers/openrouter/response-success.json"),
    ))
}

#[test]
fn openrouter_passes_the_shared_contract_kit_with_preflight() {
    let capture = SequenceClient::new(vec![chat_success(), supported()]);
    let capture_provider = provider(
        &capture,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    );
    capture_provider
        .translate(&request(), &TranslationCall::without_progress())
        .expect("capture translation");
    let captured = capture.requests();
    assert!(
        captured.len() == 2,
        "preflight and translation were captured"
    );

    let client = FakeHttpClient::new(vec![
        (
            captured[0].clone(),
            Ok(supported().expect("supported response")),
        ),
        (
            captured[1].clone(),
            Ok(chat_success().expect("success response")),
        ),
    ]);
    let provider = OpenRouterTranslationProvider::new(
        &client,
        ApiKey::new(FIXTURE_KEY).expect("fixture key"),
        MODEL,
    )
    .expect("provider");
    contract::check(&provider, &request()).expect("contract passes");
}

#[test]
fn preflight_cache_keeps_positive_and_negative_results_for_fifteen_minutes() {
    let positive_client = SequenceClient::new(vec![supported()]);
    let positive_clock = Arc::new(TestClock::new(100));
    let positive = provider(
        &positive_client,
        Arc::new(RecordingSleeper::default()),
        positive_clock.clone(),
    );
    assert!(
        positive
            .preflight(&TranslationCall::without_progress())
            .expect("positive capability")
            .supports_structured_outputs(),
        "structured output is supported"
    );
    assert!(
        positive
            .preflight(&TranslationCall::without_progress())
            .expect("cached positive capability")
            .supports_structured_outputs(),
        "positive capability is cached"
    );
    assert!(
        positive_client.request_count() == 1,
        "positive cache avoids a second GET"
    );

    let negative_client = SequenceClient::new(vec![Ok(response(
        200,
        include_str!("../../../../fixtures/providers/openrouter/capability-unsupported.json"),
    ))]);
    let negative = provider(
        &negative_client,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    );
    assert!(
        negative.preflight(&TranslationCall::without_progress())
            == Err(OpenRouterPreflightError::CapabilityMissing),
        "unsupported capability is rejected"
    );
    assert!(
        negative.preflight(&TranslationCall::without_progress())
            == Err(OpenRouterPreflightError::CapabilityMissing),
        "negative capability is cached"
    );
    assert!(
        negative_client.request_count() == 1,
        "negative cache avoids a second GET"
    );
}

#[test]
fn capability_cache_expires_at_fifteen_minutes() {
    let client = SequenceClient::new(vec![supported(), supported()]);
    let clock = Arc::new(TestClock::new(100));
    let provider = provider(
        &client,
        Arc::new(RecordingSleeper::default()),
        clock.clone(),
    );
    provider
        .preflight(&TranslationCall::without_progress())
        .expect("initial capability");
    clock.advance(900);
    provider
        .preflight(&TranslationCall::without_progress())
        .expect("refreshed capability");
    assert!(
        client.request_count() == 2,
        "expired capability is refreshed"
    );
}

#[test]
fn unsupported_capability_never_sends_a_translation_post() {
    let client = SequenceClient::new(vec![Ok(response(
        200,
        include_str!("../../../../fixtures/providers/openrouter/capability-unsupported.json"),
    ))]);
    let provider = provider(
        &client,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    );
    let first = provider.translate(&request(), &TranslationCall::without_progress());
    let second = provider.translate(&request(), &TranslationCall::without_progress());
    assert!(
        first == Err(TranslationProviderError::Permanent)
            && second == Err(TranslationProviderError::Permanent),
        "unsupported capability refuses the job"
    );
    assert!(
        client.method_count(HttpMethod::Get) == 1,
        "negative capability is fetched once"
    );
    assert!(
        client.method_count(HttpMethod::Post) == 0,
        "unsupported model has no translation POST"
    );
}

#[test]
fn preflight_transport_and_parse_errors_do_not_get_cached() {
    let client = SequenceClient::new(vec![
        supported(),
        Err(HttpError::Transport),
        Err(HttpError::Transport),
        Err(HttpError::Transport),
    ]);
    let sleeper = Arc::new(RecordingSleeper::default());
    let transport_provider = provider(&client, sleeper, Arc::new(TestClock::new(100)));
    assert!(
        transport_provider.preflight(&TranslationCall::without_progress())
            == Err(OpenRouterPreflightError::Transient),
        "transport exhaustion is transient"
    );
    assert!(
        transport_provider
            .preflight(&TranslationCall::without_progress())
            .expect("second preflight succeeds")
            .supports_structured_outputs(),
        "transient failure is not cached"
    );
    assert!(
        client.request_count() == 4,
        "three retries plus the uncached second GET"
    );

    let malformed_client = SequenceClient::new(vec![
        supported(),
        Ok(response(
            200,
            include_str!("../../../../fixtures/providers/openrouter/capability-malformed.json"),
        )),
    ]);
    let malformed_provider = provider(
        &malformed_client,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    );
    assert!(
        malformed_provider.preflight(&TranslationCall::without_progress())
            == Err(OpenRouterPreflightError::Permanent),
        "malformed capability response is permanent"
    );
    assert!(
        malformed_provider
            .preflight(&TranslationCall::without_progress())
            .expect("malformed response is not cached")
            .supports_structured_outputs(),
        "second capability response is consulted"
    );
    assert!(
        malformed_client.request_count() == 2,
        "parse failure is not cached"
    );
}

#[test]
fn preflight_bad_gateway_is_transient_and_never_starts_the_job() {
    let client = SequenceClient::new(vec![
        Ok(response(
            502,
            include_str!("../../../../fixtures/providers/openrouter/response-502.json"),
        )),
        Ok(response(
            502,
            include_str!("../../../../fixtures/providers/openrouter/response-502.json"),
        )),
        Ok(response(
            502,
            include_str!("../../../../fixtures/providers/openrouter/response-502.json"),
        )),
    ]);
    let provider = provider(
        &client,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    );
    assert!(
        provider.preflight(&TranslationCall::without_progress())
            == Err(OpenRouterPreflightError::Transient),
        "preflight 502 exhaustion is transient"
    );
    assert!(
        client.method_count(HttpMethod::Get) == 3,
        "preflight retry budget is bounded"
    );
    assert!(
        client.method_count(HttpMethod::Post) == 0,
        "failed preflight has no translation POST"
    );
}

#[test]
fn chat_request_matches_golden_and_declares_openai_routing() {
    let client = SequenceClient::new(vec![chat_success(), supported()]);
    let provider = provider(
        &client,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    );
    provider
        .translate(&request(), &TranslationCall::without_progress())
        .expect("translation");
    let requests = client.requests();
    assert!(requests[0].method == HttpMethod::Get, "preflight uses GET");
    assert!(
        requests[0].url == format!("{OPENROUTER_MODEL_ENDPOINT_PREFIX}{MODEL}"),
        "model endpoint is exact"
    );
    assert!(
        requests[0].timeout_ms == Some(PROVIDER_TIMEOUT_MS),
        "preflight timeout is bounded"
    );
    let sent = &requests[1];
    assert!(sent.method == HttpMethod::Post, "translation uses POST");
    assert!(
        sent.url == OPENROUTER_CHAT_ENDPOINT,
        "translation endpoint is exact"
    );
    assert!(
        sent.timeout_ms == Some(PROVIDER_TIMEOUT_MS),
        "translation timeout is bounded"
    );
    assert!(
        sent.headers
            .iter()
            .any(|header| header.name == "HTTP-Referer")
            && sent.headers.iter().any(|header| header.name == "X-Title"),
        "identifier headers are present"
    );
    let actual: Value =
        serde_json::from_slice(sent.body.as_ref().expect("request body")).expect("request JSON");
    let expected: Value = serde_json::from_str(include_str!(
        "../../../../fixtures/providers/openrouter/request-success.golden"
    ))
    .expect("golden JSON");
    assert!(actual == expected, "OpenRouter request differs from golden");
}

#[test]
fn analysis_request_matches_golden_and_maps_the_validated_result() {
    let client = SequenceClient::new(vec![
        Ok(response(
            200,
            include_str!(
                "../../../../fixtures/providers/openrouter/response-analysis-success.json"
            ),
        )),
        supported(),
    ]);
    let provider = provider(
        &client,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    );
    let actual_analysis = provider
        .analyze_document(&analysis_request(), &TranslationCall::without_progress())
        .expect("analysis");
    assert_eq!(actual_analysis, fixture_analysis());

    let requests = client.requests();
    assert_eq!(requests[0].method, HttpMethod::Get);
    let actual: Value =
        serde_json::from_slice(requests[1].body.as_ref().expect("analysis request body"))
            .expect("request JSON");
    let expected: Value = serde_json::from_str(include_str!(
        "../../../../fixtures/providers/openrouter/request-analysis.golden"
    ))
    .expect("golden JSON");
    assert_eq!(actual, expected, "analysis request differs from golden");
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
            include_str!(
                "../../../../fixtures/providers/openrouter/request-targeted-repair.golden"
            ),
        ),
        (
            full,
            include_str!("../../../../fixtures/providers/openrouter/request-full-retry.golden"),
        ),
    ] {
        let client = SequenceClient::new(vec![chat_success(), supported()]);
        provider(
            &client,
            Arc::new(RecordingSleeper::default()),
            Arc::new(TestClock::new(100)),
        )
        .translate(&request, &TranslationCall::without_progress())
        .expect("translation request");
        let requests = client.requests();
        let actual: Value =
            serde_json::from_slice(requests[1].body.as_ref().expect("translation request body"))
                .expect("request JSON");
        let expected: Value = serde_json::from_str(golden).expect("golden JSON");
        assert_eq!(actual, expected, "mode request differs from golden");
    }
}

#[test]
fn invalid_empty_refused_or_oversized_analysis_never_sends_a_block_post() {
    for body in [
        r#"{"choices":[{"message":{"content":"{\"summary\":\"   \",\"characters\":[],\"glossary\":[]}","refusal":null}}]}"#,
        r#"{"choices":[{"message":{"content":"{\"summary\":\"valid\",\"characters\":[],\"glossary\":[],\"extra\":true}","refusal":null}}]}"#,
        r#"{"choices":[{"message":{"content":"{}","refusal":"fixture refusal"}}]}"#,
    ] {
        let client = SequenceClient::new(vec![Ok(response(200, body)), supported()]);
        let provider = provider(
            &client,
            Arc::new(RecordingSleeper::default()),
            Arc::new(TestClock::new(100)),
        );
        assert_eq!(
            provider.analyze_document(&analysis_request(), &TranslationCall::without_progress()),
            Err(TranslationProviderError::Permanent)
        );
        assert_eq!(client.method_count(HttpMethod::Post), 1);
    }

    let oversized_response = SequenceClient::new(vec![
        Ok(response(200, vec![b'x'; MAX_PROVIDER_BODY_BYTES + 1])),
        supported(),
    ]);
    let oversized_provider = provider(
        &oversized_response,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    );
    assert_eq!(
        oversized_provider
            .analyze_document(&analysis_request(), &TranslationCall::without_progress()),
        Err(TranslationProviderError::Permanent)
    );
    assert_eq!(oversized_response.method_count(HttpMethod::Post), 1);

    let mut oversized = analysis_request();
    oversized.transcript[0].text = "x".repeat(MAX_PROVIDER_BODY_BYTES);
    let client = SequenceClient::new(vec![supported()]);
    let provider = provider(
        &client,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    );
    assert_eq!(
        provider.analyze_document(&oversized, &TranslationCall::without_progress()),
        Err(TranslationProviderError::Permanent)
    );
    assert_eq!(client.method_count(HttpMethod::Post), 0);
}

#[test]
fn success_response_uses_shared_parser_and_identity() {
    let client = SequenceClient::new(vec![chat_success(), supported()]);
    let provider = provider(
        &client,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    );
    let identity = provider.identity();
    assert!(
        identity.provider() == "openrouter",
        "provider identity is OpenRouter"
    );
    assert!(
        identity.model() == MODEL,
        "technical model identity is preserved"
    );
    let result = provider
        .translate(&request(), &TranslationCall::without_progress())
        .expect("translation");
    assert!(result.cues.len() == 2, "two cues are mapped");
    assert!(
        result.cues[0].cue_id == CueId::new(10),
        "first cue id is mapped"
    );
    assert!(
        result.cues[0].text == "İlk OpenRouter satırı",
        "first cue text is mapped"
    );
}

#[test]
fn transient_chat_failures_retry_with_the_shared_bounded_budget() {
    let client = SequenceClient::new(vec![
        chat_success(),
        Ok(response(
            502,
            include_str!("../../../../fixtures/providers/openrouter/response-502.json"),
        )),
        Ok(response(
            429,
            include_str!("../../../../fixtures/providers/openrouter/response-429.json"),
        )),
        supported(),
    ]);
    let sleeper = Arc::new(RecordingSleeper::default());
    provider(&client, sleeper.clone(), Arc::new(TestClock::new(100)))
        .translate(&request(), &TranslationCall::without_progress())
        .expect("retry succeeds");
    assert!(
        client.method_count(HttpMethod::Get) == 1,
        "preflight is not retried after caching"
    );
    assert!(
        client.method_count(HttpMethod::Post) == 3,
        "two transient chat failures are retried"
    );
    assert!(
        sleeper.delays() == vec![Duration::from_millis(500), Duration::from_secs(2)],
        "shared fallback delays are used"
    );
}

#[test]
fn cancellation_before_preflight_prevents_every_send() {
    let client = SequenceClient::new(Vec::new());
    let provider = provider(
        &client,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    );
    let call = TranslationCall::without_progress();
    call.cancel();
    assert!(
        provider.translate(&request(), &call) == Err(TranslationProviderError::Cancelled),
        "cancelled call is refused"
    );
    assert!(
        client.request_count() == 0,
        "cancelled call does not preflight"
    );
}

#[test]
fn unapproved_chat_endpoint_is_rejected_before_send() {
    let client = SequenceClient::new(Vec::new());
    let provider = OpenRouterTranslationProvider::with_endpoint_and_retry_sleeper(
        &client,
        ApiKey::new(FIXTURE_KEY).expect("fixture key"),
        MODEL,
        "https://evil.example/api/v1/chat/completions",
        &format!("{OPENROUTER_MODEL_ENDPOINT_PREFIX}{MODEL}"),
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    )
    .expect("provider");
    assert!(
        provider.translate(&request(), &TranslationCall::without_progress())
            == Err(TranslationProviderError::Permanent),
        "unapproved endpoint is permanent"
    );
    assert!(
        client.request_count() == 0,
        "unapproved endpoint has no send"
    );
}

#[test]
fn sensitive_openrouter_surfaces_are_shape_only() {
    let secret = "PRIVATE OPENROUTER SECRET";
    let dialogue = "PRIVATE OPENROUTER DIALOGUE";
    let raw_response = format!(r#"{{"choices":[{{"message":{{"content":"{dialogue}"}}}}]}}"#);
    let client = SequenceClient::new(vec![supported()]);
    let provider = provider(
        &client,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    );
    let capabilities = provider
        .preflight(&TranslationCall::without_progress())
        .expect("capabilities");
    let input = TranslationRequest {
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
    let http_request = HttpRequest::post_json(
        "https://private.example/chat?token=PRIVATE_OPENROUTER_TOKEN",
        vec![HttpHeader {
            name: "Authorization".into(),
            value: secret.into(),
        }],
        raw_response.as_bytes().to_vec(),
        MAX_PROVIDER_BODY_BYTES,
        PROVIDER_TIMEOUT_MS,
    );
    let http_response = response(200, raw_response.as_bytes());
    for output in [
        format!("{provider:?}"),
        format!("{input:?}"),
        format!("{capabilities:?}"),
        format!("{http_request:?}"),
        format!("{http_response:?}"),
        format!("{:?}", OpenRouterPreflightError::Permanent),
    ] {
        assert!(!output.contains(secret), "secret does not reach Debug");
        assert!(!output.contains(dialogue), "dialogue does not reach Debug");
        assert!(
            !output.contains("PRIVATE_OPENROUTER_TOKEN"),
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

#[test]
fn retry_after_is_clamped_on_chat_failures() {
    let client = SequenceClient::new(vec![
        chat_success(),
        Ok(response_with_headers(
            429,
            vec![HttpHeader {
                name: "Retry-After".into(),
                value: "20".into(),
            }],
            include_str!("../../../../fixtures/providers/openrouter/response-429.json"),
        )),
        supported(),
    ]);
    let sleeper = Arc::new(RecordingSleeper::default());
    provider(&client, sleeper.clone(), Arc::new(TestClock::new(100)))
        .translate(&request(), &TranslationCall::without_progress())
        .expect("retry succeeds");
    assert!(
        sleeper.delays() == vec![Duration::from_secs(10)],
        "retry-after is clamped"
    );
}

#[test]
fn translate_and_analyze_report_token_usage_including_billed_cost() {
    let translate_client = SequenceClient::new(vec![chat_success(), supported()]);
    let translate_call = TranslationCall::without_progress();
    provider(
        &translate_client,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    )
    .translate(&request(), &translate_call)
    .expect("translation");
    assert_eq!(
        translate_call.total_usage(),
        nen_ports::translation::TokenUsage {
            input_tokens: 150,
            cached_input_tokens: 30,
            output_tokens: 45,
            cost_usd: Some(0.0021),
        },
        "OpenRouter reports its own billed cost"
    );

    let analysis_client = SequenceClient::new(vec![
        Ok(response(
            200,
            include_str!(
                "../../../../fixtures/providers/openrouter/response-analysis-success.json"
            ),
        )),
        supported(),
    ]);
    let analysis_call = TranslationCall::without_progress();
    provider(
        &analysis_client,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    )
    .analyze_document(&analysis_request(), &analysis_call)
    .expect("analysis");
    assert_eq!(
        analysis_call.total_usage(),
        nen_ports::translation::TokenUsage {
            input_tokens: 300,
            cached_input_tokens: 0,
            output_tokens: 80,
            cost_usd: Some(0.0038),
        }
    );
}

#[test]
fn request_asks_openrouter_to_include_billed_usage() {
    let client = SequenceClient::new(vec![chat_success(), supported()]);
    provider(
        &client,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    )
    .translate(&request(), &TranslationCall::without_progress())
    .expect("translation");
    let requests = client.requests();
    let sent: Value =
        serde_json::from_slice(requests[1].body.as_ref().expect("body")).expect("json");
    assert_eq!(sent["usage"]["include"].as_bool(), Some(true));
}

#[test]
fn missing_usage_defaults_to_zero_without_failing_the_call() {
    let client = SequenceClient::new(vec![
        Ok(response(
            200,
            r#"{"choices":[{"message":{"content":"{\"translations\":[{\"cueId\":10,\"text\":\"x\"},{\"cueId\":20,\"text\":\"y\"}]}","refusal":null}}]}"#,
        )),
        supported(),
    ]);
    let call = TranslationCall::without_progress();
    let result = provider(
        &client,
        Arc::new(RecordingSleeper::default()),
        Arc::new(TestClock::new(100)),
    )
    .translate(&request(), &call);
    assert!(
        result.is_ok(),
        "a response with no usage field still succeeds"
    );
    assert_eq!(
        call.total_usage(),
        nen_ports::translation::TokenUsage::default(),
        "no usage object means every counter stays at zero, not an error"
    );
}
