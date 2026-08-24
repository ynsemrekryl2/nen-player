//! NEN-008 — how expensive is it to move a large cue list across the FFI
//! boundary, and does a windowed handle beat handing over the whole list?
//!
//! **This is not product code** (CLAUDE.md rule 7). It opens its own throwaway
//! FFI gate, which ADR-0028 permits for spikes; ADR-0006's "`nen-ffi` is the
//! single gate" rule stays binding for `core/crates/*`. Nothing here is a draft
//! of a product API — the outcome of this spike is a number in an ADR, not code.
//!
//! Two approaches are exposed so the Swift harness can measure them
//! side by side:
//!
//! * **A — full list:** [`all_cues`] hands every cue over in one call.
//! * **B — window/handle:** [`CueHandle`] keeps the list in Rust and answers
//!   [`CueHandle::cues`] / [`CueHandle::active_cue`] queries.
//!
//! No SRT parsing happens here: that is M2 (`NEN-013`). The fixture is
//! generated, deterministic and copyright-clean.

use std::fmt;
use std::sync::Arc;

uniffi::setup_scaffolding!();

/// Spacing of the synthetic timeline: one cue every 3 s, on screen for 2.5 s.
/// The 500 ms gap is deliberate — it gives [`CueHandle::active_cue`] a
/// "no cue is active" case to be tested against.
const CUE_PERIOD_MS: u64 = 3_000;
const CUE_ON_SCREEN_MS: u64 = 2_500;

/// Fixed byte cost of one cue's non-text fields: `u32` + `u64` + `u64`.
const CUE_FIXED_BYTES: u64 = 4 + 8 + 8;

/// Sentence fragments cycled by cue index so cue lengths vary the way a real
/// subtitle file's do, without any randomness. Copyright-clean filler.
const TEXT_TEMPLATES: [&str; 5] = [
    "Yes.",
    "I told you this would happen sooner or later.",
    "We should leave before the others notice anything.",
    "Wait — say that again, slowly.",
    "It was never about the money, and you knew that from the start.",
];

/// One cue of the synthetic document.
#[derive(Clone, PartialEq, Eq, uniffi::Record)]
pub struct SpikeCue {
    /// Zero-based position in the document.
    pub index: u32,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

// K23: subtitle dialogue is never logged, so no `derive(Debug)` on a type
// carrying cue text — not even here, where the text is synthetic. See
// docs/security-policy.md → "`Debug` / `Display` kuralı".
impl fmt::Debug for SpikeCue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpikeCue")
            .field("index", &self.index)
            .field("start_ms", &self.start_ms)
            .field("end_ms", &self.end_ms)
            .field("text", &"<redacted>")
            .field("text_len", &self.text.len())
            .finish()
    }
}

/// Builds cue `index`. Pure and deterministic: same index, same bytes, always.
fn cue_at(index: u32) -> SpikeCue {
    let start_ms = u64::from(index) * CUE_PERIOD_MS;
    let template = TEXT_TEMPLATES[index as usize % TEXT_TEMPLATES.len()];
    SpikeCue {
        index,
        start_ms,
        end_ms: start_ms + CUE_ON_SCREEN_MS,
        // The index is woven in so no two cues share a byte pattern — a
        // measurement over `count` identical strings would flatter any
        // implementation that dedups or interns behind our back.
        text: format!("{index}. {template}"),
    }
}

fn build(count: u32) -> Vec<SpikeCue> {
    (0..count).map(cue_at).collect()
}

/// **Approach A** — hand the whole document over in a single call.
#[uniffi::export]
pub fn all_cues(count: u32) -> Vec<SpikeCue> {
    build(count)
}

/// Size of the fixture, so the timings recorded next to it mean something.
///
/// Returns UTF-8 text bytes plus the fixed per-cue fields; it is the payload
/// as it exists in Rust, before whatever the binding layer wraps around it.
#[uniffi::export]
pub fn payload_bytes(count: u32) -> u64 {
    (0..count)
        .map(|i| cue_at(i).text.len() as u64 + CUE_FIXED_BYTES)
        .sum()
}

/// **Approach B** — the list stays in Rust; the caller pulls windows.
#[derive(uniffi::Object)]
pub struct CueHandle {
    cues: Vec<SpikeCue>,
}

#[uniffi::export]
impl CueHandle {
    /// Materialises the document once, on this side of the boundary.
    #[uniffi::constructor]
    pub fn new(count: u32) -> Arc<Self> {
        Arc::new(Self { cues: build(count) })
    }

    pub fn count(&self) -> u32 {
        self.cues.len() as u32
    }

    /// The `[start, start + len)` window, clamped to the document. Out-of-range
    /// windows return fewer cues (or none) rather than failing: a scrolling UI
    /// asks for the window it can see, not one it has validated first.
    pub fn cues(&self, start: u32, len: u32) -> Vec<SpikeCue> {
        let from = (start as usize).min(self.cues.len());
        let to = from.saturating_add(len as usize).min(self.cues.len());
        self.cues[from..to].to_vec()
    }

    /// The cue on screen at `at_ms`, if any.
    ///
    /// Binary search, not a linear scan — `docs/DECISIONS.md` forbids scanning
    /// the full list for a lookup, and a linear version would measure the scan
    /// rather than the FFI call.
    pub fn active_cue(&self, at_ms: u64) -> Option<SpikeCue> {
        let candidate = match self.cues.binary_search_by_key(&at_ms, |c| c.start_ms) {
            Ok(i) => i,
            Err(0) => return None, // before the first cue starts
            Err(i) => i - 1,
        };
        let cue = &self.cues[candidate];
        (at_ms < cue.end_ms).then(|| cue.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const N: u32 = 1_000;

    #[test]
    fn generated_cues_are_deterministic() {
        assert_eq!(all_cues(N), all_cues(N));
    }

    #[test]
    fn no_two_cues_share_their_text() {
        let cues = all_cues(N);
        let mut texts: Vec<&str> = cues.iter().map(|c| c.text.as_str()).collect();
        texts.sort_unstable();
        texts.dedup();

        assert_eq!(texts.len(), N as usize);
    }

    #[test]
    fn payload_bytes_matches_the_generated_fixture() {
        let expected: u64 = all_cues(N)
            .iter()
            .map(|c| c.text.len() as u64 + CUE_FIXED_BYTES)
            .sum();

        assert_eq!(payload_bytes(N), expected);
    }

    /// The whole point of approach B is that it answers with the *same* cues as
    /// approach A. If it did not, the two measurements would not be comparable.
    #[test]
    fn window_matches_the_same_slice_of_the_full_list() {
        let full = all_cues(N);
        let handle = CueHandle::new(N);

        assert_eq!(handle.count(), N);
        assert_eq!(handle.cues(0, 40), full[0..40]);
        assert_eq!(handle.cues(500, 40), full[500..540]);
        assert_eq!(handle.cues(N - 10, 40), full[(N as usize - 10)..]);
    }

    #[test]
    fn window_past_the_end_is_empty_not_an_error() {
        let handle = CueHandle::new(N);

        assert!(handle.cues(N, 40).is_empty());
        assert!(handle.cues(N + 5_000, 40).is_empty());
        assert!(handle.cues(0, 0).is_empty());
    }

    #[test]
    fn active_cue_covers_the_whole_time_a_cue_is_on_screen() {
        let handle = CueHandle::new(N);
        let second = cue_at(1);

        for at in [second.start_ms, second.start_ms + 1, second.end_ms - 1] {
            assert_eq!(handle.active_cue(at).map(|c| c.index), Some(1), "at {at}");
        }
    }

    #[test]
    fn active_cue_is_none_in_the_gaps_and_outside_the_document() {
        let handle = CueHandle::new(N);
        let first = cue_at(0);
        let last = cue_at(N - 1);

        // The 500 ms gap between cue 0 and cue 1.
        assert_eq!(handle.active_cue(first.end_ms), None);
        assert_eq!(handle.active_cue(first.end_ms + 100), None);
        // After the last cue leaves the screen.
        assert_eq!(handle.active_cue(last.end_ms), None);
        assert_eq!(handle.active_cue(last.end_ms + 60_000), None);
    }

    #[test]
    fn active_cue_on_an_empty_document_is_none() {
        assert_eq!(CueHandle::new(0).active_cue(0), None);
    }
}
