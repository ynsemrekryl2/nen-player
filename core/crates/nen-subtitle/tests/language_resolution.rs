//! Cross-boundary behavior for NEN-020's metadata policy and K23 guard.

mod support;

use nen_domain::source::LanguageTag;
use nen_subtitle::{language, srt};

fn fixture(name: &str) -> nen_domain::subtitle::SubtitleDocument {
    let path = support::corpus_dir("languages").join(name);
    srt::parse(&support::read(&path)).unwrap_or_else(|err| panic!("{name}: {err}"))
}

fn tag(value: &str) -> LanguageTag {
    LanguageTag::parse(value).unwrap()
}

#[test]
fn conflicting_metadata_wins_and_exposes_reliable_text_candidate() {
    let resolution = language::resolve_language(&fixture("english.srt"), Some(&tag("tr")))
        .expect("static canonical detector mapping is valid");

    assert_eq!(resolution.language(), Some(&tag("tr")));
    let conflict = resolution
        .metadata_conflict()
        .expect("English text must conflict with Turkish metadata");
    assert_eq!(conflict.detected().language(), &tag("en"));
    assert!(conflict.detected().confidence() > language::MIN_TEXT_CONFIDENCE);
}

#[test]
fn matching_primary_preserves_metadata_region_without_conflict() {
    let resolution = language::resolve_language(&fixture("english.srt"), Some(&tag("en-us")))
        .expect("static canonical detector mapping is valid");

    assert_eq!(resolution.language(), Some(&tag("en-us")));
    assert_eq!(resolution.metadata_conflict(), None);
}

#[test]
fn short_mixed_text_is_unknown_without_metadata_and_does_not_conflict_with_it() {
    let document = fixture("short-mixed.srt");
    let without_metadata = language::resolve_language(&document, None)
        .expect("static canonical detector mapping is valid");
    assert_eq!(without_metadata, language::LanguageResolution::Unknown);

    let with_metadata = language::resolve_language(&document, Some(&tag("fr")))
        .expect("static canonical detector mapping is valid");
    assert_eq!(with_metadata.language(), Some(&tag("fr")));
    assert_eq!(with_metadata.metadata_conflict(), None);
}

#[test]
fn language_results_never_retain_or_debug_subtitle_dialogue() {
    let marker = "K23_PRIVATE_SUBTITLE_DIALOGUE";
    let input = format!("1\n00:00:01,000 --> 00:00:03,000\n{marker}\n");
    let document = srt::parse(&input).expect("guard fixture is valid SRT");
    let resolution = language::resolve_language(&document, Some(&tag("tr")))
        .expect("static canonical detector mapping is valid");

    let printed = format!("{resolution:?}");
    assert!(
        !printed.contains(marker),
        "subtitle dialogue leaked: {printed}"
    );
}
