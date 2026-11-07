//! The embedded stages through Docker. `arclang pil`, `hdl` and `soc` need
//! toolchains a host rarely has: Zephyr and Renode, GHDL and Yosys, LiteX
//! and Verilator. ArcLang Studio (`tools/studio`, published as
//! `ghcr.io/mbaroudi/arclang-studio:<version>`) holds them all, with the
//! compiler of the same version. When this host lacks a stage's tools, the
//! command runs itself in that image (pulled the first time), the model's
//! directory mounted at `/work`, as this host's user, and its verdict lands
//! in the model's cache as if it had run here: `check`, the ladder and
//! `realize` read it the same way. `ARCLANG_EMBEDDED_DOCKER` names another
//! image, `dev` the images `tools/embedded/*/run.sh` build with the compiler
//! of their target volume (for work on the compiler itself), or `off` keeps
//! everything on the host.

use std::path::Path;
use std::process::{Command, Stdio};

/// The image of each stage, as `tools/embedded/*/run.sh` build them (`dev`).
pub const IMAGES: [(&str, &str); 3] = [("pil", "arclang-embedded-soc:1"), ("hdl", "arclang-embedded-fpga:1"), ("soc", "arclang-embedded-system:1")];

/// ArcLang Studio, published at each release.
pub const STUDIO: &str = "ghcr.io/mbaroudi/arclang-studio";

/// The Studio of this compiler's version: the same compiler in the image.
pub fn studio_image() -> String {
    format!("{STUDIO}:{}", env!("CARGO_PKG_VERSION"))
}

/// The tools a stage runs on the host, each with the arguments that say it runs.
fn tools(stage: &str) -> Vec<(&'static str, &'static [&'static str])> {
    match stage {
        "pil" => vec![("west", &["--version"]), ("renode", &["--version"])],
        "hdl" => vec![("ghdl", &["--version"])],
        "soc" => vec![("verilator", &["--version"]), ("ghdl", &["--version"])],
        _ => Vec::new(),
    }
}

fn runs(program: &str, args: &[&str]) -> bool {
    Command::new(program).args(args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok_and(|s| s.success())
}

/// The tools of the stage this host does not have.
pub fn missing_tools(stage: &str) -> Vec<&'static str> {
    let mut missing: Vec<&'static str> = tools(stage).into_iter().filter(|(program, args)| !runs(program, args)).map(|(program, _)| program).collect();
    if stage == "soc" {
        let python = std::env::var("ARCLANG_LITEX_PYTHON").unwrap_or_else(|_| "python3".to_string());
        if !runs(&python, &["-c", "import litex"]) {
            missing.push("litex (python)");
        }
    }
    missing
}

fn setting() -> Option<String> {
    std::env::var("ARCLANG_EMBEDDED_DOCKER").ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

/// Whether the stages run in the images built from the sources (`dev`).
fn dev() -> bool {
    setting().is_some_and(|v| v.eq_ignore_ascii_case("dev"))
}

/// The image a stage runs in: ArcLang Studio of this version, the stage's
/// own with `dev`, or the image `ARCLANG_EMBEDDED_DOCKER` names; none when
/// that is `off`.
pub fn image_of(stage: &str) -> Option<String> {
    match setting() {
        Some(v) if v.eq_ignore_ascii_case("off") => None,
        Some(v) if v.eq_ignore_ascii_case("dev") => IMAGES.iter().find(|(s, _)| *s == stage).map(|(_, image)| image.to_string()),
        Some(image) => Some(image),
        None => Some(studio_image()),
    }
}

fn docker_ready() -> bool {
    runs("docker", &["version", "--format", "{{.Server.Version}}"])
}

fn image_present(image: &str) -> bool {
    runs("docker", &["image", "inspect", image])
}

/// The command's own arguments as the container sees them: the model at
/// `/work/<its name>` (its directory is mounted there), a `--keep` directory
/// under the model's directory at its place under `/work`, the rest as given.
pub fn container_args(args: &[String], input: &Path) -> Result<Vec<String>, String> {
    let given = input.to_string_lossy().to_string();
    let name = input.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let dir = input.parent().filter(|p| !p.as_os_str().is_empty()).map(Path::to_path_buf).unwrap_or_else(|| Path::new(".").to_path_buf());
    let mut out = Vec::with_capacity(args.len());
    let mut keep_next = false;
    for a in args {
        if keep_next {
            keep_next = false;
            let keep = Path::new(a);
            // The directory may not exist yet: compare the paths as written, canonical where they exist.
            let canonical = |p: &Path| p.canonicalize().unwrap_or_else(|_| std::env::current_dir().map(|c| c.join(p)).unwrap_or_else(|_| p.to_path_buf()));
            let (model_dir, kept) = (canonical(&dir), canonical(keep));
            let inside = kept.strip_prefix(&model_dir).map(|rel| rel.to_path_buf()).map_err(|_| format!("--keep {}: when the command runs in a Docker image, the directory kept must be inside the model's directory {}", keep.display(), dir.display()))?;
            out.push(format!("/work/{}", inside.display()));
            continue;
        }
        if a == "--keep" {
            keep_next = true;
        }
        out.push(if *a == given { format!("/work/{name}") } else { a.clone() });
    }
    Ok(out)
}

/// Run this command in the stage's image when the host lacks the stage's
/// tools and Docker has the image: `Ok(Some(code))` with the container's
/// exit code, `Ok(None)` when the command runs here, `Err` when it can run
/// nowhere, saying what is missing.
pub fn relay(stage: &str, input: &Path) -> Result<Option<i32>, String> {
    let missing = missing_tools(stage);
    if missing.is_empty() {
        return Ok(None);
    }
    let Some(image) = image_of(stage) else { return Ok(None) };
    let what = missing.join(", ");
    if !docker_ready() {
        return Err(format!("{stage}: no {what} on this host, and no Docker daemon to run the image {image} (start Docker, or install the tools; ARCLANG_EMBEDDED_DOCKER=off keeps the command on the host)"));
    }
    if !image_present(&image) {
        if dev() {
            return Err(format!("{stage}: no {what} on this host, and the image {image} is not built: build it with tools/embedded/{}/run.sh (docs/embedded-targets.md), or ARCLANG_EMBEDDED_DOCKER names another", match stage { "pil" => "soc", "hdl" => "fpga", _ => "system" }));
        }
        eprintln!("{stage}: no {what} on this host: pulling {image} once (several GB: every tool of the chain, docs/studio.md)");
        let pulled = Command::new("docker").args(["pull", &image]).stdin(Stdio::null()).status().is_ok_and(|s| s.success());
        if !pulled {
            return Err(format!("{stage}: no {what} on this host, and the image {image} cannot be pulled (offline? build it with tools/studio/build.sh, docs/studio.md), or ARCLANG_EMBEDDED_DOCKER names another"));
        }
    }
    let dir = input.parent().filter(|p| !p.as_os_str().is_empty()).map(Path::to_path_buf).unwrap_or_else(|| Path::new(".").to_path_buf());
    let dir = dir.canonicalize().map_err(|e| format!("{}: {e}", dir.display()))?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args = container_args(&args, input)?;
    eprintln!("{stage}: no {what} on this host: running in the Docker image {image} ({} mounted at /work)", dir.display());
    let mut command = Command::new("docker");
    command.args(["run", "--rm", "-i"]).arg("-v").arg(format!("{}:/work", dir.display()));
    command.args(["-w", "/work", "-e", "ARCLANG_LITEX_PYTHON=/opt/litex/bin/python", "-e", "ARCLANG_EMBEDDED_DOCKER=off"]);
    // As this host's user: what the stage writes beside the model is the user's.
    if let Some(user) = host_user() {
        command.args(["--user", &user, "-e", "HOME=/tmp"]);
    }
    let script = if dev() {
        // The compiler its build loop left in the target volume.
        command.args(["-v", "arclang-embedded-target:/target"]);
        "if [ -x /target/release/arclang ]; then exec /target/release/arclang \"$@\"; else echo 'no arclang in the target volume: build it with tools/embedded/<image>/run.sh cargo build --release' >&2; exit 2; fi"
    } else {
        "mkdir -p \"$HOME/.cache\"; if command -v arclang >/dev/null 2>&1; then exec arclang \"$@\"; else echo 'no arclang in the image' >&2; exit 2; fi"
    };
    let status = command.arg("--entrypoint").arg("sh").arg(&image).args(["-c", script, "sh"]).args(&args).status().map_err(|e| format!("docker run {image}: {e}"))?;
    Ok(Some(status.code().unwrap_or(1)))
}

/// `uid:gid` of this host's user, on a Unix host.
fn host_user() -> Option<String> {
    if cfg!(windows) {
        return None;
    }
    let id = |flag: &str| Command::new("id").arg(flag).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).filter(|v| !v.is_empty());
    Some(format!("{}:{}", id("-u")?, id("-g")?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_container_sees_the_model_under_work_and_the_rest_as_given() {
        let args: Vec<String> = ["pil", "examples/x/model.arc", "--test", "TC-1", "--keep", "out"].iter().map(|s| s.to_string()).collect();
        let mapped = container_args(&args, Path::new("examples/x/model.arc"));
        assert!(mapped.unwrap_err().contains("must be inside the model's directory"), "a kept directory outside the model's is refused");
        let args: Vec<String> = ["hdl", "examples/x/model.arc", "--synth", "--keep", "examples/x/keep/synth"].iter().map(|s| s.to_string()).collect();
        let mapped = container_args(&args, Path::new("examples/x/model.arc")).unwrap();
        assert_eq!(mapped, vec!["hdl", "/work/model.arc", "--synth", "--keep", "/work/keep/synth"]);
    }

    #[test]
    fn off_keeps_every_stage_on_the_host_and_each_stage_has_its_image() {
        assert_eq!(IMAGES.iter().map(|(s, _)| *s).collect::<Vec<_>>(), vec!["pil", "hdl", "soc"]);
        std::env::set_var("ARCLANG_EMBEDDED_DOCKER", "off");
        assert_eq!(image_of("pil"), None);
        std::env::set_var("ARCLANG_EMBEDDED_DOCKER", "my/image:2");
        assert_eq!(image_of("hdl").as_deref(), Some("my/image:2"));
        std::env::set_var("ARCLANG_EMBEDDED_DOCKER", "dev");
        assert_eq!(image_of("soc").as_deref(), Some("arclang-embedded-system:1"));
        std::env::remove_var("ARCLANG_EMBEDDED_DOCKER");
        assert_eq!(image_of("pil"), Some(format!("ghcr.io/mbaroudi/arclang-studio:{}", env!("CARGO_PKG_VERSION"))), "by default ArcLang Studio of this version, for every stage");
        assert_eq!(image_of("soc"), image_of("hdl"));
    }
}
