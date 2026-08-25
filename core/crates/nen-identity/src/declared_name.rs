//! The name a server declares for a file (ADR-0009 Karar 6, layer 4).
//!
//! For remote media the filesystem has no name to offer, but the server often
//! does: `Content-Disposition: attachment; filename="..."` (RFC 6266). Debrid
//! and CDN endpoints that hand out opaque URLs routinely still declare the
//! real release name there, which is why ADR-0009 ranks this alongside the
//! local filename — both are a *declaration* of what the file is called,
//! rather than something we inferred from a URL.
//!
//! # Hostile by default
//!
//! The value is attacker-controlled: `docs/security-policy.md` §2 lists
//! filenames as untrusted input, and a header can carry path separators,
//! `..`, control characters, bidi overrides, or an RFC 5987 `filename*`
//! encoding that hides any of those behind percent-escapes. Everything here
//! reduces the value to **one flat segment** — it can never become a path.
//!
//! # No I/O
//!
//! Fetching the header is NEN-036's job (ADR-0009 Karar 2). This module only
//! parses and sanitizes the string it is handed.

/// Longest declared name we will keep.
const MAX_NAME_BYTES: usize = 512;

/// Extracts a filename from a `Content-Disposition` header value.
///
/// RFC 5987's `filename*` form wins when present — it is the one that can
/// carry non-ASCII, so a server that sends both means the plain `filename` as
/// the degraded fallback. The result is always passed through [`sanitize`].
pub fn from_content_disposition(header_value: &str) -> Option<String> {
    let mut plain = None;

    for parameter in header_value.split(';') {
        let parameter = parameter.trim();
        let Some((key, value)) = parameter.split_once('=') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();

        if key == "filename*" {
            if let Some(decoded) = decode_ext_value(value) {
                if let Some(clean) = sanitize(&decoded) {
                    return Some(clean);
                }
            }
        } else if key == "filename" && plain.is_none() {
            plain = Some(unquote(value).to_string());
        }
    }

    plain.as_deref().and_then(sanitize)
}

/// Reduces an arbitrary declared name to a single safe path segment.
///
/// Returns `None` when nothing usable survives — an empty name, a name that
/// was only separators, or one that reduced to `.`/`..`.
pub fn sanitize(raw: &str) -> Option<String> {
    // Take the last segment first: a value of `../../etc/passwd` contributes
    // `passwd` and nothing else. Both separators are treated as separators
    // regardless of host platform, since the value came from the network.
    let last = raw.rsplit(['/', '\\']).next().unwrap_or(raw);

    let cleaned: String = last
        .chars()
        .filter(|c| !c.is_control())
        .filter(|c| !is_bidi_or_zero_width(*c))
        .collect();

    let trimmed = cleaned.trim().trim_matches('.');
    if trimmed.is_empty() {
        return None;
    }

    let bounded = if trimmed.len() > MAX_NAME_BYTES {
        let mut end = MAX_NAME_BYTES;
        while end > 0 && !trimmed.is_char_boundary(end) {
            end -= 1;
        }
        trimmed.get(..end).unwrap_or("")
    } else {
        trimmed
    };

    if bounded.is_empty() {
        None
    } else {
        Some(bounded.to_string())
    }
}

/// Decodes RFC 5987's `charset'language'percent-encoded-value`.
///
/// Only UTF-8 and ISO-8859-1 are recognized, per the RFC; anything else is
/// refused rather than guessed at.
fn decode_ext_value(value: &str) -> Option<String> {
    let value = unquote(value);
    let mut parts = value.splitn(3, '\'');
    let charset = parts.next()?.to_ascii_lowercase();
    let _language = parts.next()?;
    let encoded = parts.next()?;

    if charset != "utf-8" && charset != "iso-8859-1" {
        return None;
    }

    let mut bytes = Vec::with_capacity(encoded.len());
    let mut rest = encoded.as_bytes();
    while let Some((&first, tail)) = rest.split_first() {
        if first == b'%' {
            let hex = tail
                .get(..2)
                .and_then(|pair| std::str::from_utf8(pair).ok());
            if let Some(decoded) = hex.and_then(|hex| u8::from_str_radix(hex, 16).ok()) {
                bytes.push(decoded);
                rest = tail.get(2..).unwrap_or(&[]);
                continue;
            }
        }
        bytes.push(first);
        rest = tail;
    }

    if charset == "iso-8859-1" {
        Some(bytes.iter().map(|&b| b as char).collect())
    } else {
        Some(String::from_utf8_lossy(&bytes).into_owned())
    }
}

fn unquote(value: &str) -> &str {
    let value = value.trim();
    value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(value)
}

/// Bidi overrides ("Trojan Source") and zero-width characters, matching the
/// set `nen_subtitle::encoding` strips for the same reason (ADR-0008).
fn is_bidi_or_zero_width(c: char) -> bool {
    matches!(c,
        '\u{202A}'..='\u{202E}'
            | '\u{2066}'..='\u{2069}'
            | '\u{200B}'..='\u{200D}'
            | '\u{2060}'
            | '\u{FEFF}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quoted_filename_is_extracted() {
        let name = from_content_disposition(r#"attachment; filename="Inception.2010.1080p.mkv""#);
        assert_eq!(name.as_deref(), Some("Inception.2010.1080p.mkv"));
    }

    #[test]
    fn an_unquoted_filename_is_extracted() {
        let name = from_content_disposition("attachment; filename=Movie.2010.mkv");
        assert_eq!(name.as_deref(), Some("Movie.2010.mkv"));
    }

    #[test]
    fn the_extended_form_wins_over_the_plain_one() {
        let header = "attachment; filename=\"fallback.mkv\"; \
                      filename*=UTF-8''Am%C3%A9lie.2001.1080p.mkv";
        assert_eq!(
            from_content_disposition(header).as_deref(),
            Some("Amélie.2001.1080p.mkv")
        );
    }

    #[test]
    fn an_unknown_charset_falls_back_rather_than_guessing() {
        let header = "attachment; filename=\"plain.mkv\"; filename*=SHIFT_JIS''%82%A0.mkv";
        assert_eq!(
            from_content_disposition(header).as_deref(),
            Some("plain.mkv")
        );
    }

    #[test]
    fn a_header_without_a_filename_yields_nothing() {
        assert_eq!(from_content_disposition("inline"), None);
        assert_eq!(from_content_disposition(""), None);
    }

    #[test]
    fn path_traversal_is_reduced_to_one_segment() {
        assert_eq!(sanitize("../../etc/passwd").as_deref(), Some("passwd"));
        assert_eq!(
            sanitize(r"..\..\windows\system32").as_deref(),
            Some("system32")
        );
        assert_eq!(
            sanitize("/absolute/path/Movie.mkv").as_deref(),
            Some("Movie.mkv")
        );
    }

    #[test]
    fn a_name_that_is_only_dots_is_refused() {
        assert_eq!(sanitize(".."), None);
        assert_eq!(sanitize("."), None);
        assert_eq!(sanitize("../.."), None);
        assert_eq!(sanitize(""), None);
        assert_eq!(sanitize("   "), None);
        assert_eq!(sanitize("///"), None);
    }

    #[test]
    fn traversal_hidden_in_an_extended_value_is_still_reduced() {
        let header = "attachment; filename*=UTF-8''%2e%2e%2f%2e%2e%2fpasswd";
        assert_eq!(from_content_disposition(header).as_deref(), Some("passwd"));
    }

    #[test]
    fn control_and_bidi_characters_are_stripped() {
        assert_eq!(
            sanitize("Movie\u{0007}\u{202E}gnp.2010.mkv").as_deref(),
            Some("Moviegnp.2010.mkv")
        );
        assert_eq!(sanitize("a\u{200B}b").as_deref(), Some("ab"));
        assert_eq!(sanitize("Movie\r\n.mkv").as_deref(), Some("Movie.mkv"));
    }

    #[test]
    fn a_newline_cannot_inject_a_second_header() {
        let name = from_content_disposition("attachment; filename=\"a.mkv\r\nX-Evil: 1\"");
        assert_eq!(name.as_deref(), Some("a.mkvX-Evil: 1"));
        assert!(!name.unwrap().contains('\n'));
    }

    #[test]
    fn the_name_is_length_bounded() {
        let long = format!("{}.mkv", "a".repeat(10_000));
        let name = sanitize(&long).unwrap();
        assert!(name.len() <= MAX_NAME_BYTES);
    }

    #[test]
    fn truncation_respects_char_boundaries() {
        let long = "é".repeat(10_000);
        let name = sanitize(&long).unwrap();
        assert!(name.len() <= MAX_NAME_BYTES);
        assert!(name.chars().all(|c| c == 'é'));
    }

    #[test]
    fn hostile_input_never_panics() {
        for value in [
            "filename*=UTF-8''%",
            "filename*=''",
            "filename*=UTF-8'",
            "filename=",
            ";;;;;",
            "filename=\"\u{202e}\u{202e}\u{202e}\"",
            "filename*=UTF-8''%ff%fe%00",
        ] {
            let _ = from_content_disposition(value);
        }
    }
}
