//! The subtitle menu as a **projection** of the catalog (product-spec §8).

use crate::catalog::SubtitleSourceCatalog;
use nen_domain::source::{LanguageTag, SubtitlePreferences, SubtitleSource, SubtitleSourceKind};
use std::collections::BTreeMap;

/// One heading in the subtitle menu.
///
/// The variants carry no display text on purpose (ADR-0010 Karar 7). A
/// language's visible name is its **own** name — "English", "Français",
/// "Türkçe" — in every UI language, and the rest of the chrome ("Kapalı",
/// "Kullanıcı Altyazıları", "Dil Belirsiz") is translated by the platform. Both
/// halves of that are the UI's business; keeping them out of here is what lets
/// one core drive three platforms and lets the golden test pin an order that
/// does not shift with the UI language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuGroup {
    /// "Subtitles off" — always present, never a catalog entry (Karar 3).
    Closed,
    /// User-loaded files and discovered sidecars, whatever their language (§8).
    UserSubtitles,
    /// Everything else, keyed by language.
    Language(LanguageTag),
    /// Sources whose language could not be established (`Dil Belirsiz`).
    UnknownLanguage,
}

/// A heading and the entries under it, in menu order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSection<'a> {
    pub group: MenuGroup,
    pub entries: Vec<&'a SubtitleSource>,
}

/// The projected menu. Derived on every call, never stored in the catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubtitleMenu<'a> {
    pub sections: Vec<MenuSection<'a>>,
}

impl<'a> SubtitleMenu<'a> {
    pub fn section(&self, group: &MenuGroup) -> Option<&MenuSection<'a>> {
        self.sections.iter().find(|s| &s.group == group)
    }

    /// The group headings in order — the thing ADR-0010 Karar 4 fixes.
    pub fn groups(&self) -> impl Iterator<Item = &MenuGroup> {
        self.sections.iter().map(|s| &s.group)
    }
}

/// Projects the catalog into the menu of §8, as extended by ADR-0010.
///
/// Order (Karar 4):
///
/// ```text
/// Kapalı
/// Kullanıcı Altyazıları     (omitted when there are none)
/// <primary preference>      (omitted when that language has no sources)
/// <secondary preference>
/// <remaining languages>     ascending by LanguageTag
/// Dil Belirsiz              (always last, omitted when empty)
/// ```
///
/// Within a group, entries keep their catalog order. That order is **not** a
/// priority (Karar 5): writing a kind-based ranking here would be an automatic
/// source priority, which §7 forbids. The order §8's example shows falls out of
/// the sequence sources are collected in (`docs/architecture.md`), so no rule is
/// needed to produce it.
///
/// Preferences only reorder. They never hide or filter a group — §8 keeps every
/// source in one menu, and a hidden group is a subtitle the user cannot find.
pub fn project<'a>(
    catalog: &'a SubtitleSourceCatalog,
    preferences: &SubtitlePreferences,
) -> SubtitleMenu<'a> {
    let mut sections = vec![MenuSection {
        group: MenuGroup::Closed,
        entries: Vec::new(),
    }];

    let user_entries: Vec<&SubtitleSource> = catalog.of_kind(SubtitleSourceKind::User).collect();
    if !user_entries.is_empty() {
        sections.push(MenuSection {
            group: MenuGroup::UserSubtitles,
            entries: user_entries,
        });
    }

    // BTreeMap keeps the languages we did not put first in ascending tag order,
    // and each Vec keeps its language's entries in catalog order.
    let mut by_language: BTreeMap<&LanguageTag, Vec<&SubtitleSource>> = BTreeMap::new();
    let mut unknown: Vec<&SubtitleSource> = Vec::new();
    for source in catalog.sources() {
        if source.kind() == SubtitleSourceKind::User {
            continue;
        }
        match source.language() {
            Some(language) => by_language.entry(language).or_default().push(source),
            None => unknown.push(source),
        }
    }

    for preferred in preferences.ordered() {
        if let Some(entries) = by_language.remove(preferred) {
            sections.push(MenuSection {
                group: MenuGroup::Language(preferred.clone()),
                entries,
            });
        }
    }

    for (language, entries) in by_language {
        sections.push(MenuSection {
            group: MenuGroup::Language(language.clone()),
            entries,
        });
    }

    if !unknown.is_empty() {
        sections.push(MenuSection {
            group: MenuGroup::UnknownLanguage,
            entries: unknown,
        });
    }

    SubtitleMenu { sections }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nen_domain::source::{SubtitleSource, SubtitleSourceId};

    fn tag(s: &str) -> LanguageTag {
        LanguageTag::parse(s).expect("valid tag")
    }

    fn embedded(index: u32, lang: Option<&str>, label: &str) -> SubtitleSource {
        SubtitleSource::new(
            SubtitleSourceId::embedded(index),
            lang.map(tag),
            label.to_owned(),
        )
    }

    fn user_id(seed: u8) -> SubtitleSourceId {
        SubtitleSourceId::user([seed; 32])
    }

    fn groups(menu: &SubtitleMenu<'_>) -> Vec<MenuGroup> {
        menu.groups().cloned().collect()
    }

    #[test]
    fn closed_is_present_even_for_an_empty_catalog() {
        let catalog = SubtitleSourceCatalog::new();
        let menu = project(&catalog, &SubtitlePreferences::none());
        assert_eq!(groups(&menu), [MenuGroup::Closed]);
    }

    #[test]
    fn without_preferences_languages_follow_tag_order() {
        let catalog: SubtitleSourceCatalog = [
            embedded(0, Some("tr"), "Türkçe"),
            embedded(1, Some("en"), "English"),
            embedded(2, Some("fr"), "Français"),
        ]
        .into_iter()
        .collect();
        let menu = project(&catalog, &SubtitlePreferences::none());
        assert_eq!(
            groups(&menu),
            [
                MenuGroup::Closed,
                MenuGroup::Language(tag("en")),
                MenuGroup::Language(tag("fr")),
                MenuGroup::Language(tag("tr")),
            ]
        );
    }

    #[test]
    fn preferred_languages_come_first_in_preference_order() {
        let catalog: SubtitleSourceCatalog = [
            embedded(0, Some("en"), "English"),
            embedded(1, Some("fr"), "Français"),
            embedded(2, Some("tr"), "Türkçe"),
        ]
        .into_iter()
        .collect();
        let prefs = SubtitlePreferences::new(Some(tag("tr")), Some(tag("en")));
        let menu = project(&catalog, &prefs);
        assert_eq!(
            groups(&menu),
            [
                MenuGroup::Closed,
                MenuGroup::Language(tag("tr")),
                MenuGroup::Language(tag("en")),
                MenuGroup::Language(tag("fr")),
            ]
        );
    }

    #[test]
    fn a_preferred_language_without_sources_shows_no_group() {
        let catalog: SubtitleSourceCatalog =
            [embedded(0, Some("en"), "English")].into_iter().collect();
        let prefs = SubtitlePreferences::new(Some(tag("de")), Some(tag("en")));
        let menu = project(&catalog, &prefs);
        assert_eq!(
            groups(&menu),
            [MenuGroup::Closed, MenuGroup::Language(tag("en"))]
        );
    }

    #[test]
    fn unknown_language_group_is_always_last() {
        let catalog: SubtitleSourceCatalog = [
            embedded(0, None, "Track 1"),
            embedded(1, Some("tr"), "Türkçe"),
            embedded(2, Some("en"), "English"),
        ]
        .into_iter()
        .collect();
        let prefs = SubtitlePreferences::new(Some(tag("tr")), None);
        let menu = project(&catalog, &prefs);
        assert_eq!(
            groups(&menu).last(),
            Some(&MenuGroup::UnknownLanguage),
            "Dil Belirsiz stays behind every resolved language"
        );
        assert_eq!(groups(&menu).len(), 4);
    }

    #[test]
    fn user_sources_stay_in_their_own_group_whatever_their_language() {
        let catalog: SubtitleSourceCatalog = [
            SubtitleSource::new(user_id(1), Some(tag("en")), "Movie.en.srt"),
            SubtitleSource::new(user_id(2), None, "unknown.srt"),
            embedded(0, Some("en"), "English"),
        ]
        .into_iter()
        .collect();
        let menu = project(&catalog, &SubtitlePreferences::none());
        assert_eq!(
            groups(&menu),
            [
                MenuGroup::Closed,
                MenuGroup::UserSubtitles,
                MenuGroup::Language(tag("en")),
            ],
            "a user file with no language does not open a Dil Belirsiz group"
        );
        assert_eq!(
            menu.section(&MenuGroup::UserSubtitles)
                .map(|s| s.entries.len()),
            Some(2)
        );
    }

    #[test]
    fn ai_output_sits_in_its_target_language_group() {
        let origin = SubtitleSourceId::embedded(0);
        let catalog: SubtitleSourceCatalog = [
            embedded(0, Some("en"), "English"),
            SubtitleSource::new(
                SubtitleSourceId::ai(&origin, &tag("tr")),
                Some(tag("tr")),
                "AI Türkçe",
            ),
            embedded(1, Some("tr"), "Türkçe"),
        ]
        .into_iter()
        .collect();
        let menu = project(&catalog, &SubtitlePreferences::none());

        // No group is specific to AI: the artifact is an ordinary entry of the
        // Turkish group, marked only by its kind (§8, §9 — ADR-0010 Karar 8).
        assert_eq!(
            groups(&menu),
            [
                MenuGroup::Closed,
                MenuGroup::Language(tag("en")),
                MenuGroup::Language(tag("tr")),
            ]
        );
        let turkish = menu
            .section(&MenuGroup::Language(tag("tr")))
            .expect("Turkish group");
        let kinds: Vec<SubtitleSourceKind> = turkish.entries.iter().map(|e| e.kind()).collect();
        assert_eq!(
            kinds,
            [SubtitleSourceKind::Ai, SubtitleSourceKind::Embedded]
        );
    }

    #[test]
    fn entries_keep_catalog_order_inside_a_group() {
        let catalog: SubtitleSourceCatalog = [
            embedded(0, Some("en"), "English"),
            SubtitleSource::new(
                SubtitleSourceId::opensubtitles("os-1"),
                Some(tag("en")),
                "English — WEB-DL",
            ),
        ]
        .into_iter()
        .collect();
        let menu = project(&catalog, &SubtitlePreferences::none());
        let english = menu
            .section(&MenuGroup::Language(tag("en")))
            .expect("English group");
        let labels: Vec<&str> = english.entries.iter().map(|e| e.label()).collect();
        assert_eq!(labels, ["English", "English — WEB-DL"]);
    }
}
