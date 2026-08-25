//! Redaction guard for the identity types (NEN-018 DoD #5–#8).
//!
//! `docs/security-policy.md` K23 forbids four of the things this crate handles
//! from ever reaching a log: a private full path (#3), a media URL (#1), a
//! token-bearing query (#2), and hash or filename metadata (#8). Every type
//! that carries one of them writes `Debug` by hand.
//!
//! # The negative control
//!
//! A guard test that only asserts "the forbidden string is absent" can pass
//! for the wrong reason — a typo in the needle, a field that was never
//! populated, a `Debug` that prints nothing useful at all. So this file also
//! builds [`DerivedEvidence`], a deliberately broken twin holding the *same
//! values* with `#[derive(Debug)]`, and proves it really does leak every
//! pattern the real type hides. That is what shows the checks below are
//! testing something.
//!
//! This mirrors NEN-006's `BadFixtureWithDerivedDebug` and NEN-010's
//! sanitizing-constructor bypass.

use nen_identity::container::from_tags;
use nen_identity::evidence::{HandoffMetadata, MediaEvidence};
use nen_identity::nfo;
use nen_identity::os_hash;
use nen_identity::release_name;

/// The private path used throughout, carrying a username the way a real one
/// would.
const PRIVATE_PATH: &str = "/Users/ynsemre/Movies/Library/Inception.2010.1080p.BluRay.x264.mkv";

/// A media URL with a token in its query — K23 #1 and #2 in one string.
const TOKEN_URL: &str =
    "https://stream.private-host.example/deliver/USER7781/Inception.2010.1080p.mkv\
     ?token=eyJhbGciOiJIUzI1NiJ9.SUPERSECRET&user=ynsemre@example.com";

/// Every pattern that must never appear in `Debug` output.
fn forbidden_patterns() -> Vec<&'static str> {
    vec![
        "ynsemre",              // K23 #3 — username inside a private path
        "/Users/",              // K23 #3 — private path structure
        "Library",              // K23 #3 — library layout
        "Inception",            // K23 #8 — filename / title metadata
        "SUPERSECRET",          // K23 #2 — token
        "eyJhbGciOiJIUzI1NiJ9", // K23 #2 — token
        "USER7781",             // K23 #1 — identifying URL path segment
        "private-host.example", // K23 #1 — media host
        "ynsemre@example.com",  // K23 #2 — identifying query value
        "tt1375666",            // K23 #8 — identity metadata from a sidecar
    ]
}

fn full_evidence() -> MediaEvidence {
    let hash = os_hash::of_bytes(&vec![0x5Au8; 300_000]).expect("fixture is large enough");

    MediaEvidence::for_local_file(PRIVATE_PATH)
        .with_declared_name("Inception.2010.1080p.BluRay.x264.mkv")
        .with_size(9_876_543_210)
        .with_os_hash(hash)
        .with_container(from_tags(
            Some("Inception"),
            Some("2010"),
            Some(8_880_000),
            &[],
        ))
        .with_nfo(nfo::parse(
            "<movie><title>Inception</title><year>2010</year>\
             <uniqueid type=\"imdb\">tt1375666</uniqueid></movie>",
        ))
        .with_handoff(HandoffMetadata {
            title: Some("Inception".into()),
            year: Some(2010),
            ..HandoffMetadata::default()
        })
        .with_siblings(vec!["Inception.2010.1080p.BluRay.x264.mkv".into()])
}

/// Everything a caller could plausibly print, in one string.
fn everything_printable(evidence: &MediaEvidence) -> String {
    format!(
        "{evidence:?} {:?} {:?} {:?} {:?} {:?}",
        evidence.resolve(),
        evidence.identity_candidates(),
        evidence.os_hash(),
        evidence.container(),
        evidence.nfo(),
    )
}

#[test]
fn evidence_debug_leaks_none_of_the_forbidden_patterns() {
    let printed = everything_printable(&full_evidence());

    for pattern in forbidden_patterns() {
        assert!(
            !printed.contains(pattern),
            "K23 violation: {pattern:?} appeared in {printed}"
        );
    }
}

#[test]
fn only_the_extension_and_size_class_are_reported() {
    let printed = format!("{:?}", full_evidence());

    assert!(
        printed.contains("mkv"),
        "the extension is loggable: {printed}"
    );
    assert!(
        printed.contains("huge"),
        "the size class is loggable: {printed}"
    );
    assert!(
        !printed.contains("9876543210"),
        "the exact size is not loggable: {printed}"
    );
}

/// DoD #5: a media URL's query must never enter the evidence at all — not the
/// struct, not a hint, not a resolved title, not a candidate.
#[test]
fn a_token_bearing_url_contributes_nothing_but_its_path() {
    let evidence = MediaEvidence::for_remote_url(TOKEN_URL);
    let printed = everything_printable(&evidence);

    for pattern in [
        "SUPERSECRET",
        "eyJhbGciOiJIUzI1NiJ9",
        "token",
        "ynsemre@example.com",
        "private-host.example",
    ] {
        assert!(
            !printed.contains(pattern),
            "{pattern:?} survived from the URL into {printed}"
        );
    }

    // The path still did its job: the identity was recovered from it.
    assert_eq!(evidence.resolve().title.as_deref(), Some("Inception"));
    assert_eq!(evidence.resolve().year, Some(2010));
}

/// DoD #6: a server-declared name can never become a path.
#[test]
fn a_hostile_declared_name_is_reduced_to_one_segment() {
    let hostile = [
        "../../../etc/passwd",
        r"..\..\Windows\System32\config",
        "/etc/shadow",
        "..",
        "a/b/c/Movie.2010.mkv",
        "Movie\u{202E}gnp.2010.mkv",
        "Movie\r\nX-Injected: 1.mkv",
    ];

    for raw in hostile {
        let evidence = MediaEvidence::for_remote_url("https://h/opaque/0").with_declared_name(raw);
        let printed = everything_printable(&evidence);

        assert!(
            !printed.contains('/'),
            "{raw:?} kept a separator: {printed}"
        );
        assert!(
            !printed.contains('\\'),
            "{raw:?} kept a separator: {printed}"
        );
        assert!(!printed.contains('\n'), "{raw:?} kept a newline: {printed}");
        assert!(
            !printed.contains("etc") && !printed.contains("passwd") && !printed.contains("shadow"),
            "{raw:?} kept a path component: {printed}"
        );
    }
}

#[test]
fn parsed_names_and_candidates_print_shape_only() {
    let parsed = release_name::parse("Inception.2010.1080p.BluRay.mkv");
    let printed = format!("{parsed:?}");

    assert!(!printed.contains("Inception"), "title leaked: {printed}");
    assert!(!printed.contains("2010"), "year leaked: {printed}");
    assert!(printed.contains("Movie"), "the kind is loggable: {printed}");
}

#[test]
fn the_hash_prints_as_redacted_wherever_it_appears() {
    let hash = os_hash::of_bytes(&vec![0x5Au8; 300_000]).expect("fixture is large enough");
    let hex = hash.to_hex();

    assert_eq!(format!("{hash:?}"), "OsHash(<redacted>)");
    assert_eq!(format!("{hash}"), "<redacted>");
    assert!(
        !everything_printable(&full_evidence()).contains(&hex),
        "the hash leaked through the evidence"
    );
}

// ---------------------------------------------------------------------------
// Negative control
// ---------------------------------------------------------------------------

/// A deliberately broken twin of [`MediaEvidence`], holding the same values
/// with a derived `Debug`. Nothing in the crate uses this; it exists only to
/// prove the assertions above are not vacuous.
///
/// The fields are read only through the derived `Debug`, which dead-code
/// analysis deliberately does not count — that is precisely the leak being
/// demonstrated, so the allow stays.
#[derive(Debug)]
#[allow(dead_code)]
struct DerivedEvidence {
    path: String,
    url: String,
    declared_name: String,
    size_bytes: u64,
    os_hash_hex: String,
    imdb_id: String,
    title: String,
}

impl DerivedEvidence {
    fn from_the_same_values() -> Self {
        let hash = os_hash::of_bytes(&vec![0x5Au8; 300_000]).expect("fixture is large enough");
        Self {
            path: PRIVATE_PATH.to_string(),
            url: TOKEN_URL.to_string(),
            declared_name: "Inception.2010.1080p.BluRay.x264.mkv".to_string(),
            size_bytes: 9_876_543_210,
            os_hash_hex: hash.to_hex(),
            imdb_id: "tt1375666".to_string(),
            title: "Inception".to_string(),
        }
    }
}

/// The control itself: `#[derive(Debug)]` over these fields really does leak
/// every pattern the hand-written impls hide. If this test ever fails, the
/// guards above have stopped proving anything and the needles need revisiting.
#[test]
fn a_derived_debug_leaks_every_pattern_the_real_types_hide() {
    let leaky = format!("{:?}", DerivedEvidence::from_the_same_values());

    for pattern in forbidden_patterns() {
        assert!(
            leaky.contains(pattern),
            "the negative control failed to leak {pattern:?}; \
             the guard tests above are no longer meaningful"
        );
    }
    assert!(
        leaky.contains("9876543210"),
        "the negative control failed to leak the exact size"
    );

    // And the same values, through the real types, leak none of it.
    let safe = everything_printable(&full_evidence());
    for pattern in forbidden_patterns() {
        assert!(
            !safe.contains(pattern),
            "{pattern:?} leaked from the real types"
        );
    }
}
