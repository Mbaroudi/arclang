//! The diagnostics of a model, whatever their origin: a compile error, a
//! compiler finding (see `arclang rules`), a blocker or warning of the
//! production gate, the verdict of a simulation. One list, with the file and
//! the line when they are known, so that SARIF, the summary and the PR
//! report all say the same thing.

use crate::compiler::production_gate::{self, Severity as GateSeverity};
use crate::compiler::rules::{self, FixHint, Severity};
use crate::compiler::simulation::{is_simulated, Outcome};
use crate::compiler::CompilationResult;
use regex::Regex;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warning,
    Note,
}

impl Level {
    pub fn sarif(self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warning => "warning",
            Level::Note => "note",
        }
    }
}

/// How well the line is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Precision {
    /// The compiler said where (a syntax error).
    Exact,
    /// The line where the element the finding names is declared.
    Element,
    /// Only the file is known; the line is the first.
    File,
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    /// `compile.error`, a rule of `arclang rules`, `gate.<check>`, `verification.*`.
    pub rule: String,
    pub level: Level,
    pub message: String,
    /// The model file, relative to the root, with `/` separators.
    pub file: String,
    pub line: usize,
    pub column: Option<usize>,
    pub precision: Precision,
    pub element: Option<String>,
    pub fix: Option<FixHint>,
    /// The gate check that reports it too, when the gate does.
    pub gate_check: Option<String>,
}

/// What the gate said of one model.
#[derive(Debug, Clone, Serialize)]
pub struct GateSummary {
    pub standard: String,
    pub passed: bool,
    pub blockers: usize,
    pub warnings: usize,
    pub requirements_total: usize,
    pub requirements_satisfied: usize,
    pub requirements_verified: usize,
}

/// A model examined: it compiled or it did not, the gate passed or it did not.
#[derive(Debug, Clone, Serialize)]
pub struct ModelRun {
    pub model: String,
    pub compiled: bool,
    pub gate: Option<GateSummary>,
    pub diagnostics: Vec<Diagnostic>,
}

impl ModelRun {
    /// The model is fit to go: it compiles and the gate passes. Nothing else says so.
    pub fn passed(&self) -> bool {
        self.compiled && self.gate.as_ref().is_some_and(|g| g.passed)
    }

    pub fn errors(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.level == Level::Error).count()
    }
}

/// What to examine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// The compiler's diagnostics (`arclang check`).
    Check,
    /// The gate's findings (`arclang gate`).
    Gate,
    /// Both, merged (`arclang sarif`, `arclang ci run`).
    Both,
}

pub fn standard_name(text: &str) -> &'static str {
    match text.to_ascii_uppercase().replace(['-', '_'], "").as_str() {
        "DO178C" => "DO178C",
        "IEC61508" => "IEC61508",
        _ => "ISO26262",
    }
}

/// The path as a SARIF URI: relative to `root`, `/`-separated.
pub fn relative_uri(path: &Path, root: &Path) -> String {
    let absolute = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    absolute.strip_prefix(&root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn level_of(severity: Severity) -> Level {
    match severity {
        Severity::Error => Level::Error,
        Severity::Warning => Level::Warning,
        Severity::Info => Level::Note,
    }
}

/// `Parser error: dir/m.arc: Expected RightBrace, got Eof at line 5, column 1`
fn compile_error(text: &str, model: &Path, root: &Path) -> Diagnostic {
    let located = Regex::new(r"^(?:[A-Za-z]+ error): (?P<file>[^:]+): (?P<msg>.*?) at line (?P<line>\d+), column (?P<column>\d+)").expect("a valid pattern");
    let (file, message, line, column, precision) = match located.captures(text) {
        Some(c) => {
            let named = PathBuf::from(&c["file"]);
            let file = if named.exists() { relative_uri(&named, root) } else { relative_uri(model, root) };
            (file, format!("{} (line {}, column {})", &c["msg"], &c["line"], &c["column"]), c["line"].parse().unwrap_or(1), c["column"].parse().ok(), Precision::Exact)
        }
        None => (relative_uri(model, root), text.to_string(), 1, None, Precision::File),
    };
    Diagnostic { rule: "compile.error".into(), level: Level::Error, message, file, line, column, precision, element: None, fix: None, gate_check: None }
}

/// The 1-based line where an element is declared: its `id:` line, then its
/// `keyword "name" {` line, then any line that names it.
pub fn locate_element(text: &str, element: &str) -> Option<usize> {
    let quoted = format!("\"{element}\"");
    let id_line = Regex::new(&format!(r#"\bid\s*:\s*"{}""#, regex::escape(element))).ok()?;
    let header = Regex::new(&format!(r#"^\s*[A-Za-z_][A-Za-z0-9_]*\s+"{}"\s*\{{"#, regex::escape(element))).ok()?;
    let lines: Vec<&str> = text.lines().collect();
    let first = |test: &dyn Fn(&str) -> bool| lines.iter().position(|l| test(l)).map(|i| i + 1);
    first(&|l| id_line.is_match(l)).or_else(|| first(&|l| header.is_match(l))).or_else(|| first(&|l| l.contains(&quoted)))
}

/// The names between single quotes in a message, in order.
fn quoted_names(message: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = message;
    while let Some(start) = rest.find('\'') {
        let after = &rest[start + 1..];
        match after.find('\'') {
            Some(end) => {
                names.push(after[..end].to_string());
                rest = &after[end + 1..];
            }
            None => break,
        }
    }
    names
}

struct Placer<'a> {
    file: String,
    text: &'a str,
}

impl Placer<'_> {
    fn place(&self, needles: &[String]) -> (usize, Precision) {
        needles
            .iter()
            .find_map(|n| locate_element(self.text, n))
            .map(|line| (line, Precision::Element))
            .unwrap_or((1, Precision::File))
    }
}

fn gate_findings(result: &CompilationResult, standard: &str, placer: &Placer) -> (GateSummary, Vec<Diagnostic>) {
    let report = production_gate::run_gate(&result.ast, &result.semantic_model, standard);
    let mut diagnostics = Vec::new();
    for finding in &report.findings {
        let needles = quoted_names(&finding.message);
        let (line, precision) = placer.place(&needles);
        diagnostics.push(Diagnostic {
            rule: format!("gate.{}", finding.check),
            level: if finding.severity == GateSeverity::Blocker { Level::Error } else { Level::Warning },
            message: finding.message.clone(),
            file: placer.file.clone(),
            line,
            column: None,
            precision,
            element: needles.first().cloned(),
            fix: None,
            gate_check: Some(finding.check.clone()),
        });
    }
    let blockers = report.findings.iter().filter(|f| f.severity == GateSeverity::Blocker).count();
    let summary = GateSummary {
        standard: standard.to_string(),
        passed: report.passed,
        blockers,
        warnings: report.findings.len() - blockers,
        requirements_total: report.requirements_total,
        requirements_satisfied: report.requirements_satisfied,
        requirements_verified: report.requirements_verified,
    };
    (summary, diagnostics)
}

fn compiler_findings(result: &CompilationResult, placer: &Placer) -> Vec<Diagnostic> {
    rules::findings(result)
        .into_iter()
        .map(|f| {
            let needles: Vec<String> = f.element.iter().cloned().collect();
            let (line, precision) = placer.place(&needles);
            Diagnostic { rule: f.rule.to_string(), level: level_of(f.severity), message: f.message, file: placer.file.clone(), line, column: None, precision, element: f.element, fix: f.fix, gate_check: None }
        })
        .collect()
}

fn same_finding(a: &str, b: &str) -> bool {
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    a == b || (short.len() >= 24 && long.contains(short))
}

/// The gate reports some of what the compiler reports (typed attributes,
/// methodology): the compiler's finding keeps its rule and its fix, and
/// says the gate counts it; a blocker stays an error.
fn merge(compiler: Vec<Diagnostic>, gate: Vec<Diagnostic>) -> Vec<Diagnostic> {
    let mut merged = compiler;
    for g in gate {
        match merged.iter_mut().find(|c| same_finding(&c.message, &g.message)) {
            Some(twin) => {
                twin.gate_check = g.gate_check.clone();
                if g.level == Level::Error {
                    twin.level = Level::Error;
                }
            }
            None => merged.push(g),
        }
    }
    merged
}

/// Compile a model and examine it.
pub fn analyze(model: &Path, root: &Path, standard: &str, scope: Scope) -> ModelRun {
    let standard = standard_name(standard);
    let uri = relative_uri(model, root);
    let result = match crate::Compiler::new(crate::CompilerConfig::default()).compile_file(model) {
        Ok(result) => result,
        Err(error) => {
            return ModelRun { model: uri, compiled: false, gate: None, diagnostics: vec![compile_error(&error.to_string(), model, root)] };
        }
    };
    let text = std::fs::read_to_string(model).unwrap_or_default();
    let placer = Placer { file: uri.clone(), text: &text };
    let compiler = if scope == Scope::Gate { Vec::new() } else { compiler_findings(&result, &placer) };
    let (gate, gate_diagnostics) = if scope == Scope::Check {
        (None, Vec::new())
    } else {
        let (summary, diagnostics) = gate_findings(&result, standard, &placer);
        (Some(summary), diagnostics)
    };
    ModelRun { model: uri, compiled: true, gate, diagnostics: merge(compiler, gate_diagnostics) }
}

/// Does the model have test cases that simulate?
pub fn has_simulations(result: &CompilationResult) -> bool {
    result.ast.test_cases.iter().any(is_simulated)
}

/// The verdicts of a simulation as diagnostics: a failure is an error, an
/// inconclusive case is a warning that says why (never a pass).
pub fn verification_diagnostics(outcomes: &[Outcome], model: &Path, root: &Path) -> Vec<Diagnostic> {
    let file = relative_uri(model, root);
    let text = std::fs::read_to_string(model).unwrap_or_default();
    outcomes
        .iter()
        .filter(|o| o.verdict != "pass")
        .map(|o| {
            let line = locate_element(&text, &o.test_case);
            let (rule, level) = if o.verdict == "fail" { ("verification.failed", Level::Error) } else { ("verification.inconclusive", Level::Warning) };
            let values: Vec<String> = o.values.iter().filter(|v| !v.holds).map(|v| v.value.clone()).collect();
            let detail = if o.reason.is_empty() { values.join("; ") } else { o.reason.clone() };
            Diagnostic {
                rule: rule.into(),
                level,
                message: format!("test case '{}' is {}{}", o.test_case, o.verdict.to_uppercase(), if detail.is_empty() { String::new() } else { format!(": {detail}") }),
                file: file.clone(),
                line: line.unwrap_or(1),
                column: None,
                precision: if line.is_some() { Precision::Element } else { Precision::File },
                element: Some(o.test_case.clone()),
                fix: None,
                gate_check: None,
            }
        })
        .collect()
}
