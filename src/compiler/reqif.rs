//! ReqIF 1.0 import/export — the contractual requirements-exchange format
//! (OMG ReqIF, used by DOORS, Polarion, Jama, Codebeamer…).
//!
//! Export: every requirement becomes a SPEC-OBJECT whose IDENTIFIER is the
//! requirement's deterministic ArcLang UUID (prefixed `_` to be a valid
//! xsd:ID), with ReqID / Title / Text / Priority / Category / SafetyLevel
//! attributes. Requirement-to-requirement traces become SPEC-RELATIONS.
//! Timestamps are FIXED so re-exporting an unchanged model is byte-identical
//! (same discipline as the Capella bridge: diffs must mean something).
//!
//! Import: reads a ReqIF file produced by any tool, maps attribute
//! definitions by LONG-NAME heuristics (ReqID/ID → id, Text/Description →
//! description, …) and emits an ArcLang `requirements` block. The original
//! SPEC-OBJECT IDENTIFIER is preserved as `reqif_id` so identity survives a
//! round-trip through a foreign tool.

use super::ast::{AttributeValue, Model};
use super::semantic::SemanticModel;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;

/// Fixed timestamp: deterministic output, meaningful diffs.
const REQIF_TIMESTAMP: &str = "2000-01-01T00:00:00Z";

fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// xsd:ID must not start with a digit; ArcLang UUIDs may.
fn xml_id(uuid: &str) -> String {
    format!("_{uuid}")
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

const ATTRS: &[(&str, &str)] = &[
    ("AD-REQID", "ReqID"),
    ("AD-TITLE", "Title"),
    ("AD-TEXT", "Text"),
    ("AD-PRIORITY", "Priority"),
    ("AD-CATEGORY", "Category"),
    ("AD-SAFETY", "SafetyLevel"),
];

pub fn generate_reqif(model: &SemanticModel, ast: &Model) -> String {
    // AST attributes carry the title (and anything else `req` declared).
    let mut ast_attrs: HashMap<&str, &HashMap<String, AttributeValue>> = HashMap::new();
    for sa in &ast.system_analysis {
        for req in &sa.requirements {
            ast_attrs.insert(req.id.as_str(), &req.attributes);
        }
    }

    let title = model.name.as_deref().unwrap_or("ArcLang Requirements");
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<REQ-IF xmlns=\"http://www.omg.org/spec/ReqIF/20110401/reqif.xsd\">\n");
    out.push_str("  <THE-HEADER>\n");
    out.push_str(&format!(
        "    <REQ-IF-HEADER IDENTIFIER=\"{}\">\n",
        xml_id(&super::identity::element_uuid("reqif-header", title))
    ));
    out.push_str(&format!("      <CREATION-TIME>{REQIF_TIMESTAMP}</CREATION-TIME>\n"));
    out.push_str("      <REQ-IF-TOOL-ID>ArcLang</REQ-IF-TOOL-ID>\n");
    out.push_str("      <REQ-IF-VERSION>1.0</REQ-IF-VERSION>\n");
    out.push_str("      <SOURCE-TOOL-ID>ArcLang</SOURCE-TOOL-ID>\n");
    out.push_str(&format!("      <TITLE>{}</TITLE>\n", esc(title)));
    out.push_str("    </REQ-IF-HEADER>\n");
    out.push_str("  </THE-HEADER>\n");
    out.push_str("  <CORE-CONTENT>\n    <REQ-IF-CONTENT>\n");

    // Datatypes
    out.push_str("      <DATATYPES>\n");
    out.push_str(&format!(
        "        <DATATYPE-DEFINITION-STRING IDENTIFIER=\"DT-STRING\" LONG-NAME=\"String\" MAX-LENGTH=\"32000\" LAST-CHANGE=\"{REQIF_TIMESTAMP}\"/>\n"
    ));
    out.push_str("      </DATATYPES>\n");

    // Spec types
    out.push_str("      <SPEC-TYPES>\n");
    out.push_str(&format!(
        "        <SPEC-OBJECT-TYPE IDENTIFIER=\"SOT-REQUIREMENT\" LONG-NAME=\"Requirement\" LAST-CHANGE=\"{REQIF_TIMESTAMP}\">\n"
    ));
    out.push_str("          <SPEC-ATTRIBUTES>\n");
    for (id, long_name) in ATTRS {
        out.push_str(&format!(
            "            <ATTRIBUTE-DEFINITION-STRING IDENTIFIER=\"{id}\" LONG-NAME=\"{long_name}\" LAST-CHANGE=\"{REQIF_TIMESTAMP}\">\n              <TYPE><DATATYPE-DEFINITION-STRING-REF>DT-STRING</DATATYPE-DEFINITION-STRING-REF></TYPE>\n            </ATTRIBUTE-DEFINITION-STRING>\n"
        ));
    }
    out.push_str("          </SPEC-ATTRIBUTES>\n");
    out.push_str("        </SPEC-OBJECT-TYPE>\n");
    out.push_str(&format!(
        "        <SPECIFICATION-TYPE IDENTIFIER=\"ST-DOCUMENT\" LONG-NAME=\"Specification\" LAST-CHANGE=\"{REQIF_TIMESTAMP}\"/>\n"
    ));
    // One relation type per distinct req→req trace kind
    let req_ids: std::collections::HashSet<&str> =
        model.requirements.iter().map(|r| r.id.as_str()).collect();
    let req_traces: Vec<_> = model
        .traces
        .iter()
        .filter(|t| req_ids.contains(t.from.as_str()) && req_ids.contains(t.to.as_str()))
        .collect();
    let mut relation_kinds: Vec<&str> = req_traces.iter().map(|t| t.trace_type.as_str()).collect();
    relation_kinds.sort_unstable();
    relation_kinds.dedup();
    for kind in &relation_kinds {
        out.push_str(&format!(
            "        <SPEC-RELATION-TYPE IDENTIFIER=\"SRT-{}\" LONG-NAME=\"{}\" LAST-CHANGE=\"{REQIF_TIMESTAMP}\"/>\n",
            esc(&kind.to_uppercase()),
            esc(kind)
        ));
    }
    out.push_str("      </SPEC-TYPES>\n");

    // Spec objects
    out.push_str("      <SPEC-OBJECTS>\n");
    for req in &model.requirements {
        let attrs = ast_attrs.get(req.id.as_str());
        let title_attr = attrs
            .and_then(|a| a.get("title"))
            .and_then(|v| v.as_string())
            .unwrap_or("");
        let category = req.category.as_deref().unwrap_or("");
        let safety = req.safety_level.as_deref().unwrap_or("");
        let values: &[(&str, &str)] = &[
            ("AD-REQID", req.id.as_str()),
            ("AD-TITLE", title_attr),
            ("AD-TEXT", req.description.as_str()),
            ("AD-PRIORITY", req.priority.as_str()),
            ("AD-CATEGORY", category),
            ("AD-SAFETY", safety),
        ];
        out.push_str(&format!(
            "        <SPEC-OBJECT IDENTIFIER=\"{}\" LAST-CHANGE=\"{REQIF_TIMESTAMP}\">\n          <VALUES>\n",
            xml_id(&req.uuid())
        ));
        for (def, value) in values {
            if value.is_empty() {
                continue;
            }
            out.push_str(&format!(
                "            <ATTRIBUTE-VALUE-STRING THE-VALUE=\"{}\">\n              <DEFINITION><ATTRIBUTE-DEFINITION-STRING-REF>{def}</ATTRIBUTE-DEFINITION-STRING-REF></DEFINITION>\n            </ATTRIBUTE-VALUE-STRING>\n",
                esc(value)
            ));
        }
        out.push_str("          </VALUES>\n          <TYPE><SPEC-OBJECT-TYPE-REF>SOT-REQUIREMENT</SPEC-OBJECT-TYPE-REF></TYPE>\n        </SPEC-OBJECT>\n");
    }
    out.push_str("      </SPEC-OBJECTS>\n");

    // Relations between requirements (derives/refines/…)
    out.push_str("      <SPEC-RELATIONS>\n");
    let uuid_of = |id: &str| {
        model
            .requirements
            .iter()
            .find(|r| r.id == id)
            .map(|r| xml_id(&r.uuid()))
    };
    for trace in &req_traces {
        if let (Some(source), Some(target)) = (uuid_of(&trace.from), uuid_of(&trace.to)) {
            out.push_str(&format!(
                "        <SPEC-RELATION IDENTIFIER=\"{}\" LAST-CHANGE=\"{REQIF_TIMESTAMP}\">\n          <SOURCE><SPEC-OBJECT-REF>{source}</SPEC-OBJECT-REF></SOURCE>\n          <TARGET><SPEC-OBJECT-REF>{target}</SPEC-OBJECT-REF></TARGET>\n          <TYPE><SPEC-RELATION-TYPE-REF>SRT-{}</SPEC-RELATION-TYPE-REF></TYPE>\n        </SPEC-RELATION>\n",
                xml_id(&trace.uuid()),
                esc(&trace.trace_type.to_uppercase())
            ));
        }
    }
    out.push_str("      </SPEC-RELATIONS>\n");

    // One specification listing every requirement, in model order
    out.push_str("      <SPECIFICATIONS>\n");
    out.push_str(&format!(
        "        <SPECIFICATION IDENTIFIER=\"{}\" LONG-NAME=\"{}\" LAST-CHANGE=\"{REQIF_TIMESTAMP}\">\n          <TYPE><SPECIFICATION-TYPE-REF>ST-DOCUMENT</SPECIFICATION-TYPE-REF></TYPE>\n          <CHILDREN>\n",
        xml_id(&super::identity::element_uuid("reqif-spec", title)),
        esc(title)
    ));
    for req in &model.requirements {
        out.push_str(&format!(
            "            <SPEC-HIERARCHY IDENTIFIER=\"{}\" LAST-CHANGE=\"{REQIF_TIMESTAMP}\">\n              <OBJECT><SPEC-OBJECT-REF>{}</SPEC-OBJECT-REF></OBJECT>\n            </SPEC-HIERARCHY>\n",
            xml_id(&super::identity::element_uuid("reqif-hierarchy", &req.id)),
            xml_id(&req.uuid())
        ));
    }
    out.push_str("          </CHILDREN>\n        </SPECIFICATION>\n");
    out.push_str("      </SPECIFICATIONS>\n");
    out.push_str("      <SPEC-RELATION-GROUPS/>\n");
    out.push_str("    </REQ-IF-CONTENT>\n  </CORE-CONTENT>\n</REQ-IF>\n");
    out
}

// ---------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct ImportedReq {
    identifier: String,
    values: Vec<(String, String)>, // (attribute-definition ref, value)
}

/// Map a foreign attribute LONG-NAME onto an ArcLang requirement field.
fn field_for(long_name: &str) -> &'static str {
    let normalized: String = long_name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    match normalized.as_str() {
        "reqid" | "id" | "identifier" | "foreignid" | "puid" => "id",
        "text" | "description" | "objecttext" | "reqiftext" => "description",
        "title" | "name" | "heading" | "objectheading" | "reqifname" => "title",
        "priority" => "priority",
        "safetylevel" | "asil" | "dal" | "sil" => "safety_level",
        "category" | "type" | "objecttype" | "artifacttype" => "category",
        _ => "",
    }
}

/// Parse a ReqIF file into an ArcLang `requirements` model source.
pub fn import_reqif(xml: &str) -> Result<String, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    // ATTRIBUTE-DEFINITION-* IDENTIFIER -> LONG-NAME
    let mut definitions: HashMap<String, String> = HashMap::new();
    let mut reqs: Vec<ImportedReq> = Vec::new();
    let mut doc_title = String::new();

    let mut current: Option<ImportedReq> = None;
    let mut pending_value: Option<String> = None; // THE-VALUE waiting for its DEFINITION ref
    let mut capture_ref = false; // inside DEFINITION/…-REF
    let mut capture_title = false;
    let mut in_xhtml_value = false;
    let mut xhtml_text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(format!("ReqIF parse error at byte {}: {e}", reader.buffer_position())),
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                let get_attr = |key: &str| -> Option<String> {
                    e.attributes().flatten().find_map(|a| {
                        (String::from_utf8_lossy(a.key.local_name().as_ref()) == key)
                            .then(|| String::from_utf8_lossy(&a.value).to_string())
                    })
                };
                match name.as_str() {
                    // MUST come first: ATTRIBUTE-DEFINITION-STRING-REF also
                    // starts with "ATTRIBUTE-DEFINITION-".
                    n if n.ends_with("-REF") && current.is_some() => {
                        capture_ref = true;
                    }
                    n if n.starts_with("ATTRIBUTE-DEFINITION-") => {
                        if let (Some(id), Some(long)) = (get_attr("IDENTIFIER"), get_attr("LONG-NAME")) {
                            definitions.insert(id, long);
                        }
                    }
                    "SPEC-OBJECT" => {
                        current = Some(ImportedReq {
                            identifier: get_attr("IDENTIFIER").unwrap_or_default(),
                            values: Vec::new(),
                        });
                    }
                    n if n.starts_with("ATTRIBUTE-VALUE-") && current.is_some() => {
                        if n == "ATTRIBUTE-VALUE-XHTML" {
                            in_xhtml_value = true;
                            xhtml_text.clear();
                        } else {
                            // STRING/INTEGER/REAL/BOOLEAN all carry THE-VALUE
                            pending_value = get_attr("THE-VALUE");
                        }
                    }
                    "TITLE" => capture_title = true,
                    _ => {}
                }
            }
            Ok(Event::Text(ref t)) => {
                let text = t.xml10_content().map_err(|e| e.to_string())?.to_string();
                if capture_ref {
                    if let Some(req) = current.as_mut() {
                        let value = if in_xhtml_value {
                            std::mem::take(&mut xhtml_text)
                        } else {
                            pending_value.take().unwrap_or_default()
                        };
                        if !value.is_empty() {
                            req.values.push((text.clone(), value));
                        }
                    }
                    capture_ref = false;
                } else if in_xhtml_value {
                    if !xhtml_text.is_empty() {
                        xhtml_text.push(' ');
                    }
                    xhtml_text.push_str(text.trim());
                } else if capture_title && doc_title.is_empty() {
                    doc_title = text;
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "SPEC-OBJECT" => {
                        if let Some(req) = current.take() {
                            reqs.push(req);
                        }
                    }
                    "ATTRIBUTE-VALUE-XHTML" => in_xhtml_value = false,
                    "TITLE" => capture_title = false,
                    _ => {}
                }
            }
            _ => {}
        }
        buf.clear();
    }

    if reqs.is_empty() {
        return Err("ReqIF import: no SPEC-OBJECT found in the file".to_string());
    }

    // Emit the .arc source
    let model_name: String = if doc_title.is_empty() {
        "ImportedRequirements".to_string()
    } else {
        doc_title
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect()
    };
    let quote = |s: &str| format!("\"{}\"", s.replace('"', "'").replace('\n', " "));

    let mut out = String::new();
    out.push_str("// Imported from ReqIF by ArcLang — reqif_id preserves the foreign identity\n");
    out.push_str(&format!("model {model_name} {{\n}}\n\nrequirements {{\n"));
    for (index, req) in reqs.iter().enumerate() {
        let mut fields: HashMap<&str, String> = HashMap::new();
        for (def_ref, value) in &req.values {
            let long_name = definitions.get(def_ref).map(|s| s.as_str()).unwrap_or(def_ref);
            let field = field_for(long_name);
            if !field.is_empty() {
                fields.entry(field).or_insert_with(|| value.clone());
            }
        }
        let id = fields
            .get("id")
            .cloned()
            .unwrap_or_else(|| format!("REQ-IMPORTED-{:03}", index + 1));
        out.push_str(&format!("  req {}", quote(&id)));
        if let Some(title) = fields.get("title") {
            out.push_str(&format!(" {}", quote(title)));
        }
        out.push_str(" {\n");
        for key in ["description", "priority", "category", "safety_level"] {
            if let Some(value) = fields.get(key) {
                out.push_str(&format!("    {key}: {}\n", quote(value)));
            }
        }
        if !req.identifier.is_empty() {
            out.push_str(&format!("    reqif_id: {}\n", quote(&req.identifier)));
        }
        out.push_str("  }\n");
    }
    out.push_str("}\n");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Compiler, CompilerConfig};

    fn compile(source: &str) -> crate::compiler::CompilationResult {
        Compiler::new(CompilerConfig::default())
            .compile_string(source)
            .expect("compiles")
    }

    #[test]
    fn export_emits_spec_objects_with_deterministic_identifiers() {
        let result = compile(
            r#"
model Demo {
}

requirements safety {
  req "REQ-001" "Range" {
    description: "Detect at 150 m"
    safety_level: "ASIL-D"
    priority: "High"
  }
  req "REQ-002" "Latency" {
    description: "Fuse within 50 ms"
    priority: "High"
  }
}

trace "REQ-002" refines "REQ-001" { rationale: "decomposition" }
"#,
        );
        let reqif = generate_reqif(&result.semantic_model, &result.ast);
        assert!(reqif.contains("SPEC-OBJECT IDENTIFIER=\"_"));
        assert!(reqif.contains("THE-VALUE=\"REQ-001\""));
        assert!(reqif.contains("THE-VALUE=\"Detect at 150 m\""));
        assert!(reqif.contains("THE-VALUE=\"ASIL-D\""));
        assert!(reqif.contains("<SPEC-RELATION IDENTIFIER="));
        // Deterministic: same model → byte-identical export
        assert_eq!(reqif, generate_reqif(&result.semantic_model, &result.ast));
    }

    #[test]
    fn arclang_reqif_round_trip_preserves_requirements() {
        let source = r#"
model RoundTrip {
}

requirements {
  req "REQ-A" "First" {
    description: "The system shall do A"
    priority: "High"
  }
  req "REQ-B" "Second" {
    description: "The system shall do B"
    safety_level: "ASIL-B"
  }
}
"#;
        let result = compile(source);
        let reqif = generate_reqif(&result.semantic_model, &result.ast);
        let arc = import_reqif(&reqif).expect("import succeeds");
        let reimported = compile(&arc);

        assert_eq!(reimported.semantic_model.requirements.len(), 2);
        let req_a = reimported
            .semantic_model
            .requirements
            .iter()
            .find(|r| r.id == "REQ-A")
            .expect("REQ-A survives");
        assert_eq!(req_a.description, "The system shall do A");
        assert_eq!(req_a.priority, "High");
        let req_b = reimported
            .semantic_model
            .requirements
            .iter()
            .find(|r| r.id == "REQ-B")
            .expect("REQ-B survives");
        assert_eq!(req_b.safety_level.as_deref(), Some("ASIL-B"));
        // Identity is stable across the round trip: same id → same UUID
        assert_eq!(
            req_a.uuid(),
            result.semantic_model.requirements[0].uuid()
        );
    }

    #[test]
    fn import_maps_foreign_doors_style_attributes() {
        // Shape a DOORS-ish file: XHTML text, "Object Text"/"Object Heading"
        let foreign = r#"<?xml version="1.0" encoding="UTF-8"?>
<REQ-IF xmlns="http://www.omg.org/spec/ReqIF/20110401/reqif.xsd">
  <THE-HEADER><REQ-IF-HEADER IDENTIFIER="_h"><TITLE>Brake Spec</TITLE></REQ-IF-HEADER></THE-HEADER>
  <CORE-CONTENT><REQ-IF-CONTENT>
    <SPEC-TYPES><SPEC-OBJECT-TYPE IDENTIFIER="SOT-1" LONG-NAME="Requirement">
      <SPEC-ATTRIBUTES>
        <ATTRIBUTE-DEFINITION-STRING IDENTIFIER="AD-1" LONG-NAME="ForeignID"/>
        <ATTRIBUTE-DEFINITION-XHTML IDENTIFIER="AD-2" LONG-NAME="Object Text"/>
        <ATTRIBUTE-DEFINITION-STRING IDENTIFIER="AD-3" LONG-NAME="Object Heading"/>
      </SPEC-ATTRIBUTES>
    </SPEC-OBJECT-TYPE></SPEC-TYPES>
    <SPEC-OBJECTS>
      <SPEC-OBJECT IDENTIFIER="_doors-0001">
        <VALUES>
          <ATTRIBUTE-VALUE-STRING THE-VALUE="SYS-42">
            <DEFINITION><ATTRIBUTE-DEFINITION-STRING-REF>AD-1</ATTRIBUTE-DEFINITION-STRING-REF></DEFINITION>
          </ATTRIBUTE-VALUE-STRING>
          <ATTRIBUTE-VALUE-XHTML>
            <THE-VALUE><div>The brake shall engage within 100 ms</div></THE-VALUE>
            <DEFINITION><ATTRIBUTE-DEFINITION-XHTML-REF>AD-2</ATTRIBUTE-DEFINITION-XHTML-REF></DEFINITION>
          </ATTRIBUTE-VALUE-XHTML>
          <ATTRIBUTE-VALUE-STRING THE-VALUE="Brake engagement">
            <DEFINITION><ATTRIBUTE-DEFINITION-STRING-REF>AD-3</ATTRIBUTE-DEFINITION-STRING-REF></DEFINITION>
          </ATTRIBUTE-VALUE-STRING>
        </VALUES>
        <TYPE><SPEC-OBJECT-TYPE-REF>SOT-1</SPEC-OBJECT-TYPE-REF></TYPE>
      </SPEC-OBJECT>
    </SPEC-OBJECTS>
  </REQ-IF-CONTENT></CORE-CONTENT>
</REQ-IF>"#;
        let arc = import_reqif(foreign).expect("import succeeds");
        let result = compile(&arc);
        let req = &result.semantic_model.requirements[0];
        assert_eq!(req.id, "SYS-42");
        assert_eq!(req.description, "The brake shall engage within 100 ms");
        // Foreign identity preserved for future re-export
        assert!(arc.contains("reqif_id: \"_doors-0001\""));
    }
}
