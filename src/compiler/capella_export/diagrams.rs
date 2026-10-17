//! Diagrams, in the `.aird`: each `diagram` declaration becomes a Sirius
//! representation of the Capella diagram kind its name or view gives, with
//! one box per element it shows, nested as Capella draws them (a function
//! in the component that performs it, a component in its parent, a
//! behaviour in the node it is deployed on). The mappings and styles of the
//! boxes are those real Capella diagrams use (`sirius_boxes.json`, read
//! from Capella's own models); Capella lays the boxes out when it opens the
//! diagram.

use super::context::{derived, Context};
use super::xml::El;
use crate::compiler::ast::DiagramDecl;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct Box {
    parent: String,
    target: String,
    #[serde(rename = "type")]
    kind: String,
    mapping: String,
    style: Option<String>,
    style_description: String,
}

#[derive(Deserialize)]
struct Representation {
    description: String,
    boxes: Vec<Box>,
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
fn representation(diagram: &DiagramDecl, shows_boxes: bool) -> Option<(&'static str, &'static str, On)> {
    let prefix = diagram.name.trim_start().strip_prefix('[').and_then(|r| r.split(']').next()).unwrap_or("");
    let by_prefix = match prefix {
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
        _ => ("epbs:ConfigurationItemPkg", On::First),
    };
    Some((by_prefix, on, which))
}

/// What the diagrams need to know of the model: the kind and owner of each
/// element, the part of each component, who performs each function.
pub(super) struct Index {
    kind: HashMap<String, String>,
    /// Ids in document order: the first package of a kind is the first met.
    order: Vec<String>,
    name: HashMap<String, String>,
    owner: HashMap<String, String>,
    performer: HashMap<String, String>,
}

/// `org.polarsys.capella.core.data.la:LogicalComponent` -> `la:LogicalComponent`.
fn short(kind: &str) -> String {
    match kind.rsplit_once(':') {
        Some((prefix, name)) => format!("{}:{name}", prefix.rsplit('.').next().unwrap_or(prefix)),
        None => kind.to_string(),
    }
}

pub(super) fn index(engineering: &El) -> Index {
    let mut index = Index { kind: HashMap::new(), order: Vec::new(), name: HashMap::new(), owner: HashMap::new(), performer: HashMap::new() };
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
        for child in el.children() {
            walk(child, Some(&id), index);
        }
    }
    walk(engineering, None, &mut index);
    index
}

fn sirius(tag: &str, kind: &str, uid: &str) -> El {
    El::root(tag).attr("xmi:type", kind).attr("uid", uid)
}

fn href(kind: &str, file: &str, id: &str) -> El {
    El::root("").attr("xmi:type", &super::xml::xsi_type(kind)).attr("href", &format!("{}#{id}", file.replace(' ', "%20")))
}

fn reference(tag: &str, kind: &str, file: &str, id: &str) -> El {
    let mut el = href(kind, file, id);
    el.rename(tag);
    el
}

/// A box for `id`, drawn as `row` says, holding `children`.
fn element(row: &Box, file: &str, uid: &str, name: &str, target: (&str, &str), semantic: &[(&str, &str)], children: Vec<El>) -> El {
    let mut el = sirius("ownedDiagramElements", &row.kind, uid).attr("name", name);
    el.push(reference("target", target.0, file, target.1));
    for (kind, id) in semantic {
        el.push(reference("semanticElements", kind, file, id));
    }
    let style = row.style.as_deref().unwrap_or("diagram:Square");
    let description_kind = match style {
        "diagram:FlatContainerStyle" => "style:FlatContainerStyleDescription",
        "diagram:WorkspaceImage" => "style:WorkspaceImageDescription",
        "diagram:Dot" => "style:DotDescription",
        "diagram:Ellipse" => "style:EllipseNodeDescription",
        _ => "style:SquareDescription",
    };
    let style_href = if row.style_description.is_empty() { format!("{}/@style", row.mapping) } else { row.style_description.clone() };
    el.push(sirius("ownedStyle", style, &format!("{uid}-style")).with(El::root("description").attr("xmi:type", description_kind).attr("href", &style_href)));
    // The mapping's own kind is its last step: `.../@containerMappings[..]/@subNodeMappings[..]` is a node mapping.
    let last = row.mapping.rsplit("/@").next().unwrap_or("");
    let mapping_kind = if last.starts_with("containerMappings") || last.starts_with("subContainerMappings") { "description_1:ContainerMapping" } else { "description_1:NodeMapping" };
    el.push(El::root("actualMapping").attr("xmi:type", mapping_kind).attr("href", &row.mapping));
    for child in children {
        el.push(child);
    }
    el
}

/// The descriptor (for the DView of its viewpoint, by odesign) and the
/// representation of each diagram; diagrams Capella has no kind for, or
/// that show nothing exported, are noted.
pub(super) fn diagrams(context: &mut Context, index: &Index, diagrams: &[DiagramDecl], file: &str) -> Vec<(String, El, El)> {
    let names = &index.name;
    let table = table();
    let mut out = Vec::new();
    for (n, diagram) in diagrams.iter().enumerate() {
        let shown: Vec<String> = diagram.shows.iter().filter_map(|s| context.resolve(s).map(str::to_string)).collect();
        let shows_boxes = shown.iter().any(|id| index.kind.get(id).is_some_and(|k| k.ends_with("Component") || k == "oa:Entity"));
        let Some((rep, target_kind, which)) = representation(diagram, shows_boxes) else {
            context.note(format!("diagram '{}' left out: Capella has no diagram for the view '{}'", diagram.name, diagram.view));
            continue;
        };
        let Some(representation) = table.get(rep) else { continue };
        let of_kind = |id: &&String| index.kind.get(*id).map(String::as_str) == Some(target_kind);
        let target = match which {
            On::Shown => shown.iter().find(of_kind).cloned(),
            On::Root => index.kind.keys().filter(of_kind).find(|id| index.name.get(*id).is_some_and(|n| n.starts_with("Root "))).cloned(),
            On::First => index.order.iter().find(of_kind).cloned(),
        };
        let Some(target) = target else {
            context.note(format!("diagram '{}' left out: the model has no {target_kind} to draw it on", diagram.name));
            continue;
        };
        let uid = format!("_{}", derived(&format!("diagram:{n}:{}", diagram.name)));
        // Each shown element in the box of what holds it, when that is shown too.
        let box_of = |id: &str| -> Option<(String, String, Vec<(String, String)>)> {
            let kind = index.kind.get(id)?.clone();
            let part = derived(&format!("{id}:part"));
            if index.kind.contains_key(&part) && !matches!(kind.as_str(), "oa:Entity") {
                return Some(("cs:Part".to_string(), part.clone(), vec![("cs:Part".to_string(), part), (kind, id.to_string())]));
            }
            Some((kind.clone(), id.to_string(), vec![(kind, id.to_string())]))
        };
        // The box holding an element: the first shown of the component
        // performing it and its owner that this kind of diagram nests it in.
        let rows = &representation.boxes;
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
        fn build(id: &str, parent: &str, rows: &[Box], uid: &str, file: &str, names: &HashMap<String, String>, children: &HashMap<String, Vec<String>>, box_of: &dyn Fn(&str) -> Option<(String, String, Vec<(String, String)>)>, dropped: &mut usize) -> Option<El> {
            let Some((target_kind, target, semantic)) = box_of(id) else { *dropped += 1; return None };
            let Some(row) = rows.iter().find(|r| r.parent == parent && r.target == target_kind) else { *dropped += 1; return None };
            let inner: Vec<El> = children.get(id).into_iter().flatten().filter_map(|c| build(c, &target_kind, rows, uid, file, names, children, box_of, dropped)).collect();
            let semantic: Vec<(&str, &str)> = semantic.iter().map(|(k, i)| (k.as_str(), i.as_str())).collect();
            let name = names.get(id).map(String::as_str).unwrap_or("");
            Some(element(row, file, &format!("{uid}-{}", derived(id)), name, (&target_kind, &target), &semantic, inner))
        }
        let mut dropped = 0;

        let boxes: Vec<El> = tops.iter().filter_map(|id| build(id, "diagram", &representation.boxes, &uid, file, names, &children, &box_of, &mut dropped)).collect();
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
