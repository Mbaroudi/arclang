//! From a SysML v2 element graph to ArcLang source: the Arcadia reading of
//! a SysML model. Levels come from the `OA`/`SA`/`LA`/`PA`/`EPBS` metadata
//! (or an `Arcadia` metadata with a `level` attribute), else from the names
//! of the enclosing packages, else from what the element is: actions are
//! system functions, parts logical components. What SysML cannot say by
//! itself is read from metadata too: `Actor`, `Entity`, `Node`,
//! `Behaviour`, `Mission`, `Capability`, `Chain`, `Link`, `Hazard`; failing
//! that, from the documentation ArcLang's own export writes (`Name (role)`).
//!
//! Every element is written with its SysML short name as id when it has
//! one, else its name: a model exported by ArcLang comes back with its ids.

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
                else if lower.contains("system analysis") || lower.starts_with("system") { Some(Level::Sa) }
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

struct Mapper<'g> {
    graph: &'g Graph,
    out: String,
    counts: BTreeMap<&'static str, usize>,
    notes: Vec<String>,
    /// Written id of every element written (definitions, usages merged into them, ports as `Owner.port`).
    ids: HashMap<String, String>,
    kinds: HashMap<String, &'static str>,
    /// Usage id -> the definition it stands for (a top-level `part p : P`).
    stands_for: HashMap<String, String>,
    /// Definition id -> the owning element (the part a `perform` or a nested usage places it in).
    placed_in: HashMap<String, String>,
    /// Ids already taken, so names stay unique where ArcLang needs it.
    taken: HashSet<String>,
    traces: Vec<(String, String, String)>,
}

pub fn map(graph: &Graph) -> Mapped {
    let mut mapper = Mapper {
        graph,
        out: String::new(),
        counts: BTreeMap::new(),
        notes: Vec::new(),
        ids: HashMap::new(),
        kinds: HashMap::new(),
        stands_for: HashMap::new(),
        placed_in: HashMap::new(),
        taken: HashSet::new(),
        traces: Vec::new(),
    };
    mapper.run();
    Mapped { source: mapper.out, counts: mapper.counts, notes: mapper.notes }
}

const DEFINITIONS: &[&str] = &[
    "PartDefinition", "ActionDefinition", "ItemDefinition", "AttributeDefinition", "EnumerationDefinition", "RequirementDefinition",
    "StateDefinition", "OccurrenceDefinition", "UseCaseDefinition", "InterfaceDefinition", "PortDefinition", "VerificationCaseDefinition",
    "ConnectionDefinition", "AllocationDefinition", "MetadataDefinition", "ConstraintDefinition", "CalculationDefinition", "FlowDefinition",
    "AnalysisCaseDefinition", "ConcernDefinition", "ViewDefinition", "ViewpointDefinition", "RenderingDefinition", "CaseDefinition",
];

impl<'g> Mapper<'g> {
    fn count(&mut self, what: &'static str) {
        *self.counts.entry(what).or_insert(0) += 1;
    }
    fn line(&mut self, depth: usize, text: &str) {
        for _ in 0..depth {
            self.out.push_str("    ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    // --- names, ids, roles, levels ---------------------------------------

    /// The documentation ArcLang's export writes: `Name (role)`.
    fn exported_doc(element: &Element) -> Option<(String, String)> {
        let doc = element.docs.first()?;
        let open = doc.rfind(" (")?;
        let role = doc[open + 2..].strip_suffix(')')?;
        let known = ["actor", "operational entity", "system component", "logical component", "physical node", "behaviour component", "hardware component", "entity", "node", "function", "activity"];
        if !known.contains(&role) {
            return None;
        }
        Some((doc[..open].to_string(), role.to_string()))
    }

    fn display_name(element: &Element) -> String {
        if let Some((name, _)) = Self::exported_doc(element) {
            return name;
        }
        element.name.clone().or_else(|| element.short_name.clone()).unwrap_or_else(|| element.id.clone())
    }

    fn description(element: &Element) -> Option<String> {
        let docs: Vec<&String> = element.docs.iter().filter(|d| Self::exported_doc(element).map(|(n, r)| format!("{n} ({r})")).as_deref() != Some(d.as_str())).collect();
        docs.first().map(|d| d.to_string())
    }

    fn written_id(element: &Element) -> String {
        element.short_name.clone().unwrap_or_else(|| Self::display_name(element))
    }

    fn has_metadata(element: &Element, names: &[&str]) -> bool {
        element.metadata.iter().any(|m| names.iter().any(|n| m.eq_ignore_ascii_case(n)))
    }

    fn role_of(&self, element: &Element) -> Option<Role> {
        if Self::has_metadata(element, &["Actor", "ArcadiaActor"]) { return Some(Role::Actor); }
        if Self::has_metadata(element, &["Entity", "OperationalEntity"]) { return Some(Role::Entity); }
        if Self::has_metadata(element, &["Node", "PhysicalNode"]) { return Some(Role::Node); }
        if Self::has_metadata(element, &["Behaviour", "Behavior", "BehaviorComponent"]) { return Some(Role::Behaviour); }
        if Self::has_metadata(element, &["Hardware", "HardwareComponent"]) { return Some(Role::Hardware); }
        if Self::has_metadata(element, &["Component"]) { return Some(Role::Component); }
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
        // The `Actor` of the SysML standard library, when a part specializes it.
        if element.specializes.iter().chain(element.types.iter()).any(|t| self.graph.get(t).and_then(|e| e.name.as_deref()) == Some("Actor")) {
            return Some(Role::Actor);
        }
        None
    }

    /// The Arcadia level of an element, when stated or implied by its packages.
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
            if owner.kind == "Package" {
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

    /// The level an element lands in, with the default of its kind.
    fn level_of(&self, element: &Element) -> Level {
        if let Some(level) = self.stated_level(element) {
            return level;
        }
        // An element placed inside another takes its level.
        if let Some(holder) = self.placed_in.get(&element.id).and_then(|h| self.graph.get(h)) {
            if let Some(level) = self.stated_level(holder) {
                return level;
            }
        }
        match element.kind.as_str() {
            "ActionDefinition" | "ActionUsage" | "UseCaseDefinition" => Level::Sa,
            "PartDefinition" | "PartUsage" => match self.role_of(element) {
                Some(Role::Node) | Some(Role::Behaviour) | Some(Role::Hardware) => Level::Pa,
                Some(Role::Entity) => Level::Oa,
                _ => Level::La,
            },
            _ => Level::La,
        }
    }

    // --- the model --------------------------------------------------------

    /// Elements that stand on their own: definitions, and usages nobody
    /// types by a definition written elsewhere (a `part engine` of its own).
    fn is_standalone(&self, element: &Element) -> bool {
        if DEFINITIONS.contains(&element.kind.as_str()) {
            return true;
        }
        false
    }

    fn collect_usages(&mut self) {
        // A top-level usage typed by exactly one definition stands for it.
        for element in &self.graph.elements {
            if !element.kind.ends_with("Usage") || element.is_end {
                continue;
            }
            let Some(def) = element.types.first() else { continue };
            if element.types.len() != 1 || !self.graph.get(def).is_some_and(|d| DEFINITIONS.contains(&d.kind.as_str())) {
                continue;
            }
            let owner_is_package = element.owner.as_deref().map(|o| self.graph.kind_of(o)) == Some("Package") || element.owner.is_none();
            if owner_is_package {
                self.stands_for.insert(element.id.clone(), def.clone());
            } else if let Some(owner) = &element.owner {
                // A part usage or a performed action nested in a part places
                // its definition there (a behaviour on a node, a performed
                // function). A step of a chain, a lifeline of a scenario or
                // the subject of a requirement only refers to it.
                let owner_def = self.definition_of(owner);
                let places = self.graph.kind_of(&owner_def) == "PartDefinition"
                    && matches!(element.kind.as_str(), "PartUsage" | "PerformActionUsage" | "ActionUsage");
                self.stands_for.insert(element.id.clone(), def.clone());
                if places && owner_def != *def {
                    self.placed_in.entry(def.clone()).or_insert(owner_def);
                }
            }
        }
    }

    /// The definition an id refers to: itself, or what its usage stands for.
    fn definition_of(&self, id: &str) -> String {
        self.stands_for.get(id).cloned().unwrap_or_else(|| id.to_string())
    }

    /// The written reference for an end of a connector: an element id, or
    /// `Owner.port` for a port or a directed parameter.
    fn end_ref(&self, id: &str) -> Option<String> {
        let element = self.graph.get(id)?;
        if element.kind == "PortUsage" || element.kind == "ItemUsage" || element.kind == "ReferenceUsage" {
            let owner = element.owner.as_deref()?;
            let owner_def = self.definition_of(owner);
            if element.kind == "PortUsage" || element.direction.is_some() {
                let owner_id = self.ids.get(&owner_def)?;
                return Some(format!("{owner_id}.{}", element.name.clone()?));
            }
            // A reference end of a connector: what it subsets.
            if let Some(target) = element.subsets.first().or(element.redefines.first()) {
                return self.end_ref(target);
            }
        }
        let def = self.definition_of(id);
        self.ids.get(&def).cloned()
    }

    fn run(&mut self) {
        self.collect_usages();
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
        let enumerations: Vec<&Element> = self.graph.of_kinds(&["EnumerationDefinition"]).collect();
        for e in enumerations {
            let id = self.remember(e, "Enumeration");
            let values: Vec<String> = e.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "EnumerationUsage").filter_map(|c| c.name.clone()).collect();
            self.count("enumerations");
            self.line(0, &format!("enumeration {} {{", string_literal(&Self::display_name(e))));
            self.line(1, &format!("id: {}", string_literal(&id)));
            self.line(1, &format!("values: {}", list_literal(&values)));
            self.line(0, "}");
        }
        let data_types: Vec<&Element> = self.graph.of_kinds(&["AttributeDefinition", "DataType"]).filter(|e| !self.is_library(e)).collect();
        for e in data_types {
            let id = self.remember(e, "DataType");
            let base = e.specializes.first().and_then(|s| self.graph.get(s)).and_then(|s| s.name.clone()).unwrap_or_else(|| "String".to_string());
            self.count("data types");
            self.line(0, &format!("data_type {} {{", string_literal(&Self::display_name(e))));
            self.line(1, &format!("id: {}", string_literal(&id)));
            self.line(1, &format!("base: {}", string_literal(&base)));
            self.line(0, "}");
        }
        let items: Vec<&Element> = self.graph.of_kinds(&["ItemDefinition"]).collect();
        for e in items {
            let fields: Vec<(String, String)> = e
                .children
                .iter()
                .filter_map(|c| self.graph.get(c))
                .filter(|c| c.kind == "AttributeUsage" || c.kind == "ItemUsage")
                .filter_map(|c| Some((c.name.clone()?, c.types.first().and_then(|t| self.graph.get(t)).and_then(|t| t.name.clone()).unwrap_or_else(|| "String".to_string()))))
                .collect();
            if !fields.is_empty() || Self::has_metadata(e, &["Class"]) {
                let id = self.remember(e, "Class");
                self.count("classes");
                self.line(0, &format!("class {} {{", string_literal(&Self::display_name(e))));
                self.line(1, &format!("id: {}", string_literal(&id)));
                if let Some(parent) = e.specializes.first().and_then(|s| self.ids.get(s)).cloned() {
                    self.line(1, &format!("extends: {}", string_literal(&parent)));
                }
                for (name, ty) in fields {
                    self.line(1, &format!("{}: {}", field_key(&name), string_literal(&ty)));
                }
                self.line(0, "}");
            } else {
                let id = self.remember(e, "ExchangeItem");
                self.count("exchange items");
                self.line(0, &format!("exchange_item {} {{ id: {} }}", string_literal(&Self::display_name(e)), string_literal(&id)));
            }
        }
        if !self.out.ends_with("}\n\n") {
            self.line(0, "");
        }
    }

    fn is_library(&self, element: &Element) -> bool {
        element.is_library || self.graph.ancestor(&element.id, |e| e.kind == "LibraryPackage" || e.is_library).is_some()
    }

    // --- requirements -----------------------------------------------------

    fn requirements(&mut self) {
        let requirements: Vec<&Element> = self
            .graph
            .of_kinds(&["RequirementDefinition", "RequirementUsage"])
            .filter(|e| !e.is_end && e.owner.as_deref().map(|o| self.graph.kind_of(o)).is_none_or(|k| k == "Package" || k == "RequirementDefinition" || k == "RequirementUsage"))
            .filter(|e| !self.stands_for.contains_key(&e.id) && !self.is_library(e))
            .filter(|e| self.graph.ancestor(&e.id, |a| a.kind.starts_with("VerificationCase") || a.kind.starts_with("AnalysisCase")).is_none())
            .collect();
        let hazards: Vec<&Element> = requirements.iter().copied().filter(|e| Self::has_metadata(e, &["Hazard"]) || e.name.as_deref().is_some_and(|n| n.starts_with("hazard_"))).collect();
        let requirements: Vec<&Element> = requirements.into_iter().filter(|e| !hazards.iter().any(|h| h.id == e.id)).collect();
        if !requirements.is_empty() {
            self.line(0, "requirements {");
            for r in requirements {
                let id = self.remember(r, "Requirement");
                self.count("requirements");
                self.line(1, &format!("requirement {} {{", string_literal(&id)));
                if let Some(doc) = Self::description(r) {
                    self.line(2, &format!("description: {}", string_literal(&doc)));
                }
                if let Some(name) = r.name.clone().filter(|n| *n != id && !n.starts_with("req_")) {
                    self.line(2, &format!("title: {}", string_literal(&name)));
                }
                self.attributes(2, r, &["description"]);
                self.line(1, "}");
            }
            self.line(0, "}");
            self.line(0, "");
        }
        if !hazards.is_empty() {
            self.line(0, "safety_analysis {");
            for h in hazards {
                let id = self.remember(h, "Hazard");
                self.count("hazards");
                self.line(1, &format!("hazard {} {{", string_literal(&id)));
                if let Some(doc) = Self::description(h) {
                    self.line(2, &format!("description: {}", string_literal(&doc)));
                }
                self.attributes(2, h, &[]);
                self.line(1, "}");
            }
            self.line(0, "}");
            self.line(0, "");
        }
    }

    /// The attribute usages of an element with a bound literal, as attributes.
    fn attributes(&mut self, depth: usize, element: &Element, skip: &[&str]) {
        let mut lines = Vec::new();
        let mut references = 0;
        for child in element.children.iter().filter_map(|c| self.graph.get(c)) {
            if child.kind != "AttributeUsage" && child.kind != "ReferenceUsage" {
                continue;
            }
            let (Some(name), Some(value)) = (&child.name, &child.value) else { continue };
            let key = field_key(name);
            if skip.contains(&key.as_str()) || matches!(key.as_str(), "id" | "name") {
                continue;
            }
            // A reference ArcLang checks (what an element involves, realizes,
            // allocates...) is a relationship, not text: as an attribute it
            // is left out, and counted.
            if REFERENCE_KEYS.contains(&key.as_str()) {
                references += 1;
                continue;
            }
            lines.push(format!("{key}: {value}"));
        }
        for line in lines {
            self.line(depth, &line);
        }
        for _ in 0..references {
            self.count("reference attributes written as text (left out: a relationship is expected)");
        }
    }

    fn remember(&mut self, element: &Element, kind: &'static str) -> String {
        let mut id = Self::written_id(element);
        while !self.taken.insert(id.clone()) {
            id.push('_');
        }
        self.ids.insert(element.id.clone(), id.clone());
        self.kinds.insert(element.id.clone(), kind);
        id
    }

    // --- levels -----------------------------------------------------------

    fn parts_at(&self, level: Level) -> Vec<&'g Element> {
        self.graph
            .of_kinds(&["PartDefinition"])
            .filter(|e| !self.is_library(e) && !self.placed_in.contains_key(&e.id) && self.level_of(e) == level)
            .collect()
    }

    fn actions_at(&self, level: Level) -> Vec<&'g Element> {
        self.graph
            .of_kinds(&["ActionDefinition"])
            .filter(|e| !self.is_library(e) && !self.placed_in.contains_key(&e.id) && self.level_of(e) == level && !self.is_chain(e))
            .collect()
    }

    /// An action definition made of steps typed by other actions is a chain.
    fn is_chain(&self, action: &Element) -> bool {
        let steps = action
            .children
            .iter()
            .filter_map(|c| self.graph.get(c))
            .filter(|c| c.kind == "ActionUsage" && c.types.iter().any(|t| self.graph.kind_of(t) == "ActionDefinition"))
            .count();
        steps >= 2 || Self::has_metadata(action, &["Chain", "FunctionalChain", "Process"])
    }

    fn chain_steps(&self, action: &Element) -> Vec<String> {
        action
            .children
            .iter()
            .filter_map(|c| self.graph.get(c))
            .filter(|c| c.kind == "ActionUsage")
            .filter_map(|c| c.types.first())
            .filter_map(|t| self.ids.get(t).cloned())
            .collect()
    }

    fn level(&mut self, level: Level) {
        let parts = self.parts_at(level);
        let actions = self.actions_at(level);
        let chains: Vec<&Element> = self.graph.of_kinds(&["ActionDefinition"]).filter(|e| self.level_of(e) == level && self.is_chain(e)).collect();
        let use_cases: Vec<&Element> = self.graph.of_kinds(&["UseCaseDefinition"]).filter(|e| self.level_of(e) == level).collect();
        let connectors: Vec<&Element> = self.connectors_at(level);
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
                // Activities first, so entities can hold them.
                for action in &actions {
                    let id = self.remember(action, "OperationalActivity");
                    let _ = id;
                }
                for part in &parts {
                    self.operational_part(part);
                }
                // Activities placed in no entity stand at the level.
                for action in &actions {
                    if !self.placed_in.contains_key(&action.id) && !self.performed_by_any(action) {
                        self.function(1, action, "activity");
                    }
                }
                for connector in &connectors {
                    self.exchange(level, connector);
                }
                for use_case in &use_cases {
                    self.use_case(use_case, "operational_capability");
                }
                for chain in &chains {
                    self.chain(chain, "operational_process");
                }
            }
            Level::Sa => {
                for action in &actions {
                    self.function(1, action, "function");
                }
                for part in &parts {
                    self.system_part(part);
                }
                for connector in &connectors {
                    self.exchange(level, connector);
                }
                for use_case in &use_cases {
                    let keyword = if Self::has_metadata(use_case, &["Mission"]) { "mission" } else { "capability" };
                    self.use_case(use_case, keyword);
                }
                for chain in &chains {
                    self.chain(chain, "functional_chain");
                }
            }
            Level::La | Level::Pa => {
                for part in &parts {
                    if level == Level::La {
                        self.logical_component(1, part);
                    } else {
                        self.physical_node(part);
                    }
                }
                // Functions performed by no component stand at the level.
                for action in &actions {
                    if !self.performed_by_any(action) {
                        self.function(1, action, "function");
                    }
                }
                for connector in &connectors {
                    self.exchange(level, connector);
                }
                for use_case in &use_cases {
                    self.use_case(use_case, "capability_realization");
                }
                for chain in &chains {
                    self.chain(chain, "functional_chain");
                }
            }
            Level::Epbs => {}
        }
        self.line(0, "}");
        self.line(0, "");
    }

    /// Whether a part performs the action (a `perform action` typed by it).
    fn performers_of(&self, action: &Element) -> Vec<String> {
        self.graph
            .of_kinds(&["PerformActionUsage", "ActionUsage"])
            .filter(|u| u.types.iter().any(|t| t == &action.id) || u.subsets.iter().any(|s| self.definition_of(s) == action.id))
            .filter_map(|u| u.owner.clone())
            .map(|o| self.definition_of(&o))
            .filter(|o| self.graph.kind_of(o) == "PartDefinition")
            .collect()
    }

    fn performed_by_any(&self, action: &Element) -> bool {
        !self.performers_of(action).is_empty()
    }

    /// The actions a part performs: its `perform action` usages and the
    /// action usages nested in it, by their definitions.
    fn performed(&self, part: &Element) -> Vec<&'g Element> {
        let mut out = Vec::new();
        for child in part.children.iter().filter_map(|c| self.graph.get(c)) {
            if child.kind == "PerformActionUsage" || child.kind == "ActionUsage" {
                let target = child.types.first().or(child.subsets.first()).map(|t| self.definition_of(t));
                if let Some(action) = target.and_then(|t| self.graph.get(&t)).filter(|a| a.kind == "ActionDefinition") {
                    out.push(action);
                } else if child.kind == "ActionUsage" && child.types.is_empty() {
                    out.push(child);
                }
            }
        }
        out
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
                        if lower.contains("out") && !lower.contains("inout") { "out" } else if lower.contains("inout") { "inout" } else { "in" }
                    });
                    Some((c, direction.or(by_type).unwrap_or("in")))
                } else if (c.kind == "ItemUsage" || c.kind == "ReferenceUsage" || c.kind == "AttributeUsage") && c.direction.is_some() && element.kind == "ActionDefinition" {
                    Some((c, match c.direction.as_deref() { Some("out") => "out", Some("inout") => "inout", _ => "in" }))
                } else {
                    None
                }
            })
            .collect()
    }

    fn write_ports(&mut self, depth: usize, owner_id: &str, element: &Element, oriented: bool) {
        for (port, direction) in self.ports_of(element) {
            let name = port.name.clone().unwrap_or_else(|| port.id.clone());
            self.ids.insert(port.id.clone(), format!("{owner_id}.{name}"));
            self.count("ports");
            if oriented {
                self.line(depth, &format!("port {direction} {}", string_literal(&name)));
            } else {
                self.line(depth, &format!("port {}", string_literal(&name)));
            }
        }
    }

    fn header(&mut self, depth: usize, element: &Element, id: &str) {
        self.line(depth, &format!("id: {}", string_literal(id)));
        if let Some(doc) = Self::description(element) {
            self.line(depth, &format!("description: {}", string_literal(&doc)));
        }
        self.attributes(depth, element, &["description"]);
    }

    fn function(&mut self, depth: usize, action: &Element, keyword: &str) {
        let id = self.ids.get(&action.id).cloned().unwrap_or_else(|| self.remember(action, "Function"));
        self.count(if keyword == "activity" { "activities" } else { "functions" });
        self.line(depth, &format!("{keyword} {} {{", string_literal(&Self::display_name(action))));
        self.header(depth + 1, action, &id);
        self.write_ports(depth + 1, &id, action, true);
        // Sub-functions: action usages nested in the definition that are definitions of their own.
        let nested: Vec<&Element> = action
            .children
            .iter()
            .filter_map(|c| self.graph.get(c))
            .filter(|c| c.kind == "ActionUsage" && c.types.is_empty() && !c.is_end)
            .collect();
        for sub in nested {
            self.function(depth + 1, sub, keyword);
        }
        self.line(depth, "}");
    }

    fn operational_part(&mut self, part: &Element) {
        let role = self.role_of(part).unwrap_or(Role::Entity);
        let keyword = if role == Role::Actor { "actor" } else { "entity" };
        let id = self.remember(part, if role == Role::Actor { "Actor" } else { "Entity" });
        self.count(if role == Role::Actor { "operational actors" } else { "operational entities" });
        self.line(1, &format!("{keyword} {} {{", string_literal(&Self::display_name(part))));
        self.header(2, part, &id);
        for action in self.performed(part) {
            self.placed_in.insert(action.id.clone(), part.id.clone());
            self.function(2, action, "activity");
        }
        self.line(1, "}");
    }

    fn system_part(&mut self, part: &Element) {
        let role = self.role_of(part).unwrap_or(Role::Component);
        let keyword = if role == Role::Actor { "actor" } else { "system_component" };
        let id = self.remember(part, if role == Role::Actor { "SystemActor" } else { "SystemComponent" });
        self.count(if role == Role::Actor { "system actors" } else { "system components" });
        self.line(1, &format!("{keyword} {} {{", string_literal(&Self::display_name(part))));
        self.header(2, part, &id);
        self.write_ports(2, &id, part, true);
        let performed: Vec<String> = self.performed(part).iter().filter_map(|a| self.ids.get(&a.id).cloned()).collect();
        if !performed.is_empty() {
            self.line(2, &format!("allocated_functions: {}", list_literal(&performed)));
        }
        self.line(1, "}");
    }

    fn logical_component(&mut self, depth: usize, part: &Element) {
        let role = self.role_of(part);
        let id = self.remember(part, "LogicalComponent");
        self.count("logical components");
        self.line(depth, &format!("component {} {{", string_literal(&Self::display_name(part))));
        self.header(depth + 1, part, &id);
        if role == Some(Role::Actor) {
            self.line(depth + 1, "actor: true");
        }
        self.write_ports(depth + 1, &id, part, true);
        for action in self.performed(part) {
            self.placed_in.insert(action.id.clone(), part.id.clone());
            self.function(depth + 1, action, "function");
        }
        for child in self.nested_parts(part) {
            self.logical_component(depth + 1, child);
        }
        self.line(depth, "}");
    }

    /// Parts nested in a part: usages typed by a definition placed here, or
    /// untyped usages that are elements of their own.
    fn nested_parts(&self, part: &Element) -> Vec<&'g Element> {
        part.children
            .iter()
            .filter_map(|c| self.graph.get(c))
            .filter(|c| c.kind == "PartUsage" && !c.is_end)
            .filter_map(|c| match c.types.first() {
                Some(def) if self.placed_in.get(def) == Some(&part.id) => self.graph.get(def),
                Some(_) => None,
                None => Some(c),
            })
            .collect()
    }

    fn physical_node(&mut self, part: &Element) {
        let role = self.role_of(part);
        let id = self.remember(part, "PhysicalNode");
        self.count("physical nodes");
        self.line(1, &format!("node {} {{", string_literal(&Self::display_name(part))));
        self.header(2, part, &id);
        if role == Some(Role::Actor) {
            self.line(2, "actor: true");
        }
        self.write_ports(2, &id, part, true);
        for action in self.performed(part) {
            self.placed_in.insert(action.id.clone(), part.id.clone());
            self.function(2, action, "function");
        }
        for child in self.nested_parts(part) {
            let child_role = self.role_of(child);
            if child_role == Some(Role::Hardware) {
                let child_id = self.remember(child, "HardwareComponent");
                self.count("hardware components");
                self.line(2, &format!("hardware_component {} {{ id: {} }}", string_literal(&Self::display_name(child)), string_literal(&child_id)));
                continue;
            }
            let child_id = self.remember(child, "BehaviorComponent");
            self.count("behaviour components");
            self.line(2, &format!("behavior_component {} {{", string_literal(&Self::display_name(child))));
            self.header(3, child, &child_id);
            self.write_ports(3, &child_id, child, true);
            if let Some(realized) = self.allocated_logical(child) {
                self.line(3, &format!("allocated_component: {}", string_literal(&realized)));
            }
            for action in self.performed(child) {
                self.placed_in.insert(action.id.clone(), child.id.clone());
                self.function(3, action, "function");
            }
            self.line(2, "}");
        }
        // Logical components allocated to the node by an allocation.
        let deployed: Vec<String> = self
            .graph
            .of_kinds(&["AllocationUsage"])
            .filter(|a| a.targets.iter().any(|t| self.definition_of(t) == part.id))
            .filter_map(|a| a.sources.first())
            .map(|s| self.definition_of(s))
            .filter(|s| self.kinds.get(s) == Some(&"LogicalComponent"))
            .filter_map(|s| self.ids.get(&s).cloned())
            .collect();
        for logical in deployed {
            self.count("deployments");
            self.line(2, &format!("deploys {}", string_literal(&logical)));
        }
        self.line(1, "}");
    }

    /// The logical component a behaviour component realizes: an allocation
    /// or a `realizes` dependency from it to a written logical component.
    fn allocated_logical(&self, behaviour: &Element) -> Option<String> {
        self.graph
            .of_kinds(&["AllocationUsage", "Dependency"])
            .filter(|a| a.sources.iter().any(|s| self.definition_of(s) == behaviour.id))
            .filter_map(|a| a.targets.first())
            .map(|t| self.definition_of(t))
            .find(|t| self.kinds.get(t) == Some(&"LogicalComponent"))
            .and_then(|t| self.ids.get(&t).cloned())
    }

    fn use_case(&mut self, use_case: &Element, keyword: &str) {
        let id = self.remember(use_case, "Capability");
        self.count("capabilities and missions");
        self.line(1, &format!("{keyword} {} {{", string_literal(&Self::display_name(use_case))));
        self.header(2, use_case, &id);
        self.line(1, "}");
    }

    fn chain(&mut self, chain: &Element, keyword: &str) {
        let id = self.remember(chain, "FunctionalChain");
        let steps = self.chain_steps(chain);
        self.count("functional chains");
        self.line(1, &format!("{keyword} {} {{", string_literal(&Self::display_name(chain))));
        self.header(2, chain, &id);
        if !steps.is_empty() {
            self.line(2, &format!("involves: {}", list_literal(&steps)));
        }
        self.line(1, "}");
    }

    // --- exchanges --------------------------------------------------------

    /// Connectors whose source lands at the level: connections, interfaces
    /// and flows outside scenarios, between written elements.
    fn connectors_at(&self, level: Level) -> Vec<&'g Element> {
        self.graph
            .of_kinds(&["ConnectionUsage", "InterfaceUsage", "FlowUsage", "FlowConnectionUsage", "SuccessionFlowUsage", "BindingConnectionUsage"])
            .filter(|c| !self.inside_occurrence(c))
            .filter(|c| {
                let Some(source) = c.sources.first() else { return false };
                let holder = self.end_holder(source);
                holder.as_deref().and_then(|h| self.graph.get(h)).is_some_and(|h| self.level_of(h) == level)
            })
            .collect()
    }

    fn inside_occurrence(&self, element: &Element) -> bool {
        self.graph.ancestor(&element.id, |e| e.kind == "OccurrenceDefinition" || e.kind == "OccurrenceUsage" || e.kind == "InteractionDefinition").is_some()
    }

    /// The definition that holds an end: the part or action of a port, or the element itself.
    fn end_holder(&self, id: &str) -> Option<String> {
        let element = self.graph.get(id)?;
        if matches!(element.kind.as_str(), "PortUsage" | "ItemUsage" | "ReferenceUsage" | "AttributeUsage") {
            let owner = element.owner.as_deref()?;
            return Some(self.definition_of(owner));
        }
        Some(self.definition_of(id))
    }

    fn exchange(&mut self, level: Level, connector: &Element) {
        let (Some(source), Some(target)) = (connector.sources.first().cloned(), connector.targets.first().cloned()) else {
            self.notes.push(format!("connector '{}' has no ends: left out", Self::display_name(connector)));
            return;
        };
        let (Some(from), Some(to)) = (self.end_ref(&source), self.end_ref(&target)) else {
            self.count("connectors to elements not written (left out)");
            return;
        };
        let from_kind = self.end_holder(&source).and_then(|h| self.kinds.get(&h).copied()).unwrap_or("");
        let to_kind = self.end_holder(&target).and_then(|h| self.kinds.get(&h).copied()).unwrap_or("");
        let functional = matches!(from_kind, "Function" | "OperationalActivity") && matches!(to_kind, "Function" | "OperationalActivity");
        let name = Self::display_name(connector);
        let id = self.remember(connector, if functional { "FunctionalExchange" } else { "ComponentExchange" });
        let items: Vec<String> = connector.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "ItemUsage" || c.kind == "PayloadFeature").filter_map(|c| c.types.first()).filter_map(|t| self.ids.get(t).cloned()).collect();
        let items = if items.is_empty() { connector.types.iter().filter_map(|t| self.ids.get(t).cloned()).collect() } else { items };
        let item_text = if items.is_empty() { String::new() } else { format!(" exchange_items: {}", list_literal(&items)) };
        let is_link = Self::has_metadata(connector, &["Link", "PhysicalLink"]) || connector.kind == "InterfaceUsage" && level == Level::Pa;
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
                    self.count("component exchanges without ports (left out)");
                    self.notes.push(format!("component exchange '{name}' joins components without ports: ArcLang needs `Component.port` ends"));
                    return;
                }
                self.count("component exchanges");
                self.line(1, &format!("component_exchange {} {{ id: {} from_port: {} to_port: {}{item_text} }}", string_literal(&name), string_literal(&id), string_literal(&from), string_literal(&to)));
            }
        }
    }

    // --- EPBS -------------------------------------------------------------

    fn epbs(&mut self) {
        let items: Vec<&Element> = self.graph.of_kinds(&["PartDefinition"]).filter(|e| self.level_of(e) == Level::Epbs && !self.placed_in.contains_key(&e.id)).collect();
        if items.is_empty() {
            return;
        }
        self.line(0, "epbs \"EPBS\" {");
        for item in items {
            let id = self.remember(item, "EpbsSystem");
            self.count("configuration items");
            self.line(1, &format!("system {} {{", string_literal(&Self::display_name(item))));
            self.header(2, item, &id);
            for child in self.nested_parts(item) {
                let child_id = self.remember(child, "EpbsSubsystem");
                self.line(2, &format!("subsystem {} {{", string_literal(&Self::display_name(child))));
                self.line(3, &format!("id: {}", string_literal(&child_id)));
                for leaf in self.nested_parts(child) {
                    let leaf_id = self.remember(leaf, "EpbsItem");
                    self.line(3, &format!("item {} {{ id: {} }}", string_literal(&Self::display_name(leaf)), string_literal(&leaf_id)));
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
        let machines: Vec<&Element> = self.graph.of_kinds(&["StateDefinition"]).filter(|e| !self.is_library(e)).collect();
        for machine in machines {
            let states: Vec<&Element> = machine.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "StateUsage").collect();
            if states.is_empty() {
                continue;
            }
            let id = self.remember(machine, "StateMachine");
            self.count("state machines");
            self.line(0, &format!("state_machine {} {{", string_literal(&Self::display_name(machine))));
            self.line(1, &format!("id: {}", string_literal(&id)));
            if let Some(owner) = self.exhibited_by(machine) {
                self.line(1, &format!("owner: {}", string_literal(&owner)));
            }
            let initial = machine.children.iter().filter_map(|c| self.graph.get(c)).find(|c| c.kind == "StateUsage" && c.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case("start") || n.eq_ignore_ascii_case("initial")));
            let mut state_names: HashMap<String, String> = HashMap::new();
            for state in &states {
                let name = Self::display_name(state);
                state_names.insert(state.id.clone(), name.clone());
                let state_id = self.remember(state, "State");
                self.count("states");
                let kind = if Self::has_metadata(state, &["Mode"]) { "mode" } else { "state" };
                self.line(1, &format!("{kind} {} {{ id: {} }}", string_literal(&name), string_literal(&state_id)));
            }
            if let Some(initial) = initial.and_then(|i| state_names.get(&i.id).cloned()) {
                self.line(1, &format!("initial: {}", string_literal(&initial)));
            } else if let Some(first) = states.first().and_then(|s| state_names.get(&s.id).cloned()) {
                self.line(1, &format!("initial: {}", string_literal(&first)));
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

    /// The part or action that exhibits a state definition: its owner when
    /// written, else what an `exhibit state` typed by it belongs to.
    fn exhibited_by(&self, machine: &Element) -> Option<String> {
        if let Some(owner) = machine.owner.as_deref().map(|o| self.definition_of(o)) {
            if let Some(id) = self.ids.get(&owner) {
                return Some(id.clone());
            }
        }
        self.graph
            .of_kinds(&["ExhibitStateUsage", "StateUsage"])
            .filter(|u| u.types.iter().any(|t| t == &machine.id))
            .filter_map(|u| u.owner.as_deref().map(|o| self.definition_of(o)))
            .find_map(|o| self.ids.get(&o).cloned())
    }

    fn scenarios(&mut self) {
        let scenarios: Vec<&Element> = self.graph.of_kinds(&["OccurrenceDefinition", "InteractionDefinition"]).filter(|e| !self.is_library(e)).collect();
        for scenario in scenarios {
            let lifelines: Vec<(&Element, String)> = scenario
                .children
                .iter()
                .filter_map(|c| self.graph.get(c))
                .filter(|c| c.kind == "PartUsage" || c.kind == "OccurrenceUsage" || c.kind == "ItemUsage")
                .filter_map(|c| {
                    let def = c.types.first().map(|t| self.definition_of(t))?;
                    let id = self.ids.get(&def)?.clone();
                    Some((c, id))
                })
                .collect();
            let messages: Vec<&Element> = scenario.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| matches!(c.kind.as_str(), "FlowUsage" | "FlowConnectionUsage" | "SuccessionFlowUsage" | "MessageUsage")).collect();
            if lifelines.is_empty() || messages.is_empty() {
                continue;
            }
            let id = self.remember(scenario, "Scenario");
            self.count("scenarios");
            let participants: Vec<String> = lifelines.iter().map(|(_, id)| id.clone()).collect();
            self.line(0, &format!("scenario {} {{", string_literal(&Self::display_name(scenario))));
            self.line(1, &format!("id: {}", string_literal(&id)));
            self.line(1, &format!("participants: {}", list_literal(&participants)));
            let lifeline_of = |path: &str| -> Option<String> {
                let head = path.split('.').next()?;
                lifelines.iter().find(|(l, _)| l.name.as_deref() == Some(head)).map(|(_, id)| id.clone())
            };
            for message in messages {
                let ends = self.event_paths(message);
                let (Some(from), Some(to)) = (ends.0.as_deref().and_then(lifeline_of), ends.1.as_deref().and_then(lifeline_of)) else {
                    self.count("messages of lifelines not written (left out)");
                    continue;
                };
                let label = Self::description(message).unwrap_or_else(|| Self::display_name(message));
                self.count("messages");
                self.line(1, &format!("message {} -> {} {}", string_literal(&from), string_literal(&to), string_literal(&label)));
            }
            self.line(0, "}");
            self.line(0, "");
        }
    }

    /// The lifeline paths of a message's two events (`l_A.m1_sent`).
    fn event_paths(&self, message: &Element) -> (Option<String>, Option<String>) {
        let path_of = |id: &str| -> Option<String> {
            let event = self.graph.get(id)?;
            // An event owned by a lifeline usage: `lifeline.event`.
            let owner = event.owner.as_deref().and_then(|o| self.graph.get(o))?;
            if owner.kind == "PartUsage" || owner.kind == "OccurrenceUsage" {
                return Some(format!("{}.{}", owner.name.clone()?, event.name.clone().unwrap_or_default()));
            }
            event.name.clone()
        };
        let from = message.sources.first().and_then(|s| path_of(s));
        let to = message.targets.first().and_then(|t| path_of(t));
        (from, to)
    }

    fn test_cases(&mut self) {
        let cases: Vec<&Element> = self.graph.of_kinds(&["VerificationCaseDefinition", "VerificationCaseUsage"]).filter(|e| !self.stands_for.contains_key(&e.id)).collect();
        for case in cases {
            // The requirements verified: the objective's requirement usages
            // and what they reference, at any depth under the case.
            let mut verifies: Vec<String> = Vec::new();
            let mut stack: Vec<&Element> = case.children.iter().filter_map(|c| self.graph.get(c)).collect();
            while let Some(c) = stack.pop() {
                if c.kind == "RequirementUsage" {
                    for r in c.subsets.iter().chain(c.types.iter()).chain(c.redefines.iter()) {
                        if let Some(id) = self.ids.get(&self.definition_of(r)).cloned() {
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
            let id = self.remember(case, "TestCase");
            self.count("test cases");
            self.line(0, &format!("test_case {} {{", string_literal(&Self::display_name(case))));
            self.header(1, case, &id);
            if !verifies.is_empty() {
                self.line(1, &format!("verifies: {}", list_literal(&verifies)));
            }
            self.line(0, "}");
            self.line(0, "");
        }
    }

    // --- traces -----------------------------------------------------------

    fn traces(&mut self) {
        let mut lines: Vec<String> = Vec::new();
        let mut seen: HashSet<(String, String, String)> = HashSet::new();
        let relations: Vec<&Element> = self.graph.of_kinds(&["SatisfyRequirementUsage", "Dependency", "AllocationUsage", "VerificationCaseUsage"]).collect();
        for relation in relations {
            let (Some(source), Some(target)) = (relation.sources.first(), relation.targets.first()) else { continue };
            let (from_def, to_def) = (self.definition_of(source), self.definition_of(target));
            let (Some(from), Some(to)) = (self.ids.get(&from_def).cloned(), self.ids.get(&to_def).cloned()) else {
                self.count("relations to elements not written (left out)");
                continue;
            };
            let kind = match relation.kind.as_str() {
                "SatisfyRequirementUsage" => "satisfies".to_string(),
                "AllocationUsage" => {
                    // A logical component on a node is a deployment, written in the node.
                    if self.kinds.get(&from_def) == Some(&"LogicalComponent") && self.kinds.get(&to_def) == Some(&"PhysicalNode") {
                        continue;
                    }
                    "allocates".to_string()
                }
                _ => relation.name.clone().unwrap_or_else(|| "refines".to_string()).to_ascii_lowercase(),
            };
            if !seen.insert((from.clone(), kind.clone(), to.clone())) {
                continue;
            }
            self.count("traces");
            if matches!(kind.as_str(), "satisfies" | "realizes" | "refines" | "implements" | "verifies" | "validates" | "allocates") && kind != "allocates" {
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

fn list_literal(items: &[String]) -> String {
    format!("[{}]", items.iter().map(|i| string_literal(i)).collect::<Vec<_>>().join(", "))
}

fn strip_port(reference: &str) -> String {
    reference.split('.').next().unwrap_or(reference).to_string()
}

/// A SysML feature name as an ArcLang attribute key (identifier).
fn field_key(name: &str) -> String {
    let key: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect();
    if key.chars().next().is_some_and(|c| c.is_ascii_digit()) { format!("_{key}") } else { key }
}
