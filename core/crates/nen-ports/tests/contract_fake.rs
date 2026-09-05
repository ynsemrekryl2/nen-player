//! The reference adapter against the shared contract kit (NEN-021 DoD #1).
//!
//! ADR-0011 Karar 4 makes the scenarios data, so this file drives the same
//! list NEN-022 will drive through FFI against the real libmpv adapter —
//! `docs/milestones/M3-macos-slice.md` requires both to pass *the same* kit.

use nen_ports::playback::contract::{applicable_count, run_all, scenarios, Applicability};
use nen_ports::playback::fake::{fake_inputs, fake_inputs_without_video, FakeEngine};
use nen_ports::playback::{Capabilities, Capability, PlaybackEngine};

#[test]
fn a_fully_capable_engine_passes_every_applicable_scenario() {
    let inputs = fake_inputs();
    let failures = run_all(FakeEngine::full, &inputs);
    assert!(failures.is_empty(), "{}", report(&failures));
}

#[test]
fn a_base_only_engine_passes_every_applicable_scenario() {
    // The other half of Karar 3: an engine that declares nothing optional must
    // still satisfy the whole mandatory base, and refuse the rest by type.
    let inputs = fake_inputs();
    let failures = run_all(FakeEngine::minimal, &inputs);
    assert!(failures.is_empty(), "{}", report(&failures));
}

#[test]
fn a_medium_with_no_video_passes_every_applicable_scenario() {
    // ADR-0038 Karar 1's other half. `None` is a state and not a failure, so
    // the whole kit must be green against a medium that has no picture — and
    // the geometry scenario must exercise the `None` branch rather than being
    // skipped. An audio file is a medium the product plays.
    let inputs = fake_inputs_without_video();
    let failures = run_all(FakeEngine::without_video, &inputs);
    assert!(failures.is_empty(), "{}", report(&failures));
}

#[test]
fn the_geometry_scenario_really_distinguishes_the_two_media() {
    // The pairing that makes the run above worth anything: each engine is
    // judged against the *other* medium's fixture and must go red. Without
    // this, a kit that ignored geometry entirely would pass both runs.
    let with_video = run_all(FakeEngine::full, &fake_inputs_without_video());
    assert!(
        !with_video.is_empty(),
        "an engine with a picture passed a fixture that declares none"
    );

    let without_video = run_all(FakeEngine::without_video, &fake_inputs());
    assert!(
        !without_video.is_empty(),
        "an engine with no picture passed a fixture that declares one"
    );
}

#[test]
fn every_partial_capability_set_passes() {
    // Capabilities are independent (ADR-0011 Karar 3): declaring one must not
    // change how another behaves. Sweeping every subset is what proves it.
    let inputs = fake_inputs();
    for bits in 0..(1u8 << Capability::ALL.len()) {
        let declared: Vec<Capability> = Capability::ALL
            .into_iter()
            .enumerate()
            .filter(|(index, _)| bits & (1 << index) != 0)
            .map(|(_, capability)| capability)
            .collect();
        let capabilities = Capabilities::new(declared.iter().copied());
        let failures = run_all(|| FakeEngine::new(capabilities), &inputs);
        assert!(
            failures.is_empty(),
            "capabilities {declared:?}:\n{}",
            report(&failures)
        );
    }
}

#[test]
fn the_run_is_not_vacuous() {
    // A kit that skipped everything would "pass". Both ends of the capability
    // range must actually execute a large number of scenarios, and every
    // scenario must apply to at least one of them.
    let total = scenarios().len();
    let full = applicable_count(Capabilities::ALL);
    let minimal = applicable_count(Capabilities::NONE);

    assert!(total >= 20, "the kit only has {total} scenarios");
    assert!(full >= 15, "only {full} scenarios apply to a full engine");
    assert!(minimal >= 15, "only {minimal} apply to a minimal engine");
    // Every scenario is either unconditional or gated on a capability, so the
    // two runs together must cover the whole list.
    assert_eq!(
        full + minimal,
        total
            + scenarios()
                .iter()
                .filter(|scenario| scenario.applies == Applicability::Always)
                .count(),
        "some scenario applies to neither a full nor a minimal engine"
    );
}

#[test]
fn every_capability_is_covered_in_both_directions() {
    // For each capability there must be a scenario for having it and one for
    // not having it — otherwise "typed error when unsupported" is untested for
    // that capability.
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
    assert_eq!(FakeEngine::full().capabilities(), Capabilities::ALL);
    assert_eq!(FakeEngine::minimal().capabilities(), Capabilities::NONE);
}

fn report(failures: &[nen_ports::playback::contract::Failure]) -> String {
    failures
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}
