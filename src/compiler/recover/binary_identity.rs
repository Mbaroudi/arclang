//! The proof that a function of a binary is the Arduino function a
//! correspondence says it is.
//!
//! A function a correspondence names is not run in the proof of the
//! correspondence: the proof's stand-in is. That is sound only if the
//! function does what the stand-in assumes of it. For the functions of the
//! pins and for `millis`, that is proven here, of the machine code of the
//! function itself and of what it calls, on an ATmega328P wired as an
//! Arduino UNO (pins 0 to 7 on port D, 8 to 13 on port B, 14 to 19 on port
//! C):
//!
//! - `digitalRead(pin)` returns the bit of the pin in the input register of
//!   its port;
//! - `digitalWrite(pin, level)` sets the bit of the pin in the output
//!   register of its port to the level, and no other bit;
//! - `pinMode(pin, mode)` sets the direction of the pin, and its pull-up;
//! - `millis()` returns the four bytes of the memory it reads;
//! - `micros()` returns the turns and the count of timer 0 as microseconds;
//!
//! and each, from any memory and any registers, for every pin: runs only
//! instructions the translation covers, returns with the stack and the
//! registers a function must keep as they were, changes no byte of the
//! memory and no other register of the chip (the PWM of the pin apart,
//! which a digital read or write turns off), and leaves interrupts as they
//! were.
//!
//! The other functions (`delay`, the analog ones) are not proven
//! yet: the proof of the correspondence says it takes them on trust.

use super::equivalence::{decide, Answer};
use std::path::Path;

/// What was asked of a function a correspondence names.
#[derive(Debug, Clone, PartialEq)]
pub enum Identity {
    /// It is the function: what is proven of it, as it is printed.
    Proven(String),
    /// It is not: what it does not do.
    Refuted(Vec<String>),
    /// No answer: why.
    Unknown(String),
    /// Not one of the functions whose identity is proven yet.
    NotAsked,
}

/// What every function is asked: (the condition, as it is written).
const COMMON: [(&str, &str); 4] = [
    ("(!arclang_uncovered && !arclang_outside)", "it runs only instructions the translation covers, within the memory and the flash"),
    ("(arclang_reg_rd(ARCLANG_SP, 2) == sp0 && arclang_r[1] == 0 && arclang_r[kept] == kept_before)", "it returns with the stack, r1 and the registers a function must keep as they were"),
    ("(arclang_mem[byte_at] == byte_before)", "it changes no byte of the memory"),
    ("(arclang_reg[0x87] == interrupts_before)", "it leaves interrupts as they were"),
];

/// The control registers of the three timers: a digital read or write turns the PWM of its pin off there,
/// by clearing bits among the four that tie a pin to its timer; how the timer counts is left as it is.
const PWM_OFF: &str = "((register_at == 0x44u || register_at == 0x80u || register_at == 0xb0u) ? ((arclang_io[register_at] & (uint8_t)~register_before) == 0 && ((arclang_io[register_at] ^ register_before) & 0x0fu) == 0) : arclang_io[register_at] == register_before)";

/// The setting, the call and what is asked of one function: (what is declared and set before the call, the conditions after it, what is printed once proven).
fn asked(name: &str, at: u32, reads: &[u32]) -> Option<(String, Vec<(String, String)>, String)> {
    let pin = "    unsigned pin = nondet_uint();\n    __CPROVER_assume(pin < 20u);\n    unsigned mask = 1u << pin_bit(pin);\n    arclang_r[24] = (uint8_t)pin;\n";
    Some(match name {
        "digitalRead" => (
            format!("{pin}    uint8_t input = arclang_io[pin_register(pin, 0x29u, 0x23u, 0x26u)];\n"),
            vec![
                ("(arclang_mem_rd(24u, 2) == (uint64_t)((input & mask) != 0))".to_string(), "it returns the bit of the pin in the input register of its port".to_string()),
                (PWM_OFF.to_string(), "of the registers of the chip, it changes only the PWM of the pin, to turn it off".to_string()),
            ],
            format!("digitalRead at 0x{at:04x} returns the level of the pin, for each of the pins 0 to 19, and changes nothing but the PWM of the pin, turned off"),
        ),
        "digitalWrite" => (
            format!("{pin}    unsigned level = nondet_uchar();\n    arclang_r[22] = (uint8_t)level;\n    unsigned output_at = pin_register(pin, 0x2bu, 0x25u, 0x28u);\n    uint8_t output = arclang_io[output_at];\n"),
            vec![
                ("(arclang_io[output_at] == (uint8_t)(level != 0 ? (output | mask) : (output & ~mask)))".to_string(), "it sets the bit of the pin in the output register of its port to the level, and no other bit".to_string()),
                (format!("(register_at == output_at || {PWM_OFF})"), "of the other registers of the chip, it changes only the PWM of the pin, to turn it off".to_string()),
            ],
            format!("digitalWrite at 0x{at:04x} sets the level of the pin and of no other, for each of the pins 0 to 19, and changes nothing else but the PWM of the pin, turned off"),
        ),
        "pinMode" => (
            format!("{pin}    unsigned mode = nondet_uchar();\n    arclang_r[22] = (uint8_t)mode;\n    unsigned output_at = pin_register(pin, 0x2bu, 0x25u, 0x28u);\n    unsigned direction_at = output_at - 1u;\n    uint8_t output = arclang_io[output_at];\n    uint8_t direction = arclang_io[direction_at];\n"),
            vec![
                (
                    "(mode == 0u ? (arclang_io[direction_at] == (uint8_t)(direction & ~mask) && arclang_io[output_at] == (uint8_t)(output & ~mask)) : mode == 2u ? (arclang_io[direction_at] == (uint8_t)(direction & ~mask) && arclang_io[output_at] == (uint8_t)(output | mask)) : (arclang_io[direction_at] == (uint8_t)(direction | mask) && arclang_io[output_at] == output))".to_string(),
                    "it makes the pin an input (0), an input pulled up (2) or an output, and touches no other pin".to_string(),
                ),
                ("(register_at == output_at || register_at == direction_at || arclang_io[register_at] == register_before)".to_string(), "it changes no other register of the chip".to_string()),
            ],
            format!("pinMode at 0x{at:04x} makes the pin an input, an input pulled up or an output, for each of the pins 0 to 19, and changes nothing else"),
        ),
        "micros" => {
            // The four bytes of the memory it reads by address are the turns of timer 0; it reads the timer too.
            let ram: Vec<u32> = reads.iter().copied().filter(|a| *a >= 0x100).collect();
            let [first, _, _, last] = ram.as_slice() else { return None };
            if *last != first + 3 || !reads.contains(&0x46) || !reads.contains(&0x35) {
                return None;
            }
            (
                format!("    uint64_t turns = arclang_mem_rd(0x{first:x}u, 4);\n    uint64_t counted = arclang_io[0x46];\n    uint64_t pending = (arclang_io[0x35] & 1u) != 0 && counted < 255u ? 1u : 0u;\n"),
                vec![
                    ("(arclang_mem_rd(22u, 4) == (((((turns + pending) << 8) + counted) * 4u) & UINT64_C(0xffffffff)))".to_string(), format!("it returns 4 times the count of timer 0 and 256 times its turns (the four bytes at 0x{first:04x}, one more when an overflow is pending): the microseconds of a 16 MHz clock")),
                    ("(arclang_io[register_at] == register_before)".to_string(), "it changes no register of the chip".to_string()),
                ],
                format!("micros at 0x{at:04x} returns the time in microseconds of a 16 MHz clock: 4 a count of timer 0, 1024 a turn (the four bytes of the memory at 0x{first:04x}), and changes nothing"),
            )
        }
        "millis" => {
            // The four bytes of the memory it reads by address: the count it returns.
            let ram: Vec<u32> = reads.iter().copied().filter(|a| *a >= 0x100).collect();
            let [first, _, _, last] = ram.as_slice() else { return None };
            if *last != first + 3 {
                return None;
            }
            (
                format!("    uint64_t count = arclang_mem_rd(0x{first:x}u, 4);\n"),
                vec![
                    ("(arclang_mem_rd(22u, 4) == count)".to_string(), format!("it returns the four bytes of the memory at 0x{first:04x}")),
                    ("(arclang_io[register_at] == register_before)".to_string(), "it changes no register of the chip".to_string()),
                ],
                format!("millis at 0x{at:04x} returns the four bytes of the memory at 0x{first:04x} and changes nothing"),
            )
        }
        _ => return None,
    })
}

/// The proof of one function, as C: any memory and registers, the call, what must hold after it.
fn harness(at: u32, setting: &str, conditions: &[(String, String)]) -> String {
    let mut c = String::from("/* The proof that a function of the binary is the Arduino function it is said to be (written by `arclang prove`). */\n#define ARCLANG_IDENTITY\n#include \"identity_code.c\"\n\nunsigned nondet_uint(void);\n");
    c.push_str("/* An Arduino UNO: pins 0 to 7 on port D, 8 to 13 on port B, 14 to 19 on port C. */\nstatic unsigned pin_register(unsigned pin, unsigned d, unsigned b, unsigned c) { return pin < 8u ? d : pin < 14u ? b : c; }\nstatic unsigned pin_bit(unsigned pin) { return pin < 8u ? pin : pin < 14u ? pin - 8u : pin - 14u; }\n\n");
    c.push_str("int main(void)\n{\n    /* any memory, any registers */\n    __CPROVER_havoc_object(arclang_mem);\n    __CPROVER_havoc_object(arclang_r);\n    __CPROVER_havoc_object(arclang_reg);\n    __CPROVER_havoc_object(arclang_io);\n");
    c.push_str("    /* a flag of the processor is 0 or 1 */\n    for (unsigned flag = 0x80u; flag <= 0x88u; flag++) { __CPROVER_assume(arclang_reg[flag] <= 1); }\n");
    c.push_str("    __CPROVER_assume(arclang_r[1] == 0); /* the compiler's convention */\n    unsigned sp0 = (unsigned)arclang_reg_rd(ARCLANG_SP, 2);\n    __CPROVER_assume(sp0 >= 0x8c0u && sp0 <= 0x8ffu);\n");
    // One byte of the memory, one register of the chip and one register to keep, each any: what holds of it holds of all.
    c.push_str("    unsigned byte_at = nondet_uint();\n    __CPROVER_assume(byte_at >= ARCLANG_RAM_FIRST && byte_at < ARCLANG_MEM_SIZE && (byte_at > sp0 || byte_at + 32u <= sp0)); /* not the stack the function may use */\n    uint8_t byte_before = arclang_mem[byte_at];\n");
    c.push_str("    unsigned register_at = nondet_uint();\n    __CPROVER_assume(register_at >= ARCLANG_IO_FIRST && register_at < ARCLANG_RAM_FIRST && register_at != 0x5du && register_at != 0x5eu && register_at != 0x5fu);\n    uint8_t register_before = arclang_io[register_at];\n");
    c.push_str("    unsigned kept = nondet_uint();\n    __CPROVER_assume((kept >= 2u && kept <= 17u) || kept == 28u || kept == 29u);\n    uint8_t kept_before = arclang_r[kept];\n    uint8_t interrupts_before = arclang_reg[0x87];\n");
    c.push_str(setting);
    c.push_str(&format!("    /* the call: its return address is on the stack */\n    arclang_reg_wr(ARCLANG_SP, 2, sp0 - 2u);\n    f_{at:04x}();\n"));
    let all: Vec<(String, String)> = COMMON.iter().map(|(condition, written)| (condition.to_string(), written.to_string())).chain(conditions.iter().cloned()).collect();
    c.push_str("#ifdef ARCLANG_PART\n    switch (ARCLANG_PART) {\n");
    for (rank, (condition, written)) in all.iter().enumerate() {
        c.push_str(&format!("    case {rank}: __CPROVER_assert({condition}, \"{}\"); break;\n", written.replace('"', "'")));
    }
    c.push_str(&format!("    }}\n#else\n    __CPROVER_assert({}, \"the function is the one it is said to be\");\n#endif\n    return 0;\n}}\n", all.iter().map(|(condition, _)| condition.as_str()).collect::<Vec<_>>().join(" && ")));
    c
}

/// The memory addresses a function reads by name, as the translation says them.
fn reads(lifted: &str, at: u32) -> Vec<u32> {
    let prefix = format!("//@ reads 0x{at:04x}");
    lifted.lines().find_map(|line| line.strip_prefix(&prefix)).map(|rest| rest.split_whitespace().filter_map(|word| u32::from_str_radix(word.trim_start_matches("0x"), 16).ok()).collect()).unwrap_or_default()
}

/// Ask whether the function at `at` is the Arduino function `name`. `dir`
/// holds the files of the proof (the fixed-width types of the board among
/// them); the translation is written there once.
pub fn check(name: &str, at: u32, lifted: &str, dir: &Path, seconds: u64) -> Result<Identity, String> {
    let Some((setting, conditions, proven)) = asked(name, at, &reads(lifted, at)) else {
        return Ok(Identity::NotAsked);
    };
    let write = |file: &str, text: &str| std::fs::write(dir.join(file), text).map_err(|e| format!("cannot write {}: {e}", dir.join(file).display()));
    write("identity_code.c", lifted)?;
    write("arclang_lifted.h", super::binary::LIFTED_H)?;
    let source = format!("identity_{name}.c");
    write(&source, &harness(at, &setting, &conditions))?;
    let flags = ["-I", "board", "--16", "-DARCLANG_INT16", "--max-field-sensitivity-array-size", "256"];
    Ok(match decide(dir, &[source.as_str()], &flags, UNWIND, seconds)? {
        Answer::Unsat => Identity::Proven(proven),
        Answer::Sat(_) => {
            // Which part does not hold: each asked alone.
            let mut failing = Vec::new();
            let all = COMMON.iter().map(|(_, written)| written.to_string()).chain(conditions.iter().map(|(_, written)| written.clone()));
            for (rank, written) in all.enumerate() {
                let part = format!("-DARCLANG_PART={rank}");
                let mut one: Vec<&str> = flags.to_vec();
                one.push(part.as_str());
                if let Answer::Sat(_) = decide(dir, &[source.as_str()], &one, UNWIND, seconds)? {
                    failing.push(written);
                }
            }
            Identity::Refuted(failing)
        }
        Answer::BeyondBound => Identity::Unknown(format!("a loop of the function at 0x{at:04x} runs more than {UNWIND} times")),
        Answer::Unknown(reason) => Identity::Unknown(reason),
    })
}

/// What is asked of the clock of a binary, and what was answered.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Clock {
    /// What is proven, as it is printed.
    pub proven: Vec<String>,
    /// What does not hold.
    pub refuted: Vec<String>,
    pub unknown: Option<String>,
    /// Every interrupt of the binary is proven to change nothing the code of loop() reads.
    pub interrupts_settled: bool,
}

/// The proof of the interrupt of timer 0, as C: from any memory and registers, the routine run once.
fn interrupt_harness(at: u32, count_at: u32, fraction_at: u32, overflows_at: u32, own: &[u32]) -> (String, Vec<String>) {
    let conditions: Vec<(String, String)> = vec![
        ("(!arclang_uncovered && !arclang_outside)".to_string(), "it runs only instructions the translation covers, within the memory and the flash".to_string()),
        ("(arclang_reg_rd(ARCLANG_SP, 2) == sp0 && arclang_r[kept] == kept_before && arclang_reg[flag_at] == flag_before && arclang_reg[0x87] == 1)".to_string(), "it returns with the stack, every register and every flag as they were, and interrupts on".to_string()),
        (format!("({} || arclang_mem[byte_at] == byte_before)", own.iter().map(|a| format!("byte_at == 0x{a:x}u")).collect::<Vec<_>>().join(" || ")), "of the memory, it changes only its own bytes".to_string()),
        ("(arclang_io[register_at] == register_before)".to_string(), "it changes no register of the chip".to_string()),
        (
            format!("(arclang_mem[0x{fraction_at:x}u] == (uint8_t)(fraction + 3u >= 125u ? fraction + 3u - 125u : fraction + 3u) && arclang_mem_rd(0x{count_at:x}u, 4) == ((count + 1u + (fraction + 3u >= 125u ? 1u : 0u)) & UINT64_C(0xffffffff)))"),
            format!("it adds 1 to the four bytes at 0x{count_at:04x}, and 1 more every time the byte at 0x{fraction_at:04x}, which it adds 3 to, passes 125: 1.024 a turn"),
        ),
        (format!("(arclang_mem_rd(0x{overflows_at:x}u, 4) == ((overflows + 1u) & UINT64_C(0xffffffff)))"), format!("it counts its turns in the four bytes at 0x{overflows_at:04x}")),
    ];
    let mut c = String::from("/* The proof of what the interrupt of timer 0 does (written by `arclang prove`). */\n#define ARCLANG_IDENTITY\n#include \"identity_code.c\"\n\nunsigned nondet_uint(void);\n\nint main(void)\n{\n    /* any memory, any registers: an interrupt falls anywhere */\n    __CPROVER_havoc_object(arclang_mem);\n    __CPROVER_havoc_object(arclang_r);\n    __CPROVER_havoc_object(arclang_reg);\n    __CPROVER_havoc_object(arclang_io);\n");
    c.push_str("    for (unsigned flag = 0x80u; flag <= 0x88u; flag++) { __CPROVER_assume(arclang_reg[flag] <= 1); }\n    unsigned sp0 = (unsigned)arclang_reg_rd(ARCLANG_SP, 2);\n    __CPROVER_assume(sp0 >= 0x8c0u && sp0 <= 0x8ffu);\n");
    c.push_str("    unsigned byte_at = nondet_uint();\n    __CPROVER_assume(byte_at >= ARCLANG_RAM_FIRST && byte_at < ARCLANG_MEM_SIZE && (byte_at > sp0 || byte_at + 32u <= sp0));\n    uint8_t byte_before = arclang_mem[byte_at];\n");
    c.push_str("    unsigned register_at = nondet_uint();\n    __CPROVER_assume(register_at >= ARCLANG_IO_FIRST && register_at < ARCLANG_RAM_FIRST && register_at != 0x5du && register_at != 0x5eu && register_at != 0x5fu);\n    uint8_t register_before = arclang_io[register_at];\n");
    c.push_str("    unsigned kept = nondet_uint();\n    __CPROVER_assume(kept < 32u);\n    uint8_t kept_before = arclang_r[kept];\n    unsigned flag_at = nondet_uint();\n    __CPROVER_assume(flag_at >= 0x80u && flag_at <= 0x86u);\n    uint8_t flag_before = arclang_reg[flag_at];\n");
    c.push_str(&format!("    uint64_t count = arclang_mem_rd(0x{count_at:x}u, 4);\n    uint64_t overflows = arclang_mem_rd(0x{overflows_at:x}u, 4);\n    unsigned fraction = arclang_mem[0x{fraction_at:x}u];\n    __CPROVER_assume(fraction < 125u); /* what the routine itself keeps it under */\n"));
    c.push_str(&format!("    /* the interrupt: the address it came from is on the stack */\n    arclang_reg_wr(ARCLANG_SP, 2, sp0 - 2u);\n    f_{at:04x}();\n"));
    c.push_str(&format!("    __CPROVER_assert(arclang_mem[0x{fraction_at:x}u] < 125u || arclang_uncovered, \"the fraction stays under 125\");\n"));
    c.push_str("#ifdef ARCLANG_PART\n    switch (ARCLANG_PART) {\n");
    for (rank, (condition, written)) in conditions.iter().enumerate() {
        c.push_str(&format!("    case {rank}: __CPROVER_assert({condition}, \"{}\"); break;\n", written.replace('"', "'")));
    }
    c.push_str(&format!("    }}\n#else\n    __CPROVER_assert({}, \"the interrupt of timer 0 keeps the time\");\n#endif\n    return 0;\n}}\n", conditions.iter().map(|(condition, _)| condition.as_str()).collect::<Vec<_>>().join(" && ")));
    (c, conditions.into_iter().map(|(_, written)| written).collect())
}

/// The proof of how the start leaves timer 0, as C: the code run from its reset to its first loop().
fn start_harness(start: u32, loop_at: u32) -> (String, Vec<String>) {
    let conditions = [
        ("(arclang_stop && !arclang_uncovered && !arclang_outside)", "the code reaches loop() from its reset"),
        ("((arclang_io[0x45] & 0x0fu) == 0x03u && (arclang_io[0x44] & 0x03u) == 0x03u)", "it leaves timer 0 counting one in 64 cycles of the clock, and overflowing every 256 counts"),
        ("((arclang_io[0x6e] & 0x01u) == 0x01u && arclang_reg[0x87] == 1)", "it leaves the overflow of timer 0 an interrupt, and interrupts on"),
    ];
    let mut c = format!("/* The proof of how the start of the binary leaves timer 0 (written by `arclang prove`). */\n#define ARCLANG_IDENTITY\nstatic void arclang_first_loop(void);\n#define c_{loop_at:04x} arclang_first_loop\n#include \"identity_code.c\"\n\nstatic void arclang_first_loop(void) {{ arclang_stop = true; }}\n\nint main(void)\n{{\n    /* the chip as its reset leaves it: every register at zero */\n    arclang_booting = true;\n    f_{start:04x}();\n");
    c.push_str("#ifdef ARCLANG_PART\n    switch (ARCLANG_PART) {\n");
    for (rank, (condition, written)) in conditions.iter().enumerate() {
        c.push_str(&format!("    case {rank}: __CPROVER_assert({condition}, \"{written}\"); break;\n"));
    }
    c.push_str(&format!("    }}\n#else\n    __CPROVER_assert({}, \"the start sets the clock\");\n#endif\n    return 0;\n}}\n", conditions.iter().map(|(condition, _)| *condition).collect::<Vec<_>>().join(" && ")));
    (c, conditions.iter().map(|(_, written)| written.to_string()).collect())
}

/// Ask one harness, and which of its parts fail when it does not hold.
fn ask(dir: &Path, source: &str, text: &str, parts: &[String], unwind: usize, seconds: u64) -> Result<Result<(), Vec<String>>, String> {
    std::fs::write(dir.join(source), text).map_err(|e| format!("cannot write {}: {e}", dir.join(source).display()))?;
    let flags = ["-I", "board", "--16", "-DARCLANG_INT16", "--max-field-sensitivity-array-size", "256"];
    match decide(dir, &[source], &flags, unwind, seconds)? {
        Answer::Unsat => Ok(Ok(())),
        Answer::Sat(_) => {
            let mut failing = Vec::new();
            for (rank, written) in parts.iter().enumerate() {
                let part = format!("-DARCLANG_PART={rank}");
                let mut one: Vec<&str> = flags.to_vec();
                one.push(part.as_str());
                if let Answer::Sat(_) = decide(dir, &[source], &one, unwind, seconds)? {
                    failing.push(written.clone());
                }
            }
            Ok(Err(failing))
        }
        Answer::BeyondBound => Err(format!("a loop runs more than {unwind} times")),
        Answer::Unknown(reason) => Err(reason),
    }
}

/// The clock of a binary: that the four bytes `millis` returns (at
/// `count_at`) are kept by the interrupt of timer 0 as the Arduino core
/// keeps them, that the start leaves timer 0 overflowing every 1.024 ms of
/// a 16 MHz clock, and that no interrupt changes what the code of `loop`
/// (the functions at `loop_code`) reads by address.
pub fn clock(vectors: &[u32], start: u32, loop_at: u32, loop_code: &[u32], count_at: Option<u32>, turns_at: Option<u32>, lifted: &str, dir: &Path, seconds: u64) -> Result<Clock, String> {
    let mut clock = Clock::default();
    let Some(&routine) = vectors.get(super::binary::TIMER0_OVERFLOW) else { return Ok(clock) };
    let (_, functions) = super::binary_shape::facts(lifted)?;
    let Some(facts) = functions.get(&routine) else {
        clock.unknown = Some(format!("the routine of the overflow of timer 0, at 0x{routine:04x}, was not translated"));
        return Ok(clock);
    };
    // Its own bytes: the count of milliseconds and its turns (four bytes in a row each) and the fraction (one byte).
    // Which run is which is known by what millis returns (the count) or by what micros reads (the turns).
    let own: Vec<u32> = facts.writes.iter().copied().filter(|a| *a >= 0x100).collect();
    let runs = |fraction: u32| -> Option<(u32, u32)> {
        let rest: Vec<u32> = own.iter().copied().filter(|a| *a != fraction).collect();
        (rest.len() == 8 && rest[3] == rest[0] + 3 && rest[7] == rest[4] + 3).then(|| (rest[0], rest[4]))
    };
    let layout = own.iter().find_map(|fraction| runs(*fraction).map(|(low, high)| (*fraction, low, high))).and_then(|(fraction, low, high)| match (count_at, turns_at) {
        (Some(count), turns) if (count == low || count == high) && turns.is_none_or(|turns| turns == low + high - count) => Some((fraction, count, low + high - count)),
        (None, Some(turns)) if turns == low || turns == high => Some((fraction, low + high - turns, turns)),
        _ => None,
    });
    let Some((fraction_at, count_at, overflows_at)) = layout else {
        clock.unknown = Some(format!("the routine of the overflow of timer 0, at 0x{routine:04x}, does not write the nine bytes the Arduino core's does (a count, a fraction, its turns) where millis and micros read them"));
        return Ok(clock);
    };
    let fraction_at = &fraction_at;
    std::fs::write(dir.join("identity_code.c"), lifted).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("arclang_lifted.h"), super::binary::LIFTED_H).map_err(|e| e.to_string())?;
    let (text, parts) = interrupt_harness(routine, count_at, *fraction_at, overflows_at, &own);
    match ask(dir, "identity_timer0.c", &text, &parts, UNWIND, seconds) {
        Ok(Ok(())) => clock.proven.push(format!("the interrupt of timer 0 (at 0x{routine:04x}) adds 1.024 to the count of milliseconds (the four bytes at 0x{count_at:04x}, which millis returns) and 1 to its turns (the four bytes at 0x{overflows_at:04x}, which micros reads) at each overflow of the timer, and changes nothing else")),
        Ok(Err(failing)) => clock.refuted.extend(failing.iter().map(|part| format!("the interrupt of timer 0, at 0x{routine:04x}: {part}"))),
        Err(reason) => clock.unknown = Some(format!("the interrupt of timer 0, at 0x{routine:04x}: {reason}")),
    }
    let (text, parts) = start_harness(start, loop_at);
    match ask(dir, "identity_start.c", &text, &parts, START_UNWIND, seconds) {
        Ok(Ok(())) => clock.proven.push("the start leaves timer 0 overflowing every 256 x 64 cycles of the clock (1.024 ms at 16 MHz), its overflow an interrupt, and interrupts on".to_string()),
        Ok(Err(failing)) => clock.refuted.extend(failing.iter().map(|part| format!("the start of the binary: {part}"))),
        Err(reason) => clock.unknown = clock.unknown.take().or(Some(format!("the start of the binary: {reason}"))),
    }
    // The other interrupts: none, when every other vector goes where most go (the routine of an interrupt nobody takes).
    let mut places: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    for to in &vectors[1..] {
        *places.entry(*to).or_default() += 1;
    }
    let nobody = places.iter().max_by_key(|(_, count)| **count).map(|(to, _)| *to);
    let others = vectors.iter().enumerate().skip(1).filter(|(vector, to)| *vector != super::binary::TIMER0_OVERFLOW && Some(**to) != nobody).count();
    // What the code of loop() reads or writes by address is none of the routine's bytes, and it uses no pointer.
    let code: Vec<&super::binary_shape::Function> = loop_code.iter().filter_map(|at| functions.get(at)).collect();
    let shared = code.iter().flat_map(|f| f.reads.iter().chain(&f.writes)).any(|a| own.contains(a));
    let pointers = code.iter().any(|f| f.loads + f.stores > 0);
    if clock.refuted.is_empty() && clock.unknown.is_none() && others == 0 && !shared && !pointers {
        clock.interrupts_settled = true;
        clock.proven.push("it is the only interrupt of the binary, and the code of loop() reads none of its bytes: an interrupt changes nothing loop() reads".to_string());
    }
    Ok(clock)
}

/// The start of a binary copies and clears at most the memory of the chip.
const START_UNWIND: usize = 2100;

/// The loops of these functions are those of the C beneath the translation (the bytes of a value, at most 8).
const UNWIND: usize = 70;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_asked_of_a_function_is_written_as_c() {
        let (setting, conditions, proven) = asked("digitalWrite", 0x4b6, &[]).unwrap();
        let c = harness(0x4b6, &setting, &conditions);
        assert!(c.contains("f_04b6();") && c.contains("arclang_r[22] = (uint8_t)level;") && c.contains("__CPROVER_havoc_object(arclang_io);"), "{c}");
        assert!(c.contains("case 4: __CPROVER_assert((arclang_io[output_at] == (uint8_t)(level != 0 ? (output | mask) : (output & ~mask)))"), "{c}");
        assert!(proven.starts_with("digitalWrite at 0x04b6 sets the level of the pin and of no other"));
    }

    #[test]
    fn millis_is_asked_of_the_four_bytes_it_reads_and_the_others_are_not_asked_yet() {
        let lifted = "//@ reads 0x029c 0x10b 0x10c 0x10d 0x10e\n//@ reads 0x0300 0x10b 0x10c\n";
        let (setting, _, proven) = asked("millis", 0x29c, &reads(lifted, 0x29c)).unwrap();
        assert!(setting.contains("arclang_mem_rd(0x10bu, 4)") && proven.contains("the four bytes of the memory at 0x010b"));
        assert!(asked("millis", 0x300, &reads(lifted, 0x300)).is_none());
        // micros reads its turns, the count of the timer and its pending overflow.
        let (setting, conditions, _) = asked("micros", 0x29e, &[0x35, 0x46, 0x10f, 0x110, 0x111, 0x112]).unwrap();
        assert!(setting.contains("arclang_mem_rd(0x10fu, 4)") && conditions[0].0.contains("* 4u"), "{setting}");
        assert!(asked("micros", 0x29e, &[0x10f, 0x110, 0x111, 0x112]).is_none());
        assert!(asked("delay", 0x2fe, &[]).is_none());
    }
}
