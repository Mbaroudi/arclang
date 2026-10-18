//! The Cybersecurity viewpoint's own elements: the scales of the security
//! needs and the kinds of threat (`security_scales`, from the
//! CybersecurityConfiguration), and per architecture the threats and the
//! primary assets of its CybersecurityPkg (`security_analysis`). A security
//! need names a literal of its scale: it is written by the literal's name.

use super::{display_name, first_ref, header_attributes, list_literal, refs, Node, Writer};
use crate::compiler::source_edit::string_literal;
use std::collections::{HashMap, HashSet};

/// The scales of the configuration: `(Capella attribute, ArcLang key)`.
const SCALES: &[(&str, &str)] = &[
    ("confidentiality", "confidentiality"),
    ("integrity", "integrity"),
    ("availability", "availability"),
    ("traceability", "traceability"),
    ("threatKind", "threat_kinds"),
];

/// The level of an architecture, as `security_analysis` names it.
fn level_of(kind: &str) -> Option<&'static str> {
    match kind.rsplit(':').next()? {
        "OperationalAnalysis" => Some("oa"),
        "SystemAnalysis" => Some("sa"),
        "LogicalArchitecture" => Some("la"),
        "PhysicalArchitecture" => Some("pa"),
        _ => None,
    }
}

/// A viewpoint kind (`Threat`), whatever prefix the file gives the viewpoint.
fn is(node: &Node, name: &str) -> bool {
    node.kind.rsplit_once(':').is_some_and(|(prefix, local)| prefix == "cybersecurity" && local == name)
}

fn configuration(project: &Node) -> Option<&Node> {
    project.descendants().into_iter().find(|n| is(n, "CybersecurityConfiguration"))
}

/// The literal names of the scale an attribute of the configuration names, in order.
fn scale<'a>(configuration: &'a Node, attribute: &str) -> Option<Vec<&'a Node>> {
    let id = first_ref(configuration.attr(attribute))?;
    let scale = configuration.tagged("ownedEnumerationPropertyTypes").find(|t| t.id() == id)?;
    Some(scale.tagged("ownedLiterals").collect())
}

/// The names of the literals of the configuration's scales, by literal id:
/// a security need or a threat kind names one.
pub(super) fn literal_names(project: &Node) -> HashMap<String, String> {
    let Some(configuration) = configuration(project) else { return HashMap::new() };
    SCALES.iter().filter_map(|(attribute, _)| scale(configuration, attribute)).flatten().map(|l| (l.id().to_string(), display_name(l))).collect()
}

/// `security_scales`, then one `security_analysis` per architecture with a
/// CybersecurityPkg, and the realizations between threats as traces.
pub(super) fn write(writer: &mut Writer, project: &Node) {
    let literals = literal_names(project);
    if let Some(configuration) = configuration(project) {
        writer.count("cybersecurity configurations (as security scales)");
        writer.line(0, "security_scales {");
        writer.line(1, &format!("id: {}", string_literal(configuration.id())));
        for (attribute, key) in SCALES {
            if let Some(literals) = scale(configuration, attribute) {
                let names: Vec<String> = literals.iter().map(|l| display_name(l)).collect();
                writer.line(1, &format!("{key}: {}", list_literal(&names)));
            }
        }
        writer.line(0, "}");
        writer.line(0, "");
    }
    let assets: HashSet<String> = project.descendants().into_iter().filter(|n| n.kind.ends_with("PrimaryAsset") && n.kind.starts_with("cybersecurity:")).map(|n| n.id().to_string()).collect();
    for architecture in project.descendants() {
        let Some(level) = level_of(&architecture.kind) else { continue };
        for package in architecture.tagged("ownedExtensions").filter(|e| is(e, "CybersecurityPkg")) {
            analysis(writer, package, level, &literals, &assets);
        }
    }
}

/// The references among `all` that name a written element; the others are counted.
fn written(writer: &mut Writer, all: Vec<String>, what: &'static str) -> Vec<String> {
    let known = writer.known(&all);
    writer.count_by(what, all.len() - known.len());
    known
}

fn analysis(writer: &mut Writer, package: &Node, level: &str, literals: &HashMap<String, String>, assets: &HashSet<String>) {
    writer.line(0, "security_analysis {");
    writer.line(1, &format!("level: {}", string_literal(level)));
    writer.line(1, &format!("id: {}", string_literal(package.id())));
    for threat in package.tagged("ownedThreats").filter(|t| is(t, "Threat")) {
        writer.remember(threat, "Threat");
        writer.count("cybersecurity threats");
        writer.line(1, &format!("threat {} {{", string_literal(&display_name(threat))));
        if let Some(kind) = first_ref(threat.attr("kind")).and_then(|k| literals.get(&k)) {
            writer.line(2, &format!("kind: {}", string_literal(kind)));
        }
        if let Some(level) = threat.attr("level").filter(|l| l.trim().parse::<i64>().is_ok()) {
            writer.line(2, &format!("level: {}", level.trim()));
        }
        if let Some(rationale) = threat.attr("rationale").filter(|r| !r.is_empty()) {
            writer.line(2, &format!("rationale: {}", string_literal(rationale)));
        }
        header_attributes(writer, 2, threat, "Threat");
        let applies: Vec<String> = threat.tagged("ownedThreatApplications").flat_map(|a| refs(a.attr("asset"))).filter(|a| assets.contains(a)).collect();
        if !applies.is_empty() {
            writer.line(2, &format!("applies_to: {}", list_literal(&applies)));
        }
        let involves = written(writer, threat.tagged("ownedThreatInvolvements").flat_map(|i| refs(i.attr("component"))).collect(), "threat involvements of elements not written (left out)");
        if !involves.is_empty() {
            writer.line(2, &format!("involves: {}", list_literal(&involves)));
        }
        writer.line(1, "}");
        for realization in threat.tagged("ownedAbstractCapabilityRealizations") {
            if let Some(target) = first_ref(realization.attr("targetElement")) {
                writer.traces.push((threat.id().to_string(), "realizes", target));
            }
        }
    }
    for asset in package.tagged("ownedPrimaryAssets") {
        let kind = match asset.kind.rsplit(':').next() {
            Some("FunctionalPrimaryAsset") => "functional",
            Some("InformationPrimaryAsset") => "information",
            Some("EnterprisePrimaryAsset") => "enterprise",
            _ => continue,
        };
        writer.remember(asset, "PrimaryAsset");
        writer.count("cybersecurity primary assets");
        writer.line(1, &format!("asset {} {{", string_literal(&display_name(asset))));
        writer.line(2, &format!("kind: {}", string_literal(kind)));
        header_attributes(writer, 2, asset, "PrimaryAsset");
        let members = written(writer, asset.tagged("ownedMembers").flat_map(|m| refs(m.attr("member"))).collect(), "primary asset members not written (left out)");
        if !members.is_empty() {
            writer.line(2, &format!("members: {}", list_literal(&members)));
        }
        writer.line(1, "}");
    }
    writer.line(0, "}");
    writer.line(0, "");
}
