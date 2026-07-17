//! FMI 2.0 export — co-simulation interface contracts.
//!
//! Emits one `modelDescription.xml` (FMI 2.0, Co-Simulation) per component
//! that exposes ports or interfaces. The architecture model is the source of
//! truth for the co-simulation contracts: variable names, causality and
//! types come from the component's ports and typed exchange items; the GUID
//! is the component's deterministic ArcLang UUID, so regenerating the export
//! never re-identifies an FMU.
//!
//! The behaviour inside each FMU is NOT generated — simulation teams
//! implement or wrap it; ArcLang guarantees the interfaces match the
//! architecture.

use super::ast::{Model, PortDirection};
use super::semantic::SemanticModel;

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// FMI type for an exchange-item / data-type reference. Defaults to Real.
fn fmi_type(model: &Model, type_ref: &str) -> &'static str {
    let base = model
        .data_types
        .iter()
        .find(|d| d.id == type_ref || d.name == type_ref)
        .and_then(|d| {
            if d.enumeration_values.is_some() {
                Some("Integer")
            } else {
                d.base_type.as_deref().map(|b| match b {
                    "int" | "integer" | "i32" | "i64" => "Integer",
                    "bool" | "boolean" => "Boolean",
                    "string" | "str" => "String",
                    _ => "Real",
                })
            }
        });
    base.unwrap_or("Real")
}

pub struct FmuDescriptor {
    pub component_id: String,
    pub component_name: String,
    pub xml: String,
}

pub fn generate_fmi_descriptors(model: &SemanticModel, ast: &Model) -> Vec<FmuDescriptor> {
    let mut descriptors = Vec::new();

    // Explicit `port in|out` declarations, per logical component id.
    let mut ast_ports: std::collections::HashMap<String, Vec<(&str, &str)>> =
        std::collections::HashMap::new();
    for la in &ast.logical_architecture {
        fn walk<'a>(
            comp: &'a super::ast::LogicalComponent,
            map: &mut std::collections::HashMap<String, Vec<(&'a str, &'a str)>>,
        ) {
            let id = if comp.id.is_empty() { &comp.name } else { &comp.id }.to_string();
            let entry = map.entry(id).or_default();
            for port in &comp.ports {
                let causality = match port.direction {
                    PortDirection::In => "input",
                    PortDirection::Out | PortDirection::InOut => "output",
                };
                entry.push((port.name.as_str(), causality));
            }
            for sub in &comp.sub_components {
                walk(sub, map);
            }
        }
        for comp in &la.components {
            walk(comp, &mut ast_ports);
        }
    }

    for component in &model.components {
        let mut variables: Vec<(String, &str, &'static str)> = Vec::new(); // (name, causality, type)
        for port in &component.interfaces_in {
            variables.push((port.name.clone(), "input", "Real"));
        }
        for port in &component.interfaces_out {
            variables.push((port.name.clone(), "output", "Real"));
        }
        if let Some(ports) = ast_ports.get(&component.id) {
            for (name, causality) in ports {
                if !variables.iter().any(|(n, _, _)| n == name) {
                    variables.push((name.to_string(), causality, "Real"));
                }
            }
        }
        // Exchanges touching this component contribute typed variables
        for interface in &model.interfaces {
            let root_from = interface.from.split('.').next().unwrap_or(&interface.from);
            let root_to = interface.to.split('.').next().unwrap_or(&interface.to);
            let matches_from =
                root_from == component.id || root_from == component.name;
            let matches_to = root_to == component.id || root_to == component.name;
            if matches_from || matches_to {
                let causality = if matches_from { "output" } else { "input" };
                let var_type = fmi_type(ast, &interface.name);
                let name = interface.name.replace(' ', "_");
                if !variables.iter().any(|(n, _, _)| *n == name) {
                    variables.push((name, causality, var_type));
                }
            }
        }

        if variables.is_empty() {
            continue;
        }

        let identifier = component
            .id
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect::<String>();
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str(&format!(
            "<fmiModelDescription fmiVersion=\"2.0\" modelName=\"{}\" guid=\"{{{}}}\" generationTool=\"ArcLang\" numberOfEventIndicators=\"0\">\n",
            xml_escape(&component.name),
            component.uuid()
        ));
        xml.push_str(&format!(
            "  <CoSimulation modelIdentifier=\"{}\" canHandleVariableCommunicationStepSize=\"true\"/>\n",
            xml_escape(&identifier)
        ));
        xml.push_str("  <ModelVariables>\n");
        let mut output_indexes = Vec::new();
        for (index, (name, causality, var_type)) in variables.iter().enumerate() {
            xml.push_str(&format!(
                "    <ScalarVariable name=\"{}\" valueReference=\"{}\" causality=\"{}\" variability=\"continuous\"{}>\n      <{}{}/>\n    </ScalarVariable>\n",
                xml_escape(name),
                index,
                causality,
                if *causality == "input" { "" } else { " initial=\"calculated\"" },
                var_type,
                if *causality == "input" { " start=\"0\"" } else { "" },
            ));
            if *causality == "output" {
                output_indexes.push(index + 1); // FMI indexes are 1-based
            }
        }
        xml.push_str("  </ModelVariables>\n");
        xml.push_str("  <ModelStructure>\n");
        if !output_indexes.is_empty() {
            xml.push_str("    <Outputs>\n");
            for index in &output_indexes {
                xml.push_str(&format!("      <Unknown index=\"{index}\"/>\n"));
            }
            xml.push_str("    </Outputs>\n");
        }
        xml.push_str("  </ModelStructure>\n");
        xml.push_str("</fmiModelDescription>\n");

        descriptors.push(FmuDescriptor {
            component_id: component.id.clone(),
            component_name: component.name.clone(),
            xml,
        });
    }

    descriptors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptors_carry_stable_guid_and_causality() {
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
        let result = Compiler::new(CompilerConfig::default())
            .compile_string(source)
            .expect("compiles");
        let descriptors = generate_fmi_descriptors(&result.semantic_model, &result.ast);
        let controller = descriptors
            .iter()
            .find(|d| d.component_id == "LC-001")
            .expect("controller FMU");

        // Deterministic identity: guid = the component's ArcLang uuid
        assert!(controller.xml.contains("guid=\"{8006ab91-390c-5908-8464-b353219dfc1f}\""));
        assert!(controller.xml.contains("causality=\"input\""));
        assert!(controller.xml.contains("causality=\"output\""));
        assert!(controller.xml.contains("fmiVersion=\"2.0\""));

        // Well-formed XML (parses without error)
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
