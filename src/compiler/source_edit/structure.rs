//! Edits that add or remove a whole declaration: an element block, or a
//! trace. Like attribute edits they splice the text and keep everything
//! else as written, and they are checked on the syntax tree: the edited
//! model must hold exactly one declaration more, or one less, and be
//! otherwise identical.
//!
//! A removed block goes with what is inside it, comments included; comments
//! around it stay.

use super::*;

/// Where a new element is declared.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Container<'a> {
    /// Inside the block of an element, designated like in attribute edits.
    Element(&'a str),
    /// Inside a layer block (`logical_architecture "Name" { }`). The name
    /// may be omitted when the file has a single block of that keyword.
    Layer { keyword: &'a str, name: Option<&'a str> },
}

/// An ArcLang string literal for `text`.
pub fn string_literal(text: &str) -> String {
    let mut literal = String::with_capacity(text.len() + 2);
    literal.push('"');
    for character in text.chars() {
        match character {
            '"' => literal.push_str("\\\""),
            '\\' => literal.push_str("\\\\"),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\t' => literal.push_str("\\t"),
            other => literal.push(other),
        }
    }
    literal.push('"');
    literal
}

fn check_keyword(keyword: &str) -> Result<(), EditError> {
    let is_keyword = !keyword.is_empty() && keyword.chars().all(|c| c.is_ascii_lowercase() || c == '_');
    if is_keyword {
        Ok(())
    } else {
        Err(EditError::Invalid(format!("'{keyword}' is not a declaration keyword")))
    }
}

impl Document {
    fn find_layer(&self, keyword: &str, name: Option<&str>) -> Result<Block, EditError> {
        let described = match name {
            Some(name) => format!("{keyword} \"{name}\""),
            None => keyword.to_string(),
        };
        let matches: Vec<Block> = self
            .blocks()
            .into_iter()
            .filter(|block| self.header(block).first().is_some_and(|first| first.text == keyword))
            .filter(|block| name.is_none() || self.header_name(block).as_deref() == name)
            .collect();
        match matches.len() {
            0 => Err(EditError::NotFound(described)),
            1 => Ok(matches.into_iter().next().expect("one match")),
            _ => Err(EditError::Ambiguous {
                element: described,
                lines: matches.iter().map(|block| self.lexemes[block.open].span.line).collect(),
            }),
        }
    }

    /// Index of the first lexeme of a block's declaration line.
    fn header_start(&self, block: &Block) -> usize {
        block.open - self.header(block).len()
    }

    /// Remove lexemes `first..=last` with the line they stand on, when they
    /// have it to themselves.
    fn remove_range(&self, first: usize, last: usize) -> String {
        let starts_line = first == 0 || self.lexemes[first].newlines_before > 0;
        let next = self.lexemes.get(last + 1);
        let ends_line = next.map_or(true, |next| next.newlines_before > 0);
        if starts_line && ends_line {
            let start = if first == 0 { 0 } else { self.end_of(first - 1) };
            // Removing the first declaration must not leave a blank line on top.
            let end = match (first, next) {
                (0, Some(next)) => next.offset,
                _ => self.end_of(last),
            };
            return self.splice(start, end, "");
        }
        // It shares its line: what follows (a trailing comment) stays put.
        let end = next.map_or(self.end_of(last), |next| if next.newlines_before > 0 { self.end_of(last) } else { next.offset });
        self.splice(self.lexemes[first].offset, end, "")
    }

    /// Add `declaration` (one or several lines, not indented) as the last
    /// thing in the block.
    fn insert_declaration(&self, block: &Block, declaration: &str) -> String {
        let close = &self.lexemes[block.close];
        let container_indentation = self.indentation_of(block.open);
        let first_line = (block.open + 1..block.close).find(|i| self.lexemes[*i].newlines_before > 0);
        let indentation = match first_line {
            Some(index) => self.indentation_of(index),
            None => format!("{container_indentation}{INDENT}"),
        };
        let indented: Vec<String> = declaration.lines().map(|line| format!("{indentation}{line}")).collect();
        let start = self.end_of(block.close - 1);
        if close.newlines_before > 0 {
            return self.splice(start, start, &format!("\n{}", indented.join("\n")));
        }
        // A block written on one line opens up to take a declaration.
        self.splice(start, close.offset, &format!("\n{}\n{}", indented.join("\n"), container_indentation))
    }
}

/// Every place two syntax trees differ, with what stands there in each.
fn changed<'v>(before: &'v Value, after: &'v Value, path: String, found: &mut Vec<(String, Option<&'v Value>, Option<&'v Value>)>) {
    match (before, after) {
        (Value::Object(left), Value::Object(right)) => {
            let keys: std::collections::BTreeSet<&String> = left.keys().chain(right.keys()).collect();
            for key in keys {
                let child = format!("{path}/{key}");
                match (left.get(key), right.get(key)) {
                    (Some(l), Some(r)) => changed(l, r, child, found),
                    (l, r) => found.push((child, l, r)),
                }
            }
        }
        (Value::Array(left), Value::Array(right)) if left.len() == right.len() => {
            for (index, (l, r)) in left.iter().zip(right).enumerate() {
                changed(l, r, format!("{path}/{index}"), found);
            }
        }
        _ if before != after => found.push((path, Some(before), Some(after))),
        _ => {}
    }
}

/// Whether `longer` is `shorter` with exactly one item inserted.
fn one_more(shorter: &[Value], longer: &[Value]) -> bool {
    longer.len() == shorter.len() + 1
        && (0..longer.len()).any(|skipped| {
            longer.iter().enumerate().filter(|(index, _)| *index != skipped).map(|(_, item)| item).eq(shorter.iter())
        })
}

/// The edited tree must be the original with one declaration added
/// (`added`) or removed, and nothing else touched.
fn verify_one_declaration(original: &str, edited: &str, added: bool) -> Result<(), EditError> {
    let before = syntax_tree(original, "the model")?;
    let after = syntax_tree(edited, "the edited model")?;
    let mut found = Vec::new();
    changed(&before, &after, String::new(), &mut found);
    let what = if added { "add" } else { "remove" };
    let (path, old, new) = match found.as_slice() {
        [] => return Err(EditError::Unsafe(format!("it would {what} nothing"))),
        [only] => only,
        several => {
            let places: Vec<&str> = several.iter().map(|(path, _, _)| path.trim_start_matches('/')).collect();
            return Err(EditError::Unsafe(format!(
                "it would change {} places of the model ({}), not {what} one declaration",
                places.len(),
                places.join(", ")
            )));
        }
    };
    let exact = match (old, new, added) {
        (Some(Value::Array(old)), Some(Value::Array(new)), true) => one_more(old, new),
        (Some(Value::Array(old)), Some(Value::Array(new)), false) => one_more(new, old),
        // A declaration kept under its name in a map.
        (None, Some(_), true) | (Some(_), None, false) => true,
        _ => false,
    };
    if exact {
        Ok(())
    } else {
        Err(EditError::Unsafe(format!(
            "`{}` would not simply {what} one declaration",
            path.trim_start_matches('/')
        )))
    }
}

/// Declare a new element: `keyword "name" { key: value ... }`, as the last
/// declaration of `container`. Attribute values are ArcLang source, as in
/// `set_attribute`.
pub fn add_element(
    source: &str,
    container: Container,
    keyword: &str,
    name: &str,
    attributes: &[(String, String)],
) -> Result<String, EditError> {
    check_keyword(keyword)?;
    if name.trim().is_empty() {
        return Err(EditError::Invalid("a new element needs a name".to_string()));
    }
    let mut lines = Vec::new();
    for (key, value) in attributes {
        check_key(key)?;
        parse_value(value.trim())?;
        lines.push(format!("{INDENT}{key}: {}", value.trim()));
    }
    let header = format!("{keyword} {}", string_literal(name));
    let declaration = match lines.is_empty() {
        true => format!("{header} {{}}"),
        false => format!("{header} {{\n{}\n}}", lines.join("\n")),
    };

    let document = Document::read(source)?;
    let block = match container {
        Container::Element(element) => document.find_element(element)?,
        Container::Layer { keyword, name } => document.find_layer(keyword, name)?,
    };
    let edited = document.insert_declaration(&block, &declaration);
    verify_one_declaration(source, &edited, true)?;
    Ok(edited)
}

/// Remove the declaration of `element`, with everything declared inside it.
pub fn remove_element(source: &str, element: &str) -> Result<String, EditError> {
    let document = Document::read(source)?;
    let block = document.find_element(element)?;
    let edited = document.remove_range(document.header_start(&block), block.close);
    verify_one_declaration(source, &edited, false)?;
    Ok(edited)
}

/// Give `element` a new name: the name on its declaration line is
/// replaced, and nothing else. Whatever knows the element by its old name
/// (a trace, an exchange end, an identifier derived from the name) is NOT
/// rewritten: the caller checks on the compiled model that the element kept
/// its identity and that nothing lost it.
pub fn rename_element(source: &str, element: &str, new_name: &str) -> Result<String, EditError> {
    if new_name.trim().is_empty() || new_name.contains('\n') {
        return Err(EditError::Invalid("a name is one non-empty line".to_string()));
    }
    let document = Document::read(source)?;
    let block = document.find_element(element)?;
    let start = document.header_start(&block);
    let is_string = |index: &usize| matches!(token(&document.lexemes[*index]), Some(Token::StringLiteral(_)));
    // The name is the first string of the declaration line, else its last word.
    let name = (start..block.open).find(is_string).or_else(|| block.open.checked_sub(1).filter(|last| *last > start));
    let Some(name) = name else {
        return Err(EditError::Unsafe(format!("the declaration of '{element}' carries no name")));
    };
    let edited = document.splice(document.lexemes[name].offset, document.end_of(name), &string_literal(new_name));
    verify(source, &edited, "name").map_err(|reason| match reason {
        EditError::Unsafe(detail) => EditError::Unsafe(format!(
            "{detail}: an element that writes no `id` takes its identity from its name, and renaming it would make it another element"
        )),
        other => other,
    })?;
    Ok(edited)
}

/// Declare `trace "from" kind "to" {}` at the end of the model.
pub fn add_trace(source: &str, from: &str, kind: &str, to: &str) -> Result<String, EditError> {
    check_keyword(kind)?;
    // Lexing first: a text that does not lex must not be appended to.
    Document::read(source)?;
    let declaration = format!("trace {} {kind} {} {{}}", string_literal(from), string_literal(to));
    let mut edited = source.trim_end_matches(['\n', ' ', '\t', '\r']).to_string();
    edited.push_str(&format!("\n{declaration}\n"));
    verify_one_declaration(source, &edited, true)?;
    Ok(edited)
}

/// Remove the trace of `kind` from one of `from` to one of `to`: each end
/// is given by every way the text may write it (identifier, name).
pub fn remove_trace(source: &str, from: &[&str], kind: &str, to: &[&str]) -> Result<String, EditError> {
    let document = Document::read(source)?;
    let described = format!("trace {} {kind} {}", from.first().unwrap_or(&"?"), to.first().unwrap_or(&"?"));
    let string = |lexeme: &Lexeme| match token(lexeme) {
        Some(Token::StringLiteral(text)) => Some(text.clone()),
        _ => None,
    };
    let matches: Vec<Block> = document
        .blocks()
        .into_iter()
        .filter(|block| document.is_relationship(block))
        .filter(|block| {
            let header = document.header(block);
            let ends: Vec<String> = header.iter().filter_map(string).collect();
            let verb = header.iter().skip(1).find(|lexeme| string(lexeme).is_none()).map(|lexeme| lexeme.text.as_str());
            matches!(ends.as_slice(), [source, target] if from.contains(&source.as_str()) && to.contains(&target.as_str()))
                && verb == Some(kind)
        })
        .collect();
    let block = match matches.len() {
        0 => return Err(EditError::NotFound(described)),
        1 => &matches[0],
        _ => {
            return Err(EditError::Ambiguous {
                element: described,
                lines: matches.iter().map(|block| document.lexemes[block.open].span.line).collect(),
            })
        }
    };
    let edited = document.remove_range(document.header_start(block), block.close);
    verify_one_declaration(source, &edited, false)?;
    Ok(edited)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL: &str = r#"// Braking model
logical_architecture "Braking" {
    // The controller decides.
    component "Controller" {
        id: "LC-001"
        latency: 25 ms   // worst case
    }

    // Watches the controller.
    component "Monitor" { id: "LC-002" } // redundant channel
}

physical_architecture "Hardware" {}

trace "LC-001" implements "LC-002" {}
"#;

    fn attributes(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs.iter().map(|(key, value)| (key.to_string(), value.to_string())).collect()
    }

    #[test]
    fn adds_an_element_at_the_end_of_its_layer() {
        let layer = Container::Layer { keyword: "logical_architecture", name: None };
        let edited = add_element(MODEL, layer, "component", "Logger", &attributes(&[("id", "\"LC-003\""), ("latency", "5 ms")])).unwrap();
        assert_eq!(
            edited,
            MODEL.replace(
                "{ id: \"LC-002\" } // redundant channel\n}",
                "{ id: \"LC-002\" } // redundant channel\n    component \"Logger\" {\n        id: \"LC-003\"\n        latency: 5 ms\n    }\n}"
            )
        );
    }

    #[test]
    fn adds_an_element_inside_another_and_into_an_empty_block() {
        let edited = add_element(MODEL, Container::Element("LC-001"), "function", "Decide", &attributes(&[("id", "\"LF-1\"")])).unwrap();
        assert!(edited.contains("        latency: 25 ms   // worst case\n        function \"Decide\" {\n            id: \"LF-1\"\n        }\n    }"));

        let hardware = Container::Layer { keyword: "physical_architecture", name: Some("Hardware") };
        let edited = add_element(MODEL, hardware, "node", "ECU \"A\"", &[]).unwrap();
        assert!(edited.contains("physical_architecture \"Hardware\" {\n    node \"ECU \\\"A\\\"\" {}\n}"));
    }

    #[test]
    fn removes_an_element_and_keeps_the_comments_around_it() {
        let edited = remove_element(MODEL, "LC-001").unwrap();
        assert_eq!(
            edited,
            MODEL.replace("    component \"Controller\" {\n        id: \"LC-001\"\n        latency: 25 ms   // worst case\n    }\n", "")
        );
        assert!(edited.contains("// The controller decides."));

        // A trailing comment keeps its line.
        let edited = remove_element(MODEL, "LC-002").unwrap();
        assert!(edited.contains("    // Watches the controller.\n    // redundant channel\n}"));
    }

    #[test]
    fn adds_and_removes_a_trace() {
        let edited = add_trace(MODEL, "LC-002", "refines", "LC-001").unwrap();
        assert_eq!(edited, format!("{MODEL}trace \"LC-002\" refines \"LC-001\" {{}}\n"));

        let edited = remove_trace(MODEL, &["Controller", "LC-001"], "implements", &["LC-002"]).unwrap();
        assert_eq!(edited, MODEL.replace("\n\ntrace \"LC-001\" implements \"LC-002\" {}\n", "\n"));
        assert!(matches!(remove_trace(MODEL, &["LC-001"], "refines", &["LC-002"]), Err(EditError::NotFound(_))));
    }

    #[test]
    fn renames_an_element_on_its_declaration_line_only() {
        let edited = rename_element(MODEL, "LC-001", "Brake \"main\" controller").unwrap();
        assert_eq!(edited, MODEL.replace("component \"Controller\" {", "component \"Brake \\\"main\\\" controller\" {"));
        assert!(matches!(rename_element(MODEL, "LC-001", "  "), Err(EditError::Invalid(_))));
        assert!(matches!(rename_element(MODEL, "LC-404", "X"), Err(EditError::NotFound(_))));
    }

    #[test]
    fn refuses_what_is_not_exactly_one_declaration() {
        let layer = Container::Layer { keyword: "logical_architecture", name: None };
        assert!(matches!(add_element(MODEL, layer, "component", "", &[]), Err(EditError::Invalid(_))));
        assert!(matches!(add_element(MODEL, layer, "Component!", "X", &[]), Err(EditError::Invalid(_))));
        assert!(matches!(add_element(MODEL, layer, "component", "X", &attributes(&[("latency", "5 parsecs")])), Err(EditError::Invalid(_))));
        // A keyword the layer does not accept does not parse as a declaration.
        assert!(matches!(add_element(MODEL, layer, "nonsense", "X", &[]), Err(EditError::Unsafe(_))));
        assert!(matches!(
            add_element(MODEL, Container::Layer { keyword: "system_analysis", name: None }, "function", "F", &[]),
            Err(EditError::NotFound(_))
        ));
        assert!(matches!(remove_element(MODEL, "LC-404"), Err(EditError::NotFound(_))));
    }
}
