//! The matrices: traceability (rows x columns, the trace kinds in the
//! cells), allocation (functions x components or nodes) and the N² chart
//! of interfaces (components x components, the exchanges in the cells).

use super::context::{layer_named, layer_rank, Context};
use super::{Cell, Column, Matrix, MatrixCell, MatrixLink, Row, TableSpec};
use crate::compiler::metamodel::Layer;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// The trace kinds of the language (`TraceKind`).
pub const TRACE_KINDS: [&str; 8] = ["satisfies", "implements", "validates", "verifies", "realizes", "refines", "derives", "allocates"];

/// The columns of the row list a matrix comes with.
pub fn columns() -> Vec<Column> {
    [("id", "Id", "text"), ("name", "Name", "links"), ("links", "Links", "text")]
        .iter()
        .map(|(k, l, t)| Column { key: k.to_string(), label: l.to_string(), ty: t.to_string() })
        .collect()
}

pub fn compute(context: &Context, spec: &TableSpec) -> Result<(Vec<Row>, Matrix), String> {
    context.check_scope(spec.scope.as_deref())?;
    let matrix = match spec.kind.as_str() {
        "traceability_matrix" => traceability(context, spec)?,
        "allocation_matrix" => allocation(context, spec)?,
        _ => n2(context, spec)?,
    };
    let rows = rows_of(context, &matrix);
    Ok((rows, matrix))
}

/// The list of a matrix's rows: each with how many links it has.
fn rows_of(context: &Context, matrix: &Matrix) -> Vec<Row> {
    let mut counts: HashMap<usize, (usize, BTreeSet<String>)> = HashMap::new();
    for cell in &matrix.cells {
        let entry = counts.entry(cell.row).or_default();
        entry.0 += cell.links.len();
        entry.1.extend(cell.kinds.iter().cloned());
    }
    matrix
        .rows
        .iter()
        .enumerate()
        .map(|(i, link)| {
            let (count, kinds) = counts.get(&i).cloned().unwrap_or_default();
            let empty = matrix.empty_rows.contains(&i);
            let mut cells = BTreeMap::new();
            cells.insert("id".to_string(), Cell::text(link.id.clone()));
            cells.insert("name".to_string(), Cell::links(vec![link.clone()]));
            let summary = if count == 0 { "none".to_string() } else { format!("{count} ({})", kinds.into_iter().collect::<Vec<_>>().join(", ")) };
            cells.insert("links".to_string(), if empty { Cell::text(summary).with_status("gap") } else { Cell::text(summary) });
            let issues = if empty { vec![empty_reason(&context.element(context.index_of_uuid(&link.uuid).unwrap_or(0)).kind.to_string())] } else { Vec::new() };
            Row { id: link.id.clone(), uuid: link.uuid.clone(), kind: link.kind.clone(), name: link.name.clone(), status: if empty { "gap".to_string() } else { "ok".to_string() }, issues, cells }
        })
        .collect()
}

fn empty_reason(kind: &str) -> String {
    if kind.ends_with("Function") || kind == "OperationalActivity" {
        "allocated to nothing".to_string()
    } else if kind == "Requirement" {
        "traced to nothing".to_string()
    } else {
        "no link in this matrix".to_string()
    }
}

/// Keep the rows the filter kept; the cells follow their rows.
pub fn keep_rows(matrix: Matrix, rows: &[Row]) -> Matrix {
    let kept: BTreeSet<&str> = rows.iter().map(|r| r.uuid.as_str()).collect();
    let mut remap = HashMap::new();
    let mut new_rows = Vec::new();
    for (i, row) in matrix.rows.iter().enumerate() {
        if kept.contains(row.uuid.as_str()) {
            remap.insert(i, new_rows.len());
            new_rows.push(row.clone());
        }
    }
    let cells = matrix.cells.iter().filter_map(|c| remap.get(&c.row).map(|&r| MatrixCell { row: r, ..c.clone() })).collect();
    let empty_rows = matrix.empty_rows.iter().filter_map(|r| remap.get(r).copied()).collect();
    Matrix { rows: new_rows, cells, empty_rows, ..matrix }
}

/// Selectors given, else the default ones.
fn selectors(given: &[String], default: &[&str]) -> Vec<String> {
    if given.is_empty() { default.iter().map(|s| s.to_string()).collect() } else { given.to_vec() }
}

struct Builder {
    cells: BTreeMap<(usize, usize), MatrixCell>,
}

impl Builder {
    fn add(&mut self, row: usize, column: usize, kind: &str, item: String, link: MatrixLink) {
        let cell = self.cells.entry((row, column)).or_insert_with(|| MatrixCell { row, column, kinds: Vec::new(), items: Vec::new(), links: Vec::new() });
        if cell.links.contains(&link) {
            return;
        }
        if !cell.kinds.iter().any(|k| k == kind) {
            cell.kinds.push(kind.to_string());
        }
        cell.items.push(item);
        cell.links.push(link);
    }
}

fn finish(context: &Context, rows: Vec<usize>, columns: Vec<usize>, builder: Builder, creatable: Vec<String>, direction: BTreeMap<String, String>, diagonal: bool) -> Matrix {
    let cells: Vec<MatrixCell> = builder.cells.into_values().collect();
    let used_rows: BTreeSet<usize> = cells.iter().map(|c| c.row).collect();
    let used_columns: BTreeSet<usize> = cells.iter().map(|c| c.column).collect();
    Matrix {
        empty_rows: (0..rows.len()).filter(|r| !used_rows.contains(r)).collect(),
        empty_columns: (0..columns.len()).filter(|c| !used_columns.contains(c)).collect(),
        rows: rows.iter().map(|&i| context.link(i)).collect(),
        columns: columns.iter().map(|&i| context.link(i)).collect(),
        cells,
        creatable,
        direction,
        diagonal,
    }
}

/// Which way a trace of `kind` goes between a row and a column element:
/// toward the requirement, from the lower level to the upper one, from
/// the component to the function it is allocated.
fn direction_of(context: &Context, kind: &str, row: Option<usize>, column: Option<usize>) -> &'static str {
    let (Some(row), Some(column)) = (row, column) else { return "column_to_row" };
    let row_kind = context.element(row).kind;
    let column_kind = context.element(column).kind;
    let is_function = |k: &str| k.ends_with("Function") || k == "OperationalActivity";
    let is_target = |k: &str| matches!(k, "Requirement" | "Capability" | "OperationalCapability");
    match kind {
        "realizes" => {
            if layer_rank(context.layer_of(row)) > layer_rank(context.layer_of(column)) { "row_to_column" } else { "column_to_row" }
        }
        "allocates" | "implements" if is_function(row_kind) && !is_function(column_kind) => "column_to_row",
        "allocates" | "implements" if is_function(column_kind) && !is_function(row_kind) => "row_to_column",
        "refines" | "derives" => "row_to_column",
        _ if is_target(row_kind) && !is_target(column_kind) => "column_to_row",
        _ if is_target(column_kind) => "row_to_column",
        _ if row_kind == "TestCase" => "row_to_column",
        _ => "column_to_row",
    }
}

fn traceability(context: &Context, spec: &TableSpec) -> Result<Matrix, String> {
    let row_selectors = selectors(&spec.rows, &["Requirement"]);
    let rows = context.elements_of(&row_selectors, spec.scope.as_deref());
    let row_set: BTreeSet<usize> = rows.iter().copied().collect();
    let kinds: Vec<String> = if spec.kinds.is_empty() { TRACE_KINDS.iter().map(|s| s.to_string()).collect() } else { spec.kinds.clone() };
    for kind in &kinds {
        if !TRACE_KINDS.contains(&kind.as_str()) {
            return Err(format!("'{kind}' is no trace kind (one of {})", TRACE_KINDS.join(", ")));
        }
    }
    // Every link of the kinds asked: traces, and the verification cases' `verifies:`.
    let mut links: Vec<(usize, usize, String, &'static str, String)> = Vec::new();
    for relationship in &context.graph.relationships {
        let (kind, source) = match relationship.kind {
            "Trace" => (relationship.extra.get("traceKind").and_then(|k| k.as_str()).unwrap_or("").to_string(), "trace"),
            "Verification" => ("verifies".to_string(), "verifies"),
            "Realization" => ("realizes".to_string(), "realizes"),
            _ => continue,
        };
        if !kinds.contains(&kind) {
            continue;
        }
        if let (Some(a), Some(b)) = (context.index_of_uuid(&relationship.source), context.index_of_uuid(&relationship.target)) {
            links.push((a, b, kind, source, relationship.uuid.clone()));
        }
    }
    let columns: Vec<usize> = if spec.matrix_columns.is_empty() {
        // The elements the rows are traced with, in graph order.
        let others: BTreeSet<usize> = links
            .iter()
            .filter_map(|(a, b, ..)| if row_set.contains(a) && !row_set.contains(b) { Some(*b) } else if row_set.contains(b) && !row_set.contains(a) { Some(*a) } else { None })
            .collect();
        others.into_iter().collect()
    } else {
        context.elements_of(&spec.matrix_columns, None)
    };
    let row_at: HashMap<usize, usize> = rows.iter().enumerate().map(|(i, &e)| (e, i)).collect();
    let column_at: HashMap<usize, usize> = columns.iter().enumerate().map(|(i, &e)| (e, i)).collect();
    let mut builder = Builder { cells: BTreeMap::new() };
    for (a, b, kind, source, uuid) in &links {
        for (r, c) in [(a, b), (b, a)] {
            if let (Some(&row), Some(&column)) = (row_at.get(r), column_at.get(c)) {
                let link = MatrixLink { kind: kind.clone(), source: source.to_string(), from: context.element(*a).id.clone(), to: context.element(*b).id.clone(), uuid: Some(uuid.clone()) };
                builder.add(row, column, kind, kind.clone(), link);
            }
        }
    }
    let creatable = kinds.clone();
    let direction = creatable.iter().map(|k| (k.clone(), direction_of(context, k, rows.first().copied(), columns.first().copied()).to_string())).collect();
    Ok(finish(context, rows, columns, builder, creatable, direction, false))
}

/// The functions and the blocks of a level, for an allocation matrix.
fn allocation_kinds(layer: Layer) -> (&'static [&'static str], &'static [&'static str]) {
    match layer {
        Layer::Operational => (&["OperationalActivity"], &["OperationalEntity", "Actor"]),
        Layer::System => (&["SystemFunction"], &["SystemComponent", "SystemActor"]),
        Layer::Physical => (&["PhysicalFunction"], &["BehaviorComponent", "PhysicalNode"]),
        _ => (&["LogicalFunction"], &["LogicalComponent"]),
    }
}

fn allocation(context: &Context, spec: &TableSpec) -> Result<Matrix, String> {
    let layer = spec.scope.as_deref().and_then(layer_named).unwrap_or(Layer::Logical);
    let scope = spec.scope.as_deref().filter(|s| layer_named(s).is_none());
    let (functions, blocks) = allocation_kinds(layer);
    let rows = context.elements_of(&selectors(&spec.rows, functions), scope);
    let columns = context.elements_of(&selectors(&spec.matrix_columns, blocks), scope);
    let column_at: HashMap<usize, usize> = columns.iter().enumerate().map(|(i, &e)| (e, i)).collect();
    let mut builder = Builder { cells: BTreeMap::new() };
    // Deployments: a component on a node hosts what the component performs.
    let mut deployed_on: HashMap<usize, Vec<usize>> = HashMap::new();
    for deployment in context.relationships("Deployment") {
        if let (Some(component), Some(node)) = (context.index_of_uuid(&deployment.source), context.index_of_uuid(&deployment.target)) {
            deployed_on.entry(component).or_default().push(node);
        }
    }
    for (r, &function) in rows.iter().enumerate() {
        let owners = context.owners(function);
        let blocks_up: Vec<usize> = owners.iter().copied().filter(|o| !context.element(*o).kind.ends_with("Function") && context.element(*o).kind != "Model").collect();
        if let Some((nearest, rest)) = blocks_up.split_first() {
            if let Some(&c) = column_at.get(nearest) {
                builder.add(r, c, "allocated", "allocated".to_string(), MatrixLink { kind: "allocated".to_string(), source: "nesting".to_string(), from: context.element(*nearest).id.clone(), to: context.element(function).id.clone(), uuid: None });
            }
            for holder in rest {
                if let Some(&c) = column_at.get(holder) {
                    builder.add(r, c, "contains", "contains".to_string(), MatrixLink { kind: "contains".to_string(), source: "nesting".to_string(), from: context.element(*holder).id.clone(), to: context.element(function).id.clone(), uuid: None });
                }
            }
            for block in &blocks_up {
                for node in deployed_on.get(block).into_iter().flatten() {
                    if let Some(&c) = column_at.get(node) {
                        builder.add(r, c, "deployed", format!("deployed with {}", context.element(*block).name), MatrixLink { kind: "deployed".to_string(), source: "deployment".to_string(), from: context.element(*node).id.clone(), to: context.element(function).id.clone(), uuid: None });
                    }
                }
            }
        }
    }
    let row_at: HashMap<usize, usize> = rows.iter().enumerate().map(|(i, &e)| (e, i)).collect();
    for trace in context.relationships("Trace") {
        let kind = trace.extra.get("traceKind").and_then(|k| k.as_str()).unwrap_or("");
        if !matches!(kind, "allocates" | "implements") {
            continue;
        }
        let (Some(a), Some(b)) = (context.index_of_uuid(&trace.source), context.index_of_uuid(&trace.target)) else { continue };
        for (block, function) in [(a, b), (b, a)] {
            if let (Some(&r), Some(&c)) = (row_at.get(&function), column_at.get(&block)) {
                builder.add(r, c, kind, kind.to_string(), MatrixLink { kind: kind.to_string(), source: "trace".to_string(), from: context.element(a).id.clone(), to: context.element(b).id.clone(), uuid: Some(trace.uuid.clone()) });
            }
        }
    }
    let creatable = vec!["allocates".to_string()];
    let direction = BTreeMap::from([("allocates".to_string(), "column_to_row".to_string())]);
    Ok(finish(context, rows, columns, builder, creatable, direction, false))
}

/// The blocks and the exchanges of a level, for an N² chart.
fn n2_kinds(layer: Layer) -> (&'static [&'static str], &'static [&'static str]) {
    match layer {
        Layer::Operational => (&["OperationalEntity", "Actor"], &["OperationalExchange"]),
        Layer::System => (&["SystemComponent", "SystemActor"], &["ComponentExchange"]),
        Layer::Physical => (&["PhysicalNode"], &["PhysicalExchange", "PhysicalLink"]),
        _ => (&["LogicalComponent"], &["ComponentExchange"]),
    }
}

fn n2(context: &Context, spec: &TableSpec) -> Result<Matrix, String> {
    let layer = spec.scope.as_deref().and_then(layer_named).unwrap_or(Layer::Logical);
    let scope = spec.scope.as_deref().filter(|s| layer_named(s).is_none());
    let (blocks, exchanges) = n2_kinds(layer);
    let members = context.elements_of(&selectors(&spec.rows, blocks), scope);
    let at: HashMap<usize, usize> = members.iter().enumerate().map(|(i, &e)| (e, i)).collect();
    // An end is the nearest member holding it: a port is its component's.
    let member_of = |index: usize| -> Option<usize> {
        std::iter::once(index).chain(context.owners(index)).find_map(|i| at.get(&i).copied())
    };
    let mut builder = Builder { cells: BTreeMap::new() };
    for relationship in context.graph.relationships.iter().filter(|r| exchanges.contains(&r.kind)) {
        let (Some(a), Some(b)) = (context.index_of_uuid(&relationship.source), context.index_of_uuid(&relationship.target)) else { continue };
        let (Some(from), Some(to)) = (member_of(a), member_of(b)) else { continue };
        if from == to {
            continue;
        }
        let name = relationship.name.clone().unwrap_or_else(|| relationship.kind.to_string());
        let link = MatrixLink { kind: relationship.kind.to_string(), source: "exchange".to_string(), from: context.element(a).id.clone(), to: context.element(b).id.clone(), uuid: Some(relationship.uuid.clone()) };
        builder.add(from, to, relationship.kind, name, link);
    }
    let mut matrix = finish(context, members.clone(), members, builder, Vec::new(), BTreeMap::new(), true);
    // A block that exchanges nothing, either way, is the gap of an N² chart.
    let used: BTreeSet<usize> = matrix.cells.iter().flat_map(|c| [c.row, c.column]).collect();
    matrix.empty_rows = (0..matrix.rows.len()).filter(|r| !used.contains(r)).collect();
    matrix.empty_columns = matrix.empty_rows.clone();
    Ok(matrix)
}
