//! Hash verification against an independent reference (NEN-018 DoD #1).
//!
//! OSDb's canonical test vectors are multi-megabyte video files, which cannot
//! live in this repository. Committing them is not what makes the check
//! meaningful anyway — what matters is that the shipping implementation agrees
//! with a *second, independently written* one over a corpus we can regenerate
//! anywhere.
//!
//! So this test carries three things:
//!
//! 1. **Hand-verifiable known answers.** An all-zero file's hash is exactly its
//!    own size; that one can be checked with a calculator, no reference
//!    implementation involved (see also the unit tests in `os_hash`).
//! 2. **An independent reference.** [`reference_hash`] below computes the same
//!    value by a deliberately different route — explicit index arithmetic and
//!    manual bit shifting rather than `as_chunks` and `from_le_bytes` — so a
//!    mistake would have to be made twice, in two different shapes, to pass.
//! 3. **A committed golden.** Deterministic synthetic files pin the exact hex
//!    output, so a change in the wire format cannot slip through even if both
//!    implementations were changed together.
//!
//! Regenerate the golden after an intentional change with:
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test -p nen-identity --test os_hash_reference
//! ```

mod support;

use nen_identity::os_hash::{self, OsHashError, CHUNK_BYTES, MIN_FILE_BYTES};

/// Sizes exercised by the corpus, including every boundary that matters.
fn corpus_sizes() -> Vec<usize> {
    let min = MIN_FILE_BYTES as usize;
    vec![
        min,              // exactly the minimum
        min + 1,          // one byte over
        min + 4096,       // a middle region that is never read
        min * 2,          // comfortably large
        min + 7,          // a size that is not a multiple of 8
        3 * CHUNK_BYTES,  // head and tail with a full chunk between them
        10 * 1024 * 1024, // a realistic small media file
    ]
}

/// Deterministic pseudo-random bytes — xorshift64*, the same generator
/// `nen-subtitle`'s fuzz smoke test uses, so a corpus is reproducible on any
/// machine without being committed.
fn synthetic(size: usize, seed: u64) -> Vec<u8> {
    let mut state = seed;
    let mut out = Vec::with_capacity(size);
    while out.len() < size {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let word = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
        out.extend_from_slice(&word.to_le_bytes());
    }
    out.truncate(size);
    out
}

/// An independent implementation of the OSDb hash.
///
/// Written to share as little as possible with the one under test: it walks
/// the windows by index, assembles each `u64` by shifting bytes into place,
/// and renders the result with explicit nibble arithmetic rather than a
/// formatting helper.
fn reference_hash(contents: &[u8]) -> String {
    let size = contents.len();
    assert!(
        size >= 2 * CHUNK_BYTES,
        "caller must supply a large enough file"
    );

    let mut sum: u64 = size as u64;

    let mut offset = 0usize;
    while offset < CHUNK_BYTES {
        let mut word: u64 = 0;
        for byte_index in 0..8 {
            let byte = contents[offset + byte_index] as u64;
            word |= byte << (8 * byte_index);
        }
        sum = sum.wrapping_add(word);
        offset += 8;
    }

    let tail_start = size - CHUNK_BYTES;
    let mut offset = 0usize;
    while offset < CHUNK_BYTES {
        let mut word: u64 = 0;
        for byte_index in 0..8 {
            let byte = contents[tail_start + offset + byte_index] as u64;
            word |= byte << (8 * byte_index);
        }
        sum = sum.wrapping_add(word);
        offset += 8;
    }

    let digits = b"0123456789abcdef";
    let mut hex = String::with_capacity(16);
    for shift in (0..16).rev() {
        let nibble = ((sum >> (shift * 4)) & 0xF) as usize;
        hex.push(digits[nibble] as char);
    }
    hex
}

#[test]
fn the_implementation_agrees_with_an_independent_reference() {
    for (index, size) in corpus_sizes().into_iter().enumerate() {
        let contents = synthetic(size, 0x4E45_4E30_3138 ^ index as u64);
        let ours = os_hash::of_bytes(&contents)
            .unwrap_or_else(|err| panic!("size {size}: {err}"))
            .to_hex();
        assert_eq!(
            ours,
            reference_hash(&contents),
            "size {size}: the two implementations disagree"
        );
    }
}

#[test]
fn the_corpus_matches_its_golden() {
    let rendered = corpus_sizes()
        .into_iter()
        .enumerate()
        .map(|(index, size)| {
            let contents = synthetic(size, 0x4E45_4E30_3138 ^ index as u64);
            let hex = os_hash::of_bytes(&contents)
                .unwrap_or_else(|err| panic!("size {size}: {err}"))
                .to_hex();
            format!("{size}\t{hex}")
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";

    support::assert_golden(&support::fixture("os-hash.tsv"), &rendered);
}

/// The windowed form and the whole-file form must never disagree — this is
/// what lets NEN-036 compute the same hash from two `Range` requests that a
/// local read produces from the file.
#[test]
fn the_windowed_form_matches_the_whole_file_form() {
    let contents = synthetic(MIN_FILE_BYTES as usize + 12_345, 7);
    let head = &contents[..CHUNK_BYTES];
    let tail = &contents[contents.len() - CHUNK_BYTES..];

    assert_eq!(
        os_hash::of(contents.len() as u64, head, tail).unwrap(),
        os_hash::of_bytes(&contents).unwrap()
    );
}

#[test]
fn a_file_below_the_minimum_is_rejected_at_every_boundary() {
    for size in [0usize, 1, CHUNK_BYTES, MIN_FILE_BYTES as usize - 1] {
        let contents = synthetic(size, 3);
        assert!(
            matches!(
                os_hash::of_bytes(&contents),
                Err(OsHashError::TooSmall { .. })
            ),
            "size {size} should have been refused"
        );
    }
    assert!(os_hash::of_bytes(&synthetic(MIN_FILE_BYTES as usize, 3)).is_ok());
}

#[test]
fn changing_one_byte_in_either_window_changes_the_hash() {
    let base = synthetic(MIN_FILE_BYTES as usize + 1024, 11);
    let baseline = os_hash::of_bytes(&base).unwrap();

    for position in [
        0usize,
        7,
        CHUNK_BYTES - 1,
        base.len() - CHUNK_BYTES,
        base.len() - 1,
    ] {
        let mut changed = base.clone();
        changed[position] ^= 0x01;
        assert_ne!(
            os_hash::of_bytes(&changed).unwrap(),
            baseline,
            "flipping a bit at {position} did not change the hash"
        );
    }
}

#[test]
fn changing_a_byte_between_the_windows_does_not() {
    let mut base = synthetic(MIN_FILE_BYTES as usize + 4096, 13);
    let baseline = os_hash::of_bytes(&base).unwrap();

    let middle = CHUNK_BYTES + 2048;
    base[middle] ^= 0xFF;
    assert_eq!(
        os_hash::of_bytes(&base).unwrap(),
        baseline,
        "the unread middle of the file must not affect the hash"
    );
}
