//! A case that verifies several requirements, run at several stages (the C,
//! the co-simulation, the logic): each requirement reads the worst of what
//! the stages say of it. A stage that fails because the implementation
//! departs from the model fails every requirement of the case, whatever its
//! expectations say. Nothing runs here: the verdicts are written to the
//! cache as the commands write them.

use arclang::compiler::embedded::{cosim, hdl, sil};
use arclang::compiler::simulation::expects::Evaluated;
use arclang::compiler::simulation::{cache, Outcome};
use arclang::compiler::verification::{requirement_status, verdict_for, Standing, Verdict};
use arclang::compiler::CompilationResult;
use arclang::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};

const SPEED: &str = "examples/automotive/speed-monitor/speed_monitor.arc";
/// The case verifies both requirements, with expectations naming each.
const CASE: &str = "TC-SPD-001";
const FILTER: &str = "SYS-SPD-001";
const WARNING: &str = "SYS-SPD-002";
const HDL_TOOLS: &str = "ghdl 4.1.0";

fn compile(path: &Path) -> CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_file(path).unwrap_or_else(|e| panic!("{} compiles: {e}", path.display()))
}

fn copy_of_the_model(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-worst-verdict-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let model = dir.join("speed_monitor.arc");
    std::fs::copy(Path::new(env!("CARGO_MANIFEST_DIR")).join(SPEED), &model).unwrap();
    model
}

/// One expectation per requirement, as a stage evaluated it.
fn values(filter_holds: bool, warning_holds: bool) -> Vec<Evaluated> {
    let one = |requirement: &str, holds: bool| Evaluated {
        expect: format!("{requirement}: checked"),
        value: format!("{requirement} = {}", if holds { "as expected" } else { "off" }),
        holds,
        margin: String::new(),
        requirement: Some(requirement.to_string()),
        slack: None,
        slack_unit: None,
    };
    vec![one(FILTER, filter_holds), one(WARNING, warning_holds)]
}

/// What a stage says of the case.
struct Said {
    verdict: &'static str,
    values: Vec<Evaluated>,
    /// The implementation reproduces the model: a failure is one of the expectations alone.
    agrees: bool,
}

impl Said {
    fn pass() -> Said {
        Said { verdict: "pass", values: values(true, true), agrees: true }
    }
    /// The expectations of one requirement fail, on the implementation as on the model.
    fn fails_on(requirement: &str) -> Said {
        Said { verdict: "fail", values: values(requirement != FILTER, requirement != WARNING), agrees: true }
    }
    /// Every expectation holds, but the implementation departs from the model.
    fn departs() -> Said {
        Said { verdict: "fail", values: values(true, true), agrees: false }
    }
}

/// The verdict of a stage (`sil`, `cosim` or `hdl`) kept beside the model, fresh.
fn cached(model: &Path, kind: &str, said: Said) {
    let result = compile(model);
    let case = result.ast.test_cases.iter().find(|t| t.id == CASE).unwrap();
    let (key, tool) = match kind {
        "sil" => (sil::cache_key(&result.ast, case, model.parent().unwrap()).unwrap(), "arclang sil (cc)".to_string()),
        "cosim" => (cosim::cache_key(&result.ast, case, model.parent().unwrap()).unwrap(), "arclang cosim (OMSimulator v2.1.2)".to_string()),
        _ => (hdl::cache_key(&result.ast, case, HDL_TOOLS, model.parent().unwrap()).unwrap(), HDL_TOOLS.to_string()),
    };
    let outcome = Outcome {
        test_case: CASE.to_string(),
        kind: kind.to_string(),
        name: case.name.clone(),
        verifies: case.verifies.clone(),
        verdict: said.verdict.to_string(),
        reason: if said.verdict == "pass" { String::new() } else { format!("{kind}: it fails") },
        values: said.values,
        agrees: said.agrees,
        key,
        model: "state_machine Overspeed warning".to_string(),
        tool,
        ..Default::default()
    };
    if kind == "sil" {
        cache::write(&cache::dir(model), &outcome).unwrap();
    } else {
        cache::write_kind(&cache::dir(model), kind, &outcome).unwrap();
    }
}

/// The verdict of the case for each of its two requirements, and its own.
fn verdicts(model: &Path) -> (String, &'static str, &'static str) {
    let result = compile(model);
    let case = result.ast.test_cases.iter().find(|t| t.id == CASE).unwrap();
    let of = |requirement: &str| verdict_for(case, requirement, &result.semantic_model).word();
    (result.semantic_model.simulations[CASE].verdict.clone(), of(FILTER), of(WARNING))
}

#[test]
fn a_failure_on_the_expectations_of_one_requirement_fails_that_requirement_only() {
    let model = copy_of_the_model("split");
    cached(&model, "sil", Said::fails_on(WARNING));
    assert_eq!(verdicts(&model), ("fail".to_string(), "pass", "fail"));
}

#[test]
fn an_implementation_that_departs_from_the_model_fails_every_requirement_of_the_case() {
    let model = copy_of_the_model("departs");
    cached(&model, "sil", Said::departs());
    assert_eq!(verdicts(&model), ("fail".to_string(), "fail", "fail"));
    // The gate's status reads the same thing.
    let result = compile(&model);
    for requirement in [FILTER, WARNING] {
        assert_eq!(requirement_status(&result.ast, &result.semantic_model, requirement).standing(), Standing::Fail, "{requirement}");
    }
}

#[test]
fn a_departure_stands_even_when_an_expectation_of_another_requirement_fails_too() {
    let model = copy_of_the_model("departs-and-fails");
    cached(&model, "sil", Said { agrees: false, ..Said::fails_on(WARNING) });
    assert_eq!(verdicts(&model), ("fail".to_string(), "fail", "fail"));
}

#[test]
fn a_cached_failure_that_does_not_say_where_it_comes_from_fails_every_requirement() {
    // A verdict kept by an earlier version does not say whether the C reproduced the model.
    let model = copy_of_the_model("earlier-format");
    cached(&model, "sil", Said::fails_on(WARNING));
    let file = cache::dir(&model).join(format!("{CASE}.json"));
    let text = std::fs::read_to_string(&file).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(json.as_object_mut().unwrap().remove("agrees").is_some(), "{text}");
    std::fs::write(&file, serde_json::to_string_pretty(&json).unwrap()).unwrap();
    assert_eq!(verdicts(&model), ("fail".to_string(), "fail", "fail"));
}

#[test]
fn two_stages_failing_on_different_requirements_fail_both() {
    for later in ["cosim", "hdl"] {
        let model = copy_of_the_model(&format!("two-stages-{later}"));
        cached(&model, "sil", Said::fails_on(FILTER));
        cached(&model, later, Said::fails_on(WARNING));
        assert_eq!(verdicts(&model), ("fail".to_string(), "fail", "fail"), "SIL then {later}");
    }
}

#[test]
fn a_later_stage_that_passes_does_not_clear_the_failure_of_an_earlier_one() {
    for later in ["cosim", "hdl"] {
        let model = copy_of_the_model(&format!("later-pass-{later}"));
        cached(&model, "sil", Said::fails_on(FILTER));
        cached(&model, later, Said::pass());
        assert_eq!(verdicts(&model), ("fail".to_string(), "fail", "pass"), "SIL then {later}");
    }
}

#[test]
fn a_later_stage_that_departs_fails_every_requirement_a_passing_earlier_one_verified() {
    for later in ["cosim", "hdl"] {
        let model = copy_of_the_model(&format!("later-departs-{later}"));
        cached(&model, "sil", Said::pass());
        cached(&model, later, Said::departs());
        assert_eq!(verdicts(&model), ("fail".to_string(), "fail", "fail"), "SIL then {later}");
    }
}

#[test]
fn three_stages_keep_the_worst_of_each_requirement() {
    let model = copy_of_the_model("three-stages");
    cached(&model, "sil", Said::pass());
    cached(&model, "cosim", Said::fails_on(FILTER));
    cached(&model, "hdl", Said::fails_on(WARNING));
    assert_eq!(verdicts(&model), ("fail".to_string(), "fail", "fail"));
    let result = compile(&model);
    let case = result.ast.test_cases.iter().find(|t| t.id == CASE).unwrap();
    // Each requirement keeps the reason of the stage that fails it.
    let why = |requirement: &str| match verdict_for(case, requirement, &result.semantic_model) {
        Verdict::Fail(why) => why,
        other => panic!("{requirement}: {other:?}"),
    };
    assert!(why(FILTER).contains(FILTER), "{}", why(FILTER));
    assert!(why(WARNING).contains(WARNING), "{}", why(WARNING));
}

/// The model with `from` replaced by `to`, and what the compiler says of it.
fn compile_error_after(name: &str, from: &str, to: &str) -> String {
    let model = copy_of_the_model(name);
    let text = std::fs::read_to_string(&model).unwrap();
    assert!(text.contains(from), "the model writes {from}");
    std::fs::write(&model, text.replace(from, to)).unwrap();
    Compiler::new(CompilerConfig::default()).compile_file(&model).err().map(|e| e.to_string()).unwrap_or_default()
}

#[test]
fn an_executed_case_cannot_check_a_requirement_it_does_not_verify() {
    // As for a simulated case: the expectation would count for no requirement.
    let error = compile_error_after("stray", "\"SYS-SPD-002: at(100 ms,", "\"SYS-SPD-003: at(100 ms,");
    assert!(error.contains("test_case 'TC-SPD-001': an expectation checks 'SYS-SPD-003', which the case does not verify (verifies: [SYS-SPD-001, SYS-SPD-002])"), "{error}");
}

#[test]
fn an_executed_case_checks_every_requirement_it_verifies() {
    // Every expectation names the filter: nothing of the case shows the warning.
    let error = compile_error_after("unchecked", "\"SYS-SPD-002: at(", "\"SYS-SPD-001: at(");
    assert!(error.contains("test_case 'TC-SPD-001' verifies 'SYS-SPD-002', but none of its expectations checks it"), "{error}");
}
