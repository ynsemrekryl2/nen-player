//! What the engine reports about an embedded audio or subtitle track.
//!
//! Enumeration and selection are part of the **mandatory base** (ADR-0011
//! Karar 3): an engine that cannot list its own tracks cannot back a subtitle
//! menu at all (§8).
//!
//! Nothing here extracts text. Product-spec §7 requires the catalog to exist
//! *before* anything is decoded, so a descriptor is metadata only; pulling a
//! track's text is a separate, lazy operation behind
//! [`Capability::EmbeddedTextExtraction`](super::Capability::EmbeddedTextExtraction)
//! and belongs to NEN-044.
//!
//! **Security (K23 #8).** A track title is very often taken straight from the
//! container and is regularly a private filename or release name. It is
//! *displayable* and **not** *loggable* — the same distinction
//! `nen_domain::source::SubtitleSource` draws for its label. [`TrackDescriptor`]
//! therefore implements `Debug` by hand and never prints the title. Do not
//! replace that impl with `#[derive(Debug)]`; `tests/guard_track_debug.rs`
//! proves with a deliberately derived twin that doing so leaks.

use nen_domain::source::LanguageTag;
use std::fmt;

/// Whether a track carries audio or subtitles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TrackKind {
    Audio,
    Subtitle,
}

impl TrackKind {
    /// Stable lowercase name. Safe to log.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Audio => "audio",
            Self::Subtitle => "subtitle",
        }
    }
}

impl fmt::Display for TrackKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The engine's index for a track inside the loaded medium.
///
/// A plain index rather than an opaque handle, because
/// `nen_domain::source::SubtitleSourceId::embedded` already identifies a
/// catalogued embedded source by exactly this number — the two have to agree
/// or the menu cannot map an entry back to a track.
///
/// The index is only meaningful for the medium it came from; it is not stable
/// across loads and carries no private data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrackId(pub u32);

impl TrackId {
    pub const fn index(self) -> u32 {
        self.0
    }
}

impl fmt::Display for TrackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Whether a subtitle codec carries text at all.
///
/// The single place this question is answered. A bitmap format (PGS, VobSub,
/// DVB, XSUB) draws pictures: there is nothing to extract, nothing to
/// translate, and nothing to sync against a transcript -- which is why §7 lets
/// such a track be *shown* while being marked untranslatable.
///
/// **The unknown direction is deliberate.** A codec this list does not know is
/// treated as **not** text. The alternative fails in the worse direction:
/// promising text for a format we cannot read produces an empty document at the
/// moment the user asks for a translation, whereas a missing entry produces a
/// track that is merely not offered for translation. Adding a format is one
/// line here and nowhere else.
///
/// Names are the container's own, as libavformat spells them -- the string an
/// adapter reads out of the demuxer, not a normalized name of ours.
pub fn subtitle_carries_text(codec: &str) -> bool {
    matches!(
        codec,
        "subrip"
            | "srt"
            | "ass"
            | "ssa"
            | "webvtt"
            | "text"
            | "mov_text"
            | "microdvd"
            | "eia_608"
            | "subviewer"
            | "subviewer1"
            | "sami"
            | "stl"
    )
}

/// One embedded track, as metadata.
#[derive(Clone, PartialEq, Eq)]
pub struct TrackDescriptor {
    id: TrackId,
    kind: TrackKind,
    language: Option<LanguageTag>,
    title: Option<String>,
    codec: String,
    is_default: bool,
    is_text: bool,
}

impl TrackDescriptor {
    pub fn new(id: TrackId, kind: TrackKind, codec: impl Into<String>) -> Self {
        let codec = codec.into();
        // Derived, never supplied. An adapter reports what the container says
        // -- the codec -- and the classification happens once, here, for every
        // platform. Letting each adapter decide would put the same list in
        // Swift and again in Kotlin, and the first one to miss a format would
        // promise text it cannot produce.
        let is_text = matches!(kind, TrackKind::Subtitle) && subtitle_carries_text(&codec);
        Self {
            id,
            kind,
            language: None,
            title: None,
            codec,
            is_default: false,
            is_text,
        }
    }

    pub fn with_language(mut self, language: Option<LanguageTag>) -> Self {
        self.language = language;
        self
    }

    /// Sets the display title. Never log the result — see the module note.
    pub fn with_title(mut self, title: Option<String>) -> Self {
        self.title = title;
        self
    }

    pub fn with_default(mut self, is_default: bool) -> Self {
        self.is_default = is_default;
        self
    }

    pub fn id(&self) -> TrackId {
        self.id
    }

    pub fn kind(&self) -> TrackKind {
        self.kind
    }

    /// The declared language, or `None` when the container does not say.
    ///
    /// Nothing here guesses: an absent language stays absent and becomes
    /// `Dil Belirsiz` downstream (ADR-0010 Karar 6).
    pub fn language(&self) -> Option<&LanguageTag> {
        self.language.as_ref()
    }

    /// Display text for the menu (§8). **Never log this.**
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// The container's codec name. Safe to log — it describes a format, not
    /// the user's file.
    pub fn codec(&self) -> &str {
        &self.codec
    }

    /// Whether the container marks this track as its default.
    pub fn is_default(&self) -> bool {
        self.is_default
    }

    /// Whether the track carries text rather than bitmaps.
    ///
    /// `false` for bitmap subtitle formats (PGS, VobSub, DVB) and for every
    /// audio track. This is what becomes the catalog's `translatable = false`
    /// mark (§7, NEN-023).
    pub fn is_text(&self) -> bool {
        self.is_text
    }
}

impl fmt::Debug for TrackDescriptor {
    /// Prints everything except the title.
    ///
    /// The title routinely carries a release name or a private filename
    /// (K23 #8). The remaining fields describe the format, not the user.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrackDescriptor")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("language", &self.language)
            .field("codec", &self.codec)
            .field("is_default", &self.is_default)
            .field("is_text", &self.is_text)
            .field("has_title", &self.title.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(input: &str) -> LanguageTag {
        LanguageTag::parse(input).expect("valid tag")
    }

    #[test]
    fn a_new_descriptor_declares_no_language_and_no_title() {
        let track = TrackDescriptor::new(TrackId(0), TrackKind::Subtitle, "subrip");
        assert_eq!(track.language(), None);
        assert_eq!(track.title(), None);
        assert!(!track.is_default());
        assert!(track.is_text());
    }

    #[test]
    fn builders_keep_identity() {
        let track = TrackDescriptor::new(TrackId(3), TrackKind::Audio, "aac")
            .with_language(Some(tag("tr")))
            .with_title(Some("Türkçe".into()))
            .with_default(true);
        assert_eq!(track.id(), TrackId(3));
        assert_eq!(track.kind(), TrackKind::Audio);
        assert_eq!(track.language().map(LanguageTag::as_str), Some("tr"));
        assert_eq!(track.title(), Some("Türkçe"));
        assert!(track.is_default());
    }

    #[test]
    fn bitmap_tracks_are_marked_as_carrying_no_text() {
        for codec in ["hdmv_pgs_subtitle", "dvd_subtitle", "dvb_subtitle", "xsub"] {
            let track = TrackDescriptor::new(TrackId(1), TrackKind::Subtitle, codec);
            assert!(!track.is_text(), "{codec} was taken for text");
        }
    }

    #[test]
    fn text_subtitle_codecs_are_recognized() {
        for codec in ["subrip", "ass", "ssa", "webvtt", "mov_text"] {
            let track = TrackDescriptor::new(TrackId(1), TrackKind::Subtitle, codec);
            assert!(track.is_text(), "{codec} was not taken for text");
        }
    }

    #[test]
    fn an_unknown_codec_is_not_taken_for_text() {
        // The conservative direction: a format nobody here knows must not
        // promise a translation it cannot deliver.
        let track = TrackDescriptor::new(TrackId(1), TrackKind::Subtitle, "some_future_format");
        assert!(!track.is_text());
        assert!(!subtitle_carries_text(""));
    }

    #[test]
    fn an_audio_track_never_carries_subtitle_text() {
        // Even when its codec name happens to collide with a subtitle one, the
        // question is meaningless for audio and the answer must be `false`.
        for codec in ["aac", "opus", "text"] {
            let track = TrackDescriptor::new(TrackId(0), TrackKind::Audio, codec);
            assert!(!track.is_text(), "audio/{codec} claimed to carry text");
        }
    }

    #[test]
    fn debug_does_not_print_the_title() {
        let track = TrackDescriptor::new(TrackId(0), TrackKind::Subtitle, "subrip")
            .with_title(Some("The.Movie.2019.1080p.PRIVATE-GROUP".into()));
        let printed = format!("{track:?}");
        assert!(
            !printed.contains("PRIVATE-GROUP"),
            "title leaked: {printed}"
        );
        assert!(printed.contains("has_title: true"));
        assert!(printed.contains("subrip"));
    }
}
