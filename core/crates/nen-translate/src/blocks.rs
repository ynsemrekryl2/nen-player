//! Deterministic translation block layout (ADR-0015).
//!
//! A [`SubtitleDocument`] is split into overlapping [`TranslationBlock`]s of
//! fixed cue-count size. Overlap exists only so each block's provider prompt
//! carries a little neighbouring dialogue as context — every cue still
//! belongs to **exactly one** block's output set (ADR-0015 Karar 2), so
//! nothing is ever translated twice.
//!
//! [`SubtitleDocument`]: nen_domain::subtitle::SubtitleDocument

use std::fmt;
use std::ops::Range;

use nen_domain::subtitle::{CueId, SubtitleDocument};

/// Bumped by hand whenever the block-boundary or overlap-split rule in
/// ADR-0015 Karar 1/2 changes. Cache identity (`NEN-097`/ADR-0018) reads this
/// as one of its components; a bump makes every cache entry keyed on the old
/// rule unreachable.
pub const BLOCK_LAYOUT_VERSION: u32 = 1;

/// `docs/product-spec.md` §10.
pub const DEFAULT_BLOCK_SIZE: usize = 40;
pub const MIN_BLOCK_SIZE: usize = 30;
pub const MAX_BLOCK_SIZE: usize = 60;
pub const DEFAULT_OVERLAP: usize = 6;

/// Validated block-size/overlap pair. The only way to get one is
/// [`BlockLayoutConfig::new`] or [`BlockLayoutConfig::default`], so a
/// [`BlockLayoutConfig`] in hand always satisfies both gates below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockLayoutConfig {
    block_size: usize,
    overlap: usize,
}

impl BlockLayoutConfig {
    /// `block_size` must fall in `MIN_BLOCK_SIZE..=MAX_BLOCK_SIZE`;
    /// `overlap` must satisfy `1 <= overlap < block_size / 2` (ADR-0015
    /// Karar 1) — the upper half of that gate is what keeps a block's output
    /// set from ever coming out empty (see `compute` below).
    pub fn new(block_size: usize, overlap: usize) -> Result<Self, BlockLayoutError> {
        if !(MIN_BLOCK_SIZE..=MAX_BLOCK_SIZE).contains(&block_size) {
            return Err(BlockLayoutError::BlockSizeOutOfRange { got: block_size });
        }
        if overlap == 0 || overlap >= block_size / 2 {
            return Err(BlockLayoutError::OverlapOutOfRange {
                got: overlap,
                block_size,
            });
        }
        Ok(Self {
            block_size,
            overlap,
        })
    }

    pub const fn block_size(self) -> usize {
        self.block_size
    }

    pub const fn overlap(self) -> usize {
        self.overlap
    }
}

/// `docs/product-spec.md` §10's defaults; known to satisfy
/// [`BlockLayoutConfig::new`]'s own gates (asserted by
/// `default_config_satisfies_its_own_gate` in this module's tests), so this
/// does not go through `new` and cannot fail.
impl Default for BlockLayoutConfig {
    fn default() -> Self {
        Self {
            block_size: DEFAULT_BLOCK_SIZE,
            overlap: DEFAULT_OVERLAP,
        }
    }
}

/// Why a [`BlockLayoutConfig`] or [`BlockLayout`] could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockLayoutError {
    /// `docs/product-spec.md` §10: block size must fall in `30..=60`.
    BlockSizeOutOfRange { got: usize },
    /// ADR-0015 Karar 1: `1 <= overlap < block_size / 2`.
    OverlapOutOfRange { got: usize, block_size: usize },
    /// A block layout needs at least one cue.
    EmptyDocument,
}

impl fmt::Display for BlockLayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlockSizeOutOfRange { got } => write!(
                f,
                "block size {got} is out of range {MIN_BLOCK_SIZE}..={MAX_BLOCK_SIZE}"
            ),
            Self::OverlapOutOfRange { got, block_size } => write!(
                f,
                "overlap {got} is out of range 1..{} for block size {block_size}",
                block_size / 2
            ),
            Self::EmptyDocument => f.write_str("document has no cues to lay out into blocks"),
        }
    }
}

impl std::error::Error for BlockLayoutError {}

/// One overlapping translation block: a context window into the document and
/// the subset of that window this block is responsible for translating.
///
/// `output` is always a subset of `window`, and every document position
/// belongs to exactly one block's `output` (see [`BlockLayout::of`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationBlock {
    index: usize,
    window: Range<usize>,
    output: Range<usize>,
}

impl TranslationBlock {
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Document-position range (`0`-based, half-open) this block sees as
    /// context — its own output plus any neighbouring overlap.
    pub fn context_positions(&self) -> Range<usize> {
        self.window.clone()
    }

    /// Document-position range this block is responsible for translating.
    pub fn output_positions(&self) -> Range<usize> {
        self.output.clone()
    }

    /// [`CueId`]s in `document` this block must return a translation for
    /// (ADR-0015 Karar 4 — the real `CueId`, not a block-local index).
    pub fn output_cue_ids(&self, document: &SubtitleDocument) -> Vec<CueId> {
        cue_ids_in(document, self.output.clone())
    }

    /// [`CueId`]s in `document` this block's prompt carries as context,
    /// including its own output.
    pub fn context_cue_ids(&self, document: &SubtitleDocument) -> Vec<CueId> {
        cue_ids_in(document, self.window.clone())
    }
}

fn cue_ids_in(document: &SubtitleDocument, positions: Range<usize>) -> Vec<CueId> {
    let cues = document.cues();
    positions
        .filter_map(|position| cues.get(position))
        .map(|cue| cue.id())
        .collect()
}

/// A document's full, deterministic split into overlapping blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockLayout {
    version: u32,
    config: BlockLayoutConfig,
    cue_count: usize,
    blocks: Vec<TranslationBlock>,
}

impl BlockLayout {
    /// Splits `document` per ADR-0015 Karar 1/2: fixed cue-count blocks, the
    /// final block's window pinned to `cue_count - block_size` so it is
    /// always full size, and each pair of neighbouring windows' overlap
    /// resolved at its floor-midpoint into exactly one owner.
    ///
    /// Deterministic: the same document and config always produce a
    /// bit-for-bit identical [`BlockLayout`].
    pub fn of(
        document: &SubtitleDocument,
        config: BlockLayoutConfig,
    ) -> Result<Self, BlockLayoutError> {
        let cue_count = document.len();
        if cue_count == 0 {
            return Err(BlockLayoutError::EmptyDocument);
        }

        let blocks = compute_blocks(cue_count, config);
        Ok(Self {
            version: BLOCK_LAYOUT_VERSION,
            config,
            cue_count,
            blocks,
        })
    }

    pub const fn version(&self) -> u32 {
        self.version
    }

    pub const fn config(&self) -> BlockLayoutConfig {
        self.config
    }

    pub const fn cue_count(&self) -> usize {
        self.cue_count
    }

    pub fn blocks(&self) -> &[TranslationBlock] {
        &self.blocks
    }
}

/// The layout algorithm itself, kept free of `SubtitleDocument` so it is easy
/// to reason about (and test) purely over cue counts.
fn compute_blocks(cue_count: usize, config: BlockLayoutConfig) -> Vec<TranslationBlock> {
    let block_size = config.block_size();

    // ADR-0015 Karar 1: a document no larger than one block is one block.
    if cue_count <= block_size {
        return vec![TranslationBlock {
            index: 0,
            window: 0..cue_count,
            output: 0..cue_count,
        }];
    }

    let starts = window_starts(cue_count, config);
    let boundaries = output_boundaries(&starts, block_size, cue_count);

    let windows = starts.iter().map(|&start| start..(start + block_size));
    let outputs = boundaries
        .iter()
        .copied()
        .zip(boundaries.iter().copied().skip(1))
        .map(|(start, end)| start..end);

    windows
        .zip(outputs)
        .enumerate()
        .map(|(index, (window, output))| TranslationBlock {
            index,
            window,
            output,
        })
        .collect()
}

/// Start positions of every window's context range, stepping by
/// `block_size - overlap` and pinning the final window to
/// `cue_count - block_size` (ADR-0015 Karar 1). Only called when
/// `cue_count > block_size`, so there are always at least two windows.
fn window_starts(cue_count: usize, config: BlockLayoutConfig) -> Vec<usize> {
    let block_size = config.block_size();
    let step = block_size - config.overlap();

    let mut starts = Vec::new();
    let mut start = 0usize;
    loop {
        starts.push(start);
        if start + block_size >= cue_count {
            break;
        }
        start += step;
    }

    if let Some(last) = starts.last_mut() {
        *last = cue_count - block_size;
    }
    starts
}

/// Output-range boundaries between `starts.len()` windows: `0`, then one
/// floor-midpoint per neighbouring pair (ADR-0015 Karar 2), then
/// `cue_count`. Always strictly increasing — see this module's tests for the
/// proof sketch (`overlap < block_size / 2` is exactly what makes that hold
/// even after the final window is pinned back).
fn output_boundaries(starts: &[usize], block_size: usize, cue_count: usize) -> Vec<usize> {
    let mut boundaries = vec![0usize];
    for (prev_start, next_start) in starts.iter().copied().zip(starts.iter().copied().skip(1)) {
        let prev_end = prev_start + block_size;
        // `prev_end > next_start` always holds here: see `window_starts` and
        // this module's `neighbouring_windows_always_overlap` test.
        let overlap_len = prev_end.saturating_sub(next_start);
        boundaries.push(next_start + overlap_len / 2);
    }
    boundaries.push(cue_count);
    boundaries
}

#[cfg(test)]
mod tests {
    use nen_domain::subtitle::{Cue, TimeSpan};

    use super::*;

    /// `cue_count` cues, 1000 ms apart, non-overlapping. Block layout never
    /// looks at timing, so any well-formed spacing is fine here.
    fn document_of(cue_count: u32) -> SubtitleDocument {
        let cues = (0..cue_count)
            .map(|i| {
                let start = i * 1_000;
                Cue::new(
                    CueId::new(i + 1),
                    TimeSpan::new(start, start + 900).expect("well-formed span"),
                    vec![format!("cue {i}")],
                )
            })
            .collect();
        SubtitleDocument::new(cues)
    }

    #[test]
    fn default_config_satisfies_its_own_gate() {
        let default = BlockLayoutConfig::default();
        BlockLayoutConfig::new(default.block_size(), default.overlap())
            .expect("defaults must pass their own validation");
    }

    #[test]
    fn rejects_block_size_out_of_range() {
        assert_eq!(
            BlockLayoutConfig::new(29, 6),
            Err(BlockLayoutError::BlockSizeOutOfRange { got: 29 })
        );
        assert_eq!(
            BlockLayoutConfig::new(61, 6),
            Err(BlockLayoutError::BlockSizeOutOfRange { got: 61 })
        );
    }

    #[test]
    fn accepts_block_size_boundaries() {
        assert!(BlockLayoutConfig::new(30, 6).is_ok());
        assert!(BlockLayoutConfig::new(60, 6).is_ok());
    }

    #[test]
    fn rejects_overlap_out_of_range() {
        assert_eq!(
            BlockLayoutConfig::new(40, 0),
            Err(BlockLayoutError::OverlapOutOfRange {
                got: 0,
                block_size: 40
            })
        );
        assert_eq!(
            BlockLayoutConfig::new(40, 20),
            Err(BlockLayoutError::OverlapOutOfRange {
                got: 20,
                block_size: 40
            })
        );
    }

    #[test]
    fn accepts_overlap_just_under_half() {
        assert!(BlockLayoutConfig::new(40, 19).is_ok());
    }

    #[test]
    fn rejects_empty_document() {
        let document = document_of(0);
        let config = BlockLayoutConfig::default();
        assert_eq!(
            BlockLayout::of(&document, config),
            Err(BlockLayoutError::EmptyDocument)
        );
    }

    #[test]
    fn document_no_larger_than_a_block_is_a_single_block() {
        let document = document_of(25);
        let layout = BlockLayout::of(&document, BlockLayoutConfig::default())
            .expect("25 cues fit in one block");
        assert_eq!(layout.blocks().len(), 1);
        let block = &layout.blocks()[0];
        assert_eq!(block.output_positions(), 0..25);
        assert_eq!(block.context_positions(), 0..25);
    }

    #[test]
    fn same_document_splits_identically_twice() {
        let document = document_of(95);
        let config = BlockLayoutConfig::default();
        let first = BlockLayout::of(&document, config).expect("valid layout");
        let second = BlockLayout::of(&document, config).expect("valid layout");
        assert_eq!(first, second);
    }

    #[test]
    fn output_ranges_partition_the_whole_document_in_order() {
        for cue_count in [41, 50, 79, 80, 81, 95, 121, 240] {
            let document = document_of(cue_count);
            let layout = BlockLayout::of(&document, BlockLayoutConfig::default())
                .unwrap_or_else(|err| panic!("cue_count {cue_count}: {err}"));

            let mut expected_start = 0usize;
            for block in layout.blocks() {
                let output = block.output_positions();
                assert_eq!(
                    output.start,
                    expected_start,
                    "cue_count {cue_count}, block {}",
                    block.index()
                );
                assert!(
                    output.end > output.start,
                    "cue_count {cue_count}, block {} has an empty output range",
                    block.index()
                );
                expected_start = output.end;
            }
            assert_eq!(
                expected_start, cue_count as usize,
                "cue_count {cue_count}: outputs do not cover the whole document"
            );
        }
    }

    #[test]
    fn neighbouring_windows_always_overlap_and_share_boundary_cues_as_context_only() {
        let document = document_of(137);
        let layout =
            BlockLayout::of(&document, BlockLayoutConfig::default()).expect("valid layout");
        let blocks = layout.blocks();
        assert!(blocks.len() >= 2, "fixture is too small to test overlap");

        for pair in blocks.windows(2) {
            let (Some(left), Some(right)) = (pair.first(), pair.get(1)) else {
                continue;
            };
            let left_window = left.context_positions();
            let right_window = right.context_positions();
            assert!(
                right_window.start < left_window.end,
                "block {} and {} do not overlap",
                left.index(),
                right.index()
            );

            // The boundary sits inside the overlap window (ADR-0015 Karar
            // 2's floor-midpoint), so both neighbours see it as context.
            let boundary = left.output_positions().end;
            assert!(boundary >= right_window.start && boundary <= left_window.end);
        }
    }

    #[test]
    fn final_window_is_pinned_to_cue_count_minus_block_size() {
        let document = document_of(95);
        let layout =
            BlockLayout::of(&document, BlockLayoutConfig::default()).expect("valid layout");
        let last = layout
            .blocks()
            .last()
            .expect("layout has at least one block");
        assert_eq!(last.context_positions(), (95 - 40)..95);
    }

    #[test]
    fn output_cue_ids_are_the_real_document_cue_ids() {
        let document = document_of(50);
        let layout =
            BlockLayout::of(&document, BlockLayoutConfig::default()).expect("valid layout");
        let first_block = layout.blocks().first().expect("at least one block");
        let ids = first_block.output_cue_ids(&document);
        assert_eq!(ids.first().map(|id| id.get()), Some(1));
    }
}
