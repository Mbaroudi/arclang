//! The logical architecture: components (nested, with their parts, ports
//! and allocated functions), the tree of logical functions, functional and
//! component exchanges, functional chains and capability realizations.

use super::context::{derived, list, port_id, text, Context};
use super::functions::{self, Placed};
use super::header::apply;
use super::relations;
use super::system::component_port;
use super::xml::El;
use crate::compiler::ast::{ComponentExchange, FunctionalExchange, LogicalArchitecture, LogicalComponent};
use std::collections::HashMap;

fn level_id(la: &LogicalArchitecture) -> String {
    if !la.id.is_empty() {
        return la.id.clone();
    }
    derived(&format!("logical-architecture:{}", la.name))
}

fn placed(la: &LogicalArchitecture) -> Vec<Placed<'_>> {
    fn components<'a>(list: &'a [LogicalComponent], into: &mut Vec<Placed<'a>>) {
        for c in list {
            functions::place(&c.functions, Some(&c.id), into);
            components(&c.sub_components, into);
        }
    }
    let mut placed = Vec::new();
    components(&la.components, &mut placed);
    functions::place(&la.functions, None, &mut placed);
    placed
}

fn register_components(context: &mut Context, list: &[LogicalComponent]) {
    for c in list {
        context.register(&c.id, &c.id);
        context.register(&c.name, &c.id);
        for port in &c.ports {
            let id = port_id(&c.id, &port.name, &port.id);
            context.register(&format!("{}.{}", c.id, port.name), &id);
            context.register(&format!("{}.{}", c.name, port.name), &id);
        }
        register_components(context, &c.sub_components);
    }
}

pub(super) fn register(context: &mut Context, la: &LogicalArchitecture) {
    register_components(context, &la.components);
    functions::register(context, &placed(la));
    for exchange in &la.functional_exchanges {
        context.register(&exchange.id, &exchange.id);
    }
    for exchange in &la.component_exchanges {
        context.register(&exchange.id, &exchange.id);
    }
    for chain in &la.functional_chains {
        context.chains.insert(chain.id.clone());
        context.register(&chain.id, &chain.id);
        context.register(&chain.name, &chain.id);
    }
    for realization in &la.capability_realizations {
        context.register(&realization.id, &realization.id);
        context.register(&realization.name, &realization.id);
    }
}

fn component(context: &Context, c: &LogicalComponent, placed: &[Placed<'_>]) -> El {
    let mut el = El::new("ownedLogicalComponents", "la:LogicalComponent", &c.id).attr("name", &c.name);
    if super::context::flag(&c.attributes, "actor") {
        el.set("actor", "true");
    }
    let mut el = apply(el, &c.attributes, &["actor", "allocated_functions", "kind", "type"]);
    for port in &c.ports {
        el.push(component_port(&c.id, port));
    }
    for allocation in functions::allocations(context, &c.id, placed, &c.allocated_functions) {
        el.push(allocation);
    }
    for (interfaces, tag, kind, reference) in [
        (&c.interfaces_out, "ownedInterfaceImplementations", "cs:InterfaceImplementation", "implementedInterface"),
        (&c.interfaces_in, "ownedInterfaceUses", "cs:InterfaceUse", "usedInterface"),
    ] {
        for interface in interfaces {
            if let Some(target) = context.resolve(&interface.name) {
                el.push(El::new(tag, kind, &derived(&format!("{}->{target}:{reference}", c.id))).attr(reference, &format!("#{target}")));
            }
        }
    }
    for sub in &c.sub_components {
        el.push(El::new("ownedFeatures", "cs:Part", &derived(&format!("{}:part", sub.id))).attr("name", &sub.name).attr("abstractType", &format!("#{}", sub.id)));
    }
    for sub in &c.sub_components {
        el.push(component(context, sub, placed));
    }
    el
}

/// A functional exchange between two function ports (or functions).
pub(super) fn functional_exchange(context: &mut Context, exchange: &FunctionalExchange) -> Option<El> {
    let label = exchange.label.clone().unwrap_or_default();
    match (context.resolve(&exchange.from_port).map(str::to_string), context.resolve(&exchange.to_port).map(str::to_string)) {
        (Some(source), Some(target)) => {
            let el = El::new("ownedFunctionalExchanges", "fa:FunctionalExchange", &exchange.id).attr("name", &label).attr("target", &format!("#{target}")).attr("source", &format!("#{source}"));
            Some(relations::exchange(context, el, &exchange.attributes))
        }
        _ => {
            context.note(format!("functional exchange '{label}' left out: an end ('{}' or '{}') is not exported", exchange.from_port, exchange.to_port));
            None
        }
    }
}

/// A component exchange between two component ports.
pub(super) fn component_exchange(context: &mut Context, exchange: &ComponentExchange) -> Option<El> {
    let label = exchange.label.clone().unwrap_or_default();
    match (context.resolve(&exchange.from_port).map(str::to_string), context.resolve(&exchange.to_port).map(str::to_string)) {
        (Some(source), Some(target)) => {
            let el = El::new("ownedComponentExchanges", "fa:ComponentExchange", &exchange.id).attr("name", &label).attr("target", &format!("#{target}")).attr("source", &format!("#{source}")).attr("kind", "FLOW");
            Some(relations::exchange(context, el, &exchange.attributes))
        }
        _ => {
            context.note(format!("component exchange '{label}' left out: an end ('{}' or '{}') is not exported", exchange.from_port, exchange.to_port));
            None
        }
    }
}

/// The two functions an exchange joins, for the chains.
pub(super) fn exchange_ends(context: &Context, exchanges: &[FunctionalExchange]) -> HashMap<String, (String, String)> {
    let owner = |port: &str| port.rsplit_once('.').map(|(f, _)| f.to_string()).unwrap_or_else(|| port.to_string());
    exchanges
        .iter()
        .filter_map(|e| Some((e.id.clone(), (context.resolve(&owner(&e.from_port))?.to_string(), context.resolve(&owner(&e.to_port))?.to_string()))))
        .collect()
}

pub(super) fn write(context: &mut Context, la: &LogicalArchitecture, above_root: Option<&str>) -> El {
    let id = level_id(la);
    let mut level = El::new("ownedArchitectures", "la:LogicalArchitecture", &id).attr("name", &la.name);
    let placed = placed(la);

    let root_id = derived(&format!("{id}:root-function"));
    let root = El::new("ownedLogicalFunctions", "la:LogicalFunction", &root_id).attr("name", "Root Logical Function");
    let mut root = functions::tree(context, "la:LogicalFunction", match above_root { Some(above) => root.with(super::realization("ownedFunctionRealizations", "fa:FunctionRealization", &root_id, above)), None => root }, &placed);
    for exchange in &la.functional_exchanges {
        if let Some(el) = functional_exchange(context, exchange) {
            root.push(el);
        }
    }
    let ends = exchange_ends(context, &la.functional_exchanges);
    for chain in &la.functional_chains {
        let el = apply(El::new("ownedFunctionalChains", "fa:FunctionalChain", &chain.id).attr("name", &chain.name), &chain.attributes, &["involves"]);
        root.push(relations::chain_steps(context, el, chain, &ends));
    }
    super::functions::place_exchanges(&mut root);
    // A level without functions has no root function, as in Capella's empty project.
    if root.children().iter().any(|c| !c.kind().ends_with("Realization")) {
        level.push(El::new("ownedFunctionPkg", "la:LogicalFunctionPkg", &derived(&format!("{id}:functions"))).attr("name", "Logical Functions").with(root));
    }

    let mut pkg = El::new("ownedLogicalComponentPkg", "la:LogicalComponentPkg", &derived(&format!("{id}:components"))).attr("name", "Structure");
    for c in &la.components {
        pkg.push(component(context, c, &placed));
    }
    for exchange in &la.component_exchanges {
        if let Some(el) = component_exchange(context, exchange) {
            pkg.push(el);
        }
    }
    for c in &la.components {
        pkg.push(El::new("ownedParts", "cs:Part", &derived(&format!("{}:part", c.id))).attr("name", &c.name).attr("abstractType", &format!("#{}", c.id)));
    }
    level.push(pkg);

    let mut realizations = El::new("ownedAbstractCapabilityPkg", "la:CapabilityRealizationPkg", &derived(&format!("{id}:capabilities"))).attr("name", "Capabilities");
    let chain_ids: Vec<&str> = la.functional_chains.iter().map(|c| c.id.as_str()).collect();
    for realization in &la.capability_realizations {
        realizations.push(capability_realization(context, realization, &chain_ids, "la:CapabilityRealization"));
    }
    level.push(realizations);
    level
}

/// A capability realization of the logical or physical level.
pub(super) fn capability_realization(context: &Context, realization: &crate::compiler::ast::Capability, chains: &[&str], kind: &str) -> El {
    let mut el = apply(
        El::new("ownedCapabilityRealizations", kind, &realization.id).attr("name", &realization.name),
        &realization.attributes,
        &["involves", "realizes", "extends", "includes", "specializes", "parent", "mission"],
    );
    for involved in &realization.involves {
        let Some(target) = context.resolve(involved).map(str::to_string) else { continue };
        let (tag, kind) = if chains.contains(&target.as_str()) {
            ("ownedFunctionalChainAbstractCapabilityInvolvements", "interaction:FunctionalChainAbstractCapabilityInvolvement")
        } else {
            ("ownedCapabilityRealizationInvolvements", "capellacommon:CapabilityRealizationInvolvement")
        };
        el.push(El::new(tag, kind, &derived(&format!("{}->{target}:involvement", realization.id))).attr("involved", &format!("#{target}")));
    }
    let realizes = realization.realizes.clone().or_else(|| text(&realization.attributes, "realizes").map(str::to_string));
    let _ = list;
    relations::capability(context, el, &realization.id, &realization.attributes, realizes.as_deref())
}

/// An empty logical architecture.
pub(super) fn empty(project_id: &str) -> El {
    // As Capella's own empty project: no package, no root function.
    El::new("ownedArchitectures", "la:LogicalArchitecture", &derived(&format!("{project_id}:la"))).attr("name", "Logical Architecture")
}
