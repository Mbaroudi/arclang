//! Function nodes and functional exchanges, shared by the system, logical
//! and physical viewpoints (a function is drawn wherever it is allocated).

use super::builder::{identity, properties, string_list, Builder, Endpoint, NodeRef, Role};
use super::{EdgeKind, NodeKind, PortDirection};
use crate::compiler::ast::{self, FunctionalExchange, Model, SystemFunction};
use std::collections::{BTreeMap, HashMap, HashSet};

const FUNCTION_PROPERTIES: [&str; 5] =
    ["description", "safety_level", "asil", "latency", "category"];

/// Placeholder the parser stores when an exchange declares no exchange item.
pub(super) const UNSPECIFIED_ITEM: &str = "Data";

/// Outcome of drawing an allocated function.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Placement {
    Drawn,
    /// The reference names no declared function.
    Unknown,
    /// The reference names several functions.
    Ambiguous,
}

/// Where a function was drawn: the owner's diagram id and display name.
struct Owner {
    id: String,
    name: String,
}

/// Every system function of the model (sub-functions included), addressable
/// by identity or name, with a record of which ones a diagram has drawn.
pub(super) struct FunctionTable<'a> {
    functions: Vec<&'a SystemFunction>,
    /// Function index -> indices of its direct sub-functions.
    children: Vec<Vec<usize>>,
    /// Reference key -> functions answering to it (several = ambiguous).
    by_key: HashMap<&'a str, Vec<usize>>,
    placed: HashMap<usize, Owner>,
}

impl<'a> FunctionTable<'a> {
    pub fn of(model: &'a Model) -> Self {
        let mut table = Self {
            functions: Vec::new(),
            children: Vec::new(),
            by_key: HashMap::new(),
            placed: HashMap::new(),
        };
        for sa in &model.system_analysis {
            for function in &sa.functions {
                table.register(function);
            }
        }
        table
    }

    fn register(&mut self, function: &'a SystemFunction) -> usize {
        let index = self.functions.len();
        self.functions.push(function);
        self.children.push(Vec::new());
        for key in [
            identity(&function.attributes, &function.id),
            function.name.as_str(),
        ] {
            let holders = self.by_key.entry(key).or_default();
            if !holders.contains(&index) {
                holders.push(index);
            }
        }
        for sub in &function.sub_functions {
            let child = self.register(sub);
            self.children[index].push(child);
        }
        index
    }

    /// Whether `reference` (a function, or `Function.port`) names a declared
    /// function — drawn in the current diagram or not.
    pub fn knows(&self, reference: &str) -> bool {
        self.by_key.contains_key(reference)
            || reference
                .rsplit_once('.')
                .is_some_and(|(owner, _)| self.by_key.contains_key(owner))
    }

    /// Draw the referenced function under `owner`, once per diagram.
    ///
    /// Arcadia allocates a function to exactly one component: a function
    /// already drawn elsewhere — itself or through its parent function — is
    /// reported and kept where it was first drawn.
    pub fn place(
        &mut self,
        builder: &mut Builder,
        owner_node: NodeRef,
        reference: &str,
    ) -> Placement {
        let index = match self.by_key.get(reference).map(Vec::as_slice) {
            None => return Placement::Unknown,
            Some([single]) => *single,
            Some(_) => return Placement::Ambiguous,
        };
        let owner = Owner {
            id: builder.node(owner_node).id.clone(),
            name: builder.node(owner_node).name.clone(),
        };
        if !self.already_placed(builder, index, &owner) {
            self.draw(builder, owner_node, index, &owner);
        }
        Placement::Drawn
    }

    /// True when the function is drawn already; reports it if that was
    /// under a different owner.
    fn already_placed(&self, builder: &mut Builder, index: usize, owner: &Owner) -> bool {
        let Some(previous) = self.placed.get(&index) else {
            return false;
        };
        if previous.id != owner.id {
            builder.report(format!(
                "function '{}' is allocated to both '{}' and '{}' — drawn in '{}' only",
                self.functions[index].name, previous.name, owner.name, previous.name
            ));
        }
        true
    }

    fn draw(&mut self, builder: &mut Builder, parent: NodeRef, index: usize, owner: &Owner) {
        let node = add_function_node(builder, Some(parent), self.functions[index]);
        self.placed.insert(
            index,
            Owner {
                id: owner.id.clone(),
                name: owner.name.clone(),
            },
        );
        for child in self.children[index].clone() {
            if !self.already_placed(builder, child, owner) {
                self.draw(builder, node, child, owner);
            }
        }
    }
}

/// Add a function and all its sub-functions (system view: every function
/// belongs to the system, nothing is allocated elsewhere).
pub(super) fn add_function(
    builder: &mut Builder,
    parent: Option<NodeRef>,
    function: &SystemFunction,
) {
    let node = add_function_node(builder, parent, function);
    for sub in &function.sub_functions {
        add_function(builder, Some(node), sub);
    }
}

/// Add one function node with its declared ports: explicit `port`
/// declarations plus the `inputs` / `outputs` lists.
fn add_function_node(
    builder: &mut Builder,
    parent: Option<NodeRef>,
    function: &SystemFunction,
) -> NodeRef {
    let node = builder.add_node(
        parent,
        identity(&function.attributes, &function.id),
        &function.name,
        NodeKind::Function,
        properties(&function.attributes, &FUNCTION_PROPERTIES),
    );
    for port in &function.ports {
        let direction = match port.direction {
            ast::PortDirection::In => PortDirection::In,
            ast::PortDirection::Out => PortDirection::Out,
            ast::PortDirection::InOut => PortDirection::InOut,
        };
        let data_type = Some(port.data_type.clone()).filter(|t| !t.is_empty());
        builder.add_port(
            node,
            &port_suffix(direction, &port.name),
            &port.name,
            direction,
            data_type,
            None,
        );
    }
    add_flow_ports(builder, node, &function.attributes);
    node
}

/// Ports declared through `inputs: [...]` / `outputs: [...]` attributes.
pub(super) fn add_flow_ports(
    builder: &mut Builder,
    node: NodeRef,
    attributes: &HashMap<String, ast::AttributeValue>,
) {
    for (key, direction) in [
        ("inputs", PortDirection::In),
        ("outputs", PortDirection::Out),
    ] {
        for name in string_list(attributes, key) {
            let exists = builder
                .node(node)
                .ports
                .iter()
                .any(|p| p.name == name && p.direction == direction);
            if !exists {
                builder.add_port(
                    node,
                    &port_suffix(direction, &name),
                    &name,
                    direction,
                    None,
                    None,
                );
            }
        }
    }
}

fn port_suffix(direction: PortDirection, name: &str) -> String {
    let side = match direction {
        PortDirection::In => "in",
        PortDirection::Out => "out",
        PortDirection::InOut => "inout",
        PortDirection::Undirected => "port",
    };
    format!("{side}.{name}")
}

/// Add the functional exchanges between functions drawn in this diagram,
/// each bound to a function port on both sides.
///
/// `allocatable` is the function table of a viewpoint that draws only the
/// allocated functions (logical, physical): an exchange reaching a declared
/// but unallocated function is then left out without a diagnostic — an
/// incomplete allocation is not a broken reference. With `None` (system
/// view) every unresolved endpoint is reported.
pub(super) fn add_functional_exchanges<'a>(
    builder: &mut Builder,
    exchanges: impl Iterator<Item = &'a FunctionalExchange>,
    allocatable: Option<&FunctionTable<'_>>,
) {
    for exchange in exchanges {
        let name = exchange
            .label
            .clone()
            .unwrap_or_else(|| format!("{} -> {}", exchange.from_port, exchange.to_port));
        let what = format!("functional_exchange '{name}'");
        let item =
            Some(exchange.data_type.as_str()).filter(|i| !i.is_empty() && *i != UNSPECIFIED_ITEM);

        let ends = [
            (&exchange.from_port, Role::Source),
            (&exchange.to_port, Role::Target),
        ];
        // An exchange with an end outside this view is not drawn at all, so
        // do not report port problems on its other end either.
        let outside_view = |builder: &Builder, reference: &str| {
            allocatable.is_some_and(|table| table.knows(reference))
                && owner_of(builder, reference).is_none()
        };
        if ends
            .iter()
            .any(|(reference, _)| outside_view(builder, reference))
        {
            continue;
        }
        let mut resolved: Vec<Endpoint> = Vec::with_capacity(2);
        for (reference, role) in ends {
            match builder.resolve_endpoint(&what, reference, role) {
                Ok(endpoint) => resolved.push(endpoint),
                Err(missing) => builder.report_missing(&what, reference, missing, "function"),
            }
        }
        let Ok([mut source, mut target]) = <[Endpoint; 2]>::try_from(resolved) else {
            continue;
        };
        if source.whole {
            source.port = bind_port(builder, source.node, PortDirection::Out, item, &name, &what);
        }
        if target.whole {
            target.port = bind_port(builder, target.node, PortDirection::In, item, &name, &what);
        }
        builder.add_edge(
            EdgeKind::FunctionalExchange,
            &source,
            &target,
            &name,
            item.map(str::to_string),
            BTreeMap::new(),
        );
    }
}

/// The node a reference (`Node` or `Node.port`) points into, if drawn.
fn owner_of(builder: &Builder, reference: &str) -> Option<NodeRef> {
    builder.resolve(reference).or_else(|| {
        reference
            .rsplit_once('.')
            .and_then(|(owner, _)| builder.resolve(owner))
    })
}

/// Find the function port an exchange uses when its endpoint names none:
/// the declared port carrying the exchange item, else a synthesized one.
/// Declaring flows that do not include the exchanged item is reported.
fn bind_port(
    builder: &mut Builder,
    node: NodeRef,
    direction: PortDirection,
    item: Option<&str>,
    exchange_name: &str,
    what: &str,
) -> Option<String> {
    if builder.node(node).kind != NodeKind::Function {
        return None;
    }
    let candidates: Vec<(String, String, bool)> = builder
        .node(node)
        .ports
        .iter()
        .filter(|p| p.direction == direction || p.direction == PortDirection::InOut)
        .map(|p| (p.id.clone(), p.name.clone(), p.synthesized))
        .collect();
    let port_name = item.unwrap_or(exchange_name);
    if let Some((id, _, _)) = candidates.iter().find(|(_, name, _)| name == port_name) {
        return Some(id.clone());
    }
    let declared: Vec<&(String, String, bool)> = candidates
        .iter()
        .filter(|(_, _, synthesized)| !synthesized)
        .collect();
    match (item, declared.as_slice()) {
        // No exchange item given: a single declared flow is unambiguous.
        (None, [(id, _, _)]) => return Some(id.clone()),
        (Some(item), [_, ..]) => {
            let function = builder.node(node).name.clone();
            let flow = if direction == PortDirection::Out {
                "output"
            } else {
                "input"
            };
            builder.report(format!(
                "{what}: function '{function}' declares no {flow} '{item}'"
            ));
        }
        _ => {}
    }
    let port = builder.add_port(
        node,
        &port_suffix(direction, port_name),
        port_name,
        direction,
        None,
        None,
    );
    builder.mark_synthesized(node, &port);
    Some(port)
}

/// References in order, without repeats.
pub(super) fn dedupe(references: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen = HashSet::new();
    references
        .into_iter()
        .filter(|r| seen.insert(r.clone()))
        .collect()
}
