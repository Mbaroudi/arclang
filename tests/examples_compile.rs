//! Golden corpus: every active example must compile with the strict parser.
//!
//! `examples/legacy/` is excluded on purpose — it holds files written for
//! unimplemented syntax versions and is documented as non-compiling.

use arclang::compiler::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};

fn collect_arc_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().map_or(false, |n| n == "legacy") {
                continue;
            }
            collect_arc_files(&path, out);
        } else if path.extension().map_or(false, |ext| ext == "arc") {
            out.push(path);
        }
    }
}

#[test]
fn all_active_examples_compile() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut files = Vec::new();
    collect_arc_files(&examples_dir, &mut files);
    files.sort();

    assert!(
        !files.is_empty(),
        "no .arc examples found under {}",
        examples_dir.display()
    );

    let mut failures = Vec::new();
    for file in &files {
        let mut compiler = Compiler::new(CompilerConfig::default());
        if let Err(e) = compiler.compile_file(file) {
            failures.push(format!("{}: {}", file.display(), e));
        }
    }

    assert!(
        failures.is_empty(),
        "{} example(s) failed to compile:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn flagship_example_extracts_real_elements() {
    // Regression guard for the historical bug where a full 5-layer model
    // compiled "successfully" to zero elements because the parser silently
    // skipped everything it did not understand.
    let flagship = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/complete_emergency_braking_simple.arc");

    let mut compiler = Compiler::new(CompilerConfig::default());
    let result = compiler.compile_file(&flagship).expect("flagship example must compile");

    assert!(
        !result.ast.operational_analysis.is_empty(),
        "operational analysis layer must be extracted"
    );
    assert!(
        !result.ast.system_analysis.is_empty(),
        "system analysis layer must be extracted"
    );
    assert!(
        !result.ast.logical_architecture.is_empty(),
        "logical architecture layer must be extracted"
    );
    assert!(
        !result.ast.physical_architecture.is_empty(),
        "physical architecture layer must be extracted"
    );
    assert!(
        result.semantic_model.components.len() >= 10,
        "expected at least 10 components, got {}",
        result.semantic_model.components.len()
    );
    assert!(
        !result.semantic_model.functions.is_empty(),
        "functions must be extracted"
    );
}
