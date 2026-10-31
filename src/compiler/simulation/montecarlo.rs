//! Monte Carlo verification: a case simulated over samples of uncertain
//! parameters, and a verdict on how likely its requirements hold.
//!
//! ```text
//! simulate: { model: "EVPhysics.TouringRange" stop_time: 5 h sources: ["physics/EVPhysics.mo"]
//!             uncertain: ["vehicleBody.mass ~ normal(1850 kg, 40 kg)",
//!                         "\"PN-BATT\".capacity ~ uniform(58 kWh, 62 kWh)",
//!                         "driveUnit.efficiency ~ triangular(0.86, 0.9, 0.92)"]
//!             samples: 300 seed: 7 probability: 0.99 confidence: 0.95 }
//! ```
//!
//! The samples are a Latin hypercube (each parameter's range cut in as many
//! strata as samples, one sample per stratum), drawn from the seed, so the
//! same case draws the same samples on every machine. Each sample is one
//! simulation with the parameters overridden and proven applied, as a sweep
//! point is. The case passes when, at the confidence given, the probability
//! that its expectations hold is shown to reach the one required: the lower
//! bound of the Clopper-Pearson interval on the samples that hold. The
//! verdict is computed here, never by an external tool, so the gate needs
//! nothing but the cache.

use super::sweep::{self, Level, Point};
use super::PointOutcome;
use crate::compiler::quantity::Quantity;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use statrs::distribution::{Beta, ContinuousCDF, Normal, Triangular, Uniform};
use std::collections::BTreeMap;

/// Samples a case draws at most.
pub const MAX_SAMPLES: usize = 1000;
pub const DEFAULT_SAMPLES: usize = 200;
pub const DEFAULT_SEED: u64 = 1;
pub const DEFAULT_CONFIDENCE: f64 = 0.95;

/// A law of an uncertain parameter, in the unit it is written in.
#[derive(Debug, Clone, PartialEq)]
pub enum Law {
    Normal { mean: f64, sd: f64 },
    Uniform { low: f64, high: f64 },
    Triangular { low: f64, mode: f64, high: f64 },
}

impl Law {
    /// The value at probability `u` (0 < u < 1).
    fn quantile(&self, u: f64) -> f64 {
        match *self {
            Law::Normal { mean, sd } => Normal::new(mean, sd).expect("checked").inverse_cdf(u),
            Law::Uniform { low, high } => Uniform::new(low, high).expect("checked").inverse_cdf(u),
            Law::Triangular { low, mode, high } => Triangular::new(low, high, mode).expect("checked").inverse_cdf(u),
        }
    }
}

/// One uncertain parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Uncertain {
    /// The Modelica path of the parameter, ArcLang ids resolved.
    pub variable: String,
    pub law: Law,
    /// The unit the law is written in (empty for a plain number).
    pub unit: String,
}

/// What a probabilistic case requires.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Requirement {
    /// The probability the expectations must be shown to hold with.
    pub probability: f64,
    /// The confidence it must be shown at.
    pub confidence: f64,
}

/// `1850 kg` -> (1850, "kg"); `0.9` -> (0.9, ""). The unit must be known.
fn number_and_unit(text: &str) -> Result<(f64, String), String> {
    let text = text.trim();
    if let Ok(n) = text.parse::<f64>() {
        return Ok((n, String::new()));
    }
    let q = Quantity::parse(text).map_err(|_| format!("uncertain: '{text}' is neither a number nor a quantity with a known unit"))?;
    Ok((q.value, q.unit.clone()))
}

/// `normal(1850 kg, 40 kg)`: its law, and the one unit its arguments share.
fn law(text: &str) -> Result<(Law, String), String> {
    let text = text.trim();
    let (name, rest) = text.split_once('(').ok_or_else(|| format!("uncertain: '{text}' is not a law: normal(mean, sd), uniform(low, high) or triangular(low, mode, high)"))?;
    let arguments = rest.strip_suffix(')').ok_or_else(|| format!("uncertain: '{text}' does not close its parenthesis"))?;
    let values = arguments.split(',').map(number_and_unit).collect::<Result<Vec<_>, _>>()?;
    let unit = values[0].1.clone();
    if values.iter().any(|(_, u)| *u != unit) {
        return Err(format!("uncertain: '{text}' mixes units: write every argument in one unit"));
    }
    let n: Vec<f64> = values.iter().map(|(v, _)| *v).collect();
    let arity = |count: usize, form: &str| if n.len() == count { Ok(()) } else { Err(format!("uncertain: {} takes {count} arguments: {form}", name.trim())) };
    let law = match name.trim() {
        "normal" => {
            arity(2, "normal(mean, sd)")?;
            if n[1] <= 0.0 {
                return Err(format!("uncertain: the sd of '{text}' must be positive"));
            }
            Law::Normal { mean: n[0], sd: n[1] }
        }
        "uniform" => {
            arity(2, "uniform(low, high)")?;
            if n[1] <= n[0] {
                return Err(format!("uncertain: '{text}' must have low < high"));
            }
            Law::Uniform { low: n[0], high: n[1] }
        }
        "triangular" => {
            arity(3, "triangular(low, mode, high)")?;
            if !(n[0] < n[2] && n[0] <= n[1] && n[1] <= n[2]) {
                return Err(format!("uncertain: '{text}' must have low <= mode <= high and low < high"));
            }
            Law::Triangular { low: n[0], mode: n[1], high: n[2] }
        }
        other => return Err(format!("uncertain: '{other}' is not a law: normal, uniform or triangular")),
    };
    Ok((law, unit))
}

/// One parameter: `variable ~ law(...)`.
pub fn parse_one(text: &str, aliases: &BTreeMap<String, String>) -> Result<Uncertain, String> {
    let (variable, law_text) = text.split_once('~').ok_or_else(|| format!("uncertain: '{text}' is not `parameter ~ law` (e.g. `body.mass ~ normal(1850 kg, 40 kg)`)"))?;
    let variable = sweep::resolve(variable.trim(), aliases).map_err(|e| e.replacen("sweep:", "uncertain:", 1))?;
    if !sweep::is_path(&variable) {
        return Err(format!("uncertain: '{variable}' is not a Modelica path (part.parameter)"));
    }
    let (law, unit) = law(law_text)?;
    Ok(Uncertain { variable, law, unit })
}

pub fn parse(texts: &[String], aliases: &BTreeMap<String, String>) -> Result<Vec<Uncertain>, String> {
    let parameters = texts.iter().map(|t| parse_one(t, aliases)).collect::<Result<Vec<_>, _>>()?;
    for (i, parameter) in parameters.iter().enumerate() {
        if parameters[..i].iter().any(|p| p.variable == parameter.variable) {
            return Err(format!("uncertain: '{}' is given twice", parameter.variable));
        }
    }
    Ok(parameters)
}

/// A uniform number in (0, 1), never 0 or 1 (the quantiles there are infinite).
fn unit_interval(rng: &mut ChaCha8Rng) -> f64 {
    ((rng.next_u64() >> 11) as f64 + 0.5) / (1u64 << 53) as f64
}

/// A value as written: four significant digits, in its unit; and in SI.
fn level(value: f64, unit: &str) -> Level {
    let shown_number = format!("{}", (value * 1e6).round() / 1e6);
    let shown = if unit.is_empty() { shown_number.clone() } else { format!("{shown_number} {unit}") };
    let si = if unit.is_empty() { value } else { Quantity::parse(&shown).map(|q| q.canonical()).unwrap_or(value) };
    Level { si, shown }
}

/// `samples` points of a Latin hypercube over the parameters, from `seed`.
pub fn sample(parameters: &[Uncertain], samples: usize, seed: u64) -> Vec<Point> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let columns: Vec<Vec<Level>> = parameters
        .iter()
        .map(|parameter| {
            // One sample per stratum, the strata in a random order.
            let mut strata: Vec<usize> = (0..samples).collect();
            for i in (1..samples).rev() {
                let j = (rng.next_u64() % (i as u64 + 1)) as usize;
                strata.swap(i, j);
            }
            strata.into_iter().map(|stratum| level(parameter.law.quantile((stratum as f64 + unit_interval(&mut rng)) / samples as f64), &parameter.unit)).collect()
        })
        .collect();
    (0..samples).map(|i| parameters.iter().zip(&columns).map(|(p, column)| (p.variable.clone(), column[i].clone())).collect()).collect()
}

/// The Clopper-Pearson interval of a proportion: `held` of `total`, at `confidence` (one-sided bounds).
pub fn interval(held: usize, total: usize, confidence: f64) -> (f64, f64) {
    let alpha = 1.0 - confidence;
    let (k, n) = (held as f64, total as f64);
    let low = if held == 0 { 0.0 } else { Beta::new(k, n - k + 1.0).expect("positive shapes").inverse_cdf(alpha) };
    let high = if held == total { 1.0 } else { Beta::new(k + 1.0, n - k).expect("positive shapes").inverse_cdf(1.0 - alpha) };
    (low, high)
}

/// Samples needed, all holding, to show `probability` at `confidence`.
pub fn samples_needed(probability: f64, confidence: f64) -> usize {
    ((1.0 - confidence).ln() / probability.ln()).ceil() as usize
}

fn percent(p: f64) -> String {
    let text = format!("{:.2}", p * 100.0);
    format!("{} %", text.trim_end_matches('0').trim_end_matches('.'))
}

/// How often the expectations held over the samples, and the probability
/// it shows: the bounds of its Clopper-Pearson interval.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Estimate {
    pub held: usize,
    pub total: usize,
    pub low: f64,
    pub high: f64,
}

/// The estimate of samples that all ran; `None` when one did not.
pub fn estimate(points: &[PointOutcome], confidence: f64) -> Option<Estimate> {
    if points.is_empty() || points.iter().any(|p| !p.error.is_empty()) {
        return None;
    }
    let held = points.iter().filter(|p| p.holds).count();
    let (low, high) = interval(held, points.len(), confidence);
    Some(Estimate { held, total: points.len(), low, high })
}

/// The verdict of a probabilistic case from its samples.
pub fn judge(points: &[PointOutcome], required: Requirement) -> (String, String) {
    if let Some(broken) = points.iter().find(|p| !p.error.is_empty()) {
        let count = points.iter().filter(|p| !p.error.is_empty()).count();
        return ("error".to_string(), format!("{count} of {} samples did not run; first at {}: {}", points.len(), broken.label, broken.error));
    }
    let total = points.len();
    let held = points.iter().filter(|p| p.holds).count();
    let (low, high) = interval(held, total, required.confidence);
    let summary = format!("holds in {held} of {total} samples: probability between {} and {} at {} confidence", percent(low), percent(high), percent(required.confidence));
    if low >= required.probability {
        return ("pass".to_string(), summary);
    }
    let first = points.iter().find(|p| !p.holds).map(|p| {
        let failed: Vec<&str> = p.values.iter().filter(|v| !v.holds).map(|v| v.value.as_str()).collect();
        format!("; first failure at {}: {}", p.label, failed.join("; "))
    });
    let why = if high < required.probability {
        format!("below the {} required", percent(required.probability))
    } else if held == total {
        format!("not enough samples to show {}: {} holding samples would", percent(required.probability), samples_needed(required.probability, required.confidence))
    } else {
        format!("not shown to reach the {} required", percent(required.probability))
    };
    ("fail".to_string(), format!("{summary}, {why}{}", first.unwrap_or_default()))
}

/// How much one parameter moves one expectation's margin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Influence {
    pub expect: String,
    pub parameter: String,
    /// Spearman's rank correlation between the parameter and the margin
    /// (positive: more of the parameter, more to spare).
    pub rank_correlation: f64,
}

fn ranks(values: &[f64]) -> Vec<f64> {
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
    let mut ranks = vec![0.0; values.len()];
    let mut start = 0;
    while start < order.len() {
        let mut end = start;
        while end + 1 < order.len() && values[order[end + 1]] == values[order[start]] {
            end += 1;
        }
        // Ties share the mean of their ranks.
        let rank = (start + end) as f64 / 2.0;
        for &index in &order[start..=end] {
            ranks[index] = rank;
        }
        start = end + 1;
    }
    ranks
}

fn correlation(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len() as f64;
    let (mx, my) = (x.iter().sum::<f64>() / n, y.iter().sum::<f64>() / n);
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (a, b) in x.iter().zip(y) {
        sxy += (a - mx) * (b - my);
        sxx += (a - mx).powi(2);
        syy += (b - my).powi(2);
    }
    (sxx > 0.0 && syy > 0.0).then(|| sxy / (sxx * syy).sqrt())
}

/// Spearman's rank correlation of each parameter with each expectation's
/// margin over the samples that ran, strongest first for each expectation.
pub fn sensitivity(points: &[PointOutcome]) -> Vec<Influence> {
    let ran: Vec<&PointOutcome> = points.iter().filter(|p| p.error.is_empty() && !p.inputs.is_empty()).collect();
    let Some(first) = ran.first() else { return Vec::new() };
    if ran.len() < 3 {
        return Vec::new();
    }
    let mut influences = Vec::new();
    for (e, expect) in first.values.iter().enumerate() {
        let slacks: Option<Vec<f64>> = ran.iter().map(|p| p.values.get(e).and_then(|v| v.slack)).collect();
        let Some(slacks) = slacks else { continue };
        if is_constant(&slacks) {
            continue;
        }
        let slack_ranks = ranks(&slacks);
        let mut of_expect: Vec<Influence> = first
            .inputs
            .iter()
            .enumerate()
            .filter_map(|(i, (parameter, _))| {
                let inputs: Vec<f64> = ran.iter().map(|p| p.inputs[i].1).collect();
                let rho = correlation(&ranks(&inputs), &slack_ranks)?;
                Some(Influence { expect: expect.expect.clone(), parameter: parameter.clone(), rank_correlation: (rho * 1000.0).round() / 1000.0 })
            })
            .collect();
        of_expect.sort_by(|a, b| b.rank_correlation.abs().total_cmp(&a.rank_correlation.abs()));
        influences.extend(of_expect);
    }
    influences
}

/// A margin that does not vary over the samples (the run ends on its bound):
/// its ranks are numerical noise, no parameter moves it. The same test as the
/// R and Python analyses.
pub fn is_constant(margins: &[f64]) -> bool {
    let (low, high) = margins.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(l, h), &m| (l.min(m), h.max(m)));
    let largest = margins.iter().fold(1.0_f64, |a, m| a.max(m.abs()));
    high - low <= 1e-6 * largest
}

/// The samples as a table for R or Python: one row per sample, the
/// parameters in SI, each expectation's margin in SI and whether it held.
pub fn samples_csv(points: &[PointOutcome]) -> String {
    let Some(first) = points.iter().find(|p| p.error.is_empty() && !p.inputs.is_empty()) else { return String::new() };
    let quote = |text: &str| format!("\"{}\"", text.replace('"', "\"\""));
    let mut header: Vec<String> = vec!["sample".to_string()];
    header.extend(first.inputs.iter().map(|(name, _)| quote(name)));
    header.extend(first.values.iter().map(|v| quote(&format!("margin: {}", v.expect))));
    header.push("holds".to_string());
    let mut lines = vec![header.join(",")];
    for (index, point) in points.iter().enumerate().filter(|(_, p)| p.error.is_empty() && p.inputs.len() == first.inputs.len() && p.values.len() == first.values.len()) {
        let mut row = vec![(index + 1).to_string()];
        row.extend(point.inputs.iter().map(|(_, v)| v.to_string()));
        row.extend(point.values.iter().map(|v| v.slack.map_or(String::new(), |s| s.to_string())));
        row.push(if point.holds { "1" } else { "0" }.to_string());
        lines.push(row.join(","));
    }
    lines.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::simulation::expects::Evaluated;

    #[test]
    fn laws_are_read_with_their_units_and_checked() {
        let mass = parse_one("vehicleBody.mass ~ normal(1850 kg, 40 kg)", &BTreeMap::new()).unwrap();
        assert_eq!((mass.law.clone(), mass.unit.as_str()), (Law::Normal { mean: 1850.0, sd: 40.0 }, "kg"));
        let aliases: BTreeMap<String, String> = [("PN-BATT".to_string(), "batteryPack".to_string())].into_iter().collect();
        assert_eq!(parse_one("\"PN-BATT\".capacity ~ uniform(58 kWh, 62 kWh)", &aliases).unwrap().variable, "batteryPack.capacity");
        assert!(parse_one("a.b ~ triangular(0.86, 0.9, 0.92)", &aliases).is_ok());
        for (text, told) in [
            ("a.b = normal(1, 2)", "parameter ~ law"),
            ("a.b ~ normal(1 kg, 2 g)", "mixes units"),
            ("a.b ~ normal(1, -2)", "must be positive"),
            ("a.b ~ uniform(2, 1)", "low < high"),
            ("a.b ~ triangular(1, 5, 3)", "low <= mode <= high"),
            ("a.b ~ beta(1, 2)", "not a law"),
            ("a.b ~ normal(1)", "takes 2 arguments"),
            ("a b ~ normal(1, 1)", "Modelica path"),
        ] {
            let error = parse_one(text, &aliases).unwrap_err();
            assert!(error.contains(told), "{text}: {error}");
        }
        assert!(parse(&["a.b ~ normal(1, 1)".into(), "a.b ~ uniform(0, 1)".into()], &aliases).unwrap_err().contains("twice"));
    }

    #[test]
    fn a_latin_hypercube_puts_one_sample_in_each_stratum_and_repeats_from_its_seed() {
        let parameters = parse(&["a.x ~ uniform(0, 10)".into(), "a.t ~ normal(20 degC, 2 K)".into()], &BTreeMap::new());
        // Mixed units (degC, K) are refused: a law is written in one unit.
        assert!(parameters.is_err());
        let parameters = parse(&["a.x ~ uniform(0, 10)".into(), "a.t ~ normal(20 degC, 2 degC)".into()], &BTreeMap::new()).unwrap();
        let points = sample(&parameters, 50, 7);
        assert_eq!(points.len(), 50);
        let mut strata: Vec<usize> = points.iter().map(|p| (p[0].1.si / 10.0 * 50.0) as usize).collect();
        strata.sort();
        assert_eq!(strata, (0..50).collect::<Vec<_>>(), "one sample per tenth of the range");
        assert_eq!(sample(&parameters, 50, 7), points, "the same seed draws the same samples");
        assert_ne!(sample(&parameters, 50, 8), points);
        // A temperature is overridden in kelvin, and shown as written.
        let t = &points[0][1].1;
        assert!(t.shown.ends_with(" degC") && (t.si - 273.15 - t.shown.trim_end_matches(" degC").parse::<f64>().unwrap()).abs() < 1e-9, "{t:?}");
    }

    #[test]
    fn the_verdict_is_the_lower_bound_of_the_interval() {
        // Clopper-Pearson, checked against R: binom.test(k, n, alternative = "greater")$conf.int.
        let (low, _) = interval(298, 300, 0.95);
        assert!((low - 0.9791637695).abs() < 1e-9, "{low}");
        let (low, high) = interval(300, 300, 0.95);
        assert!((low - 0.9900639181).abs() < 1e-9 && high == 1.0, "{low}");
        // binom.test(150, 200, alternative = "less")$conf.int
        let (_, high) = interval(150, 200, 0.95);
        assert!((high - 0.7998267548).abs() < 1e-9, "{high}");
        assert_eq!(samples_needed(0.99, 0.95), 299);
        let points = |held: usize, total: usize| -> Vec<PointOutcome> {
            (0..total).map(|i| PointOutcome { label: format!("sample {}", i + 1), holds: i < held, values: vec![Evaluated { expect: "x >= 1".into(), value: "x = 0.5".into(), holds: i < held, margin: String::new(), requirement: None, slack: Some(0.0), slack_unit: None }], ..Default::default() }).collect()
        };
        let required = Requirement { probability: 0.99, confidence: 0.95 };
        assert_eq!(judge(&points(300, 300), required).0, "pass");
        let (verdict, reason) = judge(&points(200, 200), required);
        assert_eq!(verdict, "fail");
        assert!(reason.contains("not enough samples to show 99 %: 299 holding samples would"), "{reason}");
        let (verdict, reason) = judge(&points(150, 200), required);
        assert_eq!(verdict, "fail");
        assert!(reason.contains("below the 99 % required") && reason.contains("first failure at sample 151: x = 0.5"), "{reason}");
        let mut broken = points(10, 10);
        broken[3].error = "the simulation failed".into();
        assert_eq!(judge(&broken, required).0, "error");
        // The same numbers, for a reader that draws them.
        let shown = estimate(&points(298, 300), 0.95).unwrap();
        assert_eq!((shown.held, shown.total), (298, 300));
        assert!((shown.low - 0.9791637695).abs() < 1e-9 && shown.high < 1.0);
        assert_eq!(estimate(&broken, 0.95), None);
    }

    #[test]
    fn the_parameter_that_drives_a_margin_is_found_by_its_rank_correlation() {
        let points: Vec<PointOutcome> = (0..40)
            .map(|i| {
                let (mass, noise) = (1800.0 + i as f64 * 5.0, ((i * 7919) % 13) as f64);
                PointOutcome {
                    label: format!("sample {i}"),
                    inputs: vec![("body.mass".into(), mass), ("cell.noise".into(), noise)],
                    values: vec![Evaluated { expect: "final(range) >= 300 km".into(), value: String::new(), holds: true, margin: String::new(), requirement: None, slack: Some(50_000.0 - 20.0 * mass), slack_unit: None }],
                    holds: true,
                    error: String::new(),
                }
            })
            .collect();
        let influences = sensitivity(&points);
        assert_eq!(influences[0].parameter, "body.mass");
        assert_eq!(influences[0].rank_correlation, -1.0, "more mass, less range to spare");
        assert!(influences[1].rank_correlation.abs() < 0.5, "{influences:?}");
        let csv = samples_csv(&points);
        assert!(csv.starts_with("sample,\"body.mass\",\"cell.noise\",\"margin: final(range) >= 300 km\",holds\n1,1800,0,14000,1\n"), "{csv}");
        // A run that ends on its bound: the margin is noise, nothing moves it.
        let on_bound: Vec<PointOutcome> = points.iter().enumerate().map(|(i, p)| PointOutcome { values: vec![Evaluated { slack: Some(6.0e-5 + i as f64 * 1e-10), ..p.values[0].clone() }], ..p.clone() }).collect();
        assert!(sensitivity(&on_bound).is_empty());
    }
}
