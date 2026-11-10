//! Drift: what the code as built implements against what the model as
//! designed says.
//!
//! The model generates its C. The machines are recovered from that C and
//! from the C as built, by the same reader, and compared: the differences
//! are those of the code, said in the terms of the model (a guard, the
//! target of a transition, an initial value), each with its line in the
//! code. Reading both sides the same way is what keeps a difference of
//! writing (a target named by the state that holds it, a region named by its
//! rank) from being reported as one of behaviour.
//!
//! What a comparison of machines cannot see is then looked for in the code
//! itself: the C as built is compared with the C its own recovered machines
//! generate, and every line they do not explain is reported (the body of an
//! action changed by hand, a helper edited).

use super::{check_showing, from_generated_c, to_arclang, Generated, Machine, Recovered, Transition};
use crate::compiler::ast::Model;
use std::collections::{BTreeMap, BTreeSet};

/// One difference between the code and the model.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Finding {
    /// The machine it is in; empty for one of the whole model.
    pub machine: String,
    pub what: String,
    /// Where in the code as built: the file and the line.
    pub file: Option<String>,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Default, serde::Serialize)]
pub struct Drift {
    /// The code as built is, byte for byte, the code the model generates.
    pub identical: bool,
    /// The differences between the machines of the code and those of the model.
    pub findings: Vec<Finding>,
    /// Lines of the code that its own recovered machines do not explain: (file, line, the line).
    pub unexplained: Vec<(String, usize, String)>,
    /// How many such lines there are in all.
    pub unexplained_lines: usize,
    /// The other files the model generates (the I/O interface, the glue of
    /// each binding, the host implementation, the VHDL), each against the
    /// file as built.
    pub files: Vec<FileDrift>,
}

/// A generated file against the same file as built.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FileDrift {
    pub file: String,
    /// The file is not among the files as built: it was not compared.
    pub missing: bool,
    /// Where the two files part, 1-based; 0 when they are the same.
    pub line: usize,
    /// The lines of the file as built between what the two files share at
    /// their start and at their end, and the lines the model generates there.
    pub built: Vec<String>,
    pub generated: Vec<String>,
}

impl FileDrift {
    pub fn differs(&self) -> bool {
        !self.missing && self.line > 0
    }
}

/// Each generated file (but those in `apart`) against the file `read` gives
/// for its name. The two files are compared whole: what they share at their
/// start and at their end is left out, the rest is the difference.
pub fn files(generated: &[(String, String)], apart: &[&str], read: &dyn Fn(&str) -> Option<String>) -> Vec<FileDrift> {
    generated
        .iter()
        .filter(|(name, _)| !apart.contains(&name.as_str()))
        .map(|(name, text)| {
            let Some(built) = read(name) else {
                return FileDrift { file: name.clone(), missing: true, line: 0, built: Vec::new(), generated: Vec::new() };
            };
            let (mine, theirs): (Vec<&str>, Vec<&str>) = (built.lines().collect(), text.lines().collect());
            let head = mine.iter().zip(&theirs).take_while(|(a, b)| a == b).count();
            let tail = mine[head..].iter().rev().zip(theirs[head..].iter().rev()).take_while(|(a, b)| a == b).count();
            let same = head == mine.len() && head == theirs.len();
            FileDrift {
                file: name.clone(),
                missing: false,
                line: if same { 0 } else { head + 1 },
                built: mine[head..mine.len() - tail].iter().map(|line| line.to_string()).collect(),
                generated: theirs[head..theirs.len() - tail].iter().map(|line| line.to_string()).collect(),
            }
        })
        .collect()
}

impl Drift {
    pub fn none(&self) -> bool {
        (self.identical || (self.findings.is_empty() && self.unexplained_lines == 0)) && !self.files.iter().any(FileDrift::differs)
    }
}

/// Compare the C as built with the model as designed.
pub fn drift(design: &Model, built: &Generated) -> Result<Drift, String> {
    let (files, _) = crate::compiler::embedded::export(design).map_err(|error| format!("the model generates no C to compare with: {error}"))?;
    let file = |name: &str| files.iter().find(|(known, _)| known == name).map(|(_, text)| text.as_str());
    let (Some(header), Some(code)) = (file(built.header_name), file(built.code_name)) else {
        let generated: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).filter(|name| !name.contains('/')).collect();
        return Err(format!(
            "the model generates no {} and {}: this code is that of another model (the model generates {})",
            built.header_name,
            built.code_name,
            generated.join(", ")
        ));
    };
    if header == built.header && code == built.code {
        return Ok(Drift { identical: true, ..Drift::default() });
    }
    let designed = from_generated_c(&Generated { header_name: built.header_name, header, code_name: built.code_name, code })?;
    let as_built = from_generated_c(built)?;
    let mut drift = Drift { findings: compare(&designed, &as_built, built), ..Drift::default() };

    // The step period is a line of the header, set by the bindings of the model.
    let period = regex::Regex::new(r"^#define (\w+_PERIOD_US) (\d+)UL$").expect("a pattern of the comparison");
    for (at, (read, generated)) in built.header.lines().zip(header.lines()).enumerate() {
        if let (Some(read), Some(generated)) = (period.captures(read), period.captures(generated)) {
            if read[1] == generated[1] && read[2] != generated[2] {
                drift.findings.push(Finding {
                    machine: String::new(),
                    what: format!("the step period {} is {} us in the code; the model gives {} us", &read[1], &read[2], &generated[2]),
                    file: Some(built.header_name.to_string()),
                    line: Some(at + 1),
                });
            }
        }
    }

    // What the machines do not say: the code against what its own machines generate.
    let own = check_showing(&to_arclang(&as_built, built), built, usize::MAX);
    if let Some(error) = own.error {
        drift.findings.push(Finding { machine: String::new(), what: format!("the machines read in the code cannot be generated again to look further ({error})"), file: None, line: None });
    }
    // A line already said as a difference of the machines is not said twice.
    let said = |file: &str, line: usize| drift.findings.iter().any(|f| f.file.as_deref() == Some(file) && f.line == Some(line));
    let shown = own.differences.len();
    let unexplained: Vec<(String, usize, String)> = own.differences.into_iter().filter(|d| !said(&d.file, d.line)).map(|d| (d.file, d.line, d.read)).collect();
    drift.unexplained_lines = own.differing_lines - (shown - unexplained.len());
    drift.unexplained = unexplained;
    Ok(drift)
}

fn compare(designed: &Recovered, built: &Recovered, files: &Generated) -> Vec<Finding> {
    let mut findings = Vec::new();
    for machine in &designed.machines {
        if !built.machines.iter().any(|m| m.name == machine.name) {
            findings.push(Finding { machine: machine.name.clone(), what: "the machine is in the model and not in the code".to_string(), file: None, line: None });
        }
    }
    for machine in &built.machines {
        match designed.machines.iter().find(|m| m.name == machine.name) {
            None => findings.push(Finding {
                machine: machine.name.clone(),
                what: "the machine is in the code and not in the model".to_string(),
                file: Some(files.header_name.to_string()),
                line: Some(machine.line),
            }),
            Some(design) => compare_machine(design, machine, files, &mut findings),
        }
    }
    findings
}

fn compare_machine(design: &Machine, built: &Machine, files: &Generated, findings: &mut Vec<Finding>) {
    let mut say = |what: String, file: &str, line: Option<usize>| findings.push(Finding { machine: built.name.clone(), what, file: line.map(|_| file.to_string()), line });
    let quote = |text: &Option<String>| text.as_deref().map_or("none".to_string(), |t| format!("`{t}`"));

    // Variables.
    for variable in &design.variables {
        match built.variables.iter().find(|v| v.name == variable.name) {
            None => say(format!("variable '{}' is in the model and not in the code", variable.name), files.header_name, None),
            Some(found) => {
                let line = Some(found.line);
                if (found.ty, &found.unit, found.input) != (variable.ty, &variable.unit, variable.input) {
                    say(format!("variable '{}' is declared differently in the code", variable.name), files.header_name, line);
                }
                if found.initial != variable.initial {
                    say(format!("variable '{}' starts at {} in the code; the model says {}", variable.name, quote(&found.initial), quote(&variable.initial)), files.code_name, found.initial_line);
                }
                if found.from != variable.from {
                    let wire = |from: &Option<(String, String)>| from.as_ref().map_or("nothing".to_string(), |(machine, name)| format!("\"{machine}\".{name}"));
                    say(format!("input '{}' is read from {} in the code; the model reads it from {}", variable.name, wire(&found.from), wire(&variable.from)), files.code_name, None);
                }
            }
        }
    }
    for variable in built.variables.iter().filter(|v| !design.variables.iter().any(|d| d.name == v.name)) {
        say(format!("variable '{}' is in the code and not in the model", variable.name), files.header_name, Some(variable.line));
    }

    // Equations: what each computes is the statement, whatever its comment says.
    for (rank, (text, (line, statement))) in built.equations.iter().zip(&built.equation_code).enumerate() {
        match design.equations.get(rank).zip(design.equation_code.get(rank)) {
            None => say(format!("equation `{text}` is in the code and not in the model"), files.code_name, Some(*line)),
            Some((designed, (_, generated))) if statement != generated => {
                say(format!("equation `{designed}` is computed in the code as `{statement}`; the model generates `{generated}`"), files.code_name, Some(*line))
            }
            Some((designed, _)) if text != designed => say(format!("the comment of equation `{designed}` says `{text}` in the code: the statement under it is the model's"), files.code_name, Some(*line)),
            _ => {}
        }
    }
    for text in design.equations.iter().skip(built.equations.len()) {
        say(format!("equation `{text}` is in the model and not in the code"), files.code_name, None);
    }

    // States.
    let path = |machine: &Machine, state: usize| machine.states.get(state).map_or("?".to_string(), |s| s.path.clone());
    for state in &design.states {
        match built.states.iter().find(|s| s.path == state.path) {
            None => say(format!("state '{}' is in the model and not in the code", state.path), files.header_name, None),
            Some(found) => {
                for (what, mine, theirs) in [("entry actions", &found.entry, &state.entry), ("exit actions", &found.exit, &state.exit), ("activities", &found.activity, &state.activity)] {
                    if mine != theirs {
                        say(format!("the {what} of state '{}' are [{}] in the code; the model says [{}]", state.path, mine.join(", "), theirs.join(", ")), files.code_name, None);
                    }
                }
                if found.initial != state.initial {
                    say(format!("state '{}' starts in {} in the code; the model says {}", state.path, quote(&found.initial), quote(&state.initial)), files.code_name, None);
                }
                if found.pseudo != state.pseudo {
                    say(format!("state '{}' is {} in the code; the model says {}", state.path, quote(&found.pseudo), quote(&state.pseudo)), files.code_name, None);
                }
            }
        }
    }
    for state in built.states.iter().filter(|s| !design.states.iter().any(|d| d.path == s.path)) {
        say(format!("state '{}' is in the code and not in the model", state.path), files.header_name, None);
    }
    if built.initial.map(|s| path(built, s)) != design.initial.map(|s| path(design, s)) {
        say(
            format!("the machine starts in {} in the code; the model says {}", quote(&built.initial.map(|s| path(built, s))), quote(&design.initial.map(|s| path(design, s)))),
            files.code_name,
            None,
        );
    }
    let regions = |machine: &Machine| -> BTreeSet<(Option<String>, String, String, Option<String>)> {
        machine.regions.iter().map(|r| (r.owner.map(|o| path(machine, o)), path(machine, r.first), path(machine, r.last), r.initial.map(|i| path(machine, i)))).collect()
    };
    if regions(built) != regions(design) {
        say("the regions of the machine (their states, or where they start) are not those of the model".to_string(), files.code_name, None);
    }

    // Transitions, by their ends and their trigger; what each says is then compared.
    type Key = (String, String, Option<String>);
    let keyed = |machine: &Machine| -> BTreeMap<Key, Vec<Transition>> {
        let mut by_key: BTreeMap<Key, Vec<Transition>> = BTreeMap::new();
        for transition in &machine.transitions {
            by_key.entry((path(machine, transition.from), path(machine, transition.to), transition.trigger.clone())).or_default().push(transition.clone());
        }
        by_key
    };
    let (in_code, in_model) = (keyed(built), keyed(design));
    let name = |(from, to, trigger): &Key| match trigger {
        Some(trigger) => format!("transition {from} -> {to} on \"{trigger}\""),
        None => format!("transition {from} -> {to}"),
    };
    let time = |t: &Transition| t.after_us.map(|us| format!("after {us} us")).or(t.budget_us.map(|us| format!("within {us} us")));
    for (key, found) in &in_code {
        let designed = in_model.get(key).map(Vec::as_slice).unwrap_or_default();
        for (rank, transition) in found.iter().enumerate() {
            let line = Some(transition.line);
            let Some(design) = designed.get(rank) else {
                say(format!("{} is in the code and not in the model{}", name(key), transition.guard.as_ref().map_or(String::new(), |g| format!(" (guard `{g}`)"))), files.code_name, line);
                continue;
            };
            if transition.guard != design.guard {
                say(format!("the guard of {} is {} in the code; the model says {}", name(key), quote(&transition.guard), quote(&design.guard)), files.code_name, transition.guard_line.or(line));
            }
            if time(transition) != time(design) {
                say(format!("the time of {} is {} in the code; the model says {}", name(key), quote(&time(transition)), quote(&time(design))), files.code_name, line);
            }
            if transition.actions != design.actions {
                say(format!("the actions of {} are [{}] in the code; the model says [{}]", name(key), transition.actions.join(", "), design.actions.join(", ")), files.code_name, line);
            }
        }
    }
    for (key, designed) in &in_model {
        let found = in_code.get(key).map_or(0, Vec::len);
        for design in designed.iter().skip(found) {
            say(format!("{} is in the model and not in the code{}", name(key), design.guard.as_ref().map_or(String::new(), |g| format!(" (guard `{g}`)"))), files.code_name, None);
        }
    }
}
