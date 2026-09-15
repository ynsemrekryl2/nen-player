use nen_ports::translation::{PROMPT_VERSION, SCHEMA_VERSION};
use nen_providers::{openai, openrouter};
use serde_json::Value;

fn value(input: &str) -> Value {
    serde_json::from_str(input).expect("golden JSON")
}

fn assert_same_semantics(openai_body: &str, openrouter_body: &str) {
    let openai = value(openai_body);
    let openrouter = value(openrouter_body);

    assert_eq!(openai["instructions"], openrouter["messages"][0]["content"]);
    assert_eq!(openai["input"], openrouter["messages"][1]["content"]);
    assert_eq!(
        openai["text"]["format"]["name"],
        openrouter["response_format"]["json_schema"]["name"]
    );
    assert_eq!(
        openai["text"]["format"]["strict"],
        openrouter["response_format"]["json_schema"]["strict"]
    );
    assert_eq!(
        openai["text"]["format"]["schema"],
        openrouter["response_format"]["json_schema"]["schema"]
    );
}

#[test]
fn both_real_providers_share_analysis_initial_targeted_and_full_retry_semantics() {
    for (openai, openrouter) in [
        (
            include_str!("../../../../fixtures/providers/openai/request-analysis.golden"),
            include_str!("../../../../fixtures/providers/openrouter/request-analysis.golden"),
        ),
        (
            include_str!("../../../../fixtures/providers/openai/request-success.golden"),
            include_str!("../../../../fixtures/providers/openrouter/request-success.golden"),
        ),
        (
            include_str!("../../../../fixtures/providers/openai/request-targeted-repair.golden"),
            include_str!(
                "../../../../fixtures/providers/openrouter/request-targeted-repair.golden"
            ),
        ),
        (
            include_str!("../../../../fixtures/providers/openai/request-full-retry.golden"),
            include_str!("../../../../fixtures/providers/openrouter/request-full-retry.golden"),
        ),
    ] {
        assert_same_semantics(openai, openrouter);
    }
}

#[test]
fn provider_exports_use_the_port_canonical_contract_versions() {
    assert_eq!(openai::PROMPT_VERSION, PROMPT_VERSION);
    assert_eq!(openai::SCHEMA_VERSION, SCHEMA_VERSION);
    assert_eq!(openrouter::PROMPT_VERSION, PROMPT_VERSION);
    assert_eq!(openrouter::SCHEMA_VERSION, SCHEMA_VERSION);
}
