//! SRT/WebVTT parsing and writing, encoding detection, timeline fingerprinting.
//!
//! Crate sınırları ve izinli bağımlılıklar: ADR-0006.
//!
//! Everything here parses untrusted input (`docs/security-policy.md` §2), so
//! the crate denies the panicking escape hatches outright. The `not(test)`
//! guard keeps `unwrap()` available inside `#[cfg(test)]` modules, where a
//! panic is the reporting mechanism rather than a failure mode; the
//! `tests/` directory compiles as separate crates and is unaffected.
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

pub mod encoding;
pub mod fingerprint;
pub mod index;
pub mod srt;
pub mod webvtt;
