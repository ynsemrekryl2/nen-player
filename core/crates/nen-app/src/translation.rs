//! Translation session orchestration (`NEN-099`, `docs/product-spec.md` §9).
//!
//! Selecting a subtitle source never starts a translation — the two things
//! this module ties together, [`SubtitleLibrary`] and `nen-translate`'s
//! pipeline, only meet once a caller explicitly calls [`start`] with a
//! [`TranslationJob`] built by [`prepare`]. Everything a running job needs
//! (the source document, the language pair, the block layout, the artifact
//! metadata, and the cache key computed from all of them) is captured once,
//! in `prepare`, into an immutable [`TranslationJob`]. There is no API that
//! changes a job after that: §9's "iş başlangıç source fingerprint'ine bağlı
//! kalır, retarget edilmez" is not a rule this module enforces at runtime,
//! it is a state that does not exist to violate — a running job simply has
//! no path back to [`SubtitleLibrary`] to ask what is selected now.
//!
//! [`start`] spawns the job's own worker thread and returns a
//! [`TranslationJobHandle`] immediately (ADR-0004 Karar 1: the session runs
//! the call on its own worker, not the caller's). The worker first checks
//! the job's cache identity against the [`ArtifactIndex`] — a hit returns
//! the stored [`nen_ports::persistence::ArtifactRecord`] without ever
//! calling the provider (§11) — and only on a miss drives
//! [`nen_translate::checkpoint::translate_resumable`], atomically persists
//! every validated block, and persists the final result. Cancellation is
//! [`nen_ports::translation::TranslationCall`]'s own
//! gate (ADR-0004 Karar 2); this module adds no second one.
//!
//! **What this module deliberately does not do:** it never adds its result
//! to a [`SubtitleLibrary`]. That is [`SubtitleLibrary::add_translation`],
//! a separate call a caller makes only after `join()` returns `Ok` — which
//! is what makes §9's "kullanıcı başka source izliyorsa zorla AI çıktısına
//! geçilmez" structural rather than a check somewhere: nothing in this
//! module can reach [`PlaybackSession::show_source`](crate::session::PlaybackSession::show_source).

use crate::subtitles::SubtitleLibrary;
use nen_domain::source::{LanguageTag, SubtitleSourceId};
use nen_domain::subtitle::SubtitleDocument;
use nen_ports::identity::MediaHash;
use nen_ports::persistence::{
    ArtifactIndex, ArtifactRecord, ArtifactStore, ArtifactStoreError, CacheKey, ContentAddress,
    ResumeStore, ResumeStoreError,
};
use nen_ports::translation::{
    TranslationCall, TranslationProgressSink, TranslationProvider, TranslationProviderError,
    TranslationProviderIdentity,
};
use nen_subtitle::fingerprint::SourceFingerprint;
use nen_translate::artifact::{
    self, ArtifactError, ArtifactId, ArtifactMetadata, ArtifactTimestamp, GlossaryIdentity,
};
use nen_translate::blocks::{BlockLayout, BlockLayoutConfig, BlockLayoutError};
use nen_translate::checkpoint::{translate_resumable, ResumableTranslationError, TranslationPlan};
use nen_translate::context::DocumentContext;
use nen_translate::identity::{CacheIdentity, CacheIdentityInput};
use nen_translate::repair::BlockTranslationError;
use std::fmt;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{SystemTime, UNIX_EPOCH};

/// What a caller supplies for the fields `prepare` cannot derive from the
/// library or the document (`nen_translate::artifact::assemble`'s own
/// `ArtifactMetadata` split).
pub struct TranslationMetadataSeed {
    pub id: ArtifactId,
    pub glossary: GlossaryIdentity,
    pub media_hash: Option<MediaHash>,
    pub created_at: ArtifactTimestamp,
}

/// Why a translation job was never started.
///
/// Flat and payload-free but for [`Self::Layout`], whose
/// [`BlockLayoutError`] itself carries only counts (K23 — nothing here needs
/// a hand-written `Debug`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartRefusal {
    /// The token names no catalogued, usable source.
    Unusable,
    /// The row exists but is a bitmap track — nothing to translate (§7).
    NotTranslatable,
    /// The source has no established language; `Dil Belirsiz` cannot be a
    /// translation input.
    UnknownSourceLanguage,
    /// The source is already in the requested target language (§9).
    AlreadyTargetLanguage,
    /// The row is catalogued but has no parsed document behind it yet.
    NoDocument,
    /// The document could not be split into blocks under the given config.
    Layout(BlockLayoutError),
}

impl fmt::Display for StartRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unusable => f.write_str("source is not usable"),
            Self::NotTranslatable => f.write_str("source is not translatable"),
            Self::UnknownSourceLanguage => f.write_str("source language is unknown"),
            Self::AlreadyTargetLanguage => f.write_str("source is already in the target language"),
            Self::NoDocument => f.write_str("source has no parsed document"),
            Self::Layout(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for StartRefusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Layout(error) => Some(error),
            _ => None,
        }
    }
}

/// An immutable snapshot of one translation job, fixed at [`prepare`] time.
///
/// **Security:** carries a translated-from [`SubtitleDocument`] (dialogue,
/// K23 #4) and an [`ArtifactMetadata`] that may in principle carry a
/// glossary name (K23 #8) — `Debug` is hand-written and prints shape only,
/// mirroring [`nen_translate::identity::CacheIdentityInput`]'s own impl.
pub struct TranslationJob {
    origin: SubtitleSourceId,
    document: SubtitleDocument,
    plan: TranslationPlan,
    layout: BlockLayout,
    metadata: ArtifactMetadata,
    cache_key: CacheKey,
}

impl fmt::Debug for TranslationJob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TranslationJob")
            .field("origin", &self.origin)
            .field("cue_count", &self.document.len())
            .field("target_language", &self.plan.target_language)
            .field("block_size", &self.layout.config().block_size())
            .field("overlap", &self.layout.config().overlap())
            .field("has_media_hash", &self.metadata.media_hash.is_some())
            .field(
                "has_glossary",
                &!matches!(self.metadata.glossary, GlossaryIdentity::None),
            )
            .finish()
    }
}

/// Builds a [`TranslationJob`] from the source a `token` currently names,
/// or refuses (§9's "kaynak seçmek çeviri başlatmaz" — nothing here talks to
/// a provider; it only reads the library and computes values).
///
/// Every refusal is checked before anything expensive: language checks
/// before the block layout is built, so a doomed job never touches
/// `nen-translate`'s splitting logic.
pub fn prepare(
    library: &SubtitleLibrary,
    token: u32,
    target_language: LanguageTag,
    block_layout: BlockLayoutConfig,
    provider: TranslationProviderIdentity,
    seed: TranslationMetadataSeed,
) -> Result<TranslationJob, StartRefusal> {
    if !library.is_token_usable(token) {
        return Err(StartRefusal::Unusable);
    }
    let origin = library
        .id_of(token)
        .cloned()
        .ok_or(StartRefusal::Unusable)?;
    let source = library
        .catalog()
        .get(&origin)
        .ok_or(StartRefusal::Unusable)?;
    if !source.translatable() {
        return Err(StartRefusal::NotTranslatable);
    }
    let Some(source_language) = source.language().cloned() else {
        return Err(StartRefusal::UnknownSourceLanguage);
    };
    if source_language.primary_tag() == target_language.primary_tag() {
        return Err(StartRefusal::AlreadyTargetLanguage);
    }
    let Some(document) = library.document_of(token).cloned() else {
        return Err(StartRefusal::NoDocument);
    };

    let layout = BlockLayout::of(&document, block_layout).map_err(StartRefusal::Layout)?;
    let context_terms = DocumentContext::of(&document)
        .terms()
        .iter()
        .map(|term| term.term().to_owned())
        .collect();
    let plan = TranslationPlan {
        source_language,
        target_language,
        context_terms,
    };
    let metadata = ArtifactMetadata {
        id: seed.id,
        provider,
        glossary: seed.glossary,
        media_hash: seed.media_hash,
        created_at: seed.created_at,
    };
    let cache_key: CacheKey = CacheIdentity::of(&CacheIdentityInput {
        source_fingerprint: SourceFingerprint::of(&document),
        source_language: &plan.source_language,
        target_language: &plan.target_language,
        provider: &metadata.provider,
        media_hash: metadata.media_hash,
        glossary: &metadata.glossary,
        block_layout,
    })
    .into();

    Ok(TranslationJob {
        origin,
        document,
        plan,
        layout,
        metadata,
        cache_key,
    })
}

/// The source a completed job was translated from — what
/// [`SubtitleLibrary::add_translation`] needs to name the resulting
/// `SubtitleSourceKind::Ai` entry.
impl TranslationJob {
    pub fn origin(&self) -> &SubtitleSourceId {
        &self.origin
    }
}

/// Why a started job did not produce an outcome.
///
/// Every variant delegates to an inner type that already carries no cue
/// text or private path (`BlockTranslationError`, `ArtifactError`,
/// `ArtifactStoreError` — see their own docs), so this enum derives `Debug`
/// safely rather than hand-writing one that would only repeat theirs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationError {
    /// The delivery gate was closed before or during this run; nothing was
    /// stored and nothing was checkpointed beyond what a prior run already
    /// committed.
    Cancelled,
    /// A block exhausted its repair budget, or the provider itself failed.
    Failed(BlockTranslationError),
    /// Every block reported checkpointed but `into_completed` still refused
    /// — unreachable in practice (`translate_checkpointed` only returns
    /// `Ok` once every block is committed), modelled rather than panicked.
    Incomplete,
    /// The checkpointed run's cues did not re-validate against the source
    /// document during assembly.
    Assembly(ArtifactError),
    /// The artifact store or its index refused a read or a write.
    Store(ArtifactStoreError),
    /// The separate incomplete-run store refused a read, write or cleanup.
    Resume(ResumeStoreError),
    /// The worker thread panicked before it could produce a result.
    WorkerPanicked,
}

impl fmt::Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => f.write_str("translation job cancelled"),
            Self::Failed(error) => error.fmt(f),
            Self::Incomplete => f.write_str("translation run did not complete every block"),
            Self::Assembly(error) => error.fmt(f),
            Self::Store(error) => error.fmt(f),
            Self::Resume(error) => error.fmt(f),
            Self::WorkerPanicked => f.write_str("translation worker panicked"),
        }
    }
}

impl std::error::Error for TranslationError {}

/// A finished translation job: the stored [`ArtifactRecord`], its content
/// address, and whether it came from the cache or a fresh provider run.
///
/// **Security:** hand-written `Debug` delegating to `record`'s own redacted
/// impl and `address`'s own redacted impl — neither leaks on its own, and
/// this type adds nothing that would.
pub struct TranslationOutcome {
    pub record: ArtifactRecord,
    pub address: ContentAddress,
    pub from_cache: bool,
}

impl fmt::Debug for TranslationOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TranslationOutcome")
            .field("record", &self.record)
            .field("address", &self.address)
            .field("from_cache", &self.from_cache)
            .finish()
    }
}

/// A cheap, independently cloneable capability to close a running job's
/// delivery gate (`NEN-102`).
///
/// [`TranslationJobHandle::join`] consumes `self` — the only way a caller
/// can both block on the result *and* remain able to cancel from elsewhere
/// (a UI thread handling a cancel button, say) is to have taken this handle
/// out **before** handing the job handle to whatever calls `join()`. It
/// wraps the same [`TranslationCall`] [`TranslationJobHandle::cancel`]
/// already used — no second gate, ADR-0004 Karar 2's guarantee is
/// unchanged, this only widens who can still reach it once `join()` is in
/// flight.
#[derive(Clone)]
pub struct TranslationCancelHandle {
    call: TranslationCall,
}

impl TranslationCancelHandle {
    /// Closes the job's delivery gate. Idempotent; safe to call before the
    /// job has started doing any work, while it is running, or after it has
    /// already finished.
    pub fn cancel(&self) {
        self.call.cancel();
    }
}

/// A running or finished translation job.
///
/// `cancel` reaches directly into the same [`TranslationCall`] gate the
/// worker's checkpoint boundaries check (ADR-0004 Karar 2/3) — there is no
/// second cancellation flag to keep in sync with it.
pub struct TranslationJobHandle {
    origin: SubtitleSourceId,
    call: TranslationCall,
    worker: Option<JoinHandle<Result<TranslationOutcome, TranslationError>>>,
}

impl TranslationJobHandle {
    /// The source this job was translated from — what
    /// [`SubtitleLibrary::add_translation`] needs to name the resulting
    /// `SubtitleSourceKind::Ai` entry once [`Self::join`] returns `Ok`.
    ///
    /// Captured at [`start`] time from the [`TranslationJob`] it was built
    /// from, since `start` consumes the job into the worker's closure.
    pub fn origin(&self) -> &SubtitleSourceId {
        &self.origin
    }

    /// An independent handle to this same job's delivery gate, safe to hold
    /// onto after `self` is consumed by [`Self::join`] (`NEN-102`) — see
    /// [`TranslationCancelHandle`]'s own doc comment for why one is needed.
    pub fn cancel_handle(&self) -> TranslationCancelHandle {
        TranslationCancelHandle {
            call: self.call.clone(),
        }
    }

    /// Closes the job's delivery gate. Idempotent; safe to call after the
    /// job has already finished.
    pub fn cancel(&self) {
        self.call.cancel();
    }

    pub fn is_finished(&self) -> bool {
        self.worker.as_ref().is_none_or(JoinHandle::is_finished)
    }

    /// Blocks until the job's worker thread returns. Consumes the handle —
    /// a job is joined exactly once.
    pub fn join(mut self) -> Result<TranslationOutcome, TranslationError> {
        let Some(worker) = self.worker.take() else {
            return Err(TranslationError::WorkerPanicked);
        };
        worker
            .join()
            .unwrap_or(Err(TranslationError::WorkerPanicked))
    }
}

/// Starts `job` on its own worker thread and returns immediately
/// (ADR-0004 Karar 1). The worker checks `job`'s cache identity against
/// `index` before ever calling `provider`; a hit is read from `store` and
/// returned without a single provider call.
pub fn start(
    job: TranslationJob,
    provider: Arc<dyn TranslationProvider>,
    store: Arc<dyn ArtifactStore>,
    index: Arc<dyn ArtifactIndex>,
    resume_store: Arc<dyn ResumeStore>,
    sink: Option<Arc<dyn TranslationProgressSink>>,
) -> TranslationJobHandle {
    let origin = job.origin().clone();
    let call = match sink {
        Some(sink) => TranslationCall::new(sink),
        None => TranslationCall::without_progress(),
    };
    let worker_call = call.clone();
    let worker = thread::spawn(move || {
        run_job(
            job,
            provider.as_ref(),
            store.as_ref(),
            index.as_ref(),
            resume_store.as_ref(),
            &worker_call,
        )
    });
    TranslationJobHandle {
        origin,
        call,
        worker: Some(worker),
    }
}

fn run_job(
    job: TranslationJob,
    provider: &dyn TranslationProvider,
    store: &dyn ArtifactStore,
    index: &dyn ArtifactIndex,
    resume_store: &dyn ResumeStore,
    call: &TranslationCall,
) -> Result<TranslationOutcome, TranslationError> {
    if call.is_cancelled() {
        return Err(TranslationError::Cancelled);
    }

    if let Some(entry) = index.find(job.cache_key).map_err(TranslationError::Store)? {
        let record = store.get(entry.address).map_err(TranslationError::Store)?;
        resume_store
            .delete(job.cache_key)
            .map_err(TranslationError::Resume)?;
        return Ok(TranslationOutcome {
            record,
            address: entry.address,
            from_cache: true,
        });
    }

    let completed = translate_resumable(
        provider,
        &job.document,
        &job.layout,
        &job.plan,
        call,
        job.cache_key,
        resume_store,
    )
    .map_err(|error| match error {
        ResumableTranslationError::Cancelled => TranslationError::Cancelled,
        // A cancellation observed *inside* an in-flight provider call
        // (ADR-0004 Karar 3's cooperative checkpoint) surfaces from
        // `translate_resumable` as an ordinary block failure, not its own
        // `Cancelled` variant — `repair::translate_block_with_repair`
        // propagates whatever the provider returned. From this layer's own
        // vocabulary that is still a cancellation, not a translation
        // failure, so it is normalized here rather than reported as one.
        ResumableTranslationError::Block(BlockTranslationError::Provider(
            TranslationProviderError::Cancelled,
        )) => TranslationError::Cancelled,
        ResumableTranslationError::Block(inner) => TranslationError::Failed(inner),
        ResumableTranslationError::Store(inner) => TranslationError::Resume(inner),
        ResumableTranslationError::Incomplete => TranslationError::Incomplete,
    })?;

    let artifact = artifact::assemble(
        &job.document,
        &job.layout,
        &job.plan,
        &completed,
        job.metadata.clone(),
    )
    .map_err(TranslationError::Assembly)?;

    let record = artifact.to_record();
    let address = store.put(&record).map_err(TranslationError::Store)?;
    resume_store
        .delete(job.cache_key)
        .map_err(TranslationError::Resume)?;

    Ok(TranslationOutcome {
        record,
        address,
        from_cache: false,
    })
}

/// Distinguishes artifact ids minted in the same process within the same
/// millisecond; irrelevant across process restarts, where the millisecond
/// component alone already orders them.
static ARTIFACT_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Mints an opaque [`ArtifactId`] for a job [`TranslationEnvironment::start`]
/// is about to build.
///
/// Deliberately **not** derived from the source's own identity: a user
/// file's [`SubtitleSourceId`] is a path digest (K23 #8), and an artifact id
/// is handed back across the FFI gate as a plain value that must carry
/// nothing identifying a private file. `nen:<unix_ms>-<counter>` means
/// nothing beyond "later than the last one minted here".
fn mint_artifact_id() -> ArtifactId {
    let unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or(0);
    let counter = ARTIFACT_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    ArtifactId::parse(&format!("nen:{unix_ms}-{counter}")).expect(
        "a decimal timestamp and counter joined by ':' and '-' are always a valid ArtifactId",
    )
}

/// Everything a translation job needs besides the library and the language
/// pair: the provider, the artifact store and its index.
///
/// This is `nen-app`'s composition root for `NEN-100`'s FFI surface —
/// `nen-ffi` may only depend on this crate (ADR-0006 kural 3), so wiring a
/// concrete `nen-persist` store and `nen-providers` provider together has to
/// happen on this side of the gate. The gate itself receives and returns
/// nothing more exotic than a store root path, a token and a language tag;
/// it never names `FilesystemArtifactStore` or `MockTranslationProvider`.
///
/// M5's only provider is the deterministic mock (`docs/milestones/
/// M5-translation-core.md` → Kapsam); M6 replaces it **only here**.
pub struct TranslationEnvironment {
    provider: Arc<dyn TranslationProvider>,
    store: Arc<dyn ArtifactStore>,
    index: Arc<dyn ArtifactIndex>,
    resume_store: Arc<dyn ResumeStore>,
}

impl TranslationEnvironment {
    /// Opens (creating it if needed) a content-addressed artifact store
    /// under `root` and pairs it with M5's deterministic mock provider.
    ///
    /// `FilesystemArtifactStore` implements both [`ArtifactStore`] and
    /// [`ArtifactIndex`] (`nen-persist`'s single adapter over one directory)
    /// — one object, stored here under both trait names because a running
    /// job only ever needs one of the two at a time.
    pub fn new(root: &Path) -> Result<Self, ArtifactStoreError> {
        Self::with_provider(
            root,
            Arc::new(nen_providers::translation_mock::MockTranslationProvider::new()),
        )
    }

    /// Same composition as [`Self::new`], with the provider supplied instead
    /// of hardcoded — the seam `nen-ffi`'s own
    /// `FfiTranslationEngine::with_environment` (`NEN-108`) uses to hand its
    /// integration tests a [`nen_providers::translation_mock::
    /// MockTranslationProvider`] that pauses **outside**
    /// [`TranslationCall`]'s delivery-gate lock, so a concurrent `cancel()`
    /// never has to win an OS mutex-fairness race against the very worker
    /// thread it is trying to stop. `new` is still the only constructor a
    /// real caller (M6's provider included) ever needs.
    pub fn with_provider(
        root: &Path,
        provider: Arc<dyn TranslationProvider>,
    ) -> Result<Self, ArtifactStoreError> {
        let store = Arc::new(nen_persist::FilesystemArtifactStore::new(root)?);
        Ok(Self {
            provider,
            store: store.clone() as Arc<dyn ArtifactStore>,
            index: store.clone() as Arc<dyn ArtifactIndex>,
            resume_store: store as Arc<dyn ResumeStore>,
        })
    }

    /// Builds and starts a job for `library`'s `token`, in one call.
    ///
    /// Fills in everything [`prepare`] cannot derive from the library or the
    /// document itself: a fresh [`ArtifactId`], no glossary and no media
    /// hash (neither has a producer yet — M6), the current wall-clock time,
    /// and M5's fixed default block layout (`NEN-101` offers no block-size
    /// setting). A caller across the FFI gate supplies only what a person
    /// actually chooses: which source, which target language, and whether
    /// to watch progress.
    pub fn start(
        &self,
        library: &SubtitleLibrary,
        token: u32,
        target_language: LanguageTag,
        sink: Option<Arc<dyn TranslationProgressSink>>,
    ) -> Result<TranslationJobHandle, StartRefusal> {
        let seed = TranslationMetadataSeed {
            id: mint_artifact_id(),
            glossary: GlossaryIdentity::none(),
            media_hash: None,
            created_at: ArtifactTimestamp::from_unix_ms(unix_ms_now()),
        };
        let job = prepare(
            library,
            token,
            target_language,
            BlockLayoutConfig::default(),
            self.provider.identity(),
            seed,
        )?;
        Ok(start(
            job,
            Arc::clone(&self.provider),
            Arc::clone(&self.store),
            Arc::clone(&self.index),
            Arc::clone(&self.resume_store),
            sink,
        ))
    }
}

/// Milliseconds since the Unix epoch, saturating rather than panicking on a
/// clock that reads before the epoch — the same defensive posture
/// [`ArtifactTimestamp`] itself takes no view on, since `nen-translate`
/// performs no I/O and reads no clock at all.
fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| u64::try_from(elapsed.as_millis()).ok())
        .unwrap_or(0)
}
