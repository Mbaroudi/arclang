//! Timeslices and snapshots: the stretches and moments of an element's life
//! (SysML v2 portions), checked by the compiler, kept by the SysML v2 round
//! trip, elements of the model.

use arclang::compiler::ast::OccurrenceKind;
use arclang::compiler::{elements, Compiler, CompilerConfig};

const MODEL: &str = r#"
model "Mission" {}

logical_architecture "Drone" {
    component "Drone" {
        id: "LC-DRONE"
        altitude: 0 m
        battery: 1.0
        timeslice "Takeoff" {
            description: "From the pad to cruise altitude"
            start: 0 s
            end: 40 s
            snapshot "Liftoff" { at: 0 s altitude: 0 m }
            snapshot "At cruise" { at: 40 s altitude: 120 m }
        }
        timeslice "Cruise" {
            start: 40 s
            snapshot "Low battery" { battery: 0.2 }
        }
        snapshot "Landed" { at: 25 min altitude: 0 m battery: 0.1 }
    }
}
"#;

fn compile(text: &str) -> Result<arclang::CompilationResult, String> {
    Compiler::new(CompilerConfig::default()).compile_string(text).map_err(|e| e.to_string())
}

#[test]
fn a_component_declares_the_timeslices_and_snapshots_of_its_life() {
    let result = compile(MODEL).unwrap();
    let drone = &result.ast.logical_architecture[0].components[0];
    let kinds: Vec<(OccurrenceKind, &str, usize)> = drone.occurrences.iter().map(|o| (o.kind, o.name.as_str(), o.snapshots.len())).collect();
    assert_eq!(kinds, [(OccurrenceKind::Timeslice, "Takeoff", 2), (OccurrenceKind::Timeslice, "Cruise", 1), (OccurrenceKind::Snapshot, "Landed", 0)]);
}

#[test]
fn they_are_elements_of_the_model_inside_their_element() {
    let result = compile(MODEL).unwrap();
    let graph = elements::build(&result.ast, &result.semantic_model);
    let find = |id: &str| graph.elements.iter().find(|e| e.id == id).unwrap_or_else(|| panic!("no {id}"));
    assert_eq!(find("LC-DRONE.Takeoff").kind, "Timeslice");
    assert_eq!(find("LC-DRONE.Takeoff.At cruise").kind, "Snapshot");
    assert_eq!(find("LC-DRONE.Takeoff.At cruise").owner.as_deref(), Some(find("LC-DRONE.Takeoff").uuid.as_str()));
    assert_eq!(find("LC-DRONE.Landed").owner.as_deref(), Some(find("LC-DRONE").uuid.as_str()));
}

#[test]
fn the_compiler_checks_when_they_are() {
    for (from, to, says) in [
        ("end: 40 s", "end: 40 m", "end: 40 m is not a time"),
        ("start: 0 s\n            end: 40 s", "start: 50 s\n            end: 40 s", "'Takeoff': it ends before it starts"),
        ("snapshot \"At cruise\" { at: 40 s", "snapshot \"At cruise\" { at: 45 s", "at 45 s is outside its timeslice 'Takeoff'"),
        ("snapshot \"Landed\" { at: 25 min", "snapshot \"Landed\" { start: 25 min", "a snapshot is one moment"),
        ("altitude: 120 m", "altitude: 120 kg", "altitude is a length of 'LC-DRONE', not a mass"),
    ] {
        let error = compile(&MODEL.replace(from, to)).unwrap_err();
        assert!(error.contains(says), "{says}\n{error}");
    }
    let error = compile(&MODEL.replace("snapshot \"Landed\"", "timeslice \"Takeoff\" { } snapshot \"Landed\"")).unwrap_err();
    assert!(error.contains("two timeslices or snapshots are named 'Takeoff'"), "{error}");
}

#[test]
fn a_snapshot_holds_no_snapshot() {
    let error = compile(&MODEL.replace("snapshot \"Landed\" {", "snapshot \"Landed\" { snapshot \"Inner\" { }")).unwrap_err();
    assert!(error.contains("a snapshot is one moment, it holds no snapshot"), "{error}");
}

fn import_sysml(text: &str) -> String {
    let built = arclang::compiler::sysml_parser::records(text).unwrap_or_else(|e| panic!("the SysML does not parse: {e}\n{text}"));
    arclang::compiler::sysml_import::import_records(&built.records).unwrap_or_else(|f| panic!("{}", f.message)).source
}

#[test]
fn sysml_v2_says_them_as_portions_and_reads_them_back_whole() {
    let before = compile(MODEL).unwrap();
    let text = arclang::compiler::sysmlv2_generator::generate_sysmlv2(&before.semantic_model, &before.ast);
    for line in ["timeslice Takeoff {", "snapshot 'At cruise' {", "attribute :>> altitude = 120 [m];", "attribute start : DurationValue = 0 [s];"] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    let source = import_sysml(&text);
    let after = compile(&source).unwrap_or_else(|e| panic!("{e}\n{source}"));
    let occurrences = |r: &arclang::CompilationResult| r.ast.logical_architecture[0].components[0].occurrences.clone();
    assert_eq!(occurrences(&after), occurrences(&before), "{source}");
}

#[test]
fn timeslices_and_snapshots_written_by_another_tool_are_read() {
    let source = import_sysml(
        r#"package Mission {
    private import ISQ::*;
    private import SI::*;
    part def Drone { attribute altitude : LengthValue; }
    part drone : Drone {
        timeslice takeoff {
            snapshot atCruise { :>> altitude = 120 [m]; }
        }
        snapshot landed { :>> altitude = 0 [m]; }
    }
}"#,
    );
    let result = compile(&source).unwrap_or_else(|e| panic!("{e}\n{source}"));
    let drone = &result.ast.logical_architecture[0].components[0];
    assert_eq!(drone.occurrences.len(), 2, "{source}");
    assert_eq!(drone.occurrences[0].snapshots[0].attributes["altitude"].display(), "120 m");
}
