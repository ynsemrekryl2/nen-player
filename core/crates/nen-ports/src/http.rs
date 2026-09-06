//! Transport-neutral HTTP port for remote media evidence (ADR-0039).
//!
//! The port deliberately knows nothing about identity, URLs are never printed,
//! and response bodies are bounded by each request. The application layer owns
//! redirect and evidence semantics; platform adapters only perform one request
//! at a time.

use std::fmt;

/// The largest redirect chain accepted for one evidence collection.
pub const MAX_REDIRECTS: u8 = 5;
/// The largest response a remote evidence request may return.
pub const MAX_RESPONSE_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Head,
    Get,
}

impl fmt::Debug for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Head => "Head",
            Self::Get => "Get",
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ByteRange {
    Inclusive { start: u64, end: u64 },
    Suffix { length: u64 },
}

impl fmt::Debug for ByteRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inclusive { start, end } => f
                .debug_struct("Inclusive")
                .field("start", start)
                .field("end", end)
                .finish(),
            Self::Suffix { length } => f.debug_struct("Suffix").field("length", length).finish(),
        }
    }
}

/// One request made by an adapter. The URL is intentionally hidden from
/// `Debug` because it can contain a private host or a token-bearing query.
#[derive(Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub range: Option<ByteRange>,
    pub max_body_bytes: usize,
}

impl HttpRequest {
    pub fn head(url: &str) -> Self {
        Self {
            method: HttpMethod::Head,
            url: url.to_owned(),
            range: None,
            max_body_bytes: 0,
        }
    }

    pub fn range(url: &str, range: ByteRange) -> Self {
        Self {
            method: HttpMethod::Get,
            url: url.to_owned(),
            range: Some(range),
            max_body_bytes: MAX_RESPONSE_BYTES,
        }
    }
}

impl fmt::Debug for HttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpRequest")
            .field("method", &self.method)
            .field("url", &"<redacted>")
            .field("range", &self.range)
            .field("max_body_bytes", &self.max_body_bytes)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

impl fmt::Debug for HttpHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpHeader")
            .field("name", &self.name)
            .field("value", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HttpHeader>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|header| header.name.eq_ignore_ascii_case(name))
            .map(|header| header.value.as_str())
    }
}

impl fmt::Debug for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpResponse")
            .field("status_code", &self.status_code)
            .field("header_count", &self.headers.len())
            .field("body_length", &self.body.len())
            .finish()
    }
}

/// Adapter failures contain no URL, header or body payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpError {
    Transport,
    ResponseTooLarge,
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Transport => "transport error",
            Self::ResponseTooLarge => "response too large",
        })
    }
}

impl std::error::Error for HttpError {}

/// Performs one bounded request. Redirects are deliberately not part of this
/// trait: the application layer must validate every redirect target.
pub trait HttpClient: Send + Sync {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, HttpError>;
}

/// A deterministic in-memory response source used by core contract tests.
#[cfg(test)]
pub struct FakeHttpClient {
    responses: std::sync::Mutex<Vec<(HttpRequest, Result<HttpResponse, HttpError>)>>,
}

#[cfg(test)]
impl FakeHttpClient {
    pub fn new(responses: Vec<(HttpRequest, Result<HttpResponse, HttpError>)>) -> Self {
        Self {
            responses: std::sync::Mutex::new(responses),
        }
    }
}

#[cfg(test)]
impl HttpClient for FakeHttpClient {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        let mut responses = self
            .responses
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let Some((expected, _response)) = responses.first() else {
            return Err(HttpError::Transport);
        };
        if expected != &request {
            return Err(HttpError::Transport);
        }
        let (_, response) = responses.remove(0);
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_debug_redacts_url_and_response_debug_redacts_headers() {
        let request = HttpRequest::head("https://private.example/stream.mkv?token=secret");
        let response = HttpResponse {
            status_code: 200,
            headers: vec![HttpHeader {
                name: "Content-Disposition".into(),
                value: "attachment; filename=private.mkv".into(),
            }],
            body: vec![1, 2, 3],
        };

        let request_debug = format!("{request:?}");
        let response_debug = format!("{response:?}");
        assert!(!request_debug.contains("private.example"));
        assert!(!request_debug.contains("secret"));
        assert!(!response_debug.contains("private.mkv"));
        assert!(response_debug.contains("body_length"));
    }

    #[test]
    fn header_lookup_is_case_insensitive() {
        let response = HttpResponse {
            status_code: 200,
            headers: vec![HttpHeader {
                name: "cOnTeNt-LeNgTh".into(),
                value: "131072".into(),
            }],
            body: Vec::new(),
        };
        assert_eq!(response.header("content-length"), Some("131072"));
    }
}
