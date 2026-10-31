//! Rules the model declares (`rule`), and suspect traces (`reviewed:`).

use arclang::compiler::{elements, production_gate, source_edit, suspect, Compiler, CompilerConfig};

const MODEL: &str = r#"
model "Rules" {}

system_analysis "Needs" {
    requirement "Range" { id: "REQ-RANGE" description: "Drives 300 km" }
    requirement "Charge" { id: "REQ-CHARGE" description: "Charges in 30 min" }
}

physical_architecture "Hardware" {
    node "Battery" { id: "PN-BATT" mass: 450 kg voltage: 400 V }
    node "Charger" { id: "PN-CHG" mass: 12 kg }
    node "Cable" { id: "PN-CABLE" }
}

test_case "Range test" { id: "TC-RANGE" verifies: ["REQ-RANGE"] method: "test" verdict: "pass" }

// Two traces: one with a block, one without.
trace "PN-BATT" satisfies "REQ-RANGE" { rationale: "the pack holds the energy" }
trace "PN-CHG" satisfies "REQ-CHARGE"
"#;

fn compile(text: &str) -> Result<arclang::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_string(text).map_err(|e| e.to_string())
}

fn with_rules(rules: &str) -> String {
    format!("{MODEL}\n{rules}\n")
}

fn rule_lines(result: &arclang::CompilationResult) -> Vec<&str> {
    result.warnings.iter().map(String::as_str).filter(|w| w.starts_with("rule '")).collect()
}

#[test]
fn a_rule_finds_the_elements_that_do_not_hold_it() {
    let text = with_rules(r#"rule "Every node declares its mass" {
    id: "RULE-MASS"
    kind: "PhysicalNode"
    require: "has(mass)"
    severity: "blocker"
    message: "a node without a mass cannot be budgeted"
}"#);
    let result = compile(&text).unwrap();
    assert_eq!(rule_lines(&result), ["rule 'RULE-MASS' (blocker): 'PN-CABLE': a node without a mass cannot be budgeted"]);
    let report = production_gate::run_gate(&result.ast, &result.semantic_model, "ISO26262");
    let blocker = report.findings.iter().find(|f| f.check == "rule.RULE-MASS").expect("a blocker of the gate");
    assert!(matches!(blocker.severity, production_gate::Severity::Blocker));
    assert!(blocker.message.contains("'PN-CABLE'"), "{}", blocker.message);
    // The catalogue knows the line: SARIF, the workbench and VS Code place it on the element.
    let finding = arclang::compiler::rules::classify_warning(rule_lines(&result)[0]);
    assert_eq!((finding.rule, finding.element.as_deref()), ("rule.user", Some("PN-CABLE")));
}

#[test]
fn where_chooses_and_quantities_compare_in_si() {
    let text = with_rules(r#"rule "Light enough to carry" {
    id: "RULE-LIGHT"
    kind: "PhysicalNode"
    where: "has(mass) and not (name == \"Battery\")"
    require: "mass <= 0.01 t"
}
rule "Heavy nodes are powered" {
    id: "RULE-POWER"
    kind: ["PhysicalNode"]
    where: "mass > 100 kg"
    require: "voltage >= 48 V"
    severity: "info"
}"#);
    let result = compile(&text).unwrap();
    // The charger weighs 12 kg, over 10 kg; the battery is left out by where; the cable has no mass.
    assert_eq!(rule_lines(&result), ["rule 'RULE-LIGHT' (warning): 'PN-CHG': does not hold mass <= 0.01 t"]);
    let report = production_gate::run_gate(&result.ast, &result.semantic_model, "ISO26262");
    assert!(report.findings.iter().any(|f| f.check == "rule.RULE-LIGHT" && matches!(f.severity, production_gate::Severity::Warning)));
}

#[test]
fn a_rule_counts_traces_and_verifying_cases() {
    let text = with_rules(r#"rule "Every requirement is verified" {
    id: "RULE-VERIFIED"
    kind: "Requirement"
    require: "count_in(verifies) >= 1 and count_in(satisfies) == 1"
}
rule "A node satisfies something" {
    id: "RULE-PURPOSE"
    kind: "PhysicalNode"
    require: "count(satisfies) >= 1"
}"#);
    let result = compile(&text).unwrap();
    assert_eq!(
        rule_lines(&result),
        [
            "rule 'RULE-VERIFIED' (warning): 'REQ-CHARGE': does not hold count_in(verifies) >= 1 and count_in(satisfies) == 1",
            "rule 'RULE-PURPOSE' (warning): 'PN-CABLE': does not hold count(satisfies) >= 1",
        ]
    );
}

#[test]
fn a_rule_that_cannot_hold_is_an_error() {
    let cases = [
        (r#"rule "R" { kind: "Gadget" require: "has(mass)" }"#, "'Gadget' is no kind of element"),
        (r#"rule "R" { kind: "PhysicalNode" }"#, "it needs require"),
        (r#"rule "R" { kind: "PhysicalNode" require: "mass <=" }"#, "require \"mass <=\""),
        (r#"rule "R" { kind: "PhysicalNode" require: "has(mass)" color: "red" }"#, "'color' is not one of"),
        (r#"rule "R" { kind: "PhysicalNode" require: "has(mass)" severity: "fatal" }"#, "severity 'fatal'"),
        (r#"rule "R" { kind: "PhysicalNode" require: "mass <= 5 s" }"#, "mass is mass and"),
        (r#"rule "R" { kind: "PhysicalNode" require: "name < \"B\"" }"#, "texts compare with == and != only"),
    ];
    for (rule, expected) in cases {
        let error = compile(&with_rules(rule)).err().unwrap_or_default();
        assert!(error.contains(expected), "{rule}: {error}");
    }
}

#[test]
fn a_reviewed_trace_turns_suspect_when_an_end_changes() {
    // Review both traces through the source edit, as `arclang trace --review` does.
    let result = compile(MODEL).unwrap();
    let graph = elements::build(&result.ast, &result.semantic_model);
    let mut text = MODEL.to_string();
    for trace in &result.ast.traces {
        let stamp = suspect::stamp(&graph, trace).unwrap();
        text = source_edit::set_trace_attribute(&text, &trace.from, &trace.trace_type, &trace.to, "reviewed", &source_edit::string_literal(&stamp)).unwrap();
    }
    assert!(text.contains("// Two traces: one with a block, one without."), "comments are kept");
    assert!(text.contains("trace \"PN-BATT\" satisfies \"REQ-RANGE\" { rationale: \"the pack holds the energy\" reviewed: \""), "{text}");
    assert!(text.contains("trace \"PN-CHG\" satisfies \"REQ-CHARGE\" { reviewed: \""), "a trace without a block gets one: {text}");
    let reviewed = compile(&text).unwrap();
    assert!(!reviewed.warnings.iter().any(|w| w.starts_with("suspect:")), "{:?}", reviewed.warnings);

    // The requirement's text changes: the trace to it is suspect, the other is not.
    let changed = text.replace("Drives 300 km", "Drives 320 km");
    let result = compile(&changed).unwrap();
    let suspects: Vec<&String> = result.warnings.iter().filter(|w| w.starts_with("suspect:")).collect();
    assert_eq!(suspects.len(), 1, "{suspects:?}");
    assert!(suspects[0].starts_with("suspect: trace 'PN-BATT' satisfies 'REQ-RANGE': 'REQ-RANGE' changed since the trace was reviewed"), "{}", suspects[0]);
    let report = production_gate::run_gate(&result.ast, &result.semantic_model, "ISO26262");
    assert!(report.findings.iter().any(|f| f.check == "traceability.suspect" && matches!(f.severity, production_gate::Severity::Blocker)));
    assert!(!report.passed);
    let finding = arclang::compiler::rules::classify_warning(suspects[0]);
    assert_eq!((finding.rule, finding.element.as_deref()), ("trace.suspect", Some("PN-BATT")));

    // Both ends change.
    let both = changed.replace("mass: 450 kg", "mass: 470 kg");
    let result = compile(&both).unwrap();
    assert!(result.warnings.iter().any(|w| w.contains("'PN-BATT' and 'REQ-RANGE' changed")), "{:?}", result.warnings);

    // Reviewed again: no longer suspect.
    let graph = elements::build(&result.ast, &result.semantic_model);
    let trace = result.ast.traces.iter().find(|t| t.to == "REQ-RANGE").unwrap();
    let again = source_edit::set_trace_attribute(&both, "PN-BATT", "satisfies", "REQ-RANGE", "reviewed", &source_edit::string_literal(&suspect::stamp(&graph, trace).unwrap())).unwrap();
    assert!(!compile(&again).unwrap().warnings.iter().any(|w| w.starts_with("suspect:")));
}

#[test]
fn a_hand_written_stamp_is_refused() {
    let text = MODEL.replace("trace \"PN-CHG\" satisfies \"REQ-CHARGE\"", "trace \"PN-CHG\" satisfies \"REQ-CHARGE\" { reviewed: \"yes\" }");
    let error = compile(&text).err().unwrap_or_default();
    assert!(error.contains("reviewed: \"yes\" is not the two fingerprints"), "{error}");
}

#[test]
fn the_trace_command_reviews_and_lists_suspects() {
    let dir = tempfile::tempdir().unwrap();
    let model = dir.path().join("model.arc");
    std::fs::write(&model, MODEL).unwrap();
    let run = |args: &[&str]| {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_arclang")).arg("trace").arg(&model).args(args).output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).to_string()
    };
    assert!(run(&["--suspects"]).starts_with("0 of 2 trace(s) reviewed, 0 suspect"));
    let reviewed = run(&["--review"]);
    assert!(reviewed.contains("Reviewed 2 trace(s)") && reviewed.contains("✓ PN-BATT satisfies REQ-RANGE"), "{reviewed}");
    assert!(run(&["--review"]).contains("Reviewed 0 trace(s) in") , "nothing changed: nothing written");
    let text = std::fs::read_to_string(&model).unwrap().replace("Charges in 30 min", "Charges in 20 min");
    std::fs::write(&model, text).unwrap();
    let suspects = run(&["--suspects"]);
    assert!(suspects.contains("2 of 2 trace(s) reviewed, 1 suspect") && suspects.contains("'REQ-CHARGE' changed"), "{suspects}");
    assert!(suspects.contains(&format!("arclang trace {} --review --from PN-CHG --to REQ-CHARGE", model.display())), "{suspects}");
    // Reviewing one trace, by the name of its end.
    assert!(run(&["--review", "--from", "Charger", "--to", "REQ-CHARGE"]).contains("Reviewed 1 trace(s)"));
    assert!(run(&["--suspects"]).contains("2 of 2 trace(s) reviewed, 0 suspect"));
}

#[test]
fn a_rule_judges_links_as_it_judges_elements() {
    let text = format!(
        "{MODEL}\nphysical_architecture \"Wiring\" {{\n    node \"A\" {{ id: \"PN-A\" mass: 1 kg }}\n    node \"B\" {{ id: \"PN-B\" mass: 1 kg }}\n    physical_link \"Bus\" {{ id: \"PL-BUS\" from: \"PN-A\" to: \"PN-B\" protocol: \"CAN\" }}\n    physical_link \"Spare\" {{ id: \"PL-SPARE\" from: \"PN-A\" to: \"PN-B\" }}\n}}\nrule \"Links name their protocol\" {{ id: \"R-PROTO\" kind: \"PhysicalLink\" require: \"has(protocol)\" }}\n"
    );
    let result = compile(&text).unwrap();
    assert_eq!(rule_lines(&result), ["rule 'R-PROTO' (warning): 'PL-SPARE': does not hold has(protocol)"], "the parser's default is no protocol");
}
