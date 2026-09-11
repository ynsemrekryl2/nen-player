//! `NEN-044` at the FFI boundary: `FfiPlaybackSession::prepare_embedded_
//! document` forwards to `nen_app::session::PlaybackSession` and translates
//! the answer, using a foreign engine test double — the same shape
//! `nen-app/tests/embedded_extraction.rs` drives at the Rust level, proved
//! again here because this gate has its own translation from `PlaybackError`
//! to the flat `FfiEmbeddedDocumentError` that a real Swift adapter would
//! actually receive.

use nen_ffi::playback::{
    FfiCapability, FfiPlaybackError, FfiPlaybackEvent, FfiPlaybackState, FfiTrackDescriptor,
    FfiTrackKind, FfiVideoGeometry, ForeignPlaybackEngine,
};
use nen_ffi::session::{FfiEmbeddedDocumentError, FfiPlaybackSession, FfiPrepareOutcome};
use nen_ffi::subtitles::FfiSubtitleLibrary;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A track this engine can extract real text from.
const TEXT_TRACK: u32 = 1;
/// A track whose codec carries no text.
const BITMAP_TRACK: u32 = 2;
/// A text-coded track whose engine answer this side cannot parse.
const UNPARSEABLE_TRACK: u32 = 3;
/// An id no track has.
const UNKNOWN_TRACK: u32 = 9_999;

/// A foreign engine that answers `extract_text` deterministically and counts
/// how many times each outcome was asked for.
#[derive(Default)]
struct FakeForeignEngine {
    text_calls: AtomicUsize,
}

impl FakeForeignEngine {
    fn text_calls(&self) -> usize {
        self.text_calls.load(Ordering::Acquire)
    }
}

impl ForeignPlaybackEngine for FakeForeignEngine {
    fn capabilities(&self) -> Vec<FfiCapability> {
        vec![FfiCapability::EmbeddedTextExtraction]
    }

    fn load(&self, _locator: String) -> Result<(), FfiPlaybackError> {
        Ok(())
    }

    fn play(&self) -> Result<(), FfiPlaybackError> {
        Ok(())
    }

    fn pause(&self) -> Result<(), FfiPlaybackError> {
        Ok(())
    }

    fn stop(&self) -> Result<(), FfiPlaybackError> {
        Ok(())
    }

    fn seek(&self, _to_ms: u64) -> Result<(), FfiPlaybackError> {
        Ok(())
    }

    fn position_ms(&self) -> Result<u64, FfiPlaybackError> {
        Ok(0)
    }

    fn duration_ms(&self) -> Result<Option<u64>, FfiPlaybackError> {
        Ok(Some(1_000))
    }

    fn state(&self) -> FfiPlaybackState {
        FfiPlaybackState::Ready
    }

    fn video_geometry(&self) -> Result<Option<FfiVideoGeometry>, FfiPlaybackError> {
        Ok(None)
    }

    fn tracks(&self, _kind: FfiTrackKind) -> Result<Vec<FfiTrackDescriptor>, FfiPlaybackError> {
        Ok(Vec::new())
    }

    fn select_track(
        &self,
        _kind: FfiTrackKind,
        _track: Option<u32>,
    ) -> Result<(), FfiPlaybackError> {
        Ok(())
    }

    fn selected_track(&self, _kind: FfiTrackKind) -> Result<Option<u32>, FfiPlaybackError> {
        Ok(None)
    }

    fn drain_events(&self) -> Vec<FfiPlaybackEvent> {
        Vec::new()
    }

    fn shutdown(&self) -> Result<(), FfiPlaybackError> {
        Ok(())
    }

    fn set_rate(&self, _rate: f32) -> Result<(), FfiPlaybackError> {
        Ok(())
    }

    fn set_volume(&self, _volume: f32) -> Result<(), FfiPlaybackError> {
        Ok(())
    }

    fn extract_text(&self, track: u32) -> Result<String, FfiPlaybackError> {
        match track {
            TEXT_TRACK => {
                self.text_calls.fetch_add(1, Ordering::AcqRel);
                Ok("1\n00:00:01,000 --> 00:00:02,000\nExtracted line.\n".to_string())
            }
            BITMAP_TRACK => Err(FfiPlaybackError::TrackCarriesNoText),
            UNPARSEABLE_TRACK => Ok("not a subtitle file".to_string()),
            _ => Err(FfiPlaybackError::UnknownTrack {
                kind: FfiTrackKind::Subtitle,
            }),
        }
    }

    fn inject_subtitle(&self, _webvtt: String) -> Result<(), FfiPlaybackError> {
        Ok(())
    }

    fn rendered_subtitle_text(&self) -> Result<Option<String>, FfiPlaybackError> {
        Ok(None)
    }

    fn set_subtitle_bottom_inset(&self, _fraction: f32) -> Result<(), FfiPlaybackError> {
        Ok(())
    }
}

fn track(id: u32, codec: &str) -> FfiTrackDescriptor {
    FfiTrackDescriptor {
        id,
        kind: FfiTrackKind::Subtitle,
        language: Some("en".to_string()),
        codec: codec.to_string(),
        is_default: false,
        title: None,
    }
}

/// A library with the three embedded rows catalogued, and the session
/// wrapping a fresh [`FakeForeignEngine`] — handed back separately so a test
/// can read its call counters after the session has taken ownership of it as
/// a trait object.
fn fixture() -> (
    Arc<FfiSubtitleLibrary>,
    FfiPlaybackSession,
    Arc<FakeForeignEngine>,
    u32,
    u32,
    u32,
    u32,
) {
    let library = Arc::new(FfiSubtitleLibrary::new());
    library.add_embedded(vec![
        track(TEXT_TRACK, "subrip"),
        track(BITMAP_TRACK, "hdmv_pgs_subtitle"),
        track(UNPARSEABLE_TRACK, "ass"),
    ]);
    let text_token = token_for(&library, TEXT_TRACK);
    let bitmap_token = token_for(&library, BITMAP_TRACK);
    let unparseable_token = token_for(&library, UNPARSEABLE_TRACK);
    let engine = Arc::new(FakeForeignEngine::default());
    let session = FfiPlaybackSession::new(Arc::clone(&engine) as Arc<dyn ForeignPlaybackEngine>);
    (
        library,
        session,
        engine,
        text_token,
        bitmap_token,
        unparseable_token,
        UNKNOWN_TRACK,
    )
}

fn token_for(library: &FfiSubtitleLibrary, track_id: u32) -> u32 {
    library
        .menu(None, None)
        .into_iter()
        .flat_map(|section| section.entries)
        .find(|entry| library.embedded_track_of(entry.token) == Some(track_id))
        .expect("the embedded row exists")
        .token
}

#[test]
fn extraction_attaches_a_document() {
    let (library, session, engine, text_token, ..) = fixture();
    let outcome = session
        .prepare_embedded_document(Arc::clone(&library), text_token)
        .expect("extraction succeeds");
    assert_eq!(outcome, FfiPrepareOutcome::Ready);
    assert_eq!(engine.text_calls(), 1);
}

#[test]
fn a_bitmap_track_is_a_flat_typed_refusal() {
    let (library, session, _engine, _text, bitmap_token, ..) = fixture();
    let error = session
        .prepare_embedded_document(Arc::clone(&library), bitmap_token)
        .expect_err("a bitmap track carries no text");
    assert_eq!(error, FfiEmbeddedDocumentError::TrackCarriesNoText);
}

#[test]
fn unparseable_text_is_a_flat_typed_refusal() {
    let (library, session, _engine, _text, _bitmap, unparseable_token, _unknown) = fixture();
    let error = session
        .prepare_embedded_document(Arc::clone(&library), unparseable_token)
        .expect_err("the engine's answer does not parse as SRT");
    assert_eq!(error, FfiEmbeddedDocumentError::Unparseable);
}

#[test]
fn an_unknown_token_is_unusable_not_an_error() {
    let (library, session, .., unknown_token) = fixture();
    let outcome = session
        .prepare_embedded_document(Arc::clone(&library), unknown_token)
        .expect("an unknown token is Unusable, not an error");
    assert_eq!(outcome, FfiPrepareOutcome::Unusable);
}

#[test]
fn a_second_call_does_not_decode_again() {
    let (library, session, engine, text_token, ..) = fixture();
    session
        .prepare_embedded_document(Arc::clone(&library), text_token)
        .expect("first extraction succeeds");
    session
        .prepare_embedded_document(Arc::clone(&library), text_token)
        .expect("second call is a no-op success");
    assert_eq!(
        engine.text_calls(),
        1,
        "the engine must be asked at most once per row"
    );
}
