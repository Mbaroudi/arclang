//! Fixed point for the parts of a model the hardware computes: every `Real`
//! (and `Float`) value is a signed 32-bit word with a binary point of its
//! own, its Q format, derived from the range of the value.
//!
//! - **Variables.** A variable's format comes from its range: `range lo ..
//!   hi` in its declaration, else derived: a `const` is its value; any other
//!   variable takes the hull of its initial value and of everything assigned
//!   to it (equations and actions), computed again until it no longer grows.
//!   A variable fed back on itself (`f = f + 0.2 * (v - f)`) grows at every
//!   pass: its range cannot be derived, and the compiler says to declare it.
//!   An `in` variable has no assignment: its range is declared, or it is an
//!   error. `format Q<i>.<f>` in the declaration sets the format instead.
//! - **The format of a range.** `i` integer bits, the fewest for which the
//!   largest magnitude `m` of the range is below `2^i`; `f = 31 - i` fraction
//!   bits. A range that needs more than 31 integer bits is refused.
//! - **Every operation** has a format too, from the range of its result
//!   (interval arithmetic over its operands' ranges), and is exact up to the
//!   final rounding into it: `+ -` align their operands to the finer of
//!   their two formats (at most 30 bits finer than the coarser: beyond, the
//!   finer one is rounded first), add in 64 bits and round into the result;
//!   `*` multiplies the two words exactly in 64 bits and rounds.
//! - **Rounding** is to nearest, ties toward plus infinity: `(x + 2^(n-1)) >>
//!   n`, an arithmetic shift (a shift of 63 or more gives 0). **Saturation**:
//!   every result is held in the range of its 32-bit word (a value beyond is
//!   the word's largest or smallest), never wrapped.
//! - **Division** only by a constant: `x / c` is `x * (1/c)`, the reciprocal
//!   rounded into its own format. An `Integer` stays a saturating `int32`,
//!   exactly as the model computes it (its division by a constant too).
//! - **Blocks**: their gains are constants (`k*T`, `ki*T` and `kd/T` folded
//!   at the period of the run); a `pid` has limits (its integral is bounded
//!   by them), an `integrator` without limits takes the range of the
//!   variable it feeds.
//!
//! The lowered code (`Node`, `Stmt`) is what both the bit-exact C reference
//! (`fxc`) and the VHDL (`vhdl`) print, primitive for primitive, so the two
//! compute the same bits; `bounds` gives the quantisation error it may have
//! against the floating-point model.

use super::plan::MachinePlan;
use crate::compiler::execution::expr::{self, Assign, Bin, BlockKind, Direction, Env, Expr, Fun, Program, Ty, Un, Value};
use std::collections::BTreeMap;

/// How much finer one operand may be than the other before it is rounded.
pub const MAX_ALIGN: i32 = 30;
/// The fewest integer bits a format has (a range near 0).
const MIN_INT_BITS: i32 = -30;

/// What a value is in the hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bool,
    /// A saturating `int32`, as the model's `Integer`.
    Int,
    /// A signed 32-bit word of this many fraction bits.
    Fixed(i32),
}

impl Kind {
    /// `Q6.25`, `int32`, `boolean`.
    pub fn label(self) -> String {
        match self {
            Kind::Bool => "boolean".to_string(),
            Kind::Int => "int32".to_string(),
            Kind::Fixed(f) => format!("Q{}.{f}", 31 - f),
        }
    }
}

/// A field of a block's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Field {
    /// Delay: the held value; integrator: its state.
    Held,
    Integral,
    Previous,
    Primed,
}

impl Field {
    pub fn name(self) -> &'static str {
        match self {
            Field::Held => "held",
            Field::Integral => "integral",
            Field::Previous => "previous",
            Field::Primed => "primed",
        }
    }
}

/// Where a value is kept.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Loc {
    Var(usize),
    Blk(usize, Field),
    /// A value of the step a block computes (`b<block>_<name>`).
    Tmp(usize, &'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub kind: Kind,
    /// The values it takes (a constant: its exact value, both ends).
    pub range: (f64, f64),
    pub op: Op,
}

/// The operations, each one primitive of the C reference and of the VHDL.
/// A shift `s` aligns an operand (`> 0`: left, exact; `< 0`: rounding
/// right); `d` resizes a 64-bit result into the node's format (`> 0`:
/// rounding right; `< 0`: saturating left), then saturates it to 32 bits.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// A constant: the word (`Fixed`), the integer, 0/1.
    Lit(i64),
    Read(Loc),
    Add { sub: bool, a: Box<Node>, b: Box<Node>, sa: i32, sb: i32, d: i32 },
    Mul { a: Box<Node>, b: Box<Node>, d: i32 },
    Neg { a: Box<Node>, d: i32 },
    Abs { a: Box<Node>, d: i32 },
    MinMax { max: bool, a: Box<Node>, b: Box<Node>, sa: i32, sb: i32, d: i32 },
    Sat { x: Box<Node>, lo: Box<Node>, hi: Box<Node>, s: [i32; 3], d: i32 },
    /// A word (or an `int32`, as 0 fraction bits) into this format.
    Resize { a: Box<Node>, d: i32 },
    /// A word saturated to a narrower word of `bits` bits (a variable
    /// stored in fewer than 32): the same value when it is in its range.
    Narrow { a: Box<Node>, bits: u32 },
    /// A comparison of two words, aligned.
    Cmp { op: Bin, a: Box<Node>, b: Box<Node>, sa: i32, sb: i32 },
    /// Integers: `+ - * /` saturating (a division by zero gives 0).
    IntBin { op: Bin, a: Box<Node>, b: Box<Node> },
    IntNeg(Box<Node>),
    IntCall { fun: Fun, args: Vec<Node> },
    IntCmp { op: Bin, a: Box<Node>, b: Box<Node> },
    Not(Box<Node>),
    Logic { and: bool, a: Box<Node>, b: Box<Node> },
    BoolEq { eq: bool, a: Box<Node>, b: Box<Node> },
    /// `c ? a : b`.
    Select { c: Box<Node>, a: Box<Node>, b: Box<Node> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Set(Loc, Node),
    If(Node, Vec<Stmt>, Vec<Stmt>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarFx {
    pub kind: Kind,
    /// Its range, in its unit (an `Integer` without one: the range of `int32`).
    pub range: (f64, f64),
    /// Declared (`range`), set (`format`), its value (`const`) or derived.
    pub origin: &'static str,
    /// Its initial value, as a word, an integer or 0/1.
    pub initial: i64,
    /// Its initial value, exactly.
    pub value: f64,
    /// The bits of its word: 32, or fewer (`resolution`, a narrower `format`).
    pub width: u32,
}

impl VarFx {
    /// `Q6.25`, `Q6.10 (17 bits)`, `int32`, `boolean`.
    pub fn label(&self) -> String {
        match self.kind {
            Kind::Fixed(f) if self.width < 32 => format!("Q{}.{f} ({} bits)", self.width as i32 - 1 - f, self.width),
            k => k.label(),
        }
    }

    /// A word into this variable's word: saturated to its width.
    pub fn narrow(&self, word: i64) -> i64 {
        narrow(word, self.width)
    }
}

/// A word saturated to `bits` bits.
pub fn narrow(word: i64, bits: u32) -> i64 {
    if bits >= 32 {
        return word;
    }
    let max = (1i64 << (bits - 1)) - 1;
    word.clamp(-max - 1, max)
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockFx {
    pub kind: BlockKind,
    /// Its state: each field, what it holds, its initial value (the word, exactly).
    pub fields: Vec<(Field, Kind, i64, f64)>,
    /// The values of the step it computes: name, what each holds.
    pub temps: Vec<(&'static str, Kind)>,
    pub out: Kind,
    pub out_range: (f64, f64),
    /// What it computes in a step: `b<n>_out` and its next state.
    pub stmts: Vec<Stmt>,
}

/// A machine's code, in fixed point, at one period.
#[derive(Debug, Clone, PartialEq)]
pub struct MachineFixed {
    pub period_us: u64,
    pub vars: Vec<VarFx>,
    pub blocks: Vec<BlockFx>,
    /// Each equation: its blocks' statements, then the assignment.
    pub equations: Vec<Vec<Stmt>>,
    pub guards: BTreeMap<String, Node>,
    pub actions: BTreeMap<String, Vec<Stmt>>,
}

// --- the arithmetic, as the generated C and VHDL compute it ---------------------------

/// `(x + 2^(n-1)) >> n`, arithmetic; 0 for a shift of 63 or more.
pub fn rshr(x: i64, n: i32) -> i64 {
    match n {
        n if n <= 0 => x,
        n if n >= 63 => 0,
        n => (x + (1i64 << (n - 1))) >> n,
    }
}

pub fn sat32(x: i64) -> i64 {
    x.clamp(i32::MIN as i64, i32::MAX as i64)
}

/// A 32-bit operand aligned: `s > 0` left (exact), `s < 0` rounding right.
pub fn align(a: i64, s: i32) -> i64 {
    if s >= 0 {
        a << s
    } else {
        rshr(a, -s)
    }
}

/// A 64-bit result into a 32-bit word: `d > 0` rounding right, `d < 0`
/// saturating left, then saturated.
pub fn resize(x: i64, d: i32) -> i64 {
    if d >= 0 {
        return sat32(rshr(x, d));
    }
    let l = -d;
    if l >= 32 {
        return match x.cmp(&0) {
            std::cmp::Ordering::Greater => i32::MAX as i64,
            std::cmp::Ordering::Less => i32::MIN as i64,
            std::cmp::Ordering::Equal => 0,
        };
    }
    if x > (i32::MAX as i64) >> l {
        i32::MAX as i64
    } else if x < (i32::MIN as i64) >> l {
        i32::MIN as i64
    } else {
        x << l
    }
}

/// A real number as a word of `frac` fraction bits: rounded to nearest
/// (ties up), saturated.
pub fn quantize(value: f64, frac: i32) -> i64 {
    let scaled = value * 2f64.powi(frac);
    if scaled.is_nan() {
        return 0;
    }
    let rounded = (scaled + 0.5).floor();
    rounded.clamp(i32::MIN as f64, i32::MAX as f64) as i64
}

/// The value of a word.
pub fn value_of(raw: i64, frac: i32) -> f64 {
    raw as f64 * 2f64.powi(-frac)
}

/// The fraction bits of the format of a range.
pub fn frac_for(range: (f64, f64)) -> Result<i32, String> {
    let m = range.0.abs().max(range.1.abs());
    if !m.is_finite() {
        return Err("its range is not finite".to_string());
    }
    let mut i = MIN_INT_BITS;
    while i <= 31 && m >= 2f64.powi(i) {
        i += 1;
    }
    if i > 31 {
        return Err(format!("its range {} .. {} needs more than 31 integer bits", range.0, range.1));
    }
    Ok(31 - i)
}

/// The common fraction bits of operands, and each one's shift to it.
fn alignment(fracs: &[i32]) -> (i32, Vec<i32>) {
    let hi = *fracs.iter().max().expect("operands");
    let lo = *fracs.iter().min().expect("operands");
    let common = hi.min(lo + MAX_ALIGN);
    (common, fracs.iter().map(|f| common - f).collect())
}

// --- lowering -------------------------------------------------------------------------

fn hull(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0.min(b.0), a.1.max(b.1))
}

fn product(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let p = [a.0 * b.0, a.0 * b.1, a.1 * b.0, a.1 * b.1];
    (p.iter().cloned().fold(f64::INFINITY, f64::min), p.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
}

const INT_RANGE: (f64, f64) = (i32::MIN as f64, i32::MAX as f64);

/// Whether an expression reads no variable and no block.
fn constant(e: &Expr) -> bool {
    match e {
        Expr::Const(_) => true,
        Expr::Var(_) | Expr::Block(_) => false,
        Expr::Unary(_, _, a) | Expr::Cast(_, a) => constant(a),
        Expr::Binary(_, _, a, b) => constant(a) && constant(b),
        Expr::Call(_, _, args) => args.iter().all(constant),
    }
}

/// Every constant part of an expression computed, as the model computes it.
fn fold(program: &Program, e: &Expr) -> Expr {
    if constant(e) && !matches!(e, Expr::Const(_)) {
        let mut env = Env::new(program, 10_000);
        return Expr::Const(expr::eval(program, &mut env, e));
    }
    let b = |x: &Expr| Box::new(fold(program, x));
    match e {
        Expr::Unary(op, ty, a) => Expr::Unary(*op, *ty, b(a)),
        Expr::Binary(op, ty, x, y) => Expr::Binary(*op, *ty, b(x), b(y)),
        Expr::Cast(ty, a) => Expr::Cast(*ty, b(a)),
        Expr::Call(f, ty, args) => Expr::Call(*f, *ty, args.iter().map(|a| fold(program, a)).collect()),
        other => other.clone(),
    }
}

fn number(e: &Expr) -> Option<f64> {
    match e {
        Expr::Const(v) => Some(v.number()),
        _ => None,
    }
}

fn real_ty(ty: Ty) -> bool {
    matches!(ty, Ty::Real | Ty::Float)
}

/// A real constant as a node.
pub fn real_const(value: f64) -> Result<Node, String> {
    let frac = if value == 0.0 { MAX_ALIGN } else { frac_for((value, value)).map_err(|e| format!("the constant {value}: {e}"))? };
    Ok(Node { kind: Kind::Fixed(frac), range: (value, value), op: Op::Lit(quantize(value, frac)) })
}

fn frac_of(n: &Node) -> i32 {
    match n.kind {
        Kind::Fixed(f) => f,
        _ => 0,
    }
}

/// A node into the format of `range` (or of `frac`).
/// The format of a range resolved to `step`: the fewest fraction bits whose
/// step is at most `step` (none for a step of 1 or more), the integer bits
/// of the range, a sign bit: the fraction bits and the width of the word.
pub fn resolved(range: (f64, f64), step: f64) -> Result<(i32, u32), String> {
    let int = 31 - frac_for(range)?;
    let frac = (-step.log2()).ceil().max(0.0) as i32;
    let width = 1 + int.max(0) + frac;
    if width > 32 {
        return Err(format!("a resolution of {step} over the range {} .. {} needs a word of {width} bits, more than 32: a coarser resolution, or no resolution (32 bits, the finest the range allows)", range.0, range.1));
    }
    // A range below 1 has negative integer bits: the word keeps its fraction bits.
    Ok((frac, width.max(2) as u32))
}

fn into_frac(n: Node, frac: i32, range: (f64, f64)) -> Node {
    if n.kind == Kind::Fixed(frac) {
        return n;
    }
    let d = frac_of(&n) - frac;
    Node { kind: Kind::Fixed(frac), range, op: Op::Resize { a: Box::new(n), d } }
}

/// `a + b` / `a - b` into the format of its range.
fn add(sub: bool, a: Node, b: Node, what: &str) -> Result<Node, String> {
    let range = if sub { (a.range.0 - b.range.1, a.range.1 - b.range.0) } else { (a.range.0 + b.range.0, a.range.1 + b.range.1) };
    let frac = frac_for(range).map_err(|e| format!("{what}: {e}"))?;
    add_into(sub, a, b, frac, range)
}

/// `a + b` into a given format (it saturates there).
fn add_into(sub: bool, a: Node, b: Node, frac: i32, range: (f64, f64)) -> Result<Node, String> {
    let (common, s) = alignment(&[frac_of(&a), frac_of(&b)]);
    Ok(Node { kind: Kind::Fixed(frac), range, op: Op::Add { sub, a: Box::new(a), b: Box::new(b), sa: s[0], sb: s[1], d: common - frac } })
}

fn mul(a: Node, b: Node, what: &str) -> Result<Node, String> {
    let range = product(a.range, b.range);
    let frac = frac_for(range).map_err(|e| format!("{what}: {e}"))?;
    let d = frac_of(&a) + frac_of(&b) - frac;
    Ok(Node { kind: Kind::Fixed(frac), range, op: Op::Mul { a: Box::new(a), b: Box::new(b), d } })
}

fn compare(op: Bin, a: Node, b: Node) -> Node {
    let (_, s) = alignment(&[frac_of(&a), frac_of(&b)]);
    Node { kind: Kind::Bool, range: (0.0, 1.0), op: Op::Cmp { op, a: Box::new(a), b: Box::new(b), sa: s[0], sb: s[1] } }
}

fn saturate(x: Node, lo: Node, hi: Node, frac: i32, range: (f64, f64)) -> Node {
    let (common, s) = alignment(&[frac_of(&x), frac_of(&lo), frac_of(&hi)]);
    Node { kind: Kind::Fixed(frac), range, op: Op::Sat { x: Box::new(x), lo: Box::new(lo), hi: Box::new(hi), s: [s[0], s[1], s[2]], d: common - frac } }
}

fn read(loc: Loc, kind: Kind, range: (f64, f64)) -> Node {
    Node { kind, range, op: Op::Read(loc) }
}

fn boolean(op: Op) -> Node {
    Node { kind: Kind::Bool, range: (0.0, 1.0), op }
}

struct Lower<'a> {
    program: &'a Program,
    vars: &'a [VarFx],
    /// Each block's output, once lowered.
    outs: Vec<Option<(Kind, (f64, f64))>>,
    what: String,
}

impl Lower<'_> {
    fn lower(&self, e: &Expr) -> Result<Node, String> {
        let what = &self.what;
        let b = |n: Node| Box::new(n);
        Ok(match e {
            Expr::Const(Value::Bool(v)) => Node { kind: Kind::Bool, range: (*v as u8 as f64, *v as u8 as f64), op: Op::Lit(*v as i64) },
            Expr::Const(Value::Int(i)) => Node { kind: Kind::Int, range: (*i as f64, *i as f64), op: Op::Lit(*i as i64) },
            Expr::Const(v) => real_const(v.number()).map_err(|e| format!("{what}: {e}"))?,
            Expr::Var(i) => read(Loc::Var(*i), self.vars[*i].kind, self.vars[*i].range),
            Expr::Block(n) => {
                let (kind, range) = self.outs[*n].ok_or_else(|| format!("{what}: a block is read before it is computed"))?;
                read(Loc::Tmp(*n, "out"), kind, range)
            }
            Expr::Unary(Un::Not, _, a) => boolean(Op::Not(b(self.lower(a)?))),
            Expr::Unary(Un::Neg, Ty::Int, a) => {
                let a = self.lower(a)?;
                Node { kind: Kind::Int, range: (-a.range.1, -a.range.0), op: Op::IntNeg(b(a)) }
            }
            Expr::Unary(Un::Neg, _, a) => {
                let a = self.lower(a)?;
                let range = (-a.range.1, -a.range.0);
                let frac = frac_for(range).map_err(|e| format!("{what}: {e}"))?;
                Node { kind: Kind::Fixed(frac), range, op: Op::Neg { d: frac_of(&a) - frac, a: b(a) } }
            }
            Expr::Binary(op @ (Bin::And | Bin::Or), _, x, y) => boolean(Op::Logic { and: *op == Bin::And, a: b(self.lower(x)?), b: b(self.lower(y)?) }),
            Expr::Binary(op @ (Bin::Eq | Bin::Ne), Ty::Bool, x, y) => boolean(Op::BoolEq { eq: *op == Bin::Eq, a: b(self.lower(x)?), b: b(self.lower(y)?) }),
            Expr::Binary(op, Ty::Int, x, y) if matches!(op, Bin::Lt | Bin::Le | Bin::Gt | Bin::Ge | Bin::Eq | Bin::Ne) => boolean(Op::IntCmp { op: *op, a: b(self.lower(x)?), b: b(self.lower(y)?) }),
            Expr::Binary(op, _, x, y) if matches!(op, Bin::Lt | Bin::Le | Bin::Gt | Bin::Ge | Bin::Eq | Bin::Ne) => compare(*op, self.lower(x)?, self.lower(y)?),
            Expr::Binary(Bin::Div, ty, x, y) => {
                let Some(c) = number(y) else {
                    return Err(format!("{what}: '/' by an expression that is not a constant: the hardware divides only by constants"));
                };
                if *ty == Ty::Int {
                    let x = self.lower(x)?;
                    let range = if c == 0.0 { (0.0, 0.0) } else { product(x.range, (1.0 / c, 1.0 / c)) };
                    Node { kind: Kind::Int, range, op: Op::IntBin { op: Bin::Div, a: b(x), b: b(self.lower(y)?) } }
                } else {
                    if c == 0.0 {
                        return Err(format!("{what}: a division by zero"));
                    }
                    mul(self.lower(x)?, real_const(1.0 / c).map_err(|e| format!("{what}: 1/{c}: {e}"))?, what)?
                }
            }
            Expr::Binary(op, Ty::Int, x, y) => {
                let (x, y) = (self.lower(x)?, self.lower(y)?);
                let range = match op {
                    Bin::Add => (x.range.0 + y.range.0, x.range.1 + y.range.1),
                    Bin::Sub => (x.range.0 - y.range.1, x.range.1 - y.range.0),
                    _ => product(x.range, y.range),
                };
                Node { kind: Kind::Int, range: (range.0.max(INT_RANGE.0), range.1.min(INT_RANGE.1)), op: Op::IntBin { op: *op, a: b(x), b: b(y) } }
            }
            Expr::Binary(op @ (Bin::Add | Bin::Sub), _, x, y) => add(*op == Bin::Sub, self.lower(x)?, self.lower(y)?, what)?,
            Expr::Binary(Bin::Mul, _, x, y) => mul(self.lower(x)?, self.lower(y)?, what)?,
            Expr::Binary(op, ty, _, _) => return Err(format!("{what}: '{}' between {} values is not in the hardware", op.symbol(), ty.name())),
            Expr::Cast(ty, a) if real_ty(*ty) => {
                let inner = self.lower(a)?;
                match inner.kind {
                    Kind::Fixed(_) => inner,
                    Kind::Int => {
                        let range = inner.range;
                        let frac = frac_for(range).map_err(|e| format!("{what}: an Integer used as a Real: {e} (declare a range for the Integer)"))?;
                        Node { kind: Kind::Fixed(frac), range, op: Op::Resize { a: b(inner), d: -frac } }
                    }
                    Kind::Bool => return Err(format!("{what}: a Boolean used as a number")),
                }
            }
            Expr::Cast(ty, _) => return Err(format!("{what}: a conversion to {} is not in the hardware", ty.name())),
            Expr::Call(fun, Ty::Int, args) => {
                let args = args.iter().map(|a| self.lower(a)).collect::<Result<Vec<_>, _>>()?;
                let range = match fun {
                    Fun::Min => (args[0].range.0.min(args[1].range.0), args[0].range.1.min(args[1].range.1)),
                    Fun::Max => (args[0].range.0.max(args[1].range.0), args[0].range.1.max(args[1].range.1)),
                    Fun::Abs => (0.0, args[0].range.0.abs().max(args[0].range.1.abs()).min(INT_RANGE.1)),
                    Fun::Saturate => (args[1].range.0, args[2].range.1),
                };
                Node { kind: Kind::Int, range, op: Op::IntCall { fun: *fun, args } }
            }
            Expr::Call(fun @ (Fun::Min | Fun::Max), _, args) => {
                let (x, y) = (self.lower(&args[0])?, self.lower(&args[1])?);
                let range = if *fun == Fun::Min { (x.range.0.min(y.range.0), x.range.1.min(y.range.1)) } else { (x.range.0.max(y.range.0), x.range.1.max(y.range.1)) };
                let frac = frac_for(range).map_err(|e| format!("{what}: {e}"))?;
                let (common, s) = alignment(&[frac_of(&x), frac_of(&y)]);
                Node { kind: Kind::Fixed(frac), range, op: Op::MinMax { max: *fun == Fun::Max, a: b(x), b: b(y), sa: s[0], sb: s[1], d: common - frac } }
            }
            Expr::Call(Fun::Abs, _, args) => {
                let a = self.lower(&args[0])?;
                let (lo, hi) = a.range;
                let range = if lo >= 0.0 {
                    (lo, hi)
                } else if hi <= 0.0 {
                    (-hi, -lo)
                } else {
                    (0.0, (-lo).max(hi))
                };
                let frac = frac_for(range).map_err(|e| format!("{what}: {e}"))?;
                Node { kind: Kind::Fixed(frac), range, op: Op::Abs { d: frac_of(&a) - frac, a: b(a) } }
            }
            Expr::Call(Fun::Saturate, _, args) => {
                let (x, lo, hi) = (self.lower(&args[0])?, self.lower(&args[1])?, self.lower(&args[2])?);
                let range = (lo.range.0.min(hi.range.1), hi.range.1.max(lo.range.0));
                let frac = frac_for(range).map_err(|e| format!("{what}: {e}"))?;
                saturate(x, lo, hi, frac, range)
            }
        })
    }

    /// A value into a variable: a word into its format (saturating).
    fn assign(&self, a: &Assign) -> Result<Stmt, String> {
        let node = self.lower(&fold(self.program, &a.value))?;
        let v = &self.vars[a.target];
        let node = match v.kind {
            Kind::Fixed(f) if v.width < 32 => {
                let n = into_frac(node, f, v.range);
                Node { kind: n.kind, range: n.range, op: Op::Narrow { a: Box::new(n), bits: v.width } }
            }
            Kind::Fixed(f) => into_frac(node, f, v.range),
            _ => node,
        };
        Ok(Stmt::Set(Loc::Var(a.target), node))
    }
}

/// The blocks an expression reads, in order.
fn blocks_in(e: &Expr, out: &mut Vec<usize>) {
    match e {
        Expr::Block(b) => out.push(*b),
        Expr::Unary(_, _, a) | Expr::Cast(_, a) => blocks_in(a, out),
        Expr::Binary(_, _, a, b) => {
            blocks_in(a, out);
            blocks_in(b, out);
        }
        Expr::Call(_, _, args) => args.iter().for_each(|a| blocks_in(a, out)),
        Expr::Const(_) | Expr::Var(_) => {}
    }
}

/// `y = integrator(...)`, possibly times constants (a unit conversion):
/// the block and the factor from its output to `y`.
fn peel(e: &Expr) -> Option<(usize, f64)> {
    match e {
        Expr::Block(b) => Some((*b, 1.0)),
        Expr::Cast(_, a) => peel(a),
        Expr::Binary(Bin::Mul, _, a, c) | Expr::Binary(Bin::Mul, _, c, a) if number(c).is_some() && peel(a).is_some() => peel(a).map(|(b, f)| (b, f * number(c).expect("a number"))),
        Expr::Binary(Bin::Div, _, a, c) if number(c).is_some_and(|c| c != 0.0) => peel(a).map(|(b, f)| (b, f / number(c).expect("a number"))),
        _ => None,
    }
}

/// The range of an expression over the variables' ranges (`None` where one is unknown).
fn interval(e: &Expr, ranges: &[Option<(f64, f64)>], blocks: &[Option<(f64, f64)>]) -> Option<(f64, f64)> {
    let r = |x: &Expr| interval(x, ranges, blocks);
    Some(match e {
        Expr::Const(v) => (v.number(), v.number()),
        Expr::Var(i) => ranges[*i]?,
        Expr::Block(b) => blocks[*b]?,
        Expr::Unary(Un::Not, _, _) => (0.0, 1.0),
        Expr::Unary(Un::Neg, _, a) => {
            let a = r(a)?;
            (-a.1, -a.0)
        }
        Expr::Cast(_, a) => r(a)?,
        Expr::Binary(op, _, a, b) => match op {
            Bin::Add => {
                let (a, b) = (r(a)?, r(b)?);
                (a.0 + b.0, a.1 + b.1)
            }
            Bin::Sub => {
                let (a, b) = (r(a)?, r(b)?);
                (a.0 - b.1, a.1 - b.0)
            }
            Bin::Mul => product(r(a)?, r(b)?),
            Bin::Div => {
                let c = number(b)?;
                if c == 0.0 {
                    (0.0, 0.0)
                } else {
                    product(r(a)?, (1.0 / c, 1.0 / c))
                }
            }
            _ => (0.0, 1.0),
        },
        Expr::Call(f, _, args) => {
            let a: Vec<(f64, f64)> = args.iter().map(r).collect::<Option<_>>()?;
            match f {
                Fun::Min => (a[0].0.min(a[1].0), a[0].1.min(a[1].1)),
                Fun::Max => (a[0].0.max(a[1].0), a[0].1.max(a[1].1)),
                Fun::Abs => {
                    let (lo, hi) = a[0];
                    if lo >= 0.0 {
                        (lo, hi)
                    } else if hi <= 0.0 {
                        (-hi, -lo)
                    } else {
                        (0.0, (-lo).max(hi))
                    }
                }
                Fun::Saturate => (a[1].0.min(a[2].1), a[2].1.max(a[1].0)),
            }
        }
    })
}

/// The constant parameter `n` of a block.
fn parameter(program: &Program, block: usize, n: usize, what: &str) -> Result<f64, String> {
    let arg = fold(program, &program.blocks[block].args[n]);
    number(&arg).ok_or_else(|| format!("{what}: its gains and limits are numbers in the hardware, not expressions over variables"))
}

fn block_name(kind: BlockKind) -> &'static str {
    match kind {
        BlockKind::Delay => "delay",
        BlockKind::Integrator { .. } => "integrator",
        BlockKind::Pid { .. } => "pid",
    }
}

/// The equation that reads a block: its index and target.
fn equation_of(program: &Program, block: usize) -> Option<&expr::Equation> {
    program.equations.iter().find(|e| {
        let mut found = Vec::new();
        blocks_in(&e.assign.value, &mut found);
        found.contains(&block)
    })
}

/// The range of a block's output (and of its state), from its limits, its
/// input's range or the range of the variable it feeds.
fn block_range(program: &Program, b: usize, ranges: &[Option<(f64, f64)>], what: &str) -> Result<Option<(f64, f64)>, String> {
    let block = &program.blocks[b];
    match block.kind {
        BlockKind::Delay => Ok(interval(&block.args[0], ranges, &[]).map(|r| hull(r, (block.initial.number(), block.initial.number())))),
        BlockKind::Integrator { limited: true } | BlockKind::Pid { limited: true } => {
            let n = if matches!(block.kind, BlockKind::Pid { .. }) { 4 } else { 2 };
            let (lo, hi) = (parameter(program, b, n, what)?, parameter(program, b, n + 1, what)?);
            if lo > hi {
                return Err(format!("{what}: its lower limit {lo} is above its upper limit {hi}"));
            }
            Ok(Some((lo, hi)))
        }
        BlockKind::Integrator { limited: false } => {
            let equation = equation_of(program, b).ok_or_else(|| format!("{what} is in no equation"))?;
            let Some((_, factor)) = peel(&equation.assign.value).filter(|(found, _)| *found == b) else {
                return Err(format!("{what}: an integrator without limits takes the range of the variable it feeds, directly (y = integrator(...), or times a constant); here it is inside an expression: give it limits, integrator(x, k, lo, hi)"));
            };
            Ok(ranges[equation.assign.target].map(|(lo, hi)| {
                let (a, b) = (lo / factor, hi / factor);
                (a.min(b), a.max(b))
            }))
        }
        BlockKind::Pid { limited: false } => Err(format!("{what}: a pid without limits has an integral without bound; the hardware needs its limits, pid(e, kp, ki, kd, lo, hi)")),
    }
}

/// A division by an expression that is not a constant, anywhere in `e`.
fn divides_by_variable(program: &Program, e: &Expr) -> bool {
    match e {
        Expr::Binary(Bin::Div, _, a, b) => number(&fold(program, b)).is_none() || divides_by_variable(program, a),
        Expr::Unary(_, _, a) | Expr::Cast(_, a) => divides_by_variable(program, a),
        Expr::Binary(_, _, a, b) => divides_by_variable(program, a) || divides_by_variable(program, b),
        Expr::Call(_, _, args) => args.iter().any(|a| divides_by_variable(program, a)),
        Expr::Const(_) | Expr::Var(_) | Expr::Block(_) => false,
    }
}

/// The constructs the hardware has not, refused before anything is derived.
fn refuse(program: &Program, machine: &str) -> Result<(), String> {
    let message = |what: String| format!("state_machine '{machine}': {what}: '/' by an expression that is not a constant: the hardware divides only by constants");
    for e in &program.equations {
        if divides_by_variable(program, &e.assign.value) {
            return Err(message(format!("equation \"{}\"", e.text)));
        }
    }
    for b in &program.blocks {
        if b.args.iter().any(|a| divides_by_variable(program, a)) {
            return Err(message("the arguments of a block".to_string()));
        }
    }
    for (text, g) in &program.guards {
        if divides_by_variable(program, g) {
            return Err(message(format!("guard \"{text}\"")));
        }
    }
    for (text, code) in &program.actions {
        if code.iter().any(|a| divides_by_variable(program, &a.value)) {
            return Err(message(format!("action \"{text}\"")));
        }
    }
    Ok(())
}

/// `state_machine 'M': equation "y = pid(...)": pid(...)`.
fn block_what(program: &Program, b: usize, machine: &str) -> String {
    let equation = equation_of(program, b).map(|e| e.text.clone()).unwrap_or_default();
    format!("state_machine '{machine}': equation \"{equation}\": {}(...)", block_name(program.blocks[b].kind))
}

/// The range of every variable: declared, set by a format, a constant's
/// value, or derived from what is assigned to it, again until it stops
/// growing.
fn ranges(program: &Program, machine: &str) -> Result<Vec<(f64, f64, &'static str)>, String> {
    const PASSES: usize = 24;
    let n = program.variables.len();
    let mut known: Vec<Option<(f64, f64)>> = vec![None; n];
    let mut origin: Vec<&'static str> = vec![""; n];
    let mut derived = Vec::new();
    for (i, v) in program.variables.iter().enumerate() {
        let init = v.initial.number();
        (known[i], origin[i]) = match (v.ty, v.range, v.direction) {
            (Ty::Bool, _, _) => (Some((0.0, 1.0)), "boolean"),
            (_, Some(r), _) => (Some(r), "declared"),
            (Ty::Int, None, _) => (Some(INT_RANGE), "int32"),
            (_, None, Direction::Const) => (Some((init, init)), "constant"),
            (_, None, Direction::In) => {
                if let Some((int, frac)) = v.format {
                    (Some((-2f64.powi(int), 2f64.powi(int) - 2f64.powi(-frac))), "format")
                } else {
                    return Err(format!(
                        "state_machine '{machine}': in variable '{}' ({}) has no range: the hardware holds it in fixed point, its format comes from its range; declare it (\"{} range <low> .. <high>\")",
                        v.name,
                        v.ty.name(),
                        v.text.trim()
                    ));
                }
            }
            (_, None, _) => {
                derived.push(i);
                (Some((init, init)), "derived")
            }
        };
    }
    let assigned = |i: usize| -> Vec<&Expr> {
        let mut out: Vec<&Expr> = program.equations.iter().filter(|e| e.assign.target == i).map(|e| &e.assign.value).collect();
        out.extend(program.actions.values().flatten().filter(|a| a.target == i).map(|a| &a.value));
        out
    };
    for _ in 0..PASSES {
        let block_ranges: Vec<Option<(f64, f64)>> = (0..program.blocks.len()).map(|b| block_range(program, b, &known, &block_what(program, b, machine))).collect::<Result<_, _>>()?;
        let mut changed = false;
        for &i in &derived {
            let init = program.variables[i].initial.number();
            let mut range = Some((init, init));
            for value in assigned(i) {
                range = match (range, interval(&fold(program, value), &known, &block_ranges)) {
                    (Some(r), Some(s)) => Some(hull(r, s)),
                    _ => None,
                };
            }
            if range != known[i] {
                changed = true;
                known[i] = range;
            }
        }
        if !changed {
            break;
        }
    }
    // A last pass: what still grows is fed back on itself.
    let block_ranges: Vec<Option<(f64, f64)>> = (0..program.blocks.len()).map(|b| block_range(program, b, &known, &block_what(program, b, machine))).collect::<Result<_, _>>()?;
    for &i in &derived {
        let v = &program.variables[i];
        let mut grows = false;
        let mut unknown = false;
        for value in assigned(i) {
            match (known[i], interval(&fold(program, value), &known, &block_ranges)) {
                (Some(r), Some(s)) => grows |= s.0 < r.0 || s.1 > r.1,
                _ => unknown = true,
            }
        }
        if unknown || known[i].is_none() {
            return Err(format!(
                "state_machine '{machine}': the range of variable '{}' cannot be derived (what is assigned to it reads a value of unknown range): declare it (\"{} range <low> .. <high>\")",
                v.name,
                v.text.trim()
            ));
        }
        if grows || known[i].is_some_and(|r| !r.0.is_finite() || !r.1.is_finite()) {
            return Err(format!(
                "state_machine '{machine}': the range of variable '{}' cannot be derived: it is fed back and grows at every pass; declare it (\"{} range <low> .. <high>\")",
                v.name,
                v.text.trim()
            ));
        }
    }
    Ok(known.into_iter().zip(origin).map(|(r, o)| {
        let r = r.expect("known");
        (r.0, r.1, o)
    }).collect())
}

/// Lower one block: its state, its temps, its statements.
fn lower_block(l: &Lower, b: usize, range: (f64, f64), period_us: u64, what: &str) -> Result<BlockFx, String> {
    let program = l.program;
    let block = &program.blocks[b];
    let t = expr::period_seconds(period_us);
    let tmp = |name: &'static str, n: &Node| read(Loc::Tmp(b, name), n.kind, n.range);
    let field = |f: Field, kind: Kind, range: (f64, f64)| read(Loc::Blk(b, f), kind, range);
    let lowered = |n: usize| -> Result<Node, String> { l.lower(&fold(program, &block.args[n])) };
    match block.kind {
        BlockKind::Delay => {
            let x = lowered(0)?;
            let (kind, initial) = match x.kind {
                Kind::Fixed(_) => {
                    let frac = frac_for(range).map_err(|e| format!("{what}: {e}"))?;
                    (Kind::Fixed(frac), quantize(block.initial.number(), frac))
                }
                other => (other, block.initial.number() as i64),
            };
            let held = field(Field::Held, kind, range);
            let x = match kind {
                Kind::Fixed(f) => into_frac(x, f, range),
                _ => x,
            };
            Ok(BlockFx {
                kind: block.kind,
                fields: vec![(Field::Held, kind, initial, block.initial.number())],
                temps: vec![("out", kind)],
                out: kind,
                out_range: range,
                stmts: vec![Stmt::Set(Loc::Tmp(b, "out"), held), Stmt::Set(Loc::Blk(b, Field::Held), x)],
            })
        }
        BlockKind::Integrator { limited } => {
            let x = lowered(0)?;
            let k = parameter(program, b, 1, what)?;
            let frac = frac_for(range).map_err(|e| format!("{what}: its state: {e}"))?;
            let s = field(Field::Held, Kind::Fixed(frac), range);
            let kt = real_const(k * t).map_err(|e| format!("{what}: k*T: {e}"))?;
            let inc = mul(kt, x, &format!("{what}: k*T*x"))?;
            let next = add_into(false, s.clone(), inc, frac, range)?;
            let (out, next) = if limited {
                let (lo, hi) = (real_const(range.0)?, real_const(range.1)?);
                (saturate(s, lo.clone(), hi.clone(), frac, range), saturate(next, lo, hi, frac, range))
            } else {
                (s, next)
            };
            Ok(BlockFx {
                kind: block.kind,
                fields: vec![(Field::Held, Kind::Fixed(frac), 0, 0.0)],
                temps: vec![("out", Kind::Fixed(frac))],
                out: Kind::Fixed(frac),
                out_range: range,
                stmts: vec![Stmt::Set(Loc::Tmp(b, "out"), out), Stmt::Set(Loc::Blk(b, Field::Held), next)],
            })
        }
        BlockKind::Pid { .. } => {
            let e = lowered(0)?;
            let (kp, ki, kd) = (parameter(program, b, 1, what)?, parameter(program, b, 2, what)?, parameter(program, b, 3, what)?);
            let (kp_c, kit_c, kdt_c) = (real_const(kp)?, real_const(ki * t)?, real_const(kd / t)?);
            let e_max = e.range.0.abs().max(e.range.1.abs());
            let bound = kp.abs() * e_max + (kd / t).abs() * 2.0 * e_max + (ki * t).abs() * e_max;
            let i_range = (range.0 - bound, range.1 + bound);
            let i_frac = frac_for(i_range).map_err(|e| format!("{what}: its integral: {e}"))?;
            let out_frac = frac_for(range).map_err(|e| format!("{what}: {e}"))?;
            let e_t = tmp("e", &e);
            let prev = Node { kind: e.kind, range: e.range, op: Op::Select { c: Box::new(field(Field::Primed, Kind::Bool, (0.0, 1.0))), a: Box::new(field(Field::Previous, e.kind, e.range)), b: Box::new(e_t.clone()) } };
            let prev_t = tmp("prev", &prev);
            let integral = field(Field::Integral, Kind::Fixed(i_frac), i_range);
            let p = mul(kp_c, e_t.clone(), &format!("{what}: kp*e"))?;
            let dif = add(true, e_t.clone(), prev_t, &format!("{what}: e - e_previous"))?;
            let dterm = mul(kdt_c, dif, &format!("{what}: kd*(e - e_previous)/T"))?;
            let pi = add(false, p, integral.clone(), &format!("{what}: kp*e + integral"))?;
            let raw = add(false, pi, dterm, &format!("{what}: its output before the limits"))?;
            let raw_t = tmp("raw", &raw);
            let (lo, hi) = (real_const(range.0)?, real_const(range.1)?);
            let zero = Node { kind: e.kind, range: (0.0, 0.0), op: Op::Lit(0) };
            let winding = boolean(Op::Logic {
                and: false,
                a: Box::new(boolean(Op::Logic { and: true, a: Box::new(compare(Bin::Gt, raw_t.clone(), hi.clone())), b: Box::new(compare(Bin::Gt, e_t.clone(), zero.clone())) })),
                b: Box::new(boolean(Op::Logic { and: true, a: Box::new(compare(Bin::Lt, raw_t.clone(), lo.clone())), b: Box::new(compare(Bin::Lt, e_t.clone(), zero)) })),
            });
            let wind_t = read(Loc::Tmp(b, "winding"), Kind::Bool, (0.0, 1.0));
            let ie = mul(kit_c, e_t.clone(), &format!("{what}: ki*T*e"))?;
            let next = add_into(false, integral, ie, i_frac, i_range)?;
            let out = saturate(raw_t, lo, hi, out_frac, range);
            Ok(BlockFx {
                kind: block.kind,
                fields: vec![(Field::Integral, Kind::Fixed(i_frac), 0, 0.0), (Field::Previous, e.kind, 0, 0.0), (Field::Primed, Kind::Bool, 0, 0.0)],
                temps: vec![("e", e.kind), ("prev", e.kind), ("raw", raw.kind), ("winding", Kind::Bool), ("out", Kind::Fixed(out_frac))],
                out: Kind::Fixed(out_frac),
                out_range: range,
                stmts: vec![
                    Stmt::Set(Loc::Tmp(b, "e"), e),
                    Stmt::Set(Loc::Tmp(b, "prev"), prev),
                    Stmt::Set(Loc::Tmp(b, "raw"), raw),
                    Stmt::Set(Loc::Tmp(b, "winding"), winding),
                    Stmt::If(boolean(Op::Not(Box::new(wind_t))), vec![Stmt::Set(Loc::Blk(b, Field::Integral), next)], vec![]),
                    Stmt::Set(Loc::Blk(b, Field::Previous), e_t),
                    Stmt::Set(Loc::Blk(b, Field::Primed), Node { kind: Kind::Bool, range: (1.0, 1.0), op: Op::Lit(1) }),
                    Stmt::Set(Loc::Tmp(b, "out"), out),
                ],
            })
        }
    }
}

/// The code of a machine in fixed point, at a period; an error names what
/// the hardware cannot compute.
pub fn machine(m: &MachinePlan, period_us: u64) -> Result<MachineFixed, String> {
    let program = m.program();
    let name = &m.name;
    refuse(program, name)?;
    let known = ranges(program, name)?;
    let mut vars = Vec::new();
    for (i, v) in program.variables.iter().enumerate() {
        let (lo, hi, origin) = known[i];
        let what = format!("state_machine '{name}': variable '{}'", v.name);
        let (kind, origin, width) = match v.ty {
            Ty::Bool => (Kind::Bool, origin, 1),
            Ty::Int => (Kind::Int, origin, 32),
            _ => match (v.format, v.resolution) {
                (Some((int, frac)), _) => (Kind::Fixed(frac), "format", (int + frac + 1) as u32),
                (None, Some(step)) => {
                    let (frac, width) = resolved((lo, hi), step).map_err(|e| format!("{what}: {e}"))?;
                    (Kind::Fixed(frac), origin, width)
                }
                (None, None) => (Kind::Fixed(frac_for((lo, hi)).map_err(|e| format!("{what}: {e}"))?), origin, 32),
            },
        };
        let initial = match kind {
            Kind::Fixed(f) => narrow(quantize(v.initial.number(), f), width),
            _ => v.initial.number() as i64,
        };
        vars.push(VarFx { kind, range: (lo, hi), origin, initial, value: v.initial.number(), width });
    }
    let plain: Vec<Option<(f64, f64)>> = vars.iter().map(|v| Some(v.range)).collect();
    let mut l = Lower { program, vars: &vars, outs: vec![None; program.blocks.len()], what: String::new() };
    let mut blocks = Vec::new();
    for b in 0..program.blocks.len() {
        let equation = equation_of(program, b).map(|e| e.text.clone()).unwrap_or_default();
        let what = format!("state_machine '{name}': equation \"{equation}\": {}(...)", block_name(program.blocks[b].kind));
        let range = block_range(program, b, &plain, &what)?.ok_or_else(|| format!("{what}: the range of its output cannot be derived: declare the range of the variable it feeds"))?;
        l.what = what.clone();
        let fx = lower_block(&l, b, range, period_us, &what)?;
        l.outs[b] = Some((fx.out, fx.out_range));
        blocks.push(fx);
    }
    let mut equations = Vec::new();
    for e in &program.equations {
        l.what = format!("state_machine '{name}': equation \"{}\"", e.text);
        let mut found = Vec::new();
        blocks_in(&e.assign.value, &mut found);
        let mut stmts: Vec<Stmt> = found.iter().flat_map(|b| blocks[*b].stmts.clone()).collect();
        stmts.push(l.assign(&e.assign)?);
        equations.push(stmts);
    }
    let mut guards = BTreeMap::new();
    for (text, g) in &program.guards {
        l.what = format!("state_machine '{name}': guard \"{text}\"");
        guards.insert(text.clone(), l.lower(&fold(program, g))?);
    }
    let mut actions = BTreeMap::new();
    for (text, code) in &program.actions {
        l.what = format!("state_machine '{name}': action \"{text}\"");
        actions.insert(text.clone(), code.iter().map(|a| l.assign(a)).collect::<Result<Vec<_>, _>>()?);
    }
    Ok(MachineFixed { period_us, vars, blocks, equations, guards, actions })
}

// --- the quantisation error -----------------------------------------------------------

/// Half a unit in the last place of `frac` fraction bits, when a shift rounds.
fn half_ulp(rounds: bool, frac: i32) -> f64 {
    if rounds {
        0.5 * 2f64.powi(-frac)
    } else {
        0.0
    }
}

fn magnitude(n: &Node) -> f64 {
    n.range.0.abs().max(n.range.1.abs())
}

/// A node's error as a first-order affine form: `sum(coef[l] * err(l)) +
/// d`, `|d| <= free`, where err(l) is the signed error of what it reads.
/// Linear code (sums, differences, products by a constant, resizes) keeps
/// the coefficients, so an error read twice can cancel (`f + 0.2 * (v - f)`
/// carries 0.8 of f's error, not 1.2); anything else collapses to a bound.
#[derive(Debug, Clone, Default)]
struct Affine {
    coef: BTreeMap<Loc, f64>,
    free: f64,
}

impl Affine {
    fn bound(&self, e: &BTreeMap<Loc, f64>) -> f64 {
        self.coef.iter().map(|(l, c)| c.abs() * e.get(l).copied().unwrap_or(0.0)).sum::<f64>() + self.free
    }

    fn only(free: f64) -> Affine {
        Affine { coef: BTreeMap::new(), free }
    }

    fn scaled(mut self, k: f64) -> Affine {
        for c in self.coef.values_mut() {
            *c *= k;
        }
        self.free *= k.abs();
        self
    }

    fn plus(mut self, other: Affine, sign: f64) -> Affine {
        for (l, c) in other.coef {
            *self.coef.entry(l).or_insert(0.0) += sign * c;
        }
        self.free += other.free;
        self
    }

    fn with(mut self, rounding: f64) -> Affine {
        self.free += rounding;
        self
    }
}

/// A constant's exact value, when a node is one.
fn constant_of(n: &Node) -> Option<f64> {
    match n.op {
        Op::Lit(_) if matches!(n.kind, Kind::Fixed(_)) && n.range.0 == n.range.1 => Some(n.range.0),
        _ => None,
    }
}

fn affine(n: &Node, e: &BTreeMap<Loc, f64>) -> Affine {
    let frac = frac_of(n);
    let collapsed = |x: f64| Affine::only(x);
    match &n.op {
        Op::Lit(raw) => match n.kind {
            Kind::Fixed(f) => collapsed((n.range.0 - value_of(*raw, f)).abs()),
            _ => Affine::default(),
        },
        Op::Read(loc) => Affine { coef: BTreeMap::from([(loc.clone(), 1.0)]), free: 0.0 },
        Op::Add { sub, a, b, sa, sb, d } => {
            let common = frac_of(a) + sa;
            affine(a, e).plus(affine(b, e), if *sub { -1.0 } else { 1.0 }).with(half_ulp(*sa < 0, common) + half_ulp(*sb < 0, common) + half_ulp(*d > 0, frac))
        }
        Op::Mul { a, b, d } => {
            let (fa, fb) = (affine(a, e), affine(b, e));
            let (ea, eb) = (fa.bound(e), fb.bound(e));
            let rounding = half_ulp(*d > 0, frac);
            // A product by a constant is linear: its error is the other's,
            // scaled, and the constant's own (its rounding) times the other.
            match (constant_of(a), constant_of(b)) {
                (_, Some(k)) => fa.scaled(k).with(magnitude(a) * eb + ea * eb + rounding),
                (Some(k), _) => fb.scaled(k).with(magnitude(b) * ea + ea * eb + rounding),
                _ => collapsed(magnitude(a) * eb + magnitude(b) * ea + ea * eb + rounding),
            }
        }
        Op::Neg { a, d } => affine(a, e).scaled(-1.0).with(half_ulp(*d > 0, frac)),
        Op::Abs { a, d } => collapsed(affine(a, e).bound(e) + half_ulp(*d > 0, frac)),
        Op::Resize { a, d } => match a.kind {
            Kind::Fixed(_) => affine(a, e).with(half_ulp(*d > 0, frac)),
            _ => collapsed(half_ulp(*d > 0, frac)),
        },
        Op::MinMax { a, b, sa, sb, d, .. } => {
            let common = frac_of(a) + sa;
            collapsed(affine(a, e).bound(e).max(affine(b, e).bound(e)) + half_ulp(*sa < 0 || *sb < 0, common) + half_ulp(*d > 0, frac))
        }
        Op::Sat { x, lo, hi, s, d } => {
            let common = frac_of(x) + s[0];
            let worst = affine(x, e).bound(e).max(affine(lo, e).bound(e)).max(affine(hi, e).bound(e));
            collapsed(worst + half_ulp(s.iter().any(|v| *v < 0), common) + half_ulp(*d > 0, frac))
        }
        Op::Select { a, b, .. } => collapsed(affine(a, e).bound(e).max(affine(b, e).bound(e))),
        Op::Narrow { a, .. } => affine(a, e),
        _ => Affine::default(),
    }
}

/// A bound of how far a node may be from the value the model computes,
/// given the bounds of what it reads.
fn error(n: &Node, e: &BTreeMap<Loc, f64>) -> f64 {
    affine(n, e).bound(e)
}

fn run_error(stmts: &[Stmt], e: &mut BTreeMap<Loc, f64>) {
    for s in stmts {
        match s {
            Stmt::Set(loc, n) => {
                let v = error(n, e);
                e.insert(loc.clone(), v);
            }
            Stmt::If(_, a, b) => {
                let (mut x, mut y) = (e.clone(), e.clone());
                run_error(a, &mut x);
                run_error(b, &mut y);
                for (loc, v) in x.into_iter().chain(y) {
                    let old = e.get(&loc).copied().unwrap_or(0.0);
                    e.insert(loc, old.max(v));
                }
            }
        }
    }
}

/// For each variable, a bound of how far its fixed-point value may be from
/// the floating-point model's over `steps` steps: every rounding of the
/// code (half a unit of the format it rounds into), propagated through
/// what reads it (a product by the largest magnitude of the other operand),
/// step after step; an action may run in any step. `inputs` is how far each
/// variable's value is from its word when the case sets it (the inputs).
pub fn bounds(m: &MachineFixed, steps: usize, inputs: &[f64]) -> Vec<f64> {
    let mut e: BTreeMap<Loc, f64> = BTreeMap::new();
    for (i, v) in m.vars.iter().enumerate() {
        let start = match v.kind {
            Kind::Fixed(f) => (v.value - value_of(v.initial, f)).abs().max(inputs.get(i).copied().unwrap_or(0.0)),
            _ => 0.0,
        };
        e.insert(Loc::Var(i), start);
    }
    for (b, block) in m.blocks.iter().enumerate() {
        for (field, kind, raw, value) in &block.fields {
            if let Kind::Fixed(f) = kind {
                e.insert(Loc::Blk(b, *field), (value - value_of(*raw, *f)).abs());
            }
        }
    }
    let mut worst: Vec<f64> = (0..m.vars.len()).map(|i| e[&Loc::Var(i)]).collect();
    for _ in 0..steps {
        for stmts in &m.equations {
            run_error(stmts, &mut e);
        }
        for code in m.actions.values() {
            for s in code {
                if let Stmt::Set(loc, n) = s {
                    let v = error(n, &e);
                    let old = e.get(loc).copied().unwrap_or(0.0);
                    e.insert(loc.clone(), old.max(v));
                }
            }
        }
        for (i, w) in worst.iter_mut().enumerate() {
            *w = w.max(e[&Loc::Var(i)]);
        }
    }
    worst
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_format_of_a_range_has_the_fewest_integer_bits_that_hold_it() {
        assert_eq!(frac_for((0.0, 60.0)), Ok(25), "60 < 2^6: Q6.25");
        assert_eq!(frac_for((-20.0, 16.0)), Ok(26), "20 < 2^5: Q5.26");
        assert_eq!(frac_for((0.0, 1.0)), Ok(30), "1 is not below 2^0: Q1.30");
        assert_eq!(frac_for((0.0, 0.25)), Ok(32), "0.25 < 2^-1: Q-1.32");
        assert!(frac_for((0.0, 3e9)).unwrap_err().contains("more than 31 integer bits"));
        assert!(frac_for((0.0, f64::INFINITY)).is_err());
    }

    #[test]
    fn rounding_is_to_nearest_ties_up_and_saturation_holds_the_word() {
        assert_eq!(rshr(5, 1), 3, "2.5 rounds up");
        assert_eq!(rshr(-5, 1), -2, "-2.5 rounds up, toward +inf");
        assert_eq!(rshr(-6, 1), -3);
        assert_eq!(rshr(1 << 40, 63), 0);
        assert_eq!(resize(1 << 40, 0), i32::MAX as i64, "saturates");
        assert_eq!(resize(-(1 << 40), 0), i32::MIN as i64);
        assert_eq!(resize(3, -4), 48, "left, exact");
        assert_eq!(resize(1 << 28, -4), i32::MAX as i64, "left, saturating");
        assert_eq!(resize(-1, -40), i32::MIN as i64);
        assert_eq!(resize(0, -40), 0);
        assert_eq!(quantize(54.6, 25), 1_832_071_987, "54.6 * 2^25 = 1832071987.2");
        assert_eq!(quantize(-0.5 * 2f64.powi(-25), 25), 0, "a tie rounds up");
        assert_eq!(quantize(1e12, 25), i32::MAX as i64);
        assert_eq!(align(3, 4), 48);
        assert_eq!(align(48 + 8, -4), 4);
    }

    fn charger_like(variables: &[&str], equations: &[&str]) -> Result<MachineFixed, String> {
        let text = format!(
            "state_machine \"M\" {{\n variables: [{}]\n equations: [{}]\n initial: \"A\"\n state A {{}}\n}}\n",
            variables.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(", "),
            equations.iter().map(|v| format!("\"{v}\"")).collect::<Vec<_>>().join(", ")
        );
        let model = crate::compiler::Compiler::new(crate::compiler::CompilerConfig::default()).compile_string(&text).map_err(|e| e.to_string())?;
        let plan = super::super::plan::plan(&model.ast)?;
        machine(&plan.machines[0], 10_000)
    }

    #[test]
    fn ranges_are_declared_derived_or_refused() {
        let m = charger_like(&["in v: Real [V] range 0 V .. 60 V", "target: Real [A] = 0 A", "out e: Real [A]"], &["e = target - 3 A"]).unwrap();
        assert_eq!(m.vars[0].kind, Kind::Fixed(25));
        assert_eq!((m.vars[2].range, m.vars[2].origin), ((-3.0, 0.0), "derived"));
        let fed_back = charger_like(&["in v: Real [V] range 0 V .. 60 V", "f: Real [V]"], &["f = f + 0.2 * (v - f)"]).unwrap_err();
        assert!(fed_back.contains("variable 'f' cannot be derived: it is fed back"), "{fed_back}");
        let unknown = charger_like(&["in v: Real [V]", "out f: Real [V]"], &["f = v"]).unwrap_err();
        assert!(unknown.contains("in variable 'v' (Real) has no range"), "{unknown}");
        let set = charger_like(&["in v: Real [V] range 0 V .. 60 V format Q10.21", "out f: Real [V]"], &["f = v"]).unwrap();
        assert_eq!((set.vars[0].kind, set.vars[0].origin), (Kind::Fixed(21), "format"));
    }

    #[test]
    fn what_the_hardware_cannot_compute_is_refused_by_name() {
        let div = charger_like(&["in a: Real range 1 .. 2", "in b: Real range 1 .. 2", "out y: Real"], &["y = a / b"]).unwrap_err();
        assert!(div.contains("'/' by an expression that is not a constant"), "{div}");
        let pid = charger_like(&["in e: Real [A] range -1 A .. 1 A", "out y: Real [A]"], &["y = pid(e, 1, 1 Hz, 0 s)"]).unwrap_err();
        assert!(pid.contains("a pid without limits"), "{pid}");
        let gain = charger_like(&["in e: Real range -1 .. 1", "in k: Real range 0 .. 1", "out y: Real [s] range 0 s .. 10 s"], &["y = integrator(e, k, 0 s, 10 s)"]).unwrap_err();
        assert!(gain.contains("its gains and limits are numbers"), "{gain}");
        let inside = charger_like(&["in e: Real range -1 .. 1", "out y: Real [s] range 0 s .. 10 s"], &["y = 1 s + integrator(e, 1)"]).unwrap_err();
        assert!(inside.contains("inside an expression"), "{inside}");
        assert!(charger_like(&["in a: Real range 0 .. 2", "out y: Real"], &["y = a / 4"]).is_ok(), "a constant divides");
    }

    #[test]
    fn the_quantisation_bound_grows_with_the_steps_of_an_integrator() {
        let m = charger_like(&["in i: Real [A] range -20 A .. 20 A", "out q: Real [Ah] range 0 Ah .. 1 Ah"], &["q = integrator(i, 1)"]).unwrap();
        let one = bounds(&m, 1, &[0.0, 0.0]);
        let many = bounds(&m, 100, &[0.0, 0.0]);
        assert!(many[1] > one[1] && one[1] > 0.0, "{one:?} {many:?}");
        assert!(many[1] < 1e-5, "small next to the range: {many:?}");
    }
}
