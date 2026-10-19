//! Where an identity is declared, file and line: for the errors and
//! warnings about two elements of one identity, which must name both
//! places so that the author can tell them apart.

use super::ast::Model;
use super::lexer::{Lexer, Token};

/// One source text of a model: its label (the file, empty for a text with
/// no file) and its content.
pub struct SourceFile {
    pub label: String,
    pub text: String,
}

/// Keywords whose following text is the identity of what they declare,
/// when the declaration writes no `id:`.
const DECLARING: &[&str] = &[
    "Requirement", "Req", "Component", "Function", "Actor", "Node", "Entity", "BehaviorComponent",
    "HardwareComponent", "Activity", "Capability", "Mission", "Constraint", "Interface",
];

/// The places (`file:line`, or `line N` for a text with no file) where
/// `identity` is declared: `id: "X"`, or the text after a declaring keyword.
pub fn sites(files: &[SourceFile], identity: &str) -> Vec<String> {
    let mut out = Vec::new();
    for file in files {
        let Ok((tokens, spans)) = Lexer::new(&file.text).tokenize_spanned() else { continue };
        let mut lines: Vec<usize> = Vec::new();
        for (index, token) in tokens.iter().enumerate() {
            let Token::StringLiteral(text) = token else { continue };
            if text != identity {
                continue;
            }
            let after_id = index >= 2 && matches!(&tokens[index - 1], Token::Colon) && matches!(&tokens[index - 2], Token::Identifier(key) if key == "id");
            let after_keyword = index >= 1 && DECLARING.contains(&format!("{:?}", tokens[index - 1]).as_str());
            let Some(span) = spans.get(index) else { continue };
            if (after_id || after_keyword) && !lines.contains(&span.line) {
                lines.push(span.line);
            }
        }
        for line in lines {
            out.push(if file.label.is_empty() { format!("line {line}") } else { format!("{}:{line}", file.label) });
        }
    }
    out
}

fn listed(places: &[String]) -> String {
    match places {
        [] => "in places not found".to_string(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

/// Two requirements of one identity are an error: a trace, a test case or a
/// requirements tool could not tell which one it names.
pub fn check_requirements(ast: &Model, files: &[SourceFile]) -> Result<(), String> {
    let mut seen: Vec<&str> = Vec::new();
    let mut reported: Vec<&str> = Vec::new();
    for requirement in ast.system_analysis.iter().flat_map(|sa| sa.requirements.iter()) {
        let id = requirement.id.as_str();
        if seen.contains(&id) && !reported.contains(&id) {
            reported.push(id);
        }
        seen.push(id);
    }
    let Some(first) = reported.first() else { return Ok(()) };
    let count = seen.iter().filter(|id| *id == first).count();
    let times = if count == 2 { "twice".to_string() } else { format!("{count} times") };
    Err(format!("requirement '{first}' is declared {times}: {} — give each its own id", listed(&sites(files, first))))
}

/// The warning about a duplicate identity, with the places it is declared.
pub fn locate_duplicate(warning: String, files: &[SourceFile]) -> String {
    let Some(rest) = warning.strip_prefix("duplicate element id '") else { return warning };
    let Some(identity) = rest.split('\'').next() else { return warning };
    let places = sites(files, identity);
    if places.len() < 2 {
        return warning;
    }
    format!("{warning} (declared at {})", listed(&places))
}
