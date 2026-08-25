//! What the core hands an engine to play.
//!
//! **Security (K23 #1, #3).** A locator is a media URL or a private full path —
//! two of the flatly forbidden log patterns. This type exists so that the value
//! travels through the port without ever being printable: `Debug` shows only
//! that a locator is present and how long it is.
//!
//! The type deliberately does **not** validate or canonicalize anything. Path
//! traversal and symlink rejection belong to the platform's file access gate
//! (NEN-025, product-spec §10); a value type that silently repaired a locator
//! would hide exactly the input those gates need to see.

use std::fmt;

/// A medium to play, identified opaquely.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MediaSource {
    locator: String,
}

impl MediaSource {
    /// Wraps a locator — a local path or a URL. Never log the input.
    pub fn new(locator: impl Into<String>) -> Self {
        Self {
            locator: locator.into(),
        }
    }

    /// The locator, for the adapter that has to open it. **Never log this.**
    pub fn locator(&self) -> &str {
        &self.locator
    }

    pub fn is_empty(&self) -> bool {
        self.locator.is_empty()
    }
}

impl fmt::Debug for MediaSource {
    /// Prints a length, never the locator.
    ///
    /// A length is a size class, which `docs/security-policy.md` §1 allows; the
    /// locator itself is K23 #1 or #3 depending on where it points, and the
    /// type cannot tell which — so it prints neither.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaSource")
            .field("locator_len", &self.locator.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_prints_no_part_of_the_locator() {
        let source = MediaSource::new("https://cdn.example.com/movie.mkv?token=SECRET");
        let printed = format!("{source:?}");
        for forbidden in ["cdn.example.com", "movie.mkv", "token", "SECRET", "https"] {
            assert!(
                !printed.contains(forbidden),
                "{forbidden} leaked: {printed}"
            );
        }
        assert!(printed.contains("locator_len"));
    }

    #[test]
    fn a_private_path_is_equally_hidden() {
        let source = MediaSource::new("/Users/someone/Movies/Private Folder/film.mkv");
        let printed = format!("{source:?}");
        for forbidden in ["Users", "someone", "Private", "film.mkv"] {
            assert!(
                !printed.contains(forbidden),
                "{forbidden} leaked: {printed}"
            );
        }
    }

    #[test]
    fn the_locator_is_still_available_to_the_adapter() {
        let source = MediaSource::new("/tmp/a.mkv");
        assert_eq!(source.locator(), "/tmp/a.mkv");
        assert!(!source.is_empty());
    }
}
