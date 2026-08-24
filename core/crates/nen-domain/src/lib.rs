//! Pure domain model for the Nen Player core.
//!
//! This crate has **no internal dependencies and performs no I/O** — no files,
//! no network, no clock, no randomness. Everything of that kind is injected
//! through `nen-ports`. See ADR-0006.

/// Stable identifier of the shared core, used to prove that a value really
/// travelled `nen-domain` → `nen-app` → `nen-ffi` → platform.
pub const CORE_NAME: &str = "nen-core";
