//! The views of SysML v2, drawn from the SysML v2 text of the model (the
//! export `arclang export -f sys-ml` writes, read back as declarations):
//!
//! - a **General View** (`gv:<package>`) of each package: its part and
//!   port definitions, its data structures (item definitions with fields),
//!   its parts (not the lifelines of scenarios nor the security analysis's
//!   threats), a part linked to the parts it is made of, a usage to its
//!   definition, a definition to the one it specializes;
//! - an **Interconnection View** (`iv:<package>`) of each package whose
//!   parts are connected: the parts nested as they are owned, their ports,
//!   the connections between ports, and the flows between the actions two
//!   parts perform, drawn between those parts;
//! - an **Action Flow View** (`afv:<package>`) of each package with
//!   actions: the actions, those a part performs inside it as in a lane,
//!   their items in and out as ports, the flows between them (and the parts
//!   a flow comes from or goes to), and the steps of an action in order
//!   (`then`).
//!
//! What a SysON user calls a view is here the same model in the vocabulary
//! of SysML v2; the Arcadia views stay as they are.

use super::{Diagram, Edge, EdgeKind, Node, NodeKind, Port, PortDirection, ViewKind};
use crate::compiler::ast::Model;
use crate::compiler::identity::element_uuid;
use crate::compiler::sysml_syntax::{self, Declaration, Scope};
use std::collections::{BTreeMap, HashMap, HashSet};

const CONNECTIONS: [&str; 2] = ["ConnectionUsage", "InterfaceUsage"];
const FLOWS: [&str; 2] = ["FlowUsage", "FlowConnectionUsage"];
const CONTROL_NODES: [&str; 4] = ["ForkNode", "JoinNode", "DecisionNode", "MergeNode"];

/// The keyword SysML v2 writes a declaration with: `part def`, `port`...
fn keyword(metaclass: &str) -> String {
    let (base, def) = match metaclass.strip_suffix("Definition") {
        Some(base) => (base, true),
        None => (metaclass.strip_suffix("Usage").unwrap_or(metaclass), false),
    };
    let mut word = String::new();
    for (i, c) in base.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            word.push(' ');
        }
        word.extend(c.to_lowercase());
    }
    if def {
        word.push_str(" def");
    }
    word
}

struct Model2<'d> {
    declarations: &'d [Declaration],
    scope: Scope<'d>,
    children: HashMap<usize, Vec<usize>>,
    /// Declarations inside a library package: the profile, not the model.
    library: HashSet<usize>,
}

impl<'d> Model2<'d> {
    fn new(declarations: &'d [Declaration]) -> Self {
        let mut children: HashMap<usize, Vec<usize>> = HashMap::new();
        for (index, declaration) in declarations.iter().enumerate() {
            if let Some(owner) = declaration.owner {
                children.entry(owner).or_default().push(index);
            }
        }
        let mut library = HashSet::new();
        for (index, declaration) in declarations.iter().enumerate() {
            let inside = declaration.metaclass == "LibraryPackage" || declaration.owner.is_some_and(|o| library.contains(&o));
            if inside {
                library.insert(index);
            }
        }
        Model2 { declarations, scope: Scope::new(declarations), children, library }
    }

    fn children(&self, index: usize) -> &[usize] {
        self.children.get(&index).map(Vec::as_slice).unwrap_or_default()
    }

    fn name(&self, index: usize) -> String {
        let d = &self.declarations[index];
        d.name.clone().or_else(|| d.short_name.clone()).unwrap_or_else(|| keyword(d.metaclass))
    }

    /// The id a node goes by: the ArcLang id the export writes as the short
    /// name (`<'PN-BATT'>`); a feature without one is named in its owner
    /// (`FC-ALT.step1`); else its qualified name.
    fn id(&self, index: usize) -> String {
        let d = &self.declarations[index];
        if let Some(short) = &d.short_name {
            return short.clone();
        }
        match (d.owner, &d.name) {
            (Some(owner), Some(name)) if !matches!(self.declarations[owner].metaclass, "Package" | "Namespace") => format!("{}.{name}", self.id(owner)),
            _ => self.scope.qualified_name(index).unwrap_or_else(|| format!("decl{index}")),
        }
    }

    /// `mass : MassValue = 4.5 [kg]`: the attributes of a declaration.
    fn attributes(&self, index: usize) -> Vec<String> {
        self.children(index)
            .iter()
            .map(|&c| &self.declarations[c])
            .filter(|d| d.metaclass == "AttributeUsage")
            .map(|d| {
                // The type by its name (`Distance`), the one the reader knows, not its id (`DT-DIST`).
                let typed = d
                    .typed_by
                    .first()
                    .map(|t| match self.scope.resolve(t, Some(index)).filter(|&r| !self.library.contains(&r)) {
                        Some(resolved) => format!(" : {}", self.name(resolved)),
                        None => format!(" : {}", t.rsplit("::").next().unwrap_or(t).trim_matches('\'')),
                    })
                    .unwrap_or_default();
                let value = d.expression.as_ref().map(|e| format!(" = {e}")).unwrap_or_default();
                format!("{}{typed}{value}", d.name.clone().unwrap_or_default())
            })
            .collect()
    }

    fn node(&self, index: usize, kind: NodeKind, children: Vec<Node>, ports: Vec<Port>, compartment: Vec<String>) -> Node {
        let d = &self.declarations[index];
        let id = self.id(index);
        let typed = d.typed_by.first().map(|t| t.rsplit("::").next().unwrap_or(t).trim_matches('\'').to_string());
        let mut properties = BTreeMap::from([("stereotype".to_string(), keyword(d.metaclass)), ("metaclass".to_string(), d.metaclass.to_string())]);
        if let Some(typed) = &typed {
            properties.insert("typed_by".to_string(), typed.clone());
        }
        Node {
            uuid: element_uuid("element", d.short_name.as_deref().unwrap_or(&id)),
            id,
            name: match &typed {
                Some(t) if !d.metaclass.ends_with("Definition") => format!("{} : {t}", self.name(index)),
                _ => self.name(index),
            },
            kind,
            ports,
            children,
            realizes: Vec::new(),
            compartment,
            properties,
        }
    }

    fn ports(&self, index: usize, metaclasses: &[&str], owner_id: &str) -> Vec<Port> {
        self.children(index)
            .iter()
            .map(|&c| (c, &self.declarations[c]))
            .filter(|(_, d)| metaclasses.contains(&d.metaclass) && d.name.is_some())
            .map(|(c, d)| Port {
                id: format!("{owner_id}.{}", self.name(c)),
                name: self.name(c),
                direction: match d.direction {
                    Some("in") => PortDirection::In,
                    Some("out") => PortDirection::Out,
                    Some("inout") => PortDirection::InOut,
                    _ => PortDirection::Undirected,
                },
                interface: d.typed_by.first().map(|t| t.rsplit("::").next().unwrap_or(t).trim_matches('\'').to_string()),
                protocol: None,
                synthesized: false,
            })
            .collect()
    }
}

fn edge(id: String, kind: EdgeKind, source: String, source_port: Option<String>, target: String, target_port: Option<String>, label: String) -> Edge {
    Edge { id, kind, source, source_port, target, target_port, label, exchange_item: None, properties: BTreeMap::new() }
}

fn diagram(id: String, kind: ViewKind, title: String, nodes: Vec<Node>, edges: Vec<Edge>) -> Diagram {
    Diagram { id, kind, title, nodes, edges, chains: Vec::new(), fragments: Vec::new(), invariants: Vec::new() }
}

pub(super) fn build(model: &Model) -> Vec<(Option<Diagram>, Vec<String>)> {
    let text = crate::compiler::sysml_export::export(model);
    let declarations = match sysml_syntax::read(&text) {
        Ok(declarations) => declarations,
        // The export is the compiler's own text: a failure to read it back is the compiler's.
        Err(error) => return vec![(None, vec![format!("[gv] the SysML v2 views are not drawn: the export does not read back ({error})")])],
    };
    let m = Model2::new(&declarations);
    let mut out = Vec::new();
    for view in general(&m) {
        out.push((Some(view), Vec::new()));
    }
    for (index, declaration) in declarations.iter().enumerate() {
        if !matches!(declaration.metaclass, "Package") || m.library.contains(&index) {
            continue;
        }
        if let Some(view) = interconnection(&m, index) {
            out.push((Some(view), Vec::new()));
        }
        if let Some(view) = action_flow(&m, index) {
            out.push((Some(view), Vec::new()));
        }
    }
    out
}

/// What the General View draws of a declaration: the structure of the
/// model, not its behaviour nor its bookkeeping.
/// A declaration carrying one of these metadata (`#Threat part`).
fn tagged(m: &Model2<'_>, index: usize, names: &[&str]) -> bool {
    m.children(index).iter().any(|&c| m.declarations[c].metaclass == "MetadataUsage" && m.declarations[c].typed_by.iter().any(|t| names.contains(&t.rsplit("::").next().unwrap_or(t))))
}

/// Parts that are not pieces of the system: the security analysis's
/// threats and assets, the physical paths (routes over links).
const NOT_PIECES: [&str; 4] = ["Threat", "Asset", "PrimaryAsset", "Path"];

fn in_general_view(m: &Model2<'_>, index: usize) -> bool {
    let d = &m.declarations[index];
    let owner_is = |kinds: &[&str]| d.owner.is_some_and(|o| kinds.contains(&m.declarations[o].metaclass));
    match d.metaclass {
        // A part owned by a package or a part: not a lifeline of a scenario,
        // nor a part of an action, nor a threat, an asset or a path.
        "PartUsage" => owner_is(&["Package", "PartUsage"]) && !tagged(m, index, &NOT_PIECES),
        "PartDefinition" | "PortDefinition" => true,
        // The data model: classes, their enumerations and data types; an
        // exchange item is the flows' business, a bare item definition a
        // name for a type the model did not declare.
        "ItemDefinition" => tagged(m, index, &["Class"]),
        "EnumerationDefinition" | "AttributeDefinition" => true,
        _ => false,
    }
}

/// What a box of the General View lists: its attributes, its ports with
/// what they carry, the actions it performs.
fn features(m: &Model2<'_>, index: usize) -> Vec<String> {
    let mut out = m.attributes(index);
    let short = |t: &String| t.rsplit("::").next().unwrap_or(t).trim_matches('\'').to_string();
    for &c in m.children(index) {
        let d = &m.declarations[c];
        match d.metaclass {
            "PortUsage" => {
                let direction = d.direction.map(|x| format!("{x} ")).unwrap_or_default();
                let typed = d.typed_by.first().map(|t| format!(" : {}", short(t))).unwrap_or_default();
                out.push(format!("{direction}port {}{typed}", m.name(c)));
            }
            "PerformActionUsage" if d.references.is_empty() => out.push(format!("perform {}", m.name(c))),
            "PerformActionUsage" => out.extend(d.references.iter().map(|r| format!("perform {}", short(r)))),
            "EnumerationUsage" => out.push(m.name(c)),
            _ => {}
        }
    }
    out.dedup();
    out
}

/// A General View of each package (`gv:<package>`): the definitions and
/// the parts it declares, with how they are made, typed and specialized,
/// and the definitions of other packages that type them. One view per
/// package, as the Interconnection and Action Flow views: a whole model
/// at once is not read.
fn general(m: &Model2<'_>) -> Vec<Diagram> {
    let package_of = |mut i: usize| -> Option<usize> {
        while let Some(o) = m.declarations[i].owner {
            if m.declarations[o].metaclass == "Package" {
                return Some(o);
            }
            i = o;
        }
        None
    };
    let shown: Vec<usize> = (0..m.declarations.len()).filter(|i| !m.library.contains(i)).filter(|&i| in_general_view(m, i)).collect();
    let mut packages: Vec<usize> = shown.iter().filter_map(|&i| package_of(i)).collect();
    packages.dedup();
    let mut seen_packages = HashSet::new();
    // The security analysis's scales are its rating vocabulary, not the system's structure.
    packages.retain(|p| seen_packages.insert(*p) && !tagged(m, *p, &["SecurityScales"]));
    let mut out = Vec::new();
    for package in packages {
        let own: Vec<usize> = shown.iter().copied().filter(|&i| package_of(i) == Some(package)).collect();
        // The definitions elsewhere that type or generalize what the package declares.
        let mut members = own.clone();
        for &i in &own {
            let d = &m.declarations[i];
            for written in d.typed_by.iter().chain(&d.specializes) {
                if let Some(t) = m.scope.resolve(written, d.owner).filter(|t| shown.contains(t) && !members.contains(t)) {
                    members.push(t);
                }
            }
        }
        let set: HashSet<usize> = members.iter().copied().collect();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut ids: HashSet<String> = HashSet::new();
        for &i in &members {
            let d = &m.declarations[i];
            let kind = if d.metaclass.ends_with("Definition") { NodeKind::SysmlDefinition } else { NodeKind::SysmlPart };
            let node = m.node(i, kind, Vec::new(), Vec::new(), features(m, i));
            if !ids.insert(node.id.clone()) {
                continue;
            }
            nodes.push(node);
            let id = m.id(i);
            // A class to the classes, enumerations and data types its fields are typed by.
            for &field in m.children(i).iter().filter(|&&c| d.metaclass == "ItemDefinition" && m.declarations[c].metaclass == "AttributeUsage") {
                for typed in &m.declarations[field].typed_by {
                    if let Some(definition) = m.scope.resolve(typed, Some(i)).filter(|t| set.contains(t) && *t != i) {
                        let target = m.id(definition);
                        edges.push(edge(format!("fld:{id} {target}"), EdgeKind::Typing, id.clone(), None, target, None, m.name(field)));
                    }
                }
            }
            // A part to the definitions its ports are typed by.
            for &port in m.children(i).iter().filter(|&&c| m.declarations[c].metaclass == "PortUsage") {
                for typed in &m.declarations[port].typed_by {
                    if let Some(definition) = m.scope.resolve(typed, Some(i)).filter(|t| set.contains(t)) {
                        let target = m.id(definition);
                        edges.push(edge(format!("typ:{id} {target}"), EdgeKind::Typing, id.clone(), None, target, None, String::new()));
                    }
                }
            }
            // A part is made of the parts it owns.
            if let Some(owner) = d.owner.filter(|o| set.contains(o) && m.declarations[*o].metaclass == "PartUsage") {
                let owner_id = m.id(owner);
                edges.push(edge(format!("cmp:{owner_id} {id}"), EdgeKind::Composition, owner_id, None, id.clone(), None, String::new()));
            }
            for typed in &d.typed_by {
                if let Some(definition) = m.scope.resolve(typed, d.owner).filter(|t| set.contains(t)) {
                    let target = m.id(definition);
                    edges.push(edge(format!("typ:{id} {target}"), EdgeKind::Typing, id.clone(), None, target, None, String::new()));
                }
            }
            for general in &d.specializes {
                if let Some(definition) = m.scope.resolve(general, d.owner).filter(|t| set.contains(t)) {
                    let target = m.id(definition);
                    edges.push(edge(format!("gen:{id} {target}"), EdgeKind::Generalization, id.clone(), None, target, None, String::new()));
                }
            }
        }
        let mut seen = HashSet::new();
        edges.retain(|e| ids.contains(&e.source) && ids.contains(&e.target) && seen.insert(e.id.clone()));
        let name = m.name(package);
        out.push(diagram(format!("{}:{name}", ViewKind::Gv.id()), ViewKind::Gv, format!("General View: {name}"), nodes, edges));
    }
    out
}

/// A part with the parts it owns nested in it, and its ports.
fn part_tree(m: &Model2<'_>, index: usize) -> Node {
    let id = m.id(index);
    let children = m.children(index).iter().copied().filter(|&c| m.declarations[c].metaclass == "PartUsage").map(|c| part_tree(m, c)).collect();
    let ports = m.ports(index, &["PortUsage", "InterfaceUsage"], &id);
    m.node(index, NodeKind::SysmlPart, children, ports, Vec::new())
}

/// Where a connection end lands: the node, and the port when it is one.
fn end_of(m: &Model2<'_>, written: &str, from: Option<usize>, ports: &HashSet<String>) -> Option<(String, Option<String>)> {
    let chain = m.scope.chain(written, from)?;
    let last = *chain.last()?;
    let d = &m.declarations[last];
    if matches!(d.metaclass, "PortUsage" | "ItemUsage") {
        let owner = *chain.get(chain.len().wrapping_sub(2))?;
        let owner_id = m.id(owner);
        let port = format!("{owner_id}.{}", m.name(last));
        return ports.contains(&port).then_some((owner_id, Some(port)));
    }
    Some((m.id(last), None))
}

fn port_ids(nodes: &[Node], out: &mut HashSet<String>) {
    for node in nodes {
        out.extend(node.ports.iter().map(|p| p.id.clone()));
        port_ids(&node.children, out);
    }
}

fn node_ids(nodes: &[Node], out: &mut HashSet<String>) {
    for node in nodes {
        out.insert(node.id.clone());
        node_ids(&node.children, out);
    }
}

/// The part a flow's end lands on: the part that holds the action (or
/// the action's item) it names, else the part of the package that
/// `perform`s that action of the level.
fn performer_of(m: &Model2<'_>, written: &str, package: usize) -> Option<usize> {
    let chain = m.scope.chain(written, Some(package))?;
    if let Some(&part) = chain.iter().rev().find(|&&c| m.declarations[c].metaclass == "PartUsage") {
        return Some(part);
    }
    let action = *chain.iter().rev().find(|&&c| matches!(m.declarations[c].metaclass, "ActionUsage" | "PerformActionUsage"))?;
    m.children(package).iter().copied().filter(|&p| m.declarations[p].metaclass == "PartUsage").find(|&p| {
        m.children(p).iter().any(|&c| {
            let d = &m.declarations[c];
            d.metaclass == "PerformActionUsage" && d.references.iter().any(|r| m.scope.chain(r, Some(p)).and_then(|c| c.last().copied()) == Some(action))
        })
    })
}

/// The parts of a package connected through their ports, and the flows
/// between the actions they perform, drawn between those parts.
fn interconnection(m: &Model2<'_>, package: usize) -> Option<Diagram> {
    let connections: Vec<usize> = m.children(package).iter().copied().filter(|&c| CONNECTIONS.contains(&m.declarations[c].metaclass) && m.declarations[c].ends.is_some()).collect();
    let flows: Vec<usize> = m.children(package).iter().copied().filter(|&c| FLOWS.contains(&m.declarations[c].metaclass) && m.declarations[c].ends.is_some()).collect();
    let parts: Vec<usize> = m.children(package).iter().copied().filter(|&c| m.declarations[c].metaclass == "PartUsage" && !tagged(m, c, &NOT_PIECES)).collect();
    if (connections.is_empty() && flows.is_empty()) || parts.is_empty() {
        return None;
    }
    let nodes: Vec<Node> = parts.iter().map(|&p| part_tree(m, p)).collect();
    let (mut ports, mut drawn) = (HashSet::new(), HashSet::new());
    port_ids(&nodes, &mut ports);
    node_ids(&nodes, &mut drawn);
    let mut edges = Vec::new();
    for c in connections {
        let d = &m.declarations[c];
        let (from, to) = d.ends.as_ref().expect("filtered on ends");
        // An end on a port of a part lands on that port; on an action (or its
        // item), on the part that performs it.
        let land = |written: &str| {
            end_of(m, written, Some(package), &ports).filter(|(node, _)| drawn.contains(node)).or_else(|| performer_of(m, written, package).map(|p| (m.id(p), None)))
        };
        let (Some((source, source_port)), Some((target, target_port))) = (land(from), land(to)) else { continue };
        if source == target || !drawn.contains(&source) || !drawn.contains(&target) {
            continue;
        }
        edges.push(edge(format!("cnx:{}", m.id(c)), EdgeKind::Connection, source, source_port, target, target_port, d.name.clone().unwrap_or_default()));
    }
    for f in flows {
        let d = &m.declarations[f];
        let (from, to) = d.ends.as_ref().expect("filtered on ends");
        let (Some(a), Some(b)) = (performer_of(m, from, package), performer_of(m, to, package)) else { continue };
        let (source, target) = (m.id(a), m.id(b));
        // A flow inside one part is its own business: the view shows what goes between parts.
        if source == target || !drawn.contains(&source) || !drawn.contains(&target) {
            continue;
        }
        let mut flow = edge(format!("flw:{}", m.id(f)), EdgeKind::Flow, source, None, target, None, d.name.clone().unwrap_or_default());
        flow.exchange_item = d.typed_by.first().map(|t| t.rsplit("::").next().unwrap_or(t).trim_matches('\'').to_string());
        edges.push(flow);
    }
    let mut seen = HashSet::new();
    edges.retain(|e| seen.insert(e.id.clone()));
    if edges.is_empty() {
        return None;
    }
    let name = m.name(package);
    Some(diagram(format!("{}:{name}", ViewKind::Iv.id()), ViewKind::Iv, format!("Interconnection View: {name}"), nodes, edges))
}

/// An action with its steps nested in it, its items in and out as ports.
fn action_tree(m: &Model2<'_>, index: usize, edges: &mut Vec<Edge>) -> Node {
    let id = m.id(index);
    let steps: Vec<usize> = m.children(index).iter().copied().filter(|&c| m.declarations[c].metaclass == "ActionUsage").collect();
    let mut children: Vec<Node> = steps.iter().map(|&s| action_tree(m, s, edges)).collect();
    // Fork, join, decision and merge nodes: the dots the steps go through.
    for &c in m.children(index).iter().filter(|&&c| CONTROL_NODES.contains(&m.declarations[c].metaclass)) {
        let mut dot = m.node(c, NodeKind::Junction, Vec::new(), Vec::new(), Vec::new());
        dot.name = keyword(m.declarations[c].metaclass.trim_end_matches("Node"));
        children.push(dot);
    }
    // `then`: the successions between the steps, as the action declares them.
    for &c in m.children(index).iter().filter(|&&c| m.declarations[c].metaclass == "SuccessionAsUsage") {
        let Some((first, then)) = &m.declarations[c].ends else { continue };
        let (Some(a), Some(b)) = (m.scope.chain(first, Some(index)).and_then(|c| c.last().copied()), m.scope.chain(then, Some(index)).and_then(|c| c.last().copied())) else { continue };
        let (a, b) = (m.id(a), m.id(b));
        edges.push(edge(format!("seq:{a} {b}"), EdgeKind::Succession, a, None, b, None, "then".to_string()));
    }
    let ports = m.ports(index, &["ItemUsage"], &id);
    let mut node = m.node(index, NodeKind::SysmlAction, children, ports, Vec::new());
    // A step `:> 'SF-ALT'` reads as the action it performs.
    if let Some(general) = m.declarations[index].specializes.first() {
        node.name = format!("{} :> {}", m.name(index), general.rsplit("::").next().unwrap_or(general).trim_matches('\''));
    }
    node
}

/// The actions of a package and the flows between their items.
fn action_flow(m: &Model2<'_>, package: usize) -> Option<Diagram> {
    let actions: Vec<usize> = m.children(package).iter().copied().filter(|&c| m.declarations[c].metaclass == "ActionUsage").collect();
    // A part that performs actions of its own (`perform action`) is the lane holding them.
    let lanes: Vec<(usize, Vec<usize>)> = m
        .children(package)
        .iter()
        .copied()
        .filter(|&p| m.declarations[p].metaclass == "PartUsage")
        .map(|p| (p, m.children(p).iter().copied().filter(|&c| m.declarations[c].metaclass == "PerformActionUsage" && m.declarations[c].references.is_empty()).collect::<Vec<_>>()))
        .filter(|(_, held)| !held.is_empty())
        .collect();
    if actions.is_empty() && lanes.is_empty() {
        return None;
    }
    let mut edges = Vec::new();
    let mut nodes: Vec<Node> = Vec::new();
    for (part, held) in &lanes {
        let children: Vec<Node> = held.iter().map(|&a| action_tree(m, a, &mut edges)).collect();
        nodes.push(m.node(*part, NodeKind::SysmlPart, children, Vec::new(), Vec::new()));
    }
    nodes.extend(actions.iter().map(|&a| action_tree(m, a, &mut edges)));
    let (mut ports, mut drawn) = (HashSet::new(), HashSet::new());
    port_ids(&nodes, &mut ports);
    node_ids(&nodes, &mut drawn);
    let mut sources: Vec<Node> = Vec::new();
    for &f in m.children(package).iter().filter(|&&c| FLOWS.contains(&m.declarations[c].metaclass)) {
        let d = &m.declarations[f];
        let Some((from, to)) = &d.ends else { continue };
        let (Some((source, source_port)), Some((target, target_port))) = (end_of(m, from, Some(package), &ports), end_of(m, to, Some(package), &ports)) else { continue };
        // A part a flow comes from or goes to (an actor feeding a function) is drawn beside the actions.
        for (end, written) in [(&source, from), (&target, to)] {
            if drawn.contains(end) {
                continue;
            }
            let Some(last) = m.scope.chain(written, Some(package)).and_then(|c| c.last().copied()) else { continue };
            if m.declarations[last].metaclass == "PartUsage" && m.declarations[last].owner == Some(package) {
                sources.push(m.node(last, NodeKind::SysmlPart, Vec::new(), Vec::new(), Vec::new()));
                drawn.insert(end.clone());
            }
        }
        if !drawn.contains(&source) || !drawn.contains(&target) {
            continue;
        }
        let mut flow = edge(format!("flw:{}", m.id(f)), EdgeKind::Flow, source, source_port, target, target_port, d.name.clone().unwrap_or_default());
        flow.exchange_item = d.typed_by.first().map(|t| t.rsplit("::").next().unwrap_or(t).trim_matches('\'').to_string());
        edges.push(flow);
    }
    nodes.extend(sources);
    // Only what the view draws, once each.
    let mut seen = HashSet::new();
    edges.retain(|e| drawn.contains(&e.source) && drawn.contains(&e.target) && seen.insert(e.id.clone()));
    // Lanes of actions and nothing between them is no flow to show.
    if actions.is_empty() && edges.is_empty() {
        return None;
    }
    let name = m.name(package);
    Some(diagram(format!("{}:{name}", ViewKind::Afv.id()), ViewKind::Afv, format!("Action Flow View: {name}"), nodes, edges))
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_metaclass_reads_as_its_keyword() {
        assert_eq!(super::keyword("PartDefinition"), "part def");
        assert_eq!(super::keyword("FlowConnectionUsage"), "flow connection");
        assert_eq!(super::keyword("PortUsage"), "port");
    }
}
