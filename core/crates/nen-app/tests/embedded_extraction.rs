//! `NEN-044`: an embedded row gets a document only when a caller explicitly
//! asks for one, from [`PlaybackSession::prepare_embedded_document`] — never
//! while the catalog is built, never from `show_source`.
//!
//! Before this call existed, `SubtitleLibrary::document_of` always answered
//! `None` for an embedded row and `crate::translation::prepare` refused every
//! such request with `StartRefusal::NoDocument` — nothing had ever asked the
//! engine (`docs/STATUS.md`'s own record of the gap this closes).
//! `tests/embedded_lazy.rs` (`NEN-023`) is the sibling proof at the catalog
//! layer, with an engine that panics if asked; this file is the positive
//! path plus the two ways an extraction can fail, using an engine that
//! answers instead.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use nen_app::embedded::embedded_sources;
use nen_app::playback::ShellEngine;
use nen_app::session::{EmbeddedDocumentError, PlaybackSession, PrepareOutcome, ShowOutcome};
use nen_app::subtitles::{AddOutcome, SubtitleLibrary};
use nen_app::translation::{self, TranslationMetadataSeed};
use nen_domain::source::{LanguageTag, SubtitleSourceKind};
use nen_ports::playback::{
    Capability, PlaybackError, PlaybackEvent, PlaybackState, TrackDescriptor, TrackId, TrackKind,
    VideoGeometry,
};
use nen_ports::translation::TranslationProvider;
use nen_providers::translation_mock::MockTranslationProvider;
use nen_translate::artifact::{ArtifactId, ArtifactTimestamp, GlossaryIdentity};
use nen_translate::blocks::BlockLayoutConfig;

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nen-044-extraction-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("a temp directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn tag(value: &str) -> LanguageTag {
    LanguageTag::parse(value).expect("a valid language tag")
}

/// A track this engine can extract real text from.
const TEXT_TRACK: TrackId = TrackId(1);
/// A track whose codec carries no text — the container-level classification
/// (`nen-ports::playback::subtitle_carries_text`) has already marked its
/// catalog entry `translatable = false`; the engine is what answers
/// `TrackCarriesNoText` if asked anyway.
const BITMAP_TRACK: TrackId = TrackId(2);
/// A text-coded track whose engine answer this side cannot parse.
const UNPARSEABLE_TRACK: TrackId = TrackId(3);

/// An engine that answers `extract_text` deterministically and counts how
/// many times each track was asked for, so a test can prove a second
/// `prepare_embedded_document` call did not decode again.
#[derive(Default)]
struct ExtractionEngine {
    text_calls: AtomicUsize,
    bitmap_calls: AtomicUsize,
    unparseable_calls: AtomicUsize,
}

impl ExtractionEngine {
    fn text_calls(&self) -> usize {
        self.text_calls.load(Ordering::Acquire)
    }

    fn bitmap_calls(&self) -> usize {
        self.bitmap_calls.load(Ordering::Acquire)
    }
}

impl ShellEngine for ExtractionEngine {
    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability::EmbeddedTextExtraction]
    }

    fn load(&self, _locator: String) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn play(&self) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn pause(&self) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn stop(&self) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn seek(&self, _to_ms: u64) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn position_ms(&self) -> Result<u64, PlaybackError> {
        Ok(0)
    }

    fn duration_ms(&self) -> Result<Option<u64>, PlaybackError> {
        Ok(Some(1_000))
    }

    fn state(&self) -> PlaybackState {
        PlaybackState::Ready
    }

    fn video_geometry(&self) -> Result<Option<VideoGeometry>, PlaybackError> {
        Ok(VideoGeometry::new(160, 90))
    }

    fn tracks(&self, _kind: TrackKind) -> Result<Vec<TrackDescriptor>, PlaybackError> {
        Ok(Vec::new())
    }

    fn select_track(&self, _kind: TrackKind, _track: Option<TrackId>) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn selected_track(&self, _kind: TrackKind) -> Result<Option<TrackId>, PlaybackError> {
        Ok(None)
    }

    fn drain_events(&self) -> Vec<PlaybackEvent> {
        Vec::new()
    }

    fn shutdown(&self) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn set_rate(&self, _rate: f32) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn set_volume(&self, _volume: f32) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn extract_text(&self, track: TrackId) -> Result<String, PlaybackError> {
        match track {
            TEXT_TRACK => {
                self.text_calls.fetch_add(1, Ordering::AcqRel);
                Ok("1\n00:00:01,000 --> 00:00:02,000\nExtracted line.\n".to_string())
            }
            BITMAP_TRACK => {
                self.bitmap_calls.fetch_add(1, Ordering::AcqRel);
                Err(PlaybackError::TrackCarriesNoText)
            }
            UNPARSEABLE_TRACK => {
                self.unparseable_calls.fetch_add(1, Ordering::AcqRel);
                Ok("this is not a subtitle file".to_string())
            }
            _ => Err(PlaybackError::UnknownTrack {
                kind: TrackKind::Subtitle,
            }),
        }
    }

    fn inject_subtitle(&self, _webvtt: String) -> Result<(), PlaybackError> {
        Ok(())
    }

    fn rendered_subtitle_text(&self) -> Result<Option<String>, PlaybackError> {
        Ok(None)
    }

    fn set_subtitle_bottom_inset(&self, _fraction: f32) -> Result<(), PlaybackError> {
        Ok(())
    }
}

/// A library with the three embedded rows above catalogued, and the session
/// wrapping the engine that answers them.
fn embedded_fixture() -> (SubtitleLibrary, PlaybackSession, Arc<ExtractionEngine>) {
    let tracks = vec![
        TrackDescriptor::new(TEXT_TRACK, TrackKind::Subtitle, "subrip")
            .with_language(Some(tag("en"))),
        TrackDescriptor::new(BITMAP_TRACK, TrackKind::Subtitle, "hdmv_pgs_subtitle")
            .with_language(Some(tag("tr"))),
        TrackDescriptor::new(UNPARSEABLE_TRACK, TrackKind::Subtitle, "ass")
            .with_language(Some(tag("de"))),
    ];
    let mut library = SubtitleLibrary::new();
    library.add_embedded(embedded_sources(&tracks));
    let engine = Arc::new(ExtractionEngine::default());
    let session = PlaybackSession::without_pump(Arc::clone(&engine) as Arc<dyn ShellEngine>);
    (library, session, engine)
}

fn token_for(library: &SubtitleLibrary, track: TrackId) -> u32 {
    let id = nen_domain::source::SubtitleSourceId::embedded(track.index());
    library.token_of(&id).expect("the embedded row exists")
}

fn metadata_seed(id: &str) -> TranslationMetadataSeed {
    TranslationMetadataSeed {
        id: ArtifactId::parse(id).expect("a valid artifact id"),
        glossary: GlossaryIdentity::none(),
        media_hash: None,
        created_at: ArtifactTimestamp::from_unix_ms(1_700_000_000_000),
    }
}

#[test]
fn extraction_gives_the_row_a_document_translation_can_use() {
    let (mut library, session, engine) = embedded_fixture();
    let token = token_for(&library, TEXT_TRACK);

    assert_eq!(library.document_of(token), None, "nothing extracted yet");
    let outcome = session
        .prepare_embedded_document(&mut library, token)
        .expect("extraction succeeds");
    assert_eq!(outcome, PrepareOutcome::Ready);
    assert_eq!(engine.text_calls(), 1);

    let document = library
        .document_of(token)
        .expect("the document is attached now");
    assert_eq!(document.len(), 1);

    // The gap `docs/STATUS.md` records: before this call existed, `prepare`
    // refused every embedded request with `NoDocument`. It must not any more.
    let provider = MockTranslationProvider::new();
    translation::prepare(
        &library,
        token,
        tag("tr"),
        BlockLayoutConfig::default(),
        provider.identity(),
        metadata_seed("embedded-extraction"),
    )
    .expect("a document exists, so prepare must not refuse with NoDocument");
}

#[test]
fn a_second_call_does_not_decode_again() {
    let (mut library, session, engine) = embedded_fixture();
    let token = token_for(&library, TEXT_TRACK);

    assert_eq!(
        session
            .prepare_embedded_document(&mut library, token)
            .expect("first extraction succeeds"),
        PrepareOutcome::Ready
    );
    assert_eq!(
        session
            .prepare_embedded_document(&mut library, token)
            .expect("second call is a no-op success"),
        PrepareOutcome::Ready
    );
    assert_eq!(
        engine.text_calls(),
        1,
        "the engine must be asked at most once per row"
    );
}

#[test]
fn a_bitmap_track_is_a_typed_refusal_not_an_empty_document() {
    let (mut library, session, engine) = embedded_fixture();
    let token = token_for(&library, BITMAP_TRACK);

    let error = session
        .prepare_embedded_document(&mut library, token)
        .expect_err("a bitmap track carries no text");
    assert_eq!(
        error,
        EmbeddedDocumentError::Engine(PlaybackError::TrackCarriesNoText)
    );
    assert_eq!(engine.bitmap_calls(), 1);
    assert_eq!(
        library.document_of(token),
        None,
        "a refusal must not leave a document behind"
    );

    // Not a failure to select: showing a bitmap row still just selects the
    // track (ADR-0013 Karar 3) — extraction and selection are independent.
    assert_eq!(
        session
            .show_source(&library, token)
            .expect("show refuses nothing here"),
        ShowOutcome::Shown
    );
}

#[test]
fn text_this_side_cannot_parse_is_a_typed_refusal() {
    let (mut library, session, _engine) = embedded_fixture();
    let token = token_for(&library, UNPARSEABLE_TRACK);

    let error = session
        .prepare_embedded_document(&mut library, token)
        .expect_err("the engine's answer does not parse as SRT");
    assert_eq!(error, EmbeddedDocumentError::Unparseable);
    assert_eq!(library.document_of(token), None);
}

#[test]
fn a_user_file_is_ready_without_touching_the_engine() {
    let media = TempDir::new("user-file");
    let path = media.path().join("Movie.en.srt");
    fs::write(&path, "1\n00:00:01,000 --> 00:00:02,000\nHello.\n").expect("write the fixture");
    let mut library = SubtitleLibrary::new();
    assert_eq!(library.add_file(&path, media.path()), AddOutcome::Added);
    let source_id = library
        .catalog()
        .of_kind(SubtitleSourceKind::User)
        .next()
        .expect("the file was catalogued")
        .id()
        .clone();
    let token = library.token_of(&source_id).expect("a token");

    let engine = Arc::new(ExtractionEngine::default());
    let session = PlaybackSession::without_pump(Arc::clone(&engine) as Arc<dyn ShellEngine>);

    let outcome = session
        .prepare_embedded_document(&mut library, token)
        .expect("a user file already has a document");
    assert_eq!(outcome, PrepareOutcome::Ready);
    assert_eq!(
        engine.text_calls() + engine.bitmap_calls(),
        0,
        "a user file's document never asks the engine for anything"
    );
}

#[test]
fn selecting_an_embedded_row_never_extracts() {
    // `show_source`'s embedded branch selects the engine's own track
    // (ADR-0013 Karar 3) and returns before anything could ask for text.
    // This is the `PlaybackSession`-level twin of `tests/embedded_lazy.rs`'s
    // catalog-level tripwire: selection and extraction are two different
    // calls, and only one of them decodes.
    let (library, session, engine) = embedded_fixture();
    let token = token_for(&library, TEXT_TRACK);

    assert_eq!(
        session
            .show_source(&library, token)
            .expect("show refuses nothing here"),
        ShowOutcome::Shown
    );
    assert_eq!(engine.text_calls(), 0, "selection alone must not extract");
}

#[test]
fn an_unusable_token_is_reported_without_touching_the_engine() {
    let (mut library, session, engine) = embedded_fixture();

    let outcome = session
        .prepare_embedded_document(&mut library, 9_999)
        .expect("an unknown token is Unusable, not an error");
    assert_eq!(outcome, PrepareOutcome::Unusable);
    assert_eq!(engine.text_calls() + engine.bitmap_calls(), 0);
}
