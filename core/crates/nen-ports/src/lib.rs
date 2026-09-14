//! Port traits and the shared contract test kits that every adapter must pass.
//!
//! Crate sınırları ve izinli bağımlılıklar: ADR-0006.

pub mod credentials;
pub mod filename_normalization;
pub mod http;
pub mod identity;
pub mod persistence;
pub mod playback;
pub mod renderer;
pub mod subtitle_candidates;
pub mod subtitle_download;
pub mod translation;
