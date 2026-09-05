//! The FFI-shaped bridge passes the same kit as the port it mirrors.
//!
//! NEN-022 has to prove the real libmpv adapter passes the shared kit, and the
//! adapter is Swift (ADR-0012 Karar 2). Between the kit and the adapter sits
//! [`ShellEngineBridge`], which re-shapes the port for the boundary — and a
//! bridge that quietly mis-forwarded would make the Swift result meaningless in
//! either direction: a green run would prove nothing, and a red one would send
//! the search to the wrong side of the boundary.
//!
//! So the bridge is judged here first, with no bindings built and no engine
//! running: the fake wears exactly the shape Swift will wear, and the kit runs
//! through the bridge instead of straight at the engine. Whatever the Swift run
//! then reports is about Swift.

mod support;
use support::read_webvtt;

use nen_app::playback::{
    run_contract, ContractFixture, ContractReport, ShellEngine, ShellEngineBridge,
    ShellEngineFactory,
};
use nen_domain::subtitle::SubtitleDocument;
use nen_ports::playback::contract::{applicable_count, run_all};
use nen_ports::playback::fake::{
    fake_inputs, FakeEngine, FAKE_AUDIO_TRACK, FAKE_AUDIO_TRACKS, FAKE_DURATION_MS,
    FAKE_SUBTITLE_TRACK, FAKE_SUBTITLE_TRACKS, FAKE_UNKNOWN_TRACK, FAKE_VIDEO_HEIGHT,
    FAKE_VIDEO_WIDTH,
};
use nen_ports::playback::{
    CallbackScope, Capabilities, Capability, EventQueue, MediaSource, PlaybackEngine,
    PlaybackError, PlaybackEvent, PlaybackState, TrackDescriptor, TrackId, TrackKind,
    VideoGeometry,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Whether the foreign side behaves.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Correct,
    /// Reports nothing, ever — the shape a forwarding bug takes.
    SwallowsEvents,
}

/// The fake, wearing the shape a foreign adapter has to wear.
///
/// The `Mutex` is not decoration: across the boundary an engine arrives behind
/// an `Arc` and every method takes `&self`, so the interior mutability a real
/// adapter needs is reproduced here rather than assumed away.
struct FakeShellEngine {
    inner: Mutex<FakeEngine>,
    mode: Mode,
}

impl FakeShellEngine {
    fn new(capabilities: Capabilities, mode: Mode) -> Self {
        Self {
            inner: Mutex::new(FakeEngine::new(capabilities)),
            mode,
        }
    }

    fn with<T>(&self, body: impl FnOnce(&mut FakeEngine) -> T) -> T {
        body(&mut self.inner.lock().expect("the fake is not poisoned"))
    }
}

impl ShellEngine for FakeShellEngine {
    fn capabilities(&self) -> Vec<Capability> {
        let declared = self.with(|engine| engine.capabilities());
        Capability::ALL
            .into_iter()
            .filter(|capability| declared.contains(*capability))
            .collect()
    }

    fn load(&self, locator: String) -> Result<(), PlaybackError> {
        self.with(|engine| engine.load(&MediaSource::new(locator)))
    }

    fn play(&self) -> Result<(), PlaybackError> {
        self.with(PlaybackEngine::play)
    }

    fn pause(&self) -> Result<(), PlaybackError> {
        self.with(PlaybackEngine::pause)
    }

    fn stop(&self) -> Result<(), PlaybackError> {
        self.with(PlaybackEngine::stop)
    }

    fn seek(&self, to_ms: u64) -> Result<(), PlaybackError> {
        self.with(|engine| engine.seek(Duration::from_millis(to_ms)))
    }

    fn position_ms(&self) -> Result<u64, PlaybackError> {
        self.with(|engine| engine.position().map(|at| at.as_millis() as u64))
    }

    fn duration_ms(&self) -> Result<Option<u64>, PlaybackError> {
        self.with(|engine| {
            engine
                .duration()
                .map(|total| total.map(|value| value.as_millis() as u64))
        })
    }

    fn state(&self) -> PlaybackState {
        self.with(|engine| engine.state())
    }

    fn video_geometry(&self) -> Result<Option<VideoGeometry>, PlaybackError> {
        self.with(|engine| engine.video_geometry())
    }

    fn tracks(&self, kind: TrackKind) -> Result<Vec<TrackDescriptor>, PlaybackError> {
        self.with(|engine| engine.tracks(kind))
    }

    fn select_track(&self, kind: TrackKind, track: Option<TrackId>) -> Result<(), PlaybackError> {
        self.with(|engine| engine.select_track(kind, track))
    }

    fn selected_track(&self, kind: TrackKind) -> Result<Option<TrackId>, PlaybackError> {
        self.with(|engine| engine.selected_track(kind))
    }

    fn drain_events(&self) -> Vec<PlaybackEvent> {
        let reported = self.with(|engine| engine.events().drain());
        match self.mode {
            Mode::Correct => reported,
            Mode::SwallowsEvents => Vec::new(),
        }
    }

    fn shutdown(&self) -> Result<(), PlaybackError> {
        self.with(PlaybackEngine::shutdown)
    }

    fn set_rate(&self, rate: f32) -> Result<(), PlaybackError> {
        self.with(|engine| engine.set_rate(rate))
    }

    fn set_volume(&self, volume: f32) -> Result<(), PlaybackError> {
        self.with(|engine| engine.set_volume(volume))
    }

    fn extract_text(&self, track: TrackId) -> Result<String, PlaybackError> {
        self.with(|engine| engine.extract_text(track))
    }

    fn inject_subtitle(&self, webvtt: String) -> Result<(), PlaybackError> {
        // What crosses the boundary is text, so the far side has to turn it
        // back into cues before it can draw anything — which is exactly what a
        // real engine does with the string it is handed. Reading it back here
        // is what lets the kit ask the harder question through the bridge: not
        // "was a document accepted" but "is the document that arrived the one
        // that was sent".
        assert!(webvtt.starts_with("WEBVTT"), "the bridge sent {webvtt:?}");
        let document = read_webvtt(&webvtt);
        self.with(|engine| engine.inject_subtitle(&document))
    }

    fn rendered_subtitle_text(&self) -> Result<Option<String>, PlaybackError> {
        self.with(|engine| engine.rendered_subtitle_text())
    }

    fn set_subtitle_bottom_inset(&self, fraction: f32) -> Result<(), PlaybackError> {
        self.with(|engine| engine.set_subtitle_bottom_inset(fraction))
    }
}

struct FakeShellFactory {
    capabilities: Capabilities,
    mode: Mode,
}

impl ShellEngineFactory for FakeShellFactory {
    fn build(&self) -> Arc<dyn ShellEngine> {
        Arc::new(FakeShellEngine::new(self.capabilities, self.mode))
    }
}

fn fixture() -> ContractFixture {
    ContractFixture {
        locator: "fake://medium".to_string(),
        duration_ms: Some(FAKE_DURATION_MS),
        audio_track_count: FAKE_AUDIO_TRACKS as u32,
        subtitle_track_count: FAKE_SUBTITLE_TRACKS as u32,
        audio_track: FAKE_AUDIO_TRACK.index(),
        subtitle_track: FAKE_SUBTITLE_TRACK.index(),
        unknown_track: FAKE_UNKNOWN_TRACK.index(),
        // The fake stays exact through the bridge; allowing drift here would
        // hide a bridge that rounded.
        seek_tolerance_ms: 0,
        settle_timeout_ms: 1_000,
        video_geometry: VideoGeometry::new(FAKE_VIDEO_WIDTH, FAKE_VIDEO_HEIGHT),
    }
}

fn run(capabilities: Capabilities, mode: Mode) -> ContractReport {
    run_contract(Arc::new(FakeShellFactory { capabilities, mode }), fixture())
}

#[test]
fn a_fully_capable_engine_passes_through_the_bridge() {
    let report = run(Capabilities::ALL, Mode::Correct);
    assert!(report.passed(), "{}", report.failures.join("\n"));
}

#[test]
fn a_base_only_engine_passes_through_the_bridge() {
    let report = run(Capabilities::NONE, Mode::Correct);
    assert!(report.passed(), "{}", report.failures.join("\n"));
}

#[test]
fn every_capability_subset_passes_through_the_bridge() {
    for bits in 0..(1u8 << Capability::ALL.len()) {
        let declared: Vec<Capability> = Capability::ALL
            .into_iter()
            .enumerate()
            .filter(|(index, _)| bits & (1 << index) != 0)
            .map(|(_, capability)| capability)
            .collect();
        let report = run(Capabilities::new(declared.iter().copied()), Mode::Correct);
        assert!(
            report.passed(),
            "capabilities {declared:?}:\n{}",
            report.failures.join("\n")
        );
    }
}

#[test]
fn the_report_says_how_many_scenarios_ran() {
    // A run that applied nothing would report no failures and look like a
    // pass. The number has to come back so the caller can refuse that.
    let full = run(Capabilities::ALL, Mode::Correct);
    let minimal = run(Capabilities::NONE, Mode::Correct);
    assert_eq!(full.applied, applicable_count(Capabilities::ALL) as u32);
    assert_eq!(minimal.applied, applicable_count(Capabilities::NONE) as u32);
    assert!(
        full.applied >= 15,
        "only {} scenarios applied",
        full.applied
    );
}

#[test]
fn the_bridge_reaches_the_same_verdict_as_the_port_itself() {
    // The claim this file exists for: putting the bridge in the middle changes
    // nothing about what the kit decides. Same engine, same scenarios — one run
    // straight at the port, one through the boundary shape.
    let direct = run_all(FakeEngine::full, &fake_inputs());
    let bridged = run(Capabilities::ALL, Mode::Correct);
    assert!(direct.is_empty());
    assert!(bridged.passed());
    assert_eq!(bridged.applied, applicable_count(Capabilities::ALL) as u32);
}

#[test]
fn a_bridge_that_forwards_nothing_is_caught() {
    // The control: the bridge is not passing everything regardless. Forwarding
    // events *is* the bridge's job, so an engine whose reports never arrive
    // must fail — otherwise a green Swift run would prove nothing.
    let report = run(Capabilities::ALL, Mode::SwallowsEvents);
    assert!(
        !report.passed(),
        "a silent engine passed through the bridge"
    );
}

#[test]
fn a_fixture_that_lies_about_its_medium_is_caught() {
    // The other control, on the other input: the numbers now come from the
    // adapter's fixture, so a wrong fixture must fail rather than redefine the
    // contract.
    let mut lying = fixture();
    lying.subtitle_track_count += 1;
    lying.duration_ms = Some(FAKE_DURATION_MS + 60_000);
    let report = run_contract(
        Arc::new(FakeShellFactory {
            capabilities: Capabilities::ALL,
            mode: Mode::Correct,
        }),
        lying,
    );
    assert!(!report.passed(), "a fixture that lied passed");
}

/// An engine that must never be reached.
///
/// Every method panics, so any call that gets past the bridge is a loud test
/// failure rather than a quiet wrong answer.
struct NeverCalled;

macro_rules! never {
    () => {
        panic!("the bridge forwarded a call it had to refuse")
    };
}

impl ShellEngine for NeverCalled {
    fn capabilities(&self) -> Vec<Capability> {
        Capability::ALL.to_vec()
    }
    fn load(&self, _locator: String) -> Result<(), PlaybackError> {
        never!()
    }
    fn play(&self) -> Result<(), PlaybackError> {
        never!()
    }
    fn pause(&self) -> Result<(), PlaybackError> {
        never!()
    }
    fn stop(&self) -> Result<(), PlaybackError> {
        never!()
    }
    fn seek(&self, _to_ms: u64) -> Result<(), PlaybackError> {
        never!()
    }
    fn position_ms(&self) -> Result<u64, PlaybackError> {
        never!()
    }
    fn duration_ms(&self) -> Result<Option<u64>, PlaybackError> {
        never!()
    }
    fn state(&self) -> PlaybackState {
        PlaybackState::Ready
    }
    fn video_geometry(&self) -> Result<Option<VideoGeometry>, PlaybackError> {
        never!()
    }
    fn tracks(&self, _kind: TrackKind) -> Result<Vec<TrackDescriptor>, PlaybackError> {
        never!()
    }
    fn select_track(&self, _kind: TrackKind, _track: Option<TrackId>) -> Result<(), PlaybackError> {
        never!()
    }
    fn selected_track(&self, _kind: TrackKind) -> Result<Option<TrackId>, PlaybackError> {
        never!()
    }
    fn drain_events(&self) -> Vec<PlaybackEvent> {
        Vec::new()
    }
    fn shutdown(&self) -> Result<(), PlaybackError> {
        never!()
    }
    fn set_rate(&self, _rate: f32) -> Result<(), PlaybackError> {
        never!()
    }
    fn set_volume(&self, _volume: f32) -> Result<(), PlaybackError> {
        never!()
    }
    fn extract_text(&self, _track: TrackId) -> Result<String, PlaybackError> {
        never!()
    }
    fn inject_subtitle(&self, _webvtt: String) -> Result<(), PlaybackError> {
        never!()
    }
    fn rendered_subtitle_text(&self) -> Result<Option<String>, PlaybackError> {
        never!()
    }
    fn set_subtitle_bottom_inset(&self, _fraction: f32) -> Result<(), PlaybackError> {
        never!()
    }
}

#[test]
fn the_bridge_refuses_a_reentrant_call_before_reaching_the_engine() {
    // The bug this test was written for: the bridge forwarded every call
    // straight through, so ADR-0011 Karar 2 held only because the *fake*
    // happened to guard itself. A platform adapter cannot — the mark that makes
    // a reentrant call detectable is a Rust thread-local and Swift has never
    // heard of it. The adapter would have forwarded the call and, per NEN-029,
    // deadlocked against the delivery gate's own lock.
    //
    // Karar 2 says the ban belongs to the port so "her adapter aynı yasağı
    // miras alır". The bridge is the last Rust before a foreign engine, so the
    // refusal has to happen here — *before* forwarding, which is what an engine
    // that panics on contact proves.
    let mut bridge = ShellEngineBridge::new(Arc::new(NeverCalled));
    let _scope = CallbackScope::enter();

    macro_rules! refuses {
        ($name:literal, $call:expr) => {
            assert!(
                matches!($call, Err(PlaybackError::ReentrantCall { .. })),
                concat!($name, " was not refused as a reentrant call")
            );
        };
    }

    refuses!("load", bridge.load(&MediaSource::new("x")));
    refuses!("play", bridge.play());
    refuses!("pause", bridge.pause());
    refuses!("stop", bridge.stop());
    refuses!("seek", bridge.seek(Duration::from_millis(1)));
    refuses!("seek_relative", bridge.seek_relative(1));
    refuses!("position", bridge.position());
    refuses!("duration", bridge.duration());
    refuses!("video_geometry", bridge.video_geometry());
    refuses!("tracks", bridge.tracks(TrackKind::Subtitle));
    refuses!(
        "select_track",
        bridge.select_track(TrackKind::Subtitle, None)
    );
    refuses!("selected_track", bridge.selected_track(TrackKind::Subtitle));
    refuses!("set_rate", bridge.set_rate(1.5));
    refuses!("set_volume", bridge.set_volume(0.5));
    refuses!("extract_text", bridge.extract_text(TrackId(1)));
    refuses!(
        "inject_subtitle",
        bridge.inject_subtitle(&SubtitleDocument::new(Vec::new()))
    );
    refuses!("shutdown", bridge.shutdown());
}

#[test]
fn the_bridge_owns_the_queue_so_delivery_rules_are_not_re_implemented() {
    // ADR-0011 Karar 1 lives in the shared `EventQueue`. The bridge must push
    // what the foreign side reports *into* that queue rather than hand the
    // foreign list through — otherwise every platform would have to re-earn
    // coalescing, overflow and `EventsLost` for itself.
    let engine = Arc::new(FakeShellEngine::new(Capabilities::ALL, Mode::Correct));
    let mut bridge = ShellEngineBridge::new(engine);
    bridge
        .load(&MediaSource::new("fake://medium"))
        .expect("the fake loads");

    for _ in 0..(EventQueue::DEFAULT_CAPACITY * 4) {
        bridge
            .seek(Duration::from_millis(10))
            .expect("the fake seeks");
    }

    let drained = bridge.events().drain();
    let positions = drained
        .iter()
        .filter(|event| matches!(event, PlaybackEvent::PositionChanged { .. }))
        .count();
    assert!(
        positions <= 1,
        "positions were not coalesced: {positions} of them survived"
    );
}
