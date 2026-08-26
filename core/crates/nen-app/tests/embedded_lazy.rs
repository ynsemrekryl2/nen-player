//! NEN-023 DoD #4: building the catalog extracts no track text (§7).
//!
//! "Cataloguing is not downloading or extracting" is the rule the whole
//! subtitle pipeline rests on — the menu has to exist while the medium is
//! already playing, and pulling every embedded track's text up front is exactly
//! the eager work §7 forbids.
//!
//! The check is structural rather than textual: the engine driven here
//! **panics** if anything asks it to decode. A test that merely asserted "the
//! catalog is correct" would pass just as happily with an eager implementation
//! behind it.
//!
//! `extract_text` has no real implementation yet (NEN-044 owns that). This test
//! is what keeps the rule true once it does: the day an adapter can extract,
//! this path still must not.

use nen_app::embedded::embedded_sources;
use nen_catalog::{project, SubtitleSourceCatalog};
use nen_domain::source::{LanguageTag, SubtitlePreferences};
use nen_domain::subtitle::SubtitleDocument;
use nen_ports::playback::{
    Capabilities, Capability, EventQueue, MediaSource, PlaybackEngine, PlaybackError,
    PlaybackState, TrackDescriptor, TrackId, TrackKind,
};
use std::cell::Cell;
use std::time::Duration;

/// An engine that answers metadata and detonates on decoding.
///
/// It declares `EmbeddedTextExtraction`, so nothing is protected here by the
/// capability being absent: if the catalog path wanted text, it could have it.
struct TripwireEngine {
    track_queries: Cell<usize>,
    events: EventQueue,
}

impl TripwireEngine {
    fn new() -> Self {
        Self {
            track_queries: Cell::new(0),
            events: EventQueue::default(),
        }
    }

    fn track_queries(&self) -> usize {
        self.track_queries.get()
    }
}

impl PlaybackEngine for TripwireEngine {
    fn capabilities(&self) -> Capabilities {
        Capabilities::new([Capability::EmbeddedTextExtraction])
    }

    fn load(&mut self, _source: &MediaSource) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn play(&mut self) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn pause(&mut self) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn stop(&mut self) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn seek(&mut self, _to: Duration) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn position(&self) -> Result<Duration, PlaybackError> {
        Ok(Duration::ZERO)
    }

    fn duration(&self) -> Result<Option<Duration>, PlaybackError> {
        Ok(Some(Duration::from_secs(30)))
    }

    fn state(&self) -> PlaybackState {
        PlaybackState::Playing
    }

    fn tracks(&self, kind: TrackKind) -> Result<Vec<TrackDescriptor>, PlaybackError> {
        self.track_queries.set(self.track_queries.get() + 1);
        let tag = |input: &str| LanguageTag::parse(input).ok();
        Ok(match kind {
            TrackKind::Subtitle => vec![
                TrackDescriptor::new(TrackId(1), TrackKind::Subtitle, "subrip")
                    .with_language(tag("en"))
                    .with_title(Some("English".into())),
                TrackDescriptor::new(TrackId(2), TrackKind::Subtitle, "hdmv_pgs_subtitle")
                    .with_language(tag("tr")),
            ],
            TrackKind::Audio => vec![TrackDescriptor::new(TrackId(0), TrackKind::Audio, "aac")],
        })
    }

    fn select_track(
        &mut self,
        _kind: TrackKind,
        _track: Option<TrackId>,
    ) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn selected_track(&self, _kind: TrackKind) -> Result<Option<TrackId>, PlaybackError> {
        Ok(None)
    }

    fn events(&mut self) -> &mut EventQueue {
        &mut self.events
    }

    fn shutdown(&mut self) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn extract_text(&mut self, track: TrackId) -> Result<String, PlaybackError> {
        panic!("the catalog path decoded track {track} (§7: extraction is lazy)");
    }

    fn inject_subtitle(&mut self, _document: &SubtitleDocument) -> Result<(), PlaybackError> {
        panic!("the catalog path pushed a document into the engine");
    }
}

#[test]
fn cataloguing_embedded_tracks_decodes_nothing() {
    let mut engine = TripwireEngine::new();
    engine
        .load(&MediaSource::new("fixtures/media/contract-clip.mkv"))
        .expect("load");
    engine.play().expect("play");

    // The whole path a shell walks when a medium opens: ask for tracks, turn
    // them into catalog entries, project the menu.
    let tracks = engine.tracks(TrackKind::Subtitle).expect("tracks");
    let catalog: SubtitleSourceCatalog = embedded_sources(&tracks).into_iter().collect();
    let menu = project(&catalog, &SubtitlePreferences::none());

    // Not vacuous: the tripwire is only meaningful if the path really ran.
    assert_eq!(engine.track_queries(), 1, "the engine was never asked");
    assert_eq!(catalog.len(), 2);
    assert!(
        menu.sections.len() >= 3,
        "expected Kapalı + two language groups, got {:?}",
        menu.groups().collect::<Vec<_>>()
    );
    assert!(
        engine
            .capabilities()
            .contains(Capability::EmbeddedTextExtraction),
        "the engine must be able to extract, or the tripwire proves nothing"
    );
}

#[test]
fn the_tripwire_fires_when_something_does_decode() {
    // The control: the same engine, asked directly. Without this, a tripwire
    // that could never fire would look exactly like a passing test.
    let mut engine = TripwireEngine::new();
    let fired = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = engine.extract_text(TrackId(1));
    }));
    assert!(fired.is_err(), "the tripwire never fired");
}
