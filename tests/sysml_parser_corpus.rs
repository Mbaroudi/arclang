//! The complete SysML v2 reader against the OMG pilot's example corpus
//! (`sysml-v2/sysml/src/examples`, not tracked): every `.sysml` must parse.
//! The survey prints `PARSE ok|KO <file>` lines and a total.

use arclang::compiler::sysml_parser;
use std::path::{Path, PathBuf};

fn corpus() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "sysml") {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(&Path::new(env!("CARGO_MANIFEST_DIR")).join("sysml-v2/sysml/src/examples"), &mut files);
    files.sort();
    files
}

#[test]
#[ignore = "survey over the OMG corpus, run with --include-ignored --nocapture"]
fn every_omg_example_parses() {
    let files = corpus();
    let mut ok = 0;
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap();
        match sysml_parser::parse(&text) {
            Ok(_) => {
                ok += 1;
                println!("PARSE ok {}", file.display());
            }
            Err(e) => println!("PARSE KO {}: {e}", file.display()),
        }
    }
    println!("PARSE total {ok}/{}", files.len());
}

use arclang::compiler::{elements, sysml_import, sysmlv2_generator, Compiler, CompilerConfig};
use std::collections::BTreeSet;

fn shape(source: &str) -> (BTreeSet<(String, String)>, BTreeSet<(String, String, String)>) {
    let result = Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles");
    let graph = elements::build(&result.ast, &result.semantic_model);
    let by_uuid: std::collections::BTreeMap<&str, &str> = graph.elements.iter().map(|e| (e.uuid.as_str(), e.id.as_str())).collect();
    let elements = graph.elements.iter().map(|e| (e.kind.to_string(), e.id.clone())).collect();
    let relationships = graph.relationships.iter().filter_map(|r| Some((r.kind.to_string(), by_uuid.get(r.source.as_str())?.to_string(), by_uuid.get(r.target.as_str())?.to_string()))).collect();
    (elements, relationships)
}

/// The export of an example, read by the complete parser and imported back.
fn through_parser(relative: &str) -> (usize, usize, usize, usize, Vec<String>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let source = std::fs::read_to_string(&path).unwrap();
    let before = Compiler::new(CompilerConfig::default()).compile_string(&source).expect("the example compiles");
    let text = sysmlv2_generator::generate_sysmlv2(&before.semantic_model, &before.ast);
    let built = sysml_parser::records(&text).unwrap_or_else(|e| panic!("{relative}: {e}"));
    let imported = sysml_import::import_records(&built.records).unwrap_or_else(|f| panic!("{relative}: {}\n{}", f.message, f.rejected.unwrap_or_default()));
    let (elements_before, relationships_before) = shape(&source);
    let (elements_after, relationships_after) = shape(&imported.source);
    let lost: Vec<String> = elements_before.difference(&elements_after).map(|(k, i)| format!("{k} {i}")).chain(relationships_before.difference(&relationships_after).map(|(k, s, t)| format!("{k} {s} -> {t}"))).collect();
    (elements_before.intersection(&elements_after).count(), elements_before.len(), relationships_before.intersection(&relationships_after).count(), relationships_before.len(), lost)
}

#[test]
fn the_flagship_export_comes_back_through_the_parser() {
    let (kept, total, kept_relationships, total_relationships, lost) = through_parser("examples/complete_emergency_braking_simple.arc");
    println!("PARSER ROUNDTRIP flagship: elements {kept}/{total}, relationships {kept_relationships}/{total_relationships}");
    for l in lost.iter().take(40) {
        println!("PARSER ROUNDTRIP   lost {l}");
    }
    assert_eq!(kept, total, "elements");
    assert_eq!(kept_relationships, total_relationships, "relationships");
}

#[test]
#[ignore = "survey: imports every OMG example through the parser, run with --include-ignored --nocapture"]
fn every_omg_example_imports_through_the_parser() {
    let files = corpus();
    let mut ok = 0;
    let mut compiled = 0;
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap();
        let built = match sysml_parser::records(&text) {
            Ok(built) => built,
            Err(e) => {
                println!("IMPORT KO {}: parse: {e}", file.display());
                continue;
            }
        };
        ok += 1;
        match sysml_import::import_records(&built.records) {
            Ok(imported) => {
                compiled += 1;
                let elements: usize = imported.counts.iter().filter(|(k, _)| !k.contains("left out")).map(|(_, v)| v).sum();
                println!("IMPORT ok {} ({} unresolved, {} written)", file.display(), built.unresolved.len(), elements);
            }
            Err(f) => println!("IMPORT KO {}: {}", file.display(), f.message.lines().next().unwrap_or("")),
        }
    }
    println!("IMPORT total parsed {ok}/{} compiled {compiled}/{}", files.len(), files.len());
}

#[test]
#[ignore = "debug: SYSML_DUMP=<file.sysml> writes its records as JSON next to it"]
fn dump_records() {
    let Ok(path) = std::env::var("SYSML_DUMP") else { return };
    let text = std::fs::read_to_string(&path).unwrap();
    let built = sysml_parser::records(&text).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(format!("{path}.records.json"), serde_json::to_string_pretty(&built.records).unwrap()).unwrap();
    println!("DUMP {} records, {} unresolved", built.records.len(), built.unresolved.len());
    for u in built.unresolved.iter().take(30) {
        println!("DUMP   {u}");
    }
}
