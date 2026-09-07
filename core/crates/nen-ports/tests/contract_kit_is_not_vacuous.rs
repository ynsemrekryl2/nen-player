//! NEN-022: the loosened contract kit still catches a broken engine.
//!
//! The kit had to give ground for a real engine to be judged fairly. It now
//! **waits** for a state instead of demanding it instantly
//! ([`Action::Settle`]), allows a seek to land **near** where it was asked to,
//! and matches events as a **subsequence** so an engine may report more than
//! the contract requires.
//!
//! Every one of those is a place a defect could now hide. `contract_fake.rs`
//! proves a correct engine passes; that is only half an argument — a kit that
//! passed *everything* would pass it too. This file completes the argument by
//! breaking a correct engine in exactly the directions the kit now tolerates
//! and requiring it to go red each time.
//!
//! The twins wrap [`FakeEngine`] and change one thing each. Nothing here is
//! product code, and nothing here needed a test hook added to the port: a twin
//! keeps its **own** [`EventQueue`], drains the inner engine's into it, and
//! distorts what passes through. A defect that needed the port to grow a
//! back door would be a defect this file could not honestly claim to detect.

use nen_ports::playback::contract::{run_all, ContractInputs, Failure};
use nen_ports::playback::fake::{fake_inputs, FakeEngine};
use nen_ports::playback::{
    Capabilities, EventQueue, MediaSource, PlaybackEngine, PlaybackError, PlaybackEvent,
    PlaybackState, TrackDescriptor, TrackId, TrackKind, VideoGeometry,
};
use std::cell::Cell;
use std::time::Duration;

/// Which single thing a twin gets wrong.
#[derive(Clone, Copy, PartialEq)]
enum Defect {
    /// Lands 2 s away from every requested position.
    SeekDriftsFarther,
    /// Loads, but never leaves `Buffering`.
    NeverBecomesReady,
    /// Never reports that a seek completed.
    SwallowsSeekCompleted,
    /// Reports the seek's completion, in order, at the wrong position: the one
    /// the medium held *before* the seek. `position()` still answers correctly.
    ///
    /// The NEN-051 defect, as a twin. Until the kit judged the payload this
    /// passed everything — the shape was right, the order was right, and the
    /// `Position` step asked the engine rather than the event.
    AnswersSeekWithAStalePosition,
    /// Reports the seek's completion *after* the position it belongs to.
    ReportsSeekCompletionOutOfOrder,
    /// Slips an unasked `Failed` into an otherwise correct stream.
    ReportsAnUnaskedFailure,
    /// Accepts a track id that belongs to no track.
    AcceptsAnyTrackId,
    /// Answers the display size correctly, but never announces that it changed.
    ///
    /// The ADR-0038 shape of the NEN-051 defect: the value is right, so any
    /// check that merely reads it passes. What is missing is the *reason to
    /// read* — with no `VideoGeometryChanged` the shell never asks again and
    /// the window keeps the previous medium's aspect ratio.
    SwallowsVideoGeometryChanged,
    /// Announces the change and then reports the **stored** size instead of the
    /// display size.
    ///
    /// ADR-0038 Karar 3's failure mode: an adapter that hands over the raw
    /// frame size would open a 720x576 window for a 16:9 anamorphic stream.
    /// Modelled here by transposing the size, which is wrong in exactly the
    /// way an un-corrected aspect is: plausible numbers, wrong picture.
    ReportsTheWrongVideoGeometry,
    /// Refuses a seek for no reason but that the medium is still opening.
    ///
    /// ADR-0042's defect, modelled on the real one: before the decision,
    /// `MPVPlaybackEngine` answered a seek issued in the 2.5–12 ms window
    /// between `load()` returning and `FILE_LOADED`
    /// (`evidence/M3/NEN-052-measurement.md`) with
    /// `EngineFailure(code: -12)`. The kind of the error is not the point —
    /// any refusal here is what the new scenario exists to catch — so this
    /// twin reproduces the actual code for realism, not because the kit
    /// checks it.
    RefusesASeekWhileLoading,
}

/// How many `state()` polls [`Defect::RefusesASeekWhileLoading`] keeps
/// answering `Buffering` before it admits the medium is ready. Mirrors the
/// window `contract_fake.rs`'s `LoadingWindowEngine` uses, so the positive
/// and negative twins model the same thing from opposite sides.
const LOADING_WINDOW_POLLS: u32 = 3;

/// A [`FakeEngine`] with exactly one thing wrong.
struct BrokenEngine {
    inner: FakeEngine,
    defect: Defect,
    /// The twin's own queue. Everything the inner engine reports is moved here
    /// through [`BrokenEngine::forward_events`], which is where a defect that
    /// distorts the stream applies itself.
    events: EventQueue,
    /// Set once the medium is loaded, for the twin that lies about its state.
    loaded: bool,
    /// State-poll countdown for [`Defect::RefusesASeekWhileLoading`]: how many
    /// more `state()` calls answer `Buffering` before this twin stops
    /// refusing a seek. `Cell` because `state()` takes `&self` — the twin
    /// answers differently on each poll without needing a mutable borrow.
    loading_window: Cell<u32>,
}

impl BrokenEngine {
    fn new(defect: Defect) -> Self {
        Self {
            inner: FakeEngine::full(),
            defect,
            events: EventQueue::default(),
            loaded: false,
            loading_window: Cell::new(0),
        }
    }

    /// Moves the inner engine's pending events into the twin's own queue,
    /// applying whatever this twin gets wrong on the way.
    fn forward_events(&mut self) {
        let mut pending = self.inner.events().drain();
        match self.defect {
            Defect::SwallowsSeekCompleted => {
                pending.retain(|event| !matches!(event, PlaybackEvent::SeekCompleted { .. }));
            }
            Defect::ReportsSeekCompletionOutOfOrder => {
                if let Some(at) = pending
                    .iter()
                    .position(|event| matches!(event, PlaybackEvent::SeekCompleted { .. }))
                {
                    let completion = pending.remove(at);
                    pending.push(completion);
                }
            }
            Defect::AnswersSeekWithAStalePosition => {
                for event in pending.iter_mut() {
                    if let PlaybackEvent::SeekCompleted { position } = event {
                        *position = Duration::ZERO;
                    }
                }
            }
            Defect::SwallowsVideoGeometryChanged => {
                pending.retain(|event| !matches!(event, PlaybackEvent::VideoGeometryChanged));
            }
            Defect::ReportsAnUnaskedFailure => {
                pending.push(PlaybackEvent::Failed {
                    error: PlaybackError::EngineFailure { code: 7 },
                });
            }
            _ => {}
        }
        for event in pending {
            self.events.push(event);
        }
    }
}

impl PlaybackEngine for BrokenEngine {
    fn capabilities(&self) -> Capabilities {
        self.inner.capabilities()
    }

    fn load(&mut self, source: &MediaSource) -> Result<(), PlaybackError> {
        self.inner.load(source)?;
        self.loaded = true;
        if self.defect == Defect::RefusesASeekWhileLoading {
            self.loading_window.set(LOADING_WINDOW_POLLS);
        }
        self.forward_events();
        Ok(())
    }

    fn play(&mut self) -> Result<(), PlaybackError> {
        self.inner.play()?;
        self.forward_events();
        Ok(())
    }

    fn pause(&mut self) -> Result<(), PlaybackError> {
        self.inner.pause()?;
        self.forward_events();
        Ok(())
    }

    fn stop(&mut self) -> Result<(), PlaybackError> {
        self.inner.stop()?;
        self.loaded = false;
        self.loading_window.set(0);
        self.forward_events();
        Ok(())
    }

    fn seek(&mut self, to: Duration) -> Result<(), PlaybackError> {
        if self.defect == Defect::RefusesASeekWhileLoading && self.loading_window.get() > 0 {
            // The exact code the real adapter returned before ADR-0042: mpv's
            // `MPV_ERROR_COMMAND` (`-12`), mapped by `check()`. Any refusal
            // would do for the kit; this one is the measured one.
            return Err(PlaybackError::EngineFailure { code: -12 });
        }
        let target = if self.defect == Defect::SeekDriftsFarther {
            to + Duration::from_secs(2)
        } else {
            to
        };
        self.inner.seek(target)?;
        self.forward_events();
        Ok(())
    }

    fn position(&self) -> Result<Duration, PlaybackError> {
        self.inner.position()
    }

    fn duration(&self) -> Result<Option<Duration>, PlaybackError> {
        self.inner.duration()
    }

    fn video_geometry(&self) -> Result<Option<VideoGeometry>, PlaybackError> {
        let geometry = self.inner.video_geometry()?;
        if self.defect == Defect::ReportsTheWrongVideoGeometry {
            return Ok(geometry.and_then(|size| VideoGeometry::new(size.height(), size.width())));
        }
        Ok(geometry)
    }

    fn state(&self) -> PlaybackState {
        let state = self.inner.state();
        // The events still announce readiness; only the answer to `state()`
        // lags. That is the realistic shape of the bug — an adapter that
        // forgets to update what it reports.
        if self.defect == Defect::NeverBecomesReady && self.loaded {
            return PlaybackState::Buffering;
        }
        if self.defect == Defect::RefusesASeekWhileLoading {
            let remaining = self.loading_window.get();
            if remaining > 0 {
                self.loading_window.set(remaining - 1);
                return PlaybackState::Buffering;
            }
        }
        state
    }

    fn tracks(&self, kind: TrackKind) -> Result<Vec<TrackDescriptor>, PlaybackError> {
        self.inner.tracks(kind)
    }

    fn select_track(
        &mut self,
        kind: TrackKind,
        track: Option<TrackId>,
    ) -> Result<(), PlaybackError> {
        if self.defect == Defect::AcceptsAnyTrackId {
            // Say yes to anything. The inner engine never hears about the ids
            // it would have refused, so `selected_track` keeps answering
            // `None` — the classic "selection that silently did nothing".
            return Ok(());
        }
        self.inner.select_track(kind, track)
    }

    fn selected_track(&self, kind: TrackKind) -> Result<Option<TrackId>, PlaybackError> {
        self.inner.selected_track(kind)
    }

    fn events(&mut self) -> &mut EventQueue {
        // Forwarding cannot happen here: `events()` hands out a borrow, and the
        // runner drains through it. Every operation above forwards first.
        &mut self.events
    }

    fn shutdown(&mut self) -> Result<(), PlaybackError> {
        self.inner.shutdown()
    }

    fn set_rate(&mut self, rate: f32) -> Result<(), PlaybackError> {
        self.inner.set_rate(rate)
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), PlaybackError> {
        self.inner.set_volume(volume)
    }

    fn set_subtitle_bottom_inset(&mut self, fraction: f32) -> Result<(), PlaybackError> {
        self.inner.set_subtitle_bottom_inset(fraction)
    }

    fn extract_text(&mut self, track: TrackId) -> Result<String, PlaybackError> {
        self.inner.extract_text(track)
    }

    fn inject_subtitle(
        &mut self,
        document: &nen_domain::subtitle::SubtitleDocument,
    ) -> Result<(), PlaybackError> {
        self.inner.inject_subtitle(document)
    }
}

/// The tolerance a real adapter is allowed. Every twin is judged with it, so a
/// twin that fails is failing on behaviour and not on strictness.
const REALISTIC_TOLERANCE_MS: u64 = 250;

fn realistic_inputs() -> ContractInputs {
    fake_inputs()
        .with_seek_tolerance_ms(REALISTIC_TOLERANCE_MS)
        // Short, because every settle in these runs is expected to time out.
        .with_settle_timeout_ms(120)
}

fn failures_for(defect: Defect) -> Vec<Failure> {
    run_all(|| BrokenEngine::new(defect), &realistic_inputs())
}

fn report(failures: &[Failure]) -> String {
    failures
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_correct_engine_still_passes_under_a_real_tolerance() {
    // The control for the controls: loosening the inputs must not be what
    // makes the twins below fail.
    let failures = run_all(FakeEngine::full, &realistic_inputs());
    assert!(failures.is_empty(), "{}", report(&failures));
}

#[test]
fn a_seek_landing_outside_the_tolerance_is_caught() {
    // 2 s away, against a 250 ms tolerance: the point is that tolerance is a
    // bound, not a blanket.
    let failures = failures_for(Defect::SeekDriftsFarther);
    assert!(
        !failures.is_empty(),
        "a seek 2 s off passed a {REALISTIC_TOLERANCE_MS} ms tolerance"
    );
}

#[test]
fn a_seek_answered_with_a_stale_position_is_caught() {
    // The twin moves correctly and reports the completion in the right place;
    // only the position the event carries is wrong. Every shape-level
    // assertion in the kit is satisfied by it, which is exactly how the real
    // adapter's version of this defect survived until NEN-051 measured it.
    let failures = failures_for(Defect::AnswersSeekWithAStalePosition);
    assert!(
        !failures.is_empty(),
        "a seek answered at 0 ms passed while the engine's own position was right"
    );
}

#[test]
fn an_engine_that_never_becomes_ready_is_caught() {
    // What `Settle` exists to catch. Waiting for a state must not become
    // waiting forever and calling it success.
    let failures = failures_for(Defect::NeverBecomesReady);
    assert!(!failures.is_empty(), "a never-ready engine passed");
    assert!(
        failures
            .iter()
            .any(|failure| failure.detail.contains("never reached")),
        "it failed, but not on settling:\n{}",
        report(&failures)
    );
}

#[test]
fn a_dropped_critical_event_is_caught() {
    // Subsequence matching allows *extra* events. It must never allow a
    // missing one.
    let failures = failures_for(Defect::SwallowsSeekCompleted);
    assert!(!failures.is_empty(), "a swallowed SeekCompleted passed");
}

#[test]
fn events_arriving_out_of_order_are_caught() {
    // A subsequence is ordered. Reporting the right shapes in the wrong order
    // is a different bug from reporting extras, and must still fail.
    let failures = failures_for(Defect::ReportsSeekCompletionOutOfOrder);
    assert!(!failures.is_empty(), "reordered events passed");
}

#[test]
fn an_unasked_failure_event_is_caught() {
    // The reason `Failed` and `EventsLost` are exempt from "extras are fine":
    // otherwise tolerating extras would tolerate the engine reporting that
    // playback broke.
    let failures = failures_for(Defect::ReportsAnUnaskedFailure);
    assert!(!failures.is_empty(), "an unasked Failed event passed");
    assert!(
        failures
            .iter()
            .any(|failure| failure.detail.contains("unasked")),
        "it failed, but not on the unasked event:\n{}",
        report(&failures)
    );
}

#[test]
fn a_display_size_that_is_never_announced_is_caught() {
    // The value is right and every read of it succeeds. What the kit has to
    // catch is the missing announcement — without it the shell is never told
    // to look, which is the whole reason ADR-0038 Karar 2 has an event at all.
    let failures = failures_for(Defect::SwallowsVideoGeometryChanged);
    assert!(
        !failures.is_empty(),
        "an engine that answers the right size but never announces it passed"
    );
}

#[test]
fn a_display_size_that_is_the_wrong_size_is_caught() {
    // Announced correctly, read back wrong: the ADR-0038 Karar 3 defect, where
    // an adapter hands over the stored frame size instead of the display size.
    let failures = failures_for(Defect::ReportsTheWrongVideoGeometry);
    assert!(!failures.is_empty(), "a transposed display size passed");
}

#[test]
fn a_selection_that_accepts_any_id_is_caught() {
    // Nothing about this defect involves timing or tolerance; it is here to
    // show the symbolic `TrackRef` rewrite did not weaken the id scenarios.
    let failures = failures_for(Defect::AcceptsAnyTrackId);
    assert!(!failures.is_empty(), "an engine accepting any id passed");
}

#[test]
fn a_seek_refused_while_loading_is_caught() {
    // ADR-0042: refusing a seek for the sole reason that the medium is still
    // opening is the exact defect the decision closes. This twin fails on the
    // very first `Seek` step of the new scenario — before `Settle` ever
    // runs — which is also why it needs no timing margin to be deterministic.
    let failures = failures_for(Defect::RefusesASeekWhileLoading);
    assert!(
        !failures.is_empty(),
        "a seek refused merely because the medium was still opening passed"
    );
}
