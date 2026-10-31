//! Parameter sweeps: one case simulated over a grid of conditions.
//!
//! ```text
//! simulate: { model: "EVPhysics.CityRange" stop_time: 5 h sources: ["physics/EVPhysics.mo"]
//!             sweep: ["vehicleBody.ambient = 253.15 K, 263.15 K, 293.15 K",
//!                     "\"PN-VCU\".speedController.target_speed = 80 km/h .. 130 km/h step 25 km/h"] }
//! ```
//!
//! Each axis names a parameter of the simulated model (a Modelica path, its
//! first part may be an ArcLang id in quotes) and its values: a list, or a
//! range with a step. The grid is every combination; each point is one
//! simulation with the parameters overridden, and the `expects:` must hold
//! at every point. A point whose result does not show the value it was given
//! is an error, never a silent pass: a name that is not a parameter changes
//! nothing.

use crate::compiler::quantity::Quantity;
use std::collections::BTreeMap;

/// Points a case simulates at most.
pub const MAX_POINTS: usize = 200;

/// One value of an axis: in SI units (what Modelica computes in), and as written.
#[derive(Debug, Clone, PartialEq)]
pub struct Level {
    pub si: f64,
    pub shown: String,
}

/// One swept parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Axis {
    /// The Modelica path of the parameter, ArcLang ids resolved.
    pub variable: String,
    pub levels: Vec<Level>,
}

/// One point of the grid: a level of every axis.
pub type Point = Vec<(String, Level)>;

/// `"PN-VCU".speedController.target_speed` with `PN-VCU` -> `vehicleControlUnit`.
fn resolve(variable: &str, aliases: &BTreeMap<String, String>) -> Result<String, String> {
    let Some(rest) = variable.strip_prefix('"') else { return Ok(variable.to_string()) };
    let (id, path) = rest.split_once('"').ok_or_else(|| format!("sweep: '{variable}' opens a quote it does not close"))?;
    let instance = aliases.get(id).ok_or_else(|| format!("sweep: '{id}' is not a part of the simulated architecture"))?;
    Ok(format!("{instance}{path}"))
}

fn is_path(text: &str) -> bool {
    !text.is_empty()
        && text.split('.').all(|part| {
            let part = part.split('[').next().unwrap_or("");
            part.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

/// `253.15 K` (SI 253.15), `100 km/h` (27.78 m/s), `0.9` (a plain number).
fn level(text: &str) -> Result<Level, String> {
    let text = text.trim();
    if let Ok(n) = text.parse::<f64>() {
        return Ok(Level { si: n, shown: text.to_string() });
    }
    let q = Quantity::parse(text).map_err(|_| format!("sweep: '{text}' is neither a number nor a quantity with a known unit"))?;
    Ok(Level { si: q.canonical(), shown: text.to_string() })
}

/// `130 km/h` -> (130, "km/h"); `0.9` -> (0.9, "").
fn split_number(text: &str) -> Result<(f64, String), String> {
    let text = text.trim();
    let end = text.char_indices().take_while(|(i, c)| c.is_ascii_digit() || *c == '.' || (*i == 0 && matches!(c, '-' | '+'))).map(|(i, c)| i + c.len_utf8()).last().unwrap_or(0);
    let number = text[..end].parse::<f64>().map_err(|_| format!("sweep: '{text}' does not start with a number"))?;
    Ok((number, text[end..].trim().to_string()))
}

/// `a .. b step s`, counted in the unit written.
fn range(text: &str) -> Result<Vec<Level>, String> {
    let (from, rest) = text.split_once("..").expect("a range");
    let (to, step) = rest.split_once("step").ok_or_else(|| format!("sweep: the range '{}' needs a step: `{} .. {} step <step>`", text.trim(), from.trim(), rest.trim()))?;
    let ((from, unit), (to, to_unit), (step, step_unit)) = (split_number(from)?, split_number(to)?, split_number(step)?);
    if to_unit != unit || step_unit != unit {
        return Err(format!("sweep: the range '{}' mixes units: write its bounds and step in one unit", text.trim()));
    }
    if step <= 0.0 {
        return Err(format!("sweep: the step of '{}' must be positive", text.trim()));
    }
    if to < from {
        return Err(format!("sweep: the range '{}' ends before it starts", text.trim()));
    }
    let count = ((to - from) / step + 1e-9).floor() as usize + 1;
    if count > MAX_POINTS {
        return Err(format!("sweep: the range '{}' has {count} values, more than {MAX_POINTS}", text.trim()));
    }
    (0..count)
        .map(|i| {
            let value = ((from + step * i as f64) * 1e9).round() / 1e9;
            level(&if unit.is_empty() { format!("{value}") } else { format!("{value} {unit}") })
        })
        .collect()
}

/// One axis: `variable = v1, v2, ...` or `variable = a .. b step s`.
pub fn parse_axis(text: &str, aliases: &BTreeMap<String, String>) -> Result<Axis, String> {
    let (variable, values) = text.split_once('=').ok_or_else(|| format!("sweep: '{text}' is not `parameter = values` (e.g. `body.mass = 1800 kg, 2100 kg`)"))?;
    let variable = resolve(variable.trim(), aliases)?;
    if !is_path(&variable) {
        return Err(format!("sweep: '{variable}' is not a Modelica path (part.parameter)"));
    }
    let levels = if values.contains("..") { range(values)? } else { values.split(',').map(level).collect::<Result<Vec<_>, _>>()? };
    if levels.is_empty() {
        return Err(format!("sweep: '{variable}' has no value"));
    }
    Ok(Axis { variable, levels })
}

/// The axes of a sweep, checked: no parameter twice, a grid of at most [`MAX_POINTS`].
pub fn parse(texts: &[String], aliases: &BTreeMap<String, String>) -> Result<Vec<Axis>, String> {
    let axes = texts.iter().map(|t| parse_axis(t, aliases)).collect::<Result<Vec<_>, _>>()?;
    for (i, axis) in axes.iter().enumerate() {
        if axes[..i].iter().any(|a| a.variable == axis.variable) {
            return Err(format!("sweep: '{}' is swept twice", axis.variable));
        }
    }
    let size = axes.iter().try_fold(1usize, |n, a| n.checked_mul(a.levels.len())).unwrap_or(usize::MAX);
    if size > MAX_POINTS {
        return Err(format!("sweep: {size} points, more than {MAX_POINTS}: sweep fewer values"));
    }
    Ok(axes)
}

/// Every combination of the levels, the first axis varying slowest.
pub fn grid(axes: &[Axis]) -> Vec<Point> {
    axes.iter().fold(vec![Vec::new()], |points, axis| {
        points.iter().flat_map(|point| axis.levels.iter().map(move |level| {
            let mut next = point.clone();
            next.push((axis.variable.clone(), level.clone()));
            next
        })).collect()
    })
}

/// `vehicleBody.ambient=253.15,speedController.target_speed=25`: what `-override` takes.
pub fn override_flag(point: &Point) -> String {
    point.iter().map(|(variable, level)| format!("{variable}={}", level.si)).collect::<Vec<_>>().join(",")
}

/// `ambient = 253.15 K, target_speed = 90 km/h`: a point as a reader names it.
pub fn label(point: &Point) -> String {
    point.iter().map(|(variable, level)| format!("{} = {}", variable.rsplit('.').next().unwrap_or(variable), level.shown)).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aliases() -> BTreeMap<String, String> {
        [("PN-VCU".to_string(), "vehicleControlUnit".to_string())].into_iter().collect()
    }

    #[test]
    fn axes_are_lists_or_ranges_in_si_units() {
        let axis = parse_axis("vehicleBody.ambient = 253.15 K, 263.15 K", &aliases()).unwrap();
        assert_eq!(axis.variable, "vehicleBody.ambient");
        assert_eq!(axis.levels.iter().map(|l| l.si).collect::<Vec<_>>(), vec![253.15, 263.15]);
        let speed = parse_axis("\"PN-VCU\".speedController.target_speed = 80 km/h .. 130 km/h step 25 km/h", &aliases()).unwrap();
        assert_eq!(speed.variable, "vehicleControlUnit.speedController.target_speed");
        assert_eq!(speed.levels.iter().map(|l| l.shown.as_str()).collect::<Vec<_>>(), vec!["80 km/h", "105 km/h", "130 km/h"]);
        assert!((speed.levels[0].si - 80.0 / 3.6).abs() < 1e-9);
        let plain = parse_axis("driveUnit.efficiency = 0.85, 0.9", &aliases()).unwrap();
        assert_eq!(plain.levels[1].si, 0.9);
    }

    #[test]
    fn mistakes_are_told() {
        assert!(parse_axis("ambient 253 K", &aliases()).unwrap_err().contains("parameter = values"));
        assert!(parse_axis("\"PN-NONE\".x = 1", &aliases()).unwrap_err().contains("not a part"));
        assert!(parse_axis("a.b = 1 .. 5", &aliases()).unwrap_err().contains("needs a step"));
        assert!(parse_axis("a.b = 1 K .. 5 s step 1 K", &aliases()).unwrap_err().contains("mixes units"));
        assert!(parse_axis("a.b = 5 .. 1 step 1", &aliases()).unwrap_err().contains("ends before"));
        assert!(parse_axis("a.b = 10 parsecs", &aliases()).unwrap_err().contains("known unit"));
        assert!(parse_axis("a b = 1", &aliases()).unwrap_err().contains("Modelica path"));
        let twice = parse(&["a.b = 1".to_string(), "a.b = 2".to_string()], &aliases()).unwrap_err();
        assert!(twice.contains("twice"));
        let big = parse(&["a.b = 1 .. 20 step 1".to_string(), "a.c = 1 .. 20 step 1".to_string()], &aliases()).unwrap_err();
        assert!(big.contains("400 points"), "{big}");
    }

    #[test]
    fn the_grid_is_every_combination() {
        let axes = parse(&["a.x = 1, 2".to_string(), "b.y = 10 V, 20 V, 30 V".to_string()], &aliases()).unwrap();
        let points = grid(&axes);
        assert_eq!(points.len(), 6);
        assert_eq!(override_flag(&points[0]), "a.x=1,b.y=10");
        assert_eq!(override_flag(&points[5]), "a.x=2,b.y=30");
        assert_eq!(label(&points[4]), "x = 2, y = 20 V");
        assert_eq!(grid(&[]), vec![Vec::<(String, Level)>::new()], "no axis: one run as written");
    }
}
