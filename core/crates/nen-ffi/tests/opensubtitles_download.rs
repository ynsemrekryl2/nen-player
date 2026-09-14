use nen_app::ports::credentials::CredentialStoreError;
use nen_app::ports::subtitle_download::SubtitleDownloadError;
use nen_app::subtitles::DownloadRefusal;
use nen_ffi::subtitles::FfiDownloadError;

#[test]
fn provider_and_application_download_errors_map_to_flat_variants() {
    let provider_cases = [
        (
            SubtitleDownloadError::InvalidRequest,
            FfiDownloadError::InvalidRequest,
        ),
        (
            SubtitleDownloadError::Transport,
            FfiDownloadError::Transport,
        ),
        (
            SubtitleDownloadError::HttpStatus,
            FfiDownloadError::HttpStatus,
        ),
        (
            SubtitleDownloadError::Unauthorized,
            FfiDownloadError::Unauthorized,
        ),
        (
            SubtitleDownloadError::InvalidResponse,
            FfiDownloadError::InvalidResponse,
        ),
        (
            SubtitleDownloadError::QuotaExhausted,
            FfiDownloadError::QuotaExhausted,
        ),
        (
            SubtitleDownloadError::ResponseTooLarge,
            FfiDownloadError::ResponseTooLarge,
        ),
        (
            SubtitleDownloadError::RedirectRejected,
            FfiDownloadError::RedirectRejected,
        ),
        (
            SubtitleDownloadError::ContentTooLarge,
            FfiDownloadError::ContentTooLarge,
        ),
        (
            SubtitleDownloadError::UnexpectedContentType,
            FfiDownloadError::UnexpectedContentType,
        ),
        (
            SubtitleDownloadError::ArchiveRejected,
            FfiDownloadError::ArchiveRejected,
        ),
    ];
    for (provider, ffi) in provider_cases {
        assert_eq!(
            FfiDownloadError::from(DownloadRefusal::Provider(provider)),
            ffi
        );
    }

    assert_eq!(
        FfiDownloadError::from(DownloadRefusal::CredentialStore(
            CredentialStoreError::Unavailable,
        )),
        FfiDownloadError::CredentialStoreUnavailable
    );
    assert_eq!(
        FfiDownloadError::from(DownloadRefusal::CredentialStore(
            CredentialStoreError::Denied
        )),
        FfiDownloadError::CredentialStoreDenied
    );
    assert_eq!(
        FfiDownloadError::from(DownloadRefusal::CredentialStore(
            CredentialStoreError::Corrupt
        )),
        FfiDownloadError::CredentialStoreCorrupt
    );
    assert_eq!(
        FfiDownloadError::from(DownloadRefusal::Cancelled),
        FfiDownloadError::Cancelled
    );
}

#[test]
fn download_errors_are_payload_free_at_the_ffi_boundary() {
    let output = format!(
        "{:?} {}",
        FfiDownloadError::from(DownloadRefusal::Provider(
            SubtitleDownloadError::InvalidResponse,
        )),
        FfiDownloadError::from(DownloadRefusal::InvalidCredential),
    );
    assert!(!output.contains("https://dl.opensubtitles.com"));
    assert!(!output.contains("fixture-key"));
    assert!(!output.contains("7001"));
}
