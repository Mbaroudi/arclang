//! The Capella export of the Cybersecurity viewpoint and its import back:
//! the configuration and its scales, a CybersecurityPkg per level with its
//! threats and primary assets, the needs of an element as its extension,
//! and what a threat owns (scenarios, chains, realizations).

use arclang::compiler::capella_export::export;
use arclang::compiler::capella_xmi::import_strings;
use arclang::compiler::{Compiler, CompilerConfig};
use std::collections::HashSet;

const MODEL: &str = r#"model T { name: "t" }
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
        members: ["SF-Vis"]
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

fn compile(source: &str) -> arclang::compiler::ast::Model {
    Compiler::new(CompilerConfig::default()).compile_string(source).unwrap_or_else(|e| panic!("{e}\n{source}")).ast
}

/// The lines of the security blocks and of the threat traces, trimmed.
fn security_lines(source: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut inside = false;
    for line in source.lines() {
        if line.starts_with("security_") {
            inside = true;
        }
        if inside || line.starts_with("trace \"TH-") {
            lines.push(line.trim().to_string());
        }
        if line == "}" {
            inside = false;
        }
    }
    lines.sort();
    lines
}

/// The element of this id: its opening tag and what it holds.
fn element<'a>(capella: &'a str, id: &str) -> &'a str {
    let at = capella.find(&format!("id=\"{id}\"")).unwrap_or_else(|| panic!("{id} in {capella}"));
    let open = capella[..at].rfind('<').unwrap();
    let tag = capella[open + 1..].split(|c: char| c.is_whitespace()).next().unwrap();
    let close = capella[open..].find(&format!("</{tag}>")).map(|c| open + c).unwrap_or(open + capella[open..].find("/>").unwrap());
    &capella[open..close]
}

#[test]
fn threats_and_assets_go_to_the_cybersecurity_package_of_their_level() {
    let exported = export(&compile(MODEL), "model");
    let capella = &exported.capella;
    assert!(capella.contains("xmlns:cybersecurity=\"http://www.polarsys.org/capella/cybersecurity/1.0\""));
    assert!(exported.afm.contains("vpId=\"org.polarsys.capella.cybersecurity\""));
    let threat = element(capella, "TH-SA-1");
    assert!(threat.contains("cybersecurity:Threat") && threat.contains("level=\"2\""), "{threat}");
    assert!(threat.contains("cybersecurity:ThreatApplication") && threat.contains("asset=\"#AS-SA-1\""), "{threat}");
    assert!(threat.contains("component=\"#SC-Surv\""), "{threat}");
    assert!(threat.contains("ownedScenarios") && threat.contains("id=\"SC-1\""), "the threat owns its scenario: {threat}");
    assert!(threat.contains("ownedFunctionalChains") && threat.contains("id=\"SFC-Jam\""), "the threat owns its chain: {threat}");
    assert!(threat.contains("ownedAbstractCapabilityRealizations") && threat.contains("targetElement=\"#TH-OA-1\""), "{threat}");
    assert!(element(capella, "PKG-OA").contains("id=\"TH-OA-1\""));
    let asset = element(capella, "AS-SA-1");
    assert!(asset.contains("cybersecurity:InformationPrimaryAsset") && asset.contains("member=\"#SF-Vis\""), "{asset}");
    assert!(asset.contains("cybersecurity:SecurityNeeds"), "{asset}");
}

#[test]
fn every_need_and_kind_names_a_literal_of_the_configuration() {
    let capella = export(&compile(MODEL), "model").capella;
    let ids: HashSet<&str> = capella.match_indices(" id=\"").map(|(i, _)| capella[i + 5..].split('"').next().unwrap()).collect();
    for attribute in ["confidentialityValue", "availabilityValue", "kind", "asset", "component", "member"] {
        for (i, _) in capella.match_indices(&format!(" {attribute}=\"#")) {
            let target = capella[i + attribute.len() + 4..].split('"').next().unwrap();
            assert!(ids.contains(target), "{attribute} #{target} names nothing");
        }
    }
    let configuration = element(&capella, "CFG-1");
    assert!(configuration.contains("name=\"Threat.Kind\"") && configuration.contains("name=\"Denial of service\""), "{configuration}");
}

#[test]
fn the_security_analysis_comes_back_from_capella_as_it_was_written() {
    let first = export(&compile(MODEL), "model");
    let imported = import_strings(&first.capella, None, "model.capella").unwrap_or_else(|f| panic!("{}", f.message));
    let again = compile(&imported.source);
    let second = export(&again, "model");
    let reimported = import_strings(&second.capella, None, "model.capella").unwrap_or_else(|f| panic!("{}", f.message));
    assert_eq!(security_lines(&imported.source), security_lines(&reimported.source));
    let lines = security_lines(&imported.source);
    for expected in ["kind: \"Denial of service\"", "level: 2", "security_availability: \"2\"", "applies_to: [\"AS-SA-1\"]", "members: [\"SF-Vis\"]", "threat_kinds: [\"Eavesdropping\", \"Denial of service\", \"Theft\"]"] {
        assert!(lines.iter().any(|l| l == expected), "{expected} in {lines:#?}");
    }
    assert!(imported.source.contains("trace \"TH-SA-1\" realizes \"TH-OA-1\""), "{}", imported.source);
    assert!(imported.source.contains("owner: \"TH-SA-1\""), "the scenario and the chain keep their threat");
}
