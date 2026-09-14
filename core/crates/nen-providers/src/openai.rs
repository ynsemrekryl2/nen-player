//! Direct OpenAI Responses API translation provider (NEN-116, ADR-0019).
//!
//! The adapter owns one call's short-lived credential and turns the provider's
//! untrusted response into the shared translation DTO. It never validates cue
//! completeness or order; `nen-translate` remains the authoritative local
//! validator. Request/response payloads stay inside the HTTP boundary and no
//! provider-specific payload is carried by an error or a `Debug` surface.

use super::translation_http;
use nen_ports::credentials::ApiKey;
use nen_ports::http::{HttpClient, HttpHeader, HttpRequest};
use nen_ports::translation::{
    TranslationCall, TranslationProgress, TranslationProgressPhase, TranslationProvider,
    TranslationProviderError, TranslationProviderIdentity, TranslationRequest, TranslationResponse,
};
use serde::Serialize;
use serde_json::Value;
use std::fmt;
use std::sync::Arc;

pub const OPENAI_RESPONSES_ENDPOINT: &str = "https://api.openai.com/v1/responses";
pub use super::translation_http::{
    RetrySleeper, MAX_PROVIDER_BODY_BYTES, PROMPT_VERSION, PROVIDER_TIMEOUT_MS, SCHEMA_VERSION,
};

const PROVIDER_ID: &str = "openai";

/// One direct OpenAI provider instance for a translation job.
pub struct OpenAiTranslationProvider<'a> {
    http: translation_http::HttpClientRef<'a>,
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
            translation_http::default_retry_sleeper(),
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
        Self::with_http(
            translation_http::HttpClientRef::Borrowed(http),
            api_key,
            model,
            endpoint,
            sleeper,
        )
    }

    /// Builds a worker-safe provider over the platform-owned HTTP client.
    pub fn with_shared_http(
        http: Arc<dyn HttpClient>,
        api_key: ApiKey,
        model: &str,
    ) -> Result<OpenAiTranslationProvider<'static>, TranslationProviderError> {
        Self::with_http(
            translation_http::HttpClientRef::Shared(http),
            api_key,
            model,
            OPENAI_RESPONSES_ENDPOINT,
            translation_http::default_retry_sleeper(),
        )
    }

    fn with_http<'b>(
        http: translation_http::HttpClientRef<'b>,
        api_key: ApiKey,
        model: &str,
        endpoint: &str,
        sleeper: Arc<dyn RetrySleeper>,
    ) -> Result<OpenAiTranslationProvider<'b>, TranslationProviderError> {
        if model.trim().is_empty() || model.chars().any(char::is_control) {
            return Err(TranslationProviderError::Permanent);
        }
        let identity = TranslationProviderIdentity::new(PROVIDER_ID, model)?;
        Ok(OpenAiTranslationProvider {
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

        let response = translation_http::send_with_retries(
            self.http.as_ref(),
            call,
            self.sleeper.as_ref(),
            || {
                HttpRequest::post_json(
                    &self.endpoint,
                    vec![HttpHeader {
                        name: "Authorization".to_owned(),
                        value: format!("Bearer {}", self.api_key.expose()),
                    }],
                    body.clone(),
                    MAX_PROVIDER_BODY_BYTES,
                    PROVIDER_TIMEOUT_MS,
                )
            },
        )?;

        call.checkpoint()?;
        let translated = translation_http::parse_response_body(&response.body)?;
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
        call.finish(translated)
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

fn build_request_body(
    identity: &TranslationProviderIdentity,
    request: &TranslationRequest,
) -> Result<Vec<u8>, TranslationProviderError> {
    let input = translation_http::prompt_input(request)?;
    let payload = ResponsesRequest {
        model: identity.model(),
        store: false,
        instructions: translation_http::SYSTEM_INSTRUCTIONS,
        input,
        text: ResponsesText {
            format: ResponsesFormat {
                format_type: "json_schema",
                name: translation_http::RESPONSE_SCHEMA_NAME,
                strict: true,
                schema: translation_http::response_schema(),
            },
        },
    };
    serde_json::to_vec(&payload).map_err(|_| TranslationProviderError::Permanent)
}
