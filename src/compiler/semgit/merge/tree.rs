//! A model file as a tree of declarations, each with the text that belongs
//! to it, so that two versions can be merged by element identity and every
//! comment, blank line and indentation survives.
//!
//! A node is a declaration: its header line, its own lines (attributes,
//! closing brace) and its children, in the order written. The comments and
//! blank lines just above a declaration travel with it (its `lead`).

use crate::compiler::source_edit::{outline, Declaration};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub enum Item {
    Line(String),
    Child(Node),
}

#[derive(Debug, Clone)]
pub struct Node {
    /// What identifies the declaration among its siblings: `component#LC-001`,
    /// or the header text when it has no id.
    pub key: String,
    pub keyword: String,
    pub name: Option<String>,
    pub id: Option<String>,
    pub lead: Vec<String>,
    pub items: Vec<Item>,
}

impl Node {
    pub fn root() -> Node {
        Node { key: String::new(), keyword: String::new(), name: None, id: None, lead: Vec::new(), items: Vec::new() }
    }

    /// The text of the declaration, its lead included.
    pub fn lines(&self) -> Vec<String> {
        let mut out = self.lead.clone();
        out.extend(self.body());
        out
    }

    /// The text of the declaration, its lead left out.
    pub fn body(&self) -> Vec<String> {
        let mut out = Vec::new();
        for item in &self.items {
            match item {
                Item::Line(text) => out.push(text.clone()),
                Item::Child(child) => out.extend(child.lines()),
            }
        }
        out
    }

    pub fn hash(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.lines().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// The token this node stands for in the sequence of its parent's items.
    pub fn token(&self) -> String {
        format!("C:{}@{}", self.key, self.hash())
    }

    /// `component 'Controller' [LC-001]`.
    pub fn label(&self) -> String {
        let mut out = self.keyword.clone();
        if let Some(name) = &self.name {
            out.push_str(&format!(" '{name}'"));
        }
        if let Some(id) = &self.id {
            out.push_str(&format!(" [{id}]"));
        }
        out.trim().to_string()
    }

    pub fn children(&self) -> impl Iterator<Item = &Node> {
        self.items.iter().filter_map(|i| if let Item::Child(c) = i { Some(c) } else { None })
    }
}

pub fn is_trivia(line: &str) -> bool {
    let t = line.trim();
    t.is_empty() || t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') || t.starts_with("*/")
}

fn key_of(declaration: &Declaration) -> String {
    match &declaration.id {
        Some(id) => format!("{}#{}", declaration.keyword, id),
        None => declaration.header.split_whitespace().collect::<Vec<_>>().join(" "),
    }
}

fn lines_between(lines: &[String], from: usize, to: usize) -> Vec<Item> {
    (from..=to).filter(|l| *l >= 1 && *l <= lines.len()).map(|l| Item::Line(lines[l - 1].clone())).collect()
}

/// The items covering lines `start..=end` (1-based): the lines, and the
/// declarations in `children` as nodes. `None` when the text is laid out in
/// a way lines cannot follow (two declarations on one line).
fn items_of(lines: &[String], start: usize, end: usize, children: &[Declaration], has_header: bool) -> Option<Vec<Item>> {
    let mut items: Vec<Item> = Vec::new();
    let mut next = start;
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for child in children {
        if child.line < next || child.end_line > end || (has_header && child.line <= start) {
            return None;
        }
        items.extend(lines_between(lines, next, child.line - 1));
        let mut lead: Vec<String> = Vec::new();
        let floor = if has_header { 1 } else { 0 };
        while items.len() > floor {
            match items.last() {
                Some(Item::Line(text)) if is_trivia(text) => {
                    lead.push(text.clone());
                    items.pop();
                }
                _ => break,
            }
        }
        lead.reverse();
        let mut key = key_of(child);
        let count = seen.entry(key.clone()).or_insert(0);
        *count += 1;
        if *count > 1 {
            key = format!("{key}#{count}");
        }
        let inner = items_of(lines, child.line, child.end_line, &child.children, true).unwrap_or_else(|| lines_between(lines, child.line, child.end_line));
        items.push(Item::Child(Node { key, keyword: child.keyword.clone(), name: child.name.clone(), id: child.id.clone(), lead, items: inner }));
        next = child.end_line + 1;
    }
    items.extend(lines_between(lines, next, end));
    Some(items)
}

/// What a file is made of, line by line.
#[derive(Debug, Clone)]
pub struct Parsed {
    pub root: Node,
    pub crlf: bool,
    pub final_newline: bool,
}

pub fn split_lines(source: &str) -> (Vec<String>, bool, bool) {
    let crlf = source.contains("\r\n");
    let final_newline = source.ends_with('\n');
    let body = source.strip_suffix('\n').unwrap_or(source);
    let lines: Vec<String> = if source.is_empty() { Vec::new() } else { body.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l).to_string()).collect() };
    (lines, crlf, final_newline)
}

pub fn parse(source: &str) -> Result<Parsed, String> {
    let (lines, crlf, final_newline) = split_lines(source);
    let declarations = outline(source).map_err(|e| e.to_string())?;
    let items = match items_of(&lines, 1, lines.len(), &declarations, false) {
        Some(items) => items,
        None => lines.iter().map(|l| Item::Line(l.clone())).collect(),
    };
    let mut root = Node::root();
    root.items = items;
    Ok(Parsed { root, crlf, final_newline })
}

pub fn join(lines: &[String], crlf: bool, final_newline: bool) -> String {
    let sep = if crlf { "\r\n" } else { "\n" };
    let mut text = lines.join(sep);
    if final_newline && !lines.is_empty() {
        text.push_str(sep);
    }
    text
}
