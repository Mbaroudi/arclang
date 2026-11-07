//! What CESAM asks of a model, checked by the compiler.
//!
//! Two parts:
//! - [`check`], run on every compile: the CESAM attributes the metamodel
//!   check does not cover — a state machine's `kind:` and `vision:`, a
//!   scenario's `vision:` — and the references
//!   between states (`realizes:`, `during:`, `context:`). A value outside
//!   its enumeration or a reference to nothing is an error; a reference to a
//!   state of the wrong role (a configuration realizing a lifecycle phase)
//!   a warning.
//! - [`advisories`], the methodology rules (`cesam.*` in `arclang rules`):
//!   what the CESAM method expects to find — every view of the grid
//!   modelled, stakeholders with needs, needs framed by requirements,
//!   missions with use cases, use cases with scenarios, lifecycle phases
//!   with their operational context, functional modes realized by technical
//!   configurations. Reported by `arclang check --lint --method cesam`, and
//!   by every check of a model declaring `method: cesam`.

use super::ast::{AttributeValue, Model, State, StateMachine};
use super::cesam::{self, Classifier, Placement, Vision, ASPECTS, VISIONS};
use super::elements::ElementGraph;
use super::semantic::SemanticModel;
use std::collections::{HashMap, HashSet};

fn text<'a>(attributes: &'a HashMap<String, AttributeValue>, key: &str) -> Option<&'a str> {
    attributes.get(key).and_then(|v| v.as_string())
}

fn texts(attributes: &HashMap<String, AttributeValue>, key: &str) -> Vec<String> {
    match attributes.get(key) {
        Some(AttributeValue::List(items)) => items.iter().filter_map(|i| i.as_string()).map(str::to_string).collect(),
        Some(AttributeValue::String(one)) => vec![one.clone()],
        _ => Vec::new(),
    }
}

/// Whether the model declares it follows CESAM (`model X { method: cesam }`).
pub fn declared(ast: &Model) -> bool {
    text(&ast.attributes, "method").map(|m| m.eq_ignore_ascii_case("cesam")).unwrap_or(false)
}

/// A state of a machine, with the machine's CESAM vision.
struct StateRef<'a> {
    machine: &'a StateMachine,
    state: &'a State,
    vision: Vision,
}

fn walk<'a>(machine: &'a StateMachine, states: &'a [State], vision: Vision, out: &mut Vec<StateRef<'a>>) {
    for state in states {
        out.push(StateRef { machine, state, vision });
        walk(machine, &state.sub_states, vision, out);
    }
}

fn states(ast: &Model) -> Vec<StateRef<'_>> {
    let mut out = Vec::new();
    for machine in &ast.state_machines {
        walk(machine, &machine.states, cesam::machine_vision(&machine.attributes, &machine.name), &mut out);
    }
    out
}

/// The state a reference names: `Machine.State`, the state's id, or its
/// name when only one state has it.
fn find<'s, 'a>(all: &'s [StateRef<'a>], reference: &str) -> Result<&'s StateRef<'a>, String> {
    if let Some(found) = all.iter().find(|s| format!("{}.{}", s.machine.name, s.state.name) == reference || (!s.state.id.is_empty() && s.state.id == reference)) {
        return Ok(found);
    }
    let named: Vec<&StateRef> = all.iter().filter(|s| s.state.name == reference).collect();
    match named.as_slice() {
        [one] => Ok(one),
        [] => Err(format!("'{reference}' is not a declared state")),
        _ => Err(format!("'{reference}' names states of several machines, write `Machine.{reference}`")),
    }
}

fn role(vision: Vision) -> &'static str {
    match vision {
        Vision::Operational => "a lifecycle phase",
        Vision::Functional => "a functional mode",
        Vision::Constructional => "a technical configuration",
    }
}

fn check_vision(owner: &str, attributes: &HashMap<String, AttributeValue>, errors: &mut Vec<String>) {
    if let Some(value) = text(attributes, "vision") {
        if Vision::parse(value).is_none() {
            errors.push(format!("{owner}: vision '{value}' is not one of operational, functional, constructional"));
        }
    }
}

/// The compile-time checks of the CESAM attributes: `Err` with every error,
/// else the warnings.
pub fn check(ast: &Model, semantic: &SemanticModel) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    for machine in &ast.state_machines {
        let owner = format!("state machine '{}'", machine.name);
        if let Some(kind) = text(&machine.attributes, "kind") {
            if cesam::machine_kind_vision(kind).is_none() {
                errors.push(format!("{owner}: kind '{kind}' is not one of {}", cesam::MACHINE_KINDS.join(", ")));
            }
        }
        check_vision(&owner, &machine.attributes, &mut errors);
    }
    for scenario in &ast.scenarios {
        check_vision(&format!("scenario '{}'", scenario.name), &scenario.attributes, &mut errors);
    }
    let all = states(ast);
    let declared: HashSet<&str> = semantic.all_elements.values().flat_map(|e| [e.id.as_str(), e.name.as_str()]).collect();
    for state in &all {
        let owner = format!("state '{}.{}'", state.machine.name, state.state.name);
        for (key, wanted) in [("realizes", Vision::Functional), ("during", Vision::Operational)] {
            for reference in texts(&state.state.attributes, key) {
                match find(&all, &reference) {
                    Err(why) => errors.push(format!("{owner}: {key} {why}")),
                    Ok(target) if target.vision != wanted => warnings.push(format!(
                        "cesam: {owner}: {key} '{reference}' is {}, expected {}",
                        role(target.vision),
                        role(wanted)
                    )),
                    Ok(_) => {}
                }
            }
        }
        for reference in texts(&state.state.attributes, "context") {
            if !declared.contains(reference.as_str()) {
                errors.push(format!("{owner}: context '{reference}' is not a declared element"));
            }
        }
        if !texts(&state.state.attributes, "context").is_empty() && state.vision != Vision::Operational {
            warnings.push(format!("cesam: {owner}: context is the operational context of a lifecycle phase, and this state is {}", role(state.vision)));
        }
    }
    if errors.is_empty() {
        Ok(warnings)
    } else {
        Err(errors.join("\n"))
    }
}

/// One methodology advisory: the rule it answers, the element concerned.
#[derive(Debug, Clone, PartialEq)]
pub struct Advisory {
    pub rule: &'static str,
    pub element: Option<String>,
    pub message: String,
}

fn advisory(rule: &'static str, element: Option<&str>, message: String) -> Advisory {
    Advisory { rule, element: element.map(str::to_string), message }
}

/// What the CESAM method expects and the model does not have yet.
pub fn advisories(ast: &Model, semantic: &SemanticModel) -> Vec<Advisory> {
    let graph = super::elements::build(ast, semantic);
    let mut out = empty_views(ast, &graph);
    out.extend(stakeholder_analysis(ast));
    out.extend(use_cases(ast));
    out.extend(state_relations(ast));
    out.extend(requirement_visions(ast, &graph));
    out
}

fn empty_views(ast: &Model, graph: &ElementGraph) -> Vec<Advisory> {
    let classifier = Classifier::new(graph, ast);
    let mut filled: HashSet<Placement> = graph.elements.iter().filter_map(|e| classifier.element(e)).collect();
    filled.extend(graph.relationships.iter().filter_map(|r| classifier.relationship(r)));
    let mut out = Vec::new();
    for vision in VISIONS {
        for aspect in ASPECTS {
            if !filled.contains(&Placement { vision, aspect }) {
                let view = cesam::view(vision, aspect);
                out.push(advisory("cesam.view-empty", None, format!("CESAM view '{}' ({} · {}) is empty: model it ({})", view.name, vision.label(), aspect.label(), view.capella)));
            }
        }
    }
    out
}

fn stakeholder_analysis(ast: &Model) -> Vec<Advisory> {
    let mut out = Vec::new();
    let stakeholders: Vec<_> = ast.system_analysis.iter().flat_map(|sa| &sa.stakeholders).collect();
    let concerns: Vec<_> = ast.system_analysis.iter().flat_map(|sa| &sa.concerns).collect();
    let requirements: Vec<_> = ast.system_analysis.iter().flat_map(|sa| &sa.requirements).collect();
    let named_by_concerns: HashSet<String> = concerns.iter().flat_map(|c| texts(&c.attributes, "stakeholders")).collect();
    for stakeholder in &stakeholders {
        let id = text(&stakeholder.attributes, "id").unwrap_or(&stakeholder.id);
        let name = text(&stakeholder.attributes, "name").unwrap_or(&stakeholder.id);
        if !named_by_concerns.contains(id) && !named_by_concerns.contains(name) {
            out.push(advisory("cesam.stakeholder-need", Some(id), format!("stakeholder '{name}' has no need: no concern names it in `stakeholders:`")));
        }
    }
    let framed: HashSet<String> = requirements.iter().flat_map(|r| texts(&r.attributes, "frames")).collect();
    for concern in &concerns {
        let id = text(&concern.attributes, "id").unwrap_or(&concern.id);
        if !stakeholders.is_empty() && texts(&concern.attributes, "stakeholders").is_empty() {
            out.push(advisory("cesam.need-stakeholder", Some(id), format!("concern '{id}' names no stakeholder: whose need is it?")));
        }
        let name = text(&concern.attributes, "name").unwrap_or(&concern.id);
        if !framed.contains(id) && !framed.contains(name) {
            out.push(advisory("cesam.need-requirement", Some(id), format!("concern '{id}' is framed by no requirement: no stakeholder requirement answers this need")));
        }
    }
    out
}

fn use_cases(ast: &Model) -> Vec<Advisory> {
    let mut out = Vec::new();
    let illustrated: HashSet<String> = ast
        .scenarios
        .iter()
        .filter_map(|s| text(&s.attributes, "capability").map(str::to_string))
        .chain(ast.system_analysis.iter().flat_map(|sa| &sa.functional_chains).filter_map(|c| text(&c.attributes, "capability").map(str::to_string)))
        .collect();
    for sa in &ast.system_analysis {
        for mission in &sa.missions {
            let id = text(&mission.attributes, "id").unwrap_or(&mission.name);
            let contributes = sa.capabilities.iter().any(|c| text(&c.attributes, "mission").map(|m| m == id || m == mission.name).unwrap_or(false));
            if !contributes {
                out.push(advisory("cesam.mission-use-case", Some(id), format!("mission '{}' has no use case: no capability names it in `mission:`", mission.name)));
            }
        }
        for capability in &sa.capabilities {
            let id = text(&capability.attributes, "id").unwrap_or(&capability.name);
            if !illustrated.contains(id) && !illustrated.contains(&capability.name) {
                out.push(advisory(
                    "cesam.use-case-scenario",
                    Some(id),
                    format!("use case '{}' is illustrated by no scenario: no scenario or functional chain names it in `capability:`", capability.name),
                ));
            }
        }
    }
    out
}

fn state_relations(ast: &Model) -> Vec<Advisory> {
    let all = states(ast);
    let mut out = Vec::new();
    let has_configurations = all.iter().any(|s| s.vision == Vision::Constructional);
    let realized: HashSet<(String, String)> = all
        .iter()
        .filter(|s| s.vision == Vision::Constructional)
        .flat_map(|s| texts(&s.state.attributes, "realizes"))
        .filter_map(|r| find(&all, &r).ok().map(|t| (t.machine.name.clone(), t.state.name.clone())))
        .collect();
    for state in &all {
        let label = format!("{}.{}", state.machine.name, state.state.name);
        match state.vision {
            Vision::Operational if texts(&state.state.attributes, "context").is_empty() => out.push(advisory(
                "cesam.lifecycle-context",
                Some(&label),
                format!("lifecycle phase '{label}' has no operational context: write `context: [...]`, the actors and stakeholders around the system then"),
            )),
            Vision::Functional if has_configurations && !realized.contains(&(state.machine.name.clone(), state.state.name.clone())) => out.push(advisory(
                "cesam.mode-configuration",
                Some(&label),
                format!("functional mode '{label}' is realized by no technical configuration: write `realizes: [\"{label}\"]` on the configuration that provides it"),
            )),
            _ => {}
        }
    }
    out
}

fn requirement_visions(ast: &Model, graph: &ElementGraph) -> Vec<Advisory> {
    let classifier = Classifier::new(graph, ast);
    graph
        .elements
        .iter()
        .filter(|e| e.kind == "Requirement")
        .filter(|e| !classifier.requirement_vision_known(e))
        .map(|e| {
            advisory(
                "cesam.requirement-vision",
                Some(&e.id),
                format!("requirement '{}' has no CESAM vision: declare it in `requirements stakeholder|functional|technical`, write `vision:`, or trace what satisfies it", e.id),
            )
        })
        .collect()
}

/// The advisories as the lines `arclang check --lint` prints.
pub fn lines(advisories: &[Advisory]) -> Vec<String> {
    advisories.iter().map(|a| format!("[{}] {}", a.rule, a.message)).collect()
}
