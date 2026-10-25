//! `arclang impact`: the CLI end to end, with goldens (text and Markdown on
//! the flagship example) and the formats. Regenerate the goldens with
//! `UPDATE_GOLDEN=1 cargo test --release --test impact`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn aeb() -> PathBuf {
    root().join("examples/complete_emergency_braking_simple.arc")
}

fn arclang(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arclang")).args(args).output().expect("arclang runs")
}

fn stdout(args: &[&str]) -> String {
    let out = arclang(args);
    assert!(out.status.success(), "arclang {args:?}: {}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap()
}

fn golden(name: &str, actual: &str) {
    let path = root().join("tests/fixtures/impact").join(name);
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(&path, actual).unwrap();
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("no golden {name}: run with UPDATE_GOLDEN=1"));
    assert_eq!(expected, actual, "{name} differs: UPDATE_GOLDEN=1 regenerates it");
}

fn model() -> String {
    aeb().display().to_string().replace(root().to_str().unwrap(), ".")
}

#[test]
fn the_text_of_an_element_keeps_its_layout_and_is_golden() {
    let text = stdout(&["impact", aeb().to_str().unwrap(), "RadarSensor"]);
    assert!(text.starts_with("Impact analysis for 'LC-RadarSensor':"), "{text}");
    assert!(text.contains("\n  Distance 1 (directly affected):\n"));
    assert!(text.contains("\n  Total affected elements: "));
    golden("radar_sensor.txt", &text);
}

#[test]
fn a_latency_what_if_is_golden_in_markdown() {
    let out = stdout(&["impact", aeb().to_str().unwrap(), "--change", r#"{"op":"set","element":"FuseData","key":"latency","value":"80 ms"}"#, "-f", "md"]);
    assert!(out.contains("PASS → FAIL"));
    // The path of the model in the header is the machine's: the golden keeps what follows.
    let body: String = out.lines().map(|l| if l.starts_with("Model:") { "Model: <path>".to_string() } else { l.to_string() }).collect::<Vec<_>>().join("\n");
    golden("fuse_latency.md", &body);
    let _ = model();
}

#[test]
fn json_carries_the_summary_the_gate_and_the_paths() {
    let json: serde_json::Value = serde_json::from_str(&stdout(&["impact", aeb().to_str().unwrap(), "-e", "REQ-AEB-004", "--depth", "2", "-f", "json"])).unwrap();
    assert_eq!(json["success"], true);
    assert_eq!(json["seeds"][0]["id"], "REQ-AEB-004");
    assert_eq!(json["gate"]["passed"], true);
    let test = json["impacted"].as_array().unwrap().iter().find(|i| i["id"] == "TC-004").expect("TC-004");
    assert_eq!(test["why"], "REQ-AEB-004 --verified by--> TC-004");
    assert_eq!(test["category"], "test");
}

#[test]
fn a_delete_and_a_refused_delete() {
    let out = stdout(&["impact", aeb().to_str().unwrap(), "--delete", "SystemSupervisor"]);
    assert!(out.contains("PASS → FAIL") && out.contains("REQ-AEB-007"), "{out}");
    let refused = arclang(&["impact", aeb().to_str().unwrap(), "--delete", "RadarSensor", "-f", "json"]);
    assert!(!refused.status.success());
    let answer: serde_json::Value = serde_json::from_slice(&refused.stdout).unwrap();
    assert_eq!(answer["success"], false);
    assert!(answer["error"].as_str().unwrap().contains("scenario 'EmergencyStop'"));
}

#[test]
fn kinds_and_depth_narrow_the_walk_and_the_exports_write_files() {
    let json: serde_json::Value = serde_json::from_str(&stdout(&["impact", aeb().to_str().unwrap(), "-e", "LC-BrakeController", "--kinds", "satisfies", "--depth", "1", "-f", "json"])).unwrap();
    let kinds: Vec<&str> = json["impacted"].as_array().unwrap().iter().flat_map(|i| i["path"].as_array().unwrap().iter().map(|s| s["kind"].as_str().unwrap())).collect();
    assert!(!kinds.is_empty() && kinds.iter().all(|k| *k == "satisfies"), "{kinds:?}");
    let dir = std::env::temp_dir().join(format!("arclang-impact-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for (format, file) in [("html", "r.html"), ("xlsx", "r.xlsx"), ("tsv", "r.tsv")] {
        let target = dir.join(file);
        let out = arclang(&["impact", aeb().to_str().unwrap(), "-e", "LC-BrakeController", "-f", format, "-o", target.to_str().unwrap()]);
        assert!(out.status.success(), "{format}: {}", String::from_utf8_lossy(&out.stderr));
        assert!(std::fs::metadata(&target).unwrap().len() > 100, "{format} is empty");
    }
    assert_eq!(&std::fs::read(dir.join("r.xlsx")).unwrap()[..2], b"PK");
    assert!(!arclang(&["impact", aeb().to_str().unwrap(), "-e", "LC-BrakeController", "-f", "xlsx"]).status.success(), "xlsx needs -o");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn nothing_to_analyse_is_an_error() {
    assert!(!arclang(&["impact", aeb().to_str().unwrap()]).status.success());
    assert!(!arclang(&["impact", aeb().to_str().unwrap(), "NoSuchThing"]).status.success());
    assert!(!arclang(&["impact", aeb().to_str().unwrap(), "-e", "LC-RadarSensor", "--kinds", "gravity"]).status.success());
}
