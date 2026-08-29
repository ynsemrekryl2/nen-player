//! The `SubtitleRenderer` port: what everything that draws subtitles must
//! provide, what only some can do, and the kit that proves an adapter really
//! does it.
//!
//! Shape and rules come from ADR-0013, on top of `docs/product-spec.md` §14:
//!
//! | Karar | Where it lives |
//! |---|---|
//! | 1 — M3 draws through the engine | the adapter, in `nen-app` |
//! | 2 — drawing is its own port, with its own capabilities | [`surface`], [`capability`] |
//! | 2 — the contract scenarios are data | [`contract`] |
//!
//! The application layer branches on [`Capability`], **never** on a renderer's
//! name (product-spec §14: "UI renderer implementasyonunu bilmemelidir").
//!
//! # Why the first adapter is not here
//!
//! ADR-0013 Karar 1 draws through the playback engine, and this crate holds
//! ports rather than implementations — the engine-native adapter lives in
//! `nen-app`, next to the session that owns the engine. What that buys is a
//! port with no FFI surface of its own: a platform implements
//! `PlaybackEngine::inject_subtitle` and gets a renderer for free.

pub mod capability;
pub mod contract;
pub mod error;
pub mod fake;
pub mod surface;

pub use capability::{Capabilities, Capability};
pub use error::{Operation, RenderError};
pub use fake::FakeRenderer;
pub use surface::{SubtitleRenderer, MAX_BOTTOM_INSET};
