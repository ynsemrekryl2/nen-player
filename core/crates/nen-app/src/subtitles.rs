//! The subtitle sources known for the medium being played, and the documents
//! behind the ones that could be read (NEN-025).
//!
//! [`nen_catalog::SubtitleSourceCatalog`] holds entries and nothing else — by
//! design, since ADR-0006 gives that crate no way to read a file. Two things
//! have to live next to it once files are involved, and this is the layer that
//! may own both: the parsed documents (NEN-027 renders one, NEN-044 and M5 read
//! one) and the mark on the entries that turned out to be broken (ADR-0031
//! Karar 5).
//!
//! # What is deliberately absent
//!
//! There is no eager work here. Adding a source reads exactly the one file it
//! was given; nothing scans, nothing fetches, nothing decodes a track. §7's
//! lazy rule and ADR-0031 Karar 4 ("the menu does not wait for the scan") are
//! both properties of the caller's timing, and this type stays out of the way
//! of both by never doing more than it was asked.

use crate::subtitle_files::{self, FileRejection, LoadedFile, SourceDefect};
use nen_catalog::{MenuGroup, SubtitleSourceCatalog};
use nen_domain::source::{
    LanguageTag, SubtitlePreferences, SubtitleSource, SubtitleSourceBadges, SubtitleSourceId,
    SubtitleSourceKind,
};
use nen_domain::subtitle::SubtitleDocument;
use nen_ports::credentials::{CredentialKind, CredentialStoreError, SecureCredentialStore};
use nen_ports::http::HttpClient;
use nen_ports::playback::TrackId;
use nen_ports::subtitle_candidates::SubtitleCandidate;
use nen_ports::subtitle_download::{
    SubtitleDownloadError, SubtitleDownloadRequest, SubtitleDownloader,
};
use nen_providers::opensubtitles::{OpenSubtitlesApiKey, OpenSubtitlesDownloader};
use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Why an explicit OpenSubtitles subtitle selection could not attach a
/// document. Every variant is payload-free: URLs, provider payloads,
/// credentials, private ids and subtitle dialogue stay inside the adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadRefusal {
    NotOpenSubtitles,
    CredentialStore(CredentialStoreError),
    MissingCredential,
    InvalidCredential,
    Provider(SubtitleDownloadError),
    InvalidEncoding,
    MalformedSubtitle,
    Cancelled,
}

impl fmt::Display for DownloadRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotOpenSubtitles => f.write_str("source is not an OpenSubtitles row"),
            Self::CredentialStore(error) => error.fmt(f),
            Self::MissingCredential => f.write_str("OpenSubtitles credential is missing"),
            Self::InvalidCredential => f.write_str("OpenSubtitles credential is invalid"),
            Self::Provider(error) => error.fmt(f),
            Self::InvalidEncoding => f.write_str("subtitle encoding was invalid"),
            Self::MalformedSubtitle => f.write_str("subtitle format was malformed"),
            Self::Cancelled => f.write_str("subtitle download was cancelled"),
        }
    }
}

impl std::error::Error for DownloadRefusal {}

/// What adding one file did.
///
/// The shell needs the three apart for one reason only (ADR-0031 Karar 1 and
/// 5): a [`Rejected`](Self::Rejected) file the user picked themselves earns a
/// transient notification, the same file found by a sidecar scan earns silence,
/// and a [`Defective`](Self::Defective) one earns neither — it is in the menu,
/// which is where it says so itself.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AddOutcome {
    /// Catalogued and usable.
    Added,
    /// Catalogued and marked unusable.
    Defective(SourceDefect),
    /// Not catalogued. A gate refused it.
    Rejected(FileRejection),
}

/// Why [`SubtitleLibrary::attach_embedded_document`] refused a document
/// (`NEN-044`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttachEmbeddedDocumentError {
    /// The token does not name an embedded row — either it is a user file or
    /// an AI translation (both already own their document through a
    /// different call), or the token itself is stale.
    NotAnEmbeddedRow,
}

/// One menu row, as the shell needs it (NEN-026).
///
/// [`nen_catalog::project`] answers *where* a source sits; this type adds the
/// two things the projection cannot know because they are not the catalog's:
/// the [`token`](Self::token) that lets a shell name a row without holding an
/// identity, and the [`defect`](Self::defect) that makes the row unselectable
/// (ADR-0031 Karar 5).
#[derive(Clone, PartialEq, Eq)]
pub struct MenuEntryView {
    /// Opaque, stable for the life of this library. **Not** an identity.
    ///
    /// A [`SubtitleSourceId`]'s key is a path digest or a provider id (K23 #8),
    /// and the shell's generated struct would print it on any `String(
    /// reflecting:)`. A counter cannot leak what it does not carry, and it is
    /// stable across a scan so a growing list never renumbers a row.
    pub token: u32,
    pub kind: SubtitleSourceKind,
    pub language: Option<LanguageTag>,
    pub label: String,
    /// Why this row cannot be used, when it cannot.
    pub defect: Option<SourceDefect>,
    pub translatable: bool,
    pub hearing_impaired: bool,
    pub ai_translated: bool,
}

impl fmt::Debug for MenuEntryView {
    /// Prints everything except the label, which is regularly a private
    /// filename (K23 #8) — the same line [`SubtitleSource`] holds.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MenuEntryView")
            .field("token", &self.token)
            .field("kind", &self.kind)
            .field("language", &self.language)
            .field("has_label", &!self.label.is_empty())
            .field("defect", &self.defect)
            .field("translatable", &self.translatable)
            .field("hearing_impaired", &self.hearing_impaired)
            .field("ai_translated", &self.ai_translated)
            .finish()
    }
}

/// A menu heading and the rows under it, in menu order.
///
/// The order, and which headings exist at all, is
/// [`nen_catalog::project`]'s answer and is re-derived on every call. A
/// heading with no rows never appears — `Kullanıcı Altyazıları` and
/// `Dil Belirsiz` show up the moment they have something and not before (§8).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MenuSectionView {
    pub group: MenuGroup,
    pub entries: Vec<MenuEntryView>,
}

/// Everything known about the current medium's subtitles.
#[derive(Default)]
pub struct SubtitleLibrary {
    catalog: SubtitleSourceCatalog,
    documents: HashMap<SubtitleSourceId, SubtitleDocument>,
    defects: HashMap<SubtitleSourceId, SourceDefect>,
    /// Token by identity, and identity by token. Two maps rather than one
    /// scan: the menu is re-projected on every redraw, and a selection is
    /// resolved on every click.
    tokens: HashMap<SubtitleSourceId, u32>,
    by_token: Vec<SubtitleSourceId>,
    /// OpenSubtitles' private download id, keyed by the public catalog id.
    /// This map never crosses the FFI boundary and is intentionally absent
    /// from `Debug` (ADR-0021, K23 #8).
    opensubtitles_file_ids: HashMap<SubtitleSourceId, u64>,
}

impl SubtitleLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the embedded tracks of a freshly loaded medium (NEN-023).
    pub fn add_embedded(&mut self, sources: Vec<SubtitleSource>) {
        for source in sources {
            self.remember(source.id());
            self.catalog.insert(source);
        }
    }

    /// Catalogues OpenSubtitles metadata without downloading a subtitle.
    ///
    /// The public subtitle id becomes the catalog identity. The private file
    /// id is retained only in this application-owned map for NEN-122's later
    /// explicit selection flow; no document is attached and no download is
    /// attempted here.
    pub fn add_opensubtitles(&mut self, candidates: Vec<SubtitleCandidate>) -> Vec<u32> {
        candidates
            .into_iter()
            .map(|candidate| {
                let id = SubtitleSourceId::opensubtitles(&candidate.public_id);
                let token = self.remember(&id);
                self.opensubtitles_file_ids
                    .insert(id.clone(), candidate.private_file_id);
                let label = candidate
                    .release_name
                    .unwrap_or_else(|| "OpenSubtitles".to_owned());
                let source = SubtitleSource::new(id.clone(), Some(candidate.language), label)
                    .with_badges(SubtitleSourceBadges {
                        hearing_impaired: candidate.hearing_impaired,
                        ai_translated: candidate.ai_translated,
                    });
                self.defects.remove(&id);
                self.documents.remove(&id);
                self.catalog.insert(source);
                token
            })
            .collect()
    }

    /// Copies OpenSubtitles rows from a worker-owned library into this
    /// medium's catalog. The worker library is deliberately separate from the
    /// live one: a late search or download must not mutate the next medium
    /// after `clear()` has replaced the live catalog (NEN-123).
    pub fn merge_opensubtitles_from(&mut self, other: &SubtitleLibrary) {
        let entries: Vec<_> = other
            .catalog
            .of_kind(SubtitleSourceKind::OpenSubtitles)
            .filter_map(|source| {
                let file_id = other.opensubtitles_file_ids.get(source.id()).copied()?;
                Some((
                    source.clone(),
                    file_id,
                    other.documents.get(source.id()).cloned(),
                ))
            })
            .collect();

        for (source, file_id, document) in entries {
            let id = source.id().clone();
            self.remember(&id);
            self.opensubtitles_file_ids.insert(id.clone(), file_id);
            self.defects.remove(&id);
            if let Some(document) = document {
                self.documents.insert(id.clone(), document);
            } else {
                self.documents.remove(&id);
            }
            self.catalog.insert(source);
        }
    }

    /// Copies one OpenSubtitles row into a worker-owned library before an
    /// explicit download. No other source kind can cross this narrow seam.
    pub fn copy_opensubtitles_entry_from(&mut self, other: &SubtitleLibrary, token: u32) -> bool {
        let Some(id) = other.id_of(token) else {
            return false;
        };
        if id.kind() != SubtitleSourceKind::OpenSubtitles {
            return false;
        }
        let Some(file_id) = other.opensubtitles_file_ids.get(id).copied() else {
            return false;
        };
        let Some(source) = other.catalog.get(id).cloned() else {
            return false;
        };
        self.remember(id);
        self.opensubtitles_file_ids.insert(id.clone(), file_id);
        self.catalog.insert(source);
        true
    }

    /// Gives a source its token, once and for good.
    ///
    /// Re-adding a source keeps the token it already had. That is the whole
    /// point: a user who reloads a fixed sidecar, or a scan that upserts an
    /// entry, must not renumber the row a shell is currently pointing at
    /// (ADR-0031 Karar 4.1).
    fn remember(&mut self, id: &SubtitleSourceId) -> u32 {
        if let Some(token) = self.tokens.get(id) {
            return *token;
        }
        // 1-based: a shell that stores "no selection" as zero cannot collide
        // with a real row by forgetting to wrap it.
        let token = self.by_token.len() as u32 + 1;
        self.by_token.push(id.clone());
        self.tokens.insert(id.clone(), token);
        token
    }

    /// Loads a file the user picked, from a directory it must stay inside.
    ///
    /// Reloading the same file is not an error and does not duplicate it: the
    /// identity is a digest of the canonical path (ADR-0010 Karar 2) and the
    /// catalog upserts in place, so the entry keeps its position in the menu.
    pub fn add_file(&mut self, path: &Path, root: &Path) -> AddOutcome {
        match subtitle_files::load(path, root) {
            LoadedFile::Loaded { source, document } => {
                let id = source.id().clone();
                self.remember(&id);
                // A file that parses now replaces whatever it was before, so a
                // user who fixed a broken sidecar and reloaded it gets a
                // working entry rather than a stale mark.
                self.defects.remove(&id);
                self.documents.insert(id, document);
                self.catalog.insert(source);
                AddOutcome::Added
            }
            LoadedFile::Defective { source, defect } => {
                let id = source.id().clone();
                self.remember(&id);
                self.documents.remove(&id);
                self.defects.insert(id, defect);
                self.catalog.insert(source);
                AddOutcome::Defective(defect)
            }
            // Nothing is recorded — not the entry, not the defect, not the
            // fact that the path was ever mentioned (ADR-0031 Karar 5).
            LoadedFile::Rejected(rejection) => AddOutcome::Rejected(rejection),
        }
    }

    /// Looks beside the medium for sidecars carrying its basename.
    ///
    /// Empty when there is nothing there to look at — a medium without a
    /// filename, or simply no sidecar. That is the ordinary case and not a
    /// rejection: a refusal only exists once there is a file to refuse, which
    /// is also why a candidate that is not there is skipped rather than
    /// reported. One outcome per candidate that exists, in the order
    /// [`subtitle_files::sidecars_of`] fixed.
    ///
    /// Each candidate goes through [`Self::add_file`] — the same body the file
    /// picker reaches, so ADR-0041 widens the candidate set without opening a
    /// second way into the catalog or a second set of gates.
    pub fn add_sidecars_of(&mut self, media: &Path) -> Vec<AddOutcome> {
        let Some(root) = media.parent() else {
            return Vec::new();
        };
        subtitle_files::sidecars_of(media)
            .into_iter()
            .filter(|candidate| candidate.exists())
            .map(|candidate| self.add_file(&candidate, root))
            .collect()
    }

    pub fn catalog(&self) -> &SubtitleSourceCatalog {
        &self.catalog
    }

    /// The parsed document behind a source, when there is one.
    pub fn document(&self, id: &SubtitleSourceId) -> Option<&SubtitleDocument> {
        self.documents.get(id)
    }

    /// Why a catalogued source cannot be used, when it cannot.
    pub fn defect(&self, id: &SubtitleSourceId) -> Option<SourceDefect> {
        self.defects.get(id).copied()
    }

    /// Whether a source is selectable — catalogued, and not marked broken.
    pub fn is_usable(&self, id: &SubtitleSourceId) -> bool {
        self.catalog.get(id).is_some() && !self.defects.contains_key(id)
    }

    /// The token standing for a source, when it has one.
    pub fn token_of(&self, id: &SubtitleSourceId) -> Option<u32> {
        self.tokens.get(id).copied()
    }

    /// The source a token stands for.
    pub fn id_of(&self, token: u32) -> Option<&SubtitleSourceId> {
        self.by_token
            .get(usize::try_from(token.checked_sub(1)?).ok()?)
    }

    /// Resolves an OpenSubtitles private file id for the later explicit
    /// download task. It is application-internal data and is not exposed by
    /// `nen-ffi` or any menu record.
    pub fn opensubtitles_file_id(&self, token: u32) -> Option<u64> {
        let id = self.id_of(token)?;
        if id.kind() != SubtitleSourceKind::OpenSubtitles {
            return None;
        }
        self.opensubtitles_file_ids.get(id).copied()
    }

    /// Downloads and attaches the selected OpenSubtitles document in memory.
    ///
    /// The operation is deliberately synchronous at this core seam: the
    /// platform supplies the bounded transport, while the caller supplies a
    /// media revision to make a late response harmless. A row with an already
    /// attached document is idempotent and never reads credentials or sends a
    /// second request.
    pub fn download_opensubtitles(
        &mut self,
        token: u32,
        credentials: &dyn SecureCredentialStore,
        http: &dyn HttpClient,
    ) -> Result<(), DownloadRefusal> {
        let revision = AtomicU64::new(0);
        self.download_opensubtitles_if_current(token, 0, &revision, credentials, http)
    }

    /// The revision-aware form used by a playback/session owner. No document
    /// is committed unless the media is still the one selected at request
    /// start and every byte gate, decoder and strict SRT parser has passed.
    pub fn download_opensubtitles_if_current(
        &mut self,
        token: u32,
        expected_revision: u64,
        current_revision: &AtomicU64,
        credentials: &dyn SecureCredentialStore,
        http: &dyn HttpClient,
    ) -> Result<(), DownloadRefusal> {
        if self.document_of(token).is_some() {
            return Ok(());
        }

        let id = self
            .id_of(token)
            .cloned()
            .ok_or(DownloadRefusal::NotOpenSubtitles)?;
        if id.kind() != SubtitleSourceKind::OpenSubtitles {
            return Err(DownloadRefusal::NotOpenSubtitles);
        }
        let private_file_id = self
            .opensubtitles_file_ids
            .get(&id)
            .copied()
            .ok_or(DownloadRefusal::NotOpenSubtitles)?;
        if current_revision.load(Ordering::Acquire) != expected_revision {
            return Err(DownloadRefusal::Cancelled);
        }

        let api_key = credentials
            .get(CredentialKind::OpenSubtitles)
            .map_err(DownloadRefusal::CredentialStore)?
            .ok_or(DownloadRefusal::MissingCredential)?;
        let api_key = OpenSubtitlesApiKey::new(api_key.expose())
            .map_err(|_| DownloadRefusal::InvalidCredential)?;
        let downloader = OpenSubtitlesDownloader::new(http, api_key);
        let downloaded = downloader
            .download(SubtitleDownloadRequest::new(private_file_id))
            .map_err(DownloadRefusal::Provider)?;
        if current_revision.load(Ordering::Acquire) != expected_revision {
            return Err(DownloadRefusal::Cancelled);
        }

        let text = nen_subtitle::encoding::decode(downloaded.bytes())
            .map_err(|_| DownloadRefusal::InvalidEncoding)?;
        let document =
            nen_subtitle::srt::parse(&text).map_err(|_| DownloadRefusal::MalformedSubtitle)?;
        if current_revision.load(Ordering::Acquire) != expected_revision {
            return Err(DownloadRefusal::Cancelled);
        }

        self.documents.insert(id, document);
        Ok(())
    }

    /// The menu of §8, ready to draw.
    ///
    /// Derived on every call and stored nowhere, exactly as
    /// [`nen_catalog::project`] is. Which headings exist, their order and the
    /// order inside them are **that** function's answers; this one only
    /// attaches the token and the defect. Re-deciding any of it here would put
    /// a second opinion about §8 next to the golden files that pin the first.
    pub fn menu(&self, preferences: &SubtitlePreferences) -> Vec<MenuSectionView> {
        nen_catalog::project(&self.catalog, preferences)
            .sections
            .into_iter()
            .map(|section| MenuSectionView {
                group: section.group,
                entries: section
                    .entries
                    .into_iter()
                    .map(|e| self.view_of(e))
                    .collect(),
            })
            .collect()
    }

    /// The subtitle to show when playback starts, as a token, or `None` for off.
    ///
    /// A thin wrapper on [`nen_catalog::auto_selection`]; the tier order and
    /// the language rule are that function's, written out once
    /// (ADR-0010 Karar 9). **Calling it more than once per medium is the
    /// caller's mistake** — ADR-0031 Karar 4.3 allows exactly one, at the
    /// start, and a source found later never re-triggers it.
    pub fn auto_selection(&self, preferences: &SubtitlePreferences) -> Option<u32> {
        nen_catalog::auto_selection(&self.catalog, preferences)
            .and_then(|source| self.token_of(source.id()))
    }

    /// The NEN-038 opt-in form of [`Self::auto_selection`]. The caller owns
    /// the user setting, identity-confidence and daily-attempt gates; this
    /// wrapper only projects the pure catalog decision to a token.
    pub fn auto_selection_with_opensubtitles(
        &self,
        preferences: &SubtitlePreferences,
        include_opensubtitles: bool,
    ) -> Option<u32> {
        nen_catalog::auto_selection_with_opensubtitles(
            &self.catalog,
            preferences,
            include_opensubtitles,
        )
        .and_then(|source| self.token_of(source.id()))
    }

    /// Whether the row a token names may be shown.
    ///
    /// `false` for a token nothing was ever given, which is the answer a shell
    /// holding a stale row gets — not an error, because a menu that shrank
    /// underneath a click is an ordinary race, not a fault.
    pub fn is_token_usable(&self, token: u32) -> bool {
        self.id_of(token).is_some_and(|id| self.is_usable(id))
    }

    /// The parsed document a token refers to, when the row has one.
    ///
    /// `None` for an embedded row that has not been extracted yet: that one is
    /// the engine's own track and starts with no document on this side, until
    /// [`Self::attach_embedded_document`] (`NEN-044`) gives it one. The two
    /// together are what let a caller decide how a row is shown without
    /// knowing what kind it is.
    pub fn document_of(&self, token: u32) -> Option<&SubtitleDocument> {
        self.document(self.id_of(token)?)
    }

    /// The engine track a token refers to, or `None` when the row is not one.
    ///
    /// `None` is the ordinary answer for a user file: it is a perfectly good
    /// row that simply is not a track, and showing it is NEN-027's job.
    pub fn embedded_track_of(&self, token: u32) -> Option<TrackId> {
        crate::embedded::track_of(self.id_of(token)?)
    }

    /// Records the text a lazy embedded-track extraction produced (`NEN-044`),
    /// so a later [`Self::document_of`] call finds it without asking the
    /// engine a second time.
    ///
    /// Narrow on purpose: refuses a token that is not an embedded row (a user
    /// file already has its document from [`Self::add_file`], and asking this
    /// call to overwrite it would let two different sources of truth disagree
    /// about the same row), and is a silent no-op when the row already has a
    /// document — a second extraction request for the same token is the
    /// ordinary case (the user reopens the "Altyazı" menu, or the translation
    /// gate is asked twice), not a reason to decode the container again.
    pub fn attach_embedded_document(
        &mut self,
        token: u32,
        document: SubtitleDocument,
    ) -> Result<(), AttachEmbeddedDocumentError> {
        if self.embedded_track_of(token).is_none() {
            return Err(AttachEmbeddedDocumentError::NotAnEmbeddedRow);
        }
        // `embedded_track_of` having answered `Some` proves `id_of` resolves.
        let id = self.id_of(token).expect("embedded row has an id").clone();
        self.documents.entry(id).or_insert(document);
        Ok(())
    }

    /// Catalogues a finished translation as a `SubtitleSourceKind::Ai` entry
    /// (`NEN-099`, product-spec §9).
    ///
    /// A separate, explicit call — never something a translation job does
    /// to itself. That split is what makes "kullanıcı başka source
    /// izliyorsa zorla AI çıktısına geçilmez" structural: nothing in
    /// `crate::translation` holds a `&mut SubtitleLibrary`, so a running or
    /// even a finished job has no path into the menu until a caller makes
    /// this call.
    ///
    /// `SubtitleSourceId::ai(origin, target)` is the same identity NEN-019
    /// already defined; re-translating one source into a language it was
    /// already translated to upserts the existing entry rather than adding
    /// a second row (the identity's own dedup contract, exercised by
    /// `nen-catalog`'s `dedup_covers_all_four_kinds`).
    pub fn add_translation(
        &mut self,
        origin: &SubtitleSourceId,
        outcome: &crate::translation::TranslationOutcome,
    ) -> u32 {
        let record = &outcome.record;
        let id = SubtitleSourceId::ai(origin, &record.target_language);
        let token = self.remember(&id);
        let label = format!("AI çevirisi ({})", record.target_language);
        let source = SubtitleSource::new(id.clone(), Some(record.target_language.clone()), label);
        self.defects.remove(&id);
        self.documents.insert(id, record.document.clone());
        self.catalog.insert(source);
        token
    }

    fn view_of(&self, source: &SubtitleSource) -> MenuEntryView {
        MenuEntryView {
            // Every catalogued source was remembered on the way in, so the
            // fallback is unreachable; zero is chosen anyway because it is the
            // one value a shell reads as "no row" rather than the wrong row.
            token: self.token_of(source.id()).unwrap_or(0),
            kind: source.kind(),
            language: source.language().cloned(),
            label: source.label().to_owned(),
            defect: self.defect(source.id()),
            translatable: source.translatable(),
            hearing_impaired: source.badges().hearing_impaired,
            ai_translated: source.badges().ai_translated,
        }
    }
}

impl fmt::Debug for SubtitleLibrary {
    /// Prints counts only.
    ///
    /// Every entry carries a label that is usually a private filename (K23 #8)
    /// and every document carries dialogue (K23 #4). Neither has any reason to
    /// appear in a log, so the container refuses to delegate to either.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubtitleLibrary")
            .field("source_count", &self.catalog.len())
            .field("document_count", &self.documents.len())
            .field("defect_count", &self.defects.len())
            .field(
                "opensubtitles_file_id_count",
                &self.opensubtitles_file_ids.len(),
            )
            .finish()
    }
}
