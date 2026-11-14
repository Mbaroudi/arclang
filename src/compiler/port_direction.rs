//! The direction of the ports an exchange joins: a component or functional
//! exchange leaves an `out` port and arrives on an `in` port (`inout` takes
//! either role). An exchange from an input, or to an output, joins two
//! ends that both receive or both send: the interface cannot carry what the
//! model says it carries. Endpoints written `Owner.port` are checked when
//! the owner (by id or name) and its port are found; a reference that does
//! not resolve is left to the endpoint check.

use super::ast::{ComponentPort, FunctionPort, LogicalComponent, LogicalFunction, Model, PortDirection, SystemFunction};
use std::collections::HashMap;

/// The ports of every owner, by `owner.port` (owner by id and by name).
#[derive(Default)]
struct Ports {
    directions: HashMap<String, PortDirection>,
}

impl Ports {
    fn insert(&mut self, owners: &[&str], port: &str, direction: &PortDirection) {
        for owner in owners.iter().filter(|o| !o.is_empty()) {
            self.directions.entry(format!("{owner}.{port}")).or_insert_with(|| direction.clone());
        }
    }

    fn component(&mut self, owners: &[&str], ports: &[ComponentPort]) {
        for port in ports {
            self.insert(owners, &port.name, &port.direction);
            if !port.id.is_empty() {
                self.directions.entry(port.id.clone()).or_insert_with(|| port.direction.clone());
            }
        }
    }

    fn function_ports(&mut self, owners: &[&str], ports: &[FunctionPort]) {
        for port in ports {
            self.insert(owners, &port.name, &port.direction);
            if !port.id.is_empty() {
                self.directions.entry(port.id.clone()).or_insert_with(|| port.direction.clone());
            }
        }
    }

    fn logical_function(&mut self, function: &LogicalFunction) {
        self.function_ports(&[&function.id, &function.name], &function.ports);
        for sub in &function.sub_functions {
            self.logical_function(sub);
        }
    }

    fn system_function(&mut self, function: &SystemFunction) {
        self.function_ports(&[&function.id, &function.name], &function.ports);
        for sub in &function.sub_functions {
            self.system_function(sub);
        }
    }

    fn logical_component(&mut self, component: &LogicalComponent) {
        self.component(&[&component.id, &component.name], &component.ports);
        for function in &component.functions {
            self.logical_function(function);
        }
        for sub in &component.sub_components {
            self.logical_component(sub);
        }
    }

    fn of(model: &Model) -> Ports {
        let mut ports = Ports::default();
        for sa in &model.system_analysis {
            for function in &sa.functions {
                ports.system_function(function);
            }
            for component in &sa.components {
                let id = component.attributes.get("id").and_then(|v| v.as_string()).unwrap_or_default().to_string();
                ports.component(&[&id, &component.name], &component.ports);
            }
        }
        for la in &model.logical_architecture {
            for component in &la.components {
                ports.logical_component(component);
            }
            for function in &la.functions {
                ports.logical_function(function);
            }
        }
        for pa in &model.physical_architecture {
            for node in &pa.nodes {
                ports.component(&[&node.id, &node.name], &node.component_ports);
                for function in &node.functions {
                    ports.logical_function(function);
                }
                for behavior in &node.behavior_components {
                    ports.component(&[&behavior.id, &behavior.name], &behavior.ports);
                    for function in &behavior.functions {
                        ports.logical_function(function);
                    }
                }
            }
            for function in &pa.functions {
                ports.logical_function(function);
            }
        }
        ports
    }

    fn direction(&self, endpoint: &str) -> Option<&PortDirection> {
        self.directions.get(endpoint)
    }
}

fn word(direction: &PortDirection) -> &'static str {
    match direction {
        PortDirection::In => "in",
        PortDirection::Out => "out",
        PortDirection::InOut => "inout",
    }
}

/// The exchanges of the model whose ends do not send and receive: one
/// warning line each wrong end, `exchange direction: ...`.
pub fn check(model: &Model) -> Vec<String> {
    let ports = Ports::of(model);
    let mut warnings = Vec::new();
    let mut check = |kind: &str, label: &Option<String>, from: &str, to: &str| {
        let name = label.clone().unwrap_or_else(|| format!("{from} -> {to}"));
        if let Some(direction @ PortDirection::In) = ports.direction(from) {
            warnings.push(format!(
                "exchange direction: {kind} '{name}' leaves '{from}', an {} port: an exchange leaves an out (or inout) port",
                word(direction)
            ));
        }
        if let Some(direction @ PortDirection::Out) = ports.direction(to) {
            warnings.push(format!(
                "exchange direction: {kind} '{name}' arrives on '{to}', an {} port: an exchange arrives on an in (or inout) port",
                word(direction)
            ));
        }
    };
    for sa in &model.system_analysis {
        for ex in &sa.functional_exchanges {
            check("functional_exchange", &ex.label, &ex.from_port, &ex.to_port);
        }
        for ex in &sa.component_exchanges {
            check("component_exchange", &ex.label, &ex.from_port, &ex.to_port);
        }
    }
    for la in &model.logical_architecture {
        for ex in &la.component_exchanges {
            check("component_exchange", &ex.label, &ex.from_port, &ex.to_port);
        }
        for ex in &la.functional_exchanges {
            check("functional_exchange", &ex.label, &ex.from_port, &ex.to_port);
        }
    }
    for pa in &model.physical_architecture {
        for ex in &pa.functional_exchanges {
            check("functional_exchange", &ex.label, &ex.from_port, &ex.to_port);
        }
        for ex in &pa.physical_exchanges {
            check("physical_exchange", &ex.label, &ex.from, &ex.to);
        }
    }
    warnings
}
