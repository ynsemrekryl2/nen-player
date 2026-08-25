//! OpenSubtitles-compatible file hash (ADR-0009 Karar 2).
//!
//! The OSDb hash is `file_size` plus the little-endian `u64` words of the
//! file's first and last 64 KiB, all added with wrapping arithmetic. It exists
//! because it identifies a *file* without reading it: two 64 KiB windows and a
//! size, rather than a full-content digest. That property is what makes it
//! usable over the network — the same hash can be computed from two HTTP
//! `Range` requests (NEN-036), so a remote stream is identifiable without
//! downloading it.
//!
//! # Why this takes bytes and not a path
//!
//! ADR-0009 Karar 2 keeps this crate free of I/O. The caller reads the two
//! windows — from a file, from `Range` responses, from a fixture — and hands
//! them in. Nothing here knows where they came from.
//!
//! # Why the hash is redacted
//!
//! `docs/security-policy.md` K23 #8 forbids logging "özel hash / filename
//! metadata": a media fingerprint is exactly the thing that reveals what the
//! user is watching. [`OsHash`] therefore prints `<redacted>` from both
//! `Debug` and `Display`, following the [`Redacted`](nen_domain::redact::Redacted)
//! precedent from NEN-006, and exposes the real value only through the
//! explicit [`OsHash::to_hex`] / [`OsHash::as_bytes`] accessors used when
//! talking to a provider.

use std::fmt;

use nen_domain::redact::size_class;

/// Bytes read from each end of the file. Fixed by the OSDb algorithm.
pub const CHUNK_BYTES: usize = 64 * 1024;

/// Smallest file the algorithm is defined for. Below two full chunks the head
/// and tail windows would overlap and the hash would depend on how the caller
/// resolved that overlap rather than on the file.
pub const MIN_FILE_BYTES: u64 = 2 * CHUNK_BYTES as u64;

/// An OpenSubtitles-compatible file hash.
///
/// Compared and hashed by value; printed as `<redacted>` (see module docs).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct OsHash([u8; 8]);

impl OsHash {
    /// The raw digest, for sending to a provider that expects it.
    ///
    /// Callers must not pass the result to a log or error message (K23 #8).
    pub const fn as_bytes(&self) -> &[u8; 8] {
        &self.0
    }

    /// The digest in the lowercase hex form OpenSubtitles' API expects.
    ///
    /// Same caveat as [`OsHash::as_bytes`]: this is for the wire, not the log.
    pub fn to_hex(self) -> String {
        let mut hex = String::with_capacity(16);
        for byte in self.0 {
            hex.push_str(&format!("{byte:02x}"));
        }
        hex
    }
}

impl fmt::Debug for OsHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OsHash(<redacted>)")
    }
}

impl fmt::Display for OsHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

/// Why a hash could not be computed.
///
/// Carries no identifying payload: the size is reported as a
/// [`size_class`](nen_domain::redact::size_class), which
/// `docs/security-policy.md` lists under "Loglanabilecekler", never as an
/// exact byte count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsHashError {
    /// The file is smaller than [`MIN_FILE_BYTES`].
    TooSmall { size_class: &'static str },
    /// A supplied window was not exactly [`CHUNK_BYTES`] long.
    WindowLength { expected: usize, actual: usize },
}

impl fmt::Display for OsHashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooSmall { size_class } => {
                write!(f, "file is too small to hash (size class: {size_class})")
            }
            Self::WindowLength { expected, actual } => {
                write!(f, "expected a {expected}-byte window, got {actual} bytes")
            }
        }
    }
}

impl std::error::Error for OsHashError {}

/// Computes the hash from the file's size and its two 64 KiB end windows.
///
/// `head` must be the first [`CHUNK_BYTES`] bytes of the file and `tail` the
/// last [`CHUNK_BYTES`] bytes; both are required to be exactly that long, so a
/// short read is an error rather than a silently different hash.
pub fn of(file_size: u64, head: &[u8], tail: &[u8]) -> Result<OsHash, OsHashError> {
    if file_size < MIN_FILE_BYTES {
        return Err(OsHashError::TooSmall {
            size_class: size_class(file_size),
        });
    }
    check_window(head)?;
    check_window(tail)?;

    let sum = file_size
        .wrapping_add(sum_window(head))
        .wrapping_add(sum_window(tail));
    // Big-endian: OpenSubtitles' reference implementations render the hash
    // with `%016x` on the integer, so the most significant byte comes first.
    // The words are summed little-endian; only the *rendering* is big-endian,
    // and keeping the stored bytes in wire order means `as_bytes` and
    // `to_hex` cannot disagree.
    Ok(OsHash(sum.to_be_bytes()))
}

/// Convenience wrapper for callers that already hold the whole file, such as
/// tests and small local media. Slices the two windows itself.
pub fn of_bytes(contents: &[u8]) -> Result<OsHash, OsHashError> {
    let file_size = contents.len() as u64;
    if file_size < MIN_FILE_BYTES {
        return Err(OsHashError::TooSmall {
            size_class: size_class(file_size),
        });
    }
    let head = contents.get(..CHUNK_BYTES).unwrap_or_default();
    let tail = contents
        .len()
        .checked_sub(CHUNK_BYTES)
        .and_then(|start| contents.get(start..))
        .unwrap_or_default();
    of(file_size, head, tail)
}

fn check_window(window: &[u8]) -> Result<(), OsHashError> {
    if window.len() == CHUNK_BYTES {
        Ok(())
    } else {
        Err(OsHashError::WindowLength {
            expected: CHUNK_BYTES,
            actual: window.len(),
        })
    }
}

/// Wrapping sum of a window's little-endian `u64` words.
///
/// `as_chunks` yields fixed-size arrays and a remainder; the remainder is
/// always empty here because [`check_window`] has already established the
/// length, and it is ignored rather than unwrapped either way.
fn sum_window(bytes: &[u8]) -> u64 {
    bytes.as_chunks::<8>().0.iter().fold(0u64, |acc, word| {
        acc.wrapping_add(u64::from_le_bytes(*word))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zeros(len: usize) -> Vec<u8> {
        vec![0u8; len]
    }

    #[test]
    fn all_zero_file_hashes_to_its_own_size() {
        // Hand-verifiable known answer, no reference implementation involved:
        // every u64 word is zero, so the hash is just the file size, and
        // 131072 == 0x0000_0000_0002_0000.
        let hash = of_bytes(&zeros(MIN_FILE_BYTES as usize)).unwrap();
        assert_eq!(hash.to_hex(), "0000000000020000");
    }

    #[test]
    fn a_single_word_adds_to_the_size() {
        // Same file, but the first word is 1 -> 131072 + 1 == 0x…0002_0001.
        let mut contents = zeros(MIN_FILE_BYTES as usize);
        contents[0] = 1;
        let hash = of_bytes(&contents).unwrap();
        assert_eq!(hash.to_hex(), "0000000000020001");
    }

    #[test]
    fn head_and_tail_both_contribute() {
        let mut head_only = zeros(MIN_FILE_BYTES as usize);
        head_only[0] = 1;
        let mut tail_only = zeros(MIN_FILE_BYTES as usize);
        tail_only[MIN_FILE_BYTES as usize - 8] = 1;

        assert_eq!(
            of_bytes(&head_only).unwrap().to_hex(),
            of_bytes(&tail_only).unwrap().to_hex(),
            "a word at either end must contribute the same amount"
        );
        assert_ne!(
            of_bytes(&head_only).unwrap(),
            of_bytes(&zeros(MIN_FILE_BYTES as usize)).unwrap()
        );
    }

    #[test]
    fn bytes_between_the_windows_are_ignored() {
        // The whole point of the algorithm: the middle of the file is unread.
        let size = MIN_FILE_BYTES as usize + 4096;
        let plain = zeros(size);
        let mut middle_changed = zeros(size);
        middle_changed[CHUNK_BYTES + 100] = 0xFF;

        assert_eq!(
            of_bytes(&plain).unwrap(),
            of_bytes(&middle_changed).unwrap()
        );
    }

    #[test]
    fn one_byte_in_a_window_changes_the_hash() {
        let plain = zeros(MIN_FILE_BYTES as usize);
        let mut changed = zeros(MIN_FILE_BYTES as usize);
        changed[42] = 1;
        assert_ne!(of_bytes(&plain).unwrap(), of_bytes(&changed).unwrap());
    }

    #[test]
    fn a_file_one_byte_under_the_minimum_is_rejected() {
        let err = of_bytes(&zeros(MIN_FILE_BYTES as usize - 1)).unwrap_err();
        assert!(matches!(err, OsHashError::TooSmall { .. }));
    }

    #[test]
    fn exactly_the_minimum_is_accepted() {
        assert!(of_bytes(&zeros(MIN_FILE_BYTES as usize)).is_ok());
    }

    #[test]
    fn short_windows_are_an_error_not_a_different_hash() {
        let short = zeros(CHUNK_BYTES - 1);
        let full = zeros(CHUNK_BYTES);

        assert!(matches!(
            of(MIN_FILE_BYTES, &short, &full),
            Err(OsHashError::WindowLength { .. })
        ));
        assert!(matches!(
            of(MIN_FILE_BYTES, &full, &short),
            Err(OsHashError::WindowLength { .. })
        ));
    }

    #[test]
    fn the_error_never_carries_an_exact_size() {
        let err = of_bytes(&zeros(1234)).unwrap_err();
        let printed = format!("{err} {err:?}");
        assert!(
            !printed.contains("1234"),
            "the exact byte count leaked: {printed}"
        );
        assert!(
            printed.contains("tiny"),
            "expected a size class in {printed}"
        );
    }

    #[test]
    fn the_hash_never_prints_itself() {
        let hash = of_bytes(&zeros(MIN_FILE_BYTES as usize)).unwrap();
        let hex = hash.to_hex();

        assert_eq!(format!("{hash}"), "<redacted>");
        assert_eq!(format!("{hash:?}"), "OsHash(<redacted>)");
        assert!(!format!("{hash} {hash:?}").contains(&hex));
    }

    #[test]
    fn hex_is_sixteen_lowercase_digits() {
        let mut contents = zeros(MIN_FILE_BYTES as usize);
        contents[8] = 0xAB;
        let hex = of_bytes(&contents).unwrap().to_hex();

        assert_eq!(hex.len(), 16);
        assert!(hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn wrapping_addition_does_not_panic_on_overflow() {
        // Every word at u64::MAX plus a large size overflows many times over;
        // release and debug builds must agree and neither may panic.
        let contents = vec![0xFFu8; MIN_FILE_BYTES as usize];
        let hash = of_bytes(&contents).unwrap();
        let expected = MIN_FILE_BYTES
            .wrapping_add(u64::MAX.wrapping_mul((CHUNK_BYTES / 8) as u64))
            .wrapping_add(u64::MAX.wrapping_mul((CHUNK_BYTES / 8) as u64));
        assert_eq!(hash.as_bytes(), &expected.to_be_bytes());
    }
}
