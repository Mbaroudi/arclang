//! `arclang ci init|run|models`, `arclang report --pr` and `arclang sarif`:
//! the commands a pipeline calls. The work is in `compiler::cicd`.

use super::{CliError, SafetyStandard};
use crate::compiler::cicd::diag::{self, Diagnostic, Level, ModelRun, Scope};
use crate::compiler::cicd::pipelines::{self, Options as PipelineOptions, Planned, Provider};
use crate::compiler::cicd::report::{self, FailOn};
use crate::compiler::cicd::{discover, sarif};
use crate::compiler::semgit::git;
use std::path::{Path, PathBuf};

fn failure(message: impl Into<String>) -> CliError {
    CliError::CiFailed(message.into())
}

fn standard_of(standard: &SafetyStandard) -> &'static str {
    match standard {
        SafetyStandard::ISO26262 => "ISO26262",
        SafetyStandard::DO178C => "DO178C",
        SafetyStandard::IEC61508 => "IEC61508",
    }
}

/// The root of the repository the command runs in, else the working directory.
fn repo_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    git::run(&cwd, &["rev-parse", "--show-toplevel"])
        .ok()
        .and_then(|top| std::fs::canonicalize(top.trim()).ok())
        .unwrap_or(cwd)
}

fn write_file(path: &Path, text: &str) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(())
}

// ---------------------------------------------------------------- formats

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticFormat {
    Text,
    /// SARIF 2.1.0, for GitHub code scanning and Azure DevOps
    Sarif,
}

/// `arclang check --format sarif` and `arclang gate --format sarif`.
pub fn run_scoped_sarif(input: &Path, scope: Scope, standard: &SafetyStandard) -> Result<(), CliError> {
    let run = diag::analyze(input, &repo_root(), standard_of(standard), scope);
    print!("{}", sarif::text(std::slice::from_ref(&run), env!("CARGO_PKG_VERSION")));
    match scope {
        Scope::Gate if !run.passed() => Err(failure(format!("{}: the production gate fails ({})", run.model, why(&run)))),
        Scope::Check if !run.compiled => Err(failure(format!("{}: the model does not compile", run.model))),
        _ => Ok(()),
    }
}

fn why(run: &ModelRun) -> String {
    match &run.gate {
        _ if !run.compiled => "the model does not compile".to_string(),
        Some(g) => format!("{} blocker(s)", g.blockers),
        None => "no verdict".to_string(),
    }
}

// ------------------------------------------------------------------ sarif

#[derive(clap::Args, Debug)]
pub struct SarifArgs {
    /// Models to examine (default: every entry-point model of the repository)
    #[clap(value_parser)]
    pub models: Vec<PathBuf>,

    #[clap(long, default_value = "iso26262")]
    pub standard: SafetyStandard,

    /// Write the SARIF file here (default: standard output)
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// error: exit 1 when a model does not compile or its gate fails; never: always 0
    #[clap(long, default_value = "error")]
    pub fail_on: String,
}

fn models_of(given: &[PathBuf], root: &Path) -> Vec<PathBuf> {
    if given.is_empty() {
        discover::models(root)
    } else {
        given.to_vec()
    }
}

pub fn run_sarif(args: &SarifArgs) -> Result<(), CliError> {
    let root = repo_root();
    let models = models_of(&args.models, &root);
    if models.is_empty() {
        return Err(failure("no .arc model found: give the models to examine"));
    }
    let runs: Vec<ModelRun> = models.iter().map(|m| diag::analyze(m, &root, standard_of(&args.standard), Scope::Both)).collect();
    let text = sarif::text(&runs, env!("CARGO_PKG_VERSION"));
    match &args.output {
        Some(path) => write_file(path, &text)?,
        None => print!("{text}"),
    }
    let failing: Vec<&ModelRun> = runs.iter().filter(|r| !r.passed()).collect();
    match args.fail_on.as_str() {
        "never" => Ok(()),
        "error" if failing.is_empty() => Ok(()),
        "error" => Err(failure(format!("{} of {} model(s) do not pass: {}", failing.len(), runs.len(), failing.iter().map(|r| format!("{} ({})", r.model, why(r))).collect::<Vec<_>>().join(", ")))),
        other => Err(CliError::Config(format!("--fail-on is error or never, not '{other}'"))),
    }
}

// --------------------------------------------------------------------- ci

#[derive(clap::Subcommand, Debug)]
pub enum CiCommand {
    /// Write the pipeline files for GitHub Actions, GitLab CI, Azure DevOps or Jenkins
    Init(InitArgs),
    /// What a pipeline runs: compile every model, run the production gate, simulate the cases
    /// that simulate, write the SARIF file and a summary; non-zero unless every model passes
    Run(RunArgs),
    /// The models of the repository: the .arc files no other .arc imports
    Models {
        /// Print JSON
        #[clap(long)]
        json: bool,
    },
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderChoice {
    Github,
    Gitlab,
    Azure,
    Jenkins,
    All,
}

#[derive(clap::Args, Debug)]
pub struct InitArgs {
    /// The CI system (default: the one the git remote `origin` points at)
    #[clap(long, value_enum)]
    pub provider: Option<ProviderChoice>,

    /// The standard of the production gate
    #[clap(long, default_value = "iso26262")]
    pub standard: SafetyStandard,

    /// A model the pipeline checks, relative to the repository root (repeatable; default: every entry point)
    #[clap(long = "model")]
    pub models: Vec<String>,

    /// Overwrite files that exist
    #[clap(long)]
    pub force: bool,

    /// Print the files, write nothing
    #[clap(long)]
    pub dry_run: bool,

    /// The ArcLang release the pipeline installs (default: this compiler's)
    #[clap(long)]
    pub arclang_version: Option<String>,

    /// owner/repo of the GitHub releases to download the binary from
    #[clap(long, default_value = pipelines::RELEASE_REPO)]
    pub release_repo: String,

    /// The branch whose pushes run the pipeline (default: the repository's default branch)
    #[clap(long)]
    pub branch: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerifyMode {
    /// Simulate the cases of the models that have some
    Auto,
    Never,
}

#[derive(clap::Args, Debug)]
pub struct RunArgs {
    /// Models to examine (default: every entry-point model of the repository)
    #[clap(value_parser)]
    pub models: Vec<PathBuf>,

    #[clap(long, default_value = "iso26262")]
    pub standard: SafetyStandard,

    /// Write the SARIF 2.1.0 file here
    #[clap(long)]
    pub sarif: Option<PathBuf>,

    /// Write a Markdown summary here (for the job summary)
    #[clap(long)]
    pub summary: Option<PathBuf>,

    #[clap(long, value_enum, default_value = "auto")]
    pub verify: VerifyMode,

    /// Succeed when there is no model at all (otherwise that is a failure: nothing was checked)
    #[clap(long)]
    pub allow_empty: bool,
}

pub fn run_ci(command: &CiCommand) -> Result<(), CliError> {
    match command {
        CiCommand::Init(args) => init(args),
        CiCommand::Run(args) => run(args),
        CiCommand::Models { json } => {
            let root = repo_root();
            let models: Vec<String> = discover::models(&root).iter().map(|m| diag::relative_uri(m, &root)).collect();
            if *json {
                println!("{}", serde_json::to_string_pretty(&models).unwrap_or_default());
            } else {
                for model in models {
                    println!("{model}");
                }
            }
            Ok(())
        }
    }
}

fn providers_of(args: &InitArgs, root: &Path) -> Result<Vec<Provider>, CliError> {
    match args.provider {
        Some(ProviderChoice::All) => Ok(Provider::ALL.to_vec()),
        Some(ProviderChoice::Github) => Ok(vec![Provider::Github]),
        Some(ProviderChoice::Gitlab) => Ok(vec![Provider::Gitlab]),
        Some(ProviderChoice::Azure) => Ok(vec![Provider::Azure]),
        Some(ProviderChoice::Jenkins) => Ok(vec![Provider::Jenkins]),
        None => pipelines::detect(root).map(|p| vec![p]).ok_or_else(|| {
            CliError::Config("the CI system could not be told from the git remote 'origin': give --provider github|gitlab|azure|jenkins|all".to_string())
        }),
    }
}

fn init(args: &InitArgs) -> Result<(), CliError> {
    let root = repo_root();
    let providers = providers_of(args, &root)?;
    let options = PipelineOptions {
        version: args.arclang_version.clone().unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string()),
        standard: standard_of(&args.standard).to_string(),
        models: args.models.clone(),
        branch: args.branch.clone().unwrap_or_else(|| pipelines::default_branch(&root)),
        release_repo: args.release_repo.clone(),
    };
    pipelines::validate(&options).map_err(CliError::Config)?;
    for model in &options.models {
        if !root.join(model).is_file() {
            return Err(CliError::Config(format!("--model {model}: no such file in {}", root.display())));
        }
    }

    let mut planned: Vec<(Provider, Planned)> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    for provider in &providers {
        for file in pipelines::render(*provider, &options) {
            planned.push((*provider, file));
        }
        if *provider == Provider::Gitlab {
            if root.join(".gitlab-ci.yml").exists() {
                notes.push("GitLab: .gitlab-ci.yml exists and is left alone; add to it:\n    include:\n      - local: .gitlab/arclang.gitlab-ci.yml".to_string());
            } else {
                planned.push((*provider, pipelines::gitlab_root()));
            }
        }
    }

    let existing: Vec<&str> = planned.iter().filter(|(_, f)| root.join(&f.path).exists()).map(|(_, f)| f.path.as_str()).collect();
    if args.dry_run {
        for (_, file) in &planned {
            let state = if existing.contains(&file.path.as_str()) { if args.force { " (exists: would be overwritten)" } else { " (exists: would be refused without --force)" } } else { "" };
            println!("--- {}{state}\n{}", file.path, file.content);
        }
        for note in &notes {
            println!("{note}");
        }
        println!("Dry run: nothing was written.");
        return Ok(());
    }
    if !existing.is_empty() && !args.force {
        return Err(CliError::Config(format!(
            "{} exist{}; nothing was written. Use --force to overwrite, or --dry-run to see what would be written",
            existing.join(", "),
            if existing.len() == 1 { "s" } else { "" }
        )));
    }
    for (_, file) in &planned {
        let verb = if existing.contains(&file.path.as_str()) { "overwrote" } else { "created" };
        write_file(&root.join(&file.path), &file.content)?;
        println!("{verb} {}", file.path);
    }
    for note in &notes {
        println!("{note}");
    }
    for provider in &providers {
        println!("{}: {}", provider.name(), pipelines::next_step(*provider));
    }
    println!("These files were not run against the provider: the first pipeline run is their test. The pipeline installs ArcLang {} from its GitHub release.", options.version);
    Ok(())
}

fn print_model(run: &ModelRun, verification_failed: bool) {
    let verdict = if run.passed() && !verification_failed { "PASS" } else { "FAIL" };
    let detail = match (&run.gate, run.compiled) {
        (_, false) => "does not compile".to_string(),
        (Some(g), _) => format!("gate {}: {} blocker(s), {} warning(s); requirements {}/{} satisfied, {}/{} verified", g.standard, g.blockers, g.warnings, g.requirements_satisfied, g.requirements_total, g.requirements_verified, g.requirements_total),
        (None, _) => String::new(),
    };
    println!("{verdict}  {}  ({detail})", run.model);
    for d in run.diagnostics.iter().filter(|d| d.level == Level::Error) {
        println!("      {}:{}  [{}] {}", d.file, d.line, d.rule, d.message);
    }
    for d in run.diagnostics.iter().filter(|d| d.rule == "verification.inconclusive") {
        println!("      {}:{}  [{}] {}", d.file, d.line, d.rule, d.message);
    }
}

fn summary_markdown(runs: &[(ModelRun, bool)]) -> String {
    let mut out = String::from("## ArcLang gate\n\n");
    let all = runs.iter().all(|(r, vf)| r.passed() && !vf);
    out.push_str(&format!("**{}**: {} model(s) examined.\n\n| Model | Verdict | Blockers | Warnings | Requirements verified |\n|---|---|---|---|---|\n", if all { "PASS" } else { "FAIL" }, runs.len()));
    for (r, vf) in runs {
        let ok = r.passed() && !vf;
        match &r.gate {
            Some(g) => out.push_str(&format!("| `{}` | {} | {} | {} | {}/{} |\n", r.model, if ok { "PASS" } else { "FAIL" }, g.blockers, g.warnings, g.requirements_verified, g.requirements_total)),
            None => out.push_str(&format!("| `{}` | FAIL (does not compile) | - | - | - |\n", r.model)),
        }
    }
    for (r, _) in runs {
        let shown: Vec<&Diagnostic> = r.diagnostics.iter().filter(|d| d.level == Level::Error || d.rule == "verification.inconclusive").collect();
        if shown.is_empty() {
            continue;
        }
        out.push_str(&format!("\n### `{}`\n\n", r.model));
        for d in shown {
            out.push_str(&format!("- `{}:{}` **{}**: {}\n", d.file, d.line, d.rule, d.message.replace('\n', " ")));
        }
    }
    out
}

fn run(args: &RunArgs) -> Result<(), CliError> {
    use crate::compiler::simulation::{run as simulate, tool::ToolConfig};
    let root = repo_root();
    let models = models_of(&args.models, &root);
    if models.is_empty() {
        if args.allow_empty {
            println!("No .arc model found: nothing was checked.");
            return Ok(());
        }
        return Err(failure("no .arc model found, so nothing was checked (give the models, or --allow-empty)"));
    }
    let standard = standard_of(&args.standard);
    let mut runs: Vec<(ModelRun, bool)> = Vec::new();
    for model in &models {
        let mut extra: Vec<Diagnostic> = Vec::new();
        if args.verify == VerifyMode::Auto {
            let compiled = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(model);
            if let Ok(result) = compiled {
                if diag::has_simulations(&result) {
                    match simulate::verify(&result, model, None, &ToolConfig::from_env()) {
                        Ok(outcomes) => extra = diag::verification_diagnostics(&outcomes, model, &root),
                        Err(reason) => {
                            let outcome = crate::compiler::simulation::Outcome {
                                test_case: model.display().to_string(),
                                verdict: "inconclusive".into(),
                                reason,
                                values: Vec::new(),
                                key: String::new(),
                                model: String::new(),
                                configuration: None,
                                tool: String::new(),
                                omc_version: String::new(),
                                log: String::new(),
                            };
                            extra = diag::verification_diagnostics(&[outcome], model, &root);
                        }
                    }
                }
            }
        }
        let mut run = diag::analyze(model, &root, standard, Scope::Both);
        let verification_failed = extra.iter().any(|d| d.rule == "verification.failed");
        run.diagnostics.extend(extra);
        print_model(&run, verification_failed);
        runs.push((run, verification_failed));
    }
    let model_runs: Vec<ModelRun> = runs.iter().map(|(r, _)| r.clone()).collect();
    if let Some(path) = &args.sarif {
        write_file(path, &sarif::text(&model_runs, env!("CARGO_PKG_VERSION")))?;
        println!("SARIF: {}", path.display());
    }
    if let Some(path) = &args.summary {
        write_file(path, &summary_markdown(&runs))?;
    }
    let failing: Vec<&str> = runs.iter().filter(|(r, vf)| !r.passed() || *vf).map(|(r, _)| r.model.as_str()).collect();
    if failing.is_empty() {
        println!("ArcLang gate: PASS ({} model(s))", runs.len());
        Ok(())
    } else {
        Err(failure(format!("ArcLang gate: FAIL ({} of {} model(s): {})", failing.len(), runs.len(), failing.join(", "))))
    }
}

// ----------------------------------------------------------------- report

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportFormat {
    Md,
    Json,
    Html,
}

#[derive(clap::Args, Debug)]
pub struct ReportArgs {
    /// The pull-request report: the semantic diff, the impact, the gate before and after
    #[clap(long)]
    pub pr: bool,

    /// The base: a branch, tag or commit (the target branch of the pull request)
    #[clap(long)]
    pub from: String,

    /// The head (default: the working tree)
    #[clap(long)]
    pub to: Option<String>,

    /// Models to compare (default: every entry-point model of the repository)
    #[clap(value_parser)]
    pub models: Vec<PathBuf>,

    #[clap(short, long, value_enum, default_value = "md")]
    pub format: ReportFormat,

    /// Write the report here (default: standard output)
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Write the before and after pictures (SVG) under this directory, as <dir>/diagrams/
    #[clap(long)]
    pub out_dir: Option<PathBuf>,

    #[clap(long, default_value = "iso26262")]
    pub standard: SafetyStandard,

    /// blocker: exit 1 on a new blocker or a model that no longer compiles;
    /// major: also on a regression (requirement, constraint, budget, simulation); never: always 0
    #[clap(long, default_value = "blocker")]
    pub fail_on: String,

    /// A line of Markdown added to the report (where to find the pictures, for instance)
    #[clap(long)]
    pub note: Option<String>,

    /// Succeed when there is no model at all
    #[clap(long)]
    pub allow_empty: bool,
}

pub fn run_report(args: &ReportArgs) -> Result<(), CliError> {
    if !args.pr {
        return Err(CliError::Config("only the pull-request report exists: arclang report --pr --from <base>".into()));
    }
    let fail_on = FailOn::parse(&args.fail_on).ok_or_else(|| CliError::Config(format!("--fail-on is blocker, major or never, not '{}'", args.fail_on)))?;
    let cwd = std::env::current_dir()?;
    let link_base = match &args.output {
        Some(path) => path.parent().filter(|p| !p.as_os_str().is_empty()).map(Path::to_path_buf).unwrap_or_else(|| cwd.clone()),
        None => cwd.clone(),
    };
    if let Some(parent) = args.output.as_ref().and_then(|p| p.parent()).filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let options = report::Options {
        from: args.from.clone(),
        to: args.to.clone(),
        standard: standard_of(&args.standard).to_string(),
        fail_on,
        models: args.models.clone(),
        images_dir: args.out_dir.as_ref().map(|d| d.join("diagrams")),
        link_base,
        note: args.note.clone(),
    };
    let built = report::build(&options).map_err(failure)?;
    let text = match args.format {
        ReportFormat::Md => report::markdown(&built),
        ReportFormat::Json => report::json(&built),
        ReportFormat::Html => report::html(&built),
    };
    match &args.output {
        Some(path) => write_file(path, &text)?,
        None => print!("{text}"),
    }
    if built.status == "none" && !args.allow_empty {
        return Err(failure("no .arc model found, so nothing was compared (give the models, or --allow-empty)"));
    }
    if built.fails_the_check {
        let bad: Vec<String> = built.models.iter().filter(|m| m.fails_the_check).map(|m| m.model.clone()).collect();
        return Err(failure(format!("the pull request makes the gate worse (--fail-on {}): {}", built.fail_on, bad.join(", "))));
    }
    Ok(())
}
