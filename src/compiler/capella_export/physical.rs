//! The physical architecture: nodes with their physical ports, behaviour
//! components deployed on them with their ports and the logical component
//! each one realizes, the tree of physical functions, physical links and
//! paths, component exchanges, functional chains and capability realizations.
//!
//! Capella keeps one root physical component; the first node is that root,
//! and every other component is its sub-component, through a part. A
//! behaviour component written in a node is deployed on that node.

use super::context::{derived, id_of, port_id, text, Context};
use super::functions::{self, Placed};
use super::header::apply;
use super::logical::{capability_realization, exchange_ends, functional_exchange};
use super::relations;
use super::system::component_port;
use super::xml::El;
use crate::compiler::ast::{BehaviorComponent, PhysicalArchitecture, PhysicalLink, PhysicalNode};

fn level_id(pa: &PhysicalArchitecture) -> String {
    derived(&format!("physical-architecture:{}", pa.name))
}

fn part_id(component: &str) -> String {
    derived(&format!("{component}:part"))
}

fn link_id(link: &PhysicalLink, index: usize) -> String {
    id_of(&link.attributes, &format!("link:{index}:{}", link.name))
}

/// The port of a link end: `Node.port` names one; a bare node gets a port of its own for the link.
fn link_end(context: &Context, link: &str, end: &str, side: &str) -> (Option<String>, Option<(String, String)>) {
    if let Some(port) = context.resolve(end).filter(|_| end.contains('.')) {
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

fn placed(pa: &PhysicalArchitecture) -> Vec<Placed<'_>> {
    let mut placed = Vec::new();
    for node in &pa.nodes {
        functions::place(&node.functions, Some(&node.id), &mut placed);
        for behaviour in &node.behavior_components {
            functions::place(&behaviour.functions, Some(&behaviour.id), &mut placed);
        }
    }
    functions::place(&pa.functions, None, &mut placed);
    placed
}

pub(super) fn register(context: &mut Context, pa: &PhysicalArchitecture) {
    for node in &pa.nodes {
        context.register(&node.id, &node.id);
        context.register(&node.name, &node.id);
        for port in &node.ports {
            let id = id_of(&port.attributes, &format!("port:{}.{}", node.id, port.name));
            context.register(&format!("{}.{}", node.id, port.name), &id);
            context.register(&format!("{}.{}", node.name, port.name), &id);
        }
        for behaviour in &node.behavior_components {
            context.register(&behaviour.id, &behaviour.id);
            context.register(&behaviour.name, &behaviour.id);
            for port in &behaviour.ports {
                let id = port_id(&behaviour.id, &port.name);
                context.register(&format!("{}.{}", behaviour.id, port.name), &id);
                context.register(&format!("{}.{}", behaviour.name, port.name), &id);
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

fn behaviour(context: &Context, b: &BehaviorComponent, placed: &[Placed<'_>]) -> El {
    let el = El::new("ownedPhysicalComponents", "pa:PhysicalComponent", &b.id).attr("name", &b.name).attr("nature", "BEHAVIOR");
    let mut el = apply(el, &b.attributes, &["allocated_component", "allocated_functions", "kind"]);
    for port in &b.ports {
        el.push(component_port(&b.id, port));
    }
    for allocation in functions::allocations(context, &b.id, placed, &b.allocated_functions) {
        el.push(allocation);
    }
    if let Some(target) = text(&b.attributes, "allocated_component").and_then(|c| context.resolve(c)) {
        el.push(super::realization("ownedComponentRealizations", "cs:ComponentRealization", &b.id, target));
    }
    el
}

fn node(context: &Context, n: &PhysicalNode, placed: &[Placed<'_>], link_ports: &[(String, String)]) -> El {
    let el = El::new("ownedPhysicalComponents", "pa:PhysicalComponent", &n.id).attr("name", &n.name).attr("nature", "NODE");
    let mut el = apply(el, &n.attributes, &["kind", "type", "processor", "memory"]);
    for port in &n.ports {
        let id = id_of(&port.attributes, &format!("port:{}.{}", n.id, port.name));
        el.push(apply(El::new("ownedFeatures", "cs:PhysicalPort", &id).attr("name", &port.name), &port.attributes, &[]));
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

pub(super) fn write(context: &mut Context, pa: &PhysicalArchitecture, above_root: &str) -> El {
    let id = level_id(pa);
    let mut level = El::new("ownedArchitectures", "pa:PhysicalArchitecture", &id).attr("name", &pa.name);
    let placed = placed(pa);

    let root_id = derived(&format!("{id}:root-function"));
    let root = El::new("ownedPhysicalFunctions", "pa:PhysicalFunction", &root_id).attr("name", "Root Physical Function");
    let mut root = functions::tree(context, "pa:PhysicalFunction", root.with(super::realization("ownedFunctionRealizations", "fa:FunctionRealization", &root_id, above_root)), &placed);
    for exchange in &pa.functional_exchanges {
        if let Some(el) = functional_exchange(context, exchange) {
            root.push(el);
        }
    }
    let ends = exchange_ends(context, &pa.functional_exchanges);
    for chain in &pa.functional_chains {
        let el = apply(El::new("ownedFunctionalChains", "fa:FunctionalChain", &chain.id).attr("name", &chain.name), &chain.attributes, &["involves"]);
        root.push(relations::chain_steps(context, el, &chain.id, &chain.involves, &ends));
    }
    level.push(El::new("ownedFunctionPkg", "pa:PhysicalFunctionPkg", &derived(&format!("{id}:functions"))).attr("name", "Physical Functions").with(root));

    // Links first: a link between bare nodes gives each node a port.
    let mut link_ports: Vec<(String, String)> = Vec::new();
    let mut links: Vec<El> = Vec::new();
    let mut link_nodes: std::collections::HashMap<String, (String, String)> = std::collections::HashMap::new();
    let node_of = |context: &Context, end: &str| -> Option<String> {
        let owner = if end.contains('.') && context.resolve(end).is_some() { end.rsplit_once('.').map(|(o, _)| o).unwrap_or(end) } else { end };
        context.resolve(owner).map(str::to_string)
    };
    for (index, link) in pa.links.iter().enumerate() {
        let lid = link_id(link, index);
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
        links.push(apply(El::new("ownedPhysicalLinks", "cs:PhysicalLink", &lid).attr("name", &link.name).attr("linkEnds", &format!("#{from} #{to}")), &link.attributes, &["from", "to", "protocol", "bandwidth", "exchange_items", "category", "categories"]));
    }

    let mut pkg = El::new("ownedPhysicalComponentPkg", "pa:PhysicalComponentPkg", &derived(&format!("{id}:components"))).attr("name", "Structure");
    if let Some((first, others)) = pa.nodes.split_first() {
        let mut system = node(context, first, &placed, &link_ports);
        // The root holds nodes and behaviours: it is of no nature.
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
        for n in others {
            system.push(node(context, n, &placed, &link_ports));
        }
        for n in pa.nodes.iter() {
            for b in &n.behavior_components {
                system.push(behaviour(context, b, &placed));
            }
        }
        for link in links {
            system.push(link);
        }
        for exchange in &pa.physical_exchanges {
            let label = exchange.label.clone().unwrap_or_default();
            match (context.resolve(&exchange.from).map(str::to_string), context.resolve(&exchange.to).map(str::to_string)) {
                (Some(source), Some(target)) => system.push(
                    El::new("ownedComponentExchanges", "fa:ComponentExchange", &exchange.id).attr("name", &label).attr("target", &format!("#{target}")).attr("source", &format!("#{source}")).attr("kind", "FLOW"),
                ),
                _ => context.note(format!("physical exchange '{label}' left out: an end is not exported")),
            }
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
        pkg.push(El::new("ownedParts", "cs:Part", &part_id(&first.id)).attr("name", &first.name).attr("abstractType", &format!("#{}", first.id)));
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
pub(super) fn empty(project_id: &str, above_root: &str) -> El {
    let id = derived(&format!("{project_id}:pa"));
    let root_id = derived(&format!("{id}:root-function"));
    let root = El::new("ownedPhysicalFunctions", "pa:PhysicalFunction", &root_id)
        .attr("name", "Root Physical Function")
        .with(super::realization("ownedFunctionRealizations", "fa:FunctionRealization", &root_id, above_root));
    El::new("ownedArchitectures", "pa:PhysicalArchitecture", &id)
        .attr("name", "Physical Architecture")
        .with(El::new("ownedFunctionPkg", "pa:PhysicalFunctionPkg", &derived(&format!("{id}:functions"))).attr("name", "Physical Functions").with(root))
}
