//! Shared helpers for the `nen-identity` fixture-driven tests.
//!
//! Lives in a subdirectory so Cargo treats it as a module of each test crate
//! rather than as a test target of its own — the same arrangement
//! `nen-subtitle/tests/support` uses.
//!
//! Not every test crate uses every helper, hence the blanket allow.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use nen_identity::release_name::{MediaKind, ParsedName};

/// Root of the repository's media fixture corpus.
pub fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/media")
}

pub fn fixture(name: &str) -> PathBuf {
    corpus_dir().join(name)
}

pub fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()))
}

/// Every meaningful line of a fixture list: comments (`#`) and blank lines are
/// dropped so the corpus can document itself.
pub fn entries(path: &Path) -> Vec<String> {
    read(path)
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// Compares `rendered` against the `.golden` file beside `input_path`,
/// rewriting it instead when `UPDATE_GOLDEN` is set.
pub fn assert_golden(input_path: &Path, rendered: &str) {
    let golden_path = input_path.with_extension("golden");

    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        fs::write(&golden_path, rendered)
            .unwrap_or_else(|err| panic!("cannot write {}: {err}", golden_path.display()));
        return;
    }

    assert!(
        golden_path.exists(),
        "{}: no golden snapshot; run UPDATE_GOLDEN=1 cargo test -p nen-identity",
        golden_path.display()
    );
    assert_eq!(
        rendered,
        read(&golden_path),
        "{}: output no longer matches its golden snapshot",
        golden_path.display()
    );
}

/// Canonical one-line rendering of a parse, for golden snapshots.
///
/// `input <TAB> kind <TAB> title <TAB> year <TAB> season <TAB> episode`, with
/// `-` standing in for an absent field so every row has the same shape.
pub fn render(input: &str, parsed: &ParsedName) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}",
        input,
        kind_name(parsed.kind),
        parsed.title.as_deref().unwrap_or("-"),
        opt(parsed.year),
        opt(parsed.season),
        opt(parsed.episode),
    )
}

pub fn kind_name(kind: MediaKind) -> &'static str {
    match kind {
        MediaKind::Movie => "movie",
        MediaKind::Series => "series",
        MediaKind::Unknown => "unknown",
    }
}

fn opt(value: Option<u16>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}
