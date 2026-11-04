//! The registers through which software drives a machine the hardware
//! computes: 32-bit words on a Wishbone bus (B4 classic, as LiteX's
//! `wishbone.Interface`, word addressed), in a fixed order:
//!
//! | Register | Access | What |
//! |---|---|---|
//! | `CTRL` | write | bit 0 START: one step with the inputs written; bit 1 RESET: back to `init()` |
//! | `STATUS` | read | bit 0 READY (a step may start), bit 1 DONE (the outputs are those of the last step; cleared by START) |
//! | `CYCLES` | read | the clock cycles the last step took |
//! | `EV_COUNT`, `EV_QUEUE<k>` | read/write | the events of the step, in the order they arrived (16 at most) |
//! | `COND<n>` | read/write | the conditions, 32 a word, bit k = condition 32n + k |
//! | `IN_<variable>` | read/write | an `in` variable: its word (Q format), its integer, bit 0 |
//! | `STATE<n>` | read | the states, 32 a word: active after the step |
//! | `ACT_<action>` | read | the times an action ran in the step (0 to 255) |
//! | `VAR_<variable>` | read | a variable after the step |
//! | `BUDGET<n>` | read | the budgets, 32 a word: overrun |
//!
//! The same map is written as JSON (`<block>_regs.json`), as LiteX's
//! `csr.csv` lines (`<block>_csr.csv`) and as a C driver (`<block>_hw.h`).

use super::fixed::{Kind, MachineFixed};
use super::plan::{MachinePlan, Plan};
use std::fmt::Write as _;

/// The events a step takes, at most (the C's default `QUEUE_SIZE`).
pub const QUEUE: usize = 16;

#[derive(Debug, Clone, PartialEq)]
pub enum Reg {
    Ctrl,
    Status,
    Cycles,
    EvCount,
    EvQueue(usize),
    Cond(usize),
    In(usize),
    State(usize),
    Action(usize),
    Var(usize),
    Budget(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Register {
    pub name: String,
    /// Its word index (the byte offset is 4 times it).
    pub index: usize,
    pub write: bool,
    pub reg: Reg,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Map {
    /// The block: `<model>_<machine>`.
    pub block: String,
    pub registers: Vec<Register>,
}

impl Map {
    pub fn address_bits(&self) -> usize {
        let mut bits = 1;
        while (1usize << bits) < self.registers.len() {
            bits += 1;
        }
        bits
    }

    pub fn find(&self, reg: &Reg) -> Option<&Register> {
        self.registers.iter().find(|r| &r.reg == reg)
    }

    /// The registers read after a step, in order.
    pub fn outputs(&self) -> Vec<&Register> {
        self.registers.iter().filter(|r| matches!(r.reg, Reg::State(_) | Reg::Action(_) | Reg::Var(_) | Reg::Budget(_) | Reg::Cycles)).collect()
    }
}

pub fn words(n: usize) -> usize {
    n.div_ceil(32)
}

fn upper(ident: &str) -> String {
    ident.to_uppercase()
}

pub fn map(plan: &Plan, m: &MachinePlan, fx: &MachineFixed) -> Map {
    let mut regs: Vec<(String, bool, Reg, String)> = vec![
        ("CTRL".into(), true, Reg::Ctrl, "bit 0 START: one step; bit 1 RESET: back to init()".into()),
        ("STATUS".into(), false, Reg::Status, "bit 0 READY, bit 1 DONE".into()),
        ("CYCLES".into(), false, Reg::Cycles, "clock cycles the last step took".into()),
    ];
    if !m.events.is_empty() {
        regs.push(("EV_COUNT".into(), true, Reg::EvCount, format!("how many events the step takes (at most {QUEUE})")));
        for k in 0..QUEUE {
            regs.push((format!("EV_QUEUE{k}"), true, Reg::EvQueue(k), format!("event {k} of the step, its code (0 to {})", m.events.len() - 1)));
        }
    }
    for n in 0..words(m.conditions.len()) {
        let names: Vec<String> = m.conditions.iter().skip(32 * n).take(32).map(|c| format!("\"{}\"", c.text)).collect();
        regs.push((format!("COND{n}"), true, Reg::Cond(n), format!("conditions, bit k = {}", names.join(", "))));
    }
    for (i, v) in m.program().inputs() {
        regs.push((format!("IN_{}", upper(&m.variables[i].ident)), true, Reg::In(i), format!("in variable \"{}\": {}", v.name, fx.vars[i].label())));
    }
    for n in 0..words(m.states.len()) {
        regs.push((format!("STATE{n}"), false, Reg::State(n), format!("states {} to {}, active", 32 * n, (32 * n + 31).min(m.states.len() - 1))));
    }
    for (a, act) in m.actions.iter().enumerate() {
        regs.push((format!("ACT_{}", upper(&act.ident)), false, Reg::Action(a), format!("action \"{}\": times run in the step", act.text)));
    }
    for (i, v) in m.program().variables.iter().enumerate() {
        regs.push((format!("VAR_{}", upper(&m.variables[i].ident)), false, Reg::Var(i), format!("variable \"{}\" after the step: {}", v.name, fx.vars[i].label())));
    }
    for n in 0..words(m.budgets.len()) {
        regs.push((format!("BUDGET{n}"), false, Reg::Budget(n), "budgets overrun, bit k = budget 32n + k".into()));
    }
    Map {
        block: format!("{}_{}", plan.ident, m.ident),
        registers: regs.into_iter().enumerate().map(|(index, (name, write, reg, description))| Register { name, index, write, reg, description }).collect(),
    }
}

fn json_string(s: &str) -> String {
    serde_json::to_string(s).expect("a string")
}

/// The map as JSON, machine-readable.
pub fn json(map: &Map, m: &MachinePlan, fx: &MachineFixed) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{\n  \"block\": {},\n  \"machine\": {},\n  \"bus\": \"wishbone-b4-classic\",\n  \"data_width\": 32,\n  \"addressing\": \"word\",\n  \"address_bits\": {},\n  \"registers\": [", json_string(&map.block), json_string(&m.name), map.address_bits());
    let rows: Vec<String> = map
        .registers
        .iter()
        .map(|r| {
            let format = match r.reg {
                Reg::In(i) | Reg::Var(i) => format!(", \"format\": {}", json_string(&fx.vars[i].label())),
                _ => String::new(),
            };
            format!(
                "    {{ \"name\": {}, \"index\": {}, \"offset\": \"0x{:03X}\", \"access\": \"{}\"{format}, \"description\": {} }}",
                json_string(&r.name),
                r.index,
                4 * r.index,
                if r.write { "rw" } else { "ro" },
                json_string(&r.description)
            )
        })
        .collect();
    let _ = writeln!(out, "{}\n  ]\n}}", rows.join(",\n"));
    out
}

/// The map as LiteX `csr.csv` lines, at base 0 (add the base of the block's region).
pub fn csv(map: &Map) -> String {
    let mut out = format!("#--------------------------------------------------------------------------------\n# {} registers (Wishbone, byte offsets from the block's base)\n#--------------------------------------------------------------------------------\ncsr_base,{},0x00000000,,\n", map.block, map.block);
    for r in &map.registers {
        let _ = writeln!(out, "csr_register,{}_{},0x{:08x},1,{}", map.block, r.name.to_lowercase(), 4 * r.index, if r.write { "rw" } else { "ro" });
    }
    out
}

/// The C driver: the register offsets, and one step of the machine through
/// them from the software's structures (`<model>.h`), values converted to
/// and from the words.
pub fn driver(plan: &Plan, m: &MachinePlan, fx: &MachineFixed, map: &Map) -> String {
    let b = &map.block;
    let up = b.to_uppercase();
    let t = format!("{}_{}", plan.ident, m.ident);
    let mut out = format!(
        "/*\n * The driver of block {b}: state_machine \"{}\" computed by the programmable logic.\n * Generated by ArcLang. Do not edit: regenerate. See docs/embedded-fpga.md.\n * {up}_HW_BASE is the address of the block on the bus (its Wishbone region in the SoC).\n */\n#ifndef ARCLANG_{up}_HW_H\n#define ARCLANG_{up}_HW_H\n\n#include <stdbool.h>\n#include <stdint.h>\n#include <string.h>\n#include \"{}.h\"\n\n#ifndef {up}_HW_BASE\n#error \"define {up}_HW_BASE, the address of the block {b} on the bus\"\n#endif\n\n",
        super::cgen::comment(&m.name),
        plan.ident
    );
    for r in &map.registers {
        let _ = writeln!(out, "#define {up}_{} 0x{:03X}U /* {}{} */", r.name, 4 * r.index, if r.write { "rw: " } else { "ro: " }, super::cgen::comment(&r.description));
    }
    let _ = writeln!(out, "\n/* How many times the status is read before a step is given up. */\n#ifndef {up}_HW_POLLS\n#define {up}_HW_POLLS 100000UL\n#endif\n");
    out.push_str(super::fxc::CONVERSIONS);
    if m.program().inputs().any(|(i, _)| fx.vars[i].width < 32 && matches!(fx.vars[i].kind, Kind::Fixed(_))) {
        out.push('\n');
        out.push_str(&super::fxc::NARROW.replace("static int32_t", "static inline int32_t"));
    }
    let _ = writeln!(
        out,
        "\nstatic inline void {b}_hw_write(uint32_t offset, uint32_t value)\n{{\n    *(volatile uint32_t *)(uintptr_t)({up}_HW_BASE + offset) = value;\n}}\n\nstatic inline uint32_t {b}_hw_read(uint32_t offset)\n{{\n    return *(volatile uint32_t *)(uintptr_t)({up}_HW_BASE + offset);\n}}\n\n/* Back to init(): no state active until the next step enters the initial ones. */\nstatic inline void {b}_hw_reset(void)\n{{\n    {b}_hw_write({up}_CTRL, 2U);\n}}\n\n/* One step: the inputs written, START, DONE awaited, the outputs read.\n   0, or -1 when the block never became ready or never finished. */\nstatic inline int {b}_hw_step(const {t}_inputs *in, {t}_outputs *out)\n{{\n    unsigned long n = 0UL;\n    uint32_t word = 0U;\n    while ((({b}_hw_read({up}_STATUS) & 1U) == 0U) && (n < {up}_HW_POLLS)) {{\n        n++;\n    }}\n    if (n >= {up}_HW_POLLS) {{\n        return -1;\n    }}"
    );
    if !m.events.is_empty() {
        let _ = writeln!(out, "    {{\n        uint32_t k;\n        uint32_t count = (in->queued < {QUEUE}U) ? (uint32_t)in->queued : {QUEUE}U;\n        {b}_hw_write({up}_EV_COUNT, count);\n        for (k = 0U; k < count; k++) {{\n            {b}_hw_write({up}_EV_QUEUE0 + (4U * k), (uint32_t)in->queue[k]);\n        }}\n    }}");
    }
    for n in 0..words(m.conditions.len()) {
        let _ = writeln!(out, "    word = 0U;");
        for (k, c) in m.conditions.iter().enumerate().skip(32 * n).take(32) {
            let _ = writeln!(out, "    word |= in->{} ? (1UL << {}) : 0U;", c.ident, k - 32 * n);
        }
        let _ = writeln!(out, "    {b}_hw_write({up}_COND{n}, word);");
    }
    for (i, _) in m.program().inputs() {
        let field = &m.variables[i].ident;
        let name = format!("IN_{}", field.to_uppercase());
        let value = match fx.vars[i].kind {
            Kind::Fixed(f) if fx.vars[i].width < 32 => format!("(uint32_t)arc_fx_narrow(arc_fx_from((double)in->{field}, {f}), {})", fx.vars[i].width),
            Kind::Fixed(f) => format!("(uint32_t)arc_fx_from((double)in->{field}, {f})"),
            Kind::Int => format!("(uint32_t)in->{field}"),
            Kind::Bool => format!("in->{field} ? 1U : 0U"),
        };
        let _ = writeln!(out, "    {b}_hw_write({up}_{name}, {value});");
    }
    let _ = writeln!(out, "    {b}_hw_write({up}_CTRL, 1U);\n    n = 0UL;\n    while ((({b}_hw_read({up}_STATUS) & 2U) == 0U) && (n < {up}_HW_POLLS)) {{\n        n++;\n    }}\n    if (n >= {up}_HW_POLLS) {{\n        return -1;\n    }}");
    for n in 0..words(m.states.len()) {
        let _ = writeln!(out, "    word = {b}_hw_read({up}_STATE{n});");
        for (k, s) in m.states.iter().enumerate().skip(32 * n).take(32) {
            let _ = writeln!(out, "    out->{} = ((word >> {}) & 1U) != 0U;", s.ident, k - 32 * n);
        }
    }
    for a in &m.actions {
        let _ = writeln!(out, "    out->{} = (uint8_t)({b}_hw_read({up}_ACT_{}) & 0xFFU);", a.ident, a.ident.to_uppercase());
    }
    for (i, v) in m.program().variables.iter().enumerate() {
        let field = &m.variables[i].ident;
        let read = format!("{b}_hw_read({up}_VAR_{})", field.to_uppercase());
        let value = match fx.vars[i].kind {
            Kind::Fixed(f) => format!("({})arc_fx_to((int32_t){read}, {f})", v.ty.c()),
            Kind::Int => format!("(int32_t){read}"),
            Kind::Bool => format!("({read} & 1U) != 0U"),
        };
        let _ = writeln!(out, "    out->{field} = {value};");
    }
    for n in 0..words(m.budgets.len()) {
        let _ = writeln!(out, "    word = {b}_hw_read({up}_BUDGET{n});");
        for (k, bud) in m.budgets.iter().enumerate().skip(32 * n).take(32) {
            let _ = writeln!(out, "    out->{} = ((word >> {}) & 1U) != 0U;", bud.ident, k - 32 * n);
        }
    }
    let _ = writeln!(out, "    (void)word;\n    return 0;\n}}\n\n#endif /* ARCLANG_{up}_HW_H */");
    out
}
