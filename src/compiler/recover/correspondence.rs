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
    /// The binary the correspondence names in place of sources, and where its functions are.
    pub binary: Option<super::binary::Binary>,
    /// The time of one `loop()`, when the correspondence gives it.
    pub period_us: Option<u64>,
    /// (`digitalRead` or `analogRead`, the pin as the sketch writes it, the input of the model)
    pub reads: Vec<(String, String, String)>,
    /// (the pin as the sketch writes it, what its level is in the model)
    pub writes: Vec<(String, Term)>,
    /// (`RISING`, `FALLING` or `CHANGE`, the pin as the sketch writes it, the event of the model each such edge is)
    pub events: Vec<(String, String, String)>,
    /// (the pin as the sketch writes it, the Integer variable of the model its `analogWrite` value is)
    pub pwm: Vec<(String, String)>,
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
        if !["machine", "source", "binary", "functions", "period", "reads", "writes", "events", "state", "invariant", "int", "description"].contains(&key.as_str()) {
            return Err(format!("{what}: '{key}' is not something a correspondence says (machine, source, binary, functions, period, reads, writes, events, state, invariant, int)"));
        }
    }
    let machine = text("machine").ok_or_else(|| format!("{what}: it names no machine (machine: \"...\")"))?;
    let sources = list("source");
    let binary_file = text("binary");
    if sources.is_empty() && binary_file.is_none() {
        return Err(format!("{what}: it names no source (source: \"sketch.ino\")"));
    }
    if !sources.is_empty() && binary_file.is_some() {
        return Err(format!("{what}: it names a source and a binary: the proof is of one or of the other"));
    }
    // A source is read, and what a tool says of it is printed: it is a file
    // beside the model or below it, never one reached by climbing out.
    for source in sources.iter().chain(&binary_file) {
        let path = Path::new(source);
        if path.is_absolute() || path.components().any(|part| !matches!(part, std::path::Component::Normal(_))) {
            return Err(format!("{what}: the source \"{source}\" is not a file beside the model or below it (no absolute path, no `..`)"));
        }
    }
    let binary = match binary_file {
        Some(file) => Some(super::binary::Binary { file, functions: super::binary::functions(&list("functions"), &what)?, ..Default::default() }),
        None if declaration.attributes.contains_key("functions") => return Err(format!("{what}: functions are those of a binary (binary: \"firmware.hex\")")),
        None => None,
    };
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
    let mut pwm = Vec::new();
    let analog_form = Regex::new(r"^analogWrite\((.+)\)$").expect("a pattern of the correspondence");
    for line in list("writes") {
        let (sketch, model) = split(&line, "the write")?;
        if let Some(found) = analog_form.captures(&sketch) {
            // The value written (0 to 255) is a variable of the model, never a state.
            match term(&model) {
                Term::Variable(name) => pwm.push((found[1].trim().to_string(), name)),
                _ => return Err(format!("{what}: \"{line}\": what analogWrite writes is a variable of the machine (an Integer)")),
            }
            continue;
        }
        let found = write_form.captures(&sketch).ok_or_else(|| format!("{what}: \"{sketch}\" is not a write (digitalWrite(pin) or analogWrite(pin))"))?;
        writes.push((found[1].trim().to_string(), term(&model)));
    }
    // An edge of a pin taken by an interrupt is an event of the machine.
    let edge_form = Regex::new(r"^(RISING|FALLING|CHANGE) on (.+)$").expect("a pattern of the correspondence");
    let mut events = Vec::new();
    for line in list("events") {
        let (sketch, model) = split(&line, "the event")?;
        let found = edge_form.captures(&sketch).ok_or_else(|| format!("{what}: \"{sketch}\" is not an edge taken by an interrupt (RISING on pin, FALLING on pin or CHANGE on pin)"))?;
        events.push((found[1].to_string(), found[2].trim().to_string(), model));
    }
    let mut state = Vec::new();
    for line in list("state") {
        let (sketch, model) = split(&line, "the line of state")?;
        state.push((sketch, term(&model)));
    }
    if writes.is_empty() && pwm.is_empty() && state.is_empty() {
        return Err(format!("{what}: it relates nothing (writes: [...] or state: [...]): there would be nothing to prove"));
    }
    if binary.is_some() && !events.is_empty() {
        return Err(format!("{what}: the interrupts of a binary are not read yet (events)"));
    }
    let int_bits = match declaration.attributes.get("int").map(|bits| bits.as_integer()) {
        // The binary is that of an AVR: its int has 16 bits.
        None if binary.is_some() => 16,
        Some(Some(32)) if binary.is_some() => return Err(format!("{what}: the binary is that of an AVR board, whose int has 16 bits")),
        None => 32,
        Some(Some(16)) => 16,
        Some(Some(32)) => 32,
        Some(_) => return Err(format!("{what}: int is the bits of an int on the board, 16 (an AVR board such as the Uno) or 32")),
    };
    Ok(Correspondence { name: declaration.name.clone(), machine, sources, binary, period_us, reads, writes, events, pwm, state, invariants: list("invariant"), int_bits })
}

/// The outcome of a proof of correspondence.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Outcome {
    /// Both obligations hold: every run of the code is a run of the machine.
    /// `unrelated`: the variables of the machine no line speaks of; the
    /// proof says nothing of them.
    /// `board_clock`: whether the proof still holds on the clock of the
    /// board, when the board is one whose clock is known not to be the one
    /// the proof states (an AVR).
    /// `interrupts`: when the code has interrupts, the places where it uses
    /// what a routine changes while interrupts are on. The proof takes an
    /// interrupt between two `loop()`s: one taken at such a place is not covered.
    Proven { seconds: f64, period_us: u64, int_bits: u8, unrelated: Vec<String>, board_clock: Option<BoardClock>, interrupts: Option<Interrupts> },
    /// One does not: which (`start` or `step`), and the lines of the
    /// correspondence that do not hold after it (none: the code read or
    /// wrote a pin the correspondence does not name).
    /// `case`: a case that shows it, when one could be read.
    Refuted { obligation: String, lines: Vec<String>, case: Option<Case>, seconds: f64 },
    /// No answer: why.
    Unknown { reason: String },
}

/// The proof again, on the clock of the board. The proof states a clock on
/// which `millis()` advances by the period at each `loop()`. On an AVR it
/// counts ticks of 1.024 ms: a time read can be one millisecond short, and a
/// duration one millisecond off either way. A sketch that compares a
/// duration with an exact number of its loops then acts one loop later some
/// of the time (measured on an UNO: 19 presses of 300).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum BoardClock {
    /// From any two states that correspond (times within one millisecond),
    /// one `loop()` and one step leave two that correspond.
    Holds,
    /// They can leave two that do not: the lines, and a case.
    Fails { lines: Vec<String>, case: Option<Case> },
    Unknown { reason: String },
}

/// How a proof takes the interrupts of the code: between two `loop()`s, and
/// what was asked of one taken in the middle of a `loop()`.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Interrupts {
    /// Where the code uses, with interrupts on, what a routine changes.
    pub shared: Vec<String>,
    pub middle: Middle,
}

/// An interrupt taken in the middle of `loop()`, at a use of what its
/// routine changes (see `interleave`).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Middle {
    /// It leaves what an interrupt just before the `loop()`, or just after it, leaves.
    Same,
    /// It can leave what neither does: the routine.
    Differs { routine: String },
    Unknown { reason: String },
}

/// What is printed under a proof of how it takes the interrupts of the code.
pub fn interrupts_said(interrupts: Option<&Interrupts>) -> Vec<String> {
    let Some(interrupts) = interrupts else { return Vec::new() };
    let places = interrupts.shared.iter().map(|place| format!("  {place}"));
    match &interrupts.middle {
        Middle::Same if interrupts.shared.is_empty() => vec!["interrupts: taken between two loop()s; what their routines change is only used elsewhere with interrupts off".to_string()],
        Middle::Same => {
            let mut said = vec!["interrupts: taken between two loop()s, or in the middle of one: proven that an interrupt at any use of what its routine changes leaves what one taken just before the loop(), or just after it, leaves (one interrupt in the loop(), after none, one or two since the last)".to_string()];
            said.extend(places);
            said
        }
        Middle::Differs { routine } => {
            let mut said = vec![format!("interrupts: taken between two loop()s. In the middle of one, NOT the same: an interrupt of `{routine}` taken at one of these uses can leave what neither an interrupt just before the loop() nor one just after it leaves (something is lost, or counted twice):")];
            said.extend(places);
            said
        }
        Middle::Unknown { reason } => {
            let mut said = vec![format!("interrupts: taken between two loop()s. In the middle of one: not asked ({reason})")];
            said.extend(places);
            said
        }
    }
}

/// What is printed under a proof of what it is on the clock of the board.
pub fn board_clock_said(verdict: Option<&BoardClock>) -> Vec<String> {
    const CLOCK: &str = "on the clock of the board (an AVR's millis() counts ticks of 1.024 ms: a time read can be 1 ms short)";
    match verdict {
        None => Vec::new(),
        Some(BoardClock::Holds) => vec![format!("{CLOCK}: proven too")],
        Some(BoardClock::Unknown { reason }) => vec![format!("{CLOCK}: neither proven nor refuted ({reason})")],
        Some(BoardClock::Fails { lines, case }) => {
            let mut said = vec![format!("{CLOCK}: NOT proven. The proof above is for a clock that advances by the period at each loop(); on the board, one loop() and one step can leave the two apart:")];
            said.extend(lines.iter().map(|line| format!("  does not hold after it: {line}")));
            if let Some(case) = case {
                said.push("  a case that shows it (the code's side and the model's side of each line):".to_string());
                said.extend(case.said().into_iter().map(|line| format!("    {line}")));
            }
            said.push("  a duration compared with an exact number of loops is on the edge: the code can act one loop later than the machine".to_string());
            said
        }
    }
}

/// The stand-ins every sketch is given: what the Arduino core declares, as C.
const ARDUINO_H: &str = "/* Stand-ins for the Arduino core, for the proof (written by `arclang prove`). */\n#include <stdbool.h>\n#include <stdint.h>\n#define HIGH 1\n#define LOW 0\n#define INPUT 0\n#define OUTPUT 1\n#define INPUT_PULLUP 2\n#define LED_BUILTIN 13\n#define A0 14\n#define A1 15\n#define A2 16\n#define A3 17\n#define A4 18\n#define A5 19\n#define A6 20\n#define A7 21\ntypedef uint8_t byte;\ntypedef bool boolean;\ntypedef uint16_t word;\nvoid pinMode(int pin, int mode);\nint digitalRead(int pin);\nint analogRead(int pin);\nvoid digitalWrite(int pin, int level);\nunsigned long millis(void);\nvoid delay(unsigned long ms);\nunsigned long micros(void);\nvoid delayMicroseconds(unsigned int us);\nvoid analogWrite(int pin, int value);\n#define min(a, b) ((a) < (b) ? (a) : (b))\n#define max(a, b) ((a) > (b) ? (a) : (b))\n#define abs(x) ((x) > 0 ? (x) : -(x))\n#define constrain(x, low, high) ((x) < (low) ? (low) : ((x) > (high) ? (high) : (x)))\nstatic long map(long x, long in_min, long in_max, long out_min, long out_max) { return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min; }\n/* The serial port: what is printed is not looked at; a read is an input nobody named. */\nstruct arclang_serial_port { void (*begin)(); void (*end)(); void (*print)(); void (*println)(); void (*write)(); void (*flush)(); int (*available)(void); int (*read)(void); int (*peek)(void); };\nextern const struct arclang_serial_port Serial;\n/* Interrupts: an edge of a pin calls a function of the code. */\nvoid arclang_point(void); /* a use of what a routine changes: where an interrupt may fall */\n#define CHANGE 1\n#define FALLING 2\n#define RISING 3\n#define digitalPinToInterrupt(pin) (pin)\nvoid attachInterrupt(int pin, void (*routine)(void), int mode);\nvoid detachInterrupt(int pin);\nvoid noInterrupts(void);\nvoid interrupts(void);\nvoid setup(void);\nvoid loop(void);\n";

/// The fixed-width types of the proof, written for the board and not read
/// from the C library of the machine that runs the proof: there `uint32_t`
/// is an `unsigned int`, which has 16 bits once an `int` has 16. On an AVR
/// board an `int` has 16 bits and a `long` 32; on a 32-bit board both have
/// 32. The sizes are checked where the proof is compiled.
const STDINT_H: &str = "/* The fixed-width types of the board, for the proof (written by `arclang prove`). */\n#ifndef ARCLANG_PROOF_STDINT_H\n#define ARCLANG_PROOF_STDINT_H\ntypedef signed char int8_t;\ntypedef unsigned char uint8_t;\n#ifdef ARCLANG_INT16\ntypedef int int16_t;\ntypedef unsigned int uint16_t;\ntypedef long int32_t;\ntypedef unsigned long uint32_t;\n#define INT32_C(c) c##L\n#define UINT32_C(c) c##UL\n#else\ntypedef short int16_t;\ntypedef unsigned short uint16_t;\ntypedef int int32_t;\ntypedef unsigned int uint32_t;\n#define INT32_C(c) c\n#define UINT32_C(c) c##U\n#endif\ntypedef long long int64_t;\ntypedef unsigned long long uint64_t;\n#define INT8_MAX 127\n#define INT8_MIN (-128)\n#define UINT8_MAX 255\n#define INT16_MAX 32767\n#define INT16_MIN (-32767 - 1)\n#define UINT16_MAX ((uint16_t)65535U)\n#define INT32_MAX INT32_C(2147483647)\n#define INT32_MIN (-INT32_MAX - 1)\n#define UINT32_MAX UINT32_C(4294967295)\n#define INT64_MAX 9223372036854775807LL\n#define INT64_MIN (-INT64_MAX - 1)\n#define UINT64_MAX 18446744073709551615ULL\n#define INT64_C(c) c##LL\n#define UINT64_C(c) c##ULL\n_Static_assert(sizeof(int16_t) == 2 && sizeof(uint16_t) == 2, \"int16_t has 16 bits\");\n_Static_assert(sizeof(int32_t) == 4 && sizeof(uint32_t) == 4, \"int32_t has 32 bits\");\n_Static_assert(sizeof(int64_t) == 8 && sizeof(uint64_t) == 8, \"int64_t has 64 bits\");\n_Static_assert(sizeof(unsigned long) == 4, \"a long has 32 bits on the board: millis() counts on 32\");\n#ifdef ARCLANG_INT16\n_Static_assert(sizeof(int) == 2, \"an int has 16 bits on the board\");\n#else\n_Static_assert(sizeof(int) == 4, \"an int has 32 bits on the board\");\n#endif\n#endif\n";
const STDBOOL_H: &str = "#ifndef ARCLANG_PROOF_STDBOOL_H\n#define ARCLANG_PROOF_STDBOOL_H\n#define bool _Bool\n#define true 1\n#define false 0\n#endif\n";

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
                // A state that is active has been so for one step at least: without it a
                // time could be read before it was taken, which no run does.
                format!("(!s->{id}.active[{index}] || (s->{id}.in_state[{index}] >= 1U && ARCLANG_SAME_TIME(ARCLANG_ELAPSED({sketch}), (unsigned long long)s->{id}.in_state[{index}] * {period_ms}ULL)))")
            }
        })
    };

    // The two sides of a line as numbers, for the case that refutes a proof
    // (the time in a state that is not active is -1).
    let sides = |sketch: &str, model: &Term| -> Result<(String, String), String> {
        Ok(match model {
            Term::Variable(name) => {
                let v = variable(name)?;
                let read = if v.input { format!("arclang_inputs->{id}.v_{}", v.ident) } else { format!("s.{id}.v_{}", v.ident) };
                if v.ty == "Boolean" { (format!("(({sketch}) != 0)"), format!("({read} != 0)")) } else { (format!("({sketch})"), read) }
            }
            Term::In(state) => (format!("(({sketch}) != 0)"), format!("(s.{id}.active[{}] != 0)", state_index(state)?)),
            Term::TimeIn(state) => {
                let index = state_index(state)?;
                (format!("({sketch})"), format!("(s.{id}.active[{index}] ? (long long)s.{id}.in_state[{index}] * {period_ms}LL : -1LL)"))
            }
        })
    };

    let mut c = String::new();
    c.push_str("/* The proof that this code is a machine of the model: the code, its stand-ins,\n   the correspondence as a predicate, and the two obligations.\n   Written by `arclang prove`. */\n");
    c.push_str(&format!("#include \"model/{}\"\n#include \"arduino.h\"\n\n", model.header_name));
    c.push_str(&format!("static const {prefix}_inputs *arclang_inputs; /* the inputs of the step, read by both */\nstatic unsigned long arclang_ms; /* the time, in ms */\nstatic unsigned long arclang_late; /* how far short of it millis() reads, on the clock of the board */\nstatic unsigned long arclang_delayed; /* what the delay()s of this loop() add up to */\n"));
    c.push_str("static bool arclang_unnamed; /* the code read or wrote a pin the correspondence does not name */\n");
    for (rank, _) in correspondence.writes.iter().enumerate() {
        c.push_str(&format!("static bool arclang_level_{rank}; /* the level of the pin of write {rank} */\n"));
    }
    for (rank, _) in correspondence.pwm.iter().enumerate() {
        c.push_str(&format!("static int arclang_duty_{rank}; /* the value last written to the pin of analog write {rank} */\n"));
    }
    for (rank, _) in correspondence.events.iter().enumerate() {
        c.push_str(&format!("static bool arclang_attached_{rank}; /* the routine of event {rank} is attached to its pin */\n"));
    }
    c.push_str("static bool arclang_on = true; /* interrupts are on */\nstatic bool arclang_watch, arclang_in, arclang_fired; /* an interrupt is looked for in loop(); its routine runs; it was taken */\nstatic unsigned arclang_seen, arclang_at; /* the uses passed, and the one the interrupt falls at */\n");
    // The routine the code attaches to each pin of an event: read in the code, and checked where it attaches it.
    let attach = Regex::new(r"attachInterrupt\s*\(\s*(?:digitalPinToInterrupt\s*\(\s*([^()]+?)\s*\)|([^(),]+?))\s*,\s*(\w+)\s*,\s*(\w+)\s*\)").expect("a pattern of the harness");
    let mut routines: Vec<(String, String)> = Vec::new();
    for (mode, pin, event) in &correspondence.events {
        let macro_name = Regex::new(&format!(r#"(?m)^#define (\w+_EV_\w+) \d+U /\* event "{}" \*/"#, regex::escape(event))).expect("a pattern of the harness").captures(model.header).map(|m| m[1].to_string());
        let Some(macro_name) = macro_name else {
            return Err(format!("{what}: state_machine \"{}\" has no event '{event}' (the trigger of a transition)", machine.name));
        };
        // Without the code (the compiler checking the declaration alone), the routine is not looked for.
        if sketches.is_empty() {
            continue;
        }
        let found = sketches.iter().flat_map(|(_, text)| attach.captures_iter(text)).find(|found| found.get(1).or(found.get(2)).is_some_and(|p| p.as_str().trim() == pin));
        let Some(found) = found else {
            return Err(format!("{what}: the code attaches no interrupt to {pin} (attachInterrupt(digitalPinToInterrupt({pin}), routine, {mode}))"));
        };
        if &found[4] != mode {
            return Err(format!("{what}: the code attaches the interrupt of {pin} on {}, the correspondence says {mode}", &found[4]));
        }
        routines.push((found[3].to_string(), macro_name));
    }
    c.push_str("\n/* ---- the code as written ---- */\n");
    for (name, _) in sketches {
        c.push_str(&format!("#include \"sketch/{name}\"\n"));
    }
    c.push_str("\n/* ---- the stand-ins ---- */\n");
    c.push_str("void pinMode(int pin, int mode) { (void)pin; (void)mode; }\nunsigned long millis(void) { return arclang_ms - arclang_late; }\nvoid delay(unsigned long ms) { arclang_delayed += ms; }\n");
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
        if kind == "digitalRead" {
            // A pin the code writes reads back as it was last written.
            for (rank, (pin, _)) in correspondence.writes.iter().enumerate() {
                c.push_str(&format!("    if (pin == ({pin})) {{ return arclang_level_{rank} ? 1 : 0; }}\n"));
            }
        }
        c.push_str(&format!("    arclang_unnamed = true; /* a pin with {kind} that the correspondence does not name */\n    return 0;\n}}\n"));
    }
    c.push_str("void digitalWrite(int pin, int level)\n{\n");
    for (rank, (pin, _)) in correspondence.writes.iter().enumerate() {
        c.push_str(&format!("    if (pin == ({pin})) {{ arclang_level_{rank} = level != 0; return; }}\n"));
    }
    // A level written to a pin of analog writes is its lowest or its highest value.
    for (rank, (pin, _)) in correspondence.pwm.iter().enumerate() {
        c.push_str(&format!("    if (pin == ({pin})) {{ arclang_duty_{rank} = level != 0 ? 255 : 0; return; }}\n"));
    }
    c.push_str("    arclang_unnamed = true; /* a pin the correspondence does not name */\n}\n");
    c.push_str("void analogWrite(int pin, int value)\n{\n");
    for (rank, (pin, _)) in correspondence.pwm.iter().enumerate() {
        c.push_str(&format!("    if (pin == ({pin})) {{ arclang_duty_{rank} = value; return; }}\n"));
    }
    c.push_str("    arclang_unnamed = true; /* a pin the correspondence does not name */\n}\n");
    // An interrupt attached as the correspondence says is noted; any other is one nobody named.
    c.push_str("void attachInterrupt(int pin, void (*routine)(void), int mode)\n{\n");
    for (rank, ((mode, pin, _), (routine, _))) in correspondence.events.iter().zip(&routines).enumerate() {
        c.push_str(&format!("    if (pin == ({pin}) && routine == {routine} && mode == {mode}) {{ arclang_attached_{rank} = true; return; }}\n"));
    }
    c.push_str("    (void)pin; (void)routine; (void)mode;\n    arclang_unnamed = true; /* an interrupt the correspondence does not name */\n}\nvoid detachInterrupt(int pin) { (void)pin; arclang_unnamed = true; }\nvoid noInterrupts(void) { arclang_on = false; }\nvoid interrupts(void) { arclang_on = true; }\n");
    // A use of what a routine changes, outside the routine and with interrupts on: where an interrupt may fall.
    c.push_str("void arclang_point(void)\n{\n#ifdef ARCLANG_INTERLEAVE\n    if (arclang_watch && arclang_on && !arclang_in) {\n        if (arclang_seen == arclang_at) { arclang_in = true; ARCLANG_ROUTINE(); arclang_in = false; arclang_fired = true; }\n        arclang_seen++;\n    }\n#endif\n}\n");
    // The state of the code kept and compared, for the interrupt in the middle of loop().
    let routine_names: Vec<String> = routines.iter().map(|(routine, _)| routine.clone()).collect();
    let watched_names: Vec<String> = super::interleave::changed(sketches, &routine_names).into_iter().map(|(_, variable)| variable).collect();
    let mut kept: Result<Vec<(String, String)>, String> = Ok(Vec::new());
    if !routine_names.is_empty() {
        for (_, text) in sketches {
            match (super::interleave::state(text), &mut kept) {
                (Ok(more), Ok(all)) => all.extend(more),
                (Err(reason), _) => kept = Err(reason),
                _ => {}
            }
        }
        if let Ok(state) = &kept {
            c.push_str(&super::interleave::keeper(state, correspondence.writes.len(), correspondence.pwm.len()));
        }
    }
    // The finer clock: the same time in microseconds, never short (its tick is 4 us).
    c.push_str("unsigned long micros(void) { return arclang_ms * 1000UL; }\nvoid delayMicroseconds(unsigned int us) { (void)us; }\n");
    // What the code prints on its serial port is no part of what is proven; what it would read from it is an input nobody named.
    c.push_str("static void arclang_serial_out() { }\nstatic int arclang_serial_in(void) { arclang_unnamed = true; return 0; }\nconst struct arclang_serial_port Serial = { arclang_serial_out, arclang_serial_out, arclang_serial_out, arclang_serial_out, arclang_serial_out, arclang_serial_out, arclang_serial_in, arclang_serial_in, arclang_serial_in };\n");

    // Each line of the correspondence, as a condition and as it is written.
    let mut lines: Vec<(String, String)> = vec![
        (format!("s->{id}_started"), "the machine has started".to_string()),
        ("!arclang_unnamed".to_string(), UNNAMED.to_string()),
        (format!("(arclang_delayed == 0UL || arclang_delayed == {period_ms}UL)"), DELAYS.to_string()),
        ("(arclang_late <= ARCLANG_LATE_MOST)".to_string(), "millis() reads the time, or one millisecond short of it on the clock of the board".to_string()),
    ];
    // What a refuting case shows: (the code's side as written and as C, the model's side as written and as C).
    let mut shown: Vec<(String, String, String, String)> = Vec::new();
    if correspondence.binary.is_some() {
        lines.extend(super::binary::lines());
    }
    for (rank, (pin, model_side)) in correspondence.writes.iter().enumerate() {
        let level = format!("arclang_level_{rank}");
        lines.push((relate(&level, model_side)?, format!("digitalWrite({pin}) is {}", said(model_side))));
        let (code, model) = sides(&level, model_side)?;
        shown.push((format!("the level of {pin}"), code, said(model_side), model));
    }
    for (rank, (mode, pin, event)) in correspondence.events.iter().enumerate() {
        lines.push((format!("arclang_attached_{rank}"), format!("{mode} on {pin} is {event}: its routine is attached")));
    }
    for (rank, (pin, name)) in correspondence.pwm.iter().enumerate() {
        let v = variable(name)?;
        if v.input || v.ty != "Integer" {
            return Err(format!("{what}: analogWrite({pin}) is '{name}': what it writes is a variable of the machine that is an Integer and not an input"));
        }
        let duty = format!("arclang_duty_{rank}");
        let model_side = Term::Variable(name.clone());
        lines.push((relate(&duty, &model_side)?, format!("analogWrite({pin}) is {name}")));
        let (code, model) = sides(&duty, &model_side)?;
        shown.push((format!("the value written to {pin}"), code, name.clone(), model));
    }
    for (sketch_side, model_side) in &correspondence.state {
        lines.push((relate(sketch_side, model_side)?, format!("{sketch_side} is {}", said(model_side))));
        let (code, model) = sides(sketch_side, model_side)?;
        shown.push((sketch_side.clone(), code, said(model_side), model));
    }
    let mut inputs_shown: Vec<(String, String)> = Vec::new();
    for (_, _, name) in &correspondence.reads {
        let v = variable(name)?;
        inputs_shown.push((name.clone(), if v.ty == "Boolean" { format!("(in.{id}.v_{} != 0)", v.ident) } else { format!("in.{id}.v_{}", v.ident) }));
    }
    let record = |moment: &str| -> String {
        let mut text = String::from("#ifdef ARCLANG_WITNESS\n");
        for (rank, (_, code, _, model)) in shown.iter().enumerate() {
            text.push_str(&format!("    long long arclang_w{moment}c_{rank} = (long long)({code}); long long arclang_w{moment}m_{rank} = (long long)({model});\n"));
        }
        if moment == "b" {
            for (rank, (_, read)) in inputs_shown.iter().enumerate() {
                text.push_str(&format!("    long long arclang_wi_{rank} = (long long)({read});\n"));
            }
        }
        text.push_str("#endif\n");
        text
    };
    let (before, after) = (record("b"), record("a"));
    // The start has no state before it, and has inputs: a case that refutes it gives them too.
    let mut start_inputs = String::from("#ifdef ARCLANG_WITNESS\n");
    for (rank, (_, read)) in inputs_shown.iter().enumerate() {
        start_inputs.push_str(&format!("    long long arclang_wi_{rank} = (long long)({read});\n"));
    }
    start_inputs.push_str("#endif\n");
    for invariant in &correspondence.invariants {
        lines.push((format!("(({invariant}) != 0)"), format!("invariant: {invariant}")));
    }
    // A time of the code and a time of the model are the same time. The code's
    // is taken as it would be read now by a clock that is not short: what is left
    // between it and the model's is how short the clock was when the code started
    // to count, nothing on the clock the proof states, at most a millisecond on
    // the clock of the board.
    c.push_str("\n/* ---- the clock ---- */\n#ifdef ARCLANG_BOARD_CLOCK\n#define ARCLANG_LATE_MOST 1UL\n#else\n#define ARCLANG_LATE_MOST 0UL\n#endif\n#define ARCLANG_ELAPSED(code) ({ unsigned long arclang_keep = arclang_late; arclang_late = 0UL; unsigned long arclang_read = (unsigned long)(code); arclang_late = arclang_keep; arclang_read; })\n#define ARCLANG_SAME_TIME(code, model) ({ unsigned long long arclang_code = (code); arclang_code >= (model) && arclang_code - (model) <= ARCLANG_LATE_MOST; })\n");
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
    // The events of the step: each calls the routine of its interrupt, in the order
    // they arrived, between the last loop() and this one.
    let has_queue = model.header.contains(&format!("{}_{}_QUEUE_SIZE", prefix.to_uppercase(), id.to_uppercase()));
    let mut interrupts = String::new();
    if has_queue {
        let size = format!("{}_{}_QUEUE_SIZE", prefix.to_uppercase(), id.to_uppercase());
        c.push_str(&format!("    __CPROVER_assume(in.{id}.queued <= {size});\n"));
        if !routines.is_empty() {
            interrupts.push_str(&format!("    for (unsigned arclang_k = 0; arclang_k < in.{id}.queued; arclang_k++) {{\n"));
            for (routine, event) in &routines {
                interrupts.push_str(&format!("        if (in.{id}.queue[arclang_k] == {event}) {{ {routine}(); }}\n"));
            }
            interrupts.push_str("    }\n");
        }
    } else if !correspondence.events.is_empty() {
        return Err(format!("{what}: state_machine \"{}\" takes no event", machine.name));
    }
    if !routine_names.is_empty() && kept.is_ok() {
        // An interrupt in the middle of loop(): from any two states that correspond, the routine run at a
        // use of what it changes leaves what it leaves run just before the loop(), or just after it.
        c.push_str("#ifdef ARCLANG_INTERLEAVE\n    __CPROVER_assume(arclang_corresponds(&s));\n    {\n        struct arclang_kept first, before, after, now;\n        unsigned at;\n        arclang_watch = false; arclang_in = false;\n        /* interrupts already taken since the last loop(): none, one or two */\n        { unsigned already; __CPROVER_assume(already <= 2U); for (unsigned k = 0U; k < already; k++) { ARCLANG_ROUTINE(); } }\n        arclang_keep(&first);\n        arclang_on = true; arclang_delayed = 0UL; ARCLANG_ROUTINE(); loop(); arclang_keep(&before);\n        arclang_put_back(&first); arclang_on = true; arclang_delayed = 0UL; loop(); ARCLANG_ROUTINE(); arclang_keep(&after);\n        arclang_put_back(&first); arclang_on = true; arclang_delayed = 0UL;\n        arclang_at = at; arclang_seen = 0U; arclang_fired = false; arclang_watch = true;\n        loop();\n        arclang_watch = false;\n        __CPROVER_assume(arclang_fired);\n        arclang_keep(&now);\n        __CPROVER_assert(arclang_same(&now, &before) || arclang_same(&now, &after), \"an interrupt in the middle of loop() is one before it or one after it\");\n    }\n    return 0;\n#endif\n");
    }
    c.push_str(&format!(
        "#ifdef ARCLANG_START\n    /* start: after setup() and one loop(), and after init and one step, the two correspond */\n{start_inputs}    setup();\n    arclang_ms = 0UL; /* the first loop() runs as millis() starts counting */\n    arclang_late = 0UL;\n    arclang_delayed = 0UL;\n{interrupts}    loop();\n    arclang_ms += {period_ms}UL;\n    {prefix}_init(&s);\n    {prefix}_step(&s, &in, &out);\n#else\n    /* step: from any two states that correspond, one loop() and one step leave two that correspond */\n    __CPROVER_assume(arclang_corresponds(&s));\n{before}    arclang_delayed = 0UL;\n{interrupts}    loop();\n    arclang_ms += {period_ms}UL;\n    {{ unsigned long late; __CPROVER_assume(late <= ARCLANG_LATE_MOST); arclang_late = late; }} /* what the next loop() will read */\n    {prefix}_step(&s, &in, &out);\n#endif\n{after}#ifdef ARCLANG_LINE\n    __CPROVER_assert(arclang_line(ARCLANG_LINE, &s), \"this line of the correspondence holds\");\n#else\n    __CPROVER_assert(arclang_corresponds(&s), \"the code and the model correspond\");\n#endif\n    return 0;\n}}\n"
    ));

    // One step of the machine is one loop(): the machine counts its times
    // in steps of the correspondence's period, whatever period the model's C
    // would take by default.
    let macro_name = format!("{}_{}_PERIOD_US", prefix.to_uppercase(), id.to_uppercase());
    if !model.header.contains(&format!("#ifndef {macro_name}\n")) {
        return Err(format!("{what}: the C of state_machine \"{}\" does not let its period be given ({macro_name})", machine.name));
    }
    let define = format!("-D{macro_name}={period_us}UL");
    let mut files = vec![
        ("proof.c".to_string(), c),
        ("arduino.h".to_string(), ARDUINO_H.to_string()),
        ("board/stdint.h".to_string(), STDINT_H.to_string()),
        ("board/stdbool.h".to_string(), STDBOOL_H.to_string()),
        (format!("model/{}", model.header_name), model.header.to_string()),
        (format!("model/{}", model.code_name), model.code.to_string()),
    ];
    if correspondence.binary.is_some() {
        files.push(("sketch/arclang_lifted.h".to_string(), super::binary::LIFTED_H.to_string()));
    }
    files.extend(sketches.iter().map(|(name, text)| (format!("sketch/{name}"), if watched_names.is_empty() { text.clone() } else { super::interleave::watched(text, &watched_names) })));
    // The loops of the model run over its states or its queue; those of the code are unwound as far.
    let bound = Regex::new(r"(?m)^#define \w+_(?:STATE_COUNT|QUEUE_SIZE) (\d+)U").expect("a pattern of the harness");
    let unwind = bound.captures_iter(model.header).filter_map(|b| b[1].parse::<usize>().ok()).max().unwrap_or(1).max(16) + 2;
    // What no line speaks of: the proof holds, and says nothing of it.
    let related: Vec<&str> = correspondence.writes.iter().chain(&correspondence.state).filter_map(|(_, model_side)| if let Term::Variable(name) = model_side { Some(name.as_str()) } else { None }).chain(correspondence.pwm.iter().map(|(_, name)| name.as_str())).collect();
    let unrelated = machine.variables.iter().filter(|v| !v.input && !related.contains(&v.name.as_str())).map(|v| v.name.clone()).collect();
    // The start of a binary runs its own initialisation: it copies and clears its memory byte by byte.
    let start_unwind = if correspondence.binary.is_some() { BINARY_START_UNWIND } else { unwind };
    Ok(Proof { files, unwind, start_unwind, period_us, lines: lines.into_iter().map(|(_, written)| written).collect(), unrelated, define, shown: shown.into_iter().map(|(code, _, model, _)| (code, model)).collect(), inputs: inputs_shown.into_iter().map(|(name, _)| name).collect(), routines: routine_names, middle_unasked: kept.err() })
}

/// The loops of a binary's start copy and clear at most the memory of the chip (2 KiB).
const BINARY_START_UNWIND: usize = 2100;

/// The registers of a processor are cells of small arrays: the prover follows each cell alone.
const BINARY_FLAGS: [&str; 2] = ["--max-field-sensitivity-array-size", "256"];

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
    /// The bound of the loops of the start, which for a binary is that of its memory.
    pub start_unwind: usize,
    pub period_us: u64,
    pub lines: Vec<String>,
    /// The variables of the machine, inputs apart, that no line relates.
    pub unrelated: Vec<String>,
    /// The period of the machine's step, given to its C.
    pub define: String,
    /// What a refuting case shows of each line: (the code's side, the model's side), as written.
    pub shown: Vec<(String, String)>,
    /// The inputs a refuting case gives, by name.
    pub inputs: Vec<String>,
    /// The routines of the interrupts the correspondence names.
    pub routines: Vec<String>,
    /// Why the interrupt in the middle of loop() cannot be asked, when it cannot.
    pub middle_unasked: Option<String>,
}

/// The case that refutes a proof: the inputs of the step, and both sides of
/// each line before the step (none for the start) and after it.
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize)]
pub struct Case {
    pub inputs: Vec<(String, i64)>,
    /// (the code's side, its value before and after; the model's side, its value before and after)
    pub lines: Vec<CaseLine>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CaseLine {
    pub code: String,
    pub code_before: Option<i64>,
    pub code_after: Option<i64>,
    pub model: String,
    pub model_before: Option<i64>,
    pub model_after: Option<i64>,
}

impl Case {
    /// The case as lines of text, the sides that differ after the step marked.
    pub fn said(&self) -> Vec<String> {
        let value = |v: Option<i64>| v.map(|v| if v == -1 { "-".to_string() } else { v.to_string() }).unwrap_or_else(|| "?".to_string());
        let mut text = Vec::new();
        if !self.inputs.is_empty() {
            text.push(format!("with {}", self.inputs.iter().map(|(name, v)| format!("{name} = {v}")).collect::<Vec<_>>().join(", ")));
        }
        for line in &self.lines {
            let before = if line.code_before.is_some() || line.model_before.is_some() { format!("before {} and {}, ", value(line.code_before), value(line.model_before)) } else { String::new() };
            let differ = line.code_after != line.model_after && line.model_after != Some(-1);
            text.push(format!("`{}` and `{}`: {before}after {} and {}{}", line.code, line.model, value(line.code_after), value(line.model_after), if differ { "  <- differ" } else { "" }));
        }
        text
    }
}

/// Ask CBMC itself for a case that fails the obligation, and read in its
/// trace what the harness recorded. None when it gives none in time: the
/// refutation stands without it.
fn case(dir: &Path, sources: &[&str], flags: &[&str], unwind: usize, proof: &Proof) -> Option<Case> {
    use std::process::{Command, Stdio};
    let cbmc = std::env::var("ARCLANG_CBMC").ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| "cbmc".to_string());
    let trace = dir.join("case.txt");
    let file = std::fs::File::create(&trace).ok()?;
    let mut child = Command::new(cbmc).args(sources).args(flags).args(["-DARCLANG_WITNESS", "--no-standard-checks", "--unwind", &unwind.to_string(), "--no-unwinding-assertions", "--trace"]).current_dir(dir).stdout(file).stderr(Stdio::null()).spawn().ok()?;
    let started = std::time::Instant::now();
    loop {
        match child.try_wait().ok()? {
            Some(_) => break,
            None if started.elapsed().as_secs() > 60 => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            None => std::thread::sleep(std::time::Duration::from_millis(50)),
        }
    }
    let text = std::fs::read_to_string(&trace).ok()?;
    let recorded = Regex::new(r"(?m)^  arclang_w(i|bc|bm|ac|am)_(\d+)=(-?\d+)").expect("a pattern of the case");
    let mut values = std::collections::BTreeMap::new();
    for found in recorded.captures_iter(&text) {
        values.insert((found[1].to_string(), found[2].parse::<usize>().ok()?), found[3].parse::<i64>().ok()?);
    }
    if values.is_empty() {
        return None;
    }
    let at = |kind: &str, rank: usize| values.get(&(kind.to_string(), rank)).copied();
    Some(Case {
        inputs: proof.inputs.iter().enumerate().filter_map(|(rank, name)| Some((name.clone(), at("i", rank)?))).collect(),
        lines: proof.shown.iter().enumerate().map(|(rank, (code, model))| CaseLine { code: code.clone(), code_before: at("bc", rank), code_after: at("ac", rank), model: model.clone(), model_before: at("bm", rank), model_after: at("am", rank) }).collect(),
    })
}

fn said(term: &Term) -> String {
    match term {
        Term::Variable(name) => name.clone(),
        Term::In(state) => format!("in {state}"),
        Term::TimeIn(state) => format!("time in {state}"),
    }
}

/// The C of the model's machines, by file name: (the header, the source), and the period it steps at by default.
fn machines_c(ast: &Model) -> Result<((String, String), (String, String), u64), String> {
    let (files, _) = crate::compiler::embedded::export(ast).map_err(|error| format!("the model generates no C to prove the code against: {error}"))?;
    let header = files.iter().find(|(name, text)| name.ends_with(".h") && !name.contains('/') && text.contains("/* state_machine \"")).ok_or("the model generates no header of machines")?.clone();
    let code_name = header.0.replace(".h", ".c");
    let code = files.iter().find(|(name, _)| *name == code_name).ok_or("the model generates no source of machines")?.clone();
    let default_period = Regex::new(r"(?m)^#define \w+_PERIOD_US (\d+)UL$").expect("a pattern of the proof").captures(&header.1).and_then(|p| p[1].parse::<u64>().ok()).unwrap_or(10_000);
    Ok((header, code, default_period))
}

/// What the compiler checks of the correspondences of a model, without the
/// code they name: each is well formed, and the machine, the states and the
/// variables it speaks of are those of the model. Whether it holds is the
/// proof's to say (`arclang prove`).
pub fn check(ast: &Model) -> Result<(), String> {
    if ast.correspondences.is_empty() {
        return Ok(());
    }
    let mut names = std::collections::BTreeSet::new();
    for declaration in &ast.correspondences {
        if !names.insert(declaration.name.as_str()) {
            return Err(format!("correspondence '{}' is declared twice", declaration.name));
        }
    }
    let (header, code, default_period) = machines_c(ast).map_err(|error| format!("correspondence '{}': {error}", ast.correspondences[0].name))?;
    let model = Generated { header_name: &header.0, header: &header.1, code_name: &code.0, code: &code.1 };
    for declaration in &ast.correspondences {
        harness(&read(declaration)?, &model, &[], default_period)?;
    }
    Ok(())
}

/// Prove the correspondences of a model, each against the sources it names
/// (read beside the model). `work` receives the files of each proof.
pub fn prove(ast: &Model, base: &Path, only: Option<&str>, work: &Path, seconds: u64) -> Result<Vec<(Correspondence, Outcome)>, String> {
    if ast.correspondences.is_empty() {
        return Err("the model declares no correspondence (`correspondence \"Name\" { machine: ... source: ... }`)".to_string());
    }
    let (header, code, default_period) = machines_c(ast)?;
    let model = Generated { header_name: &header.0, header: &header.1, code_name: &code.0, code: &code.1 };
    let mut outcomes = Vec::new();
    for declaration in ast.correspondences.iter().filter(|d| only.is_none_or(|name| d.name == name)) {
        let mut correspondence = read(declaration)?;
        let mut sketches = Vec::new();
        let mut machine_code = None;
        let mut vectors = None;
        for source in &correspondence.sources {
            let path = base.join(source);
            let text = std::fs::read_to_string(&path).map_err(|e| format!("correspondence '{}': cannot read {}: {e}", correspondence.name, path.display()))?;
            let name = Path::new(source).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| source.clone());
            sketches.push((name, text));
        }
        let dir = work.join(correspondence.name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>());
        if let Some(binary) = &correspondence.binary {
            let what = format!("correspondence '{}'", correspondence.name);
            let path = base.join(&binary.file);
            let text = std::fs::read_to_string(&path).map_err(|e| format!("{what}: cannot read {}: {e}", path.display()))?;
            let image = super::binary::intel_hex(&text).map_err(|e| format!("{what}: {}: {e}", binary.file))?;
            let mut entries: Vec<u32> = binary.functions.iter().map(|(_, at)| *at).collect();
            // The routine of the overflow of timer 0 is translated too: the clock is proven of it.
            vectors = super::binary::vectors(&image);
            if let Some(routine) = vectors.as_ref().map(|vectors| vectors[super::binary::TIMER0_OVERFLOW]) {
                if !entries.contains(&routine) {
                    entries.push(routine);
                }
            }
            let lifted = super::binary::lift(&image, &entries, &dir.join("lift")).map_err(|e| format!("{what}: {e}"))?;
            sketches.push(("machine_code.c".to_string(), super::binary::sketch(binary, &lifted, &what)?));
            machine_code = Some(lifted);
        }
        let written = harness(&correspondence, &model, &sketches, default_period)?;
        let Proof { files: proof, unwind, start_unwind, period_us, lines, unrelated, define, .. } = &written;
        let (step_unwind, start_unwind, period_us, unrelated) = (*unwind, *start_unwind, *period_us, unrelated.clone());
        for (name, text) in proof {
            let path = dir.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
            }
            std::fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        }
        let started = std::time::Instant::now();
        let model_source = format!("model/{}", model.code_name);
        let mut outcome = None;
        // The functions a binary's correspondence names are not run in its proof: first, that each is
        // the function it is said to be, of its own machine code (those whose identity is proven yet).
        if let (Some(binary), Some(lifted)) = (correspondence.binary.as_mut(), &machine_code) {
            let mut not_them = Vec::new();
            for (name, at) in binary.functions.clone().iter().filter(|(name, _)| name != "loop") {
                match super::binary_identity::check(name, *at, lifted, &dir, seconds)? {
                    super::binary_identity::Identity::Proven(proven) => binary.identified.push((name.clone(), proven)),
                    super::binary_identity::Identity::Refuted(parts) => not_them.extend(parts.iter().map(|part| format!("{name} at 0x{at:04x}: {part}"))),
                    super::binary_identity::Identity::Unknown(reason) => {
                        outcome = Some(Outcome::Unknown { reason: format!("whether the function at 0x{at:04x} is {name}: {reason}") });
                        break;
                    }
                    super::binary_identity::Identity::NotAsked => {}
                }
            }
            if !not_them.is_empty() {
                outcome = Some(Outcome::Refuted { obligation: "functions".to_string(), lines: not_them, case: None, seconds: started.elapsed().as_secs_f64() });
            }
            // The clock: what millis returns is what the interrupt of timer 0 keeps, as the start sets it.
            let read_at = |function: &str| binary.identified.iter().find(|(name, _)| name == function).and_then(|(_, proven)| proven.split("the memory at 0x").nth(1)).and_then(|at| u32::from_str_radix(&at[..4.min(at.len())], 16).ok());
            let (count_at, turns_at) = (read_at("millis"), read_at("micros"));
            if let (None, Some(vectors), true) = (&outcome, &vectors, count_at.is_some() || turns_at.is_some()) {
                let (start, functions) = super::binary_shape::facts(lifted)?;
                let named: std::collections::BTreeSet<u32> = binary.functions.iter().filter(|(name, _)| name != "loop").map(|(_, at)| *at).collect();
                let loop_at = binary.functions.iter().find(|(name, _)| name == "loop").map(|(_, at)| *at).unwrap_or_default();
                let loop_code = super::binary_shape::code_of(loop_at, &named, start, &functions);
                let clock = super::binary_identity::clock(vectors, start, loop_at, &loop_code, count_at, turns_at, lifted, &dir, seconds)?;
                if !clock.refuted.is_empty() {
                    outcome = Some(Outcome::Refuted { obligation: "clock".to_string(), lines: clock.refuted, case: None, seconds: started.elapsed().as_secs_f64() });
                } else if clock.unknown.is_none() {
                    binary.clock = clock.proven;
                    binary.interrupts_settled = clock.interrupts_settled;
                }
            }
        }
        // The start is of the code as it is initialised; the step, of any state it can hold.
        let mut unwind = step_unwind;
        for (obligation, mut flags) in [("start", vec!["-DARCLANG_START"]), ("step", vec!["--nondet-static"])] {
            if outcome.is_some() {
                break;
            }
            unwind = if obligation == "start" { start_unwind } else { step_unwind };
            flags.push(define.as_str());
            // The widths of the board, and its fixed-width types before any the host has.
            flags.extend(["-I", "board"]);
            flags.extend(if correspondence.int_bits == 16 { ["--16", "-DARCLANG_INT16"] } else { ["--32", "-DARCLANG_INT32"] });
            if correspondence.binary.is_some() {
                flags.extend(BINARY_FLAGS);
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
                    let shown = case(&dir, &["proof.c", model_source.as_str()], &flags, unwind, &written);
                    outcome = Some(Outcome::Refuted { obligation: obligation.to_string(), lines: failing, case: shown, seconds: started.elapsed().as_secs_f64() });
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
        // Proven on the clock it states: and on the clock of the board?
        let board_clock = if outcome.is_none() && int_bits == 16 {
            let mut flags = vec!["--nondet-static", define.as_str(), "-I", "board", "--16", "-DARCLANG_INT16", "-DARCLANG_BOARD_CLOCK"];
            if correspondence.binary.is_some() {
                flags.extend(BINARY_FLAGS);
            }
            Some(match decide(&dir, &["proof.c", model_source.as_str()], &flags, unwind, seconds)? {
                Answer::Unsat => BoardClock::Holds,
                Answer::Sat(_) => {
                    let mut failing = Vec::new();
                    for (rank, written) in lines.iter().enumerate() {
                        let only = format!("-DARCLANG_LINE={rank}");
                        let mut asked = flags.clone();
                        asked.push(only.as_str());
                        if let Answer::Sat(_) = decide(&dir, &["proof.c", model_source.as_str()], &asked, unwind, seconds)? {
                            failing.push(written.clone());
                        }
                    }
                    BoardClock::Fails { lines: failing, case: case(&dir, &["proof.c", model_source.as_str()], &flags, unwind, &written) }
                }
                Answer::BeyondBound => BoardClock::Unknown { reason: format!("a loop of the code or of the model runs more than {unwind} times") },
                Answer::Unknown(reason) => BoardClock::Unknown { reason },
            })
        } else {
            None
        };
        // Proven for interrupts between two loop()s: and one in the middle?
        let interrupts = if written.routines.is_empty() {
            None
        } else {
            let shared = super::interleave::used_with_interrupts_on(&sketches, &written.routines);
            let mut middle = Middle::Same;
            if let Some(reason) = &written.middle_unasked {
                middle = Middle::Unknown { reason: reason.clone() };
            } else if outcome.is_none() {
                for routine in &written.routines {
                    let named = format!("-DARCLANG_ROUTINE={routine}");
                    let width = if int_bits == 16 { ["--16", "-DARCLANG_INT16"] } else { ["--32", "-DARCLANG_INT32"] };
                    let flags = vec!["--nondet-static", define.as_str(), "-I", "board", width[0], width[1], "-DARCLANG_INTERLEAVE", named.as_str()];
                    match decide(&dir, &["proof.c", model_source.as_str()], &flags, unwind, seconds)? {
                        Answer::Unsat => {}
                        Answer::Sat(_) => {
                            middle = Middle::Differs { routine: routine.clone() };
                            break;
                        }
                        Answer::BeyondBound => {
                            middle = Middle::Unknown { reason: format!("a loop of the code runs more than {unwind} times") };
                            break;
                        }
                        Answer::Unknown(reason) => {
                            middle = Middle::Unknown { reason };
                            break;
                        }
                    }
                }
            }
            Some(Interrupts { shared, middle })
        };
        outcomes.push((correspondence, outcome.unwrap_or(Outcome::Proven { seconds: started.elapsed().as_secs_f64(), period_us, int_bits, unrelated, board_clock, interrupts })));
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
    fn an_edge_taken_by_an_interrupt_is_an_event() {
        let read = read(&declaration(&[("machine", text("M")), ("source", text("s.ino")), ("events", list(&["RISING on SENSOR is pulse"])), ("state", list(&["x == 1 is in A"]))])).unwrap();
        assert_eq!(read.events, [("RISING".to_string(), "SENSOR".to_string(), "pulse".to_string())]);
        let refused = super::read(&declaration(&[("machine", text("M")), ("source", text("s.ino")), ("events", list(&["HIGH on SENSOR is pulse"]))])).unwrap_err();
        assert!(refused.contains("is not an edge taken by an interrupt"), "{refused}");
    }

    #[test]
    fn an_analog_write_is_a_variable_of_the_machine() {
        let read = read(&declaration(&[("machine", text("M")), ("source", text("s.ino")), ("writes", list(&["analogWrite(LAMP) is duty", "digitalWrite(13) is beat"]))])).unwrap();
        assert_eq!(read.pwm, [("LAMP".to_string(), "duty".to_string())]);
        assert_eq!(read.writes, [("13".to_string(), Term::Variable("beat".to_string()))]);
        let refused = super::read(&declaration(&[("machine", text("M")), ("source", text("s.ino")), ("writes", list(&["analogWrite(LAMP) is in Lit"]))])).unwrap_err();
        assert!(refused.contains("what analogWrite writes is a variable of the machine"), "{refused}");
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
