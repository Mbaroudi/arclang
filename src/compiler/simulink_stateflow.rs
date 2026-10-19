//! Stateflow skeletons of the state machines, with the documented API:
//! `sfnew` makes the model and its first chart, `add_block('sflib/Chart')`
//! the others, `find(sfroot, ...)` gets each chart, then `Stateflow.State`,
//! `Stateflow.Event` and `Stateflow.Transition` (a default transition into
//! the initial state of the chart, of each region and of each composite
//! state). A trigger becomes a local event of the chart and the label of
//! its transitions; the guard, action and trigger as written are the
//! transition's description, since they are not Stateflow action language.

use super::ast::{Model, State, StateMachine, Transition};
use super::simulink_generator::{m_string, matlab_ident, unique};
use std::collections::{HashMap, HashSet};

struct Chart<'a> {
    index: usize,
    var: String,
    /// State name -> variable, first declaration wins.
    states: HashMap<&'a str, String>,
    vars: HashSet<String>,
    lines: Vec<String>,
}

impl<'a> Chart<'a> {
    /// A state (and the states it holds) inside `parent`, at grid cell `slot`.
    fn state(&mut self, parent: &str, names: &mut HashSet<String>, state: &'a State, slot: usize, origin: (usize, usize)) {
        let name = unique(names, matlab_ident(&state.name), "_");
        let var = unique(&mut self.vars, format!("s_{}_{name}", self.index), "_");
        let size = if state.sub_states.is_empty() { (120, 60) } else { (320, 220) };
        let (x, y) = (origin.0 + (slot % 3) * (size.0 + 60), origin.1 + (slot / 3) * (size.1 + 60));
        self.lines.push(format!(
            "{var} = Stateflow.State({parent}); {var}.Name = {}; {var}.Position = [{x}, {y}, {}, {}];",
            m_string(&name),
            size.0,
            size.1
        ));
        self.states.entry(state.name.as_str()).or_insert_with(|| var.clone());
        let mut inner = HashSet::new();
        for (i, sub) in state.sub_states.iter().enumerate() {
            self.state(&var, &mut inner, sub, i, (x + 20, y + 40));
        }
        if !state.initial.is_empty() {
            self.default_transition(&var, &state.initial);
        }
    }

    fn default_transition(&mut self, parent: &str, initial: &str) {
        let Some(target) = self.states.get(initial).cloned() else {
            self.lines.push(format!("% default transition: initial state {} is not drawn", m_string(initial)));
            return;
        };
        let var = unique(&mut self.vars, format!("t_default_{}", self.index), "_");
        self.lines.push(format!(
            "{var} = Stateflow.Transition({parent}); {var}.Destination = {target}; {var}.DestinationOClock = 0; {var}.SourceEndpoint = {var}.DestinationEndpoint - [0 30]; {var}.Midpoint = {var}.DestinationEndpoint - [0 15];"
        ));
    }

    fn transition(&mut self, number: usize, transition: &Transition, events: &mut HashSet<String>) {
        let (Some(source), Some(target)) = (self.states.get(transition.from.as_str()).cloned(), self.states.get(transition.to.as_str()).cloned()) else {
            self.lines.push(format!("% transition {} -> {}: an end is not a state of the chart", m_string(&transition.from), m_string(&transition.to)));
            return;
        };
        let var = format!("t_{}_{number}", self.index);
        let mut line = format!("{var} = Stateflow.Transition({}); {var}.Source = {source}; {var}.Destination = {target};", self.var);
        if !transition.trigger.trim().is_empty() {
            let event = matlab_ident(&transition.trigger);
            if events.insert(event.clone()) {
                let e = format!("e_{}_{event}", self.index);
                self.lines.push(format!("{e} = Stateflow.Event({}); {e}.Name = {}; {e}.Scope = 'Local';", self.var, m_string(&event)));
            }
            line.push_str(&format!(" {var}.LabelString = {};", m_string(&event)));
        }
        let mut description = transition.trigger.clone();
        if let Some(guard) = transition.guard.as_ref().filter(|g| !g.is_empty()) {
            description.push_str(&format!(" [{guard}]"));
        }
        if let Some(action) = transition.action.as_ref().filter(|a| !a.is_empty()) {
            description.push_str(&format!(" / {action}"));
        }
        let description = description.trim();
        if !description.is_empty() {
            line.push_str(&format!(" {var}.Description = {};", m_string(description)));
        }
        self.lines.push(line);
    }
}

fn chart(index: usize, machine: &StateMachine, charts: &mut HashSet<String>) -> Vec<String> {
    let name = unique(charts, matlab_ident(&machine.name), "_");
    let mut lines = Vec::new();
    lines.push(format!("% {}", machine.name.replace(['\n', '\r'], " ")));
    if index == 0 {
        lines.push(format!("set_param([sfModelName '/Chart'], 'Name', {});", m_string(&name)));
    } else {
        lines.push(format!("add_block('sflib/Chart', [sfModelName '/' {}]);", m_string(&name)));
    }
    let var = format!("chart_{index}");
    lines.push(format!("{var} = find(sfroot, '-isa', 'Stateflow.Chart', 'Path', [sfModelName '/' {}]);", m_string(&name)));
    let mut chart = Chart { index, var: var.clone(), states: HashMap::new(), vars: HashSet::new(), lines };
    if machine.regions.is_empty() {
        let mut names = HashSet::new();
        for (slot, state) in machine.states.iter().enumerate() {
            chart.state(&var, &mut names, state, slot, (40, 40));
        }
        chart.default_transition(&var, &machine.initial_state);
    } else {
        // Regions run together: parallel states of the chart.
        chart.lines.push(format!("{var}.Decomposition = 'PARALLEL_AND';"));
        let mut regions = HashSet::new();
        for (r, region) in machine.regions.iter().enumerate() {
            let region_name = unique(&mut regions, matlab_ident(&region.name), "_");
            let region_var = unique(&mut chart.vars, format!("s_{index}_{region_name}"), "_");
            chart.lines.push(format!(
                "{region_var} = Stateflow.State({var}); {region_var}.Name = {}; {region_var}.Position = [{}, 40, 560, 400];",
                m_string(&region_name),
                40 + r * 600
            ));
            let mut names = HashSet::new();
            for (slot, state) in region.states.iter().enumerate() {
                chart.state(&region_var, &mut names, state, slot, (60 + r * 600, 80));
            }
            chart.default_transition(&region_var, &region.initial);
        }
    }
    let mut events = HashSet::new();
    for (number, transition) in machine.transitions.iter().enumerate() {
        chart.transition(number, transition, &mut events);
    }
    chart.lines
}

/// The charts of every state machine, in one model `<model>_behaviour`, saved.
pub(super) fn write(out: &mut String, ast: &Model, model_name: &str) {
    if ast.state_machines.is_empty() {
        return;
    }
    out.push_str("%% Stateflow charts (behaviour skeletons)\n");
    out.push_str(&format!("sfModelName = {};\n", m_string(&format!("{model_name}_behaviour"))));
    out.push_str("sfnew(sfModelName);\n\n");
    let mut charts = HashSet::new();
    for (index, machine) in ast.state_machines.iter().enumerate() {
        for line in chart(index, machine, &mut charts) {
            out.push_str(&line);
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str("save_system(sfModelName);\n\n");
}
