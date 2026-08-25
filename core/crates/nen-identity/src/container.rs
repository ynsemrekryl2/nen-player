//! Metadata the container itself carries (ADR-0009 Karar 6, layer 3).
//!
//! Matroska has a `title` element and free-form tags; MP4 has the iTunes
//! atoms `©nam` and `©day`. Whoever produced the file wrote these, so they
//! count as a declaration — and unlike the filename they survive a rename,
//! which is why they rank above it.
//!
//! # What this module is, in M2
//!
//! The **shape** and the parsing of the values, not the demuxing. Reading
//! these fields out of a real file needs a demuxer, which arrives with the
//! playback engine in M3; until then a caller fills [`ContainerMetadata`] from
//! whatever it has (a test, a fixture, a handoff). ADR-0009 Karar 2 keeps the
//! I/O out of this crate either way, so this interface does not change when
//! the demuxer lands.
//!
//! `duration_ms` is carried but deliberately unused for identity here: on its
//! own a runtime identifies nothing. It exists because it is the cheapest
//! filter for narrowing provider candidates later (NEN-035).

use std::fmt;

use crate::release_name::{self, MediaKind, ParsedName};

/// Longest tag value we will accept. Container tags are untrusted input
/// (`docs/security-policy.md` §2).
const MAX_VALUE_BYTES: usize = 512;

/// Tags read out of a media container.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct ContainerMetadata {
    /// Matroska `title` / MP4 `©nam`.
    pub title: Option<String>,
    /// Matroska date tag / MP4 `©day`, already reduced to a year.
    pub year: Option<u16>,
    /// Playback duration, when the container states one.
    pub duration_ms: Option<u64>,
    /// Chapter names, in file order.
    pub chapter_titles: Vec<String>,
}

impl ContainerMetadata {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.year.is_none()
            && self.duration_ms.is_none()
            && self.chapter_titles.is_empty()
    }
}

impl fmt::Debug for ContainerMetadata {
    /// Shape and counts only — a container title names what the user is
    /// watching (K23 #8). Duration is a bare number that identifies nothing on
    /// its own, so its presence is shown but still not its value, keeping the
    /// rule simple: no field value ever prints.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContainerMetadata")
            .field("title", &presence(self.title.is_some()))
            .field("year", &presence(self.year.is_some()))
            .field("duration_ms", &presence(self.duration_ms.is_some()))
            .field("chapter_titles", &self.chapter_titles.len())
            .finish()
    }
}

fn presence(present: bool) -> &'static str {
    if present {
        "<present>"
    } else {
        "<none>"
    }
}

/// Normalizes raw tag strings into a [`ContainerMetadata`].
///
/// Values are trimmed, stripped of control characters and length-bounded; a
/// value that reduces to nothing is dropped rather than kept as an empty
/// string.
pub fn from_tags(
    title: Option<&str>,
    date: Option<&str>,
    duration_ms: Option<u64>,
    chapter_titles: &[String],
) -> ContainerMetadata {
    ContainerMetadata {
        title: title.and_then(clean_value),
        year: date.and_then(year_of),
        duration_ms,
        chapter_titles: chapter_titles
            .iter()
            .filter_map(|chapter| clean_value(chapter))
            .collect(),
    }
}

/// Reads identity out of container tags.
///
/// The title is taken as given — someone wrote it deliberately, so unlike a
/// filename it needs no release-tag boundary to be trusted. It is still run
/// through [`release_name::parse`] first, because muxers frequently copy the
/// release name verbatim into the title field, and when they do we want the
/// season and episode out of it.
pub fn to_parsed(metadata: &ContainerMetadata) -> ParsedName {
    let Some(title) = metadata.title.as_deref() else {
        return ParsedName::unknown();
    };

    let mut parsed = release_name::parse(title);

    if parsed.kind == MediaKind::Unknown {
        // A plain title like "Inception" carries no release tags, so the
        // parser reports Unknown by design. Here that is not noise: the tag
        // was written on purpose, so take it at face value.
        parsed = ParsedName {
            title: clean_value(title),
            year: None,
            season: None,
            episode: None,
            kind: MediaKind::Movie,
        };
    }

    parsed.year = parsed.year.or(metadata.year);
    parsed
}

fn year_of(value: &str) -> Option<u16> {
    let digits: String = value
        .trim()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    let year = digits.parse::<u16>().ok()?;
    (1900..=2999).contains(&year).then_some(year)
}

fn clean_value(raw: &str) -> Option<String> {
    let cleaned: String = raw
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_VALUE_BYTES)
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_title_tag_is_trusted_as_written() {
        let metadata = from_tags(Some("Inception"), Some("2010"), None, &[]);
        let parsed = to_parsed(&metadata);

        assert_eq!(parsed.title.as_deref(), Some("Inception"));
        assert_eq!(parsed.year, Some(2010));
        assert_eq!(parsed.kind, MediaKind::Movie);
    }

    #[test]
    fn a_release_name_in_the_title_tag_is_parsed() {
        let metadata = from_tags(Some("Breaking.Bad.S01E02.1080p.WEB-DL"), None, None, &[]);
        let parsed = to_parsed(&metadata);

        assert_eq!(parsed.title.as_deref(), Some("Breaking Bad"));
        assert_eq!(parsed.season, Some(1));
        assert_eq!(parsed.episode, Some(2));
        assert_eq!(parsed.kind, MediaKind::Series);
    }

    #[test]
    fn the_date_tag_fills_a_missing_year() {
        let metadata = from_tags(Some("Arrival"), Some("2016-11-11"), None, &[]);
        assert_eq!(to_parsed(&metadata).year, Some(2016));
    }

    #[test]
    fn no_title_tag_means_unknown() {
        let metadata = from_tags(None, Some("2016"), Some(7_200_000), &[]);
        assert_eq!(to_parsed(&metadata).kind, MediaKind::Unknown);
    }

    #[test]
    fn empty_and_whitespace_values_are_dropped() {
        let metadata = from_tags(Some("   "), Some("   "), None, &["".into(), " ".into()]);
        assert!(metadata.is_empty());
    }

    #[test]
    fn control_characters_are_stripped_and_values_bounded() {
        let long = "a".repeat(10_000);
        let metadata = from_tags(Some(&format!("Movie\u{0007}{long}")), None, None, &[]);
        let title = metadata.title.unwrap();
        assert!(!title.contains('\u{0007}'));
        assert!(title.chars().count() <= MAX_VALUE_BYTES);
    }

    #[test]
    fn duration_is_carried_but_does_not_create_an_identity() {
        let metadata = from_tags(None, None, Some(8_100_000), &[]);
        assert_eq!(metadata.duration_ms, Some(8_100_000));
        assert_eq!(to_parsed(&metadata).kind, MediaKind::Unknown);
    }

    #[test]
    fn debug_prints_shape_not_values() {
        let metadata = from_tags(Some("Inception"), Some("2010"), Some(8_880_000), &[]);
        let printed = format!("{metadata:?}");

        assert!(!printed.contains("Inception"), "title leaked: {printed}");
        assert!(!printed.contains("2010"), "year leaked: {printed}");
        assert!(!printed.contains("8880000"), "duration leaked: {printed}");
    }

    #[test]
    fn hostile_input_never_panics() {
        for title in ["\0", "\u{202e}", &"x".repeat(100_000), ""] {
            let metadata = from_tags(Some(title), Some(title), None, &[title.to_string()]);
            let _ = to_parsed(&metadata);
        }
    }
}
