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
//!   is the normalized tag's byte order: `en` < `fr` < `tr` (Karar 5, 7). A
//!   group is keyed by [`LanguageTag::primary_tag`], not by the full tag
//!   (ADR-0030) — the sources keep their region, the grouping does not use it.
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
/// The scope is deliberately narrow. This is what a subtitle group is sorted by
/// (ADR-0010 Karar 5); script, variant and extension subtags do not change which
/// group a subtitle belongs to, and accepting them would make two spellings of
/// the same group sort apart. An invalid tag is an `Err`, never a silently
/// repaired value — a subtitle whose language could not be established belongs
/// in `Dil Belirsiz`, and that is a `None`, not a guess.
///
/// The region is stored but is **not** the granularity at which two tags count
/// as the same language: grouping and preference matching go through
/// [`primary`](Self::primary) (ADR-0030). Keeping the region here is what lets a
/// UI still show which entry is the Brazilian one (NEN-026).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LanguageTag(String);

impl LanguageTag {
    /// Parses and normalizes a tag. Case and separator case are irrelevant;
    /// `EN-us`, `en-US` and `en-us` are the same tag.
    ///
    /// **The primary subtag is canonicalized too** (ADR-0032): a three-letter
    /// ISO 639-2 code that has an ISO 639-1 equivalent becomes that equivalent,
    /// so `parse("eng")` yields `en` and both `fre` and `fra` yield `fr`. The
    /// input and the output are therefore not always spelled the same — which
    /// is the point: a container writes `eng` where a sidecar writes `en`, and
    /// one language must not become two menu groups.
    ///
    /// A three-letter code with **no** 639-1 equivalent is left exactly as it
    /// is (`fil`, `haw`, `nds`). Nothing is guessed and nothing is truncated.
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

        let lowered = primary.to_ascii_lowercase();
        let mut tag = canonical_primary(&lowered).unwrap_or(lowered);

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

    /// The primary subtag alone — `pt` for both `pt-br` and `pt-pt`.
    ///
    /// This is the granularity at which two tags count as "the same language"
    /// (ADR-0030). Menu grouping, preference matching and `nen-subtitle`'s
    /// metadata/text conflict check all ask that one question; answering it
    /// separately in each place is how `en` and `en-us` ended up as two menu
    /// groups while a `tr-tr` track stayed invisible to a `tr` preference.
    pub fn primary(&self) -> &str {
        self.0
            .split_once('-')
            .map_or(self.0.as_str(), |(primary, _)| primary)
    }

    /// The same tag with its region dropped — the representative a language
    /// group carries (ADR-0030 Karar 1).
    ///
    /// The value is already normalized, so this cannot fail and does not go
    /// back through [`parse`](Self::parse).
    pub fn primary_tag(&self) -> Self {
        Self(self.primary().to_owned())
    }
}

/// The ISO 639-1 code for an ISO 639-2 one, when there is one (ADR-0032).
///
/// Both halves of ISO 639-2 map here: the bibliographic codes (`fre`, `ger`,
/// `chi`) and the terminological ones (`fra`, `deu`, `zho`) land on the same
/// answer, because they are the same language written twice. Twenty languages
/// have both; the rest have one code.
///
/// `None` means "no equivalent exists" — the caller keeps what it had. That is
/// the correct answer for `fil`, `haw` and `nds`, whose only code is the
/// three-letter one, and it is also what makes this table safe to be
/// incomplete: a missing row degrades to today's behaviour rather than to a
/// wrong language.
///
/// Input must already be lowercase; `LanguageTag::parse` is the only caller and
/// lowercases first.
fn canonical_primary(code: &str) -> Option<String> {
    // Only three-letter codes can be 639-2. Checking here keeps the match
    // below from ever being consulted for a tag that is already canonical.
    if code.len() != 3 {
        return None;
    }
    let two: Option<&str> = match code {
        "aar" => Some("aa"),
        "abk" => Some("ab"),
        "afr" => Some("af"),
        "aka" => Some("ak"),
        "alb" => Some("sq"),
        "amh" => Some("am"),
        "ara" => Some("ar"),
        "arg" => Some("an"),
        "arm" => Some("hy"),
        "asm" => Some("as"),
        "ava" => Some("av"),
        "ave" => Some("ae"),
        "aym" => Some("ay"),
        "aze" => Some("az"),
        "bak" => Some("ba"),
        "bam" => Some("bm"),
        "baq" => Some("eu"),
        "bel" => Some("be"),
        "ben" => Some("bn"),
        "bih" => Some("bh"),
        "bis" => Some("bi"),
        "bod" => Some("bo"),
        "bos" => Some("bs"),
        "bre" => Some("br"),
        "bul" => Some("bg"),
        "bur" => Some("my"),
        "cat" => Some("ca"),
        "ces" => Some("cs"),
        "cha" => Some("ch"),
        "che" => Some("ce"),
        "chi" => Some("zh"),
        "chu" => Some("cu"),
        "chv" => Some("cv"),
        "cor" => Some("kw"),
        "cos" => Some("co"),
        "cre" => Some("cr"),
        "cym" => Some("cy"),
        "cze" => Some("cs"),
        "dan" => Some("da"),
        "deu" => Some("de"),
        "div" => Some("dv"),
        "dut" => Some("nl"),
        "dzo" => Some("dz"),
        "ell" => Some("el"),
        "eng" => Some("en"),
        "epo" => Some("eo"),
        "est" => Some("et"),
        "eus" => Some("eu"),
        "ewe" => Some("ee"),
        "fao" => Some("fo"),
        "fas" => Some("fa"),
        "fij" => Some("fj"),
        "fin" => Some("fi"),
        "fra" => Some("fr"),
        "fre" => Some("fr"),
        "fry" => Some("fy"),
        "ful" => Some("ff"),
        "geo" => Some("ka"),
        "ger" => Some("de"),
        "gla" => Some("gd"),
        "gle" => Some("ga"),
        "glg" => Some("gl"),
        "glv" => Some("gv"),
        "gre" => Some("el"),
        "grn" => Some("gn"),
        "guj" => Some("gu"),
        "hat" => Some("ht"),
        "hau" => Some("ha"),
        "heb" => Some("he"),
        "her" => Some("hz"),
        "hin" => Some("hi"),
        "hmo" => Some("ho"),
        "hrv" => Some("hr"),
        "hun" => Some("hu"),
        "hye" => Some("hy"),
        "ibo" => Some("ig"),
        "ice" => Some("is"),
        "ido" => Some("io"),
        "iii" => Some("ii"),
        "iku" => Some("iu"),
        "ile" => Some("ie"),
        "ina" => Some("ia"),
        "ind" => Some("id"),
        "ipk" => Some("ik"),
        "isl" => Some("is"),
        "ita" => Some("it"),
        "jav" => Some("jv"),
        "jpn" => Some("ja"),
        "kal" => Some("kl"),
        "kan" => Some("kn"),
        "kas" => Some("ks"),
        "kat" => Some("ka"),
        "kau" => Some("kr"),
        "kaz" => Some("kk"),
        "khm" => Some("km"),
        "kik" => Some("ki"),
        "kin" => Some("rw"),
        "kir" => Some("ky"),
        "kom" => Some("kv"),
        "kon" => Some("kg"),
        "kor" => Some("ko"),
        "kua" => Some("kj"),
        "kur" => Some("ku"),
        "lao" => Some("lo"),
        "lat" => Some("la"),
        "lav" => Some("lv"),
        "lim" => Some("li"),
        "lin" => Some("ln"),
        "lit" => Some("lt"),
        "ltz" => Some("lb"),
        "lub" => Some("lu"),
        "lug" => Some("lg"),
        "mac" => Some("mk"),
        "mah" => Some("mh"),
        "mal" => Some("ml"),
        "mao" => Some("mi"),
        "mar" => Some("mr"),
        "may" => Some("ms"),
        "mkd" => Some("mk"),
        "mlg" => Some("mg"),
        "mlt" => Some("mt"),
        "mon" => Some("mn"),
        "mri" => Some("mi"),
        "msa" => Some("ms"),
        "mya" => Some("my"),
        "nau" => Some("na"),
        "nav" => Some("nv"),
        "nbl" => Some("nr"),
        "nde" => Some("nd"),
        "ndo" => Some("ng"),
        "nep" => Some("ne"),
        "nld" => Some("nl"),
        "nno" => Some("nn"),
        "nob" => Some("nb"),
        "nor" => Some("no"),
        "nya" => Some("ny"),
        "oci" => Some("oc"),
        "oji" => Some("oj"),
        "ori" => Some("or"),
        "orm" => Some("om"),
        "oss" => Some("os"),
        "pan" => Some("pa"),
        "per" => Some("fa"),
        "pli" => Some("pi"),
        "pol" => Some("pl"),
        "por" => Some("pt"),
        "pus" => Some("ps"),
        "que" => Some("qu"),
        "roh" => Some("rm"),
        "ron" => Some("ro"),
        "rum" => Some("ro"),
        "run" => Some("rn"),
        "rus" => Some("ru"),
        "sag" => Some("sg"),
        "san" => Some("sa"),
        "sin" => Some("si"),
        "slk" => Some("sk"),
        "slo" => Some("sk"),
        "slv" => Some("sl"),
        "sme" => Some("se"),
        "smo" => Some("sm"),
        "sna" => Some("sn"),
        "snd" => Some("sd"),
        "som" => Some("so"),
        "sot" => Some("st"),
        "spa" => Some("es"),
        "sqi" => Some("sq"),
        "srd" => Some("sc"),
        "srp" => Some("sr"),
        "ssw" => Some("ss"),
        "sun" => Some("su"),
        "swa" => Some("sw"),
        "swe" => Some("sv"),
        "tah" => Some("ty"),
        "tam" => Some("ta"),
        "tat" => Some("tt"),
        "tel" => Some("te"),
        "tgk" => Some("tg"),
        "tgl" => Some("tl"),
        "tha" => Some("th"),
        "tib" => Some("bo"),
        "tir" => Some("ti"),
        "ton" => Some("to"),
        "tsn" => Some("tn"),
        "tso" => Some("ts"),
        "tuk" => Some("tk"),
        "tur" => Some("tr"),
        "twi" => Some("tw"),
        "uig" => Some("ug"),
        "ukr" => Some("uk"),
        "urd" => Some("ur"),
        "uzb" => Some("uz"),
        "ven" => Some("ve"),
        "vie" => Some("vi"),
        "vol" => Some("vo"),
        "wel" => Some("cy"),
        "wln" => Some("wa"),
        "wol" => Some("wo"),
        "xho" => Some("xh"),
        "yid" => Some("yi"),
        "yor" => Some("yo"),
        "zha" => Some("za"),
        "zho" => Some("zh"),
        "zul" => Some("zu"),
        _ => None,
    };
    two.map(str::to_owned)
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
    /// The digest is computed by the caller — the M3 file adapter that has the
    /// path in the first place (`NEN-025`) — because this crate has no
    /// dependencies and no hash of its own. Taking a digest rather than a path
    /// is what keeps K23 #3 satisfied by construction: there is no code path by
    /// which a raw path can be stored here.
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

    /// The track index, when this identifies an embedded track.
    ///
    /// The inverse of [`embedded`](Self::embedded), and it lives next to it so
    /// the two cannot drift: something has to turn "the user picked this menu
    /// row" back into "select that track", and a second parser elsewhere would
    /// be a second opinion about what the key means.
    ///
    /// `None` for every other kind — a user file is a perfectly ordinary entry
    /// that simply is not a track.
    pub fn embedded_index(&self) -> Option<u32> {
        match self.kind {
            SubtitleSourceKind::Embedded => self.key.parse().ok(),
            _ => None,
        }
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
    translatable: bool,
}

impl SubtitleSource {
    /// A source that can be translated -- the case for everything except a
    /// bitmap embedded track, so it is the default and callers that mean it
    /// say nothing.
    pub fn new(
        id: SubtitleSourceId,
        language: Option<LanguageTag>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id,
            language,
            label: label.into(),
            translatable: true,
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
            translatable: self.translatable,
        }
    }

    /// Whether this source can be the input of an AI translation (§7, §9).
    ///
    /// `false` for a bitmap embedded track: it draws pictures, so there is no
    /// text to translate. **Untranslatable is not unselectable** -- §7 says
    /// such a track "gösterilebilir fakat çevrilemez olarak işaretlenebilir",
    /// so nothing here may be used to hide it or to skip it in auto-selection.
    pub fn translatable(&self) -> bool {
        self.translatable
    }

    /// Returns a copy carrying a different translatability, keeping identity,
    /// language and label.
    pub fn with_translatable(&self, translatable: bool) -> Self {
        Self {
            id: self.id.clone(),
            language: self.language.clone(),
            label: self.label.clone(),
            translatable,
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
            .field("translatable", &self.translatable)
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
    fn iso_639_2_codes_become_their_iso_639_1_equivalent() {
        // ADR-0032. These four were measured coming out of a real container:
        // Matroska writes ISO 639-2, so this is what an embedded track says.
        assert_eq!(tag("eng").as_str(), "en");
        assert_eq!(tag("tur").as_str(), "tr");
        assert_eq!(tag("fre").as_str(), "fr");
        assert_eq!(tag("ger").as_str(), "de");
    }

    #[test]
    fn the_bibliographic_and_terminological_codes_agree() {
        // The twenty languages ISO 639-2 spells twice must not become two
        // groups. `fre`/`fra`, `ger`/`deu`, `chi`/`zho`, `dut`/`nld`.
        for (bibliographic, terminological) in [
            ("fre", "fra"),
            ("ger", "deu"),
            ("chi", "zho"),
            ("dut", "nld"),
        ] {
            assert_eq!(
                tag(bibliographic),
                tag(terminological),
                "{bibliographic} and {terminological} disagreed"
            );
        }
    }

    #[test]
    fn a_container_track_and_a_sidecar_land_in_the_same_group() {
        // The failure this decision exists for: the user's `Movie.en.srt` says
        // `en`, the embedded track inside the same film says `eng`.
        assert_eq!(tag("eng"), tag("en"));
        assert_eq!(tag("eng").primary_tag(), tag("en").primary_tag());
    }

    #[test]
    fn a_three_letter_code_without_an_equivalent_is_left_alone() {
        // These languages have no ISO 639-1 code at all; the three-letter tag
        // is the only correct one. Guessing or truncating would be worse than
        // doing nothing.
        for code in ["fil", "haw", "nds", "ceb"] {
            assert_eq!(tag(code).as_str(), code);
        }
    }

    #[test]
    fn canonicalization_does_not_touch_a_region() {
        assert_eq!(tag("eng-US").as_str(), "en-us");
        assert_eq!(tag("por-BR").as_str(), "pt-br");
    }

    #[test]
    fn a_two_letter_tag_is_never_run_through_the_table() {
        // The table is keyed by three-letter codes; a two-letter tag that
        // happens to collide with one must pass through untouched.
        for code in ["en", "tr", "fr", "aa", "zu"] {
            assert_eq!(tag(code).as_str(), code);
        }
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
    fn primary_drops_the_region_and_leaves_a_region_free_tag_alone() {
        // ADR-0030 Karar 1/2: the granularity grouping and preference matching
        // work at. The tag itself keeps its region -- Karar 3.
        assert_eq!(tag("en-us").primary(), "en");
        assert_eq!(tag("pt-br").primary(), "pt");
        assert_eq!(tag("es-419").primary(), "es");
        assert_eq!(tag("tr").primary(), "tr");

        assert_eq!(tag("en-us").primary_tag(), tag("en"));
        assert_eq!(tag("pt-br").primary_tag(), tag("pt"));
        assert_eq!(tag("tr").primary_tag(), tag("tr"));

        assert_eq!(
            tag("en-us").as_str(),
            "en-us",
            "the source's own tag is untouched (ADR-0029 Karar 5)"
        );
    }

    #[test]
    fn two_regions_of_one_language_share_a_primary_tag() {
        // The pair the ADR argues about: pt-br and pt-pt are different
        // subtitles but one menu group.
        assert_eq!(tag("pt-br").primary_tag(), tag("pt-pt").primary_tag());
        assert_ne!(tag("pt-br"), tag("pt-pt"));
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
    fn an_embedded_id_resolves_back_to_its_track_index() {
        for index in [0u32, 1, 7, u32::MAX] {
            assert_eq!(
                SubtitleSourceId::embedded(index).embedded_index(),
                Some(index)
            );
        }
    }

    #[test]
    fn only_an_embedded_id_names_a_track() {
        // A user key is 64 hex characters and an OpenSubtitles key is whatever
        // the provider says — neither may be read as an index just because it
        // happens to parse.
        assert_eq!(SubtitleSourceId::user([0u8; 32]).embedded_index(), None);
        assert_eq!(SubtitleSourceId::opensubtitles("12").embedded_index(), None);
        assert_eq!(
            SubtitleSourceId::ai(&SubtitleSourceId::embedded(1), &tag("tr")).embedded_index(),
            None
        );
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
    fn a_source_is_translatable_unless_it_says_otherwise() {
        let source = SubtitleSource::new(SubtitleSourceId::embedded(1), Some(tag("en")), "English");
        assert!(source.translatable());

        let bitmap = source.with_translatable(false);
        assert!(!bitmap.translatable());
        // Identity, language and label survive the mark -- the menu must not
        // lose the row it is describing.
        assert_eq!(bitmap.id(), source.id());
        assert_eq!(bitmap.language(), source.language());
        assert_eq!(bitmap.label(), source.label());
    }

    #[test]
    fn translatability_survives_a_language_correction() {
        // NEN-020 fills in a language after the fact; doing so must not quietly
        // re-open a bitmap track for translation.
        let bitmap =
            SubtitleSource::new(SubtitleSourceId::embedded(1), None, "").with_translatable(false);
        assert!(!bitmap.with_language(Some(tag("tr"))).translatable());
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
