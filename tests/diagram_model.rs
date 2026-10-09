//! Behavioural tests for the viewpoint diagram model (`compiler::diagram`).
//!
//! The diagram model is what a renderer draws: one diagram per Arcadia
//! viewpoint, functions nested in the components they are allocated to,
//! exchanges bound to ports. These tests pin that contract; the golden
//! files in `tests/fixtures/diagrams/` pin the full output of the flagship
//! models (regenerate with `UPDATE_GOLDEN=1 cargo test --test diagram_model`).

use arclang::compiler::diagram::{
    build_diagrams, elk, Diagram, DiagramSet, EdgeKind, Node, NodeKind, PortDirection, ViewKind,
};
use arclang::compiler::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};

fn diagrams_of(source: &str) -> DiagramSet {
    let result = Compiler::new(CompilerConfig::default())
        .compile_string(source)
        .expect("test model compiles");
    build_diagrams(&result.ast)
}

fn view(set: &DiagramSet, kind: ViewKind) -> &Diagram {
    set.diagrams
        .iter()
        .find(|d| d.kind == kind)
        .unwrap_or_else(|| {
            panic!(
                "no {kind:?} diagram in {:?}",
                set.diagrams.iter().map(|d| d.kind).collect::<Vec<_>>()
            )
        })
}

fn find<'a>(nodes: &'a [Node], name: &str) -> Option<&'a Node> {
    nodes.iter().find_map(|n| {
        if n.name == name {
            Some(n)
        } else {
            find(&n.children, name)
        }
    })
}

fn node<'a>(diagram: &'a Diagram, name: &str) -> &'a Node {
    find(&diagram.nodes, name)
        .unwrap_or_else(|| panic!("node '{name}' not in {:?} diagram", diagram.kind))
}

const LOGICAL: &str = r#"
system_analysis SA {
  function Sense {
    outputs: ["raw"]
  }
  function Decide {
    inputs: ["raw"]
    outputs: ["cmd"]
  }
  functional_exchange SenseToDecide {
    from: Sense
    to: Decide
    exchange_item: "raw"
  }
}
logical_architecture LA {
  component Sensor {
    interface_out DataOut {
      name: "IData"
      protocol: "CAN"
    }
    allocated_function: "Sense"
  }
  component Controller {
    interface_in DataIn {
      name: "IData"
    }
    allocated_function: "Decide"
  }
  component_exchange "SensorToController" {
    from_port: "Sensor.DataOut"
    to_port: "Controller.DataIn"
    exchange_item: "raw"
    label: "Raw Data"
  }
}
"#;

#[test]
fn allocated_functions_are_nested_inside_their_component() {
    let set = diagrams_of(LOGICAL);
    let lab = view(&set, ViewKind::Lab);

    let sensor = node(lab, "Sensor");
    assert_eq!(sensor.kind, NodeKind::LogicalComponent);
    let sense = sensor
        .children
        .iter()
        .find(|c| c.name == "Sense")
        .expect("Sense nested in Sensor");
    assert_eq!(sense.kind, NodeKind::Function);
    assert!(
        lab.nodes.iter().all(|n| n.kind != NodeKind::Function),
        "no function floats at top level"
    );
}

#[test]
fn component_exchange_is_bound_to_declared_ports() {
    let set = diagrams_of(LOGICAL);
    let lab = view(&set, ViewKind::Lab);

    let edge = lab
        .edges
        .iter()
        .find(|e| e.kind == EdgeKind::ComponentExchange)
        .expect("component exchange");
    let sensor = node(lab, "Sensor");
    let controller = node(lab, "Controller");
    let out = sensor
        .ports
        .iter()
        .find(|p| p.name == "DataOut")
        .expect("DataOut port");
    let input = controller
        .ports
        .iter()
        .find(|p| p.name == "DataIn")
        .expect("DataIn port");

    assert_eq!(edge.source, sensor.id);
    assert_eq!(edge.target, controller.id);
    assert_eq!(edge.source_port.as_deref(), Some(out.id.as_str()));
    assert_eq!(edge.target_port.as_deref(), Some(input.id.as_str()));
    assert_eq!(out.direction, PortDirection::Out);
    assert_eq!(input.direction, PortDirection::In);
    assert_eq!(out.interface.as_deref(), Some("IData"));
    assert_eq!(out.protocol.as_deref(), Some("CAN"));
    assert_eq!(edge.label, "Raw Data");
    assert_eq!(edge.exchange_item.as_deref(), Some("raw"));
    assert!(
        set.diagnostics.is_empty(),
        "clean model, got {:?}",
        set.diagnostics
    );
}

#[test]
fn functional_exchange_connects_function_ports_declared_as_inputs_and_outputs() {
    let set = diagrams_of(LOGICAL);
    let lab = view(&set, ViewKind::Lab);

    let edge = lab
        .edges
        .iter()
        .find(|e| e.kind == EdgeKind::FunctionalExchange)
        .expect("functional exchange shown in LAB");
    let sense = node(lab, "Sense");
    let decide = node(lab, "Decide");
    let out = sense
        .ports
        .iter()
        .find(|p| p.name == "raw" && p.direction == PortDirection::Out)
        .expect("out port");
    let input = decide
        .ports
        .iter()
        .find(|p| p.name == "raw" && p.direction == PortDirection::In)
        .expect("in port");

    assert_eq!(edge.source_port.as_deref(), Some(out.id.as_str()));
    assert_eq!(edge.target_port.as_deref(), Some(input.id.as_str()));
    assert!(
        !out.synthesized && !input.synthesized,
        "ports come from inputs/outputs declarations"
    );
}

#[test]
fn undeclared_port_is_a_diagnostic_and_never_invented() {
    let source = LOGICAL.replace("Controller.DataIn", "Controller.Typo");
    let set = diagrams_of(&source);
    let lab = view(&set, ViewKind::Lab);

    let edge = lab
        .edges
        .iter()
        .find(|e| e.kind == EdgeKind::ComponentExchange)
        .expect("edge still drawn");
    assert_eq!(edge.target, node(lab, "Controller").id);
    assert_eq!(edge.target_port, None, "no port is fabricated");
    assert!(node(lab, "Controller")
        .ports
        .iter()
        .all(|p| p.name != "Typo"));
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.contains("Typo") && d.contains("Controller")),
        "expected a diagnostic naming the missing port, got {:?}",
        set.diagnostics
    );
}

#[test]
fn exchange_item_absent_from_declared_outputs_is_reported_and_port_marked_synthesized() {
    let source = LOGICAL.replace(
        r#"exchange_item: "raw"
  }
}
logical"#,
        r#"exchange_item: "other"
  }
}
logical"#,
    );
    let set = diagrams_of(&source);
    let lab = view(&set, ViewKind::Lab);

    let port = node(lab, "Sense")
        .ports
        .iter()
        .find(|p| p.name == "other")
        .expect("port for the exchange");
    assert!(port.synthesized);
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.contains("SenseToDecide") && d.contains("other")),
        "got {:?}",
        set.diagnostics
    );
}

#[test]
fn function_allocated_to_two_components_is_reported_and_drawn_once() {
    let source = LOGICAL.replace(
        r#"allocated_function: "Decide""#,
        r#"allocated_function: "Sense""#,
    );
    let set = diagrams_of(&source);
    let lab = view(&set, ViewKind::Lab);

    fn count(nodes: &[Node], name: &str) -> usize {
        nodes
            .iter()
            .map(|n| usize::from(n.name == name) + count(&n.children, name))
            .sum()
    }
    assert_eq!(count(&lab.nodes, "Sense"), 1);
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.contains("Sense") && d.contains("Sensor") && d.contains("Controller")),
        "got {:?}",
        set.diagnostics
    );
}

#[test]
fn system_view_nests_functions_in_the_system_and_keeps_actors_outside() {
    let set = diagrams_of(
        r#"
system_analysis SA {
  actor Driver {
    description: "operator"
  }
  function Sense {
    outputs: ["raw"]
  }
}
"#,
    );
    let sab = view(&set, ViewKind::Sab);

    let system = sab
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::System)
        .expect("system box");
    assert!(system
        .children
        .iter()
        .any(|c| c.name == "Sense" && c.kind == NodeKind::Function));
    let driver = node(sab, "Driver");
    assert_eq!(driver.kind, NodeKind::SystemActor);
    assert!(
        sab.nodes.iter().any(|n| n.id == driver.id),
        "actor is a top-level node"
    );
}

#[test]
fn operational_view_nests_activities_and_links_them_by_qualified_name() {
    let set = diagrams_of(
        r#"
operational_analysis OA {
  actor Driver {
    description: "driver"
  }
  entity Vehicle {
    activity Monitor {
      description: "observe"
    }
  }
  interaction Commands {
    from: Driver
    to: Vehicle.Monitor
  }
}
"#,
    );
    let oab = view(&set, ViewKind::Oab);

    let vehicle = node(oab, "Vehicle");
    assert_eq!(vehicle.kind, NodeKind::OperationalEntity);
    let monitor = vehicle
        .children
        .iter()
        .find(|c| c.name == "Monitor")
        .expect("activity nested");
    assert_eq!(monitor.kind, NodeKind::OperationalActivity);
    assert_eq!(node(oab, "Driver").kind, NodeKind::OperationalActor);

    let edge = oab
        .edges
        .iter()
        .find(|e| e.kind == EdgeKind::Interaction)
        .expect("interaction");
    assert_eq!(edge.source, node(oab, "Driver").id);
    assert_eq!(edge.target, monitor.id);
    assert_eq!(edge.label, "Commands");
}

#[test]
fn physical_view_nests_behavior_components_and_records_what_they_realize() {
    let source = format!(
        "{LOGICAL}{}",
        r#"
physical_architecture PA {
  node EcuA {
    behavior_component SensorSw {
      allocated_component: "Sensor"
    }
  }
  node EcuB {
    behavior_component ControlSw {
      allocated_component: "Controller"
    }
  }
  link Bus {
    protocol: "CAN"
    from: "EcuA"
    to: "EcuB"
    bandwidth: "500 Kbps"
  }
}
"#
    );
    let set = diagrams_of(&source);
    let pab = view(&set, ViewKind::Pab);

    let ecu = node(pab, "EcuA");
    assert_eq!(ecu.kind, NodeKind::PhysicalNode);
    let sw = ecu
        .children
        .iter()
        .find(|c| c.name == "SensorSw")
        .expect("behavior component nested");
    assert_eq!(sw.kind, NodeKind::BehaviorComponent);
    assert_eq!(sw.realizes, vec!["Sensor".to_string()]);

    let link = pab
        .edges
        .iter()
        .find(|e| e.kind == EdgeKind::PhysicalLink)
        .expect("physical link");
    assert_eq!(link.source, ecu.id);
    assert_eq!(link.label, "Bus");
    assert_eq!(
        link.properties.get("protocol").map(String::as_str),
        Some("CAN")
    );
}

#[test]
fn nodes_sharing_an_identity_stay_distinct_and_the_collision_is_reported() {
    let set = diagrams_of(
        r#"
physical_architecture PA {
  node Ecu {
    behavior_component BrakingModule {
      id: "BC-1"
    }
    behavior_component BrakeActuation {
      id: "BC-1"
    }
  }
}
"#,
    );
    let pab = view(&set, ViewKind::Pab);

    let a = node(pab, "BrakingModule");
    let b = node(pab, "BrakeActuation");
    assert_ne!(a.id, b.id, "diagram ids must be unique");
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.contains("BrakingModule") && d.contains("BrakeActuation")),
        "got {:?}",
        set.diagnostics
    );
    assert!(pab.validate().is_empty(), "{:?}", pab.validate());
}

#[test]
fn default_ids_use_the_whole_name_so_similar_names_do_not_collide() {
    let set = diagrams_of(
        r#"
physical_architecture PA {
  node Ecu {
    behavior_component BrakingModule {
      name: "a"
    }
    behavior_component BrakeActuation {
      name: "b"
    }
  }
}
"#,
    );
    let pab = view(&set, ViewKind::Pab);

    assert_eq!(node(pab, "BrakingModule").id, "BC-BrakingModule");
    assert_eq!(node(pab, "BrakeActuation").id, "BC-BrakeActuation");
    assert_ne!(
        node(pab, "BrakingModule").uuid,
        node(pab, "BrakeActuation").uuid
    );
    assert!(set.diagnostics.is_empty(), "got {:?}", set.diagnostics);
}

#[test]
fn functional_chain_resolves_to_nodes_and_the_exchanges_between_them() {
    let source = LOGICAL.replace(
        "system_analysis SA {",
        r#"system_analysis SA {
  functional_chain Main {
    involves: ["Sense", "Decide"]
  }"#,
    );
    let set = diagrams_of(&source);
    let sab = view(&set, ViewKind::Sab);

    let chain = sab.chains.iter().find(|c| c.name == "Main").expect("chain");
    assert_eq!(
        chain.nodes,
        vec![
            node(sab, "Sense").id.clone(),
            node(sab, "Decide").id.clone()
        ]
    );
    let exchange = sab
        .edges
        .iter()
        .find(|e| e.kind == EdgeKind::FunctionalExchange)
        .expect("exchange");
    assert_eq!(chain.edges, vec![exchange.id.clone()]);
}

#[test]
fn empty_layers_produce_no_diagram() {
    let set = diagrams_of(LOGICAL);
    assert!(set
        .diagrams
        .iter()
        .all(|d| d.kind != ViewKind::Oab && d.kind != ViewKind::Pab));
}

// ---------------------------------------------------------------------------
// Corpus invariants and golden files
// ---------------------------------------------------------------------------

fn example_files() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("examples dir").flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == "legacy") {
                    continue;
                }
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "arc") {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("examples"),
        &mut files,
    );
    files.sort();
    files
}

#[test]
fn every_example_yields_structurally_valid_deterministic_diagrams() {
    let mut checked = 0;
    for file in example_files() {
        let Ok(result) = Compiler::new(CompilerConfig::default()).compile_file(&file) else {
            continue; // import fragments do not compile standalone
        };
        let first = build_diagrams(&result.ast);
        let second = build_diagrams(&result.ast);
        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap(),
            "{} is not deterministic",
            file.display()
        );
        for diagram in &first.diagrams {
            let problems = diagram.validate();
            assert!(
                problems.is_empty(),
                "{} [{:?}]: {problems:?}",
                file.display(),
                diagram.kind
            );
            let graph = elk::to_elk(diagram);
            let dangling = elk::dangling_references(&graph);
            assert!(
                dangling.is_empty(),
                "{} [{:?}] ELK graph: {dangling:?}",
                file.display(),
                diagram.kind
            );
        }
        checked += 1;
    }
    assert!(
        checked >= 10,
        "expected the example corpus, only compiled {checked}"
    );
}

fn assert_golden(example: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = root.join("examples").join(format!("{example}.arc"));
    let golden = root
        .join("tests/fixtures/diagrams")
        .join(format!("{example}.json"));

    let result = Compiler::new(CompilerConfig::default())
        .compile_file(&source)
        .expect("example compiles");
    let generated = format!(
        "{}\n",
        serde_json::to_string_pretty(&build_diagrams(&result.ast)).unwrap()
    );

    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&golden, &generated).expect("write golden");
        return;
    }
    let expected = std::fs::read_to_string(&golden).unwrap_or_default();
    assert!(
        expected == generated,
        "diagram model of {example} changed. Review the diff, then regenerate with\n  UPDATE_GOLDEN=1 cargo test --test diagram_model"
    );
}

#[test]
fn flagship_diagram_model_matches_golden() {
    assert_golden("complete_emergency_braking_simple");
}

#[test]
fn all_layer_example_diagram_model_matches_golden() {
    assert_golden("complete_emergency_braking_mbse");
}

// ---------------------------------------------------------------------------
// Regressions from review: nothing is dropped or mis-bound silently
// ---------------------------------------------------------------------------

fn all_ids(diagram: &Diagram) -> Vec<String> {
    fn walk(nodes: &[Node], out: &mut Vec<String>) {
        for node in nodes {
            out.push(node.id.clone());
            out.extend(node.ports.iter().map(|p| p.id.clone()));
            walk(&node.children, out);
        }
    }
    let mut ids = Vec::new();
    walk(&diagram.nodes, &mut ids);
    ids.extend(diagram.edges.iter().map(|e| e.id.clone()));
    ids
}

#[test]
fn functional_exchange_to_an_undeclared_function_is_reported() {
    let source = LOGICAL.replace("to: Decide", "to: Decied");
    let set = diagrams_of(&source);
    let sab = view(&set, ViewKind::Sab);

    assert!(
        sab.edges.is_empty(),
        "no edge towards a function that does not exist"
    );
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.starts_with("[sab]") && d.contains("SenseToDecide") && d.contains("Decied")),
        "got {:?}",
        set.diagnostics
    );
}

#[test]
fn functional_exchange_to_an_unallocated_function_is_left_out_of_the_logical_view_quietly() {
    // `Decide` exists but no component allocates it: that is an incomplete
    // allocation (a lint concern), not a broken reference.
    let source = LOGICAL.replace(
        r#"    allocated_function: "Decide"
"#,
        "",
    );
    let set = diagrams_of(&source);
    let lab = view(&set, ViewKind::Lab);

    assert!(lab
        .edges
        .iter()
        .all(|e| e.kind != EdgeKind::FunctionalExchange));
    assert!(
        set.diagnostics.iter().all(|d| !d.starts_with("[lab]")),
        "got {:?}",
        set.diagnostics
    );
}

#[test]
fn behavior_component_allocating_an_undeclared_element_is_reported() {
    let set = diagrams_of(
        r#"
physical_architecture PA {
  node Ecu {
    behavior_component Sw {
      allocated_function: "Typo"
    }
  }
}
"#,
    );
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.starts_with("[pab]") && d.contains("Sw") && d.contains("Typo")),
        "got {:?}",
        set.diagnostics
    );
}

#[test]
fn a_port_and_an_inline_function_with_the_same_name_get_distinct_ids() {
    let set = diagrams_of(
        r#"
logical_architecture LA {
  component C {
    interface_in Status {
      name: "IStatus"
    }
    function Status {
      description: "report status"
    }
  }
}
"#,
    );
    let lab = view(&set, ViewKind::Lab);

    let ids = all_ids(lab);
    let unique: std::collections::HashSet<&String> = ids.iter().collect();
    assert_eq!(
        ids.len(),
        unique.len(),
        "ids must be unique across nodes, ports and edges: {ids:?}"
    );
    assert!(lab.validate().is_empty(), "{:?}", lab.validate());
}

#[test]
fn sub_function_allocated_apart_from_its_parent_is_drawn_once_and_reported() {
    let set = diagrams_of(
        r#"
system_analysis SA {
  function Parent {
    function Child {
      description: "sub"
    }
  }
}
logical_architecture LA {
  component A {
    allocated_function: "Child"
  }
  component B {
    allocated_function: "Parent"
  }
}
"#,
    );
    let lab = view(&set, ViewKind::Lab);

    fn count(nodes: &[Node], name: &str) -> usize {
        nodes
            .iter()
            .map(|n| usize::from(n.name == name) + count(&n.children, name))
            .sum()
    }
    assert_eq!(count(&lab.nodes, "Child"), 1, "Child drawn once");
    assert!(node(lab, "A").children.iter().any(|c| c.name == "Child"));
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.contains("Child") && d.contains("'A'") && d.contains("'B'")),
        "got {:?}",
        set.diagnostics
    );
    assert!(
        set.diagnostics
            .iter()
            .all(|d| !d.contains("identity collision")),
        "got {:?}",
        set.diagnostics
    );
}

#[test]
fn qualified_function_reference_still_binds_function_ports() {
    let source = LOGICAL.replace("from: Sense", "from: Sensor.Sense");
    let set = diagrams_of(&source);
    let lab = view(&set, ViewKind::Lab);

    let edge = lab
        .edges
        .iter()
        .find(|e| e.kind == EdgeKind::FunctionalExchange)
        .expect("exchange drawn");
    let out = node(lab, "Sense")
        .ports
        .iter()
        .find(|p| p.name == "raw")
        .expect("out port");
    assert_eq!(edge.source_port.as_deref(), Some(out.id.as_str()));
}

#[test]
fn port_name_shared_by_an_input_and_an_output_binds_by_exchange_direction() {
    let set = diagrams_of(
        r#"
logical_architecture LA {
  component D {
    interface_in CAN {
      name: "ICanRx"
    }
    interface_out CAN {
      name: "ICanTx"
    }
  }
  component E {
    interface_in Rx {
      name: "ICanTx"
    }
  }
  component_exchange "DToE" {
    from_port: "D.CAN"
    to_port: "E.Rx"
  }
}
"#,
    );
    let lab = view(&set, ViewKind::Lab);

    let edge = lab
        .edges
        .iter()
        .find(|e| e.kind == EdgeKind::ComponentExchange)
        .expect("edge");
    let out = node(lab, "D")
        .ports
        .iter()
        .find(|p| p.direction == PortDirection::Out)
        .expect("out port");
    assert_eq!(edge.source_port.as_deref(), Some(out.id.as_str()));
    assert!(set.diagnostics.is_empty(), "got {:?}", set.diagnostics);
}

#[test]
fn interaction_towards_an_undeclared_activity_is_not_attached_to_its_entity() {
    let set = diagrams_of(
        r#"
operational_analysis OA {
  actor Driver {
    description: "driver"
  }
  entity Vehicle {
    activity Monitor {
      description: "observe"
    }
  }
  interaction Commands {
    from: Driver
    to: Vehicle.Typo
  }
}
"#,
    );
    let oab = view(&set, ViewKind::Oab);

    assert!(
        oab.edges.is_empty(),
        "the edge must not silently retarget the entity"
    );
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.contains("Commands") && d.contains("Vehicle.Typo")),
        "got {:?}",
        set.diagnostics
    );
}

// ---------------------------------------------------------------------------
// Behaviour views: mode/state machines and exchange scenarios
// ---------------------------------------------------------------------------

const BEHAVIOUR: &str = r#"
logical_architecture LA {
  component Sensor {
    description: "s"
  }
  component Controller {
    description: "c"
  }
}
state_machine Modes {
  initial: "Standby"
  mode Standby { entry: ["arm"] }
  mode Active { }
  state Failed { }
  transition Standby -> Active { trigger: "threat" guard: "armed" action: "brake" }
  transition Active -> Failed { trigger: "fault" }
}
state_machine Power {
  initial: "Off"
  mode Off { }
  mode On { }
  transition Off -> On { trigger: "ignition" }
}
scenario Stop {
  participants: ["Sensor", "Controller"]
  message Sensor -> Controller "targets"
  message Controller -> Sensor "ack" { type: "async" timing: "5 ms" }
  message Controller -> Controller "decide"
}
"#;

fn diagram<'a>(set: &'a DiagramSet, id: &str) -> &'a Diagram {
    set.diagrams.iter().find(|d| d.id == id).unwrap_or_else(|| {
        panic!(
            "no diagram '{id}' in {:?}",
            set.diagrams.iter().map(|d| &d.id).collect::<Vec<_>>()
        )
    })
}

#[test]
fn each_state_machine_is_its_own_diagram_of_modes_states_and_transitions() {
    let set = diagrams_of(BEHAVIOUR);
    let modes = diagram(&set, "msm:Modes");
    assert_eq!(modes.kind, ViewKind::Msm);
    assert_eq!(diagram(&set, "msm:Power").kind, ViewKind::Msm);

    assert_eq!(node(modes, "Standby").kind, NodeKind::Mode);
    assert_eq!(node(modes, "Failed").kind, NodeKind::State);
    assert_eq!(
        node(modes, "Standby")
            .properties
            .get("entry")
            .map(String::as_str),
        Some("arm")
    );

    let transition = modes
        .edges
        .iter()
        .find(|e| e.source == node(modes, "Standby").id && e.target == node(modes, "Active").id)
        .expect("Standby -> Active");
    assert_eq!(transition.kind, EdgeKind::Transition);
    assert_eq!(transition.label, "threat [armed] / brake");
    assert!(set.diagnostics.is_empty(), "got {:?}", set.diagnostics);
    assert!(modes.validate().is_empty(), "{:?}", modes.validate());
}

#[test]
fn initial_state_is_marked_by_a_pseudo_state_and_its_transition() {
    let set = diagrams_of(BEHAVIOUR);
    let modes = diagram(&set, "msm:Modes");

    let initial = modes
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::InitialState)
        .expect("initial pseudo-state");
    let entry = modes
        .edges
        .iter()
        .find(|e| e.source == initial.id)
        .expect("initial transition");
    assert_eq!(entry.target, node(modes, "Standby").id);
    assert_eq!(entry.label, "");
}

/// The compiler rejects dangling behaviour references before any diagram is
/// built; `build_diagrams` also takes hand-built or imported ASTs, so it
/// must still report them rather than draw a guess.
fn behaviour_ast() -> arclang::compiler::ast::Model {
    Compiler::new(CompilerConfig::default())
        .compile_string(BEHAVIOUR)
        .expect("test model compiles")
        .ast
}

#[test]
fn transition_to_an_undeclared_state_and_unknown_initial_are_reported() {
    let mut ast = behaviour_ast();
    ast.state_machines[0].transitions[1].to = "Borken".to_string();
    ast.state_machines[1].initial_state = "Nowhere".to_string();
    let set = build_diagrams(&ast);
    let modes = diagram(&set, "msm:Modes");

    assert!(
        modes
            .edges
            .iter()
            .all(|e| e.source != node(modes, "Active").id),
        "no edge to a ghost state"
    );
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.contains("Modes") && d.contains("Borken")),
        "got {:?}",
        set.diagnostics
    );
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.contains("Power") && d.contains("Nowhere")),
        "got {:?}",
        set.diagnostics
    );
    assert!(diagram(&set, "msm:Power")
        .nodes
        .iter()
        .all(|n| n.kind != NodeKind::InitialState));
}

#[test]
fn scenario_keeps_lifelines_and_messages_in_declared_order() {
    let set = diagrams_of(BEHAVIOUR);
    let stop = diagram(&set, "es:Stop");
    assert_eq!(stop.kind, ViewKind::Es);

    let lifelines: Vec<&str> = stop.nodes.iter().map(|n| n.name.as_str()).collect();
    assert_eq!(lifelines, ["Sensor", "Controller"]);
    assert!(stop.nodes.iter().all(|n| n.kind == NodeKind::Lifeline));
    assert_eq!(
        node(stop, "Sensor")
            .properties
            .get("represents")
            .map(String::as_str),
        Some("logical_component")
    );

    let labels: Vec<&str> = stop.edges.iter().map(|e| e.label.as_str()).collect();
    assert_eq!(labels, ["targets", "ack", "decide"]);
    assert!(stop.edges.iter().all(|e| e.kind == EdgeKind::Message));
    assert_eq!(
        stop.edges[0].properties.get("type").map(String::as_str),
        Some("sync")
    );
    assert_eq!(
        stop.edges[1].properties.get("type").map(String::as_str),
        Some("async")
    );
    assert_eq!(
        stop.edges[1].properties.get("timing").map(String::as_str),
        Some("5 ms")
    );
    assert_eq!(
        stop.edges[2].source, stop.edges[2].target,
        "a message to self is kept"
    );
    assert!(stop.validate().is_empty(), "{:?}", stop.validate());
}

#[test]
fn message_naming_a_non_participant_and_undeclared_participant_are_reported() {
    let mut ast = behaviour_ast();
    ast.scenarios[0].messages[0].to = "Stranger".to_string();
    let mut ghost = ast.scenarios[0].participants[0].clone();
    ghost.id = "Ghost".to_string();
    ghost.name = "Ghost".to_string();
    ast.scenarios[0].participants.push(ghost);
    let set = build_diagrams(&ast);
    let stop = diagram(&set, "es:Stop");

    assert_eq!(
        stop.edges.len(),
        2,
        "the message with no lifeline to reach is not drawn"
    );
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.contains("Stop") && d.contains("Stranger")),
        "got {:?}",
        set.diagnostics
    );
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.contains("Stop") && d.contains("Ghost")),
        "got {:?}",
        set.diagnostics
    );
    assert!(!node(stop, "Ghost").properties.contains_key("represents"));
}

#[test]
fn scenario_export_is_a_sequence_layout_and_state_machine_export_is_layered() {
    let set = diagrams_of(BEHAVIOUR);

    let sequence = elk::to_elk(diagram(&set, "es:Stop"));
    assert_eq!(sequence["arc"]["layout"], "sequence");
    assert_eq!(sequence["arc"]["view"], "es");
    assert_eq!(sequence["children"].as_array().unwrap().len(), 2);
    assert_eq!(sequence["edges"].as_array().unwrap().len(), 3);
    assert!(elk::dangling_references(&sequence).is_empty());

    let machine = elk::to_elk(diagram(&set, "msm:Modes"));
    assert_eq!(machine["arc"]["layout"], "elk");
    assert_eq!(machine["layoutOptions"]["elk.algorithm"], "layered");
}

#[test]
fn machines_sharing_a_name_get_distinct_diagrams_and_are_reported() {
    let mut ast = behaviour_ast();
    ast.state_machines[1].name = "Modes".to_string();
    let set = build_diagrams(&ast);

    let ids: Vec<&str> = set
        .diagrams
        .iter()
        .filter(|d| d.kind == ViewKind::Msm)
        .map(|d| d.id.as_str())
        .collect();
    assert_eq!(ids, ["msm:Modes", "msm:Modes#2"]);
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.starts_with("[msm:Modes#2]") && d.contains("more than once")),
        "got {:?}",
        set.diagnostics
    );
    // The second machine kept its own content.
    assert!(find(&diagram(&set, "msm:Modes#2").nodes, "Off").is_some());
}

#[test]
fn machine_without_states_and_scenario_without_participants_are_reported() {
    let mut ast = behaviour_ast();
    ast.state_machines[1].states.clear();
    ast.scenarios[0].participants.clear();
    let set = build_diagrams(&ast);

    assert!(set
        .diagrams
        .iter()
        .all(|d| d.id != "msm:Power" && d.id != "es:Stop"));
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.starts_with("[msm:Power]") && d.contains("nothing to draw")),
        "got {:?}",
        set.diagnostics
    );
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.starts_with("[es:Stop]") && d.contains("3 message(s)")),
        "got {:?}",
        set.diagnostics
    );
}

#[test]
fn diagnostics_name_the_diagram_they_belong_to() {
    let mut ast = behaviour_ast();
    ast.state_machines[1].initial_state = "Nowhere".to_string();
    let set = build_diagrams(&ast);

    assert_eq!(set.diagnostics.len(), 1, "got {:?}", set.diagnostics);
    assert!(
        set.diagnostics[0].starts_with("[msm:Power] "),
        "got {:?}",
        set.diagnostics
    );
}

// ---------------------------------------------------------------------------
// Capabilities, data model and product breakdown
// ---------------------------------------------------------------------------

const TRANSVERSE: &str = r#"
operational_analysis OA {
  operational_capability "Avoid Collisions" {
    id: "OC-1"
    description: "avoid"
  }
}
system_analysis SA {
  mission SafeBraking {
    id: "MIS-1"
    description: "brake safely"
  }
  capability EmergencyBraking {
    id: "CAP-1"
    mission: "MIS-1"
    realizes: "OC-1"
    involves: ["Detect"]
  }
  actor Driver {
    description: "driver"
  }
  function Detect {
    outputs: ["threat"]
  }
}
class RadarFrame {
  id: "CL-1"
  range_m: "float"
  level: "ThreatLevel"
}
enumeration ThreatLevel {
  id: "EN-1"
  values: ["None", "Critical"]
}
data_type Force {
  id: "DT-1"
  base: "float"
}
exchange_item RadarTargets {
  id: "EI-1"
  mechanism: "FLOW"
  elements: ["CL-1"]
}
epbs "Product" {
  system "Brake System" {
    id: "CI-1"
    subsystem "Sensing" {
      id: "CI-2"
      item "Radar" {
        id: "CI-3"
        part_number: "ARS540"
      }
    }
  }
}
"#;

fn edge_between<'a>(
    diagram: &'a Diagram,
    from: &str,
    to: &str,
) -> &'a arclang::compiler::diagram::Edge {
    let (source, target) = (&node(diagram, from).id, &node(diagram, to).id);
    diagram
        .edges
        .iter()
        .find(|e| &e.source == source && &e.target == target)
        .unwrap_or_else(|| {
            panic!(
                "no edge {from} -> {to} in {:?}",
                diagram
                    .edges
                    .iter()
                    .map(|e| (&e.source, &e.target))
                    .collect::<Vec<_>>()
            )
        })
}

#[test]
fn capability_view_links_missions_capabilities_and_what_they_involve() {
    // The actor is added to the model after compiling: the diagram must
    // also draw models that were not written as source text.
    let mut ast = Compiler::new(CompilerConfig::default())
        .compile_string(TRANSVERSE)
        .expect("compiles")
        .ast;
    ast.system_analysis[0].capabilities[0]
        .involves
        .push("Driver".to_string());
    let set = build_diagrams(&ast);
    let cap = diagram(&set, "cap");
    assert_eq!(cap.kind, ViewKind::Cap);

    assert_eq!(node(cap, "SafeBraking").kind, NodeKind::Mission);
    assert_eq!(node(cap, "EmergencyBraking").kind, NodeKind::Capability);
    assert_eq!(
        node(cap, "Avoid Collisions").kind,
        NodeKind::OperationalCapability
    );
    assert_eq!(node(cap, "Detect").kind, NodeKind::Function);
    assert_eq!(node(cap, "Driver").kind, NodeKind::SystemActor);

    assert_eq!(
        edge_between(cap, "SafeBraking", "EmergencyBraking").kind,
        EdgeKind::Exploitation
    );
    assert_eq!(
        edge_between(cap, "EmergencyBraking", "Avoid Collisions").kind,
        EdgeKind::Realization
    );
    assert_eq!(
        edge_between(cap, "EmergencyBraking", "Detect").kind,
        EdgeKind::Involvement
    );
    assert_eq!(
        edge_between(cap, "EmergencyBraking", "Driver").kind,
        EdgeKind::Involvement
    );
    assert!(set.diagnostics.is_empty(), "got {:?}", set.diagnostics);
    assert!(cap.validate().is_empty(), "{:?}", cap.validate());
}

#[test]
fn capability_referring_to_an_unknown_mission_or_element_is_reported() {
    let mut ast = Compiler::new(CompilerConfig::default())
        .compile_string(TRANSVERSE)
        .expect("compiles")
        .ast;
    let capability = &mut ast.system_analysis[0].capabilities[0];
    capability.mission = Some("MIS-404".to_string());
    capability.involves.push("Phantom".to_string());
    let set = build_diagrams(&ast);
    let cap = diagram(&set, "cap");

    assert!(
        find(&cap.nodes, "Phantom").is_none(),
        "nothing is invented for an unknown element"
    );
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.starts_with("[cap]") && d.contains("MIS-404")),
        "got {:?}",
        set.diagnostics
    );
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.starts_with("[cap]") && d.contains("Phantom")),
        "got {:?}",
        set.diagnostics
    );
}

#[test]
fn data_view_lists_fields_and_links_typed_fields_and_exchange_items() {
    let set = diagrams_of(TRANSVERSE);
    let data = diagram(&set, "cdb");
    assert_eq!(data.kind, ViewKind::Cdb);

    let class = node(data, "RadarFrame");
    assert_eq!(class.kind, NodeKind::Class);
    assert_eq!(class.compartment, ["range_m : float", "level : ThreatLevel"]);
    let levels = node(data, "ThreatLevel");
    assert_eq!(levels.kind, NodeKind::Enumeration);
    assert_eq!(levels.compartment, ["None", "Critical"]);
    assert_eq!(node(data, "Force").kind, NodeKind::DataType);
    assert_eq!(node(data, "Force").compartment, ["base : float"]);
    assert_eq!(node(data, "RadarTargets").kind, NodeKind::ExchangeItem);
    assert_eq!(
        node(data, "RadarTargets")
            .properties
            .get("mechanism")
            .map(String::as_str),
        Some("FLOW")
    );

    let typed = edge_between(data, "RadarFrame", "ThreatLevel");
    assert_eq!(typed.kind, EdgeKind::Association);
    assert_eq!(typed.label, "level");
    assert_eq!(
        edge_between(data, "RadarTargets", "RadarFrame").kind,
        EdgeKind::ItemElement
    );
    assert_eq!(
        data.edges.len(),
        2,
        "a primitive type such as float is not an element to link"
    );
    assert!(set.diagnostics.is_empty(), "got {:?}", set.diagnostics);
}

#[test]
fn exchange_item_grouping_an_undeclared_element_is_reported() {
    let mut ast = Compiler::new(CompilerConfig::default())
        .compile_string(TRANSVERSE)
        .expect("compiles")
        .ast;
    ast.exchange_items[0].elements.push("CL-404".to_string());
    let set = build_diagrams(&ast);

    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.starts_with("[cdb]") && d.contains("RadarTargets") && d.contains("CL-404")),
        "got {:?}",
        set.diagnostics
    );
}

#[test]
fn product_breakdown_is_a_tree_from_system_down_to_items() {
    let set = diagrams_of(TRANSVERSE);
    let pbs = diagram(&set, "pbs");
    assert_eq!(pbs.kind, ViewKind::Pbs);

    for name in ["Brake System", "Sensing", "Radar"] {
        assert_eq!(node(pbs, name).kind, NodeKind::ConfigurationItem);
    }
    assert_eq!(node(pbs, "Radar").id, "CI-3");
    assert_eq!(
        node(pbs, "Radar")
            .properties
            .get("part_number")
            .map(String::as_str),
        Some("ARS540")
    );
    assert_eq!(
        node(pbs, "Sensing")
            .properties
            .get("breakdown_level")
            .map(String::as_str),
        Some("subsystem")
    );
    assert_eq!(
        edge_between(pbs, "Brake System", "Sensing").kind,
        EdgeKind::Breakdown
    );
    assert_eq!(
        edge_between(pbs, "Sensing", "Radar").kind,
        EdgeKind::Breakdown
    );
    assert_eq!(pbs.edges.len(), 2);
    assert_eq!(elk::to_elk(pbs)["layoutOptions"]["elk.direction"], "DOWN");
}

#[test]
fn class_compartment_reaches_the_layout_graph_and_sizes_the_box() {
    let set = diagrams_of(TRANSVERSE);
    let graph = elk::to_elk(diagram(&set, "cdb"));
    let class = graph["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == "CL-1")
        .expect("class node");
    let force = graph["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == "DT-1")
        .expect("data type node");

    assert_eq!(class["arc"]["compartment"][0], "range_m : float");
    assert!(
        class["height"].as_f64().unwrap() > force["height"].as_f64().unwrap(),
        "two lines need more room than one"
    );
}

#[test]
fn field_typed_by_an_ambiguous_name_is_reported_and_other_links_survive() {
    // A class and an exchange item share the name `RadarFrame`.
    let mut ast = Compiler::new(CompilerConfig::default())
        .compile_string(TRANSVERSE)
        .expect("compiles")
        .ast;
    ast.exchange_items[0].name = "RadarFrame".to_string();
    let mut field = ast.classes[0].fields[0].clone();
    field.name = "previous".to_string();
    field.attr_type = "RadarFrame".to_string();
    ast.classes[0].fields.push(field);
    let set = build_diagrams(&ast);
    let data = diagram(&set, "cdb");

    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.starts_with("[cdb]") && d.contains("previous") && d.contains("ambiguous")),
        "got {:?}",
        set.diagnostics
    );
    // The class's other typed field and the item's grouping are still drawn.
    assert!(data
        .edges
        .iter()
        .any(|e| e.kind == EdgeKind::Association && e.label == "level"));
    assert!(data.edges.iter().any(|e| e.kind == EdgeKind::ItemElement));
}

#[test]
fn involved_element_is_never_mistaken_for_a_capability_of_the_same_key() {
    // The function's id equals the capability's name.
    let mut ast = Compiler::new(CompilerConfig::default())
        .compile_string(TRANSVERSE)
        .expect("compiles")
        .ast;
    ast.system_analysis[0].functions[0].id = "EmergencyBraking".to_string();
    ast.system_analysis[0].functions[0].attributes.remove("id");
    let set = build_diagrams(&ast);
    let cap = diagram(&set, "cap");

    let function = cap
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Function)
        .expect("the function is drawn");
    let capability = cap
        .nodes
        .iter()
        .find(|n| n.kind == NodeKind::Capability)
        .expect("capability");
    let involvement = cap
        .edges
        .iter()
        .find(|e| e.kind == EdgeKind::Involvement)
        .expect("involvement");
    assert_eq!(involvement.source, capability.id);
    assert_eq!(involvement.target, function.id);
    assert_ne!(function.id, capability.id);
}

#[test]
fn capability_realizing_itself_or_a_mission_is_reported_not_drawn() {
    let mut ast = Compiler::new(CompilerConfig::default())
        .compile_string(TRANSVERSE)
        .expect("compiles")
        .ast;
    ast.system_analysis[0].capabilities[0].realizes = Some("MIS-1".to_string());
    let set = build_diagrams(&ast);
    assert!(diagram(&set, "cap")
        .edges
        .iter()
        .all(|e| e.kind != EdgeKind::Realization));
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.contains("not a capability")),
        "got {:?}",
        set.diagnostics
    );

    ast.system_analysis[0].capabilities[0].realizes = Some("CAP-1".to_string());
    let set = build_diagrams(&ast);
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.contains("cannot realize itself")),
        "got {:?}",
        set.diagnostics
    );
}

#[test]
fn breakdown_level_does_not_overwrite_an_attribute_named_level() {
    let source = TRANSVERSE.replace("part_number: \"ARS540\"", "level: \"7\"");
    let set = diagrams_of(&source);
    let radar = node(diagram(&set, "pbs"), "Radar");
    assert_eq!(radar.properties.get("level").map(String::as_str), Some("7"));
    assert_eq!(
        radar.properties.get("breakdown_level").map(String::as_str),
        Some("item")
    );
}
