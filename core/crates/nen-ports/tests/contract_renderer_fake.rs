//! The renderer contract kit, run against the reference implementation.
//!
//! The same shape as `contract_fake.rs` does for the playback port: every
//! capability subset is swept, and the run is proven not to be vacuous. What
//! this file cannot prove is that the kit is *strict* — that is
//! `contract_renderer_is_not_vacuous.rs`, which breaks a renderer on purpose.

use nen_domain::subtitle::{Cue, CueId, SubtitleDocument, TimeSpan};
use nen_ports::renderer::contract::{
    applicable_count, run_all, scenarios, Applicability, Failure, RenderInputs,
};
use nen_ports::renderer::{Capabilities, Capability, FakeRenderer};

fn inputs() -> RenderInputs {
    RenderInputs::new(SubtitleDocument::new(vec![Cue::new(
        CueId::new(1),
        TimeSpan::new(1_000, 2_000).expect("a valid span"),
        vec!["contract".to_string()],
    )]))
}

fn report(failures: &[Failure]) -> String {
    failures
        .iter()
        .map(Failure::render)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_reference_renderer_passes_every_applicable_scenario() {
    for capabilities in [Capabilities::ALL, Capabilities::NONE] {
        let failures = run_all(|| FakeRenderer::new(capabilities), &inputs());
        assert!(
            failures.is_empty(),
            "capabilities {capabilities:?}:\n{}",
            report(&failures)
        );
    }
}

#[test]
fn the_run_is_not_vacuous() {
    let total = scenarios().len();
    let full = applicable_count(Capabilities::ALL);
    let minimal = applicable_count(Capabilities::NONE);

    assert!(total >= 8, "the kit only has {total} scenarios");
    assert!(full >= 7, "only {full} scenarios apply to a full renderer");
    assert!(minimal >= 5, "only {minimal} apply to a minimal renderer");
    // Every scenario is either unconditional or gated on a capability, so the
    // two runs together must cover the whole list.
    assert_eq!(
        full + minimal,
        total
            + scenarios()
                .iter()
                .filter(|scenario| scenario.applies == Applicability::Always)
                .count(),
        "some scenario applies to neither a full nor a minimal renderer"
    );
}

#[test]
fn every_capability_is_covered_in_both_directions() {
    let all = scenarios();
    for capability in Capability::ALL {
        assert!(
            all.iter()
                .any(|s| s.applies == Applicability::WithCapability(capability)),
            "no scenario exercises {capability} when present"
        );
        assert!(
            all.iter()
                .any(|s| s.applies == Applicability::WithoutCapability(capability)),
            "no scenario exercises {capability} when absent"
        );
    }
}

#[test]
fn the_fake_declares_what_it_was_built_with() {
    use nen_ports::renderer::SubtitleRenderer;
    assert_eq!(FakeRenderer::full().capabilities(), Capabilities::ALL);
    assert_eq!(FakeRenderer::minimal().capabilities(), Capabilities::NONE);
}
