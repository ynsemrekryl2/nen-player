//! Security guard for the parser's error type (K23 #4, `docs/security-policy.md`).
//!
//! `SrtError` is the one thing that crosses out of parsing when input is bad,
//! which makes it the obvious place for cue dialogue to escape into a log.
//! Every variant is therefore constructed from input carrying a distinctive
//! sentinel, and neither `Debug` nor `Display` may reproduce any part of it.
//!
//! The negative test at the bottom proves the check discriminates rather than
//! passing on anything.

mod support;

use nen_subtitle::srt::{self, SrtError};

/// Text that must never reach an error's output. Long and distinctive so a
/// partial leak is caught as surely as a whole one.
const SENTINEL: &str = "CONFIDENTIAL-DIALOGUE-4711-she-said-what-exactly";

/// One malformed input per variant, each carrying the sentinel as its cue
/// text wherever the defect allows text at all.
fn inputs_covering_every_variant() -> Vec<String> {
    let cue = |time: &str| format!("1\n{time}\n{SENTINEL}\n");
    vec![
        String::new(),
        format!("\u{feff}{}", cue("00:00:01,000 --> 00:00:02,000")),
        format!("00:00:01,000 --> 00:00:02,000\n{SENTINEL}\n"),
        format!("one\n00:00:01,000 --> 00:00:02,000\n{SENTINEL}\n"),
        format!("99999999999\n00:00:01,000 --> 00:00:02,000\n{SENTINEL}\n"),
        format!("7\n00:00:01,000 --> 00:00:02,000\n{SENTINEL}\n"),
        format!("1\n\n2\n00:00:01,000 --> 00:00:02,000\n{SENTINEL}\n"),
        cue("00:00:01,000 00:00:02,000"),
        cue("00:00:01,000 -> 00:00:02,000"),
        cue("00:00:01,000 --> 00:00:02,000 X1:0 X2:100"),
        cue("00:00:01.000 --> 00:00:02,000"),
        cue("0a:00:01,000 --> 00:00:02,000"),
        cue("00:75:00,000 --> 00:76:00,000"),
        cue("00:00:02,000 --> 00:00:01,000"),
        cue("00:00:01,000 --> 00:00:01,000"),
        format!(
            "1\n00:00:05,000 --> 00:00:06,000\n{SENTINEL}\n\n\
             2\n00:00:01,000 --> 00:00:02,000\n{SENTINEL}\n"
        ),
        "1\n00:00:01,000 --> 00:00:02,000\n\n2\n00:00:03,000 --> 00:00:04,000\ntext\n".to_string(),
    ]
}

fn errors_covering_every_variant() -> Vec<SrtError> {
    inputs_covering_every_variant()
        .iter()
        .map(|input| {
            srt::parse(input)
                .expect_err("every input in this fixture set is malformed by construction")
        })
        .collect()
}

#[test]
fn the_fixture_set_reaches_every_error_variant() {
    let mut seen: Vec<&str> = errors_covering_every_variant()
        .iter()
        .map(SrtError::variant_name)
        .collect();
    seen.sort_unstable();
    seen.dedup();

    // All 17 variants of SrtError. If a variant is added without a fixture,
    // this fails and the guard below stops silently skipping it.
    let expected = [
        "EmptyInput",
        "EmptyText",
        "EndBeforeStart",
        "IndexOverflow",
        "LeadingBom",
        "MalformedArrow",
        "MalformedTimestamp",
        "MissingArrow",
        "MissingIndexLine",
        "MissingTimeLine",
        "NonMonotonicCue",
        "NonNumericIndex",
        "NonNumericTimestampField",
        "NonSequentialIndex",
        "TimestampFieldOutOfRange",
        "TrailingContentOnTimeLine",
        "ZeroDuration",
    ];
    assert_eq!(seen, expected);
}

#[test]
fn no_error_variant_leaks_cue_text_through_debug_or_display() {
    for error in errors_covering_every_variant() {
        let debug = format!("{error:?}");
        let display = format!("{error}");
        let variant = error.variant_name();

        assert!(
            !debug.contains(SENTINEL),
            "{variant}: Debug leaked cue text — {debug}"
        );
        assert!(
            !display.contains(SENTINEL),
            "{variant}: Display leaked cue text — {display}"
        );
        // Also reject any recognisable fragment, not just the whole string.
        for fragment in ["CONFIDENTIAL", "she-said", "4711"] {
            assert!(
                !debug.contains(fragment) && !display.contains(fragment),
                "{variant}: leaked the fragment {fragment:?}"
            );
        }
    }
}

#[test]
fn no_error_from_the_malformed_corpus_leaks_its_fixture_text() {
    for path in support::srt_files("malformed") {
        let input = support::read(&path);
        let Err(error) = srt::parse(&input) else {
            continue; // covered as a failure by tests/malformed.rs
        };

        let printed = format!("{error:?} {error}");
        for line in input.lines() {
            let text = line.trim();
            // Only the actual dialogue lines matter here; digits and the
            // timestamp punctuation legitimately appear in error payloads.
            if text.len() < 4 || text.chars().any(|c| c.is_ascii_digit()) {
                continue;
            }
            assert!(
                !printed.contains(text),
                "{}: error output contains fixture text {text:?} — {printed}",
                support::name(&path)
            );
        }
    }
}

/// Negative test: a hypothetical error variant that carries the offending
/// text, printed with a derived `Debug`. This is what the checks above are
/// meant to catch — without it, they would pass on any type at all.
#[derive(Debug)]
#[allow(dead_code)] // the derive is exactly how the field leaks below
enum BadErrorWithTextPayload {
    ParseFailed { cue_text: String },
}

#[test]
fn an_error_that_carries_cue_text_is_caught_by_the_same_check() {
    let bad = BadErrorWithTextPayload::ParseFailed {
        cue_text: SENTINEL.to_string(),
    };
    let debug = format!("{bad:?}");

    assert!(
        debug.contains(SENTINEL),
        "expected a text-carrying error variant to leak, proving the guards \
         above are not vacuous; got {debug}"
    );
}
