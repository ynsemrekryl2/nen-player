//! The `PlaybackEngine` port: what every media engine must provide, what only
//! some do, and the kit that proves an adapter really does it.
//!
//! Shape and rules come from ADR-0011, on top of ADR-0026's ownership
//! direction (core-owned session, reverse callback):
//!
//! | Karar | Where it lives |
//! |---|---|
//! | 1 — bounded queue + per-class coalescing | [`event`] |
//! | 2 — no synchronous call from inside a callback | [`event::guard_reentrancy`] |
//! | 3 — capability names only what differs | [`capability`], [`engine`] |
//! | 4 — contract scenarios are data | [`contract`] |
//!
//! The application layer branches on [`Capability`], **never** on an engine's
//! name (product-spec §4).

pub mod capability;
pub mod contract;
pub mod engine;
pub mod error;
pub mod event;
pub mod fake;
pub mod media;
pub mod track;

pub use capability::{Capabilities, Capability};
pub use engine::PlaybackEngine;
pub use error::{LoadFailure, Operation, PlaybackError};
pub use event::{
    deliver_all, guard_reentrancy, CallbackScope, DeliveryClass, EventQueue, EventSink,
    PlaybackEvent, PlaybackState,
};
pub use media::MediaSource;
pub use track::{TrackDescriptor, TrackId, TrackKind};
