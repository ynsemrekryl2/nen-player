//! Offline subtitle-language resolution (ADR-0029, NEN-020).
//!
//! Detection is deliberately document-wide: subtitle cues are short in
//! isolation, while their complete dialogue provides enough trigram evidence.
//! The assembled text is a call-local value and never enters an error, result,
//! `Debug` output or log (K23 #4).

use std::fmt;

use nen_domain::source::{LanguageTag, LanguageTagError};
use nen_domain::subtitle::SubtitleDocument;
use whatlang::Lang;

/// A text candidate must be strictly above this score to become authoritative.
///
/// The strict comparison matches Whatlang 0.18's documented `is_reliable`
/// boundary. Keeping the value here makes the product policy visible and keeps
/// a dependency update from silently changing it.
pub const MIN_TEXT_CONFIDENCE: f64 = 0.90;

/// A reliable language inferred from subtitle dialogue.
///
/// This type contains only a canonical language tag and a score. It never
/// contains the dialogue used to compute them, so its derived `Debug` is safe.
#[derive(Debug, Clone, PartialEq)]
pub struct DetectedLanguage {
    language: LanguageTag,
    confidence: f64,
}

impl DetectedLanguage {
    pub fn language(&self) -> &LanguageTag {
        &self.language
    }

    pub const fn confidence(&self) -> f64 {
        self.confidence
    }
}

/// A high-confidence text result disagreed with the authoritative metadata.
///
/// The metadata still wins. This value exists so callers can surface or count
/// the disagreement without retaining or logging subtitle dialogue.
#[derive(Debug, Clone, PartialEq)]
pub struct MetadataLanguageConflict {
    detected: DetectedLanguage,
}

impl MetadataLanguageConflict {
    pub fn detected(&self) -> &DetectedLanguage {
        &self.detected
    }
}

/// The canonical result consumed by the source-catalog layer.
#[derive(Debug, Clone, PartialEq)]
pub enum LanguageResolution {
    /// Valid metadata is authoritative; a reliable disagreement is retained.
    Metadata {
        language: LanguageTag,
        conflict: Option<MetadataLanguageConflict>,
    },
    /// No metadata was present and document text cleared the confidence gate.
    Text(DetectedLanguage),
    /// No metadata and no text result above the confidence gate.
    Unknown,
}

impl LanguageResolution {
    /// The final catalog language. `None` means ADR-0010's `Dil Belirsiz`.
    pub fn language(&self) -> Option<&LanguageTag> {
        match self {
            Self::Metadata { language, .. } => Some(language),
            Self::Text(detected) => Some(detected.language()),
            Self::Unknown => None,
        }
    }

    pub fn metadata_conflict(&self) -> Option<&MetadataLanguageConflict> {
        match self {
            Self::Metadata { conflict, .. } => conflict.as_ref(),
            Self::Text(_) | Self::Unknown => None,
        }
    }
}

/// The detector returned a language whose project-canonical tag was invalid.
///
/// The mapping is exhaustive and contains only compile-time literals, so this
/// is a defensive typed boundary rather than an expected runtime failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LanguageDetectionError {
    source: LanguageTagError,
}

impl fmt::Display for LanguageDetectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid canonical detector language tag: {}",
            self.source
        )
    }
}

impl std::error::Error for LanguageDetectionError {}

/// Resolves a subtitle's language without network or other side effects.
///
/// Metadata always determines the final language when supplied. Text is still
/// examined so a reliable primary-language disagreement can be reported.
pub fn resolve_language(
    document: &SubtitleDocument,
    metadata: Option<&LanguageTag>,
) -> Result<LanguageResolution, LanguageDetectionError> {
    let candidate = detect_document_language(document)?;
    Ok(resolve_candidate(metadata.cloned(), candidate))
}

fn detect_document_language(
    document: &SubtitleDocument,
) -> Result<Option<DetectedLanguage>, LanguageDetectionError> {
    let mut text = String::new();
    for cue in document.cues() {
        for line in cue.lines() {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(line);
        }
    }

    if !text.chars().any(|character| !character.is_whitespace()) {
        return Ok(None);
    }

    let Some(info) = whatlang::detect(&text) else {
        return Ok(None);
    };

    let language = LanguageTag::parse(canonical_primary_code(info.lang()))
        .map_err(|source| LanguageDetectionError { source })?;

    Ok(Some(DetectedLanguage {
        language,
        confidence: info.confidence(),
    }))
}

fn resolve_candidate(
    metadata: Option<LanguageTag>,
    candidate: Option<DetectedLanguage>,
) -> LanguageResolution {
    let reliable = candidate.filter(|detected| detected.confidence > MIN_TEXT_CONFIDENCE);

    match metadata {
        Some(language) => {
            let conflict = reliable
                .filter(|detected| language.primary() != detected.language.as_str())
                .map(|detected| MetadataLanguageConflict { detected });
            LanguageResolution::Metadata { language, conflict }
        }
        None => reliable
            .map(LanguageResolution::Text)
            .unwrap_or(LanguageResolution::Unknown),
    }
}

/// Project-canonical BCP-47 primary subtags for every Whatlang 0.18 language.
///
/// Matching the enum rather than its string code makes a dependency update
/// that adds a language fail at compile time until its canonical mapping is
/// consciously chosen.
const fn canonical_primary_code(language: Lang) -> &'static str {
    match language {
        Lang::Afr => "af",
        Lang::Aka => "ak",
        Lang::Amh => "am",
        Lang::Ara => "ar",
        Lang::Aze => "az",
        Lang::Bel => "be",
        Lang::Ben => "bn",
        Lang::Bul => "bg",
        Lang::Cat => "ca",
        Lang::Ces => "cs",
        Lang::Cmn => "zh",
        Lang::Cym => "cy",
        Lang::Dan => "da",
        Lang::Deu => "de",
        Lang::Ell => "el",
        Lang::Eng => "en",
        Lang::Epo => "eo",
        Lang::Est => "et",
        Lang::Fin => "fi",
        Lang::Fra => "fr",
        Lang::Guj => "gu",
        Lang::Heb => "he",
        Lang::Hin => "hi",
        Lang::Hrv => "hr",
        Lang::Hun => "hu",
        Lang::Hye => "hy",
        Lang::Ind => "id",
        Lang::Ita => "it",
        Lang::Jav => "jv",
        Lang::Jpn => "ja",
        Lang::Kan => "kn",
        Lang::Kat => "ka",
        Lang::Khm => "km",
        Lang::Kor => "ko",
        Lang::Lat => "la",
        Lang::Lav => "lv",
        Lang::Lit => "lt",
        Lang::Mal => "ml",
        Lang::Mar => "mr",
        Lang::Mkd => "mk",
        Lang::Mya => "my",
        Lang::Nep => "ne",
        Lang::Nld => "nl",
        Lang::Nob => "nb",
        Lang::Ori => "or",
        Lang::Pan => "pa",
        Lang::Pes => "fa",
        Lang::Pol => "pl",
        Lang::Por => "pt",
        Lang::Ron => "ro",
        Lang::Rus => "ru",
        Lang::Sin => "si",
        Lang::Slk => "sk",
        Lang::Slv => "sl",
        Lang::Sna => "sn",
        Lang::Spa => "es",
        Lang::Srp => "sr",
        Lang::Swe => "sv",
        Lang::Tam => "ta",
        Lang::Tel => "te",
        Lang::Tgl => "tl",
        Lang::Tha => "th",
        Lang::Tuk => "tk",
        Lang::Tur => "tr",
        Lang::Ukr => "uk",
        Lang::Urd => "ur",
        Lang::Uzb => "uz",
        Lang::Vie => "vi",
        Lang::Yid => "yi",
        Lang::Zul => "zu",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    fn tag(value: &str) -> LanguageTag {
        LanguageTag::parse(value).unwrap()
    }

    fn candidate(language: &str, confidence: f64) -> DetectedLanguage {
        DetectedLanguage {
            language: tag(language),
            confidence,
        }
    }

    #[test]
    fn threshold_is_strict_and_unknown_is_safe_fallback() {
        assert_eq!(
            resolve_candidate(None, Some(candidate("en", MIN_TEXT_CONFIDENCE))),
            LanguageResolution::Unknown
        );

        assert!(matches!(
            resolve_candidate(
                None,
                Some(candidate("en", MIN_TEXT_CONFIDENCE + f64::EPSILON))
            ),
            LanguageResolution::Text(_)
        ));
    }

    #[test]
    fn metadata_wins_and_reliable_primary_disagreement_is_retained() {
        let resolution = resolve_candidate(Some(tag("tr")), Some(candidate("en", 0.99)));

        assert_eq!(resolution.language(), Some(&tag("tr")));
        let conflict = resolution.metadata_conflict().unwrap();
        assert_eq!(conflict.detected().language(), &tag("en"));
        assert_eq!(conflict.detected().confidence(), 0.99);
    }

    #[test]
    fn region_difference_is_not_a_conflict_and_metadata_region_survives() {
        let resolution = resolve_candidate(Some(tag("en-us")), Some(candidate("en", 0.99)));

        assert_eq!(resolution.language(), Some(&tag("en-us")));
        assert_eq!(resolution.metadata_conflict(), None);
    }

    #[test]
    fn unreliable_text_never_conflicts_with_metadata() {
        let resolution = resolve_candidate(Some(tag("tr")), Some(candidate("en", 0.50)));

        assert_eq!(resolution.language(), Some(&tag("tr")));
        assert_eq!(resolution.metadata_conflict(), None);
    }

    #[test]
    fn every_detector_language_has_one_unique_valid_canonical_tag() {
        let mut tags = BTreeSet::new();
        let languages = Lang::all();
        assert_eq!(languages.len(), 70, "Whatlang language set changed");

        for language in languages {
            let code = canonical_primary_code(*language);
            LanguageTag::parse(code).unwrap_or_else(|err| panic!("{language:?}: {code}: {err}"));
            assert!(tags.insert(code), "duplicate canonical tag: {code}");
        }
    }
}
