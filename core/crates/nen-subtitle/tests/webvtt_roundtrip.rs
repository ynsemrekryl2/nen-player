//! SRT → doc → WebVTT round-trip golden tests (NEN-014 DoD).
//!
//! Every `fixtures/subtitles/valid/*.srt` parses (already proven by
//! `golden_valid.rs`) and its `webvtt::write` output must match the `.vtt`
//! snapshot beside it byte for byte.
//!
//! Regenerate the snapshots after an intentional format change with:
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test -p nen-subtitle --test webvtt_roundtrip
//! ```

mod support;

use std::fs;

use nen_subtitle::{srt, webvtt};

#[test]
fn every_valid_fixture_round_trips_to_its_vtt_golden() {
    let updating = std::env::var_os("UPDATE_GOLDEN").is_some();
    let files = support::srt_files("valid");
    assert!(!files.is_empty(), "the valid corpus is empty");

    for path in files {
        let file = support::name(&path);
        let input = support::read(&path);

        let document = srt::parse(&input)
            .unwrap_or_else(|err| panic!("{file}: expected a valid document, got {err}"));
        let rendered = webvtt::write(&document);

        let vtt_path = path.with_extension("vtt");
        if updating {
            fs::write(&vtt_path, &rendered)
                .unwrap_or_else(|err| panic!("cannot write {}: {err}", vtt_path.display()));
            continue;
        }

        assert!(
            vtt_path.exists(),
            "{file}: no .vtt snapshot; run UPDATE_GOLDEN=1 cargo test -p nen-subtitle --test webvtt_roundtrip"
        );
        assert_eq!(
            rendered,
            support::read(&vtt_path),
            "{file}: WebVTT output no longer matches its golden snapshot"
        );
    }
}

#[test]
fn output_is_utf8_and_bom_free() {
    for path in support::srt_files("valid") {
        let input = support::read(&path);
        let document = srt::parse(&input).unwrap_or_else(|err| panic!("{err}"));
        let rendered = webvtt::write(&document);
        assert!(
            !rendered.starts_with('\u{feff}'),
            "{}: WebVTT output starts with a BOM",
            support::name(&path)
        );
    }
}

#[test]
fn cue_identity_and_order_survive_the_round_trip() {
    let input = support::read(&support::corpus_dir("valid").join("multiline.srt"));
    let document = srt::parse(&input).unwrap_or_else(|err| panic!("{err}"));
    let rendered = webvtt::write(&document);

    let mut last_pos = 0;
    for cue in document.cues() {
        let id_line = cue.id().to_string();
        let start = rendered[last_pos..]
            .find(&id_line)
            .unwrap_or_else(|| panic!("cue id {id_line} not found after position {last_pos}"));
        last_pos += start + id_line.len();

        let start_ts = format!(
            "{:02}:{:02}:{:02}.{:03}",
            cue.span().start_ms() / 3_600_000,
            (cue.span().start_ms() / 60_000) % 60,
            (cue.span().start_ms() / 1_000) % 60,
            cue.span().start_ms() % 1_000,
        );
        assert!(
            rendered[last_pos..].starts_with(&format!("\n{start_ts} -->")),
            "cue {id_line}: expected timing line right after the identifier"
        );
    }
}
