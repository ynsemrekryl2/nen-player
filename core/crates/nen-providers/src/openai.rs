//! Direct OpenAI Responses API translation provider (NEN-116, ADR-0019).
//!
//! The adapter owns one call's short-lived credential and turns the provider's
//! untrusted response into the shared translation DTO. It never validates cue
//! completeness or order; `nen-translate` remains the authoritative local
//! validator. Request/response payloads stay inside the HTTP boundary and no
//! provider-specific payload is carried by an error or a `Debug` surface.

use nen_ports::credentials::ApiKey;
use nen_ports::http::{HttpClient, HttpError, HttpHeader, HttpRequest, HttpResponse};
use nen_ports::translation::{
    TranslatedCue, TranslationCall, TranslationProgress, TranslationProgressPhase,
    TranslationProvider, TranslationProviderError, TranslationProviderIdentity, TranslationRequest,
    TranslationResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

pub const OPENAI_RESPONSES_ENDPOINT: &str = "https://api.openai.com/v1/responses";
pub const MAX_PROVIDER_BODY_BYTES: usize = 1024 * 1024;
pub const PROVIDER_TIMEOUT_MS: u64 = 60_000;
pub const PROMPT_VERSION: u32 = 1;
pub const SCHEMA_VERSION: u32 = 1;

const PROVIDER_ID: &str = "openai";
const RESPONSE_SCHEMA_NAME: &str = "subtitle_translation";
const MAX_RETRIES: usize = 2;
const SYSTEM_INSTRUCTIONS: &str = "Translate subtitle cues. Use context_cues and context_terms only as context. Translate only output_cue_ids. Preserve meaning, register, names, terminology, and meaningful line breaks. Return only the requested JSON object with no explanation or extra fields.";

/// Provides the bounded wait between transient attempts.
///
/// Production uses [`ThreadRetrySleeper`]. The injected form makes the retry
/// contract deterministic in tests and lets a test close the same
/// [`TranslationCall`] while a wait is in progress.
pub trait RetrySleeper: Send + Sync {
    fn sleep(&self, duration: Duration, call: &TranslationCall);
}

struct ThreadRetrySleeper;

impl RetrySleeper for ThreadRetrySleeper {
    fn sleep(&self, duration: Duration, _call: &TranslationCall) {
        std::thread::sleep(duration);
    }
}

/// One direct OpenAI provider instance for a translation job.
pub struct OpenAiTranslationProvider<'a> {
    http: &'a dyn HttpClient,
    api_key: ApiKey,
    identity: TranslationProviderIdentity,
    endpoint: String,
    sleeper: Arc<dyn RetrySleeper>,
}

impl<'a> OpenAiTranslationProvider<'a> {
    pub fn new(
        http: &'a dyn HttpClient,
        api_key: ApiKey,
        model: &str,
    ) -> Result<Self, TranslationProviderError> {
        Self::with_endpoint_and_retry_sleeper(
            http,
            api_key,
            model,
            OPENAI_RESPONSES_ENDPOINT,
            Arc::new(ThreadRetrySleeper),
        )
    }

    /// Builds a provider with an explicit endpoint and sleeper for deterministic
    /// tests. `translate` still enforces the approved endpoint before `send`.
    pub fn with_endpoint_and_retry_sleeper(
        http: &'a dyn HttpClient,
        api_key: ApiKey,
        model: &str,
        endpoint: &str,
        sleeper: Arc<dyn RetrySleeper>,
    ) -> Result<Self, TranslationProviderError> {
        if model.trim().is_empty() || model.chars().any(char::is_control) {
            return Err(TranslationProviderError::Permanent);
        }
        let identity = TranslationProviderIdentity::new(PROVIDER_ID, model)?;
        Ok(Self {
            http,
            api_key,
            identity,
            endpoint: endpoint.to_owned(),
            sleeper,
        })
    }
}

impl fmt::Debug for OpenAiTranslationProvider<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenAiTranslationProvider")
            .field("identity", &self.identity)
            .field("has_credential", &true)
            .field("endpoint", &"<redacted>")
            .finish()
    }
}

impl TranslationProvider for OpenAiTranslationProvider<'_> {
    fn identity(&self) -> TranslationProviderIdentity {
        self.identity.clone()
    }

    fn translate(
        &self,
        request: &TranslationRequest,
        call: &TranslationCall,
    ) -> Result<TranslationResponse, TranslationProviderError> {
        if self.endpoint != OPENAI_RESPONSES_ENDPOINT {
            return Err(TranslationProviderError::Permanent);
        }

        let total = u32::try_from(request.output_cue_ids.len())
            .map_err(|_| TranslationProviderError::Permanent)?;
        if total == 0 {
            return Err(TranslationProviderError::Permanent);
        }
        let body = build_request_body(&self.identity, request)?;
        if body.len() > MAX_PROVIDER_BODY_BYTES {
            return Err(TranslationProviderError::Permanent);
        }

        call.checkpoint()?;
        call.progress(TranslationProgress {
            phase: TranslationProgressPhase::Preparing,
            done: 0,
            total,
        })?;

        let mut retries = 0usize;
        loop {
            call.checkpoint()?;
            let request = HttpRequest::post_json(
                &self.endpoint,
                vec![HttpHeader {
                    name: "Authorization".to_owned(),
                    value: format!("Bearer {}", self.api_key.expose()),
                }],
                body.clone(),
                MAX_PROVIDER_BODY_BYTES,
                PROVIDER_TIMEOUT_MS,
            );

            match self.http.send(request) {
                Err(HttpError::ResponseTooLarge) => {
                    return Err(TranslationProviderError::Permanent);
                }
                Err(HttpError::Transport) => {
                    if retries == MAX_RETRIES {
                        return Err(TranslationProviderError::Transient);
                    }
                    self.wait_for_retry(call, fallback_retry_delay(retries))?;
                    retries += 1;
                }
                Ok(response) if is_retryable_status(response.status_code) => {
                    if retries == MAX_RETRIES {
                        return Err(TranslationProviderError::Transient);
                    }
                    let delay = retry_after_or_fallback(&response, retries);
                    self.wait_for_retry(call, delay)?;
                    retries += 1;
                }
                Ok(response) if (200..300).contains(&response.status_code) => {
                    call.checkpoint()?;
                    let translated = parse_response_body(&response.body)?;
                    call.progress(TranslationProgress {
                        phase: TranslationProgressPhase::Translating,
                        done: total,
                        total,
                    })?;
                    call.progress(TranslationProgress {
                        phase: TranslationProgressPhase::Finalizing,
                        done: total,
                        total,
                    })?;
                    return call.finish(translated);
                }
                Ok(_) => return Err(TranslationProviderError::Permanent),
            }
        }
    }
}

impl OpenAiTranslationProvider<'_> {
    fn wait_for_retry(
        &self,
        call: &TranslationCall,
        delay: Duration,
    ) -> Result<(), TranslationProviderError> {
        call.checkpoint()?;
        self.sleeper.sleep(delay, call);
        call.checkpoint()
    }
}

#[derive(Serialize)]
struct ResponsesRequest<'a> {
    model: &'a str,
    store: bool,
    instructions: &'static str,
    input: String,
    text: ResponsesText,
}

#[derive(Serialize)]
struct ResponsesText {
    format: ResponsesFormat,
}

#[derive(Serialize)]
struct ResponsesFormat {
    #[serde(rename = "type")]
    format_type: &'static str,
    name: &'static str,
    strict: bool,
    schema: Value,
}

#[derive(Serialize)]
struct PromptInput {
    prompt_version: u32,
    schema_version: u32,
    source_language: String,
    target_language: String,
    context_terms: Vec<String>,
    context_cues: Vec<PromptCue>,
    output_cue_ids: Vec<u32>,
}

#[derive(Serialize)]
struct PromptCue {
    cue_id: u32,
    text: String,
}

fn build_request_body(
    identity: &TranslationProviderIdentity,
    request: &TranslationRequest,
) -> Result<Vec<u8>, TranslationProviderError> {
    let input = PromptInput {
        prompt_version: PROMPT_VERSION,
        schema_version: SCHEMA_VERSION,
        source_language: request.source_language.as_str().to_owned(),
        target_language: request.target_language.as_str().to_owned(),
        context_terms: request.context_terms.clone(),
        context_cues: request
            .context_cues
            .iter()
            .map(|cue| PromptCue {
                cue_id: cue.cue_id.get(),
                text: cue.text.clone(),
            })
            .collect(),
        output_cue_ids: request
            .output_cue_ids
            .iter()
            .map(|cue_id| cue_id.get())
            .collect(),
    };
    let input = serde_json::to_string(&input).map_err(|_| TranslationProviderError::Permanent)?;
    let payload = ResponsesRequest {
        model: identity.model(),
        store: false,
        instructions: SYSTEM_INSTRUCTIONS,
        input,
        text: ResponsesText {
            format: ResponsesFormat {
                format_type: "json_schema",
                name: RESPONSE_SCHEMA_NAME,
                strict: true,
                schema: response_schema(),
            },
        },
    };
    serde_json::to_vec(&payload).map_err(|_| TranslationProviderError::Permanent)
}

fn response_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "cues": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "cue_id": { "type": "integer" },
                        "text": { "type": "string" }
                    },
                    "required": ["cue_id", "text"]
                }
            }
        },
        "required": ["cues"]
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResponsePayload {
    cues: Vec<ResponseCue>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResponseCue {
    cue_id: u32,
    text: String,
}

fn parse_response_body(body: &[u8]) -> Result<TranslationResponse, TranslationProviderError> {
    if body.len() > MAX_PROVIDER_BODY_BYTES {
        return Err(TranslationProviderError::Permanent);
    }
    let root: Value =
        serde_json::from_slice(body).map_err(|_| TranslationProviderError::Permanent)?;
    if contains_refusal(&root) {
        return Err(TranslationProviderError::Permanent);
    }

    let payload = if let Some(output_text) = extract_output_text(&root) {
        serde_json::from_str(output_text).map_err(|_| TranslationProviderError::Permanent)?
    } else if root.get("cues").is_some() {
        root
    } else {
        return Err(TranslationProviderError::Permanent);
    };
    let payload: ResponsePayload =
        serde_json::from_value(payload).map_err(|_| TranslationProviderError::Permanent)?;
    Ok(TranslationResponse {
        cues: payload
            .cues
            .into_iter()
            .map(|cue| TranslatedCue {
                cue_id: nen_domain::subtitle::CueId::new(cue.cue_id),
                text: cue.text,
            })
            .collect(),
    })
}

fn extract_output_text(root: &Value) -> Option<&str> {
    if let Some(text) = root.get("output_text").and_then(Value::as_str) {
        return Some(text);
    }
    root.get("output")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find_map(|item| {
                item.get("content")
                    .and_then(Value::as_array)
                    .and_then(|parts| {
                        parts.iter().find_map(|part| {
                            (part.get("type").and_then(Value::as_str) == Some("output_text"))
                                .then(|| part.get("text").and_then(Value::as_str))
                                .flatten()
                        })
                    })
            })
        })
}

fn contains_refusal(value: &Value) -> bool {
    match value {
        Value::Object(object) => {
            if object
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind == "refusal")
                || object
                    .get("refusal")
                    .is_some_and(|refusal| !refusal.is_null())
            {
                return true;
            }
            object.values().any(contains_refusal)
        }
        Value::Array(items) => items.iter().any(contains_refusal),
        _ => false,
    }
}

fn is_retryable_status(status: u16) -> bool {
    status == 408 || status == 429 || (500..600).contains(&status)
}

fn retry_after_or_fallback(response: &HttpResponse, retry_index: usize) -> Duration {
    response
        .header("Retry-After")
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map(|seconds| Duration::from_secs(seconds.min(10)))
        .unwrap_or_else(|| fallback_retry_delay(retry_index))
}

fn fallback_retry_delay(retry_index: usize) -> Duration {
    match retry_index {
        0 => Duration::from_millis(500),
        _ => Duration::from_secs(2),
    }
}
