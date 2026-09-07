//! The reference adapter against the shared contract kit (NEN-021 DoD #1).
//!
//! ADR-0011 Karar 4 makes the scenarios data, so this file drives the same
//! list NEN-022 will drive through FFI against the real libmpv adapter —
//! `docs/milestones/M3-macos-slice.md` requires both to pass *the same* kit.

use nen_domain::subtitle::SubtitleDocument;
use nen_ports::playback::contract::{applicable_count, run_all, scenarios, Applicability};
use nen_ports::playback::fake::{fake_inputs, fake_inputs_without_video, FakeEngine};
use nen_ports::playback::{
    guard_reentrancy, Capabilities, Capability, EventQueue, MediaSource, Operation, PlaybackEngine,
    PlaybackError, PlaybackState, TrackDescriptor, TrackId, TrackKind, VideoGeometry,
};
use std::cell::{Cell, RefCell};
use std::time::Duration;

#[test]
fn a_fully_capable_engine_passes_every_applicable_scenario() {
    let inputs = fake_inputs();
    let failures = run_all(FakeEngine::full, &inputs);
    assert!(failures.is_empty(), "{}", report(&failures));
}

#[test]
fn a_base_only_engine_passes_every_applicable_scenario() {
    // The other half of Karar 3: an engine that declares nothing optional must
    // still satisfy the whole mandatory base, and refuse the rest by type.
    let inputs = fake_inputs();
    let failures = run_all(FakeEngine::minimal, &inputs);
    assert!(failures.is_empty(), "{}", report(&failures));
}

#[test]
fn a_medium_with_no_video_passes_every_applicable_scenario() {
    // ADR-0038 Karar 1's other half. `None` is a state and not a failure, so
    // the whole kit must be green against a medium that has no picture — and
    // the geometry scenario must exercise the `None` branch rather than being
    // skipped. An audio file is a medium the product plays.
    let inputs = fake_inputs_without_video();
    let failures = run_all(FakeEngine::without_video, &inputs);
    assert!(failures.is_empty(), "{}", report(&failures));
}

#[test]
fn the_geometry_scenario_really_distinguishes_the_two_media() {
    // The pairing that makes the run above worth anything: each engine is
    // judged against the *other* medium's fixture and must go red. Without
    // this, a kit that ignored geometry entirely would pass both runs.
    let with_video = run_all(FakeEngine::full, &fake_inputs_without_video());
    assert!(
        !with_video.is_empty(),
        "an engine with a picture passed a fixture that declares none"
    );

    let without_video = run_all(FakeEngine::without_video, &fake_inputs());
    assert!(
        !without_video.is_empty(),
        "an engine with no picture passed a fixture that declares one"
    );
}

#[test]
fn every_partial_capability_set_passes() {
    // Capabilities are independent (ADR-0011 Karar 3): declaring one must not
    // change how another behaves. Sweeping every subset is what proves it.
    let inputs = fake_inputs();
    for bits in 0..(1u8 << Capability::ALL.len()) {
        let declared: Vec<Capability> = Capability::ALL
            .into_iter()
            .enumerate()
            .filter(|(index, _)| bits & (1 << index) != 0)
            .map(|(_, capability)| capability)
            .collect();
        let capabilities = Capabilities::new(declared.iter().copied());
        let failures = run_all(|| FakeEngine::new(capabilities), &inputs);
        assert!(
            failures.is_empty(),
            "capabilities {declared:?}:\n{}",
            report(&failures)
        );
    }
}

#[test]
fn the_run_is_not_vacuous() {
    // A kit that skipped everything would "pass". Both ends of the capability
    // range must actually execute a large number of scenarios, and every
    // scenario must apply to at least one of them.
    let total = scenarios().len();
    let full = applicable_count(Capabilities::ALL);
    let minimal = applicable_count(Capabilities::NONE);

    assert!(total >= 20, "the kit only has {total} scenarios");
    assert!(full >= 15, "only {full} scenarios apply to a full engine");
    assert!(minimal >= 15, "only {minimal} apply to a minimal engine");
    // Every scenario is either unconditional or gated on a capability, so the
    // two runs together must cover the whole list.
    assert_eq!(
        full + minimal,
        total
            + scenarios()
                .iter()
                .filter(|scenario| scenario.applies == Applicability::Always)
                .count(),
        "some scenario applies to neither a full nor a minimal engine"
    );
}

#[test]
fn every_capability_is_covered_in_both_directions() {
    // For each capability there must be a scenario for having it and one for
    // not having it — otherwise "typed error when unsupported" is untested for
    // that capability.
    let all = scenarios();
    for capability in Capability::ALL {
        assert!(
            all.iter()
                .any(|s| s.applies == Applicability::WithCapability(capability)),
            "no scenario exercises {capability} when present"
        );
        assert!(
            all.iter()
                .any(|s| s.applies == Applicability::WithoutCapability(capability)),
            "no scenario exercises {capability} when absent"
        );
    }
}

#[test]
fn the_fake_declares_what_it_was_built_with() {
    assert_eq!(FakeEngine::full().capabilities(), Capabilities::ALL);
    assert_eq!(FakeEngine::minimal().capabilities(), Capabilities::NONE);
}

#[test]
fn an_engine_with_a_real_loading_window_still_passes_every_applicable_scenario() {
    // ADR-0042's positive reference. `FakeEngine` itself settles inside the
    // call to `load()`, so nothing in this kit has ever driven a seek issued
    // *while* the medium is still opening — the gap `NEN-051` measured and
    // `NEN-052` closes. `LoadingWindowEngine` reproduces the window
    // `evidence/M3/NEN-052-measurement.md` measured on the real adapter (a
    // few `state()` polls answer `Buffering` before `Ready`) and holds a seek
    // issued during it instead of losing it — the decision this ADR pins.
    // Passing the *whole* kit, not just the new scenario, is what proves the
    // window does not disturb anything the synchronous fake already satisfied.
    let inputs = fake_inputs();
    let failures = run_all(LoadingWindowEngine::new, &inputs);
    assert!(failures.is_empty(), "{}", report(&failures));
}

/// A [`FakeEngine`] whose `state()` answers `Buffering` for a short, measured
/// window after every `load()`, and holds a seek issued during that window
/// instead of losing it — the reference for ADR-0042's decision.
///
/// Built the same way `contract_kit_is_not_vacuous.rs`'s `BrokenEngine` is:
/// wraps [`FakeEngine`], keeps its own [`EventQueue`], and forwards what the
/// inner engine reports. No test hook was added to the port for this.
///
/// `state()` and the other read-only operations take `&self` on the port
/// (there is nowhere else to poll from), so counting the window down and
/// applying a seek that was waiting on it both need interior mutability —
/// [`Cell`] for the plain countdown, [`RefCell`] for the engine and queue a
/// deferred seek must reach into.
struct LoadingWindowEngine {
    inner: RefCell<FakeEngine>,
    events: RefCell<EventQueue>,
    /// `state()` polls left before this engine stops pretending to still be
    /// opening. Reset by every `load()`.
    window: Cell<u32>,
    /// A seek accepted while `window > 0`, applied the instant it reaches
    /// zero — mirrors `MPVPlaybackEngine` applying a deferred seek on
    /// `FILE_LOADED` rather than on a timer.
    deferred_seek: RefCell<Option<Duration>>,
}

/// How many `state()` polls the window lasts. Not the measured 2.5–12 ms
/// itself — this fake has no clock — but enough that `Action::Settle`'s real
/// polling loop (`contract.rs`, 5 ms apart) crosses it in a few iterations,
/// well inside any scenario's settle timeout.
const LOADING_WINDOW_POLLS: u32 = 3;

impl LoadingWindowEngine {
    fn new() -> Self {
        Self {
            inner: RefCell::new(FakeEngine::full()),
            events: RefCell::new(EventQueue::default()),
            window: Cell::new(0),
            deferred_seek: RefCell::new(None),
        }
    }

    /// Moves whatever the inner engine has queued into this engine's own
    /// queue. Called after every operation that could have produced events.
    fn forward_pending(&self) {
        let pending = self.inner.borrow_mut().events().drain();
        let mut events = self.events.borrow_mut();
        for event in pending {
            events.push(event);
        }
    }
}

impl PlaybackEngine for LoadingWindowEngine {
    fn capabilities(&self) -> Capabilities {
        self.inner.borrow().capabilities()
    }

    fn load(&mut self, source: &MediaSource) -> Result<(), PlaybackError> {
        self.inner.borrow_mut().load(source)?;
        self.window.set(LOADING_WINDOW_POLLS);
        *self.deferred_seek.borrow_mut() = None;
        self.forward_pending();
        Ok(())
    }

    fn play(&mut self) -> Result<(), PlaybackError> {
        let result = self.inner.borrow_mut().play();
        self.forward_pending();
        result
    }

    fn pause(&mut self) -> Result<(), PlaybackError> {
        let result = self.inner.borrow_mut().pause();
        self.forward_pending();
        result
    }

    fn stop(&mut self) -> Result<(), PlaybackError> {
        let result = self.inner.borrow_mut().stop();
        self.window.set(0);
        *self.deferred_seek.borrow_mut() = None;
        self.forward_pending();
        result
    }

    fn seek(&mut self, to: Duration) -> Result<(), PlaybackError> {
        guard_reentrancy(Operation::Seek)?;
        if self.window.get() > 0 {
            // ADR-0042 Karar: held, not refused. Only the latest of several
            // deferred seeks survives — the same trade the real adapter's
            // `pendingSeeks` count makes, and one no scenario here exercises
            // more than once.
            *self.deferred_seek.borrow_mut() = Some(to);
            return Ok(());
        }
        let result = self.inner.borrow_mut().seek(to);
        self.forward_pending();
        result
    }

    fn position(&self) -> Result<Duration, PlaybackError> {
        self.inner.borrow().position()
    }

    fn duration(&self) -> Result<Option<Duration>, PlaybackError> {
        self.inner.borrow().duration()
    }

    fn state(&self) -> PlaybackState {
        let remaining = self.window.get();
        if remaining == 0 {
            return self.inner.borrow().state();
        }
        self.window.set(remaining - 1);
        if remaining == 1 {
            // The window just closed: apply what was waiting on it, exactly
            // where the real adapter does — on the event that ends loading,
            // not on a poll of its own.
            if let Some(target) = self.deferred_seek.borrow_mut().take() {
                let _ = self.inner.borrow_mut().seek(target);
                self.forward_pending();
            }
        }
        PlaybackState::Buffering
    }

    fn video_geometry(&self) -> Result<Option<VideoGeometry>, PlaybackError> {
        self.inner.borrow().video_geometry()
    }

    fn tracks(&self, kind: TrackKind) -> Result<Vec<TrackDescriptor>, PlaybackError> {
        self.inner.borrow().tracks(kind)
    }

    fn select_track(
        &mut self,
        kind: TrackKind,
        track: Option<TrackId>,
    ) -> Result<(), PlaybackError> {
        let result = self.inner.borrow_mut().select_track(kind, track);
        self.forward_pending();
        result
    }

    fn selected_track(&self, kind: TrackKind) -> Result<Option<TrackId>, PlaybackError> {
        self.inner.borrow().selected_track(kind)
    }

    fn events(&mut self) -> &mut EventQueue {
        self.events.get_mut()
    }

    fn shutdown(&mut self) -> Result<(), PlaybackError> {
        let result = self.inner.borrow_mut().shutdown();
        self.window.set(0);
        *self.deferred_seek.borrow_mut() = None;
        result
    }

    fn set_rate(&mut self, rate: f32) -> Result<(), PlaybackError> {
        self.inner.borrow_mut().set_rate(rate)
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), PlaybackError> {
        self.inner.borrow_mut().set_volume(volume)
    }

    fn set_subtitle_bottom_inset(&mut self, fraction: f32) -> Result<(), PlaybackError> {
        self.inner.borrow_mut().set_subtitle_bottom_inset(fraction)
    }

    fn extract_text(&mut self, track: TrackId) -> Result<String, PlaybackError> {
        self.inner.borrow_mut().extract_text(track)
    }

    fn inject_subtitle(&mut self, document: &SubtitleDocument) -> Result<(), PlaybackError> {
        let result = self.inner.borrow_mut().inject_subtitle(document);
        self.forward_pending();
        result
    }

    fn rendered_subtitle_text(&self) -> Result<Option<String>, PlaybackError> {
        self.inner.borrow().rendered_subtitle_text()
    }
}

fn report(failures: &[nen_ports::playback::contract::Failure]) -> String {
    failures
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}
