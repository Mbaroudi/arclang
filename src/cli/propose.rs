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
- guards use `and`, `or`, `not`, comparisons and arithmetic; actions are `name := expression`;
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
- `writes`: `digitalWrite(PIN) is <variable>` or `digitalWrite(PIN) is in <State>`; EVERY pin the sketch reads or writes must be named;
- `state`: on the left a C expression over the sketch's global variables, on the right `in <State>` (true exactly when the state is active), `time in <State>` (milliseconds the state has been held) or a variable of the machine;
- `invariant`: C expressions that always hold of the sketch's variables;
- `int: 16` when the board is an AVR one (Uno, Nano, Mega).

What makes the STEP hold: it starts from any states the lines allow, so the lines must say enough. Give every state of the machine an expression, so that exactly one is true; bound each variable of the sketch with an invariant (`x == 0 || x == 1`); say how the sketch's variables give each output (`"lit == 1 is led"`); relate every timer of the sketch to the `time in` of the state that waits.
"#;

pub fn run(sketch: &Path, options: Options) -> Result<(), CliError> {
    let fail = |message: String| CliError::Compilation(message);
    let proposer = options.with.clone().or_else(|| std::env::var("ARCLANG_PROPOSER").ok().filter(|c| !c.trim().is_empty())).ok_or_else(|| {
        fail("--propose needs a proposer: --with \"<command>\" (or ARCLANG_PROPOSER), a command that reads the prompt on its input and prints a model; the sketch is sent to that command and to nothing else".to_string())
    })?;
    let code = std::fs::read_to_string(sketch).map_err(CliError::Io)?;
    let file_name = sketch.file_name().map(|n| n.to_string_lossy().into_owned()).ok_or_else(|| fail(format!("{} is not a file", sketch.display())))?;
    let output = options.output.clone().unwrap_or_else(|| sketch.with_extension("arc"));
    if options.output.is_none() && output.exists() {
        return Err(fail(format!("{} exists: name the file to write with -o", output.display())));
    }
    let work = options.keep.clone().unwrap_or_else(|| std::env::temp_dir().join(format!("arclang-propose-{}-{}", std::process::id(), crate::compiler::simulation::tool::run_number())));
    std::fs::create_dir_all(&work).map_err(CliError::Io)?;
    let outcome = turns(&proposer, &file_name, &code, &work, &options);
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
fn turns(proposer: &str, file_name: &str, code: &str, work: &Path, options: &Options) -> Result<(String, String), String> {
    let mut prompt = format!("{BRIEF}\nThe sketch, `{file_name}`:\n```c\n{code}\n```\n");
    let mut last = String::new();
    for turn in 1..=options.turns.max(1) {
        eprintln!("Turn {turn} of {}: asking the proposer", options.turns.max(1));
        let answer = ask(proposer, &prompt)?;
        let dir = work.join(format!("turn-{turn}"));
        std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
        std::fs::write(dir.join("answer.txt"), &answer).map_err(|e| e.to_string())?;
        let verdict = match model_of(&answer) {
            None => Err("your answer holds no fenced block with a state_machine and a correspondence".to_string()),
            Some(model) => judge(&model, file_name, code, &dir, options.timeout).map(|said| (model.clone(), said)).map_err(|why| format!("{why}\n\nYour model was:\n```arclang\n{model}\n```")),
        };
        match verdict {
            Ok(proven) => return Ok(proven),
            Err(why) => {
                eprintln!("  not proven: {}", why.lines().next().unwrap_or_default());
                last = why.lines().take_while(|line| !line.starts_with("Your model was")).collect::<Vec<_>>().join("\n").trim().to_string();
                prompt = format!("{BRIEF}\nThe sketch, `{file_name}`:\n```c\n{code}\n```\n\nYour previous answer was not proven.\n{why}\n\nAnswer again with the whole corrected model.\n");
            }
        }
    }
    Err(format!("no model of {file_name} was proven in {} turn(s); nothing is written. The last answer failed with:\n{last}", options.turns.max(1)))
}

/// The proposal compiled and proven: what the proof printed, or what to tell the proposer.
fn judge(model: &str, file_name: &str, code: &str, dir: &Path, seconds: u64) -> Result<String, String> {
    let compiled = crate::Compiler::new(crate::CompilerConfig::default()).compile_string(model).map_err(|error| format!("The model does not compile: {error}"))?;
    let [declaration] = compiled.ast.correspondences.as_slice() else {
        return Err(format!("The model declares {} correspondence(s): it must declare exactly one", compiled.ast.correspondences.len()));
    };
    // The proof is of the sketch that was given, whatever the proposal names.
    let read = correspondence::read(declaration)?;
    if read.sources != [file_name.to_string()] {
        return Err(format!("The correspondence names the source {:?}: it must be exactly \"{file_name}\"", read.sources));
    }
    std::fs::write(dir.join(file_name), code).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("model.arc"), model).map_err(|e| e.to_string())?;
    let outcomes = correspondence::prove(&compiled.ast, dir, None, &dir.join("proof"), seconds)?;
    let (_, outcome) = outcomes.first().ok_or("no proof was run")?;
    let of = format!("{file_name} is state_machine \"{}\"", read.machine);
    match outcome {
        Outcome::Proven { seconds, period_us, int_bits, unrelated } => {
            let mut said = format!("✓ Proven ({seconds:.1} s) {of}\n    for one loop() a step of {} ms, an int of {int_bits} bits and a long of 32, the pins the correspondence names, and the C subset of the Arduino language", period_us / 1000);
            if !unrelated.is_empty() {
                said.push_str(&format!("\n    the proof says nothing of the machine's {}: no line of the correspondence speaks of it", unrelated.join(", ")));
            }
            Ok(said)
        }
        Outcome::Refuted { obligation, lines, case, .. } => {
            let part = if obligation == "start" {
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
        let told = judge(&model("other.ino"), "sketch.ino", "void setup(){}\nvoid loop(){}\n", &dir, 10).unwrap_err();
        assert!(told.contains("it must be exactly \"sketch.ino\""), "{told}");
        let told = judge("state_machine \"M\" {", "sketch.ino", "", &dir, 10).unwrap_err();
        assert!(told.starts_with("The model does not compile"), "{told}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
