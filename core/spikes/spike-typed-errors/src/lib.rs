//! NEN-010 — can a Rust error enum cross the FFI boundary so that Swift (and,
//! later, Kotlin) can `switch`/`when` over the exact variant without ever
//! parsing a message string, and without a payload-carrying variant leaking
//! a K23-forbidden pattern anywhere along the way — Rust `{:?}`, the wire,
//! *or* Swift's own default printing of the caught error?
//!
//! **This is not product code** (CLAUDE.md rule 7). It opens its own throwaway
//! FFI gate, which ADR-0028 permits for spikes; ADR-0006's "`nen-ffi` is the
//! single gate" rule stays binding for `core/crates/*`.
//!
//! ## Design — sanitize before construction, not after
//!
//! [`AppError`] is a UniFFI "rich" error (no `#[uniffi(flat_error)]`): its
//! variant fields cross the boundary as real typed data, not a flattened
//! `to_string()`. That is exactly what NEN-006's Rust-only `ErrorFixture`
//! (see `nen-domain`'s `guard_redaction.rs`) could not prove — it never left
//! the process.
//!
//! Crossing a real FFI boundary changes the threat model. NEN-006's
//! hand-written `Debug` guarantee protects `{:?}` output *in Rust*. It says
//! nothing about the foreign side: once a raw sensitive value sits in a
//! Swift associated value, Swift's own `String(describing:)` /
//! `CustomStringConvertible` machinery can print it — Rust's `Debug` impl is
//! never consulted on that side. So the guarantee this spike needs cannot be
//! "hide it in `Debug`" — it has to be "the raw value never crosses at all."
//!
//! Both constructors below ([`make_parse_error`], [`make_network_error`])
//! take the *raw* untrusted input (a full local path, a full URL with a
//! token query) and reduce it to an already-safe derived value — the file
//! extension, an allowlist-checked host — via `nen_domain::redact`, before
//! it ever reaches an `AppError` field. Nothing sensitive is ever assigned
//! to a field, so there is nothing left for the FFI wire, Rust's `Debug`, or
//! Swift's default printing to leak. `Debug` on [`AppError`] is still
//! hand-written rather than derived (K23's blanket rule), so a later
//! maintainer who adds a raw field back cannot silently regress it.
//!
//! `Capability` and `ValidationField` are small closed enums, not free
//! strings — the same "typed, not stringly" principle applied one level
//! deeper than the outer error variant.

use std::fmt;

use nen_domain::redact::{extension, redact_host};

uniffi::setup_scaffolding!();

/// Provider hosts this spike recognizes. Real product code will have a
/// larger, ADR-governed list (`docs/security-policy.md` §3); this is
/// illustrative only, same scope note as NEN-006.
const KNOWN_PROVIDER_HOSTS: [&str; 2] = ["api.opensubtitles.com", "api.openai.com"];

/// A capability the current session may not have. Closed set — adding one
/// is a Rust-side change, not a caller-supplied string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum Capability {
    LocalAsr,
    CloudTranslation,
    HardwareDecode,
}

/// A field name a validation rule failed on. Closed set for the same reason
/// as [`Capability`] — never the raw user-supplied value that failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum ValidationField {
    SubtitlePath,
    MediaDuration,
    LanguageCode,
}

/// Representative error taxonomy for the spike (not the real M2 taxonomy —
/// see `adr: [5]`'s note in the task file). A UniFFI "rich" error: every
/// variant's fields cross to Swift as typed data, not a flattened message.
///
/// Every field on every payload-carrying variant is *already safe to log*
/// per `docs/security-policy.md` "Loglanabilecekler" by the time it is
/// constructed — see the module doc for why that has to hold before
/// construction, not after.
#[derive(uniffi::Error)]
pub enum AppError {
    /// A subtitle/media file failed to parse. Carries the file extension
    /// (safe) and a line number — never the path (K23 forbidden pattern #3).
    Parse {
        extension: Option<String>,
        line: u32,
    },
    /// A call to a subtitle/translation provider failed. Carries an
    /// allowlist-checked host (safe) and an HTTP status — never the request
    /// URL, which may carry a token query (K23 forbidden patterns #1, #2).
    Network { host: String, status: u16 },
    /// The operation was cancelled cooperatively (see NEN-009). No payload.
    Cancelled,
    /// The caller asked for a capability the current session doesn't have.
    CapabilityUnavailable { capability: Capability },
    /// A validation rule failed on a specific, closed-set field.
    Validation { field: ValidationField },
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Parse { extension, line } => f
                .debug_struct("Parse")
                .field("extension", extension)
                .field("line", line)
                .finish(),
            AppError::Network { host, status } => f
                .debug_struct("Network")
                .field("host", host)
                .field("status", status)
                .finish(),
            AppError::Cancelled => f.write_str("Cancelled"),
            AppError::CapabilityUnavailable { capability } => f
                .debug_struct("CapabilityUnavailable")
                .field("capability", capability)
                .finish(),
            AppError::Validation { field } => {
                f.debug_struct("Validation").field("field", field).finish()
            }
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for AppError {}

/// Reduces a raw path to the one field-safe fact about it: its extension.
/// The raw `path` is never read again after this call.
fn make_parse_error(path: &str, line: u32) -> AppError {
    AppError::Parse {
        extension: extension(path).map(str::to_string),
        line,
    }
}

/// Extracts just the host from a raw URL (which may carry a token query)
/// and reduces it to an allowlist-checked value. The raw `url` — scheme,
/// path, query — is never read again after this call.
fn make_network_error(url: &str, status: u16) -> AppError {
    let without_scheme = url.split("://").nth(1).unwrap_or(url);
    let host_end = without_scheme
        .find(['/', '?'])
        .unwrap_or(without_scheme.len());
    let raw_host = &without_scheme[..host_end];
    AppError::Network {
        host: redact_host(raw_host, &KNOWN_PROVIDER_HOSTS).to_string(),
        status,
    }
}

#[uniffi::export]
pub fn trigger_parse_error(path: String, line: u32) -> Result<(), AppError> {
    Err(make_parse_error(&path, line))
}

#[uniffi::export]
pub fn trigger_network_error(url: String, status: u16) -> Result<(), AppError> {
    Err(make_network_error(&url, status))
}

#[uniffi::export]
pub fn trigger_cancelled() -> Result<(), AppError> {
    Err(AppError::Cancelled)
}

#[uniffi::export]
pub fn trigger_capability_unavailable(capability: Capability) -> Result<(), AppError> {
    Err(AppError::CapabilityUnavailable { capability })
}

#[uniffi::export]
pub fn trigger_validation_error(field: ValidationField) -> Result<(), AppError> {
    Err(AppError::Validation { field })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Same 8-forbidden-pattern vocabulary as
    // `nen-domain/tests/guard_redaction.rs` (K23) — only the two patterns
    // relevant to this spike's variants (#1/#2 media URL + token query, #3
    // private path) are exercised here.
    const PRIVATE_PATH: &str = "/Users/alice/Movies/Really Personal/S01E01.mkv";
    const MEDIA_URL_WITH_TOKEN: &str =
        "https://media.example.com/stream/season1/ep01.mkv?token=eyJhbGciOiJIUzI1NiJ9.super-secret-payload";

    fn assert_no_forbidden_pattern(label: &str, debug_output: &str) {
        for pattern in [PRIVATE_PATH, MEDIA_URL_WITH_TOKEN, "media.example.com"] {
            assert!(
                !debug_output.contains(pattern),
                "{label}'s Debug output leaked a forbidden pattern: {debug_output:?} contains {pattern:?}"
            );
        }
    }

    #[test]
    fn parse_error_debug_has_no_forbidden_pattern_and_keeps_the_extension() {
        let err = make_parse_error(PRIVATE_PATH, 42);
        let debug = format!("{err:?}");
        assert_no_forbidden_pattern("Parse", &debug);
        assert!(
            debug.contains("mkv"),
            "safe derived extension should still show: {debug}"
        );
    }

    #[test]
    fn network_error_debug_has_no_forbidden_pattern_and_redacts_unknown_host() {
        let err = make_network_error(MEDIA_URL_WITH_TOKEN, 503);
        let debug = format!("{err:?}");
        assert_no_forbidden_pattern("Network", &debug);
        assert!(
            debug.contains("<redacted-host>"),
            "unlisted host must be replaced, not partially shown: {debug}"
        );
    }

    #[test]
    fn network_error_keeps_allowlisted_host() {
        let err = make_network_error("https://api.opensubtitles.com/v1/search?x=1", 200);
        let debug = format!("{err:?}");
        assert!(debug.contains("api.opensubtitles.com"));
    }

    #[test]
    fn variants_are_distinguishable_without_string_parsing() {
        fn classify(e: &AppError) -> &'static str {
            match e {
                AppError::Parse { .. } => "parse",
                AppError::Network { .. } => "network",
                AppError::Cancelled => "cancelled",
                AppError::CapabilityUnavailable { .. } => "capability_unavailable",
                AppError::Validation { .. } => "validation",
            }
        }

        assert_eq!(classify(&make_parse_error(PRIVATE_PATH, 1)), "parse");
        assert_eq!(
            classify(&make_network_error(MEDIA_URL_WITH_TOKEN, 500)),
            "network"
        );
        assert_eq!(classify(&AppError::Cancelled), "cancelled");
        assert_eq!(
            classify(&AppError::CapabilityUnavailable {
                capability: Capability::LocalAsr
            }),
            "capability_unavailable"
        );
        assert_eq!(
            classify(&AppError::Validation {
                field: ValidationField::SubtitlePath
            }),
            "validation"
        );
    }

    /// Negative test (K23 / CLAUDE.md rule 3): proves the assertion above is
    /// not vacuous. If a maintainer bypasses the sanitizing constructor and
    /// stuffs a raw path straight into the `extension` field — the same
    /// mistake `docs/security-policy.md`'s `Debug` rule exists to catch —
    /// the hand-written `Debug` above has no way to tell and *does* leak it.
    /// The guarantee lives in the constructor discipline, not the type.
    #[test]
    fn bypassing_the_sanitizing_constructor_does_leak() {
        let misused = AppError::Parse {
            extension: Some(PRIVATE_PATH.to_string()),
            line: 1,
        };
        let debug = format!("{misused:?}");
        assert!(
            debug.contains(PRIVATE_PATH),
            "expected the raw path to leak when the constructor is bypassed, proving \
             the no-leak tests above are exercising real protection, not a vacuous check; \
             got {debug:?}"
        );
    }
}
