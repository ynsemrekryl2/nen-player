//! M6 real-provider composition and cache-identity acceptance tests.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use nen_app::ports::credentials::{ApiKey, InMemoryCredentialStore, SecureCredentialStore};
use nen_app::ports::http::{HttpClient, HttpError, HttpRequest, HttpResponse};
use nen_app::ports::identity::MediaHash;
use nen_app::ports::translation::{
    TranslationCall, TranslationProvider, TranslationProviderError, TranslationProviderIdentity,
    TranslationRequest, TranslationResponse,
};
use nen_app::subtitles::{AddOutcome, SubtitleLibrary};
use nen_app::translation::{ProviderChoice, StartRefusal, TranslationEnvironment};
use nen_domain::source::LanguageTag;
use nen_providers::translation_mock::MockTranslationProvider;

struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nen-118-environment-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temp directory");
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

fn library(root: &Path) -> (SubtitleLibrary, u32) {
    let subtitle_path = root.join("Movie.en.srt");
    fs::write(
        &subtitle_path,
        "1\n00:00:01,000 --> 00:00:02,000\nFirst fixture.\n\n2\n00:00:03,000 --> 00:00:04,000\nSecond fixture.\n\n",
    )
    .expect("subtitle fixture");
    let mut library = SubtitleLibrary::new();
    assert_eq!(library.add_file(&subtitle_path, root), AddOutcome::Added);
    let source = library
        .catalog()
        .of_kind(nen_domain::source::SubtitleSourceKind::User)
        .next()
        .expect("source")
        .id()
        .clone();
    let token = library.token_of(&source).expect("token");
    (library, token)
}

#[derive(Default)]
struct RecordingHttpClient {
    calls: AtomicUsize,
    response: Mutex<Option<HttpResponse>>,
}

impl RecordingHttpClient {
    fn with_response(response: HttpResponse) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            response: Mutex::new(Some(response)),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl HttpClient for RecordingHttpClient {
    fn send(&self, _request: HttpRequest) -> Result<HttpResponse, HttpError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.response
            .lock()
            .expect("response lock")
            .clone()
            .ok_or(HttpError::Transport)
    }
}

#[test]
fn missing_real_provider_credential_refuses_before_http_and_leaves_artifacts_empty() {
    let root = TempDir::new("missing-key");
    let media = TempDir::new("missing-key-media");
    let (library, token) = library(media.path());
    let credentials = Arc::new(InMemoryCredentialStore::new());
    let http = Arc::new(RecordingHttpClient::default());
    let environment = TranslationEnvironment::new(
        root.path(),
        ProviderChoice::OpenAi {
            model: "gpt-5.6-luna".into(),
        },
        credentials,
        http.clone(),
        None,
    )
    .expect("environment");

    let refusal = match environment.start(
        &library,
        token,
        LanguageTag::parse("tr").expect("target"),
        None,
    ) {
        Ok(_) => panic!("missing key must refuse"),
        Err(refusal) => refusal,
    };
    assert_eq!(refusal, StartRefusal::MissingCredential);
    assert_eq!(http.calls(), 0);
    assert_eq!(
        fs::read_dir(root.path().join("artifacts"))
            .expect("artifacts")
            .count(),
        0
    );
}

#[test]
fn openai_environment_reads_secure_key_and_writes_an_artifact() {
    let root = TempDir::new("openai");
    let media = TempDir::new("openai-media");
    let (library, token) = library(media.path());
    let credentials = Arc::new(InMemoryCredentialStore::new());
    credentials
        .set(
            nen_app::ports::credentials::CredentialKind::OpenAi,
            ApiKey::new("fixture-key").expect("key"),
        )
        .expect("store key");
    let http = Arc::new(RecordingHttpClient::with_response(HttpResponse {
        status_code: 200,
        headers: Vec::new(),
        body: r#"{"output_text":"{\"cues\":[{\"cue_id\":1,\"text\":\"İlk fixture satırı\"},{\"cue_id\":2,\"text\":\"İkinci fixture satırı\"}]}"}"#
            .as_bytes()
            .to_vec(),
    }));
    let environment = TranslationEnvironment::new(
        root.path(),
        ProviderChoice::OpenAi {
            model: "gpt-5.6-luna".into(),
        },
        credentials,
        http.clone(),
        None,
    )
    .expect("environment");

    let outcome = environment
        .start(
            &library,
            token,
            LanguageTag::parse("tr").expect("target"),
            None,
        )
        .expect("job starts")
        .join()
        .expect("job completes");
    assert!(!outcome.from_cache);
    assert_eq!(http.calls(), 1);
    let artifacts: Vec<_> = fs::read_dir(root.path().join("artifacts"))
        .expect("artifacts")
        .map(|entry| entry.expect("entry").path())
        .collect();
    assert_eq!(artifacts.len(), 1);
    let wire = fs::read_to_string(&artifacts[0]).expect("artifact JSON");
    assert!(!wire.contains("fixture-key"));
    assert!(!format!("{:?}", outcome.record).contains("fixture-key"));
}

struct CountingProvider {
    inner: MockTranslationProvider,
    calls: AtomicUsize,
}

impl CountingProvider {
    fn new() -> Self {
        Self {
            inner: MockTranslationProvider::new(),
            calls: AtomicUsize::new(0),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl TranslationProvider for CountingProvider {
    fn identity(&self) -> TranslationProviderIdentity {
        self.inner.identity()
    }

    fn translate(
        &self,
        request: &TranslationRequest,
        call: &TranslationCall,
    ) -> Result<TranslationResponse, TranslationProviderError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.translate(request, call)
    }
}

#[test]
fn media_hash_is_part_of_environment_cache_identity() {
    let root = TempDir::new("hash-cache");
    let media = TempDir::new("hash-cache-media");
    let (library, token) = library(media.path());
    let hash_a = MediaHash::from_bytes([1; 8]);
    let hash_b = MediaHash::from_bytes([2; 8]);

    let first_provider = Arc::new(CountingProvider::new());
    let first = TranslationEnvironment::with_provider_and_media_hash(
        root.path(),
        first_provider.clone(),
        Some(hash_a),
    )
    .expect("first environment");
    assert!(
        !first
            .start(
                &library,
                token,
                LanguageTag::parse("tr").expect("target"),
                None,
            )
            .expect("first job")
            .join()
            .expect("first outcome")
            .from_cache
    );
    assert_eq!(first_provider.calls(), 1);

    let same_hash_provider = Arc::new(CountingProvider::new());
    let same_hash = TranslationEnvironment::with_provider_and_media_hash(
        root.path(),
        same_hash_provider.clone(),
        Some(hash_a),
    )
    .expect("same-hash environment");
    assert!(
        same_hash
            .start(
                &library,
                token,
                LanguageTag::parse("tr").expect("target"),
                None,
            )
            .expect("same-hash job")
            .join()
            .expect("same-hash outcome")
            .from_cache
    );
    assert_eq!(same_hash_provider.calls(), 0);

    let changed_hash_provider = Arc::new(CountingProvider::new());
    let changed_hash = TranslationEnvironment::with_provider_and_media_hash(
        root.path(),
        changed_hash_provider.clone(),
        Some(hash_b),
    )
    .expect("changed-hash environment");
    assert!(
        !changed_hash
            .start(
                &library,
                token,
                LanguageTag::parse("tr").expect("target"),
                None,
            )
            .expect("changed-hash job")
            .join()
            .expect("changed-hash outcome")
            .from_cache
    );
    assert_eq!(changed_hash_provider.calls(), 1);
}

#[test]
fn media_hash_helper_uses_nen018_windows() {
    let bytes = vec![7_u8; 131_072];
    let hash = nen_app::identity::media_hash_from_windows(
        bytes.len() as u64,
        &bytes[..65_536],
        &bytes[65_536..],
    );
    assert!(hash.is_some());
    assert_eq!(hash.expect("hash").as_bytes().len(), 8);
}
