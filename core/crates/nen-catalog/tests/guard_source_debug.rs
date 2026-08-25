use nen_domain::source::{LanguageTag, SubtitleSource, SubtitleSourceId, SubtitleSourceKind};

const PRIVATE_PATH: &str = "/private/library/Personal.Movie.tr.srt";
const PRIVATE_FILENAME: &str = "Personal.Movie.tr.srt";
const PRIVATE_FILE_ID: &str = "private-file-id-884422";
const USER_DIGEST_FRAGMENT: &str = "abababababababab";

fn tag(value: &str) -> LanguageTag {
    LanguageTag::parse(value).expect("fixture language tag is valid")
}

fn assert_forbidden_values_absent(output: &str) {
    for forbidden in [
        PRIVATE_PATH,
        PRIVATE_FILENAME,
        PRIVATE_FILE_ID,
        USER_DIGEST_FRAGMENT,
    ] {
        assert!(
            !output.contains(forbidden),
            "Debug output leaked forbidden value: {forbidden}"
        );
    }
}

#[test]
fn source_and_id_debug_hide_filename_path_digest_and_private_file_id() {
    let user = SubtitleSource::new(
        SubtitleSourceId::user([0xab; 32]),
        Some(tag("tr")),
        PRIVATE_PATH,
    );
    let provider = SubtitleSource::new(
        SubtitleSourceId::opensubtitles(PRIVATE_FILE_ID),
        Some(tag("en")),
        PRIVATE_FILENAME,
    );

    let output = format!(
        "user={user:?} user_id={:?} provider={provider:?} provider_id={:?}",
        user.id(),
        provider.id()
    );
    assert_forbidden_values_absent(&output);
}

#[derive(Debug)]
#[allow(dead_code)] // the derived output is the deliberate read in this negative control
struct DerivedSourceId<'a> {
    kind: SubtitleSourceKind,
    key: &'a str,
}

#[derive(Debug)]
#[allow(dead_code)] // the derived output is the deliberate read in this negative control
struct DerivedSource<'a> {
    id: DerivedSourceId<'a>,
    language: Option<LanguageTag>,
    label: &'a str,
    raw_path: &'a str,
}

#[test]
fn derived_debug_twin_leaks_every_value_the_real_types_hide() {
    let bad_user = DerivedSource {
        id: DerivedSourceId {
            kind: SubtitleSourceKind::User,
            key: USER_DIGEST_FRAGMENT,
        },
        language: Some(tag("tr")),
        label: PRIVATE_FILENAME,
        raw_path: PRIVATE_PATH,
    };
    let bad_provider = DerivedSource {
        id: DerivedSourceId {
            kind: SubtitleSourceKind::OpenSubtitles,
            key: PRIVATE_FILE_ID,
        },
        language: Some(tag("en")),
        label: PRIVATE_FILENAME,
        raw_path: PRIVATE_PATH,
    };

    let output = format!("{bad_user:?} {bad_provider:?}");
    for leaked in [
        PRIVATE_PATH,
        PRIVATE_FILENAME,
        PRIVATE_FILE_ID,
        USER_DIGEST_FRAGMENT,
    ] {
        assert!(
            output.contains(leaked),
            "negative control did not expose {leaked}"
        );
    }
}
