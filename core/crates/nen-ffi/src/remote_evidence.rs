//! Remote evidence's single FFI boundary (ADR-0006 and ADR-0039).
//!
//! Requests and responses cross inward to the core. The public result contains
//! only presence facts; filename, URL, header values and the hash stay inside
//! the core and cannot be printed by a platform binding.

use nen_app::ports::http::{
    ByteRange, HttpClient, HttpError, HttpHeader, HttpRequest, HttpResponse,
};
use nen_app::remote_evidence::{self, RemoteEvidenceError};
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum FfiHttpMethod {
    Head,
    Get,
    Post,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum FfiByteRange {
    Inclusive { start: u64, end: u64 },
    Suffix { length: u64 },
}

#[derive(Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiHttpRequest {
    pub method: FfiHttpMethod,
    pub url: String,
    pub range: Option<FfiByteRange>,
    pub headers: Vec<FfiHttpHeader>,
    pub max_body_bytes: u64,
    pub body: Option<Vec<u8>>,
    pub timeout_ms: Option<u64>,
}

impl From<HttpRequest> for FfiHttpRequest {
    fn from(value: HttpRequest) -> Self {
        Self {
            method: value.method.into(),
            url: value.url,
            range: value.range.map(Into::into),
            headers: value
                .headers
                .into_iter()
                .map(|header| FfiHttpHeader {
                    name: header.name,
                    value: header.value,
                })
                .collect(),
            max_body_bytes: value.max_body_bytes as u64,
            body: value.body,
            timeout_ms: value.timeout_ms,
        }
    }
}

#[derive(Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiHttpHeader {
    pub name: String,
    pub value: String,
}

#[derive(Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiHttpResponse {
    pub status_code: u16,
    pub headers: Vec<FfiHttpHeader>,
    pub body: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Error)]
pub enum FfiHttpError {
    Transport,
    ResponseTooLarge,
}

impl fmt::Display for FfiHttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Transport => "transport",
            Self::ResponseTooLarge => "response_too_large",
        })
    }
}

#[uniffi::export(with_foreign)]
pub trait ForeignHttpClient: Send + Sync {
    fn send(&self, request: FfiHttpRequest) -> Result<FfiHttpResponse, FfiHttpError>;
}

struct ForeignHttpClientAdapter {
    inner: Arc<dyn ForeignHttpClient>,
}

impl HttpClient for ForeignHttpClientAdapter {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        let response = self.inner.send(request.into()).map_err(HttpError::from)?;
        Ok(HttpResponse {
            status_code: response.status_code,
            headers: response
                .headers
                .into_iter()
                .map(|header| HttpHeader {
                    name: header.name,
                    value: header.value,
                })
                .collect(),
            body: response.body,
        })
    }
}

impl From<FfiHttpMethod> for nen_app::ports::http::HttpMethod {
    fn from(value: FfiHttpMethod) -> Self {
        match value {
            FfiHttpMethod::Head => Self::Head,
            FfiHttpMethod::Get => Self::Get,
            FfiHttpMethod::Post => Self::Post,
        }
    }
}

impl From<nen_app::ports::http::HttpMethod> for FfiHttpMethod {
    fn from(value: nen_app::ports::http::HttpMethod) -> Self {
        match value {
            nen_app::ports::http::HttpMethod::Head => Self::Head,
            nen_app::ports::http::HttpMethod::Get => Self::Get,
            nen_app::ports::http::HttpMethod::Post => Self::Post,
        }
    }
}

impl From<FfiByteRange> for ByteRange {
    fn from(value: FfiByteRange) -> Self {
        match value {
            FfiByteRange::Inclusive { start, end } => Self::Inclusive { start, end },
            FfiByteRange::Suffix { length } => Self::Suffix { length },
        }
    }
}

impl From<ByteRange> for FfiByteRange {
    fn from(value: ByteRange) -> Self {
        match value {
            ByteRange::Inclusive { start, end } => Self::Inclusive { start, end },
            ByteRange::Suffix { length } => Self::Suffix { length },
        }
    }
}

impl From<FfiHttpError> for HttpError {
    fn from(value: FfiHttpError) -> Self {
        match value {
            FfiHttpError::Transport => Self::Transport,
            FfiHttpError::ResponseTooLarge => Self::ResponseTooLarge,
        }
    }
}

/// The only evidence facts exposed to the platform. Sensitive values remain
/// in the core for the future provider call.
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Record)]
pub struct FfiRemoteEvidenceReport {
    pub has_declared_name: bool,
    pub size_bytes: Option<u64>,
    pub has_os_hash: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Error)]
pub enum FfiRemoteEvidenceError {
    InvalidUrl,
    InsecureRedirect,
    MissingRedirectLocation,
    RedirectLimitExceeded,
    HttpStatus,
    InvalidResponse,
    ResponseTooLarge,
    Transport,
}

impl fmt::Display for FfiRemoteEvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidUrl => "invalid_url",
            Self::InsecureRedirect => "insecure_redirect",
            Self::MissingRedirectLocation => "missing_redirect_location",
            Self::RedirectLimitExceeded => "redirect_limit_exceeded",
            Self::HttpStatus => "http_status",
            Self::InvalidResponse => "invalid_response",
            Self::ResponseTooLarge => "response_too_large",
            Self::Transport => "transport",
        })
    }
}

impl From<RemoteEvidenceError> for FfiRemoteEvidenceError {
    fn from(value: RemoteEvidenceError) -> Self {
        match value {
            RemoteEvidenceError::InvalidUrl => Self::InvalidUrl,
            RemoteEvidenceError::InsecureRedirect => Self::InsecureRedirect,
            RemoteEvidenceError::MissingRedirectLocation => Self::MissingRedirectLocation,
            RemoteEvidenceError::RedirectLimitExceeded => Self::RedirectLimitExceeded,
            RemoteEvidenceError::HttpStatus => Self::HttpStatus,
            RemoteEvidenceError::InvalidResponse => Self::InvalidResponse,
            RemoteEvidenceError::ResponseTooLarge => Self::ResponseTooLarge,
            RemoteEvidenceError::Transport => Self::Transport,
        }
    }
}

#[uniffi::export]
pub fn collect_remote_evidence(
    client: Arc<dyn ForeignHttpClient>,
    url: String,
) -> Result<FfiRemoteEvidenceReport, FfiRemoteEvidenceError> {
    let client = ForeignHttpClientAdapter { inner: client };
    let evidence = remote_evidence::collect_remote_evidence(&client, &url)
        .map_err(FfiRemoteEvidenceError::from)?;
    Ok(FfiRemoteEvidenceReport {
        has_declared_name: evidence.has_declared_name(),
        size_bytes: evidence.size_bytes(),
        has_os_hash: evidence.os_hash().is_some(),
    })
}

impl fmt::Debug for FfiHttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FfiHttpRequest")
            .field("method", &self.method)
            .field("url", &"<redacted>")
            .field("range", &self.range)
            .field("header_count", &self.headers.len())
            .field("max_body_bytes", &self.max_body_bytes)
            .field("body_length", &self.body.as_ref().map(Vec::len))
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

impl fmt::Debug for FfiHttpHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FfiHttpHeader")
            .field("name", &self.name)
            .field("value", &"<redacted>")
            .finish()
    }
}

impl fmt::Debug for FfiHttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FfiHttpResponse")
            .field("status_code", &self.status_code)
            .field("header_count", &self.headers.len())
            .field("body_length", &self.body.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_http_debug_never_prints_url_query_host_or_filename() {
        let request = FfiHttpRequest {
            method: FfiHttpMethod::Post,
            url: "https://private.example/opaque?token=secret".into(),
            range: None,
            headers: vec![FfiHttpHeader {
                name: "Authorization".into(),
                value: "Bearer secret-key".into(),
            }],
            max_body_bytes: 0,
            body: Some(b"super-secret-provider-payload".to_vec()),
            timeout_ms: Some(60_000),
        };
        let response = FfiHttpResponse {
            status_code: 200,
            headers: vec![FfiHttpHeader {
                name: "Content-Disposition".into(),
                value: "attachment; filename=private.mkv".into(),
            }],
            body: vec![1, 2, 3],
        };

        let request_debug = format!("{request:?}");
        let response_debug = format!("{response:?}");
        assert!(!request_debug.contains("private.example"));
        assert!(!request_debug.contains("secret"));
        assert!(!request_debug.contains("super-secret-provider-payload"));
        assert!(request_debug.contains("body_length"));
        assert!(!response_debug.contains("private.mkv"));
        assert!(response_debug.contains("body_length"));
    }
}
