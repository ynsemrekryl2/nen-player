//! Golden corpus for NEN-020 / ADR-0029.
//!
//! Every language fixture is strict-SRT parsed, resolved without metadata and
//! compared byte-for-byte with its canonical `LanguageTag` snapshot. `unknown`
//! proves the confidence gate instead of blessing the detector's best guess.

mod support;

use std::fs;
use std::path::Path;

use nen_subtitle::{language, srt};

#[test]
fn every_language_fixture_matches_its_golden() {
    let files = support::srt_files("languages");
    assert!(!files.is_empty(), "the language corpus is empty");

    for path in files {
        let file = support::name(&path);
        let document = srt::parse(&support::read(&path))
            .unwrap_or_else(|err| panic!("{file}: invalid SRT fixture: {err}"));
        let resolution = language::resolve_language(&document, None)
            .unwrap_or_else(|err| panic!("{file}: language resolution failed: {err}"));
        let rendered = format!(
            "{}\n",
            resolution
                .language()
                .map_or("unknown", |language| language.as_str())
        );
        let golden_path = path.with_extension("language.golden");

        assert!(
            golden_path.exists(),
            "{file}: missing {}",
            golden_path.display()
        );
        assert_eq!(
            rendered,
            support::read(&golden_path),
            "{file}: detected language changed"
        );
    }
}

#[test]
fn language_corpus_has_no_orphan_goldens() {
    let dir = support::corpus_dir("languages");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|err| panic!("cannot read {}: {err}", dir.display()));

    for entry in entries {
        let path = entry.expect("cannot read directory entry").path();
        if path.extension().is_some_and(|ext| ext == "golden") {
            let srt_path = language_golden_srt_path(&path);
            assert!(
                srt_path.exists(),
                "{}: golden snapshot has no .srt fixture",
                support::name(&path)
            );
        }
    }
}

fn language_golden_srt_path(golden_path: &Path) -> std::path::PathBuf {
    golden_path.with_extension("").with_extension("srt")
}
