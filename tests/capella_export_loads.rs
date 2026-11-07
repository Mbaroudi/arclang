//! What Capella 7.1 needs to open the export at all, as its validation in
//! Capella on Linux showed: the namespace of its modeller package, component
//! exchanges ended on ports, interactions between entities as communication means.

use arclang::compiler::capella_export::{export, Exported};
use arclang::compiler::simulink_connector::m2m::model::Model;
use arclang::compiler::{Compiler, CompilerConfig};
use std::path::Path;

fn example(path: &str, stem: &str) -> Exported {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let ast = Compiler::new(CompilerConfig::default()).compile_file(&path).expect("compiles").ast;
    export(&ast, stem)
}

fn model(exported: &Exported, stem: &str) -> Model {
    Model::from_strings(&[(&format!("{stem}.capella"), exported.capella.as_str())]).unwrap()
}

#[test]
fn the_project_is_in_capellas_modeller_package() {
    let exported = example("examples/automotive/adaptive-cruise/adaptive_cruise_control.arc", "acc");
    // `modeller`, as Capella spells its package: with `modeler` EMF loads no Project.
    assert!(exported.capella.contains(r#"xmlns:org.polarsys.capella.core.data.capellamodeller="http://www.polarsys.org/capella/core/modeller/7.0.0""#));
    assert!(!exported.capella.contains("capella/core/modeler/"));
}

#[test]
fn a_physical_exchange_between_nodes_ends_on_their_ports() {
    // A component is not an end of a component exchange in Capella: a port is.
    let exported = example("examples/automotive/adaptive-cruise/adaptive_cruise_control.arc", "acc");
    let m = model(&exported, "acc");
    let exchanges: Vec<usize> = (0..m.elements.len()).filter(|&e| m.class(e) == "ComponentExchange").collect();
    assert!(!exchanges.is_empty());
    for exchange in exchanges {
        for end in ["source", "target"] {
            let at = m.reference(exchange, end).unwrap_or_else(|| panic!("{:?} has its {end}", m.name(exchange)));
            assert!(matches!(m.class(at), "ComponentPort" | "PhysicalPort" | "Part"), "{:?} {end} is a {}", m.name(exchange), m.class(at));
        }
    }
}

#[test]
fn an_interaction_between_entities_is_a_communication_mean() {
    // A functional exchange links functions; between two entities Capella has a communication mean.
    let exported = example("examples/automotive/remote-start/remote_start_architecture.arc", "rs");
    let m = model(&exported, "rs");
    let alerts = (0..m.elements.len()).find(|&e| m.id(e).as_deref() == Some("OI-RS-003")).expect("the interaction is exported");
    assert_eq!(m.class(alerts), "CommunicationMean");
    for exchange in (0..m.elements.len()).filter(|&e| m.class(e) == "FunctionalExchange") {
        for end in ["source", "target"] {
            let at = m.reference(exchange, end).unwrap();
            assert!(!matches!(m.class(at), "Entity" | "PhysicalComponent" | "LogicalComponent" | "SystemComponent"), "{:?} {end} is a {}", m.name(exchange), m.class(at));
        }
    }
}
