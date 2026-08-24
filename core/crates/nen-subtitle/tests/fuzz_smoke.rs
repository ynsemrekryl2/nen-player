//! Fuzz smoke test (NEN-013 DoD #3): the parser never panics, whatever it is
//! fed, and never returns a document that violates its own invariants.
//!
//! This is a deterministic smoke test, not coverage-guided fuzzing: a fixed
//! seed drives a small xorshift generator, so a failure reproduces exactly on
//! any machine and the whole thing runs inside `cargo test` with no extra
//! toolchain. Three case families, each aimed at a different failure mode:
//!
//! 1. **Truncation** — every character boundary of every valid fixture, which
//!    is what a half-written or size-capped download actually looks like.
//! 2. **Mutation** — valid fixtures with characters inserted, deleted or
//!    replaced from the alphabet that matters to this grammar.
//! 3. **Noise** — strings built purely from that alphabet.

mod support;

use nen_subtitle::srt;

/// Fixed so the generated corpus is identical on every run and machine.
const SEED: u64 = 0x4E45_4E30_3133; // "NEN013"

/// The characters this grammar reacts to. Random text drawn from a general
/// alphabet almost never produces a near-miss time line; this one does.
const ALPHABET: &[char] = &[
    '0', '1', '2', '5', '9', ':', ',', '.', '-', '>', ' ', '\n', '\r', '\t', 'a', '\u{feff}',
];

/// xorshift64*, inlined to keep this crate dependency-free.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            (self.next_u64() % bound as u64) as usize
        }
    }

    fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.below(items.len())]
    }
}

/// The contract every case must satisfy: no panic, and if the input was
/// accepted then the resulting document holds up on its own terms.
fn assert_parse_is_total(label: &str, input: &str) {
    let Ok(document) = srt::parse(input) else {
        return; // A rejection is always an acceptable outcome.
    };

    let mut previous_start_ms = 0;
    for (expected_index, cue) in (1..).zip(document.cues()) {
        assert_eq!(
            cue.id().get(),
            expected_index,
            "{label}: accepted a document with a non-sequential index"
        );
        assert!(
            cue.span().start_ms() < cue.span().end_ms(),
            "{label}: accepted a cue that does not move forward in time"
        );
        assert!(
            cue.span().start_ms() >= previous_start_ms,
            "{label}: accepted cues that go backwards in time"
        );
        assert!(
            cue.line_count() > 0,
            "{label}: accepted a cue with no text lines"
        );
        assert!(
            cue.lines().iter().all(|line| !line.trim().is_empty()),
            "{label}: accepted a cue with a blank text line"
        );
        previous_start_ms = cue.span().start_ms();
    }
    assert!(
        !document.is_empty(),
        "{label}: accepted an input as an empty document instead of rejecting it"
    );
}

fn valid_corpus() -> Vec<(String, String)> {
    support::srt_files("valid")
        .iter()
        .map(|path| (support::name(path), support::read(path)))
        .collect()
}

#[test]
fn truncating_a_valid_fixture_anywhere_never_panics() {
    let corpus = valid_corpus();
    assert!(!corpus.is_empty(), "the valid corpus is empty");

    let mut cases = 0;
    for (name, input) in &corpus {
        for (boundary, _) in input.char_indices() {
            let truncated = input.get(..boundary).unwrap_or("");
            assert_parse_is_total(&format!("{name} truncated at {boundary}"), truncated);
            cases += 1;
        }
    }
    assert!(
        cases > 500,
        "expected a meaningful number of cases, ran {cases}"
    );
}

#[test]
fn mutating_a_valid_fixture_never_panics() {
    let corpus = valid_corpus();
    let mut rng = Rng(SEED);

    for (name, input) in &corpus {
        let original: Vec<char> = input.chars().collect();
        for case in 0..1_500 {
            let mut mutated = original.clone();
            for _ in 0..=rng.below(4) {
                if mutated.is_empty() {
                    break;
                }
                let at = rng.below(mutated.len());
                match rng.below(3) {
                    0 => {
                        mutated.remove(at);
                    }
                    1 => mutated.insert(at, rng.pick(ALPHABET)),
                    _ => mutated[at] = rng.pick(ALPHABET),
                }
            }
            let text: String = mutated.into_iter().collect();
            assert_parse_is_total(&format!("{name} mutation #{case}"), &text);
        }
    }
}

#[test]
fn random_noise_never_panics() {
    let mut rng = Rng(SEED ^ 0xFFFF);

    for case in 0..5_000 {
        let length = rng.below(120);
        let text: String = (0..length).map(|_| rng.pick(ALPHABET)).collect();
        assert_parse_is_total(&format!("noise #{case}"), &text);
    }
}

#[test]
fn the_generator_is_deterministic() {
    let mut first = Rng(SEED);
    let mut second = Rng(SEED);
    let a: Vec<u64> = (0..64).map(|_| first.next_u64()).collect();
    let b: Vec<u64> = (0..64).map(|_| second.next_u64()).collect();
    assert_eq!(a, b, "the same seed must produce the same corpus");
}
