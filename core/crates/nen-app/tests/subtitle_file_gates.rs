//! `security-policy.md` §4 and ADR-0031 Karar 5, against a real filesystem
//! (NEN-025).
//!
//! Every refusal here is built out of the real thing — a real symlink, a real
//! FIFO, a real oversized file — because that is the only way the assertion
//! means anything. A fake filesystem asked "is this a symlink?" would answer
//! whatever the test told it to answer, and the test would then prove that the
//! test is consistent with itself. Nothing in this file needs a network, a
//! credential or a fixed clock, so it stays deterministic all the same.

use nen_app::subtitle_files::{FileRejection, SourceDefect, MAX_SUBTITLE_BYTES};
use nen_app::subtitles::{AddOutcome, SubtitleLibrary};
use nen_domain::source::SubtitleSourceKind;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

const VALID_SRT: &str = "1\n00:00:01,000 --> 00:00:02,000\nHello there.\n\n\
                         2\n00:00:03,000 --> 00:00:04,500\nGeneral Kenobi.\n";

/// A directory that removes itself, so a failing test cannot leave a FIFO
/// behind in the temp directory.
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        // Process id plus a counter: two tests running in parallel, and two
        // runs of the same test, must not collide.
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("nen-025-{tag}-{}-{unique}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temp dir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, contents).expect("write fixture");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

// --- the happy path ---------------------------------------------------------

#[test]
fn a_sidecar_beside_the_medium_is_found_and_catalogued_as_a_user_source() {
    let dir = TempDir::new("sidecar");
    let media = dir.write("Inception.2010.mkv", "not really a video");
    dir.write("Inception.2010.srt", VALID_SRT);

    let mut library = SubtitleLibrary::new();
    assert_eq!(library.add_sidecar_of(&media), Some(AddOutcome::Added));

    let sources: Vec<_> = library
        .catalog()
        .of_kind(SubtitleSourceKind::User)
        .collect();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].label(), "Inception.2010.srt");
    assert!(library.is_usable(sources[0].id()));
    assert!(library.document(sources[0].id()).is_some());
}

#[test]
fn a_medium_with_no_sidecar_beside_it_is_not_a_refusal() {
    let dir = TempDir::new("no-sidecar");
    let media = dir.write("Alone.mkv", "not really a video");

    let mut library = SubtitleLibrary::new();
    assert_eq!(library.add_sidecar_of(&media), None);
    assert_eq!(library.catalog().len(), 0);
}

#[test]
fn a_sidecar_lands_in_the_language_group_its_text_belongs_to() {
    // NEN-020's detector, applied at the one moment the document exists. A
    // sidecar that always landed in `Dil Belirsiz` would make NEN-026's
    // grouping untestable.
    let dir = TempDir::new("language");
    let media = dir.write("Film.mkv", "not really a video");
    dir.write(
        "Film.srt",
        "1\n00:00:01,000 --> 00:00:04,000\n\
         Bugün hava çok güzel ve biz sahilde uzun bir yürüyüş yaptık.\n\n\
         2\n00:00:05,000 --> 00:00:09,000\n\
         Akşam olduğunda eve dönüp birlikte yemek hazırlamaya karar verdik.\n",
    );

    let mut library = SubtitleLibrary::new();
    assert_eq!(library.add_sidecar_of(&media), Some(AddOutcome::Added));

    let source = library
        .catalog()
        .of_kind(SubtitleSourceKind::User)
        .next()
        .expect("one user source");
    assert_eq!(source.language().map(|tag| tag.as_str()), Some("tr"));
}

// --- §4 #2: symlink ---------------------------------------------------------

#[test]
#[cfg(unix)]
fn a_symlinked_subtitle_is_refused_and_never_followed() {
    let dir = TempDir::new("symlink");
    let real = dir.write("real.srt", VALID_SRT);
    let link = dir.path().join("Film.srt");
    std::os::unix::fs::symlink(&real, &link).expect("symlink");

    let mut library = SubtitleLibrary::new();
    assert_eq!(
        library.add_file(&link, dir.path()),
        AddOutcome::Rejected(FileRejection::Symlink)
    );
    // Following it would have produced a perfectly good document — the target
    // is valid SRT. Nothing was catalogued, which is what proves it was not
    // followed rather than merely not enjoyed.
    assert_eq!(library.catalog().len(), 0);
}

// --- §4 #3: traversal -------------------------------------------------------

#[test]
fn a_path_containing_a_parent_component_is_refused() {
    let dir = TempDir::new("traversal");
    let nested = dir.path().join("season");
    fs::create_dir_all(&nested).expect("nested dir");
    fs::write(nested.join("Film.srt"), VALID_SRT).expect("write");

    // Spelled through a `..`, but resolving to a file that really is inside the
    // root: refused anyway, because §4 #3 is about the spelling reaching the
    // filesystem at all.
    let traversing = nested.join("..").join("season").join("Film.srt");

    let mut library = SubtitleLibrary::new();
    assert_eq!(
        library.add_file(&traversing, dir.path()),
        AddOutcome::Rejected(FileRejection::Traversal)
    );
    assert_eq!(library.catalog().len(), 0);
}

#[test]
fn a_file_outside_the_root_is_refused_even_when_spelled_plainly() {
    let inside = TempDir::new("root-inside");
    let outside = TempDir::new("root-outside");
    let stranger = outside.write("Film.srt", VALID_SRT);

    let mut library = SubtitleLibrary::new();
    assert_eq!(
        library.add_file(&stranger, inside.path()),
        AddOutcome::Rejected(FileRejection::Traversal)
    );
}

#[test]
#[cfg(unix)]
fn a_directory_symlink_cannot_smuggle_a_file_in_from_outside_the_root() {
    // The final component is an honest regular file, so the symlink check on it
    // passes; the escape is one level up. This is what canonicalizing the
    // parent is for.
    let root = TempDir::new("ancestor-root");
    let elsewhere = TempDir::new("ancestor-elsewhere");
    elsewhere.write("Film.srt", VALID_SRT);

    let door = root.path().join("door");
    std::os::unix::fs::symlink(elsewhere.path(), &door).expect("dir symlink");

    let mut library = SubtitleLibrary::new();
    assert_eq!(
        library.add_file(&door.join("Film.srt"), root.path()),
        AddOutcome::Rejected(FileRejection::Traversal)
    );
    assert_eq!(library.catalog().len(), 0);
}

// --- §4 #1: not a regular file ---------------------------------------------

#[test]
fn a_directory_is_refused() {
    let dir = TempDir::new("directory");
    let masquerading = dir.path().join("Film.srt");
    fs::create_dir_all(&masquerading).expect("dir named like a subtitle");

    let mut library = SubtitleLibrary::new();
    assert_eq!(
        library.add_file(&masquerading, dir.path()),
        AddOutcome::Rejected(FileRejection::NotRegularFile)
    );
}

#[test]
#[cfg(unix)]
fn a_fifo_is_refused_rather_than_read() {
    let dir = TempDir::new("fifo");
    let fifo = dir.path().join("Film.srt");
    let made = std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("run mkfifo");
    assert!(made.success(), "mkfifo failed");

    // The point of the gate: opening this would block forever waiting for a
    // writer. The assertion returning at all is half the proof.
    let mut library = SubtitleLibrary::new();
    assert_eq!(
        library.add_file(&fifo, dir.path()),
        AddOutcome::Rejected(FileRejection::NotRegularFile)
    );
    assert_eq!(library.catalog().len(), 0);
}

#[test]
fn a_path_that_is_not_there_is_refused_rather_than_catalogued_as_broken() {
    let dir = TempDir::new("missing");

    let mut library = SubtitleLibrary::new();
    assert_eq!(
        library.add_file(&dir.path().join("Ghost.srt"), dir.path()),
        AddOutcome::Rejected(FileRejection::NotRegularFile)
    );
    assert_eq!(library.catalog().len(), 0);
}

// --- §4 #4: size ------------------------------------------------------------

#[test]
fn a_file_over_the_size_limit_is_refused() {
    let dir = TempDir::new("too-large");
    let big = dir.path().join("Film.srt");
    let file = File::create(&big).expect("create");
    // Sparse: the gate reads metadata, not bytes, so the test need not write
    // 16 MiB to find out.
    file.set_len(MAX_SUBTITLE_BYTES + 1).expect("set_len");
    drop(file);

    let mut library = SubtitleLibrary::new();
    assert_eq!(
        library.add_file(&big, dir.path()),
        AddOutcome::Rejected(FileRejection::TooLarge)
    );
    assert_eq!(library.catalog().len(), 0);
}

#[test]
#[cfg(unix)]
fn an_oversized_file_is_refused_without_being_opened() {
    // "boyut sınırını aşan dosya **okunmuyor**" — proved, not asserted. The
    // file is both oversized and unreadable, so a gate that opened it first
    // would have to report `Unreadable`. Reporting `TooLarge` is only possible
    // if nothing was opened.
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new("unopened");
    let big = dir.path().join("Film.srt");
    let file = File::create(&big).expect("create");
    file.set_len(MAX_SUBTITLE_BYTES + 1).expect("set_len");
    drop(file);
    fs::set_permissions(&big, fs::Permissions::from_mode(0o000)).expect("chmod");

    let mut library = SubtitleLibrary::new();
    let outcome = library.add_file(&big, dir.path());

    // Restore before the assertion so a failure still cleans up.
    let _ = fs::set_permissions(&big, fs::Permissions::from_mode(0o600));
    assert_eq!(outcome, AddOutcome::Rejected(FileRejection::TooLarge));
}

#[test]
fn a_file_exactly_at_the_size_limit_is_still_admitted() {
    // The gate is "aşan" — exceeding. An off-by-one here would refuse a legal
    // file, and no negative test would ever notice.
    let dir = TempDir::new("at-limit");
    let mut padded = String::from(VALID_SRT);
    padded.push_str(&"\n".repeat(MAX_SUBTITLE_BYTES as usize - VALID_SRT.len()));
    let path = dir.write("Film.srt", &padded);
    assert_eq!(fs::metadata(&path).expect("stat").len(), MAX_SUBTITLE_BYTES);

    let mut library = SubtitleLibrary::new();
    assert_eq!(library.add_file(&path, dir.path()), AddOutcome::Added);
}

// --- ADR-0031 Karar 5: refused is not the same as broken --------------------

#[test]
fn a_malformed_subtitle_is_catalogued_and_marked_rather_than_dropped() {
    let dir = TempDir::new("malformed");
    let path = dir.write("Film.srt", "1\nthis is not a timecode\nsome text\n");

    let mut library = SubtitleLibrary::new();
    assert_eq!(
        library.add_file(&path, dir.path()),
        AddOutcome::Defective(SourceDefect::Malformed)
    );

    let source = library
        .catalog()
        .of_kind(SubtitleSourceKind::User)
        .next()
        .expect("the broken source is still an entry");
    assert_eq!(library.defect(source.id()), Some(SourceDefect::Malformed));
    assert!(!library.is_usable(source.id()));
    assert!(library.document(source.id()).is_none());
}

#[test]
fn a_malformed_subtitle_leaves_every_other_source_alone() {
    // The M3 exit criterion is "bozuk bir .srt playback'i durdurmuyor". The
    // core half of that is this: the failure is confined to its own entry and
    // is returned, never raised.
    let dir = TempDir::new("confined");
    let good = dir.write("Good.srt", VALID_SRT);
    let bad = dir.write("Bad.srt", "nonsense");

    let mut library = SubtitleLibrary::new();
    assert_eq!(library.add_file(&good, dir.path()), AddOutcome::Added);
    assert_eq!(
        library.add_file(&bad, dir.path()),
        AddOutcome::Defective(SourceDefect::Malformed)
    );

    assert_eq!(library.catalog().len(), 2);
    let good_source = library
        .catalog()
        .sources()
        .find(|s| s.label() == "Good.srt")
        .expect("the good one survived");
    assert!(library.is_usable(good_source.id()));
    assert!(library.document(good_source.id()).is_some());
}

#[test]
#[cfg(unix)]
fn every_gate_refusal_leaves_the_catalog_completely_empty() {
    // ADR-0031 Karar 5: a refused file is not listed, not even as a broken
    // entry. Asserted once per gate so a future gate that starts catalogueing
    // its refusals cannot slip through on the strength of the others.
    let dir = TempDir::new("no-trace");

    let symlink = dir.path().join("Link.srt");
    std::os::unix::fs::symlink(dir.write("target.srt", VALID_SRT), &symlink).expect("symlink");

    let directory = dir.path().join("Dir.srt");
    fs::create_dir_all(&directory).expect("dir");

    let big = dir.path().join("Big.srt");
    let file = File::create(&big).expect("create");
    file.set_len(MAX_SUBTITLE_BYTES + 1).expect("set_len");
    drop(file);

    let traversing = dir.path().join("..").join("Anything.srt");

    for path in [symlink, directory, big, traversing] {
        let mut library = SubtitleLibrary::new();
        let outcome = library.add_file(&path, dir.path());
        assert!(
            matches!(outcome, AddOutcome::Rejected(_)),
            "{outcome:?} was not a rejection"
        );
        assert_eq!(library.catalog().len(), 0, "{outcome:?} left a trace");
        assert_eq!(library.catalog().sources().count(), 0);
    }
}

// --- ADR-0010 Karar 2: identity is the path digest --------------------------

#[test]
fn loading_the_same_file_twice_leaves_one_catalog_entry() {
    let dir = TempDir::new("twice");
    let path = dir.write("Film.srt", VALID_SRT);

    let mut library = SubtitleLibrary::new();
    assert_eq!(library.add_file(&path, dir.path()), AddOutcome::Added);
    assert_eq!(library.add_file(&path, dir.path()), AddOutcome::Added);
    assert_eq!(library.catalog().len(), 1);
}

#[test]
fn two_spellings_of_one_file_are_one_entry() {
    // The digest is taken over the canonical path, so `dir/./Film.srt` and
    // `dir/Film.srt` are the same source. Digesting the path as supplied would
    // have produced two rows for one file.
    let dir = TempDir::new("spellings");
    let path = dir.write("Film.srt", VALID_SRT);
    let curdir_spelling = dir.path().join(".").join("Film.srt");

    let mut library = SubtitleLibrary::new();
    assert_eq!(library.add_file(&path, dir.path()), AddOutcome::Added);
    assert_eq!(
        library.add_file(&curdir_spelling, dir.path()),
        AddOutcome::Added
    );
    assert_eq!(library.catalog().len(), 1);
}

#[test]
fn a_sidecar_and_the_same_file_loaded_by_hand_are_one_entry() {
    let dir = TempDir::new("both-ways");
    let media = dir.write("Film.mkv", "not really a video");
    let sidecar = dir.write("Film.srt", VALID_SRT);

    let mut library = SubtitleLibrary::new();
    assert_eq!(library.add_sidecar_of(&media), Some(AddOutcome::Added));
    assert_eq!(library.add_file(&sidecar, dir.path()), AddOutcome::Added);
    assert_eq!(library.catalog().len(), 1);
}

#[test]
fn two_different_files_are_two_entries() {
    // The other half of the dedup claim: a digest that collapsed everything
    // would pass the test above and be useless.
    let dir = TempDir::new("distinct");
    let first = dir.write("First.srt", VALID_SRT);
    let second = dir.write("Second.srt", VALID_SRT);

    let mut library = SubtitleLibrary::new();
    library.add_file(&first, dir.path());
    library.add_file(&second, dir.path());
    assert_eq!(library.catalog().len(), 2);
}

#[test]
fn reloading_a_repaired_file_clears_the_mark_it_used_to_carry() {
    let dir = TempDir::new("repaired");
    let path = dir.write("Film.srt", "nonsense");

    let mut library = SubtitleLibrary::new();
    assert_eq!(
        library.add_file(&path, dir.path()),
        AddOutcome::Defective(SourceDefect::Malformed)
    );

    fs::write(&path, VALID_SRT).expect("repair");
    assert_eq!(library.add_file(&path, dir.path()), AddOutcome::Added);

    let source = library
        .catalog()
        .of_kind(SubtitleSourceKind::User)
        .next()
        .expect("still one entry");
    assert_eq!(library.catalog().len(), 1);
    assert_eq!(library.defect(source.id()), None);
    assert!(library.is_usable(source.id()));
}
