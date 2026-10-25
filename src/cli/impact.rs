//! `arclang impact`: what a change reaches, computed by the compiler.
//!
//! `arclang impact model.arc REQ-1` still prints what it always printed
//! (the elements by distance); the seeds may now be several (`--element`),
//! a proposed change (`--change`, `arclang edit` requests, or `--delete`),
//! or the changes since a git revision (`--since HEAD`), and the report is
//! also JSON, Markdown, HTML or an Excel workbook.

use super::CliError;
use crate::compiler::impact::{self, report, Direction, Options};
use crate::compiler::workbench::EditRequest;
use std::path::PathBuf;

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImpactFormat {
    Text,
    Json,
    Md,
    Html,
    Tsv,
    Xlsx,
}

#[derive(clap::Args, Debug)]
pub struct ImpactArgs {
    #[clap(value_parser)]
    pub input: PathBuf,

    /// Element id or unambiguous name (e.g. "REQ-AEB-004"); same as --element
    #[clap(value_parser)]
    pub element: Option<String>,

    /// A changed element (repeatable): its id, unique name or Owner.port
    #[clap(long = "element", short = 'e')]
    pub elements: Vec<String>,

    /// A proposed change, never written: one `arclang edit` request or a
    /// JSON array of them, inline, `@file.json`, or `-` for stdin
    #[clap(long)]
    pub change: Option<String>,

    /// What if this element were deleted, with the traces and list entries naming it
    #[clap(long)]
    pub delete: Option<String>,

    /// The changes since a git revision (HEAD, main, a commit)
    #[clap(long)]
    pub since: Option<String>,

    /// With --since: the current text of the model, when it is not the file
    /// on disk (an editor's unsaved buffer)
    #[clap(long)]
    pub current: Option<PathBuf>,

    /// How far the change is followed (links from a seed)
    #[clap(long, default_value_t = impact::DEFAULT_DEPTH)]
    pub depth: usize,

    /// The kinds of link followed, comma separated (default: all): realization,
    /// allocation, deployment, exchange, interface, chain, satisfies, verifies,
    /// safety, containment, typing, constraint, view, behavior, reference
    #[clap(long, value_delimiter = ',')]
    pub kinds: Vec<String>,

    /// both, downstream (what depends on the seed) or upstream (what it depends on)
    #[clap(long, default_value = "both")]
    pub direction: String,

    /// The standard of the production gate: ISO26262, DO178C or IEC61508
    #[clap(long, default_value = "ISO26262")]
    pub gate_standard: String,

    #[clap(short, long, value_enum, default_value_t = ImpactFormat::Text)]
    pub format: ImpactFormat,

    /// Write to this file instead of stdout (required for xlsx)
    #[clap(short, long)]
    pub output: Option<PathBuf>,
}

fn change_requests(text: &str) -> Result<Vec<EditRequest>, CliError> {
    let text = if text.trim() == "-" {
        let mut all = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut all)?;
        all
    } else if let Some(file) = text.strip_prefix('@') {
        std::fs::read_to_string(file)?
    } else {
        text.to_string()
    };
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| CliError::Config(format!("--change is not JSON: {e}")))?;
    let items = match value {
        serde_json::Value::Array(items) => items,
        one => vec![one],
    };
    items
        .into_iter()
        .map(|item| serde_json::from_value::<EditRequest>(item).map_err(|e| CliError::Config(format!("--change: invalid edit request: {e}"))))
        .collect()
}

fn standard(text: &str) -> String {
    match text.to_ascii_uppercase().replace(['-', '_', ' '], "").as_str() {
        "DO178C" => "DO178C".to_string(),
        "IEC61508" => "IEC61508".to_string(),
        _ => "ISO26262".to_string(),
    }
}

pub fn run(args: &ImpactArgs) -> Result<(), CliError> {
    let options = Options {
        depth: args.depth.clamp(1, impact::MAX_DEPTH),
        kinds: Options::parse_kinds(&args.kinds).map_err(CliError::Config)?,
        direction: Direction::parse(&args.direction).ok_or_else(|| CliError::Config(format!("unknown direction '{}': both, downstream or upstream", args.direction)))?,
        gate_standard: standard(&args.gate_standard),
    };
    let mut elements = args.elements.clone();
    elements.extend(args.element.clone());
    let source = std::fs::read_to_string(&args.input)?;
    let model = args.input.display().to_string();

    let mut requests = match &args.change {
        Some(text) => change_requests(text)?,
        None => Vec::new(),
    };
    let compile = || crate::Compiler::new(crate::CompilerConfig::default()).compile_file(&args.input).map_err(|e| CliError::Compilation(e.to_string()));
    if let Some(element) = &args.delete {
        let result = compile()?;
        requests.extend(impact::cascade_delete(&result, element).map_err(CliError::Compilation)?);
    }

    let outcome = if !requests.is_empty() {
        if args.since.is_some() {
            return Err(CliError::Config("--since and a proposed change (--change, --delete) are not combined".to_string()));
        }
        impact::what_if(Some(&args.input), &source, requests, &options).map(|mut r| {
            r.model = model.clone();
            r
        })
    } else if let Some(revision) = &args.since {
        let current = match &args.current {
            Some(path) => std::fs::read_to_string(path)?,
            None => source.clone(),
        };
        impact::since::since(&args.input, &current, revision, &options)
    } else {
        if elements.is_empty() {
            return Err(CliError::Config("name an element (positional or --element), a change (--change, --delete) or a revision (--since)".to_string()));
        }
        let result = compile()?;
        impact::analyze(&result, &model, &elements, &options)
    };

    let report = match outcome {
        Ok(report) => report,
        Err(error) => {
            if args.format == ImpactFormat::Json {
                println!("{}", serde_json::json!({ "success": false, "error": error }));
            }
            return Err(CliError::Compilation(error));
        }
    };

    let bytes: Vec<u8> = match args.format {
        ImpactFormat::Text => report::text(&report).into_bytes(),
        ImpactFormat::Json => {
            let mut value = serde_json::to_value(&report).map_err(|e| CliError::Compilation(e.to_string()))?;
            value["success"] = serde_json::Value::Bool(true);
            format!("{}\n", serde_json::to_string_pretty(&value).map_err(|e| CliError::Compilation(e.to_string()))?).into_bytes()
        }
        ImpactFormat::Md => report::markdown(&report).into_bytes(),
        ImpactFormat::Html => report::html(&report).into_bytes(),
        ImpactFormat::Tsv => report::tsv(&report).into_bytes(),
        ImpactFormat::Xlsx => report::xlsx(&report),
    };
    match &args.output {
        Some(path) => {
            std::fs::write(path, &bytes)?;
            eprintln!("{} → {}", report::headline(&report), path.display());
        }
        None if args.format == ImpactFormat::Xlsx => return Err(CliError::Config("an Excel workbook is written to a file: add -o report.xlsx".to_string())),
        None => {
            use std::io::Write;
            std::io::stdout().write_all(&bytes)?;
        }
    }
    Ok(())
}
