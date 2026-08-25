//! Shared helpers for the `nen-subtitle` fixture-driven tests.
//!
//! Lives in a subdirectory so Cargo treats it as a module of each test crate
//! rather than as a test target of its own.
//!
//! Not every test crate uses every helper, hence the blanket allow.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};

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

/// xorshift64*, inlined to keep this crate dependency-free. Same generator as
/// `fuzz_smoke.rs`: a fixed seed makes any failure reproduce byte for byte on
/// any machine.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform-ish value in `[0, bound)`; `0` when `bound` is `0`.
    pub fn below(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            0
        } else {
            (self.next_u64() % u64::from(bound)) as u32
        }
    }
}

/// Spacing between generated cue starts, in milliseconds.
pub const GENERATED_CUE_PERIOD_MS: u32 = 2_000;

/// A synthetic document of `cue_count` cues where every moment inside the
/// document has at most `overlap_depth` cues on screen.
///
/// Cue `i` runs from `i * PERIOD` to `i * PERIOD + PERIOD * depth - 500`, so
/// `depth == 1` leaves a 500 ms gap between consecutive cues (the "no cue on
/// screen" case) and larger depths stack that many speakers. Deterministic:
/// same arguments, same document, always.
pub fn generated_document(cue_count: u32, overlap_depth: u32) -> SubtitleDocument {
    let depth = overlap_depth.max(1);
    let cues = (0..cue_count)
        .map(|i| {
            let start = i * GENERATED_CUE_PERIOD_MS;
            let end = start + GENERATED_CUE_PERIOD_MS * depth - 500;
            Cue::new(
                CueId::new(i + 1),
                TimeSpan::new(start, end).expect("generated span is well formed"),
                vec![format!("cue {i}")],
            )
        })
        .collect();
    SubtitleDocument::new(cues)
}

/// The reference implementation every lookup is checked against: a linear scan
/// over the whole document, in document order.
///
/// Deliberately the dumbest possible version — its only job is to be
/// obviously correct.
pub fn linear_active_cues(document: &SubtitleDocument, at_ms: u32) -> Vec<&Cue> {
    document
        .cues()
        .iter()
        .filter(|cue| cue.span().start_ms() <= at_ms && at_ms < cue.span().end_ms())
        .collect()
}

/// Linear-scan counterpart of a range query: every cue sharing a millisecond
/// with `[from, to)`.
pub fn linear_cues_in(document: &SubtitleDocument, from: u32, to: u32) -> Vec<&Cue> {
    if to <= from {
        return Vec::new();
    }
    document
        .cues()
        .iter()
        .filter(|cue| cue.span().start_ms() < to && cue.span().end_ms() > from)
        .collect()
}

/// Cue ids of a result, which is what parity assertions compare.
pub fn cue_ids(cues: &[&Cue]) -> Vec<u32> {
    cues.iter().map(|cue| cue.id().get()).collect()
}

/// One past the last millisecond any cue in `document` is on screen.
pub fn document_end_ms(document: &SubtitleDocument) -> u32 {
    document
        .cues()
        .iter()
        .map(|cue| cue.span().end_ms())
        .max()
        .unwrap_or(0)
}
