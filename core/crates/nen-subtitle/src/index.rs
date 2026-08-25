//! Time-to-cue lookup that does not scan the whole document (NEN-017).
//!
//! `docs/product-spec.md` §14 forbids a linear full-list scan for cue lookup:
//! after a seek the right cue has to appear immediately, whatever the document
//! size. [`CueIndex`] answers "what is on screen at `t`" with two binary
//! searches plus the cues it actually returns.
//!
//! **Overlap is part of the contract.** NEN-013 accepts overlapping cues —
//! simultaneous speakers are legitimate SRT — so more than one cue can be
//! active at a single millisecond. [`CueIndex::active_cues`] returns all of
//! them in document order; this layer never silently drops one. What to do
//! with a stacked cue is the renderer's decision (NEN-027).
//!
//! # Structure, and why this one
//!
//! A [`SubtitleDocument`] is already ordered by `start_ms` (NEN-013 rejects
//! non-monotonic cues), so the only auxiliary state needed is one prefix
//! array: `max_end_prefix[i]` is the largest `end_ms` among cues `0..=i`. That
//! array is non-decreasing, which makes it binary-searchable, and it bounds
//! how far back a query has to look: no cue before the first index whose
//! prefix maximum exceeds `t` can still be on screen at `t`.
//!
//! The alternative considered was a boundary-event structure — the distinct
//! cue boundaries as segments, each storing the cues active inside it. Its
//! queries are `O(log n + k)` even for pathological documents, but its memory
//! is not: `n` mutually overlapping cues produce `O(n²)` entries. This crate
//! parses untrusted input (`docs/security-policy.md` §2) and
//! [`crate::encoding`]'s 10 MiB cap admits documents with ~100k cues, so a
//! hostile file exhausting memory is a far worse failure mode than a
//! pathological file making one query slow. The prefix array stays `O(n)` in
//! memory no matter what the input looks like.
//!
//! The cost of that choice, stated plainly: the candidate window a query has
//! to filter grows with overlap depth, so a document containing one
//! feature-length cue degrades that query towards a scan. Real subtitles have
//! a depth of one to three — see the NEN-017 benchmark record — and
//! correctness under deep overlap is covered by tests either way.

use std::ops::Range;

use nen_domain::subtitle::{Cue, SubtitleDocument};

/// Indexed time lookup over a borrowed [`SubtitleDocument`].
///
/// Borrows rather than clones: the caller (a renderer, NEN-027) is holding the
/// document anyway, and cue text is the one thing this codebase copies as
/// little as possible.
pub struct CueIndex<'a> {
    document: &'a SubtitleDocument,
    /// `max_end_prefix[i] == max(cues[0..=i].span().end_ms())`. Non-decreasing
    /// by construction, which is what makes it binary-searchable.
    max_end_prefix: Vec<u32>,
}

impl<'a> CueIndex<'a> {
    /// Builds the index in `O(n)` over a document already ordered by start.
    pub fn build(document: &'a SubtitleDocument) -> Self {
        let mut max_end_prefix = Vec::with_capacity(document.len());
        let mut running_max = 0;
        for cue in document.cues() {
            running_max = running_max.max(cue.span().end_ms());
            max_end_prefix.push(running_max);
        }
        Self {
            document,
            max_end_prefix,
        }
    }

    /// Every cue on screen at `at_ms`, in document order.
    ///
    /// A cue is on screen when `start_ms <= at_ms < end_ms` — the same
    /// half-open reading as [`TimeSpan`](nen_domain::subtitle::TimeSpan), so a
    /// cue ending exactly at `at_ms` has already left. Empty when the moment
    /// falls in a gap or outside the document; that is not an error.
    pub fn active_cues(&self, at_ms: u32) -> Vec<&'a Cue> {
        self.cues_overlapping(at_ms, at_ms.saturating_add(1))
    }

    /// The first cue on screen at `at_ms` in document order, if any.
    ///
    /// Convenience over [`CueIndex::active_cues`] for callers that genuinely
    /// want one cue. It discards the others, so a renderer that can stack
    /// simultaneous speakers should ask for the full set instead.
    pub fn active_cue(&self, at_ms: u32) -> Option<&'a Cue> {
        self.active_cues(at_ms).into_iter().next()
    }

    /// Every cue overlapping the half-open window `range`, in document order.
    ///
    /// A cue qualifies when it shares at least one millisecond with the
    /// window. An empty or reversed range returns nothing.
    pub fn cues_in(&self, range: Range<u32>) -> Vec<&'a Cue> {
        self.cues_overlapping(range.start, range.end)
    }

    /// Shared body of both queries: cues with `start_ms < to` and
    /// `end_ms > from`.
    fn cues_overlapping(&self, from: u32, to: u32) -> Vec<&'a Cue> {
        if to <= from {
            return Vec::new();
        }
        let cues = self.document.cues();
        // Cues starting at or after the window's end cannot overlap it.
        let hi = cues.partition_point(|cue| cue.span().start_ms() < to);
        // ...and before this point every cue has already ended.
        let lo = self.max_end_prefix.partition_point(|&end| end <= from);

        cues.get(lo..hi)
            .unwrap_or_default()
            .iter()
            .filter(|cue| cue.span().end_ms() > from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nen_domain::subtitle::{CueId, TimeSpan};

    /// `(start_ms, end_ms)` pairs become a document, numbered in order.
    fn document(spans: &[(u32, u32)]) -> SubtitleDocument {
        let cues = spans
            .iter()
            .enumerate()
            .map(|(i, &(start, end))| {
                Cue::new(
                    CueId::new(i as u32 + 1),
                    TimeSpan::new(start, end).unwrap(),
                    vec![format!("line {i}")],
                )
            })
            .collect();
        SubtitleDocument::new(cues)
    }

    fn ids(cues: Vec<&Cue>) -> Vec<u32> {
        cues.into_iter().map(|cue| cue.id().get()).collect()
    }

    #[test]
    fn empty_document_has_nothing_active() {
        let doc = document(&[]);
        let index = CueIndex::build(&doc);
        assert!(index.active_cues(0).is_empty());
        assert!(index.active_cues(u32::MAX).is_empty());
        assert_eq!(index.active_cue(1_000), None);
        assert!(index.cues_in(0..u32::MAX).is_empty());
    }

    #[test]
    fn single_cue_is_active_only_inside_its_span() {
        let doc = document(&[(1_000, 2_000)]);
        let index = CueIndex::build(&doc);
        assert_eq!(ids(index.active_cues(999)), Vec::<u32>::new());
        assert_eq!(ids(index.active_cues(1_000)), vec![1]);
        assert_eq!(ids(index.active_cues(1_999)), vec![1]);
        assert_eq!(ids(index.active_cues(2_000)), Vec::<u32>::new());
    }

    #[test]
    fn boundaries_are_half_open() {
        let doc = document(&[(1_000, 2_000), (2_000, 3_000)]);
        let index = CueIndex::build(&doc);
        // The moment one cue ends is the moment the next one starts, and only
        // the later cue is on screen.
        assert_eq!(ids(index.active_cues(2_000)), vec![2]);
    }

    #[test]
    fn moments_before_after_and_between_cues_are_empty() {
        let doc = document(&[(1_000, 2_000), (5_000, 6_000)]);
        let index = CueIndex::build(&doc);
        assert!(index.active_cues(0).is_empty(), "before the first cue");
        assert!(index.active_cues(3_500).is_empty(), "in the gap");
        assert!(index.active_cues(9_000).is_empty(), "after the last cue");
    }

    #[test]
    fn overlapping_cues_are_all_returned_in_document_order() {
        // Two speakers talking over each other, plus a third that starts
        // before the second ends.
        let doc = document(&[(1_000, 4_000), (2_000, 5_000), (3_000, 3_500)]);
        let index = CueIndex::build(&doc);
        assert_eq!(ids(index.active_cues(1_500)), vec![1]);
        assert_eq!(ids(index.active_cues(2_500)), vec![1, 2]);
        assert_eq!(ids(index.active_cues(3_200)), vec![1, 2, 3]);
        assert_eq!(ids(index.active_cues(4_500)), vec![2]);
    }

    #[test]
    fn a_long_cue_stays_visible_behind_later_ones() {
        // The pathological shape for the prefix-array structure: cue 1 spans
        // the whole document. Correctness must not depend on overlap depth.
        let doc = document(&[(0, 100_000), (10_000, 11_000), (90_000, 91_000)]);
        let index = CueIndex::build(&doc);
        assert_eq!(ids(index.active_cues(10_500)), vec![1, 2]);
        assert_eq!(ids(index.active_cues(50_000)), vec![1]);
        assert_eq!(ids(index.active_cues(90_500)), vec![1, 3]);
        assert_eq!(ids(index.active_cues(100_000)), Vec::<u32>::new());
    }

    #[test]
    fn range_query_returns_every_overlapping_cue() {
        let doc = document(&[(1_000, 2_000), (5_000, 6_000), (5_500, 9_000)]);
        let index = CueIndex::build(&doc);
        assert_eq!(ids(index.cues_in(0..1_000)), Vec::<u32>::new());
        assert_eq!(ids(index.cues_in(0..1_001)), vec![1]);
        assert_eq!(ids(index.cues_in(1_500..5_600)), vec![1, 2, 3]);
        assert_eq!(ids(index.cues_in(6_000..7_000)), vec![3]);
        assert_eq!(ids(index.cues_in(0..u32::MAX)), vec![1, 2, 3]);
    }

    #[test]
    fn empty_and_reversed_ranges_return_nothing() {
        let doc = document(&[(1_000, 2_000)]);
        let index = CueIndex::build(&doc);
        assert!(index.cues_in(1_500..1_500).is_empty());
        #[allow(clippy::reversed_empty_ranges)]
        let reversed = index.cues_in(2_000..1_000);
        assert!(reversed.is_empty());
    }

    #[test]
    fn lookup_at_the_maximum_timestamp_does_not_overflow() {
        let doc = document(&[(u32::MAX - 1, u32::MAX)]);
        let index = CueIndex::build(&doc);
        assert_eq!(ids(index.active_cues(u32::MAX - 1)), vec![1]);
        // `at_ms + 1` saturates here rather than wrapping to zero.
        assert_eq!(ids(index.active_cues(u32::MAX)), Vec::<u32>::new());
    }

    #[test]
    fn prefix_maximum_is_non_decreasing() {
        let doc = document(&[(0, 9_000), (1_000, 2_000), (3_000, 4_000)]);
        let index = CueIndex::build(&doc);
        assert_eq!(index.max_end_prefix, vec![9_000, 9_000, 9_000]);
    }
}
