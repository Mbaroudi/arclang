//! Reverse engineering: the model a piece of code implements.
//!
//! This first reader takes the C the compiler itself generates (`arclang
//! export -f c`): the header and the source of a model's state machines. It
//! gives back the machines as ArcLang, and says of each how it knows: what it
//! read in the code (the states and their order, where a machine starts, the
//! source and target of each transition, its guard, its timing, the initial
//! values, the wiring between machines) and what it could only read in a
//! comment the generator wrote (the names as the model writes them, the
//! units, the text of an equation or of an action).
//!
//! A comment can say anything. The proof is `check`: the recovered model is
//! compiled and its C generated again; where it is the C that was read, the
//! recovered machine is the one the code implements, comments included.
//! Where it differs, the difference is reported, never papered over.
//!
//! What the generated C does not hold cannot be recovered and is listed in
//! the notes: the name of a region, whether a variable is `out` or `const`,
//! the range of a variable, the priority of a transition. A region is recovered with its
//! states and where it starts; its name is not in the code, and it is named
//! by its rank. A transition through a pseudo-state (choice, junction, fork,
//! join, history, terminate) is read in the body of its function (`fire.rs`).

pub mod binary;
pub mod binary_shape;
mod c;
pub mod correspondence;
pub mod devicetree;
pub mod interleave;
pub mod drift;
pub mod equivalence;
mod fire;
pub mod logic;
pub mod logic_equivalence;
pub mod observe;
mod write;

use std::collections::BTreeMap;

/// How an element of the recovered model is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Read in the statements of the code.
    Code,
    /// Read in a comment of the generator: to be confirmed by `check`.
    Comment,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    /// Its name in the C: the field is `v_<ident>`.
    pub ident: String,
    pub ty: &'static str,
    pub unit: Option<String>,
    pub input: bool,
    /// The initial value, as the C writes it, and the line that sets it.
    pub initial: Option<String>,
    pub initial_line: Option<usize>,
    /// `from "Machine".variable`, for an input read from another machine.
    pub from: Option<(String, String)>,
    /// How its word is sized, when the machine is computed by programmable logic.
    pub sized: Option<logic::Sized>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct State {
    /// The path of the state from the machine (`Charging.Precharge`).
    pub path: String,
    pub entry: Vec<String>,
    pub exit: Vec<String>,
    pub activity: Vec<String>,
    /// The sub-state entered with it, when it has some.
    pub initial: Option<String>,
    /// The kind of pseudo-state it is (`choice`, `deep_history`...), when it is one.
    pub pseudo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Transition {
    pub from: usize,
    pub to: usize,
    pub trigger: Option<String>,
    pub guard: Option<String>,
    /// The line of the condition the guard was read in.
    pub guard_line: Option<usize>,
    /// The time after which it fires, in microseconds.
    pub after_us: Option<u64>,
    /// The time it has to leave its source once it may, in microseconds.
    pub budget_us: Option<u64>,
    /// That time as a comment of the generator spells it (`10 min`), kept
    /// when it is the time the code counts.
    pub time_text: Option<String>,
    pub actions: Vec<String>,
    /// Its `id`, when the code names it by one.
    pub id: Option<String>,
    /// The first and the last state it exits.
    pub exits: Option<(usize, usize)>,
    /// Where it is written: in the state `owner` (`None`: in the machine),
    /// and in which of its regions (`None`: beside them, or there is none).
    pub owner: Option<usize>,
    pub region: Option<usize>,
    pub line: usize,
}

/// A region of a machine or of a state: the states that run beside those of
/// its sibling regions. Its extent is read in the code (what a transition of
/// the region exits); its name is not in the code.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    /// The state that holds it; `None` for a region of the machine.
    pub owner: Option<usize>,
    /// The first and the last state of the region, by index.
    pub first: usize,
    pub last: usize,
    /// The state the region starts in.
    pub initial: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Machine {
    pub name: String,
    /// The identifier of the machine in the C (`overspeed_warning`).
    pub ident: String,
    /// The machine is computed by programmable logic (a register map says so).
    pub in_logic: bool,
    pub variables: Vec<Variable>,
    pub states: Vec<State>,
    pub initial: Option<usize>,
    pub transitions: Vec<Transition>,
    /// The regions, in the order of their states.
    pub regions: Vec<Region>,
    /// The equations, as the generator's comments write them.
    pub equations: Vec<String>,
    /// The statement that computes each equation, with its line.
    pub equation_code: Vec<(usize, String)>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Recovered {
    pub model: String,
    pub machines: Vec<Machine>,
    /// What was not recovered, and why.
    pub notes: Vec<String>,
}

/// The header and the source the generator wrote for a model, by file name.
pub struct Generated<'a> {
    pub header_name: &'a str,
    pub header: &'a str,
    pub code_name: &'a str,
    pub code: &'a str,
}

/// Read the machines of a model back from its generated C.
pub fn from_generated_c(generated: &Generated) -> Result<Recovered, String> {
    c::read(generated)
}

/// The recovered model as ArcLang source, each machine with where it was read.
pub fn to_arclang(recovered: &Recovered, generated: &Generated) -> String {
    write::model(recovered, generated)
}

/// One difference between the C that was read and the C the recovered model generates.
#[derive(Debug, Clone, PartialEq)]
pub struct Difference {
    pub file: String,
    /// 1-based line of the C that was read.
    pub line: usize,
    pub read: String,
    pub regenerated: String,
}

/// The outcome of `check`: whether the recovered model generates the code it was read from.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Check {
    /// The first differences of each file (empty: the files are the same).
    pub differences: Vec<Difference>,
    /// How many lines differ in all.
    pub differing_lines: usize,
    /// A step period the code was generated with that the recovered model
    /// does not give: (the macro, the value read, the value regenerated).
    /// The period comes from the `io_binding` of the model, which the
    /// machines' C does not hold; it is read, not proven.
    pub periods: Vec<(String, String, String)>,
    /// The recovered model does not compile, or generates no C: why.
    pub error: Option<String>,
}

impl Check {
    pub fn proven(&self) -> bool {
        self.error.is_none() && self.differing_lines == 0
    }
}

/// Compile the recovered source, generate its C and compare it with the C
/// that was read. Equal files prove the recovery; every difference is a
/// place where the recovered model is not the code.
pub fn check(source: &str, generated: &Generated) -> Check {
    check_showing(source, generated, 5)
}

/// As `check`, with at most `shown` differences of each file kept (their count is always whole).
pub fn check_showing(source: &str, generated: &Generated, shown_at_most: usize) -> Check {
    let compiled = match crate::Compiler::new(crate::CompilerConfig::default()).compile_string(source) {
        Ok(compiled) => compiled,
        Err(error) => return Check { error: Some(format!("the recovered model does not compile: {error}")), ..Check::default() },
    };
    let files = match crate::compiler::embedded::export(&compiled.ast) {
        Ok((files, _)) => files,
        Err(error) => return Check { error: Some(format!("the recovered model generates no C: {error}")), ..Check::default() },
    };
    let regenerated: BTreeMap<&str, &str> = files.iter().map(|(name, text)| (name.as_str(), text.as_str())).collect();
    let mut outcome = Check::default();
    let period = regex::Regex::new(r"^#define (\w+_PERIOD_US) (\d+)UL$").expect("a pattern of the check");
    for (name, read) in [(generated.header_name, generated.header), (generated.code_name, generated.code)] {
        let Some(again) = regenerated.get(name) else {
            outcome.error = Some(format!("the recovered model generates no {name}: its name is not the one the code was generated from"));
            return outcome;
        };
        let (read_lines, again_lines): (Vec<&str>, Vec<&str>) = (read.lines().collect(), again.lines().collect());
        let mut shown = 0;
        for at in 0..read_lines.len().max(again_lines.len()) {
            let (left, right) = (read_lines.get(at).copied().unwrap_or(""), again_lines.get(at).copied().unwrap_or(""));
            if left != right {
                if let (Some(read), Some(again)) = (period.captures(left), period.captures(right)) {
                    if read[1] == again[1] {
                        outcome.periods.push((read[1].to_string(), read[2].to_string(), again[2].to_string()));
                        continue;
                    }
                }
                outcome.differing_lines += 1;
                if shown < shown_at_most {
                    shown += 1;
                    outcome.differences.push(Difference { file: name.to_string(), line: at + 1, read: left.trim().to_string(), regenerated: right.trim().to_string() });
                }
            }
        }
    }
    outcome
}
