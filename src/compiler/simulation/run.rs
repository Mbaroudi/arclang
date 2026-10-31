//! `arclang verify`: export, simulate, read the result, evaluate, cache.
//! And, on every compilation of a file, the cached verdicts attached to
//! the model without simulating anything.

use super::expects::{self, Context};
use super::results::{units_from_init_xml, Series};
use super::tool::{self, Tool, ToolConfig};
use super::sweep::{self, Point};
use super::{cache, expects_of, is_simulated, replay, spec_of, Outcome, PointOutcome, Spec, Status};
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
        let inconclusive = |reason: String| Status::new("inconclusive", reason, "");
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
                    Some(outcome) => {
                        let mut status = outcome.status();
                        if outcome.names_requirements() {
                            for written in &test.verifies {
                                let target = crate::compiler::verification::requirement_key(&result.ast, written);
                                let checks = |tag: Option<&str>| tag.is_none_or(|t| crate::compiler::verification::requirement_key(&result.ast, t) == target);
                                status.by_requirement.insert(written.clone(), outcome.status_for(checks));
                            }
                        }
                        status
                    }
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

fn script(export: &Export, staged: &[String], spec: &Spec, system: Option<&str>, points: &[Point]) -> String {
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
    for (index, point) in points.iter().enumerate() {
        // -output prints each parameter at the end: the proof the override took.
        let overrides = if point.is_empty() {
            String::new()
        } else {
            format!(", simflags=\"-override={} -output={}\"", sweep::override_flag(point), point.iter().map(|(v, _)| v.as_str()).collect::<Vec<_>>().join(","))
        };
        lines.push(format!(
            "print(\"@@simulate {index}\\n\");\nsimulate({}, startTime={}, stopTime={}, numberOfIntervals={}, tolerance={}, outputFormat=\"csv\", fileNamePrefix=\"{RESULT_PREFIX}_{index}\"{overrides});",
            spec.model,
            names::real(spec.start_time),
            names::real(spec.stop_time),
            spec.intervals.unwrap_or(500),
            names::real(spec.tolerance.unwrap_or(1e-6))
        ));
        lines.push("print(getErrorString());".to_string());
    }
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
    let points = match sweep::parse(&spec.sweep, &export.instances) {
        Ok(axes) => sweep::grid(&axes),
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
    let _ = std::fs::write(dir.join("run.mos"), script(export, &staged, spec, system.as_deref(), &points));
    let run = tool::run(tool, &dir, "run.mos", config.timeout);
    let results: Vec<(Option<String>, String)> = (0..points.len())
        .map(|i| {
            let csv = std::fs::read_to_string(dir.join(format!("{RESULT_PREFIX}_{i}_res.csv"))).ok().filter(|c| !c.trim().is_empty());
            (csv, std::fs::read_to_string(dir.join(format!("{RESULT_PREFIX}_{i}_init.xml"))).unwrap_or_default())
        })
        .collect();
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
    let mut notes = Vec::new();
    let mut outcomes = Vec::new();
    for (index, (point, (csv, xml))) in points.iter().zip(&results).enumerate() {
        let evaluated = match csv {
            None => Err(format!("the simulation of {} failed: {}", spec.model, simulation_messages(&run.output, index))),
            Some(csv) => applied(point, point_section(&run.output, index)).and_then(|()| evaluate_point(csv, xml, &parsed, &mut notes)),
        };
        outcomes.push(match evaluated {
            Ok(values) => PointOutcome { label: sweep::label(point), holds: values.iter().all(|v| v.holds), values, error: String::new() },
            Err(error) => PointOutcome { label: sweep::label(point), holds: false, values: Vec::new(), error },
        });
    }
    notes.sort();
    notes.dedup();
    // The results are kept for the workbench to replay them on the diagram.
    let labels: Vec<String> = points.iter().map(sweep::label).collect();
    if let Err(error) = replay::save(&base.join(".arclang").join("sim-results"), &outcome.test_case, &outcome.key, &labels, &results, &export.parts, &parsed) {
        notes.push(format!("the results could not be kept for replay: {error}"));
    }
    if !notes.is_empty() {
        outcome.log.push_str(&format!("\n{}\n", notes.join("\n")));
    }
    conclude(outcome, outcomes);
}

/// The results kept of the last run of a case, for the workbench to replay.
pub fn replay(result: &CompilationResult, model_path: &Path, case: &str, point: usize) -> Result<replay::Replay, String> {
    let test = result.ast.test_cases.iter().filter(|t| is_simulated(t)).find(|t| t.id == case || t.name == case).ok_or_else(|| format!("no test_case '{case}' that simulates (simulate: {{ ... }})"))?;
    let export = modelica::export(&result.ast)?;
    let spec = spec_of(&test.attributes).expect("simulated").map_err(|e| format!("test_case '{}': {e}", test.name))?;
    let expects = expects_of(&test.attributes);
    let key = cache::key(&export.text, &cache::model_dir(model_path), &spec, &expects);
    let parsed: Vec<_> = expects.iter().map(|e| expects::parse(e, &export.instances)).collect::<Result<_, _>>()?;
    replay::load(&replay::dir(model_path), &test.id, point, &key, &export.parts, &parsed)
}

/// What omc printed for one point: from its `@@simulate <i>` to the next.
fn point_section(output: &str, index: usize) -> &str {
    let after = output.split(&format!("@@simulate {index}\n")).nth(1).unwrap_or("");
    after.split("@@simulate ").next().unwrap_or("")
}

/// Whether the simulation ran with the parameters of the point, as `-output`
/// printed them (`time=...,body.ambient=253.15`): omc only warns about a
/// name it does not know, and runs anyway.
fn applied(point: &Point, section: &str) -> Result<(), String> {
    if point.is_empty() {
        return Ok(());
    }
    let printed: Vec<(&str, f64)> = section
        .lines()
        .filter(|l| l.starts_with("time="))
        .last()
        .map(|l| l.split(',').filter_map(|pair| pair.split_once('=')).filter_map(|(k, v)| v.trim().parse().ok().map(|v| (k.trim(), v))).collect())
        .unwrap_or_default();
    for (variable, level) in point {
        let given = printed.iter().find(|(k, _)| k == variable).map(|(_, v)| *v).ok_or_else(|| format!("sweep: '{variable}' is not a variable of the simulated model: OpenModelica ignored it"))?;
        if (given - level.si).abs() > 1e-6 * level.si.abs().max(1.0) {
            return Err(format!("sweep: '{variable}' stayed {given} instead of {}: it is not a parameter the simulation can change (a constant, a final or an evaluated parameter)", level.si));
        }
    }
    Ok(())
}

/// The expectations on the result of one point.
fn evaluate_point(csv: &str, xml: &str, parsed: &[expects::Expect], notes: &mut Vec<String>) -> Result<Vec<expects::Evaluated>, String> {
    let mut series = Series::from_csv(csv)?;
    series.set_units(units_from_init_xml(xml));
    parsed.iter().map(|expect| expects::evaluate(expect, &mut Context { series: &series, notes })).collect()
}

/// The verdict of a case from its points: an error at one is an error, a failure at one a failure.
fn conclude(outcome: &mut Outcome, points: Vec<PointOutcome>) {
    let swept = points.len() > 1 || points.first().is_some_and(|p| !p.label.is_empty());
    let (verdict, reason, values) = super::conclusion(&points);
    outcome.verdict = verdict;
    if outcome.verdict != "pass" {
        outcome.reason = reason;
    }
    if outcome.verdict != "error" {
        outcome.values = values;
    }
    if swept {
        outcome.points = points;
    }
}

/// The `messages` of the simulation record omc prints, and the errors after it.
fn simulation_messages(output: &str, index: usize) -> String {
    let after = point_section(output, index);
    let messages = after.lines().find_map(|l| l.trim().strip_prefix("messages = ")).map(|m| m.trim_end_matches(',').trim_matches('"').to_string()).unwrap_or_default();
    let errors: Vec<&str> = after.lines().filter(|l| l.contains("Error")).take(4).collect();
    [messages].into_iter().chain(errors.into_iter().map(str::to_string)).filter(|m| !m.is_empty()).collect::<Vec<_>>().join(" | ")
}

fn error_verdict(outcome: &mut Outcome, reason: String) {
    outcome.verdict = "error".to_string();
    outcome.reason = reason;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::simulation::expects::Evaluated;

    fn point() -> Point {
        let axes = sweep::parse(&["body.ambient = 253.15 K".to_string(), "vcu.speed = 90 km/h".to_string()], &Default::default()).unwrap();
        sweep::grid(&axes).remove(0)
    }

    #[test]
    fn an_override_is_proven_by_what_the_simulation_printed() {
        let section = "LOG_SUCCESS | ok\ntime=600,body.ambient=253.15000000000000568,vcu.speed=25\nLOG_SUCCESS | done\n";
        assert_eq!(applied(&point(), section), Ok(()));
        let ignored = "LOG_STDOUT | warning | override variable name not found in model: vcu.speed\ntime=600,body.ambient=253.15\n";
        assert!(applied(&point(), ignored).unwrap_err().contains("'vcu.speed' is not a variable"));
        let stuck = "time=600,body.ambient=263.15,vcu.speed=25\n";
        assert!(applied(&point(), stuck).unwrap_err().contains("stayed 263.15 instead of 253.15"));
        assert_eq!(applied(&Vec::new(), ""), Ok(()), "no sweep: nothing to prove");
    }

    #[test]
    fn each_point_runs_in_one_script_with_its_overrides() {
        let export = modelica::Export { file_name: "P.mo".into(), ..Default::default() };
        let spec = Spec { backend: Default::default(), model: "Phys.Sys".into(), configuration: None, start_time: 0.0, stop_time: 10.0, sources: vec![], tolerance: None, intervals: None, sweep: vec![] };
        let text = script(&export, &[], &spec, None, &[Vec::new()]);
        assert!(text.contains("@@simulate 0\\n") && text.contains("fileNamePrefix=\"arclang_sim_0\")"), "{text}");
        assert!(!text.contains("simflags"));
        let text = script(&export, &[], &spec, None, &[point(), point()]);
        assert!(text.contains("@@simulate 1\\n"));
        assert!(text.contains("simflags=\"-override=body.ambient=253.15,vcu.speed=25 -output=body.ambient,vcu.speed\""), "{text}");
    }

    fn evaluated(holds: bool) -> Evaluated {
        Evaluated { expect: "min(soc) >= 0.1".into(), value: format!("min(soc) = {}", if holds { 0.2 } else { 0.05 }), holds, margin: String::new(), requirement: None }
    }

    #[test]
    fn a_case_passes_only_when_every_point_holds() {
        let at = |label: &str, holds: bool| PointOutcome { label: label.into(), values: vec![evaluated(holds)], holds, error: String::new() };
        let mut outcome = Outcome::default();
        conclude(&mut outcome, vec![at("t = 1", true), at("t = 2", false), at("t = 3", false)]);
        assert_eq!(outcome.verdict, "fail");
        assert!(outcome.reason.starts_with("2 of 3 points fail; first at t = 2: min(soc) = 0.05"), "{}", outcome.reason);
        assert_eq!(outcome.points.len(), 3);
        let mut outcome = Outcome::default();
        conclude(&mut outcome, vec![at("t = 1", true), PointOutcome { label: "t = 2".into(), error: "the simulation failed".into(), ..Default::default() }]);
        assert_eq!((outcome.verdict.as_str(), outcome.reason.as_str()), ("error", "at t = 2: the simulation failed"));
        let mut outcome = Outcome::default();
        conclude(&mut outcome, vec![at("", true)]);
        assert_eq!(outcome.verdict, "pass");
        assert!(outcome.points.is_empty(), "a case without sweep keeps its values only");
        let mut outcome = Outcome::default();
        conclude(&mut outcome, vec![at("", false)]);
        assert_eq!(outcome.reason, "min(soc) = 0.05 (expected min(soc) >= 0.1)", "unchanged without a sweep");
    }
}
