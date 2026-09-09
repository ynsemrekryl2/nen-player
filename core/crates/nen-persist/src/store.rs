//! The filesystem artifact store (ADR-0017, `NEN-096`).
//!
//! One artifact is one file: `<root>/artifacts/<64 hex>.json`, where the hex
//! is the `blake3` hash of that file's own bytes. The layout is flat on
//! purpose — ADR-0017's own scale argument is that a user accumulates a few
//! hundred artifacts in a lifetime, so sharding would buy nothing and would
//! make `NEN-098`'s directory scan two levels deep instead of one.
//!
//! Writes are atomic (ADR-0017 Karar 3): a unique temporary name in the same
//! directory, `fsync` the file, `rename` onto the address, `fsync` the
//! directory. `rename` is atomic on POSIX, so the address either does not
//! exist or holds the complete artifact — there is no moment at which a
//! reader sees a partial one. If anything fails on the way, the temporary
//! file is removed and the address stays absent.
//!
//! The store root comes from the platform (ADR-0017 Karar 4); this crate only
//! refuses to leave it.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use nen_ports::persistence::{
    ArtifactRecord, ArtifactStore, ArtifactStoreError, ContentAddress, MAX_ARTIFACT_BYTES,
};

use crate::wire;

/// Subdirectory of the injected root that holds artifact files. Keeping them
/// under a named subdirectory leaves the root free for the separate area
/// ADR-0017 Karar 5 reserves for resumable checkpoints (`NEN-105`, M6).
const ARTIFACTS_DIR: &str = "artifacts";

/// Extension of an artifact file. Honest about the format, and it gives
/// `NEN-098`'s scan something to filter on so a stray temporary file is never
/// mistaken for an artifact.
const ARTIFACT_EXTENSION: &str = "json";

/// Distinguishes concurrent temporary files within one process.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A content-addressed artifact store backed by a directory.
///
/// `Debug` prints no path (K23 #3): a store's root is a private full path,
/// and the file names under it are private hashes (K23 #8).
pub struct FilesystemArtifactStore {
    root: PathBuf,
    artifacts_dir: PathBuf,
}

impl std::fmt::Debug for FilesystemArtifactStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FilesystemArtifactStore(<redacted>)")
    }
}

impl FilesystemArtifactStore {
    /// Opens (creating the artifacts subdirectory if needed) a store under
    /// `root`, which must already exist. The root is canonicalized once here
    /// so that every later containment check compares against a path with no
    /// symlinks or `..` left in it.
    pub fn new(root: impl AsRef<Path>) -> Result<Self, ArtifactStoreError> {
        let root = fs::canonicalize(root.as_ref()).map_err(|_| ArtifactStoreError::Io)?;
        let artifacts_dir = root.join(ARTIFACTS_DIR);
        fs::create_dir_all(&artifacts_dir).map_err(|_| ArtifactStoreError::Io)?;
        let artifacts_dir = fs::canonicalize(&artifacts_dir).map_err(|_| ArtifactStoreError::Io)?;
        if !artifacts_dir.starts_with(&root) {
            return Err(ArtifactStoreError::OutsideRoot);
        }
        Ok(Self {
            root,
            artifacts_dir,
        })
    }

    /// Resolves a file name inside the artifacts directory, refusing anything
    /// that would leave the root (ADR-0017 Karar 4).
    ///
    /// Two layers, in this order. Lexical first, before any syscall: the name
    /// must be exactly one ordinary path component, so `..`, `a/b` and an
    /// absolute path are all out. Then canonical: the directory the name
    /// lands in must still canonicalize under the root, which is what catches
    /// a symlinked artifacts directory that a purely lexical check would
    /// wave through. `nen_app::subtitle_files`'s `admit` is the precedent.
    pub(crate) fn resolve(&self, file_name: &str) -> Result<PathBuf, ArtifactStoreError> {
        let mut components = Path::new(file_name).components();
        if !matches!(components.next(), Some(Component::Normal(_))) {
            return Err(ArtifactStoreError::OutsideRoot);
        }
        if components.next().is_some() {
            return Err(ArtifactStoreError::OutsideRoot);
        }

        let path = self.artifacts_dir.join(file_name);
        let parent = path.parent().ok_or(ArtifactStoreError::OutsideRoot)?;
        let canonical_parent =
            fs::canonicalize(parent).map_err(|_| ArtifactStoreError::OutsideRoot)?;
        if !canonical_parent.starts_with(&self.root) {
            return Err(ArtifactStoreError::OutsideRoot);
        }
        Ok(path)
    }

    fn path_for(&self, address: ContentAddress) -> Result<PathBuf, ArtifactStoreError> {
        self.resolve(&format!("{}.{ARTIFACT_EXTENSION}", address.to_hex()))
    }

    /// Writes `bytes` to `path` atomically, feeding the file through `write`.
    ///
    /// `write` is a parameter so a test can interrupt the write partway with
    /// the real commit path running underneath it, rather than asserting
    /// against a filesystem state hand-built to look like a crash. Production
    /// passes [`write_all`].
    pub(crate) fn commit_with(
        &self,
        path: &Path,
        bytes: &[u8],
        write: impl Fn(&mut File, &[u8]) -> std::io::Result<()>,
    ) -> Result<(), ArtifactStoreError> {
        let temp_path = self.temp_path();

        let outcome = (|| -> std::io::Result<()> {
            let mut file = File::create(&temp_path)?;
            write(&mut file, bytes)?;
            // The bytes are on the device before the name exists, so the
            // rename below can never publish an address whose content has
            // not been persisted.
            file.sync_all()?;
            drop(file);
            fs::rename(&temp_path, path)?;
            // Persist the directory entry itself, so the artifact survives a
            // power loss right after the rename.
            File::open(&self.artifacts_dir)?.sync_all()
        })();

        if outcome.is_err() {
            // Best effort: a leftover temporary file is harmless (it is not
            // at an address and does not carry the artifact extension), but
            // there is no reason to leave it.
            let _ = fs::remove_file(&temp_path);
            return Err(ArtifactStoreError::Io);
        }
        Ok(())
    }

    fn temp_path(&self) -> PathBuf {
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        self.artifacts_dir
            .join(format!(".tmp-{}-{counter}", std::process::id()))
    }

    fn read_bytes(&self, path: &Path) -> Result<Vec<u8>, ArtifactStoreError> {
        // `symlink_metadata` does not follow the link: an artifact must be a
        // regular file inside the store, not a pointer at something else.
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(ArtifactStoreError::NotFound)
            }
            Err(_) => return Err(ArtifactStoreError::Io),
        };
        if !metadata.file_type().is_file() {
            return Err(ArtifactStoreError::Corrupt);
        }
        if metadata.len() > MAX_ARTIFACT_BYTES as u64 {
            return Err(ArtifactStoreError::TooLarge);
        }

        let mut file = File::open(path).map_err(|_| ArtifactStoreError::Io)?;
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        // Bounded even if the file grew between the stat and the open.
        std::io::Read::by_ref(&mut file)
            .take(MAX_ARTIFACT_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| ArtifactStoreError::Io)?;
        if bytes.len() > MAX_ARTIFACT_BYTES {
            return Err(ArtifactStoreError::TooLarge);
        }
        Ok(bytes)
    }
}

/// The production write step: the whole buffer, once.
fn write_all(file: &mut File, bytes: &[u8]) -> std::io::Result<()> {
    file.write_all(bytes)
}

impl ArtifactStore for FilesystemArtifactStore {
    fn put(&self, record: &ArtifactRecord) -> Result<ContentAddress, ArtifactStoreError> {
        let bytes = wire::serialize(record)?;
        let address = ContentAddress::from_bytes(*blake3::hash(&bytes).as_bytes());
        let path = self.path_for(address)?;

        // The address is the hash of these exact bytes, so a file already
        // sitting there is already this artifact: writing it again would
        // produce a byte-identical file. One copy, no rewrite.
        if path.exists() {
            return Ok(address);
        }

        self.commit_with(&path, &bytes, write_all)?;
        Ok(address)
    }

    fn get(&self, address: ContentAddress) -> Result<ArtifactRecord, ArtifactStoreError> {
        let path = self.path_for(address)?;
        let bytes = self.read_bytes(&path)?;
        // Content addressing is only worth anything if it is checked: bytes
        // that do not hash to the name they were found under are not this
        // artifact, whatever they parse as.
        if blake3::hash(&bytes).as_bytes() != address.as_bytes() {
            return Err(ArtifactStoreError::Corrupt);
        }
        wire::deserialize(&bytes)
    }

    fn contains(&self, address: ContentAddress) -> Result<bool, ArtifactStoreError> {
        let path = self.path_for(address)?;
        match fs::symlink_metadata(&path) {
            Ok(metadata) => Ok(metadata.file_type().is_file()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(_) => Err(ArtifactStoreError::Io),
        }
    }
}

/// A deliberately defective twin of [`FilesystemArtifactStore::put`], used
/// only to prove that the atomicity assertions below are not deaf: it writes
/// straight to the target address the way a naive implementation would.
#[cfg(test)]
impl FilesystemArtifactStore {
    fn put_non_atomic(
        &self,
        record: &ArtifactRecord,
        write: impl Fn(&mut File, &[u8]) -> std::io::Result<()>,
    ) -> Result<ContentAddress, ArtifactStoreError> {
        let bytes = wire::serialize(record)?;
        let address = ContentAddress::from_bytes(*blake3::hash(&bytes).as_bytes());
        let path = self.path_for(address)?;
        let mut file = File::create(&path).map_err(|_| ArtifactStoreError::Io)?;
        write(&mut file, &bytes).map_err(|_| ArtifactStoreError::Io)?;
        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nen_domain::source::LanguageTag;
    use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
    use nen_ports::identity::MediaHash;
    use nen_ports::persistence::contract;
    use nen_ports::translation::TranslationProviderIdentity;
    use std::sync::atomic::AtomicU32;

    /// Private to this crate's tests on purpose. Two more copies of this
    /// four-line helper already live in `nen-app`'s tests; hoisting all three
    /// into a shared dev-crate is real work with its own risks and belongs to
    /// its own task, not to this one (Kural 5).
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("nen-096-{tag}-{}-{counter}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("a temp directory");
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

    fn store(tag: &str) -> (TempDir, FilesystemArtifactStore) {
        let dir = TempDir::new(tag);
        let store = FilesystemArtifactStore::new(dir.path()).expect("a store under a fresh root");
        (dir, store)
    }

    fn record(marker: &str) -> ArtifactRecord {
        let cues = (0..3u32)
            .map(|index| {
                let start = index * 1_000;
                Cue::new(
                    CueId::new(index + 1),
                    TimeSpan::new(start, start + 900).expect("a valid span"),
                    vec![
                        format!("satır {index} {marker}"),
                        format!("ikinci {marker}"),
                    ],
                )
            })
            .collect();

        ArtifactRecord {
            source_fingerprint: [7u8; 32],
            timeline_fingerprint: [9u8; 32],
            source_language: LanguageTag::parse("en").expect("a valid tag"),
            target_language: LanguageTag::parse("tr").expect("a valid tag"),
            provider: TranslationProviderIdentity::new("mock", "echo-1").expect("an identity"),
            pipeline_version: 1,
            block_layout_version: 1,
            glossary: None,
            media_hash: None,
            created_at_unix_ms: 1_700_000_000_000,
            document: SubtitleDocument::new(cues),
            webvtt: format!("WEBVTT\n\n1\n00:00:00.000 --> 00:00:00.900\n{marker}\n"),
        }
    }

    fn address_of(record: &ArtifactRecord) -> ContentAddress {
        let bytes = wire::serialize(record).expect("a serializable record");
        ContentAddress::from_bytes(*blake3::hash(&bytes).as_bytes())
    }

    /// Every committed artifact in the store, by file name.
    fn artifact_files(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir.join(ARTIFACTS_DIR))
            .expect("the artifacts directory")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(ARTIFACT_EXTENSION))
            .collect();
        names.sort();
        names
    }

    /// A write that stops halfway and reports failure — a crash, a full disk
    /// or a killed process, from the commit path's point of view.
    fn interrupted_write(file: &mut File, bytes: &[u8]) -> std::io::Result<()> {
        let half = bytes.len() / 2;
        assert!(half > 0, "the fixture must be long enough to truncate");
        file.write_all(&bytes[..half])?;
        Err(std::io::Error::other("interrupted mid-write"))
    }

    #[test]
    fn an_artifact_reads_back_exactly_as_it_was_written() {
        let (_dir, store) = store("roundtrip");
        let written = record("alpha");

        let address = store.put(&written).expect("a committed artifact");
        let read_back = store.get(address).expect("the artifact reads back");

        assert_eq!(read_back, written);
        // Field by field too, so a future `PartialEq` that stops comparing
        // something cannot quietly weaken this test.
        assert_eq!(read_back.source_fingerprint, written.source_fingerprint);
        assert_eq!(read_back.timeline_fingerprint, written.timeline_fingerprint);
        assert_eq!(read_back.source_language, written.source_language);
        assert_eq!(read_back.target_language, written.target_language);
        assert_eq!(read_back.provider, written.provider);
        assert_eq!(read_back.pipeline_version, written.pipeline_version);
        assert_eq!(read_back.block_layout_version, written.block_layout_version);
        assert_eq!(read_back.glossary, written.glossary);
        assert_eq!(read_back.media_hash, written.media_hash);
        assert_eq!(read_back.created_at_unix_ms, written.created_at_unix_ms);
        assert_eq!(read_back.webvtt, written.webvtt);
        assert_eq!(
            read_back.document.cues().len(),
            written.document.cues().len()
        );
        for (read, source) in read_back
            .document
            .cues()
            .iter()
            .zip(written.document.cues())
        {
            assert_eq!(read.id(), source.id());
            assert_eq!(read.span(), source.span());
            assert_eq!(read.lines(), source.lines());
        }
    }

    #[test]
    fn the_optional_fields_round_trip_when_they_are_present() {
        let (_dir, store) = store("optional");
        let mut written = record("beta");
        written.glossary = Some("hunter-x-hunter".to_owned());
        written.media_hash = Some(MediaHash::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]));

        let address = store.put(&written).expect("a committed artifact");
        assert_eq!(store.get(address).expect("reads back"), written);
    }

    #[test]
    fn the_same_content_written_twice_keeps_one_copy() {
        use std::os::unix::fs::MetadataExt;

        let (dir, store) = store("dedup");
        let written = record("alpha");

        let first = store.put(&written).expect("first commit");
        let path = store.path_for(first).expect("an in-root address");
        let inode = fs::metadata(&path).expect("the committed file").ino();

        let second = store.put(&written).expect("second commit");

        assert_eq!(first, second, "the same content moved address");
        assert_eq!(
            artifact_files(dir.path()).len(),
            1,
            "the second write left a second copy"
        );
        // The same file, not an identical replacement: a commit that already
        // exists is recognized before anything is written, so the second
        // `put` does not serialize, write and rename a byte-identical copy
        // over the first. A rename would have replaced the inode.
        assert_eq!(
            fs::metadata(&path).expect("still committed").ino(),
            inode,
            "an artifact that was already stored was written again"
        );
    }

    #[test]
    fn different_content_lands_at_a_different_address() {
        let (dir, store) = store("distinct");
        let first = store.put(&record("alpha")).expect("first commit");
        let second = store.put(&record("beta")).expect("second commit");

        assert_ne!(first, second);
        assert_eq!(artifact_files(dir.path()).len(), 2);
    }

    /// Negative (`tasks/README.md` → security/validation): the DoD's central
    /// claim. A commit interrupted mid-write must leave nothing readable.
    #[test]
    fn an_interrupted_commit_leaves_no_readable_artifact() {
        let (dir, store) = store("interrupted");
        let written = record("alpha");
        let address = address_of(&written);
        let path = store.path_for(address).expect("an in-root address");

        let bytes = wire::serialize(&written).expect("a serializable record");
        assert_eq!(
            store.commit_with(&path, &bytes, interrupted_write),
            Err(ArtifactStoreError::Io)
        );

        assert_eq!(store.get(address), Err(ArtifactStoreError::NotFound));
        assert_eq!(store.contains(address), Ok(false));
        assert_eq!(
            artifact_files(dir.path()),
            Vec::<String>::new(),
            "an interrupted commit left a readable artifact behind"
        );
    }

    /// Negative — the control above is not deaf. Written the naive way, the
    /// *same* interruption leaves exactly what the previous test forbids.
    #[test]
    fn a_plain_write_leaves_a_readable_partial_artifact() {
        let (dir, store) = store("non-atomic");
        let written = record("alpha");
        let address = address_of(&written);

        assert_eq!(
            store.put_non_atomic(&written, interrupted_write),
            Err(ArtifactStoreError::Io)
        );

        assert_eq!(
            store.contains(address),
            Ok(true),
            "the deaf-control write left nothing — the control proves nothing"
        );
        assert_eq!(
            store.get(address),
            Err(ArtifactStoreError::Corrupt),
            "a truncated file must not parse as an artifact"
        );
        assert_eq!(
            artifact_files(dir.path()).len(),
            1,
            "the partial file is visible at the address"
        );
    }

    /// Negative: no name reaches a path outside the injected root
    /// (ADR-0017 Karar 4).
    #[test]
    fn a_name_that_leaves_the_root_is_refused() {
        let (_dir, store) = store("traversal");

        for name in [
            "..",
            ".",
            "../escape.json",
            "../../etc/passwd",
            "sub/dir.json",
            "/etc/passwd",
            "",
        ] {
            assert_eq!(
                store.resolve(name),
                Err(ArtifactStoreError::OutsideRoot),
                "name escaped the root: {name:?}"
            );
        }

        // Not deaf: an ordinary artifact file name does resolve.
        let name = format!("{}.{ARTIFACT_EXTENSION}", "0".repeat(64));
        assert!(store.resolve(&name).is_ok());
    }

    /// Negative: the canonical half of the containment check. A root whose
    /// artifacts directory is a symlink pointing out of the root is refused
    /// outright — a purely lexical check would wave it through.
    #[test]
    fn a_root_whose_artifacts_directory_escapes_is_refused() {
        let dir = TempDir::new("symlinked-dir");
        let root = dir.path().join("root");
        let outside = dir.path().join("outside");
        fs::create_dir_all(&root).expect("a root");
        fs::create_dir_all(&outside).expect("a directory outside the root");
        std::os::unix::fs::symlink(&outside, root.join(ARTIFACTS_DIR)).expect("a symlink");

        assert_eq!(
            FilesystemArtifactStore::new(&root).map(|_| ()),
            Err(ArtifactStoreError::OutsideRoot)
        );
    }

    /// An artifact must be a regular file. A symlink sitting at an address is
    /// not read through, whatever it points at.
    #[test]
    fn a_symlink_at_an_address_is_not_read_through() {
        let (dir, store) = store("symlinked-file");
        let real = record("alpha");
        let real_address = store.put(&real).expect("a committed artifact");
        let real_path = store.path_for(real_address).expect("an in-root address");

        let other = address_of(&record("beta"));
        let link_path = store.path_for(other).expect("an in-root address");
        std::os::unix::fs::symlink(&real_path, &link_path).expect("a symlink");

        assert_eq!(store.contains(other), Ok(false));
        assert_eq!(store.get(other), Err(ArtifactStoreError::Corrupt));
        // The real artifact is untouched.
        assert_eq!(store.get(real_address).expect("still readable"), real);
        assert_eq!(artifact_files(dir.path()).len(), 2);
    }

    /// Bytes that do not hash to the name they sit under are not that
    /// artifact, whatever they parse as.
    #[test]
    fn bytes_that_do_not_match_their_address_are_refused() {
        let (_dir, store) = store("mismatch");
        let alpha = record("alpha");
        let beta = record("beta");

        let alpha_address = store.put(&alpha).expect("a committed artifact");
        let beta_bytes = wire::serialize(&beta).expect("a serializable record");
        let alpha_path = store.path_for(alpha_address).expect("an in-root address");
        fs::write(&alpha_path, &beta_bytes).expect("overwrite the file behind the store's back");

        assert_eq!(store.get(alpha_address), Err(ArtifactStoreError::Corrupt));
    }

    #[test]
    fn the_shared_contract_kit_passes() {
        let (_dir, store) = store("contract");
        assert_eq!(contract::check(&store), Ok(()));
    }

    /// The kit is not deaf: a store that quietly loses part of what it was
    /// given fails it.
    #[test]
    fn the_shared_contract_kit_catches_a_lossy_store() {
        struct LossyStore(FilesystemArtifactStore);

        impl ArtifactStore for LossyStore {
            fn put(&self, record: &ArtifactRecord) -> Result<ContentAddress, ArtifactStoreError> {
                self.0.put(record)
            }

            fn get(&self, address: ContentAddress) -> Result<ArtifactRecord, ArtifactStoreError> {
                let mut record = self.0.get(address)?;
                record.webvtt.clear();
                Ok(record)
            }

            fn contains(&self, address: ContentAddress) -> Result<bool, ArtifactStoreError> {
                self.0.contains(address)
            }
        }

        let (_dir, store) = store("contract-lossy");
        let violations = contract::check(&LossyStore(store)).expect_err("the kit must object");
        assert!(violations.contains(&contract::ContractViolation::RoundTrip));
    }
}
