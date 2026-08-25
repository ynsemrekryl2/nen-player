//! Kodi/Plex `.nfo` sidecars (ADR-0009 Karar 6, layer 2).
//!
//! A library owner who has already curated their collection has written down
//! what each file is. Reading that is not the same as asking the user for a
//! technical ID — `docs/product-spec.md` §6 forbids *prompting* for one, and
//! this is a file already sitting on disk. It is the strongest evidence we get
//! without a network call, which is why it ranks second, below only an
//! explicit handoff.
//!
//! Two shapes are recognized: Kodi's XML (`<movie>`, `<episodedetails>`,
//! `<tvshow>`) and the one-line form that is just a database URL.
//!
//! # Hostile by default
//!
//! `docs/security-policy.md` §2 treats any such file as adversarial. There is
//! no XML library here and none is wanted: this does a bounded scan for a
//! handful of known tags, never resolves entities or DTDs, and gives up rather
//! than working hard on malformed input. A file it cannot make sense of yields
//! an empty [`Nfo`], never an error and never a panic.
//!
//! # No I/O
//!
//! The caller reads the file (ADR-0009 Karar 2); this parses the text.

use std::fmt;

/// Largest sidecar we will look at. Real ones are a few kilobytes.
pub const MAX_NFO_BYTES: usize = 256 * 1024;

/// Longest value we will take out of a tag.
const MAX_VALUE_BYTES: usize = 512;

/// What a sidecar claimed.
///
/// Every field is optional; an unparseable or irrelevant file yields
/// [`Nfo::empty`], which the resolution walk simply steps over.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct Nfo {
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<String>,
    pub title: Option<String>,
    pub year: Option<u16>,
    pub season: Option<u16>,
    pub episode: Option<u16>,
}

impl Nfo {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self == &Self::empty()
    }
}

impl fmt::Debug for Nfo {
    /// Shape only. A title and an IMDb id are both "what the user is
    /// watching", which K23 #8 keeps out of logs.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Nfo")
            .field("imdb_id", &presence(self.imdb_id.is_some()))
            .field("tmdb_id", &presence(self.tmdb_id.is_some()))
            .field("title", &presence(self.title.is_some()))
            .field("year", &presence(self.year.is_some()))
            .field("season", &presence(self.season.is_some()))
            .field("episode", &presence(self.episode.is_some()))
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

/// Parses a sidecar's contents.
///
/// Oversized input is refused up front — before any scanning — so a hostile
/// file cannot cost more than the length check.
pub fn parse(contents: &str) -> Nfo {
    if contents.len() > MAX_NFO_BYTES {
        return Nfo::empty();
    }

    let mut nfo = Nfo {
        title: tag(contents, "title"),
        year: tag(contents, "year").and_then(|value| parse_year(&value)),
        season: tag(contents, "season").and_then(|value| value.parse().ok()),
        episode: tag(contents, "episode").and_then(|value| value.parse().ok()),
        imdb_id: find_imdb_id(contents),
        tmdb_id: tag(contents, "tmdbid").or_else(|| unique_id(contents, "tmdb")),
    };

    // `<uniqueid type="imdb">tt…</uniqueid>` is the modern Kodi spelling; the
    // bare-URL form is caught by the same scan, so only fall back here.
    if nfo.imdb_id.is_none() {
        nfo.imdb_id = unique_id(contents, "imdb");
    }
    nfo
}

/// Text of the first `<name>…</name>` element, unescaped and bounded.
fn tag(contents: &str, name: &str) -> Option<String> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");

    let start = contents.find(&open)? + open.len();
    let rest = contents.get(start..)?;
    let end = rest.find(&close)?;
    let raw = rest.get(..end)?;

    clean_value(raw)
}

/// Text of a `<uniqueid type="…">` element.
fn unique_id(contents: &str, kind: &str) -> Option<String> {
    let needle = format!("type=\"{kind}\"");
    let at = contents.find(&needle)?;
    let rest = contents.get(at..)?;
    let start = rest.find('>')? + 1;
    let body = rest.get(start..)?;
    let end = body.find("</uniqueid>")?;
    clean_value(body.get(..end)?)
}

/// An IMDb id anywhere in the file — this is what makes the one-line URL form
/// work without a separate parser for it.
fn find_imdb_id(contents: &str) -> Option<String> {
    let mut rest = contents;
    while let Some(at) = rest.find("tt") {
        let after = rest.get(at + 2..).unwrap_or("");
        let digits: String = after.chars().take_while(char::is_ascii_digit).collect();
        if digits.len() >= 7 {
            return Some(format!("tt{digits}"));
        }
        rest = after;
        if rest.is_empty() {
            break;
        }
    }
    None
}

fn parse_year(value: &str) -> Option<u16> {
    // Kodi sometimes writes a full date; the leading four digits are the year.
    let digits: String = value.chars().take_while(char::is_ascii_digit).collect();
    let year = digits.parse::<u16>().ok()?;
    (1900..=2999).contains(&year).then_some(year)
}

fn clean_value(raw: &str) -> Option<String> {
    let unescaped = raw
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&");

    let cleaned: String = unescaped
        .chars()
        .filter(|c| !c.is_control() || *c == '\n')
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
    fn kodi_movie_xml() {
        let xml = r#"<?xml version="1.0"?>
<movie>
  <title>Inception</title>
  <year>2010</year>
  <uniqueid type="imdb">tt1375666</uniqueid>
  <uniqueid type="tmdb">27205</uniqueid>
</movie>"#;
        let nfo = parse(xml);
        assert_eq!(nfo.title.as_deref(), Some("Inception"));
        assert_eq!(nfo.year, Some(2010));
        assert_eq!(nfo.imdb_id.as_deref(), Some("tt1375666"));
        assert_eq!(nfo.tmdb_id.as_deref(), Some("27205"));
    }

    #[test]
    fn kodi_episode_xml() {
        let xml = r#"<episodedetails>
  <title>Cat's in the Bag...</title>
  <season>1</season>
  <episode>2</episode>
</episodedetails>"#;
        let nfo = parse(xml);
        assert_eq!(nfo.season, Some(1));
        assert_eq!(nfo.episode, Some(2));
        assert_eq!(nfo.title.as_deref(), Some("Cat's in the Bag..."));
    }

    #[test]
    fn the_one_line_url_form() {
        let nfo = parse("https://www.imdb.com/title/tt0111161/\n");
        assert_eq!(nfo.imdb_id.as_deref(), Some("tt0111161"));
        assert_eq!(nfo.title, None);
    }

    #[test]
    fn a_full_date_yields_just_the_year() {
        let nfo = parse("<movie><year>2010-07-16</year></movie>");
        assert_eq!(nfo.year, Some(2010));
    }

    #[test]
    fn entities_are_unescaped() {
        let nfo = parse("<movie><title>Tom &amp; Jerry</title></movie>");
        assert_eq!(nfo.title.as_deref(), Some("Tom & Jerry"));
    }

    #[test]
    fn a_year_out_of_range_is_dropped() {
        assert_eq!(parse("<movie><year>1234</year></movie>").year, None);
        assert_eq!(parse("<movie><year>abcd</year></movie>").year, None);
    }

    #[test]
    fn a_short_tt_number_is_not_an_imdb_id() {
        assert_eq!(parse("tt123 and tt45").imdb_id, None);
    }

    #[test]
    fn malformed_input_yields_an_empty_result_not_an_error() {
        for text in [
            "",
            "<movie>",
            "</title>",
            "<title>",
            "<title></title>",
            "not xml at all",
            "<movie><title>   </title></movie>",
        ] {
            let nfo = parse(text);
            assert!(nfo.title.is_none(), "unexpected title from {text:?}");
        }
    }

    #[test]
    fn an_oversized_file_is_refused_before_scanning() {
        let huge = format!(
            "<movie><title>X</title></movie>{}",
            "a".repeat(MAX_NFO_BYTES)
        );
        assert!(parse(&huge).is_empty());
    }

    #[test]
    fn a_value_is_length_bounded() {
        let long = format!("<movie><title>{}</title></movie>", "a".repeat(10_000));
        let title = parse(&long).title.unwrap();
        assert!(title.chars().count() <= MAX_VALUE_BYTES);
    }

    #[test]
    fn debug_prints_shape_not_values() {
        let nfo = parse("<movie><title>Inception</title><year>2010</year></movie>");
        let printed = format!("{nfo:?}");
        assert!(!printed.contains("Inception"), "title leaked: {printed}");
        assert!(!printed.contains("2010"), "year leaked: {printed}");
        assert!(printed.contains("<present>"));
    }

    #[test]
    fn hostile_input_never_panics() {
        for text in [
            "<title>".repeat(10_000).as_str(),
            "tt".repeat(50_000).as_str(),
            "<uniqueid type=\"imdb\">",
            "&amp;".repeat(10_000).as_str(),
            "\0<title>\0</title>",
            "<title>\u{202e}</title>",
        ] {
            let _ = parse(text);
        }
    }
}
