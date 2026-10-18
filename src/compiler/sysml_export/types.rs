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
        for ty in &model.types {
            let links = types_links(e, &ty.name);
            let keyword = if links { "connection" } else if types_functions_only(e, &ty.name) { "action" } else { "part" };
            let base = ty.extends.as_deref().map(|b| format!(" :> {}", text::name(b))).unwrap_or_default();
            e.block(&format!("abstract {keyword} def {}{base}", text::name(&ty.name)), |e| {
                // A connection definition has the two ends its links connect.
                if links && ty.extends.is_none() {
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
