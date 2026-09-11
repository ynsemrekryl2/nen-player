//! Secure credential storage port and its deterministic in-memory adapter.
//!
//! The port deliberately carries no platform storage details. A platform
//! adapter stores one [`ApiKey`] per closed [`CredentialKind`] and reports
//! only payload-free failures (ADR-0020).

use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, MutexGuard};

/// The provider identities whose credentials are part of the product.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialKind {
    OpenSubtitles,
    OpenAi,
    OpenRouter,
}

impl CredentialKind {
    /// Stable account names for platform secure-storage adapters.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::OpenSubtitles => "opensubtitles",
            Self::OpenAi => "openai",
            Self::OpenRouter => "openrouter",
        }
    }

    pub const fn all() -> [Self; 3] {
        [Self::OpenSubtitles, Self::OpenAi, Self::OpenRouter]
    }
}

/// A validated API key. The value is intentionally not serializable.
///
/// Keeping the bytes in a dedicated allocation lets `Drop` overwrite this
/// allocation without adding a third-party dependency. Rust and platform FFI
/// strings can still create copies while crossing a boundary; this is the
/// best-effort memory lifetime promised by ADR-0020.
pub struct ApiKey(Vec<u8>);

impl ApiKey {
    pub fn new(value: &str) -> Result<Self, ApiKeyError> {
        let trimmed = value.trim();
        if trimmed.is_empty()
            || trimmed.chars().count() > 512
            || trimmed.chars().any(char::is_control)
        {
            return Err(ApiKeyError::Invalid);
        }
        Ok(Self(trimmed.as_bytes().to_vec()))
    }

    /// The sole explicitly named access path to the secret value.
    pub fn expose(&self) -> &str {
        // `ApiKey::new` accepts only valid UTF-8, and the private bytes are
        // never mutated except during Drop after all borrows have ended.
        std::str::from_utf8(&self.0).expect("validated API key is UTF-8")
    }
}

impl Clone for ApiKey {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl PartialEq for ApiKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ApiKey {}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(<redacted>)")
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl Drop for ApiKey {
    fn drop(&mut self) {
        self.0.fill(0);
    }
}

/// Why a caller-provided string cannot become an [`ApiKey`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiKeyError {
    Invalid,
}

impl fmt::Display for ApiKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid API key")
    }
}

impl std::error::Error for ApiKeyError {}

/// Payload-free failures from a platform secure credential store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialStoreError {
    Unavailable,
    Denied,
    Corrupt,
}

impl fmt::Display for CredentialStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unavailable => "credential store unavailable",
            Self::Denied => "credential store access denied",
            Self::Corrupt => "credential store data is corrupt",
        })
    }
}

impl std::error::Error for CredentialStoreError {}

/// Synchronous, object-safe secure credential storage boundary.
pub trait SecureCredentialStore: Send + Sync {
    fn get(&self, kind: CredentialKind) -> Result<Option<ApiKey>, CredentialStoreError>;
    fn contains(&self, kind: CredentialKind) -> Result<bool, CredentialStoreError>;
    fn set(&self, kind: CredentialKind, key: ApiKey) -> Result<(), CredentialStoreError>;
    fn delete(&self, kind: CredentialKind) -> Result<(), CredentialStoreError>;
}

/// Deterministic fake used by core tests and contract tests.
pub struct InMemoryCredentialStore {
    values: Mutex<HashMap<CredentialKind, ApiKey>>,
    #[cfg(test)]
    get_calls: std::sync::atomic::AtomicUsize,
}

impl InMemoryCredentialStore {
    pub fn new() -> Self {
        Self {
            values: Mutex::new(HashMap::new()),
            #[cfg(test)]
            get_calls: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn lock(
        &self,
    ) -> Result<MutexGuard<'_, HashMap<CredentialKind, ApiKey>>, CredentialStoreError> {
        self.values
            .lock()
            .map_err(|_| CredentialStoreError::Unavailable)
    }

    #[cfg(test)]
    fn get_call_count(&self) -> usize {
        self.get_calls.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for InMemoryCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SecureCredentialStore for InMemoryCredentialStore {
    fn get(&self, kind: CredentialKind) -> Result<Option<ApiKey>, CredentialStoreError> {
        #[cfg(test)]
        self.get_calls
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(self.lock()?.get(&kind).cloned())
    }

    fn contains(&self, kind: CredentialKind) -> Result<bool, CredentialStoreError> {
        Ok(self.lock()?.contains_key(&kind))
    }

    fn set(&self, kind: CredentialKind, key: ApiKey) -> Result<(), CredentialStoreError> {
        self.lock()?.insert(kind, key);
        Ok(())
    }

    fn delete(&self, kind: CredentialKind) -> Result<(), CredentialStoreError> {
        self.lock()?.remove(&kind);
        Ok(())
    }
}

pub mod contract {
    //! Shared behavioral contract for every [`SecureCredentialStore`] adapter.

    use super::{ApiKey, CredentialKind, SecureCredentialStore};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ContractViolation {
        UnexpectedInitialValue,
        SetGetMismatch,
        ContainsMismatch,
        KindCrossContamination,
        DeleteDidNotRemove,
        Unavailable,
    }

    fn key(value: &str) -> ApiKey {
        ApiKey::new(value).expect("contract fixture is valid")
    }

    /// Exercises the observable behavior required by ADR-0020.
    pub fn check(store: &dyn SecureCredentialStore) -> Result<(), Vec<ContractViolation>> {
        let mut failures = Vec::new();
        let kinds = CredentialKind::all();

        for kind in kinds {
            match (store.get(kind), store.contains(kind)) {
                (Ok(None), Ok(false)) => {}
                (Ok(_), Ok(_)) => failures.push(ContractViolation::UnexpectedInitialValue),
                _ => failures.push(ContractViolation::Unavailable),
            }
        }

        let values = [
            "contract-opensubtitles",
            "contract-openai",
            "contract-openrouter",
        ];
        for (kind, value) in kinds.into_iter().zip(values) {
            if store.set(kind, key(value)).is_err() {
                failures.push(ContractViolation::Unavailable);
                continue;
            }
            match (store.get(kind), store.contains(kind)) {
                (Ok(Some(actual)), Ok(true)) if actual.expose() == value => {}
                (Ok(_), Ok(_)) => failures.push(ContractViolation::SetGetMismatch),
                _ => failures.push(ContractViolation::Unavailable),
            }
        }

        for (index, kind) in kinds.into_iter().enumerate() {
            let expected = values[index];
            match store.get(kind) {
                Ok(Some(actual)) if actual.expose() == expected => {}
                Ok(_) => failures.push(ContractViolation::KindCrossContamination),
                Err(_) => failures.push(ContractViolation::Unavailable),
            }
        }

        for kind in kinds {
            if store.delete(kind).is_err() {
                failures.push(ContractViolation::Unavailable);
                continue;
            }
            match (store.get(kind), store.contains(kind)) {
                (Ok(None), Ok(false)) => {}
                (Ok(_), Ok(_)) => failures.push(ContractViolation::DeleteDidNotRemove),
                _ => failures.push(ContractViolation::Unavailable),
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::contract;
    use super::*;

    #[test]
    fn api_key_trims_and_rejects_invalid_values() {
        assert_eq!(ApiKey::new("  secret  ").expect("valid").expose(), "secret");
        assert!(ApiKey::new("").is_err());
        assert!(ApiKey::new(" \t ").is_err());
        assert!(ApiKey::new("line\nsecret").is_err());
        assert!(ApiKey::new(&"x".repeat(513)).is_err());
        assert!(ApiKey::new(&"x".repeat(512)).is_ok());
    }

    #[test]
    fn api_key_debug_and_display_are_redacted() {
        let key = ApiKey::new("sentinel-api-key").expect("valid");
        assert_eq!(format!("{key:?}"), "ApiKey(<redacted>)");
        assert_eq!(format!("{key}"), "<redacted>");
    }

    #[test]
    fn credential_kind_slugs_are_stable() {
        assert_eq!(CredentialKind::OpenSubtitles.slug(), "opensubtitles");
        assert_eq!(CredentialKind::OpenAi.slug(), "openai");
        assert_eq!(CredentialKind::OpenRouter.slug(), "openrouter");
    }

    #[test]
    fn fake_passes_the_shared_contract() {
        contract::check(&InMemoryCredentialStore::new()).expect("contract passes");
    }

    #[test]
    fn fake_contains_does_not_call_get() {
        let store = InMemoryCredentialStore::new();
        store
            .set(
                CredentialKind::OpenAi,
                ApiKey::new("sentinel").expect("valid"),
            )
            .expect("set");
        assert!(store.contains(CredentialKind::OpenAi).expect("contains"));
        assert_eq!(store.get_call_count(), 0);
    }
}
