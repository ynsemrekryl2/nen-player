//! Remote media evidence collection (NEN-036, ADR-0009 and ADR-0039).
//!
//! This module owns the policy loop. A platform adapter performs one request;
//! this layer validates schemes, redirects, response shape and range windows
//! before handing the resulting fields to `nen-identity`.

use nen_identity::evidence::MediaEvidence;
use nen_identity::{declared_name, os_hash, url_hints};
use nen_ports::http::{
    ByteRange, HttpClient, HttpError, HttpMethod, HttpRequest, HttpResponse, MAX_REDIRECTS,
    MAX_RESPONSE_BYTES,
};
use std::fmt;

/// The policy is data in the core so native adapters cannot quietly diverge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemoteEvidencePolicy {
    pub max_redirects: u8,
    pub max_response_bytes: usize,
}

impl Default for RemoteEvidencePolicy {
    fn default() -> Self {
        Self {
            max_redirects: MAX_REDIRECTS,
            max_response_bytes: MAX_RESPONSE_BYTES,
        }
    }
}

/// Remote evidence failures never carry the URL, headers or response body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteEvidenceError {
    InvalidUrl,
    InsecureRedirect,
    MissingRedirectLocation,
    RedirectLimitExceeded,
    HttpStatus,
    InvalidResponse,
    ResponseTooLarge,
    Transport,
}

impl fmt::Display for RemoteEvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidUrl => "invalid remote media URL",
            Self::InsecureRedirect => "insecure redirect",
            Self::MissingRedirectLocation => "redirect without a location",
            Self::RedirectLimitExceeded => "redirect limit exceeded",
            Self::HttpStatus => "remote media request failed",
            Self::InvalidResponse => "invalid remote media response",
            Self::ResponseTooLarge => "remote media response too large",
            Self::Transport => "remote media transport failed",
        })
    }
}

impl std::error::Error for RemoteEvidenceError {}

/// Collects the remote evidence that can be obtained without downloading the
/// media. Range support is optional: a server that says `Accept-Ranges: none`
/// produces name/size evidence and simply has no hash.
pub fn collect_remote_evidence(
    client: &dyn HttpClient,
    url: &str,
) -> Result<MediaEvidence, RemoteEvidenceError> {
    collect_with_policy(client, url, RemoteEvidencePolicy::default())
}

pub fn collect_with_policy(
    client: &dyn HttpClient,
    url: &str,
    policy: RemoteEvidencePolicy,
) -> Result<MediaEvidence, RemoteEvidenceError> {
    validate_url(url)?;
    let (final_url, head) = request_following(client, HttpMethod::Head, url, None, policy)?;
    if !(200..300).contains(&head.status_code) {
        return Err(RemoteEvidenceError::HttpStatus);
    }

    let mut size = head.header("Content-Length").and_then(parse_size);
    let declared = head
        .header("Content-Disposition")
        .and_then(declared_name::from_content_disposition)
        .or_else(|| url_hints::path_hints(&final_url).into_iter().next());

    let mut head_window = None;
    let mut tail_window = None;
    if !header_is_none(head.header("Accept-Ranges")) {
        head_window = range_window(
            client,
            &final_url,
            ByteRange::Inclusive {
                start: 0,
                end: os_hash::CHUNK_BYTES as u64 - 1,
            },
            size,
            policy,
        )?;

        if size.is_none() {
            size = head_window.as_ref().and_then(|window| window.total_size);
        }

        if let Some(total) = size {
            tail_window = range_window(
                client,
                &final_url,
                ByteRange::Suffix {
                    length: os_hash::CHUNK_BYTES as u64,
                },
                Some(total),
                policy,
            )?;
        }
    }

    let mut evidence = MediaEvidence::for_remote_url(&final_url);
    if let Some(name) = declared.as_deref() {
        evidence = evidence.with_declared_name(name);
    }
    if let Some(size) = size {
        evidence = evidence.with_size(size);
    }
    if let (Some(size), Some(head), Some(tail)) = (size, head_window, tail_window) {
        if let Ok(hash) = os_hash::of(size, &head.body, &tail.body) {
            evidence = evidence.with_os_hash(hash);
        }
    }
    Ok(evidence)
}

struct RangeWindow {
    body: Vec<u8>,
    total_size: Option<u64>,
}

fn range_window(
    client: &dyn HttpClient,
    url: &str,
    range: ByteRange,
    expected_size: Option<u64>,
    policy: RemoteEvidencePolicy,
) -> Result<Option<RangeWindow>, RemoteEvidenceError> {
    let (final_url, response) =
        request_following(client, HttpMethod::Get, url, Some(range), policy)?;
    if final_url != url || response.status_code != 206 {
        return Ok(None);
    }
    if response.body.len() != os_hash::CHUNK_BYTES
        || response.body.len() > policy.max_response_bytes
    {
        return Ok(None);
    }
    let Some((start, end, total_size)) = parse_content_range(response.header("Content-Range"))
    else {
        return Ok(None);
    };
    let expected_start = match range {
        ByteRange::Inclusive { start, .. } => start,
        ByteRange::Suffix { length } => {
            let Some(total) = expected_size.or(total_size) else {
                return Ok(None);
            };
            total
                .checked_sub(length)
                .ok_or(RemoteEvidenceError::InvalidResponse)?
        }
    };
    if start != expected_start
        || end.saturating_sub(start).saturating_add(1) != os_hash::CHUNK_BYTES as u64
        || expected_size.is_some_and(|expected| Some(expected) != total_size)
    {
        return Ok(None);
    }
    Ok(Some(RangeWindow {
        body: response.body,
        total_size,
    }))
}

fn request_following(
    client: &dyn HttpClient,
    method: HttpMethod,
    url: &str,
    range: Option<ByteRange>,
    policy: RemoteEvidencePolicy,
) -> Result<(String, HttpResponse), RemoteEvidenceError> {
    let mut current = url.to_owned();
    let mut redirects = 0;
    loop {
        validate_url(&current)?;
        let mut request = match range {
            Some(range) => HttpRequest::range(&current, range),
            None => HttpRequest::head(&current),
        };
        request.method = method;
        request.max_body_bytes = if method == HttpMethod::Head {
            0
        } else {
            policy.max_response_bytes
        };
        let response = client.send(request).map_err(map_http_error)?;
        if !is_redirect(response.status_code) {
            return Ok((current, response));
        }
        if redirects >= policy.max_redirects {
            return Err(RemoteEvidenceError::RedirectLimitExceeded);
        }
        let location = response
            .header("Location")
            .ok_or(RemoteEvidenceError::MissingRedirectLocation)?;
        let next = resolve_location(&current, location)?;
        if is_https(&current) && !is_https(&next) {
            return Err(RemoteEvidenceError::InsecureRedirect);
        }
        current = next;
        redirects += 1;
    }
}

fn map_http_error(error: HttpError) -> RemoteEvidenceError {
    match error {
        HttpError::Transport => RemoteEvidenceError::Transport,
        HttpError::ResponseTooLarge => RemoteEvidenceError::ResponseTooLarge,
    }
}

/// The scheme gate every remote locator passes.
///
/// `pub(crate)` rather than private because NEN-080's handoff parser needs the
/// **same** answer this module gives: a second opinion about what an `http`
/// URL is would be a second thing to keep correct, and the two would drift the
/// first time either changed.
pub(crate) fn validate_url(url: &str) -> Result<(), RemoteEvidenceError> {
    if url
        .chars()
        .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(RemoteEvidenceError::InvalidUrl);
    }
    let Some((scheme, rest)) = url.split_once("://") else {
        return Err(RemoteEvidenceError::InvalidUrl);
    };
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return Err(RemoteEvidenceError::InvalidUrl);
    }
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    if authority_end == 0 || rest[..authority_end].contains('\\') {
        return Err(RemoteEvidenceError::InvalidUrl);
    }
    Ok(())
}

fn resolve_location(base: &str, location: &str) -> Result<String, RemoteEvidenceError> {
    if location.is_empty() || location.chars().any(|character| character.is_control()) {
        return Err(RemoteEvidenceError::InvalidUrl);
    }
    if location.contains("://") {
        validate_url(location)?;
        return Ok(location.to_owned());
    }
    let Some((scheme, rest)) = base.split_once("://") else {
        return Err(RemoteEvidenceError::InvalidUrl);
    };
    let authority_end = rest.find('/').unwrap_or(rest.len());
    let origin = format!("{scheme}://{}", &rest[..authority_end]);
    if location.starts_with('/') {
        return Ok(format!("{origin}{location}"));
    }
    let path_end = base.find(['?', '#']).unwrap_or(base.len());
    let path = &base[..path_end];
    let directory_end = path.rfind('/').map(|index| index + 1).unwrap_or(path.len());
    let prefix = &path[..directory_end];
    let resolved = if prefix.starts_with(&origin) {
        format!("{prefix}{location}")
    } else {
        format!("{origin}/{location}")
    };
    validate_url(&resolved)?;
    Ok(resolved)
}

fn scheme_of(url: &str) -> Option<&str> {
    url.split_once("://").map(|(scheme, _)| scheme)
}

fn is_https(url: &str) -> bool {
    scheme_of(url).is_some_and(|scheme| scheme.eq_ignore_ascii_case("https"))
}

fn is_redirect(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}

fn header_is_none(value: Option<&str>) -> bool {
    value.is_some_and(|value| value.trim().eq_ignore_ascii_case("none"))
}

fn parse_size(value: &str) -> Option<u64> {
    value.trim().parse().ok()
}

fn parse_content_range(value: Option<&str>) -> Option<(u64, u64, Option<u64>)> {
    let value = value?.trim();
    let (unit, range) = value.split_once(' ')?;
    if !unit.eq_ignore_ascii_case("bytes") {
        return None;
    }
    let (bounds, total) = range.split_once('/')?;
    let (start, end) = bounds.split_once('-')?;
    Some((start.parse().ok()?, end.parse().ok()?, total.parse().ok()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nen_ports::http::{HttpHeader, HttpRequest};
    use std::sync::Mutex;

    struct Fake {
        expected: Mutex<Vec<(HttpRequest, HttpResponse)>>,
    }

    impl Fake {
        fn new(expected: Vec<(HttpRequest, HttpResponse)>) -> Self {
            Self {
                expected: Mutex::new(expected),
            }
        }
    }

    impl HttpClient for Fake {
        fn send(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
            let mut expected = self.expected.lock().map_err(|_| HttpError::Transport)?;
            let Some((wanted, _response)) = expected.first() else {
                return Err(HttpError::Transport);
            };
            if wanted != &request {
                return Err(HttpError::Transport);
            }
            Ok(expected.remove(0).1)
        }
    }

    fn response(status_code: u16, headers: &[(&str, &str)], body: Vec<u8>) -> HttpResponse {
        HttpResponse {
            status_code,
            headers: headers
                .iter()
                .map(|(name, value)| HttpHeader {
                    name: (*name).into(),
                    value: (*value).into(),
                })
                .collect(),
            body,
        }
    }

    fn windows(contents: &[u8]) -> (Vec<u8>, Vec<u8>) {
        (
            contents[..os_hash::CHUNK_BYTES].to_vec(),
            contents[contents.len() - os_hash::CHUNK_BYTES..].to_vec(),
        )
    }

    #[test]
    fn range_windows_produce_the_same_hash_as_local_contents() {
        let contents: Vec<u8> = (0..131_072).map(|byte| (byte % 251) as u8).collect();
        let (head, tail) = windows(&contents);
        let url = "https://media.invalid/opaque?token=secret";
        let fake = Fake::new(vec![
            (
                HttpRequest::head(url),
                response(
                    200,
                    &[
                        ("Content-Length", "131072"),
                        ("Accept-Ranges", "bytes"),
                        (
                            "Content-Disposition",
                            "attachment; filename*=UTF-8''Film.2010.mkv",
                        ),
                    ],
                    Vec::new(),
                ),
            ),
            (
                HttpRequest::range(
                    url,
                    ByteRange::Inclusive {
                        start: 0,
                        end: 65_535,
                    },
                ),
                response(206, &[("Content-Range", "bytes 0-65535/131072")], head),
            ),
            (
                HttpRequest::range(url, ByteRange::Suffix { length: 65_536 }),
                response(206, &[("Content-Range", "bytes 65536-131071/131072")], tail),
            ),
        ]);

        let evidence = collect_remote_evidence(&fake, url).expect("fake response is valid");
        assert_eq!(evidence.size_bytes(), Some(contents.len() as u64));
        assert_eq!(evidence.os_hash(), os_hash::of_bytes(&contents).ok());
        let identity = evidence.resolve();
        assert_eq!(identity.title.as_deref(), Some("Film"));
    }

    #[test]
    fn redirect_basename_and_no_ranges_still_produce_evidence() {
        let initial = "https://media.invalid/opaque?token=secret";
        let final_url = "https://cdn.invalid/library/Film.2010.mkv";
        let fake = Fake::new(vec![
            (
                HttpRequest::head(initial),
                response(302, &[("Location", final_url)], Vec::new()),
            ),
            (
                HttpRequest::head(final_url),
                response(200, &[("Accept-Ranges", "none")], Vec::new()),
            ),
        ]);

        let evidence = collect_remote_evidence(&fake, initial).expect("redirect is valid");
        assert_eq!(evidence.size_bytes(), None);
        assert_eq!(evidence.os_hash(), None);
        assert_eq!(evidence.resolve().title.as_deref(), Some("Film"));
    }

    #[test]
    fn unsafe_redirects_and_redirect_loops_are_rejected() {
        let downgrade = "https://media.invalid/a";
        let downgrade_fake = Fake::new(vec![(
            HttpRequest::head(downgrade),
            response(302, &[("Location", "http://media.invalid/b")], Vec::new()),
        )]);
        assert_eq!(
            collect_remote_evidence(&downgrade_fake, downgrade),
            Err(RemoteEvidenceError::InsecureRedirect)
        );

        let loop_url = "https://media.invalid/a";
        let mut responses = Vec::new();
        for _ in 0..=MAX_REDIRECTS {
            responses.push((
                HttpRequest::head(loop_url),
                response(302, &[("Location", loop_url)], Vec::new()),
            ));
        }
        let loop_fake = Fake::new(responses);
        assert_eq!(
            collect_remote_evidence(&loop_fake, loop_url),
            Err(RemoteEvidenceError::RedirectLimitExceeded)
        );
    }

    #[test]
    fn invalid_scheme_is_rejected_before_the_client_is_called() {
        let fake = Fake::new(Vec::new());
        assert_eq!(
            collect_remote_evidence(&fake, "file:///private/secret.mkv"),
            Err(RemoteEvidenceError::InvalidUrl)
        );
    }

    #[test]
    fn hostile_filename_is_flattened_before_identity_resolution() {
        let url = "https://media.invalid/opaque";
        let fake = Fake::new(vec![(
            HttpRequest::head(url),
            response(
                200,
                &[
                    ("Accept-Ranges", "none"),
                    (
                        "Content-Disposition",
                        "attachment; filename*=UTF-8''..%2F..%2Fsecret%00.mkv",
                    ),
                ],
                Vec::new(),
            ),
        )]);
        let evidence = collect_remote_evidence(&fake, url).expect("invalid name is non-fatal");
        let debug = format!("{evidence:?}");
        assert!(!debug.contains("secret"));
        assert_eq!(evidence.resolve().title, None);
    }
}
