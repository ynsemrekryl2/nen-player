//! Golden tests for the translation-block-layout fixture corpus (NEN-089
//! DoD #2).
//!
//! Every `fixtures/subtitles/blocks/*.srt` must parse, and its block layout
//! plus whole-document context must match the `.blocks.golden` snapshot
//! beside it byte for byte.
//!
//! Regenerate the snapshots after an intentional format change with:
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test -p nen-translate --test block_layout_golden
//! ```

mod support;

use std::fs;

#[test]
fn every_fixture_matches_its_golden() {
    let updating = std::env::var_os("UPDATE_GOLDEN").is_some();
    let files = support::srt_files();
    assert!(!files.is_empty(), "the block-layout corpus is empty");

    for path in files {
        let file = support::name(&path);
        let document = support::parse(&path);
        let rendered = support::render_golden(&document);

        let golden_path = path.with_extension("blocks.golden");
        if updating {
            fs::write(&golden_path, &rendered)
                .unwrap_or_else(|err| panic!("cannot write {}: {err}", golden_path.display()));
            continue;
        }

        assert!(
            golden_path.exists(),
            "{file}: no golden snapshot; run UPDATE_GOLDEN=1 cargo test -p nen-translate --test block_layout_golden"
        );
        assert_eq!(
            rendered,
            support::read(&golden_path),
            "{file}: rendered block layout does not match its golden snapshot"
        );
    }
}
