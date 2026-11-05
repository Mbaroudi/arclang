//! `schedule`: the io_bindings of a target as tasks — period, priority,
//! the step time `arclang partition` measured, the worst response time,
//! whether each meets its period (docs/embedded-c.md, "Tasks and
//! scheduling").

use super::context::Context;
use super::{Cell, Column, Row, TableSpec};
use crate::compiler::embedded::{binding, plan};
use crate::compiler::schedule;
use std::collections::BTreeMap;

pub const DEFAULT_COLUMNS: [&str; 8] = ["binding", "machine", "target", "period_us", "priority", "wcet_us", "response_us", "meets"];
const ALL: [&str; 9] = ["binding", "machine", "target", "period_us", "priority", "wcet_us", "wcet_source", "response_us", "meets"];

fn column(key: &str) -> Column {
    let (label, ty) = match key {
        "binding" => ("Binding", "text"),
        "machine" => ("Machine", "text"),
        "target" => ("Target", "text"),
        "period_us" => ("Period (us)", "number"),
        "priority" => ("Priority", "number"),
        "wcet_us" => ("Step (us)", "number"),
        "wcet_source" => ("Step time from", "text"),
        "response_us" => ("Response (us)", "number"),
        "meets" => ("Meets its period", "status"),
        other => (other, "text"),
    };
    Column { key: key.to_string(), label: label.to_string(), ty: ty.to_string() }
}

fn number(v: Option<f64>) -> Cell {
    v.map(|x| Cell::text(if x.fract() == 0.0 { format!("{x:.0}") } else { format!("{x:.2}").trim_end_matches('0').to_string() })).unwrap_or_else(|| Cell::text(String::new()))
}

pub fn compute(context: &Context, spec: &TableSpec) -> Result<(Vec<Column>, Vec<Column>, Vec<Row>), String> {
    context.check_scope(spec.scope.as_deref())?;
    let available: Vec<Column> = ALL.iter().map(|k| column(k)).collect();
    let keys: Vec<String> = if spec.columns.is_empty() { DEFAULT_COLUMNS.iter().map(|s| s.to_string()).collect() } else { spec.columns.clone() };
    for key in &keys {
        if !available.iter().any(|c| &c.key == key) {
            return Err(format!("schedule: unknown column '{key}' (one of {})", ALL.join(", ")));
        }
    }
    let ast = &context.result.ast;
    let p = plan::plan(ast)?;
    let bindings = binding::resolve_all(ast, &p)?;
    let mut targets: Vec<String> = bindings.iter().map(|b| b.target.clone()).collect();
    targets.sort();
    targets.dedup();
    let mut rows = Vec::new();
    for target in &targets {
        let analysis = schedule::analyse(&p, &bindings, target, &context.result.partition, &[]);
        for t in &analysis.tasks {
            let mut cells: BTreeMap<String, Cell> = BTreeMap::new();
            for key in &keys {
                let cell = match key.as_str() {
                    "binding" => Cell::text(t.binding.clone()),
                    "machine" => Cell::text(t.machine.clone()),
                    "target" => Cell::text(target.clone()),
                    "period_us" => number(Some(t.period_us as f64)),
                    "priority" => number(Some(t.priority as f64)),
                    "wcet_us" => number(t.wcet_us),
                    "wcet_source" => Cell::text(t.wcet_source.clone()),
                    "response_us" => number(t.response_us),
                    "meets" => match t.meets {
                        Some(true) if t.wcet_source == "measured" || t.wcet_source == "given" => Cell::text("meets").with_status("ok"),
                        Some(true) => Cell::text("meets (stale measurement)").with_status("warn"),
                        Some(false) => Cell::text("MISSES").with_status("fail"),
                        None => Cell::text(t.wcet_source.clone()).with_status("gap"),
                    },
                    _ => Cell::text(String::new()),
                };
                cells.insert(key.clone(), cell);
            }
            let status = match t.meets {
                Some(true) if t.wcet_source.starts_with("measured (stale") => "warn",
                Some(true) => "ok",
                Some(false) => "fail",
                None => "gap",
            };
            rows.push(Row { id: t.binding.clone(), uuid: String::new(), kind: "IoBinding".to_string(), name: t.binding.clone(), status: status.to_string(), issues: Vec::new(), cells, parent: None, depth: 0 });
        }
    }
    let chosen: Vec<Column> = keys.iter().map(|k| column(k)).collect();
    Ok((chosen, available, rows))
}
