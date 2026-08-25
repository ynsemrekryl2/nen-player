//! Identity hints from a remote URL's path (ADR-0009 Karar 5).
//!
//! `docs/product-spec.md` §6 originally listed only the URL's *basename* as
//! evidence. ADR-0009 widened that to every path segment, because in the
//! Stremio and debrid flows the basename is routinely meaningless
//! (`stream.mkv`, an opaque id) while the real release name sits one level up:
//! `/movies/Inception.2010.1080p/stream.mkv`. Segments are returned from the
//! basename outwards, so the most specific one is tried first.
//!
//! # What never comes back from here
//!
//! The query string, the fragment and the host are dropped before anything
//! else happens and are never parsed, stored or derived from. §6's closing
//! sentence forbids the query as an identity source outright, and K23 #1/#2
//! keep it out of logs; the host is dropped too because which service the user
//! streams from is itself revealing, and it contributes nothing to identity.

/// Longest single segment we will hand on. Long enough for any real release
/// name, short enough that a hostile URL cannot make the caller allocate.
const MAX_SEGMENT_BYTES: usize = 512;

/// Most segments we will consider, counted from the basename outwards.
const MAX_SEGMENTS: usize = 8;

/// Path segments of `url`, basename first.
///
/// Returns an empty vector for a URL with no usable path — which is the normal
/// result for an opaque streaming endpoint, not an error.
pub fn path_hints(url: &str) -> Vec<String> {
    let path = path_of(url);

    let mut segments: Vec<String> = path
        .split('/')
        .filter_map(percent_decode_segment)
        .filter(|segment| !segment.is_empty() && segment != "." && segment != "..")
        .collect();

    segments.reverse();
    segments.truncate(MAX_SEGMENTS);
    segments
}

/// Strips scheme, authority, query and fragment, leaving only the path.
fn path_of(url: &str) -> &str {
    // Query and fragment go first, before anything can read them.
    let url = url.split(['?', '#']).next().unwrap_or("");

    let after_scheme = match url.split_once("://") {
        Some((_scheme, rest)) => rest,
        None => url,
    };

    if after_scheme.len() == url.len() && !url.starts_with('/') {
        // No scheme and no leading slash: treat the whole thing as a path,
        // which is what a bare `dir/file.mkv` is.
        return after_scheme;
    }

    match after_scheme.find('/') {
        // Everything up to the first slash is the authority, and it is dropped.
        Some(slash) => after_scheme.get(slash + 1..).unwrap_or(""),
        None => "",
    }
}

/// Decodes `%XX` escapes inside one segment.
///
/// Decoding happens **after** the path has been split, so a `%2F` cannot
/// smuggle in a segment boundary — it decodes to a literal `/` inside a single
/// segment, which is then flattened to a space rather than allowed to look
/// like structure.
fn percent_decode_segment(segment: &str) -> Option<String> {
    if segment.len() > MAX_SEGMENT_BYTES {
        return None;
    }

    let mut bytes = Vec::with_capacity(segment.len());
    let mut rest = segment.as_bytes();

    while let Some((&first, tail)) = rest.split_first() {
        if first == b'%' {
            let hex = tail
                .get(..2)
                .and_then(|pair| std::str::from_utf8(pair).ok());
            if let Some(decoded) = hex.and_then(|hex| u8::from_str_radix(hex, 16).ok()) {
                bytes.push(decoded);
                rest = tail.get(2..).unwrap_or(&[]);
                continue;
            }
        }
        bytes.push(first);
        rest = tail;
    }

    let decoded = String::from_utf8_lossy(&bytes);
    let flattened: String = decoded
        .chars()
        .map(|c| if c == '/' || c == '\\' { ' ' } else { c })
        .filter(|c| !c.is_control())
        .collect();

    Some(flattened.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_segments_come_after_the_basename() {
        let hints = path_hints("https://cdn.example.com/movies/Inception.2010.1080p/stream.mkv");
        assert_eq!(hints, vec!["stream.mkv", "Inception.2010.1080p", "movies"]);
    }

    #[test]
    fn the_host_never_appears() {
        let hints = path_hints("https://real-debrid.example.com/d/ABC123/Movie.2010.mkv");
        assert!(!hints.iter().any(|h| h.contains("example.com")));
        assert!(!hints.iter().any(|h| h.contains("real-debrid")));
    }

    #[test]
    fn the_query_string_is_dropped_entirely() {
        let hints = path_hints("https://cdn.example.com/a/Movie.2010.mkv?token=SECRET&exp=99");
        let joined = hints.join(" ");
        assert!(!joined.contains("token"), "query leaked: {joined}");
        assert!(!joined.contains("SECRET"), "token leaked: {joined}");
        assert_eq!(hints.first().map(String::as_str), Some("Movie.2010.mkv"));
    }

    #[test]
    fn the_fragment_is_dropped_entirely() {
        let hints = path_hints("https://cdn.example.com/a/Movie.2010.mkv#t=120&key=SECRET");
        let joined = hints.join(" ");
        assert!(!joined.contains("SECRET"), "fragment leaked: {joined}");
    }

    #[test]
    fn a_query_before_a_slash_cannot_reintroduce_a_path() {
        let hints = path_hints("https://cdn.example.com/a?x=/Secret.2010.mkv");
        assert_eq!(hints, vec!["a"]);
    }

    #[test]
    fn percent_escapes_decode_within_a_segment() {
        let hints = path_hints("https://cdn.example.com/The%20Matrix%20(1999)/video.mkv");
        assert_eq!(hints.get(1).map(String::as_str), Some("The Matrix (1999)"));
    }

    #[test]
    fn an_encoded_slash_cannot_create_a_segment_boundary() {
        let hints = path_hints("https://cdn.example.com/a%2Fb%2Fc/video.mkv");
        assert_eq!(hints.get(1).map(String::as_str), Some("a b c"));
        assert_eq!(hints.len(), 2);
    }

    #[test]
    fn dot_segments_are_dropped() {
        let hints = path_hints("https://cdn.example.com/a/./../b/video.mkv");
        assert_eq!(hints, vec!["video.mkv", "b", "a"]);
    }

    #[test]
    fn an_opaque_endpoint_yields_what_it_has() {
        let hints = path_hints("http://127.0.0.1:11470/8f2a.../0");
        assert_eq!(hints, vec!["0", "8f2a..."]);
    }

    #[test]
    fn a_url_without_a_path_yields_nothing() {
        assert!(path_hints("https://cdn.example.com").is_empty());
        assert!(path_hints("https://cdn.example.com/").is_empty());
        assert!(path_hints("").is_empty());
    }

    #[test]
    fn a_bare_relative_path_is_accepted() {
        let hints = path_hints("movies/Inception.2010/video.mkv");
        assert_eq!(hints, vec!["video.mkv", "Inception.2010", "movies"]);
    }

    #[test]
    fn segment_count_and_length_are_bounded() {
        let deep = format!("https://h/{}/video.mkv", vec!["seg"; 100].join("/"));
        assert!(path_hints(&deep).len() <= MAX_SEGMENTS);

        let long = format!("https://h/{}/video.mkv", "a".repeat(5_000));
        let hints = path_hints(&long);
        assert!(hints.iter().all(|h| h.len() <= MAX_SEGMENT_BYTES));
    }

    #[test]
    fn hostile_input_never_panics() {
        for url in [
            "%",
            "%%%%",
            "%zz",
            "https://",
            "://",
            "/%2e%2e/%2e%2e/etc/passwd",
            "https://h/\u{202e}gnp.evk",
            "https://h/\0/video.mkv",
        ] {
            let _ = path_hints(url);
        }
    }

    #[test]
    fn control_characters_are_stripped() {
        let hints = path_hints("https://h/Movie\u{0007}.2010.mkv");
        assert_eq!(hints.first().map(String::as_str), Some("Movie.2010.mkv"));
    }
}
