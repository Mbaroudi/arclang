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
    /// What the start leaves at the cut, found by running it (see `binary::StartState`): the registers and
    /// the stack pointer a turn from any state begins with, for those the cycle does not write.
    pub start: Option<super::binary::StartState>,
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
    Ok(Cut { at: address(&said[1])?, function: address(&said[2])?, kept, start: None })
}

/// The chip a correspondence names: the pins read and written as levels, the pins written as PWM, and
/// the routine of the interrupt of the UART when the binary has one (the serial port).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Chip {
    pub reads: Vec<u8>,
    pub writes: Vec<u8>,
    /// In the order of the correspondence's `analogWrite` lines.
    pub pwm: Vec<u8>,
    pub uart: Option<u32>,
}

/// The pins of an Arduino UNO that a timer other than the clock's drives: (the pin, the output register
/// of its port, its bit, the control register of its timer, the bit of that register that connects the
/// pin to the timer, the compare register of the pin, the timer). Pins 5 and 6 are driven by timer 0,
/// which keeps the time: they are not read yet.
const PWM: [(u8, u32, u8, u32, u32, u32, u8); 4] = [
    (3, 0x2b, 3, 0xb0, 0x20, 0xb4, 2),
    (9, 0x25, 1, 0x80, 0x80, 0x88, 1),
    (10, 0x25, 2, 0x80, 0x20, 0x8a, 1),
    (11, 0x25, 3, 0xb0, 0x80, 0xb3, 2),
];

/// The pins a correspondence names, by number. With a cycle a pin is a bit of a register: it is a number
/// from 0 to 19, read or written as a level, or written as a PWM (`analogWrite`, pins 3, 9, 10, 11).
pub fn pins(correspondence: &Correspondence) -> Result<Chip, String> {
    let what = format!("correspondence '{}'", correspondence.name);
    let pin = |text: &str| text.trim().parse::<u8>().ok().filter(|pin| *pin < 20).ok_or_else(|| format!("{what}: with a cycle, a pin is its number on the board, 0 to 19 (it is \"{text}\")"));
    if let Some((kind, _, _)) = correspondence.reads.iter().find(|(kind, _, _)| kind != "digitalRead") {
        return Err(format!("{what}: with a cycle, {kind} is not read yet (only the levels of the pins)"));
    }
    let reads: Vec<u8> = correspondence.reads.iter().map(|(_, text, _)| pin(text)).collect::<Result<_, _>>()?;
    let writes: Vec<u8> = correspondence.writes.iter().map(|(text, _)| pin(text)).collect::<Result<_, _>>()?;
    let pwm: Vec<u8> = correspondence.pwm.iter().map(|(text, _)| pin(text)).collect::<Result<_, _>>()?;
    for pin in &pwm {
        if !PWM.iter().any(|known| known.0 == *pin) {
            return Err(format!("{what}: with a cycle, analogWrite({pin}) is not read: the pins driven by a timer other than the clock's are 3, 9, 10 and 11 (5 and 6 are driven by the timer that keeps the time)"));
        }
        if writes.contains(pin) || pwm.iter().filter(|other| *other == pin).count() > 1 {
            return Err(format!("{what}: pin {pin} is written as a level and as a PWM, or twice: it is one or the other"));
        }
    }
    Ok(Chip { reads, writes, pwm, uart: None })
}

/// The first address of the memory of the chip (its RAM), which is where `StartState::memory` begins.
const RAM_FIRST: u32 = 0x100;

/// The bytes of the memory some instruction writes by their address (`sts`): the variables of the sketch.
fn written_by_address(lifted: &str) -> BTreeSet<u32> {
    let store = Regex::new(r"(?m)^L_[0-9a-f]{4}: /\* sts 0x([0-9a-f]+),\w+ \*/").expect("a pattern of the cycle");
    store.captures_iter(lifted).filter_map(|found| u32::from_str_radix(&found[1], 16).ok()).collect()
}

/// The high bytes of the pointers the code loads by their address (`lds` into `r27`, `r29` or `r31`): a pointer the
/// start leaves can be a small address, whose high byte is zero.
fn pointer_high_bytes(lifted: &str) -> BTreeSet<u32> {
    let load = Regex::new(r"(?m)^L_[0-9a-f]{4}: /\* lds [XYZ]hi,0x([0-9a-f]+) \*/").expect("a pattern of the cycle");
    load.captures_iter(lifted).filter_map(|found| u32::from_str_radix(&found[1], 16).ok()).collect()
}

/// The bytes of the memory a proof takes, at the head of every turn, as the start left them: those the start leaves
/// other than zero (or the high byte of a pointer the code loads by its address), below all the stack the start used (the
/// stack grows down: what the start pushed and popped stays in the memory under the stack pointer it ends with, and is no
/// object: the first call of a turn writes over it), that no instruction writes by its address and that are not the clock. They are
/// what the start sets up once and the code only reads after: the vtables it copies from the flash, the pointer to
/// its vtable a constructor writes in an object (the serial port), the addresses of the registers an object keeps.
/// A turn reaches its computed calls through them: taken as anything, every function would be a target. It is not
/// assumed that a turn leaves them: a line holds it, and a turn that writes one through a pointer refutes the proof.
pub fn constants(lifted: &str, state: &super::binary::StartState, time: Option<(u32, u32)>) -> Vec<(u32, u8)> {
    let written = written_by_address(lifted);
    let high = pointer_high_bytes(lifted);
    let clock: Vec<u32> = time.map(|(count, turns)| (0..4).flat_map(|k| [count + k, turns + k]).collect()).unwrap_or_default();
    // A push writes where the stack pointer is, then lowers it: the lowest byte the start's stack wrote is one above the
    // lowest the pointer went. When the run did not say how low it went, nothing is taken as set up once: a byte of the
    // stack taken for an object would refute a proof that holds.
    if state.low_sp == 0 {
        return Vec::new();
    }
    let stack_floor = state.low_sp.min(state.sp);
    let free = |at: u32| at <= u32::from(stack_floor) && !written.contains(&at) && !clock.contains(&at);
    let found: Vec<(u32, u8)> = state
        .memory
        .iter()
        .enumerate()
        .map(|(offset, value)| (RAM_FIRST + offset as u32, *value))
        .filter(|(at, value)| (*value != 0 || high.contains(at)) && free(*at))
        .collect();
    // A single zero between two of them is set up once as they are: the end of a text the code prints (taken as
    // anything, the code that measures the text would never be known to stop). That a turn writes none is a line.
    let known: BTreeSet<u32> = found.iter().map(|(at, _)| *at).collect();
    let ends: Vec<(u32, u8)> = state
        .memory
        .iter()
        .enumerate()
        .map(|(offset, value)| (RAM_FIRST + offset as u32, *value))
        .filter(|(at, value)| *value == 0 && !known.contains(at) && free(*at) && *at > RAM_FIRST && known.contains(&(at - 1)) && known.contains(&(at + 1)))
        .collect();
    let mut all = [found, ends].concat();
    all.sort();
    all
}

/// Where `write()` of the serial port is (`HardwareSerial::write` of the AVR core): the first function of the vtable
/// of the object that keeps the addresses of the registers of the UART (see `serial_indexes`).
pub fn serial_write(constants: &[(u32, u8)]) -> Option<u32> {
    let value = |at: u32| constants.iter().find(|(known, _)| *known == at).map(|(_, value)| u32::from(*value));
    let word = |at: u32| value(at).map(|low| low | value(at + 1).unwrap_or(0) << 8);
    // The object: its vtable, the error of Print, the timeout of Stream (twelve bytes), then the registers.
    let object = serial_indexes(constants).first()?.checked_sub(13 + 12)?;
    word(word(object)?).map(|at| at * 2)
}

/// A turn does not wait in `write()` of the serial port: where a loop of it begins, the turn is refused and ends
/// there. Its transmit buffer is empty at the head of a turn and the chip takes each byte at once, so the real
/// code never reaches its wait for a full buffer. The prover does not know it before it has run every path: run,
/// the wait and the interrupt it calls would be unrolled in each `write()`; refused and joined to the path that
/// goes on, it would leave the registers that path keeps (the object, the text) as anything, and the next read
/// through them would be a read of any register of the chip. Ended there, no path joins: a proof that holds says
/// no turn waits.
fn refuse_wait(lifted: &str, function: u32) -> String {
    let head = format!("void f_{function:04x}(void)");
    let label = Regex::new(r"^(L_[0-9a-f]{4}): /\*").expect("a pattern of the cycle");
    let jump = Regex::new(r"goto (L_[0-9a-f]{4});").expect("a pattern of the cycle");
    let lines: Vec<&str> = lifted.lines().collect();
    let Some(first) = lines.iter().position(|line| *line == head) else { return lifted.to_string() };
    let last = lines[first..].iter().position(|line| *line == "}").map_or(lines.len(), |length| first + length);
    // The labels a jump placed after them goes back to.
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut heads: BTreeSet<String> = BTreeSet::new();
    for line in &lines[first..last] {
        if let Some(found) = label.captures(line) {
            seen.insert(found.get(1).map_or("", |m| m.as_str()));
        }
        heads.extend(jump.captures_iter(line).filter(|to| seen.contains(&to[1])).map(|to| to[1].to_string()));
    }
    lines
        .iter()
        .enumerate()
        .map(|(rank, line)| match label.captures(line) {
            Some(found) if rank > first && rank < last && heads.contains(&found[1]) => {
                format!("{line}\n    if (!arclang_booting) {{ __CPROVER_assert(0, \"a turn does not wait in write() of the serial port\"); __CPROVER_assume(0); }}")
            }
            _ => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// The functions from which the function of the cycle is reached by direct calls and jumps (the reset, the
/// function that calls main(), main() itself): a turn never calls one of them again.
fn ancestors(lifted: &str, function: u32) -> BTreeSet<u32> {
    let head = Regex::new(r"^void f_([0-9a-f]{4})\(void\)$").expect("a pattern of the cycle");
    let called = Regex::new(r"\b[cf]_([0-9a-f]{4})\(\)").expect("a pattern of the cycle");
    let mut calls: Vec<(u32, u32)> = Vec::new();
    let mut current: Option<u32> = None;
    for line in lifted.lines() {
        if let Some(found) = head.captures(line) {
            current = u32::from_str_radix(&found[1], 16).ok();
        } else if line == "}" {
            current = None;
        } else if let Some(from) = current {
            calls.extend(called.captures_iter(line).filter_map(|to| u32::from_str_radix(&to[1], 16).ok()).map(|to| (from, to)));
        }
    }
    let mut found: BTreeSet<u32> = BTreeSet::from([function]);
    loop {
        let more: Vec<u32> = calls.iter().filter(|(from, to)| found.contains(to) && !found.contains(from)).map(|(from, _)| *from).collect();
        if more.is_empty() {
            return found;
        }
        found.extend(more);
    }
}

/// The words (two bytes, low first) the start leaves in the memory at the bytes taken as constants: among them, the
/// addresses (in words, as a computed call takes them) of the functions its vtables name.
fn words(constants: &[(u32, u8)]) -> BTreeSet<u32> {
    let value = |at: u32| constants.iter().find(|(known, _)| *known == at).map(|(_, value)| u32::from(*value));
    constants.iter().filter_map(|(at, low)| value(at + 1).or(Some(0)).map(|high| u32::from(*low) | high << 8)).collect()
}

/// The computed calls of a turn that would reach the function of the cycle again, or one of the functions it is
/// reached from (the reset first of all), mark the turn as running what the translation does not cover and end it:
/// a proof that holds says there is none. Run instead, they would make the prover run the whole start again in every
/// turn for any address a computed call could take that it cannot rule out at once (its memory runs out).
///
/// With `targets` (the words the start leaves in the memory, see `words`), a computed call of a turn to any other
/// function is not run either: a C++ object calls through its vtable, which the start sets up; taken as anything,
/// the prover would try every function of the binary at every such call, one inside another.
fn guard_reentry(lifted: &str, function: u32, targets: Option<&BTreeSet<u32>>) -> String {
    let ancestors = ancestors(lifted, function);
    let case = Regex::new(r"^    case 0x([0-9a-f]+)u: c_([0-9a-f]{4})\(\); return;$").expect("a pattern of the cycle");
    lifted
        .lines()
        .map(|line| match case.captures(line) {
            Some(found)
                if u32::from_str_radix(&found[2], 16).is_ok_and(|to| ancestors.contains(&to))
                    || targets.is_some_and(|targets| u32::from_str_radix(&found[1], 16).is_ok_and(|word| !targets.contains(&word))) =>
            {
                format!("    case 0x{}u: if (!arclang_booting) {{ arclang_uncovered = true; arclang_stop = true; return; }} c_{}(); return;", &found[1], &found[2])
            }
            _ => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// The bytes of the indexes of the serial port's two buffers (receive head and tail, transmit head and tail), found
/// by the six addresses of the registers of the UART that the start leaves in the object (`HardwareSerial` of the AVR
/// core: UBRR0H, UBRR0L, UCSR0A, UCSR0B, UCSR0C, UDR0, then `_written`, then the four indexes).
pub fn serial_indexes(constants: &[(u32, u8)]) -> Vec<u32> {
    const REGISTERS: [u8; 6] = [0xc5, 0xc4, 0xc0, 0xc1, 0xc2, 0xc6];
    let value = |at: u32| constants.iter().find(|(known, _)| *known == at).map(|(_, value)| *value);
    let pointer = |at: u32, register: u8| value(at) == Some(register) && value(at + 1).unwrap_or(0) == 0;
    constants
        .iter()
        .map(|(at, _)| *at)
        .find(|at| REGISTERS.iter().enumerate().all(|(k, register)| pointer(at + 2 * k as u32, *register)))
        .map(|at| (at + 13..=at + 16).collect())
        .unwrap_or_default()
}

/// How many times the interrupt of the UART runs to empty a transmit buffer of 64 bytes, from any indexes.
pub const DRAIN_BOUND: usize = 66;

/// The ports of an ATmega328P as an Arduino UNO wires them: (the input
/// register, the first pin, how many pins). The direction register is one
/// above the input register, the output register two above.
const PORTS: [(u32, u8, u8); 3] = [(0x23, 8, 6), (0x26, 14, 6), (0x29, 0, 8)];

/// The C the proof takes for the sketch: the translation, the chip, and
/// `setup()` (the start) and `loop()` (one turn of the cycle).
/// `count_at` and `turns_at`: where the interrupt of timer 0 keeps the
/// milliseconds and its turns, when the binary keeps the time.
pub fn sketch(lifted: &str, cut: &Cut, chip: &Chip, time: Option<(u32, u32)>, scalars: &[String], what: &str) -> Result<String, String> {
    let (reads, writes) = (chip.reads.as_slice(), chip.writes.as_slice());
    // The pins of the PWM: bits of the port that the code sets as it likes (the timer, or the level), not stood in for.
    let pwm_pins: Vec<(u8, u32, u8, u32, u32, u32, u8)> = chip.pwm.iter().filter_map(|pin| PWM.iter().find(|known| known.0 == *pin).copied()).collect();
    let timers: BTreeSet<u8> = pwm_pins.iter().map(|known| known.6).collect();
    super::binary::expected(lifted, what)?;
    let start = lifted.lines().find_map(|line| line.strip_prefix("//@ start 0x")).and_then(|at| u32::from_str_radix(at, 16).ok()).ok_or_else(|| format!("{what}: the translation does not say where the code starts"))?;
    let on = |first: u8, count: u8, pins: &[u8]| -> Vec<(u8, u8)> { pins.iter().copied().filter(|pin| *pin >= first && *pin < first + count).map(|pin| (pin, pin - first)).collect() };
    let mut c = String::from("/* The machine code of the binary on a model of its chip, as the proof reads it (written by `arclang prove`). */\n#define ARCLANG_CHIP\n\n/* ---- the machine code, one instruction at a time ---- */\n");
    // The functions a turn may reach by a computed call: those whose address the start leaves in the memory (its vtables).
    let set_up: Option<Vec<(u32, u8)>> = cut.start.as_ref().filter(|state| state.reached && state.registers.len() == 32).map(|state| constants(lifted, state, time));
    let targets: Option<BTreeSet<u32>> = set_up.as_ref().map(|constants| words(constants));
    let guarded = guard_reentry(lifted, cut.function, targets.as_ref());
    // With a serial port run on the chip, a turn does not wait in its write().
    let write = if chip.uart.is_some() { set_up.as_ref().and_then(|constants| serial_write(constants)) } else { None };
    let machine_code = write.map_or(guarded.clone(), |at| refuse_wait(&guarded, at));
    c.push_str(&if std::env::var("ARCLANG_IN_LINE").is_ok() { super::binary_order::in_line(&machine_code) } else { machine_code });
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
    if chip.uart.is_some() {
        // The status of the UART: ready to send (the data register is empty), nothing received.
        c.push_str("    case 0xc0u: return 0x20u;\n");
    }
    c.push_str("    case 0x46u: arclang_last_read = arclang_waited; return (uint8_t)(arclang_now_us() / 4UL);\n    case 0x35u: arclang_waited += ARCLANG_TICK_US; arclang_set_time(); return 0;\n    default: return arclang_io[at];\n    }\n}\n");
    // A register of the chip written.
    c.push_str("static void arclang_chip_wr(unsigned at, uint8_t value)\n{\n    uint8_t before = arclang_io[at];\n    arclang_io[at] = value;\n    switch (at) {\n");
    for (input, first, count) in PORTS {
        let written = on(first, count, writes);
        let free: u32 = pwm_pins.iter().filter(|known| known.1 == input + 2).map(|known| 1u32 << known.2).sum();
        let mask: u32 = written.iter().map(|(_, bit)| 1u32 << bit).sum::<u32>() | free;
        c.push_str(&format!("    case 0x{:x}u:\n", input + 2));
        for (pin, bit) in &written {
            c.push_str(&format!("        digitalWrite({pin}, (value >> {bit}) & 1u);\n"));
        }
        // After the start, the level of a pin nobody names does not change (a pull-up set by the start is the start's).
        c.push_str(&format!("        if (!arclang_booting && ((value ^ before) & ~0x{mask:02x}u) != 0) {{ arclang_unnamed = true; }}\n        return;\n"));
    }
    // The registers of the timers that drive a PWM pin, and those of the UART: written by the code as it likes (what is proven of
    // them is in the lines: the mode of the timers, the value of the pin), never an unnamed register.
    let mut allowed: Vec<u32> = Vec::new();
    for timer in &timers {
        allowed.extend(if *timer == 1 { vec![0x80u32, 0x88, 0x89, 0x8a, 0x8b] } else { vec![0xb0u32, 0xb3, 0xb4] });
    }
    if chip.uart.is_some() {
        allowed.extend([0xc0u32, 0xc1, 0xc2, 0xc4, 0xc5, 0xc6]);
    }
    for at in &allowed {
        c.push_str(&format!("    case 0x{at:x}u:\n"));
    }
    if !allowed.is_empty() {
        c.push_str("        return;\n");
    }
    c.push_str("    default:\n        /* After its start the code writes no other register of the chip (a write that changes nothing apart). */\n        if (!arclang_booting && value != before) { arclang_wrote_register = true; }\n        return;\n    }\n}\n");
    // The directions: the pins written are outputs, and no other pin is.
    let direction = |(input, first, count): (u32, u8, u8)| {
        let pwm: u32 = pwm_pins.iter().filter(|known| known.1 == input + 2).map(|known| 1u32 << known.2).sum();
        format!("arclang_io[0x{:x}u] == 0x{:02x}u", input + 1, on(first, count, writes).iter().map(|(_, bit)| 1u32 << bit).sum::<u32>() | pwm)
    };
    c.push_str(&format!("static bool arclang_directions(void) {{ return {}; }}\n", PORTS.map(direction).join(" && ")));
    // The value an analogWrite leaves on a PWM pin: the compare register when its timer drives the pin, else the level of the pin.
    c.push_str("static int arclang_pwm_value(unsigned k)\n{\n    switch (k) {\n");
    for (rank, known) in pwm_pins.iter().enumerate() {
        let (_, port, bit, control, connect, compare, _) = *known;
        c.push_str(&format!("    case {rank}u: return (arclang_io[0x{control:x}u] & 0x{connect:02x}u) != 0 ? (int)arclang_io[0x{compare:x}u] : (((arclang_io[0x{port:x}u] >> {bit}) & 1u) != 0 ? 255 : 0);\n"));
    }
    c.push_str("    default: return 0;\n    }\n}\n");
    let set_duties: String = (0..pwm_pins.len()).map(|rank| format!("    arclang_duty_{rank} = arclang_pwm_value({rank}u);\n")).collect();
    let assume_duties: String = (0..pwm_pins.len()).map(|rank| format!("    __CPROVER_assume(arclang_pwm_value({rank}u) == arclang_duty_{rank});\n")).collect();
    // The interrupt of the UART, run the way the chip runs it: until it has emptied the transmit buffer.
    let drain = chip.uart.map(|routine| format!("static void arclang_drain(void)\n{{\n    unsigned sp = (unsigned)arclang_reg_rd(ARCLANG_SP, 2);\n    arclang_io[0xc1u] = (uint8_t)(arclang_io[0xc1u] | 0x20u);\n    for (unsigned k = 0; k < {DRAIN_BOUND}u && (arclang_io[0xc1u] & 0x20u) != 0u; k++) {{\n        arclang_reg_wr(ARCLANG_SP, 2, sp - 2u);\n        f_{routine:04x}();\n    }}\n    __CPROVER_assert((arclang_io[0xc1u] & 0x20u) == 0u && arclang_reg_rd(ARCLANG_SP, 2) == sp && !arclang_uncovered && !arclang_outside && !arclang_wrote_register && !arclang_wrote_constant, \"the interrupt of the UART empties the transmit buffer, covered, and returns\");\n}}\n")).unwrap_or_default();
    c.push_str(&drain);
    let kept: Vec<String> = cut.kept.iter().map(|register| format!("arclang_r[{register}] == arclang_kept[{register}]")).collect();
    c.push_str(&format!("static bool arclang_kept_same(void) {{ return arclang_reg_rd(ARCLANG_SP, 2) == arclang_kept_sp{}; }}\n", kept.iter().map(|one| format!(" && {one}")).collect::<String>()));
    let drain_call = if chip.uart.is_some() { "    arclang_drain(); /* its interrupt has emptied the transmit buffer before the first turn */\n" } else { "" };
    let kept_all: String = (0..32).map(|register| format!("    arclang_kept[{register}] = arclang_r[{register}];\n")).collect();
    c.push_str(&format!(
        "\n/* ---- setup() is the code from its reset to the head of its cycle; loop() is one turn of the cycle ---- */\nvoid setup(void)\n{{\n    arclang_booting = true;\n    arclang_stop = false;\n    arclang_resume = false;\n    f_{start:04x}();\n    arclang_lost = !arclang_stop;\n    arclang_booting = false;\n    arclang_stop = false;\n{drain_call}{kept_all}    arclang_kept_sp = (unsigned)arclang_reg_rd(ARCLANG_SP, 2);\n}}\nvoid loop(void)\n{{\n    arclang_waited = 0UL;\n    arclang_last_read = 0UL;\n    arclang_set_time();\n    arclang_stop = false;\n    arclang_resume = true;\n    f_{:04x}();\n    if (!arclang_stop) {{ arclang_lost = true; }}\n    arclang_stop = false;\n    /* A wait ends on the read that returns the time it waits for: that is how long the turn took. */\n    arclang_delayed = arclang_last_read / 1000UL;\n{set_duties}}}\n",
        cut.function
    ));
    // A turn from anywhere: the registers the cycle keeps are those the start leaves (run from any state,
    // which the real start is one of); everything else is anything.
    let kept_set: BTreeSet<u8> = cut.kept.iter().copied().collect();
    // The registers the cycle keeps are those the start leaves. They are not found by running the start again in
    // every turn (symbolically, on any memory: its loops are the cost): they are the constants found by running it
    // once, and the start proven (a line) leaves the registers it is taken to leave.
    let known = cut.start.as_ref().filter(|state| state.reached && state.registers.len() == 32);
    if let Some(state) = known {
        // Only the registers the cycle keeps: the others are anything at the head of a turn, whatever the start left in them.
        let same: String = (0..32usize).filter(|register| kept_set.contains(&(*register as u8))).map(|register| format!(" && arclang_kept[{register}] == {}u", state.registers[register])).collect();
        c.push_str(&format!("static bool arclang_kept_found(void) {{ return arclang_kept_sp == {}u{same}; }}\n", state.sp));
    }
    if known.is_none() {
        c.push_str("static bool arclang_kept_found(void) { return true; }\n");
    }
    // The bytes the start sets up once and the code only reads after (see `constants`).
    let constants: Vec<(u32, u8)> = known.map(|state| constants(lifted, state, time)).unwrap_or_default();
    // After the start they are read as constants, whatever the memory holds (see `arclang_constant` in lifted.h); the start
    // leaves them in the memory, and a turn writes none (both a line).
    let cases: String = constants.iter().map(|(at, value)| format!("    case 0x{at:04x}u: return {value};\n")).collect();
    c.push_str(&format!("static int arclang_constant(uint64_t at)\n{{\n    switch (at) {{\n{cases}    default: return -1;\n    }}\n}}\n"));
    // The settings of the timers no pin named is driven by (their control registers, the interrupts they raise): only the
    // code writes them, and a turn changes none (a line: it writes no register of the chip). They are, at the head of a
    // turn, as the start leaves them (the same line holds it of the start). Taken as anything, a write that changes
    // nothing on the chip (digitalRead of a pin a timer could drive disconnects that timer) would be a write refused.
    let settings: Vec<(u32, u8)> = known.map(|state| state.settings.iter().filter(|(at, _)| !allowed.contains(at)).copied().collect()).unwrap_or_default();
    c.push_str(&format!(
        "static bool arclang_kept_memory(void) {{ return !arclang_wrote_constant{}{}; }}\n",
        constants.iter().map(|(at, value)| format!(" && arclang_mem[0x{at:04x}u] == {value}u")).collect::<String>(),
        settings.iter().map(|(at, value)| format!(" && arclang_io[0x{at:02x}u] == {value}u")).collect::<String>()
    ));
    // The indexes of the serial port's buffers stay within their 64 bytes (a line): taken as anything, the interrupt
    // of the UART would never empty a buffer whose index the real code never reaches.
    let indexes = serial_indexes(&constants);
    c.push_str(&format!("static bool arclang_buffers(void) {{ return true{}; }}\n", indexes.iter().map(|at| format!(" && arclang_mem[0x{at:04x}u] < 64u")).collect::<String>()));
    let set_constants: String = constants.iter().map(|(at, value)| format!("    arclang_mem[0x{at:04x}u] = {value}u;\n")).collect();
    let begin = match known {
        Some(state) => {
            let regs: String = (0..32).map(|register| format!("    arclang_kept[{register}] = {};\n", state.registers[register])).collect();
            let set: String = (0..32).map(|register| format!("    arclang_r[{register}] = {};\n", state.registers[register])).collect();
            format!("    arclang_booting = false;\n    arclang_stop = false;\n    arclang_resume = false;\n{regs}    arclang_kept_sp = {}u;\n    arclang_reg_wr(ARCLANG_SP, 2, {}u);\n{set}", state.sp, state.sp)
        }
        None => "    setup();\n".to_string(),
    };
    c.push_str(&format!("static void arclang_from_any_turn(void)\n{{\n{begin}    __CPROVER_havoc_object(arclang_mem);\n    __CPROVER_havoc_object(arclang_io);\n{set_constants}{}", settings.iter().map(|(at, value)| format!("    arclang_io[0x{at:02x}u] = {value}u;\n")).collect::<String>()));
    for register in (0u8..32).filter(|register| !kept_set.contains(register)) {
        c.push_str(&format!("    arclang_r[{register}] = nondet_uchar();\n"));
    }
    for flag in 0x80u32..=0x88 {
        c.push_str(&format!("    arclang_reg[0x{flag:x}u] = nondet_uchar() & 1u;\n"));
    }
    if chip.uart.is_some() {
        // The interrupt of the UART has emptied the transmit buffer by the head of the turn: the state is that of a drained buffer.
        // The interrupt of the UART is run from any state in an obligation of its own (`drain`): it empties the transmit
        // buffer, covered, returns, and writes no constant. A turn starts from a state it leaves (its transmit buffer
        // empty, its interrupt off) without running it again: run in every turn, its 66 runs on any memory make the
        // question too large for the prover. Without the indexes of the buffer, it is run in the turn as before.
        let drained = match indexes.as_slice() {
            [_, _, head, tail] => format!("#ifdef ARCLANG_DRAIN\n    __CPROVER_assume(arclang_buffers());\n    arclang_drain();\n    return;\n#else\n    __CPROVER_assume(arclang_buffers() && arclang_mem[0x{head:04x}u] == arclang_mem[0x{tail:04x}u] && (arclang_io[0xc1u] & 0x20u) == 0u);\n#endif\n"),
            _ => "    __CPROVER_assume(arclang_buffers());\n    arclang_drain();\n".to_string(),
        };
        c.push_str(&format!("    arclang_uncovered = false;\n    arclang_outside = false;\n    arclang_wrote_register = false;\n    arclang_wrote_constant = false;\n{drained}"));
    }
    c.push_str("    arclang_uncovered = nondet_bool();\n    arclang_outside = nondet_bool();\n    arclang_wrote_register = nondet_bool();\n    arclang_lost = nondet_bool();\n");
    for scalar in scalars {
        c.push_str(&format!("    {scalar} = nondet_bool();\n"));
    }
    // The value a PWM pin has is what the registers of its timer give: the state starts from the value the model says.
    c.push_str(&assume_duties);
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
pub fn lines(pwm: &[u8]) -> Vec<(String, String)> {
    let mut lines = vec![
        ("!arclang_uncovered".to_string(), "the machine code runs only instructions the translation covers (no computed jump)".to_string()),
        ("!arclang_outside".to_string(), "the machine code stays within the memory and the flash of the chip".to_string()),
        ("!arclang_wrote_register".to_string(), "after its start, the machine code writes no register of the chip but the pins the correspondence names".to_string()),
        ("(!arclang_lost && !arclang_booting && !arclang_stop && !arclang_resume)".to_string(), "the machine code reaches the head of its cycle from its reset, and comes back to it".to_string()),
        ("arclang_directions()".to_string(), "the pins the correspondence says are written are outputs, and no other pin is".to_string()),
        ("arclang_kept_same()".to_string(), "the stack and the registers the cycle does not write are, at its head, as the start left them".to_string()),
        ("arclang_kept_found()".to_string(), "the registers and the stack at the head of the cycle after the start are those the proof takes for every turn (found by running the start)".to_string()),
        ("arclang_kept_memory()".to_string(), "the bytes the start sets up once (other than zero, below its stack, never written by their address: vtables, objects) are, at the head of the cycle, as the start left them, and the code writes none of them after its start; the settings of the timers no pin named is driven by are as the start left them too".to_string()),
        ("arclang_buffers()".to_string(), "the indexes of the serial port's receive and transmit buffers are within their 64 bytes".to_string()),
    ];
    // A timer that drives a PWM pin counts in 8 bits, phase correct, and runs: the start leaves it so, and the cycle only
    // connects or disconnects the pin (the two bits of the control register), never changes how it counts.
    let timers: BTreeSet<u8> = PWM.iter().filter(|known| pwm.contains(&known.0)).map(|known| known.6).collect();
    for timer in timers {
        let (a, b) = if timer == 1 { (0x80, 0x81) } else { (0xb0, 0xb1) };
        let mode = if timer == 1 { format!("(arclang_io[0x{a:x}u] & 3u) == 1u && (arclang_io[0x{b:x}u] & 0x18u) == 0u && (arclang_io[0x{b:x}u] & 7u) != 0u") } else { format!("(arclang_io[0x{a:x}u] & 3u) == 1u && (arclang_io[0x{b:x}u] & 8u) == 0u && (arclang_io[0x{b:x}u] & 7u) != 0u") };
        lines.push((format!("({mode})"), format!("timer {timer} counts in 8 bits, phase correct, and runs: a value written to its compare register is a duty of 0 to 255")));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_computed_call_of_a_turn_that_would_reach_the_reset_or_main_again_is_not_run() {
        let lifted = "void f_0000(void)\n{\n    { c_00c4(); return; }\n}\nvoid f_00c4(void)\n{\n    c_0540();\n}\nvoid f_0540(void)\n{\n    c_01b4();\n}\nvoid f_01b4(void)\n{\n    arclang_ijmp(w);\n}\nstatic void arclang_ijmp(uint64_t word)\n{\n    switch (word) {\n    case 0x0u: c_0000(); return;\n    case 0x62u: c_00c4(); return;\n    case 0xdau: c_01b4(); return;\n    case 0x2a0u: c_0540(); return;\n    }\n}\n";
        assert_eq!(ancestors(lifted, 0x0540), BTreeSet::from([0x0000, 0x00c4, 0x0540]));
        let guarded = guard_reentry(lifted, 0x0540, None);
        for at in ["0x0u: if (!arclang_booting)", "0x62u: if (!arclang_booting)", "0x2a0u: if (!arclang_booting)"] {
            assert!(guarded.contains(at), "{at}: {guarded}");
        }
        // A function the turn may call through a pointer (a virtual write) is run as it is.
        assert!(guarded.contains("    case 0xdau: c_01b4(); return;"));
        // With the words the start leaves (a vtable naming 0x00da only), a computed call to any other function is not run.
        let vtable = words(&[(0x0104, 0xda), (0x0123, 0x04), (0x0124, 0x01)]);
        assert!(vtable.contains(&0x00da) && vtable.contains(&0x0104));
        let guarded = guard_reentry(lifted, 0x0540, Some(&vtable));
        assert!(guarded.contains("    case 0xdau: c_01b4(); return;"), "{guarded}");
        assert!(!guarded.contains("    case 0x2a0u: c_0540(); return;"));
    }

    #[test]
    fn the_indexes_of_the_serial_buffers_are_found_after_the_addresses_of_the_uart_registers() {
        // The serial lamp's object: the six register addresses from 0x012f, `_written` at 0x013b, the indexes after it.
        let constants: Vec<(u32, u8)> = [(0x0104, 113), (0x012f, 0xc5), (0x0130, 0), (0x0131, 0xc4), (0x0132, 0), (0x0133, 0xc0), (0x0134, 0), (0x0135, 0xc1), (0x0136, 0), (0x0137, 0xc2), (0x0138, 0), (0x0139, 0xc6), (0x013a, 0)].to_vec();
        assert_eq!(serial_indexes(&constants), [0x013c, 0x013d, 0x013e, 0x013f]);
        // No serial port (one register address missing): no index.
        assert!(serial_indexes(&constants[..11]).is_empty());
    }

    #[test]
    fn the_end_of_a_text_between_two_constants_is_a_constant_too() {
        // "on", its end, "off", its end, then two zeros of a variable and a byte set up once.
        let memory = vec![b'o', b'n', 0, b'o', b'f', b'f', 0, 0, 0, 7];
        let state = super::super::binary::StartState { reached: true, registers: vec![0; 32], sp: 0x08fd, low_sp: 0x08fd, memory, settings: Vec::new() };
        let found = constants("", &state, None);
        assert_eq!(found, vec![(0x0100, b'o'), (0x0101, b'n'), (0x0102, 0), (0x0103, b'o'), (0x0104, b'f'), (0x0105, b'f'), (0x0109, 7)], "one zero between two constants, not a run of zeros");
    }

    #[test]
    fn write_of_the_serial_port_is_the_first_function_of_its_vtable_and_a_turn_does_not_wait_in_it() {
        // The object at 0x011c: its vtable at 0x0104, whose first word is 0x0171 (the function at 0x02e2); its registers from 0x0128.
        let constants: Vec<(u32, u8)> = [(0x0104, 0x71), (0x0105, 1), (0x011c, 4), (0x011d, 1), (0x0128, 0xc5), (0x0129, 0), (0x012a, 0xc4), (0x012b, 0), (0x012c, 0xc0), (0x012d, 0), (0x012e, 0xc1), (0x012f, 0), (0x0130, 0xc2), (0x0131, 0), (0x0132, 0xc6), (0x0133, 0)].to_vec();
        assert_eq!(serial_write(&constants), Some(0x02e2));
        assert_eq!(serial_write(&constants[4..]), None, "no vtable known: no write() placed");

        let lifted = "void f_02e2(void)\n{\nL_02e2: /* push R14 */\n    if (a) goto L_0352;\nL_0316: /* ldd R24,Y+0x1c */\n    if (b) goto L_0316;\nL_0352: /* in R25,SREG */\n    return;\n}\nvoid f_0400(void)\n{\nL_0400: /* nop */\n    goto L_0400;\n}\n";
        let refused = refuse_wait(lifted, 0x02e2);
        assert!(refused.contains("L_0316: /* ldd R24,Y+0x1c */\n    if (!arclang_booting) { __CPROVER_assert(0, \"a turn does not wait in write() of the serial port\"); __CPROVER_assume(0); }\n"), "{refused}");
        assert_eq!(refused.matches("__CPROVER_assume(0)").count(), 1, "only the loops of write(), and no label a jump only goes forward to: {refused}");
        assert_eq!(refuse_wait(lifted, 0x0999), lifted, "no such function: nothing changed");
    }

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
    fn the_constants_of_a_turn_are_what_the_start_sets_up_and_the_code_never_writes_by_address() {
        // 0x0100 a vtable (other than zero), 0x0102 a variable of the sketch (written by `sts`, 7 at the start), 0x0104
        // a pointer of the serial port loaded into Z (its high byte 0x0105 is zero), 0x0106 an index (zero, loaded into
        // r18), 0x0108 a byte of the clock, 0x010a a byte of the stack (at or above the stack pointer the start leaves).
        let lifted = "L_0200: /* sts 0x0102,R24 */\nL_0204: /* lds Zlo,0x0104 */\nL_0208: /* lds Zhi,0x0105 */\nL_020c: /* lds R18,0x0106 */\n";
        let mut memory = vec![0u8; 12];
        memory[0] = 0x71;
        memory[2] = 7;
        memory[4] = 0xc5;
        memory[8] = 3;
        memory[10] = 9;
        let state = super::super::binary::StartState { reached: true, registers: vec![0; 32], sp: 0x010a, low_sp: 0x0109, memory, settings: Vec::new() };
        let found = constants(lifted, &state, Some((0x0108, 0x0200)));
        assert_eq!(found, vec![(0x0100, 0x71), (0x0104, 0xc5), (0x0105, 0)]);
        // What the start's stack left under the pointer it ends with is no object: the stack went down to 0x0103, and
        // the bytes above it, though the start leaves them other than zero, are what a turn's first call writes over.
        let deep = super::super::binary::StartState { low_sp: 0x0103, ..state.clone() };
        assert_eq!(constants(lifted, &deep, Some((0x0108, 0x0200))), vec![(0x0100, 0x71)]);
        // How low the stack went not said: nothing is taken as set up once.
        let unsaid = super::super::binary::StartState { low_sp: 0, ..state };
        assert!(constants(lifted, &unsaid, Some((0x0108, 0x0200))).is_empty());
    }

    #[test]
    fn a_pin_is_a_bit_of_the_register_of_its_port() {
        let lifted = "//@ language avr8:LE:16:default\n//@ memory 0x100 0x8ff\n//@ start 0x0\n//@ register SP register 0x3d 2\n//@ register R24 mem 0x18 1\n//@ register Cflg register 0x80 1\n";
        let cut = Cut { at: 0x390, function: 0x272, kept: vec![1, 2], start: None };
        let chip = Chip { reads: vec![2], writes: vec![13], ..Default::default() };
        let c = sketch(lifted, &cut, &chip, Some((0x10b, 0x10f)), &["arclang_unnamed".to_string()], "c").unwrap();
        // Pin 2 is bit 2 of the input register of port D; pin 13, bit 5 of the output register of port B.
        assert!(c.contains("case 0x29u: {\n        uint8_t level = nondet_uchar();\n        level = (uint8_t)((level & ~(1u << 2)) | (digitalRead(2) ? 1u << 2 : 0u));"), "{c}");
        assert!(c.contains("case 0x25u:\n        digitalWrite(13, (value >> 5) & 1u);\n        if (!arclang_booting && ((value ^ before) & ~0x20u) != 0) { arclang_unnamed = true; }"), "{c}");
        // Pin 13 is an output and no other pin is; the time is where the interrupt keeps it.
        assert!(c.contains("arclang_io[0x24u] == 0x20u && arclang_io[0x27u] == 0x00u && arclang_io[0x2au] == 0x00u"), "{c}");
        assert!(c.contains("arclang_mem[0x10bu] = (uint8_t)(count >> 0);") && c.contains("arclang_mem[0x112u] = (uint8_t)(turns >> 24);") && c.contains("f_0272();"), "{c}");
        assert!(c.contains("arclang_r[1] == arclang_kept[1] && arclang_r[2] == arclang_kept[2]") && c.contains("arclang_r[3] = nondet_uchar();") && !c.contains("arclang_r[2] = nondet_uchar();"), "{c}");
    }
}
