//! `arclang merge --install`: the git configuration that makes `git merge`
//! use the semantic merge for `.arc` and `.layout.json` files.

use crate::compiler::semgit::git;
use std::path::{Path, PathBuf};

pub const DRIVER_NAME: &str = "arclang";
pub const ATTRIBUTE_LINES: [&str; 2] = ["*.arc merge=arclang", "*.layout.json merge=arclang"];

#[derive(Debug, Clone)]
pub struct Plan {
    pub top: PathBuf,
    pub config: Vec<(String, String, Option<String>)>,
    pub attributes_file: PathBuf,
    pub attributes_missing: Vec<String>,
}

fn quote_if_needed(binary: &str) -> String {
    if binary.contains(' ') {
        format!("\"{binary}\"")
    } else {
        binary.to_string()
    }
}

pub fn driver_command(binary: &str) -> String {
    format!("{} merge %O %A %B -o %A --path %P", quote_if_needed(binary))
}

pub fn plan(dir: &Path, binary: &str) -> Result<Plan, String> {
    let top = git::run(dir, &["rev-parse", "--show-toplevel"]).map_err(|_| format!("{} is not in a git repository", dir.display()))?;
    let top = PathBuf::from(top.trim());
    let current = |key: &str| git::run(&top, &["config", "--local", "--get", key]).ok().map(|s| s.trim().to_string());
    let wanted = [
        ("merge.arclang.name", "ArcLang semantic merge".to_string()),
        ("merge.arclang.driver", driver_command(binary)),
    ];
    let config = wanted.iter().map(|(k, v)| (k.to_string(), v.clone(), current(k))).collect();
    let attributes_file = top.join(".gitattributes");
    let existing = std::fs::read_to_string(&attributes_file).unwrap_or_default();
    let attributes_missing = ATTRIBUTE_LINES
        .iter()
        .filter(|line| !existing.lines().any(|l| l.split_whitespace().collect::<Vec<_>>() == line.split_whitespace().collect::<Vec<_>>()))
        .map(|l| l.to_string())
        .collect();
    Ok(Plan { top, config, attributes_file, attributes_missing })
}

impl Plan {
    pub fn is_noop(&self) -> bool {
        self.attributes_missing.is_empty() && self.config.iter().all(|(_, want, have)| have.as_deref() == Some(want.as_str()))
    }

    pub fn describe(&self) -> String {
        let mut out = format!("Repository: {}\n\nGit configuration (.git/config):\n", self.top.display());
        for (key, want, have) in &self.config {
            match have {
                Some(h) if h == want => out.push_str(&format!("  = {key} = {want} (already set)\n")),
                Some(h) => out.push_str(&format!("  ~ {key}: {h} -> {want}\n")),
                None => out.push_str(&format!("  + {key} = {want}\n")),
            }
        }
        out.push_str(&format!("\nAttributes ({}):\n", self.attributes_file.display()));
        for line in ATTRIBUTE_LINES {
            if self.attributes_missing.iter().any(|m| m == line) {
                out.push_str(&format!("  + {line}\n"));
            } else {
                out.push_str(&format!("  = {line} (already there)\n"));
            }
        }
        out
    }

    pub fn apply(&self) -> Result<(), String> {
        for (key, want, _) in &self.config {
            git::run(&self.top, &["config", "--local", key, want])?;
        }
        if !self.attributes_missing.is_empty() {
            let mut text = std::fs::read_to_string(&self.attributes_file).unwrap_or_default();
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            for line in &self.attributes_missing {
                text.push_str(line);
                text.push('\n');
            }
            std::fs::write(&self.attributes_file, text).map_err(|e| format!("{}: {e}", self.attributes_file.display()))?;
        }
        Ok(())
    }
}
