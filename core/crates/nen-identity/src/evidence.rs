//! The evidence a media file brings with it, and how identity is read from it
//! (ADR-0009).
//!
//! [`MediaEvidence`] is the collected input; [`MediaEvidence::resolve`] walks
//! it in the order ADR-0009 Karar 6 fixes, and
//! [`MediaEvidence::identity_candidates`] projects the result into the
//! fallback sequence `docs/product-spec.md` §6 specifies.
//!
//! # The order, and why it is that order
//!
//! | # | Layer | Kind |
//! |---|---|---|
//! | 1 | Stremio handoff metadata | declaration |
//! | 2 | `.nfo` sidecar | declaration |
//! | 3 | Container tags | declaration |
//! | 4 | Declared name — local filename, or the server's `Content-Disposition` | declaration |
//! | 5 | Parent directory names | inference |
//! | 6 | Sibling agreement | corroboration |
//! | 7 | Remote URL path segments | inference |
//!
//! Declarations come first because somebody stated them; inferences are us
//! guessing from a name. Within the declarations the order runs from most
//! deliberate (a handoff, a curated sidecar) to most incidental (whatever the
//! file happens to be called).
//!
//! The first layer that produces a [usable](crate::release_name::ParsedName::is_usable)
//! parse wins and owns the title and kind. Later layers may still fill in a
//! missing year, season or episode — a `S01E02.mkv` inside `Breaking Bad
//! (2008)/` is the ordinary case, and refusing to combine them would lose
//! information no layer had alone.
//!
//! # Redaction
//!
//! Every field here is either a private path, a filename, a media fingerprint
//! or a title — K23 #3 and #8 in some combination. `Debug` is written by hand
//! and prints only the file extension, the size class and which layers are
//! present. Do not replace it with `#[derive(Debug)]`; `guard_evidence_debug`
//! proves that a derived one leaks.

use std::fmt;

use nen_domain::redact::{extension, size_class};

use crate::container::{self, ContainerMetadata};
use crate::declared_name;
use crate::nfo::Nfo;
use crate::os_hash::OsHash;
use crate::release_name::{self, MediaKind, ParsedName};
use crate::siblings::{self, Corroboration};

/// Identity a launcher handed us, e.g. Stremio passing what it already knows.
///
/// `docs/product-spec.md` §5 calls this strong evidence but warns it must
/// never be assumed present — hence every field optional and the whole struct
/// optional on [`MediaEvidence`].
#[derive(Clone, PartialEq, Eq, Default)]
pub struct HandoffMetadata {
    pub title: Option<String>,
    pub year: Option<u16>,
    pub season: Option<u16>,
    pub episode: Option<u16>,
}

impl HandoffMetadata {
    fn to_parsed(&self) -> ParsedName {
        let kind = if self.episode.is_some() || self.season.is_some() {
            MediaKind::Series
        } else if self.title.is_some() {
            MediaKind::Movie
        } else {
            MediaKind::Unknown
        };
        ParsedName {
            title: self.title.clone(),
            year: self.year,
            season: self.season,
            episode: self.episode,
            kind,
        }
    }
}

impl fmt::Debug for HandoffMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandoffMetadata")
            .field("title", &presence(self.title.is_some()))
            .field("year", &presence(self.year.is_some()))
            .field("season", &presence(self.season.is_some()))
            .field("episode", &presence(self.episode.is_some()))
            .finish()
    }
}

/// Everything known about one piece of media, before identity is worked out.
///
/// Build with [`MediaEvidence::for_local_file`] or
/// [`MediaEvidence::for_remote_url`] and add layers with the `with_*` methods;
/// the constructors are what keep a URL's query out of the struct, so
/// assembling one field by field is deliberately not possible.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct MediaEvidence {
    file_name: Option<String>,
    declared_name: Option<String>,
    size_bytes: Option<u64>,
    os_hash: Option<OsHash>,
    container: Option<ContainerMetadata>,
    handoff: Option<HandoffMetadata>,
    nfo: Option<Nfo>,
    dir_hints: Vec<String>,
    url_hints: Vec<String>,
    sibling_names: Vec<String>,
}

impl MediaEvidence {
    /// Evidence for a file on disk. `path` supplies both the filename and the
    /// parent-directory chain.
    pub fn for_local_file(path: &str) -> Self {
        let file_name = path
            .rsplit(['/', '\\'])
            .next()
            .and_then(declared_name::sanitize);

        Self {
            file_name,
            dir_hints: crate::dir_hints::directory_hints(path),
            ..Self::default()
        }
    }

    /// Evidence for a remote stream.
    ///
    /// Only the URL's path contributes (ADR-0009 Karar 5): the query, the
    /// fragment and the host are dropped by
    /// [`url_hints::path_hints`](crate::url_hints::path_hints) before this
    /// sees them, and no field on the struct can hold them.
    pub fn for_remote_url(url: &str) -> Self {
        Self {
            url_hints: crate::url_hints::path_hints(url),
            ..Self::default()
        }
    }

    /// The name a server declared for this stream, from `Content-Disposition`
    /// or the end of the redirect chain. Sanitized to a single path segment
    /// on the way in; a value that reduces to nothing is simply not recorded.
    ///
    /// The adapter that actually performs the request is NEN-036.
    pub fn with_declared_name(mut self, raw: &str) -> Self {
        self.declared_name = declared_name::sanitize(raw);
        self
    }

    pub fn with_size(mut self, size_bytes: u64) -> Self {
        self.size_bytes = Some(size_bytes);
        self
    }

    pub fn with_os_hash(mut self, hash: OsHash) -> Self {
        self.os_hash = Some(hash);
        self
    }

    pub fn with_container(mut self, metadata: ContainerMetadata) -> Self {
        self.container = Some(metadata);
        self
    }

    pub fn with_handoff(mut self, handoff: HandoffMetadata) -> Self {
        self.handoff = Some(handoff);
        self
    }

    pub fn with_nfo(mut self, nfo: Nfo) -> Self {
        self.nfo = Some(nfo);
        self
    }

    /// Names of the other files in the same folder, for corroboration.
    pub fn with_siblings(mut self, names: Vec<String>) -> Self {
        self.sibling_names = names;
        self
    }

    pub fn os_hash(&self) -> Option<OsHash> {
        self.os_hash
    }

    pub fn size_bytes(&self) -> Option<u64> {
        self.size_bytes
    }

    /// Whether a server declaration or final URL basename supplied a name.
    /// The name itself stays private to identity consumers (K23 #8).
    pub fn has_declared_name(&self) -> bool {
        self.declared_name.is_some()
    }

    pub fn container(&self) -> Option<&ContainerMetadata> {
        self.container.as_ref()
    }

    pub fn nfo(&self) -> Option<&Nfo> {
        self.nfo.as_ref()
    }

    /// The IMDb id a sidecar declared, if any. Strongest single piece of
    /// evidence when present, and the only one that needs no matching at all.
    pub fn imdb_id(&self) -> Option<&str> {
        self.nfo.as_ref()?.imdb_id.as_deref()
    }

    /// Walks the evidence layers in ADR-0009 Karar 6 order.
    ///
    /// Always returns a value; when nothing is recognizable that value is
    /// [`MediaKind::Unknown`], which §6 requires to be a normal outcome rather
    /// than an error — playback does not depend on it.
    pub fn resolve(&self) -> ParsedName {
        let layers = self.layers();

        let mut winner = layers
            .iter()
            .find(|parsed| parsed.is_usable())
            .cloned()
            .unwrap_or_else(ParsedName::unknown);

        for layer in &layers {
            winner.fill_gaps_from(layer);
        }

        if winner.title.is_none() {
            // An episode number with no series name: the neighbours may know
            // it. This fills a gap; it never overrules a title.
            if let Some(title) = siblings::consensus_title(&self.sibling_names) {
                winner.title = Some(title);
            }
        }

        // A season or episode number settles the kind, whichever layer
        // supplied it. `Breaking Bad (2008)/S01E02.mkv` is the ordinary case:
        // the folder reads as a movie-with-year because a folder name carries
        // no episode marker, and the filename carries the marker but no name.
        // Neither layer is wrong; together they describe a series, and
        // reporting `Movie` with a season attached would be incoherent. This
        // matches `release_name`'s own rule.
        if winner.episode.is_some() || winner.season.is_some() {
            winner.kind = MediaKind::Series;
        }

        winner
    }

    /// Whether the neighbouring files agree with the resolved identity.
    ///
    /// Reported, never applied — see [`crate::siblings`] for why.
    pub fn corroboration(&self) -> Corroboration {
        siblings::corroborate(&self.resolve(), &self.sibling_names)
    }

    /// The §6 fallback sequence, most specific first.
    ///
    /// Movies degrade `title + year` to `title`; series offer
    /// `series + season + episode` and then the series alone. An unresolvable
    /// file yields an empty list — the point at which a caller may fall back
    /// to asking, which ADR-0009 Karar 7 makes a last resort.
    pub fn identity_candidates(&self) -> Vec<IdentityCandidate> {
        let resolved = self.resolve();
        let Some(title) = resolved.title.clone() else {
            return Vec::new();
        };

        let mut candidates = Vec::new();
        match resolved.kind {
            MediaKind::Series => {
                if resolved.episode.is_some() {
                    candidates.push(IdentityCandidate {
                        title: title.clone(),
                        year: resolved.year,
                        season: resolved.season,
                        episode: resolved.episode,
                    });
                }
                candidates.push(IdentityCandidate {
                    title,
                    year: resolved.year,
                    season: None,
                    episode: None,
                });
            }
            MediaKind::Movie => {
                if resolved.year.is_some() {
                    candidates.push(IdentityCandidate {
                        title: title.clone(),
                        year: resolved.year,
                        season: None,
                        episode: None,
                    });
                }
                candidates.push(IdentityCandidate {
                    title,
                    year: None,
                    season: None,
                    episode: None,
                });
            }
            MediaKind::Unknown => {}
        }
        candidates
    }

    /// Every layer's parse, in ADR-0009 Karar 6 order.
    fn layers(&self) -> Vec<ParsedName> {
        let mut layers = Vec::with_capacity(8);

        if let Some(handoff) = &self.handoff {
            layers.push(handoff.to_parsed());
        }
        if let Some(nfo) = &self.nfo {
            layers.push(nfo_to_parsed(nfo));
        }
        if let Some(metadata) = &self.container {
            layers.push(container::to_parsed(metadata));
        }
        if let Some(name) = self.file_name.as_deref().or(self.declared_name.as_deref()) {
            layers.push(release_name::parse(name));
        }
        for hint in &self.dir_hints {
            layers.push(release_name::parse(hint));
        }
        for hint in &self.url_hints {
            layers.push(release_name::parse(hint));
        }

        layers
    }
}

/// A `.nfo` states its fields outright, so unlike a filename its title needs
/// no release-tag boundary to be believed.
fn nfo_to_parsed(nfo: &Nfo) -> ParsedName {
    let kind = if nfo.episode.is_some() || nfo.season.is_some() {
        MediaKind::Series
    } else if nfo.title.is_some() {
        MediaKind::Movie
    } else {
        MediaKind::Unknown
    };
    ParsedName {
        title: nfo.title.clone(),
        year: nfo.year,
        season: nfo.season,
        episode: nfo.episode,
        kind,
    }
}

impl fmt::Debug for MediaEvidence {
    /// Extension and size class are the only two values `docs/security-policy.md`
    /// lists as loggable; everything else is reported as present or absent.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaEvidence")
            .field(
                "extension",
                &self
                    .file_name
                    .as_deref()
                    .or(self.declared_name.as_deref())
                    .and_then(extension)
                    .unwrap_or("<none>"),
            )
            .field(
                "size_class",
                &self.size_bytes.map_or("<none>", |bytes| size_class(bytes)),
            )
            .field("file_name", &presence(self.file_name.is_some()))
            .field("declared_name", &presence(self.declared_name.is_some()))
            .field("os_hash", &presence(self.os_hash.is_some()))
            .field("container", &presence(self.container.is_some()))
            .field("handoff", &presence(self.handoff.is_some()))
            .field("nfo", &presence(self.nfo.is_some()))
            .field("dir_hints", &self.dir_hints.len())
            .field("url_hints", &self.url_hints.len())
            .field("sibling_names", &self.sibling_names.len())
            .finish()
    }
}

/// One thing to try when looking the media up, in §6's fallback order.
#[derive(Clone, PartialEq, Eq)]
pub struct IdentityCandidate {
    pub title: String,
    pub year: Option<u16>,
    pub season: Option<u16>,
    pub episode: Option<u16>,
}

impl fmt::Debug for IdentityCandidate {
    /// Shape only, same reasoning as [`ParsedName`] (K23 #8).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IdentityCandidate")
            .field("title", &"<present>")
            .field("year", &presence(self.year.is_some()))
            .field("season", &presence(self.season.is_some()))
            .field("episode", &presence(self.episode.is_some()))
            .finish()
    }
}

fn presence(present: bool) -> &'static str {
    if present {
        "<present>"
    } else {
        "<none>"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::from_tags;
    use crate::nfo;

    #[test]
    fn a_local_scene_file_resolves_from_its_name() {
        let evidence =
            MediaEvidence::for_local_file("/Users/x/Movies/Inception.2010.1080p.BluRay.mkv");
        let resolved = evidence.resolve();

        assert_eq!(resolved.title.as_deref(), Some("Inception"));
        assert_eq!(resolved.year, Some(2010));
        assert_eq!(resolved.kind, MediaKind::Movie);
    }

    #[test]
    fn the_directory_rescues_a_meaningless_filename() {
        let evidence = MediaEvidence::for_local_file("/lib/Movies/Arrival (2016)/video.mkv");
        let resolved = evidence.resolve();

        assert_eq!(resolved.title.as_deref(), Some("Arrival"));
        assert_eq!(resolved.year, Some(2016));
    }

    #[test]
    fn a_url_path_segment_rescues_an_opaque_basename() {
        let evidence = MediaEvidence::for_remote_url(
            "https://cdn.example.com/movies/Inception.2010.1080p/stream.mkv",
        );
        let resolved = evidence.resolve();

        assert_eq!(resolved.title.as_deref(), Some("Inception"));
        assert_eq!(resolved.year, Some(2010));
    }

    #[test]
    fn a_declared_name_beats_the_url_path() {
        let evidence = MediaEvidence::for_remote_url("https://cdn.example.com/d/ABC123/0")
            .with_declared_name("Arrival.2016.1080p.BluRay.x264.mkv");
        let resolved = evidence.resolve();

        assert_eq!(resolved.title.as_deref(), Some("Arrival"));
        assert_eq!(resolved.year, Some(2016));
    }

    #[test]
    fn handoff_outranks_every_other_layer() {
        let evidence = MediaEvidence::for_local_file("/lib/Wrong.Title.1999.1080p.mkv")
            .with_declared_name("Also.Wrong.2001.mkv")
            .with_nfo(nfo::parse("<movie><title>Nfo Title</title></movie>"))
            .with_handoff(HandoffMetadata {
                title: Some("Right Title".into()),
                year: Some(2020),
                ..HandoffMetadata::default()
            });

        assert_eq!(evidence.resolve().title.as_deref(), Some("Right Title"));
        assert_eq!(evidence.resolve().year, Some(2020));
    }

    #[test]
    fn the_sidecar_outranks_the_container_and_the_filename() {
        let evidence = MediaEvidence::for_local_file("/lib/Wrong.1999.1080p.mkv")
            .with_container(from_tags(Some("Container Title"), None, None, &[]))
            .with_nfo(nfo::parse(
                "<movie><title>Sidecar Title</title><year>2012</year></movie>",
            ));

        assert_eq!(evidence.resolve().title.as_deref(), Some("Sidecar Title"));
        assert_eq!(evidence.resolve().year, Some(2012));
    }

    #[test]
    fn the_container_outranks_the_filename() {
        let evidence = MediaEvidence::for_local_file("/lib/Wrong.1999.1080p.mkv")
            .with_container(from_tags(Some("Container Title"), Some("2012"), None, &[]));

        assert_eq!(evidence.resolve().title.as_deref(), Some("Container Title"));
    }

    #[test]
    fn a_lower_layer_fills_a_missing_year_without_taking_the_title() {
        let evidence = MediaEvidence::for_local_file("/lib/Breaking Bad (2008)/S01E02.mkv");
        let resolved = evidence.resolve();

        // The filename owns nothing usable (no title), so the directory wins.
        assert_eq!(resolved.title.as_deref(), Some("Breaking Bad"));
        assert_eq!(resolved.year, Some(2008));
        assert_eq!(resolved.season, Some(1));
        assert_eq!(resolved.episode, Some(2));
        assert_eq!(resolved.kind, MediaKind::Series);
    }

    #[test]
    fn siblings_supply_a_series_name_the_file_lacks() {
        let evidence =
            MediaEvidence::for_local_file("/lib/season one/S01E02.mkv").with_siblings(vec![
                "Breaking.Bad.S01E01.mkv".into(),
                "Breaking.Bad.S01E03.mkv".into(),
            ]);
        let resolved = evidence.resolve();

        assert_eq!(resolved.title.as_deref(), Some("Breaking Bad"));
        assert_eq!(resolved.kind, MediaKind::Series);
        assert_eq!(evidence.corroboration(), Corroboration::Confirmed);
    }

    #[test]
    fn nothing_recognizable_resolves_to_unknown_not_an_error() {
        let evidence = MediaEvidence::for_remote_url("http://127.0.0.1:11470/8f2a9c/0");
        let resolved = evidence.resolve();

        assert_eq!(resolved.kind, MediaKind::Unknown);
        assert_eq!(resolved.title, None);
        assert!(evidence.identity_candidates().is_empty());
    }

    #[test]
    fn movie_candidates_degrade_from_title_year_to_title() {
        let evidence = MediaEvidence::for_local_file("/x/Inception.2010.1080p.mkv");
        let candidates = evidence.identity_candidates();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates.first().map(|c| c.year), Some(Some(2010)));
        assert_eq!(candidates.get(1).map(|c| c.year), Some(None));
        assert!(candidates.iter().all(|c| c.title == "Inception"));
    }

    #[test]
    fn series_candidates_degrade_from_episode_to_series() {
        let evidence = MediaEvidence::for_local_file("/x/Breaking.Bad.S01E02.720p.mkv");
        let candidates = evidence.identity_candidates();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates.first().map(|c| c.episode), Some(Some(2)));
        assert_eq!(candidates.get(1).map(|c| c.episode), Some(None));
    }

    #[test]
    fn a_url_query_never_enters_the_evidence() {
        let evidence = MediaEvidence::for_remote_url(
            "https://cdn.example.com/a/Movie.2010.mkv?token=SUPERSECRET&user=alice",
        );
        let printed = format!("{evidence:?} {:?}", evidence.resolve());
        let resolved = evidence.resolve();
        let everything = format!(
            "{printed} {:?} {:?}",
            resolved.title,
            evidence
                .identity_candidates()
                .first()
                .map(|c| c.title.clone())
        );

        assert!(!everything.contains("SUPERSECRET"), "token leaked");
        assert!(!everything.contains("alice"), "user leaked");
        assert!(!everything.contains("token"), "query key leaked");
        assert!(!everything.contains("example.com"), "host leaked");
    }

    #[test]
    fn debug_shows_only_extension_and_size_class() {
        let evidence = MediaEvidence::for_local_file("/Users/alice/Movies/Inception.2010.mkv")
            .with_size(8 * 1024 * 1024 * 1024);
        let printed = format!("{evidence:?}");

        assert!(printed.contains("mkv"), "extension should be loggable");
        assert!(printed.contains("huge"), "size class should be loggable");
        assert!(!printed.contains("alice"), "private path leaked: {printed}");
        assert!(!printed.contains("Inception"), "filename leaked: {printed}");
        assert!(!printed.contains("2010"), "filename leaked: {printed}");
    }

    #[test]
    fn the_hash_never_prints_through_the_evidence() {
        let hash = crate::os_hash::of_bytes(&vec![7u8; 200_000]).unwrap();
        let evidence = MediaEvidence::for_local_file("/x/a.mkv").with_os_hash(hash);
        let printed = format!("{evidence:?}");

        assert!(!printed.contains(&hash.to_hex()), "hash leaked: {printed}");
    }

    #[test]
    fn a_traversal_in_a_declared_name_cannot_become_a_path() {
        let evidence = MediaEvidence::for_remote_url("https://h/a/0")
            .with_declared_name("../../../etc/Movie.2010.mkv");
        assert_eq!(evidence.resolve().title.as_deref(), Some("Movie"));
    }

    #[test]
    fn a_local_filename_outranks_a_declared_one() {
        // Both are layer 4; a real local file's own name is the more direct
        // statement, so the server's declaration only applies when there is
        // no local name.
        let evidence = MediaEvidence::for_local_file("/x/Local.2001.1080p.mkv")
            .with_declared_name("Remote.2002.1080p.mkv");
        assert_eq!(evidence.resolve().year, Some(2001));
    }

    #[test]
    fn candidates_debug_never_prints_the_title() {
        let evidence = MediaEvidence::for_local_file("/x/Inception.2010.1080p.mkv");
        let printed = format!("{:?}", evidence.identity_candidates());
        assert!(!printed.contains("Inception"), "title leaked: {printed}");
    }

    #[test]
    fn hostile_input_never_panics() {
        for path in ["", "/", "\0", &"a/".repeat(10_000), "\u{202e}.mkv"] {
            let evidence = MediaEvidence::for_local_file(path)
                .with_declared_name(path)
                .with_siblings(vec![path.to_string()]);
            let _ = evidence.resolve();
            let _ = evidence.identity_candidates();
            let _ = evidence.corroboration();
            let _ = format!("{evidence:?}");
        }
    }
}
