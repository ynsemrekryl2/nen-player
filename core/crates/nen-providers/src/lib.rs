//! Provider adapters, including the deterministic mock used by tests.
//!
//! Crate sınırları ve izinli bağımlılıklar: ADR-0006.

pub mod openai;
pub mod openrouter;
pub mod opensubtitles;
mod translation_http;
pub mod translation_mock;
