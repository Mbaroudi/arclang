//! `arclang recover <sketch> --propose`: a model of code written by hand,
//! proposed by an AI and judged by the proof.
//!
//! Three roles, kept apart. The sketch is read as it is. A proposer, any
//! command that reads a prompt on its input and prints its answer, proposes
//! a state machine and the `correspondence` that ties the sketch to it.
//! ArcLang judges: the proposal is compiled and proven as `arclang prove`
//! does. Refuted, what failed goes back to the proposer, for a bounded
//! number of turns. Nothing a proposer says is taken on trust: a model is
//! only written as recovered once its proof holds, and the same proof is
//! replayed by `arclang prove` with no AI.

use super::CliError;
use crate::compiler::recover::correspondence::{self, Outcome};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub struct Options {
    /// The command that proposes: the prompt on its input, its answer on its output.
    pub with: Option<String>,
    pub turns: usize,
    /// The time one proof may take, in seconds.
    pub timeout: u64,
    pub output: Option<PathBuf>,
    pub keep: Option<PathBuf>,
}

/// The time a proposer may take for one answer.
const PROPOSER_SECONDS: u64 = 600;

/// What a proposer is told: the task, the language it answers in, and how it is judged.
const BRIEF: &str = r#"You are recovering the model of an Arduino sketch written by hand.

Answer with ONE ArcLang model in a single fenced block (```arclang ... ```), nothing else that is code. The model holds:
- `model "Name" { version: "1.0.0" }`
- one `state_machine` that does what the sketch does, one step for one `loop()`;
- one `correspondence` that ties the sketch to the machine.

Your answer is not trusted: it is compiled, then proven by a model checker. The proof has two parts. START: after `setup()` and one `loop()`, and after the machine's start and one step with the same inputs, every line of the correspondence holds. STEP: from ANY state of the sketch and ANY state of the machine for which every line holds (reachable or not), one `loop()` and one step with the same inputs leave two states for which every line holds. If a part fails you are told the lines that do not hold after it, and you answer again.

The state machine:
```arclang
state_machine "Debounced button" {
    variables: [
        "in button: Boolean",
        "led: Boolean = false"
    ]
    initial: "Low"
    state Low {}
    state Rising {}
    state High { entry: "led := true" }
    state Falling {}
    transition Low -> Rising { guard: "button" }
    transition Rising -> High { guard: "button" timing: 50 ms }
    transition Rising -> Low { guard: "not button" }
    transition High -> Falling { guard: "not button" }
    transition Falling -> Low { guard: "not button" timing: 50 ms action: "led := false" }
    transition Falling -> High { guard: "button" }
}
```
- variables are `Boolean`, `Integer` or `Real`; `in` marks an input; others take an initial value;
- guards use `and`, `or`, `not`, comparisons and + - * /, with `min(a, b)`, `max(a, b)`, `abs(a)`; there is no remainder (%) and no conditional expression: write two transitions with opposite guards instead; actions are `name := expression`, several separated by `;`;
- `timing: 50 ms` on a transition: it fires once its source state has been held that long and its guard holds;
- at most one transition fires in a step; the machine enters its initial state at its first step.

The correspondence of that machine with a sketch whose variables are `stable`, `counting` and `since`:
```arclang
correspondence "Debounce sketch" {
    machine: "Debounced button"
    source: "debounce.ino"
    period: 10 ms
    reads: ["digitalRead(BUTTON) is button"]
    writes: ["digitalWrite(LED) is led"]
    state: [
        "stable == 0 && counting == 0 is in Low",
        "stable == 0 && counting == 1 is in Rising",
        "stable == 1 && counting == 0 is in High",
        "stable == 1 && counting == 1 is in Falling",
        "millis() - since is time in Rising",
        "millis() - since is time in Falling",
        "stable == 1 is led"
    ]
    invariant: ["stable == 0 || stable == 1", "counting == 0 || counting == 1"]
}
```
- `source`: the file name of the sketch, exactly as given below;
- `period`: the time of one `loop()`; when the sketch calls `delay()`, the sum of its delays;
- `reads`: `digitalRead(PIN) is <input>` (Boolean) or `analogRead(PIN) is <input>` (Integer, 0 to 1023), the pin written as the sketch writes it;
- `writes`: `digitalWrite(PIN) is <variable>` or `digitalWrite(PIN) is in <State>`, and `analogWrite(PIN) is <variable>` (an Integer, 0 to 255); EVERY pin the sketch reads or writes must be named;
- `events`: when the sketch takes a pin by an interrupt (`attachInterrupt(digitalPinToInterrupt(PIN), routine, RISING)`), each edge is an event of the machine: `"RISING on PIN is <event>"` (or FALLING, CHANGE), and the machine has `transition A -> B { trigger: "<event>" ... }`; several events can come in one step, each fires its transition in turn; what the routine counts and `loop()` resets is 0 between two loops (say it as an invariant);
- what the sketch prints on `Serial` is not part of the proof: leave it out; a time read with `micros()` is related as `(micros() - since) / 1000 is time in <State>`;
- `state`: on the left a C expression over the sketch's global variables, on the right `in <State>` (true exactly when the state is active), `time in <State>` (milliseconds the state has been held) or a variable of the machine;
- `invariant`: C expressions that always hold of the sketch's variables;
- `int: 16` when the board is an AVR one (Uno, Nano, Mega).

What makes the STEP hold: it starts from any states the lines allow, so the lines must say enough. Give every state of the machine an expression, so that exactly one is true; bound each variable of the sketch with an invariant (`x == 0 || x == 1`); say how the sketch's variables give each output (`"lit == 1 is led"`); relate every timer of the sketch to the `time in` of the state that waits.
"#;

/// What a proposer is told of a binary, around the language of the machine (the same as for a sketch).
const BINARY_INTRO: &str = r#"You are recovering the model of the firmware of an Arduino UNO (an ATmega328P) from its binary alone: there is no source.

Answer with ONE ArcLang model in a single fenced block (```arclang ... ```), nothing else that is code. The model holds:
- `model "Name" { version: "1.0.0" }`
- one `state_machine` that does what the firmware does, one step for one turn of its `loop`;
- one `correspondence` that ties the binary to the machine.

Your answer is not trusted: it is compiled, then proven by a model checker against the machine code itself, translated instruction by instruction. The proof has two parts. START: after the code has run from its reset through one `loop`, and after the machine's start and one step with the same inputs, every line of the correspondence holds. STEP: from ANY memory of the chip and ANY state of the machine for which every line holds, one `loop` and one step with the same inputs leave two for which every line holds again. What fails comes back to you.
"#;

const BINARY_CORRESPONDENCE: &str = r#"The correspondence was started for you from the machine code: keep its `binary`, its `period` and its `functions` exactly as given (a function you add, move or rename is refused). Complete it:
- `machine`: the name of your state machine;
- `reads`: `digitalRead(2) is <input>` (Boolean) or `analogRead(0) is <input>` (Integer, 0 to 1023), the pin by its number;
- `writes`: `digitalWrite(13) is <variable>` or `digitalWrite(13) is in <State>`, and `analogWrite(9) is <variable>` (an Integer, 0 to 255); EVERY pin `loop` reads or writes must be named;
- `state`: on the left a C expression over the bytes of the memory, on the right `in <State>` (true exactly when the state is active), `time in <State>` (milliseconds the state has been held) or a variable of the machine. A binary has no names: a variable is `byte(0x0104)` (one byte), `word(0x0104)` (two bytes, an `int`, low byte first) or `dword(0x0100)` (four bytes, a `long`). A time the code took with `millis()` and kept at 0x0100 is related as `millis() - dword(0x0100) is time in <State>`;
- `invariant`: C expressions that always hold of those bytes (`word(0x0104) == 0 || word(0x0104) == 1`).

How to read the listing: one line an instruction, its address then the instruction, in Ghidra's notation for the AVR. `R25R24` is the pair r25:r24, `X`, `Y`, `Z` the pairs r27:r26, r29:r28, r31:r30 (`Xlo`, `Xhi`...). `brbs 0x126,Zflg` jumps to 0x126 when the flag Z is set (`breq`), `brbc` when it is clear (`brne`); `Cflg` is the carry. `lds R24,0x0106` loads the byte of the memory at 0x0106, `sts 0x0106,R24` stores it. A function is called with its first argument in r24 (r25:r24 for two bytes, r25:r22 for four) and its second in r22; it returns in r24 (r25:r24, or r25:r22 for a long). A comparison of two or four bytes is `cp`/`sub` on the low byte then `cpc`/`sbc` on the others. Addresses 0x0100 and above are the memory.

What makes the STEP hold: it starts from any memory the lines allow, so the lines must say enough. Give every state of the machine an expression, so that exactly one is true; bound each variable of the code with an invariant; say how the bytes give each output (`"word(0x0108) == 1 is led"`); relate every time the code keeps to the `time in` of the state that waits.
"#;

/// The binary a model is proposed for: what was found in its machine code.
struct Binary {
    /// The Intel HEX file, as it is.
    hex: String,
    /// The correspondence as `arclang recover` starts it, and the listing of `setup` and `loop`.
    started: String,
    listing: String,
    /// The functions found, which a proposal names as they are.
    functions: Vec<(String, u32)>,
}

/// Read a binary for a proposer: its translation, where its functions are, the listing of its loop.
fn read_binary(input: &Path, file_name: &str, work: &Path) -> Result<Binary, String> {
    use crate::compiler::recover::{binary, binary_shape};
    let hex = std::fs::read_to_string(input).map_err(|e| format!("cannot read {}: {e}", input.display()))?;
    let image = binary::intel_hex(&hex).map_err(|e| format!("{file_name}: {e}"))?;
    let lifted = binary::lift(&image, &[], &work.join("lift")).map_err(|e| format!("{file_name}: {e}"))?;
    let (start, functions) = binary_shape::facts(&lifted)?;
    let proposal = binary_shape::propose(start, &functions);
    if !proposal.functions.iter().any(|(name, _, _)| name == "loop") {
        return Err(format!("{file_name}: its loop() was not found in the machine code ({}): there is nothing to propose a model of", proposal.unknown.join("; ")));
    }
    Ok(Binary {
        hex,
        started: binary_shape::written(file_name, image.bytes.len(), &proposal),
        listing: binary_shape::listing(&lifted, &proposal),
        functions: proposal.functions.iter().map(|(name, at, _)| (name.clone(), *at)).collect(),
    })
}

pub fn run(sketch: &Path, options: Options) -> Result<(), CliError> {
    let fail = |message: String| CliError::Compilation(message);
    let proposer = options.with.clone().or_else(|| std::env::var("ARCLANG_PROPOSER").ok().filter(|c| !c.trim().is_empty())).ok_or_else(|| {
        fail("--propose needs a proposer: --with \"<command>\" (or ARCLANG_PROPOSER), a command that reads the prompt on its input and prints a model; the sketch is sent to that command and to nothing else".to_string())
    })?;
    let is_binary = sketch.extension().is_some_and(|e| e == "hex");
    let code = if is_binary { String::new() } else { std::fs::read_to_string(sketch).map_err(CliError::Io)? };
    let file_name = sketch.file_name().map(|n| n.to_string_lossy().into_owned()).ok_or_else(|| fail(format!("{} is not a file", sketch.display())))?;
    let output = options.output.clone().unwrap_or_else(|| sketch.with_extension("arc"));
    if options.output.is_none() && output.exists() {
        return Err(fail(format!("{} exists: name the file to write with -o", output.display())));
    }
    let work = options.keep.clone().unwrap_or_else(|| std::env::temp_dir().join(format!("arclang-propose-{}-{}", std::process::id(), crate::compiler::simulation::tool::run_number())));
    std::fs::create_dir_all(&work).map_err(CliError::Io)?;
    let binary = if is_binary { Some(read_binary(sketch, &file_name, &work)) } else { None };
    let outcome = match binary {
        Some(Err(why)) => Err(why),
        Some(Ok(binary)) => turns(&proposer, &file_name, &code, Some(&binary), &work, &options),
        None => turns(&proposer, &file_name, &code, None, &work, &options),
    };
    if options.keep.is_none() {
        let _ = std::fs::remove_dir_all(&work);
    }
    let (model, said) = outcome.map_err(fail)?;
    let header = format!(
        "// Recovered from {file_name}: proposed by `{}`, proven by ArcLang.\n// Replay the proof, with no AI: arclang prove {}\n",
        proposer.replace('\n', " "),
        output.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
    );
    std::fs::write(&output, format!("{header}{model}")).map_err(CliError::Io)?;
    println!("{said}");
    println!("  Output: {}", output.display());
    if output.parent().map(Path::to_path_buf).unwrap_or_default() != sketch.parent().map(Path::to_path_buf).unwrap_or_default() {
        println!("  note: the correspondence names {file_name} beside the model: keep the two together to replay the proof");
    }
    Ok(())
}

/// Ask, judge, tell, for as many turns as allowed: the proven model and what
/// the proof said, or why none was proven.
fn turns(proposer: &str, file_name: &str, code: &str, binary: Option<&Binary>, work: &Path, options: &Options) -> Result<(String, String), String> {
    // The same task each turn: for a binary, what was found in its machine code and the listing of its loop.
    let task = match binary {
        None => format!("{BRIEF}\nThe sketch, `{file_name}`:\n```c\n{code}\n```\n"),
        Some(binary) => {
            let (from, to) = (BRIEF.find("The state machine:").ok_or("the brief has no state machine")?, BRIEF.find("The correspondence of that machine").ok_or("the brief has no correspondence")?);
            format!("{BINARY_INTRO}\n{}{BINARY_CORRESPONDENCE}\nWhat `arclang recover` found in `{file_name}`, the correspondence started:\n```arclang\n{}```\n\nThe machine code of its setup and of its loop:\n```\n{}```\n", &BRIEF[from..to], binary.started, binary.listing)
        }
    };
    let mut prompt = task.clone();
    let mut last = String::new();
    for turn in 1..=options.turns.max(1) {
        eprintln!("Turn {turn} of {}: asking the proposer", options.turns.max(1));
        let answer = ask(proposer, &prompt)?;
        let dir = work.join(format!("turn-{turn}"));
        std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
        std::fs::write(dir.join("answer.txt"), &answer).map_err(|e| e.to_string())?;
        let verdict = match model_of(&answer) {
            None => Err("your answer holds no fenced block with a state_machine and a correspondence".to_string()),
            Some(model) => judge(&model, file_name, code, binary, &dir, options.timeout).map(|said| (model.clone(), said)).map_err(|why| format!("{why}\n\nYour model was:\n```arclang\n{model}\n```")),
        };
        match verdict {
            Ok(proven) => return Ok(proven),
            Err(why) => {
                eprintln!("  not proven: {}", why.lines().next().unwrap_or_default());
                last = why.lines().take_while(|line| !line.starts_with("Your model was")).collect::<Vec<_>>().join("\n").trim().to_string();
                prompt = format!("{task}\nYour previous answer was not proven.\n{why}\n\nAnswer again with the whole corrected model.\n");
            }
        }
    }
    Err(format!("no model of {file_name} was proven in {} turn(s); nothing is written. The last answer failed with:\n{last}", options.turns.max(1)))
}

/// The proposal compiled and proven: what the proof printed, or what to tell the proposer.
fn judge(model: &str, file_name: &str, code: &str, binary: Option<&Binary>, dir: &Path, seconds: u64) -> Result<String, String> {
    let compiled = crate::Compiler::new(crate::CompilerConfig::default()).compile_string(model).map_err(|error| format!("The model does not compile: {error}"))?;
    let [declaration] = compiled.ast.correspondences.as_slice() else {
        return Err(format!("The model declares {} correspondence(s): it must declare exactly one", compiled.ast.correspondences.len()));
    };
    // The proof is of the sketch that was given, whatever the proposal names.
    let read = correspondence::read(declaration)?;
    match (binary, &read.binary) {
        (None, _) if read.sources != [file_name.to_string()] => return Err(format!("The correspondence names the source {:?}: it must be exactly \"{file_name}\"", read.sources)),
        (None, _) => {}
        (Some(_), None) => return Err(format!("The correspondence names no binary: it must have `binary: \"{file_name}\"`")),
        (Some(_), Some(named)) if named.file != file_name => return Err(format!("The correspondence names the binary \"{}\": it must be exactly \"{file_name}\"", named.file)),
        (Some(found), Some(named)) => {
            // A function that is named is not run in the proof: only those found in the machine code are named,
            // where they were found. A proposal cannot put code out of the proof by calling it an Arduino function.
            let said = |functions: &[(String, u32)]| functions.iter().map(|(name, at)| format!("\"{name} at 0x{at:04x}\"")).collect::<Vec<_>>().join(", ");
            let (mut given, mut expected) = (named.functions.clone(), found.functions.clone());
            given.sort();
            expected.sort();
            if given != expected {
                return Err(format!("The correspondence names the functions [{}]: they must be exactly those found in the machine code, [{}]", said(&named.functions), said(&found.functions)));
            }
        }
    }
    let code = binary.map(|binary| binary.hex.as_str()).unwrap_or(code);
    std::fs::write(dir.join(file_name), code).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("model.arc"), model).map_err(|e| e.to_string())?;
    let outcomes = correspondence::prove(&compiled.ast, dir, None, &dir.join("proof"), seconds)?;
    let (_, outcome) = outcomes.first().ok_or("no proof was run")?;
    let of = format!("{file_name} is state_machine \"{}\"", read.machine);
    match outcome {
        Outcome::Proven { seconds, period_us, int_bits, unrelated, board_clock, interrupts } => {
            let mut said = format!("✓ Proven ({seconds:.1} s) {of}\n    for one loop() a step of {} ms, an int of {int_bits} bits and a long of 32, the pins the correspondence names, and the C subset of the Arduino language", period_us / 1000);
            if !unrelated.is_empty() {
                said.push_str(&format!("\n    the proof says nothing of the machine's {}: no line of the correspondence speaks of it", unrelated.join(", ")));
            }
            for line in read.binary.iter().flat_map(crate::compiler::recover::binary::trusted) {
                said.push_str(&format!("\n    {line}"));
            }
            for line in correspondence::interrupts_said(interrupts.as_ref()) {
                said.push_str(&format!("\n    {line}"));
            }
            for line in correspondence::board_clock_said(board_clock.as_ref()) {
                said.push_str(&format!("\n    {line}"));
            }
            Ok(said)
        }
        Outcome::Refuted { obligation, lines, case, .. } => {
            let part = if obligation == "functions" {
                "A function the correspondence names is not the Arduino function it is said to be. Its machine code, from some memory and registers, for some pin, fails this"
            } else if obligation == "start" {
                "START is refuted: after setup() and one loop(), and after the machine's start and one step, these lines do not hold"
            } else {
                "STEP is refuted: from two states for which every line holds, one loop() and one step can leave two for which these lines do not hold"
            };
            let shown = case.as_ref().map(|case| format!("\nA case that shows it, each line with the value of the sketch's side and of the machine's side (a time in a state that is not active is `-`):\n{}", case.said().iter().map(|line| format!("- {line}")).collect::<Vec<_>>().join("\n"))).unwrap_or_default();
            Err(format!("{part}:\n{}{shown}", lines.iter().map(|line| format!("- {line}")).collect::<Vec<_>>().join("\n")))
        }
        Outcome::Unknown { reason } => Err(format!("The proof gave no answer: {reason}")),
    }
}

/// The model in an answer: its last fenced block that holds a state machine,
/// or the answer itself when it is one with no fence.
fn model_of(answer: &str) -> Option<String> {
    let mut blocks = Vec::new();
    let mut open: Option<Vec<&str>> = None;
    for line in answer.lines() {
        if line.trim_start().starts_with("```") {
            match open.take() {
                Some(lines) => blocks.push(lines.join("\n")),
                None => open = Some(Vec::new()),
            }
        } else if let Some(lines) = open.as_mut() {
            lines.push(line);
        }
    }
    let is_model = |text: &str| text.contains("state_machine") && text.contains("correspondence");
    blocks.into_iter().rev().find(|block| is_model(block)).or_else(|| (!answer.contains("```") && is_model(answer)).then(|| answer.to_string())).map(|model| format!("{}\n", model.trim()))
}

/// Run the proposer with the prompt on its input; its output is its answer.
fn ask(proposer: &str, prompt: &str) -> Result<String, String> {
    let mut child = Command::new("sh").arg("-c").arg(proposer).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| format!("cannot run the proposer `{proposer}`: {e}"))?;
    let mut input = child.stdin.take().ok_or("the proposer has no input")?;
    let text = prompt.to_string();
    // Written beside the reads: a proposer that answers before it has read all would block both.
    let writer = std::thread::spawn(move || {
        let _ = input.write_all(text.as_bytes());
    });
    let mut out = child.stdout.take().ok_or("the proposer has no output")?;
    let mut err = child.stderr.take().ok_or("the proposer has no error output")?;
    let reader = std::thread::spawn(move || {
        let mut answer = String::new();
        let _ = out.read_to_string(&mut answer);
        answer
    });
    let complaints = std::thread::spawn(move || {
        let mut said = String::new();
        let _ = err.read_to_string(&mut said);
        said
    });
    let started = Instant::now();
    let status = loop {
        match child.try_wait().map_err(|e| format!("the proposer `{proposer}`: {e}"))? {
            Some(status) => break status,
            None if started.elapsed() > Duration::from_secs(PROPOSER_SECONDS) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("the proposer `{proposer}` gave no answer in {PROPOSER_SECONDS} s"));
            }
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    };
    let _ = writer.join();
    let answer = reader.join().unwrap_or_default();
    let said = complaints.join().unwrap_or_default();
    if !status.success() {
        return Err(format!("the proposer `{proposer}` failed ({status}): {}", said.trim().lines().last().unwrap_or("it said nothing")));
    }
    if answer.trim().is_empty() {
        return Err(format!("the proposer `{proposer}` answered nothing"));
    }
    Ok(answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_model_is_the_last_block_that_holds_a_machine_and_a_correspondence() {
        let answer = "Here:\n```c\nint x;\n```\nthen\n```arclang\nmodel \"A\" {}\nstate_machine \"M\" {}\ncorrespondence \"C\" {}\n```\nDone.";
        assert_eq!(model_of(answer).as_deref(), Some("model \"A\" {}\nstate_machine \"M\" {}\ncorrespondence \"C\" {}\n"));
        // An answer that is the model itself, with no fence.
        assert!(model_of("state_machine \"M\" {}\ncorrespondence \"C\" {}").is_some());
        // Prose, or a block of something else, is no model.
        assert_eq!(model_of("I cannot.\n```c\nint x;\n```"), None);
    }

    #[test]
    fn a_proposal_for_another_source_or_that_does_not_compile_is_told_why() {
        let dir = std::env::temp_dir().join(format!("arclang-propose-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let model = |source: &str| format!("model \"A\" {{ version: \"1.0.0\" }}\nstate_machine \"M\" {{\n    variables: [\"led: Boolean = false\"]\n    initial: \"S\"\n    state S {{}}\n}}\ncorrespondence \"C\" {{\n    machine: \"M\"\n    source: \"{source}\"\n    writes: [\"digitalWrite(13) is led\"]\n}}\n");
        let told = judge(&model("other.ino"), "sketch.ino", "void setup(){}\nvoid loop(){}\n", None, &dir, 10).unwrap_err();
        assert!(told.contains("it must be exactly \"sketch.ino\""), "{told}");
        let told = judge("state_machine \"M\" {", "sketch.ino", "", None, &dir, 10).unwrap_err();
        assert!(told.starts_with("The model does not compile"), "{told}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
