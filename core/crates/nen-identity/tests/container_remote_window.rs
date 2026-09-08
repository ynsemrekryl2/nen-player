//! Container metadata extraction from a media file's own first byte window
//! (NEN-072), against real-shaped fixtures and hand-crafted adversarial input.
//!
//! The unit tests in `container` cover the wiring and quick edge cases; this
//! covers real Matroska/MP4 byte layouts (golden) and byte sequences an
//! attacker controls (oversized declared sizes, deep nesting, an unrelated
//! file wearing the right extension) that must degrade to "nothing found"
//! rather than to a panic.

mod support;

use std::fs;

use nen_identity::container::parse_head_window;

fn window(name: &str) -> Vec<u8> {
    let path = support::fixture(&format!("container/{name}"));
    fs::read(&path).unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()))
}

#[test]
fn matroska_title_and_year_are_extracted_from_the_head_window() {
    let metadata = parse_head_window(&window("valid-title.mkv"));
    assert_eq!(metadata.title.as_deref(), Some("Nen Container Fixture"));
    assert_eq!(metadata.year, Some(2024));
}

#[test]
fn mp4_title_and_year_are_extracted_from_the_head_window() {
    let metadata = parse_head_window(&window("valid-title.mp4"));
    assert_eq!(metadata.title.as_deref(), Some("Nen Container Fixture"));
    assert_eq!(metadata.year, Some(2024));
}

/// A recognizable extension with unrecognizable content (NEN-022's
/// `broken-clip.mkv`, deterministic random bytes) must not be mistaken for a
/// valid container.
#[test]
fn a_broken_file_yields_empty_metadata_not_a_panic() {
    let bytes = fs::read(support::fixture("broken-clip.mkv")).expect("fixture exists");
    let metadata = parse_head_window(&bytes);
    assert!(metadata.is_empty());
}

#[test]
fn a_truncated_head_window_never_panics_at_any_cut_point() {
    let full = window("valid-title.mkv");
    for cut in [1, 4, 8, 16, 40, 100, 300, full.len() - 1] {
        let _ = parse_head_window(&full[..cut]);
    }
}

/// A `Segment` that declares a size far larger than the buffer it actually
/// lives in — the walker must clamp to what is actually there, not attempt to
/// read (or allocate) past the window's end.
#[test]
fn an_oversized_declared_element_size_never_panics() {
    let mut bytes = vec![0x1A, 0x45, 0xDF, 0xA3, 0x84, 0, 0, 0, 0]; // EBML header (ignored)
    bytes.extend([0x18, 0x53, 0x80, 0x67]); // Segment ID
    bytes.extend([0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE]); // 8-byte size: huge, not "unknown"
    bytes.extend([0x15, 0x49, 0xA9, 0x66, 0x84, b'x', b'x', b'x', b'x']); // trailing garbage

    let metadata = parse_head_window(&bytes);
    assert!(metadata.is_empty());
}

/// A `Segment` whose only content is thousands of levels of `Info`-shaped
/// elements wrapping one another instead of a `Title`. The walker only ever
/// descends one level into a recognized child (`Segment` → `Info` →
/// look-for-`Title`, nothing deeper), so this proves recursion depth cannot
/// be driven by how deeply the input claims to be nested.
#[test]
fn deeply_nested_elements_never_panic_or_hang() {
    let mut inner: Vec<u8> = Vec::new();
    for _ in 0..2000 {
        let mut next = vec![0x15, 0x49, 0xA9, 0x66]; // Info ID
        next.extend(encode_size_4(inner.len() as u32));
        next.extend(inner);
        inner = next;
    }
    let mut bytes = vec![0x1A, 0x45, 0xDF, 0xA3, 0x84, 0, 0, 0, 0];
    bytes.extend([0x18, 0x53, 0x80, 0x67]);
    bytes.extend(encode_size_4(inner.len() as u32));
    bytes.extend(inner);

    let metadata = parse_head_window(&bytes);
    assert!(metadata.is_empty());
}

/// Encodes `value` as a 4-byte EBML size VINT (marker `0001xxxx`, 28 data
/// bits) — the same shape `read_size` in `container.rs` decodes.
fn encode_size_4(value: u32) -> [u8; 4] {
    assert!(value < (1 << 28), "value does not fit a 4-byte EBML VINT");
    (0x1000_0000 | value).to_be_bytes()
}
