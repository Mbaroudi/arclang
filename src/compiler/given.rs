//! The files a model text imports, given in place of a file system: the
//! browser has none, so the page hands the compiler the text of each file
//! the model reaches, by path relative to the model's folder. A text that
//! imports is then assembled as its file would be on disk.

use std::cell::RefCell;
use std::collections::HashMap;

use super::identity_sites::SourceFile;
use super::{ast, Compiler, CompilerError};

/// The model's own file name (empty when it has none) and the files it may
/// import, each under its path relative to the model's folder (`a.arc`,
/// `parts/b.arc`, `../shared/c.arc`).
pub struct Given {
    pub name: String,
    pub files: HashMap<String, String>,
}

thread_local! {
    static GIVEN: RefCell<Option<Given>> = const { RefCell::new(None) };
}

/// Declares the files the texts compiled next may import, until it is called
/// again; `None` takes them back.
pub fn give(given: Option<Given>) {
    GIVEN.with(|cell| *cell.borrow_mut() = given);
}

/// `import` as written in the file `from`, as a path relative to the model's folder.
fn resolved(from: &str, import: &str) -> String {
    let mut parts: Vec<&str> = match from.rfind('/') {
        Some(at) => from[..at].split('/').collect(),
        None => Vec::new(),
    };
    for part in import.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." if parts.last().is_some_and(|last| *last != "..") => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

fn located(error: CompilerError, file: &str) -> CompilerError {
    match error {
        CompilerError::Parser(message) => CompilerError::Parser(format!("{file}: {message}")),
        CompilerError::Lexer(message) => CompilerError::Lexer(format!("{file}: {message}")),
        other => other,
    }
}

fn missing(from: &str, import: &str) -> CompilerError {
    let importer = if from.is_empty() { String::new() } else { format!("{from}: ") };
    CompilerError::Parser(format!("{importer}imported file not found: {import}"))
}

fn fragment(path: &str, given: &Given, stack: &mut Vec<String>, files: &mut Vec<SourceFile>) -> Result<(ast::Model, Vec<String>), CompilerError> {
    if stack.iter().any(|open| open == path) {
        let chain: Vec<&str> = stack.iter().map(String::as_str).chain(std::iter::once(path)).collect();
        return Err(CompilerError::Parser(format!("circular import: {}", chain.join(" -> "))));
    }
    // A file imported along two paths (a diamond) is read once.
    if files.iter().any(|file| file.label == path) {
        return Ok((ast::Model::new(), Vec::new()));
    }
    let Some(text) = given.files.get(path) else {
        return Err(missing(stack.last().map(String::as_str).unwrap_or_default(), path));
    };
    files.push(SourceFile { label: path.to_string(), text: text.clone() });
    let (mut model, mut warnings) = Compiler::parse_source(text).map_err(|error| located(error, path))?;
    stack.push(path.to_string());
    for import in std::mem::take(&mut model.imports) {
        let (imported, more) = fragment(&resolved(path, &import), given, stack, files)?;
        model.merge(imported);
        warnings.extend(more);
    }
    stack.pop();
    Ok((model, warnings))
}

/// The model `root` (parsed from `source`) with the files it imports merged
/// in, read from those given; `None` when no file was given, which leaves
/// the text with imports it cannot follow.
pub(super) fn assemble(mut root: ast::Model, source: &str) -> Option<Result<(ast::Model, Vec<String>, Vec<SourceFile>), CompilerError>> {
    GIVEN.with(|cell| {
        let given = cell.borrow();
        let given = given.as_ref()?;
        let mut files = vec![SourceFile { label: given.name.clone(), text: source.to_string() }];
        let mut stack = vec![given.name.clone()];
        let mut warnings = Vec::new();
        for import in std::mem::take(&mut root.imports) {
            match fragment(&resolved(&given.name, &import), given, &mut stack, &mut files) {
                Ok((imported, more)) => {
                    root.merge(imported);
                    warnings.extend(more);
                }
                Err(error) => return Some(Err(error)),
            }
        }
        Some(Ok((root, warnings, files)))
    })
}

#[cfg(test)]
mod tests {
    use super::resolved;

    #[test]
    fn an_import_is_resolved_beside_the_file_that_writes_it() {
        assert_eq!(resolved("main.arc", "parts/a.arc"), "parts/a.arc");
        assert_eq!(resolved("parts/a.arc", "b.arc"), "parts/b.arc");
        assert_eq!(resolved("parts/a.arc", "../shared/c.arc"), "shared/c.arc");
        assert_eq!(resolved("main.arc", "../shared/c.arc"), "../shared/c.arc");
        assert_eq!(resolved("", "./a.arc"), "a.arc");
    }
}
