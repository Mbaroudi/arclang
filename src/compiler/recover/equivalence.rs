//! Formal equivalence of the C as built with the C the model generates.
//!
//! `drift` says where two codes differ; it cannot say that two codes written
//! differently do the same. This does: one step of the code as built and one
//! step of the model's code are run from the same state, any state, with the
//! same inputs, any inputs, and the states and the outputs after the step
//! are compared, field by field. If no state and no inputs make one field
//! differ, each step of one is a step of the other, and so is every run.
//!
//! The question is put to tools made for it: CBMC translates the two C
//! programs and the comparison to one formula (every loop unwound to its
//! bound, which is checked), and Bitwuzla decides it. An answer `unsat` is
//! the proof; `sat` comes with the state and the inputs that tell the two
//! codes apart; anything else is reported as no answer, never as a proof.
//!
//! The two codes must have the same interface (the header of the machines):
//! the comparison is of the structures it declares.

use super::Generated;
use regex::Regex;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

/// The outcome of a proof attempt.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum Equivalence {
    /// For every state and every inputs, one step gives the same state and outputs.
    Proven { comparisons: usize, seconds: f64 },
    /// A state and inputs from which one step differs: the fields of the
    /// inputs and of the starting state that the tools give a value for.
    Different { witness: Vec<(String, String)>, seconds: f64 },
    /// No answer: why (the time limit, a tool that failed).
    Unknown { reason: String },
}

/// The fields of each structure of the header, in order: (type, name, array length).
type Structures = BTreeMap<String, Vec<(String, String, Option<String>)>>;

fn structures(header: &str) -> Structures {
    let block = Regex::new(r"(?s)typedef struct \{\n(.*?)\n\} (\w+);").expect("a pattern of the harness");
    let field = Regex::new(r"^\s*(\w+) (\w+)(?:\[(\w+)\])?;").expect("a pattern of the harness");
    block
        .captures_iter(header)
        .map(|found| {
            let fields = found[1].lines().filter_map(|line| field.captures(line)).map(|f| (f[1].to_string(), f[2].to_string(), f.get(3).map(|n| n.as_str().to_string()))).collect();
            (found[2].to_string(), fields)
        })
        .collect()
}

/// The files of the proof: the two codes, each with its public functions
/// renamed so that both link in one program, and the harness that runs one
/// step of each and compares.
pub struct Harness {
    pub files: Vec<(String, String)>,
    /// How many fields are compared.
    pub comparisons: usize,
    /// The bound every loop is unwound to (and checked not to exceed).
    pub unwind: usize,
}

/// Build the files of the proof for the model's `generated` code and the code as `built`.
pub fn harness(generated: &Generated, built: &Generated) -> Result<Harness, String> {
    if generated.header != built.header {
        return Err(format!(
            "{} as built is not the header the model generates: the two codes do not have the same structures, and a step of one cannot be compared with a step of the other (`arclang drift` lists the differences)",
            built.header_name
        ));
    }
    let prefix = generated.header_name.trim_end_matches(".h");
    let known = structures(generated.header);
    for needed in ["state", "inputs", "outputs"] {
        if !known.contains_key(&format!("{prefix}_{needed}")) {
            return Err(format!("{} declares no `{prefix}_{needed}`: it is not the header of a model's machines", generated.header_name));
        }
    }
    let mut lines: Vec<String> = Vec::new();
    compare(&known, &format!("{prefix}_state"), "s1", "s2", "state", 0, &mut lines);
    compare(&known, &format!("{prefix}_outputs"), "o1", "o2", "outputs", 0, &mut lines);
    let comparisons = lines.iter().filter(|line| line.contains("__CPROVER_assert")).count();

    let mut text = String::new();
    text.push_str("/* One step of the code as built against one step of the model's code, from\n   any state and for any inputs: every field of the state and of the outputs\n   must be the same after it. Written by `arclang drift --prove`. */\n");
    text.push_str(&format!("#include \"gen/{}\"\n", generated.header_name));
    text.push_str("/* The same bits: a value that is not a number is the same as itself, and -0 is not +0. */\n");
    text.push_str("static _Bool same_f64(double a, double b) { union { double d; unsigned long long u; } x, y; x.d = a; y.d = b; return x.u == y.u; }\n");
    for side in ["built", "gen"] {
        text.push_str(&format!("void {side}_{prefix}_step({prefix}_state *state, const {prefix}_inputs *inputs, {prefix}_outputs *outputs);\n"));
    }
    text.push_str(&format!("int main(void)\n{{\n    {prefix}_state s1, s2;\n    {prefix}_inputs in;\n    {prefix}_outputs o1 = {{0}}, o2 = {{0}};\n"));
    // A state the structure can hold but no machine ever has is not one to
    // start from: a history names a state of its machine, or none.
    let mut valid: Vec<String> = Vec::new();
    histories(&known, &format!("{prefix}_state"), "s1", prefix, &mut valid);
    for line in &valid {
        text.push_str(&format!("    {line}\n"));
    }
    text.push_str(&format!("    s2 = s1;\n    built_{prefix}_step(&s1, &in, &o1);\n    gen_{prefix}_step(&s2, &in, &o2);\n"));
    for line in &lines {
        text.push_str(&format!("    {line}\n"));
    }
    text.push_str("    return 0;\n}\n");

    // Each code in a file of its own, its public functions renamed.
    let public = Regex::new(r"(?m)^void (\w+)\(").expect("a pattern of the harness");
    let functions: Vec<String> = public.captures_iter(generated.header).map(|f| f[1].to_string()).collect();
    let wrapper = |side: &str, code_name: &str| {
        let mut out = String::new();
        for function in &functions {
            out.push_str(&format!("#define {function} {side}_{function}\n"));
        }
        out.push_str(&format!("#include \"{side}/{code_name}\"\n"));
        out
    };
    // Every loop of the generated code runs over the states or the queue of a machine.
    let bound = Regex::new(r"(?m)^#define \w+_(?:STATE_COUNT|QUEUE_SIZE) (\d+)U").expect("a pattern of the harness");
    let longest = bound.captures_iter(generated.header).filter_map(|b| b[1].parse::<usize>().ok()).max().unwrap_or(1);
    Ok(Harness {
        files: vec![
            ("harness.c".to_string(), text),
            ("gen.c".to_string(), wrapper("gen", generated.code_name)),
            ("built.c".to_string(), wrapper("built", built.code_name)),
            (format!("gen/{}", generated.header_name), generated.header.to_string()),
            (format!("gen/{}", generated.code_name), generated.code.to_string()),
            (format!("built/{}", built.header_name), built.header.to_string()),
            (format!("built/{}", built.code_name), built.code.to_string()),
        ],
        comparisons,
        unwind: longest + 2,
    })
}

/// What a starting state must hold to be one of a machine: each shallow
/// history (`hist_<n>` of the state structure of machine `<ident>`) is a
/// state of that machine or `NO_STATE`.
fn histories(known: &Structures, ty: &str, value: &str, prefix: &str, lines: &mut Vec<String>) {
    let Some(fields) = known.get(ty) else { return };
    for (field_ty, name, length) in fields {
        if length.is_some() {
            continue;
        }
        let is_history = field_ty == "uint16_t" && name.strip_prefix("hist_").is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
        match ty.strip_prefix(&format!("{prefix}_")).and_then(|rest| rest.strip_suffix("_state")) {
            Some(machine) if is_history => {
                let count = format!("{}_{}_STATE_COUNT", prefix.to_uppercase(), machine.to_uppercase());
                lines.push(format!("__CPROVER_assume({value}.{name} == 0xFFFFU || {value}.{name} < {count});"));
            }
            _ => histories(known, field_ty, &format!("{value}.{name}"), prefix, lines),
        }
    }
}

/// The comparisons of one structure, field by field, structures and arrays opened.
fn compare(known: &Structures, ty: &str, left: &str, right: &str, said: &str, depth: usize, lines: &mut Vec<String>) {
    match known.get(ty) {
        Some(fields) => {
            for (field_ty, name, length) in fields {
                match length {
                    Some(length) => {
                        let k = format!("k{depth}");
                        lines.push(format!("for (unsigned {k} = 0; {k} < {length}; {k}++) {{"));
                        compare(known, field_ty, &format!("{left}.{name}[{k}]"), &format!("{right}.{name}[{k}]"), &format!("{said}.{name}[]"), depth + 1, lines);
                        lines.push("}".to_string());
                    }
                    None => compare(known, field_ty, &format!("{left}.{name}"), &format!("{right}.{name}"), &format!("{said}.{name}"), depth, lines),
                }
            }
        }
        None if ty == "double" => lines.push(format!("__CPROVER_assert(same_f64({left}, {right}), \"{said} is the same\");")),
        None => lines.push(format!("__CPROVER_assert({left} == {right}, \"{said} is the same\");")),
    }
}

/// The tools, as the environment names them or as the path has them.
fn tool(variable: &str, default: &str) -> String {
    std::env::var(variable).ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| default.to_string())
}

/// Prove, or refute, that one step of the code as built is one step of the
/// model's code. `work` receives the files of the proof and is kept.
pub fn prove(generated: &Generated, built: &Generated, work: &Path, seconds: u64) -> Result<Equivalence, String> {
    let harness = harness(generated, built)?;
    for (name, text) in &harness.files {
        let path = work.join(name);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
        }
        std::fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }
    let started = std::time::Instant::now();
    let how = "run in ArcLang's formal image: tools/embedded/formal/run.sh";
    // CBMC: the two programs and the comparison as one formula.
    let cbmc = tool("ARCLANG_CBMC", "cbmc");
    let unwind = harness.unwind.to_string();
    let translated = Command::new(&cbmc)
        .args(["harness.c", "gen.c", "built.c", "--no-standard-checks", "--unwind", &unwind, "--unwinding-assertions", "--smt2", "--outfile", "equivalence.smt2"])
        .current_dir(work)
        .output()
        .map_err(|e| format!("cannot run '{cbmc}': {e} ({how})"))?;
    if !work.join("equivalence.smt2").is_file() {
        let said = format!("{}{}", String::from_utf8_lossy(&translated.stdout), String::from_utf8_lossy(&translated.stderr));
        let tail: Vec<&str> = said.lines().filter(|l| !l.trim().is_empty()).rev().take(8).collect();
        return Ok(Equivalence::Unknown { reason: format!("CBMC could not translate the code as built:\n{}", tail.into_iter().rev().collect::<Vec<_>>().join("\n")) });
    }
    // Bitwuzla: is there a state and inputs for which a comparison fails?
    let bitwuzla = tool("ARCLANG_BITWUZLA", "bitwuzla");
    let limit = format!("{}", seconds.saturating_mul(1000));
    let answer = Command::new(&bitwuzla).args(["-t", &limit, "equivalence.smt2"]).current_dir(work).output().map_err(|e| format!("cannot run '{bitwuzla}': {e} ({how})"))?;
    let said = String::from_utf8_lossy(&answer.stdout).into_owned();
    let elapsed = started.elapsed().as_secs_f64();
    match said.lines().next().map(str::trim) {
        Some("unsat") => Ok(Equivalence::Proven { comparisons: harness.comparisons, seconds: elapsed }),
        Some("sat") => {
            // A comparison that fails, or a loop that runs past its bound from
            // a state no machine has? The same question without the bound's check.
            let _ = std::fs::remove_file(work.join("equivalence.smt2"));
            let _ = Command::new(&cbmc)
                .args(["harness.c", "gen.c", "built.c", "--no-standard-checks", "--unwind", &unwind, "--no-unwinding-assertions", "--smt2", "--outfile", "equivalence.smt2"])
                .current_dir(work)
                .output();
            let within = Command::new(&bitwuzla).args(["-t", &limit, "equivalence.smt2"]).current_dir(work).output().map_err(|e| format!("cannot run '{bitwuzla}': {e} ({how})"))?;
            let within_said = String::from_utf8_lossy(&within.stdout).into_owned();
            let elapsed = started.elapsed().as_secs_f64();
            match within_said.lines().next().map(str::trim) {
                Some("sat") => Ok(Equivalence::Different { witness: witness(&within_said, generated.header), seconds: elapsed }),
                Some("unsat") => Ok(Equivalence::Unknown {
                    reason: format!(
                        "the two codes agree from every state whose loops end within {} passes, but from some state a loop of one of them runs longer (a count or an index the state holds out of its range): the proof does not cover those states",
                        harness.unwind
                    ),
                }),
                _ => Ok(Equivalence::Unknown { reason: format!("no answer within {seconds} s") }),
            }
        }
        other => Ok(Equivalence::Unknown {
            reason: match other {
                Some("unknown") | None => format!("no answer within {seconds} s"),
                Some(text) => format!("Bitwuzla answered `{text}`"),
            },
        }),
    }
}

/// The inputs and the starting state of a counterexample, as the solver
/// gives them: the scalar fields, each with its value read as its C type.
fn witness(model: &str, header: &str) -> Vec<(String, String)> {
    let value = Regex::new(r"\(\(\|main::1::(in|s1)!0@1#1\.\.([\w.\[\]]+)\| #b([01]+)\)\)").expect("a pattern of the witness");
    // The C type of each field, by its name.
    let types: BTreeMap<String, String> = Regex::new(r"(?m)^\s*(\w+) (\w+)(?:\[\w+\])?;").expect("a pattern of the witness").captures_iter(header).map(|f| (f[2].to_string(), f[1].to_string())).collect();
    let mut found: Vec<(String, String)> = Vec::new();
    for line in model.lines() {
        let Some(read) = value.captures(line) else { continue };
        // `a..active[[0]]` is `a.active[0]`.
        let path = read[2].replace("..", ".").replace("[[", "[").replace("]]", "]");
        let name = path.rsplit('.').next().unwrap_or(&path).split('[').next().unwrap_or_default().to_string();
        let bits = u64::from_str_radix(&read[3], 2).unwrap_or(0);
        let text = match types.get(&name).map(String::as_str) {
            Some("double") => format!("{}", f64::from_bits(bits)),
            Some("bool") => (bits != 0).to_string(),
            Some("int32_t") => (bits as u32 as i32).to_string(),
            Some(_) => bits.to_string(),
            // Padding, or a structure as a whole: not a field.
            None => continue,
        };
        found.push((format!("{}.{path}", if &read[1] == "in" { "inputs" } else { "state" }), text));
    }
    found.sort();
    found.dedup();
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "#define M_A_STATE_COUNT 2U\n#define M_A_QUEUE_SIZE 16U\ntypedef struct {\n    double integral;\n    bool primed;\n} m_pid_state;\n\ntypedef struct {\n    uint16_t queue[M_A_QUEUE_SIZE];\n    double v_x; /* in variable \"x\" */\n} m_a_inputs;\n\ntypedef struct {\n    bool st_on;\n    double v_x;\n} m_a_outputs;\n\ntypedef struct {\n    bool active[2];\n    double v_y;\n    m_pid_state blk_0;\n} m_a_state;\n\ntypedef struct {\n    m_a_inputs a;\n} m_inputs;\n\ntypedef struct {\n    m_a_outputs a;\n} m_outputs;\n\ntypedef struct {\n    bool a_started;\n    m_a_state a;\n} m_state;\n\nvoid m_init(m_state *state);\nvoid m_a_step(m_state *state, const m_inputs *inputs, m_outputs *outputs);\nvoid m_step(m_state *state, const m_inputs *inputs, m_outputs *outputs);\n";

    #[test]
    fn every_field_of_the_state_and_of_the_outputs_is_compared_and_both_codes_link() {
        let generated = Generated { header_name: "m.h", header: HEADER, code_name: "m.c", code: "/* model */" };
        let built = Generated { header_name: "m.h", header: HEADER, code_name: "m.c", code: "/* built */" };
        let harness = harness(&generated, &built).unwrap();
        let text = &harness.files[0].1;
        // The flag, the two states of the array, the variable, the two fields of the block, the two outputs.
        assert_eq!(harness.comparisons, 7, "{text}");
        for compared in ["s1.a_started == s2.a_started", "s1.a.active[k0] == s2.a.active[k0]", "same_f64(s1.a.v_y, s2.a.v_y)", "same_f64(s1.a.blk_0.integral, s2.a.blk_0.integral)", "o1.a.st_on == o2.a.st_on", "same_f64(o1.a.v_x, o2.a.v_x)"] {
            assert!(text.contains(compared), "{compared}\n{text}");
        }
        // The longest loop is the queue: unwound to its length, and a little more to see it end.
        assert_eq!(harness.unwind, 18);
        let wrapper = &harness.files.iter().find(|(name, _)| name == "built.c").unwrap().1;
        assert!(wrapper.contains("#define m_step built_m_step") && wrapper.contains("#define m_init built_m_init") && wrapper.ends_with("#include \"built/m.c\"\n"), "{wrapper}");
    }

    #[test]
    fn a_history_of_the_starting_state_names_a_state_of_its_machine_or_none() {
        let header = HEADER.replace("    double v_y;\n", "    double v_y;\n    uint16_t hist_3;\n");
        let generated = Generated { header_name: "m.h", header: &header, code_name: "m.c", code: "" };
        let harness = harness(&generated, &generated).unwrap();
        assert!(harness.files[0].1.contains("__CPROVER_assume(s1.a.hist_3 == 0xFFFFU || s1.a.hist_3 < M_A_STATE_COUNT);"), "{}", harness.files[0].1);
    }

    #[test]
    fn two_codes_with_different_structures_are_not_compared() {
        let generated = Generated { header_name: "m.h", header: HEADER, code_name: "m.c", code: "" };
        let other = HEADER.replace("double v_y;", "double v_y;\n    double v_z;");
        let built = Generated { header_name: "m.h", header: &other, code_name: "m.c", code: "" };
        assert!(harness(&generated, &built).err().unwrap().contains("do not have the same structures"));
    }

    #[test]
    fn a_counterexample_is_read_with_the_type_of_each_field() {
        let model = "sat\n((|main::1::in!0@1#1..a..v_x| #b0100000001100000010000000000000000000000000000000000000000000000))\n((|main::1::s1!0@1#1..a_started| #b00000001))\n((|main::1::s1!0@1#1..a..active[[1]]| #b00000001))\n((|main::1::s1!0@1#1..a..$pad1| #b0000))\n((|other::x!0@1#1| #b1))\n";
        assert_eq!(
            witness(model, HEADER),
            [("inputs.a.v_x".to_string(), "130".to_string()), ("state.a.active[1]".to_string(), "true".to_string()), ("state.a_started".to_string(), "true".to_string())]
        );
    }
}
