//! Merge of layout files, by the element each entry places.

use crate::compiler::diagram::layout_file::LayoutFile;
use crate::compiler::semgit::layout_diff::{build, entries, EntryPath};
use serde_json::Value;
use std::collections::BTreeSet;

pub struct LayoutMerge {
    pub text: String,
    /// Entries changed differently on both sides: theirs was taken.
    pub taken_from_theirs: Vec<String>,
}

fn parse(text: &str) -> Result<Option<Value>, String> {
    if text.trim().is_empty() {
        return Ok(None);
    }
    serde_json::from_str(text).map(Some).map_err(|e| format!("not a layout file: {e}"))
}

fn name(path: &EntryPath) -> String {
    path.iter().filter(|p| *p != "present").map(|p| p.replace('\u{1}', "/").trim_end_matches('/').to_string()).collect::<Vec<_>>().join(" / ")
}

pub fn merge_layout(base: &str, ours: &str, theirs: &str) -> Result<LayoutMerge, String> {
    let (b, o, t) = (parse(base).map_err(|e| format!("base: {e}"))?, parse(ours).map_err(|e| format!("ours: {e}"))?, parse(theirs).map_err(|e| format!("theirs: {e}"))?);
    let (be, oe, te) = (entries(b.as_ref()), entries(o.as_ref()), entries(t.as_ref()));
    let paths: BTreeSet<&EntryPath> = be.keys().chain(oe.keys()).chain(te.keys()).collect();
    let mut merged = std::collections::BTreeMap::new();
    let mut taken = Vec::new();
    for path in paths {
        let (bv, ov, tv) = (be.get(path), oe.get(path), te.get(path));
        let chosen = if ov == tv || tv == bv {
            ov
        } else if ov == bv {
            tv
        } else {
            taken.push(name(path));
            tv
        };
        if let Some(value) = chosen {
            merged.insert(path.clone(), value.clone());
        }
    }
    let value = build(&merged);
    let layout: LayoutFile = serde_json::from_value(value).map_err(|e| format!("the merged layout is not valid: {e}"))?;
    Ok(LayoutMerge { text: layout.to_json(), taken_from_theirs: taken })
}
