//! Typed allocations between levels, as SysML v2 has them (`allocation
//! def`): a `type` declaring `from_kind` and `to_kind` is an allocation
//! definition, and an `allocates` trace `is:` it.
//!
//! ```text
//! type "Software deployment" { from_kind: "PhysicalNode" to_kind: "LogicalComponent" }
//! trace "PN-ADAS" allocates "LC-FUSION" { is: "Software deployment" }
//! ```
//!
//! The kinds of the ends are checked: an allocation that does not go from
//! and to what its definition says is an error.

use super::ast::*;
use super::semantic::SemanticModel;

/// The kinds an allocation definition names, folded into a typed trace.
pub const KIND_KEYS: [&str; 2] = ["from_kind", "to_kind"];

/// Check the ends of every typed allocation against its definition.
pub fn check(ast: &Model, semantic: &SemanticModel) -> Result<(), String> {
    let typed: Vec<&Trace> = ast.traces.iter().filter(|t| t.trace_type == "allocates" && KIND_KEYS.iter().any(|k| t.attributes.contains_key(*k))).collect();
    if typed.is_empty() {
        return Ok(());
    }
    // The kinds of the metamodel, as the element graph names them.
    let graph = super::elements::build(ast, semantic);
    let kind_of = |reference: &str| graph.elements.iter().find(|e| e.id == reference || e.name == reference).map(|e| e.kind.to_string());
    for trace in typed {
        let Some(definition) = trace.attributes.get("is").map(|v| v.display()) else { continue };
        for (key, end, side) in [("from_kind", &trace.from, "from"), ("to_kind", &trace.to, "to")] {
            let Some(expected) = trace.attributes.get(key).and_then(|v| v.as_string()) else { continue };
            let Some(actual) = kind_of(end) else { continue };
            if actual != expected {
                return Err(format!(
                    "allocation '{}' -> '{}' is a '{}': it goes {side} a {expected}, not a {actual}",
                    trace.from,
                    trace.to,
                    definition.trim_matches('"')
                ));
            }
        }
    }
    Ok(())
}
