//! Provider-neutral AI filename-normalization port (NEN-034, ADR-0046).
//!
//! The request is deliberately smaller than a media identity lookup: only a
//! bounded, sanitized basename stem crosses the provider boundary. The
//! result is an untrusted suggestion and never represents an exact identity.

use std::fmt;

pub const MAX_FILENAME_STEM_CHARS: usize = 256;
pub const MAX_NORMALIZED_TITLE_CHARS: usize = 512;

/// The only media-derived value an AI filename normalizer may receive.
#[derive(Clone, PartialEq, Eq)]
pub struct FilenameNormalizationRequest {
    stem: String,
}

impl FilenameNormalizationRequest {
    pub fn new(stem: &str) -> Result<Self, FilenameNormalizationError> {
        let stem = stem.trim();
        if stem.is_empty()
            || stem.chars().count() > MAX_FILENAME_STEM_CHARS
            || stem.chars().any(char::is_control)
            || stem.contains(['/', '\\'])
        {
            return Err(FilenameNormalizationError::InvalidRequest);
        }
        Ok(Self {
            stem: stem.to_owned(),
        })
    }

    /// Access is intentionally limited to the adapter that must serialize the
    /// provider request. Callers must not log this value.
    pub fn stem(&self) -> &str {
        &self.stem
    }
}

impl fmt::Debug for FilenameNormalizationRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilenameNormalizationRequest")
            .field("stem_len", &self.stem.chars().count())
            .finish()
    }
}

/// The bounded, typed answer from a normalizer. It is intentionally distinct
/// from [`nen_domain::source`] or verified identity DTOs: the model is not an
/// authority and this result is only a suggestion until the user accepts it.
#[derive(Clone, PartialEq, Eq)]
pub struct FilenameNormalization {
    pub title: String,
    pub year: Option<u16>,
    pub season: Option<u16>,
    pub episode: Option<u16>,
}

impl FilenameNormalization {
    pub fn new(
        title: String,
        year: Option<u16>,
        season: Option<u16>,
        episode: Option<u16>,
    ) -> Result<Self, FilenameNormalizationError> {
        let title = title.trim();
        if title.is_empty()
            || title.chars().count() > MAX_NORMALIZED_TITLE_CHARS
            || title.chars().any(char::is_control)
            || year.is_some_and(|value| !(1900..=2999).contains(&value))
            || season.is_some_and(|value| value > 99)
            || episode.is_some_and(|value| value > 999)
        {
            return Err(FilenameNormalizationError::InvalidResponse);
        }
        Ok(Self {
            title: title.to_owned(),
            year,
            season,
            episode,
        })
    }
}

impl fmt::Debug for FilenameNormalization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilenameNormalization")
            .field("title", &"<present>")
            .field("year", &self.year.is_some())
            .field("season", &self.season.is_some())
            .field("episode", &self.episode.is_some())
            .finish()
    }
}

/// A provider may decline to normalize without turning an ordinary unknown
/// filename into an error.
#[derive(Clone, PartialEq, Eq)]
pub enum FilenameNormalizationResult {
    Normalized(FilenameNormalization),
    Unknown,
}

impl fmt::Debug for FilenameNormalizationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Normalized(_) => "FilenameNormalizationResult::normalized",
            Self::Unknown => "FilenameNormalizationResult::unknown",
        })
    }
}

/// Payload-free normalizer failures. Provider request/response bodies and
/// credentials never leave the adapter through this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilenameNormalizationError {
    InvalidRequest,
    Transport,
    HttpStatus,
    InvalidResponse,
    ResponseTooLarge,
    Cancelled,
    CapabilityMissing,
}

impl fmt::Display for FilenameNormalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidRequest => "invalid filename normalization request",
            Self::Transport => "filename normalization transport failed",
            Self::HttpStatus => "filename normalization provider request failed",
            Self::InvalidResponse => "filename normalization provider response was invalid",
            Self::ResponseTooLarge => "filename normalization provider response was too large",
            Self::Cancelled => "filename normalization cancelled",
            Self::CapabilityMissing => "filename normalization provider capability missing",
        })
    }
}

impl std::error::Error for FilenameNormalizationError {}

pub trait FilenameNormalizer: Send + Sync {
    fn normalize(
        &self,
        request: &FilenameNormalizationRequest,
    ) -> Result<FilenameNormalizationResult, FilenameNormalizationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_debug_redacts_filename_stem() {
        let request = FilenameNormalizationRequest::new("Private Film.2024").expect("request");
        let printed = format!("{request:?}");
        assert!(!printed.contains("Private Film"));
        assert!(printed.contains("stem_len"));
    }

    #[test]
    fn normalized_result_debug_redacts_title() {
        let result = FilenameNormalizationResult::Normalized(
            FilenameNormalization::new("Private Film".into(), Some(2024), None, None)
                .expect("result"),
        );
        let printed = format!("{result:?}");
        assert!(!printed.contains("Private Film"));
        assert!(!format!("{result:?}").contains("2024"));
    }

    #[test]
    fn request_rejects_path_and_control_data() {
        assert_eq!(
            FilenameNormalizationRequest::new("../Private Film"),
            Err(FilenameNormalizationError::InvalidRequest)
        );
        assert_eq!(
            FilenameNormalizationRequest::new("Private\nFilm"),
            Err(FilenameNormalizationError::InvalidRequest)
        );
    }
}
