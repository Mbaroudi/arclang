//! The functions of the logical and physical levels. ArcLang writes a
//! function under the component that performs it (or at the level when
//! nobody does), and names its parent with `parent:` when the parent is
//! written elsewhere; Capella keeps one tree of functions under the root
//! function and allocates functions to components. This rebuilds the tree.

use super::context::{port_id, text, Context};
use super::header::apply;
use super::xml::El;
use crate::compiler::ast::{FunctionPort, LogicalFunction, PortDirection};
use std::collections::HashMap;

/// A function where ArcLang writes it: who performs it, and the function it
/// is written in, if any.
pub(super) struct Placed<'a> {
    pub(super) function: &'a LogicalFunction,
    pub(super) owner: Option<String>,
    nested_in: Option<String>,
}

/// The functions written under `owner` (or at the level), with their nested ones.
pub(super) fn place<'a>(functions: &'a [LogicalFunction], owner: Option<&str>, into: &mut Vec<Placed<'a>>) {
    fn walk<'a>(functions: &'a [LogicalFunction], owner: Option<&str>, nested_in: Option<&str>, into: &mut Vec<Placed<'a>>) {
        for function in functions {
            into.push(Placed { function, owner: owner.map(str::to_string), nested_in: nested_in.map(str::to_string) });
            walk(&function.sub_functions, owner, Some(&function.id), into);
        }
    }
    walk(functions, owner, None, into);
}

pub(super) fn register(context: &mut Context, placed: &[Placed<'_>]) {
    for p in placed {
        let f = p.function;
        context.functions.insert(f.id.clone());
        context.register(&f.id, &f.id);
        context.register(&f.name, &f.id);
        for port in &f.ports {
            let id = port_id(&f.id, &port.name, &port.id);
            context.register(&format!("{}.{}", f.id, port.name), &id);
            context.register(&format!("{}.{}", f.name, port.name), &id);
        }
    }
}

pub(super) fn port(owner: &str, port: &FunctionPort) -> El {
    let (tag, kind) = match port.direction {
        PortDirection::Out => ("outputs", "fa:FunctionOutputPort"),
        _ => ("inputs", "fa:FunctionInputPort"),
    };
    El::new(tag, kind, &port_id(owner, &port.name, &port.id)).attr("name", &port.name)
}

/// The root function of the level, holding every function in its tree;
/// `kind` is the Capella kind of the level's functions. `realizes:` becomes
/// a function realization.
pub(super) fn tree(context: &Context, kind: &str, root: El, placed: &[Placed<'_>]) -> El {
    let mut children: HashMap<Option<String>, Vec<&Placed<'_>>> = HashMap::new();
    for p in placed {
        let parent = text(&p.function.attributes, "parent").and_then(|r| context.resolve(r)).map(str::to_string).or_else(|| p.nested_in.clone());
        children.entry(parent).or_default().push(p);
    }
    fn build(context: &Context, kind: &str, p: &Placed<'_>, children: &HashMap<Option<String>, Vec<&Placed<'_>>>) -> El {
        let f = p.function;
        let mut el = apply(El::new("ownedFunctions", kind, &f.id).attr("name", &f.name), &f.attributes, &["parent", "realizes", "kind"]);
        for port in &f.ports {
            el.push(self::port(&f.id, port));
        }
        if let Some(target) = text(&f.attributes, "realizes").and_then(|r| context.resolve(r)) {
            el.push(super::realization("ownedFunctionRealizations", "fa:FunctionRealization", &f.id, target));
        }
        for child in children.get(&Some(f.id.clone())).into_iter().flatten() {
            el.push(build(context, kind, child, children));
        }
        el
    }
    let mut root = root;
    for top in children.get(&None).into_iter().flatten() {
        root.push(build(context, kind, top, &children));
    }
    root
}

/// Functional allocations of a component: the functions written under it
/// and those it names in `allocated_functions`.
pub(super) fn allocations(context: &Context, component: &str, placed: &[Placed<'_>], named: &[String]) -> Vec<El> {
    let mut functions: Vec<String> = placed.iter().filter(|p| p.owner.as_deref() == Some(component)).map(|p| p.function.id.clone()).collect();
    for name in named {
        if let Some(id) = context.resolve(name).filter(|id| context.functions.contains(*id)) {
            if !functions.iter().any(|f| f == id) {
                functions.push(id.to_string());
            }
        }
    }
    functions
        .into_iter()
        .map(|target| {
            El::new("ownedFunctionalAllocation", "fa:ComponentFunctionalAllocation", &super::context::derived(&format!("{component}->{target}:allocation")))
                .attr("targetElement", &format!("#{target}"))
                .attr("sourceElement", &format!("#{component}"))
        })
        .collect()
}

/// Move each functional exchange of the root function into the lowest
/// function holding both its ends, as Capella owns exchanges.
pub(super) fn place_exchanges(root: &mut El) {
    // Parent of every function and port, under the root.
    let mut parent: HashMap<String, String> = HashMap::new();
    fn walk(el: &El, parent: &mut HashMap<String, String>) {
        for child in el.children() {
            let kind = child.kind();
            if kind.ends_with("Function") || kind.ends_with("OperationalActivity") || kind.ends_with("InputPort") || kind.ends_with("OutputPort") {
                parent.insert(child.id().to_string(), el.id().to_string());
                walk(child, parent);
            }
        }
    }
    walk(root, &mut parent);
    let root_id = root.id().to_string();
    let ancestors = |id: &str| -> Vec<String> {
        let mut chain = Vec::new();
        let mut current = parent.get(id).cloned();
        while let Some(p) = current {
            chain.push(p.clone());
            current = parent.get(&p).cloned();
        }
        chain
    };
    let exchanges = root.take_children(|c| c.kind().ends_with(":FunctionalExchange"));
    for exchange in exchanges {
        let end = |key: &str| exchange.get(key).map(|r| r.trim_start_matches('#').to_string()).unwrap_or_default();
        // The functions above an end: above a port, above its function.
        let above = |id: String| -> Vec<String> {
            let chain = ancestors(&id);
            let is_port = root.find(&id).is_some_and(|e| e.kind().ends_with("InputPort") || e.kind().ends_with("OutputPort"));
            if is_port { chain.into_iter().skip(1).collect() } else { chain }
        };
        let (from, to) = (above(end("source")), above(end("target")));
        let holder = from.iter().find(|f| to.contains(f)).cloned().unwrap_or_else(|| root_id.clone());
        match root.find_mut(&holder) {
            Some(owner) => owner.push(exchange),
            None => root.push(exchange),
        }
    }
}
