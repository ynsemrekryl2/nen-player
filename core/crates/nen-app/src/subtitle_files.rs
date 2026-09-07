//! Turning a path on disk into a catalog entry, with `security-policy.md` §4's
//! gates in front of every read (NEN-025).
//!
//! # Why the gates live here and not behind a port
//!
//! ADR-0006 rule 1 puts I/O behind `nen-ports` — but that rule binds
//! `nen-domain`, whose whole point is that it cannot reach a disk. This crate
//! is the use-case layer and is allowed every crate below it, so the question
//! is not "may it" but "should it". It should, and the reason is the evidence:
//! §4's gates are claims about **filesystem facts** — this is a symlink, that
//! is a FIFO, this path escapes its directory. A fake port could only ever
//! restate those claims, so a test written against one would prove that the
//! fake said "symlink", not that a symlink is refused. Real `std::fs` here lets
//! `tests/subtitle_file_gates.rs` build the real thing in a temporary directory
//! and watch it be refused. Nothing about that needs a credential or a network,
//! so testing-strategy's deterministic-fake rule is untouched.
//!
//! # The two outcomes are not the same outcome (ADR-0031 Karar 5)
//!
//! A file that fails a **gate** never becomes a source: it is not catalogued,
//! not shown, not remembered — [`FileRejection`]. A file that passes the gates
//! and then turns out to be undecodable or unparsable **is** a source, and a
//! broken one: it is catalogued and marked — [`SourceDefect`]. Collapsing the
//! two would either list a refused path in the menu (confirming to anyone
//! looking that it exists) or silently drop a sidecar the user expected to see.
//!
//! # Security
//!
//! The raw path never leaves this module. [`SubtitleSourceId::user`] takes a
//! digest precisely so that K23 #3 holds by construction, and both error types
//! here implement `Debug` by hand so a rejection cannot carry a path into a log
//! either. `tests/guard_subtitle_file_debug.rs` proves it with a derived twin.

use nen_domain::source::{SubtitleSource, SubtitleSourceId};
use nen_domain::subtitle::SubtitleDocument;
use nen_subtitle::{encoding, srt};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// The largest subtitle file that is read at all (`security-policy.md` §4 #4).
///
/// **This is [`encoding::MAX_INPUT_BYTES`], not a second opinion about it.**
/// NEN-015 already set the number the decoder refuses to exceed; a gate with a
/// larger limit of its own would let a file through, read all of it into
/// memory, and only then be told no — which is precisely the outcome §4 #4
/// exists to prevent ("boyut sınırını aşan dosya **okunmaz**"). A smaller one
/// would make the decoder's limit unreachable and untestable.
///
/// What the two do differ in is *when*. This one is asked of `stat`, before
/// anything is opened. The decoder's is a backstop for bytes that arrive
/// without passing a filesystem gate at all — an OpenSubtitles download in M6.
pub const MAX_SUBTITLE_BYTES: u64 = encoding::MAX_INPUT_BYTES as u64;

/// Why a file never became a source at all (`security-policy.md` §4 #1–#4).
///
/// Every variant means the same thing to the catalog: nothing was added. They
/// differ only in what the shell may say about it, and even then only when the
/// user picked the file themselves (ADR-0031 Karar 5).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FileRejection {
    /// The path contains a `..` component, or resolves outside its root.
    Traversal,
    /// The path is a symlink. It is refused, not followed (§4 #2).
    Symlink,
    /// A directory, FIFO, socket, device — or nothing at all (§4 #1).
    NotRegularFile,
    /// Larger than [`MAX_SUBTITLE_BYTES`]; not read (§4 #4).
    TooLarge,
}

impl FileRejection {
    /// Stable lowercase name. Carries no path, so it is safe to log.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Traversal => "traversal",
            Self::Symlink => "symlink",
            Self::NotRegularFile => "not-regular-file",
            Self::TooLarge => "too-large",
        }
    }
}

impl fmt::Debug for FileRejection {
    /// Prints the reason and never the path.
    ///
    /// Hand-written rather than derived because a future variant carrying the
    /// offending path would otherwise leak it into every log line that prints
    /// a rejection (K23 #3). Keeping the impl manual makes adding such a field
    /// a decision instead of an accident.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileRejection")
            .field("reason", &self.as_str())
            .finish()
    }
}

/// Why a source that *is* in the catalog cannot be used (ADR-0031 Karar 5).
///
/// The two variants are the closed set behind the menu's two reason labels —
/// `okunamadı` and `biçim hatalı`. The label text itself belongs to NEN-026;
/// what is decided here is only which of the two a failure is.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SourceDefect {
    /// The bytes could not be read, or could not be decoded to text (ADR-0008).
    Unreadable,
    /// The text is not valid SRT (NEN-013's strict parser).
    Malformed,
}

impl SourceDefect {
    /// Stable lowercase name. Carries no path or dialogue, so it may be logged.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unreadable => "unreadable",
            Self::Malformed => "malformed",
        }
    }
}

impl fmt::Debug for SourceDefect {
    /// Prints the reason only.
    ///
    /// A derived `Debug` on a variant that one day carries the underlying
    /// [`srt::SrtError`] would print the offending line — that is subtitle
    /// dialogue, K23 #4.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceDefect")
            .field("reason", &self.as_str())
            .finish()
    }
}

/// What became of one path.
///
/// [`Rejected`](Self::Rejected) carries no source on purpose: there is nothing
/// to carry. The other two both carry one, because a broken sidecar is still an
/// entry the user is entitled to see (ADR-0031 Karar 5).
pub enum LoadedFile {
    /// Gates passed, decoded and parsed.
    Loaded {
        source: SubtitleSource,
        document: SubtitleDocument,
    },
    /// Gates passed; the content did not.
    Defective {
        source: SubtitleSource,
        defect: SourceDefect,
    },
    /// A gate refused it. Nothing was read.
    Rejected(FileRejection),
}

/// A path that passed every gate in §4, together with what the gates learned.
///
/// Separate from the reading step so the gates can be tested for what they
/// are — a decision taken **before** opening anything.
struct AdmittedFile {
    /// The path with every symlinked ancestor resolved. This, not the path as
    /// supplied, is what the identity digest is taken over: two spellings of
    /// one file must dedup to one catalog entry (ADR-0010 Karar 2).
    canonical: PathBuf,
    /// What the user sees next to the entry (§8). Displayable, not loggable.
    file_name: String,
}

/// Applies `security-policy.md` §4 #1–#4 to `path`, before anything is opened.
///
/// `root` is the directory the file must stay inside. For a sidecar that is the
/// medium's own directory; for a file the user picked from the open panel it is
/// that file's directory, which makes #3 a check that the *spelling* is honest
/// rather than a check on where the user is allowed to look — the panel already
/// answered that question, and second-guessing it would refuse legitimate files.
///
/// The order differs from the policy's numbering by design: the `..` test is
/// lexical and therefore runs **before** any syscall touches the untrusted
/// path. Everything else follows §4's order.
fn admit(path: &Path, root: &Path) -> Result<AdmittedFile, FileRejection> {
    // §4 #3, lexical half. Done first because it needs no syscall: a path that
    // is already trying to escape should not be handed to the filesystem at
    // all, not even to be stat'ed.
    if path.components().any(|c| c == Component::ParentDir) {
        return Err(FileRejection::Traversal);
    }

    // §4 #1 and #2 in one call. `symlink_metadata` is the one that does not
    // follow: `metadata` would resolve the link and then report on the target,
    // which is exactly the answer §4 #2 refuses to accept. A path that does not
    // exist lands here too — it is not a regular file either, and the caller
    // has nothing to catalogue.
    let metadata = fs::symlink_metadata(path).map_err(|_| FileRejection::NotRegularFile)?;
    if metadata.file_type().is_symlink() {
        return Err(FileRejection::Symlink);
    }
    if !metadata.file_type().is_file() {
        return Err(FileRejection::NotRegularFile);
    }

    // §4 #3, resolved half. Only the parent is canonicalized: the final
    // component was just proven not to be a symlink, so resolving it would add
    // nothing, while canonicalizing the parent is what catches an ancestor
    // symlink that points out of the root.
    let file_name = path
        .file_name()
        .ok_or(FileRejection::NotRegularFile)?
        .to_string_lossy()
        .into_owned();
    let parent = path.parent().ok_or(FileRejection::Traversal)?;
    let canonical_parent = fs::canonicalize(parent).map_err(|_| FileRejection::Traversal)?;
    let canonical_root = fs::canonicalize(root).map_err(|_| FileRejection::Traversal)?;
    if !canonical_parent.starts_with(&canonical_root) {
        return Err(FileRejection::Traversal);
    }

    // §4 #4. Asked of the metadata already in hand, so an oversized file is
    // refused without ever being opened.
    if metadata.len() > MAX_SUBTITLE_BYTES {
        return Err(FileRejection::TooLarge);
    }

    Ok(AdmittedFile {
        canonical: canonical_parent.join(&file_name),
        file_name,
    })
}

/// The identity of a user file: a digest of its canonical path.
///
/// ADR-0010 Karar 2 picks the path over the content because the catalog is
/// built before anything is read (§7), and picks a **digest** over the path
/// because K23 #3 forbids the path from travelling. Both properties are needed
/// and this is the one place that has the path to hash.
///
/// `as_encoded_bytes` rather than a lossy string: a digest that changed when a
/// filename happened to be invalid UTF-8 would silently stop deduplicating.
fn path_digest(canonical: &Path) -> [u8; 32] {
    *blake3::hash(canonical.as_os_str().as_encoded_bytes()).as_bytes()
}

/// Runs the gates, then reads, decodes and parses.
///
/// `root` is as in [`admit`]. When the filename declares a language as a
/// trailing sub-extension (`Film.tr.srt`, NEN-057) that is the metadata
/// `resolve_language` treats as authoritative (ADR-0029 Karar 5); the parsed
/// document is still examined so a reliable disagreement can be reported. A
/// sidecar with no filename hint falls back to text detection alone
/// (NEN-020), and one with neither lands in `Dil Belirsiz`.
pub fn load(path: &Path, root: &Path) -> LoadedFile {
    let admitted = match admit(path, root) {
        Ok(admitted) => admitted,
        Err(rejection) => return LoadedFile::Rejected(rejection),
    };

    let id = SubtitleSourceId::user(path_digest(&admitted.canonical));
    let defective = |defect| LoadedFile::Defective {
        source: SubtitleSource::new(id.clone(), None, admitted.file_name.clone()),
        defect,
    };

    let Ok(bytes) = fs::read(&admitted.canonical) else {
        return defective(SourceDefect::Unreadable);
    };
    let Ok(text) = encoding::decode(&bytes) else {
        return defective(SourceDefect::Unreadable);
    };
    // `decode` has already sanitized; §4 #5 is satisfied inside it.
    let Ok(document) = srt::parse(&text) else {
        return defective(SourceDefect::Malformed);
    };

    // A filename hint or a confident text detector that cannot name the
    // language leaves it unset, which is the `Dil Belirsiz` group
    // (ADR-0010 Karar 6) — not an error.
    let hint = nen_subtitle::language::from_file_name(&admitted.file_name);
    let language = nen_subtitle::language::resolve_language(&document, hint.as_ref())
        .ok()
        .and_then(|resolution| resolution.language().cloned());

    LoadedFile::Loaded {
        source: SubtitleSource::new(id, language, admitted.file_name),
        document,
    }
}

/// The most candidates one scan will look at (ADR-0041 Karar 4).
///
/// Not a performance budget — a ceiling on pathological input. A medium with
/// more than sixteen subtitles beside it is not something real libraries
/// produce; a directory holding thousands of `Film.*.srt` entries is, and each
/// candidate costs up to [`MAX_SUBTITLE_BYTES`] of reading and parsing.
pub const MAX_SIDECAR_CANDIDATES: usize = 16;

/// Every path a sidecar scan looks at: the medium's own name with `.srt`, plus
/// whatever language-suffixed siblings sit next to it (ADR-0041).
///
/// One directory, read once, no recursion. The candidate set is everything
/// named `<basename>.….srt` — `Film.srt`, `Film.tr.srt`, `Film.en.sdh.srt`,
/// `Film.backup.srt`. Whether a suffix names a language is not asked here;
/// [`load`] already asks it of the filename it admits (NEN-057), so a second
/// opinion at this point would only be a second language table.
///
/// **The gates are not part of this decision.** Every returned path still goes
/// through [`admit`] one by one, so widening the set here cannot widen what is
/// accepted — ADR-0041 Karar 3, and the reason ADR-0034 Karar 2 survives it.
///
/// Order is fixed rather than whatever `read_dir` happens to yield: the exact
/// basename match comes first — so the one file today's scan finds can never be
/// the one the cap drops — and the rest sort by name. Empty when the medium has
/// no filename to build a candidate from.
pub fn sidecars_of(media: &Path) -> Vec<PathBuf> {
    if media.file_name().is_none() {
        return Vec::new();
    }
    let exact = media.with_extension("srt");
    let Some(parent) = media.parent() else {
        return vec![exact];
    };
    // `Inception.2010.mkv` → `Inception.2010`, i.e. the same basename the exact
    // match uses. Built from the exact path so the two can never disagree.
    let Some(stem) = exact.file_stem().and_then(|s| s.to_str()) else {
        return vec![exact];
    };
    let exact_name = exact.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // A directory that cannot be listed leaves the scan exactly where it was
    // before ADR-0041: the one known path. The new surface is never narrower
    // than the old one.
    let Ok(entries) = fs::read_dir(parent) else {
        return vec![exact];
    };

    let prefix = format!("{stem}.");
    let mut suffixed: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        // Case-insensitively, because on a case-insensitive volume `Film.SRT`
        // *is* the exact match: today's scan already finds it through
        // `Film.srt`, and letting the listing name it a second time would put
        // one file in the catalog twice under two spellings.
        .filter(|name| !name.eq_ignore_ascii_case(exact_name))
        .filter(|name| name.starts_with(&prefix))
        // Case-insensitive on the extension alone: `Film.SRT` is already found
        // today on a case-insensitive volume, and the listing must not be the
        // thing that takes it away. The prefix stays exact — it names the
        // medium the user opened.
        .filter(|name| {
            Path::new(name)
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("srt"))
        })
        .collect();
    suffixed.sort();

    let mut candidates = vec![exact];
    candidates.extend(
        suffixed
            .into_iter()
            .take(MAX_SIDECAR_CANDIDATES - 1)
            .map(|name| parent.join(name)),
    );
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_exact_basename_is_always_the_first_candidate() {
        // The one path today's scan finds. First in the order means the cap
        // (ADR-0041 Karar 4) can never be the thing that drops it.
        let candidates = sidecars_of(Path::new("/m/Inception.2010.mkv"));
        assert_eq!(
            candidates.first().map(PathBuf::as_path),
            Some(Path::new("/m/Inception.2010.srt"))
        );
    }

    #[test]
    fn a_candidate_replaces_the_extension_rather_than_appending_one() {
        // "aynı basename" — `Inception.2010.mkv.srt` would be a different one.
        let exact = sidecars_of(Path::new("/m/Inception.2010.mkv"))
            .into_iter()
            .next()
            .expect("a candidate");
        assert_eq!(exact.extension().and_then(|e| e.to_str()), Some("srt"));
        assert_eq!(
            exact.file_stem().and_then(|e| e.to_str()),
            Some("Inception.2010")
        );
    }

    #[test]
    fn a_path_without_a_filename_has_no_candidates() {
        assert!(sidecars_of(Path::new("/")).is_empty());
    }

    #[test]
    fn an_unlistable_directory_still_yields_the_exact_match() {
        // `/m` does not exist, so `read_dir` fails. ADR-0041 Karar 5: the scan
        // falls back to what it looked at before, never to nothing.
        assert_eq!(
            sidecars_of(Path::new("/m/Inception.2010.mkv")),
            vec![PathBuf::from("/m/Inception.2010.srt")]
        );
    }

    #[test]
    fn a_rejection_prints_its_reason_and_nothing_else() {
        let printed = format!("{:?}", FileRejection::Symlink);
        assert!(printed.contains("symlink"), "{printed}");
        assert!(!printed.contains('/'), "{printed}");
    }

    #[test]
    fn a_defect_prints_its_reason_and_nothing_else() {
        let printed = format!("{:?}", SourceDefect::Malformed);
        assert!(printed.contains("malformed"), "{printed}");
        assert!(!printed.contains('/'), "{printed}");
    }

    #[test]
    fn the_file_gate_and_the_decoder_share_one_limit() {
        // The two must not drift apart. If they did, the band between them
        // would be read in full and refused afterwards — the one behaviour
        // security-policy.md §4 #4 forbids.
        assert_eq!(MAX_SUBTITLE_BYTES, encoding::MAX_INPUT_BYTES as u64);
    }

    #[test]
    fn the_limit_still_clears_the_largest_subtitle_this_repo_has_measured() {
        // M1's 50 000-cue document is 3.1 MiB and is already far past what a
        // single film produces. Guards against someone tightening the shared
        // limit to a number that refuses legitimate files.
        const { assert!(MAX_SUBTITLE_BYTES > 3 * 3_100_000) };
    }
}
