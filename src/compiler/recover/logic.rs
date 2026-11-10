//! The machines computed by programmable logic.
//!
//! The VHDL the compiler generates follows the same machines as the C, in
//! fixed point: its words are sized from the range of each variable, which
//! the C does not hold. The register map written beside the VHDL
//! (`<block>_regs.json`) gives, for each variable, its format, the range it
//! was sized for and where that range comes from. With them the machines
//! recovered from the C say what the logic needs, and the proof is the same
//! as for the C: the recovered model must generate the files of the logic
//! as they were read, every one of them.

use super::drift::{files, FileDrift};
use super::Recovered;
use std::collections::BTreeMap;

/// What the register map says of a variable computed in the logic.
#[derive(Debug, Clone, PartialEq)]
pub struct Sized {
    /// `Q9.22`, `int32`, `boolean`.
    pub format: String,
    pub range: (f64, f64),
    /// `declared`, `set` (by a format), `constant`, `derived`...
    pub origin: String,
    pub bits: u64,
}

/// The machine a register map is of, and what it says of each variable, by name.
pub fn read_map(text: &str, file: &str) -> Result<(String, BTreeMap<String, Sized>), String> {
    let map: serde_json::Value = serde_json::from_str(text).map_err(|error| format!("{file} is not a register map: {error}"))?;
    let machine = map["machine"].as_str().ok_or_else(|| format!("{file}: no `machine`"))?.to_string();
    let name = regex::Regex::new(r#"variable "(.+?)""#).expect("a pattern of the reader");
    let mut variables = BTreeMap::new();
    for register in map["registers"].as_array().map(Vec::as_slice).unwrap_or_default() {
        let (Some(format), Some(found)) = (register["format"].as_str(), register["description"].as_str().and_then(|d| name.captures(d))) else { continue };
        let Some(origin) = register["origin"].as_str() else {
            return Err(format!("{file} was written by an earlier compiler: it does not say the range each variable was sized for, without which the logic cannot be generated again"));
        };
        let end = |at: usize| register["range"][at].as_f64().unwrap_or(f64::NAN);
        variables.insert(found[1].to_string(), Sized { format: format.to_string(), range: (end(0), end(1)), origin: origin.to_string(), bits: register["bits"].as_u64().unwrap_or(32) });
    }
    Ok((machine, variables))
}

/// Say, in the recovered machines, what the register maps hold: the machines
/// that are in the logic, and how each of their variables is sized.
pub fn apply(recovered: &mut Recovered, maps: &[(String, BTreeMap<String, Sized>)]) -> Result<(), String> {
    for (name, sized) in maps {
        let machine = recovered.machines.iter_mut().find(|m| &m.name == name).ok_or_else(|| format!("the logic computes state_machine \"{name}\", which the C read does not hold: the two are not of the same model"))?;
        machine.in_logic = true;
        for variable in &mut machine.variables {
            variable.sized = sized.get(&variable.name).cloned();
        }
        let narrow: Vec<&str> = machine.variables.iter().filter(|v| v.sized.as_ref().is_some_and(|s| s.origin == "declared" && s.bits != 32)).map(|v| v.name.as_str()).collect();
        if !narrow.is_empty() {
            recovered.notes.push(format!("state_machine \"{name}\": the word of {} is narrower than 32 bits, sized by a resolution the files do not hold: its logic will not be proven", narrow.join(", ")));
        }
    }
    recovered.notes.push("not in the files of the logic: the device and the clock of the node, the name of the node and of its component (named here by the machine)".to_string());
    Ok(())
}

/// Compile the recovered source, generate the files of its logic and compare
/// each with the file as read: every file the same is the proof.
pub fn check(source: &str, read: &dyn Fn(&str) -> Option<String>) -> Result<Vec<FileDrift>, String> {
    let compiled = crate::Compiler::new(crate::CompilerConfig::default()).compile_string(source).map_err(|error| format!("the recovered model does not compile: {error}"))?;
    let (generated, _) = crate::compiler::embedded::hdl::export(&compiled.ast).map_err(|error| format!("the recovered model generates no logic: {error}"))?;
    Ok(files(&generated, &[], read))
}
