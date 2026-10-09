//! Constraint expressions: arithmetic over typed model attributes.
//!
//! ```text
//! constraint "Braking chain budget" {
//!     id: "CST-001"
//!     assert: sum("FC-AEB", latency) + 5 ms <= "FC-AEB".latency_budget
//! }
//! ```
//!
//! Expressions are dimension-checked: adding a time to a data rate, or
//! comparing a frequency with a duration, is a compile ERROR — as is a
//! reference to an unknown element or attribute. Products and quotients
//! carry derived dimensions: `voltage * current` is a power,
//! `size / bandwidth` is a time. A constraint that evaluates
//! to false is a `constraint:` warning at compile time and a blocker in the
//! production gate. Nothing is ever skipped silently: a `sum` over a chain
//! names every member that lacks the summed attribute.

use super::ast::{AttributeValue, Model};
use super::quantity::{DimVec, Dimension, Quantity, QuantityError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
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
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
        }
    }

    pub fn is_comparison(self) -> bool {
        matches!(self, BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Eq | BinOp::Ne)
    }

    fn precedence(self) -> u8 {
        match self {
            BinOp::Mul | BinOp::Div => 3,
            BinOp::Add | BinOp::Sub => 2,
            _ => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Number(f64),
    Quantity(Quantity),
    /// `"SF-1".latency` — attribute of an element, by id or name.
    Attr { element: String, attribute: String },
    /// A bare name: an element or attribute name passed to a function.
    Ref(String),
    /// `sum(chain, attr)`, `min(chain, attr)`, `max(chain, attr)`, `count(chain)`.
    Call { function: String, args: Vec<Expr> },
    Neg(Box<Expr>),
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
}

fn number_text(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

fn name_text(name: &str) -> String {
    let plain = name.chars().next().map_or(false, |c| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if plain {
        name.to_string()
    } else {
        format!("\"{}\"", name)
    }
}

impl Expr {
    fn precedence(&self) -> u8 {
        match self {
            Expr::Binary { op, .. } => op.precedence(),
            _ => 4,
        }
    }
}

/// Canonical source form (parses back to the same expression).
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(n) => write!(f, "{}", number_text(*n)),
            Expr::Quantity(q) => write!(f, "{}", q),
            Expr::Attr { element, attribute } => write!(f, "{}.{}", name_text(element), attribute),
            Expr::Ref(name) => write!(f, "{}", name_text(name)),
            Expr::Call { function, args } => {
                let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                write!(f, "{}({})", function, args.join(", "))
            }
            Expr::Neg(inner) => write!(f, "-({})", inner),
            Expr::Binary { op, lhs, rhs } => {
                let left = if lhs.precedence() < op.precedence() { format!("({})", lhs) } else { lhs.to_string() };
                // Right operand of the same precedence needs parentheses
                // (a - (b - c)); operators are left-associative.
                let right = if rhs.precedence() <= op.precedence() { format!("({})", rhs) } else { rhs.to_string() };
                write!(f, "{} {} {}", left, op.symbol(), right)
            }
        }
    }
}

/// A computed value: canonical SI magnitude and its dimension, as exponents
/// of the base dimensions (all zero = a plain number).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Value {
    pub canonical: f64,
    pub dimension: DimVec,
}

impl Value {
    fn number(n: f64) -> Self {
        Value { canonical: n, dimension: DimVec::NONE }
    }

    fn of(quantity: &Quantity) -> Self {
        Value { canonical: quantity.canonical(), dimension: quantity.dimension().vector() }
    }

    fn dimension_label(&self) -> String {
        self.dimension.label()
    }
}

fn trim_float(n: f64) -> String {
    let rounded = (n * 1e6).round() / 1e6;
    number_text(rounded)
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.dimension.is_dimensionless() {
            return write!(f, "{}", trim_float(self.canonical));
        }
        match self.dimension.named() {
            // Durations in the unit a reviewer would say them in.
            Some(Dimension::Time) => {
                let seconds = self.canonical.abs();
                if seconds < 1.0 {
                    write!(f, "{} ms", trim_float(self.canonical * 1e3))
                } else if seconds < 60.0 {
                    write!(f, "{} s", trim_float(self.canonical))
                } else if seconds < 3600.0 {
                    write!(f, "{} min", trim_float(self.canonical / 60.0))
                } else {
                    write!(f, "{} h", trim_float(self.canonical / 3600.0))
                }
            }
            _ => write!(f, "{} {}", trim_float(self.canonical), self.dimension.unit()),
        }
    }
}

/// What expressions can see: every element's attributes, by id and by name,
/// and the ordered members of chains/paths/capabilities (`involves`).
#[derive(Debug, Default)]
pub struct Scope {
    attributes: HashMap<String, HashMap<String, AttributeValue>>,
    members: HashMap<String, Vec<String>>,
}

impl Scope {
    fn add(&mut self, id: &str, name: &str, attributes: &HashMap<String, AttributeValue>) {
        let id = attributes.get("id").and_then(|v| v.as_string()).unwrap_or(id);
        for key in [id, name] {
            if !key.is_empty() {
                // First declaration wins; ids are unique, names may not be.
                self.attributes.entry(key.to_string()).or_insert_with(|| attributes.clone());
            }
        }
        if let Some(AttributeValue::List(items)) = attributes.get("involves") {
            let members: Vec<String> = items.iter().filter_map(|v| v.as_string().map(str::to_string)).collect();
            for key in [id, name] {
                if !key.is_empty() {
                    self.members.entry(key.to_string()).or_insert_with(|| members.clone());
                }
            }
        }
    }

    fn add_members(&mut self, id: &str, name: &str, members: &[String]) {
        for key in [id, name] {
            if !key.is_empty() {
                self.members.entry(key.to_string()).or_insert_with(|| members.to_vec());
            }
        }
    }

    pub fn from_model(ast: &Model) -> Scope {
        use super::ast::*;
        let mut scope = Scope::default();

        fn activities(scope: &mut Scope, activity: &OperationalActivity) {
            scope.add(&activity.id, &activity.name, &activity.attributes);
            for sub in &activity.sub_activities {
                activities(scope, sub);
            }
        }
        fn functions(scope: &mut Scope, function: &SystemFunction) {
            scope.add(&function.id, &function.name, &function.attributes);
            for sub in &function.sub_functions {
                functions(scope, sub);
            }
        }
        fn components(scope: &mut Scope, component: &LogicalComponent) {
            scope.add(&component.id, &component.name, &component.attributes);
            for function in &component.functions {
                scope.add(&function.name, &function.name, &function.attributes);
            }
            for sub in &component.sub_components {
                components(scope, sub);
            }
        }

        for oa in &ast.operational_analysis {
            for actor in &oa.actors {
                scope.add(actor.id.as_deref().unwrap_or(""), &actor.name, &actor.attributes);
            }
            for entity in &oa.entities {
                scope.add(&entity.id, &entity.name, &entity.attributes);
                for activity in &entity.activities {
                    activities(&mut scope, activity);
                }
            }
            for activity in &oa.activities {
                activities(&mut scope, activity);
            }
            for process in &oa.processes {
                scope.add(&process.id, &process.name, &process.attributes);
                scope.add_members(&process.id, &process.name, &process.involves);
            }
        }
        for sa in &ast.system_analysis {
            for requirement in &sa.requirements {
                scope.add(&requirement.id, "", &requirement.attributes);
            }
            for function in &sa.functions {
                functions(&mut scope, function);
            }
            for component in &sa.components {
                scope.add("", &component.name, &component.attributes);
            }
            for actor in &sa.external_actors {
                scope.add(&actor.id, &actor.name, &actor.attributes);
            }
            for mission in &sa.missions {
                scope.add(&mission.id, &mission.name, &mission.attributes);
            }
            for capability in &sa.capabilities {
                scope.add(&capability.id, &capability.name, &capability.attributes);
                scope.add_members(&capability.id, &capability.name, &capability.involves);
            }
            for chain in &sa.functional_chains {
                scope.add(&chain.id, &chain.name, &chain.attributes);
                scope.add_members(&chain.id, &chain.name, &chain.involves);
            }
        }
        for la in &ast.logical_architecture {
            for component in &la.components {
                components(&mut scope, component);
            }
            for interface in &la.interfaces {
                scope.add("", &interface.name, &interface.attributes);
            }
            for chain in &la.functional_chains {
                scope.add(&chain.id, &chain.name, &chain.attributes);
                scope.add_members(&chain.id, &chain.name, &chain.involves);
            }
        }
        for pa in &ast.physical_architecture {
            for node in &pa.nodes {
                scope.add(&node.id, &node.name, &node.attributes);
            }
            for link in &pa.links {
                let mut attributes = link.attributes.clone();
                if let Some(bandwidth) = &link.bandwidth {
                    attributes.entry("bandwidth".to_string()).or_insert_with(|| AttributeValue::String(bandwidth.clone()));
                }
                scope.add("", &link.name, &attributes);
            }
            for path in &pa.paths {
                scope.add(&path.id, &path.name, &path.attributes);
                scope.add_members(&path.id, &path.name, &path.involves);
            }
        }
        for epbs in &ast.epbs {
            for system in &epbs.systems {
                scope.add("", &system.name, &system.attributes);
                for subsystem in &system.subsystems {
                    scope.add("", &subsystem.name, &subsystem.attributes);
                    for item in &subsystem.items {
                        scope.add("", &item.name, &item.attributes);
                    }
                }
            }
        }
        for test_case in &ast.test_cases {
            scope.add(&test_case.id, &test_case.name, &test_case.attributes);
        }
        scope
    }

    pub fn knows(&self, element: &str) -> bool {
        self.attributes.contains_key(element)
    }

    pub fn members(&self, element: &str) -> Option<&[String]> {
        self.members.get(element).map(Vec::as_slice)
    }

    /// Members of `collection` that are model elements with attributes
    /// (functions, components...). Exchanges named in a chain carry no
    /// attributes of their own and are not operands.
    pub fn operand_members(&self, collection: &str) -> Result<Vec<&str>, String> {
        let members = self
            .members
            .get(collection)
            .ok_or_else(|| {
                if self.knows(collection) {
                    format!("'{}' has no members (it declares no `involves:` list)", collection)
                } else {
                    format!("unknown element '{}'", collection)
                }
            })?;
        Ok(members.iter().map(String::as_str).filter(|m| self.knows(m)).collect())
    }

    fn attribute(&self, element: &str, attribute: &str) -> Result<Value, String> {
        let attributes = self
            .attributes
            .get(element)
            .ok_or_else(|| format!("unknown element '{}'", element))?;
        let value = attributes
            .get(attribute)
            .ok_or_else(|| format!("element '{}' has no attribute '{}'", element, attribute))?;
        match value.as_quantity() {
            Some(Ok(quantity)) => Ok(Value::of(&quantity)),
            Some(Err(QuantityError::MissingUnit(number))) => Ok(Value::number(number)),
            Some(Err(error)) => Err(format!("'{}'.{} is not numeric: {}", element, attribute, error)),
            None => Err(format!("'{}'.{} is not numeric ('{}')", element, attribute, value.display())),
        }
    }
}

fn same_dimension(op: BinOp, lhs: Value, rhs: Value) -> Result<(), String> {
    if lhs.dimension == rhs.dimension {
        Ok(())
    } else {
        Err(format!(
            "cannot apply '{}' to a {} and a {}",
            op.symbol(),
            lhs.dimension_label(),
            rhs.dimension_label()
        ))
    }
}

fn arithmetic(op: BinOp, lhs: Value, rhs: Value) -> Result<Value, String> {
    match op {
        BinOp::Add | BinOp::Sub => {
            same_dimension(op, lhs, rhs)?;
            let canonical = if op == BinOp::Add { lhs.canonical + rhs.canonical } else { lhs.canonical - rhs.canonical };
            Ok(Value { canonical, dimension: lhs.dimension })
        }
        BinOp::Mul => Ok(Value {
            canonical: lhs.canonical * rhs.canonical,
            dimension: lhs.dimension.multiply(rhs.dimension),
        }),
        BinOp::Div => {
            if rhs.canonical == 0.0 {
                return Err("division by zero".to_string());
            }
            Ok(Value {
                canonical: lhs.canonical / rhs.canonical,
                dimension: lhs.dimension.divide(rhs.dimension),
            })
        }
        _ => Err(format!("'{}' is a comparison, not an arithmetic operator", op.symbol())),
    }
}

fn ref_name<'e>(function: &str, position: &str, expr: &'e Expr) -> Result<&'e str, String> {
    match expr {
        Expr::Ref(name) => Ok(name),
        other => Err(format!("{}(): the {} argument must be a name, got `{}`", function, position, other)),
    }
}

/// Evaluate an arithmetic (non-comparison) expression.
pub fn evaluate(expr: &Expr, scope: &Scope) -> Result<Value, String> {
    match expr {
        Expr::Number(n) => Ok(Value::number(*n)),
        Expr::Quantity(q) => Ok(Value::of(q)),
        Expr::Attr { element, attribute } => scope.attribute(element, attribute),
        Expr::Ref(name) => Err(format!("'{}' is a name, not a value — write `{}.<attribute>`", name, name_text(name))),
        Expr::Neg(inner) => {
            let value = evaluate(inner, scope)?;
            Ok(Value { canonical: -value.canonical, dimension: value.dimension })
        }
        Expr::Binary { op, lhs, rhs } => {
            if op.is_comparison() {
                return Err("a comparison cannot be used as an operand".to_string());
            }
            arithmetic(*op, evaluate(lhs, scope)?, evaluate(rhs, scope)?)
        }
        Expr::Call { function, args } => match (function.as_str(), args.as_slice()) {
            ("count", [collection]) => {
                let collection = ref_name("count", "first", collection)?;
                Ok(Value::number(scope.operand_members(collection)?.len() as f64))
            }
            ("sum" | "min" | "max", [collection, attribute]) => {
                let collection = ref_name(function, "first", collection)?;
                let attribute = ref_name(function, "second", attribute)?;
                let members = scope.operand_members(collection)?;
                if members.is_empty() {
                    return Err(format!("{}(): '{}' has no members to aggregate", function, collection));
                }
                let mut values = Vec::with_capacity(members.len());
                let mut missing = Vec::new();
                for member in &members {
                    match scope.attribute(member, attribute) {
                        Ok(value) => values.push(value),
                        Err(reason) if reason.contains("has no attribute") => missing.push(*member),
                        Err(reason) => return Err(reason),
                    }
                }
                if !missing.is_empty() {
                    return Err(format!(
                        "{}(): member(s) of '{}' declare no '{}': {}",
                        function,
                        collection,
                        attribute,
                        missing.join(", ")
                    ));
                }
                let first = values[0];
                for value in &values[1..] {
                    same_dimension(BinOp::Add, first, *value)?;
                }
                let magnitudes = values.iter().map(|v| v.canonical);
                let canonical = match function.as_str() {
                    "sum" => magnitudes.sum(),
                    "min" => magnitudes.fold(f64::INFINITY, f64::min),
                    _ => magnitudes.fold(f64::NEG_INFINITY, f64::max),
                };
                Ok(Value { canonical, dimension: first.dimension })
            }
            ("sum" | "min" | "max", _) => Err(format!("{}() takes (collection, attribute)", function)),
            ("count", _) => Err("count() takes (collection)".to_string()),
            (other, _) => Err(format!("unknown function '{}' (available: sum, min, max, count)", other)),
        },
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub satisfied: bool,
    pub left: Value,
    pub right: Value,
}

/// Check a constraint: the expression must be a comparison of two
/// same-dimension values. `Err` means the constraint is ill-formed.
pub fn check(expr: &Expr, scope: &Scope) -> Result<Outcome, String> {
    let Expr::Binary { op, lhs, rhs } = expr else {
        return Err(format!("a constraint must be a comparison (<, <=, >, >=, ==, !=), got `{}`", expr));
    };
    if !op.is_comparison() {
        return Err(format!("a constraint must be a comparison (<, <=, >, >=, ==, !=), got `{}`", expr));
    }
    let left = evaluate(lhs, scope)?;
    let right = evaluate(rhs, scope)?;
    same_dimension(*op, left, right)?;
    // Tolerance: quantities come from decimal literals scaled by factors.
    let epsilon = 1e-9 * left.canonical.abs().max(right.canonical.abs()).max(1.0);
    let difference = left.canonical - right.canonical;
    let satisfied = match op {
        BinOp::Lt => difference < -epsilon,
        BinOp::Le => difference <= epsilon,
        BinOp::Gt => difference > epsilon,
        BinOp::Ge => difference >= -epsilon,
        BinOp::Eq => difference.abs() <= epsilon,
        _ => difference.abs() > epsilon,
    };
    Ok(Outcome { satisfied, left, right })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Compiler, CompilerConfig, CompilerError};

    const MODEL: &str = r#"
model T {}
system_analysis "SA" {
    function "Detect" { id: "SF-1" latency: 40 ms wcet: 0.03 s }
    function "Decide" { id: "SF-2" latency: "50 ms" wcet: 20 ms }
    function "Act" { id: "SF-3" }
    functional_chain "Braking" { id: "FC-1" involves: ["SF-1", "SF-2"] latency_budget: 100 ms }
    functional_chain "Full" { id: "FC-2" involves: ["SF-1", "SF-2", "SF-3"] }
}
architecture physical {
    node "ECU" { id: "PN-1" ram: 512 MB }
    link "Bus" { from: "PN-1" to: "PN-1" bandwidth: "500 kbps" }
}
"#;

    fn compile(constraints: &str) -> Result<crate::compiler::CompilationResult, CompilerError> {
        Compiler::new(CompilerConfig::default()).compile_string(&format!("{}\n{}", MODEL, constraints))
    }

    fn outcome(assertion: &str) -> Result<Outcome, String> {
        let result = compile(&format!("constraint \"C\" {{ assert: {} }}", assertion)).map_err(|e| e.to_string())?;
        check(&result.ast.constraints[0].expression, &Scope::from_model(&result.ast))
    }

    #[test]
    fn sums_latencies_across_units_and_compares_to_the_budget() {
        let result = outcome(r#"sum("FC-1", latency) <= "FC-1".latency_budget"#).unwrap();
        assert!(result.satisfied);
        assert_eq!(result.left.to_string(), "90 ms");
        assert_eq!(result.right.to_string(), "100 ms");

        let result = outcome(r#"sum("FC-1", latency) + 15 ms <= "FC-1".latency_budget"#).unwrap();
        assert!(!result.satisfied, "90 + 15 > 100");
        assert_eq!(result.left.to_string(), "105 ms");
    }

    #[test]
    fn arithmetic_precedence_scaling_ratios_and_aggregates() {
        assert!(outcome(r#""SF-1".latency + "SF-2".latency * 2 == 140 ms"#).unwrap().satisfied);
        assert!(outcome(r#"("SF-1".latency + "SF-2".latency) * 2 == 180 ms"#).unwrap().satisfied);
        assert!(outcome(r#""SF-1".latency / "SF-2".latency == 0.8"#).unwrap().satisfied);
        assert!(outcome(r#"max("FC-1", wcet) == 30 ms"#).unwrap().satisfied);
        assert!(outcome(r#"min("FC-1", wcet) == 0.02 s"#).unwrap().satisfied);
        assert!(outcome(r#"count("FC-1") == 2"#).unwrap().satisfied);
        assert!(outcome(r#""SF-2".latency - 10 ms > "SF-1".latency - 1 ms"#).unwrap().satisfied);
        assert!(outcome(r#"Bus.bandwidth >= 0.5 Mbps"#).unwrap().satisfied);
        assert!(outcome(r#""PN-1".ram < 1 GB"#).unwrap().satisfied);
        assert!(outcome(r#"-"SF-1".latency < 0 ms"#).unwrap().satisfied);
    }

    #[test]
    fn products_and_quotients_carry_derived_dimensions() {
        let source = r#"
model D {}
architecture physical {
    node "Inverter" { id: "PN-1" voltage: 400 V current: 50 A power: 22 kW energy: 11 kWh mass: 12 kg }
    link "Bus" { from: "PN-1" to: "PN-1" bandwidth: 2 Mbps frame: 64 B speed_limit: 130 km/h }
}
"#;
        let check_one = |assertion: &str| -> Result<Outcome, String> {
            let result = Compiler::new(CompilerConfig::default())
                .compile_string(&format!("{}\nconstraint \"C\" {{ assert: {} }}", source, assertion))
                .map_err(|e| e.to_string())?;
            check(&result.ast.constraints[0].expression, &Scope::from_model(&result.ast))
        };
        // P = U * I : 400 V * 50 A = 20 kW, under the 22 kW rating.
        let power = check_one(r#""PN-1".voltage * "PN-1".current <= "PN-1".power"#).unwrap();
        assert!(power.satisfied);
        assert_eq!(power.left.to_string(), "20000 W");
        // E / P = t : 11 kWh at 22 kW lasts half an hour.
        let autonomy = check_one(r#""PN-1".energy / "PN-1".power == 30 min"#).unwrap();
        assert!(autonomy.satisfied, "{:?}", autonomy);
        // size / rate = time : a 64-byte frame at 2 Mbps takes 0.256 ms.
        let frame = check_one(r#"Bus.frame / Bus.bandwidth < 1 ms"#).unwrap();
        assert!(frame.satisfied);
        assert_eq!(frame.left.to_string(), "0.256 ms");
        // speed * time = length
        assert!(check_one(r#"Bus.speed_limit * 36 s == 1.3 km"#).unwrap().satisfied);
        // power / mass has no name: the unit is spelled out.
        let density = check_one(r#""PN-1".power / "PN-1".mass > 1 kW"#).unwrap_err();
        assert!(density.contains("cannot apply '>' to a quantity in m^2·s^-3 and a power"), "{density}");
        // percent is a plain ratio.
        assert!(check_one(r#""PN-1".voltage * "PN-1".current / "PN-1".power >= 90 percent"#).unwrap().satisfied);
    }

    #[test]
    fn dimension_mismatches_are_errors() {
        let error = outcome(r#""SF-1".latency <= Bus.bandwidth"#).unwrap_err();
        assert!(error.contains("cannot apply '<=' to a time and a data rate"), "{error}");
        let error = outcome(r#""SF-1".latency + 5 <= 100 ms"#).unwrap_err();
        assert!(error.contains("cannot apply '+' to a time and a plain number"), "{error}");
        let error = outcome(r#""SF-1".latency * "SF-2".latency <= 100 ms"#).unwrap_err();
        assert!(error.contains("cannot apply '<=' to a quantity in s^2 and a time"), "{error}");
    }

    #[test]
    fn unknown_elements_attributes_and_missing_members_are_errors() {
        assert!(outcome(r#""SF-404".latency <= 1 ms"#).unwrap_err().contains("unknown element 'SF-404'"));
        assert!(outcome(r#""SF-3".latency <= 1 ms"#).unwrap_err().contains("element 'SF-3' has no attribute 'latency'"));
        let error = outcome(r#"sum("FC-2", latency) <= 1 s"#).unwrap_err();
        assert!(error.contains("member(s) of 'FC-2' declare no 'latency': SF-3"), "{error}");
        assert!(outcome(r#"sum("SF-1", latency) <= 1 s"#).unwrap_err().contains("has no members"));
        assert!(outcome(r#"avg("FC-1", latency) <= 1 s"#).unwrap_err().contains("unknown function 'avg'"));
        assert!(outcome(r#""SF-1".latency + 1 ms"#).unwrap_err().contains("must be a comparison"));
    }

    #[test]
    fn ill_formed_constraint_fails_compilation_and_violation_is_a_warning() {
        let error = compile(r#"constraint "Bad" { assert: "SF-404".latency <= 1 ms }"#).err().expect("must not compile");
        assert!(error.to_string().contains("constraint 'Bad': unknown element 'SF-404'"), "{error}");

        let result = compile(r#"constraint "Tight" { id: "CST-1" assert: sum("FC-1", latency) <= 80 ms }"#).expect("violations compile");
        assert!(
            result.warnings.iter().any(|w| w.contains("constraint 'Tight' is violated") && w.contains("90 ms") && w.contains("80 ms")),
            "{:?}",
            result.warnings
        );
        let info = &result.semantic_model.constraints[0];
        assert_eq!(info.id, "CST-1");
        assert!(!info.satisfied);
        assert_eq!(info.expression, r#"sum("FC-1", latency) <= 80 ms"#);
        assert_eq!(result.semantic_model.all_elements["CST-1"].element_type, "Constraint");
    }

    #[test]
    fn canonical_text_round_trips_through_the_parser() {
        for source in [
            r#"sum("FC-1", latency) + 15 ms <= "FC-1".latency_budget"#,
            r#"("SF-1".latency + "SF-2".latency) * 2 == 180 ms"#,
            r#""SF-2".latency - ("SF-1".latency - 1 ms) > 0 ms"#,
            r#"Bus.bandwidth / 2 >= 0.25 Mbps"#,
        ] {
            let first = compile(&format!("constraint \"C\" {{ assert: {} }}", source)).unwrap().ast.constraints[0].expression.clone();
            assert_eq!(first.to_string(), source);
            let again = compile(&format!("constraint \"C\" {{ assert: {} }}", first)).unwrap().ast.constraints[0].expression.clone();
            assert_eq!(first, again);
        }
    }
}
