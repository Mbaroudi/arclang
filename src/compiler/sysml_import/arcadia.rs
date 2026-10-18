//! From a SysML v2 element graph to ArcLang source: the Arcadia reading of
//! a SysML model.
//!
//! The structure of a SysML model is its usages: `part engine : Engine` is
//! the component, `Engine` its type. So an ArcLang element is a usage, and
//! a definition is an ArcLang `type` its usages are `is:`. Two shortcuts
//! keep models written the other way round whole: a definition used once
//! by an anonymous usage (`part p_X : X`) is one element, named and
//! identified by the definition; a definition used nowhere is an element
//! of its own.
//!
//! Levels come from the `OA`/`SA`/`LA`/`PA`/`EPBS` metadata (or an
//! `Arcadia` metadata with a `level` attribute), else from the names of the
//! enclosing packages, else from the parent, else from what the element
//! is: actions are system functions, parts logical components. What SysML
//! cannot say by itself is read from metadata too: `Actor`, `Entity`,
//! `Node`, `Behaviour`, `Hardware`, `Mission`, `Capability`, `Chain`,
//! `Link`, `Hazard`, `Mode`; failing that, from the documentation ArcLang's
//! own export writes (`Name (role)`).
//!
//! Every element is written with its SysML short name as id when it has
//! one, else its name: a model exported with its ids comes back with them.

use super::records::{Element, Graph};
use crate::compiler::source_edit::string_literal;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Level {
    Oa,
    Sa,
    La,
    Pa,
    Epbs,
}

impl Level {
    fn of_name(name: &str) -> Option<Level> {
        let lower = name.to_ascii_lowercase();
        let compact: String = lower.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        match compact.as_str() {
            "oa" | "operationalanalysis" | "operational" => Some(Level::Oa),
            "sa" | "systemanalysis" | "system" => Some(Level::Sa),
            "la" | "logicalarchitecture" | "logical" => Some(Level::La),
            "pa" | "physicalarchitecture" | "physical" => Some(Level::Pa),
            "epbs" | "epbsarchitecture" | "productbreakdown" | "product" => Some(Level::Epbs),
            _ => {
                if lower.contains("operational") { Some(Level::Oa) }
                else if lower.contains("system analysis") { Some(Level::Sa) }
                else if lower.contains("logical") { Some(Level::La) }
                else if lower.contains("physical") { Some(Level::Pa) }
                else if lower.contains("epbs") || lower.contains("product breakdown") { Some(Level::Epbs) }
                else { None }
            }
        }
    }
}

/// What was written, with the report of the import.
pub struct Mapped {
    pub source: String,
    pub counts: BTreeMap<&'static str, usize>,
    pub notes: Vec<String>,
}

/// The role a part plays, from metadata or ArcLang's own documentation line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Actor,
    Entity,
    Component,
    Node,
    Behaviour,
    Hardware,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    Part,
    Action,
}

/// One structural element: a usage, its definition, or both merged.
struct Node<'g> {
    usage: Option<&'g Element>,
    def: Option<&'g Element>,
    /// The definition gives the identity (an anonymous single usage, or no usage).
    merged: bool,
    shape: Shape,
    id: String,
    name: String,
    parent: Option<usize>,
    children: Vec<usize>,
    level: Level,
    role: Option<Role>,
    /// The ArcLang type the element `is`, when its definition is shared.
    is_type: Option<String>,
}

struct Mapper<'g> {
    graph: &'g Graph,
    nodes: Vec<Node<'g>>,
    /// Element id (usage or definition) -> node.
    node_of: HashMap<String, usize>,
    /// Definitions written as ArcLang types: definition id -> type name.
    types: HashMap<String, String>,
    out: String,
    counts: BTreeMap<&'static str, usize>,
    notes: Vec<String>,
    /// Written id of every written element; ports as `Owner.port`.
    ids: HashMap<String, String>,
    kinds: HashMap<String, &'static str>,
    taken: HashSet<String>,
    /// References said by dependencies named after ArcLang keys
    /// (`dependency involves from A to B`): source element id -> (key, target id).
    dependency_attrs: HashMap<String, Vec<(String, String)>>,
}

pub fn map(graph: &Graph) -> Mapped {
    let mut mapper = Mapper {
        graph,
        nodes: Vec::new(),
        node_of: HashMap::new(),
        types: HashMap::new(),
        out: String::new(),
        counts: BTreeMap::new(),
        notes: Vec::new(),
        ids: HashMap::new(),
        kinds: HashMap::new(),
        taken: HashSet::new(),
        dependency_attrs: HashMap::new(),
    };
    mapper.run();
    Mapped { source: mapper.out, counts: mapper.counts, notes: mapper.notes }
}

const EXPORTED_ROLES: &[&str] = &[
    "actor", "operational entity", "system component", "logical component", "physical node", "behaviour component", "hardware component", "entity", "node",
    "function", "activity",
];

/// Ancestors under which a usage is a reference, not structure.
const REFERENCE_CONTEXTS: &[&str] = &[
    "OccurrenceDefinition", "OccurrenceUsage", "InteractionDefinition", "RequirementDefinition", "RequirementUsage", "VerificationCaseDefinition",
    "VerificationCaseUsage", "AnalysisCaseDefinition", "AnalysisCaseUsage", "ConstraintDefinition", "ConstraintUsage", "CalculationDefinition",
    "CalculationUsage", "ConnectionUsage", "InterfaceUsage", "FlowUsage", "FlowConnectionUsage", "AllocationUsage", "SatisfyRequirementUsage",
    "Dependency", "StateDefinition", "StateUsage", "TransitionUsage", "UseCaseDefinition", "UseCaseUsage", "ConcernDefinition", "ViewDefinition",
    "ViewUsage", "MetadataDefinition", "MetadataUsage", "SuccessionAsUsage", "ExhibitStateUsage",
];

impl<'g> Mapper<'g> {
    fn count(&mut self, what: &'static str) {
        *self.counts.entry(what).or_insert(0) += 1;
    }
    fn count_by(&mut self, what: &'static str, n: usize) {
        if n > 0 {
            *self.counts.entry(what).or_insert(0) += n;
        }
    }
    fn line(&mut self, depth: usize, text: &str) {
        for _ in 0..depth {
            self.out.push_str("    ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    // --- names, roles, levels ----------------------------------------------

    /// The documentation ArcLang's export writes: `Name (role)`.
    fn exported_doc(element: &Element) -> Option<(String, String)> {
        let doc = element.docs.first()?;
        let open = doc.rfind(" (")?;
        let role = doc[open + 2..].strip_suffix(')')?;
        EXPORTED_ROLES.contains(&role).then(|| (doc[..open].to_string(), role.to_string()))
    }

    fn name_of(element: &Element) -> Option<String> {
        if let Some((name, _)) = Self::exported_doc(element) {
            return Some(name);
        }
        element.name.clone()
    }

    fn description(element: &Element) -> Option<String> {
        let exported = Self::exported_doc(element).map(|(n, r)| format!("{n} ({r})"));
        element.docs.iter().find(|d| exported.as_deref() != Some(d.as_str())).cloned()
    }

    fn has_metadata(element: &Element, names: &[&str]) -> bool {
        element.metadata.iter().any(|m| names.iter().any(|n| m.eq_ignore_ascii_case(n)))
    }

    fn role_of_element(&self, element: &Element) -> Option<Role> {
        if Self::has_metadata(element, &["Actor", "ArcadiaActor", "SystemActor", "OperationalActor"]) { return Some(Role::Actor); }
        if Self::has_metadata(element, &["Entity", "OperationalEntity"]) { return Some(Role::Entity); }
        if Self::has_metadata(element, &["Node", "PhysicalNode"]) { return Some(Role::Node); }
        if Self::has_metadata(element, &["Behaviour", "Behavior", "BehaviorComponent"]) { return Some(Role::Behaviour); }
        if Self::has_metadata(element, &["Hardware", "HardwareComponent"]) { return Some(Role::Hardware); }
        if Self::has_metadata(element, &["Component", "LogicalComponent", "SystemComponent"]) { return Some(Role::Component); }
        if let Some((_, role)) = Self::exported_doc(element) {
            return Some(match role.as_str() {
                "actor" => Role::Actor,
                "operational entity" | "entity" => Role::Entity,
                "physical node" | "node" => Role::Node,
                "behaviour component" => Role::Behaviour,
                "hardware component" => Role::Hardware,
                _ => Role::Component,
            });
        }
        if element.specializes.iter().chain(element.types.iter()).any(|t| self.graph.get(t).and_then(|e| e.name.as_deref()) == Some("Actor")) {
            return Some(Role::Actor);
        }
        None
    }

    /// The level stated on an element or by its enclosing packages.
    fn stated_level(&self, element: &Element) -> Option<Level> {
        for m in &element.metadata {
            if let Some(level) = Level::of_name(m) {
                return Some(level);
            }
        }
        for (_, attribute, value) in &element.metadata_values {
            if attribute.eq_ignore_ascii_case("level") {
                if let Some(level) = Level::of_name(value.trim_matches('"')) {
                    return Some(level);
                }
            }
        }
        let mut current = element.owner.as_deref().and_then(|o| self.graph.get(o));
        let mut hops = 0;
        while let Some(owner) = current {
            if owner.kind == "Package" || owner.kind == "LibraryPackage" {
                if let Some(level) = owner.name.as_deref().and_then(Level::of_name) {
                    return Some(level);
                }
            }
            if let Some(level) = owner.metadata.iter().find_map(|m| Level::of_name(m)) {
                return Some(level);
            }
            hops += 1;
            if hops > 32 { break; }
            current = owner.owner.as_deref().and_then(|o| self.graph.get(o));
        }
        None
    }

    fn is_library(&self, element: &Element) -> bool {
        element.is_library || self.graph.ancestor(&element.id, |e| e.kind == "LibraryPackage" || e.is_library).is_some()
    }

    /// Whether a usage is structure (a component, a function) rather than a
    /// reference inside a scenario, a requirement, a connector or a chain.
    fn is_structural_usage(&self, usage: &Element) -> bool {
        if usage.is_end || self.is_library(usage) {
            return false;
        }
        let mut current = usage.owner.as_deref().and_then(|o| self.graph.get(o));
        let mut hops = 0;
        while let Some(owner) = current {
            if REFERENCE_CONTEXTS.contains(&owner.kind.as_str()) {
                return false;
            }
            // A step of a chain: an action usage typed by another action
            // inside an action with successions.
            if (owner.kind == "ActionDefinition" || owner.kind == "ActionUsage") && self.is_chain(owner) && usage.kind == "ActionUsage" {
                return false;
            }
            if owner.kind == "Package" || owner.kind == "LibraryPackage" {
                break;
            }
            hops += 1;
            if hops > 32 { break; }
            current = owner.owner.as_deref().and_then(|o| self.graph.get(o));
        }
        true
    }

    /// An action made of steps typed by other actions is a chain.
    fn is_chain(&self, action: &Element) -> bool {
        if Self::has_metadata(action, &["Chain", "FunctionalChain", "Process", "OperationalProcess"]) {
            return true;
        }
        let children: Vec<&Element> = action.children.iter().filter_map(|c| self.graph.get(c)).collect();
        let typed_steps = children.iter().filter(|c| c.kind == "ActionUsage" && c.types.iter().any(|t| matches!(self.graph.kind_of(t), "ActionDefinition" | "ActionUsage"))).count();
        let successions = children.iter().filter(|c| c.kind == "SuccessionAsUsage" || c.kind == "Succession").count();
        typed_steps >= 2 && successions >= 1
    }

    // --- the structural nodes ---------------------------------------------

    fn build_nodes(&mut self) {
        // Usages that are structure.
        let usages: Vec<&'g Element> = self
            .graph
            .elements
            .iter()
            .filter(|e| matches!(e.kind.as_str(), "PartUsage" | "ActionUsage" | "PerformActionUsage"))
            .filter(|e| self.is_structural_usage(e))
            .collect();
        let mut usages_of_def: HashMap<&str, Vec<usize>> = HashMap::new();
        for usage in &usages {
            let shape = if usage.kind == "PartUsage" { Shape::Part } else { Shape::Action };
            let def = usage.types.first().and_then(|t| self.graph.get(t)).filter(|d| matches!(d.kind.as_str(), "PartDefinition" | "ActionDefinition"));
            let index = self.nodes.len();
            self.nodes.push(Node { usage: Some(usage), def, merged: false, shape, id: String::new(), name: String::new(), parent: None, children: Vec::new(), level: Level::La, role: None, is_type: None });
            self.node_of.insert(usage.id.clone(), index);
            if let Some(def) = def {
                usages_of_def.entry(def.id.as_str()).or_default().push(index);
            }
        }
        // Definitions: merged into their single anonymous usage, elements of
        // their own when used nowhere, types otherwise.
        let defs: Vec<&'g Element> = self.graph.of_kinds(&["PartDefinition", "ActionDefinition"]).filter(|d| !self.is_library(d)).collect();
        for def in defs {
            match usages_of_def.get(def.id.as_str()).map(Vec::as_slice) {
                None => {
                    let shape = if def.kind == "PartDefinition" { Shape::Part } else { Shape::Action };
                    let index = self.nodes.len();
                    self.nodes.push(Node { usage: None, def: Some(def), merged: true, shape, id: String::new(), name: String::new(), parent: None, children: Vec::new(), level: Level::La, role: None, is_type: None });
                    self.node_of.insert(def.id.clone(), index);
                }
                Some([single]) if self.nodes[*single].usage.is_some_and(|u| u.short_name.is_none() && u.docs.is_empty() && !u.children.iter().any(|c| self.graph.kind_of(c) == "MetadataUsage")) => {
                    self.nodes[*single].merged = true;
                    self.node_of.insert(def.id.clone(), *single);
                }
                Some(many) => {
                    let name = Self::name_of(def).or_else(|| def.short_name.clone()).unwrap_or_else(|| def.id.clone());
                    self.types.insert(def.id.clone(), name.clone());
                    for &index in many {
                        self.nodes[index].is_type = Some(name.clone());
                    }
                    // The definition's own structure is written under the first usage.
                    self.node_of.insert(def.id.clone(), many[0]);
                }
            }
        }
        // Identity, parent, role.
        for index in 0..self.nodes.len() {
            let (usage, def, merged) = (self.nodes[index].usage, self.nodes[index].def, self.nodes[index].merged);
            let identity = if merged { def.or(usage) } else { usage.or(def) }.expect("a node has an element");
            let other = if merged { usage } else { def };
            let short = identity.short_name.clone().or_else(|| other.and_then(|o| o.short_name.clone()));
            let name = Self::name_of(identity).or_else(|| other.and_then(Self::name_of)).unwrap_or_else(|| identity.id.clone());
            let id = short.unwrap_or_else(|| name.clone());
            let role = self.role_of_element(identity).or_else(|| other.and_then(|o| self.role_of_element(o)));
            let parent = usage.and_then(|u| self.parent_of(u));
            self.nodes[index].name = name;
            self.nodes[index].id = id;
            self.nodes[index].role = role;
            self.nodes[index].parent = parent;
        }
        for index in 0..self.nodes.len() {
            if let Some(parent) = self.nodes[index].parent {
                self.nodes[parent].children.push(index);
            }
        }
        // Levels: stated, else the parent's (resolved first), else by shape and role.
        let stated: Vec<Option<Level>> = (0..self.nodes.len())
            .map(|i| {
                let node = &self.nodes[i];
                node.usage.and_then(|u| self.stated_level(u)).or_else(|| node.def.and_then(|d| self.stated_level(d)))
            })
            .collect();
        let mut resolved: Vec<Option<Level>> = vec![None; self.nodes.len()];
        for i in 0..self.nodes.len() {
            self.resolve_level(i, &stated, &mut resolved, 0);
        }
        for i in 0..self.nodes.len() {
            self.nodes[i].level = resolved[i].expect("every node has a level");
        }
    }

    /// A node's level: stated on it, else its parent's, else the default
    /// of its shape and role.
    fn resolve_level(&self, index: usize, stated: &[Option<Level>], resolved: &mut Vec<Option<Level>>, depth: usize) -> Level {
        if let Some(level) = resolved[index] {
            return level;
        }
        let level = stated[index].unwrap_or_else(|| {
            let node = &self.nodes[index];
            match node.parent {
                Some(parent) if depth < 64 => self.resolve_level(parent, stated, resolved, depth + 1),
                _ => match (node.shape, node.role) {
                    (Shape::Action, _) => Level::Sa,
                    (Shape::Part, Some(Role::Node) | Some(Role::Behaviour) | Some(Role::Hardware)) => Level::Pa,
                    (Shape::Part, Some(Role::Entity)) => Level::Oa,
                    (Shape::Part, _) => Level::La,
                },
            }
        });
        resolved[index] = Some(level);
        level
    }

    /// The node a usage sits in: the nearest owning usage that is a node,
    /// or the node of the owning definition.
    fn parent_of(&self, usage: &Element) -> Option<usize> {
        let mut current = usage.owner.as_deref();
        let mut hops = 0;
        while let Some(owner_id) = current {
            if let Some(&index) = self.node_of.get(owner_id) {
                return Some(index);
            }
            let owner = self.graph.get(owner_id)?;
            if owner.kind == "Package" || owner.kind == "LibraryPackage" {
                return None;
            }
            hops += 1;
            if hops > 32 { return None; }
            current = owner.owner.as_deref();
        }
        None
    }

    /// The elements a node's content comes from: its usage and, when
    /// merged or the first of a type, its definition.
    fn sources(&self, index: usize) -> Vec<&'g Element> {
        let node = &self.nodes[index];
        let mut out = Vec::new();
        if let Some(u) = node.usage { out.push(u); }
        if let Some(d) = node.def {
            let first_of_type = self.node_of.get(&d.id) == Some(&index);
            if node.merged || first_of_type {
                out.push(d);
            }
        }
        out
    }

    /// The node an element id refers to: a usage, a definition, or a
    /// usage standing for one.
    fn node_for(&self, id: &str) -> Option<usize> {
        self.node_for_depth(id, 0)
    }

    fn node_for_depth(&self, id: &str, depth: usize) -> Option<usize> {
        if let Some(&i) = self.node_of.get(id) {
            return Some(i);
        }
        if depth > 8 {
            return None;
        }
        let element = self.graph.get(id)?;
        // A reference usage (an end, a lifeline) stands for what it subsets or is typed by.
        for target in element.subsets.iter().chain(element.redefines.iter()).chain(element.types.iter()) {
            if let Some(i) = self.node_for_depth(target, depth + 1) {
                return Some(i);
            }
        }
        None
    }

    // --- ends of connectors -----------------------------------------------

    /// The written reference of a connector end: a node id, or
    /// `Owner.port` for a port or a directed parameter, through feature
    /// chains (`a.b.port`) and reference ends.
    fn end_ref(&self, id: &str) -> Option<String> {
        self.end_ref_depth(id, 0)
    }

    fn end_ref_depth(&self, id: &str, depth: usize) -> Option<String> {
        if depth > 8 {
            return None;
        }
        let element = self.graph.get(id)?;
        if !element.chaining.is_empty() {
            // `a.b.port`: the last feature on its owner, the deepest usage.
            let last = element.chaining.last()?;
            let last_element = self.graph.get(last)?;
            if self.is_port_like(last_element) {
                let owner = element.chaining.iter().rev().nth(1).and_then(|o| self.node_for(o)).or_else(|| last_element.owner.as_deref().and_then(|o| self.node_for(o)))?;
                return Some(format!("{}.{}", self.ids.get(&self.node_key(owner))?, last_element.name.clone()?));
            }
            return self.end_ref_depth(last, depth + 1);
        }
        if self.is_port_like(element) {
            let owner = element.owner.as_deref().and_then(|o| self.node_for(o))?;
            return Some(format!("{}.{}", self.ids.get(&self.node_key(owner))?, element.name.clone()?));
        }
        if element.is_end || element.kind == "ReferenceUsage" || element.kind == "Feature" || element.kind == "FlowEnd" {
            for target in element.subsets.iter().chain(element.redefines.iter()) {
                if let Some(found) = self.end_ref_depth(target, depth + 1) {
                    return Some(found);
                }
            }
        }
        let index = self.node_for(id)?;
        self.ids.get(&self.node_key(index)).cloned()
    }

    fn is_port_like(&self, element: &Element) -> bool {
        element.kind == "PortUsage" || (matches!(element.kind.as_str(), "ItemUsage" | "ReferenceUsage" | "AttributeUsage" | "Feature") && element.direction.is_some())
    }

    /// The key a node is written under in `ids`.
    fn node_key(&self, index: usize) -> String {
        format!("node:{index}")
    }

    /// The node holding an end (the part or action of a port, the node itself).
    fn end_node(&self, id: &str) -> Option<usize> {
        self.end_node_depth(id, 0)
    }

    fn end_node_depth(&self, id: &str, depth: usize) -> Option<usize> {
        if depth > 8 {
            return None;
        }
        let element = self.graph.get(id)?;
        if !element.chaining.is_empty() {
            let last = element.chaining.last()?;
            let last_element = self.graph.get(last)?;
            if self.is_port_like(last_element) {
                return element.chaining.iter().rev().nth(1).and_then(|o| self.node_for(o)).or_else(|| last_element.owner.as_deref().and_then(|o| self.node_for(o)));
            }
            return self.end_node_depth(last, depth + 1);
        }
        if self.is_port_like(element) {
            return element.owner.as_deref().and_then(|o| self.node_for(o));
        }
        if element.is_end || element.kind == "ReferenceUsage" || element.kind == "Feature" || element.kind == "FlowEnd" {
            for target in element.subsets.iter().chain(element.redefines.iter()) {
                if let Some(found) = self.end_node_depth(target, depth + 1) {
                    return Some(found);
                }
            }
        }
        self.node_for(id)
    }

    // --- running ----------------------------------------------------------

    fn run(&mut self) {
        self.build_nodes();
        self.collect_dependency_attributes();
        let model_name = self
            .graph
            .roots()
            .into_iter()
            .find(|e| e.kind == "Package")
            .and_then(|e| e.name.clone())
            .unwrap_or_else(|| "Imported".to_string());
        self.line(0, &format!("model {} {{", string_literal(&model_name)));
        self.line(1, &format!("name: {}", string_literal(&model_name)));
        self.line(1, "description: \"Imported from SysML v2\"");
        self.line(0, "}");
        self.line(0, "");
        self.data_model();
        self.type_definitions();
        self.requirements();
        for level in [Level::Oa, Level::Sa, Level::La, Level::Pa] {
            self.level(level);
        }
        self.epbs();
        self.state_machines();
        self.scenarios();
        self.test_cases();
        self.traces();
    }

    // --- data -------------------------------------------------------------

    fn data_model(&mut self) {
        let enumerations: Vec<&Element> = self.graph.of_kinds(&["EnumerationDefinition"]).filter(|e| !self.is_library(e)).collect();
        for e in enumerations {
            let id = self.remember(&e.id, Self::identity_id(e), "Enumeration");
            let values: Vec<String> = e.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "EnumerationUsage").filter_map(|c| c.name.clone()).collect();
            self.count("enumerations");
            self.line(0, &format!("enumeration {} {{", string_literal(&Self::display(e))));
            self.line(1, &format!("id: {}", string_literal(&id)));
            self.line(1, &format!("values: {}", list_literal(&values)));
            self.line(0, "}");
        }
        let data_types: Vec<&Element> = self.graph.of_kinds(&["AttributeDefinition", "DataType"]).filter(|e| !self.is_library(e)).collect();
        for e in data_types {
            let id = self.remember(&e.id, Self::identity_id(e), "DataType");
            let base = e.specializes.first().and_then(|s| self.graph.get(s)).and_then(|s| s.name.clone()).unwrap_or_else(|| "String".to_string());
            self.count("data types");
            self.line(0, &format!("data_type {} {{", string_literal(&Self::display(e))));
            self.line(1, &format!("id: {}", string_literal(&id)));
            self.line(1, &format!("base: {}", string_literal(&base)));
            self.line(0, "}");
        }
        let items: Vec<&Element> = self.graph.of_kinds(&["ItemDefinition"]).filter(|e| !self.is_library(e)).collect();
        for e in items {
            let fields: Vec<(String, String)> = e
                .children
                .iter()
                .filter_map(|c| self.graph.get(c))
                .filter(|c| c.kind == "AttributeUsage" || c.kind == "ItemUsage")
                .filter_map(|c| Some((c.name.clone()?, c.types.first().and_then(|t| self.graph.get(t)).and_then(|t| t.name.clone()).unwrap_or_else(|| "String".to_string()))))
                .collect();
            if !fields.is_empty() || Self::has_metadata(e, &["Class"]) {
                let id = self.remember(&e.id, Self::identity_id(e), "Class");
                self.count("classes");
                self.line(0, &format!("class {} {{", string_literal(&Self::display(e))));
                self.line(1, &format!("id: {}", string_literal(&id)));
                if let Some(parent) = e.specializes.first().and_then(|s| self.ids.get(s)).cloned() {
                    self.line(1, &format!("extends: {}", string_literal(&parent)));
                }
                for (name, ty) in fields {
                    self.line(1, &format!("{}: {}", field_key(&name), string_literal(&ty)));
                }
                self.line(0, "}");
            } else {
                let id = self.remember(&e.id, Self::identity_id(e), "ExchangeItem");
                self.count("exchange items");
                self.line(0, &format!("exchange_item {} {{ id: {} }}", string_literal(&Self::display(e)), string_literal(&id)));
            }
        }
        if !self.out.ends_with("}\n\n") {
            self.line(0, "");
        }
    }

    /// Shared definitions as ArcLang types: their ports and attributes.
    fn type_definitions(&mut self) {
        let mut entries: Vec<(String, String)> = self.types.iter().map(|(id, name)| (id.clone(), name.clone())).collect();
        entries.sort();
        for (def_id, name) in entries {
            let Some(def) = self.graph.get(&def_id) else { continue };
            self.count("types (shared definitions)");
            let parent = def.specializes.first().and_then(|s| self.types.get(s)).cloned();
            match parent {
                Some(parent) => self.line(0, &format!("type {} extends {} {{", string_literal(&name), string_literal(&parent))),
                None => self.line(0, &format!("type {} {{", string_literal(&name))),
            }
            if let Some(doc) = Self::description(def) {
                self.line(1, &format!("description: {}", string_literal(&doc)));
            }
            self.attributes(1, &[def], &["description"]);
            // A type with ports is a component's; a function carries the
            // ports of its type itself.
            if def.kind == "PartDefinition" {
                for (port, direction) in self.ports_of(def) {
                    self.line(1, &format!("port {direction} {}", string_literal(&port.name.clone().unwrap_or_else(|| port.id.clone()))));
                }
            }
            self.line(0, "}");
        }
        if !self.types.is_empty() {
            self.line(0, "");
        }
    }

    fn display(element: &Element) -> String {
        Self::name_of(element).or_else(|| element.short_name.clone()).unwrap_or_else(|| element.id.clone())
    }

    fn identity_id(element: &Element) -> String {
        element.short_name.clone().unwrap_or_else(|| Self::display(element))
    }

    // --- requirements -----------------------------------------------------

    /// Elements of a kind that may be written as definition or usage: the
    /// usages at package level (with the definition they are typed by), and
    /// the definitions used nowhere.
    fn declared(&self, def_kind: &str, usage_kind: &str) -> Vec<(&'g Element, Option<&'g Element>)> {
        let usages: Vec<&'g Element> = self
            .graph
            .elements
            .iter()
            .filter(|e| e.kind == usage_kind && !e.is_end && !self.is_library(e))
            .filter(|e| self.graph.ancestor(e.owner.as_deref().unwrap_or(""), |a| REFERENCE_CONTEXTS.contains(&a.kind.as_str()) || matches!(a.kind.as_str(), "PartDefinition" | "PartUsage" | "ActionDefinition" | "ActionUsage")).is_none())
            .collect();
        let used: HashSet<&str> = usages.iter().filter_map(|u| u.types.first().map(String::as_str)).collect();
        let mut out: Vec<(&'g Element, Option<&'g Element>)> = usages.iter().map(|u| (*u, u.types.first().and_then(|t| self.graph.get(t)).filter(|d| d.kind == def_kind))).collect();
        for def in self.graph.elements.iter().filter(|d| d.kind == def_kind && !self.is_library(d) && !used.contains(d.id.as_str())) {
            if self.graph.ancestor(def.owner.as_deref().unwrap_or(""), |a| REFERENCE_CONTEXTS.contains(&a.kind.as_str())).is_none() {
                out.push((def, None));
            }
        }
        out
    }

    /// The element carrying the identity of a usage/definition pair.
    fn identity_of<'a>(usage: &'a Element, def: Option<&'a Element>) -> &'a Element {
        match def {
            Some(d) if usage.short_name.is_none() && d.short_name.is_some() => d,
            _ => usage,
        }
    }

    fn requirements(&mut self) {
        let all = self.declared("RequirementDefinition", "RequirementUsage");
        let (hazards, requirements): (Vec<_>, Vec<_>) = all.into_iter().partition(|(e, d)| {
            Self::has_metadata(e, &["Hazard"]) || d.is_some_and(|d| Self::has_metadata(d, &["Hazard"])) || e.name.as_deref().is_some_and(|n| n.starts_with("hazard_"))
        });
        if !requirements.is_empty() {
            self.line(0, "requirements {");
            for (e, def) in requirements {
                let identity = Self::identity_of(e, def);
                let id = self.remember(&e.id, Self::identity_id(identity), "Requirement");
                if let Some(d) = def { self.ids.entry(d.id.clone()).or_insert(id.clone()); }
                self.count("requirements");
                self.line(1, &format!("requirement {} {{", string_literal(&id)));
                let sources: Vec<&Element> = [Some(e), def].into_iter().flatten().collect();
                if let Some(doc) = sources.iter().find_map(|s| Self::description(s)) {
                    self.line(2, &format!("description: {}", string_literal(&doc)));
                }
                if let Some(name) = Self::name_of(identity).filter(|n| *n != id && !n.starts_with("req_")) {
                    self.line(2, &format!("title: {}", string_literal(&name)));
                }
                self.attributes(2, &sources, &["description", "title"]);
                let element_ids: Vec<String> = sources.iter().map(|s| s.id.clone()).collect();
                self.reference_attributes(2, &element_ids);
                self.line(1, "}");
            }
            self.line(0, "}");
            self.line(0, "");
        }
        if !hazards.is_empty() {
            self.line(0, "safety_analysis {");
            for (e, def) in hazards {
                let identity = Self::identity_of(e, def);
                let id = self.remember(&e.id, Self::identity_id(identity), "Hazard");
                self.count("hazards");
                self.line(1, &format!("hazard {} {{", string_literal(&id)));
                let sources: Vec<&Element> = [Some(e), def].into_iter().flatten().collect();
                if let Some(doc) = sources.iter().find_map(|s| Self::description(s)) {
                    self.line(2, &format!("description: {}", string_literal(&doc)));
                }
                self.attributes(2, &sources, &["description"]);
                let element_ids: Vec<String> = sources.iter().map(|s| s.id.clone()).collect();
                self.reference_attributes(2, &element_ids);
                self.line(1, "}");
            }
            self.line(0, "}");
            self.line(0, "");
        }
    }

    /// The attribute usages of elements with a bound literal, as attributes.
    fn attributes(&mut self, depth: usize, elements: &[&Element], skip: &[&str]) {
        let mut lines = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        let mut references = 0;
        for element in elements {
            for child in element.children.iter().filter_map(|c| self.graph.get(c)) {
                if child.kind != "AttributeUsage" && child.kind != "ReferenceUsage" {
                    continue;
                }
                let (Some(name), Some(value)) = (&child.name, &child.value) else { continue };
                let key = field_key(name);
                if skip.contains(&key.as_str()) || matches!(key.as_str(), "id" | "name") || !seen.insert(key.clone()) {
                    continue;
                }
                // A reference ArcLang checks (what an element involves,
                // realizes, allocates...) is a relationship, not text.
                if REFERENCE_KEYS.contains(&key.as_str()) {
                    references += 1;
                    continue;
                }
                lines.push(format!("{key}: {value}"));
            }
        }
        for line in lines {
            self.line(depth, &line);
        }
        self.count_by("reference attributes written as text (left out: a relationship is expected)", references);
    }

    /// Register a written element under its element id; the written id is unique.
    fn remember(&mut self, element_id: &str, wanted: String, kind: &'static str) -> String {
        let mut id = wanted;
        while !self.taken.insert(id.clone()) {
            id.push('_');
        }
        self.ids.insert(element_id.to_string(), id.clone());
        self.kinds.insert(element_id.to_string(), kind);
        id
    }

    fn remember_node(&mut self, index: usize, kind: &'static str) -> String {
        let wanted = self.nodes[index].id.clone();
        let key = self.node_key(index);
        let id = self.remember(&key, wanted, kind);
        if let Some(u) = self.nodes[index].usage {
            self.ids.insert(u.id.clone(), id.clone());
            self.kinds.insert(u.id.clone(), kind);
        }
        if let Some(d) = self.nodes[index].def {
            if self.node_of.get(&d.id) == Some(&index) {
                self.ids.insert(d.id.clone(), id.clone());
                self.kinds.insert(d.id.clone(), kind);
            }
        }
        self.nodes[index].id = id.clone();
        id
    }

    /// Dependencies named after an ArcLang reference key say what an element
    /// involves, realizes, is owned by...: kept by their source element.
    fn collect_dependency_attributes(&mut self) {
        for dependency in self.graph.of_kinds(&["Dependency"]) {
            let Some(name) = dependency.name.as_deref().map(str::to_ascii_lowercase) else { continue };
            if !REFERENCE_KEYS.contains(&name.as_str()) {
                continue;
            }
            let (Some(source), Some(target)) = (dependency.sources.first(), dependency.targets.first()) else { continue };
            self.dependency_attrs.entry(source.clone()).or_default().push((name.clone(), target.clone()));
        }
    }

    /// The id an element will be written under, before it is written: a
    /// node's id, else its short name or name (through a definition used once).
    fn intended_id(&self, id: &str) -> Option<String> {
        if let Some(node) = self.end_node(id) {
            return Some(self.nodes[node].id.clone());
        }
        if let Some(written) = self.ids.get(id) {
            return Some(written.clone());
        }
        let element = self.graph.get(id)?;
        let def = element.types.first().and_then(|t| self.graph.get(t));
        Some(Self::identity_id(Self::identity_of(element, def)))
    }

    /// Write the reference attributes said by dependencies from any of
    /// `element_ids` (a usage and its definition).
    fn reference_attributes(&mut self, depth: usize, element_ids: &[String]) {
        let mut by_key: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for id in element_ids {
            if let Some(entries) = self.dependency_attrs.get(id).cloned() {
                for (key, target) in entries {
                    match self.intended_id(&target) {
                        Some(written) => by_key.entry(key).or_default().push(written),
                        None => self.count("references by dependency to elements not written (left out)"),
                    }
                }
            }
        }
        for (key, targets) in by_key {
            self.count("references by dependency");
            if LIST_KEYS.contains(&key.as_str()) || targets.len() > 1 && !SINGLE_KEYS.contains(&key.as_str()) {
                self.line(depth, &format!("{key}: {}", list_literal(&targets)));
            } else {
                if targets.len() > 1 {
                    self.count_by("references by dependency beyond the single one ArcLang takes (left out)", targets.len() - 1);
                }
                self.line(depth, &format!("{key}: {}", string_literal(&targets[0])));
            }
        }
    }

    /// The element ids a node is referred to by: its usage and definition.
    fn node_element_ids(&self, index: usize) -> Vec<String> {
        let node = &self.nodes[index];
        let mut ids = Vec::new();
        if let Some(u) = node.usage { ids.push(u.id.clone()); }
        if let Some(d) = node.def {
            if node.merged || self.node_of.get(&d.id) == Some(&index) { ids.push(d.id.clone()); }
        }
        ids
    }

    // --- levels -----------------------------------------------------------

    /// The nodes of a shape written at the top of a level: those without a
    /// parent at the same level (a function in a part is written in it).
    fn roots_at(&self, level: Level, shape: Shape) -> Vec<usize> {
        (0..self.nodes.len())
            .filter(|&i| self.nodes[i].shape == shape && self.nodes[i].level == level)
            .filter(|&i| match self.nodes[i].parent {
                None => true,
                Some(p) => self.nodes[p].level != level,
            })
            .collect()
    }

    fn level(&mut self, level: Level) {
        let parts = self.roots_at(level, Shape::Part);
        let actions = self.roots_at(level, Shape::Action);
        let chains: Vec<&Element> = self.graph.of_kinds(&["ActionDefinition", "ActionUsage"]).filter(|e| !self.is_library(e) && self.is_chain(e) && self.chain_level(e) == level).collect();
        let use_cases: Vec<(&Element, Option<&Element>)> = self.declared("UseCaseDefinition", "UseCaseUsage").into_iter().filter(|(e, d)| self.element_level(e, *d) == level).collect();
        let connectors = self.connectors_at(level);
        if parts.is_empty() && actions.is_empty() && chains.is_empty() && use_cases.is_empty() && connectors.is_empty() {
            return;
        }
        let (keyword, title) = match level {
            Level::Oa => ("operational_analysis", "Operational Analysis"),
            Level::Sa => ("system_analysis", "System Analysis"),
            Level::La => ("logical_architecture", "Logical Architecture"),
            Level::Pa => ("physical_architecture", "Physical Architecture"),
            Level::Epbs => ("epbs", "EPBS"),
        };
        self.line(0, &format!("{keyword} {} {{", string_literal(title)));
        match level {
            Level::Oa => {
                for part in &parts {
                    self.operational_part(*part);
                }
                for action in &actions {
                    self.function(1, *action, "operational_activity");
                }
            }
            Level::Sa => {
                // Functions first: components list what they perform.
                for action in &actions {
                    self.function(1, *action, "function");
                }
                for part in &parts {
                    let performed: Vec<usize> = self.nodes[*part].children.iter().copied().filter(|&c| self.nodes[c].shape == Shape::Action).collect();
                    for action in &performed {
                        self.function(1, *action, "function");
                    }
                    self.system_part(*part, &performed);
                }
            }
            Level::La => {
                for part in &parts {
                    self.logical_component(1, *part);
                }
                for action in &actions {
                    self.function(1, *action, "function");
                }
            }
            Level::Pa => {
                for part in &parts {
                    self.physical_node(*part);
                }
                for action in &actions {
                    self.function(1, *action, "function");
                }
            }
            Level::Epbs => {}
        }
        for connector in &connectors {
            self.exchange(level, connector);
        }
        for (use_case, def) in &use_cases {
            let keyword = match level {
                Level::Oa => "operational_capability",
                Level::Sa => {
                    if Self::has_metadata(use_case, &["Mission"]) || def.is_some_and(|d| Self::has_metadata(d, &["Mission"])) { "mission" } else { "capability" }
                }
                _ => "capability_realization",
            };
            self.use_case(use_case, *def, keyword);
        }
        for chain in &chains {
            self.chain(chain, if level == Level::Oa { "operational_process" } else { "functional_chain" });
        }
        self.line(0, "}");
        self.line(0, "");
    }

    fn element_level(&self, element: &Element, def: Option<&Element>) -> Level {
        self.stated_level(element).or_else(|| def.and_then(|d| self.stated_level(d))).unwrap_or(Level::Sa)
    }

    /// A chain's level: stated, else that of its first written step.
    fn chain_level(&self, chain: &Element) -> Level {
        if let Some(level) = self.stated_level(chain) {
            return level;
        }
        chain
            .children
            .iter()
            .filter_map(|c| self.graph.get(c))
            .filter(|c| c.kind == "ActionUsage")
            .filter_map(|c| c.types.first())
            .filter_map(|t| self.node_for(t))
            .map(|i| self.nodes[i].level)
            .next()
            .unwrap_or(Level::Sa)
    }

    fn ports_of(&self, element: &Element) -> Vec<(&'g Element, &'static str)> {
        element
            .children
            .iter()
            .filter_map(|c| self.graph.get(c))
            .filter_map(|c| {
                if c.kind == "PortUsage" {
                    let direction = c.direction.as_deref().map(|d| match d { "out" => "out", "inout" => "inout", _ => "in" });
                    let by_type = c.types.first().and_then(|t| self.graph.get(t)).and_then(|t| t.name.as_deref()).map(|n| {
                        let lower = n.to_ascii_lowercase();
                        if lower.contains("inout") { "inout" } else if lower.contains("out") { "out" } else { "in" }
                    });
                    Some((c, direction.or(by_type).unwrap_or("in")))
                } else if matches!(c.kind.as_str(), "ItemUsage" | "ReferenceUsage" | "AttributeUsage") && c.direction.is_some() && !c.is_end {
                    Some((c, match c.direction.as_deref() { Some("out") => "out", Some("inout") => "inout", _ => "in" }))
                } else {
                    None
                }
            })
            .collect()
    }

    fn write_ports(&mut self, depth: usize, index: usize, oriented: bool) {
        let owner_id = self.nodes[index].id.clone();
        let mut written: HashSet<String> = HashSet::new();
        let mut sources = self.sources(index);
        // A function of a shared action type carries that type's ports.
        if self.nodes[index].shape == Shape::Action {
            if let Some(def) = self.nodes[index].def.filter(|d| self.types.contains_key(&d.id)) {
                if !sources.iter().any(|s| s.id == def.id) {
                    sources.push(def);
                }
            }
        }
        for source in sources {
            for (port, direction) in self.ports_of(source) {
                let name = port.name.clone().unwrap_or_else(|| port.id.clone());
                if !written.insert(name.clone()) {
                    continue;
                }
                self.ids.insert(port.id.clone(), format!("{owner_id}.{name}"));
                self.count("ports");
                if oriented {
                    self.line(depth, &format!("port {direction} {}", string_literal(&name)));
                } else {
                    self.line(depth, &format!("port {}", string_literal(&name)));
                }
            }
        }
    }

    fn header(&mut self, depth: usize, index: usize, id: &str) {
        self.line(depth, &format!("id: {}", string_literal(id)));
        let sources = self.sources(index);
        if let Some(doc) = sources.iter().find_map(|s| Self::description(s)) {
            self.line(depth, &format!("description: {}", string_literal(&doc)));
        }
        if let Some(ty) = self.nodes[index].is_type.clone() {
            self.line(depth, &format!("is: {}", string_literal(&ty)));
        }
        self.attributes(depth, &sources, &["description"]);
        let element_ids = self.node_element_ids(index);
        self.reference_attributes(depth, &element_ids);
    }

    fn function(&mut self, depth: usize, index: usize, keyword: &str) {
        if self.ids.contains_key(&self.node_key(index)) {
            return;
        }
        let id = self.remember_node(index, "Function");
        self.count(if keyword.ends_with("activity") { "activities" } else { "functions" });
        let name = self.nodes[index].name.clone();
        self.line(depth, &format!("{keyword} {} {{", string_literal(&name)));
        self.header(depth + 1, index, &id);
        self.write_ports(depth + 1, index, true);
        let subs: Vec<usize> = self.nodes[index].children.iter().copied().filter(|&c| self.nodes[c].shape == Shape::Action).collect();
        for sub in subs {
            self.function(depth + 1, sub, keyword);
        }
        self.line(depth, "}");
    }

    fn operational_part(&mut self, index: usize) {
        let role = self.nodes[index].role.unwrap_or(Role::Entity);
        let keyword = if role == Role::Actor { "actor" } else { "entity" };
        let id = self.remember_node(index, if role == Role::Actor { "Actor" } else { "Entity" });
        self.count(if role == Role::Actor { "operational actors" } else { "operational entities" });
        let name = self.nodes[index].name.clone();
        self.line(1, &format!("{keyword} {} {{", string_literal(&name)));
        self.header(2, index, &id);
        let children = self.nodes[index].children.clone();
        for child in children {
            match self.nodes[child].shape {
                Shape::Action => self.function(2, child, "activity"),
                Shape::Part => {
                    let note = format!("entity '{}' holds entity '{}': nested entities are left out", name, self.nodes[child].name);
                    self.notes.push(note);
                }
            }
        }
        self.line(1, "}");
    }

    fn system_part(&mut self, index: usize, performed: &[usize]) {
        let role = self.nodes[index].role.unwrap_or(Role::Component);
        let keyword = if role == Role::Actor { "actor" } else { "system_component" };
        let id = self.remember_node(index, if role == Role::Actor { "SystemActor" } else { "SystemComponent" });
        self.count(if role == Role::Actor { "system actors" } else { "system components" });
        let name = self.nodes[index].name.clone();
        self.line(1, &format!("{keyword} {} {{", string_literal(&name)));
        self.header(2, index, &id);
        self.write_ports(2, index, true);
        let allocated: Vec<String> = performed.iter().filter_map(|&a| self.ids.get(&self.node_key(a)).cloned()).collect();
        if !allocated.is_empty() {
            self.line(2, &format!("allocated_functions: {}", list_literal(&allocated)));
        }
        self.line(1, "}");
    }

    fn logical_component(&mut self, depth: usize, index: usize) {
        let role = self.nodes[index].role;
        let id = self.remember_node(index, "LogicalComponent");
        self.count("logical components");
        let name = self.nodes[index].name.clone();
        self.line(depth, &format!("component {} {{", string_literal(&name)));
        self.header(depth + 1, index, &id);
        if role == Some(Role::Actor) {
            self.line(depth + 1, "actor: true");
        }
        self.write_ports(depth + 1, index, true);
        let children = self.nodes[index].children.clone();
        for child in children {
            match self.nodes[child].shape {
                Shape::Action => self.function(depth + 1, child, "function"),
                Shape::Part => self.logical_component(depth + 1, child),
            }
        }
        self.line(depth, "}");
    }

    fn physical_node(&mut self, index: usize) {
        let role = self.nodes[index].role;
        let id = self.remember_node(index, "PhysicalNode");
        self.count("physical nodes");
        let name = self.nodes[index].name.clone();
        self.line(1, &format!("node {} {{", string_literal(&name)));
        self.header(2, index, &id);
        if role == Some(Role::Actor) {
            self.line(2, "actor: true");
        }
        self.write_ports(2, index, true);
        let children = self.nodes[index].children.clone();
        for child in children {
            match self.nodes[child].shape {
                Shape::Action => self.function(2, child, "function"),
                Shape::Part => self.behaviour(child),
            }
        }
        // Logical components allocated to the node.
        let deployed: Vec<String> = self
            .graph
            .of_kinds(&["AllocationUsage"])
            .filter(|a| a.targets.first().and_then(|t| self.end_node(t)) == Some(index))
            .filter_map(|a| a.sources.first())
            .filter_map(|s| self.end_node(s))
            .filter(|&s| self.kinds.get(&self.node_key(s)) == Some(&"LogicalComponent"))
            .filter_map(|s| self.ids.get(&self.node_key(s)).cloned())
            .collect();
        for logical in deployed {
            self.count("deployments");
            self.line(2, &format!("deploys {}", string_literal(&logical)));
        }
        self.line(1, "}");
    }

    fn behaviour(&mut self, index: usize) {
        let name = self.nodes[index].name.clone();
        if self.nodes[index].role == Some(Role::Hardware) {
            let id = self.remember_node(index, "HardwareComponent");
            self.count("hardware components");
            self.line(2, &format!("hardware_component {} {{ id: {} }}", string_literal(&name), string_literal(&id)));
            return;
        }
        let id = self.remember_node(index, "BehaviorComponent");
        self.count("behaviour components");
        self.line(2, &format!("behavior_component {} {{", string_literal(&name)));
        self.header(3, index, &id);
        self.write_ports(3, index, true);
        if let Some(realized) = self.allocated_logical(index) {
            self.line(3, &format!("allocated_component: {}", string_literal(&realized)));
        }
        let children = self.nodes[index].children.clone();
        for child in children {
            match self.nodes[child].shape {
                Shape::Action => self.function(3, child, "function"),
                Shape::Part => {
                    let note = format!("behaviour '{}' holds part '{}': a behaviour component holds functions only", name, self.nodes[child].name);
                    self.notes.push(note);
                }
            }
        }
        self.line(2, "}");
    }

    /// The logical component a behaviour realizes: an allocation or a
    /// `realizes` dependency from it to a written logical component.
    fn allocated_logical(&self, index: usize) -> Option<String> {
        self.graph
            .of_kinds(&["AllocationUsage", "Dependency"])
            .filter(|a| a.sources.first().and_then(|s| self.end_node(s)) == Some(index))
            .filter_map(|a| a.targets.first())
            .filter_map(|t| self.end_node(t))
            .find(|&t| self.kinds.get(&self.node_key(t)) == Some(&"LogicalComponent"))
            .and_then(|t| self.ids.get(&self.node_key(t)).cloned())
    }

    fn use_case(&mut self, use_case: &Element, def: Option<&Element>, keyword: &str) {
        let identity = Self::identity_of(use_case, def);
        let id = self.remember(&use_case.id, Self::identity_id(identity), "Capability");
        if let Some(d) = def { self.ids.entry(d.id.clone()).or_insert(id.clone()); }
        self.count("capabilities and missions");
        self.line(1, &format!("{keyword} {} {{", string_literal(&Self::display(identity))));
        self.line(2, &format!("id: {}", string_literal(&id)));
        let sources: Vec<&Element> = [Some(use_case), def].into_iter().flatten().collect();
        if let Some(doc) = sources.iter().find_map(|s| Self::description(s)) {
            self.line(2, &format!("description: {}", string_literal(&doc)));
        }
        self.attributes(2, &sources, &["description"]);
        let element_ids: Vec<String> = sources.iter().map(|s| s.id.clone()).collect();
        self.reference_attributes(2, &element_ids);
        self.line(1, "}");
    }

    fn chain(&mut self, chain: &Element, keyword: &str) {
        let id = self.remember(&chain.id, Self::identity_id(chain), "FunctionalChain");
        let steps: Vec<String> = chain
            .children
            .iter()
            .filter_map(|c| self.graph.get(c))
            .filter(|c| c.kind == "ActionUsage")
            .filter_map(|c| c.types.first())
            .filter_map(|t| self.node_for(t))
            .filter_map(|i| self.ids.get(&self.node_key(i)).cloned())
            .collect();
        self.count("functional chains");
        self.line(1, &format!("{keyword} {} {{", string_literal(&Self::display(chain))));
        self.line(2, &format!("id: {}", string_literal(&id)));
        if let Some(doc) = Self::description(chain) {
            self.line(2, &format!("description: {}", string_literal(&doc)));
        }
        if !steps.is_empty() {
            self.line(2, &format!("involves: {}", list_literal(&steps)));
        }
        self.reference_attributes(2, &[chain.id.clone()]);
        self.line(1, "}");
    }

    // --- exchanges --------------------------------------------------------

    /// Connectors whose source lands at the level, outside scenarios.
    fn connectors_at(&self, level: Level) -> Vec<&'g Element> {
        self.graph
            .of_kinds(&["ConnectionUsage", "InterfaceUsage", "FlowUsage", "FlowConnectionUsage", "SuccessionFlowUsage", "BindingConnectionUsage"])
            .filter(|c| !self.is_library(c) && !self.inside_occurrence(c))
            .filter(|c| c.sources.first().and_then(|s| self.end_node(s)).is_some_and(|n| self.nodes[n].level == level))
            .collect()
    }

    fn inside_occurrence(&self, element: &Element) -> bool {
        self.graph.ancestor(element.owner.as_deref().unwrap_or(""), |e| e.kind == "OccurrenceDefinition" || e.kind == "OccurrenceUsage" || e.kind == "InteractionDefinition").is_some()
    }

    fn exchange(&mut self, level: Level, connector: &Element) {
        let (Some(source), Some(target)) = (connector.sources.first().cloned(), connector.targets.first().cloned()) else {
            self.count("connectors without two ends (left out)");
            return;
        };
        let (Some(from), Some(to)) = (self.end_ref(&source), self.end_ref(&target)) else {
            self.count("connectors to elements not written (left out)");
            return;
        };
        let shape_of = |mapper: &Self, end: &str| mapper.end_node(end).map(|n| mapper.nodes[n].shape);
        let functional = shape_of(self, &source) == Some(Shape::Action) && shape_of(self, &target) == Some(Shape::Action);
        let name = Self::display(connector);
        let id = self.remember(&connector.id, Self::identity_id(connector), if functional { "FunctionalExchange" } else { "ComponentExchange" });
        let mut items: Vec<String> = connector.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "ItemUsage" || c.kind == "PayloadFeature").filter_map(|c| c.types.first()).filter_map(|t| self.ids.get(t).cloned()).collect();
        if items.is_empty() {
            items = connector.types.iter().filter_map(|t| self.ids.get(t).cloned()).collect();
        }
        let item_text = if items.is_empty() { String::new() } else { format!(" exchange_items: {}", list_literal(&items)) };
        let is_link = Self::has_metadata(connector, &["Link", "PhysicalLink"]) || (connector.kind == "InterfaceUsage" && level == Level::Pa);
        match (level, functional) {
            (Level::Oa, true) => {
                self.count("operational interactions");
                self.line(1, &format!("interaction {} {{ id: {} from: {} to: {}{item_text} }}", string_literal(&name), string_literal(&id), string_literal(&from), string_literal(&to)));
            }
            (Level::Oa, false) => {
                self.count("communication means");
                self.line(1, &format!("communication_means {} {{ id: {} from: {} to: {} }}", string_literal(&name), string_literal(&id), string_literal(&strip_port(&from)), string_literal(&strip_port(&to))));
            }
            (_, true) => {
                self.count("functional exchanges");
                self.line(1, &format!("functional_exchange {} {{ id: {} from: {} to: {}{item_text} }}", string_literal(&name), string_literal(&id), string_literal(&from), string_literal(&to)));
            }
            (Level::Pa, false) if is_link => {
                self.count("physical links");
                self.line(1, &format!("physical_link {} {{ id: {} from: {} to: {} }}", string_literal(&name), string_literal(&id), string_literal(&from), string_literal(&to)));
            }
            (Level::Pa, false) => {
                self.count("physical exchanges");
                self.line(1, &format!("physical_exchange {} {{ id: {} from: {} to: {}{item_text} }}", string_literal(&name), string_literal(&id), string_literal(&from), string_literal(&to)));
            }
            (_, false) => {
                if !from.contains('.') || !to.contains('.') {
                    self.count("component exchanges without ports (left out: ArcLang needs `Component.port` ends)");
                    return;
                }
                self.count("component exchanges");
                self.line(1, &format!("component_exchange {} {{ id: {} from_port: {} to_port: {}{item_text} }}", string_literal(&name), string_literal(&id), string_literal(&from), string_literal(&to)));
            }
        }
    }

    // --- EPBS -------------------------------------------------------------

    fn epbs(&mut self) {
        let items: Vec<usize> = (0..self.nodes.len()).filter(|&i| self.nodes[i].level == Level::Epbs && self.nodes[i].shape == Shape::Part && self.nodes[i].parent.is_none()).collect();
        if items.is_empty() {
            return;
        }
        self.line(0, "epbs \"EPBS\" {");
        for item in items {
            let id = self.remember_node(item, "EpbsSystem");
            self.count("configuration items");
            let name = self.nodes[item].name.clone();
            self.line(1, &format!("system {} {{", string_literal(&name)));
            self.header(2, item, &id);
            let children = self.nodes[item].children.clone();
            for child in children {
                let child_id = self.remember_node(child, "EpbsSubsystem");
                let child_name = self.nodes[child].name.clone();
                self.line(2, &format!("subsystem {} {{", string_literal(&child_name)));
                self.line(3, &format!("id: {}", string_literal(&child_id)));
                let leaves = self.nodes[child].children.clone();
                for leaf in leaves {
                    let leaf_id = self.remember_node(leaf, "EpbsItem");
                    let leaf_name = self.nodes[leaf].name.clone();
                    self.line(3, &format!("item {} {{ id: {} }}", string_literal(&leaf_name), string_literal(&leaf_id)));
                }
                self.line(2, "}");
            }
            self.line(1, "}");
        }
        self.line(0, "}");
        self.line(0, "");
    }

    // --- behaviour --------------------------------------------------------

    fn state_machines(&mut self) {
        // A state definition, or a state usage exhibited by a part, holding states.
        let machines: Vec<&Element> = self
            .graph
            .elements
            .iter()
            .filter(|e| (e.kind == "StateDefinition" || e.kind == "StateUsage" || e.kind == "ExhibitStateUsage") && !self.is_library(e))
            .filter(|e| e.children.iter().any(|c| self.graph.kind_of(c) == "StateUsage"))
            .filter(|e| self.graph.ancestor(e.owner.as_deref().unwrap_or(""), |a| a.kind == "StateDefinition" || a.kind == "StateUsage").is_none())
            .collect();
        for machine in machines {
            let states: Vec<&Element> = machine.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "StateUsage").collect();
            let id = self.remember(&machine.id, Self::identity_id(machine), "StateMachine");
            self.count("state machines");
            self.line(0, &format!("state_machine {} {{", string_literal(&Self::display(machine))));
            self.line(1, &format!("id: {}", string_literal(&id)));
            match self.exhibited_by(machine) {
                Some(owner) => self.line(1, &format!("owner: {}", string_literal(&owner))),
                None => self.reference_attributes(1, &[machine.id.clone()]),
            }
            let mut state_names: HashMap<String, String> = HashMap::new();
            for state in &states {
                let name = Self::display(state);
                state_names.insert(state.id.clone(), name.clone());
                let state_id = self.remember(&state.id, Self::identity_id(state), "State");
                self.count("states");
                let kind = if Self::has_metadata(state, &["Mode"]) { "mode" } else { "state" };
                self.line(1, &format!("{kind} {} {{ id: {} }}", string_literal(&name), string_literal(&state_id)));
            }
            let initial = states.iter().find(|s| s.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case("start") || n.eq_ignore_ascii_case("initial")));
            let entry = machine.children.iter().filter_map(|c| self.graph.get(c)).find(|c| c.kind == "TransitionUsage" && c.sources.is_empty()).and_then(|t| t.targets.first()).and_then(|t| state_names.get(t));
            if let Some(initial) = entry.cloned().or_else(|| initial.and_then(|i| state_names.get(&i.id).cloned())).or_else(|| states.first().and_then(|s| state_names.get(&s.id).cloned())) {
                self.line(1, &format!("initial: {}", string_literal(&initial)));
            }
            let transitions: Vec<&Element> = machine.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "TransitionUsage").collect();
            for transition in transitions {
                let (Some(from), Some(to)) = (transition.sources.first().and_then(|s| state_names.get(s)).cloned(), transition.targets.first().and_then(|t| state_names.get(t)).cloned()) else {
                    self.count("transitions to states not written (left out)");
                    continue;
                };
                let mut attributes = Vec::new();
                if let Some(short) = &transition.short_name { attributes.push(format!("id: {}", string_literal(short))); }
                if let Some(trigger) = &transition.trigger { attributes.push(format!("trigger: {}", string_literal(trigger))); }
                if let Some(guard) = &transition.guard { attributes.push(format!("guard: {}", string_literal(guard))); }
                if let Some(effect) = &transition.effect { attributes.push(format!("action: {}", string_literal(effect))); }
                self.count("transitions");
                self.line(1, &format!("transition {} -> {} {{ {} }}", string_literal(&from), string_literal(&to), attributes.join(" ")));
            }
            self.line(0, "}");
            self.line(0, "");
        }
    }

    /// The written element that exhibits a state machine: its owner, or the
    /// owner of an `exhibit state` typed by it.
    fn exhibited_by(&self, machine: &Element) -> Option<String> {
        if let Some(owner) = machine.owner.as_deref().and_then(|o| self.end_node(o)) {
            return self.ids.get(&self.node_key(owner)).cloned();
        }
        self.graph
            .of_kinds(&["ExhibitStateUsage", "StateUsage"])
            .filter(|u| u.types.iter().any(|t| t == &machine.id))
            .filter_map(|u| u.owner.as_deref().and_then(|o| self.end_node(o)))
            .find_map(|o| self.ids.get(&self.node_key(o)).cloned())
    }

    fn scenarios(&mut self) {
        let scenarios: Vec<&Element> = self.graph.of_kinds(&["OccurrenceDefinition", "InteractionDefinition", "OccurrenceUsage"]).filter(|e| !self.is_library(e) && !self.inside_occurrence(e)).collect();
        for scenario in scenarios {
            let lifelines: Vec<(&Element, String)> = scenario
                .children
                .iter()
                .filter_map(|c| self.graph.get(c))
                .filter(|c| matches!(c.kind.as_str(), "PartUsage" | "OccurrenceUsage" | "ItemUsage" | "ReferenceUsage"))
                .filter_map(|c| {
                    let node = c.types.iter().chain(c.subsets.iter()).find_map(|t| self.node_for(t))?;
                    let id = self.ids.get(&self.node_key(node))?.clone();
                    Some((c, id))
                })
                .collect();
            let messages: Vec<&Element> = scenario.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| matches!(c.kind.as_str(), "FlowUsage" | "FlowConnectionUsage" | "SuccessionFlowUsage" | "MessageUsage")).collect();
            if lifelines.is_empty() || messages.is_empty() {
                continue;
            }
            let id = self.remember(&scenario.id, Self::identity_id(scenario), "Scenario");
            self.count("scenarios");
            let participants: Vec<String> = lifelines.iter().map(|(_, id)| id.clone()).collect();
            self.line(0, &format!("scenario {} {{", string_literal(&Self::display(scenario))));
            self.line(1, &format!("id: {}", string_literal(&id)));
            if let Some(doc) = Self::description(scenario) {
                self.line(1, &format!("description: {}", string_literal(&doc)));
            }
            self.line(1, &format!("participants: {}", list_literal(&participants)));
            self.reference_attributes(1, &[scenario.id.clone()]);
            let lifeline_of = |lifeline_id: &str| -> Option<String> { lifelines.iter().find(|(l, _)| l.id == lifeline_id).map(|(_, id)| id.clone()) };
            for message in messages {
                let (from, to) = (message.sources.first().and_then(|s| self.event_lifeline(s, 0)), message.targets.first().and_then(|t| self.event_lifeline(t, 0)));
                let (Some(from), Some(to)) = (from.as_deref().and_then(lifeline_of), to.as_deref().and_then(lifeline_of)) else {
                    self.count("messages of lifelines not written (left out)");
                    continue;
                };
                let label = Self::description(message).unwrap_or_else(|| Self::display(message));
                self.count("messages");
                self.line(1, &format!("message {} -> {} {}", string_literal(&from), string_literal(&to), string_literal(&label)));
            }
            self.line(0, "}");
            self.line(0, "");
        }
    }

    /// The lifeline (an element id) an event of a message belongs to:
    /// through feature chains (`lifeline.event`) and reference ends.
    fn event_lifeline(&self, id: &str, depth: usize) -> Option<String> {
        if depth > 8 {
            return None;
        }
        let element = self.graph.get(id)?;
        if !element.chaining.is_empty() {
            return element.chaining.first().cloned();
        }
        if element.is_end || element.kind == "FlowEnd" || element.kind == "ReferenceUsage" || element.kind == "Feature" {
            for target in element.subsets.iter().chain(element.redefines.iter()) {
                if let Some(found) = self.event_lifeline(target, depth + 1) {
                    return Some(found);
                }
            }
        }
        // An event owned by a lifeline.
        let owner = element.owner.as_deref()?;
        if matches!(self.graph.kind_of(owner), "PartUsage" | "OccurrenceUsage" | "ItemUsage" | "ReferenceUsage") {
            return Some(owner.to_string());
        }
        None
    }

    fn test_cases(&mut self) {
        let cases = self.declared("VerificationCaseDefinition", "VerificationCaseUsage");
        for (case, def) in cases {
            let mut verifies: Vec<String> = Vec::new();
            let mut stack: Vec<&Element> = [Some(case), def].into_iter().flatten().flat_map(|c| c.children.iter().filter_map(|g| self.graph.get(g))).collect();
            while let Some(c) = stack.pop() {
                if c.kind == "RequirementUsage" {
                    for r in c.subsets.iter().chain(c.types.iter()).chain(c.redefines.iter()) {
                        if let Some(id) = self.ids.get(r).cloned() {
                            if !verifies.contains(&id) {
                                verifies.push(id);
                            }
                        }
                    }
                    stack.extend(c.children.iter().filter_map(|g| self.graph.get(g)));
                }
            }
            if verifies.is_empty() {
                self.count("verification cases verifying no written requirement (left out)");
                continue;
            }
            let identity = Self::identity_of(case, def);
            let id = self.remember(&case.id, Self::identity_id(identity), "TestCase");
            self.count("test cases");
            self.line(0, &format!("test_case {} {{", string_literal(&Self::display(identity))));
            self.line(1, &format!("id: {}", string_literal(&id)));
            let sources: Vec<&Element> = [Some(case), def].into_iter().flatten().collect();
            if let Some(doc) = sources.iter().find_map(|s| Self::description(s)) {
                self.line(1, &format!("description: {}", string_literal(&doc)));
            }
            self.attributes(1, &sources, &["description"]);
            let element_ids: Vec<String> = sources.iter().map(|s| s.id.clone()).collect();
            self.reference_attributes(1, &element_ids);
            self.line(1, &format!("verifies: {}", list_literal(&verifies)));
            self.line(0, "}");
            self.line(0, "");
        }
    }

    // --- traces -----------------------------------------------------------

    /// The written id an end of a relationship refers to (an element, not a port).
    fn element_ref(&self, id: &str) -> Option<String> {
        if let Some(node) = self.end_node(id) {
            return self.ids.get(&self.node_key(node)).cloned();
        }
        if let Some(found) = self.ids.get(id) {
            return Some(found.clone());
        }
        let element = self.graph.get(id)?;
        for target in element.subsets.iter().chain(element.types.iter()).chain(element.redefines.iter()) {
            if let Some(found) = self.ids.get(target) {
                return Some(found.clone());
            }
        }
        None
    }

    fn traces(&mut self) {
        let mut lines: Vec<String> = Vec::new();
        let mut seen: HashSet<(String, String, String)> = HashSet::new();
        let relations: Vec<&Element> = self.graph.of_kinds(&["SatisfyRequirementUsage", "Dependency", "AllocationUsage"]).filter(|e| !self.is_library(e)).collect();
        for relation in relations {
            let (Some(source), Some(target)) = (relation.sources.first(), relation.targets.first()) else { continue };
            let (Some(from), Some(to)) = (self.element_ref(source), self.element_ref(target)) else {
                self.count("relations to elements not written (left out)");
                continue;
            };
            let kind = match relation.kind.as_str() {
                "SatisfyRequirementUsage" => "satisfies".to_string(),
                "AllocationUsage" => {
                    let from_kind = self.end_node(source).and_then(|n| self.kinds.get(&self.node_key(n)).copied());
                    let to_kind = self.end_node(target).and_then(|n| self.kinds.get(&self.node_key(n)).copied());
                    if from_kind == Some("LogicalComponent") && to_kind == Some("PhysicalNode") {
                        continue; // written as `deploys` in the node
                    }
                    "allocates".to_string()
                }
                _ => relation.name.clone().unwrap_or_else(|| "refines".to_string()).to_ascii_lowercase(),
            };
            if relation.kind == "Dependency" && REFERENCE_KEYS.contains(&kind.as_str()) && !TRACE_KINDS.contains(&kind.as_str()) {
                continue; // written as an attribute of its source
            }
            if !seen.insert((from.clone(), kind.clone(), to.clone())) {
                continue;
            }
            self.count("traces");
            if TRACE_KINDS.contains(&kind.as_str()) {
                lines.push(format!("trace {} {kind} {}", string_literal(&from), string_literal(&to)));
            } else {
                lines.push(format!("trace {} -> {} {{ trace_type: {} }}", string_literal(&from), string_literal(&to), string_literal(&kind)));
            }
        }
        for line in lines {
            self.line(0, &line);
        }
    }
}

/// Attribute keys ArcLang resolves as references: a SysML text attribute
/// under one of them is not a reference the import can trust.
const REFERENCE_KEYS: &[&str] = &[
    "involves", "realizes", "mission", "exploits", "extends", "includes", "allocated_functions", "allocated_function", "allocated_component",
    "deploys", "mitigated_by", "verifies", "satisfies", "implements", "refines", "from", "to", "from_port", "to_port", "via", "owner", "parent",
    "exchange_items", "exchange_item", "categories", "inputs", "outputs", "members", "applies_to", "data_type", "is", "trigger", "capability",
];

/// Reference keys that take a list.
const LIST_KEYS: &[&str] = &["involves", "mitigated_by", "allocated_functions", "exploits", "includes", "extends", "verifies", "members", "applies_to", "exchange_items", "inputs", "outputs", "categories"];
/// Reference keys that take one element.
const SINGLE_KEYS: &[&str] = &["mission", "parent", "owner", "via", "allocated_component", "realizes", "capability", "data_type", "exchange_item", "from", "to", "from_port", "to_port", "is", "trigger"];
/// Dependency names that are traces, not attributes.
const TRACE_KINDS: &[&str] = &["satisfies", "refines", "implements", "verifies", "validates", "allocates"];

fn list_literal(items: &[String]) -> String {
    format!("[{}]", items.iter().map(|i| string_literal(i)).collect::<Vec<_>>().join(", "))
}

fn strip_port(reference: &str) -> String {
    reference.split('.').next().unwrap_or(reference).to_string()
}

/// A SysML feature name as an ArcLang attribute key: an identifier that is
/// not a keyword the attribute parser refuses as a key.
fn field_key(name: &str) -> String {
    let key: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect();
    let key = if key.chars().next().is_some_and(|c| c.is_ascii_digit()) { format!("_{key}") } else { key };
    const KEYWORD_KEYS: &[&str] = &["description", "version", "author", "priority", "rationale", "verification", "traces", "safety_level", "parent", "properties", "signals", "function", "type"];
    let is_keyword = match crate::compiler::lexer::Lexer::new(&key).tokenize() {
        Ok(tokens) => !matches!(tokens.first(), Some(crate::compiler::lexer::Token::Identifier(_))),
        Err(_) => true,
    };
    if is_keyword && !KEYWORD_KEYS.contains(&key.as_str()) { format!("sysml_{key}") } else { key }
}
