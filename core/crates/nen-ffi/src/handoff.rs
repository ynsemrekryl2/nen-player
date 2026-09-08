//! Handoff intake's single FFI boundary (NEN-080, ADR-0043).
//!
//! The shell hands over a raw `argv` array or a raw URL string; the core
//! decides what it means and hands back either a locator the shell can open
//! or a typed reason it cannot. Nothing here re-implements ADR-0043's rules —
//! this module only translates [`nen_app::handoff`]'s answer across the gate.
//!
//! # Security (K23)
//!
//! The locator crosses back **out** of the gate, unlike a subtitle path
//! ([`crate::subtitles`]) — the shell needs it to actually open the medium.
//! That is why [`FfiHandoffLocator`] and [`FfiHandoffRequest`] do not derive
//! `Debug`: a derive would print a path or a URL the moment a caller logged
//! the value it just received. Both are written by hand instead, on the same
//! pattern as [`crate::remote_evidence::FfiHttpRequest`].
//! `tests/guard_ffi_handoff_debug.rs` proves it against derived twins.

use nen_app::handoff::{self, HandoffLocator, HandoffRejection, HandoffRequest};
use std::fmt;

/// What was handed over — a local path or a remote URL, mirroring
/// [`HandoffLocator`] across the gate.
#[derive(Clone, PartialEq, Eq, uniffi::Enum)]
pub enum FfiHandoffLocator {
    LocalPath { path: String },
    Remote { url: String },
}

impl fmt::Debug for FfiHandoffLocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalPath { .. } => f
                .debug_struct("FfiHandoffLocator::LocalPath")
                .field("path", &"<redacted>")
                .finish(),
            Self::Remote { .. } => f
                .debug_struct("FfiHandoffLocator::Remote")
                .field("url", &"<redacted>")
                .finish(),
        }
    }
}

/// One handoff, ready for the shell to open the way `⌘O` does.
///
/// `start_position_ms` crosses the gate here; the macOS shell turns it into a
/// seek once the medium it targets has loaded (NEN-081).
#[derive(Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiHandoffRequest {
    pub locator: FfiHandoffLocator,
    pub start_position_ms: Option<u64>,
}

impl fmt::Debug for FfiHandoffRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FfiHandoffRequest")
            .field("locator", &self.locator)
            .field("start_position_ms", &self.start_position_ms)
            .finish()
    }
}

/// Why nothing was opened. Carries no payload — the variant is the whole
/// message, safe for the shell to show and safe for a log to print.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Error)]
pub enum FfiHandoffRejection {
    NoLocator,
    UnsupportedScheme,
    MalformedLocator,
}

impl fmt::Display for FfiHandoffRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NoLocator => "no_locator",
            Self::UnsupportedScheme => "unsupported_scheme",
            Self::MalformedLocator => "malformed_locator",
        })
    }
}

impl From<HandoffRejection> for FfiHandoffRejection {
    fn from(value: HandoffRejection) -> Self {
        match value {
            HandoffRejection::NoLocator => Self::NoLocator,
            HandoffRejection::UnsupportedScheme => Self::UnsupportedScheme,
            HandoffRejection::MalformedLocator => Self::MalformedLocator,
        }
    }
}

impl TryFrom<HandoffRequest> for FfiHandoffRequest {
    type Error = FfiHandoffRejection;

    fn try_from(value: HandoffRequest) -> Result<Self, Self::Error> {
        let locator = match value.locator {
            HandoffLocator::LocalPath(path) => {
                // `Path` is `OsStr` underneath and macOS allows non-UTF-8
                // bytes in it; UniFFI's string type does not. Refusing here
                // rather than lossily converting keeps the shell from opening
                // a path that is not the one that was actually handed over.
                let path = path
                    .to_str()
                    .ok_or(FfiHandoffRejection::MalformedLocator)?
                    .to_owned();
                FfiHandoffLocator::LocalPath { path }
            }
            HandoffLocator::Remote(url) => FfiHandoffLocator::Remote { url },
        };
        Ok(Self {
            locator,
            start_position_ms: value.start_position_ms,
        })
    }
}

/// Reads a launch's `argv`, the way NEN-078 measured a sender writing them.
#[uniffi::export]
pub fn parse_handoff_argv(argv: Vec<String>) -> Result<FfiHandoffRequest, FfiHandoffRejection> {
    handoff::parse_argv(&argv)
        .map_err(FfiHandoffRejection::from)
        .and_then(FfiHandoffRequest::try_from)
}

/// Reads what LaunchServices delivers: an opened document's `file://` URL, or
/// a `nenplayer://` string a sender built.
#[uniffi::export]
pub fn parse_handoff_url(url: String) -> Result<FfiHandoffRequest, FfiHandoffRejection> {
    handoff::parse_url(&url)
        .map_err(FfiHandoffRejection::from)
        .and_then(FfiHandoffRequest::try_from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_local_argv_locator_crosses_the_gate() {
        let request = parse_handoff_argv(vec![
            "/Applications/Nen Player.app/Contents/MacOS/NenPlayer".to_owned(),
            "/Users/x/Film.mkv".to_owned(),
        ])
        .expect("request");
        assert_eq!(
            request.locator,
            FfiHandoffLocator::LocalPath {
                path: "/Users/x/Film.mkv".to_owned()
            }
        );
        assert_eq!(request.start_position_ms, None);
    }

    #[test]
    fn a_remote_url_locator_crosses_the_gate() {
        let request = parse_handoff_url("nenplayer://https://example.test/Film.mkv#t=5".to_owned())
            .expect("request");
        assert_eq!(
            request.locator,
            FfiHandoffLocator::Remote {
                url: "https://example.test/Film.mkv".to_owned()
            }
        );
        assert_eq!(request.start_position_ms, Some(5_000));
    }

    #[test]
    fn an_unsupported_scheme_becomes_a_typed_rejection() {
        assert_eq!(
            parse_handoff_url("ftp://example.test/a.mkv".to_owned()),
            Err(FfiHandoffRejection::UnsupportedScheme)
        );
    }

    #[test]
    fn a_launch_with_no_locator_is_rejected() {
        assert_eq!(
            parse_handoff_argv(vec!["NenPlayer".to_owned()]),
            Err(FfiHandoffRejection::NoLocator)
        );
    }
}
