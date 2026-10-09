//! Validation of a parsed model against the metamodel.
//!
//! Walks every element of the AST, looks up its kind in [`Metamodel`], and
//! checks each *declared* attribute against its type: quantities must have
//! the right dimension, enumeration values must be valid, references must
//! resolve, lists must be lists. Attributes the metamodel does not declare
//! are free (ArcLang models carry project-specific metadata); only typed
//! attributes are checked.
//!
//! Severity policy (semver MINOR: no previously valid model stops
//! compiling):
//! - `Error` — a type violation (`latency: 135 MHz`, `asil: "High"`). The
//!   compiler reports it as a warning; the production gate makes it a
//!   blocker.
//! - `Warning` — advisory (a bare number where a quantity is expected: the
//!   unit is assumed, the model should state it).

use super::ast::*;
use super::metamodel::{AttrType, Metamodel};
use super::quantity::QuantityError;
use super::semantic::SemanticModel;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Level {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Diagnostic {
    pub level: Level,
    pub kind: &'static str,
    pub element: String,
    pub attribute: Option<String>,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "metamodel: {} '{}'", self.kind, self.element)?;
        if let Some(attribute) = &self.attribute {
            write!(f, ".{}", attribute)?;
        }
        write!(f, ": {}", self.message)
    }
}

struct Checker<'a> {
    metamodel: &'a Metamodel,
    semantic: &'a SemanticModel,
    requirement_keys: Vec<&'a str>,
    out: Vec<Diagnostic>,
}

/// Check `ast` against the current metamodel.
pub fn check_model(ast: &Model, semantic: &SemanticModel) -> Vec<Diagnostic> {
    let metamodel = Metamodel::current();
    check_model_with(&metamodel, ast, semantic)
}

pub fn check_model_with(metamodel: &Metamodel, ast: &Model, semantic: &SemanticModel) -> Vec<Diagnostic> {
    let mut requirement_keys: Vec<&str> = semantic.requirements.iter().map(|r| r.id.as_str()).collect();
    requirement_keys.extend(
        semantic
            .all_elements
            .values()
            .filter(|e| e.element_type == "Requirement")
            .map(|e| e.name.as_str()),
    );
    let mut checker = Checker { metamodel, semantic, requirement_keys, out: Vec::new() };
    checker.walk(ast);
    checker.out
}

impl<'a> Checker<'a> {
    fn walk(&mut self, ast: &Model) {
        for oa in &ast.operational_analysis {
            for actor in &oa.actors {
                self.check_attrs("Actor", &actor.name, &actor.attributes);
            }
            for entity in &oa.entities {
                self.check_attrs("OperationalEntity", &entity.id, &entity.attributes);
                for activity in &entity.activities {
                    self.walk_activity(activity);
                }
            }
            for capability in &oa.capabilities {
                self.check_attrs("OperationalCapability", &capability.id, &capability.attributes);
            }
            for activity in &oa.activities {
                self.walk_activity(activity);
            }
            for exchange in oa.exchanges.iter().chain(oa.communication_means.iter()) {
                let label = exchange.label.clone().unwrap_or_else(|| format!("{} -> {}", exchange.from, exchange.to));
                self.check_attrs("OperationalExchange", &label, &exchange.attributes);
            }
            for process in &oa.processes {
                self.check_attrs("OperationalProcess", &process.id, &process.attributes);
            }
        }

        for sa in &ast.system_analysis {
            for requirement in &sa.requirements {
                self.check_attrs("Requirement", &requirement.id, &requirement.attributes);
            }
            for function in &sa.functions {
                self.walk_system_function(function);
            }
            for component in &sa.components {
                self.check_attrs("SystemComponent", &component.name, &component.attributes);
            }
            for actor in &sa.external_actors {
                self.check_attrs("SystemActor", &actor.id, &actor.attributes);
            }
            for mission in &sa.missions {
                self.check_attrs("Mission", &mission.id, &mission.attributes);
            }
            for capability in &sa.capabilities {
                self.check_attrs("Capability", &capability.id, &capability.attributes);
            }
            for chain in &sa.functional_chains {
                self.check_attrs("FunctionalChain", &chain.id, &chain.attributes);
            }
        }

        for la in &ast.logical_architecture {
            for component in &la.components {
                self.walk_logical_component(component);
            }
            for interface in &la.interfaces {
                self.check_attrs("LogicalInterface", &interface.name, &interface.attributes);
            }
            for capability in &la.capability_realizations {
                self.check_attrs("CapabilityRealization", &capability.id, &capability.attributes);
            }
            for chain in &la.functional_chains {
                self.check_attrs("FunctionalChain", &chain.id, &chain.attributes);
            }
        }

        for pa in &ast.physical_architecture {
            for node in &pa.nodes {
                self.check_attrs("PhysicalNode", &node.id, &node.attributes);
                for port in &node.ports {
                    self.check_attrs("PhysicalPort", &format!("{}.{}", node.id, port.name), &port.attributes);
                }
                for deployment in &node.deployments {
                    self.check_attrs("Deployment", &deployment.component, &deployment.attributes);
                }
            }
            for link in &pa.links {
                self.check_attrs("PhysicalLink", &link.name, &link.attributes);
                if let Some(bandwidth) = &link.bandwidth {
                    if !link.attributes.contains_key("bandwidth") {
                        self.check_value("PhysicalLink", &link.name, "bandwidth",
                            &AttrType::Quantity(super::quantity::Dimension::DataRate),
                            &AttributeValue::String(bandwidth.clone()));
                    }
                }
            }
            for exchange in &pa.physical_exchanges {
                let label = exchange.label.clone().unwrap_or_else(|| format!("{} -> {}", exchange.from, exchange.to));
                if let Some(frequency) = &exchange.frequency {
                    self.check_value("PhysicalExchange", &label, "frequency",
                        &AttrType::Quantity(super::quantity::Dimension::Frequency),
                        &AttributeValue::String(frequency.clone()));
                }
            }
            for path in &pa.paths {
                self.check_attrs("PhysicalPath", &path.name, &path.attributes);
            }
        }

        for epbs in &ast.epbs {
            for system in &epbs.systems {
                self.check_attrs("EpbsSystem", &system.name, &system.attributes);
                for subsystem in &system.subsystems {
                    self.check_attrs("EpbsSubsystem", &subsystem.name, &subsystem.attributes);
                    for item in &subsystem.items {
                        self.check_attrs("EpbsItem", &item.name, &item.attributes);
                    }
                }
            }
        }

        for safety in &ast.safety_analysis {
            for hazard in &safety.hazards {
                self.check_attrs("Hazard", &hazard.name, &hazard.attributes);
            }
            for entry in &safety.fmea {
                self.check_attrs("FmeaEntry", &entry.name, &entry.attributes);
            }
        }

        for class in &ast.classes {
            self.check_attrs("Class", &class.name, &class.attributes);
        }
        for data_type in &ast.data_types {
            if let Some(unit) = &data_type.unit {
                self.check_value("DataType", &data_type.name, "unit", &AttrType::Enum("Unit"), &AttributeValue::String(unit.clone()));
            }
        }
        for test_case in &ast.test_cases {
            self.check_attrs("TestCase", &test_case.name, &test_case.attributes);
        }
        for machine in &ast.state_machines {
            for transition in &machine.transitions {
                if let Some(timing) = &transition.timing {
                    let label = format!("{}: {} -> {}", machine.name, transition.from, transition.to);
                    self.check_value("Transition", &label, "timing",
                        &AttrType::Quantity(super::quantity::Dimension::Time),
                        &AttributeValue::String(timing.clone()));
                }
            }
        }

        self.check_traces(ast);
        self.check_exchange_ports(ast);
    }

    /// An exchange endpoint written `Owner.port` must name a port or
    /// interface that `Owner` declares. (Owners themselves are resolved by
    /// the semantic analyzer.)
    fn check_exchange_ports(&mut self, ast: &Model) {
        let mut component_ports: HashMap<String, Vec<String>> = HashMap::new();
        for la in &ast.logical_architecture {
            for component in &la.components {
                collect_component_ports(component, &mut component_ports);
            }
        }
        let mut function_ports: HashMap<String, Vec<String>> = HashMap::new();
        for sa in &ast.system_analysis {
            for function in &sa.functions {
                collect_function_ports(function, &mut function_ports);
            }
        }
        for la in &ast.logical_architecture {
            for exchange in &la.component_exchanges {
                let label = exchange.label.clone().unwrap_or_else(|| format!("{} -> {}", exchange.from_port, exchange.to_port));
                for (key, endpoint) in [("from_port", &exchange.from_port), ("to_port", &exchange.to_port)] {
                    self.check_port_endpoint("ComponentExchange", &label, key, endpoint, &component_ports, "port or interface");
                }
            }
        }
        for sa in &ast.system_analysis {
            for exchange in &sa.functional_exchanges {
                let label = exchange.label.clone().unwrap_or_else(|| format!("{} -> {}", exchange.from_port, exchange.to_port));
                for (key, endpoint) in [("from", &exchange.from_port), ("to", &exchange.to_port)] {
                    self.check_port_endpoint("FunctionalExchange", &label, key, endpoint, &function_ports, "port");
                }
            }
        }
    }

    fn check_port_endpoint(
        &mut self,
        kind: &'static str,
        element: &str,
        key: &str,
        endpoint: &str,
        ports: &HashMap<String, Vec<String>>,
        what: &str,
    ) {
        let Some((owner, port)) = endpoint.rsplit_once('.') else { return };
        let Some(declared) = ports.get(owner) else { return };
        if !declared.iter().any(|p| p == port) {
            let available = if declared.is_empty() { "none".to_string() } else { declared.join(", ") };
            self.push(Level::Error, kind, element, Some(key),
                format!("'{}' declares no {} named '{}' (declared: {})", owner, what, port, available));
        }
    }

    fn walk_activity(&mut self, activity: &OperationalActivity) {
        self.check_attrs("OperationalActivity", &activity.id, &activity.attributes);
        for sub in &activity.sub_activities {
            self.walk_activity(sub);
        }
    }

    fn walk_system_function(&mut self, function: &SystemFunction) {
        self.check_attrs("SystemFunction", &function.id, &function.attributes);
        for sub in &function.sub_functions {
            self.walk_system_function(sub);
        }
    }

    fn walk_logical_component(&mut self, component: &LogicalComponent) {
        self.check_attrs("LogicalComponent", &component.id, &component.attributes);
        for function in &component.functions {
            self.check_attrs("LogicalFunction", &function.name, &function.attributes);
        }
        for sub in &component.sub_components {
            self.walk_logical_component(sub);
        }
    }

    /// Trace kind rules: the target of `satisfies`/`verifies`/`validates`
    /// must be a requirement; `refines` links requirements.
    fn check_traces(&mut self, ast: &Model) {
        let traces = ast
            .traces
            .iter()
            .chain(ast.operational_analysis.iter().flat_map(|oa| oa.traces.iter()));
        for trace in traces {
            let label = format!("{} {} {}", trace.from, trace.trace_type, trace.to);
            let target_kind = self.kind_of(&trace.to);
            let source_kind = self.kind_of(&trace.from);
            match trace.trace_type.as_str() {
                "satisfies" | "verifies" | "validates" => {
                    if let Some(kind) = target_kind {
                        if kind != "Requirement" && !(trace.trace_type == "validates" && kind.contains("Capability")) {
                            self.push(Level::Warning, "Trace", &label, None,
                                format!("'{}' targets a {} — `{}` links must point at a requirement", trace.trace_type, kind, trace.trace_type));
                        }
                    }
                }
                "refines" => {
                    if let (Some(source), Some(target)) = (source_kind, target_kind) {
                        if source != "Requirement" || target != "Requirement" {
                            self.push(Level::Warning, "Trace", &label, None,
                                "`refines` links a requirement to the requirement it refines".to_string());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn kind_of(&self, reference: &str) -> Option<&'a str> {
        self.semantic.all_elements.get(reference).map(|e| e.element_type.as_str())
    }

    fn check_attrs(&mut self, kind: &'static str, element: &str, attributes: &HashMap<String, AttributeValue>) {
        let Some(spec) = self.metamodel.kind(kind) else { return };
        // Deterministic order: diagnostics must not depend on hash iteration.
        let mut typed: Vec<(&str, &AttrType)> = spec
            .attributes
            .iter()
            // `is:` (one type or a list) is resolved by `types::resolve`.
            .filter(|a| a.key != "is" && attributes.contains_key(a.key))
            .map(|a| (a.key, &a.ty))
            .collect();
        typed.sort_by(|a, b| a.0.cmp(b.0));
        for (key, ty) in typed {
            self.check_value(kind, element, key, ty, &attributes[key]);
        }
    }

    fn check_value(&mut self, kind: &'static str, element: &str, key: &str, ty: &AttrType, value: &AttributeValue) {
        match ty {
            AttrType::Text | AttrType::Identifier => {
                if matches!(value, AttributeValue::Map(_)) {
                    self.push(Level::Error, kind, element, Some(key), "expected text, got a nested block".to_string());
                }
            }
            AttrType::Number => match value {
                AttributeValue::Number(_) => {}
                AttributeValue::String(s) if s.trim().parse::<f64>().is_ok() => {}
                other => self.push(Level::Error, kind, element, Some(key), format!("expected a number, got '{}'", other.display())),
            },
            AttrType::Boolean => match value {
                AttributeValue::Boolean(_) => {}
                AttributeValue::String(s) if matches!(s.to_ascii_lowercase().as_str(), "true" | "false" | "yes" | "no") => {}
                other => self.push(Level::Error, kind, element, Some(key), format!("expected true/false, got '{}'", other.display())),
            },
            AttrType::Quantity(dimension) => match value.as_quantity() {
                None => self.push(Level::Error, kind, element, Some(key),
                    format!("expected a {} quantity, got '{}'", dimension.label(), value.display())),
                Some(Ok(quantity)) => {
                    if quantity.dimension() != *dimension {
                        self.push(Level::Error, kind, element, Some(key),
                            format!("'{}' is a {}, expected a {} (e.g. `{}: 1 {}`)",
                                quantity, quantity.dimension().label(), dimension.label(), key, dimension.canonical_unit()));
                    }
                }
                Some(Err(QuantityError::MissingUnit(number))) => self.push(Level::Warning, kind, element, Some(key),
                    format!("{} has no unit — write `{}: {} <unit>` (a {} such as {})",
                        value.display(), key, number, dimension.label(), dimension.canonical_unit())),
                Some(Err(error)) => self.push(Level::Error, kind, element, Some(key),
                    format!("{} — expected a {} quantity", error, dimension.label())),
            },
            AttrType::Enum(name) => match value {
                AttributeValue::String(_) | AttributeValue::Number(_) => {
                    let raw = value.display();
                    if self.metamodel.normalize_enum(name, &raw).is_none() {
                        let values = self
                            .metamodel
                            .enumeration(name)
                            .map(|e| e.values.join(", "))
                            .unwrap_or_default();
                        let hint = if values.is_empty() { String::new() } else { format!(" (expected one of: {})", values) };
                        self.push(Level::Error, kind, element, Some(key), format!("'{}' is not a valid {}{}", raw, name, hint));
                    }
                }
                other => self.push(Level::Error, kind, element, Some(key),
                    format!("expected a {} value, got '{}'", name, other.display())),
            },
            AttrType::Multiplicity => {
                if let Err(reason) = super::multiplicity::Multiplicity::from_value(value) {
                    self.push(Level::Error, kind, element, Some(key), reason.to_string());
                }
            }
            AttrType::Reference(target_kind) => match value {
                AttributeValue::String(reference) => self.check_reference(kind, element, key, target_kind, reference),
                other => self.push(Level::Error, kind, element, Some(key),
                    format!("expected a reference to a {}, got '{}'", target_kind, other.display())),
            },
            AttrType::List(inner) => match value {
                AttributeValue::List(items) => {
                    for item in items {
                        self.check_value(kind, element, key, inner, item);
                    }
                }
                scalar => {
                    self.push(Level::Warning, kind, element, Some(key),
                        format!("expected a list (`{}: [...]`), got a single value", key));
                    self.check_value(kind, element, key, inner, scalar);
                }
            },
        }
    }

    fn check_reference(&mut self, kind: &'static str, element: &str, key: &str, target_kind: &str, reference: &str) {
        if target_kind == "Requirement" {
            if !self.requirement_keys.contains(&reference) {
                self.push(Level::Error, kind, element, Some(key),
                    format!("references unknown requirement '{}'", reference));
            }
            return;
        }
        // Other references are resolved (and made fatal when dangling) by
        // the semantic analyzer; here only the KIND is checked when known.
        if target_kind == "Element" {
            return;
        }
        if let Some(actual) = self.kind_of(reference) {
            if !actual.contains(target_kind) && !target_kind.contains(actual) {
                self.push(Level::Warning, kind, element, Some(key),
                    format!("'{}' is a {}, expected a {}", reference, actual, target_kind));
            }
        }
    }

    fn push(&mut self, level: Level, kind: &'static str, element: &str, attribute: Option<&str>, message: String) {
        self.out.push(Diagnostic {
            level,
            kind,
            element: element.to_string(),
            attribute: attribute.map(str::to_string),
            message,
        });
    }
}

fn collect_component_ports(component: &LogicalComponent, out: &mut HashMap<String, Vec<String>>) {
    let mut ports: Vec<String> = component.ports.iter().map(|p| p.name.clone()).collect();
    ports.extend(component.interfaces_in.iter().map(|i| i.name.clone()));
    ports.extend(component.interfaces_out.iter().map(|i| i.name.clone()));
    out.insert(component.name.clone(), ports.clone());
    if let Some(id) = component.attributes.get("id").and_then(|v| v.as_string()) {
        out.insert(id.to_string(), ports);
    }
    for sub in &component.sub_components {
        collect_component_ports(sub, out);
    }
}

fn collect_function_ports(function: &SystemFunction, out: &mut HashMap<String, Vec<String>>) {
    let ports: Vec<String> = function.ports.iter().map(|p| p.name.clone()).collect();
    out.insert(function.name.clone(), ports.clone());
    let id = function.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&function.id);
    out.insert(id.to_string(), ports);
    for sub in &function.sub_functions {
        collect_function_ports(sub, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Compiler, CompilerConfig};

    fn diagnostics(source: &str) -> Vec<Diagnostic> {
        let result = Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles");
        check_model(&result.ast, &result.semantic_model)
    }

    #[test]
    fn well_typed_model_has_no_diagnostics() {
        let out = diagnostics(
            r#"
model T {}
system_analysis "SA" {
    requirement "REQ-1" { description: "x" priority: "High" safety_level: ASIL_D }
    function "F" { id: "F-1" latency: 25 ms frequency: "100 Hz" }
    functional_chain "C" { id: "FC-1" involves: ["F-1"] latency_budget: 0.1 s }
}
architecture logical {
    component "LC" { id: "LC-1" asil: "ASIL-D" ram: 512 MB }
}
safety_analysis {
    hazard "H1" { severity: "S3" exposure: "E4" controllability: C3 asil: ASIL_D mitigated_by: ["REQ-1"] }
}
test_case "TC-1" { verifies: ["REQ-1"] method: "test" }
trace "LC-1" satisfies "REQ-1"
"#,
        );
        assert!(out.is_empty(), "{:#?}", out);
    }

    #[test]
    fn wrong_dimension_invalid_enum_and_bad_reference_are_errors() {
        let out = diagnostics(
            r#"
model T {}
system_analysis "SA" {
    requirement "REQ-1" { description: "x" priority: "Urgent" }
    function "F" { id: "F-1" latency: 135 MHz frequency: "Continuous" }
}
architecture logical {
    component "LC" { id: "LC-1" safety_level: "High" }
}
safety_analysis {
    hazard "H1" { severity: "S3" exposure: "E4" controllability: C3 mitigated_by: ["REQ-404"] }
}
"#,
        );
        let errors: Vec<String> = out.iter().filter(|d| d.level == Level::Error).map(|d| d.to_string()).collect();
        assert!(errors.iter().any(|m| m.contains("SystemFunction 'F-1'.latency") && m.contains("frequency, expected a time")), "{:#?}", errors);
        assert!(errors.iter().any(|m| m.contains("'F-1'.frequency") && m.contains("not a quantity")), "{:#?}", errors);
        assert!(errors.iter().any(|m| m.contains("Requirement 'REQ-1'.priority") && m.contains("not a valid Priority")), "{:#?}", errors);
        assert!(errors.iter().any(|m| m.contains("LogicalComponent 'LC-1'.safety_level") && m.contains("'High' is not a valid SafetyLevel")), "{:#?}", errors);
        assert!(errors.iter().any(|m| m.contains("Hazard 'H1'.mitigated_by") && m.contains("unknown requirement 'REQ-404'")), "{:#?}", errors);
        assert_eq!(errors.len(), 5, "{:#?}", errors);
    }

    #[test]
    fn unitless_quantity_is_only_a_warning() {
        let out = diagnostics(
            r#"
model T {}
system_analysis "SA" {
    function "F" { id: "F-1" latency: 5 }
}
"#,
        );
        assert_eq!(out.len(), 1, "{:#?}", out);
        assert_eq!(out[0].level, Level::Warning);
        assert!(out[0].message.contains("has no unit"), "{}", out[0]);
    }

    #[test]
    fn satisfies_pointing_at_a_component_is_flagged() {
        let out = diagnostics(
            r#"
model T {}
architecture logical {
    component "A" { id: "LC-A" }
    component "B" { id: "LC-B" }
}
trace "LC-A" satisfies "LC-B"
"#,
        );
        assert!(out.iter().any(|d| d.kind == "Trace" && d.message.contains("targets a Component")), "{:#?}", out);
    }

    #[test]
    fn exchange_endpoint_must_name_a_declared_port() {
        let out = diagnostics(
            r#"
model T {}
architecture logical {
    component "Camera" { id: "LC-1" interface_out CameraData { protocol: "CSI" } }
    component "Fusion" { id: "LC-2" interface_in CameraIn { protocol: "CSI" } }
    component_exchange "CameraFlow" { from_port: "Camera.CameraData" to_port: "Fusion.FusedIn" }
}
"#,
        );
        let errors: Vec<String> = out.iter().filter(|d| d.level == Level::Error).map(|d| d.to_string()).collect();
        assert_eq!(errors.len(), 1, "{:#?}", out);
        assert!(errors[0].contains("ComponentExchange 'CameraFlow'.to_port") && errors[0].contains("no port or interface named 'FusedIn'") && errors[0].contains("declared: CameraIn"), "{}", errors[0]);
    }

    #[test]
    fn diagnostics_are_ordered_deterministically() {
        let source = r#"
model T {}
architecture logical {
    component "LC" { id: "LC-1" safety_level: "High" asil: "Nope" ram: "lots" power: 3 }
}
"#;
        let first: Vec<String> = diagnostics(source).iter().map(|d| d.to_string()).collect();
        for _ in 0..5 {
            let again: Vec<String> = diagnostics(source).iter().map(|d| d.to_string()).collect();
            assert_eq!(first, again);
        }
        assert_eq!(first.len(), 4);
    }
}
