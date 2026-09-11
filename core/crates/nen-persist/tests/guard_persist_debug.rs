//! Security guard for the persistence layer (`docs/security-policy.md` §1).
//!
//! Three forbidden things meet in this crate at once. An
//! [`ArtifactRecord`](nen_ports::persistence::ArtifactRecord) carries
//! translated dialogue (K23 #4). A store's root is a private full path
//! (K23 #3). And an artifact's file name *is* its content address — a private
//! hash of the user's own subtitle content (K23 #8). This proves none of the
//! three reaches a `Debug` or `Display` surface, and that the proof is not
//! vacuous: the sentinel is engineered to actually be in the record, and a
//! `#[derive(Debug)]` twin of the same payload is shown to leak it.

use nen_domain::source::LanguageTag;
use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_persist::FilesystemArtifactStore;
use nen_ports::identity::MediaHash;
use nen_ports::persistence::{
    ArtifactRecord, ArtifactStore, ArtifactStoreError, CacheKey, ContentAddress,
    ContentAddressError, ResumeBlock, ResumeCue, ResumeRecord, ResumeStoreError,
};
use nen_ports::translation::TranslationProviderIdentity;
use std::fs;
use std::path::{Path, PathBuf};

/// Text that must never reach a `Debug`/`Display` surface. Long and
/// distinctive so a partial leak is caught as surely as a whole one.
const SENTINEL: &str = "Zzqxvunlogged";

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nen-096-guard-{tag}-{}-{SENTINEL}",
            std::process::id()
        ));
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

fn record_with_sentinel() -> ArtifactRecord {
    let cues = (0..2u32)
        .map(|index| {
            let start = index * 1_000;
            Cue::new(
                CueId::new(index + 1),
                TimeSpan::new(start, start + 900).expect("a valid span"),
                vec![format!("Hello {SENTINEL} dialogue {index}.")],
            )
        })
        .collect();

    ArtifactRecord {
        source_fingerprint: [0xab; 32],
        timeline_fingerprint: [0xcd; 32],
        source_language: LanguageTag::parse("en").expect("a valid tag"),
        target_language: LanguageTag::parse("tr").expect("a valid tag"),
        provider: TranslationProviderIdentity::new("mock", "echo-1").expect("an identity"),
        pipeline_version: 1,
        block_layout_version: 1,
        glossary: Some(format!("glossary-{SENTINEL}")),
        media_hash: Some(MediaHash::from_bytes([9; 8])),
        created_at_unix_ms: 1_700_000_000_000,
        document: SubtitleDocument::new(cues),
        webvtt: format!("WEBVTT\n\n1\n00:00:00.000 --> 00:00:00.900\n{SENTINEL}\n"),
        cache_identity: CacheKey::from_bytes([0xef; 32]),
    }
}

#[test]
fn no_record_debug_output_leaks_dialogue_or_a_fingerprint() {
    let record = record_with_sentinel();

    // Guard is not vacuous: the sentinel really is in the record's payload.
    assert!(record.webvtt.contains(SENTINEL));
    assert!(record
        .document
        .cues()
        .iter()
        .flat_map(|cue| cue.lines())
        .any(|line| line.contains(SENTINEL)));

    let debug = format!("{record:?}");
    assert!(
        !debug.contains(SENTINEL),
        "ArtifactRecord::Debug leaked dialogue text — {debug}"
    );
    // The fingerprints are private hashes of the user's own media (K23 #8).
    assert!(
        !debug.contains("abab") && !debug.contains("cdcd"),
        "ArtifactRecord::Debug leaked a fingerprint — {debug}"
    );
    // The cache identity is a lookup key over sensitive components
    // (ADR-0018, `NEN-098`) and doubles as file-name-adjacent metadata.
    assert!(
        !debug.contains("efef"),
        "ArtifactRecord::Debug leaked the cache identity — {debug}"
    );
}

#[test]
fn a_derived_debug_really_would_leak_the_sentinel() {
    // The hand-written impl is what does the work above; a derive over the
    // same payload does not.
    #[derive(Debug)]
    #[allow(dead_code)]
    struct DerivedTwin {
        source_fingerprint: [u8; 32],
        cache_identity: [u8; 32],
        glossary: Option<String>,
        lines: Vec<String>,
        webvtt: String,
    }

    let record = record_with_sentinel();
    let twin = DerivedTwin {
        source_fingerprint: record.source_fingerprint,
        cache_identity: *record.cache_identity.as_bytes(),
        glossary: record.glossary.clone(),
        lines: record
            .document
            .cues()
            .iter()
            .flat_map(|cue| cue.lines().to_vec())
            .collect(),
        webvtt: record.webvtt.clone(),
    };

    let debug = format!("{twin:?}");
    assert!(
        debug.contains(SENTINEL),
        "the derived twin did not leak — this control proves nothing"
    );
    assert!(
        debug.contains("171, 171"),
        "the derived twin hid the fingerprint digest"
    );
    assert!(
        debug.contains("239, 239"),
        "the derived twin hid the cache identity digest"
    );
}

#[test]
fn no_content_address_surface_prints_the_file_name() {
    let address = ContentAddress::from_bytes([0xab; 32]);
    let hex = address.to_hex();

    // Not vacuous: the hex really is this address's file name.
    assert_eq!(hex.len(), 64);
    assert!(hex.starts_with("abab"));

    let debug = format!("{address:?}");
    let display = format!("{address}");
    assert!(
        !debug.contains(&hex) && !debug.contains("abab"),
        "ContentAddress::Debug leaked the artifact file name — {debug}"
    );
    assert!(
        !display.contains(&hex) && !display.contains("abab"),
        "ContentAddress::Display leaked the artifact file name — {display}"
    );
}

#[test]
fn no_store_debug_output_prints_the_store_root() {
    let dir = TempDir::new("root");
    let store = FilesystemArtifactStore::new(dir.path()).expect("a store");

    // Not vacuous: the root path really does carry the sentinel.
    let root = dir.path().to_string_lossy().into_owned();
    assert!(root.contains(SENTINEL));

    let debug = format!("{store:?}");
    assert!(
        !debug.contains(SENTINEL) && !debug.contains(&root),
        "FilesystemArtifactStore::Debug leaked the store root — {debug}"
    );
}

#[test]
fn no_resume_debug_surface_leaks_dialogue_or_its_file_key() {
    let record = ResumeRecord {
        cache_key: CacheKey::from_bytes([0xef; 32]),
        total_blocks: 2,
        blocks: vec![ResumeBlock {
            block_index: 0,
            cues: vec![ResumeCue {
                cue_id: CueId::new(1),
                text: SENTINEL.to_owned(),
            }],
        }],
    };
    assert!(record.blocks[0].cues[0].text.contains(SENTINEL));

    for debug in [
        format!("{record:?}"),
        format!("{:?}", record.blocks[0]),
        format!("{:?}", record.blocks[0].cues[0]),
    ] {
        assert!(
            !debug.contains(SENTINEL),
            "resume dialogue leaked — {debug}"
        );
        assert!(!debug.contains("efef"), "resume key leaked — {debug}");
    }
}

#[test]
fn no_error_variant_carries_a_path_a_name_or_dialogue() {
    let dir = TempDir::new("errors");
    let store = FilesystemArtifactStore::new(dir.path()).expect("a store");
    let record = record_with_sentinel();
    let address = store.put(&record).expect("a committed artifact");
    let hex = address.to_hex();
    let root = dir.path().to_string_lossy().into_owned();

    let errors = [
        ArtifactStoreError::NotFound,
        ArtifactStoreError::Io,
        ArtifactStoreError::Corrupt,
        ArtifactStoreError::OutsideRoot,
        ArtifactStoreError::TooLarge,
    ];
    for error in errors {
        for rendered in [format!("{error:?}"), format!("{error}")] {
            assert!(!rendered.contains(SENTINEL), "{rendered}");
            assert!(!rendered.contains(&hex), "{rendered}");
            assert!(!rendered.contains(&root), "{rendered}");
        }
    }

    for error in [ContentAddressError::Length, ContentAddressError::NotHex] {
        for rendered in [format!("{error:?}"), format!("{error}")] {
            assert!(!rendered.contains(SENTINEL), "{rendered}");
            assert!(!rendered.contains(&hex), "{rendered}");
        }
    }

    for error in [
        ResumeStoreError::Io,
        ResumeStoreError::Corrupt,
        ResumeStoreError::OutsideRoot,
        ResumeStoreError::TooLarge,
    ] {
        for rendered in [format!("{error:?}"), format!("{error}")] {
            assert!(!rendered.contains(SENTINEL), "{rendered}");
            assert!(!rendered.contains(&hex), "{rendered}");
            assert!(!rendered.contains(&root), "{rendered}");
        }
    }

    // A real failure path, not only the hand-built variants: reading an
    // address that is not there must report nothing about where it looked.
    let missing = ContentAddress::from_bytes([0u8; 32]);
    let error = store.get(missing).expect_err("nothing is stored there");
    for rendered in [format!("{error:?}"), format!("{error}")] {
        assert!(!rendered.contains(&root), "{rendered}");
        assert!(!rendered.contains("artifacts"), "{rendered}");
    }
}
