//! What-if: a proposed change applied to a copy of the model text and
//! compiled, then the model before and the model after compared. Nothing
//! is written; an edit that does not compile is refused with the reason.

use super::graph::millis;
use super::{gate_state, summarize, Analysis, Category, GateState, ImpactReport, ImpactSeverity, Impacted, Options};
use crate::compiler::semantic_diff::{self, DiffReport};
use crate::compiler::workbench::{self, EditRequest};
use crate::compiler::CompilationResult;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ConstraintState {
    pub id: String,
    pub name: String,
    pub expression: String,
    pub before: Option<bool>,
    pub after: Option<bool>,
    pub left_before: String,
    pub left_after: String,
    pub right: String,
    pub flipped: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BudgetState {
    pub chain: String,
    pub name: String,
    pub budget_ms: Option<f64>,
    pub before_ms: Option<f64>,
    pub after_ms: Option<f64>,
    pub ok_before: bool,
    pub ok_after: bool,
    /// Involved functions without a latency, after the change.
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Coverage {
    /// `covered`, `unverified`, `failing`, `unsatisfied` or `absent`.
    pub status: String,
    pub satisfied_by: Vec<String>,
    pub verified_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverageDelta {
    pub requirement: String,
    pub name: String,
    pub before: Coverage,
    pub after: Coverage,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CoverageCounts {
    pub requirements: usize,
    pub satisfied: usize,
    pub verified: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerdictState {
    pub test: String,
    pub name: String,
    pub before: String,
    pub after: String,
    /// A verdict stated by a run elsewhere (`verdict:`, a simulation) that
    /// the change makes out of date: the run is to redo.
    pub stale: bool,
    pub why: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WhatIf {
    pub requests: Vec<EditRequest>,
    pub gate_before: GateState,
    pub gate_after: GateState,
    pub new_blockers: Vec<String>,
    pub removed_blockers: Vec<String>,
    pub constraints: Vec<ConstraintState>,
    pub budgets: Vec<BudgetState>,
    pub coverage_before: CoverageCounts,
    pub coverage_after: CoverageCounts,
    pub coverage: Vec<CoverageDelta>,
    pub verdicts: Vec<VerdictState>,
    pub diff: DiffReport,
}

fn compile(path: Option<&Path>, text: &str) -> Result<CompilationResult, String> {
    match path {
        Some(path) => crate::compiler::source_edit::compile_beside(path, text),
        None => workbench::compile_text(text),
    }
}

/// What an edit names: the elements it touches, as written.
fn designators(request: &EditRequest) -> Vec<String> {
    match request {
        EditRequest::Set { element, .. } | EditRequest::Unset { element, .. } | EditRequest::Rename { element, .. } | EditRequest::Remove { element } | EditRequest::Move { element, .. } | EditRequest::Duplicate { element, .. } => vec![element.clone()],
        EditRequest::Add { container, .. } | EditRequest::AddRaw { container, .. } => container.iter().cloned().collect(),
        EditRequest::WrapFragment { scenario, .. } => vec![scenario.clone()],
        EditRequest::WrapInvariant { scenario, lifeline, name, .. } => vec![scenario.clone(), lifeline.clone(), name.clone()],
        EditRequest::AddTrace { from, to, .. } | EditRequest::RemoveTrace { from, to, .. } => vec![from.clone(), to.clone()],
        EditRequest::Reconnect { from, to, new_from, new_to, .. } => [Some(from), Some(to), new_from.as_ref(), new_to.as_ref()].into_iter().flatten().cloned().collect(),
        EditRequest::DuplicateMany { elements, .. } => elements.clone(),
    }
}

fn describe(request: &EditRequest) -> String {
    let op = serde_json::to_value(request).ok().and_then(|v| v.get("op").and_then(|o| o.as_str()).map(str::to_string)).unwrap_or_default();
    format!("{op} {}", designators(request).join(", "))
}

/// Apply the requests one after the other, each refused unless the model compiles.
pub fn apply(source: &str, requests: &[EditRequest]) -> Result<String, String> {
    let mut text = source.to_string();
    for (index, request) in requests.iter().enumerate() {
        text = workbench::edit(&text, request).map_err(|e| format!("edit {} of {} ({}) refused: {}", index + 1, requests.len(), describe(request), e.trim_start_matches("edit refused: ")))?;
    }
    Ok(text)
}

/// The coverage of every requirement: what satisfies it, what verifies it, the verdicts.
fn coverage(analysis: &Analysis) -> BTreeMap<String, (String, Coverage)> {
    let context = &analysis.context;
    let mut out = BTreeMap::new();
    for element in &context.graph.elements {
        if element.kind != "Requirement" {
            continue;
        }
        let mut satisfied_by = BTreeSet::new();
        let mut verified_by = BTreeSet::new();
        let mut failing = false;
        for r in &context.graph.relationships {
            if r.target != element.uuid {
                continue;
            }
            let Some(source) = context.index_of_uuid(&r.source) else { continue };
            let source = context.element(source);
            match r.kind {
                "Trace" if r.extra.get("traceKind").and_then(|v| v.as_str()) == Some("satisfies") => {
                    satisfied_by.insert(source.id.clone());
                }
                "Verification" => {
                    let verdict = context.verdicts.get(&source.id).map(|(v, _)| v.clone()).unwrap_or_else(|| "pending".to_string());
                    failing |= verdict == "fail";
                    verified_by.insert(format!("{} ({verdict})", source.id));
                }
                _ => {}
            }
        }
        let status = if satisfied_by.is_empty() {
            "unsatisfied"
        } else if verified_by.is_empty() {
            "unverified"
        } else if failing {
            "failing"
        } else {
            "covered"
        };
        out.insert(element.id.clone(), (element.name.clone(), Coverage { status: status.to_string(), satisfied_by: satisfied_by.into_iter().collect(), verified_by: verified_by.into_iter().collect() }));
    }
    out
}

fn counts(coverage: &BTreeMap<String, (String, Coverage)>) -> CoverageCounts {
    CoverageCounts {
        requirements: coverage.len(),
        satisfied: coverage.values().filter(|(_, c)| c.status != "unsatisfied").count(),
        verified: coverage.values().filter(|(_, c)| c.status == "covered").count(),
    }
}

type BudgetRow = (String, Option<f64>, Option<f64>, Vec<String>);

/// The latency of every functional chain with a budget: the sum over what it involves.
fn budgets(analysis: &Analysis) -> BTreeMap<String, BudgetRow> {
    let context = &analysis.context;
    let mut out = BTreeMap::new();
    for element in context.graph.elements.iter().filter(|e| e.kind == "FunctionalChain") {
        let budget = millis(element.attributes.get("latency_budget"));
        let mut total = 0.0;
        let mut any = false;
        let mut missing = Vec::new();
        for r in context.graph.relationships.iter().filter(|r| r.kind == "Involvement" && r.source == element.uuid) {
            let Some(member) = context.index_of_uuid(&r.target).map(|i| context.element(i)) else { continue };
            match millis(member.attributes.get("latency")) {
                Some(ms) => {
                    total += ms;
                    any = true;
                }
                None if member.kind.ends_with("Function") => missing.push(member.id.clone()),
                None => {}
            }
        }
        if budget.is_some() || any {
            out.insert(element.id.clone(), (element.name.clone(), budget, any.then_some(total), missing));
        }
    }
    out
}

fn stated_verdict(test: &crate::compiler::ast::TestCase) -> Option<String> {
    let mut keys: Vec<&String> = test.attributes.keys().filter(|k| k.as_str() == "verdict" || k.starts_with("simulation")).collect();
    keys.sort();
    keys.first().map(|k| test.attributes[*k].display())
}

/// The impact report of a proposed change, or the reason it is refused.
pub fn what_if(path: Option<&Path>, source: &str, requests: Vec<EditRequest>, options: &Options) -> Result<ImpactReport, String> {
    if requests.is_empty() {
        return Err("a what-if needs at least one edit".to_string());
    }
    let before = compile(path, source).map_err(|e| format!("the model does not compile: {e}"))?;
    let edited = apply(source, &requests)?;
    let after = compile(path, &edited).map_err(|e| format!("the change is refused: the edited model does not compile: {e}"))?;
    let diff = semantic_diff::diff_compiled(&before, &after);
    let mut report = compare(&before, &after, diff, requests, options)?;
    report.model = path.map(|p| p.display().to_string()).unwrap_or_else(|| "model".to_string());
    Ok(report)
}

/// The ids a diff touches.
fn touched(diff: &DiffReport) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    ids.extend(diff.added.iter().map(|e| e.id.clone()));
    // A constraint whose computed value moved is a consequence, not a change.
    const COMPUTED: [&str; 3] = ["left", "right", "satisfied"];
    ids.extend(diff.modified.iter().filter(|e| !e.changes.iter().all(|c| COMPUTED.contains(&c.field.as_str()))).map(|e| e.element.id.clone()));
    ids.extend(diff.removed.iter().map(|e| e.id.clone()));
    // A link added or removed with one of its ends is that end's change;
    // a link changed alone changes both its ends.
    let ends_changed: BTreeSet<&str> = diff.added.iter().chain(&diff.removed).flat_map(|e| [e.id.as_str(), e.name.as_str()]).collect();
    let alone = |a: &str, b: &str| !ends_changed.contains(a) && !ends_changed.contains(b) && !ends_changed.contains(a.split('.').next().unwrap_or(a)) && !ends_changed.contains(b.split('.').next().unwrap_or(b));
    for t in diff.traces_added.iter().chain(&diff.traces_removed) {
        if alone(&t.from, &t.to) {
            ids.push(t.from.clone());
            ids.push(t.to.clone());
        }
    }
    for t in &diff.traces_modified {
        ids.push(t.trace.from.clone());
        ids.push(t.trace.to.clone());
    }
    for r in diff.relationships_added.iter().chain(&diff.relationships_removed) {
        if alone(&r.source, &r.target) {
            ids.push(r.source.clone());
            ids.push(r.target.clone());
        }
    }
    for r in &diff.relationships_modified {
        ids.push(r.relationship.source.clone());
        ids.push(r.relationship.target.clone());
    }
    let mut seen = BTreeSet::new();
    ids.retain(|id| !id.is_empty() && seen.insert(id.clone()));
    ids
}

/// The two compiled models compared: the impact of what differs, and the deltas.
pub fn compare(before: &CompilationResult, after: &CompilationResult, diff: DiffReport, requests: Vec<EditRequest>, options: &Options) -> Result<ImpactReport, String> {
    let old = Analysis::new(before, &options.gate_standard);
    let new = Analysis::new(after, &options.gate_standard);

    // A delete with its clean-up (traces, list entries): the deleted element is the change.
    let deleting = requests.iter().any(|r| matches!(r, EditRequest::Remove { .. }));
    let primary = |r: &&EditRequest| !deleting || matches!(r, EditRequest::Remove { .. } | EditRequest::Add { .. } | EditRequest::AddRaw { .. } | EditRequest::Move { .. });
    let mut wanted: Vec<String> = requests.iter().filter(primary).flat_map(designators).collect();
    let removed: BTreeSet<&str> = diff.removed.iter().map(|e| e.id.as_str()).collect();
    wanted.extend(touched(&diff).into_iter().filter(|id| !deleting || removed.contains(id.as_str()) || diff.added.iter().any(|e| &e.id == id)));
    let (mut seeds_new, mut seeds_old) = (Vec::new(), Vec::new());
    let mut seen = BTreeSet::new();
    for reference in &wanted {
        if let Some(node) = new.graph.resolve(&new.context, reference) {
            if seen.insert(new.graph.node(node).uuid.clone()) {
                seeds_new.push(node);
            }
        } else if let Some(node) = old.graph.resolve(&old.context, reference) {
            if seen.insert(old.graph.node(node).uuid.clone()) {
                seeds_old.push(node);
            }
        }
    }
    // What a seed owns is reached from it: the seeds are the outermost changed elements.
    let outermost = |analysis: &Analysis, seeds: &mut Vec<usize>| {
        let elements: Vec<Option<usize>> = seeds.iter().map(|&s| analysis.graph.node(s).element).collect();
        let kept: Vec<usize> = seeds
            .iter()
            .zip(&elements)
            .filter(|(_, element)| element.map_or(true, |e| !analysis.context.owners(e).iter().any(|o| elements.contains(&Some(*o)))))
            .map(|(&s, _)| s)
            .collect();
        *seeds = kept;
    };
    outermost(&new, &mut seeds_new);
    outermost(&old, &mut seeds_old);
    let seed_uuids: BTreeSet<String> = seeds_new.iter().map(|&s| new.graph.node(s).uuid.clone()).chain(seeds_old.iter().map(|&s| old.graph.node(s).uuid.clone())).collect();

    // Reached in the new model; reached from what the change removes, in the old one.
    let mut impacted = new.impacted(&seeds_new, options);
    let mut known: HashMap<String, usize> = impacted.iter().enumerate().map(|(i, item)| (item.uuid.clone(), i)).collect();
    for mut item in old.impacted(&seeds_old, options) {
        if seed_uuids.contains(&item.uuid) {
            continue;
        }
        match known.get(&item.uuid) {
            Some(&at) if impacted[at].distance <= item.distance => {}
            Some(&at) => impacted[at].distance = item.distance,
            None => {
                item.removed = new.graph.by_uuid(&item.uuid).is_none();
                known.insert(item.uuid.clone(), impacted.len());
                impacted.push(item);
            }
        }
    }
    impacted.retain(|item| !seed_uuids.contains(&item.uuid));

    let gate_before = gate_state(&old.context);
    let gate_after = gate_state(&new.context);
    let new_blockers: Vec<String> = gate_after.blockers.iter().filter(|b| !gate_before.blockers.contains(b)).cloned().collect();
    let removed_blockers: Vec<String> = gate_before.blockers.iter().filter(|b| !gate_after.blockers.contains(b)).cloned().collect();

    let constraints = constraints(before, after);
    let (budget_before, budget_after) = (budgets(&old), budgets(&new));
    let mut budget_ids: BTreeSet<&String> = budget_before.keys().collect();
    budget_ids.extend(budget_after.keys());
    let budgets: Vec<BudgetState> = budget_ids
        .into_iter()
        .map(|id| {
            let b = budget_before.get(id);
            let a = budget_after.get(id);
            let budget = a.and_then(|x| x.1).or(b.and_then(|x| x.1));
            let within = |sum: Option<f64>, budget: Option<f64>| match (sum, budget) {
                (Some(s), Some(limit)) => s <= limit + 1e-9,
                _ => true,
            };
            BudgetState {
                chain: id.clone(),
                name: a.or(b).map(|x| x.0.clone()).unwrap_or_default(),
                budget_ms: budget,
                before_ms: b.and_then(|x| x.2),
                after_ms: a.and_then(|x| x.2),
                ok_before: within(b.and_then(|x| x.2), b.and_then(|x| x.1)),
                ok_after: within(a.and_then(|x| x.2), a.and_then(|x| x.1)),
                missing: a.map(|x| x.3.clone()).unwrap_or_default(),
            }
        })
        .collect();

    let (cov_before, cov_after) = (coverage(&old), coverage(&new));
    let absent = || Coverage { status: "absent".to_string(), satisfied_by: Vec::new(), verified_by: Vec::new() };
    let mut requirement_ids: BTreeSet<&String> = cov_before.keys().collect();
    requirement_ids.extend(cov_after.keys());
    let coverage: Vec<CoverageDelta> = requirement_ids
        .into_iter()
        .filter_map(|id| {
            let b = cov_before.get(id).map(|x| x.1.clone()).unwrap_or_else(absent);
            let a = cov_after.get(id).map(|x| x.1.clone()).unwrap_or_else(absent);
            (a != b).then(|| CoverageDelta { requirement: id.clone(), name: cov_after.get(id).or(cov_before.get(id)).map(|x| x.0.clone()).unwrap_or_default(), before: b, after: a })
        })
        .collect();

    let verdicts = verdicts(&old, &new, &impacted, &seed_uuids);

    escalate(&new, &mut impacted, &new_blockers, &coverage, &constraints, &budgets);

    let seeds = seeds_new.iter().map(|&s| new.seed(s)).chain(seeds_old.iter().map(|&s| old.seed(s))).collect();
    let what_if = WhatIf {
        requests,
        coverage_before: counts(&cov_before),
        coverage_after: counts(&cov_after),
        gate_before,
        gate_after: gate_after.clone(),
        new_blockers,
        removed_blockers,
        constraints,
        budgets,
        coverage,
        verdicts,
        diff,
    };
    Ok(ImpactReport {
        model: String::new(),
        seeds,
        depth: options.depth,
        kinds: options.kinds.iter().map(|k| k.as_str()).collect(),
        direction: options.direction,
        summary: summarize(&impacted),
        impacted,
        gate: gate_after,
        what_if: Some(what_if),
        since: None,
        links: new.graph.census(),
    })
}

fn constraints(before: &CompilationResult, after: &CompilationResult) -> Vec<ConstraintState> {
    let old = &before.semantic_model.constraints;
    let new = &after.semantic_model.constraints;
    let key = |c: &crate::compiler::semantic::ConstraintInfo| if c.id.is_empty() { c.name.clone() } else { c.id.clone() };
    let mut ids: Vec<String> = new.iter().map(key).collect();
    ids.extend(old.iter().map(key).filter(|k| !new.iter().any(|c| key(c) == *k)));
    ids.into_iter()
        .map(|id| {
            let b = old.iter().find(|c| key(c) == id);
            let a = new.iter().find(|c| key(c) == id);
            let either = a.or(b).expect("one side");
            ConstraintState {
                id: id.clone(),
                name: either.name.clone(),
                expression: either.expression.clone(),
                before: b.map(|c| c.satisfied),
                after: a.map(|c| c.satisfied),
                left_before: b.map(|c| c.left.clone()).unwrap_or_default(),
                left_after: a.map(|c| c.left.clone()).unwrap_or_default(),
                right: either.right.clone(),
                flipped: b.map(|c| c.satisfied) != a.map(|c| c.satisfied),
            }
        })
        .collect()
}

/// The verdicts that change, and those stated elsewhere that the change makes stale.
fn verdicts(old: &Analysis, new: &Analysis, impacted: &[Impacted], seeds: &BTreeSet<String>) -> Vec<VerdictState> {
    let touched: BTreeSet<&str> = impacted.iter().map(|i| i.uuid.as_str()).chain(seeds.iter().map(String::as_str)).collect();
    let mut out = Vec::new();
    for test in &new.context.result.ast.test_cases {
        let before = old.context.verdicts.get(&test.id).map(|(v, _)| v.clone()).unwrap_or_else(|| "absent".to_string());
        let after = new.context.verdicts.get(&test.id).map(|(v, _)| v.clone()).unwrap_or_else(|| "pending".to_string());
        let node = new.graph.resolve(&new.context, &test.id);
        let uuid = node.map(|n| new.graph.node(n).uuid.clone()).unwrap_or_default();
        let reached = touched.contains(uuid.as_str());
        let stated = stated_verdict(test);
        let stale = reached && stated.is_some();
        if before != after || stale {
            let why = if stale {
                impacted.iter().find(|i| i.uuid == uuid).map(|i| i.why.clone()).unwrap_or_else(|| format!("{} is changed", test.id))
            } else {
                new.context.verdicts.get(&test.id).and_then(|(_, why)| why.clone()).unwrap_or_default()
            };
            out.push(VerdictState { test: test.id.clone(), name: test.name.clone(), before, after, stale, why });
        }
    }
    out
}

/// What the comparison proves raises the severity: a requirement left
/// unsatisfied, an element named by a new gate blocker, a constraint or a
/// budget newly broken.
fn escalate(new: &Analysis, impacted: &mut Vec<Impacted>, new_blockers: &[String], coverage: &[CoverageDelta], constraints: &[ConstraintState], budgets: &[BudgetState]) {
    let raise = |reference: &str, reason: String, impacted: &mut Vec<Impacted>| {
        let node = new.graph.resolve(&new.context, reference);
        let uuid = node.map(|n| new.graph.node(n).uuid.clone());
        if let Some(item) = impacted.iter_mut().find(|i| Some(&i.uuid) == uuid.as_ref() || i.id == reference) {
            item.severity = ImpactSeverity::Blocker;
            item.reason = reason;
            return;
        }
        if let Some(n) = node {
            let n = new.graph.node(n);
            let category = match n.kind.as_str() {
                "Requirement" => Category::Requirement,
                "Constraint" | "FunctionalChain" => Category::Budget,
                "TestCase" => Category::Test,
                _ => Category::Architecture,
            };
            impacted.push(Impacted { id: n.id.clone(), uuid: n.uuid.clone(), name: n.name.clone(), kind: n.kind.clone(), layer: n.layer.clone(), category, severity: ImpactSeverity::Blocker, reason, distance: 1, why: "the change itself".to_string(), path: Vec::new(), integrity: n.integrity.clone(), removed: false });
        }
    };
    for delta in coverage {
        if delta.before.status != "unsatisfied" && delta.before.status != "absent" && delta.after.status == "unsatisfied" {
            raise(&delta.requirement, "newly unsatisfied: nothing satisfies it any more".to_string(), impacted);
        }
        if delta.before.status != "failing" && delta.after.status == "failing" {
            raise(&delta.requirement, "a verification case now fails".to_string(), impacted);
        }
    }
    for constraint in constraints.iter().filter(|c| c.flipped && c.after == Some(false)) {
        raise(&constraint.id, format!("constraint now violated: {} (was {}, {} against {})", constraint.expression, constraint.left_before, constraint.left_after, constraint.right), impacted);
    }
    for budget in budgets.iter().filter(|b| b.ok_before && !b.ok_after) {
        raise(&budget.chain, format!("latency budget exceeded: {:.0} ms for {:.0} ms", budget.after_ms.unwrap_or(0.0), budget.budget_ms.unwrap_or(0.0)), impacted);
    }
    for blocker in new_blockers {
        let named: Vec<String> = impacted.iter().filter(|i| blocker.contains(&format!("'{}'", i.id)) || blocker.contains(&format!("'{}'", i.name))).map(|i| i.id.clone()).collect();
        for id in named {
            if let Some(item) = impacted.iter_mut().find(|i| i.id == id) {
                if item.severity != ImpactSeverity::Blocker || !item.reason.starts_with("new gate blocker") {
                    item.severity = ImpactSeverity::Blocker;
                    item.reason = format!("new gate blocker: {blocker}");
                }
            }
        }
    }
}

/// The edits that delete an element with what refers to it: the traces to
/// and from it (and from what it owns), its mentions in the lists of other
/// elements (`verifies:`, `involves:`, `mitigated_by:`, `shows:`...),
/// then the element. What else still names it makes the delete refused.
pub fn cascade_delete(result: &CompilationResult, element: &str) -> Result<Vec<EditRequest>, String> {
    let context = crate::compiler::tables::Context::new(result, "ISO26262");
    let index = context.resolve(element).ok_or_else(|| format!("no element '{element}' in this model"))?;
    let record = context.element(index);
    let mut doomed: BTreeSet<String> = BTreeSet::new();
    let mut doomed_indices = vec![index];
    for (i, _) in context.graph.elements.iter().enumerate() {
        if context.owners(i).contains(&index) {
            doomed_indices.push(i);
        }
    }
    for &i in &doomed_indices {
        let e = context.element(i);
        doomed.insert(e.id.clone());
        doomed.insert(e.name.clone());
    }
    let names = |text: &str| doomed.contains(text) || doomed.contains(text.split('.').next().unwrap_or(text));
    let mut requests = Vec::new();
    let ast = &result.ast;
    let traces = ast.traces.iter().chain(ast.operational_analysis.iter().flat_map(|oa| oa.traces.iter()));
    for trace in traces {
        if names(&trace.from) || names(&trace.to) {
            requests.push(EditRequest::RemoveTrace { from: trace.from.clone(), kind: trace.trace_type.clone(), to: trace.to.clone() });
        }
    }
    let literal = |items: &[String]| format!("[{}]", items.iter().map(|i| format!("{i:?}")).collect::<Vec<_>>().join(", "));
    for test in &ast.test_cases {
        if test.verifies.iter().any(|v| names(v)) && !doomed.contains(&test.id) {
            let kept: Vec<String> = test.verifies.iter().filter(|v| !names(v)).cloned().collect();
            requests.push(if kept.is_empty() { EditRequest::Unset { element: test.id.clone(), key: "verifies".to_string() } } else { EditRequest::Set { element: test.id.clone(), key: "verifies".to_string(), value: literal(&kept) } });
        }
    }
    const LIST_KEYS: [&str; 10] = ["involves", "mitigated_by", "participants", "selects", "elements", "asserts", "applies_to", "members", "realizes", "allocated_component"];
    for (i, other) in context.graph.elements.iter().enumerate() {
        if doomed_indices.contains(&i) || other.kind == "Model" {
            continue;
        }
        let mut keys: Vec<&String> = other.attributes.keys().filter(|k| LIST_KEYS.contains(&k.as_str())).collect();
        keys.sort();
        for key in keys {
            let value = &other.attributes[key];
            let designator = if other.id.is_empty() { other.name.clone() } else { other.id.clone() };
            match value {
                crate::compiler::ast::AttributeValue::List(items) => {
                    let texts: Vec<String> = items.iter().filter_map(|v| v.as_string()).map(str::to_string).collect();
                    if texts.iter().any(|t| names(t)) {
                        let kept: Vec<String> = texts.into_iter().filter(|t| !names(t)).collect();
                        requests.push(if kept.is_empty() { EditRequest::Unset { element: designator, key: key.clone() } } else { EditRequest::Set { element: designator, key: key.clone(), value: literal(&kept) } });
                    }
                }
                crate::compiler::ast::AttributeValue::String(text) if names(text) => {
                    requests.push(EditRequest::Unset { element: designator, key: key.clone() });
                }
                _ => {}
            }
        }
    }
    for diagram in &ast.diagrams {
        if diagram.shows.iter().any(|s| names(s)) {
            let kept: Vec<String> = diagram.shows.iter().filter(|s| !names(s)).cloned().collect();
            if !kept.is_empty() {
                requests.push(EditRequest::Set { element: diagram.name.clone(), key: "shows".to_string(), value: literal(&kept) });
            }
        }
    }
    requests.push(EditRequest::Remove { element: if record.id.is_empty() { record.name.clone() } else { record.id.clone() } });
    Ok(requests)
}
