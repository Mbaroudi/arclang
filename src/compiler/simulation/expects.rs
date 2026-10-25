//! `expects:` of a simulated test case: comparisons over the variables of
//! the simulation result.
//!
//! ```text
//! expects: ["min(batteryPack.soc) >= 0.1", "final(vehicleBody.distance) >= 300 km",
//!           "at(1 h, vehicleBody.speed) > 95 km/h", "mean(driveUnit.power) < 25 kW"]
//! ```
//!
//! A variable is a Modelica name of the result (`a.b.c`, `x[2]`); its first
//! segment may be an ArcLang id or name in quotes (`"PN-BATT".soc`),
//! replaced by the instance it became. A variable is a time series: it is
//! only read through `min`, `max`, `final`, `initial`, `mean` (over time)
//! or `at(t, var)` (interpolated). Numbers may carry a unit (`300 km`);
//! values are compared in SI units and dimension-checked when the unit of
//! the variable is known.

use super::results::{Series, Unit};
use crate::compiler::quantity::{lookup_unit, DimVec, UNITS};
use std::collections::BTreeMap;

pub const FUNCTIONS: [&str; 6] = ["min", "max", "final", "initial", "mean", "at"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl Cmp {
    fn symbol(self) -> &'static str {
        match self {
            Cmp::Lt => "<",
            Cmp::Le => "<=",
            Cmp::Gt => ">",
            Cmp::Ge => ">=",
            Cmp::Eq => "==",
            Cmp::Ne => "!=",
        }
    }

    fn holds(self, a: f64, b: f64) -> bool {
        // Equality within a relative tolerance: results are numerical.
        let close = (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0);
        match self {
            Cmp::Lt => a < b,
            Cmp::Le => a <= b || close,
            Cmp::Gt => a > b,
            Cmp::Ge => a >= b || close,
            Cmp::Eq => close,
            Cmp::Ne => !close,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    /// A number in SI, the unit written (`300 km`) kept for display.
    Number { value: f64, dim: DimVec, unit: Option<String> },
    /// `fn(var)` or `at(t, var)`.
    Call { function: String, at: Option<Box<Term>>, variable: String },
    Neg(Box<Term>),
    Binary { op: char, lhs: Box<Term>, rhs: Box<Term> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expect {
    pub text: String,
    pub lhs: Term,
    pub cmp: Cmp,
    pub rhs: Term,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64, Option<String>),
    Name(String),
    Quoted(String),
    Op(&'static str),
    Open,
    Close,
    Comma,
}

fn unit_at(text: &str) -> Option<&'static str> {
    // The longest unit symbol starting here and ending at a word boundary.
    let mut best: Option<&'static str> = None;
    for unit in UNITS {
        if text.starts_with(unit.symbol) {
            let next = text[unit.symbol.len()..].chars().next();
            let boundary = next.is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
            if boundary && best.is_none_or(|b| unit.symbol.len() > b.len()) {
                best = Some(unit.symbol);
            }
        }
    }
    best
}

fn tokens(text: &str) -> Result<Vec<Token>, String> {
    let mut out = Vec::new();
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut i = 0;
    while i < chars.len() {
        let (at, c) = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || (c == '.' && chars.get(i + 1).is_some_and(|(_, d)| d.is_ascii_digit())) {
            let mut end = i;
            while end < chars.len() && (chars[end].1.is_ascii_digit() || chars[end].1 == '.') {
                end += 1;
            }
            if end < chars.len() && matches!(chars[end].1, 'e' | 'E') {
                let mut exp = end + 1;
                if exp < chars.len() && matches!(chars[exp].1, '+' | '-') {
                    exp += 1;
                }
                if exp < chars.len() && chars[exp].1.is_ascii_digit() {
                    end = exp;
                    while end < chars.len() && chars[end].1.is_ascii_digit() {
                        end += 1;
                    }
                }
            }
            let stop = chars.get(end).map(|(p, _)| *p).unwrap_or(text.len());
            let value: f64 = text[at..stop].parse().map_err(|_| format!("'{}' is not a number", &text[at..stop]))?;
            let rest = text[stop..].trim_start();
            let skipped = text.len() - stop - rest.len();
            let unit = unit_at(rest);
            if let Some(unit) = unit {
                let unit_end = stop + skipped + unit.len();
                while i < chars.len() && chars[i].0 < unit_end {
                    i += 1;
                }
            } else {
                i = end;
            }
            out.push(Token::Number(value, unit.map(str::to_string)));
        } else if c.is_alphabetic() || c == '_' {
            let mut end = i;
            while end < chars.len() && (chars[end].1.is_alphanumeric() || matches!(chars[end].1, '_' | '.' | '[' | ']')) {
                end += 1;
            }
            let stop = chars.get(end).map(|(p, _)| *p).unwrap_or(text.len());
            out.push(Token::Name(text[at..stop].to_string()));
            i = end;
        } else if c == '"' {
            let close = text[at + 1..].find('"').ok_or("unclosed quote")?;
            let quoted = text[at + 1..at + 1 + close].to_string();
            let mut tail_end = at + 1 + close + 1;
            // `"PN-BATT".soc`: the quoted id with the rest of the path.
            let tail: String = text[tail_end..].chars().take_while(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | '[' | ']')).collect();
            tail_end += tail.len();
            out.push(Token::Quoted(format!("{quoted}\u{0}{tail}")));
            while i < chars.len() && chars[i].0 < tail_end {
                i += 1;
            }
        } else {
            let two = text.get(at..at + 2).unwrap_or("");
            let op: &'static str = match (two, c) {
                (">=", _) => ">=",
                ("<=", _) => "<=",
                ("==", _) => "==",
                ("!=", _) => "!=",
                (_, '>') => ">",
                (_, '<') => "<",
                (_, '+') => "+",
                (_, '-') => "-",
                (_, '*') => "*",
                (_, '/') => "/",
                (_, '(') => {
                    out.push(Token::Open);
                    i += 1;
                    continue;
                }
                (_, ')') => {
                    out.push(Token::Close);
                    i += 1;
                    continue;
                }
                (_, ',') => {
                    out.push(Token::Comma);
                    i += 1;
                    continue;
                }
                _ => return Err(format!("unexpected '{c}'")),
            };
            out.push(Token::Op(op));
            i += op.len();
        }
    }
    Ok(out)
}

struct Parser<'a> {
    tokens: Vec<Token>,
    at: usize,
    aliases: &'a BTreeMap<String, String>,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.at).cloned();
        self.at += 1;
        token
    }

    fn sum(&mut self) -> Result<Term, String> {
        let mut lhs = self.product()?;
        while let Some(Token::Op(op @ ("+" | "-"))) = self.peek().cloned() {
            self.at += 1;
            let rhs = self.product()?;
            lhs = Term::Binary { op: op.chars().next().unwrap(), lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }
        Ok(lhs)
    }

    fn product(&mut self) -> Result<Term, String> {
        let mut lhs = self.unary()?;
        while let Some(Token::Op(op @ ("*" | "/"))) = self.peek().cloned() {
            self.at += 1;
            let rhs = self.unary()?;
            lhs = Term::Binary { op: op.chars().next().unwrap(), lhs: Box::new(lhs), rhs: Box::new(rhs) };
        }
        Ok(lhs)
    }

    fn unary(&mut self) -> Result<Term, String> {
        if self.peek() == Some(&Token::Op("-")) {
            self.at += 1;
            return Ok(Term::Neg(Box::new(self.unary()?)));
        }
        self.primary()
    }

    fn variable(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Token::Name(name)) if !FUNCTIONS.contains(&name.as_str()) => Ok(name),
            Some(Token::Quoted(quoted)) => {
                let (element, rest) = quoted.split_once('\u{0}').unwrap_or((&quoted, ""));
                let instance = self.aliases.get(element).ok_or_else(|| format!("\"{element}\" is no simulated element"))?;
                Ok(format!("{instance}{rest}"))
            }
            other => Err(format!("expected a variable, got {}", describe(other.as_ref()))),
        }
    }

    fn primary(&mut self) -> Result<Term, String> {
        match self.next() {
            Some(Token::Number(value, unit)) => Ok(match unit {
                None => Term::Number { value, dim: DimVec::NONE, unit: None },
                Some(symbol) => {
                    let spec = lookup_unit(&symbol).expect("tokenized from the table");
                    Term::Number { value: value * spec.factor, dim: spec.dimension.vector(), unit: Some(symbol) }
                }
            }),
            Some(Token::Open) => {
                let inner = self.sum()?;
                self.expect(Token::Close)?;
                Ok(inner)
            }
            Some(Token::Name(function)) if FUNCTIONS.contains(&function.as_str()) => {
                self.expect(Token::Open)?;
                let at = if function == "at" {
                    let time = self.sum()?;
                    self.expect(Token::Comma)?;
                    Some(Box::new(time))
                } else {
                    None
                };
                let variable = self.variable()?;
                self.expect(Token::Close)?;
                Ok(Term::Call { function, at, variable })
            }
            Some(Token::Name(name)) | Some(Token::Quoted(name)) => Err(format!(
                "'{}' is a time series: read it with {}(...) or at(t, ...)",
                name.replace('\u{0}', ""),
                FUNCTIONS[..5].join("(...), ")
            )),
            other => Err(format!("unexpected {}", describe(other.as_ref()))),
        }
    }

    fn expect(&mut self, token: Token) -> Result<(), String> {
        match self.next() {
            Some(found) if found == token => Ok(()),
            other => Err(format!("expected {}, got {}", describe(Some(&token)), describe(other.as_ref()))),
        }
    }
}

fn describe(token: Option<&Token>) -> String {
    match token {
        None => "the end".to_string(),
        Some(Token::Number(n, _)) => format!("'{n}'"),
        Some(Token::Name(n)) => format!("'{n}'"),
        Some(Token::Quoted(q)) => format!("'\"{}'", q.replace('\u{0}', "\"")),
        Some(Token::Op(op)) => format!("'{op}'"),
        Some(Token::Open) => "'('".to_string(),
        Some(Token::Close) => "')'".to_string(),
        Some(Token::Comma) => "','".to_string(),
    }
}

/// Parse one expectation; quoted ArcLang ids resolve through `aliases`
/// (id or name -> instance path).
pub fn parse(text: &str, aliases: &BTreeMap<String, String>) -> Result<Expect, String> {
    let tokens = tokens(text).map_err(|e| format!("expects \"{text}\": {e}"))?;
    let mut parser = Parser { tokens, at: 0, aliases };
    let wrap = |e: String| format!("expects \"{text}\": {e}");
    let lhs = parser.sum().map_err(wrap)?;
    let cmp = match parser.next() {
        Some(Token::Op(">=")) => Cmp::Ge,
        Some(Token::Op("<=")) => Cmp::Le,
        Some(Token::Op(">")) => Cmp::Gt,
        Some(Token::Op("<")) => Cmp::Lt,
        Some(Token::Op("==")) => Cmp::Eq,
        Some(Token::Op("!=")) => Cmp::Ne,
        other => return Err(wrap(format!("expected a comparison (>=, <=, >, <, ==, !=), got {}", describe(other.as_ref())))),
    };
    let rhs = parser.sum().map_err(wrap)?;
    if let Some(extra) = parser.peek() {
        return Err(wrap(format!("unexpected {} after the comparison", describe(Some(extra)))));
    }
    Ok(Expect { text: text.to_string(), lhs, cmp, rhs })
}

/// The syntax of an expectation, without the model: quoted ids are not
/// resolved yet.
pub fn check_syntax(text: &str) -> Result<(), String> {
    let tokens = tokens(text).map_err(|e| format!("expects \"{text}\": {e}"))?;
    let aliases: BTreeMap<String, String> = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Quoted(q) => q.split_once('\u{0}').map(|(e, _)| (e.to_string(), "x".to_string())),
            _ => None,
        })
        .collect();
    parse(text, &aliases).map(|_| ())
}

/// A value in SI; `dim` is `None` when a variable's unit is unknown.
#[derive(Debug, Clone, Copy)]
struct Value {
    si: f64,
    dim: Option<DimVec>,
}

/// The outcome of one expectation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Evaluated {
    pub expect: String,
    /// `final(vehicleBody.distance) = 287.4 km`.
    pub value: String,
    pub holds: bool,
}

pub struct Context<'a> {
    pub series: &'a Series,
    pub notes: &'a mut Vec<String>,
}

fn variable_unit(context: &Context<'_>, variable: &str) -> Result<(usize, Option<Unit>), String> {
    let column = context.series.column(variable).ok_or_else(|| format!("'{variable}' is not a variable of the simulation result"))?;
    Ok((column, context.series.unit(variable)))
}

fn eval(term: &Term, context: &mut Context<'_>) -> Result<Value, String> {
    match term {
        Term::Number { value, dim, .. } => Ok(Value { si: *value, dim: Some(*dim) }),
        Term::Neg(inner) => eval(inner, context).map(|v| Value { si: -v.si, ..v }),
        Term::Binary { op, lhs, rhs } => {
            let (a, b) = (eval(lhs, context)?, eval(rhs, context)?);
            match op {
                '+' | '-' => {
                    let dim = same(a.dim, b.dim, &format!("'{op}'"))?;
                    Ok(Value { si: if *op == '+' { a.si + b.si } else { a.si - b.si }, dim })
                }
                '*' => Ok(Value { si: a.si * b.si, dim: a.dim.zip(b.dim).map(|(x, y)| x.multiply(y)) }),
                _ => Ok(Value { si: a.si / b.si, dim: a.dim.zip(b.dim).map(|(x, y)| x.divide(y)) }),
            }
        }
        Term::Call { function, at, variable } => {
            let (column, unit) = variable_unit(context, variable)?;
            let series = context.series;
            let raw = match function.as_str() {
                "min" => series.values(column).fold(f64::INFINITY, f64::min),
                "max" => series.values(column).fold(f64::NEG_INFINITY, f64::max),
                "initial" => series.values(column).next().ok_or("the result is empty")?,
                "final" => series.values(column).last().ok_or("the result is empty")?,
                "mean" => series.mean(column).ok_or("the result is empty")?,
                _ => {
                    let time = eval(at.as_deref().expect("at has a time"), context)?;
                    if let Some(dim) = time.dim {
                        if dim != crate::compiler::quantity::Dimension::Time.vector() && !dim.is_dimensionless() {
                            return Err(format!("at(...): the instant is a {}, not a time", dim.label()));
                        }
                    }
                    series.at(column, time.si).ok_or_else(|| format!("at({} s, ...) is outside the simulated time", time.si))?
                }
            };
            Ok(match unit {
                Some(unit) => Value { si: raw * unit.factor, dim: Some(unit.dim) },
                None => {
                    context.notes.push(format!("the unit of '{variable}' is unknown: compared as a value in SI units"));
                    Value { si: raw, dim: None }
                }
            })
        }
    }
}

fn same(a: Option<DimVec>, b: Option<DimVec>, what: &str) -> Result<Option<DimVec>, String> {
    match (a, b) {
        (Some(x), Some(y)) if x != y => Err(format!("{what} between a {} and a {}", x.label(), y.label())),
        (Some(x), _) | (_, Some(x)) => Ok(Some(x)),
        _ => Ok(None),
    }
}

/// The unit a side is best shown in: the one the other side writes.
fn written_unit(term: &Term) -> Option<&str> {
    match term {
        Term::Number { unit, .. } => unit.as_deref(),
        Term::Neg(inner) => written_unit(inner),
        Term::Binary { lhs, rhs, .. } => written_unit(lhs).or_else(|| written_unit(rhs)),
        Term::Call { .. } => None,
    }
}

fn shown(value: Value, unit: Option<&str>) -> String {
    // Four significant digits.
    let number = |n: f64| {
        let decimals = if n == 0.0 { 0 } else { (3 - n.abs().log10().floor() as i32).clamp(0, 12) as usize };
        let text = format!("{n:.decimals$}");
        let text = if text.contains('.') { text.trim_end_matches('0').trim_end_matches('.').to_string() } else { text };
        if text == "-0" { "0".to_string() } else { text }
    };
    match (unit.and_then(lookup_unit), value.dim) {
        (Some(spec), Some(dim)) if spec.dimension.vector() == dim => format!("{} {}", number(value.si / spec.factor), spec.symbol),
        (_, Some(dim)) if !dim.is_dimensionless() => format!("{} {}", number(value.si), dim.unit()),
        _ => number(value.si),
    }
}

fn label(term: &Term) -> String {
    match term {
        Term::Call { function, at: None, variable } => format!("{function}({variable})"),
        Term::Call { variable, .. } => format!("at(..., {variable})"),
        _ => "value".to_string(),
    }
}

pub fn evaluate(expect: &Expect, context: &mut Context<'_>) -> Result<Evaluated, String> {
    let wrap = |e: String| format!("expects \"{}\": {e}", expect.text);
    let lhs = eval(&expect.lhs, context).map_err(wrap)?;
    let rhs = eval(&expect.rhs, context).map_err(wrap)?;
    same(lhs.dim, rhs.dim, &format!("comparing with {}", expect.cmp.symbol())).map_err(wrap)?;
    let holds = expect.cmp.holds(lhs.si, rhs.si);
    let unit = written_unit(&expect.rhs).or_else(|| written_unit(&expect.lhs));
    let value = format!("{} = {}", label(&expect.lhs), shown(lhs, unit));
    Ok(Evaluated { expect: expect.text.clone(), value, holds })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series() -> Series {
        let csv = "\"time\",\"battery.soc\",\"body.distance\",\"body.speed\"\n0,1,0,0\n1800,0.6,150000,27.7\n3600,0.2,300500,27.8\n";
        let mut series = Series::from_csv(csv).unwrap();
        series.set_units(
            [("battery.soc".to_string(), "1".to_string()), ("body.distance".to_string(), "m".to_string()), ("body.speed".to_string(), "m/s".to_string())].into_iter().collect(),
        );
        series
    }

    fn run(text: &str) -> Result<Evaluated, String> {
        let aliases: BTreeMap<String, String> = [("PN-BATT".to_string(), "battery".to_string())].into_iter().collect();
        let expect = parse(text, &aliases)?;
        let series = series();
        let mut notes = Vec::new();
        evaluate(&expect, &mut Context { series: &series, notes: &mut notes })
    }

    #[test]
    fn functions_over_the_result() {
        assert!(run("min(battery.soc) >= 0.1").unwrap().holds);
        assert!(!run("min(battery.soc) >= 0.25").unwrap().holds);
        assert_eq!(run("final(body.distance) >= 300 km").unwrap().value, "final(body.distance) = 300.5 km");
        assert!(run("initial(battery.soc) == 1").unwrap().holds);
        assert!(run("max(body.speed) < 100 km/h").unwrap().holds);
        // Trapezoidal mean: (0.8*1800 + 0.4*1800) / 3600.
        assert_eq!(run("mean(battery.soc) > 0").unwrap().value, "mean(battery.soc) = 0.6");
        assert_eq!(run("at(900 s, battery.soc) > 0").unwrap().value, "at(..., battery.soc) = 0.8");
        assert!(run("at(0.5 h, body.distance) == 150 km").unwrap().holds);
        assert!(run("final(body.distance) - initial(body.distance) > 2 * 150 km").unwrap().holds);
        assert!(run("min(\"PN-BATT\".soc) >= 0.1").unwrap().holds, "an ArcLang id names the instance");
    }

    #[test]
    fn dimensions_are_checked_when_units_are_known() {
        let error = run("final(body.distance) >= 300 s").unwrap_err();
        assert!(error.contains("length") && error.contains("time"), "{error}");
        let error = run("final(body.distance) >= 300").unwrap_err();
        assert!(error.contains("plain number"), "{error}");
        assert!(run("final(nothing.here) > 1").unwrap_err().contains("not a variable"));
    }

    #[test]
    fn syntax_errors_are_clear() {
        assert!(check_syntax("min(battery.soc) >= 0.1").is_ok());
        assert!(check_syntax("min(\"PN-BATT\".soc) >= 0.1").is_ok());
        assert!(check_syntax("battery.soc >= 0.1").unwrap_err().contains("time series"));
        assert!(check_syntax("min(battery.soc)").unwrap_err().contains("comparison"));
        assert!(check_syntax("min(battery.soc >= 1").unwrap_err().contains("expected ')'"));
        assert!(check_syntax("at(battery.soc) > 1").is_err());
    }
}
