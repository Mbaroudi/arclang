//! The action language as C: the typed tree of `execution::expr` printed
//! operator for operator, so the C computes what `Machine` computes.
//!
//! - `Integer` is `int32_t`; `+ - * /`, negation and `abs` go through
//!   helpers that compute in `int64_t` and saturate (a division by zero
//!   gives 0), as the model does;
//! - `Real` is `double`, `Float` is `float`; the operators are C's (IEEE
//!   754; the generated file turns floating-point contraction off);
//! - `min`, `max`, `abs`, `saturate` and the blocks are static functions,
//!   so every argument is evaluated once; a block's state is a field of the
//!   machine's state structure.

use super::plan::MachinePlan;
use crate::compiler::execution::expr::{Assign, Bin, BlockKind, Direction, Expr, Fun, Program, Ty, Un, Value};
use std::collections::BTreeSet;
use std::fmt::Write as _;

/// The helpers a file uses, by name.
#[derive(Debug, Default)]
pub struct Helpers {
    pub used: BTreeSet<String>,
}

fn suffix(ty: Ty) -> &'static str {
    match ty {
        Ty::Int => "i32",
        Ty::Real => "f64",
        Ty::Float => "f32",
        Ty::Bool => "b",
    }
}

/// A floating constant C reads back to the same value.
pub fn real(value: f64) -> String {
    let text = format!("{value:?}");
    let text = if text.contains(['.', 'e', 'E']) || text.contains("inf") || text.contains("NaN") { text } else { format!("{text}.0") };
    if value < 0.0 {
        format!("({text})")
    } else {
        text
    }
}

pub fn float(value: f32) -> String {
    let text = format!("{value:?}");
    let text = if text.contains(['.', 'e', 'E']) { text } else { format!("{text}.0") };
    if value < 0.0 {
        format!("({text}f)")
    } else {
        format!("{text}f")
    }
}

pub fn constant(value: Value) -> String {
    match value {
        Value::Bool(b) => if b { "true" } else { "false" }.to_string(),
        Value::Int(i32::MIN) => "INT32_MIN".to_string(),
        Value::Int(i) if i < 0 => format!("({i})"),
        Value::Int(i) => i.to_string(),
        Value::Real(r) => real(r),
        Value::Float(f) => float(f),
    }
}

/// How the C reaches the machine's values.
pub struct Access<'a> {
    pub machine: &'a MachinePlan,
    /// The state structure (`st`) and the inputs (`in`).
    pub state: &'a str,
    pub inputs: &'a str,
    /// The period macro, for the blocks' `T`.
    pub period: &'a str,
}

impl Access<'_> {
    fn program(&self) -> &Program {
        self.machine.program()
    }

    pub fn variable(&self, i: usize) -> String {
        let v = &self.program().variables[i];
        let field = &self.machine.variables[i].ident;
        if v.direction == Direction::In {
            format!("{}->{field}", self.inputs)
        } else {
            format!("{}->{field}", self.state)
        }
    }

    pub fn expr(&self, e: &Expr, h: &mut Helpers) -> String {
        match e {
            Expr::Const(v) => constant(*v),
            Expr::Var(i) => self.variable(*i),
            Expr::Unary(Un::Not, _, a) => format!("(!{})", self.expr(a, h)),
            Expr::Unary(Un::Neg, Ty::Int, a) => {
                h.used.insert("neg_i32".to_string());
                format!("arc_neg_i32({})", self.expr(a, h))
            }
            Expr::Unary(Un::Neg, _, a) => format!("(-{})", self.expr(a, h)),
            Expr::Binary(op @ (Bin::Add | Bin::Sub | Bin::Mul | Bin::Div), Ty::Int, a, b) => {
                let name = match op {
                    Bin::Add => "add",
                    Bin::Sub => "sub",
                    Bin::Mul => "mul",
                    _ => "div",
                };
                h.used.insert(format!("{name}_i32"));
                format!("arc_{name}_i32({}, {})", self.expr(a, h), self.expr(b, h))
            }
            Expr::Binary(op, _, a, b) => format!("({} {} {})", self.expr(a, h), op.symbol(), self.expr(b, h)),
            Expr::Cast(ty, a) => format!("(({}){})", ty.c(), self.expr(a, h)),
            Expr::Call(f, ty, args) => {
                let name = match f {
                    Fun::Min => "min",
                    Fun::Max => "max",
                    Fun::Abs => "abs",
                    Fun::Saturate => "saturate",
                };
                let helper = format!("{name}_{}", suffix(*ty));
                h.used.insert(helper.clone());
                let args: Vec<String> = args.iter().map(|a| self.expr(a, h)).collect();
                format!("arc_{helper}({})", args.join(", "))
            }
            Expr::Block(b) => {
                let block = &self.program().blocks[*b];
                let args: Vec<String> = block.args.iter().map(|a| self.expr(a, h)).collect();
                let field = format!("&{}->blk_{b}", self.state);
                let t = format!("((double){} * 1.0e-6)", self.period);
                match block.kind {
                    BlockKind::Delay => {
                        let helper = format!("delay_{}", suffix(block.ty));
                        h.used.insert(helper.clone());
                        format!("arc_{helper}({field}, {})", args[0])
                    }
                    BlockKind::Integrator { limited } => {
                        let helper = if limited { "integrator_limited" } else { "integrator" };
                        h.used.insert(helper.to_string());
                        // (state, x, k, T[, lo, hi])
                        let (gains, limits) = args.split_at(2);
                        let mut list = gains.to_vec();
                        list.push(t);
                        list.extend(limits.iter().cloned());
                        format!("arc_{helper}({field}, {})", list.join(", "))
                    }
                    BlockKind::Pid { limited } => {
                        let helper = if limited { "pid_limited" } else { "pid" };
                        h.used.insert(helper.to_string());
                        let (gains, limits) = args.split_at(4);
                        let mut list = gains.to_vec();
                        list.push(t);
                        list.extend(limits.iter().cloned());
                        format!("arc_{helper}({field}, {})", list.join(", "))
                    }
                }
            }
        }
    }

    /// Statements, one per line, at `indent`.
    pub fn statements(&self, code: &[Assign], indent: &str, h: &mut Helpers, out: &mut String) {
        for s in code {
            let _ = writeln!(out, "{indent}{} = {};", self.variable(s.target), self.expr(&s.value, h));
        }
    }
}

/// Whether a program uses PID blocks (the header then declares their state type).
pub fn uses_pid(program: &Program) -> bool {
    program.blocks.iter().any(|b| matches!(b.kind, BlockKind::Pid { .. }))
}

/// The C type of a block's state field.
pub fn block_field(program: &Program, b: usize, pid_type: &str) -> String {
    let block = &program.blocks[b];
    match block.kind {
        BlockKind::Delay => block.ty.c().to_string(),
        BlockKind::Integrator { .. } => "double".to_string(),
        BlockKind::Pid { .. } => pid_type.to_string(),
    }
}

/// The definitions of the helpers used, in a fixed order.
pub fn definitions(h: &Helpers, pid_type: &str) -> String {
    let mut out = String::new();
    let used = |name: &str| h.used.contains(name);
    let int_ops = ["add_i32", "sub_i32", "mul_i32", "div_i32", "neg_i32", "abs_i32"];
    if int_ops.iter().any(|n| used(n)) {
        out.push_str("/* Integers saturate to the range of int32_t; a division by zero gives 0. */\nstatic int32_t arc_sat_i32(int64_t v)\n{\n    int32_t r;\n    if (v > (int64_t)INT32_MAX) {\n        r = (int32_t)INT32_MAX;\n    } else if (v < (int64_t)INT32_MIN) {\n        r = (int32_t)INT32_MIN;\n    } else {\n        r = (int32_t)v;\n    }\n    return r;\n}\n\n");
    }
    for (name, op) in [("add", "+"), ("sub", "-"), ("mul", "*")] {
        if used(&format!("{name}_i32")) {
            let _ = writeln!(out, "static int32_t arc_{name}_i32(int32_t a, int32_t b)\n{{\n    return arc_sat_i32((int64_t)a {op} (int64_t)b);\n}}\n");
        }
    }
    if used("div_i32") {
        out.push_str("static int32_t arc_div_i32(int32_t a, int32_t b)\n{\n    int32_t r = 0;\n    if (b != 0) {\n        r = arc_sat_i32((int64_t)a / (int64_t)b);\n    }\n    return r;\n}\n\n");
    }
    if used("neg_i32") {
        out.push_str("static int32_t arc_neg_i32(int32_t a)\n{\n    return arc_sat_i32(-(int64_t)a);\n}\n\n");
    }
    if used("abs_i32") {
        out.push_str("static int32_t arc_abs_i32(int32_t a)\n{\n    int64_t v = (int64_t)a;\n    return arc_sat_i32((v < 0) ? -v : v);\n}\n\n");
    }
    for ty in [Ty::Int, Ty::Real, Ty::Float] {
        let (c, s) = (ty.c(), suffix(ty));
        let zero = match ty {
            Ty::Int => "0",
            Ty::Real => "0.0",
            _ => "0.0f",
        };
        if used(&format!("min_{s}")) {
            let _ = writeln!(out, "static {c} arc_min_{s}({c} a, {c} b)\n{{\n    return (a < b) ? a : b;\n}}\n");
        }
        if used(&format!("max_{s}")) {
            let _ = writeln!(out, "static {c} arc_max_{s}({c} a, {c} b)\n{{\n    return (a > b) ? a : b;\n}}\n");
        }
        if ty != Ty::Int && used(&format!("abs_{s}")) {
            let _ = writeln!(out, "static {c} arc_abs_{s}({c} a)\n{{\n    return (a < {zero}) ? -a : a;\n}}\n");
        }
        if used(&format!("saturate_{s}")) || (ty == Ty::Real && (used("integrator_limited") || used("pid_limited"))) {
            let _ = writeln!(out, "static {c} arc_saturate_{s}({c} x, {c} lo, {c} hi)\n{{\n    {c} r = x;\n    if (x < lo) {{\n        r = lo;\n    }} else if (x > hi) {{\n        r = hi;\n    }} else {{\n        /* within the limits */\n    }}\n    return r;\n}}\n");
        }
    }
    for ty in [Ty::Bool, Ty::Int, Ty::Real, Ty::Float] {
        let (c, s) = (ty.c(), suffix(ty));
        if used(&format!("delay_{s}")) {
            let _ = writeln!(out, "/* Unit delay: the input of the previous step. */\nstatic {c} arc_delay_{s}({c} *held, {c} x)\n{{\n    {c} out = *held;\n    *held = x;\n    return out;\n}}\n");
        }
    }
    if used("integrator") {
        out.push_str("/* Discrete integrator, forward Euler: y = s; s := s + k*T*x. */\nstatic double arc_integrator(double *s, double x, double k, double t)\n{\n    double out = *s;\n    *s = *s + ((k * t) * x);\n    return out;\n}\n\n");
    }
    if used("integrator_limited") {
        out.push_str("/* Discrete integrator, forward Euler, its state and output held in [lo, hi]. */\nstatic double arc_integrator_limited(double *s, double x, double k, double t, double lo, double hi)\n{\n    double out = arc_saturate_f64(*s, lo, hi);\n    *s = arc_saturate_f64(*s + ((k * t) * x), lo, hi);\n    return out;\n}\n\n");
    }
    if used("pid") {
        let _ = writeln!(out, "/* PID, parallel: kp*e + integral + kd*(e - e_previous)/T; integral := integral + ki*T*e.\n   No derivative on the first step. */\nstatic double arc_pid({pid_type} *p, double e, double kp, double ki, double kd, double t)\n{{\n    double previous = p->primed ? p->previous : e;\n    double out = ((kp * e) + p->integral) + ((kd * (e - previous)) / t);\n    p->integral = p->integral + ((ki * t) * e);\n    p->previous = e;\n    p->primed = true;\n    return out;\n}}\n");
    }
    if used("pid_limited") {
        let _ = writeln!(out, "/* PID, parallel, its output held in [lo, hi]; clamping anti-windup: no\n   integration further into the saturation. */\nstatic double arc_pid_limited({pid_type} *p, double e, double kp, double ki, double kd, double t, double lo, double hi)\n{{\n    double previous = p->primed ? p->previous : e;\n    double raw = ((kp * e) + p->integral) + ((kd * (e - previous)) / t);\n    bool winding = ((raw > hi) && (e > 0.0)) || ((raw < lo) && (e < 0.0));\n    if (!winding) {{\n        p->integral = p->integral + ((ki * t) * e);\n    }}\n    p->previous = e;\n    p->primed = true;\n    return arc_saturate_f64(raw, lo, hi);\n}}\n");
    }
    out
}
