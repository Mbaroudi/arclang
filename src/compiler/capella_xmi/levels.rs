//! The five Arcadia levels of a Capella model, written level by level:
//! operational analysis, system analysis, logical and physical
//! architectures, EPBS. Each level declares its elements, their functions
//! and ports, its exchanges, its chains, its capabilities with their
//! relations, and records the realizations towards the level above.

use super::{data, display_name, first_ref, header_attributes, list_literal, refs, text_attr, Node, Writer};
use crate::compiler::source_edit::string_literal;
use std::collections::{HashMap, HashSet};

// --- functions and exchanges, shared by the levels ------------------------------------

/// The functions `holder` owns (`ownedFunctions`), nested, with their ports.
pub(super) fn functions(writer: &mut Writer, depth: usize, holder: &Node, keyword: &str, what: &'static str, kind: &'static str) {
    for function in holder.tagged("ownedFunctions") {
        writer.count(what);
        let id = writer.remember(function, kind);
        writer.line(depth, &format!("{keyword} {} {{", string_literal(&display_name(function))));
        header_attributes(writer, depth + 1, function, kind);
        function_ports(writer, depth + 1, function, &id);
        functions(writer, depth + 1, function, keyword, what, kind);
        writer.line(depth, "}");
        for realization in function.tagged("ownedFunctionRealizations") {
            if let Some(target) = first_ref(realization.attr("targetElement")) {
                writer.traces.push((id.clone(), "realizes", target));
            }
        }
    }
}

/// The input and output ports of a function, each with its Capella id.
fn function_ports(writer: &mut Writer, depth: usize, function: &Node, owner: &str) {
    let mut taken: HashSet<String> = HashSet::new();
    for (tag, direction) in [("inputs", "in"), ("outputs", "out")] {
        for port in function.tagged(tag) {
            writer.port_owner.insert(port.id().to_string(), owner.to_string());
            writer.kinds.insert(port.id().to_string(), "FunctionPort");
            let name = data::unique_name(&mut taken, &display_name(port));
            // A trace names a port as `Owner.port`.
            writer.ids.insert(port.id().to_string(), format!("{owner}.{name}"));
            writer.port_ref.insert(port.id().to_string(), format!("{owner}.{name}"));
            for realization in port.tagged("ownedPortRealizations") {
                if let Some(target) = first_ref(realization.attr("targetElement")) {
                    writer.traces.push((port.id().to_string(), "realizes", target));
                }
            }
            writer.count("function ports");
            writer.line(depth, &format!("port {direction} {} {{ id: {} }}", string_literal(&name), string_literal(port.id())));
        }
    }
}

/// The attributes an exchange carries besides its ends: its items, its categories.
fn exchange_attributes(writer: &mut Writer, exchange: &Node) -> String {
    let mut out = String::new();
    let items = writer.known(&refs(exchange.attr("exchangedItems")));
    if !items.is_empty() {
        out.push_str(&format!(" exchange_items: {}", list_literal(&items)));
    }
    if let Some(categories) = writer.categories.get(exchange.id()).cloned() {
        if categories.len() == 1 {
            out.push_str(&format!(" category: {}", string_literal(&categories[0])));
        } else {
            out.push_str(&format!(" categories: {}", list_literal(&categories)));
        }
    }
    if let Some(description) = text_attr(exchange, "description") {
        out.push_str(&format!(" description: {}", string_literal(&super::plain(&description))));
    }
    out
}

/// Functional exchanges anywhere under `holder`, written at `depth`.
pub(super) fn functional_exchanges(writer: &mut Writer, depth: usize, holder: &Node, keyword: &str, kind: &'static str, by_port: bool) {
    let mut taken: HashSet<String> = HashSet::new();
    let mut left_out = 0;
    for exchange in holder.find_all("fa:FunctionalExchange") {
        let (Some(source), Some(target)) = (first_ref(exchange.attr("source")), first_ref(exchange.attr("target"))) else { continue };
        // An end is a function port (`Function.port`), or the function itself;
        // operational interactions name the activity.
        let end = |writer: &Writer, reference: &str| -> Option<String> {
            if by_port {
                if let Some(port) = writer.port_ref.get(reference) {
                    return Some(port.clone());
                }
            }
            writer.port_owner.get(reference).cloned().or_else(|| writer.ids.get(reference).cloned())
        };
        let (Some(from), Some(to)) = (end(writer, &source), end(writer, &target)) else {
            left_out += 1;
            continue;
        };
        writer.count(if keyword == "interaction" { "interactions" } else { "functional exchanges" });
        writer.remember(exchange, kind);
        for realization in exchange.tagged("ownedFunctionalExchangeRealizations") {
            if let Some(target) = first_ref(realization.attr("targetElement")) {
                writer.traces.push((exchange.id().to_string(), "realizes", target));
            }
        }
        let name = data::unique_name(&mut taken, &display_name(exchange));
        let extra = exchange_attributes(writer, exchange);
        writer.line(depth, &format!("{keyword} {} {{ id: {} from: {} to: {}{extra} }}", string_literal(&name), string_literal(exchange.id()), string_literal(&from), string_literal(&to)));
    }
    if left_out > 0 {
        writer.count_by("functional exchanges whose end is not an imported function (left out)", left_out);
    }
}

/// The involved elements of a functional chain or operational process, in
/// the order its links give; document order when the links are missing.
fn chain_order(chain: &Node) -> Vec<String> {
    let involvements: Vec<&Node> = chain.tagged("ownedFunctionalChainInvolvements").collect();
    let functions: Vec<&Node> = involvements.iter().copied().filter(|i| !i.is("fa:FunctionalChainInvolvementLink")).collect();
    let links: Vec<&Node> = involvements.iter().copied().filter(|i| i.is("fa:FunctionalChainInvolvementLink")).collect();
    let involved = |node: &Node| first_ref(node.attr("involved"));
    if links.is_empty() {
        return functions.iter().filter_map(|f| involved(f)).collect();
    }
    // Start at the function no link points to; follow the links.
    let targeted: HashSet<String> = links.iter().filter_map(|l| first_ref(l.attr("target"))).collect();
    let next: HashMap<String, Vec<String>> = links.iter().fold(HashMap::new(), |mut map, link| {
        if let (Some(source), Some(target)) = (first_ref(link.attr("source")), first_ref(link.attr("target"))) {
            map.entry(source).or_default().push(target);
        }
        map
    });
    let by_id: HashMap<String, &Node> = functions.iter().map(|f| (f.id().to_string(), *f)).collect();
    let mut order: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = functions.iter().filter(|f| !targeted.contains(f.id())).map(|f| f.id().to_string()).rev().collect();
    while let Some(current) = stack.pop() {
        if !seen.insert(current.clone()) {
            continue;
        }
        if let Some(function) = by_id.get(&current) {
            if let Some(id) = involved(function) {
                order.push(id);
            }
        }
        if let Some(following) = next.get(&current) {
            for target in following.iter().rev() {
                stack.push(target.clone());
            }
        }
    }
    // Functions no link reaches keep their document order at the end.
    for function in &functions {
        if seen.insert(function.id().to_string()) {
            if let Some(id) = involved(function) {
                order.push(id);
            }
        }
    }
    order
}

/// The chains of `holder`: `functional_chain` (or `operational_process`)
/// declarations whose `involves` lists the elements written for its steps.
fn chains(writer: &mut Writer, depth: usize, holder: &Node, capella_kind: &str, keyword: &str, what: &'static str, kind: &'static str, accept: &[&str], taken: &mut HashSet<String>) {
    for chain in holder.find_all(capella_kind) {
        let steps = chain_order(chain);
        let involved = writer.known_of(&steps, accept);
        if involved.is_empty() {
            writer.count(if steps.is_empty() { "empty chains (left out: a chain involves at least one function)" } else { "chains whose steps were not imported (left out)" });
            continue;
        }
        writer.count(what);
        writer.remember(chain, kind);
        let name = data::unique_name(taken, &display_name(chain));
        writer.line(depth, &format!("{keyword} {} {{", string_literal(&name)));
        header_attributes(writer, depth + 1, chain, kind);
        writer.line(depth + 1, &format!("involves: {}", list_literal(&involved)));
        writer.line(depth, "}");
        for realization in chain.tagged("ownedFunctionalChainRealizations") {
            if let Some(target) = first_ref(realization.attr("targetElement")) {
                writer.traces.push((chain.id().to_string(), "realizes", target));
            }
        }
    }
}

/// The relations a capability declares: what it involves, extends, includes
/// and specializes, and what it realizes, filtered to what is written.
fn capability_relations(writer: &mut Writer, depth: usize, capability: &Node, involvable: &[&str], realizes_kind: Option<&str>) {
    let mut involves: Vec<String> = Vec::new();
    for tag in [
        "ownedEntityOperationalCapabilityInvolvements",
        "ownedCapabilityInvolvements",
        "ownedCapabilityRealizationInvolvements",
        "ownedAbstractFunctionAbstractCapabilityInvolvements",
        "ownedFunctionalChainAbstractCapabilityInvolvements",
    ] {
        involves.extend(capability.tagged(tag).filter_map(|i| first_ref(i.attr("involved"))));
    }
    // Processes and chains the capability owns are involved in it.
    involves.extend(capability.tagged("ownedFunctionalChains").map(|c| c.id().to_string()));
    let involves = writer.known_of(&involves, involvable);
    let mut unique: Vec<String> = Vec::new();
    for id in involves {
        if !unique.contains(&id) {
            unique.push(id);
        }
    }
    if !unique.is_empty() {
        writer.count_by("capability involvements", unique.len());
        writer.line(depth, &format!("involves: {}", list_literal(&unique)));
    }
    let own = capability.id().to_string();
    let not_self = |ids: Vec<String>| -> Vec<String> { ids.into_iter().filter(|id| *id != own).collect() };
    let extends = not_self(writer.known(&capability.tagged("extends").filter_map(|e| first_ref(e.attr("extended"))).collect::<Vec<_>>()));
    if !extends.is_empty() {
        writer.line(depth, &format!("extends: {}", list_literal(&extends)));
    }
    let includes = not_self(writer.known(&capability.tagged("includes").filter_map(|i| first_ref(i.attr("included"))).collect::<Vec<_>>()));
    if !includes.is_empty() {
        writer.line(depth, &format!("includes: {}", list_literal(&includes)));
    }
    let specializes = not_self(writer.known(&capability.tagged("superGeneralizations").filter_map(|g| first_ref(g.attr("super"))).chain(refs(capability.attr("super")).into_iter()).collect::<Vec<_>>()));
    if !specializes.is_empty() {
        writer.line(depth, &format!("specializes: {}", list_literal(&specializes)));
    }
    if let Some(kind) = realizes_kind {
        let realized: Vec<String> = capability.tagged("ownedAbstractCapabilityRealizations").filter_map(|r| first_ref(r.attr("targetElement"))).collect();
        if let Some(target) = writer.known_of(&realized, &[kind]).first() {
            writer.line(depth, &format!("realizes: {}", string_literal(target)));
        }
    }
}

// --- operational analysis ------------------------------------------------------------

pub(super) fn operational(writer: &mut Writer, oa: &Node) {
    writer.line(0, &format!("operational_analysis {} {{", string_literal(&display_name(oa))));
    writer.line(1, &format!("id: {}", string_literal(oa.id())));
    let mut taken: HashSet<String> = HashSet::new();

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
    let parent_of: HashMap<String, String> = entities.iter().flat_map(|e| e.tagged("ownedEntities").map(move |child| (child.id().to_string(), e.id().to_string()))).collect();
    for entity in &entities {
        let actor = entity.attr("actor") == Some("true");
        let kind = if actor { "Actor" } else { "OperationalEntity" };
        writer.count(if actor { "operational actors" } else { "operational entities" });
        writer.remember(entity, kind);
        let name = data::unique_name(&mut taken, &display_name(entity));
        writer.names.insert(entity.id().to_string(), name.clone());
        writer.line(1, &format!("{} {} {{", if actor { "actor" } else { "entity" }, string_literal(&name)));
        header_attributes(writer, 2, entity, kind);
        if let Some(parent) = parent_of.get(entity.id()) {
            writer.line(2, &format!("part_of: {}", string_literal(parent)));
        }
        writer.line(1, "}");
    }
    let roles = oa.find_all("oa:Role").len();
    if roles > 0 {
        writer.count_by("roles (left out: ArcLang has no operational role)", roles);
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
        fn write_activities(writer: &mut Writer, depth: usize, performer: &HashMap<String, String>, list: &[&Node]) {
            for activity in list {
                writer.count("operational activities");
                let id = writer.remember(activity, "OperationalActivity");
                writer.line(depth, &format!("operational_activity {} {{", string_literal(&display_name(activity))));
                header_attributes(writer, depth + 1, activity, "OperationalActivity");
                if let Some(entity) = performer.get(&id) {
                    writer.line(depth + 1, &format!("performed_by: {}", string_literal(entity)));
                }
                function_ports(writer, depth + 1, activity, &id);
                let nested: Vec<&Node> = activity.tagged("ownedFunctions").collect();
                write_activities(writer, depth + 1, performer, &nested);
                writer.line(depth, "}");
            }
        }
        write_activities(writer, 1, &performer, &all);
    }
    functional_exchanges(writer, 1, oa, "interaction", "OperationalExchange", false);
    for mean in oa.find_all("oa:CommunicationMean") {
        let (Some(source), Some(target)) = (first_ref(mean.attr("source")), first_ref(mean.attr("target"))) else { continue };
        let end = |writer: &Writer, id: &str| writer.ids.get(id).cloned().or_else(|| writer.part_type.get(id).and_then(|t| writer.ids.get(t).cloned()));
        let (Some(from), Some(to)) = (end(writer, &source), end(writer, &target)) else {
            writer.count("communication means between entities not written (left out)");
            continue;
        };
        writer.count("communication means");
        writer.remember(mean, "CommunicationMean");
        let name = data::unique_name(&mut taken, &display_name(mean));
        let extra = exchange_attributes(writer, mean);
        writer.line(1, &format!("communication_means {} {{ id: {} from: {} to: {}{extra} }}", string_literal(&name), string_literal(mean.id()), string_literal(&from), string_literal(&to)));
    }
    chains(writer, 1, oa, "oa:OperationalProcess", "operational_process", "operational processes", "OperationalProcess", &["OperationalActivity", "OperationalExchange"], &mut taken);
    for capability in oa.find_all("oa:OperationalCapability") {
        writer.count("operational capabilities");
        writer.remember(capability, "OperationalCapability");
        let name = data::unique_name(&mut taken, &display_name(capability));
        writer.line(1, &format!("operational_capability {} {{", string_literal(&name)));
        header_attributes(writer, 2, capability, "OperationalCapability");
        capability_relations(writer, 2, capability, &["Actor", "OperationalEntity", "OperationalActivity", "OperationalProcess"], None);
        writer.line(1, "}");
    }
    writer.line(0, "}");
    writer.line(0, "");
}

// --- system analysis ------------------------------------------------------------------

pub(super) fn system(writer: &mut Writer, sa: &Node) {
    writer.line(0, &format!("system_analysis {} {{", string_literal(&display_name(sa))));
    writer.line(1, &format!("id: {}", string_literal(sa.id())));
    let mut taken: HashSet<String> = HashSet::new();

    if let Some(pkg) = sa.find("ctx:SystemFunctionPkg") {
        for root in pkg.tagged("ownedSystemFunctions") {
            if root.tagged("ownedFunctions").next().is_some() {
                functions(writer, 1, root, "function", "system functions", "SystemFunction");
            } else {
                // A lone root function is a function of its own.
                let wrapper = Node { tag: "root".to_string(), kind: String::new(), attrs: Default::default(), children: vec![root.clone()], text: String::new() };
                functions(writer, 1, &wrapper, "function", "system functions", "SystemFunction");
            }
        }
    }
    functional_exchanges(writer, 1, sa, "functional_exchange", "FunctionalExchange", true);

    for component in sa.find_all("ctx:SystemComponent") {
        let actor = component.attr("actor") == Some("true");
        let kind = if actor { "SystemActor" } else { "SystemComponent" };
        writer.count(if actor { "system actors" } else { "system components" });
        let id = writer.remember(component, kind);
        let name = data::unique_name(&mut taken, &display_name(component));
        writer.names.insert(component.id().to_string(), name.clone());
        writer.line(1, &format!("{} {} {{", if actor { "actor" } else { "system_component" }, string_literal(&name)));
        header_attributes(writer, 2, component, kind);
        component_ports(writer, 2, component, &id);
        let allocated = writer.known_of(&component.tagged("ownedFunctionalAllocation").filter_map(|a| first_ref(a.attr("targetElement"))).collect::<Vec<_>>(), &["SystemFunction"]);
        if !allocated.is_empty() {
            writer.count_by("system function allocations", allocated.len());
            writer.line(2, &format!("allocated_functions: {}", list_literal(&allocated)));
        }
        writer.line(1, "}");
        for realization in component.tagged("ownedComponentRealizations") {
            if let Some(target) = first_ref(realization.attr("targetElement")) {
                writer.traces.push((component.id().to_string(), "realizes", target));
            }
        }
    }
    component_exchanges(writer, 1, sa, "system");
    chains(writer, 1, sa, "fa:FunctionalChain", "functional_chain", "functional chains", "FunctionalChain", &["SystemFunction"], &mut taken);

    // Missions exploit capabilities: a capability names the first mission exploiting it.
    let mut exploited: HashMap<String, Vec<String>> = HashMap::new();
    for mission in sa.find_all("ctx:Mission") {
        writer.count("missions");
        writer.remember(mission, "Mission");
        let name = data::unique_name(&mut taken, &display_name(mission));
        writer.line(1, &format!("mission {} {{", string_literal(&name)));
        header_attributes(writer, 2, mission, "Mission");
        let involved = writer.known_of(&mission.tagged("ownedMissionInvolvements").filter_map(|i| first_ref(i.attr("involved"))).collect::<Vec<_>>(), &["SystemActor", "SystemComponent"]);
        if !involved.is_empty() {
            writer.line(2, &format!("involves: {}", list_literal(&involved)));
        }
        writer.line(1, "}");
        for exploitation in mission.tagged("ownedCapabilityExploitations") {
            if let Some(capability) = first_ref(exploitation.attr("capability")) {
                exploited.entry(capability).or_default().push(mission.id().to_string());
            }
        }
    }
    for capability in sa.find_all("ctx:Capability") {
        writer.count("capabilities");
        writer.remember(capability, "Capability");
        let name = data::unique_name(&mut taken, &display_name(capability));
        writer.line(1, &format!("capability {} {{", string_literal(&name)));
        header_attributes(writer, 2, capability, "Capability");
        if let Some(missions) = exploited.get(capability.id()) {
            writer.line(2, &format!("mission: {}", string_literal(&missions[0])));
            if missions.len() > 1 {
                writer.line(2, &format!("also_exploited_by: {}", list_literal(&missions[1..])));
            }
        }
        capability_relations(writer, 2, capability, &["SystemActor", "SystemComponent", "SystemFunction", "FunctionalChain"], Some("OperationalCapability"));
        writer.line(1, "}");
    }
    writer.line(0, "}");
    writer.line(0, "");
}

// --- functions of a level ---------------------------------------------------------

/// Which component (or behaviour component) performs each function of a
/// level, from the functional allocations, and the parent of each function.
struct FunctionTree<'a> {
    by_id: HashMap<String, &'a Node>,
    allocated_to: HashMap<String, String>,
    parent_of: HashMap<String, String>,
    written: HashSet<String>,
}

impl<'a> FunctionTree<'a> {
    fn of(level: &'a Node, function_kind: &str) -> Self {
        let functions = level.find_all(function_kind);
        let by_id: HashMap<String, &Node> = functions.iter().map(|f| (f.id().to_string(), *f)).collect();
        let mut parent_of = HashMap::new();
        for function in &functions {
            for child in function.tagged("ownedFunctions") {
                parent_of.insert(child.id().to_string(), function.id().to_string());
            }
        }
        let mut allocated_to = HashMap::new();
        for allocation in level.find_all("fa:ComponentFunctionalAllocation") {
            if let (Some(target), Some(owner)) = (first_ref(allocation.attr("targetElement")), first_ref(allocation.attr("sourceElement"))) {
                allocated_to.insert(target, owner);
            }
        }
        FunctionTree { by_id, allocated_to, parent_of, written: HashSet::new() }
    }

    fn is_root(function: &Node) -> bool {
        function.name().starts_with("Root ")
    }

    /// The functions written directly under `owner` (a component id), or at
    /// the level when `owner` is `None`: those allocated there whose parent is
    /// not written in the same place.
    fn tops(&self, owner: Option<&str>) -> Vec<&'a Node> {
        let mut tops: Vec<&Node> = self
            .by_id
            .values()
            .copied()
            .filter(|f| !Self::is_root(f))
            .filter(|f| self.allocated_to.get(f.id()).map(String::as_str) == owner)
            .filter(|f| {
                let parent = self.parent_of.get(f.id()).and_then(|p| self.by_id.get(p));
                match parent {
                    None => true,
                    Some(parent) if Self::is_root(parent) => true,
                    Some(parent) => self.allocated_to.get(parent.id()).map(String::as_str) != owner,
                }
            })
            .collect();
        tops.sort_by_key(|f| (f.name().to_string(), f.id().to_string()));
        tops
    }
}

/// Write a function of the level with its ports, what it realizes, and the
/// sub-functions performed in the same place; a sub-function performed
/// elsewhere is written there and names this one as `parent:`.
fn level_function(writer: &mut Writer, depth: usize, function: &Node, owner: Option<&str>, tree: &mut FunctionTree<'_>, kind: &'static str, what: &'static str, taken: &mut HashSet<String>) {
    if !tree.written.insert(function.id().to_string()) {
        return;
    }
    writer.count(what);
    let id = writer.remember(function, kind);
    let name = data::unique_name(taken, &display_name(function));
    writer.line(depth, &format!("function {} {{", string_literal(&name)));
    header_attributes(writer, depth + 1, function, kind);
    let realized: Vec<String> = function.tagged("ownedFunctionRealizations").filter_map(|r| first_ref(r.attr("targetElement"))).collect();
    let realized = writer.known(&realized);
    if let Some(first) = realized.first() {
        writer.count("function realizations");
        writer.line(depth + 1, &format!("realizes: {}", string_literal(first)));
        for other in &realized[1..] {
            writer.traces.push((id.clone(), "realizes", other.clone()));
        }
    }
    if let Some(parent) = tree.parent_of.get(function.id()).filter(|p| tree.by_id.get(*p).is_some_and(|n| !FunctionTree::is_root(n))) {
        let parent_written_here = owner.is_some() && tree.allocated_to.get(parent).map(String::as_str) == owner;
        if !parent_written_here && !(owner.is_none() && tree.allocated_to.get(parent).is_none()) {
            writer.line(depth + 1, &format!("parent: {}", string_literal(parent)));
        }
    }
    function_ports(writer, depth + 1, function, &id);
    let mut children: Vec<&Node> = function.tagged("ownedFunctions").collect();
    children.sort_by_key(|c| (c.name().to_string(), c.id().to_string()));
    let mut nested_taken: HashSet<String> = HashSet::new();
    for child in children {
        let child_owner = tree.allocated_to.get(child.id()).map(String::as_str);
        // A child performed by the same component, or by nobody, nests here.
        if child_owner == owner || (child_owner.is_none() && owner.is_some()) {
            level_function(writer, depth + 1, child, owner, tree, kind, what, &mut nested_taken);
        }
    }
    writer.line(depth, "}");
}

/// The functions `owner` performs, with their trees.
fn owned_functions(writer: &mut Writer, depth: usize, owner: &str, tree: &mut FunctionTree<'_>, kind: &'static str, what: &'static str) {
    let mut taken: HashSet<String> = HashSet::new();
    for function in tree.tops(Some(owner)) {
        level_function(writer, depth, function, Some(owner), tree, kind, what, &mut taken);
    }
}

/// The functions nobody performs, written at the level.
fn unallocated_functions(writer: &mut Writer, depth: usize, tree: &mut FunctionTree<'_>, kind: &'static str, what: &'static str, taken: &mut HashSet<String>) {
    for function in tree.tops(None) {
        level_function(writer, depth, function, None, tree, kind, what, taken);
    }
}

/// The oriented ports of a component of any level.
fn component_ports(writer: &mut Writer, depth: usize, component: &Node, owner: &str) {
    let mut port_names: HashSet<String> = HashSet::new();
    for port in component.tagged("ownedFeatures").filter(|f| f.is("fa:ComponentPort")) {
        writer.port_owner.insert(port.id().to_string(), owner.to_string());
        // An unnamed port is named by its id, so that an exchange can name it.
        let label = data::unique_name(&mut port_names, &display_name(port));
        writer.port_ref.insert(port.id().to_string(), format!("{owner}.{label}"));
        writer.ids.insert(port.id().to_string(), format!("{owner}.{label}"));
        writer.kinds.insert(port.id().to_string(), "ComponentPort");
        for realization in port.tagged("ownedPortRealizations") {
            if let Some(target) = first_ref(realization.attr("targetElement")) {
                writer.traces.push((port.id().to_string(), "realizes", target));
            }
        }
        let direction = match port.attr("orientation") {
            Some("OUT") => "out",
            Some("INOUT") => "inout",
            _ => "in",
        };
        writer.count("component ports");
        writer.line(depth, &format!("port {direction} {} {{ id: {} }}", string_literal(&label), string_literal(port.id())));
    }
}

/// The component exchanges of a level, between component ports.
fn component_exchanges(writer: &mut Writer, depth: usize, level: &Node, level_name: &str) {
    let mut exchange_names: HashSet<String> = HashSet::new();
    let mut left_out = 0;
    for exchange in level.find_all("fa:ComponentExchange") {
        let (Some(source), Some(target)) = (first_ref(exchange.attr("source")), first_ref(exchange.attr("target"))) else { continue };
        let (Some(from), Some(to)) = (writer.port_ref.get(&source).cloned(), writer.port_ref.get(&target).cloned()) else {
            left_out += 1;
            continue;
        };
        writer.count("component exchanges");
        writer.remember(exchange, "ComponentExchange");
        for realization in exchange.tagged("ownedComponentExchangeRealizations") {
            if let Some(target) = first_ref(realization.attr("targetElement")) {
                writer.traces.push((exchange.id().to_string(), "realizes", target));
            }
        }
        let name = data::unique_name(&mut exchange_names, &display_name(exchange));
        let extra = exchange_attributes(writer, exchange);
        writer.line(depth, &format!("component_exchange {} {{ id: {} from_port: {} to_port: {}{extra} }}", string_literal(&name), string_literal(exchange.id()), string_literal(&from), string_literal(&to)));
    }
    if left_out > 0 {
        let _ = level_name;
        writer.count_by("component exchanges whose end is not an imported component port (left out)", left_out);
    }
}

// --- logical architecture ------------------------------------------------------------

fn logical_component(writer: &mut Writer, depth: usize, component: &Node, tree: &mut FunctionTree<'_>, taken: &mut HashSet<String>) {
    writer.count("logical components");
    let id = writer.remember(component, "LogicalComponent");
    let name = data::unique_name(taken, &display_name(component));
    writer.names.insert(id.clone(), name.clone());
    writer.line(depth, &format!("component {} {{", string_literal(&name)));
    header_attributes(writer, depth + 1, component, "LogicalComponent");
    if component.attr("actor") == Some("true") {
        writer.line(depth + 1, "actor: true");
    }
    component_ports(writer, depth + 1, component, &id);
    owned_functions(writer, depth + 1, &id, tree, "LogicalFunction", "logical functions");
    data::component_interfaces(writer, depth + 1, component);
    for child in component.tagged("ownedLogicalComponents") {
        logical_component(writer, depth + 1, child, tree, taken);
    }
    writer.line(depth, "}");
    for realization in component.tagged("ownedComponentRealizations") {
        if let Some(realized) = first_ref(realization.attr("targetElement")) {
            writer.traces.push((id.clone(), "realizes", realized));
        }
    }
}

pub(super) fn logical(writer: &mut Writer, la: &Node) {
    writer.line(0, &format!("logical_architecture {} {{", string_literal(&display_name(la))));
    writer.line(1, &format!("id: {}", string_literal(la.id())));
    let mut taken: HashSet<String> = HashSet::new();
    // Interfaces of every level are declared here, each naming its level:
    // the logical architecture is where ArcLang declares interfaces.
    let engineering = writer.engineering.clone();
    for (level, kind) in [("system", "ctx:SystemAnalysis"), ("logical", "la:LogicalArchitecture"), ("physical", "pa:PhysicalArchitecture")] {
        if let Some(architecture) = engineering.find(kind) {
            data::interfaces(writer, 1, architecture, level, &mut taken);
        }
    }

    let mut tree = FunctionTree::of(la, "la:LogicalFunction");
    let all_components = la.find_all("la:LogicalComponent");
    let nested: HashSet<String> = all_components.iter().flat_map(|p| p.tagged("ownedLogicalComponents").map(|x| x.id().to_string())).collect();
    for root in all_components.iter().copied().filter(|c| !nested.contains(c.id())) {
        logical_component(writer, 1, root, &mut tree, &mut taken);
    }
    unallocated_functions(writer, 1, &mut tree, "LogicalFunction", "logical functions (performed by no component)", &mut taken);
    functional_exchanges(writer, 1, la, "functional_exchange", "FunctionalExchange", true);
    component_exchanges(writer, 1, la, "logical");
    chains(writer, 1, la, "fa:FunctionalChain", "functional_chain", "functional chains", "FunctionalChain", &["LogicalFunction"], &mut taken);
    for realization in la.find_all("la:CapabilityRealization") {
        writer.count("capability realizations");
        writer.remember(realization, "CapabilityRealization");
        let name = data::unique_name(&mut taken, &display_name(realization));
        writer.line(1, &format!("capability_realization {} {{", string_literal(&name)));
        header_attributes(writer, 2, realization, "CapabilityRealization");
        capability_relations(writer, 2, realization, &["LogicalComponent", "FunctionalChain"], Some("Capability"));
        writer.line(1, "}");
    }
    writer.line(0, "}");
    writer.line(0, "");
}

// --- physical architecture -----------------------------------------------------------

pub(super) fn physical(writer: &mut Writer, pa: &Node) {
    writer.line(0, &format!("physical_architecture {} {{", string_literal(&display_name(pa))));
    writer.line(1, &format!("id: {}", string_literal(pa.id())));
    let mut taken: HashSet<String> = HashSet::new();
    let components: Vec<&Node> = pa.find_all("pa:PhysicalComponent");
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
    let mut tree = FunctionTree::of(pa, "pa:PhysicalFunction");
    for node in nodes_to_write {
        if !written_nodes.insert(node.id().to_string()) {
            continue;
        }
        writer.count("physical nodes");
        let node_id = writer.remember(node, "PhysicalNode");
        let node_name = data::unique_name(&mut taken, &display_name(node));
        writer.names.insert(node_id.clone(), node_name.clone());
        writer.line(1, &format!("node {} {{", string_literal(&node_name)));
        header_attributes(writer, 2, node, "PhysicalNode");
        let mut port_names: HashSet<String> = HashSet::new();
        for port in node.tagged("ownedFeatures").filter(|f| f.is("cs:PhysicalPort")) {
            writer.port_owner.insert(port.id().to_string(), node_id.clone());
            writer.kinds.insert(port.id().to_string(), "PhysicalPort");
            let label = data::unique_name(&mut port_names, &display_name(port));
            // A trace names a physical port as `Node.port`.
            writer.ids.insert(port.id().to_string(), format!("{node_id}.{label}"));
            writer.port_ref.insert(port.id().to_string(), format!("{node_id}.{label}"));
            writer.count("physical ports");
            writer.line(2, &format!("port {} {{ id: {} }}", string_literal(&label), string_literal(port.id())));
        }
        // The functions the node itself performs (a node may be a behaviour too).
        owned_functions(writer, 2, &node_id, &mut tree, "PhysicalFunction", "physical functions");
        let mut behaviour_names: HashSet<String> = HashSet::new();
        for behaviour in behaviours.iter().filter(|b| b.id() != root) {
            // A behaviour deployed on something that is not a node (another
            // behaviour, a part of no node) is held by the root node.
            let host = deployed_on.get(behaviour.id()).filter(|h| nodes.iter().any(|n| n.id() == *h)).cloned().unwrap_or_else(|| root.clone());
            if host != node_id {
                continue;
            }
            writer.count("behaviour components");
            let behaviour_id = writer.remember(behaviour, "BehaviorComponent");
            let behaviour_name = data::unique_name(&mut behaviour_names, &display_name(behaviour));
            writer.names.insert(behaviour_id.clone(), behaviour_name.clone());
            writer.line(2, &format!("behavior_component {} {{", string_literal(&behaviour_name)));
            header_attributes(writer, 3, behaviour, "BehaviorComponent");
            component_ports(writer, 3, behaviour, &behaviour_id);
            if let Some(realization) = behaviour.tagged("ownedComponentRealizations").next() {
                if let Some(realized) = first_ref(realization.attr("targetElement")) {
                    writer.line(3, &format!("allocated_component: {}", string_literal(&realized)));
                }
            }
            owned_functions(writer, 3, &behaviour_id, &mut tree, "PhysicalFunction", "physical functions");
            writer.line(2, "}");
        }
        writer.line(1, "}");
    }
    unallocated_functions(writer, 1, &mut tree, "PhysicalFunction", "physical functions (performed by no component)", &mut taken);
    functional_exchanges(writer, 1, pa, "functional_exchange", "FunctionalExchange", true);
    let mut link_names: HashSet<String> = HashSet::new();
    let mut left_out = 0;
    for link in pa.find_all("cs:PhysicalLink") {
        let ends = refs(link.attr("linkEnds"));
        if ends.len() != 2 {
            continue;
        }
        let (Some(from), Some(to)) = (writer.port_owner.get(&ends[0]).cloned(), writer.port_owner.get(&ends[1]).cloned()) else {
            left_out += 1;
            continue;
        };
        writer.count("physical links");
        writer.remember(link, "PhysicalLink");
        let name = data::unique_name(&mut link_names, &display_name(link));
        writer.names.insert(link.id().to_string(), name.clone());
        let extra = exchange_attributes(writer, link);
        writer.line(1, &format!("physical_link {} {{ id: {} from: {} to: {}{extra} }}", string_literal(&name), string_literal(link.id()), string_literal(&from), string_literal(&to)));
    }
    if left_out > 0 {
        writer.count_by("physical links whose end is not a port of an imported node (left out)", left_out);
    }
    let mut exchange_names: HashSet<String> = HashSet::new();
    for exchange in pa.find_all("fa:ComponentExchange") {
        let (Some(source), Some(target)) = (first_ref(exchange.attr("source")), first_ref(exchange.attr("target"))) else { continue };
        // Bound to the ports of the behaviour components when they are written.
        let end = |writer: &Writer, reference: &str| writer.port_ref.get(reference).cloned().or_else(|| writer.port_owner.get(reference).cloned().filter(|owner| writer.written(owner)));
        let (Some(from), Some(to)) = (end(writer, &source), end(writer, &target)) else {
            writer.count("physical component exchanges whose end is not an imported component (left out)");
            continue;
        };
        writer.count("physical exchanges");
        writer.remember(exchange, "PhysicalExchange");
        for realization in exchange.tagged("ownedComponentExchangeRealizations") {
            if let Some(target) = first_ref(realization.attr("targetElement")) {
                writer.traces.push((exchange.id().to_string(), "realizes", target));
            }
        }
        let name = data::unique_name(&mut exchange_names, &display_name(exchange));
        let extra = exchange_attributes(writer, exchange);
        writer.line(1, &format!("physical_exchange {} {{ id: {} from: {} to: {}{extra} }}", string_literal(&name), string_literal(exchange.id()), string_literal(&from), string_literal(&to)));
    }
    // Physical paths: ordered links, named.
    for path in pa.find_all("cs:PhysicalPath") {
        let involvements: Vec<&Node> = path.tagged("ownedPhysicalPathInvolvements").collect();
        let next: HashMap<String, String> = involvements.iter().filter_map(|i| first_ref(i.attr("nextInvolvements")).map(|n| (i.id().to_string(), n))).collect();
        let targeted: HashSet<&String> = next.values().collect();
        let by_id: HashMap<String, &Node> = involvements.iter().map(|i| (i.id().to_string(), *i)).collect();
        let mut order: Vec<String> = Vec::new();
        let mut current = involvements.iter().find(|i| !targeted.contains(&i.id().to_string())).map(|i| i.id().to_string());
        let mut seen: HashSet<String> = HashSet::new();
        while let Some(id) = current {
            if !seen.insert(id.clone()) {
                break;
            }
            if let Some(involved) = by_id.get(&id).and_then(|i| first_ref(i.attr("involved"))) {
                order.push(involved);
            }
            current = next.get(&id).cloned();
        }
        for involvement in &involvements {
            if seen.insert(involvement.id().to_string()) {
                if let Some(involved) = first_ref(involvement.attr("involved")) {
                    order.push(involved);
                }
            }
        }
        let links: Vec<String> = order.iter().filter(|id| writer.kinds.get(id.as_str()) == Some(&"PhysicalLink")).filter_map(|id| writer.names.get(id).cloned()).collect();
        if links.is_empty() {
            writer.count("physical paths without an imported link (left out)");
            continue;
        }
        writer.count("physical paths");
        writer.remember(path, "PhysicalPath");
        let name = data::unique_name(&mut taken, &display_name(path));
        writer.line(1, &format!("physical_path {} {{", string_literal(&name)));
        header_attributes(writer, 2, path, "PhysicalPath");
        writer.line(2, &format!("involves: {}", list_literal(&links)));
        writer.line(1, "}");
    }
    chains(writer, 1, pa, "fa:FunctionalChain", "functional_chain", "functional chains", "FunctionalChain", &["PhysicalFunction"], &mut taken);
    for realization in pa.find_all("la:CapabilityRealization") {
        writer.count("capability realizations");
        writer.remember(realization, "CapabilityRealization");
        let name = data::unique_name(&mut taken, &display_name(realization));
        writer.line(1, &format!("capability_realization {} {{", string_literal(&name)));
        header_attributes(writer, 2, realization, "CapabilityRealization");
        capability_relations(writer, 2, realization, &["PhysicalNode", "BehaviorComponent", "FunctionalChain"], Some("CapabilityRealization"));
        writer.line(1, "}");
    }
    writer.line(0, "}");
    writer.line(0, "");
}

// --- EPBS ------------------------------------------------------------------------------

pub(super) fn epbs(writer: &mut Writer, architecture: &Node) {
    let roots: Vec<&Node> = architecture
        .find_all("epbs:ConfigurationItem")
        .into_iter()
        .filter(|item| !architecture.find_all("epbs:ConfigurationItem").iter().any(|p| p.tagged("ownedConfigurationItems").any(|c| c.id() == item.id())))
        .collect();
    if roots.is_empty() {
        return;
    }
    writer.line(0, &format!("epbs {} {{", string_literal(&display_name(architecture))));
    let mut taken: HashSet<String> = HashSet::new();
    fn realizations(writer: &mut Writer, depth: usize, item: &Node) {
        let realized = writer.known(&item.tagged("ownedPhysicalArtifactRealizations").filter_map(|r| first_ref(r.attr("targetElement"))).collect::<Vec<_>>());
        if !realized.is_empty() {
            writer.count_by("configuration item realizations", realized.len());
            writer.line(depth, &format!("realizes: {}", list_literal(&realized)));
        }
    }
    fn items(writer: &mut Writer, depth: usize, holder: &Node, part_of: Option<&str>, taken: &mut HashSet<String>) {
        for item in holder.tagged("ownedConfigurationItems") {
            writer.count("configuration items");
            writer.remember(item, "EpbsItem");
            let name = data::unique_name(taken, &display_name(item));
            writer.line(depth, &format!("item {} {{", string_literal(&name)));
            header_attributes(writer, depth + 1, item, "EpbsItem");
            if let Some(kind) = item.attr("kind") {
                writer.line(depth + 1, &format!("kind: {}", string_literal(kind)));
            }
            if let Some(parent) = part_of {
                writer.line(depth + 1, &format!("part_of: {}", string_literal(parent)));
            }
            realizations(writer, depth + 1, item);
            writer.line(depth, "}");
            // Deeper items are listed beside their parent, which they name.
            items(writer, depth, item, Some(item.id()), taken);
        }
    }
    for root in roots {
        writer.count("configuration items");
        writer.remember(root, "EpbsSystem");
        let name = data::unique_name(&mut taken, &display_name(root));
        writer.line(1, &format!("system {} {{", string_literal(&name)));
        header_attributes(writer, 2, root, "EpbsSystem");
        if let Some(kind) = root.attr("kind") {
            writer.line(2, &format!("kind: {}", string_literal(kind)));
        }
        realizations(writer, 2, root);
        for subsystem in root.tagged("ownedConfigurationItems") {
            writer.count("configuration items");
            writer.remember(subsystem, "EpbsSubsystem");
            let name = data::unique_name(&mut taken, &display_name(subsystem));
            writer.line(2, &format!("subsystem {} {{", string_literal(&name)));
            header_attributes(writer, 3, subsystem, "EpbsSubsystem");
            if let Some(kind) = subsystem.attr("kind") {
                writer.line(3, &format!("kind: {}", string_literal(kind)));
            }
            realizations(writer, 3, subsystem);
            items(writer, 3, subsystem, None, &mut taken);
            writer.line(2, "}");
        }
        writer.line(1, "}");
    }
    writer.line(0, "}");
    writer.line(0, "");
}
