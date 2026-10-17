//! The constructs Capella has that the language lacked: state and function
//! invariants on the lifelines of a scenario, control nodes and sequence
//! links of a functional chain, a chain involving another chain, and
//! associations between classes as elements of their own.

use arclang::compiler::diagram::build_diagrams;
use arclang::compiler::elements;
use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"model T { name: "t" }
class "Seat" { id: "CL-Seat" passenger: "Passenger" }
class "Passenger" { id: "CL-Pax" label: "String" }
association "has" { id: "AS-1" from: "CL-Seat" to: "CL-Pax" role: "passenger" kind: "composition" multiplicity: "0..1" }
operational_analysis "OA" {
    entity "Crew" {
        id: "OE-Crew"
        activity "Serve" { id: "OA-Serve" }
        activity "Clean" { id: "OA-Clean" }
    }
    interaction "meal done" { id: "OI-1" from: "OA-Serve" to: "OA-Clean" }
}
system_analysis "SA" {
    id: "ARCH-SA"
    function "Sense" { id: "SF-Sense" port out "targets" { id: "PORT-1" } }
    function "Decide" { id: "SF-Decide" port in "targets" port out "order" }
    function "Brake" { id: "SF-Brake" }
    function "Warn" { id: "SF-Warn" }
    functional_exchange "targets" { id: "SA-targets" from: "SF-Sense.targets" to: "SF-Decide.targets" }
    actor "Driver" { id: "SA-Driver" }
    functional_chain "Warn path" { id: "SFC-Warn" involves: ["SF-Warn"] }
    functional_chain "Brake path" {
        id: "SFC-1"
        involves: ["SF-Sense", "SA-targets", "SF-Decide", "SF-Brake", "SFC-Warn"]
        control and "Split" { id: "CN-1" }
        control and "Join"
        sequence "SF-Decide" -> "Split" { id: "SL-1" }
        sequence "Split" -> "SF-Brake" { condition: "obstacle close" }
        sequence "Split" -> "SFC-Warn" { condition: "obstacle far" }
        sequence "SF-Brake" -> "Join"
        sequence "SFC-Warn" -> "Join"
    }
}
state_machine "Modes" {
    id: "SM-1"
    initial: "Off"
    state "Off" { id: "ST-Off" }
    state "Armed" { id: "ST-Armed" }
    transition "Off" -> "Armed" { trigger: "" }
}
scenario "Emergency braking" {
    id: "SC-1"
    participants: ["SA-Driver", "SF-Sense", "SF-Decide"]
    message "SA-Driver" -> "SF-Sense" "drive" { id: "MSG-1" }
    state "SF-Decide" "Armed" { id: "SI-1" }
    function "SF-Sense" "Sense" {
        id: "SI-2"
        message "SF-Sense" -> "SF-Decide" "targets"
        message "SF-Decide" -> "SF-Sense" "ack"
    }
    message "SF-Decide" -> "SA-Driver" "brake"
}
trace "SA-targets" realizes "OI-1"
"#;

fn compile(source: &str) -> arclang::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles")
}

fn fails(source: &str) -> String {
    match Compiler::new(CompilerConfig::default()).compile_string(source) {
        Ok(_) => panic!("expected a compile error"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn scenario_lifelines_carry_state_and_function_invariants() {
    let result = compile(MODEL);
    let scenario = &result.ast.scenarios[0];
    assert_eq!(scenario.messages.len(), 4);
    let [armed, sensing] = scenario.invariants.as_slice() else { panic!("{:?}", scenario.invariants) };
    assert_eq!((armed.id.as_str(), armed.lifeline.as_str(), armed.name.as_str()), ("SI-1", "SF-Decide", "Armed"));
    assert!(armed.end < armed.start, "an invariant without a message is an instant");
    assert_eq!((sensing.start, sensing.end), (1, 2));
    assert_eq!(sensing.kind, arclang::compiler::ast::InvariantKind::Function);

    let diagrams = build_diagrams(&result.ast);
    let es = diagrams.diagrams.iter().find(|d| d.id == "es:Emergency braking").expect("scenario view");
    assert_eq!(es.invariants.len(), 2, "{:?}", es.invariants);
    assert_eq!(es.invariants[1].lifeline, "Emergency braking.SF-Sense");
    assert_eq!(es.invariants[1].label, "Sense");
    assert_eq!((es.invariants[1].start, es.invariants[1].end), (1, 2));

    let graph = elements::build(&result.ast, &result.semantic_model);
    assert!(graph.elements.iter().any(|e| e.kind == "StateFragment" && e.id == "SI-1"));
}

#[test]
fn an_invariant_must_name_a_participant_and_a_known_state_or_function() {
    let unknown_state = MODEL.replace(r#"state "SF-Decide" "Armed""#, r#"state "SF-Decide" "Asleep""#);
    assert!(fails(&unknown_state).contains("Asleep"));
    let stranger = MODEL.replace(r#"state "SF-Decide" "Armed""#, r#"state "SF-Brake" "Armed""#);
    assert!(fails(&stranger).contains("SF-Brake"));
}

#[test]
fn functional_chains_branch_through_control_nodes_and_involve_chains() {
    let result = compile(MODEL);
    let chain = &result.ast.system_analysis[0].functional_chains[1];
    assert_eq!(chain.controls.len(), 2);
    assert_eq!(chain.controls[0].id, "CN-1");
    assert_eq!(chain.controls[1].id, "SFC-1.Join");
    assert_eq!(chain.sequences.len(), 5);
    assert_eq!(chain.sequences[1].condition.as_deref(), Some("obstacle close"));
    assert!(result.semantic_model.functional_chains.iter().any(|c| c.id == "SFC-1" && c.involves.contains(&"SFC-Warn".to_string())));

    let diagrams = build_diagrams(&result.ast);
    let sab = diagrams.diagrams.iter().find(|d| d.id == "sab").expect("sab");
    let drawn = sab.chains.iter().find(|c| c.id == "SFC-1").expect("chain drawn");
    assert_eq!(drawn.controls.len(), 2);
    assert_eq!(drawn.sequences.len(), 5);
    assert_eq!((drawn.sequences[0].source.as_str(), drawn.sequences[0].target.as_str()), ("SF-Decide", "CN-1"));
    assert_eq!(drawn.sequences[2].target, "SFC-Warn");

    let graph = elements::build(&result.ast, &result.semantic_model);
    assert!(graph.elements.iter().any(|e| e.kind == "ControlNode" && e.id == "CN-1"));
    assert!(graph.relationships.iter().filter(|r| r.kind == "SequenceLink").count() == 5, "{:?}", graph.unresolved);
}

#[test]
fn a_sequence_link_must_join_steps_or_control_nodes_of_its_chain() {
    let elsewhere = MODEL.replace(r#"sequence "SF-Brake" -> "Join""#, r#"sequence "SF-Sense" -> "Nowhere""#);
    assert!(fails(&elsewhere).contains("Nowhere"));
    let outside = MODEL.replace(r#"sequence "SF-Brake" -> "Join""#, r#"sequence "SA-Driver" -> "Join""#);
    assert!(fails(&outside).contains("SA-Driver"));
}

#[test]
fn associations_are_elements_between_classes() {
    let result = compile(MODEL);
    let association = &result.ast.associations[0];
    assert_eq!((association.id.as_str(), association.from.as_str(), association.to.as_str()), ("AS-1", "CL-Seat", "CL-Pax"));
    assert_eq!(association.kind, arclang::compiler::ast::AssociationKind::Composition);

    let diagrams = build_diagrams(&result.ast);
    let cdb = diagrams.diagrams.iter().find(|d| d.id == "cdb").expect("cdb");
    let edges: Vec<_> = cdb.edges.iter().filter(|e| e.source == "CL-Seat" && e.target == "CL-Pax").collect();
    assert_eq!(edges.len(), 1, "the association stands for the field it names: {edges:?}");
    assert!(edges[0].id.ends_with("AS-1"), "{}", edges[0].id);
    assert_eq!(edges[0].label, "has");
    assert_eq!(edges[0].properties.get("kind").map(String::as_str), Some("composition"));
    assert_eq!(edges[0].properties.get("multiplicity").map(String::as_str), Some("0..1"));

    let graph = elements::build(&result.ast, &result.semantic_model);
    let relation = graph.relationships.iter().find(|r| r.kind == "Association").expect("association relationship");
    assert_eq!(relation.name.as_deref(), Some("has"));
}

#[test]
fn an_association_must_join_two_classes() {
    let unknown = MODEL.replace(r#"to: "CL-Pax""#, r#"to: "CL-Ghost""#);
    assert!(fails(&unknown).contains("CL-Ghost"));
    let no_such_field = MODEL.replace(r#"role: "passenger""#, r#"role: "pilot""#);
    assert!(fails(&no_such_field).contains("pilot"));
}

#[test]
fn operational_interactions_are_trace_endpoints() {
    let result = compile(MODEL);
    assert!(result.semantic_model.traces.iter().any(|t| t.from == "SA-targets" && t.to == "OI-1"), "{:?}", result.semantic_model.traces);
}

#[test]
fn written_ids_reach_the_model_on_ports_messages_scenarios_machines_and_levels() {
    let result = compile(MODEL);
    let sa = &result.ast.system_analysis[0];
    assert_eq!(sa.id, "ARCH-SA");
    assert_eq!(sa.functions[0].ports[0].id, "PORT-1");
    assert_eq!(result.ast.scenarios[0].id, "SC-1");
    assert_eq!(result.ast.scenarios[0].messages[0].id, "MSG-1");
    assert_eq!(result.ast.state_machines[0].id, "SM-1");
    let graph = elements::build(&result.ast, &result.semantic_model);
    assert!(graph.elements.iter().any(|e| e.kind == "FunctionPort" && e.id == "PORT-1"), "a port keeps its written id");
    assert!(graph.elements.iter().any(|e| e.kind == "Scenario" && e.id == "SC-1"));
    assert!(graph.elements.iter().any(|e| e.kind == "StateMachine" && e.id == "SM-1"));
}

#[test]
fn a_chain_without_involvement_is_a_warning_not_an_error() {
    let empty = MODEL.replace(r#"functional_chain "Warn path" { id: "SFC-Warn" involves: ["SF-Warn"] }"#, r#"functional_chain "Warn path" { id: "SFC-Warn" }"#);
    let result = compile(&empty);
    assert!(result.warnings.iter().any(|w| w.contains("Warn path") && w.contains("involves nothing")), "{:?}", result.warnings);
}
