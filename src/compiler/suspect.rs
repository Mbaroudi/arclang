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
//!
//! A test case's `verifies:` is watched the same way: `reviewed:` on the
//! case holds one entry per requirement it verifies, the requirement and the
//! fingerprints of the case and of the requirement
//! (`reviewed: ["REQ-RANGE 1c0de2a7 3fa9c2d1"]`). A changed expectation or
//! a changed requirement makes that verification suspect; a requirement the
//! case verifies but its `reviewed:` does not name is suspect too (never
//! reviewed). The fingerprint of an element leaves out its own `reviewed:`
//! and `verifies:`: adding a verified requirement does not unsettle the
//! verifications already reviewed.

use super::ast::{AttributeValue, Model, TestCase, Trace};
use super::elements::{attributes_json, ElementGraph, ElementRecord};
use sha2::{Digest, Sha256};

/// Eight hex digits: the kind, name and attributes of an element.
pub fn fingerprint(element: &ElementRecord) -> String {
    let mut attributes = element.attributes.clone();
    attributes.remove("reviewed");
    attributes.remove("verifies");
    let mut hash = Sha256::new();
    hash.update(element.kind.as_bytes());
    hash.update([0]);
    hash.update(element.name.as_bytes());
    hash.update([0]);
    hash.update(attributes_json(&attributes).to_string().as_bytes());
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

/// What `reviewed:` on a test case should say for one requirement it
/// verifies: `"REQ-RANGE <case> <requirement>"`.
pub fn verification_stamp(graph: &ElementGraph, case: &TestCase, requirement: &str) -> Option<String> {
    let designator = if case.id.is_empty() { &case.name } else { &case.id };
    Some(format!("{requirement} {} {}", fingerprint(end(graph, designator)?), fingerprint(end(graph, requirement)?)))
}

/// What a suspect is about: a trace, or a test case's `verifies:`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Link {
    Trace,
    Verification,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Suspect {
    pub link: Link,
    pub from: String,
    pub trace_type: String,
    pub to: String,
    /// The ends that changed since the review, as written; empty for a
    /// verification never reviewed.
    pub changed: Vec<String>,
}

impl Suspect {
    /// `suspect: trace 'PN-BATT' satisfies 'REQ-RANGE': 'REQ-RANGE' changed since the trace was reviewed; ...`,
    /// `suspect: verification 'TC-RANGE' verifies 'REQ-RANGE': ...`.
    pub fn line(&self) -> String {
        let (noun, holds) = match self.link {
            Link::Trace => ("trace", "it still holds"),
            Link::Verification => ("verification", "the case still verifies the requirement"),
        };
        let what = match self.changed.as_slice() {
            [] => "never reviewed, while the case reviews its other verifications".to_string(),
            [one] => format!("'{one}' changed since the {noun} was reviewed"),
            _ => format!("'{}' and '{}' changed since the {noun} was reviewed", self.from, self.to),
        };
        format!(
            "suspect: {noun} '{}' {} '{}': {what}; check that {holds}, then `arclang trace <model> --review --from {} --to {}`",
            self.from, self.trace_type, self.to, self.from, self.to
        )
    }
}

/// The ends and type of the trace or verification a suspect line is about:
/// `(from, type, to)`; the type of a verification is `verifies`.
pub fn ends_of_line(line: &str) -> Option<(String, String, String)> {
    let rest = line.strip_prefix("suspect: trace '").or_else(|| line.strip_prefix("suspect: verification '"))?;
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
            out.push(Suspect { link: Link::Trace, from: trace.from.clone(), trace_type: trace.trace_type.clone(), to: trace.to.clone(), changed });
        }
    }
    for case in &model.test_cases {
        out.extend(verification_suspects(case, graph)?);
    }
    Ok(out)
}

/// One entry of a test case's `reviewed:`: `(requirement, case, requirement)`.
fn verification_entries(case: &TestCase) -> Result<Option<Vec<(String, String, String)>>, String> {
    let Some(value) = case.attributes.get("reviewed") else { return Ok(None) };
    let designator = if case.id.is_empty() { &case.name } else { &case.id };
    let malformed = |written: &str| {
        format!("test_case '{designator}': reviewed: entry \"{written}\" is not a requirement and the two fingerprints `arclang trace --review` writes")
    };
    let AttributeValue::List(items) = value else {
        return Err(format!("test_case '{designator}': reviewed: is a list, one entry per verified requirement, as `arclang trace --review` writes it"));
    };
    let mut entries = Vec::new();
    for item in items {
        let written = item.as_string().unwrap_or_default();
        let parts: Vec<&str> = written.split_whitespace().collect();
        let [requirement, case_print, requirement_print] = parts.as_slice() else { return Err(malformed(&written)) };
        if ![case_print, requirement_print].iter().all(|p| p.len() == 8 && p.chars().all(|c| c.is_ascii_hexdigit())) {
            return Err(malformed(&written));
        }
        entries.push((requirement.to_string(), case_print.to_string(), requirement_print.to_string()));
    }
    Ok(Some(entries))
}

/// The entry of `entries` about a verified requirement, however each writes it (id or name).
fn entry_for<'e>(graph: &ElementGraph, entries: &'e [(String, String, String)], requirement: &str) -> Option<&'e (String, String, String)> {
    let same = |a: &str, b: &str| a == b || matches!((end(graph, a), end(graph, b)), (Some(x), Some(y)) if x.uuid == y.uuid);
    entries.iter().find(|(written, _, _)| same(written, requirement))
}

fn verification_suspects(case: &TestCase, graph: &ElementGraph) -> Result<Vec<Suspect>, String> {
    let Some(entries) = verification_entries(case)? else { return Ok(Vec::new()) };
    let designator = if case.id.is_empty() { &case.name } else { &case.id };
    let Some(record) = end(graph, designator) else { return Ok(Vec::new()) };
    let now = fingerprint(record);
    let mut out = Vec::new();
    for requirement in &case.verifies {
        let Some(target) = end(graph, requirement) else { continue };
        let changed = match entry_for(graph, &entries, requirement) {
            None => Vec::new(),
            Some((_, case_print, requirement_print)) => {
                let mut changed = Vec::new();
                if *case_print != now {
                    changed.push(designator.clone());
                }
                if *requirement_print != fingerprint(target) {
                    changed.push(requirement.clone());
                }
                if changed.is_empty() {
                    continue;
                }
                changed
            }
        };
        out.push(Suspect { link: Link::Verification, from: designator.clone(), trace_type: "verifies".into(), to: requirement.clone(), changed });
    }
    Ok(out)
}

/// What a test case's `reviewed:` should say once the verifications of
/// `requirements` are reviewed: fresh entries for those, the others kept as
/// they are, entries for requirements no longer verified dropped. An error
/// when the current `reviewed:` is malformed.
pub fn reviewed_entries(graph: &ElementGraph, case: &TestCase, requirements: &[&str]) -> Result<Vec<String>, String> {
    let entries = verification_entries(case)?.unwrap_or_default();
    let mut out = Vec::new();
    for requirement in &case.verifies {
        if requirements.contains(&requirement.as_str()) {
            if let Some(stamp) = verification_stamp(graph, case, requirement) {
                out.push(stamp);
            }
        } else if let Some((written, case_print, requirement_print)) = entry_for(graph, &entries, requirement) {
            out.push(format!("{written} {case_print} {requirement_print}"));
        }
    }
    Ok(out)
}

/// Whether a model has a trace or a verification to watch at all (the
/// element graph is built only then).
pub fn watches(model: &Model) -> bool {
    model.traces.iter().any(|t| t.attributes.contains_key("reviewed")) || model.test_cases.iter().any(|c| c.attributes.contains_key("reviewed"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_suspect_line_names_its_trace() {
        let suspect = Suspect { link: Link::Trace, from: "PN-BATT".into(), trace_type: "satisfies".into(), to: "REQ-RANGE".into(), changed: vec!["REQ-RANGE".into()] };
        assert_eq!(ends_of_line(&suspect.line()), Some(("PN-BATT".into(), "satisfies".into(), "REQ-RANGE".into())));
        let verification = Suspect { link: Link::Verification, from: "TC-RANGE".into(), trace_type: "verifies".into(), to: "REQ-RANGE".into(), changed: vec![] };
        assert!(verification.line().starts_with("suspect: verification 'TC-RANGE' verifies 'REQ-RANGE': never reviewed"));
        assert_eq!(ends_of_line(&verification.line()), Some(("TC-RANGE".into(), "verifies".into(), "REQ-RANGE".into())));
        assert_eq!(ends_of_line("constraint: x"), None);
    }
}
