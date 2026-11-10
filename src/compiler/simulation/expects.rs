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

    /// Whether `a` compares to `b`, two values equal when they are within
    /// `epsilon` of each other (results are numerical). A value that is not
    /// a finite number compares with nothing.
    fn holds(self, a: f64, b: f64, epsilon: f64) -> bool {
        if !a.is_finite() || !b.is_finite() {
            return false;
        }
        let close = a == b || (a - b).abs() <= epsilon;
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
    /// The comparison as written, without its requirement.
    pub text: String,
    /// The requirement it checks (`REQ-SHED: ...`); `None`: every requirement the case verifies.
    pub requirement: Option<String>,
    pub lhs: Term,
    /// `== 120 m within 1 %` / `within 0.5 m`: how far equal may be, relative
    /// (a percentage of the right side) or absolute; `None`: numerically equal.
    pub tolerance: Option<Term>,
    pub cmp: Cmp,
    pub rhs: Term,
}

/// A token of an expectation, and of the action language
/// (`execution::expr`), which shares the lexer: numbers with their unit,
/// names, quoted ids, operators.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
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

pub(crate) fn tokens(text: &str) -> Result<Vec<Token>, String> {
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
                (":=", _) => ":=",
                ("&&", _) => "&&",
                ("||", _) => "||",
                ("<=", _) => "<=",
                ("==", _) => "==",
                ("!=", _) => "!=",
                (_, '>') => ">",
                (_, '<') => "<",
                (_, '+') => "+",
                (_, '-') => "-",
                (_, '*') => "*",
                (_, '/') => "/",
                (_, '=') => "=",
                (_, '!') => "!",
                (_, ';') => ";",
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
    /// A bare variable stands for its value (a requirement's constraint), not a time series.
    values: bool,
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
                    Term::Number { value: spec.to_canonical(value), dim: spec.dimension.vector(), unit: Some(symbol) }
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
            Some(Token::Name(_)) | Some(Token::Quoted(_)) if self.values => {
                self.at -= 1;
                Ok(Term::Call { function: "value".to_string(), at: None, variable: self.variable()? })
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
/// `REQ-SHED: at(40 min, x) <= 1 W`: the requirement an expectation checks,
/// then the comparison. A comparison holds no colon, so one names a requirement.
pub fn split_requirement(text: &str) -> (Option<&str>, &str) {
    match text.split_once(':') {
        Some((requirement, rest)) if !requirement.trim().is_empty() && !requirement.contains('"') => (Some(requirement.trim()), rest.trim()),
        _ => (None, text),
    }
}

pub fn parse(full: &str, aliases: &BTreeMap<String, String>) -> Result<Expect, String> {
    parse_with(full, aliases, false)
}

fn parse_with(full: &str, aliases: &BTreeMap<String, String>, values: bool) -> Result<Expect, String> {
    let (requirement, text) = split_requirement(full);
    let tokens = tokens(text).map_err(|e| format!("expects \"{text}\": {e}"))?;
    let mut parser = Parser { tokens, at: 0, aliases, values };
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
    let tolerance = if parser.peek() == Some(&Token::Name("within".to_string())) {
        parser.at += 1;
        if !matches!(cmp, Cmp::Eq | Cmp::Ne) {
            return Err(wrap("`within` says how far equal may be: it goes with == or !=".to_string()));
        }
        Some(parser.sum().map_err(wrap)?)
    } else {
        None
    };
    if let Some(extra) = parser.peek() {
        return Err(wrap(format!("unexpected {} after the comparison", describe(Some(extra)))));
    }
    Ok(Expect { text: text.to_string(), requirement: requirement.map(str::to_string), lhs, cmp, rhs, tolerance })
}

/// The syntax of an expectation, without the model: quoted ids are not
/// resolved yet.
pub fn check_syntax(full: &str) -> Result<(), String> {
    syntax(full, false)
}

fn syntax(full: &str, values: bool) -> Result<(), String> {
    let text = split_requirement(full).1;
    let tokens = tokens(text).map_err(|e| format!("expects \"{text}\": {e}"))?;
    let aliases: BTreeMap<String, String> = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Quoted(q) => q.split_once('\u{0}').map(|(e, _)| (e.to_string(), "x".to_string())),
            _ => None,
        })
        .collect();
    parse_with(full, &aliases, values).map(|_| ())
}

/// A requirement's constraint (`require:`, `assume:`): a comparison written as an
/// expectation is (`mass <= 1200 kg`, `max(cabin.T) <= 30 °C`), its variables the subject's
/// features. Its syntax and units are checked; it is not evaluated here.
pub fn check_constraint(text: &str) -> Result<(), String> {
    syntax(text, true).map_err(|e| e.trim_start_matches(&format!("expects \"{text}\": ")).to_string())
}

/// A value in SI; `dim` is `None` when a variable's unit is unknown.
#[derive(Debug, Clone, Copy)]
struct Value {
    si: f64,
    dim: Option<DimVec>,
    /// The magnitude of what the value was computed from, in SI: the largest
    /// level its signal reached, the terms of a difference. Two values are
    /// numerically equal relative to it, never to an absolute amount: a
    /// nanosecond is not zero, the residue of 12 V - 12 V is.
    scale: f64,
}

impl Value {
    /// A value that stands for itself.
    fn of(si: f64, dim: Option<DimVec>) -> Value {
        Value { si, dim, scale: si.abs() }
    }
}

/// Two values are numerically equal within this fraction of their scale.
const RELATIVE_TOLERANCE: f64 = 1e-9;

/// The outcome of one expectation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Evaluated {
    pub expect: String,
    /// `final(vehicleBody.distance) = 287.4 km`.
    pub value: String,
    pub holds: bool,
    /// How far the value is from its bound: `short by 0.07341`,
    /// `19.9 km to spare`. Empty for an equality that holds.
    #[serde(default)]
    pub margin: String,
    /// The requirement the expectation checks, when it names one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirement: Option<String>,
    /// The margin as a number, in SI units: positive on the side that holds
    /// (`final(d) >= 300 km` at 320 km: 20000). `None` for an equality.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slack: Option<f64>,
    /// The unit the margin reads in (the one the expectation is written in):
    /// `slack / factor` is the margin in it. `None` for a plain number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slack_unit: Option<SlackUnit>,
}

/// `km`, 1000: a margin of 20000 (SI) reads 20 km.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SlackUnit {
    pub symbol: String,
    pub factor: f64,
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
        Term::Number { value, dim, .. } => Ok(Value::of(*value, Some(*dim))),
        Term::Neg(inner) => eval(inner, context).map(|v| Value { si: -v.si, ..v }),
        Term::Binary { op, lhs, rhs } => {
            let (a, b) = (eval(lhs, context)?, eval(rhs, context)?);
            match op {
                '+' | '-' => {
                    let dim = same(a.dim, b.dim, &format!("'{op}'"))?;
                    Ok(Value { si: if *op == '+' { a.si + b.si } else { a.si - b.si }, dim, scale: a.scale.max(b.scale) })
                }
                '*' => Ok(Value { si: a.si * b.si, dim: a.dim.zip(b.dim).map(|(x, y)| x.multiply(y)), scale: a.scale * b.scale }),
                _ => {
                    // The divisor counts at its own scale, not at the level it
                    // happens to end on: a signal settling near zero must not
                    // widen the tolerance of the quotient.
                    let si = a.si / b.si;
                    Ok(Value { si, dim: a.dim.zip(b.dim).map(|(x, y)| x.divide(y)), scale: (a.scale / b.scale.max(b.si.abs())).max(si.abs()) })
                }
            }
        }
        Term::Call { function, at, variable } => {
            let (column, unit) = variable_unit(context, variable)?;
            let series = context.series;
            let raw = match function.as_str() {
                // A step that is not a number is not skipped: the extremum is not known.
                "min" => series.values(column).fold(f64::INFINITY, |least, v| if least.is_nan() || v.is_nan() { f64::NAN } else { least.min(v) }),
                "max" => series.values(column).fold(f64::NEG_INFINITY, |most, v| if most.is_nan() || v.is_nan() { f64::NAN } else { most.max(v) }),
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
                    series.at(column, time.si).ok_or_else(|| {
                        let seconds = |t: f64| format!("{} s", (t * 1000.0).round() / 1000.0);
                        let span = |t: Option<f64>| t.map(seconds).unwrap_or_else(|| "?".to_string());
                        format!("at({}, ...) is outside the simulated time ({} to {})", seconds(time.si), span(series.times().next()), span(series.times().last()))
                    })?
                }
            };
            // The scale of a reading is the largest level its signal reached.
            let reached = series.values(column).filter(|v| v.is_finite()).fold(raw.abs(), |most, v| most.max(v.abs()));
            Ok(match unit {
                Some(unit) => Value { si: raw * unit.factor, dim: Some(unit.dim), scale: reached * unit.factor.abs() },
                None => {
                    context.notes.push(format!("the unit of '{variable}' is unknown: compared as a value in SI units"));
                    Value { si: raw, dim: None, scale: reached }
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
        (Some(spec), Some(dim)) if spec.dimension.vector() == dim => format!("{} {}", number(spec.from_canonical(value.si)), spec.symbol),
        (_, Some(dim)) if !dim.is_dimensionless() => format!("{} {}", number(value.si), dim.unit()),
        _ => number(value.si),
    }
}

fn label(term: &Term) -> String {
    match term {
        Term::Call { function, at: None, variable } if function == "value" => variable.clone(),
        Term::Call { function, at: None, variable } => format!("{function}({variable})"),
        Term::Call { variable, at: Some(t), .. } => match t.as_ref() {
            Term::Number { value, dim, unit } => format!("at({}, {variable})", shown(Value::of(*value, Some(*dim)), unit.as_deref())),
            _ => format!("at(..., {variable})"),
        },
        _ => "value".to_string(),
    }
}

pub fn evaluate(expect: &Expect, context: &mut Context<'_>) -> Result<Evaluated, String> {
    let wrap = |e: String| format!("expects \"{}\": {e}", expect.text);
    let lhs = eval(&expect.lhs, context).map_err(wrap)?;
    let rhs = eval(&expect.rhs, context).map_err(wrap)?;
    same(lhs.dim, rhs.dim, &format!("comparing with {}", expect.cmp.symbol())).map_err(wrap)?;
    let within = match &expect.tolerance {
        None => None,
        Some(term) => {
            let tolerance = eval(term, context).map_err(wrap)?;
            if written_unit(term) == Some("%") {
                Some(tolerance.si.abs() * rhs.si.abs())
            } else {
                same(tolerance.dim, rhs.dim, "within").map_err(wrap)?;
                Some(tolerance.si.abs())
            }
        }
    };
    let unit = written_unit(&expect.rhs).or_else(|| written_unit(&expect.lhs));
    let value = format!("{} = {}", label(&expect.lhs), shown(lhs, unit));
    // A run that gave a value that is not a number (it diverged, it divided
    // by zero) shows nothing: no expectation holds on it, not even `!=`.
    if !lhs.si.is_finite() || !rhs.si.is_finite() || within.is_some_and(|w| !w.is_finite()) {
        return Ok(Evaluated { expect: expect.text.clone(), value, holds: false, margin: "not a number".to_string(), requirement: expect.requirement.clone(), slack: None, slack_unit: None });
    }
    // A scale that is not finite gives no tolerance: only equal values are equal.
    let epsilon = Some(RELATIVE_TOLERANCE * lhs.scale.max(rhs.scale)).filter(|e| e.is_finite()).unwrap_or(0.0);
    let equal = Cmp::Eq.holds(lhs.si, rhs.si, epsilon);
    let close = within.is_some_and(|w| (lhs.si - rhs.si).abs() <= w) || equal;
    let holds = match (expect.cmp, within) {
        (Cmp::Eq, Some(_)) => close,
        (Cmp::Ne, Some(_)) => !close,
        _ => expect.cmp.holds(lhs.si, rhs.si, epsilon),
    };
    let gap_si = (lhs.si - rhs.si).abs();
    // A gap is a difference: 4.9 K, never 4.9 °C read as a level.
    let gap_unit = unit.filter(|u| lookup_unit(u).is_none_or(|spec| spec.offset == 0.0)).or(unit.map(|_| "K"));
    let gap = shown(Value::of(gap_si, lhs.dim.or(rhs.dim)), gap_unit);
    // A gap below what four significant digits show: the value sits on its bound.
    let on_bound = gap_si <= 5e-5 * lhs.si.abs().max(rhs.si.abs());
    let margin = match (expect.cmp, holds) {
        (Cmp::Eq, true) if within.is_some() && !equal => format!("off by {gap}, within its tolerance"),
        (Cmp::Eq, true) => String::new(),
        (_, true) if on_bound => "at the bound".to_string(),
        (Cmp::Ne, false) => "equal".to_string(),
        (Cmp::Eq, false) | (Cmp::Ne, true) => format!("off by {gap}"),
        (Cmp::Gt | Cmp::Ge, false) => format!("short by {gap}"),
        (Cmp::Lt | Cmp::Le, false) => format!("over by {gap}"),
        _ => format!("{gap} to spare"),
    };
    let slack = match expect.cmp {
        Cmp::Gt | Cmp::Ge => Some(lhs.si - rhs.si),
        Cmp::Lt | Cmp::Le => Some(rhs.si - lhs.si),
        Cmp::Eq | Cmp::Ne => None,
    };
    let dim = lhs.dim.or(rhs.dim);
    let slack_unit = match (slack, gap_unit.and_then(lookup_unit), dim) {
        (None, _, _) => None,
        (_, Some(spec), _) => Some(SlackUnit { symbol: spec.symbol.to_string(), factor: spec.factor }),
        (_, None, Some(dim)) if !dim.is_dimensionless() => Some(SlackUnit { symbol: dim.unit().to_string(), factor: 1.0 }),
        _ => None,
    };
    Ok(Evaluated { expect: expect.text.clone(), value, holds, margin, requirement: expect.requirement.clone(), slack, slack_unit })
}

/// When an expectation stops holding, in seconds of the run: `None` while it
/// holds. An extremum (`max(cells.T) <= 55 °C`) fails at the first instant
/// the values up to it break the bound; `at(t, x)` fails at `t`; anything
/// else (`final`, `mean`) at the end of the run.
pub fn first_failure(expect: &Expect, series: &Series) -> Option<f64> {
    let mut notes = Vec::new();
    let mut fails = |s: &Series| evaluate(expect, &mut Context { series: s, notes: &mut notes }).map(|e| !e.holds).unwrap_or(false);
    if !fails(series) {
        return None;
    }
    if [&expect.lhs, &expect.rhs].iter().any(|t| uses_extremum(t)) {
        let (mut low, mut high) = (1, series.len());
        while low < high {
            let middle = (low + high) / 2;
            if fails(&series.prefix(middle)) {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        return series.times().nth(low - 1);
    }
    at_time(&expect.lhs).or_else(|| at_time(&expect.rhs)).or_else(|| series.times().last())
}

fn uses_extremum(term: &Term) -> bool {
    match term {
        Term::Call { function, .. } => function == "min" || function == "max",
        Term::Neg(inner) => uses_extremum(inner),
        Term::Binary { lhs, rhs, .. } => uses_extremum(lhs) || uses_extremum(rhs),
        Term::Number { .. } => false,
    }
}

fn at_time(term: &Term) -> Option<f64> {
    match term {
        Term::Call { function, at: Some(at), .. } if function == "at" => match at.as_ref() {
            Term::Number { value, .. } => Some(*value),
            _ => None,
        },
        Term::Neg(inner) => at_time(inner),
        Term::Binary { lhs, rhs, .. } => at_time(lhs).or_else(|| at_time(rhs)),
        _ => None,
    }
}

/// The variables an expectation reads (`cells.T`), as the result names them.
pub fn variables(expect: &Expect) -> Vec<String> {
    fn walk(term: &Term, out: &mut Vec<String>) {
        match term {
            Term::Call { variable, .. } => out.push(variable.clone()),
            Term::Neg(inner) => walk(inner, out),
            Term::Binary { lhs, rhs, .. } => {
                walk(lhs, out);
                walk(rhs, out);
            }
            Term::Number { .. } => {}
        }
    }
    let mut out = Vec::new();
    walk(&expect.lhs, &mut out);
    walk(&expect.rhs, &mut out);
    out
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
        assert!(run("max(body.speed) < 101 km/h").unwrap().holds);
        // Trapezoidal mean: (0.8*1800 + 0.4*1800) / 3600.
        assert_eq!(run("mean(battery.soc) > 0").unwrap().value, "mean(battery.soc) = 0.6");
        assert_eq!(run("at(900 s, battery.soc) > 0").unwrap().value, "at(900 s, battery.soc) = 0.8");
        assert!(run("at(0.5 h, body.distance) == 150 km").unwrap().holds);
        assert!(run("final(body.distance) - initial(body.distance) > 2 * 150 km").unwrap().holds);
        assert!(run("min(\"PN-BATT\".soc) >= 0.1").unwrap().holds, "an ArcLang id names the instance");
    }

    #[test]
    fn an_expectation_may_name_the_requirement_it_checks() {
        let aliases = BTreeMap::new();
        let tagged = parse("REQ-SHED: final(body.distance) >= 300 km", &aliases).unwrap();
        assert_eq!(tagged.requirement.as_deref(), Some("REQ-SHED"));
        assert_eq!(tagged.text, "final(body.distance) >= 300 km");
        assert_eq!(parse("final(body.distance) >= 300 km", &aliases).unwrap().requirement, None);
        assert!(check_syntax("Load shedding: at(40 min, cabin.drawn) <= 1 W").is_ok(), "a requirement by its name");
        assert!(check_syntax("REQ-SHED: at(40 min, cabin.drawn) <=").is_err());
        let evaluated = run("REQ-RANGE: final(body.distance) >= 300 km").unwrap();
        assert_eq!(evaluated.requirement.as_deref(), Some("REQ-RANGE"));
        assert_eq!(evaluated.expect, "final(body.distance) >= 300 km");
    }

    #[test]
    fn celsius_compares_as_a_temperature_level() {
        let aliases = BTreeMap::new();
        let warm = parse("max(cells.T) <= 55 °C", &aliases).unwrap();
        assert!(matches!(warm.rhs, Term::Number { value, .. } if (value - 328.15).abs() < 1e-9), "{:?}", warm.rhs);
        assert_eq!(shown(Value::of(323.3, Some(crate::compiler::quantity::Dimension::Temperature.vector())), Some("°C")), "50.15 °C");
        let mut series = Series::from_csv("time,cells.T\n0,308.15\n1,323.3\n").unwrap();
        series.set_units([("cells.T".to_string(), "K".to_string())].into_iter().collect());
        let mut notes = Vec::new();
        let evaluated = evaluate(&warm, &mut Context { series: &series, notes: &mut notes }).unwrap();
        assert_eq!(evaluated.value, "max(cells.T) = 50.15 °C");
        assert_eq!(evaluated.margin, "4.85 K to spare", "a gap is a difference, in kelvin");
    }

    #[test]
    fn margins_say_how_far_a_value_is_from_its_bound() {
        assert_eq!(run("min(battery.soc) >= 0.25").unwrap().margin, "short by 0.05");
        assert_eq!(run("min(battery.soc) >= 0.1").unwrap().margin, "0.1 to spare");
        assert_eq!(run("final(body.distance) >= 300 km").unwrap().margin, "0.5 km to spare");
        assert_eq!(run("max(body.speed) < 100 km/h").unwrap().margin, "over by 0.08 km/h");
        assert_eq!(run("initial(battery.soc) == 1").unwrap().margin, "");
        assert_eq!(run("initial(battery.soc) == 0.5").unwrap().margin, "off by 0.5");
        // A run that ends on its bound (a terminate() at 300 km) is not "0.00000006 km to spare".
        assert_eq!(run("final(body.distance) >= 300.4999999 km").unwrap().margin, "at the bound");
        assert_eq!(run("final(body.distance) >= 300.6 km").unwrap().margin, "short by 0.1 km");
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

#[cfg(test)]
mod constraint_tests {
    use super::check_constraint;

    #[test]
    fn a_requirement_constraint_is_a_comparison_of_the_subjects_features() {
        for ok in ["mass <= 1200 kg", "max(cabin.T) <= 30 °C", "range >= 400 km", "ambient.T >= -20 °C"] {
            assert!(check_constraint(ok).is_ok(), "{ok}: {:?}", check_constraint(ok));
        }
        assert!(check_constraint("mass").unwrap_err().contains("comparison"));
        assert!(check_constraint("mass <= ").is_err());
    }
}

#[cfg(test)]
mod tolerance_tests {
    use super::{evaluate, parse, Context};
    use crate::compiler::simulation::results::Series;
    use std::collections::BTreeMap;

    fn holds(text: &str) -> (bool, String) {
        let series = Series::from_csv("\"time\",\"x\"\n0,0\n10,0.6933\n").unwrap();
        let expect = parse(text, &BTreeMap::new()).unwrap();
        let evaluated = evaluate(&expect, &mut Context { series: &series, notes: &mut Vec::new() }).unwrap();
        (evaluated.holds, evaluated.margin)
    }

    #[test]
    fn equal_within_a_percentage_or_an_amount() {
        assert!(!holds("at(10 s, x) == 0.69").0, "numerically equal only");
        assert_eq!(holds("at(10 s, x) == 0.69 within 1 %"), (true, "off by 0.0033, within its tolerance".to_string()));
        assert!(!holds("at(10 s, x) == 0.69 within 0.1 %").0);
        assert!(holds("at(10 s, x) == 0.69 within 0.01").0);
        assert!(holds("at(10 s, x) != 0.5 within 1 %").0);
        assert!(parse("final(x) >= 1 within 1 %", &BTreeMap::new()).unwrap_err().contains("goes with == or !="));
    }
}

#[cfg(test)]
mod comparison_tests {
    use super::{evaluate, parse, Context, Evaluated};
    use crate::compiler::simulation::results::Series;
    use std::collections::BTreeMap;

    /// The expectation on a result of one variable `x`, in `unit`.
    fn on(values: &[&str], unit: &str, text: &str) -> Evaluated {
        let rows: String = values.iter().enumerate().map(|(step, value)| format!("{step},{value}\n")).collect();
        let mut series = Series::from_csv(&format!("\"time\",\"x\"\n{rows}")).unwrap();
        series.set_units([("x".to_string(), unit.to_string())].into_iter().collect());
        let expect = parse(text, &BTreeMap::new()).unwrap();
        evaluate(&expect, &mut Context { series: &series, notes: &mut Vec::new() }).unwrap()
    }

    #[test]
    fn a_value_that_is_not_a_number_holds_no_expectation() {
        // A run that diverged at one step: neither its extrema nor its mean are evidence.
        let diverged = ["0", "NaN", "1"];
        for text in ["max(x) <= 5", "min(x) >= -5", "max(x) != 3", "mean(x) <= 5", "at(1 s, x) != 3", "max(x) == 1 within 50 %", "max(x) != 3 within 1 %"] {
            let evaluated = on(&diverged, "1", text);
            assert!(!evaluated.holds, "{text}: {evaluated:?}");
            assert_eq!(evaluated.margin, "not a number", "{text}");
            assert_eq!(evaluated.slack, None, "{text}");
        }
        assert!(!on(&["0", "1", "NaN"], "1", "final(x) != 3").holds);
        // The steps before it still say what they say.
        assert!(on(&diverged, "1", "initial(x) == 0").holds);
    }

    #[test]
    fn an_infinite_value_holds_no_expectation() {
        // Was held: an infinite side made any two values compare as equal.
        for text in ["max(x) <= 5", "max(x) == 5", "max(x) >= 5", "max(x) != 5", "final(x) / initial(x) >= 1"] {
            let evaluated = on(&["0", "inf"], "1", text);
            assert!(!evaluated.holds, "{text}: {evaluated:?}");
            assert_eq!(evaluated.margin, "not a number", "{text}");
        }
        assert!(!on(&["0", "-inf"], "1", "min(x) <= 5").holds);
    }

    #[test]
    fn the_tolerance_of_a_comparison_follows_the_scale_of_what_is_compared() {
        // Was held: two quantities less than 1e-9 apart in SI units compared as equal.
        assert!(!on(&["0", "1.9e-9"], "s", "max(x) <= 1 ns").holds);
        assert!(!on(&["0", "1.5e-9"], "s", "final(x) <= 1 ns").holds);
        assert!(!on(&["0", "1.5e-9"], "s", "final(x) == 1 ns").holds);
        assert!(on(&["0", "1.5e-9"], "s", "final(x) != 1 ns").holds);
        assert!(on(&["0", "0.5e-9"], "s", "final(x) <= 1 ns").holds);
        // Numerical equality still holds, at any scale.
        assert!(on(&["0", "1.0000000000001e-9"], "s", "final(x) == 1 ns").holds);
        assert!(on(&["0", "300500.00000001"], "m", "final(x) <= 300.5 km").holds);
    }

    #[test]
    fn the_noise_of_a_signal_is_read_at_the_scale_of_the_signal() {
        // A voltage that reached 12 V and comes back to zero within the solver's noise.
        let settled = ["0", "12", "1e-12"];
        assert!(on(&settled, "V", "final(x) == 0 V").holds);
        assert!(on(&["0", "12", "-1e-12"], "V", "final(x) >= 0 V").holds);
        assert!(on(&settled, "V", "max(x) - 12 V <= 0 V").holds);
        // A signal that never left that level is not noise: a nanovolt is not zero.
        assert!(!on(&["1e-9", "1e-9"], "V", "final(x) == 0 V").holds);
        assert!(!on(&["1e-9", "1e-9"], "V", "final(x) <= 0 V").holds);
        assert!(on(&["0", "0"], "V", "final(x) == 0 V").holds);
    }

    #[test]
    fn a_divisor_that_settles_near_zero_does_not_widen_the_tolerance() {
        // Was held: the scale of the quotient was 12 V over the picovolt the divisor ended on.
        let settled = ["0", "12", "1e-12"];
        assert!(!on(&settled, "V", "final(x) / final(x) == 5").holds);
        assert!(on(&settled, "V", "final(x) / final(x) == 1").holds);
        // A constant divisor scales the noise of the signal with the signal.
        assert!(on(&settled, "V", "final(x) / 2 == 0 V").holds);
    }
}
