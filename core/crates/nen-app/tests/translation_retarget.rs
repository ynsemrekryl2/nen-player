//! §9's retarget and cancellation negatives (`NEN-099`).
//!
//! *"Çeviri sırasında kullanıcı başka source seçerse: mevcut iş başlangıç
//! source fingerprint'ine bağlı kalır · yeni source'a retarget edilmez ·
//! kullanıcı iptal edebilir · kullanıcı başka source izliyorsa zorla AI
//! çıktısına geçilmez."* — every clause here is measured against a job that
//! is genuinely in flight, held there by
//! [`nen_providers::translation_mock::MockCallGate`], while the main thread
//! does the things a user's next click would do: pick a different source,
//! or cancel.

mod support;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use nen_app::playback::ShellEngine;
use nen_app::session::{PlaybackSession, ShowOutcome};
use nen_app::subtitles::{AddOutcome, SubtitleLibrary};
use nen_app::translation::{self, TranslationError, TranslationMetadataSeed};
use nen_domain::source::{LanguageTag, SubtitleSourceKind};
use nen_persist::FilesystemArtifactStore;
use nen_ports::persistence::{ArtifactIndex, ArtifactStore};
use nen_ports::playback::Capabilities;
use nen_ports::translation::TranslationProvider;
use nen_providers::translation_mock::{MockCallGate, MockTranslationProvider};
use nen_subtitle::fingerprint::SourceFingerprint;
use nen_translate::artifact::{ArtifactId, ArtifactTimestamp, GlossaryIdentity};
use nen_translate::blocks::BlockLayoutConfig;
use support::FakeShell;

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nen-099-retarget-{tag}-{}-{:?}",
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
/// returning the token naming the newly loaded row.
///
/// Each call writes a distinctly named file, so `SubtitleSourceId::user`'s
/// path-digest identity (ADR-0010 Karar 2) always mints a new entry — the
/// most recently inserted `User` row is unambiguously the one this call just
/// added (`SubtitleSourceCatalog` preserves insertion order).
fn add(library: &mut SubtitleLibrary, directory: &Path, name: &str, contents: &str) -> u32 {
    let path = directory.join(name);
    fs::write(&path, contents).expect("write the fixture");
    assert_eq!(
        library.add_file(&path, directory),
        AddOutcome::Added,
        "the fixture must be loadable, or this test measures the gate instead"
    );
    let source_id = library
        .catalog()
        .of_kind(SubtitleSourceKind::User)
        .last()
        .expect("the file was just catalogued")
        .id()
        .clone();
    library.token_of(&source_id).expect("a token")
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

fn session() -> PlaybackSession {
    let engine: Arc<dyn ShellEngine> = Arc::new(FakeShell::new(Capabilities::ALL));
    let session = PlaybackSession::without_pump(engine);
    session
        .load("fixtures/media/contract-clip.mkv".to_string())
        .expect("load");
    session
}

#[test]
fn a_running_job_keeps_the_start_fingerprint_and_the_ai_output_is_never_forced_onto_screen() {
    let media = TempDir::new("mid-flight");
    let store_dir = TempDir::new("mid-flight-store");
    let mut library = SubtitleLibrary::new();
    let first_token = add(
        &mut library,
        media.path(),
        "First.en.srt",
        &srt(6, "first-source"),
    );
    let second_token = add(
        &mut library,
        media.path(),
        "Second.en.srt",
        &srt(6, "second-source"),
    );

    let first_fingerprint =
        SourceFingerprint::of(library.document_of(first_token).expect("a document"));
    let second_fingerprint =
        SourceFingerprint::of(library.document_of(second_token).expect("a document"));
    assert_ne!(
        first_fingerprint, second_fingerprint,
        "the fixture must actually vary the source"
    );

    let session = session();
    assert_eq!(
        session.show_source(&library, first_token).expect("shown"),
        ShowOutcome::Shown
    );

    let gate = Arc::new(MockCallGate::default());
    let provider = Arc::new(MockTranslationProvider::with_call_gate(gate.clone()));
    let job = translation::prepare(
        &library,
        first_token,
        tag("tr"),
        BlockLayoutConfig::default(),
        provider.identity(),
        metadata_seed("mid-flight"),
    )
    .expect("a valid job");
    let origin = job.origin().clone();

    let concrete_store = store(store_dir.path());
    let artifact_store: Arc<dyn ArtifactStore> = concrete_store.clone();
    let index: Arc<dyn ArtifactIndex> = concrete_store.clone();
    let handle = translation::start(
        job,
        provider.clone(),
        artifact_store,
        index,
        concrete_store,
        None,
    );

    // The provider call is now in flight — this is the moment a real
    // translation job would be running while the user keeps interacting.
    gate.wait_until_arrived();

    // §9: the user selects a different source while the job is running.
    assert_eq!(
        session.show_source(&library, second_token).expect("shown"),
        ShowOutcome::Shown
    );

    gate.release();
    let outcome = handle.join().expect("the job completes");

    // The job stayed bound to the source it started with, not the one the
    // user switched to mid-flight.
    assert_eq!(
        outcome.record.source_fingerprint,
        *first_fingerprint.as_bytes()
    );
    assert_ne!(
        outcome.record.source_fingerprint,
        *second_fingerprint.as_bytes()
    );

    // Cataloguing the result does not change what is on screen: nothing
    // calls `show_source` with the new AI token.
    library.add_translation(&origin, &outcome);
    let shown = session
        .expected_subtitle_text(1_000)
        .expect("the currently shown source has a cue at this moment");
    assert!(
        shown.contains("second-source"),
        "the second source must still be on screen: {shown}"
    );
    assert!(
        !shown.contains("[tr]"),
        "the AI output must never be forced onto screen: {shown}"
    );
}

#[test]
fn cancelling_a_running_job_leaves_no_artifact_and_no_catalog_entry() {
    let media = TempDir::new("cancel");
    let store_dir = TempDir::new("cancel-store");
    let mut library = SubtitleLibrary::new();
    let token = add(
        &mut library,
        media.path(),
        "Movie.en.srt",
        &srt(6, "cancel-me"),
    );

    let gate = Arc::new(MockCallGate::default());
    let provider = Arc::new(MockTranslationProvider::with_call_gate(gate.clone()));
    let job = translation::prepare(
        &library,
        token,
        tag("tr"),
        BlockLayoutConfig::default(),
        provider.identity(),
        metadata_seed("cancel"),
    )
    .expect("a valid job");

    let concrete_store = store(store_dir.path());
    let artifact_store: Arc<dyn ArtifactStore> = concrete_store.clone();
    let index: Arc<dyn ArtifactIndex> = concrete_store.clone();
    let handle = translation::start(
        job,
        provider.clone(),
        artifact_store,
        index.clone(),
        concrete_store,
        None,
    );

    gate.wait_until_arrived();
    handle.cancel();
    gate.release();

    let result = handle.join();
    assert!(
        matches!(result, Err(TranslationError::Cancelled)),
        "expected Cancelled, got {result:?}"
    );
    assert!(
        index.entries().expect("the index scan succeeds").is_empty(),
        "a cancelled job must not persist anything"
    );
    assert_eq!(
        library.catalog().of_kind(SubtitleSourceKind::Ai).count(),
        0,
        "a cancelled job has no outcome to catalogue"
    );
}
