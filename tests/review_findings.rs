//! Defects found by the review of 2026-11-22, each held by the input that showed it: a gate that
//! passed without what it says it asks for, a text that ended the process, and model text that
//! reached generated code as code.

use arclang::compiler::production_gate::{run_gate, GateReport};
use arclang::compiler::{Compiler, CompilerConfig};

fn compile(text: &str) -> Result<arclang::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_string(text).map_err(|e| e.to_string())
}

fn gate(text: &str) -> GateReport {
    let result = compile(text).unwrap();
    run_gate(&result.ast, &result.semantic_model, "iso26262")
}

fn says(report: &GateReport, check: &str, words: &str) -> bool {
    report.findings.iter().any(|f| f.check == check && f.message.contains(words))
}

const SATISFIED: &str = r#"model T {}
system_analysis "SA" {
    requirement "Range" { id: "REQ-1" description: "range" }
}
architecture logical { component "C1" { id: "LC-1" function "X" { id: "LF-1" } } }
trace "LC-1" satisfies "REQ-1"
test_case "TC-1" { id: "TC-1" verifies: ["REQ-1"] method: "test" verdict: "pass" }
"#;

#[test]
fn two_requirements_that_satisfy_each_other_are_satisfied_by_no_element() {
    let report = gate(
        r#"model T {}
system_analysis "SA" {
    requirement "Range" { id: "REQ-1" description: "range" }
    requirement "Other" { id: "REQ-2" description: "x" }
}
trace "REQ-2" satisfies "REQ-1"
trace "REQ-1" satisfies "REQ-2"
test_case "TC-1" { id: "TC-1" verifies: ["REQ-1", "REQ-2"] method: "test" verdict: "pass" }
"#,
    );
    assert_eq!(report.requirements_satisfied, 0, "{:?}", report.findings);
    assert!(!report.passed);
    assert!(says(&report, "requirements.satisfaction", "REQ-1") && says(&report, "requirements.satisfaction", "REQ-2"));
}

#[test]
fn an_element_of_the_architecture_still_satisfies() {
    let report = gate(SATISFIED);
    assert_eq!(report.requirements_satisfied, 1, "{:?}", report.findings);
}

#[test]
fn a_fault_tree_beyond_its_budget_blocks_the_gate() {
    let hazard = |fit: &str| {
        format!(
            "{SATISFIED}safety_analysis {{\n    hazard \"H1\" {{ id: \"HAZ-1\" severity: \"S3\" exposure: \"E4\" controllability: \"C3\" asil: \"ASIL-D\" mitigated_by: [\"REQ-1\"] }}\n    fmea \"FM\" {{ id: \"FM-1\" element: \"LC-1\" leads_to: [\"HAZ-1\"] fit: {fit} }}\n}}\n"
        )
    };
    let beyond = gate(&hazard("5000"));
    assert!(says(&beyond, "safety.fault_tree", "beyond the 10 FIT"), "{:?}", beyond.findings);
    assert!(!beyond.passed);
    let within = gate(&hazard("5"));
    assert!(!within.findings.iter().any(|f| f.check == "safety.fault_tree"), "{:?}", within.findings);
}

#[test]
fn an_empty_list_of_mitigations_mitigates_nothing() {
    let report = gate(&format!(
        "{SATISFIED}safety_analysis {{\n    hazard \"H1\" {{ severity: \"S3\" exposure: \"E4\" controllability: \"C3\" asil: \"ASIL_D\" mitigated_by: [] }}\n}}\n"
    ));
    assert!(says(&report, "safety.mitigation", "has no mitigation"), "{:?}", report.findings);
}

#[test]
fn a_negative_latency_does_not_hide_another_in_a_chain() {
    let report = gate(
        r#"model T {}
system_analysis "SA" {
    requirement "REQ-1" { description: "fast" }
    function "A" { id: "F-A" latency: -5000 ms }
    function "B" { id: "F-B" latency: 5050 ms }
    functional_chain "Chain" { id: "FC-1" involves: ["F-A", "F-B"] latency_budget: 100 ms }
}
architecture logical { component "C1" { id: "LC-1" function "X" { id: "LF-1" } } }
trace "LC-1" satisfies "REQ-1"
test_case "TC-1" { verifies: ["REQ-1"] method: "test" verdict: "pass" }
"#,
    );
    assert!(!report.passed, "{:?}", report.findings);
    assert!(report.findings.iter().any(|f| f.message.contains("not a time that can be taken")), "{:?}", report.findings);
}

#[test]
fn a_negated_comparison_does_not_hold_for_a_value_the_element_lacks() {
    let model = |require: &str| {
        format!(
            "model T {{}}\narchitecture physical {{\n  node \"Heavy\" {{ id: \"PN-1\" mass: 80 kg }}\n  node \"Light\" {{ id: \"PN-3\" mass: 8 kg }}\n  node \"Unknown\" {{ id: \"PN-2\" }}\n}}\nrule \"Mass cap\" {{ id: \"R1\" kind: \"PhysicalNode\" require: \"{require}\" severity: \"warning\" message: \"too heavy or unknown\" }}\n"
        )
    };
    let flagged = |require: &str| -> Vec<&'static str> {
        let warnings = compile(&model(require)).unwrap().warnings.join("\n");
        [("PN-1", "Heavy"), ("PN-3", "Light"), ("PN-2", "Unknown")].into_iter().filter(|(id, _)| warnings.lines().any(|l| l.contains("R1") && l.contains(&format!("'{id}'")))).map(|(_, name)| name).collect()
    };
    // Written one way or the other, the rule says the same of each node.
    assert_eq!(flagged("mass <= 50 kg"), ["Heavy", "Unknown"]);
    assert_eq!(flagged("not (mass > 50 kg)"), ["Heavy", "Unknown"]);
    // `has` is known for every element, and its negation too.
    assert_eq!(flagged("not has(mass) or mass <= 50 kg"), ["Heavy"]);
}

#[test]
fn one_variant_named_by_its_id_and_by_its_name_is_one_choice() {
    let model = |selects: &str| {
        format!(
            "model T {{}}\narchitecture logical {{\n  component \"Engine\" {{ id: \"LC-E\" variation: true\n     component \"Four cylinders\" {{ id: \"LC-4CYL\" variant: true }}\n     component \"Six cylinders\" {{ id: \"LC-6CYL\" variant: true }}\n  }}\n}}\nconfiguration \"City\" {{ id: \"CFG-CITY\" selects: [{selects}] }}\n"
        )
    };
    compile(&model("\"LC-4CYL\", \"Four cylinders\"")).expect("one variant, named twice");
    let error = compile(&model("\"LC-4CYL\", \"Six cylinders\"")).unwrap_err();
    assert!(error.contains("two variants of variation"), "{error}");
}

/// A text nested thousands deep is refused with an error, on a thread as small as a server's.
fn on_a_small_stack<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(work).unwrap().join().expect("the text is refused, the thread does not die")
}

#[test]
fn a_model_nested_too_deep_is_an_error_not_the_end_of_the_process() {
    for (open, close) in [("[", "]"), ("(", ")")] {
        let text = format!("model \"A\" {{ version: \"1.0\" x: {}1{} }}", open.repeat(20_000), close.repeat(20_000));
        let error = on_a_small_stack(move || compile(&text)).unwrap_err();
        assert!(error.contains("Nested deeper than"), "{error}");
    }
    let components: String = (0..5_000).map(|i| format!("component \"C{i}\" {{ id: \"LC-{i}\" ")).collect();
    let text = format!("model T {{}}\narchitecture logical {{ {components}{} }}", "}".repeat(5_000));
    let error = on_a_small_stack(move || compile(&text)).unwrap_err();
    assert!(error.contains("Nested deeper than"), "{error}");
}

#[test]
fn a_model_nested_as_deep_as_models_are_compiles() {
    let components: String = (0..40).map(|i| format!("component \"C{i}\" {{ id: \"LC-{i}\" ")).collect();
    let text = format!("model T {{}}\narchitecture logical {{ {components}{} }}", "}".repeat(40));
    on_a_small_stack(move || compile(&text)).expect("forty levels");
}

#[test]
fn a_rule_nested_too_deep_is_an_error() {
    for require in [format!("{}true{}", "(".repeat(50_000), ")".repeat(50_000)), format!("{}true", "not ".repeat(50_000))] {
        let text = format!("model T {{}}\narchitecture physical {{ node \"N\" {{ id: \"PN-1\" }} }}\nrule \"R\" {{ id: \"R1\" kind: \"PhysicalNode\" require: \"{require}\" }}\n");
        let error = on_a_small_stack(move || compile(&text)).unwrap_err();
        assert!(error.contains("nested deeper than"), "{error}");
    }
}

#[test]
fn a_sysml_text_nested_too_deep_is_an_error() {
    let text = format!("package P {{ {}{} }}", "part a { ".repeat(5_000), "}".repeat(5_000));
    let error = on_a_small_stack(move || arclang::compiler::sysml_parser::parse(&text).map(|_| ())).unwrap_err();
    assert!(error.contains("nested deeper than"), "{error}");
}

#[test]
fn a_capella_file_nested_too_deep_is_an_error() {
    let text = format!("<xmi:XMI>{}{}</xmi:XMI>", "<a>".repeat(50_000), "</a>".repeat(50_000));
    let error = on_a_small_stack(move || arclang::compiler::capella_xmi::parse(&text).map(|_| ())).unwrap_err();
    assert!(error.contains("nested deeper than"), "{error}");
}

const SWEPT: &str = r#"model T {}
system_analysis "SA" { requirement "Range" { id: "REQ-1" description: "range" } }
test_case "TC-1" {
    id: "TC-1" verifies: ["REQ-1"] method: "analysis"
    simulate: { model: "P.M" stop_time: 10 s SOURCES SWEEP }
    expects: ["REQ-1: final(x) >= 1"]
}
"#;

#[test]
fn a_swept_variable_is_a_path_and_nothing_more() {
    let model = |sweep: &str| SWEPT.replace("SOURCES", "").replace("SWEEP", &format!("sweep: [\"{sweep}\"]"));
    compile(&model("cells[3].T = 1, 2")).expect("a subscript of whole numbers");
    for hostile in ["a.b[1\\\"); system(\\\"touch x\\\"); (\\\"] = 1, 2", "a.b[1 = 1, 2", "a[x] = 1, 2", "a[] = 1, 2"] {
        assert!(compile(&model(hostile)).is_err(), "{hostile} is no path to a variable");
    }
}

#[test]
fn a_number_that_is_not_finite_is_no_level_and_no_law() {
    let sweep = |levels: &str| SWEPT.replace("SOURCES", "").replace("SWEEP", &format!("sweep: [\"a.b = {levels}\"]"));
    assert!(compile(&sweep("1, nan")).unwrap_err().contains("not a finite number"));
    let uncertain = |law: &str| SWEPT.replace("SOURCES", "").replace("SWEEP", &format!("uncertain: [\"a.b ~ {law}\"] samples: 10 probability: 0.9"));
    for law in ["normal(nan, 1) in [0, 1]", "normal(0, nan)", "uniform(0, inf)", "normal(0, 1) in [nan, 2]"] {
        let error = compile(&uncertain(law)).unwrap_err();
        assert!(error.contains("not a finite number"), "{law}: {error}");
    }
}

#[test]
fn the_sources_of_a_simulation_are_files_of_the_model_s_folder() {
    let model = |sources: &str| SWEPT.replace("SWEEP", "").replace("SOURCES", &format!("sources: [{sources}]"));
    compile(&model("\"physics/P.mo\"")).expect("a file beside the model");
    for outside in ["\"../../outside.mo\"", "\"/etc/hosts\""] {
        let error = compile(&model(outside)).unwrap_err();
        assert!(error.contains("inside the model's folder"), "{error}");
    }
}

#[test]
fn a_label_of_an_imported_diagram_stays_inside_its_string() {
    let from_plantuml = arclang::compiler::plantuml_importer::import_plantuml("@startuml\n[Foo \"bar\"] --> [Baz]\n@enduml\n").unwrap();
    compile(&from_plantuml).unwrap_or_else(|e| panic!("{e}\n{from_plantuml}"));
    let from_mermaid = arclang::compiler::mermaid_importer::import_mermaid("flowchart TD\n    A[\"say \\\"hi\\\" \\\\\"] --> B[plain]\n").unwrap();
    compile(&from_mermaid).unwrap_or_else(|e| panic!("{e}\n{from_mermaid}"));
}

#[test]
fn a_name_of_the_model_cannot_close_a_comment_of_the_generated_code() {
    let result = compile(
        "model \"A */ #define X 1\nLine2\" {}\narchitecture logical {\n  component \"C */ int y;\" { id: \"LC-1\" port in \"p*/q\" }\n}\n",
    )
    .unwrap();
    let header = arclang::compiler::c_header_generator::generate_c_headers(&result.semantic_model, &result.ast);
    for line in header.lines().filter(|l| l.contains("#define X") || l.contains("int y;")) {
        assert!(line.trim_start().starts_with("/*") || line.trim_start().starts_with('*'), "live code from a name: {line}");
    }
    assert!(!header.contains("p*/q"), "{header}");
    let proto = arclang::compiler::proto_generator::generate_proto(&result.semantic_model, &result.ast);
    assert!(!proto.lines().any(|l| l.starts_with("Line2")), "{proto}");
}
