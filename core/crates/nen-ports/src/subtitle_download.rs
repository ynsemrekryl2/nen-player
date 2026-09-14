//! Provider-neutral subtitle download port (NEN-122, ADR-0021).
//!
//! A downloader returns bounded bytes only after its provider-specific link and
//! transport policy has passed. Encoding and strict SRT parsing remain in the
//! application layer; this type never crosses the FFI boundary.

use std::fmt;

/// The maximum subtitle body accepted by the download port (ADR-0021).
pub const MAX_DOWNLOAD_BYTES: usize = 10 * 1024 * 1024;
/// The maximum response used to obtain a temporary download link.
pub const MAX_DOWNLOAD_METADATA_BYTES: usize = 1024 * 1024;
/// The maximum number of temporary-link redirect hops.
pub const MAX_DOWNLOAD_REDIRECTS: u8 = 5;

/// Private provider identity needed only to form an explicit download request.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SubtitleDownloadRequest {
    pub private_file_id: u64,
}

impl SubtitleDownloadRequest {
    pub const fn new(private_file_id: u64) -> Self {
        Self { private_file_id }
    }
}

impl fmt::Debug for SubtitleDownloadRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubtitleDownloadRequest")
            .field("private_file_id", &"<redacted>")
            .finish()
    }
}

/// Bytes returned by a successful metadata-authorized download.
#[derive(Clone, PartialEq, Eq)]
pub struct DownloadedSubtitle {
    bytes: Vec<u8>,
}

impl DownloadedSubtitle {
    pub fn new(bytes: Vec<u8>) -> Result<Self, SubtitleDownloadError> {
        if bytes.len() > MAX_DOWNLOAD_BYTES {
            return Err(SubtitleDownloadError::ContentTooLarge);
        }
        Ok(Self { bytes })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl fmt::Debug for DownloadedSubtitle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DownloadedSubtitle")
            .field("body_length", &self.bytes.len())
            .finish()
    }
}

/// Payload-free failures from a provider download adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubtitleDownloadError {
    InvalidRequest,
    Transport,
    HttpStatus,
    Unauthorized,
    InvalidResponse,
    QuotaExhausted,
    ResponseTooLarge,
    RedirectRejected,
    ContentTooLarge,
    UnexpectedContentType,
    ArchiveRejected,
}

impl fmt::Display for SubtitleDownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidRequest => "invalid subtitle download request",
            Self::Transport => "subtitle download transport failed",
            Self::HttpStatus => "subtitle download request failed",
            Self::Unauthorized => "subtitle download was unauthorized",
            Self::InvalidResponse => "subtitle download response was invalid",
            Self::QuotaExhausted => "subtitle download quota is exhausted",
            Self::ResponseTooLarge => "subtitle download metadata was too large",
            Self::RedirectRejected => "subtitle download redirect was rejected",
            Self::ContentTooLarge => "subtitle content was too large",
            Self::UnexpectedContentType => "subtitle content type was unexpected",
            Self::ArchiveRejected => "subtitle archive content was rejected",
        })
    }
}

impl std::error::Error for SubtitleDownloadError {}

/// Performs one explicit provider download. Implementations must not log the
/// private request identity, URL, response body or credential.
pub trait SubtitleDownloader: Send + Sync {
    fn download(
        &self,
        request: SubtitleDownloadRequest,
    ) -> Result<DownloadedSubtitle, SubtitleDownloadError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_request_and_body_debug_are_redacted() {
        let request = SubtitleDownloadRequest::new(884_422);
        let body = DownloadedSubtitle::new(b"1\n00:00:00,000 --> 00:00:01,000\ntext".to_vec())
            .expect("small body");
        assert!(!format!("{request:?}").contains("884422"));
        assert!(!format!("{body:?}").contains("text"));
    }

    #[test]
    fn body_constructor_enforces_the_download_budget() {
        assert_eq!(
            DownloadedSubtitle::new(vec![b'x'; MAX_DOWNLOAD_BYTES + 1]),
            Err(SubtitleDownloadError::ContentTooLarge)
        );
    }
}
