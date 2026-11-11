//! A board observed against the model: what it does, measured, beside what
//! the model says it does.
//!
//! An instrument drives the inputs of the board and times its outputs: a
//! second board running MicroPython on a serial port, wired to the pins of
//! the first (`arclang observe`). The inputs it played are replayed on the
//! model, by the executor of the model itself, at the moments the board's
//! own loop ran, and the outputs are compared change by change.
//!
//! The board's loop is not seen, but its outputs only change when it runs:
//! the moments of the changes lie on a regular lattice, whose period is the
//! real period of the loop and whose phase is where the loop stands against
//! the instrument's clock. Both are measured from the changes themselves,
//! with no help from the model.
//!
//! What this gives is an agreement on what was played, never a proof: "the
//! board did what the model does on these N changes". It is the measure a
//! proof's assumptions are checked against (the period of a loop, the clock).

use super::correspondence::{Correspondence, Term};
use crate::compiler::embedded::plan::Plan;
use crate::compiler::embedded::reference::{Runner, Vectors};
use crate::compiler::execution::expr::{Ty, Value};
use serde::{Deserialize, Serialize};

/// The levels of one pin over a run: its level at the start, and each change (time in us, level).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trace {
    pub name: String,
    pub first: bool,
    pub changes: Vec<(u64, bool)>,
}

impl Trace {
    fn at(&self, time: f64) -> bool {
        let mut level = self.first;
        for &(when, to) in &self.changes {
            if when as f64 > time {
                break;
            }
            level = to;
        }
        level
    }
}

/// What the instrument played on the inputs and saw on the outputs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recording {
    pub driven: Vec<Trace>,
    pub watched: Vec<Trace>,
}

/// One change of an output of the board, placed on the loop of the board.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Placed {
    pub output: String,
    pub time_us: u64,
    /// The loop of the board it happened in, counted from the first change.
    pub at_loop: i64,
    /// The step of the model at which the same change happens: the same
    /// loop, the one before (the board is one loop late), or none.
    pub model_step: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Report {
    /// The real period of the board's loop, in us, and how far the changes are from that lattice at most.
    pub loop_us: f64,
    pub lattice_worst_us: f64,
    pub model_period_us: u64,
    pub seconds: f64,
    pub inputs_played: usize,
    pub changes: Vec<Placed>,
    /// Changes of the model the board did not make.
    pub missing: usize,
    pub at_step: usize,
    pub one_loop_late: usize,
    pub unexplained: usize,
    /// Input edges that fell too close to a moment the board reads for the
    /// instrument to tell on which side, and were taken on the side the board showed.
    pub decided_by_board: usize,
    pub notes: Vec<String>,
}

impl Report {
    pub fn agrees(&self) -> bool {
        self.unexplained == 0 && self.missing == 0
    }
}

/// The instrument sees a change a little after the board read its inputs: its own polling.
const SEEN_AFTER_US: f64 = 200.0;
/// An input edge this close to a read of the board cannot be told before or after it.
const RESOLUTION_US: f64 = 600.0;
/// The steps the model runs on the idle inputs before the recording starts.
const SETTLE_STEPS: i64 = 300;

/// The lattice the times lie on: (its period, its phase, the index of each
/// time, the furthest a time is from it). The period is sought near `nominal`.
pub fn lattice(times: &[f64], nominal: f64) -> Option<(f64, f64, Vec<i64>, f64)> {
    if times.len() < 2 {
        return None;
    }
    let mut index = vec![0i64];
    let (mut period, mut phase) = (nominal, times[0]);
    for k in 1..times.len() {
        // The next time is a whole number of loops after the last: the period known so far says how many.
        let predicted = phase + index[k - 1] as f64 * period;
        index.push(index[k - 1] + ((times[k] - predicted) / period).round() as i64);
        // Least squares of time against index, once two distinct indices are known.
        let n = (k + 1) as f64;
        let (sx, sy) = (index.iter().map(|&i| i as f64).sum::<f64>(), times[..=k].iter().sum::<f64>());
        let sxx = index.iter().map(|&i| (i * i) as f64).sum::<f64>();
        let sxy = index.iter().zip(times).map(|(&i, &t)| i as f64 * t).sum::<f64>();
        let spread = n * sxx - sx * sx;
        if spread > 0.0 {
            let slope = (n * sxy - sx * sy) / spread;
            // A period far from the one sought is a lattice of something else.
            if (slope - nominal).abs() < nominal * 0.1 {
                period = slope;
                phase = (sy - slope * sx) / n;
            }
        }
    }
    let worst = index.iter().zip(times).map(|(&i, &t)| (t - (phase + i as f64 * period)).abs()).fold(0.0, f64::max);
    Some((period, phase, index, worst))
}

/// Where the outputs watched are in a row of the executor, and how each reads as a level.
struct Columns {
    machine: usize,
    /// What each driven pin is for the machine.
    inputs: Vec<Driven>,
    /// (the watched output's name, its column in the row of the machine's run)
    outputs: Vec<(String, usize)>,
}

/// A pin the instrument drives: the level of an input variable, or the edges an interrupt takes as an event.
enum Driven {
    Level(usize),
    /// (the event of the machine, whether a rising edge is one, whether a falling edge is one)
    Edges(usize, bool, bool),
}

fn columns(plan: &Plan, correspondence: &Correspondence, recording: &Recording) -> Result<Columns, String> {
    let what = format!("correspondence '{}'", correspondence.name);
    let machine = plan.machines.iter().position(|m| m.name == correspondence.machine).ok_or_else(|| format!("{what}: the model has no state_machine \"{}\"", correspondence.machine))?;
    let m = &plan.machines[machine];
    // The row of a run: every machine before this one, then its states, its actions, its variables.
    let before: usize = plan.machines[..machine].iter().map(|p| p.states.len() + p.actions.len() + p.variables.len() + p.budgets.len()).sum();
    let variable = |name: &str| m.program().variables.iter().position(|v| v.name == name);
    let mut inputs = Vec::new();
    for trace in &recording.driven {
        if let Some((mode, _, _)) = correspondence.events.iter().find(|(_, _, event)| *event == trace.name) {
            let event = m.events.iter().position(|e| e.text == trace.name).ok_or_else(|| format!("{what}: state_machine \"{}\" has no event '{}'", m.name, trace.name))?;
            inputs.push(Driven::Edges(event, mode != "FALLING", mode != "RISING"));
            continue;
        }
        let index = variable(&trace.name).ok_or_else(|| format!("{what}: state_machine \"{}\" has no variable nor event '{}' to drive", m.name, trace.name))?;
        if !correspondence.reads.iter().any(|(_, _, name)| *name == trace.name) {
            return Err(format!("{what}: '{}' is not an input the correspondence reads from a pin, nor an event it takes from one", trace.name));
        }
        inputs.push(Driven::Level(index));
    }
    let mut outputs = Vec::new();
    for trace in &recording.watched {
        // An output is named by what the correspondence writes to a pin: a variable, or a state.
        let term = correspondence
            .writes
            .iter()
            .map(|(_, term)| term)
            .find(|term| match term {
                Term::Variable(name) => *name == trace.name,
                Term::In(state) => *state == trace.name,
                Term::TimeIn(_) => false,
            })
            .ok_or_else(|| format!("{what}: '{}' is not what the correspondence writes to a pin (a variable, or a state)", trace.name))?;
        let column = match term {
            Term::Variable(name) => before + m.states.len() + m.actions.len() + variable(name).ok_or_else(|| format!("{what}: no variable '{name}'"))?,
            Term::In(state) => before + m.states.iter().position(|s| s.path == *state || s.path.rsplit('.').next() == Some(state.as_str())).ok_or_else(|| format!("{what}: no state '{state}'"))?,
            Term::TimeIn(_) => unreachable!("filtered above"),
        };
        outputs.push((trace.name.clone(), column));
    }
    Ok(Columns { machine, inputs, outputs })
}

/// The changes the model makes on each output when it reads the driven
/// inputs at the moments given: per output, the steps at which it changes.
fn replay(plan: &Plan, columns: &Columns, period_us: u64, driven: &[Trace], first: i64, last: i64, moment: &dyn Fn(i64) -> f64) -> Vec<Vec<i64>> {
    let steps = (last - first + 1).max(1) as usize;
    let mut vectors = Vectors::quiet(plan, steps);
    let program = plan.machines[columns.machine].program();
    for (trace, driven) in driven.iter().zip(&columns.inputs) {
        match driven {
            Driven::Level(variable) => {
                for k in 0..steps {
                    let level = trace.at(moment(first + k as i64));
                    vectors.values[columns.machine][*variable][k] = match program.variables[*variable].ty {
                        Ty::Bool => Value::Bool(level),
                        Ty::Int => Value::Int(level as i32),
                        Ty::Real => Value::Real(level as u8 as f64),
                        Ty::Float => Value::Float(level as u8 as f32),
                    };
                }
            }
            // An interrupt takes each edge when it happens: the step that follows sees
            // every edge since the one before, in the order they came.
            Driven::Edges(event, rising, falling) => {
                for &(time, level) in &trace.changes {
                    if (level && !*rising) || (!level && !*falling) {
                        continue;
                    }
                    let step = (0..steps).find(|&k| time as f64 <= moment(first + k as i64));
                    if let Some(k) = step {
                        vectors.queue[columns.machine][k].push(*event);
                        vectors.events[columns.machine][*event][k] = true;
                    }
                }
            }
        }
    }
    let mut runner = Runner::new(plan, period_us);
    let mut changes = vec![Vec::new(); columns.outputs.len()];
    let mut level: Vec<Option<bool>> = vec![None; columns.outputs.len()];
    for k in 0..steps {
        let row = runner.step(&vectors, k);
        let step = first + k as i64;
        for (output, (_, column)) in columns.outputs.iter().enumerate() {
            let now = row[*column] != 0.0;
            // What the model does while it settles on the idle inputs is before the recording.
            if level[output].is_some_and(|before| before != now) && moment(step) >= 0.0 {
                changes[output].push(step);
            }
            level[output] = Some(now);
        }
    }
    changes
}

/// Compare a recording with the model.
pub fn compare(plan: &Plan, correspondence: &Correspondence, period_us: u64, recording: &Recording) -> Result<Report, String> {
    let columns = columns(plan, correspondence, recording)?;
    let mut all: Vec<(u64, usize)> = recording.watched.iter().enumerate().flat_map(|(output, trace)| trace.changes.iter().map(move |&(time, _)| (time, output))).collect();
    all.sort();
    let times: Vec<f64> = all.iter().map(|&(time, _)| time as f64).collect();
    let Some((loop_us, phase, index, worst)) = lattice(&times, period_us as f64) else {
        return Err(format!("the outputs watched changed {} time(s): two changes at least are needed to find the loop of the board", all.len()));
    };
    let mut notes = Vec::new();
    if worst > loop_us / 4.0 {
        notes.push(format!("the changes of the outputs are up to {:.2} ms from a regular loop of {:.3} ms: the board's loop is not regular, or these outputs do not change in it", worst / 1000.0, loop_us / 1000.0));
    }
    let first = -((phase / loop_us).ceil() as i64) - SETTLE_STEPS;
    let last = index.last().copied().unwrap_or(0) + 40;

    // An input edge too close to a read of the board is taken, to begin with, where the instrument saw it.
    let mut driven = recording.driven.clone();
    let mut decided = 0usize;
    let moment = move |step: i64| phase + step as f64 * loop_us - SEEN_AFTER_US;
    let near = |time: u64| {
        let offset = (time as f64 - (phase - SEEN_AFTER_US)).rem_euclid(loop_us);
        offset < RESOLUTION_US || loop_us - offset < RESOLUTION_US
    };
    let mismatch = |model: &[Vec<i64>]| -> Option<i64> {
        // The first loop at which the board and the model part, over every output.
        let mut firsts = Vec::new();
        for (output, steps) in model.iter().enumerate() {
            let board: Vec<i64> = all.iter().zip(&index).filter(|((_, o), _)| *o == output).map(|(_, &n)| n).collect();
            for k in 0..board.len().max(steps.len()) {
                match (board.get(k), steps.get(k)) {
                    (Some(&b), Some(&m)) if b == m || b == m + 1 => {}
                    (Some(&b), Some(&m)) => {
                        firsts.push(b.min(m));
                        break;
                    }
                    (Some(&b), None) => {
                        firsts.push(b);
                        break;
                    }
                    (None, Some(&m)) => {
                        firsts.push(m);
                        break;
                    }
                    (None, None) => {}
                }
            }
        }
        firsts.into_iter().min()
    };
    let mut model = replay(plan, &columns, period_us, &driven, first, last, &moment);
    let mut tried: Vec<(usize, usize)> = Vec::new();
    while let Some(at) = mismatch(&model) {
        // The last edge before the parting that the instrument could not place: taken on the other side.
        let limit = moment(at) + loop_us;
        let candidate = driven
            .iter()
            .enumerate()
            .flat_map(|(trace, t)| t.changes.iter().enumerate().map(move |(edge, &(time, _))| (trace, edge, time)))
            .filter(|&(trace, edge, time)| (time as f64) < limit && near(time) && !tried.contains(&(trace, edge)))
            .max_by_key(|&(_, _, time)| time);
        let Some((trace, edge, time)) = candidate else { break };
        tried.push((trace, edge));
        let offset = (time as f64 - (phase - SEEN_AFTER_US)).rem_euclid(loop_us);
        let moved = if offset < RESOLUTION_US { time as f64 - offset - 1.0 } else { time as f64 + (loop_us - offset) + 1.0 };
        let mut attempt = driven.clone();
        attempt[trace].changes[edge].0 = moved.max(0.0) as u64;
        let again = replay(plan, &columns, period_us, &attempt, first, last, &moment);
        // Kept when the two then agree further than before.
        if mismatch(&again).is_none_or(|next| next > at) {
            driven = attempt;
            model = again;
            decided += 1;
        }
        if tried.len() > 64 {
            break;
        }
    }

    // Each change of the board, with the step of the model that makes it.
    let mut changes = Vec::new();
    let (mut at_step, mut late, mut unexplained, mut missing) = (0, 0, 0, 0);
    for (output, steps) in model.iter().enumerate() {
        let board: Vec<(u64, i64)> = all.iter().zip(&index).filter(|((_, o), _)| *o == output).map(|(&(time, _), &n)| (time, n)).collect();
        let mut parted = false;
        for (k, &(time, n)) in board.iter().enumerate() {
            let step = if parted { None } else { steps.get(k).copied().filter(|&m| n == m || n == m + 1) };
            match step {
                Some(m) if m == n => at_step += 1,
                Some(_) => late += 1,
                None => {
                    parted = true;
                    unexplained += 1;
                }
            }
            changes.push(Placed { output: columns.outputs[output].0.clone(), time_us: time, at_loop: n, model_step: step });
        }
        if !parted && steps.len() > board.len() {
            missing += steps.len() - board.len();
        }
    }
    changes.sort_by_key(|c| c.time_us);
    Ok(Report {
        loop_us,
        lattice_worst_us: worst,
        model_period_us: period_us,
        seconds: times.last().copied().unwrap_or(0.0) / 1e6,
        inputs_played: recording.driven.iter().map(|t| t.changes.len()).sum(),
        changes,
        missing,
        at_step,
        one_loop_late: late,
        unexplained,
        decided_by_board: decided,
        notes,
    })
}

/// What the instrument is asked to play: per segment, the input that changes and how long it then holds (ms).
pub fn stimulus(inputs: usize, segments: usize, seed: u64) -> Vec<(usize, u32)> {
    let mut state = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let mut draw = |n: u32| {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((state >> 33) as u32) % n
    };
    (0..segments)
        .map(|_| {
            let which = draw(inputs.max(1) as u32) as usize;
            // One hold in three is too short to count for a loop of some tens of milliseconds; the others are plain.
            let hold = if draw(3) == 0 { 5 + draw(36) } else { 70 + draw(81) };
            (which, hold)
        })
        .collect()
}

/// The MicroPython program the instrument runs: `drive` and `watch` are
/// (the instrument's pin, the level at rest) and the instrument's pins.
pub fn program(drive: &[(u32, bool)], watch: &[u32], play: &[(usize, u32)]) -> String {
    let pins = |list: &[String]| list.join(", ");
    let drives: Vec<String> = drive.iter().map(|(pin, rest)| format!("({pin}, {})", *rest as u8)).collect();
    let watches: Vec<String> = watch.iter().map(u32::to_string).collect();
    let segments: Vec<String> = play.iter().map(|(which, hold)| format!("({which}, {hold})")).collect();
    format!(
        "from machine import Pin\nimport time, gc\nD = [{}]\nW = [{}]\nS = [{}]\nouts = [Pin(g, Pin.OUT, value=r) for g, r in D]\nins = [Pin(g, Pin.IN) for g in W]\ntime.sleep_ms(600)\ngc.collect()\ngc.disable()\nlevels = [r for _, r in D]\nlast = [p.value() for p in ins]\nfirst = list(last)\nedges = []\nchanges = []\nstart = time.ticks_us()\ndef watch(until):\n    while time.ticks_diff(time.ticks_us(), start) < until:\n        for k in range(len(ins)):\n            v = ins[k].value()\n            if v != last[k]:\n                changes.append((time.ticks_diff(time.ticks_us(), start), k, v))\n                last[k] = v\nfor which, hold in S:\n    levels[which] ^= 1\n    outs[which].value(levels[which])\n    t = time.ticks_diff(time.ticks_us(), start)\n    edges.append((t, which, levels[which]))\n    watch(t + hold * 1000)\nfor k in range(len(outs)):\n    if levels[k] != D[k][1]:\n        levels[k] = D[k][1]\n        outs[k].value(levels[k])\n        edges.append((time.ticks_diff(time.ticks_us(), start), k, levels[k]))\nwatch(time.ticks_diff(time.ticks_us(), start) + 400000)\ngc.enable()\nprint('ARCLANG FIRST', first)\nprint('ARCLANG EDGES', edges)\nprint('ARCLANG CHANGES', changes)\n",
        pins(&drives),
        pins(&watches),
        pins(&segments)
    )
}

/// What the instrument printed, as a recording.
pub fn recorded(printed: &str, drive: &[(String, bool)], watch: &[String]) -> Result<Recording, String> {
    let line = |tag: &str| printed.lines().find_map(|l| l.trim().strip_prefix(tag)).ok_or_else(|| format!("the instrument did not print `{tag}`: {}", printed.lines().last().unwrap_or("it printed nothing")));
    let triple = regex::Regex::new(r"\((\d+), (\d+), (\d+)\)").expect("a pattern of the recording");
    let first: Vec<bool> = line("ARCLANG FIRST")?.trim().trim_matches(|c| c == '[' || c == ']').split(',').filter(|v| !v.trim().is_empty()).map(|v| v.trim() != "0").collect();
    if first.len() != watch.len() {
        return Err(format!("the instrument gave the level of {} output(s) for {} watched", first.len(), watch.len()));
    }
    let mut driven: Vec<Trace> = drive.iter().map(|(name, rest)| Trace { name: name.clone(), first: *rest, changes: Vec::new() }).collect();
    let mut watched: Vec<Trace> = watch.iter().zip(&first).map(|(name, &first)| Trace { name: name.clone(), first, changes: Vec::new() }).collect();
    for (tag, traces) in [("ARCLANG EDGES", &mut driven), ("ARCLANG CHANGES", &mut watched)] {
        for found in triple.captures_iter(line(tag)?) {
            let (time, which, level): (u64, usize, u8) = (found[1].parse().unwrap_or(0), found[2].parse().unwrap_or(usize::MAX), found[3].parse().unwrap_or(0));
            traces.get_mut(which).ok_or_else(|| format!("the instrument names pin {which}, which was not asked"))?.changes.push((time, level != 0));
        }
    }
    Ok(Recording { driven, watched })
}

/// Run a MicroPython program on the instrument through its raw REPL and give what it printed.
pub fn play(port: &str, program: &str, seconds: u64) -> Result<String, String> {
    use std::io::{Read, Write};
    use std::sync::mpsc;
    use std::time::{Duration, Instant};
    // The port is opened first and set up through its own descriptor: a system
    // that forgets the settings of a port once it is closed (macOS) keeps them so.
    let mut writer = std::fs::OpenOptions::new().read(true).write(true).open(port).map_err(|e| format!("cannot open {port}: {e}"))?;
    let stty = std::process::Command::new("stty")
        .args(["115200", "raw", "-echo", "cs8", "-cstopb", "-parenb", "clocal", "-hupcl"])
        .stdin(writer.try_clone().map_err(|e| e.to_string())?)
        .output()
        .map_err(|e| format!("cannot run stty: {e}"))?;
    if !stty.status.success() {
        return Err(format!("cannot set up {port}: {}", String::from_utf8_lossy(&stty.stderr).trim()));
    }
    let mut reader = writer.try_clone().map_err(|e| e.to_string())?;
    let (sender, receiver) = mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        while let Ok(count) = reader.read(&mut buffer) {
            if count == 0 || sender.send(buffer[..count].to_vec()).is_err() {
                break;
            }
        }
    });
    let until = |marker: &[u8], limit: Duration| -> Vec<u8> {
        let (mut data, end) = (Vec::new(), Instant::now() + limit);
        while Instant::now() < end && !data.windows(marker.len()).any(|w| w == marker) {
            if let Ok(bytes) = receiver.recv_timeout(Duration::from_millis(100)) {
                data.extend(bytes);
            }
        }
        data
    };
    let send = |writer: &mut std::fs::File, bytes: &[u8]| writer.write_all(bytes).and_then(|()| writer.flush()).map_err(|e| format!("cannot write to {port}: {e}"));
    // Stop whatever runs, then the raw mode: a program is sent whole and run on ^D.
    // Opening the port can restart the board: it is asked until it answers.
    let mut banner = Vec::new();
    for _ in 0..6 {
        send(&mut writer, b"\r\x03\x03")?;
        until(b"\x00never", Duration::from_millis(400));
        send(&mut writer, b"\r\x01")?;
        banner = until(b"raw REPL; CTRL-B to exit\r\n>", Duration::from_millis(1500));
        if banner.windows(8).any(|w| w == b"raw REPL") {
            break;
        }
    }
    if !banner.windows(8).any(|w| w == b"raw REPL") {
        return Err(format!("no MicroPython answers on {port}: the instrument is a board running MicroPython, its console on this port"));
    }
    for chunk in program.as_bytes().chunks(128) {
        send(&mut writer, chunk)?;
        std::thread::sleep(Duration::from_millis(15));
    }
    send(&mut writer, b"\x04")?;
    let out = until(b"\x04>", Duration::from_secs(seconds));
    let _ = send(&mut writer, b"\x02");
    let text = String::from_utf8_lossy(&out).into_owned();
    let text = text.strip_prefix("OK").unwrap_or(&text);
    let (printed, error) = text.split_once('\x04').unwrap_or((text, ""));
    let error = error.replace("\x04>", "");
    if !error.trim().is_empty() {
        return Err(format!("the instrument failed: {}", error.trim().lines().last().unwrap_or_default()));
    }
    Ok(printed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_loop_of_a_board_is_found_from_the_moments_its_outputs_change() {
        // A loop of 10.018 ms starting at 3.2 ms, changes at some of its turns, each seen within 0.2 ms.
        let turns = [0i64, 31, 62, 63, 140, 141, 290, 905, 1700];
        let times: Vec<f64> = turns.iter().enumerate().map(|(k, &n)| 3200.0 + n as f64 * 10018.0 + (k % 3) as f64 * 90.0).collect();
        let (period, _, index, worst) = lattice(&times, 10000.0).unwrap();
        assert!((period - 10018.0).abs() < 0.5, "{period}");
        assert_eq!(index, turns.to_vec());
        assert!(worst < 200.0, "{worst}");
        // One change is not enough.
        assert!(lattice(&[1.0], 10000.0).is_none());
    }

    #[test]
    fn what_the_instrument_prints_is_read_as_a_recording() {
        let printed = "ARCLANG FIRST [0]\r\nARCLANG EDGES [(195, 0, 0), (142232, 0, 1)]\r\nARCLANG CHANGES [(55400, 0, 1)]\r\n";
        let recording = recorded(printed, &[("button".to_string(), true)], &["led".to_string()]).unwrap();
        assert_eq!(recording.driven[0].changes, vec![(195, false), (142232, true)]);
        assert_eq!(recording.watched[0], Trace { name: "led".to_string(), first: false, changes: vec![(55400, true)] });
        assert!(recording.driven[0].at(100.0) && !recording.driven[0].at(1000.0) && recording.driven[0].at(200000.0));
        assert!(recorded("Traceback", &[], &[]).is_err());
    }

    #[test]
    fn the_program_of_the_instrument_drives_and_watches_the_pins_given() {
        let play = stimulus(1, 50, 7);
        assert_eq!(play, stimulus(1, 50, 7), "the same seed plays the same run");
        assert!(play.iter().all(|&(which, hold)| which == 0 && (5..=150).contains(&hold)));
        let program = program(&[(13, true)], &[35], &play[..2]);
        assert!(program.contains("D = [(13, 1)]") && program.contains("W = [35]") && program.contains("print('ARCLANG CHANGES', changes)"), "{program}");
    }
}
