//! `ValidatedSubtitleArtifact` assembly and its WebVTT output (NEN-094,
//! `docs/product-spec.md` §11).
//!
//! [`assemble`] is the only way to build a [`ValidatedSubtitleArtifact`], and
//! it only accepts a [`CompletedBlocks`] — a type [`crate::checkpoint`] can
//! produce only once every block of a run is checkpointed and validated. A
//! caller cannot construct one from a partial run, so a half-translated
//! document can never become an artifact (the "no partial publish" rule
//! carried one more step down the pipeline).
//!
//! `assemble` re-derives the final document from the source
//! [`SubtitleDocument`] and `completed`'s validated cues rather than trusting
//! either input on its own: every cue ID and `TimeSpan` in the result is
//! checked against the source at its exact document position before it is
//! accepted, so a `completed` set assembled against a different document (or
//! reordered relative to `layout`) is rejected rather than silently
//! stitched into something that looks like a translation of `document` but
//! is not. WebVTT rendering reuses [`nen_subtitle::webvtt::write`] — this
//! module does not write subtitle text twice.

use crate::blocks::{BlockLayout, BlockLayoutConfig};
use crate::checkpoint::{CompletedBlocks, TranslationPlan};
use crate::identity::{CacheIdentity, CacheIdentityInput};
use nen_domain::source::LanguageTag;
use nen_domain::subtitle::{Cue, CueId, SubtitleDocument};
use nen_ports::identity::MediaHash;
use nen_ports::persistence::ArtifactRecord;
use nen_ports::translation::TranslationProviderIdentity;
use nen_subtitle::fingerprint::{SourceFingerprint, TimelineFingerprint};
use std::fmt;

/// Re-exported from [`crate::versions`], where every hand-bumped pipeline
/// version constant lives together (ADR-0018 Karar 4, `NEN-097`). Kept at
/// this path too so the call site below and any existing caller need no
/// change.
pub use crate::versions::PIPELINE_VERSION;

/// Longest allowed [`ArtifactId`] value, in bytes.
const MAX_ARTIFACT_ID_LEN: usize = 200;

/// Opaque, externally-assigned artifact identifier.
///
/// This crate does not mint IDs — the storage layer that will eventually
/// persist an artifact (`NEN-096`'s content-addressed store, or `NEN-097`'s
/// cache identity) owns that decision, and neither ADR is `accepted` yet
/// (Kural 4). `ArtifactId` only validates the shape any reasonable scheme
/// (a content hash, a UUID) will produce: non-empty, bounded, and free of
/// whitespace/control characters that would make it unsafe to use as a path
/// segment or index key later.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArtifactId(String);

/// Why a candidate string could not become an [`ArtifactId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactIdError {
    Empty,
    TooLong { length: usize },
    InvalidChar { position: usize },
}

impl fmt::Display for ArtifactIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("artifact id is empty"),
            Self::TooLong { length } => {
                write!(
                    f,
                    "artifact id is {length} bytes, longer than {MAX_ARTIFACT_ID_LEN}"
                )
            }
            Self::InvalidChar { position } => {
                write!(
                    f,
                    "artifact id has a disallowed character at byte {position}"
                )
            }
        }
    }
}

impl std::error::Error for ArtifactIdError {}

impl ArtifactId {
    /// Accepts ASCII alphanumerics plus `-` `_` `.` `:` — enough for a hex
    /// digest, a UUID, or a namespaced key like `blake3:...`, and nothing
    /// that would need escaping as a path segment or index key.
    pub fn parse(value: &str) -> Result<Self, ArtifactIdError> {
        if value.is_empty() {
            return Err(ArtifactIdError::Empty);
        }
        if value.len() > MAX_ARTIFACT_ID_LEN {
            return Err(ArtifactIdError::TooLong {
                length: value.len(),
            });
        }
        if let Some((position, _)) = value
            .char_indices()
            .find(|(_, c)| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':')))
        {
            return Err(ArtifactIdError::InvalidChar { position });
        }
        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ArtifactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArtifactId({})", self.0)
    }
}

/// Glossary identity carried by an artifact. M5 has no glossary write
/// surface (that is M6), so every artifact assembled today carries
/// [`GlossaryIdentity::none`]; the field exists so `NEN-097`'s cache identity
/// (ADR-0018) has somewhere to read a real value from once one exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlossaryIdentity {
    None,
    Named(String),
}

impl GlossaryIdentity {
    pub const fn none() -> Self {
        Self::None
    }

    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }
}

/// Milliseconds since the Unix epoch. `nen-translate` performs no I/O and
/// reads no clock (crate docs) — the caller supplies this, which is also
/// what keeps artifact assembly deterministic and golden-testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArtifactTimestamp(u64);

impl ArtifactTimestamp {
    pub const fn from_unix_ms(unix_ms: u64) -> Self {
        Self(unix_ms)
    }

    pub const fn unix_ms(self) -> u64 {
        self.0
    }
}

/// Everything about an artifact that [`assemble`] cannot derive from the
/// source document, layout and completed blocks themselves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactMetadata {
    pub id: ArtifactId,
    pub provider: TranslationProviderIdentity,
    pub glossary: GlossaryIdentity,
    pub media_hash: Option<MediaHash>,
    pub created_at: ArtifactTimestamp,
}

/// A translation result that exists only because the whole source document
/// was translated and locally validated (`docs/product-spec.md` §11).
///
/// Can only be produced by [`assemble`]. Cue ID, order and `TimeSpan` of
/// [`Self::translated_document`] are byte-for-byte the source document's —
/// [`assemble`] checks this at every cue before returning `Ok`.
pub struct ValidatedSubtitleArtifact {
    id: ArtifactId,
    source_fingerprint: SourceFingerprint,
    timeline_fingerprint: TimelineFingerprint,
    source_language: LanguageTag,
    target_language: LanguageTag,
    provider: TranslationProviderIdentity,
    pipeline_version: u32,
    block_layout_version: u32,
    /// The block layout `assemble` was given — kept so
    /// [`Self::cache_identity`] can derive ADR-0018's identity from the
    /// artifact's own fields rather than asking a caller for it again.
    block_layout: BlockLayoutConfig,
    glossary: GlossaryIdentity,
    media_hash: Option<MediaHash>,
    created_at: ArtifactTimestamp,
    translated: SubtitleDocument,
    webvtt: String,
}

impl ValidatedSubtitleArtifact {
    pub const fn id(&self) -> &ArtifactId {
        &self.id
    }

    pub const fn source_fingerprint(&self) -> SourceFingerprint {
        self.source_fingerprint
    }

    pub const fn timeline_fingerprint(&self) -> TimelineFingerprint {
        self.timeline_fingerprint
    }

    pub const fn source_language(&self) -> &LanguageTag {
        &self.source_language
    }

    pub const fn target_language(&self) -> &LanguageTag {
        &self.target_language
    }

    pub const fn provider(&self) -> &TranslationProviderIdentity {
        &self.provider
    }

    pub const fn pipeline_version(&self) -> u32 {
        self.pipeline_version
    }

    pub const fn block_layout_version(&self) -> u32 {
        self.block_layout_version
    }

    pub const fn glossary(&self) -> &GlossaryIdentity {
        &self.glossary
    }

    pub const fn media_hash(&self) -> Option<MediaHash> {
        self.media_hash
    }

    pub const fn created_at(&self) -> ArtifactTimestamp {
        self.created_at
    }

    /// Normalized cues: source order, source `CueId`s, source `TimeSpan`s,
    /// translated text.
    pub const fn translated_document(&self) -> &SubtitleDocument {
        &self.translated
    }

    /// UTF-8 WebVTT rendering of [`Self::translated_document`]
    /// (`nen_subtitle::webvtt::write`, not re-implemented here).
    pub fn webvtt(&self) -> &str {
        &self.webvtt
    }

    /// This artifact's own ADR-0018 cache identity (`NEN-097`/`NEN-098`),
    /// derived entirely from fields the artifact already carries — a
    /// [`CacheIdentityInput`] built here can never disagree with the
    /// artifact it describes, because there is nowhere else for the values
    /// to come from.
    pub fn cache_identity(&self) -> CacheIdentity {
        CacheIdentity::of(&CacheIdentityInput {
            source_fingerprint: self.source_fingerprint,
            source_language: &self.source_language,
            target_language: &self.target_language,
            provider: &self.provider,
            media_hash: self.media_hash,
            glossary: &self.glossary,
            block_layout: self.block_layout,
        })
    }

    /// Projects this artifact into the shape the persistence port stores
    /// (`NEN-096`, ADR-0017 Karar 2).
    ///
    /// The record lives in `nen-ports` rather than here because ADR-0006
    /// confines `nen-persist` to `nen-domain` + `nen-ports`; it cannot see
    /// this type. Every field below is one of the getters above, so the
    /// stored file is exactly what a validated artifact already knows about
    /// itself — no storage layer invents a field. `cache_identity` is
    /// [`Self::cache_identity`]'s own computed value (ADR-0018, `NEN-098`),
    /// not a second, independently supplied one.
    ///
    /// **There is deliberately no inverse.** Reading a store yields an
    /// [`ArtifactRecord`], never a `ValidatedSubtitleArtifact`: [`assemble`]
    /// stays the only way to obtain one, so bytes off a disk this process
    /// does not own cannot impersonate a validated translation.
    ///
    /// [`Self::id`] is **not** part of the record. The stored artifact's
    /// identity is its content address, which the store mints from the
    /// serialized bytes; carrying an externally assigned id inside those
    /// bytes would give the same translation two addresses under two ids.
    pub fn to_record(&self) -> ArtifactRecord {
        ArtifactRecord {
            source_fingerprint: *self.source_fingerprint.as_bytes(),
            timeline_fingerprint: *self.timeline_fingerprint.as_bytes(),
            source_language: self.source_language.clone(),
            target_language: self.target_language.clone(),
            provider: self.provider.clone(),
            pipeline_version: self.pipeline_version,
            block_layout_version: self.block_layout_version,
            glossary: match &self.glossary {
                GlossaryIdentity::None => None,
                GlossaryIdentity::Named(name) => Some(name.clone()),
            },
            media_hash: self.media_hash,
            created_at_unix_ms: self.created_at.unix_ms(),
            document: self.translated.clone(),
            webvtt: self.webvtt.clone(),
            cache_identity: self.cache_identity().into(),
        }
    }
}

/// Hand-written per `docs/security-policy.md` §1 (K23 #4): an artifact
/// carries translated dialogue in `translated`/`webvtt`, so `Debug` reports
/// shape and identity only, never text. There is intentionally no `Display`
/// impl — nothing about this type should ever be formatted for a log line.
impl fmt::Debug for ValidatedSubtitleArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidatedSubtitleArtifact")
            .field("id", &self.id)
            .field("source_fingerprint", &self.source_fingerprint)
            .field("timeline_fingerprint", &self.timeline_fingerprint)
            .field("source_language", &self.source_language)
            .field("target_language", &self.target_language)
            .field("provider", &self.provider)
            .field("pipeline_version", &self.pipeline_version)
            .field("block_layout_version", &self.block_layout_version)
            .field("glossary", &self.glossary)
            .field("cue_count", &self.translated.len())
            .field("webvtt_len", &self.webvtt.len())
            .finish()
    }
}

/// A structural reason [`assemble`] refused to produce an artifact. No
/// variant carries cue text — only positions, counts and [`CueId`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactError {
    /// `layout` was not built for `document` (`BlockLayout::cue_count`
    /// disagrees with the document's own length).
    LayoutMismatch { expected: usize, received: usize },
    /// `completed` does not have one block per block in `layout` — a
    /// `completed` set assembled against a differently-shaped layout than
    /// the one passed here.
    BlockCount { expected: usize, received: usize },
    /// A completed block's cue count does not match the layout's output
    /// window for that block.
    BlockCueCount {
        block_index: usize,
        expected: usize,
        received: usize,
    },
    /// A validated cue's ID does not match the source document at this
    /// document position.
    CueMismatch {
        position: usize,
        expected: CueId,
        received: CueId,
    },
    /// A validated cue's timing does not match the source document's.
    SpanMismatch { cue_id: CueId },
    /// The assembled document's timeline does not match the source's, after
    /// every field-level check above already passed. Should be unreachable
    /// in practice — kept as the final, cheap gate on the exact invariant
    /// the DoD asks for.
    TimelineMismatch,
}

impl fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LayoutMismatch { expected, received } => write!(
                f,
                "layout was built for {expected} cues, document has {received}"
            ),
            Self::BlockCount { expected, received } => write!(
                f,
                "completed blocks ({received}) do not match layout blocks ({expected})"
            ),
            Self::BlockCueCount {
                block_index,
                expected,
                received,
            } => write!(
                f,
                "block {block_index} has {received} cues, layout expects {expected}"
            ),
            Self::CueMismatch {
                position,
                expected,
                received,
            } => write!(
                f,
                "document position {position} expected cue {expected}, got {received}"
            ),
            Self::SpanMismatch { cue_id } => {
                write!(f, "cue {cue_id} timing does not match the source document")
            }
            Self::TimelineMismatch => {
                f.write_str("assembled document's timeline fingerprint does not match the source")
            }
        }
    }
}

impl std::error::Error for ArtifactError {}

/// Assembles a [`ValidatedSubtitleArtifact`] from a fully checkpointed
/// translation run.
///
/// `completed` can only exist once every block of `layout` validated
/// (`crate::checkpoint::BlockCheckpoints::into_completed`), but this function
/// does not stop there: it re-checks every cue's ID and `TimeSpan` against
/// `document` at the exact position `layout` assigned it, so `completed`
/// blocks that do not actually belong to `document` (wrong document, wrong
/// layout, reordered) are rejected rather than assembled into something that
/// only looks valid.
pub fn assemble(
    document: &SubtitleDocument,
    layout: &BlockLayout,
    plan: &TranslationPlan,
    completed: &CompletedBlocks,
    metadata: ArtifactMetadata,
) -> Result<ValidatedSubtitleArtifact, ArtifactError> {
    if layout.cue_count() != document.len() {
        return Err(ArtifactError::LayoutMismatch {
            expected: layout.cue_count(),
            received: document.len(),
        });
    }

    let layout_blocks = layout.blocks();
    let completed_blocks = completed.blocks();
    if completed_blocks.len() != layout_blocks.len() {
        return Err(ArtifactError::BlockCount {
            expected: layout_blocks.len(),
            received: completed_blocks.len(),
        });
    }

    let document_cues = document.cues();
    let mut translated_cues = Vec::with_capacity(document.len());

    // `layout.blocks()` and `completed.blocks()` are both, by construction,
    // ordered by block index starting at 0 (`BlockLayout::of`,
    // `BlockCheckpoints::into_completed`) — pairing them by position here is
    // therefore already pairing them by block index; there is no separate
    // "block order" to go wrong that either structure's own invariants don't
    // already rule out.
    for (block, validated) in layout_blocks.iter().zip(completed_blocks) {
        let output = block.output_positions();
        if validated.cues().len() != output.len() {
            return Err(ArtifactError::BlockCueCount {
                block_index: block.index(),
                expected: output.len(),
                received: validated.cues().len(),
            });
        }

        for (doc_position, validated_cue) in output.zip(validated.cues()) {
            let Some(source_cue) = document_cues.get(doc_position) else {
                return Err(ArtifactError::LayoutMismatch {
                    expected: layout.cue_count(),
                    received: document.len(),
                });
            };

            if source_cue.id() != validated_cue.cue_id() {
                return Err(ArtifactError::CueMismatch {
                    position: doc_position,
                    expected: source_cue.id(),
                    received: validated_cue.cue_id(),
                });
            }
            if source_cue.span() != validated_cue.span() {
                return Err(ArtifactError::SpanMismatch {
                    cue_id: source_cue.id(),
                });
            }

            translated_cues.push(Cue::new(
                source_cue.id(),
                source_cue.span(),
                cue_lines(validated_cue.text()),
            ));
        }
    }

    let translated = SubtitleDocument::new(translated_cues);
    let timeline_fingerprint = TimelineFingerprint::of(document);
    if TimelineFingerprint::of(&translated) != timeline_fingerprint {
        return Err(ArtifactError::TimelineMismatch);
    }

    let webvtt = nen_subtitle::webvtt::write(&translated);

    Ok(ValidatedSubtitleArtifact {
        id: metadata.id,
        source_fingerprint: SourceFingerprint::of(document),
        timeline_fingerprint,
        source_language: plan.source_language.clone(),
        target_language: plan.target_language.clone(),
        provider: metadata.provider,
        pipeline_version: PIPELINE_VERSION,
        block_layout_version: layout.version(),
        block_layout: layout.config(),
        glossary: metadata.glossary,
        media_hash: metadata.media_hash,
        created_at: metadata.created_at,
        translated,
        webvtt,
    })
}

/// Splits one validated cue's text back into display lines the way
/// [`crate::checkpoint::request_for_block`] joined them (`lines().join("\n")`).
/// `\r\n` is normalized to `\n` first; a line left empty by that split is
/// dropped rather than kept, because an empty line inside a WebVTT cue
/// payload ends the cue early (`nen_subtitle::webvtt`) — it is not a valid
/// display line, just an artifact of the join/split round trip. Validation
/// (`crate::validation::validate_block`) already requires the cue's trimmed
/// text to be non-empty, so at least one non-empty line always survives.
fn cue_lines(text: &str) -> Vec<String> {
    text.replace("\r\n", "\n")
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{BlockLayout, BlockLayoutConfig};
    use crate::checkpoint::{translate_checkpointed, BlockCheckpoints};
    use nen_domain::subtitle::{Cue, CueId, TimeSpan};
    use nen_ports::translation::{
        TranslatedCue, TranslationCall, TranslationProvider, TranslationProviderError,
        TranslationProviderIdentity, TranslationRequest, TranslationResponse,
    };

    fn document(cue_count: u32) -> SubtitleDocument {
        SubtitleDocument::new(
            (1..=cue_count)
                .map(|id| {
                    Cue::new(
                        CueId::new(id),
                        TimeSpan::new(id * 1_000, id * 1_000 + 500).expect("valid span"),
                        vec![format!("Source {id}")],
                    )
                })
                .collect(),
        )
    }

    fn layout(document: &SubtitleDocument) -> BlockLayout {
        BlockLayout::of(document, BlockLayoutConfig::new(30, 1).expect("config")).expect("layout")
    }

    fn plan() -> TranslationPlan {
        TranslationPlan {
            source_language: LanguageTag::parse("en").expect("language"),
            target_language: LanguageTag::parse("tr").expect("language"),
            context_terms: Vec::new(),
        }
    }

    fn metadata() -> ArtifactMetadata {
        ArtifactMetadata {
            id: ArtifactId::parse("artifact-1").expect("id"),
            provider: TranslationProviderIdentity::new("test", "echo").expect("identity"),
            glossary: GlossaryIdentity::none(),
            media_hash: None,
            created_at: ArtifactTimestamp::from_unix_ms(0),
        }
    }

    struct EchoProvider;

    impl TranslationProvider for EchoProvider {
        fn identity(&self) -> TranslationProviderIdentity {
            TranslationProviderIdentity::new("test", "echo").expect("identity")
        }

        fn translate(
            &self,
            request: &TranslationRequest,
            call: &TranslationCall,
        ) -> Result<TranslationResponse, TranslationProviderError> {
            let cues = request
                .output_cue_ids
                .iter()
                .map(|cue_id| TranslatedCue {
                    cue_id: *cue_id,
                    text: format!("translated {}", cue_id.get()),
                })
                .collect();
            call.finish(TranslationResponse { cues })
        }
    }

    fn completed_blocks_for(document: &SubtitleDocument, layout: &BlockLayout) -> CompletedBlocks {
        let mut checkpoints = BlockCheckpoints::for_layout(layout);
        translate_checkpointed(
            &EchoProvider,
            document,
            layout,
            &plan(),
            &TranslationCall::without_progress(),
            &mut checkpoints,
        )
        .expect("run completes");
        checkpoints.into_completed().expect("all checkpointed")
    }

    #[test]
    fn assembled_artifact_matches_source_cue_ids_order_and_spans() {
        let document = document(95);
        let layout = layout(&document);
        let completed = completed_blocks_for(&document, &layout);

        let artifact = assemble(&document, &layout, &plan(), &completed, metadata())
            .expect("assembly succeeds");

        let translated = artifact.translated_document();
        assert_eq!(translated.len(), document.len());
        for (source, translated) in document.cues().iter().zip(translated.cues()) {
            assert_eq!(source.id(), translated.id());
            assert_eq!(source.span(), translated.span());
        }
    }

    #[test]
    fn timeline_fingerprint_matches_source_document() {
        let document = document(95);
        let layout = layout(&document);
        let completed = completed_blocks_for(&document, &layout);

        let artifact = assemble(&document, &layout, &plan(), &completed, metadata())
            .expect("assembly succeeds");

        assert_eq!(
            artifact.timeline_fingerprint(),
            TimelineFingerprint::of(&document)
        );
    }

    #[test]
    fn interior_blank_line_is_dropped_but_a_wholly_blank_split_never_happens() {
        assert_eq!(cue_lines("hello\n\nworld"), vec!["hello", "world"]);
        assert_eq!(cue_lines("just one line"), vec!["just one line"]);
        assert_eq!(
            cue_lines("line one\r\nline two"),
            vec!["line one", "line two"]
        );
    }

    #[test]
    fn to_record_projects_every_field_and_drops_the_id() {
        let document = document(95);
        let layout = layout(&document);
        let completed = completed_blocks_for(&document, &layout);
        let mut metadata = metadata();
        metadata.glossary = GlossaryIdentity::named("hunter-x-hunter");
        metadata.media_hash = Some(MediaHash::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]));
        metadata.created_at = ArtifactTimestamp::from_unix_ms(1_700_000_000_000);

        let artifact =
            assemble(&document, &layout, &plan(), &completed, metadata).expect("assembly succeeds");
        let record = artifact.to_record();

        assert_eq!(
            record.source_fingerprint,
            *artifact.source_fingerprint().as_bytes()
        );
        assert_eq!(
            record.timeline_fingerprint,
            *artifact.timeline_fingerprint().as_bytes()
        );
        assert_eq!(record.source_language, *artifact.source_language());
        assert_eq!(record.target_language, *artifact.target_language());
        assert_eq!(record.provider, *artifact.provider());
        assert_eq!(record.pipeline_version, artifact.pipeline_version());
        assert_eq!(record.block_layout_version, artifact.block_layout_version());
        assert_eq!(record.glossary.as_deref(), Some("hunter-x-hunter"));
        assert_eq!(record.media_hash, artifact.media_hash());
        assert_eq!(record.created_at_unix_ms, 1_700_000_000_000);
        assert_eq!(record.document, *artifact.translated_document());
        assert_eq!(record.webvtt, artifact.webvtt());

        // The store mints the stored identity from the content; the
        // externally-assigned id is deliberately not carried into it.
        assert!(!record.webvtt.contains(artifact.id().as_str()));
    }

    #[test]
    fn to_record_maps_an_absent_glossary_to_none() {
        let document = document(50);
        let layout = layout(&document);
        let completed = completed_blocks_for(&document, &layout);

        let artifact = assemble(&document, &layout, &plan(), &completed, metadata())
            .expect("assembly succeeds");

        assert_eq!(artifact.to_record().glossary, None);
    }

    #[test]
    fn artifact_id_rejects_empty_too_long_and_disallowed_characters() {
        assert_eq!(ArtifactId::parse(""), Err(ArtifactIdError::Empty));
        assert_eq!(
            ArtifactId::parse(&"a".repeat(MAX_ARTIFACT_ID_LEN + 1)),
            Err(ArtifactIdError::TooLong {
                length: MAX_ARTIFACT_ID_LEN + 1
            })
        );
        assert_eq!(
            ArtifactId::parse("has space"),
            Err(ArtifactIdError::InvalidChar { position: 3 })
        );
        assert!(ArtifactId::parse("blake3:abc-123_.").is_ok());
    }
}
