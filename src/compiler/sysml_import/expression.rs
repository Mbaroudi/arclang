//! SysML expressions as ArcLang expressions: the value of an attribute
//! (`30 [ms]`) or the body of a constraint (`'SF-1'.latency + 'SF-2'.latency
//! <= 50 [ms]`), read from the expression elements of the API records:
//! `OperatorExpression`, `FeatureChainExpression`, `FeatureReferenceExpression`,
//! `InvocationExpression` and the literals.

use super::records::{Element, Graph};
use crate::compiler::quantity::UNITS;
use crate::compiler::source_edit::string_literal;
use std::collections::BTreeSet;

/// What an expression says, in ArcLang's terms.
#[derive(Debug, Clone, PartialEq)]
enum Expr {
    /// A number, string or boolean, rendered.
    Literal(String),
    /// `30 ms`: a number and an ArcLang unit symbol.
    Quantity(String, String),
    /// `"element".attribute`: the attribute of a written element.
    Attr(String, String),
    /// A written element by itself (the chain of `max(chain, attr)`), or a bare name.
    Ref(String),
    /// `[a, b]`: a list of values (`(a, b)` in SysML).
    List(Vec<Expr>),
    /// A unit, as a reference to its definition.
    Unit(String),
    Call(String, Vec<Expr>),
    Neg(Box<Expr>),
    Binary(String, Box<Expr>, Box<Expr>),
}

/// The ArcLang text of the expression element `id`, or why it has none.
/// `refer` gives the written id of an element; `chains` the written id of
/// every functional chain with the written ids of its steps, for the
/// aggregates (`max`, `min`) the export unfolds over a chain's members.
pub(super) fn render(graph: &Graph, id: &str, refer: &dyn Fn(&str) -> Option<String>, chains: &[(String, Vec<String>)]) -> Result<String, String> {
    let expr = read(graph, id, refer, chains, 0)?;
    if let Some(name) = bare_reference(&expr) {
        return Err(format!("'{name}' is a reference, not a value"));
    }
    match &expr {
        Expr::Unit(unit) => Err(format!("'{unit}' is a unit, not a value")),
        _ => Ok(text(&expr, true)),
    }
}

/// A reference standing where ArcLang wants a value (anywhere but as an
/// argument of `max(chain, attribute)`).
fn bare_reference(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Ref(name) => Some(name),
        Expr::Call(_, _) | Expr::Literal(_) | Expr::Quantity(_, _) | Expr::Attr(_, _) | Expr::Unit(_) => None,
        Expr::List(items) => items.iter().find_map(bare_reference),
        Expr::Neg(inner) => bare_reference(inner),
        Expr::Binary(_, lhs, rhs) => bare_reference(lhs).or_else(|| bare_reference(rhs)),
    }
}

/// Whether a rendered value is one ArcLang takes as an attribute value: a
/// literal, a quantity, a list of them.
pub(super) fn is_attribute_value(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed == "true" || trimmed == "false" {
        return true;
    }
    if trimmed.starts_with('"') {
        return is_one_string(trimmed);
    }
    if let Some(inner) = trimmed.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
        return inner.trim().is_empty() || split_top_level(inner).iter().all(|item| is_attribute_value(item));
    }
    // `12`, `-1.5`, `30 ms`
    let mut parts = trimmed.splitn(2, ' ');
    let number = parts.next().unwrap_or("");
    let unit = parts.next().unwrap_or("");
    number.parse::<f64>().is_ok() && (unit.is_empty() || unit.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '%' | '°' | 'µ' | '*' | '^')))
}

/// Whether the text is one string literal and nothing after it.
fn is_one_string(text: &str) -> bool {
    let mut chars = text.chars().skip(1);
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                chars.next();
            }
            '"' => return chars.next().is_none(),
            _ => {}
        }
    }
    false
}

fn split_top_level(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start, mut quoted) = (0i32, 0usize, false);
    for (i, c) in text.char_indices() {
        match c {
            '"' => quoted = !quoted,
            '[' | '(' if !quoted => depth += 1,
            ']' | ')' if !quoted => depth -= 1,
            ',' if !quoted && depth == 0 => {
                out.push(text[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(text[start..].trim());
    out
}

const BINARY: &[&str] = &["+", "-", "*", "/", "<", "<=", ">", ">=", "==", "!="];

fn read(graph: &Graph, id: &str, refer: &dyn Fn(&str) -> Option<String>, chains: &[(String, Vec<String>)], depth: usize) -> Result<Expr, String> {
    if depth > 32 {
        return Err("the expression is too deep".to_string());
    }
    let element = graph.get(id).ok_or_else(|| format!("expression element {id} is missing"))?;
    let operand = |i: usize| -> Result<Expr, String> {
        let argument = element.arguments.get(i).ok_or_else(|| format!("`{}` lacks its operand {i}", element.operator.as_deref().unwrap_or("?")))?;
        read(graph, argument, refer, chains, depth + 1)
    };
    match element.kind.as_str() {
        "LiteralInteger" | "LiteralRational" | "LiteralString" | "LiteralBoolean" => {
            element.value.clone().map(Expr::Literal).ok_or_else(|| "a literal without a value".to_string())
        }
        "FeatureReferenceExpression" => {
            let referent = element.referent.as_deref().ok_or_else(|| "a reference to nothing".to_string())?;
            if let Some(unit) = unit_symbol(graph, referent) {
                return Ok(Expr::Unit(unit));
            }
            let written = refer(referent).ok_or_else(|| format!("'{}' is not a written element", display(graph, referent)))?;
            Ok(Expr::Ref(written))
        }
        "FeatureChainExpression" => {
            // `'SF-1'.latency`: the target feature is the attribute, its
            // owner the element; failing a target, the written path.
            let base = operand(0)?;
            if let Some(target) = element.target_feature.as_deref().and_then(|t| graph.get(t)) {
                let attribute = target
                    .name
                    .clone()
                    .or_else(|| target.redefines.iter().filter_map(|r| graph.get(r)).find_map(|r| r.name.clone()))
                    .ok_or_else(|| "an attribute without a name".to_string())?;
                let owner = target.owner.as_deref().ok_or_else(|| "an attribute owned by nothing".to_string())?;
                // An attribute inherited from a definition (an ArcLang type)
                // is read on the element the chain starts from.
                let inherited = graph.get(owner).is_some_and(|o| o.kind.ends_with("Definition"));
                return match (&base, inherited) {
                    (Expr::Ref(written), true) => Ok(Expr::Attr(written.clone(), attribute)),
                    _ => {
                        let written = refer(owner).ok_or_else(|| format!("'{}' is not a written element", display(graph, owner)))?;
                        Ok(Expr::Attr(written, attribute))
                    }
                };
            }
            let path = element.target_path.clone().ok_or_else(|| "a feature chain without a target".to_string())?;
            match (base, path.rsplit_once('.')) {
                (Expr::Ref(written), None) => Ok(Expr::Attr(written, path)),
                (Expr::Ref(_), Some((nested, _))) => Err(format!("the path '{nested}' is not resolved")),
                _ => Err("a feature chain on a value".to_string()),
            }
        }
        "OperatorExpression" => {
            let operator = element.operator.as_deref().ok_or_else(|| "an operator expression without an operator".to_string())?;
            match (operator, element.arguments.len()) {
                ("[", 2) => match (operand(0)?, operand(1)?) {
                    (Expr::Literal(number), Expr::Unit(unit)) => Ok(Expr::Quantity(number, unit)),
                    _ => Err("a quantity needs a number and a unit".to_string()),
                },
                ("-", 1) => Ok(Expr::Neg(Box::new(operand(0)?))),
                (",", _) => {
                    let mut items = Vec::new();
                    for i in 0..element.arguments.len() {
                        match operand(i)? {
                            Expr::List(nested) => items.extend(nested),
                            item => items.push(item),
                        }
                    }
                    Ok(Expr::List(items))
                }
                (op, 2) if BINARY.contains(&op) => Ok(Expr::Binary(op.to_string(), Box::new(operand(0)?), Box::new(operand(1)?))),
                (op, n) => Err(format!("operator `{op}` with {n} operand(s) is not ArcLang's")),
            }
        }
        "InvocationExpression" => {
            let name = function_name(graph, element).ok_or_else(|| "an invocation of nothing".to_string())?;
            let mut operands = Vec::new();
            flatten(graph, element, &name, refer, chains, depth, &mut operands)?;
            aggregate(&name, operands, chains)
        }
        other => Err(format!("{other} is not an expression ArcLang reads")),
    }
}

/// The operands of nested invocations of one function (`max(max(a, b), c)`).
fn flatten(graph: &Graph, element: &Element, name: &str, refer: &dyn Fn(&str) -> Option<String>, chains: &[(String, Vec<String>)], depth: usize, out: &mut Vec<Expr>) -> Result<(), String> {
    for argument in &element.arguments {
        match graph.get(argument) {
            Some(nested) if nested.kind == "InvocationExpression" && function_name(graph, nested).as_deref() == Some(name) => {
                flatten(graph, nested, name, refer, chains, depth + 1, out)?;
            }
            _ => out.push(read(graph, argument, refer, chains, depth + 1)?),
        }
    }
    Ok(())
}

/// `max`/`min` over the same attribute of the steps of one chain is
/// ArcLang's `max(chain, attribute)`.
fn aggregate(name: &str, operands: Vec<Expr>, chains: &[(String, Vec<String>)]) -> Result<Expr, String> {
    if !matches!(name, "max" | "min" | "sum") {
        return Err(format!("function '{name}' is not ArcLang's"));
    }
    let attrs: Vec<(&String, &String)> = operands
        .iter()
        .filter_map(|o| match o {
            Expr::Attr(element, attribute) => Some((element, attribute)),
            _ => None,
        })
        .collect();
    let Some((_, attribute)) = attrs.first().filter(|_| attrs.len() == operands.len()) else {
        return Err(format!("{name}() over values that are not attributes of elements"));
    };
    if attrs.iter().any(|(_, a)| a != attribute) {
        return Err(format!("{name}() over different attributes"));
    }
    let members: BTreeSet<&String> = attrs.iter().map(|(e, _)| *e).collect();
    let chain = chains
        .iter()
        .find(|(_, steps)| steps.iter().collect::<BTreeSet<_>>() == members)
        .ok_or_else(|| format!("{name}() over elements that are not the steps of one chain"))?;
    Ok(Expr::Call(name.to_string(), vec![Expr::Ref(chain.0.clone()), Expr::Ref((*attribute).clone())]))
}

/// The unqualified name of the function an invocation calls.
fn function_name(graph: &Graph, element: &Element) -> Option<String> {
    let function = element.function.as_deref()?;
    let name = graph.get(function).and_then(|f| f.name.clone().or_else(|| f.short_name.clone())).unwrap_or_else(|| function.to_string());
    Some(name.rsplit("::").next().unwrap_or(&name).to_string())
}

/// The ArcLang symbol of a unit referred to by its SysML name or symbol: a
/// library element, or one declared in a units package (never a model
/// attribute that happens to be named `m`).
fn unit_symbol(graph: &Graph, referent: &str) -> Option<String> {
    let element = graph.get(referent)?;
    let owner = element.owner.as_deref().and_then(|o| graph.get(o));
    let in_units = owner.is_some_and(|o| o.kind == "LibraryPackage" || o.name.as_deref().is_some_and(|n| n.ends_with("Units") || n == "SI" || n == "ISQ" || n == "USCustomaryUnits"));
    if !(element.is_library || in_units) {
        return None;
    }
    let candidates = [element.short_name.as_deref(), element.name.as_deref()];
    for candidate in candidates.into_iter().flatten() {
        if let Some(spec) = UNITS.iter().find(|u| u.sysml.trim_matches('\'') == candidate || u.symbol == candidate) {
            return Some(spec.symbol.to_string());
        }
    }
    None
}

fn display(graph: &Graph, id: &str) -> String {
    graph.get(id).and_then(|e| e.name.clone().or_else(|| e.short_name.clone())).unwrap_or_else(|| id.to_string())
}

fn text(expr: &Expr, top: bool) -> String {
    match expr {
        Expr::Literal(value) => value.clone(),
        Expr::Quantity(number, unit) => format!("{number} {unit}"),
        Expr::Attr(element, attribute) => format!("{}.{attribute}", name_text(element)),
        Expr::Ref(name) => name_text(name),
        Expr::Unit(unit) => unit.clone(),
        Expr::List(items) => format!("[{}]", items.iter().map(|i| text(i, true)).collect::<Vec<_>>().join(", ")),
        Expr::Call(function, args) => format!("{function}({})", args.iter().map(|a| text(a, true)).collect::<Vec<_>>().join(", ")),
        Expr::Neg(inner) => format!("-({})", text(inner, true)),
        Expr::Binary(op, lhs, rhs) => {
            let written = format!("{} {op} {}", text(lhs, false), text(rhs, false));
            if top { written } else { format!("({written})") }
        }
    }
}

/// A name as ArcLang's expressions take it: bare when it is an identifier.
fn name_text(name: &str) -> String {
    let plain = name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if plain { name.to_string() } else { string_literal(name) }
}
