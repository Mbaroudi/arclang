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
    // The machines of the logic are those of a node of kind "fpga": the
    // files do not name it, each machine gets one of its own.
    let in_logic: Vec<&Machine> = recovered.machines.iter().filter(|m| m.in_logic).collect();
    if !in_logic.is_empty() {
        out.push_str("\nphysical_architecture \"Recovered hardware\" {\n");
        for machine in in_logic {
            out.push_str(&format!("    node {} {{\n        kind: \"fpga\"\n        behavior_component {} {{}}\n    }}\n", quoted(&format!("Logic of {}", machine.name)), quoted(&component(machine))));
        }
        out.push_str("}\n");
    }
    out
}

/// The component a machine of the logic is the behaviour of.
fn component(machine: &Machine) -> String {
    format!("{} logic", machine.name)
}

fn write_machine(out: &mut String, machine: &Machine, generated: &Generated) {
    out.push_str(&format!("// {}:{}\n", generated.header_name, machine.line));
    out.push_str(&format!("state_machine {} {{\n", quoted(&machine.name)));
    if machine.in_logic {
        out.push_str(&format!("    owner: {}\n", quoted(&component(machine))));
    }
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
        // A machine with regions starts in each of them.
        if let Some(first) = machine.initial.and_then(|i| machine.states.get(i)).filter(|_| !machine.regions.iter().any(|r| r.owner.is_none())) {
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
    let owner = prefix.strip_suffix('.').and_then(|path| machine.states.iter().position(|s| s.path == path));
    let regions: Vec<&super::Region> = machine.regions.iter().filter(|r| r.owner == owner).collect();
    if !regions.is_empty() {
        // The name of a region is not in the code: it is named by its rank.
        for (rank, region) in regions.iter().enumerate() {
            out.push_str(&format!("{pad}region \"Region {}\" {{\n", rank + 1));
            if let Some(first) = region.initial.and_then(|i| machine.states.get(i)) {
                out.push_str(&format!("{pad}    initial: {}\n", quoted(&first.path[prefix.len()..])));
            }
            write_states(out, machine, prefix, depth + 1, Some((region.first, region.last)));
            out.push_str(&format!("{pad}}}\n"));
        }
        // A transition of this level that is in no region: beside them.
        for transition in machine.transitions.iter().filter(|t| t.owner == owner && t.region.is_none()) {
            out.push_str(&format!("{pad}{}\n", write_transition(machine, transition, prefix)));
        }
        return;
    }
    write_states(out, machine, prefix, depth, None);
}

/// The states of a level, all of them or those of one region, with the transitions that leave them.
fn write_states(out: &mut String, machine: &Machine, prefix: &str, depth: usize, range: Option<(usize, usize)>) {
    let pad = "    ".repeat(depth);
    let here: Vec<(usize, &State)> = machine
        .states
        .iter()
        .enumerate()
        .filter(|(index, s)| s.path.strip_prefix(prefix).is_some_and(|rest| !rest.is_empty() && !rest.contains('.')) && range.is_none_or(|(first, last)| (first..=last).contains(index)))
        .collect();
    for &(_, state) in &here {
        let name = &state.path[prefix.len()..];
        let mut inside = String::new();
        if let Some(kind) = &state.pseudo {
            inside.push_str(&format!("{pad}    kind: {}\n", quoted(kind)));
        }
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
    // The transitions of this level, or of this region of it.
    let owner = prefix.strip_suffix('.').and_then(|path| machine.states.iter().position(|s| s.path == path));
    let region = range.and_then(|(first, last)| machine.regions.iter().position(|r| r.owner == owner && r.first == first && r.last == last));
    for transition in machine.transitions.iter().filter(|t| t.owner == owner && t.region == region) {
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
    if let Some(id) = &transition.id {
        said.push(format!("id: {}", quoted(id)));
    }
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
    let sized = variable.sized.as_ref();
    if variable.input {
        text.push_str("in ");
    } else if sized.is_some_and(|s| s.origin == "constant") {
        text.push_str("const ");
    }
    text.push_str(&format!("{}: {}", variable.name, variable.ty));
    if let Some(unit) = &variable.unit {
        text.push_str(&format!(" [{unit}]"));
    }
    // The range a word was sized for, or the format it was set to: the logic follows them.
    let unit = variable.unit.as_deref().map(|u| format!(" {u}")).unwrap_or_default();
    match sized {
        Some(s) if s.origin == "declared" => text.push_str(&format!(" range {}{unit} .. {}{unit}", s.range.0, s.range.1)),
        Some(s) if s.origin == "set" => text.push_str(&format!(" format {}", s.format)),
        _ => {}
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
