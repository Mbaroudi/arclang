//! `spec/RULES.md` is GENERATED from the rule catalog declared in code.
//! This test fails when the file and the code disagree, so the published
//! catalog never drifts from what `check` reports.

use std::path::Path;

#[test]
fn rules_spec_is_up_to_date() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("spec/RULES.md");
    let on_disk = std::fs::read_to_string(&path).expect("spec/RULES.md exists");
    let generated = arclang::compiler::rules::to_markdown();
    assert_eq!(
        on_disk, generated,
        "spec/RULES.md is out of date with src/compiler/rules.rs.\n\
         Regenerate it:  cargo run -q -- rules > spec/RULES.md"
    );
}

#[test]
fn every_gate_check_named_in_the_page_is_a_check_the_gate_runs() {
    let page = arclang::compiler::rules::to_markdown();
    let gate_source = include_str!("../src/compiler/production_gate.rs");
    for check in [
        "requirements.presence", "requirements.satisfaction", "requirements.verification", "verification.dangling",
        "safety.hara", "safety.asil", "safety.dal", "safety.mitigation", "timing.latency", "timing.budget",
        "timing.units", "icd.completeness", "constraints.violated", "metamodel.types", "methodology",
    ] {
        assert!(page.contains(&format!("`{check}`")), "{check} missing from the page");
        assert!(gate_source.contains(&format!("\"{check}\"")), "{check} is not a check the gate runs");
    }
}
