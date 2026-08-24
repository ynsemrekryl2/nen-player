//! Golden tests for the valid SRT corpus (NEN-013 DoD #1).
//!
//! Every `fixtures/subtitles/valid/*.srt` must parse, and its canonical
//! rendering must match the `.golden` snapshot beside it byte for byte.
//!
//! Regenerate the snapshots after an intentional format change with:
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test -p nen-subtitle --test golden_valid
//! ```

mod support;

use std::fs;

use nen_subtitle::srt;

#[test]
fn every_valid_fixture_matches_its_golden() {
    let updating = std::env::var_os("UPDATE_GOLDEN").is_some();
    let files = support::srt_files("valid");
    assert!(!files.is_empty(), "the valid corpus is empty");

    for path in files {
        let file = support::name(&path);
        let input = support::read(&path);

        let document = srt::parse(&input)
            .unwrap_or_else(|err| panic!("{file}: expected a valid document, got {err}"));
        let rendered = support::render_golden(&document);

        let golden_path = path.with_extension("golden");
        if updating {
            fs::write(&golden_path, &rendered)
                .unwrap_or_else(|err| panic!("cannot write {}: {err}", golden_path.display()));
            continue;
        }

        assert!(
            golden_path.exists(),
            "{file}: no golden snapshot; run UPDATE_GOLDEN=1 cargo test -p nen-subtitle"
        );
        assert_eq!(
            rendered,
            support::read(&golden_path),
            "{file}: parsed document no longer matches its golden snapshot"
        );
    }
}

#[test]
fn parsing_is_deterministic_across_runs() {
    for path in support::srt_files("valid") {
        let input = support::read(&path);
        let first = srt::parse(&input).unwrap_or_else(|err| panic!("{err}"));
        let second = srt::parse(&input).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(
            support::render_golden(&first),
            support::render_golden(&second),
            "{}: two parses of the same input disagree",
            support::name(&path)
        );
    }
}

#[test]
fn the_valid_corpus_has_no_orphan_goldens() {
    let dir = support::corpus_dir("valid");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|err| panic!("cannot read {}: {err}", dir.display()));

    for entry in entries {
        let path = entry.expect("cannot read directory entry").path();
        if path.extension().is_some_and(|ext| ext == "golden") {
            assert!(
                path.with_extension("srt").exists(),
                "{}: golden snapshot has no .srt fixture",
                support::name(&path)
            );
        }
    }
}
