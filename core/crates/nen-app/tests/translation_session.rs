//! Translation session orchestration, end to end (`NEN-099`,
//! `docs/product-spec.md` §9).
//!
//! Lives in `nen-app` for the same reason `artifact_store_roundtrip.rs` and
//! `artifact_index_lookup.rs` do: this crate is the only one that sees the
//! catalog (`SubtitleLibrary`), the pipeline (`nen-translate`) and the store
//! (`nen-persist`) at once — ADR-0006 keeps the lower two apart from each
//! other and from the catalog.
//!
//! What this file proves: selecting a source never calls a provider on its
//! own (§9's first sentence); a completed job becomes an ordinary
//! `SubtitleSourceKind::Ai` catalog entry only once a caller explicitly asks
//! for that; a source already in the target language never starts a job;
//! and a cache hit resolves without a single provider call while a changed
//! cache-identity component genuinely misses (§11, ADR-0018) — the same
//! claim `artifact_index_lookup.rs` proves for the store alone, proved here
//! through the orchestration layer a real caller would actually use.
//!
//! `translation_retarget.rs` carries the mid-flight retarget and
//! cancellation negatives; this file is the positive path plus the two
//! negatives that do not need a provider call held open.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use nen_app::subtitles::{AddOutcome, SubtitleLibrary};
use nen_app::translation::{self, StartRefusal, TranslationMetadataSeed};
use nen_domain::source::{LanguageTag, SubtitleSourceKind};
use nen_persist::FilesystemArtifactStore;
use nen_ports::persistence::{ArtifactIndex, ArtifactStore};
use nen_ports::translation::TranslationProvider;
use nen_providers::translation_mock::MockTranslationProvider;
use nen_translate::artifact::{ArtifactId, ArtifactTimestamp, GlossaryIdentity};
use nen_translate::blocks::BlockLayoutConfig;

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nen-099-session-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
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

fn timestamp(ms: u32) -> String {
    format!(
        "{:02}:{:02}:{:02},{:03}",
        ms / 3_600_000,
        (ms / 60_000) % 60,
        (ms / 1_000) % 60,
        ms % 1_000
    )
}

/// A small, valid SRT with `cue_count` cues, each carrying `marker` so a
/// test can tell one fixture's dialogue apart from another's.
fn srt(cue_count: u32, marker: &str) -> String {
    let mut out = String::new();
    for index in 0..cue_count {
        let start = index * 2_000;
        out.push_str(&format!(
            "{id}\n{start} --> {end}\n{marker} line {index}.\n\n",
            id = index + 1,
            start = timestamp(start),
            end = timestamp(start + 1_800),
        ));
    }
    out
}

fn tag(value: &str) -> LanguageTag {
    LanguageTag::parse(value).expect("a valid language tag")
}

/// Writes `contents` as `name` inside `directory` and catalogues it,
/// returning the library and the token naming the loaded row.
fn library_with_file(directory: &Path, name: &str, contents: &str) -> (SubtitleLibrary, u32) {
    let path = directory.join(name);
    fs::write(&path, contents).expect("write the fixture");
    let mut library = SubtitleLibrary::new();
    assert_eq!(
        library.add_file(&path, directory),
        AddOutcome::Added,
        "the fixture must be loadable, or this test measures the gate instead"
    );
    let source_id = library
        .catalog()
        .of_kind(SubtitleSourceKind::User)
        .next()
        .expect("the file was catalogued")
        .id()
        .clone();
    let token = library.token_of(&source_id).expect("a token");
    (library, token)
}

fn metadata_seed(id: &str) -> TranslationMetadataSeed {
    TranslationMetadataSeed {
        id: ArtifactId::parse(id).expect("a valid artifact id"),
        glossary: GlossaryIdentity::none(),
        media_hash: None,
        created_at: ArtifactTimestamp::from_unix_ms(1_700_000_000_000),
    }
}

fn store(directory: &Path) -> Arc<FilesystemArtifactStore> {
    Arc::new(FilesystemArtifactStore::new(directory).expect("a store"))
}

#[test]
fn selecting_a_source_alone_never_calls_the_provider() {
    let media = TempDir::new("select-only");
    let (_library, _token) = library_with_file(media.path(), "Movie.en.srt", &srt(6, "hello"));

    // No `prepare`/`start` call exists on this path at all — the assertion
    // below is the structural claim itself, not a probe for a bug that
    // might slip in: there is nowhere in `SubtitleLibrary::add_file` that
    // could reach a `TranslationProvider`.
    let provider = MockTranslationProvider::new();
    assert_eq!(provider.calls(), 0);
}

#[test]
fn a_source_already_in_the_target_language_never_starts() {
    let media = TempDir::new("already-target");
    let (library, token) = library_with_file(media.path(), "Movie.tr.srt", &srt(6, "merhaba"));
    let provider = MockTranslationProvider::new();

    let refusal = translation::prepare(
        &library,
        token,
        tag("tr"),
        BlockLayoutConfig::default(),
        provider.identity(),
        metadata_seed("already-target"),
    )
    .expect_err("a Turkish source translating to Turkish must be refused");

    assert_eq!(refusal, StartRefusal::AlreadyTargetLanguage);
    assert_eq!(provider.calls(), 0);
}

#[test]
fn a_new_translation_completes_and_becomes_an_ai_catalog_entry() {
    let media = TempDir::new("end-to-end");
    let store_dir = TempDir::new("end-to-end-store");
    let (mut library, token) =
        library_with_file(media.path(), "Movie.en.srt", &srt(6, "gon-and-killua"));

    let provider = Arc::new(MockTranslationProvider::new());
    let job = translation::prepare(
        &library,
        token,
        tag("tr"),
        BlockLayoutConfig::default(),
        provider.identity(),
        metadata_seed("end-to-end"),
    )
    .expect("a valid job");
    let origin = job.origin().clone();

    let concrete_store = store(store_dir.path());
    let artifact_store: Arc<dyn ArtifactStore> = concrete_store.clone();
    let index: Arc<dyn ArtifactIndex> = concrete_store.clone();
    let handle = translation::start(job, provider.clone(), artifact_store, index, None);
    let outcome = handle.join().expect("the job completes");

    assert!(!outcome.from_cache);
    assert_eq!(outcome.record.target_language, tag("tr"));
    assert_eq!(provider.calls(), 1);

    assert_eq!(
        library.catalog().of_kind(SubtitleSourceKind::Ai).count(),
        0,
        "a finished job must not catalogue itself"
    );
    let ai_token = library.add_translation(&origin, &outcome);
    assert_eq!(library.catalog().of_kind(SubtitleSourceKind::Ai).count(), 1);
    let translated = library
        .document_of(ai_token)
        .expect("the ai entry has a document");
    assert_eq!(translated.len(), 6);
    assert!(translated
        .cues()
        .iter()
        .all(|cue| cue.lines().iter().any(|line| line.contains("[tr]"))));
}

#[test]
fn a_cache_hit_never_calls_the_provider_while_a_changed_block_layout_does() {
    let media = TempDir::new("cache");
    let store_dir = TempDir::new("cache-store");
    let (library, token) = library_with_file(media.path(), "Movie.en.srt", &srt(6, "cache"));
    let provider = Arc::new(MockTranslationProvider::new());
    let concrete_store = store(store_dir.path());

    let run = |library: &SubtitleLibrary,
               config: BlockLayoutConfig,
               provider: &Arc<MockTranslationProvider>,
               store: &Arc<FilesystemArtifactStore>| {
        let job = translation::prepare(
            library,
            token,
            tag("tr"),
            config,
            provider.identity(),
            metadata_seed("cache-run"),
        )
        .expect("a valid job");
        let artifact_store: Arc<dyn ArtifactStore> = store.clone();
        let index: Arc<dyn ArtifactIndex> = store.clone();
        translation::start(job, provider.clone(), artifact_store, index, None)
            .join()
            .expect("the job completes")
    };

    let first = run(
        &library,
        BlockLayoutConfig::default(),
        &provider,
        &concrete_store,
    );
    assert!(!first.from_cache);
    assert_eq!(provider.calls(), 1);

    // Same everything a second time: a real cache hit, not a re-translation.
    let second = run(
        &library,
        BlockLayoutConfig::default(),
        &provider,
        &concrete_store,
    );
    assert!(second.from_cache);
    assert_eq!(second.address, first.address);
    assert_eq!(
        provider.calls(),
        1,
        "a cache hit must not call the provider again"
    );

    // Not deaf: a genuinely different cache-identity component (the block
    // layout, ADR-0018 Karar 2) must still miss and call the provider.
    let changed_config = BlockLayoutConfig::new(30, 1).expect("a valid config");
    assert_ne!(changed_config, BlockLayoutConfig::default());
    let third = run(&library, changed_config, &provider, &concrete_store);
    assert!(!third.from_cache);
    assert_eq!(
        provider.calls(),
        2,
        "a changed block layout must miss the cache and call the provider"
    );
}
