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

use super::expression;
use super::records::{Element, Graph};
use crate::compiler::source_edit::string_literal;
use std::collections::{BTreeMap, HashMap, HashSet};

mod generic;
mod product_line;
mod security;

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
    /// A function written in the component that performs it: the function
    /// it decomposes (`parent:`), which stays at the level.
    decomposes: Option<usize>,
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
    /// (performer element id, performed element id) of every `perform`.
    performances: Vec<(String, String)>,
    /// Connectors `exchange` gave up on, counted already.
    lost: HashSet<String>,
    /// Names borne by more than one node: references to them are qualified.
    shared_names: HashSet<String>,
    /// The text uses the Arcadia profile (it is an ArcLang export or written
    /// for it); else it is any SysML v2 model, read with [`generic`] rules.
    profiled: bool,
    /// The model's own tags: metadata definitions without features outside
    /// any library (`metadata def Safety;`), written back as `tags:`.
    tags: HashSet<String>,
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
        performances: Vec::new(),
        shared_names: HashSet::new(),
        lost: HashSet::new(),
        profiled: graph.elements.iter().any(|e| e.name.as_deref() == Some("ArcadiaProfile")),
        tags: HashSet::new(),
    };
    mapper.tags = mapper.user_tags();
    mapper.run();
    Mapped { source: mapper.out, counts: mapper.counts, notes: mapper.notes }
}

/// Connectors written as exchanges.
const CONNECTOR_KINDS: &[&str] = &["ConnectionUsage", "InterfaceUsage", "FlowUsage", "FlowConnectionUsage", "SuccessionFlowUsage", "BindingConnectorAsUsage"];

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

    fn user_tags(&self) -> HashSet<String> {
        self.graph
            .of_kinds(&["MetadataDefinition"])
            .filter(|d| !self.is_library(d) && d.children.iter().filter_map(|c| self.graph.get(c)).all(|c| c.kind.ends_with("Membership") || c.kind == "Comment" || c.kind == "Documentation"))
            .filter_map(|d| d.name.clone())
            .collect()
    }

    /// The port definition the export gives the ports of a type that name
    /// no other (`port def 'Data port' { inout item carried : Data; }`).
    fn is_carrier(&self, def: &Element) -> bool {
        let features: Vec<&Element> = def.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind.ends_with("Usage")).collect();
        def.kind == "PortDefinition" && !def.is_abstract && def.name.as_deref().is_some_and(|n| n.ends_with(" port")) && features.iter().all(|f| f.name.as_deref() == Some("carried"))
    }

    fn is_library(&self, element: &Element) -> bool {
        element.is_library || self.graph.ancestor(&element.id, |e| e.kind == "LibraryPackage" || e.is_library).is_some()
    }

    /// Whether a usage is structure (a component, a function) rather than a
    /// reference inside a scenario, a requirement, a connector or a chain.
    fn is_structural_usage(&self, usage: &Element) -> bool {
        if usage.is_end || self.is_library(usage) || Self::has_metadata(usage, security::SECURITY_METADATA) || Self::has_metadata(usage, &["Configuration"]) {
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
        // Outside the profile, typed sub-actions are a decomposition.
        (self.profiled && typed_steps >= 2) || (typed_steps >= 1 && successions >= 1)
    }

    // --- the structural nodes ---------------------------------------------

    fn build_nodes(&mut self) {
        // Usages that are structure.
        let usages: Vec<&'g Element> = self
            .graph
            .elements
            .iter()
            // Outside the profile, accepting and sending are functions too.
            .filter(|e| matches!(e.kind.as_str(), "PartUsage" | "ActionUsage" | "PerformActionUsage") || (!self.profiled && matches!(e.kind.as_str(), "AcceptActionUsage" | "SendActionUsage")))
            .filter(|e| self.is_structural_usage(e) && !self.is_chain(e) && !Self::has_metadata(e, &["Path"]))
            .collect();
        let anonymous = |u: &Element| u.short_name.is_none() && u.docs.is_empty() && !u.children.iter().any(|c| self.graph.kind_of(c) == "MetadataUsage");
        // A `perform X` referencing an action elsewhere is a performance,
        // not an element: the part performs that function.
        let mut performances: Vec<(String, String)> = Vec::new();
        let mut aliases: HashMap<String, String> = HashMap::new();
        let mut candidates: Vec<&'g Element> = Vec::new();
        for usage in &usages {
            if usage.kind == "PerformActionUsage" && usage.types.is_empty() {
                if let Some(target) = usage.subsets.first().or(usage.redefines.first()) {
                    aliases.insert(usage.id.clone(), target.clone());
                    if let Some(owner) = &usage.owner {
                        performances.push((owner.clone(), target.clone()));
                    }
                    continue;
                }
            }
            candidates.push(usage);
        }
        // Definitions that are ArcLang types whatever their use: abstract
        // definitions, and those of a package named `Types`.
        let declared_types: Vec<&'g Element> = self
            .graph
            .of_kinds(&["PartDefinition", "ActionDefinition", "ConnectionDefinition", "InterfaceDefinition", "PortDefinition", "AllocationDefinition"])
            .filter(|d| !self.is_library(d) && !self.is_chain(d))
            .filter(|d| d.is_abstract || d.owner.as_deref().and_then(|o| self.graph.get(o)).and_then(|o| o.name.as_deref()) == Some("Types"))
            .filter(|d| !self.is_carrier(d))
            .collect();
        for def in declared_types {
            self.declare_type(def);
        }
        let def_of = |u: &Element| u.types.first().and_then(|t| self.graph.get(t)).filter(|d| matches!(d.kind.as_str(), "PartDefinition" | "ActionDefinition"));
        let mut by_def: HashMap<&str, Vec<&'g Element>> = HashMap::new();
        for usage in &candidates {
            if let Some(def) = def_of(usage) {
                by_def.entry(def.id.as_str()).or_default().push(usage);
            }
        }
        // Per definition: one anonymous usage -> one element named by the
        // definition; one identified usage among anonymous ones -> that
        // usage is the element, the others stand for it (a `perform` of
        // the level function); several -> a type and its usages.
        let mut merged_usages: HashSet<String> = HashSet::new();
        let mut main_of_def: HashMap<String, String> = HashMap::new();
        for (def_id, group) in &by_def {
            if self.types.contains_key(*def_id) {
                main_of_def.insert(def_id.to_string(), group[0].id.clone());
                continue;
            }
            let identified: Vec<&&Element> = group.iter().filter(|u| !anonymous(u)).collect();
            // Anonymous usages of one definition, at most one of them in a
            // part (`action a : X;` at the level and `perform action p : X`
            // in the part): one element, at home in the part.
            let nested: Vec<&&Element> = group.iter().filter(|u| u.owner.as_deref().map(|o| self.graph.kind_of(o)).is_some_and(|k| k != "Package" && k != "LibraryPackage")).collect();
            // Outside the profile only a definition's one usage merges with it.
            if identified.is_empty() && nested.len() <= 1 && (self.profiled || group.len() == 1) {
                let main = nested.first().copied().unwrap_or(&group[0]);
                merged_usages.insert(main.id.clone());
                main_of_def.insert(def_id.to_string(), main.id.clone());
                for u in group {
                    if u.id != main.id {
                        aliases.insert(u.id.clone(), main.id.clone());
                    }
                }
            } else if identified.len() == 1 && (self.profiled || group.len() == 1) {
                // The profile's `perform` of a level function; outside it,
                // several usages of one definition are distinct elements.
                let main = identified[0];
                main_of_def.insert(def_id.to_string(), main.id.clone());
                for u in group {
                    if u.id != main.id {
                        aliases.insert(u.id.clone(), main.id.clone());
                        if let Some(owner) = &u.owner {
                            performances.push((owner.clone(), main.id.clone()));
                        }
                    }
                }
            } else {
                let def = self.graph.get(def_id).expect("a grouped definition");
                self.declare_type(def);
                main_of_def.insert(def_id.to_string(), group[0].id.clone());
            }
        }
        for usage in &candidates {
            if aliases.contains_key(&usage.id) {
                continue;
            }
            let shape = if usage.kind == "PartUsage" { Shape::Part } else { Shape::Action };
            let def = def_of(usage);
            let is_type = def.and_then(|d| self.types.get(&d.id).cloned());
            let index = self.nodes.len();
            self.nodes.push(Node { usage: Some(usage), def, merged: merged_usages.contains(&usage.id), shape, id: String::new(), name: String::new(), parent: None, children: Vec::new(), level: Level::La, role: None, is_type, decomposes: None });
            self.node_of.insert(usage.id.clone(), index);
            if let Some(def) = def {
                if main_of_def.get(&def.id) == Some(&usage.id) {
                    self.node_of.insert(def.id.clone(), index);
                }
            }
        }
        for (alias, target) in &aliases {
            if let Some(&index) = self.node_of.get(target) {
                self.node_of.insert(alias.clone(), index);
            }
        }
        // Definitions used nowhere are elements of their own. Chains are
        // not structure: they are written from their steps.
        let defs: Vec<&'g Element> = self
            .graph
            .of_kinds(&["PartDefinition", "ActionDefinition"])
            .filter(|d| !self.is_library(d) && !self.is_chain(d) && !by_def.contains_key(d.id.as_str()) && !self.types.contains_key(&d.id))
            .collect();
        for def in defs {
            let shape = if def.kind == "PartDefinition" { Shape::Part } else { Shape::Action };
            let index = self.nodes.len();
            self.nodes.push(Node { usage: None, def: Some(def), merged: true, shape, id: String::new(), name: String::new(), parent: None, children: Vec::new(), level: Level::La, role: None, is_type: None, decomposes: None });
            self.node_of.insert(def.id.clone(), index);
        }
        self.performances = performances;
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
        // A function performed by a part and declared nowhere else sits in it.
        let performances = self.performances.clone();
        for (performer, performed) in &performances {
            let (Some(part), Some(action)) = (self.node_for(performer), self.node_for(performed)) else { continue };
            // Outside the profile a function is allocated, not nested (`allocated_functions`).
            if self.profiled && self.nodes[action].parent.is_none() && part != action && self.nodes[action].shape == Shape::Action {
                self.nodes[action].parent = Some(part);
            }
        }
        if !self.profiled {
            self.allocate_performed_functions();
            self.inherit_subsetted_types();
            self.shared_names = self.names_borne_twice();
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

    /// Register a definition as an ArcLang type. A type is known by its
    /// name: two definitions of one name (one per level) get distinct names.
    fn declare_type(&mut self, def: &Element) -> String {
        if let Some(name) = self.types.get(&def.id) {
            return name.clone();
        }
        let base = Self::name_of(def).or_else(|| def.short_name.clone()).unwrap_or_else(|| def.id.clone());
        let mut name = base.clone();
        let mut n = 1;
        while self.types.values().any(|existing| *existing == name) {
            n += 1;
            name = format!("{base} ({n})");
        }
        self.types.insert(def.id.clone(), name.clone());
        name
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
                    (Shape::Action, _) if self.profiled => Level::Sa,
                    // Any SysML model: functions sit with the parts that perform them.
                    (Shape::Action, _) => Level::La,
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
        // A type's attributes and documentation are the type's: written once, with it.
        if let Some(d) = node.def.filter(|_| node.is_type.is_none()) {
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
        // A feature chain (`perform tree.step;`) stands for its last step.
        if let Some(last) = element.chaining.last() {
            return self.node_for_depth(last, depth + 1);
        }
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
        if let Some((owner, port)) = self.outer_port(element) {
            return Some(format!("{}.{}", self.written_ref(owner)?, port));
        }
        if !element.chaining.is_empty() {
            // `a.b.port`: the last feature on its owner, the deepest usage.
            let last = element.chaining.last()?;
            let last_element = self.graph.get(last)?;
            if self.is_port_like(last_element) {
                let owner = element.chaining.iter().rev().nth(1).and_then(|o| self.node_for(o)).or_else(|| last_element.owner.as_deref().and_then(|o| self.node_for(o)))?;
                return Some(format!("{}.{}", self.written_ref(owner)?, last_element.name.clone()?));
            }
            return self.end_ref_depth(last, depth + 1);
        }
        if self.is_port_like(element) {
            let owner = element.owner.as_deref().and_then(|o| self.node_for(o))?;
            return Some(format!("{}.{}", self.written_ref(owner)?, element.name.clone()?));
        }
        if element.is_end || element.kind == "ReferenceUsage" || element.kind == "Feature" || element.kind == "FlowEnd" {
            for target in element.subsets.iter().chain(element.redefines.iter()) {
                if let Some(found) = self.end_ref_depth(target, depth + 1) {
                    return Some(found);
                }
            }
        }
        let index = self.node_for(id)?;
        self.written_ref(index)
    }

    fn is_port_like(&self, element: &Element) -> bool {
        element.kind == "PortUsage" || (matches!(element.kind.as_str(), "ItemUsage" | "ReferenceUsage" | "AttributeUsage" | "Feature") && element.direction.is_some()) || self.is_accepted_payload(element)
    }

    /// The payload of an `accept` (`accept cmd : Cmd via p`), outside the profile.
    fn is_accepted_payload(&self, element: &Element) -> bool {
        !self.profiled
            && element.name.is_some()
            && !element.is_end
            && matches!(element.kind.as_str(), "ItemUsage" | "ReferenceUsage" | "AttributeUsage")
            && element.owner.as_deref().is_some_and(|o| self.graph.kind_of(o) == "AcceptActionUsage")
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
        if let Some((owner, _)) = self.outer_port(element) {
            return Some(owner);
        }
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
        // The root package: its short name is the model's header name, its
        // name the model's `name:`.
        let root = self.graph.roots().into_iter().find(|e| e.kind == "Package");
        let model_name = root.and_then(|e| e.name.clone()).unwrap_or_else(|| "Imported".to_string());
        let header = root.and_then(|e| e.short_name.clone()).unwrap_or_else(|| model_name.clone());
        self.line(0, &format!("model {} {{", string_literal(&header)));
        self.line(1, &format!("name: {}", string_literal(&model_name)));
        match root.and_then(Self::description) {
            Some(doc) => self.line(1, &format!("description: {}", string_literal(&doc))),
            None => self.line(1, "description: \"Imported from SysML v2\""),
        }
        if let Some(root) = root {
            self.attributes(1, &[root], &["description", "name"]);
        }
        self.line(0, "}");
        self.line(0, "");
        self.data_model();
        self.type_definitions();
        self.requirements();
        for level in [Level::Oa, Level::Sa, Level::La, Level::Pa] {
            self.level(level);
        }
        self.epbs();
        self.count_connectors_left_out();
        self.state_machines();
        self.scenarios();
        self.message_scenarios();
        self.test_cases();
        self.security();
        self.configurations();
        self.constraints();
        self.traces();
        self.sysml_views();
    }

    // --- constraints ------------------------------------------------------

    /// `constraint` usages as ArcLang constraints: their asserted
    /// expression over the attributes of written elements. An expression
    /// ArcLang cannot say leaves the constraint out, and says so.
    fn constraints(&mut self) {
        let constraints: Vec<&'g Element> = self.graph.of_kinds(&["ConstraintUsage", "AssertConstraintUsage"]).filter(|c| !self.is_library(c) && !self.is_case_assertion(c)).collect();
        for constraint in constraints {
            let expression = constraint.children.iter().filter_map(|c| self.graph.get(c)).find(|c| c.kind.ends_with("Expression") || c.kind.starts_with("Literal"));
            let text = match expression {
                Some(expression) => self.expression_text(&expression.id),
                // The export keeps an expression it cannot write as text: ArcLang's own.
                None => constraint
                    .children
                    .iter()
                    .filter_map(|c| self.graph.get(c))
                    .find(|c| c.kind == "AttributeUsage" && c.name.as_deref() == Some("expression"))
                    .and_then(|c| c.value.as_deref())
                    .and_then(unquote)
                    .ok_or_else(|| "no expression".to_string()),
            };
            let text = match text {
                Ok(text) => text,
                Err(reason) => {
                    self.count("constraints left out (expression not ArcLang's)");
                    if self.notes.len() < 40 {
                        self.notes.push(format!("constraint '{}' left out: {reason}", Self::display(constraint)));
                    }
                    continue;
                }
            };
            let id = self.remember(&constraint.id, Self::identity_id(constraint), "Constraint");
            self.count("constraints");
            self.line(0, &format!("constraint {} {{", string_literal(&Self::display(constraint))));
            self.line(1, &format!("id: {}", string_literal(&id)));
            if let Some(doc) = Self::description(constraint) {
                self.line(1, &format!("description: {}", string_literal(&doc)));
            }
            self.attributes(1, &[constraint], &["description", "expression"]);
            self.line(1, &format!("assert: {text}"));
            self.line(0, "}");
        }
        if !self.ids.is_empty() {
            self.line(0, "");
        }
    }

    // --- data -------------------------------------------------------------

    fn data_model(&mut self) {
        // The enumerations of the security scales are scales, not data.
        let is_scale = |e: &Element| e.owner.as_deref().and_then(|o| self.graph.get(o)).is_some_and(|o| Self::has_metadata(o, security::SECURITY_METADATA));
        let enumerations: Vec<&Element> = self.graph.of_kinds(&["EnumerationDefinition"]).filter(|e| !self.is_library(e) && !is_scale(e)).collect();
        for e in enumerations {
            let id = self.remember(&e.id, Self::identity_id(e), "Enumeration");
            let values: Vec<String> = e.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "EnumerationUsage").filter_map(|c| c.name.clone()).collect();
            self.count("enumerations");
            self.line(0, &format!("enumeration {} {{", string_literal(&Self::display(e))));
            self.line(1, &format!("id: {}", string_literal(&id)));
            self.line(1, &format!("values: {}", list_literal(&values)));
            self.line(0, "}");
        }
        let data_types: Vec<&Element> = self.graph.of_kinds(&["AttributeDefinition", "DataType"]).filter(|e| !self.is_library(e) && !Self::has_metadata(e, security::SECURITY_METADATA)).collect();
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
            // A field is a typed member without a bound value; a member with
            // a value is an attribute of the item (its mechanism, its status).
            let fields: Vec<(String, String)> = e
                .children
                .iter()
                .filter_map(|c| self.graph.get(c))
                .filter(|c| (c.kind == "AttributeUsage" || c.kind == "ItemUsage") && c.value.is_none())
                .filter_map(|c| Some((c.name.clone()?, c.types.first().and_then(|t| self.graph.get(t)).and_then(|t| t.name.clone()).unwrap_or_else(|| "String".to_string()))))
                .filter(|(name, _)| !matches!(field_key(name).as_str(), "id" | "description" | "extends" | "summary" | "constraints" | "status"))
                .collect();
            let is_class = Self::has_metadata(e, &["Class"]) || (!fields.is_empty() && !Self::has_metadata(e, &["ExchangeItem"]));
            if is_class {
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
                if let Some(doc) = Self::description(e) {
                    self.line(1, &format!("description: {}", string_literal(&doc)));
                }
                self.attributes(1, &[e], &["description", "extends"]);
                self.line(0, "}");
            } else {
                let id = self.remember(&e.id, Self::identity_id(e), "ExchangeItem");
                self.count("exchange items");
                self.line(0, &format!("exchange_item {} {{", string_literal(&Self::display(e))));
                self.line(1, &format!("id: {}", string_literal(&id)));
                if let Some(doc) = Self::description(e) {
                    self.line(1, &format!("description: {}", string_literal(&doc)));
                }
                self.attributes(1, &[e], &["description"]);
                self.reference_attributes(1, &[e.id.clone()]);
                self.line(0, "}");
            }
        }
        self.associations();
        if !self.out.ends_with("}\n\n") {
            self.line(0, "");
        }
    }

    /// `#Association connection def` between two classes: its ends are the
    /// classes, its attributes the role, kind and multiplicity.
    fn associations(&mut self) {
        let associations: Vec<&'g Element> = self.graph.of_kinds(&["ConnectionDefinition", "AssociationStructure", "Association"]).filter(|e| !self.is_library(e) && Self::has_metadata(e, &["Association"])).collect();
        for association in associations {
            let ends: Vec<String> = association
                .children
                .iter()
                .filter_map(|c| self.graph.get(c))
                .filter(|c| c.is_end || c.kind == "ReferenceUsage")
                .filter_map(|c| c.types.first().or(c.subsets.first()))
                .filter_map(|t| self.ids.get(t).cloned())
                .collect();
            let [from, to] = ends.as_slice() else {
                self.count("associations without two class ends (left out)");
                continue;
            };
            let id = self.remember(&association.id, Self::identity_id(association), "Association");
            self.count("associations");
            self.line(0, &format!("association {} {{", string_literal(&Self::display(association))));
            self.line(1, &format!("id: {}", string_literal(&id)));
            self.line(1, &format!("from: {}", string_literal(from)));
            self.line(1, &format!("to: {}", string_literal(to)));
            if let Some(doc) = Self::description(association) {
                self.line(1, &format!("description: {}", string_literal(&doc)));
            }
            self.attributes(1, &[association], &["description", "from", "to"]);
            self.line(0, "}");
        }
    }

    /// Shared definitions as ArcLang types: their ports and attributes.
    fn type_definitions(&mut self) {
        let mut entries: Vec<(String, String)> = self.types.iter().map(|(id, name)| (id.clone(), name.clone())).collect();
        entries.sort();
        for (def_id, name) in entries {
            let Some(def) = self.graph.get(&def_id) else { continue };
            self.count("types");
            let parent = def.specializes.first().and_then(|s| self.types.get(s)).cloned();
            match parent {
                Some(parent) => self.line(0, &format!("type {} extends {} {{", string_literal(&name), string_literal(&parent))),
                None => self.line(0, &format!("type {} {{", string_literal(&name))),
            }
            if let Some(doc) = Self::description(def) {
                // The export says a type's required attributes in its documentation.
                match doc.strip_prefix("instances must provide:") {
                    Some(list) => {
                        let required: Vec<String> = list.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                        self.line(1, &format!("required: {}", list_literal(&required)));
                    }
                    None => self.line(1, &format!("description: {}", string_literal(&doc))),
                }
            }
            self.attributes(1, &[def], &["description"]);
            // A type with ports is a component's; a function carries the
            // ports of its type itself.
            // A port definition gives the ports a port of its type holds.
            if def.kind == "PartDefinition" || def.kind == "PortDefinition" {
                for (port, direction) in self.ports_of(def).into_iter().filter(|(p, _)| def.kind == "PartDefinition" || p.kind == "PortUsage") {
                    self.line(1, &format!("port {} {}", direction.unwrap_or("in"), string_literal(&port.name.clone().unwrap_or_else(|| port.id.clone()))));
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
        let is_tagged = |e: &Element, d: Option<&Element>, names: &[&str]| Self::has_metadata(e, names) || d.is_some_and(|d| Self::has_metadata(d, names));
        let (fmea, all): (Vec<_>, Vec<_>) = all.into_iter().partition(|(e, d)| is_tagged(e, *d, &["Fmea", "FMEA", "FmeaEntry"]));
        let (hazards, requirements): (Vec<_>, Vec<_>) = all.into_iter().partition(|(e, d)| {
            is_tagged(e, *d, &["Hazard"]) || e.name.as_deref().is_some_and(|n| n.starts_with("hazard_"))
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
        if !hazards.is_empty() || !fmea.is_empty() {
            self.line(0, "safety_analysis {");
            for (e, def) in fmea {
                let identity = Self::identity_of(e, def);
                let id = self.remember(&e.id, Self::identity_id(identity), "FmeaEntry");
                self.count("FMEA entries");
                self.line(1, &format!("fmea {} {{", string_literal(&Self::display(identity))));
                self.line(2, &format!("id: {}", string_literal(&id)));
                let sources: Vec<&Element> = [Some(e), def].into_iter().flatten().collect();
                if let Some(doc) = sources.iter().find_map(|s| Self::description(s)) {
                    self.line(2, &format!("description: {}", string_literal(&doc)));
                }
                self.attributes(2, &sources, &["description"]);
                let element_ids: Vec<String> = sources.iter().map(|s| s.id.clone()).collect();
                self.reference_attributes(2, &element_ids);
                self.line(1, "}");
            }
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
        let mut unwritten: Vec<String> = Vec::new();
        let mut computed: Vec<String> = Vec::new();
        let graph = self.graph;
        for element in elements {
            for child in element.children.iter().filter_map(|c| graph.get(c)) {
                if child.kind != "AttributeUsage" && child.kind != "ReferenceUsage" {
                    continue;
                }
                // A redefinition (`attribute :>> period = 33 [ms]`) is named by what it redefines.
                let name = child.name.clone().or_else(|| child.redefines.iter().filter_map(|r| graph.get(r)).find_map(|r| r.name.clone()));
                let Some(name) = name else { continue };
                // A value that is not a literal is an expression: a quantity (`30 [ms]`).
                let mut is_formula = false;
                let value = match (&child.value, &child.value_expression) {
                    (Some(value), _) => value.clone(),
                    // A value that is not a literal, a quantity or a list (a
                    // computation, a reference) is kept as text, and counted.
                    (None, Some(expression)) => match self.expression_text(expression) {
                        Ok(text) if expression::is_attribute_value(&text) => text,
                        Ok(text) => {
                            computed.push(format!("'{}': attribute '{name}' = {text} kept as text", Self::display(element)));
                            is_formula = true;
                            string_literal(&text)
                        }
                        Err(_) => match child.source_text.clone() {
                            Some(source) => {
                                computed.push(format!("'{}': attribute '{name}' = {source} kept as text", Self::display(element)));
                                is_formula = true;
                                string_literal(&source)
                            }
                            None => {
                                unwritten.push(format!("'{}': attribute '{name}' left out: no value ArcLang can say", Self::display(element)));
                                continue;
                            }
                        },
                    },
                    (None, None) => continue,
                };
                let key = field_key(&name);
                // A formula for a typed key (`mass = dryMass + cargoMass`) is
                // text, not the quantity the key holds: it keeps its own key.
                let key = if is_formula && !self.profiled && generic::is_typed_key(&key) { format!("{key}_expression") } else { key };
                if skip.contains(&key.as_str()) || matches!(key.as_str(), "id" | "name") || !seen.insert(key.clone()) {
                    continue;
                }
                // A reference ArcLang checks (what an element involves,
                // realizes, allocates...) is a relationship, not text.
                if REFERENCE_KEYS.contains(&key.as_str()) {
                    references += 1;
                    let note = format!("'{}': reference attribute '{key}' written as text is left out (a `dependency {key}` is expected)", Self::display(element));
                    if self.notes.len() < 40 && !self.notes.contains(&note) {
                        self.notes.push(note);
                    }
                    continue;
                }
                lines.push(format!("{key}: {value}"));
            }
        }
        for line in lines {
            self.line(depth, &line);
        }
        self.count_by("reference attributes written as text (left out: a relationship is expected)", references);
        self.count_by("attribute values that are not ArcLang values (left out)", unwritten.len());
        self.count_by("attribute values that are expressions (kept as text)", computed.len());
        for note in unwritten.into_iter().chain(computed) {
            if self.notes.len() < 40 {
                self.notes.push(note);
            }
        }
    }


    /// An expression element as ArcLang text (an attribute value, a
    /// constraint), or why it is not one.
    fn expression_text(&self, id: &str) -> Result<String, String> {
        let chains: Vec<(String, Vec<String>)> = self
            .graph
            .of_kinds(&["ActionDefinition", "ActionUsage"])
            .filter(|e| !self.is_library(e) && self.is_chain(e))
            .map(|chain| {
                let written = self.ids.get(&chain.id).cloned().unwrap_or_else(|| Self::identity_id(chain));
                (written, self.chain_steps(chain))
            })
            .collect();
        // A port is no element of an expression: ArcLang reads attributes of elements.
        expression::render(
            self.graph,
            id,
            &|element| {
                let is_end = self.graph.get(element).is_some_and(|e| e.is_end);
                if is_end || self.port_end(element).is_some() {
                    None
                } else {
                    self.element_ref(element).or_else(|| self.intended_id(element))
                }
            },
            &chains,
        )
    }

    /// The written ids of the functions a chain steps through.
    fn chain_steps(&self, chain: &Element) -> Vec<String> {
        chain
            .children
            .iter()
            .filter_map(|c| self.graph.get(c))
            .filter(|c| c.kind == "ActionUsage")
            .filter_map(|c| c.types.first().or(c.subsets.first()).or(c.redefines.first()))
            .filter_map(|t| self.end_node(t))
            .filter_map(|i| self.ids.get(&self.node_key(i)).cloned())
            .collect()
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
            if !REFERENCE_KEYS.contains(&name.as_str()) || Self::has_metadata(dependency, &["Trace"]) {
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
        self.reference_attributes_except(depth, element_ids, &[]);
    }

    fn reference_attributes_except(&mut self, depth: usize, element_ids: &[String], except: &[&str]) {
        let mut by_key: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for id in element_ids {
            if let Some(entries) = self.dependency_attrs.get(id).cloned() {
                for (key, target) in entries {
                    if except.contains(&key.as_str()) {
                        continue;
                    }
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
                // An operational actor holds no activity: its activities stand at the level.
                Some(p) => self.nodes[p].level != level || (level == Level::Oa && shape == Shape::Action && self.nodes[p].role == Some(Role::Actor)),
            })
            .collect()
    }

    fn level(&mut self, level: Level) {
        let parts = self.roots_at(level, Shape::Part);
        let actions = self.roots_at(level, Shape::Action);
        let chains: Vec<&Element> = self.graph.of_kinds(&["ActionDefinition", "ActionUsage"]).filter(|e| !self.is_library(e) && self.is_chain(e) && self.chain_level(e) == level).collect();
        let use_cases: Vec<(&Element, Option<&Element>)> = self.declared("UseCaseDefinition", "UseCaseUsage").into_iter().filter(|(e, d)| self.element_level(e, *d) == level).collect();
        let connectors = self.connectors_at(level);
        let paths: Vec<&Element> = if level == Level::Pa { self.graph.of_kinds(&["PartUsage"]).filter(|e| !self.is_library(e) && Self::has_metadata(e, &["Path"])).collect() } else { Vec::new() };
        if parts.is_empty() && actions.is_empty() && chains.is_empty() && use_cases.is_empty() && connectors.is_empty() && paths.is_empty() {
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
                    let mut performed: Vec<usize> = self.nodes[*part].children.iter().copied().filter(|&c| self.nodes[c].shape == Shape::Action).collect();
                    for (performer, action) in &self.performances {
                        if self.node_for(performer) == Some(*part) {
                            if let Some(a) = self.node_for(action) {
                                if !performed.contains(&a) { performed.push(a); }
                            }
                        }
                    }
                    for action in &performed {
                        self.function(1, *action, "function");
                    }
                    self.system_part(*part, &performed);
                }
            }
            // Outside the profile, functions first: a function written in
            // the part performing it names its parent, written already.
            Level::La if !self.profiled => {
                for action in &actions {
                    self.function(1, *action, "function");
                }
                for part in &parts {
                    self.logical_component(1, *part);
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
        for path in &paths {
            self.physical_path(path);
        }
        self.line(0, "}");
        self.line(0, "");
    }

    /// `#Path part`: a physical path over the links it involves.
    fn physical_path(&mut self, path: &Element) {
        let id = self.remember(&path.id, Self::identity_id(path), "PhysicalPath");
        let mut links = Vec::new();
        for (key, target) in self.dependency_attrs.get(&path.id).cloned().unwrap_or_default() {
            if key == "involves" {
                match self.intended_id(&target) {
                    Some(written) if !links.contains(&written) => links.push(written),
                    Some(_) => {}
                    None => self.count("references by dependency to elements not written (left out)"),
                }
            }
        }
        if links.is_empty() {
            self.count("physical paths without links (left out)");
            return;
        }
        self.count("physical paths");
        self.line(1, &format!("physical_path {} {{", string_literal(&Self::display(path))));
        self.line(2, &format!("id: {}", string_literal(&id)));
        if let Some(doc) = Self::description(path) {
            self.line(2, &format!("description: {}", string_literal(&doc)));
        }
        self.line(2, &format!("involves: {}", list_literal(&links)));
        self.attributes(2, &[path], &["description", "involves"]);
        self.reference_attributes_except(2, &[path.id.clone()], &["involves"]);
        self.line(1, "}");
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

    /// The ports of an element with their direction, `None` when the port
    /// states none (a physical port on a node, a component port read as `in`).
    fn ports_of(&self, element: &Element) -> Vec<(&'g Element, Option<&'static str>)> {
        element
            .children
            .iter()
            .filter_map(|c| self.graph.get(c))
            .filter_map(|c| {
                if c.kind == "PortUsage" {
                    let direction = c.direction.as_deref().map(|d| match d { "out" => "out", "inout" => "inout", _ => "in" });
                    let by_type = c.types.first().and_then(|t| self.graph.get(t)).and_then(|t| t.name.as_deref()).and_then(|n| {
                        let lower = n.to_ascii_lowercase();
                        if lower.contains("inout") { Some("inout") } else if lower.contains("out") { Some("out") } else if lower.contains("in") { Some("in") } else { None }
                    });
                    let tagged = if Self::has_metadata(c, &["PhysicalPort"]) { None } else if Self::has_metadata(c, &["ComponentPort"]) { Some("in") } else { direction.or(by_type) };
                    // A SysML port states no direction: both ways.
                    let tagged = if self.profiled { tagged } else { tagged.or(Some("inout")) };
                    Some((c, tagged))
                } else if matches!(c.kind.as_str(), "ItemUsage" | "ReferenceUsage" | "AttributeUsage") && c.direction.is_some() && !c.is_end {
                    Some((c, Some(match c.direction.as_deref() { Some("out") => "out", Some("inout") => "inout", _ => "in" })))
                } else if self.is_accepted_payload(c) {
                    // What an action accepts is what it hands on.
                    Some((c, Some("out")))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Write the ports of a node: oriented component ports, and on a node
    /// (`physical`) the ports without a direction as physical ports.
    fn write_ports(&mut self, depth: usize, index: usize, physical: bool) {
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
                if physical && direction.is_none() {
                    self.ids.insert(port.id.clone(), format!("{owner_id}.{name}"));
                    self.count("ports");
                    let id_text = port.short_name.as_deref().filter(|s| *s != name).map(|s| format!(" {{ id: {} }}", string_literal(s))).unwrap_or_default();
                    self.line(depth, &format!("port {}{id_text}", string_literal(&name)));
                    continue;
                }
                self.component_port(depth, port, direction, &owner_id, 0);
            }
        }
    }

    /// A component port and the ports it holds (its own, and those of its
    /// port definition), each known as `Owner.port.sub`.
    fn component_port(&mut self, depth: usize, port: &'g Element, direction: Option<&'static str>, holder: &str, nesting: usize) {
        let name = port.name.clone().unwrap_or_else(|| port.id.clone());
        let path = format!("{holder}.{name}");
        self.ids.insert(port.id.clone(), path.clone());
        self.count("ports");
        let head = format!("port {} {}", direction.unwrap_or("in"), string_literal(&name));
        // The port's short name is its id, when it differs from its name.
        let id_line = port.short_name.as_deref().filter(|s| *s != name).map(|s| format!("id: {}", string_literal(s)));
        let mut subs: Vec<(&'g Element, Option<&'static str>)> = self.ports_of(port).into_iter().filter(|(p, _)| p.kind == "PortUsage").collect();
        // Typed by an ArcLang type (a port definition): `is:` it, and the
        // ports the type gives are not written again.
        let typed: Option<String> = port.types.iter().find_map(|t| self.types.get(t)).cloned();
        for def in port.types.iter().filter(|t| !self.types.contains_key(*t)).filter_map(|t| self.graph.get(t)).filter(|d| d.kind == "PortDefinition") {
            for (sub, sub_direction) in self.ports_of(def).into_iter().filter(|(p, _)| p.kind == "PortUsage") {
                if !subs.iter().any(|(s, _)| s.name == sub.name) {
                    subs.push((sub, sub_direction));
                }
            }
        }
        let is_line = typed.map(|t| format!("is: {}", string_literal(&t)));
        if subs.is_empty() || nesting > 8 {
            let lines: Vec<String> = id_line.into_iter().chain(is_line).collect();
            let body = if lines.is_empty() { String::new() } else { format!(" {{ {} }}", lines.join(" ")) };
            self.line(depth, &format!("{head}{body}"));
            return;
        }
        self.line(depth, &format!("{head} {{"));
        for line in id_line.into_iter().chain(is_line) {
            self.line(depth + 1, &line);
        }
        for (sub, sub_direction) in subs {
            self.component_port(depth + 1, sub, sub_direction, &path, nesting + 1);
        }
        self.line(depth, "}");
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
        let mut tags: Vec<String> = Vec::new();
        for tag in sources.iter().flat_map(|s| s.metadata.iter()).filter(|m| self.tags.contains(*m)) {
            if !tags.contains(tag) {
                tags.push(tag.clone());
            }
        }
        if !tags.is_empty() {
            self.line(depth, &format!("tags: {}", list_literal(&tags)));
        }
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
        if let Some(parent) = self.nodes[index].decomposes.and_then(|p| self.ids.get(&self.node_key(p)).cloned()) {
            self.line(depth + 1, &format!("parent: {}", string_literal(&parent)));
        }
        self.write_ports(depth + 1, index, false);
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
                Shape::Action if role == Role::Actor => {} // written at the level
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
        self.write_ports(2, index, false);
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
        self.variability(depth + 1, index);
        self.write_ports(depth + 1, index, false);
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
        self.write_ports(3, index, false);
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
        // The steps, then what a `dependency involves` adds (the exchanges of the chain).
        let mut steps = self.chain_steps(chain);
        for (key, target) in self.dependency_attrs.get(&chain.id).cloned().unwrap_or_default() {
            if key == "involves" {
                match self.intended_id(&target) {
                    Some(written) if !steps.contains(&written) => steps.push(written),
                    Some(_) => {}
                    None => self.count("references by dependency to elements not written (left out)"),
                }
            }
        }
        self.count("functional chains");
        self.line(1, &format!("{keyword} {} {{", string_literal(&Self::display(chain))));
        self.line(2, &format!("id: {}", string_literal(&id)));
        if let Some(doc) = Self::description(chain) {
            self.line(2, &format!("description: {}", string_literal(&doc)));
        }
        if !steps.is_empty() {
            self.line(2, &format!("involves: {}", list_literal(&steps)));
        }
        self.attributes(2, &[chain], &["description", "involves"]);
        self.reference_attributes_except(2, &[chain.id.clone()], &["involves"]);
        self.chain_controls(chain);
        self.line(1, "}");
    }

    /// The control nodes of a chain (`fork`, `join`, `decide`, `merge`) and
    /// its identified successions as `control` and `sequence` lines.
    fn chain_controls(&mut self, chain: &Element) {
        let children: Vec<&'g Element> = chain.children.iter().filter_map(|c| self.graph.get(c)).collect();
        let mut control_names: HashMap<String, String> = HashMap::new();
        for node in children.iter().filter(|c| matches!(c.kind.as_str(), "ForkNode" | "JoinNode" | "DecisionNode" | "MergeNode")) {
            let kind = if Self::has_metadata(node, &["Iterate"]) {
                "iterate"
            } else if matches!(node.kind.as_str(), "ForkNode" | "JoinNode") {
                "and"
            } else {
                "or"
            };
            let name = Self::display(node);
            control_names.insert(node.id.clone(), name.clone());
            self.count("control nodes");
            match &node.short_name {
                Some(id) => self.line(2, &format!("control {kind} {} {{ id: {} }}", string_literal(&name), string_literal(id))),
                None => self.line(2, &format!("control {kind} {}", string_literal(&name))),
            }
        }
        // A step of the chain refers to a function; a control node is named.
        let end_text = |mapper: &Self, end: &str| -> Option<String> {
            if let Some(name) = control_names.get(end) {
                return Some(name.clone());
            }
            let element = mapper.graph.get(end)?;
            if element.kind == "ActionUsage" && element.owner.as_deref() == Some(chain.id.as_str()) {
                let target = element.types.first().or(element.subsets.first()).or(element.redefines.first())?;
                let node = mapper.end_node(target)?;
                return mapper.ids.get(&mapper.node_key(node)).cloned();
            }
            mapper.end_ref(end)
        };
        for link in children.iter().filter(|c| matches!(c.kind.as_str(), "SuccessionAsUsage" | "Succession") && c.short_name.is_some()) {
            let (Some(source), Some(target)) = (link.sources.first(), link.targets.first()) else { continue };
            let (Some(from), Some(to)) = (end_text(self, source), end_text(self, target)) else {
                self.count("sequence links to elements not written (left out)");
                continue;
            };
            let condition = link.children.iter().filter_map(|c| self.graph.get(c)).find(|c| c.kind == "AttributeUsage" && c.name.as_deref() == Some("condition")).and_then(|c| c.value.clone());
            let id = link.short_name.clone().unwrap_or_default();
            self.count("sequence links");
            match condition {
                Some(condition) => self.line(2, &format!("sequence {} -> {} {{ id: {} condition: {condition} }}", string_literal(&from), string_literal(&to), string_literal(&id))),
                None => self.line(2, &format!("sequence {} -> {} {{ id: {} }}", string_literal(&from), string_literal(&to), string_literal(&id))),
            }
        }
    }

    // --- exchanges --------------------------------------------------------

    /// Connectors whose source lands at the level, outside scenarios.
    fn connectors_at(&self, level: Level) -> Vec<&'g Element> {
        self.graph
            .of_kinds(CONNECTOR_KINDS)
            .filter(|c| !self.is_library(c) && !self.inside_occurrence(c) && (self.profiled || !self.is_message(c)))
            .filter(|c| self.connector_anchor(c).is_some_and(|n| self.nodes[n].level == level))
            .collect()
    }

    fn inside_occurrence(&self, element: &Element) -> bool {
        self.graph.ancestor(element.owner.as_deref().unwrap_or(""), |e| e.kind == "OccurrenceDefinition" || e.kind == "OccurrenceUsage" || e.kind == "InteractionDefinition").is_some()
    }

    fn exchange(&mut self, level: Level, connector: &Element) {
        let (Some(source), Some(target)) = (connector.sources.first().cloned(), connector.targets.first().cloned()) else {
            self.connector_lost(connector, "connectors without two ends (left out)");
            return;
        };
        // `connect (a, b, c)`: every end; the first two are `from` and `to`.
        let every_end = self.every_end(connector);
        let written = every_end
            .as_ref()
            .map(|ends| (ends[0].clone(), ends[1].clone()))
            .or_else(|| self.written_ends(connector))
            .or_else(|| Some((self.end_ref(&source)?, self.end_ref(&target)?)))
            // A port a port holds is known through the path written.
            .or_else(|| self.ends_as_written(connector))
            .or_else(|| self.flow_port_ends(connector).map(|((from, _), (to, _))| (from, to)));
        let Some((from, to)) = written else {
            self.connector_lost(connector, "connectors to elements not written (left out)");
            return;
        };
        let shape_of = |mapper: &Self, end: &str| mapper.end_node(end).map(|n| mapper.nodes[n].shape);
        let (source_shape, target_shape) = (shape_of(self, &source), shape_of(self, &target));
        // Between functions, an exchange is functional; at the operational
        // level an interaction joins activities and entities alike.
        let functional = match level {
            Level::Oa => source_shape == Some(Shape::Action) || target_shape == Some(Shape::Action),
            _ => source_shape == Some(Shape::Action) && target_shape == Some(Shape::Action),
        };
        let (name, wanted) = if self.profiled {
            (Self::display(connector), Self::identity_id(connector))
        } else {
            let name = self.connector_name(connector, &from, &to);
            (name.clone(), connector.short_name.clone().unwrap_or(name))
        };
        let id = self.remember(&connector.id, wanted, if functional { "FunctionalExchange" } else { "ComponentExchange" });
        let mut items: Vec<String> = connector.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "ItemUsage" || c.kind == "PayloadFeature").filter_map(|c| c.types.first()).filter_map(|t| self.ids.get(t).cloned()).collect();
        if items.is_empty() {
            items = connector.types.iter().filter_map(|t| self.ids.get(t).cloned()).collect();
        }
        if items.is_empty() {
            items = self.flow_port_items(connector);
        }
        let is_link = Self::has_metadata(connector, &["Link", "PhysicalLink"]) || (connector.kind == "InterfaceUsage" && level == Level::Pa);
        let is_interface = Self::has_metadata(connector, &["Interface"]);
        // The keyword, the end keys, whether the ends are ports or whole
        // elements, and whether exchange items are carried.
        let (keyword, counted, end_keys, whole_ends, carries) = match (level, functional) {
            (Level::Oa, true) => ("interaction", "operational interactions", ("from", "to"), false, true),
            (Level::Oa, false) => ("communication_means", "communication means", ("from", "to"), true, false),
            (_, true) => ("functional_exchange", "functional exchanges", ("from", "to"), false, true),
            (Level::Pa, false) if is_link => ("physical_link", "physical links", ("from", "to"), false, false),
            (Level::Pa, false) => ("physical_exchange", "physical exchanges", ("from", "to"), false, true),
            (Level::La, false) if is_interface => ("interface", "logical interfaces", ("from", "to"), true, false),
            // Ends are ports (`Component.port`) or whole components.
            (_, false) => ("component_exchange", "component exchanges", ("from_port", "to_port"), false, true),
        };
        self.count(counted);
        let (from, to) = if whole_ends { (strip_port(&from), strip_port(&to)) } else { (from, to) };
        self.line(1, &format!("{keyword} {} {{", string_literal(&name)));
        self.line(2, &format!("id: {}", string_literal(&id)));
        // Only a component exchange takes more than two ends in ArcLang.
        if every_end.is_some() && keyword != "component_exchange" {
            self.count("ends beyond the second of a connection that is not a component exchange (left out)");
        }
        match every_end.filter(|_| keyword == "component_exchange") {
            Some(ends) => self.line(2, &format!("ends: {}", list_literal(&ends))),
            None => {
                self.line(2, &format!("{}: {}", end_keys.0, string_literal(&from)));
                self.line(2, &format!("{}: {}", end_keys.1, string_literal(&to)));
            }
        }
        if carries && !items.is_empty() {
            self.line(2, &format!("exchange_items: {}", list_literal(&items)));
        }
        // A connector typed by an ArcLang type `is:` it.
        if let Some(ty) = connector.types.iter().find_map(|t| self.types.get(t)).cloned() {
            self.line(2, &format!("is: {}", string_literal(&ty)));
        }
        self.attributes(2, &[connector], &["from", "to", "from_port", "to_port", "exchange_items", "ends"]);
        self.line(1, "}");
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
            .filter(|e| self.graph.ancestor(e.owner.as_deref().unwrap_or(""), |a| matches!(a.kind.as_str(), "StateDefinition" | "StateUsage" | "ExhibitStateUsage")).is_none())
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
                self.state(1, state, &mut state_names);
            }
            let initial = states.iter().find(|s| s.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case("start") || n.eq_ignore_ascii_case("initial")));
            // `entry; then Idle;`: a succession from the (unnamed) entry action
            // to the first state; else a transition from nowhere.
            let is_entry_action = |id: &str| self.graph.get(id).is_some_and(|e| e.kind == "ActionUsage" && e.name.is_none() && e.owner.as_deref() == Some(machine.id.as_str()));
            let entry = machine
                .children
                .iter()
                .filter_map(|c| self.graph.get(c))
                .find(|c| {
                    (c.kind == "TransitionUsage" && c.sources.is_empty()) || (matches!(c.kind.as_str(), "SuccessionAsUsage" | "Succession" | "TransitionUsage") && c.sources.first().is_some_and(|s| is_entry_action(s)))
                })
                .and_then(|t| t.targets.first())
                .and_then(|t| state_names.get(t));
            if let Some(initial) = entry.cloned().or_else(|| initial.and_then(|i| state_names.get(&i.id).cloned())).or_else(|| states.first().and_then(|s| state_names.get(&s.id).cloned())) {
                self.line(1, &format!("initial: {}", string_literal(&initial)));
            }
            // The transition from the entry action is the initial state, written above.
            let entry_transition = self.entry_of(machine, &state_names).and_then(|(_, t)| t);
            let transitions: Vec<&Element> = machine.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "TransitionUsage" && Some(&c.id) != entry_transition.as_ref()).collect();
            for transition in transitions {
                self.transition(1, transition, &state_names);
            }
            self.line(0, "}");
            self.line(0, "");
        }
    }

    /// A state, with the states and transitions nested in it (a region).
    fn state(&mut self, depth: usize, state: &Element, state_names: &mut HashMap<String, String>) {
        let name = Self::display(state);
        state_names.insert(state.id.clone(), name.clone());
        let state_id = self.remember(&state.id, Self::identity_id(state), "State");
        self.count("states");
        let kind = if Self::has_metadata(state, &["Mode"]) { "mode" } else { "state" };
        let nested: Vec<&Element> = state.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "StateUsage").collect();
        let transitions: Vec<&Element> = state.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "TransitionUsage").collect();
        if nested.is_empty() && transitions.is_empty() {
            self.line(depth, &format!("{kind} {} {{ id: {} }}", string_literal(&name), string_literal(&state_id)));
            return;
        }
        self.line(depth, &format!("{kind} {} {{", string_literal(&name)));
        self.line(depth + 1, &format!("id: {}", string_literal(&state_id)));
        for inner in &nested {
            self.state(depth + 1, inner, state_names);
        }
        // The region's initial state: where its entry goes.
        let entry = self.entry_of(state, state_names);
        if let Some((initial, _)) = &entry {
            self.line(depth + 1, &format!("initial: {}", string_literal(initial)));
        }
        let entry_transition = entry.and_then(|(_, t)| t);
        for transition in transitions.into_iter().filter(|t| Some(&t.id) != entry_transition.as_ref()) {
            self.transition(depth + 1, transition, state_names);
        }
        self.line(depth, "}");
    }

    fn transition(&mut self, depth: usize, transition: &Element, state_names: &HashMap<String, String>) {
        let (Some(from), Some(to)) = (transition.sources.first().and_then(|s| state_names.get(s)).cloned(), transition.targets.first().and_then(|t| state_names.get(t)).cloned()) else {
            self.count("transitions to states not written (left out)");
            return;
        };
        let mut attributes = Vec::new();
        if let Some(short) = &transition.short_name { attributes.push(format!("id: {}", string_literal(short))); }
        if let Some(trigger) = &transition.trigger { attributes.push(format!("trigger: {}", string_literal(trigger))); }
        if let Some(guard) = &transition.guard { attributes.push(format!("guard: {}", string_literal(guard))); }
        if let Some(effect) = &transition.effect { attributes.push(format!("action: {}", string_literal(effect))); }
        self.count("transitions");
        self.line(depth, &format!("transition {} -> {} {{ {} }}", string_literal(&from), string_literal(&to), attributes.join(" ")));
    }

    /// Where the entry of a machine or region goes (`entry; then off;`, or
    /// `entry action initial; transition initial then off;`): the state,
    /// and the transition that says it when it is one.
    fn entry_of(&self, container: &Element, state_names: &HashMap<String, String>) -> Option<(String, Option<String>)> {
        let children: Vec<&Element> = container.children.iter().filter_map(|c| self.graph.get(c)).collect();
        // `entry;` is an action usage, `entry action initial;` a performed one.
        let is_entry = |id: &str| children.iter().any(|c| c.id == id && matches!(c.kind.as_str(), "ActionUsage" | "PerformActionUsage"));
        children
            .iter()
            .filter(|c| matches!(c.kind.as_str(), "SuccessionAsUsage" | "Succession" | "TransitionUsage"))
            .find(|c| c.sources.first().is_some_and(|s| is_entry(s)))
            .and_then(|t| Some((state_names.get(t.targets.first()?)?.clone(), (t.kind == "TransitionUsage").then(|| t.id.clone()))))
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
                    let node = c.types.iter().chain(c.subsets.iter()).find_map(|t| self.end_node(t))?;
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
            // State invariants of the lifelines: spans over the messages, by index.
            let mut invariants = self.invariants(&lifelines);
            invariants.sort_by(|a, b| a.start.cmp(&b.start).then(b.end.cmp(&a.end)));
            let mut open: Vec<usize> = Vec::new();
            let mut depth = 1;
            let mut next_invariant = 0;
            for (index, message) in messages.into_iter().enumerate() {
                while next_invariant < invariants.len() && invariants[next_invariant].start <= index {
                    let invariant = &invariants[next_invariant];
                    next_invariant += 1;
                    self.count("state invariants");
                    let head = format!("{} {} {} {{", invariant.keyword, string_literal(&invariant.lifeline), string_literal(&invariant.name));
                    let id_line = format!("id: {}", string_literal(&invariant.id));
                    self.line(depth, &head);
                    self.line(depth + 1, &id_line);
                    match invariant.end {
                        // An instant: a block holding no message.
                        None => self.line(depth, "}"),
                        Some(end) => {
                            open.push(end);
                            depth += 1;
                        }
                    }
                }
                let (from, to) = (message.sources.first().and_then(|s| self.event_lifeline(s, 0)), message.targets.first().and_then(|t| self.event_lifeline(t, 0)));
                match (from.as_deref().and_then(lifeline_of), to.as_deref().and_then(lifeline_of)) {
                    (Some(from), Some(to)) => {
                        let label = Self::description(message).unwrap_or_else(|| Self::display(message));
                        self.count("messages");
                        self.line(depth, &format!("message {} -> {} {}", string_literal(&from), string_literal(&to), string_literal(&label)));
                    }
                    _ => self.count("messages of lifelines not written (left out)"),
                }
                while open.last().is_some_and(|end| *end <= index) {
                    open.pop();
                    depth -= 1;
                    self.line(depth, "}");
                }
            }
            // Invariants after the last message, and spans the messages did not close.
            for invariant in invariants.iter().skip(next_invariant) {
                self.count("state invariants");
                self.line(depth, &format!("{} {} {} {{ id: {} }}", invariant.keyword, string_literal(&invariant.lifeline), string_literal(&invariant.name), string_literal(&invariant.id)));
            }
            while open.pop().is_some() {
                depth -= 1;
                self.line(depth, "}");
            }
            self.line(0, "}");
            self.line(0, "");
        }
    }

    /// The `#Invariant` states and functions of the lifelines: what state a
    /// lifeline is in over a span of messages (`start`, `endAt` indexes;
    /// `end` is read too for texts written by hand).
    fn invariants(&self, lifelines: &[(&Element, String)]) -> Vec<Invariant> {
        let mut out = Vec::new();
        for (lifeline, lifeline_id) in lifelines {
            for child in lifeline.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| Self::has_metadata(c, &["Invariant"])) {
                let target = child.subsets.first().or(child.types.first()).or(child.redefines.first());
                let referenced = target.and_then(|t| self.graph.get(t)).map(|t| match t.chaining.last().and_then(|l| self.graph.get(l)) {
                    Some(last) => last,
                    None => t,
                });
                let attribute = |key: &str| child.children.iter().filter_map(|c| self.graph.get(c)).find(|c| c.kind == "AttributeUsage" && c.name.as_deref() == Some(key)).and_then(|c| c.value.clone());
                let name = match referenced {
                    Some(state) => state.short_name.clone().or_else(|| state.name.clone()).unwrap_or_else(|| state.id.clone()),
                    None => match attribute("target").as_deref().and_then(unquote) {
                        Some(text) => text,
                        None => continue,
                    },
                };
                let number = |key: &str| attribute(key).and_then(|v| v.parse::<usize>().ok());
                let keyword = if child.kind == "ActionUsage" || child.kind == "PerformActionUsage" {
                    "function"
                } else if referenced.is_some_and(|s| Self::has_metadata(s, &["Mode"])) {
                    "mode"
                } else {
                    "state"
                };
                out.push(Invariant { lifeline: lifeline_id.clone(), name, keyword, id: Self::identity_id(child), start: number("start").unwrap_or(0), end: number("endAt").or_else(|| number("end")) });
            }
        }
        out
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

    /// The constraints a verification case asserts (`assert 'CST-1';`): an analysis.
    fn asserted_constraints(&self, sources: &[&'g Element]) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for assertion in sources.iter().flat_map(|s| s.children.iter().filter_map(|c| self.graph.get(c))).filter(|c| c.kind == "AssertConstraintUsage") {
            let Some(target) = assertion.subsets.iter().chain(assertion.redefines.iter()).chain(assertion.types.iter()).find_map(|t| self.graph.get(t)).filter(|t| t.kind.ends_with("ConstraintUsage")) else { continue };
            let id = self.ids.get(&target.id).cloned().unwrap_or_else(|| Self::identity_id(target));
            if !out.contains(&id) {
                out.push(id);
            }
        }
        out
    }

    /// An assertion inside a verification case refers to a constraint; it is not one.
    fn is_case_assertion(&self, constraint: &Element) -> bool {
        constraint.kind == "AssertConstraintUsage"
            && constraint.owner.as_deref().and_then(|o| self.graph.get(o)).is_some_and(|o| o.kind.starts_with("VerificationCase") || o.kind.starts_with("AnalysisCase"))
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
            let asserts = self.asserted_constraints(&sources);
            if !asserts.is_empty() {
                self.line(1, &format!("asserts: {}", list_literal(&asserts)));
            }
            self.line(0, "}");
            self.line(0, "");
        }
    }

    // --- traces -----------------------------------------------------------

    /// The written id an end of a relationship refers to (an element, not a port).
    /// The port an end is, when it is one: the element, or the last feature of its chain.
    fn port_end(&self, id: &str) -> Option<&'g Element> {
        let element = self.graph.get(id)?;
        let end = match element.chaining.last() {
            Some(last) => self.graph.get(last)?,
            None => element,
        };
        self.is_port_like(end).then_some(end)
    }

    fn element_ref(&self, id: &str) -> Option<String> {
        // A port is referred to as `Owner.port`, as ArcLang names it.
        if let Some(port) = self.port_end(id) {
            return self.ids.get(&port.id).cloned();
        }
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
                    let between_ports = self.port_end(source).is_some() || self.port_end(target).is_some();
                    if !between_ports && from_kind == Some("LogicalComponent") && to_kind == Some("PhysicalNode") {
                        continue; // written as `deploys` in the node
                    }
                    "allocates".to_string()
                }
                _ => relation.name.clone().unwrap_or_else(|| "refines".to_string()).to_ascii_lowercase(),
            };
            // A dependency under a reference key is the source's attribute
            // (`realizes: ...`), unless it is marked as a trace.
            if relation.kind == "Dependency" && REFERENCE_KEYS.contains(&kind.as_str()) && !TRACE_KINDS.contains(&kind.as_str()) && !Self::has_metadata(relation, &["Trace"]) {
                continue;
            }
            if !seen.insert((from.clone(), kind.clone(), to.clone())) {
                continue;
            }
            self.count("traces");
            // An allocation typed by an allocation definition `is:` it.
            let typed = (relation.kind == "AllocationUsage").then(|| relation.types.iter().find_map(|t| self.types.get(t)).cloned()).flatten();
            if let Some(definition) = typed {
                lines.push(format!("trace {} allocates {} {{ is: {} }}", string_literal(&from), string_literal(&to), string_literal(&definition)));
            } else if TRACE_KINDS.contains(&kind.as_str()) && kind != "allocates" {
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
    "exchange_items", "exchange_item", "categories", "inputs", "outputs", "members", "applies_to", "data_type", "is", "trigger", "capability", "selects",
];

/// Reference keys that take a list.
const LIST_KEYS: &[&str] = &["selects", "involves", "mitigated_by", "allocated_functions", "exploits", "includes", "extends", "verifies", "members", "applies_to", "exchange_items", "inputs", "outputs", "categories"];
/// Reference keys that take one element.
const SINGLE_KEYS: &[&str] = &["mission", "parent", "owner", "via", "allocated_component", "realizes", "capability", "data_type", "exchange_item", "from", "to", "from_port", "to_port", "is", "trigger"];
/// Dependency names that are traces, not attributes.
const TRACE_KINDS: &[&str] = &["satisfies", "refines", "implements", "verifies", "validates", "allocates"];

/// The text of a rendered string literal (`"a \"b\""` -> `a "b"`).
fn unquote(literal: &str) -> Option<String> {
    let inner = literal.strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                other => out.push(other),
            },
            other => out.push(other),
        }
    }
    Some(out)
}

/// A state invariant of a scenario, as written: `state lifeline name { ... }`.
struct Invariant {
    lifeline: String,
    name: String,
    keyword: &'static str,
    id: String,
    start: usize,
    end: Option<usize>,
}

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
    let is_keyword = match crate::compiler::lexer::Lexer::new(&key).tokenize() {
        Ok(tokens) => !matches!(tokens.first(), Some(crate::compiler::lexer::Token::Identifier(_))),
        Err(_) => true,
    };
    if is_keyword && !keyword_is_attribute_key(&key) { format!("sysml_{key}") } else { key }
}

/// Whether a keyword doubles as an attribute key wherever ArcLang takes
/// attributes: tried once on a probe model, remembered.
fn keyword_is_attribute_key(key: &str) -> bool {
    use std::cell::RefCell;
    use std::collections::HashMap;
    thread_local! {
        static KNOWN: RefCell<HashMap<String, bool>> = RefCell::new(HashMap::new());
    }
    if let Some(&known) = KNOWN.with(|k| k.borrow().get(key).copied()).as_ref() {
        return known;
    }
    let probe = format!(
        "model Probe {{}}\nrequirements {{ requirement \"r\" {{ {key}: 1 }} }}\n\
         system_analysis \"S\" {{ function \"f\" {{ {key}: 1 }} actor \"a\" {{ {key}: 1 }} functional_exchange \"x\" {{ from: \"f\" to: \"f\" {key}: 1 }} }}\n\
         logical_architecture \"L\" {{ component \"c\" {{ {key}: 1 function \"g\" {{ {key}: 1 }} }} }}\n\
         physical_architecture \"P\" {{ node \"n\" {{ {key}: 1 }} physical_link \"l\" {{ from: \"n\" to: \"n\" {key}: 1 }} }}\n"
    );
    let accepted = crate::compiler::lexer::Lexer::new(&probe)
        .tokenize()
        .ok()
        .and_then(|tokens| crate::compiler::parser::Parser::new(tokens).parse().ok())
        .is_some();
    KNOWN.with(|k| k.borrow_mut().insert(key.to_string(), accepted));
    accepted
}
