//! The Capella Cybersecurity viewpoint in the language: `security_scales`
//! (the scales of the needs and the threat kinds), `security_analysis` with
//! its threats and primary assets, threats realizing threats by a trace,
//! and scenarios owned by a threat. Also the integer/decimal distinction of
//! attribute values the round trip of property values needs.

use arclang::compiler::ast::{AssetKind, AttributeValue};
use arclang::compiler::elements;
use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"model T { name: "t" }
exchange_item "Visuals" { id: "EI-Vis" mechanism: "FLOW" }
class "Frame" { id: "CL-Frame" width: "Integer" }
security_scales {
    id: "CFG-1"
    confidentiality: ["0", "1", "2", "3"]
    integrity: ["0", "1", "2", "3"]
    availability: ["0", "1", "2", "3"]
    traceability: ["0", "1", "2", "3"]
    threat_kinds: ["Eavesdropping", "Denial of service", "Theft"]
}
operational_analysis "OA" {
    entity "Ministry" { id: "OE-Min" activity "Monitor crowd" { id: "OA-Mon" } }
    actor "Foreign agent" { id: "OA-Agent" }
}
system_analysis "SA" {
    id: "ARCH-SA"
    function "Provide visuals" { id: "SF-Vis" }
    system_component "Surveillance" { id: "SC-Surv" }
    actor "Public forces" { id: "SA-Forces" }
    functional_chain "Jamming" { id: "SFC-Jam" owner: "TH-SA-1" involves: ["SF-Vis"] }
}
security_analysis {
    level: "oa"
    id: "PKG-OA"
    threat "Block visualization of public events" {
        id: "TH-OA-1"
        kind: "Denial of service"
        level: 2
        rationale: "A foreign agent wants to block the visualization of public events"
        security_confidentiality: "0"
        security_availability: "2"
        applies_to: ["AS-OA-1"]
        involves: ["OE-Min"]
    }
    asset "Provision of public event visuals" {
        id: "AS-OA-1"
        kind: "functional"
        members: ["OA-Mon"]
        security_availability: "2"
    }
}
security_analysis {
    level: "sa"
    threat "Block provision of visuals" {
        id: "TH-SA-1"
        kind: "Denial of service"
        level: 2
        applies_to: ["Service of supply of visuals"]
        involves: ["SC-Surv"]
    }
    asset "Service of supply of visuals" {
        id: "AS-SA-1"
        kind: "information"
        members: ["SF-Vis", "EI-Vis", "CL-Frame"]
        security_confidentiality: "1"
    }
}
trace "TH-SA-1" realizes "TH-OA-1"
scenario "Blocking attack" {
    id: "SC-1"
    owner: "TH-SA-1"
    participants: ["SA-Forces", "SC-Surv"]
    message "SA-Forces" -> "SC-Surv" "jam"
}
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
fn threats_and_assets_are_parsed_with_their_references() {
    let result = compile(MODEL);
    let scales = result.ast.security_scales.as_ref().expect("scales");
    assert_eq!(scales.id, "CFG-1");
    assert_eq!(scales.scale("threat_kinds").unwrap(), vec!["Eavesdropping", "Denial of service", "Theft"]);
    assert_eq!(result.ast.security_analysis.len(), 2);
    let oa = &result.ast.security_analysis[0];
    assert_eq!(oa.level(), Some("oa"));
    assert_eq!(oa.id, "PKG-OA");
    let threat = &oa.threats[0];
    assert_eq!(threat.id, "TH-OA-1");
    assert_eq!(threat.kind.as_deref(), Some("Denial of service"));
    assert_eq!(threat.attributes.get("level"), Some(&AttributeValue::Integer(2)));
    assert_eq!(threat.applies_to, vec!["AS-OA-1"]);
    assert_eq!(threat.involves, vec!["OE-Min"]);
    let asset = &result.ast.security_analysis[1].assets[0];
    assert_eq!(asset.kind, AssetKind::Information);
    assert_eq!(asset.members, vec!["SF-Vis", "EI-Vis", "CL-Frame"]);
    assert!(result.warnings.iter().all(|w| !w.contains("owner")), "{:?}", result.warnings);
}

#[test]
fn a_chain_owner_must_be_a_declared_element() {
    let unknown = MODEL.replace(r#"owner: "TH-SA-1" involves"#, r#"owner: "TH-Nope" involves"#);
    let result = compile(&unknown);
    assert!(result.warnings.iter().any(|w| w.contains("functional_chain 'Jamming': owner 'TH-Nope'")), "{:?}", result.warnings);
}

#[test]
fn threats_and_assets_are_elements_joined_by_their_relationships() {
    let result = compile(MODEL);
    assert!(result.semantic_model.traces.iter().any(|t| t.from == "TH-SA-1" && t.to == "TH-OA-1"), "{:?}", result.semantic_model.traces);
    let graph = elements::build(&result.ast, &result.semantic_model);
    assert_eq!(graph.elements.iter().filter(|e| e.kind == "Threat").count(), 2);
    assert_eq!(graph.elements.iter().filter(|e| e.kind == "PrimaryAsset").count(), 2);
    let threat = graph.elements.iter().find(|e| e.id == "TH-OA-1").unwrap();
    assert_eq!(threat.attributes.get("kind").and_then(|v| v.as_string()), Some("Denial of service"));
    assert_eq!(graph.relationships.iter().filter(|r| r.kind == "ThreatApplication").count(), 2, "{:?}", graph.unresolved);
    assert_eq!(graph.relationships.iter().filter(|r| r.kind == "ThreatInvolvement").count(), 2, "{:?}", graph.unresolved);
    assert_eq!(graph.relationships.iter().filter(|r| r.kind == "AssetMember").count(), 4, "{:?}", graph.unresolved);
    assert!(graph.unresolved.is_empty(), "{:?}", graph.unresolved);
}

#[test]
fn a_threat_kind_must_be_one_of_the_declared_threat_kinds() {
    let unknown = MODEL.replace(r#"kind: "Denial of service"
        level: 2
        rationale"#, r#"kind: "Magic"
        level: 2
        rationale"#);
    let error = fails(&unknown);
    assert!(error.contains("Magic") && error.contains("Eavesdropping"), "{error}");
    // Without declared scales any kind is accepted.
    let free = unknown.replace(r#"threat_kinds: ["Eavesdropping", "Denial of service", "Theft"]"#, "");
    compile(&free);
}

#[test]
fn applies_to_names_assets_and_involves_names_declared_elements() {
    let not_an_asset = MODEL.replace(r#"applies_to: ["AS-OA-1"]"#, r#"applies_to: ["OE-Min"]"#);
    assert!(fails(&not_an_asset).contains("PrimaryAsset"));
    let unknown = MODEL.replace(r#"involves: ["OE-Min"]"#, r#"involves: ["Nobody"]"#);
    assert!(fails(&unknown).contains("Nobody"));
    let unknown_member = MODEL.replace(r#"members: ["OA-Mon"]"#, r#"members: ["Nothing"]"#);
    assert!(fails(&unknown_member).contains("Nothing"));
}

#[test]
fn asset_kind_and_threat_level_are_checked_at_parse_time() {
    let kind = MODEL.replace(r#"kind: "information""#, r#"kind: "secret""#);
    assert!(fails(&kind).contains("functional, information, enterprise"));
    let level = MODEL.replace("level: 2\n        rationale", "level: 2.5\n        rationale");
    assert!(fails(&level).contains("level must be an integer"));
    let bad_level = MODEL.replace(r#"level: "oa""#, r#"level: "xx""#);
    assert!(fails(&bad_level).contains("oa, sa, la, pa"));
}

#[test]
fn a_security_need_off_its_scale_is_a_warning() {
    let off = MODEL.replace(r#"security_confidentiality: "1""#, r#"security_confidentiality: "9""#);
    let result = compile(&off);
    assert!(result.warnings.iter().any(|w| w.contains("security_confidentiality '9'")), "{:?}", result.warnings);
}

#[test]
fn integers_and_decimals_are_kept_apart() {
    let result = compile(r#"model T { name: "t" }
system_analysis "SA" {
    function "F" { id: "SF-1" count: 800 mass: 800.0 ratio: 0.5 negative: -3 }
}
"#);
    let function = &result.ast.system_analysis[0].functions[0];
    assert_eq!(function.attributes.get("count"), Some(&AttributeValue::Integer(800)));
    assert_eq!(function.attributes.get("mass"), Some(&AttributeValue::Number(800.0)));
    assert_eq!(function.attributes.get("negative"), Some(&AttributeValue::Integer(-3)));
    assert_eq!(function.attributes["count"].display(), "800");
    assert_eq!(function.attributes["mass"].display(), "800.0");
    assert_eq!(function.attributes["ratio"].display(), "0.5");
    assert_eq!(function.attributes["mass"].as_number(), Some(800.0));
    assert_eq!(function.attributes["count"].as_number(), Some(800.0));
}
