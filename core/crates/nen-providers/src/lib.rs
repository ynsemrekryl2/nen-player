//! Provider adapters, including the deterministic mock used by tests.
//!
//! Crate sınırları ve izinli bağımlılıklar: ADR-0006.

pub mod filename_normalization;
pub mod openai;
pub mod openrouter;
pub mod opensubtitles;
mod translation_http;
pub mod translation_mock;
