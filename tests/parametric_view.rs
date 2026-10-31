//! The parametric view: each constraint bound to the values it reads, with
//! the value of each side and its verdict.

use arclang::compiler::diagram::{build_diagrams, Diagram, EdgeKind, NodeKind, ViewKind};
use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"
model "Charger" {}

system_analysis "Needs" {
    function "Detect" { id: "SF-DET" latency: 10 ms }
    function "Stop" { id: "SF-STOP" latency: 40 ms }
    functional_chain "Emergency stop" { id: "FC-STOP" involves: ["SF-DET", "SF-STOP"] latency_budget: 100 ms }
}

physical_architecture "Hardware" {
    node "Cabinet" { id: "PN-PWR" voltage: 1000 V rated_current: 500 A power: 350 kW }
}

constraint "Stop within budget" { id: "CST-STOP" assert: sum("FC-STOP", latency) <= "FC-STOP".latency_budget }
constraint "Rating covers the power" { id: "CST-PWR" assert: "PN-PWR".voltage * "PN-PWR".rated_current >= "PN-PWR".power }
"#;

fn view(text: &str) -> Diagram {
    let result = Compiler::new(CompilerConfig::default()).compile_string(text).unwrap_or_else(|e| panic!("{e}"));
    build_diagrams(&result.ast).diagrams.into_iter().find(|d| d.kind == ViewKind::Par).expect("a parametric view")
}

#[test]
fn each_constraint_is_bound_to_the_values_it_reads() {
    let diagram = view(MODEL);
    assert_eq!(diagram.title, "Parametric");
    let stop = diagram.nodes.iter().find(|n| n.id == "CST-STOP").unwrap();
    assert_eq!(stop.kind, NodeKind::Constraint);
    assert_eq!(stop.compartment, ["sum(\"FC-STOP\", latency) <= \"FC-STOP\".latency_budget", "50 ms against 100 ms"]);
    assert_eq!(stop.properties["verdict"], "pass");
    let cabinet = diagram.nodes.iter().find(|n| n.id == "PN-PWR").unwrap();
    assert_eq!(cabinet.kind, NodeKind::PhysicalNode);
    let values: Vec<(&str, &str)> = cabinet.children.iter().map(|c| (c.id.as_str(), c.compartment[0].as_str())).collect();
    assert_eq!(values, [("PN-PWR.voltage", "= 1000 V"), ("PN-PWR.rated_current", "= 500 A"), ("PN-PWR.power", "= 350 kW")]);
    assert!(cabinet.children.iter().all(|c| c.kind == NodeKind::ValueProperty));
    let chain = diagram.nodes.iter().find(|n| n.id == "FC-STOP").unwrap();
    assert_eq!(chain.kind, NodeKind::FunctionalChain);
    let mut bound: Vec<(&str, &str, &str)> = diagram.edges.iter().filter(|e| e.target == "CST-STOP").map(|e| (e.source.as_str(), e.label.as_str(), e.target.as_str())).collect();
    bound.sort();
    assert_eq!(bound, [("FC-STOP.latency_budget", "", "CST-STOP"), ("SF-DET.latency", "sum", "CST-STOP"), ("SF-STOP.latency", "sum", "CST-STOP")]);
    assert!(diagram.edges.iter().all(|e| e.kind == EdgeKind::Binding));
}

#[test]
fn a_violated_constraint_says_so() {
    let diagram = view(&MODEL.replace("latency_budget: 100 ms", "latency_budget: 30 ms"));
    let stop = diagram.nodes.iter().find(|n| n.id == "CST-STOP").unwrap();
    assert_eq!(stop.properties["verdict"], "fail");
    assert_eq!(stop.compartment[1], "50 ms against 30 ms");
}

#[test]
fn a_value_read_by_two_constraints_is_drawn_once() {
    let text = format!("{MODEL}\nconstraint \"Detect is quick\" {{ id: \"CST-DET\" assert: \"SF-DET\".latency <= 20 ms }}\n");
    let diagram = view(&text);
    let detect = diagram.nodes.iter().find(|n| n.id == "SF-DET").unwrap();
    assert_eq!(detect.children.len(), 1);
    assert_eq!(diagram.edges.iter().filter(|e| e.source == "SF-DET.latency").count(), 2);
}

#[test]
fn a_model_without_constraints_has_no_parametric_view() {
    let result = Compiler::new(CompilerConfig::default()).compile_string("model \"M\" {}\nphysical_architecture \"P\" { node \"N\" { id: \"PN-N\" } }\n").unwrap();
    assert!(build_diagrams(&result.ast).diagrams.iter().all(|d| d.kind != ViewKind::Par));
}
