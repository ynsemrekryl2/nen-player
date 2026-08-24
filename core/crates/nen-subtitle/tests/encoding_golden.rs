//! Golden tests for the encoding corpus (NEN-015 DoD #1).
//!
//! Every fixture in `fixtures/subtitles/encodings/` that is expected to
//! decode successfully must produce text matching its `.decoded.golden`
//! snapshot byte for byte, and that text must still be valid SRT (a cheap
//! sanity check, not a second golden — cue-structure correctness is already
//! `srt.rs`'s job).
//!
//! `undecodable-garbage.srt` is deliberately excluded here — it is covered
//! as a negative case in `tests/encoding_negative.rs`. `POSITIVE_FIXTURES`
//! plus that one negative fixture is the whole corpus; the completeness test
//! below keeps that true.
//!
//! Regenerate the snapshots after an intentional format change with:
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test -p nen-subtitle --test encoding_golden
//! ```

mod support;

use std::fs;
use std::path::Path;

use nen_subtitle::{encoding, srt};

/// Fixtures expected to decode successfully.
const POSITIVE_FIXTURES: &[&str] = &[
    "utf8-bom.srt",
    "utf16le-bom.srt",
    "utf16be-bom.srt",
    "cp1254-turkish-no-bom.srt",
    "cp1252-no-bom.srt",
    "bidi-override-injection.srt",
    "zero-width-injection.srt",
];

/// The one fixture that is expected to fail to decode; covered in
/// `tests/encoding_negative.rs`, not here.
const NEGATIVE_FIXTURE: &str = "undecodable-garbage.srt";

#[test]
fn every_positive_fixture_matches_its_decoded_golden() {
    let updating = std::env::var_os("UPDATE_GOLDEN").is_some();

    for file in POSITIVE_FIXTURES {
        let path = support::corpus_dir("encodings").join(file);
        let bytes = support::read_bytes(&path);

        let decoded = encoding::decode(&bytes)
            .unwrap_or_else(|err| panic!("{file}: expected successful decode, got {err}"));

        // The decoded text must still be well-formed SRT — a sanity check
        // that decoding didn't produce nonsense, not a second golden.
        srt::parse(&decoded)
            .unwrap_or_else(|err| panic!("{file}: decoded text is not valid SRT: {err}"));

        let golden_path = path.with_extension("decoded.golden");
        if updating {
            fs::write(&golden_path, &decoded)
                .unwrap_or_else(|err| panic!("cannot write {}: {err}", golden_path.display()));
            continue;
        }

        assert!(
            golden_path.exists(),
            "{file}: no golden snapshot; run UPDATE_GOLDEN=1 cargo test -p nen-subtitle --test encoding_golden"
        );
        assert_eq!(
            decoded,
            support::read(&golden_path),
            "{file}: decoded text no longer matches its golden snapshot"
        );
    }
}

#[test]
fn decoding_is_deterministic_across_runs() {
    for file in POSITIVE_FIXTURES {
        let path = support::corpus_dir("encodings").join(file);
        let bytes = support::read_bytes(&path);

        let first = encoding::decode(&bytes).unwrap_or_else(|err| panic!("{file}: {err}"));
        let second = encoding::decode(&bytes).unwrap_or_else(|err| panic!("{file}: {err}"));
        assert_eq!(
            first, second,
            "{file}: two decodes of the same input disagree"
        );
    }
}

/// Guards the fixture list itself: every `*.srt` in the corpus must be
/// either a declared positive fixture or the one declared negative fixture —
/// a new file added without updating either list would otherwise sit
/// untested forever.
#[test]
fn the_corpus_is_fully_accounted_for() {
    let dir = support::corpus_dir("encodings");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|err| panic!("cannot read {}: {err}", dir.display()));

    let mut on_disk: Vec<String> = entries
        .map(|entry| entry.expect("cannot read directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "srt"))
        .map(|path| support::name(&path))
        .collect();
    on_disk.sort();

    let mut expected: Vec<String> = POSITIVE_FIXTURES
        .iter()
        .map(|&s| s.to_string())
        .chain(std::iter::once(NEGATIVE_FIXTURE.to_string()))
        .collect();
    expected.sort();

    assert_eq!(
        on_disk, expected,
        "the encodings corpus and the declared fixture lists have drifted apart"
    );
}

#[test]
fn the_corpus_has_no_orphan_goldens() {
    let dir = support::corpus_dir("encodings");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|err| panic!("cannot read {}: {err}", dir.display()));

    for entry in entries {
        let path = entry.expect("cannot read directory entry").path();
        if path.extension().is_some_and(|ext| ext == "golden") {
            let srt_path = orphan_check_srt_path(&path);
            assert!(
                srt_path.exists(),
                "{}: golden snapshot has no .srt fixture",
                support::name(&path)
            );
        }
    }
}

/// `foo.decoded.golden` -> `foo.srt`. `Path::with_extension` only strips one
/// extension, so `.decoded.golden`'s two-part suffix needs both stripped.
fn orphan_check_srt_path(golden_path: &Path) -> std::path::PathBuf {
    golden_path.with_extension("").with_extension("srt")
}
