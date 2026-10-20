//! The layout file: folds, the open container and manual placement, kept
//! next to the model (`model.layout.json`) so an arranged diagram is shared
//! and versioned like the model. It never changes what the model means: it
//! only says how to show it, and what it names must exist.

use arclang::compiler::diagram::layout_file::{sidecar_path, LayoutFile};
use arclang::compiler::diagram::{build_diagrams, html, DiagramSet};
use arclang::compiler::{Compiler, CompilerConfig};
use std::path::Path;

const MODEL: &str = r#"
logical_architecture "LA" {
    component "Perception" {
        id: "LC-1"
        function "Detect" { id: "LF-1" }
    }
    component "Decision" {
        id: "LC-2"
        function "Decide" { id: "LF-2" }
    }
}
"#;

fn diagrams() -> DiagramSet {
    let result = Compiler::new(CompilerConfig::default())
        .compile_string(MODEL)
        .expect("compiles");
    build_diagrams(&result.ast)
}

const LAYOUT: &str = r#"{
  "arclang_layout": "1",
  "views": { "lab": { "folded": ["LC-2"], "open": null } },
  "arrangements": [
    { "view": "lab", "open": null, "folded": ["LC-2"], "places": { "LC-1": { "dx": 40, "dy": -12.5 } } }
  ]
}"#;

#[test]
fn layout_file_sits_next_to_the_model() {
    assert_eq!(
        sidecar_path(Path::new("models/braking.arc")),
        Path::new("models/braking.layout.json")
    );
}

#[test]
fn layout_file_round_trips_byte_for_byte() {
    let layout = LayoutFile::parse(LAYOUT).expect("parses");
    let written = layout.to_json();
    assert_eq!(LayoutFile::parse(&written).expect("parses again"), layout);
    assert_eq!(LayoutFile::parse(&written).unwrap().to_json(), written, "deterministic");
    assert!(written.ends_with("}\n"));
}

#[test]
fn layout_that_names_what_the_model_draws_is_applied_whole() {
    let mut set = diagrams();
    let warnings = set.apply_layout(LayoutFile::parse(LAYOUT).unwrap());

    assert!(warnings.is_empty(), "got {warnings:?}");
    let layout = set.layout.as_ref().expect("kept");
    assert_eq!(layout.views["lab"].folded, vec!["LC-2"]);
    assert_eq!(layout.arrangements[0].places["LC-1"].dx, 40.0);
}

#[test]
fn layout_naming_something_the_model_no_longer_draws_is_reported_and_left_out() {
    let stale = r#"{
      "arclang_layout": "1",
      "views": {
        "lab": { "folded": ["LC-2", "LC-9", "LF-1"], "open": "LC-8" },
        "pab": { "folded": [], "open": null }
      },
      "arrangements": [
        { "view": "lab", "open": null, "folded": [], "places": { "LC-1": { "dx": 1, "dy": 2 }, "Gone": { "dx": 3, "dy": 4 } } },
        { "view": "lab", "open": "LC-8", "folded": [], "places": { "LC-1": { "dx": 1, "dy": 2 } } }
      ]
    }"#;
    let mut set = diagrams();
    let warnings = set.apply_layout(LayoutFile::parse(stale).unwrap());

    let layout = set.layout.as_ref().unwrap();
    assert_eq!(layout.views["lab"].folded, vec!["LC-2"]);
    assert_eq!(layout.views["lab"].open, None);
    assert!(!layout.views.contains_key("pab"));
    assert_eq!(layout.arrangements.len(), 1);
    assert_eq!(layout.arrangements[0].places.keys().collect::<Vec<_>>(), vec!["LC-1"]);
    for expected in [
        "[layout] view 'pab' is not drawn by this model",
        "[lab] layout file: folded 'LC-9' is not a container of this view",
        "[lab] layout file: folded 'LF-1' is not a container of this view",
        "[lab] layout file: open 'LC-8' is not a container of this view",
        "[lab] layout file: placed 'Gone' is not drawn in this view",
        "[lab] layout file: an arrangement is for the open container 'LC-8', which is not a container of this view",
    ] {
        assert!(warnings.iter().any(|w| w == expected), "missing `{expected}` in {warnings:?}");
    }
}

#[test]
fn layout_file_that_is_not_one_is_refused_with_a_reason() {
    for (text, reason) in [
        ("not json", "is not valid JSON"),
        (r#"{"views":{}}"#, "arclang_layout"),
        (r#"{"arclang_layout":"7","views":{}}"#, "version '7' is not supported"),
        (r#"{"arclang_layout":"1","views":{},"extra":1}"#, "unknown field"),
        (
            r#"{"arclang_layout":"1","arrangements":[{"view":"lab","open":null,"folded":[],"places":{"A":{"dx":1e300,"dy":0}}}]}"#,
            "'A' is placed farther than a sheet can be",
        ),
    ] {
        let error = LayoutFile::parse(text).expect_err(text);
        assert!(error.contains(reason), "`{text}` gave `{error}`, expected `{reason}`");
    }
}

#[test]
fn viewer_receives_the_layout_and_the_name_to_save_it_under() {
    let mut set = diagrams();
    set.apply_layout(LayoutFile::parse(LAYOUT).unwrap());
    set.layout_name = Some("braking.layout.json".to_string());
    let page = html::standalone_html(&set, "Braking");

    assert!(page.contains(r#""layout":{"arclang_layout":"1""#), "layout embedded");
    assert!(page.contains(r#""layout_name":"braking.layout.json""#));

    let plain = html::standalone_html(&diagrams(), "Braking");
    assert!(plain.contains(r#""layout":null"#), "no layout file, no layout");
}

#[test]
fn diagram_model_export_is_unchanged_by_a_missing_layout() {
    let json = serde_json::to_value(diagrams()).unwrap();
    assert!(json.get("layout").is_none());
    assert!(json.get("layout_name").is_none());
}

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("arclang-layout-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

#[test]
fn layout_next_to_the_model_is_picked_up_without_being_asked_for() {
    use arclang::compiler::diagram::layout_file::arrange;
    let dir = scratch("sidecar");
    let model = dir.join("braking.arc");
    std::fs::write(dir.join("braking.layout.json"), LAYOUT).unwrap();

    let mut set = diagrams();
    let warnings = arrange(&mut set, &model, None).expect("reads the sidecar");

    assert!(warnings.is_empty(), "got {warnings:?}");
    assert_eq!(set.layout.as_ref().unwrap().views["lab"].folded, vec!["LC-2"]);
    assert_eq!(set.layout_name.as_deref(), Some("braking.layout.json"));
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn model_without_a_layout_still_names_the_file_to_save() {
    use arclang::compiler::diagram::layout_file::arrange;
    let dir = scratch("none");

    let mut set = diagrams();
    let warnings = arrange(&mut set, &dir.join("braking.arc"), None).expect("nothing to read");

    assert!(warnings.is_empty());
    assert!(set.layout.is_none());
    assert_eq!(set.layout_name.as_deref(), Some("braking.layout.json"));
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn layout_asked_for_by_name_must_exist_and_be_a_layout() {
    use arclang::compiler::diagram::layout_file::arrange;
    let dir = scratch("explicit");
    let model = dir.join("braking.arc");

    let missing = arrange(&mut diagrams(), &model, Some(&dir.join("other.json")))
        .expect_err("a file named on the command line must exist");
    assert!(missing.contains("cannot read layout file"), "got: {missing}");

    std::fs::write(dir.join("other.json"), "{}").unwrap();
    let broken = arrange(&mut diagrams(), &model, Some(&dir.join("other.json")))
        .expect_err("not a layout");
    assert!(broken.contains("other.json") && broken.contains("arclang_layout"), "got: {broken}");
    std::fs::remove_dir_all(dir).ok();
}

#[test]
fn everything_left_out_of_a_layout_reaches_the_viewer() {
    let stale = r#"{ "arclang_layout": "1", "views": { "pab": { "folded": [], "open": null }, "lab": { "folded": ["LC-9"], "open": null } } }"#;
    let mut set = diagrams();
    let warnings = set.apply_layout(LayoutFile::parse(stale).unwrap());

    for warning in &warnings {
        assert!(set.diagnostics.contains(warning), "`{warning}` is only on the command line");
    }
    assert!(set.diagnostics.iter().any(|d| d.starts_with("[layout] ")));
}

#[test]
fn two_arrangements_for_one_layout_are_reported_and_one_is_kept() {
    // LF-1 is not a container: once it is left out, both arrangements are
    // for the same layout.
    let twice = r#"{
      "arclang_layout": "1",
      "arrangements": [
        { "view": "lab", "open": null, "folded": ["LC-2", "LF-1"], "places": { "LC-1": { "dx": 40, "dy": 0 } } },
        { "view": "lab", "open": null, "folded": ["LC-2"], "places": { "LC-1": { "dx": 99, "dy": 0 } } }
      ]
    }"#;
    let mut set = diagrams();
    let warnings = set.apply_layout(LayoutFile::parse(twice).unwrap());

    assert_eq!(set.layout.as_ref().unwrap().arrangements.len(), 1);
    assert!(
        warnings
            .iter()
            .any(|w| w == "[lab] layout file: two arrangements are for the same layout — only the first is kept"),
        "got {warnings:?}"
    );
}

#[test]
fn the_open_container_is_not_part_of_what_is_folded_around_it() {
    let layout = r#"{
      "arclang_layout": "1",
      "arrangements": [
        { "view": "lab", "open": "LC-1", "folded": ["LC-1", "LC-2"], "places": { "LF-1": { "dx": 5, "dy": 5 } } }
      ]
    }"#;
    let mut set = diagrams();
    let warnings = set.apply_layout(LayoutFile::parse(layout).unwrap());

    assert!(warnings.is_empty(), "got {warnings:?}");
    assert_eq!(set.layout.as_ref().unwrap().arrangements[0].folded, vec!["LC-2"]);
}

#[test]
fn a_resized_box_keeps_its_size_in_the_file_and_an_absurd_size_is_refused() {
    let resized = r#"{ "arclang_layout": "1", "arrangements": [ { "view": "lab", "places": { "LC-1": { "dx": 0, "dy": 0, "dw": 60, "dh": 24.5 } } } ] }"#;
    let layout = LayoutFile::parse(resized).expect("a resize is part of a placement");
    let text = layout.to_json();
    assert!(text.contains("\"dw\": 60.0") && text.contains("\"dh\": 24.5"), "{text}");
    // A placement without a resize writes no size.
    assert!(!LayoutFile::parse(LAYOUT).unwrap().to_json().contains("\"dw\""));
    let absurd = resized.replace("\"dw\": 60", "\"dw\": 1e9");
    assert!(LayoutFile::parse(&absurd).unwrap_err().contains("resized beyond"));
}

#[test]
fn a_bent_exchange_keeps_its_waypoints_and_a_stale_one_is_reported() {
    let model = r#"
logical_architecture "LA" {
    component "A" { id: "LC-A" port out "p" }
    component "B" { id: "LC-B" port in "q" }
    component_exchange "Flow" { from_port: "LC-A.p" to_port: "LC-B.q" }
}
"#;
    let result = Compiler::new(CompilerConfig::default()).compile_string(model).expect("compiles");
    let mut set = build_diagrams(&result.ast);
    let layout = r#"{
  "arclang_layout": "1",
  "arrangements": [
    { "view": "lab", "edges": { "ce:Flow": { "points": [{ "x": 120, "y": 40 }] }, "ce:Gone": { "points": [{ "x": 1, "y": 2 }] } } }
  ]
}"#;
    let parsed = LayoutFile::parse(layout).expect("parses");
    assert_eq!(LayoutFile::parse(&parsed.to_json()).expect("round trip"), parsed);
    let warnings = set.apply_layout(parsed);
    assert!(warnings.iter().any(|w| w.contains("ce:Gone")), "{warnings:?}");
    let kept = set.layout.as_ref().expect("kept");
    assert_eq!(kept.arrangements[0].edges["ce:Flow"].points.len(), 1);
    assert!(!kept.arrangements[0].edges.contains_key("ce:Gone"));
    let absurd = layout.replace("120", "1e9");
    assert!(LayoutFile::parse(&absurd).unwrap_err().contains("bends farther"));
}

#[test]
fn line_styles_names_ports_and_stacking_are_kept_and_what_is_stale_is_reported() {
    let model = r#"
logical_architecture "LA" {
    component "A" { id: "LC-A" port out "p" }
    component "B" { id: "LC-B" port in "q" }
    component_exchange "Flow" { from_port: "LC-A.p" to_port: "LC-B.q" }
}
"#;
    let result = Compiler::new(CompilerConfig::default()).compile_string(model).expect("compiles");
    let mut set = build_diagrams(&result.ast);
    let lab = set.diagrams.iter().find(|d| d.id == "lab").expect("logical view");
    let port = lab.nodes.iter().find(|n| n.id == "LC-A").expect("A").ports[0].id.clone();
    let layout = format!(
        r#"{{
  "arclang_layout": "1",
  "arrangements": [
    {{ "view": "lab",
       "edges": {{ "ce:Flow": {{ "style": "curved", "label": {{ "dx": 12, "dy": -8 }} }}, "ce:Odd": {{ "style": "zigzag" }} }},
       "ports": {{ "{port}": {{ "dx": 0, "dy": 24 }}, "LC-A::gone": {{ "dx": 1, "dy": 1 }} }},
       "order": {{ "LC-B": 1, "LC-A": -1, "LC-Gone": 2 }} }}
  ]
}}"#
    );
    let parsed = LayoutFile::parse(&layout).expect("parses");
    let text = parsed.to_json();
    assert_eq!(LayoutFile::parse(&text).expect("round trip"), parsed);
    assert!(!text.contains("\"points\""), "an exchange that is not bent writes no points: {text}");
    let warnings = set.apply_layout(parsed);
    for stale in ["ce:Odd", "LC-A::gone", "LC-Gone"] {
        assert!(warnings.iter().any(|w| w.contains(stale)), "{stale} reported: {warnings:?}");
    }
    let kept = &set.layout.as_ref().expect("kept").arrangements[0];
    assert_eq!(kept.edges["ce:Flow"].style.as_deref(), Some("curved"));
    assert_eq!(kept.edges["ce:Flow"].label.map(|l| (l.dx, l.dy)), Some((12.0, -8.0)));
    assert_eq!(kept.ports.keys().collect::<Vec<_>>(), vec![&port]);
    assert_eq!(kept.order.get("LC-B"), Some(&1));
    assert_eq!(kept.order.get("LC-A"), Some(&-1));
    assert!(!kept.order.contains_key("LC-Gone"));
    // The standalone page receives all of it.
    let page = html::standalone_html(&set, "LA");
    assert!(page.contains("\"style\":\"curved\"") && page.contains("\"order\""), "the page carries the arrangement");
    // Absurd values are refused with the reason.
    let far = layout.replace("\"dy\": 24", "\"dy\": 1e9");
    assert!(LayoutFile::parse(&far).unwrap_err().contains("moved farther"));
    let deep = layout.replace("\"LC-B\": 1", "\"LC-B\": 1000000");
    assert!(LayoutFile::parse(&deep).unwrap_err().contains("stacked deeper"));
}

#[test]
fn a_file_without_the_new_arrangements_is_written_as_before() {
    // Styles, names, ports and stacking are written only when used: a file
    // made without them keeps the shape earlier builds read.
    let text = LayoutFile::parse(LAYOUT).unwrap().to_json();
    for key in ["\"ports\"", "\"order\"", "\"style\"", "\"label\""] {
        assert!(!text.contains(key), "{key} in {text}");
    }
}
