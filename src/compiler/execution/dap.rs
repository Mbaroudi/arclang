//! The Debug Adapter Protocol for `arclang debug`: a state machine, a
//! scenario or a functional chain of a model run step by step from VS Code.
//! A breakpoint on a line stops when a step reaches it (a state entered, a
//! transition fired, a message sent, a chain step executed); the call stack
//! shows where the run is; the guards and conditions are set by the reader;
//! *Step Over* is one step; events are fired from the debug console.
//!
//! The desktop runs it on stdin/stdout (`arclang debug --dap`); the browser
//! build runs it in WebAssembly, one request at a time (`handle`), with the
//! model's text in the launch request.

use super::debug_target::{load, load_source, Outcome, Run, Target};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;

#[derive(Default)]
pub struct Adapter {
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
        let waits = if target.events().is_empty() && !matches!(target.run, Run::Machine(_)) { "next step" } else { "waiting for an event" };
        let outcome = target.step(event);
        self.output(out, &outcome.text)?;
        self.stop(out, &outcome, waits)
    }

    fn launch(&mut self, out: &mut impl Write, request: &Value) -> std::io::Result<()> {
        let args = &request["arguments"];
        let Some(program) = args["program"].as_str() else { return self.refuse(out, request, "launch needs a program: the .arc model") };
        let path = PathBuf::from(program);
        // The browser sends the model's text with its name: there is no file to read there.
        let loaded = match args["source"].as_str() {
            Some(source) => load_source(source, program, args["machine"].as_str(), args["scenario"].as_str(), args["chain"].as_str()),
            None => load(&path, args["machine"].as_str(), args["scenario"].as_str(), args["chain"].as_str()),
        };
        match loaded {
            Err(why) => self.refuse(out, request, &why),
            Ok((mut target, _)) => {
                let started = target.start();
                let how = match target.run {
                    Run::Machine(_) => "Fire an event in this console: its name, or `fire <event>`; `set <guard> true|false`; `events`. Step Over is a completion step.",
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
        if self.target.as_ref().is_some_and(|t| matches!(t.run, Run::Machine(_))) {
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
            let machine = matches!(target.run, Run::Machine(_));
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

    /// Answers one request (its responses and events written to `out`); false once the client disconnects.
    pub fn handle(&mut self, out: &mut impl Write, request: &Value) -> std::io::Result<bool> {
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

    /// One request as JSON, its responses and events as a JSON array, unframed:
    /// the browser's inline debug adapter. `done` once the client disconnects.
    pub fn handle_json(&mut self, request: &str) -> Value {
        let Ok(request) = serde_json::from_str::<Value>(request) else { return json!({ "messages": [], "done": false }) };
        let mut framed = Vec::new();
        let done = !self.handle(&mut framed, &request).unwrap_or(false);
        json!({ "messages": unframe(&framed), "done": done })
    }

    /// Read framed requests until the client disconnects.
    pub fn serve(&mut self, input: impl Read, mut out: impl Write) -> std::io::Result<()> {
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

/// The messages of framed output (`Content-Length` counts bytes, not characters).
fn unframe(raw: &[u8]) -> Vec<Value> {
    let mut messages = Vec::new();
    let mut rest = raw;
    while let Some(start) = rest.windows(4).position(|w| w == b"\r\n\r\n") {
        let header = String::from_utf8_lossy(&rest[..start]);
        let Some(length) = header.trim().strip_prefix("Content-Length:").and_then(|v| v.trim().parse::<usize>().ok()) else { break };
        let body = &rest[start + 4..];
        if body.len() < length {
            break;
        }
        if let Ok(message) = serde_json::from_slice(&body[..length]) {
            messages.push(message);
        }
        rest = &body[length..];
    }
    messages
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(value: Value) -> Vec<u8> {
        let body = value.to_string();
        format!("Content-Length: {}\r\n\r\n{body}", body.len()).into_bytes()
    }

    fn messages(raw: &[u8]) -> Vec<Value> {
        unframe(raw)
    }

    #[test]
    fn the_browser_runs_a_session_from_the_model_text() {
        let source = "model T { name: \"t\" }\nstate_machine \"Modes\" {\n  initial: \"Off\"\n  mode Off {}\n  mode Armé {}\n  transition Off -> Armé { trigger: \"GO\" }\n}\n";
        let mut adapter = Adapter::default();
        let ask = |adapter: &mut Adapter, seq: i64, command: &str, arguments: Value| adapter.handle_json(&json!({ "seq": seq, "type": "request", "command": command, "arguments": arguments }).to_string());
        assert_eq!(ask(&mut adapter, 1, "initialize", json!({}))["messages"][1]["event"], "initialized");
        let launched = ask(&mut adapter, 2, "launch", json!({ "program": "/workspace/m.arc", "source": source }));
        assert_eq!(launched["messages"][0]["success"], true, "{launched}");
        ask(&mut adapter, 3, "configurationDone", json!({}));
        let fired = ask(&mut adapter, 4, "evaluate", json!({ "expression": "GO" }));
        let lit = fired["messages"].as_array().unwrap().iter().filter(|m| m["event"] == "arclang.state").last().cloned().unwrap();
        assert_eq!(lit["body"]["leaves"], json!(["Armé"]));
        let frames = ask(&mut adapter, 5, "stackTrace", json!({}))["messages"][0]["body"]["stackFrames"].clone();
        assert_eq!((frames[0]["line"].as_i64(), frames[0]["source"]["path"].as_str()), (Some(5), Some("/workspace/m.arc")));
        assert_eq!(ask(&mut adapter, 6, "disconnect", json!({}))["done"], true);
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
