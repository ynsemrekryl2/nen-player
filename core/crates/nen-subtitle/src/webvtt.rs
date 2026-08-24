//! WebVTT writing (NEN-014).
//!
//! `SubtitleDocument` → UTF-8 WebVTT text, with no dependency on how the
//! document was produced. This module only writes; reading/parsing WebVTT is
//! out of scope (see the task's YAPILMAYACAK) and may come later as its own
//! task.
//!
//! WebVTT reserves `&`, `<` and `>` in cue payload text (they start character
//! references and tag-like markup), so those three are escaped on the way
//! out — everything else in a line passes through unchanged, UTF-8 and all.
//! No BOM is ever written.

use nen_domain::subtitle::SubtitleDocument;

/// Renders a document as a complete WebVTT file: the `WEBVTT` header, then one
/// block per cue in document order — its id as an identifier line, its
/// timing, and its text lines — separated by blank lines.
pub fn write(document: &SubtitleDocument) -> String {
    let mut out = String::from("WEBVTT\n\n");

    for cue in document.cues() {
        out.push_str(&cue.id().to_string());
        out.push('\n');
        out.push_str(&format_timestamp(cue.span().start_ms()));
        out.push_str(" --> ");
        out.push_str(&format_timestamp(cue.span().end_ms()));
        out.push('\n');
        for line in cue.lines() {
            out.push_str(&escape_cue_text(line));
            out.push('\n');
        }
        out.push('\n');
    }

    out
}

/// Formats milliseconds as WebVTT's `HH:MM:SS.mmm`, zero-padded 2/2/2/3 —
/// the same field widths as SRT's `HH:MM:SS,mmm`, just with `.` instead of
/// `,`. `TimeSpan` guarantees `ms <= MAX_TIMESTAMP_MS`, so all four fields
/// stay within their padded width.
fn format_timestamp(ms: u32) -> String {
    let hours = ms / 3_600_000;
    let minutes = (ms / 60_000) % 60;
    let seconds = (ms / 1_000) % 60;
    let millis = ms % 1_000;
    format!("{hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
}

/// Escapes the three characters WebVTT cue payload text reserves. `&` is
/// replaced first so the `&` it introduces for `<`/`>` is never itself
/// re-escaped.
fn escape_cue_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_zero_and_the_max_srt_timestamp() {
        assert_eq!(format_timestamp(0), "00:00:00.000");
        assert_eq!(
            format_timestamp(nen_domain::subtitle::MAX_TIMESTAMP_MS),
            "99:59:59.999"
        );
    }

    #[test]
    fn escapes_ampersand_before_angle_brackets_without_double_escaping() {
        assert_eq!(escape_cue_text("Tom & Jerry"), "Tom &amp; Jerry");
        assert_eq!(escape_cue_text("<i>hi</i>"), "&lt;i&gt;hi&lt;/i&gt;");
        assert_eq!(escape_cue_text("a < b & c > d"), "a &lt; b &amp; c &gt; d");
    }

    #[test]
    fn leaves_ordinary_unicode_text_untouched() {
        assert_eq!(escape_cue_text("café — 日本語"), "café — 日本語");
    }
}
