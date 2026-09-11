//! A translation job, across the gate (`NEN-100`).
//!
//! `nen-app`'s [`TranslationEnvironment`] is this module's whole reason to
//! exist: it is the composition root that wires a concrete
//! `FilesystemArtifactStore` (`nen-persist`) to M5's deterministic mock
//! provider (`nen-providers`), which `nen-ffi` is not allowed to name in its
//! production `[dependencies]` (ADR-0006 kural 3 — this crate depends on
//! `nen-app` only). `[dev-dependencies]` is the one narrow exception
//! (`NEN-108`): `tests/translation_gate.rs` names `nen-providers` directly
//! through [`FfiTranslationEngine::with_environment`], a `#[doc(hidden)]`
//! constructor outside the `#[uniffi::export]` surface — no generated
//! binding, and so no real caller, can reach it. So this gate takes and
//! returns nothing more exotic than a store root path, a catalog token and a
//! language tag; a caller across it can start a job, get
//! told why not, watch its progress, cancel it, and — once it finishes — add
//! its result to the same [`FfiSubtitleLibrary`] it started from.
//!
//! # Direction of travel (ADR-0004, ADR-0033 Notlar)
//!
//! Progress travels **push**, through a foreign
//! [`ForeignTranslationProgressSink`] — unlike [`crate::playback`]'s events,
//! which `drain_events()` **pulls** (ADR-0033 Karar 3). That decision is
//! about a continuous ~30 Hz stream where an idle consumer would let a
//! dispatch queue back up; a translation job's progress is a handful of
//! discrete events per block, and [`nen_app::ports::translation::
//! TranslationCall`] (ADR-0004 Karar 2/5) already is the single delivery
//! gate a push needs: `cancel()` waits out a callback in flight and nothing
//! is delivered after it returns. This module adds no second gate.
//!
//! # Security (K23)
//!
//! Cue text (K23 #4) never reaches this file. [`FfiTranslationProgress`] and
//! [`FfiTranslationSummary`] are built only from closed enums, counts and a
//! language tag — the same kind of value [`crate::playback::
//! FfiTrackDescriptor`]'s `language` field already crosses unredacted. The
//! two error types are flat and payload-free, mirroring
//! [`crate::playback::FfiPlaybackError`]'s `Display` rule: it prints one of
//! this crate's own constants, never anything from the value it was built
//! from. Neither `FfiTranslationEngine` nor `FfiTranslationJob` derives
//! `Debug` (`crate::session`'s reasoning: the store root is a private path,
//! K23 #3, and nothing above needs to print either object as a whole).
//! `tests/guard_ffi_translation_debug.rs` measures a real run driven end to
//! end through this gate, with sentinel dialogue and a sentinel store root,
//! and shows a naive derive would have leaked it.

use crate::subtitles::FfiSubtitleLibrary;
use nen_app::domain::source::LanguageTag;
use nen_app::ports::translation::{TranslationProgress, TranslationProgressPhase};
use nen_app::translation::{
    StartRefusal, TranslationCancelHandle, TranslationEnvironment, TranslationError,
    TranslationJobHandle, TranslationOutcome,
};
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// One phase of a translation job's progress (mirrors
/// [`TranslationProgressPhase`]). Payload-free, so `Debug` is derived —
/// there is nothing for a derive to leak.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiTranslationPhase {
    Preparing,
    Translating,
    Finalizing,
}

impl From<TranslationProgressPhase> for FfiTranslationPhase {
    fn from(value: TranslationProgressPhase) -> Self {
        match value {
            TranslationProgressPhase::Preparing => Self::Preparing,
            TranslationProgressPhase::Translating => Self::Translating,
            TranslationProgressPhase::Finalizing => Self::Finalizing,
        }
    }
}

/// One progress report (mirrors [`TranslationProgress`]): a phase and two
/// monotonic counters (ADR-0004 Karar 4), nothing else. `Debug` is derived —
/// the same shape [`TranslationProgress`] itself already derives it for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct FfiTranslationProgress {
    pub phase: FfiTranslationPhase,
    pub done: u32,
    pub total: u32,
}

impl From<TranslationProgress> for FfiTranslationProgress {
    fn from(value: TranslationProgress) -> Self {
        Self {
            phase: value.phase.into(),
            done: value.done,
            total: value.total,
        }
    }
}

/// What a finished job leaves for the caller: whether it came from the
/// cache, how many cues it produced, and which language it is in.
///
/// **No content address, no fingerprint.** [`nen_ports::persistence::
/// ContentAddress`] and [`nen_ports::persistence::ArtifactRecord`] already
/// redact both in their own `Debug`/`Display` (`<redacted>`) — a guarantee
/// this crate's host language does not share, so neither crosses at all.
/// `Debug` is derived: every field here is a bool, a count, or a language
/// tag, none of them a K23 class.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiTranslationSummary {
    pub from_cache: bool,
    pub cue_count: u32,
    pub target_language: String,
}

impl From<&TranslationOutcome> for FfiTranslationSummary {
    fn from(value: &TranslationOutcome) -> Self {
        Self {
            from_cache: value.from_cache,
            cue_count: u32::try_from(value.record.document.len()).unwrap_or(u32::MAX),
            target_language: value.record.target_language.as_str().to_owned(),
        }
    }
}

/// Why a job never started.
///
/// Flat and payload-free, unlike [`StartRefusal`] itself
/// (`StartRefusal::Layout` carries a [`nen_translate::blocks::
/// BlockLayoutError`] with counts): the shell has no use for those counts
/// and a payload here is one more thing this crate would have to prove safe
/// to print. `Display` names the variant and nothing else — the rule
/// [`crate::playback::FfiPlaybackError`] already holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Error)]
pub enum FfiTranslationStartError {
    /// The token names no catalogued, usable source.
    Unusable,
    /// The row is a bitmap track — nothing to translate.
    NotTranslatable,
    /// The source has no established language.
    UnknownSourceLanguage,
    /// The source is already in the requested target language.
    AlreadyTargetLanguage,
    /// The row is catalogued but has no parsed document yet.
    NoDocument,
    /// The document could not be split into blocks under M5's fixed layout.
    LayoutRefused,
    /// `target_language` is not a BCP-47 tag this core can parse.
    InvalidTargetLanguage,
    /// The artifact store could not be opened at the given root.
    StoreUnavailable,
}

impl fmt::Display for FfiTranslationStartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unusable => "unusable",
            Self::NotTranslatable => "not_translatable",
            Self::UnknownSourceLanguage => "unknown_source_language",
            Self::AlreadyTargetLanguage => "already_target_language",
            Self::NoDocument => "no_document",
            Self::LayoutRefused => "layout_refused",
            Self::InvalidTargetLanguage => "invalid_target_language",
            Self::StoreUnavailable => "store_unavailable",
        })
    }
}

impl std::error::Error for FfiTranslationStartError {}

impl From<StartRefusal> for FfiTranslationStartError {
    fn from(value: StartRefusal) -> Self {
        match value {
            StartRefusal::Unusable => Self::Unusable,
            StartRefusal::NotTranslatable => Self::NotTranslatable,
            StartRefusal::UnknownSourceLanguage => Self::UnknownSourceLanguage,
            StartRefusal::AlreadyTargetLanguage => Self::AlreadyTargetLanguage,
            StartRefusal::NoDocument => Self::NoDocument,
            StartRefusal::Layout(_) => Self::LayoutRefused,
        }
    }
}

/// Why a started job did not produce an outcome.
///
/// Flat and payload-free, for the same reason as
/// [`FfiTranslationStartError`]: [`TranslationError`]'s own variants already
/// carry no cue text or private path (its own doc comment), but their inner
/// types (`BlockTranslationError`, `ArtifactError`, `ArtifactStoreError`)
/// are not this crate's to re-derive `Debug` on and hand to a host language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Error)]
pub enum FfiTranslationError {
    /// The delivery gate was closed before or during this run.
    Cancelled,
    /// A block exhausted its repair budget, or the provider failed.
    Failed,
    /// Every block reported checkpointed but assembly still refused —
    /// unreachable in practice, modelled rather than panicked.
    Incomplete,
    /// The checkpointed run's cues did not re-validate against the source.
    AssemblyRejected,
    /// The artifact store or its index refused a read or a write.
    StoreFailed,
    /// The worker thread panicked before it could produce a result.
    WorkerPanicked,
    /// [`FfiTranslationJob::join`] was already called once.
    AlreadyJoined,
}

impl fmt::Display for FfiTranslationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
            Self::Incomplete => "incomplete",
            Self::AssemblyRejected => "assembly_rejected",
            Self::StoreFailed => "store_failed",
            Self::WorkerPanicked => "worker_panicked",
            Self::AlreadyJoined => "already_joined",
        })
    }
}

impl std::error::Error for FfiTranslationError {}

impl From<TranslationError> for FfiTranslationError {
    fn from(value: TranslationError) -> Self {
        match value {
            TranslationError::Cancelled => Self::Cancelled,
            TranslationError::Failed(_) => Self::Failed,
            TranslationError::Incomplete => Self::Incomplete,
            TranslationError::Assembly(_) => Self::AssemblyRejected,
            TranslationError::Store(_) => Self::StoreFailed,
            TranslationError::Resume(_) => Self::StoreFailed,
            TranslationError::WorkerPanicked => Self::WorkerPanicked,
        }
    }
}

/// The shell's own progress sink, implemented on the platform side.
///
/// **Never call back into the [`FfiTranslationJob`] this progress belongs
/// to from inside this method.** [`nen_app::ports::translation::
/// TranslationCall::progress`] calls the sink while holding its delivery
/// gate's lock; reaching back in — `cancel()` included — would deadlock the
/// same non-reentrant lock [`nen_app::ports::translation::
/// TranslationCall::commit`]'s own doc already warns against.
#[uniffi::export(with_foreign)]
pub trait ForeignTranslationProgressSink: Send + Sync {
    fn on_progress(&self, progress: FfiTranslationProgress);
}

/// Dresses a foreign sink for [`TranslationEnvironment::start`], which knows
/// only the core's own [`nen_app::ports::translation::
/// TranslationProgressSink`].
struct ForeignProgressSinkAdapter {
    inner: Arc<dyn ForeignTranslationProgressSink>,
}

impl nen_app::ports::translation::TranslationProgressSink for ForeignProgressSinkAdapter {
    fn on_progress(&self, progress: TranslationProgress) {
        self.inner.on_progress(progress.into());
    }
}

/// Where a translation job is, from the caller's side of the gate.
enum JobState {
    Running(TranslationJobHandle),
    /// `join()` returned `Ok`. Boxed so this variant does not dominate the
    /// enum's size (`TranslationOutcome` carries a whole `ArtifactRecord`).
    Done(Box<FinishedJob>),
    /// `join()` returned `Err`.
    Failed,
    /// `join()` was already called once; nothing is kept to join again.
    Taken,
}

/// `join()`'s success case, kept until [`FfiTranslationJob::catalog_into`]
/// consumes it.
///
/// The origin travels alongside the outcome because
/// [`TranslationJobHandle::origin`] lived on the handle this state
/// replaced — [`nen_app::subtitles::SubtitleLibrary::add_translation`] needs
/// both.
struct FinishedJob {
    origin: nen_app::domain::source::SubtitleSourceId,
    outcome: TranslationOutcome,
}

/// A running or finished translation job.
///
/// No `Debug`: a running job holds a [`TranslationJobHandle`] and a
/// finished one holds a [`TranslationOutcome`], and neither is this type's
/// to print — the same reasoning [`crate::session::FfiPlaybackSession`]
/// gives for holding no `Debug` of its own.
#[derive(uniffi::Object)]
pub struct FfiTranslationJob {
    /// Independent of `state` (`NEN-102`) — deliberately so.
    /// [`FfiTranslationJob::join`] moves the running [`TranslationJobHandle`]
    /// out of `state` and into a blocking call on the first thread that
    /// joins; a caller cancelling from another thread while that call is in
    /// flight must still reach the job's delivery gate, and `state` alone
    /// cannot offer that once it reads `Taken`. `cancel_handle` is captured
    /// once, in [`FfiTranslationEngine::start`], before `state` is ever
    /// touched, and outlives every value `state` can hold.
    cancel_handle: TranslationCancelHandle,
    state: Mutex<JobState>,
}

/// Takes the lock without caring whether a previous holder panicked — the
/// same reasoning `crate::subtitles`'s own `lock` helper gives: the state
/// behind it is still true.
fn lock(mutex: &Mutex<JobState>) -> std::sync::MutexGuard<'_, JobState> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[uniffi::export]
impl FfiTranslationJob {
    /// Closes the job's delivery gate (ADR-0004 Karar 2). Idempotent and
    /// safe to call before the job has started doing any work, while
    /// [`Self::join`] is blocking on another thread, or after the job has
    /// already finished or been joined — `cancel_handle` does not depend on
    /// `state`, unlike a caller reaching into `JobState::Running` would
    /// (`NEN-102`; see `cancel_handle`'s own doc comment).
    pub fn cancel(&self) {
        self.cancel_handle.cancel();
    }

    pub fn is_finished(&self) -> bool {
        match &*lock(&self.state) {
            JobState::Running(handle) => handle.is_finished(),
            JobState::Done(_) | JobState::Failed | JobState::Taken => true,
        }
    }

    /// Blocks until the job's worker thread returns.
    ///
    /// ADR-0004 Karar 1 puts async ownership with the caller and opens no
    /// new runtime for it — a shell calls this from its own background
    /// task, the way `NEN-102` will. Answers `AlreadyJoined` on a second
    /// call rather than replaying a stale result: a [`TranslationJobHandle`]
    /// joins exactly once, and this wrapper does not pretend otherwise.
    pub fn join(&self) -> Result<FfiTranslationSummary, FfiTranslationError> {
        let handle = {
            let mut state = lock(&self.state);
            match std::mem::replace(&mut *state, JobState::Taken) {
                JobState::Running(handle) => handle,
                already_finished @ (JobState::Done(_) | JobState::Failed | JobState::Taken) => {
                    *state = already_finished;
                    return Err(FfiTranslationError::AlreadyJoined);
                }
            }
        };
        let origin = handle.origin().clone();
        match handle.join() {
            Ok(outcome) => {
                let summary = FfiTranslationSummary::from(&outcome);
                *lock(&self.state) = JobState::Done(Box::new(FinishedJob { origin, outcome }));
                Ok(summary)
            }
            Err(error) => {
                *lock(&self.state) = JobState::Failed;
                Err(error.into())
            }
        }
    }

    /// Adds a finished job's result to `library` as a new `Ai` catalog row
    /// and returns its token.
    ///
    /// `None` unless [`Self::join`] has already returned `Ok` — `NEN-099`'s
    /// deliberately separate, explicit step: nothing about starting or
    /// finishing a job reaches a menu on its own, only this call does, and
    /// only a caller that holds a finished job can make it.
    pub fn catalog_into(&self, library: Arc<FfiSubtitleLibrary>) -> Option<u32> {
        let state = lock(&self.state);
        let JobState::Done(finished) = &*state else {
            return None;
        };
        Some(
            library
                .with_mut(|library| library.add_translation(&finished.origin, &finished.outcome)),
        )
    }
}

/// The composition root a shell opens once per store root, and starts every
/// translation job through.
///
/// No `Debug`: the root it was opened with is a private path (K23 #3), and
/// nothing above needs to print this object as a whole (`crate::session`'s
/// reasoning again).
#[derive(uniffi::Object)]
pub struct FfiTranslationEngine {
    inner: TranslationEnvironment,
}

#[uniffi::export]
impl FfiTranslationEngine {
    /// Opens (creating it if needed) a content-addressed artifact store
    /// under `store_root`, paired with M5's deterministic mock provider
    /// (`nen_app::translation::TranslationEnvironment::new` — M6 replaces
    /// the provider only there, never here).
    #[uniffi::constructor]
    pub fn new(store_root: String) -> Result<Self, FfiTranslationStartError> {
        let inner = TranslationEnvironment::new(Path::new(&store_root))
            .map_err(|_store_error| FfiTranslationStartError::StoreUnavailable)?;
        Ok(Self { inner })
    }

    /// Builds and starts a job for `library`'s `token` in one call.
    ///
    /// `sink` is optional: a caller that only polls
    /// [`FfiTranslationJob::is_finished`] need not implement one.
    pub fn start(
        &self,
        library: Arc<FfiSubtitleLibrary>,
        token: u32,
        target_language: String,
        sink: Option<Arc<dyn ForeignTranslationProgressSink>>,
    ) -> Result<Arc<FfiTranslationJob>, FfiTranslationStartError> {
        let target_language = LanguageTag::parse(&target_language)
            .map_err(|_parse_error| FfiTranslationStartError::InvalidTargetLanguage)?;
        let progress_sink: Option<Arc<dyn nen_app::ports::translation::TranslationProgressSink>> =
            sink.map(|sink| {
                Arc::new(ForeignProgressSinkAdapter { inner: sink })
                    as Arc<dyn nen_app::ports::translation::TranslationProgressSink>
            });
        let handle = library.with(|library| {
            self.inner
                .start(library, token, target_language, progress_sink)
        })?;
        // Captured before `handle` is ever moved into `state` — the whole
        // point of `cancel_handle` (`NEN-102`) is to outlive whatever
        // `join()` later does to `state`.
        let cancel_handle = handle.cancel_handle();
        Ok(Arc::new(FfiTranslationJob {
            cancel_handle,
            state: Mutex::new(JobState::Running(handle)),
        }))
    }
}

impl FfiTranslationEngine {
    /// Wraps an already-composed [`TranslationEnvironment`] — never part of
    /// the `#[uniffi::export]` surface above, so no generated binding names
    /// it and a real caller across the gate has no way to reach it; the
    /// only constructor `nen_ffi.swift`/Kotlin ever sees is [`Self::new`].
    ///
    /// This exists solely so `tests/translation_gate.rs` (`NEN-108`) can
    /// hand a job a [`nen_providers::translation_mock::
    /// MockTranslationProvider`] built with a
    /// [`nen_providers::translation_mock::MockCallGate`], which pauses the
    /// worker thread **between** provider calls rather than inside one, on
    /// the far side of [`nen_app::ports::translation::TranslationCall`]'s
    /// own delivery-gate lock. Everything else about a job started from the
    /// result — `start`, `cancel`, `join`, `catalog_into` — is the exact
    /// same exported path a real caller drives.
    #[doc(hidden)]
    pub fn with_environment(inner: TranslationEnvironment) -> Self {
        Self { inner }
    }
}
