//! Identity hints from the directories a local file sits in (ADR-0009 Karar 6,
//! layer 5).
//!
//! This is the local counterpart of [`url_hints`](crate::url_hints): in a Kodi
//! or Plex library the folder is often better named than the file
//! (`Movies/Inception (2010)/video.mkv`), so the parent chain is worth reading
//! when the filename itself says nothing.
//!
//! Directory names are returned nearest-first. Generic library roots
//! (`Movies`, `Downloads`) need no special handling — they carry no year, no
//! season marker and no release tag, so
//! [`release_name::parse`](crate::release_name::parse) already reports them as
//! [`Unknown`](crate::release_name::MediaKind::Unknown) and the walk moves on.
//!
//! # No I/O
//!
//! Nothing here touches the filesystem (ADR-0009 Karar 2). The caller passes
//! the path it already has; this only splits it.

/// Most ancestors we will consider, counted from the file outwards.
const MAX_DEPTH: usize = 6;

/// Longest directory name we will hand on.
const MAX_SEGMENT_BYTES: usize = 512;

/// Parent directory names of `path`, nearest first.
///
/// The final component is treated as the file itself and excluded — it is
/// already covered by the filename layer.
pub fn directory_hints(path: &str) -> Vec<String> {
    let mut components: Vec<String> = path
        .split(['/', '\\'])
        .map(|component| component.trim())
        .filter(|component| {
            !component.is_empty()
                && *component != "."
                && *component != ".."
                && component.len() <= MAX_SEGMENT_BYTES
        })
        .map(|component| component.chars().filter(|c| !c.is_control()).collect())
        .collect();

    // Drop the file itself; what is left are its ancestors.
    components.pop();
    components.reverse();
    components.truncate(MAX_DEPTH);
    components.retain(|component: &String| !component.is_empty());
    components
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parents_are_returned_nearest_first() {
        let hints = directory_hints("/Users/x/Movies/Inception (2010)/video.mkv");
        assert_eq!(hints, vec!["Inception (2010)", "Movies", "x", "Users"]);
    }

    #[test]
    fn the_file_itself_is_not_a_hint() {
        let hints = directory_hints("/library/Show/S01/episode.mkv");
        assert!(!hints.iter().any(|h| h == "episode.mkv"));
    }

    #[test]
    fn windows_separators_work_too() {
        let hints = directory_hints(r"D:\Media\Arrival (2016)\movie.mkv");
        assert_eq!(hints.first().map(String::as_str), Some("Arrival (2016)"));
    }

    #[test]
    fn a_bare_filename_has_no_parents() {
        assert!(directory_hints("video.mkv").is_empty());
        assert!(directory_hints("").is_empty());
        assert!(directory_hints("/").is_empty());
    }

    #[test]
    fn dot_segments_are_dropped() {
        let hints = directory_hints("/a/./../b/video.mkv");
        assert_eq!(hints, vec!["b", "a"]);
    }

    #[test]
    fn depth_and_length_are_bounded() {
        let deep = format!("/{}/video.mkv", vec!["d"; 50].join("/"));
        assert!(directory_hints(&deep).len() <= MAX_DEPTH);

        let long = format!("/{}/video.mkv", "a".repeat(5_000));
        assert!(directory_hints(&long).is_empty());
    }

    #[test]
    fn control_characters_are_stripped() {
        let hints = directory_hints("/lib/Show\u{0007} (2019)/e.mkv");
        assert_eq!(hints.first().map(String::as_str), Some("Show (2019)"));
    }

    #[test]
    fn hostile_input_never_panics() {
        for path in ["////", "\\\\", "\0/\0", &"/".repeat(10_000)] {
            let _ = directory_hints(path);
        }
    }
}
