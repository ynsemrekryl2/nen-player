//! NEN-021 DoD #3: nothing above the adapter branches on a playback engine's
//! **name**.
//!
//! Product-spec §4 and `docs/architecture.md` state the rule as code:
//!
//! ```text
//! // YANLIŞ
//! if engine.name == "mpv" { ... }
//! // DOĞRU
//! if engine.capabilities.contains(.externalSubtitleInjection) { ... }
//! ```
//!
//! The evidence the task asks for is a grep. Running it as a test instead of
//! once by hand is what makes it hold: every future commit is checked, not just
//! this one.
//!
//! Comments are excluded on purpose — this file, ADR-0011 and the port's own
//! documentation all name engines while discussing them, and that is not
//! branching. What is forbidden is an engine's name reaching *code*.

use std::fs;
use std::path::{Path, PathBuf};

/// Engine names and product names that must not appear in code.
const ENGINE_NAMES: [&str; 8] = [
    "mpv",
    "avplayer",
    "avkit",
    "media3",
    "exoplayer",
    "vlc",
    "gstreamer",
    "avfoundation",
];

/// Strips line comments so that prose may name engines freely.
///
/// Deliberately simple: this repo uses `//` and `//!` and no block comments in
/// the crates being scanned. A `//` inside a string literal would be stripped
/// too — that direction is safe, because it can only make the scan *miss*
/// something, never invent a violation, and the synthetic control below proves
/// the scan still catches real code.
fn code_only(line: &str) -> &str {
    match line.find("//") {
        Some(index) => &line[..index],
        None => line,
    }
}

fn violations_in(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (number, line) in source.lines().enumerate() {
        let code = code_only(line).to_ascii_lowercase();
        for name in ENGINE_NAMES {
            if code.contains(name) {
                found.push(format!("line {}: {}", number + 1, line.trim()));
                break;
            }
        }
    }
    found
}

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(rust_files(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    files
}

fn crate_src(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/")
        .join(name)
        .join("src")
}

#[test]
fn no_engine_name_appears_in_port_or_application_code() {
    let mut reported = Vec::new();
    let mut scanned = 0usize;

    for crate_name in ["nen-ports", "nen-app"] {
        let root = crate_src(crate_name);
        assert!(root.is_dir(), "{} not found at {root:?}", crate_name);
        for file in rust_files(&root) {
            scanned += 1;
            let source = fs::read_to_string(&file).expect("readable source");
            for violation in violations_in(&source) {
                reported.push(format!("{}: {violation}", file.display()));
            }
        }
    }

    assert!(scanned > 0, "the scan found no files to read");
    assert!(
        reported.is_empty(),
        "an engine name reached code — capability, not identity (§4):\n{}",
        reported.join("\n")
    );
}

#[test]
fn the_scan_catches_a_real_violation() {
    // Negative control: without this, "no violations found" could just mean
    // the scan does not work.
    let offending = r#"
        fn choose(engine: &Engine) -> bool {
            if engine.name() == "mpv" {
                return true;
            }
            false
        }
    "#;
    let found = violations_in(offending);
    assert_eq!(found.len(), 1, "expected exactly one hit, got {found:?}");
    assert!(found[0].contains("engine.name()"));
}

#[test]
fn the_scan_ignores_prose_that_names_engines() {
    // The port's own documentation discusses libmpv and AVPlayer constantly.
    let documented = r#"
        /// Implemented by libmpv on macOS and AVPlayer on iOS.
        // mpv is the first engine; Media3 comes with M10.
        pub trait PlaybackEngine {}
    "#;
    assert!(violations_in(documented).is_empty());
}

#[test]
fn a_trailing_comment_does_not_hide_code_before_it() {
    // Stripping comments must not become a way to smuggle a branch in.
    let sneaky = r#"if engine.name() == "vlc" { } // just a note"#;
    assert_eq!(violations_in(sneaky).len(), 1);
}
