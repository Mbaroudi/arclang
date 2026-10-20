//! Live tables and matrices (`table` declarations, `arclang table`): each
//! kind computed on a model with gaps on purpose, the filter, the exports
//! (CSV golden, TSV, Markdown, HTML, Excel), determinism, the table as an
//! element of the model, and its SysML v2 round trip.
//! Regenerate the goldens with `UPDATE_GOLDEN=1 cargo test --test tables`.

use arclang::compiler::elements;
use arclang::compiler::tables::{self, export, xlsx, Options, Table, TableSpec};
use arclang::compiler::{sysml_import, sysml_parser, sysmlv2_generator, Compiler, CompilerConfig};
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tables/braking.arc")
}

fn compile(source: &str) -> arclang::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles")
}

fn model() -> arclang::CompilationResult {
    compile(&std::fs::read_to_string(fixture()).unwrap())
}

fn declared(result: &arclang::CompilationResult, name: &str) -> Table {
    let decl = result.ast.tables.iter().find(|t| t.name == name).unwrap_or_else(|| panic!("no table {name}"));
    tables::compute(result, &TableSpec::of(decl), &Options::default()).unwrap()
}

fn row<'t>(table: &'t Table, id: &str) -> &'t tables::Row {
    table.rows.iter().find(|r| r.id == id).unwrap_or_else(|| panic!("no row {id} in {}", table.name))
}

#[test]
fn a_table_declaration_is_parsed_and_checked() {
    let result = model();
    assert_eq!(result.ast.tables.len(), 7);
    let sat = result.ast.tables.iter().find(|t| t.name == "Satisfaction").unwrap();
    assert_eq!((sat.id.as_str(), sat.kind.as_str()), ("TBL-SAT", "traceability_matrix"));
    let spec = TableSpec::of(sat);
    assert_eq!(spec.rows, vec!["Requirement"]);
    assert_eq!(spec.matrix_columns, vec!["LogicalComponent"], "for a matrix, `columns` names its columns");
    let no_kind = Compiler::new(CompilerConfig::default()).compile_string("model M {}\ntable \"T\" { scope: \"SA\" }\n");
    assert!(no_kind.is_err_and(|e| e.to_string().contains("needs `kind:`")));
    let unknown = Compiler::new(CompilerConfig::default()).compile_string("model M {}\ntable \"T\" { kind: pivot }\n");
    assert!(unknown.is_err_and(|e| e.to_string().contains("unknown kind 'pivot'")));
    // `table` stays an attribute key and a name elsewhere.
    compile("model M {}\nlogical_architecture \"LA\" { component \"table\" { table: \"wood\" } }\n");
}

#[test]
fn a_declared_table_is_an_element_of_the_model_with_a_stable_uuid() {
    let a = model();
    let graph = elements::build(&a.ast, &a.semantic_model);
    let tables_in_graph: Vec<_> = graph.elements.iter().filter(|e| e.kind == "Table").collect();
    assert_eq!(tables_in_graph.len(), 7);
    let sat = tables_in_graph.iter().find(|e| e.id == "TBL-SAT").unwrap();
    assert_eq!(sat.name, "Satisfaction");
    let again = model();
    let again = elements::build(&again.ast, &again.semantic_model);
    assert_eq!(again.elements.iter().find(|e| e.id == "TBL-SAT").unwrap().uuid, sat.uuid, "same declaration, same uuid");
    let table = declared(&a, "Satisfaction");
    assert_eq!(table.uuid.as_deref(), Some(sat.uuid.as_str()));
    assert!(table.declared);
}

#[test]
fn requirements_coverage_shows_satisfaction_verdicts_and_the_gate() {
    let result = model();
    let table = declared(&result, "Requirement coverage");
    let keys: Vec<&str> = table.columns.iter().map(|c| c.key.as_str()).collect();
    assert_eq!(keys, ["id", "name", "text", "priority", "satisfied_by", "verified_by", "verdict", "gate", "status"]);

    let covered = row(&table, "REQ-1");
    assert_eq!(covered.status, "ok");
    let satisfied = &covered.cells["satisfied_by"];
    assert_eq!(satisfied.links.iter().map(|l| l.id.as_str()).collect::<Vec<_>>(), ["LC-RADAR"]);
    assert!(!satisfied.links[0].uuid.is_empty(), "links carry uuids");
    assert_eq!(covered.cells["verified_by"].links[0].verdict.as_deref(), Some("pass"));
    assert_eq!(covered.cells["gate"].text, "pass");

    let failing = row(&table, "REQ-2");
    assert_eq!(failing.status, "fail");
    assert_eq!(failing.cells["verdict"].status.as_deref(), Some("fail"));
    assert!(failing.issues.iter().any(|i| i.contains("Timing test fails")), "{:?}", failing.issues);
    assert_eq!(failing.cells["gate"].status.as_deref(), Some("fail"), "the gate blocks on a failing verification");

    let pending = row(&table, "REQ-3");
    assert_eq!((pending.status.as_str(), pending.cells["verdict"].text.as_str()), ("warn", "pending"));

    let open = row(&table, "REQ-4");
    assert_eq!(open.status, "gap");
    assert_eq!(open.cells["status"].text, "Not covered");
    assert_eq!(open.cells["satisfied_by"].status.as_deref(), Some("gap"));
    assert_eq!(open.cells["gate"].text, "2 blockers");

    assert_eq!((table.summary.ok, table.summary.warn, table.summary.gap, table.summary.fail), (1, 1, 1, 1));
    assert!(!table.summary.gate_passed);
    assert_eq!(table.summary.gate_blockers, 3);
}

#[test]
fn coverage_cells_say_how_they_are_edited() {
    let result = model();
    let table = declared(&result, "Requirement coverage");
    let r = row(&table, "REQ-1");
    let text = r.cells["text"].edit.as_ref().expect("the text is editable");
    assert_eq!((text.op.as_str(), text.element.as_str(), text.key.as_deref()), ("set", "REQ-1", Some("description")));
    let name = r.cells["name"].edit.as_ref().unwrap();
    assert_eq!(name.key.as_deref(), Some("title"), "a requirement's name is its title");
    let priority = r.cells["priority"].edit.as_ref().unwrap();
    assert_eq!(priority.value_type.as_deref(), Some("Enum"));
    assert!(priority.values.contains(&"Critical".to_string()));
    assert!(r.cells["satisfied_by"].edit.is_none(), "computed cells are not edited");
}

#[test]
fn a_traceability_matrix_holds_the_trace_kinds_and_says_which_way_a_click_goes() {
    let result = model();
    let table = declared(&result, "Satisfaction");
    let m = table.matrix.as_ref().unwrap();
    assert_eq!(m.rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["REQ-1", "REQ-2", "REQ-3", "REQ-4"]);
    assert_eq!(m.columns.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["LC-RADAR", "LC-CTRL", "LC-BRAKE", "LC-LOG"]);
    let cell = m.cells.iter().find(|c| c.row == 1 && c.column == 2).expect("REQ-2 x LC-BRAKE");
    assert_eq!(cell.kinds, ["satisfies"]);
    assert_eq!((cell.links[0].source.as_str(), cell.links[0].from.as_str(), cell.links[0].to.as_str()), ("trace", "LC-BRAKE", "REQ-2"));
    assert_eq!(m.empty_rows, [3], "REQ-4 is traced to nothing");
    assert_eq!(m.empty_columns, [3], "the logger satisfies nothing");
    assert_eq!(m.creatable, ["satisfies"]);
    assert_eq!(m.direction["satisfies"], "column_to_row", "a component satisfies a requirement");
    assert_eq!(row(&table, "REQ-4").status, "gap");
    // Without `columns`, the elements the rows are traced with; every kind by default.
    let spec = TableSpec { kind: "traceability_matrix".into(), ..TableSpec::default() };
    let open = tables::compute(&result, &spec, &Options::default()).unwrap();
    let m = open.matrix.unwrap();
    let columns: Vec<&str> = m.columns.iter().map(|c| c.id.as_str()).collect();
    assert!(columns.contains(&"TC-1") && columns.contains(&"LC-CTRL") && !columns.contains(&"LC-LOG"), "{columns:?}");
    assert!(m.cells.iter().any(|c| c.kinds == ["verifies"] && c.links[0].source == "verifies"));
}

#[test]
fn an_allocation_matrix_shows_what_performs_each_function_and_what_nothing_does() {
    let result = model();
    let table = declared(&result, "Allocation");
    let m = table.matrix.as_ref().unwrap();
    assert_eq!(m.rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["LF-SENSE", "LF-DECIDE", "LF-ACT", "LF-ARCHIVE"]);
    assert!(m.cells.iter().any(|c| c.row == 0 && c.column == 0 && c.kinds == ["allocated"]));
    assert_eq!(m.empty_rows, [3], "Archive is allocated to nothing");
    assert_eq!(row(&table, "LF-ARCHIVE").issues, ["allocated to nothing"]);
    assert_eq!((m.creatable.as_slice(), m.direction["allocates"].as_str()), (["allocates".to_string()].as_slice(), "column_to_row"));
    // At the physical level, a node hosts what the components deployed on it perform.
    let spec = TableSpec { kind: "allocation_matrix".into(), scope: Some("PA".into()), rows: vec!["LogicalFunction".into()], matrix_columns: vec!["PhysicalNode".into()], ..TableSpec::default() };
    let pa = tables::compute(&result, &spec, &Options::default()).unwrap();
    let m = pa.matrix.unwrap();
    assert!(m.cells.iter().any(|c| m.rows[c.row].id == "LF-DECIDE" && m.columns[c.column].id == "PN-BRAKE" && c.kinds == ["deployed"]), "{:?}", m.cells);
}

#[test]
fn an_n2_chart_puts_the_exchanges_between_their_components() {
    let result = model();
    let table = declared(&result, "Interfaces");
    let m = table.matrix.as_ref().unwrap();
    assert!(m.diagonal);
    let at = |from: &str, to: &str| m.cells.iter().find(|c| m.rows[c.row].id == from && m.columns[c.column].id == to);
    assert_eq!(at("LC-RADAR", "LC-CTRL").unwrap().items, ["Targets"]);
    assert_eq!(at("LC-CTRL", "LC-BRAKE").unwrap().items, ["Command"]);
    assert!(at("LC-CTRL", "LC-RADAR").is_none(), "an N² chart is oriented");
    assert_eq!(row(&table, "LC-LOG").status, "gap", "a component exchanging nothing");
    assert!(m.creatable.is_empty());
}

#[test]
fn an_icd_takes_the_protocol_where_it_is_written() {
    let result = model();
    let table = declared(&result, "ICD");
    let targets = row(&table, "Targets");
    assert_eq!(targets.cells["from"].links[0].id, "LC-RADAR");
    assert_eq!(targets.cells["from_port"].links[0].id, "LC-RADAR.targets");
    assert_eq!(targets.cells["items"].links[0].id, "EI-TARGETS");
    let protocol = &targets.cells["protocol"];
    assert_eq!(protocol.text, "CAN FD");
    assert_eq!(protocol.edit.as_ref().unwrap().element, "LC-RADAR.targets", "edited on the port that states it");
    assert_eq!(targets.cells["latency"].text, "10 ms");
    assert_eq!(targets.cells["latency"].edit.as_ref().unwrap().element, "Targets");
    assert!(!targets.uuid.is_empty());
    let command = row(&table, "Command");
    assert_eq!(command.status, "gap", "carries no exchange item");
    // The physical level: the link gives protocol and bandwidth.
    let spec = TableSpec { kind: "icd".into(), scope: Some("PA".into()), ..TableSpec::default() };
    let pa = tables::compute(&result, &spec, &Options::default()).unwrap();
    let frames = row(&pa, "Frames");
    assert_eq!((frames.cells["protocol"].text.as_str(), frames.cells["bandwidth"].text.as_str()), ("CAN FD", "5 Mbps"));
    assert_eq!(frames.cells["protocol"].edit.as_ref().unwrap().element, "PL-CAN");
}

#[test]
fn an_elements_table_picks_its_columns_among_the_attributes() {
    let result = model();
    let table = declared(&result, "Components");
    assert_eq!(table.rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["LC-RADAR", "LC-CTRL", "LC-BRAKE", "LC-LOG"]);
    assert_eq!(table.columns.iter().map(|c| c.key.as_str()).collect::<Vec<_>>(), ["id", "name", "description"]);
    let available: Vec<&str> = table.available_columns.iter().map(|c| c.key.as_str()).collect();
    assert!(available.contains(&"safety_level") && available.contains(&"owner"), "{available:?}");
    let edit = row(&table, "LC-RADAR").cells["description"].edit.as_ref().unwrap();
    assert_eq!((edit.op.as_str(), edit.key.as_deref()), ("set", Some("description")));
    assert_eq!(row(&table, "LC-RADAR").cells["name"].edit.as_ref().unwrap().op, "rename");
    let unknown = TableSpec { kind: "elements".into(), row_type: Some("Gizmo".into()), ..TableSpec::default() };
    assert!(tables::compute(&result, &unknown, &Options::default()).unwrap_err().contains("no kind of the metamodel"));
    let bad_scope = TableSpec { kind: "elements".into(), scope: Some("Nowhere".into()), ..TableSpec::default() };
    assert!(tables::compute(&result, &bad_scope, &Options::default()).unwrap_err().contains("scope 'Nowhere'"));
    // Scope by element: what is under it.
    let ports = TableSpec { kind: "elements".into(), row_type: Some("ComponentPort".into()), scope: Some("LC-CTRL".into()), ..TableSpec::default() };
    let ports = tables::compute(&result, &ports, &Options::default()).unwrap();
    assert_eq!(ports.rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["LC-CTRL.targets", "LC-CTRL.command"]);
}

#[test]
fn the_filter_keeps_the_rows_asked_for() {
    let result = model();
    let open = declared(&result, "Open requirements");
    assert_eq!(open.rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["REQ-2", "REQ-4"]);
    let spec = |filter: &str| TableSpec { kind: "requirements_coverage".into(), filter: Some(filter.into()), ..TableSpec::default() };
    let ids = |filter: &str| tables::compute(&result, &spec(filter), &Options::default()).unwrap().rows.iter().map(|r| r.id.clone()).collect::<Vec<_>>();
    assert_eq!(ids("priority=high"), ["REQ-1", "REQ-2"]);
    assert_eq!(ids("satisfied_by:LC-CTRL -is:fail"), ["REQ-3"]);
    assert_eq!(ids("\"within 100\""), ["REQ-2"]);
    let gaps = TableSpec { kind: "requirements_coverage".into(), gaps_only: true, ..TableSpec::default() };
    assert_eq!(tables::compute(&result, &gaps, &Options::default()).unwrap().rows.len(), 2, "a pending verdict is no gap");
}

#[test]
fn the_same_model_gives_the_same_bytes() {
    let a = tables::declared_tables(&model(), &Options::default()).unwrap();
    let b = tables::declared_tables(&model(), &Options::default()).unwrap();
    assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
    assert_eq!(xlsx::workbook(&a), xlsx::workbook(&b));
    assert_eq!(export::html(&a, "t"), export::html(&b, "t"));
}

#[test]
fn csv_tsv_markdown_and_html_render_the_grid() {
    let result = model();
    let coverage = declared(&result, "Requirement coverage");
    let csv = export::csv(&coverage);
    let first = csv.lines().next().unwrap();
    assert_eq!(first, "Req id,Name,Requirement text,Priority,Satisfied by,Verified by,Verdict,Gate,Status");
    assert!(csv.contains("REQ-2,Brake in time,Brake within 100 ms,High,\"Controller, Brake\",Timing test (fail),fail,1 blocker,Failing"), "{csv}");
    let tsv = export::tsv(&coverage);
    assert!(tsv.lines().nth(1).unwrap().split('\t').count() == 9);
    let md = export::markdown(&declared(&result, "Satisfaction"));
    assert!(md.contains("| Brake in time |  | satisfies | satisfies |  |"), "{md}");
    let html = export::html(&[coverage], "Braking");
    assert!(html.contains("<tr id=\"REQ-4\" class=\"gap\">") && html.contains("prefers-color-scheme: dark"));
}

/// The bytes of one stored entry of a zip (the workbook stores without compression).
fn entry<'a>(zip: &'a [u8], name: &str) -> &'a str {
    let at = (30..zip.len().saturating_sub(name.len()))
        .find(|&at| &zip[at..at + name.len()] == name.as_bytes() && zip[at - 30..at - 26] == [0x50, 0x4b, 0x03, 0x04])
        .unwrap_or_else(|| panic!("no {name}"));
    let header = at - 30;
    let size = u32::from_le_bytes(zip[header + 18..header + 22].try_into().unwrap()) as usize;
    std::str::from_utf8(&zip[at + name.len()..at + name.len() + size]).unwrap()
}

#[test]
fn the_excel_workbook_has_one_sheet_per_table_with_its_colours() {
    let all = tables::declared_tables(&model(), &Options::default()).unwrap();
    let book = xlsx::workbook(&all);
    let workbook = entry(&book, "xl/workbook.xml");
    assert!(workbook.contains("<sheet name=\"Requirement coverage\" sheetId=\"1\"") && workbook.contains("<sheet name=\"Components\" sheetId=\"7\""), "{workbook}");
    let sheet = entry(&book, "xl/worksheets/sheet1.xml");
    assert!(sheet.contains("state=\"frozen\"") && sheet.contains("<autoFilter ref=\"A1:I5\"/>"));
    assert!(sheet.contains("<t xml:space=\"preserve\">Not covered</t>"));
    // The status cell of the failing requirement carries the red style (4).
    assert!(sheet.contains("<c r=\"I3\" s=\"4\" t=\"inlineStr\"><is><t xml:space=\"preserve\">Failing</t>"), "{sheet}");
    let matrix = entry(&book, "xl/worksheets/sheet2.xml");
    assert!(matrix.contains("xSplit=\"1\" ySplit=\"1\""));
    assert!(entry(&book, "[Content_Types].xml").contains("/xl/worksheets/sheet7.xml"));
}

#[test]
fn a_table_spec_writes_its_declaration_back() {
    let result = model();
    for decl in &result.ast.tables {
        let spec = TableSpec::of(decl);
        let text = format!("model M {{}}\n{}\n", spec.declaration());
        let again = compile(&text);
        assert_eq!(TableSpec::of(&again.ast.tables[0]), spec, "{text}");
    }
}

#[test]
fn tables_go_to_sysml_v2_as_element_table_views_and_come_back() {
    let before = model();
    let text = sysmlv2_generator::generate_sysmlv2(&before.semantic_model, &before.ast);
    assert!(text.contains("view <'TBL-SAT'> Satisfaction {"), "{text}");
    assert!(text.contains("attribute table : String = \"traceability_matrix\";"), "{text}");
    assert!(text.contains("render Views::asElementTable;"));
    let built = sysml_parser::records(&text).expect("the export parses");
    let imported = sysml_import::import_records(&built.records).unwrap_or_else(|f| panic!("{}", f.message));
    let after = compile(&imported.source);
    let specs = |r: &arclang::CompilationResult| r.ast.tables.iter().map(|t| (t.name.clone(), t.id.clone(), TableSpec::of(t))).collect::<Vec<_>>();
    assert_eq!(specs(&after), specs(&before), "{}", imported.source);
    let description = after.ast.tables.iter().find(|t| t.name == "Open requirements").unwrap().attributes.get("description").and_then(|v| v.as_string()).map(str::to_string);
    assert_eq!(description.as_deref(), Some("What the gate still blocks on"));
}

#[test]
fn the_flagship_with_its_tables_still_comes_back_whole_from_sysml() {
    let flagship = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/complete_emergency_braking_simple.arc")).unwrap();
    let source = format!("{flagship}\ntable \"Requirement coverage\" {{ kind: requirements_coverage scope: \"SA\" }}\ntable \"ICD\" {{ id: \"TBL-ICD\" kind: icd scope: \"LA\" columns: [\"name\", \"from\", \"to\", \"protocol\"] }}\n");
    let before = compile(&source);
    let text = sysmlv2_generator::generate_sysmlv2(&before.semantic_model, &before.ast);
    let built = sysml_parser::records(&text).unwrap();
    let imported = sysml_import::import_records(&built.records).unwrap_or_else(|f| panic!("{}", f.message));
    let after = compile(&imported.source);
    let ids = |r: &arclang::CompilationResult| {
        let g = elements::build(&r.ast, &r.semantic_model);
        g.elements.iter().map(|e| (e.kind.to_string(), e.id.clone())).collect::<std::collections::BTreeSet<_>>()
    };
    let (a, b) = (ids(&before), ids(&after));
    let lost: Vec<_> = a.difference(&b).collect();
    assert!(lost.is_empty(), "lost {lost:?}");
    assert!(a.contains(&("Table".to_string(), "TBL-ICD".to_string())));
}

#[test]
fn the_cli_writes_every_format_and_the_csv_golden_holds() {
    let bin = env!("CARGO_BIN_EXE_arclang");
    let run = |args: &[&str]| {
        let out = std::process::Command::new(bin).arg("table").arg(fixture()).args(args).output().unwrap();
        assert!(out.status.success(), "{args:?}: {}", String::from_utf8_lossy(&out.stderr));
        out.stdout
    };
    let golden_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tables");
    for (name, args) in [
        ("coverage.csv", vec!["--name", "Requirement coverage", "-f", "csv"]),
        ("satisfaction.csv", vec!["--name", "TBL-SAT", "-f", "csv"]),
        ("icd.md", vec!["--name", "ICD", "-f", "md"]),
        ("gaps.tsv", vec!["--kind", "requirements_coverage", "--gaps", "-f", "tsv"]),
    ] {
        let got = String::from_utf8(run(&args)).unwrap();
        let path = golden_dir.join(name);
        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&path, &got).unwrap();
        }
        let want = std::fs::read_to_string(&path).unwrap_or_default();
        assert_eq!(got, want, "{name} changed: review, then UPDATE_GOLDEN=1 cargo test --test tables");
    }
    let json: serde_json::Value = serde_json::from_slice(&run(&["--name", "Requirement coverage"])).unwrap();
    assert_eq!(json["rows"][0]["cells"]["satisfied_by"]["links"][0]["id"], "LC-RADAR");
    assert!(json["rows"][0]["uuid"].as_str().is_some_and(|u| u.len() == 36));
    let all: serde_json::Value = serde_json::from_slice(&run(&[])).unwrap();
    assert_eq!(all.as_array().map(Vec::len), Some(7));
    let dir = tempfile::tempdir().unwrap();
    let book = dir.path().join("t.xlsx");
    run(&["-f", "xlsx", "-o", book.to_str().unwrap()]);
    assert_eq!(&std::fs::read(&book).unwrap()[0..4], b"PK\x03\x04");
    let declaration = String::from_utf8(run(&["--kind", "icd", "--scope", "PA", "--name", "Physical ICD", "--declaration"])).unwrap();
    assert_eq!(declaration, "table \"Physical ICD\" {\n    kind: icd\n    scope: \"PA\"\n}\n");
    let refused = std::process::Command::new(bin).arg("table").arg(fixture()).args(["-f", "csv"]).output().unwrap();
    assert!(!refused.status.success() && String::from_utf8_lossy(&refused.stderr).contains("pick one with --name"));
}
