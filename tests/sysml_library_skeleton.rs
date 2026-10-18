//! `spec/sysml_library_skeleton.tsv` is what the parser reads of the SysML
//! v2 standard library (`sysml-v2/sysml.library`, not tracked): every
//! `.sysml` and `.kerml` file must parse, and the skeleton must match.
//! `UPDATE_SYSML_SKELETON=1` writes it.

use arclang::compiler::sysml_parser::{library, parse, parse_kerml};
use std::path::{Path, PathBuf};

fn library_files(dir: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "sysml" || e == "kerml") {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(dir, &mut files);
    files.sort();
    files
}

#[test]
#[ignore = "needs the SysML v2 release in sysml-v2/"]
fn the_skeleton_is_what_the_parser_reads_of_the_standard_library() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = library_files(&root.join("sysml-v2/sysml.library"));
    assert!(files.len() > 90, "the library is in sysml-v2/sysml.library");
    let mut roots = Vec::new();
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap();
        let parsed = if file.extension().is_some_and(|e| e == "kerml") { parse_kerml(&text) } else { parse(&text) };
        roots.extend(parsed.unwrap_or_else(|e| panic!("{}: {e}", file.display())));
    }
    let skeleton = library::skeleton_of(&roots);
    let path = root.join("spec/sysml_library_skeleton.tsv");
    if std::env::var("UPDATE_SYSML_SKELETON").is_ok() {
        std::fs::write(&path, &skeleton).unwrap();
    }
    assert!(std::fs::read_to_string(&path).unwrap() == skeleton, "spec/sysml_library_skeleton.tsv is out of date: run with UPDATE_SYSML_SKELETON=1");
}

#[test]
fn the_skeleton_holds_the_library_names_models_use() {
    let roots = library::roots();
    let names: Vec<&str> = roots.iter().filter_map(|r| r.name.as_deref()).collect();
    for package in ["ISQ", "SI", "ScalarValues", "ShapeItems", "Quantities", "Base"] {
        assert!(names.contains(&package), "{package} in {names:?}");
    }
}
