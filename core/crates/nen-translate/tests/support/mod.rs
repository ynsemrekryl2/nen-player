//! Shared helpers for the `nen-translate` fixture-driven tests.
//!
//! Lives in a subdirectory so Cargo treats it as a module of each test crate
//! rather than as a test target of its own. Mirrors
//! `nen-subtitle/tests/support/mod.rs`'s shape; not shared as a library
//! because these are test-only helpers (Kural 5 — no unrelated refactor to
//! introduce a shared dev-dependency crate for this).
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use nen_domain::source::LanguageTag;
use nen_domain::subtitle::SubtitleDocument;
use nen_ports::translation::{
    TranslatedCue, TranslationCall, TranslationProvider, TranslationProviderError,
    TranslationProviderIdentity, TranslationRequest, TranslationResponse,
};
use nen_subtitle::srt;
use nen_translate::artifact::{
    self, ArtifactError, ArtifactId, ArtifactMetadata, ArtifactTimestamp, GlossaryIdentity,
    ValidatedSubtitleArtifact,
};
use nen_translate::blocks::BlockLayout;
use nen_translate::checkpoint::{translate_checkpointed, BlockCheckpoints, TranslationPlan};
use nen_translate::context::DocumentContext;

/// Root of the repository's translation-block fixture corpus.
pub fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/subtitles/blocks")
}

/// Every `*.srt` file of the corpus, sorted so failures are reported in a
/// stable order across machines.
pub fn srt_files() -> Vec<PathBuf> {
    let dir = corpus_dir();
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|err| panic!("cannot read {}: {err}", dir.display()));

    let mut files: Vec<PathBuf> = entries
        .map(|entry| entry.expect("cannot read directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "srt"))
        .collect();
    files.sort();
    files
}

pub fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()))
}

pub fn name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

pub fn parse(path: &Path) -> SubtitleDocument {
    let input = read(path);
    srt::parse(&input)
        .unwrap_or_else(|err| panic!("{}: expected a valid document, got {err}", name(path)))
}

/// Canonical, deterministic rendering of a document's default-config block
/// layout plus its whole-document context — the golden snapshot format for
/// the block-layout corpus.
pub fn render_golden(document: &SubtitleDocument) -> String {
    let layout = BlockLayout::of(document, Default::default())
        .unwrap_or_else(|err| panic!("expected a valid layout, got {err}"));
    let context = DocumentContext::of(document);

    let mut out = String::new();
    out.push_str(&format!(
        "version\t{}\nblock_size\t{}\noverlap\t{}\ncue_count\t{}\nblocks\t{}\n",
        layout.version(),
        layout.config().block_size(),
        layout.config().overlap(),
        layout.cue_count(),
        layout.blocks().len(),
    ));
    for block in layout.blocks() {
        let window = block.context_positions();
        let output = block.output_positions();
        out.push_str(&format!(
            "block\t{}\twindow\t{}..{}\toutput\t{}..{}\n",
            block.index(),
            window.start,
            window.end,
            output.start,
            output.end,
        ));
    }

    out.push_str(&format!("context_terms\t{}\n", context.terms().len()));
    for term in context.terms() {
        out.push_str(&format!("term\t{}\t{}\n", term.term(), term.occurrences()));
    }

    out
}

/// Deterministic translation provider for artifact tests: echoes the source
/// text of every requested cue, tagged with the target language, so a run's
/// output is a pure function of `document`.
pub struct EchoTranslationProvider;

impl TranslationProvider for EchoTranslationProvider {
    fn identity(&self) -> TranslationProviderIdentity {
        TranslationProviderIdentity::new("nen-test", "echo-golden")
            .expect("static test identity is valid")
    }

    fn translate(
        &self,
        request: &TranslationRequest,
        call: &TranslationCall,
    ) -> Result<TranslationResponse, TranslationProviderError> {
        let cues = request
            .output_cue_ids
            .iter()
            .map(|cue_id| {
                let source = request
                    .context_cues
                    .iter()
                    .find(|cue| cue.cue_id == *cue_id)
                    .expect("output cue is always in the block's own context window");
                TranslatedCue {
                    cue_id: *cue_id,
                    text: format!("[{}] {}", request.target_language.as_str(), source.text),
                }
            })
            .collect();
        call.finish(TranslationResponse { cues })
    }
}

/// The en→tr plan every artifact-fixture test shares. Fixed so a golden
/// snapshot and its assertions stay stable across runs.
pub fn plan() -> TranslationPlan {
    TranslationPlan {
        source_language: LanguageTag::parse("en").expect("language"),
        target_language: LanguageTag::parse("tr").expect("language"),
        context_terms: Vec::new(),
    }
}

/// Runs [`EchoTranslationProvider`] to completion over `document` and
/// `layout`, returning the resulting [`nen_translate::checkpoint::CompletedBlocks`].
/// Shared by the negative tests, which pair this with a *different* document
/// to prove `assemble` re-checks rather than trusts it.
pub fn build_completed(
    document: &SubtitleDocument,
    layout: &BlockLayout,
) -> nen_translate::checkpoint::CompletedBlocks {
    let mut checkpoints = BlockCheckpoints::for_layout(layout);
    translate_checkpointed(
        &EchoTranslationProvider,
        document,
        layout,
        &plan(),
        &TranslationCall::without_progress(),
        &mut checkpoints,
    )
    .unwrap_or_else(|err| panic!("expected the run to complete, got {err}"));
    checkpoints
        .into_completed()
        .unwrap_or_else(|err| panic!("expected every block to be checkpointed, got {err}"))
}

/// Runs [`EchoTranslationProvider`] to completion over `document`'s default
/// block layout and assembles the resulting artifact — the fixture pipeline
/// shared by the golden and guard tests.
pub fn build_artifact(document: &SubtitleDocument) -> ValidatedSubtitleArtifact {
    let layout = BlockLayout::of(document, Default::default())
        .unwrap_or_else(|err| panic!("expected a valid layout, got {err}"));
    let completed = build_completed(document, &layout);

    artifact::assemble(document, &layout, &plan(), &completed, artifact_metadata())
        .unwrap_or_else(|err| panic!("expected assembly to succeed, got {err}"))
}

/// Fixed, deterministic metadata so a golden snapshot never depends on wall
/// clock time or an externally-minted ID.
pub fn artifact_metadata() -> ArtifactMetadata {
    ArtifactMetadata {
        id: ArtifactId::parse("golden-artifact").expect("valid artifact id"),
        provider: EchoTranslationProvider.identity(),
        glossary: GlossaryIdentity::none(),
        media_hash: None,
        created_at: ArtifactTimestamp::from_unix_ms(0),
    }
}

/// Canonical, deterministic rendering of a document's assembled artifact —
/// the golden snapshot format for the artifact corpus (`NEN-094`).
pub fn render_artifact_golden(document: &SubtitleDocument) -> String {
    let artifact = build_artifact(document);

    let mut out = String::new();
    out.push_str(&format!(
        "source_fingerprint\t{}\ntimeline_fingerprint\t{}\nsource_language\t{}\ntarget_language\t{}\nprovider\t{}\nmodel\t{}\npipeline_version\t{}\nblock_layout_version\t{}\ncue_count\t{}\n",
        artifact.source_fingerprint(),
        artifact.timeline_fingerprint(),
        artifact.source_language().as_str(),
        artifact.target_language().as_str(),
        artifact.provider().provider(),
        artifact.provider().model(),
        artifact.pipeline_version(),
        artifact.block_layout_version(),
        artifact.translated_document().len(),
    ));
    out.push_str("--- webvtt ---\n");
    out.push_str(artifact.webvtt());

    out
}

/// Attempts to assemble an artifact and returns the [`ArtifactError`]
/// instead of panicking — for the negative tests that expect assembly to be
/// refused.
pub fn try_build_artifact(
    document: &SubtitleDocument,
    layout: &BlockLayout,
    completed: &nen_translate::checkpoint::CompletedBlocks,
) -> Result<ValidatedSubtitleArtifact, ArtifactError> {
    artifact::assemble(document, layout, &plan(), completed, artifact_metadata())
}
