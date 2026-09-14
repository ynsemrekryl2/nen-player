//! Shared bounded HTTP, prompt and response rules for real translators.
//!
//! OpenAI Responses and OpenRouter Chat Completions have different wire
//! envelopes, but their safety boundary is one contract: the same prompt
//! input, strict cue schema, response parser, retry budget and payload-free
//! error classification.

use nen_domain::subtitle::CueId;
use nen_ports::http::{HttpClient, HttpError, HttpHeader, HttpMethod, HttpRequest, HttpResponse};
use nen_ports::translation::{
    TranslatedCue, TranslationCall, TranslationProviderError, TranslationRequest,
    TranslationResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

pub const MAX_PROVIDER_BODY_BYTES: usize = 1024 * 1024;
pub const PROVIDER_TIMEOUT_MS: u64 = 60_000;
pub const PROMPT_VERSION: u32 = 1;
pub const SCHEMA_VERSION: u32 = 1;
pub const RESPONSE_SCHEMA_NAME: &str = "subtitle_translation";
pub const MAX_RETRIES: usize = 2;
pub const SYSTEM_INSTRUCTIONS: &str = "Translate subtitle cues. Use context_cues and context_terms only as context. Translate only output_cue_ids. Preserve meaning, register, names, terminology, and meaningful line breaks. Return only the requested JSON object with no explanation or extra fields.";

/// Provides the bounded wait between transient attempts.
pub trait RetrySleeper: Send + Sync {
    fn sleep(&self, duration: Duration, call: &TranslationCall);
}

/// A provider can borrow a deterministic test client or own the shared
/// platform client used by a real translation environment. Keeping both
/// shapes here lets the contract tests stay lightweight while the worker gets
/// a `'static` provider that is safe to move onto its own thread.
pub(crate) enum HttpClientRef<'a> {
    Borrowed(&'a dyn HttpClient),
    Shared(Arc<dyn HttpClient>),
}

impl HttpClientRef<'_> {
    pub(crate) fn as_ref(&self) -> &dyn HttpClient {
        match self {
            Self::Borrowed(client) => *client,
            Self::Shared(client) => client.as_ref(),
        }
    }
}

struct ThreadRetrySleeper;

impl RetrySleeper for ThreadRetrySleeper {
    fn sleep(&self, duration: Duration, _call: &TranslationCall) {
        std::thread::sleep(duration);
    }
}

pub(crate) fn default_retry_sleeper() -> Arc<dyn RetrySleeper> {
    Arc::new(ThreadRetrySleeper)
}

/// Sends one bounded request and retries only the ADR-0019 transient classes.
/// The request closure is called once per attempt, keeping all provider wire
/// envelopes in their adapter while sharing retry and error semantics.
pub(crate) fn send_with_retries(
    http: &dyn HttpClient,
    call: &TranslationCall,
    sleeper: &dyn RetrySleeper,
    mut request: impl FnMut() -> HttpRequest,
) -> Result<HttpResponse, TranslationProviderError> {
    let mut retries = 0usize;
    loop {
        call.checkpoint()?;
        match http.send(request()) {
            Err(HttpError::ResponseTooLarge) => {
                return Err(TranslationProviderError::Permanent);
            }
            Err(HttpError::Transport) => {
                if retries == MAX_RETRIES {
                    return Err(TranslationProviderError::Transient);
                }
                wait_for_retry(call, sleeper, fallback_retry_delay(retries))?;
                retries += 1;
            }
            Ok(response) if is_retryable_status(response.status_code) => {
                if retries == MAX_RETRIES {
                    return Err(TranslationProviderError::Transient);
                }
                let delay = retry_after_or_fallback(&response, retries);
                wait_for_retry(call, sleeper, delay)?;
                retries += 1;
            }
            Ok(response) if (200..300).contains(&response.status_code) => return Ok(response),
            Ok(_) => return Err(TranslationProviderError::Permanent),
        }
    }
}

fn wait_for_retry(
    call: &TranslationCall,
    sleeper: &dyn RetrySleeper,
    delay: Duration,
) -> Result<(), TranslationProviderError> {
    call.checkpoint()?;
    sleeper.sleep(delay, call);
    call.checkpoint()
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

pub(crate) fn bounded_get(url: &str, headers: Vec<HttpHeader>) -> HttpRequest {
    HttpRequest {
        method: HttpMethod::Get,
        url: url.to_owned(),
        range: None,
        headers,
        max_body_bytes: MAX_PROVIDER_BODY_BYTES,
        body: None,
        timeout_ms: Some(PROVIDER_TIMEOUT_MS),
    }
}

pub(crate) fn prompt_input(
    request: &TranslationRequest,
) -> Result<String, TranslationProviderError> {
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
    serde_json::to_string(&input).map_err(|_| TranslationProviderError::Permanent)
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

pub(crate) fn response_schema() -> Value {
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

pub(crate) fn parse_response_body(
    body: &[u8],
) -> Result<TranslationResponse, TranslationProviderError> {
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
                cue_id: CueId::new(cue.cue_id),
                text: cue.text,
            })
            .collect(),
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

fn extract_output_text(root: &Value) -> Option<&str> {
    if let Some(text) = root.get("output_text").and_then(Value::as_str) {
        return Some(text);
    }
    if let Some(text) = root
        .get("output")
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
    {
        return Some(text);
    }
    root.get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| {
            choices.iter().find_map(|choice| {
                choice
                    .get("message")
                    .and_then(|message| message.get("content"))
                    .and_then(Value::as_str)
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
