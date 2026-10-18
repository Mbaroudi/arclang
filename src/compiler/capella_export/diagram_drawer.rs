//! Drawing one diagram: boxes nested as Capella nests them, the ports of
//! each box on its border, and the edges between what is drawn: exchanges
//! and links between their ports (or their elements), a chain through each
//! function it involves (input port to output port), a physical path
//! through each node it crosses (port to port).

use super::context::derived;
use super::diagrams::{reference, sirius, Box, Edge, Index, Representation};
use super::xml::El;
use std::collections::HashMap;

/// What a box of an element is drawn on: target kind, target id, semantic elements.
pub(super) type Target = (String, String, Vec<(String, String)>);

/// How a diagram element looks: its style and the mapping that draws it.
struct Look<'b> {
    style: Option<&'b str>,
    style_description: &'b str,
    mapping: &'b str,
}

impl<'b> Look<'b> {
    fn of_box(row: &'b Box) -> Self {
        Look { style: row.style.as_deref(), style_description: &row.style_description, mapping: &row.mapping }
    }
    fn of_edge(row: &'b Edge) -> Self {
        Look { style: row.style.as_deref(), style_description: &row.style_description, mapping: &row.mapping }
    }
}

pub(super) struct Drawer<'a> {
    representation: &'a Representation,
    index: &'a Index,
    file: &'a str,
    uid: &'a str,
    children: &'a HashMap<String, Vec<String>>,
    /// The diagram element drawn for each model element (boxes and ports).
    drawn: HashMap<String, String>,
    pub(super) dropped: usize,
}

fn style_kind(style: &str) -> &'static str {
    match style {
        "diagram:FlatContainerStyle" => "style:FlatContainerStyleDescription",
        "diagram:WorkspaceImage" => "style:WorkspaceImageDescription",
        "diagram:Dot" => "style:DotDescription",
        "diagram:Ellipse" => "style:EllipseNodeDescription",
        "diagram:EdgeStyle" => "style:EdgeStyleDescription",
        _ => "style:SquareDescription",
    }
}

/// The mapping's own kind is its last step: `.../@containerMappings[..]/@subNodeMappings[..]` is a node mapping.
fn mapping_kind(mapping: &str) -> &'static str {
    let last = mapping.rsplit("/@").next().unwrap_or("");
    if last.starts_with("containerMappings") || last.starts_with("subContainerMappings") {
        "description_1:ContainerMapping"
    } else if last.starts_with("edgeMappings") {
        "description_1:EdgeMapping"
    } else {
        "description_1:NodeMapping"
    }
}

impl<'a> Drawer<'a> {
    pub(super) fn new(representation: &'a Representation, index: &'a Index, file: &'a str, uid: &'a str, children: &'a HashMap<String, Vec<String>>) -> Self {
        Drawer { representation, index, file, uid, children, drawn: HashMap::new(), dropped: 0 }
    }

    fn element_uid(&self, id: &str) -> String {
        format!("{}-{}", self.uid, derived(id))
    }

    /// The parts every diagram element has: its target, semantic elements, style and mapping.
    fn body(&self, mut el: El, target: (&str, &str), semantic: &[(String, String)], look: Look<'_>, uid: &str) -> El {
        let Look { style, style_description, mapping } = look;
        el.push(reference("target", target.0, self.file, target.1));
        for (kind, id) in semantic {
            el.push(reference("semanticElements", kind, self.file, id));
        }
        let style = style.unwrap_or("diagram:Square");
        let style_href = if style_description.is_empty() { format!("{mapping}/@style") } else { style_description.to_string() };
        el.push(sirius("ownedStyle", style, &format!("{uid}-style")).with(El::root("description").attr("xmi:type", style_kind(style)).attr("href", &style_href)));
        el.push(El::root("actualMapping").attr("xmi:type", mapping_kind(mapping)).attr("href", mapping));
        el
    }

    fn name(&self, id: &str) -> String {
        self.index.name.get(id).cloned().unwrap_or_default()
    }

    /// The box of `id` under a parent drawn on `parent` kind, with its ports and nested boxes.
    pub(super) fn draw(&mut self, id: &str, parent: &str, box_of: &dyn Fn(&str) -> Option<Target>) -> Option<El> {
        let Some((target_kind, target, semantic)) = box_of(id) else {
            self.dropped += 1;
            return None;
        };
        let Some(row) = self.representation.boxes.iter().find(|r| r.parent == parent && r.target == target_kind) else {
            self.dropped += 1;
            return None;
        };
        let uid = self.element_uid(id);
        self.drawn.insert(id.to_string(), uid.clone());
        let el = sirius("ownedDiagramElements", &row.kind, &uid).attr("name", &self.name(id));
        let mut el = self.body(el, (&target_kind, &target), &semantic, Look::of_box(row), &uid);
        for port in self.ports(id) {
            if let Some(node) = self.port(&port, &target_kind) {
                el.push(node);
            }
        }
        for child in self.children.get(id).cloned().unwrap_or_default() {
            if let Some(inner) = self.draw(&child, &target_kind, box_of) {
                el.push(inner);
            }
        }
        Some(el)
    }

    /// The ports an element owns, in document order.
    fn ports(&self, id: &str) -> Vec<String> {
        self.index
            .order
            .iter()
            .filter(|p| self.index.owner.get(*p).map(String::as_str) == Some(id))
            .filter(|p| self.index.kind.get(*p).is_some_and(|k| k.ends_with("Port")))
            .cloned()
            .collect()
    }

    fn port(&mut self, port: &str, parent: &str) -> Option<El> {
        let kind = self.index.kind.get(port)?.clone();
        let row: &Box = self.representation.bordered.iter().find(|r| r.parent == parent && r.target == kind)?;
        let uid = self.element_uid(port);
        self.drawn.insert(port.to_string(), uid.clone());
        let el = sirius("ownedBorderedNodes", &row.kind, &uid).attr("name", &self.name(port));
        Some(self.body(el, (&kind, port), &[(kind.clone(), port.to_string())], Look::of_box(row), &uid))
    }

    fn edge(&self, row: &Edge, target: (&str, &str), source_uid: &str, target_uid: &str, n: usize) -> El {
        let uid = format!("{}-{}", self.element_uid(target.1), n);
        let el = sirius("ownedDiagramElements", "diagram:DEdge", &uid).attr("sourceNode", source_uid).attr("targetNode", target_uid);
        self.body(el, target, &[(target.0.to_string(), target.1.to_string())], Look::of_edge(row), &uid)
    }

    fn edge_row(&self, target: &str, from: &str, to: &str) -> Option<&'a Edge> {
        let kind = |id: &str| self.index.kind.get(id).cloned().unwrap_or_default();
        let (source_kind, target_kind) = (self.box_kind(from).unwrap_or_else(|| kind(from)), self.box_kind(to).unwrap_or_else(|| kind(to)));
        self.representation.edges.iter().find(|e| e.target == target && e.source_kind == source_kind && e.target_kind == target_kind)
    }

    /// The kind a drawn element's box targets: a component is drawn through its part.
    fn box_kind(&self, id: &str) -> Option<String> {
        let kind = self.index.kind.get(id)?;
        if kind.ends_with("Port") {
            return Some(kind.clone());
        }
        let part = derived(&format!("{id}:part"));
        let as_part = self.index.kind.contains_key(&part) && !self.representation.boxes.iter().any(|b| b.target == *kind) && kind != "oa:Entity";
        Some(if as_part { "cs:Part".to_string() } else { kind.clone() })
    }

    /// Exchanges and links between drawn ends, then the chains and paths shown.
    pub(super) fn edges(&self, shown: &[String]) -> Vec<El> {
        let mut out = Vec::new();
        for (id, from, to) in &self.index.links {
            let (Some(a), Some(b)) = (self.drawn.get(from), self.drawn.get(to)) else { continue };
            let Some(kind) = self.index.kind.get(id) else { continue };
            if let Some(row) = self.edge_row(kind, from, to) {
                out.push(self.edge(row, (kind, id), a, b, 0));
            }
        }
        for id in shown {
            let Some(kind) = self.index.kind.get(id).cloned() else { continue };
            let Some(steps) = self.index.steps.get(id) else { continue };
            let mut n = 0;
            if kind.ends_with(":FunctionalChain") {
                // Through each involved function: from its input port to its output port.
                for function in steps {
                    let ports = self.ports(function);
                    let input = ports.iter().find(|p| self.index.kind.get(*p).is_some_and(|k| k.ends_with("InputPort")) && self.drawn.contains_key(*p));
                    let output = ports.iter().find(|p| self.index.kind.get(*p).is_some_and(|k| k.ends_with("OutputPort")) && self.drawn.contains_key(*p));
                    if let (Some(i), Some(o)) = (input, output) {
                        if let Some(row) = self.edge_row(&kind, i, o) {
                            out.push(self.edge(row, (&kind, id), &self.drawn[i], &self.drawn[o], n));
                            n += 1;
                        }
                    }
                }
            } else if kind.ends_with(":PhysicalPath") {
                // Through each node between two of its links: from one link's port to the next's.
                let links: Vec<&(String, String, String)> = steps.iter().filter_map(|s| self.index.links.iter().find(|(l, _, _)| l == s)).collect();
                for pair in links.windows(2) {
                    let (first, second) = (pair[0], pair[1]);
                    let owner = |p: &str| self.index.owner.get(p).cloned();
                    let shared = [&first.1, &first.2].into_iter().find_map(|p| [&second.1, &second.2].into_iter().find(|q| owner(q) == owner(p) && owner(p).is_some()).map(|q| (p.clone(), q.clone())));
                    if let Some((p, q)) = shared {
                        if let (Some(a), Some(b), Some(row)) = (self.drawn.get(&p), self.drawn.get(&q), self.edge_row(&kind, &p, &q)) {
                            out.push(self.edge(row, (&kind, id), a, b, n));
                            n += 1;
                        }
                    }
                }
            }
        }
        out
    }
}
