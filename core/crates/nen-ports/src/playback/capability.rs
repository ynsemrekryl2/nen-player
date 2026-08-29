//! What a playback engine can do beyond the mandatory base (ADR-0011 Karar 3).
//!
//! A [`Capability`] names something that genuinely **differs between engines**.
//! The base every adapter must provide — load, play, pause, stop, absolute
//! seek, position, duration, state, track enumeration and selection, the event
//! stream and shutdown — is not represented here at all: an implementation
//! without it is not a `PlaybackEngine`, so asking about it would only produce
//! checks that are always true.
//!
//! The application layer branches on these, **never** on the engine's name
//! (`docs/architecture.md` → "Capability modeli", product-spec §4). That rule
//! is what lets a second engine appear without touching a single call site.

use std::fmt;

/// An optional engine capability.
///
/// Free of private data, so it may be logged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    /// The engine can hand back the *text* of an embedded subtitle track.
    ///
    /// Optional because a bitmap track (PGS, VobSub) has no text to extract at
    /// all — NEN-023 marks those `translatable = false`. Extraction stays lazy
    /// (§7): having the capability does not mean anything is extracted up
    /// front. No adapter declares it yet; NEN-044 is where one first can.
    EmbeddedTextExtraction,
    /// The engine can render a subtitle document supplied from outside the
    /// media file.
    ///
    /// Optional because the rendering path is platform-owned (NEN-027); an
    /// engine may only be able to show what is already inside the container.
    ExternalSubtitleInjection,
    /// The engine can be asked what subtitle text it is drawing **right now**.
    ///
    /// Optional because drawing and reporting are different abilities: an
    /// engine may render a subtitle perfectly and expose no way to read the
    /// line it has on screen. Where it exists, it is what turns "the right cue
    /// is displayed" from an assertion about what we *asked for* into one
    /// about what the engine actually drew (ADR-0013 Karar 2).
    ///
    /// **Security:** the answer is subtitle dialogue (K23 #4). Displayable,
    /// never loggable.
    RenderedTextObservation,
    /// The engine can play at a rate other than 1.0.
    ///
    /// The most commonly restricted capability: an engine may support no rate
    /// change at all, or only a narrow range.
    PlaybackRate,
    /// The engine can set its own output volume.
    ///
    /// Optional because on some platforms volume is a system-level control and
    /// the engine has no say in it.
    Volume,
}

impl Capability {
    /// Every capability, in declaration order. Used by the contract kit to
    /// prove that *each* unsupported capability produces a typed error.
    pub const ALL: [Capability; 5] = [
        Capability::EmbeddedTextExtraction,
        Capability::ExternalSubtitleInjection,
        Capability::RenderedTextObservation,
        Capability::PlaybackRate,
        Capability::Volume,
    ];

    /// Stable lowercase name. Safe to log.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmbeddedTextExtraction => "embedded_text_extraction",
            Self::ExternalSubtitleInjection => "external_subtitle_injection",
            Self::RenderedTextObservation => "rendered_text_observation",
            Self::PlaybackRate => "playback_rate",
            Self::Volume => "volume",
        }
    }

    const fn bit(self) -> u8 {
        match self {
            Self::EmbeddedTextExtraction => 1 << 0,
            Self::ExternalSubtitleInjection => 1 << 1,
            Self::RenderedTextObservation => 1 << 2,
            Self::PlaybackRate => 1 << 3,
            Self::Volume => 1 << 4,
        }
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The set of capabilities an engine declares.
///
/// Deliberately a small value type rather than a collection: the set is fixed
/// at four entries, an adapter declares it once, and copying it across the FFI
/// boundary must stay free.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Capabilities(u8);

impl Capabilities {
    /// An engine that provides the mandatory base and nothing else.
    pub const NONE: Capabilities = Capabilities(0);

    /// Every optional capability.
    pub const ALL: Capabilities = Capabilities(0b1_1111);

    /// Builds a set from the given capabilities.
    pub fn new(capabilities: impl IntoIterator<Item = Capability>) -> Self {
        let mut bits = 0;
        for capability in capabilities {
            bits |= capability.bit();
        }
        Self(bits)
    }

    pub const fn contains(self, capability: Capability) -> bool {
        self.0 & capability.bit() != 0
    }

    pub fn with(self, capability: Capability) -> Self {
        Self(self.0 | capability.bit())
    }

    pub fn without(self, capability: Capability) -> Self {
        Self(self.0 & !capability.bit())
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The declared capabilities, in [`Capability::ALL`] order.
    pub fn iter(self) -> impl Iterator<Item = Capability> {
        Capability::ALL
            .into_iter()
            .filter(move |capability| self.contains(*capability))
    }
}

impl FromIterator<Capability> for Capabilities {
    fn from_iter<T: IntoIterator<Item = Capability>>(iter: T) -> Self {
        Self::new(iter)
    }
}

impl fmt::Debug for Capabilities {
    /// Prints the declared names, not the bit pattern — a raw `u8` in a log
    /// tells a reader nothing.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_set_contains_nothing() {
        for capability in Capability::ALL {
            assert!(!Capabilities::NONE.contains(capability));
        }
        assert!(Capabilities::NONE.is_empty());
    }

    #[test]
    fn all_set_contains_everything() {
        for capability in Capability::ALL {
            assert!(Capabilities::ALL.contains(capability));
        }
        assert_eq!(Capabilities::ALL.iter().count(), Capability::ALL.len());
    }

    #[test]
    fn with_and_without_are_inverses() {
        let set = Capabilities::NONE.with(Capability::Volume);
        assert!(set.contains(Capability::Volume));
        assert!(!set.without(Capability::Volume).contains(Capability::Volume));
    }

    #[test]
    fn each_capability_occupies_its_own_bit() {
        // A shared bit would make one capability silently imply another.
        for capability in Capability::ALL {
            let set = Capabilities::new([capability]);
            for other in Capability::ALL {
                assert_eq!(
                    set.contains(other),
                    other == capability,
                    "{capability} leaked into {other}"
                );
            }
        }
    }

    #[test]
    fn iteration_follows_declaration_order() {
        let set = Capabilities::new([Capability::Volume, Capability::EmbeddedTextExtraction]);
        assert_eq!(
            set.iter().collect::<Vec<_>>(),
            vec![Capability::EmbeddedTextExtraction, Capability::Volume]
        );
    }

    #[test]
    fn debug_prints_names_not_bits() {
        let set = Capabilities::new([Capability::PlaybackRate]);
        assert_eq!(format!("{set:?}"), "{PlaybackRate}");
    }
}
