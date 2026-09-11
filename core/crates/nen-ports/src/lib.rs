//! Port traits and the shared contract test kits that every adapter must pass.
//!
//! Crate sınırları ve izinli bağımlılıklar: ADR-0006.

pub mod credentials;
pub mod http;
pub mod identity;
pub mod persistence;
pub mod playback;
pub mod renderer;
pub mod translation;
