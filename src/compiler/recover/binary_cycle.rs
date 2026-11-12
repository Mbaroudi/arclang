//! A binary whose `loop` is no function: the cycle of its `main`.
//!
//! The Arduino IDE builds with link-time optimisation: `setup`, `loop` and
//! most of the Arduino functions are folded into `main`. There is then no
//! function to name and none to stand in for. A correspondence says where
//! the cycle `main` never leaves starts (`functions: ["cycle at 0x0390"]`),
//! and the whole of the machine code is run on a model of the chip at the
//! level of its registers:
//!
//! - **the start** is the code from its reset to its first arrival at the
//!   head of the cycle; **one turn** is from one arrival to the next;
//! - **a pin is a bit of a register**: `digitalRead(2) is button` is bit 2
//!   of the input register of port D, `digitalWrite(13) is led` bit 5 of
//!   the output register of port B. A pin nobody names reads as anything,
//!   and may not be written; the pins written, and no other, are outputs;
//! - **the time** is what the interrupt of timer 0 keeps (proven of it
//!   first): the count of milliseconds `millis` returns, the turns and the
//!   counter `micros` reads. They are set before each turn, and time passes
//!   within a turn as the code reads it: 4 microseconds (one count of the
//!   timer) each time it has read the timer, so that a wait that reads the
//!   time until enough has passed ends, having taken that long;
//! - **the registers the cycle does not write** hold, at its head, what the
//!   start left in them (an optimised cycle keeps its constants there):
//!   they are those of the start in every turn, and that the turn leaves
//!   them so is part of what is proven.
//!
//! The chip is an ATmega328P wired as an Arduino UNO.

use super::correspondence::Correspondence;
use regex::Regex;
use std::collections::BTreeSet;

/// The cycle of a translation: where it is cut, the function that holds
/// it, and the registers r0..r31 that hold at the cut what the start left.
#[derive(Debug, Clone, PartialEq)]
pub struct Cut {
    pub at: u32,
    pub function: u32,
    pub kept: Vec<u8>,
}

/// What a function may change without putting it back: r0, r18 to r27, r30, r31.
const CLOBBERED: [u8; 13] = [0, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 30, 31];

/// Read what the translation says of the cycle it was cut at.
pub fn cut(lifted: &str, what: &str) -> Result<Cut, String> {
    let form = Regex::new(r"(?m)^//@ cut 0x([0-9a-f]+) in 0x([0-9a-f]+) to 0x[0-9a-f]+ calls=([01]) writes((?: \d+)*)$").expect("a pattern of the cycle");
    let said = form.captures(lifted).ok_or_else(|| format!("{what}: the translation does not say where the cycle is cut"))?;
    let address = |text: &str| u32::from_str_radix(text, 16).map_err(|e| format!("{what}: {e}"));
    let written: BTreeSet<u8> = said[4].split_whitespace().filter_map(|register| register.parse().ok()).collect();
    let calls = &said[3] == "1";
    let kept = (0u8..32).filter(|register| !written.contains(register) && !(calls && CLOBBERED.contains(register))).collect();
    Ok(Cut { at: address(&said[1])?, function: address(&said[2])?, kept })
}

/// The pins a correspondence names, by number: (those read, those written).
/// With a cycle a pin is a bit of a register: it is a number from 0 to 19,
/// read or written as a level.
pub fn pins(correspondence: &Correspondence) -> Result<(Vec<u8>, Vec<u8>), String> {
    let what = format!("correspondence '{}'", correspondence.name);
    let pin = |text: &str| text.trim().parse::<u8>().ok().filter(|pin| *pin < 20).ok_or_else(|| format!("{what}: with a cycle, a pin is its number on the board, 0 to 19 (it is \"{text}\")"));
    if let Some((kind, _, _)) = correspondence.reads.iter().find(|(kind, _, _)| kind != "digitalRead") {
        return Err(format!("{what}: with a cycle, {kind} is not read yet (only the levels of the pins)"));
    }
    if !correspondence.pwm.is_empty() {
        return Err(format!("{what}: with a cycle, analogWrite is not read yet (only the levels of the pins)"));
    }
    let reads: Vec<u8> = correspondence.reads.iter().map(|(_, text, _)| pin(text)).collect::<Result<_, _>>()?;
    let writes: Vec<u8> = correspondence.writes.iter().map(|(text, _)| pin(text)).collect::<Result<_, _>>()?;
    Ok((reads, writes))
}

/// The ports of an ATmega328P as an Arduino UNO wires them: (the input
/// register, the first pin, how many pins). The direction register is one
/// above the input register, the output register two above.
const PORTS: [(u32, u8, u8); 3] = [(0x23, 8, 6), (0x26, 14, 6), (0x29, 0, 8)];

/// The C the proof takes for the sketch: the translation, the chip, and
/// `setup()` (the start) and `loop()` (one turn of the cycle).
/// `count_at` and `turns_at`: where the interrupt of timer 0 keeps the
/// milliseconds and its turns, when the binary keeps the time.
pub fn sketch(lifted: &str, cut: &Cut, reads: &[u8], writes: &[u8], time: Option<(u32, u32)>, scalars: &[String], what: &str) -> Result<String, String> {
    super::binary::expected(lifted, what)?;
    let start = lifted.lines().find_map(|line| line.strip_prefix("//@ start 0x")).and_then(|at| u32::from_str_radix(at, 16).ok()).ok_or_else(|| format!("{what}: the translation does not say where the code starts"))?;
    let on = |first: u8, count: u8, pins: &[u8]| -> Vec<(u8, u8)> { pins.iter().copied().filter(|pin| *pin >= first && *pin < first + count).map(|pin| (pin, pin - first)).collect() };
    let mut c = String::from("/* The machine code of the binary on a model of its chip, as the proof reads it (written by `arclang prove`). */\n#define ARCLANG_CHIP\n\n/* ---- the machine code, one instruction at a time ---- */\n");
    c.push_str(lifted);
    c.push_str("\n/* ---- the chip: its pins, its time ---- */\n_Bool nondet_bool(void);\nstatic bool arclang_lost;          /* the code did not reach the head of its cycle */\nstatic unsigned long arclang_waited; /* the microseconds passed in this turn */\nstatic unsigned long arclang_last_read; /* the time, from the start of the turn, the last read of the timer returned */\n#ifndef ARCLANG_TICK_US\n#define ARCLANG_TICK_US 4UL /* what passes each time the code has read the timer: one count of it */\n#endif\nstatic uint8_t arclang_kept[32];   /* the registers at the head of the cycle, as the start left them */\nstatic unsigned arclang_kept_sp;\n");
    // The time: what the interrupt of timer 0 keeps, as it would be now.
    c.push_str("static unsigned long arclang_now_us(void) { return arclang_ms * 1000UL + arclang_waited; }\nstatic void arclang_set_time(void)\n{\n");
    if let Some((count_at, turns_at)) = time {
        c.push_str(&format!("    unsigned long count = millis() + arclang_waited / 1000UL;\n    unsigned long turns = (arclang_now_us() / 4UL) >> 8;\n"));
        for k in 0..4u32 {
            c.push_str(&format!("    arclang_mem[0x{:x}u] = (uint8_t)(count >> {});\n    arclang_mem[0x{:x}u] = (uint8_t)(turns >> {});\n", count_at + k, 8 * k, turns_at + k, 8 * k));
        }
    }
    c.push_str("}\n");
    // A register of the chip read.
    c.push_str("static uint8_t arclang_chip_rd(unsigned at)\n{\n    switch (at) {\n");
    for (input, first, count) in PORTS {
        let named: Vec<(u8, u8)> = on(first, count, &[reads, writes].concat());
        // The input register: the level of each pin named; a pin nobody names reads as anything.
        c.push_str(&format!("    case 0x{input:x}u: {{\n        uint8_t level = nondet_uchar();\n"));
        for (pin, bit) in &named {
            c.push_str(&format!("        level = (uint8_t)((level & ~(1u << {bit})) | (digitalRead({pin}) ? 1u << {bit} : 0u));\n"));
        }
        c.push_str("        return level;\n    }\n");
        // The output register: a pin written reads back as it was written.
        c.push_str(&format!("    case 0x{:x}u: {{\n        uint8_t level = arclang_io[at];\n", input + 2));
        for (pin, bit) in on(first, count, writes) {
            c.push_str(&format!("        level = (uint8_t)((level & ~(1u << {bit})) | (digitalRead({pin}) ? 1u << {bit} : 0u));\n"));
        }
        c.push_str("        return level;\n    }\n");
    }
    // The counter of timer 0 is the time; once its pending overflow has been asked for (what a read of the time ends with), 4 us have passed.
    c.push_str("    case 0x46u: arclang_last_read = arclang_waited; return (uint8_t)(arclang_now_us() / 4UL);\n    case 0x35u: arclang_waited += ARCLANG_TICK_US; arclang_set_time(); return 0;\n    default: return arclang_io[at];\n    }\n}\n");
    // A register of the chip written.
    c.push_str("static void arclang_chip_wr(unsigned at, uint8_t value)\n{\n    uint8_t before = arclang_io[at];\n    arclang_io[at] = value;\n    switch (at) {\n");
    for (input, first, count) in PORTS {
        let written = on(first, count, writes);
        let mask: u32 = written.iter().map(|(_, bit)| 1u32 << bit).sum();
        c.push_str(&format!("    case 0x{:x}u:\n", input + 2));
        for (pin, bit) in &written {
            c.push_str(&format!("        digitalWrite({pin}, (value >> {bit}) & 1u);\n"));
        }
        // After the start, the level of a pin nobody names does not change (a pull-up set by the start is the start's).
        c.push_str(&format!("        if (!arclang_booting && ((value ^ before) & ~0x{mask:02x}u) != 0) {{ arclang_unnamed = true; }}\n        return;\n"));
    }
    c.push_str("    default:\n        /* After its start the code writes no other register of the chip (a write that changes nothing apart). */\n        if (!arclang_booting && value != before) { arclang_wrote_register = true; }\n        return;\n    }\n}\n");
    // The directions: the pins written are outputs, and no other pin is.
    let direction = |(input, first, count): (u32, u8, u8)| format!("arclang_io[0x{:x}u] == 0x{:02x}u", input + 1, on(first, count, writes).iter().map(|(_, bit)| 1u32 << bit).sum::<u32>());
    c.push_str(&format!("static bool arclang_directions(void) {{ return {}; }}\n", PORTS.map(direction).join(" && ")));
    let kept: Vec<String> = cut.kept.iter().map(|register| format!("arclang_r[{register}] == arclang_kept[{register}]")).collect();
    c.push_str(&format!("static bool arclang_kept_same(void) {{ return arclang_reg_rd(ARCLANG_SP, 2) == arclang_kept_sp{}; }}\n", kept.iter().map(|one| format!(" && {one}")).collect::<String>()));
    let kept_all: String = (0..32).map(|register| format!("    arclang_kept[{register}] = arclang_r[{register}];\n")).collect();
    c.push_str(&format!(
        "\n/* ---- setup() is the code from its reset to the head of its cycle; loop() is one turn of the cycle ---- */\nvoid setup(void)\n{{\n    arclang_booting = true;\n    arclang_stop = false;\n    arclang_resume = false;\n    f_{start:04x}();\n    arclang_lost = !arclang_stop;\n    arclang_booting = false;\n    arclang_stop = false;\n{kept_all}    arclang_kept_sp = (unsigned)arclang_reg_rd(ARCLANG_SP, 2);\n}}\nvoid loop(void)\n{{\n    arclang_waited = 0UL;\n    arclang_last_read = 0UL;\n    arclang_set_time();\n    arclang_stop = false;\n    arclang_resume = true;\n    f_{:04x}();\n    if (!arclang_stop) {{ arclang_lost = true; }}\n    arclang_stop = false;\n    /* A wait ends on the read that returns the time it waits for: that is how long the turn took. */\n    arclang_delayed = arclang_last_read / 1000UL;\n}}\n",
        cut.function
    ));
    // A turn from anywhere: the registers the cycle keeps are those the start leaves (run from any state,
    // which the real start is one of); everything else is anything.
    let kept_set: BTreeSet<u8> = cut.kept.iter().copied().collect();
    c.push_str("static void arclang_from_any_turn(void)\n{\n    setup();\n    __CPROVER_havoc_object(arclang_mem);\n    __CPROVER_havoc_object(arclang_io);\n");
    for register in (0u8..32).filter(|register| !kept_set.contains(register)) {
        c.push_str(&format!("    arclang_r[{register}] = nondet_uchar();\n"));
    }
    for flag in 0x80u32..=0x88 {
        c.push_str(&format!("    arclang_reg[0x{flag:x}u] = nondet_uchar() & 1u;\n"));
    }
    c.push_str("    arclang_uncovered = nondet_bool();\n    arclang_outside = nondet_bool();\n    arclang_wrote_register = nondet_bool();\n    arclang_lost = nondet_bool();\n");
    for scalar in scalars {
        c.push_str(&format!("    {scalar} = nondet_bool();\n"));
    }
    c.push_str("}\n");
    Ok(c)
}

/// How many times each loop of the code runs in the start and its first
/// turn, as the prover names the loops: the bounds of a turn from anywhere.
///
/// In the start the time is known, and a wait that reads it ends when it
/// has read enough. In a turn from anywhere the time is any, and the
/// prover, which unwinds a loop without asking whether it can still run,
/// would unwind every wait to the common bound, each time it is entered.
/// It is given what the start counted instead; a loop that would run more
/// in some turn fails the proof as beyond its bound, it is not passed over.
pub fn bounds(dir: &std::path::Path, sources: &[&str], flags: &[&str], unwind: usize) -> Result<String, String> {
    let cbmc = std::env::var("ARCLANG_CBMC").ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| "cbmc".to_string());
    let asked = std::process::Command::new(&cbmc)
        .args(sources)
        .args(flags)
        .args(["--no-standard-checks", "--unwind", &unwind.to_string(), "--verbosity", "9", "--smt2", "--outfile", "bounds.smt2"])
        .current_dir(dir)
        .output()
        .map_err(|e| format!("cannot run '{cbmc}': {e}"))?;
    let _ = std::fs::remove_file(dir.join("bounds.smt2"));
    let said = String::from_utf8_lossy(&asked.stdout);
    // The loops of the machine code and of the two functions that run it: `f_0272.0`, `setup.0`.
    let turn = Regex::new(r"(?m)^Unwinding loop ((?:f_[0-9a-f]{4}|setup|loop)\.\d+) iteration (\d+) ").expect("a pattern of the cycle");
    let mut most: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for found in turn.captures_iter(&said) {
        let count: usize = found[2].parse().map_err(|_| "a count of turns is not read".to_string())?;
        let known = most.entry(found[1].to_string()).or_default();
        *known = (*known).max(count);
    }
    Ok(most.iter().map(|(name, count)| format!("{name}:{}", count + 2)).collect::<Vec<_>>().join(","))
}

/// What a proof over a cycle holds besides the lines of its
/// correspondence: (the condition, as it is written).
pub fn lines() -> Vec<(String, String)> {
    vec![
        ("!arclang_uncovered".to_string(), "the machine code runs only instructions the translation covers (no computed jump)".to_string()),
        ("!arclang_outside".to_string(), "the machine code stays within the memory and the flash of the chip".to_string()),
        ("!arclang_wrote_register".to_string(), "after its start, the machine code writes no register of the chip but the pins the correspondence names".to_string()),
        ("(!arclang_lost && !arclang_booting && !arclang_stop && !arclang_resume)".to_string(), "the machine code reaches the head of its cycle from its reset, and comes back to it".to_string()),
        ("arclang_directions()".to_string(), "the pins the correspondence says are written are outputs, and no other pin is".to_string()),
        ("arclang_kept_same()".to_string(), "the stack and the registers the cycle does not write are, at its head, as the start left them".to_string()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_registers_a_cycle_keeps_are_those_it_does_not_write_and_no_function_may_change() {
        let lifted = "//@ start 0x0\n//@ cut 0x0390 in 0x0272 to 0x04de calls=1 writes 12 13 14 18 24\n";
        let read = cut(lifted, "c").unwrap();
        assert_eq!((read.at, read.function), (0x390, 0x272));
        assert_eq!(read.kept, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 15, 16, 17, 28, 29]);
        // Without a call, what a function may change is kept too.
        let read = cut("//@ cut 0x0010 in 0x0000 to 0x0020 calls=0 writes 24\n", "c").unwrap();
        assert!(read.kept.contains(&0) && read.kept.contains(&31) && !read.kept.contains(&24));
        assert!(cut("//@ start 0x0\n", "c").unwrap_err().contains("does not say where the cycle is cut"));
    }

    #[test]
    fn a_pin_is_a_bit_of_the_register_of_its_port() {
        let lifted = "//@ language avr8:LE:16:default\n//@ memory 0x100 0x8ff\n//@ start 0x0\n//@ register SP register 0x3d 2\n//@ register R24 mem 0x18 1\n//@ register Cflg register 0x80 1\n";
        let cut = Cut { at: 0x390, function: 0x272, kept: vec![1, 2] };
        let c = sketch(lifted, &cut, &[2], &[13], Some((0x10b, 0x10f)), &["arclang_unnamed".to_string()], "c").unwrap();
        // Pin 2 is bit 2 of the input register of port D; pin 13, bit 5 of the output register of port B.
        assert!(c.contains("case 0x29u: {\n        uint8_t level = nondet_uchar();\n        level = (uint8_t)((level & ~(1u << 2)) | (digitalRead(2) ? 1u << 2 : 0u));"), "{c}");
        assert!(c.contains("case 0x25u:\n        digitalWrite(13, (value >> 5) & 1u);\n        if (!arclang_booting && ((value ^ before) & ~0x20u) != 0) { arclang_unnamed = true; }"), "{c}");
        // Pin 13 is an output and no other pin is; the time is where the interrupt keeps it.
        assert!(c.contains("arclang_io[0x24u] == 0x20u && arclang_io[0x27u] == 0x00u && arclang_io[0x2au] == 0x00u"), "{c}");
        assert!(c.contains("arclang_mem[0x10bu] = (uint8_t)(count >> 0);") && c.contains("arclang_mem[0x112u] = (uint8_t)(turns >> 24);") && c.contains("f_0272();"), "{c}");
        assert!(c.contains("arclang_r[1] == arclang_kept[1] && arclang_r[2] == arclang_kept[2]") && c.contains("arclang_r[3] = nondet_uchar();") && !c.contains("arclang_r[2] = nondet_uchar();"), "{c}");
    }
}
