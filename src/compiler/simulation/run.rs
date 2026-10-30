//! `arclang verify`: export, simulate, read the result, evaluate, cache.
//! And, on every compilation of a file, the cached verdicts attached to
//! the model without simulating anything.

use super::expects::{self, Context};
use super::results::{units_from_init_xml, Series};
use super::tool::{self, Tool, ToolConfig};
use super::{cache, expects_of, is_simulated, spec_of, Outcome, Spec, Status};
use crate::compiler::modelica::{self, names, Export};
use crate::compiler::CompilationResult;
use std::path::{Path, PathBuf};

const RESULT_PREFIX: &str = "arclang_sim";
const LOG_LIMIT: usize = 20_000;

fn configuration_id(result: &CompilationResult, spec: &Spec) -> Option<String> {
    let written = spec.configuration.as_ref()?;
    result.ast.configurations.iter().find(|c| &c.id == written || &c.name == written).map(|c| c.id.clone()).or_else(|| Some(written.clone()))
}

fn configuration_name(result: &CompilationResult, spec: &Spec) -> Option<String> {
    let written = spec.configuration.as_ref()?;
    result.ast.configurations.iter().find(|c| &c.id == written || &c.name == written).map(|c| c.name.clone())
}

/// The cached verdict of every simulated case, as this compilation sees it:
/// fresh when its key still matches, otherwise inconclusive (stale, or
/// never simulated). Nothing is simulated here.
pub fn attach(result: &mut CompilationResult, model_path: &Path) {
    if !result.ast.test_cases.iter().any(is_simulated) {
        return;
    }
    let dir = cache::dir(model_path);
    let base = cache::model_dir(model_path);
    let export = modelica::export(&result.ast);
    let mut statuses = std::collections::HashMap::new();
    let mut warnings = Vec::new();
    for test in result.ast.test_cases.iter().filter(|t| is_simulated(t)) {
        let Some(Ok(spec)) = spec_of(&test.attributes) else { continue };
        let inconclusive = |reason: String| Status { verdict: "inconclusive".to_string(), reason, evidence: String::new() };
        let status = match &export {
            Err(error) => inconclusive(format!("the model does not export to Modelica: {error}")),
            Ok(export) => {
                let key = cache::key(&export.text, &base, &spec, &expects_of(&test.attributes));
                match cache::read(&dir, &test.id) {
                    None => inconclusive(format!("never simulated: run `arclang verify {} --test {}`", model_path.display(), test.id)),
                    Some(outcome) if outcome.key != key => inconclusive(format!(
                        "stale: the model, its Modelica sources or the case changed since it was simulated; run `arclang verify {} --test {}`",
                        model_path.display(),
                        test.id
                    )),
                    Some(outcome) => outcome.status(),
                }
            }
        };
        match status.verdict.as_str() {
            "pass" => {}
            "fail" => warnings.push(format!("verification: test_case '{}' fails: {}", test.name, status.reason)),
            other => warnings.push(format!("verification: test_case '{}' is {other}: {}", test.name, status.reason)),
        }
        statuses.insert(test.id.clone(), status);
    }
    result.semantic_model.simulations = statuses;
    result.warnings.extend(warnings);
}

/// Simulate the cases of a model (one, with `only`), cache and return
/// their outcomes. An absent tool makes every case inconclusive, not cached.
pub fn verify(result: &CompilationResult, model_path: &Path, only: Option<&str>, config: &ToolConfig) -> Result<Vec<Outcome>, String> {
    let cases: Vec<_> = result.ast.test_cases.iter().filter(|t| is_simulated(t)).filter(|t| only.is_none_or(|o| t.id == o || t.name == o)).collect();
    if cases.is_empty() {
        return Err(match only {
            Some(o) => format!("no test_case '{o}' that simulates (simulate: {{ ... }})"),
            None => "no test_case simulates (simulate: { ... })".to_string(),
        });
    }
    let export = modelica::export(&result.ast)?;
    let base = cache::model_dir(model_path);
    let dir = cache::dir(model_path);
    let tool = tool::locate(config);
    // A backend that is not built answers before looking for OpenModelica.
    let mut outcomes = Vec::new();
    for test in cases {
        let spec = spec_of(&test.attributes).expect("simulated").map_err(|e| format!("test_case '{}': {e}", test.name))?;
        let expects = expects_of(&test.attributes);
        let key = cache::key(&export.text, &base, &spec, &expects);
        let mut outcome = Outcome {
            test_case: test.id.clone(),
            name: test.name.clone(),
            verifies: test.verifies.clone(),
            configuration_name: configuration_name(result, &spec),
            verdict: "inconclusive".to_string(),
            reason: String::new(),
            values: Vec::new(),
            key,
            model: spec.model.clone(),
            configuration: configuration_id(result, &spec),
            tool: String::new(),
            ..Default::default()
        };
        if !spec.backend.available() {
            outcome.reason = format!("backend '{}' is declared but not implemented yet: only 'openmodelica' runs", spec.backend.name());
            outcomes.push(outcome);
            continue;
        }
        match &tool {
            Err(reason) => outcome.reason = reason.clone(),
            Ok(tool) => {
                outcome.tool = tool.describe();
                let started = std::time::Instant::now();
                simulate(tool, &export, &base, &spec, &expects, config, &mut outcome);
                outcome.seconds = started.elapsed().as_secs_f64();
                if outcome.verdict != "inconclusive" {
                    cache::write(&dir, &outcome)?;
                }
            }
        }
        outcomes.push(outcome);
    }
    Ok(outcomes)
}

fn work_dir(key: &str) -> Result<PathBuf, String> {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("arclang-sim-{}-{}-{nanos}", &key[..12], std::process::id()));
    std::fs::create_dir_all(dir.join("sources")).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    Ok(dir)
}

/// The files of a run: the package, the sources under fixed names. No
/// text of the model reaches the script but checked class names.
fn stage(dir: &Path, export: &Export, base: &Path, spec: &Spec) -> Result<Vec<String>, String> {
    std::fs::write(dir.join(&export.file_name), &export.text).map_err(|e| e.to_string())?;
    let mut staged = Vec::new();
    for (index, source) in spec.sources.iter().enumerate() {
        let path = base.join(source);
        let content = std::fs::read(&path).map_err(|e| format!("source '{source}' cannot be read ({}): {e}", path.display()))?;
        let stem: String = Path::new(source).file_stem().and_then(|s| s.to_str()).unwrap_or("source").chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
        let name = format!("sources/{index}_{stem}.mo");
        std::fs::write(dir.join(&name), content).map_err(|e| e.to_string())?;
        staged.push(name);
    }
    Ok(staged)
}

fn script(export: &Export, staged: &[String], spec: &Spec, system: Option<&str>) -> String {
    let msl = modelica::MSL_VERSION;
    let mut lines = vec![
        "print(\"@@version \" + getVersion() + \"\\n\");".to_string(),
        format!("if not loadModel(Modelica, {{\"{msl}\"}}) then installPackage(Modelica, \"{msl}+maint.om\", exactMatch=true); end if;"),
        format!("print(\"@@msl \" + String(loadModel(Modelica, {{\"{msl}\"}})) + \"\\n\");"),
        "print(getErrorString());".to_string(),
    ];
    for file in std::iter::once(&export.file_name).chain(staged) {
        lines.push(format!("print(\"@@load {file} \" + String(loadFile(\"{file}\")) + \"\\n\");"));
        lines.push("print(getErrorString());".to_string());
    }
    if let Some(system) = system {
        lines.push(format!("print(\"@@extends \" + String(extendsFrom({}, {system})) + \"\\n\");", spec.model));
    }
    lines.push(format!(
        "print(\"@@simulate\\n\");\nsimulate({}, startTime={}, stopTime={}, numberOfIntervals={}, tolerance={}, outputFormat=\"csv\", fileNamePrefix=\"{RESULT_PREFIX}\");",
        spec.model,
        names::real(spec.start_time),
        names::real(spec.stop_time),
        spec.intervals.unwrap_or(500),
        names::real(spec.tolerance.unwrap_or(1e-6))
    ));
    lines.push("print(getErrorString());".to_string());
    lines.join("\n") + "\n"
}

fn marker<'o>(output: &'o str, prefix: &str) -> Option<&'o str> {
    output.lines().find_map(|l| l.strip_prefix(prefix)).map(str::trim)
}

fn simulate(tool: &Tool, export: &Export, base: &Path, spec: &Spec, expects: &[String], config: &ToolConfig, outcome: &mut Outcome) {
    let parsed: Result<Vec<_>, String> = expects.iter().map(|e| expects::parse(e, &export.instances)).collect();
    let parsed = match parsed {
        Ok(parsed) => parsed,
        Err(error) => return error_verdict(outcome, error),
    };
    let system = match &outcome.configuration {
        Some(id) => match export.systems.get(id) {
            Some(system) => Some(system.clone()),
            None => return error_verdict(outcome, format!("configuration '{id}' has no system model")),
        },
        None => None,
    };
    let dir = match work_dir(&outcome.key) {
        Ok(dir) => dir,
        Err(error) => return outcome.reason = error,
    };
    let staged = match stage(&dir, export, base, spec) {
        Ok(staged) => staged,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&dir);
            return error_verdict(outcome, error);
        }
    };
    let _ = std::fs::write(dir.join("run.mos"), script(export, &staged, spec, system.as_deref()));
    let run = tool::run(tool, &dir, "run.mos", config.timeout);
    let csv = std::fs::read_to_string(dir.join(format!("{RESULT_PREFIX}_res.csv")));
    let xml = std::fs::read_to_string(dir.join(format!("{RESULT_PREFIX}_init.xml"))).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&dir);
    let run = match run {
        Ok(run) => run,
        // The tool did not finish: nothing is known of the model.
        Err(error) => return outcome.reason = error,
    };
    let mut log = run.output.clone();
    if log.len() > LOG_LIMIT {
        let mut cut = log.len() - LOG_LIMIT;
        while !log.is_char_boundary(cut) {
            cut += 1;
        }
        log = format!("...{}", &log[cut..]);
    }
    outcome.log = log;
    outcome.omc_version = marker(&run.output, "@@version ").unwrap_or("").to_string();
    if outcome.omc_version.is_empty() {
        outcome.reason = format!("{} did not run omc: {}", tool.describe(), run.output.lines().last().unwrap_or("no output"));
        return;
    }
    if marker(&run.output, "@@msl ") != Some("true") {
        outcome.reason = format!("the Modelica Standard Library {} could not be loaded or installed", modelica::MSL_VERSION);
        return;
    }
    for line in run.output.lines().filter_map(|l| l.strip_prefix("@@load ")) {
        if let Some(file) = line.strip_suffix(" false") {
            return error_verdict(outcome, format!("{file} does not load in OpenModelica (see the log)"));
        }
    }
    if let (Some(system), Some(extends)) = (&system, marker(&run.output, "@@extends ")) {
        if extends != "true" {
            return error_verdict(outcome, format!("{} does not extend {system}, the system model of configuration {}", spec.model, outcome.configuration.clone().unwrap_or_default()));
        }
    }
    let csv = match csv {
        Ok(csv) if !csv.trim().is_empty() => csv,
        _ => {
            let messages = simulation_messages(&run.output);
            return error_verdict(outcome, format!("the simulation of {} failed: {messages}", spec.model));
        }
    };
    let mut series = match Series::from_csv(&csv) {
        Ok(series) => series,
        Err(error) => return error_verdict(outcome, error),
    };
    series.set_units(units_from_init_xml(&xml));
    let mut notes = Vec::new();
    let mut values = Vec::new();
    for expect in &parsed {
        match expects::evaluate(expect, &mut Context { series: &series, notes: &mut notes }) {
            Ok(value) => values.push(value),
            Err(error) => {
                outcome.values = values;
                return error_verdict(outcome, error);
            }
        }
    }
    notes.dedup();
    if !notes.is_empty() {
        outcome.log.push_str(&format!("\n{}\n", notes.join("\n")));
    }
    let failing: Vec<&str> = values.iter().filter(|v| !v.holds).map(|v| v.value.as_str()).collect();
    if failing.is_empty() {
        outcome.verdict = "pass".to_string();
    } else {
        outcome.verdict = "fail".to_string();
        outcome.reason = format!("{} (expected {})", failing.join("; "), values.iter().filter(|v| !v.holds).map(|v| v.expect.as_str()).collect::<Vec<_>>().join("; "));
    }
    outcome.values = values;
}

/// The `messages` of the simulation record omc prints, and the errors after it.
fn simulation_messages(output: &str) -> String {
    let after = output.split("@@simulate").nth(1).unwrap_or("");
    let messages = after.lines().find_map(|l| l.trim().strip_prefix("messages = ")).map(|m| m.trim_end_matches(',').trim_matches('"').to_string()).unwrap_or_default();
    let errors: Vec<&str> = after.lines().filter(|l| l.contains("Error")).take(4).collect();
    [messages].into_iter().chain(errors.into_iter().map(str::to_string)).filter(|m| !m.is_empty()).collect::<Vec<_>>().join(" | ")
}

fn error_verdict(outcome: &mut Outcome, reason: String) {
    outcome.verdict = "error".to_string();
    outcome.reason = reason;
}
