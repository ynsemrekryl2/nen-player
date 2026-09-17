//! OpenRouter Chat Completions translation provider (NEN-117, ADR-0019).
//!
//! The adapter performs a bounded, cached capability preflight before any
//! translation POST. The only permitted upstream route is OpenAI; all wire
//! payloads cross the shared translation HTTP boundary and remain absent from
//! errors and Debug output.

use super::filename_normalization;
use super::translation_http;
use nen_ports::credentials::ApiKey;
use nen_ports::filename_normalization::{
    FilenameNormalizationError, FilenameNormalizationRequest, FilenameNormalizationResult,
    FilenameNormalizer,
};
use nen_ports::http::{HttpClient, HttpHeader, HttpRequest};
use nen_ports::translation::{
    DocumentAnalysis, DocumentAnalysisRequest, TranslationCall, TranslationProgress,
    TranslationProgressPhase, TranslationProvider, TranslationProviderError,
    TranslationProviderIdentity, TranslationRequest, TranslationResponse,
};
use serde::Serialize;
use serde_json::Value;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub const OPENROUTER_CHAT_ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";
pub const OPENROUTER_MODEL_ENDPOINT_PREFIX: &str = "https://openrouter.ai/api/v1/model/";
pub use super::translation_http::{
    RetrySleeper, MAX_PROVIDER_BODY_BYTES, PROMPT_VERSION, PROVIDER_TIMEOUT_MS, SCHEMA_VERSION,
};

const PROVIDER_ID: &str = "openrouter";
const CAPABILITY_CACHE_SECONDS: u64 = 15 * 60;
const REFERER: &str = "https://nen.player";
const APPLICATION_TITLE: &str = "Nen Player";

/// The result of OpenRouter's model capability lookup.
///
/// The field is intentionally private: callers can ask only the capability
/// that gates this provider, not depend on the upstream's full model catalog.
#[derive(Clone, PartialEq, Eq)]
pub struct ProviderCapabilities {
    structured_outputs: bool,
}

impl ProviderCapabilities {
    pub fn supports_structured_outputs(&self) -> bool {
        self.structured_outputs
    }
}

impl fmt::Debug for ProviderCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProviderCapabilities")
            .field("structured_outputs", &self.structured_outputs)
            .finish()
    }
}

/// Why the preflight did not permit a translation job to start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenRouterPreflightError {
    Cancelled,
    Transient,
    Permanent,
    CapabilityMissing,
}

impl fmt::Display for OpenRouterPreflightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cancelled => "openrouter preflight cancelled",
            Self::Transient => "openrouter preflight temporarily unavailable",
            Self::Permanent => "openrouter preflight failed",
            Self::CapabilityMissing => "openrouter structured output capability missing",
        })
    }
}

impl std::error::Error for OpenRouterPreflightError {}

/// A deterministic clock seam for the 15-minute capability cache.
pub trait Clock: Send + Sync {
    fn now_seconds(&self) -> u64;
}

struct SystemClock;

impl Clock for SystemClock {
    fn now_seconds(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs())
    }
}

struct CachedCapabilities {
    capabilities: ProviderCapabilities,
    expires_at: u64,
}

/// One OpenRouter provider instance for a translation job.
pub struct OpenRouterTranslationProvider<'a> {
    http: translation_http::HttpClientRef<'a>,
    api_key: ApiKey,
    identity: TranslationProviderIdentity,
    chat_endpoint: String,
    model_endpoint: String,
    sleeper: Arc<dyn RetrySleeper>,
    clock: Arc<dyn Clock>,
    capabilities: Mutex<Option<CachedCapabilities>>,
}

impl<'a> OpenRouterTranslationProvider<'a> {
    pub fn new(
        http: &'a dyn HttpClient,
        api_key: ApiKey,
        model: &str,
    ) -> Result<Self, OpenRouterPreflightError> {
        Self::with_endpoint_and_retry_sleeper(
            http,
            api_key,
            model,
            OPENROUTER_CHAT_ENDPOINT,
            &format!("{OPENROUTER_MODEL_ENDPOINT_PREFIX}{model}"),
            translation_http::default_retry_sleeper(),
            Arc::new(SystemClock),
        )
    }

    /// Builds a provider with deterministic endpoint, retry and clock seams
    /// for tests. Both endpoints remain checked against the approved constants
    /// before any HTTP request is sent.
    pub fn with_endpoint_and_retry_sleeper(
        http: &'a dyn HttpClient,
        api_key: ApiKey,
        model: &str,
        chat_endpoint: &str,
        model_endpoint: &str,
        sleeper: Arc<dyn RetrySleeper>,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, OpenRouterPreflightError> {
        Self::with_http(
            translation_http::HttpClientRef::Borrowed(http),
            api_key,
            model,
            chat_endpoint,
            model_endpoint,
            sleeper,
            clock,
        )
    }

    /// Builds a worker-safe provider over the platform-owned HTTP client.
    pub fn with_shared_http(
        http: Arc<dyn HttpClient>,
        api_key: ApiKey,
        model: &str,
    ) -> Result<OpenRouterTranslationProvider<'static>, OpenRouterPreflightError> {
        Self::with_http(
            translation_http::HttpClientRef::Shared(http),
            api_key,
            model,
            OPENROUTER_CHAT_ENDPOINT,
            &format!("{OPENROUTER_MODEL_ENDPOINT_PREFIX}{model}"),
            translation_http::default_retry_sleeper(),
            Arc::new(SystemClock),
        )
    }

    fn with_http<'b>(
        http: translation_http::HttpClientRef<'b>,
        api_key: ApiKey,
        model: &str,
        chat_endpoint: &str,
        model_endpoint: &str,
        sleeper: Arc<dyn RetrySleeper>,
        clock: Arc<dyn Clock>,
    ) -> Result<OpenRouterTranslationProvider<'b>, OpenRouterPreflightError> {
        if !valid_model(model) {
            return Err(OpenRouterPreflightError::Permanent);
        }
        let identity = TranslationProviderIdentity::new(PROVIDER_ID, model)
            .map_err(|_| OpenRouterPreflightError::Permanent)?;
        Ok(OpenRouterTranslationProvider {
            http,
            api_key,
            identity,
            chat_endpoint: chat_endpoint.to_owned(),
            model_endpoint: model_endpoint.to_owned(),
            sleeper,
            clock,
            capabilities: Mutex::new(None),
        })
    }

    /// Checks and caches the selected model's structured-output capability.
    /// `CapabilityMissing` is the provider-side equivalent of the application
    /// `StartRefusal::ProviderCapabilityMissing` mapping added by NEN-118.
    pub fn preflight(
        &self,
        call: &TranslationCall,
    ) -> Result<ProviderCapabilities, OpenRouterPreflightError> {
        if self.chat_endpoint != OPENROUTER_CHAT_ENDPOINT
            || self.model_endpoint
                != format!(
                    "{OPENROUTER_MODEL_ENDPOINT_PREFIX}{}",
                    self.identity.model()
                )
        {
            return Err(OpenRouterPreflightError::Permanent);
        }
        call.checkpoint()
            .map_err(map_translation_error_to_preflight)?;

        let now = self.clock.now_seconds();
        if let Some(cached) = self
            .capabilities
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .filter(|cached| now < cached.expires_at)
        {
            return result_for_capabilities(cached.capabilities.clone());
        }

        let response = translation_http::send_with_retries(
            self.http.as_ref(),
            call,
            self.sleeper.as_ref(),
            || translation_http::bounded_get(&self.model_endpoint, self.headers()),
        )
        .map_err(map_translation_error_to_preflight)?;
        call.checkpoint()
            .map_err(map_translation_error_to_preflight)?;
        let capabilities = parse_capabilities(&response.body)?;
        let expires_at = now.saturating_add(CAPABILITY_CACHE_SECONDS);
        self.capabilities
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .replace(CachedCapabilities {
                capabilities: capabilities.clone(),
                expires_at,
            });
        result_for_capabilities(capabilities)
    }

    fn headers(&self) -> Vec<HttpHeader> {
        vec![
            HttpHeader {
                name: "Authorization".to_owned(),
                value: format!("Bearer {}", self.api_key.expose()),
            },
            HttpHeader {
                name: "HTTP-Referer".to_owned(),
                value: REFERER.to_owned(),
            },
            HttpHeader {
                name: "X-Title".to_owned(),
                value: APPLICATION_TITLE.to_owned(),
            },
        ]
    }
}

impl fmt::Debug for OpenRouterTranslationProvider<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenRouterTranslationProvider")
            .field("identity", &self.identity)
            .field("has_credential", &true)
            .field("endpoints", &"<redacted>")
            .finish()
    }
}

impl TranslationProvider for OpenRouterTranslationProvider<'_> {
    fn identity(&self) -> TranslationProviderIdentity {
        self.identity.clone()
    }

    fn analyze_document(
        &self,
        request: &DocumentAnalysisRequest,
        call: &TranslationCall,
    ) -> Result<DocumentAnalysis, TranslationProviderError> {
        if request.transcript.is_empty() {
            return Err(TranslationProviderError::Permanent);
        }
        self.preflight(call)
            .map_err(map_preflight_error_to_translation)?;
        let body = build_analysis_request_body(&self.identity, request)?;
        if body.len() > MAX_PROVIDER_BODY_BYTES {
            return Err(TranslationProviderError::Permanent);
        }

        call.checkpoint()?;
        let response = translation_http::send_with_retries(
            self.http.as_ref(),
            call,
            self.sleeper.as_ref(),
            || {
                HttpRequest::post_json(
                    &self.chat_endpoint,
                    self.headers(),
                    body.clone(),
                    MAX_PROVIDER_BODY_BYTES,
                    PROVIDER_TIMEOUT_MS,
                )
            },
        )?;
        call.checkpoint()?;
        let (analysis, usage) = translation_http::parse_analysis_response_body(&response.body)?;
        call.report_usage(usage);
        Ok(analysis)
    }

    fn translate(
        &self,
        request: &TranslationRequest,
        call: &TranslationCall,
    ) -> Result<TranslationResponse, TranslationProviderError> {
        let total = u32::try_from(request.output_cue_ids.len())
            .map_err(|_| TranslationProviderError::Permanent)?;
        if total == 0 {
            return Err(TranslationProviderError::Permanent);
        }
        self.preflight(call)
            .map_err(map_preflight_error_to_translation)?;
        let body = build_translation_request_body(&self.identity, request)?;
        if body.len() > MAX_PROVIDER_BODY_BYTES {
            return Err(TranslationProviderError::Permanent);
        }

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
                    &self.chat_endpoint,
                    self.headers(),
                    body.clone(),
                    MAX_PROVIDER_BODY_BYTES,
                    PROVIDER_TIMEOUT_MS,
                )
            },
        )?;
        call.checkpoint()?;
        let (translated, usage) =
            translation_http::parse_translation_response_body(&response.body)?;
        call.report_usage(usage);
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

impl FilenameNormalizer for OpenRouterTranslationProvider<'_> {
    fn normalize(
        &self,
        request: &FilenameNormalizationRequest,
    ) -> Result<FilenameNormalizationResult, FilenameNormalizationError> {
        let call = TranslationCall::without_progress();
        self.preflight(&call).map_err(|error| match error {
            OpenRouterPreflightError::Cancelled => FilenameNormalizationError::Cancelled,
            OpenRouterPreflightError::Transient => FilenameNormalizationError::Transport,
            OpenRouterPreflightError::Permanent => FilenameNormalizationError::HttpStatus,
            OpenRouterPreflightError::CapabilityMissing => {
                FilenameNormalizationError::CapabilityMissing
            }
        })?;
        filename_normalization::normalize_openrouter(
            self.http.as_ref(),
            &self.api_key,
            &self.identity,
            request,
            &call,
        )
    }
}

fn valid_model(model: &str) -> bool {
    !model.trim().is_empty()
        && model.len() <= 256
        && model.matches('/').count() == 1
        && !model.contains("..")
        && model
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_./".contains(character))
}

fn result_for_capabilities(
    capabilities: ProviderCapabilities,
) -> Result<ProviderCapabilities, OpenRouterPreflightError> {
    if capabilities.structured_outputs {
        Ok(capabilities)
    } else {
        Err(OpenRouterPreflightError::CapabilityMissing)
    }
}

fn parse_capabilities(body: &[u8]) -> Result<ProviderCapabilities, OpenRouterPreflightError> {
    if body.len() > MAX_PROVIDER_BODY_BYTES {
        return Err(OpenRouterPreflightError::Permanent);
    }
    let root: Value =
        serde_json::from_slice(body).map_err(|_| OpenRouterPreflightError::Permanent)?;
    let parameters = root
        .get("data")
        .and_then(|data| data.get("supported_parameters"))
        .or_else(|| root.get("supported_parameters"))
        .and_then(Value::as_array)
        .ok_or(OpenRouterPreflightError::Permanent)?;
    Ok(ProviderCapabilities {
        structured_outputs: parameters
            .iter()
            .any(|parameter| parameter.as_str() == Some("structured_outputs")),
    })
}

fn map_translation_error_to_preflight(error: TranslationProviderError) -> OpenRouterPreflightError {
    match error {
        TranslationProviderError::Cancelled => OpenRouterPreflightError::Cancelled,
        TranslationProviderError::Transient => OpenRouterPreflightError::Transient,
        TranslationProviderError::Permanent | TranslationProviderError::ResponseTooLarge => {
            OpenRouterPreflightError::Permanent
        }
    }
}

fn map_preflight_error_to_translation(error: OpenRouterPreflightError) -> TranslationProviderError {
    match error {
        OpenRouterPreflightError::Cancelled => TranslationProviderError::Cancelled,
        OpenRouterPreflightError::Transient => TranslationProviderError::Transient,
        OpenRouterPreflightError::Permanent | OpenRouterPreflightError::CapabilityMissing => {
            TranslationProviderError::Permanent
        }
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage>,
    response_format: ChatResponseFormat,
    provider: ProviderRouting,
    usage: UsageRequest,
}

/// Asks OpenRouter to include its own billed `usage.cost` in the response
/// (NEN-138) — the only source of a real dollar figure this codebase uses;
/// see `nen_ports::translation::TokenUsage` for why no price table exists.
#[derive(Serialize)]
struct UsageRequest {
    include: bool,
}

#[derive(Serialize)]
struct ChatMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct ChatResponseFormat {
    #[serde(rename = "type")]
    format_type: &'static str,
    json_schema: JsonSchemaFormat,
}

#[derive(Serialize)]
struct JsonSchemaFormat {
    name: &'static str,
    strict: bool,
    schema: Value,
}

#[derive(Serialize)]
struct ProviderRouting {
    order: Vec<&'static str>,
    allow_fallbacks: bool,
    require_parameters: bool,
}

fn build_analysis_request_body(
    identity: &TranslationProviderIdentity,
    request: &DocumentAnalysisRequest,
) -> Result<Vec<u8>, TranslationProviderError> {
    build_request_body(
        identity,
        translation_http::analysis_system_instructions(request),
        translation_http::analysis_prompt_input(request)?,
        translation_http::ANALYSIS_SCHEMA_NAME,
        translation_http::analysis_response_schema(),
    )
}

fn build_translation_request_body(
    identity: &TranslationProviderIdentity,
    request: &TranslationRequest,
) -> Result<Vec<u8>, TranslationProviderError> {
    build_request_body(
        identity,
        translation_http::translation_system_instructions(request),
        translation_http::translation_prompt_input(request)?,
        translation_http::TRANSLATION_SCHEMA_NAME,
        translation_http::translation_response_schema(&request.output_cue_ids),
    )
}

fn build_request_body(
    identity: &TranslationProviderIdentity,
    system_instructions: String,
    user_input: String,
    schema_name: &'static str,
    schema: Value,
) -> Result<Vec<u8>, TranslationProviderError> {
    let payload = ChatRequest {
        model: identity.model(),
        messages: vec![
            ChatMessage {
                role: "system",
                content: system_instructions,
            },
            ChatMessage {
                role: "user",
                content: user_input,
            },
        ],
        response_format: ChatResponseFormat {
            format_type: "json_schema",
            json_schema: JsonSchemaFormat {
                name: schema_name,
                strict: true,
                schema,
            },
        },
        provider: ProviderRouting {
            order: vec!["OpenAI"],
            allow_fallbacks: false,
            require_parameters: true,
        },
        usage: UsageRequest { include: true },
    };
    serde_json::to_vec(&payload).map_err(|_| TranslationProviderError::Permanent)
}
