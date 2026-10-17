//! Add-on viewpoints of Capella: STPA (labs4capella) and Cybersecurity
//! (eclipse-capella). What they hold is written where the language has a
//! place for it: STPA hazards become hazards, their system constraints the
//! requirements that mitigate them, the losses a hazard leads to a list on
//! it; the security needs and storage of a function, an exchange or a
//! component become attributes of that element ([`extension_values`]).
//! Everything else is counted and named in the notes, never dropped in
//! silence.

use super::{display_name, list_literal, refs, text_attr, Node, PropertyValue, Writer};
use crate::compiler::source_edit::string_literal;
use std::collections::{BTreeMap, HashMap};

/// The STPA and Cybersecurity kinds the language has no place for yet,
/// with what they are.
const NOT_PLACED: &[(&str, &str)] = &[
    ("stpa:UnsafeControlAction", "STPA unsafe control actions"),
    ("stpa:LossScenario", "STPA loss scenarios"),
    ("stpa:ControlEntity", "STPA control structure entities"),
    ("stpa:ControlAction", "STPA control actions"),
    ("stpa:ControlRelation", "STPA control relations"),
    ("stpa:FeedbackRelation", "STPA feedback relations"),
    ("stpa:FeedbackElement", "STPA feedback"),
    ("stpa:Responsibility", "STPA responsibilities"),
    ("stpa:ProcessModelVariable", "STPA process model variables"),
    ("stpa:InformationExchangeRelation", "STPA information exchanges"),
    ("stpa:PieceOfInformation", "STPA pieces of information"),
    ("stpa:ControlFlaw", "STPA control flaws"),
    ("stpa:ControlFlawSet", "STPA control flaw sets"),
    ("stpa:UnsafeControlActionCategory", "STPA unsafe control action categories"),
    ("stpa:DecompositionCategory", "STPA hazard decomposition categories"),
    ("cybersecurity:Threat", "cybersecurity threats"),
    ("cybersecurity:ThreatInvolvement", "components involved in a threat"),
    ("cybersecurity:ThreatApplication", "assets a threat applies to"),
    ("cybersecurity:ThreatSourceUse", "actors used as a threat source"),
    ("cybersecurity:FunctionalPrimaryAsset", "functional primary assets"),
    ("cybersecurity:InformationPrimaryAsset", "information primary assets"),
    ("cybersecurity:EnterprisePrimaryAsset", "enterprise primary assets"),
    ("cybersecurity:PrimaryAssetMember", "members of primary assets"),
    ("cybersecurity:CybersecurityConfiguration", "the cybersecurity configuration (scales of the needs)"),
];

/// A viewpoint attribute of the element it extends: `(Capella attribute, ArcLang key)`.
const EXTENSIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "cybersecurity:SecurityNeeds",
        &[
            ("confidentialityValue", "security_confidentiality"),
            ("integrityValue", "security_integrity"),
            ("availabilityValue", "security_availability"),
            ("traceabilityValue", "security_traceability"),
        ],
    ),
    ("cybersecurity:FunctionStorage", &[("dataStorage", "data_storage"), ("remanentData", "remanent_data")]),
    (
        "cybersecurity:TrustBoundaryStorage",
        &[("trusted", "trusted"), ("threatSource", "threat_source"), ("threatSourceProfile", "threat_source_profile"), ("rationale", "trust_rationale")],
    ),
];

/// The prefixes of Capella's own metamodel, as the importer shortens them.
const CORE: &[&str] = &["oa", "ctx", "la", "pa", "epbs", "fa", "cs", "information", "datatype", "datavalue", "interaction", "capellacommon", "capellacore", "capellamodeller", "libraries", "deployment", "communication", "activity", "behavior", "modellingcore", "re", "Requirements", "CapellaRequirements", "filtering"];

/// A viewpoint element of this kind (`stpa:Hazard` is `Hazard` of the STPA
/// viewpoint): the prefix of an add-on is the file's choice, its kind name is not.
fn is_viewpoint(node: &Node, kind: &str) -> bool {
    let local = kind.rsplit(':').next().unwrap_or(kind);
    match node.kind.rsplit_once(':') {
        Some((prefix, name)) => name == local && !CORE.contains(&prefix),
        None => false,
    }
}

/// Every viewpoint element of this kind under `node`.
fn all_of<'a>(node: &'a Node, kind: &str) -> Vec<&'a Node> {
    node.descendants().into_iter().filter(|n| is_viewpoint(n, kind)).collect()
}

fn literal(raw: &str) -> String {
    let raw = raw.trim();
    if raw == "true" || raw == "false" || raw.parse::<f64>().is_ok() {
        raw.to_string()
    } else {
        string_literal(raw)
    }
}

/// The viewpoint attributes of the elements that viewpoint extensions
/// (`ownedExtensions`) extend, by Capella id of the extended element, in the
/// order of the document. An attribute Capella leaves at its default is not
/// in the file, and is not written.
pub(super) fn extension_values(project: &Node) -> HashMap<String, Vec<PropertyValue>> {
    let mut values: HashMap<String, Vec<PropertyValue>> = HashMap::new();
    for element in project.descendants() {
        for extension in element.tagged("ownedExtensions") {
            let Some((_, keys)) = EXTENSIONS.iter().find(|(kind, _)| is_viewpoint(extension, kind)) else { continue };
            for (attribute, key) in keys.iter() {
                if let Some(raw) = extension.attr(attribute) {
                    values.entry(element.id().to_string()).or_default().push(PropertyValue { key: key.to_string(), literal: literal(raw) });
                }
            }
        }
    }
    values
}

/// The STPA hazards (with their sub-hazards and the losses they lead to) and
/// the system constraints that mitigate them; the rest of the viewpoints is
/// counted in the notes.
pub(super) fn viewpoints(writer: &mut Writer, project: &Node) {
    let constraints = all_of(project, "stpa:SystemConstraint");
    if !constraints.is_empty() {
        writer.line(0, "requirements stpa {");
        for constraint in &constraints {
            let id = writer.remember(constraint, "Requirement");
            writer.count("STPA system constraints (as requirements)");
            writer.line(1, &format!("req {} {} {{", string_literal(&id), string_literal(&display_name(constraint))));
            writer.line(2, &format!("id: {}", string_literal(&id)));
            if let Some(user_id) = constraint.attr("userID") {
                writer.line(2, &format!("stpa_id: {}", string_literal(user_id)));
            }
            writer.line(2, &format!("description: {}", string_literal(&text_attr(constraint, "description").unwrap_or_else(|| display_name(constraint)))));
            writer.line(1, "}");
        }
        writer.line(0, "}");
        writer.line(0, "");
    }

    let losses: HashMap<String, String> = all_of(project, "stpa:Loss").into_iter().map(|loss| (loss.id().to_string(), loss_label(loss))).collect();
    writer.count_by("STPA losses (as the losses of their hazards)", losses.len());
    let hazards = all_of(project, "stpa:Hazard");
    if !hazards.is_empty() {
        let parents: HashMap<String, String> =
            hazards.iter().flat_map(|parent| parent.tagged("subHazards").map(move |sub| (sub.id().to_string(), parent.id().to_string()))).collect();
        writer.line(0, "safety_analysis {");
        for hazard in &hazards {
            let id = writer.remember(hazard, "Hazard");
            writer.count("STPA hazards");
            writer.line(1, &format!("hazard {} {{", string_literal(&display_name(hazard))));
            writer.line(2, &format!("id: {}", string_literal(&id)));
            if let Some(user_id) = hazard.attr("userID") {
                writer.line(2, &format!("stpa_id: {}", string_literal(user_id)));
            }
            if let Some(description) = text_attr(hazard, "description") {
                writer.line(2, &format!("description: {}", string_literal(&super::plain(&description))));
            }
            if let Some(parent) = parents.get(&id) {
                writer.line(2, &format!("sub_hazard_of: {}", string_literal(parent)));
            }
            let led_to: Vec<String> = refs(hazard.attr("losses")).iter().filter_map(|loss| losses.get(loss).cloned()).collect();
            if !led_to.is_empty() {
                writer.line(2, &format!("losses: {}", list_literal(&led_to)));
            }
            // A constraint names its hazards, a hazard its constraints: either side says it.
            let mut mitigated: Vec<String> = refs(hazard.attr("systemConstraints"));
            for constraint in &constraints {
                if refs(constraint.attr("hazards")).contains(&id) && !mitigated.iter().any(|m| m == constraint.id()) {
                    mitigated.push(constraint.id().to_string());
                }
            }
            if !mitigated.is_empty() {
                writer.line(2, &format!("mitigated_by: {}", list_literal(&mitigated)));
            }
            writer.line(1, "}");
        }
        writer.line(0, "}");
        writer.line(0, "");
    }

    let mut left_out: BTreeMap<&str, usize> = BTreeMap::new();
    for node in project.descendants() {
        if let Some((_, what)) = NOT_PLACED.iter().find(|(kind, _)| is_viewpoint(node, kind)) {
            *left_out.entry(what).or_insert(0) += 1;
        }
    }
    for (what, count) in left_out {
        writer.note(format!("{count} {what} left out: ArcLang has no place for them yet"));
    }
}

fn loss_label(loss: &Node) -> String {
    match loss.attr("userID") {
        Some(user_id) if !user_id.is_empty() => format!("{user_id} {}", display_name(loss)),
        _ => display_name(loss),
    }
}

#[cfg(test)]
mod tests {
    use super::super::{parse, Writer};
    use super::*;
    use crate::compiler::metamodel::Metamodel;
    use crate::compiler::{Compiler, CompilerConfig};
    use std::collections::{BTreeMap, HashMap};

    const STPA: &str = r##"<org.polarsys.capella.core.data.capellamodeller:Project xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:stpa="http://www.thalesgroup.com/capella/stpa" id="p" name="P">
  <ownedModelRoots xsi:type="org.polarsys.capella.core.data.capellamodeller:SystemEngineering" id="se" name="P">
    <ownedExtensions xsi:type="stpa:StpaAnalysis" id="stpa" name="STPA">
      <lossPackage xsi:type="stpa:LossPackage" id="lp" name="Losses">
        <losses xsi:type="stpa:Loss" id="l1" name="Loss of life or injury" userID="L-1" hazards="#h1"/>
      </lossPackage>
      <hazardConstraintPackage xsi:type="stpa:HazardConstraintPackage" id="hcp" name="Hazards">
        <rootHazards xsi:type="stpa:Hazard" id="h1" name="Aircraft too close to terrain" userID="H-1" losses="#l1" systemConstraints="#sc1">
          <subHazards xsi:type="stpa:Hazard" id="h1-1" name="Descent too steep" userID="H-1.1"/>
        </rootHazards>
        <rootSystemConstraints xsi:type="stpa:SystemConstraint" id="sc1" name="Keep a safe distance from terrain" userID="SC-1" hazards="#h1"/>
        <rootSystemConstraints xsi:type="stpa:SystemConstraint" id="sc2" name="Limit the descent rate" userID="SC-2" hazards="#h1-1"/>
      </hazardConstraintPackage>
      <lossScenarioPackage xsi:type="stpa:LossScenarioPackage" id="lsp" name="Scenarios">
        <lossScenarios xsi:type="stpa:LossScenario" id="ls1" name="LS-1" hazards="#h1"/>
      </lossScenarioPackage>
    </ownedExtensions>
  </ownedModelRoots>
</org.polarsys.capella.core.data.capellamodeller:Project>"##;

    fn writer() -> Writer {
        Writer {
            out: String::new(),
            counts: BTreeMap::new(),
            notes: Vec::new(),
            ids: HashMap::new(),
            kinds: HashMap::new(),
            names: HashMap::new(),
            port_owner: HashMap::new(),
            port_ref: HashMap::new(),
            part_type: HashMap::new(),
            traces: Vec::new(),
            value_groups: HashMap::new(),
            values: HashMap::new(),
            categories: HashMap::new(),
            status_literals: HashMap::new(),
            extension_values: HashMap::new(),
            engineering: Node::default(),
            metamodel: Metamodel::current(),
        }
    }

    fn written(xml: &str) -> Writer {
        let document = parse(xml).expect("xml");
        let mut writer = writer();
        writer.line(0, "model \"P\" { name: \"P\" }");
        viewpoints(&mut writer, &document.children[0]);
        writer
    }

    #[test]
    fn stpa_hazards_and_their_constraints_compile_as_hazards_mitigated_by_requirements() {
        let writer = written(STPA);
        let source = &writer.out;
        Compiler::new(CompilerConfig { optimization_level: 0, target: "json".to_string() }).compile_string(source).unwrap_or_else(|e| panic!("{e}\n{source}"));
        assert!(source.contains("hazard \"Aircraft too close to terrain\""), "{source}");
        assert!(source.contains("stpa_id: \"H-1\""), "{source}");
        assert!(source.contains("losses: [\"L-1 Loss of life or injury\"]"), "{source}");
        assert!(source.contains("mitigated_by: [\"sc1\"]"), "{source}");
        // A constraint that names its hazard mitigates it, though the hazard does not say so.
        assert!(source.contains("sub_hazard_of: \"h1\"") && source.contains("mitigated_by: [\"sc2\"]"), "{source}");
        assert_eq!(writer.counts.get("STPA hazards").copied(), Some(2));
        assert_eq!(writer.counts.get("STPA system constraints (as requirements)").copied(), Some(2));
        assert!(writer.notes.iter().any(|n| n == "1 STPA loss scenarios left out: ArcLang has no place for them yet"), "{:?}", writer.notes);
    }

    #[test]
    fn security_needs_become_attributes_of_the_element_they_extend() {
        let xml = r#"<Project xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <ownedFunctionalExchanges xsi:type="org.polarsys.capella.core.data.fa:FunctionalExchange" id="fe1" name="Credentials">
    <ownedExtensions xsi:type="org.polarsys.capella.cybersecurity.model:SecurityNeeds" id="sn" confidentialityValue="3" integrityValue="2"/>
    <ownedExtensions xsi:type="org.polarsys.capella.cybersecurity.model:FunctionStorage" id="fs" dataStorage="true"/>
  </ownedFunctionalExchanges>
  <ownedComponents xsi:type="org.polarsys.capella.core.data.pa:PhysicalComponent" id="pc1" name="Gateway">
    <ownedExtensions xsi:type="org.polarsys.capella.cybersecurity.model:TrustBoundaryStorage" id="tb" trusted="true" rationale="Hardened"/>
  </ownedComponents>
</Project>"#;
        let document = parse(xml).expect("xml");
        let values = extension_values(&document.children[0]);
        let pairs = |id: &str| -> Vec<(String, String)> { values[id].iter().map(|v| (v.key.clone(), v.literal.clone())).collect() };
        assert_eq!(
            pairs("fe1"),
            vec![("security_confidentiality".into(), "3".into()), ("security_integrity".into(), "2".into()), ("data_storage".into(), "true".into())]
        );
        assert_eq!(pairs("pc1"), vec![("trusted".into(), "true".into()), ("trust_rationale".into(), "\"Hardened\"".into())]);
    }

    #[test]
    fn a_model_without_viewpoints_writes_nothing() {
        let writer = written(r#"<Project id="p" name="P"/>"#);
        assert_eq!(writer.out, "model \"P\" { name: \"P\" }\n");
        assert!(writer.notes.is_empty());
    }
}
