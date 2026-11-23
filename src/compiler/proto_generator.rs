//! Protobuf (proto3) interface-contract export.
//!
//! One .proto file: enums and messages from the data model, one message per
//! exchange item, and one gRPC service per logical component whose rpcs are
//! its OUTGOING exchanges. protoc then gives Java, C++, Python, Go, TS…
//! for free — one deterministic export, every implementation language.

use super::ast::Model;
use super::semantic::SemanticModel;

fn pascal(raw: &str) -> String {
    let mut out = String::new();
    let mut upper_next = true;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            if upper_next {
                out.push(ch.to_ascii_uppercase());
                upper_next = false;
            } else {
                out.push(ch);
            }
        } else {
            upper_next = true;
        }
    }
    if out.is_empty() {
        out.push('X');
    }
    if out.chars().next().unwrap().is_ascii_digit() {
        out.insert(0, 'X');
    }
    out
}

fn snake(raw: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && prev_lower {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
            prev_lower = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        } else if !out.is_empty() && !out.ends_with('_') {
            out.push('_');
            prev_lower = false;
        }
    }
    out.trim_matches('_').to_string()
}

fn proto_type(attr_type: &str) -> &'static str {
    match attr_type.to_ascii_lowercase().as_str() {
        "int" | "integer" | "i32" => "int32",
        "i64" | "long" => "int64",
        "u32" | "uint" => "uint32",
        "u8" | "byte" => "uint32",
        "float" | "f32" => "float",
        "double" | "f64" | "real" => "double",
        "bool" | "boolean" => "bool",
        "string" | "str" | "text" => "string",
        _ => "bytes",
    }
}

/// A text of the model as it stands in a `//` comment: on one line.
fn one_line(text: &str) -> String {
    text.replace(['\r', '\n'], " ")
}

/// The services to declare: one for each component that sends something,
/// with what it sends, each once. A component the model shows at several
/// levels under one name is one service.
fn services(model: &SemanticModel) -> Vec<(&str, Vec<&str>)> {
    let mut services: Vec<(&str, Vec<&str>)> = Vec::new();
    for comp in &model.components {
        let outgoing = model.interfaces.iter().filter(|i| {
            let root = i.from.split('.').next().unwrap_or(&i.from);
            root == comp.id || root == comp.name
        });
        for iface in outgoing {
            let at = match services.iter().position(|(name, _)| *name == comp.name.as_str()) {
                Some(at) => at,
                None => {
                    services.push((comp.name.as_str(), Vec::new()));
                    services.len() - 1
                }
            };
            if !services[at].1.contains(&iface.name.as_str()) {
                services[at].1.push(iface.name.as_str());
            }
        }
    }
    services
}

/// The names of the model that `generate_proto` would declare twice.
pub fn clashes(model: &SemanticModel, ast: &Model) -> Vec<String> {
    let mut d = super::c_header_generator::Declared::default();
    d.add("", "Ack", "the acknowledgement message");
    for dt in &ast.data_types {
        if let Some(values) = &dt.enumeration_values {
            d.add("", &pascal(&dt.name), &format!("data type '{}'", dt.name));
            let prefix = snake(&dt.name).to_uppercase();
            d.add("", &format!("{prefix}_UNSPECIFIED"), &format!("the unspecified value of '{}'", dt.name));
            for v in values {
                d.add("", &format!("{prefix}_{}", snake(&v.name).to_uppercase()), &format!("value '{}' of '{}'", v.name, dt.name));
            }
        }
    }
    for class in &ast.classes {
        d.add("", &pascal(&class.name), &format!("class '{}'", class.name));
        for field in &class.fields {
            d.add(&format!("class '{}'", class.name), &snake(&field.name), &format!("field '{}'", field.name));
        }
    }
    for item in &ast.exchange_items {
        d.add("", &format!("{}Msg", pascal(&item.name)), &format!("exchange item '{}'", item.name));
        for attr in &item.attributes {
            d.add(&format!("exchange item '{}'", item.name), &snake(&attr.name), &format!("attribute '{}'", attr.name));
        }
    }
    for (component, sent) in services(model) {
        d.add("", &format!("{}Service", pascal(component)), &format!("component '{component}'"));
        for exchange in sent {
            d.add(&format!("component '{component}'"), &pascal(exchange), &format!("exchange '{exchange}'"));
        }
    }
    d.clashes()
}

pub fn generate_proto(model: &SemanticModel, ast: &Model) -> String {
    let model_name = model.name.clone().unwrap_or_else(|| "arclang_model".into());
    let mut out = String::new();

    out.push_str(&format!(
        "// Interface contracts generated by ArcLang from model \"{}\".\n\
         // Contracts only; regeneration is deterministic.\n\
         syntax = \"proto3\";\n\npackage {};\n\n",
        one_line(&model_name),
        snake(&model_name)
    ));

    // ---- Enums ----
    for dt in &ast.data_types {
        if let Some(values) = &dt.enumeration_values {
            let name = pascal(&dt.name);
            out.push_str(&format!("enum {name} {{\n"));
            out.push_str(&format!(
                "  {}_UNSPECIFIED = 0;\n",
                snake(&dt.name).to_uppercase()
            ));
            for (i, v) in values.iter().enumerate() {
                out.push_str(&format!(
                    "  {}_{} = {};\n",
                    snake(&dt.name).to_uppercase(),
                    snake(&v.name).to_uppercase(),
                    i + 1
                ));
            }
            out.push_str("}\n\n");
        }
    }

    // ---- Messages from classes ----
    for class in &ast.classes {
        out.push_str(&format!("message {} {{\n", pascal(&class.name)));
        for (i, field) in class.fields.iter().enumerate() {
            out.push_str(&format!(
                "  {} {} = {};\n",
                proto_type(&field.attr_type),
                snake(&field.name),
                i + 1
            ));
        }
        out.push_str("}\n\n");
    }

    // ---- Messages from exchange items ----
    for item in &ast.exchange_items {
        out.push_str(&format!(
            "// mechanism: {}\nmessage {}Msg {{\n",
            one_line(&item.stereotype),
            pascal(&item.name)
        ));
        if item.attributes.is_empty() {
            out.push_str("  bytes payload = 1;\n");
        }
        for (i, attr) in item.attributes.iter().enumerate() {
            out.push_str(&format!(
                "  {} {} = {};\n",
                proto_type(&attr.attr_type),
                snake(&attr.name),
                i + 1
            ));
        }
        out.push_str("}\n\n");
    }

    out.push_str("message Ack {}\n\n");

    // ---- One service per component with outgoing exchanges ----
    let msg_for = |exchange_name: &str| -> String {
        for item in &ast.exchange_items {
            if snake(&item.name) == snake(exchange_name) {
                return format!("{}Msg", pascal(&item.name));
            }
        }
        "Ack".into()
    };
    for (component, sent) in services(model) {
        out.push_str(&format!("service {}Service {{\n", pascal(component)));
        for exchange in sent {
            out.push_str(&format!("  rpc {}({}) returns (Ack);\n", pascal(exchange), msg_for(exchange)));
        }
        out.push_str("}\n\n");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Compiler, CompilerConfig};

    #[test]
    fn proto_covers_enums_messages_and_services() {
        let flagship = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/automotive/emergency-braking/complete_emergency_braking_simple.arc");
        let result = Compiler::new(CompilerConfig::default())
            .compile_file(&flagship)
            .expect("compiles");
        let proto = generate_proto(&result.semantic_model, &result.ast);

        assert!(proto.contains("syntax = \"proto3\";"));
        assert!(proto.contains("enum "));
        assert!(proto.contains("message "));
        assert!(proto.contains("service "));
        assert!(proto.contains("rpc "));
        // enum zero value convention respected
        assert!(proto.contains("_UNSPECIFIED = 0;"));
        // deterministic
        assert_eq!(proto, generate_proto(&result.semantic_model, &result.ast));
    }
}
