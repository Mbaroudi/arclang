//! The recovered machines as ArcLang source.

use super::{Generated, Machine, Recovered, State, Transition, Variable};

fn quoted(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

pub(super) fn model(recovered: &Recovered, generated: &Generated) -> String {
    let mut out = String::new();
    out.push_str(&format!("// Recovered by `arclang recover` from {} and {}.\n", generated.header_name, generated.code_name));
    out.push_str("// Read in the code: the states and their order, where each machine starts, the\n");
    out.push_str("// ends, the guard and the timing of each transition, the initial values, the\n");
    out.push_str("// wiring. Read in the generator's comments: the names, the units, the text of\n");
    out.push_str("// the equations and of the actions. `arclang recover --check` proves both: it\n");
    out.push_str("// generates the C of this model again and compares it with the C it was read from.\n");
    for note in &recovered.notes {
        out.push_str(&format!("// Note: {note}\n"));
    }
    out.push_str(&format!("\nmodel {} {{\n    version: \"0.0.0\"\n}}\n", quoted(&recovered.model)));
    for machine in &recovered.machines {
        out.push('\n');
        write_machine(&mut out, machine, generated);
    }
    out
}

fn write_machine(out: &mut String, machine: &Machine, generated: &Generated) {
    out.push_str(&format!("// {}:{}\n", generated.header_name, machine.line));
    out.push_str(&format!("state_machine {} {{\n", quoted(&machine.name)));
    if !machine.variables.is_empty() {
        out.push_str("    variables: [\n");
        let lines: Vec<String> = machine.variables.iter().map(|v| format!("        {}", quoted(&variable(v)))).collect();
        out.push_str(&lines.join(",\n"));
        out.push_str("\n    ]\n");
    }
    if !machine.equations.is_empty() {
        out.push_str("    equations: [\n");
        let lines: Vec<String> = machine.equations.iter().map(|e| format!("        {}", quoted(e))).collect();
        out.push_str(&lines.join(",\n"));
        out.push_str("\n    ]\n");
    }
    // A block that only computes: the generator gives it one state of its own, `Flow`.
    let only_computes = machine.transitions.is_empty() && machine.states.len() == 1 && machine.states[0].path == "Flow" && !machine.equations.is_empty();
    if !only_computes {
        if let Some(first) = machine.initial.and_then(|i| machine.states.get(i)) {
            out.push_str(&format!("    initial: {}\n", quoted(&first.path)));
        }
        write_level(out, machine, "", 1);
    }
    out.push_str("}\n");
}

/// The states whose path is `prefix` plus one name, each with those it holds
/// and the transitions that leave them.
fn write_level(out: &mut String, machine: &Machine, prefix: &str, depth: usize) {
    let pad = "    ".repeat(depth);
    let here: Vec<(usize, &State)> = machine.states.iter().enumerate().filter(|(_, s)| s.path.strip_prefix(prefix).is_some_and(|rest| !rest.is_empty() && !rest.contains('.'))).collect();
    for &(_, state) in &here {
        let name = &state.path[prefix.len()..];
        let mut inside = String::new();
        for (key, actions) in [("entry", &state.entry), ("exit", &state.exit), ("do", &state.activity)] {
            match actions.as_slice() {
                [] => {}
                [one] => inside.push_str(&format!("{pad}    {key}: {}\n", quoted(one))),
                several => inside.push_str(&format!("{pad}    {key}: [{}]\n", several.iter().map(|a| quoted(a)).collect::<Vec<_>>().join(", "))),
            }
        }
        if let Some(first) = &state.initial {
            inside.push_str(&format!("{pad}    initial: {}\n", quoted(first)));
        }
        write_level(&mut inside, machine, &format!("{}.", state.path), depth + 1);
        if inside.is_empty() {
            out.push_str(&format!("{pad}state {} {{}}\n", quoted(name)));
        } else {
            out.push_str(&format!("{pad}state {} {{\n{inside}{pad}}}\n", quoted(name)));
        }
    }
    // A transition is written at the level of its source.
    for transition in machine.transitions.iter().filter(|t| here.iter().any(|(index, _)| *index == t.from)) {
        out.push_str(&format!("{pad}{}\n", write_transition(machine, transition, prefix)));
    }
}

/// A state as a transition of the level `prefix` names it: by its name when it
/// is of that level, by its path otherwise.
fn named(machine: &Machine, state: usize, prefix: &str) -> String {
    let path = machine.states.get(state).map(|s| s.path.as_str()).unwrap_or("?");
    match path.strip_prefix(prefix) {
        Some(rest) if !rest.contains('.') => quoted(rest),
        _ => quoted(path.rsplit('.').next().unwrap_or(path)),
    }
}

fn write_transition(machine: &Machine, transition: &Transition, prefix: &str) -> String {
    let mut said: Vec<String> = Vec::new();
    if let Some(trigger) = &transition.trigger {
        said.push(format!("trigger: {}", quoted(trigger)));
    }
    if let Some(guard) = &transition.guard {
        said.push(format!("guard: {}", quoted(guard)));
    }
    match transition.actions.as_slice() {
        [] => {}
        [one] => said.push(format!("action: {}", quoted(one))),
        several => said.push(format!("action: [{}]", several.iter().map(|a| quoted(a)).collect::<Vec<_>>().join(", "))),
    }
    // A timed transition fires after its time; a triggered one has that long to fire.
    if let Some(after) = transition.after_us.or(transition.budget_us) {
        said.push(format!("timing: {}", transition.time_text.clone().unwrap_or_else(|| duration(after))));
    }
    let body = if said.is_empty() { "{}".to_string() } else { format!("{{ {} }}", said.join(" ")) };
    format!("transition {} -> {} {body}", named(machine, transition.from, prefix), named(machine, transition.to, prefix))
}

fn duration(microseconds: u64) -> String {
    if microseconds % 1_000_000 == 0 {
        format!("{} s", microseconds / 1_000_000)
    } else if microseconds % 1_000 == 0 {
        format!("{} ms", microseconds / 1_000)
    } else {
        format!("{microseconds} us")
    }
}

/// `in raw: Real [km/h]`, `filtered: Real [km/h] = 0 km/h`.
fn variable(variable: &Variable) -> String {
    let mut text = String::new();
    if variable.input {
        text.push_str("in ");
    }
    text.push_str(&format!("{}: {}", variable.name, variable.ty));
    if let Some(unit) = &variable.unit {
        text.push_str(&format!(" [{unit}]"));
    }
    if let Some((machine, name)) = &variable.from {
        text.push_str(&format!(" from \"{machine}\".{name}"));
    }
    if let Some(initial) = &variable.initial {
        let value = initial.strip_suffix(".0").unwrap_or(initial);
        match &variable.unit {
            Some(unit) => text.push_str(&format!(" = {value} {unit}")),
            None => text.push_str(&format!(" = {value}")),
        }
    }
    text
}
