//! K23 guard for `nen_app::translation` (`NEN-099`).
//!
//! [`TranslationJob`] carries a full translated-from [`SubtitleDocument`]
//! (dialogue, K23 #4) and an [`ArtifactMetadata`] that may in principle
//! carry a glossary name (K23 #8); [`TranslationOutcome`] carries a stored
//! [`ArtifactRecord`]. This proves neither type's `Debug` leaks a sentinel
//! placed in the dialogue or the glossary name, and that the guard is not
//! blind: a deliberately naive `#[derive(Debug)]` twin is shown to leak the
//! same sentinel, so the redaction demonstrably comes from the hand-written
//! `Debug` impls — mirroring `nen-translate`'s own
//! `guard_cache_identity_debug.rs`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use nen_app::subtitles::{AddOutcome, SubtitleLibrary};
use nen_app::translation::{self, TranslationMetadataSeed};
use nen_domain::source::{LanguageTag, SubtitleSourceKind};
use nen_persist::FilesystemArtifactStore;
use nen_ports::identity::MediaHash;
use nen_ports::persistence::{ArtifactIndex, ArtifactStore};
use nen_ports::translation::TranslationProvider;
use nen_providers::translation_mock::MockTranslationProvider;
use nen_translate::artifact::{ArtifactId, ArtifactTimestamp, GlossaryIdentity};
use nen_translate::blocks::BlockLayoutConfig;

/// Long and distinctive so a partial leak is caught as surely as a whole
/// one.
const DIALOGUE_SENTINEL: &str = "Zzqxvunlogged-dialogue";
const GLOSSARY_SENTINEL: &str = "Zzqxvunlogged-glossary";

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nen-099-guard-{tag}-{}-{:?}",
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

fn tag(value: &str) -> LanguageTag {
    LanguageTag::parse(value).expect("a valid language tag")
}

/// A library holding one user file whose only cue carries [`DIALOGUE_SENTINEL`],
/// and the token naming it.
fn library_with_sentinel_dialogue(directory: &Path) -> (SubtitleLibrary, u32) {
    let contents = format!("1\n00:00:01,000 --> 00:00:02,500\n{DIALOGUE_SENTINEL}\n");
    let path = directory.join("Movie.en.srt");
    fs::write(&path, contents).expect("write the fixture");
    let mut library = SubtitleLibrary::new();
    assert_eq!(library.add_file(&path, directory), AddOutcome::Added);
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

fn sentinel_job(
    library: &SubtitleLibrary,
    token: u32,
    provider: &MockTranslationProvider,
) -> translation::TranslationJob {
    translation::prepare(
        library,
        token,
        tag("tr"),
        BlockLayoutConfig::default(),
        provider.identity(),
        TranslationMetadataSeed {
            id: ArtifactId::parse("guard-sentinel").expect("a valid artifact id"),
            glossary: GlossaryIdentity::named(GLOSSARY_SENTINEL),
            media_hash: Some(MediaHash::from_bytes([1, 2, 3, 4, 5, 6, 7, 8])),
            created_at: ArtifactTimestamp::from_unix_ms(1_700_000_000_000),
        },
    )
    .expect("a valid job")
}

#[test]
fn a_translation_job_debug_output_leaks_neither_the_dialogue_nor_the_glossary_name() {
    let media = TempDir::new("job");
    let (library, token) = library_with_sentinel_dialogue(media.path());
    let provider = MockTranslationProvider::new();
    let job = sentinel_job(&library, token, &provider);

    let debug = format!("{job:?}");
    assert!(
        !debug.contains(DIALOGUE_SENTINEL),
        "TranslationJob::Debug leaked the source dialogue — {debug}"
    );
    assert!(
        !debug.contains(GLOSSARY_SENTINEL),
        "TranslationJob::Debug leaked the glossary name — {debug}"
    );
    // Not blind to shape: the safe summary fields are still there.
    assert!(debug.contains("cue_count"), "{debug}");
    assert!(debug.contains("has_glossary"), "{debug}");
}

#[test]
fn a_translation_outcome_debug_output_leaks_neither_the_translated_dialogue_nor_its_address() {
    let media = TempDir::new("outcome");
    let store_dir = TempDir::new("outcome-store");
    let (library, token) = library_with_sentinel_dialogue(media.path());
    let provider = Arc::new(MockTranslationProvider::new());
    let job = sentinel_job(&library, token, &provider);

    let concrete_store = Arc::new(FilesystemArtifactStore::new(store_dir.path()).expect("a store"));
    let artifact_store: Arc<dyn ArtifactStore> = concrete_store.clone();
    let index: Arc<dyn ArtifactIndex> = concrete_store.clone();
    let outcome = translation::start(job, provider.clone(), artifact_store, index, None)
        .join()
        .expect("the job completes");

    let debug = format!("{outcome:?}");
    assert!(
        !debug.contains(DIALOGUE_SENTINEL),
        "TranslationOutcome::Debug leaked the source dialogue — {debug}"
    );
    // The mock's translated text embeds the source line, so this also rules
    // out the translated (not just the source) dialogue.
    assert!(
        !debug.contains(&format!("[tr] {DIALOGUE_SENTINEL}")),
        "TranslationOutcome::Debug leaked the translated dialogue — {debug}"
    );
    assert!(
        !debug.contains(&outcome.address.to_hex()),
        "TranslationOutcome::Debug leaked its own content address — {debug}"
    );
}

/// A guard-rail on the guards above: if either type were changed to a naive
/// `#[derive(Debug)]`, the sentinel would show up immediately. This
/// documents that the redaction comes from the hand-written `Debug` impls,
/// not from the sentinel simply never reaching a formatter.
#[test]
fn the_guard_sentinels_would_be_visible_in_derived_debug_twins() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct LeakyTranslationJob {
        dialogue: String,
        glossary_name: String,
    }

    let leaked = format!(
        "{:?}",
        LeakyTranslationJob {
            dialogue: DIALOGUE_SENTINEL.to_owned(),
            glossary_name: GLOSSARY_SENTINEL.to_owned(),
        }
    );
    assert!(leaked.contains(DIALOGUE_SENTINEL));
    assert!(leaked.contains(GLOSSARY_SENTINEL));
}
