//! Where the Node renderer of the SVG diagrams (`arclang diagram -f
//! component`, ...) lives: `$ARCLANG_DIAGRAM_SERVICE`, else the
//! `arcviz-web/apps/diagram-service` of a checkout above the binary or the
//! working directory. It is a built Node project (`dist/`): a compiler
//! installed alone has none, and says so.

use std::path::{Path, PathBuf};

/// The environment variable that names the renderer's directory.
pub const ENV: &str = "ARCLANG_DIAGRAM_SERVICE";

const RELATIVE: &str = "arcviz-web/apps/diagram-service";

/// Whether `dir` is a built renderer: its sources alone do not render.
fn built(dir: &Path) -> bool {
    dir.join("dist").is_dir()
}

/// The renderer's directory: `configured` when given, else the first
/// directory above one of `starts` that holds a built one.
pub fn find(configured: Option<PathBuf>, starts: &[PathBuf]) -> Option<PathBuf> {
    if let Some(dir) = configured {
        return built(&dir).then_some(dir);
    }
    starts.iter().flat_map(|start| start.ancestors().map(Path::to_path_buf).collect::<Vec<_>>()).map(|dir| dir.join(RELATIVE)).find(|dir| built(dir))
}

/// The renderer's directory for this process.
pub fn locate() -> Option<PathBuf> {
    let configured = std::env::var_os(ENV).map(PathBuf::from);
    let mut starts = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        starts.extend(exe.parent().map(Path::to_path_buf));
    }
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd);
    }
    find(configured, &starts)
}

/// Why there is no renderer, and how to give one.
pub fn missing() -> String {
    format!("the SVG diagram renderer was not found: set {ENV} to a built {RELATIVE} (npm install && npm run build), or use -f html, viewpoints or elk, which need nothing else")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("arclang-diagram-service-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_renderer_of_a_checkout_above_the_binary_is_found() {
        let root = temp("checkout");
        std::fs::create_dir_all(root.join(RELATIVE).join("dist")).unwrap();
        let binary_dir = root.join("target/release");
        std::fs::create_dir_all(&binary_dir).unwrap();
        assert_eq!(find(None, &[binary_dir]), Some(root.join(RELATIVE)));
    }

    #[test]
    fn a_configured_directory_wins_and_must_exist() {
        let root = temp("configured");
        std::fs::create_dir_all(root.join("dist")).unwrap();
        assert_eq!(find(Some(root.clone()), &[]), Some(root.clone()));
        assert_eq!(find(Some(root.join("missing")), &[root.clone()]), None);
    }

    #[test]
    fn a_renderer_that_was_never_built_is_not_found() {
        let root = temp("unbuilt");
        std::fs::create_dir_all(root.join(RELATIVE)).unwrap();
        assert_eq!(find(None, &[root]), None);
    }

    #[test]
    fn without_a_checkout_there_is_none() {
        assert_eq!(find(None, &[temp("alone")]), None);
    }
}
