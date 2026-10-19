//! ReqIF → ArcLang. Every SPEC-OBJECT is a requirement, written in the
//! specification that places it (one `system_analysis` per SPECIFICATION,
//! named by it); its SPEC-HIERARCHY parent is its `parent:`; a SPEC-RELATION
//! between two of them is a `refines` trace. The foreign IDENTIFIER of every
//! object and relation is kept as `reqif_id`, so the export gives them back.

use super::xml::{self, Node};
use super::quote;
use std::collections::{HashMap, HashSet};

struct Object<'x> {
    identifier: String,
    node: &'x Node,
}

/// What a foreign attribute name is in ArcLang: one of the requirement's
/// own fields, or `""` (kept as an attribute of its own).
fn field_for(long_name: &str) -> &'static str {
    let bare = long_name.strip_prefix("ReqIF.").or_else(|| long_name.strip_prefix("reqif.")).unwrap_or(long_name);
    let normalized: String = bare.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_lowercase();
    match normalized.as_str() {
        "reqid" | "id" | "identifier" | "foreignid" | "puid" => "id",
        "text" | "description" | "objecttext" => "description",
        "title" | "name" | "heading" | "objectheading" | "chaptername" => "title",
        "priority" => "priority",
        "safetylevel" | "asil" | "dal" | "sil" => "safety_level",
        "category" | "objecttype" | "artifacttype" => "category",
        _ => "",
    }
}

/// The ArcLang key of an attribute that is no field: the requirement
/// attribute of that name when the metamodel has one (an ArcLang export),
/// else `reqif_` and the name in snake case.
fn attribute_key(long_name: &str, known: &HashSet<&'static str>) -> String {
    let mut snake = String::new();
    let mut previous = ' ';
    let bare = long_name.strip_prefix("ReqIF.").or_else(|| long_name.strip_prefix("reqif.")).unwrap_or(long_name);
    for c in bare.chars() {
        if c.is_ascii_alphanumeric() {
            // `ForeignCreatedBy` -> `foreign_created_by`; `NOTES` -> `notes`.
            if c.is_ascii_uppercase() && previous.is_ascii_lowercase() {
                snake.push('_');
            }
            snake.push(c.to_ascii_lowercase());
        } else if !snake.ends_with('_') && !snake.is_empty() {
            snake.push('_');
        }
        previous = c;
    }
    let snake = snake.trim_matches('_').to_string();
    if known.contains(snake.as_str()) {
        snake
    } else {
        format!("reqif_{}", snake.strip_prefix("reqif_").unwrap_or(&snake))
    }
}

/// Attribute definitions and enumeration values: identifier -> LONG-NAME.
fn definitions(root: &Node) -> HashMap<String, String> {
    let mut values: HashMap<String, String> = HashMap::new();
    let mut enums = Vec::new();
    root.descendants("ENUM-VALUE", &mut enums);
    for value in enums {
        if let Some(id) = value.attribute("IDENTIFIER") {
            values.insert(id.to_string(), value.attribute("LONG-NAME").unwrap_or(id).to_string());
        }
    }
    let mut out = HashMap::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if node.name.starts_with("ATTRIBUTE-DEFINITION-") && !node.name.ends_with("-REF") {
            if let Some(id) = node.attribute("IDENTIFIER") {
                out.insert(id.to_string(), node.attribute("LONG-NAME").unwrap_or(id).to_string());
            }
        }
        stack.extend(node.children.iter());
    }
    // Enumeration values are looked up by their own identifier.
    out.extend(values);
    out
}

/// The text of one ATTRIBUTE-VALUE-*.
fn value_text(value: &Node, definitions: &HashMap<String, String>) -> String {
    match value.name.as_str() {
        "ATTRIBUTE-VALUE-XHTML" => value.child("THE-VALUE").map(xml::plain_text).unwrap_or_default(),
        "ATTRIBUTE-VALUE-ENUMERATION" => value
            .child("VALUES")
            .map(|v| v.children.iter().filter_map(|r| definitions.get(r.text.trim()).cloned()).collect::<Vec<_>>().join(", "))
            .unwrap_or_default(),
        _ => value.attribute("THE-VALUE").unwrap_or_default().to_string(),
    }
}

/// The DEFAULT-VALUE of every attribute definition, by SPEC-OBJECT-TYPE:
/// what an object of that type has when it gives no value of its own.
fn defaults(root: &Node, definitions: &HashMap<String, String>) -> HashMap<String, Vec<(String, String)>> {
    let mut types = Vec::new();
    root.descendants("SPEC-OBJECT-TYPE", &mut types);
    let mut out = HashMap::new();
    for spec_type in types {
        let Some(id) = spec_type.attribute("IDENTIFIER") else { continue };
        let mut found = Vec::new();
        for definition in spec_type.child("SPEC-ATTRIBUTES").map(|a| a.children.iter().collect::<Vec<_>>()).unwrap_or_default() {
            let (Some(definition_id), Some(default)) = (definition.attribute("IDENTIFIER"), definition.child("DEFAULT-VALUE").and_then(|d| d.children.first())) else { continue };
            let text = value_text(default, definitions);
            if !text.is_empty() {
                found.push((definition_id.to_string(), text));
            }
        }
        out.insert(id.to_string(), found);
    }
    out
}

/// The (definition, value) pairs of one SPEC-OBJECT, its type's defaults
/// for what it does not give.
fn values(object: &Node, definitions: &HashMap<String, String>, defaults: &HashMap<String, Vec<(String, String)>>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for value in object.child("VALUES").map(|v| v.children.iter().collect::<Vec<_>>()).unwrap_or_default() {
        let Some(definition) = value.child("DEFINITION").and_then(Node::reference) else { continue };
        let text = value_text(value, definitions);
        if !text.is_empty() {
            out.push((definition, text));
        }
    }
    let spec_type = object.child("TYPE").and_then(Node::reference);
    for (definition, text) in spec_type.and_then(|t| defaults.get(&t)).into_iter().flatten() {
        if !out.iter().any(|(d, _)| d == definition) {
            out.push((definition.clone(), text.clone()));
        }
    }
    out
}

/// One placed requirement: the object, its parent object, its specification.
struct Placement {
    object: String,
    parent: Option<String>,
}

fn hierarchy(children: &Node, parent: Option<&str>, out: &mut Vec<Placement>, placed: &mut HashSet<String>) {
    for entry in children.children_named("SPEC-HIERARCHY") {
        let Some(object) = entry.child("OBJECT").and_then(Node::reference) else { continue };
        if placed.insert(object.clone()) {
            out.push(Placement { object: object.clone(), parent: parent.map(str::to_string) });
        }
        if let Some(nested) = entry.child("CHILDREN") {
            hierarchy(nested, Some(&object), out, placed);
        }
    }
}

/// Parse a ReqIF file into ArcLang source.
pub fn import_reqif(text: &str) -> Result<String, String> {
    let document = xml::parse(text)?;
    let root = document.child("REQ-IF").ok_or("ReqIF import: no REQ-IF element")?;
    let definitions = definitions(root);
    let defaults = defaults(root, &definitions);
    let known: HashSet<&'static str> = super::super::metamodel::Metamodel::current().kind("Requirement").map(|k| k.attributes.iter().map(|a| a.key).collect()).unwrap_or_default();

    let mut found = Vec::new();
    root.descendants("SPEC-OBJECT", &mut found);
    let objects: Vec<Object> = found.into_iter().filter_map(|node| Some(Object { identifier: node.attribute("IDENTIFIER")?.to_string(), node })).collect();
    if objects.is_empty() {
        return Err("ReqIF import: no SPEC-OBJECT found in the file".to_string());
    }

    // Written ids: the object's own id field, else a number; unique.
    let mut ids: HashMap<String, String> = HashMap::new();
    let mut taken: HashSet<String> = HashSet::new();
    let mut fields_of: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for (index, object) in objects.iter().enumerate() {
        let pairs: Vec<(String, String)> = values(object.node, &definitions, &defaults)
            .into_iter()
            .map(|(definition, value)| (definitions.get(&definition).cloned().unwrap_or(definition), value))
            .collect();
        let own = pairs.iter().find(|(name, _)| field_for(name) == "id").map(|(_, v)| v.clone());
        let mut id = own.unwrap_or_else(|| format!("REQ-IMPORTED-{:03}", index + 1));
        let base = id.clone();
        let mut n = 1;
        while !taken.insert(id.clone()) {
            n += 1;
            id = format!("{base}-{n}");
        }
        ids.insert(object.identifier.clone(), id);
        fields_of.insert(object.identifier.clone(), pairs);
    }

    // Specifications, and what each places.
    let mut specifications: Vec<(String, Vec<Placement>)> = Vec::new();
    let mut placed: HashSet<String> = HashSet::new();
    let mut found = Vec::new();
    root.descendants("SPECIFICATION", &mut found);
    for specification in found {
        let name = specification.attribute("LONG-NAME").or_else(|| specification.attribute("IDENTIFIER")).unwrap_or("Specification").to_string();
        let mut placements = Vec::new();
        if let Some(children) = specification.child("CHILDREN") {
            hierarchy(children, None, &mut placements, &mut placed);
        }
        specifications.push((name, placements));
    }
    let title = root.child("THE-HEADER").and_then(|h| h.child("REQ-IF-HEADER")).and_then(|h| h.child("TITLE")).map(|t| t.text.trim().to_string()).filter(|t| !t.is_empty());
    let unplaced: Vec<Placement> = objects.iter().filter(|o| !placed.contains(&o.identifier)).map(|o| Placement { object: o.identifier.clone(), parent: None }).collect();
    if !unplaced.is_empty() {
        specifications.push((title.clone().unwrap_or_else(|| "Imported requirements".to_string()), unplaced));
    }

    let model_name: String = title.clone().unwrap_or_else(|| "ImportedRequirements".to_string()).chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    let mut out = String::new();
    out.push_str("// Imported from ReqIF by ArcLang: reqif_id keeps the foreign identity\n");
    out.push_str(&format!("model {model_name} {{\n}}\n"));
    for (name, placements) in &specifications {
        if placements.is_empty() {
            continue;
        }
        out.push_str(&format!("\nsystem_analysis {} {{\n", quote(name)));
        for placement in placements {
            let Some(pairs) = fields_of.get(&placement.object) else { continue };
            write_requirement(&mut out, &ids[&placement.object], &placement.object, placement.parent.as_ref().and_then(|p| ids.get(p)), pairs, &known);
        }
        out.push_str("}\n");
    }

    // Relations between the imported requirements.
    let types: HashMap<String, String> = {
        let mut found = Vec::new();
        root.descendants("SPEC-RELATION-TYPE", &mut found);
        found.iter().filter_map(|t| Some((t.attribute("IDENTIFIER")?.to_string(), t.attribute("LONG-NAME").unwrap_or("").to_string()))).collect()
    };
    let mut relations = Vec::new();
    root.descendants("SPEC-RELATION", &mut relations);
    let mut traces = String::new();
    for relation in relations {
        let end = |name: &str| relation.child(name).and_then(Node::reference).and_then(|r| ids.get(&r).cloned());
        let (Some(source), Some(target)) = (end("SOURCE"), end("TARGET")) else { continue };
        let mut attributes = Vec::new();
        if let Some(kind) = relation.child("TYPE").and_then(Node::reference).and_then(|t| types.get(&t)).filter(|k| !k.is_empty() && *k != "refines") {
            attributes.push(format!("reqif_type: {}", quote(kind)));
        }
        if let Some(identifier) = relation.attribute("IDENTIFIER") {
            attributes.push(format!("reqif_id: {}", quote(identifier)));
        }
        traces.push_str(&format!("trace {} refines {} {{ {} }}\n", quote(&source), quote(&target), attributes.join(" ")));
    }
    if !traces.is_empty() {
        out.push('\n');
        out.push_str(&traces);
    }
    Ok(out)
}

fn write_requirement(out: &mut String, id: &str, identifier: &str, parent: Option<&String>, pairs: &[(String, String)], known: &HashSet<&'static str>) {
    let mut fields: HashMap<&str, &str> = HashMap::new();
    let mut others: Vec<(String, &str)> = Vec::new();
    for (name, value) in pairs {
        match field_for(name) {
            "" => {
                let key = attribute_key(name, known);
                if !others.iter().any(|(k, _)| *k == key) && key != "reqif_id" && key != "parent" {
                    others.push((key, value));
                }
            }
            field => {
                fields.entry(field).or_insert(value);
            }
        }
    }
    out.push_str(&format!("    req {}", quote(id)));
    if let Some(title) = fields.get("title") {
        out.push_str(&format!(" {}", quote(title)));
    }
    out.push_str(" {\n");
    for key in ["description", "priority", "category", "safety_level"] {
        if let Some(value) = fields.get(key) {
            out.push_str(&format!("        {key}: {}\n", quote(value)));
        }
    }
    for (key, value) in others {
        out.push_str(&format!("        {key}: {}\n", quote(value)));
    }
    if let Some(parent) = parent {
        out.push_str(&format!("        parent: {}\n", quote(parent)));
    }
    out.push_str(&format!("        reqif_id: {}\n", quote(identifier)));
    out.push_str("    }\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreign_attribute_names_become_keys() {
        let known: HashSet<&'static str> = ["verification_method"].into_iter().collect();
        assert_eq!(attribute_key("NOTES", &known), "reqif_notes");
        assert_eq!(attribute_key("ReqIF.ForeignCreatedBy", &known), "reqif_foreign_created_by");
        assert_eq!(attribute_key("verification_method", &known), "verification_method");
        assert_eq!(field_for("ReqIF.ChapterName"), "title");
        assert_eq!(field_for("ReqIF.ForeignID"), "id");
    }
}
