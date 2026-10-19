//! ReqIF 1.0 import/export: the requirements-exchange format of DOORS,
//! Polarion, Jama, Codebeamer (OMG ReqIF).
//!
//! Export: one SPECIFICATION per group of requirements (`system_analysis`
//! or `requirements` block), named by it, its SPEC-HIERARCHY nested by
//! `parent:`. Every requirement is a SPEC-OBJECT whose IDENTIFIER is its
//! `reqif_id` when it came from another tool, so that tool sees an update,
//! else its deterministic ArcLang UUID (prefixed `_`, a valid xsd:ID).
//! Requirement-to-requirement traces are SPEC-RELATIONs; traces to other
//! elements, which ReqIF cannot carry, are counted in the notes. Timestamps
//! are fixed: re-exporting an unchanged model is byte-identical.
//!
//! Import: see [`import`].

mod export;
mod import;
pub mod xml;

pub use export::{generate_reqif, generate_reqif_with_notes};
pub use import::import_reqif;

/// An ArcLang string literal.
pub(crate) fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
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
