//! Golden test for the release-name corpus (NEN-018 DoD #2, #3).
//!
//! Every name in `fixtures/media/release-names.tsv` is parsed and its result
//! recorded in the `.golden` snapshot beside it. Regenerate after an
//! intentional parser change with:
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test -p nen-identity --test release_name_golden
//! ```

mod support;

use nen_identity::release_name::{self, MediaKind};

#[test]
fn the_corpus_matches_its_golden() {
    let path = support::fixture("release-names.tsv");
    let names = support::entries(&path);

    assert!(
        names.len() >= 30,
        "the corpus must carry at least 30 names (DoD #2), found {}",
        names.len()
    );

    let rendered = names
        .iter()
        .map(|name| support::render(name, &release_name::parse(name)))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";

    support::assert_golden(&path, &rendered);
}

#[test]
fn parsing_is_deterministic_across_runs() {
    for name in support::entries(&support::fixture("release-names.tsv")) {
        let first = release_name::parse(&name);
        let second = release_name::parse(&name);
        assert_eq!(
            support::render(&name, &first),
            support::render(&name, &second),
            "two parses of {name:?} disagree"
        );
    }
}

/// DoD #3: an unresolvable name is a normal outcome, never an error.
///
/// `release_name::parse` has no error type at all, so the thing worth proving
/// is that the corpus really contains names it gives up on — otherwise the
/// claim would be vacuous.
#[test]
fn the_corpus_contains_names_that_stay_unknown() {
    let unknown: Vec<String> = support::entries(&support::fixture("release-names.tsv"))
        .into_iter()
        .filter(|name| release_name::parse(name).kind == MediaKind::Unknown)
        .collect();

    assert!(
        unknown.len() >= 5,
        "expected several deliberately unresolvable names, found {unknown:?}"
    );
    for name in &unknown {
        let parsed = release_name::parse(name);
        assert!(!parsed.is_usable(), "{name:?} should not be usable");
    }
}

/// The corpus is only meaningful if most of it *does* resolve — a parser that
/// gave up on everything would pass the test above.
#[test]
fn the_corpus_resolves_the_names_it_should() {
    let names = support::entries(&support::fixture("release-names.tsv"));
    let resolved = names
        .iter()
        .filter(|name| release_name::parse(name).is_usable())
        .count();

    assert!(
        resolved * 4 >= names.len() * 3,
        "only {resolved}/{} names resolved; the parser has regressed",
        names.len()
    );
}
