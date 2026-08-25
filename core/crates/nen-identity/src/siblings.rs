//! Agreement with the files sitting next to this one (ADR-0009 Karar 6,
//! layer 6).
//!
//! A single `S01E02` in a filename can be a coincidence; twelve of them in one
//! folder are a season. This layer reads that agreement.
//!
//! # It reports, it does not overrule
//!
//! ADR-0009 gives the winning layer ownership of the identity, so nothing here
//! demotes a parse or replaces a title that a higher layer established. The
//! two things it does are strictly additive:
//!
//! * [`corroborate`] says whether the neighbours agree, for a later confidence
//!   model to weigh (NEN-035) — it returns a verdict, it does not mutate.
//! * [`consensus_title`] offers a series name when the file itself has an
//!   episode number but no name (`S01E02.mkv`), which is a gap, not a conflict.
//!
//! The narrower reading is deliberate. Letting a majority overrule a parse
//! would mean one oddly-named folder could rewrite a correctly identified
//! file, and that trade is worse than the false `SxxEyy` it would catch.

use std::collections::HashMap;

use crate::release_name::{parse, MediaKind, ParsedName};

/// Most neighbours we will read. A media folder has tens of files; anything
/// past this is either not a media folder or not worth the work.
const MAX_SIBLINGS: usize = 256;

/// What the neighbouring files say about a parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corroboration {
    /// Nothing to compare against — a single-file folder, or no series parse.
    NotApplicable,
    /// At least one neighbour is the same series, and no neighbour disagrees
    /// about the season.
    Confirmed,
    /// Neighbours exist and parse as series, but none matches this one.
    Unconfirmed,
}

/// Checks `parsed` against the names of the files beside it.
///
/// Only meaningful for a series parse; a movie folder tells us nothing, so it
/// reports [`Corroboration::NotApplicable`].
pub fn corroborate(parsed: &ParsedName, siblings: &[String]) -> Corroboration {
    if parsed.kind != MediaKind::Series {
        return Corroboration::NotApplicable;
    }
    let Some(title) = parsed.title.as_deref() else {
        return Corroboration::NotApplicable;
    };

    let mut saw_series = false;
    let mut agreed = false;

    for sibling in siblings.iter().take(MAX_SIBLINGS) {
        let other = parse(sibling);
        if other.kind != MediaKind::Series {
            continue;
        }
        saw_series = true;

        let same_title = other
            .title
            .as_deref()
            .is_some_and(|other_title| eq_loose(other_title, title));
        let season_agrees = match (parsed.season, other.season) {
            (Some(a), Some(b)) => a == b,
            _ => true,
        };

        if same_title && season_agrees {
            agreed = true;
        }
    }

    if !saw_series {
        Corroboration::NotApplicable
    } else if agreed {
        Corroboration::Confirmed
    } else {
        Corroboration::Unconfirmed
    }
}

/// The series title the neighbours agree on, if they agree at all.
///
/// Used only to fill a title the file itself does not have. Requires a strict
/// majority of the series-looking neighbours, so a folder with two unrelated
/// shows in it contributes nothing rather than picking one arbitrarily.
pub fn consensus_title(siblings: &[String]) -> Option<String> {
    let mut counts: HashMap<String, (usize, String)> = HashMap::new();
    let mut total = 0usize;

    for sibling in siblings.iter().take(MAX_SIBLINGS) {
        let other = parse(sibling);
        if other.kind != MediaKind::Series {
            continue;
        }
        let Some(title) = other.title else {
            continue;
        };
        total += 1;
        let entry = counts
            .entry(normalize(&title))
            .or_insert((0, title.clone()));
        entry.0 += 1;
    }

    if total == 0 {
        return None;
    }

    let (count, title) = counts.into_values().max_by_key(|(count, _)| *count)?;
    (count * 2 > total).then_some(title)
}

/// Compares titles ignoring case and separator noise, so `Breaking.Bad` and
/// `Breaking Bad` count as the same show.
fn eq_loose(a: &str, b: &str) -> bool {
    normalize(a) == normalize(b)
}

fn normalize(title: &str) -> String {
    title
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn neighbours_from_the_same_season_confirm() {
        let parsed = parse("Breaking.Bad.S01E02.720p.mkv");
        let siblings = names(&[
            "Breaking.Bad.S01E01.720p.mkv",
            "Breaking.Bad.S01E03.720p.mkv",
        ]);
        assert_eq!(corroborate(&parsed, &siblings), Corroboration::Confirmed);
    }

    #[test]
    fn separator_differences_do_not_break_agreement() {
        let parsed = parse("Breaking.Bad.S01E02.mkv");
        let siblings = names(&["Breaking Bad - S01E01.mkv"]);
        assert_eq!(corroborate(&parsed, &siblings), Corroboration::Confirmed);
    }

    #[test]
    fn a_different_show_does_not_confirm() {
        let parsed = parse("Breaking.Bad.S01E02.mkv");
        let siblings = names(&["The.Wire.S01E01.mkv", "The.Wire.S01E02.mkv"]);
        assert_eq!(corroborate(&parsed, &siblings), Corroboration::Unconfirmed);
    }

    #[test]
    fn a_movie_parse_is_not_applicable() {
        let parsed = parse("Inception.2010.1080p.mkv");
        let siblings = names(&["Breaking.Bad.S01E01.mkv"]);
        assert_eq!(
            corroborate(&parsed, &siblings),
            Corroboration::NotApplicable
        );
    }

    #[test]
    fn an_empty_folder_is_not_applicable() {
        let parsed = parse("Breaking.Bad.S01E02.mkv");
        assert_eq!(corroborate(&parsed, &[]), Corroboration::NotApplicable);
    }

    #[test]
    fn corroboration_never_changes_the_parse() {
        let parsed = parse("Breaking.Bad.S01E02.mkv");
        let before = parsed.clone();
        let _ = corroborate(&parsed, &names(&["The.Wire.S01E01.mkv"]));
        assert_eq!(parsed, before, "corroborate must not mutate its input");
    }

    #[test]
    fn a_majority_supplies_a_missing_title() {
        let siblings = names(&[
            "Breaking.Bad.S01E01.mkv",
            "Breaking.Bad.S01E03.mkv",
            "The.Wire.S01E01.mkv",
        ]);
        assert_eq!(consensus_title(&siblings).as_deref(), Some("Breaking Bad"));
    }

    #[test]
    fn a_tie_supplies_nothing() {
        let siblings = names(&["Breaking.Bad.S01E01.mkv", "The.Wire.S01E01.mkv"]);
        assert_eq!(consensus_title(&siblings), None);
    }

    #[test]
    fn a_folder_of_movies_supplies_nothing() {
        let siblings = names(&["Inception.2010.mkv", "Arrival.2016.mkv"]);
        assert_eq!(consensus_title(&siblings), None);
    }

    #[test]
    fn hostile_and_huge_input_never_panics() {
        let many: Vec<String> = (0..10_000)
            .map(|i| format!("Show.S01E{i:02}.mkv"))
            .collect();
        let parsed = parse("Show.S01E01.mkv");
        let _ = corroborate(&parsed, &many);
        let _ = consensus_title(&many);

        let junk = names(&["", "....", "\u{202e}", "\0"]);
        let _ = corroborate(&parsed, &junk);
        let _ = consensus_title(&junk);
    }
}
