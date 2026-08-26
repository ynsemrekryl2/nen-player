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
use nen_catalog::SubtitleSourceCatalog;
use nen_domain::source::{SubtitleSource, SubtitleSourceId};
use nen_domain::subtitle::SubtitleDocument;
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

/// Everything known about the current medium's subtitles.
#[derive(Default)]
pub struct SubtitleLibrary {
    catalog: SubtitleSourceCatalog,
    documents: HashMap<SubtitleSourceId, SubtitleDocument>,
    defects: HashMap<SubtitleSourceId, SourceDefect>,
}

impl SubtitleLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the embedded tracks of a freshly loaded medium (NEN-023).
    pub fn add_embedded(&mut self, sources: Vec<SubtitleSource>) {
        for source in sources {
            self.catalog.insert(source);
        }
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
