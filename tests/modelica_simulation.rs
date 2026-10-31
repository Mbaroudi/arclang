//! The OpenModelica bridge: `arclang export -f modelica`, `simulate:` and
//! `expects:` on test cases, the simulation cache, and the verdicts it
//! gives the production gate and the coverage table.
//!
//! The last test runs OpenModelica for real (Docker or ARCLANG_OMC); it is
//! ignored by default: `cargo test --release --test modelica_simulation -- --ignored`.

use arclang::compiler::modelica;
use arclang::compiler::simulation::{self, cache, run, tool::ToolConfig, Outcome};
use arclang::compiler::tables::{self, Options, TableSpec};
use arclang::compiler::{production_gate, Compiler, CompilerConfig};
use std::path::{Path, PathBuf};

const DEMO: &str = "examples/automotive/ev_range_simulation.arc";
const PHYSICS: &str = "examples/automotive/physics/EVPhysics.mo";

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn compile_string(source: &str) -> Result<arclang::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_string(source).map_err(|e| e.to_string())
}

/// The demo copied to a fresh directory: no cache there.
fn demo_copy() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("physics")).unwrap();
    std::fs::copy(root().join(DEMO), dir.path().join("ev_range_simulation.arc")).unwrap();
    std::fs::copy(root().join(PHYSICS), dir.path().join("physics/EVPhysics.mo")).unwrap();
    let model = dir.path().join("ev_range_simulation.arc");
    (dir, model)
}

fn compile_file(path: &Path) -> arclang::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_file(path).expect("the demo compiles")
}

// ---- Export ---------------------------------------------------------------

#[test]
fn the_demo_exports_to_the_golden_package() {
    let result = compile_file(&root().join(DEMO));
    let export = modelica::export(&result.ast).unwrap();
    assert_eq!(export.file_name, "EVRangeSimulation.mo");
    let golden = root().join("tests/fixtures/modelica/EVRangeSimulation.mo");
    if std::env::var_os("ARCLANG_BLESS").is_some() {
        std::fs::write(&golden, &export.text).unwrap();
    }
    let expected = std::fs::read_to_string(&golden).expect("golden package");
    assert_eq!(export.text, expected, "regenerate with ARCLANG_BLESS=1 when the change is intended");
    // Deterministic: the same model gives the same text.
    assert_eq!(modelica::export(&compile_file(&root().join(DEMO)).ast).unwrap().text, export.text);
    assert!(export.notes.is_empty(), "{:?}", export.notes);
    assert_eq!(export.instances["PN-BATT"], "batteryPack");
    assert_eq!(export.instances["BC-SPD"], "vehicleControlUnit.speedController");
    assert_eq!(export.systems["CFG-TOURING"], "EVRangeSimulation.Systems.Touring");
}

#[test]
fn ports_parameters_and_connections_become_modelica() {
    let result = compile_file(&root().join(DEMO));
    let text = modelica::export(&result.ast).unwrap().text;
    assert!(text.contains("Modelica.Electrical.Analog.Interfaces.PositivePin hv_p \"electrical port hv_p\" annotation(__ArcLang(id=\"PN-BATT.hv_p\"));"));
    assert!(text.contains("Modelica.Electrical.Analog.Interfaces.NegativePin hv_n"), "pole: negative, from the type");
    assert!(text.contains("Modelica.Mechanics.Rotational.Interfaces.Flange_b shaft"));
    assert!(text.contains("Modelica.Blocks.Interfaces.RealInput torque_cmd"));
    assert!(text.contains("parameter Modelica.Units.SI.Mass mass = 2100 \"2100 kg\";"));
    assert!(text.contains("parameter Real efficiency(unit=\"1\") = 0.9 \"90 %\";"));
    assert!(text.contains("parameter Real gear_ratio = 9;"));
    assert!(text.contains("parameter Modelica.Units.SI.Energy energy \"set by the variant of Battery each configuration selects\";"));
    assert!(text.contains("constrainedby EVRangeSimulation.Physical.BatteryPack(energy=2.304e8)"), "64 kWh");
    assert!(text.contains("constrainedby EVRangeSimulation.Physical.BatteryPack(energy=2.952e8)"), "82 kWh");
    assert!(text.contains("connect(batteryPack.hv_p, driveUnit.hv_p) annotation(__ArcLang(id=\"PL-HVP\"));"));
    assert!(text.contains("connect(speed, speedController.speed) annotation(__ArcLang(id=\"PL-SPEED-IN\"));"));
    assert!(text.contains("replaceable EVRangeSimulation.Behaviour.VehicleControlUnit.SpeedController speedController constrainedby"));
}

const BASE: &str = r#"
model "M" {}
logical_architecture "L" {
    component "Engine" { id: "LC-ENG" variation: true
        component "Small" { id: "LC-S" variant: true power: 50 kW }
        component "Big" { id: "LC-B" variant: true power: 90 kW }
    }
    component "Radio" { id: "LC-R" variation: true
        component "Basic radio" { id: "LC-R1" variant: true }
        component "Premium radio" { id: "LC-R2" variant: true }
    }
}
configuration "Eco" { id: "CFG-ECO" selects: ["LC-S", "LC-R1"] }
configuration "Sport" { id: "CFG-SPORT" selects: ["LC-B", "LC-R2"] }
"#;

#[test]
fn configurations_keep_their_variants_and_leave_the_others_out() {
    let source = format!(
        "{BASE}\nphysical_architecture \"P\" {{\n\
         node \"Motor\" {{ id: \"PN-M\" deploys \"LC-ENG\" port \"shaft\" {{ physics: mechanical_rotational }} }}\n\
         node \"Premium amplifier\" {{ id: \"PN-AMP\" deploys \"LC-R2\" port \"supply\" {{ physics: electrical }} }}\n\
         node \"Wheels\" {{ id: \"PN-W\" port \"hub\" {{ physics: mechanical_rotational }} port \"bus\" {{ physics: electrical }} }}\n\
         physical_link \"Axle\" {{ from: \"PN-M.shaft\" to: \"PN-W.hub\" }}\n\
         physical_link \"Amp supply\" {{ from: \"PN-AMP.supply\" to: \"PN-W.bus\" }}\n}}\n"
    );
    let result = compile_string(&source).unwrap();
    let text = modelica::export(&result.ast).unwrap().text;
    let eco = &text[text.find("partial model Eco").unwrap()..text.find("end Eco;").unwrap()];
    let sport = &text[text.find("partial model Sport").unwrap()..text.find("end Sport;").unwrap()];
    assert!(eco.contains("M.Physical.Motor(power=50000)") && sport.contains("M.Physical.Motor(power=90000)"), "{text}");
    assert!(!eco.contains("premiumAmplifier") && !eco.contains("Amp supply"), "a variant not selected is out, its links too");
    assert!(sport.contains("premiumAmplifier") && sport.contains("connect(premiumAmplifier.supply, wheels.bus)"));
}

#[test]
fn ports_without_physics_are_reported_not_guessed_and_fluid_is_deferred() {
    let source = r#"
model "M" {}
physical_architecture "P" {
    node "Pump" { id: "PN-P" port "outlet" { physics: fluid } port "data" {} }
    node "Tank" { id: "PN-T" port "inlet" { physics: fluid } }
    physical_link "Pipe" { from: "PN-P.outlet" to: "PN-T.inlet" }
}
"#;
    let result = compile_string(source).unwrap();
    let export = modelica::export(&result.ast).unwrap();
    assert!(export.notes.iter().any(|n| n.contains("'PN-P.data' says no physics: not exported")), "{:?}", export.notes);
    assert!(export.notes.iter().any(|n| n.contains("'PN-P.outlet'") && n.contains("fluid ports are deferred")), "{:?}", export.notes);
    assert!(export.notes.iter().any(|n| n.contains("'Pipe' joins a port that is not exported")), "{:?}", export.notes);
    assert!(!export.text.contains("outlet"));
}

#[test]
fn incompatible_ends_are_a_compile_error() {
    let source = r#"
model "M" {}
physical_architecture "P" {
    node "A" { id: "PN-A" port "p" { physics: electrical } }
    node "B" { id: "PN-B" port "f" { physics: mechanical_rotational } }
    physical_link "Wrong" { from: "PN-A.p" to: "PN-B.f" }
}
"#;
    let error = compile_string(source).unwrap_err();
    assert!(error.contains("'Wrong': 'PN-A.p' is electrical and 'PN-B.f' is mechanical_rotational"), "{error}");
    let signals = r#"
model "M" {}
physical_architecture "P" {
    node "A" { id: "PN-A" port in "x" { physics: signal } }
    node "B" { id: "PN-B" port in "y" { physics: signal } }
    physical_exchange "Two inputs" { from: "PN-A.x" to: "PN-B.y" }
}
"#;
    let error = compile_string(signals).unwrap_err();
    assert!(error.contains("both signal inputs"), "{error}");
}

#[test]
fn a_model_without_physical_nodes_does_not_export() {
    let result = compile_string("model \"M\" {}\nlogical_architecture \"L\" { component \"C\" { id: \"LC-1\" } }").unwrap();
    assert!(modelica::export(&result.ast).unwrap_err().contains("no physical node"));
}

// ---- The language ---------------------------------------------------------

fn case(body: &str) -> Result<arclang::CompilationResult, String> {
    compile_string(&format!(
        "model \"M\" {{}}\nsystem_analysis \"S\" {{ requirement \"R\" {{ id: \"REQ-1\" }} }}\n\
         configuration \"C\" {{ id: \"CFG-1\" selects: [] }}\n\
         test_case \"T\" {{ id: \"TC-1\" verifies: [\"REQ-1\"] method: \"analysis\" {body} }}"
    ))
}

#[test]
fn simulate_blocks_are_checked_when_parsed() {
    let ok = "simulate: { model: \"P.M\" stop_time: 1 h configuration: \"CFG-1\" sources: [\"p.mo\"] } expects: [\"final(a.b) >= 2 km\"]";
    assert!(case(ok).is_ok(), "{:?}", case(ok).err());
    let errors = [
        ("simulate: { stop_time: 1 h } expects: [\"final(x) > 1\"]", "model is required"),
        ("simulate: { model: \"P.M\" } expects: [\"final(x) > 1\"]", "stop_time is required"),
        ("simulate: { model: \"P.M\" stop_time: 3600 } expects: [\"final(x) > 1\"]", "stop_time has no unit"),
        ("simulate: { model: \"P.M\" stop_time: 1 km } expects: [\"final(x) > 1\"]", "not a time"),
        ("simulate: { model: \"P.M; system(\\\"x\\\")\" stop_time: 1 s } expects: [\"final(x) > 1\"]", "is not a Modelica class name"),
        ("simulate: { model: \"P.M\" stop_time: 1 s speed: 2 } expects: [\"final(x) > 1\"]", "'speed' is not one of"),
        ("simulate: { model: \"P.M\" stop_time: 1 s backend: \"matlab\" } expects: [\"final(x) > 1\"]", "backend 'matlab' is not one of"),
        ("simulate: { model: \"P.M\" stop_time: 1 s }", "needs expects"),
        ("expects: [\"final(x) > 1\"]", "expects: needs simulate"),
        ("simulate: { model: \"P.M\" stop_time: 1 s } expects: [\"x > 1\"]", "time series"),
        ("simulate: { model: \"P.M\" stop_time: 1 s } expects: [\"final(x) > 1\"] verdict: \"pass\"", "verdict comes from the simulation"),
        ("simulate: \"P.M\" expects: [\"final(x) > 1\"]", "simulate is a block"),
        ("simulate: { model: \"P.M\" stop_time: 1 s configuration: \"CFG-9\" } expects: [\"final(x) > 1\"]", "configuration 'CFG-9', which is none"),
        ("simulate: { model: \"P.M\" stop_time: 1 s sweep: [\"body.mass 1800 kg\"] } expects: [\"final(x) > 1\"]", "parameter = values"),
        ("simulate: { model: \"P.M\" stop_time: 1 s sweep: [\"a.b = 1 .. 300 step 1\"] } expects: [\"final(x) > 1\"]", "more than 200"),
        ("simulate: { model: \"P.M\" stop_time: 1 s sweep: 3 } expects: [\"final(x) > 1\"]", "sweep is a list"),
    ];
    let swept = "simulate: { model: \"P.M\" stop_time: 1 h sweep: [\"body.mass = 1800 kg, 2100 kg\", \"\\\"PN-VCU\\\".pi.k = 100 .. 300 step 100\"] } expects: [\"final(a.b) >= 2 km\"]";
    assert!(case(swept).is_ok(), "a sweep, an ArcLang id in quotes: {:?}", case(swept).err());
    for (body, expected) in errors {
        let error = case(body).err().unwrap_or_else(|| panic!("`{body}` compiles"));
        assert!(error.contains(expected), "`{body}`: {error}");
    }
}

#[test]
fn a_simulated_case_compiled_from_text_is_inconclusive_and_blocks_the_gate() {
    let result = case("simulate: { model: \"P.M\" stop_time: 1 h } expects: [\"final(a.b) >= 2 km\"]").unwrap();
    let report = production_gate::run_gate(&result.ast, &result.semantic_model, "ISO26262");
    let finding = report.findings.iter().find(|f| f.check == "requirements.verification").expect("a verification blocker");
    assert!(finding.message.contains("verification cases are inconclusive (T: no simulation verdict"), "{}", finding.message);
    assert_eq!(report.requirements_verified, 0);
}

#[test]
fn backends_other_than_openmodelica_are_declared_and_inconclusive() {
    let (_dir, model) = demo_copy();
    let text = std::fs::read_to_string(&model).unwrap().replace("simulate: { model: \"EVPhysics.TouringRange\"", "simulate: { backend: \"fmu\" model: \"EVPhysics.TouringRange\"");
    std::fs::write(&model, text).unwrap();
    let result = compile_file(&model);
    let config = ToolConfig { omc: Some("/nonexistent/arclang/omc".into()), ..ToolConfig::default() };
    let outcomes = run::verify(&result, &model, Some("TC-RANGE-LR"), &config).unwrap();
    assert_eq!(outcomes[0].verdict, "inconclusive");
    assert!(outcomes[0].reason.contains("backend 'fmu' is declared but not implemented yet"), "{}", outcomes[0].reason);
}

// ---- Cache and verdicts ---------------------------------------------------

fn cached(model: &Path, test: &str, verdict: &str, values: Vec<simulation::expects::Evaluated>, reason: &str) {
    let result = compile_file(model);
    let export = modelica::export(&result.ast).unwrap();
    let case = result.ast.test_cases.iter().find(|t| t.id == test).unwrap();
    let spec = simulation::spec_of(&case.attributes).unwrap().unwrap();
    let key = cache::key(&export.text, model.parent().unwrap(), &spec, &simulation::expects_of(&case.attributes));
    let outcome = Outcome {
        test_case: test.to_string(),
        verdict: verdict.to_string(),
        reason: reason.to_string(),
        values,
        key,
        model: spec.model,
        configuration: spec.configuration,
        tool: "test".into(),
        omc_version: "OpenModelica test".into(),
        ..Default::default()
    };
    cache::write(&cache::dir(model), &outcome).unwrap();
}

fn value(text: &str, holds: bool) -> simulation::expects::Evaluated {
    simulation::expects::Evaluated { expect: "e".into(), value: text.into(), holds, margin: String::new(), requirement: None }
}

#[test]
fn the_gate_reads_cached_verdicts_and_says_when_they_are_missing_or_stale() {
    let (_dir, model) = demo_copy();
    // Never simulated: inconclusive, a blocker, a warning of `check`.
    let result = compile_file(&model);
    assert!(result.warnings.iter().any(|w| w.contains("'Winter range, long range pack' is inconclusive: never simulated")), "{:?}", result.warnings);
    let report = production_gate::run_gate(&result.ast, &result.semantic_model, "ISO26262");
    assert!(report.findings.iter().any(|f| f.message.contains("verification cases are inconclusive") && f.message.contains("never simulated")));

    cached(&model, "TC-RANGE-LR", "pass", vec![value("final(vehicleBody.distance) = 319.9 km", true), value("min(batteryPack.soc) = 0.2403", true)], "");
    cached(&model, "TC-RANGE-STD", "fail", vec![value("min(batteryPack.soc) = 0.02664", false)], "min(batteryPack.soc) = 0.02664 (expected min(\"PN-BATT\".soc) >= 0.1)");
    cached(&model, "TC-RANGE-ENVELOPE", "pass", vec![value("min(batteryPack.soc) = 0.2016", true)], "");
    let result = compile_file(&model);
    let report = production_gate::run_gate(&result.ast, &result.semantic_model, "ISO26262");
    let blocker = report.findings.iter().find(|f| f.check == "requirements.verification").unwrap();
    assert!(blocker.message.contains("verification cases fail (Winter range, standard pack: min(batteryPack.soc) = 0.02664"), "{}", blocker.message);
    assert!(!report.findings.iter().any(|f| f.message.contains("inconclusive")));

    // The coverage table shows the verdict and the key values.
    let table = tables::compute(&result, &TableSpec { kind: "requirements_coverage".into(), ..TableSpec::default() }, &Options::default()).unwrap();
    let json = serde_json::to_string(&table).unwrap();
    assert!(json.contains("Key values") && json.contains("TC-RANGE-LR: final(vehicleBody.distance) = 319.9 km; min(batteryPack.soc) = 0.2403"), "{json}");
    assert!(json.contains("TC-RANGE-STD: min(batteryPack.soc) = 0.02664 (fails)"), "{json}");

    // All pass: both requirements (the range, its envelope) are verified, the gate has no blocker on them.
    cached(&model, "TC-RANGE-STD", "pass", vec![value("min(batteryPack.soc) = 0.3", true)], "");
    let result = compile_file(&model);
    let report = production_gate::run_gate(&result.ast, &result.semantic_model, "ISO26262");
    assert_eq!(report.requirements_verified, 2);

    // A source changed: stale, inconclusive again.
    let physics = model.parent().unwrap().join("physics/EVPhysics.mo");
    let text = std::fs::read_to_string(&physics).unwrap();
    std::fs::write(&physics, text.replace("tauMax = 350", "tauMax = 300")).unwrap();
    let result = compile_file(&model);
    assert!(result.warnings.iter().any(|w| w.contains("is inconclusive: stale")), "{:?}", result.warnings);
}

#[test]
fn without_openmodelica_the_verdict_is_inconclusive_and_not_cached() {
    let (_dir, model) = demo_copy();
    let result = compile_file(&model);
    let config = ToolConfig { omc: Some("/nonexistent/arclang/omc".into()), ..ToolConfig::default() };
    let outcomes = run::verify(&result, &model, Some("TC-RANGE-LR"), &config).unwrap();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].verdict, "inconclusive");
    assert!(outcomes[0].reason.starts_with("OpenModelica not available"), "{}", outcomes[0].reason);
    assert!(!cache::dir(&model).join("TC-RANGE-LR.json").exists(), "an inconclusive run is not cached");
    assert!(run::verify(&result, &model, Some("TC-NONE"), &config).unwrap_err().contains("no test_case 'TC-NONE'"));
}

// ---- Expectations on a fixture result --------------------------------------

#[test]
fn expectations_are_evaluated_on_a_result_file() {
    use simulation::expects::{evaluate, parse, Context};
    use simulation::results::{units_from_init_xml, Series};
    let fixtures = root().join("tests/fixtures/modelica");
    let mut series = Series::from_csv(&std::fs::read_to_string(fixtures.join("range_res.csv")).unwrap()).unwrap();
    series.set_units(units_from_init_xml(&std::fs::read_to_string(fixtures.join("range_init.xml")).unwrap()));
    let aliases = [("PN-BATT".to_string(), "batteryPack".to_string())].into_iter().collect();
    let eval = |text: &str| {
        let mut notes = Vec::new();
        evaluate(&parse(text, &aliases).unwrap(), &mut Context { series: &series, notes: &mut notes })
    };
    let distance = eval("final(vehicleBody.distance) >= 300 km").unwrap();
    assert!(distance.holds);
    assert_eq!(distance.value, "final(vehicleBody.distance) = 319.9 km");
    let soc = eval("min(\"PN-BATT\".soc) >= 0.1").unwrap();
    assert!(!soc.holds, "{soc:?}");
    assert!(eval("at(1 h, vehicleBody.car.v) > 50 km/h").unwrap().holds);
    assert!(eval("mean(driveUnit.electrical) < 20 kW").unwrap().holds);
    assert!(eval("max(vehicleBody.distance) > 2 h").unwrap_err().contains("length"));
}

// ---- OpenModelica, for real -----------------------------------------------

#[test]
#[ignore = "runs OpenModelica (Docker or ARCLANG_OMC): cargo test --release --test modelica_simulation -- --ignored --nocapture"]
fn openmodelica_checks_the_package_and_verifies_the_demo() {
    use simulation::tool;
    let (dir, model) = demo_copy();
    let result = compile_file(&model);
    let export = modelica::export(&result.ast).unwrap();
    let config = ToolConfig::from_env();
    let omc = tool::locate(&config).expect("OpenModelica is available");

    // Every generated class loads and checks.
    let work = dir.path().join("check");
    std::fs::create_dir_all(&work).unwrap();
    std::fs::write(work.join(&export.file_name), &export.text).unwrap();
    let mut script = format!(
        "if not loadModel(Modelica, {{\"4.1.0\"}}) then installPackage(Modelica, \"4.1.0+maint.om\", exactMatch=true); end if;\nloadModel(Modelica, {{\"4.1.0\"}});\nloadFile(\"{}\"); getErrorString();\n",
        export.file_name
    );
    let paths = [
        "EVRangeSimulation.Behaviour.VehicleControlUnit.SpeedController",
        "EVRangeSimulation.Physical.BatteryPack",
        "EVRangeSimulation.Physical.DriveUnit",
        "EVRangeSimulation.Physical.VehicleBody",
        "EVRangeSimulation.Physical.VehicleControlUnit",
        "EVRangeSimulation.Systems.City",
        "EVRangeSimulation.Systems.Touring",
    ];
    let partials = export.text.lines().filter(|l| l.trim().starts_with("partial model ")).count();
    assert_eq!(partials, paths.len(), "every generated model is checked");
    for path in paths {
        script.push_str(&format!("print(\"@@check {path}\\n\" + checkModel({path}) + \"\\n\" + getErrorString() + \"\\n\");\n"));
    }
    std::fs::write(work.join("check.mos"), script).unwrap();
    let run = tool::run(&omc, &work, "check.mos", None).unwrap();
    println!("{}", run.output);
    for path in paths {
        let after = run.output.split(&format!("@@check {path}\n")).nth(1).expect("checked");
        assert!(after.contains(&format!("Check of {path} completed successfully.")), "{path}: {after}");
    }

    // The demo: the long range pack passes, the standard pack fails.
    let outcomes = run::verify(&result, &model, None, &config).unwrap();
    for outcome in &outcomes {
        println!("{} {}: {} {}", outcome.test_case, outcome.verdict, outcome.evidence(), outcome.reason);
    }
    let verdict = |id: &str| outcomes.iter().find(|o| o.test_case == id).unwrap().verdict.clone();
    assert_eq!(verdict("TC-RANGE-LR"), "pass");
    assert_eq!(verdict("TC-RANGE-STD"), "fail");
    // What the log says of a failure: how far, how long, what OpenModelica printed.
    let std = outcomes.iter().find(|o| o.test_case == "TC-RANGE-STD").unwrap();
    let soc = std.values.iter().find(|v| !v.holds).unwrap();
    assert!(soc.margin.starts_with("short by 0.013"), "{soc:?}");
    // The run ends when the 300 km are driven: the distance sits on its bound.
    assert!(std.values.iter().any(|v| v.holds && v.margin == "at the bound"), "{:?}", std.values);
    assert!(std.seconds > 0.0 && std.log.contains("@@simulate"));
    // The gate now reads them from the cache.
    let result = compile_file(&model);
    assert_eq!(result.semantic_model.simulations["TC-RANGE-LR"].verdict, "pass");
    assert_eq!(result.semantic_model.simulations["TC-RANGE-STD"].verdict, "fail");
}

#[test]
#[ignore = "runs OpenModelica (Docker or ARCLANG_OMC): cargo test --release --test modelica_simulation -- --ignored --nocapture"]
fn openmodelica_runs_a_sweep_and_proves_each_override() {
    let (_dir, model) = demo_copy();
    let text = std::fs::read_to_string(&model).unwrap();
    // The long range pack, over three temperatures and two speeds; then a name that is not a parameter.
    let sweep = "sources: [\"physics/EVPhysics.mo\"] sweep: [\"vehicleBody.ambient = 253.15 K, 263.15 K, 293.15 K\", \"\\\"PN-VCU\\\".speedController.target_speed = 90 km/h .. 110 km/h step 20 km/h\"] }";
    let marker = "configuration: \"CFG-TOURING\" stop_time: 5 h sources: [\"physics/EVPhysics.mo\"] }";
    assert!(text.contains(marker));
    std::fs::write(&model, text.replacen(marker, &format!("configuration: \"CFG-TOURING\" stop_time: 5 h {sweep}"), 1)).unwrap();
    let config = ToolConfig::from_env();
    let outcomes = run::verify(&compile_file(&model), &model, Some("TC-RANGE-LR"), &config).unwrap();
    let lr = &outcomes[0];
    for point in &lr.points {
        println!("{} {}: {}", if point.holds { "ok  " } else { "FAIL" }, point.label, point.values.iter().map(|v| format!("{} ({})", v.value, v.margin)).collect::<Vec<_>>().join("; "));
    }
    assert_eq!(lr.points.len(), 6);
    assert!(lr.points.iter().all(|p| p.error.is_empty()), "every override took: {:?}", lr.points);
    assert_eq!(lr.points[0].label, "ambient = 253.15 K, target_speed = 90 km/h");
    // Colder air is denser: less charge left, at the same speed.
    let soc = |i: usize| lr.points[i].values.iter().find(|v| v.value.starts_with("min(")).unwrap().value.rsplit(' ').next().unwrap().parse::<f64>().unwrap();
    assert!(soc(0) < soc(4), "253 K {} against 293 K {}", soc(0), soc(4));

    let text = std::fs::read_to_string(&model).unwrap().replacen("vehicleBody.ambient = ", "vehicleBody.nothing = ", 1);
    std::fs::write(&model, text).unwrap();
    let outcomes = run::verify(&compile_file(&model), &model, Some("TC-RANGE-LR"), &config).unwrap();
    assert_eq!(outcomes[0].verdict, "error");
    assert!(outcomes[0].reason.contains("'vehicleBody.nothing' is not a variable"), "{}", outcomes[0].reason);
}

// ---- Diagnosis of the tools (`verify --check-tools`) ----------------------

fn check_tools(args: &[&str], envs: &[(&str, &str)]) -> std::process::Output {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_arclang"));
    command.arg("verify").arg("--check-tools").args(args).env_remove("ARCLANG_OMC").env_remove("ARCLANG_OMC_DOCKER");
    for (k, v) in envs {
        command.env(k, v);
    }
    command.output().expect("arclang runs")
}

#[test]
fn check_tools_needs_no_model_and_reports_json() {
    let out = check_tools(&["--json"], &[("ARCLANG_OMC", "/nonexistent/arclang/omc"), ("ARCLANG_OMC_DOCKER", "example/om:1")]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json on stdout");
    assert!(json["tool"].is_null());
    assert_eq!(json["image"], "example/om:1");
    assert_eq!(json["library_volume"], simulation::tool::LIBRARY_VOLUME);
    assert!(json["detail"].as_str().unwrap().starts_with("OpenModelica not available"));
    assert!(json["docker_installed"].is_boolean() && json["local_omc"] == false);
}

#[test]
fn check_tools_prints_a_summary_without_json() {
    let out = check_tools(&[], &[("ARCLANG_OMC", "/nonexistent/arclang/omc")]);
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("OpenModelica not available") && text.contains("Docker installed"), "{text}");
}

#[test]
fn verify_names_each_case_its_requirements_and_configuration_even_without_a_tool() {
    let (_dir, model) = demo_copy();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_arclang"))
        .args(["verify", "--json"])
        .arg(&model)
        .env("ARCLANG_OMC", "/nonexistent/arclang/omc")
        .env_remove("ARCLANG_OMC_DOCKER")
        .output()
        .unwrap();
    assert!(!out.status.success(), "inconclusive is not a pass");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json on stdout");
    let std = json.as_array().unwrap().iter().find(|o| o["test_case"] == "TC-RANGE-STD").expect("the standard case");
    assert_eq!(std["verdict"], "inconclusive");
    assert_eq!(std["name"], "Winter range, standard pack");
    assert_eq!(std["verifies"], serde_json::json!(["REQ-RANGE"]));
    assert_eq!(std["configuration"], "CFG-CITY");
    assert_eq!(std["configuration_name"], "City");
    assert_eq!(std["seconds"], 0.0);
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(!text.contains("Compilation error"), "a case that does not pass is not a compilation error: {text}");
}

#[test]
fn verify_without_a_model_nor_check_tools_is_a_usage_error() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_arclang")).arg("verify").output().unwrap();
    assert!(!out.status.success());
}

// ---- Requirements of a product line, expectations by requirement ----------

fn edit(model: &Path, edits: &[(&str, &str)]) {
    let mut text = std::fs::read_to_string(model).unwrap();
    for (from, to) in edits {
        assert!(text.contains(from), "the demo holds {from:?}");
        text = text.replacen(from, to, 1);
    }
    std::fs::write(model, text).unwrap();
}

fn compile_err(model: &Path) -> String {
    Compiler::new(CompilerConfig::default()).compile_file(model).err().map(|e| e.to_string()).unwrap_or_default()
}

const RANGE_ID: &str = "id: \"REQ-RANGE\"\n";
const CITY_REQUIREMENT: &str = "    requirement \"City range\" {\n        id: \"REQ-CITY\"\n        applies_to: [\"City\"]\n        description: \"At -10 °C, the city car drives 200 km\"\n    }\n";

#[test]
fn a_requirement_applies_to_the_configurations_it_names() {
    let (_dir, model) = demo_copy();
    // Only the touring car must drive 300 km: the city case cannot verify it.
    edit(&model, &[(RANGE_ID, "id: \"REQ-RANGE\"\n        applies_to: [\"CFG-TOURING\"]\n")]);
    let error = compile_err(&model);
    assert!(error.contains("test_case 'Winter range, standard pack' verifies 'REQ-RANGE' in configuration City, but 'REQ-RANGE' applies only to CFG-TOURING"), "{error}");
    // The city car has its own requirement.
    edit(&model, &[
        ("system_analysis \"Vehicle needs\" {\n", &format!("system_analysis \"Vehicle needs\" {{\n{CITY_REQUIREMENT}")),
        ("    verifies: [\"REQ-RANGE\"]\n    method: \"analysis\"\n    simulate: { model: \"EVPhysics.CityRange\"", "    verifies: [\"REQ-CITY\"]\n    method: \"analysis\"\n    simulate: { model: \"EVPhysics.CityRange\""),
        ("trace \"PN-BATT\" satisfies \"REQ-RANGE\"\n", "trace \"PN-BATT\" satisfies \"REQ-RANGE\"\ntrace \"PN-BATT\" satisfies \"REQ-CITY\"\n"),
    ]);
    assert_eq!(compile_err(&model), "");
    cached(&model, "TC-RANGE-LR", "pass", vec![value("min(batteryPack.soc) = 0.2874", true)], "");
    cached(&model, "TC-RANGE-ENVELOPE", "pass", vec![value("min(batteryPack.soc) = 0.2016", true)], "");
    cached(&model, "TC-RANGE-STD", "fail", vec![value("min(batteryPack.soc) = 0.08693", false)], "min(batteryPack.soc) = 0.08693 (expected min(\"PN-BATT\".soc) >= 0.1)");
    let result = compile_file(&model);
    let report = production_gate::run_gate(&result.ast, &result.semantic_model, "ISO26262");
    let blockers: Vec<&str> = report.findings.iter().filter(|f| f.check == "requirements.verification").map(|f| f.message.as_str()).collect();
    // The city car's failure blocks its requirement, not the touring car's.
    assert_eq!(blockers.len(), 1, "{blockers:?}");
    assert!(blockers[0].starts_with("requirement 'REQ-CITY': verification cases fail"), "{blockers:?}");
    assert_eq!(report.requirements_verified, 2, "REQ-RANGE and its envelope");

    // A configuration named that is none, and a requirement left without a case in one of its configurations.
    edit(&model, &[("applies_to: [\"City\"]", "applies_to: [\"Town\"]")]);
    assert!(compile_err(&model).contains("requirement 'REQ-CITY' applies to 'Town', which is no configuration"), "{}", compile_err(&model));
    edit(&model, &[("applies_to: [\"Town\"]", "applies_to: [\"City\"]"), ("applies_to: [\"CFG-TOURING\"]", "applies_to: [\"CFG-TOURING\", \"City\"]")]);
    let result = compile_file(&model);
    let report = production_gate::run_gate(&result.ast, &result.semantic_model, "ISO26262");
    assert!(report.findings.iter().any(|f| f.message == "requirement 'REQ-RANGE' applies to configuration City (CFG-CITY), which no verification case runs in"), "{:?}", report.findings);
}

#[test]
fn an_expectation_that_names_its_requirement_judges_only_that_requirement() {
    let (_dir, model) = demo_copy();
    let trip = "    requirement \"Trip length\" {\n        id: \"REQ-TRIP\"\n        description: \"The vehicle drives 300 km\"\n    }\n";
    edit(&model, &[
        ("system_analysis \"Vehicle needs\" {\n", &format!("system_analysis \"Vehicle needs\" {{\n{trip}")),
        ("trace \"PN-BATT\" satisfies \"REQ-RANGE\"\n", "trace \"PN-BATT\" satisfies \"REQ-RANGE\"\ntrace \"PN-BATT\" satisfies \"REQ-TRIP\"\n"),
        (
            "    verifies: [\"REQ-RANGE\"]\n    method: \"analysis\"\n    simulate: { model: \"EVPhysics.CityRange\" configuration: \"CFG-CITY\" stop_time: 5 h sources: [\"physics/EVPhysics.mo\"] }\n    expects: [\"final(vehicleBody.distance) >= 300 km\", \"min(\\\"PN-BATT\\\".soc) >= 0.1\"]",
            "    verifies: [\"REQ-RANGE\", \"REQ-TRIP\"]\n    method: \"analysis\"\n    simulate: { model: \"EVPhysics.CityRange\" configuration: \"CFG-CITY\" stop_time: 5 h sources: [\"physics/EVPhysics.mo\"] }\n    expects: [\"Trip length: final(vehicleBody.distance) >= 300 km\", \"REQ-RANGE: min(\\\"PN-BATT\\\".soc) >= 0.1\"]",
        ),
    ]);
    assert_eq!(compile_err(&model), "");
    let tagged = |text: &str, holds: bool, requirement: &str| simulation::expects::Evaluated { expect: "e".into(), value: text.into(), holds, margin: String::new(), requirement: Some(requirement.into()) };
    cached(&model, "TC-RANGE-STD", "fail", vec![tagged("final(vehicleBody.distance) = 300 km", true, "Trip length"), tagged("min(batteryPack.soc) = 0.08693", false, "REQ-RANGE")], "min(batteryPack.soc) = 0.08693 (expected e)");
    cached(&model, "TC-RANGE-LR", "pass", vec![value("min(batteryPack.soc) = 0.2874", true)], "");
    cached(&model, "TC-RANGE-ENVELOPE", "pass", vec![value("min(batteryPack.soc) = 0.2016", true)], "");
    let result = compile_file(&model);
    let status = &result.semantic_model.simulations["TC-RANGE-STD"];
    assert_eq!(status.by_requirement["REQ-TRIP"].verdict, "pass");
    assert_eq!(status.by_requirement["REQ-RANGE"].verdict, "fail");
    let report = production_gate::run_gate(&result.ast, &result.semantic_model, "ISO26262");
    let blockers: Vec<&str> = report.findings.iter().filter(|f| f.check == "requirements.verification").map(|f| f.message.as_str()).collect();
    assert_eq!(blockers.len(), 1, "the trip is driven: only the reserve fails: {blockers:?}");
    assert!(blockers[0].contains("'REQ-RANGE'") && blockers[0].contains("min(batteryPack.soc) = 0.08693"), "{blockers:?}");

    // An expectation naming a requirement the case does not verify; a requirement no expectation checks.
    edit(&model, &[("\"Trip length: final", "\"REQ-RANGE-ENVELOPE: final")]);
    assert!(compile_err(&model).contains("an expectation checks 'REQ-RANGE-ENVELOPE', which the case does not verify"), "{}", compile_err(&model));
    edit(&model, &[("\"REQ-RANGE-ENVELOPE: final", "\"REQ-RANGE: final")]);
    assert!(compile_err(&model).contains("verifies 'REQ-TRIP', but none of its expectations checks it"), "{}", compile_err(&model));
}

#[test]
fn physics_units_reach_the_modelica_parameters() {
    // The demo writes `ambient: -10 °C` and `drag_area: 0.62 m²`.
    let (_dir, model) = demo_copy();
    let result = compile_file(&model);
    let export = modelica::export(&result.ast).unwrap();
    assert!(export.text.contains("parameter Modelica.Units.SI.Temperature ambient = 263.15 \"-10 °C\";"), "{}", export.text);
    assert!(export.text.contains("parameter Modelica.Units.SI.Area drag_area = 0.62 \"0.62 m²\";"));
    assert!(export.text.contains("parameter Real efficiency(unit=\"1\") = 0.9 \"90 %\";"), "no conversion noise");
}
