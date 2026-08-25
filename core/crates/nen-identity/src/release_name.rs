//! Release-name parsing: a filename in, a [`ParsedName`] out (ADR-0009 Karar 3).
//!
//! # Never an error
//!
//! Parsing a name that means nothing to us is not a failure. `docs/product-spec.md`
//! §6 is explicit that media plays whether or not we work out what it is, so
//! this module has no error type: an unrecognizable name yields
//! [`MediaKind::Unknown`] and the caller moves on to the next evidence layer.
//!
//! # What counts as a signal
//!
//! A bare `video.mkv` should *not* produce the movie title "video". The parser
//! therefore only commits to a title once it has seen some structural signal —
//! a season/episode marker, a year, or a recognized release tag. That is the
//! difference between "this name is telling us something" and "this name is
//! just a word", and it keeps the caller from ranking noise above a real hint
//! from the directory or the server.
//!
//! # Redaction
//!
//! A parsed title is what the user is watching, which K23 #8 keeps out of
//! logs. [`ParsedName`] therefore implements `Debug` by hand and prints only
//! the shape of the result — which fields are present, never their values.
//! Do not replace it with `#[derive(Debug)]`.

use std::fmt;

/// What kind of thing a name turned out to describe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MediaKind {
    Movie,
    Series,
    /// Nothing usable could be extracted.
    #[default]
    Unknown,
}

/// The identity coordinates recovered from a name.
///
/// `title` is `None` whenever nothing before the first structural signal
/// survived cleanup — a bare `S01E02.mkv` has an episode but no series name,
/// and the directory or handoff layer is expected to supply it.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct ParsedName {
    pub title: Option<String>,
    pub year: Option<u16>,
    pub season: Option<u16>,
    pub episode: Option<u16>,
    pub kind: MediaKind,
}

impl ParsedName {
    /// Nothing was recognized.
    pub fn unknown() -> Self {
        Self::default()
    }

    /// Whether this result is good enough to stop the layer walk.
    ///
    /// A result needs both a kind and a title: an episode number with no
    /// series name identifies nothing on its own, so it must not outrank a
    /// lower layer that knows the name.
    pub fn is_usable(&self) -> bool {
        self.kind != MediaKind::Unknown && self.title.is_some()
    }

    /// Fills in coordinates this result is missing from `other`, without ever
    /// touching `title` or `kind`.
    ///
    /// ADR-0009 Karar 6 gives the winning layer ownership of the identity
    /// itself; lower layers may still contribute a year or an episode number
    /// the winner did not carry (a `S01E02.mkv` inside `Breaking Bad (2008)/`
    /// is the ordinary case). Restricting this to the numeric fields keeps
    /// "first non-`Unknown` layer wins" true for the parts that decide *what*
    /// the media is.
    pub fn fill_gaps_from(&mut self, other: &ParsedName) {
        self.year = self.year.or(other.year);
        self.season = self.season.or(other.season);
        self.episode = self.episode.or(other.episode);
    }
}

impl fmt::Debug for ParsedName {
    /// Prints shape, never values — see the module docs (K23 #8).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParsedName")
            .field("kind", &self.kind)
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

/// Parses one name — a filename, a directory name, a URL path segment, or a
/// server-declared name. All four are the same problem, so they share this.
pub fn parse(name: &str) -> ParsedName {
    let stem = strip_leading_group(strip_extension(name));
    let tokens = tokenize(stem);

    let mut result = ParsedName::unknown();
    let mut title_tokens: Vec<&str> = Vec::new();
    let mut title_closed = false;
    let mut saw_tag = false;

    let mut index = 0;
    while let Some(token) = tokens.get(index) {
        let mut consumed = 1;

        if let Some((season, episode)) = season_episode(token) {
            result.season = result.season.or(season);
            result.episode = result.episode.or(Some(episode));
            title_closed = true;
        } else if let Some((season, episode, width)) = spelled_out_season_episode(&tokens, index) {
            result.season = result.season.or(season);
            result.episode = result.episode.or(episode);
            title_closed = true;
            consumed = width;
        } else if let Some(value) = year(token) {
            // `Blade.Runner.2049.2017.1080p`: two year-shaped tokens in a row
            // means the first one is part of the title and the second is the
            // release year. Without this the title loses its own number and
            // the film is looked up as the wrong year entirely.
            let next_is_year = tokens.get(index + 1).and_then(|next| year(next)).is_some();
            if next_is_year && !title_closed {
                title_tokens.push(token);
            } else {
                result.year = result.year.or(Some(value));
                title_closed = true;
            }
        } else if is_release_tag(token) {
            saw_tag = true;
            title_closed = true;
        } else if let Some(episode) = dashed_episode(&tokens, index) {
            result.episode = result.episode.or(Some(episode));
            title_closed = true;
        } else if !title_closed {
            title_tokens.push(token);
        }

        index += consumed;
    }

    result.title = clean_title(&title_tokens);
    result.kind = classify(&result, saw_tag);

    if result.kind == MediaKind::Unknown {
        // Keep the invariant crisp: `Unknown` means nothing was recognized, so
        // it must not leave a half-title behind for a caller to read directly.
        result.title = None;
    }

    result
}

/// Drops a leading `[Group]` tag, the anime-release convention. It is the one
/// bracketed prefix common enough to be worth handling, and keeping it would
/// put the fansub group's name at the front of every such title.
///
/// Only a *well-formed, short* prefix is removed. An unclosed `[` would
/// otherwise pair with a later bracket — `[Unclosed Show - 05 [1080p]` would
/// lose the episode along with everything else — so a candidate containing
/// another `[` is left alone, and so is one too long to be a group name.
fn strip_leading_group(stem: &str) -> &str {
    const MAX_GROUP_LEN: usize = 40;

    let trimmed = stem.trim_start();
    let Some(rest) = trimmed.strip_prefix('[') else {
        return trimmed;
    };
    let Some(close) = rest.find(']') else {
        return trimmed;
    };
    let Some(group) = rest.get(..close) else {
        return trimmed;
    };
    if group.len() > MAX_GROUP_LEN || group.contains('[') {
        return trimmed;
    }
    rest.get(close + 1..).unwrap_or(trimmed).trim_start()
}

/// A result is only trusted when the name showed structure. See the module
/// docs for why a bare word stays `Unknown`.
fn classify(parsed: &ParsedName, saw_tag: bool) -> MediaKind {
    // A season marker is a series marker even with no episode: a season pack
    // (`Show.S02.COMPLETE.1080p`) and a `Season 02/` folder are both series,
    // and calling either a movie would send the wrong query to a provider.
    if parsed.episode.is_some() || parsed.season.is_some() {
        return MediaKind::Series;
    }
    let has_signal = parsed.year.is_some() || saw_tag;
    if has_signal && parsed.title.is_some() {
        MediaKind::Movie
    } else {
        MediaKind::Unknown
    }
}

/// Drops a trailing extension when it looks like one: short and alphanumeric.
/// `Movie.2010.1080p` must keep its `1080p`, so length alone is not enough —
/// the tail also has to not parse as anything meaningful.
fn strip_extension(name: &str) -> &str {
    let Some(dot) = name.rfind('.') else {
        return name;
    };
    if dot == 0 {
        return name;
    }
    let Some(ext) = name.get(dot + 1..) else {
        return name;
    };
    let looks_like_extension = (1..=4).contains(&ext.len())
        && ext.chars().all(|c| c.is_ascii_alphanumeric())
        && !ext.chars().all(|c| c.is_ascii_digit());
    if looks_like_extension {
        name.get(..dot).unwrap_or(name)
    } else {
        name
    }
}

/// Splits a name into comparable tokens.
///
/// Scene names separate with `.` or `_`; some use `-` throughout while others
/// use it only before the release group, and plenty of titles contain a real
/// hyphen (`Spider-Man`). Hyphens are therefore only treated as separators
/// when nothing else in the name is one — otherwise `Spider-Man` would lose
/// its own name.
fn tokenize(stem: &str) -> Vec<&str> {
    let has_other_separator = stem.contains(['.', '_', ' ']);
    stem.split(move |c: char| {
        c.is_whitespace() || c == '.' || c == '_' || (c == '-' && !has_other_separator)
    })
    .filter(|token| !token.is_empty())
    .collect()
}

/// `S01E02`, `s1e2`, `S01E02E03` (first episode wins) and `1x02`.
fn season_episode(token: &str) -> Option<(Option<u16>, u16)> {
    let token = trim_brackets(token);
    let lower = token.to_ascii_lowercase();

    if let Some(rest) = lower.strip_prefix('s') {
        let (season_digits, rest) = take_digits(rest);
        if !season_digits.is_empty() {
            if let Some(rest) = rest.strip_prefix('e') {
                let (episode_digits, _) = take_digits(rest);
                if !episode_digits.is_empty() {
                    let season = season_digits.parse::<u16>().ok()?;
                    let episode = episode_digits.parse::<u16>().ok()?;
                    if season <= 99 {
                        return Some((Some(season), episode));
                    }
                }
            }
        }
    }

    let (season_digits, rest) = take_digits(&lower);
    if !season_digits.is_empty() {
        if let Some(rest) = rest.strip_prefix('x') {
            let (episode_digits, tail) = take_digits(rest);
            // Guard against resolutions like `1920x1080`.
            if !episode_digits.is_empty() && tail.is_empty() {
                let season = season_digits.parse::<u16>().ok()?;
                let episode = episode_digits.parse::<u16>().ok()?;
                if season <= 99 && episode <= 999 {
                    return Some((Some(season), episode));
                }
            }
        }
    }

    None
}

/// `Season 1 Episode 2`, and the `S01` / `E02` pair when written apart.
/// Returns how many tokens were consumed so the caller can skip them.
fn spelled_out_season_episode(
    tokens: &[&str],
    index: usize,
) -> Option<(Option<u16>, Option<u16>, usize)> {
    let first = tokens.get(index)?;
    let lower = trim_brackets(first).to_ascii_lowercase();

    if lower == "season" {
        let season = tokens.get(index + 1).and_then(|t| bare_number(t))?;
        let has_episode = tokens
            .get(index + 2)
            .map(|t| t.to_ascii_lowercase())
            .is_some_and(|t| t == "episode" || t == "ep");
        if has_episode {
            if let Some(episode) = tokens.get(index + 3).and_then(|t| bare_number(t)) {
                return Some((Some(season), Some(episode), 4));
            }
        }
        return Some((Some(season), None, 2));
    }

    // `S01` followed by `E02`.
    if let Some(rest) = lower.strip_prefix('s') {
        let (digits, tail) = take_digits(rest);
        if !digits.is_empty() && tail.is_empty() {
            let season = digits.parse::<u16>().ok()?;
            if season <= 99 {
                let next = tokens
                    .get(index + 1)
                    .map(|t| trim_brackets(t).to_ascii_lowercase());
                if let Some(next) = next {
                    if let Some(rest) = next.strip_prefix('e') {
                        let (episode_digits, tail) = take_digits(rest);
                        if !episode_digits.is_empty() && tail.is_empty() {
                            let episode = episode_digits.parse::<u16>().ok()?;
                            return Some((Some(season), Some(episode), 2));
                        }
                    }
                }
                return Some((Some(season), None, 1));
            }
        }
    }

    None
}

/// The anime-style `Title - 05` episode: a short bare number that follows a
/// standalone `-`. Four-digit numbers are years and are handled elsewhere, so
/// this only fires on one to three digits.
fn dashed_episode(tokens: &[&str], index: usize) -> Option<u16> {
    if *tokens.get(index)? != "-" {
        return None;
    }
    let next = tokens.get(index + 1)?;
    if next.len() > 3 {
        return None;
    }
    bare_number(next)
}

fn year(token: &str) -> Option<u16> {
    let token = trim_brackets(token);
    if token.len() != 4 || !token.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let value = token.parse::<u16>().ok()?;
    (1900..=2999).contains(&value).then_some(value)
}

fn bare_number(token: &str) -> Option<u16> {
    let token = trim_brackets(token);
    if token.is_empty() || !token.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    token.parse::<u16>().ok()
}

fn take_digits(text: &str) -> (&str, &str) {
    let end = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    (text.get(..end).unwrap_or(""), text.get(end..).unwrap_or(""))
}

fn trim_brackets(token: &str) -> &str {
    token.trim_matches(|c: char| matches!(c, '(' | ')' | '[' | ']' | '{' | '}' | ',' | '\''))
}

/// Tags that mark the end of a title. Not exhaustive by design — the list only
/// has to be good enough to find the *boundary*, since anything after it is
/// discarded either way.
const RELEASE_TAGS: &[&str] = &[
    // resolution / format
    "480p",
    "576p",
    "720p",
    "1080p",
    "1440p",
    "2160p",
    "4k",
    "8k",
    "uhd",
    "hd",
    "sd",
    "hdr",
    "hdr10",
    "sdr",
    "dv",
    "imax",
    "3d",
    "10bit",
    "8bit",
    "hi10p", // source
    "bluray",
    "blu",
    "bdrip",
    "brrip",
    "bdremux",
    "remux",
    "webrip",
    "webdl",
    "web",
    "hdtv",
    "pdtv",
    "dvdrip",
    "dvdscr",
    "dvd",
    "hdrip",
    "cam",
    "camrip",
    "ts",
    "telesync",
    "tc",
    "r5",
    "vodrip",
    "amzn",
    "nf",
    "hmax",
    "dsnp",
    "atvp",
    "hulu", // codec
    "x264",
    "x265",
    "h264",
    "h265",
    "hevc",
    "avc",
    "xvid",
    "divx",
    "av1",
    "vp9",
    // audio
    "aac",
    "aac2",
    "ac3",
    "eac3",
    "dts",
    "dtshd",
    "truehd",
    "atmos",
    "flac",
    "mp3",
    "opus",
    "dd5",
    "ddp5",
    "dd2",
    "ddp",
    "5",
    "7", // edition / status
    "proper",
    "repack",
    "extended",
    "unrated",
    "uncut",
    "remastered",
    "theatrical",
    "directors",
    "director",
    "cut",
    "limited",
    "internal",
    "complete",
    "multi",
    "dual",
    "subbed",
    "dubbed",
    "subs",
    "sub",
    "dub",
    "hardsub",
    "retail",
    "criterion",
    "anniversary",
];

fn is_release_tag(token: &str) -> bool {
    let token = trim_brackets(token).to_ascii_lowercase();
    if token.is_empty() {
        return false;
    }
    if RELEASE_TAGS.contains(&token.as_str()) {
        return true;
    }
    // `dd5.1` and `ddp5.1` survive tokenizing as `dd5` + `1`; the numeric half
    // is caught by the bare entries above. Channel layouts written together
    // (`5.1ch`, `7.1`) end up here.
    token
        .strip_suffix("ch")
        .is_some_and(|rest| rest.chars().all(|c| c.is_ascii_digit()) && !rest.is_empty())
}

fn clean_title(tokens: &[&str]) -> Option<String> {
    let joined = tokens
        .iter()
        .map(|token| trim_brackets(token))
        .filter(|token| !token.is_empty() && *token != "-")
        .collect::<Vec<_>>()
        .join(" ");

    let trimmed = joined
        .trim()
        .trim_matches(|c: char| c == '-' || c == '_' || c.is_whitespace());

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(name: &str) -> ParsedName {
        parse(name)
    }

    #[test]
    fn scene_movie_with_dots() {
        let p = t("Inception.2010.1080p.BluRay.x264-GROUP.mkv");
        assert_eq!(p.title.as_deref(), Some("Inception"));
        assert_eq!(p.year, Some(2010));
        assert_eq!(p.kind, MediaKind::Movie);
        assert_eq!(p.season, None);
    }

    #[test]
    fn multi_word_movie_title_survives() {
        let p = t("The.Grand.Budapest.Hotel.2014.1080p.mkv");
        assert_eq!(p.title.as_deref(), Some("The Grand Budapest Hotel"));
        assert_eq!(p.year, Some(2014));
    }

    #[test]
    fn parenthesised_year() {
        let p = t("Arrival (2016) [1080p].mkv");
        assert_eq!(p.title.as_deref(), Some("Arrival"));
        assert_eq!(p.year, Some(2016));
        assert_eq!(p.kind, MediaKind::Movie);
    }

    #[test]
    fn series_with_sxxeyy() {
        let p = t("Breaking.Bad.S01E02.720p.HDTV.x264.mkv");
        assert_eq!(p.title.as_deref(), Some("Breaking Bad"));
        assert_eq!(p.season, Some(1));
        assert_eq!(p.episode, Some(2));
        assert_eq!(p.kind, MediaKind::Series);
    }

    #[test]
    fn series_with_single_digit_sxex() {
        let p = t("Fringe.s3e11.HDTV.mkv");
        assert_eq!(p.season, Some(3));
        assert_eq!(p.episode, Some(11));
        assert_eq!(p.kind, MediaKind::Series);
    }

    #[test]
    fn series_with_nxnn() {
        let p = t("The.Office.3x07.DVDRip.avi");
        assert_eq!(p.title.as_deref(), Some("The Office"));
        assert_eq!(p.season, Some(3));
        assert_eq!(p.episode, Some(7));
    }

    #[test]
    fn resolution_is_not_mistaken_for_a_season() {
        let p = t("Movie.Name.2019.1920x1080.x264.mkv");
        assert_eq!(p.season, None);
        assert_eq!(p.episode, None);
        assert_eq!(p.kind, MediaKind::Movie);
    }

    #[test]
    fn spelled_out_season_and_episode() {
        let p = t("Chernobyl Season 1 Episode 3 1080p.mkv");
        assert_eq!(p.title.as_deref(), Some("Chernobyl"));
        assert_eq!(p.season, Some(1));
        assert_eq!(p.episode, Some(3));
        assert_eq!(p.kind, MediaKind::Series);
    }

    #[test]
    fn split_season_and_episode_tokens() {
        let p = t("Dark S02 E05 WEBRip.mkv");
        assert_eq!(p.title.as_deref(), Some("Dark"));
        assert_eq!(p.season, Some(2));
        assert_eq!(p.episode, Some(5));
    }

    #[test]
    fn anime_style_dashed_episode() {
        let p = t("[SubGroup] Steins Gate - 05 [1080p].mkv");
        assert_eq!(p.episode, Some(5));
        assert_eq!(p.season, None);
        assert_eq!(p.kind, MediaKind::Series);
    }

    #[test]
    fn hyphen_inside_a_title_is_kept() {
        let p = t("Spider-Man.2002.1080p.BluRay.mkv");
        assert_eq!(p.title.as_deref(), Some("Spider-Man"));
        assert_eq!(p.year, Some(2002));
    }

    #[test]
    fn hyphen_only_names_still_split() {
        let p = t("The-Matrix-1999-1080p-BluRay");
        assert_eq!(p.title.as_deref(), Some("The Matrix"));
        assert_eq!(p.year, Some(1999));
    }

    #[test]
    fn a_number_in_a_title_is_not_taken_as_the_year() {
        let p = t("Blade.Runner.2049.2017.1080p.WEB-DL.mkv");
        assert_eq!(p.title.as_deref(), Some("Blade Runner 2049"));
        assert_eq!(p.year, Some(2017));

        let p = t("2012.2009.1080p.BluRay.mkv");
        assert_eq!(p.title.as_deref(), Some("2012"));
        assert_eq!(p.year, Some(2009));
    }

    #[test]
    fn a_single_year_is_still_the_year() {
        let p = t("Inception.2010.1080p.mkv");
        assert_eq!(p.title.as_deref(), Some("Inception"));
        assert_eq!(p.year, Some(2010));
    }

    #[test]
    fn a_leading_group_tag_is_not_part_of_the_title() {
        let p = t("[SubGroup] Steins Gate - 05 [1080p].mkv");
        assert_eq!(p.title.as_deref(), Some("Steins Gate"));
        assert_eq!(p.episode, Some(5));
    }

    #[test]
    fn an_unterminated_bracket_is_left_alone() {
        let p = t("[Unclosed Steins Gate - 05 [1080p].mkv");
        assert_eq!(p.episode, Some(5));
    }

    #[test]
    fn a_season_without_an_episode_is_still_a_series() {
        let p = t("Dark.S02.COMPLETE.1080p.NF.WEBRip.mkv");
        assert_eq!(p.title.as_deref(), Some("Dark"));
        assert_eq!(p.season, Some(2));
        assert_eq!(p.episode, None);
        assert_eq!(p.kind, MediaKind::Series);

        let folder = t("Season 02");
        assert_eq!(folder.season, Some(2));
        assert_eq!(folder.kind, MediaKind::Series);
    }

    #[test]
    fn a_bare_word_is_unknown_not_a_title() {
        let p = t("video.mkv");
        assert_eq!(p.kind, MediaKind::Unknown);
        assert_eq!(p.year, None);
    }

    #[test]
    fn unknown_never_leaves_a_half_title_behind() {
        for name in ["video.mkv", "VID_20240115_143022.mp4", "untitled"] {
            let p = t(name);
            assert_eq!(p.kind, MediaKind::Unknown);
            assert_eq!(p.title, None, "{name} left a title behind");
        }
    }

    #[test]
    fn an_opaque_stream_name_is_unknown() {
        assert_eq!(t("stream.mkv").kind, MediaKind::Unknown);
        assert_eq!(t("a1b2c3d4e5f6.mp4").kind, MediaKind::Unknown);
        assert_eq!(t("").kind, MediaKind::Unknown);
    }

    #[test]
    fn episode_without_a_series_name_has_no_title() {
        let p = t("S01E02.mkv");
        assert_eq!(p.title, None);
        assert_eq!(p.episode, Some(2));
        assert_eq!(p.kind, MediaKind::Series);
        assert!(!p.is_usable(), "an episode number alone identifies nothing");
    }

    #[test]
    fn turkish_title_is_preserved() {
        let p = t("Ayla.Savasin.Kizi.2017.1080p.WEB-DL.mkv");
        assert_eq!(p.title.as_deref(), Some("Ayla Savasin Kizi"));
        assert_eq!(p.year, Some(2017));
    }

    #[test]
    fn non_ascii_title_is_preserved() {
        let p = t("Amélie.2001.1080p.BluRay.mkv");
        assert_eq!(p.title.as_deref(), Some("Amélie"));
    }

    #[test]
    fn a_year_like_number_out_of_range_is_not_a_year() {
        let p = t("Movie.1234.1080p.mkv");
        assert_eq!(p.year, None);
    }

    #[test]
    fn extension_is_dropped_but_a_tag_is_not() {
        assert_eq!(strip_extension("Movie.2010.1080p.mkv"), "Movie.2010.1080p");
        assert_eq!(strip_extension("Movie.2010.1080p"), "Movie.2010.1080p");
        assert_eq!(strip_extension("no_extension"), "no_extension");
    }

    #[test]
    fn fill_gaps_never_overwrites_title_or_kind() {
        let mut winner = ParsedName {
            title: Some("Breaking Bad".into()),
            year: None,
            season: None,
            episode: None,
            kind: MediaKind::Movie,
        };
        let lower = ParsedName {
            title: Some("Something Else".into()),
            year: Some(2008),
            season: Some(1),
            episode: Some(2),
            kind: MediaKind::Series,
        };
        winner.fill_gaps_from(&lower);

        assert_eq!(winner.title.as_deref(), Some("Breaking Bad"));
        assert_eq!(winner.kind, MediaKind::Movie);
        assert_eq!(winner.year, Some(2008));
        assert_eq!(winner.season, Some(1));
    }

    #[test]
    fn debug_prints_shape_not_values() {
        let p = t("Inception.2010.1080p.BluRay.mkv");
        let printed = format!("{p:?}");

        assert!(!printed.contains("Inception"), "title leaked: {printed}");
        assert!(!printed.contains("2010"), "year leaked: {printed}");
        assert!(printed.contains("Movie"));
        assert!(printed.contains("<present>"));
    }

    #[test]
    fn parsing_never_panics_on_hostile_input() {
        let hostile = [
            "....",
            "S..E..",
            "sE",
            "999999999999999999999x999999999999",
            "S99999E99999",
            "----",
            "[[[[]]]]",
            "\u{202e}gnp.4202.eivoM",
            "\0\0\0",
            &"a".repeat(10_000),
            &"S01E02.".repeat(1_000),
        ];
        for name in hostile {
            let _ = parse(name);
        }
    }
}
