//! The size of the picture the engine is actually showing.
//!
//! ADR-0038 Karar 1: the port carries **one** number for the stream being
//! played — its aspect-correct display size — and nothing else. Codec, fps,
//! dynamic range and rotation class do not travel; Karar 3 keeps the scope at
//! the size alone.
//!
//! What travels is the size with pixel aspect ratio and rotation **already
//! applied**, which is the adapter's obligation rather than the core's
//! interpretation. Measured on libmpv (`evidence/M3/NEN-068-measurement.md`):
//! a 720x576 PAL stream with SAR 64:45 reports `1024x576`, so the engine hands
//! over the display size and nobody has to derive it.
//!
//! **Security (K23).** ADR-0038 Karar 5 places this outside every forbidden
//! class: a width and a height are not a media URL, a path, a filename or
//! dialogue. This type therefore derives `Debug` and may be logged — and
//! `tests/guard_playback_debug.rs` asserts that it still prints its numbers, so
//! nobody redacts it "to be safe" and leaves a diagnostic blind.

/// The display size of the video being played, in pixels.
///
/// Both dimensions are greater than zero; [`VideoGeometry::new`] is the only
/// way to build one, so a degenerate size cannot exist. That is what lets the
/// presentation layer divide by the height without asking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VideoGeometry {
    width: u32,
    height: u32,
}

impl VideoGeometry {
    /// A size, or `None` when either dimension is zero.
    ///
    /// Refusing rather than clamping: a zero came from an engine that did not
    /// know yet, and inventing a `1` for it would put a 1-pixel-wide window on
    /// screen instead of leaving the aspect lock off.
    pub const fn new(width: u32, height: u32) -> Option<Self> {
        if width == 0 || height == 0 {
            None
        } else {
            Some(Self { width, height })
        }
    }

    pub const fn width(self) -> u32 {
        self.width
    }

    pub const fn height(self) -> u32 {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_size_keeps_both_dimensions() {
        let geometry = VideoGeometry::new(1_024, 576).expect("a valid size");
        assert_eq!((geometry.width(), geometry.height()), (1_024, 576));
    }

    #[test]
    fn a_zero_dimension_is_refused_rather_than_repaired() {
        // Either direction: an engine that answered `0x576` knows as little as
        // one that answered `1024x0`.
        assert_eq!(VideoGeometry::new(0, 576), None);
        assert_eq!(VideoGeometry::new(1_024, 0), None);
        assert_eq!(VideoGeometry::new(0, 0), None);
    }

    #[test]
    fn debug_prints_the_numbers_it_carries() {
        // ADR-0038 Karar 5: the size is loggable. A guard that hid it would
        // remove the one fact this type exists to report.
        let printed = format!(
            "{:?}",
            VideoGeometry::new(1_024, 576).expect("a valid size")
        );
        assert!(printed.contains("1024"), "width missing: {printed}");
        assert!(printed.contains("576"), "height missing: {printed}");
    }
}
