//! HW/SW partitioning (`arclang partition`): the same machine measured on
//! both sides with the real tools — as C on the processor (`arclang pil`:
//! the time a step takes of the target's virtual time, the code's
//! footprint) and as VHDL on the programmable logic (`arclang hdl --synth`:
//! LUTs, flip-flops, DSPs, block RAMs, the Fmax against the clock) — and,
//! per machine, whether each side fits its period, what the model allocates
//! today, and what the measurements say. The measurements are cached beside
//! the model (`.arclang/sim-cache/partition.json`), keyed by the generated C
//! and VHDL, the target and the device; the table kind `partition` reads
//! them. The command says what fits; the choice stays in the model (`node
//! ... deploys`).

use crate::compiler::ast::Model;
use crate::compiler::embedded::{alloc, binding, case, plan::Plan, vhdl};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The machine as C on the processor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Software {
    pub target: String,
    /// The longest a step took, in microseconds of the target's virtual time, over the cases run.
    pub step_us: f64,
    pub cases: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bss: Option<u64>,
    pub tools: String,
}

/// The machine as VHDL on the programmable logic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Logic {
    pub device: String,
    pub clock_mhz: f64,
    pub luts: u64,
    pub ffs: u64,
    pub dsps: u64,
    pub brams: u64,
    pub fmax_mhz: f64,
    pub timing_met: bool,
}

/// One machine, both sides.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    pub machine: String,
    pub ident: String,
    pub period_us: u64,
    /// `software` or `logic`: what the model allocates today.
    pub now: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software: Option<Software>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logic: Option<Logic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logic_error: Option<String>,
    /// What the measurements depend on: the generated C and VHDL, the target, the device, the clock.
    pub key: String,
}

impl Entry {
    /// Whether the C keeps its period on the processor (a step within the period); None when not measured.
    pub fn fits_software(&self) -> Option<bool> {
        self.software.as_ref().map(|s| s.step_us <= self.period_us as f64)
    }

    /// Whether the VHDL meets timing on the device; None when not measured.
    pub fn fits_logic(&self) -> Option<bool> {
        self.logic.as_ref().map(|l| l.timing_met)
    }

    /// The margin of the C: how many times the step fits in the period.
    pub fn margin(&self) -> Option<f64> {
        self.software.as_ref().filter(|s| s.step_us > 0.0).map(|s| self.period_us as f64 / s.step_us)
    }

    /// What the measurements say, in a few words: never a decision.
    pub fn advice(&self) -> String {
        match (self.fits_software(), self.fits_logic()) {
            (Some(true), Some(true)) => format!("fits both; now {}", self.now),
            (Some(true), Some(false)) => format!("fits the processor only (the logic misses timing); now {}", self.now),
            (Some(false), Some(true)) => format!("fits the logic only (a step overruns the period on the processor); now {}", self.now),
            (Some(false), Some(false)) => format!("fits neither as it is; now {}", self.now),
            (Some(true), None) => format!("fits the processor; the logic is not measured; now {}", self.now),
            (Some(false), None) => format!("a step overruns the period on the processor; the logic is not measured; now {}", self.now),
            (None, Some(true)) => format!("fits the logic; the processor is not measured; now {}", self.now),
            (None, Some(false)) => format!("the logic misses timing; the processor is not measured; now {}", self.now),
            (None, None) => format!("not measured; now {}", self.now),
        }
    }
}

/// `.arclang/sim-cache/partition.json` beside the model.
pub fn cache_path(model_path: &Path) -> PathBuf {
    crate::compiler::simulation::cache::dir(model_path).join("partition.json")
}

pub fn read(model_path: &Path) -> Vec<Entry> {
    std::fs::read_to_string(cache_path(model_path)).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

pub fn write(model_path: &Path, entries: &[Entry]) -> Result<PathBuf, String> {
    let path = cache_path(model_path);
    std::fs::create_dir_all(path.parent().expect("a parent")).map_err(|e| format!("cannot make {}: {e}", path.parent().expect("a parent").display()))?;
    std::fs::write(&path, serde_json::to_string_pretty(entries).map_err(|e| e.to_string())?).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

/// What a machine's measurements depend on.
pub fn key(ast: &Model, plan: &Plan, machine: usize, period_us: u64, target: &str, device: &str, clock_hz: f64) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    let mut field = |label: &str, bytes: &[u8]| {
        hash.update(label.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    };
    field("format", b"arclang-partition-1");
    let (files, _) = crate::compiler::embedded::export(ast)?;
    for (name, text) in &files {
        field("c", name.as_bytes());
        field("text", text.as_bytes());
    }
    let m = &plan.machines[machine];
    let fx = crate::compiler::embedded::fixed::machine(m, period_us)?;
    for (name, text) in vhdl::files(plan, m, &fx) {
        field("vhdl", name.as_bytes());
        field("text", text.as_bytes());
    }
    field("period", period_us.to_string().as_bytes());
    field("target", target.as_bytes());
    field("device", device.as_bytes());
    field("clock", format!("{clock_hz}").as_bytes());
    Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// The period a machine runs at: its binding's, else the default step.
pub fn period_of(bindings: &[binding::Binding], machine: usize) -> u64 {
    bindings.iter().find(|b| b.machine == machine).map(|b| b.period_us).unwrap_or(case::DEFAULT_STEP_US)
}

/// The number in `step() took at most 38.2 us ...`.
fn step_us_of(timing: &str) -> Option<f64> {
    timing.strip_prefix("step() took at most ")?.split_whitespace().next()?.parse().ok()
}

/// The entries from the two runs' JSON: `pil` (the outcomes of `arclang pil
/// --json`), `hdl` (`arclang hdl --synth --json`); an absent or failed run
/// leaves its side unmeasured with the reason.
#[allow(clippy::too_many_arguments)]
pub fn from_runs(ast: &Model, plan: &Plan, bindings: &[binding::Binding], pil: Result<&serde_json::Value, String>, hdl: Result<&serde_json::Value, String>, target: &str, device: &str, clock_hz: f64, only: Option<&str>) -> Result<Vec<Entry>, String> {
    let allocations = alloc::allocations(ast, plan)?;
    let mut entries = Vec::new();
    for (index, m) in plan.machines.iter().enumerate() {
        if only.is_some_and(|o| !m.is(o)) {
            continue;
        }
        let period_us = period_of(bindings, index);
        let allocated = allocations.iter().find(|a| a.machine == index);
        let (software, software_error) = match &pil {
            Ok(outcomes) => {
                let mine: Vec<&serde_json::Value> = outcomes.as_array().map(|a| a.iter().filter(|o| o["machine"].as_str() == Some(m.name.as_str()) && o["binding"].is_null()).collect()).unwrap_or_default();
                let steps: Vec<f64> = mine.iter().filter_map(|o| o["timing"].as_str().and_then(step_us_of)).collect();
                if steps.is_empty() {
                    (None, Some(if mine.is_empty() { "no executed case drives this machine through step() (a case without a binding)".to_string() } else { "the PIL run reported no step time".to_string() }))
                } else {
                    let footprint = mine.iter().find_map(|o| o.get("footprint").filter(|f| !f.is_null()));
                    (
                        Some(Software {
                            target: target.to_string(),
                            step_us: steps.iter().copied().fold(0.0, f64::max),
                            cases: steps.len(),
                            text: footprint.and_then(|f| f["text"].as_u64()),
                            data: footprint.and_then(|f| f["data"].as_u64()),
                            bss: footprint.and_then(|f| f["bss"].as_u64()),
                            tools: mine[0]["compiler"].as_str().unwrap_or(target).to_string(),
                        }),
                        None,
                    )
                }
            }
            Err(e) => (None, Some(e.clone())),
        };
        // The block synthesised is the machine's wrapper (`<entity>_wb`).
        let entity = vhdl::entity(plan, m);
        let wrapper = format!("{entity}_wb");
        let (logic, logic_error) = match &hdl {
            Ok(report) => match report["synthesis"].as_array().and_then(|s| s.iter().find(|x| x["block"].as_str().is_some_and(|b| b == entity || b == wrapper))) {
                Some(s) if s.get("error").is_some() => (None, Some(s["error"].as_str().unwrap_or("synthesis failed").to_string())),
                Some(s) => (
                    Some(Logic {
                        // The part as asked (the report names nextpnr's arguments too): the key reads it back.
                        device: device.to_string(),
                        clock_mhz: s["clock_mhz"].as_f64().unwrap_or(clock_hz / 1e6),
                        luts: s["luts"].as_u64().unwrap_or(0),
                        ffs: s["ffs"].as_u64().unwrap_or(0),
                        dsps: s["dsps"].as_u64().unwrap_or(0),
                        brams: s["brams"].as_u64().unwrap_or(0),
                        fmax_mhz: s["fmax_mhz"].as_f64().unwrap_or(0.0),
                        timing_met: s["timing_met"].as_bool().unwrap_or(false),
                    }),
                    None,
                ),
                None => (None, Some(format!("the HDL run synthesised no block '{entity}'"))),
            },
            Err(e) => (None, Some(e.clone())),
        };
        entries.push(Entry {
            machine: m.name.clone(),
            ident: m.ident.clone(),
            period_us,
            now: if allocated.is_some() { "logic".to_string() } else { "software".to_string() },
            node: allocated.map(|a| a.node.clone()),
            software,
            software_error,
            logic,
            logic_error,
            key: key(ast, plan, index, period_us, target, device, clock_hz)?,
        });
    }
    Ok(entries)
}

/// The cached entries that still hold for the model (their key matches), by machine.
pub fn fresh(ast: &Model, model_path: &Path) -> Vec<(Entry, bool)> {
    let Ok(plan) = crate::compiler::embedded::plan::plan(ast) else { return Vec::new() };
    let Ok(bindings) = binding::resolve_all(ast, &plan) else { return Vec::new() };
    read(model_path)
        .into_iter()
        .map(|e| {
            let fresh = plan.machines.iter().position(|m| m.name == e.machine).and_then(|i| {
                let target = e.software.as_ref().map(|s| s.target.clone()).unwrap_or_default();
                let (device, clock) = e.logic.as_ref().map(|l| (l.device.clone(), l.clock_mhz * 1e6)).unwrap_or_default();
                key(ast, &plan, i, period_of(&bindings, i), &target, &device, clock).ok()
            });
            let is_fresh = fresh.as_deref() == Some(e.key.as_str());
            (e, is_fresh)
        })
        .collect()
}

fn kib(bytes: u64) -> String {
    if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

/// An entry on a few lines, as `arclang partition` prints it.
pub fn lines(e: &Entry) -> Vec<String> {
    let mut out = vec![format!("{} — period {} us · {}", e.machine, e.period_us, e.advice())];
    match (&e.software, &e.software_error) {
        (Some(s), _) => {
            let size = match (s.text, s.data, s.bss) {
                (Some(t), Some(d), Some(b)) => format!(", {} text + {} data + {} bss", kib(t), kib(d), kib(b)),
                _ => String::new(),
            };
            out.push(format!("    C on {}: a step takes at most {:.1} us ({}×{}){size}, over {} case(s)", s.target, s.step_us, e.margin().map(|m| format!("{m:.0}")).unwrap_or_else(|| "∞".into()), if e.fits_software() == Some(true) { " of margin" } else { ": OVERRUN" }, s.cases));
        }
        (None, Some(why)) => out.push(format!("    C on the processor: not measured ({why})")),
        _ => {}
    }
    match (&e.logic, &e.logic_error) {
        (Some(l), _) => out.push(format!("    VHDL on {}: {} LUT4, {} FF, {} DSP, {} BRAM; Fmax {:.1} MHz for {} MHz: {}", l.device, l.luts, l.ffs, l.dsps, l.brams, l.fmax_mhz, l.clock_mhz, if l.timing_met { "met" } else { "NOT MET" })),
        (None, Some(why)) => out.push(format!("    VHDL on the logic: not measured ({why})")),
        _ => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(step_us: f64, timing_met: bool) -> Entry {
        Entry {
            machine: "Charger".into(),
            ident: "charger".into(),
            period_us: 10_000,
            now: "logic".into(),
            node: Some("Charger fabric".into()),
            software: Some(Software { target: "zephyr/litex_vexriscv".into(), step_us, cases: 2, text: Some(4096), data: Some(16), bss: Some(256), tools: "gcc".into() }),
            software_error: None,
            logic: Some(Logic { device: "LFE5U-25F-6BG381C".into(), clock_mhz: 25.0, luts: 1200, ffs: 640, dsps: 4, brams: 0, fmax_mhz: 41.0, timing_met }),
            logic_error: None,
            key: "k".into(),
        }
    }

    #[test]
    fn what_fits_is_read_from_the_period_and_the_timing() {
        let both = entry(38.2, true);
        assert_eq!((both.fits_software(), both.fits_logic()), (Some(true), Some(true)));
        assert_eq!(both.margin().map(|m| m.round()), Some(262.0));
        assert_eq!(both.advice(), "fits both; now logic");
        let slow = entry(12_000.0, false);
        assert_eq!(slow.advice(), "fits neither as it is; now logic");
        assert!(lines(&slow)[1].contains("OVERRUN"));
        assert!(lines(&slow)[2].contains("NOT MET"));
        let unmeasured = Entry { software: None, software_error: Some("no renode".into()), ..entry(1.0, true) };
        assert_eq!(unmeasured.fits_software(), None);
        assert!(unmeasured.advice().starts_with("fits the logic; the processor is not measured"));
        assert!(lines(&unmeasured)[1].contains("not measured (no renode)"));
    }

    #[test]
    fn the_step_time_is_read_from_the_pil_timing_text() {
        assert_eq!(step_us_of("step() took at most 38.2 us of the target's virtual time"), Some(38.2));
        assert_eq!(step_us_of("16 cycles, one every 10000 us"), None);
    }
}
