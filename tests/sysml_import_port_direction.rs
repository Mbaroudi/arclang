//! The direction of an imported SysML v2 port comes from the directed items
//! of its definition, reversed when the port is conjugated (`port p : ~P`),
//! and not from letters inside the definition's name. The shapes are those of
//! the Apollo 11 model of Airbus (Systems Engineering, 2026, MPL 2.0): a
//! staging port viewed from the lower stage (`inout loads`, `out commands`),
//! its conjugate on the upper stage, and a symmetric docking port. Read from
//! their names, `StagingPort` and `DockingPort` both hold "in": every end
//! became an input and each interface was reported as leaving an in port.

use arclang::compiler::sysml_import;
use arclang::{Compiler, CompilerConfig};

const STAGES: &str = "package Stages {
    item def StructuralLoad;
    item def CommandSignal;
    port def StagingPort {
        inout item loads : StructuralLoad;
        out item commands : CommandSignal;
    }
    port def DockingPort {
        inout item loads : StructuralLoad;
    }
    port def TorqueIn;
    interface def StagingInterface {
        end source : StagingPort;
        end target : ~StagingPort;
    }
    part rocket {
        part stage1 { port upperStagePort : StagingPort; }
        part stage2 {
            port lowerStagePort : ~StagingPort;
            port dockingPort : DockingPort;
            port shaft : TorqueIn;
        }
        part stage3 { port dockingPort : DockingPort; }
        interface stage1ToStage2 : StagingInterface connect stage1.upperStagePort to stage2.lowerStagePort;
        connect stage2.dockingPort to stage3.dockingPort;
    }
}
";

fn imported(text: &str) -> String {
    sysml_import::import_text(text).unwrap_or_else(|f| panic!("{}\n{}", f.message, f.rejected.unwrap_or_default())).source
}

fn assert_has(source: &str, expected: &str) {
    assert!(source.contains(expected), "expected `{expected}` in:\n{source}");
}

#[test]
fn a_port_takes_the_direction_of_its_items_and_its_conjugate_the_reverse() {
    let source = imported(STAGES);
    assert_has(&source, "port out \"upperStagePort\"");
    assert_has(&source, "port in \"lowerStagePort\"");
    // Only inout items: a port both ways, conjugated or not.
    assert_has(&source, "port inout \"dockingPort\"");
}

#[test]
fn a_definition_without_items_is_read_from_the_words_of_its_name() {
    assert_has(&imported(STAGES), "port in \"shaft\"");
}

#[test]
fn the_staging_interface_leaves_an_out_port_and_arrives_on_an_in_port() {
    let source = imported(STAGES);
    let result = Compiler::new(CompilerConfig::default()).compile_string(&source).unwrap_or_else(|e| panic!("{e}\n{source}"));
    let found: Vec<&String> = result.warnings.iter().filter(|w| w.starts_with("exchange direction: ")).collect();
    assert!(found.is_empty(), "{found:#?}\n{source}");
}

#[test]
fn a_negation_keeps_the_space_after_not() {
    // ArcLang's `require` reads a comparison, so the import is refused; the
    // text it refuses says `not contaminated`, not `notcontaminated`.
    let refused = match sysml_import::import_text(
        "package P {
    requirement def R {
        attribute contaminated : ScalarValues::Boolean;
        require constraint { not contaminated }
    }
}
",
    ) {
        Ok(imported) => imported.source,
        Err(failure) => failure.rejected.unwrap_or_default(),
    };
    assert!(!refused.contains("notcontaminated"), "{refused}");
    assert_has(&refused, "require: [\"not contaminated\"]");
}
