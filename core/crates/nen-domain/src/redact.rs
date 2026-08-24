//! Helpers for keeping sensitive values out of `Debug`/log output (K23).
//!
//! See `docs/security-policy.md` §1 for the forbidden-pattern list and the
//! hand-written-`Debug` rule these helpers exist to make easy to follow.

use std::fmt;

/// Wraps a value so it can never accidentally leak through `Debug` or
/// `Display` — both always print `<redacted>` regardless of `T`.
///
/// Use [`Redacted::reveal`] to get at the real value for actual use (e.g.
/// opening the file at a path, sending a URL over the network). The point
/// of this type is not to prevent use, only to prevent an accidental
/// `#[derive(Debug)]`-style leak from ever printing the inner value.
pub struct Redacted<T>(T);

impl<T> Redacted<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Returns the wrapped value for real use. Callers must not pass the
    /// result to a log, error message, or further `Debug`/`Display` output.
    pub fn reveal(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl<T> fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

/// Extracts a file extension from a path or filename without exposing the
/// rest of it. Safe to log per `docs/security-policy.md` "Loglanabilecekler".
///
/// Returns `None` when there is no extension (e.g. no `.`, or the name
/// starts with `.` and has nothing after it — a dotfile, not an extension).
pub fn extension(path_or_name: &str) -> Option<&str> {
    let name = path_or_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(path_or_name);
    let dot = name.rfind('.')?;
    if dot == 0 {
        return None;
    }
    let ext = &name[dot + 1..];
    if ext.is_empty() {
        None
    } else {
        Some(ext)
    }
}

/// Buckets a byte count into a small fixed set of named classes. Safe to log
/// per `docs/security-policy.md` "Loglanabilecekler" ("boyut sınıfı") —
/// unlike the exact byte count, a class carries no per-file identifying
/// signal.
pub fn size_class(bytes: u64) -> &'static str {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;

    if bytes < 64 * KIB {
        "tiny"
    } else if bytes < 4 * MIB {
        "small"
    } else if bytes < 256 * MIB {
        "medium"
    } else if bytes < 4 * GIB {
        "large"
    } else {
        "huge"
    }
}

/// Maps a host to itself only if it is on the fixed `allowlist`; otherwise
/// returns a constant placeholder. Safe to log per
/// `docs/security-policy.md` "Loglanabilecekler" ("redakte edilmiş host
/// adı — yalnız approved listede olan sabit host adları").
///
/// This is deliberately not a redaction of *part* of the host — an
/// unrecognized host is fully replaced, since anything derived from
/// untrusted input could itself be identifying.
pub fn redact_host<'a>(host: &str, allowlist: &[&'a str]) -> &'a str {
    allowlist
        .iter()
        .find(|&&approved| approved == host)
        .copied()
        .unwrap_or("<redacted-host>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacted_debug_and_display_never_show_the_value() {
        let secret = Redacted::new("sk-super-secret-token".to_string());
        assert_eq!(format!("{secret:?}"), "<redacted>");
        assert_eq!(format!("{secret}"), "<redacted>");
        assert_eq!(secret.reveal(), "sk-super-secret-token");
    }

    #[test]
    fn extension_strips_the_rest_of_the_path() {
        assert_eq!(
            extension("/Users/alice/Movies/The Show S01E01.mkv"),
            Some("mkv")
        );
        assert_eq!(extension("subtitle.srt"), Some("srt"));
        assert_eq!(extension("no_extension"), None);
        assert_eq!(extension(".gitignore"), None);
    }

    #[test]
    fn size_class_buckets_without_exact_bytes() {
        assert_eq!(size_class(1024), "tiny");
        assert_eq!(size_class(1024 * 1024), "small");
        assert_eq!(size_class(100 * 1024 * 1024), "medium");
        assert_eq!(size_class(1024 * 1024 * 1024), "large");
        assert_eq!(size_class(5u64 * 1024 * 1024 * 1024), "huge");
    }

    #[test]
    fn redact_host_only_returns_allowlisted_hosts() {
        let allowlist = ["api.opensubtitles.com", "api.openai.com"];
        assert_eq!(
            redact_host("api.opensubtitles.com", &allowlist),
            "api.opensubtitles.com"
        );
        assert_eq!(
            redact_host("evil.example.com", &allowlist),
            "<redacted-host>"
        );
    }
}
