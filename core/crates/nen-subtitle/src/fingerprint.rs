//! Timeline and source fingerprints (ADR-0007, NEN-016).
//!
//! A [`SubtitleDocument`] is fingerprinted two ways: [`TimelineFingerprint`]
//! sees only cue timing, [`SourceFingerprint`] sees timing and text. Two
//! documents that are the same timeline with different dialogue (translations
//! of one source, say) share a `TimelineFingerprint` but not a
//! `SourceFingerprint` — that split is what lets `docs/product-spec.md` §12's
//! `SyncProfile` survive a translation while §11's cache identity still keys
//! on the actual text.
//!
//! Both hash with BLAKE3 over a fixed byte encoding, not the document's `Vec`
//! layout directly, so the result is stable across process runs and platforms
//! (needed once these become persisted cache/session keys). `CueId` never
//! enters either hash — see ADR-0007 for why.

use nen_domain::subtitle::SubtitleDocument;

/// BLAKE3 digest of a document's cue timings only. See the module docs for
/// why text and [`CueId`](nen_domain::subtitle::CueId) are excluded.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimelineFingerprint([u8; 32]);

/// BLAKE3 digest of a document's cue timings **and** text.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceFingerprint([u8; 32]);

impl TimelineFingerprint {
    /// Encoding (ADR-0007): `u32` LE cue count, then per cue in document
    /// order `start_ms: u32 LE` + `end_ms: u32 LE`.
    pub fn of(document: &SubtitleDocument) -> Self {
        let mut input = Vec::new();
        push_cue_count(&mut input, document);
        for cue in document.cues() {
            push_timing(&mut input, cue.span().start_ms(), cue.span().end_ms());
        }
        Self(*blake3::hash(&input).as_bytes())
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl SourceFingerprint {
    /// Same timing encoding as [`TimelineFingerprint::of`], with each cue
    /// additionally carrying a `u32` LE line count and then, per line, a
    /// `u32` LE byte length followed by its UTF-8 bytes. The line count and
    /// per-line length prefixes make the encoding unambiguous: without them
    /// two different line splits (`["ab", "c"]` vs. `["a", "bc"]`) could
    /// concatenate to the same bytes and collide.
    pub fn of(document: &SubtitleDocument) -> Self {
        let mut input = Vec::new();
        push_cue_count(&mut input, document);
        for cue in document.cues() {
            push_timing(&mut input, cue.span().start_ms(), cue.span().end_ms());
            push_u32(&mut input, cue.lines().len());
            for line in cue.lines() {
                push_u32(&mut input, line.len());
                input.extend_from_slice(line.as_bytes());
            }
        }
        Self(*blake3::hash(&input).as_bytes())
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Encodes `value` as `u32` LE, saturating rather than panicking — no
/// realistic document approaches `u32::MAX` cues or a `u32::MAX`-byte line,
/// and this crate denies `unwrap`/`expect` outside tests.
fn push_u32(input: &mut Vec<u8>, value: usize) {
    let value = u32::try_from(value).unwrap_or(u32::MAX);
    input.extend_from_slice(&value.to_le_bytes());
}

fn push_cue_count(input: &mut Vec<u8>, document: &SubtitleDocument) {
    push_u32(input, document.len());
}

fn push_timing(input: &mut Vec<u8>, start_ms: u32, end_ms: u32) {
    input.extend_from_slice(&start_ms.to_le_bytes());
    input.extend_from_slice(&end_ms.to_le_bytes());
}

fn fmt_hex(bytes: &[u8; 32], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for byte in bytes {
        write!(f, "{byte:02x}")?;
    }
    Ok(())
}

impl std::fmt::Display for TimelineFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_hex(&self.0, f)
    }
}

impl std::fmt::Debug for TimelineFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimelineFingerprint({self})")
    }
}

impl std::fmt::Display for SourceFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_hex(&self.0, f)
    }
}

impl std::fmt::Debug for SourceFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SourceFingerprint({self})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nen_domain::subtitle::{Cue, CueId, TimeSpan};

    fn doc(cues: Vec<(u32, u32, u32, &[&str])>) -> SubtitleDocument {
        SubtitleDocument::new(
            cues.into_iter()
                .map(|(id, start, end, lines)| {
                    Cue::new(
                        CueId::new(id),
                        TimeSpan::new(start, end).unwrap(),
                        lines.iter().map(|s| s.to_string()).collect(),
                    )
                })
                .collect(),
        )
    }

    #[test]
    fn same_timeline_same_timeline_fingerprint() {
        let a = doc(vec![
            (1, 0, 1_000, &["hello"]),
            (2, 1_000, 2_000, &["world"]),
        ]);
        let b = doc(vec![
            (1, 0, 1_000, &["farklı"]),
            (2, 1_000, 2_000, &["metin"]),
        ]);
        assert_eq!(TimelineFingerprint::of(&a), TimelineFingerprint::of(&b));
    }

    #[test]
    fn one_ms_shift_changes_timeline_fingerprint() {
        let a = doc(vec![(1, 0, 1_000, &["hello"])]);
        let b = doc(vec![(1, 0, 1_001, &["hello"])]);
        assert_ne!(TimelineFingerprint::of(&a), TimelineFingerprint::of(&b));
    }

    #[test]
    fn text_change_changes_source_not_timeline() {
        let a = doc(vec![(1, 0, 1_000, &["hello"])]);
        let b = doc(vec![(1, 0, 1_000, &["goodbye"])]);
        assert_eq!(TimelineFingerprint::of(&a), TimelineFingerprint::of(&b));
        assert_ne!(SourceFingerprint::of(&a), SourceFingerprint::of(&b));
    }

    #[test]
    fn cue_order_change_changes_both_fingerprints() {
        let a = doc(vec![
            (1, 0, 1_000, &["first"]),
            (2, 1_000, 2_000, &["second"]),
        ]);
        let b = doc(vec![
            (2, 1_000, 2_000, &["second"]),
            (1, 0, 1_000, &["first"]),
        ]);
        assert_ne!(TimelineFingerprint::of(&a), TimelineFingerprint::of(&b));
        assert_ne!(SourceFingerprint::of(&a), SourceFingerprint::of(&b));
    }

    #[test]
    fn cue_id_does_not_affect_either_fingerprint() {
        let a = doc(vec![(1, 0, 1_000, &["hello"])]);
        let b = doc(vec![(99, 0, 1_000, &["hello"])]);
        assert_eq!(TimelineFingerprint::of(&a), TimelineFingerprint::of(&b));
        assert_eq!(SourceFingerprint::of(&a), SourceFingerprint::of(&b));
    }

    #[test]
    fn different_line_split_changes_source_fingerprint() {
        let a = doc(vec![(1, 0, 1_000, &["ab", "c"])]);
        let b = doc(vec![(1, 0, 1_000, &["a", "bc"])]);
        assert_ne!(SourceFingerprint::of(&a), SourceFingerprint::of(&b));
    }

    #[test]
    fn empty_document_does_not_panic() {
        let empty = SubtitleDocument::new(vec![]);
        let _ = TimelineFingerprint::of(&empty);
        let _ = SourceFingerprint::of(&empty);
    }

    #[test]
    fn display_is_lowercase_hex_of_64_chars() {
        let a = doc(vec![(1, 0, 1_000, &["hello"])]);
        let hex = TimelineFingerprint::of(&a).to_string();
        assert_eq!(hex.len(), 64);
        assert!(hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }
}
