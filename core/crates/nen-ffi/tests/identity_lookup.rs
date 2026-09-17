//! NEN-120's app → FFI identity lookup path.

use nen_ffi::credentials::{
    FfiCredentialError, FfiCredentialKind, FfiSecureCredentialStore, ForeignSecureCredentialStore,
};
use nen_ffi::identity::{
    lookup_verified_identity_by_hash, FfiIdentityLookupError, FfiIdentityLookupResult,
    FfiIdentityLookupStatus, FfiVerifiedMediaIdentity,
};
use nen_ffi::remote_evidence::{FfiHttpError, FfiHttpRequest, FfiHttpResponse, ForeignHttpClient};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const SENTINEL_KEY: &str = "Zzqxvun-identity-api-key";
const SENTINEL_TITLE: &str = "Private Filename.mkv";
const HASH: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

struct FakeCredentialStore {
    key: Option<String>,
    gets: AtomicUsize,
}

impl FakeCredentialStore {
    fn new(key: Option<&str>) -> Self {
        Self {
            key: key.map(str::to_owned),
            gets: AtomicUsize::new(0),
        }
    }

    fn gets(&self) -> usize {
        self.gets.load(Ordering::Acquire)
    }
}

impl ForeignSecureCredentialStore for FakeCredentialStore {
    fn get(&self, _kind: FfiCredentialKind) -> Result<Option<String>, FfiCredentialError> {
        self.gets.fetch_add(1, Ordering::AcqRel);
        Ok(self.key.clone())
    }

    fn contains(&self, _kind: FfiCredentialKind) -> Result<bool, FfiCredentialError> {
        Ok(self.key.is_some())
    }

    fn set(&self, _kind: FfiCredentialKind, _value: String) -> Result<(), FfiCredentialError> {
        Ok(())
    }

    fn delete(&self, _kind: FfiCredentialKind) -> Result<(), FfiCredentialError> {
        Ok(())
    }
}

struct FakeHttp {
    calls: AtomicUsize,
}

impl FakeHttp {
    fn calls(&self) -> usize {
        self.calls.load(Ordering::Acquire)
    }
}

impl ForeignHttpClient for FakeHttp {
    fn send(&self, _request: FfiHttpRequest) -> Result<FfiHttpResponse, FfiHttpError> {
        self.calls.fetch_add(1, Ordering::AcqRel);
        Ok(FfiHttpResponse {
            status_code: 200,
            headers: Vec::new(),
            body: br#"{"data":[{"attributes":{"moviehash_match":true,"feature_details":{"title":"Private Filename.mkv","year":2010}}}]}"#.to_vec(),
        })
    }
}

fn store(key: Option<&str>) -> (Arc<FfiSecureCredentialStore>, Arc<FakeCredentialStore>) {
    let foreign = Arc::new(FakeCredentialStore::new(key));
    let store = Arc::new(FfiSecureCredentialStore::new(foreign.clone()));
    (store, foreign)
}

#[test]
fn exact_match_maps_to_a_redacted_ffi_record() {
    let (credentials, _) = store(Some(SENTINEL_KEY));
    let http = Arc::new(FakeHttp {
        calls: AtomicUsize::new(0),
    });

    let answer = lookup_verified_identity_by_hash(Some(HASH.to_vec()), credentials, http.clone())
        .expect("lookup");

    assert_eq!(answer.status, FfiIdentityLookupStatus::Match);
    assert_eq!(
        answer.identity,
        Some(FfiVerifiedMediaIdentity {
            title: SENTINEL_TITLE.into(),
            year: Some(2010),
            season: None,
            episode: None,
        })
    );
    assert_eq!(http.calls(), 1);
}

#[test]
fn missing_credential_is_typed_and_does_not_call_http() {
    let (credentials, foreign) = store(None);
    let http = Arc::new(FakeHttp {
        calls: AtomicUsize::new(0),
    });

    let answer = lookup_verified_identity_by_hash(Some(HASH.to_vec()), credentials, http.clone())
        .expect("missing credential is an ordinary result");

    assert_eq!(answer.status, FfiIdentityLookupStatus::NoCredential);
    assert_eq!(answer.identity, None);
    assert_eq!(foreign.gets(), 1);
    assert_eq!(http.calls(), 0);
}

#[test]
fn missing_hash_returns_before_credential_or_http_access() {
    let (credentials, foreign) = store(Some(SENTINEL_KEY));
    let http = Arc::new(FakeHttp {
        calls: AtomicUsize::new(0),
    });

    let answer = lookup_verified_identity_by_hash(None, credentials, http.clone())
        .expect("missing hash is an ordinary result");

    assert_eq!(answer.status, FfiIdentityLookupStatus::NoHash);
    assert_eq!(foreign.gets(), 0);
    assert_eq!(http.calls(), 0);
}

#[test]
fn identity_records_and_errors_never_print_hash_or_key_sentinels() {
    let answer = FfiIdentityLookupResult {
        status: FfiIdentityLookupStatus::Match,
        identity: Some(FfiVerifiedMediaIdentity {
            title: SENTINEL_TITLE.into(),
            year: Some(2010),
            season: None,
            episode: None,
        }),
    };
    let debug = format!("{answer:?}");
    assert!(!debug.contains(SENTINEL_TITLE));
    assert!(!debug.contains(SENTINEL_KEY));
    assert!(!debug.contains("0001020304050607"));

    let errors = [
        FfiIdentityLookupError::InvalidMediaHash,
        FfiIdentityLookupError::CredentialStoreUnavailable,
        FfiIdentityLookupError::ProviderTransport,
        FfiIdentityLookupError::RemoteTransport,
    ];
    for error in errors {
        let printed = format!("{error:?} {error}");
        assert!(!printed.contains(SENTINEL_KEY));
        assert!(!printed.contains("0001020304050607"));
    }
}
