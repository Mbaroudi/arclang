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
    /// At the end of the file: a class, an exchange item, a state machine,
    /// a scenario, a diagram.
    Document,
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
    // A port is declared with its direction: `port out "Name"`.
    let is_port = matches!(keyword, "port in" | "port out" | "port inout");
    let is_keyword = is_port || (!keyword.is_empty() && keyword.chars().all(|c| c.is_ascii_lowercase() || c == '_'));
    if is_keyword {
        Ok(())
    } else {
        Err(EditError::Invalid(format!("'{keyword}' is not a declaration keyword")))
    }
}

impl Document {
    /// A declaration without a block: `keyword Name` alone on its line, as
    /// a function allocated by name or a port without attributes (`port in
    /// "Name"`) is written. The lexeme indexes of its keyword and its name.
    /// `layer::Name` looks inside that layer only; `Owner.port` and
    /// `Owner::port` directly inside the owner's block.
    pub(super) fn find_bare_declaration(&self, element: &str) -> Option<(usize, usize)> {
        let layer = element.split_once("::").and_then(|(layer, name)| self.find_layer(layer, None).ok().map(|block| (block, name)));
        let owner = element
            .rsplit_once("::")
            .or_else(|| element.rsplit_once('.'))
            .and_then(|(owner, name)| self.find_element(owner).ok().map(|block| (block, name)));
        let (scope, element, direct) = match (layer, owner) {
            (Some((block, name)), _) => (Some(block), name, false),
            (None, Some((block, name))) => (Some(block), name, true),
            (None, None) => (None, element, false),
        };
        let lexemes = &self.lexemes;
        (0..lexemes.len().saturating_sub(1)).find_map(|index| {
            let keyword = token(&lexemes[index])?.keyword_text()?;
            let is_port = keyword == "port";
            if !is_port && !matches!(keyword, "function" | "activity" | "operational_activity" | "deploys") {
                return None;
            }
            // A port is declared with its direction: `port out "Name"`.
            let name_index = if is_port { index + 2 } else { index + 1 };
            if is_port && !matches!(token(&lexemes[index + 1])?, Token::In | Token::Out | Token::Identifier(_)) {
                return None;
            }
            let name = match token(lexemes.get(name_index)?)? {
                Token::StringLiteral(text) => text.as_str(),
                Token::Identifier(text) => text.as_str(),
                _ => return None,
            };
            if name != element {
                return None;
            }
            // A port may share the line of its owner (`component "B" { id: "LC-B" port in "q" }`).
            let own_line = lexemes[index].newlines_before > 0 || index == 0 || is_port;
            let ends_line = lexemes.get(name_index + 1).is_none_or(|next| next.newlines_before > 0 || matches!(token(next), Some(Token::RightBrace)));
            let in_scope = scope.as_ref().is_none_or(|block| index > block.open && index < block.close);
            let directly = !direct || scope.as_ref().is_some_and(|block| enclosing_open(lexemes, index) == block.open);
            (own_line && ends_line && in_scope && directly).then_some((index, name_index))
        })
    }

    pub(super) fn find_layer(&self, keyword: &str, name: Option<&str>) -> Result<Block, EditError> {
        let described = match name {
            Some(name) => format!("{keyword} \"{name}\""),
            None => keyword.to_string(),
        };
        let matches: Vec<Block> = self
            .blocks()
            .into_iter()
            .filter(|block| {
                let header = self.header(block);
                let first = header.first().map(|lexeme| lexeme.text.as_str());
                let second = header.get(1).map(|lexeme| lexeme.text.as_str());
                // `logical_architecture` is also written `architecture logical`.
                first == Some(keyword)
                    || matches!((keyword, first, second), ("logical_architecture", Some("architecture"), Some("logical")) | ("physical_architecture", Some("architecture"), Some("physical")))
            })
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

    /// `trace "A" satisfies "B"` written without a body: the first and last
    /// lexeme of each such statement linking `from` to `to` by `kind`.
    fn bodiless_traces(&self, from: &[&str], kind: &str, to: &[&str]) -> Vec<(usize, usize)> {
        let code: Vec<usize> = (0..self.lexemes.len()).filter(|i| token(&self.lexemes[*i]).is_some()).collect();
        let text = |at: usize| match token(&self.lexemes[at]) {
            Some(Token::StringLiteral(text)) => Some(text.as_str()),
            _ => None,
        };
        code.windows(4)
            .enumerate()
            .filter(|(_, w)| token(&self.lexemes[w[0]]) == Some(&Token::Trace))
            .filter(|(at, _)| code.get(at + 4).map_or(true, |next| token(&self.lexemes[*next]) != Some(&Token::LeftBrace)))
            .filter(|(_, w)| {
                matches!((text(w[1]), text(w[3])), (Some(source), Some(target)) if from.contains(&source) && to.contains(&target))
                    && text(w[2]).is_none()
                    && self.lexemes[w[2]].text == kind
            })
            .map(|(_, w)| (w[0], w[3]))
            .collect()
    }

    /// Index of the first lexeme of a block's declaration line.
    pub(super) fn header_start(&self, block: &Block) -> usize {
        block.open - self.header(block).len()
    }

    /// Remove lexemes `first..=last` with the line they stand on, when they
    /// have it to themselves.
    pub(super) fn remove_range(&self, first: usize, last: usize) -> String {
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
    pub(super) fn insert_declaration(&self, block: &Block, declaration: &str) -> String {
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
        // A block written on one line keeps a one-line declaration on that line.
        if !declaration.contains('\n') && block.close > block.open + 1 {
            let at = self.end_of(block.close - 1);
            return self.splice(at, close.offset, &format!(" {declaration} "));
        }
        // Otherwise it opens up: each of its statements on a line of its own, then the declaration.
        let inside = format!("{container_indentation}{INDENT}");
        let mut lines: Vec<String> = self.statements(block).into_iter().map(|(first, last)| {
            let text: String = self.chars[self.lexemes[first].offset..self.end_of(last)].iter().collect();
            format!("{inside}{text}")
        }).collect();
        lines.extend(declaration.lines().map(|line| format!("{inside}{line}")));
        self.splice(self.end_of(block.open), close.offset, &format!("\n{}\n{}", lines.join("\n"), container_indentation))
    }

    /// The statements written directly in a one-line block, as lexeme ranges:
    /// an attribute (`key: value`) or a declaration (`port in "q"`, `function "F" { }`).
    fn statements(&self, block: &Block) -> Vec<(usize, usize)> {
        let children = self.children(block);
        let ends_value = |index: usize| matches!(token(&self.lexemes[index]), Some(Token::StringLiteral(_) | Token::Number(_) | Token::Integer(_) | Token::Identifier(_) | Token::RightBrace | Token::RightBracket | Token::RightParen));
        let mut starts = Vec::new();
        for (at, &index) in children.iter().enumerate() {
            let lexeme = &self.lexemes[index];
            if token(lexeme).is_none() {
                continue;
            }
            let previous = at.checked_sub(1).map(|p| children[p]);
            let attribute = is_word(lexeme) && is(&self.lexemes, index + 1, &Token::Colon) && previous.is_none_or(|p| !is(&self.lexemes, p, &Token::Colon));
            let declaration = token(lexeme).is_some_and(|t| t.keyword_text().is_some()) && previous.is_none_or(ends_value);
            if previous.is_none() || attribute || declaration {
                starts.push(at);
            }
        }
        starts
            .iter()
            .enumerate()
            .map(|(n, &at)| {
                let last_at = starts.get(n + 1).map_or(children.len(), |next| *next) - 1;
                // A nested block or list ends where its bracket closes.
                let last = children[last_at];
                let last = if matches!(token(&self.lexemes[last]), Some(Token::LeftBrace | Token::LeftBracket | Token::LeftParen)) { matching_close(&self.lexemes, last).map_or(last, |end| end - 1) } else { last };
                (children[at], last)
            })
            .collect()
    }
}

/// Every place two syntax trees differ, with what stands there in each. A
/// list that changed length differs as a whole: its items cannot be
/// matched one to one.
pub(super) fn changed<'v>(before: &'v Value, after: &'v Value, path: String, found: &mut Vec<(String, Option<&'v Value>, Option<&'v Value>)>) {
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
/// How many values a syntax tree holds, wherever they stand.
fn count_leaves(value: &Value) -> usize {
    match value {
        Value::Array(items) => items.iter().map(count_leaves).sum(),
        Value::Object(map) => map.values().map(count_leaves).sum(),
        _ => 1,
    }
}

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

    add_declaration(source, container, &declaration)
}

/// Add `declaration`, written as ArcLang source (one declaration, possibly
/// several lines), as the last declaration of `container`, or at the end
/// of the file. The text must lex without a comment, and the edited model
/// must differ from the original by exactly that one declaration.
pub fn add_declaration(source: &str, container: Container, declaration: &str) -> Result<String, EditError> {
    let declaration = declaration.trim();
    if declaration.is_empty() {
        return Err(EditError::Invalid("a declaration is not empty".to_string()));
    }
    let lexemes = Lexer::new(declaration).tokenize_lossless().map_err(|e| EditError::Invalid(format!("`{declaration}` does not lex: {e}")))?;
    if lexemes.iter().any(|lexeme| token(lexeme).is_none()) {
        return Err(EditError::Invalid("a declaration added this way carries no comment".to_string()));
    }
    let document = Document::read(source)?;
    let edited = match container {
        Container::Element(element) => match document.find_element(element) {
            Ok(block) => document.insert_declaration(&block, declaration),
            // A bare declaration (`function "X"`) opens a block to hold what is added.
            Err(EditError::NotFound(_)) => {
                let (_, name_index) = document.find_bare_declaration(element).ok_or_else(|| EditError::NotFound(element.to_string()))?;
                let indentation = document.indentation_of(name_index);
                let body: String = declaration.lines().map(|line| format!("{indentation}{INDENT}{line}\n")).collect();
                let at = document.end_of(name_index);
                document.splice(at, at, &format!(" {{\n{body}{indentation}}}"))
            }
            Err(error) => return Err(error),
        },
        Container::Layer { keyword, name } => document.insert_declaration(&document.find_layer(keyword, name)?, declaration),
        Container::Document => {
            let mut text = source.trim_end().to_string();
            if !text.is_empty() {
                text.push_str("\n\n");
            }
            text.push_str(declaration);
            text.push('\n');
            text
        }
    };
    verify_one_declaration(source, &edited, true)?;
    Ok(edited)
}

/// Every reference to `name` rewritten as a reference to `id`: a string
/// equal to the name, or starting with `name.` (a port of it), and a bare
/// identifier equal to the name where a reference stands (an arrow end, a
/// list item). The declaration of the element itself, and the `name:` it
/// writes, are left alone: `rename_element` changes those. Returns the
/// text and how many references moved.
pub fn rename_references(source: &str, name: &str, id: &str, element: &str) -> Result<(String, usize), EditError> {
    let document = Document::read(source)?;
    let own = document.find_element(element).ok().map(|block| (document.header_start(&block), block.open, block))
        .or_else(|| document.find_bare_declaration(element).map(|(start, end)| (start, end + 1, Block { open: end, close: end })));
    let own_name_value = own.as_ref().and_then(|(_, _, block)| document.find_attribute(block, "name").map(|a| a.value_start));
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    let lexemes = &document.lexemes;
    let qualified = format!("{name}.");
    for (index, lexeme) in lexemes.iter().enumerate() {
        if let Some((start, open, _)) = &own {
            if index >= *start && index < *open {
                continue;
            }
        }
        if Some(index) == own_name_value {
            continue;
        }
        let replacement = match token(lexeme) {
            Some(Token::StringLiteral(text)) if text == name => Some(string_literal(id)),
            Some(Token::StringLiteral(text)) if text.starts_with(&qualified) => Some(string_literal(&format!("{id}.{}", &text[qualified.len()..]))),
            Some(Token::Identifier(text)) if text == name => {
                let key = lexemes.get(index + 1).is_some_and(|next| matches!(token(next), Some(Token::Colon)));
                let declared = index > 0 && token(&lexemes[index - 1]).is_some_and(|t| t.keyword_text().is_some()) && lexemes.get(index + 1).is_some_and(|next| matches!(token(next), Some(Token::LeftBrace)) || next.newlines_before > 0);
                if key || declared { None } else { Some(string_literal(id)) }
            }
            _ => None,
        };
        if let Some(replacement) = replacement {
            edits.push((lexeme.offset, document.end_of(index), replacement));
        }
    }
    let count = edits.len();
    let mut text = source.to_string();
    for (start, end, replacement) in edits.into_iter().rev() {
        let (before, rest) = text.split_at(char_offset(&text, start));
        let after = &rest[char_offset(rest, end - start)..];
        text = format!("{before}{replacement}{after}");
    }
    Ok((text, count))
}

/// Byte offset of the `index`th character.
fn char_offset(text: &str, index: usize) -> usize {
    text.char_indices().nth(index).map(|(offset, _)| offset).unwrap_or(text.len())
}

/// Move the declaration of `element`, with everything declared inside it,
/// into `container`, as its last declaration: a function reallocated to
/// another component, a behaviour component deployed on another node, a
/// component nested elsewhere. The text of the declaration is kept as
/// written. The caller checks on the compiled model that every element
/// kept its identity.
pub fn move_element(source: &str, element: &str, container: Container) -> Result<String, EditError> {
    let document = Document::read(source)?;
    // A declaration with no block (`function "Name"` on its line) is
    // moved as that line.
    let (start, end, block_open) = match document.find_element(element) {
        Ok(block) => (document.header_start(&block), block.close, block.open),
        Err(EditError::NotFound(_)) => {
            let (start, end) = document.find_bare_declaration(element).ok_or_else(|| EditError::NotFound(element.to_string()))?;
            (start, end, start)
        }
        Err(other) => return Err(other),
    };
    let block = Block { open: block_open, close: end };
    let text: String = document.chars[document.lexemes[start].offset..document.end_of(end)].iter().collect();
    // The lines after the first lose the indentation the declaration had
    // where it was: the container's own indentation is put in their place.
    let base = document.indentation_of(start);
    let declaration = text
        .trim()
        .lines()
        .enumerate()
        .map(|(index, line)| if index == 0 { line } else { line.strip_prefix(base.as_str()).unwrap_or(line) })
        .collect::<Vec<_>>()
        .join("\n");
    let target = match container {
        Container::Element(other) => document.find_element(other)?,
        Container::Layer { keyword, name } => document.find_layer(keyword, name)?,
        Container::Document => return Err(EditError::Invalid("an element moves into an element or a layer".to_string())),
    };
    if target.open > block.open && target.close < block.close {
        return Err(EditError::Unsafe(format!("'{element}' cannot be moved inside something it holds")));
    }
    if target.open == block.open {
        return Err(EditError::Unsafe(format!("'{element}' cannot be moved into itself")));
    }
    // Remove first, then find the container again in the shorter text.
    let removed = document.remove_range(start, end);
    let shorter = Document::read(&removed)?;
    let target = match container {
        Container::Element(other) => shorter.find_element(other)?,
        Container::Layer { keyword, name } => shorter.find_layer(keyword, name)?,
        Container::Document => unreachable!("refused above"),
    };
    let edited = shorter.insert_declaration(&target, &declaration);
    // The declarations are the same, one of them elsewhere.
    let before = syntax_tree(source, "the model")?;
    let after = syntax_tree(&edited, "the edited model")?;
    if count_leaves(&before) != count_leaves(&after) {
        return Err(EditError::Unsafe(format!("moving '{element}' would not keep the model's declarations")));
    }
    Ok(edited)
}

/// Wrap the messages of `scenario` from message `first` to message `last`
/// (their labels, in order) in a combined fragment: `fragment <kind> "label"
/// { ... }`. With `operands`, the frame is cut into operands at the messages
/// named, each with the guard given; the first operand opens at `first`
/// when no cut names it. The two ends and every cut must be messages of
/// the same block (the scenario, or one fragment or operand of it), so that
/// a fragment already there is kept whole. The messages keep their order,
/// their text and their count; the compiler reads the result.
pub fn wrap_fragment(
    source: &str,
    scenario: &str,
    kind: &str,
    label: &str,
    first: &str,
    last: &str,
    operands: &[(String, String)],
) -> Result<String, EditError> {
    if !matches!(kind, "alt" | "opt" | "loop" | "par" | "seq" | "critical") {
        return Err(EditError::Invalid(format!("'{kind}' is not a fragment kind: use alt, opt, loop, par, seq or critical")));
    }
    let tag = if label.is_empty() { String::new() } else { format!(" {}", string_literal(label)) };
    wrap_messages(source, scenario, &format!("fragment {kind}{tag}"), first, last, operands)
}

/// Assert a state, mode or function on a lifeline of a scenario while the
/// messages `first` … `last` happen: they are wrapped in
/// `state "Lifeline" "Name" { ... }` (Capella: a state fragment).
pub fn wrap_invariant(source: &str, scenario: &str, keyword: &str, lifeline: &str, name: &str, first: &str, last: &str) -> Result<String, EditError> {
    if !matches!(keyword, "state" | "mode" | "function") {
        return Err(EditError::Invalid(format!("'{keyword}' is not what a lifeline asserts: use state, mode or function")));
    }
    if name.trim().is_empty() {
        return Err(EditError::Invalid("an invariant names the state, mode or function it asserts".to_string()));
    }
    wrap_messages(source, scenario, &format!("{keyword} {} {}", string_literal(lifeline), string_literal(name.trim())), first, last, &[])
}

/// Wrap the messages `first` … `last` of a scenario in `header { ... }`,
/// cut in operands at the messages `operands` name.
fn wrap_messages(source: &str, scenario: &str, header: &str, first: &str, last: &str, operands: &[(String, String)]) -> Result<String, EditError> {
    let document = Document::read(source)?;
    let block = document.find_element(scenario)?;
    let lexemes = &document.lexemes;
    let messages = document.message_labels(&block);
    let at = |name: &str| {
        messages
            .iter()
            .find(|(_, label)| label == name)
            .map(|(i, _)| *i)
            .ok_or_else(|| EditError::NotFound(format!("message '{name}' in scenario '{scenario}'")))
    };
    let start = at(first)?;
    let end = at(last)?;
    if end < start {
        return Err(EditError::Invalid(format!("message '{last}' comes before '{first}'")));
    }
    let home = enclosing_open(lexemes, start);
    if enclosing_open(lexemes, end) != home {
        return Err(EditError::Invalid(format!(
            "messages '{first}' and '{last}' are not in the same block; a fragment or an invariant wraps messages of one block, so one already there stays whole"
        )));
    }
    let mut cuts: Vec<(usize, &str)> = Vec::new();
    for (message, guard) in operands {
        let index = at(message)?;
        if index < start || index > end {
            return Err(EditError::Invalid(format!("message '{message}' is outside '{first}' … '{last}'")));
        }
        if enclosing_open(lexemes, index) != home {
            return Err(EditError::Invalid(format!("message '{message}' is inside a fragment; an operand starts at a message of the block wrapped")));
        }
        cuts.push((index, guard.as_str()));
    }
    cuts.sort_by_key(|(index, _)| *index);
    cuts.dedup_by_key(|(index, _)| *index);
    if !cuts.is_empty() && cuts[0].0 != start {
        cuts.insert(0, (start, ""));
    }

    let indentation = document.indentation_of(start);
    // One level deeper, as this file indents: what the scenario adds to its own line.
    let unit = indentation.strip_prefix(document.indentation_of(document.header_start(&block)).as_str()).filter(|u| !u.is_empty()).unwrap_or(INDENT).to_string();
    let inner = format!("{indentation}{unit}");
    let text_start = lexemes[start].offset;
    let text_end = document.end_of(line_end(lexemes, end, block.close));
    let piece = |from: usize, to: usize| -> String { document.chars[from..to].iter().collect() };
    let reindent = |text: &str, prefix: &str| -> String {
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| format!("{prefix}{}\n", line.strip_prefix(indentation.as_str()).unwrap_or(line.trim_start())))
            .collect()
    };
    let mut out = format!("{header} {{\n");
    if cuts.is_empty() {
        out.push_str(&reindent(&piece(text_start, text_end), &inner));
    } else {
        for (n, (index, guard)) in cuts.iter().enumerate() {
            let from = lexemes[*index].offset;
            let to = cuts.get(n + 1).map_or(text_end, |(next, _)| lexemes[*next].offset);
            let guard_tag = if guard.is_empty() { String::new() } else { format!(" {}", string_literal(guard)) };
            out.push_str(&format!("{inner}operand{guard_tag} {{\n"));
            out.push_str(&reindent(&piece(from, to), &format!("{inner}{unit}")));
            out.push_str(&format!("{inner}}}\n"));
        }
    }
    out.push_str(&format!("{indentation}}}"));
    let edited = document.splice(text_start, text_end, &out);

    // The same messages, in the same order, under the same scenario.
    let after = Document::read(&edited)?;
    let kept: Vec<String> = after.message_labels(&after.find_element(scenario)?).into_iter().map(|(_, label)| label).collect();
    let before: Vec<String> = messages.into_iter().map(|(_, label)| label).collect();
    if kept != before {
        return Err(EditError::Unsafe("wrapping the messages would change them".to_string()));
    }
    syntax_tree(&edited, "the edited model")?;
    Ok(edited)
}

/// The index of the `{` that directly encloses lexeme `index`.
fn enclosing_open(lexemes: &[Lexeme], index: usize) -> usize {
    let mut depth = 0usize;
    for j in (0..index).rev() {
        match token(&lexemes[j]) {
            Some(Token::RightBrace) => depth += 1,
            Some(Token::LeftBrace) if depth == 0 => return j,
            Some(Token::LeftBrace) => depth -= 1,
            _ => {}
        }
    }
    0
}

/// The last lexeme of the line that starts at `index`, taking the block
/// opened on that line with it; never past `limit`.
fn line_end(lexemes: &[Lexeme], index: usize, limit: usize) -> usize {
    let mut j = index + 1;
    while j < limit && lexemes[j].newlines_before == 0 {
        if matches!(token(&lexemes[j]), Some(Token::LeftBrace)) {
            if let Some(close) = matching_close(lexemes, j) {
                return close - 1;
            }
        }
        j += 1;
    }
    j - 1
}

impl Document {
    /// The messages written directly or in fragments of `block`: the index
    /// of each `message` keyword with the label that follows the arrow.
    fn message_labels(&self, block: &Block) -> Vec<(usize, String)> {
        let lexemes = &self.lexemes;
        (block.open + 1..block.close)
            .filter(|&i| matches!(token(&lexemes[i]), Some(Token::Message)))
            .filter_map(|i| {
                let label = (i + 4..block.close)
                    .take_while(|&j| lexemes[j].newlines_before == 0)
                    .find_map(|j| match token(&lexemes[j]) {
                        Some(Token::StringLiteral(text)) => Some(text.clone()),
                        _ => None,
                    });
                label.map(|label| (i, label))
            })
            .collect()
    }
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
    // A declaration with no block (`function "Name"` on its line): its name
    // is the token after the keyword.
    let (start, open) = match document.find_element(element) {
        Ok(block) => (document.header_start(&block), block.open),
        Err(EditError::NotFound(_)) => {
            let (keyword, name) = document.find_bare_declaration(element).ok_or_else(|| EditError::NotFound(element.to_string()))?;
            (keyword, name + 1)
        }
        Err(other) => return Err(other),
    };
    // A `req "ID" "Title"` line names the requirement by its title.
    let header_title = document.find_element(element).ok().and_then(|block| document.header_title(&block));
    let block = Block { open, close: open };
    let is_string = |index: &usize| matches!(token(&document.lexemes[*index]), Some(Token::StringLiteral(_)));
    // The name is the first string of the declaration line, else its last word.
    let name = header_title.or_else(|| (start..block.open).find(is_string)).or_else(|| block.open.checked_sub(1).filter(|last| *last > start));
    let Some(name) = name else {
        return Err(EditError::Unsafe(format!("the declaration of '{element}' carries no name")));
    };
    let edited = document.splice(document.lexemes[name].offset, document.end_of(name), &string_literal(new_name));
    let key = if header_title.is_some() { "title" } else { "name" };
    verify(source, &edited, key).map_err(|reason| match reason {
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
    let matches: Vec<(usize, usize)> = document
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
        .map(|block| (document.header_start(&block), block.close))
        .chain(document.bodiless_traces(from, kind, to))
        .collect();
    let (first, last) = match matches.len() {
        0 => return Err(EditError::NotFound(described)),
        1 => matches[0],
        _ => {
            return Err(EditError::Ambiguous {
                element: described,
                lines: matches.iter().map(|(first, _)| document.lexemes[*first].span.line).collect(),
            })
        }
    };
    let edited = document.remove_range(first, last);
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
    fn removes_a_trace_written_without_a_body() {
        let model = MODEL.replace("trace \"LC-001\" implements \"LC-002\" {}", "trace \"LC-001\" implements \"LC-002\"\ntrace \"LC-002\" refines \"LC-001\" // kept");
        let edited = remove_trace(&model, &["LC-001"], "implements", &["LC-002"]).unwrap();
        assert!(!edited.contains("implements"), "{edited}");
        assert!(edited.contains("trace \"LC-002\" refines \"LC-001\" // kept\n"), "{edited}");
        assert!(matches!(remove_trace(&model, &["LC-001"], "satisfies", &["LC-002"]), Err(EditError::NotFound(_))));
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

    #[test]
    fn a_bare_function_opens_a_block_for_what_is_added() {
        let model = "model T { name: \"t\" }\nlogical_architecture LA {\n    component Fusion {\n        id: \"LC-F\"\n        function \"Fuse data\"\n    }\n}\n";
        let edited = add_element(model, Container::Element("Fuse data"), "port out", "objects", &[]).unwrap();
        assert!(edited.contains("        function \"Fuse data\" {\n            port out \"objects\" {}\n        }\n"), "{edited}");
    }
}
