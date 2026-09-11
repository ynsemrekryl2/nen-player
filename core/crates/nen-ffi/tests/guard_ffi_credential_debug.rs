//! K23 guard for the secure credential port and reverse-FFI surface.

use nen_app::ports::credentials::{ApiKey, CredentialKind, CredentialStoreError};
use nen_ffi::credentials::FfiCredentialError;

const SENTINEL: &str = "Zzqxvunlogged-api-key";

fn forbidden(output: &str) {
    assert!(!output.contains(SENTINEL), "credential leaked: {output}");
}

#[test]
fn port_and_ffi_secret_adjacent_types_never_print_the_sentinel() {
    let key = ApiKey::new(SENTINEL).expect("valid sentinel");
    forbidden(&format!("{key:?} {key}"));
    for error in [
        CredentialStoreError::Unavailable,
        CredentialStoreError::Denied,
        CredentialStoreError::Corrupt,
    ] {
        forbidden(&format!("{error:?} {error}"));
    }
    for error in [
        FfiCredentialError::Invalid,
        FfiCredentialError::Unavailable,
        FfiCredentialError::Denied,
        FfiCredentialError::Corrupt,
    ] {
        forbidden(&format!("{error:?} {error}"));
    }
    assert_eq!(CredentialKind::OpenAi.slug(), "openai");
}

#[test]
fn the_guard_is_not_blind_to_a_derived_debug_secret() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct LeakyTwin {
        secret: String,
    }

    let leaked = format!(
        "{:?}",
        LeakyTwin {
            secret: SENTINEL.into()
        }
    );
    assert!(leaked.contains(SENTINEL));
}
