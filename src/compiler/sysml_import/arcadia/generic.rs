//! Rules for SysML v2 models written without the Arcadia profile, in the
//! style of the OMG examples: functions allocated by `perform`, connectors
//! named after what they carry, messages between lifelines wherever they
//! are written, and every connector left out counted and named.

use super::{list_literal, Level, Mapper, Shape, CONNECTOR_KINDS};
use crate::compiler::source_edit::string_literal;
use crate::compiler::sysml_import::records::Element;
use std::collections::{HashMap, HashSet};

/// How many left-out connectors the notes name.
const NAMED_LOSSES: usize = 8;

impl<'g> Mapper<'g> {
    /// A function performed by a part (`perform tree.step;`) is written in
    /// that part, as Capella allocates it; the function it decomposes stays
    /// at the level and is named by `parent:`. Only leaves are allocated:
    /// a decomposed function is where its leaves are. The first performer
    /// of a function wins.
    pub(super) fn allocate_performed_functions(&mut self) {
        let decomposed: HashSet<usize> = self.nodes.iter().filter(|n| n.shape == Shape::Action).filter_map(|n| n.parent).collect();
        let mut moved: HashSet<usize> = HashSet::new();
        let mut beyond_first = 0;
        for at in 0..self.performances.len() {
            let (Some(part), Some(action)) = (self.node_for(&self.performances[at].0), self.node_for(&self.performances[at].1)) else { continue };
            if self.nodes[part].shape != Shape::Part || self.nodes[action].shape != Shape::Action || decomposed.contains(&action) {
                continue;
            }
            // Arcadia allocates a function to one component.
            if moved.contains(&action) {
                beyond_first += 1;
                continue;
            }
            match self.nodes[action].parent {
                // Declared in a part already: allocated there.
                Some(parent) if self.nodes[parent].shape == Shape::Part => continue,
                Some(parent) => self.nodes[action].decomposes = Some(parent),
                None => {}
            }
            self.nodes[action].parent = Some(part);
            moved.insert(action);
        }
        self.count_by("function allocations", moved.len());
        if beyond_first > 0 {
            self.count_by("performances of a function allocated already (left out)", beyond_first);
            self.notes.push(format!("{beyond_first} performances of functions performed by another part already are left out: Arcadia allocates a function to one component"));
        }
    }

    /// Names that several nodes bear.
    pub(super) fn names_borne_twice(&self) -> HashSet<String> {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for node in &self.nodes {
            *counts.entry(node.name.as_str()).or_insert(0) += 1;
        }
        counts.into_iter().filter(|(_, n)| *n > 1).map(|(name, _)| name.to_string()).collect()
    }

    /// The written reference of a node: its id, qualified by its parent's
    /// when another element bears the same name (`vehicle_b.engine`), so
    /// that the configurations of a model do not make it ambiguous.
    pub(super) fn written_ref(&self, index: usize) -> Option<String> {
        self.written_ref_depth(index, 0)
    }

    fn written_ref_depth(&self, index: usize, depth: usize) -> Option<String> {
        let id = self.ids.get(&self.node_key(index))?.clone();
        if self.profiled {
            return Some(id);
        }
        let shared = self.shared_names.contains(&self.nodes[index].name) || self.shared_names.contains(&id);
        match (shared, self.nodes[index].parent) {
            (true, Some(parent)) if depth < 32 => Some(format!("{}.{id}", self.written_ref_depth(parent, depth + 1)?)),
            _ => Some(id),
        }
    }

    /// The name of a connector: its own, else what it carries, else its ends.
    pub(super) fn connector_name(&self, connector: &Element, from: &str, to: &str) -> String {
        if let Some(name) = Self::name_of(connector).or_else(|| connector.short_name.clone()) {
            return name;
        }
        let carried = connector
            .children
            .iter()
            .filter_map(|c| self.graph.get(c))
            .filter(|c| c.kind == "ItemUsage" || c.kind == "PayloadFeature")
            .flat_map(|c| c.types.iter())
            .chain(connector.types.iter())
            .filter_map(|t| self.graph.get(t))
            .find_map(Self::name_of);
        carried.unwrap_or_else(|| match (last_segment(from), last_segment(to)) {
            // `generateTorque.torque to amplifyTorque.torque`: the torque.
            (a, b) if a == b => a.to_string(),
            (a, b) => format!("{a} to {b}"),
        })
    }

    /// The written ends of a connection, interface or binding, read from
    /// its end features: the path as written (`car_b.tank.outlet`), not the
    /// port of the definition, which the first part of that definition
    /// carries. Flows keep their features (`generateTorque.torque`).
    pub(super) fn written_ends(&self, connector: &Element) -> Option<(String, String)> {
        if self.profiled || matches!(connector.kind.as_str(), "FlowUsage" | "FlowConnectionUsage" | "SuccessionFlowUsage") {
            return None;
        }
        let ends: Vec<&Element> = connector.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.is_end && c.kind == "ReferenceUsage").collect();
        let [from, to] = ends.as_slice() else { return None };
        Some((self.written_end(connector, from)?, self.written_end(connector, to)?))
    }

    /// The ends of a flow between items of ports (`from h.exit.air to
    /// c.entry.air`): the ports, and the parts holding them.
    pub(super) fn flow_port_ends(&self, flow: &Element) -> Option<((String, usize), (String, usize))> {
        let [from, to] = self.flow_port_bases(flow)?;
        Some(((self.end_ref(from)?, self.end_node(from)?), (self.end_ref(to)?, self.end_node(to)?)))
    }

    /// The ports a flow between port items goes between, as written.
    fn flow_port_bases<'e>(&'e self, flow: &'e Element) -> Option<[&'e str; 2]> {
        if self.profiled || !matches!(flow.kind.as_str(), "FlowUsage" | "FlowConnectionUsage" | "SuccessionFlowUsage") {
            return None;
        }
        let ends: Vec<&Element> = flow.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "FlowEnd").collect();
        let [from, to] = ends.as_slice() else { return None };
        let base = |end: &'e Element| -> Option<&'e str> {
            let base = end.subsets.first()?;
            let element = self.graph.get(base)?;
            let port = element.chaining.last().map(String::as_str).unwrap_or(base);
            (self.graph.kind_of(port) == "PortUsage").then_some(base.as_str())
        };
        Some([base(from)?, base(to)?])
    }

    /// What a flow between port items carries: the types of the items its
    /// ends name (`h.outlet.air`, an `Air`).
    pub(super) fn flow_port_items(&self, flow: &Element) -> Vec<String> {
        if self.flow_port_bases(flow).is_none() {
            return Vec::new();
        }
        let mut items: Vec<String> = Vec::new();
        let ends = flow.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "FlowEnd");
        let features = ends.flat_map(|end| end.children.iter().filter_map(|c| self.graph.get(c)));
        for feature in features {
            let typed = feature.redefines.iter().filter_map(|r| self.graph.get(r)).flat_map(|r| r.types.iter());
            for ty in typed.chain(feature.types.iter()) {
                if let Some(id) = self.ids.get(ty).filter(|id| !items.contains(id)) {
                    items.push(id.clone());
                }
            }
        }
        items
    }

    /// The node a connector is placed by: its source's, or the part holding
    /// the source port of a flow between port items. Known before any id.
    pub(super) fn connector_anchor(&self, connector: &Element) -> Option<usize> {
        connector.sources.first().and_then(|s| self.end_node(s)).or_else(|| self.flow_port_bases(connector).and_then(|[from, _]| self.end_node(from)))
    }

    /// A connector `exchange` gave up on, counted there.
    pub(super) fn connector_lost(&mut self, connector: &Element, why: &'static str) {
        self.count(why);
        self.lost.insert(connector.id.clone());
    }

    /// Whether a binding joins values (`bind a = x;` between attributes),
    /// not elements of the architecture.
    fn binds_values(&self, connector: &Element) -> bool {
        connector.kind == "BindingConnectorAsUsage"
            && connector.sources.iter().chain(connector.targets.iter()).all(|end| matches!(self.graph.kind_of(end), "AttributeUsage" | "ReferenceUsage" | "Feature" | "") && self.end_node(end).is_none())
    }

    /// One end: a path is read through its steps; a port named alone is
    /// one of the part holding the connector (its own or inherited).
    fn written_end(&self, connector: &Element, end: &Element) -> Option<String> {
        let referenced = end.subsets.first().and_then(|r| self.graph.get(r));
        if let Some(port) = referenced.filter(|r| r.chaining.is_empty() && r.kind == "PortUsage") {
            if let Some(holder) = connector.owner.as_deref().and_then(|o| self.enclosing_part(o)) {
                return Some(format!("{}.{}", self.written_ref(holder)?, port.name.clone()?));
            }
        }
        self.end_ref(&end.id)
    }

    /// The nearest written part at or above an element.
    fn enclosing_part(&self, id: &str) -> Option<usize> {
        if let Some(n) = self.node_of.get(id).copied().filter(|&n| self.nodes[n].shape == Shape::Part) {
            return Some(n);
        }
        self.owning_part(id)
    }

    /// A part with no type of its own that subsets a typed part
    /// (`part vehicle_b_1 :> vehicle_b`) is of that part's type.
    pub(super) fn inherit_subsetted_types(&mut self) {
        for index in 0..self.nodes.len() {
            let node = &self.nodes[index];
            if node.is_type.is_some() || node.def.is_some() || node.shape != Shape::Part {
                continue;
            }
            let Some(usage) = node.usage else { continue };
            let ty = usage.subsets.iter().filter_map(|s| self.node_of.get(s)).find_map(|&n| self.nodes[n].is_type.clone());
            self.nodes[index].is_type = ty;
        }
    }

    /// A message: a flow between events (`message of X from a.e to b.f`),
    /// whose ends are event occurrences it owns.
    pub(super) fn is_message(&self, flow: &Element) -> bool {
        flow.kind == "FlowUsage" && !self.event_ends(flow).is_empty()
    }

    /// The source and target events of a message, in order.
    fn event_ends(&self, flow: &Element) -> Vec<&'g Element> {
        flow.children.iter().filter_map(|c| self.graph.get(c)).filter(|c| c.kind == "EventOccurrenceUsage").collect()
    }

    /// Connectors that no level writes, outside scenarios: counted, and
    /// the first ones named in the notes.
    pub(super) fn count_connectors_left_out(&mut self) {
        if self.profiled {
            return;
        }
        let mut value_bindings = 0;
        let lost: Vec<String> = self
            .graph
            .of_kinds(CONNECTOR_KINDS)
            .filter(|c| !self.is_library(c) && !self.inside_occurrence(c) && !self.is_message(c) && !self.inside_definition(c))
            // Not written, and not counted by `exchange` already.
            .filter(|c| !self.ids.contains_key(&c.id) && !self.lost.contains(&c.id))
            .collect::<Vec<_>>()
            .into_iter()
            .filter(|c| {
                let values = self.binds_values(c);
                if values {
                    value_bindings += 1;
                }
                !values
            })
            .map(|c| Self::name_of(c).unwrap_or_else(|| format!("{} {}", c.kind, self.end_text(c))))
            .collect();
        self.count_by("bindings between values (not architecture, left out)", value_bindings);
        if lost.is_empty() {
            return;
        }
        self.count_by("connectors between elements not written (left out)", lost.len());
        let named: Vec<&str> = lost.iter().take(NAMED_LOSSES).map(String::as_str).collect();
        let more = lost.len().saturating_sub(NAMED_LOSSES);
        let tail = if more > 0 { format!(" and {more} more") } else { String::new() };
        self.notes.push(format!("connectors left out (an end is no written element): {}{tail}", named.join("; ")));
    }

    /// A connector of a definition (`interface def I { connect a to b; }`)
    /// joins the ends of the definition, not elements: nothing to write.
    fn inside_definition(&self, connector: &Element) -> bool {
        let owner = connector.owner.as_deref().map(|o| self.graph.kind_of(o)).unwrap_or("");
        matches!(owner, "InterfaceDefinition" | "ConnectionDefinition" | "FlowDefinition" | "AllocationDefinition")
    }

    /// `a.b to c.d`, from the written ends of a connector, as far as known.
    fn end_text(&self, connector: &Element) -> String {
        let end = |id: Option<&String>| id.and_then(|i| self.end_ref(i)).unwrap_or_else(|| "?".to_string());
        format!("{} to {}", end(connector.sources.first()), end(connector.targets.first()))
    }

    // --- messages ---------------------------------------------------------

    /// Messages written outside the profile's scenarios: in a part
    /// (`part part0 { message of x from driver.a to car.b; }`) or in an
    /// occurrence whose lifelines are parts nested in its one part
    /// (`occurrence CruiseControl { part vehicle_b { ... message ... } }`).
    /// One scenario per element holding messages; its lifelines are the
    /// written components the messages go between.
    pub(super) fn message_scenarios(&mut self) {
        if self.profiled {
            return;
        }
        let mut holders: Vec<&'g Element> = Vec::new();
        for flow in self.graph.of_kinds(&["FlowUsage"]).filter(|f| !self.is_library(f) && self.is_message(f)) {
            let Some(holder) = self.scenario_holder(flow) else { continue };
            // Not a scenario of the profile already (`scenarios` writes those).
            if self.kinds.get(&holder.id) != Some(&"Scenario") && !holders.iter().any(|h| h.id == holder.id) {
                holders.push(holder);
            }
        }
        for holder in holders {
            self.message_scenario(holder);
        }
    }

    /// The occurrence a message belongs to, else the element holding it.
    fn scenario_holder(&self, flow: &Element) -> Option<&'g Element> {
        let owner = flow.owner.as_deref()?;
        let occurrence = self.graph.ancestor(owner, |e| matches!(e.kind.as_str(), "OccurrenceDefinition" | "OccurrenceUsage" | "InteractionDefinition"));
        occurrence.or_else(|| self.graph.get(owner))
    }

    fn message_scenario(&mut self, holder: &'g Element) {
        let messages: Vec<&'g Element> = self
            .graph
            .of_kinds(&["FlowUsage"])
            .filter(|f| self.is_message(f) && self.scenario_holder(f).is_some_and(|h| h.id == holder.id))
            .collect();
        let mut participants: Vec<String> = Vec::new();
        let mut lines: Vec<(String, String, String)> = Vec::new();
        for message in &messages {
            let ends = self.event_ends(message);
            let from = ends.first().and_then(|e| self.lifeline_of_event(e));
            let to = ends.get(1).and_then(|e| self.lifeline_of_event(e));
            let (Some(from), Some(to)) = (from, to) else {
                self.count("messages of lifelines not written (left out)");
                continue;
            };
            for lifeline in [&from, &to] {
                if !participants.contains(lifeline) {
                    participants.push(lifeline.clone());
                }
            }
            let label = Self::name_of(message).unwrap_or_else(|| self.connector_name(message, &from, &to));
            lines.push((from, to, label));
        }
        if lines.is_empty() {
            return;
        }
        // A part holding messages is a component too: the scenario is its
        // interaction, keyed and named apart.
        let is_occurrence = matches!(holder.kind.as_str(), "OccurrenceDefinition" | "OccurrenceUsage" | "InteractionDefinition");
        let name = if is_occurrence { Self::display(holder) } else { format!("{} interaction", Self::display(holder)) };
        let id = self.remember(&format!("scenario:{}", holder.id), if is_occurrence { Self::identity_id(holder) } else { name.clone() }, "Scenario");
        self.count("scenarios");
        self.line(0, &format!("scenario {} {{", string_literal(&name)));
        self.line(1, &format!("id: {}", string_literal(&id)));
        if let Some(doc) = Self::description(holder) {
            self.line(1, &format!("description: {}", string_literal(&doc)));
        }
        self.line(1, &format!("participants: {}", list_literal(&participants)));
        for (from, to, label) in lines {
            self.count("messages");
            self.line(1, &format!("message {} -> {} {}", string_literal(&from), string_literal(&to), string_literal(&label)));
        }
        self.line(0, "}");
        self.line(0, "");
    }

    /// The written component an event end belongs to: the deepest part of
    /// the chain it references (`speedSensor.speedSensorPort.sent` is on
    /// `speedSensor`), through redefinitions to the written part.
    fn lifeline_of_event(&self, element: &Element) -> Option<String> {
        let referenced = element.subsets.iter().chain(element.redefines.iter()).find_map(|t| self.graph.get(t))?;
        let steps: Vec<&str> = if referenced.chaining.is_empty() { vec![referenced.id.as_str()] } else { referenced.chaining.iter().map(String::as_str).collect() };
        // The deepest step that is a written part; else the part owning the last step.
        let part = steps
            .iter()
            .rev()
            .filter(|s| matches!(self.graph.kind_of(s), "PartUsage" | "PartDefinition"))
            .find_map(|s| self.node_for(s).filter(|&n| self.nodes[n].shape == Shape::Part))
            .or_else(|| self.owning_part(steps.last()?))?;
        // Participants are named by id, unique, never by a qualified path.
        self.ids.get(&self.node_key(part)).cloned()
    }

    /// The nearest written part owning an element (an event on a port of a part).
    fn owning_part(&self, id: &str) -> Option<usize> {
        let mut current = self.graph.get(id)?.owner.as_deref();
        let mut hops = 0;
        while let Some(owner) = current {
            if let Some(n) = self.node_for(owner).filter(|&n| self.nodes[n].shape == Shape::Part) {
                return Some(n);
            }
            hops += 1;
            if hops > 32 {
                return None;
            }
            current = self.graph.get(owner)?.owner.as_deref();
        }
        None
    }
}

// --- views ----------------------------------------------------------------

impl<'g> Mapper<'g> {
    /// The views a SysML tool draws of a model, as diagrams cut from the
    /// logical view: the interconnection view of each part that is a
    /// configuration (a root part) or holds connections, and the dataflow
    /// of each action tree over the parts that perform its leaves.
    pub(super) fn sysml_views(&mut self) {
        if self.profiled {
            return;
        }
        let mut views: Vec<(String, Vec<String>, Option<usize>)> = Vec::new();
        for index in 0..self.nodes.len() {
            if let Some(shows) = self.interconnection(index) {
                views.push((format!("{} interconnection", self.nodes[index].name), shows, self.nodes[index].parent));
            }
        }
        for index in 0..self.nodes.len() {
            if self.nodes[index].shape != Shape::Action || self.function_root(index) != index {
                continue;
            }
            let leaves = self.leaves_of(index);
            if leaves.len() >= 2 {
                views.push((format!("{} dataflow", self.nodes[index].name), leaves, None));
            }
        }
        let mut titles: Vec<String> = Vec::new();
        for (title, shows, parent) in views {
            let title = self.distinct_title(&titles, title, parent);
            self.count("views");
            self.line(0, &format!("diagram {} {{", string_literal(&title)));
            self.line(1, "view: lab");
            self.line(1, &format!("shows: {}", list_literal(&shows)));
            self.line(0, "}");
            self.line(0, "");
            titles.push(title);
        }
    }

    /// What the interconnection view of a part shows: the part itself,
    /// when it is a written logical component holding parts and is a root
    /// or holds connections of its own.
    fn interconnection(&self, index: usize) -> Option<Vec<String>> {
        let node = &self.nodes[index];
        if node.shape != Shape::Part || node.level != Level::La || !node.children.iter().any(|&c| self.nodes[c].shape == Shape::Part) {
            return None;
        }
        let usage = node.usage.map(|u| u.id.as_str());
        let holds_connections = usage.is_some_and(|u| self.graph.of_kinds(CONNECTOR_KINDS).any(|c| c.owner.as_deref() == Some(u) && self.ids.contains_key(&c.id)));
        if node.parent.is_some() && !holds_connections {
            return None;
        }
        Some(vec![self.ids.get(&self.node_key(index))?.clone()])
    }

    /// The function a function belongs to at the top: up through the
    /// functions holding it and those it decomposes.
    fn function_root(&self, index: usize) -> usize {
        let mut current = index;
        for _ in 0..64 {
            let up = self.nodes[current].parent.filter(|&p| self.nodes[p].shape == Shape::Action).or(self.nodes[current].decomposes);
            match up {
                Some(next) => current = next,
                None => break,
            }
        }
        current
    }

    /// The written leaf functions under a root function, in order.
    fn leaves_of(&self, root: usize) -> Vec<String> {
        (0..self.nodes.len())
            .filter(|&i| i != root && self.nodes[i].shape == Shape::Action && self.function_root(i) == root)
            .filter(|&i| !self.nodes.iter().any(|n| n.parent == Some(i) && n.shape == Shape::Action))
            .filter_map(|i| self.ids.get(&self.node_key(i)).cloned())
            .collect()
    }

    /// A diagram title not taken yet: with the parent's name, else a number.
    fn distinct_title(&self, taken: &[String], title: String, parent: Option<usize>) -> String {
        if !taken.contains(&title) {
            return title;
        }
        if let Some(parent) = parent {
            let qualified = format!("{title} ({})", self.nodes[parent].name);
            if !taken.contains(&qualified) {
                return qualified;
            }
        }
        (2..).map(|n| format!("{title} {n}")).find(|t| !taken.contains(t)).expect("an unused number")
    }
}

/// Whether the metamodel types a key as something other than text (a
/// quantity, a number, a reference): a formula cannot be its value.
pub(super) fn is_typed_key(key: &str) -> bool {
    use crate::compiler::metamodel::{AttrType, Metamodel};
    use std::sync::OnceLock;
    static TYPED: OnceLock<HashSet<&'static str>> = OnceLock::new();
    TYPED
        .get_or_init(|| {
            let metamodel = Metamodel::current();
            metamodel.kinds.iter().flat_map(|k| k.attributes.iter()).filter(|a| !matches!(a.ty, AttrType::Text)).map(|a| a.key).collect()
        })
        .contains(key)
}

/// The last segment of a written reference (`engine.drivePwrPort`: `drivePwrPort`).
fn last_segment(reference: &str) -> &str {
    reference.rsplit('.').next().unwrap_or(reference)
}
