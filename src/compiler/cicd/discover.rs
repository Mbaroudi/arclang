//! Which `.arc` files are models. A file that another one imports is a
//! fragment; the others are the entry points a pipeline checks.

use crate::compiler::semgit::git;
use regex::Regex;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const SKIPPED_DIRS: [&str; 5] = ["target", "node_modules", "baselines", ".git", ".arclang"];

/// Every `.arc` file under `root`: those git knows (tracked, or new and not
/// ignored) when `root` is in a repository, otherwise a walk.
pub fn arc_files(root: &Path) -> Vec<PathBuf> {
    let listed = git::run(root, &["ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", "*.arc"]).ok();
    let mut files: Vec<PathBuf> = match listed {
        Some(out) => out.split('\0').filter(|p| !p.is_empty()).map(|p| root.join(p)).filter(|p| p.is_file()).collect(),
        None => {
            let mut found = Vec::new();
            walk(root, &mut found);
            found
        }
    };
    files.sort();
    files.dedup();
    files
}

fn walk(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if !SKIPPED_DIRS.contains(&name.as_str()) && !name.starts_with('.') {
                walk(&path, found);
            }
        } else if path.extension().is_some_and(|e| e == "arc") {
            found.push(path);
        }
    }
}

/// The files a file imports, resolved beside it.
pub fn imports_of(file: &Path) -> Vec<PathBuf> {
    let pattern = Regex::new(r#"(?m)^\s*import\s+"([^"]+)""#).expect("a valid pattern");
    let Ok(text) = std::fs::read_to_string(file) else { return Vec::new() };
    let base = file.parent().unwrap_or(Path::new("."));
    pattern.captures_iter(&text).map(|c| base.join(&c[1])).collect()
}

fn normal(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// The entry points among `files`: not imported by another of them.
pub fn entry_points(files: &[PathBuf]) -> Vec<PathBuf> {
    let imported: BTreeSet<PathBuf> = files.iter().flat_map(|f| imports_of(f)).map(|p| normal(&p)).collect();
    files.iter().filter(|f| !imported.contains(&normal(f))).cloned().collect()
}

/// The models of a tree.
pub fn models(root: &Path) -> Vec<PathBuf> {
    entry_points(&arc_files(root))
}
