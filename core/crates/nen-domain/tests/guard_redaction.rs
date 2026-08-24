//! Guard test for `docs/security-policy.md` §1 (K23): none of the 8
//! forbidden patterns may ever appear in `{:?}` output of a sensitive type.
//!
//! These fixture types are illustrative only (see NEN-006's scope) — the
//! real domain types (`MediaRef` etc.) are introduced by later M2 tasks.

use std::fmt;
use std::path::PathBuf;

use nen_domain::redact::extension;

/// The 8 forbidden patterns from `docs/security-policy.md` §1, each
/// represented by a concrete sample value of that kind.
const MEDIA_URL: &str = "https://media.example.com/stream/season1/ep01.mkv";
const TOKEN_QUERY: &str = "token=eyJhbGciOiJIUzI1NiJ9.super-secret-payload";
const PRIVATE_PATH: &str = "/Users/alice/Movies/Really Personal/S01E01.mkv";
const CUE_TEXT: &str = "I can't believe you did that to her.";
const PROVIDER_RESPONSE: &str = r#"{"choices":[{"text":"raw model output"}]}"#;
const API_KEY: &str = "sk-test-abcdef1234567890";
const PRIVATE_FILE_ID: &str = "osdb-file-773311998";
const PRIVATE_HASH_FILENAME: &str = "a94a8fe5ccb19ba61c4c0873d391e987982fbbd.mkv";

const FORBIDDEN_PATTERNS: [&str; 8] = [
    MEDIA_URL,
    TOKEN_QUERY,
    PRIVATE_PATH,
    CUE_TEXT,
    PROVIDER_RESPONSE,
    API_KEY,
    PRIVATE_FILE_ID,
    PRIVATE_HASH_FILENAME,
];

fn assert_no_forbidden_pattern(label: &str, debug_output: &str) {
    for pattern in FORBIDDEN_PATTERNS {
        assert!(
            !debug_output.contains(pattern),
            "{label}'s Debug output leaked a forbidden pattern: {debug_output:?} contains {pattern:?}"
        );
    }
}

/// Stand-in for a future `MediaRef` — holds a URL and a full local path,
/// both forbidden (#1, #3). `Debug` is hand-written per the K23 rule; only
/// the safe derived extension is shown.
struct MediaRefFixture {
    // Intentionally never read outside the redacted-field constructor:
    // that's the whole point — Debug must never touch it.
    #[allow(dead_code)]
    url: String,
    path: PathBuf,
}

impl fmt::Debug for MediaRefFixture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaRefFixture")
            .field("url", &"<redacted>")
            .field("path", &"<redacted>")
            .field("extension", &extension(self.path.to_str().unwrap_or("")))
            .finish()
    }
}

/// Stand-in for a raw provider response payload (#5).
struct ProviderResponseFixture {
    raw: String,
}

impl fmt::Debug for ProviderResponseFixture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProviderResponseFixture")
            .field("raw", &"<redacted>")
            .field(
                "size_class",
                &nen_domain::redact::size_class(self.raw.len() as u64),
            )
            .finish()
    }
}

/// Stand-in for a user-entered provider API key (#6).
struct ApiKeyFixture {
    #[allow(dead_code)] // intentionally never read — see MediaRefFixture
    key: String,
}

impl fmt::Debug for ApiKeyFixture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiKeyFixture")
            .field("key", &"<redacted>")
            .finish()
    }
}

/// Stand-in for a typed error with a sensitive payload variant — proves
/// payload-carrying error variants follow the same K23 rule (task DoD #3).
// Payload fields are intentionally never read — see MediaRefFixture.
#[allow(dead_code)]
enum ErrorFixture {
    /// Carries the raw dialogue that failed to parse (#4) and an
    /// OpenSubtitles private file ID (#7) it came from.
    ParseFailed {
        cue_text: String,
        source_file_id: String,
    },
    ProviderRejected {
        raw_response: String,
    },
    NotFound,
}

impl fmt::Debug for ErrorFixture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorFixture::ParseFailed { .. } => {
                f.debug_struct("ParseFailed").finish_non_exhaustive()
            }
            ErrorFixture::ProviderRejected { .. } => {
                f.debug_struct("ProviderRejected").finish_non_exhaustive()
            }
            ErrorFixture::NotFound => f.write_str("NotFound"),
        }
    }
}

/// Returns a stable, string-parse-free classification of the error variant.
/// Used to prove (DoD #3) that a caller can distinguish variants without
/// ever touching the redacted payload.
fn classify(error: &ErrorFixture) -> &'static str {
    match error {
        ErrorFixture::ParseFailed { .. } => "parse_failed",
        ErrorFixture::ProviderRejected { .. } => "provider_rejected",
        ErrorFixture::NotFound => "not_found",
    }
}

#[test]
fn media_ref_debug_has_no_forbidden_pattern() {
    let fixture = MediaRefFixture {
        url: MEDIA_URL.to_string() + "?" + TOKEN_QUERY,
        path: PathBuf::from(PRIVATE_PATH),
    };
    let debug = format!("{fixture:?}");
    assert_no_forbidden_pattern("MediaRefFixture", &debug);
    // The safe derived value is allowed to show up.
    assert!(debug.contains("mkv"));
}

#[test]
fn provider_response_debug_has_no_forbidden_pattern() {
    let fixture = ProviderResponseFixture {
        raw: PROVIDER_RESPONSE.to_string(),
    };
    assert_no_forbidden_pattern("ProviderResponseFixture", &format!("{fixture:?}"));
}

#[test]
fn api_key_debug_has_no_forbidden_pattern() {
    let fixture = ApiKeyFixture {
        key: API_KEY.to_string(),
    };
    assert_no_forbidden_pattern("ApiKeyFixture", &format!("{fixture:?}"));
}

#[test]
fn error_payload_debug_has_no_forbidden_pattern() {
    let parse_failed = ErrorFixture::ParseFailed {
        cue_text: CUE_TEXT.to_string(),
        source_file_id: PRIVATE_FILE_ID.to_string(),
    };
    let rejected = ErrorFixture::ProviderRejected {
        raw_response: PROVIDER_RESPONSE.to_string(),
    };
    assert_no_forbidden_pattern("ErrorFixture::ParseFailed", &format!("{parse_failed:?}"));
    assert_no_forbidden_pattern("ErrorFixture::ProviderRejected", &format!("{rejected:?}"));
}

#[test]
fn error_variants_are_distinguishable_without_string_parsing() {
    let parse_failed = ErrorFixture::ParseFailed {
        cue_text: CUE_TEXT.to_string(),
        source_file_id: PRIVATE_FILE_ID.to_string(),
    };
    let rejected = ErrorFixture::ProviderRejected {
        raw_response: PROVIDER_RESPONSE.to_string(),
    };
    let not_found = ErrorFixture::NotFound;

    assert_eq!(classify(&parse_failed), "parse_failed");
    assert_eq!(classify(&rejected), "provider_rejected");
    assert_eq!(classify(&not_found), "not_found");
}

/// Negative test (task DoD #2): a type that breaks the K23 rule by using
/// `#[derive(Debug)]` directly on a sensitive field. This is what the hand
/// Debug rule forbids in real product code — kept here only to prove the
/// pattern check above actually discriminates, rather than trivially
/// passing on anything.
#[derive(Debug)]
struct BadFixtureWithDerivedDebug {
    // rustc doesn't count a derived Debug as a "read" for dead-code
    // purposes, but the derive is exactly how this field leaks below.
    #[allow(dead_code)]
    api_key: String,
}

#[test]
fn derived_debug_on_a_sensitive_type_is_caught_by_the_pattern_check() {
    let bad = BadFixtureWithDerivedDebug {
        api_key: API_KEY.to_string(),
    };
    let debug = format!("{bad:?}");

    // Unlike every hand-written Debug above, this DOES leak the pattern —
    // proving the check in `assert_no_forbidden_pattern` is not vacuous: it
    // would have failed this type had it been asserted the same way.
    assert!(
        debug.contains(API_KEY),
        "expected the derived Debug to leak the API key, proving the guard \
         test can actually catch a #[derive(Debug)] mistake; got {debug:?}"
    );
}
