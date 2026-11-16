//! A firmware binary, without its sources, as code a proof can read.
//!
//! The machine code is translated to C one instruction at a time, from the
//! meaning Ghidra's description of the processor gives each instruction
//! (`Lift.java`, run headless). Nothing is decompiled: a register is a cell
//! of an array, a jump a `goto`, the memory an array of bytes. That C takes
//! the place of the sketch in the proof of a `correspondence`:
//!
//! ```arclang
//! correspondence "Lamp, as flashed" {
//!     machine: "Lamp toggled by a button"
//!     binary: "uno_lamp.hex"
//!     period: 10 ms
//!     functions: ["loop at 0x0102", "pinMode at 0x043e", "digitalRead at 0x0514",
//!                 "digitalWrite at 0x04b6", "millis at 0x029c", "delay at 0x02fe"]
//!     reads: ["digitalRead(2) is button"]
//!     writes: ["digitalWrite(13) is led"]
//!     state: ["byte(0x0106) == 1 is in Down", "millis() - dword(0x0100) is time in Arming"]
//! }
//! ```
//!
//! The code runs from its reset to the first call of `loop` (that is the
//! `setup()` of the proof), then `loop` is one turn. The Arduino functions
//! the correspondence names are not run: they are the stand-ins of the
//! proof, called as the compiler of the board calls them (avr-gcc: the first
//! argument in r24, the result in r24, or r22 to r25 for a long). That they
//! are those functions is what the correspondence says, and what the proof
//! takes on trust; every other instruction is run as it is.
//!
//! The chip is an ATmega328P (an Arduino UNO); the image starts at address 0.

use regex::Regex;
use sha2::{Digest, Sha256};
use std::path::Path;

/// What machine code translated to C runs on: memory, registers, flash.
pub const LIFTED_H: &str = include_str!("lifted.h");
/// The translation, as a Ghidra script.
const LIFT_JAVA: &str = include_str!("Lift.java");
/// The processor Ghidra reads the image as: an AVR with a program counter of 16 bits.
const LANGUAGE: &str = "avr8:LE:16:default";
/// What the translation must find of that processor, for the stand-ins to read the right cells.
const EXPECTED: [&str; 5] = [
    "//@ language avr8:LE:16:default",
    "//@ memory 0x100 0x8ff",
    "//@ register SP register 0x3d 2",
    "//@ register R24 mem 0x18 1",
    "//@ register Cflg register 0x80 1",
];
/// The operations of the chip the C beneath the translation knows (`lifted.h`).
const OPERATIONS: [&str; 4] = ["watchdog_reset", "sleep", "break", "store_program_mem"];

/// The binary a correspondence names, and where its functions are.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Binary {
    pub file: String,
    /// (the function, its address in the flash)
    pub functions: Vec<(String, u32)>,
    /// The head of the cycle of main, for a binary whose loop is folded into it (`cycle at 0x0390`):
    /// no function is then named, and the whole of the code is run on a model of the chip.
    pub cycle: Option<u32>,
    /// For a cycle: the time that passes each time the code has read the timer, in microseconds.
    /// Set by the proof from the correspondence.
    pub tick_us: u64,
    /// The functions proven to be the Arduino functions they are said to be: (the function, what is proven of it).
    /// Filled by the proof; empty as the correspondence is read.
    pub identified: Vec<(String, String)>,
    /// What is proven of the clock of the binary (the interrupt of timer 0 and the start that sets it).
    /// Filled by the proof.
    pub clock: Vec<String>,
    /// The interrupts of the binary are all proven to change nothing the code of loop() reads.
    pub interrupts_settled: bool,
    /// Why the clock of the binary could not be established, when it could not.
    pub clock_unknown: Option<String>,
    /// For a cycle: the routine of the interrupt of the UART (the serial port), run to empty the transmit buffer.
    pub uart: Option<u32>,
    /// For a cycle: the pins driven by a timer (`analogWrite`).
    pub pwm: Vec<u8>,
}

/// The default target of the interrupt vectors, where the ones nobody takes go, and the routine the vector
/// of the UART's transmit buffer (data register empty) goes to when it is another: the binary has a serial port.
pub fn uart_routine(vectors: &[u32]) -> Option<u32> {
    let mut places: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    for to in vectors.iter().skip(1) {
        *places.entry(*to).or_default() += 1;
    }
    let nobody = places.iter().max_by_key(|(_, count)| **count).map(|(to, _)| *to)?;
    vectors.get(UART_DATA_EMPTY).copied().filter(|to| *to != nobody)
}

/// The vector of the UART data register empty on an ATmega328P.
pub const UART_DATA_EMPTY: usize = 19;

/// An Arduino function a correspondence can name: the bytes of its
/// arguments, the bytes of what it returns, and whether its last argument
/// is signed.
struct Known {
    name: &'static str,
    arguments: &'static [usize],
    returns: usize,
    signed: bool,
}

const KNOWN: [Known; 9] = [
    Known {
        name: "pinMode",
        arguments: &[1, 1],
        returns: 0,
        signed: false,
    },
    Known {
        name: "digitalRead",
        arguments: &[1],
        returns: 2,
        signed: false,
    },
    Known {
        name: "digitalWrite",
        arguments: &[1, 1],
        returns: 0,
        signed: false,
    },
    Known {
        name: "analogRead",
        arguments: &[1],
        returns: 2,
        signed: false,
    },
    Known {
        name: "analogWrite",
        arguments: &[1, 2],
        returns: 0,
        signed: true,
    },
    Known {
        name: "millis",
        arguments: &[],
        returns: 4,
        signed: false,
    },
    Known {
        name: "micros",
        arguments: &[],
        returns: 4,
        signed: false,
    },
    Known {
        name: "delay",
        arguments: &[4],
        returns: 0,
        signed: false,
    },
    Known {
        name: "delayMicroseconds",
        arguments: &[2],
        returns: 0,
        signed: false,
    },
];

/// Read the `functions` of a correspondence: `<name> at 0x<address>`.
pub fn functions(lines: &[String], what: &str) -> Result<Vec<(String, u32)>, String> {
    let form =
        Regex::new(r"^(\w+) at 0[xX]([0-9a-fA-F]{1,6})$").expect("a pattern of the correspondence");
    let mut found: Vec<(String, u32)> = Vec::new();
    for line in lines {
        let read = form.captures(line.trim()).ok_or_else(|| {
            format!("{what}: \"{line}\" does not say where a function is (`loop at 0x0102`)")
        })?;
        let name = read[1].to_string();
        let address =
            u32::from_str_radix(&read[2], 16).map_err(|e| format!("{what}: \"{line}\": {e}"))?;
        if name != "loop" && name != "cycle" && !KNOWN.iter().any(|known| known.name == name) {
            return Err(format!(
                "{what}: '{name}' is not a function a correspondence can place (loop, {})",
                KNOWN
                    .iter()
                    .map(|known| known.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if address % 2 != 0 {
            return Err(format!(
                "{what}: \"{line}\": an instruction of the chip starts at an even address"
            ));
        }
        if found
            .iter()
            .any(|(other, at)| *other == name || *at == address)
        {
            return Err(format!(
                "{what}: \"{line}\": the function or the address is given twice"
            ));
        }
        found.push((name, address));
    }
    // A cycle is the whole of the code: nothing is stood in for, so nothing else is named.
    if found.iter().any(|(name, _)| name == "cycle") {
        if found.len() > 1 {
            return Err(format!("{what}: with a cycle (a binary whose loop is folded into main), every function is run: only the cycle is placed (functions: [\"cycle at 0x0390\"])"));
        }
        return Ok(found);
    }
    if !found.iter().any(|(name, _)| name == "loop") {
        return Err(format!("{what}: it does not say where loop is in the binary (functions: [\"loop at 0x0102\", ...])"));
    }
    Ok(found)
}

/// The bytes of a binary, and the address of the first.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub base: u32,
    pub bytes: Vec<u8>,
}

/// Read an Intel HEX file: the bytes from the lowest address written to the
/// highest, 0xFF where nothing is written (an erased flash).
pub fn intel_hex(text: &str) -> Result<Image, String> {
    let mut written: std::collections::BTreeMap<u32, u8> = std::collections::BTreeMap::new();
    let mut upper = 0u32;
    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let wrong = |why: &str| format!("line {}: {why}", number + 1);
        let digits = line
            .strip_prefix(':')
            .ok_or_else(|| wrong("an Intel HEX record starts with ':'"))?;
        if digits.len() % 2 != 0 || !digits.is_ascii() {
            return Err(wrong("a record is pairs of hexadecimal digits"));
        }
        let record: Vec<u8> = (0..digits.len() / 2)
            .map(|k| u8::from_str_radix(&digits[2 * k..2 * k + 2], 16))
            .collect::<Result<_, _>>()
            .map_err(|_| wrong("a record is pairs of hexadecimal digits"))?;
        if record.len() < 5 || record.len() != usize::from(record[0]) + 5 {
            return Err(wrong("the record is not as long as it says"));
        }
        if record.iter().fold(0u8, |sum, byte| sum.wrapping_add(*byte)) != 0 {
            return Err(wrong("the checksum is wrong"));
        }
        let (count, address, data) = (
            usize::from(record[0]),
            u32::from(record[1]) << 8 | u32::from(record[2]),
            &record[4..record.len() - 1],
        );
        match record[3] {
            0 => {
                for (k, byte) in data.iter().enumerate() {
                    written.insert(upper + address + k as u32, *byte);
                }
            }
            1 => break,
            2 if count == 2 => upper = (u32::from(data[0]) << 8 | u32::from(data[1])) << 4,
            4 if count == 2 => upper = (u32::from(data[0]) << 8 | u32::from(data[1])) << 16,
            3 | 5 => {}
            kind => {
                return Err(wrong(&format!(
                    "a record of type {kind} is not one of an Intel HEX file"
                )))
            }
        }
    }
    let (Some((&low, _)), Some((&high, _))) = (written.first_key_value(), written.last_key_value())
    else {
        return Err("the file holds no byte".to_string());
    };
    if high - low >= 1 << 20 {
        return Err(
            "the file spans more than 1 MiB: not the flash of a microcontroller".to_string(),
        );
    }
    Ok(Image {
        base: low,
        bytes: (low..=high)
            .map(|at| written.get(&at).copied().unwrap_or(0xFF))
            .collect(),
    })
}

/// Translate the machine code of an image to C, with Ghidra (`analyzeHeadless`,
/// or the command `ARCLANG_GHIDRA` names), in `dir`. A translation already
/// made of the same image, for the same functions, is read again.
pub fn lift(image: &Image, entries: &[u32], cut: Option<u32>, dir: &Path) -> Result<String, String> {
    use std::process::{Command, Stdio};
    if image.base != 0 {
        return Err(format!("the image starts at address 0x{:04X}: only a program that starts at address 0 is read (not a bootloader)", image.base));
    }
    if image.bytes.len() > 0x8000 {
        return Err(format!(
            "the image has {} bytes: the flash of an ATmega328P has 32768",
            image.bytes.len()
        ));
    }
    let mut hash = Sha256::new();
    hash.update(&image.bytes);
    hash.update(LIFT_JAVA.as_bytes());
    if let Some(cut) = cut {
        hash.update(b"cut");
        hash.update(cut.to_le_bytes());
    }
    for entry in entries {
        hash.update(entry.to_le_bytes());
    }
    let key = format!("/* {:x} */\n", hash.finalize());
    let out = dir.join("lifted.c");
    if let Ok(text) = std::fs::read_to_string(&out) {
        if let Some(lifted) = text.strip_prefix(&key) {
            return Ok(lifted.to_string());
        }
    }
    let write = |name: &str, bytes: &[u8]| {
        std::fs::write(dir.join(name), bytes)
            .map_err(|e| format!("cannot write {}: {e}", dir.join(name).display()))
    };
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let _ = std::fs::remove_dir_all(dir.join("project"));
    std::fs::create_dir_all(dir.join("project"))
        .map_err(|e| format!("cannot create {}: {e}", dir.join("project").display()))?;
    write("image.bin", &image.bytes)?;
    std::fs::create_dir_all(dir.join("scripts"))
        .map_err(|e| format!("cannot create {}: {e}", dir.join("scripts").display()))?;
    write("scripts/Lift.java", LIFT_JAVA.as_bytes())?;
    let _ = std::fs::remove_file(dir.join("raw.c"));
    let ghidra = std::env::var("ARCLANG_GHIDRA")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "analyzeHeadless".to_string());
    let mut command = Command::new(&ghidra);
    // Ghidra finds a script only in a directory given whole: the directory is named from the root.
    let scripts = dir
        .join("scripts")
        .canonicalize()
        .map_err(|e| format!("cannot find {}: {e}", dir.join("scripts").display()))?;
    command
        .args([
            "project",
            "image",
            "-import",
            "image.bin",
            "-processor",
            LANGUAGE,
            "-cspec",
            "gcc",
            "-loader",
            "BinaryLoader",
            "-scriptPath",
        ])
        .arg(&scripts)
        .args(["-postScript", "Lift.java", "raw.c"]);
    command.args(entries.iter().map(|entry| format!("0x{entry:x}")));
    command.args(cut.iter().map(|cut| format!("cut:0x{cut:x}")));
    let log = std::fs::File::create(dir.join("ghidra.log"))
        .map_err(|e| format!("cannot write {}: {e}", dir.join("ghidra.log").display()))?;
    let errors = log.try_clone().map_err(|e| e.to_string())?;
    let status = command.current_dir(dir).stdin(Stdio::null()).stdout(log).stderr(errors).status().map_err(|e| {
        format!("Ghidra could not be run ({ghidra}: {e}): a binary is read with Ghidra's `analyzeHeadless`, or the command ARCLANG_GHIDRA names (tools/embedded/reverse builds an image that has it)")
    })?;
    let lifted = std::fs::read_to_string(dir.join("raw.c")).map_err(|_| {
        let said = std::fs::read_to_string(dir.join("ghidra.log")).unwrap_or_default();
        let last: Vec<&str> = said
            .lines()
            .filter(|line| line.contains("ERROR"))
            .take(3)
            .collect();
        format!(
            "Ghidra translated nothing ({status}){}",
            if last.is_empty() {
                String::new()
            } else {
                format!(": {}", last.join(" | "))
            }
        )
    })?;
    same_flash(image, &lifted)?;
    std::fs::write(&out, format!("{key}{lifted}"))
        .map_err(|e| format!("cannot write {}: {e}", out.display()))?;
    Ok(lifted)
}

/// The interrupt vectors of an ATmega328P: 26 jumps at the start of the
/// flash, four bytes each. Where each goes, or none when the image does not
/// start with such a table.
pub fn vectors(image: &Image) -> Option<Vec<u32>> {
    if image.base != 0 || image.bytes.len() < 26 * 4 {
        return None;
    }
    (0..26).map(|vector| {
        let at = &image.bytes[4 * vector..4 * vector + 4];
        // jmp k, k on 16 bits: 0x940C then the address in words.
        (at[0] == 0x0C && at[1] == 0x94).then(|| (u32::from(at[2]) | u32::from(at[3]) << 8) * 2)
    }).collect()
}

/// The vector of the overflow of timer 0, which the Arduino core counts the time with.
pub const TIMER0_OVERFLOW: usize = 16;

/// What the start leaves, found by running it: whether it reached the cut (or the first loop), the registers
/// r0..r31 there and the stack pointer. The start is deterministic: it is run once, natively, on the translation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StartState {
    pub reached: bool,
    pub registers: Vec<u8>,
    pub sp: u16,
    /// The lowest the stack pointer went while the start ran; 0 when it was not said.
    pub low_sp: u16,
    /// The memory the start leaves, from its first address (`//@ memory`) on; empty when it was not read.
    pub memory: Vec<u8>,
    /// What the start leaves in the settings of the timers (their control registers, the interrupts they raise):
    /// registers only the code writes. Empty when it was not said.
    pub settings: Vec<(u32, u8)>,
}

/// The settings of the timers of an ATmega328P: TCCR0A, TCCR0B, TIMSK0 to TIMSK2, TCCR1A to TCCR1C, TCCR2A, TCCR2B.
pub const SETTINGS: [u32; 10] = [0x44, 0x45, 0x6e, 0x6f, 0x70, 0x80, 0x81, 0x82, 0xb0, 0xb1];

/// How many times the translation is made again with the targets of computed jumps found by running the start.
const DISCOVER_ROUNDS: usize = 6;
/// The time the start may take when it is run to find them, in seconds.
const DISCOVER_SECONDS: u64 = 30;

/// Translate the machine code, as `lift` does, then run its start (deterministic: from the reset, until it
/// reaches the cut or the first loop) on the translation itself, natively, to find the functions a computed
/// jump goes to: the global constructors of C++ objects, which the start calls by a table of the flash. The
/// code is translated again with them as functions, until the start meets no target it does not know. A
/// target left unknown stays not covered (a flag of the proof): nothing is guessed.
pub fn lift_discovering(image: &Image, entries: &[u32], cut: Option<u32>, dir: &Path) -> Result<String, String> {
    lift_with_start(image, entries, cut, dir).map(|(lifted, _)| lifted)
}

/// `lift_discovering`, and what the start of the final translation leaves.
pub fn lift_with_start(image: &Image, entries: &[u32], cut: Option<u32>, dir: &Path) -> Result<(String, StartState), String> {
    lift_with_entries(image, entries, cut, dir).map(|(lifted, state, _)| (lifted, state))
}

/// `lift_with_start`, and the functions the final translation was made with: those given and those found.
pub fn lift_with_entries(image: &Image, entries: &[u32], cut: Option<u32>, dir: &Path) -> Result<(String, StartState, Vec<u32>), String> {
    let mut entries = entries.to_vec();
    let mut lifted = lift(image, &entries, cut, dir)?;
    let mut state = StartState::default();
    for _ in 0..DISCOVER_ROUNDS {
        let (targets, left) = indirect_targets(&lifted, dir)?;
        state = left;
        let found: Vec<u32> = targets.into_iter().filter(|at| !entries.contains(at)).collect();
        if found.is_empty() {
            break;
        }
        entries.extend(found);
        lifted = lift(image, &entries, cut, dir)?;
    }
    Ok((lifted, state, entries))
}

/// The targets of the computed jumps the start of a translation takes that are no function of it.
fn indirect_targets(lifted: &str, dir: &Path) -> Result<(Vec<u32>, StartState), String> {
    use std::process::{Command, Stdio};
    let start = lifted.lines().find_map(|line| line.strip_prefix("//@ start 0x")).and_then(|at| u32::from_str_radix(at, 16).ok()).ok_or("the translation does not say where the code starts")?;
    let write = |name: &str, text: &str| std::fs::write(dir.join(name), text).map_err(|e| format!("cannot write {}: {e}", dir.join(name).display()));
    write("arclang_lifted.h", LIFTED_H)?;
    // The run ends itself after the time it is given (alarm): killing the command that started it does not end it
    // when that command is a container's client, and a start that never reaches the cut would run for ever.
    write("discover.c", &format!("#define ARCLANG_NATIVE_DISCOVER\n#include <signal.h>\n#include <unistd.h>\n#include \"lifted.c\"\nunsigned char nondet_uchar(void) {{ return 0; }}\n/* Out of its time (a start that reaches no cut: it runs the loop of the code for ever): what it found is said, and it ends. */\nstatic void arclang_out_of_time(int signal) {{ (void)signal; fflush(stdout); _exit(0); }}\nint main(void)\n{{\n    signal(SIGALRM, arclang_out_of_time);\n    alarm({DISCOVER_SECONDS}u);\n    arclang_booting = true;\n    f_{start:04x}();\n    printf(\"reached %d\\n\", (int)arclang_stop);\n    printf(\"registers\");\n    for (unsigned k = 0; k < 32u; k++) {{ printf(\" %u\", (unsigned)arclang_r[k]); }}\n    printf(\"\\nsp %u\\n\", (unsigned)arclang_reg_rd(ARCLANG_SP, 2));\n    printf(\"low_sp %u\\n\", arclang_low_sp);\n    printf(\"settings\");\n    for (unsigned k = 0; k < 0x100u; k++) {{ if (arclang_is_setting(k)) {{ printf(\" %u:%u\", k, (unsigned)arclang_settings[k]); }} }}\n    printf(\"\\n\");\n    printf(\"memory\");\n    for (unsigned k = ARCLANG_RAM_FIRST; k < ARCLANG_MEM_SIZE; k++) {{ printf(\" %u\", (unsigned)arclang_mem[k]); }}\n    printf(\"\\n\");\n    return 0;\n}}\n"))?;
    // Compiled and run in the same place: the command ARCLANG_NATIVE names (it is given discover.c, and compiles and
    // runs it, as a container does), or the C compiler of this computer.
    let _ = std::fs::remove_file(dir.join("discover"));
    let mut command = match std::env::var("ARCLANG_NATIVE").ok().filter(|v| !v.trim().is_empty()) {
        Some(runner) => {
            let mut command = Command::new(runner);
            command.arg("discover.c");
            command
        }
        None => {
            let mut command = Command::new("sh");
            command.args(["-c", "${CC:-cc} -O1 -w -o discover discover.c && ./discover"]);
            command
        }
    };
    let mut child = command.current_dir(dir).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| format!("cannot run the translation to find the targets of computed jumps ({e}): set ARCLANG_NATIVE to a command that compiles and runs a C file"))?;
    let began = std::time::Instant::now();
    let mut killed = false;
    loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(_) => break,
            None if began.elapsed().as_secs() > DISCOVER_SECONDS => {
                let _ = child.kill();
                killed = true;
                break;
            }
            None => std::thread::sleep(std::time::Duration::from_millis(20)),
        }
    }
    let mut said = String::new();
    if let Some(mut out) = child.stdout.take() {
        use std::io::Read;
        let _ = out.read_to_string(&mut said);
    }
    let mut complaint = String::new();
    if let Some(mut err) = child.stderr.take() {
        use std::io::Read;
        let _ = err.read_to_string(&mut complaint);
    }
    let status = child.wait().map_err(|e| e.to_string())?;
    // Ended by its own alarm (128 + SIGALRM, as a shell says it) is out of time too, not a translation that does not compile.
    let killed = killed || status.code() == Some(142);
    if !status.success() && !killed {
        return Err(format!("the translation could not be compiled and run to find the targets of computed jumps ({status}):\n{}", complaint.lines().take(6).collect::<Vec<_>>().join("\n")));
    }
    let mut found: Vec<u32> = said.lines().filter_map(|line| line.strip_prefix("target 0x")).filter_map(|at| u32::from_str_radix(at.trim(), 16).ok()).filter(|at| at % 2 == 0 && *at < 0x8000).collect();
    found.sort_unstable();
    found.dedup();
    let registers: Vec<u8> = said.lines().find_map(|line| line.strip_prefix("registers ")).map(|rest| rest.split_whitespace().filter_map(|value| value.parse::<u8>().ok()).collect()).unwrap_or_default();
    let sp = said.lines().find_map(|line| line.strip_prefix("sp ")).and_then(|value| value.trim().parse::<u16>().ok()).unwrap_or_default();
    let reached = said.lines().any(|line| line.trim() == "reached 1");
    let memory: Vec<u8> = said.lines().find_map(|line| line.strip_prefix("memory ")).map(|rest| rest.split_whitespace().filter_map(|value| value.parse::<u8>().ok()).collect()).unwrap_or_default();
    let low_sp = said.lines().find_map(|line| line.strip_prefix("low_sp ")).and_then(|value| value.trim().parse::<u32>().ok()).filter(|value| *value < 0xffff).map(|value| value as u16).unwrap_or_default();
    let settings: Vec<(u32, u8)> = said
        .lines()
        .find_map(|line| line.strip_prefix("settings "))
        .map(|rest| rest.split_whitespace().filter_map(|pair| pair.split_once(':')).filter_map(|(at, value)| Some((at.parse::<u32>().ok()?, value.parse::<u8>().ok()?))).filter(|(at, _)| SETTINGS.contains(at)).collect())
        .unwrap_or_default();
    Ok((found, StartState { reached, registers: if registers.len() == 32 { registers } else { Vec::new() }, sp, low_sp, memory, settings }))
}

/// The flash the translation gives its code to read is the image, byte for
/// byte: checked at every translation, since a proof over another flash
/// would be a proof of another binary.
fn same_flash(image: &Image, lifted: &str) -> Result<(), String> {
    let wrong = || "the flash in the translation is not the image that was given: the translation is not used".to_string();
    let (_, after) = lifted.split_once("static const uint8_t arclang_flash[ARCLANG_FLASH_SIZE] = {").ok_or_else(wrong)?;
    let (bytes, _) = after.split_once("};").ok_or_else(wrong)?;
    let flash: Vec<u8> = bytes.split(',').map(str::trim).filter(|byte| !byte.is_empty()).map(|byte| u8::from_str_radix(byte.trim_start_matches("0x"), 16)).collect::<Result<_, _>>().map_err(|_| wrong())?;
    // An image of an odd length ends in the middle of a word: the translation gives the whole word.
    if flash.len() < image.bytes.len() || flash.len() > image.bytes.len() + 1 || flash[..image.bytes.len()] != image.bytes[..] {
        return Err(wrong());
    }
    Ok(())
}

/// The translation is of the processor and of the chip the proof was written for, and
/// uses no operation of the chip the C beneath it does not know.
pub fn expected(lifted: &str, what: &str) -> Result<(), String> {
    for expected in EXPECTED {
        if !lifted.lines().any(|line| line == expected) {
            return Err(format!("{what}: the translation does not say `{}`: Ghidra's description of the processor is not the one the proof was written for", expected.trim_start_matches("//@ ")));
        }
    }
    for line in lifted.lines() {
        if let Some(operation) = line.strip_prefix("//@ operation ") {
            if !OPERATIONS.contains(&operation) {
                return Err(format!("{what}: the code uses an operation of the chip the proof does not know ({operation})"));
            }
        }
    }
    Ok(())
}

/// The C the proof takes for the sketch: the translation, the functions the
/// correspondence names stood in for, and `setup()` and `loop()`.
pub fn sketch(binary: &Binary, lifted: &str, what: &str) -> Result<String, String> {
    expected(lifted, what)?;
    let start = lifted
        .lines()
        .find_map(|line| line.strip_prefix("//@ start 0x"))
        .and_then(|at| u32::from_str_radix(at, 16).ok())
        .ok_or_else(|| format!("{what}: the translation does not say where the code starts"))?;
    let has = |address: u32| lifted.contains(&format!("\nvoid f_{address:04x}(void);\n"));
    for (name, address) in &binary.functions {
        if !has(*address) {
            return Err(format!(
                "{what}: no function was found at 0x{address:04x}, where {name} is said to be"
            ));
        }
    }
    let loop_at = binary
        .functions
        .iter()
        .find(|(name, _)| name == "loop")
        .map(|(_, at)| *at)
        .ok_or_else(|| format!("{what}: it does not say where loop is"))?;

    let mut c = String::from("/* The machine code of the binary, as the proof reads it (written by `arclang prove`). */\n");
    c.push_str(&format!(
        "static void arclang_loop_called(void);\n#define c_{loop_at:04x} arclang_loop_called\n"
    ));
    for (name, address) in binary.functions.iter().filter(|(name, _)| name != "loop") {
        c.push_str(&format!(
            "static void arclang_shim_{name}(void);\n#define c_{address:04x} arclang_shim_{name}\n"
        ));
    }
    c.push_str("\n/* ---- the machine code, one instruction at a time ---- */\n");
    c.push_str(lifted);
    c.push_str("\n/* ---- the functions the correspondence names, as the compiler of the board calls them ---- */\n");
    c.push_str("static bool arclang_lost; /* the code did not reach loop() from its reset */\n");
    // What a function may change beside its result (r0, r18 to r27, r30, r31 and the flags) is anything after it;
    // its return address, pushed by the call, is taken off the stack as its `ret` would.
    c.push_str("static void arclang_returned(void)\n{\n    static const unsigned char changed[] = { 0, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 30, 31 };\n    for (unsigned k = 0; k < sizeof changed; k++) { arclang_r[changed[k]] = arclang_any(); }\n    for (unsigned flag = 0x80u; flag <= 0x86u; flag++) { arclang_reg[flag] = arclang_any() & 1u; }\n    arclang_reg_wr(ARCLANG_SP, 2, arclang_reg_rd(ARCLANG_SP, 2) + 2u);\n}\n");
    for (name, _) in binary.functions.iter().filter(|(name, _)| name != "loop") {
        let known = KNOWN
            .iter()
            .find(|known| known.name == name)
            .ok_or_else(|| {
                format!("{what}: '{name}' is not a function a correspondence can place")
            })?;
        // avr-gcc: the arguments go down from r25, each on an even number of registers.
        let mut next = 26usize;
        let mut arguments = Vec::new();
        for (rank, bytes) in known.arguments.iter().enumerate() {
            next -= bytes + bytes % 2;
            let read = format!("arclang_mem_rd({next}u, {bytes})");
            arguments.push(
                match (*bytes, known.signed && rank + 1 == known.arguments.len()) {
                    (4, _) => format!("(unsigned long){read}"),
                    (2, true) => format!("(int)arclang_signed({read}, 2)"),
                    (2, false) => format!("(unsigned int){read}"),
                    _ => format!("(int){read}"),
                },
            );
        }
        let call = format!("{name}({})", arguments.join(", "));
        c.push_str(&format!("static void arclang_shim_{name}(void)\n{{\n"));
        match known.returns {
            0 => c.push_str(&format!("    {call};\n    arclang_returned();\n")),
            4 => c.push_str(&format!("    unsigned long result = {call};\n    arclang_returned();\n    arclang_mem_wr(22u, 4, (uint64_t)result);\n")),
            _ => c.push_str(&format!("    int result = {call};\n    arclang_returned();\n    arclang_mem_wr(24u, 2, (uint64_t)(unsigned int)result);\n")),
        }
        c.push_str("}\n");
    }
    c.push_str(&format!(
        "\n/* ---- setup() is the code from its reset to its first loop(); loop() is one turn ---- */\nstatic void arclang_loop_called(void)\n{{\n    if (arclang_booting) {{ arclang_stop = true; return; }}\n    f_{loop_at:04x}();\n}}\nvoid setup(void)\n{{\n    arclang_booting = true;\n    arclang_stop = false;\n    f_{start:04x}();\n    arclang_lost = !arclang_stop;\n    arclang_booting = false;\n    arclang_stop = false;\n    /* the call of loop() that stopped the start is the one loop() makes again */\n    arclang_reg_wr(ARCLANG_SP, 2, arclang_reg_rd(ARCLANG_SP, 2) + 2u);\n}}\nvoid loop(void)\n{{\n    arclang_reg_wr(ARCLANG_SP, 2, arclang_reg_rd(ARCLANG_SP, 2) - 2u);\n    f_{loop_at:04x}();\n}}\n"
    ));
    Ok(c)
}

/// What a proof over a binary holds besides the lines of its correspondence:
/// (the condition, as it is written).
pub fn lines() -> Vec<(String, String)> {
    vec![
        ("!arclang_uncovered".to_string(), "the machine code runs only instructions the translation covers (no computed jump)".to_string()),
        ("!arclang_outside".to_string(), "the machine code stays within the memory and the flash of the chip".to_string()),
        ("!arclang_wrote_register".to_string(), "the machine code writes no port of the chip itself, and no other register once started: its pins go through the functions the correspondence names".to_string()),
        ("(!arclang_lost && !arclang_booting && !arclang_stop)".to_string(), "the machine code reaches loop() from its reset".to_string()),
        ("(arclang_r[1] == 0 && arclang_reg_rd(ARCLANG_SP, 2) >= 0x8c0u && arclang_reg_rd(ARCLANG_SP, 2) <= 0x8ffu)".to_string(), "between two loop()s, r1 is zero and the stack is within the last 64 bytes of the memory".to_string()),
    ]
}

/// What a proof over a binary is of, what it proves of the functions it
/// does not run, and what it takes on trust, as it is printed under it.
pub fn trusted(binary: &Binary) -> Vec<String> {
    let mut said = vec![format!("of the machine code of {} itself, translated one instruction at a time (ATmega328P, Ghidra's description of the AVR): no source is read", binary.file)];
    if !binary.identified.is_empty() {
        said.push("proven of the machine code of the functions it does not run, from any memory and registers, on the pins of an Arduino UNO:".to_string());
        said.extend(binary.identified.iter().map(|(_, proven)| format!("  {proven}")));
    }
    if binary.cycle.is_some() {
        said.push(format!("the whole of it is run, from its reset: no function is stood in for. The pins are bits of the registers of the chip (an Arduino UNO); the time is what the interrupt of timer 0 keeps, and {} us pass each time the code has read the timer{}", binary.tick_us.max(4), if binary.tick_us.max(4) > 4 { " (a coarser step than the timer's own 4 us: a wait is read at that step, and what the code does between two reads closer than that is not looked at)" } else { "" }));
    }
    let on_trust: Vec<String> = binary.functions.iter().filter(|(name, _)| name != "loop" && !binary.identified.iter().any(|(proven, _)| proven == name)).map(|(name, at)| format!("{name} at 0x{at:04x}")).collect();
    if !on_trust.is_empty() {
        said.push(format!("taken on trust: these are the Arduino functions they are said to be, and are not run ({})", on_trust.join(", ")));
    }
    if !binary.clock.is_empty() {
        said.push("proven of the clock of the binary:".to_string());
        said.extend(binary.clock.iter().map(|proven| format!("  {proven}")));
        said.push("taken on trust: the clock of the board is 16 MHz, of which timer 0 counts the cycles".to_string());
    } else if binary.clock_unknown.is_none() && binary.identified.iter().any(|(name, _)| name == "millis" || name == "micros") {
        said.push("taken on trust: the bytes millis and micros read count the time (the interrupt of timer 0 keeps them)".to_string());
    }
    if binary.cycle.is_some() && !binary.pwm.is_empty() {
        let pins = binary.pwm.iter().map(u8::to_string).collect::<Vec<_>>().join(", ");
        let (subject, its) = if binary.pwm.len() == 1 { (format!("pin {pins} is"), "it") } else { (format!("the pins {pins} are"), "them") };
        said.push(format!("taken on trust: {subject} driven by the timer when it is connected to {its}, by the level of the pin otherwise; the start leaves the timer counting in 8 bits (proven: a line), and a value of 0 to 255 in its compare register is that duty"));
    }
    if let Some(uart) = binary.uart {
        said.push(format!("taken on trust: the serial port is not looked at, what the code prints included; its transmit buffer is empty at the head of each turn (its interrupt, at 0x{uart:04x}, has emptied it: it is run to do so, and proven to empty it within 64 bytes and to return); a byte received changes nothing the code reads"));
    }
    if let Some(why) = &binary.clock_unknown {
        said.push(format!("NOT established: the clock of the binary ({why}). {}", if binary.cycle.is_some() { "The bytes the interrupt of timer 0 keeps are then taken as any value in every turn, and no time is proven." } else { "That the bytes millis and micros read count the time is taken on trust." }));
    }
    if !binary.interrupts_settled {
        said.push("taken on trust: an interrupt changes nothing the code of loop() reads (the clock is millis() and micros(), as named)".to_string());
    }
    said
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_intel_hex_file_is_its_bytes_from_the_lowest_address() {
        let image = intel_hex(":040000000C946100FB\n:020000040000FA\n:00000001FF\n").unwrap();
        assert_eq!(
            image,
            Image {
                base: 0,
                bytes: vec![0x0C, 0x94, 0x61, 0x00]
            }
        );
        let gap = intel_hex(":0100000011EE\n:0100030022DA\n:00000001FF\n").unwrap();
        assert_eq!(gap.bytes, [0x11, 0xFF, 0xFF, 0x22]);
        assert!(intel_hex(":040000000C946100FC\n")
            .unwrap_err()
            .contains("checksum"));
        assert!(intel_hex("hello\n")
            .unwrap_err()
            .contains("starts with ':'"));
        assert!(intel_hex(":00000001FF\n").unwrap_err().contains("no byte"));
    }

    #[test]
    fn what_is_proven_of_a_function_is_no_longer_said_to_be_taken_on_trust() {
        let mut binary = Binary { file: "a.hex".to_string(), functions: vec![("loop".to_string(), 0x102), ("millis".to_string(), 0x29c), ("delay".to_string(), 0x2fe)], ..Default::default() };
        let said = trusted(&binary).join("\n");
        assert!(said.contains("are not run (millis at 0x029c, delay at 0x02fe)") && !said.contains("proven of the machine code of the functions"), "{said}");
        binary.identified.push(("millis".to_string(), "millis at 0x029c returns the four bytes of the memory at 0x010b and changes nothing".to_string()));
        let said = trusted(&binary).join("\n");
        assert!(said.contains("  millis at 0x029c returns the four bytes") && said.contains("are not run (delay at 0x02fe)") && said.contains("the bytes millis and micros read count the time"), "{said}");
        // A clock that could not be established is said, never left out of the verdict.
        binary.clock_unknown = Some("the interrupt of timer 0: CBMC could not translate the code".to_string());
        let said = trusted(&binary).join("\n");
        assert!(said.contains("NOT established: the clock of the binary (the interrupt of timer 0: CBMC could not translate the code)") && !said.contains("taken on trust: the bytes millis"), "{said}");
        binary.cycle = Some(0x390);
        let said = trusted(&binary).join("\n");
        assert!(said.contains("taken as any value in every turn, and no time is proven"), "{said}");
    }

    #[test]
    fn the_interrupt_vectors_are_read_at_the_start_of_the_image() {
        let mut bytes = Vec::new();
        for vector in 0..26u32 {
            let to = if vector == 16 { 0x208u32 } else { 0xe6 } / 2;
            bytes.extend([0x0C, 0x94, to as u8, (to >> 8) as u8]);
        }
        let read = vectors(&Image { base: 0, bytes: bytes.clone() }).unwrap();
        assert_eq!((read[0], read[TIMER0_OVERFLOW], read.len()), (0xe6, 0x208, 26));
        bytes[5] = 0x95;
        assert_eq!(vectors(&Image { base: 0, bytes }), None);
    }

    #[test]
    fn a_translation_whose_flash_is_not_the_image_is_not_used() {
        let image = Image { base: 0, bytes: vec![0x0C, 0x94, 0x61] };
        let lifted = |bytes: &str| format!("#define ARCLANG_FLASH_SIZE 4u\nstatic const uint8_t arclang_flash[ARCLANG_FLASH_SIZE] = {{\n    {bytes}\n}};\n");
        assert!(same_flash(&image, &lifted("0x0c, 0x94, 0x61, 0xff,")).is_ok());
        assert!(same_flash(&image, &lifted("0x0c, 0x61, 0x94, 0xff,")).unwrap_err().contains("is not the image"));
        assert!(same_flash(&image, &lifted("0x0c, 0x94,")).unwrap_err().contains("is not the image"));
        assert!(same_flash(&image, "void f(void);").unwrap_err().contains("is not the image"));
    }

    #[test]
    fn the_functions_of_a_binary_are_placed_by_address() {
        let lines = |text: &[&str]| {
            functions(
                &text.iter().map(|line| line.to_string()).collect::<Vec<_>>(),
                "correspondence 'B'",
            )
        };
        assert_eq!(
            lines(&["loop at 0x0102", "millis at 0x029C"]).unwrap(),
            [("loop".to_string(), 0x102), ("millis".to_string(), 0x29c)]
        );
        assert!(lines(&["millis at 0x029c"])
            .unwrap_err()
            .contains("where loop is"));
        assert!(lines(&["loop at 0x0102", "tone at 0x0200"])
            .unwrap_err()
            .contains("not a function a correspondence can place"));
        assert!(lines(&["loop at 0x0103"])
            .unwrap_err()
            .contains("even address"));
        assert!(lines(&["loop 0x0102"])
            .unwrap_err()
            .contains("does not say where a function is"));
    }

    #[test]
    fn a_named_function_is_called_as_the_compiler_of_the_board_calls_it() {
        let lifted = "//@ language avr8:LE:16:default\n//@ memory 0x100 0x8ff\n//@ start 0x0\n//@ register SP register 0x3d 2\n//@ register R24 mem 0x18 1\n//@ register Cflg register 0x80 1\n//@ operation watchdog_reset\n\nvoid f_0000(void);\nvoid f_0102(void);\nvoid f_04b6(void);\nvoid f_02fe(void);\nvoid f_0200(void);\n";
        let binary = Binary {
            file: "a.hex".to_string(),
            functions: vec![
                ("loop".to_string(), 0x102),
                ("digitalWrite".to_string(), 0x4b6),
                ("delay".to_string(), 0x2fe),
                ("analogWrite".to_string(), 0x200),
            ],
            ..Default::default()
        };
        let c = sketch(&binary, lifted, "correspondence 'B'").unwrap();
        assert!(c.contains("#define c_0102 arclang_loop_called"), "{c}");
        assert!(
            c.contains("digitalWrite((int)arclang_mem_rd(24u, 1), (int)arclang_mem_rd(22u, 1));"),
            "{c}"
        );
        assert!(
            c.contains("delay((unsigned long)arclang_mem_rd(22u, 4));"),
            "{c}"
        );
        assert!(c.contains("analogWrite((int)arclang_mem_rd(24u, 1), (int)arclang_signed(arclang_mem_rd(22u, 2), 2));"), "{c}");
        assert!(
            c.contains("f_0000();\n    arclang_lost = !arclang_stop;"),
            "{c}"
        );
        let nowhere = Binary {
            file: "a.hex".to_string(),
            functions: vec![("loop".to_string(), 0x300)],
            ..Default::default()
        };
        assert!(sketch(&nowhere, lifted, "correspondence 'B'")
            .unwrap_err()
            .contains("no function was found at 0x0300"));
        assert!(sketch(
            &binary,
            &lifted.replace("watchdog_reset", "encrypt"),
            "correspondence 'B'"
        )
        .unwrap_err()
        .contains("an operation of the chip the proof does not know (encrypt)"));
        assert!(sketch(
            &binary,
            &lifted.replace("0x3d 2", "0x5d 2"),
            "correspondence 'B'"
        )
        .unwrap_err()
        .contains("not the one the proof was written for"));
    }
}
