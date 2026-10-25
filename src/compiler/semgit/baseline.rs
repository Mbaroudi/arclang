//! Baselines: the state of a model an auditor (ISO 26262, DO-178C) accepts.
//!
//! `baselines/<name>/` holds the gate report, the requirement coverage, the
//! exports and a manifest that binds them to the git commit, the compiler
//! and the identities of the model, with the SHA-256 of every file. An
//! annotated tag `baseline/<name>` marks the commit; `verify` proves that
//! nothing moved since.

use super::git;
use super::rich::{self, RichDiff, Source};
use crate::compiler::production_gate::{self, GateReport, Severity};
use crate::compiler::tables::{self, export, TableSpec};
use crate::compiler::{elements, CompilationResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub const DIRECTORY: &str = "baselines";
pub const TAG_PREFIX: &str = "baseline/";
pub const STANDARDS: [&str; 3] = ["iso26262", "do178c", "iec61508"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub id: String,
    pub uuid: String,
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub commit: String,
    pub branch: String,
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateSummary {
    pub standard: String,
    pub passed: bool,
    pub blockers: usize,
    pub warnings: usize,
    pub requirements_total: usize,
    pub requirements_satisfied: usize,
    pub requirements_verified: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub arclang_baseline: String,
    pub name: String,
    pub standard: String,
    pub created: String,
    pub compiler_version: String,
    pub git: GitInfo,
    /// The model file, relative to the repository.
    pub model: String,
    /// The model and what it imports, as committed.
    pub sources: Vec<FileEntry>,
    pub gate: GateSummary,
    pub identities_sha256: String,
    pub identities: Vec<Identity>,
    /// The files of the baseline directory (the manifest itself excluded).
    pub files: Vec<FileEntry>,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

pub fn standard_name(text: &str) -> Result<(&'static str, &'static str), String> {
    match text.to_ascii_lowercase().replace(['-', '_', ' '], "").as_str() {
        "iso26262" => Ok(("iso26262", "ISO26262")),
        "do178c" => Ok(("do178c", "DO178C")),
        "iec61508" => Ok(("iec61508", "IEC61508")),
        other => Err(format!("unknown standard '{other}': one of {}", STANDARDS.join(", "))),
    }
}

pub fn valid_name(name: &str) -> Result<(), String> {
    let ok = !name.is_empty() && name.len() <= 100 && !name.starts_with(['.', '-']) && !name.contains("..") && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    if ok {
        Ok(())
    } else {
        Err(format!("'{name}' is not a baseline name: letters, digits, '.', '_' and '-' only"))
    }
}

fn repo_of(dir: &Path) -> Result<PathBuf, String> {
    let top = git::run(dir, &["rev-parse", "--show-toplevel"]).map_err(|_| format!("{} is not in a git repository", dir.display()))?;
    std::fs::canonicalize(top.trim()).map_err(|e| e.to_string())
}

/// Tracked files modified, staged or deleted, and model files not tracked yet.
pub fn dirty_files(top: &Path) -> Result<Vec<String>, String> {
    let out = git::run(top, &["status", "--porcelain=v1", "--untracked-files=all"])?;
    Ok(out
        .lines()
        .filter_map(|line| {
            let (status, path) = (line.get(..2)?, line.get(3..)?.trim().trim_matches('"').to_string());
            if path.starts_with(&format!("{DIRECTORY}/")) {
                return None;
            }
            if status == "??" && !(path.ends_with(".arc") || path.ends_with(".layout.json")) {
                return None;
            }
            Some(format!("{} {}", status.trim(), path))
        })
        .collect())
}

fn gate_markdown(name: &str, report: &GateReport, model: &str) -> String {
    let mut out = format!(
        "# Production gate: {}\n\nBaseline `{name}`, model `{model}`, standard {}.\n\n**Verdict: {}**\n\n- Requirements: {} total, {} satisfied, {} verified\n- Constraints: {} declared, {} hold\n",
        if report.passed { "PASS" } else { "FAIL" },
        report.standard,
        if report.passed { "PASS" } else { "FAIL" },
        report.requirements_total,
        report.requirements_satisfied,
        report.requirements_verified,
        report.constraints_total,
        report.constraints_satisfied
    );
    for (title, severity) in [("Blockers", Severity::Blocker), ("Warnings", Severity::Warning)] {
        let items: Vec<_> = report.findings.iter().filter(|f| f.severity == severity).collect();
        if !items.is_empty() {
            out.push_str(&format!("\n## {title}\n\n"));
            for f in items {
                out.push_str(&format!("- [{}] {}\n", f.check, f.message));
            }
        }
    }
    out
}

fn identities_of(result: &CompilationResult) -> Vec<Identity> {
    let graph = elements::build(&result.ast, &result.semantic_model);
    let mut all: Vec<Identity> = graph.elements.iter().map(|e| Identity { id: e.id.clone(), uuid: e.uuid.clone(), kind: e.kind.to_string(), name: e.name.clone() }).collect();
    all.extend(graph.relationships.iter().map(|r| Identity { id: String::new(), uuid: r.uuid.clone(), kind: r.kind.to_string(), name: r.name.clone().unwrap_or_default() }));
    all.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    all
}

fn identities_hash(identities: &[Identity]) -> String {
    sha256_hex(identities.iter().map(|i| i.uuid.as_str()).collect::<Vec<_>>().join("\n").as_bytes())
}

fn entry(dir: &Path, relative: &str) -> Result<FileEntry, String> {
    let bytes = std::fs::read(dir.join(relative)).map_err(|e| format!("{relative}: {e}"))?;
    Ok(FileEntry { path: relative.to_string(), sha256: sha256_hex(&bytes), bytes: bytes.len() as u64 })
}

pub struct Created {
    pub manifest: Manifest,
    pub directory: PathBuf,
    pub tag: String,
}

pub struct CreateOptions<'a> {
    pub name: &'a str,
    pub standard: &'a str,
    pub model: &'a Path,
    pub require_pass: bool,
    pub allow_dirty: bool,
}

pub fn create(options: &CreateOptions) -> Result<Created, String> {
    valid_name(options.name)?;
    let (standard_key, standard) = standard_name(options.standard)?;
    let located = git::locate(options.model)?;
    let top = located.top.clone();
    if !options.allow_dirty {
        let dirty = dirty_files(&top)?;
        if !dirty.is_empty() {
            return Err(format!("the working tree is not clean; commit or stash first:\n  {}", dirty.iter().take(10).cloned().collect::<Vec<_>>().join("\n  ")));
        }
    }
    let tag = format!("{TAG_PREFIX}{}", options.name);
    if git::run(&top, &["rev-parse", "--verify", "--quiet", &format!("refs/tags/{tag}")]).is_ok() {
        return Err(format!("the tag {tag} already exists: a baseline is never overwritten"));
    }
    let dir = top.join(DIRECTORY).join(options.name);
    if dir.exists() {
        return Err(format!("{} already exists: a baseline is never overwritten", dir.display()));
    }
    let commit = git::run(&top, &["rev-parse", "HEAD"])?.trim().to_string();
    let branch = git::run(&top, &["rev-parse", "--abbrev-ref", "HEAD"])?.trim().to_string();

    let result = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(options.model).map_err(|e| format!("the model does not compile: {e}"))?;
    let report = production_gate::run_gate(&result.ast, &result.semantic_model, standard);
    if options.require_pass && !report.passed {
        return Err(format!("the production gate fails ({} blocker(s)); a baseline was required to pass", report.findings.iter().filter(|f| f.severity == Severity::Blocker).count()));
    }

    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let write = |name: &str, text: String| std::fs::write(dir.join(name), text).map_err(|e| format!("{name}: {e}"));
    let model_label = located.relative.clone();
    write("gate_report.json", format!("{}\n", serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?))?;
    write("gate_report.md", gate_markdown(options.name, &report, &model_label))?;
    let table = tables::compute(&result, &TableSpec { kind: "requirements_coverage".into(), ..TableSpec::default() }, &tables::Options { gate_standard: standard.to_string() })?;
    write("requirements_coverage.csv", export::csv(&table))?;
    write("model.sysml", crate::compiler::sysml_export::export(&result.ast))?;
    write("model.json", format!("{}\n", result.ast.to_json().map_err(|e| e.to_string())?))?;

    let identities = identities_of(&result);
    let mut sources = Vec::new();
    for file in crate::Compiler::files_of(options.model).map_err(|e| e.to_string())? {
        let relative = file.strip_prefix(&top).map_err(|_| format!("{} is outside the repository", file.display()))?.to_string_lossy().replace('\\', "/");
        sources.push(entry(&top, &relative)?);
    }
    if let Ok(layout) = entry(&top, &super::git::sidecar_relative(&located.relative)) {
        sources.push(layout);
    }
    let mut files = Vec::new();
    for name in ["gate_report.json", "gate_report.md", "requirements_coverage.csv", "model.sysml", "model.json"] {
        files.push(entry(&dir, name)?);
    }
    let manifest = Manifest {
        arclang_baseline: "1".into(),
        name: options.name.to_string(),
        standard: standard_key.to_string(),
        created: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        compiler_version: env!("CARGO_PKG_VERSION").to_string(),
        git: GitInfo { commit: commit.clone(), branch, tag: tag.clone() },
        model: model_label,
        sources,
        gate: GateSummary {
            standard: standard.to_string(),
            passed: report.passed,
            blockers: report.findings.iter().filter(|f| f.severity == Severity::Blocker).count(),
            warnings: report.findings.iter().filter(|f| f.severity == Severity::Warning).count(),
            requirements_total: report.requirements_total,
            requirements_satisfied: report.requirements_satisfied,
            requirements_verified: report.requirements_verified,
        },
        identities_sha256: identities_hash(&identities),
        identities,
        files,
    };
    let text = format!("{}\n", serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?);
    std::fs::write(dir.join("manifest.json"), &text).map_err(|e| e.to_string())?;
    let message = format!(
        "ArcLang baseline {} ({standard}), gate {}, manifest sha256 {}",
        options.name,
        if report.passed { "PASS" } else { "FAIL" },
        sha256_hex(text.as_bytes())
    );
    git::run(&top, &["tag", "-a", &tag, "-m", &message, &commit])?;
    Ok(Created { manifest, directory: dir, tag })
}

fn read_manifest(top: &Path, name: &str) -> Result<(Manifest, String, PathBuf), String> {
    valid_name(name)?;
    let dir = top.join(DIRECTORY).join(name);
    let text = std::fs::read_to_string(dir.join("manifest.json")).map_err(|_| format!("no baseline '{name}' (no {DIRECTORY}/{name}/manifest.json)"))?;
    let manifest: Manifest = serde_json::from_str(&text).map_err(|e| format!("the manifest of '{name}' is not readable: {e}"))?;
    Ok((manifest, text, dir))
}

#[derive(Debug, Clone, Serialize)]
pub struct Check {
    pub check: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Verification {
    pub name: String,
    pub ok: bool,
    pub checks: Vec<Check>,
}

pub fn verify(dir: &Path, name: &str) -> Result<Verification, String> {
    let top = repo_of(dir)?;
    let (manifest, manifest_text, bdir) = read_manifest(&top, name)?;
    let mut checks = Vec::new();
    let mut add = |check: &str, ok: bool, detail: String| checks.push(Check { check: check.to_string(), ok, detail });
    for f in &manifest.files {
        match std::fs::read(bdir.join(&f.path)) {
            Ok(bytes) => {
                let sum = sha256_hex(&bytes);
                add(&format!("file {}", f.path), sum == f.sha256, if sum == f.sha256 { "matches the manifest".into() } else { format!("changed: sha256 is {sum}, the manifest says {}", f.sha256) });
            }
            Err(_) => add(&format!("file {}", f.path), false, "missing".into()),
        }
    }
    let tag = &manifest.git.tag;
    let tag_ref = format!("refs/tags/{tag}");
    match git::run(&top, &["cat-file", "-t", &tag_ref]) {
        Ok(kind) if kind.trim() == "tag" => {
            add("tag", true, format!("{tag} is an annotated tag"));
            let target = git::run(&top, &["rev-parse", &format!("{tag_ref}^{{commit}}")]).map(|s| s.trim().to_string()).unwrap_or_default();
            add("tag commit", target == manifest.git.commit, if target == manifest.git.commit { format!("{tag} points at {}", &manifest.git.commit[..manifest.git.commit.len().min(10)]) } else { format!("{tag} points at {target}, the manifest says {}", manifest.git.commit) });
            let message = git::run(&top, &["tag", "-l", "--format=%(contents)", tag]).unwrap_or_default();
            let sum = sha256_hex(manifest_text.as_bytes());
            add("manifest", message.contains(&sum), if message.contains(&sum) { "the manifest is the one the tag was made with".into() } else { "the manifest is not the one the tag was made with".into() });
        }
        Ok(_) => add("tag", false, format!("{tag} is not an annotated tag")),
        Err(_) => add("tag", false, format!("the tag {tag} does not exist")),
    }
    for source in &manifest.sources {
        let committed = git::run(&top, &["show", &format!("{}:{}", manifest.git.commit, source.path)]);
        match committed {
            Ok(text) => {
                let sum = sha256_hex(text.as_bytes());
                add(&format!("source {}", source.path), sum == source.sha256, if sum == source.sha256 { "the committed file matches".into() } else { "the committed file differs from the manifest".into() });
            }
            Err(_) => add(&format!("source {}", source.path), false, format!("not in commit {}", manifest.git.commit)),
        }
    }
    // The identities of the model as committed.
    match rich::load(&top.join(&manifest.model), &Source::Git(manifest.git.commit.clone())) {
        Ok(loaded) => {
            let now = identities_hash(&identities_of(&loaded.result));
            add("identities", now == manifest.identities_sha256, if now == manifest.identities_sha256 { format!("{} identities match", manifest.identities.len()) } else { "the identities of the model at that commit differ from the manifest".into() });
        }
        Err(e) => add("identities", false, e),
    }
    let ok = checks.iter().all(|c| c.ok);
    Ok(Verification { name: name.to_string(), ok, checks })
}

#[derive(Debug, Clone, Serialize)]
pub struct Listed {
    pub name: String,
    pub tag: bool,
    pub created: String,
    pub standard: String,
    pub commit: String,
    pub gate: String,
}

pub fn list(dir: &Path) -> Result<Vec<Listed>, String> {
    let top = repo_of(dir)?;
    let mut names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    if let Ok(entries) = std::fs::read_dir(top.join(DIRECTORY)) {
        names.extend(entries.flatten().filter(|e| e.path().join("manifest.json").exists()).map(|e| e.file_name().to_string_lossy().to_string()));
    }
    let tags = git::run(&top, &["tag", "-l", &format!("{TAG_PREFIX}*")]).unwrap_or_default();
    let tagged: std::collections::BTreeSet<String> = tags.lines().filter_map(|t| t.strip_prefix(TAG_PREFIX)).map(str::to_string).collect();
    names.extend(tagged.iter().cloned());
    Ok(names
        .into_iter()
        .map(|name| match read_manifest(&top, &name) {
            Ok((m, _, _)) => Listed { tag: tagged.contains(&name), created: m.created, standard: m.standard, commit: m.git.commit, gate: if m.gate.passed { "PASS".into() } else { format!("FAIL ({} blockers)", m.gate.blockers) }, name },
            Err(_) => {
                let commit = git::run(&top, &["rev-parse", &format!("{TAG_PREFIX}{name}^{{commit}}")]).map(|s| s.trim().to_string()).unwrap_or_default();
                Listed { tag: true, created: String::new(), standard: String::new(), commit, gate: "(no manifest in this checkout)".into(), name }
            }
        })
        .collect())
}

#[derive(Debug, Serialize)]
pub struct BaselineDiff {
    pub a: String,
    pub b: String,
    pub gate_a: Option<GateSummary>,
    pub gate_b: Option<GateSummary>,
    pub diff: RichDiff,
}

pub fn diff(dir: &Path, a: &str, b: &str) -> Result<BaselineDiff, String> {
    let top = repo_of(dir)?;
    let side = |name: &str| -> Result<(Option<Manifest>, String), String> {
        match read_manifest(&top, name) {
            Ok((m, _, _)) => {
                let commit = m.git.commit.clone();
                Ok((Some(m), commit))
            }
            Err(_) => {
                let commit = git::commit_of(&top, &format!("{TAG_PREFIX}{name}")).map_err(|_| format!("no baseline '{name}'"))?;
                Ok((None, commit))
            }
        }
    };
    let ((ma, ca), (mb, cb)) = (side(a)?, side(b)?);
    let model_rel = mb.as_ref().or(ma.as_ref()).map(|m| m.model.clone()).ok_or("the baselines name no model")?;
    let model = top.join(&model_rel);
    let standard = mb.as_ref().map(|m| m.gate.standard.clone()).unwrap_or_else(|| "ISO26262".into());
    let mut rich_diff = rich::diff_sources(&model, &Source::Git(ca), &Source::Git(cb), &standard)?;
    rich_diff.from = a.to_string();
    rich_diff.to = b.to_string();
    Ok(BaselineDiff { a: a.to_string(), b: b.to_string(), gate_a: ma.map(|m| m.gate), gate_b: mb.map(|m| m.gate), diff: rich_diff })
}
