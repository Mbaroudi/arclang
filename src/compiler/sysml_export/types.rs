//! ArcLang types are SysML definitions, the only ones the export invents
//! nothing for: `type Sensor extends Device { ... }` is `abstract part def
//! Sensor :> Device`, its attributes defaults and its ports; an element
//! `is: Sensor` is typed by it (`part <'LC-1'> Radar : Sensor`) and writes
//! only what it changes, as redefinitions (`attribute :>> rate = 20 [Hz];`).

use super::{attributes, levels, text, Exporter};
use crate::compiler::ast::AttributeValue;
use crate::compiler::types;
use std::collections::HashMap;

pub(super) const PACKAGE: &str = "Types";

pub(super) fn register(e: &mut Exporter<'_>) {
    let model = e.model;
    for ty in &model.types {
        e.register(PACKAGE, &ty.name, &[]);
    }
}

/// Whether physical links are typed by `name`: then it is a connection definition.
fn types_links(e: &Exporter<'_>, name: &str) -> bool {
    e.model.physical_architecture.iter().flat_map(|pa| pa.links.iter()).any(|l| types::declared_types(&l.attributes).map(|n| n.contains(&name)).unwrap_or(false))
}

/// Whether component ports are typed by `name`: then it is a port definition.
fn types_ports(e: &Exporter<'_>, name: &str) -> bool {
    fn any(ports: &[crate::compiler::ast::ComponentPort], name: &str) -> bool {
        ports.iter().any(|p| types::declared_types(&p.attributes).map(|n| n.contains(&name)).unwrap_or(false) || any(&p.ports, name))
    }
    fn components(list: &[crate::compiler::ast::LogicalComponent], name: &str) -> bool {
        list.iter().any(|c| any(&c.ports, name) || components(&c.sub_components, name))
    }
    // Every level declares typed ports: system components, logical
    // components, physical nodes and the behaviour components they host.
    let system = e.model.system_analysis.iter().any(|sa| sa.components.iter().any(|c| any(&c.ports, name)));
    let physical = e.model.physical_architecture.iter().flat_map(|pa| &pa.nodes).any(|n| any(&n.component_ports, name) || n.behavior_components.iter().any(|b| any(&b.ports, name)));
    system || physical || e.model.logical_architecture.iter().any(|la| components(&la.components, name)) || e.model.types.iter().any(|t| any(&t.ports, name))
}

/// Whether exchanges are typed by `name`: then it is an interface definition,
/// written as the connection definition its exchanges are usages of.
fn types_exchanges(e: &Exporter<'_>, name: &str) -> bool {
    let uses = |attributes: &HashMap<String, AttributeValue>| types::declared_types(attributes).map(|n| n.contains(&name)).unwrap_or(false);
    e.model.logical_architecture.iter().any(|la| la.component_exchanges.iter().any(|x| uses(&x.attributes)) || la.functional_exchanges.iter().any(|x| uses(&x.attributes)))
        || e.model.system_analysis.iter().any(|sa| sa.functional_exchanges.iter().any(|x| uses(&x.attributes)))
}

/// Whether allocations are typed by `name`: then it is an allocation definition.
fn types_allocations(e: &Exporter<'_>, name: &str) -> bool {
    e.model.traces.iter().any(|t| t.trace_type == "allocates" && types::declared_types(&t.attributes).map(|n| n.contains(&name)).unwrap_or(false))
}

/// Whether only functions are typed by `name`: then it is an action definition.
fn types_functions_only(e: &Exporter<'_>, name: &str) -> bool {
    let mut functions = false;
    let mut others = false;
    let uses = |attributes: &HashMap<String, AttributeValue>| types::declared_types(attributes).map(|n| n.contains(&name)).unwrap_or(false);
    for sa in &e.model.system_analysis {
        levels::visit_system_functions(&sa.functions, &mut |f| functions |= uses(&f.attributes));
        others |= sa.components.iter().any(|c| uses(&c.attributes)) || sa.external_actors.iter().any(|a| uses(&a.attributes));
    }
    for la in &e.model.logical_architecture {
        levels::visit_logical_functions(la, &mut |f| functions |= uses(&f.attributes));
        fn any(list: &[crate::compiler::ast::LogicalComponent], uses: &dyn Fn(&HashMap<String, AttributeValue>) -> bool) -> bool {
            list.iter().any(|c| uses(&c.attributes) || any(&c.sub_components, uses))
        }
        others |= any(&la.components, &uses);
    }
    for pa in &e.model.physical_architecture {
        others |= pa.nodes.iter().any(|n| uses(&n.attributes));
    }
    functions && !others
}

pub(super) fn write(e: &mut Exporter<'_>) {
    let model = e.model;
    if model.types.is_empty() {
        return;
    }
    e.package(&format!("package {PACKAGE}"), PACKAGE, |e| {
        // The port definitions the ports of the types are of.
        fn interfaces(ports: &[crate::compiler::ast::ComponentPort], out: &mut Vec<String>) {
            for port in ports {
                if types::declared_types(&port.attributes).map(|n| n.is_empty()).unwrap_or(true) {
                    out.push(port.interface_type.clone());
                }
                interfaces(&port.ports, out);
            }
        }
        let mut used = Vec::new();
        for ty in &model.types {
            interfaces(&ty.ports, &mut used);
        }
        if !used.is_empty() {
            levels::port_defs(e, used);
        }
        for ty in &model.types {
            let allocations = types_allocations(e, &ty.name);
            let links = !allocations && (types_links(e, &ty.name) || types_exchanges(e, &ty.name));
            let ports = !links && !allocations && types_ports(e, &ty.name);
            let keyword = if allocations { "allocation" } else if links { "connection" } else if ports { "port" } else if types_functions_only(e, &ty.name) { "action" } else { "part" };
            let base = ty.extends.as_deref().map(|b| format!(" :> {}", text::name(b))).unwrap_or_default();
            e.block(&format!("abstract {keyword} def {}{base}", text::name(&ty.name)), |e| {
                // A connection definition has the two ends its links connect.
                if (links || allocations) && ty.extends.is_none() {
                    e.line("end source;");
                    e.line("end target;");
                }
                if !ty.required.is_empty() {
                    e.line(&text::doc(&format!("instances must provide: {}", ty.required.join(", "))));
                }
                // A definition gives defaults, which its instances may change.
                let inherited = ty.extends.as_deref().and_then(|b| e.types.get(b)).map(|t| t.attributes.clone());
                for line in attributes::lines(&e.metamodel, "LogicalComponent", &ty.attributes, &[], inherited.as_ref(), &mut e.units) {
                    e.line(&line.replacen(" = ", " default = ", 1));
                }
                let inherited_ports: Vec<String> = ty.extends.as_deref().and_then(|b| e.types.get(b)).map(|t| t.ports.iter().map(|p| p.name.clone()).collect()).unwrap_or_default();
                let own: Vec<_> = ty.ports.iter().filter(|p| !inherited_ports.contains(&p.name)).cloned().collect();
                levels::component_ports(e, &own);
            });
        }
    });
}

impl Exporter<'_> {
    /// ` : Sensor`, the types an element `is:`, as their definitions.
    pub(super) fn typing(&self, attributes: &HashMap<String, AttributeValue>) -> String {
        let names: Vec<String> = types::declared_types(attributes).unwrap_or_default().into_iter().filter_map(|n| self.reference(n)).collect();
        if names.is_empty() { String::new() } else { format!(" : {}", names.join(", ")) }
    }

    /// The attributes an element inherits from its types, when it has some.
    pub(super) fn inherited(&self, attributes: &HashMap<String, AttributeValue>) -> Option<types::Inherited> {
        let names = types::declared_types(attributes).ok()?;
        if names.is_empty() {
            return None;
        }
        types::inherit(&self.types, &names).ok()
    }
}
