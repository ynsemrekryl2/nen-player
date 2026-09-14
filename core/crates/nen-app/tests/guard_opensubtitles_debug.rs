use nen_app::identity::ProviderCandidateError;
use nen_app::ports::identity::MediaHash;
use nen_app::ports::subtitle_candidates::SubtitleCandidateSearchError;
use nen_app::subtitles::MenuEntryView;
use nen_domain::source::{
    LanguageTag, SubtitleSource, SubtitleSourceBadges, SubtitleSourceId, SubtitleSourceKind,
};

const PRIVATE_FILE_ID: &str = "private-file-id-884422";
const PRIVATE_FILENAME: &str = "Personal.Movie.tr.srt";
const PRIVATE_HASH: &str = "0001020304050607";

fn language(value: &str) -> LanguageTag {
    LanguageTag::parse(value).expect("language")
}

#[test]
fn provider_source_menu_view_hash_and_error_debug_are_redacted() {
    let source = SubtitleSource::new(
        SubtitleSourceId::opensubtitles(PRIVATE_FILE_ID),
        Some(language("tr")),
        PRIVATE_FILENAME,
    )
    .with_badges(SubtitleSourceBadges {
        hearing_impaired: true,
        ai_translated: true,
    });
    let entry = MenuEntryView {
        token: 1,
        kind: SubtitleSourceKind::OpenSubtitles,
        language: Some(language("tr")),
        label: PRIVATE_FILENAME.into(),
        defect: None,
        translatable: true,
        hearing_impaired: true,
        ai_translated: true,
    };
    let hash = MediaHash::from_bytes([0, 1, 2, 3, 4, 5, 6, 7]);
    let error = ProviderCandidateError::Provider(SubtitleCandidateSearchError::InvalidResponse);
    let output = format!("{source:?} {entry:?} {hash:?} {error:?}");

    assert!(!output.contains(PRIVATE_FILE_ID), "{output}");
    assert!(!output.contains(PRIVATE_FILENAME), "{output}");
    assert!(!output.contains(PRIVATE_HASH), "{output}");
}

#[derive(Debug)]
#[allow(dead_code)]
struct DerivedSensitiveTwin {
    private_file_id: &'static str,
    filename: &'static str,
    hash: &'static str,
}

#[test]
fn derived_sensitive_twin_is_the_negative_control() {
    let output = format!(
        "{:?}",
        DerivedSensitiveTwin {
            private_file_id: PRIVATE_FILE_ID,
            filename: PRIVATE_FILENAME,
            hash: PRIVATE_HASH,
        }
    );
    assert!(output.contains(PRIVATE_FILE_ID));
    assert!(output.contains(PRIVATE_FILENAME));
    assert!(output.contains(PRIVATE_HASH));
}
