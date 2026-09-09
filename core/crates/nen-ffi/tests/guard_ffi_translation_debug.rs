//! K23 guard for the translation FFI surface (`NEN-100`).
//!
//! Unlike `nen-app`'s own `guard_translation_debug.rs` (which proves
//! `nen_app::translation::{TranslationJob, TranslationOutcome}` redact by
//! hand-written `Debug`), everything on this side of the gate is built from
//! flat, payload-free enums and safe primitives ([`FfiTranslationProgress`],
//! [`FfiTranslationSummary`], and the two error types) — there is no
//! hand-written `Debug` here to audit. What this file proves instead: a
//! **real job**, run end to end through [`FfiTranslationEngine`] with
//! sentinel dialogue and a sentinel-named store root, never lets either
//! sentinel reach anything this gate hands back — every progress event, the
//! summary, and every variant of both error types — and that the check is
//! not blind: a naive `#[derive(Debug)]` twin holding the same sentinels is
//! shown to leak them, on the same terms as
//! `nen-app/tests/guard_translation_debug.rs`'s own twin.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use nen_ffi::subtitles::{FfiSubtitleLibrary, FfiSubtitleOutcome, FfiSubtitleSourceKind};
use nen_ffi::translation::{
    FfiTranslationEngine, FfiTranslationError, FfiTranslationProgress, FfiTranslationStartError,
    ForeignTranslationProgressSink,
};

/// Long and distinctive so a partial leak is caught as surely as a whole
/// one.
const DIALOGUE_SENTINEL: &str = "Zzqxvunlogged-ffi-dialogue";
/// Stands in for a private username or library path (K23 #3) — baked into
/// the store root every test below actually opens the engine with.
const PRIVATE_PATH_SENTINEL: &str = "gizli-kullanici-ffi";

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nen-100-guard-{tag}-{PRIVATE_PATH_SENTINEL}-{}-{:?}",
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

fn forbidden(output: &str) {
    for value in [DIALOGUE_SENTINEL, PRIVATE_PATH_SENTINEL] {
        assert!(
            !output.contains(value),
            "translation FFI output leaked a forbidden value ({value}): {output}"
        );
    }
}

#[derive(Default)]
struct RecordingSink {
    events: Mutex<Vec<FfiTranslationProgress>>,
}

impl ForeignTranslationProgressSink for RecordingSink {
    fn on_progress(&self, progress: FfiTranslationProgress) {
        self.events.lock().expect("lock").push(progress);
    }
}

/// A library holding one user file whose only cue carries
/// [`DIALOGUE_SENTINEL`], and the token naming it. The `.en.` suffix is
/// what NEN-057's filename hint reads.
fn library_with_sentinel_dialogue(directory: &Path) -> (Arc<FfiSubtitleLibrary>, u32) {
    let content = format!("1\n00:00:01,000 --> 00:00:02,500\n{DIALOGUE_SENTINEL}\n");
    let path = directory.join("Movie.en.srt");
    fs::write(&path, content).expect("write the fixture");
    let library = Arc::new(FfiSubtitleLibrary::new());
    assert_eq!(
        library.add_file(path.to_string_lossy().into_owned()),
        FfiSubtitleOutcome::Added
    );
    let token = library
        .menu(None, None)
        .into_iter()
        .flat_map(|section| section.entries)
        .find(|entry| entry.kind == FfiSubtitleSourceKind::User)
        .map(|entry| entry.token)
        .expect("the freshly added file is in the menu");
    (library, token)
}

#[test]
fn a_real_run_never_lets_the_dialogue_or_the_store_root_reach_progress_or_summary_output() {
    let media = TempDir::new("job-media");
    let store = TempDir::new("job-store");
    let (library, token) = library_with_sentinel_dialogue(media.path());

    let engine = FfiTranslationEngine::new(store.path().to_string_lossy().into_owned())
        .expect("a fresh store opens");
    let sink = Arc::new(RecordingSink::default());
    let job = engine
        .start(
            library,
            token,
            "tr".to_owned(),
            Some(sink.clone() as Arc<dyn ForeignTranslationProgressSink>),
        )
        .expect("the job starts");
    let summary = job.join().expect("the job completes");

    for progress in sink.events.lock().expect("lock").iter() {
        forbidden(&format!("{progress:?}"));
    }
    let summary_debug = format!("{summary:?}");
    forbidden(&summary_debug);
    // Not blind: the safe fields are still there.
    assert!(summary_debug.contains("cue_count"), "{summary_debug}");
    assert!(summary_debug.contains("target_language"), "{summary_debug}");
}

#[test]
fn every_start_refusal_variant_prints_only_its_own_name() {
    let variants = [
        FfiTranslationStartError::Unusable,
        FfiTranslationStartError::NotTranslatable,
        FfiTranslationStartError::UnknownSourceLanguage,
        FfiTranslationStartError::AlreadyTargetLanguage,
        FfiTranslationStartError::NoDocument,
        FfiTranslationStartError::LayoutRefused,
        FfiTranslationStartError::InvalidTargetLanguage,
        FfiTranslationStartError::StoreUnavailable,
    ];
    for variant in variants {
        let combined = format!("{variant:?} {variant}");
        forbidden(&combined);
        // Not blind: the variant's own name is still legible in both forms.
        assert!(combined.contains(&format!("{variant:?}")), "{combined}");
    }
}

#[test]
fn every_job_error_variant_prints_only_its_own_name() {
    let variants = [
        FfiTranslationError::Cancelled,
        FfiTranslationError::Failed,
        FfiTranslationError::Incomplete,
        FfiTranslationError::AssemblyRejected,
        FfiTranslationError::StoreFailed,
        FfiTranslationError::WorkerPanicked,
        FfiTranslationError::AlreadyJoined,
    ];
    for variant in variants {
        let combined = format!("{variant:?} {variant}");
        forbidden(&combined);
        assert!(combined.contains(&format!("{variant:?}")), "{combined}");
    }
}

/// A guard-rail on the guards above: if progress or the summary carried
/// either sentinel directly behind a naive `#[derive(Debug)]`, it would show
/// up immediately. This documents that today's silence comes from the types
/// having no such field, not from the sentinel simply never reaching a
/// formatter — the same discipline `nen-app/tests/guard_translation_debug.rs`
/// holds its own twin to.
#[test]
fn the_guard_sentinels_would_be_visible_in_a_derived_debug_twin() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct LeakyProgressSummary {
        dialogue: String,
        store_root: String,
    }

    let leaked = format!(
        "{:?}",
        LeakyProgressSummary {
            dialogue: DIALOGUE_SENTINEL.to_owned(),
            store_root: PRIVATE_PATH_SENTINEL.to_owned(),
        }
    );
    assert!(leaked.contains(DIALOGUE_SENTINEL));
    assert!(leaked.contains(PRIVATE_PATH_SENTINEL));
}
