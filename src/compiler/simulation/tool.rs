//! OpenModelica as an external tool: a local `omc`, or the official image
//! run by Docker. Nothing is passed through a shell: the model, the
//! sources and the script are files in a working directory mounted at
//! `/work`, and the only arguments are fixed.
//!
//! A model's Modelica can run commands where it is simulated. The container
//! that runs it is therefore started without a network, without a way to
//! gain privileges, and with the library volume read-only: what it reads in
//! `/work` does not leave, and it cannot alter the library the next model is
//! simulated with. The library is installed by another container, which
//! runs no file of the model.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// The image used when `ARCLANG_OMC_DOCKER` names none.
pub const DEFAULT_IMAGE: &str = "openmodelica/openmodelica:v1.27.1-minimal";
/// The Docker volume keeping the Modelica Standard Library between runs:
/// the minimal image has none, the first run installs it there.
pub const LIBRARY_VOLUME: &str = "arclang-openmodelica-libraries";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(900);
/// `ARCLANG_OMC_NETWORK=on` leaves the container that runs a model on Docker's network.
pub const NETWORK: &str = "ARCLANG_OMC_NETWORK";
/// Where the image's `omc` looks for the libraries.
const LIBRARIES: &str = "/root/.openmodelica/libraries";

#[derive(Debug, Clone, PartialEq)]
pub enum Tool {
    Local(PathBuf),
    Docker(String),
    /// No OpenModelica: this computer's shell and C compiler only (a given
    /// FMU with a binary for this platform, stepped here).
    Host,
}

impl Tool {
    pub fn describe(&self) -> String {
        match self {
            Tool::Local(path) => format!("omc ({})", path.display()),
            Tool::Docker(image) => format!("docker image {image}"),
            Tool::Host => "this computer's C compiler".to_string(),
        }
    }
}

/// Where to look for OpenModelica, read once from the environment.
#[derive(Debug, Clone, Default)]
pub struct ToolConfig {
    /// `ARCLANG_OMC`: the path to a local `omc`.
    pub omc: Option<String>,
    /// `ARCLANG_OMC_DOCKER`: the image to run.
    pub docker_image: Option<String>,
    /// `ARCLANG_OMC_TIMEOUT`: seconds a simulation may take.
    pub timeout: Option<Duration>,
}

impl ToolConfig {
    pub fn from_env() -> ToolConfig {
        let read = |key: &str| std::env::var(key).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        ToolConfig {
            omc: read("ARCLANG_OMC"),
            docker_image: read("ARCLANG_OMC_DOCKER"),
            timeout: read("ARCLANG_OMC_TIMEOUT").and_then(|s| s.parse::<u64>().ok()).map(Duration::from_secs),
        }
    }
}

fn runs(program: &str, args: &[&str]) -> bool {
    Command::new(program).args(args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok_and(|s| s.success())
}

fn docker_ready() -> bool {
    runs("docker", &["version", "--format", "{{.Server.Version}}"])
}

/// The tool to run, or why there is none: `ARCLANG_OMC` first, then
/// `ARCLANG_OMC_DOCKER`, then an `omc` on the PATH, then Docker with the
/// default image.
pub fn locate(config: &ToolConfig) -> Result<Tool, String> {
    if let Some(omc) = &config.omc {
        return if runs(omc, &["--version"]) {
            Ok(Tool::Local(PathBuf::from(omc)))
        } else {
            Err(format!("OpenModelica not available: ARCLANG_OMC={omc} does not run"))
        };
    }
    if let Some(image) = &config.docker_image {
        return if docker_ready() {
            Ok(Tool::Docker(image.clone()))
        } else {
            Err(format!("OpenModelica not available: ARCLANG_OMC_DOCKER={image}, but no Docker daemon answers"))
        };
    }
    if runs("omc", &["--version"]) {
        return Ok(Tool::Local(PathBuf::from("omc")));
    }
    if docker_ready() {
        return Ok(Tool::Docker(DEFAULT_IMAGE.to_string()));
    }
    Err("OpenModelica not available: no omc on the PATH (ARCLANG_OMC), and no Docker daemon to run it (ARCLANG_OMC_DOCKER)".to_string())
}

/// What is installed to run OpenModelica, for a human or a program (the
/// editor reads it as JSON): the single source of truth for discovery.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Diagnosis {
    /// `local`, `docker` or null: the tool a simulation would use now.
    pub tool: Option<String>,
    /// The tool described (`docker image ...`, `omc (...)`), or why there is none.
    pub detail: String,
    pub docker_installed: bool,
    pub docker_running: bool,
    /// The image a simulation would run.
    pub image: String,
    /// Null when Docker is not running (cannot tell).
    pub image_pulled: Option<bool>,
    pub library_volume: String,
    /// The Modelica Standard Library version a simulation loads.
    pub msl_version: String,
    /// Whether the Modelica Standard Library is in the volume; null when it cannot be told.
    pub library_installed: Option<bool>,
    /// `omc` on the PATH or `ARCLANG_OMC` runs.
    pub local_omc: bool,
}

impl Diagnosis {
    pub fn summary(&self) -> String {
        let yes = |b: Option<bool>| match b { Some(true) => "yes", Some(false) => "no", None => "unknown" };
        format!(
            "OpenModelica: {}\n  local omc: {}\n  Docker installed: {}\n  Docker running: {}\n  image {}: pulled {}\n  Modelica library ({}): installed {}",
            self.detail,
            if self.local_omc { "yes" } else { "no" },
            if self.docker_installed { "yes" } else { "no" },
            if self.docker_running { "yes" } else { "no" },
            self.image,
            yes(self.image_pulled),
            self.library_volume,
            yes(self.library_installed),
        )
    }
}

/// Look at the machine: what [`locate`] would pick and what is missing.
pub fn diagnose(config: &ToolConfig) -> Diagnosis {
    let located = locate(config);
    let image = config.docker_image.clone().unwrap_or_else(|| DEFAULT_IMAGE.to_string());
    let docker_installed = runs("docker", &["--version"]);
    let docker_running = docker_installed && docker_ready();
    let local_omc = match &config.omc {
        Some(omc) => runs(omc, &["--version"]),
        None => runs("omc", &["--version"]),
    };
    let image_pulled = docker_running.then(|| runs("docker", &["image", "inspect", &image]));
    let library_installed = match image_pulled {
        Some(true) => libraries_in_volume(&image).map(|libraries| libraries.iter().any(|l| l.starts_with("Modelica "))),
        _ => None,
    };
    let (tool, detail) = match &located {
        Ok(Tool::Local(_)) => (Some("local".to_string()), located.as_ref().unwrap().describe()),
        Ok(Tool::Docker(_)) => (Some("docker".to_string()), located.as_ref().unwrap().describe()),
        Ok(Tool::Host) => (Some("host".to_string()), located.as_ref().unwrap().describe()),
        Err(error) => (None, error.clone()),
    };
    Diagnosis { tool, detail, docker_installed, docker_running, image, image_pulled, library_volume: LIBRARY_VOLUME.to_string(), msl_version: crate::compiler::modelica::MSL_VERSION.to_string(), library_installed, local_omc }
}

/// What a run printed.
#[derive(Debug)]
pub struct Run {
    pub success: bool,
    pub output: String,
}

/// The entries of the library volume (`Modelica 4.1.0+maint.om`), None when it cannot be read.
fn libraries_in_volume(image: &str) -> Option<Vec<String>> {
    Command::new("docker")
        .args(["run", "--rm", "--network", "none", "-v"])
        .arg(format!("{LIBRARY_VOLUME}:/l:ro"))
        .args(["--entrypoint", "ls", image, "/l"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().map(str::to_string).collect())
}

/// Whether an entry of the library volume is the Modelica Standard Library of this version.
fn is_the_library(entry: &str, msl: &str) -> bool {
    entry.strip_prefix("Modelica ").and_then(|version| version.strip_prefix(msl)).is_some_and(|rest| rest.is_empty() || rest.starts_with(['+', '-', ' ']))
}

/// The flags of a container that runs what a model wrote.
fn isolation(network_kept: bool) -> Vec<&'static str> {
    let mut flags = vec!["--security-opt", "no-new-privileges"];
    if !network_kept {
        flags.extend(["--network", "none"]);
    }
    flags
}

fn network_kept() -> bool {
    std::env::var(NETWORK).is_ok_and(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "on" | "1" | "true" | "yes"))
}

/// Waits for a container until the deadline; past it, stops and removes it: None.
fn wait(child: &mut std::process::Child, deadline: Instant, container: Option<&str>) -> Result<Option<std::process::ExitStatus>, String> {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(Some(status)),
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                if let Some(container) = container {
                    let _ = Command::new("docker").args(["rm", "-f", container]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                }
                return Ok(None);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => return Err(e.to_string()),
        }
    }
}

/// The library is in the volume, as far as this process has seen.
static LIBRARY_READY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Installs the Modelica Standard Library in the volume when it is not
/// there. This container is the only one with the network and the volume
/// writable, and it runs no file of the model: its script is the line
/// below. When the library cannot be installed (offline), the model's run
/// says so as it did: it cannot load it.
fn install_library(image: &str, container: &str, deadline: Instant) -> Result<(), String> {
    use std::sync::atomic::Ordering;
    let msl = crate::compiler::modelica::MSL_VERSION;
    if LIBRARY_READY.load(Ordering::Relaxed) {
        return Ok(());
    }
    if libraries_in_volume(image).is_some_and(|libraries| libraries.iter().any(|entry| is_the_library(entry, msl))) {
        LIBRARY_READY.store(true, Ordering::Relaxed);
        return Ok(());
    }
    let line = format!("if not loadModel(Modelica, {{\"{msl}\"}}) then installPackage(Modelica, \"{msl}+maint.om\", exactMatch=true); end if;");
    let mut child = Command::new("docker")
        .args(["run", "--rm", "--name", container, "--security-opt", "no-new-privileges", "-v"])
        .arg(format!("{LIBRARY_VOLUME}:{LIBRARIES}"))
        .args(["--entrypoint", "sh", image, "-c", "printf '%s\\n' \"$1\" > /tmp/arclang-library.mos && exec omc /tmp/arclang-library.mos", "sh", &line])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("cannot start docker image {image}: {e}"))?;
    match wait(&mut child, deadline, Some(container)).map_err(|e| format!("docker image {image} failed: {e}"))? {
        Some(_) => Ok(()),
        None => Err(format!("installing the Modelica Standard Library {msl} took more than the time a simulation may take (ARCLANG_OMC_TIMEOUT)")),
    }
}

/// Run `omc script` in `dir` (the script a file of `dir`).
pub fn run(tool: &Tool, dir: &Path, script: &str, timeout: Option<Duration>) -> Result<Run, String> {
    let dir = dir.canonicalize().map_err(|e| format!("cannot use the working directory {}: {e}", dir.display()))?;
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let container = format!("arclang-omc-{}-{nanos}-{}", std::process::id(), run_number());
    let deadline = Instant::now() + timeout.unwrap_or(DEFAULT_TIMEOUT);
    let mut command = match tool {
        Tool::Local(omc) => {
            let mut command = Command::new(omc);
            command.arg(script).current_dir(&dir);
            command
        }
        Tool::Host => return Err("OpenModelica not available: no omc on the PATH (ARCLANG_OMC), and no Docker daemon to run it (ARCLANG_OMC_DOCKER)".to_string()),
        Tool::Docker(image) => {
            install_library(image, &format!("{container}-library"), deadline)?;
            let mut command = Command::new("docker");
            command
                .arg("run")
                .arg("--rm")
                .arg("--name")
                .arg(&container)
                .args(isolation(network_kept()))
                .arg("-v")
                .arg(format!("{}:/work", dir.display()))
                .arg("-v")
                .arg(format!("{LIBRARY_VOLUME}:{LIBRARIES}:ro"))
                .arg("-w")
                .arg("/work")
                .arg(image)
                .arg("omc")
                .arg(script);
            command
        }
    };
    let log = dir.join("omc.log");
    let file = std::fs::File::create(&log).map_err(|e| format!("cannot write {}: {e}", log.display()))?;
    let err = file.try_clone().map_err(|e| e.to_string())?;
    let mut child = command.stdin(Stdio::null()).stdout(file).stderr(err).spawn().map_err(|e| format!("cannot start {}: {e}", tool.describe()))?;
    let named = matches!(tool, Tool::Docker(_)).then_some(container.as_str());
    let Some(status) = wait(&mut child, deadline, named).map_err(|e| format!("{} failed: {e}", tool.describe()))? else {
        return Err(format!("the simulation took more than {} s (ARCLANG_OMC_TIMEOUT)", timeout.unwrap_or(DEFAULT_TIMEOUT).as_secs()));
    };
    let output = std::fs::read_to_string(&log).unwrap_or_default();
    Ok(Run { success: status.success(), output })
}

/// A program of `dir` started where the tool runs (in the image, or here), talking on its
/// stdin and stdout; its stderr goes to `log`. `sh script` is the only command: the script is
/// a file of `dir`. The second value names the container to remove when the program is stopped.
pub fn spawn(tool: &Tool, dir: &Path, script: &str, log: &Path) -> Result<(std::process::Child, Option<String>), String> {
    let dir = dir.canonicalize().map_err(|e| format!("cannot use the working directory {}: {e}", dir.display()))?;
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let (mut command, container) = match tool {
        Tool::Local(_) | Tool::Host => {
            let mut command = Command::new("sh");
            command.arg(script).current_dir(&dir);
            (command, None)
        }
        Tool::Docker(image) => {
            let container = format!("arclang-twin-{}-{nanos}-{}", std::process::id(), run_number());
            let mut command = Command::new("docker");
            command.args(["run", "-i", "--rm", "--name", &container]).args(isolation(network_kept())).arg("-v").arg(format!("{}:/work", dir.display())).args(["-w", "/work", image, "sh", script]);
            (command, Some(container))
        }
    };
    let err = std::fs::File::create(log).map_err(|e| format!("cannot write {}: {e}", log.display()))?;
    let child = command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(err).spawn().map_err(|e| format!("cannot start {}: {e}", tool.describe()))?;
    Ok((child, container))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_container_of_a_model_has_no_network_unless_it_is_kept() {
        assert_eq!(isolation(false), ["--security-opt", "no-new-privileges", "--network", "none"]);
        assert_eq!(isolation(true), ["--security-opt", "no-new-privileges"]);
    }

    #[test]
    fn the_library_of_this_version_is_recognised_in_the_volume() {
        assert!(is_the_library("Modelica 4.1.0+maint.om", "4.1.0"));
        assert!(is_the_library("Modelica 4.1.0", "4.1.0"));
        // Another version, or another library, is not it: the install runs.
        assert!(!is_the_library("Modelica 4.0.0+maint.om", "4.1.0"));
        assert!(!is_the_library("Modelica 4.1.01+maint.om", "4.1.0"));
        assert!(!is_the_library("ModelicaServices 4.1.0+maint.om", "4.1.0"));
        assert!(!is_the_library("index.json", "4.1.0"));
    }

    #[test]
    fn an_explicit_omc_that_does_not_run_is_not_available() {
        let config = ToolConfig { omc: Some("/nonexistent/arclang/omc".into()), ..ToolConfig::default() };
        let error = locate(&config).unwrap_err();
        assert!(error.starts_with("OpenModelica not available"), "{error}");
    }

    #[test]
    fn the_diagnosis_of_an_explicit_omc_that_does_not_run_names_no_tool() {
        let config = ToolConfig { omc: Some("/nonexistent/arclang/omc".into()), docker_image: Some("example/om:1".into()), ..ToolConfig::default() };
        let d = diagnose(&config);
        assert_eq!(d.tool, None);
        assert!(!d.local_omc);
        assert_eq!(d.image, "example/om:1");
        assert_eq!(d.library_volume, LIBRARY_VOLUME);
        let json = serde_json::to_value(&d).unwrap();
        for key in ["tool", "detail", "docker_installed", "docker_running", "image", "image_pulled", "library_volume", "msl_version", "library_installed", "local_omc"] {
            assert!(json.get(key).is_some(), "{key}");
        }
        assert!(d.summary().starts_with("OpenModelica: OpenModelica not available"));
    }
}

/// A number no other run of this process has: two runs started in the same
/// instant (two threads, a clock that counts in microseconds) must not share
/// a work directory or a container name.
pub(crate) fn run_number() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
