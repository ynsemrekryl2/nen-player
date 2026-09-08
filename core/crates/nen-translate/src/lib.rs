//! Translation block pipeline, validation and checkpointing.
//!
//! Crate sınırları ve izinli bağımlılıklar: ADR-0006.
//!
//! `blocks` deterministically splits a [`SubtitleDocument`] into overlapping
//! translation blocks (ADR-0015); `context` extracts a small, deterministic
//! document-wide context; `validation` checks untrusted provider DTOs
//! locally (ADR-0016); `repair` bounds how many times an invalid block is
//! retried (`NEN-092`); `checkpoint` drives blocks in sequence, committing
//! each only under the provider port's cancellation gate so a cancelled or
//! interrupted run never produces a late or partial result (`NEN-093`); and
//! `artifact` assembles a fully checkpointed run into a
//! `ValidatedSubtitleArtifact` and its WebVTT output, re-checking every cue
//! against the source document rather than trusting the checkpointed blocks
//! on their own (`NEN-094`). None of these modules performs I/O.
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

pub mod artifact;
pub mod blocks;
pub mod checkpoint;
pub mod context;
pub mod repair;
pub mod validation;
