//! End-to-end: a real translation run becomes a stored artifact and comes
//! back off disk unchanged (`NEN-096`, ADR-0017).
//!
//! This test lives in `nen-app` because `nen-app` is the only crate that sees
//! both `nen-translate` (which assembles the artifact) and `nen-persist`
//! (which stores it) — ADR-0006 keeps `nen-persist` itself confined to
//! `nen-domain` + `nen-ports`, so it cannot reach a `ValidatedSubtitleArtifact`
//! on its own. Every other test of the store works on a hand-built
//! `ArtifactRecord`; this one proves the seam between the two halves.

use std::fs;
use std::path::{Path, PathBuf};

use nen_domain::source::LanguageTag;
use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_persist::FilesystemArtifactStore;
use nen_ports::persistence::ArtifactStore;
use nen_ports::translation::{TranslationCall, TranslationProvider};
use nen_providers::translation_mock::MockTranslationProvider;
use nen_translate::artifact::{
    self, ArtifactId, ArtifactMetadata, ArtifactTimestamp, GlossaryIdentity,
    ValidatedSubtitleArtifact,
};
use nen_translate::blocks::BlockLayout;
use nen_translate::checkpoint::{translate_checkpointed, BlockCheckpoints, TranslationPlan};

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!("nen-096-app-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("a temp directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn document(cue_count: u32) -> SubtitleDocument {
    let cues = (0..cue_count)
        .map(|index| {
            let start = index * 2_000;
            Cue::new(
                CueId::new(index + 1),
                TimeSpan::new(start, start + 1_800).expect("a valid span"),
                vec![
                    format!("Gon and Killua line {index}."),
                    "Second line.".to_owned(),
                ],
            )
        })
        .collect();
    SubtitleDocument::new(cues)
}

fn plan() -> TranslationPlan {
    TranslationPlan {
        source_language: LanguageTag::parse("en").expect("a valid tag"),
        target_language: LanguageTag::parse("tr").expect("a valid tag"),
        context_terms: Vec::new(),
    }
}

/// Runs `provider` over `document` and assembles the artifact — the same path
/// `NEN-099` will drive for real.
fn translated_artifact(
    provider: &dyn TranslationProvider,
    document: &SubtitleDocument,
) -> ValidatedSubtitleArtifact {
    let layout = BlockLayout::of(document, Default::default()).expect("a valid layout");
    let mut checkpoints = BlockCheckpoints::for_layout(&layout);
    translate_checkpointed(
        provider,
        document,
        &layout,
        &plan(),
        &TranslationCall::without_progress(),
        &mut checkpoints,
    )
    .expect("the run completes");
    let completed = checkpoints
        .into_completed()
        .expect("every block is checkpointed");

    let metadata = ArtifactMetadata {
        id: ArtifactId::parse("roundtrip-artifact").expect("a valid id"),
        provider: provider.identity(),
        glossary: GlossaryIdentity::none(),
        media_hash: None,
        created_at: ArtifactTimestamp::from_unix_ms(1_700_000_000_000),
    };

    artifact::assemble(document, &layout, &plan(), &completed, metadata).expect("assembly succeeds")
}

#[test]
fn an_assembled_artifact_survives_a_trip_through_the_store() {
    let dir = TempDir::new("roundtrip");
    let store = FilesystemArtifactStore::new(dir.path()).expect("a store");

    // One block; the multi-block path is covered by the test below.
    let source = document(30);
    let assembled = translated_artifact(&MockTranslationProvider::new(), &source);
    let record = assembled.to_record();

    let address = store.put(&record).expect("a committed artifact");
    let read_back = store.get(address).expect("the artifact reads back");

    assert_eq!(read_back, record);

    // The DoD's own words: cue IDs, order and timings are byte-for-byte the
    // source document's, all the way through the store.
    assert_eq!(read_back.document.len(), source.len());
    for (stored, original) in read_back.document.cues().iter().zip(source.cues()) {
        assert_eq!(stored.id(), original.id());
        assert_eq!(stored.span(), original.span());
    }
    assert_eq!(read_back.webvtt, assembled.webvtt());
    assert_eq!(
        read_back.source_fingerprint,
        *assembled.source_fingerprint().as_bytes()
    );
    assert_eq!(
        read_back.timeline_fingerprint,
        *assembled.timeline_fingerprint().as_bytes()
    );
    assert_eq!(read_back.provider, *assembled.provider());
    assert_eq!(read_back.pipeline_version, assembled.pipeline_version());
    assert_eq!(
        read_back.block_layout_version,
        assembled.block_layout_version()
    );
    assert_eq!(
        read_back.created_at_unix_ms,
        assembled.created_at().unix_ms()
    );
}

#[test]
fn the_same_translation_run_stores_at_the_same_address_twice() {
    let dir = TempDir::new("deterministic");
    let store = FilesystemArtifactStore::new(dir.path()).expect("a store");

    let source = document(30);
    // Two independent runs of the deterministic pipeline, from scratch.
    let first = store
        .put(&translated_artifact(&MockTranslationProvider::new(), &source).to_record())
        .expect("first commit");
    let second = store
        .put(&translated_artifact(&MockTranslationProvider::new(), &source).to_record())
        .expect("second commit");

    assert_eq!(
        first, second,
        "the deterministic pipeline produced two addresses for one input"
    );
    let stored = fs::read_dir(dir.path().join("artifacts"))
        .expect("the artifacts directory")
        .count();
    assert_eq!(stored, 1, "an identical rerun left a second copy");
}

/// A document large enough to be split into several overlapping blocks still
/// assembles into one artifact and survives the store unchanged — the cue
/// count, IDs and timings the DoD names are the source document's from end to
/// end.
#[test]
fn a_multi_block_artifact_survives_the_store() {
    let dir = TempDir::new("multi-block");
    let store = FilesystemArtifactStore::new(dir.path()).expect("a store");

    let source = document(95);
    let layout = BlockLayout::of(&source, Default::default()).expect("a valid layout");
    assert!(
        layout.blocks().len() > 1,
        "the fixture must actually span several blocks"
    );

    let assembled = translated_artifact(&MockTranslationProvider::new(), &source);
    let record = assembled.to_record();
    let address = store.put(&record).expect("a committed artifact");
    let read_back = store.get(address).expect("the artifact reads back");

    assert_eq!(read_back, record);
    assert_eq!(read_back.document.len(), source.len());
    for (stored, original) in read_back.document.cues().iter().zip(source.cues()) {
        assert_eq!(stored.id(), original.id());
        assert_eq!(stored.span(), original.span());
    }
    assert_eq!(read_back.webvtt, assembled.webvtt());
}
