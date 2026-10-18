//! What the SysML v2 import reads of the Arcadia profile: ports with ids,
//! exchanges between whole components, logical interfaces, ArcLang types,
//! quantities, lists and constraints. On text in the profile, and on the
//! export of the examples that carry each construct.

use arclang::compiler::{sysml_import, sysmlv2_generator, Compiler, CompilerConfig};
use std::path::Path;

fn imported(text: &str) -> String {
    sysml_import::import_text(text).unwrap_or_else(|f| panic!("{}\n{}", f.message, f.rejected.unwrap_or_default())).source
}

fn exported_example(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let source = std::fs::read_to_string(&path).unwrap();
    let result = Compiler::new(CompilerConfig::default()).compile_string(&source).expect("the example compiles");
    sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast)
}

fn assert_has(source: &str, expected: &str) {
    assert!(source.contains(expected), "expected `{expected}` in:\n{source}");
}

const PROFILE: &str = "library package ArcadiaProfile { metadata def OA; metadata def SA; metadata def LA; metadata def PA; metadata def Interface; metadata def Link; metadata def Chain; }\n    private import ArcadiaProfile::*;";

#[test]
fn ports_keep_their_ids_and_exchanges_join_ports_or_whole_components() {
    let text = format!(
        "package <'M'> 'Model' {{\n    {PROFILE}\n    #LA package 'Logical Architecture' {{\n        port def 'Data port' {{ inout item carried; }}\n        part <'LC-1'> Sensor {{ out port <'0ad613f5'> 'CP 1' : 'Data port'; }}\n        part <'LC-2'> Fusion {{ in port <'585a7588'> 'CP 2' : 'Data port'; }}\n        connection 'Radar Data' connect 'LC-1'.'CP 1' to 'LC-2'.'CP 2';\n        connection Whole connect 'LC-1' to 'LC-2';\n        #Interface connection <'LI-1'> 'Command Interface' connect 'LC-1' to 'LC-2' {{\n            attribute interface_type : String = \"Data\";\n        }}\n    }}\n}}\n"
    );
    let source = imported(&text);
    assert_has(&source, "model \"M\" {\n    name: \"Model\"");
    assert_has(&source, "port out \"CP 1\" { id: \"0ad613f5\" }");
    assert_has(&source, "port in \"CP 2\" { id: \"585a7588\" }");
    assert_has(&source, "component_exchange \"Radar Data\" {\n        id: \"Radar Data\"\n        from_port: \"LC-1.CP 1\"\n        to_port: \"LC-2.CP 2\"");
    assert_has(&source, "component_exchange \"Whole\" {\n        id: \"Whole\"\n        from_port: \"LC-1\"\n        to_port: \"LC-2\"");
    assert_has(&source, "interface \"Command Interface\" {\n        id: \"LI-1\"\n        from: \"LC-1\"\n        to: \"LC-2\"\n        interface_type: \"Data\"");
}

#[test]
fn the_timing_example_comes_back_with_its_types_quantities_and_constraints() {
    let source = imported(&exported_example("examples/automotive/timing_constraints.arc"));
    // Types of the `Types` package, their required attributes and defaults.
    assert_has(&source, "type \"ECU\" {\n    required: [\"ram\"]\n    voltage: 12 V\n}");
    assert_has(&source, "type \"Safety ECU\" extends \"ECU\" {\n    safety_level: \"ASIL-B\"\n}");
    assert_has(&source, "type \"Periodic task\" {\n    required: [\"wcet\", \"latency\"]\n    period: 20 ms\n}");
    // Quantities on elements, redefinitions named by what they redefine.
    assert_has(&source, "latency: 30 ms");
    assert_has(&source, "period: 33 ms");
    assert_has(&source, "wcet: 0.035 s");
    assert_has(&source, "is: \"Periodic task\"");
    // A link typed by a type, with its attributes.
    assert_has(&source, "is: \"CAN FD bus\"\n        load: 600 kbps");
    // Constraints: sums unfolded by the export, `max` over a chain folded back.
    assert_has(&source, "assert: ((\"SF-DET\".latency + \"SF-EST\".latency) + \"SF-CMD\".latency) <= (\"FC-LKA\".latency_budget * 0.8)");
    assert_has(&source, "assert: max(\"FC-LKA\", wcet) <= (\"Capture frame\".frame_period * 1.1)");
    assert_has(&source, "assert: (ChassisCAN.load / ChassisCAN.bandwidth) <= 0.4");
    // The chain involves its steps.
    assert_has(&source, "involves: [\"SF-DET\", \"SF-EST\", \"SF-CMD\"]");
}

#[test]
fn the_flight_control_example_comes_back_with_its_interfaces_and_lists() {
    let source = imported(&exported_example("examples/aerospace/flight_control_system.arc"));
    assert_has(&source, "interface \"Command Interface\" {\n        id: \"LI-001\"\n        from: \"LC-001\"\n        to: \"LC-003\"\n        interface_type: \"Data\"");
    assert_has(&source, "phases: [\"takeoff\", \"cruise\", \"landing\"]");
}
