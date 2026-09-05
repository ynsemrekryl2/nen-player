//! K23: nothing on the playback port prints a media URL, a token, a private
//! path, a filename or subtitle dialogue.
//!
//! Two things are proven here, and the second is what makes the first worth
//! anything:
//!
//! 1. The port's types do not leak.
//! 2. **The check is not vacuous** — a deliberately derived twin of each type,
//!    carrying the same values, leaks every pattern the guard looks for. If the
//!    hand-written `Debug` impls were ever replaced with `#[derive(Debug)]`,
//!    these tests would catch it, because the twin proves that is exactly what
//!    a derive does.
//!
//! Same technique as `nen-domain`'s `guard_redaction.rs` and `nen-identity`'s
//! `guard_evidence_debug.rs`.

use nen_ports::playback::error::LoadFailure;
use nen_ports::playback::{
    Capability, MediaSource, Operation, PlaybackError, TrackDescriptor, TrackId, TrackKind,
    VideoGeometry,
};

/// A media URL with a token (K23 #1, #2).
const MEDIA_URL: &str = "https://cdn.example.com/stream.mkv?token=SECRETTOKEN";
/// A private full path (K23 #3).
const PRIVATE_PATH: &str = "/Users/someone/Movies/Private/film.mkv";
/// A private filename / release name (K23 #8).
const PRIVATE_TITLE: &str = "The.Film.2019.1080p.WEB-DL.PRIVATEGROUP";
/// Subtitle dialogue (K23 #4).
const DIALOGUE: &str = "Bunu kimseye söyleme";

/// Every substring that must never appear in a `Debug` line.
fn forbidden_fragments() -> Vec<&'static str> {
    vec![
        "cdn.example.com",
        "stream.mkv",
        "SECRETTOKEN",
        "token=",
        "/Users/someone",
        "Private",
        "film.mkv",
        "PRIVATEGROUP",
        "WEB-DL",
        DIALOGUE,
    ]
}

fn assert_clean(what: &str, printed: &str) {
    for fragment in forbidden_fragments() {
        assert!(
            !printed.contains(fragment),
            "{what} leaked {fragment:?}: {printed}"
        );
    }
}

fn assert_leaks(what: &str, printed: &str, fragment: &str) {
    assert!(
        printed.contains(fragment),
        "{what} was expected to leak {fragment:?} but did not: {printed}\n\
         If this fails the negative control is broken, and the positive tests \
         above prove nothing."
    );
}

#[test]
fn media_source_never_prints_its_locator() {
    for locator in [MEDIA_URL, PRIVATE_PATH] {
        let source = MediaSource::new(locator);
        assert_clean("MediaSource", &format!("{source:?}"));
    }
}

#[test]
fn track_descriptor_never_prints_its_title() {
    let track = TrackDescriptor::new(TrackId(0), TrackKind::Subtitle, "subrip")
        .with_title(Some(PRIVATE_TITLE.to_string()));
    assert_clean("TrackDescriptor", &format!("{track:?}"));
}

#[test]
fn track_descriptor_still_prints_what_is_safe() {
    // A guard that hid everything would be useless for diagnostics.
    let track = TrackDescriptor::new(TrackId(7), TrackKind::Subtitle, "subrip")
        .with_title(Some(PRIVATE_TITLE.to_string()));
    let printed = format!("{track:?}");
    assert!(printed.contains("subrip"), "codec missing: {printed}");
    assert!(printed.contains('7'), "track id missing: {printed}");
    assert!(printed.contains("has_title: true"), "{printed}");
}

#[test]
fn video_geometry_prints_its_numbers_and_nothing_else() {
    // ADR-0038 Karar 5 places the display size **outside** K23: it is not a
    // URL, a path, a filename or dialogue, and it is loggable.
    //
    // So this guard runs in both directions. The size must stay clean — a
    // future field able to hold a string would be caught by the first half —
    // and it must still *print*, because a type redacted "to be safe" would
    // take the one fact it exists to report out of every diagnostic.
    let geometry = VideoGeometry::new(1_024, 576).expect("a valid size");
    let printed = format!("{geometry:?}");
    assert_clean("VideoGeometry", &printed);
    assert!(printed.contains("1024"), "width missing: {printed}");
    assert!(printed.contains("576"), "height missing: {printed}");
}

#[test]
fn no_playback_error_variant_carries_private_data() {
    // Every variant, constructed and printed. The variants hold only bounded
    // enums and numbers by design (see the module note in error.rs); this test
    // is what stops a future variant from quietly holding a `String`.
    let errors = [
        PlaybackError::Unsupported {
            operation: Operation::SetRate,
            capability: Capability::PlaybackRate,
        },
        PlaybackError::ReentrantCall {
            operation: Operation::Play,
        },
        PlaybackError::NotLoaded {
            operation: Operation::Seek,
        },
        PlaybackError::ShutDown {
            operation: Operation::Load,
        },
        PlaybackError::UnknownTrack {
            kind: TrackKind::Subtitle,
        },
        PlaybackError::RateOutOfRange {
            requested: 4.0,
            min: 0.5,
            max: 2.0,
        },
        PlaybackError::LoadFailed {
            reason: LoadFailure::NotFound,
        },
        PlaybackError::InsetOutOfRange {
            operation: Operation::SetSubtitleBottomInset,
        },
        PlaybackError::EngineFailure { code: -22 },
    ];
    for error in errors {
        assert_clean("PlaybackError", &format!("{error:?}"));
        assert_clean("PlaybackError (Display)", &error.to_string());
    }
}

// --- Negative control -------------------------------------------------------
//
// Deliberately broken twins holding the same values. Each must leak, which is
// what proves the guards above are testing something real.

#[derive(Debug)]
#[allow(dead_code)]
struct DerivedMediaSource {
    locator: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct DerivedTrackDescriptor {
    id: u32,
    codec: String,
    title: Option<String>,
}

#[derive(Debug)]
#[allow(dead_code)]
enum LeakyPlaybackError {
    LoadFailed { locator: String },
    ExtractFailed { dialogue: String },
}

#[test]
fn a_derived_media_source_really_does_leak() {
    let twin = DerivedMediaSource {
        locator: MEDIA_URL.to_string(),
    };
    let printed = format!("{twin:?}");
    assert_leaks("DerivedMediaSource", &printed, "SECRETTOKEN");
    assert_leaks("DerivedMediaSource", &printed, "cdn.example.com");
}

#[test]
fn a_derived_track_descriptor_really_does_leak() {
    let twin = DerivedTrackDescriptor {
        id: 0,
        codec: "subrip".to_string(),
        title: Some(PRIVATE_TITLE.to_string()),
    };
    assert_leaks(
        "DerivedTrackDescriptor",
        &format!("{twin:?}"),
        "PRIVATEGROUP",
    );
}

#[test]
fn an_error_holding_strings_really_does_leak() {
    // This is the shape PlaybackError deliberately does not have. NEN-010
    // measured why it must not: an error crossing the FFI boundary is printed
    // by the host language, which never consults a Rust `Debug` impl — so a
    // value that is only safe because of how Rust prints it is not safe.
    assert_leaks(
        "LeakyPlaybackError",
        &format!(
            "{:?}",
            LeakyPlaybackError::LoadFailed {
                locator: PRIVATE_PATH.to_string()
            }
        ),
        "/Users/someone",
    );
    assert_leaks(
        "LeakyPlaybackError",
        &format!(
            "{:?}",
            LeakyPlaybackError::ExtractFailed {
                dialogue: DIALOGUE.to_string()
            }
        ),
        DIALOGUE,
    );
}
