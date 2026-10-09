//! Production-readiness gate.
//!
//! Answers "can this architecture dossier go to production review?" with a
//! deterministic PASS/FAIL verdict. Every check maps to what an industrial
//! design review (PDR/CDR, ISO 26262 or DO-178C flavored) actually requires;
//! the gate refuses to say "ready" when the dossier is not.

use super::ast::{AttributeValue, Model};
use super::quantity::{Quantity, QuantityError};
use super::semantic::SemanticModel;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Severity {
    Blocker,
    Warning,
}

#[derive(Debug, Clone, Serialize)]
pub struct GateFinding {
    pub check: String,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct GateReport {
    pub standard: String,
    pub findings: Vec<GateFinding>,
    pub requirements_total: usize,
    pub requirements_satisfied: usize,
    pub requirements_verified: usize,
    pub constraints_total: usize,
    pub constraints_satisfied: usize,
    pub passed: bool,
}

/// ISO 26262-3 table 4: ASIL from Severity (S1-S3), Exposure (E1-E4),
/// Controllability (C1-C3). Returns "QM" or "ASIL-A".."ASIL-D".
pub fn compute_asil(s: u8, e: u8, c: u8) -> Option<&'static str> {
    if !(1..=3).contains(&s) || !(1..=4).contains(&e) || !(1..=3).contains(&c) {
        return None;
    }
    // Rows: S1..S3 x E1..E4, columns C1..C3.
    const TABLE: [[[&str; 3]; 4]; 3] = [
        // S1
        [["QM", "QM", "QM"], ["QM", "QM", "QM"], ["QM", "QM", "ASIL-A"], ["QM", "ASIL-A", "ASIL-B"]],
        // S2
        [["QM", "QM", "QM"], ["QM", "QM", "ASIL-A"], ["QM", "ASIL-A", "ASIL-B"], ["ASIL-A", "ASIL-B", "ASIL-C"]],
        // S3
        [["QM", "QM", "ASIL-A"], ["QM", "ASIL-A", "ASIL-B"], ["ASIL-A", "ASIL-B", "ASIL-C"], ["ASIL-B", "ASIL-C", "ASIL-D"]],
    ];
    Some(TABLE[(s - 1) as usize][(e - 1) as usize][(c - 1) as usize])
}

/// DO-178C: DAL from failure condition severity.
pub fn dal_for_condition(condition: &str) -> Option<&'static str> {
    match condition.to_lowercase().as_str() {
        "catastrophic" => Some("DAL-A"),
        "hazardous" | "severe-major" => Some("DAL-B"),
        "major" => Some("DAL-C"),
        "minor" => Some("DAL-D"),
        "no_effect" | "no-effect" | "none" => Some("DAL-E"),
        _ => None,
    }
}

/// Parse a duration text like "50 ms", "0.1 s", "100ms" into milliseconds.
/// A bare number is read as milliseconds (legacy convention).
pub fn parse_millis(text: &str) -> Option<f64> {
    match Quantity::parse(text) {
        Ok(quantity) => quantity.as_millis(),
        Err(QuantityError::MissingUnit(value)) => Some(value),
        Err(_) => None,
    }
}

/// Outcome of reading a time-typed attribute through the type system.
#[derive(Debug, PartialEq)]
enum TimeRead {
    /// Not declared.
    Absent,
    /// A proper time quantity, in milliseconds.
    Millis(f64),
    /// A bare number: accepted as milliseconds, but the model should say so.
    UnitlessMillis(f64),
    /// Declared but not a time: the reason, for the finding.
    Invalid(String),
}

/// Read a time attribute (`latency: 25 ms`, legacy `"25 ms"`, legacy `25`).
fn read_time(attributes: &HashMap<String, AttributeValue>, key: &str) -> TimeRead {
    let Some(value) = attributes.get(key) else { return TimeRead::Absent };
    match value.as_quantity() {
        None => TimeRead::Invalid(format!("{} is not a quantity", value.display())),
        Some(Ok(quantity)) => match quantity.as_millis() {
            Some(ms) => TimeRead::Millis(ms),
            None => TimeRead::Invalid(format!(
                "'{}' is a {}, not a time",
                quantity,
                quantity.dimension().label()
            )),
        },
        Some(Err(QuantityError::MissingUnit(number))) => TimeRead::UnitlessMillis(number),
        Some(Err(error)) => TimeRead::Invalid(error.to_string()),
    }
}

fn attr<'a>(attributes: &'a HashMap<String, AttributeValue>, key: &str) -> Option<&'a str> {
    attributes.get(key).and_then(|v| v.as_string())
}

fn level_digit(attributes: &HashMap<String, AttributeValue>, key: &str) -> Option<u8> {
    attr(attributes, key).and_then(|v| {
        v.trim_start_matches(|c: char| c.is_ascii_alphabetic())
            .parse::<u8>()
            .ok()
    })
}

pub fn run_gate(ast: &Model, semantic: &SemanticModel, standard: &str) -> GateReport {
    let metamodel = super::metamodel::Metamodel::current();
    let mut findings = Vec::new();
    let mut push = |check: &str, severity: Severity, message: String| {
        findings.push(GateFinding { check: check.to_string(), severity, message });
    };

    // ---- 1. Requirements: satisfaction and verification coverage ---------
    let satisfied: HashSet<&str> = semantic
        .traces
        .iter()
        .filter(|t| t.trace_type == "satisfies")
        .map(|t| t.to.as_str())
        .collect();
    let verified: HashSet<&str> = ast
        .test_cases
        .iter()
        .flat_map(|tc| tc.verifies.iter().map(|r| r.as_str()))
        .collect();

    let mut requirements_satisfied = 0;
    let mut requirements_verified = 0;
    for requirement in &semantic.requirements {
        if satisfied.contains(requirement.id.as_str()) {
            requirements_satisfied += 1;
        } else {
            push(
                "requirements.satisfaction",
                Severity::Blocker,
                format!("requirement '{}' is satisfied by no architecture element (no satisfies trace)", requirement.id),
            );
        }
        let is_verified = verified.contains(requirement.id.as_str())
            || semantic
                .all_elements
                .get(requirement.id.as_str())
                .map(|e| verified.contains(e.name.as_str()))
                .unwrap_or(false);
        if is_verified {
            requirements_verified += 1;
        } else {
            push(
                "requirements.verification",
                Severity::Blocker,
                format!("requirement '{}' has no verification case (test_case verifies: [...])", requirement.id),
            );
        }
    }
    if semantic.requirements.is_empty() {
        push(
            "requirements.presence",
            Severity::Blocker,
            "the model declares no requirements at all".to_string(),
        );
    }
    // test_case verifies references must resolve to declared requirements
    let requirement_keys: HashSet<&str> = semantic
        .requirements
        .iter()
        .map(|r| r.id.as_str())
        .collect();
    for test_case in &ast.test_cases {
        for reference in &test_case.verifies {
            if !requirement_keys.contains(reference.as_str()) {
                push(
                    "verification.dangling",
                    Severity::Blocker,
                    format!("test_case '{}' verifies unknown requirement '{}'", test_case.name, reference),
                );
            }
        }
    }

    // ---- 2. Safety: HARA presence and ASIL/DAL consistency ---------------
    let declares_safety_level = semantic
        .components
        .iter()
        .any(|c| c.safety_level.is_some() || c.asil.is_some())
        || semantic.requirements.iter().any(|r| r.safety_level.is_some());
    let hazard_count: usize = ast.safety_analysis.iter().map(|s| s.hazards.len()).sum();

    if declares_safety_level && hazard_count == 0 {
        push(
            "safety.hara",
            Severity::Blocker,
            "elements declare ASIL/DAL levels but the model contains no hazard analysis (safety_analysis { hazard ... })".to_string(),
        );
    }
    for safety in &ast.safety_analysis {
        for hazard in &safety.hazards {
            match standard {
                "DO178C" => {
                    let condition = attr(&hazard.attributes, "condition")
                        .or_else(|| attr(&hazard.attributes, "severity"));
                    match condition.and_then(dal_for_condition) {
                        Some(computed) => {
                            if let Some(declared) = attr(&hazard.attributes, "dal") {
                                if metamodel.normalize_enum("Dal", declared) != Some(computed) {
                                    push(
                                        "safety.dal",
                                        Severity::Blocker,
                                        format!("hazard '{}': declared DAL '{}' contradicts computed {} for its failure condition", hazard.name, declared, computed),
                                    );
                                }
                            }
                        }
                        None => push(
                            "safety.dal",
                            Severity::Blocker,
                            format!("hazard '{}': missing or invalid failure condition (catastrophic|hazardous|major|minor|no_effect)", hazard.name),
                        ),
                    }
                }
                _ => {
                    // ISO 26262: S/E/C must be present and consistent
                    let s = level_digit(&hazard.attributes, "severity");
                    let e = level_digit(&hazard.attributes, "exposure");
                    let c = level_digit(&hazard.attributes, "controllability");
                    match (s, e, c) {
                        (Some(s), Some(e), Some(c)) => match compute_asil(s, e, c) {
                            Some(computed) => {
                                if let Some(declared) = attr(&hazard.attributes, "asil") {
                                    if metamodel.normalize_enum("Asil", declared) != Some(computed) {
                                        push(
                                            "safety.asil",
                                            Severity::Blocker,
                                            format!("hazard '{}': declared ASIL '{}' contradicts ISO 26262 table (S{} E{} C{} => {})", hazard.name, declared, s, e, c, computed),
                                        );
                                    }
                                }
                            }
                            None => push(
                                "safety.asil",
                                Severity::Blocker,
                                format!("hazard '{}': S{}/E{}/C{} outside the ISO 26262 ranges (S1-3, E1-4, C1-3)", hazard.name, s, e, c),
                            ),
                        },
                        _ => push(
                            "safety.hara",
                            Severity::Blocker,
                            format!("hazard '{}': missing severity/exposure/controllability (ISO 26262 HARA)", hazard.name),
                        ),
                    }
                }
            }
            // Mitigations must reference declared requirements
            if let Some(AttributeValue::List(items)) = hazard.attributes.get("mitigated_by") {
                for item in items {
                    if let Some(reference) = item.as_string() {
                        if !requirement_keys.contains(reference) {
                            push(
                                "safety.mitigation",
                                Severity::Blocker,
                                format!("hazard '{}': mitigation references unknown requirement '{}'", hazard.name, reference),
                            );
                        }
                    }
                }
            } else {
                push(
                    "safety.mitigation",
                    Severity::Blocker,
                    format!("hazard '{}' has no mitigation (mitigated_by: [requirement ids])", hazard.name),
                );
            }
        }
    }

    // ---- 3. Timing budgets on functional chains --------------------------
    // Latencies are TYPED: a function whose latency is not a time quantity
    // (wrong dimension, free text) is a blocker, a unitless number a warning.
    let mut function_latency: HashMap<&str, f64> = HashMap::new();
    for function in ast.system_analysis.iter().flat_map(|sa| sa.functions.iter()) {
        let read = match read_time(&function.attributes, "latency") {
            TimeRead::Absent => read_time(&function.attributes, "execution_time"),
            other => other,
        };
        match read {
            TimeRead::Absent => {}
            TimeRead::Millis(ms) => {
                function_latency.insert(function.id.as_str(), ms);
            }
            TimeRead::UnitlessMillis(ms) => {
                function_latency.insert(function.id.as_str(), ms);
                push(
                    "timing.units",
                    Severity::Warning,
                    format!("function '{}': latency {} has no unit — assumed milliseconds; write `latency: {} ms`", function.id, ms, ms),
                );
            }
            TimeRead::Invalid(reason) => push(
                "timing.units",
                Severity::Blocker,
                format!("function '{}': latency is not a time quantity ({})", function.id, reason),
            ),
        }
    }
    let function_names: HashMap<&str, &str> = ast
        .system_analysis
        .iter()
        .flat_map(|sa| sa.functions.iter())
        .map(|f| (f.name.as_str(), f.id.as_str()))
        .collect();

    for chain in &semantic.functional_chains {
        let ast_chain = ast
            .system_analysis
            .iter()
            .flat_map(|sa| sa.functional_chains.iter())
            .chain(ast.logical_architecture.iter().flat_map(|la| la.functional_chains.iter()))
            .find(|c| c.id == chain.id);
        let budget = match ast_chain.map(|c| read_time(&c.attributes, "latency_budget")) {
            None | Some(TimeRead::Absent) => None,
            Some(TimeRead::Millis(ms)) => Some(ms),
            Some(TimeRead::UnitlessMillis(ms)) => {
                push(
                    "timing.units",
                    Severity::Warning,
                    format!("functional_chain '{}': latency_budget {} has no unit — assumed milliseconds", chain.name, ms),
                );
                Some(ms)
            }
            Some(TimeRead::Invalid(reason)) => {
                push(
                    "timing.units",
                    Severity::Blocker,
                    format!("functional_chain '{}': latency_budget is not a time quantity ({})", chain.name, reason),
                );
                None
            }
        };
        match budget {
            None => push(
                "timing.budget",
                Severity::Warning,
                format!("functional_chain '{}' declares no latency_budget — end-to-end timing is unbounded", chain.name),
            ),
            Some(budget_ms) => {
                let mut total = 0.0;
                let mut missing = Vec::new();
                for involved in &chain.involves {
                    let key = function_names.get(involved.as_str()).copied().unwrap_or(involved.as_str());
                    match function_latency.get(key) {
                        Some(ms) => total += ms,
                        None => {
                            // Only functions count; exchanges without latency are skipped
                            if semantic
                                .all_elements
                                .get(involved.as_str())
                                .map(|e| e.element_type.contains("Function"))
                                .unwrap_or(false)
                            {
                                missing.push(involved.clone());
                            }
                        }
                    }
                }
                for function in missing {
                    push(
                        "timing.latency",
                        Severity::Blocker,
                        format!("functional_chain '{}': involved function '{}' declares no latency — the {} ms budget cannot be demonstrated", chain.name, function, budget_ms),
                    );
                }
                if total > budget_ms {
                    push(
                        "timing.budget",
                        Severity::Blocker,
                        format!("functional_chain '{}': sum of function latencies {:.1} ms exceeds the {:.1} ms budget", chain.name, total, budget_ms),
                    );
                }
            }
        }
    }

    // ---- 4. ICD completeness on inter-node exchanges ---------------------
    for pa in &ast.physical_architecture {
        for exchange in &pa.physical_exchanges {
            let mut missing = Vec::new();
            if exchange.via.is_none() {
                missing.push("via (carrying link)");
            }
            if exchange.frequency.is_none() {
                missing.push("frequency");
            }
            if exchange.message_type == "Data" {
                missing.push("message_type");
            }
            if !missing.is_empty() {
                push(
                    "icd.completeness",
                    Severity::Blocker,
                    format!("physical_exchange '{}': incomplete ICD — missing {}", exchange.label.as_deref().unwrap_or(""), missing.join(", ")),
                );
            }
        }
        for link in &pa.links {
            if link.protocol == "Unknown" {
                push(
                    "icd.completeness",
                    Severity::Blocker,
                    format!("link '{}' declares no protocol", link.name),
                );
            }
        }
    }

    // ---- 5. Constraints: every declared constraint must hold --------------
    for constraint in &semantic.constraints {
        if !constraint.satisfied {
            push(
                "constraints.violated",
                Severity::Blocker,
                format!(
                    "constraint '{}' is violated: `{}` — left side is {}, right side is {}",
                    constraint.name, constraint.expression, constraint.left, constraint.right
                ),
            );
        }
    }

    // ---- 5b. Typed metamodel: type violations are blockers ----------------
    for diagnostic in super::metamodel_check::check_model(ast, semantic) {
        let severity = match diagnostic.level {
            super::metamodel_check::Level::Error => Severity::Blocker,
            super::metamodel_check::Level::Warning => Severity::Warning,
        };
        push("metamodel.types", severity, diagnostic.to_string());
    }

    // ---- 6. Methodology lints become gate warnings ------------------------
    for lint in super::semantic::arcadia_methodology_lints(ast) {
        push("methodology", Severity::Warning, lint);
    }

    let passed = !findings.iter().any(|f| f.severity == Severity::Blocker);
    GateReport {
        standard: standard.to_string(),
        findings,
        requirements_total: semantic.requirements.len(),
        requirements_satisfied,
        requirements_verified,
        constraints_total: semantic.constraints.len(),
        constraints_satisfied: semantic.constraints.iter().filter(|c| c.satisfied).count(),
        passed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asil_matrix_matches_iso26262_table4() {
        assert_eq!(compute_asil(3, 4, 3), Some("ASIL-D"));
        assert_eq!(compute_asil(3, 4, 2), Some("ASIL-C"));
        assert_eq!(compute_asil(3, 3, 3), Some("ASIL-C"));
        assert_eq!(compute_asil(2, 4, 3), Some("ASIL-C"));
        assert_eq!(compute_asil(1, 4, 3), Some("ASIL-B"));
        assert_eq!(compute_asil(1, 1, 1), Some("QM"));
        assert_eq!(compute_asil(3, 1, 3), Some("ASIL-A"));
        assert_eq!(compute_asil(0, 1, 1), None);
        assert_eq!(compute_asil(3, 5, 1), None);
    }

    #[test]
    fn dal_mapping_matches_do178c() {
        assert_eq!(dal_for_condition("catastrophic"), Some("DAL-A"));
        assert_eq!(dal_for_condition("Hazardous"), Some("DAL-B"));
        assert_eq!(dal_for_condition("major"), Some("DAL-C"));
        assert_eq!(dal_for_condition("minor"), Some("DAL-D"));
        assert_eq!(dal_for_condition("no_effect"), Some("DAL-E"));
        assert_eq!(dal_for_condition("weird"), None);
    }

    #[test]
    fn millis_parser_handles_common_units() {
        assert_eq!(parse_millis("50 ms"), Some(50.0));
        assert_eq!(parse_millis("100ms"), Some(100.0));
        assert_eq!(parse_millis("0.1 s"), Some(100.0));
        assert!((parse_millis("200 us").unwrap() - 0.2).abs() < 1e-12);
        assert_eq!(parse_millis("50"), Some(50.0), "bare numbers are legacy milliseconds");
        assert_eq!(parse_millis("garbage"), None);
    }

    #[test]
    fn typed_time_reads_distinguish_units_dimensions_and_text() {
        use super::super::quantity::Quantity;
        let mut attributes = HashMap::new();
        attributes.insert("typed".to_string(), AttributeValue::Quantity(Quantity::new(0.25, "s").unwrap()));
        attributes.insert("legacy".to_string(), AttributeValue::String("250 ms".to_string()));
        attributes.insert("bare".to_string(), AttributeValue::Number(250.0));
        attributes.insert("wrong".to_string(), AttributeValue::Quantity(Quantity::new(135.0, "MHz").unwrap()));
        attributes.insert("text".to_string(), AttributeValue::String("fast".to_string()));

        assert_eq!(read_time(&attributes, "typed"), TimeRead::Millis(250.0));
        assert_eq!(read_time(&attributes, "legacy"), TimeRead::Millis(250.0));
        assert_eq!(read_time(&attributes, "bare"), TimeRead::UnitlessMillis(250.0));
        assert_eq!(read_time(&attributes, "missing"), TimeRead::Absent);
        assert!(matches!(read_time(&attributes, "wrong"), TimeRead::Invalid(reason) if reason.contains("frequency, not a time")));
        assert!(matches!(read_time(&attributes, "text"), TimeRead::Invalid(_)));
    }

    fn compile(source: &str) -> (Model, SemanticModel) {
        let result = crate::compiler::Compiler::new(crate::compiler::CompilerConfig::default())
            .compile_string(source)
            .expect("compiles");
        (result.ast, result.semantic_model)
    }

    #[test]
    fn gate_sums_typed_latencies_across_units_against_the_budget() {
        let (ast, semantic) = compile(
            r#"
model T {}
system_analysis "SA" {
    requirement "REQ-1" { description: "fast" }
    function "A" { id: "F-A" latency: 0.04 s }
    function "B" { id: "F-B" latency: 50 ms }
    functional_chain "Chain" { id: "FC-1" involves: ["F-A", "F-B"] latency_budget: 100 ms }
}
test_case "TC-1" { verifies: ["REQ-1"] method: "test" }
architecture logical { component "C" { id: "LC-1" } }
trace "LC-1" satisfies "REQ-1"
"#,
        );
        let report = run_gate(&ast, &semantic, "ISO26262");
        assert!(report.passed, "findings: {:?}", report.findings);

        let (ast, semantic) = compile(
            r#"
model T {}
system_analysis "SA" {
    function "A" { id: "F-A" latency: 0.06 s }
    function "B" { id: "F-B" latency: 50 ms }
    functional_chain "Chain" { id: "FC-1" involves: ["F-A", "F-B"] latency_budget: 100 ms }
}
"#,
        );
        let report = run_gate(&ast, &semantic, "ISO26262");
        assert!(report.findings.iter().any(|f| f.check == "timing.budget" && f.message.contains("110.0 ms exceeds the 100.0 ms budget")),
            "findings: {:?}", report.findings);
    }

    #[test]
    fn declared_asil_is_compared_after_normalization_and_type_errors_block() {
        let (ast, semantic) = compile(
            r#"
model T {}
system_analysis "SA" { requirement "REQ-1" { description: "x" } }
safety_analysis {
    hazard "H1" { severity: "S3" exposure: "E4" controllability: "C3" asil: "ASIL_D" mitigated_by: ["REQ-1"] }
    hazard "H2" { severity: "S3" exposure: "E4" controllability: "C3" asil: "ASIL_B" mitigated_by: ["REQ-1"] }
}
architecture logical { component "C" { id: "LC-1" safety_level: "High" } }
"#,
        );
        let report = run_gate(&ast, &semantic, "ISO26262");
        let asil: Vec<_> = report.findings.iter().filter(|f| f.check == "safety.asil").collect();
        assert_eq!(asil.len(), 1, "only H2 contradicts the table: {:?}", asil);
        assert!(asil[0].message.contains("H2"));
        assert!(report.findings.iter().any(|f| f.check == "metamodel.types" && f.severity == Severity::Blocker && f.message.contains("'High' is not a valid SafetyLevel")),
            "{:?}", report.findings);
    }

    #[test]
    fn violated_constraint_blocks_the_gate() {
        let source = |limit: &str| {
            format!(
                r#"
model T {{}}
system_analysis "SA" {{
    function "A" {{ id: "F-A" latency: 60 ms }}
    function "B" {{ id: "F-B" latency: 30 ms }}
    functional_chain "Chain" {{ id: "FC-1" involves: ["F-A", "F-B"] latency_budget: 100 ms }}
}}
constraint "Margin" {{ assert: sum("FC-1", latency) <= "FC-1".latency_budget * {} }}
"#,
                limit
            )
        };
        let (ast, semantic) = compile(&source("0.8"));
        let report = run_gate(&ast, &semantic, "ISO26262");
        assert!(
            report.findings.iter().any(|f| f.check == "constraints.violated" && f.severity == Severity::Blocker && f.message.contains("left side is 90 ms, right side is 80 ms")),
            "{:?}",
            report.findings
        );
        let (ast, semantic) = compile(&source("0.9"));
        let report = run_gate(&ast, &semantic, "ISO26262");
        assert!(!report.findings.iter().any(|f| f.check == "constraints.violated"), "{:?}", report.findings);
    }

    #[test]
    fn gate_blocks_a_latency_of_the_wrong_dimension_and_warns_on_unitless() {
        let (ast, semantic) = compile(
            r#"
model T {}
system_analysis "SA" {
    function "A" { id: "F-A" latency: 20 MHz }
    function "B" { id: "F-B" latency: 5 }
    functional_chain "Chain" { id: "FC-1" involves: ["F-A", "F-B"] latency_budget: 100 ms }
}
"#,
        );
        let report = run_gate(&ast, &semantic, "ISO26262");
        let units: Vec<_> = report.findings.iter().filter(|f| f.check == "timing.units").collect();
        assert!(units.iter().any(|f| f.severity == Severity::Blocker && f.message.contains("F-A") && f.message.contains("frequency, not a time")), "{:?}", units);
        assert!(units.iter().any(|f| f.severity == Severity::Warning && f.message.contains("F-B") && f.message.contains("assumed milliseconds")), "{:?}", units);
    }
}
