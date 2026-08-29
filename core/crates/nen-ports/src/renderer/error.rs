//! What a render operation can refuse to do, as a typed value.
//!
//! The rules are [`playback::error`](crate::playback::error)'s, and so is the
//! reason: an operation the renderer cannot perform returns a **typed error**,
//! never a panic and never a silent no-op. A silent no-op is the worse failure
//! here in particular — the caller believes the user is looking at a subtitle.
//!
//! **Security (K23).** No variant carries a path, a filename, a locator or a
//! line of dialogue, and none may ever be given one. The document that failed
//! to draw is not named; only what went wrong is. NEN-010 measured why this
//! has to hold at construction time rather than in a hand-written `Debug`: an
//! error crossing the FFI boundary is re-printed by the host language, which
//! never sees a Rust `Debug` impl at all.

use super::capability::Capability;
use std::fmt;

/// Which render operation produced an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operation {
    Show,
    Clear,
    RenderedText,
    SetBottomInset,
}

impl Operation {
    /// Stable lowercase name. Safe to log.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Show => "show",
            Self::Clear => "clear",
            Self::RenderedText => "rendered_text",
            Self::SetBottomInset => "set_bottom_inset",
        }
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A refused or failed render operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// The renderer does not declare the capability this operation needs.
    Unsupported {
        operation: Operation,
        capability: Capability,
    },
    /// This renderer cannot draw on this surface at all.
    ///
    /// Distinct from [`Self::Unsupported`], and the distinction matters: an
    /// absent capability means "I draw, but I cannot answer that"; this means
    /// "I cannot draw here". The engine-native renderer answers this when the
    /// engine underneath it does not accept an external document — a fact
    /// about the pairing, not about either half.
    Unavailable { operation: Operation },
    /// There is no medium to draw on.
    NoMedia { operation: Operation },
    /// The surface has been shut down; it accepts nothing further.
    ShutDown { operation: Operation },
    /// Called synchronously from inside an event callback, on the same thread
    /// (ADR-0011 Karar 2).
    ReentrantCall { operation: Operation },
    /// The bottom inset is outside `0.0..=`[`MAX_BOTTOM_INSET`]
    /// (ADR-0037 Karar 2).
    ///
    /// [`MAX_BOTTOM_INSET`]: crate::renderer::MAX_BOTTOM_INSET
    ///
    /// Carries no number. The caller sent the value and the ceiling is a
    /// constant of this crate, so repeating either would only give the error a
    /// float to print. What the variant exists for is the refusal itself: a
    /// shell that asks for an impossible inset must not get a quietly clamped
    /// one back, because a clamped inset looks exactly like a working one.
    InsetOutOfRange { operation: Operation },
    /// The surface failed for a reason of its own.
    ///
    /// The code belongs to whatever draws; it is opaque to the core and
    /// meaningless to the user. The UI must never show it and must never
    /// branch on it — that would be branching on the renderer's identity by
    /// another name (product-spec §14).
    SurfaceFailure { operation: Operation, code: i32 },
}

impl RenderError {
    /// The operation that was refused. Every variant names one.
    pub const fn operation(&self) -> Operation {
        match self {
            Self::Unsupported { operation, .. }
            | Self::Unavailable { operation }
            | Self::NoMedia { operation }
            | Self::ShutDown { operation }
            | Self::ReentrantCall { operation }
            | Self::InsetOutOfRange { operation }
            | Self::SurfaceFailure { operation, .. } => *operation,
        }
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported {
                operation,
                capability,
            } => write!(f, "{operation} needs the {capability} capability"),
            Self::Unavailable { operation } => {
                write!(f, "{operation} on a surface that cannot draw subtitles")
            }
            Self::NoMedia { operation } => write!(f, "{operation} with no medium to draw on"),
            Self::ShutDown { operation } => write!(f, "{operation} after shutdown"),
            Self::ReentrantCall { operation } => {
                write!(f, "{operation} called from inside an event callback")
            }
            Self::InsetOutOfRange { operation } => {
                write!(f, "{operation} outside the accepted inset range")
            }
            Self::SurfaceFailure { operation, .. } => write!(f, "{operation} failed"),
        }
    }
}

impl std::error::Error for RenderError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_names_its_operation() {
        assert_eq!(
            RenderError::NoMedia {
                operation: Operation::Show
            }
            .operation(),
            Operation::Show
        );
        assert_eq!(
            RenderError::SurfaceFailure {
                operation: Operation::Clear,
                code: -7
            }
            .operation(),
            Operation::Clear
        );
    }

    #[test]
    fn the_surface_code_never_reaches_the_display_text() {
        // Same rule as PlaybackError::EngineFailure: the code is for the
        // surface's own diagnostics, not for a user and not for a branch.
        let shown = RenderError::SurfaceFailure {
            operation: Operation::Show,
            code: -12345,
        }
        .to_string();
        assert!(!shown.contains("12345"), "surface code leaked: {shown}");
    }

    #[test]
    fn unavailable_and_unsupported_are_different_answers() {
        // "I cannot draw here" and "I draw, but cannot answer that" must not
        // collapse into one variant: a caller shows a different thing for each.
        let unavailable = RenderError::Unavailable {
            operation: Operation::Show,
        };
        let unsupported = RenderError::Unsupported {
            operation: Operation::RenderedText,
            capability: Capability::ObservedText,
        };
        assert_ne!(unavailable, unsupported);
    }
}
