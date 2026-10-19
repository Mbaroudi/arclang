//! Limits the example authors hit in 4.3.0, each pinned: keyword keys on an
//! interface, a typed `frequency` on a physical exchange, the SysML round
//! trip of a state fragment with a body, and hazards classified by SIL
//! under IEC 61508.

use arclang::compiler::ast::AttributeValue;
use arclang::compiler::production_gate::{run_gate, Severity};
use arclang::compiler::{elements, sysml_import, sysml_parser, sysmlv2_generator, workbench, Compiler, CompilerConfig};
use std::path::Path;

fn compile(source: &str) -> arclang::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).unwrap_or_else(|e| panic!("compiles: {e}"))
}

fn export(result: &arclang::CompilationResult) -> String {
    sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast)
}

const INTERFACE_MODEL: &str = r#"model T {}
architecture logical {
    component "C" { id: "LC-1" }
    component "D" { id: "LC-2" }
    interface "Bus" {
        id: "IF-1" from: "LC-1" to: "LC-2"
        protocol: "CAN FD" latency: 10 ms frequency: 1 Hz bandwidth: 2 Mbps
    }
}
"#;

#[test]
fn interface_attribute_keys_may_be_keywords() {
    let result = compile(INTERFACE_MODEL);
    let interface = &result.ast.logical_architecture[0].interfaces[0];
    assert_eq!(interface.attributes.get("protocol").and_then(|v| v.as_string()), Some("CAN FD"));
    assert_eq!(interface.attributes.get("latency").map(|v| v.display()).as_deref(), Some("10 ms"));
    assert_eq!(interface.attributes.get("frequency").map(|v| v.display()).as_deref(), Some("1 Hz"));
    assert_eq!(interface.attributes.get("bandwidth").map(|v| v.display()).as_deref(), Some("2 Mbps"));

    // `arclang workbench`: the relationship carries its attributes.
    let json = workbench::elements_json(&elements::build(&result.ast, &result.semantic_model));
    let bus = json["relationships"].as_array().unwrap().iter().find(|r| r["kind"] == "LogicalInterface").expect("the interface is a relationship");
    assert_eq!(bus["attributes"]["protocol"], "CAN FD", "{bus}");
    assert_eq!(bus["attributes"]["latency"]["unit"], "ms", "{bus}");
    assert_eq!(bus["attributes"]["latency"]["value"], 10.0, "{bus}");

    let text = export(&result);
    assert!(text.contains("attribute protocol : String = \"CAN FD\";"), "{text}");
    assert!(text.contains("attribute latency : DurationValue = 10 [ms];"), "{text}");
    assert!(text.contains("attribute frequency : FrequencyValue = 1 [Hz];"), "{text}");
}

const EXCHANGE_MODEL: &str = r#"model T {}
system_analysis "SA" {
    requirement "REQ-1" { description: "fast" }
}
architecture logical {
    component "C" { id: "LC-1" }
    component "D" { id: "LC-2" }
}
architecture physical {
    node "ECU" { id: "PN-1" deploys "LC-1" }
    node "ACT" { id: "PN-2" deploys "LC-2" }
    physical_link "Cable" { id: "PL-1" from: "PN-1" to: "PN-2" protocol: "CAN" }
    physical_exchange "Setpoint" { id: "PX-1" from: "PN-1" to: "PN-2" via: "PL-1" message_type: "frame 0x120" frequency: 100 Hz }
    physical_exchange "Legacy" { id: "PX-2" from: "PN-1" to: "PN-2" via: "PL-1" message_type: "frame 0x121" frequency: "50 Hz" }
}
test_case "TC-1" { verifies: ["REQ-1"] method: "test" }
trace "LC-1" satisfies "REQ-1"
trace "LC-2" satisfies "REQ-1"
"#;

#[test]
fn physical_exchange_frequency_as_a_quantity_completes_the_icd() {
    let result = compile(EXCHANGE_MODEL);
    let exchanges = &result.ast.physical_architecture[0].physical_exchanges;
    assert_eq!(exchanges[0].frequency.as_deref(), Some("100 Hz"));
    assert_eq!(exchanges[1].frequency.as_deref(), Some("50 Hz"), "the string form is still accepted");
    assert!(matches!(exchanges[0].attributes.get("frequency"), Some(AttributeValue::Quantity(q)) if q.unit == "Hz"), "the typed value is kept");
    assert_eq!(exchanges[0].message_type, "frame 0x120");
    assert_eq!(exchanges[0].via.as_deref(), Some("PL-1"));

    let report = run_gate(&result.ast, &result.semantic_model, "ISO26262");
    let icd: Vec<_> = report.findings.iter().filter(|f| f.check == "icd.completeness").collect();
    assert!(icd.is_empty(), "{icd:?}");
    assert!(result.warnings.iter().all(|w| !w.contains("frequency")), "{:?}", result.warnings);

    let text = export(&result);
    assert!(text.contains("attribute frequency : FrequencyValue = 100 [Hz];"), "{text}");
    assert!(text.contains("attribute frequency : FrequencyValue = 50 [Hz];"), "{text}");

    let without = compile(&EXCHANGE_MODEL.replace(" frequency: 100 Hz", ""));
    let report = run_gate(&without.ast, &without.semantic_model, "ISO26262");
    assert!(report.findings.iter().any(|f| f.check == "icd.completeness" && f.message.contains("'Setpoint'") && f.message.contains("frequency")), "{:?}", report.findings);
}

const FRAGMENT_MODEL: &str = r#"model T {}
system_analysis "SA" {
    function "Sense" { id: "SF-Sense" }
    actor "Driver" { id: "SA-Driver" }
}
architecture logical {
    component "Controller" { id: "LC-X" }
}
state_machine "Modes" {
    id: "SM-1"
    initial: "Idle"
    mode "Idle" { id: "MD-Idle" }
    mode "Compute" { id: "MD-Compute" }
    transition "Idle" -> "Compute" { trigger: "SF-Sense" }
}
scenario "Braking" {
    id: "SC-1"
    participants: ["SA-Driver", "LC-X", "SF-Sense"]
    message "SA-Driver" -> "LC-X" "drive" { id: "MSG-1" }
    mode "LC-X" "Compute" {
        id: "SI-1"
        message "LC-X" -> "SF-Sense" "targets" { id: "MSG-2" }
        message "SF-Sense" -> "LC-X" "ack" { id: "MSG-3" }
    }
    message "LC-X" -> "SA-Driver" "brake" { id: "MSG-4" }
}
"#;

#[test]
fn a_state_fragment_with_a_body_round_trips_through_sysml() {
    let before = compile(FRAGMENT_MODEL);
    let text = export(&before);
    // `end` is a reserved word of SysML v2: the span ends at `endAt`.
    assert!(text.contains("attribute start : Integer = 1;"), "{text}");
    assert!(text.contains("attribute endAt : Integer = 2;"), "{text}");
    assert!(!text.contains("attribute end :"), "{text}");

    let built = sysml_parser::records(&text).unwrap_or_else(|e| panic!("the export does not parse: {e}\n{text}"));
    let imported = sysml_import::import_records(&built.records).unwrap_or_else(|f| panic!("{}\n{}", f.message, f.rejected.unwrap_or_default()));
    let after = compile(&imported.source);
    let scenario = &after.ast.scenarios[0];
    assert_eq!(scenario.messages.len(), 4, "{}", imported.source);
    let [invariant] = scenario.invariants.as_slice() else { panic!("{:?}", scenario.invariants) };
    assert_eq!((invariant.id.as_str(), invariant.lifeline.as_str()), ("SI-1", "LC-X"));
    assert_eq!((invariant.start, invariant.end), (1, 2), "the span is preserved: {}", imported.source);
    assert_eq!(invariant.kind, arclang::compiler::ast::InvariantKind::State);
    assert!(["Compute", "MD-Compute"].contains(&invariant.name.as_str()), "{}", invariant.name);
}

fn rail_example() -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/rail/level_crossing_protection.arc")).unwrap()
}

/// The rail example with its hazards classified by SIL instead of S/E/C.
fn rail_example_with_sil(sil: &str) -> String {
    let mut out = String::new();
    let mut skip_class = false;
    for line in rail_example().lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("severity:") {
            out.push_str(&format!("        sil: \"{sil}\"\n"));
            skip_class = true;
            continue;
        }
        if skip_class && (trimmed.starts_with("exposure:") || trimmed.starts_with("controllability:")) {
            continue;
        }
        skip_class = false;
        out.push_str(line);
        out.push('\n');
    }
    assert_ne!(out, rail_example(), "the example classifies its hazards by S/E/C");
    out
}

#[test]
fn iec61508_accepts_hazards_classified_by_sil() {
    let written = compile(&rail_example());
    let report = run_gate(&written.ast, &written.semantic_model, "IEC61508");
    assert!(report.passed, "S/E/C is still accepted: {:?}", report.findings);

    let by_sil = compile(&rail_example_with_sil("SIL-4"));
    assert!(by_sil.warnings.iter().all(|w| !w.contains("sil")), "{:?}", by_sil.warnings);
    let report = run_gate(&by_sil.ast, &by_sil.semantic_model, "IEC61508");
    assert!(report.passed, "a SIL classifies the hazard: {:?}", report.findings);
    assert!(report.findings.iter().all(|f| f.check != "safety.hara"), "{:?}", report.findings);

    // The value is checked against the Sil enumeration.
    let wrong = compile(&rail_example_with_sil("SIL-9"));
    let report = run_gate(&wrong.ast, &wrong.semantic_model, "IEC61508");
    assert!(report.findings.iter().any(|f| f.check == "metamodel.types" && f.severity == Severity::Blocker && f.message.contains("'SIL-9' is not a valid Sil")), "{:?}", report.findings);

    // ISO 26262 still wants its table.
    let report = run_gate(&by_sil.ast, &by_sil.semantic_model, "ISO26262");
    assert_eq!(report.findings.iter().filter(|f| f.check == "safety.hara").count(), 3, "{:?}", report.findings);
}
