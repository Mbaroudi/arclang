//! The requirements table as a tree: a requirement under another
//! (`parent:`) is a row under its row, in every format.

use arclang::compiler::tables::{self, export, Options, TableSpec};
use arclang::compiler::{Compiler, CompilerConfig};

const MODEL: &str = r#"
model "Car" {}
requirements {
    requirement "Safety" { id: "REQ-1" description: "The car is safe" }
    requirement "Range" { id: "REQ-2" description: "The car drives 400 km" }
    requirement "Braking" { id: "REQ-1.1" description: "Stops within 40 m" parent: "REQ-1" }
    requirement "ABS" { id: "REQ-1.1.1" description: "Wheels do not lock" parent: "Braking" }
    requirement "Airbags" { id: "REQ-1.2" description: "Six airbags" parent: "REQ-1" }
}
"#;

fn table(text: &str) -> tables::Table {
    let result = Compiler::new(CompilerConfig::default()).compile_string(text).unwrap();
    let spec = TableSpec { kind: "requirements_coverage".to_string(), ..TableSpec::default() };
    tables::compute(&result, &spec, &Options::default()).unwrap()
}

#[test]
fn each_requirement_follows_its_parent_one_level_deeper() {
    let table = table(MODEL);
    let tree: Vec<(&str, usize, Option<&str>)> = table.rows.iter().map(|r| (r.id.as_str(), r.depth, r.parent.as_deref())).collect();
    assert_eq!(
        tree,
        [("REQ-1", 0, None), ("REQ-1.1", 1, Some("REQ-1")), ("REQ-1.1.1", 2, Some("REQ-1.1")), ("REQ-1.2", 1, Some("REQ-1")), ("REQ-2", 0, None)]
    );
}

#[test]
fn a_parent_that_loops_leaves_the_rows_at_the_top() {
    let looped = MODEL.replace(r#"description: "The car is safe""#, r#"description: "The car is safe" parent: "REQ-1.1.1""#);
    let table = table(&looped);
    assert_eq!(table.rows.len(), 5, "no row lost");
    assert!(table.rows.iter().any(|r| r.depth == 0 && r.id == "REQ-2"));
}

#[test]
fn exports_say_the_depth() {
    let table = table(MODEL);
    let csv = export::csv(&table);
    assert!(csv.contains("\r\n    REQ-1.1.1,"), "{csv}");
    // The workbook is stored, not compressed: its sheet reads as text.
    let workbook = String::from_utf8_lossy(&tables::xlsx::workbook(&[table])).to_string();
    assert!(workbook.contains(r#"<row r="4" outlineLevel="2">"#), "Excel groups the rows under their parent");
}

#[test]
fn the_json_carries_depth_and_parent_only_where_nested() {
    let table = table(MODEL);
    let json = serde_json::to_value(&table.rows).unwrap();
    assert_eq!(json[0].get("depth"), None);
    assert_eq!(json[1]["depth"], 1);
    assert_eq!(json[1]["parent"], "REQ-1");
}
