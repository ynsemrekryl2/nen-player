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

/// Reads container-declared title/year out of a media file's own first byte
/// window (NEN-072, ADR-0009 Karar 6, layer 3).
///
/// This is not a demuxer: it walks only far enough to find `Title`/`Tags`
/// (Matroska) or `moov/udta/meta/ilst` (MP4) *when they sit inside the given
/// window*. A container that puts its metadata later in the file — a large
/// MP4 muxed without `+faststart`, for instance — simply yields
/// [`ContainerMetadata::empty`]; that is a scope boundary, not a defect.
///
/// `window` is untrusted remote input (`docs/security-policy.md` §2): every
/// read is bounds-checked against `window`'s own length, nesting depth and
/// the number of elements visited are capped, and no arithmetic can
/// overflow. Malformed, truncated, oversized-claimed-size or pathologically
/// nested input all fall through to an empty result rather than a panic.
pub fn parse_head_window(window: &[u8]) -> ContainerMetadata {
    let (title, date) = if window.starts_with(&matroska::EBML_MAGIC) {
        matroska::find_tags(window)
    } else if mp4::looks_like_isobmff(window) {
        mp4::find_tags(window)
    } else {
        (None, None)
    };
    from_tags(title.as_deref(), date.as_deref(), None, &[])
}

fn decode_utf8(bytes: &[u8]) -> Option<String> {
    std::str::from_utf8(bytes).ok().map(str::to_owned)
}

/// A minimal, bounded EBML element walker — just enough of Matroska to find
/// `\Segment\Info\Title` and `\Segment\Tags\Tag\SimpleTag`. IDs and sizes
/// below follow the Matroska/EBML specification; element IDs used here never
/// exceed 4 bytes and sizes are read up to 8 bytes per the VINT encoding.
mod matroska {
    pub(super) const EBML_MAGIC: [u8; 4] = [0x1A, 0x45, 0xDF, 0xA3];

    const MAX_DEPTH: u32 = 6;
    const MAX_ELEMENTS: usize = 4096;

    const SEGMENT: u32 = 0x1853_8067;
    const INFO: u32 = 0x1549_A966;
    const TITLE: u32 = 0x7BA9;
    const TAGS: u32 = 0x1254_C367;
    const TAG: u32 = 0x7373;
    const SIMPLE_TAG: u32 = 0x67C8;
    const TAG_NAME: u32 = 0x45A3;
    const TAG_STRING: u32 = 0x4487;

    pub(super) fn find_tags(bytes: &[u8]) -> (Option<String>, Option<String>) {
        let mut budget = MAX_ELEMENTS;
        let mut title = None;
        let mut date = None;
        for (id, content) in ElementCursor::new(bytes) {
            if budget == 0 {
                break;
            }
            budget -= 1;
            if id == SEGMENT {
                find_in_segment(content, 1, &mut budget, &mut title, &mut date);
                break;
            }
        }
        (title, date)
    }

    fn find_in_segment(
        bytes: &[u8],
        depth: u32,
        budget: &mut usize,
        title: &mut Option<String>,
        date: &mut Option<String>,
    ) {
        if depth > MAX_DEPTH {
            return;
        }
        for (id, content) in ElementCursor::new(bytes) {
            if *budget == 0 {
                return;
            }
            *budget -= 1;
            match id {
                INFO if title.is_none() => find_in_info(content, depth + 1, budget, title),
                TAGS if date.is_none() => find_in_tags(content, depth + 1, budget, date),
                _ => {}
            }
            if title.is_some() && date.is_some() {
                return;
            }
        }
    }

    fn find_in_info(bytes: &[u8], depth: u32, budget: &mut usize, title: &mut Option<String>) {
        if depth > MAX_DEPTH {
            return;
        }
        for (id, content) in ElementCursor::new(bytes) {
            if *budget == 0 {
                return;
            }
            *budget -= 1;
            if id == TITLE && title.is_none() {
                *title = super::decode_utf8(content);
            }
        }
    }

    fn find_in_tags(bytes: &[u8], depth: u32, budget: &mut usize, date: &mut Option<String>) {
        if depth > MAX_DEPTH {
            return;
        }
        for (id, content) in ElementCursor::new(bytes) {
            if *budget == 0 {
                return;
            }
            *budget -= 1;
            if id == TAG && date.is_none() {
                find_in_tag(content, depth + 1, budget, date);
            }
        }
    }

    fn find_in_tag(bytes: &[u8], depth: u32, budget: &mut usize, date: &mut Option<String>) {
        if depth > MAX_DEPTH {
            return;
        }
        for (id, content) in ElementCursor::new(bytes) {
            if *budget == 0 {
                return;
            }
            *budget -= 1;
            if id == SIMPLE_TAG && date.is_none() {
                find_in_simple_tag(content, depth + 1, budget, date);
            }
        }
    }

    fn find_in_simple_tag(bytes: &[u8], depth: u32, budget: &mut usize, date: &mut Option<String>) {
        if depth > MAX_DEPTH {
            return;
        }
        let mut name = None;
        let mut value = None;
        for (id, content) in ElementCursor::new(bytes) {
            if *budget == 0 {
                return;
            }
            *budget -= 1;
            match id {
                TAG_NAME if name.is_none() => name = super::decode_utf8(content),
                TAG_STRING if value.is_none() => value = super::decode_utf8(content),
                _ => {}
            }
        }
        if let (Some(name), Some(value)) = (name, value) {
            if matches!(name.to_ascii_uppercase().as_str(), "DATE" | "DATE_RELEASED") {
                *date = Some(value);
            }
        }
    }

    /// Walks a byte range as a flat sequence of EBML elements, yielding
    /// `(id, content)` for each. Stops (does not panic) on any structural
    /// inconsistency — this is untrusted input, not a validated file.
    struct ElementCursor<'a> {
        bytes: &'a [u8],
        offset: usize,
    }

    impl<'a> ElementCursor<'a> {
        fn new(bytes: &'a [u8]) -> Self {
            Self { bytes, offset: 0 }
        }
    }

    impl<'a> Iterator for ElementCursor<'a> {
        type Item = (u32, &'a [u8]);

        fn next(&mut self) -> Option<Self::Item> {
            let rest = self.bytes.get(self.offset..)?;
            let (id, id_len) = read_id(rest)?;
            let (size, size_len) = read_size(rest.get(id_len..)?)?;
            let content_start = self.offset + id_len + size_len;
            if content_start > self.bytes.len() {
                return None;
            }
            let content_end = match size {
                Some(size) => content_start
                    .saturating_add(usize::try_from(size).unwrap_or(usize::MAX))
                    .min(self.bytes.len()),
                None => self.bytes.len(),
            };
            let content = self.bytes.get(content_start..content_end)?;
            self.offset = content_end;
            Some((id, content))
        }
    }

    /// Length in bytes of an EBML VINT from its leading byte, per the number
    /// of leading zero bits before the first set bit. `0` has none set and is
    /// not a valid VINT lead byte.
    fn vint_length(marker: u8) -> Option<u32> {
        if marker == 0 {
            return None;
        }
        Some(marker.leading_zeros() + 1)
    }

    /// Reads an EBML element ID, which (unlike a size) keeps its length
    /// marker bits as part of the value.
    fn read_id(bytes: &[u8]) -> Option<(u32, usize)> {
        let len = vint_length(*bytes.first()?)? as usize;
        if len > 4 {
            return None;
        }
        let slice = bytes.get(..len)?;
        let mut value: u32 = 0;
        for &b in slice {
            value = (value << 8) | u32::from(b);
        }
        Some((value, len))
    }

    /// Reads an EBML data-size VINT, masking out the length marker. An
    /// all-data-bits-set value means "unknown size" (streamed muxers); the
    /// caller treats that as "extends to the end of the given window".
    fn read_size(bytes: &[u8]) -> Option<(Option<u64>, usize)> {
        let len = vint_length(*bytes.first()?)? as usize;
        if len > 8 {
            return None;
        }
        let slice = bytes.get(..len)?;
        let (&first, rest) = slice.split_first()?;
        // `len` can be 8 (all bits are the marker, no data bits left in the
        // first byte); `>>8` on a `u8` would panic, so shift defensively.
        let marker_mask = 0xFFu8.checked_shr(len as u32).unwrap_or(0);
        let mut value = u64::from(first & marker_mask);
        let mut all_ones = first & marker_mask == marker_mask;
        for &b in rest {
            value = (value << 8) | u64::from(b);
            all_ones &= b == 0xFF;
        }
        if all_ones {
            Some((None, len))
        } else {
            Some((Some(value), len))
        }
    }
}

/// A minimal, bounded ISOBMFF (MP4) box walker — just enough to find
/// `moov/udta/meta/ilst`'s `©nam`/`©day` atoms.
mod mp4 {
    const MAX_DEPTH: u32 = 6;
    const MAX_BOXES: usize = 4096;

    const NAME_ATOM: [u8; 4] = *b"\xa9nam";
    const DAY_ATOM: [u8; 4] = *b"\xa9day";

    pub(super) fn looks_like_isobmff(bytes: &[u8]) -> bool {
        bytes.get(4..8).is_some_and(|fourcc| fourcc == b"ftyp")
    }

    pub(super) fn find_tags(bytes: &[u8]) -> (Option<String>, Option<String>) {
        let mut budget = MAX_BOXES;
        let mut title = None;
        let mut date = None;
        for (fourcc, content) in BoxCursor::new(bytes) {
            if budget == 0 {
                break;
            }
            budget -= 1;
            if &fourcc == b"moov" {
                find_udta(content, 1, &mut budget, &mut title, &mut date);
                break;
            }
        }
        (title, date)
    }

    fn find_udta(
        bytes: &[u8],
        depth: u32,
        budget: &mut usize,
        title: &mut Option<String>,
        date: &mut Option<String>,
    ) {
        if depth > MAX_DEPTH {
            return;
        }
        for (fourcc, content) in BoxCursor::new(bytes) {
            if *budget == 0 {
                return;
            }
            *budget -= 1;
            if &fourcc == b"udta" {
                find_meta(content, depth + 1, budget, title, date);
                return;
            }
        }
    }

    fn find_meta(
        bytes: &[u8],
        depth: u32,
        budget: &mut usize,
        title: &mut Option<String>,
        date: &mut Option<String>,
    ) {
        if depth > MAX_DEPTH {
            return;
        }
        for (fourcc, content) in BoxCursor::new(bytes) {
            if *budget == 0 {
                return;
            }
            *budget -= 1;
            if &fourcc == b"meta" {
                // `meta` is a FullBox: 4-byte version+flags precede its children.
                let children = content.get(4..).unwrap_or(&[]);
                find_ilst(children, depth + 1, budget, title, date);
                return;
            }
        }
    }

    fn find_ilst(
        bytes: &[u8],
        depth: u32,
        budget: &mut usize,
        title: &mut Option<String>,
        date: &mut Option<String>,
    ) {
        if depth > MAX_DEPTH {
            return;
        }
        for (fourcc, content) in BoxCursor::new(bytes) {
            if *budget == 0 {
                return;
            }
            *budget -= 1;
            if &fourcc == b"ilst" {
                find_tag_atoms(content, depth + 1, budget, title, date);
                return;
            }
        }
    }

    fn find_tag_atoms(
        bytes: &[u8],
        depth: u32,
        budget: &mut usize,
        title: &mut Option<String>,
        date: &mut Option<String>,
    ) {
        if depth > MAX_DEPTH {
            return;
        }
        for (fourcc, content) in BoxCursor::new(bytes) {
            if *budget == 0 {
                return;
            }
            *budget -= 1;
            if fourcc == NAME_ATOM && title.is_none() {
                *title = find_data_string(content, budget);
            } else if fourcc == DAY_ATOM && date.is_none() {
                *date = find_data_string(content, budget);
            }
            if title.is_some() && date.is_some() {
                return;
            }
        }
    }

    fn find_data_string(bytes: &[u8], budget: &mut usize) -> Option<String> {
        for (fourcc, content) in BoxCursor::new(bytes) {
            if *budget == 0 {
                return None;
            }
            *budget -= 1;
            if &fourcc == b"data" {
                // 4-byte type + 4-byte locale precede the payload.
                return super::decode_utf8(content.get(8..)?);
            }
        }
        None
    }

    /// Walks a byte range as a flat sequence of ISOBMFF boxes, yielding
    /// `(fourcc, content)` for each. Stops (does not panic) on any structural
    /// inconsistency — this is untrusted input, not a validated file.
    struct BoxCursor<'a> {
        bytes: &'a [u8],
        offset: usize,
    }

    impl<'a> BoxCursor<'a> {
        fn new(bytes: &'a [u8]) -> Self {
            Self { bytes, offset: 0 }
        }
    }

    impl<'a> Iterator for BoxCursor<'a> {
        type Item = ([u8; 4], &'a [u8]);

        fn next(&mut self) -> Option<Self::Item> {
            let rest = self.bytes.get(self.offset..)?;
            let size32 = u32::from_be_bytes(rest.get(..4)?.try_into().ok()?);
            let fourcc: [u8; 4] = rest.get(4..8)?.try_into().ok()?;
            let (header_len, box_size): (usize, u64) = if size32 == 1 {
                (16, u64::from_be_bytes(rest.get(8..16)?.try_into().ok()?))
            } else if size32 == 0 {
                (8, rest.len() as u64)
            } else {
                (8, u64::from(size32))
            };
            let content_start = self.offset + header_len;
            let box_end = self
                .offset
                .saturating_add(usize::try_from(box_size).unwrap_or(usize::MAX))
                .min(self.bytes.len());
            if content_start > box_end {
                return None;
            }
            let content = self.bytes.get(content_start..box_end)?;
            self.offset = box_end;
            Some((fourcc, content))
        }
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

    #[test]
    fn parse_head_window_on_unrecognized_bytes_is_empty() {
        let metadata = parse_head_window(b"just a plain file, not a container");
        assert!(metadata.is_empty());
    }

    #[test]
    fn parse_head_window_on_empty_input_is_empty() {
        assert!(parse_head_window(&[]).is_empty());
    }

    #[test]
    fn parse_head_window_on_truncated_matroska_magic_never_panics() {
        let magic = [0x1A, 0x45, 0xDF, 0xA3];
        for cut in 0..=magic.len() {
            assert!(parse_head_window(&magic[..cut]).is_empty());
        }
    }

    #[test]
    fn parse_head_window_on_truncated_isobmff_header_never_panics() {
        let header = b"\x00\x00\x00\x18ftypisom";
        for cut in 0..=header.len() {
            assert!(parse_head_window(&header[..cut]).is_empty());
        }
    }
}
