//! Persistence adapter: the filesystem artifact store ADR-0017 decided on.
//!
//! Crate sınırları ve izinli bağımlılıklar: ADR-0006 — this crate sees
//! `nen-domain` and `nen-ports` only, which is why the stored shape
//! ([`nen_ports::persistence::ArtifactRecord`]) is defined at the port and
//! not here.
//!
//! [`store::FilesystemArtifactStore`] is the whole adapter: one artifact is
//! one canonical JSON file under an injected root, named by the `blake3`
//! hash of its own bytes, written by temp-file-and-`rename` so no reader can
//! ever see a half-written one.
//!
//! Everything read here is untrusted input — bytes on a disk this process
//! does not control, which another program may have truncated, replaced or
//! symlinked (`docs/security-policy.md` §2). The crate therefore denies the
//! panicking escape hatches outright, following `nen-subtitle`. The
//! `not(test)` guard keeps `unwrap()` available inside `#[cfg(test)]`
//! modules, where a panic is the reporting mechanism rather than a failure
//! mode.
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::indexing_slicing
    )
)]

pub mod store;
mod wire;

pub use store::FilesystemArtifactStore;
