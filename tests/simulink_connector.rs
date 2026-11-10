//! The connector's own tests, run on ArcLang's port: each pivot example gives
//! the script the connector's generator gives (one of the orders its tests
//! accept), line by line, the empty lines of the expected file left out as
//! the connector's harness does.

use arclang::compiler::simulink_connector::{pivot, script};
use std::path::Path;

fn fixtures() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/simulink_connector"))
}

/// Why the generated lines differ from the expected file, or None when they are the same.
fn differs(generated: &[&str], expected: &str) -> Option<String> {
    let expected: Vec<&str> = expected.lines().filter(|l| !l.is_empty()).collect();
    for (i, line) in generated.iter().enumerate() {
        match expected.get(i) {
            None => return Some(format!("{} additional line(s)", generated.len() - expected.len())),
            Some(want) if want != line => return Some(format!("line {}: expecting '{want}' got '{line}'", i + 1)),
            _ => {}
        }
    }
    (generated.len() != expected.len()).then(|| format!("{} line(s) missing", expected.len() - generated.len()))
}

#[test]
fn the_generator_writes_the_connectors_scripts() {
    let dir = fixtures().join("generator");
    let mut names: Vec<String> = std::fs::read_dir(dir.join("expected")).unwrap().filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).filter(|n| n.ends_with(".m")).collect();
    names.sort();
    let mut failures = Vec::new();
    for name in &names {
        let model = dir.join("pivot").join(name.replace(".m", ".mlcore"));
        let text = match pivot::load(&model).and_then(|p| script(&p)) {
            Ok(text) => text,
            Err(message) => message,
        };
        let generated: Vec<&str> = text.split('\n').collect();
        let generated = if generated.last() == Some(&"") { &generated[..generated.len() - 1] } else { &generated[..] };
        let mut why = Vec::new();
        let mut matched = false;
        for suffix in ["", ".1", ".2", ".3"] {
            let Ok(expected) = std::fs::read_to_string(dir.join("expected").join(format!("{name}{suffix}"))) else { continue };
            match differs(generated, &expected) {
                None => {
                    matched = true;
                    break;
                }
                Some(d) => why.push(format!("{name}{suffix}: {d}")),
            }
        }
        if !matched {
            failures.push(why.join(" | "));
        }
    }
    assert!(failures.is_empty(), "{} of {} scripts differ:\n{}", failures.len(), names.len(), failures.join("\n"));
}

/// `PIVOT=X.mlcore cargo test --test simulink_connector show -- --ignored --nocapture`: one script.
#[test]
#[ignore]
fn show() {
    let name = std::env::var("PIVOT").unwrap_or_default();
    let p = pivot::load(&fixtures().join("generator/pivot").join(name)).unwrap();
    println!("{}", script(&p).unwrap_or_else(|e| e));
}

// ---- The model-to-model half ---------------------------------------------------------

use arclang::compiler::simulink_connector::m2m::{self, model::Model, Options, Platform, Strategy};
use std::collections::HashMap;

/// A configuration of the connector's m2m tests (`testN.properties`).
fn configuration(name: &str) -> HashMap<String, String> {
    configuration_in("m2m", name)
}

fn configuration_in(set: &str, name: &str) -> HashMap<String, String> {
    let text = std::fs::read_to_string(fixtures().join(set).join("configurations").join(format!("{name}.properties"))).unwrap();
    text.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect()
}

/// The script of a configuration: the Capella selection made a pivot, the pivot a script.
fn m2m_script(name: &str) -> String {
    m2m_script_in("m2m", name)
}

fn m2m_script_in(set: &str, name: &str) -> String {
    let props = configuration_in(set, name);
    let project = &props["project"];
    // The Linux set runs the connector on its own projects too (`m2m/pp`, `m2m/mystate`).
    let dir = [set, "m2m"].iter().map(|s| fixtures().join(s).join(project)).find(|d| d.is_dir()).unwrap();
    let model = Model::load(&dir.join(format!("{project}.capella"))).unwrap();
    let strategy = Strategy::parse(&props["strategy"]).unwrap();
    let mut options = Options::default();
    if let Some(v) = props.get("leaf.functions.export.option") {
        options.leaf_functions = v.clone();
    }
    if let Some(v) = props.get("port.naming.option") {
        options.port_naming = v.clone();
    }
    if set == "m2m_linux" {
        options.platform = Platform::LinuxGtk;
    }
    let ids: Vec<&str> = props["elements"].split_whitespace().collect();
    match m2m::transform(&model, strategy, &ids, &options).and_then(|p| script(&p)) {
        Ok(text) => text,
        Err(message) => message,
    }
}

/// Why the script differs from every expected one the connector accepts, or None.
fn differs_from_all(dir: &Path, name: &str, text: &str) -> Option<String> {
    let generated: Vec<&str> = text.split('\n').collect();
    let generated = if generated.last() == Some(&"") { &generated[..generated.len() - 1] } else { &generated[..] };
    let mut why = Vec::new();
    for suffix in ["", ".1", ".2", ".3"] {
        let Ok(expected) = std::fs::read_to_string(dir.join(format!("{name}{suffix}"))) else { continue };
        match differs(generated, &expected) {
            None => return None,
            Some(d) => why.push(format!("{name}{suffix}: {d}")),
        }
    }
    Some(why.join(" | "))
}

#[test]
fn the_m2m_writes_the_connectors_scripts() {
    let dir = fixtures().join("m2m/expected");
    let mut names: Vec<String> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).filter(|n| n.ends_with(".m")).collect();
    names.sort();
    assert_eq!(names.len(), 34, "the connector's 34 configurations");
    let mut failures = Vec::new();
    for name in &names {
        let text = m2m_script(name.trim_end_matches(".m"));
        if let Some(why) = differs_from_all(&dir, name, &text) {
            failures.push(why);
        }
    }
    assert!(failures.is_empty(), "{} of {} configurations differ:\n{}", failures.len(), names.len(), failures.join("\n"));
}

/// `CONFIG=test1 cargo test --test simulink_connector show_m2m -- --ignored --nocapture`.
#[test]
#[ignore]
fn show_m2m() {
    println!("{}", m2m_script(&std::env::var("CONFIG").unwrap_or_default()));
}

/// The real connector, run headless in Capella 7.1 on Linux GTK (`m2m_linux/README.md`),
/// on state machines drawn by ArcLang's export and by Capella itself: nested states, a
/// state with two regions and one with three, machines with three regions, transitions
/// with bends, a loop, a choice, long names; and on its own diagram tests.
///
/// Every line of every script is the connector's but the geometry (positions, clock
/// positions, ends) of the configurations in `GEOMETRY_GAPS`, whose figures the port
/// does not yet measure as that platform draws them: there, every other line is.
#[test]
fn the_m2m_matches_the_connector_run_on_linux() {
    /// What the port does not measure yet, by configuration:
    /// - msm1, msm2, msm3, msm7, msm8 (drawn by ArcLang): the image's bounds take in the
    ///   labels of pseudo states and the edges (their bends, lanes and labels), and a
    ///   state's regions start 1 px higher at each level.
    /// - msm4, msm5, msm6 (drawn by Capella): a mode without a region is sized 41 px
    ///   high around its label, regions stacked in a state are sized by Sirius.
    /// - test15, test18, test19: the ends of some transitions (anchors whose handle
    ///   bounds are not those of the other states of the same diagrams).
    /// - test28: a state holding a junction is 4 px narrower and 1 px shorter.
    const GEOMETRY_GAPS: &[&str] = &["msm1", "msm2", "msm3", "msm4", "msm5", "msm6", "msm7", "msm8", "test15", "test18", "test19", "test28"];
    let geometry = |line: &str| ["Position", "OClock", "EndPoint", "Midpoint"].iter().any(|k| line.contains(k));
    let dir = fixtures().join("m2m_linux/expected");
    let mut names: Vec<String> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).filter(|n| n.ends_with(".m")).collect();
    names.sort();
    assert_eq!(names.len(), 15, "{names:?}");
    let mut failures = Vec::new();
    for name in &names {
        let config = name.trim_end_matches(".m");
        let text = m2m_script_in("m2m_linux", config);
        let generated: Vec<&str> = text.split('\n').collect();
        let generated = if generated.last() == Some(&"") { &generated[..generated.len() - 1] } else { &generated[..] };
        let expected = std::fs::read_to_string(dir.join(name)).unwrap();
        let exact = differs(generated, &expected);
        if !GEOMETRY_GAPS.contains(&config) {
            if let Some(why) = exact {
                failures.push(format!("{name}: {why}"));
            }
            continue;
        }
        // A gap: the same lines but the geometry, and the gap still there.
        let expected: Vec<&str> = expected.lines().filter(|l| !l.is_empty()).collect();
        if generated.len() != expected.len() {
            failures.push(format!("{name}: {} lines, the connector's {}", generated.len(), expected.len()));
            continue;
        }
        if let Some((i, (g, e))) = generated.iter().zip(&expected).enumerate().find(|(_, (g, e))| g != e && !(geometry(g) && geometry(e))) {
            failures.push(format!("{name}: line {}: expecting '{e}' got '{g}'", i + 1));
        }
        if exact.is_none() {
            failures.push(format!("{name}: now the connector's exactly: take it out of GEOMETRY_GAPS"));
        }
    }
    assert!(failures.is_empty(), "{} of {} configurations:\n{}", failures.len(), names.len(), failures.join("\n"));
}

/// `CONFIG=msm1 cargo test --test simulink_connector show_m2m_linux -- --ignored --nocapture`.
#[test]
#[ignore]
fn show_m2m_linux() {
    println!("{}", m2m_script_in("m2m_linux", &std::env::var("CONFIG").unwrap_or_default()));
}

/// The pivot written as an `.mlcore` and read again gives the same script.
#[test]
fn the_written_pivot_reads_back_the_same() {
    use arclang::compiler::simulink_connector::mlcore;
    for n in 1..=34 {
        let name = format!("test{n}");
        let props = configuration(&name);
        let project = &props["project"];
        let model = Model::load(&fixtures().join("m2m").join(project).join(format!("{project}.capella"))).unwrap();
        let mut options = Options::default();
        if let Some(v) = props.get("leaf.functions.export.option") {
            options.leaf_functions = v.clone();
        }
        if let Some(v) = props.get("port.naming.option") {
            options.port_naming = v.clone();
        }
        let ids: Vec<&str> = props["elements"].split_whitespace().collect();
        let p = m2m::transform(&model, Strategy::parse(&props["strategy"]).unwrap(), &ids, &options).unwrap();
        let text = mlcore::write(&p);
        let again = pivot::read(&text, Path::new(".")).unwrap_or_else(|e| panic!("{name}: {e}\n{text}"));
        assert_eq!(script(&again), script(&p), "{name}");
        assert_eq!(mlcore::write(&again), text, "{name}: written twice the same");
    }
}

/// An ArcLang model goes through the Capella exporter, then the connector's m2m: its
/// functions become models, its state machines charts, laid out from the state machine
/// diagrams the export writes, as the connector lays out a model drawn in Capella.
#[test]
fn an_arclang_model_exports_through_capella() {
    use arclang::compiler::simulink_connector::export;
    let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/automotive/remote-start/remote_start_architecture.arc"));
    let result = arclang::Compiler::new(arclang::CompilerConfig::default()).compile_file(path).unwrap();
    let exported = arclang::compiler::capella_export::export(&result.ast, "remote_start");
    let model = Model::from_strings(&[("remote_start.capella", exported.capella.as_str()), ("remote_start.aird", exported.aird.as_str())]).unwrap();
    let options = Options::default();
    let functions = export::export(&model, Strategy::Functions, &[], &options).unwrap();
    assert!(functions.script.contains("new_system("), "{}", functions.script);
    let machines = export::export(&model, Strategy::StateMachines, &[], &options).unwrap();
    assert!(machines.script.contains("Stateflow.State(parent)") && machines.script.contains("transition.SourceOClock"), "{}", machines.script);
    assert!(machines.notes.iter().all(|n| !n.contains("ArcLang lays its chart out")), "{:?}", machines.notes);
    // ArcLang's own layout is not needed: the diagrams are there.
    let drawn = export::export(&model, Strategy::StateMachines, &[], &Options { draw_without_diagram: true, ..Options::default() }).unwrap();
    assert_eq!(drawn.script, machines.script);
    // What is written reads back to the same script.
    let again = pivot::read(&machines.pivot, Path::new(".")).unwrap();
    assert_eq!(script(&again).unwrap(), machines.script);
}

#[test]
fn a_text_with_a_quote_stays_one_matlab_text() {
    use arclang::compiler::simulink_connector::export;
    let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/automotive/remote-start/remote_start_architecture.arc"));
    let result = arclang::Compiler::new(arclang::CompilerConfig::default()).compile_file(path).unwrap();
    let exported = arclang::compiler::capella_export::export(&result.ast, "remote_start");
    let model = Model::from_strings(&[("remote_start.capella", exported.capella.as_str()), ("remote_start.aird", exported.aird.as_str())]).unwrap();
    let machines = export::export(&model, Strategy::StateMachines, &[], &Options::default()).unwrap();
    let mut pivot = pivot::read(&machines.pivot, Path::new(".")).unwrap();
    let hostile = "driver's request'); system('touch owned'); disp('\nsystem('second line')";
    let mut written = 0;
    for o in &mut pivot.objs {
        for field in [&mut o.description, &mut o.condition, &mut o.event, &mut o.entry, &mut o.id] {
            if field.as_deref().is_some_and(|text| !text.is_empty()) {
                *field = Some(hostile.to_string());
                written += 1;
            }
        }
    }
    assert!(written > 0, "the model has texts to write");
    let script = script(&pivot).unwrap();
    assert!(script.contains("driver''s request''); system(''touch owned''); disp('' system(''second line'')"), "{script}");
    for line in script.lines() {
        assert!(line.matches('\'').count() % 2 == 0, "a quote is left open: {line}");
        assert!(!line.trim_start().starts_with("system("), "a text became a statement: {line}");
    }
}
