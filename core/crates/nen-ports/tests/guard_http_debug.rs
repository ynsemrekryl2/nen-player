//! K23 guard for `NEN-115`'s `POST`/JSON body addition to `HttpRequest`
//! (`docs/security-policy.md` — media URL, token-bearing query and, now,
//! a provider request body / `Authorization` header must never reach a
//! `Debug` line).
//!
//! Same technique as `nen-ports`'s own `guard_persistence_index_debug.rs`:
//! a distinctive sentinel proves the guard is not vacuous, and a
//! `#[derive(Debug)]` twin of the same payload is shown to leak it.

use nen_ports::http::{HttpError, HttpHeader, HttpRequest};

/// A provider request body distinctive enough that it cannot appear by
/// accident (ADR-0019's OpenAI/OpenRouter payload).
const SENTINEL_BODY: &str = "super-secret-provider-payload-Zzqxvun";
/// An `Authorization` header value carrying a credential (ADR-0020).
const SENTINEL_AUTH: &str = "Bearer Zzqxvun-api-key";

fn forbidden(output: &str) {
    assert!(!output.contains(SENTINEL_BODY), "body leaked: {output}");
    assert!(
        !output.contains(SENTINEL_AUTH),
        "auth header leaked: {output}"
    );
}

#[test]
fn a_post_requests_debug_never_prints_its_body_or_authorization_header() {
    let request = HttpRequest::post_json(
        "https://api.openai.com/v1/responses",
        vec![HttpHeader {
            name: "Authorization".into(),
            value: SENTINEL_AUTH.into(),
        }],
        SENTINEL_BODY.as_bytes().to_vec(),
        1024 * 1024,
        60_000,
    );

    forbidden(&format!("{request:?}"));
    assert!(format!("{request:?}").contains("body_length"));

    for error in [HttpError::Transport, HttpError::ResponseTooLarge] {
        forbidden(&format!("{error:?} {error}"));
    }
}

#[test]
fn the_guard_is_not_blind_to_a_derived_debug_twin() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct LeakyTwin {
        body: String,
        authorization: String,
    }

    let leaked = format!(
        "{:?}",
        LeakyTwin {
            body: SENTINEL_BODY.into(),
            authorization: SENTINEL_AUTH.into(),
        }
    );
    assert!(leaked.contains(SENTINEL_BODY));
    assert!(leaked.contains(SENTINEL_AUTH));
}
