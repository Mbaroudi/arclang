//! Simulink / System Composer export.
//!
//! Emits a MATLAB script (.m) that rebuilds the architecture in MathWorks
//! System Composer with its documented API: the logical architecture as one
//! model (components nested as the model nests them, oriented ports, port to
//! port connections, through the boundary of the components between two
//! ends), the physical architecture as a second model (nodes, behaviour
//! components, physical ports and links), and Stateflow skeletons of the
//! state machines (see [`simulink_stateflow`](super::simulink_stateflow)).
//! The script is plain text: its generation is tested; its execution
//! belongs to a licensed MATLAB environment.
//!
//! Scope: architecture skeleton and interface contracts. Behaviour stays in
//! Simulink, as in the usual industrial split. Actors, operational entities
//! and capabilities are not components of an architecture model and are not
//! written.

use super::ast::{ComponentPort, LogicalComponent, Model, PortDirection};
use super::semantic::SemanticModel;
use std::collections::{BTreeMap, HashSet};

/// A valid MATLAB identifier: letters/digits/underscore, starts with a
/// letter, at most 63 characters.
pub(super) fn matlab_ident(raw: &str) -> String {
    let mut out = String::new();
    for c in raw.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('_') {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_');
    let mut ident = if trimmed.is_empty() { "X".to_string() } else { trimmed.to_string() };
    if !ident.starts_with(|c: char| c.is_ascii_alphabetic()) {
        ident.insert(0, 'C');
    }
    ident.truncate(63);
    ident
}

/// A MATLAB character vector: quotes doubled, on one line.
pub(super) fn m_string(text: &str) -> String {
    format!("'{}'", text.replace(['\n', '\r'], " ").replace('\'', "''"))
}

/// `base`, else `base_2`, `base_3`... the first not in `taken`, which it joins.
pub(super) fn unique(taken: &mut HashSet<String>, base: String, separator: &str) -> String {
    let mut candidate = base.clone();
    let mut n = 1;
    while taken.contains(&candidate) {
        n += 1;
        candidate = format!("{base}{separator}{n}");
    }
    taken.insert(candidate.clone());
    candidate
}

/// A component of one of the architecture models.
struct Part {
    var: String,
    parent: Option<usize>,
    /// Display name, unique among the parts of its level.
    name: String,
    /// Written id and name: what an exchange end calls it by.
    id: String,
    label: String,
    /// Port name -> direction (`in`, `out`, `physical`), in declaration order.
    ports: Vec<(String, &'static str)>,
}

struct Architecture {
    /// The variable of the model's root architecture (`arch`, `phys`).
    root: String,
    parts: Vec<Part>,
    names: BTreeMap<Option<usize>, HashSet<String>>,
}

impl Architecture {
    fn new(root: &str) -> Self {
        Architecture { root: root.to_string(), parts: Vec::new(), names: BTreeMap::new() }
    }

    fn add(&mut self, vars: &mut HashSet<String>, parent: Option<usize>, id: &str, label: &str) -> usize {
        let var = unique(vars, format!("c_{}", matlab_ident(id)), "_");
        let name = unique(self.names.entry(parent).or_default(), label.to_string(), " (");
        let name = if name == label { name } else { format!("{name})") };
        self.parts.push(Part { var, parent, name, id: id.to_string(), label: label.to_string(), ports: Vec::new() });
        self.parts.len() - 1
    }

    /// Declare a port on a part once; the name it is declared under.
    fn port(&mut self, part: usize, name: &str, direction: &'static str) -> String {
        if !self.parts[part].ports.iter().any(|(n, _)| n == name) {
            self.parts[part].ports.push((name.to_string(), direction));
        }
        name.to_string()
    }

    /// The part an exchange end names, by id or else by a name only one bears.
    fn find(&self, key: &str) -> Option<usize> {
        if let Some(index) = self.parts.iter().position(|p| p.id == key) {
            return Some(index);
        }
        let named: Vec<usize> = (0..self.parts.len()).filter(|&i| self.parts[i].label == key).collect();
        (named.len() == 1).then(|| named[0])
    }

    /// `Owner.port.sub` -> (part, `port_sub`); `Owner` -> (part, None).
    fn end(&self, reference: &str) -> Option<(usize, Option<String>)> {
        let segments: Vec<&str> = reference.split('.').collect();
        (1..=segments.len()).rev().find_map(|k| {
            let part = self.find(&segments[..k].join("."))?;
            let port = (k < segments.len()).then(|| segments[k..].join("_"));
            Some((part, port))
        })
    }

    fn arch_of(&self, part: Option<usize>) -> String {
        match part {
            Some(p) => format!("{}.Architecture", self.parts[p].var),
            None => self.root.clone(),
        }
    }

    /// The parts holding `part`, nearest first.
    fn holders(&self, part: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut current = self.parts[part].parent;
        while let Some(p) = current {
            out.push(p);
            current = self.parts[p].parent;
        }
        out
    }
}

/// The written id of a component: `id:`, else its id, else its name.
fn written_id<'a>(attributes: &'a std::collections::HashMap<String, super::ast::AttributeValue>, id: &'a str, name: &'a str) -> &'a str {
    attributes.get("id").and_then(|v| v.as_string()).filter(|s| !s.is_empty()).unwrap_or(if id.is_empty() { name } else { id })
}

fn direction(port: &ComponentPort) -> &'static str {
    match port.direction {
        PortDirection::In => "in",
        PortDirection::Out => "out",
        // System Composer has no bidirectional data port: a physical port.
        PortDirection::InOut => "physical",
    }
}

/// A port and the ports it holds, flattened: `nuts` and `nuts_lugNut`.
fn flatten_ports(ports: &[ComponentPort], prefix: &str, out: &mut Vec<(String, &'static str)>) {
    for port in ports {
        let name = if prefix.is_empty() { port.name.clone() } else { format!("{prefix}_{}", port.name) };
        out.push((name.clone(), direction(port)));
        flatten_ports(&port.ports, &name, out);
    }
}

fn logical_parts(arch: &mut Architecture, vars: &mut HashSet<String>, parent: Option<usize>, components: &[LogicalComponent]) {
    for component in components {
        let id = written_id(&component.attributes, &component.id, &component.name).to_string();
        let part = arch.add(vars, parent, &id, &component.name);
        let mut ports = Vec::new();
        flatten_ports(&component.ports, "", &mut ports);
        ports.extend(component.interfaces_in.iter().map(|i| (i.name.clone(), "in")));
        ports.extend(component.interfaces_out.iter().map(|i| (i.name.clone(), "out")));
        for (name, direction) in ports {
            arch.port(part, &name, direction);
        }
        logical_parts(arch, vars, Some(part), &component.sub_components);
    }
}

/// One connection between two ends, through the boundaries in between.
struct Exchange {
    label: String,
    from: String,
    to: String,
}

/// The lines connecting one exchange: up from the source to the level both
/// ends share, across it, and down to the target.
fn connect(arch: &mut Architecture, exchange: &Exchange, out: &mut Vec<String>) {
    let (Some((source, source_port)), Some((target, target_port))) = (arch.end(&exchange.from), arch.end(&exchange.to)) else {
        out.push(format!("% {} not connected: an end is not a component of this model", m_string(&exchange.label)));
        return;
    };
    if source == target {
        out.push(format!("% {} not connected: both ends are on one component", m_string(&exchange.label)));
        return;
    }
    let boundary = exchange.label.clone();
    let source_port = source_port.unwrap_or_else(|| boundary.clone());
    let target_port = target_port.unwrap_or_else(|| boundary.clone());
    let source_port = arch.port(source, &source_port, "out");
    let target_port = arch.port(target, &target_port, "in");
    let up = arch.holders(source);
    let down = arch.holders(target);
    let common = up.iter().find(|h| down.contains(h)).copied();
    let source_holds_target = down.contains(&source);
    let target_holds_source = up.contains(&target);

    // The source side, as seen from the shared level.
    let mut current = if source_holds_target {
        format!("{}.Architecture.getPort({})", arch.parts[source].var, m_string(&source_port))
    } else {
        format!("{}.getPort({})", arch.parts[source].var, m_string(&source_port))
    };
    let shared = if source_holds_target { Some(source) } else if target_holds_source { Some(target) } else { common };
    if !source_holds_target {
        for &holder in up.iter().take_while(|&&h| Some(h) != shared) {
            let name = arch.port(holder, &boundary, "out");
            out.push(format!("connect({current}, {}.Architecture.getPort({}));", arch.parts[holder].var, m_string(&name)));
            current = format!("{}.getPort({})", arch.parts[holder].var, m_string(&name));
        }
    }
    // The target side, from the shared level down.
    let mut path: Vec<usize> = if target_holds_source { Vec::new() } else { down.iter().take_while(|&&h| Some(h) != shared).copied().collect() };
    path.reverse();
    let mut lines = Vec::new();
    let mut entry = if target_holds_source {
        format!("{}.Architecture.getPort({})", arch.parts[target].var, m_string(&target_port))
    } else {
        format!("{}.getPort({})", arch.parts[target].var, m_string(&target_port))
    };
    for &holder in path.iter().rev() {
        let name = arch.port(holder, &boundary, "in");
        lines.push(format!("connect({}.Architecture.getPort({}), {entry});", arch.parts[holder].var, m_string(&name)));
        entry = format!("{}.getPort({})", arch.parts[holder].var, m_string(&name));
    }
    out.push(format!("connect({current}, {entry}); % {}", exchange.label.replace(['\n', '\r'], " ")));
    lines.reverse();
    out.extend(lines);
}

/// The model's components, then their ports, then the connections.
fn write_architecture(out: &mut String, arch: &mut Architecture, exchanges: &[Exchange]) {
    let mut connections = Vec::new();
    let mut seen = HashSet::new();
    for exchange in exchanges {
        if seen.insert((exchange.label.clone(), exchange.from.clone(), exchange.to.clone())) {
            connect(arch, exchange, &mut connections);
        }
    }
    out.push_str("%% Components\n");
    for index in 0..arch.parts.len() {
        let part = &arch.parts[index];
        out.push_str(&format!("{} = addComponent({}, {});\n", part.var, arch.arch_of(part.parent), m_string(&part.name)));
    }
    out.push_str("\n%% Ports\n");
    for part in &arch.parts {
        for (name, direction) in &part.ports {
            out.push_str(&format!("addPort({}.Architecture, {}, {});\n", part.var, m_string(name), m_string(direction)));
        }
    }
    out.push_str("\n%% Connections\n");
    for line in connections {
        out.push_str(&line);
        out.push('\n');
    }
    out.push('\n');
}

fn logical_model(out: &mut String, ast: &Model, vars: &mut HashSet<String>) {
    let mut arch = Architecture::new("arch");
    for la in &ast.logical_architecture {
        logical_parts(&mut arch, vars, None, &la.components);
    }
    let mut exchanges = Vec::new();
    for la in &ast.logical_architecture {
        for exchange in &la.component_exchanges {
            let label = exchange.label.clone().filter(|l| !l.is_empty()).unwrap_or_else(|| exchange.id.clone());
            exchanges.push(Exchange { label, from: exchange.from_port.clone(), to: exchange.to_port.clone() });
        }
        for interface in &la.interfaces {
            exchanges.push(Exchange { label: interface.name.clone(), from: interface.from.clone(), to: interface.to.clone() });
        }
    }
    write_architecture(out, &mut arch, &exchanges);
}

fn physical_model(out: &mut String, ast: &Model, vars: &mut HashSet<String>) {
    let mut arch = Architecture::new("phys");
    for pa in &ast.physical_architecture {
        for node in &pa.nodes {
            let part = arch.add(vars, None, written_id(&node.attributes, &node.id, &node.name), &node.name);
            for port in &node.ports {
                arch.port(part, &port.name, "physical");
            }
            let mut ports = Vec::new();
            flatten_ports(&node.component_ports, "", &mut ports);
            for (name, direction) in ports {
                arch.port(part, &name, direction);
            }
            for behaviour in &node.behavior_components {
                let inner = arch.add(vars, Some(part), written_id(&behaviour.attributes, &behaviour.id, &behaviour.name), &behaviour.name);
                let mut ports = Vec::new();
                flatten_ports(&behaviour.ports, "", &mut ports);
                for (name, direction) in ports {
                    arch.port(inner, &name, direction);
                }
            }
        }
    }
    // A link joins physical ports: no direction, the source writes it.
    let mut exchanges = Vec::new();
    for pa in &ast.physical_architecture {
        for link in &pa.links {
            let label = if link.name.is_empty() { format!("{} - {}", link.from, link.to) } else { link.name.clone() };
            for end in [&link.from, &link.to] {
                if let Some((part, port)) = arch.end(end) {
                    arch.port(part, &port.unwrap_or_else(|| label.clone()), "physical");
                }
            }
            exchanges.push(Exchange { label, from: link.from.clone(), to: link.to.clone() });
        }
    }
    write_architecture(out, &mut arch, &exchanges);
}

pub fn generate_simulink_script(model: &SemanticModel, ast: &Model) -> String {
    let mut out = String::new();
    let model_name = matlab_ident(model.name.as_deref().unwrap_or("ArcLangModel"));
    let mut vars: HashSet<String> = ["arch", "phys", "archModel", "physModel"].iter().map(|s| s.to_string()).collect();

    out.push_str("%% Generated by ArcLang: System Composer architecture skeleton\n");
    out.push_str("% Run inside MATLAB with System Composer (and Stateflow for the charts).\n");
    out.push_str("% Mapping: logical component -> component (nested as in the model),\n");
    out.push_str("% port in/out -> port, inout -> physical port, exchange -> connection,\n");
    out.push_str("% physical node and link -> second model, state_machine -> Stateflow chart.\n\n");
    out.push_str(&format!("archModel = systemcomposer.createModel({});\n", m_string(&model_name)));
    out.push_str("arch = archModel.Architecture;\n\n");
    logical_model(&mut out, ast, &mut vars);
    out.push_str("save(archModel);\n\n");

    if ast.physical_architecture.iter().any(|pa| !pa.nodes.is_empty()) {
        out.push_str("%% Physical architecture\n");
        out.push_str(&format!("physModel = systemcomposer.createModel({});\n", m_string(&format!("{model_name}_physical"))));
        out.push_str("phys = physModel.Architecture;\n\n");
        physical_model(&mut out, ast, &mut vars);
        out.push_str("save(physModel);\n\n");
    }

    super::simulink_stateflow::write(&mut out, ast, &model_name);
    out.push_str(&format!("disp({});\n", m_string(&format!("ArcLang architecture {model_name} generated."))));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matlab_identifiers_are_sanitized() {
        assert_eq!(matlab_ident("Brake Controller"), "Brake_Controller");
        assert_eq!(matlab_ident("REQ-AEB-001"), "REQ_AEB_001");
        assert_eq!(matlab_ident("42x"), "C42x");
    }

    #[test]
    fn names_are_made_unique() {
        let mut taken = HashSet::new();
        assert_eq!(unique(&mut taken, "Off".to_string(), "_"), "Off");
        assert_eq!(unique(&mut taken, "Off".to_string(), "_"), "Off_2");
    }
}
