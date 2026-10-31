//! The drawing of the Modelica package: Placement, Line, Icon and Diagram
//! annotations that make Modex and OMEdit draw the architecture, and the
//! guarantee that the simulation path is untouched by them.

use arclang::compiler::diagram::layout_file::LayoutFile;
use arclang::compiler::modelica::{self, Options};
use arclang::compiler::{Compiler, CompilerConfig};
use std::path::{Path, PathBuf};

const DEMO: &str = "examples/automotive/ev_range_simulation.arc";

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn compile(path: &str) -> arclang::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_file(&root().join(path)).expect("compiles")
}

fn drawn(layout: Option<LayoutFile>) -> modelica::Export {
    modelica::export_with(&compile(DEMO).ast, &Options { placement: true, layout }).unwrap()
}

/// The ArcLang ids a package carries, in order.
fn ids(text: &str) -> Vec<String> {
    text.split("__ArcLang(id=\"").skip(1).map(|rest| rest.split('"').next().unwrap().to_string()).collect()
}

/// (origin x, origin y, half width, half height) of each `replaceable` instance of a model.
fn boxes(text: &str, model: &str) -> Vec<(f64, f64, f64, f64)> {
    let start = text.find(&format!("partial model {model} ")).expect("model");
    let body = &text[start..];
    let body = &body[..body.find(&format!("end {model};")).unwrap()];
    body.lines()
        .filter(|l| l.trim_start().starts_with("replaceable"))
        .map(|line| {
            let origin = line.split("origin={").nth(1).unwrap().split('}').next().unwrap();
            let extent = line.split("extent={{").nth(1).unwrap().split("}}").next().unwrap();
            let num = |s: &str| s.trim().parse::<f64>().unwrap();
            let o: Vec<f64> = origin.split(',').map(num).collect();
            let e: Vec<f64> = extent.replace("},{", ",").split(',').map(num).collect();
            (o[0], o[1], e[2], e[3])
        })
        .collect()
}

#[test]
fn the_drawn_demo_matches_its_golden() {
    let export = drawn(None);
    let golden = root().join("tests/fixtures/modelica/EVRangeSimulation.placed.mo");
    if std::env::var_os("ARCLANG_BLESS").is_some() {
        std::fs::write(&golden, &export.text).unwrap();
    }
    let expected = std::fs::read_to_string(&golden).expect("golden package");
    assert_eq!(export.text, expected, "regenerate with ARCLANG_BLESS=1 when the change is intended");
    assert_eq!(drawn(None).text, export.text, "the same model draws the same way");
}

#[test]
fn without_placement_the_package_is_the_old_one_byte_for_byte() {
    let ast = compile(DEMO).ast;
    let plain = modelica::export(&ast).unwrap().text;
    assert_eq!(modelica::export_with(&ast, &Options::default()).unwrap().text, plain);
    let golden = std::fs::read_to_string(root().join("tests/fixtures/modelica/EVRangeSimulation.mo")).unwrap();
    assert_eq!(plain, golden, "the simulation path hashes this text: it must not change");
    assert!(!plain.contains("Placement(") && !plain.contains("Line(points"));
}

#[test]
fn the_drawing_keeps_every_arclang_id_and_every_connection() {
    let ast = compile(DEMO).ast;
    let plain = modelica::export(&ast).unwrap().text;
    let text = drawn(None).text;
    assert_eq!(ids(&text), ids(&plain));
    let connects = |t: &str| t.lines().filter(|l| l.trim_start().starts_with("connect(")).map(|l| l.split(" annotation").next().unwrap().trim().to_string()).collect::<Vec<_>>();
    assert_eq!(connects(&text), connects(&plain));
    assert!(text.contains("Line(points={{"));
    assert!(text.contains("color={0,0,255}"), "electrical wires are blue");
    assert!(text.contains("color={0,0,127}"), "signal wires are dark blue");
    assert!(text.contains("color={95,95,95}"), "mechanical wires are grey");
    assert!(text.contains("Diagram(coordinateSystem(extent={{"));
    assert!(text.contains("Icon(coordinateSystem(extent={{"));
}

#[test]
fn boxes_do_not_overlap_and_every_instance_is_placed() {
    let text = drawn(None).text;
    for model in ["City", "Touring", "VehicleControlUnit"] {
        let placed = boxes(&text, model);
        assert!(!placed.is_empty(), "{model} has instances");
        for (i, a) in placed.iter().enumerate() {
            for b in &placed[i + 1..] {
                let apart = (a.0 - b.0).abs() >= a.2 + b.2 || (a.1 - b.1).abs() >= a.3 + b.3;
                assert!(apart, "{model}: {a:?} overlaps {b:?}");
            }
        }
    }
    assert_eq!(boxes(&text, "Touring").len(), 4);
}

#[test]
fn signals_flow_from_the_left_to_the_right() {
    let text = drawn(None).text;
    let touring = boxes(&text, "Touring");
    // Battery -> drive -> body -> control unit, as the connections go.
    assert!(touring.windows(2).all(|w| w[0].0 < w[1].0), "{touring:?}");
}

#[test]
fn the_layout_file_moves_a_box_from_where_the_automatic_layout_puts_it() {
    let layout = LayoutFile::parse(r#"{"arclang_layout":"1","arrangements":[{"view":"pab","open":null,"places":{"PN-DRIVE":{"dx":100,"dy":-50}}}]}"#).unwrap();
    let automatic = boxes(&drawn(None).text, "Touring");
    let moved = boxes(&drawn(Some(layout)).text, "Touring");
    assert_eq!(moved[0], automatic[0]);
    assert!((moved[1].0 - automatic[1].0 - 10.0).abs() < 0.11, "{moved:?} {automatic:?}");
    assert!((moved[1].1 - automatic[1].1 - 5.0).abs() < 0.11, "dy -50 is up on the sheet (y down), up in Modelica (y up)");
}

#[test]
fn split_writes_one_file_per_class_in_a_library_folder() {
    let export = drawn(None);
    let paths: Vec<&str> = export.files.iter().map(|(p, _)| p.as_str()).collect();
    for expected in [
        "EVRangeSimulation/package.mo",
        "EVRangeSimulation/package.order",
        "EVRangeSimulation/Physical/package.mo",
        "EVRangeSimulation/Physical/BatteryPack.mo",
        "EVRangeSimulation/Behaviour/VehicleControlUnit/SpeedController.mo",
        "EVRangeSimulation/Systems/Touring.mo",
    ] {
        assert!(paths.contains(&expected), "missing {expected} in {paths:?}");
    }
    let (_, touring) = export.files.iter().find(|(p, _)| p.ends_with("Systems/Touring.mo")).unwrap();
    assert!(touring.contains("within EVRangeSimulation.Systems;\npartial model Touring"));
    // Every class of the single file is in a file of the folder, with the same ids.
    let mut from_files: Vec<String> = export.files.iter().flat_map(|(_, t)| ids(t)).collect();
    let mut from_single = ids(&export.text);
    from_files.sort();
    from_single.sort();
    from_files.dedup();
    from_single.dedup();
    assert_eq!(from_files, from_single);
}

#[test]
fn a_split_export_replaces_the_single_file_arclang_wrote_beside_it() {
    let dir = tempfile::tempdir().unwrap();
    let export = |split: bool| {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_arclang"));
        command.arg("export").arg(root().join(DEMO)).args(["-f", "modelica", "-o"]).arg(dir.path());
        if split {
            command.arg("--split");
        }
        let out = command.output().unwrap();
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).to_string()
    };
    let single = dir.path().join("EVRangeSimulation.mo");
    export(false);
    assert!(single.is_file());
    let text = export(true);
    assert!(!single.exists(), "the package would be defined twice");
    assert!(text.contains("removed") && dir.path().join("EVRangeSimulation/Systems/City.mo").is_file(), "{text}");
    // A file of the same name that ArcLang did not write is never touched.
    std::fs::write(&single, "package EVRangeSimulation end EVRangeSimulation;").unwrap();
    export(true);
    assert!(single.is_file());
    // And a single-file export says the folder is there.
    assert!(export(false).contains("also holds a split export"));
}

#[test]
fn every_example_with_physical_nodes_draws_without_overlap() {
    let mut seen = 0;
    let mut stack: Vec<PathBuf> = vec![root().join("examples")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "arc") && !path.to_string_lossy().contains("legacy") {
                let Ok(result) = Compiler::new(CompilerConfig::default()).compile_file(&path) else { continue };
                let Ok(export) = modelica::export_with(&result.ast, &Options { placement: true, layout: None }) else { continue };
                seen += 1;
                for line in export.text.lines().filter(|l| l.contains("Placement(") || l.contains("Line(points")) {
                    assert!(!line.contains("NaN") && !line.contains("inf"), "{}: {line}", path.display());
                }
                for model in export.text.lines().filter_map(|l| l.trim_start().strip_prefix("partial model ")).filter_map(|l| l.split_whitespace().next()) {
                    let placed = boxes(&export.text, model);
                    for (i, a) in placed.iter().enumerate() {
                        for b in &placed[i + 1..] {
                            assert!((a.0 - b.0).abs() >= a.2 + b.2 || (a.1 - b.1).abs() >= a.3 + b.3, "{}: {model}: {a:?} overlaps {b:?}", path.display());
                        }
                    }
                }
            }
        }
    }
    assert!(seen >= 3, "examples exported: {seen}");
}
