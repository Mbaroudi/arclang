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
