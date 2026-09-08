//! Golden tests for the assembled translation artifact (NEN-094 DoD #1).
//!
//! Every `fixtures/subtitles/blocks/*.srt` is run through
//! [`support::EchoTranslationProvider`] to a completed, checkpointed run,
//! assembled into a [`nen_translate::artifact::ValidatedSubtitleArtifact`],
//! and its rendering (fingerprints, language pair, provider, versions, and
//! the full WebVTT body) must match the `.artifact.golden` snapshot beside
//! it byte for byte. Because block output ranges partition the whole
//! document, a byte-identical WebVTT body across a multi-block fixture is
//! itself evidence that the blocks' cue IDs, order and timings survived
//! assembly unchanged (M5 exit criterion: "Cue ID/sıra/zamanlar girdiyle
//! birebir aynı").
//!
//! Regenerate the snapshots after an intentional format change with:
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test -p nen-translate --test artifact_golden
//! ```

mod support;

use std::fs;

#[test]
fn every_fixture_artifact_matches_its_golden() {
    let updating = std::env::var_os("UPDATE_GOLDEN").is_some();
    let files = support::srt_files();
    assert!(!files.is_empty(), "the block-layout corpus is empty");

    for path in files {
        let file = support::name(&path);
        let document = support::parse(&path);
        let rendered = support::render_artifact_golden(&document);

        let golden_path = path.with_extension("artifact.golden");
        if updating {
            fs::write(&golden_path, &rendered)
                .unwrap_or_else(|err| panic!("cannot write {}: {err}", golden_path.display()));
            continue;
        }

        assert!(
            golden_path.exists(),
            "{file}: no golden snapshot; run UPDATE_GOLDEN=1 cargo test -p nen-translate --test artifact_golden"
        );
        assert_eq!(
            rendered,
            support::read(&golden_path),
            "{file}: rendered artifact does not match its golden snapshot"
        );
    }
}
