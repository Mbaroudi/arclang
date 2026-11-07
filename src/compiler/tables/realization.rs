//! `realization`: per requirement, the state of each stage of its
//! realization (the simulation, SIL, PIL, HDL, the SoC, the board, the
//! twin) from the verdicts the runs cached, and what to run next.

use super::context::Context;
use super::{Cell, Column, Row, TableSpec};
use crate::compiler::realization::{self, STAGES};
use std::collections::BTreeMap;

pub const DEFAULT_COLUMNS: [&str; 10] = ["id", "name", "simulation", "sil", "pil", "hdl", "soc", "board", "twin", "next"];

fn column(key: &str) -> Column {
    let (label, ty) = match key {
        "id" => ("Req id".to_string(), "text"),
        "name" => ("Name".to_string(), "text"),
        "next" => ("Next".to_string(), "text"),
        "reached" => ("Reached".to_string(), "text"),
        stage => (realization::label(stage).to_string(), "status"),
    };
    Column { key: key.to_string(), label, ty: ty.to_string() }
}

/// A stage's state as a cell's status: `ok`, `warn`, `gap` or `fail`.
fn status(state: &str) -> &'static str {
    match state {
        "pass" => "ok",
        "fail" | "error" => "fail",
        "stale" => "warn",
        "missing" => "gap",
        _ => "ok",
    }
}

pub fn compute(context: &Context, spec: &TableSpec) -> Result<(Vec<Column>, Vec<Column>, Vec<Row>), String> {
    context.check_scope(spec.scope.as_deref())?;
    let available: Vec<Column> = DEFAULT_COLUMNS.iter().chain(["reached"].iter()).map(|k| column(k)).collect();
    let keys: Vec<String> = if spec.columns.is_empty() { DEFAULT_COLUMNS.iter().map(|s| s.to_string()).collect() } else { spec.columns.clone() };
    for key in &keys {
        if !available.iter().any(|c| &c.key == key) {
            return Err(format!("realization: unknown column '{key}' (one of {})", available.iter().map(|c| c.key.as_str()).collect::<Vec<_>>().join(", ")));
        }
    }
    let ladders = realization::ladders(context.result, None);
    let requirements: Vec<usize> = (0..context.graph.elements.len())
        .filter(|&i| context.element(i).kind == "Requirement")
        .filter(|&i| context.in_scope(i, spec.scope.as_deref()))
        .collect();
    let mut rows = Vec::new();
    for r in requirements {
        let element = context.element(r);
        let Some(ladder) = ladders.iter().find(|l| l.requirement == element.id || l.name == element.name || l.requirement == element.name) else { continue };
        let mut cells: BTreeMap<String, Cell> = BTreeMap::new();
        let mut issues = Vec::new();
        let mut worst = "ok";
        for key in &keys {
            let cell = match key.as_str() {
                "id" => Cell::text(element.id.clone()),
                "name" => Cell::text(element.name.clone()),
                "next" => Cell::text(ladder.next.clone()),
                "reached" => Cell::text(ladder.reached().map(realization::label).unwrap_or("").to_string()),
                stage => {
                    let Some(s) = ladder.stages.iter().find(|s| s.stage == stage) else { continue };
                    if s.state == "n/a" {
                        Cell::text("–")
                    } else {
                        let st = status(&s.state);
                        if st != "ok" {
                            issues.push(format!("{}: {}", realization::label(stage), s.cases.iter().filter(|c| c.state != "pass").map(|c| format!("{} {}", c.id, c.state)).collect::<Vec<_>>().join(", ")));
                        }
                        worst = match (worst, st) {
                            ("fail", _) | (_, "fail") => "fail",
                            ("warn", _) | (_, "warn") => "warn",
                            ("gap", _) | (_, "gap") => "gap",
                            _ => "ok",
                        };
                        Cell::text(format!("{}{}", s.state, if s.target.is_empty() { String::new() } else { format!(" ({})", s.target) })).with_status(st)
                    }
                }
            };
            cells.insert(key.clone(), cell);
        }
        rows.push(Row { id: element.id.clone(), uuid: element.uuid.clone(), kind: element.kind.to_string(), name: element.name.clone(), status: worst.to_string(), issues, cells, parent: None, depth: 0 });
    }
    let columns: Vec<Column> = keys.iter().map(|k| column(k)).collect();
    let _ = STAGES;
    Ok((columns, available, rows))
}
