//! The diagrams of the `.aird`: each Capella diagram that shows imported
//! elements becomes a `diagram` declaration on the ArcLang view of its
//! kind. A diagram of a state machine, a scenario, a chain or a path is
//! the element's own view in ArcLang: it is noted, not declared.

use super::{first_ref, refs, Node, Writer};
use crate::compiler::source_edit::string_literal;
use std::collections::{HashMap, HashSet};

/// The ArcLang view a Capella diagram stands for, from the prefix Capella
/// puts in its name (`[LAB] ...`), else from its description.
fn view_of(name: &str, description: &str) -> Option<&'static str> {
    let prefix = name.trim_start().strip_prefix('[').and_then(|rest| rest.split(']').next()).unwrap_or("");
    let by_prefix = match prefix {
        "OAB" | "OEB" | "OEBD" | "OAIB" | "ORB" | "OABD" | "OCB" => Some("oab"),
        "SAB" | "SDFB" | "CSA" | "SFBD" | "CEI" | "CDI" | "CII" | "IDB" => Some("sab"),
        "LAB" | "LDFB" | "LCBD" | "LFBD" => Some("lab"),
        "PAB" | "PDFB" | "PCBD" | "PFBD" => Some("pab"),
        "MCB" | "CC" | "COC" | "CRB" | "MB" | "CM" | "CTX" => Some("cap"),
        "CDB" | "CDI_DATA" => Some("cdb"),
        "CIBD" | "EAB" | "CBD" => Some("pbs"),
        _ => None,
    };
    if by_prefix.is_some() {
        return by_prefix;
    }
    let kind = description.rsplit("name='").next().unwrap_or("").trim_end_matches("']").replace("%20", " ");
    match kind.as_str() {
        "Operational Architecture Blank" | "Operational Entity Blank" | "Operational Entity Breakdown" | "Operational Activity Breakdown" | "Operational Capabilities Blank" => Some("oab"),
        "System Architecture Blank" | "System Data Flow Blank" | "Contextual System Actors" | "System Function Breakdown" => Some("sab"),
        "Logical Architecture Blank" | "Logical Data Flow Blank" | "Logical Component Breakdown" | "Logical Function Breakdown" => Some("lab"),
        "Physical Architecture Blank" | "Physical Data Flow Blank" | "Physical Component Breakdown" | "Physical Function Breakdown" => Some("pab"),
        "Missions Capabilities Blank" | "Contextual Capability" | "Contextual Operational Capability" | "Missions Blank" | "Contextual Mission" => Some("cap"),
        "Class Diagram Blank" => Some("cdb"),
        "Configuration Items Breakdown" | "EPBS Architecture Blank" => Some("pbs"),
        _ => None,
    }
}

/// Diagrams that are an element's own view in ArcLang, by prefix.
fn own_view_of(name: &str) -> Option<&'static str> {
    let prefix = name.trim_start().strip_prefix('[').and_then(|rest| rest.split(']').next()).unwrap_or("");
    match prefix {
        "MSM" | "M&S" => Some("the state machine's own view"),
        "ES" | "FS" | "IS" | "OES" | "OAS" => Some("the scenario's own view"),
        "SFCD" | "LFCD" | "PFCD" | "OPD" => Some("the chain's own listing"),
        "PPD" => Some("the physical path's own listing"),
        _ => None,
    }
}

/// The view most of the shown elements belong to.
fn view_by_kinds(writer: &Writer, shown: &[String]) -> Option<&'static str> {
    let mut votes: HashMap<&'static str, usize> = HashMap::new();
    for id in shown {
        let view = match writer.kinds.get(id).copied() {
            Some("Actor" | "OperationalEntity" | "OperationalActivity" | "OperationalExchange" | "OperationalProcess" | "CommunicationMean") => "oab",
            Some("OperationalCapability" | "Mission" | "Capability" | "CapabilityRealization") => "cap",
            Some("SystemActor" | "SystemComponent" | "SystemFunction" | "FunctionalExchange" | "FunctionalChain" | "FunctionPort") => "sab",
            Some("LogicalComponent" | "LogicalFunction" | "ComponentExchange" | "LogicalInterface" | "ComponentPort") => "lab",
            Some("PhysicalNode" | "BehaviorComponent" | "PhysicalLink" | "PhysicalExchange" | "PhysicalPath" | "PhysicalPort") => "pab",
            Some("EpbsSystem" | "EpbsSubsystem" | "EpbsItem") => "pbs",
            Some("Class" | "Enumeration" | "DataType" | "ExchangeItem") => "cdb",
            _ => continue,
        };
        *votes.entry(view).or_insert(0) += 1;
    }
    votes.into_iter().max_by_key(|(_, n)| *n).map(|(view, _)| view)
}

/// The `href` of a child element: `Model.capella#id`.
fn hrefs<'a>(node: &'a Node, tag: &'a str) -> impl Iterator<Item = String> + 'a {
    node.tagged(tag).flat_map(|child| refs(child.attr("href")).into_iter())
}

pub(super) fn diagrams(writer: &mut Writer, aird: &Node) {
    let everything = aird.descendants();
    let descriptors: Vec<&Node> = everything.iter().copied().filter(|n| n.tag == "ownedRepresentationDescriptors").collect();
    let representations: HashMap<String, &Node> = everything
        .iter()
        .copied()
        .filter(|n| n.tag == "ownedRepresentations" || n.tag.ends_with("DSemanticDiagram"))
        .filter_map(|n| n.attr("uid").map(|uid| (uid.to_string(), n)))
        .collect();
    let mut taken: HashSet<String> = HashSet::new();
    for descriptor in descriptors {
        let name = descriptor.name();
        let description = descriptor.tagged("description").next().and_then(|d| d.attr("href")).unwrap_or("");
        if let Some(own) = own_view_of(name) {
            writer.count(match own {
                "the state machine's own view" => "diagrams of state machines (each machine is a view of its own)",
                "the scenario's own view" => "diagrams of scenarios (each scenario is a view of its own)",
                "the chain's own listing" => "diagrams of chains and processes (a chain is listed, not drawn)",
                _ => "diagrams of physical paths (a path is listed, not drawn)",
            });
            continue;
        }
        let Some(diagram) = first_ref(descriptor.attr("repPath")).and_then(|path| representations.get(&path).copied()) else {
            writer.note(format!("diagram '{name}' left out: its representation was not found in the .aird"));
            continue;
        };
        let mut shown: Vec<String> = Vec::new();
        let mut seen = HashSet::new();
        for element in diagram.descendants().into_iter().filter(|e| e.tag == "ownedDiagramElements") {
            for target in hrefs(element, "target").chain(hrefs(element, "semanticElements")) {
                if let Some(id) = writer.ids.get(&target) {
                    if seen.insert(id.clone()) {
                        shown.push(id.clone());
                    }
                }
            }
        }
        // The view: by the diagram's kind, else by what it shows.
        let Some(view) = view_of(name, description).or_else(|| view_by_kinds(writer, &shown)) else {
            writer.note(format!("diagram '{name}' left out: its kind has no ArcLang view"));
            continue;
        };
        // Only the elements the view draws are listed: a view declares what it holds.
        let shown: Vec<String> = shown.into_iter().filter(|id| view_by_kinds(writer, std::slice::from_ref(id)) == Some(view) || view == "cap" && matches!(writer.kinds.get(id).copied(), Some("SystemActor" | "SystemComponent" | "Actor" | "OperationalEntity" | "SystemFunction" | "OperationalActivity" | "FunctionalChain" | "OperationalProcess" | "LogicalComponent"))).collect();
        if shown.is_empty() {
            writer.note(format!("diagram '{name}' left out: it shows no imported element"));
            continue;
        }
        writer.count("diagrams");
        let unique = super::data::unique_name(&mut taken, name);
        let list: Vec<String> = shown.iter().map(|id| string_literal(id)).collect();
        writer.line(0, &format!("diagram {} {{", string_literal(&unique)));
        writer.line(1, &format!("view: {view}"));
        writer.line(1, &format!("shows: [{}]", list.join(", ")));
        writer.line(0, "}");
        writer.line(0, "");
    }
}
