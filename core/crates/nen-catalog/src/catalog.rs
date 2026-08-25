//! The single catalog every subtitle source lives in (product-spec §7).

use nen_domain::source::{SubtitleSource, SubtitleSourceId, SubtitleSourceKind};
use std::fmt;

/// All subtitle sources known for the media being played, in the order they
/// were discovered.
///
/// **Dedup is by [`SubtitleSourceId`]** (ADR-0010 Karar 2). Content-level dedup
/// would need the subtitle text, and §7 forbids fetching it just to catalogue —
/// so two entries are "the same source" when their metadata identity matches,
/// and never because their text happens to match.
///
/// Insertion order is meaningful but carries **no priority**: it is the order
/// entries appear inside their menu group (Karar 5). The one place a priority
/// exists is [`crate::auto_selection`], and it is written out there explicitly
/// rather than being inferred from this order.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct SubtitleSourceCatalog {
    entries: Vec<SubtitleSource>,
}

impl SubtitleSourceCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a source, or updates the one already carrying this identity.
    ///
    /// An upsert keeps the entry **in place**. That matters once NEN-020 fills
    /// in a language that was unknown at discovery time: re-inserting must not
    /// move the row out from under a user who is looking at the menu.
    ///
    /// Returns `true` when an existing entry was replaced.
    pub fn insert(&mut self, source: SubtitleSource) -> bool {
        match self.entries.iter_mut().find(|e| e.id() == source.id()) {
            Some(existing) => {
                *existing = source;
                true
            }
            None => {
                self.entries.push(source);
                false
            }
        }
    }

    /// Every source, in insertion order.
    pub fn sources(&self) -> impl Iterator<Item = &SubtitleSource> {
        self.entries.iter()
    }

    /// Sources of one kind, in insertion order.
    pub fn of_kind(&self, kind: SubtitleSourceKind) -> impl Iterator<Item = &SubtitleSource> {
        self.entries.iter().filter(move |e| e.kind() == kind)
    }

    pub fn get(&self, id: &SubtitleSourceId) -> Option<&SubtitleSource> {
        self.entries.iter().find(|e| e.id() == id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl FromIterator<SubtitleSource> for SubtitleSourceCatalog {
    fn from_iter<T: IntoIterator<Item = SubtitleSource>>(iter: T) -> Self {
        let mut catalog = Self::new();
        for source in iter {
            catalog.insert(source);
        }
        catalog
    }
}

impl fmt::Debug for SubtitleSourceCatalog {
    /// Prints the entry count only.
    ///
    /// Every entry carries a display label that is often a private filename
    /// (K23 #8), so the container must not delegate to its entries' `Debug`
    /// wholesale — even though those are themselves redacted, a count is all
    /// this type has any reason to say.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubtitleSourceCatalog")
            .field("source_count", &self.entries.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nen_domain::source::LanguageTag;

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

    fn user_id(seed: u8) -> SubtitleSourceId {
        SubtitleSourceId::user([seed; 32])
    }

    #[test]
    fn same_identity_is_stored_once() {
        let mut catalog = SubtitleSourceCatalog::new();
        assert!(!catalog.insert(embedded(0, "en", "English")));
        assert!(catalog.insert(embedded(0, "en", "English")));
        assert_eq!(catalog.len(), 1);
    }

    #[test]
    fn dedup_covers_all_four_kinds() {
        let origin = SubtitleSourceId::embedded(0);
        let cases = [
            SubtitleSourceId::embedded(3),
            user_id(4),
            SubtitleSourceId::opensubtitles("os-public-9911"),
            SubtitleSourceId::ai(&origin, &tag("tr")),
        ];
        for id in cases {
            let mut catalog = SubtitleSourceCatalog::new();
            catalog.insert(SubtitleSource::new(id.clone(), Some(tag("tr")), "x"));
            catalog.insert(SubtitleSource::new(id.clone(), Some(tag("tr")), "x"));
            assert_eq!(catalog.len(), 1, "kind {:?} deduped", id.kind());
        }
    }

    #[test]
    fn upsert_updates_in_place_without_reordering() {
        let mut catalog = SubtitleSourceCatalog::new();
        catalog.insert(embedded(0, "en", "English"));
        catalog.insert(embedded(1, "fr", "Français"));
        catalog.insert(embedded(2, "tr", "Türkçe"));

        // NEN-020 later establishes a language for the first track.
        let updated =
            SubtitleSource::new(SubtitleSourceId::embedded(0), Some(tag("de")), "Deutsch");
        assert!(catalog.insert(updated));

        let order: Vec<&str> = catalog.sources().map(SubtitleSource::label).collect();
        assert_eq!(order, ["Deutsch", "Français", "Türkçe"]);
    }

    #[test]
    fn debug_reports_a_count_not_the_entries() {
        let mut catalog = SubtitleSourceCatalog::new();
        catalog.insert(SubtitleSource::new(
            user_id(7),
            Some(tag("tr")),
            "Inception.tr.srt",
        ));
        let printed = format!("{catalog:?}");
        assert!(!printed.contains("Inception"));
        assert!(printed.contains("source_count"));
    }
}
