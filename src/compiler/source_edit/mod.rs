//! Edits to model source that keep everything else as written.
//!
//! An edit replaces, inserts or removes ONE attribute of ONE element, by
//! splicing the source text between two token positions. Comments, layout
//! and every other declaration are untouched, because nothing is rebuilt
//! from the model. This is the write path that `arclang set` uses and that
//! API writes will use.
//!
//! The element and the extent of a value are found on the token stream, with
//! rules that mirror the parser's. Since those rules are a second reading of
//! the grammar, no edit is trusted on its own: both texts are parsed with the
//! real parser and the two syntax trees must differ by the requested
//! attribute and nothing else, otherwise the edit is refused.

use super::ast::AttributeValue;
use super::elements::ElementGraph;
use super::lexer::{Lexeme, LexemeKind, Lexer, Token};
use super::metamodel::{AttrType, Metamodel};
use super::parser::Parser;
use serde_json::Value;
use std::fmt;
use std::path::Path;

mod structure;
mod relink;
mod duplicate;

pub use relink::{relationship_spots, replace_lexemes, spelled_like, RelationshipEnds, RelationshipSpots, Spot};
pub use duplicate::{duplicate_element, duplicate_relationship, CopyTarget};
pub use structure::{add_declaration, add_element, add_trace, move_element, remove_element, remove_trace, rename_element, rename_references, string_literal, wrap_fragment, Container};

/// One declaration of the source, where it stands: what an outline or a
/// "go to definition" needs, read from the text itself.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Declaration {
    /// The first word of the declaration line: `component`, `req`, `trace`...
    pub keyword: String,
    /// The name on the declaration line (its first string, else its last word).
    pub name: Option<String>,
    /// The `id:` written in the block, when there is one.
    pub id: Option<String>,
    /// The declaration line as written, up to the brace: `trace "A" satisfies "B"`.
    pub header: String,
    /// 1-based line and column of the declaration's first word.
    pub line: usize,
    pub column: usize,
    /// 1-based line of the closing brace.
    pub end_line: usize,
    pub children: Vec<Declaration>,
}

/// Every declaration of the source, nested as the braces nest. Fails only
/// when the text does not lex.
pub fn outline(source: &str) -> Result<Vec<Declaration>, EditError> {
    let document = Document::read(source)?;
    let blocks = document.blocks();
    // Blocks come in order of their `{`; a block holds the ones opened
    // inside it. A stack of open blocks places each one under its parent.
    let mut roots: Vec<Declaration> = Vec::new();
    let mut stack: Vec<(Block, Declaration)> = Vec::new();
    let flush = |stack: &mut Vec<(Block, Declaration)>, roots: &mut Vec<Declaration>, open: usize| {
        while stack.last().is_some_and(|(block, _)| block.close < open) {
            let (_, done) = stack.pop().expect("checked");
            match stack.last_mut() {
                Some((_, parent)) => parent.children.push(done),
                None => roots.push(done),
            }
        }
    };
    for block in blocks {
        flush(&mut stack, &mut roots, block.open);
        let header = document.header(&block);
        let first = header.first();
        let declaration = Declaration {
            keyword: first.map(|lexeme| lexeme.text.clone()).unwrap_or_default(),
            name: document.header_name(&block),
            id: document.written_id(&block),
            header: header.iter().map(|lexeme| lexeme.text.as_str()).collect::<Vec<_>>().join(" "),
            line: first.map(|lexeme| lexeme.span.line).unwrap_or(document.lexemes[block.open].span.line),
            column: first.map(|lexeme| lexeme.span.column).unwrap_or(document.lexemes[block.open].span.column),
            end_line: document.lexemes[block.close].span.line,
            children: Vec::new(),
        };
        stack.push((block, declaration));
    }
    flush(&mut stack, &mut roots, usize::MAX);
    Ok(roots)
}

const INDENT: &str = "    ";

#[derive(Debug, Clone, PartialEq)]
pub enum EditError {
    /// The source, or the value to write, is not valid ArcLang.
    Invalid(String),
    /// No element with this identifier or name.
    NotFound(String),
    /// Several elements answer to this identifier or name.
    Ambiguous { element: String, lines: Vec<usize> },
    /// The attribute to remove is not written on the element.
    NoSuchAttribute { element: String, key: String },
    /// The edit would have changed something other than what was asked.
    Unsafe(String),
    /// The attribute was written but the compiled element does not show it.
    NoEffect { element: String, key: String, shown: Option<String> },
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EditError::Invalid(reason) => write!(f, "{reason}"),
            EditError::NotFound(element) => write!(
                f,
                "no element with id or name '{element}' is declared in this file"
            ),
            EditError::Ambiguous { element, lines } => {
                let lines: Vec<String> = lines.iter().map(usize::to_string).collect();
                write!(
                    f,
                    "'{element}' designates {} elements (lines {}): use an id that is unique",
                    lines.len(),
                    lines.join(", ")
                )
            }
            EditError::NoSuchAttribute { element, key } => write!(
                f,
                "element '{element}' has no attribute '{key}' written on it"
            ),
            EditError::Unsafe(reason) => write!(f, "edit refused: {reason}"),
            EditError::NoEffect { element, key, shown } => {
                let shown = shown.as_deref().unwrap_or("nothing");
                write!(
                    f,
                    "edit refused: '{key}' of '{element}' would still read {shown} once compiled; \
                     its value comes from elsewhere (the declaration line, for instance)"
                )
            }
        }
    }
}

impl std::error::Error for EditError {}

/// Set attribute `key` of `element` to `value`, where `value` is ArcLang
/// source for one value (`"text"`, `25 ms`, `[A, B]`, `{ cpu: 4 }`). The
/// attribute is replaced where it stands, or added at the end of the block.
///
/// Two values live on the declaration line rather than in the block, and
/// are rewritten there: the title of `req "ID" "Title"` (`title`, or `name`,
/// which reads the same) and the direction of `port in "Name"` (`direction`).
/// A port declared without a block (`port in "Name"`) is given one.
/// The keyword of the layer block an element of `kind` is declared in
/// (`operational_analysis`, `system_analysis`, `logical_architecture`,
/// `physical_architecture`, `epbs`), from the metamodel; None for the kinds
/// that live outside the layers. `keyword::Name` designates an element
/// whose name another layer also uses.
pub fn layer_keyword_of_kind(kind: &str) -> Option<&'static str> {
    let metamodel = crate::compiler::metamodel::Metamodel::current();
    let layer = metamodel.kinds.iter().find(|k| k.name == kind).map(|k| format!("{:?}", k.layer))?;
    match layer.as_str() {
        "Operational" => Some("operational_analysis"),
        "System" => Some("system_analysis"),
        "Logical" => Some("logical_architecture"),
        "Physical" => Some("physical_architecture"),
        "Epbs" => Some("epbs"),
        _ => None,
    }
}

pub fn set_attribute(source: &str, element: &str, key: &str, value: &str) -> Result<String, EditError> {
    let value = value.trim();
    check_key(key)?;
    // A port's direction is a word of its declaration line, not a value.
    let parsed = match (key, value) {
        ("direction", "in" | "out" | "inout") => AttributeValue::String(value.to_string()),
        _ => parse_value(value)?,
    };
    let document = Document::read(source)?;
    let block = match document.find_element(element) {
        Ok(block) => block,
        Err(EditError::NotFound(_)) => return document.set_on_bare_declaration(source, element, key, value),
        Err(error) => return Err(error),
    };

    if let Some(title) = document.header_title(&block).filter(|_| key == "title" || key == "name") {
        if document.find_attribute(&block, "title").is_some() {
            return Err(EditError::Unsafe(format!(
                "'{element}' writes its title both on its declaration line and as `title:` in its block; remove one of them first"
            )));
        }
        let AttributeValue::String(_) = parsed else {
            return Err(EditError::Invalid(format!("the title of '{element}' is text: write it as \"text\"")));
        };
        let edited = document.splice(document.lexemes[title].offset, document.end_of(title), value);
        verify(source, &edited, "title")?;
        return Ok(edited);
    }
    if key == "direction" {
        if let Some(direction) = document.header_direction(&block) {
            let edited = document.set_direction(direction, value)?;
            verify(source, &edited, key)?;
            return Ok(edited);
        }
    }

    let edited = match document.find_attribute(&block, key) {
        Some(attribute) => {
            let start = document.lexemes[attribute.value_start].offset;
            let end = document.end_of(attribute.value_end - 1);
            document.splice(start, end, value)
        }
        None => document.insert_attribute(&block, key, value),
    };
    verify(source, &edited, key)?;
    Ok(edited)
}

/// Remove attribute `key` from `element`. A comment on the same line stays.
pub fn remove_attribute(source: &str, element: &str, key: &str) -> Result<String, EditError> {
    check_key(key)?;
    let document = Document::read(source)?;
    let block = document.find_element(element)?;
    if let Some(title) = document.header_title(&block).filter(|_| key == "title" || key == "name") {
        return Err(EditError::Unsafe(format!(
            "the title of '{element}' is written on its declaration line (`req {} {}`) and a requirement keeps a title: set it to another text instead",
            document.header(&block).get(1).map_or("", |lexeme| lexeme.text.as_str()),
            document.lexemes[title].text
        )));
    }
    if key == "direction" && document.header_direction(&block).is_some() {
        return Err(EditError::Unsafe(format!(
            "the direction of port '{element}' is written on its declaration line and a port keeps one: set it to in, out or inout instead"
        )));
    }
    let attribute = document.find_attribute(&block, key).ok_or_else(|| EditError::NoSuchAttribute {
        element: element.to_string(),
        key: key.to_string(),
    })?;

    let lexemes = &document.lexemes;
    let key_lexeme = &lexemes[attribute.key];
    let following = &lexemes[attribute.value_end];
    let own_line = key_lexeme.newlines_before > 0 && following.newlines_before > 0;
    let (start, end) = if own_line {
        // Take the line with it: from the end of what precedes the key.
        (document.end_of(attribute.key - 1), document.end_of(attribute.value_end - 1))
    } else {
        // Shares its line: remove up to whatever comes next, comma included.
        let comma = matches!(token(following), Some(Token::Comma));
        let next = if comma { attribute.value_end + 1 } else { attribute.value_end };
        (key_lexeme.offset, lexemes[next].offset)
    };
    let edited = document.splice(start, end, "");
    verify(source, &edited, key)?;
    Ok(edited)
}

fn check_key(key: &str) -> Result<(), EditError> {
    let is_word = !key.is_empty()
        && key.chars().all(|c| c.is_alphanumeric() || c == '_')
        && !key.starts_with(|c: char| c.is_ascii_digit());
    if is_word {
        Ok(())
    } else {
        Err(EditError::Invalid(format!("'{key}' is not an attribute name")))
    }
}

/// Parse `value` as the single attribute value it must be. A comment is
/// refused: it would be dropped or, worse, swallow the rest of the line.
pub fn parse_value(value: &str) -> Result<AttributeValue, EditError> {
    let invalid = |reason: &str| {
        EditError::Invalid(format!(
            "`{value}` is not a value ({reason}); write text as \"text\", a quantity as 25 ms, a list as [A, B]"
        ))
    };
    let lexemes = Lexer::new(value).tokenize_lossless().map_err(|e| invalid(&e))?;
    if lexemes.is_empty() {
        return Err(invalid("it is empty"));
    }
    if lexemes.iter().any(|lexeme| token(lexeme).is_none()) {
        return Err(invalid("it contains a comment"));
    }
    let tokens = Lexer::new(value).tokenize().map_err(|e| invalid(&e))?;
    Parser::new(tokens).parse_value().map_err(|e| invalid(&e))
}

/// Confirm on the COMPILED model that `set_attribute` took effect: the
/// element shows `value` for `key`. A value written in a block can be
/// shadowed by the declaration line (`req "ID" "Title" { ... }`); such an
/// edit would be a write that changes nothing, and is refused.
pub fn confirm_effect(graph: &ElementGraph, element: &str, key: &str, value: &str) -> Result<(), EditError> {
    let expected = match (key, value.trim()) {
        ("direction", word @ ("in" | "out" | "inout")) => AttributeValue::String(word.to_string()),
        (_, value) => parse_value(value)?,
    };
    // A port goes by `Owner.port`; the diagrams write it `Owner::port`.
    let dotted = element.replace("::", ".");
    let record = graph
        .elements
        .iter()
        .find(|record| record.id == element || record.id == dotted)
        .or_else(|| graph.elements.iter().find(|record| record.name == element))
        .ok_or_else(|| EditError::NotFound(element.to_string()))?;
    let shown = record.attributes.get(key);
    if shown == Some(&expected) {
        return Ok(());
    }
    // What is not an attribute of the compiled element but reads from it
    // all the same: its display name, and the data of its kind (a port's
    // direction).
    let text = match &expected {
        AttributeValue::String(text) => Some(text.as_str()),
        _ => None,
    };
    let as_name = key == "name" && shown.is_none() && text == Some(record.name.as_str());
    let as_extra = shown.is_none() && record.extra.get(key).and_then(Value::as_str).is_some_and(|extra| Some(extra) == text);
    if as_name || as_extra {
        return Ok(());
    }
    Err(EditError::NoEffect {
        element: element.to_string(),
        key: key.to_string(),
        shown: shown.map(AttributeValue::display),
    })
}

/// The members of the enumeration the metamodel gives attribute `key` of
/// elements of `kind`, when it is one.
pub fn enumeration_values(kind: &str, key: &str) -> Option<&'static [&'static str]> {
    enumeration_of(&Metamodel::current(), kind, key).map(|(_, members)| members)
}

/// The enumeration (name, members) behind `key` on `kind`, if any.
fn enumeration_of(metamodel: &Metamodel, kind: &str, key: &str) -> Option<(&'static str, &'static [&'static str])> {
    let spec = metamodel.kind(kind)?.attributes.iter().find(|attribute| attribute.key == key)?;
    match spec.ty {
        AttrType::Enum(name) => Some((name, metamodel.enumeration(name)?.values)),
        _ => None,
    }
}

/// Refuse a text that is not a member of the enumeration `key` draws from
/// on elements of `kind` (`method` of a test case, `asil` of a hazard...):
/// the compiler would keep it as a finding, an edit must not write it.
pub fn check_enumeration(kind: &str, key: &str, value: &str) -> Result<(), EditError> {
    let metamodel = Metamodel::current();
    let Some((name, members)) = enumeration_of(&metamodel, kind, key) else { return Ok(()) };
    let raw = match parse_value(value.trim())? {
        AttributeValue::String(text) => text,
        // Anything else is not a member either; the compiler says so in its own words.
        _ => return Ok(()),
    };
    if metamodel.normalize_enum(name, &raw).is_some() {
        return Ok(());
    }
    Err(EditError::Invalid(format!(
        "\"{raw}\" is not a {key} of a {kind}: use one of {}",
        members.iter().map(|member| format!("\"{member}\"")).collect::<Vec<_>>().join(", ")
    )))
}

fn token(lexeme: &Lexeme) -> Option<&Token> {
    match &lexeme.kind {
        LexemeKind::Token(token) => Some(token),
        LexemeKind::LineComment | LexemeKind::BlockComment => None,
    }
}

fn is_word(lexeme: &Lexeme) -> bool {
    matches!(token(lexeme), Some(t) if matches!(t, Token::Identifier(_)) || t.keyword_text().is_some())
}

fn is(lexemes: &[Lexeme], index: usize, expected: &Token) -> bool {
    lexemes.get(index).and_then(token) == Some(expected)
}

/// Index just past the bracket matching the opener at `open`.
fn matching_close(lexemes: &[Lexeme], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, lexeme) in lexemes.iter().enumerate().skip(open) {
        match token(lexeme) {
            Some(Token::LeftBrace | Token::LeftBracket | Token::LeftParen) => depth += 1,
            Some(Token::RightBrace | Token::RightBracket | Token::RightParen) => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// Index just past the attribute value starting at `start`. Mirrors
/// `Parser::parse_attribute_value`: a string, a number with an optional
/// unit, a bracketed list, a braced map, or a (dotted) bare name.
fn value_end(lexemes: &[Lexeme], start: usize) -> Option<usize> {
    match token(lexemes.get(start)?)? {
        Token::StringLiteral(_) => Some(start + 1),
        Token::Number(_) | Token::Integer(_) if is(lexemes, start + 1, &Token::Percent) => Some(start + 2),
        Token::Number(_) | Token::Integer(_) => {
            let has_unit = matches!(lexemes.get(start + 1).and_then(token), Some(Token::Identifier(_)))
                && !is(lexemes, start + 2, &Token::Colon);
            if !has_unit {
                return Some(start + 1);
            }
            let compound = is(lexemes, start + 2, &Token::Slash)
                && matches!(lexemes.get(start + 3).and_then(token), Some(Token::Identifier(_)));
            Some(if compound { start + 4 } else { start + 2 })
        }
        Token::LeftBracket | Token::LeftBrace => matching_close(lexemes, start),
        _ if is_word(&lexemes[start]) => {
            let mut end = start + 1;
            while is(lexemes, end, &Token::Dot) && lexemes.get(end + 1).is_some_and(is_word) {
                end += 2;
            }
            Some(end)
        }
        _ => None,
    }
}

/// An element declaration: `kind "Name" ... { ... }`.
struct Block {
    /// Index of `{` and of its matching `}`.
    open: usize,
    close: usize,
}

struct Attribute {
    key: usize,
    value_start: usize,
    /// Index just past the value.
    value_end: usize,
}

struct Document {
    chars: Vec<char>,
    lexemes: Vec<Lexeme>,
}

impl Document {
    fn read(source: &str) -> Result<Document, EditError> {
        let lexemes = Lexer::new(source).tokenize_lossless().map_err(EditError::Invalid)?;
        Ok(Document { chars: source.chars().collect(), lexemes })
    }

    fn end_of(&self, index: usize) -> usize {
        let lexeme = &self.lexemes[index];
        lexeme.offset + lexeme.text.chars().count()
    }

    fn splice(&self, start: usize, end: usize, replacement: &str) -> String {
        let mut edited: String = self.chars[..start].iter().collect();
        edited.push_str(replacement);
        edited.extend(&self.chars[end..]);
        edited
    }

    /// Every braced block that declares something: a `{` that is not the
    /// value of an attribute (`key: {`) nor an item of a list.
    fn blocks(&self) -> Vec<Block> {
        let mut blocks = Vec::new();
        for (open, lexeme) in self.lexemes.iter().enumerate() {
            if token(lexeme) != Some(&Token::LeftBrace) || open == 0 {
                continue;
            }
            let is_value = matches!(
                token(&self.lexemes[open - 1]),
                Some(Token::Colon | Token::Comma | Token::LeftBracket)
            );
            if is_value {
                continue;
            }
            if let Some(end) = matching_close(&self.lexemes, open) {
                blocks.push(Block { open, close: end - 1 });
            }
        }
        blocks
    }

    /// Lexemes directly inside a block: not in a nested block, list or map.
    fn children(&self, block: &Block) -> Vec<usize> {
        let mut children = Vec::new();
        let mut index = block.open + 1;
        while index < block.close {
            children.push(index);
            let opens = matches!(
                token(&self.lexemes[index]),
                Some(Token::LeftBrace | Token::LeftBracket | Token::LeftParen)
            );
            index = match matching_close(&self.lexemes, index) {
                Some(end) if opens => end,
                _ => index + 1,
            };
        }
        children
    }

    fn find_attribute(&self, block: &Block, key: &str) -> Option<Attribute> {
        self.children(block).into_iter().find_map(|index| {
            let lexeme = &self.lexemes[index];
            let is_key = is_word(lexeme) && lexeme.text == key && is(&self.lexemes, index + 1, &Token::Colon);
            // A key only ever starts an attribute: after `{`, a value or a comment.
            let starts_attribute = !is(&self.lexemes, index - 1, &Token::Colon);
            if !is_key || !starts_attribute {
                return None;
            }
            let value_end = value_end(&self.lexemes, index + 2).filter(|end| *end <= block.close)?;
            Some(Attribute { key: index, value_start: index + 2, value_end })
        })
    }

    /// The lexemes of a block's declaration line, before its `{`.
    fn header(&self, block: &Block) -> &[Lexeme] {
        let mut start = block.open;
        while start > 0 {
            let lexeme = &self.lexemes[start - 1];
            let boundary = matches!(token(lexeme), None | Some(Token::LeftBrace | Token::RightBrace));
            if boundary {
                break;
            }
            start -= 1;
            if lexeme.newlines_before > 0 {
                break;
            }
        }
        &self.lexemes[start..block.open]
    }

    /// `trace "A" satisfies "B" { }` links two elements; it is not one, and
    /// the names on its line are those of its ends.
    fn is_relationship(&self, block: &Block) -> bool {
        let header = self.header(block);
        if matches!(header.first().and_then(token), Some(Token::Trace | Token::Transition | Token::Message)) {
            return true;
        }
        // `connection "A" -> "B"`, `deployment "C" -> "N"`: the strings are the
        // ends, not a name. A link names itself before its arrow.
        let names_itself = header.first().is_some_and(|first| first.text == "physical_link" || first.text == "link");
        !names_itself && header.iter().any(|lexeme| matches!(token(lexeme), Some(Token::Arrow)))
    }

    /// The name in a block's header: its first string, else its last word.
    fn header_name(&self, block: &Block) -> Option<String> {
        let header = self.header(block);
        let string = header.iter().find_map(|lexeme| match token(lexeme) {
            Some(Token::StringLiteral(text)) => Some(text.clone()),
            _ => None,
        });
        string.or_else(|| header.last().filter(|l| is_word(l)).map(|l| l.text.clone()))
    }

    fn written_id(&self, block: &Block) -> Option<String> {
        let attribute = self.find_attribute(block, "id")?;
        match token(&self.lexemes[attribute.value_start])? {
            Token::StringLiteral(text) => Some(text.clone()),
            Token::Identifier(text) => Some(text.clone()),
            _ => None,
        }
    }

    /// The `name:` written in the block, when there is one: the display
    /// name of an element declared by an identifier.
    fn written_name(&self, block: &Block) -> Option<String> {
        let attribute = self.find_attribute(block, "name")?;
        match token(&self.lexemes[attribute.value_start])? {
            Token::StringLiteral(text) => Some(text.clone()),
            _ => None,
        }
    }

    /// The title on a `req "ID" "Title"` line: the index of its lexeme.
    fn header_title(&self, block: &Block) -> Option<usize> {
        let header = self.header(block);
        let is_req = matches!(header.first().and_then(token), Some(Token::Req));
        let titled = is_req && header.len() == 3 && matches!(token(&header[2]), Some(Token::StringLiteral(_)));
        titled.then(|| block.open - 1)
    }

    /// The direction on a `port in "Name"` line: the index of its lexeme.
    fn header_direction(&self, block: &Block) -> Option<usize> {
        let header = self.header(block);
        let is_port = matches!(header.first().and_then(token), Some(Token::Port));
        (is_port && header.len() == 3).then(|| block.open - 2)
    }

    /// `value` written in place of the direction word at `direction`.
    fn set_direction(&self, direction: usize, value: &str) -> Result<String, EditError> {
        if !matches!(value, "in" | "out" | "inout") {
            return Err(EditError::Invalid(format!("'{value}' is not a port direction: use in, out or inout")));
        }
        Ok(self.splice(self.lexemes[direction].offset, self.end_of(direction), value))
    }

    /// `set` on a declaration that has no block (`port in "Name"` on its
    /// line): the direction is on the line; any other attribute opens a
    /// block to hold it.
    fn set_on_bare_declaration(&self, source: &str, element: &str, key: &str, value: &str) -> Result<String, EditError> {
        let (keyword, name) = self.find_bare_declaration(element).ok_or_else(|| EditError::NotFound(element.to_string()))?;
        let is_port = matches!(token(&self.lexemes[keyword]), Some(Token::Port));
        let edited = if key == "direction" && is_port {
            self.set_direction(keyword + 1, value)?
        } else {
            let at = self.end_of(name);
            self.splice(at, at, &format!(" {{ {key}: {value} }}"))
        };
        verify(source, &edited, key)?;
        Ok(edited)
    }

    /// The blocks declared directly inside `owner`: not those nested deeper.
    fn direct_children(&self, owner: &Block) -> Vec<Block> {
        let inside: Vec<Block> = self
            .blocks()
            .into_iter()
            .filter(|block| block.open > owner.open && block.close < owner.close)
            .collect();
        let mut direct = Vec::new();
        let mut covered_until = 0;
        for block in inside {
            if block.open > covered_until {
                covered_until = block.close;
                direct.push(block);
            }
        }
        direct
    }

    /// `Owner.child` (a port, as the compiler names it) or `Owner::child`
    /// (as the diagrams write it): the block of `child` directly inside
    /// the owner's, by its written id, its header name or its `name:`.
    fn find_nested(&self, element: &str) -> Result<Block, EditError> {
        let (owner, child) = element
            .rsplit_once("::")
            .or_else(|| element.rsplit_once('.'))
            .ok_or_else(|| EditError::NotFound(element.to_string()))?;
        let owner = self.find_element(owner).map_err(|_| EditError::NotFound(element.to_string()))?;
        let matches: Vec<Block> = self
            .direct_children(&owner)
            .into_iter()
            .filter(|block| !self.is_relationship(block))
            .filter(|block| {
                self.written_id(block).as_deref() == Some(child)
                    || self.header_name(block).as_deref() == Some(child)
                    || self.written_name(block).as_deref() == Some(child)
            })
            .collect();
        match matches.len() {
            0 => Err(EditError::NotFound(element.to_string())),
            1 => Ok(matches.into_iter().next().expect("one match")),
            _ => Err(EditError::Ambiguous {
                element: element.to_string(),
                lines: matches.iter().map(|block| self.lexemes[block.open].span.line).collect(),
            }),
        }
    }

    /// The one block declaring `element`: by its written `id`, and when no
    /// block carries that id, by name among the blocks that write no id —
    /// the name on the declaration line, or the `name:` written inside.
    /// `layer::Name` looks inside that layer only; `Owner.port` and
    /// `Owner::port` inside the owner's block (see [`Self::find_nested`]).
    fn find_element(&self, element: &str) -> Result<Block, EditError> {
        match self.find_declared(element) {
            Err(EditError::NotFound(_)) => self.find_nested(element),
            other => other,
        }
    }

    fn find_declared(&self, element: &str) -> Result<Block, EditError> {
        // `layer::Name` designates the element inside that layer block only:
        // the same name at two levels is two elements.
        let (scope, element) = match element.split_once("::") {
            Some((layer, name)) => match self.find_layer(layer, None) {
                Ok(layer) => (Some(layer), name),
                // Not a layer: an owner, which `find_nested` resolves.
                Err(_) => return Err(EditError::NotFound(element.to_string())),
            },
            None => (None, element),
        };
        let blocks: Vec<Block> = self
            .blocks()
            .into_iter()
            .filter(|block| scope.as_ref().is_none_or(|layer| block.open > layer.open && block.close < layer.close))
            .collect();
        let ids: Vec<Option<String>> = blocks.iter().map(|block| self.written_id(block)).collect();
        let by_id: Vec<usize> = (0..blocks.len()).filter(|i| ids[*i].as_deref() == Some(element)).collect();
        let matches = if by_id.is_empty() {
            (0..blocks.len())
                .filter(|i| ids[*i].is_none() && !self.is_relationship(&blocks[*i]))
                .filter(|i| {
                    self.header_name(&blocks[*i]).as_deref() == Some(element)
                        || self.written_name(&blocks[*i]).as_deref() == Some(element)
                })
                .collect()
        } else {
            by_id
        };

        match matches.as_slice() {
            [] => Err(EditError::NotFound(element.to_string())),
            [only] => Ok(blocks.into_iter().nth(*only).expect("index comes from this list")),
            several => Err(EditError::Ambiguous {
                element: element.to_string(),
                lines: several.iter().map(|i| self.lexemes[blocks[*i].open].span.line).collect(),
            }),
        }
    }

    /// Whitespace that starts the line `index` is on.
    fn indentation_of(&self, index: usize) -> String {
        let offset = self.lexemes[index].offset;
        let line_start = self.chars[..offset].iter().rposition(|c| *c == '\n').map_or(0, |p| p + 1);
        self.chars[line_start..offset].iter().take_while(|c| c.is_whitespace()).collect()
    }

    /// Add `key: value` as the last thing in the block, laid out like the
    /// block already is.
    fn insert_attribute(&self, block: &Block, key: &str, value: &str) -> String {
        let attribute = format!("{key}: {value}");
        let close = &self.lexemes[block.close];
        let empty = block.close == block.open + 1;
        let multiline = close.newlines_before > 0;

        if empty && !multiline {
            let start = self.end_of(block.open);
            return self.splice(start, close.offset, &format!(" {attribute} "));
        }
        if !multiline {
            let start = self.end_of(block.close - 1);
            return self.splice(start, close.offset, &format!(" {attribute} "));
        }
        // Indent like the first line of the block, or one level in.
        let first_line = (block.open + 1..block.close).find(|i| self.lexemes[*i].newlines_before > 0);
        let indentation = match first_line {
            Some(index) => self.indentation_of(index),
            None => format!("{}{INDENT}", self.indentation_of(block.close)),
        };
        let start = self.end_of(block.close - 1);
        self.splice(start, start, &format!("\n{indentation}{attribute}"))
    }
}

/// Compile `text` as if it were the file at `path`: written beside it, so
/// that relative imports resolve, and removed whatever the outcome. This is
/// how an edit is tried before the real file is touched.
pub fn compile_beside(path: &Path, text: &str) -> Result<crate::CompilationResult, String> {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let directory = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("model.arc");
    let sibling = directory.join(format!(
        ".{}.{}-{}.edit.arc",
        name,
        std::process::id(),
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::write(&sibling, text).map_err(|e| format!("{}: {}", sibling.display(), e))?;
    let compiled = crate::Compiler::new(crate::CompilerConfig::default()).compile_file(&sibling);
    let removed = std::fs::remove_file(&sibling);
    // Name the real file, not the scratch copy.
    let mut result = compiled.map_err(|e| {
        e.to_string().replace(&sibling.display().to_string(), &path.display().to_string())
    })?;
    removed.map_err(|e| format!("{}: {}", sibling.display(), e))?;
    result.name_file_as(&sibling, &path.display().to_string());
    Ok(result)
}

fn syntax_tree(source: &str, what: &str) -> Result<Value, EditError> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|e| EditError::Unsafe(format!("{what} does not lex: {e}")))?;
    let model = Parser::new(tokens)
        .parse()
        .map_err(|e| EditError::Unsafe(format!("{what} does not parse: {e}")))?;
    serde_json::to_value(&model).map_err(|e| EditError::Unsafe(e.to_string()))
}

/// The edit must be what was asked and nothing more: both texts parse, and
/// their syntax trees differ in one element only, on `key`.
///
/// One element may show the change in several places: the parser keeps an
/// attribute both in a field of its element and in the element's attribute
/// map, a class keeps its fields in a list of their own (`fields`), and a
/// port's `interface_type` or `port_type` is read off the attribute set.
/// Those all stand on the one element; anything on another is refused.
fn verify(original: &str, edited: &str, key: &str) -> Result<(), EditError> {
    let before = syntax_tree(original, "the model")?;
    let after = syntax_tree(edited, "the edited model")?;
    let mut found = Vec::new();
    structure::changed(&before, &after, String::new(), &mut found);

    let about_key = |path: &str| path.split('/').any(|segment| segment == key || segment == "fields");
    // The element a change about the key stands on: the path up to the
    // key, less the attribute map or the field list it is kept in.
    let owner = |path: &str| {
        let mut segments: Vec<&str> = path.split('/').take_while(|segment| *segment != key).collect();
        if let Some(fields) = segments.iter().position(|segment| *segment == "fields") {
            segments.truncate(fields);
        }
        if segments.last() == Some(&"attributes") {
            segments.pop();
        }
        segments.join("/")
    };
    let owners: std::collections::BTreeSet<String> = found.iter().filter(|(path, _, _)| about_key(path)).map(|(path, _, _)| owner(path)).collect();
    if owners.len() > 1 {
        return Err(EditError::Unsafe(format!(
            "'{key}' would change in {} places of the model",
            owners.len()
        )));
    }
    let element = owners.into_iter().next();
    let is_scalar = |value: Option<&Value>| value.is_some_and(|value| !value.is_array() && !value.is_object());
    for (path, old, new) in &found {
        if about_key(path) {
            continue;
        }
        // A single value the parser derives from the attribute, on the same element.
        let parent = path.rsplit_once('/').map_or("", |(parent, _)| parent);
        let derived = element.as_deref() == Some(parent) && is_scalar(*old) && is_scalar(*new);
        if !derived {
            return Err(EditError::Unsafe(format!(
                "it would also change `{}`, not only '{key}'",
                path.trim_start_matches('/')
            )));
        }
    }
    Ok(())
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
        safety_level: "ASIL-B"
    }

    component "Monitor" { id: "LC-002" }

    component "Logger" {}
}
"#;

    #[test]
    fn a_port_turns_around_on_its_declaration_line() {
        let source = "logical_architecture L {\n    component \"A\" {\n        port out \"p\"\n        port in \"q\" { protocol: \"x\" }\n    }\n}\n";
        let turned = set_attribute(source, "A.p", "direction", "in").unwrap();
        assert!(turned.contains("port in \"p\""), "{turned}");
        let again = set_attribute(&turned, "A.q", "direction", "out").unwrap();
        assert!(again.contains("port out \"q\" { protocol: \"x\" }"), "{again}");
        assert!(set_attribute(source, "A.p", "direction", "sideways").is_err());
    }

    #[test]
    fn replaces_a_value_and_nothing_else() {
        let edited = set_attribute(MODEL, "LC-001", "latency", "10 ms").unwrap();
        assert_eq!(edited, MODEL.replace("latency: 25 ms", "latency: 10 ms"));
        assert!(edited.contains("// worst case"));
    }

    #[test]
    fn replaces_a_value_of_another_shape() {
        let edited = set_attribute(MODEL, "LC-001", "safety_level", "[\"ASIL-B\", \"SIL-2\"]").unwrap();
        assert!(edited.contains("safety_level: [\"ASIL-B\", \"SIL-2\"]\n"));
    }

    #[test]
    fn adds_a_missing_attribute_in_the_block_layout() {
        let edited = set_attribute(MODEL, "LC-001", "owner", "\"Chassis\"").unwrap();
        assert!(edited.contains("        safety_level: \"ASIL-B\"\n        owner: \"Chassis\"\n    }"));

        let edited = set_attribute(MODEL, "LC-002", "latency", "5 ms").unwrap();
        assert!(edited.contains("component \"Monitor\" { id: \"LC-002\" latency: 5 ms }"));

        let edited = set_attribute(MODEL, "Logger", "latency", "5 ms").unwrap();
        assert!(edited.contains("component \"Logger\" { latency: 5 ms }"));
    }

    #[test]
    fn finds_an_element_by_name_when_it_writes_no_id() {
        let edited = set_attribute(MODEL, "Logger", "description", "\"Audit trail\"").unwrap();
        assert!(edited.contains("component \"Logger\" { description: \"Audit trail\" }"));
        // "Controller" writes an id: it answers to the id, not to the name.
        assert_eq!(
            set_attribute(MODEL, "Controller", "latency", "1 ms"),
            Err(EditError::NotFound("Controller".to_string()))
        );
    }

    #[test]
    fn removes_an_attribute_with_its_line() {
        let edited = remove_attribute(MODEL, "LC-001", "safety_level").unwrap();
        assert_eq!(edited, MODEL.replace("        safety_level: \"ASIL-B\"\n", ""));
    }

    #[test]
    fn removing_an_attribute_keeps_the_comment_beside_it() {
        let edited = remove_attribute(MODEL, "LC-001", "latency").unwrap();
        assert!(!edited.contains("latency"));
        assert!(edited.contains("        // worst case\n"));
    }

    #[test]
    fn refuses_what_it_cannot_do_exactly() {
        assert_eq!(
            remove_attribute(MODEL, "LC-002", "latency"),
            Err(EditError::NoSuchAttribute { element: "LC-002".to_string(), key: "latency".to_string() })
        );
        assert_eq!(
            set_attribute(MODEL, "LC-404", "latency", "1 ms"),
            Err(EditError::NotFound("LC-404".to_string()))
        );
        for value in ["", "ASIL-B", "1 ms 2 ms", "\"a\" // note", "\"x\" }", "[1, 2"] {
            assert!(
                matches!(set_attribute(MODEL, "LC-001", "latency", value), Err(EditError::Invalid(_))),
                "`{value}` should be refused"
            );
        }
        assert!(matches!(
            set_attribute(MODEL, "LC-001", "not a key", "1"),
            Err(EditError::Invalid(_))
        ));
    }

    #[test]
    fn a_trace_naming_an_element_is_not_that_element() {
        let source = "requirements system {\n    req \"R-1\" \"Stop\" {\n        priority: High\n    }\n}\ntrace \"R-1\" satisfies \"R-0\" {}\n";
        let edited = set_attribute(source, "R-1", "priority", "Low").unwrap();
        assert!(edited.contains("priority: Low"));
    }

    #[test]
    fn refuses_an_ambiguous_element() {
        let twice = "logical_architecture \"A\" {\n    component \"X\" {}\n    component \"X\" {}\n}\n";
        assert_eq!(
            set_attribute(twice, "X", "latency", "1 ms"),
            Err(EditError::Ambiguous { element: "X".to_string(), lines: vec![2, 3] })
        );
    }

    #[test]
    fn refuses_a_value_the_parser_rejects() {
        // `25 parsecs` has the shape of a quantity but is not one.
        assert!(matches!(
            set_attribute(MODEL, "LC-001", "latency", "25 parsecs"),
            Err(EditError::Invalid(_))
        ));
    }

    #[test]
    fn confirms_the_effect_on_the_compiled_model() {
        let source = "requirements system {\n    req \"R-1\" \"Stop in time\" {\n        priority: High\n    }\n}\n";
        let compiled = |text: &str| {
            let result = crate::Compiler::new(crate::CompilerConfig::default())
                .compile_string(text)
                .unwrap();
            crate::compiler::elements::build(&result.ast, &result.semantic_model)
        };

        let edited = set_attribute(source, "R-1", "priority", "Low").unwrap();
        assert_eq!(confirm_effect(&compiled(&edited), "R-1", "priority", "Low"), Ok(()));

        // The title is given on the declaration line: the edit rewrites it
        // there, where the compiler reads it.
        let edited = set_attribute(source, "R-1", "title", "\"Stop sooner\"").unwrap();
        assert!(edited.contains("req \"R-1\" \"Stop sooner\" {"), "{edited}");
        assert_eq!(confirm_effect(&compiled(&edited), "R-1", "title", "\"Stop sooner\""), Ok(()));
    }

    #[test]
    fn setting_the_same_value_changes_nothing() {
        assert_eq!(set_attribute(MODEL, "LC-001", "latency", "25 ms").unwrap(), MODEL);
    }
}
