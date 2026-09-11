//! The translation FFI surface, driven entirely through the gate (`NEN-100`).
//!
//! Every test here drives a job through [`FfiTranslationEngine`]/
//! [`FfiTranslationJob`] only — the same `start`/`cancel`/`join`/
//! `catalog_into` path a real caller across the gate uses. The one exception
//! is how the two cancellation tests below **build** the engine: they call
//! [`FfiTranslationEngine::with_environment`] to hand a job a
//! [`nen_providers::translation_mock::MockCallGate`]-backed provider instead
//! of going through [`FfiTranslationEngine::new`]. That constructor is
//! `#[doc(hidden)]` and outside the `#[uniffi::export]` surface — no
//! generated binding names it, so it changes nothing about what a real
//! caller across the gate can reach (ADR-0006 kural 2). See its doc comment,
//! and `NEN-108`, for why: pausing a worker thread *inside* a progress
//! callback (as the plain [`FfiTranslationEngine::new`] +
//! [`ForeignTranslationProgressSink`] path forces, since [`nen_app::ports::
//! translation::TranslationCall::progress`] holds its delivery-gate lock for
//! the callback's whole duration) makes the moment `cancel()` unblocks a
//! straight race between two threads for the same freed lock — the OS's
//! std `Mutex` is not FIFO, and a `thread::sleep` before releasing the pause
//! only narrows the window, never closes it (measured red on CI, run
//! `34569242736`, after passing every local run). Pausing the worker
//! *between* two provider calls — entirely outside that lock, the way
//! `nen-app`'s own `tests/translation_retarget.rs` already does — removes
//! the race instead of narrowing it: `cancel()` runs uncontended the moment
//! it is called, and the worker's very next `checkpoint()` sees it closed
//! deterministically, every time.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use nen_app::translation::TranslationEnvironment;
use nen_ffi::subtitles::{FfiSubtitleLibrary, FfiSubtitleOutcome, FfiSubtitleSourceKind};
use nen_ffi::translation::{
    FfiTranslationEngine, FfiTranslationError, FfiTranslationJob, FfiTranslationPhase,
    FfiTranslationProgress, FfiTranslationStartError, ForeignTranslationProgressSink,
};
use nen_providers::translation_mock::{MockCallGate, MockTranslationProvider};

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

/// Opens an engine over `store_root` whose provider pauses — via `gate` —
/// after delivering a progress event but before the next `checkpoint()`,
/// entirely outside `TranslationCall`'s own delivery-gate lock (module doc).
fn gated_engine(store_root: &Path) -> (Arc<MockCallGate>, FfiTranslationEngine) {
    let gate = Arc::new(MockCallGate::default());
    let provider = Arc::new(MockTranslationProvider::with_call_gate(gate.clone()));
    let inner =
        TranslationEnvironment::with_provider(store_root, provider).expect("a fresh store opens");
    (gate, FfiTranslationEngine::with_environment(inner))
}

/// Blocks until `job.is_finished()` is `true` — a deterministic marker of a
/// state transition, not a duration, so this never races the transition it
/// waits for the way a fixed `thread::sleep` would (module doc). The bound
/// is only a diagnostic backstop against a genuine deadlock, never expected
/// to fire.
fn wait_until_finished(job: &FfiTranslationJob) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !job.is_finished() {
        assert!(
            std::time::Instant::now() < deadline,
            "job did not reach a finished state within the deadline"
        );
        thread::yield_now();
    }
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

    let (gate, engine) = gated_engine(store.path());
    let sink = Arc::new(RecordingSink::default());
    let job = engine
        .start(
            library.clone(),
            token,
            "tr".to_owned(),
            Some(sink.clone() as Arc<dyn ForeignTranslationProgressSink>),
        )
        .expect("the job starts");

    // The worker has delivered `Preparing` (recorded by `sink`, under and
    // released by `TranslationCall`'s own delivery-gate lock, same as any
    // other progress event) and is now paused **inside the provider**,
    // between that call and its first `checkpoint()` — outside every lock
    // this job holds (module doc). `cancel()` below is therefore never
    // contending with the worker for anything: it runs to completion the
    // moment it is called.
    gate.wait_until_arrived();
    job.cancel();

    // From here on, nothing new can reach the sink: `cancel()` has already
    // returned, the gate is permanently closed, and every later
    // `checkpoint`/`progress` call sees that under the same lock before it
    // could ever reach the sink again (`nen_ports::translation::
    // TranslationCall::progress`'s own early return). This is the property
    // "iptal sonrası late commit yok" actually rests on.
    let count_after_cancel_returns = sink.events.lock().expect("lock").len();
    assert_eq!(
        count_after_cancel_returns, 1,
        "only the Preparing event should have been delivered before the pause"
    );

    gate.release();
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

/// `NEN-102`'s own regression: the shell always calls `join()` immediately
/// after `start()` (there is no other sane way to get the result off the
/// main actor), so a cancel button's `cancel()` call almost always lands
/// while `join()` is already in flight on a **different** thread than the
/// one that will eventually call `cancel()`. Measured red before
/// `TranslationCancelHandle` (`nen-app`) existed: `FfiTranslationJob::cancel`
/// read `JobState`, and `join()`'s own first action swapped that state to
/// `Taken` — permanently — the instant `join()` was called, regardless of
/// how long the run actually took afterward. `cancel()` was a silent no-op
/// for the job's entire remaining duration.
#[test]
fn cancelling_after_join_has_already_been_called_still_stops_the_job() {
    let media = TempDir::new("cancel-after-join-media");
    let store = TempDir::new("cancel-after-join-store");
    let library = Arc::new(FfiSubtitleLibrary::new());
    let token = add_file(
        &library,
        media.path(),
        "Movie.en.srt",
        &["One", "Two", "Three", "Four", "Five"],
    );

    let (gate, engine) = gated_engine(store.path());
    let sink = Arc::new(RecordingSink::default());
    let job = engine
        .start(
            library.clone(),
            token,
            "tr".to_owned(),
            Some(sink as Arc<dyn ForeignTranslationProgressSink>),
        )
        .expect("the job starts");

    gate.wait_until_arrived();

    // Unlike the test above: `join()` is called first, on its own thread —
    // the shell's own shape (`translateSelectedSubtitle()` calls `join()`
    // right after `start()`) — and only then does something else call
    // `cancel()`.
    let joiner = {
        let job = job.clone();
        thread::spawn(move || job.join())
    };
    // `join()`'s very first action swaps `JobState` to `Taken`, and
    // `is_finished()` reads `Taken` as finished regardless of whether the
    // worker itself has actually returned yet (its own doc comment) — so
    // this deterministically observes the moment `join()` is genuinely in
    // flight, the exact ordering this test's regression depended on,
    // without guessing at how long that takes.
    wait_until_finished(&job);

    // The worker is still paused outside `TranslationCall`'s lock (same
    // reasoning as the test above), so `cancel()` here is not racing
    // `join()` for anything and needs no thread of its own.
    job.cancel();
    gate.release();

    let result = joiner.join().expect("the joiner thread does not panic");
    assert_eq!(
        result,
        Err(FfiTranslationError::Cancelled),
        "cancel() called after join() was already in flight had no effect"
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
