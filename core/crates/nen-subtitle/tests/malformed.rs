//! Malformed corpus tests (NEN-013 DoD #2).
//!
//! Every `fixtures/subtitles/malformed/*.srt` isolates exactly one defect and
//! must come back as one specific `SrtError` variant — never `Ok`, never a
//! different variant. The table below is the contract; the completeness test
//! makes it impossible to add a fixture without pinning what it proves.

mod support;

use nen_subtitle::srt;

/// fixture file name → expected `SrtError` variant name.
const EXPECTED: &[(&str, &str)] = &[
    ("empty-input.srt", "EmptyInput"),
    ("leading-bom.srt", "LeadingBom"),
    ("missing-index-line.srt", "MissingIndexLine"),
    ("non-numeric-index.srt", "NonNumericIndex"),
    ("index-overflow.srt", "IndexOverflow"),
    ("duplicate-index.srt", "NonSequentialIndex"),
    ("out-of-order-index.srt", "NonSequentialIndex"),
    ("skipped-index.srt", "NonSequentialIndex"),
    ("missing-time-line.srt", "MissingTimeLine"),
    ("missing-arrow.srt", "MissingArrow"),
    ("short-arrow.srt", "MalformedArrow"),
    ("long-arrow.srt", "MalformedArrow"),
    ("doubled-arrow.srt", "MalformedArrow"),
    (
        "trailing-content-on-time-line.srt",
        "TrailingContentOnTimeLine",
    ),
    ("period-instead-of-comma.srt", "MalformedTimestamp"),
    ("two-digit-millis.srt", "MalformedTimestamp"),
    ("missing-hours.srt", "MalformedTimestamp"),
    ("minutes-out-of-range.srt", "TimestampFieldOutOfRange"),
    ("seconds-out-of-range.srt", "TimestampFieldOutOfRange"),
    ("non-numeric-timestamp.srt", "NonNumericTimestampField"),
    ("negative-timestamp.srt", "NonNumericTimestampField"),
    ("end-before-start.srt", "EndBeforeStart"),
    ("zero-duration.srt", "ZeroDuration"),
    ("non-monotonic-cue.srt", "NonMonotonicCue"),
    ("empty-text.srt", "EmptyText"),
];

#[test]
fn every_malformed_fixture_returns_its_expected_variant() {
    assert!(
        EXPECTED.len() >= 20,
        "the malformed corpus must hold at least 20 cases, found {}",
        EXPECTED.len()
    );

    for (file, expected) in EXPECTED {
        let path = support::corpus_dir("malformed").join(file);
        let input = support::read(&path);

        match srt::parse(&input) {
            Ok(document) => panic!("{file}: expected {expected}, parsed {document:?} instead"),
            Err(err) => assert_eq!(
                err.variant_name(),
                *expected,
                "{file}: wrong error variant ({err})"
            ),
        }
    }
}

/// Guards the table itself: a fixture added to the corpus without an entry
/// here would otherwise sit untested forever.
#[test]
fn the_table_covers_the_whole_malformed_corpus() {
    let on_disk: Vec<String> = support::srt_files("malformed")
        .iter()
        .map(|path| support::name(path))
        .collect();

    for file in &on_disk {
        assert!(
            EXPECTED.iter().any(|(name, _)| name == file),
            "{file}: fixture exists on disk but no expected variant is declared in EXPECTED"
        );
    }

    for (file, _) in EXPECTED {
        assert!(
            on_disk.iter().any(|name| name == file),
            "{file}: declared in EXPECTED but missing from the corpus"
        );
    }
}

/// The distinct-variant requirement is what makes the corpus useful: if two
/// unrelated defects collapsed into one catch-all error, the table would still
/// pass while telling us nothing.
#[test]
fn the_corpus_exercises_a_broad_spread_of_variants() {
    let mut variants: Vec<&str> = EXPECTED.iter().map(|(_, variant)| *variant).collect();
    variants.sort_unstable();
    variants.dedup();
    assert!(
        variants.len() >= 15,
        "expected the corpus to cover at least 15 distinct variants, covered {}: {variants:?}",
        variants.len()
    );
}
