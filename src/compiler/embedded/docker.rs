//! The embedded stages through Docker. `arclang pil`, `hdl` and `soc` need
//! toolchains a host rarely has: Zephyr and Renode, GHDL and Yosys, LiteX
//! and Verilator. The images under `tools/embedded` hold them. When this
//! host lacks a stage's tools and Docker has the stage's image, the command
//! runs itself in the image, the model's directory mounted at `/work`, and
//! its verdict lands in the model's cache as if it had run here: `check`,
//! the ladder and `realize` read it the same way. `ARCLANG_EMBEDDED_DOCKER`
//! names another image (one for every stage), or `off` keeps everything on
//! the host.

use std::path::Path;
use std::process::{Command, Stdio};

/// The image of each stage, as `tools/embedded/*/run.sh` build them.
pub const IMAGES: [(&str, &str); 3] = [("pil", "arclang-embedded-soc:1"), ("hdl", "arclang-embedded-fpga:1"), ("soc", "arclang-embedded-system:1")];

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

/// The image a stage runs in: the stage's own, or `ARCLANG_EMBEDDED_DOCKER`;
/// none when that is `off`.
pub fn image_of(stage: &str) -> Option<String> {
    match std::env::var("ARCLANG_EMBEDDED_DOCKER").ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
        Some(v) if v.eq_ignore_ascii_case("off") => None,
        Some(image) => Some(image),
        None => IMAGES.iter().find(|(s, _)| *s == stage).map(|(_, image)| image.to_string()),
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
        return Err(format!("{stage}: no {what} on this host, and the image {image} is not built: build it with tools/embedded/{}/run.sh (docs/embedded-targets.md), or ARCLANG_EMBEDDED_DOCKER names another", match stage { "pil" => "soc", "hdl" => "fpga", _ => "system" }));
    }
    let dir = input.parent().filter(|p| !p.as_os_str().is_empty()).map(Path::to_path_buf).unwrap_or_else(|| Path::new(".").to_path_buf());
    let dir = dir.canonicalize().map_err(|e| format!("{}: {e}", dir.display()))?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args = container_args(&args, input)?;
    eprintln!("{stage}: no {what} on this host: running in the Docker image {image} ({} mounted at /work)", dir.display());
    // The arclang of the image: the one its build loop left in the target volume, else one on its PATH.
    let script = "if [ -x /target/release/arclang ]; then exec /target/release/arclang \"$@\"; elif command -v arclang >/dev/null 2>&1; then exec arclang \"$@\"; else echo 'no arclang in the image: build it with tools/embedded/<image>/run.sh cargo build --release' >&2; exit 2; fi";
    let status = Command::new("docker")
        .args(["run", "--rm", "-i"])
        .arg("-v")
        .arg(format!("{}:/work", dir.display()))
        .args(["-v", "arclang-embedded-target:/target", "-w", "/work", "-e", "ARCLANG_LITEX_PYTHON=/opt/litex/bin/python"])
        .arg(&image)
        .args(["sh", "-c", script, "sh"])
        .args(&args)
        .status()
        .map_err(|e| format!("docker run {image}: {e}"))?;
    Ok(Some(status.code().unwrap_or(1)))
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
        std::env::remove_var("ARCLANG_EMBEDDED_DOCKER");
        assert_eq!(image_of("soc").as_deref(), Some("arclang-embedded-system:1"));
    }
}
