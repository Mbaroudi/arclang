//! What ties the layers together reaches the viewer whole: every
//! realization, allocation, deployment, satisfaction and verification the
//! compiler knows (not only the `trace` declarations), each with the layer
//! and the identity of its two ends, and the layer of every drawn element.

use arclang::compiler::diagram::{build_diagrams, build_linked_diagrams, html, DiagramSet, Link, Node};
use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"model T { name: "t" }
operational_analysis "OA" {
    operational_entity "Vehicle" {
        id: "OE-1"
        operational_activity "Monitor Road" { id: "OA-1" }
    }
}
system_analysis "SA" {
    requirement "REQ-1" { title: "Detect obstacles" }
    function "Sense" { id: "SF-Sense" }
}
logical_architecture "LA" {
    component "Sensors" {
        id: "LC-Sensors"
        function "Sense targets" { id: "LF-Sense" realizes: "SF-Sense" }
    }
}
physical_architecture "PA" {
    node "ECU" {
        id: "PN-ECU"
        deploys "LC-Sensors"
        behavior_component "Sensor driver" {
            id: "PC-Sensor"
            function "Sense targets" { id: "PF-Sense" realizes: "LF-Sense" }
        }
    }
}
trace "SF-Sense" realizes "OA-1"
trace "LC-Sensors" satisfies "REQ-1"
"#;

fn linked(source: &str) -> DiagramSet {
    let result = Compiler::new(CompilerConfig::default()).compile_string(source).expect("test model compiles");
    build_linked_diagrams(&result)
}

fn found<'a>(set: &'a DiagramSet, kind: &str, source: &str, target: &str) -> &'a Link {
    set.links
        .iter()
        .find(|l| l.kind == kind && l.source == source && l.target == target)
        .unwrap_or_else(|| panic!("no link {source} {kind} {target} in {:#?}", set.links))
}

fn node<'a>(set: &'a DiagramSet, diagram: &str, id: &str) -> &'a Node {
    let diagram = set.diagrams.iter().find(|d| d.id == diagram).unwrap_or_else(|| panic!("no diagram {diagram}"));
    let mut stack: Vec<&Node> = diagram.nodes.iter().collect();
    while let Some(node) = stack.pop() {
        if node.id == id {
            return node;
        }
        stack.extend(node.children.iter());
    }
    panic!("no node {id} in {}", diagram.id)
}

#[test]
fn a_realization_written_as_an_attribute_is_a_link_between_two_layers() {
    let set = linked(MODEL);

    let logical = found(&set, "realizes", "LF-Sense", "SF-Sense");
    assert_eq!((logical.source_layer, logical.target_layer), (Some("LA"), Some("SA")));
    assert_eq!(logical.back.as_deref(), Some("realized by"));
    let physical = found(&set, "realizes", "PF-Sense", "LF-Sense");
    assert_eq!((physical.source_layer, physical.target_layer), (Some("PA"), Some("LA")));
}

#[test]
fn a_deployment_is_a_link_from_the_component_to_the_node_hosting_it() {
    let set = linked(MODEL);

    let deployed = found(&set, "deployed on", "LC-Sensors", "PN-ECU");
    assert_eq!((deployed.source_layer, deployed.target_layer), (Some("LA"), Some("PA")));
    assert_eq!(deployed.back.as_deref(), Some("hosts"));
}

#[test]
fn a_trace_is_one_link_and_gains_the_layers_and_identities_of_its_ends() {
    let set = linked(MODEL);

    let realizes: Vec<&Link> = set.links.iter().filter(|l| l.source == "SF-Sense" && l.target == "OA-1").collect();
    assert_eq!(realizes.len(), 1, "the trace is not said twice: {realizes:#?}");
    assert_eq!((realizes[0].source_layer, realizes[0].target_layer), (Some("SA"), Some("OA")));
    assert_eq!(realizes[0].source_uuid.as_deref(), Some(node(&set, "sab", "SF-Sense").uuid.as_str()));
    assert_eq!(realizes[0].target_uuid.as_deref(), Some(node(&set, "oab", "OA-1").uuid.as_str()));

    let satisfies = found(&set, "satisfies", "LC-Sensors", "REQ-1");
    assert_eq!(satisfies.source_layer, Some("LA"));
    assert!(satisfies.target_uuid.is_some(), "a requirement no view draws still has its identity");
}

#[test]
fn the_traces_come_first_in_the_order_they_were_written() {
    let set = linked(MODEL);
    let traces = build_diagrams(&Compiler::new(CompilerConfig::default()).compile_string(MODEL).unwrap().ast).links;

    assert_eq!(traces.len(), 2);
    for (written, linked) in traces.iter().zip(&set.links) {
        assert_eq!((&written.kind, &written.source, &written.target), (&linked.kind, &linked.source, &linked.target));
    }
}

#[test]
fn what_a_view_already_shows_by_nesting_is_not_a_link() {
    let set = linked(MODEL);

    assert!(
        !set.links.iter().any(|l| l.source == "LF-Sense" && l.target == "LC-Sensors"),
        "a function drawn inside its component needs no link to it: {:#?}",
        set.links
    );
}

#[test]
fn every_drawn_element_says_its_layer() {
    let set = linked(MODEL);

    assert_eq!(node(&set, "oab", "OA-1").layer, Some("OA"));
    assert_eq!(node(&set, "sab", "SF-Sense").layer, Some("SA"));
    assert_eq!(node(&set, "lab", "LF-Sense").layer, Some("LA"));
    assert_eq!(node(&set, "pab", "PF-Sense").layer, Some("PA"));
    assert_eq!(node(&set, "pab", "PN-ECU").layer, Some("PA"));
}

#[test]
fn the_viewer_is_given_the_layers() {
    let set = linked(MODEL);
    let payload = html::viewer_payload(&set, "t");

    let lab = payload["graphs"].as_array().unwrap().iter().find(|g| g["id"] == "lab").expect("the logical view");
    let component = lab["children"].as_array().unwrap().iter().find(|n| n["id"] == "LC-Sensors").expect("the component");
    assert_eq!(component["arc"]["layer"], "LA");
    let link = payload["links"].as_array().unwrap().iter().find(|l| l["source"] == "LF-Sense" && l["target"] == "SF-Sense").expect("the realization");
    assert_eq!(link["source_layer"], "LA");
    assert_eq!(link["target_layer"], "SA");
    assert_eq!(link["back"], "realized by");
}

#[test]
fn the_diagrams_alone_stay_as_they_were() {
    let result = Compiler::new(CompilerConfig::default()).compile_string(MODEL).unwrap();
    let set = build_diagrams(&result.ast);

    assert!(set.links.iter().all(|l| l.back.is_none() && l.source_layer.is_none() && l.source_uuid.is_none()));
    let text = serde_json::to_string(&html::viewer_payload(&set, "t")).unwrap();
    assert!(!text.contains("\"layer\""), "no layer is written where none was asked for");
}

/// Elements that state no id: the views name a function by its component,
/// and two elements of different kinds may share a name.
const UNNAMED: &str = r#"model T { name: "t" }
system_analysis "SA" {
    function "Sense" { }
    actor "Radar" { }
}
logical_architecture "LA" {
    component "Radar" {
        function "Acquire frames" { realizes: "Sense" }
    }
    component "Camera" {
        function "Acquire frames" { }
    }
}
"#;

#[test]
fn an_element_without_an_id_is_still_found_under_the_component_drawing_it() {
    let set = linked(UNNAMED);

    assert_eq!(node(&set, "lab", "Radar").layer, Some("LA"));
    let sab = set.diagrams.iter().find(|d| d.id == "sab").expect("the system view");
    let actor = sab.nodes.iter().find(|n| n.name == "Radar").expect("the actor is drawn");
    assert_eq!(actor.layer, Some("SA"), "the actor, not the component of the same name");
    assert_eq!(node(&set, "lab", "Radar.Acquire frames").layer, Some("LA"));
    assert_eq!(node(&set, "lab", "Camera.Acquire frames").layer, Some("LA"));
    let realizes = found(&set, "realizes", "Radar.Acquire frames", "SF-Sense");
    assert_eq!((realizes.source_layer, realizes.target_layer), (Some("LA"), Some("SA")));
}

#[test]
fn the_explorer_page_of_a_compiled_model_carries_the_layers_and_the_links() {
    use arclang::compiler::arcviz_explorer::{generate_explorer_html, generate_explorer_html_linked};
    let result = Compiler::new(CompilerConfig::default()).compile_string(MODEL).expect("test model compiles");

    let (linked, _) = generate_explorer_html_linked(&result, &result.semantic_model, None).expect("the explorer renders");
    assert!(linked.contains(r#""layer":"LA""#), "the views of the page say the layers");
    assert!(linked.contains(r#""kind":"deployed on""#) && linked.contains(r#""back":"hosts""#), "the deployment is a link of the page");

    let (plain, _) = generate_explorer_html(&result.semantic_model, &result.ast).expect("the explorer renders");
    assert!(!plain.contains(r#""kind":"deployed on""#), "from the syntax tree alone, the page has the traces only");
}
