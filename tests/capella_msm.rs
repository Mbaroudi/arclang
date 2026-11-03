//! The state machine diagrams of the Capella export: every machine is drawn
//! in the `.aird` as Capella draws it (a Mode State Machine on each of its
//! top-level regions), with the place and size of every box in the GMF
//! notation, so that Capella shows it laid out and the Simulink connector
//! reads the chart's layout from it; a machine written without an owner
//! still goes to Capella.

use arclang::compiler::capella_export::{export, Exported};
use arclang::compiler::simulink_connector::export as connector;
use arclang::compiler::simulink_connector::m2m::{model::Model, Options, Strategy};
use arclang::compiler::{capella_xmi, Compiler, CompilerConfig};
use std::path::Path;

const MACHINES: &str = r#"model T { name: "MSM test" }
system_analysis "SA" {
    id: "ARCH-SA"
    function "Sense" { id: "SF-Sense" }
    system_component "Controller" { id: "SC-Ctrl" allocated_functions: ["SF-Sense"] }
}
state_machine "Modes" {
    id: "SM-1"
    owner: "SC-Ctrl"
    initial: "Off"
    state "Off" { id: "ST-Off" }
    state "Running" {
        id: "ST-Run"
        initial: "Idle"
        state "Idle" { id: "ST-Idle" }
        state "Busy" { id: "ST-Busy" }
        state "Done" { id: "ST-Done" kind: "final" }
        transition "Idle" -> "Busy" { trigger: "go" }
        transition "Busy" -> "Done" { trigger: "finish" guard: "ready" }
    }
    state "Check" { id: "ST-Check" kind: "choice" }
    transition "Off" -> "Running" { trigger: "power" }
    transition "Running" -> "Check" { trigger: "stop" }
    transition "Check" -> "Off" { trigger: "" guard: "cold" }
    transition "Check" -> "Running" { trigger: "" guard: "warm" }
    transition "Off" -> "Off" { trigger: "tick" }
}
state_machine "Two regions" {
    id: "SM-2"
    owner: "SC-Ctrl"
    region "Power" { id: "RG-1" initial: "On" state "On" { id: "ST-On" } state "Off2" { id: "ST-Off2" } transition "On" -> "Off2" { trigger: "cut" } transition "Off2" -> "On" { trigger: "restore" } }
    region "Link" { initial: "Linked" state "Linked" { id: "ST-Linked" } state "Lost" { id: "ST-Lost" } transition "Linked" -> "Lost" { trigger: "drop" } }
}
"#;

fn exported(source: &str, stem: &str) -> Exported {
    let ast = Compiler::new(CompilerConfig { optimization_level: 0, target: "json".to_string() }).compile_string(source).expect("compiles").ast;
    export(&ast, stem)
}

fn example(path: &str, stem: &str) -> Exported {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let ast = Compiler::new(CompilerConfig::default()).compile_file(&path).expect("compiles").ast;
    export(&ast, stem)
}

fn model(exported: &Exported, stem: &str) -> Model {
    Model::from_strings(&[(&format!("{stem}.capella"), exported.capella.as_str()), (&format!("{stem}.aird"), exported.aird.as_str())]).unwrap()
}

/// The notation node of a Sirius element, and its stored bounds.
fn bounds(m: &Model, sirius: usize) -> Option<(i32, i32, i32, i32)> {
    let uid = m.attr(sirius, "uid")?;
    let node = (0..m.elements.len()).find(|&n| m.attr(n, "element") == Some(uid) && m.class(n) == "Node")?;
    let at = m.owned(node, "layoutConstraint").into_iter().next()?;
    let get = |k: &str| m.attr(at, k).and_then(|v| v.parse().ok());
    Some((get("x")?, get("y")?, get("width")?, get("height")?))
}

#[test]
fn every_region_of_every_machine_has_a_mode_state_machine_diagram_with_its_layout() {
    let exported = exported(MACHINES, "msm");
    let m = model(&exported, "msm");
    let diagrams: Vec<usize> = (0..m.elements.len()).filter(|&d| m.class(d) == "DSemanticDiagram").collect();
    // Modes has one region, Two regions has two: one diagram each.
    assert_eq!(diagrams.len(), 3);
    for &d in &diagrams {
        let region = m.reference(d, "target").expect("a diagram is on a region");
        assert!(m.is(region, "Region") && m.container(region).is_some_and(|c| m.is(c, "StateMachine")));
        assert!(m.owned(d, "ownedAnnotationEntries").iter().any(|&a| m.attr(a, "source") == Some("GMF_DIAGRAMS")), "the diagram has its GMF notation");
    }
    let descriptors: Vec<String> = (0..m.elements.len()).filter(|&e| m.class(e) == "DRepresentationDescriptor").filter_map(|e| m.name(e)).collect();
    assert!(descriptors.contains(&"[MSM] Modes".to_string()) && descriptors.contains(&"[MSM] Two regions - Link".to_string()), "{descriptors:?}");
    // Every state, pseudo state and nested region is drawn, with a place and a size.
    let drawn = |e: usize| m.referencers(e, "target").into_iter().filter(|&s| matches!(m.class(s), "DNode" | "DNodeContainer")).collect::<Vec<_>>();
    let mut states = 0;
    for e in 0..m.elements.len() {
        let is_state = m.is(e, "AbstractState") || m.is(e, "Region") && m.container(e).is_some_and(|c| m.is(c, "State"));
        if !is_state || m.attr(e, "href").is_some() {
            continue;
        }
        states += 1;
        let views = drawn(e);
        assert_eq!(views.len(), 1, "{:?} is drawn once", m.name(e));
        let (_, _, w, h) = bounds(&m, views[0]).unwrap_or_else(|| panic!("{:?} has bounds", m.name(e)));
        assert!(w > 0 && h > 0, "{:?} has a size", m.name(e));
    }
    assert!(states >= 14, "{states} states, pseudo states and regions");
    // A sub-state is inside its state.
    let named = |n: &str| (0..m.elements.len()).find(|&e| m.name(e).as_deref() == Some(n) && m.is(e, "AbstractState")).unwrap();
    let running = bounds(&m, drawn(named("Running"))[0]).unwrap();
    let busy = bounds(&m, drawn(named("Busy"))[0]).unwrap();
    assert!(6 + busy.0 + busy.2 <= running.2 && 34 + busy.1 + busy.3 <= running.3, "Busy {busy:?} fits in Running {running:?}");
    // An edge per transition, between the drawn ends.
    let transitions: Vec<usize> = (0..m.elements.len()).filter(|&e| m.class(e) == "StateTransition" && m.attr(e, "href").is_none()).collect();
    for &t in &transitions {
        let edges: Vec<usize> = m.referencers(t, "target").into_iter().filter(|&s| m.class(s) == "DEdge").collect();
        assert_eq!(edges.len(), 1, "transition {:?} is drawn once", m.id(t));
        let uid = m.attr(edges[0], "uid").unwrap();
        assert!((0..m.elements.len()).any(|n| m.class(n) == "Edge" && m.attr(n, "element") == Some(uid)), "its notation edge");
    }
    // Modes: 7 declared and 2 from initial states; Two regions: 3 declared and 2 initial.
    assert_eq!(transitions.len(), 7 + 2 + 3 + 2, "the declared transitions and one per initial state");
}

#[test]
fn a_region_names_the_states_it_shows() {
    let exported = exported(MACHINES, "msm");
    let m = model(&exported, "msm");
    for region in (0..m.elements.len()).filter(|&e| m.class(e) == "Region" && m.attr(e, "href").is_none()) {
        assert_eq!(m.refs(region, "involvedStates"), m.owned(region, "ownedStates"), "{:?}", m.id(region));
    }
}

#[test]
fn the_connector_reads_the_layout_of_the_exported_diagrams() {
    let exported = exported(MACHINES, "msm");
    let m = model(&exported, "msm");
    let charts = connector::export(&m, Strategy::StateMachines, &[], &Options::default()).expect("every region has its diagram");
    assert!(charts.notes.iter().all(|n| !n.contains("ArcLang lays its chart out")), "{:?}", charts.notes);
    assert!(charts.script.contains("Stateflow.State(parent)") && charts.script.contains("Position"), "{}", charts.script);
    // Without its diagrams the same model is drawn on the connector's grid: the
    // positions come from the diagrams.
    let bare = Model::from_strings(&[("msm.capella", exported.capella.as_str())]).unwrap();
    let grid = connector::export(&bare, Strategy::StateMachines, &[], &Options { draw_without_diagram: true, ..Options::default() }).unwrap();
    assert_ne!(grid.script, charts.script);
    assert!(connector::export(&bare, Strategy::StateMachines, &[], &Options::default()).is_err());
}

#[test]
fn a_machine_without_an_owner_goes_to_the_structure_of_the_system() {
    // The cruise control's operating modes name functions of four components:
    // no component is theirs, and the system analysis has actors only.
    let exported = example("examples/automotive/adaptive-cruise/adaptive_cruise_control.arc", "acc");
    assert!(exported.notes.iter().any(|n| n.contains("'ACC operating modes' has no owner") && n.contains("'Structure'")), "{:?}", exported.notes);
    assert!(exported.notes.iter().all(|n| !n.contains("left out: no component to own it")), "{:?}", exported.notes);
    let m = model(&exported, "acc");
    let machine = (0..m.elements.len()).find(|&e| m.class(e) == "StateMachine" && m.name(e).as_deref() == Some("ACC operating modes")).expect("the machine is exported");
    assert_eq!(m.class(m.container(machine).unwrap()), "SystemComponentPkg");
    let charts = connector::export(&m, Strategy::StateMachines, &[], &Options::default()).expect("the machine reaches the connector");
    for mode in ["Off", "Standby", "Cruising", "Following", "Overridden", "Degraded"] {
        assert!(charts.script.contains(&format!("'{mode}'")), "{mode} in\n{}", charts.script);
    }
}

#[test]
fn a_machine_without_an_owner_goes_to_the_component_performing_what_it_names() {
    let source = r#"model T { name: "t" }
system_analysis "SA" {
    function "Watch" { id: "SF-Watch" }
    function "Stop" { id: "SF-Stop" }
    system_component "Plant" { id: "SC-Plant" }
    system_component "Guard" { id: "SC-Guard" allocated_functions: ["SF-Watch", "SF-Stop"] }
}
state_machine "Guarding" {
    initial: "Watching"
    mode Watching {}
    mode Stopped {}
    transition Watching -> Stopped { trigger: "SF-Watch" action: ["SF-Stop"] }
}
"#;
    let exported = exported(source, "t");
    assert!(exported.notes.iter().any(|n| n.contains("'Guarding' has no owner: it goes to 'Guard'")), "{:?}", exported.notes);
    let m = model(&exported, "t");
    let machine = (0..m.elements.len()).find(|&e| m.class(e) == "StateMachine").unwrap();
    assert_eq!(m.id(m.container(machine).unwrap()).as_deref(), Some("SC-Guard"));
}

#[test]
fn the_import_reads_the_machines_back_and_leaves_their_diagrams() {
    let exported = exported(MACHINES, "msm");
    let dir = std::env::temp_dir().join(format!("arclang-msm-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("msm.capella"), &exported.capella).unwrap();
    std::fs::write(dir.join("msm.aird"), &exported.aird).unwrap();
    let imported = capella_xmi::import_file(&dir.join("msm.capella")).unwrap_or_else(|f| panic!("{}", f.message));
    std::fs::remove_dir_all(&dir).ok();
    assert_eq!(imported.counts.get("diagrams of state machines (each machine is a view of its own)").copied(), Some(3), "{:?}", imported.counts);
    let again = Compiler::new(CompilerConfig { optimization_level: 0, target: "json".to_string() }).compile_string(&imported.source).expect("the import compiles").ast;
    let original = Compiler::new(CompilerConfig { optimization_level: 0, target: "json".to_string() }).compile_string(MACHINES).unwrap().ast;
    let shape = |ast: &arclang::compiler::ast::Model| ast.state_machines.iter().map(|m| (m.id.clone(), m.states.len(), m.transitions.len(), m.regions.len())).collect::<Vec<_>>();
    assert_eq!(shape(&again), shape(&original));
    // A second export of the import writes the same machines and the same diagrams.
    let second = export(&again, "msm");
    let machines = |capella: &str| capella.split("<ownedStateMachines").skip(1).map(|m| m.split("</ownedStateMachines>").next().unwrap_or("").to_string()).collect::<Vec<_>>();
    assert_eq!(machines(&second.capella), machines(&exported.capella));
    assert_eq!(second.aird, exported.aird);
}


#[test]
fn a_state_names_the_states_of_its_regions() {
    // Capella's DWF_SM_03: a state's referencedStates are its regions' involvedStates.
    let exported = exported(MACHINES, "msm");
    let m = model(&exported, "msm");
    let holders: Vec<usize> = (0..m.elements.len()).filter(|&e| matches!(m.class(e), "State" | "Mode") && !m.owned(e, "ownedRegions").is_empty()).collect();
    assert!(!holders.is_empty());
    for state in holders {
        let involved: Vec<usize> = m.owned(state, "ownedRegions").into_iter().flat_map(|r| m.refs(r, "involvedStates")).collect();
        assert_eq!(m.refs(state, "referencedStates"), involved, "{:?}", m.name(state));
    }
}

#[test]
fn a_diagram_names_an_element_by_its_id_as_it_is() {
    // EMF looks a fragment up without decoding it: Capella resolves `#Two regions.Link`,
    // not `#Two%20regions.Link` (the connector then found no diagram for the region).
    let exported = exported(MACHINES, "msm");
    assert!(exported.aird.contains("msm.capella#Two regions.Link\""), "the region's own id");
    assert!(!exported.aird.contains("#Two%20regions"));
}
