//! What each export strategy adds to the elements chosen (`ExportHandler.computeScope`
//! and its visitors): a list without duplicates, each element visited once, in
//! the order the visitors add them.

use super::model::Model;
use super::{function, Strategy};

struct Collector {
    items: Vec<usize>,
}

impl Collector {
    fn add(&mut self, e: Option<usize>) {
        if let Some(e) = e {
            if !self.items.contains(&e) {
                self.items.push(e);
            }
        }
    }

    fn add_all(&mut self, list: impl IntoIterator<Item = usize>) {
        for e in list {
            self.add(Some(e));
        }
    }
}

/// The selection the strategy transforms.
pub fn compute(m: &Model, strategy: Strategy, targets: &[usize]) -> Result<Vec<usize>, String> {
    let mut c = Collector { items: Vec::new() };
    c.add_all(targets.iter().copied());
    let mut visited: Vec<usize> = Vec::new();
    let mut n = 0;
    while n < c.items.len() {
        let e = c.items[n];
        n += 1;
        if visited.contains(&e) {
            continue;
        }
        match strategy {
            Strategy::Functions => {
                function_visitor(m, &mut c, e, true);
                port_visitor(m, &mut c, e);
            }
            Strategy::FunctionalChains => {
                chain_visitor(m, &mut c, e);
                function_visitor(m, &mut c, e, false);
            }
            Strategy::StateMachines => state_machine_visitor(m, &mut c, e)?,
            Strategy::DataPackages => {
                package_visitor(m, &mut c, e);
                type_visitor(m, &mut c, e);
                property_visitor(m, &mut c, e);
                exchange_item_visitor(m, &mut c, e);
            }
        }
        visited.push(e);
    }
    Ok(c.items)
}

/// `FunctionVisitor`: a function's ports, and its sub-functions when asked.
fn function_visitor(m: &Model, c: &mut Collector, e: usize, sub_functions: bool) {
    if m.is(e, "AbstractFunction") {
        c.add_all(function::ports_of(m, e));
        if sub_functions {
            c.add_all(function::sub_functions(m, e));
        }
    }
}

/// `PortVisitor`: a port's functional exchanges.
fn port_visitor(m: &Model, c: &mut Collector, e: usize) {
    if m.is(e, "FunctionInputPort") || m.is(e, "FunctionOutputPort") {
        c.add_all(function::exchanges_of_port(m, e));
    }
}

/// `FunctionalChainVisitor`: the components that perform the chain (and theirs, by name), its
/// functions, its exchanges.
fn chain_visitor(m: &Model, c: &mut Collector, e: usize) {
    if !m.is(e, "FunctionalChain") {
        return;
    }
    let mut components = function::chain_components(m, e);
    components.sort_by_key(|&k| m.name(k).unwrap_or_default());
    c.add_all(components);
    c.add_all(function::chain_functions(m, e));
    c.add_all(function::chain_exchanges(m, e));
}

/// `StateMachineVisitor`: a machine's regions and the diagram of each, a region's states and
/// transitions, a state's regions, a transition's triggers.
fn state_machine_visitor(m: &Model, c: &mut Collector, e: usize) -> Result<(), String> {
    if m.is(e, "StateMachine") {
        for region in m.owned(e, "ownedRegions") {
            c.add(Some(region));
            let diagram = super::diagram::diagram_of(m, region).ok_or_else(|| format!("Cannot find diagram for region with id {}", m.id(region).unwrap_or_default()))?;
            c.add(Some(diagram));
        }
    } else if m.is(e, "Region") {
        c.add_all(m.refs(e, "involvedStates"));
        c.add_all(m.owned(e, "ownedTransitions"));
    } else if m.is(e, "State") {
        c.add_all(m.owned(e, "ownedRegions"));
    } else if m.is(e, "StateTransition") {
        c.add_all(m.refs(e, "triggers"));
    }
    Ok(())
}

/// `PackageVisitor`: a data package's packages, classes, types and exchange items; an
/// interface package's packages and interfaces.
fn package_visitor(m: &Model, c: &mut Collector, e: usize) {
    if m.is(e, "DataPkg") {
        for feature in ["ownedDataPkgs", "ownedClasses", "ownedDataTypes", "ownedExchangeItems"] {
            c.add_all(m.owned(e, feature));
        }
    } else if m.is(e, "InterfacePkg") {
        c.add_all(m.owned(e, "ownedInterfacePkgs"));
        c.add_all(m.owned(e, "ownedInterfaces"));
    }
}

/// `TypeVisitor`: a classifier's properties, an enumeration's literals.
fn type_visitor(m: &Model, c: &mut Collector, e: usize) {
    if m.is(e, "Classifier") {
        c.add_all(super::data::contained_properties(m, e));
    } else if m.is(e, "GeneralClass") {
        c.add_all(m.owned(e, "nestedGeneralClasses"));
    } else if m.is(e, "Enumeration") {
        c.add_all(m.owned(e, "ownedLiterals"));
    }
}

/// `PropertyVisitor`: the type of a composition, or of a navigable association's end.
fn property_visitor(m: &Model, c: &mut Collector, e: usize) {
    if m.class(e) != "Property" {
        return;
    }
    match m.attr(e, "aggregationKind").unwrap_or("UNSET") {
        "COMPOSITION" => c.add(m.reference(e, "abstractType")),
        // `getAssociation()` is the first association that has the property among its
        // navigable members, so it always has it.
        "ASSOCIATION" => {
            if m.referencers(e, "navigableMembers").into_iter().any(|a| m.is(a, "Association")) {
                c.add(m.reference(e, "abstractType"));
            }
        }
        _ => {}
    }
}

/// `ExchangeItemVisitor`: an exchange item's elements, and their class, enumeration or
/// physical quantity.
fn exchange_item_visitor(m: &Model, c: &mut Collector, e: usize) {
    if m.is(e, "ExchangeItem") {
        c.add_all(m.owned(e, "ownedElements"));
    } else if m.is(e, "ExchangeItemElement") {
        if let Some(t) = m.reference(e, "abstractType") {
            if m.is(t, "Class") || m.is(t, "Enumeration") || m.is(t, "PhysicalQuantity") {
                c.add(Some(t));
            }
        }
    }
}
