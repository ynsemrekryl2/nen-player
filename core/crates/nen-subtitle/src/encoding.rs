//! Encoding detection and sanitization (NEN-015, ADR-0008).
//!
//! [`srt::parse`](crate::srt::parse) takes `&str` and is deliberately
//! encoding-agnostic — this module is the layer that runs *before* it,
//! turning untrusted bytes of unknown encoding into the clean, BOM-free
//! UTF-8 text the parser assumes:
//!
//! ```text
//! let text = encoding::decode(&bytes)?;
//! let document = srt::parse(&text)?;
//! ```
//!
//! Detection order: a byte-order mark (UTF-8, UTF-16LE, UTF-16BE) wins if
//! present; otherwise the bytes are tried as strict UTF-8, and only if that
//! fails does decoding fall back to Windows-1254 — the one legacy code page
//! this module supports (ADR-0008 explains why not Windows-1252 too: without
//! a BOM the two are indistinguishable outside six Turkish-specific byte
//! values, and guessing between them is language detection, which is
//! NEN-020's job, not this one).
//!
//! After decoding, [`sanitize`] strips control characters (other than `\n`
//! and `\r`, which the SRT format needs), bidi override/embedding/isolate
//! characters, and zero-width characters. These are removed rather than
//! rejected: the rest of a file is still legitimate even if one character
//! in it is not.

use std::fmt;

use encoding_rs::Encoding;
use nen_domain::redact::size_class;

/// Raw input larger than this is rejected before any decode attempt.
/// Real subtitle files run tens to a few hundred KiB; this bounds the
/// worst-case cost of decoding a hostile input while leaving generous
/// headroom for legitimate ones.
pub const MAX_INPUT_BYTES: usize = 10 * 1024 * 1024;

/// The encoding a decode attempt was made under, for [`EncodingError`].
/// Carries no content — only which of the four supported encodings applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedEncoding {
    Utf8Bom,
    Utf16Le,
    Utf16Be,
    Windows1254,
}

impl DetectedEncoding {
    pub const fn variant_name(&self) -> &'static str {
        match self {
            Self::Utf8Bom => "Utf8Bom",
            Self::Utf16Le => "Utf16Le",
            Self::Utf16Be => "Utf16Be",
            Self::Windows1254 => "Windows1254",
        }
    }
}

impl fmt::Display for DetectedEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.variant_name())
    }
}

/// Everything [`decode`] can refuse, and why.
///
/// **No variant carries the input bytes or any decoded text.** `size_class`
/// is what `docs/security-policy.md` §1 lists as loggable ("boyut sınıfı");
/// `declared` names an encoding, never content. `#[derive(Debug)]` is
/// therefore safe here, on the same basis `srt::SrtError` documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingError {
    /// Raw input exceeded [`MAX_INPUT_BYTES`]; not decoded at all.
    TooLarge { size_class: &'static str },
    /// The bytes are not valid under the encoding that was determined to
    /// apply (by BOM, or by legacy fallback when no BOM was present).
    UndecodableBytes { declared: DetectedEncoding },
}

impl EncodingError {
    pub const fn variant_name(&self) -> &'static str {
        match self {
            Self::TooLarge { .. } => "TooLarge",
            Self::UndecodableBytes { .. } => "UndecodableBytes",
        }
    }
}

impl fmt::Display for EncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge { size_class } => {
                write!(
                    f,
                    "input too large ({size_class}), exceeds {MAX_INPUT_BYTES} bytes"
                )
            }
            Self::UndecodableBytes { declared } => {
                write!(f, "input is not valid {declared}")
            }
        }
    }
}

impl std::error::Error for EncodingError {}

/// Byte-order marks this module recognizes, longest first so a UTF-8 BOM
/// (`EF BB BF`) is never mistaken for a UTF-16 one sharing its first byte.
const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];
const UTF16LE_BOM: [u8; 2] = [0xFF, 0xFE];
const UTF16BE_BOM: [u8; 2] = [0xFE, 0xFF];

/// Decodes raw subtitle bytes of unknown encoding into clean, sanitized
/// UTF-8 text. See the module docs for the detection order and
/// [`sanitize`] for what gets stripped.
pub fn decode(bytes: &[u8]) -> Result<String, EncodingError> {
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(EncodingError::TooLarge {
            size_class: size_class(bytes.len() as u64),
        });
    }

    let text = if bytes.starts_with(&UTF8_BOM) {
        decode_with(
            encoding_rs::UTF_8,
            bytes.split_at(UTF8_BOM.len()).1,
            DetectedEncoding::Utf8Bom,
        )?
    } else if bytes.starts_with(&UTF16LE_BOM) {
        decode_with(
            encoding_rs::UTF_16LE,
            bytes.split_at(UTF16LE_BOM.len()).1,
            DetectedEncoding::Utf16Le,
        )?
    } else if bytes.starts_with(&UTF16BE_BOM) {
        decode_with(
            encoding_rs::UTF_16BE,
            bytes.split_at(UTF16BE_BOM.len()).1,
            DetectedEncoding::Utf16Be,
        )?
    } else if let Ok(text) = std::str::from_utf8(bytes) {
        text.to_string()
    } else {
        decode_with(
            encoding_rs::WINDOWS_1254,
            bytes,
            DetectedEncoding::Windows1254,
        )?
    };

    Ok(sanitize(&text))
}

/// Decodes `bytes` under `encoding`, without BOM handling (the caller has
/// already consumed any BOM). Errors if the encoding rejects any part of
/// the input rather than silently substituting a replacement character for
/// it — this module treats that as undecodable, not mojibake.
fn decode_with(
    encoding: &'static Encoding,
    bytes: &[u8],
    declared: DetectedEncoding,
) -> Result<String, EncodingError> {
    let (cow, had_errors) = encoding.decode_without_bom_handling(bytes);
    if had_errors {
        Err(EncodingError::UndecodableBytes { declared })
    } else {
        Ok(cow.into_owned())
    }
}

/// Strips characters that are either useless (most control characters) or
/// actively dangerous (bidi overrides, zero-width characters) in subtitle
/// text, while leaving `\n`/`\r` alone — `srt::parse`'s line splitting
/// depends on them.
pub fn sanitize(text: &str) -> String {
    text.chars().filter(|&c| keep_char(c)).collect()
}

fn keep_char(c: char) -> bool {
    if c == '\n' || c == '\r' {
        return true;
    }
    if c.is_control() {
        return false;
    }
    if is_bidi_control(c) {
        return false;
    }
    if is_zero_width(c) {
        return false;
    }
    true
}

/// Explicit bidirectional override/embedding/isolate characters — the
/// "Trojan Source" class of attack. Legitimate right-to-left text renders
/// correctly from character properties alone via the Unicode bidi
/// algorithm, so these are never needed and are removed outright rather
/// than escaped or neutralized.
///
/// `U+202A..=U+202E`: LRE, RLE, PDF, LRO, RLO.
/// `U+2066..=U+2069`: LRI, RLI, FSI, PDI.
fn is_bidi_control(c: char) -> bool {
    matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

/// Zero-width characters, including a mid-file byte-order mark (a leading
/// one is already consumed at the byte level in [`decode`]; any other
/// occurrence is removed here as defense in depth).
///
/// `U+200B..=U+200D`: ZWSP, ZWNJ, ZWJ. `U+2060`: word joiner. `U+FEFF`: BOM
/// appearing outside position 0.
fn is_zero_width(c: char) -> bool {
    matches!(c, '\u{200B}'..='\u{200D}' | '\u{2060}' | '\u{FEFF}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_plain_utf8_with_no_bom() {
        assert_eq!(decode("hello".as_bytes()).unwrap(), "hello");
    }

    #[test]
    fn strips_utf8_bom() {
        let mut bytes = UTF8_BOM.to_vec();
        bytes.extend_from_slice("hello".as_bytes());
        assert_eq!(decode(&bytes).unwrap(), "hello");
    }

    #[test]
    fn decodes_utf16le_with_bom() {
        let mut bytes = UTF16LE_BOM.to_vec();
        for unit in "hi".encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        assert_eq!(decode(&bytes).unwrap(), "hi");
    }

    #[test]
    fn decodes_utf16be_with_bom() {
        let mut bytes = UTF16BE_BOM.to_vec();
        for unit in "hi".encode_utf16() {
            bytes.extend_from_slice(&unit.to_be_bytes());
        }
        assert_eq!(decode(&bytes).unwrap(), "hi");
    }

    #[test]
    fn falls_back_to_windows_1254_when_not_valid_utf8() {
        // 0xDD is capital dotted I (İ) in Windows-1254; not valid UTF-8 on its own.
        let bytes = [0xDDu8];
        assert_eq!(decode(&bytes).unwrap(), "\u{130}");
    }

    #[test]
    fn rejects_input_over_the_size_limit() {
        let bytes = vec![b'a'; MAX_INPUT_BYTES + 1];
        let err = decode(&bytes).unwrap_err();
        assert_eq!(err.variant_name(), "TooLarge");
    }

    #[test]
    fn rejects_unpaired_utf16_surrogate_rather_than_substituting() {
        let mut bytes = UTF16LE_BOM.to_vec();
        bytes.extend_from_slice(&0xD800u16.to_le_bytes()); // lone high surrogate
        let err = decode(&bytes).unwrap_err();
        assert_eq!(
            err,
            EncodingError::UndecodableBytes {
                declared: DetectedEncoding::Utf16Le
            }
        );
    }

    #[test]
    fn sanitize_strips_bidi_override_but_keeps_the_rest() {
        let input = "safe\u{202E}text";
        assert_eq!(sanitize(input), "safetext");
    }

    #[test]
    fn sanitize_strips_zero_width_characters() {
        let input = "wo\u{200B}rd";
        assert_eq!(sanitize(input), "word");
    }

    #[test]
    fn sanitize_strips_control_characters_but_keeps_newlines() {
        let input = "line1\n\u{0007}line2\r\n";
        assert_eq!(sanitize(input), "line1\nline2\r\n");
    }

    #[test]
    fn sanitize_strips_mid_file_bom() {
        let input = "before\u{FEFF}after";
        assert_eq!(sanitize(input), "beforeafter");
    }
}
