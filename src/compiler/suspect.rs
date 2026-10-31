//! Suspect traces: a trace reviewed against what its two ends said, whose
//! end has changed since.
//!
//! `arclang trace model.arc --review` writes on each trace the fingerprint
//! of its two ends as they are (`reviewed: "3fa9c2d1 7be04a55"`): the
//! fingerprint of an element is its kind, name and attributes. When one end
//! changes (a requirement's text, a component's mass), its fingerprint no
//! longer matches: the trace is suspect, a warning of `check` and a blocker
//! of the production gate, until someone reviews it again. A trace without
//! `reviewed:` is not watched.

use super::ast::{Model, Trace};
use super::elements::{attributes_json, ElementGraph, ElementRecord};
use sha2::{Digest, Sha256};

/// Eight hex digits: the kind, name and attributes of an element.
pub fn fingerprint(element: &ElementRecord) -> String {
    let mut hash = Sha256::new();
    hash.update(element.kind.as_bytes());
    hash.update([0]);
    hash.update(element.name.as_bytes());
    hash.update([0]);
    hash.update(attributes_json(&element.attributes).to_string().as_bytes());
    hash.finalize().iter().take(4).map(|b| format!("{b:02x}")).collect()
}

/// The element a trace end names, by id or by name.
pub fn end<'g>(graph: &'g ElementGraph, written: &str) -> Option<&'g ElementRecord> {
    graph.elements.iter().find(|e| e.id == written).or_else(|| graph.elements.iter().find(|e| e.name == written))
}

/// What a trace's `reviewed:` should say for its ends as they are now.
pub fn stamp(graph: &ElementGraph, trace: &Trace) -> Option<String> {
    Some(format!("{} {}", fingerprint(end(graph, &trace.from)?), fingerprint(end(graph, &trace.to)?)))
}

#[derive(Debug, Clone, PartialEq)]
pub struct Suspect {
    pub from: String,
    pub trace_type: String,
    pub to: String,
    /// The ends that changed since the review, as written.
    pub changed: Vec<String>,
}

impl Suspect {
    /// `suspect: trace 'PN-BATT' satisfies 'REQ-RANGE': 'REQ-RANGE' changed since the trace was reviewed; ...`.
    pub fn line(&self) -> String {
        let changed = match self.changed.as_slice() {
            [one] => format!("'{one}' changed"),
            _ => format!("'{}' and '{}' changed", self.from, self.to),
        };
        format!(
            "suspect: trace '{}' {} '{}': {changed} since the trace was reviewed; check that it still holds, then `arclang trace <model> --review --from {} --to {}`",
            self.from, self.trace_type, self.to, self.from, self.to
        )
    }
}

/// The ends and type of the trace a suspect line is about: `(from, type, to)`.
pub fn ends_of_line(line: &str) -> Option<(String, String, String)> {
    let rest = line.strip_prefix("suspect: trace '")?;
    let (from, rest) = rest.split_once("' ")?;
    let (kind, rest) = rest.split_once(" '")?;
    let (to, _) = rest.split_once("': ")?;
    Some((from.to_string(), kind.to_string(), to.to_string()))
}

/// The reviewed traces whose ends changed, and the traces whose `reviewed:`
/// is not two fingerprints (an error: a hand-edited stamp would hide a change).
pub fn suspects(model: &Model, graph: &ElementGraph) -> Result<Vec<Suspect>, String> {
    let mut out = Vec::new();
    for trace in &model.traces {
        let Some(value) = trace.attributes.get("reviewed") else { continue };
        let written = value.as_string().unwrap_or_default();
        let parts: Vec<&str> = written.split_whitespace().collect();
        let well_formed = parts.len() == 2 && parts.iter().all(|p| p.len() == 8 && p.chars().all(|c| c.is_ascii_hexdigit()));
        if !well_formed {
            return Err(format!(
                "trace '{}' {} '{}': reviewed: \"{written}\" is not the two fingerprints `arclang trace --review` writes",
                trace.from, trace.trace_type, trace.to
            ));
        }
        let (Some(from), Some(to)) = (end(graph, &trace.from), end(graph, &trace.to)) else { continue };
        let mut changed = Vec::new();
        if fingerprint(from) != parts[0] {
            changed.push(trace.from.clone());
        }
        if fingerprint(to) != parts[1] {
            changed.push(trace.to.clone());
        }
        if !changed.is_empty() {
            out.push(Suspect { from: trace.from.clone(), trace_type: trace.trace_type.clone(), to: trace.to.clone(), changed });
        }
    }
    Ok(out)
}

/// Whether a model has a trace to watch at all (the element graph is built only then).
pub fn watches(model: &Model) -> bool {
    model.traces.iter().any(|t| t.attributes.contains_key("reviewed"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_suspect_line_names_its_trace() {
        let suspect = Suspect { from: "PN-BATT".into(), trace_type: "satisfies".into(), to: "REQ-RANGE".into(), changed: vec!["REQ-RANGE".into()] };
        assert_eq!(ends_of_line(&suspect.line()), Some(("PN-BATT".into(), "satisfies".into(), "REQ-RANGE".into())));
        assert_eq!(ends_of_line("constraint: x"), None);
    }
}
