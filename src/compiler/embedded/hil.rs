//! Hardware in the loop: the generated C on a board, driven through its
//! serial port with the input vectors of each `execute:` case, its outputs
//! read back step by step and judged as SIL judges the C on this computer
//! (compared with the model-level run, the case's `expects:` on both).
//!
//! The board runs the direct harness (`host/harness.c` of `export -f c`)
//! behind its serial port as stdio, and says `# arclang hil ready <model>
//! <board>` whenever it waits for a case: it then reads the number of steps
//! and one line of inputs per step, and prints one CSV row of the outputs
//! per step (lines starting with `#` are comments). A program on this
//! computer can stand for the board (`--command`): the same protocol on
//! its stdin and stdout, which is how the command is tested without a
//! board.

use super::plan::Plan;
use super::sil::{self, Outcome};
use crate::compiler::ast::{Model, TestCase};
use crate::compiler::simulation::cache;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

/// What the board prints when it waits for a case, followed by the
/// model's identifier and the board's name.
pub const READY: &str = "# arclang hil ready";

/// The kind of the cached verdict (`<case>.hil.json`).
pub const KIND: &str = "hil";

#[derive(Debug, Clone)]
pub struct Options {
    /// The board's serial port.
    pub port: Option<String>,
    /// A program standing for the board, run by `sh -c`.
    pub command: Option<String>,
    pub baud: u32,
    /// Only this case (id or name).
    pub only: Option<String>,
    /// How long to wait for the board to be ready, and for each line it answers.
    pub timeout: Duration,
}

/// One connection to the board: where to write, the lines it prints, and
/// the process when a program stands for it.
struct Session {
    writer: Box<dyn Write + Send>,
    lines: Receiver<String>,
    child: Option<Child>,
}

/// The board, as the command talks to it.
pub struct Board {
    options: Options,
    session: Option<Session>,
    /// Whether the session already said it is ready for the next case.
    ready: bool,
    /// The model the board holds, as the ready line names it.
    pub ident: String,
    /// The board's name, as the ready line gives it (`arduino-uno`).
    pub name: String,
}

impl Board {
    /// Open the port (or start the program) and wait for the board to be
    /// ready. A port that goes away meanwhile (the board re-enumerating
    /// after a flash or a reset) is opened again, within the timeout.
    pub fn open(options: &Options) -> Result<Board, String> {
        let mut board = Board { options: options.clone(), session: None, ready: false, ident: String::new(), name: String::new() };
        let deadline = Instant::now() + options.timeout;
        loop {
            let attempt = board.start().and_then(|session| wait_ready(&session, deadline.saturating_duration_since(Instant::now())).map(|found| (session, found)));
            match attempt {
                Ok((session, (ident, name))) => {
                    board.ident = ident;
                    board.name = name;
                    board.ready = true;
                    board.session = Some(session);
                    return Ok(board);
                }
                Err(e) if board.options.port.is_some() && e.contains("went away") && Instant::now() + Duration::from_millis(600) < deadline => {
                    std::thread::sleep(Duration::from_millis(500));
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Where the board is: the port, or the program.
    pub fn place(&self) -> String {
        match (&self.options.port, &self.options.command) {
            (Some(port), _) => port.clone(),
            (None, Some(command)) => format!("program `{command}`"),
            (None, None) => String::new(),
        }
    }

    fn start(&self) -> Result<Session, String> {
        match (&self.options.port, &self.options.command) {
            (Some(port), _) => open_serial(port, self.options.baud),
            (None, Some(command)) => start_program(command),
            (None, None) => Err("give the board's serial port (--port) or a program standing for it (--command)".to_string()),
        }
    }

    /// Send a case's lines (the number of steps, then one per step) and read
    /// the board's rows, one per step, as the text `parse_rows` reads.
    pub fn exchange(&mut self, ident: &str, stdin: &str, steps: usize) -> Result<String, String> {
        let timeout = self.options.timeout;
        let mut session = match self.session.take() {
            Some(session) => session,
            None => {
                let session = self.start()?;
                let (found, name) = wait_ready(&session, timeout)?;
                self.ident = found;
                self.name = name;
                self.ready = true;
                session
            }
        };
        if !self.ready {
            let (found, name) = wait_ready(&session, timeout)?;
            self.ident = found;
            self.name = name;
        }
        self.ready = false;
        if self.ident != ident {
            self.finish(session);
            return Err(format!("the board holds the model '{}', not '{ident}': flash it with the C of this model", self.ident));
        }
        let exchanged = (|| -> Result<String, String> {
            session.writer.write_all(stdin.as_bytes()).and_then(|()| session.writer.flush()).map_err(|e| format!("cannot write to the board: {e}"))?;
            let mut rows = String::new();
            let mut received = 0usize;
            let started = Instant::now();
            while received < steps {
                let line = match session.lines.recv_timeout(timeout) {
                    Ok(line) => line,
                    Err(mpsc::RecvTimeoutError::Timeout) => return Err(format!("the board gave {received} of {steps} rows, then nothing for {} s", timeout.as_secs())),
                    Err(mpsc::RecvTimeoutError::Disconnected) => return Err(format!("the board went away after {received} of {steps} rows")),
                };
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some(error) = line.strip_prefix("ERROR ") {
                    return Err(error.to_string());
                }
                rows.push_str(line);
                rows.push('\n');
                received += 1;
            }
            let _ = started;
            Ok(rows)
        })();
        self.finish(session);
        exchanged
    }

    /// After a case: a program is done (the next case starts another), its
    /// stdin closed first so that a harness still waiting for input exits,
    /// killed if it does not within a second; the port stays open, the board
    /// will say it is ready again.
    fn finish(&mut self, session: Session) {
        let Session { writer, lines, child } = session;
        match child {
            Some(mut child) => {
                drop(writer);
                drop(lines);
                let deadline = Instant::now() + Duration::from_secs(1);
                loop {
                    match child.try_wait() {
                        Ok(Some(_)) => break,
                        Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
                        _ => {
                            let _ = child.kill();
                            let _ = child.wait();
                            break;
                        }
                    }
                }
            }
            None => self.session = Some(Session { writer, lines, child: None }),
        }
    }
}

/// The lines the board prints, from a reader, on a channel (the reader
/// blocks; the command waits with a timeout).
fn lines_of<R: std::io::Read + Send + 'static>(reader: R) -> Receiver<String> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    if sender.send(line.trim_end_matches(['\r', '\n']).to_string()).is_err() {
                        break;
                    }
                }
            }
        }
    });
    receiver
}

/// Open the serial port raw at the speed; opening it resets most boards.
fn open_serial(port: &str, baud: u32) -> Result<Session, String> {
    let file = std::fs::OpenOptions::new().read(true).write(true).open(port).map_err(|e| format!("cannot open the serial port {port}: {e}"))?;
    let flag = if cfg!(target_os = "macos") { "-f" } else { "-F" };
    let stty = Command::new("stty").arg(flag).arg(port).args([baud.to_string().as_str(), "raw", "-echo", "cs8", "-crtscts", "clocal"]).output().map_err(|e| format!("cannot run stty for {port}: {e}"))?;
    if !stty.status.success() {
        return Err(format!("cannot set the serial port {port} to {baud} baud: {}", String::from_utf8_lossy(&stty.stderr).trim()));
    }
    let writer = file.try_clone().map_err(|e| format!("cannot open the serial port {port} for writing: {e}"))?;
    Ok(Session { writer: Box::new(writer), lines: lines_of(file), child: None })
}

/// Start the program standing for the board, its stdin and stdout piped.
fn start_program(command: &str) -> Result<Session, String> {
    let mut child = Command::new("sh").arg("-c").arg(command).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn().map_err(|e| format!("cannot run `{command}`: {e}"))?;
    let stdin = child.stdin.take().expect("piped");
    let stdout = child.stdout.take().expect("piped");
    Ok(Session { writer: Box::new(stdin), lines: lines_of(stdout), child: Some(child) })
}

/// Wait for the ready line; what comes before it (the boot of the board)
/// is ignored. Gives the model's identifier and the board's name.
fn wait_ready(session: &Session, timeout: Duration) -> Result<(String, String), String> {
    let deadline = Instant::now() + timeout;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(format!("the board did not say `{READY} <model> <board>` within {} s: is the harness flashed, is the port the right one?", timeout.as_secs()));
        }
        match session.lines.recv_timeout(left) {
            Ok(line) => {
                if let Some(rest) = line.trim().strip_prefix(READY) {
                    let mut words = rest.split_whitespace();
                    let ident = words.next().unwrap_or_default().to_string();
                    let name = words.collect::<Vec<_>>().join(" ");
                    if ident.is_empty() {
                        return Err(format!("the board's ready line names no model: '{line}'"));
                    }
                    return Ok((ident, if name.is_empty() { "board".to_string() } else { name }));
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err("the board went away before it was ready".to_string()),
        }
    }
}

/// Run every executed case of a compiled model (or `options.only`) on the board.
pub fn run(ast: &Model, options: &Options) -> Result<Vec<Outcome>, String> {
    run_with(ast, options, &mut super::plant::Plants::none())
}

/// As `run`, with the plants of the cases (a case with a plant is not run on
/// a board yet and says so).
pub fn run_with(ast: &Model, options: &Options, plants: &mut super::plant::Plants) -> Result<Vec<Outcome>, String> {
    let plan = super::plan::plan(ast)?;
    let bindings = super::binding::resolve_all(ast, &plan)?;
    let selected: Vec<&TestCase> = sil::cases(ast).into_iter().filter(|t| options.only.as_deref().is_none_or(|o| t.id == o || t.name == o)).collect();
    if selected.is_empty() {
        return Err(match &options.only {
            Some(only) => format!("no test_case '{only}' that executes (execute: {{ ... }})"),
            None => "the model has no test_case that executes (execute: { machine: ... stop_time: ... inputs: [...] })".to_string(),
        });
    }
    let mut board = Board::open(options)?;
    Ok(selected.into_iter().map(|t| run_case(t, &plan, &bindings, &mut board, ast, plants)).collect())
}

/// Open the board and run the cases; the board's name and place for the report.
pub fn run_reporting(ast: &Model, options: &Options, plants: &mut super::plant::Plants) -> Result<(Vec<Outcome>, String, String), String> {
    let plan = super::plan::plan(ast)?;
    let bindings = super::binding::resolve_all(ast, &plan)?;
    let selected: Vec<&TestCase> = sil::cases(ast).into_iter().filter(|t| options.only.as_deref().is_none_or(|o| t.id == o || t.name == o)).collect();
    if selected.is_empty() {
        return Err(match &options.only {
            Some(only) => format!("no test_case '{only}' that executes (execute: {{ ... }})"),
            None => "the model has no test_case that executes (execute: { machine: ... stop_time: ... inputs: [...] })".to_string(),
        });
    }
    let mut board = Board::open(options)?;
    let outcomes = selected.into_iter().map(|t| run_case(t, &plan, &bindings, &mut board, ast, plants)).collect();
    Ok((outcomes, board.name.clone(), board.place()))
}

fn run_case(test: &TestCase, plan: &Plan, bindings: &[super::binding::Binding], board: &mut Board, ast: &Model, plants: &mut super::plant::Plants) -> Outcome {
    let implementation = format!("{} on {}", board.name, board.place());
    let prepared = match sil::prepare(test, plan, bindings, &implementation, ast, plants) {
        Ok(p) => p,
        Err(outcome) => return *outcome,
    };
    if prepared.case.plant.is_some() {
        let mut outcome = prepared.outcome;
        outcome.reason = "a case with a plant is not run on a board yet: run it in SIL or PIL".to_string();
        return outcome;
    }
    if prepared.binding.is_some() {
        let mut outcome = prepared.outcome;
        outcome.reason = "a case through an io_binding is not run on a board yet (the board's channels are not wired to its pins): run it in SIL or PIL".to_string();
        return outcome;
    }
    let stdin = sil::direct_stdin(plan, &prepared.vectors);
    let steps = prepared.case.steps;
    let started = Instant::now();
    match board.exchange(&plan.ident, &stdin, steps).and_then(|rows| sil::parse_rows(&rows, &prepared.model.names, prepared.case.step_us)) {
        Ok(table) => {
            let elapsed = started.elapsed();
            let mut outcome = sil::conclude(prepared, table);
            outcome.timing = Some(format!("{steps} steps answered by the board in {:.2} s ({:.1} ms a step, the serial line included)", elapsed.as_secs_f64(), elapsed.as_secs_f64() * 1000.0 / steps.max(1) as f64));
            outcome
        }
        Err(e) => {
            let mut outcome = prepared.outcome;
            outcome.reason = e;
            outcome
        }
    }
}

/// Keep the verdicts beside the model as `<case>.hil.json` (kind `hil`,
/// the board as target), where `check`, `gate`, the tables and the
/// realization ladder read them. Fresh while the generated C and the case
/// are those of the run: whether the board still holds that C is for the
/// one who flashes it to know.
pub fn cache(ast: &Model, model_path: &Path, outcomes: &[Outcome], board: &str) -> Result<Vec<PathBuf>, String> {
    let dir = cache::dir(model_path);
    let base = cache::model_dir(model_path);
    let mut written = Vec::new();
    for test in sil::cases(ast) {
        let runs: Vec<&Outcome> = outcomes.iter().filter(|o| o.test_case == test.id).collect();
        if runs.is_empty() {
            continue;
        }
        let mut outcome = sil::cached_outcome(&runs, sil::cache_key(ast, test, &base)?);
        outcome.kind = KIND.to_string();
        outcome.target = board.to_string();
        outcome.tool = format!("arclang hil (on the board {})", runs[0].compiler);
        if outcome.verdict != "pass" {
            outcome.reason = outcome.reason.replacen("SIL with", "on the board", 1);
        }
        written.push(cache::write_kind(&dir, KIND, &outcome)?);
    }
    Ok(written)
}
