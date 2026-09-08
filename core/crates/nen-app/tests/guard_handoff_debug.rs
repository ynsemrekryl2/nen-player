//! K23 guard for the handoff intake types (NEN-080, NEN-083, ADR-0043).
//!
//! A handoff locator is exactly what §1 #1/#2/#3 forbid a log from carrying:
//! a media URL (which can hold a token), a private full path, or a query
//! string. `HandoffLocator` and `HandoffRequest` hold one of those directly,
//! so both get the same treatment `subtitle_files` gets in
//! `guard_subtitle_file_debug.rs`: hand-written `Debug`, proved here against
//! derived twins that would leak.

use nen_app::handoff::{HandoffLocator, HandoffRejection, HandoffRequest};
use std::path::PathBuf;

const PRIVATE_PATH: &str = "/Users/gizli-kullanici/Videolar/Sevgilimle.Tatil.2019.mkv";
const PRIVATE_URL: &str = "https://example.test/stream?token=S3CR3T-TOKEN&user=gizli-kullanici";
const FORBIDDEN: [&str; 5] = [
    "gizli-kullanici",
    "S3CR3T-TOKEN",
    "Sevgilimle.Tatil",
    "example.test",
    "token=",
];

fn forbidden(output: &str) {
    for value in FORBIDDEN {
        assert!(
            !output.contains(value),
            "Debug output leaked a forbidden value ({value}): {output}"
        );
    }
}

fn rejection_name(rejection: HandoffRejection) -> &'static str {
    match rejection {
        HandoffRejection::NoLocator => "no-locator",
        HandoffRejection::UnsupportedScheme => "unsupported-scheme",
        HandoffRejection::MalformedLocator => "malformed-locator",
    }
}

#[test]
fn a_local_locator_never_prints_its_path() {
    let locator = HandoffLocator::LocalPath(PathBuf::from(PRIVATE_PATH));
    let output = format!("{locator:?}");
    forbidden(&output);
    assert!(output.contains("local"), "{output}");
    assert!(output.contains("mkv"), "{output}"); // the extension is safe
}

#[test]
fn a_remote_locator_never_prints_its_url() {
    let locator = HandoffLocator::Remote(PRIVATE_URL.to_owned());
    let output = format!("{locator:?}");
    forbidden(&output);
    assert!(output.contains("remote"), "{output}");
    assert!(output.contains("https"), "{output}"); // the scheme is safe
}

#[test]
fn a_parsed_argv_never_prints_its_input() {
    let request = nen_app::handoff::parse_argv(&[
        "NenPlayer".to_owned(),
        "--start=42".to_owned(),
        PRIVATE_URL.to_owned(),
    ])
    .expect("the fixture is a valid remote handoff");
    let output = format!("{request:?}");
    forbidden(&output);
    assert!(output.contains("42000"), "{output}");
}

#[test]
fn a_request_never_prints_through_to_its_locator() {
    let request = HandoffRequest {
        locator: HandoffLocator::LocalPath(PathBuf::from(PRIVATE_PATH)),
        start_position_ms: Some(42_000),
    };
    let output = format!("{request:?}");
    forbidden(&output);
    assert!(output.contains("42000"), "{output}"); // position is not secret
}

#[test]
fn a_rejection_never_carries_the_input_that_caused_it() {
    let rejections = [
        HandoffRejection::NoLocator,
        HandoffRejection::UnsupportedScheme,
        HandoffRejection::MalformedLocator,
    ];
    let output = rejections
        .iter()
        .map(|rejection| format!("{rejection:?} {rejection}"))
        .collect::<Vec<_>>()
        .join(" ");
    forbidden(&output);
    assert!(output.contains("UnsupportedScheme"), "{output}");
    assert!(output.contains("unsupported media scheme"), "{output}");
    for rejection in rejections {
        assert!(!rejection_name(rejection).is_empty());
    }
}

#[test]
fn a_derived_debug_on_these_shapes_would_leak() {
    // The control. If this twin stops leaking, the guard above has stopped
    // proving anything and the fixtures need to be made private again.
    #[derive(Debug)]
    #[allow(dead_code)]
    enum DerivedLocatorTwin {
        LocalPath(PathBuf),
        Remote(String),
    }

    let local = DerivedLocatorTwin::LocalPath(PathBuf::from(PRIVATE_PATH));
    let remote = DerivedLocatorTwin::Remote(PRIVATE_URL.to_owned());
    let printed = format!("{local:?} {remote:?}");
    assert!(printed.contains(PRIVATE_PATH), "{printed}");
    assert!(printed.contains(PRIVATE_URL), "{printed}");
}
