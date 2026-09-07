//! The menu the shell draws (NEN-026).
//!
//! `nen-catalog`'s goldens pin *which* headings exist and in what order.
//! This file pins the layer above them: the token that lets a shell name a row
//! without holding an identity, and the defect that makes a row unselectable
//! (ADR-0031 Karar 5). Nothing here re-decides grouping — a test that did would
//! be a second opinion about §8 sitting next to the goldens that hold the first.

use nen_app::embedded::embedded_sources;
use nen_app::subtitle_files::SourceDefect;
use nen_app::subtitles::{AddOutcome, MenuSectionView, SubtitleLibrary};
use nen_catalog::MenuGroup;
use nen_domain::source::{LanguageTag, SubtitlePreferences, SubtitleSourceKind};
use nen_ports::playback::{TrackDescriptor, TrackId, TrackKind};
use std::fs;
use std::path::{Path, PathBuf};

const VALID_SRT: &str = "1\n00:00:01,000 --> 00:00:02,000\nHello there.\n";
/// Times that do not advance — NEN-013's strict parser refuses it, so the file
/// is read fine and *then* found to be broken. That is `biçim hatalı`.
const MALFORMED_SRT: &str = "1\n00:00:05,000 --> 00:00:02,000\nBackwards.\n";

fn tag(input: &str) -> LanguageTag {
    LanguageTag::parse(input).expect("valid tag")
}

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("nen-026-{name}-{}-{unique}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temp dir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).expect("write fixture");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// English text, French bitmap, one titleless track — plus an audio track that
/// must not become a subtitle source.
fn reported_tracks() -> Vec<TrackDescriptor> {
    vec![
        TrackDescriptor::new(TrackId(1), TrackKind::Audio, "opus").with_default(true),
        TrackDescriptor::new(TrackId(2), TrackKind::Subtitle, "subrip")
            .with_language(Some(tag("en")))
            .with_title(Some("English".into())),
        TrackDescriptor::new(TrackId(3), TrackKind::Subtitle, "hdmv_pgs_subtitle")
            .with_language(Some(tag("fr"))),
        TrackDescriptor::new(TrackId(4), TrackKind::Subtitle, "ass"),
    ]
}

fn groups(sections: &[MenuSectionView]) -> Vec<MenuGroup> {
    sections.iter().map(|s| s.group.clone()).collect()
}

fn section<'a>(sections: &'a [MenuSectionView], group: &MenuGroup) -> &'a MenuSectionView {
    sections
        .iter()
        .find(|s| &s.group == group)
        .unwrap_or_else(|| panic!("{group:?} missing from {:?}", groups(sections)))
}

// --- what the menu carries --------------------------------------------------

#[test]
fn an_empty_library_offers_closed_and_nothing_else() {
    let library = SubtitleLibrary::new();
    let menu = library.menu(&SubtitlePreferences::none());
    assert_eq!(groups(&menu), [MenuGroup::Closed]);
    assert!(menu[0].entries.is_empty());
}

#[test]
fn a_heading_with_nothing_under_it_never_appears() {
    // The rule the shell's first column draws: `Kullanıcı Altyazıları` and
    // `Dil Belirsiz` exist only once something is in them (§8).
    let mut library = SubtitleLibrary::new();
    library.add_embedded(embedded_sources(&[TrackDescriptor::new(
        TrackId(2),
        TrackKind::Subtitle,
        "subrip",
    )
    .with_language(Some(tag("en")))]));

    assert_eq!(
        groups(&library.menu(&SubtitlePreferences::none())),
        [MenuGroup::Closed, MenuGroup::Language(tag("en"))],
        "no user group, no unknown group"
    );
}

#[test]
fn a_titleless_track_opens_the_unknown_group_and_keeps_its_empty_label() {
    let mut library = SubtitleLibrary::new();
    library.add_embedded(embedded_sources(&reported_tracks()));
    let menu = library.menu(&SubtitlePreferences::none());

    assert_eq!(
        groups(&menu),
        [
            MenuGroup::Closed,
            MenuGroup::Language(tag("en")),
            MenuGroup::Language(tag("fr")),
            MenuGroup::UnknownLanguage,
        ]
    );
    let unknown = section(&menu, &MenuGroup::UnknownLanguage);
    assert_eq!(unknown.entries.len(), 1);
    assert_eq!(
        unknown.entries[0].label, "",
        "the UI substitutes the endonym, the core invents nothing"
    );
}

#[test]
fn a_bitmap_track_crosses_marked_untranslatable() {
    let mut library = SubtitleLibrary::new();
    library.add_embedded(embedded_sources(&reported_tracks()));
    let menu = library.menu(&SubtitlePreferences::none());

    let french = section(&menu, &MenuGroup::Language(tag("fr")));
    assert!(!french.entries[0].translatable);
    let english = section(&menu, &MenuGroup::Language(tag("en")));
    assert!(english.entries[0].translatable);
}

#[test]
fn a_broken_sidecar_stays_in_the_menu_carrying_its_reason() {
    // ADR-0031 Karar 5 + ADR-0035 Karar 2: it is in the list, and the list is
    // where it says why.
    let dir = TempDir::new("broken");
    let media = dir.write("Clip.mkv", "not really a video");
    dir.write("Clip.srt", MALFORMED_SRT);

    let mut library = SubtitleLibrary::new();
    assert_eq!(
        library.add_sidecars_of(&media),
        vec![AddOutcome::Defective(SourceDefect::Malformed)]
    );

    let menu = library.menu(&SubtitlePreferences::none());
    let user = section(&menu, &MenuGroup::UserSubtitles);
    assert_eq!(user.entries.len(), 1);
    assert_eq!(user.entries[0].defect, Some(SourceDefect::Malformed));
}

#[test]
fn a_readable_sidecar_carries_no_reason() {
    let dir = TempDir::new("fine");
    let media = dir.write("Clip.mkv", "not really a video");
    dir.write("Clip.srt", VALID_SRT);

    let mut library = SubtitleLibrary::new();
    assert_eq!(library.add_sidecars_of(&media), vec![AddOutcome::Added]);

    let menu = library.menu(&SubtitlePreferences::none());
    let user = section(&menu, &MenuGroup::UserSubtitles);
    assert_eq!(user.entries[0].defect, None);
    assert_eq!(user.entries[0].kind, SubtitleSourceKind::User);
}

// --- tokens -----------------------------------------------------------------

#[test]
fn every_row_carries_a_token_that_resolves_back_to_its_source() {
    let mut library = SubtitleLibrary::new();
    library.add_embedded(embedded_sources(&reported_tracks()));

    for entry in library
        .menu(&SubtitlePreferences::none())
        .iter()
        .flat_map(|s| &s.entries)
    {
        assert_ne!(entry.token, 0, "zero is reserved for 'no row'");
        let id = library.id_of(entry.token).expect("a source");
        assert_eq!(library.token_of(id), Some(entry.token));
    }
}

#[test]
fn a_growing_library_never_renumbers_a_row() {
    // ADR-0031 Karar 4.1/4.2. The scan finishing is what makes
    // `Kullanıcı Altyazıları` appear at position two, pushing every language
    // heading down a row; if the tokens moved with them, the shell's selection
    // would land on a different subtitle than the one the user picked.
    let dir = TempDir::new("growth");
    let media = dir.write("Clip.mkv", "not really a video");

    let mut library = SubtitleLibrary::new();
    library.add_embedded(embedded_sources(&reported_tracks()));
    let before: Vec<(u32, String)> = library
        .menu(&SubtitlePreferences::none())
        .iter()
        .flat_map(|s| &s.entries)
        .map(|e| (e.token, e.label.clone()))
        .collect();

    dir.write("Clip.srt", VALID_SRT);
    assert_eq!(library.add_sidecars_of(&media), vec![AddOutcome::Added]);

    let after: Vec<(u32, String)> = library
        .menu(&SubtitlePreferences::none())
        .iter()
        .flat_map(|s| &s.entries)
        .map(|e| (e.token, e.label.clone()))
        .collect();

    assert_eq!(after.len(), before.len() + 1);
    for pair in &before {
        assert!(
            after.contains(pair),
            "{pair:?} changed token when the scan landed"
        );
    }
}

#[test]
fn reloading_a_file_keeps_its_token() {
    // The user fixed a broken sidecar and loaded it again: same row, same
    // token, now without a reason.
    let dir = TempDir::new("reload");
    let path = dir.write("Movie.srt", MALFORMED_SRT);

    let mut library = SubtitleLibrary::new();
    library.add_file(&path, dir.path());
    let token = library.menu(&SubtitlePreferences::none())[1].entries[0].token;

    fs::write(&path, VALID_SRT).expect("rewrite");
    assert_eq!(library.add_file(&path, dir.path()), AddOutcome::Added);

    let entry = &library.menu(&SubtitlePreferences::none())[1].entries[0];
    assert_eq!(entry.token, token);
    assert_eq!(entry.defect, None);
}

#[test]
fn an_unknown_token_resolves_to_nothing_rather_than_a_neighbour() {
    let mut library = SubtitleLibrary::new();
    library.add_embedded(embedded_sources(&reported_tracks()));
    assert!(library.id_of(0).is_none(), "zero is not a row");
    assert!(library.id_of(9_999).is_none());
    assert!(library.embedded_track_of(9_999).is_none());
}

// --- selection --------------------------------------------------------------

#[test]
fn an_embedded_row_resolves_to_the_track_the_engine_reported() {
    let mut library = SubtitleLibrary::new();
    library.add_embedded(embedded_sources(&reported_tracks()));
    let menu = library.menu(&SubtitlePreferences::none());

    let english = &section(&menu, &MenuGroup::Language(tag("en"))).entries[0];
    assert_eq!(library.embedded_track_of(english.token), Some(TrackId(2)));
}

#[test]
fn a_user_row_resolves_to_no_track_at_all() {
    // Not an error: showing a user file is NEN-027's job, not the engine's
    // track selection.
    let dir = TempDir::new("usertrack");
    let path = dir.write("Movie.srt", VALID_SRT);

    let mut library = SubtitleLibrary::new();
    library.add_file(&path, dir.path());
    let token = library.menu(&SubtitlePreferences::none())[1].entries[0].token;
    assert!(library.embedded_track_of(token).is_none());
}

#[test]
fn auto_selection_names_a_row_in_the_preferred_language() {
    let mut library = SubtitleLibrary::new();
    library.add_embedded(embedded_sources(&reported_tracks()));

    let prefs = SubtitlePreferences::new(Some(tag("fr")), None);
    let token = library.auto_selection(&prefs).expect("a pick");
    let french = library.id_of(token).expect("a source");
    assert_eq!(
        library.menu(&prefs)[1].group,
        MenuGroup::Language(tag("fr")),
        "the preferred language is hoisted"
    );
    assert_eq!(library.embedded_track_of(token), Some(TrackId(3)));
    assert!(library.is_usable(french));
}

#[test]
fn auto_selection_picks_nothing_when_no_preference_matches() {
    let mut library = SubtitleLibrary::new();
    library.add_embedded(embedded_sources(&reported_tracks()));
    assert_eq!(
        library.auto_selection(&SubtitlePreferences::new(Some(tag("tr")), None)),
        None
    );
    assert_eq!(library.auto_selection(&SubtitlePreferences::none()), None);
}

// --- K23 --------------------------------------------------------------------

#[test]
fn a_menu_row_prints_without_its_label() {
    // The label is regularly a private filename (K23 #8). The guard is the
    // hand-written `Debug`; this is what proves it is still hand-written.
    let dir = TempDir::new("redact");
    let path = dir.write("Inception.2010.tr.srt", VALID_SRT);

    let mut library = SubtitleLibrary::new();
    library.add_file(&path, dir.path());
    let entry = library.menu(&SubtitlePreferences::none())[1].entries[0].clone();

    assert_eq!(entry.label, "Inception.2010.tr.srt", "it is displayable");
    let printed = format!("{entry:?}");
    assert!(!printed.contains("Inception"), "{printed}");
    assert!(!printed.contains(".srt"), "{printed}");
    assert!(printed.contains("has_label: true"), "{printed}");
}
