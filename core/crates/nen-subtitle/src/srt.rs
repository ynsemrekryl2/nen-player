//! Strict SRT parsing (NEN-013).
//!
//! "Strict" is a deliberate choice, not an unfinished tolerant parser: every
//! deviation from the format is rejected with a distinct typed error rather
//! than repaired or silently dropped. A file that parses is a file we can
//! reason about downstream (fingerprint, translation, sync) without carrying
//! "…unless the source was weird" caveats.
//!
//! Input is `&str`: this module assumes valid UTF-8 and does **not** do
//! encoding detection — that is NEN-015, which runs before this and hands
//! over decoded text. A leading BOM is still rejected loudly here rather than
//! skipped, so nothing about the byte layer can pass through unnoticed.
//!
//! What is *not* an error: cues that overlap in time. Simultaneous speakers
//! are legitimate in real subtitle files, and rejecting them would refuse
//! valid user files (NEN-025). Ordering is still enforced — see
//! [`SrtError::NonMonotonicCue`].

use std::fmt;

use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan, TimeSpanError};

/// The separator between the two timestamps of a cue's time line.
const ARROW: &str = "-->";

/// Byte-order mark; rejected rather than skipped (see module docs).
const BOM: char = '\u{feff}';

/// Everything strict SRT parsing can refuse, and where.
///
/// **No variant carries subtitle text, a path or any other untrusted string.**
/// `block` (1-based cue block ordinal) and `line` (1-based line number in the
/// input) are what `docs/security-policy.md` §1 lists as loggable — "hata
/// varyantı (payload'sız) · blok indeksi". `#[derive(Debug)]` is therefore
/// safe here, and `tests/guard_error_debug.rs` proves it stays that way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SrtError {
    /// The input held no cue blocks at all (empty or only blank lines).
    EmptyInput,
    /// The input starts with a byte-order mark; decoding belongs to NEN-015.
    LeadingBom,
    /// A block starts with its time line — the index line is missing.
    MissingIndexLine { block: u32, line: u32 },
    /// The index line is not a plain sequence of digits.
    NonNumericIndex { block: u32, line: u32 },
    /// The index line is all digits but does not fit in `u32`.
    IndexOverflow { block: u32, line: u32 },
    /// Indices must run 1, 2, 3, … with no gaps, repeats or reordering.
    NonSequentialIndex {
        block: u32,
        line: u32,
        expected: u32,
        found: u32,
    },
    /// The block has an index line and nothing after it.
    MissingTimeLine { block: u32, line: u32 },
    /// The time line has no `-->` and nothing that looks like one.
    MissingArrow { block: u32, line: u32 },
    /// The time line has something arrow-like but not exactly one `-->`.
    MalformedArrow { block: u32, line: u32 },
    /// Extra tokens on the time line (e.g. `X1:… Y1:…` position fields).
    TrailingContentOnTimeLine { block: u32, line: u32 },
    /// A timestamp is not shaped `HH:MM:SS,mmm`.
    MalformedTimestamp { block: u32, line: u32 },
    /// A timestamp field contains something other than ASCII digits.
    NonNumericTimestampField { block: u32, line: u32 },
    /// A timestamp field is well-formed but out of range (e.g. `MM` > 59).
    TimestampFieldOutOfRange { block: u32, line: u32 },
    /// The cue ends before it starts.
    EndBeforeStart { block: u32, line: u32 },
    /// The cue starts and ends at the same millisecond.
    ZeroDuration { block: u32, line: u32 },
    /// A cue starts earlier than the cue before it. Equal start times are
    /// allowed (simultaneous speakers); going backwards is not.
    NonMonotonicCue {
        block: u32,
        line: u32,
        previous_start_ms: u32,
        start_ms: u32,
    },
    /// The block has an index and a time line but no text lines.
    EmptyText { block: u32, line: u32 },
}

impl SrtError {
    /// The variant's name on its own, with no payload attached.
    ///
    /// This is the one projection of an error that `docs/security-policy.md`
    /// §1 allows into a log ("hata varyantı (payload'sız)"), so callers that
    /// need to report *what* went wrong without the where should use this
    /// rather than `Display`.
    pub const fn variant_name(&self) -> &'static str {
        match self {
            Self::EmptyInput => "EmptyInput",
            Self::LeadingBom => "LeadingBom",
            Self::MissingIndexLine { .. } => "MissingIndexLine",
            Self::NonNumericIndex { .. } => "NonNumericIndex",
            Self::IndexOverflow { .. } => "IndexOverflow",
            Self::NonSequentialIndex { .. } => "NonSequentialIndex",
            Self::MissingTimeLine { .. } => "MissingTimeLine",
            Self::MissingArrow { .. } => "MissingArrow",
            Self::MalformedArrow { .. } => "MalformedArrow",
            Self::TrailingContentOnTimeLine { .. } => "TrailingContentOnTimeLine",
            Self::MalformedTimestamp { .. } => "MalformedTimestamp",
            Self::NonNumericTimestampField { .. } => "NonNumericTimestampField",
            Self::TimestampFieldOutOfRange { .. } => "TimestampFieldOutOfRange",
            Self::EndBeforeStart { .. } => "EndBeforeStart",
            Self::ZeroDuration { .. } => "ZeroDuration",
            Self::NonMonotonicCue { .. } => "NonMonotonicCue",
            Self::EmptyText { .. } => "EmptyText",
        }
    }
}

impl fmt::Display for SrtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => f.write_str("input contains no cue blocks"),
            Self::LeadingBom => f.write_str("input starts with a byte-order mark"),
            Self::MissingIndexLine { block, line } => {
                write!(f, "block {block} (line {line}): missing index line")
            }
            Self::NonNumericIndex { block, line } => {
                write!(f, "block {block} (line {line}): index is not numeric")
            }
            Self::IndexOverflow { block, line } => {
                write!(f, "block {block} (line {line}): index does not fit in u32")
            }
            Self::NonSequentialIndex {
                block,
                line,
                expected,
                found,
            } => write!(
                f,
                "block {block} (line {line}): expected index {expected}, found {found}"
            ),
            Self::MissingTimeLine { block, line } => {
                write!(f, "block {block} (line {line}): missing time line")
            }
            Self::MissingArrow { block, line } => {
                write!(f, "block {block} (line {line}): time line has no '-->'")
            }
            Self::MalformedArrow { block, line } => {
                write!(f, "block {block} (line {line}): malformed '-->' separator")
            }
            Self::TrailingContentOnTimeLine { block, line } => {
                write!(f, "block {block} (line {line}): extra content on time line")
            }
            Self::MalformedTimestamp { block, line } => write!(
                f,
                "block {block} (line {line}): timestamp is not HH:MM:SS,mmm"
            ),
            Self::NonNumericTimestampField { block, line } => write!(
                f,
                "block {block} (line {line}): timestamp field is not numeric"
            ),
            Self::TimestampFieldOutOfRange { block, line } => {
                write!(f, "block {block} (line {line}): timestamp field out of range")
            }
            Self::EndBeforeStart { block, line } => {
                write!(f, "block {block} (line {line}): end time before start time")
            }
            Self::ZeroDuration { block, line } => {
                write!(f, "block {block} (line {line}): zero-duration cue")
            }
            Self::NonMonotonicCue {
                block,
                line,
                previous_start_ms,
                start_ms,
            } => write!(
                f,
                "block {block} (line {line}): starts at {start_ms} ms, before the previous cue's {previous_start_ms} ms"
            ),
            Self::EmptyText { block, line } => {
                write!(f, "block {block} (line {line}): cue has no text lines")
            }
        }
    }
}

impl std::error::Error for SrtError {}

/// One line of a block: its 1-based line number in the input, and its content
/// with the trailing `\r` of a CRLF ending already removed.
type BlockLine<'a> = (u32, &'a str);

/// Parses a complete SRT document.
///
/// Returns the first error encountered; parsing is all-or-nothing, there is no
/// partial document and no recovery.
pub fn parse(input: &str) -> Result<SubtitleDocument, SrtError> {
    if input.starts_with(BOM) {
        return Err(SrtError::LeadingBom);
    }

    let blocks = split_blocks(input);
    if blocks.is_empty() {
        return Err(SrtError::EmptyInput);
    }

    let mut cues = Vec::with_capacity(blocks.len());
    let mut expected_index: u32 = 1;
    let mut previous_start_ms: Option<u32> = None;

    for (offset, block) in blocks.iter().enumerate() {
        let block_no = to_u32(offset).saturating_add(1);

        // `split_blocks` never emits an empty block, so `first()` is always
        // `Some`; the `else` arm exists only to avoid indexing.
        let Some(&(index_line_no, index_line)) = block.first() else {
            continue;
        };

        let index = parse_index(index_line, block_no, index_line_no)?;
        if index != expected_index {
            return Err(SrtError::NonSequentialIndex {
                block: block_no,
                line: index_line_no,
                expected: expected_index,
                found: index,
            });
        }

        let Some(&(time_line_no, time_line)) = block.get(1) else {
            return Err(SrtError::MissingTimeLine {
                block: block_no,
                line: index_line_no,
            });
        };
        let (start_ms, end_ms) = parse_time_line(time_line, block_no, time_line_no)?;

        let span = TimeSpan::new(start_ms, end_ms).map_err(|err| match err {
            TimeSpanError::EndBeforeStart => SrtError::EndBeforeStart {
                block: block_no,
                line: time_line_no,
            },
            TimeSpanError::ZeroDuration => SrtError::ZeroDuration {
                block: block_no,
                line: time_line_no,
            },
        })?;

        if let Some(previous) = previous_start_ms {
            if start_ms < previous {
                return Err(SrtError::NonMonotonicCue {
                    block: block_no,
                    line: time_line_no,
                    previous_start_ms: previous,
                    start_ms,
                });
            }
        }

        let text_lines = block.get(2..).unwrap_or(&[]);
        if text_lines.is_empty() {
            return Err(SrtError::EmptyText {
                block: block_no,
                line: time_line_no,
            });
        }

        let lines = text_lines
            .iter()
            .map(|&(_, text)| text.to_string())
            .collect();

        cues.push(Cue::new(CueId::new(index), span, lines));
        previous_start_ms = Some(start_ms);
        expected_index = expected_index.saturating_add(1);
    }

    Ok(SubtitleDocument::new(cues))
}

/// Groups non-blank lines into blocks. Blank lines (including whitespace-only
/// ones) separate blocks, so a cue's text can never contain one; runs of blank
/// lines and trailing blank lines are tolerated, as they carry no meaning.
///
/// Both LF and CRLF inputs are accepted: a single trailing `\r` is stripped
/// from each line, which is what distinguishes a line ending from content.
fn split_blocks(input: &str) -> Vec<Vec<BlockLine<'_>>> {
    let mut blocks: Vec<Vec<BlockLine<'_>>> = Vec::new();
    let mut current: Vec<BlockLine<'_>> = Vec::new();

    for (offset, raw) in input.split('\n').enumerate() {
        let line_no = to_u32(offset).saturating_add(1);
        let line = raw.strip_suffix('\r').unwrap_or(raw);

        if line.trim().is_empty() {
            if !current.is_empty() {
                blocks.push(std::mem::take(&mut current));
            }
        } else {
            current.push((line_no, line));
        }
    }

    if !current.is_empty() {
        blocks.push(current);
    }

    blocks
}

fn parse_index(line: &str, block: u32, line_no: u32) -> Result<u32, SrtError> {
    let text = line.trim();

    if text.contains(ARROW) {
        return Err(SrtError::MissingIndexLine {
            block,
            line: line_no,
        });
    }
    if text.is_empty() || !is_ascii_digits(text) {
        return Err(SrtError::NonNumericIndex {
            block,
            line: line_no,
        });
    }

    text.parse().map_err(|_| SrtError::IndexOverflow {
        block,
        line: line_no,
    })
}

fn parse_time_line(line: &str, block: u32, line_no: u32) -> Result<(u32, u32), SrtError> {
    let Some(arrow_at) = line.find(ARROW) else {
        // Distinguish "there is no separator at all" from "there is something
        // separator-shaped but wrong", so the fixture corpus can pin both.
        return Err(if line.contains('>') || line.contains('-') {
            SrtError::MalformedArrow {
                block,
                line: line_no,
            }
        } else {
            SrtError::MissingArrow {
                block,
                line: line_no,
            }
        });
    };

    let before = line.get(..arrow_at).unwrap_or("");
    let after = line
        .get(arrow_at.saturating_add(ARROW.len())..)
        .unwrap_or("");

    // `--->`, `-->>` and a second `-->` are all malformed rather than
    // "a timestamp with junk around it".
    if before.ends_with('-') || after.starts_with('>') || after.contains(ARROW) {
        return Err(SrtError::MalformedArrow {
            block,
            line: line_no,
        });
    }

    let start_text = single_token(before, block, line_no)?;
    let end_text = single_token(after, block, line_no)?;

    Ok((
        parse_timestamp(start_text, block, line_no)?,
        parse_timestamp(end_text, block, line_no)?,
    ))
}

/// Returns the one and only whitespace-delimited token of `part`.
fn single_token(part: &str, block: u32, line_no: u32) -> Result<&str, SrtError> {
    let mut tokens = part.split_whitespace();
    let first = tokens.next().ok_or(SrtError::MalformedTimestamp {
        block,
        line: line_no,
    })?;
    if tokens.next().is_some() {
        return Err(SrtError::TrailingContentOnTimeLine {
            block,
            line: line_no,
        });
    }
    Ok(first)
}

/// Parses `HH:MM:SS,mmm` with exact field widths.
fn parse_timestamp(text: &str, block: u32, line_no: u32) -> Result<u32, SrtError> {
    let malformed = SrtError::MalformedTimestamp {
        block,
        line: line_no,
    };

    let mut colon_parts = text.split(':');
    let hours = colon_parts.next().ok_or(malformed)?;
    let minutes = colon_parts.next().ok_or(malformed)?;
    let rest = colon_parts.next().ok_or(malformed)?;
    if colon_parts.next().is_some() {
        return Err(malformed);
    }

    let mut comma_parts = rest.split(',');
    let seconds = comma_parts.next().ok_or(malformed)?;
    let millis = comma_parts.next().ok_or(malformed)?;
    if comma_parts.next().is_some() {
        return Err(malformed);
    }

    let hours = parse_field(hours, 2, block, line_no)?;
    let minutes = parse_field(minutes, 2, block, line_no)?;
    let seconds = parse_field(seconds, 2, block, line_no)?;
    let millis = parse_field(millis, 3, block, line_no)?;

    if minutes > 59 || seconds > 59 {
        return Err(SrtError::TimestampFieldOutOfRange {
            block,
            line: line_no,
        });
    }

    // Bounded by the field widths above, so this cannot actually overflow —
    // written with `checked_*` anyway so that no arithmetic here can panic.
    let out_of_range = SrtError::TimestampFieldOutOfRange {
        block,
        line: line_no,
    };
    hours
        .checked_mul(3_600_000)
        .and_then(|h| h.checked_add(minutes.checked_mul(60_000)?))
        .and_then(|hm| hm.checked_add(seconds.checked_mul(1_000)?))
        .and_then(|hms| hms.checked_add(millis))
        .ok_or(out_of_range)
}

fn parse_field(text: &str, width: usize, block: u32, line_no: u32) -> Result<u32, SrtError> {
    if text.is_empty() {
        return Err(SrtError::MalformedTimestamp {
            block,
            line: line_no,
        });
    }
    if !is_ascii_digits(text) {
        return Err(SrtError::NonNumericTimestampField {
            block,
            line: line_no,
        });
    }
    if text.len() != width {
        return Err(SrtError::MalformedTimestamp {
            block,
            line: line_no,
        });
    }
    text.parse()
        .map_err(|_| SrtError::TimestampFieldOutOfRange {
            block,
            line: line_no,
        })
}

fn is_ascii_digits(text: &str) -> bool {
    text.bytes().all(|byte| byte.is_ascii_digit())
}

/// Saturating `usize` → `u32`; line and block counters never legitimately
/// reach `u32::MAX`, and saturating beats an unwrap on a value we only report.
fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_CUES: &str = "1\n00:00:01,000 --> 00:00:02,000\nFirst line\n\n\
                            2\n00:00:03,500 --> 00:00:05,000\nSecond line\nkeeps its break\n";

    #[test]
    fn parses_indices_timings_and_line_structure() {
        let doc = parse(TWO_CUES).unwrap();
        assert_eq!(doc.len(), 2);

        let first = &doc.cues()[0];
        assert_eq!(first.id().get(), 1);
        assert_eq!(first.span().start_ms(), 1_000);
        assert_eq!(first.span().end_ms(), 2_000);
        assert_eq!(first.lines(), ["First line"]);

        let second = &doc.cues()[1];
        assert_eq!(second.span().start_ms(), 3_500);
        assert_eq!(second.lines(), ["Second line", "keeps its break"]);
    }

    #[test]
    fn crlf_and_lf_produce_identical_documents() {
        let crlf = TWO_CUES.replace('\n', "\r\n");
        assert_eq!(parse(TWO_CUES).unwrap(), parse(&crlf).unwrap());
    }

    #[test]
    fn accepts_overlapping_cues() {
        let overlapping = "1\n00:00:01,000 --> 00:00:05,000\nSpeaker A\n\n\
                           2\n00:00:02,000 --> 00:00:04,000\nSpeaker B\n";
        let doc = parse(overlapping).unwrap();
        assert!(doc.cues()[0].span().overlaps(doc.cues()[1].span()));
    }

    #[test]
    fn rejects_cue_starting_before_the_previous_one() {
        let backwards = "1\n00:00:05,000 --> 00:00:06,000\nLater\n\n\
                         2\n00:00:01,000 --> 00:00:02,000\nEarlier\n";
        assert_eq!(
            parse(backwards),
            Err(SrtError::NonMonotonicCue {
                block: 2,
                line: 6,
                previous_start_ms: 5_000,
                start_ms: 1_000,
            })
        );
    }

    #[test]
    fn reports_the_line_number_of_the_offending_line() {
        let bad = "1\n00:00:01,000 --> 00:00:02,000\nOk\n\n\
                   2\n00:00:03,000 -> 00:00:04,000\nBad\n";
        assert_eq!(
            parse(bad),
            Err(SrtError::MalformedArrow { block: 2, line: 6 })
        );
    }

    #[test]
    fn parses_the_maximum_srt_timestamp() {
        let max = "1\n99:59:59,998 --> 99:59:59,999\nEdge\n";
        let doc = parse(max).unwrap();
        assert_eq!(
            doc.cues()[0].span().end_ms(),
            nen_domain::subtitle::MAX_TIMESTAMP_MS
        );
    }

    #[test]
    fn blank_input_is_empty_input_not_an_empty_document() {
        assert_eq!(parse(""), Err(SrtError::EmptyInput));
        assert_eq!(parse("\n\n   \n"), Err(SrtError::EmptyInput));
    }
}
