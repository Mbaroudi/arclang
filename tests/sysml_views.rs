//! The SysML v2 views of a model: General, Interconnection and Action Flow,
//! drawn from the SysML v2 text the model exports.

use arclang::compiler::diagram::{build_diagrams, Diagram, EdgeKind, Node, NodeKind, ViewKind};
use arclang::compiler::{Compiler, CompilerConfig};

fn drone() -> Vec<Diagram> {
    let result = Compiler::new(CompilerConfig::default()).compile_file(std::path::Path::new("examples/aerospace/drone-endurance/drone_endurance.arc")).unwrap();
    build_diagrams(&result.ast).diagrams
}

fn find<'n>(nodes: &'n [Node], id: &str) -> Option<&'n Node> {
    nodes.iter().find_map(|n| if n.id == id { Some(n) } else { find(&n.children, id) })
}

#[test]
fn the_general_view_shows_definitions_and_parts_and_how_parts_are_made() {
    let diagrams = drone();
    // One General View per package.
    let views: Vec<&Diagram> = diagrams.iter().filter(|d| d.kind == ViewKind::Gv).collect();
    assert!(views.len() > 1 && views.iter().all(|v| v.id.starts_with("gv:") && v.title.starts_with("General View: ")), "{:?}", views.iter().map(|v| &v.id).collect::<Vec<_>>());
    let nodes: Vec<Node> = views.iter().flat_map(|v| v.nodes.clone()).collect();
    let signal = nodes.iter().find(|n| n.name == "Signal").unwrap();
    assert_eq!((signal.kind, signal.properties["stereotype"].as_str()), (NodeKind::SysmlDefinition, "port def"));
    assert_eq!(signal.compartment, ["physics : String = \"signal\""]);
    let battery = find(&nodes, "PN-BATT").unwrap();
    assert_eq!((battery.kind, battery.properties["stereotype"].as_str()), (NodeKind::SysmlPart, "part"));
    assert!(battery.compartment.contains(&"voltage : ElectricPotentialValue = 44.4 [V]".to_string()), "{:?}", battery.compartment);
    let made: Vec<(&str, &str)> = views.iter().flat_map(|v| v.edges.iter()).filter(|e| e.kind == EdgeKind::Composition).map(|e| (e.source.as_str(), e.target.as_str())).collect();
    assert!(made.contains(&("LC-UAV", "LC-ENERGY")) && made.contains(&("LC-ENERGY", "LC-PACK-EXT")) && made.contains(&("PN-FC", "BC-ALT")), "{made:?}");
}

#[test]
fn an_interconnection_view_nests_the_parts_and_connects_their_ports() {
    let diagrams = drone();
    let iv = diagrams.iter().find(|d| d.id == "iv:Drone hardware").expect("the hardware's interconnection view");
    let controller = find(&iv.nodes, "PN-FC").unwrap();
    assert_eq!(controller.children.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["BC-ALT"], "the altitude loop is inside the flight controller");
    let ports: Vec<&str> = find(&iv.nodes, "PN-BATT").unwrap().ports.iter().map(|p| p.id.as_str()).collect();
    assert_eq!(ports, ["PN-BATT.p", "PN-BATT.n", "PN-BATT.mount"]);
    let plus = iv.edges.iter().find(|e| e.label == "Power harness plus").unwrap();
    assert_eq!((plus.kind, plus.source_port.as_deref(), plus.target_port.as_deref()), (EdgeKind::Connection, Some("PN-BATT.p"), Some("PN-PROP.p")));
    let into_loop = iv.edges.iter().find(|e| e.label == "Altitude to the loop").unwrap();
    assert_eq!((into_loop.source_port.as_deref(), into_loop.target_port.as_deref()), (Some("PN-FC.altitude"), Some("BC-ALT.altitude")), "a port of a nested part");
    assert_eq!(iv.edges.len(), 10);
}

#[test]
fn an_action_flow_view_draws_the_flows_between_items_and_the_steps_in_order() {
    let diagrams = drone();
    let afv = diagrams.iter().find(|d| d.id == "afv:Mission needs").expect("the action flow of the system analysis");
    let flow = afv.edges.iter().find(|e| e.kind == EdgeKind::Flow && e.label == "thrust demand").unwrap();
    assert_eq!((flow.source_port.as_deref(), flow.target_port.as_deref()), (Some("SF-HOLD.thrust"), Some("SF-THRUST.thrust")));
    let hold = find(&afv.nodes, "SF-HOLD").unwrap();
    assert_eq!(hold.ports.iter().map(|p| format!("{:?} {}", p.direction, p.name)).collect::<Vec<_>>(), ["In altitude", "Out thrust"]);
    let loop_ = find(&afv.nodes, "FC-ALT").unwrap();
    assert_eq!(loop_.children.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["step1 :> SF-ALT", "step2 :> SF-HOLD", "step3 :> SF-THRUST"]);
    let then: Vec<(&str, &str)> = afv.edges.iter().filter(|e| e.kind == EdgeKind::Succession).map(|e| (e.source.as_str(), e.target.as_str())).collect();
    assert_eq!(then, [("FC-ALT.step1", "FC-ALT.step2"), ("FC-ALT.step2", "FC-ALT.step3")]);
}

#[test]
fn the_views_of_an_operational_world_show_what_its_actors_and_entities_exchange() {
    // Activities performed by an actor and by an entity, exchanging through interactions;
    // a scenario (lifelines) and a threat, which the General View leaves out.
    let text = r#"
model W {}
operational_analysis "World" {
    actor "Driver" { id: "OA-DRV" }
    entity "Car" { id: "OA-CAR" activity "Regulate" { id: "OAC-REG" } activity "Inform" { id: "OAC-INF" } }
    operational_activity "Choose" { id: "OAC-CHOOSE" performed_by: "OA-DRV" }
    operational_activity "Watch" { id: "OAC-WATCH" performed_by: "OA-DRV" }
    interaction "wishes" { id: "OE-1" from: "OAC-CHOOSE" to: "OAC-REG" }
    interaction "state" { id: "OE-2" from: "OAC-INF" to: "OAC-WATCH" }
    interaction "need" { id: "OE-3" from: "OAC-REG" to: "OAC-INF" }
}
scenario "Drive" { participants: ["OA-DRV", "OA-CAR"] message "OA-DRV" -> "OA-CAR" "set" }
security_analysis {
    asset "Settings" { id: "AS-1" kind: "information" members: ["OAC-CHOOSE"] }
    threat "Forged settings" { id: "TH-1" level: 2 applies_to: ["AS-1"] involves: ["OA-CAR"] }
}
"#;
    let result = Compiler::new(CompilerConfig::default()).compile_string(text).unwrap();
    let set = build_diagrams(&result.ast);
    let iv = set.diagrams.iter().find(|d| d.id == "iv:World").expect("the world's interconnection view");
    let between: Vec<(&str, &str)> = iv.edges.iter().map(|e| (e.source.as_str(), e.target.as_str())).collect();
    assert_eq!(between, [("OA-DRV", "OA-CAR"), ("OA-CAR", "OA-DRV")], "the flows between the two parts; the one inside the car is not between parts");
    let afv = set.diagrams.iter().find(|d| d.id == "afv:World").expect("the world's action flow view");
    let driver = find(&afv.nodes, "OA-DRV").expect("the driver's lane");
    assert_eq!(driver.children.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["OAC-CHOOSE", "OAC-WATCH"]);
    assert_eq!(afv.edges.iter().filter(|e| e.kind == EdgeKind::Flow).count(), 3, "every interaction, between the lanes and inside one");
    let general: Vec<&Node> = set.diagrams.iter().filter(|d| d.kind == ViewKind::Gv).flat_map(|d| d.nodes.iter()).collect();
    assert!(general.iter().all(|n| !n.id.starts_with("l_") && n.id != "TH-1" && n.id != "AS-1"), "no lifeline, no threat: {:?}", general.iter().map(|n| &n.id).collect::<Vec<_>>());
    assert!(general.iter().any(|n| n.id == "OA-DRV") && general.iter().any(|n| n.id == "OA-CAR"));
}

#[test]
fn every_example_draws_its_sysml_views_without_a_dangling_end() {
    for entry in walk(std::path::Path::new("examples")) {
        let Ok(result) = Compiler::new(CompilerConfig::default()).compile_file(&entry) else { continue };
        let set = build_diagrams(&result.ast);
        for diagram in set.diagrams.iter().filter(|d| matches!(d.kind, ViewKind::Gv | ViewKind::Iv | ViewKind::Afv)) {
            let problems = diagram.validate();
            assert!(problems.is_empty(), "{} {}: {problems:?}", entry.display(), diagram.id);
        }
        assert!(!set.diagnostics.iter().any(|d| d.contains("does not read back")), "{}: {:?}", entry.display(), set.diagnostics);
    }
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else if path.extension().is_some_and(|e| e == "arc") {
            out.push(path);
        }
    }
    out.sort();
    out
}
