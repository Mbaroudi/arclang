//! Import of SysML v2 models into ArcLang. The pivot is the element listing
//! of the Systems Modeling API (JSON records): a `.json` file holds one, and
//! a `.sysml` text is read into one first (today the dialect ArcLang
//! exports; the general notation follows). The records become an Arcadia
//! model ([`arcadia`]) written as ArcLang source, which must compile: the
//! import fails rather than hand over a model the compiler refuses.

pub mod arcadia;
mod expression;
pub mod records;

use crate::compiler::{Compiler, CompilerConfig};
use std::collections::BTreeMap;
use std::path::Path;

/// What an import hands over: the source and the report.
pub struct Imported {
    pub source: String,
    pub counts: BTreeMap<&'static str, usize>,
    pub notes: Vec<String>,
}

/// Why an import failed, with the refused text when the mapping wrote one.
pub struct ImportFailure {
    pub message: String,
    pub rejected: Option<String>,
}

impl From<String> for ImportFailure {
    fn from(message: String) -> Self {
        ImportFailure { message, rejected: None }
    }
}

/// Import a `.json` listing of SysML v2 API records, a `.sysml` or
/// `.kerml` text, or a directory: the `.sysml` and `.kerml` files under it,
/// one model.
pub fn import_file(path: &Path) -> Result<Imported, ImportFailure> {
    if path.is_dir() {
        return import_directory(path);
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let is_text = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("sysml") || e.eq_ignore_ascii_case("kerml"));
    if is_text {
        import_text(&text)
    } else {
        import_json(&text)
    }
}

/// The `.sysml` and `.kerml` files under a directory, in name order, as one model.
fn import_directory(dir: &Path) -> Result<Imported, ImportFailure> {
    fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("sysml") || e.eq_ignore_ascii_case("kerml")) {
                out.push(path);
            }
        }
    }
    let mut paths = Vec::new();
    walk(dir, &mut paths);
    paths.sort();
    if paths.is_empty() {
        return Err(format!("no .sysml or .kerml file under {}", dir.display()).into());
    }
    let mut files = Vec::new();
    for path in &paths {
        let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let name = path.strip_prefix(dir).unwrap_or(path).display().to_string();
        files.push((name, text));
    }
    let borrowed: Vec<(&str, &str)> = files.iter().map(|(n, t)| (n.as_str(), t.as_str())).collect();
    import_files(&borrowed)
}

/// Import SysML v2 textual notation, read by the complete parser
/// (`sysml_parser`): any KerML / SysML v2 text.
pub fn import_text(text: &str) -> Result<Imported, ImportFailure> {
    import_files(&[("", text)])
}

/// Import the files of one model (`name`, text): names resolve across them.
pub fn import_files(files: &[(&str, &str)]) -> Result<Imported, ImportFailure> {
    let built = crate::compiler::sysml_parser::records_of_files(files).map_err(|e| format!("SysML text not read: {e}"))?;
    let mut imported = import_records(&built.records)?;
    let unresolved = built.unresolved.len();
    if unresolved > 0 {
        imported.counts.insert("names designating nothing known (left unresolved)", unresolved);
        for note in built.unresolved.into_iter().take(20) {
            imported.notes.push(note);
        }
    }
    Ok(imported)
}

/// Import a JSON listing of API records.
pub fn import_json(text: &str) -> Result<Imported, ImportFailure> {
    let json: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("not JSON: {e}"))?;
    let records = records::records_of(&json)?;
    import_records(&records)
}

/// Import API records already in memory.
pub fn import_records(records: &[serde_json::Value]) -> Result<Imported, ImportFailure> {
    let graph = records::load(records)?;
    let mapped = arcadia::map(&graph);
    verify(mapped)
}

/// The written source must compile; else the import fails and keeps the text.
fn verify(mapped: arcadia::Mapped) -> Result<Imported, ImportFailure> {
    let arcadia::Mapped { source, counts, mut notes } = mapped;
    match Compiler::new(CompilerConfig::default()).compile_string(&source) {
        Ok(result) => {
            for warning in result.warnings.iter().take(12) {
                notes.push(format!("compiler warning: {warning}"));
            }
            if result.warnings.len() > 12 {
                notes.push(format!("{} more compiler warnings", result.warnings.len() - 12));
            }
            Ok(Imported { source, counts, notes })
        }
        Err(error) => Err(ImportFailure { message: format!("the imported model does not compile: {error}"), rejected: Some(source) }),
    }
}
