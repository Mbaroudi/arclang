//! `arclang verify`: export, simulate, read the result, evaluate, cache.
//! And, on every compilation of a file, the cached verdicts attached to
//! the model without simulating anything.

use super::expects::{self, Context};
use super::results::{units_from_init_xml, Series};
use super::tool::{self, Tool, ToolConfig};
use super::montecarlo;
use super::sweep::{self, Point};
use super::{cache, case_expects, is_simulated, replay, spec_of, Outcome, PointOutcome, Rung, Spec, Status};
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
    let executed = attach_executed(result, model_path);
    if !result.ast.test_cases.iter().any(is_simulated) {
        result.semantic_model.simulations = executed;
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
        let configuration = configuration_id(result, &spec);
        let status = match export.as_ref().map_err(|e| e.clone()).and_then(|export| super::effective(spec, &base, export, configuration).map(|spec| (export, spec))) {
            Err(error) => {
                let reason = format!("the model does not export to Modelica: {error}");
                inconclusive(reason.clone()).with_rung(Rung::missing("simulation", reason))
            }
            Ok((export, spec)) => {
                let key = cache::key(&export.text, &base, &spec, &case_expects(&result.ast, test));
                match cache::read(&dir, &test.id) {
                    None => {
                        let reason = format!("never simulated: run `arclang verify {} --test {}`", model_path.display(), test.id);
                        inconclusive(reason.clone()).with_rung(Rung::missing("simulation", reason))
                    }
                    Some(outcome) if outcome.key != key => {
                        let reason = format!(
                            "stale: the model, its Modelica sources or the case changed since it was simulated; run `arclang verify {} --test {}`",
                            model_path.display(),
                            test.id
                        );
                        inconclusive(reason.clone()).with_rung(Rung::stale("simulation", outcome.tool.clone(), reason))
                    }
                    Some(outcome) => {
                        let status = status_of(result, test, &outcome);
                        let rung = Rung::fresh("simulation", outcome.tool.clone(), &status);
                        status.with_rung(rung)
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
    statuses.extend(executed);
    statuses.extend(attach_twins(result, model_path));
    result.semantic_model.simulations = statuses;
    result.warnings.extend(warnings);
}

/// The cached verdict of every twin (`arclang twin` writes `<twin>.twin.json`),
/// keyed by the twin's id, as one rung: fresh when the model, its sources,
/// the measurements and the twin are those it ran, stale otherwise, missing
/// when never run. Nothing of it is a warning: a twin is evidence from the
/// field, not a case the model owes.
fn attach_twins(result: &CompilationResult, model_path: &Path) -> std::collections::HashMap<String, Status> {
    let mut statuses = std::collections::HashMap::new();
    if result.ast.twins.is_empty() {
        return statuses;
    }
    let dir = cache::dir(model_path);
    for twin in &result.ast.twins {
        // A twin never run has no status: the ladder reads its absence as the missing rung.
        let Some(outcome) = cache::read_kind(&dir, &twin.id, "twin") else { continue };
        let status = {
            match super::twin::cache_key(result, model_path, twin) {
                Ok(key) if key == outcome.key => {
                    let status = outcome.status();
                    let rung = Rung::fresh("twin", outcome.target.clone(), &status);
                    status.with_rung(rung)
                }
                _ => {
                    let reason = format!("stale: the model, its sources, the measurements or the twin changed since it ran; run `arclang twin {} --twin {}`", model_path.display(), twin.id);
                    Status::new("inconclusive", reason.clone(), "").with_rung(Rung::stale("twin", outcome.target.clone(), reason))
                }
            }
        };
        statuses.insert(twin.id.clone(), status);
    }
    statuses
}

/// The status of a case of the cache, for each requirement it verifies too.
/// A case that does not pass leaves at least one of its requirements
/// unverified: when the expectations kept for each all hold (the case fails
/// on what no single requirement shows, a probability over all of them, an
/// expectation naming a requirement it does not verify), its verdict is
/// that of every requirement.
fn status_of(result: &CompilationResult, test: &crate::compiler::ast::TestCase, outcome: &Outcome) -> Status {
    let mut status = outcome.status();
    if outcome.names_requirements() {
        for written in &test.verifies {
            let target = crate::compiler::verification::requirement_key(&result.ast, written);
            let checks = |tag: Option<&str>| tag.is_none_or(|t| crate::compiler::verification::requirement_key(&result.ast, t) == target);
            status.by_requirement.insert(written.clone(), outcome.status_for(checks));
        }
        if status.verdict != "pass" && status.by_requirement.values().all(|each| each.verdict == "pass") {
            let case = outcome.status();
            for each in status.by_requirement.values_mut() {
                *each = case.clone();
            }
        }
    }
    status
}

/// The cached SIL verdict of every case that executes at fixed step
/// (`arclang sil` writes them): fresh when the generated C, the case and
/// the plant it closes its loop on are those it ran, otherwise inconclusive. A failing or stale verdict is a
/// warning; a case never run is not (the compiler already judged it on the
/// model).
fn attach_executed(result: &mut CompilationResult, model_path: &Path) -> std::collections::HashMap<String, Status> {
    let mut statuses = std::collections::HashMap::new();
    let executed: Vec<_> = crate::compiler::embedded::sil::cases(&result.ast).into_iter().cloned().collect();
    if executed.is_empty() {
        return statuses;
    }
    let dir = cache::dir(model_path);
    let base = cache::model_dir(model_path);
    for test in &executed {
        let inconclusive = |reason: String| Status::new("inconclusive", reason, "");
        // What a stale verdict's sentence names: a case in the loop also follows the physics of its plant.
        let in_the_loop = !crate::compiler::embedded::plant::files(test).is_empty();
        let status = match crate::compiler::embedded::sil::cache_key(&result.ast, test, &base) {
            Err(e) => {
                let reason = format!("the C is not generated: {e}");
                inconclusive(reason.clone()).with_rung(Rung::missing("sil", reason))
            }
            Ok(key) => match cache::read(&dir, &test.id) {
                None => {
                    let reason = format!("never run in SIL: run `arclang sil {} --test {}`", model_path.display(), test.id);
                    inconclusive(reason.clone()).with_rung(Rung::missing("sil", reason))
                }
                Some(outcome) if outcome.key != key => {
                    let changed = if in_the_loop { "the generated C, the case or the plant it closes its loop on changed" } else { "the generated C or the case changed" };
                    let reason = format!("stale: {changed} since its SIL run; run `arclang sil {} --test {}`", model_path.display(), test.id);
                    result.warnings.push(format!("verification: test_case '{}' is inconclusive: {reason}", test.name));
                    inconclusive(reason.clone()).with_rung(Rung::stale("sil", outcome.tool.clone(), reason))
                }
                Some(outcome) => {
                    let status = status_of(result, test, &outcome);
                    if status.verdict != "pass" {
                        result.warnings.push(format!("verification: test_case '{}' {} in SIL: {}", test.name, if status.verdict == "fail" { "fails" } else { "is in error" }, status.reason));
                    }
                    let rung = Rung::fresh("sil", outcome.tool.clone(), &status);
                    status.with_rung(rung)
                }
            },
        };
        let status = with_cosim(result, model_path, &dir, test, status);
        let status = with_pil(result, model_path, &dir, test, status);
        let status = with_hdl(result, model_path, &dir, test, status);
        let status = with_soc(result, model_path, &dir, test, status);
        statuses.insert(test.id.clone(), status);
    }
    statuses
}

/// A case's status with the verdict of a later stage, when one is cached
/// beside its SIL verdict (`<case>.<kind>.json`): processor in the loop
/// (`arclang pil`), the logic in simulation (`arclang hdl`), the SoC or the
/// board (`arclang soc`). A fresh failure or error is the case's verdict
/// (it is the same C on the target, the hardware computes the machine),
/// and each requirement keeps the worst of what the stages say of it; a
/// fresh verdict stands for the earlier stages when none of them is fresh
/// (never run, or stale), and a pass is added to the evidence otherwise; a stale one is a warning only. Each is
/// a rung of the status, kept apart.
fn with_stage(result: &mut CompilationResult, model_path: &Path, dir: &Path, test: &crate::compiler::ast::TestCase, status: Status, kind: &str) -> Status {
    let Some(outcome) = cache::read_kind(dir, &test.id, kind) else { return status };
    let base = cache::model_dir(model_path);
    // `label` names the stage in a verdict's warning; `stale` is the sentence of a stale one.
    let (fresh, stage, label, stale, command, changed) = match kind {
        "pil" => (
            crate::compiler::embedded::target::find(&outcome.target).and_then(|package| crate::compiler::embedded::pil::cache_key(&result.ast, test, &package, &base)).is_ok_and(|key| key == outcome.key),
            "pil",
            format!("PIL on {}", outcome.target),
            format!("its PIL verdict on {} is stale", outcome.target),
            "pil",
            "the generated C, the case, its plant or the target package changed",
        ),
        "hdl" => (
            crate::compiler::embedded::hdl::cache_key(&result.ast, test, &outcome.tool, &base).is_ok_and(|key| key == outcome.key),
            "hdl",
            format!("HDL {}", outcome.target),
            "its HDL verdict is stale".to_string(),
            "hdl",
            "the generated VHDL, the fixed-point C, the case, a machine it reads or its plant changed",
        ),
        _ => (
            crate::compiler::embedded::system::cache_key(&result.ast, test, &outcome.tool, &base).is_ok_and(|key| key == outcome.key),
            if outcome.tool.contains("on the board") { "board" } else { "soc" },
            format!("{} {}", if outcome.tool.contains("on the board") { "board" } else { "SoC" }, outcome.target),
            format!("its {} verdict is stale", if outcome.tool.contains("on the board") { "board" } else { "SoC" }),
            "soc",
            "the generated VHDL, the fixed-point C, the application, the case, a machine it reads or its plant changed",
        ),
    };
    let target = if kind == "pil" { outcome.target.clone() } else { format!("{} ({})", outcome.target, outcome.tool) };
    if !fresh {
        let reason = format!("{stale} ({changed}); run `arclang {command} {} --test {}`", model_path.display(), test.id);
        result.warnings.push(format!("verification: test_case '{}': {reason}", test.name));
        return status.with_rung(Rung::stale(stage, target, reason));
    }
    let later = status_of(result, test, &outcome);
    let rung = Rung::fresh(stage, target, &later);
    if later.verdict != "pass" {
        result.warnings.push(format!("verification: test_case '{}' {} in {label}: {}", test.name, if later.verdict == "fail" { "fails" } else { "is in error" }, later.reason));
    }
    // The earlier stages say nothing (never run, or stale): this one stands
    // for the case. One that ran, is fresh and concludes nothing is not
    // stood for: a later pass does not make it conclude.
    if !status.rungs.iter().any(|r| r.state == "fresh") {
        let mut rungs = status.rungs;
        rungs.push(rung);
        return Status { rungs, ..later };
    }
    if later.verdict != "pass" {
        return status.worst_with(later).with_rung(rung);
    }
    let mut status = status;
    status.evidence = format!("{} · {label}: pass", status.evidence).trim_start_matches(" · ").to_string();
    status.with_rung(rung)
}

/// A case's status with its co-simulation verdict, when one is cached
/// beside its SIL verdict (`arclang cosim` writes `<case>.cosim.json`): the
/// same generated C, run as an FMU under OMSimulator and judged as SIL
/// judges it. Fresh, it is evidence of the SIL stage: it stands for a SIL
/// verdict missing or stale, a failure or an error is the case's verdict,
/// a pass joins the C harness's; the stage keeps one rung, naming both.
/// Stale, it is a warning only.
fn with_cosim(result: &mut CompilationResult, model_path: &Path, dir: &Path, test: &crate::compiler::ast::TestCase, status: Status) -> Status {
    let Some(outcome) = cache::read_kind(dir, &test.id, "cosim") else { return status };
    let base = cache::model_dir(model_path);
    let fresh = crate::compiler::embedded::cosim_verdict::cache_key(&result.ast, test, &base).is_ok_and(|key| key == outcome.key);
    let target = crate::compiler::embedded::cosim_verdict::target_of(&outcome);
    if !fresh {
        let reason = format!("its co-simulation verdict is stale (the generated C, the plant or the case changed); run `arclang cosim {} --test {}`", model_path.display(), test.id);
        result.warnings.push(format!("verification: test_case '{}': {reason}", test.name));
        return status;
    }
    let later = status_of(result, test, &outcome);
    let label = format!("co-simulation in {target}");
    if later.verdict != "pass" {
        result.warnings.push(format!("verification: test_case '{}' {} in {label}: {}", test.name, if later.verdict == "fail" { "fails" } else { "is in error" }, later.reason));
    }
    let sil = status.rungs.iter().find(|r| r.stage == "sil" && r.state == "fresh").map(|r| r.target.clone());
    let others: Vec<Rung> = status.rungs.iter().filter(|r| r.stage != "sil").cloned().collect();
    match sil {
        // No C harness ran (or its verdict no longer holds): the co-simulation is the stage's evidence.
        None => {
            let rung = Rung::fresh("sil", target, &later);
            Status { rungs: others.into_iter().chain([rung]).collect(), ..later }
        }
        // Both ran the C: each requirement keeps the worst of the two.
        Some(harness) if later.verdict != "pass" => {
            let rung = Rung::fresh("sil", format!("{harness} · {target}"), &later);
            Status { rungs: others.into_iter().chain([rung]).collect(), ..status.worst_with(later) }
        }
        Some(harness) => {
            let mut status = status;
            status.evidence = format!("{} · {label}: pass", status.evidence).trim_start_matches(" · ").to_string();
            let rung = Rung::fresh("sil", format!("{harness} · {target}"), &status);
            Status { rungs: others.into_iter().chain([rung]).collect(), ..status }
        }
    }
}

fn with_pil(result: &mut CompilationResult, model_path: &Path, dir: &Path, test: &crate::compiler::ast::TestCase, status: Status) -> Status {
    with_stage(result, model_path, dir, test, status, "pil")
}

fn with_hdl(result: &mut CompilationResult, model_path: &Path, dir: &Path, test: &crate::compiler::ast::TestCase, status: Status) -> Status {
    with_stage(result, model_path, dir, test, status, "hdl")
}

fn with_soc(result: &mut CompilationResult, model_path: &Path, dir: &Path, test: &crate::compiler::ast::TestCase, status: Status) -> Status {
    with_stage(result, model_path, dir, test, status, "soc")
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
        let configuration = configuration_id(result, &spec);
        let spec = super::effective(spec, &base, &export, configuration).map_err(|e| format!("test_case '{}': {e}", test.name))?;
        let expects = case_expects(&result.ast, test);
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
    let sampled = !spec.uncertain.is_empty();
    let points = if sampled {
        match montecarlo::parse(&spec.uncertain, &export.instances) {
            Ok(parameters) => montecarlo::sample(&parameters, spec.samples, spec.seed),
            Err(error) => return error_verdict(outcome, error),
        }
    } else {
        match sweep::parse(&spec.sweep, &export.instances) {
            Ok(axes) => sweep::grid(&axes),
            Err(error) => return error_verdict(outcome, error),
        }
    };
    outcome.probability = spec.required;
    let label = |index: usize, point: &Point| if sampled { format!("sample {} ({})", index + 1, sweep::label(point)) } else { sweep::label(point) };
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
    // Many runs need more than the default time: about 3 s each.
    let timeout = config.timeout.or_else(|| (points.len() > 300).then(|| std::time::Duration::from_secs(3 * points.len() as u64)));
    let run = tool::run(tool, &dir, "run.mos", timeout);
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
        let inputs = if sampled { point.iter().map(|(variable, level)| (variable.clone(), level.si)).collect() } else { Vec::new() };
        outcomes.push(match evaluated {
            Ok(values) => PointOutcome { label: label(index, point), holds: values.iter().all(|v| v.holds), values, error: String::new(), inputs },
            Err(error) => PointOutcome { label: label(index, point), holds: false, values: Vec::new(), error, inputs },
        });
    }
    notes.sort();
    notes.dedup();
    // The results are kept for the workbench to replay them on the diagram.
    let labels: Vec<String> = points.iter().enumerate().map(|(i, p)| label(i, p)).collect();
    let results_dir = base.join(".arclang").join("sim-results");
    if let Err(error) = replay::save(&results_dir, &outcome.test_case, &outcome.key, &labels, &results, &export.parts, &parsed) {
        notes.push(format!("the results could not be kept for replay: {error}"));
    }
    // The samples, as a table R or Python can read (`arclang statistics`).
    if sampled {
        let table = replay::case_dir(&results_dir, &outcome.test_case).join("samples.csv");
        if let Err(error) = std::fs::write(&table, montecarlo::samples_csv(&outcomes)) {
            notes.push(format!("the samples could not be written to {}: {error}", table.display()));
        }
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
    let configuration = configuration_id(result, &spec);
    let spec = super::effective(spec, &cache::model_dir(model_path), &export, configuration).map_err(|e| format!("test_case '{}': {e}", test.name))?;
    let expects = case_expects(&result.ast, test);
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
    let (verdict, reason, values) = super::conclusion_for(&points, outcome.probability);
    if let Some(required) = outcome.probability {
        outcome.sensitivity = montecarlo::sensitivity(&points);
        outcome.estimate = montecarlo::estimate(&points, required.confidence);
    }
    // A probabilistic pass says with what probability.
    let keep_reason = outcome.probability.is_some() && verdict == "pass";
    outcome.verdict = verdict;
    if outcome.verdict != "pass" || keep_reason {
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
        let spec = Spec { model: "Phys.Sys".into(), stop_time: 10.0, ..Default::default() };
        let text = script(&export, &[], &spec, None, &[Vec::new()]);
        assert!(text.contains("@@simulate 0\\n") && text.contains("fileNamePrefix=\"arclang_sim_0\")"), "{text}");
        assert!(!text.contains("simflags"));
        let text = script(&export, &[], &spec, None, &[point(), point()]);
        assert!(text.contains("@@simulate 1\\n"));
        assert!(text.contains("simflags=\"-override=body.ambient=253.15,vcu.speed=25 -output=body.ambient,vcu.speed\""), "{text}");
    }

    fn evaluated(holds: bool) -> Evaluated {
        Evaluated { expect: "min(soc) >= 0.1".into(), value: format!("min(soc) = {}", if holds { 0.2 } else { 0.05 }), holds, margin: String::new(), requirement: None, slack: None, slack_unit: None }
    }

    #[test]
    fn a_case_passes_only_when_every_point_holds() {
        let at = |label: &str, holds: bool| PointOutcome { label: label.into(), values: vec![evaluated(holds)], holds, error: String::new(), inputs: Vec::new() };
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
