//! The subtitle library, across the gate (NEN-025).
//!
//! A gate and nothing else, on the same terms as [`crate::session`]: every
//! method translates its arguments, calls
//! [`nen_app::subtitles::SubtitleLibrary`] and translates the answer back.
//!
//! # Security (K23)
//!
//! A path goes **in** and never comes back out. What the shell receives is a
//! reason from a closed set — no path, no filename, no line of dialogue. That
//! is deliberate and it is also what ADR-0031 Karar 1 and 2 need: the transient
//! notification the shell shows for a refused file names the *variant*, not the
//! file, and there is no way for it to name the file because this gate never
//! offered one.
//!
//! The enums below carry no payload, so `Debug` is derived on them rather than
//! hand-written — there is nothing for a derive to leak. The library object
//! itself derives none, for the reason [`crate::session`] gives.

use crate::playback::FfiTrackDescriptor;
use nen_app::catalog::MenuGroup;
use nen_app::domain::source::{LanguageTag, SubtitlePreferences, SubtitleSourceKind};
use nen_app::embedded::embedded_sources;
use nen_app::ports::playback::TrackDescriptor;
use nen_app::subtitle_files::{FileRejection, SourceDefect};
use nen_app::subtitles::{AddOutcome, MenuEntryView, MenuSectionView, SubtitleLibrary};
use std::fmt;
use std::path::Path;
use std::sync::Mutex;

/// Why a catalogued source cannot be used (ADR-0031 Karar 5).
///
/// The shell turns these into the menu's reason labels in NEN-026. It is a
/// closed set on purpose: an open one would eventually carry a parser message,
/// and a parser message carries the line it failed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiSourceDefect {
    Unreadable,
    Malformed,
}

impl From<SourceDefect> for FfiSourceDefect {
    fn from(defect: SourceDefect) -> Self {
        match defect {
            SourceDefect::Unreadable => Self::Unreadable,
            SourceDefect::Malformed => Self::Malformed,
        }
    }
}

/// Why a file never became a source at all (`security-policy.md` §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiFileRejection {
    Traversal,
    Symlink,
    NotRegularFile,
    TooLarge,
}

impl From<FileRejection> for FfiFileRejection {
    fn from(rejection: FileRejection) -> Self {
        match rejection {
            FileRejection::Traversal => Self::Traversal,
            FileRejection::Symlink => Self::Symlink,
            FileRejection::NotRegularFile => Self::NotRegularFile,
            FileRejection::TooLarge => Self::TooLarge,
        }
    }
}

/// What adding one file did.
///
/// Not a `Result`: none of the three is an error the shell should handle as
/// one. A refused file and a broken file are both **outcomes** the product has
/// a defined behaviour for, and modelling them as failures would push them onto
/// the same path as a load that genuinely could not proceed — which is exactly
/// the collapse ADR-0031 Karar 1 separates into three classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiSubtitleOutcome {
    Added,
    Defective { reason: FfiSourceDefect },
    Rejected { reason: FfiFileRejection },
}

impl From<AddOutcome> for FfiSubtitleOutcome {
    fn from(outcome: AddOutcome) -> Self {
        match outcome {
            AddOutcome::Added => Self::Added,
            AddOutcome::Defective(defect) => Self::Defective {
                reason: defect.into(),
            },
            AddOutcome::Rejected(rejection) => Self::Rejected {
                reason: rejection.into(),
            },
        }
    }
}

/// Which kind of source a row is — §8's origin badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum FfiSubtitleSourceKind {
    Embedded,
    User,
    OpenSubtitles,
    Ai,
}

impl From<SubtitleSourceKind> for FfiSubtitleSourceKind {
    fn from(kind: SubtitleSourceKind) -> Self {
        match kind {
            SubtitleSourceKind::Embedded => Self::Embedded,
            SubtitleSourceKind::User => Self::User,
            SubtitleSourceKind::OpenSubtitles => Self::OpenSubtitles,
            SubtitleSourceKind::Ai => Self::Ai,
        }
    }
}

/// One heading of §8's menu.
///
/// Carries no display text, for the reason [`nen_catalog::MenuGroup`] gives
/// (ADR-0010 Karar 7): a language's visible name is its own name in every UI
/// language, and the rest of the chrome is translated by the platform. The tag
/// here is region-free — `en` and `en-us` share one heading (ADR-0030).
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum FfiMenuGroup {
    Closed,
    UserSubtitles,
    Language { tag: String },
    UnknownLanguage,
}

impl From<MenuGroup> for FfiMenuGroup {
    fn from(group: MenuGroup) -> Self {
        match group {
            MenuGroup::Closed => Self::Closed,
            MenuGroup::UserSubtitles => Self::UserSubtitles,
            MenuGroup::Language(tag) => Self::Language {
                tag: tag.as_str().to_owned(),
            },
            MenuGroup::UnknownLanguage => Self::UnknownLanguage,
        }
    }
}

/// One row of the menu.
#[derive(Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiMenuEntry {
    /// What the shell hands back to select this row.
    ///
    /// Opaque and stable for the life of the library. It is **not** the
    /// source's identity: that identity is a path digest or a provider id
    /// (K23 #8), and a generated Swift struct prints every field it holds. A
    /// counter has nothing to print.
    pub token: u32,
    pub kind: FfiSubtitleSourceKind,
    /// The source's own full tag, which may carry a region even when its
    /// heading does not.
    pub language: Option<String>,
    /// What §8 shows on the row. Empty for a titleless embedded track — the
    /// shell fills that in with the language's endonym.
    pub label: String,
    /// Set when the row is in the menu but cannot be used (ADR-0031 Karar 5).
    /// A row carrying one is drawn dimmed and is **not** selectable.
    pub defect: Option<FfiSourceDefect>,
    pub translatable: bool,
}

impl From<MenuEntryView> for FfiMenuEntry {
    fn from(view: MenuEntryView) -> Self {
        Self {
            token: view.token,
            kind: view.kind.into(),
            language: view.language.map(|tag| tag.as_str().to_owned()),
            label: view.label,
            defect: view.defect.map(Into::into),
            translatable: view.translatable,
        }
    }
}

impl fmt::Debug for FfiMenuEntry {
    /// Prints everything except the label — regularly a private filename
    /// (K23 #8), on the same terms as [`FfiTrackDescriptor`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FfiMenuEntry")
            .field("token", &self.token)
            .field("kind", &self.kind)
            .field("language", &self.language)
            .field("has_label", &!self.label.is_empty())
            .field("defect", &self.defect)
            .field("translatable", &self.translatable)
            .finish()
    }
}

/// A heading and its rows, in menu order.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiMenuSection {
    pub group: FfiMenuGroup,
    pub entries: Vec<FfiMenuEntry>,
}

impl From<MenuSectionView> for FfiMenuSection {
    fn from(view: MenuSectionView) -> Self {
        Self {
            group: view.group.into(),
            entries: view.entries.into_iter().map(Into::into).collect(),
        }
    }
}

/// The two preferred languages, as the shell holds them.
///
/// Strings rather than a parsed type: the shell seeds these from the system
/// locale, which hands out things like `tr-TR`, and a tag it cannot parse is
/// the same as no preference at all — never an error the user has to see.
fn preferences(primary: Option<String>, secondary: Option<String>) -> SubtitlePreferences {
    let parse = |value: Option<String>| value.and_then(|tag| LanguageTag::parse(&tag).ok());
    SubtitlePreferences::new(parse(primary), parse(secondary))
}

/// The subtitle sources known for one medium.
#[derive(uniffi::Object)]
pub struct FfiSubtitleLibrary {
    inner: Mutex<SubtitleLibrary>,
}

#[uniffi::export]
impl FfiSubtitleLibrary {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(SubtitleLibrary::new()),
        }
    }

    /// Loads a subtitle file the user picked themselves.
    ///
    /// The root the file must stay inside is its own directory. On every
    /// platform this method is reached from an OS file picker, and the picker
    /// has already decided where the user may look; re-deciding it here would
    /// refuse legitimate files while proving nothing. What §4 #3 still buys is
    /// the check that the *spelling* is honest — a `..` in a path that arrived
    /// from somewhere other than a picker is refused.
    pub fn add_file(&self, path: String) -> FfiSubtitleOutcome {
        let path = Path::new(&path);
        let root = path.parent().unwrap_or(path);
        lock(&self.inner).add_file(path, root).into()
    }

    /// Looks beside a medium for a sidecar with the same basename.
    ///
    /// `None` when there is nothing beside it — the ordinary case, and not a
    /// refusal. A refusal only exists once there is a file to refuse, and
    /// ADR-0031 Karar 5 has the shell stay silent about those anyway.
    pub fn add_sidecar_for(&self, media_path: String) -> Option<FfiSubtitleOutcome> {
        lock(&self.inner)
            .add_sidecar_of(Path::new(&media_path))
            .map(Into::into)
    }

    /// Catalogues the embedded subtitle tracks of a freshly loaded medium.
    ///
    /// The shell hands over what the engine reported and forgets it; which of
    /// those tracks is a subtitle, which carries text and how each becomes a
    /// catalog entry is [`embedded_sources`]'s answer, written once for every
    /// platform (NEN-023).
    pub fn add_embedded(&self, tracks: Vec<FfiTrackDescriptor>) {
        let tracks: Vec<TrackDescriptor> = tracks.into_iter().map(Into::into).collect();
        lock(&self.inner).add_embedded(embedded_sources(&tracks));
    }

    /// §8's menu, ready to draw.
    ///
    /// Re-derived on every call. Which headings exist, their order, the order
    /// inside them and the rule that an empty heading never appears are all
    /// `nen-catalog`'s — a shell that re-decided any of it would be a second
    /// opinion about §8 next to the goldens that pin the first.
    pub fn menu(&self, primary: Option<String>, secondary: Option<String>) -> Vec<FfiMenuSection> {
        lock(&self.inner)
            .menu(&preferences(primary, secondary))
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// The row to show when playback starts, or `None` for off.
    ///
    /// **Called once per medium, at the start** (ADR-0031 Karar 4.3). A source
    /// discovered later never re-triggers it, however well it matches: the
    /// subtitle a user is watching must not change without them.
    pub fn auto_selection(
        &self,
        primary: Option<String>,
        secondary: Option<String>,
    ) -> Option<u32> {
        lock(&self.inner).auto_selection(&preferences(primary, secondary))
    }

    /// The engine track a row refers to, or `None` when the row is not one.
    ///
    /// `None` is the ordinary answer for a user file. Showing that file is
    /// NEN-027's job; nothing here fails because a row is not a track.
    pub fn embedded_track_of(&self, token: u32) -> Option<u32> {
        lock(&self.inner)
            .embedded_track_of(token)
            .map(|track| track.index())
    }

    /// Whether a row may be selected — in the catalog, and not marked broken.
    pub fn is_usable(&self, token: u32) -> bool {
        let library = lock(&self.inner);
        library.id_of(token).is_some_and(|id| library.is_usable(id))
    }

    /// How many sources the catalog holds.
    ///
    /// Kept from NEN-025: it is the one number that proves the wiring end to
    /// end — loading one file twice leaves one entry — without walking the
    /// projection to count rows.
    pub fn source_count(&self) -> u32 {
        lock(&self.inner).catalog().len() as u32
    }

    /// Forgets everything. Called when a new medium is loaded.
    pub fn clear(&self) {
        *lock(&self.inner) = SubtitleLibrary::new();
    }
}

impl Default for FfiSubtitleLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Takes the lock without caring whether a previous holder panicked, on the
/// same reasoning as [`crate::session`]: the catalog behind it is still true.
fn lock(mutex: &Mutex<SubtitleLibrary>) -> std::sync::MutexGuard<'_, SubtitleLibrary> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_is_refused_rather_than_reported_as_broken() {
        let library = FfiSubtitleLibrary::new();
        assert_eq!(
            library.add_file("/nen-025/definitely/not/here.srt".into()),
            FfiSubtitleOutcome::Rejected {
                reason: FfiFileRejection::NotRegularFile
            }
        );
        assert_eq!(library.source_count(), 0);
    }

    #[test]
    fn a_traversing_path_is_refused_at_the_gate() {
        let library = FfiSubtitleLibrary::new();
        assert_eq!(
            library.add_file("/tmp/../etc/passwd.srt".into()),
            FfiSubtitleOutcome::Rejected {
                reason: FfiFileRejection::Traversal
            }
        );
    }

    #[test]
    fn a_medium_with_nothing_beside_it_answers_nothing() {
        let library = FfiSubtitleLibrary::new();
        assert_eq!(
            library.add_sidecar_for("/nen-025/definitely/not/here.mkv".into()),
            None
        );
    }

    fn subtitle_track(id: u32, language: Option<&str>, title: Option<&str>) -> FfiTrackDescriptor {
        FfiTrackDescriptor {
            id,
            kind: crate::playback::FfiTrackKind::Subtitle,
            language: language.map(ToOwned::to_owned),
            codec: "subrip".to_owned(),
            is_default: false,
            title: title.map(ToOwned::to_owned),
        }
    }

    fn headings(sections: &[FfiMenuSection]) -> Vec<FfiMenuGroup> {
        sections.iter().map(|s| s.group.clone()).collect()
    }

    #[test]
    fn a_fresh_library_offers_closed_and_nothing_else() {
        let library = FfiSubtitleLibrary::new();
        assert_eq!(headings(&library.menu(None, None)), [FfiMenuGroup::Closed]);
    }

    #[test]
    fn the_container_language_crosses_canonicalised() {
        // ADR-0032: Matroska writes ISO 639-2, the core answers `en`. The
        // shell has to group by the tag the core decided, not the raw one.
        let library = FfiSubtitleLibrary::new();
        library.add_embedded(vec![subtitle_track(2, Some("eng"), Some("English"))]);

        assert_eq!(
            headings(&library.menu(None, None)),
            [
                FfiMenuGroup::Closed,
                FfiMenuGroup::Language {
                    tag: "en".to_owned()
                }
            ]
        );
    }

    #[test]
    fn a_row_resolves_back_to_the_track_it_came_from() {
        let library = FfiSubtitleLibrary::new();
        library.add_embedded(vec![
            subtitle_track(2, Some("en"), Some("English")),
            subtitle_track(3, Some("tr"), None),
        ]);

        let menu = library.menu(None, None);
        let turkish = &menu
            .iter()
            .find(|s| {
                s.group
                    == FfiMenuGroup::Language {
                        tag: "tr".to_owned(),
                    }
            })
            .expect("Turkish heading")
            .entries[0];

        assert_eq!(library.embedded_track_of(turkish.token), Some(3));
        assert!(library.is_usable(turkish.token));
        assert_eq!(turkish.label, "", "the shell writes the endonym");
    }

    #[test]
    fn a_preference_the_core_cannot_parse_is_simply_no_preference() {
        // The shell seeds these from the system locale. A tag it cannot read
        // must leave the menu in its default order, never raise an error at a
        // user who only opened a file.
        let library = FfiSubtitleLibrary::new();
        library.add_embedded(vec![
            subtitle_track(2, Some("en"), None),
            subtitle_track(3, Some("tr"), None),
        ]);

        let nonsense = library.menu(Some("not a tag".to_owned()), None);
        assert_eq!(headings(&nonsense), headings(&library.menu(None, None)));
        assert_eq!(
            library.auto_selection(Some("not a tag".to_owned()), None),
            None
        );
    }

    #[test]
    fn a_regional_preference_still_hoists_its_language() {
        // `Locale.preferredLanguages` hands out `tr-TR`; the tracks say `tr`.
        let library = FfiSubtitleLibrary::new();
        library.add_embedded(vec![
            subtitle_track(2, Some("en"), None),
            subtitle_track(3, Some("tr"), None),
        ]);

        assert_eq!(
            headings(&library.menu(Some("tr-TR".to_owned()), None))[1],
            FfiMenuGroup::Language {
                tag: "tr".to_owned()
            }
        );
        assert_eq!(
            library.auto_selection(Some("tr-TR".to_owned()), None),
            Some(2)
        );
    }

    #[test]
    fn an_unknown_token_is_neither_usable_nor_a_track() {
        let library = FfiSubtitleLibrary::new();
        library.add_embedded(vec![subtitle_track(2, Some("en"), None)]);
        assert!(!library.is_usable(0));
        assert!(!library.is_usable(9_999));
        assert_eq!(library.embedded_track_of(9_999), None);
    }

    #[test]
    fn a_menu_row_prints_without_its_label() {
        let entry = FfiMenuEntry {
            token: 4,
            kind: FfiSubtitleSourceKind::User,
            language: Some("tr".to_owned()),
            label: "Inception.2010.tr.srt".to_owned(),
            defect: Some(FfiSourceDefect::Malformed),
            translatable: true,
        };
        let printed = format!("{entry:?}");
        assert!(!printed.contains("Inception"), "{printed}");
        assert!(!printed.contains(".srt"), "{printed}");
        assert!(printed.contains("Malformed"), "{printed}");
    }

    #[test]
    fn an_outcome_prints_its_reason_and_could_not_print_a_path() {
        // The enums carry no path to print. This is the guard for that claim:
        // it fails the day a variant grows a `String` field.
        let printed = format!(
            "{:?}",
            FfiSubtitleOutcome::Rejected {
                reason: FfiFileRejection::Symlink
            }
        );
        assert!(printed.contains("Symlink"), "{printed}");
        assert!(!printed.contains('/'), "{printed}");
    }
}
