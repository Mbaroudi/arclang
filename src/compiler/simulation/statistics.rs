//! `arclang verify model.arc --test CASE --statistics r|python`: the samples
//! of a Monte Carlo case, analysed further with R or Python, without
//! simulating again.
//!
//! The analysis reads `samples.csv`, which `arclang verify` keeps beside the
//! replay results, and writes `statistics.json` beside it: for each margin,
//! the share of failing samples, a normal fit and the failure probability it
//! extrapolates, and the influence of each parameter (PRCC and SRRC with R,
//! base R only; PRCC with Python and NumPy, and Borgonovo's delta and
//! Sobol's S1 when SALib is installed), with intervals. It informs the engineer; the verdict, and the
//! gate, stay the compiler's, so neither ever needs R or Python.
//!
//! The interpreters are `Rscript` and `python3` on the PATH, or
//! `ARCLANG_RSCRIPT` and `ARCLANG_PYTHON`. The scripts are part of ArcLang
//! (written to a temporary directory for each run); no model text reaches
//! them but the samples table.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const PYTHON_SCRIPT: &str = include_str!("statistics/analysis.py");
const R_SCRIPT: &str = include_str!("statistics/analysis.R");
const TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Engine {
    R,
    Python,
}

impl Engine {
    pub fn parse(text: &str) -> Option<Engine> {
        match text.to_ascii_lowercase().as_str() {
            "r" => Some(Engine::R),
            "python" | "py" => Some(Engine::Python),
            _ => None,
        }
    }

    fn program(self) -> String {
        let (variable, default) = match self {
            Engine::R => ("ARCLANG_RSCRIPT", "Rscript"),
            Engine::Python => ("ARCLANG_PYTHON", "python3"),
        };
        std::env::var(variable).ok().filter(|p| !p.trim().is_empty()).unwrap_or_else(|| default.to_string())
    }

    fn script(self) -> (&'static str, &'static str) {
        match self {
            Engine::R => ("analysis.R", R_SCRIPT),
            Engine::Python => ("analysis.py", PYTHON_SCRIPT),
        }
    }

    fn install_hint(self) -> &'static str {
        match self {
            Engine::R => "install R (https://cran.r-project.org), base R is enough; or set ARCLANG_RSCRIPT",
            Engine::Python => "install Python 3 with NumPy (pip install numpy SALib); or set ARCLANG_PYTHON",
        }
    }
}

/// Analyse the samples kept in `case_dir`; the report, also written there.
pub fn analyse(case_dir: &Path, engine: Engine) -> Result<serde_json::Value, String> {
    let samples = case_dir.join("samples.csv");
    if !samples.is_file() {
        return Err(format!("no samples in {}: run `arclang verify` on a case with uncertain: first", case_dir.display()));
    }
    // The interpreter may run elsewhere (another directory, a container): an absolute path.
    let samples = samples.canonicalize().map_err(|e| format!("{}: {e}", samples.display()))?;
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let work = std::env::temp_dir().join(format!("arclang-stats-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&work).map_err(|e| format!("cannot create {}: {e}", work.display()))?;
    let (name, text) = engine.script();
    let script = work.join(name);
    let output = work.join("statistics.json");
    let outcome = std::fs::write(&script, text).map_err(|e| e.to_string()).and_then(|()| run(engine, &script, &samples, &output));
    let report = outcome.and_then(|()| std::fs::read_to_string(&output).map_err(|e| format!("the analysis wrote no report: {e}")));
    let _ = std::fs::remove_dir_all(&work);
    let report: serde_json::Value = serde_json::from_str(&report?).map_err(|e| format!("the analysis report does not read as JSON: {e}"))?;
    let kept: PathBuf = case_dir.join("statistics.json");
    std::fs::write(&kept, serde_json::to_string_pretty(&report).unwrap_or_default() + "\n").map_err(|e| format!("cannot write {}: {e}", kept.display()))?;
    Ok(report)
}

fn run(engine: Engine, script: &Path, samples: &Path, output: &Path) -> Result<(), String> {
    let program = engine.program();
    let mut child = Command::new(&program)
        .arg(script)
        .arg(samples)
        .arg(output)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("{program} does not run ({e}): {}", engine.install_hint()))?;
    let deadline = Instant::now() + TIMEOUT;
    loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) => {
                let out = child.wait_with_output().map_err(|e| e.to_string())?;
                if status.success() {
                    return Ok(());
                }
                let stderr = String::from_utf8_lossy(&out.stderr);
                let last: Vec<&str> = stderr.lines().filter(|l| !l.trim().is_empty()).rev().take(3).collect();
                let hint = if stderr.contains("No module named") || stderr.contains("there is no package") { format!(": {}", engine.install_hint()) } else { String::new() };
                return Err(format!("{program} failed{hint}: {}", last.into_iter().rev().collect::<Vec<_>>().join(" | ")));
            }
            None if Instant::now() > deadline => {
                let _ = child.kill();
                return Err(format!("{program} took more than {} s", TIMEOUT.as_secs()));
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

/// The report as lines for the terminal.
pub fn summary(report: &serde_json::Value) -> Vec<String> {
    let mut lines = vec![format!("  Engine: {}", report["engine"].as_str().unwrap_or("?"))];
    for margin in report["margins"].as_array().into_iter().flatten() {
        let percent = |key: &str| match margin[key].as_f64() {
            None => "?".to_string(),
            Some(p) if p > 0.0 && p < 1e-4 => format!("{p:.1e}"),
            Some(p) => format!("{:.2} %", p * 100.0),
        };
        lines.push(format!("\n  {}", margin["expect"].as_str().unwrap_or("?")));
        lines.push(format!("    {} samples, {} fail; a normal fit of the margin puts the failure probability at {}", margin["samples"], percent("failure_fraction"), percent("fitted_failure_probability")));
        if margin["constant"].as_bool() == Some(true) {
            lines.push("    its margin does not vary over the samples (the run ends on its bound): no parameter moves it".to_string());
        }
        for influence in margin["influences"].as_array().into_iter().flatten() {
            let number = |key: &str| influence[key].as_f64().map_or("undefined".to_string(), |v| format!("{v:+.3}"));
            lines.push(format!("    {:<6} {:<40} {}  [{} .. {}]", influence["method"].as_str().unwrap_or("?"), influence["parameter"].as_str().unwrap_or("?"), number("value"), number("low"), number("high")));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_engine_is_named_and_a_missing_one_says_how_to_install_it() {
        assert_eq!(Engine::parse("R"), Some(Engine::R));
        assert_eq!(Engine::parse("python"), Some(Engine::Python));
        assert_eq!(Engine::parse("matlab"), None);
        let dir = tempfile::tempdir().unwrap();
        assert!(analyse(dir.path(), Engine::R).unwrap_err().contains("no samples"));
        std::fs::write(dir.path().join("samples.csv"), "sample,\"a.x\",\"margin: x >= 1\",holds\n1,1,0,1\n").unwrap();
        std::env::set_var("ARCLANG_PYTHON", "/nonexistent/python-for-arclang");
        let error = analyse(dir.path(), Engine::Python).unwrap_err();
        std::env::remove_var("ARCLANG_PYTHON");
        assert!(error.contains("does not run") && error.contains("pip install numpy SALib"), "{error}");
    }
}
