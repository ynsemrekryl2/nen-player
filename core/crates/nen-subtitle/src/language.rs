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

/// Filename suffixes that mark a subtitle's accessibility flavor rather than
/// its language, matched case-insensitively (NEN-057).
///
/// `LanguageTag::parse` accepts any two-or-three-letter alphabetic code, and
/// each of these happens to parse as one — `sdh` and `cc` are real ISO 639-2
/// codes in their own right, `forced` merely has the right shape. Treating
/// them as a language would open a spurious `SDH`/`CC`/`FORCED` menu group for
/// a source whose language is (and was already, correctly) found from its
/// text. `hi` is deliberately absent: it collides with "hearing impaired" in
/// some libraries, but it is also Hindi's real ISO 639-1 code, and a two-letter
/// code that names an actual language is kept rather than swallowed.
const NON_LANGUAGE_FILENAME_MARKERS: [&str; 3] = ["sdh", "cc", "forced"];

/// Reads the language a subtitle filename declares as a trailing
/// sub-extension, if any (NEN-057).
///
/// `Film.tr.srt` and `Film.pt-BR.srt` name their language explicitly;
/// `Film.forced.srt` and `Film.2019.srt` do not, and that is not an error —
/// an unrecognized or non-language trailing segment is silently ignored, the
/// same way absent metadata is ignored elsewhere in this module. Trailing
/// accessibility markers (see [`NON_LANGUAGE_FILENAME_MARKERS`]) are skipped
/// so `Film.en.sdh.srt` still yields `en`.
///
/// The candidate must be a genuine *sub*-extension — something must remain in
/// front of it — so `tr.srt` and `.tr.srt` produce no hint: there the
/// candidate is the filename itself, not a qualifier on one.
///
/// Takes the bare filename the user already sees (no directory component) and
/// returns a value that carries no filename of its own, so a caller cannot
/// forward this into a log and violate K23 #8 by accident.
pub fn from_file_name(file_name: &str) -> Option<LanguageTag> {
    let stem = file_name.strip_suffix(".srt").unwrap_or(file_name);
    let mut segments: Vec<&str> = stem.split('.').collect();

    while segments.last().is_some_and(|segment| {
        NON_LANGUAGE_FILENAME_MARKERS
            .iter()
            .any(|marker| marker.eq_ignore_ascii_case(segment))
    }) {
        segments.pop();
    }

    // `split_last` gives the candidate and everything in front of it without
    // an index that clippy (rightly) can't prove is in bounds. A single
    // remaining segment (empty `prefix`) or a prefix made only of empty
    // strings (`.tr` before the dropped `.srt`) means the candidate is the
    // filename itself, not a qualifier on one — `tr.srt` and `.tr.srt` both
    // land here.
    let (candidate, prefix) = segments.split_last()?;
    if !prefix.iter().any(|segment| !segment.is_empty()) {
        return None;
    }

    LanguageTag::parse(candidate).ok()
}

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

    // --- from_file_name (NEN-057) -------------------------------------

    #[test]
    fn a_declared_language_suffix_is_read() {
        assert_eq!(from_file_name("Film.tr.srt"), Some(tag("tr")));
        assert_eq!(from_file_name("Film.EN.srt"), Some(tag("en")));
        assert_eq!(
            from_file_name("Film.eng.srt"),
            Some(tag("en")),
            "639-2 canonicalizes"
        );
        assert_eq!(
            from_file_name("Film.hi.srt"),
            Some(tag("hi")),
            "a real ISO code, not the marker"
        );
    }

    #[test]
    fn a_region_subtag_is_kept_and_normalized() {
        assert_eq!(from_file_name("Film.pt-BR.srt"), Some(tag("pt-br")));
        assert_eq!(
            tag("pt-br").primary(),
            "pt",
            "grouping stays primary-only (ADR-0030)"
        );
    }

    #[test]
    fn a_trailing_accessibility_marker_is_skipped_for_the_language_beneath_it() {
        assert_eq!(from_file_name("Film.en.sdh.srt"), Some(tag("en")));
        assert_eq!(
            from_file_name("Film.tr.CC.srt"),
            Some(tag("tr")),
            "markers match case-insensitively"
        );
    }

    #[test]
    fn a_bare_marker_or_unparseable_suffix_produces_no_hint() {
        for name in [
            "Film.sdh.srt",
            "Film.cc.srt",
            "Film.forced.srt",
            "Film.2019.srt",
            "Film.srt",
            "Film.zh-hant-cn.srt",
        ] {
            assert_eq!(from_file_name(name), None, "{name}");
        }
    }

    #[test]
    fn a_language_shaped_filename_with_no_real_prefix_produces_no_hint() {
        // Here the candidate segment is the whole name, not a qualifier on
        // one — there is nothing for it to be a language *of*.
        assert_eq!(from_file_name("tr.srt"), None);
        assert_eq!(from_file_name(".tr.srt"), None);
    }

    #[test]
    fn a_filename_hint_becomes_authoritative_metadata_and_text_still_conflicts() {
        let hint = from_file_name("Film.tr.srt").expect("a hint");
        let text = Some(candidate("en", 0.99));

        let resolution = resolve_candidate(Some(hint), text);

        assert_eq!(resolution.language(), Some(&tag("tr")));
        let conflict = resolution
            .metadata_conflict()
            .expect("a reliable disagreement");
        assert_eq!(conflict.detected().language(), &tag("en"));
    }
}
