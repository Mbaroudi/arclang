//! FMI 2.0 export: co-simulation interface contracts.
//!
//! One `modelDescription.xml` (FMI 2.0, Co-Simulation) per logical
//! component, physical behaviour component and physical node that has ports
//! or ends an exchange. A variable per port: `in` is an input, `out` an
//! output, `inout` both (`name_in`, `name_out`, since FMI 2.0 has no
//! bidirectional variable); a port held by a port is `outer_inner`; an
//! exchange that ends on a component without a port is a variable named
//! after the exchange. Types come from what flows (the exchange item or the
//! port's interface, resolved to a data type), Real otherwise; a Real is
//! continuous, an Integer, Boolean or String discrete. Outputs are listed in
//! `<Outputs>` and, being calculated, in `<InitialUnknowns>` (FMI 2.0
//! §2.2.8).
//!
//! The GUID is derived from the model, the component's id and its interface
//! (names, causalities and types): it survives a regeneration and edits that
//! leave the interface alone, changes when the interface changes, and two
//! models do not share it.
//!
//! The behaviour inside each FMU is NOT generated: there are no binaries or
//! sources; simulation teams implement or wrap it, and ArcLang guarantees
//! the interfaces match the architecture.

use super::ast::{AttributeValue, ComponentExchange, ComponentPort, LogicalComponent, Model, PortDirection};
use super::semantic::SemanticModel;
use std::collections::{HashMap, HashSet};

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// The FMI type of a data type, an exchange item or a scalar name; Real otherwise.
fn fmi_type(model: &Model, type_ref: &str) -> &'static str {
    let scalar = |name: &str| match name.to_ascii_lowercase().as_str() {
        "int" | "integer" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" => Some("Integer"),
        "bool" | "boolean" => Some("Boolean"),
        "string" | "str" | "text" => Some("String"),
        "real" | "float" | "double" | "f32" | "f64" => Some("Real"),
        _ => None,
    };
    if let Some(found) = scalar(type_ref) {
        return found;
    }
    model
        .data_types
        .iter()
        .find(|d| d.id == type_ref || d.name == type_ref)
        .and_then(|d| {
            if d.enumeration_values.is_some() {
                return Some("Integer");
            }
            let base = d.base_type.as_deref().or_else(|| d.attributes.get("base_type").and_then(|v| v.as_string()));
            base.and_then(scalar)
        })
        .unwrap_or("Real")
}

pub struct FmuDescriptor {
    pub component_id: String,
    pub component_name: String,
    pub xml: String,
}

/// A component that may get an FMU: its written id, name and ports.
struct Unit<'a> {
    id: String,
    name: String,
    ports: Vec<(String, &'a ComponentPort)>,
    interfaces: Vec<(String, &'static str)>,
}

fn written_id(attributes: &HashMap<String, AttributeValue>, id: &str, name: &str) -> String {
    attributes.get("id").and_then(|v| v.as_string()).filter(|s| !s.is_empty()).unwrap_or(if id.is_empty() { name } else { id }).to_string()
}

/// A port and the ports it holds: `nuts`, `nuts_lugNut`.
fn flatten<'a>(ports: &'a [ComponentPort], prefix: &str, out: &mut Vec<(String, &'a ComponentPort)>) {
    for port in ports {
        let name = if prefix.is_empty() { port.name.clone() } else { format!("{prefix}_{}", port.name) };
        out.push((name.clone(), port));
        flatten(&port.ports, &name, out);
    }
}

fn logical_units<'a>(components: &'a [LogicalComponent], out: &mut Vec<Unit<'a>>) {
    for c in components {
        let mut ports = Vec::new();
        flatten(&c.ports, "", &mut ports);
        let interfaces = c.interfaces_in.iter().map(|i| (i.name.clone(), "input")).chain(c.interfaces_out.iter().map(|i| (i.name.clone(), "output"))).collect();
        out.push(Unit { id: written_id(&c.attributes, &c.id, &c.name), name: c.name.clone(), ports, interfaces });
        logical_units(&c.sub_components, out);
    }
}

fn units(ast: &Model) -> Vec<Unit<'_>> {
    let mut out = Vec::new();
    for la in &ast.logical_architecture {
        logical_units(&la.components, &mut out);
    }
    for pa in &ast.physical_architecture {
        for node in &pa.nodes {
            let mut ports = Vec::new();
            flatten(&node.component_ports, "", &mut ports);
            out.push(Unit { id: written_id(&node.attributes, &node.id, &node.name), name: node.name.clone(), ports, interfaces: Vec::new() });
            for behaviour in &node.behavior_components {
                let mut ports = Vec::new();
                flatten(&behaviour.ports, "", &mut ports);
                out.push(Unit { id: written_id(&behaviour.attributes, &behaviour.id, &behaviour.name), name: behaviour.name.clone(), ports, interfaces: Vec::new() });
            }
        }
    }
    out
}

/// `Owner.port.sub` -> (unit index, Some(`port_sub`)); `Owner` -> (unit, None).
fn end(units: &[Unit<'_>], reference: &str) -> Option<(usize, Option<String>)> {
    let segments: Vec<&str> = reference.split('.').collect();
    (1..=segments.len()).rev().find_map(|k| {
        let key = segments[..k].join(".");
        let unit = units.iter().position(|u| u.id == key).or_else(|| {
            let named: Vec<usize> = (0..units.len()).filter(|&i| units[i].name == key).collect();
            (named.len() == 1).then(|| named[0])
        })?;
        Some((unit, (k < segments.len()).then(|| segments[k..].join("_"))))
    })
}

struct Variable {
    name: String,
    causality: &'static str,
    ty: &'static str,
}

/// The exchanges of the model, every level, as (label, from, to, item).
fn exchanges(ast: &Model) -> Vec<(String, &str, &str, &str)> {
    let label = |x: &ComponentExchange| x.label.clone().filter(|l| !l.is_empty()).unwrap_or_else(|| x.id.clone());
    let mut out = Vec::new();
    for la in &ast.logical_architecture {
        for x in &la.component_exchanges {
            out.push((label(x), x.from_port.as_str(), x.to_port.as_str(), x.exchange_item.as_str()));
        }
    }
    out
}

fn variables(ast: &Model, units: &[Unit<'_>], index: usize, flows: &[(String, &str, &str, &str)]) -> Vec<Variable> {
    let unit = &units[index];
    // What flows through each port of the unit, from the exchanges ending on it.
    let mut items: HashMap<String, &str> = HashMap::new();
    let mut portless: Vec<(String, &'static str, &str)> = Vec::new();
    for (label, from, to, item) in flows {
        for (reference, causality) in [(*from, "output"), (*to, "input")] {
            match end(units, reference) {
                Some((u, Some(port))) if u == index && !item.is_empty() => {
                    items.entry(port).or_insert(*item);
                }
                Some((u, None)) if u == index => portless.push((label.clone(), causality, *item)),
                _ => {}
            }
        }
    }
    let mut out: Vec<Variable> = Vec::new();
    for (name, port) in &unit.ports {
        let flowing = items.get(name).copied().filter(|i| !i.is_empty()).unwrap_or(port.interface_type.as_str());
        let ty = fmi_type(ast, flowing);
        match port.direction {
            PortDirection::In => out.push(Variable { name: name.clone(), causality: "input", ty }),
            PortDirection::Out => out.push(Variable { name: name.clone(), causality: "output", ty }),
            PortDirection::InOut => {
                out.push(Variable { name: format!("{name}_in"), causality: "input", ty });
                out.push(Variable { name: format!("{name}_out"), causality: "output", ty });
            }
        }
    }
    for (name, causality) in &unit.interfaces {
        out.push(Variable { name: name.clone(), causality, ty: "Real" });
    }
    for (name, causality, item) in portless {
        if !out.iter().any(|v| v.name == name) {
            out.push(Variable { name, causality, ty: fmi_type(ast, item) });
        }
    }
    out
}

/// A C identifier for `modelIdentifier`.
fn c_ident(raw: &str) -> String {
    let mut ident: String = raw.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    if !ident.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
        ident.insert(0, '_');
    }
    ident
}

fn scalar_variable(index: usize, variable: &Variable) -> String {
    let variability = if variable.ty == "Real" { "continuous" } else { "discrete" };
    let (initial, start) = match (variable.causality, variable.ty) {
        ("input", "Boolean") => ("", " start=\"false\""),
        ("input", "String") => ("", " start=\"\""),
        ("input", _) => ("", " start=\"0\""),
        _ => (" initial=\"calculated\"", ""),
    };
    format!(
        "    <ScalarVariable name=\"{}\" valueReference=\"{index}\" causality=\"{}\" variability=\"{variability}\"{initial}>\n      <{}{start}/>\n    </ScalarVariable>\n",
        xml_escape(&variable.name),
        variable.causality,
        variable.ty
    )
}

fn description(model_name: &str, unit: &Unit<'_>, variables: &[Variable]) -> String {
    let signature: String = variables.iter().map(|v| format!("{}:{}:{};", v.name, v.causality, v.ty)).collect();
    let guid = super::identity::element_uuid("fmi", &format!("{model_name}\n{}\n{signature}", unit.id));
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(&format!(
        "<fmiModelDescription fmiVersion=\"2.0\" modelName=\"{}\" guid=\"{{{guid}}}\" generationTool=\"ArcLang\" numberOfEventIndicators=\"0\">\n",
        xml_escape(&unit.name)
    ));
    xml.push_str(&format!("  <CoSimulation modelIdentifier=\"{}\" canHandleVariableCommunicationStepSize=\"true\"/>\n", xml_escape(&c_ident(&unit.id))));
    xml.push_str("  <ModelVariables>\n");
    for (index, variable) in variables.iter().enumerate() {
        xml.push_str(&scalar_variable(index, variable));
    }
    xml.push_str("  </ModelVariables>\n  <ModelStructure>\n");
    // FMI indexes are 1-based, in the order of ModelVariables.
    let outputs: Vec<usize> = variables.iter().enumerate().filter(|(_, v)| v.causality == "output").map(|(i, _)| i + 1).collect();
    if !outputs.is_empty() {
        for section in ["Outputs", "InitialUnknowns"] {
            xml.push_str(&format!("    <{section}>\n"));
            for index in &outputs {
                xml.push_str(&format!("      <Unknown index=\"{index}\"/>\n"));
            }
            xml.push_str(&format!("    </{section}>\n"));
        }
    }
    xml.push_str("  </ModelStructure>\n</fmiModelDescription>\n");
    xml
}

pub fn generate_fmi_descriptors(model: &SemanticModel, ast: &Model) -> Vec<FmuDescriptor> {
    let model_name = model.name.clone().filter(|n| !n.is_empty()).unwrap_or_else(|| ast.id.clone());
    let units = units(ast);
    let flows = exchanges(ast);
    let mut taken = HashSet::new();
    let mut descriptors = Vec::new();
    for index in 0..units.len() {
        let variables = variables(ast, &units, index, &flows);
        if variables.is_empty() || !taken.insert(units[index].id.clone()) {
            continue;
        }
        descriptors.push(FmuDescriptor {
            component_id: units[index].id.clone(),
            component_name: units[index].name.clone(),
            xml: description(&model_name, &units[index], &variables),
        });
    }
    descriptors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptors_are_well_formed_with_causality() {
        use crate::compiler::{Compiler, CompilerConfig};
        let source = r#"
model Demo {
}

architecture logical {
    component "Controller" {
        id: "LC-001"
        port in Threat { }
        port out Command { }
        function "compute"
    }
}
"#;
        let result = Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles");
        let descriptors = generate_fmi_descriptors(&result.semantic_model, &result.ast);
        let controller = descriptors.iter().find(|d| d.component_id == "LC-001").expect("controller FMU");
        assert!(controller.xml.contains("causality=\"input\""));
        assert!(controller.xml.contains("causality=\"output\""));
        assert!(controller.xml.contains("fmiVersion=\"2.0\""));
        let mut reader = quick_xml::Reader::from_str(&controller.xml);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("generated XML is not well-formed: {e}"),
            }
            buf.clear();
        }
    }
}
