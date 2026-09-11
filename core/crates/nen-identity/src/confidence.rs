//! Confidence scoring and candidate ranking for media identity (NEN-035).
//!
//! The resolver in [`crate::evidence::MediaEvidence`] deliberately keeps the
//! first usable evidence layer as the backwards-compatible answer.  This
//! module is the cautious projection used when several layers disagree: it
//! keeps every compatible hypothesis, scores the independent support behind
//! each one and only returns an automatic answer when the calibrated threshold
//! is met.

use std::fmt;

use crate::evidence::{IdentityCandidate, MediaEvidence};
use crate::release_name::{MediaKind, ParsedName};
use crate::siblings::{self, Corroboration};

/// The score used by the automatic-selection gate.
pub const AUTOMATIC_CONFIDENCE_THRESHOLD: ConfidenceScore = ConfidenceScore(60);

const VERIFIED_WEIGHT: u16 = 100;
const HANDOFF_WEIGHT: u16 = 90;
const NFO_WEIGHT: u16 = 80;
const CONTAINER_WEIGHT: u16 = 70;
const DECLARED_NAME_WEIGHT: u16 = 60;
const DIRECTORY_WEIGHT: u16 = 50;
const URL_WEIGHT: u16 = 40;
const SUPPORT_BONUS: u16 = 10;
const CORROBORATION_BONUS: u16 = 10;
const UNCONFIRMED_PENALTY: u16 = 10;
const CONFLICT_PENALTY: u16 = 20;

/// A bounded, comparable confidence score.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfidenceScore(u8);

impl ConfidenceScore {
    /// Creates a score, clamping values above the public 0–100 range.
    pub const fn new(value: u8) -> Self {
        Self(if value > 100 { 100 } else { value })
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

impl fmt::Debug for ConfidenceScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ConfidenceScore").field(&self.0).finish()
    }
}

impl fmt::Display for ConfidenceScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A source of identity evidence.  The names are safe to expose; they carry
/// no filename, path, hash or provider payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvidenceSource {
    VerifiedHash,
    Handoff,
    Nfo,
    Container,
    DeclaredName,
    Directory,
    UrlPath,
}

impl EvidenceSource {
    fn weight(self) -> u16 {
        match self {
            Self::VerifiedHash => VERIFIED_WEIGHT,
            Self::Handoff => HANDOFF_WEIGHT,
            Self::Nfo => NFO_WEIGHT,
            Self::Container => CONTAINER_WEIGHT,
            Self::DeclaredName => DECLARED_NAME_WEIGHT,
            Self::Directory => DIRECTORY_WEIGHT,
            Self::UrlPath => URL_WEIGHT,
        }
    }

    fn order(self) -> u8 {
        match self {
            Self::VerifiedHash => 0,
            Self::Handoff => 1,
            Self::Nfo => 2,
            Self::Container => 3,
            Self::DeclaredName => 4,
            Self::Directory => 5,
            Self::UrlPath => 6,
        }
    }
}

/// One ranked, redaction-safe identity candidate.
#[derive(Clone, PartialEq, Eq)]
pub struct RankedIdentityCandidate {
    pub candidate: IdentityCandidate,
    pub score: ConfidenceScore,
    pub strongest_source: EvidenceSource,
    pub supporting_sources: Vec<EvidenceSource>,
}

impl fmt::Debug for RankedIdentityCandidate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RankedIdentityCandidate")
            .field("candidate", &self.candidate)
            .field("score", &self.score)
            .field("strongest_source", &self.strongest_source)
            .field("supporting_sources", &self.supporting_sources)
            .finish()
    }
}

/// The choices a caller may render.  The manual option is deliberately an
/// explicit final item rather than a hidden boolean, so callers cannot put it
/// before the scored candidates by accident.
#[derive(Clone, PartialEq, Eq)]
pub enum IdentityChoice {
    Candidate(RankedIdentityCandidate),
    ManualEntry,
}

impl fmt::Debug for IdentityChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Candidate(candidate) => f.debug_tuple("Candidate").field(candidate).finish(),
            Self::ManualEntry => f.write_str("ManualEntry"),
        }
    }
}

/// The confidence gate's result.
#[derive(Clone, PartialEq, Eq)]
pub enum IdentityAssessment {
    /// The top candidate is safe to use without asking the user.
    Automatic(RankedIdentityCandidate),
    /// Candidates are below the gate or tied.  Manual entry follows them via
    /// [`Self::choices`].
    NeedsSelection(Vec<RankedIdentityCandidate>),
    /// No usable evidence exists; manual entry is the only option.
    ManualEntry,
}

impl IdentityAssessment {
    /// Projects the assessment into the user-facing order.  This is model
    /// data only; drawing the list belongs to a platform layer.
    pub fn choices(&self) -> Vec<IdentityChoice> {
        match self {
            Self::Automatic(candidate) => vec![IdentityChoice::Candidate(candidate.clone())],
            Self::NeedsSelection(candidates) => candidates
                .iter()
                .cloned()
                .map(IdentityChoice::Candidate)
                .chain(std::iter::once(IdentityChoice::ManualEntry))
                .collect(),
            Self::ManualEntry => vec![IdentityChoice::ManualEntry],
        }
    }

    pub fn candidates(&self) -> &[RankedIdentityCandidate] {
        match self {
            Self::Automatic(candidate) => std::slice::from_ref(candidate),
            Self::NeedsSelection(candidates) => candidates,
            Self::ManualEntry => &[],
        }
    }
}

impl fmt::Debug for IdentityAssessment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Automatic(candidate) => f.debug_tuple("Automatic").field(candidate).finish(),
            Self::NeedsSelection(candidates) => {
                f.debug_tuple("NeedsSelection").field(candidates).finish()
            }
            Self::ManualEntry => f.write_str("ManualEntry"),
        }
    }
}

#[derive(Clone)]
struct CandidateGroup {
    parsed: ParsedName,
    sources: Vec<EvidenceSource>,
    conflicts: usize,
}

pub(crate) fn assess(evidence: &MediaEvidence) -> IdentityAssessment {
    let observations = evidence.confidence_layers();
    let usable = observations
        .iter()
        .filter(|(_, parsed)| parsed.is_usable())
        .collect::<Vec<_>>();

    if usable.is_empty() {
        return IdentityAssessment::ManualEntry;
    }

    let mut groups: Vec<CandidateGroup> = Vec::new();
    for (source, parsed) in &usable {
        let Some(group) = groups
            .iter_mut()
            .find(|group| compatible(&group.parsed, parsed))
        else {
            groups.push(CandidateGroup {
                parsed: (*parsed).clone(),
                sources: vec![*source],
                conflicts: 0,
            });
            continue;
        };

        merge(&mut group.parsed, parsed);
        if !group.sources.contains(source) {
            group.sources.push(*source);
            group.sources.sort_by_key(|item| item.order());
        }
    }

    for group in &mut groups {
        group.conflicts = usable
            .iter()
            .filter(|(_, parsed)| !compatible(&group.parsed, parsed))
            .count();
    }

    let mut ranked = groups
        .into_iter()
        .map(|group| rank_group(group, evidence))
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| {
                left.strongest_source
                    .order()
                    .cmp(&right.strongest_source.order())
            })
            .then_with(|| specificity(&right.candidate).cmp(&specificity(&left.candidate)))
        // `sort_by` is stable, so equal candidates retain the evidence
        // walk order without comparing private title text.
    });

    let Some(top) = ranked.first().cloned() else {
        return IdentityAssessment::ManualEntry;
    };
    let tied = ranked.get(1).is_some_and(|next| next.score == top.score);
    let has_conflict = ranked.len() > 1
        && !top
            .supporting_sources
            .contains(&EvidenceSource::VerifiedHash);
    if top.score >= AUTOMATIC_CONFIDENCE_THRESHOLD && !tied && !has_conflict {
        IdentityAssessment::Automatic(top)
    } else {
        IdentityAssessment::NeedsSelection(ranked)
    }
}

fn rank_group(group: CandidateGroup, evidence: &MediaEvidence) -> RankedIdentityCandidate {
    let strongest_source = group
        .sources
        .iter()
        .copied()
        .min_by_key(|source| source.order())
        .unwrap_or(EvidenceSource::UrlPath);
    let corroboration = siblings::corroborate(&group.parsed, evidence.sibling_names());

    let mut score = strongest_source.weight();
    score += SUPPORT_BONUS * group.sources.len().saturating_sub(1) as u16;
    score += match corroboration {
        Corroboration::Confirmed => CORROBORATION_BONUS,
        Corroboration::Unconfirmed => 0,
        Corroboration::NotApplicable => 0,
    };
    if corroboration == Corroboration::Unconfirmed {
        score = score.saturating_sub(UNCONFIRMED_PENALTY);
    }
    score = score.saturating_sub(CONFLICT_PENALTY * group.conflicts as u16);

    RankedIdentityCandidate {
        candidate: to_candidate(&group.parsed),
        score: ConfidenceScore::new(score.min(100) as u8),
        strongest_source,
        supporting_sources: group.sources,
    }
}

fn compatible(left: &ParsedName, right: &ParsedName) -> bool {
    let (Some(left_title), Some(right_title)) = (left.title.as_deref(), right.title.as_deref())
    else {
        return false;
    };
    if normalize(left_title) != normalize(right_title) {
        return false;
    }
    if conflicts(left.year, right.year)
        || conflicts(left.season, right.season)
        || conflicts(left.episode, right.episode)
    {
        return false;
    }
    if left.kind != right.kind
        && left.season.is_none()
        && left.episode.is_none()
        && right.season.is_none()
        && right.episode.is_none()
    {
        return false;
    }
    true
}

fn conflicts<T: PartialEq>(left: Option<T>, right: Option<T>) -> bool {
    matches!((left, right), (Some(left), Some(right)) if left != right)
}

fn merge(target: &mut ParsedName, other: &ParsedName) {
    target.year = target.year.or(other.year);
    target.season = target.season.or(other.season);
    target.episode = target.episode.or(other.episode);
    if target.season.is_some() || target.episode.is_some() {
        target.kind = MediaKind::Series;
    }
}

fn to_candidate(parsed: &ParsedName) -> IdentityCandidate {
    IdentityCandidate {
        title: parsed.title.clone().unwrap_or_default(),
        year: parsed.year,
        season: parsed.season,
        episode: parsed.episode,
    }
}

fn specificity(candidate: &IdentityCandidate) -> u8 {
    u8::from(candidate.year.is_some())
        + u8::from(candidate.season.is_some())
        + u8::from(candidate.episode.is_some())
}

fn normalize(title: &str) -> String {
    title
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::from_tags;
    use crate::evidence::HandoffMetadata;
    use crate::nfo;

    fn movie(path: &str) -> MediaEvidence {
        MediaEvidence::for_local_file(path)
    }

    #[test]
    fn a_strong_single_layer_is_automatic() {
        let result = assess(&movie("/media/Inception.2010.1080p.mkv"));
        let IdentityAssessment::Automatic(candidate) = result else {
            panic!("a declared filename should clear the calibrated gate")
        };
        assert_eq!(candidate.candidate.title, "Inception");
        assert_eq!(candidate.candidate.year, Some(2010));
        assert_eq!(candidate.score.value(), 60);
        assert_eq!(candidate.strongest_source, EvidenceSource::DeclaredName);
    }

    #[test]
    fn independent_layers_confirm_and_raise_the_score() {
        let evidence = movie("/media/Inception.2010.1080p.mkv")
            .with_container(from_tags(Some("Inception"), Some("2010"), None, &[]))
            .with_nfo(nfo::parse(
                "<movie><title>Inception</title><year>2010</year></movie>",
            ));
        let IdentityAssessment::Automatic(candidate) = assess(&evidence) else {
            panic!("matching layers should be automatic")
        };
        assert_eq!(candidate.score.value(), 100);
        assert_eq!(candidate.supporting_sources.len(), 3);
    }

    #[test]
    fn verified_hash_identity_remains_automatic_even_with_a_conflicting_name() {
        let evidence = movie("/media/Wrong.1999.1080p.mkv").with_verified_identity(
            "Verified Film".into(),
            Some(2020),
            None,
            None,
        );
        let IdentityAssessment::Automatic(candidate) = assess(&evidence) else {
            panic!("an exact provider identity is decisive")
        };
        assert_eq!(candidate.candidate.title, "Verified Film");
        assert_eq!(candidate.strongest_source, EvidenceSource::VerifiedHash);
    }

    #[test]
    fn conflicting_layers_are_ranked_and_exposed_before_manual_entry() {
        let evidence = movie("/media/Wrong.1999.1080p.mkv").with_handoff(HandoffMetadata {
            title: Some("Right Film".into()),
            year: Some(2020),
            ..HandoffMetadata::default()
        });
        let result = assess(&evidence);
        let IdentityAssessment::NeedsSelection(ref candidates) = result else {
            panic!("conflicting evidence must ask")
        };
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].candidate.title, "Right Film");
        assert!(matches!(
            result.choices().last(),
            Some(IdentityChoice::ManualEntry)
        ));
    }

    #[test]
    fn no_evidence_returns_only_manual_entry() {
        let result = assess(&MediaEvidence::for_local_file("/media/video.mkv"));
        assert_eq!(result, IdentityAssessment::ManualEntry);
        assert_eq!(result.choices(), vec![IdentityChoice::ManualEntry]);
    }

    #[test]
    fn a_tie_is_never_silently_selected() {
        let evidence = movie("/media/Alpha.2010.mkv")
            .with_handoff(HandoffMetadata {
                title: Some("Beta".into()),
                year: Some(2010),
                ..HandoffMetadata::default()
            })
            .with_container(from_tags(Some("Gamma"), Some("2010"), None, &[]));
        let result = assess(&evidence);
        assert!(matches!(result, IdentityAssessment::NeedsSelection(_)));
    }
}
