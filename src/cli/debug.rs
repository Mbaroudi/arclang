//! `arclang debug`: run a state machine, a scenario or a functional chain
//! of a model step by step.
//!
//! In the terminal, `arclang debug model.arc --events plug,ready` runs the
//! first state machine (or `--machine`, `--scenario`, `--chain`) and prints
//! each step; a scenario sends its messages and a chain executes its steps
//! to the end. With `--dap` it speaks the Debug Adapter Protocol on
//! stdin/stdout, as VS Code runs a debugger: a breakpoint on a line stops
//! when a step reaches it (a state entered, a transition fired, a message
//! sent, a chain step executed); the call stack shows where the run is; the
//! guards and conditions are set by the reader; *Step Over* is one step;
//! events are fired from the debug console (`fire <event>`, or the event).

use super::debug_target::{load, Outcome, Target};
use super::CliError;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;

#[derive(clap::Args, Debug)]
pub struct DebugArgs {
    /// The model (not needed with --dap: the launch request names it)
    pub input: Option<PathBuf>,
    /// The state machine to run, by name or id
    #[clap(long)]
    pub machine: Option<String>,
    /// The scenario to play, by name or id
    #[clap(long)]
    pub scenario: Option<String>,
    /// The functional chain (or operational process) to run, by name or id
    #[clap(long)]
    pub chain: Option<String>,
    /// Events to fire in order, comma-separated (`-` for a completion step)
    #[clap(long, value_delimiter = ',')]
    pub events: Vec<String>,
    /// Guards and conditions that hold, comma-separated, as written in the model
    #[clap(long = "set", value_delimiter = ',')]
    pub conditions: Vec<String>,
    /// List what the model can run, as JSON
    #[clap(long)]
    pub list: bool,
    /// Print the steps as JSON
    #[clap(long)]
    pub json: bool,
    /// Speak the Debug Adapter Protocol on stdin/stdout (VS Code)
    #[clap(long)]
    pub dap: bool,
}

/// Steps a scenario or a chain runs at most from the terminal.
const TERMINAL_STEPS: usize = 1000;

pub fn run(args: &DebugArgs) -> Result<(), CliError> {
    if args.dap {
        return Adapter::default().serve(std::io::stdin().lock(), std::io::stdout()).map_err(CliError::Io);
    }
    let input = args.input.as_ref().ok_or_else(|| CliError::Config("name the model: arclang debug model.arc --events ...".to_string()))?;
    let (mut target, listed) = load(input, args.machine.as_deref(), args.scenario.as_deref(), args.chain.as_deref()).map_err(CliError::Compilation)?;
    if args.list {
        println!("{}", serde_json::to_string_pretty(&listed).unwrap_or_default());
        return Ok(());
    }
    for condition in &args.conditions {
        target.set_condition(condition, true);
    }
    let mut text = format!("{} {}\n", target.kind(), target.name());
    for note in target.notes() {
        text.push_str(&format!("note: {note}\n"));
    }
    let mut steps: Vec<Value> = Vec::new();
    let mut record = |outcome: &Outcome, text: &mut String| {
        text.push_str(&outcome.text);
        steps.push(json!({ "text": outcome.text, "lines": outcome.lines, "actions": outcome.actions, "idle": outcome.idle, "done": outcome.done }));
    };
    let started = target.start();
    record(&started, &mut text);
    if matches!(target.run, super::debug_target::Run::Machine(_)) {
        for event in &args.events {
            let event = event.trim();
            let outcome = target.step(if event == "-" { None } else { Some(event) });
            record(&outcome, &mut text);
        }
    } else {
        for _ in 0..TERMINAL_STEPS {
            let outcome = target.step(None);
            let stop = outcome.done || outcome.idle.is_some();
            record(&outcome, &mut text);
            if stop {
                break;
            }
        }
    }
    if args.json {
        println!("{}", serde_json::to_string_pretty(&json!({ "kind": target.kind(), "name": target.name(), "steps": steps })).unwrap_or_default());
    } else {
        print!("{text}");
    }
    Ok(())
}

#[derive(Default)]
struct Adapter {
    seq: i64,
    target: Option<Target>,
    path: Option<PathBuf>,
    breakpoints: Vec<i64>,
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

    /// What is running, for the workbench to light the diagram.
    fn state_event(&mut self, out: &mut impl Write) -> std::io::Result<()> {
        let Some(target) = &self.target else { return Ok(()) };
        let body = target.lit();
        self.event(out, "arclang.state", body)
    }

    fn hit(&self, outcome: &Outcome) -> Option<i64> {
        outcome.lines.iter().copied().find(|line| self.breakpoints.contains(line))
    }

    fn stop(&mut self, out: &mut impl Write, outcome: &Outcome, quiet_reason: &str) -> std::io::Result<()> {
        self.state_event(out)?;
        if outcome.done {
            self.output(out, "the run is done\n")?;
        }
        let (reason, description) = match self.hit(outcome) {
            Some(line) => ("breakpoint", format!("line {line}: {}", outcome.text.lines().next().unwrap_or("").trim())),
            None if outcome.done => ("step", "done".to_string()),
            None => ("step", quiet_reason.to_string()),
        };
        self.event(out, "stopped", json!({ "reason": reason, "description": description, "threadId": 1, "allThreadsStopped": true }))
    }

    fn step(&mut self, out: &mut impl Write, event: Option<&str>) -> std::io::Result<()> {
        if self.target.is_none() {
            return self.output(out, "nothing is running\n");
        }
        // The client shows the reason of the last stop: it runs, then stops again.
        self.event(out, "continued", json!({ "threadId": 1, "allThreadsContinued": true }))?;
        let Some(target) = self.target.as_mut() else { return Ok(()) };
        let waits = if target.events().is_empty() && !matches!(target.run, super::debug_target::Run::Machine(_)) { "next step" } else { "waiting for an event" };
        let outcome = target.step(event);
        self.output(out, &outcome.text)?;
        self.stop(out, &outcome, waits)
    }

    fn launch(&mut self, out: &mut impl Write, request: &Value) -> std::io::Result<()> {
        let args = &request["arguments"];
        let Some(program) = args["program"].as_str() else { return self.refuse(out, request, "launch needs a program: the .arc model") };
        let path = PathBuf::from(program);
        match load(&path, args["machine"].as_str(), args["scenario"].as_str(), args["chain"].as_str()) {
            Err(why) => self.refuse(out, request, &why),
            Ok((mut target, _)) => {
                let started = target.start();
                let how = match target.run {
                    super::debug_target::Run::Machine(_) => "Fire an event in this console: its name, or `fire <event>`; `set <guard> true|false`; `events`. Step Over is a completion step.",
                    _ => "Step Over (F10) runs one step, Continue runs to the end or a breakpoint; `set <condition> true|false` in this console.",
                };
                let mut text = format!("Running the {} {}. {how}\n", target.kind(), target.name());
                for note in target.notes() {
                    text.push_str(&format!("note: {note}\n"));
                }
                text.push_str(&started.text);
                self.target = Some(target);
                self.path = Some(path);
                self.respond(out, request, json!({}))?;
                self.output(out, &text)
            }
        }
    }

    fn set_breakpoints(&mut self, out: &mut impl Write, request: &Value) -> std::io::Result<()> {
        let lines: Vec<i64> = request["arguments"]["breakpoints"].as_array().map(|a| a.iter().filter_map(|b| b["line"].as_i64()).collect()).unwrap_or_default();
        let mut verified = Vec::new();
        self.breakpoints.clear();
        for line in lines {
            let ok = self.target.as_ref().is_some_and(|t| t.breakable(line));
            if ok {
                self.breakpoints.push(line);
                verified.push(json!({ "verified": true, "line": line }));
            } else {
                verified.push(json!({ "verified": false, "line": line, "message": "no step of the run reaches this line (a state, a transition, a message or a chain step)" }));
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
        let frames: Vec<Value> = match &self.target {
            Some(target) => target.frames().into_iter().enumerate().map(|(n, (name, line))| json!({ "id": n + 1, "name": name, "source": self.source_ref(), "line": line, "column": 1 })).collect(),
            None => Vec::new(),
        };
        let total = frames.len();
        self.respond(out, request, json!({ "stackFrames": frames, "totalFrames": total }))
    }

    fn scopes(&mut self, out: &mut impl Write, request: &Value) -> std::io::Result<()> {
        let first = self.target.as_ref().map(|t| t.position().0).unwrap_or("Run");
        let mut scopes = vec![json!({ "name": first, "variablesReference": 1, "expensive": false }), json!({ "name": "Conditions", "variablesReference": 2, "expensive": false })];
        if self.target.as_ref().is_some_and(|t| matches!(t.run, super::debug_target::Run::Machine(_))) {
            scopes.push(json!({ "name": "Events", "variablesReference": 3, "expensive": false }));
        }
        scopes.push(json!({ "name": "Last step", "variablesReference": 4, "expensive": false }));
        self.respond(out, request, json!({ "scopes": scopes }))
    }

    fn variables(&mut self, out: &mut impl Write, request: &Value) -> std::io::Result<()> {
        let reference = request["arguments"]["variablesReference"].as_i64().unwrap_or(0);
        let Some(target) = &self.target else { return self.respond(out, request, json!({ "variables": [] })) };
        let item = |name: &str, value: String| json!({ "name": name, "value": value, "variablesReference": 0 });
        let variables: Vec<Value> = match reference {
            1 => target.position().1.into_iter().map(|(k, v)| item(&k, v)).collect(),
            2 => target.conditions().into_iter().map(|(g, v)| item(&g, v.to_string())).collect(),
            3 => target.events().into_iter().map(|e| item(&e, "waited for".to_string())).collect(),
            4 => target.last.actions.iter().enumerate().map(|(i, a)| item(&format!("{i}"), a.clone())).chain(target.last.idle.iter().map(|w| item("idle", w.clone()))).collect(),
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
        if let Some(target) = self.target.as_mut() {
            target.set_condition(name, holds);
        }
        self.respond(out, request, json!({ "value": holds.to_string() }))?;
        self.state_event(out)
    }

    fn evaluate(&mut self, out: &mut impl Write, request: &Value) -> std::io::Result<()> {
        let expression = request["arguments"]["expression"].as_str().unwrap_or("").trim().to_string();
        let Some(target) = self.target.as_mut() else { return self.refuse(out, request, "nothing is running") };
        if expression == "events" {
            let text = format!("events now: {}", target.events().join(", "));
            return self.respond(out, request, json!({ "result": text, "variablesReference": 0 }));
        }
        if let Some(rest) = expression.strip_prefix("set ") {
            let (guard, value) = match rest.rsplit_once(' ') {
                Some((g, v)) if v == "true" || v == "false" => (g, v),
                _ => (rest, "true"),
            };
            let holds = value != "false";
            let guard = guard.trim().trim_matches('"').to_string();
            target.set_condition(&guard, holds);
            self.respond(out, request, json!({ "result": format!("'{guard}' is {holds}"), "variablesReference": 0 }))?;
            return self.state_event(out);
        }
        let event = expression.strip_prefix("fire ").unwrap_or(&expression).trim().trim_matches('"').to_string();
        self.respond(out, request, json!({ "result": format!("fire {event}"), "variablesReference": 0 }))?;
        self.step(out, if event.is_empty() || event == "-" { None } else { Some(&event) })
    }

    fn run_on(&mut self, out: &mut impl Write) -> std::io::Result<()> {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        loop {
            let Some(target) = self.target.as_mut() else { return Ok(()) };
            let machine = matches!(target.run, super::debug_target::Run::Machine(_));
            let outcome = target.step(None);
            let quiet = outcome.idle.is_some() || (machine && outcome.lines.is_empty());
            if !quiet {
                self.output(out, &outcome.text)?;
            }
            let looped = machine && !seen.insert(outcome.text.clone());
            if self.hit(&outcome).is_some() || quiet || outcome.done || looped {
                if quiet && !machine {
                    self.output(out, &outcome.text)?;
                }
                return self.stop(out, &outcome, if machine { "waiting for an event" } else { "stopped" });
            }
        }
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
                self.event(out, "stopped", json!({ "reason": "entry", "description": "started", "threadId": 1, "allThreadsStopped": true }))?;
            }
            "threads" => {
                let name = self.target.as_ref().map(|t| format!("{} {}", t.kind(), t.name())).unwrap_or_else(|| "run".to_string());
                self.respond(out, request, json!({ "threads": [{ "id": 1, "name": name }] }))?;
            }
            "stackTrace" => self.stack(out, request)?,
            "scopes" => self.scopes(out, request)?,
            "variables" => self.variables(out, request)?,
            "setVariable" => self.set_variable(out, request)?,
            "evaluate" => self.evaluate(out, request)?,
            "fire" => {
                let event = request["arguments"]["event"].as_str().map(str::to_string);
                self.respond(out, request, json!({}))?;
                self.step(out, event.as_deref())?;
            }
            // Step Over: one step (a completion step for a state machine).
            "next" | "stepIn" | "stepOut" => {
                self.respond(out, request, json!({}))?;
                self.step(out, None)?;
            }
            // Continue: steps until a breakpoint, the end, or nothing to do.
            "continue" => {
                self.respond(out, request, json!({ "allThreadsContinued": true }))?;
                self.event(out, "continued", json!({ "threadId": 1, "allThreadsContinued": true }))?;
                self.run_on(out)?;
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
        assert_eq!(stops[3].0, "breakpoint");
        assert!(stops[3].1.starts_with("line 6:"), "{stops:?}");
        let frames = response("stackTrace")["body"]["stackFrames"].clone();
        assert_eq!((frames[0]["name"].as_str().unwrap(), frames[0]["line"].as_i64().unwrap()), ("mode Correcting", 6));
        let lit = all.iter().filter(|m| m["event"] == "arclang.state").last().unwrap();
        assert_eq!(lit["body"]["leaves"], json!(["Correcting"]));
        assert_eq!(lit["body"]["nodes"], json!(["LKA modes.Correcting"]));
        assert_eq!(lit["body"]["edges"], json!(["tr:Armed -> Correcting"]));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
