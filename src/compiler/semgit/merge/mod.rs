//! Semantic three-way merge of ArcLang files, by element identity.
//!
//! Two branches that change different elements, or different attributes of
//! one element, merge without a conflict wherever the changes are in the
//! file. A real conflict (the same attribute changed differently, an element
//! changed on one side and removed on the other) is written as markers
//! around the ELEMENT, naming it and both values; the file stays well
//! formed, each side being a whole element. Comments, formatting and the
//! order of the declarations nobody touched are kept as written.

mod diff3;
pub mod install;
pub mod layout;
mod tree;

use diff3::{merge3, Chunk};
use serde::Serialize;
use std::collections::HashMap;
use tree::{Item, Node};

#[derive(Debug, Clone, Serialize)]
pub struct Conflict {
    pub element: String,
    pub label: String,
    pub reason: String,
    pub ours: String,
    pub theirs: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Merged {
    pub text: String,
    pub conflicts: Vec<Conflict>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Policy {
    Markers,
    Ours,
    Theirs,
}

struct Ctx {
    policy: Policy,
    conflicts: Vec<Conflict>,
}

const OURS_MARK: &str = "<<<<<<< ours";
const MIDDLE_MARK: &str = "=======";
const THEIRS_MARK: &str = ">>>>>>> theirs";

fn short(lines: &[String]) -> String {
    let joined = lines.iter().map(|l| l.trim()).filter(|l| !l.is_empty() && *l != "{" && *l != "}").collect::<Vec<_>>().join("; ");
    if joined.chars().count() > 100 {
        format!("{}...", joined.chars().take(100).collect::<String>())
    } else {
        joined
    }
}

/// The words of a line, a quoted string being one word.
fn words(line: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    let (mut quoted, mut escaped) = (false, false);
    for c in line.chars() {
        if quoted {
            current.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                quoted = false;
            }
        } else if c == '"' {
            quoted = true;
            current.push(c);
        } else if c.is_whitespace() {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// Two attributes of one element written on one line, changed one on each
/// side: merged word by word.
fn merge_words(base: &[String], ours: &[String], theirs: &[String]) -> Option<Vec<String>> {
    let [b] = base else { return None };
    let [o] = ours else { return None };
    let [t] = theirs else { return None };
    let strip = |l: &String| l.strip_prefix("L:").map(str::to_string);
    let (b, o, t) = (strip(b)?, strip(o)?, strip(t)?);
    let (bw, ow, tw) = (words(&b), words(&o), words(&t));
    let mut merged = Vec::new();
    for chunk in merge3(&bw, &ow, &tw) {
        match chunk {
            Chunk::Text(words) => merged.extend(words),
            Chunk::Region { .. } => return None,
        }
    }
    let indent: String = o.chars().take_while(|c| c.is_whitespace()).collect();
    Some(vec![format!("L:{indent}{}", merged.join(" "))])
}

fn tokens_of(node: &Node) -> Vec<String> {
    node.items
        .iter()
        .map(|item| match item {
            Item::Line(text) => format!("L:{text}"),
            Item::Child(child) => child.token(),
        })
        .collect()
}

fn is_child(token: &str) -> bool {
    token.starts_with("C:")
}

fn key_of_token(token: &str) -> &str {
    let rest = &token[2..];
    rest.rsplit_once('@').map(|(k, _)| k).unwrap_or(rest)
}

fn plain(tokens: &[String]) -> Vec<String> {
    tokens.iter().filter_map(|t| t.strip_prefix("L:").map(str::to_string)).collect()
}

/// A marker block around an element: ours, then theirs.
fn marked(label: &str, reason: &str, ours: Vec<String>, theirs: Vec<String>) -> Vec<String> {
    let mut out = vec![format!("{OURS_MARK} {label}: {reason}")];
    out.extend(ours);
    out.push(MIDDLE_MARK.to_string());
    out.extend(theirs);
    out.push(format!("{THEIRS_MARK} {label}"));
    out
}

struct Versions<'a> {
    nodes: HashMap<String, &'a Node>,
}

impl<'a> Versions<'a> {
    fn of(nodes: [&'a Node; 3]) -> Self {
        let mut map = HashMap::new();
        for node in nodes {
            for child in node.children() {
                map.insert(child.token(), child);
            }
        }
        Versions { nodes: map }
    }

    fn node(&self, token: &str) -> Option<&'a Node> {
        self.nodes.get(token).copied()
    }
}

fn emit(token: &str, versions: &Versions) -> Vec<String> {
    match token.strip_prefix("L:") {
        Some(text) => vec![text.to_string()],
        None => versions.node(token).map(|n| n.lines()).unwrap_or_default(),
    }
}

/// The whole declaration three versions agree to describe, merged.
fn merge_node(base: &Node, ours: &Node, theirs: &Node, ctx: &mut Ctx) -> Vec<String> {
    if ours.hash() == theirs.hash() || theirs.hash() == base.hash() {
        return ours.lines();
    }
    if ours.hash() == base.hash() {
        return theirs.lines();
    }
    let known = ctx.conflicts.len();
    let merged = merge_node_with(base, ours, theirs, ctx);
    match merged {
        Ok(lines) => lines,
        Err(reason) => {
            // What its children reported is shown inside the markers instead.
            ctx.conflicts.truncate(known);
            // The declaration itself conflicts: both whole versions, each with
            // the rest merged its own way.
            let (mut as_ours, mut as_theirs) = (Ctx { policy: Policy::Ours, conflicts: Vec::new() }, Ctx { policy: Policy::Theirs, conflicts: Vec::new() });
            let o = merge_node_with(base, ours, theirs, &mut as_ours).unwrap_or_else(|_| ours.lines());
            let t = merge_node_with(base, ours, theirs, &mut as_theirs).unwrap_or_else(|_| theirs.lines());
            let (mut o, mut t) = (o, t);
            // The lead is common unless it is what conflicts.
            if ours.lead == theirs.lead {
                o.drain(..ours.lead.len().min(o.len()));
                t.drain(..theirs.lead.len().min(t.len()));
                let mut out = ours.lead.clone();
                out.extend(marked(&ours.label(), &reason, o, t));
                ctx.conflicts.push(conflict(ours, &reason, &out));
                return out;
            }
            let out = marked(&ours.label(), &reason, o, t);
            ctx.conflicts.push(conflict(ours, &reason, &out));
            out
        }
    }
}

fn conflict(node: &Node, reason: &str, block: &[String]) -> Conflict {
    let middle = block.iter().position(|l| l == MIDDLE_MARK).unwrap_or(block.len());
    let text = |lines: &[String]| lines.join("\n");
    let start = block.iter().position(|l| l.starts_with(OURS_MARK)).unwrap_or(0);
    Conflict {
        element: node.id.clone().or_else(|| node.name.clone()).unwrap_or_else(|| node.key.clone()),
        label: node.label(),
        reason: reason.to_string(),
        ours: text(&block[(start + 1).min(middle)..middle]),
        theirs: text(&block[(middle + 1).min(block.len())..block.len().saturating_sub(1)]),
    }
}

/// Merge the content of a declaration. `Err` carries the reason when its own
/// lines conflict (and the policy is to mark it).
fn merge_node_with(base: &Node, ours: &Node, theirs: &Node, ctx: &mut Ctx) -> Result<Vec<String>, String> {
    let mut failure: Option<String> = None;
    let mut out: Vec<String> = Vec::new();

    // The comments above it.
    let lead = if ours.lead == theirs.lead || theirs.lead == base.lead {
        ours.lead.clone()
    } else if ours.lead == base.lead {
        theirs.lead.clone()
    } else {
        failure.get_or_insert_with(|| format!("the comments above it were changed differently: {} (ours) vs {} (theirs)", short(&ours.lead), short(&theirs.lead)));
        match ctx.policy {
            Policy::Theirs => theirs.lead.clone(),
            _ => ours.lead.clone(),
        }
    };
    out.extend(lead);

    let (bt, ot, tt) = (tokens_of(base), tokens_of(ours), tokens_of(theirs));
    let versions = Versions::of([base, ours, theirs]);
    let mut emitted_keys: Vec<String> = Vec::new();
    for chunk in merge3(&bt, &ot, &tt) {
        match chunk {
            Chunk::Text(tokens) => {
                for token in tokens {
                    if is_child(&token) {
                        emitted_keys.push(key_of_token(&token).to_string());
                    }
                    out.extend(emit(&token, &versions));
                }
            }
            Chunk::Region { base: b, ours: o, theirs: t } => {
                let only_children = [&b, &o, &t].iter().all(|s| s.iter().all(|x| is_child(x)));
                if only_children {
                    let lines = resolve_children(&b, &o, &t, &versions, ctx, &mut emitted_keys);
                    out.extend(lines);
                } else if let Some(tokens) = merge_words(&b, &o, &t) {
                    out.extend(tokens.iter().flat_map(|x| emit(x, &versions)));
                } else {
                    failure.get_or_insert_with(|| format!("{} (ours) vs {} (theirs)", short_or(&plain(&o)), short_or(&plain(&t))));
                    let chosen = if ctx.policy == Policy::Theirs { &t } else { &o };
                    for token in chosen {
                        out.extend(emit(token, &versions));
                    }
                }
            }
        }
    }
    let mut seen = std::collections::HashSet::new();
    if let Some(dup) = emitted_keys.iter().find(|k| !seen.insert(k.as_str())) {
        failure.get_or_insert_with(|| format!("'{dup}' ended up declared twice (moved on one side, changed on the other)"));
    }
    match failure {
        Some(reason) if ctx.policy == Policy::Markers => Err(reason),
        _ => Ok(out),
    }
}

fn short_or(lines: &[String]) -> String {
    let s = short(lines);
    if s.is_empty() {
        "nothing".to_string()
    } else {
        s
    }
}

/// Declarations changed by both sides in the same place of their container,
/// matched by identity.
fn resolve_children(base: &[String], ours: &[String], theirs: &[String], versions: &Versions, ctx: &mut Ctx, emitted: &mut Vec<String>) -> Vec<String> {
    let by_key = |tokens: &[String]| -> HashMap<String, String> { tokens.iter().map(|t| (key_of_token(t).to_string(), t.clone())).collect() };
    let (b, o, t) = (by_key(base), by_key(ours), by_key(theirs));
    let mut order: Vec<String> = Vec::new();
    for token in ours {
        order.push(key_of_token(token).to_string());
    }
    // What only theirs holds goes after the declaration it followed.
    let mut inserted: Vec<(String, Option<String>)> = Vec::new();
    for (i, token) in theirs.iter().enumerate() {
        let key = key_of_token(token).to_string();
        if !order.contains(&key) {
            let anchor = theirs[..i].iter().rev().map(|t| key_of_token(t).to_string()).find(|k| order.contains(k));
            inserted.push((key, anchor));
        }
    }
    let mut new_from_theirs: Vec<String> = Vec::new();
    for (key, anchor) in inserted {
        let at = match &anchor {
            Some(a) => order.iter().position(|k| k == a).map(|p| p + 1).unwrap_or(order.len()),
            None => 0,
        };
        order.insert(at, key.clone());
        new_from_theirs.push(key);
    }
    // A run of additions made on both sides has no order of its own: by key.
    let is_new = |k: &String| !b.contains_key(k);
    let mut i = 0;
    while i < order.len() {
        if !is_new(&order[i]) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < order.len() && is_new(&order[j]) {
            j += 1;
        }
        let from_theirs = order[i..j].iter().any(|k| new_from_theirs.contains(k));
        let from_ours = order[i..j].iter().any(|k| !new_from_theirs.contains(k));
        if from_theirs && from_ours {
            order[i..j].sort();
        }
        i = j;
    }
    let mut out = Vec::new();
    for key in order {
        let node = |m: &HashMap<String, String>| m.get(&key).and_then(|t| versions.node(t));
        match (node(&b), node(&o), node(&t)) {
            (Some(bn), Some(on), Some(tn)) => {
                emitted.push(key.clone());
                out.extend(merge_node(bn, on, tn, ctx));
            }
            (None, Some(on), Some(tn)) => {
                emitted.push(key.clone());
                if on.hash() == tn.hash() {
                    out.extend(on.lines());
                } else {
                    out.extend(side_conflict(on, Some(on), Some(tn), "added on both sides with different content", ctx));
                }
            }
            (None, Some(on), None) => {
                emitted.push(key.clone());
                out.extend(on.lines());
            }
            (None, None, Some(tn)) => {
                emitted.push(key.clone());
                out.extend(tn.lines());
            }
            (Some(bn), Some(on), None) => {
                if on.hash() == bn.hash() {
                    continue;
                }
                emitted.push(key.clone());
                out.extend(side_conflict(on, Some(on), None, "changed on ours, removed on theirs", ctx));
            }
            (Some(bn), None, Some(tn)) => {
                if tn.hash() == bn.hash() {
                    continue;
                }
                emitted.push(key.clone());
                out.extend(side_conflict(tn, None, Some(tn), "removed on ours, changed on theirs", ctx));
            }
            _ => {}
        }
    }
    out
}

/// An element held by one or both sides that cannot be settled: whole on
/// each side (or absent), under markers.
fn side_conflict(node: &Node, ours: Option<&Node>, theirs: Option<&Node>, reason: &str, ctx: &mut Ctx) -> Vec<String> {
    let pick = |n: Option<&Node>| n.map(|n| n.lines()).unwrap_or_default();
    match ctx.policy {
        Policy::Ours => pick(ours),
        Policy::Theirs => pick(theirs),
        Policy::Markers => {
            // Their lead is common when both have the element; keep it outside.
            let (o, t) = (pick(ours), pick(theirs));
            let out = marked(&node.label(), reason, o, t);
            ctx.conflicts.push(conflict(node, reason, &out));
            out
        }
    }
}

/// Merge three versions of an ArcLang file.
pub fn merge_arc(base: &str, ours: &str, theirs: &str) -> Result<Merged, String> {
    let (b, o, t) = (tree::parse(base).map_err(|e| format!("base: {e}"))?, tree::parse(ours).map_err(|e| format!("ours: {e}"))?, tree::parse(theirs).map_err(|e| format!("theirs: {e}"))?);
    let mut ctx = Ctx { policy: Policy::Markers, conflicts: Vec::new() };
    // The root cannot be wrapped in markers: its own lines conflict region by region.
    let lines = merge_root(&b.root, &o.root, &t.root, &mut ctx);
    let crlf = o.crlf;
    let final_newline = if ours == base { t.final_newline } else { o.final_newline };
    Ok(Merged { text: tree::join(&lines, crlf, final_newline), conflicts: ctx.conflicts })
}

fn merge_root(base: &Node, ours: &Node, theirs: &Node, ctx: &mut Ctx) -> Vec<String> {
    let (bt, ot, tt) = (tokens_of(base), tokens_of(ours), tokens_of(theirs));
    let versions = Versions::of([base, ours, theirs]);
    let mut out = Vec::new();
    let mut emitted = Vec::new();
    for chunk in merge3(&bt, &ot, &tt) {
        match chunk {
            Chunk::Text(tokens) => {
                for token in tokens {
                    out.extend(emit(&token, &versions));
                }
            }
            Chunk::Region { base: b, ours: o, theirs: t } => {
                if [&b, &o, &t].iter().all(|s| s.iter().all(|x| is_child(x))) {
                    out.extend(resolve_children(&b, &o, &t, &versions, ctx, &mut emitted));
                } else {
                    let lines = |tokens: &[String]| -> Vec<String> { tokens.iter().flat_map(|t| emit(t, &versions)).collect() };
                    let (lo, lt) = (lines(&o), lines(&t));
                    let reason = format!("{} (ours) vs {} (theirs)", short_or(&plain(&o)), short_or(&plain(&t)));
                    let block = marked("file level", &reason, lo.clone(), lt.clone());
                    ctx.conflicts.push(Conflict { element: "(file)".into(), label: "file level".into(), reason, ours: lo.join("\n"), theirs: lt.join("\n") });
                    out.extend(block);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests;
