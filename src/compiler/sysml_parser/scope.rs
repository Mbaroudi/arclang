//! Name resolution over the flattened syntax tree: what a qualified name
//! or a feature chain designates from where it is written. Members are
//! found in the namespace itself, in what it inherits (through its
//! specializations and typings, boundedly), in what it imports, then in the
//! enclosing namespaces up to the root; `$::` starts at the root.
//!
//! Imported namespaces are resolved once, when the scope is built: first
//! without consulting imports, then the remaining ones through the imports
//! already resolved (`import Parts::Wheel` where `Parts` is itself
//! imported). A public import is re-exported: what `P::*` brings includes
//! what `P` publicly imports.

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

/// The name an unnamed feature takes from the feature it redefines or
/// references: the last segment of that name or chain.
fn implicit_name(node: &Node) -> Option<&str> {
    if node.name.is_some() || node.short_name.is_some() {
        return None;
    }
    let specialization = node.specializations.iter().find(|s| matches!(s.kind, SpecKind::Redefinition | SpecKind::Reference))?;
    match &specialization.target {
        Target::Name(name) => name.segments.last().map(String::as_str),
        Target::Chain(steps) => steps.last()?.segments.last().map(String::as_str),
    }
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
    /// `public import`: re-exported to whoever imports the namespace.
    public: bool,
    /// For a membership import: the member itself.
    member: Option<usize>,
}

/// Passes over the imports: an import found through another import waits
/// for that one.
const IMPORT_PASSES: usize = 6;

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
        // Unnamed features are known by the name of what they redefine or
        // reference (`:>> mass`, `perform a.step`), after the declared names.
        for (index, element) in flat.iter().enumerate() {
            let Some(owner) = element.owner else { continue };
            if let Some(name) = implicit_name(element.node) {
                members[owner].entry(name.to_string()).or_insert(index);
            }
        }
        let mut scope = Scope { flat, members, imports: vec![Vec::new(); n], root_imports: Vec::new(), roots: root_indexes, generals_cache: RefCell::new(HashMap::new()) };
        // Imports: resolved outward from the importing namespace, first
        // without imports, then through the imports resolved so far.
        let mut pending: Vec<(usize, Option<usize>)> = scope.flat.iter().enumerate().filter(|(_, f)| matches!(f.node.detail, Detail::Import { .. })).map(|(i, f)| (i, f.owner)).collect();
        for pass in 0..IMPORT_PASSES {
            let before = pending.len();
            pending.retain(|&(index, owner)| !scope.resolve_import(index, owner, pass > 0));
            if pending.len() == before && pass > 0 {
                break;
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

    /// Resolve one import and record it; false when its target is not found.
    fn resolve_import(&mut self, index: usize, owner: Option<usize>, use_imports: bool) -> bool {
        let node = self.flat[index].node;
        let Detail::Import { target, namespace: whole, recursive, .. } = &node.detail else { return true };
        let found = if use_imports { self.resolve_name(target, owner) } else { self.resolve_without_imports(target, owner) };
        let Some(found) = found else { return false };
        let public = node.prefixes.visibility == Some("public");
        let imported = if *whole {
            Imported { namespace: found, whole: true, recursive: *recursive, public, member: None }
        } else {
            Imported { namespace: self.flat[found].owner.unwrap_or(found), whole: false, recursive: false, public, member: Some(found) }
        };
        // `P::**`: P itself, and everything under it.
        let under = (!*whole && *recursive).then_some(Imported { namespace: found, whole: true, recursive: true, public, member: None });
        let list = match owner {
            Some(owner) => &mut self.imports[owner],
            None => &mut self.root_imports,
        };
        list.push(imported);
        list.extend(under);
        true
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
        if depth > 10 {
            return None;
        }
        for import in imports {
            if import.whole {
                if let Some(&found) = self.members[import.namespace].get(name) {
                    return Some(found);
                }
                if let Some(found) = self.member_in(import.namespace, name, seen, depth + 1, false) {
                    return Some(found);
                }
                // What the imported namespace publicly imports comes along,
                // once per lookup: imports that cycle or form diamonds are
                // followed once (keys past `len` mark followed namespaces).
                let followed = self.flat.len() + import.namespace;
                let public: Vec<Imported> = self.imports[import.namespace].iter().copied().filter(|i| i.public).collect();
                if !public.is_empty() && seen.insert(followed) {
                    if let Some(found) = self.imported_member(&public, name, seen, depth + 1) {
                        return Some(found);
                    }
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
        let mut out = Vec::new();
        for specialization in &node.specializations {
            // Only a feature known by its implicit name can find itself: a
            // typing or a redefinition is looked up from the owner.
            let context = if matches!(specialization.kind, SpecKind::Redefinition | SpecKind::Typing) { self.flat[index].owner } else { self.specialization_context(index, &specialization.target) };
            let found = match &specialization.target {
                Target::Name(name) if specialization.kind == SpecKind::Redefinition && name.segments.len() == 1 => {
                    self.redefined(index, name.last()).or_else(|| self.resolve_name_from(name, context, &mut HashSet::new(), 4, true))
                }
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

    /// The feature a redefinition `:>> name` of `index` redefines: a member
    /// its owner inherits, never `index` itself (known by that same name).
    pub fn redefined(&self, index: usize, name: &str) -> Option<usize> {
        let owner = self.flat[index].owner?;
        for general in self.generals(owner) {
            let mut seen = HashSet::new();
            if let Some(found) = self.member_in(general, name, &mut seen, 1, true).filter(|&f| f != index) {
                return Some(found);
            }
        }
        None
    }

    /// Where the target of a specialization of `index` is resolved from: its
    /// owner, or further out when the name found there is `index` itself
    /// (`perform massTests;` is known as `massTests` by its implicit name).
    pub fn specialization_context(&self, index: usize, target: &Target) -> Option<usize> {
        let mut context = self.flat[index].owner;
        for _ in 0..64 {
            let found = match target {
                Target::Name(name) => self.resolve_name(name, context),
                Target::Chain(steps) => steps.first().and_then(|first| self.resolve_name(first, context)),
            };
            if found != Some(index) {
                return context;
            }
            context = self.flat[context?].owner;
        }
        context
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
