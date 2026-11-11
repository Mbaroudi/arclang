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
//!   two differs; the cycle and the outputs are given. The run is looked
//!   for twice: before any proof, GHDL simulates the two side by side on a
//!   long run of random inputs, which reaches what is far from reset; when
//!   the proof fails, SymbiYosys searches the first cycles exhaustively. A
//!   run that shows a difference is a refutation whoever found it; a run
//!   that shows none proves nothing, and the proof is still asked.
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
    /// `found`: how the run was found (`simulation`: a run of random inputs
    /// in GHDL; `search`: the shortest run, by SymbiYosys).
    Different { cycle: usize, outputs: Vec<(String, String, String)>, found: String, seconds: f64 },
    Unknown { reason: String },
}

/// The cycles after reset the points are checked equal for: the base of the
/// induction, which assumes two.
const BASE_CYCLES: usize = 8;
/// The cycles after reset a difference is searched in.
const SEARCH_CYCLES: usize = 400;
/// The clocks of the run of random inputs the two are simulated on, and its seed.
const SIMULATED_CYCLES: usize = 200_000;
const SEED: usize = 1;
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

    // First, the two simulated side by side on a long run of random inputs:
    // a difference found there is found in seconds, and settles the question.
    if let Some((cycle, outputs)) = simulate(&ghdl, generated, built, entity, &model_text, work, until)? {
        return Ok(LogicEquivalence::Different { cycle, outputs, found: "simulation".to_string(), seconds: started.elapsed().as_secs_f64() });
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
    if Instant::now() >= until {
        return unknown(format!("{why_not}, and no output differs on a run of {SIMULATED_CYCLES} clocks of random inputs"));
    }
    std::fs::write(work.join("reset.smtc"), smtc("in_rst")).map_err(|e| e.to_string())?;
    let script = format!("{load}miter -equiv -flatten -make_outputs -make_assert gold gate differ\nhierarchy -top differ\n");
    std::fs::write(work.join("differ.sby"), sby_file(SEARCH_CYCLES, &script, left(until))).map_err(|e| e.to_string())?;
    match run(&sby, &["-f", "differ.sby"], work, until + Duration::from_secs(5))? {
        Some((_, said)) if said.contains("DONE (FAIL") => {
            let cycle = Regex::new(r"Checking assertions in step (\d+)").expect("a pattern of the search").captures_iter(&said).last().and_then(|c| c[1].parse().ok()).unwrap_or(0);
            let trace = std::fs::read_to_string(work.join("differ/engine_0/trace.vcd")).unwrap_or_default();
            Ok(LogicEquivalence::Different { cycle, outputs: differing(&trace), found: "search".to_string(), seconds: started.elapsed().as_secs_f64() })
        }
        Some((_, said)) if said.contains("DONE (PASS") => unknown(format!("{why_not}, and no output differs on a run of {SIMULATED_CYCLES} clocks of random inputs nor in the first {SEARCH_CYCLES} clocks after reset")),
        Some((_, said)) if said.contains("DONE (TIMEOUT") || said.is_empty() => unknown(format!("{why_not}, and no output was found to differ on a run of {SIMULATED_CYCLES} clocks of random inputs nor from reset within {seconds} s")),
        None => unknown(format!("{why_not}, and no output was found to differ on a run of {SIMULATED_CYCLES} clocks of random inputs nor from reset within {seconds} s")),
        Some((_, said)) => unknown(format!("{why_not}; SymbiYosys could not search for a difference:\n{}", last_lines(&said, 6))),
    }
}

/// A port of an entity: its name, whether it is an input, its type and,
/// for a vector, its width.
#[derive(Debug, Clone, PartialEq)]
struct Port {
    name: String,
    input: bool,
    kind: String,
    width: Option<usize>,
}

/// The ports of an entity, when each is of a type the simulation can drive
/// and compare (`std_logic`, and vectors of it: `std_logic_vector`, `unsigned`, `signed`).
fn port_list(vhdl: &str, entity: &str) -> Option<Vec<Port>> {
    let comment = Regex::new(r"--[^\n]*").expect("a pattern of the ports");
    let text = comment.replace_all(vhdl, "").to_lowercase();
    let start = text.find(&format!("entity {} is", entity.to_lowercase()))?;
    let end = start + text[start..].find("end entity").or_else(|| text[start..].find(&format!("end {}", entity.to_lowercase())))?;
    let open = start + text[start..end].find("port")? + 4;
    let inside = text[open..end].trim().strip_prefix('(')?.trim_end().trim_end_matches(';').trim_end().strip_suffix(')')?;
    let vector = Regex::new(r"^(std_logic_vector|unsigned|signed)\s*\(\s*(\d+)\s+downto\s+(\d+)\s*\)$").expect("a pattern of the ports");
    let mut ports = Vec::new();
    for declaration in inside.split(';').map(str::trim).filter(|d| !d.is_empty()) {
        let (names, rest) = declaration.split_once(':')?;
        let (direction, kind) = rest.trim().split_once(char::is_whitespace)?;
        let kind = kind.trim();
        let (kind, width) = if kind == "std_logic" {
            (kind.to_string(), None)
        } else {
            let found = vector.captures(kind)?;
            (found[1].to_string(), Some(found[2].parse::<usize>().ok()?.checked_sub(found[3].parse::<usize>().ok()?)? + 1))
        };
        let input = match direction {
            "in" => true,
            "out" => false,
            _ => return None,
        };
        for name in names.split(',') {
            ports.push(Port { name: name.trim().to_string(), input, kind: kind.clone(), width });
        }
    }
    Some(ports)
}

/// The test bench that runs the two side by side from reset on random
/// inputs and stops at the first clock an output differs, naming each.
fn bench(entity: &str, ports: &[Port], cycles: usize, seed: usize) -> String {
    let declared = |port: &Port| match port.width {
        None => "std_logic".to_string(),
        Some(width) => format!("{}({} downto 0)", port.kind, width - 1),
    };
    let mut b = String::from("-- Written by `arclang drift --hdl --prove`: the logic the model generates (gold) and the logic as built (gate), side by side.\nlibrary gold;\nlibrary gate;\nlibrary ieee;\nuse ieee.std_logic_1164.all;\nuse ieee.numeric_std.all;\nuse ieee.math_real.all;\nuse std.textio.all;\n\nentity arclang_side_by_side is\nend entity;\n\narchitecture sim of arclang_side_by_side is\n");
    for port in ports {
        if port.input {
            b.push_str(&format!("  signal {} : {};\n", port.name, declared(port)));
        } else {
            b.push_str(&format!("  signal g_{0}, b_{0} : {1};\n", port.name, declared(port)));
        }
    }
    b.push_str("begin\n");
    for (library, prefix) in [("gold", "g_"), ("gate", "b_")] {
        let map: Vec<String> = ports.iter().map(|port| if port.input { format!("{0} => {0}", port.name) } else { format!("{0} => {prefix}{0}", port.name) }).collect();
        b.push_str(&format!("  {library}_side : entity {library}.{entity} port map ({});\n", map.join(", ")));
    }
    b.push_str(&format!("  process\n    variable s1 : positive := {seed};\n    variable s2 : positive := 7919;\n    variable r : real;\n    variable l : line;\n    variable differs : boolean := false;\n"));
    b.push_str("    impure function pick(n : natural) return natural is\n    begin\n      uniform(s1, s2, r);\n      return natural(floor(r * real(n)));\n    end function;\n");
    // Half the values are small: a count, an index, a condition are met far more often than by a word drawn whole.
    b.push_str("    impure function bits(w : natural) return std_logic_vector is\n      variable v : std_logic_vector(w - 1 downto 0) := (others => '0');\n    begin\n      if pick(2) = 0 then\n        v(minimum(w, 4) - 1 downto 0) := std_logic_vector(to_unsigned(pick(2 ** minimum(w, 4)), minimum(w, 4)));\n      else\n        for k in 0 to w - 1 loop\n          if pick(2) = 1 then\n            v(k) := '1';\n          end if;\n        end loop;\n      end if;\n      return v;\n    end function;\n  begin\n");
    for port in ports.iter().filter(|port| port.input && port.name != "clk") {
        b.push_str(&format!("    {} <= {};\n", port.name, if port.width.is_some() { "(others => '0')" } else { "'0'" }));
    }
    b.push_str(&format!("    rst <= '1';\n    clk <= '0';\n    wait for 5 ns;\n    clk <= '1';\n    wait for 5 ns;\n    rst <= '0';\n    for c in 1 to {cycles} loop\n"));
    for port in ports.iter().filter(|port| port.input && port.name != "clk" && port.name != "rst") {
        // An input keeps its value one clock in two: a condition has to hold a while to be seen.
        match port.width {
            None => b.push_str(&format!("      if pick(2) = 0 then\n        if pick(2) = 0 then {0} <= '1'; else {0} <= '0'; end if;\n      end if;\n", port.name)),
            Some(width) if port.kind == "std_logic_vector" => b.push_str(&format!("      if pick(2) = 0 then {} <= bits({width}); end if;\n", port.name)),
            Some(width) => b.push_str(&format!("      if pick(2) = 0 then {} <= {}(bits({width})); end if;\n", port.name, port.kind)),
        }
    }
    b.push_str("      clk <= '0';\n      wait for 5 ns;\n      clk <= '1';\n      wait for 5 ns;\n");
    for port in ports.iter().filter(|port| !port.input) {
        let shown = |side: &str| match port.width {
            None => format!("std_logic'image({side}_{})", port.name),
            Some(_) => format!("to_bstring(std_logic_vector({side}_{}))", port.name),
        };
        b.push_str(&format!("      if g_{0} /= b_{0} then\n        write(l, string'(\"DIFFER \") & integer'image(c) & string'(\" {0} \") & {1} & string'(\" \") & {2});\n        writeline(output, l);\n        differs := true;\n      end if;\n", port.name, shown("g"), shown("b")));
    }
    b.push_str("      if differs then\n        std.env.finish;\n      end if;\n    end loop;\n    write(l, string'(\"SAME\"));\n    writeline(output, l);\n    std.env.finish;\n  end process;\nend architecture;\n");
    b
}

/// The two simulated side by side on a run of random inputs: the clock at
/// which outputs differ and each of them, or None when none does (or when
/// the simulation cannot be run: it is one way to find a difference, and
/// its absence proves nothing).
fn simulate(ghdl: &str, generated: &[(String, String)], built: &[(String, String)], entity: &str, model_text: &str, work: &Path, until: Instant) -> Result<Option<(usize, Vec<(String, String, String)>)>, String> {
    let Some(ports) = port_list(model_text, entity) else {
        return Ok(None);
    };
    if !ports.iter().any(|port| port.input && port.name == "clk" && port.width.is_none()) || !ports.iter().any(|port| port.input && port.name == "rst" && port.width.is_none()) {
        return Ok(None);
    }
    let dir = work.join("simulation");
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    // Each side in a library of its own; a file name of its own too, as GHDL names its objects after the files.
    for (side, files) in [("gold", generated), ("gate", built)] {
        let mut names = Vec::new();
        for name in ordered(files) {
            let text = files.iter().find(|(file, _)| file == name).map(|(_, text)| text.as_str()).unwrap_or_default();
            let renamed = format!("{side}_{name}");
            std::fs::write(dir.join(&renamed), text).map_err(|e| format!("cannot write {renamed}: {e}"))?;
            names.push(renamed);
        }
        let library = format!("--work={side}");
        let mut analyse = vec!["-a", "--std=08", library.as_str()];
        analyse.extend(names.iter().map(String::as_str));
        if !matches!(run(ghdl, &analyse, &dir, until)?, Some((true, _))) {
            return Ok(None);
        }
    }
    std::fs::write(dir.join("arclang_side_by_side.vhd"), bench(entity, &ports, SIMULATED_CYCLES, SEED)).map_err(|e| e.to_string())?;
    for step in [vec!["-a", "--std=08", "arclang_side_by_side.vhd"], vec!["-e", "--std=08", "arclang_side_by_side"]] {
        if !matches!(run(ghdl, &step, &dir, until)?, Some((true, _))) {
            return Ok(None);
        }
    }
    let Some((_, said)) = run(ghdl, &["-r", "--std=08", "arclang_side_by_side"], &dir, until)? else {
        return Ok(None);
    };
    let line = Regex::new(r"(?m)^DIFFER (\d+) (\S+) (\S+) (\S+)\s*$").expect("a pattern of the simulation");
    let mut cycle = 0;
    let mut outputs = Vec::new();
    // A value as the bench prints it (`'1'`, or the bits of a vector), without its leading zeros.
    let value = |text: &str| {
        let bits = text.trim_matches('\'').trim_start_matches('0');
        format!("0b{}", if bits.is_empty() { "0" } else { bits })
    };
    for found in line.captures_iter(&said) {
        cycle = found[1].parse().unwrap_or(0);
        outputs.push((found[2].to_string(), value(&found[3]), value(&found[4])));
    }
    Ok(if outputs.is_empty() { None } else { Some((cycle, outputs)) })
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
    fn the_ports_are_read_with_their_types_and_the_bench_drives_and_compares_them() {
        let vhdl = "entity blk is\n  port (\n    clk, rst : in std_logic; -- the clock, the reset\n    i_v : in signed(31 downto 0);\n    ev : in std_logic_vector(7 downto 0);\n    done : out std_logic;\n    o_v : out unsigned(15 downto 0)\n  );\nend entity;";
        let ports = port_list(vhdl, "blk").unwrap();
        assert_eq!(ports.len(), 6);
        assert_eq!(ports[2], Port { name: "i_v".to_string(), input: true, kind: "signed".to_string(), width: Some(32) });
        assert_eq!(ports[5], Port { name: "o_v".to_string(), input: false, kind: "unsigned".to_string(), width: Some(16) });
        let bench = bench("blk", &ports, 1000, 1);
        assert!(bench.contains("gold_side : entity gold.blk port map (clk => clk, rst => rst, i_v => i_v, ev => ev, done => g_done, o_v => g_o_v);"), "{bench}");
        assert!(bench.contains("if pick(2) = 0 then i_v <= signed(bits(32)); end if;") && bench.contains("if pick(2) = 0 then ev <= bits(8); end if;"), "{bench}");
        assert!(bench.contains("if g_o_v /= b_o_v then") && bench.contains("for c in 1 to 1000 loop"), "{bench}");
        // A port of a type the bench cannot drive: no simulation, and the proof is asked alone.
        assert_eq!(port_list("entity blk is port (clk : in std_logic; n : in integer);\nend entity;", "blk"), None);
    }

    #[test]
    fn the_package_is_analysed_first_and_what_instantiates_last() {
        let files: Vec<(String, String)> = ["m_blk_wb.vhd", "m_blk.vhd", "arclang_fx_pkg.vhd", "m_blk_regs.json"].iter().map(|name| (name.to_string(), String::new())).collect();
        assert_eq!(ordered(&files), vec!["arclang_fx_pkg.vhd", "m_blk.vhd", "m_blk_wb.vhd"]);
    }
}
