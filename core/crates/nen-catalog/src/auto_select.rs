//! Which subtitle, if any, is showing when playback starts (ADR-0010 Karar 9).

use crate::catalog::SubtitleSourceCatalog;
use nen_domain::source::{SubtitlePreferences, SubtitleSource, SubtitleSourceKind};

/// The kinds auto-selection may pick, most preferred first.
///
/// The full tier order decided by ADR-0010 Karar 9 is
/// `embedded` → `user` → `opensubtitles`; **the third tier is deliberately not
/// in this list yet.** Selecting an OpenSubtitles candidate means downloading
/// it, and §7 says "seçilmeden download yapmama" — so until media identity is
/// measurably accurate enough (NEN-033 · NEN-035 · NEN-036), auto-download
/// would silently fetch the wrong subtitle whenever identification was wrong.
/// Opening that tier is NEN-038's job and needs its own ADR; the order is
/// written out here in full so that day is one line, not a fresh argument.
///
/// `ai` is absent for a different reason, and permanently: §9 requires
/// translation to start only on an explicit command, and auto-opening an
/// existing artifact would bind the user to AI output without asking.
///
/// The tiers are written here rather than inferred from catalog order because
/// this is the one place a priority exists — menu order carries none (Karar 5),
/// and leaving a functional decision to an implicit collection order would make
/// that claim untrue.
pub const AUTO_SELECTABLE_KINDS: &[SubtitleSourceKind] =
    &[SubtitleSourceKind::Embedded, SubtitleSourceKind::User];

/// Picks the subtitle to show when playback starts, or `None` for "off".
///
/// Only the preferred languages are considered, primary before secondary; no
/// other language is ever opened automatically. Within a language the tier
/// order of [`AUTO_SELECTABLE_KINDS`] decides, and entries of the same kind fall
/// back to catalog order.
///
/// This function performs **no** I/O and triggers nothing: it names a source
/// that is already playable. Loading and rendering it is M3's job.
///
/// Per §9 the returned source becomes both the displayed subtitle and the source
/// an explicit "translate with AI" command would use — auto-selection carries
/// exactly the meaning a manual selection would, and the user can change it
/// from the menu.
pub fn auto_selection<'a>(
    catalog: &'a SubtitleSourceCatalog,
    preferences: &SubtitlePreferences,
) -> Option<&'a SubtitleSource> {
    for language in preferences.ordered() {
        for kind in AUTO_SELECTABLE_KINDS {
            let found = catalog
                .sources()
                .find(|s| s.kind() == *kind && s.language() == Some(language));
            if found.is_some() {
                return found;
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use nen_domain::source::{LanguageTag, SubtitleSourceId};

    fn tag(s: &str) -> LanguageTag {
        LanguageTag::parse(s).expect("valid tag")
    }

    fn embedded(index: u32, lang: &str, label: &str) -> SubtitleSource {
        SubtitleSource::new(
            SubtitleSourceId::embedded(index),
            Some(tag(lang)),
            label.to_owned(),
        )
    }

    fn user(seed: u8, lang: &str, label: &str) -> SubtitleSource {
        SubtitleSource::new(
            SubtitleSourceId::user([seed; 32]),
            Some(tag(lang)),
            label.to_owned(),
        )
    }

    fn opensubtitles(id: &str, lang: &str) -> SubtitleSource {
        SubtitleSource::new(
            SubtitleSourceId::opensubtitles(id),
            Some(tag(lang)),
            "candidate".to_owned(),
        )
    }

    fn ai(lang: &str) -> SubtitleSource {
        let origin = SubtitleSourceId::embedded(0);
        SubtitleSource::new(
            SubtitleSourceId::ai(&origin, &tag(lang)),
            Some(tag(lang)),
            "AI".to_owned(),
        )
    }

    #[test]
    fn embedded_wins_over_a_user_file_in_the_same_language() {
        let catalog: SubtitleSourceCatalog =
            [user(1, "tr", "Movie.tr.srt"), embedded(0, "tr", "Türkçe")]
                .into_iter()
                .collect();
        let prefs = SubtitlePreferences::new(Some(tag("tr")), None);
        let picked = auto_selection(&catalog, &prefs).expect("a pick");
        assert_eq!(picked.kind(), SubtitleSourceKind::Embedded);
    }

    #[test]
    fn a_user_file_is_picked_when_no_embedded_track_matches() {
        let catalog: SubtitleSourceCatalog =
            [embedded(0, "en", "English"), user(1, "tr", "Movie.tr.srt")]
                .into_iter()
                .collect();
        let prefs = SubtitlePreferences::new(Some(tag("tr")), None);
        let picked = auto_selection(&catalog, &prefs).expect("a pick");
        assert_eq!(picked.kind(), SubtitleSourceKind::User);
    }

    #[test]
    fn the_secondary_preference_is_used_when_the_primary_has_nothing() {
        let catalog: SubtitleSourceCatalog = [embedded(0, "en", "English")].into_iter().collect();
        let prefs = SubtitlePreferences::new(Some(tag("tr")), Some(tag("en")));
        let picked = auto_selection(&catalog, &prefs).expect("a pick");
        assert_eq!(picked.language(), Some(&tag("en")));
    }

    #[test]
    fn opensubtitles_is_never_selected_automatically() {
        // Negative control for §7 "seçilmeden download yapmama": the only
        // Turkish source is a candidate that would have to be downloaded, so
        // playback starts with subtitles off rather than fetching it.
        let catalog: SubtitleSourceCatalog = [opensubtitles("os-1", "tr")].into_iter().collect();
        let prefs = SubtitlePreferences::new(Some(tag("tr")), None);
        assert!(auto_selection(&catalog, &prefs).is_none());
        assert!(!AUTO_SELECTABLE_KINDS.contains(&SubtitleSourceKind::OpenSubtitles));
    }

    #[test]
    fn ai_output_is_never_selected_automatically() {
        // Negative control for §9: translation, and being shown a translation,
        // both wait for an explicit command.
        let catalog: SubtitleSourceCatalog = [ai("tr")].into_iter().collect();
        let prefs = SubtitlePreferences::new(Some(tag("tr")), None);
        assert!(auto_selection(&catalog, &prefs).is_none());
        assert!(!AUTO_SELECTABLE_KINDS.contains(&SubtitleSourceKind::Ai));
    }

    #[test]
    fn no_language_outside_the_preferences_is_selected() {
        // Negative control for Karar 9 item 1: a catalog full of playable
        // tracks stays off when none of them is a preferred language.
        let catalog: SubtitleSourceCatalog = [
            embedded(0, "en", "English"),
            embedded(1, "fr", "Français"),
            user(1, "de", "Movie.de.srt"),
        ]
        .into_iter()
        .collect();
        let prefs = SubtitlePreferences::new(Some(tag("tr")), Some(tag("es")));
        assert!(auto_selection(&catalog, &prefs).is_none());
    }

    #[test]
    fn nothing_is_selected_without_preferences() {
        let catalog: SubtitleSourceCatalog = [embedded(0, "tr", "Türkçe")].into_iter().collect();
        assert!(auto_selection(&catalog, &SubtitlePreferences::none()).is_none());
    }

    #[test]
    fn an_unknown_language_source_is_never_selected() {
        let catalog: SubtitleSourceCatalog = [SubtitleSource::new(
            SubtitleSourceId::embedded(0),
            None,
            "Track 1",
        )]
        .into_iter()
        .collect();
        let prefs = SubtitlePreferences::new(Some(tag("tr")), None);
        assert!(auto_selection(&catalog, &prefs).is_none());
    }

    #[test]
    fn same_kind_falls_back_to_catalog_order() {
        let catalog: SubtitleSourceCatalog = [
            embedded(3, "tr", "Türkçe forced"),
            embedded(1, "tr", "Türkçe full"),
        ]
        .into_iter()
        .collect();
        let prefs = SubtitlePreferences::new(Some(tag("tr")), None);
        let picked = auto_selection(&catalog, &prefs).expect("a pick");
        assert_eq!(picked.label(), "Türkçe forced");
    }
}
