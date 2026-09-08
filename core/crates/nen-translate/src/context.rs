//! Whole-document context extraction (ADR-0015 Karar 3).
//!
//! Purely local and deterministic — no provider call. Finds terms that read
//! as proper nouns (repeated, capitalized) so every block's prompt can carry
//! the same small, document-wide hint. M5's quality bar is structural, not
//! linguistic (S3, 2026-09-08): this module makes no claim about whether the
//! terms it finds are actually useful, only that the same document always
//! yields the same terms.
//!
//! **Security:** an extracted term is part of the subtitle dialogue, so it
//! falls under `docs/security-policy.md` §1 (K23 #4). [`ContextTerm`] and
//! [`DocumentContext`] therefore implement `Debug` by hand and print only
//! counts — never the term text itself, matching
//! `nen_domain::subtitle::Cue`. Do not replace those impls with
//! `#[derive(Debug)]`.

use std::collections::HashMap;
use std::fmt;

use nen_domain::subtitle::SubtitleDocument;

/// Upper bound on how many terms [`DocumentContext::of`] keeps (ADR-0015
/// Karar 3).
pub const CONTEXT_TERM_LIMIT: usize = 32;

/// A term repeated across the document, with how many times it was seen.
#[derive(Clone, PartialEq, Eq)]
pub struct ContextTerm {
    term: String,
    occurrences: usize,
}

impl ContextTerm {
    pub fn term(&self) -> &str {
        &self.term
    }

    pub const fn occurrences(&self) -> usize {
        self.occurrences
    }
}

/// Hand-written per `docs/security-policy.md` §1: prints the term's
/// **length**, never the term itself.
impl fmt::Debug for ContextTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContextTerm")
            .field("term_len", &self.term.chars().count())
            .field("occurrences", &self.occurrences)
            .finish()
    }
}

/// The small, structured context extracted from a whole [`SubtitleDocument`]
/// (ADR-0015 Karar 3). Enters every block's prompt unchanged.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct DocumentContext {
    terms: Vec<ContextTerm>,
}

impl DocumentContext {
    /// Extracts repeated, capitalized terms from `document`'s dialogue.
    ///
    /// A token is any maximal run of `char::is_alphanumeric` characters; a
    /// token is a **candidate** when its first character is
    /// `char::is_uppercase` and it is at least 2 characters long. A line's
    /// first token is "sentence position" — it alone proves nothing, since
    /// any word can open a sentence. A candidate becomes a term only once it
    /// has been seen at least 3 times **and** at least one of those sightings
    /// was outside sentence position. Terms are compared case-sensitively
    /// (`Tom` and `TOM` are different terms), sorted by descending
    /// occurrence count and then ascending alphabetically, and capped at
    /// [`CONTEXT_TERM_LIMIT`].
    ///
    /// Deterministic: the same document always yields the same terms in the
    /// same order. Writing systems without an uppercase/lowercase distinction
    /// (e.g. most CJK text) never produce a candidate, so the result is an
    /// empty set for them — a known limit of this rule, not a bug (ADR-0015
    /// Karar 3).
    pub fn of(document: &SubtitleDocument) -> Self {
        let mut counts: HashMap<&str, (usize, bool)> = HashMap::new();

        for cue in document.cues() {
            for line in cue.lines() {
                for (position, token) in line_tokens(line).into_iter().enumerate() {
                    if !is_candidate(token) {
                        continue;
                    }
                    let entry = counts.entry(token).or_insert((0, false));
                    entry.0 += 1;
                    if position > 0 {
                        entry.1 = true;
                    }
                }
            }
        }

        let mut terms: Vec<ContextTerm> = counts
            .into_iter()
            .filter(|(_, (total, seen_outside_start))| *total >= 3 && *seen_outside_start)
            .map(|(term, (occurrences, _))| ContextTerm {
                term: term.to_string(),
                occurrences,
            })
            .collect();

        terms.sort_by(|a, b| {
            b.occurrences
                .cmp(&a.occurrences)
                .then_with(|| a.term.cmp(&b.term))
        });
        terms.truncate(CONTEXT_TERM_LIMIT);

        Self { terms }
    }

    pub fn terms(&self) -> &[ContextTerm] {
        &self.terms
    }
}

/// Hand-written per `docs/security-policy.md` §1: prints how many terms were
/// found, never the terms themselves.
impl fmt::Debug for DocumentContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DocumentContext")
            .field("term_count", &self.terms.len())
            .finish()
    }
}

/// A candidate is a token whose first character is uppercase and which is at
/// least 2 characters long.
fn is_candidate(token: &str) -> bool {
    let mut chars = token.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_uppercase() && chars.next().is_some()
}

/// Splits `line` into maximal runs of `char::is_alphanumeric` characters, in
/// order. Never panics: slices only at boundaries `char_indices` itself
/// produced, via `str::get` rather than direct indexing.
fn line_tokens(line: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start: Option<usize> = None;

    for (index, ch) in line.char_indices() {
        if ch.is_alphanumeric() {
            start.get_or_insert(index);
        } else if let Some(token_start) = start.take() {
            if let Some(token) = line.get(token_start..index) {
                tokens.push(token);
            }
        }
    }
    if let Some(token_start) = start {
        if let Some(token) = line.get(token_start..) {
            tokens.push(token);
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use nen_domain::subtitle::{Cue, CueId, TimeSpan};

    use super::*;

    fn document_of(lines: &[&[&str]]) -> SubtitleDocument {
        let cues = lines
            .iter()
            .enumerate()
            .map(|(i, cue_lines)| {
                let start = (i as u32) * 1_000;
                Cue::new(
                    CueId::new(i as u32 + 1),
                    TimeSpan::new(start, start + 900).expect("well-formed span"),
                    cue_lines.iter().map(|line| line.to_string()).collect(),
                )
            })
            .collect();
        SubtitleDocument::new(cues)
    }

    #[test]
    fn finds_a_term_repeated_outside_sentence_position() {
        let document = document_of(&[
            &["Gon looked at Killua."],
            &["Killua smiled back."],
            &["Then Killua left."],
        ]);
        let context = DocumentContext::of(&document);
        let names: Vec<&str> = context.terms().iter().map(ContextTerm::term).collect();
        assert_eq!(names, vec!["Killua"]);
    }

    #[test]
    fn a_term_seen_only_at_sentence_start_is_not_kept() {
        // "Gon" opens every line here and never appears mid-sentence, so a
        // capital letter alone never proves it is a proper noun.
        let document = document_of(&[&["Gon ran."], &["Gon jumped."], &["Gon won."]]);
        let context = DocumentContext::of(&document);
        assert!(context.terms().is_empty());
    }

    #[test]
    fn a_term_seen_fewer_than_three_times_is_not_kept() {
        let document = document_of(&[&["Hello Killua."], &["Hi Killua."]]);
        let context = DocumentContext::of(&document);
        assert!(context.terms().is_empty());
    }

    #[test]
    fn terms_are_case_sensitive() {
        let document = document_of(&[
            &["Hello Killua."],
            &["Hi Killua."],
            &["Bye Killua."],
            &["Hello KILLUA."],
            &["Hi KILLUA."],
            &["Bye KILLUA."],
        ]);
        let context = DocumentContext::of(&document);
        let names: Vec<&str> = context.terms().iter().map(ContextTerm::term).collect();
        assert_eq!(names, vec!["KILLUA", "Killua"]);
    }

    #[test]
    fn sorted_by_descending_count_then_alphabetically() {
        let document = document_of(&[
            &["Hi Zebra Apple."],
            &["Hi Zebra Apple."],
            &["Hi Zebra Apple."],
            &["Hi Apple."],
        ]);
        let context = DocumentContext::of(&document);
        let names: Vec<&str> = context.terms().iter().map(ContextTerm::term).collect();
        // Apple: 4 occurrences, Zebra: 3 — count wins over alphabetical order.
        assert_eq!(names, vec!["Apple", "Zebra"]);
    }

    #[test]
    fn extraction_is_deterministic() {
        let document = document_of(&[
            &["Hi Killua, meet Gon."],
            &["Killua and Gon are friends."],
            &["Gon trusts Killua."],
        ]);
        let first = DocumentContext::of(&document);
        let second = DocumentContext::of(&document);
        assert_eq!(first, second);
    }

    #[test]
    fn caps_at_the_term_limit() {
        // 40 distinct terms, each capitalized, each repeated 3 times with a
        // mid-sentence sighting — more than CONTEXT_TERM_LIMIT (32).
        let mut lines: Vec<String> = Vec::new();
        for n in 0..40 {
            let term = format!("Term{n:02}");
            for _ in 0..3 {
                lines.push(format!("hello {term} and {term}"));
            }
        }
        let line_refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let cue_lines: Vec<&[&str]> = line_refs.iter().map(std::slice::from_ref).collect();
        let document = document_of(&cue_lines);

        let context = DocumentContext::of(&document);
        assert_eq!(context.terms().len(), CONTEXT_TERM_LIMIT);
    }

    #[test]
    fn writing_without_case_distinction_yields_no_candidates() {
        let document = document_of(&[&["你好 你好 你好"], &["你好 你好"]]);
        let context = DocumentContext::of(&document);
        assert!(context.terms().is_empty());
    }

    #[test]
    fn debug_never_prints_the_term_text() {
        let document = document_of(&[
            &["Secret Killua dialogue."],
            &["More Killua dialogue."],
            &["Even more Killua dialogue."],
        ]);
        let context = DocumentContext::of(&document);
        let printed = format!("{context:?}");
        assert!(!printed.contains("Killua"), "{printed}");
        assert!(printed.contains("term_count: 1"), "{printed}");

        let term = context.terms().first().expect("one term expected");
        let printed_term = format!("{term:?}");
        assert!(!printed_term.contains("Killua"), "{printed_term}");
        assert!(printed_term.contains("occurrences: 3"), "{printed_term}");
    }
}
