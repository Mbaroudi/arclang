//! The plant in the loop (`execute: { plant: { ... } }`): a case whose
//! machine does not take its inputs from a list but from a model of what
//! it controls — the physics in Modelica, built once as an FMU by
//! OpenModelica, or an FMU from any other tool (FMI 2.0 co-simulation:
//! Simulink, Amesim, Dymola). At each step the machine reads the plant's
//! variables into its `in` variables, steps, and the plant advances one
//! period with the machine's outputs held (a zero-order hold, as a
//! controller's DAC). The same loop runs around the model (the reference),
//! the generated C (`arclang sil`), the fixed-point C and the SoC
//! (`arclang soc`); the batch stages (`hdl`, `pil`) replay the vectors the
//! loop recorded. Nothing is real time: the plant and the device advance in
//! lock step, so a run is reproducible to the bit.

use super::case::{Case, Plant as Spec};
use super::plan::Plan;
use super::reference::{self, Table, Vectors};
use crate::compiler::execution::expr::{Ty, Value};
use crate::compiler::simulation::tool::ToolConfig;
#[cfg(feature = "native")]
use crate::compiler::simulation::twin_step::Model;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// What a plant does: start anew, initialise at t = 0 with the inputs, and
/// advance to `t` with the inputs held since the last instant; each answer
/// is the plant's output variables, in SI, in the order of `Spec::outputs`.
pub trait Plant {
    fn restart(&mut self) -> Result<(), String>;
    fn init(&mut self, inputs: &[f64]) -> Result<Vec<f64>, String>;
    fn advance(&mut self, t: f64, inputs: &[f64]) -> Result<Vec<f64>, String>;
}

/// What a device does in the loop: step `k` on the vectors' inputs of that
/// step (the plant's outputs are already in them), answering the row of
/// `reference::columns` — the model's values, or the device's words.
pub trait Device {
    fn step(&mut self, vectors: &Vectors, k: usize) -> Result<Vec<f64>, String>;
}

impl Device for reference::Runner<'_> {
    fn step(&mut self, vectors: &Vectors, k: usize) -> Result<Vec<f64>, String> {
        Ok(reference::Runner::step(self, vectors, k))
    }
}

/// A loop run: the vectors as the plant filled them (the batch stages
/// replay them), the device's rows, and the plant's outputs at each step.
pub struct Run {
    pub vectors: Vectors,
    pub table: Table,
    pub plant: Vec<Vec<f64>>,
}

/// The column of a machine's variable in `reference::columns`.
pub fn column(plan: &Plan, machine: usize, variable: usize) -> usize {
    let mut base = 0;
    for (n, m) in plan.machines.iter().enumerate() {
        if n == machine {
            return base + m.states.len() + m.actions.len() + variable;
        }
        base += m.states.len() + m.actions.len() + m.variables.len() + m.budgets.len();
    }
    base
}

/// A number as the machine stores a variable of this type.
fn value_of(ty: Ty, x: f64) -> Value {
    match ty {
        Ty::Bool => Value::Bool(x != 0.0),
        Ty::Int => Value::Int(x.round() as i32),
        Ty::Real => Value::Real(x),
        Ty::Float => Value::Float(x as f32),
    }
}

/// Run the case's loop: the device around the plant, `case.steps` steps.
/// `words` says what a device's row holds for a variable: `None` for the
/// model's values, `Some(f)` for a device reporting words `f` turns into
/// values (the fixed point).
pub fn run(plan: &Plan, case: &Case, plant: &mut dyn Plant, device: &mut dyn Device, words: Option<&dyn Fn(usize, f64) -> f64>) -> Result<Run, String> {
    run_held(plan, case, plant, device, words, false)
}

/// The loop of [`run`], or (`late`) with the machine's outputs reaching the
/// plant one period late: over `[t_k, t_k+1]` the plant is driven by step
/// `k - 1` (the initial outputs at first). That is the loop a fixed-step
/// co-simulation master runs when every FMU steps from the same instant
/// (OMSimulator's): a controller's outputs of a step are read at its end.
pub fn run_held(plan: &Plan, case: &Case, plant: &mut dyn Plant, device: &mut dyn Device, words: Option<&dyn Fn(usize, f64) -> f64>, late: bool) -> Result<Run, String> {
    let spec = case.plant.as_ref().ok_or("the case has no plant")?;
    let m = &plan.machines[case.machine];
    let program = m.program();
    let mut vectors = case.vectors(plan);
    let to_plant = |row: &[f64]| -> Vec<f64> {
        spec.inputs
            .iter()
            .map(|(_, i)| {
                let x = row[column(plan, case.machine, *i)];
                let x = words.map_or(x, |f| f(*i, x));
                x * program.variables[*i].scale
            })
            .collect()
    };
    plant.restart()?;
    // The plant starts on the machine's initial outputs.
    let initial: Vec<f64> = spec.inputs.iter().map(|(_, i)| program.variables[*i].initial.number() * program.variables[*i].scale).collect();
    let mut outputs = plant.init(&initial)?;
    let mut held = initial.clone();
    let mut rows = Vec::with_capacity(case.steps);
    let mut plant_rows = Vec::with_capacity(case.steps);
    for k in 0..case.steps {
        if outputs.len() != spec.outputs.len() {
            return Err(format!("the plant answered {} values for {} outputs", outputs.len(), spec.outputs.len()));
        }
        for ((i, _), y) in spec.outputs.iter().zip(&outputs) {
            let v = &program.variables[*i];
            if !y.is_finite() {
                return Err(format!("the plant gives '{}' a value that is not a number at step {k} ({} s)", v.name, k as f64 * case.step_us as f64 / 1e6));
            }
            vectors.values[case.machine][*i][k] = value_of(v.ty, y / v.scale);
        }
        plant_rows.push(outputs.clone());
        let row = device.step(&vectors, k)?;
        let t = (k + 1) as f64 * case.step_us as f64 / 1e6;
        let now = to_plant(&row);
        outputs = plant.advance(t, if late { &held } else { &now })?;
        held = now;
        rows.push(row);
    }
    Ok(Run { vectors, table: Table { names: reference::columns(plan), period_us: case.step_us, rows, units: reference::units(plan) }, plant: plant_rows })
}

/// The plants of a model's cases, built once each where the model lives:
/// a Modelica plant as an FMU by OpenModelica (kept under
/// `.arclang/plant/` beside the model, so a stage that runs in a Docker
/// image steps the FMU the host built), a given FMU as it is.
pub struct Plants {
    #[cfg_attr(not(feature = "native"), allow(dead_code))]
    base: Option<PathBuf>,
    #[cfg_attr(not(feature = "native"), allow(dead_code))]
    config: ToolConfig,
    built: HashMap<String, Box<dyn Plant>>,
}

impl Plants {
    /// No plant can be built: a case with one is an error that says so.
    pub fn none() -> Plants {
        Plants { base: None, config: ToolConfig::from_env(), built: HashMap::new() }
    }

    /// The plants of the model in `base` (its directory: the sources and the FMUs are relative to it).
    pub fn at(base: &Path) -> Plants {
        Plants { base: Some(base.to_path_buf()), config: ToolConfig::from_env(), built: HashMap::new() }
    }

    /// The directory of the FMUs built for the model's plants.
    pub fn cache_dir(base: &Path) -> PathBuf {
        base.join(".arclang").join("plant")
    }

    /// What the FMU of a Modelica plant depends on: the model, its sources as
    /// they are, the wires.
    fn key(base: &Path, spec: &Spec) -> Result<String, String> {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        let mut field = |label: &str, bytes: &[u8]| {
            hash.update(label.as_bytes());
            hash.update((bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
        };
        field("format", b"arclang-plant-fmu-2");
        field("model", spec.label().as_bytes());
        for source in &spec.sources {
            let text = std::fs::read(base.join(source)).map_err(|e| format!("plant: source '{source}' cannot be read: {e}"))?;
            field("source", source.as_bytes());
            field("text", &text);
        }
        for (target, _) in &spec.inputs {
            field("input", target.as_bytes());
        }
        for (_, variable) in &spec.outputs {
            field("output", variable.as_bytes());
        }
        Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect())
    }

    /// Build the FMU of a Modelica plant into the cache if it is not there:
    /// the path of the FMU. A given FMU is its own file.
    #[cfg(feature = "native")]
    pub fn prepare(&self, ast: &crate::compiler::ast::Model, spec: &Spec) -> Result<PathBuf, String> {
        let base = self.base.as_ref().ok_or("the plant needs the model's directory (run the command on the model file)")?;
        if let Some(fmu) = &spec.fmu {
            let path = base.join(fmu);
            if !path.exists() {
                return Err(format!("plant: the FMU {} does not exist", path.display()));
            }
            return Ok(path);
        }
        let model = spec.model.as_deref().unwrap_or_default();
        let cached = Self::cache_dir(base).join(format!("{}.fmu", Self::key(base, spec)?));
        if cached.exists() {
            return Ok(cached);
        }
        let export = crate::compiler::modelica::export(ast).map_err(|e| format!("the model does not export to Modelica: {e}"))?;
        // The plant's own `input` variables are the FMU's inputs: no wrapper binding.
        let inputs: Vec<String> = spec.inputs.iter().map(|(t, _)| t.clone()).collect();
        let variables: Vec<String> = spec.outputs.iter().map(|(_, v)| v.clone()).collect();
        let built = Model::build_modelica(&export, model, &spec.sources, &[], inputs, variables, spec.tolerance, base, &self.config).map_err(|e| format!("plant {model}: {e}"))?;
        std::fs::create_dir_all(cached.parent().expect("a parent")).map_err(|e| format!("cannot make {}: {e}", Self::cache_dir(base).display()))?;
        std::fs::copy(built.fmu_file(), &cached).map_err(|e| format!("cannot keep the plant's FMU as {}: {e}", cached.display()))?;
        Ok(cached)
    }

    /// A plant given for the cases naming `label` (`model:` or `fmu:`): the
    /// tests' way to close a loop without OpenModelica.
    pub fn inject(&mut self, label: &str, plant: Box<dyn Plant>) {
        self.built.insert(label.to_string(), plant);
    }

    /// The plant of a case, built the first time it is asked for.
    #[cfg(not(feature = "native"))]
    pub fn get(&mut self, _ast: &crate::compiler::ast::Model, case: &Case) -> Result<&mut dyn Plant, String> {
        let spec = case.plant.as_ref().ok_or("the case has no plant")?;
        if self.built.contains_key(&spec.label()) {
            return Ok(self.built.get_mut(&spec.label()).expect("injected").as_mut());
        }
        Err("a plant is built and stepped by the native compiler (OpenModelica, a C compiler), not in the browser".to_string())
    }

    /// The plant of a case, built the first time it is asked for.
    #[cfg(feature = "native")]
    pub fn get(&mut self, ast: &crate::compiler::ast::Model, case: &Case) -> Result<&mut dyn Plant, String> {
        let spec = case.plant.as_ref().ok_or("the case has no plant")?;
        if self.built.contains_key(&spec.label()) {
            return Ok(self.built.get_mut(&spec.label()).expect("injected").as_mut());
        }
        let key = format!("{}|{:?}|{:?}|{:?}", spec.label(), spec.inputs, spec.outputs, spec.sources);
        if !self.built.contains_key(&key) {
            let path = self.prepare(ast, spec)?;
            let inputs: Vec<String> = spec.inputs.iter().map(|(t, _)| t.clone()).collect();
            let variables: Vec<String> = spec.outputs.iter().map(|(_, v)| v.clone()).collect();
            let model = Model::from_fmu(&path, inputs, variables, spec.tolerance, &self.config).map_err(|e| format!("plant {}: {e}", spec.label()))?;
            self.built.insert(key.clone(), Box::new(model));
        }
        Ok(self.built.get_mut(&key).expect("just built").as_mut())
    }

    /// Build the FMU of every plant of the cases chosen (`only`, else all),
    /// before a stage runs where OpenModelica is not (a Docker image): the
    /// paths built, and the first error.
    #[cfg(feature = "native")]
    pub fn prepare_all(&self, ast: &crate::compiler::ast::Model, plan: &Plan, bindings: &[super::binding::Binding], only: Option<&str>) -> Result<Vec<PathBuf>, String> {
        let mut out = Vec::new();
        for test in ast.test_cases.iter().filter(|t| only.is_none_or(|o| t.id == o || t.name == o)) {
            if let Some(Ok(case)) = super::case::read(test, plan, bindings) {
                if let Some(spec) = &case.plant {
                    out.push(self.prepare(ast, spec).map_err(|e| format!("test_case '{}': {e}", test.name))?);
                }
            }
        }
        Ok(out)
    }

    /// How the plant of a case is described in a verdict's tools.
    pub fn describe(case: &Case) -> String {
        match &case.plant {
            Some(spec) => format!("; in the loop with the plant {} (FMI 2.0 co-simulation)", spec.label()),
            None => String::new(),
        }
    }
}

/// The model's own loop around the plant of a case: the vectors as the
/// plant filled them and the model's table — the reference of every
/// implementation, what the batch stages replay.
pub fn model_loop(ast: &crate::compiler::ast::Model, plan: &Plan, case: &Case, plants: &mut Plants) -> Result<(Vectors, Table), String> {
    model_loop_held(ast, plan, case, plants, false)
}

/// The model in the loop, its outputs reaching the plant now or (`late`) one period late (see [`run_held`]).
pub fn model_loop_held(ast: &crate::compiler::ast::Model, plan: &Plan, case: &Case, plants: &mut Plants, late: bool) -> Result<(Vectors, Table), String> {
    let plant = plants.get(ast, case)?;
    let mut runner = reference::Runner::new(plan, case.step_us);
    let run = run_held(plan, case, plant, &mut runner, None, late)?;
    Ok((run.vectors, run.table))
}

/// A harness run as a program on this computer, one line of inputs a
/// step on its stdin, one row of outputs a step on its stdout (the C of
/// `arclang sil`, the fixed-point reference).
pub struct Program<'a> {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    lines: std::sync::mpsc::Receiver<String>,
    line: Box<dyn Fn(&Vectors, usize) -> String + 'a>,
    names: Vec<String>,
    period_us: u64,
}

impl<'a> Program<'a> {
    /// Starts `binary`, telling it the steps to come; `line` writes a step's inputs as the harness reads them.
    pub fn start(binary: &Path, steps: usize, names: Vec<String>, period_us: u64, line: Box<dyn Fn(&Vectors, usize) -> String + 'a>) -> Result<Program<'a>, String> {
        use std::io::{BufRead, Write};
        let mut child = std::process::Command::new(binary).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).spawn().map_err(|e| format!("cannot run {}: {e}", binary.display()))?;
        let mut stdin = child.stdin.take().expect("piped");
        let stdout = child.stdout.take().expect("piped");
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for l in std::io::BufReader::new(stdout).lines().map_while(Result::ok) {
                if tx.send(l).is_err() {
                    break;
                }
            }
        });
        writeln!(stdin, "{steps}").and_then(|_| stdin.flush()).map_err(|e| format!("cannot feed the C run: {e}"))?;
        Ok(Program { child, stdin, lines: rx, line, names, period_us })
    }

    fn fail(&mut self, what: String) -> String {
        let _ = self.child.kill();
        let _ = self.child.wait();
        what
    }
}

impl Device for Program<'_> {
    fn step(&mut self, vectors: &Vectors, k: usize) -> Result<Vec<f64>, String> {
        use std::io::Write;
        let line = (self.line)(vectors, k);
        if let Err(e) = writeln!(self.stdin, "{line}").and_then(|_| self.stdin.flush()) {
            return Err(self.fail(format!("cannot feed the C run at step {k}: {e}")));
        }
        let answer = match self.lines.recv_timeout(std::time::Duration::from_secs(60)) {
            Ok(l) => l,
            Err(_) => return Err(self.fail(format!("the C run gave no row for step {k}"))),
        };
        if let Some(error) = answer.strip_prefix("ERROR ") {
            return Err(self.fail(error.to_string()));
        }
        let table = crate::compiler::embedded::sil::parse_rows(&answer, &self.names, self.period_us).map_err(|e| self.fail(e))?;
        table.rows.into_iter().next().ok_or_else(|| "the C run gave an empty row".to_string())
    }
}

impl Drop for Program<'_> {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// A plant for the tests: first-order lags, one per output, each following
/// the input of the same index with the time constant `tau` (an input past
/// the outputs is ignored; an output past the inputs holds 0).
pub struct Lag {
    pub tau: f64,
    pub outputs: usize,
    state: Vec<f64>,
    t: f64,
}

impl Lag {
    pub fn new(tau: f64, outputs: usize) -> Lag {
        Lag { tau, outputs, state: vec![0.0; outputs], t: 0.0 }
    }
}

impl Plant for Lag {
    fn restart(&mut self) -> Result<(), String> {
        self.state = vec![0.0; self.outputs];
        self.t = 0.0;
        Ok(())
    }
    fn init(&mut self, _inputs: &[f64]) -> Result<Vec<f64>, String> {
        Ok(self.state.clone())
    }
    fn advance(&mut self, t: f64, inputs: &[f64]) -> Result<Vec<f64>, String> {
        let h = t - self.t;
        self.t = t;
        for (j, s) in self.state.iter_mut().enumerate() {
            let u = inputs.get(j).copied().unwrap_or(0.0);
            *s += (u - *s) * (1.0 - (-h / self.tau).exp());
        }
        Ok(self.state.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::embedded::{binding, plan};
    use crate::{Compiler, CompilerConfig};

    const MODEL: &str = r#"
model Loop { metadata { version: "1.0.0" } }
requirements system {
    req "SYS-1" "The room warms" { description: "The heater shall bring the room to its setpoint" }
}
state_machine "Heater" {
    variables: ["in temperature: Real [K] range 0 K .. 400 K", "out power: Real [W] = 0 W", "const setpoint: Real [K] = 80 K", "const k_p: Real [W/K] = 10 W/K"]
    equations: ["power = max(0 W, min(100 W, k_p * (setpoint - temperature)))"]
    initial: "On"
    state On {}
}
test_case "TC-LOOP" {
    verifies: ["SYS-1"]
    execute: {
        machine: "Heater"
        step: 100 ms
        stop_time: 2 s
        plant: {
            model: "Physics.Room"
            sources: ["physics/Room.mo"]
            inputs: ["heat <- power"]
            outputs: ["temperature <- T"]
        }
    }
    expects: ["final(\"Heater\".power) < 100 W"]
}
"#;

    fn compiled(text: &str) -> crate::compiler::ast::Model {
        Compiler::new(CompilerConfig::default()).compile_string(text).expect("compiles").ast
    }

    #[test]
    fn the_loop_feeds_the_plant_to_the_machine_and_the_machine_to_the_plant() {
        let ast = compiled(MODEL);
        let p = plan::plan(&ast).unwrap();
        let bindings = binding::resolve_all(&ast, &p).unwrap();
        let case = super::super::case::read(&ast.test_cases[0], &p, &bindings).unwrap().unwrap();
        let spec = case.plant.as_ref().unwrap();
        assert_eq!(spec.label(), "Physics.Room");
        assert_eq!(spec.sources, vec!["physics/Room.mo"]);
        let mut plant = Lag::new(0.5, 1);
        let mut model = reference::Runner::new(&p, case.step_us);
        let run = run(&p, &case, &mut plant, &mut model, None).unwrap();
        assert_eq!(run.table.rows.len(), 21);
        let temperature = column(&p, 0, 0);
        let power = column(&p, 0, 1);
        // Cold at first: full power; the room warms along the lag; the power eases off.
        assert_eq!(run.table.rows[0][temperature], 0.0);
        assert_eq!(run.table.rows[0][power], 100.0);
        assert!(run.table.rows[20][temperature] > 50.0, "{:?}", run.table.rows[20]);
        assert!(run.table.rows[20][power] < 100.0);
        // The vectors carry what the plant gave, so a batch stage replays the same run.
        assert_eq!(run.vectors.values[0][0][20].number(), run.table.rows[20][temperature]);
        assert_eq!(run.plant[20][0], run.table.rows[20][temperature]);
        let replay = reference::run(&p, &run.vectors, case.step_us);
        assert_eq!(replay.rows, run.table.rows);
    }

    #[test]
    fn a_plant_is_refused_when_its_wires_are_wrong() {
        let refused = |plant: &str| -> String {
            let text = MODEL.replace(
                "plant: {\n            model: \"Physics.Room\"\n            sources: [\"physics/Room.mo\"]\n            inputs: [\"heat <- power\"]\n            outputs: [\"temperature <- T\"]\n        }",
                plant,
            );
            assert_ne!(text, MODEL, "the replacement applies");
            // The compiler reads every executed case: a wrong plant is a compile error.
            Compiler::new(CompilerConfig::default()).compile_string(&text).map(|_| ()).unwrap_err().to_string()
        };
        assert!(refused("plant: { model: \"P.R\" inputs: [\"heat <- temperature\"] }").contains("is an input of the machine"));
        assert!(refused("plant: { model: \"P.R\" outputs: [\"power <- T\"] }").contains("is not an input of state_machine"));
        assert!(refused("plant: { model: \"P.R\" outputs: [\"temperature T\"] }").contains("<left> <- <right>"));
        assert!(refused("plant: { model: \"P.R\" }").contains("wire it to the machine"));
        assert!(refused("plant: { fmu: \"a.fmu\" model: \"P.R\" outputs: [\"temperature <- T\"] }").contains("not both"));
        assert!(refused("plant: { model: \"P.R\" outputs: [\"temperature <- T\"] extra: 1 }").contains("is not one of"));
        assert!(refused("plant: { model: \"P.R\" outputs: [\"temperature <- T\"] tolerance: 0 }").contains("positive number"));
    }
}
