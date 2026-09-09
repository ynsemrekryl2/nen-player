//! The translation FFI surface, driven entirely through the gate (`NEN-100`).
//!
//! Every test here goes through [`FfiTranslationEngine`]/[`FfiTranslationJob`]
//! only — never `nen_app::translation` directly — because that is the whole
//! point of this crate (ADR-0006 kural 2): a caller across the gate has no
//! other way in.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use nen_ffi::subtitles::{FfiSubtitleLibrary, FfiSubtitleOutcome, FfiSubtitleSourceKind};
use nen_ffi::translation::{
    FfiTranslationEngine, FfiTranslationError, FfiTranslationPhase, FfiTranslationProgress,
    FfiTranslationStartError, ForeignTranslationProgressSink,
};

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nen-100-gate-{tag}-{}-{:?}",
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

/// Writes `lines.len()` non-overlapping cues, one per line, starting one
/// second apart. Real timestamps and IDs matter to nothing here except
/// being valid — the strict SRT parser is not this file's concern.
fn write_srt(path: &Path, lines: &[&str]) {
    let mut content = String::new();
    for (index, line) in lines.iter().enumerate() {
        let start = 1 + index * 3;
        let end = start + 2;
        content.push_str(&format!(
            "{}\n00:00:{start:02},000 --> 00:00:{end:02},000\n{line}\n\n",
            index + 1
        ));
    }
    fs::write(path, content).expect("write the fixture");
}

/// Adds a fresh user file to `library` and returns the token naming it.
///
/// The `.en.` suffix is what NEN-057's filename hint reads — the content
/// itself need not look like any real language, and none of these fixtures'
/// dialogue does.
fn add_file(library: &FfiSubtitleLibrary, directory: &Path, name: &str, lines: &[&str]) -> u32 {
    let path = directory.join(name);
    write_srt(&path, lines);
    assert_eq!(
        library.add_file(path.to_string_lossy().into_owned()),
        FfiSubtitleOutcome::Added
    );
    library
        .menu(None, None)
        .into_iter()
        .flat_map(|section| section.entries)
        .find(|entry| entry.kind == FfiSubtitleSourceKind::User)
        .map(|entry| entry.token)
        .expect("the freshly added file is in the menu")
}

fn artifact_count(store_root: &Path) -> usize {
    fs::read_dir(store_root.join("artifacts"))
        .map(|entries| entries.count())
        .unwrap_or(0)
}

fn ai_entries(library: &FfiSubtitleLibrary) -> usize {
    library
        .menu(None, None)
        .into_iter()
        .flat_map(|section| section.entries)
        .filter(|entry| entry.kind == FfiSubtitleSourceKind::Ai)
        .count()
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

/// A two-party handshake, on the same terms as
/// `nen_providers::translation_mock::MockCallGate`: one side arrives and
/// waits, the other releases it once it has observed the arrival.
#[derive(Default)]
struct Rendezvous {
    state: Mutex<RendezvousState>,
    changed: Condvar,
}

#[derive(Default)]
struct RendezvousState {
    arrived: bool,
    released: bool,
}

impl Rendezvous {
    fn wait_until_arrived(&self) {
        let mut state = self.state.lock().expect("lock");
        while !state.arrived {
            state = self.changed.wait(state).expect("wait");
        }
    }

    fn release(&self) {
        let mut state = self.state.lock().expect("lock");
        state.released = true;
        self.changed.notify_all();
    }

    fn arrive_and_wait(&self) {
        let mut state = self.state.lock().expect("lock");
        state.arrived = true;
        self.changed.notify_all();
        while !state.released {
            state = self.changed.wait(state).expect("wait");
        }
    }
}

/// Records every event, and pauses **inside the delivery gate** on the
/// first one — `TranslationCall::progress` holds its gate lock for the
/// duration of the callback, so while this is paused `cancel()` is
/// guaranteed to block on the same lock (ADR-0004 Karar 2), and nothing else
/// on the job's own worker thread can race ahead of it.
struct PausingSink {
    events: Mutex<Vec<FfiTranslationProgress>>,
    rendezvous: Arc<Rendezvous>,
    paused: AtomicBool,
}

impl PausingSink {
    fn new(rendezvous: Arc<Rendezvous>) -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            rendezvous,
            paused: AtomicBool::new(false),
        }
    }
}

impl ForeignTranslationProgressSink for PausingSink {
    fn on_progress(&self, progress: FfiTranslationProgress) {
        self.events.lock().expect("lock").push(progress);
        if !self.paused.swap(true, Ordering::SeqCst) {
            self.rendezvous.arrive_and_wait();
        }
    }
}

#[test]
fn progress_events_arrive_in_order_and_the_summary_matches_the_source() {
    let media = TempDir::new("progress-media");
    let store = TempDir::new("progress-store");
    let library = Arc::new(FfiSubtitleLibrary::new());
    let token = add_file(
        &library,
        media.path(),
        "Movie.en.srt",
        &["Alpha line", "Bravo line", "Charlie line"],
    );

    let engine = FfiTranslationEngine::new(store.path().to_string_lossy().into_owned())
        .expect("a fresh store opens");
    let sink = Arc::new(RecordingSink::default());
    let job = engine
        .start(
            library.clone(),
            token,
            "tr".to_owned(),
            Some(sink.clone() as Arc<dyn ForeignTranslationProgressSink>),
        )
        .expect("the job starts");

    let summary = job.join().expect("the job completes");
    assert!(!summary.from_cache);
    assert_eq!(summary.cue_count, 3);
    assert_eq!(summary.target_language, "tr");

    let events = sink.events.lock().expect("lock").clone();
    let expected = [
        FfiTranslationProgress {
            phase: FfiTranslationPhase::Preparing,
            done: 0,
            total: 3,
        },
        FfiTranslationProgress {
            phase: FfiTranslationPhase::Translating,
            done: 1,
            total: 3,
        },
        FfiTranslationProgress {
            phase: FfiTranslationPhase::Translating,
            done: 2,
            total: 3,
        },
        FfiTranslationProgress {
            phase: FfiTranslationPhase::Translating,
            done: 3,
            total: 3,
        },
        FfiTranslationProgress {
            phase: FfiTranslationPhase::Finalizing,
            done: 3,
            total: 3,
        },
    ];
    assert_eq!(events, expected);

    let token = job
        .catalog_into(library.clone())
        .expect("a finished job catalogs its result");
    assert!(library.is_usable(token));
    assert_eq!(ai_entries(&library), 1);
}

#[test]
fn a_second_run_against_the_same_store_root_is_served_from_cache() {
    let media = TempDir::new("cache-media");
    let store = TempDir::new("cache-store");
    let library = Arc::new(FfiSubtitleLibrary::new());
    let token = add_file(&library, media.path(), "Movie.en.srt", &["Only line"]);

    let first_engine = FfiTranslationEngine::new(store.path().to_string_lossy().into_owned())
        .expect("a fresh store opens");
    let first = first_engine
        .start(library.clone(), token, "tr".to_owned(), None)
        .expect("the job starts")
        .join()
        .expect("the job completes");
    assert!(!first.from_cache);
    assert_eq!(artifact_count(store.path()), 1);

    // A fresh engine over the *same root* — as if the app had restarted —
    // proves the store root itself is what is honored, not the provider or
    // index this process happened to keep alive.
    let second_engine = FfiTranslationEngine::new(store.path().to_string_lossy().into_owned())
        .expect("the same store reopens");
    let second = second_engine
        .start(library.clone(), token, "tr".to_owned(), None)
        .expect("the job starts")
        .join()
        .expect("the job completes");
    assert!(second.from_cache);
    assert_eq!(
        artifact_count(store.path()),
        1,
        "no second artifact written"
    );
}

#[test]
fn catalog_into_answers_none_while_running_and_some_once_joined() {
    let media = TempDir::new("catalog-media");
    let store = TempDir::new("catalog-store");
    let library = Arc::new(FfiSubtitleLibrary::new());
    let token = add_file(&library, media.path(), "Movie.en.srt", &["Only line"]);

    let engine = FfiTranslationEngine::new(store.path().to_string_lossy().into_owned())
        .expect("a fresh store opens");
    let rendezvous = Arc::new(Rendezvous::default());
    let sink = Arc::new(PausingSink::new(rendezvous.clone()));
    let job = engine
        .start(
            library.clone(),
            token,
            "tr".to_owned(),
            Some(sink as Arc<dyn ForeignTranslationProgressSink>),
        )
        .expect("the job starts");

    rendezvous.wait_until_arrived();
    assert!(!job.is_finished());
    assert_eq!(
        job.catalog_into(library.clone()),
        None,
        "a running job has nothing to catalog yet"
    );
    rendezvous.release();

    job.join().expect("the job completes");
    let token = job
        .catalog_into(library.clone())
        .expect("a finished job catalogs its result");
    assert!(library.is_usable(token));
}

#[test]
fn cancelling_a_paused_job_delivers_no_further_progress_and_leaves_no_artifact_or_catalog_entry() {
    let media = TempDir::new("cancel-media");
    let store = TempDir::new("cancel-store");
    let library = Arc::new(FfiSubtitleLibrary::new());
    let token = add_file(
        &library,
        media.path(),
        "Movie.en.srt",
        &["One", "Two", "Three", "Four", "Five"],
    );

    let engine = FfiTranslationEngine::new(store.path().to_string_lossy().into_owned())
        .expect("a fresh store opens");
    let rendezvous = Arc::new(Rendezvous::default());
    let sink = Arc::new(PausingSink::new(rendezvous.clone()));
    let job = engine
        .start(
            library.clone(),
            token,
            "tr".to_owned(),
            Some(sink.clone() as Arc<dyn ForeignTranslationProgressSink>),
        )
        .expect("the job starts");

    // The very first progress callback (`Preparing`) is where `sink` pauses,
    // holding the delivery gate's own lock (ADR-0004 Karar 2) — so `cancel()`
    // on another thread is guaranteed to block on the same lock until
    // `release()` lets the callback return.
    rendezvous.wait_until_arrived();
    let canceller = {
        let job = job.clone();
        thread::spawn(move || job.cancel())
    };
    // Give the canceller a chance to reach (and block on) the delivery
    // gate's lock before the paused callback is allowed to return — the
    // same precaution `nen-ports`'s own
    // `cancel_waits_for_an_in_flight_commit_and_blocks_every_commit_after`
    // takes against the identical race (a freshly unblocked worker thread
    // can otherwise barge back onto the lock before a woken waiter is
    // scheduled). What the assertions below prove does not depend on this
    // sleep for correctness — only for reliably exercising the case where
    // cancellation actually lands before the run finishes, rather than
    // racing to the end first.
    thread::sleep(std::time::Duration::from_millis(50));
    rendezvous.release();
    canceller
        .join()
        .expect("the canceller thread does not panic");

    // From here on, nothing new can reach the sink: once `cancel()` has
    // returned, the gate is permanently closed, and every later
    // `checkpoint`/`progress` call sees that under the same lock before it
    // could ever reach the sink again (`nen_ports::translation::
    // TranslationCall::progress`'s own early return). This is the property
    // "iptal sonrası late commit yok" actually rests on, independent of
    // exactly how many events raced through *before* `cancel()` took hold.
    let count_after_cancel_returns = sink.events.lock().expect("lock").len();

    let result = job.join();
    assert_eq!(result, Err(FfiTranslationError::Cancelled));

    let count_after_worker_finished = sink.events.lock().expect("lock").len();
    assert_eq!(
        count_after_worker_finished, count_after_cancel_returns,
        "a progress callback arrived after cancel() had already returned"
    );

    assert_eq!(
        artifact_count(store.path()),
        0,
        "a cancelled job wrote no artifact"
    );
    assert_eq!(job.catalog_into(library.clone()), None);
    assert_eq!(
        ai_entries(&library),
        0,
        "a cancelled job added no catalog row"
    );
}

#[test]
fn starting_is_refused_when_the_source_is_already_the_target_language() {
    let media = TempDir::new("already-target-media");
    let store = TempDir::new("already-target-store");
    let library = Arc::new(FfiSubtitleLibrary::new());
    let token = add_file(&library, media.path(), "Movie.en.srt", &["Only line"]);

    let engine = FfiTranslationEngine::new(store.path().to_string_lossy().into_owned())
        .expect("a fresh store opens");
    let refusal = match engine.start(library, token, "en".to_owned(), None) {
        Err(refusal) => refusal,
        Ok(_) => panic!("the source is already English"),
    };
    assert_eq!(refusal, FfiTranslationStartError::AlreadyTargetLanguage);
    assert_eq!(artifact_count(store.path()), 0);
}

#[test]
fn starting_is_refused_for_an_unknown_token() {
    let store = TempDir::new("unknown-token-store");
    let library = Arc::new(FfiSubtitleLibrary::new());

    let engine = FfiTranslationEngine::new(store.path().to_string_lossy().into_owned())
        .expect("a fresh store opens");
    let refusal = match engine.start(library, 9_999, "tr".to_owned(), None) {
        Err(refusal) => refusal,
        Ok(_) => panic!("no such token was ever catalogued"),
    };
    assert_eq!(refusal, FfiTranslationStartError::Unusable);
    assert_eq!(artifact_count(store.path()), 0);
}

#[test]
fn starting_is_refused_for_a_target_language_the_core_cannot_parse() {
    let media = TempDir::new("bad-language-media");
    let store = TempDir::new("bad-language-store");
    let library = Arc::new(FfiSubtitleLibrary::new());
    let token = add_file(&library, media.path(), "Movie.en.srt", &["Only line"]);

    let engine = FfiTranslationEngine::new(store.path().to_string_lossy().into_owned())
        .expect("a fresh store opens");
    let refusal = match engine.start(library, token, "not a tag".to_owned(), None) {
        Err(refusal) => refusal,
        Ok(_) => panic!("not a BCP-47 tag"),
    };
    assert_eq!(refusal, FfiTranslationStartError::InvalidTargetLanguage);
    assert_eq!(artifact_count(store.path()), 0);
}

#[test]
fn joining_a_job_twice_answers_already_joined_the_second_time() {
    let media = TempDir::new("double-join-media");
    let store = TempDir::new("double-join-store");
    let library = Arc::new(FfiSubtitleLibrary::new());
    let token = add_file(&library, media.path(), "Movie.en.srt", &["Only line"]);

    let engine = FfiTranslationEngine::new(store.path().to_string_lossy().into_owned())
        .expect("a fresh store opens");
    let job = engine
        .start(library, token, "tr".to_owned(), None)
        .expect("the job starts");

    job.join().expect("the first join completes");
    assert_eq!(job.join(), Err(FfiTranslationError::AlreadyJoined));
}

#[test]
fn opening_an_engine_at_a_nonexistent_root_is_refused() {
    let missing = std::env::temp_dir().join("nen-100-gate-definitely-does-not-exist");
    let _ = fs::remove_dir_all(&missing);
    let refusal = match FfiTranslationEngine::new(missing.to_string_lossy().into_owned()) {
        Err(refusal) => refusal,
        Ok(_) => panic!("the root does not exist"),
    };
    assert_eq!(refusal, FfiTranslationStartError::StoreUnavailable);
}
