//! Golden-file test for the SysML v2 export of the flagship model.
//!
//! The export is deterministic by policy (`docs/VERSIONING.md`): a diff in
//! this file always means the exporter or the model changed. Regenerate
//! with `UPDATE_GOLDEN=1 cargo test --test sysmlv2_golden`.

use arclang::compiler::sysmlv2_generator::generate_sysmlv2;
use arclang::compiler::{Compiler, CompilerConfig};
use std::path::Path;

#[test]
fn flagship_sysmlv2_export_matches_golden() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = root.join("examples/complete_emergency_braking_simple.arc");
    let golden = root.join("tests/fixtures/sysmlv2/complete_emergency_braking_simple.sysml");

    let result = Compiler::new(CompilerConfig::default()).compile_file(&source).expect("flagship compiles");
    let generated = generate_sysmlv2(&result.semantic_model, &result.ast);

    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&golden, &generated).expect("write golden");
        return;
    }
    let expected = std::fs::read_to_string(&golden).unwrap_or_default();
    assert!(
        expected == generated,
        "SysML v2 export of the flagship model changed. Review the diff, then regenerate with\n  UPDATE_GOLDEN=1 cargo test --test sysmlv2_golden"
    );
    // The flagship export must be rich, not a skeleton.
    assert!(generated.contains("port def"), "ports exported");
    assert!(generated.contains("allocate "), "deployments exported");
    assert!(generated.contains("verification def"), "test cases exported");
    assert!(generated.contains("DurationValue"), "typed latencies exported");
    assert!(generated.contains("connect "), "exchanges exported");
    assert!(!generated.contains("unresolved"), "every endpoint and trace of the flagship resolves");
}
