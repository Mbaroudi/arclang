//! The physical architecture: nodes with their physical ports, behavior
//! components deployed on them with their ports and the logical component
//! each one realizes, the tree of physical functions, physical links and
//! paths, component exchanges, functional chains and capability realizations.
//!
//! Capella keeps one root physical component; the first node is that root,
//! and every other component is its sub-component, through a part. A
//! behavior component written in a node is deployed on that node.

use super::context::{derived, id_of, port_id, text, Context};
use super::functions::{self, Placed};
use super::header::apply;
use super::logical::{capability_realization, exchange_ends, functional_exchange};
use super::relations;
use super::system::component_port;
use super::xml::El;
use crate::compiler::ast::{BehaviorComponent, PhysicalArchitecture, PhysicalLink, PhysicalNode};

fn level_id(pa: &PhysicalArchitecture) -> String {
    if !pa.id.is_empty() {
        return pa.id.clone();
    }
    derived(&format!("physical-architecture:{}", pa.name))
}

fn part_id(component: &str) -> String {
    derived(&format!("{component}:part"))
}

fn link_id(link: &PhysicalLink, index: usize) -> String {
    id_of(&link.attributes, &format!("link:{index}:{}", link.name))
}

/// The port of a link end: `Node.port` names one; a bare node gets a port of its own for the link.
/// Whether `Owner.port` names an oriented component port (a node's
/// `port in|out`, or any port of a behavior component).
fn is_component_port(pa: &PhysicalArchitecture, end: &str) -> bool {
    let by_id = pa.nodes.iter().any(|n| n.component_ports.iter().any(|p| p.id == end) || n.behavior_components.iter().any(|b| b.ports.iter().any(|p| p.id == end)));
    if by_id {
        return true;
    }
    let Some((owner, port)) = end.rsplit_once('.') else { return false };
    pa.nodes.iter().any(|n| {
        ((n.id == owner || n.name == owner) && n.component_ports.iter().any(|p| p.name == port))
            || n.behavior_components.iter().any(|b| (b.id == owner || b.name == owner) && b.ports.iter().any(|p| p.name == port))
    })
}

/// Whether a link's or an exchange's end names a port: `owner.port`, or a port's own id.
fn names_a_port(context: &Context, end: &str) -> bool {
    end.contains('.') || context.port_owners.contains_key(end)
}

fn link_end(context: &Context, link: &str, end: &str, side: &str) -> (Option<String>, Option<(String, String)>) {
    if let Some(port) = context.resolve(end).filter(|_| names_a_port(context, end)) {
        return (Some(port.to_string()), None);
    }
    match context.resolve(end) {
        Some(node) => {
            let id = derived(&format!("{link}:{side}:port"));
            (Some(id.clone()), Some((node.to_string(), id)))
        }
        None => (None, None),
    }
}

/// The end of a physical exchange: a port as written, or a new component port
/// of the component written, since Capella ends an exchange on a port.
fn exchange_end(context: &Context, exchange: &str, end: &str, side: &str) -> (Option<String>, Option<(String, El)>) {
    let Some(resolved) = context.resolve(end) else { return (None, None) };
    if names_a_port(context, end) {
        return (Some(resolved.to_string()), None);
    }
    let id = derived(&format!("{exchange}:{side}:port"));
    let orientation = if side == "source" { "OUT" } else { "IN" };
    let port = El::new("ownedFeatures", "fa:ComponentPort", &id).attr("name", &format!("CP {}", &id[..8])).attr("orientation", orientation).attr("kind", "FLOW");
    (Some(id), Some((resolved.to_string(), port)))
}

fn placed(pa: &PhysicalArchitecture) -> Vec<Placed<'_>> {
    let mut placed = Vec::new();
    for node in &pa.nodes {
        functions::place(&node.functions, Some(&node.id), &mut placed);
        for behavior in &node.behavior_components {
            functions::place(&behavior.functions, Some(&behavior.id), &mut placed);
        }
    }
    functions::place(&pa.functions, None, &mut placed);
    placed
}

/// A port by its own id, as the compiler resolves a link's or an exchange's end.
fn register_by_id(context: &mut Context, own: &str, id: &str, owner: &str) {
    if own.is_empty() {
        return;
    }
    context.register(own, id);
    context.port_owners.insert(own.to_string(), owner.to_string());
}

pub(super) fn register(context: &mut Context, pa: &PhysicalArchitecture) {
    for node in &pa.nodes {
        context.register(&node.id, &node.id);
        context.register(&node.name, &node.id);
        for port in &node.ports {
            let id = id_of(&port.attributes, &format!("port:{}.{}", node.id, port.name));
            context.register(&format!("{}.{}", node.id, port.name), &id);
            context.register(&format!("{}.{}", node.name, port.name), &id);
            if let Some(own) = port.attributes.get("id").and_then(|v| v.as_string()) {
                register_by_id(context, own, &id, &node.id);
            }
        }
        // A node's oriented ports are component ports, as a behavior's are.
        for port in &node.component_ports {
            let id = port_id(&node.id, &port.name, &port.id);
            context.register(&format!("{}.{}", node.id, port.name), &id);
            context.register(&format!("{}.{}", node.name, port.name), &id);
            register_by_id(context, &port.id, &id, &node.id);
        }
        for behavior in &node.behavior_components {
            context.register(&behavior.id, &behavior.id);
            context.register(&behavior.name, &behavior.id);
            for port in &behavior.ports {
                let id = port_id(&behavior.id, &port.name, &port.id);
                context.register(&format!("{}.{}", behavior.id, port.name), &id);
                context.register(&format!("{}.{}", behavior.name, port.name), &id);
                register_by_id(context, &port.id, &id, &behavior.id);
            }
        }
    }
    functions::register(context, &placed(pa));
    for (index, link) in pa.links.iter().enumerate() {
        let id = link_id(link, index);
        context.register(&id, &id);
        context.register(&link.name, &id);
    }
    for exchange in &pa.physical_exchanges {
        context.register(&exchange.id, &exchange.id);
    }
    for exchange in &pa.functional_exchanges {
        context.register(&exchange.id, &exchange.id);
    }
    for chain in &pa.functional_chains {
        context.chains.insert(chain.id.clone());
        context.register(&chain.id, &chain.id);
        context.register(&chain.name, &chain.id);
    }
    for path in &pa.paths {
        context.register(&path.id, &path.id);
        context.register(&path.name, &path.id);
    }
    for realization in &pa.capability_realizations {
        context.register(&realization.id, &realization.id);
        context.register(&realization.name, &realization.id);
    }
}

fn behavior(context: &Context, b: &BehaviorComponent, placed: &[Placed<'_>]) -> El {
    let el = El::new("ownedPhysicalComponents", "pa:PhysicalComponent", &b.id).attr("name", &b.name).attr("nature", "BEHAVIOR");
    let mut el = apply(el, &b.attributes, &["allocated_component", "allocated_functions", "kind"]);
    for port in &b.ports {
        el.push(component_port(&b.id, port));
    }
    for allocation in functions::allocations(context, &b.id, placed, &b.allocated_functions) {
        el.push(allocation);
    }
    if let Some(target) = b.allocated_component.as_deref().or_else(|| text(&b.attributes, "allocated_component")).and_then(|c| context.resolve(c)) {
        el.push(super::realization("ownedComponentRealizations", "cs:ComponentRealization", &b.id, target));
    }
    el
}

fn is_actor(n: &PhysicalNode) -> bool {
    super::context::flag(&n.attributes, "actor")
}

fn node(context: &Context, n: &PhysicalNode, placed: &[Placed<'_>], link_ports: &[(String, String)]) -> El {
    let el = El::new("ownedPhysicalComponents", "pa:PhysicalComponent", &n.id).attr("name", &n.name).attr("nature", "NODE");
    let mut el = if is_actor(n) { el.attr("actor", "true") } else { el };
    el = apply(el, &n.attributes, &["kind", "type", "processor", "memory", "actor"]);
    for port in &n.ports {
        let id = id_of(&port.attributes, &format!("port:{}.{}", n.id, port.name));
        el.push(apply(El::new("ownedFeatures", "cs:PhysicalPort", &id).attr("name", &port.name), &port.attributes, &[]));
    }
    for port in &n.component_ports {
        el.push(component_port(&n.id, port));
    }
    for (owner, port) in link_ports {
        if owner == &n.id {
            el.push(El::new("ownedFeatures", "cs:PhysicalPort", port).attr("name", &format!("PP {}", &port[..8])));
        }
    }
    for allocation in functions::allocations(context, &n.id, placed, &[]) {
        el.push(allocation);
    }
    el
}

pub(super) fn write(context: &mut Context, pa: &PhysicalArchitecture, above_root: Option<&str>) -> El {
    let id = level_id(pa);
    let mut level = El::new("ownedArchitectures", "pa:PhysicalArchitecture", &id).attr("name", &pa.name);
    let placed = placed(pa);

    let root_id = derived(&format!("{id}:root-function"));
    let root = El::new("ownedPhysicalFunctions", "pa:PhysicalFunction", &root_id).attr("name", "Root Physical Function");
    let mut root = functions::tree(context, "pa:PhysicalFunction", match above_root { Some(above) => root.with(super::realization("ownedFunctionRealizations", "fa:FunctionRealization", &root_id, above)), None => root }, &placed);
    for exchange in &pa.functional_exchanges {
        if let Some(el) = functional_exchange(context, exchange) {
            root.push(el);
        }
    }
    let ends = exchange_ends(context, &pa.functional_exchanges);
    for chain in &pa.functional_chains {
        let el = apply(El::new("ownedFunctionalChains", "fa:FunctionalChain", &chain.id).attr("name", &chain.name), &chain.attributes, &["involves", "owner"]);
        root.push(relations::chain_steps(context, el, chain, &ends));
    }
    super::functions::place_exchanges(&mut root);
    // A level without functions has no root function, as in Capella's empty project.
    if root.children().iter().any(|c| !c.kind().ends_with("Realization")) {
        level.push(El::new("ownedFunctionPkg", "pa:PhysicalFunctionPkg", &derived(&format!("{id}:functions"))).attr("name", "Physical Functions").with(root));
    }

    // Links first: a link between bare nodes gives each node a port.
    let mut link_ports: Vec<(String, String)> = Vec::new();
    let mut links: Vec<El> = Vec::new();
    let mut link_nodes: std::collections::HashMap<String, (String, String)> = std::collections::HashMap::new();
    let node_of = |context: &Context, end: &str| -> Option<String> {
        if let Some(owner) = context.port_owners.get(end) {
            return context.resolve(owner).map(str::to_string);
        }
        let owner = if end.contains('.') && context.resolve(end).is_some() { end.rsplit_once('.').map(|(o, _)| o).unwrap_or(end) } else { end };
        context.resolve(owner).map(str::to_string)
    };
    // A link between oriented component ports (a node's port and a behavior
    // component's, say) is a component exchange in Capella: physical links
    // end on physical ports only.
    let mut port_exchanges: Vec<El> = Vec::new();
    for (index, link) in pa.links.iter().enumerate() {
        let lid = link_id(link, index);
        if is_component_port(pa, &link.from) || is_component_port(pa, &link.to) {
            let (source, target) = (context.resolve(&link.from).map(str::to_string), context.resolve(&link.to).map(str::to_string));
            let (Some(source), Some(target)) = (source, target) else {
                context.note(format!("physical link '{}' left out: an end is not exported", link.name));
                continue;
            };
            context.note(format!("physical link '{}' joins component ports: exported as a component exchange, which Capella requires between them", link.name));
            port_exchanges.push(El::new("ownedComponentExchanges", "fa:ComponentExchange", &lid).attr("name", &link.name).attr("target", &format!("#{target}")).attr("source", &format!("#{source}")).attr("kind", "FLOW"));
            continue;
        }
        let (from, new_from) = link_end(context, &lid, &link.from, "from");
        let (to, new_to) = link_end(context, &lid, &link.to, "to");
        let (Some(from), Some(to)) = (from, to) else {
            context.note(format!("physical link '{}' left out: an end is not exported", link.name));
            continue;
        };
        if let (Some(a), Some(b)) = (node_of(context, &link.from), node_of(context, &link.to)) {
            link_nodes.insert(lid.clone(), (a, b));
        }
        link_ports.extend(new_from.into_iter().chain(new_to));
        links.push(apply(El::new("ownedPhysicalLinks", "cs:PhysicalLink", &lid).attr("name", &link.name).attr("linkEnds", &format!("#{from} #{to}")), &link.attributes, &["from", "to", "exchange_items", "category", "categories"]));
    }

    let mut pkg = El::new("ownedPhysicalComponentPkg", "pa:PhysicalComponentPkg", &derived(&format!("{id}:components"))).attr("name", "Structure");
    let (actors, systems): (Vec<&PhysicalNode>, Vec<&PhysicalNode>) = pa.nodes.iter().partition(|n| is_actor(n));
    // The root first: Capella (and the import) take the first component as the system.
    if let Some((first, others)) = systems.split_first() {
        let mut system = node(context, first, &placed, &link_ports);
        // The root holds nodes and behaviors: it is of no nature.
        system.remove("nature");
        // Every other component is a part of the root, deployed where it is written.
        let mut parts: Vec<El> = Vec::new();
        let deploy = |part: &mut El, node: &PhysicalNode| {
            for b in &node.behavior_components {
                part.push(
                    El::new("ownedDeploymentLinks", "deployment:PartDeploymentLink", &derived(&format!("{}->{}:deployment", b.id, node.id)))
                        .attr("deployedElement", &format!("#{}", part_id(&b.id)))
                        .attr("location", &format!("#{}", part_id(&node.id))),
                );
            }
        };
        for n in others {
            let mut part = El::new("ownedFeatures", "cs:Part", &part_id(&n.id)).attr("name", &n.name).attr("abstractType", &format!("#{}", n.id));
            deploy(&mut part, n);
            parts.push(part);
        }
        for n in pa.nodes.iter() {
            for b in &n.behavior_components {
                parts.push(El::new("ownedFeatures", "cs:Part", &part_id(&b.id)).attr("name", &b.name).attr("abstractType", &format!("#{}", b.id)));
            }
        }
        for part in parts {
            system.push(part);
        }
        // Capella ends a component exchange on a port: an exchange written
        // between two components gives each end a component port.
        let mut exchanges = Vec::new();
        let mut exchange_ports: Vec<(String, El)> = Vec::new();
        for exchange in &pa.physical_exchanges {
            let label = exchange.label.clone().unwrap_or_default();
            let (source, new_source) = exchange_end(context, &exchange.id, &exchange.from, "source");
            let (target, new_target) = exchange_end(context, &exchange.id, &exchange.to, "target");
            let (Some(source), Some(target)) = (source, target) else {
                context.note(format!("physical exchange '{label}' left out: an end is not exported"));
                continue;
            };
            exchange_ports.extend(new_source.into_iter().chain(new_target));
            let el = El::new("ownedComponentExchanges", "fa:ComponentExchange", &exchange.id).attr("name", &label).attr("target", &format!("#{target}")).attr("source", &format!("#{source}")).attr("kind", "FLOW");
            exchanges.push(relations::exchange(context, el, &exchange.attributes));
            // The link carrying the exchange allocates it.
            if let Some(via) = exchange.attributes.get("via").and_then(|v| v.as_string()) {
                let carrier = pa.links.iter().enumerate().find(|(_, l)| l.name == via || l.attributes.get("id").and_then(|i| i.as_string()) == Some(via)).map(|(i, l)| link_id(l, i));
                match carrier.and_then(|lid| links.iter_mut().find(|l| l.id() == lid)) {
                    Some(link) => {
                        let lid = link.id().to_string();
                        link.push(
                            El::new("ownedComponentExchangeAllocations", "fa:ComponentExchangeAllocation", &derived(&format!("{lid}->{}:allocates", exchange.id)))
                                .attr("targetElement", &format!("#{}", exchange.id))
                                .attr("sourceElement", &format!("#{lid}")),
                        );
                    }
                    None => context.note(format!("physical exchange '{label}': its carrying link '{via}' is not exported")),
                }
            }
        }
        let with_ports = |mut el: El, owner: &str| {
            for (_, port) in exchange_ports.iter().filter(|(o, _)| o == owner) {
                el.push(port.clone());
            }
            el
        };
        for n in others {
            system.push(with_ports(node(context, n, &placed, &link_ports), &n.id));
        }
        for n in pa.nodes.iter() {
            for b in &n.behavior_components {
                system.push(with_ports(behavior(context, b, &placed), &b.id));
            }
        }
        for link in links {
            system.push(link);
        }
        for exchange in exchanges.into_iter().chain(port_exchanges) {
            system.push(exchange);
        }
        for path in &pa.paths {
            let mut el = apply(El::new("ownedPhysicalPath", "cs:PhysicalPath", &path.id).attr("name", &path.name), &path.attributes, &["involves"]);
            // Capella's path alternates the parts of the nodes and the links between them.
            let mut involved: Vec<String> = Vec::new();
            for step in &path.involves {
                let Some(link) = context.resolve(step).map(str::to_string) else { continue };
                if let Some((a, b)) = link_nodes.get(&link) {
                    let (first, second) = if involved.last() == Some(&part_id(b)) { (b, a) } else { (a, b) };
                    if involved.last() != Some(&part_id(first)) {
                        involved.push(part_id(first));
                    }
                    involved.push(link);
                    involved.push(part_id(second));
                } else {
                    involved.push(link);
                }
            }
            let steps: Vec<(String, String)> = involved.into_iter().enumerate().map(|(i, target)| (derived(&format!("{}:{i}", path.id)), target)).collect();
            for (i, (involvement, link)) in steps.iter().enumerate() {
                let mut step = El::new("ownedPhysicalPathInvolvements", "cs:PhysicalPathInvolvement", involvement).attr("involved", &format!("#{link}"));
                if let Some((next, _)) = steps.get(i + 1) {
                    step.set("nextInvolvements", &format!("#{next}"));
                }
                el.push(step);
            }
            system.push(el);
        }
        pkg.push(system);
        for actor in &actors {
            pkg.push(node(context, actor, &placed, &link_ports));
        }
        pkg.push(El::new("ownedParts", "cs:Part", &part_id(&first.id)).attr("name", &first.name).attr("abstractType", &format!("#{}", first.id)));
        for actor in &actors {
            // An actor's part holds the deployments of the behaviors written in it.
            let mut part = El::new("ownedParts", "cs:Part", &part_id(&actor.id)).attr("name", &actor.name).attr("abstractType", &format!("#{}", actor.id));
            for b in &actor.behavior_components {
                part.push(
                    El::new("ownedDeploymentLinks", "deployment:PartDeploymentLink", &derived(&format!("{}->{}:deployment", b.id, actor.id)))
                        .attr("deployedElement", &format!("#{}", part_id(&b.id)))
                        .attr("location", &format!("#{}", part_id(&actor.id))),
                );
            }
            pkg.push(part);
        }
    } else if !pa.links.is_empty() || !pa.physical_exchanges.is_empty() {
        context.note("physical links and exchanges left out: the physical architecture has no node".to_string());
    }
    level.push(pkg);

    let mut realizations = El::new("ownedAbstractCapabilityPkg", "la:CapabilityRealizationPkg", &derived(&format!("{id}:capabilities"))).attr("name", "Capabilities");
    let chain_ids: Vec<&str> = pa.functional_chains.iter().map(|c| c.id.as_str()).collect();
    for realization in &pa.capability_realizations {
        realizations.push(capability_realization(context, realization, &chain_ids, "la:CapabilityRealization"));
    }
    level.push(realizations);
    level
}

/// An empty physical architecture.
pub(super) fn empty(project_id: &str) -> El {
    // As Capella's own empty project: no package, no root function.
    El::new("ownedArchitectures", "pa:PhysicalArchitecture", &derived(&format!("{project_id}:pa"))).attr("name", "Physical Architecture")
}
