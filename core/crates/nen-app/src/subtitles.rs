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
    LanguageTag, SubtitlePreferences, SubtitleSource, SubtitleSourceId, SubtitleSourceKind,
};
use nen_domain::subtitle::SubtitleDocument;
use nen_ports::playback::TrackId;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

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

    /// Looks beside the medium for a sidecar with the same basename.
    ///
    /// `None` when there is nothing there to look at — a medium without a
    /// filename, or simply no sidecar. That is the ordinary case and not a
    /// rejection: a refusal only exists once there is a file to refuse.
    pub fn add_sidecar_of(&mut self, media: &Path) -> Option<AddOutcome> {
        let sidecar = subtitle_files::sidecar_of(media)?;
        if !sidecar.exists() {
            return None;
        }
        let root = media.parent()?;
        Some(self.add_file(&sidecar, root))
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

    /// The engine track a token refers to, or `None` when the row is not one.
    ///
    /// `None` is the ordinary answer for a user file: it is a perfectly good
    /// row that simply is not a track, and showing it is NEN-027's job.
    pub fn embedded_track_of(&self, token: u32) -> Option<TrackId> {
        crate::embedded::track_of(self.id_of(token)?)
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
            .finish()
    }
}
