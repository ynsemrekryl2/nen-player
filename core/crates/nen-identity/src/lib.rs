//! Media evidence collection, hashing and release-name parsing (ADR-0009).
//!
//! Crate sınırları ve izinli bağımlılıklar: ADR-0006.
//!
//! # What this crate is for
//!
//! `docs/product-spec.md` §6 forbids asking the user for a technical ID
//! (IMDb, Stremio, anything of that kind). Identity has to be *derived* from
//! whatever the media brings with it. That matters beyond tidiness: without an
//! identity there is no OpenSubtitles query (§7), so a file whose identity we
//! cannot work out is a file whose subtitles the user has to find by hand.
//!
//! ADR-0009 therefore models identity as **layers of evidence** rather than a
//! single filename guess. The layers split in two:
//!
//! * **Declarations** — someone states what this file is: Stremio handoff
//!   metadata, a `.nfo` sidecar, the container's own tags, or the name the
//!   filesystem/server gives it.
//! * **Inferences** — we work it out ourselves: parent directory names,
//!   agreement with sibling files, remote URL path segments.
//!
//! Declarations outrank inferences, and the first layer that yields something
//! other than [`Unknown`](release_name::MediaKind::Unknown) wins. Every layer
//! is optional; a missing one costs precision, never correctness. Showing the
//! user a list of candidates is the last resort (ADR-0009 Karar 7), which is
//! why every cheap offline layer lives here rather than being deferred.
//!
//! # No I/O
//!
//! Nothing here touches the filesystem, the network or the clock (ADR-0009
//! Karar 2). Bytes, directory listings and header values are handed *in*;
//! reading them is an adapter's job (`nen-ports`, NEN-036). That keeps the
//! whole crate testable without a fixture filesystem, and it is why
//! [`os_hash::of`] takes two byte windows instead of a path.
//!
//! # Untrusted input
//!
//! Filenames, `.nfo` files, container tags and server headers are all listed
//! as hostile input by `docs/security-policy.md` §2, so the crate denies the
//! panicking escape hatches outright. The `not(test)` guard keeps `unwrap()`
//! available inside `#[cfg(test)]` modules, where a panic is the reporting
//! mechanism rather than a failure mode; the `tests/` directory compiles as
//! separate crates and is unaffected.
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

pub mod confidence;
pub mod container;
pub mod declared_name;
pub mod dir_hints;
pub mod evidence;
pub mod nfo;
pub mod os_hash;
pub mod release_name;
pub mod siblings;
pub mod url_hints;
