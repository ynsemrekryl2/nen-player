//! A deterministic in-memory `PlaybackEngine`, and the reference for what the
//! contract requires.
//!
//! Two jobs, and it is worth being explicit about both:
//!
//! 1. It is the **reference implementation** — when the contract kit and an
//!    adapter disagree about what a rule means, this is what the rule means.
//! 2. It lets every crate above the port (`nen-app`, and M3's shell work) be
//!    tested without a media engine, a file or a clock.
//!
//! It is **not** product behaviour: no decoding, no I/O, no threads, no time.
//! Playback position moves only when something moves it, which is what makes
//! the kit's expectations exact rather than timing-dependent.

use super::capability::{Capabilities, Capability};
use super::contract::ContractInputs;
use super::engine::PlaybackEngine;
use super::error::{Operation, PlaybackError};
use super::event::{guard_reentrancy, EventQueue, PlaybackEvent, PlaybackState};
use super::media::MediaSource;
use super::track::{TrackDescriptor, TrackId, TrackKind};
use nen_domain::source::LanguageTag;
use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use std::time::Duration;

/// The medium the fake engine pretends to play.
pub const FAKE_DURATION_MS: u64 = 120_000;

/// The rate range the fake accepts when it declares
/// [`Capability::PlaybackRate`].
pub const FAKE_RATE_RANGE: (f32, f32) = (0.5, 2.0);

/// How many audio tracks the fake medium has.
pub const FAKE_AUDIO_TRACKS: usize = 1;

/// How many subtitle tracks the fake medium has.
pub const FAKE_SUBTITLE_TRACKS: usize = 2;

/// An audio track the fake medium has.
pub const FAKE_AUDIO_TRACK: TrackId = TrackId(2);

/// A subtitle track the fake medium has.
pub const FAKE_SUBTITLE_TRACK: TrackId = TrackId(1);

/// An id no fake track of any kind has.
pub const FAKE_UNKNOWN_TRACK: TrackId = TrackId(9_999);

/// A fake engine whose capability set is chosen by the caller.
#[derive(Debug)]
pub struct FakeEngine {
    capabilities: Capabilities,
    state: PlaybackState,
    position: Duration,
    rate: f32,
    volume: f32,
    selected_audio: Option<TrackId>,
    selected_subtitle: Option<TrackId>,
    injected_cues: Option<usize>,
    shut_down: bool,
    events: EventQueue,
}

impl FakeEngine {
    /// An engine declaring every optional capability.
    pub fn full() -> Self {
        Self::new(Capabilities::ALL)
    }

    /// An engine providing only the mandatory base.
    ///
    /// This is the one that proves Karar 3's other half: every gated operation
    /// must come back as a typed refusal rather than a panic or a no-op.
    pub fn minimal() -> Self {
        Self::new(Capabilities::NONE)
    }

    pub fn new(capabilities: Capabilities) -> Self {
        Self {
            capabilities,
            state: PlaybackState::Idle,
            position: Duration::ZERO,
            rate: 1.0,
            volume: 1.0,
            selected_audio: None,
            selected_subtitle: None,
            injected_cues: None,
            shut_down: false,
            events: EventQueue::default(),
        }
    }

    /// The current rate. Test-facing; not part of the port.
    pub fn rate(&self) -> f32 {
        self.rate
    }

    /// The current volume. Test-facing; not part of the port.
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// How many cues the last injected document carried, if any.
    /// Test-facing; not part of the port.
    pub fn injected_cues(&self) -> Option<usize> {
        self.injected_cues
    }

    fn descriptors(&self, kind: TrackKind) -> Vec<TrackDescriptor> {
        match kind {
            // Ids are deliberately disjoint across kinds so that the contract
            // can prove an audio id is not silently accepted as a subtitle one.
            TrackKind::Subtitle => vec![
                TrackDescriptor::new(TrackId(0), TrackKind::Subtitle, "subrip")
                    .with_language(LanguageTag::parse("en").ok())
                    .with_title(Some("English".into()))
                    .with_default(true),
                TrackDescriptor::new(TrackId(1), TrackKind::Subtitle, "hdmv_pgs_subtitle")
                    .with_language(LanguageTag::parse("tr").ok())
                    .with_text(false),
            ],
            TrackKind::Audio => vec![TrackDescriptor::new(TrackId(2), TrackKind::Audio, "aac")
                .with_language(LanguageTag::parse("en").ok())
                .with_default(true)],
        }
    }

    fn has_track(&self, kind: TrackKind, track: TrackId) -> bool {
        self.descriptors(kind)
            .iter()
            .any(|descriptor| descriptor.id() == track)
    }

    /// The order every operation checks in: contract violation, then lifecycle,
    /// then capability, then media.
    fn ensure_live(&self, operation: Operation) -> Result<(), PlaybackError> {
        guard_reentrancy(operation)?;
        if self.shut_down {
            return Err(PlaybackError::ShutDown { operation });
        }
        Ok(())
    }

    fn ensure_loaded(&self, operation: Operation) -> Result<(), PlaybackError> {
        self.ensure_live(operation)?;
        if !self.state.has_media() {
            return Err(PlaybackError::NotLoaded { operation });
        }
        Ok(())
    }

    fn require(&self, capability: Capability, operation: Operation) -> Result<(), PlaybackError> {
        if self.capabilities.contains(capability) {
            Ok(())
        } else {
            Err(PlaybackError::Unsupported {
                operation,
                capability,
            })
        }
    }

    fn set_state(&mut self, state: PlaybackState) {
        self.state = state;
        self.events.push(PlaybackEvent::StateChanged { state });
    }

    fn duration_value(&self) -> Duration {
        Duration::from_millis(FAKE_DURATION_MS)
    }
}

impl Default for FakeEngine {
    fn default() -> Self {
        Self::full()
    }
}

impl PlaybackEngine for FakeEngine {
    fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    fn load(&mut self, source: &MediaSource) -> Result<(), PlaybackError> {
        self.ensure_live(Operation::Load)?;
        if source.is_empty() {
            return Err(PlaybackError::LoadFailed {
                reason: super::error::LoadFailure::NotFound,
            });
        }
        self.position = Duration::ZERO;
        self.selected_audio = None;
        self.selected_subtitle = None;
        self.set_state(PlaybackState::Buffering);
        self.set_state(PlaybackState::Ready);
        self.events.push(PlaybackEvent::TracksChanged);
        Ok(())
    }

    fn play(&mut self) -> Result<(), PlaybackError> {
        self.ensure_loaded(Operation::Play)?;
        self.set_state(PlaybackState::Playing);
        Ok(())
    }

    fn pause(&mut self) -> Result<(), PlaybackError> {
        self.ensure_loaded(Operation::Pause)?;
        self.set_state(PlaybackState::Paused);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), PlaybackError> {
        self.ensure_loaded(Operation::Stop)?;
        self.position = Duration::ZERO;
        self.selected_audio = None;
        self.selected_subtitle = None;
        self.set_state(PlaybackState::Idle);
        Ok(())
    }

    fn seek(&mut self, to: Duration) -> Result<(), PlaybackError> {
        self.ensure_loaded(Operation::Seek)?;
        let duration = self.duration_value();
        // Seeking past the end is not an error — it ends playback.
        let ended = to >= duration;
        self.position = to.min(duration);
        self.events.push(PlaybackEvent::SeekCompleted {
            position: self.position,
        });
        self.events.push(PlaybackEvent::PositionChanged {
            position: self.position,
        });
        if ended {
            self.set_state(PlaybackState::Ended);
            self.events.push(PlaybackEvent::EndReached);
        }
        Ok(())
    }

    fn position(&self) -> Result<Duration, PlaybackError> {
        self.ensure_loaded(Operation::Position)?;
        Ok(self.position)
    }

    fn duration(&self) -> Result<Option<Duration>, PlaybackError> {
        self.ensure_loaded(Operation::Duration)?;
        Ok(Some(self.duration_value()))
    }

    fn state(&self) -> PlaybackState {
        self.state
    }

    fn tracks(&self, kind: TrackKind) -> Result<Vec<TrackDescriptor>, PlaybackError> {
        self.ensure_loaded(Operation::Tracks)?;
        Ok(self.descriptors(kind))
    }

    fn select_track(
        &mut self,
        kind: TrackKind,
        track: Option<TrackId>,
    ) -> Result<(), PlaybackError> {
        self.ensure_loaded(Operation::SelectTrack)?;
        if let Some(id) = track {
            if !self.has_track(kind, id) {
                return Err(PlaybackError::UnknownTrack { kind });
            }
        }
        match kind {
            TrackKind::Audio => self.selected_audio = track,
            TrackKind::Subtitle => self.selected_subtitle = track,
        }
        Ok(())
    }

    fn selected_track(&self, kind: TrackKind) -> Result<Option<TrackId>, PlaybackError> {
        self.ensure_loaded(Operation::SelectTrack)?;
        Ok(match kind {
            TrackKind::Audio => self.selected_audio,
            TrackKind::Subtitle => self.selected_subtitle,
        })
    }

    fn events(&mut self) -> &mut EventQueue {
        &mut self.events
    }

    fn shutdown(&mut self) -> Result<(), PlaybackError> {
        guard_reentrancy(Operation::Shutdown)?;
        // Idempotent by contract: a shell cannot always know whether its own
        // teardown already ran.
        self.shut_down = true;
        self.state = PlaybackState::Idle;
        Ok(())
    }

    fn set_rate(&mut self, rate: f32) -> Result<(), PlaybackError> {
        self.ensure_live(Operation::SetRate)?;
        self.require(Capability::PlaybackRate, Operation::SetRate)?;
        let (min, max) = FAKE_RATE_RANGE;
        if !(min..=max).contains(&rate) {
            return Err(PlaybackError::RateOutOfRange {
                requested: rate,
                min,
                max,
            });
        }
        self.rate = rate;
        Ok(())
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), PlaybackError> {
        self.ensure_live(Operation::SetVolume)?;
        self.require(Capability::Volume, Operation::SetVolume)?;
        self.volume = volume.clamp(0.0, 1.0);
        Ok(())
    }

    fn extract_text(&mut self, track: TrackId) -> Result<String, PlaybackError> {
        self.ensure_live(Operation::ExtractText)?;
        self.require(Capability::EmbeddedTextExtraction, Operation::ExtractText)?;
        if !self.has_track(TrackKind::Subtitle, track) {
            return Err(PlaybackError::UnknownTrack {
                kind: TrackKind::Subtitle,
            });
        }
        Ok("1\n00:00:01,000 --> 00:00:02,000\nfake cue\n".to_string())
    }

    fn inject_subtitle(&mut self, document: &SubtitleDocument) -> Result<(), PlaybackError> {
        self.ensure_live(Operation::InjectSubtitle)?;
        self.require(
            Capability::ExternalSubtitleInjection,
            Operation::InjectSubtitle,
        )?;
        self.injected_cues = Some(document.len());
        Ok(())
    }
}

/// The inputs the contract kit needs to drive a [`FakeEngine`].
pub fn fake_inputs() -> ContractInputs {
    let span = TimeSpan::new(0, 1_000).expect("a valid span");
    let document = SubtitleDocument::new(vec![Cue::new(
        CueId::new(1),
        span,
        vec!["contract".to_string()],
    )]);
    ContractInputs::new(MediaSource::new("fake://medium"), document)
        .with_duration_ms(Some(FAKE_DURATION_MS))
        .with_track_counts(FAKE_AUDIO_TRACKS, FAKE_SUBTITLE_TRACKS)
        .with_tracks(FAKE_AUDIO_TRACK, FAKE_SUBTITLE_TRACK, FAKE_UNKNOWN_TRACK)
    // No `with_seek_tolerance_ms`: the fake is exact, and leaving the default
    // at zero is what keeps the loosened kit strict here. A real adapter
    // declares the tolerance it measured; the fake is not allowed one.
}
