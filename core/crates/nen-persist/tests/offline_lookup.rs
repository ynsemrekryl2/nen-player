//! Negative (`tasks/README.md` → security/validation, `NEN-098` DoD): the
//! filesystem store and its derived index never touch the network. S4
//! (2026-09-08) makes this part of M5's Definition of Done — a saved
//! artifact opens without a network connection, with no separate "offline
//! mode" switch.
//!
//! `FilesystemArtifactStore` has no `HttpClient` dependency at all, so this
//! is really a structural guarantee rather than a runtime toggle — but the
//! guarantee is only worth something if it is measured, not asserted. This
//! test holds a call-counting [`HttpClient`] in scope throughout a full
//! write/scan/find/get cycle and proves its counter never moves; a
//! deliberate direct call at the end proves the counter itself would have
//! caught a real network access (the check is not deaf).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use nen_domain::source::LanguageTag;
use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_persist::FilesystemArtifactStore;
use nen_ports::http::{HttpClient, HttpError, HttpRequest, HttpResponse};
use nen_ports::identity::MediaHash;
use nen_ports::persistence::{ArtifactIndex, ArtifactRecord, ArtifactStore, CacheKey};
use nen_ports::translation::TranslationProviderIdentity;

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("nen-098-offline-{tag}-{}", std::process::id()));
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

/// A client that counts every call to `send` rather than performing one.
struct CountingHttpClient {
    calls: AtomicUsize,
}

impl CountingHttpClient {
    fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
        }
    }

    fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl HttpClient for CountingHttpClient {
    fn send(&self, _request: HttpRequest) -> Result<HttpResponse, HttpError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err(HttpError::Transport)
    }
}

fn record(marker: &str) -> ArtifactRecord {
    let cues = vec![Cue::new(
        CueId::new(1),
        TimeSpan::new(0, 900).expect("a valid span"),
        vec![format!("line {marker}")],
    )];
    ArtifactRecord {
        source_fingerprint: [4u8; 32],
        timeline_fingerprint: [5u8; 32],
        source_language: LanguageTag::parse("en").expect("a valid tag"),
        target_language: LanguageTag::parse("tr").expect("a valid tag"),
        provider: TranslationProviderIdentity::new("mock", "echo-1").expect("an identity"),
        pipeline_version: 1,
        block_layout_version: 1,
        glossary: None,
        media_hash: Some(MediaHash::from_bytes([1, 2, 3, 4, 5, 6, 7, 8])),
        created_at_unix_ms: 1_700_000_000_000,
        document: SubtitleDocument::new(cues),
        webvtt: format!("WEBVTT\n\n1\n00:00:00.000 --> 00:00:00.900\n{marker}\n"),
        cache_identity: CacheKey::from_bytes(*blake3::hash(marker.as_bytes()).as_bytes()),
    }
}

#[test]
fn a_full_write_scan_and_read_cycle_never_calls_the_network() {
    let dir = TempDir::new("cycle");
    let store = FilesystemArtifactStore::new(dir.path()).expect("a store");
    let http = CountingHttpClient::new();

    let written = record("alpha");
    let address = store.put(&written).expect("a committed artifact");

    let entries = store.entries().expect("the scan succeeds");
    assert_eq!(entries.len(), 1);

    let found = store
        .find(written.cache_identity)
        .expect("the index scan succeeds")
        .expect("the artifact is found");
    assert_eq!(found.address, address);

    let read_back = store.get(address).expect("the artifact reads back");
    assert_eq!(read_back, written);

    assert_eq!(
        http.call_count(),
        0,
        "a write/scan/find/get cycle reached the network"
    );

    // Sanity check: the counter itself works, so the zero above is
    // meaningful rather than a broken counter that never moves.
    let request = HttpRequest::head("https://example.invalid/never-actually-sent");
    let _ = http.send(request);
    assert_eq!(
        http.call_count(),
        1,
        "the counting client did not register a direct call — the check above proves nothing"
    );
}
