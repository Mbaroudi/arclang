//! Diagrams, in the `.aird`: each `diagram` declaration becomes a Sirius
//! representation of the Capella diagram kind its name or view gives, with
//! one box per element it shows, nested as Capella draws them (a function
//! in the component that performs it, a component in its parent, a
//! behavior in the node it is deployed on). The mappings and styles of the
//! boxes are those real Capella diagrams use (`sirius_boxes.json`, read
//! from Capella's own models); Capella lays the boxes out when it opens the
//! diagram.

use super::context::{derived, Context};
use super::xml::El;
use crate::compiler::ast::DiagramDecl;
use serde::Deserialize;
use std::collections::HashMap;

/// How a kind of diagram draws an element under a parent of some kind.
#[derive(Deserialize)]
pub(super) struct Box {
    pub(super) parent: String,
    pub(super) target: String,
    #[serde(rename = "type")]
    pub(super) kind: String,
    pub(super) mapping: String,
    pub(super) style: Option<String>,
    pub(super) style_description: String,
}

/// How a kind of diagram draws an exchange, link, chain or path between two ends.
#[derive(Deserialize)]
pub(super) struct Edge {
    pub(super) target: String,
    pub(super) source_kind: String,
    pub(super) target_kind: String,
    pub(super) mapping: String,
    pub(super) style: Option<String>,
    pub(super) style_description: String,
}

#[derive(Deserialize)]
pub(super) struct Representation {
    pub(super) description: String,
    pub(super) boxes: Vec<Box>,
    #[serde(default)]
    pub(super) bordered: Vec<Box>,
    #[serde(default)]
    pub(super) edges: Vec<Edge>,
}

fn table() -> HashMap<String, Representation> {
    serde_json::from_str(include_str!("sirius_boxes.json")).expect("the Sirius mapping table is valid JSON")
}

/// Which element a diagram is drawn on: the first element of its kind
/// (a package), the root function of the level, or the first shown one.
#[derive(Clone, Copy, PartialEq)]
enum On {
    First,
    Root,
    Shown,
}

/// The Capella representation of a diagram: by the `[XXX]` prefix of its
/// name, else by its view (a view showing functions only is a data flow);
/// the kind of element it is drawn on, and which one.
fn representation(diagram: &DiagramDecl, shows_boxes: bool, shows_realizations: bool, shows_interfaces: bool) -> Option<(&'static str, &'static str, On)> {
    let prefix = diagram.name.trim_start().strip_prefix('[').and_then(|r| r.split(']').next()).unwrap_or("");
    let by_prefix = match prefix {
        // An architecture diagram showing functions only is drawn as a data flow:
        // its functions have no component to stand in.
        "LAB" if !shows_boxes => "Logical Data Flow Blank",
        "SAB" if !shows_boxes => "System Data Flow Blank",
        "PAB" if !shows_boxes => "Physical Data Flow Blank",
        "LAB" => "Logical Architecture Blank",
        "SAB" => "System Architecture Blank",
        "PAB" => "Physical Architecture Blank",
        "OAB" | "OEB" => "Operational Entity Blank",
        "OEBD" => "Operational Entity Breakdown",
        "OABD" => "Operational Activity Breakdown",
        "OAIB" | "COAI" => "Operational Activity Interaction Blank",
        "SDFB" | "CSDF" => "System Data Flow Blank",
        "LDFB" | "CLDF" => "Logical Data Flow Blank",
        "PDFB" | "CPDF" => "Physical Data Flow Blank",
        "CDB" => "Class Diagram Blank",
        "MCB" | "MB" => "Missions Capabilities Blank",
        "CC" => "Contextual Capability",
        "CM" => "Contextual Mission",
        "CSA" | "SC" => "Contextual System Actors",
        "OCB" => "Operational Capabilities Blank",
        "COC" => "Contextual Operational Capability",
        "CIBD" | "EAB" | "CBD" => "Configuration Items Breakdown",
        "CRB" => "Capability Realization Blank",
        "CRI" => "Contextual Capability Realization Involvement",
        "IDB" | "ID" => "Interfaces Diagram Blank",
        // An architecture view showing interfaces is an interfaces diagram.
        _ if shows_interfaces && matches!(diagram.view.as_str(), "lab" | "sab" | "pab") => "Interfaces Diagram Blank",
        _ => match (diagram.view.as_str(), shows_boxes) {
            ("lab", true) => "Logical Architecture Blank",
            ("lab", false) => "Logical Data Flow Blank",
            ("sab", true) => "System Architecture Blank",
            ("sab", false) => "System Data Flow Blank",
            ("pab", true) => "Physical Architecture Blank",
            ("pab", false) => "Physical Data Flow Blank",
            ("oab", true) => "Operational Entity Blank",
            ("oab", false) => "Operational Activity Interaction Blank",
            ("cdb", _) => "Class Diagram Blank",
            ("cap", _) if shows_realizations => "Capability Realization Blank",
            ("cap", _) => "Missions Capabilities Blank",
            ("pbs", _) => "Configuration Items Breakdown",
            _ => return None,
        },
    };
    let (on, which) = match by_prefix {
        "Logical Architecture Blank" => ("la:LogicalComponentPkg", On::First),
        "System Architecture Blank" => ("ctx:SystemComponentPkg", On::First),
        "Physical Architecture Blank" => ("pa:PhysicalComponentPkg", On::First),
        "Operational Entity Blank" | "Operational Entity Breakdown" => ("oa:EntityPkg", On::First),
        "Operational Activity Breakdown" | "Operational Activity Interaction Blank" => ("oa:OperationalActivity", On::Root),
        "System Data Flow Blank" => ("ctx:SystemFunction", On::Root),
        "Logical Data Flow Blank" => ("la:LogicalFunction", On::Root),
        "Physical Data Flow Blank" => ("pa:PhysicalFunction", On::Root),
        "Class Diagram Blank" => ("information:DataPkg", On::First),
        "Missions Capabilities Blank" => ("ctx:CapabilityPkg", On::First),
        "Contextual Capability" => ("ctx:Capability", On::Shown),
        "Contextual Mission" => ("ctx:Mission", On::Shown),
        "Contextual System Actors" => ("ctx:SystemComponent", On::Shown),
        "Operational Capabilities Blank" => ("oa:OperationalCapabilityPkg", On::First),
        "Contextual Operational Capability" => ("oa:OperationalCapability", On::Shown),
        "Capability Realization Blank" => ("la:CapabilityRealizationPkg", On::First),
        "Contextual Capability Realization Involvement" => ("la:CapabilityRealization", On::Shown),
        "Interfaces Diagram Blank" => ("la:LogicalComponent", On::First),
        _ => ("epbs:ConfigurationItemPkg", On::First),
    };
    Some((by_prefix, on, which))
}

/// What the diagrams need to know of the model: the kind and owner of each
/// element, the part of each component, who performs each function.
pub(super) struct Index {
    pub(super) kind: HashMap<String, String>,
    /// Ids in document order: the first package of a kind is the first met.
    pub(super) order: Vec<String>,
    pub(super) name: HashMap<String, String>,
    pub(super) owner: HashMap<String, String>,
    pub(super) performer: HashMap<String, String>,
    /// Exchanges and links with their two ends, in document order.
    pub(super) links: Vec<(String, String, String)>,
    /// The functions a chain involves, and the links a path involves, in order.
    pub(super) steps: HashMap<String, Vec<String>>,
}

/// `org.polarsys.capella.core.data.la:LogicalComponent` -> `la:LogicalComponent`.
fn short(kind: &str) -> String {
    match kind.rsplit_once(':') {
        Some((prefix, name)) => format!("{}:{name}", prefix.rsplit('.').next().unwrap_or(prefix)),
        None => kind.to_string(),
    }
}

pub(super) fn index(engineering: &El) -> Index {
    let mut index = Index { kind: HashMap::new(), order: Vec::new(), name: HashMap::new(), owner: HashMap::new(), performer: HashMap::new(), links: Vec::new(), steps: HashMap::new() };
    fn walk(el: &El, owner: Option<&str>, index: &mut Index) {
        let id = el.id().to_string();
        index.kind.insert(id.clone(), short(el.kind()));
        index.order.push(id.clone());
        if let Some(name) = el.get("name") {
            index.name.insert(id.clone(), name.to_string());
        }
        if let Some(owner) = owner {
            index.owner.insert(id.clone(), owner.to_string());
        }
        if el.kind().ends_with(":ComponentFunctionalAllocation") {
            if let (Some(function), Some(component)) = (el.get("targetElement"), el.get("sourceElement")) {
                index.performer.insert(function.trim_start_matches('#').to_string(), component.trim_start_matches('#').to_string());
            }
        }
        let reference = |key: &str| el.get(key).map(|r| r.trim_start_matches('#').to_string());
        let local = el.kind().rsplit(':').next().unwrap_or("");
        match local {
            "FunctionalExchange" | "ComponentExchange" | "CommunicationMean" => {
                if let (Some(a), Some(b)) = (reference("source"), reference("target")) {
                    index.links.push((id.clone(), a, b));
                }
            }
            "PhysicalLink" => {
                let ends: Vec<String> = el.get("linkEnds").unwrap_or("").split_whitespace().map(|e| e.trim_start_matches('#').to_string()).collect();
                if let [a, b] = ends.as_slice() {
                    index.links.push((id.clone(), a.clone(), b.clone()));
                }
            }
            "FunctionalChainInvolvementFunction" | "PhysicalPathInvolvement" => {
                if let (Some(owner), Some(involved)) = (owner, reference("involved")) {
                    index.steps.entry(owner.to_string()).or_default().push(involved);
                }
            }
            _ => {}
        }
        for child in el.children() {
            walk(child, Some(&id), index);
        }
    }
    walk(engineering, None, &mut index);
    index
}

pub(super) fn sirius(tag: &str, kind: &str, uid: &str) -> El {
    El::root(tag).attr("xmi:type", kind).attr("uid", uid)
}

fn href(kind: &str, file: &str, id: &str) -> El {
    El::root("").attr("xmi:type", &super::xml::xsi_type(kind)).attr("href", &format!("{}#{id}", file.replace(' ', "%20")))
}

pub(super) fn reference(tag: &str, kind: &str, file: &str, id: &str) -> El {
    let mut el = href(kind, file, id);
    el.rename(tag);
    el
}

/// The descriptor (for the DView of its viewpoint, by odesign) and the
/// representation of each diagram; diagrams Capella has no kind for, or
/// that show nothing exported, are noted.
pub(super) fn diagrams(context: &mut Context, index: &Index, diagrams: &[DiagramDecl], file: &str) -> Vec<(String, El, El)> {
    let table = table();
    let mut out = Vec::new();
    for (n, diagram) in diagrams.iter().enumerate() {
        let shown: Vec<String> = diagram.shows.iter().filter_map(|s| context.resolve(s).map(str::to_string)).collect();
        let shows_boxes = shown.iter().any(|id| index.kind.get(id).is_some_and(|k| k.ends_with("Component") || k == "oa:Entity"));
        let shows_realizations = shown.iter().any(|id| index.kind.get(id).is_some_and(|k| k == "la:CapabilityRealization"));
        let shows_interfaces = shown.iter().any(|id| index.kind.get(id).is_some_and(|k| k == "cs:Interface"));
        let Some((rep, target_kind, which)) = representation(diagram, shows_boxes, shows_realizations, shows_interfaces) else {
            context.note(format!("diagram '{}' left out: Capella has no diagram for the view '{}'", diagram.name, diagram.view));
            continue;
        };
        let Some(representation) = table.get(rep) else { continue };
        let of_kind = |id: &&String| index.kind.get(*id).map(String::as_str) == Some(target_kind);
        let target = match which {
            On::Shown => shown.iter().find(of_kind).or_else(|| index.order.iter().find(of_kind)).cloned(),
            On::Root => index.kind.keys().filter(of_kind).find(|id| index.name.get(*id).is_some_and(|n| n.starts_with("Root "))).cloned(),
            On::First => index.order.iter().find(of_kind).cloned(),
        };
        let Some(target) = target else {
            context.note(format!("diagram '{}' left out: the model has no {target_kind} to draw it on", diagram.name));
            continue;
        };
        let uid = format!("_{}", derived(&format!("diagram:{n}:{}", diagram.name)));
        // An element is drawn as itself, or through its part when this kind
        // of diagram draws parts (a component in an architecture diagram).
        let rows = &representation.boxes;
        let draws = |parent: Option<&str>, target: &str| rows.iter().any(|r| r.target == target && parent.is_none_or(|p| r.parent == p));
        let box_of = |id: &str| -> Option<super::diagram_drawer::Target> {
            let kind = index.kind.get(id)?.clone();
            let part = derived(&format!("{id}:part"));
            if index.kind.contains_key(&part) && !draws(None, &kind) && draws(None, "cs:Part") {
                return Some(("cs:Part".to_string(), part.clone(), vec![("cs:Part".to_string(), part), (kind, id.to_string())]));
            }
            Some((kind.clone(), id.to_string(), vec![(kind, id.to_string())]))
        };
        // The box holding an element: the first shown of the component
        // performing it and its owner that this kind of diagram nests it in.
        let target_of = |id: &str| box_of(id).map(|(kind, _, _)| kind);
        let holder = |id: &str| -> Option<String> {
            let child = target_of(id)?;
            [index.performer.get(id), index.owner.get(id)]
                .into_iter()
                .flatten()
                .find(|h| shown.contains(h) && target_of(h).is_some_and(|parent| rows.iter().any(|r| r.parent == parent && r.target == child)))
                .cloned()
        };
        let mut children: HashMap<String, Vec<String>> = HashMap::new();
        let mut tops: Vec<String> = Vec::new();
        for id in &shown {
            match holder(id) {
                Some(h) => children.entry(h).or_default().push(id.clone()),
                None => tops.push(id.clone()),
            }
        }
        // What cannot stand at the top of this kind of diagram goes into the
        // first shown box that can hold it (Capella draws an unallocated
        // function in a component), and chains and paths are edges.
        let top_level = |id: &str| target_of(id).is_some_and(|k| rows.iter().any(|r| r.parent == "diagram" && r.target == k));
        let is_edge = |id: &str| index.kind.get(id).is_some_and(|k| k.ends_with(":FunctionalChain") || k.ends_with(":PhysicalPath")) && !top_level(id);
        let mut placed: Vec<String> = Vec::new();
        for id in tops.iter().filter(|id| !top_level(id) && !is_edge(id)) {
            let Some(child) = target_of(id) else { continue };
            let host = tops.iter().find(|h| *h != id && top_level(h) && target_of(h).is_some_and(|p| rows.iter().any(|r| r.parent == p && r.target == child)));
            if let Some(host) = host {
                children.entry(host.clone()).or_default().push(id.clone());
                placed.push(id.clone());
            }
        }
        tops.retain(|id| !placed.contains(id) && !is_edge(id));
        let mut drawer = super::diagram_drawer::Drawer::new(representation, index, file, &uid, &children);
        let mut boxes: Vec<El> = tops.iter().filter_map(|id| drawer.draw(id, "diagram", &box_of)).collect();
        boxes.extend(drawer.edges(&shown));
        let dropped = drawer.dropped;
        if dropped > 0 {
            context.note(format!("diagram '{}': {dropped} shown elements have no box in a Capella {rep} diagram", diagram.name));
        }
        if boxes.is_empty() {
            context.note(format!("diagram '{}' left out: none of the elements it shows can be drawn in a Capella {rep} diagram", diagram.name));
            continue;
        }
        let target_kind_full = index.kind.get(&target).cloned().unwrap_or_default();
        let mut representation_el = El::root("diagram:DSemanticDiagram").attr("uid", &uid);
        for b in boxes {
            representation_el.push(b);
        }
        representation_el.push(El::root("description").attr("xmi:type", "description_1:DiagramDescription").attr("href", &representation.description));
        representation_el.push(reference("target", &target_kind_full, file, &target));
        representation_el.push(El::root("activatedLayers").attr("xmi:type", "description_1:Layer").attr("href", &format!("{}/@defaultLayer", representation.description)));
        let odesign = representation.description.split("/description/").nth(1).and_then(|d| d.split(".odesign").next()).unwrap_or("common").to_string();
        let descriptor = sirius("ownedRepresentationDescriptors", "viewpoint:DRepresentationDescriptor", &format!("{uid}-descriptor"))
            .attr("name", &diagram.name)
            .attr("repPath", &format!("#{uid}"))
            .with(El::root("description").attr("xmi:type", "description_1:DiagramDescription").attr("href", &representation.description))
            .with(reference("target", &target_kind_full, file, &target));
        out.push((odesign, descriptor, representation_el));
    }
    out
}
