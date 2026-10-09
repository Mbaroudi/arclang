//! `spec/METAMODEL.md` is GENERATED from the metamodel declared in code.
//! This test is what keeps the specification honest: if the document and
//! the compiler disagree, the build fails.

use arclang::compiler::metamodel::Metamodel;
use std::path::Path;

#[test]
fn metamodel_spec_matches_the_compiler() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("spec/METAMODEL.md");
    let on_disk = std::fs::read_to_string(&path).unwrap_or_default();
    let generated = Metamodel::current().to_markdown();
    assert!(
        on_disk == generated,
        "spec/METAMODEL.md is out of date with src/compiler/metamodel.rs.\n\
         Regenerate it:  cargo run -q -- metamodel --format markdown > spec/METAMODEL.md"
    );
}

#[test]
fn every_kind_declares_its_arcadia_and_sysml_mapping() {
    for kind in &Metamodel::current().kinds {
        assert!(!kind.arcadia.is_empty(), "{} has no Arcadia mapping", kind.name);
        assert!(!kind.sysml.is_empty(), "{} has no SysML v2 mapping", kind.name);
        assert!(!kind.doc.is_empty(), "{} is undocumented", kind.name);
    }
}
