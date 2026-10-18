//! ArcLang -> SysML v2 text -> ArcLang: what comes back. The text is read
//! by the complete parser (`sysml_parser`), as `arclang import -f sysml`
//! reads any SysML v2 text, and rendered as API records for the import. The
//! element graph of the model compiled before and after is compared by
//! (kind, id) for elements and (kind, source, target) for relationships,
//! kind by kind, so a loss names what the export or the import drops.

use arclang::compiler::elements;
use arclang::compiler::{sysml_import, sysml_parser, sysmlv2_generator, Compiler, CompilerConfig};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn compile(source: &str) -> arclang::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles")
}

/// (kind, id) of every element, (kind, source id, target id) of every relationship.
fn shape(result: &arclang::CompilationResult) -> (BTreeSet<(String, String)>, BTreeSet<(String, String, String)>) {
    let graph = elements::build(&result.ast, &result.semantic_model);
    let by_uuid: BTreeMap<&str, &str> = graph.elements.iter().map(|e| (e.uuid.as_str(), e.id.as_str())).collect();
    let elements = graph.elements.iter().map(|e| (e.kind.to_string(), e.id.clone())).collect();
    let relationships = graph
        .relationships
        .iter()
        .filter_map(|r| Some((r.kind.to_string(), by_uuid.get(r.source.as_str())?.to_string(), by_uuid.get(r.target.as_str())?.to_string())))
        .collect();
    (elements, relationships)
}

struct Measure {
    kept_elements: usize,
    total_elements: usize,
    kept_relationships: usize,
    total_relationships: usize,
    lost_by_kind: BTreeMap<String, usize>,
    /// Lost relationships by kind: their (source, target) pairs, to name them.
    lost_relationships_by_kind: BTreeMap<String, Vec<(String, String)>>,
}

fn round_trip(path: &Path) -> Measure {
    let source = std::fs::read_to_string(path).unwrap();
    let before = compile(&source);
    let text = sysmlv2_generator::generate_sysmlv2(&before.semantic_model, &before.ast);
    let built = sysml_parser::records(&text).unwrap_or_else(|e| panic!("{}: the export does not parse: {e}", path.display()));
    let imported = sysml_import::import_records(&built.records).unwrap_or_else(|f| panic!("{}: {}\n{}", path.display(), f.message, f.rejected.unwrap_or_default()));
    let after = compile(&imported.source);
    let (elements_before, relationships_before) = shape(&before);
    let (elements_after, relationships_after) = shape(&after);
    let mut lost_by_kind: BTreeMap<String, usize> = BTreeMap::new();
    for (kind, id) in elements_before.difference(&elements_after) {
        // The same id under another kind is a kind confusion, not a loss of the element.
        let kept_elsewhere = elements_after.iter().any(|(_, other)| other == id);
        *lost_by_kind.entry(if kept_elsewhere { format!("{kind} (kind changed)") } else { kind.clone() }).or_insert(0) += 1;
    }
    let mut lost_relationships_by_kind: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (kind, source, target) in relationships_before.difference(&relationships_after) {
        lost_relationships_by_kind.entry(kind.clone()).or_default().push((source.clone(), target.clone()));
    }
    Measure {
        lost_relationships_by_kind,
        kept_elements: elements_before.intersection(&elements_after).count(),
        total_elements: elements_before.len(),
        kept_relationships: relationships_before.intersection(&relationships_after).count(),
        total_relationships: relationships_before.len(),
        lost_by_kind,
    }
}

fn report(name: &str, m: &Measure) {
    println!(
        "ROUNDTRIP {name}: elements {}/{} ({}%), relationships {}/{} ({}%)",
        m.kept_elements,
        m.total_elements,
        100 * m.kept_elements / m.total_elements.max(1),
        m.kept_relationships,
        m.total_relationships,
        100 * m.kept_relationships / m.total_relationships.max(1)
    );
    for (kind, n) in &m.lost_by_kind {
        println!("ROUNDTRIP   lost {n:4} {kind}");
    }
    for (kind, pairs) in &m.lost_relationships_by_kind {
        let shown: Vec<String> = pairs.iter().take(3).map(|(s, t)| format!("{s} -> {t}")).collect();
        println!("ROUNDTRIP   lost relationship {:4} {kind}: {}{}", pairs.len(), shown.join(", "), if pairs.len() > 3 { ", ..." } else { "" });
    }
}

#[test]
fn the_flagship_example_comes_back_from_sysml() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/complete_emergency_braking_simple.arc");
    let m = round_trip(&path);
    report("complete_emergency_braking_simple", &m);
    assert!(m.kept_elements == m.total_elements, "{} of {} elements come back", m.kept_elements, m.total_elements);
    assert!(m.kept_relationships == m.total_relationships, "{} of {} relationships come back", m.kept_relationships, m.total_relationships);
}

#[test]
fn the_capella_ife_example_comes_back_whole() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/capella/in_flight_entertainment.arc");
    let m = round_trip(&path);
    report("in_flight_entertainment", &m);
    assert!(m.kept_elements == m.total_elements, "{} of {} elements come back", m.kept_elements, m.total_elements);
    assert!(m.kept_relationships == m.total_relationships, "{} of {} relationships come back", m.kept_relationships, m.total_relationships);
}

#[test]
#[ignore = "survey: prints the round trip of every example, run with --ignored --nocapture"]
fn every_example_round_trip_is_measured() {
    fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == "legacy" || n == "multifile") {
                    continue;
                }
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "arc") {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(&Path::new(env!("CARGO_MANIFEST_DIR")).join("examples"), &mut files);
    files.sort();
    let (mut kept, mut total) = (0, 0);
    for file in files {
        let source = std::fs::read_to_string(&file).unwrap();
        if Compiler::new(CompilerConfig::default()).compile_string(&source).is_err() {
            continue;
        }
        // A model that does not come back is reported, and the survey goes on.
        let name = file.file_name().unwrap().to_str().unwrap().to_string();
        match std::panic::catch_unwind(|| round_trip(&file)) {
            Ok(m) => {
                report(&name, &m);
                kept += m.kept_elements;
                total += m.total_elements;
            }
            Err(_) => {
                println!("ROUNDTRIP {name}: does not come back (see the panic above)");
                total += shape(&compile(&source)).0.len();
            }
        }
    }
    println!("ROUNDTRIP total elements {kept}/{total} ({}%)", 100 * kept / total.max(1));
}
