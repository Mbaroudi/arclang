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
    /// The outputs whose value is measured (the share of the time a pin is high, 0 to 255).
    #[serde(default)]
    pub measured: Vec<Measure>,
    /// When the instrument stopped looking, in us (0 when the recording does not say).
    #[serde(default)]
    pub end_us: u64,
}

/// The value of one pin over a run, as the instrument measures it: the share
/// of each of its periods the pin was high, 0 to 255 (a pin that stays put is
/// 0 or 255). A change is the end of the first period that showed the new
/// value, once the next confirmed it; `window_us` is how long after the loop
/// that made it that is at most.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measure {
    pub name: String,
    pub first: i64,
    pub window_us: u64,
    pub changes: Vec<(u64, i64)>,
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
    /// The changes of the measured values that are the model's, in value and in step, and the first that is not.
    pub values_same: usize,
    pub values_differ: Option<String>,
    pub notes: Vec<String>,
}

impl Report {
    pub fn agrees(&self) -> bool {
        self.unexplained == 0 && self.missing == 0 && self.values_differ.is_none()
    }
}

/// The instrument sees a change a little after the board read its inputs: its own polling.
const SEEN_AFTER_US: f64 = 200.0;
/// An input edge this close to a read of the board cannot be told before or after it.
const RESOLUTION_US: f64 = 600.0;
/// How far a measured value may be from the model's (of 255): what a window of a few periods of the pin can be off by.
const VALUE_TOLERANCE: i64 = 20;
/// How long a measured pin stays without a change before the instrument takes it for a steady level, in us.
pub const MEASURE_QUIET_US: u64 = 5_000;
/// How long after the loop that made it a change of a measured value is seen at most, in us:
/// one period of the pin for the new value to show, or the quiet time, and one more to confirm it started there.
pub const MEASURE_WINDOW_US: u64 = 7_500;
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
    /// (the measured output's name, its column)
    measured: Vec<(String, usize)>,
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
    let mut measured = Vec::new();
    for measure in &recording.measured {
        if !correspondence.pwm.iter().any(|(_, name)| *name == measure.name) {
            return Err(format!("{what}: '{}' is not a value the correspondence writes to a pin with analogWrite", measure.name));
        }
        let index = variable(&measure.name).ok_or_else(|| format!("{what}: no variable '{}'", measure.name))?;
        measured.push((measure.name.clone(), before + m.states.len() + m.actions.len() + index));
    }
    Ok(Columns { machine, inputs, outputs, measured })
}

/// The changes the model makes on each output when it reads the driven
/// inputs at the moments given: per output, the steps at which it changes.
fn replay(plan: &Plan, columns: &Columns, period_us: u64, driven: &[Trace], first: i64, last: i64, moment: &dyn Fn(i64) -> f64) -> Vec<Vec<i64>> {
    let rows = run(plan, columns, period_us, driven, first, last, moment);
    let mut changes = vec![Vec::new(); columns.outputs.len()];
    let mut level: Vec<Option<bool>> = vec![None; columns.outputs.len()];
    for (k, row) in rows.iter().enumerate() {
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

/// The run of the model on the driven inputs read at the moments given: its row at each step.
fn run(plan: &Plan, columns: &Columns, period_us: u64, driven: &[Trace], first: i64, last: i64, moment: &dyn Fn(i64) -> f64) -> Vec<Vec<f64>> {
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
    (0..steps).map(|k| runner.step(&vectors, k)).collect()
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
    // The model is run to the end of the recording (a recording that does not say where it ends: a little past the last change).
    let last = if recording.end_us > 0 { ((recording.end_us as f64 - 300.0 - phase) / loop_us).floor() as i64 } else { index.last().copied().unwrap_or(0) + 40 };

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
    // The measured values: each change of the board against the model's, in value and in step:
    // how many are the model's, and the first that is not (the step at which they part, and what parts).
    let values = |driven: &[Trace]| -> (usize, Option<(i64, String)>) {
        let mut same = 0usize;
        if columns.measured.is_empty() {
            return (same, None);
        }
        let rows = run(plan, &columns, period_us, driven, first, last, &moment);
        for (measure, (name, column)) in recording.measured.iter().zip(&columns.measured) {
            // The model's changes of this value: (the step, the value).
            let mut model: Vec<(i64, i64)> = Vec::new();
            let mut held: Option<i64> = None;
            for (k, row) in rows.iter().enumerate() {
                let (step, now) = (first + k as i64, row[*column].round() as i64);
                if held.is_some_and(|before| before != now) && moment(step) >= 0.0 {
                    model.push((step, now));
                }
                held = Some(now);
            }
            // A change is seen a little after the loop that made it (less than a loop, or the measure says nothing of when): the last loop before it.
            let board: Vec<(i64, i64, u64)> = measure.changes.iter().map(|&(time, value)| (((time as f64 + 300.0 - phase) / loop_us).floor() as i64, value, time)).collect();
            for k in 0..board.len().max(model.len()) {
                let parting = match (board.get(k), model.get(k)) {
                    (Some(&(n, value, time)), Some(&(m, wanted))) if (value - wanted).abs() > VALUE_TOLERANCE => Some((n.min(m), format!("at {:.1} ms {name} is measured {value} where the model has {wanted} (loop {n}, step {m})", time as f64 / 1000.0))),
                    (Some(&(n, value, time)), Some(&(m, _))) if !(m - 1..=m + 1).contains(&n) => Some((n.min(m), format!("at {:.1} ms {name} becomes {value} at loop {n} of the board, where the model makes it at step {m}", time as f64 / 1000.0))),
                    (Some(_), Some(_)) => None,
                    (Some(&(n, value, time)), None) => Some((n, format!("at {:.1} ms {name} becomes {value} (loop {n}), a change the model does not make", time as f64 / 1000.0))),
                    (None, Some(&(m, wanted))) => Some((m, format!("the model makes {name} {wanted} at step {m}, a change the board was not seen to make"))),
                    (None, None) => None,
                };
                match parting {
                    None => same += 1,
                    Some(parting) => return (same, Some(parting)),
                }
            }
        }
        (same, None)
    };
    // Where the board and the model first part, on the levels followed or on the values measured.
    let parting = |model: &[Vec<i64>], driven: &[Trace]| -> Option<i64> {
        match (mismatch(model), values(driven).1.map(|(step, _)| step)) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    };
    let mut model = replay(plan, &columns, period_us, &driven, first, last, &moment);
    let mut tried: Vec<(usize, usize)> = Vec::new();
    while let Some(at) = parting(&model, &driven) {
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
        if parting(&again, &attempt).is_none_or(|next| next > at) {
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

    // The measured values, on the inputs as they were finally taken.
    let (values_same, values_differ) = match values(&driven) {
        (same, parting) => (same, parting.map(|(_, what)| what)),
    };
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
        values_same,
        values_differ,
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

/// The MicroPython program the instrument runs: `drive` is (the instrument's
/// pin, the level at rest), `watch` the pins whose level it follows, `measure`
/// those whose share of time high it measures over a window.
pub fn program(drive: &[(u32, bool)], watch: &[u32], measure: &[u32], play: &[(usize, u32)]) -> String {
    let pins = |list: &[String]| list.join(", ");
    let drives: Vec<String> = drive.iter().map(|(pin, rest)| format!("({pin}, {})", *rest as u8)).collect();
    let watches: Vec<String> = watch.iter().map(u32::to_string).collect();
    let measures: Vec<String> = measure.iter().map(u32::to_string).collect();
    let segments: Vec<String> = play.iter().map(|(which, hold)| format!("({which}, {hold})")).collect();
    const BODY: &str = r#"outs = [Pin(g, Pin.OUT, value=r) for g, r in D]
ins = [Pin(g, Pin.IN) for g in W]
meas = [Pin(g, Pin.IN) for g in M]
time.sleep_ms(600)
gc.collect()
gc.disable()
levels = [r for _, r in D]
last = [p.value() for p in ins]
first = list(last)
edges = []
changes = []
m_level = [p.value() for p in meas]
m_rise = [None] * len(meas)
m_high = [0] * len(meas)
m_edge = [0] * len(meas)
value = [None] * len(meas)
value_first = [0] * len(meas)
maybe = [None] * len(meas)
maybe_at = [0] * len(meas)
values = []
start = time.ticks_us()
def feed(k, v, t):
    if value[k] is None:
        value[k] = v
        value_first[k] = v
    elif abs(v - value[k]) > 20:
        if maybe[k] is not None and abs(v - maybe[k]) <= 12:
            values.append((maybe_at[k], k, v))
            value[k] = v
            maybe[k] = None
        else:
            maybe[k] = v
            maybe_at[k] = t
    else:
        maybe[k] = None
def watch(until):
    while True:
        now = time.ticks_diff(time.ticks_us(), start)
        if now >= until:
            return
        for k in range(len(ins)):
            v = ins[k].value()
            if v != last[k]:
                changes.append((time.ticks_diff(time.ticks_us(), start), k, v))
                last[k] = v
        for k in range(len(meas)):
            v = meas[k].value()
            if v != m_level[k]:
                if v:
                    if m_rise[k] is not None and now > m_rise[k]:
                        period = now - m_rise[k]
                        feed(k, (m_high[k] * 255 + period // 2) // period, now)
                    m_rise[k] = now
                elif m_rise[k] is not None:
                    m_high[k] = now - m_rise[k]
                m_level[k] = v
                m_edge[k] = now
            elif now - m_edge[k] > QUIET:
                feed(k, 255 if v else 0, now)
                m_edge[k] = now
                m_rise[k] = None
for which, hold in S:
    levels[which] ^= 1
    outs[which].value(levels[which])
    t = time.ticks_diff(time.ticks_us(), start)
    edges.append((t, which, levels[which]))
    watch(t + hold * 1000)
for k in range(len(outs)):
    if levels[k] != D[k][1]:
        levels[k] = D[k][1]
        outs[k].value(levels[k])
        edges.append((time.ticks_diff(time.ticks_us(), start), k, levels[k]))
watch(time.ticks_diff(time.ticks_us(), start) + 400000)
end = time.ticks_diff(time.ticks_us(), start)
gc.enable()
print('ARCLANG FIRST', first)
print('ARCLANG EDGES', edges)
print('ARCLANG CHANGES', changes)
print('ARCLANG VALUES FIRST', value_first)
print('ARCLANG VALUES', values)
print('ARCLANG END', end)
"#;
    format!("from machine import Pin\nimport time, gc\nD = [{}]\nW = [{}]\nM = [{}]\nS = [{}]\nQUIET = {MEASURE_QUIET_US}\n{BODY}", pins(&drives), pins(&watches), pins(&measures), pins(&segments))
}

/// What the instrument printed, as a recording.
pub fn recorded(printed: &str, drive: &[(String, bool)], watch: &[String], measure: &[String]) -> Result<Recording, String> {
    let line = |tag: &str| printed.lines().find_map(|l| l.trim().strip_prefix(tag)).ok_or_else(|| format!("the instrument did not print `{tag}`: {}", printed.lines().last().unwrap_or("it printed nothing")));
    let triple = regex::Regex::new(r"\((\d+), (\d+), (\d+)\)").expect("a pattern of the recording");
    let numbers = |text: &str| -> Vec<i64> { text.trim().trim_matches(|c| c == '[' || c == ']').split(',').filter_map(|v| v.trim().parse().ok()).collect() };
    let first = numbers(line("ARCLANG FIRST ")?);
    if first.len() != watch.len() {
        return Err(format!("the instrument gave the level of {} output(s) for {} watched", first.len(), watch.len()));
    }
    let mut driven: Vec<Trace> = drive.iter().map(|(name, rest)| Trace { name: name.clone(), first: *rest, changes: Vec::new() }).collect();
    let mut watched: Vec<Trace> = watch.iter().zip(&first).map(|(name, &first)| Trace { name: name.clone(), first: first != 0, changes: Vec::new() }).collect();
    for (tag, traces) in [("ARCLANG EDGES ", &mut driven), ("ARCLANG CHANGES ", &mut watched)] {
        for found in triple.captures_iter(line(tag)?) {
            let (time, which, level): (u64, usize, u8) = (found[1].parse().unwrap_or(0), found[2].parse().unwrap_or(usize::MAX), found[3].parse().unwrap_or(0));
            traces.get_mut(which).ok_or_else(|| format!("the instrument names pin {which}, which was not asked"))?.changes.push((time, level != 0));
        }
    }
    let mut measured = Vec::new();
    if !measure.is_empty() {
        let first = numbers(line("ARCLANG VALUES FIRST ")?);
        if first.len() != measure.len() {
            return Err(format!("the instrument gave the value of {} output(s) for {} measured", first.len(), measure.len()));
        }
        measured = measure.iter().zip(&first).map(|(name, &first)| Measure { name: name.clone(), first, window_us: MEASURE_WINDOW_US, changes: Vec::new() }).collect::<Vec<_>>();
        let values = printed.lines().find_map(|l| l.trim().strip_prefix("ARCLANG VALUES ").filter(|rest| !rest.starts_with("FIRST"))).ok_or("the instrument did not print `ARCLANG VALUES`")?;
        for found in triple.captures_iter(values) {
            let (time, which, value): (u64, usize, i64) = (found[1].parse().unwrap_or(0), found[2].parse().unwrap_or(usize::MAX), found[3].parse().unwrap_or(0));
            measured.get_mut(which).ok_or_else(|| format!("the instrument names measured pin {which}, which was not asked"))?.changes.push((time, value));
        }
    }
    let end_us = printed.lines().find_map(|l| l.trim().strip_prefix("ARCLANG END ")).and_then(|t| t.trim().parse().ok()).unwrap_or(0);
    Ok(Recording { driven, watched, measured, end_us })
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
        let recording = recorded(printed, &[("button".to_string(), true)], &["led".to_string()], &[]).unwrap();
        assert_eq!(recording.driven[0].changes, vec![(195, false), (142232, true)]);
        assert_eq!(recording.watched[0], Trace { name: "led".to_string(), first: false, changes: vec![(55400, true)] });
        assert!(recording.driven[0].at(100.0) && !recording.driven[0].at(1000.0) && recording.driven[0].at(200000.0));
        assert!(recorded("Traceback", &[], &[], &[]).is_err());
        // A measured value: its first reading and its changes.
        let printed = "ARCLANG FIRST [0]\r\nARCLANG EDGES []\r\nARCLANG CHANGES []\r\nARCLANG VALUES FIRST [0]\r\nARCLANG VALUES [(40800, 0, 85), (910000, 0, 170)]\r\n";
        let recording = recorded(printed, &[], &["beat".to_string()], &["duty".to_string()]).unwrap();
        assert_eq!(recording.measured, [Measure { name: "duty".to_string(), first: 0, window_us: MEASURE_WINDOW_US, changes: vec![(40800, 85), (910000, 170)] }]);
    }

    #[test]
    fn the_program_of_the_instrument_drives_and_watches_the_pins_given() {
        let play = stimulus(1, 50, 7);
        assert_eq!(play, stimulus(1, 50, 7), "the same seed plays the same run");
        assert!(play.iter().all(|&(which, hold)| which == 0 && (5..=150).contains(&hold)));
        let program = program(&[(13, true)], &[35], &[14], &play[..2]);
        assert!(program.contains("D = [(13, 1)]") && program.contains("W = [35]") && program.contains("M = [14]") && program.contains("print('ARCLANG CHANGES', changes)") && program.contains("QUIET = 5000"), "{program}");
    }
}
