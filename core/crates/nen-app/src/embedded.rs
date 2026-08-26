//! Embedded tracks, as catalog entries (NEN-023).
//!
//! The engine reports [`TrackDescriptor`]s; the catalog holds
//! [`SubtitleSource`]s. This module is the one place the two meet, and it lives
//! here rather than in `nen-catalog` because ADR-0006 gives that crate
//! `domain`, `subtitle` and `identity` — not `ports`. Turning what a device
//! engine says into a use-case value is exactly this layer's job.
//!
//! # Why the mapping is an identity, not a lookup table
//!
//! [`SubtitleSourceId::embedded`] is documented as taking "the engine's index
//! for a track", and [`TrackId`] as being the number
//! `SubtitleSourceId::embedded` wants. So the two agree by construction and
//! [`track_of`] can invert [`embedded_sources`] without either side keeping
//! state. That matters at selection time: the menu hands back a
//! `SubtitleSourceId` and something has to call `select_track` with a
//! `TrackId`.
//!
//! # Nothing here decodes anything (§7)
//!
//! Building the catalog reads metadata only. Text extraction is a separate,
//! lazy operation that does not exist yet (NEN-044); `tests/embedded_lazy.rs`
//! proves this path never reaches for it, using an engine that panics if asked.

use nen_domain::source::{SubtitleSource, SubtitleSourceId};
use nen_ports::playback::{TrackDescriptor, TrackId, TrackKind};

/// Catalog entries for the embedded subtitle tracks of a loaded medium.
///
/// Audio tracks are skipped: they are selectable (the port enumerates them) but
/// they are not subtitle *sources*, and §7's catalog holds sources.
///
/// Order is the engine's order, which is the container's. ADR-0010 Karar 5 says
/// catalog order carries no priority, so preserving it is free and reordering
/// would only invent one.
pub fn embedded_sources(tracks: &[TrackDescriptor]) -> Vec<SubtitleSource> {
    tracks
        .iter()
        .filter(|track| track.kind() == TrackKind::Subtitle)
        .map(|track| {
            SubtitleSource::new(
                SubtitleSourceId::embedded(track.id().index()),
                track.language().cloned(),
                // The container's own title, or nothing.
                //
                // Nothing is substituted for a titleless track on purpose: §8's
                // menu shows "English" / "Türkçe" for exactly that case, and
                // ADR-0010 Karar 7 puts a language's visible name in the UI, in
                // that language's own words. Inventing a label here would mean
                // inventing it in one language and then showing it in every
                // other one.
                track.title().unwrap_or_default(),
            )
            // §7: a bitmap track "gösterilebilir fakat çevrilemez olarak
            // işaretlenebilir". The engine already answered which it is, by
            // codec, in `nen-ports`.
            .with_translatable(track.is_text())
        })
        .collect()
}

/// The track a catalog entry refers to, or `None` when the entry is not an
/// embedded one.
///
/// This is what turns "the user picked this menu row" into "select that track".
/// A `None` answer is not an error: a user file or an OpenSubtitles candidate
/// is a perfectly ordinary entry that simply is not a track.
pub fn track_of(id: &SubtitleSourceId) -> Option<TrackId> {
    id.embedded_index().map(TrackId)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nen_domain::source::{LanguageTag, SubtitleSourceKind};

    fn tag(input: &str) -> LanguageTag {
        LanguageTag::parse(input).expect("valid tag")
    }

    fn tracks() -> Vec<TrackDescriptor> {
        vec![
            TrackDescriptor::new(TrackId(0), TrackKind::Audio, "aac").with_default(true),
            TrackDescriptor::new(TrackId(1), TrackKind::Subtitle, "subrip")
                .with_language(Some(tag("en")))
                .with_title(Some("English (SDH)".into())),
            TrackDescriptor::new(TrackId(2), TrackKind::Subtitle, "hdmv_pgs_subtitle")
                .with_language(Some(tag("tr"))),
            TrackDescriptor::new(TrackId(3), TrackKind::Subtitle, "ass"),
        ]
    }

    #[test]
    fn audio_tracks_do_not_become_subtitle_sources() {
        let sources = embedded_sources(&tracks());
        assert_eq!(sources.len(), 3);
        assert!(sources
            .iter()
            .all(|source| source.kind() == SubtitleSourceKind::Embedded));
    }

    #[test]
    fn a_text_track_keeps_its_language_and_title() {
        let sources = embedded_sources(&tracks());
        let english = &sources[0];
        assert_eq!(english.language().map(LanguageTag::as_str), Some("en"));
        assert_eq!(english.label(), "English (SDH)");
        assert!(english.translatable());
    }

    #[test]
    fn a_bitmap_track_is_catalogued_but_not_translatable() {
        // Both halves matter: §7 says such a track may be shown, and the point
        // of the mark is that it is still there to be shown.
        let sources = embedded_sources(&tracks());
        let bitmap = &sources[1];
        assert_eq!(bitmap.language().map(LanguageTag::as_str), Some("tr"));
        assert!(!bitmap.translatable());
    }

    #[test]
    fn a_titleless_track_carries_an_empty_label() {
        // The UI fills this in with the language's endonym (ADR-0010 Karar 7).
        let sources = embedded_sources(&tracks());
        assert_eq!(sources[2].label(), "");
        assert_eq!(sources[2].language(), None);
    }

    #[test]
    fn every_source_resolves_back_to_the_track_it_came_from() {
        let tracks = tracks();
        let sources = embedded_sources(&tracks);
        let subtitles: Vec<&TrackDescriptor> = tracks
            .iter()
            .filter(|t| t.kind() == TrackKind::Subtitle)
            .collect();

        assert_eq!(sources.len(), subtitles.len());
        for (source, track) in sources.iter().zip(subtitles) {
            assert_eq!(track_of(source.id()), Some(track.id()));
        }
    }

    #[test]
    fn a_source_that_is_not_a_track_resolves_to_nothing() {
        for id in [
            SubtitleSourceId::user([7u8; 32]),
            SubtitleSourceId::opensubtitles("public-id"),
            SubtitleSourceId::ai(&SubtitleSourceId::embedded(1), &tag("tr")),
        ] {
            assert_eq!(track_of(&id), None, "{id:?} resolved to a track");
        }
    }

    #[test]
    fn no_tracks_means_no_entries_rather_than_a_placeholder() {
        assert!(embedded_sources(&[]).is_empty());
    }
}
