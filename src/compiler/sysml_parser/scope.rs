//! Name resolution over the flattened syntax tree: what a qualified name
//! or a feature chain designates from where it is written. Members are
//! found in the namespace itself, in what it inherits (through its
//! specializations and typings, boundedly), in what it imports, then in the
//! enclosing namespaces up to the root; `$::` starts at the root.
//!
//! Imported namespaces are resolved once, when the scope is built, without
//! consulting imports themselves: an import names a declared namespace.

use super::syntax::{Detail, Node, QualifiedName, SpecKind, Target};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

/// One element of the flattened tree.
pub struct Flat<'n> {
    pub node: &'n Node,
    pub owner: Option<usize>,
    /// Children in document order (members, not relationships).
    pub children: Vec<usize>,
}

/// Flatten the tree: owners before their members, in document order.
pub fn flatten(roots: &[Node]) -> Vec<Flat<'_>> {
    let mut flat = Vec::new();
    fn walk<'n>(node: &'n Node, owner: Option<usize>, flat: &mut Vec<Flat<'n>>) -> usize {
        let index = flat.len();
        flat.push(Flat { node, owner, children: Vec::new() });
        let mut children = Vec::new();
        for member in &node.members {
            children.push(walk(member, Some(index), flat));
        }
        // Nodes a detail owns: parameters and bodies of action nodes,
        // triggers and effects of transitions, payloads of accepts.
        for owned in detail_nodes(&node.detail) {
            children.push(walk(owned, Some(index), flat));
        }
        flat[index].children = children;
        index
    }
    for root in roots {
        walk(root, None, &mut flat);
    }
    flat
}

/// The nodes a detail owns, in order.
pub fn detail_nodes(detail: &Detail) -> Vec<&Node> {
    match detail {
        Detail::Transition { trigger, effect, .. } => trigger.iter().chain(effect.iter()).map(|b| &**b).collect(),
        Detail::Accept { payload, .. } => vec![&**payload],
        Detail::If { then, otherwise, .. } => std::iter::once(&**then).chain(otherwise.iter().map(|b| &**b)).collect(),
        Detail::While { body, .. } => vec![&**body],
        Detail::For { variable, body, .. } => vec![&**variable, &**body],
        _ => Vec::new(),
    }
}

/// A resolved import of a namespace.
#[derive(Clone, Copy)]
struct Imported {
    /// The imported namespace, or the namespace holding the imported member.
    namespace: usize,
    /// `P::*`: every member; else the one member named by the import.
    whole: bool,
    recursive: bool,
    /// For a membership import: the member itself.
    member: Option<usize>,
}

pub struct Scope<'n> {
    pub flat: Vec<Flat<'n>>,
    /// Members of each namespace by declared name and short name, and by alias.
    members: Vec<HashMap<String, usize>>,
    /// Resolved imports of each namespace; `root_imports` those written at the top.
    imports: Vec<Vec<Imported>>,
    root_imports: Vec<Imported>,
    /// Indexes of the top-level elements.
    roots: Vec<usize>,
    /// The generals of each element, once resolved (a cycle resolves to nothing).
    generals_cache: RefCell<HashMap<usize, Vec<usize>>>,
}

impl<'n> Scope<'n> {
    pub fn new(roots: &'n [Node]) -> Self {
        let flat = flatten(roots);
        let n = flat.len();
        let mut members: Vec<HashMap<String, usize>> = vec![HashMap::new(); n];
        let root_indexes: Vec<usize> = flat.iter().enumerate().filter(|(_, f)| f.owner.is_none()).map(|(i, _)| i).collect();
        for (index, element) in flat.iter().enumerate() {
            let Some(owner) = element.owner else { continue };
            for name in [element.node.name.as_deref(), element.node.short_name.as_deref()].into_iter().flatten() {
                members[owner].entry(name.to_string()).or_insert(index);
            }
        }
        let mut scope = Scope { flat, members, imports: vec![Vec::new(); n], root_imports: Vec::new(), roots: root_indexes, generals_cache: RefCell::new(HashMap::new()) };
        // Imports: resolved without imports, outward from the importing namespace.
        let import_nodes: Vec<(usize, Option<usize>)> = scope.flat.iter().enumerate().filter(|(_, f)| matches!(f.node.detail, Detail::Import { .. })).map(|(i, f)| (i, f.owner)).collect();
        for (index, owner) in import_nodes {
            let Detail::Import { target, namespace: whole, recursive, .. } = &scope.flat[index].node.detail else { continue };
            let found = scope.resolve_without_imports(target, owner);
            let imported = match (found, *whole) {
                (Some(ns), true) => Imported { namespace: ns, whole: true, recursive: *recursive, member: None },
                (Some(member), false) => Imported { namespace: scope.flat[member].owner.unwrap_or(member), whole: false, recursive: false, member: Some(member) },
                (None, _) => continue,
            };
            match owner {
                Some(owner) => scope.imports[owner].push(imported),
                None => scope.root_imports.push(imported),
            }
        }
        // Aliases: another name for a member.
        let aliases: Vec<(usize, Vec<String>, QualifiedName)> = scope
            .flat
            .iter()
            .filter_map(|f| match &f.node.detail {
                Detail::Alias { target } => {
                    let names: Vec<String> = [f.node.name.clone(), f.node.short_name.clone()].into_iter().flatten().collect();
                    Some((f.owner?, names, target.clone()))
                }
                _ => None,
            })
            .collect();
        for (owner, names, target) in aliases {
            if let Some(found) = scope.resolve_name(&target, Some(owner)) {
                for name in names {
                    scope.members[owner].entry(name).or_insert(found);
                }
            }
        }
        scope
    }

    pub fn len(&self) -> usize {
        self.flat.len()
    }

    pub fn node(&self, index: usize) -> &'n Node {
        self.flat[index].node
    }

    pub fn owner(&self, index: usize) -> Option<usize> {
        self.flat[index].owner
    }

    pub fn children(&self, index: usize) -> &[usize] {
        &self.flat[index].children
    }

    /// The qualified name of an element: the names of its owners and its own.
    pub fn qualified_name(&self, index: usize) -> Option<String> {
        let mut names = Vec::new();
        let mut current = Some(index);
        while let Some(i) = current {
            let node = self.flat[i].node;
            let name = node.name.clone().or_else(|| node.short_name.clone())?;
            names.push(super::syntax::quote(&name));
            current = self.flat[i].owner;
        }
        names.reverse();
        Some(names.join("::"))
    }

    // --- members ----------------------------------------------------------

    /// A member of `namespace` by name: declared, inherited, or imported.
    pub fn member(&self, namespace: usize, name: &str) -> Option<usize> {
        let mut seen = HashSet::new();
        self.member_in(namespace, name, &mut seen, 0, true)
    }

    fn member_in(&self, namespace: usize, name: &str, seen: &mut HashSet<usize>, depth: usize, use_imports: bool) -> Option<usize> {
        if depth > 10 || !seen.insert(namespace) {
            return None;
        }
        if let Some(&found) = self.members[namespace].get(name) {
            return Some(found);
        }
        // Inherited: members of the generals, types and subsetted features.
        for general in self.generals(namespace) {
            if let Some(found) = self.member_in(general, name, seen, depth + 1, use_imports) {
                return Some(found);
            }
        }
        if use_imports {
            if let Some(found) = self.imported_member(&self.imports[namespace], name, seen, depth) {
                return Some(found);
            }
        }
        None
    }

    fn imported_member(&self, imports: &[Imported], name: &str, seen: &mut HashSet<usize>, depth: usize) -> Option<usize> {
        for import in imports {
            if import.whole {
                if let Some(&found) = self.members[import.namespace].get(name) {
                    return Some(found);
                }
                if let Some(found) = self.member_in(import.namespace, name, seen, depth + 1, false) {
                    return Some(found);
                }
                if import.recursive {
                    if let Some(found) = self.member_recursive(import.namespace, name, depth + 1) {
                        return Some(found);
                    }
                }
            } else if let Some(member) = import.member {
                let node = self.flat[member].node;
                if node.name.as_deref() == Some(name) || node.short_name.as_deref() == Some(name) {
                    return Some(member);
                }
            }
        }
        None
    }

    /// A member anywhere under `namespace` (`import P::**`).
    fn member_recursive(&self, namespace: usize, name: &str, depth: usize) -> Option<usize> {
        if depth > 10 {
            return None;
        }
        for child in &self.flat[namespace].children {
            if let Some(&found) = self.members[*child].get(name) {
                return Some(found);
            }
            if let Some(found) = self.member_recursive(*child, name, depth + 1) {
                return Some(found);
            }
        }
        None
    }

    /// What an element specializes, is typed by, subsets or redefines, as
    /// resolved elements: where its inherited members come from.
    fn generals(&self, index: usize) -> Vec<usize> {
        if let Some(cached) = self.generals_cache.borrow().get(&index) {
            return cached.clone();
        }
        // While they are being resolved, an element has no generals: a
        // cycle of specializations ends here.
        self.generals_cache.borrow_mut().insert(index, Vec::new());
        let node = self.flat[index].node;
        let context = self.flat[index].owner;
        let mut out = Vec::new();
        for specialization in &node.specializations {
            let found = match &specialization.target {
                Target::Name(name) => self.resolve_name_from(name, context, &mut HashSet::new(), 4, true),
                Target::Chain(steps) => self.resolve_chain(steps, context).and_then(|r| r.last().copied()),
            };
            if let Some(found) = found {
                if found != index && !out.contains(&found) {
                    out.push(found);
                }
                // A subsetted or redefined feature brings its own types along.
                if specialization.kind != SpecKind::Typing {
                    for ty in self.generals(found).into_iter().filter(|&g| self.is_typing_general(found, g)) {
                        if !out.contains(&ty) {
                            out.push(ty);
                        }
                    }
                }
            }
        }
        self.generals_cache.borrow_mut().insert(index, out.clone());
        out
    }

    /// Whether `general` is a type of `index` (not a subsetted feature).
    fn is_typing_general(&self, index: usize, general: usize) -> bool {
        let kind = self.flat[general].node.kind;
        let _ = index;
        kind.ends_with("Definition") || kind == "Package"
    }

    // --- names ------------------------------------------------------------

    /// Resolve a qualified name written inside `context` (`None`: at the root).
    pub fn resolve_name(&self, name: &QualifiedName, context: Option<usize>) -> Option<usize> {
        self.resolve_name_from(name, context, &mut HashSet::new(), 0, true)
    }

    fn resolve_without_imports(&self, name: &QualifiedName, context: Option<usize>) -> Option<usize> {
        self.resolve_name_from(name, context, &mut HashSet::new(), 0, false)
    }

    fn resolve_name_from(&self, name: &QualifiedName, context: Option<usize>, seen: &mut HashSet<usize>, depth: usize, use_imports: bool) -> Option<usize> {
        if depth > 10 {
            return None;
        }
        let first = name.segments.first()?;
        let mut found = if name.global { self.root_member(first) } else { self.lookup_outward(first, context, depth, use_imports) }?;
        for segment in &name.segments[1..] {
            let mut inner = HashSet::new();
            found = self.member_in(found, segment, &mut inner, depth + 1, use_imports)?;
        }
        let _ = seen;
        Some(found)
    }

    fn root_member(&self, name: &str) -> Option<usize> {
        self.roots.iter().copied().find(|&r| {
            let node = self.flat[r].node;
            node.name.as_deref() == Some(name) || node.short_name.as_deref() == Some(name)
        })
    }

    /// The first segment: in the context, its generals and imports, then outward.
    fn lookup_outward(&self, name: &str, context: Option<usize>, depth: usize, use_imports: bool) -> Option<usize> {
        let mut current = context;
        let mut hops = 0;
        while let Some(namespace) = current {
            let mut seen = HashSet::new();
            if let Some(found) = self.member_in(namespace, name, &mut seen, depth, use_imports) {
                return Some(found);
            }
            // The namespace itself, by its own name (`Vehicle::x` from inside Vehicle).
            let node = self.flat[namespace].node;
            if node.name.as_deref() == Some(name) || node.short_name.as_deref() == Some(name) {
                return Some(namespace);
            }
            current = self.flat[namespace].owner;
            hops += 1;
            if hops > 64 {
                break;
            }
        }
        // At the root: top-level elements and the imports written there.
        if let Some(found) = self.root_member(name) {
            return Some(found);
        }
        if use_imports {
            let mut seen = HashSet::new();
            return self.imported_member(&self.root_imports, name, &mut seen, depth);
        }
        None
    }

    /// Resolve a feature chain: each step a member of the previous feature
    /// (through its types). All steps, when every one resolves.
    pub fn resolve_chain(&self, steps: &[QualifiedName], context: Option<usize>) -> Option<Vec<usize>> {
        let mut out = Vec::with_capacity(steps.len());
        let first = self.resolve_name(steps.first()?, context)?;
        out.push(first);
        for step in &steps[1..] {
            let previous = *out.last()?;
            let found = if step.segments.len() == 1 { self.member(previous, step.last()) } else { self.resolve_name(step, Some(previous)) }?;
            out.push(found);
        }
        Some(out)
    }

    /// Resolve a target: a name or a chain; the designated element, and the
    /// chain's elements when it is one.
    pub fn resolve_target(&self, target: &Target, context: Option<usize>) -> Option<Vec<usize>> {
        match target {
            Target::Name(name) => self.resolve_name(name, context).map(|i| vec![i]),
            Target::Chain(steps) => self.resolve_chain(steps, context),
        }
    }
}
