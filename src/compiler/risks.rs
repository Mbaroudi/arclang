//! Risks of the project and resources of the system.
//!
//! `risk "Cell supplier delay" { id: likelihood: severity: bears_on: [...] mitigated_by: [...] status: }`
//! is a risk of the project, not a hazard of the product (those are in the
//! safety analysis): how likely it is, how severe, the elements it bears on
//! and what mitigates it. Its level is computed from its likelihood and its
//! severity, never given, and never assumed when one of them is not said.
//!
//! `resource "28 V bus power" { id: capacity: }` is something functions
//! consume and produce (`consumes: [...]`, `produces: [...]` on a function).
//!
//! Both are declared at the top of a file. What a risk names must be
//! declared; a `consumes:` or `produces:` that names no declared resource is
//! kept as the model's own text and said (`metamodel_check`).

use crate::compiler::ast::{AttributeValue, Model, Risk};
use crate::compiler::semantic::{ElementInfo, SemanticModel};

/// How likely a risk is and how severe: the scale of both.
pub const SCALE: [&str; 3] = ["low", "medium", "high"];
/// Where a risk stands.
pub const STATUSES: [&str; 4] = ["open", "mitigated", "accepted", "closed"];

fn text<'r>(risk: &'r Risk, key: &str) -> Option<&'r str> {
    risk.attributes.get(key).and_then(|v| v.as_string())
}

fn rank(risk: &Risk, key: &str) -> Option<usize> {
    let written = text(risk, key)?;
    SCALE.iter().position(|step| step.eq_ignore_ascii_case(written.trim()))
}

/// The level of a risk, from its likelihood and its severity: `low` when
/// both are low or one is low and the other medium, `high` when one is high
/// and the other at least medium, `medium` between. `None` when one of the
/// two is not said.
pub fn level(risk: &Risk) -> Option<&'static str> {
    let score = (rank(risk, "likelihood")? + 1) * (rank(risk, "severity")? + 1);
    Some(match score {
        1 | 2 => "low",
        3 | 4 => "medium",
        _ => "high",
    })
}

/// Where the risk stands: `open` when it does not say.
pub fn status(risk: &Risk) -> &'static str {
    text(risk, "status").and_then(|written| STATUSES.into_iter().find(|s| s.eq_ignore_ascii_case(written.trim()))).unwrap_or("open")
}

/// The names a list attribute holds: a list, or one alone.
pub fn names<'a>(attributes: &'a std::collections::HashMap<String, AttributeValue>, key: &str) -> Vec<&'a str> {
    match attributes.get(key) {
        Some(AttributeValue::List(items)) => items.iter().filter_map(|v| v.as_string()).collect(),
        Some(AttributeValue::String(one)) => vec![one.as_str()],
        _ => Vec::new(),
    }
}

/// The resource a reference names, by id or by name: its id.
pub fn resource<'m>(ast: &'m Model, reference: &str) -> Option<&'m str> {
    ast.resources.iter().find(|r| r.id == reference).or_else(|| ast.resources.iter().find(|r| r.name == reference)).map(|r| r.id.as_str())
}

/// Risks and resources join the elements of the model. One declared twice,
/// a likelihood, a severity or a status that is no value of its scale, a
/// risk naming an element the model does not declare are errors; a risk
/// that cannot be sized, or a high one left open with nothing mitigating
/// it, is a warning.
pub fn check(ast: &Model, semantic: &mut SemanticModel) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let known = |semantic: &SemanticModel, reference: &str| semantic.all_elements.contains_key(reference) || semantic.all_elements.values().any(|e| e.name == reference);
    for risk in &ast.risks {
        for (key, scale) in [("likelihood", &SCALE[..]), ("severity", &SCALE[..]), ("status", &STATUSES[..])] {
            if let Some(value) = risk.attributes.get(key) {
                if !value.as_string().is_some_and(|written| scale.iter().any(|step| step.eq_ignore_ascii_case(written.trim()))) {
                    errors.push(format!("risk '{}': {key} '{}' is not one of: {}", risk.id, value.display(), scale.join(", ")));
                }
            }
        }
        for key in ["bears_on", "mitigated_by"] {
            for reference in names(&risk.attributes, key) {
                if !known(semantic, reference) {
                    errors.push(format!("risk '{}': {key} '{reference}' names no element — declare it or fix the reference", risk.id));
                }
            }
        }
    }
    let declared = ast.risks.iter().map(|r| ("risk", "Risk", &r.id, &r.name)).chain(ast.resources.iter().map(|r| ("resource", "Resource", &r.id, &r.name)));
    for (word, kind, id, name) in declared {
        match semantic.all_elements.get(id) {
            Some(other) if other.element_type == kind => errors.push(format!("{word} '{id}' is declared more than once")),
            Some(other) => errors.push(format!("{word} '{name}': its id '{id}' is already that of a {}", other.element_type)),
            None => {
                semantic.all_elements.insert(id.clone(), ElementInfo::new(id.clone(), name.clone(), kind));
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    for risk in &ast.risks {
        match level(risk) {
            None => warnings.push(format!("risk '{}' cannot be sized: say its likelihood and its severity ({})", risk.id, SCALE.join(", "))),
            Some("high") if status(risk) == "open" && names(&risk.attributes, "mitigated_by").is_empty() => {
                warnings.push(format!("risk '{}' is high and open, and nothing mitigates it: name what does (mitigated_by: [...]) or say the decision taken (status: \"accepted\")", risk.id));
            }
            Some(_) => {}
        }
    }
    Ok(warnings)
}
