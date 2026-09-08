//! Translation block pipeline, validation and checkpointing.
//!
//! Crate sınırları ve izinli bağımlılıklar: ADR-0006.
//!
//! `blocks` deterministically splits a [`SubtitleDocument`] into overlapping
//! translation blocks (ADR-0015); `context` extracts a small, deterministic
//! document-wide context from the same document. Neither module performs
//! I/O or calls a provider — that starts at `NEN-090`.
//!
//! [`SubtitleDocument`]: nen_domain::subtitle::SubtitleDocument
//!
//! Everything here parses/derives from a `SubtitleDocument` that already
//! carries user-visible dialogue, so the crate denies the panicking escape
//! hatches outright, matching `nen-subtitle`. The `not(test)` guard keeps
//! `unwrap()` available inside `#[cfg(test)]` modules, where a panic is the
//! reporting mechanism rather than a failure mode; the `tests/` directory
//! compiles as separate crates and is unaffected.
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

pub mod blocks;
pub mod context;
