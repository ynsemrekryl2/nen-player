//! Helpers shared by this crate's integration tests.
//!
//! Test-only by construction: nothing here is compiled into the library, and
//! nothing here is product behaviour.
//!
//! Lives in a subdirectory so Cargo treats it as a module of each test crate
//! rather than as a test target of its own. Not every test crate uses every
//! helper, hence the blanket allow.
#![allow(dead_code)]

use nen_app::playback::ShellEngine;
use nen_domain::subtitle::SubtitleDocument;
use nen_ports::playback::fake::FakeEngine;
use nen_ports::playback::{
    Capabilities, Capability, MediaSource, PlaybackEngine, PlaybackError, PlaybackEvent,
    PlaybackState, TrackDescriptor, TrackId, TrackKind,
};
use std::sync::Mutex;
use std::time::Duration;

/// Reads back what `nen_subtitle::webvtt::write` wrote.
///
/// Deliberately minimal and test-only: it understands the shape this codebase
/// emits, not WebVTT at large. Parsing WebVTT is nobody's job in the product
/// (NEN-014 writes and does not read), and a general parser here would be
/// untested code standing next to the thing it is meant to check.
pub fn read_webvtt(text: &str) -> SubtitleDocument {
    use nen_domain::subtitle::{Cue, CueId, TimeSpan};

    fn timestamp(field: &str) -> u32 {
        let (hours, rest) = field.split_once(':').expect("HH:MM:SS.mmm");
        let (minutes, rest) = rest.split_once(':').expect("MM:SS.mmm");
        let (seconds, millis) = rest.split_once('.').expect("SS.mmm");
        let parse = |value: &str| value.parse::<u32>().expect("a number");
        parse(hours) * 3_600_000 + parse(minutes) * 60_000 + parse(seconds) * 1_000 + parse(millis)
    }

    let mut cues = Vec::new();
    for block in text.split("\n\n").skip(1) {
        let mut lines = block.lines();
        let Some(id) = lines.next() else { continue };
        let Some(timing) = lines.next() else { continue };
        let Some((start, end)) = timing.split_once(" --> ") else {
            continue;
        };
        let payload: Vec<String> = lines
            .map(|line| {
                line.replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&amp;", "&")
            })
            .collect();
        if payload.is_empty() {
            continue;
        }
        cues.push(Cue::new(
            CueId::new(id.parse().expect("a cue id")),
            TimeSpan::new(timestamp(start), timestamp(end)).expect("a valid span"),
            payload,
        ));
    }
    SubtitleDocument::new(cues)
}

/// A `ShellEngine` over the reference engine, wearing the shape a foreign
/// adapter wears.
///
/// The session only speaks to a `ShellEngine`, so a session test needs one.
/// What it adds over calling the fake directly is exactly what the boundary
/// adds: the document crosses as text and has to be read back before anything
/// can be drawn, which is what a real engine does with the string it is given.
pub struct FakeShell {
    inner: Mutex<FakeEngine>,
}

impl FakeShell {
    pub fn new(capabilities: Capabilities) -> Self {
        Self {
            inner: Mutex::new(FakeEngine::new(capabilities)),
        }
    }

    fn with<T>(&self, body: impl FnOnce(&mut FakeEngine) -> T) -> T {
        body(&mut self.inner.lock().expect("the fake never panics"))
    }

    /// What the engine underneath was actually told (ADR-0037 Karar 5).
    ///
    /// Reads the engine rather than the renderer on purpose: a session test
    /// that asked the renderer would be asking the thing that recorded the
    /// value, not the thing that has to act on it.
    pub fn subtitle_bottom_inset(&self) -> f32 {
        self.with(|engine| engine.subtitle_bottom_inset())
    }
}

impl ShellEngine for FakeShell {
    fn capabilities(&self) -> Vec<Capability> {
        self.with(|engine| engine.capabilities().iter().collect())
    }

    fn load(&self, locator: String) -> Result<(), PlaybackError> {
        self.with(|engine| engine.load(&MediaSource::new(locator)))
    }

    fn play(&self) -> Result<(), PlaybackError> {
        self.with(|engine| engine.play())
    }

    fn pause(&self) -> Result<(), PlaybackError> {
        self.with(|engine| engine.pause())
    }

    fn stop(&self) -> Result<(), PlaybackError> {
        self.with(|engine| engine.stop())
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
        self.with(|engine| engine.events().drain())
    }

    fn shutdown(&self) -> Result<(), PlaybackError> {
        self.with(|engine| engine.shutdown())
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

/// A deterministic xorshift, so a failing moment reproduces on any machine.
///
/// The same generator `nen-subtitle`'s parity test uses, for the same reason.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// Uniform-ish value in `[0, bound)`; `0` when `bound` is `0`.
    pub fn below(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            return 0;
        }
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        (x.wrapping_mul(0x2545_F491_4F6C_DD1D) % u64::from(bound)) as u32
    }
}
