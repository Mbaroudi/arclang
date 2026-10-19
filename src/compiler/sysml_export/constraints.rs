//! Constraints as asserted SysML expressions over the attributes of the
//! exported usages: `"SF-1".latency + "SF-2".latency <= 50 ms` becomes
//! `assert constraint { 'SF-1'.latency + 'SF-2'.latency <= 50 [ms] }`, an
//! aggregate over a chain the expression over its members.

use super::{text, Exporter};
use crate::compiler::constraint::{Expr, Scope};

/// The SysML expression of `expr`, or why it cannot be written.
fn expression(e: &mut Exporter<'_>, expr: &Expr, scope: &Scope, top: bool) -> Result<String, String> {
    Ok(match expr {
        // A constraint's numbers are written as the model writes them.
        Expr::Number(n) => if n.fract() == 0.0 && n.abs() < 1e15 { format!("{}", *n as i64) } else { n.to_string() },
        Expr::Quantity(quantity) => {
            e.units.insert(quantity.spec().sysml);
            text::quantity(quantity)
        }
        Expr::Attr { element, attribute } => format!("{}.{}", path(e, element)?, text::name(attribute)),
        Expr::Ref(name) => return Err(format!("'{name}' is not a value")),
        Expr::Neg(inner) => format!("-({})", expression(e, inner, scope, false)?),
        Expr::Binary { op, lhs, rhs } => {
            let written = format!("{} {} {}", expression(e, lhs, scope, false)?, op.symbol(), expression(e, rhs, scope, false)?);
            if top { written } else { format!("({written})") }
        }
        Expr::Call { function, args } => {
            let Some(Expr::Ref(collection)) = args.first() else { return Err(format!("{function}() without a collection")) };
            let members = scope.operand_members(collection)?;
            if function == "count" {
                return Ok(members.len().to_string());
            }
            let Some(Expr::Ref(attribute)) = args.get(1) else { return Err(format!("{function}() without an attribute")) };
            let mut paths = Vec::new();
            for member in members {
                paths.push(format!("{}.{}", path(e, member)?, text::name(attribute)));
            }
            match function.as_str() {
                "sum" => format!("({})", paths.join(" + ")),
                "min" | "max" => {
                    let mut folded = paths[0].clone();
                    for path in &paths[1..] {
                        folded = format!("DataFunctions::{function}({folded}, {path})");
                    }
                    folded
                }
                other => return Err(format!("unknown function '{other}'")),
            }
        }
    })
}

fn path(e: &Exporter<'_>, element: &str) -> Result<String, String> {
    e.feature(element).ok_or_else(|| match e.reference(element) {
        Some(_) => format!("'{element}' is a variant or in one, which SysML v2 reaches only inside its variation"),
        None => format!("no exported element '{element}'"),
    })
}

pub(super) fn write(e: &mut Exporter<'_>) {
    let model = e.model;
    if model.constraints.is_empty() {
        return;
    }
    let scope = Scope::from_model(model);
    for constraint in &model.constraints {
        let head = text::declared(&constraint.id, &constraint.name);
        match expression(e, &constraint.expression, &scope, true) {
            Ok(written) => e.block(&format!("assert constraint {head}"), |e| {
                e.header("Constraint", &constraint.attributes, &[]);
                e.line(&written);
            }),
            // Not writable as an expression: kept as text, and said.
            Err(reason) => e.block(&format!("constraint {head}"), |e| {
                e.header("Constraint", &constraint.attributes, &[]);
                e.line(&format!("attribute expression : String = {};", text::string(&constraint.expression.to_string())));
                e.line(&format!("// not an expression of the export: {reason}"));
            }),
        }
    }
}
