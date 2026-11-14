//! An exchange leaves an out port and arrives on an in port (rule
//! interface.direction). The case is the hybrid powertrain of Banagar et
//! al. (J. Syst. Softw. 232, 2026, CC BY): its P4 configuration, as
//! published, gives the electric motor's two mechanical ports the reverse
//! directions of the P1 motor, so two interfaces join two inputs and two
//! outputs; the P1 model and the corrected P4 say nothing.

use arclang::compiler::rules;
use arclang::{Compiler, CompilerConfig};
use std::path::Path;

fn warnings(example: &str) -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/automotive/hybrid-powertrain").join(example);
    let result = Compiler::new(CompilerConfig::default()).compile_file(&path).unwrap_or_else(|e| panic!("{example}: {e}"));
    result.warnings.into_iter().filter(|w| w.starts_with("exchange direction: ")).collect()
}

#[test]
fn the_published_p4_motor_ports_join_two_inputs_and_two_outputs() {
    let found = warnings("hybrid_powertrain_p4.arc");
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found[0].contains("'int5: trq_int' leaves 'em.mtr_trq_out', an in port"), "{}", found[0]);
    assert!(found[1].contains("'int6: spe_int' arrives on 'em.mtr_speed_in', an out port"), "{}", found[1]);
    let finding = rules::classify_warning(&found[0]);
    assert_eq!(finding.rule, "interface.direction");
    assert_eq!(finding.severity, rules::Severity::Warning);
    assert_eq!(finding.element.as_deref(), Some("int5: trq_int"));
}

#[test]
fn p1_and_the_corrected_p4_send_every_signal_from_its_producer() {
    assert_eq!(warnings("hybrid_powertrain_p1.arc"), Vec::<String>::new());
    assert_eq!(warnings("hybrid_powertrain_p4_fixed.arc"), Vec::<String>::new());
}

#[test]
fn inout_ports_take_either_role_and_unresolved_ends_are_left_to_the_endpoint_check() {
    let text = r#"model M { name: "m" }
logical_architecture "LA" {
    component "A" { id: "A" port inout "bus" port out "x" }
    component "B" { id: "B" port inout "bus" port in "y" }
    component_exchange "bus" { from_port: "A.bus" to_port: "B.bus" }
    component_exchange "by name" { from_port: "A.x" to_port: "B.y" }
    component_exchange "unknown" { from_port: "A.nowhere" to_port: "B.y" }
    component_exchange "backwards" { from_port: "B.y" to_port: "A.x" }
}"#;
    let result = Compiler::new(CompilerConfig::default()).compile_string(text).unwrap();
    let found: Vec<&String> = result.warnings.iter().filter(|w| w.starts_with("exchange direction: ")).collect();
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found.iter().all(|w| w.contains("'backwards'")), "{found:#?}");
}
