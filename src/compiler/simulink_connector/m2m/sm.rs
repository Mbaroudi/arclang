//! State machines: each becomes a Simulink model holding a Stateflow chart,
//! its states states, its pseudo states junctions, its transitions transitions
//! with their triggers as messages fed by ports (`SMInitializeTransformation`,
//! `StateMachineRule`, `RegionRule`, `StateRule`, `FinalStateRule`,
//! `PseudostateRule`, `StateTransitionRule`, the state machines'
//! `FunctionalExchangeRule`). Positions come from the diagrams (`diagram`).

use super::super::java;
use super::super::pivot::Kind;
use super::model::Model;
use super::transposer::{Premise, Rule};
use super::{diagram, function, Key, Run, Target};

/// `SMInitializeTransformation`: a model for each state machine selected.
pub fn initialize(run: &mut Run) -> Result<(), String> {
    let m = run.m;
    for e in run.selection.clone() {
        if !m.is(e, "StateMachine") {
            continue;
        }
        let Some(layer) = run.layer(e) else { return Err(format!("NullPointerException: no engineering perspective for {}", run.label(e))) };
        let system = run.new_obj(Kind::System, m.name(e), m.id(e));
        run.set_layer(system, layer);
        let project = run.project;
        run.contain(project, system, |o| &mut o.systems);
        run.set_main(Key::S(m.id(e).unwrap_or_default()), system);
    }
    Ok(())
}

fn regions(m: &Model, e: usize) -> Vec<usize> {
    m.owned(e, "ownedRegions")
}

pub fn applicable(run: &Run, rule: Rule, e: usize) -> bool {
    let m = run.m;
    match rule {
        Rule::State => !m.is(e, "FinalState"),
        Rule::FinalState => m.is(e, "FinalState"),
        Rule::Pseudostate => !(m.is(e, "EntryPointPseudoState") || m.is(e, "ExitPointPseudoState") || m.is(e, "InitialPseudoState")),
        Rule::Region => m.container(e).is_some_and(|p| (m.is(p, "State") || m.is(p, "StateMachine")) && regions(m, p).len() > 1),
        _ => true,
    }
}

/// `AbstractStateRule.identifyCapellaStateParent`: the region when its owner has several,
/// else the owner (a state machine or a state).
fn state_parent(m: &Model, state: usize) -> Option<usize> {
    let region = m.container(state)?;
    let owner = m.container(region)?;
    if m.is(owner, "StateMachine") || m.is(owner, "State") {
        return Some(if regions(m, owner).len() > 1 { region } else { owner });
    }
    None
}

pub fn premises(run: &Run, rule: Rule, e: usize) -> Vec<Premise> {
    let m = run.m;
    match rule {
        Rule::StateMachine => Vec::new(),
        Rule::State | Rule::FinalState | Rule::Pseudostate => state_parent(m, e).map(|p| vec![(Some(p), true)]).unwrap_or_default(),
        Rule::Region => vec![(m.container(e), true)],
        Rule::StateTransition => {
            let mut out = vec![(m.container(e), true), (m.reference(e, "source"), false), (m.reference(e, "target"), false)];
            out.extend(m.refs(e, "triggers").into_iter().map(|t| (Some(t), false)));
            out
        }
        Rule::ExchangeBus => {
            let mut out: Vec<Premise> = m.refs(e, "exchangedItems").into_iter().map(|ei| (Some(ei), false)).collect();
            let port = |feature: &str, class: &str| m.reference(e, feature).filter(|&p| m.is(p, class));
            out.push((port("source", "FunctionOutputPort").and_then(|p| m.container(p)), false));
            out.push((port("target", "FunctionInputPort").and_then(|p| m.container(p)), false));
            out
        }
        _ => Vec::new(),
    }
}

pub fn create(run: &mut Run, rule: Rule, e: usize) -> Result<Option<Target>, String> {
    let m = run.m;
    let made = match rule {
        Rule::StateMachine => {
            let chart = run.new_obj(Kind::Chart, m.name(e), m.id(e));
            let Some(system) = run.target(&Key::S(m.id(e).unwrap_or_default())).filter(|&s| run.p[s].kind == Kind::System) else {
                return Err(format!("Wrong element type: {}", m.name(e).unwrap_or_default()));
            };
            run.create_default_dictionaries(e, system)?;
            run.contain(system, chart, |o| &mut o.blocks);
            Some(chart)
        }
        Rule::Region => {
            let state = run.new_obj(Kind::State, m.name(e), m.id(e));
            match run.main_opt(m.container(e)) {
                Some(parent) if run.p[parent].kind == Kind::State => run.contain(parent, state, |o| &mut o.sub_states),
                Some(parent) if run.p[parent].kind == Kind::Chart => run.contain(parent, state, |o| &mut o.states),
                _ => {}
            }
            diagram::graphical(run, state, e, false);
            Some(state)
        }
        Rule::State => {
            let state = run.new_obj(Kind::State, m.name(e), m.id(e));
            if regions(m, e).len() > 1 {
                run.p[state].decomposition = "PARALLEL_AND".to_string();
            }
            add_to_parent(run, state, e);
            diagram::graphical(run, state, e, false);
            Some(state)
        }
        Rule::FinalState | Rule::Pseudostate => {
            let junction = run.new_obj(Kind::Junction, m.name(e), m.id(e));
            if m.is(e, "DeepHistoryPseudoState") || m.is(e, "ShallowHistoryPseudoState") {
                run.p[junction].junction = "HISTORY".to_string();
            }
            add_to_parent(run, junction, e);
            diagram::graphical(run, junction, e, false);
            Some(junction)
        }
        Rule::StateTransition => create_transition(run, e)?,
        Rule::ExchangeBus => {
            let bus = function::create_type(run, e)?;
            run.set_main(Key::E(e), bus);
            Some(bus)
        }
        _ => None,
    };
    Ok(made.map(Target::P))
}

/// `StateMachineRule.update`.
pub fn update_chart(run: &mut Run, chart: usize, machine: usize) {
    if run.p[chart].kind == Kind::Chart {
        run.p[chart].originator = run.m.id(machine);
        run.p[chart].name = run.m.name(machine);
    }
}

/// `AbstractStateRule.addToParent`: in the chart or in the state its parent became. (The
/// connector only logs a state whose parent was not transformed; it is then in no chart.)
fn add_to_parent(run: &mut Run, made: usize, state: usize) {
    match run.main_opt(state_parent(run.m, state)) {
        Some(parent) if run.p[parent].kind == Kind::Chart => run.contain(parent, made, |o| &mut o.states),
        Some(parent) if run.p[parent].kind == Kind::State => run.contain(parent, made, |o| &mut o.sub_states),
        _ => {}
    }
}

fn machine_of(m: &Model, e: usize) -> Option<usize> {
    m.container(e).and_then(|c| m.ancestor(c, "StateMachine"))
}

/// `getID`: the key of a machine's symbol for an event.
fn symbol_key(m: &Model, machine: usize, event: usize) -> Key {
    Key::S(format!("{}---{}", m.id(machine).unwrap_or_else(|| "null".to_string()), m.id(event).unwrap_or_else(|| "null".to_string())))
}

fn create_transition(run: &mut Run, e: usize) -> Result<Option<usize>, String> {
    let m = run.m;
    let Some(machine) = machine_of(m, e) else { return Ok(None) };
    let Some(chart) = run.main(machine) else { return Ok(None) };
    let transition = run.new_obj(Kind::Transition, m.name(e), m.id(e));
    run.contain(chart, transition, |o| &mut o.transitions);
    run.p[transition].comment = Some(m.attr(e, "triggerDescription").unwrap_or("").to_string());
    for event in m.refs(e, "triggers") {
        if m.is(event, "TimeEvent") || m.is(event, "ChangeEvent") || run.target(&symbol_key(m, machine, event)).is_some() {
            continue;
        }
        create_ports_and_line(run, machine, chart, event)?;
    }
    run.p[transition].event = Some(triggers(run, machine, e));
    run.p[transition].condition = Some(match m.reference(e, "guard") {
        Some(guard) => default_text(m, guard),
        None => String::new(),
    });
    run.p[transition].state_source = run.main_opt(m.reference(e, "source"));
    run.p[transition].destination = run.main_opt(m.reference(e, "target"));
    let initial = m.reference(e, "source").is_some_and(|s| m.is(s, "InitialPseudoState"));
    diagram::graphical(run, transition, e, !initial);
    Ok(Some(transition))
}

/// `SymbolFactory.createEvent`'s suffix: `_` and the capitals of the class's name.
fn class_initials(class: &str) -> String {
    std::iter::once('_').chain(class.chars().filter(char::is_ascii_uppercase)).collect()
}

/// A message for the event in the chart, fed by a port of the chart, itself fed by a port of
/// the model.
fn create_ports_and_line(run: &mut Run, machine: usize, chart: usize, event: usize) -> Result<(), String> {
    let m = run.m;
    let Some(system) = run.p[chart].container else { return Err("The chart is in no model".to_string()) };
    let datatype = run.main(event).filter(|&d| run.p[d].kind.is_datatype());
    let name = format!("{}{}", java::normalize(m.name(event).as_deref()), class_initials(m.class(event)));
    let symbol = run.new_obj(Kind::Message, Some(name), m.id(event));
    run.p[symbol].scope = "Input".to_string();
    run.contain(chart, symbol, |o| &mut o.symbols);
    run.set_main(symbol_key(m, machine, event), symbol);
    let index = inports(run, chart) + 1;
    let target = run.new_obj(Kind::Port, m.name(event), m.id(event));
    run.p[target].port_kind = "Inport".to_string();
    run.p[target].datatype = datatype;
    run.p[target].index = index;
    run.contain(chart, target, |o| &mut o.ports);
    run.p[symbol].underlying_port = Some(target);
    let index = inports(run, system) + 1;
    let source = run.new_obj(Kind::Port, m.name(event), m.id(event));
    run.p[source].port_kind = "Inport".to_string();
    run.p[source].index = index;
    run.p[source].datatype = datatype;
    run.set_param(source, "BusOutputAsStruct", "On");
    run.contain(system, source, |o| &mut o.ports);
    let line = run.new_obj(Kind::Line, Some(String::new()), None);
    run.p[line].line_source = Some(source);
    run.p[line].line_target = Some(target);
    run.contain(system, line, |o| &mut o.lines);
    Ok(())
}

fn inports(run: &Run, holder: usize) -> i32 {
    run.p[holder].ports.iter().filter(|&&p| run.p[p].port_kind == "Inport").count() as i32
}

/// `getTransitionTrigger`: the messages, and `at(…)`/`after(…)` for the time events, by `|`.
fn triggers(run: &Run, machine: usize, transition: usize) -> String {
    let m = run.m;
    let events = m.refs(transition, "triggers");
    let mut out = String::new();
    for &event in &events {
        if let Some(symbol) = run.target(&symbol_key(m, machine, event)).filter(|&s| run.p[s].kind.is_symbol()) {
            out.push_str(&run.p[symbol].name.clone().unwrap_or_else(|| "null".to_string()));
        }
        if m.is(event, "TimeEvent") {
            let mut trigger = m.name(event).unwrap_or_else(|| "null".to_string());
            if let Some(expression) = m.reference(event, "expression").filter(|&x| m.is(x, "Constraint")) {
                trigger = default_text(m, expression);
            }
            let kind = m.attr(event, "kind").unwrap_or("AT").to_lowercase();
            out.push_str(&format!("{kind}({trigger})"));
        }
        let first = events.iter().position(|&x| x == event).unwrap_or(0);
        if first + 1 < events.len() {
            out.push_str(" | ");
        }
    }
    out
}

/// `CapellaEmbeddedLinkedTextEditorInput.getDefaultText`: the constraint's name, else its
/// first body, its links (`<a href="id"/>`) shown by the name of what they point to.
fn default_text(m: &Model, constraint: usize) -> String {
    if let Some(name) = m.name(constraint).filter(|n| !n.is_empty()) {
        return name;
    }
    let Some(spec) = m.owned(constraint, "ownedSpecification").into_iter().next() else { return String::new() };
    if m.class(spec) != "OpaqueExpression" {
        return String::new();
    }
    let bodies = m.owned(spec, "bodies");
    let languages = m.owned(spec, "languages");
    let (Some(&body), Some(&language)) = (bodies.first(), languages.first()) else { return String::new() };
    let body = m.elements[body].text.clone();
    if m.elements[language].text != "capella:linkedText" {
        return body;
    }
    linked_text(m, &body)
}

fn linked_text(m: &Model, body: &str) -> String {
    let mut out = String::new();
    let mut rest = body;
    while let Some(start) = rest.find("<a href=\"") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 9..];
        let Some(quote) = after.find('"') else { break };
        let id = &after[..quote];
        let tail = &after[quote + 1..];
        let (label, next) = if let Some(t) = tail.strip_prefix("/>") {
            (None, t)
        } else if let Some(close) = tail.find("</a>") {
            let inner = tail.find('>').map(|g| &tail[g + 1..close]).unwrap_or("");
            (Some(inner.to_string()), &tail[close + 4..])
        } else {
            (None, tail)
        };
        let name = m.by_id(id).and_then(|e| m.name(e)).or(label).unwrap_or_default();
        out.push_str(&name);
        rest = next;
    }
    out.push_str(rest);
    out.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&amp;", "&")
}
