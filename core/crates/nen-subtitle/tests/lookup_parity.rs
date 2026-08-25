//! Lookup parity (NEN-017 DoD #1): over 10 000 random seeks the index returns
//! exactly what a linear scan returns — same cues, same order.
//!
//! The index is the only part of this crate whose correctness is not visible
//! in its output format, so it is checked against a reference implementation
//! that is too dumb to be wrong (`support::linear_active_cues`). The seeds are
//! fixed, so a failure reproduces exactly on any machine.

mod support;

use nen_subtitle::index::CueIndex;
use nen_subtitle::srt;
use support::{
    cue_ids, document_end_ms, generated_document, linear_active_cues, linear_cues_in, name, read,
    srt_files, Rng, GENERATED_CUE_PERIOD_MS,
};

use nen_domain::subtitle::SubtitleDocument;

/// Fixed so the seek corpus is identical on every run and machine.
const SEED: u64 = 0x4E45_4E30_3137; // "NEN017"

/// What the DoD asks for.
const SEEK_COUNT: usize = 10_000;

/// Every document the parity sweep runs over, labelled for assertion messages:
/// the real fixture corpus plus synthetic shapes the corpus does not contain
/// (empty, single cue, deep overlap, one document-length cue).
fn corpus() -> Vec<(String, SubtitleDocument)> {
    let mut corpus: Vec<(String, SubtitleDocument)> = srt_files("valid")
        .iter()
        .map(|path| {
            let document = srt::parse(&read(path))
                .unwrap_or_else(|err| panic!("{} should parse: {err:?}", name(path)));
            (name(path), document)
        })
        .collect();

    corpus.push(("generated/empty".to_string(), generated_document(0, 1)));
    corpus.push(("generated/single".to_string(), generated_document(1, 1)));
    corpus.push(("generated/gaps".to_string(), generated_document(200, 1)));
    corpus.push(("generated/depth-3".to_string(), generated_document(200, 3)));
    corpus.push(("generated/depth-8".to_string(), generated_document(200, 8)));
    corpus.push((
        // The shape the prefix-array structure is weakest against: a cue that
        // covers the entire document, so no query can rule out the start.
        "generated/one-long-cue".to_string(),
        long_cue_over(generated_document(200, 1)),
    ));
    corpus
}

/// Prepends a cue spanning the whole of `document`.
fn long_cue_over(document: SubtitleDocument) -> SubtitleDocument {
    use nen_domain::subtitle::{Cue, CueId, TimeSpan};

    let end = document_end_ms(&document);
    let mut cues = vec![Cue::new(
        CueId::new(0),
        TimeSpan::new(0, end).expect("document is not empty"),
        vec!["the whole film".to_string()],
    )];
    cues.extend(document.cues().iter().cloned());
    SubtitleDocument::new(cues)
}

/// Seek space: the document plus a margin on either side, so moments before
/// the first cue and after the last one are sampled too.
fn seek_bound(document: &SubtitleDocument) -> u32 {
    document_end_ms(document).saturating_add(GENERATED_CUE_PERIOD_MS * 2)
}

#[test]
fn ten_thousand_random_seeks_match_a_linear_scan() {
    let corpus = corpus();
    let mut rng = Rng::new(SEED);
    let mut performed = 0;

    for round in 0..SEEK_COUNT {
        let (label, document) = corpus
            .get(round % corpus.len())
            .expect("index is inside the corpus");
        let index = CueIndex::build(document);

        let at_ms = rng.below(seek_bound(document));
        let expected = cue_ids(&linear_active_cues(document, at_ms));
        let actual = cue_ids(&index.active_cues(at_ms));
        assert_eq!(
            actual, expected,
            "{label}: active cues at {at_ms} ms disagree with a linear scan"
        );

        // The single-cue convenience wrapper has to agree with its own list.
        assert_eq!(
            index.active_cue(at_ms).map(|cue| cue.id().get()),
            expected.first().copied(),
            "{label}: active_cue at {at_ms} ms is not the first of active_cues"
        );

        performed += 1;
    }

    assert_eq!(performed, SEEK_COUNT, "the DoD asks for 10 000 seeks");
}

#[test]
fn random_range_queries_match_a_linear_scan() {
    let corpus = corpus();
    let mut rng = Rng::new(SEED);

    for round in 0..SEEK_COUNT {
        let (label, document) = corpus
            .get(round % corpus.len())
            .expect("index is inside the corpus");
        let index = CueIndex::build(document);

        let bound = seek_bound(document);
        let from = rng.below(bound);
        // Windows from a single millisecond up to a large slice of the film.
        let to = from.saturating_add(rng.below(GENERATED_CUE_PERIOD_MS * 10) + 1);

        assert_eq!(
            cue_ids(&index.cues_in(from..to)),
            cue_ids(&linear_cues_in(document, from, to)),
            "{label}: cues in [{from}, {to}) disagree with a linear scan"
        );
    }
}

#[test]
fn every_cue_boundary_is_sampled_exactly() {
    // Random seeks land on boundary milliseconds only by accident, so the
    // exact `start_ms` / `end_ms` / `end_ms - 1` of every cue in the corpus is
    // checked directly as well.
    for (label, document) in corpus() {
        let index = CueIndex::build(&document);
        for cue in document.cues() {
            for at_ms in [
                cue.span().start_ms().saturating_sub(1),
                cue.span().start_ms(),
                cue.span().end_ms() - 1,
                cue.span().end_ms(),
            ] {
                assert_eq!(
                    cue_ids(&index.active_cues(at_ms)),
                    cue_ids(&linear_active_cues(&document, at_ms)),
                    "{label}: boundary {at_ms} ms of cue {} disagrees with a linear scan",
                    cue.id()
                );
            }
        }
    }
}
