//! Where the functions of a firmware binary are, found by the shape of
//! their machine code (`arclang recover firmware.hex`).
//!
//! The translation of the binary (`Lift.java`) says of each function what
//! it calls, with the constants its caller loads in the argument registers,
//! where it loops, which registers of the chip and which bytes of the
//! memory it reads and writes by address, and how much it reads through a
//! pointer or from the flash. From that:
//!
//! - `main` is the first function, from the reset, that calls in a cycle
//!   and calls nothing after it;
//!   `loop` is what it calls there, `setup` what it calls just before;
//! - an Arduino function is known by what only it does on this chip:
//!   `millis` copies four bytes of the memory with interrupts masked and
//!   touches nothing else, `micros` also reads the counter of timer 0,
//!   `digitalRead` reads the pin tables of the flash and one port through a
//!   pointer and writes nothing, and so on.
//!
//! It is a proposal, said with its reasons. `arclang prove` then takes the
//! functions it names on trust: nothing here is a proof.

use std::collections::{BTreeMap, BTreeSet};

/// A call, or a jump that does not come back: where it is, the function
/// called, and the registers known to hold a constant there.
#[derive(Debug, Clone, PartialEq)]
pub struct Site {
    pub at: u32,
    pub to: u32,
    pub registers: BTreeMap<u8, u8>,
}

/// What the translation says of one function.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Function {
    pub sites: Vec<Site>,
    /// The jumps back: (from, to).
    pub backs: Vec<(u32, u32)>,
    /// The reads of the flash, the reads and the writes through a pointer (the stack apart).
    pub flash: u32,
    pub loads: u32,
    pub stores: u32,
    /// It masks the interrupts.
    pub masks: bool,
    /// The addresses of the memory (registers of the chip and RAM) it reads and writes by name.
    pub reads: BTreeSet<u32>,
    pub writes: BTreeSet<u32>,
}

/// The first address of the RAM of an ATmega328P; below are the registers of the chip.
const RAM: u32 = 0x100;
/// Where the translation keeps a copy of the status register: every function that masks interrupts writes it.
const STATUS: u32 = 0x5f;
/// The control registers of the three timers: what turning a PWM off writes.
const TIMER_CONTROLS: [u32; 3] = [0x44, 0x80, 0xb0];
/// The counter of timer 0, which only `micros` reads; the converter's channel, which only `analogRead` sets.
const TIMER0_COUNTER: u32 = 0x46;
const CONVERTER_CHANNEL: u32 = 0x7c;

fn hexadecimal(text: &str) -> Option<u32> {
    u32::from_str_radix(text.strip_prefix("0x")?, 16).ok()
}

/// Read the facts of a translation: where it starts, and its functions by address.
pub fn facts(lifted: &str) -> Result<(u32, BTreeMap<u32, Function>), String> {
    let mut start = None;
    let mut functions: BTreeMap<u32, Function> = BTreeMap::new();
    let mut current = 0u32;
    for line in lifted.lines() {
        let Some(fact) = line.strip_prefix("//@ ") else { continue };
        let words: Vec<&str> = fact.split_whitespace().collect();
        let address = |rank: usize| words.get(rank).and_then(|word| hexadecimal(word)).ok_or_else(|| format!("the translation says `{fact}`, which is not read"));
        match words.first().copied() {
            Some("start") => start = Some(address(1)?),
            Some("function") => {
                current = address(1)?;
                functions.entry(current).or_default();
            }
            Some("site") => {
                let mut registers = BTreeMap::new();
                for word in &words[3.min(words.len())..] {
                    let (register, value) = word.split_once('=').ok_or_else(|| format!("the translation says `{fact}`, which is not read"))?;
                    let register = register.strip_prefix('r').and_then(|r| r.parse::<u8>().ok()).ok_or_else(|| format!("the translation says `{fact}`, which is not read"))?;
                    registers.insert(register, hexadecimal(value).ok_or_else(|| format!("the translation says `{fact}`, which is not read"))? as u8);
                }
                functions.entry(current).or_default().sites.push(Site { at: address(1)?, to: address(2)?, registers });
            }
            Some("back") => functions.entry(current).or_default().backs.push((address(1)?, address(2)?)),
            Some("shape") => {
                let function = functions.entry(address(1)?).or_default();
                for word in &words[2..] {
                    let (name, value) = word.split_once('=').ok_or_else(|| format!("the translation says `{fact}`, which is not read"))?;
                    let value: u32 = value.parse().map_err(|_| format!("the translation says `{fact}`, which is not read"))?;
                    match name {
                        "flash" => function.flash = value,
                        "loads" => function.loads = value,
                        "stores" => function.stores = value,
                        "masks" => function.masks = value != 0,
                        _ => {}
                    }
                }
            }
            Some(kind @ ("reads" | "writes")) => {
                let function = functions.entry(address(1)?).or_default();
                let addresses = words[2..].iter().filter_map(|word| hexadecimal(word));
                if kind == "reads" {
                    function.reads.extend(addresses);
                } else {
                    function.writes.extend(addresses);
                }
            }
            _ => {}
        }
    }
    Ok((start.ok_or("the translation does not say where the code starts")?, functions))
}

/// What is proposed of a binary.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Proposal {
    /// (the function, its address, why it is taken for it)
    pub functions: Vec<(String, u32, String)>,
    pub setup: Option<u32>,
    /// The calls of the named functions made from `setup` and from `loop`, as they would be written.
    pub in_setup: Vec<String>,
    pub in_loop: Vec<String>,
    /// The pins `loop` reads and writes, by number, when its calls give them.
    pub reads: BTreeSet<(String, u8)>,
    pub writes: BTreeSet<(String, u8)>,
    /// What `loop` waits, when it waits once and a constant time: the period of one turn, in ms.
    pub period_ms: Option<u32>,
    /// The bytes of the RAM the code of `loop` reads or writes by address, in runs: (the first, how many).
    pub memory: Vec<(u32, u32)>,
    /// What could not be told, said.
    pub unknown: Vec<String>,
}

/// The Arduino function a function is, by its shape, and why.
fn role(at: u32, function: &Function, functions: &BTreeMap<u32, Function>) -> Option<(&'static str, String)> {
    let is_pwm_off = |to: u32| functions.get(&to).is_some_and(|f| f.sites.is_empty() && f.flash == 0 && TIMER_CONTROLS.iter().all(|register| f.writes.contains(register)));
    let chip_reads: Vec<u32> = function.reads.iter().copied().filter(|a| *a < RAM && *a != STATUS).collect();
    let chip_writes: Vec<u32> = function.writes.iter().copied().filter(|a| *a < RAM && *a != STATUS).collect();
    let ram_reads: Vec<u32> = function.reads.iter().copied().filter(|a| *a >= RAM).collect();
    let ram_writes = function.writes.iter().any(|a| *a >= RAM);
    let four_in_a_row = ram_reads.len() == 4 && ram_reads[3] == ram_reads[0] + 3;
    let _ = at;
    if function.sites.is_empty() && function.masks && four_in_a_row && !ram_writes && chip_writes.is_empty() && function.flash == 0 && function.stores == 0 {
        if chip_reads.is_empty() {
            return Some(("millis", format!("it copies the four bytes at 0x{:04x} with interrupts masked, and touches nothing else", ram_reads[0])));
        }
        if chip_reads.contains(&TIMER0_COUNTER) {
            return Some(("micros", format!("it copies the four bytes at 0x{:04x} with interrupts masked and reads the counter of timer 0", ram_reads[0])));
        }
    }
    if function.flash >= 2 && chip_writes.is_empty() && !ram_writes {
        let turns_pwm_off = function.sites.iter().any(|site| is_pwm_off(site.to));
        let only_that = function.sites.iter().all(|site| is_pwm_off(site.to));
        if turns_pwm_off && only_that && function.loads >= 1 && function.stores == 0 {
            return Some(("digitalRead", "it reads the pin tables of the flash, turns the PWM of the pin off, reads one port through a pointer and writes nothing".to_string()));
        }
        if turns_pwm_off && only_that && function.stores >= 1 {
            return Some(("digitalWrite", "it reads the pin tables of the flash, turns the PWM of the pin off and writes one port through a pointer".to_string()));
        }
        if function.sites.is_empty() && function.stores >= 2 {
            return Some(("pinMode", "it reads the pin tables of the flash and writes the direction and the level of one port through pointers, without touching a PWM".to_string()));
        }
    }
    if chip_writes.contains(&CONVERTER_CHANNEL) && function.sites.is_empty() {
        return Some(("analogRead", "it sets the channel of the converter and reads its result".to_string()));
    }
    None
}

/// Propose where the functions of a binary are, from the facts of its translation.
pub fn propose(start: u32, functions: &BTreeMap<u32, Function>) -> Proposal {
    let mut proposal = Proposal::default();
    // main: from the reset, the first function that calls in a cycle.
    let in_cycle = |function: &Function, site: &Site| function.backs.iter().any(|(from, to)| *to <= site.at && site.at <= *from);
    let mut seen = BTreeSet::new();
    let mut queue = std::collections::VecDeque::from([start]);
    let mut main = None;
    while let Some(at) = queue.pop_front() {
        if !seen.insert(at) {
            continue;
        }
        let Some(function) = functions.get(&at) else { continue };
        // main never leaves its cycle: it calls nothing after it (the start-up code that runs the
        // constructors also calls in a cycle, and then calls main).
        let last_back = function.backs.iter().map(|(from, _)| *from).max().unwrap_or(0);
        if function.sites.iter().any(|site| site.to != start && in_cycle(function, site)) && function.sites.iter().all(|site| site.at <= last_back) {
            main = Some(at);
            break;
        }
        queue.extend(function.sites.iter().map(|site| site.to));
    }
    let Some(main_at) = main else {
        proposal.unknown.push("no function, from the reset, calls another in a cycle: main and its loop() were not found (a binary built with link-time optimisation has loop() folded into main)".to_string());
        return proposal;
    };
    let main = &functions[&main_at];
    let cycle: Vec<&Site> = main.sites.iter().filter(|site| site.to != start && in_cycle(main, site)).collect();
    let loop_at = cycle[0].to;
    proposal.functions.push(("loop".to_string(), loop_at, format!("main (0x{main_at:04x}) calls it in its cycle")));
    if cycle.len() > 1 {
        proposal.unknown.push(format!("main calls {} functions in its cycle; the first is taken for loop() (the others would be the serial port's events)", cycle.len()));
    }
    proposal.setup = main.sites.iter().filter(|site| site.at < cycle[0].at).next_back().map(|site| site.to);

    // The Arduino functions, by their shape; a shape two functions have names neither.
    let mut roles: BTreeMap<&'static str, Vec<(u32, String)>> = BTreeMap::new();
    for (at, function) in functions {
        if let Some((name, why)) = role(*at, function, functions) {
            roles.entry(name).or_default().push((*at, why));
        }
    }
    let mut named: BTreeMap<u32, &'static str> = BTreeMap::new();
    for (name, found) in &roles {
        match found.as_slice() {
            [(at, _)] => {
                named.insert(*at, name);
            }
            several => proposal.unknown.push(format!("{} functions have the shape of {name} ({}): none is named", several.len(), several.iter().map(|(at, _)| format!("0x{at:04x}")).collect::<Vec<_>>().join(", "))),
        }
    }
    // delay waits on micros, in a cycle; analogWrite sets a pin as an output and may write its level.
    let of = |name: &str| named.iter().find(|(_, n)| **n == name).map(|(at, _)| *at);
    let mut later: Vec<(u32, &'static str, String)> = Vec::new();
    for (at, function) in functions {
        if named.contains_key(at) {
            continue;
        }
        if let Some(micros) = of("micros") {
            if !function.backs.is_empty() && function.sites.iter().any(|site| site.to == micros) && function.reads.iter().all(|a| *a < RAM) && !function.writes.iter().any(|a| *a >= RAM) {
                later.push((*at, "delay", format!("it calls micros (0x{micros:04x}) in a cycle and keeps nothing in the memory")));
            }
        }
        if let (Some(pin_mode), Some(digital_write)) = (of("pinMode"), of("digitalWrite")) {
            if function.sites.iter().any(|site| site.to == pin_mode) && function.sites.iter().any(|site| site.to == digital_write) && function.writes.iter().any(|a| *a < RAM && *a != STATUS) {
                later.push((*at, "analogWrite", "it sets the pin as an output, writes the compare register of a timer, and writes the pin itself for 0 and 255".to_string()));
            }
        }
    }
    for name in ["delay", "analogWrite"] {
        let found: Vec<&(u32, &'static str, String)> = later.iter().filter(|(_, n, _)| *n == name).collect();
        match found.as_slice() {
            [] => {}
            [(at, _, why)] => {
                named.insert(*at, name);
                roles.entry(name).or_default().push((*at, why.clone()));
            }
            several => proposal.unknown.push(format!("{} functions have the shape of {name}: none is named", several.len())),
        }
    }

    // What setup and loop call, through the functions that are not named.
    let reach = |from: u32| -> (Vec<Site>, BTreeSet<u32>) {
        let (mut sites, mut own, mut queue) = (Vec::new(), BTreeSet::new(), vec![from]);
        while let Some(at) = queue.pop() {
            if named.contains_key(&at) || !own.insert(at) {
                continue;
            }
            for site in functions.get(&at).map(|f| f.sites.as_slice()).unwrap_or_default() {
                if named.contains_key(&site.to) {
                    sites.push(site.clone());
                } else if site.to != start {
                    queue.push(site.to);
                }
            }
        }
        sites.sort_by_key(|site| site.at);
        (sites, own)
    };
    let written = |site: &Site| -> String {
        let name = named[&site.to];
        let byte = |register: u8| site.registers.get(&register).map(|value| value.to_string()).unwrap_or_else(|| "?".to_string());
        match name {
            "pinMode" | "digitalWrite" => format!("{name}({}, {})", byte(24), byte(22)),
            "digitalRead" | "analogRead" => format!("{name}({})", byte(24)),
            "analogWrite" => format!("{name}({}, ?)", byte(24)),
            "delay" => match (22..=25).map(|r| site.registers.get(&r).copied()).collect::<Option<Vec<u8>>>() {
                Some(bytes) => format!("delay({})", u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])),
                None => "delay(?)".to_string(),
            },
            _ => format!("{name}()"),
        }
    };
    let (loop_sites, loop_code) = reach(loop_at);
    let setup_sites = proposal.setup.map(|at| reach(at).0).unwrap_or_default();
    let used: BTreeSet<u32> = loop_sites.iter().chain(&setup_sites).map(|site| site.to).collect();
    for (name, found) in &roles {
        for (at, why) in found {
            if named.get(at) == Some(name) && used.contains(at) {
                proposal.functions.push((name.to_string(), *at, why.clone()));
            }
        }
    }
    proposal.functions[1..].sort_by_key(|(_, at, _)| *at);
    proposal.in_setup = setup_sites.iter().map(&written).collect();
    proposal.in_loop = loop_sites.iter().map(&written).collect();
    for site in &loop_sites {
        let (name, pin) = (named[&site.to], site.registers.get(&24).copied());
        match (name, pin) {
            ("digitalRead" | "analogRead", Some(pin)) => {
                proposal.reads.insert((name.to_string(), pin));
            }
            ("digitalWrite" | "analogWrite", Some(pin)) => {
                proposal.writes.insert((name.to_string(), pin));
            }
            ("digitalRead" | "analogRead" | "digitalWrite" | "analogWrite", None) => proposal.unknown.push(format!("the pin of the {name} at 0x{:04x} is not a constant loaded just before the call", site.at)),
            _ => {}
        }
    }
    let delays: Vec<&Site> = loop_sites.iter().filter(|site| named[&site.to] == "delay").collect();
    if let [delay] = delays.as_slice() {
        if let Some(bytes) = (22..=25).map(|r| delay.registers.get(&r).copied()).collect::<Option<Vec<u8>>>() {
            proposal.period_ms = Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]));
        }
    }
    if proposal.period_ms.is_none() {
        proposal.unknown.push("loop() does not wait once a constant time: its period is to be said (period: ... ms)".to_string());
    }
    let bytes: BTreeSet<u32> = loop_code.iter().filter_map(|at| functions.get(at)).flat_map(|f| f.reads.iter().chain(&f.writes)).copied().filter(|a| *a >= RAM).collect();
    for at in bytes {
        match proposal.memory.last_mut() {
            Some((first, count)) if *first + *count == at => *count += 1,
            _ => proposal.memory.push((at, 1)),
        }
    }
    if loop_code.iter().filter_map(|at| functions.get(at)).any(|f| f.loads + f.stores > 0) {
        proposal.unknown.push("the code of loop() also reads or writes the memory through pointers: bytes it uses that way are not listed".to_string());
    }
    proposal
}

/// The proposal as a file a person or an AI completes: the correspondence
/// with what is known, the machine left to write.
pub fn written(file: &str, bytes: usize, proposal: &Proposal) -> String {
    let stem = std::path::Path::new(file).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "firmware".to_string());
    let mut text = format!("// Recovered from {file} ({bytes} bytes, read as an ATmega328P) by `arclang recover`.\n// A proposal: where its functions are, by the shape of their machine code. `arclang prove`\n// takes the Arduino functions named here on trust; the machine is left to write.\n");
    text.push_str(&format!("model \"{stem}, as flashed\" {{ version: \"1.0.0\" }}\n\n"));
    if !proposal.in_setup.is_empty() {
        text.push_str(&format!("// setup() calls: {}\n", proposal.in_setup.join(", ")));
    }
    if !proposal.in_loop.is_empty() {
        text.push_str(&format!("// loop() calls:  {}\n", proposal.in_loop.join(", ")));
    }
    if !proposal.memory.is_empty() {
        let runs: Vec<String> = proposal.memory.iter().map(|(first, count)| if *count == 1 { format!("0x{first:04x}") } else { format!("0x{first:04x} to 0x{:04x} ({count} bytes)", first + count - 1) }).collect();
        text.push_str(&format!("// loop() keeps its state in the memory at: {}\n//   read it as byte(at), word(at) (two bytes, an int) or dword(at) (four, a long)\n", runs.join(", ")));
    }
    for unknown in &proposal.unknown {
        text.push_str(&format!("// not found: {unknown}\n"));
    }
    text.push_str("\n// state_machine \"...\" { ... }   <- to write: its inputs are the pins read, its variables the pins written\n\n");
    text.push_str(&format!("correspondence \"{stem} binary\" {{\n    machine: \"...\"\n    binary: \"{file}\"\n"));
    if let Some(period) = proposal.period_ms {
        text.push_str(&format!("    period: {period} ms\n"));
    }
    text.push_str("    functions: [\n");
    for (rank, (name, at, why)) in proposal.functions.iter().enumerate() {
        let comma = if rank + 1 < proposal.functions.len() { "," } else { "" };
        text.push_str(&format!("        \"{name} at 0x{at:04x}\"{comma}  // {why}\n"));
    }
    text.push_str("    ]\n");
    let lines = |pins: &BTreeSet<(String, u8)>| pins.iter().map(|(name, pin)| format!("\"{name}({pin}) is ...\"")).collect::<Vec<_>>().join(", ");
    if !proposal.reads.is_empty() {
        text.push_str(&format!("    reads: [{}]\n", lines(&proposal.reads)));
    }
    if !proposal.writes.is_empty() {
        text.push_str(&format!("    writes: [{}]\n", lines(&proposal.writes)));
    }
    text.push_str("    state: [\"... is in ...\"]\n}\n");
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The facts of a small program: a reset, main, setup, loop, and the Arduino functions loop uses.
    const FACTS: &str = "//@ start 0x0\n//@ function 0x0000 calls 0x0010\n//@ site 0x0000 0x0010\n//@ function 0x0010 calls 0x0100\n//@ back 0x0018 0x0014\n//@ site 0x0016 0x0300\n//@ site 0x001a 0x0100\n//@ function 0x0100 calls\n//@ back 0x0110 0x010a\n//@ site 0x0100 0x0300\n//@ site 0x0104 0x0200\n//@ site 0x010a 0x0220\n//@ site 0x010e 0x0000\n//@ function 0x0200 calls\n//@ site 0x0202 0x0500 r22=0x1 r24=0xd\n//@ function 0x0220 calls\n//@ site 0x0222 0x0600 r24=0x2\n//@ site 0x0228 0x0400\n//@ site 0x0230 0x0700 r22=0x1 r24=0xd\n//@ site 0x0238 0x0800 r22=0x14 r23=0x0 r24=0x0 r25=0x0\n//@ reads 0x0220 0x100 0x101\n//@ writes 0x0220 0x100 0x101 0x104\n//@ function 0x0300 calls\n//@ function 0x0400 calls\n//@ shape 0x0400 flash=0 loads=0 stores=0 masks=1\n//@ reads 0x0400 0x10b 0x10c 0x10d 0x10e\n//@ writes 0x0400 0x5f\n//@ function 0x0450 calls\n//@ shape 0x0450 flash=0 loads=0 stores=0 masks=1\n//@ reads 0x0450 0x35 0x46 0x10f 0x110 0x111 0x112\n//@ function 0x04a0 calls\n//@ reads 0x04a0 0x44 0x80 0xb0\n//@ writes 0x04a0 0x44 0x80 0xb0\n//@ function 0x0500 calls\n//@ shape 0x0500 flash=6 loads=5 stores=4 masks=1\n//@ function 0x0600 calls\n//@ site 0x0610 0x04a0\n//@ shape 0x0600 flash=5 loads=1 stores=0 masks=0\n//@ function 0x0700 calls\n//@ site 0x0710 0x04a0\n//@ shape 0x0700 flash=5 loads=2 stores=1 masks=1\n//@ function 0x0800 calls\n//@ back 0x0820 0x0810\n//@ site 0x0812 0x0450\n";

    #[test]
    fn the_functions_of_a_binary_are_found_by_their_shape() {
        let (start, functions) = facts(FACTS).unwrap();
        let proposal = propose(start, &functions);
        let named: Vec<(&str, u32)> = proposal.functions.iter().map(|(name, at, _)| (name.as_str(), *at)).collect();
        assert_eq!(named, [("loop", 0x220), ("millis", 0x400), ("pinMode", 0x500), ("digitalRead", 0x600), ("digitalWrite", 0x700), ("delay", 0x800)]);
        assert_eq!(proposal.setup, Some(0x200));
        assert_eq!(proposal.in_setup, ["pinMode(13, 1)"]);
        assert_eq!(proposal.in_loop, ["digitalRead(2)", "millis()", "digitalWrite(13, 1)", "delay(20)"]);
        assert_eq!(proposal.period_ms, Some(20));
        assert_eq!(proposal.memory, [(0x100, 2), (0x104, 1)]);
        assert!(proposal.reads.contains(&("digitalRead".to_string(), 2)) && proposal.writes.contains(&("digitalWrite".to_string(), 13)));
        let text = written("lamp.hex", 1388, &proposal);
        assert!(text.contains("\"loop at 0x0220\",  // main (0x0100) calls it in its cycle"), "{text}");
        assert!(text.contains("period: 20 ms") && text.contains("reads: [\"digitalRead(2) is ...\"]") && text.contains("0x0100 to 0x0101 (2 bytes), 0x0104"), "{text}");
    }

    #[test]
    fn what_cannot_be_told_is_said_and_not_guessed() {
        // Two functions with the shape of millis: neither is named.
        let twice = format!("{FACTS}//@ function 0x0900 calls\n//@ shape 0x0900 flash=0 loads=0 stores=0 masks=1\n//@ reads 0x0900 0x120 0x121 0x122 0x123\n");
        let (start, functions) = facts(&twice).unwrap();
        let proposal = propose(start, &functions);
        assert!(!proposal.functions.iter().any(|(name, _, _)| name == "millis"), "{proposal:?}");
        assert!(proposal.unknown.iter().any(|said| said.contains("2 functions have the shape of millis")), "{proposal:?}");
        // No cycle that calls: no main, and it is said.
        let (start, functions) = facts("//@ start 0x0\n//@ function 0x0000 calls\n//@ back 0x0004 0x0002\n").unwrap();
        let proposal = propose(start, &functions);
        assert!(proposal.functions.is_empty() && proposal.unknown[0].contains("main and its loop() were not found"), "{proposal:?}");
    }
}
