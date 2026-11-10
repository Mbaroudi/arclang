//! The proof that code written by hand is a machine of the model.
//!
//! A sketch has its own variables and reads its pins; the model has the
//! structures the compiler generates. A `correspondence` of the model says
//! how they relate:
//!
//! ```arclang
//! correspondence "Thermostat sketch" {
//!     machine: "Thermostat"
//!     source: "thermostat.ino"
//!     period: 100 ms
//!     reads: ["analogRead(SENSOR) is temperature"]
//!     writes: ["digitalWrite(HEATER) is heater"]
//!     state: ["mode == 1 is in Heating", "mode == 0 is in Idle"]
//! }
//! ```
//!
//! The left of each `is` is the sketch's (a read, a write, an expression
//! over its globals, in C); the right is the model's: a variable, `in
//! <State>` (the state is active) or `time in <State>` (the milliseconds
//! since it was entered).
//!
//! The proof is an induction on the turns of `loop()`:
//! - **start**: after `setup()` and one `loop()`, and after the model's
//!   `init` and one step, with the same inputs, the two correspond;
//! - **step**: from any two states that correspond, with the same inputs, one
//!   `loop()` and one step leave two states that correspond.
//!
//! The Arduino functions are stand-ins: a read gives the input of the step,
//! a write records the level of its pin, `millis()` advances by one period a
//! turn. A read or a write of a pin the correspondence does not name fails
//! the proof, and so does a correspondence that does not hold: a wrong table
//! cannot give a wrong proof.

use super::equivalence::{decide, Answer};
use super::{from_generated_c, Generated, Machine};
use crate::compiler::ast::{AttributeValue, CorrespondenceDecl, Model};
use regex::Regex;
use std::path::Path;

/// What the model's side of a line is.
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Variable(String),
    /// The state is active.
    In(String),
    /// The milliseconds since the state was entered.
    TimeIn(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Correspondence {
    pub name: String,
    pub machine: String,
    pub sources: Vec<String>,
    /// The time of one `loop()`, when the correspondence gives it.
    pub period_us: Option<u64>,
    /// (`digitalRead` or `analogRead`, the pin as the sketch writes it, the input of the model)
    pub reads: Vec<(String, String, String)>,
    /// (the pin as the sketch writes it, what its level is in the model)
    pub writes: Vec<(String, Term)>,
    /// (an expression of the sketch, what it is in the model)
    pub state: Vec<(String, Term)>,
    /// What always holds of the sketch's own variables (`lit == 0 || lit == 1`):
    /// part of what is proven, and what keeps the proof to states the sketch can be in.
    pub invariants: Vec<String>,
    /// The bits of an `int` on the board: 16 on an AVR board (the Uno), 32 otherwise.
    pub int_bits: u8,
}

fn term(text: &str) -> Term {
    let text = text.trim();
    if let Some(state) = text.strip_prefix("time in ") {
        Term::TimeIn(state.trim().to_string())
    } else if let Some(state) = text.strip_prefix("in ") {
        Term::In(state.trim().to_string())
    } else {
        Term::Variable(text.to_string())
    }
}

/// Read a `correspondence` declaration.
pub fn read(declaration: &CorrespondenceDecl) -> Result<Correspondence, String> {
    let what = format!("correspondence '{}'", declaration.name);
    let text = |key: &str| declaration.attributes.get(key).and_then(|v| v.as_string()).map(str::to_string);
    let list = |key: &str| -> Vec<String> {
        match declaration.attributes.get(key) {
            Some(AttributeValue::List(items)) => items.iter().filter_map(|i| i.as_string()).map(str::to_string).collect(),
            Some(value) => value.as_string().map(|one| vec![one.to_string()]).unwrap_or_default(),
            None => Vec::new(),
        }
    };
    for key in declaration.attributes.keys() {
        if !["machine", "source", "period", "reads", "writes", "state", "invariant", "int", "description"].contains(&key.as_str()) {
            return Err(format!("{what}: '{key}' is not something a correspondence says (machine, source, period, reads, writes, state, invariant, int)"));
        }
    }
    let machine = text("machine").ok_or_else(|| format!("{what}: it names no machine (machine: \"...\")"))?;
    let sources = list("source");
    if sources.is_empty() {
        return Err(format!("{what}: it names no source (source: \"sketch.ino\")"));
    }
    let period_us = match declaration.attributes.get("period") {
        None => None,
        Some(AttributeValue::Quantity(q)) if q.dimension().label() == "time" => Some((q.canonical() * 1e6).round() as u64),
        Some(_) => return Err(format!("{what}: period is a time (period: 100 ms)")),
    };
    let split = |line: &str, of: &str| -> Result<(String, String), String> {
        line.rsplit_once(" is ").map(|(sketch, model)| (sketch.trim().to_string(), model.trim().to_string())).ok_or_else(|| format!("{what}: {of} \"{line}\" does not say what it is in the model (`<the sketch's> is <the model's>`)"))
    };
    let read_form = Regex::new(r"^(digitalRead|analogRead)\((.+)\)$").expect("a pattern of the correspondence");
    let write_form = Regex::new(r"^digitalWrite\((.+)\)$").expect("a pattern of the correspondence");
    let mut reads = Vec::new();
    for line in list("reads") {
        let (sketch, model) = split(&line, "the read")?;
        let found = read_form.captures(&sketch).ok_or_else(|| format!("{what}: \"{sketch}\" is not a read (digitalRead(pin) or analogRead(pin))"))?;
        reads.push((found[1].to_string(), found[2].trim().to_string(), model));
    }
    let mut writes = Vec::new();
    for line in list("writes") {
        let (sketch, model) = split(&line, "the write")?;
        let found = write_form.captures(&sketch).ok_or_else(|| format!("{what}: \"{sketch}\" is not a write (digitalWrite(pin))"))?;
        writes.push((found[1].trim().to_string(), term(&model)));
    }
    let mut state = Vec::new();
    for line in list("state") {
        let (sketch, model) = split(&line, "the line of state")?;
        state.push((sketch, term(&model)));
    }
    if writes.is_empty() && state.is_empty() {
        return Err(format!("{what}: it relates nothing (writes: [...] or state: [...]): there would be nothing to prove"));
    }
    let int_bits = match declaration.attributes.get("int").map(|bits| bits.as_integer()) {
        None => 32,
        Some(Some(16)) => 16,
        Some(Some(32)) => 32,
        Some(_) => return Err(format!("{what}: int is the bits of an int on the board, 16 (an AVR board such as the Uno) or 32")),
    };
    Ok(Correspondence { name: declaration.name.clone(), machine, sources, period_us, reads, writes, state, invariants: list("invariant"), int_bits })
}

/// The outcome of a proof of correspondence.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Outcome {
    /// Both obligations hold: every run of the code is a run of the machine.
    /// `unrelated`: the variables of the machine no line speaks of; the
    /// proof says nothing of them.
    Proven { seconds: f64, period_us: u64, int_bits: u8, unrelated: Vec<String> },
    /// One does not: which (`start` or `step`), and the lines of the
    /// correspondence that do not hold after it (none: the code read or
    /// wrote a pin the correspondence does not name).
    Refuted { obligation: String, lines: Vec<String>, seconds: f64 },
    /// No answer: why.
    Unknown { reason: String },
}

/// The stand-ins every sketch is given: what the Arduino core declares, as C.
const ARDUINO_H: &str = "/* Stand-ins for the Arduino core, for the proof (written by `arclang prove`). */\n#include <stdbool.h>\n#include <stdint.h>\n#define HIGH 1\n#define LOW 0\n#define INPUT 0\n#define OUTPUT 1\n#define INPUT_PULLUP 2\n#define LED_BUILTIN 13\n#define A0 14\n#define A1 15\n#define A2 16\n#define A3 17\n#define A4 18\n#define A5 19\n#define A6 20\n#define A7 21\ntypedef uint8_t byte;\ntypedef bool boolean;\ntypedef uint16_t word;\nvoid pinMode(int pin, int mode);\nint digitalRead(int pin);\nint analogRead(int pin);\nvoid digitalWrite(int pin, int level);\nunsigned long millis(void);\nvoid delay(unsigned long ms);\nvoid setup(void);\nvoid loop(void);\n";

/// The files of the proof: the model's C, the sketch with its stand-ins and
/// the correspondence as a C predicate, and the two obligations.
pub fn harness(correspondence: &Correspondence, model: &Generated, sketches: &[(String, String)], default_period_us: u64) -> Result<Proof, String> {
    let what = format!("correspondence '{}'", correspondence.name);
    let recovered = from_generated_c(model)?;
    let machine: &Machine = recovered.machines.iter().find(|m| m.name == correspondence.machine).ok_or_else(|| {
        format!("{what}: the model has no state_machine \"{}\" in its C ({})", correspondence.machine, recovered.machines.iter().map(|m| m.name.as_str()).collect::<Vec<_>>().join(", "))
    })?;
    let prefix = model.header_name.trim_end_matches(".h");
    let id = &machine.ident;
    let period_us = correspondence.period_us.unwrap_or(default_period_us);
    let period_ms = period_us / 1000;
    if period_ms == 0 || period_us % 1000 != 0 {
        return Err(format!("{what}: the period of one loop() is a whole number of milliseconds (`millis()` counts them); it is {period_us} us"));
    }
    let variable = |name: &str| machine.variables.iter().find(|v| v.name == name).ok_or_else(|| format!("{what}: state_machine \"{}\" has no variable '{name}'", machine.name));
    let state_index = |name: &str| {
        machine
            .states
            .iter()
            .position(|s| s.path == name)
            .or_else(|| machine.states.iter().position(|s| s.path.rsplit('.').next() == Some(name)))
            .ok_or_else(|| format!("{what}: state_machine \"{}\" has no state '{name}'", machine.name))
    };
    // The model's side of a line, as C over the state `s` and the inputs of the step,
    // and how it compares with the sketch's side.
    let relate = |sketch: &str, model: &Term| -> Result<String, String> {
        Ok(match model {
            Term::Variable(name) => {
                let v = variable(name)?;
                let read = if v.input { format!("arclang_inputs->{id}.v_{}", v.ident) } else { format!("s->{id}.v_{}", v.ident) };
                if v.ty == "Boolean" { format!("((({sketch}) != 0) == {read})") } else { format!("(({sketch}) == {read})") }
            }
            Term::In(state) => format!("((({sketch}) != 0) == s->{id}.active[{}])", state_index(state)?),
            // The time in a state means something while the state is active.
            Term::TimeIn(state) => {
                let index = state_index(state)?;
                format!("(!s->{id}.active[{index}] || ((unsigned long)({sketch}) == (unsigned long)s->{id}.in_state[{index}] * {period_ms}UL))")
            }
        })
    };

    let mut c = String::new();
    c.push_str("/* The proof that this code is a machine of the model: the code, its stand-ins,\n   the correspondence as a predicate, and the two obligations.\n   Written by `arclang prove`. */\n");
    c.push_str(&format!("#include \"model/{}\"\n#include \"arduino.h\"\n\n", model.header_name));
    c.push_str(&format!("static const {prefix}_inputs *arclang_inputs; /* the inputs of the step, read by both */\nstatic unsigned long arclang_ms; /* what millis() says */\nstatic unsigned long arclang_delayed; /* what the delay()s of this loop() add up to */\n"));
    c.push_str("static bool arclang_unnamed; /* the code read or wrote a pin the correspondence does not name */\n");
    for (rank, _) in correspondence.writes.iter().enumerate() {
        c.push_str(&format!("static bool arclang_level_{rank}; /* the level of the pin of write {rank} */\n"));
    }
    c.push_str("\n/* ---- the code as written ---- */\n");
    for (name, _) in sketches {
        c.push_str(&format!("#include \"sketch/{name}\"\n"));
    }
    c.push_str("\n/* ---- the stand-ins ---- */\n");
    c.push_str("void pinMode(int pin, int mode) { (void)pin; (void)mode; }\nunsigned long millis(void) { return arclang_ms; }\nvoid delay(unsigned long ms) { arclang_delayed += ms; }\n");
    let mut assumptions: Vec<String> = Vec::new();
    for kind in ["digitalRead", "analogRead"] {
        c.push_str(&format!("int {kind}(int pin)\n{{\n"));
        for (_, pin, name) in correspondence.reads.iter().filter(|(k, _, _)| k == kind) {
            let v = variable(name)?;
            if !v.input {
                return Err(format!("{what}: '{name}' is not an input of state_machine \"{}\": a read of the sketch is an input of the model", machine.name));
            }
            let read = format!("arclang_inputs->{id}.v_{}", v.ident);
            match (kind, v.ty) {
                ("digitalRead", "Boolean") => c.push_str(&format!("    if (pin == ({pin})) {{ return {read} ? 1 : 0; }}\n")),
                ("digitalRead", "Integer") => {
                    assumptions.push(format!("in.{id}.v_{} >= 0 && in.{id}.v_{} <= 1", v.ident, v.ident));
                    c.push_str(&format!("    if (pin == ({pin})) {{ return (int){read}; }}\n"));
                }
                ("analogRead", "Integer") => {
                    assumptions.push(format!("in.{id}.v_{} >= 0 && in.{id}.v_{} <= 1023", v.ident, v.ident));
                    c.push_str(&format!("    if (pin == ({pin})) {{ return (int){read}; }}\n"));
                }
                (_, ty) => return Err(format!("{what}: {kind}({pin}) is '{name}', a {ty}: a digital read is a Boolean or an Integer, an analog read an Integer (0 to 1023)")),
            }
        }
        c.push_str(&format!("    arclang_unnamed = true; /* a pin with {kind} that the correspondence does not name */\n    return 0;\n}}\n"));
    }
    c.push_str("void digitalWrite(int pin, int level)\n{\n");
    for (rank, (pin, _)) in correspondence.writes.iter().enumerate() {
        c.push_str(&format!("    if (pin == ({pin})) {{ arclang_level_{rank} = level != 0; return; }}\n"));
    }
    c.push_str("    arclang_unnamed = true; /* a pin the correspondence does not name */\n}\n");

    // Each line of the correspondence, as a condition and as it is written.
    let mut lines: Vec<(String, String)> = vec![
        (format!("s->{id}_started"), "the machine has started".to_string()),
        ("!arclang_unnamed".to_string(), UNNAMED.to_string()),
        (format!("(arclang_delayed == 0UL || arclang_delayed == {period_ms}UL)"), DELAYS.to_string()),
    ];
    for (rank, (pin, model_side)) in correspondence.writes.iter().enumerate() {
        lines.push((relate(&format!("arclang_level_{rank}"), model_side)?, format!("digitalWrite({pin}) is {}", said(model_side))));
    }
    for (sketch_side, model_side) in &correspondence.state {
        lines.push((relate(sketch_side, model_side)?, format!("{sketch_side} is {}", said(model_side))));
    }
    for invariant in &correspondence.invariants {
        lines.push((format!("(({invariant}) != 0)"), format!("invariant: {invariant}")));
    }
    c.push_str(&format!("\n/* ---- the correspondence ---- */\nstatic bool arclang_line(int line, const {prefix}_state *s)\n{{\n    switch (line) {{\n"));
    for (rank, (condition, written)) in lines.iter().enumerate() {
        c.push_str(&format!("    case {rank}: return {condition}; /* {} */\n", written.replace("*/", "* /")));
    }
    c.push_str("    default: return true;\n    }\n}\n");
    c.push_str(&format!("static bool arclang_corresponds(const {prefix}_state *s)\n{{\n    return {};\n}}\n", (0..lines.len()).map(|rank| format!("arclang_line({rank}, s)")).collect::<Vec<_>>().join(" && ")));

    c.push_str(&format!("\n/* ---- the obligations ---- */\nint main(void)\n{{\n    {prefix}_state s;\n    {prefix}_inputs in;\n    {prefix}_outputs out = {{0}};\n    arclang_inputs = &in;\n"));
    for assumption in &assumptions {
        c.push_str(&format!("    __CPROVER_assume({assumption});\n"));
    }
    c.push_str(&format!(
        "#ifdef ARCLANG_START\n    /* start: after setup() and one loop(), and after init and one step, the two correspond */\n    setup();\n    {{ unsigned long any; arclang_ms = any; }} /* the first loop() runs at any time */\n    arclang_delayed = 0UL;\n    loop();\n    arclang_ms += {period_ms}UL;\n    {prefix}_init(&s);\n    {prefix}_step(&s, &in, &out);\n#else\n    /* step: from any two states that correspond, one loop() and one step leave two that correspond */\n    __CPROVER_assume(arclang_corresponds(&s));\n    arclang_delayed = 0UL;\n    loop();\n    arclang_ms += {period_ms}UL;\n    {prefix}_step(&s, &in, &out);\n#endif\n#ifdef ARCLANG_LINE\n    __CPROVER_assert(arclang_line(ARCLANG_LINE, &s), \"this line of the correspondence holds\");\n#else\n    __CPROVER_assert(arclang_corresponds(&s), \"the code and the model correspond\");\n#endif\n    return 0;\n}}\n"
    ));

    let mut files = vec![
        ("proof.c".to_string(), c),
        ("arduino.h".to_string(), ARDUINO_H.to_string()),
        (format!("model/{}", model.header_name), model.header.to_string()),
        (format!("model/{}", model.code_name), model.code.to_string()),
    ];
    files.extend(sketches.iter().map(|(name, text)| (format!("sketch/{name}"), text.clone())));
    // The loops of the model run over its states or its queue; those of the code are unwound as far.
    let bound = Regex::new(r"(?m)^#define \w+_(?:STATE_COUNT|QUEUE_SIZE) (\d+)U").expect("a pattern of the harness");
    let unwind = bound.captures_iter(model.header).filter_map(|b| b[1].parse::<usize>().ok()).max().unwrap_or(1).max(16) + 2;
    // What no line speaks of: the proof holds, and says nothing of it.
    let related: Vec<&str> = correspondence.writes.iter().chain(&correspondence.state).filter_map(|(_, model_side)| if let Term::Variable(name) = model_side { Some(name.as_str()) } else { None }).collect();
    let unrelated = machine.variables.iter().filter(|v| !v.input && !related.contains(&v.name.as_str())).map(|v| v.name.clone()).collect();
    Ok(Proof { files, unwind, period_us, lines: lines.into_iter().map(|(_, written)| written).collect(), unrelated })
}

/// The line every correspondence holds without writing it.
const UNNAMED: &str = "the code reads and writes only the pins the correspondence names";

/// A `loop()` that waits says how long it takes: it is the period, or the
/// model and the code do not count the same time.
const DELAYS: &str = "the delay()s of one loop(), when it has some, add up to the period";

/// The files of a proof, the bound of its loops, the period of one turn and
/// the lines of the correspondence, in the order the proof numbers them.
pub struct Proof {
    pub files: Vec<(String, String)>,
    pub unwind: usize,
    pub period_us: u64,
    pub lines: Vec<String>,
    /// The variables of the machine, inputs apart, that no line relates.
    pub unrelated: Vec<String>,
}

fn said(term: &Term) -> String {
    match term {
        Term::Variable(name) => name.clone(),
        Term::In(state) => format!("in {state}"),
        Term::TimeIn(state) => format!("time in {state}"),
    }
}

/// Prove the correspondences of a model, each against the sources it names
/// (read beside the model). `work` receives the files of each proof.
pub fn prove(ast: &Model, base: &Path, only: Option<&str>, work: &Path, seconds: u64) -> Result<Vec<(Correspondence, Outcome)>, String> {
    if ast.correspondences.is_empty() {
        return Err("the model declares no correspondence (`correspondence \"Name\" { machine: ... source: ... }`)".to_string());
    }
    let (files, _) = crate::compiler::embedded::export(ast).map_err(|error| format!("the model generates no C to prove the code against: {error}"))?;
    let header = files.iter().find(|(name, text)| name.ends_with(".h") && !name.contains('/') && text.contains("/* state_machine \"")).ok_or("the model generates no header of machines")?;
    let code_name = header.0.replace(".h", ".c");
    let code = files.iter().find(|(name, _)| *name == code_name).ok_or("the model generates no source of machines")?;
    let model = Generated { header_name: &header.0, header: &header.1, code_name: &code.0, code: &code.1 };
    let default_period = Regex::new(r"(?m)^#define \w+_PERIOD_US (\d+)UL$").expect("a pattern of the proof").captures(&header.1).and_then(|p| p[1].parse::<u64>().ok()).unwrap_or(10_000);
    let mut outcomes = Vec::new();
    for declaration in ast.correspondences.iter().filter(|d| only.is_none_or(|name| d.name == name)) {
        let correspondence = read(declaration)?;
        let mut sketches = Vec::new();
        for source in &correspondence.sources {
            let path = base.join(source);
            let text = std::fs::read_to_string(&path).map_err(|e| format!("correspondence '{}': cannot read {}: {e}", correspondence.name, path.display()))?;
            let name = Path::new(source).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| source.clone());
            sketches.push((name, text));
        }
        let Proof { files: proof, unwind, period_us, lines, unrelated } = harness(&correspondence, &model, &sketches, default_period)?;
        let dir = work.join(correspondence.name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>());
        for (name, text) in &proof {
            let path = dir.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
            }
            std::fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        }
        let started = std::time::Instant::now();
        let model_source = format!("model/{}", model.code_name);
        let mut outcome = None;
        // The start is of the code as it is initialised; the step, of any state it can hold.
        for (obligation, mut flags) in [("start", vec!["-DARCLANG_START"]), ("step", vec!["--nondet-static"])] {
            if correspondence.int_bits == 16 {
                flags.push("--16");
            }
            match decide(&dir, &["proof.c", model_source.as_str()], &flags, unwind, seconds)? {
                Answer::Unsat => {}
                Answer::Sat(_) => {
                    // Which lines do not hold after it: each asked alone.
                    let mut failing = Vec::new();
                    for (rank, written) in lines.iter().enumerate() {
                        let only = format!("-DARCLANG_LINE={rank}");
                        let mut asked = flags.clone();
                        asked.push(only.as_str());
                        if let Answer::Sat(_) = decide(&dir, &["proof.c", model_source.as_str()], &asked, unwind, seconds)? {
                            failing.push(written.clone());
                        }
                    }
                    outcome = Some(Outcome::Refuted { obligation: obligation.to_string(), lines: failing, seconds: started.elapsed().as_secs_f64() });
                    break;
                }
                Answer::BeyondBound => {
                    outcome = Some(Outcome::Unknown { reason: format!("in the {obligation}, a loop of the code or of the model runs more than {unwind} times: the proof does not cover it") });
                    break;
                }
                Answer::Unknown(reason) => {
                    outcome = Some(Outcome::Unknown { reason: format!("{obligation}: {reason}") });
                    break;
                }
            }
        }
        let int_bits = correspondence.int_bits;
        outcomes.push((correspondence, outcome.unwrap_or(Outcome::Proven { seconds: started.elapsed().as_secs_f64(), period_us, int_bits, unrelated })));
    }
    if outcomes.is_empty() {
        return Err(format!("the model has no correspondence '{}'", only.unwrap_or_default()));
    }
    Ok(outcomes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn declaration(lines: &[(&str, AttributeValue)]) -> CorrespondenceDecl {
        CorrespondenceDecl { name: "Sketch".to_string(), attributes: lines.iter().map(|(k, v)| (k.to_string(), v.clone())).collect::<HashMap<_, _>>() }
    }
    fn text(value: &str) -> AttributeValue {
        AttributeValue::String(value.to_string())
    }
    fn list(values: &[&str]) -> AttributeValue {
        AttributeValue::List(values.iter().map(|v| text(v)).collect())
    }

    #[test]
    fn a_correspondence_is_read_line_by_line() {
        let read = read(&declaration(&[
            ("machine", text("Thermostat")),
            ("source", text("thermostat.ino")),
            ("reads", list(&["analogRead(SENSOR) is temperature"])),
            ("writes", list(&["digitalWrite(HEATER) is heater", "digitalWrite(12) is in Heating"])),
            ("state", list(&["mode == 1 is in Heating", "millis() - since is time in Heating", "count is ticks"])),
        ]))
        .unwrap();
        assert_eq!(read.reads, [("analogRead".to_string(), "SENSOR".to_string(), "temperature".to_string())]);
        assert_eq!(read.writes, [("HEATER".to_string(), Term::Variable("heater".to_string())), ("12".to_string(), Term::In("Heating".to_string()))]);
        assert_eq!(read.state[0], ("mode == 1".to_string(), Term::In("Heating".to_string())));
        assert_eq!(read.state[1], ("millis() - since".to_string(), Term::TimeIn("Heating".to_string())));
        assert_eq!(read.state[2], ("count".to_string(), Term::Variable("ticks".to_string())));
    }

    #[test]
    fn what_a_correspondence_cannot_say_is_refused_with_the_reason() {
        let wrong = |lines: &[(&str, AttributeValue)]| read(&declaration(lines)).unwrap_err();
        assert!(wrong(&[("source", text("a.ino")), ("state", list(&["m is in A"]))]).contains("names no machine"));
        assert!(wrong(&[("machine", text("M")), ("state", list(&["m is in A"]))]).contains("names no source"));
        assert!(wrong(&[("machine", text("M")), ("source", text("a.ino"))]).contains("relates nothing"));
        assert!(wrong(&[("machine", text("M")), ("source", text("a.ino")), ("state", list(&["mode == 1"]))]).contains("does not say what it is in the model"));
        assert!(wrong(&[("machine", text("M")), ("source", text("a.ino")), ("reads", list(&["Serial.read() is x"])), ("state", list(&["m is in A"]))]).contains("is not a read"));
        assert!(wrong(&[("machine", text("M")), ("source", text("a.ino")), ("stat", list(&["m is in A"]))]).contains("'stat' is not something a correspondence says"));
    }
}
