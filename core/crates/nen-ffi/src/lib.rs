//! The single external gate of the Nen Player core (ADR-0006).
//!
//! No other crate exposes an FFI surface. Redaction (K23) and typed-error
//! mapping are therefore enforceable at exactly one boundary.
//!
//! The binding technology is a **candidate** until ADR-0003 is accepted; the
//! rule that this crate is the only gate is not.

uniffi::setup_scaffolding!();

pub mod handoff;
pub mod playback;
pub mod remote_evidence;
pub mod session;
pub mod subtitles;
pub mod translation;

/// Human-readable identifier of this core build.
///
/// Skeleton smoke function for NEN-007: it carries a value across
/// `nen-domain` → `nen-app` → FFI → platform.
#[uniffi::export]
pub fn version() -> String {
    nen_app::version()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exported_version_matches_app_layer() {
        assert_eq!(version(), nen_app::version());
    }
}
