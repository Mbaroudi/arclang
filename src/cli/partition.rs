//! `arclang partition`: both sides of every machine measured with the real
//! tools, through this same binary — `arclang pil --json` for the C on the
//! processor, `arclang hdl --synth --json --as-if <device>@<MHz>` for the
//! VHDL on the logic (each command relays itself to its Docker image when
//! the host lacks the tools) — then the trade-off per machine, cached beside
//! the model for the table kind `partition`.

use super::CliError;
use crate::compiler::embedded::{alloc, binding, pil, plan};
use crate::compiler::partition::{self, Entry};
use std::path::Path;

pub struct Options {
    pub target: Option<String>,
    pub device: Option<String>,
    pub clock_mhz: Option<f64>,
    pub machine: Option<String>,
    pub dry_run: bool,
    pub json: bool,
}

/// `arclang <args>` run as a subprocess, its stdout parsed as JSON; the
/// reason when it ran nothing (its stderr's last lines).
fn run_json(args: &[String]) -> Result<serde_json::Value, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let output = std::process::Command::new(exe).args(args).output().map_err(|e| format!("cannot run arclang {}: {e}", args.join(" ")))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    // The commands print their JSON even when a case fails; nothing when they could not run.
    let start = stdout.find(['[', '{']).ok_or_else(|| {
        let tail: Vec<&str> = stderr.lines().filter(|l| !l.trim().is_empty()).rev().take(3).collect();
        format!("arclang {} ran nothing: {}", args.join(" "), tail.into_iter().rev().collect::<Vec<_>>().join(" "))
    })?;
    serde_json::from_str(&stdout[start..]).map_err(|e| format!("arclang {} printed no JSON: {e}", args.join(" ")))
}

pub fn run(input: &Path, options: Options) -> Result<(), CliError> {
    let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(input).map_err(|e| CliError::Compilation(e.to_string()))?;
    let ast = &result.ast;
    let p = plan::plan(ast).map_err(CliError::Compilation)?;
    if p.machines.is_empty() {
        return Err(CliError::Config("the model has no state_machine to partition".to_string()));
    }
    if let Some(wanted) = &options.machine {
        if !p.machines.iter().any(|m| m.is(wanted)) {
            return Err(CliError::Config(format!("no state_machine '{wanted}' in the model")));
        }
    }
    let bindings = binding::resolve_all(ast, &p).map_err(CliError::Compilation)?;
    let allocations = alloc::allocations(ast, &p).map_err(CliError::Compilation)?;
    // The target: asked, else the bindings', else the default package.
    let target = match &options.target {
        Some(t) => t.clone(),
        None => pil::choose_target(&bindings, None).map(|pkg| pkg.name).unwrap_or_else(|_| "zephyr/litex_vexriscv".to_string()),
    };
    // The device and the clock: asked, else the first logic node's, else the defaults.
    let device = options.device.clone().or_else(|| allocations.first().map(|a| a.part.clone())).unwrap_or_else(|| alloc::DEFAULT_DEVICE.to_string());
    let clock_hz = options.clock_mhz.map(|m| m * 1e6).or_else(|| allocations.first().map(|a| a.clock_hz)).unwrap_or(alloc::DEFAULT_CLOCK_HZ);
    alloc::device(&device).map_err(CliError::Config)?;
    let model = input.display().to_string();
    let mut pil_args = vec!["pil".to_string(), model.clone(), "--json".to_string(), "--target".to_string(), target.clone()];
    let mut hdl_args = vec!["hdl".to_string(), model.clone(), "--synth".to_string(), "--json".to_string(), "--as-if".to_string(), format!("{device}@{}", clock_hz / 1e6)];
    if let Some(m) = &options.machine {
        hdl_args.extend(["--machine".to_string(), m.clone()]);
        // PIL runs every case; the entries keep the machine's own.
        let _ = &mut pil_args;
    }
    if options.dry_run {
        if options.json {
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "runs": [format!("arclang {}", pil_args.join(" ")), format!("arclang {}", hdl_args.join(" "))], "target": target, "device": device, "clock_mhz": clock_hz / 1e6 })).unwrap_or_default());
        } else {
            println!("Partition {model} — nothing runs (--dry-run); would run:");
            println!("  arclang {}", pil_args.join(" "));
            println!("  arclang {}", hdl_args.join(" "));
        }
        return Ok(());
    }
    if !options.json {
        println!("Partition {model} — C on {target} against VHDL on {device} at {} MHz", clock_hz / 1e6);
        println!("→ arclang {}", pil_args.join(" "));
    }
    let pil_json = run_json(&pil_args);
    if !options.json {
        println!("→ arclang {}", hdl_args.join(" "));
    }
    let hdl_json = run_json(&hdl_args);
    let entries: Vec<Entry> = partition::from_runs(ast, &p, &bindings, pil_json.as_ref().map_err(|e| e.clone()), hdl_json.as_ref().map_err(|e| e.clone()), &target, &device, clock_hz, options.machine.as_deref()).map_err(CliError::Compilation)?;
    // The cache keeps every machine: the ones measured now replace their entry.
    let mut cached = partition::read(input);
    for e in &entries {
        cached.retain(|c| c.machine != e.machine);
        cached.push(e.clone());
    }
    let path = partition::write(input, &cached).map_err(CliError::Config)?;
    if options.json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "target": target, "device": device, "clock_mhz": clock_hz / 1e6, "machines": entries })).unwrap_or_default());
    } else {
        println!();
        for e in &entries {
            for line in partition::lines(e) {
                println!("  {line}");
            }
        }
        println!("\n  {} machine(s) measured; kept in {} (the table kind `partition` reads it)", entries.len(), path.display());
    }
    let unmeasured = entries.iter().filter(|e| e.software.is_none() && e.logic.is_none()).count();
    if unmeasured == 0 {
        Ok(())
    } else {
        Err(CliError::CiFailed(format!("{unmeasured} of {} machine(s) measured on neither side", entries.len())))
    }
}
