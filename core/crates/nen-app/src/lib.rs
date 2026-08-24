//! Application / use-case layer: the only crate the FFI gate talks to.
//!
//! Crate sınırları ve izinli bağımlılıklar: ADR-0006.

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
