//! The `PlaybackEngine` port itself.
//!
//! The operation list is product-spec §4's, unchanged. What ADR-0011 adds is
//! the shape:
//!
//! - **The base is mandatory and not queryable** (Karar 3). Everything on this
//!   trait without a `Capability` in its doc must work on every adapter.
//! - **Four operations are capability-gated.** Their default implementation
//!   returns [`PlaybackError::Unsupported`], so an adapter that does not
//!   declare a capability gets the correct refusal for free and an adapter that
//!   declares one without implementing it is caught by the contract kit.
//! - **Relative seek is not a capability.** It has a default implementation
//!   over `position` + `seek`; an engine with an atomic native seek overrides
//!   it. Making it optional would force the application layer to branch on
//!   something every engine can in fact do.
//! - **Every operation must call [`guard_reentrancy`] first** (Karar 2). The
//!   contract kit checks this for each operation, on every adapter.
//!
//! The engine's *name* appears nowhere in this file and must appear nowhere in
//! the application layer (product-spec §4, `docs/architecture.md`).

use super::capability::{Capabilities, Capability};
use super::error::{Operation, PlaybackError};
use super::event::{guard_reentrancy, EventQueue, PlaybackState};
use super::media::MediaSource;
use super::track::{TrackDescriptor, TrackId, TrackKind};
use nen_domain::subtitle::SubtitleDocument;
use std::time::Duration;

/// A device media engine, behind one contract.
///
/// Implemented by an adapter per platform (libmpv on macOS, later AVPlayer and
/// Media3). The core owns the session and is called back — ADR-0026 yön A — so
/// an adapter reports through [`PlaybackEngine::events`] rather than by calling
/// the shell itself.
pub trait PlaybackEngine {
    /// The optional capabilities this engine provides.
    ///
    /// Must be constant for the lifetime of the engine: the application layer
    /// builds UI affordances from it, and a set that changed underneath would
    /// make a menu lie.
    fn capabilities(&self) -> Capabilities;

    /// Loads a medium. Does not start playback.
    ///
    /// Must return without waiting for the whole medium: product-spec §4
    /// requires media to play before subtitle discovery finishes, and M3's exit
    /// criteria repeat it.
    fn load(&mut self, source: &MediaSource) -> Result<(), PlaybackError>;

    fn play(&mut self) -> Result<(), PlaybackError>;

    fn pause(&mut self) -> Result<(), PlaybackError>;

    /// Stops playback and releases the medium; the engine returns to
    /// [`PlaybackState::Idle`] and can load again.
    fn stop(&mut self) -> Result<(), PlaybackError>;

    /// Seeks to an absolute position.
    ///
    /// Seeking past the end is not an error — it ends playback, and the engine
    /// reports [`PlaybackState::Ended`].
    fn seek(&mut self, to: Duration) -> Result<(), PlaybackError>;

    fn position(&self) -> Result<Duration, PlaybackError>;

    /// The medium's duration, or `None` when it has none — a live stream has no
    /// end to report.
    fn duration(&self) -> Result<Option<Duration>, PlaybackError>;

    fn state(&self) -> PlaybackState;

    /// The embedded tracks of one kind, as metadata only.
    ///
    /// No text is extracted here (§7 lazy); see [`PlaybackEngine::extract_text`].
    fn tracks(&self, kind: TrackKind) -> Result<Vec<TrackDescriptor>, PlaybackError>;

    /// Selects a track, or `None` to select nothing of that kind.
    ///
    /// `None` on [`TrackKind::Subtitle`] is what the menu's `Kapalı` entry does
    /// (§8).
    fn select_track(
        &mut self,
        kind: TrackKind,
        track: Option<TrackId>,
    ) -> Result<(), PlaybackError>;

    /// The currently selected track of a kind, if any.
    fn selected_track(&self, kind: TrackKind) -> Result<Option<TrackId>, PlaybackError>;

    /// The pending event queue (ADR-0011 Karar 1).
    ///
    /// Delivery is `super::event::deliver_all`, not the adapter's job — that is
    /// what keeps the reentrancy mark in one place.
    fn events(&mut self) -> &mut EventQueue;

    /// Releases everything. After this the engine accepts nothing further and
    /// returns [`PlaybackError::ShutDown`].
    ///
    /// Must be idempotent: shutting down twice is not an error, because a
    /// shell cannot always know whether its own teardown already ran.
    fn shutdown(&mut self) -> Result<(), PlaybackError>;

    /// Seeks by a signed offset in milliseconds.
    ///
    /// Not a capability (ADR-0011 Karar 3): the default is exact over
    /// `position` + `seek`, clamping at zero. An adapter with an atomic native
    /// relative seek overrides this.
    fn seek_relative(&mut self, delta_ms: i64) -> Result<(), PlaybackError> {
        guard_reentrancy(Operation::Seek)?;
        let current = self.position()?;
        let target = if delta_ms >= 0 {
            current.saturating_add(Duration::from_millis(delta_ms as u64))
        } else {
            current.saturating_sub(Duration::from_millis(delta_ms.unsigned_abs()))
        };
        self.seek(target)
    }

    /// Sets the playback rate.
    ///
    /// Needs [`Capability::PlaybackRate`].
    fn set_rate(&mut self, rate: f32) -> Result<(), PlaybackError> {
        let _ = rate;
        Err(PlaybackError::Unsupported {
            operation: Operation::SetRate,
            capability: Capability::PlaybackRate,
        })
    }

    /// Sets the output volume, `0.0..=1.0`.
    ///
    /// Needs [`Capability::Volume`].
    fn set_volume(&mut self, volume: f32) -> Result<(), PlaybackError> {
        let _ = volume;
        Err(PlaybackError::Unsupported {
            operation: Operation::SetVolume,
            capability: Capability::Volume,
        })
    }

    /// Extracts the text of an embedded subtitle track.
    ///
    /// Needs [`Capability::EmbeddedTextExtraction`]. Called lazily — on
    /// selection or on a translation request, never while building the catalog
    /// (§7).
    ///
    /// **Security:** the returned text is subtitle dialogue (K23 #4). It may be
    /// parsed and displayed; it may never be logged.
    fn extract_text(&mut self, track: TrackId) -> Result<String, PlaybackError> {
        let _ = track;
        Err(PlaybackError::Unsupported {
            operation: Operation::ExtractText,
            capability: Capability::EmbeddedTextExtraction,
        })
    }

    /// The subtitle text the engine is drawing right now, if any.
    ///
    /// Needs [`Capability::RenderedTextObservation`]. `None` means nothing is
    /// on screen at this moment — a gap between cues is not an error.
    ///
    /// The answer is what the engine **actually drew**, not what it was asked
    /// to draw. That distinction is the whole point: it is what lets a caller
    /// check a rendered cue against the document it came from (ADR-0013).
    ///
    /// **Security:** the returned text is subtitle dialogue (K23 #4). It may be
    /// displayed and compared; it may never be logged.
    fn rendered_subtitle_text(&self) -> Result<Option<String>, PlaybackError> {
        Err(PlaybackError::Unsupported {
            operation: Operation::RenderedText,
            capability: Capability::RenderedTextObservation,
        })
    }

    /// Keeps the subtitle out of the bottom `fraction` of the surface
    /// (ADR-0037 Karar 5).
    ///
    /// **Base, not a capability**, and it has no default: an engine that
    /// silently ignored this would draw the subtitle underneath whatever the
    /// shell put on top of it, which is exactly the defect NEN-066 measured —
    /// libmpv composited the line correctly and the transport bar covered it.
    /// A renderer that cannot be told where not to draw cannot be paired with
    /// a shell that draws anything of its own.
    ///
    /// `fraction` is a share of the **surface height**, never points or
    /// pixels, and is always within `0.0..=`[`renderer::MAX_BOTTOM_INSET`](crate::renderer::MAX_BOTTOM_INSET):
    /// the renderer port validates before delegating here (ADR-0037 Karar 2).
    fn set_subtitle_bottom_inset(&mut self, fraction: f32) -> Result<(), PlaybackError>;

    /// Hands the engine a subtitle document to render.
    ///
    /// Needs [`Capability::ExternalSubtitleInjection`]. The rendering path
    /// itself is NEN-027's.
    fn inject_subtitle(&mut self, document: &SubtitleDocument) -> Result<(), PlaybackError> {
        let _ = document;
        Err(PlaybackError::Unsupported {
            operation: Operation::InjectSubtitle,
            capability: Capability::ExternalSubtitleInjection,
        })
    }
}
