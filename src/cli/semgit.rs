//! `arclang diff --from`, `history`, `merge` and `baseline`: the commands of
//! semantic git. The work is in `compiler::semgit`; this is the command line.

use super::CliError;
use crate::compiler::semgit::baseline::{self, CreateOptions};
use crate::compiler::semgit::history as element_history;
use crate::compiler::semgit::git;
use crate::compiler::semgit::merge::{self, install, layout};
use crate::compiler::semgit::render::{self, Format};
use crate::compiler::semgit::rich::{self, Source};
use std::io::{BufRead, Write};
use std::path::PathBuf;

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffFormat {
    Text,
    Json,
    Md,
    Html,
}

impl From<DiffFormat> for Format {
    fn from(f: DiffFormat) -> Format {
        match f {
            DiffFormat::Text => Format::Text,
            DiffFormat::Json => Format::Json,
            DiffFormat::Md => Format::Md,
            DiffFormat::Html => Format::Html,
        }
    }
}

fn failure(message: String) -> CliError {
    CliError::Compilation(message)
}

fn emit(text: &str, output: &Option<PathBuf>) -> Result<(), CliError> {
    match output {
        Some(path) => std::fs::write(path, text).map_err(CliError::Io),
        None => {
            print!("{text}");
            Ok(())
        }
    }
}

/// `arclang diff` in its semantic-git forms: revisions, or a chosen format.
#[allow(clippy::too_many_arguments)]
pub fn run_diff(old: PathBuf, new: Option<PathBuf>, from: Option<String>, to: Option<String>, format: Format, output: Option<PathBuf>, gate_standard: &str) -> Result<(), CliError> {
    let (model, source_from, source_to) = match (&new, &from) {
        (Some(new), None) => {
            if to.is_some() {
                return Err(CliError::Config("--to goes with --from".into()));
            }
            (new.clone(), Source::File(old.clone()), Source::File(new.clone()))
        }
        (None, Some(rev)) => (old.clone(), Source::parse(rev), to.as_deref().map(Source::parse).unwrap_or(Source::Worktree)),
        (Some(_), Some(_)) => return Err(CliError::Config("give two files, or one file with --from, not both".into())),
        (None, None) => return Err(CliError::Config("give two model files, or one with --from <revision>".into())),
    };
    let standard = match gate_standard.to_ascii_uppercase().replace(['-', '_'], "").as_str() {
        "DO178C" => "DO178C",
        "IEC61508" => "IEC61508",
        _ => "ISO26262",
    };
    let diff = rich::diff_sources(&model, &source_from, &source_to, standard).map_err(failure)?;
    emit(&render::render(&diff, format), &output)?;
    if diff.model_changed {
        Err(CliError::DiffFound)
    } else {
        Ok(())
    }
}

#[derive(clap::Args, Debug)]
pub struct HistoryArgs {
    #[clap(value_parser)]
    pub input: PathBuf,
    /// The element: its id or unique name
    #[clap(value_parser)]
    pub element: String,
    /// The most commits listed
    #[clap(long, default_value_t = 20)]
    pub limit: usize,
    #[clap(long)]
    pub json: bool,
}

pub fn run_history(args: &HistoryArgs) -> Result<(), CliError> {
    let history = element_history::history(&args.input, &args.element, args.limit.max(1)).map_err(failure)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&history).map_err(|e| failure(e.to_string()))?);
    } else {
        print!("{}", element_history::text(&history));
    }
    Ok(())
}

#[derive(clap::Args, Debug)]
pub struct RestoreArgs {
    /// The model file, as it is now
    #[clap(value_parser)]
    pub input: PathBuf,
    /// The git revision to go back to
    #[clap(long)]
    pub rev: String,
    /// Only these elements (their ids), the rest left as it is; repeatable.
    /// Without it, the whole file as the revision has it
    #[clap(long = "element")]
    pub elements: Vec<String>,
    /// Write the result here; default: standard output. The file itself is never changed unless named here
    #[clap(short, long)]
    pub output: Option<PathBuf>,
}

/// `arclang restore model.arc --rev v1.0 --element LC-001`: the text of the
/// model with the elements (or the whole file) as they were at a revision.
/// An element removed since is put back in its parent. Nothing is
/// committed: the result is a change to review, as any edit.
pub fn run_restore(args: &RestoreArgs) -> Result<(), CliError> {
    git::check_revision(&args.rev).map_err(failure)?;
    let located = git::locate(&args.input).map_err(failure)?;
    let old = git::show(&located.top, &args.rev, &located.relative)
        .ok_or_else(|| failure(format!("{} is not in revision {}", located.relative, args.rev)))?;
    let text = if args.elements.is_empty() {
        old
    } else {
        let mut now = std::fs::read_to_string(&args.input).map_err(|e| failure(format!("{}: {e}", args.input.display())))?;
        for element in &args.elements {
            now = merge::restore_element(&now, &old, element)
                .map_err(failure)?
                .ok_or_else(|| failure(format!("element '{element}' is not in {} at {}", located.relative, args.rev)))?;
        }
        now
    };
    match &args.output {
        Some(path) => std::fs::write(path, &text)?,
        None => print!("{text}"),
    }
    Ok(())
}

#[derive(clap::Args, Debug)]
pub struct MergeArgs {
    /// The common ancestor (git's %O)
    #[clap(value_parser)]
    pub base: Option<PathBuf>,
    /// Our version (git's %A)
    #[clap(value_parser)]
    pub ours: Option<PathBuf>,
    /// Their version (git's %B)
    #[clap(value_parser)]
    pub theirs: Option<PathBuf>,
    /// Write the result here (git's driver passes %A); default: standard output
    #[clap(short, long)]
    pub output: Option<PathBuf>,
    /// The path of the file in the repository (git's %P): where imports resolve, and what kind of file it is
    #[clap(long)]
    pub path: Option<PathBuf>,
    /// Describe the outcome as JSON on standard output
    #[clap(long)]
    pub json: bool,
    /// Do not compile the result
    #[clap(long)]
    pub no_compile: bool,
    /// Set git up to use this merge for .arc and .layout.json files
    #[clap(long)]
    pub install: bool,
    /// With --install: do not ask
    #[clap(long, short = 'y')]
    pub yes: bool,
    /// With --install: show what would change and stop
    #[clap(long)]
    pub dry_run: bool,
    /// With --install: the arclang to run (default: this one)
    #[clap(long)]
    pub binary: Option<String>,
}

fn run_install(args: &MergeArgs) -> Result<(), CliError> {
    let binary = args.binary.clone().unwrap_or_else(|| {
        std::env::current_exe()
            .ok()
            .filter(|p| p.file_stem().is_some_and(|s| s.to_string_lossy().starts_with("arclang")))
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "arclang".to_string())
    });
    let plan = install::plan(&std::env::current_dir()?, &binary).map_err(failure)?;
    print!("{}", plan.describe());
    if plan.is_noop() {
        println!("\nNothing to change: the semantic merge is already installed.");
        return Ok(());
    }
    if args.dry_run {
        return Ok(());
    }
    if !args.yes {
        print!("\nApply these changes? [y/N] ");
        std::io::stdout().flush()?;
        let mut answer = String::new();
        std::io::stdin().lock().read_line(&mut answer)?;
        if !matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
            println!("Nothing changed.");
            return Ok(());
        }
    }
    plan.apply().map_err(failure)?;
    println!("\nInstalled. `git merge` now merges .arc and .layout.json files by element.");
    Ok(())
}

pub fn run_merge(args: &MergeArgs) -> Result<(), CliError> {
    if args.install {
        return run_install(args);
    }
    let (Some(base), Some(ours), Some(theirs)) = (&args.base, &args.ours, &args.theirs) else {
        return Err(CliError::Config("merge needs <base> <ours> <theirs> (or --install)".into()));
    };
    let read = |p: &PathBuf| std::fs::read_to_string(p).map_err(|e| failure(format!("{}: {e}", p.display())));
    let (b, o, t) = (read(base)?, read(ours)?, read(theirs)?);
    let name = args.path.as_ref().or(args.output.as_ref()).or(Some(ours)).map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    let is_layout = name.ends_with(".layout.json") || (name.ends_with(".json") && o.trim_start().starts_with('{'));
    let (text, conflicts, notes): (String, Vec<merge::Conflict>, Vec<String>) = if is_layout {
        let merged = layout::merge_layout(&b, &o, &t).map_err(failure)?;
        let notes = merged.taken_from_theirs.iter().map(|e| format!("layout entry '{e}' changed on both sides: took theirs")).collect();
        (merged.text, Vec::new(), notes)
    } else {
        let merged = merge::merge_arc(&b, &o, &t).map_err(failure)?;
        (merged.text, merged.conflicts, Vec::new())
    };
    match &args.output {
        Some(path) => std::fs::write(path, &text)?,
        None if !args.json => print!("{text}"),
        None => {}
    }
    for note in &notes {
        eprintln!("arclang merge: {note}");
    }
    let mut compile_error: Option<String> = None;
    if conflicts.is_empty() && !is_layout && !args.no_compile {
        let place = args.path.clone().filter(|p| p.parent().is_some_and(|d| d.as_os_str().is_empty() || d.exists())).unwrap_or_else(|| ours.clone());
        if let Err(e) = crate::compiler::source_edit::compile_beside(&place, &text) {
            compile_error = Some(e);
        }
    }
    if args.json {
        let outcome = serde_json::json!({"clean": conflicts.is_empty() && compile_error.is_none(), "conflicts": conflicts, "notes": notes, "compile_error": compile_error, "text": if args.output.is_none() { Some(&text) } else { None }});
        println!("{}", serde_json::to_string_pretty(&outcome).unwrap_or_default());
    }
    if !conflicts.is_empty() {
        eprintln!("arclang merge: {} conflict(s), written as markers around the element:", conflicts.len());
        for c in &conflicts {
            eprintln!("  {}: {}", c.label, c.reason);
        }
        return Err(CliError::MergeConflict(format!("{} element conflict(s)", conflicts.len())));
    }
    if let Some(e) = compile_error {
        return Err(CliError::MergeConflict(format!("the merged model does not compile: {e}")));
    }
    Ok(())
}

#[derive(clap::Subcommand, Debug)]
pub enum BaselineCommand {
    /// Check the tree is clean and the model compiles, run the gate, write baselines/<name>/ and tag the commit baseline/<name>
    Create {
        name: String,
        /// iso26262, do178c or iec61508
        #[clap(long, default_value = "iso26262")]
        standard: String,
        /// The model file (default: the only .arc file at the top of the repository)
        #[clap(long)]
        model: Option<PathBuf>,
        /// Refuse to make a baseline of a model whose gate fails
        #[clap(long)]
        require_pass: bool,
        /// Allow uncommitted changes (the baseline is then not reproducible)
        #[clap(long)]
        allow_dirty: bool,
    },
    /// The baselines of the repository
    List {
        #[clap(long)]
        json: bool,
    },
    /// The semantic diff between two baselines, with their gate reports
    Diff {
        a: String,
        b: String,
        #[clap(short, long, value_enum, default_value_t = DiffFormat::Text)]
        format: DiffFormat,
    },
    /// Check that the files still match the manifest and the tag
    Verify {
        name: String,
        #[clap(long)]
        json: bool,
    },
}

fn find_model(explicit: &Option<PathBuf>) -> Result<PathBuf, CliError> {
    if let Some(path) = explicit {
        return Ok(path.clone());
    }
    let cwd = std::env::current_dir()?;
    let top = crate::compiler::semgit::git::run(&cwd, &["rev-parse", "--show-toplevel"]).map_err(failure)?;
    let files = crate::compiler::semgit::git::run(std::path::Path::new(top.trim()), &["ls-files", "*.arc"]).map_err(failure)?;
    let candidates: Vec<&str> = files.lines().filter(|l| !l.contains('/')).collect();
    let all: Vec<&str> = files.lines().collect();
    match (candidates.as_slice(), all.as_slice()) {
        ([one], _) | ([], [one]) => Ok(PathBuf::from(top.trim()).join(one)),
        _ => Err(CliError::Config(format!("which model? give --model <file> ({} .arc files are tracked)", all.len()))),
    }
}

pub fn run_baseline(command: &BaselineCommand) -> Result<(), CliError> {
    let cwd = std::env::current_dir()?;
    match command {
        BaselineCommand::Create { name, standard, model, require_pass, allow_dirty } => {
            let model = find_model(model)?;
            let created = baseline::create(&CreateOptions { name, standard, model: &model, require_pass: *require_pass, allow_dirty: *allow_dirty }).map_err(failure)?;
            let m = &created.manifest;
            println!("Baseline '{}' created ({}).", m.name, m.gate.standard);
            println!("  Gate: {} ({} blocker(s), {} warning(s)); requirements {} total, {} satisfied, {} verified", if m.gate.passed { "PASS" } else { "FAIL" }, m.gate.blockers, m.gate.warnings, m.gate.requirements_total, m.gate.requirements_satisfied, m.gate.requirements_verified);
            println!("  Commit: {}", m.git.commit);
            println!("  Tag: {} (not pushed: git push origin {})", created.tag, created.tag);
            println!("  Directory: {}", created.directory.display());
            for f in &m.files {
                println!("    {}  {}", &f.sha256[..12], f.path);
            }
            println!("  Commit baselines/{} to keep it with the model.", m.name);
            Ok(())
        }
        BaselineCommand::List { json } => {
            let all = baseline::list(&cwd).map_err(failure)?;
            if *json {
                println!("{}", serde_json::to_string_pretty(&all).unwrap_or_default());
            } else if all.is_empty() {
                println!("No baseline. `arclang baseline create <name>` makes one.");
            } else {
                for b in all {
                    println!("{:<24} {:<10} {:<10} {}  {}{}", b.name, b.standard, b.gate, b.commit.get(..10).unwrap_or(&b.commit), b.created, if b.tag { "" } else { "  (tag missing)" });
                }
            }
            Ok(())
        }
        BaselineCommand::Diff { a, b, format } => {
            let result = baseline::diff(&cwd, a, b).map_err(failure)?;
            match format {
                DiffFormat::Json => println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default()),
                other => {
                    let side = |g: &Option<baseline::GateSummary>| g.as_ref().map(|g| format!("{} ({}; {} blocker(s); {}/{} satisfied, {}/{} verified)", if g.passed { "PASS" } else { "FAIL" }, g.standard, g.blockers, g.requirements_satisfied, g.requirements_total, g.requirements_verified, g.requirements_total)).unwrap_or_else(|| "(no manifest)".into());
                    println!("Gate reports\n  {a}: {}\n  {b}: {}\n", side(&result.gate_a), side(&result.gate_b));
                    print!("{}", render::render(&result.diff, (*other).into()));
                }
            }
            Ok(())
        }
        BaselineCommand::Verify { name, json } => {
            let v = baseline::verify(&cwd, name).map_err(failure)?;
            if *json {
                println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
            } else {
                for c in &v.checks {
                    println!("  {} {}: {}", if c.ok { "ok  " } else { "FAIL" }, c.check, c.detail);
                }
                println!("\nBaseline '{}': {}", v.name, if v.ok { "VERIFIED" } else { "NOT VERIFIED" });
            }
            if v.ok {
                Ok(())
            } else {
                Err(failure(format!("baseline '{}' does not verify", v.name)))
            }
        }
    }
}

/// `arclang workbench <file> --rev <revision>`: the snapshot of the model as
/// committed, its layout file included, for the workbench's compare mode and
/// its element history.
pub fn run_workbench_at(input: &std::path::Path, revision: &str, gate_standard: &str, exports: bool) -> Result<(), CliError> {
    use crate::compiler::workbench::{snapshot_file_json, SnapshotOptions};
    let standard = match gate_standard.to_ascii_uppercase().replace(['-', '_', ' '], "").as_str() {
        "DO178C" => "DO178C",
        "IEC61508" => "IEC61508",
        _ => "ISO26262",
    };
    let loaded = rich::load(input, &Source::Git(revision.to_string())).map_err(failure)?;
    let mut json: serde_json::Value = serde_json::from_str(&snapshot_file_json(&loaded.path, &SnapshotOptions { gate_standard: standard, exports, explorer_html: false })).map_err(|e| failure(e.to_string()))?;
    if let (Some(layout), Some(diagrams)) = (loaded.layout.clone(), json.get_mut("diagrams").and_then(|d| d.as_object_mut())) {
        diagrams.insert("layout".into(), layout);
        let name = crate::compiler::diagram::layout_file::sidecar_path(&loaded.path);
        diagrams.insert("layout_name".into(), serde_json::json!(name.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()));
    }
    println!("{json}");
    Ok(())
}
