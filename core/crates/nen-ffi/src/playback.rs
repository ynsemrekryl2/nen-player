//! The playback port, as it appears to a platform shell.
//!
//! ADR-0026 keeps the session in the core; ADR-0012 Karar 2 puts the macOS
//! engine in Swift. So the engine is a **foreign object the core calls**, and
//! this module is where that shape is declared — `nen-ffi` being the only
//! crate allowed an FFI surface (ADR-0006 kural 2).
//!
//! Nothing here decides anything. The logic is [`nen_app::playback`]: this file
//! is translation, and every type below exists because a Rust type could not
//! cross as it stands.
//!
//! # Security (K23)
//!
//! NEN-010 measured the rule these types obey: **an error crossing the boundary
//! is re-printed by the host language**, which never sees a Rust `Debug` impl.
//! A value that is safe only because of how Rust prints it is not safe. So
//! every field that travels **outbound** is a bounded enum or a number — never
//! a path, a URL, a filename or a title.
//!
//! Two strings do travel: the locator, which becomes a
//! [`MediaSource`](nen_ports::playback::MediaSource), and a track title, which
//! becomes a [`TrackDescriptor`] (NEN-023). The safety argument is that the
//! value already exists on the platform side, so crossing adds no exposure, and
//! once across it is behind a guarded `Debug`.
//!
//! The locator only ever travels **inbound**; nothing sends it back. A track
//! title does return, in the descriptors [`session`](crate::session) hands the
//! shell — but unchanged, and to the side that produced it. That is a
//! round-trip of the platform's own value, not the core disclosing something:
//! §8's menu needs the title the container declared, and the alternative would
//! be making the shell re-correlate its own list by id. What stays forbidden is
//! **Rust printing it**, which is a different rule and the one
//! `tests/guard_ffi_track_debug.rs` holds.
//!
//! `tests/guard_ffi_playback_debug.rs` holds that line with a deliberately
//! leaky twin.

use nen_app::playback::{run_contract, ContractFixture, ShellEngine, ShellEngineFactory};
use nen_app::ports::playback::{
    Capability, LoadFailure, Operation, PlaybackError, PlaybackEvent, PlaybackState,
    TrackDescriptor, TrackId, TrackKind,
};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

/// An optional engine capability (ADR-0011 Karar 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiCapability {
    EmbeddedTextExtraction,
    ExternalSubtitleInjection,
    PlaybackRate,
    Volume,
}

impl From<FfiCapability> for Capability {
    fn from(value: FfiCapability) -> Self {
        match value {
            FfiCapability::EmbeddedTextExtraction => Self::EmbeddedTextExtraction,
            FfiCapability::ExternalSubtitleInjection => Self::ExternalSubtitleInjection,
            FfiCapability::PlaybackRate => Self::PlaybackRate,
            FfiCapability::Volume => Self::Volume,
        }
    }
}

impl From<Capability> for FfiCapability {
    fn from(value: Capability) -> Self {
        match value {
            Capability::EmbeddedTextExtraction => Self::EmbeddedTextExtraction,
            Capability::ExternalSubtitleInjection => Self::ExternalSubtitleInjection,
            Capability::PlaybackRate => Self::PlaybackRate,
            Capability::Volume => Self::Volume,
        }
    }
}

/// Whether a track carries audio or subtitles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiTrackKind {
    Audio,
    Subtitle,
}

impl From<FfiTrackKind> for TrackKind {
    fn from(value: FfiTrackKind) -> Self {
        match value {
            FfiTrackKind::Audio => Self::Audio,
            FfiTrackKind::Subtitle => Self::Subtitle,
        }
    }
}

impl From<TrackKind> for FfiTrackKind {
    fn from(value: TrackKind) -> Self {
        match value {
            TrackKind::Audio => Self::Audio,
            TrackKind::Subtitle => Self::Subtitle,
        }
    }
}

/// Where playback is, as a whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiPlaybackState {
    Idle,
    Buffering,
    Ready,
    Playing,
    Paused,
    Ended,
    Failed,
}

impl From<FfiPlaybackState> for PlaybackState {
    fn from(value: FfiPlaybackState) -> Self {
        match value {
            FfiPlaybackState::Idle => Self::Idle,
            FfiPlaybackState::Buffering => Self::Buffering,
            FfiPlaybackState::Ready => Self::Ready,
            FfiPlaybackState::Playing => Self::Playing,
            FfiPlaybackState::Paused => Self::Paused,
            FfiPlaybackState::Ended => Self::Ended,
            FfiPlaybackState::Failed => Self::Failed,
        }
    }
}

impl From<PlaybackState> for FfiPlaybackState {
    fn from(value: PlaybackState) -> Self {
        match value {
            PlaybackState::Idle => Self::Idle,
            PlaybackState::Buffering => Self::Buffering,
            PlaybackState::Ready => Self::Ready,
            PlaybackState::Playing => Self::Playing,
            PlaybackState::Paused => Self::Paused,
            PlaybackState::Ended => Self::Ended,
            PlaybackState::Failed => Self::Failed,
        }
    }
}

/// Why loading failed. Says nothing about *which* medium (K23 #1, #3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiLoadFailure {
    NotFound,
    Unreadable,
    UnsupportedFormat,
    NetworkUnavailable,
}

impl From<FfiLoadFailure> for LoadFailure {
    fn from(value: FfiLoadFailure) -> Self {
        match value {
            FfiLoadFailure::NotFound => Self::NotFound,
            FfiLoadFailure::Unreadable => Self::Unreadable,
            FfiLoadFailure::UnsupportedFormat => Self::UnsupportedFormat,
            FfiLoadFailure::NetworkUnavailable => Self::NetworkUnavailable,
        }
    }
}

impl From<LoadFailure> for FfiLoadFailure {
    fn from(value: LoadFailure) -> Self {
        match value {
            LoadFailure::NotFound => Self::NotFound,
            LoadFailure::Unreadable => Self::Unreadable,
            LoadFailure::UnsupportedFormat => Self::UnsupportedFormat,
            LoadFailure::NetworkUnavailable => Self::NetworkUnavailable,
        }
    }
}

/// A refused or failed playback operation, as the shell reports it.
///
/// **No `operation` field.** The port's error names the operation it refused,
/// but an adapter never has to say which method it is inside — the core called
/// it and already knows. Leaving it out removes a whole class of adapter bug
/// (reporting the wrong operation) and keeps the boundary honest about who
/// owns which fact.
#[derive(Debug, Clone, Copy, PartialEq, uniffi::Error)]
pub enum FfiPlaybackError {
    /// The engine does not declare the capability this operation needs.
    Unsupported { capability: FfiCapability },
    /// Called synchronously from inside an event callback (ADR-0011 Karar 2).
    ReentrantCall,
    /// The operation needs loaded media and nothing is loaded.
    NotLoaded,
    /// The engine has been shut down.
    ShutDown,
    /// No track with the given id, or the id belongs to the other kind.
    UnknownTrack { kind: FfiTrackKind },
    /// The rate is outside what the engine accepts.
    RateOutOfRange { requested: f32, min: f32, max: f32 },
    /// Loading failed.
    LoadFailed { reason: FfiLoadFailure },
    /// The engine failed for a reason of its own. The code is opaque and must
    /// never be shown or branched on (product-spec §4).
    EngineFailure { code: i32 },
}

impl std::fmt::Display for FfiPlaybackError {
    /// Names the variant and nothing else.
    ///
    /// `uniffi::Error` requires `Display`, and whatever it prints is what the
    /// host language shows. So it prints a constant of this crate — never a
    /// payload, not even the opaque engine code.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Unsupported { .. } => "unsupported",
            Self::ReentrantCall => "reentrant_call",
            Self::NotLoaded => "not_loaded",
            Self::ShutDown => "shut_down",
            Self::UnknownTrack { .. } => "unknown_track",
            Self::RateOutOfRange { .. } => "rate_out_of_range",
            Self::LoadFailed { .. } => "load_failed",
            Self::EngineFailure { .. } => "engine_failure",
        })
    }
}

impl std::error::Error for FfiPlaybackError {}

impl From<PlaybackError> for FfiPlaybackError {
    /// Takes the operation off, for the same reason it was never on.
    ///
    /// The shell called the method it is holding the error from, so the
    /// operation would be a fact it already has — and one more field able to
    /// disagree with reality.
    fn from(value: PlaybackError) -> Self {
        match value {
            PlaybackError::Unsupported { capability, .. } => Self::Unsupported {
                capability: capability.into(),
            },
            PlaybackError::ReentrantCall { .. } => Self::ReentrantCall,
            PlaybackError::NotLoaded { .. } => Self::NotLoaded,
            PlaybackError::ShutDown { .. } => Self::ShutDown,
            PlaybackError::UnknownTrack { kind } => Self::UnknownTrack { kind: kind.into() },
            PlaybackError::RateOutOfRange {
                requested,
                min,
                max,
            } => Self::RateOutOfRange {
                requested,
                min,
                max,
            },
            PlaybackError::LoadFailed { reason } => Self::LoadFailed {
                reason: reason.into(),
            },
            PlaybackError::EngineFailure { code } => Self::EngineFailure { code },
        }
    }
}

impl FfiPlaybackError {
    /// Puts the operation back on, from the call site that knows it.
    fn into_port(self, operation: Operation) -> PlaybackError {
        match self {
            Self::Unsupported { capability } => PlaybackError::Unsupported {
                operation,
                capability: capability.into(),
            },
            Self::ReentrantCall => PlaybackError::ReentrantCall { operation },
            Self::NotLoaded => PlaybackError::NotLoaded { operation },
            Self::ShutDown => PlaybackError::ShutDown { operation },
            Self::UnknownTrack { kind } => PlaybackError::UnknownTrack { kind: kind.into() },
            Self::RateOutOfRange {
                requested,
                min,
                max,
            } => PlaybackError::RateOutOfRange {
                requested,
                min,
                max,
            },
            Self::LoadFailed { reason } => PlaybackError::LoadFailed {
                reason: reason.into(),
            },
            Self::EngineFailure { code } => PlaybackError::EngineFailure { code },
        }
    }
}

/// One embedded track, as metadata.
///
/// `is_text` is deliberately **absent**. Whether a codec carries text is one
/// question with one answer, and
/// [`subtitle_carries_text`](nen_ports::playback::subtitle_carries_text)
/// answers it on this side for every platform. An adapter that classified its
/// own tracks would put the same list in Swift and again in Kotlin, and the
/// first one to miss a format would promise text it cannot produce.
#[derive(Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiTrackDescriptor {
    pub id: u32,
    pub kind: FfiTrackKind,
    /// BCP-47 tag as the container declares it; `None` when it declares none.
    pub language: Option<String>,
    pub codec: String,
    pub is_default: bool,
    /// What the container calls this track — the menu's label (§8).
    ///
    /// **Never logged.** A track title is regularly a release name or a private
    /// filename (K23 #8), which is why this type prints by hand and why
    /// `tests/guard_ffi_track_debug.rs` proves a derived twin leaks. The
    /// platform side owns the same discipline: it already has the value, so
    /// crossing adds no exposure — in either direction, this being the one
    /// field that also travels back out — but printing it does.
    pub title: Option<String>,
}

impl From<FfiTrackDescriptor> for TrackDescriptor {
    fn from(value: FfiTrackDescriptor) -> Self {
        TrackDescriptor::new(TrackId(value.id), value.kind.into(), value.codec)
            .with_language(
                value
                    .language
                    .and_then(|tag| nen_app::domain::source::LanguageTag::parse(&tag).ok()),
            )
            .with_title(value.title)
            .with_default(value.is_default)
    }
}

impl From<TrackDescriptor> for FfiTrackDescriptor {
    /// The core's descriptor, on its way back to the shell that supplied it.
    ///
    /// `language` is not the string that came in: the core canonicalises what
    /// the container declared (ADR-0032 turns `eng` into `en`), and the shell
    /// must see the same tag the menu groups by, not the raw one.
    fn from(value: TrackDescriptor) -> Self {
        Self {
            id: value.id().index(),
            kind: value.kind().into(),
            language: value.language().map(|tag| tag.as_str().to_owned()),
            codec: value.codec().to_owned(),
            is_default: value.is_default(),
            title: value.title().map(ToOwned::to_owned),
        }
    }
}

impl fmt::Debug for FfiTrackDescriptor {
    /// Prints everything except the title — the same line
    /// [`TrackDescriptor`] holds, for the same reason (K23 #8).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FfiTrackDescriptor")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("language", &self.language)
            .field("codec", &self.codec)
            .field("is_default", &self.is_default)
            .field("has_title", &self.title.is_some())
            .finish()
    }
}

/// Something the engine reports.
#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum FfiPlaybackEvent {
    PositionChanged { position_ms: u64 },
    StateChanged { state: FfiPlaybackState },
    SeekCompleted { position_ms: u64 },
    TracksChanged,
    EndReached,
    Failed { error: FfiPlaybackError },
}

impl From<FfiPlaybackEvent> for PlaybackEvent {
    fn from(value: FfiPlaybackEvent) -> Self {
        match value {
            FfiPlaybackEvent::PositionChanged { position_ms } => Self::PositionChanged {
                position: Duration::from_millis(position_ms),
            },
            FfiPlaybackEvent::StateChanged { state } => Self::StateChanged {
                state: state.into(),
            },
            FfiPlaybackEvent::SeekCompleted { position_ms } => Self::SeekCompleted {
                position: Duration::from_millis(position_ms),
            },
            FfiPlaybackEvent::TracksChanged => Self::TracksChanged,
            FfiPlaybackEvent::EndReached => Self::EndReached,
            FfiPlaybackEvent::Failed { error } => Self::Failed {
                // A `Failed` event is not the answer to any one call, so there
                // is no call site to take the operation from.
                error: error.into_port(Operation::State),
            },
        }
    }
}

/// `EventsLost` is **not** in [`FfiPlaybackEvent`] on purpose: it is the shared
/// queue's own verdict about overflow (ADR-0011 Karar 1), not something an
/// adapter may claim. An adapter that could synthesise it could hide a stream
/// it simply failed to deliver.
const _: () = ();

/// The engine, implemented by the platform shell.
///
/// Mirrors [`nen_app::playback::ShellEngine`]; the differences from the port
/// itself are documented there.
#[uniffi::export(with_foreign)]
pub trait ForeignPlaybackEngine: Send + Sync {
    fn capabilities(&self) -> Vec<FfiCapability>;

    fn load(&self, locator: String) -> Result<(), FfiPlaybackError>;
    fn play(&self) -> Result<(), FfiPlaybackError>;
    fn pause(&self) -> Result<(), FfiPlaybackError>;
    fn stop(&self) -> Result<(), FfiPlaybackError>;
    fn seek(&self, to_ms: u64) -> Result<(), FfiPlaybackError>;

    fn position_ms(&self) -> Result<u64, FfiPlaybackError>;
    fn duration_ms(&self) -> Result<Option<u64>, FfiPlaybackError>;
    fn state(&self) -> FfiPlaybackState;

    fn tracks(&self, kind: FfiTrackKind) -> Result<Vec<FfiTrackDescriptor>, FfiPlaybackError>;
    fn select_track(&self, kind: FfiTrackKind, track: Option<u32>) -> Result<(), FfiPlaybackError>;
    fn selected_track(&self, kind: FfiTrackKind) -> Result<Option<u32>, FfiPlaybackError>;

    /// Everything reported since the last call, in the order it happened.
    fn drain_events(&self) -> Vec<FfiPlaybackEvent>;

    fn shutdown(&self) -> Result<(), FfiPlaybackError>;

    fn set_rate(&self, rate: f32) -> Result<(), FfiPlaybackError>;
    fn set_volume(&self, volume: f32) -> Result<(), FfiPlaybackError>;
    /// Subtitle dialogue (K23 #4): displayable, never loggable.
    fn extract_text(&self, track: u32) -> Result<String, FfiPlaybackError>;
    fn inject_subtitle(&self, webvtt: String) -> Result<(), FfiPlaybackError>;
}

/// Builds a fresh engine. The kit rebuilds one per scenario.
#[uniffi::export(with_foreign)]
pub trait ForeignEngineFactory: Send + Sync {
    fn build(&self) -> Arc<dyn ForeignPlaybackEngine>;
}

/// A foreign engine, wearing the shape the application layer expects.
struct ForeignEngineAdapter {
    inner: Arc<dyn ForeignPlaybackEngine>,
}

/// Dresses a foreign engine for the application layer.
///
/// The kit builds its own through [`ForeignFactoryAdapter`]; a session is
/// handed one engine and keeps it, so it needs this directly.
pub(crate) fn shell_engine(engine: Arc<dyn ForeignPlaybackEngine>) -> Arc<dyn ShellEngine> {
    Arc::new(ForeignEngineAdapter { inner: engine })
}

impl ShellEngine for ForeignEngineAdapter {
    fn capabilities(&self) -> Vec<Capability> {
        self.inner
            .capabilities()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn load(&self, locator: String) -> Result<(), PlaybackError> {
        self.inner
            .load(locator)
            .map_err(|error| error.into_port(Operation::Load))
    }

    fn play(&self) -> Result<(), PlaybackError> {
        self.inner
            .play()
            .map_err(|error| error.into_port(Operation::Play))
    }

    fn pause(&self) -> Result<(), PlaybackError> {
        self.inner
            .pause()
            .map_err(|error| error.into_port(Operation::Pause))
    }

    fn stop(&self) -> Result<(), PlaybackError> {
        self.inner
            .stop()
            .map_err(|error| error.into_port(Operation::Stop))
    }

    fn seek(&self, to_ms: u64) -> Result<(), PlaybackError> {
        self.inner
            .seek(to_ms)
            .map_err(|error| error.into_port(Operation::Seek))
    }

    fn position_ms(&self) -> Result<u64, PlaybackError> {
        self.inner
            .position_ms()
            .map_err(|error| error.into_port(Operation::Position))
    }

    fn duration_ms(&self) -> Result<Option<u64>, PlaybackError> {
        self.inner
            .duration_ms()
            .map_err(|error| error.into_port(Operation::Duration))
    }

    fn state(&self) -> PlaybackState {
        self.inner.state().into()
    }

    fn tracks(&self, kind: TrackKind) -> Result<Vec<TrackDescriptor>, PlaybackError> {
        self.inner
            .tracks(kind.into())
            .map(|tracks| tracks.into_iter().map(Into::into).collect())
            .map_err(|error| error.into_port(Operation::Tracks))
    }

    fn select_track(&self, kind: TrackKind, track: Option<TrackId>) -> Result<(), PlaybackError> {
        self.inner
            .select_track(kind.into(), track.map(TrackId::index))
            .map_err(|error| error.into_port(Operation::SelectTrack))
    }

    fn selected_track(&self, kind: TrackKind) -> Result<Option<TrackId>, PlaybackError> {
        self.inner
            .selected_track(kind.into())
            .map(|selected| selected.map(TrackId))
            .map_err(|error| error.into_port(Operation::SelectTrack))
    }

    fn drain_events(&self) -> Vec<PlaybackEvent> {
        self.inner
            .drain_events()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn shutdown(&self) -> Result<(), PlaybackError> {
        self.inner
            .shutdown()
            .map_err(|error| error.into_port(Operation::Shutdown))
    }

    fn set_rate(&self, rate: f32) -> Result<(), PlaybackError> {
        self.inner
            .set_rate(rate)
            .map_err(|error| error.into_port(Operation::SetRate))
    }

    fn set_volume(&self, volume: f32) -> Result<(), PlaybackError> {
        self.inner
            .set_volume(volume)
            .map_err(|error| error.into_port(Operation::SetVolume))
    }

    fn extract_text(&self, track: TrackId) -> Result<String, PlaybackError> {
        self.inner
            .extract_text(track.index())
            .map_err(|error| error.into_port(Operation::ExtractText))
    }

    fn inject_subtitle(&self, webvtt: String) -> Result<(), PlaybackError> {
        self.inner
            .inject_subtitle(webvtt)
            .map_err(|error| error.into_port(Operation::InjectSubtitle))
    }
}

struct ForeignFactoryAdapter {
    inner: Arc<dyn ForeignEngineFactory>,
}

impl ShellEngineFactory for ForeignFactoryAdapter {
    fn build(&self) -> Arc<dyn ShellEngine> {
        Arc::new(ForeignEngineAdapter {
            inner: self.inner.build(),
        })
    }
}

/// What the adapter's own fixture is.
///
/// The scenario list names no durations, counts or ids (ADR-0011 Karar 4);
/// these travel with the adapter that supplies the medium.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiContractFixture {
    /// The medium to load. **Never log this** (K23 #1, #3).
    pub locator: String,
    pub duration_ms: Option<u64>,
    pub audio_track_count: u32,
    pub subtitle_track_count: u32,
    pub audio_track: u32,
    pub subtitle_track: u32,
    pub unknown_track: u32,
    /// How far a seek may land from where it was asked to.
    pub seek_tolerance_ms: u64,
    /// How long to wait for a state before calling it a failure.
    pub settle_timeout_ms: u64,
}

/// The result of one contract run.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiContractReport {
    /// One line per failing step, already rendered by the kit.
    pub failures: Vec<String>,
    /// How many scenarios ran. A caller must refuse a run of zero.
    pub applied: u32,
}

/// Runs the shared contract kit against a foreign engine.
///
/// This is the whole reason the boundary types above exist: `M3`'s exit
/// criterion is that the real adapter passes **the same** kit as the fake, and
/// ADR-0011 Karar 4 keeps the scenarios data so it can be *the same* list
/// rather than a second copy that drifts.
#[uniffi::export]
pub fn run_playback_contract(
    factory: Arc<dyn ForeignEngineFactory>,
    fixture: FfiContractFixture,
) -> FfiContractReport {
    let report = run_contract(
        Arc::new(ForeignFactoryAdapter { inner: factory }),
        ContractFixture {
            locator: fixture.locator,
            duration_ms: fixture.duration_ms,
            audio_track_count: fixture.audio_track_count,
            subtitle_track_count: fixture.subtitle_track_count,
            audio_track: fixture.audio_track,
            subtitle_track: fixture.subtitle_track,
            unknown_track: fixture.unknown_track,
            seek_tolerance_ms: fixture.seek_tolerance_ms,
            settle_timeout_ms: fixture.settle_timeout_ms,
        },
    );
    FfiContractReport {
        failures: report.failures,
        applied: report.applied,
    }
}

/// How many scenarios apply to an engine declaring these capabilities.
///
/// Lets the shell assert its run was not vacuous without hard-coding a number
/// that the kit is free to grow past.
#[uniffi::export]
pub fn applicable_scenario_count(capabilities: Vec<FfiCapability>) -> u32 {
    nen_app::playback::applicable_scenario_count(capabilities.into_iter().map(Into::into).collect())
}
