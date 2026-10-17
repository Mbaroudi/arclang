//! Functions of each level, as Arcadia has them: logical functions inside
//! the component that performs them, physical functions inside behaviour
//! components, each with ports and sub-functions, exchanged between at
//! their own level and realizing the function of the level above; the
//! system, its components and its actors with ports and component
//! exchanges.

use arclang::compiler::diagram::build_diagrams;
use arclang::compiler::elements;
use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"model T { name: "t" }
system_analysis "SA" {
    function "Sense" { id: "SF-Sense" port out "targets" }
    function "Decide" { id: "SF-Decide" port in "targets" port out "order" }
    functional_exchange "targets" { from: "SF-Sense.targets" to: "SF-Decide.targets" }
    system_component "AEB" { id: "SC-AEB" port in "vehicle bus" port out "brake command" }
    actor "Driver" { id: "SA-Driver" port out "pedal" }
    component_exchange "pedal" { from_port: "SA-Driver.pedal" to_port: "SC-AEB.vehicle bus" }
}
logical_architecture "LA" {
    component "Sensors" {
        id: "LC-Sensors"
        function "Sense targets" {
            id: "LF-Sense"
            realizes: "SF-Sense"
            port out "targets"
            function "Filter noise" { id: "LF-Filter" }
        }
    }
    component "Controller" {
        id: "LC-Ctrl"
        function "Decide braking" { id: "LF-Decide" realizes: "SF-Decide" port in "targets" port out "order" }
    }
    function "Root logical function" { id: "LF-Root" }
    functional_exchange "targets" { from: "LF-Sense.targets" to: "LF-Decide.targets" }
}
physical_architecture "PA" {
    node "ECU" {
        id: "PN-ECU"
        behavior_component "Controller software" {
            id: "PC-Ctrl"
            function "Decide braking" { id: "PF-Decide" realizes: "LF-Decide" port in "targets" }
        }
        behavior_component "Sensor driver" {
            id: "PC-Sensor"
            function "Sense targets" { id: "PF-Sense" realizes: "LF-Sense" port out "targets" }
        }
    }
    functional_exchange "targets" { from: "PF-Sense.targets" to: "PF-Decide.targets" }
}
"#;

fn compile(source: &str) -> arclang::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles")
}

#[test]
fn logical_functions_carry_ports_sub_functions_and_their_own_exchanges() {
    let result = compile(MODEL);
    let la = &result.ast.logical_architecture[0];
    let sensors = &la.components[0];
    let sense = &sensors.functions[0];
    assert_eq!(sense.id, "LF-Sense");
    assert_eq!(sense.ports.len(), 1);
    assert_eq!(sense.sub_functions[0].id, "LF-Filter");
    assert_eq!(la.functions[0].id, "LF-Root");
    assert_eq!(la.functional_exchanges.len(), 1);
    let set = build_diagrams(&result.ast);
    let lab = set.diagrams.iter().find(|d| d.id == "lab").expect("lab drawn");
    let edge = lab.edges.iter().find(|e| e.label == "targets" && e.source != "SF-Sense").expect("the logical exchange is drawn");
    assert!(edge.source == "LF-Sense" && edge.target == "LF-Decide", "{edge:?}");
    assert!(lab.nodes.iter().flat_map(|n| n.children.iter()).any(|n| n.id == "LF-Sense" && n.children.iter().any(|c| c.id == "LF-Filter")));
    // A function no component performs stands on its own, so that a declared diagram can list it.
    assert!(lab.nodes.iter().any(|n| n.id == "LF-Root"));
}

#[test]
fn physical_functions_live_in_behaviour_components_and_exchange_at_their_level() {
    let result = compile(MODEL);
    let pa = &result.ast.physical_architecture[0];
    let behaviour = &pa.nodes[0].behavior_components[0];
    assert_eq!(behaviour.functions[0].id, "PF-Decide");
    let set = build_diagrams(&result.ast);
    let pab = set.diagrams.iter().find(|d| d.id == "pab").expect("pab drawn");
    let edge = pab.edges.iter().find(|e| e.label == "targets" && e.source != "SF-Sense").expect("the physical exchange is drawn");
    assert!(edge.source == "PF-Sense" && edge.target == "PF-Decide", "{edge:?}");
}

#[test]
fn the_system_level_has_component_ports_and_exchanges() {
    let result = compile(MODEL);
    let sa = &result.ast.system_analysis[0];
    assert_eq!(sa.components[0].ports.len(), 2);
    assert_eq!(sa.external_actors[0].ports.len(), 1);
    assert_eq!(sa.component_exchanges.len(), 1);
    let set = build_diagrams(&result.ast);
    let sab = set.diagrams.iter().find(|d| d.id == "sab").expect("sab drawn");
    let edge = sab.edges.iter().find(|e| e.label == "pedal").expect("the component exchange is drawn");
    assert_eq!((edge.source.as_str(), edge.target.as_str()), ("SA-Driver", "SC-AEB"));
    let aeb = sab.nodes.iter().find(|n| n.id == "SC-AEB").expect("the system component");
    assert_eq!(aeb.ports.len(), 2);
}

#[test]
fn level_functions_are_elements_with_their_kind_and_realizations() {
    let result = compile(MODEL);
    let graph = elements::build(&result.ast, &result.semantic_model);
    let kinds: Vec<(&str, &str)> = graph.elements.iter().map(|e| (e.kind, e.id.as_str())).collect();
    assert!(kinds.contains(&("LogicalFunction", "LF-Sense")), "{kinds:?}");
    assert!(kinds.contains(&("LogicalFunction", "LF-Filter")));
    assert!(kinds.contains(&("LogicalFunction", "LF-Root")));
    assert!(kinds.contains(&("PhysicalFunction", "PF-Decide")));
    assert!(kinds.contains(&("BehaviorComponent", "PC-Ctrl")));
    let sense = graph.elements.iter().find(|e| e.id == "LF-Sense").expect("LF-Sense");
    let ports = graph.elements.iter().filter(|e| e.kind == "FunctionPort" && e.owner.as_deref() == Some(sense.uuid.as_str())).count();
    assert_eq!(ports, 1);
    assert_eq!(sense.attributes.get("realizes").and_then(|v| v.as_string()), Some("SF-Sense"));
}

#[test]
fn an_exchange_to_an_unknown_function_is_reported() {
    let bad = MODEL.replace(r#"to: "LF-Decide.targets" }"#, r#"to: "LF-Nowhere.targets" }"#);
    let result = compile(&bad);
    assert!(result.warnings.iter().any(|w| w.contains("LF-Nowhere")), "{:?}", result.warnings);
}
