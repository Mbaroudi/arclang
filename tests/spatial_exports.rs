//! The spatial exports of the example corridor, end to end: the GeoJSON
//! is RFC 7946 with every located element and every link, the STEP file
//! has one solid per shape. With Open CASCADE at hand
//! (`ARCLANG_OCC_PYTHON=/path/to/python` of an environment with
//! pythonocc-core), the STEP file is read by it: every solid valid, each
//! with the volume the model declares.

use arclang::compiler::spatial::{geojson, step};
use arclang::compiler::{Compiler, CompilerConfig};
use std::path::Path;

fn corridor() -> arclang::CompilationResult {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/spatial/charging_corridor.arc");
    Compiler::new(CompilerConfig::default()).compile_file(&path).expect("the corridor compiles")
}

#[test]
fn the_corridor_compiles_without_a_finding() {
    let result = corridor();
    let findings = arclang::compiler::rules::findings(&result);
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn the_corridor_on_a_map_has_every_located_element_and_every_link() {
    let (text, notes) = geojson::export(&corridor().ast);
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let features = value["features"].as_array().unwrap();
    let kinds = |geometry: &str| features.iter().filter(|f| f["geometry"]["type"] == geometry).count();
    assert_eq!((kinds("Point"), kinds("LineString")), (7, 3), "{text}");
    assert!(notes.is_empty(), "{notes:?}");
    for feature in features {
        let first = &feature["geometry"]["coordinates"];
        let point = if first[0].is_array() { &first[0] } else { first };
        let (lon, lat) = (point[0].as_f64().unwrap(), point[1].as_f64().unwrap());
        assert!((3.0..6.0).contains(&lon) && (45.0..48.0).contains(&lat), "longitude first: {feature}");
    }
}

#[test]
fn the_corridor_in_cad_has_one_solid_per_shape_and_open_cascade_agrees() {
    let exported = step::export(&corridor().ast, "2026-10-20T00:00:00").unwrap();
    assert_eq!(exported.solids, 9);
    let Ok(python) = std::env::var("ARCLANG_OCC_PYTHON") else {
        eprintln!("ARCLANG_OCC_PYTHON is not set: the STEP file is not read by Open CASCADE here (CI does it)");
        return;
    };
    let file = std::env::temp_dir().join(format!("arclang-corridor-{}.step", std::process::id()));
    std::fs::write(&file, &exported.text).unwrap();
    let tool = Path::new(env!("CARGO_MANIFEST_DIR")).join("tools/step_validate.py");
    let volumes = ["39", "39", "39", "39", "8.448", "8.448", "0.576", "1.71059", "1.71059"];
    let output = std::process::Command::new(python)
        .arg(tool)
        .arg(&file)
        .args(["--expect-solids", "9", "--expect-volume"])
        .args(volumes)
        .output()
        .expect("python runs");
    let _ = std::fs::remove_file(&file);
    let report = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "{report}\n{}", String::from_utf8_lossy(&output.stderr));
    assert!(report.contains("OK "), "{report}");
}
