//! Reverse-FFI boundary for platform secure credential stores (ADR-0020).

use nen_app::ports::credentials::{
    ApiKey, CredentialKind, CredentialStoreError, SecureCredentialStore,
};
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum FfiCredentialKind {
    OpenSubtitles,
    OpenAi,
    OpenRouter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Error)]
pub enum FfiCredentialError {
    /// The value supplied by the UI does not satisfy `ApiKey` validation.
    Invalid,
    Unavailable,
    Denied,
    Corrupt,
}

impl fmt::Display for FfiCredentialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Invalid => "invalid",
            Self::Unavailable => "unavailable",
            Self::Denied => "denied",
            Self::Corrupt => "corrupt",
        })
    }
}

impl From<FfiCredentialKind> for CredentialKind {
    fn from(value: FfiCredentialKind) -> Self {
        match value {
            FfiCredentialKind::OpenSubtitles => Self::OpenSubtitles,
            FfiCredentialKind::OpenAi => Self::OpenAi,
            FfiCredentialKind::OpenRouter => Self::OpenRouter,
        }
    }
}

impl From<CredentialKind> for FfiCredentialKind {
    fn from(value: CredentialKind) -> Self {
        match value {
            CredentialKind::OpenSubtitles => Self::OpenSubtitles,
            CredentialKind::OpenAi => Self::OpenAi,
            CredentialKind::OpenRouter => Self::OpenRouter,
        }
    }
}

impl From<CredentialStoreError> for FfiCredentialError {
    fn from(value: CredentialStoreError) -> Self {
        match value {
            CredentialStoreError::Unavailable => Self::Unavailable,
            CredentialStoreError::Denied => Self::Denied,
            CredentialStoreError::Corrupt => Self::Corrupt,
        }
    }
}

impl From<FfiCredentialError> for CredentialStoreError {
    fn from(value: FfiCredentialError) -> Self {
        match value {
            // Foreign stores never receive an unvalidated value from the
            // adapter. If one nevertheless reports Invalid, treat it as
            // corrupt storage data rather than inventing a fourth port error.
            FfiCredentialError::Invalid | FfiCredentialError::Corrupt => Self::Corrupt,
            FfiCredentialError::Unavailable => Self::Unavailable,
            FfiCredentialError::Denied => Self::Denied,
        }
    }
}

/// A platform implementation supplied by Swift (or another UniFFI host).
#[uniffi::export(with_foreign)]
pub trait ForeignSecureCredentialStore: Send + Sync {
    fn get(&self, kind: FfiCredentialKind) -> Result<Option<String>, FfiCredentialError>;
    fn contains(&self, kind: FfiCredentialKind) -> Result<bool, FfiCredentialError>;
    fn set(&self, kind: FfiCredentialKind, value: String) -> Result<(), FfiCredentialError>;
    fn delete(&self, kind: FfiCredentialKind) -> Result<(), FfiCredentialError>;
}

struct ForeignSecureCredentialStoreAdapter {
    inner: Arc<dyn ForeignSecureCredentialStore>,
}

impl SecureCredentialStore for ForeignSecureCredentialStoreAdapter {
    fn get(&self, kind: CredentialKind) -> Result<Option<ApiKey>, CredentialStoreError> {
        self.inner
            .get(kind.into())
            .map_err(CredentialStoreError::from)?
            .map(|value| ApiKey::new(&value).map_err(|_| CredentialStoreError::Corrupt))
            .transpose()
    }

    fn contains(&self, kind: CredentialKind) -> Result<bool, CredentialStoreError> {
        self.inner
            .contains(kind.into())
            .map_err(CredentialStoreError::from)
    }

    fn set(&self, kind: CredentialKind, key: ApiKey) -> Result<(), CredentialStoreError> {
        self.inner
            .set(kind.into(), key.expose().to_owned())
            .map_err(CredentialStoreError::from)
    }

    fn delete(&self, kind: CredentialKind) -> Result<(), CredentialStoreError> {
        self.inner
            .delete(kind.into())
            .map_err(CredentialStoreError::from)
    }
}

/// The UI-facing store object. Secret values can be written and presence can
/// be queried, but a stored value is never returned to the UI.
#[derive(uniffi::Object)]
pub struct FfiSecureCredentialStore {
    inner: ForeignSecureCredentialStoreAdapter,
}

#[uniffi::export]
impl FfiSecureCredentialStore {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn ForeignSecureCredentialStore>) -> Self {
        Self {
            inner: ForeignSecureCredentialStoreAdapter { inner: store },
        }
    }

    pub fn contains(&self, kind: FfiCredentialKind) -> Result<bool, FfiCredentialError> {
        self.inner.contains(kind.into()).map_err(Into::into)
    }

    pub fn set(&self, kind: FfiCredentialKind, value: String) -> Result<(), FfiCredentialError> {
        let key = ApiKey::new(&value).map_err(|_| FfiCredentialError::Invalid)?;
        self.inner.set(kind.into(), key).map_err(Into::into)
    }

    pub fn delete(&self, kind: FfiCredentialKind) -> Result<(), FfiCredentialError> {
        self.inner.delete(kind.into()).map_err(Into::into)
    }
}

impl FfiSecureCredentialStore {
    /// Internal composition hook for future provider environments. It is not
    /// exported to Swift, so the platform UI still cannot read a secret.
    #[allow(dead_code)]
    pub(crate) fn as_port(&self) -> &dyn SecureCredentialStore {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct ForeignFake {
        values: Mutex<std::collections::HashMap<FfiCredentialKind, String>>,
    }

    impl ForeignSecureCredentialStore for ForeignFake {
        fn get(&self, kind: FfiCredentialKind) -> Result<Option<String>, FfiCredentialError> {
            Ok(self.values.lock().expect("lock").get(&kind).cloned())
        }

        fn contains(&self, kind: FfiCredentialKind) -> Result<bool, FfiCredentialError> {
            Ok(self.values.lock().expect("lock").contains_key(&kind))
        }

        fn set(&self, kind: FfiCredentialKind, value: String) -> Result<(), FfiCredentialError> {
            self.values.lock().expect("lock").insert(kind, value);
            Ok(())
        }

        fn delete(&self, kind: FfiCredentialKind) -> Result<(), FfiCredentialError> {
            self.values.lock().expect("lock").remove(&kind);
            Ok(())
        }
    }

    #[test]
    fn foreign_adapter_round_trips_without_exposing_get_on_the_object() {
        let foreign = Arc::new(ForeignFake::default());
        let object = FfiSecureCredentialStore::new(foreign.clone());
        object
            .set(FfiCredentialKind::OpenAi, "  sentinel-key  ".into())
            .expect("set");
        assert!(object
            .contains(FfiCredentialKind::OpenAi)
            .expect("contains"));
        assert_eq!(
            foreign
                .get(FfiCredentialKind::OpenAi)
                .expect("get")
                .as_deref(),
            Some("sentinel-key")
        );
        object.delete(FfiCredentialKind::OpenAi).expect("delete");
        assert!(!object
            .contains(FfiCredentialKind::OpenAi)
            .expect("contains"));
    }

    #[test]
    fn foreign_invalid_stored_value_is_corrupt() {
        let foreign = Arc::new(ForeignFake::default());
        foreign
            .set(FfiCredentialKind::OpenRouter, "\n".into())
            .expect("set");
        let adapter = ForeignSecureCredentialStoreAdapter { inner: foreign };
        assert_eq!(
            adapter.get(CredentialKind::OpenRouter),
            Err(CredentialStoreError::Corrupt)
        );
    }

    #[test]
    fn invalid_ui_input_never_reaches_foreign_store() {
        let foreign = Arc::new(ForeignFake::default());
        let object = FfiSecureCredentialStore::new(foreign.clone());
        assert_eq!(
            object.set(FfiCredentialKind::OpenAi, "  \n  ".into()),
            Err(FfiCredentialError::Invalid)
        );
        assert!(!foreign
            .contains(FfiCredentialKind::OpenAi)
            .expect("contains"));
    }
}
