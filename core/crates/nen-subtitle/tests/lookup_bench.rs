//! Lookup baseline (NEN-017 DoD #2): what indexed lookup costs on a 50 000
//! cue document, next to the linear scan `docs/product-spec.md` §14 forbids.
//!
//! **This is a baseline report, not a threshold.** It asserts nothing about
//! timing — machines differ, and a test that fails on a loaded laptop is a
//! test that gets ignored. It is `#[ignore]`d so `cargo test` (and CI) stays
//! fast; run it through `scripts/bench-cue-lookup.sh`, which builds in release
//! and passes `--nocapture`.
//!
//! Method: each measured sample times a *batch* of lookups and divides,
//! because a single lookup is on the order of the clock's own read overhead.
//! Batch sizes differ per implementation (the linear scan is thousands of
//! times slower, so it needs far fewer iterations to fill the same wall
//! clock). A checksum is accumulated from every result and printed, so the
//! optimiser cannot delete the work being measured.
//!
//! Every row is measured twice and only the second pass is reported. Without
//! that, whichever row ran first came out roughly 2.5× slower than the same
//! row running second — first-touch page faults on a freshly allocated 50 000
//! cue document, not a property of the lookup.

mod support;

use std::hint::black_box;
use std::time::{Duration, Instant};

use nen_domain::subtitle::SubtitleDocument;
use nen_subtitle::index::CueIndex;
use support::{
    document_end_ms, generated_document, linear_active_cues, Rng, GENERATED_CUE_PERIOD_MS,
};

const SEED: u64 = 0x4E45_4E30_3137; // "NEN017"
const CUE_COUNT: u32 = 50_000;

/// Timed batches per row; percentiles are taken across these.
const SAMPLES: usize = 100;
const INDEX_BATCH: usize = 100;
const LINEAR_BATCH: usize = 10;

#[test]
#[ignore = "baseline measurement; run via scripts/bench-cue-lookup.sh"]
fn cue_lookup_baseline() {
    println!("\nNEN-017 — indexed cue lookup baseline");
    println!("{CUE_COUNT} cues · {SAMPLES} samples · index batch {INDEX_BATCH} · linear batch {LINEAR_BATCH}");
    println!("Percentiles are per lookup, taken across batch means.\n");

    println!("| document | build | index p50 | index p95 | index p99 | linear p50 | speed-up |");
    println!("|---|---|---|---|---|---|---|");

    for (label, depth) in [("50k cues, no overlap", 1), ("50k cues, depth 3", 3)] {
        let document = generated_document(CUE_COUNT, depth);
        let _discarded = pass(&document);
        report(label, &document, pass(&document));
    }
    println!();
}

struct Pass {
    build: Duration,
    indexed: Vec<Duration>,
    linear: Vec<Duration>,
    checksum: u64,
}

fn pass(document: &SubtitleDocument) -> Pass {
    let started = Instant::now();
    let index = CueIndex::build(document);
    let build = started.elapsed();
    black_box(&index);

    let mut checksum: u64 = 0;
    let indexed = measure(SAMPLES, INDEX_BATCH, |at_ms| {
        let cues = index.active_cues(at_ms);
        checksum = checksum.wrapping_add(black_box(&cues).len() as u64);
    });
    let linear = measure(SAMPLES, LINEAR_BATCH, |at_ms| {
        let cues = linear_active_cues(document, at_ms);
        checksum = checksum.wrapping_add(black_box(&cues).len() as u64);
    });

    Pass {
        build,
        indexed,
        linear,
        checksum,
    }
}

fn report(label: &str, document: &SubtitleDocument, pass: Pass) {
    let bound = document_end_ms(document).saturating_add(GENERATED_CUE_PERIOD_MS);
    let indexed_p50 = percentile(&pass.indexed, 50);
    let linear_p50 = percentile(&pass.linear, 50);

    println!(
        "| {label} | {} | {} | {} | {} | {} | {} |",
        micros(pass.build),
        nanos(indexed_p50),
        nanos(percentile(&pass.indexed, 95)),
        nanos(percentile(&pass.indexed, 99)),
        micros(linear_p50),
        linear_speedup(linear_p50, indexed_p50),
    );
    println!(
        "|   |   |   |   |   | checksum {} | seek bound {bound} ms |",
        pass.checksum
    );
}

/// Runs `samples` batches of `batch` lookups at pseudo-random moments and
/// returns the mean duration of one lookup in each batch.
fn measure(samples: usize, batch: usize, mut lookup: impl FnMut(u32)) -> Vec<Duration> {
    let mut rng = Rng::new(SEED);
    let mut moments = Vec::with_capacity(samples * batch);
    for _ in 0..samples * batch {
        moments.push(rng.below(CUE_COUNT * GENERATED_CUE_PERIOD_MS));
    }

    // Warm the caches so the first sample is not the outlier that defines p99.
    for at_ms in moments.iter().take(batch) {
        lookup(*at_ms);
    }

    let mut per_lookup = Vec::with_capacity(samples);
    for chunk in moments.chunks(batch) {
        let started = Instant::now();
        for at_ms in chunk {
            lookup(*at_ms);
        }
        per_lookup.push(started.elapsed() / chunk.len() as u32);
    }
    per_lookup
}

fn percentile(samples: &[Duration], percent: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percent) / 100;
    let index = rank.min(sorted.len().saturating_sub(1));
    sorted.get(index).copied().unwrap_or_default()
}

fn linear_speedup(linear: Duration, indexed: Duration) -> String {
    if indexed.is_zero() {
        return "n/a".to_string();
    }
    format!("{:.0}×", linear.as_secs_f64() / indexed.as_secs_f64())
}

fn nanos(duration: Duration) -> String {
    format!("{} ns", duration.as_nanos())
}

fn micros(duration: Duration) -> String {
    format!("{:.2} µs", duration.as_secs_f64() * 1e6)
}
