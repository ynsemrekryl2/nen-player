//! Subtitle source value types: what a catalog entry is, and how the user's
//! language preferences are expressed.
//!
//! These are pure values — no I/O, no hashing, no policy. `nen-catalog` builds
//! the catalog, the menu projection and the auto-selection policy from them
//! (NEN-019); `nen-translate` later treats the selected source as the
//! translation input (product-spec §9).
//!
//! Shape and rules come from ADR-0010:
//!
//! - a source is identified by **metadata**, never by its content — the catalog
//!   is built before anything is downloaded or extracted (§7 "lazy"), so a
//!   content fingerprint simply is not available (Karar 2);
//! - the language is supplied from outside as an `Option<LanguageTag>`; nothing
//!   here detects it (NEN-020's job), and `None` means "Dil Belirsiz" (Karar 6);
//! - ordering of language groups is by [`LanguageTag`], which is why `Ord` here
//!   is the normalized tag's byte order: `en` < `fr` < `tr` (Karar 5, 7).
//!
//! **Security:** a [`SubtitleSource`] carries a display label that is very often
//! a private filename, and a [`SubtitleSourceId`] carries either a digest of a
//! private path or an OpenSubtitles public id. `docs/security-policy.md` K23 #3
//! (private full path) and #8 (private hash / filename metadata) forbid all of
//! those from ever reaching a log — being *displayable* is not being
//! *loggable*. Both types therefore implement `Debug` by hand and print only
//! the kind, the language and a length. Do not replace those impls with
//! `#[derive(Debug)]`; `tests/guard_source_debug.rs` in `nen-catalog` proves,
//! with a deliberately derived twin, that doing so leaks.

use std::fmt;

/// Where a subtitle source came from (product-spec §7).
///
/// This doubles as the origin badge shown next to an entry (§8) and as the
/// auto-selection tier (ADR-0010 Karar 9). It is free of private data, so it
/// may be logged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SubtitleSourceKind {
    /// A track inside the media file itself.
    Embedded,
    /// A file the user loaded, or a sidecar found next to the media.
    User,
    /// A candidate from OpenSubtitles — catalogued, not downloaded (§7).
    OpenSubtitles,
    /// A validated translation artifact produced by this app (§9).
    Ai,
}

impl SubtitleSourceKind {
    /// Stable lowercase name. Safe to log and stable enough to build ids from.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Embedded => "embedded",
            Self::User => "user",
            Self::OpenSubtitles => "opensubtitles",
            Self::Ai => "ai",
        }
    }
}

impl fmt::Display for SubtitleSourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why a string could not become a [`LanguageTag`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageTagError {
    /// The input was empty or only separators.
    Empty,
    /// The primary subtag is not 2–3 ASCII letters.
    InvalidPrimary,
    /// The region subtag is neither 2 ASCII letters nor 3 ASCII digits.
    InvalidRegion,
    /// More than a primary subtag and one region subtag were given.
    TooManySubtags,
}

impl fmt::Display for LanguageTagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("empty language tag"),
            Self::InvalidPrimary => f.write_str("primary subtag must be 2-3 ASCII letters"),
            Self::InvalidRegion => {
                f.write_str("region subtag must be 2 ASCII letters or 3 ASCII digits")
            }
            Self::TooManySubtags => f.write_str("only a primary and one region subtag are allowed"),
        }
    }
}

/// A normalized BCP-47 language tag: a primary subtag and an optional region,
/// lowercased — `en`, `tr`, `pt-br`.
///
/// The scope is deliberately narrow. This is what a subtitle group is keyed by
/// and sorted by (ADR-0010 Karar 5); script, variant and extension subtags do
/// not change which group a subtitle belongs to, and accepting them would make
/// two spellings of the same group sort apart. An invalid tag is an `Err`, never
/// a silently repaired value — a subtitle whose language could not be
/// established belongs in `Dil Belirsiz`, and that is a `None`, not a guess.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LanguageTag(String);

impl LanguageTag {
    /// Parses and normalizes a tag. Case and separator case are irrelevant;
    /// `EN-us`, `en-US` and `en-us` are the same tag.
    pub fn parse(input: &str) -> Result<Self, LanguageTagError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(LanguageTagError::Empty);
        }

        // Subtag *count* is checked before subtag *shape* so that `zh-hant-cn`
        // reports the thing that is actually wrong with it — three subtags —
        // rather than blaming `hant` for not looking like a region.
        let parts: Vec<&str> = trimmed.split('-').collect();
        if parts.len() > 2 {
            return Err(LanguageTagError::TooManySubtags);
        }

        let primary = parts[0];
        if primary.len() < 2
            || primary.len() > 3
            || !primary.bytes().all(|b| b.is_ascii_alphabetic())
        {
            return Err(LanguageTagError::InvalidPrimary);
        }

        let mut tag = primary.to_ascii_lowercase();

        if let Some(region) = parts.get(1) {
            let alpha2 = region.len() == 2 && region.bytes().all(|b| b.is_ascii_alphabetic());
            let digit3 = region.len() == 3 && region.bytes().all(|b| b.is_ascii_digit());
            if !alpha2 && !digit3 {
                return Err(LanguageTagError::InvalidRegion);
            }
            tag.push('-');
            tag.push_str(&region.to_ascii_lowercase());
        }

        Ok(Self(tag))
    }

    /// The normalized tag, e.g. `pt-br`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// What makes two catalog entries "the same source" (ADR-0010 Karar 2).
///
/// Identity is **metadata**, never content: the catalog exists before anything
/// is downloaded or extracted (§7), so a `SourceFingerprint` is not available
/// and computing one would be exactly the eager work §7 forbids.
///
/// The key is opaque by construction. For a user file it is a digest of the
/// path supplied by the caller — the raw path never enters this type (K23 #3);
/// for OpenSubtitles it is the *public* source id, never the private file id.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubtitleSourceId {
    kind: SubtitleSourceKind,
    key: String,
}

impl SubtitleSourceId {
    /// A track of the media being played, identified by its index.
    pub fn embedded(track_index: u32) -> Self {
        Self {
            kind: SubtitleSourceKind::Embedded,
            key: track_index.to_string(),
        }
    }

    /// A user file, identified by a digest of its path.
    ///
    /// The digest is computed by the caller (`nen_catalog::user_source_id`)
    /// because this crate has no dependencies. Passing a digest rather than a
    /// path is what keeps K23 #3 satisfied by construction: there is no code
    /// path by which a raw path can be stored here.
    pub fn user(path_digest: [u8; 32]) -> Self {
        let mut key = String::with_capacity(64);
        for byte in path_digest {
            key.push(char::from_digit((byte >> 4) as u32, 16).unwrap_or('0'));
            key.push(char::from_digit((byte & 0x0f) as u32, 16).unwrap_or('0'));
        }
        Self {
            kind: SubtitleSourceKind::User,
            key,
        }
    }

    /// An OpenSubtitles candidate, identified by its **opaque public** id (§7).
    pub fn opensubtitles(public_id: &str) -> Self {
        Self {
            kind: SubtitleSourceKind::OpenSubtitles,
            key: public_id.to_owned(),
        }
    }

    /// A translation artifact: the source it was translated from, plus the
    /// target language. Re-translating the same source into the same language
    /// is the same catalog entry, not a second one.
    pub fn ai(origin: &SubtitleSourceId, target: &LanguageTag) -> Self {
        Self {
            kind: SubtitleSourceKind::Ai,
            key: format!("{}:{}/{}", origin.kind.as_str(), origin.key, target),
        }
    }

    /// Which kind of source this identifies.
    pub fn kind(&self) -> SubtitleSourceKind {
        self.kind
    }
}

impl fmt::Debug for SubtitleSourceId {
    /// Prints the kind only.
    ///
    /// The key is a path digest, an OpenSubtitles public id or something
    /// derived from one of those — K23 #8 keeps all of them out of logs, and a
    /// digest is not safe just because it is unreadable: it is stable and
    /// therefore correlatable across sessions.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubtitleSourceId")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

/// One entry of the subtitle catalog (product-spec §7, glossary).
///
/// The label is what §8 shows next to the entry — very often a private
/// filename. It is displayable and **not** loggable; see the module note.
#[derive(Clone, PartialEq, Eq)]
pub struct SubtitleSource {
    id: SubtitleSourceId,
    language: Option<LanguageTag>,
    label: String,
}

impl SubtitleSource {
    pub fn new(
        id: SubtitleSourceId,
        language: Option<LanguageTag>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id,
            language,
            label: label.into(),
        }
    }

    pub fn id(&self) -> &SubtitleSourceId {
        &self.id
    }

    pub fn kind(&self) -> SubtitleSourceKind {
        self.id.kind()
    }

    /// The language, or `None` for `Dil Belirsiz`. Nothing here detects it —
    /// NEN-020 supplies it and the catalog groups the answer as given.
    pub fn language(&self) -> Option<&LanguageTag> {
        self.language.as_ref()
    }

    /// Display text for the menu (§8). Never log this.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns a copy carrying a different language, keeping identity and label.
    /// Used when detection (NEN-020) fills in a language that was unknown when
    /// the source was first catalogued.
    pub fn with_language(&self, language: Option<LanguageTag>) -> Self {
        Self {
            id: self.id.clone(),
            language,
            label: self.label.clone(),
        }
    }
}

impl fmt::Debug for SubtitleSource {
    /// Prints kind, language and the label's length — never the label.
    ///
    /// The language is safe: it is a two-letter tag shared by millions of
    /// files. The label is not: for a user source it is a filename, which K23
    /// #8 forbids.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubtitleSource")
            .field("kind", &self.id.kind())
            .field("language", &self.language)
            .field("label_len", &self.label.chars().count())
            .finish()
    }
}

/// The user's first and second preferred subtitle languages (ADR-0010 Karar 4).
///
/// Preferences only ever change the **order** of the menu and which language
/// auto-selection looks at. They never hide, filter or alter a group: §8 keeps
/// every source in one menu, and a hidden group is a subtitle the user cannot
/// find.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SubtitlePreferences {
    primary: Option<LanguageTag>,
    secondary: Option<LanguageTag>,
}

impl SubtitlePreferences {
    /// No preference set — the state the app starts in.
    pub fn none() -> Self {
        Self::default()
    }

    /// Builds a preference pair. A secondary equal to the primary is dropped,
    /// so `ordered()` never yields the same language twice.
    pub fn new(primary: Option<LanguageTag>, secondary: Option<LanguageTag>) -> Self {
        let secondary = match (&primary, secondary) {
            (Some(p), Some(s)) if *p == s => None,
            (_, s) => s,
        };
        Self { primary, secondary }
    }

    pub fn primary(&self) -> Option<&LanguageTag> {
        self.primary.as_ref()
    }

    pub fn secondary(&self) -> Option<&LanguageTag> {
        self.secondary.as_ref()
    }

    /// The preferred languages in order, skipping unset ones.
    ///
    /// This is the single sequence both the menu order (Karar 4) and
    /// auto-selection (Karar 9) walk, so the two can never disagree about what
    /// "preferred" means.
    pub fn ordered(&self) -> impl Iterator<Item = &LanguageTag> {
        self.primary.iter().chain(self.secondary.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(s: &str) -> LanguageTag {
        LanguageTag::parse(s).expect("valid tag")
    }

    #[test]
    fn language_tag_normalizes_case_and_region() {
        assert_eq!(tag("EN").as_str(), "en");
        assert_eq!(tag("pt-BR").as_str(), "pt-br");
        assert_eq!(tag("  tr  ").as_str(), "tr");
        assert_eq!(tag("es-419").as_str(), "es-419");
    }

    #[test]
    fn language_tag_rejects_malformed_input() {
        assert_eq!(LanguageTag::parse(""), Err(LanguageTagError::Empty));
        assert_eq!(
            LanguageTag::parse("e"),
            Err(LanguageTagError::InvalidPrimary)
        );
        assert_eq!(
            LanguageTag::parse("engl"),
            Err(LanguageTagError::InvalidPrimary)
        );
        assert_eq!(
            LanguageTag::parse("e1"),
            Err(LanguageTagError::InvalidPrimary)
        );
        assert_eq!(
            LanguageTag::parse("en-USA"),
            Err(LanguageTagError::InvalidRegion)
        );
        assert_eq!(
            LanguageTag::parse("zh-hant-cn"),
            Err(LanguageTagError::TooManySubtags)
        );
    }

    #[test]
    fn language_tags_sort_in_tag_order() {
        let mut tags = [tag("tr"), tag("en"), tag("fr")];
        tags.sort();
        let order: Vec<&str> = tags.iter().map(LanguageTag::as_str).collect();
        // ADR-0010 Karar 5: this is also English < Français < Türkçe.
        assert_eq!(order, ["en", "fr", "tr"]);
    }

    #[test]
    fn ids_of_different_kinds_never_collide() {
        let embedded = SubtitleSourceId::embedded(2);
        let user = SubtitleSourceId::user([2u8; 32]);
        let public = SubtitleSourceId::opensubtitles("2");
        assert_ne!(embedded, public);
        assert_ne!(user, public);
        assert_eq!(embedded, SubtitleSourceId::embedded(2));
        assert_ne!(embedded, SubtitleSourceId::embedded(3));
    }

    #[test]
    fn ai_identity_is_origin_plus_target() {
        let origin = SubtitleSourceId::embedded(1);
        let other = SubtitleSourceId::embedded(2);
        assert_eq!(
            SubtitleSourceId::ai(&origin, &tag("tr")),
            SubtitleSourceId::ai(&origin, &tag("tr"))
        );
        assert_ne!(
            SubtitleSourceId::ai(&origin, &tag("tr")),
            SubtitleSourceId::ai(&origin, &tag("en"))
        );
        assert_ne!(
            SubtitleSourceId::ai(&origin, &tag("tr")),
            SubtitleSourceId::ai(&other, &tag("tr"))
        );
    }

    #[test]
    fn preferences_drop_a_secondary_equal_to_the_primary() {
        let prefs = SubtitlePreferences::new(Some(tag("tr")), Some(tag("TR")));
        assert_eq!(prefs.secondary(), None);
        let ordered: Vec<&str> = prefs.ordered().map(LanguageTag::as_str).collect();
        assert_eq!(ordered, ["tr"]);
    }

    #[test]
    fn preferences_yield_primary_then_secondary() {
        let prefs = SubtitlePreferences::new(Some(tag("tr")), Some(tag("en")));
        let ordered: Vec<&str> = prefs.ordered().map(LanguageTag::as_str).collect();
        assert_eq!(ordered, ["tr", "en"]);
        assert_eq!(SubtitlePreferences::none().ordered().count(), 0);
    }

    #[test]
    fn debug_never_prints_the_label_or_the_key() {
        let source = SubtitleSource::new(
            SubtitleSourceId::user([0xab; 32]),
            Some(tag("tr")),
            "Inception.2010.tr.srt",
        );
        let printed = format!("{source:?} {:?}", source.id());
        assert!(!printed.contains("Inception"));
        assert!(!printed.contains("abab"));
        assert!(printed.contains("User"));
        assert!(printed.contains("label_len"));
    }
}
