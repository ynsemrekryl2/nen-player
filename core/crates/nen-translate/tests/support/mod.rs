//! Shared helpers for the `nen-translate` fixture-driven tests.
//!
//! Lives in a subdirectory so Cargo treats it as a module of each test crate
//! rather than as a test target of its own. Mirrors
//! `nen-subtitle/tests/support/mod.rs`'s shape; not shared as a library
//! because these are test-only helpers (Kural 5 — no unrelated refactor to
//! introduce a shared dev-dependency crate for this).
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use nen_domain::subtitle::SubtitleDocument;
use nen_subtitle::srt;
use nen_translate::blocks::BlockLayout;
use nen_translate::context::DocumentContext;

/// Root of the repository's translation-block fixture corpus.
pub fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/subtitles/blocks")
}

/// Every `*.srt` file of the corpus, sorted so failures are reported in a
/// stable order across machines.
pub fn srt_files() -> Vec<PathBuf> {
    let dir = corpus_dir();
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

pub fn name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

pub fn parse(path: &Path) -> SubtitleDocument {
    let input = read(path);
    srt::parse(&input)
        .unwrap_or_else(|err| panic!("{}: expected a valid document, got {err}", name(path)))
}

/// Canonical, deterministic rendering of a document's default-config block
/// layout plus its whole-document context — the golden snapshot format for
/// the block-layout corpus.
pub fn render_golden(document: &SubtitleDocument) -> String {
    let layout = BlockLayout::of(document, Default::default())
        .unwrap_or_else(|err| panic!("expected a valid layout, got {err}"));
    let context = DocumentContext::of(document);

    let mut out = String::new();
    out.push_str(&format!(
        "version\t{}\nblock_size\t{}\noverlap\t{}\ncue_count\t{}\nblocks\t{}\n",
        layout.version(),
        layout.config().block_size(),
        layout.config().overlap(),
        layout.cue_count(),
        layout.blocks().len(),
    ));
    for block in layout.blocks() {
        let window = block.context_positions();
        let output = block.output_positions();
        out.push_str(&format!(
            "block\t{}\twindow\t{}..{}\toutput\t{}..{}\n",
            block.index(),
            window.start,
            window.end,
            output.start,
            output.end,
        ));
    }

    out.push_str(&format!("context_terms\t{}\n", context.terms().len()));
    for term in context.terms() {
        out.push_str(&format!("term\t{}\t{}\n", term.term(), term.occurrences()));
    }

    out
}
