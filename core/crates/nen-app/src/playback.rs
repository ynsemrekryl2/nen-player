//! Driving the shared contract kit against an engine that lives outside Rust.
//!
//! ADR-0026 put the session in the core and ADR-0012 Karar 2 put the macOS
//! engine in Swift, so the adapter the kit has to judge is on the other side of
//! the FFI boundary. ADR-0011 Karar 4 then forbids the obvious shortcut:
//! the scenarios are **data** precisely so NEN-022 drives *the same list*
//! rather than writing a second kit that drifts from the first.
//!
//! This module is the joint. [`ShellEngine`] is the port re-shaped into
//! signatures that survive a language boundary, [`ShellEngineBridge`] turns one
//! back into a real [`PlaybackEngine`], and [`run_contract`] runs the one list
//! against it.
//!
//! **`nen-ffi` owns the FFI surface, not this crate** (ADR-0006 kural 2).
//! Nothing here mentions `uniffi`; the gate translates its own types into
//! these. Keeping the shapes FFI-*friendly* while staying FFI-*free* is what
//! lets the bridge be unit-tested here, with no bindings built.
//!
//! # What the boundary changes
//!
//! - **`&mut self` becomes `&self`.** A foreign object arrives behind an
//!   `Arc`; the bridge owns the mutable part (the queue) on this side.
//! - **`events()` cannot cross.** The port hands out `&mut EventQueue`, which
//!   is a Rust borrow. The foreign side reports through
//!   [`ShellEngine::drain_events`] instead and the bridge pushes what comes
//!   back into a queue **of its own** — so coalescing, overflow and
//!   `EventsLost` stay in [`nen_ports::playback::event`], identical for every
//!   adapter, rather than being re-implemented per platform.
//! - **A locator is a `String`.** It becomes a [`MediaSource`] on this side and
//!   is never logged (K23 #1/#3).
//!
//! # The reentrancy ban is applied here, not over there
//!
//! ADR-0011 Karar 2 is explicit that the ban belongs to the **port**, so that
//! "her adapter aynı yasağı miras alır". The bridge is the last piece of Rust
//! before a foreign engine, so this is where the inheritance has to happen: it
//! calls [`guard_reentrancy`] before forwarding anything.
//!
//! Leaving it to the adapter would mean every platform re-implementing a
//! thread-local check, and the first one to forget it would deadlock instead of
//! returning [`PlaybackError::ReentrantCall`] — which is the exact failure
//! NEN-029 measured and the exact reason the rule exists.

use nen_domain::subtitle::SubtitleDocument;
use nen_ports::playback::contract::{self, ContractInputs};
use nen_ports::playback::{
    guard_reentrancy, Capabilities, Capability, EventQueue, MediaSource, Operation, PlaybackEngine,
    PlaybackError, PlaybackEvent, PlaybackState, TrackDescriptor, TrackId, TrackKind,
};
use std::sync::Arc;
use std::time::Duration;

/// The port, in shapes that survive a language boundary.
///
/// Every method mirrors one on [`PlaybackEngine`]. The two differences are
/// forced by the boundary and documented on the module.
pub trait ShellEngine: Send + Sync {
    fn capabilities(&self) -> Vec<Capability>;

    fn load(&self, locator: String) -> Result<(), PlaybackError>;
    fn play(&self) -> Result<(), PlaybackError>;
    fn pause(&self) -> Result<(), PlaybackError>;
    fn stop(&self) -> Result<(), PlaybackError>;
    fn seek(&self, to_ms: u64) -> Result<(), PlaybackError>;

    fn position_ms(&self) -> Result<u64, PlaybackError>;
    /// `None` means the medium reports no duration — a live stream.
    fn duration_ms(&self) -> Result<Option<u64>, PlaybackError>;
    fn state(&self) -> PlaybackState;

    fn tracks(&self, kind: TrackKind) -> Result<Vec<TrackDescriptor>, PlaybackError>;
    fn select_track(&self, kind: TrackKind, track: Option<TrackId>) -> Result<(), PlaybackError>;
    fn selected_track(&self, kind: TrackKind) -> Result<Option<TrackId>, PlaybackError>;

    /// Everything the engine has reported since the last call.
    ///
    /// Order matters and must be the order things happened; the bridge pushes
    /// these into the shared queue, which is what applies ADR-0011 Karar 1.
    fn drain_events(&self) -> Vec<PlaybackEvent>;

    fn shutdown(&self) -> Result<(), PlaybackError>;

    fn set_rate(&self, rate: f32) -> Result<(), PlaybackError>;
    fn set_volume(&self, volume: f32) -> Result<(), PlaybackError>;
    /// The text is subtitle dialogue (K23 #4): displayable, never loggable.
    fn extract_text(&self, track: TrackId) -> Result<String, PlaybackError>;
    /// The document, already serialized as WebVTT — the form an engine that
    /// renders external subtitles actually wants (NEN-027).
    fn inject_subtitle(&self, webvtt: String) -> Result<(), PlaybackError>;
    /// What the engine is drawing right now, if anything.
    ///
    /// The text is subtitle dialogue (K23 #4): displayable and comparable,
    /// never loggable.
    fn rendered_subtitle_text(&self) -> Result<Option<String>, PlaybackError>;
}

/// Builds a fresh engine.
///
/// The kit rebuilds the engine per scenario so one cannot leave state behind
/// that makes the next pass — or fail — for the wrong reason. Across the
/// boundary that needs an object, not a closure.
pub trait ShellEngineFactory: Send + Sync {
    fn build(&self) -> Arc<dyn ShellEngine>;
}

/// A foreign engine, wearing the real port.
pub struct ShellEngineBridge {
    inner: Arc<dyn ShellEngine>,
    events: EventQueue,
}

impl ShellEngineBridge {
    pub fn new(inner: Arc<dyn ShellEngine>) -> Self {
        Self {
            inner,
            events: EventQueue::default(),
        }
    }

    /// Moves whatever the foreign engine has reported into the shared queue.
    ///
    /// Called before every observation rather than after every command,
    /// because a real engine reports on its own thread: an event can land
    /// between two calls, and only pulling immediately before a look can see
    /// it. `events()` itself cannot do this — it hands out a borrow.
    fn pull(&mut self) {
        for event in self.inner.drain_events() {
            self.events.push(event);
        }
    }
}

impl PlaybackEngine for ShellEngineBridge {
    fn capabilities(&self) -> Capabilities {
        Capabilities::new(self.inner.capabilities())
    }

    fn load(&mut self, source: &MediaSource) -> Result<(), PlaybackError> {
        guard_reentrancy(Operation::Load)?;
        let result = self.inner.load(source.locator().to_string());
        self.pull();
        result
    }

    fn play(&mut self) -> Result<(), PlaybackError> {
        guard_reentrancy(Operation::Play)?;
        let result = self.inner.play();
        self.pull();
        result
    }

    fn pause(&mut self) -> Result<(), PlaybackError> {
        guard_reentrancy(Operation::Pause)?;
        let result = self.inner.pause();
        self.pull();
        result
    }

    fn stop(&mut self) -> Result<(), PlaybackError> {
        guard_reentrancy(Operation::Stop)?;
        let result = self.inner.stop();
        self.pull();
        result
    }

    fn seek(&mut self, to: Duration) -> Result<(), PlaybackError> {
        guard_reentrancy(Operation::Seek)?;
        let result = self.inner.seek(to.as_millis().min(u64::MAX as u128) as u64);
        self.pull();
        result
    }

    fn position(&self) -> Result<Duration, PlaybackError> {
        guard_reentrancy(Operation::Position)?;
        self.inner.position_ms().map(Duration::from_millis)
    }

    fn duration(&self) -> Result<Option<Duration>, PlaybackError> {
        guard_reentrancy(Operation::Duration)?;
        self.inner
            .duration_ms()
            .map(|ms| ms.map(Duration::from_millis))
    }

    fn state(&self) -> PlaybackState {
        self.inner.state()
    }

    fn tracks(&self, kind: TrackKind) -> Result<Vec<TrackDescriptor>, PlaybackError> {
        guard_reentrancy(Operation::Tracks)?;
        self.inner.tracks(kind)
    }

    fn select_track(
        &mut self,
        kind: TrackKind,
        track: Option<TrackId>,
    ) -> Result<(), PlaybackError> {
        guard_reentrancy(Operation::SelectTrack)?;
        let result = self.inner.select_track(kind, track);
        self.pull();
        result
    }

    fn selected_track(&self, kind: TrackKind) -> Result<Option<TrackId>, PlaybackError> {
        guard_reentrancy(Operation::SelectTrack)?;
        self.inner.selected_track(kind)
    }

    fn events(&mut self) -> &mut EventQueue {
        self.pull();
        &mut self.events
    }

    fn shutdown(&mut self) -> Result<(), PlaybackError> {
        guard_reentrancy(Operation::Shutdown)?;
        let result = self.inner.shutdown();
        self.pull();
        result
    }

    fn set_rate(&mut self, rate: f32) -> Result<(), PlaybackError> {
        guard_reentrancy(Operation::SetRate)?;
        self.inner.set_rate(rate)
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), PlaybackError> {
        guard_reentrancy(Operation::SetVolume)?;
        self.inner.set_volume(volume)
    }

    fn extract_text(&mut self, track: TrackId) -> Result<String, PlaybackError> {
        guard_reentrancy(Operation::ExtractText)?;
        self.inner.extract_text(track)
    }

    fn inject_subtitle(&mut self, document: &SubtitleDocument) -> Result<(), PlaybackError> {
        guard_reentrancy(Operation::InjectSubtitle)?;
        self.inner
            .inject_subtitle(nen_subtitle::webvtt::write(document))
    }

    fn rendered_subtitle_text(&self) -> Result<Option<String>, PlaybackError> {
        guard_reentrancy(Operation::RenderedText)?;
        self.inner.rendered_subtitle_text()
    }
}

/// What the adapter's own fixture is, so the kit judges it against its medium.
///
/// The kit itself names no numbers (ADR-0011 Karar 4); these are the adapter's
/// and travel with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractFixture {
    /// The medium to load. **Never log this.**
    pub locator: String,
    pub duration_ms: Option<u64>,
    pub audio_track_count: u32,
    pub subtitle_track_count: u32,
    pub audio_track: u32,
    pub subtitle_track: u32,
    pub unknown_track: u32,
    pub seek_tolerance_ms: u64,
    pub settle_timeout_ms: u64,
}

impl ContractFixture {
    fn into_inputs(self) -> ContractInputs {
        ContractInputs::new(MediaSource::new(self.locator), injectable_document())
            .with_duration_ms(self.duration_ms)
            .with_track_counts(
                self.audio_track_count as usize,
                self.subtitle_track_count as usize,
            )
            .with_tracks(
                TrackId(self.audio_track),
                TrackId(self.subtitle_track),
                TrackId(self.unknown_track),
            )
            .with_seek_tolerance_ms(self.seek_tolerance_ms)
            .with_settle_timeout_ms(self.settle_timeout_ms)
    }
}

/// The document the injection scenarios hand the engine.
///
/// Deliberately trivial and built here rather than supplied by the adapter:
/// what injection scenarios prove is that the engine accepts a document, not
/// that it renders any particular one.
fn injectable_document() -> SubtitleDocument {
    use nen_domain::subtitle::{Cue, CueId, TimeSpan};
    let span = TimeSpan::new(1_000, 2_000).expect("a valid span");
    SubtitleDocument::new(vec![Cue::new(
        CueId::new(1),
        span,
        vec!["contract".to_string()],
    )])
}

/// The result of one contract run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractReport {
    /// One line per failing step, already rendered.
    ///
    /// Rendered here rather than handed over structurally because
    /// [`contract::Failure`] prints through `Debug` impls that are guarded
    /// against K23 leaks; re-modelling it for the boundary would mean
    /// re-earning that guarantee on the other side.
    pub failures: Vec<String>,
    /// How many scenarios actually ran.
    ///
    /// A run that skipped everything would otherwise report zero failures and
    /// look like a pass.
    pub applied: u32,
}

impl ContractReport {
    pub fn passed(&self) -> bool {
        self.failures.is_empty()
    }
}

/// How many scenarios apply to an engine declaring these capabilities.
///
/// Exposed so a shell can refuse a run that applied nothing, without pinning a
/// number the kit is free to grow past.
pub fn applicable_scenario_count(capabilities: Vec<Capability>) -> u32 {
    contract::applicable_count(Capabilities::new(capabilities)) as u32
}

/// Runs the one scenario list against a foreign engine.
pub fn run_contract(
    factory: Arc<dyn ShellEngineFactory>,
    fixture: ContractFixture,
) -> ContractReport {
    let inputs = fixture.into_inputs();

    // One engine is built up front only to read its capability set, then shut
    // down again. Asking the factory a second time *after* the run would leave
    // a real engine alive — an mpv handle is a process-wide resource, not a
    // struct — and asking mid-run would mean guessing which of the rebuilt
    // engines answered.
    let probe = factory.build();
    let capabilities = Capabilities::new(probe.capabilities());
    let _ = probe.shutdown();
    drop(probe);

    let failures = contract::run_all(|| ShellEngineBridge::new(factory.build()), &inputs);
    ContractReport {
        failures: failures.iter().map(ToString::to_string).collect(),
        applied: contract::applicable_count(capabilities) as u32,
    }
}
