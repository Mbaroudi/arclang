//! The operational analysis reaches Capella whole: an activity written
//! inside the entity that performs it, its interactions, the capability
//! that involves it, and what holds before and after a capability
//! (issue #276).

use arclang::compiler::capella_export::export;
use arclang::compiler::capella_xmi::import_strings;
use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"model "Hall" { id: "M" }
operational_analysis "Operations" {
    entity "Operator" {
        id: "OA-OP"
        activity "Ask for cooling" { id: "OA-ASK" }
    }
    entity "Hall" {
        id: "OA-HALL"
        activity "Remove the heat" { id: "OA-COOL" }
    }
    interaction "Cooling request" { from: "OA-ASK" to: "OA-COOL" }
    operational_capability "Keep the hall cool" {
        id: "OC-COOL"
        precondition: "The hall is above 27 °C"
        postcondition: "The hall is back under 25 °C"
        involves: ["OA-OP", "OA-HALL", "OA-ASK", "OA-COOL"]
    }
}
"#;

fn compile(source: &str) -> arclang::compiler::ast::Model {
    Compiler::new(CompilerConfig::default()).compile_string(source).unwrap_or_else(|e| panic!("{e}\n{source}")).ast
}

fn line<'a>(capella: &'a str, id: &str) -> &'a str {
    capella.lines().find(|l| l.contains(&format!("id=\"{id}\""))).unwrap_or_else(|| panic!("no {id} in\n{capella}"))
}

#[test]
fn an_activity_written_inside_an_entity_is_exported_and_performed_by_it() {
    let capella = export(&compile(MODEL), "Hall").capella;
    assert!(line(&capella, "OA-ASK").contains("oa:OperationalActivity"), "{capella}");
    assert!(line(&capella, "OA-COOL").contains("oa:OperationalActivity"), "{capella}");
    let allocation = capella.lines().find(|l| l.contains("ComponentFunctionalAllocation") && l.contains("targetElement=\"#OA-ASK\"")).unwrap_or_else(|| panic!("{capella}"));
    assert!(allocation.contains("sourceElement=\"#OA-OP\""), "{allocation}");
}

#[test]
fn its_interactions_and_the_capability_that_involves_it_follow() {
    let exported = export(&compile(MODEL), "Hall");
    let capella = &exported.capella;
    assert!(capella.lines().any(|l| l.contains("fa:FunctionalExchange") && l.contains("source=\"#OA-ASK\"") && l.contains("target=\"#OA-COOL\"")), "{capella}");
    for activity in ["OA-ASK", "OA-COOL"] {
        assert!(capella.lines().any(|l| l.contains("AbstractFunctionAbstractCapabilityInvolvement") && l.contains(&format!("involved=\"#{activity}\""))), "{activity}: {capella}");
    }
}

#[test]
fn what_holds_before_and_after_a_capability_is_a_constraint_it_names() {
    let capella = export(&compile(MODEL), "Hall").capella;
    let capability = line(&capella, "OC-COOL");
    for (reference, text) in [("preCondition", "The hall is above 27 °C"), ("postCondition", "The hall is back under 25 °C")] {
        let id = capability.split(&format!("{reference}=\"#")).nth(1).and_then(|r| r.split('"').next()).unwrap_or_else(|| panic!("no {reference}: {capability}"));
        assert!(line(&capella, id).contains("capellacore:Constraint"), "{capella}");
        assert!(capella.contains(text), "{capella}");
    }
    assert!(!capella.contains("name=\"precondition\"") && !capella.contains("name=\"postcondition\""), "not a property value: {capella}");
}

#[test]
fn they_come_back_from_capella_as_they_left() {
    let capella = export(&compile(MODEL), "Hall").capella;
    let back = import_strings(&capella, None, "Hall.capella").unwrap_or_else(|f| panic!("{}", f.message)).source;
    assert!(back.contains("precondition: \"The hall is above 27 °C\""), "{back}");
    assert!(back.contains("postcondition: \"The hall is back under 25 °C\""), "{back}");
    assert!(!back.contains("constraints:"), "{back}");
    assert!(back.contains("performed_by: \"OA-OP\"") && back.contains("performed_by: \"OA-HALL\""), "{back}");
    let again = export(&compile(&back), "Hall").capella;
    let shape = |c: &str| ["oa:OperationalActivity", "fa:FunctionalExchange", "preCondition=", "postCondition=", "ComponentFunctionalAllocation", "CapabilityInvolvement"].map(|k| c.matches(k).count());
    assert_eq!(shape(&capella), shape(&again));
}
