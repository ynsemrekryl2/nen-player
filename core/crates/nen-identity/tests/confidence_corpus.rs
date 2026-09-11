//! Fixture evidence for NEN-035's calibrated confidence gate.

mod support;

use nen_identity::confidence::IdentityAssessment;
use nen_identity::evidence::MediaEvidence;

struct Case {
    path: String,
    kind: String,
    title: String,
    year: Option<u16>,
    season: Option<u16>,
    episode: Option<u16>,
    expectation: String,
}

fn cases() -> Vec<Case> {
    support::read(&support::fixture("identity-confidence.tsv"))
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .map(|line| {
            let fields = line.split('\t').collect::<Vec<_>>();
            assert_eq!(fields.len(), 7, "confidence fixture row has seven fields");
            Case {
                path: fields[0].to_owned(),
                kind: fields[1].to_owned(),
                title: fields[2].to_owned(),
                year: optional(fields[3]),
                season: optional(fields[4]),
                episode: optional(fields[5]),
                expectation: fields[6].to_owned(),
            }
        })
        .collect()
}

fn optional(value: &str) -> Option<u16> {
    (value != "-").then(|| value.parse().expect("numeric confidence fixture field"))
}

#[test]
fn the_fixture_has_zero_wrong_automatic_decisions_and_reports_selection_rate() {
    let cases = cases();
    let mut automatic = 0usize;

    for case in &cases {
        let evidence = MediaEvidence::for_local_file(&format!("/fixture/{}", case.path));
        let assessment = evidence.assess_identity();
        match case.expectation.as_str() {
            "automatic" => {
                automatic += 1;
                let IdentityAssessment::Automatic(candidate) = assessment else {
                    panic!("{} should be automatic", case.path)
                };
                let kind = if candidate.candidate.season.is_some()
                    || candidate.candidate.episode.is_some()
                {
                    "series"
                } else {
                    "movie"
                };
                assert_eq!(kind, case.kind, "{} kind", case.path);
                assert_eq!(candidate.candidate.title, case.title, "{} title", case.path);
                assert_eq!(candidate.candidate.year, case.year, "{} year", case.path);
                assert_eq!(
                    candidate.candidate.season, case.season,
                    "{} season",
                    case.path
                );
                assert_eq!(
                    candidate.candidate.episode, case.episode,
                    "{} episode",
                    case.path
                );
            }
            "selection" => {
                assert!(matches!(assessment, IdentityAssessment::NeedsSelection(_)));
                assert!(matches!(
                    assessment.choices().last(),
                    Some(nen_identity::confidence::IdentityChoice::ManualEntry)
                ));
                let candidate = assessment
                    .candidates()
                    .first()
                    .expect("selection has a candidate");
                let kind = if candidate.candidate.season.is_some()
                    || candidate.candidate.episode.is_some()
                {
                    "series"
                } else {
                    "movie"
                };
                assert_eq!(kind, case.kind);
                assert_eq!(candidate.candidate.title, case.title);
                assert_eq!(candidate.candidate.year, case.year);
            }
            "manual" => assert_eq!(assessment, IdentityAssessment::ManualEntry),
            other => panic!("unknown confidence expectation {other:?}"),
        }
    }

    let percentage = automatic * 100 / cases.len();
    println!(
        "identity confidence fixture: {automatic}/{} automatic ({percentage}%), 0 wrong automatic decisions",
        cases.len()
    );
    assert!(
        percentage >= 70,
        "automatic selection rate regressed: {percentage}%"
    );
}

#[test]
fn the_calibrated_profile_has_a_stable_golden_report() {
    let cases = cases();
    let mut automatic = 0usize;
    let mut selection = 0usize;
    let mut manual = 0usize;

    for case in &cases {
        let evidence = MediaEvidence::for_local_file(&format!("/fixture/{}", case.path));
        match evidence.assess_identity() {
            IdentityAssessment::Automatic(_) => automatic += 1,
            IdentityAssessment::NeedsSelection(_) => selection += 1,
            IdentityAssessment::ManualEntry => manual += 1,
        }
    }

    let report = format!(
        "threshold\t{}\n\
weight\tverified_hash\t100\n\
weight\thandoff\t90\n\
weight\tnfo\t80\n\
weight\tcontainer\t70\n\
weight\tdeclared_name\t60\n\
weight\tdirectory\t50\n\
weight\turl_path\t40\n\
bonus\tindependent_support\t10\n\
bonus\tsibling_confirmation\t10\n\
penalty\tsibling_unconfirmed\t10\n\
penalty\tconflict\t20\n\
outcome\tautomatic\t{automatic}\n\
outcome\tselection\t{selection}\n\
outcome\tmanual\t{manual}\n\
selection_rate_percent\t{}\n\
wrong_automatic\t0\n",
        nen_identity::confidence::AUTOMATIC_CONFIDENCE_THRESHOLD.value(),
        selection * 100 / cases.len(),
    );
    support::assert_golden(&support::fixture("identity-confidence.tsv"), &report);
}

#[test]
fn confidence_debug_never_contains_sensitive_identity_values() {
    let evidence = MediaEvidence::for_local_file("/Users/alice/Private Film.2020.mkv");
    let printed = format!("{:?}", evidence.assess_identity());
    assert!(!printed.contains("Private Film"));
    assert!(!printed.contains("/Users/alice"));
}
