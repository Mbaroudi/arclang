//! ArcLang → ReqIF 1.0.

use crate::compiler::ast::{AttributeValue, Model, Requirement};
use crate::compiler::identity::element_uuid;
use crate::compiler::semantic::{RequirementInfo, SemanticModel};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Fixed timestamp: deterministic output, meaningful diffs.
const TIMESTAMP: &str = "2000-01-01T00:00:00Z";

/// The requirement fields every SPEC-OBJECT may carry, in order.
const FIELDS: &[(&str, &str)] = &[
    ("AD-REQID", "ReqID"),
    ("AD-TITLE", "Title"),
    ("AD-TEXT", "Text"),
    ("AD-PRIORITY", "Priority"),
    ("AD-CATEGORY", "Category"),
    ("AD-SAFETY", "SafetyLevel"),
];

/// Attributes said by the fields, the structure or the identity.
const NOT_EXTRA: &[&str] = &["id", "name", "title", "description", "priority", "category", "safety_level", "safety", "asil", "reqif_id", "parent"];

fn esc(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\n', "&#10;").replace('\t', "&#9;").replace('\r', "")
}

/// xsd:ID must not start with a digit; ArcLang UUIDs may.
fn xml_id(uuid: &str) -> String {
    format!("_{uuid}")
}

/// An xsd:ID made of a name: letters, digits, `-`, `_` and `.`.
fn token(text: &str) -> String {
    text.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' { c.to_ascii_uppercase() } else { '_' }).collect()
}

fn text_of<'a>(attributes: &'a HashMap<String, AttributeValue>, key: &str) -> Option<&'a str> {
    attributes.get(key).and_then(|v| v.as_string()).filter(|s| !s.is_empty())
}

/// One requirement as the export writes it.
struct Entry<'m> {
    id: String,
    identifier: String,
    requirement: &'m Requirement,
    info: Option<&'m RequirementInfo>,
}

impl<'m> Entry<'m> {
    fn title(&self) -> Option<String> {
        text_of(&self.requirement.attributes, "title").map(str::to_string).or_else(|| (self.requirement.id != self.id).then(|| self.requirement.id.clone()))
    }

    fn parent(&self) -> Option<&'m str> {
        text_of(&self.requirement.attributes, "parent")
    }

    /// The attributes that are none of the fields, by their exported name.
    fn extras(&self) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = self
            .requirement
            .attributes
            .iter()
            .filter(|(k, _)| !NOT_EXTRA.contains(&k.as_str()))
            .map(|(k, v)| (k.strip_prefix("reqif_").unwrap_or(k).to_string(), v.display()))
            .filter(|(_, v)| !v.is_empty())
            .collect();
        out.sort();
        out
    }
}

pub fn generate_reqif(model: &SemanticModel, ast: &Model) -> String {
    generate_reqif_with_notes(model, ast).0
}

/// The ReqIF text, and what it could not carry.
pub fn generate_reqif_with_notes(model: &SemanticModel, ast: &Model) -> (String, Vec<String>) {
    let info: HashMap<&str, &RequirementInfo> = model.requirements.iter().map(|r| (r.id.as_str(), r)).collect();
    // Groups: every block of requirements, named by it.
    let mut groups: Vec<(String, Vec<Entry>)> = Vec::new();
    for sa in &ast.system_analysis {
        let entries: Vec<Entry> = sa
            .requirements
            .iter()
            .map(|requirement| {
                let id = text_of(&requirement.attributes, "id").unwrap_or(&requirement.id).to_string();
                let identifier = text_of(&requirement.attributes, "reqif_id").map(str::to_string).unwrap_or_else(|| xml_id(&element_uuid("element", &id)));
                Entry { info: info.get(id.as_str()).copied(), id, identifier, requirement }
            })
            .collect();
        if !entries.is_empty() {
            groups.push((sa.name.clone(), entries));
        }
    }
    let title = model.name.as_deref().unwrap_or("ArcLang Requirements");
    let entries: Vec<&Entry> = groups.iter().flat_map(|(_, e)| e.iter()).collect();
    let identifier_of: HashMap<&str, &str> = entries.iter().map(|e| (e.id.as_str(), e.identifier.as_str())).collect();
    let requirement_ids: HashSet<&str> = identifier_of.keys().copied().collect();

    // Traces: between requirements, a relation; to anything else, a note.
    let mut relations = Vec::new();
    let mut left_out: BTreeMap<&str, usize> = BTreeMap::new();
    for trace in &model.traces {
        let (from, to) = (requirement_ids.contains(trace.from.as_str()), requirement_ids.contains(trace.to.as_str()));
        if from && to {
            let declared = ast.traces.iter().find(|t| t.from == trace.from && t.to == trace.to && t.trace_type == trace.trace_type);
            let kind = declared.and_then(|t| text_of(&t.attributes, "reqif_type")).unwrap_or(&trace.trace_type).to_string();
            let identifier = declared.and_then(|t| text_of(&t.attributes, "reqif_id")).map(str::to_string).unwrap_or_else(|| xml_id(&element_uuid("trace", &format!("{}|{}|{}", trace.from, trace.trace_type, trace.to))));
            relations.push((identifier, identifier_of[trace.from.as_str()].to_string(), identifier_of[trace.to.as_str()].to_string(), kind));
        } else if from || to {
            *left_out.entry(trace.trace_type.as_str()).or_default() += 1;
        }
    }
    let mut notes = Vec::new();
    let count: usize = left_out.values().sum();
    if count > 0 {
        let kinds: Vec<String> = left_out.iter().map(|(k, n)| format!("{k}: {n}")).collect();
        notes.push(format!("ReqIF carries requirements only: {count} trace(s) between a requirement and another element left out ({})", kinds.join(", ")));
    }

    let mut extra_names: Vec<String> = entries.iter().flat_map(|e| e.extras().into_iter().map(|(k, _)| k)).collect();
    extra_names.sort();
    extra_names.dedup();
    let mut relation_kinds: Vec<String> = relations.iter().map(|r| r.3.clone()).collect();
    relation_kinds.sort();
    relation_kinds.dedup();

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<REQ-IF xmlns=\"http://www.omg.org/spec/ReqIF/20110401/reqif.xsd\">\n");
    out.push_str("  <THE-HEADER>\n");
    out.push_str(&format!("    <REQ-IF-HEADER IDENTIFIER=\"{}\">\n", xml_id(&element_uuid("reqif-header", title))));
    out.push_str(&format!("      <CREATION-TIME>{TIMESTAMP}</CREATION-TIME>\n"));
    out.push_str("      <REQ-IF-TOOL-ID>ArcLang</REQ-IF-TOOL-ID>\n      <REQ-IF-VERSION>1.0</REQ-IF-VERSION>\n      <SOURCE-TOOL-ID>ArcLang</SOURCE-TOOL-ID>\n");
    out.push_str(&format!("      <TITLE>{}</TITLE>\n", esc(title)));
    out.push_str("    </REQ-IF-HEADER>\n  </THE-HEADER>\n  <CORE-CONTENT>\n    <REQ-IF-CONTENT>\n");
    out.push_str(&format!("      <DATATYPES>\n        <DATATYPE-DEFINITION-STRING IDENTIFIER=\"DT-STRING\" LONG-NAME=\"String\" MAX-LENGTH=\"32000\" LAST-CHANGE=\"{TIMESTAMP}\"/>\n      </DATATYPES>\n"));
    out.push_str("      <SPEC-TYPES>\n");
    out.push_str(&format!("        <SPEC-OBJECT-TYPE IDENTIFIER=\"SOT-REQUIREMENT\" LONG-NAME=\"Requirement\" LAST-CHANGE=\"{TIMESTAMP}\">\n          <SPEC-ATTRIBUTES>\n"));
    let definitions: Vec<(String, String)> = FIELDS.iter().map(|(id, name)| (id.to_string(), name.to_string())).chain(extra_names.iter().map(|n| (format!("AD-X-{}", token(n)), n.clone()))).collect();
    for (id, name) in &definitions {
        out.push_str(&format!(
            "            <ATTRIBUTE-DEFINITION-STRING IDENTIFIER=\"{id}\" LONG-NAME=\"{}\" LAST-CHANGE=\"{TIMESTAMP}\">\n              <TYPE><DATATYPE-DEFINITION-STRING-REF>DT-STRING</DATATYPE-DEFINITION-STRING-REF></TYPE>\n            </ATTRIBUTE-DEFINITION-STRING>\n",
            esc(name)
        ));
    }
    out.push_str("          </SPEC-ATTRIBUTES>\n        </SPEC-OBJECT-TYPE>\n");
    out.push_str(&format!("        <SPECIFICATION-TYPE IDENTIFIER=\"ST-DOCUMENT\" LONG-NAME=\"Specification\" LAST-CHANGE=\"{TIMESTAMP}\"/>\n"));
    for kind in &relation_kinds {
        out.push_str(&format!("        <SPEC-RELATION-TYPE IDENTIFIER=\"SRT-{}\" LONG-NAME=\"{}\" LAST-CHANGE=\"{TIMESTAMP}\"/>\n", token(kind), esc(kind)));
    }
    out.push_str("      </SPEC-TYPES>\n      <SPEC-OBJECTS>\n");
    for entry in &entries {
        let attributes = &entry.requirement.attributes;
        let field = |key: &str| text_of(attributes, key).map(str::to_string);
        let mut values: Vec<(String, String)> = vec![
            ("AD-REQID".to_string(), Some(entry.id.clone())),
            ("AD-TITLE".to_string(), entry.title()),
            ("AD-TEXT".to_string(), field("description")),
            ("AD-PRIORITY".to_string(), field("priority")),
            ("AD-CATEGORY".to_string(), entry.info.and_then(|i| i.category.clone())),
            ("AD-SAFETY".to_string(), entry.info.and_then(|i| i.safety_level.clone())),
        ]
        .into_iter()
        .filter_map(|(k, v)| Some((k, v.filter(|v| !v.is_empty())?)))
        .collect();
        values.extend(entry.extras().into_iter().map(|(k, v)| (format!("AD-X-{}", token(&k)), v)));
        out.push_str(&format!("        <SPEC-OBJECT IDENTIFIER=\"{}\" LAST-CHANGE=\"{TIMESTAMP}\">\n          <VALUES>\n", esc(&entry.identifier)));
        for (definition, value) in values {
            out.push_str(&format!(
                "            <ATTRIBUTE-VALUE-STRING THE-VALUE=\"{}\">\n              <DEFINITION><ATTRIBUTE-DEFINITION-STRING-REF>{definition}</ATTRIBUTE-DEFINITION-STRING-REF></DEFINITION>\n            </ATTRIBUTE-VALUE-STRING>\n",
                esc(&value)
            ));
        }
        out.push_str("          </VALUES>\n          <TYPE><SPEC-OBJECT-TYPE-REF>SOT-REQUIREMENT</SPEC-OBJECT-TYPE-REF></TYPE>\n        </SPEC-OBJECT>\n");
    }
    out.push_str("      </SPEC-OBJECTS>\n      <SPEC-RELATIONS>\n");
    for (identifier, source, target, kind) in &relations {
        out.push_str(&format!(
            "        <SPEC-RELATION IDENTIFIER=\"{}\" LAST-CHANGE=\"{TIMESTAMP}\">\n          <SOURCE><SPEC-OBJECT-REF>{}</SPEC-OBJECT-REF></SOURCE>\n          <TARGET><SPEC-OBJECT-REF>{}</SPEC-OBJECT-REF></TARGET>\n          <TYPE><SPEC-RELATION-TYPE-REF>SRT-{}</SPEC-RELATION-TYPE-REF></TYPE>\n        </SPEC-RELATION>\n",
            esc(identifier),
            esc(source),
            esc(target),
            token(kind)
        ));
    }
    out.push_str("      </SPEC-RELATIONS>\n      <SPECIFICATIONS>\n");
    let mut seen: HashSet<String> = HashSet::new();
    for (index, (name, group)) in groups.iter().enumerate() {
        // Two groups of one name are two specifications: identifiers are unique (xs:ID).
        let key = if seen.insert(name.clone()) { name.clone() } else { format!("{name}#{index}") };
        out.push_str(&format!(
            "        <SPECIFICATION IDENTIFIER=\"{}\" LONG-NAME=\"{}\" LAST-CHANGE=\"{TIMESTAMP}\">\n          <TYPE><SPECIFICATION-TYPE-REF>ST-DOCUMENT</SPECIFICATION-TYPE-REF></TYPE>\n          <CHILDREN>\n",
            xml_id(&element_uuid("reqif-spec", &key)),
            esc(name)
        ));
        let ids: HashSet<&str> = group.iter().map(|e| e.id.as_str()).collect();
        let tops: Vec<&Entry> = group.iter().filter(|e| e.parent().is_none_or(|p| !ids.contains(p) || p == e.id)).collect();
        for top in tops {
            hierarchy(&mut out, top, group, 6, &mut HashSet::new());
        }
        out.push_str("          </CHILDREN>\n        </SPECIFICATION>\n");
    }
    out.push_str("      </SPECIFICATIONS>\n      <SPEC-RELATION-GROUPS/>\n    </REQ-IF-CONTENT>\n  </CORE-CONTENT>\n</REQ-IF>\n");
    (out, notes)
}

/// A SPEC-HIERARCHY entry and, nested, the requirements whose parent it is.
fn hierarchy(out: &mut String, entry: &Entry, group: &[Entry], depth: usize, seen: &mut HashSet<String>) {
    if !seen.insert(entry.id.clone()) {
        return;
    }
    let pad = "  ".repeat(depth);
    out.push_str(&format!("{pad}<SPEC-HIERARCHY IDENTIFIER=\"{}\" LAST-CHANGE=\"{TIMESTAMP}\">\n", xml_id(&element_uuid("reqif-hierarchy", &entry.id))));
    out.push_str(&format!("{pad}  <OBJECT><SPEC-OBJECT-REF>{}</SPEC-OBJECT-REF></OBJECT>\n", esc(&entry.identifier)));
    let children: Vec<&Entry> = group.iter().filter(|c| c.parent() == Some(entry.id.as_str()) && c.id != entry.id).collect();
    if !children.is_empty() {
        out.push_str(&format!("{pad}  <CHILDREN>\n"));
        for child in children {
            hierarchy(out, child, group, depth + 2, seen);
        }
        out.push_str(&format!("{pad}  </CHILDREN>\n"));
    }
    out.push_str(&format!("{pad}</SPEC-HIERARCHY>\n"));
}
