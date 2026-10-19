//! Verification and analysis cases tied to requirements: a `test_case`
//! `asserts:` constraints, and its verdict is theirs (pass when they all
//! hold, fail otherwise); a test run elsewhere says its `verdict:`. A
//! requirement with a failing case is not verified, for the production
//! gate. In SysML v2, a verification case asserting the constraints.

use arclang::compiler::production_gate::run_gate;
use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"
system_analysis "SA" {
    requirement "Memory" { id: "REQ-MEM" description: "Enough memory for the model" }
}

logical_architecture "Logical" {
    component "Accelerator" { id: "LC-ACC" ram: 96 GB }
    component "HBM stack" { id: "LC-HBM" ram: 24 GB }
}

trace "LC-ACC" satisfies "REQ-MEM"

constraint "HBM capacity" {
    id: "CST-HBM"
    assert: 6 * "LC-HBM".ram >= "LC-ACC".ram
}

test_case "Memory sizing" {
    id: "TC-MEM"
    verifies: ["REQ-MEM"]
    method: "analysis"
    asserts: ["CST-HBM"]
}
"#;

fn compile(source: &str) -> Result<arclang::compiler::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_string(source).map_err(|e| e.to_string())
}

fn verification_findings(source: &str) -> Vec<String> {
    let result = compile(source).unwrap();
    let gate = run_gate(&result.ast, &result.semantic_model, "iso26262");
    gate.findings.iter().filter(|f| f.check.starts_with("requirements.verification") || f.check.starts_with("verification")).map(|f| f.message.clone()).collect()
}

#[test]
fn an_analysis_whose_constraints_hold_verifies_its_requirement() {
    let result = compile(MODEL).unwrap();
    assert!(result.warnings.iter().all(|w| !w.contains("verification:")), "{:?}", result.warnings);
    let gate = run_gate(&result.ast, &result.semantic_model, "iso26262");
    assert_eq!(gate.requirements_verified, 1);
    assert!(verification_findings(MODEL).is_empty(), "{:?}", verification_findings(MODEL));
}

#[test]
fn an_analysis_whose_constraint_fails_leaves_its_requirement_unverified() {
    let failing = MODEL.replace("ram: 24 GB", "ram: 12 GB");
    let result = compile(&failing).unwrap();
    assert!(result.warnings.iter().any(|w| w.contains("verification: test_case 'Memory sizing' fails: constraint 'HBM capacity' does not hold")), "{:?}", result.warnings);
    let gate = run_gate(&result.ast, &result.semantic_model, "iso26262");
    assert_eq!(gate.requirements_verified, 0);
    let findings = verification_findings(&failing);
    assert!(findings.iter().any(|f| f == "requirement 'REQ-MEM': verification cases fail (Memory sizing)"), "{findings:?}");
}

#[test]
fn a_declared_failing_verdict_leaves_its_requirement_unverified() {
    let failing = MODEL.replace("method: \"analysis\"\n    asserts: [\"CST-HBM\"]", "method: \"test\"\n    verdict: \"fail\"");
    let findings = verification_findings(&failing);
    assert!(findings.iter().any(|f| f == "requirement 'REQ-MEM': verification cases fail (Memory sizing)"), "{findings:?}");
}

#[test]
fn asserting_an_unknown_constraint_is_an_error() {
    let error = compile(&MODEL.replace("asserts: [\"CST-HBM\"]", "asserts: [\"CST-NONE\"]")).err().unwrap_or_default();
    assert!(error.contains("test_case 'Memory sizing' asserts 'CST-NONE', which is no constraint"), "{error}");
}

#[test]
fn a_verdict_outside_the_sysml_verdicts_is_an_error() {
    let error = compile(&MODEL.replace("asserts: [\"CST-HBM\"]", "verdict: \"ok\"")).err().unwrap_or_default();
    assert!(error.contains("test_case 'Memory sizing': verdict 'ok' is not one of pass|fail|inconclusive|error"), "{error}");
}

#[test]
fn a_verdict_given_beside_asserted_constraints_is_an_error() {
    let error = compile(&MODEL.replace("asserts: [\"CST-HBM\"]", "asserts: [\"CST-HBM\"]\n    verdict: \"pass\"")).err().unwrap_or_default();
    assert!(error.contains("test_case 'Memory sizing': its verdict comes from the constraints it asserts, it is not given"), "{error}");
}

#[test]
fn an_analysis_case_goes_through_sysml_v2_and_back() {
    let result = compile(MODEL).unwrap();
    let text = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);
    assert!(text.contains("verification <'TC-MEM'> 'Memory sizing'"), "{text}");
    assert!(text.contains("assert 'CST-HBM';"), "{text}");
    let back = arclang::compiler::sysml_import::import_text(&text).unwrap_or_else(|f| panic!("{}\n{text}", f.message)).source;
    assert!(back.contains("asserts: [\"CST-HBM\"]"), "{back}");
    assert!(back.contains("verifies: [\"REQ-MEM\"]"), "{back}");
    compile(&back).unwrap_or_else(|e| panic!("{e}\n{back}"));
}

#[test]
fn a_passing_case_does_not_hide_a_failing_one() {
    let source = MODEL.replace("ram: 24 GB", "ram: 12 GB") + "\ntest_case \"Memory review\" { id: \"TC-REVIEW\" verifies: [\"REQ-MEM\"] method: \"inspection\" verdict: \"pass\" }\n";
    let findings = verification_findings(&source);
    assert!(findings.iter().any(|f| f == "requirement 'REQ-MEM': verification cases fail (Memory sizing)"), "{findings:?}");
}
