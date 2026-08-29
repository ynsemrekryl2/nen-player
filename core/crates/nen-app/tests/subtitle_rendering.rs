//! Showing a subtitle, and proving the cue on screen is the right one
//! (NEN-027, ADR-0013).
//!
//! Two questions, and they need different instruments:
//!
//! 1. **Which row reaches the screen how.** An embedded row is a track the
//!    engine already has; a user's file is a document the renderer is given.
//!    ADR-0013 Karar 3 puts that decision in the session, so it is checked at
//!    the session.
//! 2. **Whether the cue drawn after a seek is the cue the document says.**
//!    Checked as a parity sweep: `CueIndex`'s answer (two binary searches,
//!    NEN-017) against what the engine reports it drew — and the reference
//!    engine finds that by walking the whole cue list, a method too dumb to be
//!    wrong. Same shape as `nen-subtitle`'s own lookup parity test, one layer
//!    up and across the boundary the shell will use.
//!
//! Nothing here asserts on the *content* of a cue beyond equality. The text is
//! dialogue (K23 #4): comparable, never printed. Assertion messages carry the
//! moment, not the line.

mod support;

use nen_app::playback::ShellEngine;
use nen_app::renderer::{EngineNativeRenderer, RendererState};
use nen_app::session::{PlaybackSession, ShowOutcome};
use nen_app::subtitles::{AddOutcome, SubtitleLibrary};
use nen_domain::source::SubtitleSourceKind;
use nen_domain::subtitle::SubtitleDocument;
use nen_ports::playback::fake::FakeEngine;
use nen_ports::playback::{Capabilities, Capability, MediaSource, PlaybackEngine, TrackKind};
use nen_ports::renderer::contract::{run_scenario, scenarios, Failure, RenderInputs};
use nen_ports::renderer::SubtitleRenderer;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use support::{FakeShell, Rng};

/// Fixed so a failing moment reproduces on every machine.
const SEED: u64 = 0x4E45_4E30_3237; // "NEN027"

/// How many moments the parity sweep visits.
const SEEK_COUNT: u32 = 4_000;

/// The medium the reference engine pretends to play is 120 s; every moment the
/// sweep visits has to be inside it.
const SWEEP_END_MS: u32 = 100_000;

struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nen-027-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temp dir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// An SRT with gaps, overlaps and multi-line cues — the three shapes a
/// renderer can get wrong in different ways.
fn corpus_srt() -> String {
    let mut out = String::new();
    let mut id = 1;
    let mut push = |id: &mut u32, start: u32, end: u32, lines: &str| {
        let stamp = |ms: u32| {
            format!(
                "{:02}:{:02}:{:02},{:03}",
                ms / 3_600_000,
                (ms / 60_000) % 60,
                (ms / 1_000) % 60,
                ms % 1_000
            )
        };
        out.push_str(&format!(
            "{id}\n{} --> {}\n{lines}\n\n",
            stamp(start),
            stamp(end)
        ));
        *id += 1;
    };

    // 200 cues on a 400 ms period with 150 ms gaps between them, plus an
    // overlapping second speaker every tenth cue and a two-line cue every
    // seventh — so the sweep lands in gaps, in single cues, and in stacks.
    for index in 0..200u32 {
        let start = index * 400;
        push(&mut id, start, start + 250, &format!("line {index}"));
        if index % 10 == 0 {
            push(&mut id, start + 100, start + 600, &format!("over {index}"));
        }
        if index % 7 == 0 {
            push(
                &mut id,
                start + 260,
                start + 390,
                &format!("first {index}\nsecond {index}"),
            );
        }
    }
    out
}

/// A library holding one user file, and the token that names it.
fn library_with_file(directory: &Path, contents: &str) -> (SubtitleLibrary, u32) {
    let path = directory.join("Clip.tr.srt");
    fs::write(&path, contents).expect("write the sidecar");
    let mut library = SubtitleLibrary::new();
    assert_eq!(
        library.add_file(&path, directory),
        AddOutcome::Added,
        "the fixture must be loadable, or the test measures the gate instead"
    );
    let token = library
        .menu(&Default::default())
        .into_iter()
        .flat_map(|section| section.entries)
        .find(|entry| entry.defect.is_none())
        .expect("a usable row")
        .token;
    (library, token)
}

fn session(capabilities: Capabilities) -> PlaybackSession {
    let engine: Arc<dyn ShellEngine> = Arc::new(FakeShell::new(capabilities));
    let session = PlaybackSession::without_pump(engine);
    session
        .load("fixtures/media/contract-clip.mkv".to_string())
        .expect("load");
    session
}

#[test]
fn a_user_file_reaches_the_screen_and_the_cue_matches_the_document_after_every_seek() {
    let directory = TempDir::new("parity");
    let (library, token) = library_with_file(directory.path(), &corpus_srt());
    let session = session(Capabilities::ALL);

    assert_eq!(
        session.show_source(&library, token).expect("shown"),
        ShowOutcome::Shown
    );

    let mut rng = Rng::new(SEED);
    let mut moments_with_a_cue = 0;
    let mut moments_in_a_gap = 0;

    for _ in 0..SEEK_COUNT {
        let moment = rng.below(SWEEP_END_MS);
        session
            .seek(Duration::from_millis(u64::from(moment)))
            .expect("seek");

        let expected = session.expected_subtitle_text(u64::from(moment));
        let drawn = session
            .rendered_subtitle_text()
            .expect("the engine reports");

        assert_eq!(
            expected.is_some(),
            drawn.is_some(),
            "at {moment} ms one side had a cue and the other did not"
        );
        assert_eq!(expected, drawn, "the cue at {moment} ms differs");

        if expected.is_some() {
            moments_with_a_cue += 1;
        } else {
            moments_in_a_gap += 1;
        }
    }

    // Printed rather than only asserted: the evidence record quotes these two
    // numbers, and a number nobody can reproduce is not evidence.
    println!("moments with a cue: {moments_with_a_cue}, moments in a gap: {moments_in_a_gap}");

    // Both halves of the DoD have to have actually happened. A sweep that only
    // ever landed inside cues would prove nothing about the empty moment, and
    // one that never landed in a cue would prove nothing at all.
    assert!(
        moments_with_a_cue > 500,
        "only {moments_with_a_cue} moments had a cue"
    );
    assert!(
        moments_in_a_gap > 500,
        "only {moments_in_a_gap} moments fell in a gap"
    );
}

#[test]
fn a_moment_past_the_last_cue_draws_nothing() {
    // The end of the document is a gap like any other, and the one a seek
    // reaches most easily by accident.
    let directory = TempDir::new("past-end");
    let (library, token) = library_with_file(directory.path(), &corpus_srt());
    let session = session(Capabilities::ALL);
    session.show_source(&library, token).expect("shown");

    session.seek(Duration::from_millis(95_000)).expect("seek");
    assert_eq!(session.expected_subtitle_text(95_000), None);
    assert_eq!(session.rendered_subtitle_text().expect("reports"), None);
}

#[test]
fn turning_subtitles_off_takes_the_document_off_screen() {
    let directory = TempDir::new("off");
    let (library, token) = library_with_file(directory.path(), &corpus_srt());
    let session = session(Capabilities::ALL);
    session.show_source(&library, token).expect("shown");
    session.seek(Duration::from_millis(100)).expect("seek");
    assert!(session.rendered_subtitle_text().expect("reports").is_some());

    session.hide_subtitle().expect("off");

    assert_eq!(session.rendered_subtitle_text().expect("reports"), None);
    assert_eq!(
        session.expected_subtitle_text(100),
        None,
        "nothing is showing, so nothing is expected either"
    );
}

#[test]
fn an_embedded_row_is_selected_on_the_engine_rather_than_drawn() {
    // The other half of ADR-0013 Karar 3: the same call, a different route,
    // and the shell never learns which.
    let mut engine = FakeEngine::full();
    engine
        .load(&MediaSource::new("fixtures/media/contract-clip.mkv"))
        .expect("load");
    let mut library = SubtitleLibrary::new();
    library.add_embedded(nen_app::embedded::embedded_sources(
        &engine.tracks(TrackKind::Subtitle).expect("tracks"),
    ));
    let token = library
        .menu(&Default::default())
        .into_iter()
        .flat_map(|section| section.entries)
        .find(|entry| entry.defect.is_none() && entry.kind == SubtitleSourceKind::Embedded)
        .expect("a usable embedded row")
        .token;

    let session = session(Capabilities::ALL);
    assert_eq!(
        session.show_source(&library, token).expect("shown"),
        ShowOutcome::Shown
    );

    assert_eq!(
        session.selected_track(TrackKind::Subtitle).expect("asked"),
        library.embedded_track_of(token),
        "the engine is not showing the track the row named"
    );
    assert_eq!(
        session.expected_subtitle_text(0),
        None,
        "an embedded track has no document on this side to expect anything from"
    );
}

#[test]
fn a_row_that_cannot_be_shown_changes_nothing() {
    let directory = TempDir::new("unusable");
    let (library, token) = library_with_file(directory.path(), &corpus_srt());
    let session = session(Capabilities::ALL);
    session.show_source(&library, token).expect("shown");
    session.seek(Duration::from_millis(100)).expect("seek");

    for stale in [0, token + 1, 9_999] {
        assert_eq!(
            session.show_source(&library, stale).expect("answered"),
            ShowOutcome::Unusable,
            "token {stale} was treated as a row"
        );
    }
    // The subtitle that was showing is still showing: a menu that shrank under
    // a click must not take the user's subtitle away.
    assert!(session.rendered_subtitle_text().expect("reports").is_some());
}

#[test]
fn loading_another_medium_forgets_the_document() {
    // The old document's timeline says nothing about the new medium's moments,
    // and answering from it would be a confident wrong answer.
    let directory = TempDir::new("reload");
    let (library, token) = library_with_file(directory.path(), &corpus_srt());
    let session = session(Capabilities::ALL);
    session.show_source(&library, token).expect("shown");
    session.seek(Duration::from_millis(100)).expect("seek");
    assert!(session.expected_subtitle_text(100).is_some());

    session
        .load("fixtures/media/menu-clip.mkv".to_string())
        .expect("load");

    assert_eq!(session.expected_subtitle_text(100), None);
}

#[test]
fn an_engine_that_cannot_draw_an_external_document_refuses_and_says_so() {
    // The negative control for the capability: a shell that gets this back has
    // a typed refusal to show, not a subtitle that silently never appears.
    let directory = TempDir::new("no-injection");
    let (library, token) = library_with_file(directory.path(), &corpus_srt());
    let session = session(Capabilities::ALL.without(Capability::ExternalSubtitleInjection));

    let error = session
        .show_source(&library, token)
        .expect_err("a row that cannot be drawn is a refusal, not a silent no-op");
    assert!(
        matches!(
            error,
            nen_ports::playback::PlaybackError::Unsupported {
                capability: Capability::ExternalSubtitleInjection,
                ..
            }
        ),
        "{error:?}"
    );
}

#[test]
fn the_engine_native_renderer_passes_the_renderer_contract_kit() {
    // The kit that M7's overlay will also have to pass. Driven scenario by
    // scenario rather than through `run_all`, because this renderer borrows the
    // engine it draws through and cannot be produced by a `Fn()`.
    let document = SubtitleDocument::new(
        nen_subtitle::srt::parse(&corpus_srt())
            .expect("the corpus parses")
            .cues()
            .to_vec(),
    );
    let moment = document.cues()[0].span().start_ms();
    let inputs = RenderInputs::new(document);

    for capabilities in [
        Capabilities::ALL,
        Capabilities::ALL.without(Capability::RenderedTextObservation),
    ] {
        let mut failures: Vec<Failure> = Vec::new();
        for scenario in scenarios() {
            let mut engine = FakeEngine::new(capabilities);
            engine
                .load(&MediaSource::new("fixtures/media/contract-clip.mkv"))
                .expect("load");
            // The kit's precondition: the surface is inside the document's
            // first cue, so "a shown document is on screen" is a question about
            // the renderer rather than about where time happens to be.
            engine
                .seek(Duration::from_millis(u64::from(moment)))
                .expect("seek");
            let mut state = RendererState::new();
            let mut renderer = EngineNativeRenderer::new(&mut engine, &mut state);
            if !scenario.applies.applies_to(renderer.capabilities()) {
                continue;
            }
            if let Some(failure) = run_scenario(&mut renderer, &scenario, &inputs) {
                failures.push(failure);
            }
        }
        assert!(
            failures.is_empty(),
            "engine capabilities {capabilities:?}:\n{}",
            failures
                .iter()
                .map(Failure::render)
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
