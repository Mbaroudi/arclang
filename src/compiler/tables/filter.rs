//! The filter of a table, as written in `filter:` and typed in the filter
//! bar of the workbench (which implements the same rules):
//!
//! - a word matches the rows where some cell contains it (`brake`);
//! - `column:text` keeps the rows whose column contains the text
//!   (`priority:high`), `column=text` those where it is exactly the text,
//!   `column!=text` those where it is not;
//! - `is:gap` keeps the rows with a gap or a failure, `is:fail`, `is:warn`,
//!   `is:ok` the rows of that status;
//! - a term starting with `-` is negated (`-status:covered`);
//! - `"two words"` is one term. Every term must match. Case is ignored.

use super::Row;

#[derive(Debug, Clone, PartialEq)]
enum Op {
    Contains,
    Equals,
    NotEquals,
}

#[derive(Debug, Clone, PartialEq)]
struct Term {
    negated: bool,
    column: Option<String>,
    op: Op,
    value: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Query {
    terms: Vec<Term>,
}

fn split(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for c in text.chars() {
        match c {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    out.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

impl Query {
    pub fn parse(text: &str) -> Query {
        let terms = split(text)
            .into_iter()
            .map(|raw| {
                let (negated, body) = match raw.strip_prefix('-') {
                    Some(rest) if !rest.is_empty() => (true, rest.to_string()),
                    _ => (false, raw.clone()),
                };
                let lowered = body.to_lowercase();
                for (token, op) in [("!=", Op::NotEquals), ("=", Op::Equals), (":", Op::Contains)] {
                    if let Some((column, value)) = lowered.split_once(token) {
                        if !column.is_empty() && column.chars().all(|c| c.is_alphanumeric() || c == '_') {
                            return Term { negated, column: Some(column.to_string()), op, value: value.to_string() };
                        }
                    }
                }
                Term { negated, column: None, op: Op::Contains, value: lowered }
            })
            .collect();
        Query { terms }
    }

    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    pub fn matches(&self, row: &Row) -> bool {
        self.terms.iter().all(|term| term_matches(term, row) != term.negated)
    }
}

fn term_matches(term: &Term, row: &Row) -> bool {
    let compare = |text: &str| {
        let text = text.to_lowercase();
        match term.op {
            Op::Contains => text.contains(&term.value),
            Op::Equals => text == term.value,
            Op::NotEquals => text != term.value,
        }
    };
    match term.column.as_deref() {
        Some("is") => match term.value.as_str() {
            "gap" | "gaps" => row.status == "gap" || row.status == "fail",
            other => row.status == other,
        },
        Some("id") => compare(&row.id),
        Some("name") if !row.cells.contains_key("name") => compare(&row.name),
        Some("kind") if !row.cells.contains_key("kind") => compare(&row.kind),
        Some(column) => match row.cells.get(column) {
            Some(cell) => compare(&cell.text) || cell.links.iter().any(|l| compare(&l.id)),
            None => term.op == Op::NotEquals,
        },
        None => {
            [&row.id, &row.name, &row.kind].iter().any(|t| compare(t))
                || row.cells.values().any(|cell| compare(&cell.text) || cell.links.iter().any(|l| compare(&l.id)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::tables::Cell;
    use std::collections::BTreeMap;

    fn row(status: &str, priority: &str) -> Row {
        let mut cells = BTreeMap::new();
        cells.insert("priority".to_string(), Cell::text(priority));
        cells.insert("text".to_string(), Cell::text("The brake shall hold"));
        Row { id: "REQ-1".into(), uuid: "u".into(), kind: "Requirement".into(), name: "Brake".into(), status: status.into(), issues: vec![], cells }
    }

    #[test]
    fn words_columns_status_and_negation() {
        assert!(Query::parse("brake").matches(&row("ok", "High")));
        assert!(Query::parse("priority:hi").matches(&row("ok", "High")));
        assert!(!Query::parse("priority=hi").matches(&row("ok", "High")));
        assert!(Query::parse("priority=high").matches(&row("ok", "High")));
        assert!(Query::parse("priority!=low").matches(&row("ok", "High")));
        assert!(Query::parse("is:gap").matches(&row("fail", "High")));
        assert!(!Query::parse("is:gap").matches(&row("ok", "High")));
        assert!(Query::parse("-is:gap \"shall hold\"").matches(&row("ok", "High")));
        assert!(!Query::parse("-brake").matches(&row("ok", "High")));
    }
}
