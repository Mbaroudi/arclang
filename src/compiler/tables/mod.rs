//! Live tables and matrices: views of a model computed by the compiler on
//! every compile, never stored values that drift from the model.
//!
//! A `table "Name" { kind: requirements_coverage ... }` declaration is a
//! model element (kind `Table` of the element graph, with its UUID): it is
//! versioned with the model text, exported to SysML v2 as a `view` rendered
//! `asElementTable`, and computed again whenever the model changes. The
//! same computation answers `arclang table` (JSON, CSV, TSV, Markdown,
//! HTML, Excel) and the table view of the VS Code workbench.
//!
//! Kinds:
//! - `requirements_coverage`: per requirement, its text, what satisfies it,
//!   the verification cases with their computed verdict, and the gate;
//! - `traceability_matrix`: rows x columns of elements, the trace kinds in
//!   the cells (`rows`, `columns`, `kinds`);
//! - `allocation_matrix`: functions x components or nodes;
//! - `interface_n2`: the N² chart of components, exchanges in the cells;
//! - `icd`: one row per exchange: ends, ports, items, protocol, bandwidth,
//!   latency;
//! - `elements`: any kind, the columns picked among its attributes.

mod context;
mod coverage;
mod elements_table;
pub mod export;
pub mod filter;
mod icd;
mod matrix;
pub mod xlsx;

use crate::compiler::ast::AttributeValue;
use crate::compiler::CompilationResult;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

pub use context::Context;

/// The kinds of table the compiler computes, as written in `kind:`.
pub const KINDS: [&str; 6] = ["requirements_coverage", "traceability_matrix", "allocation_matrix", "interface_n2", "icd", "elements"];

/// `table "Name" { kind: requirements_coverage scope: "SA" ... }` as written.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDecl {
    pub name: String,
    /// The written `id`, else the name.
    pub id: String,
    pub kind: String,
    pub attributes: HashMap<String, AttributeValue>,
}

/// What to compute: a declared table, or one asked for on the command line
/// or from the workbench.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TableSpec {
    pub name: Option<String>,
    pub id: Option<String>,
    pub kind: String,
    /// A layer (`OA`, `SA`, `LA`, `PA`, `EPBS`) or an element: what the rows are taken from.
    #[serde(default)]
    pub scope: Option<String>,
    /// The kind of the rows (`Requirement`, `LogicalComponent`...), for `elements`.
    #[serde(default)]
    pub row_type: Option<String>,
    /// The columns shown, in order; empty for the kind's default.
    #[serde(default)]
    pub columns: Vec<String>,
    /// For a matrix: what its rows and its columns are (kinds or layers).
    #[serde(default)]
    pub rows: Vec<String>,
    #[serde(default)]
    pub matrix_columns: Vec<String>,
    /// For a traceability matrix: the trace kinds shown.
    #[serde(default)]
    pub kinds: Vec<String>,
    /// A filter (see [`filter`]): `priority:High`, `is:gap`, free words.
    #[serde(default)]
    pub filter: Option<String>,
    /// Keep only the rows with a gap.
    #[serde(default)]
    pub gaps_only: bool,
}

fn texts(value: Option<&AttributeValue>) -> Vec<String> {
    match value {
        Some(AttributeValue::List(items)) => items.iter().filter_map(|v| v.as_string()).map(str::to_string).collect(),
        Some(AttributeValue::String(one)) => one.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        _ => Vec::new(),
    }
}

impl TableSpec {
    /// The spec a declaration states. For a matrix `columns:` names the
    /// column elements; for a table, the columns shown.
    pub fn of(decl: &TableDecl) -> TableSpec {
        let text = |key: &str| decl.attributes.get(key).and_then(|v| v.as_string()).map(str::to_string);
        let matrix = is_matrix(&decl.kind);
        let columns = texts(decl.attributes.get("columns"));
        TableSpec {
            name: Some(decl.name.clone()),
            id: Some(decl.id.clone()),
            kind: decl.kind.clone(),
            scope: text("scope"),
            row_type: text("row_type"),
            columns: if matrix { Vec::new() } else { columns.clone() },
            rows: texts(decl.attributes.get("rows")),
            matrix_columns: if matrix { columns } else { Vec::new() },
            kinds: texts(decl.attributes.get("kinds")),
            filter: text("filter"),
            gaps_only: matches!(decl.attributes.get("gaps_only"), Some(AttributeValue::Boolean(true))),
        }
    }

    /// The `table` declaration that states this spec, as ArcLang source.
    pub fn declaration(&self) -> String {
        let list = |items: &[String]| format!("[{}]", items.iter().map(|i| quoted(i)).collect::<Vec<_>>().join(", "));
        let mut lines = vec![format!("kind: {}", self.kind)];
        if let Some(id) = self.id.as_ref().filter(|id| Some(*id) != self.name.as_ref()) {
            lines.push(format!("id: {}", quoted(id)));
        }
        if let Some(scope) = &self.scope {
            lines.push(format!("scope: {}", quoted(scope)));
        }
        if let Some(row_type) = &self.row_type {
            lines.push(format!("row_type: {}", quoted(row_type)));
        }
        if !self.rows.is_empty() {
            lines.push(format!("rows: {}", list(&self.rows)));
        }
        let columns = if is_matrix(&self.kind) { &self.matrix_columns } else { &self.columns };
        if !columns.is_empty() {
            lines.push(format!("columns: {}", list(columns)));
        }
        if !self.kinds.is_empty() {
            lines.push(format!("kinds: {}", list(&self.kinds)));
        }
        if let Some(filter) = self.filter.as_ref().filter(|f| !f.trim().is_empty()) {
            lines.push(format!("filter: {}", quoted(filter)));
        }
        if self.gaps_only {
            lines.push("gaps_only: true".to_string());
        }
        let name = self.name.clone().unwrap_or_else(|| default_title(&self.kind).to_string());
        let body: String = lines.iter().map(|l| format!("    {l}\n")).collect();
        format!("table {} {{\n{body}}}", quoted(&name))
    }
}

fn quoted(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| format!("\"{text}\""))
}

pub fn is_matrix(kind: &str) -> bool {
    matches!(kind, "traceability_matrix" | "allocation_matrix" | "interface_n2")
}

/// The title of a table of this kind that names none.
pub fn default_title(kind: &str) -> &'static str {
    match kind {
        "requirements_coverage" => "Requirement coverage",
        "traceability_matrix" => "Traceability matrix",
        "allocation_matrix" => "Allocation matrix",
        "interface_n2" => "N² interfaces",
        "icd" => "Interface control document",
        _ => "Elements",
    }
}

/// An element a cell names: enough to link to it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    pub id: String,
    pub uuid: String,
    pub name: String,
    pub kind: String,
    /// For a verification case: `pass`, `fail` or `pending`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub verdict: Option<String>,
}

/// How a cell is edited: one `arclang edit` request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellEdit {
    /// `set` (an attribute) or `rename` (the name, identity kept).
    pub op: String,
    pub element: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub key: Option<String>,
    /// The metamodel type of the attribute (`Text`, `Enum`, `Quantity`...).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub value_type: Option<String>,
    /// The values an enumeration takes.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    pub text: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub links: Vec<Link>,
    /// `ok`, `warn`, `gap` or `fail` when the cell carries a verdict.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub edit: Option<CellEdit>,
}

impl Cell {
    pub fn text(text: impl Into<String>) -> Cell {
        Cell { text: text.into(), ..Cell::default() }
    }
    pub fn links(links: Vec<Link>) -> Cell {
        let text = links.iter().map(|l| l.name.clone()).collect::<Vec<_>>().join(", ");
        Cell { text, links, ..Cell::default() }
    }
    pub fn editable(mut self, edit: Option<CellEdit>) -> Cell {
        self.edit = edit;
        self
    }
    pub fn with_status(mut self, status: &str) -> Cell {
        self.status = Some(status.to_string());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Column {
    pub key: String,
    pub label: String,
    /// `text`, `links`, `status`, `number`: how the cells read.
    #[serde(rename = "type")]
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Row {
    pub id: String,
    pub uuid: String,
    pub kind: String,
    pub name: String,
    /// `ok`, `warn`, `gap` or `fail`.
    pub status: String,
    /// Why the row is not ok, one line each.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub issues: Vec<String>,
    pub cells: BTreeMap<String, Cell>,
}

/// One cell of a matrix that holds something.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatrixCell {
    pub row: usize,
    pub column: usize,
    /// The kinds of what joins the two (`satisfies`, `allocates`...).
    pub kinds: Vec<String>,
    /// What the cell says, one entry per link (an exchange name, `satisfies`).
    pub items: Vec<String>,
    /// The links of the cell: kind, what declares it (`trace` when a
    /// `trace` declaration does, so the cell can remove it), its ends.
    pub links: Vec<MatrixLink>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatrixLink {
    pub kind: String,
    /// `trace`, `nesting`, `deployment`, `verifies`, `exchange`...
    pub source: String,
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub uuid: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Matrix {
    pub rows: Vec<Link>,
    pub columns: Vec<Link>,
    pub cells: Vec<MatrixCell>,
    /// The trace kinds a click may create, and which way each goes.
    pub creatable: Vec<String>,
    /// `row_to_column` or `column_to_row`, by trace kind.
    pub direction: BTreeMap<String, String>,
    /// Rows and columns with nothing in them (an unallocated function, a
    /// requirement nothing satisfies): the gaps of the matrix.
    pub empty_rows: Vec<usize>,
    pub empty_columns: Vec<usize>,
    /// Whether the diagonal is the element itself (an N² chart).
    pub diagonal: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub rows: usize,
    pub ok: usize,
    pub warn: usize,
    pub gap: usize,
    pub fail: usize,
    /// The production gate: passed, and its blockers about these rows.
    pub gate_standard: String,
    pub gate_passed: bool,
    pub gate_blockers: usize,
}

/// A computed table: what every output renders.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub id: String,
    /// The UUID of the declaration when the table is declared in the model.
    pub uuid: Option<String>,
    pub kind: String,
    pub declared: bool,
    pub spec: TableSpec,
    pub columns: Vec<Column>,
    /// Every column this kind could show, for a column picker.
    pub available_columns: Vec<Column>,
    pub rows: Vec<Row>,
    pub matrix: Option<Matrix>,
    pub summary: Summary,
}

/// Options shared by every table of a run.
#[derive(Debug, Clone)]
pub struct Options {
    pub gate_standard: String,
}

impl Default for Options {
    fn default() -> Self {
        Options { gate_standard: "ISO26262".to_string() }
    }
}

/// Compute one table of a compiled model.
pub fn compute(result: &CompilationResult, spec: &TableSpec, options: &Options) -> Result<Table, String> {
    let context = Context::new(result, &options.gate_standard);
    compute_in(&context, spec)
}

/// Compute one table with a context already built (several tables of one model).
pub fn compute_in(context: &Context, spec: &TableSpec) -> Result<Table, String> {
    let kind = spec.kind.as_str();
    let (columns, available, mut rows, matrix) = match kind {
        "requirements_coverage" => {
            let (c, a, r) = coverage::compute(context, spec)?;
            (c, a, r, None)
        }
        "elements" => {
            let (c, a, r) = elements_table::compute(context, spec)?;
            (c, a, r, None)
        }
        "icd" => {
            let (c, a, r) = icd::compute(context, spec)?;
            (c, a, r, None)
        }
        "traceability_matrix" | "allocation_matrix" | "interface_n2" => {
            let (rows, matrix) = matrix::compute(context, spec)?;
            (matrix::columns(), matrix::columns(), rows, Some(matrix))
        }
        other => return Err(format!("unknown table kind '{other}': one of {}", KINDS.join(", "))),
    };
    if let Some(text) = spec.filter.as_deref().filter(|f| !f.trim().is_empty()) {
        let query = filter::Query::parse(text);
        rows.retain(|row| query.matches(row));
    }
    if spec.gaps_only {
        rows.retain(|row| row.status == "gap" || row.status == "fail");
    }
    let matrix = matrix.map(|m| matrix::keep_rows(m, &rows));
    let count = |status: &str| rows.iter().filter(|r| r.status == status).count();
    // The gate's blockers that name an element of these rows, whatever the columns shown.
    let blockers = rows.iter().filter_map(|r| context.index_of_uuid(&r.uuid)).map(|i| context.blockers_of(i).len()).sum();
    let summary = Summary {
        rows: rows.len(),
        ok: count("ok"),
        warn: count("warn"),
        gap: count("gap"),
        fail: count("fail"),
        gate_standard: context.gate_standard.clone(),
        gate_passed: context.gate_passed,
        gate_blockers: blockers,
    };
    let declared = spec.name.as_ref().and_then(|name| context.declared(name));
    Ok(Table {
        name: spec.name.clone().unwrap_or_else(|| default_title(kind).to_string()),
        id: spec.id.clone().or_else(|| spec.name.clone()).unwrap_or_else(|| kind.to_string()),
        uuid: declared.map(|d| d.uuid.clone()),
        kind: kind.to_string(),
        declared: declared.is_some(),
        spec: spec.clone(),
        columns,
        available_columns: available,
        rows,
        matrix,
        summary,
    })
}

/// Every table the model declares, in declaration order.
pub fn declared_tables(result: &CompilationResult, options: &Options) -> Result<Vec<Table>, String> {
    let context = Context::new(result, &options.gate_standard);
    result.ast.tables.iter().map(|decl| compute_in(&context, &TableSpec::of(decl)).map_err(|e| format!("table '{}': {e}", decl.name))).collect()
}

/// The checks of a declaration the parser cannot make alone: a known kind.
pub fn check_declaration(decl: &TableDecl) -> Result<(), String> {
    if !KINDS.contains(&decl.kind.as_str()) {
        return Err(format!("table '{}': unknown kind '{}' (one of {})", decl.name, decl.kind, KINDS.join(", ")));
    }
    Ok(())
}

/// The label of a column key: `satisfied_by` reads `Satisfied by`.
pub fn label_of(key: &str) -> String {
    let words = key.replace('_', " ");
    let mut chars = words.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
