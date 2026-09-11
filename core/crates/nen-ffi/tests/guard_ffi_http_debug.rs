//! K23 guard for `NEN-115`'s FFI-crossing HTTP types. Same sentinel and
//! not-blind technique as `guard_ffi_credential_debug.rs`.

use nen_ffi::remote_evidence::{FfiHttpError, FfiHttpHeader, FfiHttpMethod, FfiHttpRequest};

const SENTINEL_BODY: &str = "super-secret-provider-payload-Zzqxvun";
const SENTINEL_AUTH: &str = "Bearer Zzqxvun-api-key";

fn forbidden(output: &str) {
    assert!(!output.contains(SENTINEL_BODY), "body leaked: {output}");
    assert!(
        !output.contains(SENTINEL_AUTH),
        "auth header leaked: {output}"
    );
}

#[test]
fn ffi_http_request_debug_never_prints_its_body_or_authorization_header() {
    let request = FfiHttpRequest {
        method: FfiHttpMethod::Post,
        url: "https://api.openai.com/v1/responses".into(),
        range: None,
        headers: vec![FfiHttpHeader {
            name: "Authorization".into(),
            value: SENTINEL_AUTH.into(),
        }],
        max_body_bytes: 1024 * 1024,
        body: Some(SENTINEL_BODY.as_bytes().to_vec()),
        timeout_ms: Some(60_000),
    };

    let debug = format!("{request:?}");
    forbidden(&debug);
    assert!(debug.contains("body_length"));

    for error in [FfiHttpError::Transport, FfiHttpError::ResponseTooLarge] {
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
