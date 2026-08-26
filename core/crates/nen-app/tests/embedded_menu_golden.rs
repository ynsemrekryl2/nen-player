//! What a medium's own tracks look like in the menu, end to end (NEN-023).
//!
//! `nen-catalog`'s goldens pin the *projection* against a hand-built catalog
//! (ADR-0010 Karar 10). This one pins the step before it: real
//! [`TrackDescriptor`]s, as an engine reports them, walked all the way through
//! [`embedded_sources`] into the same projection. A mapping bug — a lost
//! language, a bitmap track quietly marked translatable, an audio track
//! catalogued as a subtitle — shows up here as a diff.
//!
//! It is also the evidence for ADR-0031 Karar 4's first half: the menu a user
//! sees the instant a medium opens is `Kapalı` plus the embedded tracks, with
//! no scan having run.
//!
//! The renderer below deliberately mirrors `nen-catalog`'s golden test. The two
//! are separate artifacts on purpose: this crate cannot reach into that one's
//! test module, and a shared formatter would make one golden's meaning depend
//! on the other's file.

use nen_app::embedded::embedded_sources;
use nen_catalog::{project, MenuGroup, SubtitleMenu, SubtitleSourceCatalog};
use nen_domain::source::{LanguageTag, SubtitlePreferences};
use nen_ports::playback::{TrackDescriptor, TrackId, TrackKind};
use std::fs;
use std::path::PathBuf;

fn tag(input: &str) -> LanguageTag {
    LanguageTag::parse(input).expect("valid tag")
}

/// What an engine reports for a container carrying one audio track, two text
/// subtitle tracks and one bitmap track — the shape
/// `fixtures/media/bitmap-subs-clip.mkv` actually has, plus a titleless track
/// to pin the empty-label rule.
fn reported_tracks() -> Vec<TrackDescriptor> {
    vec![
        TrackDescriptor::new(TrackId(1), TrackKind::Audio, "opus")
            .with_language(Some(tag("en")))
            .with_default(true),
        TrackDescriptor::new(TrackId(2), TrackKind::Subtitle, "subrip")
            .with_language(Some(tag("en")))
            .with_title(Some("English".into()))
            .with_default(true),
        TrackDescriptor::new(TrackId(3), TrackKind::Subtitle, "hdmv_pgs_subtitle")
            .with_language(Some(tag("fr"))),
        TrackDescriptor::new(TrackId(4), TrackKind::Subtitle, "ass"),
    ]
}

fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../fixtures/catalog")
        .join(name)
}

fn group_name(group: &MenuGroup) -> String {
    match group {
        MenuGroup::Closed => "closed".to_owned(),
        MenuGroup::UserSubtitles => "user".to_owned(),
        MenuGroup::Language(language) => format!("language:{language}"),
        MenuGroup::UnknownLanguage => "unknown".to_owned(),
    }
}

fn snapshot(menu: &SubtitleMenu<'_>) -> String {
    let mut output = String::new();
    for section in &menu.sections {
        output.push_str("group\t");
        output.push_str(&group_name(&section.group));
        output.push('\n');
        for source in &section.entries {
            output.push_str("entry\t");
            output.push_str(source.kind().as_str());
            output.push('\t');
            output.push_str(source.language().map_or("unknown", LanguageTag::as_str));
            output.push('\t');
            output.push_str(source.label());
            if !source.translatable() {
                output.push_str("\tbitmap");
            }
            output.push('\n');
        }
    }
    output
}

#[test]
fn a_mediums_own_tracks_project_into_the_menu() {
    let tracks = reported_tracks();
    let catalog: SubtitleSourceCatalog = embedded_sources(&tracks).into_iter().collect();
    let menu = project(&catalog, &SubtitlePreferences::none());

    let expected = fs::read_to_string(golden_path("menu-embedded-bitmap.golden"))
        .expect("golden fixture is readable");
    assert_eq!(snapshot(&menu), expected);
}

#[test]
fn the_menu_a_medium_opens_with_is_closed_plus_its_tracks() {
    // ADR-0031 Karar 4: no scan has run, so there is no user group and no
    // OpenSubtitles candidate — and `Kapalı` is there regardless.
    let tracks = reported_tracks();
    let catalog: SubtitleSourceCatalog = embedded_sources(&tracks).into_iter().collect();
    let menu = project(&catalog, &SubtitlePreferences::none());

    let groups: Vec<String> = menu.groups().map(group_name).collect();
    assert_eq!(groups, ["closed", "language:en", "language:fr", "unknown"]);
}
