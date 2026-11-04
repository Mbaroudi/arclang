//! The bit-exact fixed-point C of the machines the hardware computes: the
//! same generator as the floating-point C (`cgen`), the code of each
//! machine printed from its fixed-point lowering (`fixed`), every operation
//! a static helper whose arithmetic is the VHDL's, so the two give the same
//! words. It is the oracle of `arclang hdl`, and C a processor can run
//! where it has no floating point.
//!
//! A `Real` (or `Float`) is an `int32_t`, the word of its Q format; an
//! `Integer` an `int32_t` and a `Boolean` a `bool`, as in the floating-point
//! C. Inputs are words too: the caller converts a value with
//! `arc_fx_from(value, fraction bits)` (rounded to nearest, ties up,
//! saturated), an output back with `arc_fx_to`.

use super::cexpr::{Access, Helpers};
use super::fixed::{Kind, Loc, MachineFixed, Node, Op, Stmt};
use super::plan::Plan;
use crate::compiler::execution::expr::{Bin, BlockKind, Fun};
use std::fmt::Write as _;

pub fn c_type(kind: Kind) -> &'static str {
    match kind {
        Kind::Bool => "bool",
        _ => "int32_t",
    }
}

/// A constant of a kind.
pub fn lit(kind: Kind, raw: i64) -> String {
    match kind {
        Kind::Bool => if raw != 0 { "true" } else { "false" }.to_string(),
        _ if raw == i32::MIN as i64 => "INT32_MIN".to_string(),
        _ if raw < 0 => format!("({raw})"),
        _ => raw.to_string(),
    }
}

/// The C type of a block's state.
pub fn block_type(fx: &MachineFixed, b: usize, pid_type: &str) -> String {
    let block = &fx.blocks[b];
    match block.kind {
        BlockKind::Pid { .. } => pid_type.to_string(),
        _ => c_type(block.fields[0].1).to_string(),
    }
}

/// The initial state of a block, as statements on `path`.
pub fn block_init(fx: &MachineFixed, b: usize, path: &str, out: &mut String) {
    let block = &fx.blocks[b];
    match block.kind {
        BlockKind::Pid { .. } => {
            for (field, kind, raw, _) in &block.fields {
                let _ = writeln!(out, "    {path}.{} = {};", field.name(), lit(*kind, *raw));
            }
        }
        _ => {
            let (_, kind, raw, _) = block.fields[0];
            let _ = writeln!(out, "    {path} = {};", lit(kind, raw));
        }
    }
}

fn loc(access: &Access, fx: &MachineFixed, l: &Loc) -> String {
    match l {
        Loc::Var(i) => access.variable(*i),
        Loc::Blk(b, field) => match fx.blocks[*b].kind {
            BlockKind::Pid { .. } => format!("{}->blk_{b}.{}", access.state, field.name()),
            _ => format!("{}->blk_{b}", access.state),
        },
        Loc::Tmp(b, name) => format!("b{b}_{name}"),
    }
}

fn helper(h: &mut Helpers, name: &str) {
    h.used.insert(name.to_string());
}

fn aligned(access: &Access, fx: &MachineFixed, n: &Node, s: i32, h: &mut Helpers) -> String {
    helper(h, "fx_align");
    format!("arc_fx_align({}, {s})", expr(access, fx, n, h))
}

fn cmp_symbol(op: Bin) -> &'static str {
    op.symbol()
}

/// A node as a C expression.
pub fn expr(access: &Access, fx: &MachineFixed, n: &Node, h: &mut Helpers) -> String {
    let e = |n: &Node, h: &mut Helpers| expr(access, fx, n, h);
    match &n.op {
        Op::Lit(raw) => lit(n.kind, *raw),
        Op::Read(l) => loc(access, fx, l),
        Op::Add { sub, a, b, sa, sb, d } => {
            helper(h, "fx_resize");
            let (x, y) = (aligned(access, fx, a, *sa, h), aligned(access, fx, b, *sb, h));
            format!("arc_fx_resize({x} {} {y}, {d})", if *sub { "-" } else { "+" })
        }
        Op::Mul { a, b, d } => {
            helper(h, "fx_resize");
            format!("arc_fx_resize((int64_t){} * (int64_t){}, {d})", e(a, h), e(b, h))
        }
        Op::Neg { a, d } => {
            helper(h, "fx_resize");
            format!("arc_fx_resize(-(int64_t){}, {d})", e(a, h))
        }
        Op::Abs { a, d } => {
            helper(h, "fx_resize");
            helper(h, "fx_abs");
            format!("arc_fx_resize(arc_fx_abs({}), {d})", e(a, h))
        }
        Op::MinMax { max, a, b, sa, sb, d } => {
            helper(h, "fx_resize");
            let name = if *max { "fx_max" } else { "fx_min" };
            helper(h, name);
            let (x, y) = (aligned(access, fx, a, *sa, h), aligned(access, fx, b, *sb, h));
            format!("arc_fx_resize(arc_{name}({x}, {y}), {d})")
        }
        Op::Sat { x, lo, hi, s, d } => {
            helper(h, "fx_resize");
            helper(h, "fx_clamp");
            let (a, b, c) = (aligned(access, fx, x, s[0], h), aligned(access, fx, lo, s[1], h), aligned(access, fx, hi, s[2], h));
            format!("arc_fx_resize(arc_fx_clamp({a}, {b}, {c}), {d})")
        }
        Op::Resize { a, d } => {
            helper(h, "fx_resize");
            format!("arc_fx_resize((int64_t){}, {d})", e(a, h))
        }
        Op::Cmp { op, a, b, sa, sb } => {
            let (x, y) = (aligned(access, fx, a, *sa, h), aligned(access, fx, b, *sb, h));
            format!("({x} {} {y})", cmp_symbol(*op))
        }
        Op::IntBin { op, a, b } => {
            let name = match op {
                Bin::Add => "add",
                Bin::Sub => "sub",
                Bin::Mul => "mul",
                _ => "div",
            };
            helper(h, &format!("{name}_i32"));
            format!("arc_{name}_i32({}, {})", e(a, h), e(b, h))
        }
        Op::IntNeg(a) => {
            helper(h, "neg_i32");
            format!("arc_neg_i32({})", e(a, h))
        }
        Op::IntCall { fun, args } => {
            let name = match fun {
                Fun::Min => "min",
                Fun::Max => "max",
                Fun::Abs => "abs",
                Fun::Saturate => "saturate",
            };
            helper(h, &format!("{name}_i32"));
            let args: Vec<String> = args.iter().map(|a| e(a, h)).collect();
            format!("arc_{name}_i32({})", args.join(", "))
        }
        Op::IntCmp { op, a, b } => format!("({} {} {})", e(a, h), cmp_symbol(*op), e(b, h)),
        Op::Not(a) => format!("(!{})", e(a, h)),
        Op::Logic { and, a, b } => format!("({} {} {})", e(a, h), if *and { "&&" } else { "||" }, e(b, h)),
        Op::BoolEq { eq, a, b } => format!("({} {} {})", e(a, h), if *eq { "==" } else { "!=" }, e(b, h)),
        Op::Select { c, a, b } => format!("({} ? {} : {})", e(c, h), e(a, h), e(b, h)),
    }
}

/// Statements, one per line, at `indent`.
pub fn stmts(access: &Access, fx: &MachineFixed, code: &[Stmt], indent: &str, h: &mut Helpers, out: &mut String) {
    for s in code {
        match s {
            Stmt::Set(l, n) => {
                let _ = writeln!(out, "{indent}{} = {};", loc(access, fx, l), expr(access, fx, n, h));
            }
            Stmt::If(c, then, otherwise) => {
                let _ = writeln!(out, "{indent}if ({}) {{", expr(access, fx, c, h));
                stmts(access, fx, then, &format!("{indent}    "), h, out);
                if !otherwise.is_empty() {
                    let _ = writeln!(out, "{indent}}} else {{");
                    stmts(access, fx, otherwise, &format!("{indent}    "), h, out);
                }
                let _ = writeln!(out, "{indent}}}");
            }
        }
    }
}

/// The values of the step a list of statements computes (`b<n>_<name>`), in order.
pub fn temps(fx: &MachineFixed, code: &[Stmt]) -> Vec<(Loc, Kind)> {
    let mut out: Vec<(Loc, Kind)> = Vec::new();
    for s in code {
        match s {
            Stmt::Set(l @ Loc::Tmp(b, name), _) => {
                if !out.iter().any(|(x, _)| x == l) {
                    let kind = fx.blocks[*b].temps.iter().find(|(n, _)| n == name).map(|(_, k)| *k).unwrap_or(Kind::Int);
                    out.push((l.clone(), kind));
                }
            }
            Stmt::If(_, a, b) => {
                for t in temps(fx, a).into_iter().chain(temps(fx, b)) {
                    if !out.iter().any(|(x, _)| *x == t.0) {
                        out.push(t);
                    }
                }
            }
            Stmt::Set(..) => {}
        }
    }
    out
}

/// Equation `k`: its blocks' values of the step in a scope of their own.
pub fn equation(access: &Access, fx: &MachineFixed, k: usize, indent: &str, h: &mut Helpers, out: &mut String) {
    let code = &fx.equations[k];
    let locals = temps(fx, code);
    if locals.is_empty() {
        stmts(access, fx, code, indent, h, out);
        return;
    }
    let _ = writeln!(out, "{indent}{{");
    let inner = format!("{indent}    ");
    for (l, kind) in &locals {
        let _ = writeln!(out, "{inner}{} {};", c_type(*kind), loc(access, fx, l));
    }
    stmts(access, fx, code, &inner, h, out);
    let _ = writeln!(out, "{indent}}}");
}

/// The helpers of the fixed-point code that are used, in a fixed order.
pub fn definitions(h: &Helpers) -> String {
    let used = |name: &str| h.used.contains(name);
    let mut out = String::new();
    if !(used("fx_align") || used("fx_resize")) {
        return out;
    }
    out.push_str(
        "/* Fixed point (docs/embedded-fpga.md): a Real is the int32_t word of its Q
   format. The shifts are floor divisions by powers of two (exact on any C99
   compiler, whatever it does with a signed right shift); rounding is to
   nearest, ties toward plus infinity; every result saturates to 32 bits. */
static int64_t arc_fx_floor(int64_t x, int n)
{
    return (x < 0) ? ~((~x) >> n) : (x >> n);
}

/* (x + 2^(n-1)) >> n; 0 for a shift of 63 or more. */
static int64_t arc_fx_rshr(int64_t x, int n)
{
    int64_t r = x;
    if (n >= 63) {
        r = 0;
    } else if (n > 0) {
        r = arc_fx_floor(x + ((int64_t)1 << (n - 1)), n);
    } else {
        /* no shift */
    }
    return r;
}

/* An operand aligned: s > 0 left (exact: s <= 30), s < 0 rounding right. */
static int64_t arc_fx_align(int32_t a, int s)
{
    int64_t r;
    if (s >= 0) {
        r = (int64_t)a * ((int64_t)1 << s);
    } else {
        r = arc_fx_rshr((int64_t)a, -s);
    }
    return r;
}

/* A result into a word: d > 0 rounding right, d < 0 saturating left; saturated. */
static int32_t arc_fx_resize(int64_t x, int d)
{
    int64_t v = x;
    if (d > 0) {
        v = arc_fx_rshr(x, d);
    } else if (d < 0) {
        int l = -d;
        if (l >= 32) {
            v = (x > 0) ? (int64_t)INT32_MAX : ((x < 0) ? (int64_t)INT32_MIN : 0);
        } else if (x > (((int64_t)1 << (31 - l)) - 1)) {
            v = (int64_t)INT32_MAX;
        } else if (x < -((int64_t)1 << (31 - l))) {
            v = (int64_t)INT32_MIN;
        } else {
            v = x * ((int64_t)1 << l);
        }
    } else {
        /* the same format */
    }
    if (v > (int64_t)INT32_MAX) {
        v = (int64_t)INT32_MAX;
    } else if (v < (int64_t)INT32_MIN) {
        v = (int64_t)INT32_MIN;
    } else {
        /* within the word */
    }
    return (int32_t)v;
}

",
    );
    if used("fx_abs") {
        out.push_str("static int64_t arc_fx_abs(int32_t a)\n{\n    int64_t v = (int64_t)a;\n    return (v < 0) ? -v : v;\n}\n\n");
    }
    if used("fx_min") {
        out.push_str("static int64_t arc_fx_min(int64_t a, int64_t b)\n{\n    return (a < b) ? a : b;\n}\n\n");
    }
    if used("fx_max") {
        out.push_str("static int64_t arc_fx_max(int64_t a, int64_t b)\n{\n    return (a > b) ? a : b;\n}\n\n");
    }
    if used("fx_clamp") {
        out.push_str("static int64_t arc_fx_clamp(int64_t x, int64_t lo, int64_t hi)\n{\n    int64_t r = x;\n    if (x < lo) {\n        r = lo;\n    } else if (x > hi) {\n        r = hi;\n    } else {\n        /* within the limits */\n    }\n    return r;\n}\n\n");
    }
    out
}

/// The plan of the fixed-point reference: the machines named, their code
/// in fixed point at `period_us`, its own name (`<model>_fx`).
pub fn plan(plan: &Plan, machines: &[usize], period_us: u64) -> Result<Plan, String> {
    let mut out = Plan { model: plan.model.clone(), ident: format!("{}_fx", plan.ident), machines: Vec::new() };
    for &m in machines {
        let mut machine = plan.machines[m].clone();
        machine.fixed = Some(Box::new(super::fixed::machine(&machine, period_us)?));
        out.machines.push(machine);
    }
    Ok(out)
}

/// The value of a word, and a value as a word, for the caller (the driver, a harness).
pub const CONVERSIONS: &str = "/* A value as the word of a Q format of `frac` fraction bits (rounded to
   nearest, ties toward plus infinity, saturated), and back. */
static inline int32_t arc_fx_from(double value, int frac)
{
    double scaled = floor(ldexp(value, frac) + 0.5);
    int32_t r;
    if (scaled >= 2147483647.0) {
        r = INT32_MAX;
    } else if (scaled <= -2147483648.0) {
        r = INT32_MIN;
    } else {
        r = (int32_t)scaled;
    }
    return r;
}

static inline double arc_fx_to(int32_t word, int frac)
{
    return ldexp((double)word, -frac);
}
";
