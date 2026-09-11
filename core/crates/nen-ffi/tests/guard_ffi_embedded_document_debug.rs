//! K23 guard for the embedded-extraction FFI surface (`NEN-044`).
//!
//! `FfiEmbeddedDocumentError` is flat and payload-free, the same convention
//! `guard_ffi_translation_debug.rs` already holds `FfiTranslationStartError`
//! and `FfiTranslationError` to: nothing here has a field capable of holding
//! the engine's answer, which is subtitle dialogue (K23 #4). This proves it
//! two ways — every variant prints only its own name, and a real extraction
//! whose engine answer carries a sentinel never lets that sentinel reach the
//! outcome or the error `prepare_embedded_document` hands back.

use nen_ffi::playback::{
    FfiCapability, FfiPlaybackError, FfiPlaybackEvent, FfiPlaybackState, FfiTrackDescriptor,
    FfiTrackKind, FfiVideoGeometry, ForeignPlaybackEngine,
};
use nen_ffi::session::{FfiEmbeddedDocumentError, FfiPlaybackSession};
use nen_ffi::subtitles::FfiSubtitleLibrary;
use std::sync::Arc;

/// Long and distinctive so a partial leak is caught as surely as a whole one.
const DIALOGUE_SENTINEL: &str = "Zzqxvunlogged-embedded-dialogue";

fn forbidden(output: &str) {
    assert!(
        !output.contains(DIALOGUE_SENTINEL),
        "embedded-extraction FFI output leaked the dialogue sentinel: {output}"
    );
}

/// An engine whose extraction answer is unparseable text carrying the
/// sentinel — the one shape of real data that reaches this gate.
struct SentinelEngine;

impl ForeignPlaybackEngine for SentinelEngine {
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
    fn extract_text(&self, _track: u32) -> Result<String, FfiPlaybackError> {
        // Not valid SRT: this is dialogue this side must refuse to parse
        // rather than reflect back — the exact input `Unparseable` exists for.
        Ok(format!("garbled: {DIALOGUE_SENTINEL}"))
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

#[test]
fn every_variant_prints_only_its_own_name() {
    let variants = [
        FfiEmbeddedDocumentError::Unsupported,
        FfiEmbeddedDocumentError::ReentrantCall,
        FfiEmbeddedDocumentError::NotLoaded,
        FfiEmbeddedDocumentError::ShutDown,
        FfiEmbeddedDocumentError::UnknownTrack,
        FfiEmbeddedDocumentError::TrackCarriesNoText,
        FfiEmbeddedDocumentError::EngineFailure,
        FfiEmbeddedDocumentError::Unparseable,
    ];
    for variant in variants {
        let combined = format!("{variant:?} {variant}");
        forbidden(&combined);
        assert!(combined.contains(&format!("{variant:?}")), "{combined}");
    }
}

#[test]
fn a_real_extraction_never_lets_the_engines_answer_reach_the_error() {
    let library = Arc::new(FfiSubtitleLibrary::new());
    library.add_embedded(vec![FfiTrackDescriptor {
        id: 1,
        kind: FfiTrackKind::Subtitle,
        language: Some("en".to_string()),
        codec: "subrip".to_string(),
        is_default: false,
        title: None,
    }]);
    let token = library
        .menu(None, None)
        .into_iter()
        .flat_map(|section| section.entries)
        .find(|entry| library.embedded_track_of(entry.token) == Some(1))
        .expect("the embedded row exists")
        .token;

    let session = FfiPlaybackSession::new(Arc::new(SentinelEngine));
    let error = session
        .prepare_embedded_document(Arc::clone(&library), token)
        .expect_err("the engine's answer is not valid SRT");
    assert_eq!(error, FfiEmbeddedDocumentError::Unparseable);
    forbidden(&format!("{error:?} {error}"));
}
