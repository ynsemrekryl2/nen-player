//! K23 #8: a track title never reaches a log — not even the gate's own type.
//!
//! [`FfiTrackDescriptor`] is the one record that carries a string a container
//! wrote, and a container writes release names and private filenames into
//! `title` all the time. `nen-ports` already holds this line for
//! `TrackDescriptor`; the gate has to hold it separately, because a
//! `#[derive(Debug)]` here would print the title before the value ever reaches
//! that type.
//!
//! The twin at the bottom is what keeps these assertions honest. Every check
//! above is "this string does not contain that", which passes for free if the
//! fragments are wrong or the printing is empty.

use nen_ffi::playback::{FfiTrackDescriptor, FfiTrackKind};

/// A title built out of what K23 #8 names, so a leak is unmistakable.
const SECRET_TITLE: &str = "S3CR3T.Release.Name.2019.1080p-PRIVATEGROUP";

const FORBIDDEN: [&str; 4] = ["S3CR3T", "Release.Name", "PRIVATEGROUP", "1080p"];

fn descriptor() -> FfiTrackDescriptor {
    FfiTrackDescriptor {
        id: 3,
        kind: FfiTrackKind::Subtitle,
        language: Some("en".to_string()),
        codec: "subrip".to_string(),
        is_default: true,
        title: Some(SECRET_TITLE.to_string()),
    }
}

#[test]
fn debug_never_prints_the_title() {
    let printed = format!("{:?}", descriptor());
    for fragment in FORBIDDEN {
        assert!(
            !printed.contains(fragment),
            "title leaked {fragment}: {printed}"
        );
    }
}

#[test]
fn debug_still_prints_what_is_safe() {
    // The guard must not be satisfied by printing nothing: a descriptor that
    // says nothing is useless in a log, and a future reader would "fix" it.
    let printed = format!("{:?}", descriptor());
    assert!(printed.contains("subrip"), "{printed}");
    assert!(printed.contains("Subtitle"), "{printed}");
    assert!(printed.contains("has_title: true"), "{printed}");
}

#[test]
fn a_titleless_track_says_so_rather_than_going_silent() {
    let mut track = descriptor();
    track.title = None;
    let printed = format!("{track:?}");
    assert!(printed.contains("has_title: false"), "{printed}");
}

#[test]
fn a_derived_descriptor_really_does_leak() {
    // The control. Same fields, same values, `#[derive(Debug)]` — which is
    // exactly what someone replacing the hand-written impl would get.
    #[derive(Debug)]
    #[allow(dead_code)]
    struct DerivedTwin {
        id: u32,
        language: Option<String>,
        codec: String,
        is_default: bool,
        title: Option<String>,
    }

    let twin = DerivedTwin {
        id: 3,
        language: Some("en".to_string()),
        codec: "subrip".to_string(),
        is_default: true,
        title: Some(SECRET_TITLE.to_string()),
    };
    let printed = format!("{twin:?}");

    let leaked = FORBIDDEN
        .iter()
        .filter(|fragment| printed.contains(*fragment))
        .count();
    assert_eq!(
        leaked,
        FORBIDDEN.len(),
        "the twin only leaked {leaked} of {} fragments: {printed}",
        FORBIDDEN.len()
    );
}

#[test]
fn a_containers_three_letter_language_arrives_canonical() {
    // ADR-0032, at the seam it exists for. Matroska writes ISO 639-2, so this
    // is literally what `fixtures/media/contract-clip.mkv` reports; the value
    // has to reach the catalog as `en`, or the embedded track and the user's
    // `Movie.en.srt` become two menu groups.
    use nen_app::ports::playback::TrackDescriptor;

    let converted: TrackDescriptor = FfiTrackDescriptor {
        id: 3,
        kind: FfiTrackKind::Subtitle,
        language: Some("eng".to_string()),
        codec: "subrip".to_string(),
        is_default: true,
        title: None,
    }
    .into();

    assert_eq!(converted.language().map(|l| l.as_str()), Some("en"));
}

#[test]
fn a_bitmap_codec_is_classified_on_this_side_of_the_boundary() {
    // The adapter reports a codec and nothing else; `is_text` is not a field
    // it can get wrong, because it is not a field it sends.
    use nen_app::ports::playback::TrackDescriptor;

    let bitmap: TrackDescriptor = FfiTrackDescriptor {
        id: 3,
        kind: FfiTrackKind::Subtitle,
        language: Some("fre".to_string()),
        codec: "hdmv_pgs_subtitle".to_string(),
        is_default: false,
        title: None,
    }
    .into();

    assert!(!bitmap.is_text());
    assert_eq!(bitmap.language().map(|l| l.as_str()), Some("fr"));
}
