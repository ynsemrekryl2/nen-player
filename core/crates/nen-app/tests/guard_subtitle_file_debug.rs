//! K23 guard for the file-loading types (NEN-025).
//!
//! Two things must never reach a log from this path: the private full path of
//! a subtitle file (K23 #3, `security-policy.md` §1 #3) and the dialogue inside
//! it (K23 #4). Every type NEN-025 introduces either holds one of those or
//! stands next to something that does, so each one is printed here and searched.
//!
//! The derived twins are the point. A `Debug` written by hand is only a
//! promise; the twins show what the derive would have printed instead, so this
//! file fails the day someone replaces an impl with `#[derive(Debug)]`.

use nen_app::subtitle_files::{FileRejection, SourceDefect};
use nen_app::subtitles::SubtitleLibrary;
use std::fs;
use std::path::{Path, PathBuf};

const PRIVATE_DIR: &str = "Sevgilimle.Tatil.2019";
const PRIVATE_FILENAME: &str = "Sevgilimle.Tatil.2019.tr.srt";
const PRIVATE_DIALOGUE: &str = "Bunu kimseye söyleme.";

fn forbidden(output: &str) {
    for value in [PRIVATE_DIR, PRIVATE_FILENAME, PRIVATE_DIALOGUE] {
        assert!(
            !output.contains(value),
            "Debug output leaked a forbidden value ({value}): {output}"
        );
    }
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "nen-025-guard-{}-{PRIVATE_DIR}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temp dir");
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

#[test]
fn a_rejection_never_prints_a_path() {
    let output = format!(
        "{:?} {:?} {:?} {:?}",
        FileRejection::Traversal,
        FileRejection::Symlink,
        FileRejection::NotRegularFile,
        FileRejection::TooLarge
    );
    forbidden(&output);
    assert!(output.contains("symlink"), "{output}");
}

#[test]
fn a_defect_never_prints_the_line_that_caused_it() {
    let output = format!(
        "{:?} {:?}",
        SourceDefect::Unreadable,
        SourceDefect::Malformed
    );
    forbidden(&output);
    assert!(output.contains("malformed"), "{output}");
}

#[test]
fn the_library_prints_counts_and_never_a_filename_or_a_cue() {
    let dir = TempDir::new();
    let path = dir.path().join(PRIVATE_FILENAME);
    fs::write(
        &path,
        format!("1\n00:00:01,000 --> 00:00:02,000\n{PRIVATE_DIALOGUE}\n"),
    )
    .expect("write");

    let mut library = SubtitleLibrary::new();
    library.add_file(&path, dir.path());

    let output = format!("{library:?}");
    forbidden(&output);
    assert!(output.contains("source_count"), "{output}");
    assert!(output.contains("document_count"), "{output}");
}

#[test]
fn a_derived_debug_on_these_types_would_leak() {
    // The control. If this twin stops leaking, the guard above has stopped
    // proving anything and the fixtures need to be made private again.
    #[derive(Debug)]
    #[allow(dead_code)]
    struct DerivedRejectionTwin {
        reason: &'static str,
        path: PathBuf,
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    struct DerivedLibraryTwin {
        label: String,
        cue: String,
    }

    let rejection = DerivedRejectionTwin {
        reason: "symlink",
        path: PathBuf::from(format!("/Users/x/{PRIVATE_DIR}/{PRIVATE_FILENAME}")),
    };
    let library = DerivedLibraryTwin {
        label: PRIVATE_FILENAME.to_owned(),
        cue: PRIVATE_DIALOGUE.to_owned(),
    };

    let printed = format!("{rejection:?} {library:?}");
    assert!(printed.contains(PRIVATE_FILENAME), "{printed}");
    assert!(printed.contains(PRIVATE_DIALOGUE), "{printed}");
    assert!(printed.contains(PRIVATE_DIR), "{printed}");
}
