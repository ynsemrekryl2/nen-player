//! Subtitle source catalog: grouping, dedup and menu projection.
//!
//! Product-spec §7 requires a **single** catalog holding all four source kinds,
//! and §8 gives the menu that catalog must project into. ADR-0010 turns those
//! into rules this crate implements:
//!
//! - dedup is by [`SubtitleSourceId`], i.e. by **metadata** — §7's "cataloguing
//!   is not downloading" means no content is available to fingerprint (Karar 2);
//! - the projection is derived, never stored, and structural: it returns groups
//!   and kinds, not display text, because a language's visible name is its own
//!   name in every UI language and that mapping belongs to the platform
//!   (Karar 7);
//! - group order puts the user's preferred languages above the rest, which are
//!   ordered by tag (Karar 4, 5);
//! - auto-selection touches only preferred languages and only sources that are
//!   already playable (Karar 9).
//!
//! Crate sınırları ve izinli bağımlılıklar: ADR-0006.

pub mod auto_select;
pub mod catalog;
pub mod menu;

pub use auto_select::{auto_selection, AUTO_SELECTABLE_KINDS};
pub use catalog::SubtitleSourceCatalog;
pub use menu::{project, MenuGroup, MenuSection, SubtitleMenu};
