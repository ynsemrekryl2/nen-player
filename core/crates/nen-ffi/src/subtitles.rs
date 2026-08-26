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

use nen_app::subtitle_files::{FileRejection, SourceDefect};
use nen_app::subtitles::{AddOutcome, SubtitleLibrary};
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

    /// How many sources the catalog holds.
    ///
    /// The menu projection itself is NEN-026's; this is the one number that
    /// lets the shell prove the wiring end to end — that loading one file twice
    /// leaves one entry — without NEN-026's types existing yet.
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
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
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
