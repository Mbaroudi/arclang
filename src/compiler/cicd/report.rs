//! The pull-request report: for each model, the semantic diff between the
//! base and the head of a pull request, what the change reaches, the
//! production gate before and after (new blockers named), the coverage, the
//! budgets, the simulation verdicts, and the picture of the model before
//! and after. Written for a comment: Markdown, with a hidden marker so that
//! a pipeline updates its one comment instead of adding another.
//!
//! The verdict is the gate's: a model that does not compile, or whose gate
//! fails, is FAIL whatever else the report says.

use super::diag::standard_name;
use super::discover;
use super::svg::{self, Side};
use crate::compiler::production_gate::{self, Severity};
use crate::compiler::semgit::git;
use crate::compiler::semgit::render::{element_details, field_line, headline, ms, relation_line, truth};
use crate::compiler::semgit::rich::{self, Loaded, RichDiff, Source};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// The hidden marker a pipeline looks for to update its comment.
pub const MARKER: &str = "<!-- arclang-pr-report -->";
/// GitHub refuses a comment longer than 65536 characters.
const COMMENT_LIMIT: usize = 60_000;
const ELEMENTS_LISTED: usize = 40;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailOn {
    /// A blocker the change introduces, or a model that no longer compiles.
    Blocker,
    /// The above, and a regression: a requirement no longer satisfied or verified,
    /// a constraint that flips to violated, a budget newly exceeded, a verdict that stops passing.
    Major,
    Never,
}

impl FailOn {
    pub fn parse(text: &str) -> Option<FailOn> {
        match text {
            "blocker" => Some(FailOn::Blocker),
            "major" => Some(FailOn::Major),
            "never" => Some(FailOn::Never),
            _ => None,
        }
    }
}

pub struct Options {
    pub from: String,
    /// `None`: the working tree.
    pub to: Option<String>,
    pub standard: String,
    pub fail_on: FailOn,
    pub models: Vec<PathBuf>,
    /// Where the SVG pictures are written; `None`: none are.
    pub images_dir: Option<PathBuf>,
    /// The directory the report is read from, so that the pictures are linked by relative path.
    pub link_base: PathBuf,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GateView {
    pub standard: String,
    pub passed: bool,
    /// `[check] message`, as `arclang gate` prints them.
    pub blockers: Vec<String>,
    pub requirements_total: usize,
    pub requirements_satisfied: usize,
    pub requirements_verified: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SideView {
    /// The model exists at this revision.
    pub present: bool,
    pub compiled: bool,
    pub error: Option<String>,
    pub gate: Option<GateView>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Images {
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelReport {
    pub model: String,
    pub base: SideView,
    pub head: SideView,
    pub new_blockers: Vec<String>,
    pub resolved_blockers: Vec<String>,
    /// What got worse besides the blockers (see `--fail-on major`).
    pub regressions: Vec<String>,
    /// `pass` only when the head compiles and its gate passes.
    pub status: &'static str,
    /// This model makes the exit status non-zero under the chosen `--fail-on`.
    pub fails_the_check: bool,
    pub diff: Option<RichDiff>,
    pub images: Option<Images>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub marker: &'static str,
    pub tool_version: &'static str,
    pub from: String,
    pub to: String,
    pub from_commit: String,
    pub to_commit: Option<String>,
    pub standard: String,
    pub fail_on: &'static str,
    /// `pass`, `fail`, or `none` when there is no model to examine.
    pub status: &'static str,
    pub models: Vec<ModelReport>,
    /// `.arc` files at the base that are no longer there.
    pub removed_files: Vec<String>,
    pub fails_the_check: bool,
    pub note: Option<String>,
}

fn gate_view(loaded: &Loaded, standard: &str) -> GateView {
    let report = production_gate::run_gate(&loaded.result.ast, &loaded.result.semantic_model, standard);
    GateView {
        standard: standard.to_string(),
        passed: report.passed,
        blockers: report.findings.iter().filter(|f| f.severity == Severity::Blocker).map(|f| format!("[{}] {}", f.check, f.message)).collect(),
        requirements_total: report.requirements_total,
        requirements_satisfied: report.requirements_satisfied,
        requirements_verified: report.requirements_verified,
    }
}

fn exists_at(model: &Path, source: &Source) -> bool {
    match source {
        Source::Worktree | Source::File(_) => model.exists(),
        Source::Git(revision) => git::locate(model)
            .ok()
            .and_then(|l| git::commit_of(&l.top, revision).ok().and_then(|c| git::show(&l.top, &c, &l.relative)))
            .is_some(),
    }
}

/// One side of a model: loaded or the reason it is not.
fn side(model: &Path, source: &Source, standard: &str) -> (SideView, Option<Loaded>) {
    if !exists_at(model, source) {
        return (SideView { present: false, compiled: false, error: None, gate: None }, None);
    }
    match rich::load(model, source) {
        Ok(loaded) => {
            let gate = gate_view(&loaded, standard);
            (SideView { present: true, compiled: true, error: None, gate: Some(gate) }, Some(loaded))
        }
        Err(error) => (SideView { present: true, compiled: false, error: Some(error), gate: None }, None),
    }
}

fn regressions_of(diff: &RichDiff) -> Vec<String> {
    let c = &diff.consequences;
    let mut out = Vec::new();
    for x in &c.newly_unsatisfied {
        out.push(format!("requirement {} '{}' is no longer satisfied ({} -> {})", x.requirement, x.name, x.before.status, x.after.status));
    }
    for x in &c.newly_unverified {
        out.push(format!("requirement {} '{}' is no longer verified ({} -> {})", x.requirement, x.name, x.before.status, x.after.status));
    }
    for k in c.constraints.iter().filter(|k| k.before == Some(true) && k.after == Some(false)) {
        out.push(format!("constraint {} no longer holds ({})", k.id, k.expression));
    }
    for b in c.budgets.iter().filter(|b| b.ok_before && !b.ok_after) {
        out.push(format!("budget {} is exceeded ({} ms for {} ms)", b.name, ms(b.after_ms), ms(b.budget_ms)));
    }
    for v in c.verdicts.iter().filter(|v| v.before == "pass" && v.after != "pass") {
        out.push(format!("simulation {} no longer passes ({} -> {})", v.test, v.before, v.after));
    }
    out
}

fn slug(model: &str) -> String {
    model.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect::<String>().trim_matches('-').to_string()
}

fn relative_to(path: &Path, base: &Path) -> String {
    let path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let base = std::fs::canonicalize(base).unwrap_or_else(|_| base.to_path_buf());
    path.strip_prefix(&base).unwrap_or(&path).to_string_lossy().replace('\\', "/")
}

fn write_images(options: &Options, model: &str, base: &Loaded, head: &Loaded, diff: &RichDiff, standard: &str) -> Result<Option<Images>, String> {
    let Some(dir) = &options.images_dir else { return Ok(None) };
    if !diff.model_changed {
        return Ok(None);
    }
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let name = slug(model);
    let mut written = Vec::new();
    for (suffix, loaded, which, label) in [("before", base, Side::Before, &diff.from), ("after", head, Side::After, &diff.to)] {
        let path = dir.join(format!("{name}-{suffix}.svg"));
        let caption = format!("{model} - {suffix} ({label})");
        std::fs::write(&path, svg::render(&loaded.result, diff, which, &caption, standard)).map_err(|e| format!("{}: {e}", path.display()))?;
        written.push(relative_to(&path, &options.link_base));
    }
    Ok(Some(Images { before: written.remove(0), after: written.remove(0) }))
}

fn model_report(options: &Options, model: &Path, label: &str, from: &Source, to: &Source, standard: &str) -> Result<ModelReport, String> {
    let (base_view, base) = side(model, from, standard);
    let (head_view, head) = side(model, to, standard);
    let head_gate = head_view.gate.clone();
    let base_blockers: Vec<String> = base_view.gate.as_ref().map(|g| g.blockers.clone()).unwrap_or_default();
    let head_blockers: Vec<String> = head_gate.as_ref().map(|g| g.blockers.clone()).unwrap_or_default();
    let new_blockers: Vec<String> = head_blockers.iter().filter(|b| !base_blockers.contains(b)).cloned().collect();
    let resolved_blockers: Vec<String> = base_blockers.iter().filter(|b| !head_blockers.contains(b)).cloned().collect();

    let mut diff = None;
    let mut images = None;
    if let (Some(base), Some(head)) = (&base, &head) {
        let d = rich::compare(model, base, head, standard)?;
        images = write_images(options, label, base, head, &d, standard)?;
        diff = Some(d);
    }
    let regressions = diff.as_ref().map(regressions_of).unwrap_or_default();
    let passed = head_view.compiled && head_gate.as_ref().is_some_and(|g| g.passed);
    let introduced = !head_view.compiled || !new_blockers.is_empty();
    let fails_the_check = match options.fail_on {
        FailOn::Never => false,
        FailOn::Blocker => introduced,
        FailOn::Major => introduced || !regressions.is_empty(),
    };
    Ok(ModelReport {
        model: label.to_string(),
        base: base_view,
        head: head_view,
        new_blockers,
        resolved_blockers,
        regressions,
        status: if passed { "pass" } else { "fail" },
        fails_the_check,
        diff,
        images,
    })
}

fn arc_paths_at(top: &Path, commit: &str) -> Vec<String> {
    git::run(top, &["ls-tree", "-r", "--name-only", commit]).map(|out| out.lines().filter(|l| l.ends_with(".arc")).map(str::to_string).collect()).unwrap_or_default()
}

/// Build the report. The working directory is inside the repository.
pub fn build(options: &Options) -> Result<Report, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let top = git::run(&cwd, &["rev-parse", "--show-toplevel"]).map_err(|_| "this is not a git repository: the report compares two revisions".to_string())?;
    let top = std::fs::canonicalize(top.trim()).map_err(|e| e.to_string())?;
    let standard = standard_name(&options.standard);
    let from_commit = git::commit_of(&top, &options.from)?;
    let to_commit = match &options.to {
        Some(rev) => Some(git::commit_of(&top, rev)?),
        None => None,
    };
    let from = Source::Git(options.from.clone());
    let to = options.to.as_deref().map(Source::parse).unwrap_or(Source::Worktree);

    let models: Vec<PathBuf> = if options.models.is_empty() { discover::models(&top) } else { options.models.iter().map(|m| if m.is_absolute() { m.clone() } else { cwd.join(m) }).collect() };
    let mut reports = Vec::new();
    for model in &models {
        let label = crate::compiler::cicd::diag::relative_uri(model, &top);
        reports.push(model_report(options, model, &label, &from, &to, standard)?);
    }

    // Model files that existed at the base and are gone at the head.
    let head_files: Vec<String> = match &to_commit {
        Some(commit) => arc_paths_at(&top, commit),
        None => discover::arc_files(&top).iter().map(|p| crate::compiler::cicd::diag::relative_uri(p, &top)).collect(),
    };
    let removed_files: Vec<String> = arc_paths_at(&top, &from_commit).into_iter().filter(|p| !head_files.contains(p)).collect();

    let status = if reports.is_empty() {
        "none"
    } else if reports.iter().all(|r| r.status == "pass") {
        "pass"
    } else {
        "fail"
    };
    let fails_the_check = reports.iter().any(|r| r.fails_the_check);
    Ok(Report {
        marker: MARKER,
        tool_version: env!("CARGO_PKG_VERSION"),
        from: options.from.clone(),
        to: options.to.clone().unwrap_or_else(|| "WORKTREE".to_string()),
        from_commit,
        to_commit,
        standard: standard.to_string(),
        fail_on: match options.fail_on {
            FailOn::Blocker => "blocker",
            FailOn::Major => "major",
            FailOn::Never => "never",
        },
        status,
        models: reports,
        removed_files,
        fails_the_check,
        note: options.note.clone(),
    })
}

fn gate_word(side: &SideView) -> String {
    if !side.present {
        return "(not there)".to_string();
    }
    match &side.gate {
        Some(g) if g.passed => "PASS".to_string(),
        Some(g) => format!("FAIL ({} blocker{})", g.blockers.len(), if g.blockers.len() == 1 { "" } else { "s" }),
        None => "does not compile".to_string(),
    }
}

fn short(commit: &str) -> &str {
    &commit[..commit.len().min(8)]
}

fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}

fn coverage_cell(m: &ModelReport) -> String {
    match (&m.base.gate, &m.head.gate) {
        (Some(b), Some(h)) => format!(
            "{}/{} -> {}/{} satisfied, {}/{} -> {}/{} verified",
            b.requirements_satisfied, b.requirements_total, h.requirements_satisfied, h.requirements_total, b.requirements_verified, b.requirements_total, h.requirements_verified, h.requirements_total
        ),
        (None, Some(h)) => format!("{}/{} satisfied, {}/{} verified", h.requirements_satisfied, h.requirements_total, h.requirements_verified, h.requirements_total),
        _ => "-".to_string(),
    }
}

fn changes_cell(m: &ModelReport) -> String {
    match &m.diff {
        Some(d) if d.model_changed => {
            let s = &d.summary;
            format!("+{} -{} ~{}", s.elements_added, s.elements_removed, s.elements_modified)
        }
        Some(d) if d.layout_only => "layout only".to_string(),
        Some(_) => "none".to_string(),
        None if !m.base.present && m.head.present => "new model".to_string(),
        None => "not comparable".to_string(),
    }
}

fn model_section(m: &ModelReport) -> String {
    let mut out = format!("### `{}`\n\n", m.model);
    if !m.head.compiled {
        out.push_str(&format!("**FAIL: the model does not compile at the head.**\n\n```text\n{}\n```\n\n", m.head.error.as_deref().unwrap_or("the model is missing")));
        return out;
    }
    out.push_str(&format!("Production gate: **{}** -> **{}**\n\n", gate_word(&m.base), gate_word(&m.head)));
    if !m.base.present {
        out.push_str("This model is new in the pull request: every blocker below is new.\n\n");
    } else if !m.base.compiled {
        out.push_str(&format!("The model did not compile at the base ({}): every blocker below counts as new.\n\n", cell(m.base.error.as_deref().unwrap_or(""))));
    }
    if !m.new_blockers.is_empty() {
        out.push_str("New blockers:\n\n");
        for b in &m.new_blockers {
            out.push_str(&format!("- {b}\n"));
        }
        out.push('\n');
    }
    let kept: Vec<&String> = m.head.gate.iter().flat_map(|g| &g.blockers).filter(|b| !m.new_blockers.contains(b)).collect();
    if !kept.is_empty() {
        out.push_str("Blockers already there at the base (not introduced by this change):\n\n");
        for b in kept {
            out.push_str(&format!("- {b}\n"));
        }
        out.push('\n');
    }
    if !m.resolved_blockers.is_empty() {
        out.push_str("Resolved by this change:\n\n");
        for b in &m.resolved_blockers {
            out.push_str(&format!("- {b}\n"));
        }
        out.push('\n');
    }
    if !m.regressions.is_empty() {
        out.push_str("Regressions:\n\n");
        for r in &m.regressions {
            out.push_str(&format!("- {r}\n"));
        }
        out.push('\n');
    }
    let Some(d) = &m.diff else { return out };
    let c = &d.consequences;
    out.push_str(&format!("Requirements: {}\n\n", coverage_cell(m)));
    if !c.budgets.is_empty() {
        out.push_str("| Latency budget | Before | After | Limit |\n|---|---|---|---|\n");
        for b in &c.budgets {
            out.push_str(&format!("| `{}` | {} ms | {} ms{} | {} ms |\n", cell(&b.name), ms(b.before_ms), ms(b.after_ms), if b.ok_before && !b.ok_after { " (exceeded)" } else { "" }, ms(b.budget_ms)));
        }
        out.push('\n');
    }
    for k in &c.constraints {
        out.push_str(&format!("- Constraint `{}`: {} -> {} ({})\n", k.id, truth(k.before), truth(k.after), cell(&k.expression)));
    }
    if !c.verdicts.is_empty() {
        out.push_str("| Simulation | Before | After |\n|---|---|---|\n");
        for v in &c.verdicts {
            out.push_str(&format!("| `{}` | {} | {}{} |\n", cell(&v.test), v.before, v.after, if v.stale { " (stale: to redo)" } else { "" }));
        }
        out.push('\n');
    }
    out.push_str(&format!("Impact: {} element(s) reached by the changes, {} of them blocker(s) of the gate.\n\n", c.impacted, c.impacted_blockers));
    out.push_str(&diff_details(d));
    if let Some(images) = &m.images {
        out.push_str(&format!("| Before | After |\n|---|---|\n| ![before]({}) | ![after]({}) |\n\n", images.before, images.after));
    }
    out
}

/// The changes, folded: counts in the summary, the elements inside.
fn diff_details(d: &RichDiff) -> String {
    if !d.model_changed {
        return if d.layout_only { "Only the layout file changed: nothing the model means has changed.\n\n".to_string() } else { "No semantic change to this model.\n\n".to_string() };
    }
    let mut out = format!("<details><summary>Semantic diff: {}</summary>\n\n", headline(d));
    let mut listed = 0;
    for layer in &d.layers {
        let title = if layer.label == layer.layer { layer.label.clone() } else { format!("{} ({})", layer.label, layer.layer) };
        out.push_str(&format!("**{title}**: +{} -{} ~{}\n\n", layer.counts.added, layer.counts.removed, layer.counts.modified));
        for e in layer.kinds.iter().flat_map(|k| &k.elements) {
            if listed == ELEMENTS_LISTED {
                break;
            }
            listed += 1;
            out.push_str(&format!("- {} {} `{}` {}\n", e.status, e.kind, e.id, e.name));
            for line in element_details(e) {
                out.push_str(&format!("  - {line}\n"));
            }
        }
        out.push('\n');
    }
    let total: usize = d.layers.iter().map(|l| l.counts.added + l.counts.removed + l.counts.modified).sum();
    if total > listed {
        out.push_str(&format!("... and {} more element(s); `arclang diff --from` lists them all.\n\n", total - listed));
    }
    if !d.relationships.is_empty() {
        out.push_str("Relationships:\n\n");
        for r in d.relationships.iter().take(ELEMENTS_LISTED) {
            out.push_str(&format!("- {}\n", relation_line(r)));
            for c in &r.changes {
                out.push_str(&format!("  - {}\n", field_line(c)));
            }
        }
        out.push('\n');
    }
    out.push_str("</details>\n\n");
    out
}

pub fn markdown(r: &Report) -> String {
    let mut out = format!("{MARKER}\n");
    match r.status {
        "none" => out.push_str("## ArcLang gate: no model found\n\nNo `.arc` model was found, so nothing was checked. This is not a pass.\n\n"),
        status => {
            let failing = r.models.iter().filter(|m| m.status != "pass").count();
            let introduced: usize = r.models.iter().map(|m| m.new_blockers.len()).sum();
            out.push_str(&format!("## ArcLang gate: **{}**\n\n", status.to_ascii_uppercase()));
            if status == "fail" {
                out.push_str(&format!("{failing} of {} model(s) fail the {} gate; {introduced} new blocker(s) in this change.\n\n", r.models.len(), r.standard));
            } else {
                out.push_str(&format!("All {} model(s) compile and pass the {} gate.\n\n", r.models.len(), r.standard));
            }
        }
    }
    out.push_str(&format!("Base `{}` ({}) -> head `{}`{} - ArcLang {}\n\n", r.from, short(&r.from_commit), r.to, r.to_commit.as_deref().map(|c| format!(" ({})", short(c))).unwrap_or_default(), r.tool_version));
    if !r.models.is_empty() {
        out.push_str("| Model | Gate before | Gate after | New blockers | Requirements | Changes |\n|---|---|---|---|---|---|\n");
        for m in &r.models {
            out.push_str(&format!("| `{}` | {} | {} | {} | {} | {} |\n", cell(&m.model), gate_word(&m.base), gate_word(&m.head), m.new_blockers.len(), coverage_cell(m), changes_cell(m)));
        }
        out.push('\n');
    }
    if !r.removed_files.is_empty() {
        out.push_str(&format!("Removed from the base: {}.\n\n", r.removed_files.iter().map(|f| format!("`{f}`")).collect::<Vec<_>>().join(", ")));
    }
    let mut sections = String::new();
    for m in &r.models {
        let section = model_section(m);
        if out.len() + sections.len() + section.len() > COMMENT_LIMIT {
            sections.push_str("_The report is cut here: the comment limit is reached. `arclang report --pr` writes it whole to a file, and the CI artifact has it._\n\n");
            break;
        }
        sections.push_str(&section);
    }
    out.push_str(&sections);
    if let Some(note) = &r.note {
        out.push_str(&format!("{note}\n\n"));
    }
    out.push_str(&format!("<sub>The verdict is the production gate's ({}); \"new\" means absent from the base. Exit status with `--fail-on {}`: {}.</sub>\n", r.standard, r.fail_on, if r.fails_the_check { "non-zero" } else { "zero" }));
    out
}

fn esc(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

pub fn html(r: &Report) -> String {
    let mut body = format!("{MARKER}\n<h1>ArcLang gate: <span class=\"{}\">{}</span></h1>\n", r.status, if r.status == "none" { "no model found".to_string() } else { r.status.to_ascii_uppercase() });
    body.push_str(&format!("<p>Base <code>{}</code> ({}) &rarr; head <code>{}</code>, {} gate, ArcLang {}</p>\n", esc(&r.from), esc(short(&r.from_commit)), esc(&r.to), esc(&r.standard), r.tool_version));
    if r.status == "none" {
        body.push_str("<p>No <code>.arc</code> model was found, so nothing was checked. This is not a pass.</p>\n");
    }
    for m in &r.models {
        body.push_str(&format!("<h2><code>{}</code> <small class=\"{}\">{}</small></h2>\n", esc(&m.model), m.status, m.status.to_ascii_uppercase()));
        if !m.head.compiled {
            body.push_str(&format!("<p>The model does not compile at the head.</p><pre>{}</pre>\n", esc(m.head.error.as_deref().unwrap_or(""))));
            continue;
        }
        body.push_str(&format!("<p>Production gate: <b>{}</b> &rarr; <b>{}</b>. Requirements: {}. Changes: {}.</p>\n", esc(&gate_word(&m.base)), esc(&gate_word(&m.head)), esc(&coverage_cell(m)), esc(&changes_cell(m))));
        for (title, list) in [("New blockers", &m.new_blockers), ("Resolved blockers", &m.resolved_blockers), ("Regressions", &m.regressions)] {
            if !list.is_empty() {
                body.push_str(&format!("<h3>{title}</h3><ul>{}</ul>\n", list.iter().map(|b| format!("<li>{}</li>", esc(b))).collect::<String>()));
            }
        }
        if let Some(d) = &m.diff {
            body.push_str(&format!("<p>Semantic diff: {}</p>\n", esc(&headline(d))));
            for layer in &d.layers {
                body.push_str(&format!("<h3>{} ({}): +{} -{} ~{}</h3><ul>", esc(&layer.label), esc(&layer.layer), layer.counts.added, layer.counts.removed, layer.counts.modified));
                for e in layer.kinds.iter().flat_map(|k| &k.elements).take(ELEMENTS_LISTED) {
                    body.push_str(&format!("<li>{} {} <code>{}</code> {}</li>", e.status, esc(&e.kind), esc(&e.id), esc(&e.name)));
                }
                body.push_str("</ul>\n");
            }
        }
        if let Some(images) = &m.images {
            body.push_str(&format!("<p><img src=\"{}\" alt=\"before\"> <img src=\"{}\" alt=\"after\"></p>\n", esc(&images.before), esc(&images.after)));
        }
    }
    format!("<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><title>ArcLang pull-request report</title>\n<style>body{{font:14px/1.5 system-ui,sans-serif;max-width:60rem;margin:2rem auto;padding:0 1rem}}code{{background:#eee;padding:0 .25em;border-radius:3px}}.pass{{color:#1a7f37}}.fail{{color:#c62828}}small{{font-size:.6em}}img{{max-width:48%}}</style></head>\n<body>\n{body}</body></html>\n")
}

pub fn json(r: &Report) -> String {
    format!("{}\n", serde_json::to_string_pretty(r).unwrap_or_default())
}
