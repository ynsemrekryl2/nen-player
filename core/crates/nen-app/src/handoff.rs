//! What another application handed us, turned into something the session can
//! run (ADR-0043).
//!
//! # Why the parsing is here and not in the shell (ADR-0043 Karar 4)
//!
//! The shell collects a raw `argv` array or a raw URL string and passes it
//! through untouched. Every judgement about it — which flags mean something,
//! which schemes are allowed, what a start position is worth — is made here.
//! Two reasons, both from the ADR: M10's Android intent surface inherits the
//! same policy instead of writing a second copy of it in Kotlin, and K23's
//! guard over this input has exactly one place to stand.
//!
//! # Why handoff has no port (ADR-0043 Karar 3)
//!
//! Every row of `docs/architecture.md`'s port table is a capability the core
//! *calls*. Handoff runs the other way: the OS pushes it at the shell. So
//! there is nothing to inject and nothing to fake — this module is a pure
//! function over a string, and the result is fed to the load path that already
//! exists.
//!
//! # Security
//!
//! Everything arriving here is hostile (`security-policy.md` §2): the locator
//! is attacker-shaped text until a gate says otherwise, no `unwrap`/`expect`
//! is used, and no rejection carries the input back out. The scheme gate for
//! `http`/`https` is [`crate::remote_evidence`]'s own — a second opinion about
//! URLs would be a second thing to keep correct.
//!
//! `Debug` is written by hand for both value types so a locator cannot reach a
//! log (K23 #1, #2, #3); `tests/guard_handoff_debug.rs` proves it against
//! derived twins.

use crate::remote_evidence::validate_url;
use std::fmt;
use std::path::{Path, PathBuf};

/// The flag names that carry a start position (ADR-0043 Karar 2).
///
/// Three spellings, one meaning: mpv's, VLC's, and Nen Player's own. Accepting
/// the first two is not impersonation — NEN-078 Bulgu 6 measured that what a
/// sender pins is the **CLI contract**, not an application identity, so this
/// opens the door to every sender that speaks it rather than to one of them.
///
/// A closed set. A fourth spelling is added only by a new measurement.
const START_FLAGS: [&str; 3] = ["--start", "--start-time", "--start-position"];

/// The fragment that carries a start position on the scheme surface.
///
/// The Media Fragments URI convention, so `nenplayer://https://host/a.mkv#t=42`
/// reads the way a browser-shaped sender would already write it.
const TIME_FRAGMENT: &str = "#t=";

/// The prefix the `nenplayer` scheme adds in front of an untouched media URL.
const SCHEME_PREFIX: &str = "nenplayer://";

/// What was handed over, once it is known to be one of the two things the
/// player can open.
///
/// The split is not cosmetic: a local path goes through the same security
/// scope and recents bookkeeping `⌘O` uses, and a remote URL deliberately
/// goes through neither (NEN-042 — a query string can carry a token).
#[derive(Clone, PartialEq, Eq)]
pub enum HandoffLocator {
    /// An absolute path on this machine.
    LocalPath(PathBuf),
    /// An `http` or `https` URL that passed [`validate_url`].
    Remote(String),
}

impl HandoffLocator {
    /// The safe derivative a log may carry: what kind of thing this is.
    const fn kind(&self) -> &'static str {
        match self {
            Self::LocalPath(_) => "local",
            Self::Remote(_) => "remote",
        }
    }

    /// The extension of a local path, which `security-policy.md` §1 lists as
    /// loggable. `None` for a remote URL — the last path segment of a URL is
    /// not a filename this side gets to claim.
    fn extension(&self) -> Option<&str> {
        match self {
            Self::LocalPath(path) => path.extension().and_then(|value| value.to_str()),
            Self::Remote(_) => None,
        }
    }

    /// `http` or `https`, lowercased — a scheme names no host and no path.
    fn scheme(&self) -> Option<&'static str> {
        match self {
            Self::LocalPath(_) => None,
            Self::Remote(url) => {
                if url.len() >= 5 && url[..5].eq_ignore_ascii_case("https") {
                    Some("https")
                } else {
                    Some("http")
                }
            }
        }
    }
}

impl fmt::Debug for HandoffLocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandoffLocator")
            .field("kind", &self.kind())
            .field("value", &"<redacted>")
            .field("extension", &self.extension())
            .field("scheme", &self.scheme())
            .finish()
    }
}

/// One handoff, resolved.
///
/// `start_position_ms` is parsed here but applied by NEN-081 — this task
/// stops at "the medium opens the way `⌘O` opens it". The unit is
/// milliseconds because that is what ADR-0042's `deferredSeekMs` contract
/// speaks; seconds only exist at the boundary, where senders write them.
#[derive(Clone, PartialEq, Eq)]
pub struct HandoffRequest {
    pub locator: HandoffLocator,
    pub start_position_ms: Option<u64>,
}

impl fmt::Debug for HandoffRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandoffRequest")
            .field("locator", &self.locator)
            .field("start_position_ms", &self.start_position_ms)
            .finish()
    }
}

/// Why nothing was opened.
///
/// Carries no payload on purpose, exactly like
/// [`RemoteEvidenceError`](crate::remote_evidence::RemoteEvidenceError): the
/// variant is the whole message, so a shell can show it and a log can print it
/// without either of them having to remember what is secret.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandoffRejection {
    /// The launch carried no positional locator at all — an ordinary start,
    /// not a handoff.
    NoLocator,
    /// A well-formed URI naming a scheme this player does not open
    /// (`ftp:`, `javascript:`, …).
    UnsupportedScheme,
    /// Locator-shaped input that does not resolve: an empty string, control
    /// characters, a relative path, a `file://` URL with a foreign authority.
    MalformedLocator,
}

impl HandoffRejection {
    /// Stable lowercase name. Carries no input, so it is safe to log.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoLocator => "no-locator",
            Self::UnsupportedScheme => "unsupported-scheme",
            Self::MalformedLocator => "malformed-locator",
        }
    }
}

impl fmt::Display for HandoffRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NoLocator => "no medium was handed over",
            Self::UnsupportedScheme => "unsupported media scheme",
            Self::MalformedLocator => "malformed media locator",
        })
    }
}

impl std::error::Error for HandoffRejection {}

/// Reads a launch's arguments the way NEN-078 measured a sender writing them.
///
/// `argv[0]` is the executable and is skipped. The first positional argument
/// is the locator; a later one is ignored, because a player opens one medium.
///
/// **An unrecognized flag is dropped, never consumed.** Bulgu 4 measured
/// `--no-terminal` and `--no-video-title-show` — boolean flags that take no
/// value. Swallowing the token after an unknown flag would let a sender's
/// harmless switch eat the locator standing behind it, which is the one thing
/// this function must not lose.
pub fn parse_argv(argv: &[String]) -> Result<HandoffRequest, HandoffRejection> {
    let mut locator: Option<&str> = None;
    let mut start_position_ms = None;
    let mut index = 1;

    while index < argv.len() {
        let argument = argv[index].as_str();
        index += 1;

        if !argument.starts_with('-') {
            if locator.is_none() {
                locator = Some(argument);
            }
            continue;
        }

        let (name, inline_value) = match argument.split_once('=') {
            Some((name, value)) => (name, Some(value)),
            None => (argument, None),
        };
        if !START_FLAGS.contains(&name) {
            continue;
        }
        let value = match inline_value {
            Some(value) => Some(value),
            None => {
                // The separate-token spelling: `--start 42`. Consuming it is
                // what keeps `42` from being mistaken for the locator, and is
                // safe here precisely because the name is recognized.
                let next = argv.get(index).map(String::as_str);
                if next.is_some() {
                    index += 1;
                }
                next
            }
        };
        // A last spelling wins, and an unreadable value simply leaves the
        // position unset (ADR-0043 Karar 2: it falls silently, the medium
        // still opens).
        start_position_ms = value.and_then(parse_seconds_as_millis);
    }

    let locator = locator.ok_or(HandoffRejection::NoLocator)?;
    Ok(HandoffRequest {
        locator: resolve_locator(locator)?,
        start_position_ms,
    })
}

/// Reads what LaunchServices delivers: an opened document's `file://` URL, or
/// a `nenplayer://` string a sender built.
///
/// Both arrive through the same AppKit callback, so both are answered here.
/// The scheme form wraps an untouched media URL — `nenplayer://https://host/a.mkv`
/// — and may end in `#t=<seconds>`.
pub fn parse_url(raw: &str) -> Result<HandoffRequest, HandoffRejection> {
    let trimmed = raw.trim();
    if trimmed.len() < SCHEME_PREFIX.len()
        || !trimmed[..SCHEME_PREFIX.len()].eq_ignore_ascii_case(SCHEME_PREFIX)
    {
        // Not the custom scheme: a document URL, judged on its own terms.
        return Ok(HandoffRequest {
            locator: resolve_locator(trimmed)?,
            start_position_ms: None,
        });
    }

    let inner = &trimmed[SCHEME_PREFIX.len()..];
    let (inner, start_position_ms) = match inner.rfind(TIME_FRAGMENT) {
        Some(at) => {
            let value = &inner[at + TIME_FRAGMENT.len()..];
            match parse_seconds_as_millis(value) {
                // Only a fragment that reads as a time is treated as one; an
                // unreadable `#t=` stays part of the URL rather than being
                // quietly cut off it.
                Some(millis) => (&inner[..at], Some(millis)),
                None => (inner, None),
            }
        }
        None => (inner, None),
    };

    Ok(HandoffRequest {
        locator: resolve_locator(inner)?,
        start_position_ms,
    })
}

/// The gate every locator passes, whichever surface carried it.
fn resolve_locator(raw: &str) -> Result<HandoffLocator, HandoffRejection> {
    let raw = raw.trim();
    if raw.is_empty() || raw.chars().any(char::is_control) {
        return Err(HandoffRejection::MalformedLocator);
    }

    if let Some(rest) = strip_scheme_ignoring_case(raw, "file://") {
        return local_path_from_file_url(rest);
    }
    if raw.starts_with('/') {
        // A bare absolute path. Checked before the scheme shape below, because
        // a POSIX filename is allowed to contain a colon.
        return Ok(HandoffLocator::LocalPath(PathBuf::from(raw)));
    }
    match scheme_of(raw) {
        Some(scheme)
            if scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https") =>
        {
            // NEN-036's gate, not a second copy of it: control characters,
            // whitespace, a missing authority and a backslash in it are all
            // already refused there.
            validate_url(raw).map_err(|_| HandoffRejection::MalformedLocator)?;
            Ok(HandoffLocator::Remote(raw.to_owned()))
        }
        Some(_) => Err(HandoffRejection::UnsupportedScheme),
        // A relative path. The core has no working directory, and an `.app`
        // launched by LaunchServices inherits `/` as one — resolving it here
        // would invent a medium rather than open the one that was handed over.
        None => Err(HandoffRejection::MalformedLocator),
    }
}

/// `file:///Users/x/a.mkv` → `/Users/x/a.mkv`.
///
/// The authority must be empty or `localhost`: a `file://` URL naming another
/// host is not something this machine can open, and treating it as a local
/// path would silently answer a different question.
fn local_path_from_file_url(rest: &str) -> Result<HandoffLocator, HandoffRejection> {
    let authority_end = rest.find('/').ok_or(HandoffRejection::MalformedLocator)?;
    let authority = &rest[..authority_end];
    if !authority.is_empty() && !authority.eq_ignore_ascii_case("localhost") {
        return Err(HandoffRejection::MalformedLocator);
    }
    let encoded = &rest[authority_end..];
    let decoded = percent_decode(encoded).ok_or(HandoffRejection::MalformedLocator)?;
    let path = Path::new(&decoded);
    if !path.is_absolute() {
        return Err(HandoffRejection::MalformedLocator);
    }
    Ok(HandoffLocator::LocalPath(path.to_path_buf()))
}

/// The scheme of a URI, if it has one shaped like RFC 3986's.
///
/// Deliberately does not require `://`: `javascript:alert(1)` has a scheme and
/// must be refused as one rather than falling through to "relative path".
fn scheme_of(raw: &str) -> Option<&str> {
    let end = raw.find(':')?;
    let scheme = &raw[..end];
    let mut characters = scheme.chars();
    if !characters.next()?.is_ascii_alphabetic() {
        return None;
    }
    if !characters
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.'))
    {
        return None;
    }
    Some(scheme)
}

fn strip_scheme_ignoring_case<'a>(raw: &'a str, prefix: &str) -> Option<&'a str> {
    if raw.len() >= prefix.len() && raw[..prefix.len()].eq_ignore_ascii_case(prefix) {
        Some(&raw[prefix.len()..])
    } else {
        None
    }
}

/// Percent-decoding, refusing anything it cannot decode.
///
/// A truncated or non-hex escape is not repaired and not passed through: this
/// is untrusted text on its way to becoming a filesystem path, and guessing
/// what a malformed escape meant is how a path stops being the one that was
/// handed over. The result must still be valid UTF-8.
fn percent_decode(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = hex_value(*bytes.get(index + 1)?)?;
            let low = hex_value(*bytes.get(index + 2)?)?;
            decoded.push(high * 16 + low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Seconds at the boundary, milliseconds inside (ADR-0043 Karar 2).
///
/// Written by hand rather than through `f64` so the accepted set is exactly
/// what a sender writes: digits, optionally a decimal point and more digits.
/// A float parse would also swallow `1e9`, `inf` and `NaN`, and would round
/// the value on its way in — none of which any measured sender produces, and
/// all of which would be a surprise at the far end.
///
/// Anything else is `None`, and `None` means the medium simply starts from the
/// beginning: an unreadable position is never an error the user is shown.
fn parse_seconds_as_millis(raw: &str) -> Option<u64> {
    let raw = raw.trim();
    let (whole, fraction) = match raw.split_once('.') {
        Some((whole, fraction)) => (whole, fraction),
        None => (raw, ""),
    };
    if whole.is_empty() || !whole.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let seconds: u64 = whole.parse().ok()?;
    // Truncated to milliseconds, not rounded: a sender writing more precision
    // than the contract carries is asking for a moment this player cannot name.
    let mut millis = 0;
    for position in 0..3 {
        let digit = fraction
            .as_bytes()
            .get(position)
            .map_or(0, |byte| u64::from(byte - b'0'));
        millis = millis * 10 + digit;
    }
    seconds.checked_mul(1000)?.checked_add(millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(arguments: &[&str]) -> Vec<String> {
        std::iter::once("/Applications/Nen Player.app/Contents/MacOS/NenPlayer")
            .chain(arguments.iter().copied())
            .map(str::to_owned)
            .collect()
    }

    fn local(request: &HandoffRequest) -> &Path {
        match &request.locator {
            HandoffLocator::LocalPath(path) => path,
            HandoffLocator::Remote(url) => panic!("expected a local path, got remote {url}"),
        }
    }

    fn remote(request: &HandoffRequest) -> &str {
        match &request.locator {
            HandoffLocator::Remote(url) => url,
            HandoffLocator::LocalPath(path) => {
                panic!("expected a remote URL, got local {}", path.display())
            }
        }
    }

    #[test]
    fn a_positional_absolute_path_is_the_locator() {
        let request = parse_argv(&argv(&["/Users/x/Film.mkv"])).expect("request");
        assert_eq!(local(&request), Path::new("/Users/x/Film.mkv"));
        assert_eq!(request.start_position_ms, None);
    }

    #[test]
    fn a_positional_https_url_is_the_locator() {
        let request = parse_argv(&argv(&["https://example.test/a/Film.mkv"])).expect("request");
        assert_eq!(remote(&request), "https://example.test/a/Film.mkv");
    }

    #[test]
    fn plain_http_is_opened_too() {
        // `security-policy.md` §3: provider APIs are HTTPS-only, user and
        // handoff media URLs are not.
        let request = parse_argv(&argv(&["http://example.test/Film.mkv"])).expect("request");
        assert_eq!(remote(&request), "http://example.test/Film.mkv");
    }

    #[test]
    fn a_launch_without_a_positional_is_not_a_handoff() {
        assert_eq!(parse_argv(&argv(&[])), Err(HandoffRejection::NoLocator));
        assert_eq!(
            parse_argv(&argv(&["--no-terminal"])),
            Err(HandoffRejection::NoLocator)
        );
    }

    #[test]
    fn an_unsupported_scheme_is_refused_by_name() {
        for locator in ["ftp://example.test/a.mkv", "javascript:alert(1)", "data:,x"] {
            assert_eq!(
                parse_argv(&argv(&[locator])),
                Err(HandoffRejection::UnsupportedScheme),
                "{locator}"
            );
        }
    }

    #[test]
    fn a_malformed_locator_is_refused_without_panicking() {
        for locator in ["Film.mkv", "../Film.mkv", "https://", "https://a b/c.mkv"] {
            assert_eq!(
                parse_argv(&argv(&[locator])),
                Err(HandoffRejection::MalformedLocator),
                "{locator}"
            );
        }
    }

    #[test]
    fn every_start_flag_spelling_means_the_same_thing() {
        for name in START_FLAGS {
            let inline = parse_argv(&argv(&[&format!("{name}=42"), "/Users/x/Film.mkv"]))
                .expect("inline value");
            let separate =
                parse_argv(&argv(&[name, "42", "/Users/x/Film.mkv"])).expect("separate value");
            assert_eq!(inline.start_position_ms, Some(42_000), "{name}");
            assert_eq!(separate, inline, "{name}");
        }
    }

    #[test]
    fn a_separate_flag_value_is_never_mistaken_for_the_locator() {
        // The reason NEN-080 has to know the flag names at all, even though
        // NEN-081 is what applies the position.
        let request = parse_argv(&argv(&["--start", "42", "/Users/x/Film.mkv"])).expect("request");
        assert_eq!(local(&request), Path::new("/Users/x/Film.mkv"));
    }

    #[test]
    fn an_unrecognized_flag_is_dropped_and_never_eats_the_locator() {
        // Exactly what NEN-078 Bulgu 4 measured Stremio sending.
        let request = parse_argv(&argv(&[
            "--no-terminal",
            "--no-video-title-show",
            "/Users/x/Film.mkv",
        ]))
        .expect("request");
        assert_eq!(local(&request), Path::new("/Users/x/Film.mkv"));
    }

    #[test]
    fn a_second_positional_is_ignored() {
        let request =
            parse_argv(&argv(&["/Users/x/First.mkv", "/Users/x/Second.mkv"])).expect("request");
        assert_eq!(local(&request), Path::new("/Users/x/First.mkv"));
    }

    #[test]
    fn an_unreadable_position_falls_silently() {
        for value in ["abc", "-5", "", "1e9", "NaN", "4,2"] {
            let request = parse_argv(&argv(&[&format!("--start={value}"), "/Users/x/Film.mkv"]))
                .expect("the medium still opens");
            assert_eq!(request.start_position_ms, None, "{value}");
        }
    }

    #[test]
    fn a_fractional_position_is_truncated_to_milliseconds() {
        let request =
            parse_argv(&argv(&["--start=12.3456", "/Users/x/Film.mkv"])).expect("request");
        assert_eq!(request.start_position_ms, Some(12_345));
    }

    #[test]
    fn a_zero_position_is_carried_as_zero() {
        // ADR-0043 Karar 2: `0` and "no position" mean the same thing, so no
        // branch turns one into the other — NEN-078 Bulgu 5 measured every
        // real Stremio launch arriving as `0`.
        let request = parse_argv(&argv(&["--start=0", "/Users/x/Film.mkv"])).expect("request");
        assert_eq!(request.start_position_ms, Some(0));
    }

    #[test]
    fn an_overflowing_position_falls_rather_than_wrapping() {
        let request = parse_argv(&argv(&[
            &format!("--start={}", u64::MAX),
            "/Users/x/Film.mkv",
        ]))
        .expect("request");
        assert_eq!(request.start_position_ms, None);
    }

    #[test]
    fn an_opened_document_arrives_as_a_file_url() {
        let request = parse_url("file:///Users/x/Film%20Gecesi.mkv").expect("request");
        assert_eq!(local(&request), Path::new("/Users/x/Film Gecesi.mkv"));
        assert_eq!(request.start_position_ms, None);
    }

    #[test]
    fn a_file_url_naming_another_host_is_refused() {
        assert_eq!(
            parse_url("file://otherhost/Users/x/Film.mkv"),
            Err(HandoffRejection::MalformedLocator)
        );
        assert_eq!(
            parse_url("file://localhost/Users/x/Film.mkv").map(|request| request.locator
                == HandoffLocator::LocalPath(PathBuf::from("/Users/x/Film.mkv"))),
            Ok(true)
        );
    }

    #[test]
    fn a_broken_percent_escape_is_refused_rather_than_repaired() {
        for locator in ["file:///Users/x/Film%2.mkv", "file:///Users/x/Film%zz.mkv"] {
            assert_eq!(
                parse_url(locator),
                Err(HandoffRejection::MalformedLocator),
                "{locator}"
            );
        }
    }

    #[test]
    fn the_scheme_carries_the_media_url_untouched() {
        let request = parse_url("nenplayer://https://example.test/a/Film.mkv").expect("request");
        assert_eq!(remote(&request), "https://example.test/a/Film.mkv");
        assert_eq!(request.start_position_ms, None);
    }

    #[test]
    fn the_scheme_reads_a_time_fragment() {
        let request =
            parse_url("nenplayer://https://example.test/a/Film.mkv#t=90.5").expect("request");
        assert_eq!(remote(&request), "https://example.test/a/Film.mkv");
        assert_eq!(request.start_position_ms, Some(90_500));
    }

    #[test]
    fn an_unreadable_time_fragment_stays_part_of_the_url() {
        // Cutting it off would hand the engine a different URL than the sender
        // wrote; leaving it there lets the URL be judged as what it is.
        assert_eq!(
            parse_url("nenplayer://https://example.test/a.mkv#t=later"),
            Ok(HandoffRequest {
                locator: HandoffLocator::Remote("https://example.test/a.mkv#t=later".to_owned()),
                start_position_ms: None,
            })
        );
    }

    #[test]
    fn the_scheme_also_carries_a_local_path() {
        let request = parse_url("nenplayer://file:///Users/x/Film.mkv#t=8").expect("request");
        assert_eq!(local(&request), Path::new("/Users/x/Film.mkv"));
        assert_eq!(request.start_position_ms, Some(8_000));
    }

    #[test]
    fn the_scheme_refuses_what_argv_refuses() {
        assert_eq!(
            parse_url("nenplayer://ftp://example.test/a.mkv"),
            Err(HandoffRejection::UnsupportedScheme)
        );
        assert_eq!(
            parse_url("nenplayer://"),
            Err(HandoffRejection::MalformedLocator)
        );
    }

    #[test]
    fn schemes_are_matched_without_regard_to_case() {
        assert!(parse_url("NENPLAYER://HTTPS://example.test/a.mkv").is_ok());
        assert!(parse_url("FILE:///Users/x/Film.mkv").is_ok());
    }

    #[test]
    fn a_colon_in_a_filename_does_not_look_like_a_scheme() {
        let request = parse_argv(&argv(&["/Users/x/S01:E02.mkv"])).expect("request");
        assert_eq!(local(&request), Path::new("/Users/x/S01:E02.mkv"));
    }

    #[test]
    fn a_rejection_names_itself_without_naming_the_input() {
        assert_eq!(HandoffRejection::NoLocator.as_str(), "no-locator");
        assert_eq!(
            HandoffRejection::UnsupportedScheme.as_str(),
            "unsupported-scheme"
        );
        assert_eq!(
            HandoffRejection::MalformedLocator.as_str(),
            "malformed-locator"
        );
    }
}
