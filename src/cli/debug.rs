//! `arclang debug`: run a state machine of a model step by step.
//!
//! In the terminal, `arclang debug model.arc --events plug,ready` enters the
//! initial states, fires the events in order and prints each step. With
//! `--dap` it speaks the Debug Adapter Protocol on stdin/stdout, as VS Code
//! runs a debugger: breakpoints on a state (when it is entered) or on a
//! transition (when it fires), the active states as the call stack, the
//! guards as conditions the reader sets, and events fired from the debug
//! console (`fire <event>`, or the event alone), or a completion step with
//! *Step Over*.

use super::CliError;
use crate::compiler::execution::{Machine, Step};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

#[derive(clap::Args, Debug)]
pub struct DebugArgs {
    /// The model (not needed with --dap: the launch request names it)
    pub input: Option<PathBuf>,
    /// The state machine to run, by name or id (default: the first)
    #[clap(long)]
    pub machine: Option<String>,
    /// Events to fire in order, comma-separated (`-` for a completion step)
    #[clap(long, value_delimiter = ',')]
    pub events: Vec<String>,
    /// Guards that hold, comma-separated, as written in the model
    #[clap(long = "set", value_delimiter = ',')]
    pub conditions: Vec<String>,
    /// Print the steps as JSON
    #[clap(long)]
    pub json: bool,
    /// Speak the Debug Adapter Protocol on stdin/stdout (VS Code)
    #[clap(long)]
    pub dap: bool,
}

fn load(path: &Path, wanted: Option<&str>) -> Result<(Machine, Vec<String>), String> {
    let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(path).map_err(|e| e.to_string())?;
    let machines = &result.ast.state_machines;
    if machines.is_empty() {
        return Err(format!("{} declares no state machine to run", path.display()));
    }
    let names: Vec<String> = machines.iter().map(|m| m.name.clone()).collect();
    let chosen = match wanted {
        Some(name) => machines.iter().find(|m| m.name == name || m.id == name).ok_or_else(|| format!("no state machine '{name}': the model has {}", names.join(", ")))?,
        None => &machines[0],
    };
    Ok((Machine::new(chosen), names))
}

fn describe(machine: &Machine, step: &Step) -> String {
    let mut out = String::new();
    let event = step.event.clone().unwrap_or_else(|| "(completion)".to_string());
    if let Some(idle) = &step.idle {
        out.push_str(&format!("{event}: nothing fired, {idle}\n"));
    } else {
        for index in &step.fired {
            if let Some(t) = machine.transitions.iter().find(|t| t.index == *index) {
                out.push_str(&format!("{event}: fired {}\n", t.label));
            }
        }
        if step.fired.is_empty() && !step.entered.is_empty() {
            out.push_str("start\n");
        }
        for action in &step.actions {
            out.push_str(&format!("  {action}\n"));
        }
    }
    out.push_str(&format!("  active: {}\n", machine.leaves().join(", ")));
    out
}

pub fn run(args: &DebugArgs) -> Result<(), CliError> {
    if args.dap {
        return Adapter::default().serve(std::io::stdin().lock(), std::io::stdout()).map_err(CliError::Io);
    }
    let input = args.input.as_ref().ok_or_else(|| CliError::Config("name the model: arclang debug model.arc --events ...".to_string()))?;
    let (mut machine, _) = load(input, args.machine.as_deref()).map_err(CliError::Compilation)?;
    for condition in &args.conditions {
        machine.conditions.insert(condition.trim().to_string(), true);
    }
    let mut steps = vec![machine.start()];
    let mut text = format!("{}\n", machine.name);
    for note in &machine.notes {
        text.push_str(&format!("note: {note}\n"));
    }
    text.push_str(&describe(&machine, &steps[0]));
    for event in &args.events {
        let event = event.trim();
        let step = machine.fire(if event == "-" { None } else { Some(event) });
        text.push_str(&describe(&machine, &step));
        steps.push(step);
    }
    if args.json {
        println!("{}", serde_json::to_string_pretty(&json!({ "machine": machine.name, "steps": steps, "active": machine.leaves() })).unwrap_or_default());
    } else {
        print!("{text}");
    }
    Ok(())
}

/// A breakpoint: a state entered, or a transition fired.
#[derive(Debug, Clone, PartialEq)]
enum Break {
    State(usize),
    Transition(usize),
}

#[derive(Default)]
struct Adapter {
    seq: i64,
    machine: Option<Machine>,
    path: Option<PathBuf>,
    source: String,
    breakpoints: Vec<(i64, Break)>,
    last: Option<Step>,
}

/// The line (1-based) that declares a state or a transition, from the text.
fn line_of_state(source: &str, name: &str) -> Option<i64> {
    let quoted = format!("\"{name}\"");
    source.lines().position(|line| {
        let words: Vec<&str> = line.split_whitespace().collect();
        matches!(words.first(), Some(&"state") | Some(&"mode")) && words.get(1).is_some_and(|w| *w == name || *w == quoted || w.trim_end_matches('{') == name)
    }).map(|i| i as i64 + 1)
}

fn line_of_transition(source: &str, from: &str, to: &str) -> Vec<i64> {
    let bare = |s: &str| s.trim_matches('"').to_string();
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let words: Vec<&str> = line.split_whitespace().collect();
            words.first() == Some(&"transition") && words.len() >= 4 && words[2] == "->" && bare(words[1]) == from && bare(words[3]) == to
        })
        .map(|(i, _)| i as i64 + 1)
        .collect()
}

impl Adapter {
    fn send(&mut self, out: &mut impl Write, mut message: Value) -> std::io::Result<()> {
        self.seq += 1;
        message["seq"] = json!(self.seq);
        let body = message.to_string();
        write!(out, "Content-Length: {}\r\n\r\n{body}", body.len())?;
        out.flush()
    }

    fn respond(&mut self, out: &mut impl Write, request: &Value, body: Value) -> std::io::Result<()> {
        let message = json!({ "type": "response", "request_seq": request["seq"], "success": true, "command": request["command"], "body": body });
        self.send(out, message)
    }

    fn refuse(&mut self, out: &mut impl Write, request: &Value, why: &str) -> std::io::Result<()> {
        let message = json!({ "type": "response", "request_seq": request["seq"], "success": false, "command": request["command"], "message": why });
        self.send(out, message)
    }

    fn event(&mut self, out: &mut impl Write, event: &str, body: Value) -> std::io::Result<()> {
        self.send(out, json!({ "type": "event", "event": event, "body": body }))
    }

    fn output(&mut self, out: &mut impl Write, text: &str) -> std::io::Result<()> {
        self.event(out, "output", json!({ "category": "console", "output": text }))
    }

    /// The state of the machine, for the workbench to light the diagram.
    fn state_event(&mut self, out: &mut impl Write) -> std::io::Result<()> {
        let Some(machine) = &self.machine else { return Ok(()) };
        let fired: Vec<String> = self.last.as_ref().map(|s| s.fired.iter().filter_map(|i| machine.transitions.iter().find(|t| t.index == *i)).map(|t| format!("{} -> {}", machine.states[t.source].path, machine.states[t.target].path)).collect()).unwrap_or_default();
        let body = json!({ "machine": machine.name, "active": machine.active.iter().map(|&s| machine.states[s].path.clone()).collect::<Vec<_>>(), "leaves": machine.leaves(), "fired": fired, "events": machine.events() });
        self.event(out, "arclang.state", body)
    }

    fn hit(&self, step: &Step) -> Option<String> {
        let machine = self.machine.as_ref()?;
        for (_, b) in &self.breakpoints {
            match b {
                Break::State(s) if step.entered.contains(&machine.states[*s].path) => return Some(format!("entered {}", machine.states[*s].path)),
                Break::Transition(t) if step.fired.contains(t) => return Some(format!("fired {}", machine.transitions.iter().find(|x| x.index == *t).map(|x| x.label.clone()).unwrap_or_default())),
                _ => {}
            }
        }
        None
    }

    fn step(&mut self, out: &mut impl Write, event: Option<&str>) -> std::io::Result<()> {
        if self.machine.is_none() {
            return self.output(out, "the machine is not running\n");
        }
        // The client shows the reason of the last stop: it runs, then stops again.
        self.event(out, "continued", json!({ "threadId": 1, "allThreadsContinued": true }))?;
        let Some(machine) = self.machine.as_mut() else { return Ok(()) };
        let step = machine.fire(event);
        let text = describe(machine, &step);
        let reason = self.hit(&step);
        self.last = Some(step);
        self.output(out, &text)?;
        self.state_event(out)?;
        let (reason, description) = match reason {
            Some(why) => ("breakpoint", why),
            None => ("step", "waiting for an event".to_string()),
        };
        self.event(out, "stopped", json!({ "reason": reason, "description": description, "threadId": 1, "allThreadsStopped": true }))
    }

    fn launch(&mut self, out: &mut impl Write, request: &Value) -> std::io::Result<()> {
        let args = &request["arguments"];
        let Some(program) = args["program"].as_str() else { return self.refuse(out, request, "launch needs a program: the .arc model") };
        let path = PathBuf::from(program);
        match load(&path, args["machine"].as_str()) {
            Err(why) => self.refuse(out, request, &why),
            Ok((mut machine, names)) => {
                self.source = std::fs::read_to_string(&path).unwrap_or_default();
                let started = machine.start();
                let mut text = format!("Running {} (state machines: {}). Fire an event in this console: its name, or `fire <event>`; `set <guard> true|false`; `events`.\n", machine.name, names.join(", "));
                for note in &machine.notes {
                    text.push_str(&format!("note: {note}\n"));
                }
                text.push_str(&describe(&machine, &started));
                self.machine = Some(machine);
                self.path = Some(path);
                self.last = Some(started);
                self.respond(out, request, json!({}))?;
                self.output(out, &text)
            }
        }
    }

    fn set_breakpoints(&mut self, out: &mut impl Write, request: &Value) -> std::io::Result<()> {
        let lines: Vec<i64> = request["arguments"]["breakpoints"].as_array().map(|a| a.iter().filter_map(|b| b["line"].as_i64()).collect()).unwrap_or_default();
        let mut verified = Vec::new();
        self.breakpoints.clear();
        let found: Vec<(i64, Option<Break>)> = lines
            .iter()
            .map(|&line| {
                let Some(machine) = &self.machine else { return (line, None) };
                let state = machine.states.iter().position(|s| line_of_state(&self.source, &s.name) == Some(line)).map(Break::State);
                let transition = machine.transitions.iter().find(|t| line_of_transition(&self.source, &machine.states[t.source].name, &machine.states[t.target].name).contains(&line)).map(|t| Break::Transition(t.index));
                (line, state.or(transition))
            })
            .collect();
        for (line, found) in found {
            match found {
                Some(b) => {
                    self.breakpoints.push((line, b));
                    verified.push(json!({ "verified": true, "line": line }));
                }
                None => verified.push(json!({ "verified": false, "line": line, "message": "not a state or a transition of the running machine" })),
            }
        }
        self.respond(out, request, json!({ "breakpoints": verified }))
    }

    fn source_ref(&self) -> Value {
        match &self.path {
            Some(path) => json!({ "name": path.file_name().and_then(|n| n.to_str()).unwrap_or("model.arc"), "path": path.to_string_lossy() }),
            None => json!({}),
        }
    }

    fn stack(&mut self, out: &mut impl Write, request: &Value) -> std::io::Result<()> {
        let Some(machine) = &self.machine else { return self.respond(out, request, json!({ "stackFrames": [], "totalFrames": 0 })) };
        let mut frames = Vec::new();
        // The active states, deepest first: each a frame at its declaration.
        let mut active: Vec<usize> = machine.active.iter().copied().collect();
        active.sort_by_key(|&s| std::cmp::Reverse(machine.states[s].path.matches('.').count()));
        for (n, s) in active.into_iter().enumerate() {
            let state = &machine.states[s];
            frames.push(json!({ "id": n + 1, "name": format!("{} {}", if state.mode { "mode" } else { "state" }, state.path), "source": self.source_ref(), "line": line_of_state(&self.source, &state.name).unwrap_or(1), "column": 1 }));
        }
        let total = frames.len();
        self.respond(out, request, json!({ "stackFrames": frames, "totalFrames": total }))
    }

    fn variables(&mut self, out: &mut impl Write, request: &Value) -> std::io::Result<()> {
        let reference = request["arguments"]["variablesReference"].as_i64().unwrap_or(0);
        let Some(machine) = &self.machine else { return self.respond(out, request, json!({ "variables": [] })) };
        let item = |name: &str, value: String| json!({ "name": name, "value": value, "variablesReference": 0 });
        let variables: Vec<Value> = match reference {
            1 => machine.leaves().into_iter().enumerate().map(|(i, leaf)| item(&format!("active[{i}]"), leaf)).collect(),
            2 => machine.guards().into_iter().map(|g| { let v = machine.conditions.get(&g).copied().unwrap_or(false); item(&g, v.to_string()) }).collect(),
            3 => machine.events().into_iter().map(|e| { let enabled = !machine.enabled(Some(&e)).is_empty(); item(&e, if enabled { "fires now".to_string() } else { "guarded".to_string() }) }).collect(),
            4 => match &self.last {
                Some(step) => step.actions.iter().enumerate().map(|(i, a)| item(&format!("{i}"), a.clone())).chain(step.idle.iter().map(|w| item("idle", w.clone()))).collect(),
                None => Vec::new(),
            },
            _ => Vec::new(),
        };
        self.respond(out, request, json!({ "variables": variables }))
    }

    fn set_variable(&mut self, out: &mut impl Write, request: &Value) -> std::io::Result<()> {
        let args = &request["arguments"];
        let (name, value) = (args["name"].as_str().unwrap_or(""), args["value"].as_str().unwrap_or("").trim().to_ascii_lowercase());
        if args["variablesReference"].as_i64() != Some(2) {
            return self.refuse(out, request, "only a condition (a guard) is set");
        }
        let Some(holds) = (match value.as_str() { "true" => Some(true), "false" => Some(false), _ => None }) else {
            return self.refuse(out, request, "a condition is true or false");
        };
        if let Some(machine) = self.machine.as_mut() {
            machine.conditions.insert(name.to_string(), holds);
        }
        self.respond(out, request, json!({ "value": holds.to_string() }))?;
        self.state_event(out)
    }

    fn evaluate(&mut self, out: &mut impl Write, request: &Value) -> std::io::Result<()> {
        let expression = request["arguments"]["expression"].as_str().unwrap_or("").trim().to_string();
        let Some(machine) = self.machine.as_mut() else { return self.refuse(out, request, "the machine is not running") };
        if expression == "events" {
            let text = format!("events now: {}", machine.events().join(", "));
            return self.respond(out, request, json!({ "result": text, "variablesReference": 0 }));
        }
        if let Some(rest) = expression.strip_prefix("set ") {
            let (guard, value) = rest.rsplit_once(' ').unwrap_or((rest, "true"));
            let holds = value.trim() != "false";
            machine.conditions.insert(guard.trim().trim_matches('"').to_string(), holds);
            let text = format!("'{}' is {holds}", guard.trim().trim_matches('"'));
            self.respond(out, request, json!({ "result": text, "variablesReference": 0 }))?;
            return self.state_event(out);
        }
        let event = expression.strip_prefix("fire ").unwrap_or(&expression).trim().trim_matches('"').to_string();
        self.respond(out, request, json!({ "result": format!("fire {event}"), "variablesReference": 0 }))?;
        self.step(out, if event.is_empty() || event == "-" { None } else { Some(&event) })
    }

    fn handle(&mut self, out: &mut impl Write, request: &Value) -> std::io::Result<bool> {
        let command = request["command"].as_str().unwrap_or("");
        match command {
            "initialize" => {
                self.respond(out, request, json!({ "supportsConfigurationDoneRequest": true, "supportsSetVariable": true, "supportsRestartRequest": false, "supportsEvaluateForHovers": false }))?;
                self.event(out, "initialized", json!({}))?;
            }
            "launch" => self.launch(out, request)?,
            "setBreakpoints" => self.set_breakpoints(out, request)?,
            "setExceptionBreakpoints" | "setFunctionBreakpoints" => self.respond(out, request, json!({ "breakpoints": [] }))?,
            "configurationDone" => {
                self.respond(out, request, json!({}))?;
                self.state_event(out)?;
                self.event(out, "stopped", json!({ "reason": "entry", "description": "initial states entered", "threadId": 1, "allThreadsStopped": true }))?;
            }
            "threads" => {
                let name = self.machine.as_ref().map(|m| m.name.clone()).unwrap_or_else(|| "machine".to_string());
                self.respond(out, request, json!({ "threads": [{ "id": 1, "name": name }] }))?;
            }
            "stackTrace" => self.stack(out, request)?,
            "scopes" => self.respond(out, request, json!({ "scopes": [
                { "name": "Active states", "variablesReference": 1, "expensive": false },
                { "name": "Conditions (guards)", "variablesReference": 2, "expensive": false },
                { "name": "Events", "variablesReference": 3, "expensive": false },
                { "name": "Last step", "variablesReference": 4, "expensive": false },
            ] }))?,
            "variables" => self.variables(out, request)?,
            "setVariable" => self.set_variable(out, request)?,
            "evaluate" => self.evaluate(out, request)?,
            "fire" => {
                let event = request["arguments"]["event"].as_str().map(str::to_string);
                self.respond(out, request, json!({}))?;
                self.step(out, event.as_deref())?;
            }
            // Step Over is a completion step; Continue fires completion
            // steps until none fires or a breakpoint stops it.
            "next" | "stepIn" | "stepOut" => {
                self.respond(out, request, json!({}))?;
                self.step(out, None)?;
            }
            "continue" => {
                self.respond(out, request, json!({ "allThreadsContinued": true }))?;
                let mut seen: BTreeSet<Vec<String>> = BTreeSet::new();
                loop {
                    let Some(machine) = self.machine.as_mut() else { break };
                    let step = machine.fire(None);
                    let leaves = machine.leaves();
                    let text = describe(machine, &step);
                    let quiet = step.fired.is_empty();
                    let reason = self.hit(&step);
                    self.last = Some(step);
                    if !quiet {
                        self.output(out, &text)?;
                    }
                    if reason.is_some() || quiet || !seen.insert(leaves) {
                        self.state_event(out)?;
                        let (r, d) = match reason { Some(w) => ("breakpoint", w), None => ("pause", "waiting for an event".to_string()) };
                        self.event(out, "stopped", json!({ "reason": r, "description": d, "threadId": 1, "allThreadsStopped": true }))?;
                        break;
                    }
                }
            }
            "pause" => {
                self.respond(out, request, json!({}))?;
                self.event(out, "stopped", json!({ "reason": "pause", "threadId": 1, "allThreadsStopped": true }))?;
            }
            "disconnect" | "terminate" => {
                self.respond(out, request, json!({}))?;
                self.event(out, "terminated", json!({}))?;
                return Ok(false);
            }
            _ => self.refuse(out, request, &format!("'{command}' is not supported by the ArcLang debugger"))?,
        }
        Ok(true)
    }

    /// Read framed requests until the client disconnects.
    fn serve(&mut self, input: impl Read, mut out: impl Write) -> std::io::Result<()> {
        let mut reader = BufReader::new(input);
        loop {
            let mut length = None;
            loop {
                let mut header = String::new();
                if reader.read_line(&mut header)? == 0 {
                    return Ok(());
                }
                let header = header.trim();
                if header.is_empty() {
                    break;
                }
                if let Some(value) = header.strip_prefix("Content-Length:") {
                    length = value.trim().parse::<usize>().ok();
                }
            }
            let Some(length) = length else { continue };
            let mut body = vec![0; length];
            reader.read_exact(&mut body)?;
            let Ok(request) = serde_json::from_slice::<Value>(&body) else { continue };
            if !self.handle(&mut out, &request)? {
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(value: Value) -> Vec<u8> {
        let body = value.to_string();
        format!("Content-Length: {}\r\n\r\n{body}", body.len()).into_bytes()
    }

    fn messages(raw: &[u8]) -> Vec<Value> {
        let text = String::from_utf8_lossy(raw);
        text.split("Content-Length: ").filter(|p| !p.is_empty()).map(|part| {
            let (_, body) = part.split_once("\r\n\r\n").expect("framed");
            serde_json::from_str(body).expect("json")
        }).collect()
    }

    #[test]
    fn a_session_breaks_on_a_state_and_a_guard_is_a_condition() {
        let dir = std::env::temp_dir().join(format!("arclang-dap-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let model = dir.join("lka.arc");
        std::fs::write(&model, "model T { name: \"t\" }\nstate_machine \"LKA modes\" {\n  initial: \"Off\"\n  mode Off {}\n  mode Armed {}\n  mode Correcting {}\n  transition Off -> Armed { trigger: \"SF-DET\" }\n  transition Armed -> Correcting { trigger: \"SF-EST\" guard: \"departure imminent\" }\n}\n").unwrap();
        let mut input = Vec::new();
        for (seq, request) in [
            json!({ "command": "initialize", "arguments": {} }),
            json!({ "command": "launch", "arguments": { "program": model.to_string_lossy() } }),
            json!({ "command": "setBreakpoints", "arguments": { "source": { "path": model.to_string_lossy() }, "breakpoints": [{ "line": 6 }, { "line": 2 }] } }),
            json!({ "command": "configurationDone" }),
            json!({ "command": "evaluate", "arguments": { "expression": "SF-DET" } }),
            json!({ "command": "evaluate", "arguments": { "expression": "fire SF-EST" } }),
            json!({ "command": "setVariable", "arguments": { "variablesReference": 2, "name": "departure imminent", "value": "true" } }),
            json!({ "command": "fire", "arguments": { "event": "SF-EST" } }),
            json!({ "command": "stackTrace", "arguments": { "threadId": 1 } }),
            json!({ "command": "disconnect" }),
        ].into_iter().enumerate() {
            let mut request = request;
            request["seq"] = json!(seq + 1);
            request["type"] = json!("request");
            input.extend(frame(request));
        }
        let mut output = Vec::new();
        Adapter::default().serve(&input[..], &mut output).unwrap();
        let all = messages(&output);
        let response = |command: &str| all.iter().find(|m| m["type"] == "response" && m["command"] == command).cloned().unwrap();
        assert_eq!(response("launch")["success"], true);
        let breakpoints = response("setBreakpoints")["body"]["breakpoints"].clone();
        assert_eq!((breakpoints[0]["verified"].clone(), breakpoints[1]["verified"].clone()), (json!(true), json!(false)), "{breakpoints}");
        let stops: Vec<(String, String)> = all.iter().filter(|m| m["event"] == "stopped").map(|m| (m["body"]["reason"].as_str().unwrap().to_string(), m["body"]["description"].as_str().unwrap_or("").to_string())).collect();
        assert_eq!(stops[0].0, "entry");
        assert_eq!(stops[1], ("step".to_string(), "waiting for an event".to_string()));
        // Guard not set: nothing fires. Set: the breakpoint on Correcting stops it.
        assert_eq!(stops[2].0, "step");
        assert!(all.iter().any(|m| m["event"] == "output" && m["body"]["output"].as_str().unwrap_or("").contains("nothing fired")));
        assert_eq!(stops[3], ("breakpoint".to_string(), "entered Correcting".to_string()));
        let frames = response("stackTrace")["body"]["stackFrames"].clone();
        assert_eq!((frames[0]["name"].as_str().unwrap(), frames[0]["line"].as_i64().unwrap()), ("mode Correcting", 6));
        let lit = all.iter().filter(|m| m["event"] == "arclang.state").last().unwrap();
        assert_eq!(lit["body"]["leaves"], json!(["Correcting"]));
        assert_eq!(lit["body"]["fired"], json!(["Armed -> Correcting"]));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
