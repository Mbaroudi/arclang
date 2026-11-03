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

use arclang::compiler::simulink_connector::m2m::{self, model::Model, Options, Strategy};
use std::collections::HashMap;

/// A configuration of the connector's m2m tests (`testN.properties`).
fn configuration(name: &str) -> HashMap<String, String> {
    let text = std::fs::read_to_string(fixtures().join("m2m/configurations").join(format!("{name}.properties"))).unwrap();
    text.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect()
}

/// The script of a configuration: the Capella selection made a pivot, the pivot a script.
fn m2m_script(name: &str) -> String {
    let props = configuration(name);
    let project = &props["project"];
    let model = Model::load(&fixtures().join("m2m").join(project).join(format!("{project}.capella"))).unwrap();
    let strategy = Strategy::parse(&props["strategy"]).unwrap();
    let mut options = Options::default();
    if let Some(v) = props.get("leaf.functions.export.option") {
        options.leaf_functions = v.clone();
    }
    if let Some(v) = props.get("port.naming.option") {
        options.port_naming = v.clone();
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
