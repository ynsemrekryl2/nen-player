//! Application / use-case layer: the only crate the FFI gate talks to.
//!
//! Crate sınırları ve izinli bağımlılıklar: ADR-0006.

pub mod embedded;
pub mod handoff;
pub mod identity;
pub mod playback;
pub mod remote_evidence;
pub mod renderer;
pub mod session;
pub mod subtitle_files;
pub mod subtitles;
pub mod translation;

// The FFI gate is only allowed to depend on this crate (ADR-0006 kural 3), but
// the types it has to translate — states, capabilities, track kinds, menu
// headings — are the port's, the domain's and the catalog's. Re-exporting them
// here keeps the dependency arrow where the ADR draws it (`nen-ffi -> nen-app`)
// instead of adding an edge, and makes the gate name them as what they are:
// this layer's vocabulary.
pub use nen_catalog as catalog;
pub use nen_domain as domain;
pub use nen_ports as ports;

/// Human-readable identifier of this core build.
///
/// The value is composed here from a `nen-domain` constant and this crate's
/// own version, so a caller that receives it has proven the whole chain
/// `nen-domain` → `nen-app` is wired up.
pub fn version() -> String {
    format!("{} {}", nen_domain::CORE_NAME, env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_combines_domain_constant_and_crate_version() {
        assert_eq!(version(), format!("nen-core {}", env!("CARGO_PKG_VERSION")));
    }
}
