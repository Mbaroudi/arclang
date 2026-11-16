//! A block of a binary that is one step of the machines of the model.
//!
//! A cycle relates a machine to the pins of the chip. Code generated from
//! the model is not written that way: its sketch fills a structure of
//! inputs, calls the step, and reads a structure of outputs. Built with
//! link-time optimisation, the step is no function either: it is a run of
//! instructions of `main`. A correspondence names that block, and where the
//! three structures of the generated header are in the memory:
//!
//! ```arclang
//! correspondence "Lock on the Uno" {
//!     binary: "uno_lock.hex"
//!     functions: ["cycle at 0x0fc6", "step from 0x1026 to 0x1294"]
//!     memory: ["state at 0x0439", "inputs at 0x0411", "outputs at 0x0409"]
//! }
//! ```
//!
//! The block runs from the instruction at its first address up to, and not
//! including, the one at its second. What is proven: from any memory, the
//! block leaves in the state and the outputs what one step of the model
//! leaves, bit for bit, and touches nothing else. The structures are laid
//! out as avr-gcc lays them out: no padding, the low byte first.
//!
//! What it does not say is said with it: when the block runs, and what the
//! rest of the cycle does to the state between two runs.

use regex::Regex;
use std::collections::BTreeMap;
use std::path::Path;

/// The block of a correspondence, and what its proof found of it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Block {
    /// The address of its first instruction, and of the first instruction after it.
    pub from: u32,
    pub to: u32,
    /// Where the structures of the generated header are in the memory.
    pub state: u32,
    pub inputs: u32,
    pub outputs: u32,
    /// Filled by the proof: the function that holds the block, the bytes of the three structures, the stack
    /// pointer and the registers the cycle keeps, as the start leaves them: (the register, its value).
    pub function: u32,
    pub sizes: [u32; 3],
    pub sp: u16,
    pub kept: Vec<(u8, u8)>,
}

/// The bytes under the stack pointer a block may use (what it pushes, the addresses its calls keep).
const STACK: u32 = 64;
/// The memory of the chip: its first and last address.
const RAM: (u32, u32) = (0x100, 0x8ff);
/// The loops of a step run at most as many times as a queue is long; none of it is shorter than this.
const UNWIND: usize = 20;

/// Read the block out of the `functions` and the `memory` of a correspondence: the block when one is named, and
/// the functions that are left.
pub fn read(functions: &[String], memory: &[String], what: &str) -> Result<(Option<Block>, Vec<String>), String> {
    let step = Regex::new(r"^step from 0[xX]([0-9a-fA-F]{1,6}) to 0[xX]([0-9a-fA-F]{1,6})$").expect("a pattern of the block");
    let placed = Regex::new(r"^(state|inputs|outputs) at 0[xX]([0-9a-fA-F]{1,6})$").expect("a pattern of the block");
    let (steps, others): (Vec<String>, Vec<String>) = functions.iter().cloned().partition(|line| line.trim().starts_with("step "));
    let Some(line) = steps.first() else {
        if !memory.is_empty() {
            return Err(format!("{what}: memory says where the structures of a step are (functions: [\"cycle at 0x...\", \"step from 0x... to 0x...\"])"));
        }
        return Ok((None, others));
    };
    if steps.len() > 1 {
        return Err(format!("{what}: one block is the step: \"{}\" is a second", steps[1]));
    }
    let found = step.captures(line.trim()).ok_or_else(|| format!("{what}: \"{line}\" does not say where the step is (`step from 0x1026 to 0x1294`: from its first instruction to the first after it)"))?;
    let address = |text: &str| u32::from_str_radix(text, 16).map_err(|e| format!("{what}: \"{line}\": {e}"));
    let (from, to) = (address(&found[1])?, address(&found[2])?);
    if from % 2 != 0 || to % 2 != 0 {
        return Err(format!("{what}: \"{line}\": an instruction of the chip starts at an even address"));
    }
    if to <= from {
        return Err(format!("{what}: \"{line}\": the block ends after it starts"));
    }
    let mut at: BTreeMap<String, u32> = BTreeMap::new();
    for line in memory {
        let found = placed.captures(line.trim()).ok_or_else(|| format!("{what}: \"{line}\" does not say where a structure of the step is (`state at 0x0439`, `inputs at 0x0411`, `outputs at 0x0409`)"))?;
        if at.insert(found[1].to_string(), u32::from_str_radix(&found[2], 16).map_err(|e| format!("{what}: \"{line}\": {e}"))?).is_some() {
            return Err(format!("{what}: \"{line}\": the structure is placed twice"));
        }
    }
    let of = |name: &str| at.get(name).copied().ok_or_else(|| format!("{what}: memory does not say where the {name} of the step {} (memory: [\"state at 0x...\", \"inputs at 0x...\", \"outputs at 0x...\"])", if name == "state" { "is" } else { "are" }));
    Ok((Some(Block { from, to, state: of("state")?, inputs: of("inputs")?, outputs: of("outputs")?, ..Default::default() }), others))
}

/// A scalar of a structure: its path from the structure, where it is from the first byte, and its type.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub path: String,
    pub offset: u32,
    pub size: u32,
    pub boolean: bool,
    /// The type as the header writes it.
    pub ty: String,
    /// For the count of a queue: how many events the queue holds.
    pub at_most: Option<u32>,
}

/// The size of a scalar of the generated C on an AVR, where a `bool` is a byte.
fn scalar(ty: &str) -> Option<u32> {
    match ty {
        "bool" | "uint8_t" | "int8_t" => Some(1),
        "uint16_t" | "int16_t" => Some(2),
        "uint32_t" | "int32_t" => Some(4),
        "uint64_t" | "int64_t" => Some(8),
        _ => None,
    }
}

/// The name the generated C gives everything of the model: what its step function is called before `_step`.
pub fn prefix(header: &str) -> Result<String, String> {
    let step = Regex::new(r"(?m)^void (\w+)_step\((\w+)_state \*state, const (\w+)_inputs \*inputs, (\w+)_outputs \*outputs\);$").expect("a pattern of the block");
    let found = step.captures_iter(header).find(|found| found[1] == found[2] && found[1] == found[3] && found[1] == found[4]).map(|found| found[1].to_string());
    found.ok_or_else(|| "the header the model generates declares no step of its machines".to_string())
}

/// The scalars of a structure of the generated header, laid out as avr-gcc lays them out: in the order they are
/// declared, with no padding (every type of the chip is aligned on one byte).
pub fn layout(header: &str, structure: &str) -> Result<Vec<Field>, String> {
    let comment = Regex::new(r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/").expect("a pattern of the block");
    let bare = comment.replace_all(header, "");
    let typedef = Regex::new(r"typedef struct \{([^}]*)\} (\w+);").expect("a pattern of the block");
    let member = Regex::new(r"^(\w+) (\w+)(?:\[(\w+)\])?$").expect("a pattern of the block");
    let define = Regex::new(r"(?m)^#define (\w+) (\d+)U?$").expect("a pattern of the block");
    let sizes: BTreeMap<&str, u32> = define.captures_iter(&bare).filter_map(|found| Some((found.get(1)?.as_str(), found[2].parse().ok()?))).collect();
    let structures: BTreeMap<&str, &str> = typedef.captures_iter(&bare).filter_map(|found| Some((found.get(2)?.as_str(), found.get(1)?.as_str()))).collect();
    fn walk(structure: &str, path: &str, offset: &mut u32, structures: &BTreeMap<&str, &str>, sizes: &BTreeMap<&str, u32>, member: &Regex, fields: &mut Vec<Field>) -> Result<(), String> {
        let body = structures.get(structure).ok_or_else(|| format!("the header the model generates has no structure `{structure}`"))?;
        let members: Vec<(String, String, Option<u32>)> = body
            .split(';')
            .map(str::trim)
            .filter(|declared| !declared.is_empty())
            .map(|declared| {
                let found = member.captures(declared).ok_or_else(|| format!("`{declared}` of `{structure}` is not a member a block is proven with"))?;
                let count = match found.get(3).map(|count| count.as_str()) {
                    None => None,
                    Some(count) => Some(count.trim_end_matches('U').parse().ok().or_else(|| sizes.get(count).copied()).ok_or_else(|| format!("the size `{count}` of `{}` in `{structure}` is not a number the header gives", &found[2]))?),
                };
                Ok((found[1].to_string(), found[2].to_string(), count))
            })
            .collect::<Result<_, String>>()?;
        let queue = members.iter().find(|(_, name, count)| name == "queue" && count.is_some()).and_then(|(_, _, count)| *count);
        for (ty, name, count) in &members {
            for rank in 0..count.unwrap_or(1) {
                let here = format!("{path}{}{name}{}", if path.is_empty() { "" } else { "." }, count.map(|_| format!("[{rank}]")).unwrap_or_default());
                match scalar(ty) {
                    Some(size) => {
                        fields.push(Field { path: here, offset: *offset, size, boolean: ty == "bool", ty: ty.clone(), at_most: queue.filter(|_| name == "queued") });
                        *offset += size;
                    }
                    None if structures.contains_key(ty.as_str()) => walk(ty, &here, offset, structures, sizes, member, fields)?,
                    None => return Err(format!("`{here}` is a `{ty}`: a block is proven for machines whose state, inputs and outputs are booleans and integers")),
                }
            }
        }
        Ok(())
    }
    let (mut fields, mut offset) = (Vec::new(), 0);
    walk(structure, "", &mut offset, &structures, &sizes, &member, &mut fields)?;
    if fields.is_empty() {
        return Err(format!("the structure `{structure}` of the generated header is empty: there is nothing to place in the memory"));
    }
    Ok(fields)
}

/// The three structures of a step: (what the correspondence calls it, the variable of the proof, its fields, where it is).
type Structures = Vec<(&'static str, &'static str, Vec<Field>, u32)>;

/// Read the three structures of the header, and check that they lie in the memory of the chip, apart.
fn structures(block: &Block, header: &str, what: &str) -> Result<(String, Structures), String> {
    let name = prefix(header).map_err(|e| format!("{what}: {e}"))?;
    let mut found: Structures = Vec::new();
    for (said, variable, at) in [("state", "s", block.state), ("inputs", "in", block.inputs), ("outputs", "out", block.outputs)] {
        found.push((said, variable, layout(header, &format!("{name}_{said}")).map_err(|e| format!("{what}: {e}"))?, at));
    }
    let bytes = |fields: &[Field]| fields.last().map_or(0, |last| last.offset + last.size);
    for (said, _, fields, at) in &found {
        if *at < RAM.0 || at + bytes(fields) - 1 > RAM.1 {
            return Err(format!("{what}: the {said} ({} bytes at 0x{at:04x}) {} not within the memory of the chip (0x{:04x} to 0x{:04x})", bytes(fields), if *said == "state" { "is" } else { "are" }, RAM.0, RAM.1));
        }
    }
    for (rank, (said, _, fields, at)) in found.iter().enumerate() {
        for (other, _, others, there) in &found[rank + 1..] {
            if *at < there + bytes(others) && *there < at + bytes(fields) {
                return Err(format!("{what}: the {said} (0x{at:04x}, {} bytes) and the {other} (0x{there:04x}, {} bytes) share bytes of the memory", bytes(fields), bytes(others)));
            }
        }
    }
    Ok((name, found))
}

/// What the compiler checks of a block without its binary: the header of the model gives the three structures,
/// and they fit in the memory of the chip where the correspondence places them.
pub fn check(block: &Block, header: &str, what: &str) -> Result<(), String> {
    structures(block, header, what).map(|_| ())
}

/// The bytes of a field in the memory, as one number (the low byte first).
fn bytes_of(field: &Field, at: u32) -> String {
    let wide = if field.size > 4 { "unsigned long long" } else { "unsigned long" };
    let parts: Vec<String> = (0..field.size).map(|k| format!("(({wide})arclang_mem[0x{:04x}u] << {})", at + field.offset + k, 8 * k)).collect();
    format!("({})", parts.join(" | "))
}

/// The translation with the block made what the function of the cycle runs: entered at its first instruction,
/// ended before the first after it.
fn cut_at(lifted: &str, block: &Block, cycle: u32, function: u32, what: &str) -> Result<String, String> {
    let head = format!("void f_{function:04x}(void)");
    let lines: Vec<&str> = lifted.lines().collect();
    let first = lines.iter().position(|line| *line == head).ok_or_else(|| format!("{what}: the translation has no function at 0x{function:04x}"))?;
    let last = lines[first..].iter().position(|line| *line == "}").map_or(lines.len(), |length| first + length);
    let (enter, from, to) = (format!("    if (arclang_resume) {{ arclang_resume = false; goto R_{cycle:04x}; }}"), format!("L_{:04x}: /*", block.from), format!("L_{:04x}: /*", block.to));
    let within = |start: &str| lines[first..last].iter().any(|line| line.starts_with(start));
    for (label, at, which) in [(&from, block.from, "first"), (&to, block.to, "second")] {
        if !within(label) {
            return Err(format!("{what}: 0x{at:04x}, the {which} address of the step, is not an instruction of the function of the cycle (0x{function:04x})"));
        }
    }
    if !lines[first..last].contains(&enter.as_str()) {
        return Err(format!("{what}: the translation does not enter the function of the cycle (0x{function:04x}) at its head"));
    }
    let mut written = String::with_capacity(lifted.len() + 256);
    for (rank, line) in lines.iter().enumerate() {
        let inside = rank > first && rank < last;
        if inside && *line == enter {
            written.push_str(&format!("    if (arclang_resume) {{ arclang_resume = false; goto L_{:04x}; }}\n", block.from));
        } else {
            written.push_str(line);
            written.push('\n');
            if inside && line.starts_with(&to) {
                written.push_str("    arclang_block_end = true; arclang_stop = true; return;\n");
            }
        }
    }
    Ok(written)
}

/// The files of the proof of a block, and its lines in the order the proof holds them: (the files, the lines, the bound of its loops).
#[allow(clippy::type_complexity)]
fn harness(block: &Block, cut: &super::binary_cycle::Cut, lifted: &str, header_name: &str, header: &str, what: &str) -> Result<(Vec<(String, String)>, Vec<String>, usize), String> {
    let (name, found) = structures(block, header, what)?;
    let start = cut.start.as_ref().filter(|state| state.reached && state.registers.len() == 32).ok_or_else(|| format!("{what}: the start of the binary, run to find what it leaves, did not reach the head of the cycle at 0x{:04x}", cut.at))?;
    let machine_code = super::binary_order::in_line(&cut_at(lifted, block, cut.at, cut.function, what)?);
    let mut c = format!("/* The proof that a block of the machine code is one step of the machines of the model (written by `arclang prove`). */\n#include <stdbool.h>\n#include <stdint.h>\n#include \"model/{header_name}\"\nstatic bool arclang_block_end; /* the block ran to the first instruction after it */\n#include \"machine_code.c\"\nunsigned int nondet_uint(void);\n\nint main(void)\n{{\n    static {name}_state s;\n    static {name}_inputs in;\n    static {name}_outputs out;\n");
    c.push_str("    /* Any memory; any registers, but those the cycle keeps and the stack pointer, which are as the start leaves them. */\n    __CPROVER_havoc_object(arclang_mem);\n");
    for register in 0u8..32 {
        c.push_str(&match cut.kept.contains(&register) {
            true => format!("    arclang_r[{register}] = {}u;\n", start.registers[usize::from(register)]),
            false => format!("    arclang_r[{register}] = nondet_uchar();\n"),
        });
    }
    c.push_str(&format!("    for (unsigned flag = 0x80u; flag <= 0x88u; flag++) {{ arclang_reg[flag] = nondet_uchar() & 1u; }}\n    arclang_reg_wr(ARCLANG_SP, 2, {}u);\n    arclang_booting = false;\n    arclang_stop = false;\n    arclang_uncovered = false;\n    arclang_outside = false;\n    arclang_wrote_register = false;\n    arclang_block_end = false;\n", start.sp));
    // The structures of the model are what the memory holds: a boolean is 0 or 1, a queue holds no more than its size.
    for (_, variable, fields, at) in found.iter().filter(|(said, ..)| *said != "outputs") {
        for field in fields {
            if field.boolean {
                c.push_str(&format!("    __CPROVER_assume(arclang_mem[0x{:04x}u] <= 1u);\n", at + field.offset));
            }
            if let Some(most) = field.at_most {
                c.push_str(&format!("    __CPROVER_assume({} <= {most}u);\n", bytes_of(field, *at)));
            }
            c.push_str(&format!("    {variable}.{} = ({}){};\n", field.path, field.ty, bytes_of(field, *at)));
        }
    }
    // A byte of the memory that is neither of the state, nor of the outputs, nor under the stack pointer: any one.
    let bytes = |fields: &[Field]| fields.last().map_or(0, |last| last.offset + last.size);
    let apart: Vec<String> = found.iter().filter(|(said, ..)| *said != "inputs").map(|(_, _, fields, at)| format!("!(other >= 0x{at:04x}u && other < 0x{:04x}u)", at + bytes(fields))).collect();
    let sp = u32::from(start.sp);
    c.push_str(&format!("    unsigned other = nondet_uint();\n    __CPROVER_assume(other >= 0x{:04x}u && other <= 0x{:04x}u && {} && !(other > {}u && other <= {sp}u));\n    uint8_t was = arclang_mem[other];\n", RAM.0, RAM.1, apart.join(" && "), sp.saturating_sub(STACK)));
    c.push_str(&format!("\n    arclang_resume = true;\n    f_{:04x}();\n    {name}_step(&s, &in, &out);\n\n", cut.function));
    let mut lines: Vec<(String, String)> = vec![(
        "arclang_block_end && !arclang_uncovered && !arclang_outside && !arclang_wrote_register".to_string(),
        format!("the block runs from 0x{:04x} to 0x{:04x} on instructions the translation covers, within the memory of the chip, and writes no register of the chip", block.from, block.to),
    )];
    for (said, variable, fields, at) in found.iter().filter(|(said, ..)| *said != "inputs") {
        for field in fields {
            let wide = if field.size > 4 { "unsigned long long" } else { "unsigned long" };
            let unsigned = format!("u{}", field.ty.trim_start_matches('u'));
            let model = if field.boolean { format!("({wide}){variable}.{}", field.path) } else { format!("({wide})({unsigned}){variable}.{}", field.path) };
            lines.push((format!("{model} == {}", bytes_of(field, *at)), format!("{said}: {} (0x{:04x}) is what the step of the model leaves", field.path, at + field.offset)));
        }
    }
    let kept: String = cut.kept.iter().map(|register| format!(" && arclang_r[{register}] == {}u", start.registers[usize::from(*register)])).collect();
    lines.push((format!("arclang_reg_rd(ARCLANG_SP, 2) == {}u{kept}", start.sp), "the block leaves the stack pointer and the registers the cycle keeps as they were".to_string()));
    lines.push(("arclang_mem[other] == was".to_string(), format!("the block writes no byte of the memory outside the state, the outputs and the {STACK} bytes under the stack pointer: its inputs and everything else are as they were")));
    for (condition, written) in &lines {
        c.push_str(&format!("    __CPROVER_assert({condition}, \"{}\");\n", written.replace('"', "'")));
    }
    c.push_str("    return 0;\n}\n");
    let longest = found.iter().flat_map(|(_, _, fields, _)| fields.iter().filter_map(|field| field.at_most)).max().unwrap_or(0) as usize;
    Ok((vec![("proof.c".to_string(), c), ("machine_code.c".to_string(), machine_code), ("arclang_lifted.h".to_string(), super::binary::LIFTED_H.to_string())], lines.into_iter().map(|(_, written)| written).collect(), UNWIND.max(longest + 4)))
}

/// What CBMC says of the checks of a proof.
enum Said {
    /// Every check holds, and no loop runs past its bound.
    Holds,
    /// These checks do not hold, as the proof writes them.
    Fails(Vec<String>),
    /// The checks hold where the loops end within their bound, and some loop does not.
    BeyondBound,
    Unknown(String),
}

/// Put the checks of a block to CBMC and its own solver, which says of each check whether it holds: the question
/// is small (seconds), and a refutation names its lines. (The formula CBMC writes for another solver is not used
/// here: CBMC 6.4 fails to write it for some of these questions.)
fn asked(dir: &Path, sources: &[&str], flags: &[&str], unwind: usize, seconds: u64) -> Result<Said, String> {
    use std::process::{Command, Stdio};
    let cbmc = std::env::var("ARCLANG_CBMC").ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| "cbmc".to_string());
    let said = dir.join("cbmc.txt");
    let file = std::fs::File::create(&said).map_err(|e| format!("cannot write {}: {e}", said.display()))?;
    let errors = file.try_clone().map_err(|e| format!("cannot write {}: {e}", said.display()))?;
    let mut child = Command::new(&cbmc)
        .args(sources)
        .args(flags)
        .args(["--no-standard-checks", "--unwind", &unwind.to_string(), "--unwinding-assertions"])
        .current_dir(dir)
        .stdout(file)
        .stderr(Stdio::from(errors))
        .spawn()
        .map_err(|e| format!("cannot run '{cbmc}': {e} (run in ArcLang's formal image: tools/embedded/formal/run.sh)"))?;
    let started = std::time::Instant::now();
    let status = loop {
        match child.try_wait().map_err(|e| format!("cannot run '{cbmc}': {e}"))? {
            Some(status) => break status,
            None if started.elapsed().as_secs() > seconds => {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(Said::Unknown(format!("no answer within {seconds} s")));
            }
            None => std::thread::sleep(std::time::Duration::from_millis(50)),
        }
    };
    let text = std::fs::read_to_string(&said).map_err(|e| format!("cannot read {}: {e}", said.display()))?;
    Ok(read_answer(&text, status.code()))
}

/// Read what CBMC printed. A proof is its own words for it, and nothing less: every check counted, none failed.
fn read_answer(text: &str, exit: Option<i32>) -> Said {
    // A function CBMC has no body for is one it lets do anything: its effects would be missed.
    let no_body = Regex::new(r"no body for (?:function|callee) (\w+)").expect("a pattern of the block");
    let missing: std::collections::BTreeSet<&str> = no_body.captures_iter(text).filter_map(|found| found.get(1)).map(|name| name.as_str()).collect();
    if !missing.is_empty() {
        return Said::Unknown(format!("the code calls {}, which this proof has no body for", missing.into_iter().collect::<Vec<_>>().join(", ")));
    }
    let refuted = Regex::new(r"(?m)^\[([^\]]+)\] line \d+ (.+): FAILURE$").expect("a pattern of the block");
    let failures: Vec<(String, String)> = refuted.captures_iter(text).map(|found| (found[1].to_string(), found[2].to_string())).collect();
    let lines: Vec<String> = failures.iter().filter(|(check, _)| !check.contains(".unwind.")).map(|(_, written)| written.clone()).collect();
    if !lines.is_empty() {
        return Said::Fails(lines);
    }
    if !failures.is_empty() {
        return Said::BeyondBound;
    }
    if exit == Some(0) && text.contains("VERIFICATION SUCCESSFUL") && text.contains("** 0 of ") {
        return Said::Holds;
    }
    if exit == Some(137) || exit.is_none() {
        return Said::Unknown("CBMC was killed before it answered (the memory of the machine or of its container ran out)".to_string());
    }
    let tail: Vec<&str> = text.lines().filter(|line| !line.trim().is_empty()).rev().take(6).collect();
    Said::Unknown(format!("CBMC gave no verdict:\n{}", tail.into_iter().rev().collect::<Vec<_>>().join("\n")))
}

/// Prove the block of a correspondence against the step of the model. `model`: (the header, the source) of its
/// generated C, by file name. The block is filled with what the proof found of it.
#[allow(clippy::too_many_arguments)]
pub fn prove(block: &mut Block, cycle: u32, image: &super::binary::Image, model: (&str, &str, &str, &str), board: &[(&str, &str)], dir: &Path, seconds: u64, what: &str) -> Result<super::correspondence::Outcome, String> {
    use super::correspondence::Outcome;
    let (header_name, header, code_name, code) = model;
    // Before any tool is run: the structures are those of the header, where the correspondence places them.
    check(block, header, what)?;
    let (lifted, start) = super::binary::lift_with_start(image, &[], Some(cycle), &dir.join("lift")).map_err(|e| format!("{what}: {e}"))?;
    let mut cut = super::binary_cycle::cut(&lifted, what)?;
    cut.start = Some(start);
    let (files, lines, unwind) = harness(block, &cut, &lifted, header_name, header, what)?;
    let model_source = format!("model/{code_name}");
    let model_files = [(format!("model/{header_name}"), header.to_string()), (model_source.clone(), code.to_string())];
    for (name, text) in files.iter().chain(&model_files).map(|(name, text)| (name.as_str(), text.as_str())).chain(board.iter().copied()) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        std::fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }
    if let Some(state) = &cut.start {
        let (_, found) = structures(block, header, what)?;
        block.function = cut.function;
        block.sp = state.sp;
        block.kept = cut.kept.iter().map(|register| (*register, state.registers[usize::from(*register)])).collect();
        block.sizes = [0, 1, 2].map(|rank| found[rank].2.last().map_or(0, |last| last.offset + last.size));
    }
    let started = std::time::Instant::now();
    // Every byte of the memory is followed alone, as in a cycle: the stack is where the start left it.
    let flags = ["--nondet-static", "-I", "board", "--16", "-DARCLANG_INT16", "--max-field-sensitivity-array-size", "4096"];
    let sources = ["proof.c", model_source.as_str()];
    Ok(match asked(dir, &sources, &flags, unwind, seconds)? {
        Said::Holds => Outcome::Proven { seconds: started.elapsed().as_secs_f64(), period_us: 0, int_bits: 16, unrelated: Vec::new(), board_clock: None, interrupts: None },
        // In the order of the proof, each once (a line can be refuted on several paths).
        Said::Fails(refuted) => Outcome::Refuted { obligation: "block".to_string(), lines: lines.iter().filter(|line| refuted.contains(line)).cloned().collect(), case: None, seconds: started.elapsed().as_secs_f64() },
        Said::BeyondBound => Outcome::Unknown { reason: format!("a loop of the block or of the model runs more than {unwind} times: the proof does not cover it") },
        Said::Unknown(reason) => Outcome::Unknown { reason: format!("block: {reason}") },
    })
}

/// What the proof of a block is of, what it takes without proving it, and what it does not say, as it is printed under it.
pub fn said(file: &str, cycle: u32, block: &Block) -> Vec<String> {
    let kept = block.kept.iter().map(|(register, value)| format!("r{register} = {value}")).collect::<Vec<_>>().join(", ");
    vec![
        format!("of the machine code of {file} itself, translated one instruction at a time (ATmega328P, Ghidra's description of the AVR): no source is read"),
        format!("the block is the instructions from 0x{:04x} up to 0x{:04x} of the function at 0x{:04x}, run from any memory in which the booleans of the state and of the inputs are 0 or 1 and a queue holds no more events than its size", block.from, block.to, block.function),
        format!("the structures of the generated header are laid out as avr-gcc lays them out (no padding, the low byte first): the state ({} bytes at 0x{:04x}), the inputs ({} bytes at 0x{:04x}), the outputs ({} bytes at 0x{:04x})", block.sizes[0], block.state, block.sizes[1], block.inputs, block.sizes[2], block.outputs),
        format!("proven with it: the block runs only instructions the translation covers, writes no register of the chip, leaves the stack pointer and the registers the cycle keeps, and writes no byte of the memory outside the state, the outputs and the {STACK} bytes under the stack pointer"),
        format!("taken, not proven here: at the first instruction of the block the stack pointer ({}) and the registers the cycle keeps ({kept}) are as the start leaves them at the head of the cycle (0x{cycle:04x}), found by running the start once", block.sp),
        "NOT said: when the block runs (its period, the time of the model), that the rest of the cycle leaves the state alone between two runs of the block, that the start leaves the initial state, where the inputs come from and what the outputs drive".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "#ifndef M_QUEUE_SIZE\n#define M_QUEUE_SIZE 4U\n#endif\ntypedef struct {\n    uint16_t queue[M_QUEUE_SIZE]; /* the events; of the step */\n    uint8_t queued; /* how many */\n    bool cond_ready; /* condition \"ready\" */\n} m_lamp_inputs;\ntypedef struct {\n    bool st_on; /* state On */\n    int16_t v_level;\n} m_lamp_outputs;\ntypedef struct {\n    bool active[2];\n    uint32_t in_state[2]; /* steps since the state was entered (saturates) */\n} m_lamp_state;\ntypedef struct {\n    m_lamp_inputs lamp;\n} m_inputs;\ntypedef struct {\n    m_lamp_outputs lamp;\n} m_outputs;\ntypedef struct {\n    bool lamp_started;\n    m_lamp_state lamp;\n} m_state;\nvoid m_init(m_state *state);\nvoid m_step(m_state *state, const m_inputs *inputs, m_outputs *outputs);\nvoid m_lamp_step(m_state *state, const m_inputs *inputs, m_outputs *outputs);\n";

    fn lines(of: &[&str]) -> Vec<String> {
        of.iter().map(|line| line.to_string()).collect()
    }

    #[test]
    fn the_structures_of_the_header_are_laid_out_with_no_padding() {
        assert_eq!(prefix(HEADER).unwrap(), "m");
        let state = layout(HEADER, "m_state").unwrap();
        let placed: Vec<(&str, u32, u32)> = state.iter().map(|field| (field.path.as_str(), field.offset, field.size)).collect();
        assert_eq!(placed, [("lamp_started", 0, 1), ("lamp.active[0]", 1, 1), ("lamp.active[1]", 2, 1), ("lamp.in_state[0]", 3, 4), ("lamp.in_state[1]", 7, 4)]);
        let inputs = layout(HEADER, "m_inputs").unwrap();
        // The size of the queue is the one the header defines; its count is at most that size.
        assert_eq!(inputs.iter().map(|field| (field.path.as_str(), field.offset)).collect::<Vec<_>>(), [("lamp.queue[0]", 0), ("lamp.queue[1]", 2), ("lamp.queue[2]", 4), ("lamp.queue[3]", 6), ("lamp.queued", 8), ("lamp.cond_ready", 9)]);
        assert_eq!(inputs[4].at_most, Some(4));
        assert!(inputs[5].boolean && !inputs[4].boolean);
        let outputs = layout(HEADER, "m_outputs").unwrap();
        assert_eq!((outputs[1].path.as_str(), outputs[1].offset, outputs[1].size, outputs[1].ty.as_str()), ("lamp.v_level", 1, 2, "int16_t"));
    }

    #[test]
    fn a_type_that_is_no_boolean_and_no_integer_is_refused_by_its_name() {
        let refused = layout(&HEADER.replace("int16_t v_level", "double v_level"), "m_outputs").unwrap_err();
        assert!(refused.contains("`lamp.v_level` is a `double`"), "{refused}");
        assert!(layout(HEADER, "m_nothing").unwrap_err().contains("has no structure `m_nothing`"));
    }

    #[test]
    fn a_block_is_read_from_the_functions_and_the_memory_of_a_correspondence() {
        let (block, others) = read(&lines(&["cycle at 0x0fc6", "step from 0x1026 to 0x1294"]), &lines(&["state at 0x0439", "inputs at 0x0411", "outputs at 0x0409"]), "c").unwrap();
        let block = block.unwrap();
        assert_eq!((block.from, block.to, block.state, block.inputs, block.outputs), (0x1026, 0x1294, 0x439, 0x411, 0x409));
        assert_eq!(others, ["cycle at 0x0fc6"]);
        assert_eq!(read(&lines(&["cycle at 0x0fc6"]), &[], "c").unwrap(), (None, lines(&["cycle at 0x0fc6"])));
    }

    #[test]
    fn what_a_block_cannot_say_is_refused_with_the_reason() {
        let memory = lines(&["state at 0x0439", "inputs at 0x0411", "outputs at 0x0409"]);
        let wrong = |functions: &[&str], memory: &[String]| read(&lines(functions), memory, "c").unwrap_err();
        assert!(wrong(&["step at 0x1026"], &memory).contains("does not say where the step is"));
        assert!(wrong(&["step from 0x1027 to 0x1294"], &memory).contains("even address"));
        assert!(wrong(&["step from 0x1294 to 0x1026"], &memory).contains("ends after it starts"));
        assert!(wrong(&["step from 0x1026 to 0x1294", "step from 0x1300 to 0x1400"], &memory).contains("is a second"));
        assert!(wrong(&["step from 0x1026 to 0x1294"], &memory[..2]).contains("where the outputs of the step are"));
        assert!(wrong(&["step from 0x1026 to 0x1294"], &lines(&["state at 0x0439", "state at 0x0440"])).contains("placed twice"));
        assert!(wrong(&["step from 0x1026 to 0x1294"], &lines(&["heap at 0x0439"])).contains("does not say where a structure"));
        assert!(wrong(&["cycle at 0x0fc6"], &memory).contains("memory says where the structures of a step are"));
    }

    #[test]
    fn the_structures_lie_in_the_memory_of_the_chip_and_apart() {
        let block = |state, inputs, outputs| Block { from: 0x100, to: 0x200, state, inputs, outputs, ..Default::default() };
        assert!(check(&block(0x200, 0x220, 0x240), HEADER, "c").is_ok());
        assert!(check(&block(0x8fa, 0x220, 0x240), HEADER, "c").unwrap_err().contains("the state (11 bytes at 0x08fa) is not within the memory of the chip"));
        assert!(check(&block(0x80, 0x220, 0x240), HEADER, "c").unwrap_err().contains("not within the memory"));
        assert!(check(&block(0x200, 0x205, 0x240), HEADER, "c").unwrap_err().contains("the state (0x0200, 11 bytes) and the inputs (0x0205, 10 bytes) share bytes"));
    }

    #[test]
    fn a_proof_is_cbmcs_own_words_for_it_and_a_refutation_names_its_lines() {
        let holds = "[main.assertion.1] line 5 the state: SUCCESS\n\n** 0 of 2 failed (1 iterations)\nVERIFICATION SUCCESSFUL\n";
        assert!(matches!(read_answer(holds, Some(0)), Said::Holds));
        // Without its words, or with an exit that is not a success, it is no proof.
        assert!(matches!(read_answer("** 0 of 2 failed (1 iterations)\n", Some(0)), Said::Unknown(_)));
        assert!(matches!(read_answer(holds, Some(6)), Said::Unknown(_)));
        assert!(matches!(read_answer("", Some(137)), Said::Unknown(reason) if reason.contains("killed")));
        let fails = "[main.assertion.3] line 9 state: lamp.active[0] (0x0201) is what the step of the model leaves: FAILURE\n[f_0100.unwind.2] line 40 unwinding assertion loop 2: FAILURE\n** 2 of 9 failed (2 iterations)\nVERIFICATION FAILED\n";
        assert!(matches!(read_answer(fails, Some(10)), Said::Fails(lines) if lines == ["state: lamp.active[0] (0x0201) is what the step of the model leaves"]));
        let beyond = "[f_0100.unwind.2] line 40 unwinding assertion loop 2: FAILURE\n** 1 of 9 failed (2 iterations)\nVERIFICATION FAILED\n";
        assert!(matches!(read_answer(beyond, Some(10)), Said::BeyondBound));
        assert!(matches!(read_answer("**** WARNING: no body for function f_0632\nVERIFICATION SUCCESSFUL\n** 0 of 1 failed\n", Some(0)), Said::Unknown(reason) if reason.contains("f_0632")));
    }

    #[test]
    fn the_block_is_entered_at_its_first_instruction_and_ended_before_the_first_after_it() {
        let lifted = "void f_0100(void)\n{\n    if (arclang_resume) { arclang_resume = false; goto R_0102; }\n    goto L_0100;\nL_0100: /* nop */\nL_0102: /* head */\n    if (arclang_cut()) { return; }\nR_0102: ;\nL_0104: /* first */\n    x = 1;\nL_0106: /* after */\n    y = 1;\n    arclang_uncovered = true; /* the function runs off its end */\n}\n";
        let block = Block { from: 0x104, to: 0x106, ..Default::default() };
        let written = cut_at(lifted, &block, 0x102, 0x100, "c").unwrap();
        assert!(written.contains("    if (arclang_resume) { arclang_resume = false; goto L_0104; }\n"), "{written}");
        assert!(written.contains("L_0106: /* after */\n    arclang_block_end = true; arclang_stop = true; return;\n    y = 1;\n"), "{written}");
        let outside = cut_at(lifted, &Block { from: 0x104, to: 0x300, ..Default::default() }, 0x102, 0x100, "c").unwrap_err();
        assert!(outside.contains("0x0300, the second address of the step, is not an instruction of the function of the cycle (0x0100)"), "{outside}");
    }
}
