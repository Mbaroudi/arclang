//! `arclang table`: the live tables of a model, computed by the compiler
//! and written as JSON (with ids and UUIDs for linking), CSV, TSV,
//! Markdown, HTML or an Excel workbook.

use super::CliError;
use crate::compiler::tables::{self, export, xlsx, Options, Table, TableSpec};
use crate::{Compiler, CompilerConfig};
use std::path::PathBuf;

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableFormat {
    Json,
    Csv,
    Tsv,
    Md,
    Html,
    Xlsx,
}

#[derive(clap::Args, Debug)]
pub struct TableArgs {
    #[clap(value_parser)]
    pub input: PathBuf,

    /// A table the model declares, by name or id (default: every declared table)
    #[clap(long)]
    pub name: Option<String>,

    /// Compute a table the model does not declare: requirements_coverage,
    /// traceability_matrix, allocation_matrix, interface_n2, icd or elements
    #[clap(long)]
    pub kind: Option<String>,

    /// What the rows are taken from: a layer (OA, SA, LA, PA, EPBS) or an element
    #[clap(long)]
    pub scope: Option<String>,

    /// The kind of the rows (`elements`), e.g. Requirement or LogicalComponent
    #[clap(long)]
    pub row_type: Option<String>,

    /// The columns shown (comma separated); for a matrix, the kinds of its columns
    #[clap(long, value_delimiter = ',')]
    pub columns: Vec<String>,

    /// For a matrix: the kinds (or layers) of its rows (comma separated)
    #[clap(long, value_delimiter = ',')]
    pub rows: Vec<String>,

    /// For a traceability matrix: the trace kinds (comma separated)
    #[clap(long, value_delimiter = ',')]
    pub kinds: Vec<String>,

    /// Keep the rows matching: words, column:text, column=text, column!=text, is:gap, -term
    #[clap(long, allow_hyphen_values = true)]
    pub filter: Option<String>,

    /// Keep only the rows with a gap or a failure
    #[clap(long)]
    pub gaps: bool,

    #[clap(short, long, value_enum, default_value = "json")]
    pub format: TableFormat,

    /// Write to this file instead of stdout (needed for xlsx)
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// The standard the gate column is computed against: ISO26262, DO178C or IEC61508
    #[clap(long, default_value = "ISO26262")]
    pub gate_standard: String,

    /// List the tables the model declares and the kinds the compiler computes
    #[clap(long)]
    pub list: bool,

    /// Print the `table` declaration of the table asked for, as ArcLang source
    #[clap(long)]
    pub declaration: bool,
}

impl TableArgs {
    /// The spec asked for: the declared one, else the kind given, with the
    /// options of the command line over it.
    fn spec(&self, declared: Option<TableSpec>) -> Option<TableSpec> {
        let mut spec = match (declared, &self.kind) {
            (Some(spec), _) => spec,
            (None, Some(kind)) => TableSpec { kind: kind.clone(), ..TableSpec::default() },
            (None, None) => return None,
        };
        if let Some(kind) = &self.kind {
            spec.kind = kind.clone();
        }
        if self.scope.is_some() {
            spec.scope = self.scope.clone();
        }
        if self.row_type.is_some() {
            spec.row_type = self.row_type.clone();
        }
        if !self.columns.is_empty() {
            if tables::is_matrix(&spec.kind) {
                spec.matrix_columns = self.columns.clone();
            } else {
                spec.columns = self.columns.clone();
            }
        }
        if !self.rows.is_empty() {
            spec.rows = self.rows.clone();
        }
        if !self.kinds.is_empty() {
            spec.kinds = self.kinds.clone();
        }
        if self.filter.is_some() {
            spec.filter = self.filter.clone();
        }
        spec.gaps_only |= self.gaps;
        if spec.name.is_none() && self.kind.is_some() {
            spec.name = self.name.clone();
        }
        Some(spec)
    }
}

pub fn run(args: &TableArgs) -> Result<(), CliError> {
    let result = Compiler::new(CompilerConfig::default())
        .compile_file(&args.input)
        .map_err(|e| CliError::Compilation(format!("{}: {e}", args.input.display())))?;
    let options = Options { gate_standard: args.gate_standard.clone() };

    if args.list {
        let listing = serde_json::json!({
            "declared": result.ast.tables.iter().map(|t| serde_json::json!({"name": t.name, "id": t.id, "kind": t.kind, "scope": t.attributes.get("scope").and_then(|v| v.as_string())})).collect::<Vec<_>>(),
            "kinds": tables::KINDS,
        });
        println!("{}", serde_json::to_string_pretty(&listing).unwrap_or_default());
        return Ok(());
    }

    let declared = match &args.name {
        Some(name) => {
            let decl = result.ast.tables.iter().find(|t| &t.name == name || &t.id == name);
            match (decl, &args.kind) {
                (Some(decl), _) => Some(TableSpec::of(decl)),
                (None, Some(_)) => None,
                (None, None) => {
                    let known: Vec<&str> = result.ast.tables.iter().map(|t| t.name.as_str()).collect();
                    return Err(CliError::Config(format!("the model declares no table '{name}' (declared: {})", if known.is_empty() { "none".to_string() } else { known.join(", ") })));
                }
            }
        }
        None => None,
    };
    let single = args.name.is_some() || args.kind.is_some();
    let computed: Vec<Table> = match args.spec(declared) {
        Some(spec) => {
            if args.declaration {
                println!("{}", spec.declaration());
                return Ok(());
            }
            vec![tables::compute(&result, &spec, &options).map_err(CliError::Config)?]
        }
        None => {
            if result.ast.tables.is_empty() {
                return Err(CliError::Config(format!(
                    "{} declares no table: name a kind with --kind ({})",
                    args.input.display(),
                    tables::KINDS.join(", ")
                )));
            }
            let mut all = tables::declared_tables(&result, &options).map_err(CliError::Config)?;
            if args.gaps || args.filter.is_some() {
                all = result
                    .ast
                    .tables
                    .iter()
                    .map(|decl| args.spec(Some(TableSpec::of(decl))).expect("a declared spec"))
                    .map(|spec| tables::compute(&result, &spec, &options))
                    .collect::<Result<_, _>>()
                    .map_err(CliError::Config)?;
            }
            all
        }
    };
    let title = result.semantic_model.name.clone().unwrap_or_else(|| "Model".to_string());
    let bytes: Vec<u8> = match args.format {
        TableFormat::Json => {
            let value = if single { serde_json::to_value(&computed[0]) } else { serde_json::to_value(&computed) };
            let text = value.and_then(|v| serde_json::to_string_pretty(&v)).map_err(|e| CliError::Config(e.to_string()))?;
            format!("{text}\n").into_bytes()
        }
        TableFormat::Csv | TableFormat::Tsv => {
            if computed.len() > 1 {
                return Err(CliError::Config(format!("{} tables are declared: pick one with --name (CSV and TSV hold one table)", computed.len())));
            }
            if args.format == TableFormat::Csv { export::csv(&computed[0]) } else { export::tsv(&computed[0]) }.into_bytes()
        }
        TableFormat::Md => computed.iter().map(export::markdown).collect::<Vec<_>>().join("\n").into_bytes(),
        TableFormat::Html => export::html(&computed, &format!("{title} — tables")).into_bytes(),
        TableFormat::Xlsx => {
            if args.output.is_none() {
                return Err(CliError::Config("an Excel workbook is written to a file: give -o <file>.xlsx".to_string()));
            }
            xlsx::workbook(&computed)
        }
    };
    match &args.output {
        Some(path) => {
            std::fs::write(path, &bytes)?;
            eprintln!("wrote {} ({} table{}, {} bytes)", path.display(), computed.len(), if computed.len() == 1 { "" } else { "s" }, bytes.len());
        }
        None => {
            use std::io::Write;
            std::io::stdout().write_all(&bytes)?;
        }
    }
    Ok(())
}
