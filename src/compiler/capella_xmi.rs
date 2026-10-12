//! Import of a Capella project (`.capella`, Capella 6/7 XMI) into ArcLang.
//!
//! The model is read level by level — operational analysis, system
//! analysis, logical and physical architectures, EPBS — with its
//! requirements, data, state machines and realizations, and written as
//! ArcLang source that the compiler then compiles: an import that does not
//! compile is an import that failed, never a half model. Capella's own
//! identifiers become the `id` of every element, so a model imported twice
//! keeps its identities.
//!
//! The diagrams of the `.aird` beside the file become `diagram`
//! declarations: the elements a Capella diagram shows are what the
//! ArcLang diagram shows, as Capella users expect their diagrams back.

use super::source_edit::string_literal;
use super::{Compiler, CompilerConfig};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

/// One XML element, with its `xsi:type` shortened to `pkg:Kind`.
#[derive(Debug, Clone, Default)]
pub struct Node {
    pub tag: String,
    pub kind: String,
    pub attrs: BTreeMap<String, String>,
    pub children: Vec<Node>,
}

impl Node {
    fn attr(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).map(String::as_str)
    }
    fn id(&self) -> &str {
        self.attr("id").unwrap_or("")
    }
    fn name(&self) -> &str {
        self.attr("name").unwrap_or("")
    }
    fn is(&self, kind: &str) -> bool {
        self.kind == kind
    }
    /// Children with this tag, in order.
    fn tagged<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = &'a Node> + 'a {
        self.children.iter().filter(move |child| child.tag == tag)
    }
    /// Every descendant, depth first.
    fn walk<'a>(&'a self, into: &mut Vec<&'a Node>) {
        for child in &self.children {
            into.push(child);
            child.walk(into);
        }
    }
    /// The first descendant of this kind.
    fn find(&self, kind: &str) -> Option<&Node> {
        let mut all = Vec::new();
        self.walk(&mut all);
        all.into_iter().find(|node| node.is(kind))
    }
    fn find_all(&self, kind: &str) -> Vec<&Node> {
        let mut all = Vec::new();
        self.walk(&mut all);
        all.into_iter().filter(|node| node.is(kind)).collect()
    }
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&#xA;", "\n").replace("&#xD;", "").replace("&#x9;", "\t").replace("&amp;", "&")
}

fn short_kind(xsi: &str) -> String {
    // `org.polarsys.capella.core.data.la:LogicalComponent` -> `la:LogicalComponent`;
    // `Requirements:Requirement` stays.
    match xsi.rsplit_once(':') {
        Some((prefix, kind)) => format!("{}:{}", prefix.rsplit('.').next().unwrap_or(prefix), kind),
        None => xsi.to_string(),
    }
}

/// Parse an XMI document into a tree. Fails on malformed XML.
pub fn parse(xml: &str) -> Result<Node, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut stack: Vec<Node> = vec![Node { tag: "#document".to_string(), ..Default::default() }];
    let mut buffer = Vec::new();
    loop {
        let event = reader.read_event_into(&mut buffer).map_err(|e| format!("XML error at byte {}: {e}", reader.buffer_position()))?;
        let is_empty = matches!(event, Event::Empty(_));
        match event {
            Event::Start(start) | Event::Empty(start) => {
                let mut node = Node { tag: String::from_utf8_lossy(start.name().as_ref()).to_string(), ..Default::default() };
                for attribute in start.attributes().flatten() {
                    let key = String::from_utf8_lossy(attribute.key.as_ref()).to_string();
                    let value = unescape(&String::from_utf8_lossy(&attribute.value));
                    if key == "xsi:type" || key == "xmi:type" {
                        node.kind = short_kind(&value);
                    } else {
                        node.attrs.insert(key, value);
                    }
                }
                if is_empty {
                    stack.last_mut().expect("document").children.push(node);
                } else {
                    stack.push(node);
                }
            }
            Event::End(_) => {
                let done = stack.pop().expect("an open element");
                stack.last_mut().expect("document").children.push(done);
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    stack.pop().ok_or_else(|| "empty document".to_string())
}

/// A reference attribute: `#id`, or `file.capella#id`, several separated by spaces.
fn refs(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or("")
        .split_whitespace()
        .map(|item| item.rsplit('#').next().unwrap_or(item).to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn first_ref(value: Option<&str>) -> Option<String> {
    refs(value).into_iter().next()
}

/// What the import produced.
#[derive(Debug, Clone)]
pub struct Imported {
    pub source: String,
    /// What was read, by kind of element, for the report.
    pub counts: BTreeMap<&'static str, usize>,
    /// What was left out and why.
    pub notes: Vec<String>,
}

struct Writer {
    out: String,
    counts: BTreeMap<&'static str, usize>,
    notes: Vec<String>,
    /// Capella id -> ArcLang id written for it.
    ids: HashMap<String, String>,
    /// Capella id of a port -> id of the element holding it.
    port_owner: HashMap<String, String>,
    /// Capella id of a component port -> `component.port`, as an exchange names it.
    port_ref: HashMap<String, String>,
    /// Capella id of a part -> id of the component it types.
    part_type: HashMap<String, String>,
    traces: Vec<(String, &'static str, String)>,
}

impl Writer {
    fn line(&mut self, depth: usize, text: &str) {
        for _ in 0..depth {
            self.out.push_str("    ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }
    fn count(&mut self, what: &'static str) {
        *self.counts.entry(what).or_insert(0) += 1;
    }
    fn remember(&mut self, node: &Node) -> String {
        let id = node.id().to_string();
        self.ids.insert(id.clone(), id.clone());
        id
    }
}

fn text_attr(node: &Node, key: &str) -> Option<String> {
    node.attr(key).map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

/// `description` as Capella writes it: HTML; kept as plain text.
fn plain(html: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for c in html.chars() {
        match c {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn header_attributes(writer: &mut Writer, depth: usize, node: &Node) {
    writer.line(depth, &format!("id: {}", string_literal(node.id())));
    if let Some(description) = text_attr(node, "description") {
        writer.line(depth, &format!("description: {}", string_literal(&plain(&description))));
    }
}

// --- the levels --------------------------------------------------------------

fn functions(writer: &mut Writer, depth: usize, holder: &Node, keyword: &str, what: &'static str) {
    for function in holder.tagged("ownedFunctions") {
        writer.count(what);
        let id = writer.remember(function);
        writer.line(depth, &format!("{keyword} {} {{", string_literal(function.name())));
        header_attributes(writer, depth + 1, function);
        for port in function.tagged("inputs") {
            writer.port_owner.insert(port.id().to_string(), id.clone());
            writer.line(depth + 1, &format!("port in {} {{ id: {} }}", string_literal(port.name()), string_literal(port.id())));
        }
        for port in function.tagged("outputs") {
            writer.port_owner.insert(port.id().to_string(), id.clone());
            writer.line(depth + 1, &format!("port out {} {{ id: {} }}", string_literal(port.name()), string_literal(port.id())));
        }
        functions(writer, depth + 1, function, keyword, what);
        writer.line(depth, "}");
        for realization in function.tagged("ownedFunctionRealizations") {
            if let Some(target) = first_ref(realization.attr("targetElement")) {
                writer.traces.push((id.clone(), "realizes", target));
            }
        }
    }
}

/// Functional exchanges anywhere under `holder`, written at `depth`.
fn functional_exchanges(writer: &mut Writer, depth: usize, holder: &Node, keyword: &str) {
    for exchange in holder.find_all("fa:FunctionalExchange") {
        let (Some(source), Some(target)) = (first_ref(exchange.attr("source")), first_ref(exchange.attr("target"))) else { continue };
        // An end is a function port, or (operational interactions) the activity itself.
        let end = |writer: &Writer, reference: &str| -> Option<String> {
            writer.port_owner.get(reference).cloned().or_else(|| writer.ids.get(reference).cloned())
        };
        let (Some(from), Some(to)) = (end(writer, &source), end(writer, &target)) else {
            writer.notes.push(format!("functional exchange '{}' left out: an end belongs to no imported function", exchange.name()));
            continue;
        };
        writer.count("functional exchanges");
        let name = if exchange.name().is_empty() { exchange.id().to_string() } else { exchange.name().to_string() };
        writer.line(depth, &format!("{keyword} {} {{ id: {} from: {} to: {} }}", string_literal(&name), string_literal(exchange.id()), string_literal(&from), string_literal(&to)));
    }
}

fn operational(writer: &mut Writer, oa: &Node) {
    writer.line(0, &format!("operational_analysis {} {{", string_literal(oa.name())));
    writer.line(1, &format!("id: {}", string_literal(oa.id())));
    // Activities an entity performs: its functional allocations.
    let mut performer: HashMap<String, String> = HashMap::new();
    let entities: Vec<&Node> = oa.find_all("oa:Entity");
    for entity in &entities {
        for allocation in entity.tagged("ownedFunctionalAllocation") {
            if let Some(activity) = first_ref(allocation.attr("targetElement")) {
                performer.insert(activity, entity.id().to_string());
            }
        }
    }
    for entity in &entities {
        let actor = entity.attr("actor") == Some("true");
        writer.count(if actor { "operational actors" } else { "operational entities" });
        writer.remember(entity);
        writer.line(1, &format!("{} {} {{", if actor { "actor" } else { "entity" }, string_literal(entity.name())));
        header_attributes(writer, 2, entity);
        writer.line(1, "}");
    }
    for capability in oa.find_all("oa:OperationalCapability") {
        writer.count("operational capabilities");
        writer.remember(capability);
        writer.line(1, &format!("operational_capability {} {{", string_literal(capability.name())));
        header_attributes(writer, 2, capability);
        writer.line(1, "}");
    }
    // The root activity holds the real ones.
    if let Some(pkg) = oa.find("oa:OperationalActivityPkg") {
        let roots: Vec<&Node> = pkg.tagged("ownedOperationalActivities").collect();
        let mut all: Vec<&Node> = Vec::new();
        for root in &roots {
            if root.tagged("ownedFunctions").next().is_some() {
                all.extend(root.tagged("ownedFunctions"));
            } else {
                all.push(root);
            }
        }
        fn write_activities(writer: &mut Writer, performer: &HashMap<String, String>, list: &[&Node]) {
            for activity in list {
                writer.count("operational activities");
                let id = writer.remember(activity);
                writer.line(1, &format!("operational_activity {} {{", string_literal(activity.name())));
                header_attributes(writer, 2, activity);
                if let Some(entity) = performer.get(&id) {
                    writer.line(2, &format!("performed_by: {}", string_literal(entity)));
                }
                for port in activity.tagged("inputs").chain(activity.tagged("outputs")) {
                    writer.port_owner.insert(port.id().to_string(), id.clone());
                }
                writer.line(1, "}");
                let nested: Vec<&Node> = activity.tagged("ownedFunctions").collect();
                write_activities(writer, performer, &nested);
            }
        }
        write_activities(writer, &performer, &all);
    }
    functional_exchanges(writer, 1, oa, "interaction");
    writer.line(0, "}");
    writer.line(0, "");
}

fn system(writer: &mut Writer, sa: &Node) {
    writer.line(0, &format!("system_analysis {} {{", string_literal(sa.name())));
    writer.line(1, &format!("id: {}", string_literal(sa.id())));
    for component in sa.find_all("ctx:SystemComponent") {
        let actor = component.attr("actor") == Some("true");
        writer.count(if actor { "system actors" } else { "system components" });
        writer.remember(component);
        writer.line(1, &format!("{} {} {{", if actor { "actor" } else { "system_component" }, string_literal(component.name())));
        header_attributes(writer, 2, component);
        writer.line(1, "}");
    }
    for mission in sa.find_all("ctx:Mission") {
        writer.count("missions");
        writer.remember(mission);
        writer.line(1, &format!("mission {} {{", string_literal(mission.name())));
        header_attributes(writer, 2, mission);
        writer.line(1, "}");
    }
    for capability in sa.find_all("ctx:Capability") {
        writer.count("capabilities");
        writer.remember(capability);
        writer.line(1, &format!("capability {} {{", string_literal(capability.name())));
        header_attributes(writer, 2, capability);
        writer.line(1, "}");
    }
    if let Some(pkg) = sa.find("ctx:SystemFunctionPkg") {
        for root in pkg.tagged("ownedSystemFunctions") {
            if root.tagged("ownedFunctions").next().is_some() {
                functions(writer, 1, root, "function", "system functions");
            } else {
                // A lone root function is a function of its own.
                let wrapper = Node { tag: "root".to_string(), kind: String::new(), attrs: BTreeMap::new(), children: vec![root.clone()] };
                functions(writer, 1, &wrapper, "function", "system functions");
            }
        }
    }
    functional_exchanges(writer, 1, sa, "functional_exchange");
    writer.line(0, "}");
    writer.line(0, "");
}

fn logical_component(writer: &mut Writer, depth: usize, component: &Node, functions_by_id: &HashMap<String, &Node>) {
    writer.count("logical components");
    let id = writer.remember(component);
    writer.line(depth, &format!("component {} {{", string_literal(component.name())));
    header_attributes(writer, depth + 1, component);
    for port in component.tagged("ownedFeatures").filter(|f| f.is("fa:ComponentPort")) {
        writer.port_owner.insert(port.id().to_string(), id.clone());
        // An unnamed port is named by its id, so that an exchange can name it.
        let label = if port.name().is_empty() { port.id().to_string() } else { port.name().to_string() };
        writer.port_ref.insert(port.id().to_string(), format!("{id}.{label}"));
        let direction = match port.attr("orientation") {
            Some("OUT") => "out",
            Some("INOUT") => "inout",
            _ => "in",
        };
        writer.line(depth + 1, &format!("port {direction} {} {{ id: {} }}", string_literal(&label), string_literal(port.id())));
    }
    for allocation in component.tagged("ownedFunctionalAllocation") {
        let Some(target) = first_ref(allocation.attr("targetElement")) else { continue };
        let Some(function) = functions_by_id.get(&target) else {
            writer.notes.push(format!("component '{}': allocated function {target} not found", component.name()));
            continue;
        };
        writer.count("logical functions");
        let function_id = writer.remember(function);
        writer.line(depth + 1, &format!("function {} {{", string_literal(function.name())));
        header_attributes(writer, depth + 2, function);
        writer.line(depth + 1, "}");
        for port in function.tagged("inputs").chain(function.tagged("outputs")) {
            writer.port_owner.insert(port.id().to_string(), function_id.clone());
        }
        for realization in function.tagged("ownedFunctionRealizations") {
            if let Some(realized) = first_ref(realization.attr("targetElement")) {
                writer.traces.push((function_id.clone(), "realizes", realized));
            }
        }
    }
    for child in component.tagged("ownedLogicalComponents") {
        logical_component(writer, depth + 1, child, functions_by_id);
    }
    writer.line(depth, "}");
    for realization in component.tagged("ownedComponentRealizations") {
        if let Some(realized) = first_ref(realization.attr("targetElement")) {
            writer.traces.push((id.clone(), "realizes", realized));
        }
    }
}

fn logical(writer: &mut Writer, la: &Node) {
    writer.line(0, &format!("logical_architecture {} {{", string_literal(la.name())));
    writer.line(1, &format!("id: {}", string_literal(la.id())));
    let functions_by_id: HashMap<String, &Node> = la.find_all("la:LogicalFunction").into_iter().map(|f| (f.id().to_string(), f)).collect();
    // Functions no component performs are listed by the compiler as unallocated.
    let allocated: HashSet<String> = la.find_all("fa:ComponentFunctionalAllocation").iter().filter_map(|a| first_ref(a.attr("targetElement"))).collect();
    for root in la.find_all("la:LogicalComponent").into_iter().filter(|c| c.tag == "ownedLogicalComponents" && !la.find_all("la:LogicalComponent").iter().any(|p| p.tagged("ownedLogicalComponents").any(|x| x.id() == c.id()))) {
        logical_component(writer, 1, root, &functions_by_id);
    }
    for (id, function) in &functions_by_id {
        if !allocated.contains(id) && function.tagged("ownedFunctions").next().is_none() && !function.name().starts_with("Root ") {
            writer.notes.push(format!("logical function '{}' is allocated to no component: left out", function.name()));
        }
    }
    for exchange in la.find_all("fa:ComponentExchange") {
        let (Some(source), Some(target)) = (first_ref(exchange.attr("source")), first_ref(exchange.attr("target"))) else { continue };
        let (Some(from), Some(to)) = (writer.port_ref.get(&source).cloned(), writer.port_ref.get(&target).cloned()) else {
            writer.notes.push(format!("component exchange '{}' left out: an end is not an imported component port", exchange.name()));
            continue;
        };
        writer.count("component exchanges");
        let name = if exchange.name().is_empty() { exchange.id().to_string() } else { exchange.name().to_string() };
        writer.line(1, &format!("component_exchange {} {{ id: {} from_port: {} to_port: {} }}", string_literal(&name), string_literal(exchange.id()), string_literal(&from), string_literal(&to)));
    }
    writer.line(0, "}");
    writer.line(0, "");
}

fn physical(writer: &mut Writer, pa: &Node) {
    writer.line(0, &format!("physical_architecture {} {{", string_literal(pa.name())));
    writer.line(1, &format!("id: {}", string_literal(pa.id())));
    let components: Vec<&Node> = pa.find_all("pa:PhysicalComponent");
    for part in pa.find_all("cs:Part") {
        if let Some(typed) = first_ref(part.attr("abstractType")) {
            writer.part_type.insert(part.id().to_string(), typed);
        }
    }
    // Where a behaviour component is deployed: the node whose part holds its part.
    let mut deployed_on: HashMap<String, String> = HashMap::new();
    for link in pa.find_all("deployment:PartDeploymentLink") {
        let (Some(location), Some(element)) = (first_ref(link.attr("location")), first_ref(link.attr("deployedElement"))) else { continue };
        let (Some(node), Some(behaviour)) = (writer.part_type.get(&location).cloned(), writer.part_type.get(&element).cloned()) else { continue };
        deployed_on.insert(behaviour, node);
    }
    let by_id: HashMap<String, &Node> = components.iter().map(|c| (c.id().to_string(), *c)).collect();
    let is_node = |c: &Node| c.attr("nature") == Some("NODE");
    let nodes: Vec<&Node> = components.iter().copied().filter(|c| is_node(c)).collect();
    let behaviours: Vec<&Node> = components.iter().copied().filter(|c| !is_node(c)).collect();
    // The root physical system holds what is deployed nowhere.
    let root = components.first().map(|c| c.id().to_string()).unwrap_or_default();
    let mut written_nodes: HashSet<String> = HashSet::new();
    let mut nodes_to_write: Vec<&Node> = nodes.clone();
    if !nodes.iter().any(|n| n.id() == root) {
        if let Some(root_node) = by_id.get(&root) {
            nodes_to_write.insert(0, root_node);
        }
    }
    for node in nodes_to_write {
        if !written_nodes.insert(node.id().to_string()) {
            continue;
        }
        writer.count("physical nodes");
        let node_id = writer.remember(node);
        writer.line(1, &format!("node {} {{", string_literal(node.name())));
        header_attributes(writer, 2, node);
        for port in node.tagged("ownedFeatures").filter(|f| f.is("cs:PhysicalPort")) {
            writer.port_owner.insert(port.id().to_string(), node_id.clone());
            writer.line(2, &format!("port {} {{ id: {} }}", string_literal(port.name()), string_literal(port.id())));
        }
        for behaviour in behaviours.iter().filter(|b| b.id() != root) {
            let host = deployed_on.get(behaviour.id()).cloned().unwrap_or_else(|| root.clone());
            if host != node_id {
                continue;
            }
            writer.count("behaviour components");
            let behaviour_id = writer.remember(behaviour);
            writer.line(2, &format!("behavior_component {} {{", string_literal(behaviour.name())));
            header_attributes(writer, 3, behaviour);
            if let Some(realization) = behaviour.tagged("ownedComponentRealizations").next() {
                if let Some(realized) = first_ref(realization.attr("targetElement")) {
                    writer.line(3, &format!("allocated_component: {}", string_literal(&realized)));
                }
            }
            writer.line(2, "}");
            let _ = behaviour_id;
        }
        writer.line(1, "}");
    }
    for link in pa.find_all("cs:PhysicalLink") {
        let ends = refs(link.attr("linkEnds"));
        if ends.len() != 2 {
            continue;
        }
        let (Some(from), Some(to)) = (writer.port_owner.get(&ends[0]).cloned(), writer.port_owner.get(&ends[1]).cloned()) else {
            writer.notes.push(format!("physical link '{}' left out: an end is not a port of an imported node", link.name()));
            continue;
        };
        writer.count("physical links");
        let name = if link.name().is_empty() { link.id().to_string() } else { link.name().to_string() };
        writer.line(1, &format!("physical_link {} {{ id: {} from: {} to: {} }}", string_literal(&name), string_literal(link.id()), string_literal(&from), string_literal(&to)));
    }
    writer.line(0, "}");
    writer.line(0, "");
}

fn requirements(writer: &mut Writer, root: &Node) {
    let modules: Vec<&Node> = root.find_all("CapellaRequirements:CapellaModule");
    if modules.is_empty() {
        return;
    }
    writer.line(0, "requirements system {");
    for module in modules {
        for requirement in module.find_all("Requirements:Requirement") {
            writer.count("requirements");
            let id = requirement.id().to_string();
            let short = text_attr(requirement, "ReqIFIdentifier").unwrap_or_else(|| id.clone());
            let title = text_attr(requirement, "ReqIFLongName").or_else(|| text_attr(requirement, "ReqIFName")).unwrap_or_else(|| short.clone());
            writer.ids.insert(id.clone(), short.clone());
            writer.line(1, &format!("req {} {} {{", string_literal(&short), string_literal(&title)));
            writer.line(2, &format!("id: {}", string_literal(&short)));
            let text = text_attr(requirement, "ReqIFText").map(|t| plain(&t)).unwrap_or_else(|| title.clone());
            writer.line(2, &format!("description: {}", string_literal(&text)));
            writer.line(1, "}");
        }
    }
    writer.line(0, "}");
    writer.line(0, "");
}

fn data(writer: &mut Writer, root: &Node) {
    for item in root.find_all("information:ExchangeItem") {
        writer.count("exchange items");
        writer.remember(item);
        let mechanism = item.attr("exchangeMechanism").unwrap_or("FLOW");
        writer.line(0, &format!("exchange_item {} {{ id: {} mechanism: {} }}", string_literal(item.name()), string_literal(item.id()), string_literal(mechanism)));
    }
    for class in root.find_all("information:Class") {
        writer.count("classes");
        writer.remember(class);
        writer.line(0, &format!("class {} {{ id: {} }}", string_literal(class.name()), string_literal(class.id())));
    }
    writer.line(0, "");
}

fn state_machines(writer: &mut Writer, root: &Node) {
    for machine in root.find_all("capellacommon:StateMachine") {
        let Some(region) = machine.tagged("ownedRegions").next() else { continue };
        let states: Vec<&Node> = region.tagged("ownedStates").collect();
        if states.is_empty() {
            continue;
        }
        writer.count("state machines");
        // Only states and modes are declared; a transition through a
        // choice, fork, join or final pseudo-state is left out.
        let names: HashMap<String, String> = states
            .iter()
            .filter(|s| s.is("capellacommon:State") || s.is("capellacommon:Mode"))
            .map(|s| (s.id().to_string(), s.name().to_string()))
            .collect();
        writer.line(0, &format!("state_machine {} {{", string_literal(machine.name())));
        let initial = states.iter().find(|s| s.is("capellacommon:InitialPseudoState"));
        let first_real = states.iter().find(|s| s.is("capellacommon:State") || s.is("capellacommon:Mode"));
        if let Some(first) = first_real {
            writer.line(1, &format!("initial: {}", string_literal(first.name())));
        }
        for state in states.iter().filter(|s| s.is("capellacommon:State") || s.is("capellacommon:Mode")) {
            writer.line(1, &format!("{} {} {{}}", if state.is("capellacommon:Mode") { "mode" } else { "state" }, string_literal(state.name())));
        }
        for transition in region.tagged("ownedTransitions") {
            let (Some(source), Some(target)) = (first_ref(transition.attr("source")), first_ref(transition.attr("target"))) else { continue };
            let from = names.get(&source);
            let to = names.get(&target);
            let (Some(from), Some(to)) = (from, to) else {
                if initial.is_none_or(|i| i.id() != source) {
                    let note = format!("state machine '{}': transitions through pseudo-states (choice, fork, join, final) were left out", machine.name());
                    if !writer.notes.contains(&note) {
                        writer.notes.push(note);
                    }
                }
                continue;
            };
            if from.is_empty() || to.is_empty() {
                continue;
            }
            let trigger = text_attr(transition, "triggerDescription").unwrap_or_default();
            writer.line(1, &format!("transition {} -> {} {{ trigger: {} }}", string_literal(from), string_literal(to), string_literal(&trigger)));
        }
        writer.line(0, "}");
        writer.line(0, "");
    }
}

// --- diagrams of the .aird ----------------------------------------------------

/// The ArcLang view a Capella diagram stands for, from the prefix Capella
/// puts in its name (`[LAB] ...`), else from its description.
fn view_of(name: &str, description: &str) -> Option<&'static str> {
    let prefix = name.trim_start().strip_prefix('[').and_then(|rest| rest.split(']').next()).unwrap_or("");
    let by_prefix = match prefix {
        "OAB" | "OEB" | "OEBD" | "OAIB" | "ORB" => Some("oab"),
        "SAB" | "SDFB" | "CSA" => Some("sab"),
        "LAB" | "LDFB" | "LCBD" => Some("lab"),
        "PAB" | "PDFB" | "PCBD" => Some("pab"),
        "MCB" | "OCB" | "CC" | "COC" | "CRB" => Some("cap"),
        "CDB" => Some("cdb"),
        "CIBD" => Some("pbs"),
        _ => None,
    };
    if by_prefix.is_some() {
        return by_prefix;
    }
    let kind = description.rsplit("name='").next().unwrap_or("").trim_end_matches("']").replace("%20", " ");
    match kind.as_str() {
        "Operational Architecture Blank" | "Operational Entity Blank" | "Operational Entity Breakdown" => Some("oab"),
        "System Architecture Blank" | "System Data Flow Blank" | "Contextual System Actors" => Some("sab"),
        "Logical Architecture Blank" | "Logical Data Flow Blank" | "Logical Component Breakdown" => Some("lab"),
        "Physical Architecture Blank" | "Physical Data Flow Blank" | "Physical Component Breakdown" => Some("pab"),
        "Missions Capabilities Blank" | "Operational Capabilities Blank" | "Contextual Capability" | "Contextual Operational Capability" => Some("cap"),
        "Class Diagram Blank" => Some("cdb"),
        "Configuration Items Breakdown" => Some("pbs"),
        _ => None,
    }
}

/// The `href` of a child element: `Model.capella#id`.
fn hrefs<'a>(node: &'a Node, tag: &'a str) -> impl Iterator<Item = String> + 'a {
    node.tagged(tag).flat_map(|child| refs(child.attr("href")).into_iter())
}

fn diagrams(writer: &mut Writer, aird: &Node) {
    let mut everything = Vec::new();
    aird.walk(&mut everything);
    let descriptors: Vec<&Node> = everything.iter().copied().filter(|n| n.tag == "ownedRepresentationDescriptors").collect();
    let representations: HashMap<String, &Node> = everything
        .iter()
        .copied()
        .filter(|n| n.tag == "ownedRepresentations" || n.tag.ends_with("DSemanticDiagram"))
        .filter_map(|n| n.attr("uid").map(|uid| (uid.to_string(), n)))
        .collect();
    for descriptor in descriptors {
        let name = descriptor.name();
        let description = descriptor.tagged("description").next().and_then(|d| d.attr("href")).unwrap_or("");
        let Some(view) = view_of(name, description) else {
            writer.notes.push(format!("diagram '{name}' left out: its kind has no ArcLang view"));
            continue;
        };
        let Some(diagram) = first_ref(descriptor.attr("repPath")).and_then(|path| representations.get(&path).copied()) else {
            writer.notes.push(format!("diagram '{name}' left out: its representation was not found in the .aird"));
            continue;
        };
        let mut shown: Vec<String> = Vec::new();
        let mut seen = HashSet::new();
        let mut elements = Vec::new();
        diagram.walk(&mut elements);
        for element in elements.iter().filter(|e| e.tag == "ownedDiagramElements") {
            for target in hrefs(element, "target").chain(hrefs(element, "semanticElements")) {
                if let Some(id) = writer.ids.get(&target) {
                    if seen.insert(id.clone()) {
                        shown.push(id.clone());
                    }
                }
            }
        }
        if shown.is_empty() {
            writer.notes.push(format!("diagram '{name}' left out: it shows no imported element"));
            continue;
        }
        writer.count("diagrams");
        let list: Vec<String> = shown.iter().map(|id| string_literal(id)).collect();
        writer.line(0, &format!("diagram {} {{", string_literal(name)));
        writer.line(1, &format!("view: {view}"));
        writer.line(1, &format!("shows: [{}]", list.join(", ")));
        writer.line(0, "}");
        writer.line(0, "");
    }
}

// --- the import -----------------------------------------------------------------

/// Import a `.capella` file; the `.aird` beside it, when there is one, gives
/// the diagrams. The result compiles, or the import fails.
pub fn import_file(path: &Path) -> Result<Imported, String> {
    let xml = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let aird_path = path.with_extension("aird");
    let aird = match std::fs::read_to_string(&aird_path) {
        Ok(text) => Some(text),
        Err(_) => None,
    };
    import_strings(&xml, aird.as_deref(), &path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default())
}

pub fn import_strings(capella_xml: &str, aird_xml: Option<&str>, file_name: &str) -> Result<Imported, String> {
    let document = parse(capella_xml)?;
    let project = document.children.first().ok_or("no root element")?;
    if !project.kind.ends_with(":Project") && !project.tag.ends_with(":Project") {
        return Err(format!("not a Capella project: the root is <{}>", project.tag));
    }
    let engineering = project.find("capellamodeller:SystemEngineering").ok_or("no SystemEngineering in the project")?;
    let mut writer = Writer {
        out: String::new(),
        counts: BTreeMap::new(),
        notes: Vec::new(),
        ids: HashMap::new(),
        port_owner: HashMap::new(),
        port_ref: HashMap::new(),
        part_type: HashMap::new(),
        traces: Vec::new(),
    };
    let project_name = project.attr("name").filter(|n| !n.is_empty()).unwrap_or("Capella model");
    writer.line(0, &format!("model {} {{", string_literal(project_name)));
    writer.line(1, &format!("name: {}", string_literal(project_name)));
    writer.line(1, &format!("description: {}", string_literal(&format!("Imported from Capella ({file_name})"))));
    writer.line(0, "}");
    writer.line(0, "");

    requirements(&mut writer, project);
    for architecture in engineering.tagged("ownedArchitectures") {
        match architecture.kind.as_str() {
            "oa:OperationalAnalysis" => operational(&mut writer, architecture),
            "ctx:SystemAnalysis" => system(&mut writer, architecture),
            "la:LogicalArchitecture" => logical(&mut writer, architecture),
            "pa:PhysicalArchitecture" => physical(&mut writer, architecture),
            "epbs:EPBSArchitecture" => {
                let items = architecture.find_all("epbs:ConfigurationItem").len();
                if items > 0 {
                    writer.notes.push(format!("EPBS: {items} configuration items left out (not imported yet)"));
                }
            }
            other => writer.notes.push(format!("architecture '{}' of kind {other} left out", architecture.name())),
        }
    }
    data(&mut writer, engineering);
    state_machines(&mut writer, engineering);

    // Realizations between what was written.
    let traces = std::mem::take(&mut writer.traces);
    for (from, kind, to) in traces {
        if writer.ids.contains_key(&from) && writer.ids.contains_key(&to) {
            writer.count("realization traces");
            writer.line(0, &format!("trace {} {kind} {}", string_literal(&from), string_literal(&to)));
        }
    }
    writer.line(0, "");

    if let Some(aird_xml) = aird_xml {
        match parse(aird_xml) {
            Ok(aird) => diagrams(&mut writer, &aird),
            Err(error) => writer.notes.push(format!("the .aird could not be read: {error}")),
        }
    } else {
        writer.notes.push("no .aird beside the file: no diagram imported".to_string());
    }

    let source = writer.out.clone();
    Compiler::new(CompilerConfig { optimization_level: 0, target: "json".to_string() })
        .compile_string(&source)
        .map_err(|e| format!("the imported model does not compile: {e}"))?;
    Ok(Imported { source, counts: writer.counts, notes: writer.notes })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/capella/test7_0/Model Test 7.0.capella")
    }

    #[test]
    fn the_capella_7_fixture_imports_compiles_and_keeps_its_diagrams() {
        let imported = import_file(&fixture()).expect("imports");
        let counts = &imported.counts;
        assert!(counts.get("logical components").copied().unwrap_or(0) >= 10, "{counts:?}");
        assert!(counts.get("logical functions").copied().unwrap_or(0) >= 10, "{counts:?}");
        assert!(counts.get("physical nodes").copied().unwrap_or(0) >= 1, "{counts:?}");
        assert!(counts.get("requirements").copied().unwrap_or(0) >= 5, "{counts:?}");
        assert!(counts.get("component exchanges").copied().unwrap_or(0) >= 5, "{counts:?}");
        assert!(counts.get("diagrams").copied().unwrap_or(0) >= 3, "{counts:?} {:?}", imported.notes);
        // Capella's identifiers are the ids: an import is stable.
        assert!(imported.source.contains("id: \"") && imported.source.contains("component_exchange"));
        let again = import_file(&fixture()).expect("imports again");
        assert_eq!(again.source, imported.source);
    }

    #[test]
    fn a_file_that_is_not_a_project_is_refused() {
        assert!(import_strings("<a/>", None, "a.capella").is_err());
        assert!(import_strings("<not xml", None, "a.capella").is_err());
    }
}
