//! What the Modelica package is made of, read from the physical
//! architecture: the parts (nodes and behaviour components) with their
//! parameters and connectors, the connections between them, and, per
//! configuration, the parts it keeps and the variant parameters it sets.

use super::connector::{self, End, Orientation, Physics};
use super::names;
use super::units;
use crate::compiler::ast::{AttributeValue, ComponentPort, LogicalComponent, Model};
use crate::compiler::types::{effective_types, EffectiveType};
use crate::compiler::variability;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Attributes that are structure, not physics: never parameters.
const NOT_PARAMETERS: &[&str] = &["id", "multiplicity", "variation", "variant", "x", "y", "width", "height"];

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: String,
    pub value: Option<f64>,
    pub comment: String,
}

#[derive(Debug, Clone)]
pub struct Port {
    /// The ArcLang name of the port (`hv_p`).
    pub name: String,
    /// `Owner-id.port`, for traceability.
    pub id: String,
    pub member: String,
    pub physics: Physics,
    pub orientation: Orientation,
    pub connector: &'static str,
}

#[derive(Debug, Clone)]
pub struct Part {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub class: String,
    pub instance: String,
    pub params: Vec<Param>,
    pub ports: Vec<Port>,
    /// Logical components it deploys or realizes, by written id.
    pub realizes: Vec<String>,
    /// For a behaviour component: the node holding it.
    pub node: Option<usize>,
    /// For a node: its behaviour components.
    pub behaviours: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub id: String,
    /// (part, port) of each end.
    pub ends: [(usize, usize); 2],
    /// The node whose model holds it; `None` for the system models.
    pub within: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct System {
    pub id: String,
    pub name: String,
    pub class: String,
    pub selects: Vec<String>,
    /// The nodes it keeps, with the modifiers the variants it selects set.
    pub nodes: Vec<(usize, Vec<String>)>,
}

#[derive(Debug, Default)]
pub struct Structure {
    pub parts: Vec<Part>,
    pub connections: Vec<Connection>,
    pub systems: Vec<System>,
    pub notes: Vec<String>,
}

fn written_id(component: &LogicalComponent) -> String {
    component.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&component.id).to_string()
}

fn logical_index(ast: &Model) -> HashMap<String, &LogicalComponent> {
    fn walk<'a>(components: &'a [LogicalComponent], out: &mut HashMap<String, &'a LogicalComponent>) {
        for component in components {
            out.insert(written_id(component), component);
            out.entry(component.name.clone()).or_insert(component);
            walk(&component.sub_components, out);
        }
    }
    let mut out = HashMap::new();
    for la in &ast.logical_architecture {
        walk(&la.components, &mut out);
    }
    out
}

fn numeric_params(attributes: &HashMap<String, AttributeValue>) -> Vec<(String, String, f64, String)> {
    let mut keys: Vec<&String> = attributes.keys().filter(|k| !NOT_PARAMETERS.contains(&k.as_str())).collect();
    keys.sort();
    keys.into_iter().filter_map(|k| units::parameter(&attributes[k]).map(|(ty, v, written)| (k.clone(), ty, v, written))).collect()
}

struct Reader<'a> {
    ast: &'a Model,
    types: BTreeMap<String, EffectiveType>,
    logical: HashMap<String, &'a LogicalComponent>,
    variations: Vec<variability::Variation>,
    out: Structure,
}

impl<'a> Reader<'a> {
    fn port(&mut self, owner: &str, name: &str, orientation: Orientation, attributes: &HashMap<String, AttributeValue>, taken: &mut Vec<String>) -> Option<Port> {
        let id = format!("{owner}.{name}");
        let inherited = crate::compiler::types::declared_types(attributes).ok().and_then(|names| names.first().and_then(|n| self.types.get(*n))).map(|t| &t.attributes);
        let physics = match connector::physics_of(attributes, inherited) {
            Ok(Some(physics)) => physics,
            Ok(None) => {
                self.out.notes.push(format!("port '{id}' says no physics: not exported (write `physics:` on it or on its definition)"));
                return None;
            }
            Err(error) => {
                self.out.notes.push(format!("port '{id}': {error}: not exported"));
                return None;
            }
        };
        let mut merged = inherited.cloned().unwrap_or_default();
        merged.extend(attributes.iter().map(|(k, v)| (k.clone(), v.clone())));
        match connector::connector(physics, orientation, &merged) {
            Ok(connector) => Some(Port { name: name.to_string(), id, member: names::unique(names::member_name(name), taken), physics, orientation, connector }),
            Err(error) => {
                self.out.notes.push(format!("port '{id}': {error}: not exported"));
                None
            }
        }
    }

    fn component_ports(&mut self, owner: &str, ports: &[ComponentPort], taken: &mut Vec<String>) -> Vec<Port> {
        ports.iter().filter_map(|p| self.port(owner, &p.name, Orientation::from(&p.direction), &p.attributes, taken)).collect()
    }

    /// Own numeric attributes, then those the variants of a realized
    /// variation bring (set per configuration).
    fn params(&mut self, owner: &str, attributes: &HashMap<String, AttributeValue>, realizes: &[String]) -> Vec<Param> {
        let mut params: Vec<Param> = numeric_params(attributes)
            .into_iter()
            .map(|(name, ty, value, written)| Param { name: names::member_name(&name), ty, value: Some(value), comment: written })
            .collect();
        for realized in realizes {
            let Some(variation) = self.variations.iter().find(|v| &v.id == realized).cloned() else { continue };
            let variation_name = self.logical.get(&variation.id).map(|c| c.name.clone()).unwrap_or_else(|| variation.id.clone());
            for (variant, _) in &variation.variants {
                let Some(component) = self.logical.get(variant) else { continue };
                for (name, ty, _, _) in numeric_params(&component.attributes) {
                    let member = names::member_name(&name);
                    match params.iter().find(|p| p.name == member) {
                        Some(existing) if existing.ty != ty => self.out.notes.push(format!(
                            "'{owner}': variant '{variant}' sets {name} as {ty}, the part declares {}: left out",
                            existing.ty
                        )),
                        Some(_) => {}
                        None => params.push(Param { name: member, ty, value: None, comment: format!("set by the variant of {variation_name} each configuration selects") }),
                    }
                }
            }
        }
        params
    }

    fn realized(&self, id: &str, own: impl IntoIterator<Item = String>) -> Vec<String> {
        let mut out: Vec<String> = own.into_iter().collect();
        for trace in &self.ast.traces {
            if trace.from == id && matches!(trace.trace_type.as_str(), "allocates" | "deploys" | "realizes" | "implements") && !out.contains(&trace.to) {
                out.push(trace.to.clone());
            }
        }
        // By written id: a name resolves to the component's id.
        out.into_iter().map(|r| self.logical.get(&r).map(|c| written_id(c)).unwrap_or(r)).collect()
    }

    fn parts(&mut self) {
        let ast = self.ast;
        let mut classes = Vec::new();
        let mut instances = Vec::new();
        for pa in &ast.physical_architecture {
            for node in &pa.nodes {
                let realizes = self.realized(&node.id, node.deployments.iter().map(|d| d.component.clone()));
                let mut taken = Vec::new();
                let mut ports: Vec<Port> = node.ports.iter().filter_map(|p| self.port(&node.id, &p.name, Orientation::None, &p.attributes, &mut taken)).collect();
                ports.extend(self.component_ports(&node.id, &node.component_ports, &mut taken));
                let params = self.params(&node.id, &node.attributes, &realizes);
                let index = self.out.parts.len();
                let class = names::unique(names::class_name(&node.name), &mut classes);
                let instance = names::unique(names::instance_name(&node.name), &mut instances);
                self.out.parts.push(Part {
                    id: node.id.clone(),
                    name: node.name.clone(),
                    description: node.attributes.get("description").map(|d| d.display()),
                    class,
                    instance,
                    params,
                    ports,
                    realizes,
                    node: None,
                    behaviours: Vec::new(),
                });
                let mut members: Vec<String> = self.out.parts[index].ports.iter().map(|p| p.member.clone()).chain(self.out.parts[index].params.iter().map(|p| p.name.clone())).collect();
                let mut behaviour_classes = Vec::new();
                for behaviour in &node.behavior_components {
                    let realizes = self.realized(&behaviour.id, behaviour.allocated_component.clone());
                    let mut taken = Vec::new();
                    let ports = self.component_ports(&behaviour.id, &behaviour.ports, &mut taken);
                    let params = self.params(&behaviour.id, &behaviour.attributes, &realizes);
                    let child = self.out.parts.len();
                    self.out.parts.push(Part {
                        id: behaviour.id.clone(),
                        name: behaviour.name.clone(),
                        description: behaviour.attributes.get("description").map(|d| d.display()),
                        class: names::unique(names::class_name(&behaviour.name), &mut behaviour_classes),
                        instance: names::unique(names::instance_name(&behaviour.name), &mut members),
                        params,
                        ports,
                        realizes,
                        node: Some(index),
                        behaviours: Vec::new(),
                    });
                    self.out.parts[index].behaviours.push(child);
                }
            }
        }
    }

    /// The part and port an end names: `Owner.port`, the owner by id or name.
    fn end(&self, reference: &str) -> Option<(usize, usize)> {
        let (owner, port) = reference.rsplit_once('.')?;
        let part = self.out.parts.iter().position(|p| p.id == owner).or_else(|| self.out.parts.iter().position(|p| p.name == owner))?;
        let port = self.out.parts[part].ports.iter().position(|p| p.name == port)?;
        Some((part, port))
    }

    fn ports_named(&self, reference: &str) -> bool {
        reference.rsplit_once('.').is_some_and(|(owner, _)| self.out.parts.iter().any(|p| p.id == owner || p.name == owner))
    }

    fn connections(&mut self) -> Result<(), String> {
        let ast = self.ast;
        let mut links: Vec<(String, &str, &str)> = Vec::new();
        for pa in &ast.physical_architecture {
            for link in &pa.links {
                let id = link.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&link.name).to_string();
                links.push((id, &link.from, &link.to));
            }
            for exchange in &pa.physical_exchanges {
                links.push((exchange.id.clone(), &exchange.from, &exchange.to));
            }
        }
        for (id, from, to) in links {
            let (Some(a), Some(b)) = (self.end(from), self.end(to)) else {
                if self.ports_named(from) && self.ports_named(to) {
                    self.out.notes.push(format!("'{id}' joins a port that is not exported: left out"));
                }
                continue;
            };
            let node_of = |end: (usize, usize)| self.out.parts[end.0].node.unwrap_or(end.0);
            let within = if node_of(a) == node_of(b) && (self.out.parts[a.0].node.is_some() || self.out.parts[b.0].node.is_some() || a.0 == b.0) {
                Some(node_of(a))
            } else if self.out.parts[a.0].node.is_none() && self.out.parts[b.0].node.is_none() {
                None
            } else {
                self.out.notes.push(format!("'{id}' joins behaviour components of different nodes: route it through ports of the nodes; left out"));
                continue;
            };
            let end = |(part, port): (usize, usize)| {
                let p = &self.out.parts[part].ports[port];
                End { label: p.id.as_str(), physics: p.physics, orientation: p.orientation, outside: within == Some(part) }
            };
            connector::compatible(end(a), end(b)).map_err(|e| format!("'{id}': {e}"))?;
            self.out.connections.push(Connection { id, ends: [a, b], within });
        }
        Ok(())
    }

    fn systems(&mut self) {
        let ast = self.ast;
        let configurations: Vec<(String, String, Vec<String>)> = if ast.configurations.is_empty() {
            vec![(String::new(), "System".to_string(), Vec::new())]
        } else {
            ast.configurations.iter().map(|c| (c.id.clone(), c.name.clone(), c.selects.clone())).collect()
        };
        let mut classes = Vec::new();
        for (id, name, selects) in configurations {
            let excluded = self.excluded(&id);
            let selected: HashSet<String> = selects.iter().map(|s| self.logical.get(s).map(|c| written_id(c)).unwrap_or_else(|| s.clone())).collect();
            let mut nodes = Vec::new();
            for (index, part) in self.out.parts.iter().enumerate().filter(|(_, p)| p.node.is_none()) {
                if !part.realizes.is_empty() && part.realizes.iter().all(|r| excluded.contains(r)) {
                    continue;
                }
                let mut modifiers = self.variant_modifiers(part, &selected);
                for &child in &part.behaviours {
                    let inner = self.variant_modifiers(&self.out.parts[child], &selected);
                    if !inner.is_empty() {
                        modifiers.push(format!("{}({})", self.out.parts[child].instance, inner.join(", ")));
                    }
                }
                nodes.push((index, modifiers));
            }
            let class = names::unique(names::class_name(&name), &mut classes);
            self.out.systems.push(System { id, name, class, selects, nodes });
        }
    }

    /// The logical components a configuration leaves out, with all they hold.
    fn excluded(&self, configuration: &str) -> HashSet<String> {
        if configuration.is_empty() {
            return HashSet::new();
        }
        fn hold(component: &LogicalComponent, out: &mut HashSet<String>) {
            for sub in &component.sub_components {
                out.insert(written_id(sub));
                hold(sub, out);
            }
        }
        let mut out = variability::excluded_by(self.ast, configuration);
        for id in out.clone() {
            if let Some(component) = self.logical.get(&id) {
                hold(component, &mut out);
            }
        }
        out
    }

    fn variant_modifiers(&self, part: &Part, selected: &HashSet<String>) -> Vec<String> {
        let mut out = Vec::new();
        for realized in &part.realizes {
            let Some(variation) = self.variations.iter().find(|v| &v.id == realized) else { continue };
            let Some((variant, _)) = variation.variants.iter().find(|(id, _)| selected.contains(id)) else { continue };
            let Some(component) = self.logical.get(variant) else { continue };
            for (name, _, value, _) in numeric_params(&component.attributes) {
                let member = names::member_name(&name);
                if part.params.iter().any(|p| p.name == member) {
                    out.push(format!("{member}={}", names::real(value)));
                }
            }
        }
        out
    }
}

pub fn read(ast: &Model) -> Result<Structure, String> {
    let mut reader = Reader {
        ast,
        types: effective_types(&ast.types).unwrap_or_default(),
        logical: logical_index(ast),
        variations: variability::variations(ast),
        out: Structure::default(),
    };
    reader.parts();
    reader.connections()?;
    reader.systems();
    Ok(reader.out)
}
