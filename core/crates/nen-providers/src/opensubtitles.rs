//! OpenSubtitles hash lookup (NEN-033, ADR-0040).
//!
//! This module only asks the metadata search endpoint for an exact hash match.
//! Subtitle downloads and catalog projection belong to later tasks.

use nen_ports::http::{HttpClient, HttpError, HttpHeader, HttpRequest};
use nen_ports::identity::{
    IdentityLookup, IdentityLookupError, MediaHash, MediaIdentityLookup, VerifiedMediaIdentity,
};
use serde_json::Value;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

const SEARCH_ENDPOINT: &str = "https://api.opensubtitles.com/api/v1/subtitles";
const USER_AGENT: &str = "Nen Player/0.1";
const MAX_REDIRECTS: u8 = 5;
const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_TITLE_CHARS: usize = 512;

/// API credentials are supplied by the platform secure-storage adapter later.
#[derive(Clone, PartialEq, Eq)]
pub struct OpenSubtitlesApiKey(String);

impl OpenSubtitlesApiKey {
    pub fn new(value: &str) -> Result<Self, IdentityLookupError> {
        let trimmed = value.trim();
        if trimmed.is_empty()
            || trimmed.chars().count() > 512
            || trimmed.chars().any(char::is_control)
        {
            return Err(IdentityLookupError::InvalidCredential);
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl fmt::Debug for OpenSubtitlesApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OpenSubtitlesApiKey(<redacted>)")
    }
}

impl fmt::Display for OpenSubtitlesApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

/// Real OpenSubtitles lookup adapter. The HTTP transport is injected so tests
/// never contact the provider and platform adapters can enforce their own I/O
/// boundary.
pub struct OpenSubtitlesIdentityLookup<'a> {
    http: &'a dyn HttpClient,
    api_key: OpenSubtitlesApiKey,
}

impl<'a> OpenSubtitlesIdentityLookup<'a> {
    pub fn new(http: &'a dyn HttpClient, api_key: OpenSubtitlesApiKey) -> Self {
        Self { http, api_key }
    }
}

impl MediaIdentityLookup for OpenSubtitlesIdentityLookup<'_> {
    fn lookup_by_hash(&self, hash: MediaHash) -> Result<IdentityLookup, IdentityLookupError> {
        let hash = hex(hash);
        let initial = format!("{SEARCH_ENDPOINT}?moviehash={hash}&moviehash_match=only");
        let response = request_following(self.http, &self.api_key, &initial)?;
        parse_response(&response.body)
    }
}

fn request_following(
    http: &dyn HttpClient,
    api_key: &OpenSubtitlesApiKey,
    initial: &str,
) -> Result<nen_ports::http::HttpResponse, IdentityLookupError> {
    if !is_allowed_url(initial) {
        return Err(IdentityLookupError::RedirectRejected);
    }

    let mut current = initial.to_owned();
    for redirect_count in 0..=MAX_REDIRECTS {
        let request = HttpRequest::get(
            &current,
            vec![
                HttpHeader {
                    name: "Api-Key".into(),
                    value: api_key.0.clone(),
                },
                HttpHeader {
                    name: "User-Agent".into(),
                    value: USER_AGENT.into(),
                },
                HttpHeader {
                    name: "Accept".into(),
                    value: "application/json".into(),
                },
            ],
            MAX_RESPONSE_BYTES,
        );
        let response = http.send(request).map_err(map_http_error)?;

        if is_redirect(response.status_code) {
            if redirect_count == MAX_REDIRECTS {
                return Err(IdentityLookupError::RedirectRejected);
            }
            let location = response
                .header("Location")
                .ok_or(IdentityLookupError::RedirectRejected)?;
            current = resolve_redirect(&current, location)
                .filter(|url| is_allowed_url(url))
                .ok_or(IdentityLookupError::RedirectRejected)?;
            continue;
        }

        if !(200..300).contains(&response.status_code) {
            return Err(IdentityLookupError::HttpStatus);
        }
        if response.body.len() > MAX_RESPONSE_BYTES {
            return Err(IdentityLookupError::ResponseTooLarge);
        }
        return Ok(response);
    }

    Err(IdentityLookupError::RedirectRejected)
}

fn map_http_error(error: HttpError) -> IdentityLookupError {
    match error {
        HttpError::Transport => IdentityLookupError::Transport,
        HttpError::ResponseTooLarge => IdentityLookupError::ResponseTooLarge,
    }
}

fn parse_response(body: &[u8]) -> Result<IdentityLookup, IdentityLookupError> {
    let root: Value =
        serde_json::from_slice(body).map_err(|_| IdentityLookupError::InvalidResponse)?;
    let rows = root
        .get("data")
        .and_then(Value::as_array)
        .ok_or(IdentityLookupError::InvalidResponse)?;

    let mut identities = Vec::new();
    for row in rows {
        let Some(attributes) = row.get("attributes").and_then(Value::as_object) else {
            continue;
        };
        let exact = attributes
            .get("moviehash_match")
            .or_else(|| attributes.get("movie_hash_match"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !exact {
            continue;
        }
        let Some(details) = attributes.get("feature_details").and_then(Value::as_object) else {
            continue;
        };
        let Some(title) = details
            .get("title")
            .and_then(Value::as_str)
            .and_then(clean_title)
        else {
            continue;
        };
        let identity = VerifiedMediaIdentity {
            title,
            year: bounded_number(details.get("year"), 9_999),
            season: bounded_number(
                details
                    .get("season_number")
                    .or_else(|| details.get("season")),
                u16::MAX as u64,
            ),
            episode: bounded_number(
                details
                    .get("episode_number")
                    .or_else(|| details.get("episode")),
                u16::MAX as u64,
            ),
        };
        if !identities.contains(&identity) {
            identities.push(identity);
        }
    }

    match identities.len() {
        0 => Ok(IdentityLookup::NoMatch),
        1 => Ok(IdentityLookup::Match(identities.remove(0))),
        _ => Ok(IdentityLookup::Ambiguous),
    }
}

fn clean_title(value: &str) -> Option<String> {
    let cleaned: String = value
        .chars()
        .filter(|character| !character.is_control() && !is_bidi_control(*character))
        .collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() || cleaned.chars().count() > MAX_TITLE_CHARS {
        None
    } else {
        Some(cleaned.to_owned())
    }
}

fn is_bidi_control(character: char) -> bool {
    matches!(character, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

fn bounded_number(value: Option<&Value>, maximum: u64) -> Option<u16> {
    let number = value?.as_u64()?;
    (number > 0 && number <= maximum).then_some(number as u16)
}

fn hex(hash: MediaHash) -> String {
    let mut output = String::with_capacity(16);
    for byte in hash.as_bytes() {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn is_redirect(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}

fn is_allowed_url(url: &str) -> bool {
    if url.chars().any(char::is_control) {
        return false;
    }
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    let Some(path_start) = rest.find('/') else {
        return false;
    };
    let host = &rest[..path_start];
    let path = rest[path_start..]
        .split(['?', '#'])
        .next()
        .unwrap_or_default();
    matches!(host, "api.opensubtitles.com" | "vip-api.opensubtitles.com")
        && path == "/api/v1/subtitles"
}

fn resolve_redirect(current: &str, location: &str) -> Option<String> {
    if location.starts_with("https://") {
        return Some(location.to_owned());
    }
    if !location.starts_with('/') {
        return None;
    }
    let authority = current
        .strip_prefix("https://")?
        .split(['/', '?', '#'])
        .next()?;
    Some(format!("https://{authority}{location}"))
}

/// A deterministic provider fake for contract and application tests.
pub struct FakeMediaIdentityLookup {
    answer: Result<IdentityLookup, IdentityLookupError>,
    calls: AtomicUsize,
}

impl FakeMediaIdentityLookup {
    pub fn new(answer: Result<IdentityLookup, IdentityLookupError>) -> Self {
        Self {
            answer,
            calls: AtomicUsize::new(0),
        }
    }

    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::Relaxed)
    }
}

impl MediaIdentityLookup for FakeMediaIdentityLookup {
    fn lookup_by_hash(&self, _hash: MediaHash) -> Result<IdentityLookup, IdentityLookupError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        self.answer.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nen_ports::http::{FakeHttpClient, HttpResponse};

    fn hash() -> MediaHash {
        MediaHash::from_bytes([0, 1, 2, 3, 4, 5, 6, 7])
    }

    fn response(body: &str) -> HttpResponse {
        HttpResponse {
            status_code: 200,
            headers: Vec::new(),
            body: body.as_bytes().to_vec(),
        }
    }

    fn client(body: &str) -> FakeHttpClient {
        let request = HttpRequest::get(
            "https://api.opensubtitles.com/api/v1/subtitles?moviehash=0001020304050607&moviehash_match=only",
            vec![
                HttpHeader {
                    name: "Api-Key".into(),
                    value: "test-key".into(),
                },
                HttpHeader {
                    name: "User-Agent".into(),
                    value: USER_AGENT.into(),
                },
                HttpHeader {
                    name: "Accept".into(),
                    value: "application/json".into(),
                },
            ],
            MAX_RESPONSE_BYTES,
        );
        FakeHttpClient::new(vec![(request, Ok(response(body)))])
    }

    fn lookup(body: &str) -> Result<IdentityLookup, IdentityLookupError> {
        let http = client(body);
        let key = OpenSubtitlesApiKey::new("test-key").expect("valid key");
        OpenSubtitlesIdentityLookup::new(&http, key).lookup_by_hash(hash())
    }

    #[test]
    fn exact_records_with_one_identity_become_a_match() {
        let answer = lookup(
            r#"{"data":[{"attributes":{"moviehash_match":true,"feature_details":{"title":"Film","year":2010}}}]}"#,
        )
        .expect("lookup");
        assert_eq!(
            answer,
            IdentityLookup::Match(VerifiedMediaIdentity {
                title: "Film".into(),
                year: Some(2010),
                season: None,
                episode: None,
            })
        );
    }

    #[test]
    fn duplicate_exact_records_are_one_match_but_different_ones_are_ambiguous() {
        let duplicate = lookup(
            r#"{"data":[{"attributes":{"moviehash_match":true,"feature_details":{"title":"Film","year":2010}}},{"attributes":{"moviehash_match":true,"feature_details":{"title":"Film","year":2010}}}]}"#,
        )
        .expect("lookup");
        assert!(matches!(duplicate, IdentityLookup::Match(_)));

        let ambiguous = lookup(
            r#"{"data":[{"attributes":{"moviehash_match":true,"feature_details":{"title":"Film","year":2010}}},{"attributes":{"moviehash_match":true,"feature_details":{"title":"Other","year":2011}}}]}"#,
        )
        .expect("lookup");
        assert_eq!(ambiguous, IdentityLookup::Ambiguous);
    }

    #[test]
    fn empty_or_non_exact_records_are_no_match() {
        let answer = lookup(
            r#"{"data":[{"attributes":{"moviehash_match":false,"feature_details":{"title":"Film","year":2010}}}]}"#,
        )
        .expect("lookup");
        assert_eq!(answer, IdentityLookup::NoMatch);
    }

    #[test]
    fn credential_and_provider_values_are_redacted() {
        let key = OpenSubtitlesApiKey::new("private-api-key").expect("valid key");
        assert!(!format!("{key:?}").contains("private-api-key"));
        assert!(!format!("{key}").contains("private-api-key"));
        assert!(!format!("{:?}", hash()).contains("0001020304050607"));
    }
}
