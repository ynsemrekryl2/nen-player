//! End-to-end: a stored artifact is found by its own cache identity, and a
//! query built from a **changed** cache identity component genuinely misses
//! it — through the real pipeline and the real filesystem store, not only
//! the pure `HashMap`-based negative tests in `nen-translate`'s
//! `cache_identity_negative.rs` (`NEN-098`, ADR-0018 Karar 6).
//!
//! Lives in `nen-app` for the same reason `artifact_store_roundtrip.rs`
//! does: `nen-app` is the only crate that sees both `nen-translate` (which
//! assembles the artifact and computes its identity) and `nen-persist`
//! (which stores and indexes it) — ADR-0006 keeps `nen-persist` confined to
//! `nen-domain` + `nen-ports`.

use std::fs;
use std::path::{Path, PathBuf};

use nen_domain::source::LanguageTag;
use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_persist::FilesystemArtifactStore;
use nen_ports::persistence::{ArtifactIndex, ArtifactStore};
use nen_ports::translation::TranslationProvider;
use nen_providers::translation_mock::MockTranslationProvider;
use nen_translate::artifact::{
    self, ArtifactId, ArtifactMetadata, ArtifactTimestamp, GlossaryIdentity,
    ValidatedSubtitleArtifact,
};
use nen_translate::blocks::{BlockLayout, BlockLayoutConfig};
use nen_translate::checkpoint::{translate_checkpointed, BlockCheckpoints, TranslationPlan};

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!("nen-098-app-{tag}-{}", std::process::id()));
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
                vec![format!("Gon and Killua line {index}.")],
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

/// Runs the real mock provider over `document` under `config` and assembles
/// the artifact — the only thing varied across the two builds below is the
/// block layout config, i.e. one of ADR-0018's own cache identity
/// components.
fn translated_artifact(
    document: &SubtitleDocument,
    config: BlockLayoutConfig,
) -> ValidatedSubtitleArtifact {
    let provider = MockTranslationProvider::new();
    let layout = BlockLayout::of(document, config).expect("a valid layout");
    let mut checkpoints = BlockCheckpoints::for_layout(&layout);
    translate_checkpointed(
        &provider,
        document,
        &layout,
        &plan(),
        &nen_ports::translation::TranslationCall::without_progress(),
        &mut checkpoints,
    )
    .expect("the run completes");
    let completed = checkpoints
        .into_completed()
        .expect("every block is checkpointed");

    let metadata = ArtifactMetadata {
        id: ArtifactId::parse("index-lookup-artifact").expect("a valid id"),
        provider: provider.identity(),
        glossary: GlossaryIdentity::none(),
        media_hash: None,
        created_at: ArtifactTimestamp::from_unix_ms(1_700_000_000_000),
    };

    artifact::assemble(document, &layout, &plan(), &completed, metadata).expect("assembly succeeds")
}

#[test]
fn a_stored_artifact_is_found_by_its_own_cache_identity() {
    let dir = TempDir::new("find");
    let store = FilesystemArtifactStore::new(dir.path()).expect("a store");

    let source = document(30);
    let assembled = translated_artifact(&source, BlockLayoutConfig::default());
    let record = assembled.to_record();
    let address = store.put(&record).expect("a committed artifact");

    let found = store
        .find(record.cache_identity)
        .expect("the index scan succeeds")
        .expect("the artifact is found by its own cache identity");
    assert_eq!(found.address, address);
}

/// The DoD's central negative claim, end to end: changing a cache identity
/// component (here, the block layout config — `block size`/`overlap`) and
/// re-deriving the identity produces a key that a real, on-disk index does
/// not resolve to the artifact built under the old config.
#[test]
fn a_query_under_a_changed_block_layout_component_does_not_find_the_old_artifact() {
    let dir = TempDir::new("stale-component");
    let store = FilesystemArtifactStore::new(dir.path()).expect("a store");

    let source = document(30);
    let old_config = BlockLayoutConfig::default(); // block_size 40, overlap 6
    let new_config = BlockLayoutConfig::new(34, 8).expect("a valid config");
    assert_ne!(
        old_config, new_config,
        "the fixture must actually vary the block layout"
    );

    let old_artifact = translated_artifact(&source, old_config);
    let new_artifact = translated_artifact(&source, new_config);
    assert_ne!(
        old_artifact.cache_identity(),
        new_artifact.cache_identity(),
        "changing the block layout must change the cache identity"
    );

    // Only the artifact built under the *new* config is ever stored — the
    // way a real reader would after the config changed.
    let new_record = new_artifact.to_record();
    let new_address = store.put(&new_record).expect("a committed artifact");

    // A lookup keyed on the *old*, now-stale identity must miss.
    let stale_key = old_artifact.cache_identity().into();
    assert_eq!(
        store.find(stale_key).expect("the index scan succeeds"),
        None,
        "a query under a changed cache identity component found the old artifact"
    );

    // Not deaf: the *new* identity does resolve to what was actually stored.
    let fresh_key = new_artifact.cache_identity().into();
    let found = store
        .find(fresh_key)
        .expect("the index scan succeeds")
        .expect("the freshly stored artifact is found by its own identity");
    assert_eq!(found.address, new_address);
}
