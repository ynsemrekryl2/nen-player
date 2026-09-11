//! What a playback operation can refuse to do, as a typed value.
//!
//! `docs/architecture.md` is explicit: an operation the engine cannot perform
//! returns a **typed error** — never a panic, never a silent no-op. A silent
//! no-op is the worse of the two failures, because the caller believes it
//! worked.
//!
//! **Security (K23).** No variant carries a media URL, a private path, a
//! filename, a token or any provider payload, and none may ever be given one.
//! NEN-010 measured why this has to hold at *construction* time rather than in
//! a hand-written `Debug`: an error crossing the FFI boundary is re-printed by
//! the host language, which never sees a Rust `Debug` impl at all. A value that
//! is safe only because of how Rust prints it is not safe. Every field here is
//! therefore a bounded enum or a number.
//! `tests/guard_playback_error_debug.rs` holds that line, with a deliberately
//! leaky twin proving the check is not vacuous.

use super::capability::Capability;
use super::track::TrackKind;
use std::fmt;

/// Which port operation produced an error.
///
/// Present so that a refusal says *what* was refused without the caller having
/// to correlate log lines. Free of private data — the name of an operation is
/// a constant of this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operation {
    Load,
    Play,
    Pause,
    Stop,
    Seek,
    Position,
    Duration,
    State,
    VideoGeometry,
    SetRate,
    SetVolume,
    Tracks,
    SelectTrack,
    ExtractText,
    InjectSubtitle,
    RenderedText,
    SetSubtitleBottomInset,
    Shutdown,
}

impl Operation {
    /// Stable lowercase name. Safe to log.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Play => "play",
            Self::Pause => "pause",
            Self::Stop => "stop",
            Self::Seek => "seek",
            Self::Position => "position",
            Self::Duration => "duration",
            Self::State => "state",
            Self::VideoGeometry => "video_geometry",
            Self::SetRate => "set_rate",
            Self::SetVolume => "set_volume",
            Self::Tracks => "tracks",
            Self::SelectTrack => "select_track",
            Self::ExtractText => "extract_text",
            Self::InjectSubtitle => "inject_subtitle",
            Self::RenderedText => "rendered_text",
            Self::SetSubtitleBottomInset => "set_subtitle_bottom_inset",
            Self::Shutdown => "shutdown",
        }
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why loading failed, without saying anything about *what* was being loaded.
///
/// The media URL and the private path are K23 #1 and #3; naming the category
/// of failure is enough for the UI to say something useful and for a log to be
/// diagnosable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LoadFailure {
    /// Nothing at the given location.
    NotFound,
    /// Found, but could not be opened or read.
    Unreadable,
    /// Opened, but the container or codec is not playable by this engine.
    UnsupportedFormat,
    /// A remote medium could not be reached.
    NetworkUnavailable,
}

impl LoadFailure {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::Unreadable => "unreadable",
            Self::UnsupportedFormat => "unsupported_format",
            Self::NetworkUnavailable => "network_unavailable",
        }
    }
}

impl fmt::Display for LoadFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A refused or failed playback operation.
///
/// Every variant is inspectable without parsing a string — that is what makes
/// an exhaustive `switch` possible on the platform side (NEN-010 proved the
/// Swift end of this: adding a variant breaks a non-exhaustive switch at
/// compile time).
// No `Eq`: `RateOutOfRange` carries `f32`, and pretending float equality is
// total would be a lie the compiler is right to refuse.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackError {
    /// The engine does not declare the capability this operation needs
    /// (ADR-0011 Karar 3).
    Unsupported {
        operation: Operation,
        capability: Capability,
    },
    /// Called synchronously from inside an event callback, on the same thread
    /// (ADR-0011 Karar 2).
    ///
    /// This is a contract violation by the caller, not an engine fault. It
    /// returns instead of deadlocking: NEN-029 showed the same-thread call
    /// deadlocks against the delivery gate's own lock.
    ReentrantCall { operation: Operation },
    /// The operation needs loaded media and nothing is loaded.
    NotLoaded { operation: Operation },
    /// The engine has been shut down; it accepts nothing further.
    ShutDown { operation: Operation },
    /// No track with the given id, or the id belongs to the other kind.
    UnknownTrack { kind: TrackKind },
    /// The track exists but does not carry text (ADR-0045 Karar 3) — a
    /// bitmap subtitle, or any track [`super::track::subtitle_carries_text`]
    /// does not recognise. Never returned for a missing id; that is
    /// [`Self::UnknownTrack`].
    TrackCarriesNoText,
    /// The engine supports rate changes but not this rate.
    ///
    /// Carries the bounds so the caller can clamp instead of guessing.
    RateOutOfRange { requested: f32, min: f32, max: f32 },
    /// Loading failed. See [`LoadFailure`] — deliberately says nothing about
    /// which medium (K23 #1, #3).
    LoadFailed { reason: LoadFailure },
    /// The bottom inset is outside `0.0..=MAX_SUBTITLE_BOTTOM_INSET`
    /// (ADR-0037 Karar 2).
    ///
    /// Carries no number: the caller sent the value and the ceiling is a
    /// constant of this crate, so repeating either would only give the error a
    /// float to print. The refusal exists so an out-of-range inset cannot pass
    /// as a silently clamped one.
    InsetOutOfRange { operation: Operation },
    /// The engine failed for a reason of its own.
    ///
    /// The code is the engine's, opaque to the core and meaningless to the
    /// user; the UI must never show it and must never branch on it — that
    /// would be branching on the engine's identity by another name
    /// (product-spec §4).
    EngineFailure { code: i32 },
}

impl PlaybackError {
    /// The operation that was refused, when the variant names one.
    pub const fn operation(&self) -> Option<Operation> {
        match self {
            Self::Unsupported { operation, .. }
            | Self::ReentrantCall { operation }
            | Self::NotLoaded { operation }
            | Self::ShutDown { operation }
            | Self::InsetOutOfRange { operation } => Some(*operation),
            Self::UnknownTrack { .. }
            | Self::TrackCarriesNoText
            | Self::RateOutOfRange { .. }
            | Self::LoadFailed { .. }
            | Self::EngineFailure { .. } => None,
        }
    }
}

impl fmt::Display for PlaybackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported {
                operation,
                capability,
            } => write!(f, "{operation} needs the {capability} capability"),
            Self::ReentrantCall { operation } => {
                write!(f, "{operation} called from inside an event callback")
            }
            Self::NotLoaded { operation } => write!(f, "{operation} needs loaded media"),
            Self::ShutDown { operation } => write!(f, "{operation} after shutdown"),
            Self::InsetOutOfRange { operation } => {
                write!(f, "{operation} outside the accepted inset range")
            }
            Self::UnknownTrack { kind } => write!(f, "no such {kind} track"),
            Self::TrackCarriesNoText => f.write_str("the track carries no text"),
            Self::RateOutOfRange {
                requested,
                min,
                max,
            } => write!(f, "rate {requested} outside [{min}, {max}]"),
            Self::LoadFailed { reason } => write!(f, "load failed: {reason}"),
            Self::EngineFailure { .. } => f.write_str("engine failure"),
        }
    }
}

impl std::error::Error for PlaybackError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_are_inspectable_without_parsing_strings() {
        let error = PlaybackError::Unsupported {
            operation: Operation::SetRate,
            capability: Capability::PlaybackRate,
        };
        // The point of a typed error: the discriminant answers the question.
        match error {
            PlaybackError::Unsupported { capability, .. } => {
                assert_eq!(capability, Capability::PlaybackRate);
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn operation_is_reported_where_it_applies() {
        assert_eq!(
            PlaybackError::NotLoaded {
                operation: Operation::Play
            }
            .operation(),
            Some(Operation::Play)
        );
        assert_eq!(
            PlaybackError::LoadFailed {
                reason: LoadFailure::NotFound
            }
            .operation(),
            None
        );
    }

    #[test]
    fn rate_error_carries_the_bounds_so_a_caller_can_clamp() {
        let error = PlaybackError::RateOutOfRange {
            requested: 4.0,
            min: 0.5,
            max: 2.0,
        };
        let PlaybackError::RateOutOfRange { min, max, .. } = error else {
            panic!("wrong variant");
        };
        assert_eq!((min, max), (0.5, 2.0));
    }

    #[test]
    fn engine_failure_code_never_reaches_the_display_text() {
        // The code is for the engine's own diagnostics; §4 forbids showing it
        // to the user or branching on it.
        let shown = PlaybackError::EngineFailure { code: -12345 }.to_string();
        assert!(!shown.contains("12345"), "engine code leaked: {shown}");
    }
}
