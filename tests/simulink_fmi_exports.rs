//! The Simulink / System Composer script and the FMI 2.0 model descriptions
//! follow the documented APIs: every variable the script uses is defined,
//! the logical hierarchy is kept, connections are port to port, Stateflow
//! charts are built from documented calls; every model description says
//! its initial unknowns, and its GUID changes with its interface.

use arclang::compiler::fmi_generator::generate_fmi_descriptors;
use arclang::compiler::simulink_generator::generate_simulink_script;
use arclang::compiler::{CompilationResult, Compiler, CompilerConfig};
use std::collections::BTreeSet;

const MODEL: &str = r#"
model Demo {}

data_type "Count" { base_type: "integer" }

system_analysis "SA" {
    actor "Driver" { id: "ACT-DRV" }
}

logical_architecture "Logical" {
    component "Vehicle" {
        id: "LC-VEH"
        component "Sensor" {
            port out "speed" { }
            port inout "bus" { }
        }
        component "Brake" { id: "LC-BRK" port in "speed" { } }
    }
    component "Display" { id: "LC-DSP" port in "count" { } }
    component "Display" { id: "LC-DSP2" }
    component "Counter" { id: "LC-CNT" port out "count" { } }
    component_exchange "Speed" { from_port: "Sensor.speed" to_port: "LC-BRK.speed" }
    component_exchange "Count" { from_port: "LC-CNT.count" to_port: "LC-DSP.count" exchange_item: "Count" }
    component_exchange "Status" { from_port: "LC-BRK" to_port: "LC-DSP2" }
}

physical_architecture "Physical" {
    node "ECU" { id: "PN-ECU" port "eth" }
    node "Gateway" { id: "PN-GW" port "eth" }
    link "Backbone" { from: "PN-ECU.eth" to: "PN-GW.eth" protocol: "Ethernet" }
}

state_machine "Modes" {
    initial: "Off"
    state "Off" { }
    state "Off!" { }
    state "On" { }
    transition "Off" -> "On" { trigger: "power on" guard: "battery > 10 %" }
    transition "On" -> "Off!" { trigger: "fault" }
}
"#;

fn compile(source: &str) -> CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).unwrap_or_else(|e| panic!("{e}"))
}

fn script(source: &str) -> String {
    let result = compile(source);
    generate_simulink_script(&result.semantic_model, &result.ast)
}

/// Variables assigned (`x = ...`) and variables used as call arguments or
/// receivers (`x.`, `(x,`, `x)`): every one used must be assigned before.
fn undefined_variables(script: &str) -> Vec<String> {
    let mut defined: BTreeSet<String> = ["sfroot", "true", "false"].iter().map(|s| s.to_string()).collect();
    let mut missing = Vec::new();
    for line in script.lines() {
        let code = line.split('%').next().unwrap_or("");
        let code = code.split('\'').enumerate().filter(|(i, _)| i % 2 == 0).map(|(_, s)| s).collect::<Vec<_>>().join(" ");
        for statement in code.split(';') {
            let (left, right) = match statement.split_once('=') {
                Some((l, r)) if !r.starts_with('=') && !l.trim().ends_with(['<', '>', '~']) => (l.trim(), r),
                _ => ("", statement),
            };
            for token in right.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.')) {
                let head = token.split('.').next().unwrap_or("");
                let is_var = head.starts_with("c_") || head.starts_with("p_") || head.starts_with("s_") || head.starts_with("t_") || head.starts_with("chart") || head == "arch" || head == "archModel" || head == "phys" || head == "physModel";
                if is_var && !defined.contains(head) && !missing.contains(&head.to_string()) {
                    missing.push(head.to_string());
                }
            }
            let target = left.split('.').next().unwrap_or("").trim();
            if !target.is_empty() {
                if left.contains('.') && !defined.contains(target) && !missing.contains(&target.to_string()) {
                    missing.push(target.to_string());
                }
                defined.insert(target.to_string());
            }
        }
    }
    missing
}

#[test]
fn every_variable_the_script_uses_is_defined() {
    let text = script(MODEL);
    assert_eq!(undefined_variables(&text), Vec::<String>::new(), "{text}");
}

#[test]
fn the_script_uses_the_documented_system_composer_calls() {
    let text = script(MODEL);
    assert!(text.contains("archModel = systemcomposer.createModel('Demo');"), "{text}");
    assert!(!text.contains(", true)"), "{text}");
    assert!(text.contains("save(archModel);"), "{text}");
    // The hierarchy: the sensor is in the vehicle.
    assert!(text.contains("c_LC_Sensor = addComponent(c_LC_VEH.Architecture, 'Sensor');"), "{text}");
    // Port to port, through the vehicle's boundary for the sensor.
    assert!(!text.contains("connect(arch, c_"), "{text}");
    assert!(text.contains("connect(c_LC_Sensor.getPort('speed'), "), "{text}");
    // Two displays at one level get distinct names.
    assert!(text.contains("addComponent(arch, 'Display');") && text.contains("addComponent(arch, 'Display (2)');"), "{text}");
    // An actor of the system analysis is not a component of the logical architecture.
    assert!(!text.contains("'Driver'"), "{text}");
    // An exchange between two components without ports gets ports of its own.
    assert!(text.contains("addPort(c_LC_BRK.Architecture, 'Status', 'out')"), "{text}");
    // The physical architecture is its own model, links port to port.
    assert!(text.contains("physModel = systemcomposer.createModel('Demo_physical');"), "{text}");
    assert!(text.contains("connect(c_PN_ECU.getPort('eth'), c_PN_GW.getPort('eth'));"), "{text}");
}

#[test]
fn each_exchange_is_connected_once() {
    let text = script(MODEL);
    for exchange in ["% Speed", "% Count", "% Status"] {
        assert_eq!(text.matches(exchange).count(), 1, "{exchange}\n{text}");
    }
}

#[test]
fn stateflow_charts_come_from_documented_calls() {
    let text = script(MODEL);
    assert!(!text.contains("Stateflow.Chart("), "{text}");
    assert!(text.contains("sfnew(sfModelName);"), "{text}");
    assert!(text.contains("find(sfroot, '-isa', 'Stateflow.Chart', 'Path', "), "{text}");
    // Names stay distinct after cleaning: Off and Off!.
    assert!(text.contains(".Name = 'Off';") && text.contains(".Name = 'Off_2';"), "{text}");
    // A default transition into the initial state.
    assert!(text.contains("t_default_0.Destination = s_0_Off;"), "{text}");
    // The trigger is an event of the chart; the guard is said in the description.
    assert!(text.contains(".LabelString = 'power_on';"), "{text}");
    assert!(text.contains("e_0_power_on = Stateflow.Event(chart_0); e_0_power_on.Name = 'power_on';"), "{text}");
    assert!(text.contains(".Description = 'power on [battery > 10 %]';"), "{text}");
    assert!(text.contains("save_system(sfModelName);"), "{text}");
}

#[test]
fn every_example_gives_a_script_without_undefined_variables() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut files = Vec::new();
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() && !path.ends_with("legacy") {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "arc") {
                out.push(path);
            }
        }
    }
    walk(&root, &mut files);
    let mut checked = 0;
    for file in files {
        let Ok(result) = Compiler::new(CompilerConfig::default()).compile_file(&file) else { continue };
        let text = generate_simulink_script(&result.semantic_model, &result.ast);
        assert_eq!(undefined_variables(&text), Vec::<String>::new(), "{}", file.display());
        checked += 1;
    }
    assert!(checked > 10, "{checked}");
}

fn descriptor(source: &str, id: &str) -> String {
    let result = compile(source);
    generate_fmi_descriptors(&result.semantic_model, &result.ast).into_iter().find(|d| d.component_id == id).unwrap_or_else(|| panic!("no FMU for {id}")).xml
}

fn guid(xml: &str) -> String {
    xml.split("guid=\"").nth(1).unwrap().split('"').next().unwrap().to_string()
}

#[test]
fn outputs_are_initial_unknowns() {
    let xml = descriptor(MODEL, "LC-CNT");
    assert!(xml.contains("<InitialUnknowns>\n      <Unknown index=\"1\"/>\n    </InitialUnknowns>"), "{xml}");
}

#[test]
fn the_guid_follows_the_interface_and_the_model() {
    let before = guid(&descriptor(MODEL, "LC-CNT"));
    let unrelated = guid(&descriptor(&MODEL.replace("component \"Display\" { id: \"LC-DSP2\" }", "component \"Display\" { id: \"LC-DSP2\" description: \"spare\" }"), "LC-CNT"));
    assert_eq!(before, unrelated);
    let changed = guid(&descriptor(&MODEL.replace("port out \"count\" { }", "port out \"count\" { }\n        port in \"reset\" { }"), "LC-CNT"));
    assert_ne!(before, changed);
    let other_model = guid(&descriptor(&MODEL.replace("model Demo {}", "model Other {}"), "LC-CNT"));
    assert_ne!(before, other_model);
}

#[test]
fn variables_are_typed_by_what_flows_and_written_once() {
    let xml = descriptor(MODEL, "LC-CNT");
    assert!(xml.contains("name=\"count\" valueReference=\"0\" causality=\"output\" variability=\"discrete\" initial=\"calculated\">\n      <Integer/>"), "{xml}");
    assert_eq!(xml.matches("<ScalarVariable").count(), 1, "{xml}");
}

#[test]
fn an_inout_port_is_an_input_and_an_output() {
    let xml = descriptor(MODEL, "LC-Sensor");
    assert!(xml.contains("name=\"bus_in\"") && xml.contains("name=\"bus_out\""), "{xml}");
    assert!(xml.contains("name=\"speed\""), "{xml}");
}

#[test]
fn a_component_without_ports_but_an_exchange_has_its_variable() {
    let xml = descriptor(MODEL, "LC-DSP2");
    assert!(xml.contains("name=\"Status\"") && xml.contains("causality=\"input\""), "{xml}");
}
