//! Subtitle value types: cue identity, timing and the document that holds them.
//!
//! These are pure values — no I/O, no parsing. `nen-subtitle` builds them from
//! SRT input (NEN-013); `nen-catalog`, `nen-translate` and `nen-sync` consume
//! them later.
//!
//! **Security:** a [`Cue`] carries subtitle dialogue, which
//! `docs/security-policy.md` §1 (K23 #4) forbids from ever reaching a log.
//! `Cue` and [`SubtitleDocument`] therefore implement `Debug` by hand and
//! print only counts and timings — never the text itself. Do not replace
//! those impls with `#[derive(Debug)]`.

use std::fmt;

/// Largest timestamp expressible in SRT's `HH:MM:SS,mmm` form (`99:59:59,999`).
pub const MAX_TIMESTAMP_MS: u32 = 359_999_999;

/// Identifier of a cue within its source document.
///
/// For SRT this is the block index as written in the file. Per ADR-0007
/// (NEN-016) this stays document-local — there is no separate stable,
/// cross-source cue identity; cross-source/cross-translation matching is done
/// at the whole-document fingerprint level (`nen_subtitle::fingerprint`), not
/// per cue. A `CueId` only ever means "the n-th cue of this document, as the
/// source numbered it".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CueId(u32);

impl CueId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for CueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Why a start/end pair could not become a [`TimeSpan`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSpanError {
    /// `end` is strictly before `start`.
    EndBeforeStart,
    /// `end` equals `start`; a cue that is displayed for zero milliseconds is
    /// rejected rather than silently kept.
    ZeroDuration,
}

impl fmt::Display for TimeSpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EndBeforeStart => f.write_str("end time is before start time"),
            Self::ZeroDuration => f.write_str("end time equals start time"),
        }
    }
}

impl std::error::Error for TimeSpanError {}

/// A half-open display interval in milliseconds, valid by construction:
/// `start_ms < end_ms` always holds.
///
/// Callers downstream (`nen_subtitle::fingerprint`, NEN-017 lookup) may rely
/// on that invariant without re-checking it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeSpan {
    start_ms: u32,
    end_ms: u32,
}

impl TimeSpan {
    pub fn new(start_ms: u32, end_ms: u32) -> Result<Self, TimeSpanError> {
        if end_ms < start_ms {
            return Err(TimeSpanError::EndBeforeStart);
        }
        if end_ms == start_ms {
            return Err(TimeSpanError::ZeroDuration);
        }
        Ok(Self { start_ms, end_ms })
    }

    pub const fn start_ms(self) -> u32 {
        self.start_ms
    }

    pub const fn end_ms(self) -> u32 {
        self.end_ms
    }

    /// Never overflows: `end_ms > start_ms` is guaranteed by [`TimeSpan::new`].
    pub const fn duration_ms(self) -> u32 {
        self.end_ms - self.start_ms
    }

    /// True when the two spans share at least one millisecond.
    ///
    /// Overlap is *not* an error in this core: SRT files with simultaneous
    /// speakers are legitimate. Consumers that care (renderer, sync) decide
    /// what to do about it.
    pub const fn overlaps(self, other: Self) -> bool {
        self.start_ms < other.end_ms && other.start_ms < self.end_ms
    }
}

/// One subtitle cue: an identifier, a display interval and its text lines.
///
/// Line structure is preserved rather than joined into a single string, so a
/// writer (NEN-014) can reproduce the original layout byte for byte.
#[derive(Clone, PartialEq, Eq)]
pub struct Cue {
    id: CueId,
    span: TimeSpan,
    lines: Vec<String>,
}

impl Cue {
    pub fn new(id: CueId, span: TimeSpan, lines: Vec<String>) -> Self {
        Self { id, span, lines }
    }

    pub const fn id(&self) -> CueId {
        self.id
    }

    pub const fn span(&self) -> TimeSpan {
        self.span
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

/// Hand-written per `docs/security-policy.md` §1: prints the line **count**,
/// never the dialogue.
impl fmt::Debug for Cue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cue")
            .field("id", &self.id)
            .field("span", &self.span)
            .field("line_count", &self.lines.len())
            .finish()
    }
}

/// A parsed subtitle document — for now just its cues, in source order.
///
/// `nen_subtitle::fingerprint` computes the timeline/source fingerprint from
/// this (ADR-0007, NEN-016); NEN-017 adds indexed lookup. Nothing here
/// decides those, so nothing here should be read as fixing their design.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct SubtitleDocument {
    cues: Vec<Cue>,
}

impl SubtitleDocument {
    pub fn new(cues: Vec<Cue>) -> Self {
        Self { cues }
    }

    pub fn cues(&self) -> &[Cue] {
        &self.cues
    }

    pub fn len(&self) -> usize {
        self.cues.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cues.is_empty()
    }
}

/// Hand-written per `docs/security-policy.md` §1: cue **count** is loggable,
/// cue text is not.
impl fmt::Debug for SubtitleDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubtitleDocument")
            .field("cue_count", &self.cues.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_span_rejects_end_before_start() {
        assert_eq!(
            TimeSpan::new(2_000, 1_000),
            Err(TimeSpanError::EndBeforeStart)
        );
    }

    #[test]
    fn time_span_rejects_zero_duration() {
        assert_eq!(
            TimeSpan::new(1_000, 1_000),
            Err(TimeSpanError::ZeroDuration)
        );
    }

    #[test]
    fn time_span_exposes_duration_and_overlap() {
        let a = TimeSpan::new(1_000, 3_000).unwrap();
        let b = TimeSpan::new(2_500, 4_000).unwrap();
        let c = TimeSpan::new(3_000, 4_000).unwrap();
        assert_eq!(a.duration_ms(), 2_000);
        assert!(a.overlaps(b));
        assert!(b.overlaps(a));
        assert!(!a.overlaps(c), "touching spans do not overlap");
    }

    #[test]
    fn max_timestamp_matches_srt_upper_bound() {
        let expected = 99 * 3_600_000 + 59 * 60_000 + 59 * 1_000 + 999;
        assert_eq!(MAX_TIMESTAMP_MS, expected);
    }

    #[test]
    fn cue_debug_prints_line_count_not_dialogue() {
        let cue = Cue::new(
            CueId::new(1),
            TimeSpan::new(0, 1_000).unwrap(),
            vec!["Do not log me".to_string(), "nor me".to_string()],
        );
        let printed = format!("{cue:?}");
        assert!(!printed.contains("Do not log me"), "{printed}");
        assert!(!printed.contains("nor me"), "{printed}");
        assert!(printed.contains("line_count: 2"), "{printed}");
    }

    #[test]
    fn document_debug_prints_cue_count_not_dialogue() {
        let doc = SubtitleDocument::new(vec![Cue::new(
            CueId::new(1),
            TimeSpan::new(0, 1_000).unwrap(),
            vec!["Do not log me".to_string()],
        )]);
        let printed = format!("{doc:?}");
        assert!(!printed.contains("Do not log me"), "{printed}");
        assert!(printed.contains("cue_count: 1"), "{printed}");
    }
}
