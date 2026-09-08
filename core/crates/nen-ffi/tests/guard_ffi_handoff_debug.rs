//! K23 guard for the handoff gate types (NEN-080, NEN-083, ADR-0043).
//!
//! [`FfiHandoffLocator`] and [`FfiHandoffRequest`] are the one pair of types
//! in this gate that hand a path or a URL back **out** to the caller
//! (`crate::handoff`'s module docs explain why). That makes their `Debug`
//! the only thing standing between a caller that logs what it just received
//! and a violation of `security-policy.md` §1 #1/#3. The twin at the bottom
//! is the control: it shows what a derive would have printed instead.

use nen_ffi::handoff::{FfiHandoffLocator, FfiHandoffRejection, FfiHandoffRequest};

const PRIVATE_PATH: &str = "/Users/gizli-kullanici/Videolar/Sevgilimle.Tatil.2019.mkv";
const PRIVATE_URL: &str = "https://example.test/stream?token=S3CR3T-TOKEN";
const FORBIDDEN: [&str; 4] = ["gizli-kullanici", "S3CR3T-TOKEN", "example.test", "token="];

fn forbidden(output: &str) {
    for value in FORBIDDEN {
        assert!(
            !output.contains(value),
            "Debug output leaked a forbidden value ({value}): {output}"
        );
    }
}

fn rejection_name(rejection: FfiHandoffRejection) -> &'static str {
    match rejection {
        FfiHandoffRejection::NoLocator => "no_locator",
        FfiHandoffRejection::UnsupportedScheme => "unsupported_scheme",
        FfiHandoffRejection::MalformedLocator => "malformed_locator",
    }
}

#[test]
fn a_local_locator_never_prints_its_path_across_the_gate() {
    let locator = FfiHandoffLocator::LocalPath {
        path: PRIVATE_PATH.to_owned(),
    };
    let output = format!("{locator:?}");
    forbidden(&output);
    assert!(output.contains("LocalPath"), "{output}");
}

#[test]
fn a_remote_locator_never_prints_its_url_across_the_gate() {
    let locator = FfiHandoffLocator::Remote {
        url: PRIVATE_URL.to_owned(),
    };
    let output = format!("{locator:?}");
    forbidden(&output);
    assert!(output.contains("Remote"), "{output}");
}

#[test]
fn a_request_never_prints_through_to_its_locator() {
    let request = FfiHandoffRequest {
        locator: FfiHandoffLocator::LocalPath {
            path: PRIVATE_PATH.to_owned(),
        },
        start_position_ms: Some(7_000),
    };
    let output = format!("{request:?}");
    forbidden(&output);
    assert!(output.contains("7000"), "{output}");
}

#[test]
fn a_rejection_carries_no_input() {
    let rejections = [
        FfiHandoffRejection::NoLocator,
        FfiHandoffRejection::UnsupportedScheme,
        FfiHandoffRejection::MalformedLocator,
    ];
    let output = rejections
        .iter()
        .map(|rejection| format!("{rejection:?} {rejection}"))
        .collect::<Vec<_>>()
        .join(" ");
    forbidden(&output);
    assert!(output.contains("UnsupportedScheme"), "{output}");
    assert!(output.contains("unsupported_scheme"), "{output}");
    for rejection in rejections {
        assert!(!rejection_name(rejection).is_empty());
    }
}

#[test]
fn a_derived_debug_on_this_shape_would_leak() {
    // The control. If this twin stops leaking, the guard above has stopped
    // proving anything and the fixtures need to be made private again.
    #[derive(Debug)]
    #[allow(dead_code)]
    enum DerivedLocatorTwin {
        LocalPath { path: String },
        Remote { url: String },
    }

    let local = DerivedLocatorTwin::LocalPath {
        path: PRIVATE_PATH.to_owned(),
    };
    let remote = DerivedLocatorTwin::Remote {
        url: PRIVATE_URL.to_owned(),
    };
    let printed = format!("{local:?} {remote:?}");
    assert!(printed.contains(PRIVATE_PATH), "{printed}");
    assert!(printed.contains(PRIVATE_URL), "{printed}");
}
