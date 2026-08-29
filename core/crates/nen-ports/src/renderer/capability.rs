//! What a subtitle renderer can do beyond drawing (ADR-0013 Karar 2).
//!
//! The same rule as [`playback::capability`](crate::playback::capability), for
//! the same reason (ADR-0011 Karar 3): a capability names something that
//! genuinely **differs between renderers**. Showing a document and taking it
//! off screen are not here at all — a thing that cannot do both is not a
//! renderer, so asking about them would only produce checks that are always
//! true.

use std::fmt;

/// An optional renderer capability.
///
/// Free of private data, so it may be logged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    /// The renderer can be asked what it is drawing **right now**.
    ///
    /// Optional because drawing and reporting are different abilities: a
    /// renderer may put the right line on screen and expose no way to read it
    /// back. Where it exists, it is what turns "the right cue is displayed"
    /// from a claim about what we asked for into one about what was drawn —
    /// which is exactly what NEN-027's acceptance measures.
    ///
    /// **Security:** the answer is subtitle dialogue (K23 #4). Displayable,
    /// never loggable.
    ObservedText,
}

impl Capability {
    /// Every capability, in declaration order. Used by the contract kit to
    /// prove that each unsupported capability produces a typed error.
    pub const ALL: [Capability; 1] = [Capability::ObservedText];

    /// Stable lowercase name. Safe to log.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ObservedText => "observed_text",
        }
    }

    const fn bit(self) -> u8 {
        match self {
            Self::ObservedText => 1 << 0,
        }
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The set of capabilities a renderer declares.
///
/// A bit set rather than a collection, on
/// [`playback::Capabilities`](crate::playback::Capabilities)' reasoning: the
/// set is small, an adapter declares it once, and copying it across the FFI
/// boundary must stay free.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Capabilities(u8);

impl Capabilities {
    /// A renderer that draws and nothing more.
    pub const NONE: Capabilities = Capabilities(0);

    /// Every optional capability.
    pub const ALL: Capabilities = Capabilities(0b1);

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
    /// Prints the declared names, not the bit pattern.
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
    fn each_capability_occupies_its_own_bit() {
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
    fn with_and_without_are_inverses() {
        let set = Capabilities::NONE.with(Capability::ObservedText);
        assert!(set.contains(Capability::ObservedText));
        assert!(!set
            .without(Capability::ObservedText)
            .contains(Capability::ObservedText));
    }

    #[test]
    fn debug_prints_names_not_bits() {
        let set = Capabilities::new([Capability::ObservedText]);
        assert_eq!(format!("{set:?}"), "{ObservedText}");
    }
}
