//! The result of a simulation: the CSV OpenModelica writes
//! (`outputFormat="csv"`), and the units of its variables, read from the
//! `<model>_init.xml` it writes beside it.

use crate::compiler::quantity::DimVec;
use std::collections::BTreeMap;

/// A Modelica unit string, as a factor to SI and a dimension.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Unit {
    pub factor: f64,
    pub dim: DimVec,
}

const fn dim(time: i8, length: i8, mass: i8, current: i8, temperature: i8) -> DimVec {
    DimVec { time, length, mass, current, temperature, information: 0, angle: 0 }
}

/// Unit symbols of Modelica's SI types (unprefixed, `kg` aside).
const SYMBOLS: &[(&str, DimVec)] = &[
    ("1", dim(0, 0, 0, 0, 0)),
    // The plane angle is a dimension of its own in ArcLang (see `quantity.rs`).
    ("rad", DimVec { angle: 1, ..DimVec::NONE }),
    ("s", dim(1, 0, 0, 0, 0)),
    ("m", dim(0, 1, 0, 0, 0)),
    ("g", dim(0, 0, 1, 0, 0)),
    ("A", dim(0, 0, 0, 1, 0)),
    ("K", dim(0, 0, 0, 0, 1)),
    ("Hz", dim(-1, 0, 0, 0, 0)),
    ("N", dim(-2, 1, 1, 0, 0)),
    ("J", dim(-2, 2, 1, 0, 0)),
    ("W", dim(-3, 2, 1, 0, 0)),
    ("V", dim(-3, 2, 1, -1, 0)),
    ("Pa", dim(-2, -1, 1, 0, 0)),
    ("C", dim(1, 0, 0, 1, 0)),
    ("Ohm", dim(-3, 2, 1, -2, 0)),
    ("F", dim(4, -2, -1, 2, 0)),
    ("H", dim(-2, 2, 1, -2, 0)),
    ("S", dim(3, -2, -1, 2, 0)),
    ("Wh", dim(-2, 2, 1, 0, 0)),
    ("h", dim(1, 0, 0, 0, 0)),
    ("min", dim(1, 0, 0, 0, 0)),
];

const PREFIXES: &[(&str, f64)] = &[("G", 1e9), ("M", 1e6), ("k", 1e3), ("m", 1e-3), ("u", 1e-6), ("n", 1e-9)];

fn symbol(text: &str) -> Option<Unit> {
    let extra = match text {
        "Wh" => 3600.0,
        "h" => 3600.0,
        "min" => 60.0,
        "g" => 1e-3,
        _ => 1.0,
    };
    if let Some((_, d)) = SYMBOLS.iter().find(|(s, _)| *s == text) {
        return Some(Unit { factor: extra, dim: *d });
    }
    for (prefix, factor) in PREFIXES {
        if let Some(rest) = text.strip_prefix(prefix) {
            if !rest.is_empty() {
                if let Some(unit) = symbol(rest) {
                    return Some(Unit { factor: unit.factor * factor, dim: unit.dim });
                }
            }
        }
    }
    None
}

fn power(unit: Unit, exponent: i32) -> Unit {
    let mut dim = DimVec::NONE;
    for _ in 0..exponent.abs() {
        dim = if exponent > 0 { dim.multiply(unit.dim) } else { dim.divide(unit.dim) };
    }
    Unit { factor: unit.factor.powi(exponent), dim }
}

/// `m/s2`, `kg.m2`, `N.m`, `J/(kg.K)`, `1`: the Modelica unit syntax
/// (factors joined by `.`, a `/` dividing by the rest, exponents written
/// after the symbol). `None` when a symbol is unknown.
pub fn parse_unit(text: &str) -> Option<Unit> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let (numerator, denominator) = match text.split_once('/') {
        Some((n, d)) => (n, Some(d.trim_start_matches('(').trim_end_matches(')'))),
        None => (text, None),
    };
    let product = |part: &str| -> Option<Unit> {
        let mut out = Unit { factor: 1.0, dim: DimVec::NONE };
        for factor in part.split('.') {
            let split = factor.find(|c: char| c.is_ascii_digit() || c == '-').filter(|&i| i > 0).unwrap_or(factor.len());
            let (name, exponent) = factor.split_at(split);
            let exponent: i32 = if exponent.is_empty() { 1 } else { exponent.parse().ok()? };
            let unit = if name.is_empty() && exponent == 1 { Unit { factor: 1.0, dim: DimVec::NONE } } else { power(symbol(name)?, exponent) };
            out = Unit { factor: out.factor * unit.factor, dim: out.dim.multiply(unit.dim) };
        }
        Some(out)
    };
    let top = if numerator == "1" { Unit { factor: 1.0, dim: DimVec::NONE } } else { product(numerator)? };
    match denominator {
        None => Some(top),
        Some(d) => {
            let bottom = product(d)?;
            Some(Unit { factor: top.factor / bottom.factor, dim: top.dim.divide(bottom.dim) })
        }
    }
}

/// Time series of a simulation, column by column.
#[derive(Debug, Clone, Default)]
pub struct Series {
    names: Vec<String>,
    rows: Vec<Vec<f64>>,
    units: BTreeMap<String, String>,
}

impl Series {
    /// Parse the CSV of OpenModelica: a header of quoted names, then rows
    /// of numbers; `time` is the first column.
    pub fn from_csv(text: &str) -> Result<Series, String> {
        let mut lines = text.lines().filter(|l| !l.trim().is_empty());
        let header = lines.next().ok_or("the result file is empty")?;
        let names: Vec<String> = split_csv(header).into_iter().map(|n| n.trim().trim_matches('"').to_string()).collect();
        if names.first().map(String::as_str) != Some("time") {
            return Err("the result file does not start with a time column".to_string());
        }
        let mut rows = Vec::new();
        for (number, line) in lines.enumerate() {
            let row: Result<Vec<f64>, _> = split_csv(line).iter().map(|v| v.trim().parse::<f64>()).collect();
            let row = row.map_err(|_| format!("line {} of the result file is not numbers", number + 2))?;
            if row.len() != names.len() {
                return Err(format!("line {} of the result file has {} values for {} columns", number + 2, row.len(), names.len()));
            }
            rows.push(row);
        }
        if rows.is_empty() {
            return Err("the result file holds no time step".to_string());
        }
        Ok(Series { names, rows, units: BTreeMap::new() })
    }

    pub fn set_units(&mut self, units: BTreeMap<String, String>) {
        self.units = units;
    }

    pub fn column(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|n| n == name)
    }

    pub fn unit(&self, name: &str) -> Option<Unit> {
        self.units.get(name).and_then(|u| parse_unit(u))
    }

    pub fn values(&self, column: usize) -> impl Iterator<Item = f64> + '_ {
        self.rows.iter().map(move |r| r[column])
    }

    fn times(&self) -> impl Iterator<Item = f64> + '_ {
        self.values(0)
    }

    /// The mean over time (trapezoidal), or the value of a single instant.
    pub fn mean(&self, column: usize) -> Option<f64> {
        let first = self.rows.first()?;
        let last = self.rows.last()?;
        let span = last[0] - first[0];
        if span <= 0.0 {
            return Some(first[column]);
        }
        let area: f64 = self.rows.windows(2).map(|w| (w[1][0] - w[0][0]) * (w[0][column] + w[1][column]) / 2.0).sum();
        Some(area / span)
    }

    /// The value at `time`, linearly interpolated; `None` outside the run.
    pub fn at(&self, column: usize, time: f64) -> Option<f64> {
        let first = self.times().next()?;
        let last = self.times().last()?;
        let tolerance = 1e-9 * last.abs().max(1.0);
        if time < first - tolerance || time > last + tolerance {
            return None;
        }
        // The last row at or before `time`: after an event, the value after it.
        let index = self.rows.iter().rposition(|r| r[0] <= time + tolerance).unwrap_or(0);
        let row = &self.rows[index];
        match self.rows.get(index + 1) {
            Some(next) if next[0] > row[0] => Some(row[column] + (next[column] - row[column]) * (time - row[0]) / (next[0] - row[0])),
            _ => Some(row[column]),
        }
    }
}

fn split_csv(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for c in line.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                current.push(c);
            }
            ',' if !quoted => out.push(std::mem::take(&mut current)),
            _ => current.push(c),
        }
    }
    out.push(current);
    out
}

fn xml_attribute(tag: &str, key: &str) -> Option<String> {
    // `key="v"` or `key = "v"`, the key a whole word (not `aliasVariable`).
    let mut from = 0;
    while let Some(found) = tag[from..].find(key) {
        let start = from + found;
        from = start + key.len();
        if !tag[..start].ends_with(|c: char| c.is_whitespace()) {
            continue;
        }
        let Some(rest) = tag[from..].trim_start().strip_prefix('=') else { continue };
        let Some(value) = rest.trim_start().strip_prefix('"') else { continue };
        let end = value.find('"')?;
        return Some(value[..end].replace("&quot;", "\"").replace("&lt;", "<").replace("&gt;", ">").replace("&apos;", "'").replace("&amp;", "&"));
    }
    None
}

/// The unit of every variable the `<model>_init.xml` declares one for.
pub fn units_from_init_xml(xml: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for block in xml.split("<ScalarVariable").skip(1) {
        let block = block.split("</ScalarVariable>").next().unwrap_or(block);
        let header = block.split('>').next().unwrap_or("");
        let Some(name) = xml_attribute(header, "name") else { continue };
        let typed = block.find("<Real").map(|i| &block[i..]).and_then(|t| t.split('>').next());
        if let Some(unit) = typed.and_then(|t| xml_attribute(t, "unit")).filter(|u| !u.is_empty()) {
            out.insert(name, unit);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modelica_units_have_dimensions() {
        let speed = parse_unit("m/s").unwrap();
        assert_eq!(speed.dim, crate::compiler::quantity::Dimension::Speed.vector());
        let torque = parse_unit("N.m").unwrap();
        assert_eq!(torque.dim, crate::compiler::quantity::Dimension::Energy.vector());
        assert_eq!(parse_unit("kg").unwrap(), Unit { factor: 1.0, dim: dim(0, 0, 1, 0, 0) });
        assert_eq!(parse_unit("km").unwrap().factor, 1000.0);
        assert_eq!(parse_unit("m/s2").unwrap().dim, dim(-2, 1, 0, 0, 0));
        assert_eq!(parse_unit("1").unwrap().dim, DimVec::NONE);
        assert_eq!(parse_unit("rad/s").unwrap().dim, DimVec { time: -1, angle: 1, ..DimVec::NONE });
        assert_eq!(parse_unit("J/(kg.K)").unwrap().dim, dim(-2, 2, 0, 0, -1));
        assert!(parse_unit("furlong").is_none());
    }

    #[test]
    fn csv_and_init_xml() {
        let series = Series::from_csv("\"time\",\"a.x\"\n0,1\n1,3\n1,5\n2,5\n").unwrap();
        let x = series.column("a.x").unwrap();
        assert_eq!(series.at(x, 0.5), Some(2.0));
        assert_eq!(series.at(x, 1.0), Some(5.0), "after the event");
        assert_eq!(series.at(x, 3.0), None);
        assert!(Series::from_csv("\"x\"\n1\n").is_err());
        assert!(Series::from_csv("\"time\",\"x\"\n0,a\n").is_err());
        let xml = r#"<ScalarVariable name = "a" /><ScalarVariable
    name = "body.distance" valueReference="1000" variability="continuous"
    alias = "alias" aliasVariable="body.car.s">
    <Real start="0.0" unit="m" displayUnit="km" />
  </ScalarVariable>
  <ScalarVariable name="flag" valueReference="1">
    <Boolean start="false" />
  </ScalarVariable>"#;
        let units = units_from_init_xml(xml);
        assert_eq!(units.get("body.distance").map(String::as_str), Some("m"));
        assert!(!units.contains_key("flag"));
    }
}
