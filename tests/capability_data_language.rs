//! Language coverage for capabilities and the data model: actors involved in
//! capabilities, nested capabilities, capability relations, units of data
//! types and the declared order of class fields. Each behaviour is checked
//! from source text down to the diagram a reader sees.

use arclang::compiler::ast::Model;
use arclang::compiler::diagram::{build_diagrams, Diagram, DiagramSet, EdgeKind, Node, NodeKind};
use arclang::compiler::{Compiler, CompilerConfig};

fn compile(source: &str) -> Result<Model, String> {
    Compiler::new(CompilerConfig::default())
        .compile_string(source)
        .map(|result| result.ast)
        .map_err(|error| error.to_string())
}

fn diagram<'a>(set: &'a DiagramSet, id: &str) -> &'a Diagram {
    set.diagrams
        .iter()
        .find(|d| d.id == id)
        .unwrap_or_else(|| panic!("no '{id}' diagram"))
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
    find(&diagram.nodes, name).unwrap_or_else(|| panic!("node '{name}' is not drawn"))
}

fn edge_kinds(diagram: &Diagram, from: &str, to: &str) -> Vec<(EdgeKind, String)> {
    let (from, to) = (&node(diagram, from).id, &node(diagram, to).id);
    diagram
        .edges
        .iter()
        .filter(|e| &e.source == from && &e.target == to)
        .map(|e| (e.kind, e.label.clone()))
        .collect()
}

const ACTORS: &str = r#"
operational_analysis "OA" {
    actor "Pilot" { id: "OA-ACT-1" }
    operational_entity "Tower" {
        id: "OE-1"
        operational_activity "Clear Runway" { id: "OA-1" }
    }
    operational_capability "Land Safely" {
        id: "OC-1"
        involves: ["Pilot", "Tower", "Clear Runway"]
    }
}
system_analysis "SA" {
    actor "Driver" { id: "ACT-1" }
    function "Brake" { id: "SF-1" }
    capability "Stop" { id: "CAP-1" involves: ["Brake", "Driver"] }
}
"#;

#[test]
fn capability_involving_a_system_actor_compiles_and_is_drawn() {
    let set = build_diagrams(&compile(ACTORS).expect("an actor may be involved"));
    let cap = diagram(&set, "cap");

    assert_eq!(node(cap, "Driver").kind, NodeKind::SystemActor);
    assert_eq!(
        edge_kinds(cap, "Stop", "Driver"),
        vec![(EdgeKind::Involvement, String::new())]
    );
    assert!(set.diagnostics.is_empty(), "got {:?}", set.diagnostics);
}

#[test]
fn operational_capability_involves_actors_entities_and_activities() {
    let set = build_diagrams(&compile(ACTORS).expect("compiles"));
    let cap = diagram(&set, "cap");

    assert_eq!(node(cap, "Pilot").kind, NodeKind::OperationalActor);
    assert_eq!(node(cap, "Tower").kind, NodeKind::OperationalEntity);
    assert_eq!(
        node(cap, "Clear Runway").kind,
        NodeKind::OperationalActivity
    );
    for involved in ["Pilot", "Tower", "Clear Runway"] {
        assert_eq!(
            edge_kinds(cap, "Land Safely", involved),
            vec![(EdgeKind::Involvement, String::new())],
            "Land Safely involves {involved}"
        );
    }
}

#[test]
fn operational_capability_involving_an_undeclared_element_is_a_compile_error() {
    let error = compile(
        r#"operational_analysis "OA" {
            operational_capability "Land" { id: "OC-1" involves: ["Ghost"] }
        }"#,
    )
    .expect_err("a dangling involvement must not compile");
    assert!(
        error.contains("operational_capability 'Land' involves: unknown element 'Ghost'"),
        "got: {error}"
    );
}

const NESTED: &str = r#"
operational_analysis "OA" {
    operational_capability "Travel" {
        id: "OC-1"
        operational_capability "Board" { id: "OC-2" }
        operational_capability "Disembark" {
            id: "OC-3"
            operational_capability "Collect Luggage" { id: "OC-4" }
        }
    }
}
system_analysis "SA" {
    capability "Transport" {
        id: "CAP-1"
        realizes: "OC-1"
        capability "Load" { id: "CAP-2" realizes: "OC-2" }
    }
}
logical_architecture "LA" {
    capability_realization "Move" {
        id: "CR-1"
        realizes: "CAP-1"
        capability_realization "Lift" { id: "CR-2" realizes: "CAP-2" }
    }
}
"#;

#[test]
fn nested_capabilities_keep_their_parent_at_every_level() {
    let model = compile(NESTED).expect("nested capabilities compile");

    let oa = &model.operational_analysis[0];
    let parents: Vec<(&str, Option<&str>)> = oa
        .capabilities
        .iter()
        .map(|c| (c.name.as_str(), c.parent.as_deref()))
        .collect();
    assert_eq!(
        parents,
        vec![
            ("Travel", None),
            ("Board", Some("OC-1")),
            ("Disembark", Some("OC-1")),
            ("Collect Luggage", Some("OC-3")),
        ],
        "every capability is declared once, in source order, with its parent"
    );
    let system = &model.system_analysis[0].capabilities;
    assert_eq!(system[1].parent.as_deref(), Some("CAP-1"));
    let logical = &model.logical_architecture[0].capability_realizations;
    assert_eq!(logical[1].parent.as_deref(), Some("CR-1"));
}

#[test]
fn nested_capabilities_are_drawn_inside_their_parent() {
    let set = build_diagrams(&compile(NESTED).expect("compiles"));
    let cap = diagram(&set, "cap");

    let travel = node(cap, "Travel");
    let inside: Vec<&str> = travel.children.iter().map(|n| n.name.as_str()).collect();
    assert_eq!(inside, vec!["Board", "Disembark"]);
    assert!(find(&node(cap, "Disembark").children, "Collect Luggage").is_some());
    assert!(find(&node(cap, "Transport").children, "Load").is_some());
    assert!(find(&node(cap, "Move").children, "Lift").is_some());
    // A nested capability still realizes across levels.
    assert_eq!(
        edge_kinds(cap, "Load", "Board"),
        vec![(EdgeKind::Realization, String::new())]
    );
    assert!(set.diagnostics.is_empty(), "got {:?}", set.diagnostics);
    assert!(cap.validate().is_empty(), "{:?}", cap.validate());
}

const RELATED: &str = r#"
operational_analysis "OA" {
    operational_capability "Travel" { id: "OC-1" includes: ["Board"] }
    operational_capability "Board" { id: "OC-2" }
    operational_capability "Board With Assistance" { id: "OC-3" specializes: ["OC-2"] }
}
system_analysis "SA" {
    capability "Brake" { id: "CAP-1" }
    capability "Emergency Brake" {
        id: "CAP-2"
        extends: ["Brake"]
        includes: ["Warn Driver"]
        specializes: ["Brake"]
    }
    capability "Warn Driver" { id: "CAP-3" }
}
logical_architecture "LA" {
    capability_realization "Brake Logic" { id: "CR-1" realizes: "CAP-1" }
    capability_realization "Emergency Logic" { id: "CR-2" realizes: "CAP-2" extends: ["CR-1"] }
}
"#;

#[test]
fn capability_relations_are_drawn_at_every_level() {
    let set = build_diagrams(&compile(RELATED).expect("capability relations compile"));
    let cap = diagram(&set, "cap");

    assert_eq!(
        edge_kinds(cap, "Travel", "Board"),
        vec![(EdgeKind::CapabilityAssociation, "«include»".to_string())]
    );
    assert_eq!(
        edge_kinds(cap, "Board With Assistance", "Board"),
        vec![(EdgeKind::Generalization, String::new())]
    );
    assert_eq!(
        edge_kinds(cap, "Emergency Brake", "Brake"),
        vec![
            (EdgeKind::CapabilityAssociation, "«extend»".to_string()),
            (EdgeKind::Generalization, String::new()),
        ]
    );
    assert_eq!(
        edge_kinds(cap, "Emergency Brake", "Warn Driver"),
        vec![(EdgeKind::CapabilityAssociation, "«include»".to_string())]
    );
    assert_eq!(
        edge_kinds(cap, "Emergency Logic", "Brake Logic"),
        vec![(EdgeKind::CapabilityAssociation, "«extend»".to_string())]
    );
    assert!(set.diagnostics.is_empty(), "got {:?}", set.diagnostics);
    assert!(cap.validate().is_empty(), "{:?}", cap.validate());
}

#[test]
fn capability_relation_to_an_unknown_or_foreign_capability_is_a_compile_error() {
    let unknown = compile(
        r#"system_analysis "SA" {
            capability "Brake" { id: "CAP-1" extends: ["Ghost"] }
        }"#,
    )
    .expect_err("a dangling relation must not compile");
    assert!(
        unknown.contains("capability 'Brake' extends: unknown element 'Ghost'"),
        "got: {unknown}"
    );

    let foreign = compile(
        r#"operational_analysis "OA" {
            operational_capability "Travel" { id: "OC-1" }
        }
        system_analysis "SA" {
            function "Stop" { id: "SF-1" }
            capability "Brake" { id: "CAP-1" includes: ["Stop"] specializes: ["OC-1"] }
        }"#,
    )
    .expect_err("a relation links capabilities of one level");
    assert!(
        foreign.contains("capability 'Brake' includes: 'Stop' is a SystemFunction, not a capability of the same level"),
        "got: {foreign}"
    );
    assert!(
        foreign.contains("capability 'Brake' specializes: 'Travel' is a OperationalCapability, not a capability of the same level"),
        "got: {foreign}"
    );

    let own = compile(
        r#"system_analysis "SA" {
            capability "Brake" { id: "CAP-1" includes: ["CAP-1"] }
        }"#,
    )
    .expect_err("a capability cannot relate to itself");
    assert!(
        own.contains("capability 'Brake' includes: a capability cannot relate to itself"),
        "got: {own}"
    );
}

const DATA: &str = r#"
data_type "Speed" { base: "float" unit: "m/s" }
data_type "Counter" { base: "uint16" }
class "RadarFrame" {
    id: "CLS-1"
    description: "One radar sweep"
    timestamp: "uint64"
    range: "float"
    azimuth: "float"
    closing_speed: "Speed"
}
"#;

#[test]
fn data_type_keeps_its_unit_down_to_the_diagram() {
    let model = compile(DATA).expect("compiles");
    let speed = model.data_types.iter().find(|d| d.name == "Speed").unwrap();
    assert_eq!(speed.unit.as_deref(), Some("m/s"));
    let counter = model
        .data_types
        .iter()
        .find(|d| d.name == "Counter")
        .unwrap();
    assert_eq!(counter.unit, None);

    let set = build_diagrams(&model);
    let cdb = diagram(&set, "cdb");
    assert_eq!(
        node(cdb, "Speed").compartment,
        vec!["base : float", "unit : m/s"]
    );
    assert_eq!(node(cdb, "Counter").compartment, vec!["base : uint16"]);
}

#[test]
fn data_type_with_an_unknown_unit_is_reported() {
    // Attribute type violations are warnings at compile time and blockers
    // in the production gate, like every other typed attribute.
    let result = Compiler::new(CompilerConfig::default())
        .compile_string(r#"data_type "Speed" { base: "float" unit: "furlong" }"#)
        .expect("compiles with a warning");
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.contains("DataType 'Speed'.unit: 'furlong' is not a valid Unit")),
        "got: {:?}",
        result.warnings
    );

    let known = Compiler::new(CompilerConfig::default())
        .compile_string(DATA)
        .expect("compiles");
    assert!(
        !known.warnings.iter().any(|w| w.contains("Unit")),
        "got: {:?}",
        known.warnings
    );
}

#[test]
fn class_fields_keep_their_declared_order() {
    let model = compile(DATA).expect("compiles");
    let names: Vec<&str> = model.classes[0]
        .fields
        .iter()
        .map(|f| f.name.as_str())
        .collect();
    assert_eq!(
        names,
        vec!["timestamp", "range", "azimuth", "closing_speed"]
    );

    let set = build_diagrams(&model);
    assert_eq!(
        node(diagram(&set, "cdb"), "RadarFrame").compartment,
        vec![
            "timestamp : uint64",
            "range : float",
            "azimuth : float",
            "closing_speed : Speed",
        ],
        "UML attribute notation, in declared order"
    );
}

#[test]
fn class_declaring_a_field_twice_is_a_compile_error() {
    let error = compile(r#"class "Frame" { range: "float" range: "uint8" }"#)
        .expect_err("a field declared twice must not compile");
    assert!(
        error.contains("class 'Frame' declares field 'range' twice"),
        "got: {error}"
    );
}

#[test]
fn compiling_twice_gives_the_same_field_order() {
    let first = compile(DATA).expect("compiles");
    let second = compile(DATA).expect("compiles");
    let order =
        |m: &Model| -> Vec<String> { m.classes[0].fields.iter().map(|f| f.name.clone()).collect() };
    assert_eq!(order(&first), order(&second));
}

#[test]
fn class_field_whose_type_is_not_text_is_a_compile_error() {
    let error = compile(r#"class "Frame" { range: "float" count: 5 }"#)
        .expect_err("a field must name its type");
    assert!(
        error.contains("class 'Frame': field 'count' must name its type as text"),
        "got: {error}"
    );
}

#[test]
fn nested_capabilities_without_an_id_are_identified_under_their_parent() {
    let model = compile(
        r#"system_analysis "SA" {
            capability "Top1" { capability "Brake" { } }
            capability "Top2" { capability "Brake" { } }
        }"#,
    )
    .expect("same name under different parents is not a collision");
    let ids: Vec<&str> = model.system_analysis[0]
        .capabilities
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(
        ids,
        vec!["CAP-Top1", "CAP-Top1/Brake", "CAP-Top2", "CAP-Top2/Brake"]
    );
}

#[test]
fn two_capabilities_sharing_an_id_are_a_compile_error() {
    let error = compile(
        r#"system_analysis "SA" {
            capability "Brake" { id: "CAP-1" }
            capability "Steer" { id: "CAP-1" }
        }"#,
    )
    .expect_err("one id, two capabilities");
    assert!(
        error.contains("capabilities 'Brake' and 'Steer' share the id 'CAP-1'"),
        "got: {error}"
    );
}

#[test]
fn capability_involving_something_that_cannot_be_involved_is_a_compile_error() {
    let error = compile(
        r#"operational_analysis "OA" {
            operational_capability "Travel" { id: "OC-1" }
        }
        system_analysis "SA" {
            capability "Brake" { id: "CAP-1" involves: ["Travel"] }
        }"#,
    )
    .expect_err("a capability does not involve a capability");
    assert!(
        error.contains("capability 'Brake' involves: 'Travel' is a OperationalCapability, which a capability cannot involve"),
        "got: {error}"
    );
}

#[test]
fn capability_specializing_itself_through_others_is_a_compile_error() {
    let error = compile(
        r#"system_analysis "SA" {
            capability "A" { id: "CAP-A" specializes: ["CAP-B"] }
            capability "B" { id: "CAP-B" specializes: ["CAP-C"] }
            capability "C" { id: "CAP-C" specializes: ["CAP-A"] }
        }"#,
    )
    .expect_err("a generalization cycle has no meaning");
    assert!(
        error.contains("capability 'A' specializes: cycle CAP-A -> CAP-B -> CAP-C -> CAP-A"),
        "got: {error}"
    );
}

#[test]
fn capability_relation_written_twice_is_a_compile_error() {
    let same_target = compile(
        r#"system_analysis "SA" {
            capability "A" { id: "CAP-A" includes: ["B", "CAP-B"] }
            capability "B" { id: "CAP-B" }
        }"#,
    )
    .expect_err("one relation, one target, once");
    assert!(
        same_target.contains("capability 'A' includes: 'B' is named twice"),
        "got: {same_target}"
    );

    let same_key = compile(
        r#"system_analysis "SA" {
            capability "A" { id: "CAP-A" includes: ["B"] includes: ["C"] }
            capability "B" { id: "CAP-B" }
            capability "C" { id: "CAP-C" }
        }"#,
    )
    .expect_err("an attribute written twice would lose one value");
    assert!(
        same_key.contains("capability 'A' declares attribute 'includes' twice"),
        "got: {same_key}"
    );
}

#[test]
fn capability_view_reports_an_involvement_it_cannot_tell_apart() {
    // Reachable only for models built outside the compiler, which rejects
    // them: the drawing must not pick one of two same-named elements.
    let mut model = compile(
        r#"system_analysis "SA" {
            function "Scan" { id: "SF-1" }
            function "Scan" { id: "SF-2" }
            capability "Brake" { id: "CAP-1" }
        }"#,
    )
    .expect("compiles");
    model.system_analysis[0].capabilities[0]
        .involves
        .push("Scan".to_string());

    let set = build_diagrams(&model);
    assert!(
        set.diagnostics
            .iter()
            .any(|d| d.contains("capability 'Brake'") && d.contains("'Scan' is ambiguous")),
        "got {:?}",
        set.diagnostics
    );
    assert!(diagram(&set, "cap").edges.is_empty());
}

#[test]
fn capability_reference_that_is_not_a_name_is_a_compile_error() {
    let error = compile(
        r#"system_analysis "SA" {
            mission "Protect" { id: "MIS-1" }
            capability "Brake" {
                id: "CAP-1"
                extends: 5
                involves: ["Fine", 3]
                mission: ["MIS-1"]
            }
        }"#,
    )
    .expect_err("a reference is a name, never a number or a nested list");
    for expected in [
        "capability 'Brake' extends: expected a name or a list of names, got '5'",
        "capability 'Brake' involves: expected a name or a list of names, got '3'",
        "capability 'Brake' mission: expected one name, got a list",
    ] {
        assert!(error.contains(expected), "missing `{expected}` in: {error}");
    }
}

#[test]
fn capability_realizes_only_a_capability_of_the_level_above() {
    let sideways = compile(
        r#"system_analysis "SA" {
            capability "Brake" { id: "CAP-1" }
            capability "Stop" { id: "CAP-2" realizes: "CAP-1" }
        }"#,
    )
    .expect_err("a system capability realizes an operational one");
    assert!(
        sideways.contains(
            "capability 'Stop' realizes: 'Brake' is a SystemCapability, expected a OperationalCapability"
        ),
        "got: {sideways}"
    );

    let not_a_mission = compile(
        r#"system_analysis "SA" {
            function "Scan" { id: "SF-1" }
            capability "Brake" { id: "CAP-1" mission: "Scan" }
        }"#,
    )
    .expect_err("a mission reference names a mission");
    assert!(
        not_a_mission
            .contains("capability 'Brake' mission: 'Scan' is a SystemFunction, expected a Mission"),
        "got: {not_a_mission}"
    );

    let operational = compile(
        r#"operational_analysis "OA" {
            operational_capability "Travel" { id: "OC-1" }
            operational_capability "Board" { id: "OC-2" realizes: "OC-1" }
        }"#,
    )
    .expect_err("nothing sits above the operational level");
    assert!(
        operational.contains(
            "operational_capability 'Board' realizes: an operational capability realizes nothing"
        ),
        "got: {operational}"
    );
}

#[test]
fn realizes_by_a_name_shared_across_levels_picks_the_level_above() {
    let model = Compiler::new(CompilerConfig::default())
        .compile_string(
            r#"operational_analysis "OA" {
                operational_capability "Avoid Collisions" { id: "OC-1" }
            }
            system_analysis "SA" {
                capability "Avoid Collisions" { id: "CAP-1" realizes: "Avoid Collisions" }
            }
            logical_architecture "LA" {
                capability_realization "Avoid Collisions" { id: "CR-1" realizes: "Avoid Collisions" }
            }"#,
        )
        .expect("the level above disambiguates the name")
        .semantic_model;
    let realized = |id: &str| {
        model
            .capabilities
            .iter()
            .find(|c| c.id == id)
            .and_then(|c| c.realizes.clone())
    };
    assert_eq!(realized("CAP-1").as_deref(), Some("OC-1"));
    assert_eq!(realized("CR-1").as_deref(), Some("CAP-1"));
}

#[test]
fn sysml_export_keeps_capability_nesting_and_relations() {
    let result = Compiler::new(CompilerConfig::default())
        .compile_string(
            r#"system_analysis "SA" {
                capability "Brake" { id: "CAP-1" }
                capability "Emergency Brake" {
                    id: "CAP-2"
                    specializes: ["Brake"]
                    includes: ["Warn"]
                    extends: ["Brake"]
                    capability "Detect" { id: "CAP-4" }
                }
                capability "Warn" { id: "CAP-3" }
            }
            logical_architecture "LA" {
                capability_realization "Brake Logic" { id: "CR-1" realizes: "CAP-1" }
            }"#,
        )
        .expect("compiles");
    let sysml =
        arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);

    for expected in [
        "use case def <'CAP-2'> Emergency_Brake :> Brake {",
        "include use case uc_CAP_3 : Warn;",
        "use case uc_CAP_4 : Detect;",
        "dependency extends from Emergency_Brake to Brake;",
        "use case def <'CR-1'> Brake_Logic {",
    ] {
        assert!(
            sysml.contains(expected),
            "missing `{expected}` in:\n{sysml}"
        );
    }
    assert!(
        !sysml.contains("attribute specializes") && !sysml.contains("attribute includes"),
        "relations are relations, not text attributes:\n{sysml}"
    );
}

#[test]
fn identifier_that_is_not_text_is_a_compile_error() {
    let capability = compile(r#"system_analysis "SA" { capability "Brake" { id: ["CAP-1"] } }"#)
        .expect_err("an id is one text value");
    assert!(
        capability.contains("capability 'Brake': id must be text"),
        "got: {capability}"
    );

    let class = compile(r#"class "Frame" { id: 5 range: "float" }"#)
        .expect_err("an id is one text value");
    assert!(
        class.contains("class 'Frame': id must be text"),
        "got: {class}"
    );
}

#[test]
fn sysml_export_keeps_what_a_capability_involves() {
    let result = Compiler::new(CompilerConfig::default())
        .compile_string(ACTORS)
        .expect("compiles");
    let sysml =
        arclang::compiler::sysmlv2_generator::generate_sysmlv2(&result.semantic_model, &result.ast);

    assert!(
        sysml.contains(r#"attribute involves : String = "SF-1, ACT-1";"#),
        "system capability:\n{sysml}"
    );
    assert!(
        sysml.contains(r#"attribute involves : String = "OA-ACT-1, OE-1, OA-1";"#),
        "operational capability:\n{sysml}"
    );
}
