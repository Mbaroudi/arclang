//! Target packages: the third layer. A target implements `arclang_io.h`
//! once and says how to build, run and flash what ArcLang generates for it,
//! as data: a directory with a `target.json` and the files it lists.
//!
//! ```text
//! targets/zephyr/litex_vexriscv/
//!     target.json          name, runtime, board, channels, files, build, emulator
//!     litex_vexriscv.overlay, arclang_module/..., renode/..., run.sh
//! targets/zephyr/common/   what every Zephyr target shares (arclang_io_zephyr.c, mains, CMake)
//! ```
//!
//! An `io_binding` names the target it is deployed on (`target:
//! "zephyr/litex_vexriscv"`, `host` by default); the compiler checks that
//! the package exists and that it has every channel the binding uses.
//! Packages are built in (`targets/` beside this file, below) or found in
//! the directories of `ARCLANG_TARGET_PATH` (`<dir>/<name>/target.json`):
//! a new board is a new directory, not a change of the compiler.

use super::binding::{Binding, Sink, Source};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The packages compiled into ArcLang: (path under `targets/`, text).
const BUILTIN: &[(&str, &str)] = &[
    ("host/target.json", include_str!("targets/host/target.json")),
    ("runtime/arclang_io_host.h", include_str!("runtime/arclang_io_host.h")),
    ("runtime/arclang_io_host.c", include_str!("runtime/arclang_io_host.c")),
    ("zephyr/common/CMakeLists.txt", include_str!("targets/zephyr/common/CMakeLists.txt")),
    ("zephyr/common/prj.conf", include_str!("targets/zephyr/common/prj.conf")),
    ("zephyr/common/arclang_io_zephyr.c", include_str!("targets/zephyr/common/arclang_io_zephyr.c")),
    ("zephyr/common/main.c", include_str!("targets/zephyr/common/main.c")),
    ("zephyr/common/pil_io.c", include_str!("targets/zephyr/common/pil_io.c")),
    ("zephyr/common/pil_direct.c", include_str!("targets/zephyr/common/pil_direct.c")),
    ("zephyr/common/pil_link.c", include_str!("targets/zephyr/common/pil_link.c")),
    ("zephyr/litex_vexriscv/target.json", include_str!("targets/zephyr/litex_vexriscv/target.json")),
    ("zephyr/litex_vexriscv/litex_vexriscv.overlay", include_str!("targets/zephyr/litex_vexriscv/litex_vexriscv.overlay")),
    ("zephyr/litex_vexriscv/litex_vexriscv.conf", include_str!("targets/zephyr/litex_vexriscv/litex_vexriscv.conf")),
    ("zephyr/litex_vexriscv/arclang_module/zephyr/module.yml", include_str!("targets/zephyr/litex_vexriscv/arclang_module/zephyr/module.yml")),
    ("zephyr/litex_vexriscv/arclang_module/CMakeLists.txt", include_str!("targets/zephyr/litex_vexriscv/arclang_module/CMakeLists.txt")),
    ("zephyr/litex_vexriscv/arclang_module/Kconfig", include_str!("targets/zephyr/litex_vexriscv/arclang_module/Kconfig")),
    ("zephyr/litex_vexriscv/arclang_module/drivers/can_litex_sja1000.c", include_str!("targets/zephyr/litex_vexriscv/arclang_module/drivers/can_litex_sja1000.c")),
    ("zephyr/litex_vexriscv/arclang_module/drivers/adc_litex_regs.c", include_str!("targets/zephyr/litex_vexriscv/arclang_module/drivers/adc_litex_regs.c")),
    ("zephyr/litex_vexriscv/arclang_module/dts/bindings/arclang,litex-sja1000.yaml", include_str!("targets/zephyr/litex_vexriscv/arclang_module/dts/bindings/arclang,litex-sja1000.yaml")),
    ("zephyr/litex_vexriscv/arclang_module/dts/bindings/arclang,litex-adc.yaml", include_str!("targets/zephyr/litex_vexriscv/arclang_module/dts/bindings/arclang,litex-adc.yaml")),
    ("zephyr/litex_vexriscv/renode/arclang_litex_vexriscv.repl", include_str!("targets/zephyr/litex_vexriscv/renode/arclang_litex_vexriscv.repl")),
    ("zephyr/litex_vexriscv/renode/ArcLangSJA1000.cs", include_str!("targets/zephyr/litex_vexriscv/renode/ArcLangSJA1000.cs")),
    ("zephyr/litex_vexriscv/renode/RegisterADC.cs", include_str!("targets/zephyr/litex_vexriscv/renode/RegisterADC.cs")),
    ("zephyr/litex_vexriscv/renode/LiteXPWM.cs", include_str!("targets/zephyr/litex_vexriscv/renode/LiteXPWM.cs")),
    ("zephyr/litex_vexriscv/renode/LiteXWatchdog.cs", include_str!("targets/zephyr/litex_vexriscv/renode/LiteXWatchdog.cs")),
    ("zephyr/litex_vexriscv/renode/pil_bench.py", include_str!("targets/zephyr/litex_vexriscv/renode/pil_bench.py")),
    ("zephyr/litex_vexriscv/run.sh", include_str!("targets/zephyr/litex_vexriscv/run.sh")),
];

/// The default target of a binding.
pub const HOST: &str = "host";

/// What a target offers of one channel kind.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Channel {
    /// The channel numbers: `"0-31"`, `"0"`, `"0,2,5-7"`; absent: none needed (watchdog).
    #[serde(default)]
    pub channels: Option<String>,
    /// For CAN: how many distinct identifiers it takes.
    #[serde(default)]
    pub identifiers: Option<usize>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileSpec {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub executable: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Build {
    /// The build command; `{app}`, `{build}`, `{main}` and `{period_define}` are replaced.
    pub command: Vec<String>,
    /// The image the build makes, `{build}` replaced.
    pub image: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Emulator {
    pub kind: String,
    /// The emulator's command; `{app}` is replaced. The plan's path is in
    /// `ARCLANG_PIL_PLAN`.
    pub command: Vec<String>,
    /// Emulator-side models to load first (paths in the exported application).
    #[serde(default)]
    pub peripherals: Vec<String>,
    pub platform: String,
    pub bench: String,
    /// The emulator's names of the channels (the bench's plan).
    #[serde(default)]
    pub channels: BTreeMap<String, serde_json::Value>,
}

/// A target package, read from its `target.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct Package {
    pub name: String,
    #[serde(default)]
    pub summary: String,
    /// What is generated for it: `host` (the C and the host board) or
    /// `zephyr` (a Zephyr application).
    pub runtime: String,
    #[serde(default)]
    pub board: Option<String>,
    #[serde(default)]
    pub toolchain: Option<String>,
    pub channels: BTreeMap<String, Channel>,
    #[serde(default)]
    pub files: Vec<FileSpec>,
    #[serde(default)]
    pub build: Option<Build>,
    #[serde(default)]
    pub emulator: Option<Emulator>,
    /// Where it was read from: `None` for a built-in package.
    #[serde(skip)]
    pub dir: Option<PathBuf>,
    #[serde(skip)]
    pub builtin_dir: String,
}

/// The channel kinds a package declares.
pub const KINDS: [&str; 10] = ["gpio_in", "gpio_out", "adc", "dac", "pwm", "can_rx", "can_tx", "uart_rx", "uart_tx", "watchdog"];

fn normalize(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

fn builtin(path: &str) -> Option<&'static str> {
    let path = normalize(path);
    BUILTIN.iter().find(|(p, _)| *p == path).map(|(_, text)| *text)
}

fn search_path() -> Vec<PathBuf> {
    std::env::var_os("ARCLANG_TARGET_PATH").map(|p| std::env::split_paths(&p).collect()).unwrap_or_default()
}

fn parse(text: &str, origin: &str) -> Result<Package, String> {
    let package: Package = serde_json::from_str(text).map_err(|e| format!("target package {origin}: target.json: {e}"))?;
    for kind in package.channels.keys() {
        if !KINDS.contains(&kind.as_str()) {
            return Err(format!("target package {origin}: '{kind}' is no channel kind ({})", KINDS.join(", ")));
        }
    }
    for (kind, channel) in &package.channels {
        if let Some(list) = &channel.channels {
            numbers(list).map_err(|e| format!("target package {origin}: {kind}: {e}"))?;
        }
    }
    if !["host", "zephyr"].contains(&package.runtime.as_str()) {
        return Err(format!("target package {origin}: runtime '{}' is not one ArcLang generates for (host, zephyr)", package.runtime));
    }
    Ok(package)
}

/// The names of every package: built in, then on `ARCLANG_TARGET_PATH`.
pub fn names() -> Vec<String> {
    let mut out: BTreeSet<String> = BUILTIN.iter().filter_map(|(p, _)| p.strip_suffix("/target.json")).map(str::to_string).collect();
    for dir in search_path() {
        for entry in walk(&dir) {
            if let Ok(rel) = entry.strip_prefix(&dir) {
                if let Some(name) = rel.parent().and_then(|p| p.to_str()) {
                    out.insert(name.replace('\\', "/"));
                }
            }
        }
    }
    out.into_iter().collect()
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![(dir.to_path_buf(), 0)];
    while let Some((d, depth)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && depth < 3 {
                stack.push((path, depth + 1));
            } else if path.file_name().is_some_and(|n| n == "target.json") {
                out.push(path);
            }
        }
    }
    out
}

/// The package named `name`: a directory of `ARCLANG_TARGET_PATH` first
/// (so a package can be overridden), else a built-in one.
pub fn find(name: &str) -> Result<Package, String> {
    if name.is_empty() || name.split('/').any(|p| p.is_empty() || p == "." || p == "..") {
        return Err(format!("'{name}' is no target name (a name is `family/board`, or `host`)"));
    }
    for dir in search_path() {
        let path = dir.join(name).join("target.json");
        if let Ok(text) = std::fs::read_to_string(&path) {
            let mut package = parse(&text, &path.display().to_string())?;
            package.dir = path.parent().map(Path::to_path_buf);
            return check_name(package, name);
        }
    }
    match builtin(&format!("{name}/target.json")) {
        Some(text) => {
            let mut package = parse(text, name)?;
            package.builtin_dir = name.to_string();
            check_name(package, name)
        }
        None => Err(format!("'{name}' is no target package (known: {}; more in the directories of ARCLANG_TARGET_PATH, as <dir>/{name}/target.json)", names().join(", "))),
    }
}

fn check_name(package: Package, name: &str) -> Result<Package, String> {
    if package.name != name {
        return Err(format!("target package {name}: its target.json names it '{}'", package.name));
    }
    Ok(package)
}

impl Package {
    /// A file of the package, by its path relative to the package
    /// (`../common/x` reaches a sibling directory).
    pub fn read(&self, path: &str) -> Result<String, String> {
        match &self.dir {
            Some(dir) => {
                let full = dir.join(path);
                std::fs::read_to_string(&full).map_err(|e| format!("target package {}: cannot read {}: {e}", self.name, full.display()))
            }
            None => builtin(&format!("{}/{path}", self.builtin_dir)).map(str::to_string).ok_or_else(|| format!("target package {}: no file {path}", self.name)),
        }
    }

    /// Every file the package puts in an export: (destination, text, executable).
    pub fn files(&self) -> Result<Vec<(String, String, bool)>, String> {
        self.files.iter().map(|f| Ok((f.to.clone(), self.read(&f.from)?, f.executable))).collect()
    }

    /// What it offers, said for a message: `gpio_in 0-31, adc 0-7, watchdog`.
    pub fn offers(&self) -> String {
        KINDS
            .iter()
            .filter_map(|k| {
                self.channels.get(*k).map(|c| match (&c.channels, c.identifiers) {
                    (Some(list), _) => format!("{k} {list}"),
                    (None, Some(n)) => format!("{k} ({n} identifiers)"),
                    _ => k.to_string(),
                })
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn has(&self, kind: &str, number: u32) -> Result<(), String> {
        let channel = self.channels.get(kind).ok_or_else(|| format!("target '{}' has no {kind} (it has {})", self.name, self.offers()))?;
        if let Some(list) = &channel.channels {
            if !numbers(list).map(|n| n.contains(&number)).unwrap_or(false) {
                return Err(format!("target '{}' has {kind} {list}, not {number}", self.name));
            }
        }
        Ok(())
    }
}

/// `"0-31"`, `"0,2,5-7"` as numbers.
pub fn numbers(list: &str) -> Result<BTreeSet<u32>, String> {
    let mut out = BTreeSet::new();
    for part in list.split(',').map(str::trim).filter(|p| !p.is_empty()) {
        let (a, b) = part.split_once('-').unwrap_or((part, part));
        let a: u32 = a.trim().parse().map_err(|_| format!("'{part}' is no channel range (`0-31`, `0,2`)"))?;
        let b: u32 = b.trim().parse().map_err(|_| format!("'{part}' is no channel range (`0-31`, `0,2`)"))?;
        if b < a || b - a > 65_535 {
            return Err(format!("'{part}' is no channel range (`0-31`, `0,2`)"));
        }
        out.extend(a..=b);
    }
    Ok(out)
}

/// Whether the target has every channel the binding uses, the first
/// missing one with the reason.
pub fn check(binding: &Binding, package: &Package) -> Result<(), String> {
    let what = |written: &str, e: String| format!("io_binding '{}': '{written}' cannot be deployed: {e}", binding.name);
    let mut received: BTreeSet<u32> = BTreeSet::new();
    let mut sent: BTreeSet<u32> = BTreeSet::new();
    for input in &binding.inputs {
        let result = match input.source {
            Source::GpioLevel(pin) | Source::GpioEdge(pin, _) => package.has("gpio_in", pin as u32),
            Source::Adc { channel, .. } => package.has("adc", channel as u32),
            Source::Can(id) => {
                received.insert(id);
                package.has("can_rx", 0)
            }
            Source::Uart { port, .. } => package.has("uart_rx", port as u32),
        };
        result.map_err(|e| what(&input.written, e))?;
    }
    for output in &binding.outputs {
        let result = match output.sink {
            Sink::Gpio(pin) => package.has("gpio_out", pin as u32),
            Sink::Pwm(channel, _) => package.has("pwm", channel as u32),
            Sink::Dac(channel, _) => package.has("dac", channel as u32),
            Sink::Can(id) => {
                sent.insert(id);
                package.has("can_tx", 0)
            }
            Sink::Uart(port, _) => package.has("uart_tx", port as u32),
        };
        result.map_err(|e| what(&output.written, e))?;
    }
    for (kind, ids) in [("can_rx", &received), ("can_tx", &sent)] {
        if let Some(max) = package.channels.get(kind).and_then(|c| c.identifiers) {
            if ids.len() > max {
                return Err(format!("io_binding '{}': {} CAN identifiers, target '{}' takes {max} ({kind})", binding.name, ids.len(), package.name));
            }
        }
    }
    if binding.watchdog {
        package.has("watchdog", 0).map_err(|e| format!("io_binding '{}': watchdog: true cannot be deployed: {e}", binding.name))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_packages_read_and_list_their_files() {
        for name in ["host", "zephyr/litex_vexriscv"] {
            let package = find(name).unwrap();
            assert_eq!(package.name, name);
            let files = package.files().unwrap();
            assert!(!files.is_empty(), "{name}");
        }
        assert!(names().contains(&"zephyr/litex_vexriscv".to_string()));
        let e = find("zephyr/nope").unwrap_err();
        assert!(e.contains("is no target package (known: host, zephyr/litex_vexriscv"), "{e}");
        assert!(find("../x").is_err());
    }

    #[test]
    fn channel_ranges() {
        assert_eq!(numbers("0-3, 7").unwrap().into_iter().collect::<Vec<_>>(), vec![0, 1, 2, 3, 7]);
        assert!(numbers("3-1").is_err());
        assert!(numbers("a").is_err());
    }
}
