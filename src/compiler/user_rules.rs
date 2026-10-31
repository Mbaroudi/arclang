//! Rules the model declares: what its authors want of its elements, checked
//! by the compiler like its own rules.
//!
//! ```arc
//! rule "Every node declares its mass" {
//!     id: "RULE-MASS"
//!     kind: "PhysicalNode"
//!     where: "has(deploys)"
//!     require: "has(mass) and mass <= 50 kg"
//!     severity: "blocker"
//!     message: "a node without a mass cannot be budgeted"
//! }
//! ```
//!
//! `kind` names one kind of the metamodel or a list of them; `where` (if
//! written) chooses among those elements; `require` is what each chosen
//! element must hold. Both are predicates over the element: its attributes
//! by their key (`mass`, `asil`), `name`, `id` and `kind`; numbers with
//! units, compared in SI and dimension-checked; text in quotes; `true` and
//! `false`; `has(key)`, `size(key)` (the items of a list), `count(type)`
//! (the traces of that type from the element) and `count_in(type)` (to it;
//! `count_in(verifies)` counts the test cases verifying a requirement);
//! comparisons, `and`, `or`, `not` and parentheses. A comparison with an
//! attribute the element does not have does not hold.
//!
//! A `blocker` stops the production gate; a `warning` (the default) is a
//! warning of `check` and of the gate; `info` is advice.

use super::ast::{AttributeValue, Model};
use super::elements::{ElementGraph, ElementRecord};
use super::quantity::{lookup_unit, DimVec, UNITS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Blocker,
    Warning,
    Info,
}

impl Level {
    pub fn word(self) -> &'static str {
        match self {
            Level::Blocker => "blocker",
            Level::Warning => "warning",
            Level::Info => "info",
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserRule {
    pub id: String,
    pub name: String,
    pub kinds: Vec<String>,
    pub filter: Option<(String, Pred)>,
    pub require: (String, Pred),
    pub level: Level,
    pub message: String,
}

/// One element that does not hold a rule.
#[derive(Debug, Clone, PartialEq)]
pub struct Violation {
    pub rule: String,
    pub rule_name: String,
    pub level: Level,
    pub element: String,
    pub kind: String,
    pub message: String,
}

impl Violation {
    /// `rule 'RULE-MASS' (blocker): 'PN-BATT': a node without a mass cannot be budgeted`.
    pub fn line(&self) -> String {
        format!("rule '{}' ({}): '{}': {}", self.rule, self.level.word(), self.element, self.message)
    }
}

const RULE_KEYS: [&str; 7] = ["id", "kind", "where", "require", "severity", "message", "description"];

/// The rules of a model, read and checked: known keys, known kinds,
/// predicates that parse. Any fault is an error naming the rule.
pub fn read(model: &Model) -> Result<Vec<UserRule>, String> {
    let kinds: Vec<&str> = super::metamodel::Metamodel::current().kinds.iter().map(|k| k.name).collect();
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for rule in &model.rules {
        let at = |e: String| format!("rule '{}': {e}", rule.name);
        let mut keys: Vec<&String> = rule.attributes.keys().collect();
        keys.sort();
        if let Some(unknown) = keys.iter().find(|k| !RULE_KEYS.contains(&k.as_str())) {
            return Err(at(format!("'{unknown}' is not one of {}", RULE_KEYS.join(", "))));
        }
        let text = |key: &str| rule.attributes.get(key).and_then(|v| v.as_string()).map(str::to_string);
        let id = text("id").unwrap_or_else(|| rule.name.clone());
        if !seen.insert(id.clone()) {
            return Err(format!("two rules are named '{id}'"));
        }
        let wanted: Vec<String> = match rule.attributes.get("kind") {
            Some(AttributeValue::List(items)) => items.iter().filter_map(|v| v.as_string()).map(str::to_string).collect(),
            Some(value) => value.as_string().map(|s| vec![s.to_string()]).unwrap_or_default(),
            None => Vec::new(),
        };
        if wanted.is_empty() {
            return Err(at("it needs kind: the kind of element it applies to (e.g. kind: \"PhysicalNode\")".to_string()));
        }
        if let Some(unknown) = wanted.iter().find(|k| !kinds.contains(&k.as_str())) {
            return Err(at(format!("'{unknown}' is no kind of element; `arclang metamodel` lists them")));
        }
        let require = text("require").ok_or_else(|| at("it needs require: what each element must hold".to_string()))?;
        let parsed = parse(&require).map_err(|e| at(format!("require \"{require}\": {e}")))?;
        let filter = match text("where") {
            Some(w) => Some((w.clone(), parse(&w).map_err(|e| at(format!("where \"{w}\": {e}")))?)),
            None => None,
        };
        let level = match text("severity").as_deref() {
            None | Some("warning") => Level::Warning,
            Some("blocker") => Level::Blocker,
            Some("info") => Level::Info,
            Some(other) => return Err(at(format!("severity '{other}' is not blocker, warning or info"))),
        };
        let message = text("message").unwrap_or_else(|| format!("does not hold {require}"));
        out.push(UserRule { id, name: rule.name.clone(), kinds: wanted, filter, require: (require, parsed), level, message });
    }
    Ok(out)
}

/// Every element that does not hold a rule. A rule comparing quantities of
/// two dimensions is an error: it can hold for no element.
pub fn evaluate(rules: &[UserRule], graph: &ElementGraph) -> Result<Vec<Violation>, String> {
    // Links, exchanges and other relationships are judged like elements: by their name and attributes.
    let relationships: Vec<ElementRecord> = graph
        .relationships
        .iter()
        .map(|r| ElementRecord {
            uuid: r.uuid.clone(),
            id: r.attributes.get("id").and_then(|v| v.as_string()).map(str::to_string).or_else(|| r.name.clone()).unwrap_or_else(|| r.uuid.clone()),
            kind: r.kind,
            name: r.name.clone().unwrap_or_default(),
            owner: None,
            attributes: r.attributes.clone(),
            extra: r.extra.clone(),
        })
        .collect();
    let mut out = Vec::new();
    for rule in rules {
        for element in graph.elements.iter().chain(relationships.iter()).filter(|e| rule.kinds.iter().any(|k| k == e.kind)) {
            let context = Element { record: element, graph };
            let at = |e: String| format!("rule '{}': {e}", rule.id);
            if let Some((_, filter)) = &rule.filter {
                if !holds(filter, &context).map_err(at)? {
                    continue;
                }
            }
            if !holds(&rule.require.1, &context).map_err(at)? {
                out.push(Violation {
                    rule: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    level: rule.level,
                    element: element.id.clone(),
                    kind: element.kind.to_string(),
                    message: rule.message.clone(),
                });
            }
        }
    }
    Ok(out)
}

/// The elements of a kind (or of every kind, `*`) that hold a predicate,
/// written as a rule's `where:` (`mass > 10 kg and has(voltage)`).
pub fn select<'g>(graph: &'g ElementGraph, kind: &str, predicate: Option<&str>) -> Result<Vec<&'g ElementRecord>, String> {
    let known: Vec<&str> = super::metamodel::Metamodel::current().kinds.iter().map(|k| k.name).collect();
    if kind != "*" && !known.contains(&kind) {
        return Err(format!("'{kind}' is no kind of element (see `arclang metamodel`)"));
    }
    let parsed = predicate.map(parse).transpose()?;
    let mut out = Vec::new();
    for element in graph.elements.iter().filter(|e| kind == "*" || e.kind == kind) {
        let keep = match &parsed {
            Some(pred) => holds(pred, &Element { record: element, graph })?,
            None => true,
        };
        if keep {
            out.push(element);
        }
    }
    Ok(out)
}

/// The rules of a compiled model and what they find: `None` when it declares none.
pub fn violations(model: &Model, semantic: &super::semantic::SemanticModel) -> Result<Option<Vec<Violation>>, String> {
    if model.rules.is_empty() {
        return Ok(None);
    }
    let rules = read(model)?;
    let graph = super::elements::build(model, semantic);
    evaluate(&rules, &graph).map(Some)
}

// ---- Predicates -------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Pred {
    And(Box<Pred>, Box<Pred>),
    Or(Box<Pred>, Box<Pred>),
    Not(Box<Pred>),
    Compare(Term, Cmp, Term),
    /// A term standing alone: `has(mass)`, `true`, a boolean attribute.
    Truth(Term),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cmp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Number { si: f64, dim: DimVec },
    Text(String),
    Bool(bool),
    Attribute(String),
    Call(String, String),
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64, Option<&'static str>),
    Text(String),
    Word(String),
    Op(&'static str),
    Open,
    Close,
}

fn tokens(text: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || (c == '-' && chars.get(i + 1).is_some_and(char::is_ascii_digit)) {
            let start = i;
            if c == '-' {
                i += 1;
            }
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '_' || ((chars[i] == 'e' || chars[i] == 'E') && chars.get(i + 1).is_some_and(|n| n.is_ascii_digit() || *n == '-' || *n == '+'))) {
                if chars[i] == 'e' || chars[i] == 'E' {
                    i += 1;
                }
                i += 1;
            }
            let number: String = chars[start..i].iter().filter(|c| **c != '_').collect();
            let value: f64 = number.parse().map_err(|_| format!("'{number}' is not a number"))?;
            let mut j = i;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }
            let rest: String = chars[j..].iter().collect();
            let unit = UNITS
                .iter()
                .filter(|u| rest.starts_with(u.symbol) && rest[u.symbol.len()..].chars().next().is_none_or(|n| !(n.is_alphanumeric() || n == '_')))
                .max_by_key(|u| u.symbol.len())
                .map(|u| u.symbol);
            if let Some(symbol) = unit {
                i = j + symbol.chars().count();
            }
            out.push(Token::Number(value, unit));
        } else if c == '"' {
            let end = chars[i + 1..].iter().position(|c| *c == '"').ok_or("a text is not closed")?;
            out.push(Token::Text(chars[i + 1..i + 1 + end].iter().collect()));
            i += end + 2;
        } else if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            out.push(Token::Word(chars[start..i].iter().collect()));
        } else if c == '(' {
            out.push(Token::Open);
            i += 1;
        } else if c == ')' {
            out.push(Token::Close);
            i += 1;
        } else {
            let two: String = chars[i..(i + 2).min(chars.len())].iter().collect();
            let op = ["<=", ">=", "==", "!="].into_iter().find(|op| two == *op).or_else(|| ["<", ">"].into_iter().find(|op| c.to_string() == *op));
            match op {
                Some(op) => {
                    out.push(Token::Op(op));
                    i += op.len();
                }
                None => return Err(format!("unexpected '{c}'")),
            }
        }
    }
    Ok(out)
}

const FUNCTIONS: [&str; 4] = ["has", "size", "count", "count_in"];

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.at).cloned();
        self.at += 1;
        token
    }

    fn word(&self, word: &str) -> bool {
        matches!(self.peek(), Some(Token::Word(w)) if w == word)
    }

    fn or(&mut self) -> Result<Pred, String> {
        let mut left = self.and()?;
        while self.word("or") {
            self.at += 1;
            left = Pred::Or(Box::new(left), Box::new(self.and()?));
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<Pred, String> {
        let mut left = self.not()?;
        while self.word("and") {
            self.at += 1;
            left = Pred::And(Box::new(left), Box::new(self.not()?));
        }
        Ok(left)
    }

    fn not(&mut self) -> Result<Pred, String> {
        if self.word("not") {
            self.at += 1;
            return Ok(Pred::Not(Box::new(self.not()?)));
        }
        // A parenthesis opens a predicate (`(a or b) and c`); a term never needs one.
        if self.peek() == Some(&Token::Open) {
            self.at += 1;
            let inner = self.or()?;
            match self.next() {
                Some(Token::Close) => return Ok(inner),
                _ => return Err("a parenthesis is not closed".to_string()),
            }
        }
        let left = self.term()?;
        let cmp = match self.peek() {
            Some(Token::Op(op)) => match *op {
                "<" => Cmp::Lt,
                "<=" => Cmp::Le,
                ">" => Cmp::Gt,
                ">=" => Cmp::Ge,
                "==" => Cmp::Eq,
                _ => Cmp::Ne,
            },
            _ => return Ok(Pred::Truth(left)),
        };
        self.at += 1;
        Ok(Pred::Compare(left, cmp, self.term()?))
    }

    fn term(&mut self) -> Result<Term, String> {
        match self.next() {
            Some(Token::Number(value, unit)) => Ok(match unit.and_then(lookup_unit) {
                Some(spec) => Term::Number { si: spec.to_canonical(value), dim: spec.dimension.vector() },
                None => Term::Number { si: value, dim: DimVec::NONE },
            }),
            Some(Token::Text(text)) => Ok(Term::Text(text)),
            Some(Token::Word(w)) if w == "true" || w == "false" => Ok(Term::Bool(w == "true")),
            Some(Token::Word(w)) if FUNCTIONS.contains(&w.as_str()) => {
                let argument = match (self.next(), self.next(), self.next()) {
                    (Some(Token::Open), Some(Token::Word(a)), Some(Token::Close)) => a,
                    (Some(Token::Open), Some(Token::Text(a)), Some(Token::Close)) => a,
                    _ => return Err(format!("{w} takes one name: {w}(mass)")),
                };
                Ok(Term::Call(w, argument))
            }
            Some(Token::Word(w)) if ["and", "or", "not"].contains(&w.as_str()) => Err(format!("'{w}' where a value is expected")),
            Some(Token::Word(w)) => Ok(Term::Attribute(w)),
            Some(other) => Err(format!("unexpected {other:?} where a value is expected")),
            None => Err("the predicate ends where a value is expected".to_string()),
        }
    }
}

pub fn parse(text: &str) -> Result<Pred, String> {
    let mut parser = Parser { tokens: tokens(text)?, at: 0 };
    let pred = parser.or()?;
    if let Some(extra) = parser.peek() {
        return Err(format!("unexpected {extra:?} after the predicate"));
    }
    Ok(pred)
}

// ---- Evaluation -------------------------------------------------------------

struct Element<'a> {
    record: &'a ElementRecord,
    graph: &'a ElementGraph,
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Number(f64, DimVec),
    Text(String),
    Bool(bool),
    /// The element has no such attribute.
    Missing,
}

impl Element<'_> {
    fn attribute(&self, key: &str) -> Value {
        match key {
            "name" => return Value::Text(self.record.name.clone()),
            "id" => return Value::Text(self.record.id.clone()),
            "kind" => return Value::Text(self.record.kind.to_string()),
            _ => {}
        }
        match self.record.attributes.get(key) {
            None => Value::Missing,
            Some(AttributeValue::Quantity(q)) => Value::Number(q.canonical(), q.dimension().vector()),
            Some(AttributeValue::Number(n)) => Value::Number(*n, DimVec::NONE),
            Some(AttributeValue::Integer(i)) => Value::Number(*i as f64, DimVec::NONE),
            Some(AttributeValue::Boolean(b)) => Value::Bool(*b),
            Some(other) => Value::Text(other.display()),
        }
    }

    /// The relationships of a type from (or to) the element, test cases verifying it included.
    fn count(&self, kind: &str, incoming: bool) -> usize {
        let me = &self.record.uuid;
        self.graph
            .relationships
            .iter()
            .filter(|r| relationship_type(r).is_some_and(|t| t.eq_ignore_ascii_case(kind)))
            .filter(|r| if incoming { &r.target == me } else { &r.source == me })
            .count()
    }

    fn value(&self, term: &Term) -> Value {
        match term {
            Term::Number { si, dim } => Value::Number(*si, *dim),
            Term::Text(text) => Value::Text(text.clone()),
            Term::Bool(b) => Value::Bool(*b),
            Term::Attribute(key) => self.attribute(key),
            Term::Call(function, argument) => match function.as_str() {
                "has" => Value::Bool(self.attribute(argument) != Value::Missing || self.record.extra.contains_key(argument.as_str())),
                "size" => match self.record.attributes.get(argument) {
                    Some(AttributeValue::List(items)) => Value::Number(items.len() as f64, DimVec::NONE),
                    Some(_) => Value::Number(1.0, DimVec::NONE),
                    None => Value::Number(0.0, DimVec::NONE),
                },
                "count" => Value::Number(self.count(argument, false) as f64, DimVec::NONE),
                _ => Value::Number(self.count(argument, true) as f64, DimVec::NONE),
            },
        }
    }
}

/// What a relationship is counted as: a trace by its type (`satisfies`,
/// `refines`), a test case's `verifies`, any other by its kind (`Deployment`).
fn relationship_type(r: &super::elements::RelationshipRecord) -> Option<&str> {
    match r.kind {
        "Trace" => r.extra.get("traceKind").and_then(|v| v.as_str()),
        "Verification" => Some("verifies"),
        other => Some(other),
    }
}

fn holds(pred: &Pred, element: &Element<'_>) -> Result<bool, String> {
    Ok(match pred {
        Pred::And(a, b) => holds(a, element)? && holds(b, element)?,
        Pred::Or(a, b) => holds(a, element)? || holds(b, element)?,
        Pred::Not(a) => !holds(a, element)?,
        Pred::Truth(term) => match element.value(term) {
            Value::Bool(b) => b,
            Value::Missing => false,
            other => return Err(format!("{term:?} is {}, not true or false: compare it (`{} > 0`)", describe(&other), show(term))),
        },
        Pred::Compare(lhs, cmp, rhs) => compare(element.value(lhs), *cmp, element.value(rhs), lhs, rhs)?,
    })
}

fn show(term: &Term) -> String {
    match term {
        Term::Attribute(key) => key.clone(),
        Term::Call(f, a) => format!("{f}({a})"),
        other => format!("{other:?}"),
    }
}

fn describe(value: &Value) -> String {
    match value {
        Value::Number(_, dim) => format!("a {}", dim.label()),
        Value::Text(_) => "a text".to_string(),
        Value::Bool(_) => "true or false".to_string(),
        Value::Missing => "missing".to_string(),
    }
}

fn compare(a: Value, cmp: Cmp, b: Value, lhs: &Term, rhs: &Term) -> Result<bool, String> {
    let close = |x: f64, y: f64| (x - y).abs() <= 1e-9 * x.abs().max(y.abs()).max(1.0);
    Ok(match (a, b) {
        (Value::Missing, _) | (_, Value::Missing) => false,
        (Value::Number(x, dx), Value::Number(y, dy)) => {
            if dx != dy {
                // A bare number in the model against a quantity in the rule: the element does not hold it.
                if dx.is_dimensionless() || dy.is_dimensionless() {
                    return Ok(false);
                }
                return Err(format!("{} is {} and {} {}: they never compare", show(lhs), dx.label(), show(rhs), dy.label()));
            }
            match cmp {
                Cmp::Lt => x < y && !close(x, y),
                Cmp::Le => x <= y || close(x, y),
                Cmp::Gt => x > y && !close(x, y),
                Cmp::Ge => x >= y || close(x, y),
                Cmp::Eq => close(x, y),
                Cmp::Ne => !close(x, y),
            }
        }
        (Value::Text(x), Value::Text(y)) => match cmp {
            Cmp::Eq => x == y,
            Cmp::Ne => x != y,
            _ => return Err(format!("texts compare with == and != only ({} against {})", show(lhs), show(rhs))),
        },
        (Value::Bool(x), Value::Bool(y)) => match cmp {
            Cmp::Eq => x == y,
            Cmp::Ne => x != y,
            _ => return Err("true and false compare with == and != only".to_string()),
        },
        (x, y) => return Err(format!("{} is {} and {} is {}: they never compare", show(lhs), describe(&x), show(rhs), describe(&y))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicates_parse_with_precedence_units_and_functions() {
        let p = parse("has(mass) and mass <= 50 kg or not (asil == \"D\")").unwrap();
        assert!(matches!(p, Pred::Or(_, _)), "{p:?}");
        let Pred::Compare(_, Cmp::Le, Term::Number { si, dim }) = parse("mass <= 50 kg").unwrap() else { panic!() };
        assert_eq!((si, dim), (50.0, crate::compiler::quantity::Dimension::Mass.vector()));
        let Pred::Compare(_, _, Term::Number { si, .. }) = parse("ambient >= -10 °C").unwrap() else { panic!() };
        assert!((si - 263.15).abs() < 1e-9);
        assert!(matches!(parse("count_in(verifies) >= 1").unwrap(), Pred::Compare(Term::Call(f, a), Cmp::Ge, _) if f == "count_in" && a == "verifies"));
        assert!(parse("mass <=").is_err());
        assert!(parse("has(mass").is_err());
        assert!(parse("mass @ 3").is_err());
    }
}
