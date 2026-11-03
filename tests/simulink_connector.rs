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
