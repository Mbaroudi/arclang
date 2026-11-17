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
/// requirements tool could not tell which one it names. A requirement is
/// named by the text after its keyword and by its `id:`: neither may name
/// another requirement (one copied with its `id:` left as it was would be
/// verified by the cases of the first).
pub fn check_requirements(ast: &Model, files: &[SourceFile]) -> Result<(), String> {
    let mut seen: Vec<&str> = Vec::new();
    let mut first: Option<&str> = None;
    for requirement in ast.system_analysis.iter().flat_map(|sa| sa.requirements.iter()) {
        let name = requirement.id.as_str();
        let written = requirement.attributes.get("id").and_then(|v| v.as_string()).filter(|id| *id != name);
        for identity in std::iter::once(name).chain(written) {
            if first.is_none() && seen.contains(&identity) {
                first = Some(identity);
            }
        }
        seen.push(name);
        seen.extend(written);
    }
    let Some(first) = first else { return Ok(()) };
    let count = seen.iter().filter(|id| **id == first).count();
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

/// One declaration as it is written: its keyword, its name, the block it is written in, its `id:`.
struct Declared {
    keyword: String,
    name: String,
    /// The block the declaration is written in: the file and the rank of the block in it.
    within: (usize, usize),
    id: Option<String>,
    /// What follows its name: its title and its body, token by token.
    written: String,
    place: String,
}

/// The keyword as it is written: `BehaviorComponent` is `behavior_component`.
fn written(keyword: &str) -> String {
    let mut out = String::new();
    for (rank, letter) in keyword.chars().enumerate() {
        if letter.is_uppercase() && rank > 0 {
            out.push('_');
        }
        out.extend(letter.to_lowercase());
    }
    out
}

/// A line written twice is an error, not a model: a copy, word for word, of
/// a declaration in the block that already holds it, a declaration written again with its
/// `id:` anywhere in the model, a trace between the same two elements
/// written again. Nothing could tell
/// the two apart, and a tool or an assistant that repeats a line would go
/// unnoticed. Requirements are left to `check_requirements`, which also
/// reads their names as identities.
pub fn check_declarations(files: &[SourceFile]) -> Result<(), String> {
    let mut declared: Vec<Declared> = Vec::new();
    let mut traces: Vec<((String, String, String), String)> = Vec::new();
    for (file_rank, file) in files.iter().enumerate() {
        let Ok((tokens, spans)) = Lexer::new(&file.text).tokenize_spanned() else { continue };
        let place = |index: usize| {
            let line = spans.get(index).map_or(0, |span| span.line);
            if file.label.is_empty() { format!("line {line}") } else { format!("{}:{line}", file.label) }
        };
        // The blocks open at each token: their rank, and the declaration each one is the body of.
        let mut open: Vec<(usize, Option<(usize, usize)>)> = vec![(0, None)];
        let mut blocks = 0;
        let mut body_of: Option<(usize, usize)> = None;
        for (index, token) in tokens.iter().enumerate() {
            match token {
                Token::LeftBrace => {
                    blocks += 1;
                    open.push((blocks, body_of.take()));
                }
                Token::RightBrace if open.len() > 1 => {
                    if let Some((_, Some((owner, from)))) = open.pop() {
                        declared[owner].written = tokens[from..=index].iter().map(|token| format!("{token:?}")).collect::<Vec<_>>().join(" ");
                    }
                }
                Token::Trace => {
                    if let (Some(Token::StringLiteral(from)), Some(kind), Some(Token::StringLiteral(to))) = (tokens.get(index + 1), tokens.get(index + 2), tokens.get(index + 3)) {
                        traces.push(((from.clone(), format!("{kind:?}"), to.clone()), place(index)));
                    }
                }
                Token::Identifier(key) if key == "id" => {
                    let owner = open.last().and_then(|(_, owner)| *owner);
                    if let (Some((owner, _)), Some(Token::Colon), Some(Token::StringLiteral(id))) = (owner, tokens.get(index + 1), tokens.get(index + 2)) {
                        declared[owner].id.get_or_insert_with(|| id.clone());
                    }
                }
                _ => {
                    let keyword = format!("{token:?}");
                    if !DECLARING.contains(&keyword.as_str()) || keyword == "Requirement" || keyword == "Req" {
                        continue;
                    }
                    let name = match tokens.get(index + 1) {
                        Some(Token::StringLiteral(name)) | Some(Token::Identifier(name)) => name.clone(),
                        _ => continue,
                    };
                    // A declaration opens its block after its name, or after its name and its title.
                    let opens = match (tokens.get(index + 2), tokens.get(index + 3)) {
                        (Some(Token::LeftBrace), _) => true,
                        (Some(Token::StringLiteral(_)), Some(Token::LeftBrace)) => true,
                        _ => false,
                    };
                    // `provides interface "X" {}` names an interface, it does not declare one.
                    let used = index > 0 && matches!(tokens[index - 1], Token::Provides | Token::Requires | Token::Implements);
                    if !opens || used {
                        continue;
                    }
                    let within = (file_rank, open.last().map_or(0, |(rank, _)| *rank));
                    declared.push(Declared { keyword: written(&keyword), name, within, id: None, written: String::new(), place: place(index) });
                    body_of = Some((declared.len() - 1, index + 2));
                }
            }
        }
    }
    for (rank, one) in declared.iter().enumerate() {
        for other in &declared[..rank] {
            // Two different elements under one id are the warning `duplicate element id`; the same one twice is a copy.
            if one.id.is_some() && one.id == other.id && one.keyword == other.keyword && one.name == other.name {
                return Err(format!("{} '{}' (id '{}') is declared twice: {} and {} — keep one, or give each its own id", one.keyword, one.name, one.id.as_deref().unwrap_or_default(), other.place, one.place));
            }
            // With no id, two declarations of one name that differ in what they say are told apart by it (two exchanges between the same two components).
            if one.within == other.within && one.keyword == other.keyword && one.name == other.name && one.id.is_none() && other.id.is_none() && one.written == other.written {
                return Err(format!("{} '{}' is written twice in the same block, word for word: {} and {} — keep one", one.keyword, one.name, other.place, one.place));
            }
        }
    }
    for (rank, (trace, place)) in traces.iter().enumerate() {
        if let Some((_, first)) = traces[..rank].iter().find(|(other, _)| other == trace) {
            return Err(format!("the trace from '{}' to '{}' is written twice: {first} and {place} — keep one", trace.0, trace.2));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checked(text: &str) -> Result<(), String> {
        check_declarations(&[SourceFile { label: "m.arc".to_string(), text: text.to_string() }])
    }

    #[test]
    fn a_declaration_copied_in_its_block_is_refused_with_both_lines() {
        let error = checked("logical_architecture \"L\" {\n    component \"Pump\" { }\n    component \"Pump\" { }\n}\n").unwrap_err();
        assert_eq!(error, "component 'Pump' is written twice in the same block, word for word: m.arc:2 and m.arc:3 — keep one");
        // The same name saying something else is another element.
        assert_eq!(checked("logical_architecture \"L\" {\n    interface \"A_to_A\" { item: \"x\" }\n    interface \"A_to_A\" { item: \"y\" }\n}\n"), Ok(()));
    }

    #[test]
    fn a_declaration_written_again_with_its_id_is_refused_wherever_it_is() {
        let error = checked("logical_architecture \"L\" {\n    component \"Pump\" { id: \"LC-1\"\n        function \"Move\" { id: \"LF-1\" }\n    }\n    component \"Valve\" {\n        function \"Move\" \"Move the water\" { id: \"LF-1\" }\n    }\n}\n").unwrap_err();
        assert_eq!(error, "function 'Move' (id 'LF-1') is declared twice: m.arc:3 and m.arc:6 — keep one, or give each its own id");
        // Two different elements under one id are left to the warning the compiler already gives.
        assert_eq!(checked("logical_architecture \"L\" {\n    component \"Pump\" { id: \"LC-1\" }\n    component \"Valve\" { id: \"LC-1\" }\n}\n"), Ok(()));
    }

    #[test]
    fn a_trace_written_again_is_refused() {
        let error = checked("trace \"LC-1\" satisfies \"R1\" { }\ntrace \"LC-1\" satisfies \"R2\" { }\ntrace \"LC-1\" satisfies \"R1\" { }\n").unwrap_err();
        assert_eq!(error, "the trace from 'LC-1' to 'R1' is written twice: m.arc:1 and m.arc:3 — keep one");
    }

    #[test]
    fn what_can_be_told_apart_is_taken() {
        // The same name in two blocks, the same name under two ids, a name used and not declared, an `id:` of a nested block.
        let model = "logical_architecture \"L\" {\n    component \"A\" { id: \"LC-1\" function \"Run\" { } }\n    component \"B\" { id: \"LC-2\" function \"Run\" { } }\n    component \"Valve\" { id: \"LC-3\" }\n    component \"Valve\" { id: \"LC-4\" }\n    interface \"Flow\" { from: \"A\" to: \"B\" metadata { id: \"LC-1\" } }\n}\ntrace \"LC-1\" satisfies \"R1\" { }\ntrace \"LC-2\" satisfies \"R1\" { }\n";
        assert_eq!(checked(model), Ok(()));
        // An interface a component both provides and requires is named twice, not declared twice.
        assert_eq!(checked("logical_architecture \"L\" {\n    component \"A\" {\n        provides interface \"Loop\" {}\n        requires interface \"Loop\" {}\n    }\n}\n"), Ok(()));
    }
}
