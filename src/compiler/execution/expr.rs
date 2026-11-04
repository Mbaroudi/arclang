//! The action language of a state machine: typed variables, discrete
//! expressions, assignments, difference equations and the blocks of
//! discrete-time data flow. Its semantics is here, once: `Machine` evaluates
//! it, the C generator (`embedded::cexpr`) prints the same tree as C.
//!
//! ```text
//! state_machine "Charger" {
//!     variables: ["in voltage: Real [V]", "out current: Real [A] = 0 A",
//!                 "count: Integer = 0", "const limit: Real [V] = 54.6 V"]
//!     equations: ["error = limit - voltage",
//!                 "current = pid(error, 2 A/V, 0.5 A/(V*s), 0 A*s/V, 0 A, 16 A)"]
//!     transition Bulk -> Absorption { guard: "voltage >= limit" action: "count := count + 1" }
//! }
//! ```
//!
//! - A variable is `[in|out|const] name: Type [unit] = initial`. Types:
//!   `Boolean` (C `bool`), `Integer` (C `int32_t`), `Real` (C `double`),
//!   `Float` (C `float`). The unit, from the unit table, fixes the
//!   dimension and the unit the value is stored in; the initial value is
//!   zero (false) when omitted. `in`: set by the caller every step; `out`
//!   and the others: kept by the machine, reported every step; `const`:
//!   never assigned.
//! - Expressions: `+ - * /`, `< <= > >= == !=`, `and or not` (or `&& || !`),
//!   parentheses, numbers with a unit (`30 km/h`), `true`, `false`, and the
//!   functions `min(a, b)`, `max(a, b)`, `abs(a)`, `saturate(x, lo, hi)`,
//!   `gain(x, k)`.
//! - In `equations:` only, the blocks with a state: `delay(x)` /
//!   `delay(x, initial)` (unit delay), `integrator(x, k)` /
//!   `integrator(x, k, lo, hi)` (forward Euler), `pid(e, kp, ki, kd)` /
//!   `pid(e, kp, ki, kd, lo, hi)` (parallel, forward Euler integral,
//!   backward difference derivative, clamping anti-windup).
//! - A guard is an expression when it parses as one and names declared
//!   variables; an action is code when it is `target := expr` statements
//!   (`;` between them). Anything else is prose, as before: a guard is a
//!   condition the caller sets, an action is reported.
//!
//! Integers saturate (to the range of `int32_t`) instead of overflowing, an
//! integer divided by zero is zero; reals follow IEEE 754 (no contraction).
//! Values of different units of a dimension are converted where they meet:
//! the right side to the unit of the left, a number folded at compile time.

use crate::compiler::quantity::{lookup_unit, DimVec, UnitSpec};
use crate::compiler::simulation::expects::{tokens, Token};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Ty {
    Bool,
    Int,
    Real,
    Float,
}

impl Ty {
    pub fn name(self) -> &'static str {
        match self {
            Ty::Bool => "Boolean",
            Ty::Int => "Integer",
            Ty::Real => "Real",
            Ty::Float => "Float",
        }
    }

    pub fn c(self) -> &'static str {
        match self {
            Ty::Bool => "bool",
            Ty::Int => "int32_t",
            Ty::Real => "double",
            Ty::Float => "float",
        }
    }

    fn numeric(self) -> bool {
        self != Ty::Bool
    }

    fn parse(word: &str) -> Option<Ty> {
        Some(match word {
            "Boolean" => Ty::Bool,
            "Integer" => Ty::Int,
            "Real" => Ty::Real,
            "Float" => Ty::Float,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum Value {
    Bool(bool),
    Int(i32),
    Real(f64),
    Float(f32),
}

impl Value {
    pub fn zero(ty: Ty) -> Value {
        match ty {
            Ty::Bool => Value::Bool(false),
            Ty::Int => Value::Int(0),
            Ty::Real => Value::Real(0.0),
            Ty::Float => Value::Float(0.0),
        }
    }

    pub fn ty(self) -> Ty {
        match self {
            Value::Bool(_) => Ty::Bool,
            Value::Int(_) => Ty::Int,
            Value::Real(_) => Ty::Real,
            Value::Float(_) => Ty::Float,
        }
    }

    /// The value as a number (true is 1): what a result column holds.
    pub fn number(self) -> f64 {
        match self {
            Value::Bool(b) => b as u8 as f64,
            Value::Int(i) => i as f64,
            Value::Real(r) => r,
            Value::Float(f) => f as f64,
        }
    }

    fn truth(self) -> bool {
        matches!(self, Value::Bool(true))
    }

    /// A number written in the model, as a value of this type; `None` when
    /// it is not one (a fraction for an Integer, out of range).
    pub fn of(ty: Ty, number: f64) -> Option<Value> {
        match ty {
            Ty::Bool => None,
            Ty::Int => (number.fract() == 0.0 && number >= i32::MIN as f64 && number <= i32::MAX as f64).then_some(Value::Int(number as i32)),
            Ty::Real => number.is_finite().then_some(Value::Real(number)),
            Ty::Float => (number.is_finite() && number.abs() <= f32::MAX as f64).then_some(Value::Float(number as f32)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Direction {
    In,
    Out,
    Local,
    Const,
}

#[derive(Debug, Clone, Serialize)]
pub struct Variable {
    pub name: String,
    pub ty: Ty,
    pub direction: Direction,
    /// The unit symbol it is stored in, when it has one.
    pub unit: Option<String>,
    #[serde(skip)]
    pub dim: DimVec,
    /// Multiply a stored value by `scale` to get it in SI.
    pub scale: f64,
    pub initial: Value,
    /// The declaration as written.
    pub text: String,
    /// `range lo .. hi`: the values it takes, in its unit (what fixed point
    /// is derived from; the model itself does not clamp to it).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<(f64, f64)>,
    /// `format Q<i>.<f>`: the fixed-point format it is held in, when the
    /// model sets it (integer bits, fraction bits, i + f = 31).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<(i32, i32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Un {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Bin {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

impl Bin {
    fn compares(self) -> bool {
        matches!(self, Bin::Lt | Bin::Le | Bin::Gt | Bin::Ge | Bin::Eq | Bin::Ne)
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Bin::Add => "+",
            Bin::Sub => "-",
            Bin::Mul => "*",
            Bin::Div => "/",
            Bin::Lt => "<",
            Bin::Le => "<=",
            Bin::Gt => ">",
            Bin::Ge => ">=",
            Bin::Eq => "==",
            Bin::Ne => "!=",
            Bin::And => "&&",
            Bin::Or => "||",
        }
    }
}

/// The pure functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Fun {
    Min,
    Max,
    Abs,
    Saturate,
}

/// The blocks with a state (equations only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BlockKind {
    /// `delay(x, initial)`: the input of the previous step.
    Delay,
    /// `integrator(x, k[, lo, hi])`: y = s; s := s + k*T*x (clamped).
    Integrator { limited: bool },
    /// `pid(e, kp, ki, kd[, lo, hi])`.
    Pid { limited: bool },
}

/// A typed expression: every operand of an operator has the operator's
/// type (conversions are explicit `Cast`s), units are converted.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Expr {
    Const(Value),
    Var(usize),
    Unary(Un, Ty, Box<Expr>),
    /// The operator, the type of its operands, the operands.
    Binary(Bin, Ty, Box<Expr>, Box<Expr>),
    Cast(Ty, Box<Expr>),
    /// A pure function over operands of one numeric type.
    Call(Fun, Ty, Vec<Expr>),
    /// The output of a block instance; it computes and updates its state
    /// when evaluated (its arguments are in `Program::blocks`).
    Block(usize),
}

#[derive(Debug, Clone, Serialize)]
pub struct BlockInstance {
    pub kind: BlockKind,
    /// Its arguments, as Reals in SI units (`Delay`: of the input's type and unit).
    pub args: Vec<Expr>,
    /// The type of its output (`Real`, but the input's for `Delay`).
    pub ty: Ty,
    /// Initial output of a delay.
    pub initial: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Assign {
    pub target: usize,
    pub value: Expr,
}

#[derive(Debug, Clone, Serialize)]
pub struct Equation {
    pub text: String,
    pub assign: Assign,
}

/// The variables and the compiled code of one machine.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Program {
    pub variables: Vec<Variable>,
    pub equations: Vec<Equation>,
    pub blocks: Vec<BlockInstance>,
    /// Guard text -> its expression (Boolean).
    pub guards: BTreeMap<String, Expr>,
    /// Action text -> its statements.
    pub actions: BTreeMap<String, Vec<Assign>>,
}

/// The state of a block instance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BlockState {
    /// Delay: the held value; integrator, PID: the integral.
    pub held: Value,
    /// PID: the previous error.
    pub previous: f64,
    /// PID: whether a previous error exists (no derivative kick on the first step).
    pub primed: bool,
}

impl Program {
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    pub fn variable(&self, name: &str) -> Option<usize> {
        self.variables.iter().position(|v| v.name == name)
    }

    pub fn initial_values(&self) -> Vec<Value> {
        self.variables.iter().map(|v| v.initial).collect()
    }

    pub fn initial_blocks(&self) -> Vec<BlockState> {
        self.blocks.iter().map(|b| BlockState { held: if b.kind == BlockKind::Delay { b.initial } else { Value::Real(0.0) }, previous: 0.0, primed: false }).collect()
    }

    pub fn inputs(&self) -> impl Iterator<Item = (usize, &Variable)> {
        self.variables.iter().enumerate().filter(|(_, v)| v.direction == Direction::In)
    }
}

// --- declarations ---------------------------------------------------------------------

fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

const RESERVED: [&str; 14] = ["and", "or", "not", "true", "false", "min", "max", "abs", "saturate", "gain", "delay", "integrator", "pid", "within"];

/// A number with an optional unit (`54.6 V`, `0`, `-3`), in that unit.
fn number_with_unit(text: &str) -> Result<(f64, Option<&'static UnitSpec>), String> {
    let text = text.trim();
    let split = text.find(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E' | '_'))).unwrap_or(text.len());
    // An exponent marker is part of the number only when digits follow it.
    let (number, unit) = text.split_at(split);
    let number: String = number.chars().filter(|c| *c != '_').collect();
    let value: f64 = number.trim().parse().map_err(|_| format!("'{text}' is not a number"))?;
    let unit = unit.trim();
    if unit.is_empty() {
        return Ok((value, None));
    }
    let spec = lookup_unit(unit).ok_or_else(|| format!("'{unit}' is not a unit of the unit table"))?;
    Ok((value, Some(spec)))
}

fn affine(spec: &UnitSpec) -> Result<(), String> {
    if spec.offset != 0.0 {
        return Err(format!("'{}' has an offset from its SI unit: values in it do not add; use {}", spec.symbol, spec.dimension.canonical_unit()));
    }
    Ok(())
}

/// `text` split at the word `word` (with spaces around it): before, after.
fn split_word<'t>(text: &'t str, word: &str) -> (&'t str, Option<&'t str>) {
    let pattern = format!(" {word} ");
    match text.find(&pattern) {
        Some(at) => (text[..at].trim(), Some(text[at + pattern.len()..].trim())),
        None => (text, None),
    }
}

/// `Q<i>.<f>`: a signed 32-bit word, i integer bits, f fraction bits.
fn q_format(written: &str) -> Result<(i32, i32), String> {
    let bad = || format!("format '{written}' is a Q format of a signed 32-bit word, Q<integer bits>.<fraction bits> with the two adding up to 31 (Q6.25)");
    let (i, f) = written.trim().strip_prefix('Q').and_then(|q| q.split_once('.')).ok_or_else(bad)?;
    let (i, f): (i32, i32) = (i.parse().map_err(|_| bad())?, f.parse().map_err(|_| bad())?);
    if i < 0 || f < 0 || i + f != 31 {
        return Err(bad());
    }
    Ok((i, f))
}

/// `[in|out|const] name: Type [unit] [range lo .. hi] [format Qi.f] = initial`.
pub fn declaration(text: &str) -> Result<Variable, String> {
    let what = format!("variable \"{text}\"");
    let mut rest = text.trim();
    let mut direction = Direction::Local;
    for (word, d) in [("in ", Direction::In), ("out ", Direction::Out), ("const ", Direction::Const)] {
        if let Some(after) = rest.strip_prefix(word) {
            direction = d;
            rest = after.trim_start();
        }
    }
    let (name, rest) = rest.split_once(':').ok_or_else(|| format!("{what} reads `[in|out|const] name: Type [unit] = initial` (\"out current: Real [A] = 0 A\")"))?;
    let name = name.trim();
    if !valid_name(name) || RESERVED.contains(&name) {
        return Err(format!("{what}: '{name}' is not a name a variable can have (letters, digits, _; not a keyword or a function)"));
    }
    let (typed, initial) = match rest.split_once('=') {
        Some((t, i)) => (t.trim(), Some(i.trim())),
        None => (rest.trim(), None),
    };
    let (typed, format) = split_word(typed, "format");
    let (typed, range) = split_word(typed, "range");
    let (type_word, unit) = match typed.split_once('[') {
        Some((t, u)) => {
            let u = u.trim().strip_suffix(']').ok_or_else(|| format!("{what}: the unit closes with ]"))?.trim();
            (t.trim(), Some(u))
        }
        None => (typed, None),
    };
    let ty = Ty::parse(type_word).ok_or_else(|| format!("{what}: '{type_word}' is not a type (Boolean, Integer, Real, Float)"))?;
    let spec = match unit {
        None => None,
        Some(u) => {
            let spec = lookup_unit(u).ok_or_else(|| format!("{what}: '{u}' is not a unit of the unit table"))?;
            affine(spec).map_err(|e| format!("{what}: {e}"))?;
            if ty == Ty::Bool {
                return Err(format!("{what}: a Boolean has no unit"));
            }
            Some(spec)
        }
    };
    let initial = match initial {
        None => Value::zero(ty),
        Some(written) => value_in(ty, spec, written).map_err(|e| format!("{what}: {e}"))?,
    };
    let range = match range {
        None => None,
        Some(written) => {
            if ty == Ty::Bool {
                return Err(format!("{what}: a Boolean has no range"));
            }
            let (lo, hi) = written.split_once("..").ok_or_else(|| format!("{what}: a range reads `range <low> .. <high>` (\"range 0 V .. 60 V\")"))?;
            let (lo, hi) = (value_in(ty, spec, lo).map_err(|e| format!("{what}: range: {e}"))?.number(), value_in(ty, spec, hi).map_err(|e| format!("{what}: range: {e}"))?.number());
            if lo > hi {
                return Err(format!("{what}: range {lo} .. {hi}: the low end is above the high end"));
            }
            if direction != Direction::In && !(lo..=hi).contains(&initial.number()) {
                return Err(format!("{what}: the initial value {} is outside its range {lo} .. {hi}", initial.number()));
            }
            Some((lo, hi))
        }
    };
    let format = match format {
        None => None,
        Some(written) => {
            if !matches!(ty, Ty::Real | Ty::Float) {
                return Err(format!("{what}: a fixed-point format is for a Real or a Float, not a {}", ty.name()));
            }
            Some(q_format(written).map_err(|e| format!("{what}: {e}"))?)
        }
    };
    Ok(Variable {
        name: name.to_string(),
        ty,
        direction,
        unit: spec.map(|s| s.symbol.to_string()),
        dim: spec.map(|s| s.dimension.vector()).unwrap_or(DimVec::NONE),
        scale: spec.map(|s| s.factor).unwrap_or(1.0),
        initial,
        text: text.to_string(),
        range,
        format,
    })
}

/// A value written in the model (`true`, `54.6 V`, `3`) for a variable of
/// this type and unit, converted to that unit.
fn value_in(ty: Ty, spec: Option<&'static UnitSpec>, written: &str) -> Result<Value, String> {
    let written = written.trim();
    match (written, ty) {
        ("true" | "false", Ty::Bool) => return Ok(Value::Bool(written == "true")),
        (other, Ty::Bool) => return Err(format!("a Boolean is true or false, not '{other}'")),
        _ => {}
    }
    let (value, written_unit) = number_with_unit(written)?;
    let value = match (written_unit, spec) {
        (Some(w), Some(s)) => {
            affine(w)?;
            if w.dimension.vector() != s.dimension.vector() {
                return Err(format!("{written} is a {}, the variable a {}", w.dimension.label(), s.dimension.label()));
            }
            value * w.factor / s.factor
        }
        (Some(w), None) => return Err(format!("{written} has a unit ({}), the variable none: declare it [{}]", w.symbol, w.symbol)),
        (None, _) => value,
    };
    Value::of(ty, value).ok_or_else(|| format!("{value} is not a value of a {}", ty.name()))
}

impl Variable {
    /// A value written for this variable (an input vector: `speed = 50 km/h`).
    pub fn value(&self, written: &str) -> Result<Value, String> {
        value_in(self.ty, self.unit.as_deref().and_then(lookup_unit), written)
    }
}

// --- parsing --------------------------------------------------------------------------

/// What the parser reads, before types and units.
#[derive(Debug, Clone, PartialEq)]
enum Raw {
    Number(f64, Option<String>),
    Bool(bool),
    Name(String),
    Unary(Un, Box<Raw>),
    Binary(Bin, Box<Raw>, Box<Raw>),
    Call(String, Vec<Raw>),
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}

fn word(token: Option<&Token>, w: &str) -> bool {
    matches!(token, Some(Token::Name(n)) if n == w)
}

impl Parser {
    fn new(text: &str) -> Result<Parser, String> {
        Ok(Parser { tokens: tokens(text)?, at: 0 })
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.at).cloned();
        self.at += 1;
        t
    }

    fn done(&self) -> bool {
        self.at >= self.tokens.len()
    }

    fn or(&mut self) -> Result<Raw, String> {
        let mut lhs = self.and()?;
        while matches!(self.peek(), Some(Token::Op("||"))) || word(self.peek(), "or") {
            self.at += 1;
            lhs = Raw::Binary(Bin::Or, Box::new(lhs), Box::new(self.and()?));
        }
        Ok(lhs)
    }

    fn and(&mut self) -> Result<Raw, String> {
        let mut lhs = self.not()?;
        while matches!(self.peek(), Some(Token::Op("&&"))) || word(self.peek(), "and") {
            self.at += 1;
            lhs = Raw::Binary(Bin::And, Box::new(lhs), Box::new(self.not()?));
        }
        Ok(lhs)
    }

    fn not(&mut self) -> Result<Raw, String> {
        if matches!(self.peek(), Some(Token::Op("!"))) || word(self.peek(), "not") {
            self.at += 1;
            return Ok(Raw::Unary(Un::Not, Box::new(self.not()?)));
        }
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Raw, String> {
        let lhs = self.sum()?;
        let op = match self.peek() {
            Some(Token::Op("<")) => Bin::Lt,
            Some(Token::Op("<=")) => Bin::Le,
            Some(Token::Op(">")) => Bin::Gt,
            Some(Token::Op(">=")) => Bin::Ge,
            Some(Token::Op("==")) => Bin::Eq,
            Some(Token::Op("!=")) => Bin::Ne,
            _ => return Ok(lhs),
        };
        self.at += 1;
        Ok(Raw::Binary(op, Box::new(lhs), Box::new(self.sum()?)))
    }

    fn sum(&mut self) -> Result<Raw, String> {
        let mut lhs = self.product()?;
        loop {
            let op = match self.peek() {
                Some(Token::Op("+")) => Bin::Add,
                Some(Token::Op("-")) => Bin::Sub,
                _ => return Ok(lhs),
            };
            self.at += 1;
            lhs = Raw::Binary(op, Box::new(lhs), Box::new(self.product()?));
        }
    }

    fn product(&mut self) -> Result<Raw, String> {
        let mut lhs = self.unary()?;
        loop {
            let op = match self.peek() {
                Some(Token::Op("*")) => Bin::Mul,
                Some(Token::Op("/")) => Bin::Div,
                _ => return Ok(lhs),
            };
            self.at += 1;
            lhs = Raw::Binary(op, Box::new(lhs), Box::new(self.unary()?));
        }
    }

    fn unary(&mut self) -> Result<Raw, String> {
        if matches!(self.peek(), Some(Token::Op("-"))) {
            self.at += 1;
            return Ok(Raw::Unary(Un::Neg, Box::new(self.unary()?)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Raw, String> {
        match self.next() {
            Some(Token::Number(value, unit)) => Ok(Raw::Number(value, unit)),
            Some(Token::Open) => {
                let inner = self.or()?;
                self.close()?;
                Ok(inner)
            }
            Some(Token::Name(name)) if name == "true" || name == "false" => Ok(Raw::Bool(name == "true")),
            Some(Token::Name(name)) => {
                if self.peek() == Some(&Token::Open) {
                    self.at += 1;
                    let mut args = Vec::new();
                    if self.peek() != Some(&Token::Close) {
                        args.push(self.or()?);
                        while self.peek() == Some(&Token::Comma) {
                            self.at += 1;
                            args.push(self.or()?);
                        }
                    }
                    self.close()?;
                    return Ok(Raw::Call(name, args));
                }
                Ok(Raw::Name(name))
            }
            other => Err(format!("unexpected {}", describe(other.as_ref()))),
        }
    }

    fn close(&mut self) -> Result<(), String> {
        match self.next() {
            Some(Token::Close) => Ok(()),
            other => Err(format!("expected ')', got {}", describe(other.as_ref()))),
        }
    }
}

fn describe(token: Option<&Token>) -> String {
    match token {
        None => "the end".to_string(),
        Some(Token::Number(n, u)) => format!("'{n}{}'", u.as_ref().map(|u| format!(" {u}")).unwrap_or_default()),
        Some(Token::Name(n)) => format!("'{n}'"),
        Some(Token::Quoted(q)) => format!("'\"{}'", q.replace('\u{0}', "\"")),
        Some(Token::Op(op)) => format!("'{op}'"),
        Some(Token::Open) => "'('".to_string(),
        Some(Token::Close) => "')'".to_string(),
        Some(Token::Comma) => "','".to_string(),
    }
}

/// An expression, parsed whole; `None` when the text is not one.
fn parse_expr(text: &str) -> Option<Raw> {
    let mut p = Parser::new(text).ok()?;
    let raw = p.or().ok()?;
    p.done().then_some(raw)
}

/// `a := e; b := f` (or `=`); `None` when the text is not statements.
fn parse_statements(text: &str) -> Option<Vec<(String, Raw)>> {
    let mut p = Parser::new(text).ok()?;
    let mut out = Vec::new();
    loop {
        let target = match p.next() {
            Some(Token::Name(n)) => n,
            _ => return None,
        };
        if !matches!(p.next(), Some(Token::Op(":=" | "="))) {
            return None;
        }
        let value = p.or().ok()?;
        out.push((target, value));
        match p.next() {
            None => return Some(out),
            Some(Token::Op(";")) if p.done() => return Some(out),
            Some(Token::Op(";")) => {}
            _ => return None,
        }
    }
}

fn names(raw: &Raw, out: &mut Vec<String>) {
    match raw {
        Raw::Name(n) => out.push(n.clone()),
        Raw::Unary(_, a) => names(a, out),
        Raw::Binary(_, a, b) => {
            names(a, out);
            names(b, out);
        }
        Raw::Call(_, args) => args.iter().for_each(|a| names(a, out)),
        Raw::Number(..) | Raw::Bool(_) => {}
    }
}

// --- types and units ------------------------------------------------------------------

/// A typed node: its expression, type, dimension, the factor of its unit to
/// SI, and its value when it is a number written in the model.
#[derive(Debug, Clone)]
struct Typed {
    expr: Expr,
    ty: Ty,
    dim: DimVec,
    scale: f64,
    /// A literal (folded when converted): its value in `scale`.
    literal: Option<f64>,
}

struct Checker<'a> {
    variables: &'a [Variable],
    blocks: Option<&'a mut Vec<BlockInstance>>,
    /// Whether a block may appear here (not inside a block's arguments).
    in_block: bool,
}

fn dim_label(dim: DimVec) -> String {
    dim.label()
}

fn literal_ty(value: f64) -> Ty {
    if value.fract() == 0.0 && value.abs() <= i32::MAX as f64 {
        Ty::Int
    } else {
        Ty::Real
    }
}

fn constant(ty: Ty, value: f64) -> Result<Expr, String> {
    Value::of(ty, value).map(Expr::Const).ok_or_else(|| format!("{value} is not a value of a {}", ty.name()))
}

fn cast(t: Typed, ty: Ty) -> Result<Typed, String> {
    if t.ty == ty {
        return Ok(t);
    }
    if let Some(v) = t.literal {
        return Ok(Typed { expr: constant(ty, v)?, ty, ..t });
    }
    Ok(Typed { expr: Expr::Cast(ty, Box::new(t.expr)), ty, ..t })
}

/// `t` in the unit of factor `scale` (same dimension).
fn rescale(t: Typed, scale: f64) -> Result<Typed, String> {
    if t.scale == scale {
        return Ok(t);
    }
    let factor = t.scale / scale;
    if let Some(v) = t.literal {
        let value = v * factor;
        let ty = if t.ty == Ty::Int && value.fract() != 0.0 { Ty::Real } else { t.ty };
        return Ok(Typed { expr: constant(ty, value)?, ty, scale, literal: Some(value), ..t });
    }
    let ty = if t.ty == Ty::Int { Ty::Real } else { t.ty };
    let t = cast(t, ty)?;
    let k = Typed { expr: constant(ty, factor)?, ty, dim: DimVec::NONE, scale: 1.0, literal: Some(factor) };
    Ok(Typed { expr: Expr::Binary(Bin::Mul, ty, Box::new(t.expr), Box::new(k.expr)), ty, dim: t.dim, scale, literal: None })
}

/// The common type of two numeric operands: a number written in the model
/// takes the type of the other side when it is a value of it; else Real
/// over Float over Integer.
fn unify(a: &Typed, b: &Typed) -> Ty {
    if a.ty == b.ty {
        return a.ty;
    }
    if let (Some(v), None) = (a.literal, b.literal) {
        if Value::of(b.ty, v).is_some() {
            return b.ty;
        }
    }
    if let (None, Some(v)) = (a.literal, b.literal) {
        if Value::of(a.ty, v).is_some() {
            return a.ty;
        }
    }
    match (a.ty, b.ty) {
        (Ty::Real, _) | (_, Ty::Real) => Ty::Real,
        (Ty::Float, _) | (_, Ty::Float) => Ty::Float,
        _ => Ty::Int,
    }
}

impl Checker<'_> {
    fn check(&mut self, raw: &Raw) -> Result<Typed, String> {
        match raw {
            Raw::Number(value, unit) => {
                let (scale, dim) = match unit {
                    None => (1.0, DimVec::NONE),
                    Some(u) => {
                        let spec = lookup_unit(u).ok_or_else(|| format!("'{u}' is not a unit"))?;
                        affine(spec)?;
                        (spec.factor, spec.dimension.vector())
                    }
                };
                let ty = literal_ty(*value);
                Ok(Typed { expr: constant(ty, *value)?, ty, dim, scale, literal: Some(*value) })
            }
            Raw::Bool(b) => Ok(Typed { expr: Expr::Const(Value::Bool(*b)), ty: Ty::Bool, dim: DimVec::NONE, scale: 1.0, literal: None }),
            Raw::Name(name) => {
                let i = self.variables.iter().position(|v| &v.name == name).ok_or_else(|| format!("'{name}' is no variable of the machine"))?;
                let v = &self.variables[i];
                Ok(Typed { expr: Expr::Var(i), ty: v.ty, dim: v.dim, scale: v.scale, literal: None })
            }
            Raw::Unary(Un::Not, a) => {
                let a = self.check(a)?;
                if a.ty != Ty::Bool {
                    return Err(format!("`not` of a {}: it takes a Boolean", a.ty.name()));
                }
                Ok(Typed { expr: Expr::Unary(Un::Not, Ty::Bool, Box::new(a.expr)), ..a })
            }
            Raw::Unary(Un::Neg, a) => {
                let a = self.check(a)?;
                if !a.ty.numeric() {
                    return Err("`-` of a Boolean".to_string());
                }
                if let Some(v) = a.literal {
                    return Ok(Typed { expr: constant(a.ty, -v)?, literal: Some(-v), ..a });
                }
                Ok(Typed { expr: Expr::Unary(Un::Neg, a.ty, Box::new(a.expr)), literal: None, ..a })
            }
            Raw::Binary(op, a, b) => self.binary(*op, a, b),
            Raw::Call(name, args) => self.call(name, args),
        }
    }

    /// Two operands of `what` of one dimension, in the unit of the first
    /// (a plain 0 goes with any dimension), of their common type.
    fn same_unit(&mut self, a: Typed, b: Typed, what: &str) -> Result<(Typed, Typed), String> {
        if !a.ty.numeric() || !b.ty.numeric() {
            return Err(format!("{what} between a {} and a {}: it takes numbers", a.ty.name(), b.ty.name()));
        }
        let zero = |t: &Typed| t.literal == Some(0.0) && t.dim.is_dimensionless();
        let (a, b) = if a.dim == b.dim {
            let b = rescale(b, a.scale)?;
            (a, b)
        } else if zero(&b) {
            (a.clone(), Typed { dim: a.dim, scale: a.scale, ..b })
        } else if zero(&a) {
            (Typed { dim: b.dim, scale: b.scale, ..a }, b)
        } else {
            return Err(format!("{what} between a {} and a {}", dim_label(a.dim), dim_label(b.dim)));
        };
        let ty = unify(&a, &b);
        Ok((cast(a, ty)?, cast(b, ty)?))
    }

    fn binary(&mut self, op: Bin, a: &Raw, b: &Raw) -> Result<Typed, String> {
        let (a, b) = (self.check(a)?, self.check(b)?);
        let what = format!("'{}'", op.symbol());
        match op {
            Bin::And | Bin::Or => {
                if a.ty != Ty::Bool || b.ty != Ty::Bool {
                    return Err(format!("{what} between a {} and a {}: it takes Booleans", a.ty.name(), b.ty.name()));
                }
                Ok(Typed { expr: Expr::Binary(op, Ty::Bool, Box::new(a.expr), Box::new(b.expr)), ty: Ty::Bool, dim: DimVec::NONE, scale: 1.0, literal: None })
            }
            Bin::Eq | Bin::Ne if a.ty == Ty::Bool || b.ty == Ty::Bool => {
                if a.ty != b.ty {
                    return Err(format!("{what} between a {} and a {}", a.ty.name(), b.ty.name()));
                }
                Ok(Typed { expr: Expr::Binary(op, Ty::Bool, Box::new(a.expr), Box::new(b.expr)), ty: Ty::Bool, dim: DimVec::NONE, scale: 1.0, literal: None })
            }
            _ if op.compares() => {
                let (a, b) = self.same_unit(a, b, &what)?;
                Ok(Typed { expr: Expr::Binary(op, a.ty, Box::new(a.expr), Box::new(b.expr)), ty: Ty::Bool, dim: DimVec::NONE, scale: 1.0, literal: None })
            }
            Bin::Add | Bin::Sub => {
                let (a, b) = self.same_unit(a, b, &what)?;
                let ty = a.ty;
                Ok(Typed { expr: Expr::Binary(op, ty, Box::new(a.expr), Box::new(b.expr)), ty, dim: a.dim, scale: a.scale, literal: None })
            }
            _ => {
                if !a.ty.numeric() || !b.ty.numeric() {
                    return Err(format!("{what} between a {} and a {}: it takes numbers", a.ty.name(), b.ty.name()));
                }
                let ty = unify(&a, &b);
                let (dim, scale) = if op == Bin::Mul { (a.dim.multiply(b.dim), a.scale * b.scale) } else { (a.dim.divide(b.dim), a.scale / b.scale) };
                let (a, b) = (cast(a, ty)?, cast(b, ty)?);
                Ok(Typed { expr: Expr::Binary(op, ty, Box::new(a.expr), Box::new(b.expr)), ty, dim, scale, literal: None })
            }
        }
    }

    /// An argument in SI units, as a Real.
    fn si(&mut self, raw: &Raw) -> Result<Typed, String> {
        let t = self.check(raw)?;
        if !t.ty.numeric() {
            return Err("a block takes numbers, not a Boolean".to_string());
        }
        let t = rescale(t, 1.0)?;
        cast(t, Ty::Real)
    }

    fn call(&mut self, name: &str, args: &[Raw]) -> Result<Typed, String> {
        let arity = |n: &[usize]| -> Result<(), String> {
            if n.contains(&args.len()) {
                Ok(())
            } else {
                Err(format!("{name}(...) takes {} arguments, not {}", n.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" or "), args.len()))
            }
        };
        match name {
            "min" | "max" => {
                arity(&[2])?;
                let (a, b) = (self.check(&args[0])?, self.check(&args[1])?);
                let (a, b) = self.same_unit(a, b, &format!("{name}(...)"))?;
                let f = if name == "min" { Fun::Min } else { Fun::Max };
                Ok(Typed { expr: Expr::Call(f, a.ty, vec![a.expr, b.expr]), literal: None, ..a })
            }
            "abs" => {
                arity(&[1])?;
                let a = self.check(&args[0])?;
                if !a.ty.numeric() {
                    return Err("abs(...) of a Boolean".to_string());
                }
                Ok(Typed { expr: Expr::Call(Fun::Abs, a.ty, vec![a.expr.clone()]), literal: None, ..a })
            }
            "saturate" => {
                arity(&[3])?;
                let x = self.check(&args[0])?;
                let (lo, hi) = (self.check(&args[1])?, self.check(&args[2])?);
                let (x1, lo) = self.same_unit(x.clone(), lo, "saturate(...)")?;
                let (x2, hi) = self.same_unit(x, hi, "saturate(...)")?;
                let ty = if x1.ty == x2.ty { x1.ty } else { Ty::Real };
                let (x, lo, hi) = (cast(x1, ty)?, cast(lo, ty)?, cast(hi, ty)?);
                if let (Some(l), Some(h)) = (lo.literal, hi.literal) {
                    if l > h {
                        return Err(format!("saturate(...): the lower limit {l} is above the upper {h}"));
                    }
                }
                Ok(Typed { expr: Expr::Call(Fun::Saturate, ty, vec![x.expr.clone(), lo.expr, hi.expr]), literal: None, ..x })
            }
            "gain" => {
                arity(&[2])?;
                self.binary(Bin::Mul, &args[0], &args[1])
            }
            "delay" | "integrator" | "pid" => self.block(name, args, &arity),
            other => Err(format!("'{other}(...)' is no function (min, max, abs, saturate, gain; in equations delay, integrator, pid)")),
        }
    }

    fn block(&mut self, name: &str, args: &[Raw], arity: &dyn Fn(&[usize]) -> Result<(), String>) -> Result<Typed, String> {
        if self.blocks.is_none() {
            return Err(format!("{name}(...) keeps a state from step to step: it is written in equations:, not in a guard or an action"));
        }
        if self.in_block {
            return Err(format!("{name}(...) inside the arguments of a block: give the inner one an equation of its own"));
        }
        self.in_block = true;
        let result = self.block_inner(name, args, arity);
        self.in_block = false;
        result
    }

    fn block_inner(&mut self, name: &str, args: &[Raw], arity: &dyn Fn(&[usize]) -> Result<(), String>) -> Result<Typed, String> {
        let time = crate::compiler::quantity::Dimension::Time.vector();
        let (kind, typed_args, ty, dim, scale, initial) = match name {
            "delay" => {
                arity(&[1, 2])?;
                let x = self.check(&args[0])?;
                let initial = match args.get(1) {
                    None => Value::zero(x.ty),
                    Some(raw) => {
                        let i = self.check(raw)?;
                        let (x2, i) = if x.ty == Ty::Bool || i.ty == Ty::Bool {
                            if x.ty != i.ty {
                                return Err("delay(x, initial): the initial value is not of the type of x".to_string());
                            }
                            (x.clone(), i)
                        } else {
                            self.same_unit(x.clone(), i, "delay(x, initial)")?
                        };
                        if x2.ty != x.ty {
                            return Err(format!("delay(x, initial): the initial value is not a {}", x.ty.name()));
                        }
                        match i.expr {
                            Expr::Const(v) => v,
                            _ => return Err("delay(x, initial): the initial value is a number".to_string()),
                        }
                    }
                };
                (BlockKind::Delay, vec![x.expr.clone()], x.ty, x.dim, x.scale, initial)
            }
            "integrator" => {
                arity(&[2, 4])?;
                let x = self.si(&args[0])?;
                let k = self.si(&args[1])?;
                let dim = x.dim.multiply(k.dim).multiply(time);
                let mut list = vec![x.expr, k.expr];
                for raw in &args[2..] {
                    let limit = self.si(raw)?;
                    if limit.dim != dim && !(limit.literal == Some(0.0) && limit.dim.is_dimensionless()) {
                        return Err(format!("integrator(...): a limit is a {}, the output a {}", dim_label(limit.dim), dim_label(dim)));
                    }
                    list.push(limit.expr);
                }
                (BlockKind::Integrator { limited: args.len() == 4 }, list, Ty::Real, dim, 1.0, Value::Real(0.0))
            }
            _ => {
                arity(&[4, 6])?;
                let e = self.si(&args[0])?;
                let (kp, ki, kd) = (self.si(&args[1])?, self.si(&args[2])?, self.si(&args[3])?);
                let dim = e.dim.multiply(kp.dim);
                if e.dim.multiply(ki.dim).multiply(time) != dim {
                    return Err(format!("pid(...): ki*e*time is a {}, kp*e a {}", dim_label(e.dim.multiply(ki.dim).multiply(time)), dim_label(dim)));
                }
                if e.dim.multiply(kd.dim).divide(time) != dim {
                    return Err(format!("pid(...): kd*e/time is a {}, kp*e a {}", dim_label(e.dim.multiply(kd.dim).divide(time)), dim_label(dim)));
                }
                let mut list = vec![e.expr, kp.expr, ki.expr, kd.expr];
                for raw in &args[4..] {
                    let limit = self.si(raw)?;
                    if limit.dim != dim && !(limit.literal == Some(0.0) && limit.dim.is_dimensionless()) {
                        return Err(format!("pid(...): a limit is a {}, the output a {}", dim_label(limit.dim), dim_label(dim)));
                    }
                    list.push(limit.expr);
                }
                (BlockKind::Pid { limited: args.len() == 6 }, list, Ty::Real, dim, 1.0, Value::Real(0.0))
            }
        };
        let blocks = self.blocks.as_mut().expect("checked");
        blocks.push(BlockInstance { kind, args: typed_args, ty, initial });
        Ok(Typed { expr: Expr::Block(blocks.len() - 1), ty, dim, scale, literal: None })
    }
}

fn assign(variables: &[Variable], target: &str, value: Typed, what: &str) -> Result<Assign, String> {
    let i = variables.iter().position(|v| v.name == target).ok_or_else(|| format!("{what}: '{target}' is no variable of the machine"))?;
    let v = &variables[i];
    match v.direction {
        Direction::In => return Err(format!("{what}: '{target}' is an input (in): the caller sets it, the machine does not")),
        Direction::Const => return Err(format!("{what}: '{target}' is a constant")),
        _ => {}
    }
    let value = if v.ty == Ty::Bool || value.ty == Ty::Bool {
        if v.ty != value.ty {
            return Err(format!("{what}: a {} assigned to {} '{target}'", value.ty.name(), v.ty.name()));
        }
        value
    } else {
        let zero = value.literal == Some(0.0) && value.dim.is_dimensionless();
        if value.dim != v.dim && !zero {
            return Err(format!("{what}: a {} assigned to '{target}', a {}", dim_label(value.dim), dim_label(v.dim)));
        }
        let value = if zero { value } else { rescale(value, v.scale)? };
        if v.ty == Ty::Int && value.ty != Ty::Int {
            return Err(format!("{what}: a {} assigned to Integer '{target}' (an Integer holds whole numbers: declare it Real, or compute in Integers)", value.ty.name()));
        }
        cast(value, v.ty)?
    };
    Ok(Assign { target: i, value: value.expr })
}

/// Read the `variables:` and `equations:` of a machine.
pub fn program(variables: &[String], equations: &[String]) -> Result<Program, String> {
    let mut vars: Vec<Variable> = Vec::new();
    for text in variables {
        let v = declaration(text)?;
        if vars.iter().any(|w| w.name == v.name) {
            return Err(format!("variable '{}' is declared twice", v.name));
        }
        vars.push(v);
    }
    let mut program = Program { variables: vars, ..Program::default() };
    let mut assigned: BTreeMap<usize, String> = BTreeMap::new();
    for text in equations {
        let what = format!("equation \"{text}\"");
        let statements = parse_statements(text).filter(|s| s.len() == 1).ok_or_else(|| format!("{what} reads `target = expression`"))?;
        let (target, raw) = &statements[0];
        let mut blocks = std::mem::take(&mut program.blocks);
        let typed = Checker { variables: &program.variables, blocks: Some(&mut blocks), in_block: false }.check(raw).map_err(|e| format!("{what}: {e}"))?;
        program.blocks = blocks;
        let a = assign(&program.variables, target, typed, &what)?;
        if let Some(other) = assigned.insert(a.target, text.clone()) {
            return Err(format!("{what}: '{target}' is already computed by \"{other}\" (one equation per variable)"));
        }
        program.equations.push(Equation { text: text.clone(), assign: a });
    }
    Ok(program)
}

impl Program {
    /// A guard as an expression; `Ok(None)` when it is prose (it parses as no
    /// expression, or names no variable of the machine).
    pub fn guard(&self, text: &str) -> Result<Option<Expr>, String> {
        let Some(raw) = parse_expr(text) else { return Ok(None) };
        let mut used = Vec::new();
        names(&raw, &mut used);
        if !self.uses_code(&used) {
            return Ok(None);
        }
        let typed = Checker { variables: &self.variables, blocks: None, in_block: false }.check(&raw).map_err(|e| format!("guard \"{text}\": {e}"))?;
        if typed.ty != Ty::Bool {
            return Err(format!("guard \"{text}\" is a {}, not a Boolean", typed.ty.name()));
        }
        Ok(Some(typed.expr))
    }

    /// Whether the names of a text make it code: the machine has variables
    /// and the text names one (a name that is none is then a mistake).
    fn uses_code(&self, used: &[String]) -> bool {
        !self.variables.is_empty() && used.iter().any(|n| self.variable(n).is_some())
    }

    /// An action as statements; `Ok(None)` when it is prose.
    pub fn action(&self, text: &str) -> Result<Option<Vec<Assign>>, String> {
        let Some(statements) = parse_statements(text) else { return Ok(None) };
        let mut used: Vec<String> = statements.iter().map(|(t, _)| t.clone()).collect();
        for (_, raw) in &statements {
            names(raw, &mut used);
        }
        if !self.uses_code(&used) {
            return Ok(None);
        }
        let what = format!("action \"{text}\"");
        statements
            .iter()
            .map(|(target, raw)| {
                let typed = Checker { variables: &self.variables, blocks: None, in_block: false }.check(raw).map_err(|e| format!("{what}: {e}"))?;
                let a = assign(&self.variables, target, typed, &what)?;
                if self.equations.iter().any(|e| e.assign.target == a.target) {
                    return Err(format!("{what}: '{target}' is computed by an equation; an action does not assign it"));
                }
                Ok(a)
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some)
    }
}

// --- evaluation: the semantics --------------------------------------------------------

/// The values a step works on: the variables and the blocks' states.
#[derive(Debug, Clone, Serialize)]
pub struct Env {
    pub values: Vec<Value>,
    pub blocks: Vec<BlockState>,
    /// The step period, in seconds (the `T` of the blocks).
    pub period: f64,
}

impl Env {
    pub fn new(program: &Program, period_us: u64) -> Env {
        Env { values: program.initial_values(), blocks: program.initial_blocks(), period: period_seconds(period_us) }
    }
}

/// `T`, as the C computes it: `(double)PERIOD_US * 1.0e-6`.
pub fn period_seconds(period_us: u64) -> f64 {
    period_us as f64 * 1.0e-6
}

pub fn saturate_i32(v: i64) -> i32 {
    v.clamp(i32::MIN as i64, i32::MAX as i64) as i32
}

fn arith_int(op: Bin, a: i32, b: i32) -> i32 {
    let (a, b) = (a as i64, b as i64);
    saturate_i32(match op {
        Bin::Add => a + b,
        Bin::Sub => a - b,
        Bin::Mul => a * b,
        _ => {
            if b == 0 {
                0
            } else {
                a / b
            }
        }
    })
}

fn compare<T: PartialOrd>(op: Bin, a: T, b: T) -> bool {
    match op {
        Bin::Lt => a < b,
        Bin::Le => a <= b,
        Bin::Gt => a > b,
        Bin::Ge => a >= b,
        Bin::Eq => a == b,
        _ => a != b,
    }
}

fn clamp_f64(x: f64, lo: f64, hi: f64) -> f64 {
    if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    }
}

pub fn eval(program: &Program, env: &mut Env, expr: &Expr) -> Value {
    match expr {
        Expr::Const(v) => *v,
        Expr::Var(i) => env.values[*i],
        Expr::Unary(Un::Not, _, a) => Value::Bool(!eval(program, env, a).truth()),
        Expr::Unary(Un::Neg, _, a) => match eval(program, env, a) {
            Value::Int(i) => Value::Int(saturate_i32(-(i as i64))),
            Value::Real(r) => Value::Real(-r),
            Value::Float(f) => Value::Float(-f),
            b => b,
        },
        Expr::Binary(Bin::And, _, a, b) => Value::Bool(eval(program, env, a).truth() && eval(program, env, b).truth()),
        Expr::Binary(Bin::Or, _, a, b) => Value::Bool(eval(program, env, a).truth() || eval(program, env, b).truth()),
        Expr::Binary(op, _, a, b) => {
            let (x, y) = (eval(program, env, a), eval(program, env, b));
            match (x, y) {
                (Value::Int(x), Value::Int(y)) if op.compares() => Value::Bool(compare(*op, x, y)),
                (Value::Int(x), Value::Int(y)) => Value::Int(arith_int(*op, x, y)),
                (Value::Real(x), Value::Real(y)) if op.compares() => Value::Bool(compare(*op, x, y)),
                (Value::Real(x), Value::Real(y)) => Value::Real(match op {
                    Bin::Add => x + y,
                    Bin::Sub => x - y,
                    Bin::Mul => x * y,
                    _ => x / y,
                }),
                (Value::Float(x), Value::Float(y)) if op.compares() => Value::Bool(compare(*op, x, y)),
                (Value::Float(x), Value::Float(y)) => Value::Float(match op {
                    Bin::Add => x + y,
                    Bin::Sub => x - y,
                    Bin::Mul => x * y,
                    _ => x / y,
                }),
                (Value::Bool(x), Value::Bool(y)) => Value::Bool(if *op == Bin::Eq { x == y } else { x != y }),
                _ => unreachable!("operands of one type"),
            }
        }
        Expr::Cast(ty, a) => {
            let v = eval(program, env, a);
            match (ty, v) {
                (Ty::Real, Value::Int(i)) => Value::Real(i as f64),
                (Ty::Real, Value::Float(f)) => Value::Real(f as f64),
                (Ty::Float, Value::Int(i)) => Value::Float(i as f32),
                (Ty::Float, Value::Real(r)) => Value::Float(r as f32),
                (_, v) => v,
            }
        }
        Expr::Call(f, _, args) => {
            let values: Vec<Value> = args.iter().map(|a| eval(program, env, a)).collect();
            call(*f, &values)
        }
        Expr::Block(b) => block(program, env, *b),
    }
}

fn call(f: Fun, v: &[Value]) -> Value {
    match (f, v) {
        (Fun::Min, [Value::Int(a), Value::Int(b)]) => Value::Int(if a < b { *a } else { *b }),
        (Fun::Max, [Value::Int(a), Value::Int(b)]) => Value::Int(if a > b { *a } else { *b }),
        (Fun::Abs, [Value::Int(a)]) => Value::Int(saturate_i32((*a as i64).abs())),
        (Fun::Saturate, [Value::Int(x), Value::Int(lo), Value::Int(hi)]) => Value::Int(if x < lo { *lo } else if x > hi { *hi } else { *x }),
        (Fun::Min, [Value::Real(a), Value::Real(b)]) => Value::Real(if a < b { *a } else { *b }),
        (Fun::Max, [Value::Real(a), Value::Real(b)]) => Value::Real(if a > b { *a } else { *b }),
        (Fun::Abs, [Value::Real(a)]) => Value::Real(if *a < 0.0 { -a } else { *a }),
        (Fun::Saturate, [Value::Real(x), Value::Real(lo), Value::Real(hi)]) => Value::Real(clamp_f64(*x, *lo, *hi)),
        (Fun::Min, [Value::Float(a), Value::Float(b)]) => Value::Float(if a < b { *a } else { *b }),
        (Fun::Max, [Value::Float(a), Value::Float(b)]) => Value::Float(if a > b { *a } else { *b }),
        (Fun::Abs, [Value::Float(a)]) => Value::Float(if *a < 0.0 { -a } else { *a }),
        (Fun::Saturate, [Value::Float(x), Value::Float(lo), Value::Float(hi)]) => Value::Float(if x < lo { *lo } else if x > hi { *hi } else { *x }),
        _ => unreachable!("typed call"),
    }
}

fn real(v: Value) -> f64 {
    match v {
        Value::Real(r) => r,
        other => other.number(),
    }
}

/// A block's output this step, and its state for the next.
fn block(program: &Program, env: &mut Env, b: usize) -> Value {
    let instance = &program.blocks[b];
    let args: Vec<Value> = instance.args.iter().map(|a| eval(program, env, a)).collect();
    let t = env.period;
    let state = &mut env.blocks[b];
    match instance.kind {
        BlockKind::Delay => {
            let out = state.held;
            state.held = args[0];
            out
        }
        BlockKind::Integrator { limited } => {
            let (x, k) = (real(args[0]), real(args[1]));
            let s = real(state.held);
            let (out, next) = if limited {
                let (lo, hi) = (real(args[2]), real(args[3]));
                (clamp_f64(s, lo, hi), clamp_f64(s + k * t * x, lo, hi))
            } else {
                (s, s + k * t * x)
            };
            state.held = Value::Real(next);
            Value::Real(out)
        }
        BlockKind::Pid { limited } => {
            let (e, kp, ki, kd) = (real(args[0]), real(args[1]), real(args[2]), real(args[3]));
            let integral = real(state.held);
            let previous = if state.primed { state.previous } else { e };
            let raw = kp * e + integral + kd * (e - previous) / t;
            let (out, winding) = if limited {
                let (lo, hi) = (real(args[4]), real(args[5]));
                let out = clamp_f64(raw, lo, hi);
                // Clamping: no integration further into the saturation.
                (out, (raw > hi && e > 0.0) || (raw < lo && e < 0.0))
            } else {
                (raw, false)
            };
            if !winding {
                state.held = Value::Real(integral + ki * t * e);
            }
            state.previous = e;
            state.primed = true;
            Value::Real(out)
        }
    }
}

/// Run statements: each assignment in order.
pub fn execute(program: &Program, env: &mut Env, statements: &[Assign]) {
    for s in statements {
        let v = eval(program, env, &s.value);
        env.values[s.target] = v;
    }
}

/// The equations, in the order written.
pub fn flow(program: &Program, env: &mut Env) {
    for e in &program.equations {
        let v = eval(program, env, &e.assign.value);
        env.values[e.assign.target] = v;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prog(vars: &[&str], eqs: &[&str]) -> Result<Program, String> {
        program(&vars.iter().map(|s| s.to_string()).collect::<Vec<_>>(), &eqs.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn declarations_read_type_unit_and_initial_value() {
        let v = declaration("out current: Real [A] = 1500 mA").unwrap();
        assert_eq!((v.ty, v.direction, v.unit.as_deref(), v.initial), (Ty::Real, Direction::Out, Some("A"), Value::Real(1.5)));
        assert_eq!(declaration("count: Integer").unwrap().initial, Value::Int(0));
        assert_eq!(declaration("armed: Boolean = true").unwrap().initial, Value::Bool(true));
        assert!(declaration("n: Integer = 1.5").unwrap_err().contains("not a value of a Integer"));
        assert!(declaration("t: Real [s] = 3 V").unwrap_err().contains("is a voltage, the variable a time"));
        assert!(declaration("x: Complex").unwrap_err().contains("not a type"));
        assert!(declaration("temp: Real [°C]").unwrap_err().contains("offset"));
    }

    #[test]
    fn declarations_read_a_range_and_a_fixed_point_format() {
        let v = declaration("in voltage: Real [V] range 0 V .. 60000 mV format Q6.25").unwrap();
        assert_eq!((v.range, v.format, v.unit.as_deref()), (Some((0.0, 60.0)), Some((6, 25)), Some("V")));
        let v = declaration("filtered: Real [V] range -1 V .. 60 V = 48 V").unwrap();
        assert_eq!((v.range, v.initial), (Some((-1.0, 60.0)), Value::Real(48.0)));
        assert_eq!(declaration("n: Integer range 0 .. 10 = 3").unwrap().range, Some((0.0, 10.0)));
        assert_eq!(declaration("in v: Real [V]").unwrap().range, None, "no range: none");
        assert!(declaration("x: Real [V] range 5 V .. 1 V").unwrap_err().contains("low end is above"));
        assert!(declaration("x: Real [V] range 0 V .. 1 s").unwrap_err().contains("is a time"));
        assert!(declaration("x: Real [V] range 0 V .. 1 V = 3 V").unwrap_err().contains("outside its range"));
        assert!(declaration("x: Real range 0 1").unwrap_err().contains("range <low> .. <high>"));
        assert!(declaration("x: Real format Q6.20").unwrap_err().contains("adding up to 31"));
        assert!(declaration("n: Integer format Q31.0").unwrap_err().contains("for a Real or a Float"));
        assert!(declaration("b: Boolean range 0 .. 1").unwrap_err().contains("no range"));
    }

    #[test]
    fn guards_and_actions_are_typed_and_unit_checked() {
        let p = prog(&["in speed: Real [km/h]", "limit: Real [m/s] = 10 m/s", "count: Integer", "on: Boolean"], &[]).unwrap();
        assert!(p.guard("speed > limit and not on").unwrap().is_some());
        assert!(p.guard("speed > 30 km/h").unwrap().is_some());
        assert_eq!(p.guard("lead vehicle in range").unwrap(), None, "prose stays prose");
        assert_eq!(p.guard("armed").unwrap(), None, "a name that is no variable, alone: prose");
        assert!(p.guard("speed > 3 s").unwrap_err().contains("between a speed and a time"));
        assert!(p.guard("count + 1").unwrap_err().contains("is a Integer, not a Boolean"));
        assert!(p.guard("speed > lmit").unwrap_err().contains("'lmit' is no variable"));
        assert!(p.action("count := count + 1; on := true").unwrap().is_some());
        assert!(p.action("count := speed").unwrap_err().contains("assigned to 'count'"));
        assert!(p.action("speed := 3 km/h").unwrap_err().contains("is an input"));
        assert_eq!(p.action("close contactors").unwrap(), None);
    }

    #[test]
    fn evaluation_converts_units_and_saturates_integers() {
        let p = prog(&["in speed: Real [km/h]", "out ms: Real [m/s]", "n: Integer = 2147483647"], &["ms = speed"]).unwrap();
        let mut env = Env::new(&p, 10_000);
        env.values[0] = Value::Real(36.0);
        flow(&p, &mut env);
        assert!((env.values[1].number() - 10.0).abs() < 1e-12);
        let inc = p.action("n := n + 1").unwrap().unwrap();
        execute(&p, &mut env, &inc);
        assert_eq!(env.values[2], Value::Int(i32::MAX));
        let div = p.action("n := n / 0").unwrap().unwrap();
        execute(&p, &mut env, &div);
        assert_eq!(env.values[2], Value::Int(0));
    }

    #[test]
    fn blocks_keep_their_state_from_step_to_step() {
        let p = prog(
            &["in u: Real", "out d: Real", "out i: Real", "out c: Real"],
            &["d = delay(u, 5)", "i = integrator(u, 2 Hz, -1, 1)", "c = pid(u, 1, 10 Hz, 0.01 s, -100, 100)"],
        )
        .unwrap();
        assert_eq!(p.blocks.len(), 3);
        let mut env = Env::new(&p, 100_000);
        env.values[0] = Value::Real(1.0);
        flow(&p, &mut env);
        assert_eq!(env.values[1], Value::Real(5.0));
        assert_eq!(env.values[2], Value::Real(0.0));
        // First PID step: no derivative kick, integral 0: kp*e.
        assert_eq!(env.values[3], Value::Real(1.0));
        flow(&p, &mut env);
        assert_eq!(env.values[1], Value::Real(1.0));
        assert!((env.values[2].number() - 0.2).abs() < 1e-12);
        assert!((env.values[3].number() - 2.0).abs() < 1e-12, "{:?}", env.values[3]);
        assert!(prog(&["in u: Real", "out y: Real"], &["y = delay(delay(u))"]).unwrap_err().contains("inside the arguments"));
        let p2 = prog(&["in u: Real", "out y: Real"], &[]).unwrap();
        assert!(p2.action("y := delay(u)").unwrap_err().contains("written in equations"));
        assert!(prog(&["in e: Real [m]", "out y: Real [N]"], &["y = pid(e, 2 N / 1 m, 1 N / 1 m, 1 N / 1 m, 0 N, 0 N)"]).unwrap_err().contains("ki*e*time"));
    }
}
