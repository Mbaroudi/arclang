//! Functions, their ports and exchanges, functional chains: `FunctionRule`,
//! `FunctionPortRule`, `FunctionalExchangeRule`, `FunctionalChainRule` and their
//! helpers (`FunctionTransformationHelper`, `FunctionalExchangeHelper`). A function
//! becomes a Simulink model (a `System`) or a subsystem of its parent's, its ports
//! become ports, propagated up to the model that shows the exchange, and each
//! exchange becomes a line.

use super::super::pivot::Kind;
use super::model::Model;
use super::transposer::{Premise, Rule};
use super::{data, Key, Run, Target};

// ---- What Capella derives ------------------------------------------------------------

/// `FunctionExt.getOwnedFunctionPorts`: the inputs, then the outputs.
pub fn ports_of(m: &Model, f: usize) -> Vec<usize> {
    let mut out: Vec<usize> = m.owned(f, "inputs").into_iter().filter(|&p| m.is(p, "Port")).collect();
    out.extend(m.owned(f, "outputs").into_iter().filter(|&p| m.is(p, "Port")));
    out
}

/// `AbstractFunction.getOwnedFunctions`.
fn owned_functions(m: &Model, f: usize) -> Vec<usize> {
    m.owned(f, "ownedFunctions")
}

/// The function packages a function or a package owns.
fn function_packages(m: &Model, e: usize) -> Vec<usize> {
    m.elements[e].contents.iter().copied().filter(|&c| m.is(c, "FunctionPkg")).collect()
}

fn package_functions(m: &Model, pkg: usize) -> Vec<usize> {
    let mut out: Vec<usize> = m.elements[pkg].contents.iter().copied().filter(|&c| m.is(c, "AbstractFunction")).collect();
    for sub in function_packages(m, pkg) {
        out.extend(package_functions(m, sub));
    }
    out
}

/// `AbstractFunction.getSubFunctions` (`FunctionExt.getFirstLevelAbstractFunctions`): the owned
/// functions, then those of the owned function packages.
pub fn sub_functions(m: &Model, f: usize) -> Vec<usize> {
    let mut out = owned_functions(m, f);
    for pkg in function_packages(m, f) {
        out.extend(package_functions(m, pkg));
    }
    out
}

fn is_function(m: &Model, e: Option<usize>) -> bool {
    e.is_some_and(|e| m.is(e, "AbstractFunction"))
}

/// The functional exchanges that end at an input port, or start at an output port.
pub fn exchanges_of_port(m: &Model, port: usize) -> Vec<usize> {
    let feature = if m.is(port, "FunctionInputPort") { "target" } else { "source" };
    m.referencers(port, feature).into_iter().filter(|&e| m.is(e, "FunctionalExchange")).collect()
}

fn source_port(m: &Model, fe: usize) -> Option<usize> {
    m.reference(fe, "source").filter(|&p| m.is(p, "FunctionOutputPort"))
}

fn target_port(m: &Model, fe: usize) -> Option<usize> {
    m.reference(fe, "target").filter(|&p| m.is(p, "FunctionInputPort"))
}

fn exchanged_items(m: &Model, fe: usize) -> Vec<usize> {
    m.refs(fe, "exchangedItems")
}

/// The involvements of a chain and of the chains it involves (`getFlatInvolvements`).
fn flat_involvements(m: &Model, chain: usize) -> Vec<usize> {
    let mut out = Vec::new();
    let mut to_visit = vec![chain];
    let mut visited = Vec::new();
    while !to_visit.is_empty() {
        let c = to_visit.remove(0);
        if visited.contains(&c) {
            continue;
        }
        visited.push(c);
        for involvement in m.owned(c, "ownedFunctionalChainInvolvements") {
            if let Some(involved) = m.reference(involvement, "involved") {
                if m.is(involved, "FunctionalChain") {
                    to_visit.push(involved);
                }
            }
            out.push(involvement);
        }
    }
    out
}

fn flat_involved(m: &Model, chain: usize, class: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for involvement in flat_involvements(m, chain) {
        if let Some(e) = m.reference(involvement, "involved").filter(|&e| m.is(e, class)) {
            if !out.contains(&e) {
                out.push(e);
            }
        }
    }
    out
}

fn involved(m: &Model, chain: usize, class: &str) -> Vec<usize> {
    m.owned(chain, "ownedFunctionalChainInvolvements").into_iter().filter_map(|i| m.reference(i, "involved")).filter(|&e| m.is(e, class)).collect()
}

/// `FunctionalChain.getInvolvedFunctions`.
pub fn chain_functions(m: &Model, chain: usize) -> Vec<usize> {
    involved(m, chain, "AbstractFunction")
}

/// `FunctionalChain.getInvolvedFunctionalExchanges`.
pub fn chain_exchanges(m: &Model, chain: usize) -> Vec<usize> {
    involved(m, chain, "FunctionalExchange")
}

/// The components that perform a chain's functions and, recursively, the components their
/// parts are in (`FunctionalChain_enactedComponents`, `Component_parentComponent`).
pub fn chain_components(m: &Model, chain: usize) -> Vec<usize> {
    let mut direct = Vec::new();
    for f in chain_functions(m, chain) {
        for allocation in m.referencers(f, "targetElement") {
            if m.is(allocation, "ComponentFunctionalAllocation") {
                if let Some(block) = m.reference(allocation, "sourceElement").filter(|&b| m.is(b, "Component")) {
                    direct.push(block);
                }
            }
        }
    }
    let mut all: Vec<usize> = Vec::new();
    for &c in &direct {
        if !all.contains(&c) {
            all.push(c);
        }
    }
    let mut pending = direct;
    while !pending.is_empty() {
        let mut parents = Vec::new();
        for &c in &pending {
            for part in m.referencers(c, "abstractType") {
                if m.is(part, "Part") {
                    if let Some(parent) = m.container(part).and_then(|p| m.ancestor(p, "Component")) {
                        parents.push(parent);
                    }
                }
            }
        }
        let done = pending.clone();
        pending.extend(parents.iter().copied());
        pending.retain(|c| !done.contains(c));
        for p in parents {
            if !all.contains(&p) {
                all.push(p);
            }
        }
    }
    all
}

// ---- Premises ------------------------------------------------------------------------

pub fn premises(run: &Run, rule: Rule, e: usize) -> Vec<Premise> {
    let m = run.m;
    let mut out = Vec::new();
    match rule {
        Rule::Function => {
            if !transform_to_system(run, e) {
                out.push((m.container(e), true));
            }
            if run.options.leaf_as_model() {
                for sub in sub_functions(m, e) {
                    if owned_functions(m, sub).is_empty() {
                        out.push((Some(sub), false));
                    }
                }
            }
        }
        Rule::FunctionPort => {
            let mut parent = m.container(e);
            out.push((parent, true));
            parent = parent.and_then(|p| m.container(p));
            while is_function(m, parent) {
                out.push((parent, true));
                parent = parent.and_then(|p| m.container(p));
            }
            for fe in exchanges_of_port(m, e) {
                for ei in exchanged_items(m, fe) {
                    out.push((Some(ei), false));
                }
            }
        }
        Rule::FunctionalExchange => {
            for ei in exchanged_items(m, e) {
                out.push((Some(ei), false));
            }
            out.push((source_port(m, e), false));
            out.push((target_port(m, e), false));
        }
        Rule::FunctionalChain => {
            for f in flat_involved(m, e, "AbstractFunction") {
                out.push((Some(f), false));
                for pin in m.owned(f, "outputs").into_iter().chain(m.owned(f, "inputs")) {
                    out.push((Some(pin), false));
                }
            }
        }
        _ => {}
    }
    out
}

/// `FunctionRule.transformToSystem`: a function becomes a model when its container is not
/// transformed, or when it is a leaf and leaves are models.
fn transform_to_system(run: &Run, f: usize) -> bool {
    if !run.selected(run.m.container(f)) {
        return true;
    }
    run.options.leaf_as_model() && owned_functions(run.m, f).is_empty()
}

// ---- Rules ---------------------------------------------------------------------------

pub fn create(run: &mut Run, rule: Rule, e: usize) -> Result<Option<Target>, String> {
    Ok(match rule {
        Rule::Function => Some(Target::P(create_function(run, e)?)),
        Rule::FunctionPort => Some(Target::Ports(create_ports(run, e)?)),
        Rule::FunctionalExchange => {
            create_exchange_line(run, e)?;
            None
        }
        Rule::FunctionalChain => Some(Target::P(create_chain(run, e)?)),
        _ => None,
    })
}

fn create_function(run: &mut Run, f: usize) -> Result<usize, String> {
    let m = run.m;
    let name = m.name(f).unwrap_or_default();
    let result = if transform_to_system(run, f) {
        let system = run.new_obj(Kind::System, None, None);
        if let Some(layer) = run.layer(f) {
            run.set_layer(system, layer);
        }
        let project = run.project;
        run.contain(project, system, |o| &mut o.systems);
        system
    } else {
        let Some(parent) = run.main_opt(m.container(f)) else { return Err(format!("Target parent for Function {name} can not be found")) };
        if !run.p[parent].kind.is_diagram() {
            return Err(format!("Problem occured when transforming the Function {name}"));
        }
        let subsystem = run.new_obj(Kind::Subsystem, None, None);
        run.p[subsystem].source = Some("CapellaElements/Subsystem/Function".to_string());
        run.contain(parent, subsystem, |o| &mut o.blocks);
        subsystem
    };
    run.p[result].originator = m.id(f);
    run.p[result].name = m.name(f);
    run.set_main(Key::E(f), result);
    if run.p[result].kind == Kind::System {
        run.create_default_dictionaries(f, result)?;
    }
    if run.options.leaf_as_model() {
        create_system_references(run, f, result)?;
    }
    if enable_block(run, f) {
        let port = run.new_obj(Kind::Port, Some(format!("EN_{name}")), None);
        run.p[port].id = Some(format!("enable_{}", m.id(f).unwrap_or_else(|| "null".to_string())));
        run.p[port].port_kind = "Enable".to_string();
        run.p[port].description = Some(states_description(m, f));
        run.contain(result, port, |o| &mut o.ports);
        let selection = run.selection.clone();
        if run.options.leaf_as_subsystem() && run.selected(m.container(f)) {
            propagate(run, port, f, &selection)?;
        }
        run.set_param(port, "AttributesFormatString", "Available in Modes & States: %<Description>");
        run.set_param(port, "ShowName", "off");
    }
    Ok(result)
}

fn available_in_states(m: &Model, f: usize) -> Vec<usize> {
    m.refs(f, "availableInStates")
}

fn enable_block(run: &Run, f: usize) -> bool {
    !available_in_states(run.m, f).is_empty() && (owned_functions(run.m, f).is_empty() || !run.selected(run.m.container(f)))
}

fn states_description(m: &Model, f: usize) -> String {
    available_in_states(m, f).iter().map(|&s| m.name(s).unwrap_or_else(|| "null".to_string())).collect::<Vec<_>>().join(", ")
}

/// A reference to the model of each leaf sub-function already made.
fn create_system_references(run: &mut Run, f: usize, diagram: usize) -> Result<(), String> {
    let m = run.m;
    for sub in sub_functions(m, f) {
        if !owned_functions(m, sub).is_empty() {
            continue;
        }
        let Some(system) = run.main(sub) else { continue };
        let reference = run.new_obj(Kind::SystemReference, None, None);
        run.p[reference].system = Some(system);
        run.p[reference].source = Some("CapellaElements/ModelRef/Model_Function".to_string());
        run.p[reference].name = Some(format!("Reference to {}", run.p[system].name.clone().unwrap_or_else(|| "null".to_string())));
        run.contain(diagram, reference, |o| &mut o.blocks);
        let selection = run.selection.clone();
        for port in run.ports(reference) {
            let source = run.source_port[&port];
            if run.p[source].port_kind == "Enable" {
                propagate(run, port, sub, &selection)?;
            }
        }
    }
    Ok(())
}

/// The name a port takes in a holder: the radix, else the radix with the first free `_n`.
fn unique_name(run: &mut Run, holder: usize, radix: Option<String>) -> Option<String> {
    let names: Vec<Option<String>> = run.ports(holder).iter().map(|&p| port_name(run, p)).collect();
    let mut name = radix.clone();
    let mut index = 1;
    while names.contains(&name) {
        name = Some(format!("{}_{index}", radix.clone().unwrap_or_else(|| "null".to_string())));
        index += 1;
    }
    name
}

/// A port's name; a derived port's is its system port's.
fn port_name(run: &Run, p: usize) -> Option<String> {
    match run.source_port.get(&p) {
        Some(&source) => run.p[source].name.clone(),
        None => run.p[p].name.clone(),
    }
}

fn port_kind(run: &Run, p: usize) -> String {
    run.source_port.get(&p).map(|&s| run.p[s].port_kind.clone()).unwrap_or_else(|| run.p[p].port_kind.clone())
}

/// `PortNamingStrategy.computePortName(FunctionPort)`: the first exchange's name, or the port's.
fn capella_port_name(run: &Run, port: usize) -> Option<String> {
    if run.options.ports_named_by_exchange() {
        if let Some(&fe) = exchanges_of_port(run.m, port).first() {
            return run.m.name(fe);
        }
    }
    run.m.name(port)
}

/// `FunctionTransformationHelper.getPortIndex`: its rank among the holder's ports of its kind.
fn port_index(run: &mut Run, port: usize) -> i32 {
    let holder = run.p[port].container.expect("a port in a holder");
    let kind = port_kind(run, port);
    let same: Vec<usize> = run.ports(holder).into_iter().filter(|&p| port_kind(run, p) == kind).collect();
    same.iter().position(|&p| p == port).map(|i| i as i32 + 1).unwrap_or(0)
}

/// `FunctionPortRule.doCreate`: a port for each exchange of the Capella port (one if it has none).
fn create_ports(run: &mut Run, port: usize) -> Result<Vec<usize>, String> {
    let m = run.m;
    let input = m.is(port, "FunctionInputPort");
    let kind = if input { "Inport" } else if m.is(port, "FunctionOutputPort") { "Outport" } else { "null" };
    let mut results = Vec::new();
    for fe in exchanges_of_port(m, port) {
        let other = if input { source_port(m, fe) } else { target_port(m, fe) };
        results.push(create_port(run, port, Some(fe), kind, other)?);
    }
    if results.is_empty() {
        results.push(create_port(run, port, None, kind, None)?);
    }
    Ok(results)
}

fn create_port(run: &mut Run, port: usize, fe: Option<usize>, kind: &str, other: Option<usize>) -> Result<usize, String> {
    let m = run.m;
    let container = m.container(port);
    let holder = match run.main_opt(container) {
        Some(h) if run.p[h].kind.is_block() || run.p[h].kind.is_diagram() => h,
        _ => return Err(format!("Port Transformation Exception: Invalid target for: {}", container.map(|c| run.label(c)).unwrap_or_default())),
    };
    let p = run.new_obj(Kind::Port, None, m.id(port));
    run.p[p].port_kind = kind.to_string();
    if let Some(fe) = fe {
        let datatype = exchange_type(run, fe)?;
        run.p[p].datatype = Some(datatype);
    }
    let name = unique_name(run, holder, capella_port_name(run, port));
    run.p[p].name = name;
    run.contain(holder, p, |o| &mut o.ports);
    let index = port_index(run, p);
    run.p[p].index = index;
    run.set_main(Key::S(port_key(m, fe, port)), p);
    let shown = fe.is_none_or(|fe| target_diagram(run, fe).is_some());
    if other.is_none() || !run.selected(other) || !shown {
        let selection = run.selection.clone();
        if let Some(c) = container {
            propagate(run, p, c, &selection)?;
        }
    }
    Ok(p)
}

/// `FunctionalExchangeHelper.getKey`.
fn port_key(m: &Model, fe: Option<usize>, port: usize) -> String {
    let fe = fe.and_then(|fe| m.id(fe)).unwrap_or_else(|| "no".to_string());
    format!("Port:{fe}:{}", m.id(port).unwrap_or_else(|| "null".to_string()))
}

/// The bus of an exchange, made once (`FunctionPortRule.getType`).
fn exchange_type(run: &mut Run, fe: usize) -> Result<usize, String> {
    let key = Key::S(format!("type:{}", run.m.id(fe).unwrap_or_else(|| "null".to_string())));
    if let Some(t) = run.target(&key) {
        return Ok(t);
    }
    let bus = create_type(run, fe)?;
    run.set_main(key, bus);
    Ok(bus)
}

/// `FunctionalExchangeHelper.createType`: a bus with an element for each exchange item, put in
/// the layer's dictionary.
pub fn create_type(run: &mut Run, fe: usize) -> Result<usize, String> {
    let m = run.m;
    let bus = run.new_obj(Kind::Bus, Some(data::bus_name(m, fe)), m.id(fe));
    let architecture = m.ancestor(fe, "BlockArchitecture");
    let Some(dictionary) = run.main_opt(architecture) else { return Err(format!("NullPointerException: no dictionary for the layer of {}", run.label(fe))) };
    run.contain(dictionary, bus, |o| &mut o.datatypes);
    for ei in exchanged_items(m, fe) {
        let sub = run.main(ei);
        let element = run.new_obj(Kind::BusElement, m.name(ei), m.id(ei));
        run.p[element].datatype = sub;
        run.contain(bus, element, |o| &mut o.bus_elements);
    }
    Ok(bus)
}

/// `FunctionalExchangeHelper.getTargetDiagram`: the diagram of the common ancestor of the
/// exchange's ports, or of the first container that has one.
fn target_diagram(run: &Run, fe: usize) -> Option<usize> {
    let m = run.m;
    let (source, target) = (source_port(m, fe)?, target_port(m, fe)?);
    let mut ancestor = m.common_ancestor(target, source);
    let mut diagram = run.main_opt(ancestor).filter(|&d| run.p[d].kind.is_diagram());
    while diagram.is_none() && ancestor.is_some() {
        ancestor = ancestor.and_then(|a| m.container(a));
        diagram = run.main_opt(ancestor).filter(|&d| run.p[d].kind.is_diagram());
    }
    diagram
}

/// `FunctionTransformationHelper.propagatePortToParent`: a port for each transformed parent of
/// the Capella holder, each linked to the one below it.
fn propagate(run: &mut Run, port: usize, capella_holder: usize, selection: &[usize]) -> Result<(), String> {
    let m = run.m;
    let mut parent = capella_holder;
    let mut origin = run.source_port.get(&port).copied().unwrap_or(port);
    while m.is(parent, "AbstractFunction") && m.container(parent).is_some_and(|c| selection.contains(&c)) {
        let function = m.container(parent).expect("a container");
        let Some(holder) = run.main(function) else { return Ok(()) };
        let kind = run.p[holder].kind;
        if !(kind.is_block() || kind.is_diagram()) {
            return Err(format!("Port Propagation Exception: Invalid target for: {}", m.name(function).unwrap_or_default()));
        }
        let propagated = run.ports(holder).iter().any(|p| run.reference.get(p) == Some(&port));
        if propagated {
            return Ok(());
        }
        let radix = port_name(run, port);
        let name = unique_name(run, holder, radix);
        let copy = run.new_obj(Kind::Port, name, run.p[origin].originator.clone());
        run.p[copy].id = Some("propagated_null".to_string());
        let origin_kind = port_kind(run, origin);
        run.p[copy].port_kind = if origin_kind == "Enable" { "Inport".to_string() } else { origin_kind.clone() };
        run.p[copy].datatype = run.p[origin].datatype;
        run.reference.insert(copy, origin);
        run.contain(holder, copy, |o| &mut o.ports);
        let index = port_index(run, copy);
        run.p[copy].index = index;
        if !run.p[holder].kind.is_diagram() {
            return Err("Invalid parent target".to_string());
        }
        let line_name = if run.options.ports_named_by_exchange() { run.p[copy].name.clone() } else { None };
        let line = create_line(run, origin, copy, line_name);
        let id = run.uuid("line");
        run.p[line].id = Some(format!("propagated_{id}"));
        run.p[line].originator = run.p[origin].originator.clone();
        run.contain(holder, line, |o| &mut o.lines);
        origin = copy;
        parent = function;
    }
    Ok(())
}

/// `FunctionTransformationHelper.createLine`: from the outer port into the inner one for an
/// input, the other way for an output.
fn create_line(run: &mut Run, origin: usize, outer: usize, name: Option<String>) -> usize {
    let line = run.new_obj(Kind::Line, name, None);
    let inner = concrete_port(run, outer, origin);
    let (source, target) = match port_kind(run, origin).as_str() {
        "Outport" => (inner, outer),
        _ => (outer, inner),
    };
    run.p[line].line_source = Some(source);
    run.p[line].line_target = Some(target);
    line
}

/// `getConcretPort`: the port of the reference to the inner port's model, if the outer diagram
/// holds one.
fn concrete_port(run: &mut Run, outer: usize, origin: usize) -> usize {
    let Some(diagram) = run.p[outer].container.filter(|&d| run.p[d].kind.is_diagram()) else { return origin };
    let origin_holder = run.p[origin].container;
    let reference = run.p[diagram].blocks.iter().copied().find(|&b| run.p[b].kind == Kind::SystemReference && run.p[b].system.is_some() && run.p[b].system == origin_holder);
    match reference {
        Some(r) => run.referenced_port(r, origin).unwrap_or(origin),
        None => origin,
    }
}

/// `FunctionalExchangeRule` (functions): a line between the two transformed functions.
fn create_exchange_line(run: &mut Run, fe: usize) -> Result<(), String> {
    let m = run.m;
    if run.selected(m.reference(fe, "source")) && run.selected(m.reference(fe, "target")) {
        if let Some(diagram) = target_diagram(run, fe) {
            let line = exchange_line(run, fe, diagram)?;
            run.contain(diagram, line, |o| &mut o.lines);
        }
    }
    Ok(())
}

/// `FunctionalExchangeHelper.createLine`.
fn exchange_line(run: &mut Run, fe: usize, diagram: usize) -> Result<usize, String> {
    let m = run.m;
    let (Some(source), Some(target)) = (source_port(m, fe), target_port(m, fe)) else { return Err(format!("NullPointerException: the exchange {} does not link two function ports", run.label(fe))) };
    let ancestor = m.common_ancestor(target, source);
    let line = run.new_obj(Kind::Line, m.name(fe), m.id(fe));
    let out = line_end(run, diagram, ancestor, fe, source)?;
    run.p[line].line_source = out;
    let inp = line_end(run, diagram, ancestor, fe, target)?;
    run.p[line].line_target = inp;
    Ok(line)
}

fn line_end(run: &mut Run, diagram: usize, ancestor: Option<usize>, fe: usize, port: usize) -> Result<Option<usize>, String> {
    let m = run.m;
    let Some(p) = run.target(&Key::S(port_key(m, Some(fe), port))) else { return Err(format!("NullPointerException: the port {} was not transformed", run.label(port))) };
    let containment = containment(m, port, ancestor);
    if let Some(c) = m.container(port) {
        propagate(run, p, c, &containment)?;
    }
    Ok(referencing_port(run, p, diagram))
}

/// `computeContainment`: the functions between the port's function and the ancestor.
fn containment(m: &Model, port: usize, ancestor: Option<usize>) -> Vec<usize> {
    let mut out = Vec::new();
    let mut parent = m.container(port).and_then(|c| m.container(c));
    while let Some(p) = parent {
        if Some(p) == ancestor {
            break;
        }
        if m.is(p, "AbstractFunction") {
            out.push(p);
        }
        parent = m.container(p);
    }
    out
}

/// `getReferencingPort`: the port of a block of the diagram that is, or comes from, this one.
fn referencing_port(run: &mut Run, port: usize, diagram: usize) -> Option<usize> {
    for block in run.p[diagram].blocks.clone() {
        if run.p[block].kind == Kind::SystemReference {
            if let Some(p) = run.referenced_port(block, port) {
                return Some(p);
            }
        } else if run.p[block].kind.is_block() {
            for p in run.ports(block) {
                if connected(run, port, p) {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn connected(run: &Run, base: usize, candidate: usize) -> bool {
    let reference = run.reference.get(&candidate).copied();
    if base == candidate || reference == Some(base) {
        return true;
    }
    match reference {
        Some(r) => connected(run, base, r),
        None => false,
    }
}

// ---- Functional chains ---------------------------------------------------------------

fn create_chain(run: &mut Run, chain: usize) -> Result<usize, String> {
    let m = run.m;
    let system = run.new_obj(Kind::System, m.name(chain), m.id(chain));
    if let Some(layer) = run.layer(chain) {
        run.set_layer(system, layer);
    }
    run.set_main(Key::E(chain), system);
    run.create_default_dictionaries(chain, system)?;
    run.set_main(Key::E(chain), system);
    let project = run.project;
    run.contain(project, system, |o| &mut o.systems);
    for f in flat_involved(m, chain, "AbstractFunction") {
        if !run.selected(Some(f)) {
            continue;
        }
        let target = run.main(f);
        let reference = run.new_obj(Kind::SystemReference, None, None);
        run.p[reference].system = target;
        run.p[reference].source = Some("CapellaElements/ModelRef/Model_Function".to_string());
        let name = target.and_then(|t| run.p[t].name.clone()).unwrap_or_else(|| "null".to_string());
        run.p[reference].name = Some(format!("Reference to {name}"));
        run.contain(system, reference, |o| &mut o.blocks);
    }
    let selection = run.selection.clone();
    let mut exchanges: Vec<usize> = flat_involved(m, chain, "FunctionalExchange")
        .into_iter()
        .filter(|&fe| selection.contains(&fe) && run.selected(m.reference(fe, "source")) && run.selected(m.reference(fe, "target")))
        .collect();
    exchanges.sort_by_key(|fe| selection.iter().position(|e| e == fe));
    for fe in exchanges {
        let line = exchange_line(run, fe, system)?;
        run.contain(system, line, |o| &mut o.lines);
    }
    Ok(system)
}
