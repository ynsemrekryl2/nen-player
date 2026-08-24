//! Shared helpers for the `nen-subtitle` fixture-driven tests.
//!
//! Lives in a subdirectory so Cargo treats it as a module of each test crate
//! rather than as a test target of its own.
//!
//! Not every test crate uses every helper, hence the blanket allow.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use nen_domain::subtitle::SubtitleDocument;

/// Root of the repository's subtitle fixture corpus.
pub fn corpus_dir(kind: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../fixtures/subtitles")
        .join(kind)
}

/// Every `*.srt` file of a corpus, sorted so failures are reported in a
/// stable order across machines.
pub fn srt_files(kind: &str) -> Vec<PathBuf> {
    let dir = corpus_dir(kind);
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|err| panic!("cannot read {}: {err}", dir.display()));

    let mut files: Vec<PathBuf> = entries
        .map(|entry| entry.expect("cannot read directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "srt"))
        .collect();
    files.sort();
    files
}

pub fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()))
}

/// Reads a fixture as raw bytes rather than requiring it to already be valid
/// UTF-8 text — needed for the encoding corpus, which is deliberately not
/// UTF-8 for most of its files.
pub fn read_bytes(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()))
}

/// The file's name, for use in assertion messages.
pub fn name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// Canonical, deterministic rendering of a parsed document — the golden
/// snapshot format for the whole subtitle corpus.
///
/// One header line with the cue count, then one line per cue:
/// `id \t start_ms \t end_ms \t line_count \t text`, where the text has its
/// backslashes, newlines and tabs escaped so a cue always occupies exactly
/// one line.
pub fn render_golden(document: &SubtitleDocument) -> String {
    let mut out = format!("cues\t{}\n", document.len());
    for cue in document.cues() {
        let text = escape(&cue.lines().join("\n"));
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            cue.id().get(),
            cue.span().start_ms(),
            cue.span().end_ms(),
            cue.line_count(),
            text,
        ));
    }
    out
}

fn escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}
