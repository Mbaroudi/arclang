//! `io_binding`: the events and conditions of a state machine read from I/O
//! channels, its states and actions written to them, written once in the
//! model and checked by the compiler.
//!
//! ```text
//! io_binding "ACC ECU" {
//!     machine: "ACC operating modes"
//!     period: 10 ms
//!     inputs: ["LF-022 <- gpio 4 rising", "main switch on <- gpio 3",
//!              "set pressed above 30 km/h <- adc 0 > 2048", "LF-009 <- can 0x120"]
//!     outputs: ["Following -> gpio 10", "Degraded -> can 0x300"]
//! }
//! ```
//!
//! An input binds an event (an instant: a `gpio` edge, a `can` frame
//! received, a `uart` byte received) or a condition (a level: a `gpio`
//! level, an `adc` compared with a raw threshold, the first byte of the last
//! `can` frame not zero). An output writes a state (1 while active) or an
//! action (the times it ran in the step) to a `gpio`, a `pwm` duty (per
//! mille, while not zero), a `dac` value (while not zero), a `can` frame
//! (its first byte the value, every period) or a `uart` byte (each step
//! the value is not zero).

use super::plan::{c_name, MachinePlan, Plan};
use crate::compiler::ast::{AttributeValue, IoBinding, Model};
use crate::compiler::quantity::Dimension;
use std::collections::BTreeMap;

pub const KEYS: [&str; 9] = ["id", "description", "machine", "period", "inputs", "outputs", "watchdog", "target", "priority"];

/// Priorities a binding may ask for: 1 is the highest.
pub const PRIORITIES: std::ops::RangeInclusive<u32> = 1..=32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Rising,
    Falling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    GpioLevel(u16),
    GpioEdge(u16, Edge),
    /// Holds when the raw value is above (`true`) or below the threshold.
    Adc { channel: u16, above: bool, threshold: u16 },
    Can(u32),
    Uart { port: u16, byte: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sink {
    Gpio(u16),
    Pwm(u16, u16),
    Dac(u16, u16),
    Can(u32),
    Uart(u16, u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Event(usize),
    Condition(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    State(usize),
    Action(usize),
}

#[derive(Debug, Clone)]
pub struct Input {
    pub written: String,
    pub target: Target,
    pub source: Source,
}

#[derive(Debug, Clone)]
pub struct Output {
    pub written: String,
    pub origin: Origin,
    pub sink: Sink,
}

/// A binding resolved against its machine.
#[derive(Debug, Clone)]
pub struct Binding {
    pub name: String,
    pub id: String,
    pub ident: String,
    /// Index of the machine in the plan.
    pub machine: usize,
    pub period_us: u64,
    pub watchdog: bool,
    /// `priority:` as written (1 the highest); none: by the period (rate monotonic), see `priorities`.
    pub priority: Option<u32>,
    /// The target package it is deployed on (`target:`, `host` by default).
    pub target: String,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    /// What the binding leaves unbound.
    pub warnings: Vec<String>,
}

impl Binding {
    pub fn is(&self, reference: &str) -> bool {
        self.name == reference || self.id == reference
    }

    pub fn input_of(&self, target: Target) -> Option<&Input> {
        self.inputs.iter().find(|i| i.target == target)
    }
}

fn number(text: &str, what: &str, max: u64) -> Result<u64, String> {
    let text = text.trim();
    let value = match text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        Some(hex) => u64::from_str_radix(hex, 16),
        None => text.parse::<u64>(),
    }
    .map_err(|_| format!("{what} '{text}' is not a whole number (decimal or 0x hexadecimal)"))?;
    if value > max {
        return Err(format!("{what} {text} is above {max}"));
    }
    Ok(value)
}

fn channel(words: &[&str], at: usize, what: &str) -> Result<u16, String> {
    let word = words.get(at).ok_or_else(|| format!("{} needs {what}", words[0]))?;
    number(word, what, u16::MAX as u64).map(|n| n as u16)
}

const CAN_MAX_ID: u64 = 0x1FFF_FFFF;

/// `gpio 4`, `gpio 4 rising`, `adc 0 > 2048`, `can 0x120`, `uart 1 0x41`.
pub fn parse_source(text: &str) -> Result<Source, String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let extra = |n: usize| -> Result<(), String> {
        match words.get(n) {
            Some(word) => Err(format!("unexpected '{word}' in '{text}'")),
            None => Ok(()),
        }
    };
    match words.first().copied() {
        Some("gpio") => {
            let pin = channel(&words, 1, "a pin number")?;
            match words.get(2).copied() {
                None => Ok(Source::GpioLevel(pin)),
                Some("rising") => extra(3).map(|_| Source::GpioEdge(pin, Edge::Rising)),
                Some("falling") => extra(3).map(|_| Source::GpioEdge(pin, Edge::Falling)),
                Some(other) => Err(format!("'{other}' is no edge of gpio {pin}: write `gpio {pin}` (a level) or `gpio {pin} rising|falling` (an edge)")),
            }
        }
        Some("adc") => {
            let ch = channel(&words, 1, "a channel number")?;
            let above = match words.get(2).copied() {
                Some(">") => true,
                Some("<") => false,
                _ => return Err(format!("adc {ch} is compared with a raw threshold: `adc {ch} > 2048` or `adc {ch} < 512`")),
            };
            let threshold = channel(&words, 3, "a raw threshold")?;
            extra(4)?;
            if (above && threshold == u16::MAX) || (!above && threshold == 0) {
                return Err(format!("'{text}' can never hold: the raw value is 0 to {}", u16::MAX));
            }
            Ok(Source::Adc { channel: ch, above, threshold })
        }
        Some("can") => {
            let id = number(words.get(1).ok_or("can needs a frame identifier")?, "a CAN identifier", CAN_MAX_ID)? as u32;
            extra(2).map(|_| Source::Can(id))
        }
        Some("uart") => {
            let port = channel(&words, 1, "a port number")?;
            let byte = number(words.get(2).ok_or_else(|| format!("uart {port} needs the byte that is the event: `uart {port} 0x41`"))?, "a byte", 255)? as u8;
            extra(3).map(|_| Source::Uart { port, byte })
        }
        Some(other) => Err(format!("'{other}' is no input channel: gpio, adc, can or uart")),
        None => Err("the channel is missing after `<-`".to_string()),
    }
}

/// `gpio 10`, `pwm 2 750`, `dac 1 4095`, `can 0x300`, `uart 0 0x45`.
pub fn parse_sink(text: &str) -> Result<Sink, String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let extra = |n: usize| -> Result<(), String> {
        match words.get(n) {
            Some(word) => Err(format!("unexpected '{word}' in '{text}'")),
            None => Ok(()),
        }
    };
    match words.first().copied() {
        Some("gpio") => {
            let pin = channel(&words, 1, "a pin number")?;
            extra(2).map(|_| Sink::Gpio(pin))
        }
        Some("pwm") => {
            let ch = channel(&words, 1, "a channel number")?;
            let duty = number(words.get(2).ok_or_else(|| format!("pwm {ch} needs its duty while on, in per mille: `pwm {ch} 750`"))?, "a duty", 1000)? as u16;
            extra(3).map(|_| Sink::Pwm(ch, duty))
        }
        Some("dac") => {
            let ch = channel(&words, 1, "a channel number")?;
            let raw = channel(&words, 2, "the raw value while on")?;
            extra(3).map(|_| Sink::Dac(ch, raw))
        }
        Some("can") => {
            let id = number(words.get(1).ok_or("can needs a frame identifier")?, "a CAN identifier", CAN_MAX_ID)? as u32;
            extra(2).map(|_| Sink::Can(id))
        }
        Some("uart") => {
            let port = channel(&words, 1, "a port number")?;
            let byte = number(words.get(2).ok_or_else(|| format!("uart {port} needs the byte to send: `uart {port} 0x45`"))?, "a byte", 255)? as u8;
            extra(3).map(|_| Sink::Uart(port, byte))
        }
        Some(other) => Err(format!("'{other}' is no output channel: gpio, pwm, dac, can or uart")),
        None => Err("the channel is missing after `->`".to_string()),
    }
}

fn strings(attributes: &std::collections::HashMap<String, AttributeValue>, key: &str) -> Result<Vec<String>, String> {
    match attributes.get(key) {
        None => Ok(Vec::new()),
        Some(AttributeValue::List(items)) => items.iter().map(|i| i.as_string().map(str::to_string).ok_or_else(|| format!("{key} is a list of strings"))).collect(),
        Some(AttributeValue::String(one)) => Ok(vec![one.clone()]),
        Some(_) => Err(format!("{key} is a list of strings")),
    }
}

fn shown(list: Vec<&str>) -> String {
    if list.is_empty() {
        "none".to_string()
    } else {
        list.iter().map(|t| format!("'{t}'")).collect::<Vec<_>>().join(", ")
    }
}

/// The period of a binding (`period: 10 ms`), in microseconds.
pub fn period_of(value: &AttributeValue, what: &str) -> Result<u64, String> {
    let q = match value.as_quantity() {
        Some(Ok(q)) if q.dimension() == Dimension::Time => q,
        _ => return Err(format!("{what} is a time: `10 ms`, got '{}'", value.display())),
    };
    let us = q.canonical() * 1e6;
    if us < 1.0 || us > u32::MAX as f64 || (us - us.round()).abs() > 1e-6 {
        return Err(format!("{what} {} is not a whole number of microseconds from 1 us to {} s", value.display(), u32::MAX / 1_000_000));
    }
    Ok(us.round() as u64)
}

fn input_target(m: &MachinePlan, name: &str) -> Result<Target, String> {
    if let Some(e) = m.event(name) {
        return Ok(Target::Event(e));
    }
    if let Some(c) = m.condition(name) {
        return Ok(Target::Condition(c));
    }
    Err(format!(
        "'{name}' is no event or condition of state_machine '{}' (events: {}; conditions: {})",
        m.name,
        shown(m.events.iter().map(|e| e.text.as_str()).collect()),
        shown(m.conditions.iter().map(|c| c.text.as_str()).collect())
    ))
}

fn output_origin(m: &MachinePlan, name: &str) -> Result<Origin, String> {
    if let Some(s) = m.state(name) {
        return Ok(Origin::State(s));
    }
    if let Some(a) = m.action(name) {
        return Ok(Origin::Action(a));
    }
    Err(format!(
        "'{name}' is no state or action of state_machine '{}' (states: {}; actions: {})",
        m.name,
        shown(m.states.iter().map(|s| s.path.as_str()).collect()),
        shown(m.actions.iter().map(|a| a.text.as_str()).collect())
    ))
}

fn sink_key(sink: Sink) -> String {
    match sink {
        Sink::Gpio(p) => format!("gpio {p}"),
        Sink::Pwm(c, _) => format!("pwm {c}"),
        Sink::Dac(c, _) => format!("dac {c}"),
        Sink::Can(id) => format!("can 0x{id:X}"),
        Sink::Uart(p, b) => format!("uart {p} 0x{b:02X}"),
    }
}

pub fn resolve(binding: &IoBinding, plan: &Plan) -> Result<Binding, String> {
    let what = format!("io_binding '{}'", binding.name);
    let a = &binding.attributes;
    let mut keys: Vec<&String> = a.keys().collect();
    keys.sort();
    if let Some(unknown) = keys.iter().find(|k| !KEYS.contains(&k.as_str())) {
        return Err(format!("{what}: '{unknown}' is not one of {}", KEYS.join(", ")));
    }
    let machine_ref = match a.get("machine").and_then(|v| v.as_string()) {
        Some(m) => m.to_string(),
        None if plan.machines.len() == 1 => plan.machines[0].name.clone(),
        None => return Err(format!("{what}: name the state_machine it binds (machine: \"...\"): the model has {}", plan.machines.len())),
    };
    let machine = plan
        .machines
        .iter()
        .position(|m| m.is(&machine_ref))
        .ok_or_else(|| format!("{what}: machine '{machine_ref}' is no state_machine of the model (state machines: {})", shown(plan.machines.iter().map(|m| m.name.as_str()).collect())))?;
    let m = &plan.machines[machine];
    let period_us = period_of(a.get("period").ok_or_else(|| format!("{what} needs its period (period: 10 ms): the step rate on the target"))?, &format!("{what}: period"))?;
    let watchdog = match a.get("watchdog") {
        None => false,
        Some(AttributeValue::Boolean(b)) => *b,
        Some(AttributeValue::String(s)) if s == "true" || s == "false" => s == "true",
        Some(other) => return Err(format!("{what}: watchdog is true or false, got '{}'", other.display())),
    };
    let priority = match a.get("priority") {
        None => None,
        Some(v) => Some(
            v.as_number()
                .filter(|n| n.fract() == 0.0 && PRIORITIES.contains(&(*n as u32)))
                .map(|n| n as u32)
                .ok_or_else(|| format!("{what}: priority is a whole number from {} (the highest) to {}, got '{}'", PRIORITIES.start(), PRIORITIES.end(), v.display()))?,
        ),
    };
    let mut inputs = Vec::new();
    for written in strings(a, "inputs").map_err(|e| format!("{what}: {e}"))? {
        let (name, channel) = written.rsplit_once("<-").ok_or_else(|| format!("{what}: input '{written}' reads `event or condition <- channel` (`LF-022 <- gpio 4 rising`)"))?;
        let name = name.trim();
        let target = input_target(m, name).map_err(|e| format!("{what}: {e}"))?;
        let source = parse_source(channel).map_err(|e| format!("{what}: input '{written}': {e}"))?;
        match (target, source) {
            (Target::Event(_), Source::GpioLevel(pin)) => {
                return Err(format!("{what}: '{name}' is an event, an instant: bind it to an edge (`gpio {pin} rising` or `gpio {pin} falling`), not to a level"))
            }
            (Target::Event(_), Source::Adc { channel, .. }) => {
                return Err(format!("{what}: '{name}' is an event, an instant: adc {channel} gives a level, bind it to a condition"))
            }
            (Target::Condition(_), Source::GpioEdge(pin, _)) => {
                return Err(format!("{what}: '{name}' is a condition, a level: bind it to `gpio {pin}`, not to an edge"))
            }
            (Target::Condition(_), Source::Uart { port, .. }) => {
                return Err(format!("{what}: '{name}' is a condition, a level: a byte on uart {port} is an instant, bind it to an event"))
            }
            _ => {}
        }
        if let Some(previous) = inputs.iter().find(|i: &&Input| i.target == target) {
            return Err(format!("{what}: '{name}' is bound twice ('{}' and '{written}')", previous.written));
        }
        inputs.push(Input { written: written.clone(), target, source });
    }
    let mut outputs: Vec<Output> = Vec::new();
    let mut driven: BTreeMap<String, String> = BTreeMap::new();
    for written in strings(a, "outputs").map_err(|e| format!("{what}: {e}"))? {
        let (name, channel) = written.rsplit_once("->").ok_or_else(|| format!("{what}: output '{written}' reads `state or action -> channel` (`Following -> gpio 10`)"))?;
        let name = name.trim();
        let origin = output_origin(m, name).map_err(|e| format!("{what}: {e}"))?;
        let sink = parse_sink(channel).map_err(|e| format!("{what}: output '{written}': {e}"))?;
        if let Some(previous) = driven.insert(sink_key(sink), written.clone()) {
            return Err(format!("{what}: {} is written by two outputs ('{previous}' and '{written}')", sink_key(sink)));
        }
        if let Sink::Gpio(pin) = sink {
            if let Some(input) = inputs.iter().find(|i| matches!(i.source, Source::GpioLevel(p) | Source::GpioEdge(p, _) if p == pin)) {
                return Err(format!("{what}: gpio {pin} is read by input '{}' and written by output '{written}'", input.written));
            }
        }
        outputs.push(Output { written: written.clone(), origin, sink });
    }
    if inputs.is_empty() && outputs.is_empty() {
        return Err(format!("{what} binds nothing: give inputs: [\"event <- channel\"] or outputs: [\"state -> channel\"]"));
    }
    let target = match a.get("target") {
        None => super::target::HOST.to_string(),
        Some(AttributeValue::String(name)) => name.clone(),
        Some(other) => return Err(format!("{what}: target is the name of a target package (`target: \"zephyr/litex_vexriscv\"`), got '{}'", other.display())),
    };
    let mut warnings = Vec::new();
    for (k, e) in m.events.iter().enumerate() {
        if !inputs.iter().any(|i| i.target == Target::Event(k)) {
            warnings.push(format!("{what}: event '{}' is bound to no channel: on the target it never occurs", e.text));
        }
    }
    for (k, c) in m.conditions.iter().enumerate() {
        if !inputs.iter().any(|i| i.target == Target::Condition(k)) {
            warnings.push(format!("{what}: condition '{}' is bound to no channel: on the target it never holds", c.text));
        }
    }
    let resolved = Binding {
        ident: format!("{}_{}", plan.ident, c_name(&binding.name)),
        name: binding.name.clone(),
        id: binding.id.clone(),
        machine,
        period_us,
        watchdog,
        priority,
        target,
        inputs,
        outputs,
        warnings,
    };
    let package = super::target::find(&resolved.target).map_err(|e| format!("{what}: target: {e}"))?;
    super::target::check(&resolved, &package)?;
    Ok(resolved)
}

/// Every binding of the model, resolved; the first error stops.
/// The priority each binding runs at (1 the highest), in the bindings'
/// order: the one written, else by the period — the shortest period the
/// highest (rate monotonic), equal periods equal, after the priorities
/// written.
pub fn priorities(bindings: &[Binding]) -> Vec<u32> {
    let written: Vec<u32> = bindings.iter().filter_map(|b| b.priority).collect();
    let next = written.iter().max().map(|p| p + 1).unwrap_or(1);
    let mut periods: Vec<u64> = bindings.iter().filter(|b| b.priority.is_none()).map(|b| b.period_us).collect();
    periods.sort_unstable();
    periods.dedup();
    bindings
        .iter()
        .map(|b| match b.priority {
            Some(p) => p,
            None => next + periods.iter().position(|p| *p == b.period_us).unwrap_or(0) as u32,
        })
        .collect()
}

pub fn resolve_all(ast: &Model, plan: &Plan) -> Result<Vec<Binding>, String> {
    let mut out: Vec<Binding> = Vec::new();
    for binding in &ast.io_bindings {
        let resolved = resolve(binding, plan)?;
        if out.iter().any(|b| b.ident == resolved.ident || b.id == resolved.id) {
            return Err(format!("io_binding '{}' is declared twice", binding.name));
        }
        out.push(resolved);
    }
    // A machine's task steps the blocks it reads that no binding runs:
    // each such block in one task only; a wire from another task's machine
    // would need a rate transition, which is not generated.
    let mut stepped_by: Vec<Option<usize>> = vec![None; plan.machines.len()];
    for (k, b) in out.iter().enumerate() {
        for u in super::plan::upstream(plan, b.machine) {
            let what = format!("io_binding '{}': state_machine '{}' reads state_machine '{}'", b.name, plan.machines[b.machine].name, plan.machines[u].name);
            if let Some(other) = out.iter().find(|o| o.machine == u) {
                return Err(format!("{what}, which io_binding '{}' runs as a task of its own: a wire between two tasks needs a rate transition, which is not generated (leave '{}' unbound: the task of '{}' steps it before it)", other.name, plan.machines[u].name, plan.machines[b.machine].name));
            }
            match stepped_by[u] {
                Some(j) if out[j].machine != b.machine => {
                    return Err(format!("{what}, which the task of io_binding '{}' steps too: a block is stepped by one task (give each task its own block)", out[j].name));
                }
                _ => stepped_by[u] = Some(k),
            }
        }
    }
    Ok(out)
}
