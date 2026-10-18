//! What a Capella round trip needs the compiled model to keep: the
//! attributes of exchanges, the actor flag of logical components and
//! physical nodes, the kind and attributes of states and transitions,
//! the port ends of physical links, the EPBS items as elements, and the
//! allocation of a behaviour component kept apart from its functions.

use arclang::compiler::diagram::build_diagrams;
use arclang::compiler::elements;
use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"model T { name: "t" }
system_analysis "SA" {
    id: "ARCH-SA"
    function "Sense" { id: "SF-Sense" port out "targets" }
    function "Decide" { id: "SF-Decide" port in "targets" }
    functional_exchange "targets" { id: "SA-targets" from: "SF-Sense.targets" to: "SF-Decide.targets" exchange_items: ["Targets"] categories: ["data"] description: "the detected targets" }
    system_component "AEB" { id: "SC-AEB" port in "bus" }
    actor "Driver" { id: "SA-Driver" port out "pedal" }
    component_exchange "pedal" { id: "SA-pedal" from_port: "SA-Driver.pedal" to_port: "SC-AEB.bus" categories: ["mechanical"] }
}
logical_architecture "LA" {
    component "Driver" { id: "LC-Driver" actor: true }
    component "Controller" { id: "LC-Ctrl" function "Decide" { id: "LF-Decide" realizes: "SF-Decide" } }
}
physical_architecture "PA" {
    node "Passenger" { id: "PN-Pax" actor: true }
    node "ECU" {
        id: "PN-ECU"
        port "eth0" { id: "PP-1" }
        behavior_component "Controller software" { id: "PC-Ctrl" allocated_component: "LC-Ctrl" allocated_functions: ["LF-Decide"] port in "bus" }
    }
    node "Gateway" { id: "PN-GW" port "eth1" }
    physical_link "ethernet" { id: "PL-1" from: "PN-ECU.eth0" to: "PN-GW.eth1" protocol: "Ethernet" }
    physical_exchange "frames" { id: "PE-1" from: "PC-Ctrl.bus" to: "PN-GW" via: "PL-1" exchange_items: ["Targets"] }
}
epbs "Product" {
    id: "ARCH-EPBS"
    system "IFE" {
        id: "CI-IFE"
        subsystem "Cabin" {
            id: "CI-Cabin"
            item "Seat unit" { id: "CI-Seat" }
        }
    }
}
data_type "Speed" { id: "DT-Speed" base: "Real" unit_name: "knots" status: "REVIEWED" }
enumeration "Level" { id: "EN-Level" values: ["Low", "High"] status: "DRAFT" }
exchange_item "Targets" { id: "EI-Targets" mechanism: "FLOW" status: "DRAFT" }
state_machine "Modes" {
    id: "SM-1"
    owner: "SC-AEB"
    initial: "Off"
    state "Off" { id: "ST-Off" }
    state "Running" {
        id: "ST-Run"
        initial: "Idle"
        state "Idle" { id: "ST-Idle" }
        state "Done" { id: "ST-Done" kind: "final" }
        transition "Idle" -> "Done" { id: "TR-2" trigger: "" events: ["timeout"] guard: "ready" }
    }
    transition "Off" -> "Running" { id: "TR-1" trigger: "power" events: ["power on", "wake"] action: ["SF-Sense", "SF-Decide"] }
}
scenario "Seat use" {
    owner: "SC-AEB"
    participants: ["CI-Seat", "SA-Driver"]
    message "SA-Driver" -> "CI-Seat" "press" { kind: "CREATE" exchange: "SA-pedal" }
}
"#;

fn compile(source: &str) -> arclang::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles")
}

#[test]
fn exchanges_keep_their_attributes() {
    let result = compile(MODEL);
    let sa = &result.ast.system_analysis[0];
    let targets = &sa.functional_exchanges[0];
    assert_eq!(targets.attributes.get("description").and_then(|v| v.as_string()), Some("the detected targets"));
    assert!(targets.attributes.contains_key("exchange_items") && targets.attributes.contains_key("categories"));
    assert!(sa.component_exchanges[0].attributes.contains_key("categories"));
    let pa = &result.ast.physical_architecture[0];
    assert!(pa.physical_exchanges[0].attributes.contains_key("exchange_items"));
}

#[test]
fn actors_are_kept_on_logical_components_and_physical_nodes() {
    let result = compile(MODEL);
    let la = &result.ast.logical_architecture[0];
    assert_eq!(la.components[0].attributes.get("actor").map(|v| v.display()), Some("true".to_string()));
    let pa = &result.ast.physical_architecture[0];
    assert_eq!(pa.nodes[0].attributes.get("actor").map(|v| v.display()), Some("true".to_string()));
}

#[test]
fn states_keep_kind_initial_and_attributes_and_transitions_keep_events() {
    let result = compile(MODEL);
    let machine = &result.ast.state_machines[0];
    let running = &machine.states[1];
    assert_eq!(running.initial, "Idle");
    assert_eq!(running.sub_states[1].attributes.get("kind").and_then(|v| v.as_string()), Some("final"));
    let power = machine.transitions.iter().find(|t| t.id == "TR-1").expect("transition");
    assert!(power.attributes.contains_key("events"), "{:?}", power.attributes);
    let nested = machine.transitions.iter().find(|t| t.id == "TR-2").expect("nested transition");
    assert_eq!(nested.guard.as_deref(), Some("ready"));
}

#[test]
fn a_physical_link_ends_on_ports_and_is_drawn_between_their_nodes() {
    let result = compile(MODEL);
    assert!(result.warnings.iter().all(|w| !w.contains("PL-1") && !w.contains("ethernet")), "{:?}", result.warnings);
    let diagrams = build_diagrams(&result.ast);
    let pab = diagrams.diagrams.iter().find(|d| d.id == "pab").expect("pab");
    let link = pab.edges.iter().find(|e| e.label == "ethernet").expect("link drawn");
    assert_eq!((link.source.as_str(), link.target.as_str()), ("PN-ECU", "PN-GW"));
    assert_eq!(link.source_port.as_deref(), Some("PN-ECU::eth0"));
    assert_eq!(link.target_port.as_deref(), Some("PN-GW::eth1"));
}

#[test]
fn a_behaviour_component_keeps_its_allocated_component_apart_from_its_functions() {
    let result = compile(MODEL);
    let behaviour = &result.ast.physical_architecture[0].nodes[1].behavior_components[0];
    assert_eq!(behaviour.allocated_component.as_deref(), Some("LC-Ctrl"));
    assert_eq!(behaviour.allocated_functions, vec!["LF-Decide"]);
}

#[test]
fn epbs_items_are_elements_a_scenario_can_name() {
    let result = compile(MODEL);
    let graph = elements::build(&result.ast, &result.semantic_model);
    assert!(graph.elements.iter().any(|e| e.kind == "EpbsItem" && e.id == "CI-Seat"), "{:?}", graph.elements.iter().filter(|e| e.kind.starts_with("Epbs")).map(|e| (&e.kind, &e.id)).collect::<Vec<_>>());
    assert_eq!(result.ast.scenarios[0].participants[0].id, "CI-Seat");
}

#[test]
fn an_empty_operational_process_is_a_warning_not_an_error() {
    let source = r#"model T { name: "t" }
operational_analysis "OA" {
    entity "Crew" { id: "OE-Crew" activity "Serve" { id: "OA-Serve" } }
    operational_process "Not described yet" { id: "OP-1" }
}
"#;
    let result = compile(source);
    assert!(result.warnings.iter().any(|w| w.contains("Not described yet")), "{:?}", result.warnings);
}

#[test]
fn data_types_exchange_items_and_messages_keep_their_attributes() {
    let result = compile(MODEL);
    let speed = result.ast.data_types.iter().find(|d| d.id == "DT-Speed").expect("data type");
    assert_eq!(speed.attributes.get("unit_name").and_then(|v| v.as_string()), Some("knots"));
    let level = result.ast.data_types.iter().find(|d| d.id == "EN-Level").expect("enumeration");
    assert_eq!(level.attributes.get("status").and_then(|v| v.as_string()), Some("DRAFT"));
    assert_eq!(result.ast.exchange_items[0].properties.get("status").and_then(|v| v.as_string()), Some("DRAFT"));
    let message = &result.ast.scenarios[0].messages[0];
    assert_eq!(message.attributes.get("kind").and_then(|v| v.as_string()), Some("CREATE"));
    assert_eq!(message.attributes.get("exchange").and_then(|v| v.as_string()), Some("SA-pedal"));
    assert!(result.warnings.iter().all(|w| !w.contains("SA-pedal") && !w.contains("owner")), "{:?}", result.warnings);
}

#[test]
fn an_unknown_carried_exchange_or_owner_is_a_warning() {
    let stranger = MODEL.replace(r#"exchange: "SA-pedal""#, r#"exchange: "SA-ghost""#).replace(r#"owner: "SC-AEB""#, r#"owner: "Nobody""#);
    let result = compile(&stranger);
    assert!(result.warnings.iter().any(|w| w.contains("SA-ghost")), "{:?}", result.warnings);
    assert!(result.warnings.iter().filter(|w| w.contains("Nobody")).count() == 2, "{:?}", result.warnings);
}

#[test]
fn a_level_block_is_an_element_an_owner_can_name() {
    let at_level = MODEL.replace(r#"owner: "SC-AEB""#, r#"owner: "ARCH-SA""#);
    let result = compile(&at_level);
    assert!(result.warnings.iter().all(|w| !w.contains("ARCH-SA")), "{:?}", result.warnings);
}

#[test]
fn a_transition_action_may_be_a_list_and_the_epbs_block_carries_its_id() {
    let result = compile(MODEL);
    let power = result.ast.state_machines[0].transitions.iter().find(|t| t.id == "TR-1").expect("transition");
    assert_eq!(power.action.as_deref(), Some("SF-Sense, SF-Decide"));
    assert_eq!(result.ast.epbs[0].id, "ARCH-EPBS");
    let at_epbs = MODEL.replace(r#"owner: "SC-AEB""#, r#"owner: "ARCH-EPBS""#);
    assert!(compile(&at_epbs).warnings.iter().all(|w| !w.contains("ARCH-EPBS")));
}

#[test]
fn a_region_keeps_the_transitions_declared_in_it() {
    let result = compile(MODEL);
    let machine = &result.ast.state_machines[0];
    let running = &machine.states[1];
    assert_eq!(running.transitions.len(), 1);
    assert_eq!(running.transitions[0].id, "TR-2");
    assert_eq!(machine.transitions.len(), 2, "the flat list still holds every transition");
}
