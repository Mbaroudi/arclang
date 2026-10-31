//! Monte Carlo verification: a case simulated over samples of uncertain
//! parameters, judged on how likely its expectations hold.
//!
//! The last test runs OpenModelica for real (Docker or ARCLANG_OMC); it is
//! ignored by default: `cargo test --release --test monte_carlo -- --ignored --nocapture`.

use arclang::compiler::modelica;
use arclang::compiler::simulation::{self, cache, montecarlo, run, tool::ToolConfig, Outcome, PointOutcome};
use arclang::compiler::{production_gate, Compiler, CompilerConfig};
use std::path::{Path, PathBuf};

const DEMO: &str = "examples/automotive/ev_range_simulation.arc";
const PHYSICS: &str = "examples/automotive/physics/EVPhysics.mo";
const CASE: &str = "simulate: { model: \"EVPhysics.TouringRange\" configuration: \"CFG-TOURING\" stop_time: 5 h sources: [\"physics/EVPhysics.mo\"] }";

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// The EV demo in a fresh directory, its long range case sampled.
fn demo(sampling: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("physics")).unwrap();
    std::fs::copy(root().join(PHYSICS), dir.path().join("physics/EVPhysics.mo")).unwrap();
    let text = std::fs::read_to_string(root().join(DEMO)).unwrap();
    assert!(text.contains(CASE));
    let sampled = CASE.replace(" }", &format!(" {sampling} }}"));
    let model = dir.path().join("ev_range_simulation.arc");
    std::fs::write(&model, text.replacen(CASE, &sampled, 1)).unwrap();
    (dir, model)
}

const SAMPLING: &str = "uncertain: [\"vehicleBody.ambient ~ uniform(253.15 K, 293.15 K)\", \"\\\"PN-VCU\\\".speedController.target_speed ~ triangular(90 km/h, 100 km/h, 110 km/h)\"] samples: 30 seed: 7 probability: 0.9";

fn compile(path: &Path) -> Result<arclang::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_file(path).map_err(|e| e.to_string())
}

fn case(body: &str) -> Result<arclang::CompilationResult, String> {
    Compiler::new(CompilerConfig::default())
        .compile_string(&format!(
            "model \"M\" {{}}\nsystem_analysis \"S\" {{ requirement \"R\" {{ id: \"REQ-1\" }} }}\n\
             test_case \"T\" {{ id: \"TC-1\" verifies: [\"REQ-1\"] method: \"analysis\" {body} expects: [\"final(a.b) >= 2 km\"] }}"
        ))
        .map_err(|e| e.to_string())
}

#[test]
fn a_monte_carlo_case_is_checked_when_parsed() {
    let ok = "simulate: { model: \"P.M\" stop_time: 1 h uncertain: [\"body.mass ~ normal(1850 kg, 40 kg)\", \"\\\"PN-BATT\\\".capacity ~ uniform(58 kWh, 62 kWh)\"] samples: 300 seed: 7 probability: 0.99 confidence: 0.9 }";
    assert!(case(ok).is_ok(), "{:?}", case(ok).err());
    for (body, told) in [
        ("simulate: { model: \"P.M\" stop_time: 1 h uncertain: [\"body.mass ~ normal(1850 kg, 40 kg)\"] }", "needs probability"),
        ("simulate: { model: \"P.M\" stop_time: 1 h probability: 0.99 }", "probability needs uncertain"),
        ("simulate: { model: \"P.M\" stop_time: 1 h samples: 50 }", "samples needs uncertain"),
        ("simulate: { model: \"P.M\" stop_time: 1 h uncertain: [\"a.b ~ uniform(0, 1)\"] probability: 1.5 }", "probability is a number between 0 and 1"),
        ("simulate: { model: \"P.M\" stop_time: 1 h uncertain: [\"a.b ~ uniform(0, 1)\"] probability: 0.9 confidence: 0.3 }", "confidence is a number between 0.5 and 1"),
        ("simulate: { model: \"P.M\" stop_time: 1 h uncertain: [\"a.b ~ uniform(0, 1)\"] probability: 0.9 samples: 5000 }", "samples is a whole number from 10 to 1000"),
        ("simulate: { model: \"P.M\" stop_time: 1 h uncertain: [\"a.b ~ uniform(0, 1)\"] probability: 0.9 sweep: [\"a.c = 1, 2\"] }", "either sweeps or samples"),
        ("simulate: { model: \"P.M\" stop_time: 1 h uncertain: [\"a.b ~ gamma(1, 2)\"] probability: 0.9 }", "not a law"),
        ("simulate: { model: \"P.M\" stop_time: 1 h uncertain: [\"a.b ~ normal(1 kg, 2 g)\"] probability: 0.9 }", "mixes units"),
    ] {
        let error = case(body).err().unwrap_or_else(|| panic!("`{body}` compiles"));
        assert!(error.contains(told), "`{body}`: {error}");
    }
}

/// A cached outcome of the sampled case: `held` of `total` samples hold.
fn cache_samples(model: &Path, held: usize, total: usize) {
    let result = compile(model).unwrap();
    let export = modelica::export(&result.ast).unwrap();
    let case = result.ast.test_cases.iter().find(|t| t.id == "TC-RANGE-LR").unwrap();
    let spec = simulation::spec_of(&case.attributes).unwrap().unwrap();
    let key = cache::key(&export.text, model.parent().unwrap(), &spec, &simulation::expects_of(&case.attributes));
    let points: Vec<PointOutcome> = (0..total)
        .map(|i| PointOutcome {
            label: format!("sample {}", i + 1),
            holds: i < held,
            values: vec![simulation::expects::Evaluated { expect: "min(\"PN-BATT\".soc) >= 0.1".into(), value: format!("min(batteryPack.soc) = {}", if i < held { "0.2" } else { "0.05" }), holds: i < held, margin: String::new(), requirement: None, slack: Some(if i < held { 0.1 } else { -0.05 }) }],
            ..Default::default()
        })
        .collect();
    let (verdict, reason, values) = simulation::conclusion_for(&points, spec.required);
    let outcome = Outcome { test_case: "TC-RANGE-LR".into(), verdict, reason, values, key, model: spec.model, tool: "test".into(), points, probability: spec.required, ..Default::default() };
    cache::write(&cache::dir(model), &outcome).unwrap();
}

#[test]
fn the_gate_takes_the_probabilistic_verdict_and_a_new_seed_makes_it_stale() {
    let (_dir, model) = demo(&SAMPLING.replace("samples: 30", "samples: 300").replace("probability: 0.9", "probability: 0.99"));
    // 295 of 300 hold: at most 99.6 %, shown only above 97.0 %: not 99 %.
    cache_samples(&model, 295, 300);
    let result = compile(&model).unwrap();
    let status = &result.semantic_model.simulations["TC-RANGE-LR"];
    assert_eq!(status.verdict, "fail");
    assert!(status.reason.contains("holds in 295 of 300 samples") && status.reason.contains("not shown to reach the 99 % required"), "{}", status.reason);
    let report = production_gate::run_gate(&result.ast, &result.semantic_model, "ISO26262");
    let blocker = report.findings.iter().find(|f| f.check == "requirements.verification").unwrap();
    assert!(blocker.message.contains("holds in 295 of 300 samples"), "{}", blocker.message);

    cache_samples(&model, 300, 300);
    let status = compile(&model).unwrap().semantic_model.simulations["TC-RANGE-LR"].clone();
    assert_eq!(status.verdict, "pass", "{}", status.reason);

    // Another seed draws other samples: the cached verdict no longer counts.
    std::fs::write(&model, std::fs::read_to_string(&model).unwrap().replace("seed: 7", "seed: 8")).unwrap();
    let result = compile(&model).unwrap();
    assert!(result.semantic_model.simulations["TC-RANGE-LR"].reason.starts_with("stale"));
}

#[test]
#[ignore = "runs OpenModelica (Docker or ARCLANG_OMC): cargo test --release --test monte_carlo -- --ignored --nocapture"]
fn openmodelica_runs_the_samples_and_finds_what_drives_the_margin() {
    let (dir, model) = demo(SAMPLING);
    let outcomes = run::verify(&compile(&model).unwrap(), &model, Some("TC-RANGE-LR"), &ToolConfig::from_env()).unwrap();
    let outcome = &outcomes[0];
    println!("{}: {}", outcome.verdict, outcome.reason);
    assert_eq!(outcome.points.len(), 30);
    assert!(outcome.points.iter().all(|p| p.error.is_empty()), "every override took: {:?}", outcome.points.iter().find(|p| !p.error.is_empty()));
    // The samples are those of the seed: each in its stratum, each proven applied.
    let ambients: Vec<f64> = outcome.points.iter().map(|p| p.inputs[0].1).collect();
    let mut strata: Vec<usize> = ambients.iter().map(|t| ((t - 253.15) / 40.0 * 30.0) as usize).collect();
    strata.sort();
    assert_eq!(strata, (0..30).collect::<Vec<_>>());
    assert!(outcome.reason.starts_with(&format!("holds in {} of 30 samples", outcome.points.iter().filter(|p| p.holds).count())), "{}", outcome.reason);
    // Faster is less charge left; warmer air, more.
    let rho = |parameter: &str| outcome.sensitivity.iter().find(|i| i.expect.starts_with("min(") && i.parameter.ends_with(parameter)).unwrap().rank_correlation;
    assert!(rho("target_speed") < -0.5, "{:?}", outcome.sensitivity);
    assert!(rho("ambient") > 0.0, "{:?}", outcome.sensitivity);
    // The samples, for R or Python.
    let table = std::fs::read_to_string(dir.path().join(".arclang/sim-results/TC-RANGE-LR/samples.csv")).unwrap();
    assert_eq!(table.lines().count(), 31);
    assert!(table.starts_with("sample,\"vehicleBody.ambient\",\"vehicleControlUnit.speedController.target_speed\","), "{table}");
    let _ = montecarlo::MAX_SAMPLES;
}

/// Samples where the margin falls with the first parameter and ignores the second.
fn synthetic_samples(dir: &Path) {
    let mut lines = vec!["sample,\"body.mass\",\"cell.noise\",\"margin: final(range) >= 300 km\",holds".to_string()];
    for i in 0..60 {
        let mass = 1800.0 + 5.0 * i as f64;
        let noise = ((i * 7919) % 17) as f64;
        // The mass sets the trend; the other terms shuffle the order, as real results do.
        let margin = 20_000.0 - 10.0 * (mass - 1800.0) + 400.0 * ((i * 31) % 7) as f64;
        lines.push(format!("{},{mass},{noise},{margin},{}", i + 1, u8::from(margin >= 0.0)));
    }
    std::fs::write(dir.join("samples.csv"), lines.join("\n") + "\n").unwrap();
}

#[test]
#[ignore = "runs R and Python (ARCLANG_RSCRIPT, ARCLANG_PYTHON, or Rscript and python3 with NumPy): cargo test --test monte_carlo -- --ignored"]
fn r_and_python_find_the_parameter_that_moves_the_margin_and_agree() {
    use simulation::statistics::{analyse, Engine};
    let dir = tempfile::tempdir().unwrap();
    synthetic_samples(dir.path());
    let prcc = |engine: Engine| -> Vec<(String, f64)> {
        let report = analyse(dir.path(), engine).unwrap_or_else(|e| panic!("{engine:?}: {e}"));
        assert!(dir.path().join("statistics.json").is_file());
        let margin = &report["margins"][0];
        assert_eq!(margin["samples"], 60);
        assert_eq!(margin["failure_fraction"].as_f64(), Some(0.0));
        margin["influences"].as_array().unwrap().iter().filter(|i| i["method"] == "PRCC").map(|i| (i["parameter"].as_str().unwrap().to_string(), i["value"].as_f64().unwrap())).collect()
    };
    let (r, python) = (prcc(Engine::R), prcc(Engine::Python));
    println!("R {r:?}\nPython {python:?}");
    for found in [&r, &python] {
        assert_eq!(found[0].0, "body.mass");
        assert!(found[0].1 < -0.6, "the mass drives the margin down: {found:?}");
        assert!(found[1].1.abs() < 0.2, "the noise does not: {found:?}");
    }
    for ((_, a), (_, b)) in r.iter().zip(&python) {
        assert!((a - b).abs() < 1e-3, "R {a} and Python {b} compute the same PRCC");
    }
}
