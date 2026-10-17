//! Requirements, as Capella's Requirements viewpoint holds them: a module
//! in the system analysis, the requirement types, attribute definitions and
//! relation types in a types folder, each requirement with its ReqIF
//! fields and attribute values, and the relations: an element satisfying a
//! requirement owns an outgoing relation to it, a requirement refining
//! another owns an internal relation.

use super::context::{derived, text, Context};
use super::xml::El;
use crate::compiler::ast::{AttributeValue, Model, Requirement};
use std::collections::BTreeMap;

/// Requirement attributes written as ReqIF fields, not as attribute values.
const FIELDS: &[(&str, &str)] = &[("reqif_name", "ReqIFName"), ("chapter", "ReqIFChapterName"), ("prefix", "ReqIFPrefix"), ("foreign_id", "ReqIFForeignID")];
const HEADER: &[&str] = &["id", "capella_id", "title", "description", "requirement_type", "reqif_name", "chapter", "prefix", "foreign_id"];

/// The requirements of the model, but the STPA system constraints.
pub(super) fn all(model: &Model) -> Vec<&Requirement> {
    let stpa: Vec<*const Requirement> = super::stpa::constraints(model).into_iter().map(|r| r as *const Requirement).collect();
    model.system_analysis.iter().flat_map(|sa| sa.requirements.iter()).filter(|r| !stpa.contains(&(*r as *const Requirement))).collect()
}

fn capella_id(r: &Requirement) -> String {
    text(&r.attributes, "capella_id").map(str::to_string).unwrap_or_else(|| derived(&format!("requirement:{}", r.id)))
}

pub(super) fn register(context: &mut Context, model: &Model) {
    for r in all(model) {
        let id = capella_id(r);
        context.register(&r.id, &id);
        context.register(&id, &id);
        context.requirements.insert(id);
    }
}

/// The kind of value attribute an attribute value is written with.
fn value_kind(value: &AttributeValue) -> (&'static str, &'static str, String) {
    match value {
        AttributeValue::Boolean(b) => ("Requirements:BooleanValueAttribute", "Boolean", b.to_string()),
        AttributeValue::Number(n) if n.fract() == 0.0 => ("Requirements:IntegerValueAttribute", "Integer", format!("{}", *n as i64)),
        AttributeValue::Number(n) => ("Requirements:RealValueAttribute", "Real", n.to_string()),
        AttributeValue::String(s) if s == "true" || s == "false" => ("Requirements:BooleanValueAttribute", "Boolean", s.clone()),
        other => ("Requirements:StringValueAttribute", "String", other.as_string().map(str::to_string).unwrap_or_else(|| other.display())),
    }
}

/// The module and the types folder, for the system analysis' `ownedExtensions`.
pub(super) fn write(context: &Context, model: &Model, level: &str) -> Vec<El> {
    let requirements = all(model);
    if requirements.is_empty() {
        return Vec::new();
    }
    let folder_id = derived(&format!("{level}:requirement-types"));
    // Types, attribute definitions (by key, with their data type) and relation types.
    let mut types: BTreeMap<String, String> = BTreeMap::new();
    let mut definitions: BTreeMap<String, (String, &'static str)> = BTreeMap::new();
    for r in &requirements {
        if let Some(kind) = text(&r.attributes, "requirement_type") {
            types.entry(kind.to_string()).or_insert_with(|| derived(&format!("{folder_id}:type:{kind}")));
        }
        for (key, value) in &r.attributes {
            if HEADER.contains(&key.as_str()) {
                continue;
            }
            let (_, data, _) = value_kind(value);
            definitions.entry(key.clone()).or_insert_with(|| (derived(&format!("{folder_id}:definition:{key}")), data));
        }
    }
    // Relation types: the `relation:` names of the requirement traces.
    let mut relation_types: Vec<String> = model.traces.iter().filter_map(|t| text(&t.attributes, "relation").map(str::to_string)).collect();
    relation_types.sort();
    relation_types.dedup();
    if relation_types.is_empty() {
        relation_types.push("Relation".to_string());
    }
    let mut folder = El::new("ownedExtensions", "CapellaRequirements:CapellaTypesFolder", &folder_id).attr("ReqIFLongName", "Types");
    let data_types: Vec<&str> = {
        let mut kinds: Vec<&str> = definitions.values().map(|(_, k)| *k).collect();
        kinds.sort();
        kinds.dedup();
        kinds
    };
    for kind in &data_types {
        folder.push(El::new("ownedDefinitionTypes", "Requirements:DataTypeDefinition", &derived(&format!("{folder_id}:data:{kind}"))).attr("ReqIFLongName", kind));
    }
    // Each requirement type defines the attributes; a requirement's values
    // name the definitions of its own type.
    let untyped = derived(&format!("{folder_id}:type:Requirement"));
    let definition_of = |type_id: &str, key: &str| derived(&format!("{type_id}:definition:{key}"));
    let mut all_types: Vec<(String, String)> = types.iter().map(|(n, i)| (n.clone(), i.clone())).collect();
    if requirements.iter().any(|r| text(&r.attributes, "requirement_type").is_none()) && !definitions.is_empty() {
        all_types.push(("Requirement".to_string(), untyped.clone()));
    }
    for (name, id) in &all_types {
        let mut t = El::new("ownedTypes", "Requirements:RequirementType", id).attr("ReqIFLongName", name);
        for (key, (_, kind)) in &definitions {
            t.push(El::new("ownedAttributes", "Requirements:AttributeDefinition", &definition_of(id, key)).attr("ReqIFLongName", key).attr("definitionType", &format!("#{}", derived(&format!("{folder_id}:data:{kind}")))));
        }
        folder.push(t);
    }
    for name in &relation_types {
        folder.push(El::new("ownedTypes", "Requirements:RelationType", &relation_type_id(level, name)).attr("ReqIFLongName", name));
    }

    let mut module = El::new("ownedExtensions", "CapellaRequirements:CapellaModule", &derived(&format!("{level}:requirements"))).attr("ReqIFLongName", "Requirements");
    for r in &requirements {
        let id = capella_id(r);
        let title = text(&r.attributes, "title").unwrap_or(&r.id);
        let mut el = El::new("ownedRequirements", "Requirements:Requirement", &id).attr("ReqIFIdentifier", &r.id).attr("ReqIFLongName", title);
        if let Some(description) = text(&r.attributes, "description").filter(|d| *d != title) {
            el.set("ReqIFText", description);
        }
        for (key, field) in FIELDS {
            if let Some(value) = text(&r.attributes, key) {
                el.set(field, value);
            }
        }
        if let Some(kind) = text(&r.attributes, "requirement_type").and_then(|k| types.get(k)) {
            el.set("requirementType", &format!("#{kind}"));
        }
        let type_id = text(&r.attributes, "requirement_type").and_then(|k| types.get(k)).cloned().unwrap_or_else(|| untyped.clone());
        let mut keys: Vec<&String> = r.attributes.keys().filter(|k| !HEADER.contains(&k.as_str())).collect();
        keys.sort();
        for key in keys {
            let (kind, _, value) = value_kind(&r.attributes[key]);
            let definition = definition_of(&type_id, key);
            el.push(El::new("ownedAttributes", kind, &derived(&format!("{id}:attribute:{key}"))).attr("definition", &format!("#{definition}")).attr("value", &value));
        }
        // A requirement refining another owns the relation.
        for trace in model.traces.iter().filter(|t| t.trace_type == "refines") {
            let (Some(from), Some(to)) = (context.resolve(&trace.from), context.resolve(&trace.to)) else { continue };
            if from == id && context.requirements.contains(to) {
                let relation_type = relation_type_id(level, text(&trace.attributes, "relation").unwrap_or("Relation"));
                el.push(El::new("ownedRelations", "Requirements:InternalRelation", &derived(&format!("{from}->{to}:refines"))).attr("relationType", &format!("#{relation_type}")).attr("source", &format!("#{from}")).attr("target", &format!("#{to}")));
            }
        }
        module.push(el);
    }
    vec![module, folder]
}

/// The id of a relation type, by name.
fn relation_type_id(level: &str, name: &str) -> String {
    derived(&format!("{level}:requirement-types:relation:{name}"))
}

/// Outgoing relations of the elements satisfying a requirement.
pub(super) fn relations(context: &mut Context, engineering: &mut El, model: &Model, level: &str) {
    for trace in model.traces.iter().filter(|t| t.trace_type == "satisfies" || t.trace_type == "refines") {
        let (Some(from), Some(to)) = (context.resolve(&trace.from).map(str::to_string), context.resolve(&trace.to).map(str::to_string)) else { continue };
        let (element, requirement) = match (context.requirements.contains(&from), context.requirements.contains(&to)) {
            (false, true) => (from, to),
            (true, false) => (to, from),
            _ => continue,
        };
        let relation_type = relation_type_id(level, text(&trace.attributes, "relation").unwrap_or("Relation"));
        if let Some(owner) = engineering.find_mut(&element) {
            owner.push(
                El::new("ownedExtensions", "CapellaRequirements:CapellaOutgoingRelation", &derived(&format!("{element}->{requirement}:relation")))
                    .attr("relationType", &format!("#{relation_type}"))
                    .attr("source", &format!("#{element}"))
                    .attr("target", &format!("#{requirement}")),
            );
        }
    }
}
