use nen_catalog::{project, MenuGroup, SubtitleMenu, SubtitleSourceCatalog};
use nen_domain::source::{LanguageTag, SubtitlePreferences, SubtitleSource, SubtitleSourceId};
use std::fs;
use std::path::PathBuf;

fn tag(value: &str) -> LanguageTag {
    LanguageTag::parse(value).expect("fixture language tag is valid")
}

fn fixture_catalog() -> SubtitleSourceCatalog {
    let english = SubtitleSourceId::embedded(0);
    [
        SubtitleSource::new(
            SubtitleSourceId::user([1; 32]),
            Some(tag("en")),
            "Movie.en.srt",
        ),
        SubtitleSource::new(
            SubtitleSourceId::user([2; 32]),
            Some(tag("fr")),
            "subtitle.srt",
        ),
        SubtitleSource::new(english.clone(), Some(tag("en")), "English"),
        SubtitleSource::new(
            SubtitleSourceId::opensubtitles("public-en-web-dl"),
            Some(tag("en")),
            "English — WEB-DL",
        ),
        SubtitleSource::new(SubtitleSourceId::embedded(1), Some(tag("fr")), "Français"),
        SubtitleSource::new(SubtitleSourceId::embedded(2), Some(tag("tr")), "Türkçe"),
        SubtitleSource::new(
            SubtitleSourceId::ai(&english, &tag("tr")),
            Some(tag("tr")),
            "AI Türkçe",
        ),
    ]
    .into_iter()
    .collect()
}

/// Two Portuguese subtitles that really are different text (ADR-0030).
fn region_catalog() -> SubtitleSourceCatalog {
    [
        SubtitleSource::new(
            SubtitleSourceId::embedded(0),
            Some(tag("pt-br")),
            "Português (BR)",
        ),
        SubtitleSource::new(
            SubtitleSourceId::embedded(1),
            Some(tag("pt-pt")),
            "Português (PT)",
        ),
    ]
    .into_iter()
    .collect()
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
            // Only the exceptional case is written, so the goldens ADR-0010
            // Karar 10 pins stay byte-for-byte what they were: an ordinary
            // source is translatable and says nothing about it.
            if !source.translatable() {
                output.push_str("\tbitmap");
            }
            output.push('\n');
        }
    }
    output
}

fn assert_golden(name: &str, actual: &str) {
    let expected = fs::read_to_string(golden_path(name)).expect("golden fixture is readable");
    assert_eq!(actual, expected, "projection differs from {name}");
}

#[test]
fn menu_without_preferences_matches_adr_0010_decision_10() {
    let catalog = fixture_catalog();
    let menu = project(&catalog, &SubtitlePreferences::none());
    assert_golden("menu-default.golden", &snapshot(&menu));
}

#[test]
fn menu_with_turkish_then_english_matches_adr_0010_decision_10() {
    let catalog = fixture_catalog();
    let preferences = SubtitlePreferences::new(Some(tag("tr")), Some(tag("en")));
    let menu = project(&catalog, &preferences);
    assert_golden("menu-preferred-tr-en.golden", &snapshot(&menu));
}

#[test]
fn regions_of_one_language_share_one_heading_and_keep_their_tags() {
    // ADR-0030 Karar 1/3 at the layer the user actually sees: one `pt` heading,
    // both entries still under it, each still carrying its own region so a UI
    // can label them apart (NEN-026).
    let catalog = region_catalog();
    let menu = project(&catalog, &SubtitlePreferences::none());
    assert_golden("menu-pt-regions.golden", &snapshot(&menu));
}
