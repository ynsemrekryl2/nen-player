//! The session object the platform shell drives (ADR-0033 Karar 1).
//!
//! A gate and nothing else: every method here translates its arguments, calls
//! [`nen_app::session::PlaybackSession`] and translates the answer back. The
//! pump, the queue and the lifetime are that type's, so they are testable
//! without building a single binding.
//!
//! # Security (K23)
//!
//! The session holds no locator, no path and no title — it holds an engine and
//! a queue. No `Debug` is derived on it for the same reason the rest of this
//! crate hand-writes its own: a derived one would print whatever a field
//! happens to hold today, and the guard would have to be re-argued every time a
//! field is added.

use crate::playback::{
    shell_engine, FfiPlaybackError, FfiPlaybackState, FfiTrackDescriptor, FfiTrackKind,
    ForeignPlaybackEngine,
};
use crate::subtitles::FfiSubtitleLibrary;
use nen_app::ports::playback::{PlaybackEvent, TrackId};
use nen_app::session::{PlaybackSession, ShowOutcome};
use std::sync::Arc;
use std::time::Duration;

/// Something that happened, as the **core** reports it to the shell.
///
/// Deliberately not [`FfiPlaybackEvent`](crate::playback::FfiPlaybackEvent),
/// which is what an adapter reports *inward*. The two differ by exactly one
/// variant, and that variant is the point: `EventsLost` is the shared queue's
/// own verdict about overflow (ADR-0011 Karar 1). An adapter may not claim it —
/// one that could would be able to hide a stream it simply failed to deliver —
/// but the shell must be told, because being told is what makes the resync in
/// ADR-0031 Karar 3 possible.
#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum FfiSessionEvent {
    PositionChanged {
        position_ms: u64,
    },
    StateChanged {
        state: FfiPlaybackState,
    },
    SeekCompleted {
        position_ms: u64,
    },
    TracksChanged,
    EndReached,
    Failed {
        error: FfiPlaybackError,
    },
    /// The queue overflowed and dropped this many critical events. The shell
    /// re-reads state, position and tracks, and says nothing to the user.
    EventsLost {
        dropped: u32,
    },
}

impl From<PlaybackEvent> for FfiSessionEvent {
    fn from(value: PlaybackEvent) -> Self {
        match value {
            PlaybackEvent::PositionChanged { position } => Self::PositionChanged {
                position_ms: millis(position),
            },
            PlaybackEvent::StateChanged { state } => Self::StateChanged {
                state: state.into(),
            },
            PlaybackEvent::SeekCompleted { position } => Self::SeekCompleted {
                position_ms: millis(position),
            },
            PlaybackEvent::TracksChanged => Self::TracksChanged,
            PlaybackEvent::EndReached => Self::EndReached,
            PlaybackEvent::Failed { error } => Self::Failed {
                error: error.into(),
            },
            PlaybackEvent::EventsLost { dropped } => Self::EventsLost { dropped },
        }
    }
}

/// What showing a menu row did.
///
/// Not an error channel, on the same reasoning as
/// [`FfiSubtitleOutcome`](crate::subtitles::FfiSubtitleOutcome): a row that
/// cannot be shown is an outcome the product has a defined behaviour for —
/// nothing happens — while an engine refusing is a failure the user hears
/// about. The shell distinguishes them because it must show a message for one
/// and stay silent for the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiShowOutcome {
    Shown,
    Unusable,
}

impl From<ShowOutcome> for FfiShowOutcome {
    fn from(outcome: ShowOutcome) -> Self {
        match outcome {
            ShowOutcome::Shown => Self::Shown,
            ShowOutcome::Unusable => Self::Unusable,
        }
    }
}

fn millis(value: Duration) -> u64 {
    value.as_millis().min(u128::from(u64::MAX)) as u64
}

/// One playback session: the only thing the shell talks to.
///
/// The shell hands over the engine it built and then forgets it. Calling the
/// engine directly would put the session back in the shell and undo ADR-0026.
#[derive(uniffi::Object)]
pub struct FfiPlaybackSession {
    inner: PlaybackSession,
}

#[uniffi::export]
impl FfiPlaybackSession {
    /// Takes ownership of an engine and starts pulling from it.
    #[uniffi::constructor]
    pub fn new(engine: Arc<dyn ForeignPlaybackEngine>) -> Self {
        Self {
            inner: PlaybackSession::new(shell_engine(engine)),
        }
    }

    /// **Never log the locator** (K23 #1, #3). It goes straight into a
    /// `MediaSource` and is not kept here.
    pub fn load(&self, locator: String) -> Result<(), FfiPlaybackError> {
        self.inner.load(locator).map_err(Into::into)
    }

    pub fn play(&self) -> Result<(), FfiPlaybackError> {
        self.inner.play().map_err(Into::into)
    }

    pub fn pause(&self) -> Result<(), FfiPlaybackError> {
        self.inner.pause().map_err(Into::into)
    }

    pub fn stop(&self) -> Result<(), FfiPlaybackError> {
        self.inner.stop().map_err(Into::into)
    }

    pub fn seek(&self, to_ms: u64) -> Result<(), FfiPlaybackError> {
        self.inner
            .seek(Duration::from_millis(to_ms))
            .map_err(Into::into)
    }

    pub fn position_ms(&self) -> Result<u64, FfiPlaybackError> {
        self.inner.position().map(millis).map_err(Into::into)
    }

    /// `None` means the medium reports no duration — a live stream.
    pub fn duration_ms(&self) -> Result<Option<u64>, FfiPlaybackError> {
        self.inner
            .duration()
            .map(|total| total.map(millis))
            .map_err(Into::into)
    }

    pub fn state(&self) -> Result<FfiPlaybackState, FfiPlaybackError> {
        self.inner.state().map(Into::into).map_err(Into::into)
    }

    pub fn tracks(&self, kind: FfiTrackKind) -> Result<Vec<FfiTrackDescriptor>, FfiPlaybackError> {
        self.inner
            .tracks(kind.into())
            .map(|tracks| tracks.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    /// `None` deselects — §8's `Kapalı`.
    pub fn select_track(
        &self,
        kind: FfiTrackKind,
        track: Option<u32>,
    ) -> Result<(), FfiPlaybackError> {
        self.inner
            .select_track(kind.into(), track.map(TrackId))
            .map_err(Into::into)
    }

    pub fn selected_track(&self, kind: FfiTrackKind) -> Result<Option<u32>, FfiPlaybackError> {
        self.inner
            .selected_track(kind.into())
            .map(|track| track.map(|id| id.index()))
            .map_err(Into::into)
    }

    /// Puts the subtitle a menu row names on screen (ADR-0013 Karar 3).
    ///
    /// The shell hands over its library and a token and is told what happened.
    /// Which kind of row it was — an embedded track or a file the user loaded
    /// — is decided here and nowhere else, so a second platform cannot decide
    /// it differently. **No dialogue crosses this call in either direction.**
    pub fn show_subtitle(
        &self,
        library: Arc<FfiSubtitleLibrary>,
        token: u32,
    ) -> Result<FfiShowOutcome, FfiPlaybackError> {
        library
            .with(|library| self.inner.show_source(library, token))
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Declares the share of the surface the shell's own chrome covers, so
    /// the subtitle stays out from under it (ADR-0037).
    ///
    /// A fraction of the surface **height**, `0.0..=0.5`. The shell is the
    /// only side that knows how tall its chrome is and how tall the surface
    /// is; sending points would make this side ask for a scale factor it has
    /// no use for. `0.0` when the chrome is hidden.
    pub fn set_subtitle_bottom_inset(&self, fraction: f32) -> Result<(), FfiPlaybackError> {
        self.inner
            .set_subtitle_bottom_inset(fraction)
            .map_err(Into::into)
    }

    /// §8's `Kapalı`: nothing on screen, whatever was drawing it.
    pub fn hide_subtitle(&self) -> Result<(), FfiPlaybackError> {
        self.inner.hide_subtitle().map_err(Into::into)
    }

    /// The text that **should** be on screen at a moment.
    ///
    /// `nen_subtitle::CueIndex`'s answer over the document being shown
    /// (NEN-017), or `None` when nothing is shown and in a gap between cues.
    ///
    /// **Security:** subtitle dialogue (K23 #4). Displayable and comparable,
    /// never loggable.
    pub fn expected_subtitle_text(&self, at_ms: u64) -> Option<String> {
        self.inner.expected_subtitle_text(at_ms)
    }

    /// The text that **is** on screen, as the engine reports it.
    ///
    /// The pair with [`Self::expected_subtitle_text`]: comparing the two is
    /// how "the right cue is displayed after a seek" becomes something a test
    /// can measure rather than assert. Needs the engine to declare
    /// `RenderedTextObservation`; without it this is a typed refusal.
    pub fn rendered_subtitle_text(&self) -> Result<Option<String>, FfiPlaybackError> {
        self.inner.rendered_subtitle_text().map_err(Into::into)
    }

    pub fn set_rate(&self, rate: f32) -> Result<(), FfiPlaybackError> {
        self.inner.set_rate(rate).map_err(Into::into)
    }

    pub fn set_volume(&self, volume: f32) -> Result<(), FfiPlaybackError> {
        self.inner.set_volume(volume).map_err(Into::into)
    }

    /// Everything the queue holds, in order, emptying it.
    ///
    /// Called when the shell is ready to draw, not when the engine feels like
    /// talking (ADR-0033 Karar 3). A shell that stops calling this accumulates
    /// nothing: the pump keeps pulling, the queue keeps its limit, and the next
    /// call says how much was lost.
    pub fn drain_events(&self) -> Vec<FfiSessionEvent> {
        self.inner
            .drain_events()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// Stops the pump, then the engine. Calling it twice is not an error.
    pub fn shutdown(&self) -> Result<(), FfiPlaybackError> {
        self.inner.shutdown().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::playback::FfiCapability;
    use nen_app::ports::playback::{Capability, LoadFailure, Operation, PlaybackError, TrackKind};
    use nen_app::ports::playback::{PlaybackState, TrackDescriptor};

    /// `EventsLost` only exists in this direction.
    ///
    /// Nothing on the Swift side can produce an overflow to notice a mistake
    /// here — a real engine would have to fall 64 critical events behind — so
    /// the mapping is checked where it is written.
    #[test]
    fn the_loss_marker_crosses_with_its_count() {
        assert_eq!(
            FfiSessionEvent::from(PlaybackEvent::EventsLost { dropped: 7 }),
            FfiSessionEvent::EventsLost { dropped: 7 }
        );
    }

    #[test]
    fn every_event_keeps_what_it_carried() {
        let crossed: Vec<FfiSessionEvent> = vec![
            PlaybackEvent::PositionChanged {
                position: Duration::from_millis(1_500),
            },
            PlaybackEvent::StateChanged {
                state: PlaybackState::Playing,
            },
            PlaybackEvent::SeekCompleted {
                position: Duration::from_millis(12_000),
            },
            PlaybackEvent::TracksChanged,
            PlaybackEvent::EndReached,
            PlaybackEvent::Failed {
                error: PlaybackError::LoadFailed {
                    reason: LoadFailure::NotFound,
                },
            },
        ]
        .into_iter()
        .map(Into::into)
        .collect();

        assert_eq!(
            crossed,
            vec![
                FfiSessionEvent::PositionChanged { position_ms: 1_500 },
                FfiSessionEvent::StateChanged {
                    state: FfiPlaybackState::Playing
                },
                FfiSessionEvent::SeekCompleted {
                    position_ms: 12_000
                },
                FfiSessionEvent::TracksChanged,
                FfiSessionEvent::EndReached,
                FfiSessionEvent::Failed {
                    error: FfiPlaybackError::LoadFailed {
                        reason: crate::playback::FfiLoadFailure::NotFound
                    }
                },
            ]
        );
    }

    #[test]
    fn an_error_loses_its_operation_and_nothing_else() {
        assert_eq!(
            FfiPlaybackError::from(PlaybackError::ShutDown {
                operation: Operation::Play
            }),
            FfiPlaybackError::ShutDown
        );
        assert_eq!(
            FfiPlaybackError::from(PlaybackError::Unsupported {
                operation: Operation::SetRate,
                capability: Capability::PlaybackRate,
            }),
            FfiPlaybackError::Unsupported {
                capability: FfiCapability::PlaybackRate
            }
        );
        assert_eq!(
            FfiPlaybackError::from(PlaybackError::RateOutOfRange {
                requested: 4.0,
                min: 0.5,
                max: 2.0,
            }),
            FfiPlaybackError::RateOutOfRange {
                requested: 4.0,
                min: 0.5,
                max: 2.0
            }
        );
    }

    /// The descriptor comes back with the tag the core canonicalised, not the
    /// one the container wrote (ADR-0032).
    #[test]
    fn a_descriptor_returns_carrying_the_canonical_language() {
        let track = TrackDescriptor::new(TrackId(3), TrackKind::Subtitle, "subrip")
            .with_language(nen_app::domain::source::LanguageTag::parse("eng").ok())
            .with_title(Some("Forced".to_owned()))
            .with_default(true);

        let crossed = FfiTrackDescriptor::from(track);

        assert_eq!(crossed.id, 3);
        assert_eq!(crossed.language.as_deref(), Some("en"));
        assert_eq!(crossed.codec, "subrip");
        assert_eq!(crossed.title.as_deref(), Some("Forced"));
        assert!(crossed.is_default);
    }
}
