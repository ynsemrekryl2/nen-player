//! Negative tests for encoding detection and sanitization (NEN-015 DoD
//! #2–#4). Mandatory per `docs/testing-strategy.md` — this is a
//! security/validation task, not just a format task.

mod support;

use nen_subtitle::encoding::{self, DetectedEncoding, EncodingError};

/// Text that must never reach an error's output, mirroring
/// `guard_error_debug.rs`'s approach for `SrtError`.
const SENTINEL: &str = "CONFIDENTIAL-SUBTITLE-9130-do-not-leak-me";

#[test]
fn oversized_input_is_rejected_before_any_decode_attempt() {
    let bytes = vec![b'a'; encoding::MAX_INPUT_BYTES + 1];
    let err = encoding::decode(&bytes).expect_err("input over the size limit must be rejected");
    assert_eq!(err.variant_name(), "TooLarge");
}

#[test]
fn input_at_exactly_the_limit_is_still_accepted() {
    let bytes = vec![b'a'; encoding::MAX_INPUT_BYTES];
    assert!(encoding::decode(&bytes).is_ok());
}

#[test]
fn undecodable_bytes_are_rejected_not_silently_mangled() {
    let path = support::corpus_dir("encodings").join("undecodable-garbage.srt");
    let bytes = support::read_bytes(&path);

    let err = encoding::decode(&bytes)
        .expect_err("a lone UTF-16 surrogate must be rejected, not mojibake'd");
    assert_eq!(
        err,
        EncodingError::UndecodableBytes {
            declared: DetectedEncoding::Utf16Le
        }
    );
}

#[test]
fn sanitize_removes_bidi_override_characters() {
    for c in ['\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{202E}'] {
        let input = format!("before{c}{SENTINEL}{c}after");
        let cleaned = encoding::sanitize(&input);
        assert_eq!(
            cleaned,
            format!("before{SENTINEL}after"),
            "char {c:?} was not stripped"
        );
    }
    for c in ['\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}'] {
        let input = format!("before{c}after");
        assert_eq!(
            encoding::sanitize(&input),
            "beforeafter",
            "char {c:?} was not stripped"
        );
    }
}

#[test]
fn sanitize_removes_zero_width_characters() {
    for c in ['\u{200B}', '\u{200C}', '\u{200D}', '\u{2060}', '\u{FEFF}'] {
        let input = format!("before{c}after");
        assert_eq!(
            encoding::sanitize(&input),
            "beforeafter",
            "char {c:?} was not stripped"
        );
    }
}

#[test]
fn sanitize_removes_control_characters_except_newline_and_carriage_return() {
    let mut input = String::from("a");
    for byte in 0x00u8..=0x1F {
        if byte == b'\n' || byte == b'\r' {
            continue; // kept — srt.rs's line splitting depends on them
        }
        input.push(byte as char);
    }
    input.push('\x7F');
    for byte in 0x80u8..=0x9F {
        input.push(byte as char);
    }
    input.push('b');

    assert_eq!(encoding::sanitize(&input), "ab");
    assert_eq!(encoding::sanitize("a\nb\rc"), "a\nb\rc");
}

/// Neither error variant may carry the raw bytes or decoded text that
/// triggered it — same discipline `guard_error_debug.rs` proves for
/// `SrtError`. Both current variants are structurally incapable of it
/// (`TooLarge` carries only a size class, `UndecodableBytes` only an
/// encoding label), but this pins that as a tested property rather than an
/// accident of the current field list.
#[test]
fn no_error_variant_leaks_input_through_debug_or_display() {
    let big = vec![b'a'; encoding::MAX_INPUT_BYTES + 1];
    let too_large = encoding::decode(&big).unwrap_err();

    let path = support::corpus_dir("encodings").join("undecodable-garbage.srt");
    let bytes = support::read_bytes(&path);
    let undecodable = encoding::decode(&bytes).unwrap_err();

    for error in [too_large, undecodable] {
        let debug = format!("{error:?}");
        let display = format!("{error}");
        assert!(!debug.contains(SENTINEL), "Debug leaked input — {debug}");
        assert!(
            !display.contains(SENTINEL),
            "Display leaked input — {display}"
        );
    }
}
