//! The proof that the VHDL as built does what the VHDL the model generates
//! does (`arclang drift --hdl <dir> --prove`).
//!
//! Each side is synthesized by GHDL to a netlist, and Yosys compares the two:
//!
//! - **Proven**, in two parts that together are an induction on the clock.
//!   The step: every signal the two netlists share by name is a point where
//!   they must agree; Yosys proves that points equal for two cycles stay
//!   equal (`equiv_simple`, `equiv_induct`). A point between two anonymous
//!   wires that GHDL happened to number alike is no claim of anyone's: when
//!   it is not proven it is dropped and the proof is run again, whole, on
//!   the points left, until every one of them is proven. A point with a
//!   name of the design (a port, a signal) is never dropped. The base:
//!   from reset, the same points are equal for the first cycles
//!   (SymbiYosys, bounded check).
//! - **Different**: from reset and with the same inputs, an output of the
//!   two differs within the cycles searched; the cycle and the outputs are
//!   given.
//! - Neither, said as such: the time allowed, a design GHDL does not
//!   synthesize, other ports, no reset to start from.
//!
//! What the proof is of: the netlists GHDL synthesizes, from reset
//! (`rst`), for any inputs. The two entities must have the same ports.

use regex::Regex;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum LogicEquivalence {
    /// `points`: the signals proven equal; `dropped`: the anonymous wires
    /// paired by their number that were not, and are no part of the claim.
    Proven { points: usize, dropped: usize, seconds: f64 },
    /// The cycle after reset at which outputs differ, and each: (its name,
    /// its value for the model's logic, its value for the logic as built).
    Different { cycle: usize, outputs: Vec<(String, String, String)>, seconds: f64 },
    Unknown { reason: String },
}

/// The cycles after reset the points are checked equal for: the base of the
/// induction, which assumes two.
const BASE_CYCLES: usize = 8;
/// The cycles after reset a difference is searched in.
const SEARCH_CYCLES: usize = 400;
/// How many times the proof is run again after dropping anonymous points.
const PASSES: usize = 8;
const HOW: &str = "run in ArcLang's formal image: tools/embedded/formal/run.sh";

fn tool(variable: &str, default: &str) -> String {
    std::env::var(variable).ok().filter(|v| !v.trim().is_empty()).unwrap_or_else(|| default.to_string())
}

/// Run a tool in `dir` for at most what is left before `until`: its output
/// and whether it succeeded, or None when the time ran out.
fn run(program: &str, args: &[&str], dir: &Path, until: Instant) -> Result<Option<(bool, String)>, String> {
    let log = dir.join("tool.out");
    let file = std::fs::File::create(&log).map_err(|e| format!("cannot write {}: {e}", log.display()))?;
    let errors = file.try_clone().map_err(|e| e.to_string())?;
    let mut child = Command::new(program).args(args).current_dir(dir).stdin(Stdio::null()).stdout(file).stderr(errors).spawn().map_err(|e| format!("cannot run '{program}': {e} ({HOW})"))?;
    loop {
        match child.try_wait().map_err(|e| format!("'{program}': {e}"))? {
            Some(status) => return Ok(Some((status.success(), std::fs::read_to_string(&log).unwrap_or_default()))),
            None if Instant::now() >= until => {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(None);
            }
            None => std::thread::sleep(Duration::from_millis(50)),
        }
    }
}

/// The ports of an entity as its VHDL declares them, comments and spacing apart.
fn ports(vhdl: &str, entity: &str) -> Option<String> {
    let comment = Regex::new(r"--[^\n]*").expect("a pattern of the ports");
    let text = comment.replace_all(vhdl, "").to_lowercase();
    let start = text.find(&format!("entity {} is", entity.to_lowercase()))?;
    let end = text[start..].find("end entity").or_else(|| text[start..].find(&format!("end {}", entity.to_lowercase())))?;
    Some(text[start..start + end].split_whitespace().collect::<String>())
}

/// The files of one side in the order GHDL analyses them: the package, the
/// blocks, then what instantiates them.
fn ordered(files: &[(String, String)]) -> Vec<&str> {
    let mut names: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).filter(|name| name.ends_with(".vhd")).collect();
    names.sort_by_key(|name| (!name.ends_with("_pkg.vhd"), name.ends_with("_wb.vhd"), name.to_string()));
    names
}

/// Prove, or refute, that the entity as built does what the entity the
/// model generates does. `generated` and `built` are the VHDL files of each
/// side, by name; `work` receives the files of the proof and is kept.
pub fn prove(generated: &[(String, String)], built: &[(String, String)], entity: &str, work: &Path, seconds: u64) -> Result<LogicEquivalence, String> {
    let started = Instant::now();
    let until = started + Duration::from_secs(seconds);
    let unknown = |reason: String| Ok(LogicEquivalence::Unknown { reason });
    let file = format!("{entity}.vhd");
    let text = |files: &[(String, String)]| files.iter().find(|(name, _)| *name == file).map(|(_, text)| text.clone());
    let (Some(model_text), Some(built_text)) = (text(generated), text(built)) else {
        return unknown(format!("{file} is not on both sides"));
    };
    match (ports(&model_text, entity), ports(&built_text, entity)) {
        (Some(model), Some(code)) if model == code => {}
        (Some(_), Some(_)) => return unknown(format!("entity {entity} of the code as built has other ports than the model's: a logic with another interface is not compared")),
        _ => return unknown(format!("{file} declares no entity {entity}")),
    }
    if !Regex::new(r"(?m)^\s*rst\s*:\s*in\s").expect("a pattern of the reset").is_match(&model_text) {
        return unknown(format!("entity {entity} has no input `rst`: there is no reset to compare the two from"));
    }
    let (ghdl, yosys, sby) = (tool("ARCLANG_GHDL", "ghdl"), tool("ARCLANG_YOSYS", "yosys"), tool("ARCLANG_SBY", "sby"));

    // Each side synthesized to a netlist of its own.
    for (side, files) in [("gold", generated), ("gate", built)] {
        let dir = work.join(side);
        std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
        for (name, text) in files.iter().filter(|(name, _)| name.ends_with(".vhd")) {
            std::fs::write(dir.join(name), text).map_err(|e| format!("cannot write {name}: {e}"))?;
        }
        let whose = if side == "gold" { "the model generates" } else { "as built" };
        let mut analyse = vec!["-a", "--std=08"];
        analyse.extend(ordered(files));
        match run(&ghdl, &analyse, &dir, until)? {
            None => return unknown(format!("no answer within {seconds} s")),
            Some((false, said)) => return unknown(format!("GHDL does not accept the VHDL {whose}:\n{}", last_lines(&said, 6))),
            Some((true, _)) => {}
        }
        match run(&ghdl, &["--synth", "--std=08", "--out=verilog", entity], &dir, until)? {
            None => return unknown(format!("no answer within {seconds} s")),
            Some((false, said)) => return unknown(format!("GHDL does not synthesize the VHDL {whose}:\n{}", last_lines(&said, 6))),
            Some((true, netlist)) => std::fs::write(work.join(format!("{side}.v")), netlist).map_err(|e| e.to_string())?,
        }
    }

    // The two netlists side by side, each flattened under its own name.
    let load = format!(
        "read_verilog gold.v\nhierarchy -top {entity}\nproc\nflatten\nopt_clean\nrename {entity} gold\ndesign -stash gold\nread_verilog gate.v\nhierarchy -top {entity}\nproc\nflatten\nopt_clean\nrename {entity} gate\ndesign -stash gate\ndesign -copy-from gold -as gold gold\ndesign -copy-from gate -as gate gate\n"
    );
    let points = "equiv_make gold gate equiv\nhierarchy -top equiv\nflatten\nopt_clean\n";
    let dropping = |dropped: &[String]| dropped.iter().map(|wire| format!("select -set bad w:{wire} %co1 t:$equiv %i\ndelete @bad\n")).collect::<String>();

    // The step, run again whole each time anonymous points are dropped.
    let unproven = Regex::new(r"(?m)Unproven \$equiv \S+ \\(\S+) \\(\S+)").expect("a pattern of the status");
    let counted = Regex::new(r"Of those cells (\d+) are proven and (\d+) are unproven").expect("a pattern of the status");
    let anonymous = Regex::new(r"^n\d+_(gold|gate)$").expect("a pattern of the wires");
    let mut dropped: Vec<String> = Vec::new();
    let mut step: Option<usize> = None;
    let mut why_not = String::new();
    for _ in 0..PASSES {
        let script = format!("{load}{points}{}equiv_struct\nequiv_simple -short\nequiv_induct -seq 2\nequiv_status\n", dropping(&dropped));
        std::fs::write(work.join("step.ys"), &script).map_err(|e| e.to_string())?;
        let said = match run(&yosys, &["-q", "-l", "step.log", "-s", "step.ys"], work, until)? {
            None => {
                why_not = format!("no answer within {seconds} s");
                break;
            }
            Some((false, said)) => return unknown(format!("Yosys could not compare the two netlists:\n{}", last_lines(&said, 6))),
            Some((true, _)) => std::fs::read_to_string(work.join("step.log")).unwrap_or_default(),
        };
        let Some(count) = counted.captures_iter(&said).last() else {
            return unknown("Yosys did not say how many points it proved".to_string());
        };
        let (proven, left): (usize, usize) = (count[1].parse().unwrap_or(0), count[2].parse().unwrap_or(usize::MAX));
        if left == 0 {
            step = Some(proven);
            break;
        }
        let wires: Vec<String> = unproven.captures_iter(&said).flat_map(|found| [found[1].to_string(), found[2].to_string()]).collect();
        if wires.is_empty() || wires.iter().any(|wire| !anonymous.is_match(wire)) {
            why_not = "signals of the design are not proven equal".to_string();
            break;
        }
        let before = dropped.len();
        for wire in wires {
            if !dropped.contains(&wire) {
                dropped.push(wire);
            }
        }
        if dropped.len() == before {
            why_not = "points that could not be dropped stay unproven".to_string();
            break;
        }
    }

    let smtc = |reset: &str| format!("initial\nassume (= [{reset}] true)\n");
    let sby_file = |mode_depth: usize, script: &str, left: u64| format!("[options]\nmode bmc\ndepth {mode_depth}\nskip 1\nsmtc reset.smtc\ntimeout {left}\n[engines]\nsmtbmc bitwuzla\n[script]\n{script}[files]\ngold.v\ngate.v\nreset.smtc\n");
    let left = |until: Instant| until.saturating_duration_since(Instant::now()).as_secs().max(1);

    // The base: from reset, the points proven to stay equal are equal.
    if let Some(points_proven) = step {
        std::fs::write(work.join("reset.smtc"), smtc("rst")).map_err(|e| e.to_string())?;
        let script = format!("{load}{points}{}equiv_miter -assert base\nhierarchy -top base\nequiv_remove -gold\n", dropping(&dropped));
        std::fs::write(work.join("base.sby"), sby_file(BASE_CYCLES, &script, left(until))).map_err(|e| e.to_string())?;
        match run(&sby, &["-f", "base.sby"], work, until + Duration::from_secs(5))? {
            None => why_not = format!("no answer within {seconds} s"),
            Some((_, said)) if said.contains("DONE (PASS") => {
                return Ok(LogicEquivalence::Proven { points: points_proven, dropped: dropped.len(), seconds: started.elapsed().as_secs_f64() });
            }
            Some((_, said)) if said.contains("DONE (FAIL") => why_not = "from reset, signals of the two are not equal".to_string(),
            Some((_, said)) if said.contains("DONE (TIMEOUT") => why_not = format!("no answer within {seconds} s"),
            Some((_, said)) => return unknown(format!("SymbiYosys could not check the start from reset:\n{}", last_lines(&said, 6))),
        }
    }

    // Not proven: is there a run from reset where an output differs?
    if Instant::now() >= until {
        return unknown(why_not);
    }
    std::fs::write(work.join("reset.smtc"), smtc("in_rst")).map_err(|e| e.to_string())?;
    let script = format!("{load}miter -equiv -flatten -make_outputs -make_assert gold gate differ\nhierarchy -top differ\n");
    std::fs::write(work.join("differ.sby"), sby_file(SEARCH_CYCLES, &script, left(until))).map_err(|e| e.to_string())?;
    match run(&sby, &["-f", "differ.sby"], work, until + Duration::from_secs(5))? {
        Some((_, said)) if said.contains("DONE (FAIL") => {
            let cycle = Regex::new(r"Checking assertions in step (\d+)").expect("a pattern of the search").captures_iter(&said).last().and_then(|c| c[1].parse().ok()).unwrap_or(0);
            let trace = std::fs::read_to_string(work.join("differ/engine_0/trace.vcd")).unwrap_or_default();
            Ok(LogicEquivalence::Different { cycle, outputs: differing(&trace), seconds: started.elapsed().as_secs_f64() })
        }
        Some((_, said)) if said.contains("DONE (PASS") => unknown(format!("{why_not}, and no output differs in the first {SEARCH_CYCLES} cycles after reset")),
        Some((_, said)) if said.contains("DONE (TIMEOUT") || said.is_empty() => unknown(format!("{why_not}, and no output was found to differ from reset within {seconds} s")),
        None => unknown(format!("{why_not}, and no output was found to differ from reset within {seconds} s")),
        Some((_, said)) => unknown(format!("{why_not}; SymbiYosys could not search for a difference:\n{}", last_lines(&said, 6))),
    }
}

fn last_lines(text: &str, count: usize) -> String {
    let lines: Vec<&str> = text.lines().filter(|line| !line.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(count)..].join("\n")
}

/// The outputs whose last value in a trace differs between the two sides:
/// (the output, the value for the model's logic, the value for the logic as built).
fn differing(vcd: &str) -> Vec<(String, String, String)> {
    let declared = Regex::new(r"^\$var \w+ \d+ (\S+) (gold|gate)_(\S+?)(?: \[[\d:]+\])? \$end").expect("a pattern of the trace");
    let mut names: std::collections::BTreeMap<String, (String, String)> = std::collections::BTreeMap::new();
    let mut values: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for line in vcd.lines() {
        let line = line.trim();
        if let Some(found) = declared.captures(line) {
            names.insert(found[1].to_string(), (found[2].to_string(), found[3].to_string()));
        } else if let Some(rest) = line.strip_prefix('b') {
            if let Some((value, id)) = rest.split_once(' ') {
                values.insert(id.to_string(), value.to_string());
            }
        } else if line.len() >= 2 && matches!(line.as_bytes()[0], b'0' | b'1' | b'x' | b'z') && !line.starts_with('#') {
            values.insert(line[1..].to_string(), line[..1].to_string());
        }
    }
    let mut sides: std::collections::BTreeMap<String, (Option<String>, Option<String>)> = std::collections::BTreeMap::new();
    for (id, (side, output)) in &names {
        let value = values.get(id).map(|v| v.trim_start_matches('0').to_string()).map(|v| if v.is_empty() { "0".to_string() } else { v });
        let entry = sides.entry(output.clone()).or_default();
        if side == "gold" {
            entry.0 = value;
        } else {
            entry.1 = value;
        }
    }
    sides.into_iter().filter_map(|(output, (model, built))| Some((output, model?, built?))).filter(|(_, model, built)| model != built).map(|(output, model, built)| (output, format!("0b{model}"), format!("0b{built}"))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ports_of_an_entity_are_compared_comments_and_spacing_apart() {
        let one = "entity blk is\n  port (\n    clk : in std_logic; -- the clock\n    q   : out std_logic\n  );\nend entity;\narchitecture rtl of blk is begin end;";
        let same = "entity BLK is port (clk : in std_logic;\n q : out std_logic);\nend entity;";
        let other = "entity blk is port (clk : in std_logic; q : out std_logic; r : out std_logic);\nend entity;";
        assert_eq!(ports(one, "blk"), ports(same, "blk"));
        assert_ne!(ports(one, "blk"), ports(other, "blk"));
        assert_eq!(ports(one, "another"), None);
    }

    #[test]
    fn the_outputs_that_differ_at_the_end_of_a_trace_are_read() {
        let vcd = "$scope module differ $end\n$var wire 1 n1 gold_done $end\n$var wire 1 n2 gate_done $end\n$var wire 8 n3 gold_o_v [7:0] $end\n$var wire 8 n4 gate_o_v [7:0] $end\n$upscope $end\n#0\n0n1\n0n2\nb00000100 n3\nb00000100 n4\n#10\n1n1\n1n2\nb00000010 n4\n";
        assert_eq!(differing(vcd), vec![("o_v".to_string(), "0b100".to_string(), "0b10".to_string())]);
    }

    #[test]
    fn the_package_is_analysed_first_and_what_instantiates_last() {
        let files: Vec<(String, String)> = ["m_blk_wb.vhd", "m_blk.vhd", "arclang_fx_pkg.vhd", "m_blk_regs.json"].iter().map(|name| (name.to_string(), String::new())).collect();
        assert_eq!(ordered(&files), vec!["arclang_fx_pkg.vhd", "m_blk.vhd", "m_blk_wb.vhd"]);
    }
}
