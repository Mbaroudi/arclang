//! OpenModelica as an external tool: a local `omc`, or the official image
//! run by Docker. Nothing is passed through a shell: the model, the
//! sources and the script are files in a working directory mounted at
//! `/work`, and the only arguments are fixed.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// The image used when `ARCLANG_OMC_DOCKER` names none.
pub const DEFAULT_IMAGE: &str = "openmodelica/openmodelica:v1.27.1-minimal";
/// The Docker volume keeping the Modelica Standard Library between runs:
/// the minimal image has none, the first run installs it there.
pub const LIBRARY_VOLUME: &str = "arclang-openmodelica-libraries";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(900);

#[derive(Debug, Clone, PartialEq)]
pub enum Tool {
    Local(PathBuf),
    Docker(String),
}

impl Tool {
    pub fn describe(&self) -> String {
        match self {
            Tool::Local(path) => format!("omc ({})", path.display()),
            Tool::Docker(image) => format!("docker image {image}"),
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
        Some(true) => Command::new("docker")
            .args(["run", "--rm", "-v"])
            .arg(format!("{LIBRARY_VOLUME}:/l:ro"))
            .args(["--entrypoint", "ls", &image, "/l"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().any(|l| l.starts_with("Modelica "))),
        _ => None,
    };
    let (tool, detail) = match &located {
        Ok(Tool::Local(_)) => (Some("local".to_string()), located.as_ref().unwrap().describe()),
        Ok(Tool::Docker(_)) => (Some("docker".to_string()), located.as_ref().unwrap().describe()),
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

/// Run `omc script` in `dir` (the script a file of `dir`).
pub fn run(tool: &Tool, dir: &Path, script: &str, timeout: Option<Duration>) -> Result<Run, String> {
    let dir = dir.canonicalize().map_err(|e| format!("cannot use the working directory {}: {e}", dir.display()))?;
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let container = format!("arclang-omc-{}-{nanos}", std::process::id());
    let mut command = match tool {
        Tool::Local(omc) => {
            let mut command = Command::new(omc);
            command.arg(script).current_dir(&dir);
            command
        }
        Tool::Docker(image) => {
            let mut command = Command::new("docker");
            command
                .arg("run")
                .arg("--rm")
                .arg("--name")
                .arg(&container)
                .arg("-v")
                .arg(format!("{}:/work", dir.display()))
                .arg("-v")
                .arg(format!("{LIBRARY_VOLUME}:/root/.openmodelica/libraries"))
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
    let deadline = Instant::now() + timeout.unwrap_or(DEFAULT_TIMEOUT);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                if matches!(tool, Tool::Docker(_)) {
                    let _ = Command::new("docker").args(["rm", "-f", &container]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                }
                return Err(format!("the simulation took more than {} s (ARCLANG_OMC_TIMEOUT)", timeout.unwrap_or(DEFAULT_TIMEOUT).as_secs()));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => return Err(format!("{} failed: {e}", tool.describe())),
        }
    };
    let output = std::fs::read_to_string(&log).unwrap_or_default();
    Ok(Run { success: status.success(), output })
}

#[cfg(test)]
mod tests {
    use super::*;

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
