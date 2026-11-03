//! A live twin that advances with the data (`advance: "step"`): the wrapper
//! is built once as an OpenModelica Co-Simulation FMU (FMI 2.0, CVODE), and a
//! small stepper (`twin_stepper.c`, compiled where the FMU was built: in the
//! OpenModelica image, or with the local compiler) advances it to each new
//! instant of the data. The model keeps its state from one sample to the
//! next and is never run again from the start of a window.
//!
//! Between two instants each driven input goes linearly from one sample to
//! the next (its value and its slope are set before the step), as
//! OpenModelica's `-csvInput` interpolates a file. An instant is stepped to
//! once the next one has arrived (an instant may come one column at a
//! time), and a sample older than the model's time drives nothing. At every
//! refresh the twin is judged over the latest window of what the model
//! computed, as a twin on a file is.

use super::results::{units_from_init_xml, Series};
use super::tool::{self, Tool, ToolConfig};
use super::twin::{self, Measurements, Outcome, Spec};
use super::twin_live::Buffer;
use crate::compiler::ast::Twin;
use crate::compiler::modelica;
use crate::compiler::quantity::UnitSpec;
use crate::compiler::CompilationResult;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

const STEPPER: &str = include_str!("twin_stepper.c");
const FMU: &str = "arclang_twin";
/// How long the stepper may take to start (compiling it included) or to make one step.
const PATIENCE: Duration = Duration::from_secs(120);

/// Extracts the FMU, compiles the stepper beside it and runs it.
fn script(tolerance: f64) -> String {
    format!(
        "set -e\nmkdir -p fmu\ncd fmu && unzip -qo ../{FMU}.fmu && cd ..\n${{CC:-cc}} -O2 -o stepper stepper.c -ldl 1>&2\nlibrary=$(ls \"$PWD\"/fmu/binaries/*/{FMU}.* | head -n 1)\nexec ./stepper \"$library\" {FMU} fmu/modelDescription.xml \"file://$PWD/fmu/resources\" {}\n",
        modelica::names::real(tolerance)
    )
}

/// The twin's model as an FMU, advanced by its stepper.
pub struct Model {
    tool: Tool,
    dir: PathBuf,
    tolerance: f64,
    child: Option<(Child, ChildStdin, Option<String>)>,
    lines: Option<Receiver<String>>,
    inputs: usize,
    /// The variables read after each step, as the run names them.
    pub variables: Vec<String>,
    pub units: BTreeMap<String, String>,
    pub instances: BTreeMap<String, String>,
    pub omc_version: String,
    pub tool_text: String,
}

impl Model {
    /// Builds the FMU and starts its stepper (tens of seconds, once).
    pub fn build(result: &CompilationResult, model_path: &Path, spec: &Spec, config: &ToolConfig) -> Result<Model, String> {
        let export = modelica::export(&result.ast).map_err(|e| format!("the model does not export to Modelica: {e}"))?;
        let variables = twin::read_variables(spec, &export.instances)?;
        let tool = tool::locate(config)?;
        let (dir, staged) = twin::stage(&export, spec, &super::cache::model_dir(model_path))?;
        let mut lines = twin::load_lines(&export, &staged);
        lines.push("setCommandLineOptions(\"--fmiFlags=s:cvode\");".to_string());
        lines.push(format!("print(\"@@fmu \" + buildModelFMU({}, version=\"2.0\", fmuType=\"cs\", fileNamePrefix=\"{FMU}\") + \"\\n\");", "ArcLangTwin"));
        lines.push("print(getErrorString());".to_string());
        let built = (|| -> Result<String, String> {
            std::fs::write(dir.join("twin_fmu.mos"), lines.join("\n") + "\n").map_err(|e| e.to_string())?;
            let run = tool::run(&tool, &dir, "twin_fmu.mos", config.timeout)?;
            if !dir.join(format!("{FMU}.fmu")).exists() {
                return Err(twin::failure(&run.output, "built as an FMU"));
            }
            std::fs::write(dir.join("stepper.c"), STEPPER).map_err(|e| e.to_string())?;
            Ok(run.output.lines().find_map(|l| l.strip_prefix("@@version ")).unwrap_or("").trim().to_string())
        })();
        let omc_version = match built {
            Ok(version) => version,
            Err(error) => {
                let _ = std::fs::remove_dir_all(&dir);
                return Err(error);
            }
        };
        let mut model = Model {
            tool_text: format!("{}, FMU stepped", tool.describe()),
            tool,
            dir,
            tolerance: spec.tolerance.unwrap_or(1e-6),
            child: None,
            lines: None,
            inputs: spec.drives.len(),
            variables,
            units: BTreeMap::new(),
            instances: export.instances,
            omc_version,
        };
        model.start()?;
        model.units = units_from_init_xml(&std::fs::read_to_string(model.dir.join("fmu/modelDescription.xml")).unwrap_or_default());
        Ok(model)
    }

    /// Starts the stepper (again, after a failure): the model is then initialized anew.
    pub fn start(&mut self) -> Result<(), String> {
        self.stop();
        std::fs::write(self.dir.join("twin_step.sh"), script(self.tolerance)).map_err(|e| e.to_string())?;
        let (mut child, container) = tool::spawn(&self.tool, &self.dir, "twin_step.sh", &self.dir.join("stepper.log"))?;
        let stdin = child.stdin.take().ok_or("the stepper has no input")?;
        let stdout = child.stdout.take().ok_or("the stepper has no output")?;
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if line.starts_with("@@") && sender.send(line).is_err() {
                    return;
                }
            }
        });
        self.child = Some((child, stdin, container));
        self.lines = Some(receiver);
        self.answer()?;
        let inputs: Vec<String> = (0..self.inputs).map(|i| format!("twin_in_{i}")).collect();
        self.ask(&format!("in {}", inputs.join(" ")))?;
        self.ask(&format!("out {}", self.variables.join(" ")))?;
        Ok(())
    }

    fn stop(&mut self) {
        if let Some((mut child, mut stdin, container)) = self.child.take() {
            let _ = writeln!(stdin, "quit");
            drop(stdin);
            let deadline = Instant::now() + Duration::from_secs(2);
            while Instant::now() < deadline && matches!(child.try_wait(), Ok(None)) {
                std::thread::sleep(Duration::from_millis(50));
            }
            let _ = child.kill();
            let _ = child.wait();
            if let Some(container) = container {
                let _ = std::process::Command::new("docker").args(["rm", "-f", &container]).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
            }
        }
        self.lines = None;
    }

    /// The stepper's next answer: `@@ok` and its numbers, or why it failed.
    fn answer(&mut self) -> Result<Vec<f64>, String> {
        let lines = self.lines.as_ref().ok_or("the stepper is not running")?;
        let line = match lines.recv_timeout(PATIENCE) {
            Ok(line) => line,
            Err(RecvTimeoutError::Timeout) => return Err(format!("the stepper has not answered for {} s", PATIENCE.as_secs())),
            Err(RecvTimeoutError::Disconnected) => {
                let log = std::fs::read_to_string(self.dir.join("stepper.log")).unwrap_or_default();
                let last: Vec<&str> = log.lines().filter(|l| !l.trim().is_empty()).rev().take(3).collect();
                return Err(format!("the stepper stopped{}", if last.is_empty() { String::new() } else { format!(": {}", last.into_iter().rev().collect::<Vec<_>>().join(" ")) }));
            }
        };
        if let Some(why) = line.strip_prefix("@@error ") {
            return Err(format!("the model: {why}"));
        }
        let rest = line.strip_prefix("@@ok").or_else(|| line.strip_prefix("@@ready")).ok_or_else(|| format!("the stepper said '{line}'"))?;
        rest.split_whitespace().map(|v| v.parse::<f64>().map_err(|_| format!("the stepper said '{line}'"))).collect()
    }

    fn ask(&mut self, line: &str) -> Result<Vec<f64>, String> {
        let (_, stdin, _) = self.child.as_mut().ok_or("the stepper is not running")?;
        writeln!(stdin, "{line}").and_then(|_| stdin.flush()).map_err(|e| format!("the stepper does not listen: {e}"))?;
        self.answer()
    }

    fn numbers(values: &[f64]) -> String {
        values.iter().map(|v| format!("{v:e}")).collect::<Vec<_>>().join(" ")
    }

    /// Starts the model at `t` (model time) with these inputs; the variables after initialization.
    pub fn init(&mut self, t: f64, inputs: &[f64]) -> Result<Vec<f64>, String> {
        let answer = self.ask(&format!("init {t:e} {}", Self::numbers(inputs)))?;
        Ok(answer.into_iter().skip(1).collect())
    }

    /// Advances to `t`, the inputs going from `inputs` along `slopes`; the variables at `t`.
    pub fn step(&mut self, t: f64, inputs: &[f64], slopes: &[f64]) -> Result<Vec<f64>, String> {
        let answer = self.ask(&format!("step {t:e} {} {}", Self::numbers(inputs), Self::numbers(slopes)))?;
        Ok(answer.into_iter().skip(1).collect())
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        self.stop();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The model following the data: what it computed at each instant stepped to.
pub struct Stepping {
    pub model: Model,
    drives: Vec<(String, Option<&'static UnitSpec>)>,
    /// The data's instant of the model's time 0.
    origin: Option<f64>,
    /// The instant (data time) the model is at.
    at: f64,
    /// The drives at that instant, as measured (in their columns' units).
    last: Vec<Option<f64>>,
    results: BTreeMap<i64, Vec<f64>>,
}

impl Stepping {
    pub fn new(model: Model, spec: &Spec) -> Stepping {
        let drives: Vec<(String, Option<&'static UnitSpec>)> = spec.drives.iter().map(|d| (d.column.clone(), d.unit)).collect();
        let last = vec![None; drives.len()];
        Stepping { model, drives, origin: None, at: f64::NEG_INFINITY, last, results: BTreeMap::new() }
    }

    /// Steps the model to every instant of the buffer it has not reached, the latest one only
    /// when `complete` (it may still be arriving). The number of steps made.
    pub fn advance(&mut self, buffer: &Buffer, complete: bool) -> Result<usize, String> {
        let latest = buffer.latest();
        let mut made = 0;
        for (t, row) in buffer.after(self.at) {
            if !complete && Some(t) == latest {
                break;
            }
            let measured: Vec<Option<f64>> = self.drives.iter().zip(&self.last).map(|((column, _), last)| row.get(column).copied().or(*last)).collect();
            if measured.iter().any(Option::is_none) {
                // The model starts once every drive has a value.
                self.last = measured;
                continue;
            }
            let si: Vec<f64> = self.drives.iter().zip(&measured).map(|((_, unit), v)| unit.map_or(v.unwrap_or(0.0), |u| u.to_canonical(v.unwrap_or(0.0)))).collect();
            let values = match self.origin {
                None => {
                    let values = self.model.init(0.0, &si)?;
                    self.origin = Some(t);
                    values
                }
                Some(origin) => {
                    let before: Vec<f64> = self.drives.iter().zip(&self.last).map(|((_, unit), v)| unit.map_or(v.unwrap_or(0.0), |u| u.to_canonical(v.unwrap_or(0.0)))).collect();
                    let slopes: Vec<f64> = si.iter().zip(&before).map(|(after, before)| (after - before) / (t - self.at)).collect();
                    self.model.step(t - origin, &before, &slopes)?
                }
            };
            self.results.insert((t * 1000.0).round() as i64, values);
            self.last = measured;
            self.at = t;
            made += 1;
        }
        Ok(made)
    }

    /// After a failure: the stepper again, the model initialized at the next instant.
    pub fn restart(&mut self) -> Result<(), String> {
        self.origin = None;
        self.model.start()
    }

    /// Forgets what is older than `keep` seconds before the model's instant.
    pub fn trim(&mut self, keep: f64) {
        let oldest = ((self.at - keep) * 1000.0).round() as i64;
        self.results = self.results.split_off(&oldest);
    }

    /// What the model computed since `from` (data time), times counted from `from`, as a run's CSV.
    fn series(&self, from: f64) -> Option<Series> {
        let start = (from * 1000.0).round() as i64;
        let rows: Vec<String> = self
            .results
            .range(start..)
            .map(|(k, values)| std::iter::once(*k as f64 / 1000.0 - from).chain(values.iter().copied()).map(|v| format!("{v:e}")).collect::<Vec<_>>().join(","))
            .collect();
        if rows.len() < 2 {
            return None;
        }
        let header: Vec<String> = std::iter::once("time".to_string()).chain(self.model.variables.iter().map(|v| format!("\"{v}\""))).collect();
        let mut series = Series::from_csv(&format!("{}\n{}\n", header.join(","), rows.join("\n"))).ok()?;
        series.set_units(self.model.units.clone());
        Some(series)
    }

    /// The twin judged over the window: the measurements against what the model computed.
    pub fn outcome(&self, twin: &Twin, spec: &Spec, measurements: Measurements, from: f64, label: &str, started: Instant) -> Option<Outcome> {
        let series = self.series(from)?;
        let outcome = Outcome {
            twin: twin.id.clone(),
            name: twin.name.clone(),
            verifies: twin.verifies.clone(),
            verdict: "inconclusive".to_string(),
            data: label.to_string(),
            samples: measurements.times.len(),
            duration: *measurements.times.last().unwrap_or(&0.0),
            data_start: measurements.start.clone(),
            tool: self.model.tool_text.clone(),
            omc_version: self.model.omc_version.clone(),
            ..Outcome::default()
        };
        let mut judged = twin::judge(outcome, spec, &self.model.instances, &series, &measurements);
        judged.seconds = started.elapsed().as_secs_f64();
        Some(judged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stepper_is_compiled_beside_the_fmu_it_steps() {
        let text = script(1e-6);
        assert!(text.contains("unzip -qo ../arclang_twin.fmu"));
        assert!(text.contains("${CC:-cc} -O2 -o stepper stepper.c -ldl"));
        assert!(text.contains("exec ./stepper \"$library\" arclang_twin fmu/modelDescription.xml \"file://$PWD/fmu/resources\""));
        for word in ["@@ready", "@@ok", "@@error", "fmi2SetRealInputDerivatives", "fmi2DoStep"] {
            assert!(STEPPER.contains(word), "{word}");
        }
    }
}
