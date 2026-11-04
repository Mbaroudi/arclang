//! The hardware/software split: which state machines the model gives to the
//! programmable logic of the system, as Arcadia allocates behavior to
//! physical components, with nothing new in the language but the node's
//! nature.
//!
//! A machine is the behavior of a component (`owner:`); a physical node
//! performs it when it `deploys` that component (or the machine itself, by
//! name or id), or when the component is one of its behavior components.
//! The node is programmable logic when it says so: `kind: "fpga"` (or
//! `"programmable_logic"`). Its `device:` is the FPGA part (default
//! `LFE5U-25F-6BG381C`, a Lattice ECP5 of the ULX3S class), `clock:` the
//! clock of the logic (default 25 MHz) and `board:` the board it is on, for
//! `arclang soc` (default: the board of that part).
//!
//! ```text
//! physical_architecture "Charger hardware" {
//!     node "Charger SoC" { processor: "VexRiscv (LiteX)" }
//!     node "Charger fabric" {
//!         kind: "fpga"
//!         device: "LFE5U-25F-6BG381C"
//!         clock: 25 MHz
//!         behavior_component "Charge controller" { id: "PC-CTRL" }
//!     }
//! }
//! state_machine "Charger" { owner: "PC-CTRL" ... }
//! ```

use super::plan::Plan;
use crate::compiler::ast::{AttributeValue, Model, PhysicalNode};
use crate::compiler::quantity::Dimension;

/// The default part: a Lattice ECP5 25k LUT, 381-ball BGA, speed grade 6.
pub const DEFAULT_DEVICE: &str = "LFE5U-25F-6BG381C";
pub const DEFAULT_CLOCK_HZ: f64 = 25e6;

/// An ECP5 part as nextpnr-ecp5 names it.
#[derive(Debug, Clone, PartialEq)]
pub struct Device {
    pub part: String,
    /// `25k`, `um5g-45k`...: nextpnr-ecp5's `--<size>`.
    pub size: String,
    /// `CABGA381`...
    pub package: String,
    pub speed: u8,
}

/// `LFE5U-25F-6BG381C` -> 25k, CABGA381, speed 6.
pub fn device(part: &str) -> Result<Device, String> {
    let bad = || format!("device '{part}' is no Lattice ECP5 part ArcLang knows (LFE5U-<12|25|45|85>F-<6|7|8><BG256|BG381|BG554|BG756|MG285>C, LFE5UM-..., LFE5UM5G-...)");
    let upper = part.trim().to_uppercase();
    let fields: Vec<&str> = upper.split('-').collect();
    if fields.len() != 3 {
        return Err(bad());
    }
    let family = match fields[0] {
        "LFE5U" => "",
        "LFE5UM" => "um-",
        "LFE5UM5G" => "um5g-",
        _ => return Err(bad()),
    };
    let size = match fields[1] {
        "12F" => "12k",
        "25F" => "25k",
        "45F" => "45k",
        "85F" => "85k",
        _ => return Err(bad()),
    };
    let rest = fields[2];
    let speed: u8 = rest.get(0..1).and_then(|s| s.parse().ok()).filter(|s| (6..=8).contains(s)).ok_or_else(bad)?;
    let code = rest.get(1..).unwrap_or("").trim_end_matches(['C', 'I']);
    let package = match code {
        "BG256" => "CABGA256",
        "BG381" => "CABGA381",
        "BG554" => "CABGA554",
        "BG756" => "CABGA756",
        "MG285" => "CSFBGA285",
        "SG285" => "CSFBGA285",
        _ => return Err(bad()),
    };
    Ok(Device { part: part.trim().to_string(), size: format!("{family}{size}"), package: package.to_string(), speed })
}

impl Allocation {
    /// The node's part, as nextpnr-ecp5 names it.
    pub fn device(&self) -> Result<Device, String> {
        device(&self.part).map_err(|e| format!("node '{}': {e}", self.node))
    }
}

/// A machine the programmable logic computes.
#[derive(Debug, Clone, PartialEq)]
pub struct Allocation {
    /// Its index in the plan.
    pub machine: usize,
    pub node: String,
    /// The part as written (`device:`), read when synthesising (`device()`).
    pub part: String,
    pub clock_hz: f64,
    /// The node's `board:`, when it names one (`ulx3s`, `colorlight_i5`...).
    pub board: Option<String>,
}

/// Whether a node is programmable logic.
pub fn is_logic(node: &PhysicalNode) -> bool {
    node.attributes.get("kind").map(|v| v.display().trim().to_lowercase().replace([' ', '-'], "_")).is_some_and(|k| k == "fpga" || k == "programmable_logic")
}

fn clock(node: &PhysicalNode) -> Result<f64, String> {
    match node.attributes.get("clock") {
        None => Ok(DEFAULT_CLOCK_HZ),
        Some(v) => match v.as_quantity() {
            Some(Ok(q)) if q.dimension() == Dimension::Frequency && q.canonical() > 0.0 => Ok(q.canonical()),
            _ => Err(format!("node '{}': clock is a frequency (clock: 25 MHz), got '{}'", node.name, v.display())),
        },
    }
}

/// The machines allocated to programmable logic, with their node.
pub fn allocations(ast: &Model, plan: &Plan) -> Result<Vec<Allocation>, String> {
    let mut out: Vec<Allocation> = Vec::new();
    for pa in &ast.physical_architecture {
        for node in pa.nodes.iter().filter(|n| is_logic(n)) {
            let part = node.attributes.get("device").map(AttributeValue::display).unwrap_or_else(|| DEFAULT_DEVICE.to_string());
            let clock_hz = clock(node)?;
            let board = node.attributes.get("board").map(|v| v.display().trim().to_string()).filter(|b| !b.is_empty());
            for (index, machine) in ast.state_machines.iter().enumerate() {
                let owner = machine.attributes.get("owner").and_then(|v| v.as_string());
                let names = |r: &str| r == machine.name || (!machine.id.is_empty() && r == machine.id) || owner.is_some_and(|o| o == r);
                let deployed = node.deployments.iter().any(|d| names(&d.component));
                let performed = owner.is_some_and(|o| node.behavior_components.iter().any(|b| b.name == o || b.id == o || b.allocated_component.as_deref() == Some(o)));
                if !(deployed || performed) {
                    continue;
                }
                if let Some(other) = out.iter().find(|a| a.machine == index) {
                    return Err(format!("state_machine '{}' is allocated to two programmable logic nodes, '{}' and '{}'", machine.name, other.node, node.name));
                }
                if plan.machines[index].events.len() > 255 {
                    return Err(format!("state_machine '{}' has {} events: the hardware takes at most 255", machine.name, plan.machines[index].events.len()));
                }
                out.push(Allocation { machine: index, node: node.name.clone(), part: part.clone(), clock_hz, board: board.clone() });
            }
        }
    }
    out.sort_by_key(|a| a.machine);
    Ok(out)
}

/// What the compiler checks of the allocations: every allocated machine can
/// be computed by the hardware (fixed point, the constructs it has), at the
/// period it is generated for.
pub fn check(ast: &Model) -> Result<Vec<String>, String> {
    if !ast.physical_architecture.iter().flat_map(|p| &p.nodes).any(is_logic) || ast.state_machines.is_empty() {
        return Ok(Vec::new());
    }
    let plan = super::plan::plan(ast)?;
    let found = allocations(ast, &plan)?;
    let bindings = super::binding::resolve_all(ast, &plan)?;
    let period = super::default_period_us(&bindings);
    for a in &found {
        super::fixed::machine(&plan.machines[a.machine], period).map_err(|e| format!("{e} (it is allocated to programmable logic node '{}': the hardware computes it in fixed point)", a.node))?;
    }
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecp5_parts_are_named_as_nextpnr_names_them() {
        assert_eq!(device("LFE5U-25F-6BG381C").unwrap(), Device { part: "LFE5U-25F-6BG381C".into(), size: "25k".into(), package: "CABGA381".into(), speed: 6 });
        assert_eq!(device("LFE5UM5G-85F-8BG756C").unwrap().size, "um5g-85k");
        assert_eq!(device("LFE5U-12F-6BG256C").unwrap().package, "CABGA256");
        assert!(device("XC7A35T").unwrap_err().contains("no Lattice ECP5 part"));
    }
}
