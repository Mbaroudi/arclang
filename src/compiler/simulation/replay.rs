//! Replaying a simulation on the diagram: the results of each run are kept
//! beside the model (`.arclang/sim-results/<case>/`), trimmed to the values
//! of the parts and the variables the expectations read, so that the
//! workbench can move through time, show each part's values on its box and
//! jump to the first instant an expectation fails.

use super::cache::model_dir;
use super::expects::{self, Expect};
use super::results::{units_from_init_xml, Series};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Time steps kept per run: enough for a slider, small beside the model.
pub const STEPS: usize = 600;
/// Columns kept per run, time aside.
pub const COLUMNS: usize = 400;

pub fn dir(model_path: &Path) -> PathBuf {
    model_dir(model_path).join(".arclang").join("sim-results")
}

pub fn case_dir(dir: &Path, test_id: &str) -> PathBuf {
    let safe: String = test_id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    dir.join(safe)
}

/// Whether results of a run of the case are kept.
pub fn kept(dir: &Path, test_id: &str) -> bool {
    case_dir(dir, test_id).join("index.json").is_file()
}

#[derive(Debug, Clone, Serialize, serde::Deserialize, Default)]
pub struct Index {
    /// The cache key of the run the results come from.
    pub key: String,
    /// The label of each point of a sweep (empty for a single run).
    pub points: Vec<String>,
}

/// Whether a result column is worth keeping: a direct variable of a part
/// (`batteryPack.soc`) or a variable an expectation reads.
fn worth_keeping(name: &str, parts: &BTreeMap<String, String>, read: &[String]) -> bool {
    read.iter().any(|r| r == name) || parts.values().any(|path| name.strip_prefix(path.as_str()).and_then(|rest| rest.strip_prefix('.')).is_some_and(|member| !member.contains('.') && !member.starts_with('$')))
}

/// Keep the results of a run: one CSV per point, trimmed, and an index.
/// The results of an earlier run of the case are replaced.
pub fn save(dir: &Path, test_id: &str, key: &str, labels: &[String], runs: &[(Option<String>, String)], parts: &BTreeMap<String, String>, parsed: &[Expect]) -> Result<(), String> {
    let target = case_dir(dir, test_id);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).map_err(|e| format!("cannot create {}: {e}", target.display()))?;
    let read: Vec<String> = parsed.iter().flat_map(expects::variables).collect();
    for (index, (csv, xml)) in runs.iter().enumerate() {
        let Some(csv) = csv else { continue };
        let mut series = Series::from_csv(csv)?;
        series.set_units(units_from_init_xml(xml));
        let mut kept: std::collections::HashSet<String> = read.iter().cloned().collect();
        kept.extend(series.names().iter().skip(1).filter(|n| worth_keeping(n, parts, &read)).take(COLUMNS).cloned());
        let trimmed = series.trimmed(&|name| kept.contains(name), STEPS);
        std::fs::write(target.join(format!("{index}.csv")), trimmed.to_csv()).map_err(|e| e.to_string())?;
        let units = serde_json::to_string(trimmed.units()).map_err(|e| e.to_string())?;
        std::fs::write(target.join(format!("{index}.units.json")), units).map_err(|e| e.to_string())?;
    }
    // A single run has no point to choose.
    let points = if labels.len() == 1 && labels[0].is_empty() { Vec::new() } else { labels.to_vec() };
    let index = Index { key: key.to_string(), points };
    std::fs::write(target.join("index.json"), serde_json::to_string_pretty(&index).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

/// A run read back for the workbench.
#[derive(Debug, Clone, Serialize)]
pub struct Replay {
    pub case: String,
    pub point: usize,
    /// The labels of every point of the sweep (empty for a single run).
    pub points: Vec<String>,
    /// Whether the model changed since the run (its cache key differs).
    pub stale: bool,
    pub time: Vec<f64>,
    /// The values of each part, by element id.
    pub elements: Vec<ElementSeries>,
    pub expectations: Vec<Expectation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ElementSeries {
    pub id: String,
    pub instance: String,
    pub values: Vec<Variable>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Variable {
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub unit: String,
    pub series: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Expectation {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirement: Option<String>,
    pub holds: bool,
    pub value: String,
    pub margin: String,
    /// The first instant it fails, in seconds; absent while it holds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_failure: Option<f64>,
    /// The element whose variable it reads first, when it reads a part's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
}

/// Read back the results of one point of a case.
pub fn load(dir: &Path, test_id: &str, point: usize, current_key: &str, parts: &BTreeMap<String, String>, parsed: &[Expect]) -> Result<Replay, String> {
    let target = case_dir(dir, test_id);
    let index: Index = std::fs::read_to_string(target.join("index.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .ok_or_else(|| format!("no results kept for '{test_id}': run `arclang verify` on the model first"))?;
    let csv = std::fs::read_to_string(target.join(format!("{point}.csv"))).map_err(|_| format!("'{test_id}' has no results for point {point} (the run failed there, or the sweep has fewer points)"))?;
    let mut series = Series::from_csv(&csv)?;
    let units: BTreeMap<String, String> = std::fs::read_to_string(target.join(format!("{point}.units.json"))).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
    series.set_units(units.clone());
    let mut elements = Vec::new();
    for (id, path) in parts {
        let values: Vec<Variable> = series
            .names()
            .iter()
            .enumerate()
            .filter_map(|(column, name)| {
                let member = name.strip_prefix(path.as_str())?.strip_prefix('.')?;
                (!member.contains('.')).then(|| Variable { name: member.to_string(), unit: units.get(name).cloned().unwrap_or_default(), series: series.values(column).collect() })
            })
            .collect();
        if !values.is_empty() {
            elements.push(ElementSeries { id: id.clone(), instance: path.clone(), values });
        }
    }
    let element_of = |variable: &str| parts.iter().find(|(_, path)| variable.strip_prefix(path.as_str()).is_some_and(|rest| rest.starts_with('.'))).map(|(id, _)| id.clone());
    let mut notes = Vec::new();
    let expectations = parsed
        .iter()
        .map(|expect| {
            let evaluated = expects::evaluate(expect, &mut expects::Context { series: &series, notes: &mut notes });
            let (holds, value, margin) = match &evaluated {
                Ok(e) => (e.holds, e.value.clone(), e.margin.clone()),
                Err(reason) => (false, reason.clone(), String::new()),
            };
            Expectation {
                text: expect.text.clone(),
                requirement: expect.requirement.clone(),
                holds,
                value,
                margin,
                first_failure: if holds { None } else { expects::first_failure(expect, &series) },
                element: expects::variables(expect).first().and_then(|v| element_of(v)),
            }
        })
        .collect();
    Ok(Replay { case: test_id.to_string(), point, points: index.points, stale: index.key != current_key, time: series.times().collect(), elements, expectations })
}
