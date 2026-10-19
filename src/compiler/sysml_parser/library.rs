//! The SysML v2 standard library as a skeleton: the packages, types and
//! features of the OMG library with their names, metaclasses,
//! specializations, imports and aliases, without documentation, values or
//! expressions. Names a model takes from the library (`ISQ::mass`,
//! `ScalarValues::Real`, `ShapeItems::Box`) resolve against it, and what
//! they inherit with them.
//!
//! `spec/sysml_library_skeleton.tsv` is not written by hand: it is derived
//! from the library of the SysML v2 release by `tests/sysml_library_skeleton.rs`,
//! which fails when the file no longer matches what the parser reads there.
//! One line per element: depth, metaclass, name, short name, flags, then
//! one `K:target` per relationship (`T` typing, `C` subclassification,
//! `S` subsetting, `R` redefinition, `F` reference, `X` crossing, `I`
//! import, `A` alias), separated by tabs.

use super::parser::parse_target;
use super::syntax::*;
use std::collections::HashMap;
use std::sync::OnceLock;

const SKELETON: &str = include_str!("../../../spec/sysml_library_skeleton.tsv");

/// Kinds that hold nothing a name can designate.
const LEFT_OUT: &[&str] = &["Comment", "Documentation", "TextualRepresentation", "Relationship", "Expression", "MetadataUsage", "Filter"];

/// The library packages, read once.
pub fn roots() -> &'static [Node] {
    static ROOTS: OnceLock<Vec<Node>> = OnceLock::new();
    ROOTS.get_or_init(|| read(SKELETON))
}

/// The skeleton of parsed library texts.
pub fn skeleton_of(roots: &[Node]) -> String {
    let mut out = String::new();
    for root in roots {
        write(root, 0, &mut out);
    }
    out
}

fn kept(node: &Node) -> bool {
    if LEFT_OUT.contains(&node.kind) || matches!(node.detail, Detail::Result(_) | Detail::Filter(_) | Detail::Annotation { .. }) {
        return false;
    }
    let implicit = node.specializations.iter().any(|s| matches!(s.kind, SpecKind::Redefinition | SpecKind::Reference));
    node.name.is_some() || node.short_name.is_some() || implicit || matches!(node.detail, Detail::Import { .. } | Detail::Alias { .. })
}

fn write(node: &Node, depth: usize, out: &mut String) {
    if !kept(node) {
        return;
    }
    let mut fields = vec![depth.to_string(), node.kind.to_string(), node.name.clone().unwrap_or_default(), node.short_name.clone().unwrap_or_default()];
    let mut flags = Vec::new();
    if node.prefixes.visibility == Some("public") {
        flags.push("public");
    }
    if let Some(direction) = node.prefixes.direction {
        flags.push(direction);
    }
    if node.prefixes.is_abstract {
        flags.push("abstract");
    }
    if let Detail::Import { namespace, recursive, .. } = &node.detail {
        if *namespace {
            flags.push("ns");
        }
        if *recursive {
            flags.push("rec");
        }
    }
    fields.push(flags.join(","));
    for specialization in &node.specializations {
        let key = match (specialization.kind, specialization.conjugated) {
            (SpecKind::Typing, true) => "~",
            (SpecKind::Typing, false) => "T",
            (SpecKind::Subclassification, _) => "C",
            (SpecKind::Subsetting, _) => "S",
            (SpecKind::Redefinition, _) => "R",
            (SpecKind::Reference, _) => "F",
            (SpecKind::Crossing, _) => "X",
        };
        fields.push(format!("{key}:{}", specialization.target.text()));
    }
    match &node.detail {
        Detail::Import { target, .. } => fields.push(format!("I:{}", target.text())),
        Detail::Alias { target } => fields.push(format!("A:{}", target.text())),
        _ => {}
    }
    out.push_str(&fields.join("\t"));
    out.push('\n');
    for member in &node.members {
        write(member, depth + 1, out);
    }
}

/// Read a skeleton back into nodes. Lines that do not read are skipped:
/// the skeleton is generated, and a name missing from it is only a name
/// that stays unresolved.
fn read(text: &str) -> Vec<Node> {
    let mut kinds: HashMap<String, &'static str> = HashMap::new();
    let mut stack: Vec<Node> = Vec::new();
    let mut roots: Vec<Node> = Vec::new();
    let close = |stack: &mut Vec<Node>, roots: &mut Vec<Node>, depth: usize| {
        while stack.len() > depth {
            let node = stack.pop().expect("a node to close");
            match stack.last_mut() {
                Some(parent) => parent.members.push(node),
                None => roots.push(node),
            }
        }
    };
    // Below a line that does not read, its whole subtree is skipped.
    let mut skipping: Option<usize> = None;
    for line in text.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        let depth = fields.first().and_then(|d| d.parse::<usize>().ok());
        if let (Some(skip), Some(depth)) = (skipping, depth) {
            if depth > skip {
                continue;
            }
            skipping = None;
        }
        let ([_, kind, name, short, flags, relationships @ ..], Some(depth)) = (fields.as_slice(), depth) else {
            if let Some(depth) = depth {
                skipping = Some(depth);
            }
            continue;
        };
        if depth > stack.len() {
            skipping = Some(depth);
            continue;
        }
        close(&mut stack, &mut roots, depth);
        // One leaked string per distinct metaclass, once (the roots are read once).
        let kind: &'static str = kinds.entry(kind.to_string()).or_insert_with(|| Box::leak(kind.to_string().into_boxed_str()));
        stack.push(node_of(kind, name, short, flags, relationships));
    }
    close(&mut stack, &mut roots, 0);
    roots
}

fn node_of(kind: &'static str, name: &str, short: &str, flags: &str, relationships: &[&str]) -> Node {
    let mut node = Node::new(kind, "OwningMembership");
    node.name = Some(name.to_string()).filter(|n| !n.is_empty());
    node.short_name = Some(short.to_string()).filter(|n| !n.is_empty());
    let flags: Vec<&str> = flags.split(',').collect();
    node.prefixes.visibility = flags.contains(&"public").then_some("public");
    node.prefixes.direction = ["in", "out", "inout"].into_iter().find(|d| flags.contains(d));
    node.prefixes.is_abstract = flags.contains(&"abstract");
    for relationship in relationships {
        let Some((key, text)) = relationship.split_once(':') else { continue };
        let Ok(target) = parse_target(text) else { continue };
        let kind = match key {
            "T" | "~" => SpecKind::Typing,
            "C" => SpecKind::Subclassification,
            "S" => SpecKind::Subsetting,
            "R" => SpecKind::Redefinition,
            "F" => SpecKind::Reference,
            "X" => SpecKind::Crossing,
            "I" | "A" => {
                let name = target.first().clone();
                node.detail = if key == "I" {
                    Detail::Import { target: name, namespace: flags.contains(&"ns"), recursive: flags.contains(&"rec"), all: false, filters: Vec::new() }
                } else {
                    Detail::Alias { target: name }
                };
                continue;
            }
            _ => continue,
        };
        node.specializations.push(Specialization { kind, target, conjugated: key == "~" });
    }
    node
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_skeleton_reads_back_as_the_tree_it_was_written_from() {
        let text = "standard library package Q { public import B::*; attribute def MassValue :> ScalarQuantityValue { doc /* d */ } attribute mass : MassValue :> scalarQuantities = 1; alias M for MassValue; }";
        let roots = crate::compiler::sysml_parser::parse(text).unwrap();
        let skeleton = skeleton_of(&roots);
        let back = read(&skeleton);
        assert_eq!(skeleton_of(&back), skeleton);
        let q = &back[0];
        assert_eq!(q.members.len(), 4, "{skeleton}");
        assert!(matches!(&q.members[0].detail, Detail::Import { namespace: true, .. }));
        assert_eq!(q.members[0].prefixes.visibility, Some("public"));
        assert!(q.members[1].members.is_empty(), "documentation is left out");
        assert_eq!(q.members[2].specializations.len(), 2);
        assert!(q.members[2].value.is_none());
    }
}
