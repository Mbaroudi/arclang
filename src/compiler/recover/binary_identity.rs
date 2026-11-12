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
//!
//! and each, from any memory and any registers, for every pin: runs only
//! instructions the translation covers, returns with the stack and the
//! registers a function must keep as they were, changes no byte of the
//! memory and no other register of the chip (the PWM of the pin apart,
//! which a digital read or write turns off), and leaves interrupts as they
//! were.
//!
//! The other functions (`delay`, `micros`, the analog ones) are not proven
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

/// The control registers of the three timers: a digital read or write turns the PWM of its pin off there.
const PWM_OFF: &str = "((register_at == 0x44u || register_at == 0x80u || register_at == 0xb0u) ? (arclang_io[register_at] & (uint8_t)~register_before) == 0 : arclang_io[register_at] == register_before)";

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
        assert!(asked("delay", 0x2fe, &[]).is_none());
    }
}
