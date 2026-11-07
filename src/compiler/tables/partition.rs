//! `partition`: per state machine, what `arclang partition` measured on
//! both sides — the C on the processor (a step's time against the period,
//! the footprint) and the VHDL on the logic (resources, Fmax against the
//! clock) — what fits, and what the model allocates today.

use super::context::Context;
use super::{Cell, Column, Row, TableSpec};
use std::collections::BTreeMap;

pub const DEFAULT_COLUMNS: [&str; 9] = ["machine", "period_us", "now", "step_us", "margin", "software", "luts", "fmax_mhz", "logic"];
const ALL: [&str; 16] = ["machine", "period_us", "now", "node", "target", "step_us", "margin", "text", "software", "device", "clock_mhz", "luts", "ffs", "dsps", "fmax_mhz", "logic"];

fn column(key: &str) -> Column {
    let (label, ty) = match key {
        "machine" => ("Machine", "text"),
        "period_us" => ("Period (us)", "number"),
        "now" => ("Now", "text"),
        "node" => ("Node", "text"),
        "target" => ("Target", "text"),
        "step_us" => ("Step (us)", "number"),
        "margin" => ("Margin (×)", "number"),
        "text" => ("Text (B)", "number"),
        "software" => ("C fits", "status"),
        "device" => ("Device", "text"),
        "clock_mhz" => ("Clock (MHz)", "number"),
        "luts" => ("LUT4", "number"),
        "ffs" => ("FF", "number"),
        "dsps" => ("DSP", "number"),
        "fmax_mhz" => ("Fmax (MHz)", "number"),
        "logic" => ("VHDL fits", "status"),
        other => (other, "text"),
    };
    Column { key: key.to_string(), label: label.to_string(), ty: ty.to_string() }
}

fn fit(fits: Option<bool>, fresh: bool, why: Option<&str>) -> Cell {
    match fits {
        Some(true) if fresh => Cell::text("fits").with_status("ok"),
        Some(false) if fresh => Cell::text("does not fit").with_status("fail"),
        Some(_) => Cell::text("stale: run arclang partition").with_status("warn"),
        None => Cell::text(why.unwrap_or("not measured")).with_status("gap"),
    }
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
            return Err(format!("partition: unknown column '{key}' (one of {})", ALL.join(", ")));
        }
    }
    let mut rows = Vec::new();
    for (e, fresh) in &context.result.partition {
        let mut cells: BTreeMap<String, Cell> = BTreeMap::new();
        for key in &keys {
            let cell = match key.as_str() {
                "machine" => Cell::text(e.machine.clone()),
                "period_us" => number(Some(e.period_us as f64)),
                "now" => Cell::text(e.now.clone()),
                "node" => Cell::text(e.node.clone().unwrap_or_default()),
                "target" => Cell::text(e.software.as_ref().map(|s| s.target.clone()).unwrap_or_default()),
                "step_us" => number(e.software.as_ref().map(|s| s.step_us)),
                "margin" => number(e.margin().map(|m| (m * 10.0).round() / 10.0)),
                "text" => number(e.software.as_ref().and_then(|s| s.text).map(|t| t as f64)),
                "software" => fit(e.fits_software(), *fresh, e.software_error.as_deref()),
                "device" => Cell::text(e.logic.as_ref().map(|l| l.device.clone()).unwrap_or_default()),
                "clock_mhz" => number(e.logic.as_ref().map(|l| l.clock_mhz)),
                "luts" => number(e.logic.as_ref().map(|l| l.luts as f64)),
                "ffs" => number(e.logic.as_ref().map(|l| l.ffs as f64)),
                "dsps" => number(e.logic.as_ref().map(|l| l.dsps as f64)),
                "fmax_mhz" => number(e.logic.as_ref().map(|l| l.fmax_mhz)),
                "logic" => fit(e.fits_logic(), *fresh, e.logic_error.as_deref()),
                _ => Cell::text(String::new()),
            };
            cells.insert(key.clone(), cell);
        }
        let status = match (e.fits_software(), e.fits_logic(), *fresh) {
            (_, _, false) => "warn",
            (Some(false), Some(false), _) => "fail",
            (None, None, _) => "gap",
            _ => "ok",
        };
        let issues = e.software_error.iter().chain(e.logic_error.iter()).cloned().collect();
        rows.push(Row { id: e.ident.clone(), uuid: String::new(), kind: "StateMachine".to_string(), name: e.machine.clone(), status: status.to_string(), issues, cells, parent: None, depth: 0 });
    }
    let chosen: Vec<Column> = keys.iter().map(|k| column(k)).collect();
    Ok((chosen, available, rows))
}
