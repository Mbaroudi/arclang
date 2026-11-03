//! A Capella model as EMF holds it, for the m2m: every element of the
//! `.capella` (and of the `.aird`, whose diagrams give the state machines their
//! geometry), its class, its features, its container, its contents in order,
//! its references resolved, and the references made to it (EMF's inverse
//! cross references, in the order the documents make them).
//!
//! The classes are Capella's own (`classes.rs`), so `is` answers as the Java
//! `instanceof` does.

use super::super::super::capella_xmi::{self, Node};
use super::classes::CLASSES;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::OnceLock;

/// An element of a document.
#[derive(Debug, Clone, Default)]
pub struct Element {
    /// The class: `PhysicalFunction`, `DSemanticDiagram`, `Node`…
    pub class: String,
    /// The feature that contains it: `ownedFunctions`, `children`…
    pub feature: String,
    pub attrs: BTreeMap<String, String>,
    pub container: Option<usize>,
    pub contents: Vec<usize>,
    pub text: String,
    /// The document it was read from.
    pub resource: usize,
}

/// The documents read, their elements in an arena.
#[derive(Debug, Default)]
pub struct Model {
    pub elements: Vec<Element>,
    /// Each document: its file name and its roots.
    pub resources: Vec<(String, Vec<usize>)>,
    ids: HashMap<(usize, String), usize>,
    /// (target, feature) -> the elements that refer to it by that feature, in document order.
    inverse: HashMap<(usize, String), Vec<usize>>,
    /// Reference features written as child elements (`<target href=…/>`): element -> (feature, target).
    child_refs: HashMap<usize, Vec<(String, usize)>>,
}

fn hierarchy() -> &'static HashMap<&'static str, &'static [&'static str]> {
    static TABLE: OnceLock<HashMap<&'static str, &'static [&'static str]>> = OnceLock::new();
    TABLE.get_or_init(|| CLASSES.iter().copied().collect())
}

/// The direct supertypes of a class, in the metamodel's order.
pub fn supertypes(class: &str) -> &'static [&'static str] {
    hierarchy().get(class).copied().unwrap_or(&[])
}

/// Whether `class` is `ancestor` or one of its subclasses.
pub fn is_a(class: &str, ancestor: &str) -> bool {
    class == ancestor || supertypes(class).iter().any(|s| is_a(s, ancestor))
}

fn short_class(name: &str) -> String {
    name.rsplit(':').next().unwrap_or(name).to_string()
}

impl Model {
    /// Reads a `.capella` file and, when it is there, the `.aird` beside it.
    pub fn load(capella: &Path) -> Result<Model, String> {
        let mut model = Model::default();
        let name = capella.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let text = std::fs::read_to_string(capella).map_err(|e| format!("{}: {e}", capella.display()))?;
        model.read(&name, &text)?;
        let aird = capella.with_extension("aird");
        if let Ok(text) = std::fs::read_to_string(&aird) {
            model.read(&aird.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(), &text)?;
        }
        model.link();
        Ok(model)
    }

    /// Reads documents given as text: (file name, content), the semantic model first.
    pub fn from_strings(documents: &[(&str, &str)]) -> Result<Model, String> {
        let mut model = Model::default();
        for (name, text) in documents {
            model.read(name, text)?;
        }
        model.link();
        Ok(model)
    }

    fn read(&mut self, name: &str, text: &str) -> Result<(), String> {
        let document = capella_xmi::parse(text)?;
        let resource = self.resources.len();
        self.resources.push((name.to_string(), Vec::new()));
        let roots: Vec<&Node> = match document.children.first() {
            Some(root) if root.tag == "xmi:XMI" => root.children.iter().collect(),
            Some(root) => vec![root],
            None => Vec::new(),
        };
        for root in roots {
            let i = self.element(root, None, resource);
            self.resources[resource].1.push(i);
        }
        Ok(())
    }

    fn element(&mut self, node: &Node, container: Option<usize>, resource: usize) -> usize {
        let class = if node.kind.is_empty() { if container.is_none() { short_class(&node.tag) } else { String::new() } } else { short_class(&node.kind) };
        let me = self.elements.len();
        self.elements.push(Element { class, feature: node.tag.clone(), attrs: node.attrs.clone(), container, contents: Vec::new(), text: node.text.clone(), resource });
        for key in ["id", "uid", "xmi:id"] {
            if let Some(id) = node.attrs.get(key) {
                self.ids.insert((resource, id.clone()), me);
            }
        }
        for child in &node.children {
            let c = self.element(child, Some(me), resource);
            self.elements[me].contents.push(c);
        }
        me
    }

    fn resource_named(&self, file: &str) -> Option<usize> {
        let file = file.rsplit('/').next().unwrap_or(file);
        self.resources.iter().position(|(name, _)| name == file)
    }

    /// Resolves `#id`, `file#id` or a bare id, from the document `resource`.
    fn resolve(&self, resource: usize, token: &str) -> Option<usize> {
        let (file, id) = match token.split_once('#') {
            Some((file, id)) => (file, id),
            None => ("", token),
        };
        let resource = if file.is_empty() { resource } else { self.resource_named(file)? };
        self.ids.get(&(resource, id.to_string())).copied()
    }

    /// Every reference made by attribute or by `href`, resolved, and their inverses.
    fn link(&mut self) {
        for i in 0..self.elements.len() {
            let e = &self.elements[i];
            if let Some(href) = e.attrs.get("href") {
                if let (Some(container), Some(target)) = (e.container, self.resolve(e.resource, href)) {
                    let feature = e.feature.clone();
                    self.child_refs.entry(container).or_default().push((feature.clone(), target));
                    self.inverse.entry((target, feature)).or_default().push(container);
                }
            }
            let mut found = Vec::new();
            for (feature, value) in &e.attrs {
                if matches!(feature.as_str(), "id" | "uid" | "xmi:id" | "name" | "href") {
                    continue;
                }
                for token in reference_tokens(value) {
                    if let Some(target) = self.resolve(e.resource, token) {
                        found.push((target, feature.clone()));
                    }
                }
            }
            for key in found {
                self.inverse.entry(key).or_default().push(i);
            }
        }
    }

    // ---- Reading features --------------------------------------------------------

    pub fn class(&self, i: usize) -> &str {
        &self.elements[i].class
    }

    pub fn is(&self, i: usize, class: &str) -> bool {
        is_a(self.class(i), class)
    }

    pub fn attr(&self, i: usize, feature: &str) -> Option<&str> {
        self.elements[i].attrs.get(feature).map(String::as_str)
    }

    pub fn id(&self, i: usize) -> Option<String> {
        self.attr(i, "id").map(str::to_string)
    }

    pub fn name(&self, i: usize) -> Option<String> {
        self.attr(i, "name").map(str::to_string)
    }

    pub fn container(&self, i: usize) -> Option<usize> {
        self.elements[i].container
    }

    /// The elements contained by this feature, in order.
    pub fn owned(&self, i: usize, feature: &str) -> Vec<usize> {
        self.elements[i].contents.iter().copied().filter(|&c| self.elements[c].feature == feature && !self.elements[c].attrs.contains_key("href")).collect()
    }

    /// The elements a reference feature points to, in order.
    pub fn refs(&self, i: usize, feature: &str) -> Vec<usize> {
        let e = &self.elements[i];
        let mut out: Vec<usize> = e.attrs.get(feature).map(|v| reference_tokens(v).filter_map(|t| self.resolve(e.resource, t)).collect()).unwrap_or_default();
        if let Some(children) = self.child_refs.get(&i) {
            out.extend(children.iter().filter(|(f, _)| f == feature).map(|&(_, t)| t));
        }
        out
    }

    pub fn reference(&self, i: usize, feature: &str) -> Option<usize> {
        self.refs(i, feature).into_iter().next()
    }

    /// The elements that refer to this one by `feature`, in document order.
    pub fn referencers(&self, i: usize, feature: &str) -> Vec<usize> {
        self.inverse.get(&(i, feature.to_string())).cloned().unwrap_or_default()
    }

    /// The element of this document with this id.
    pub fn by_id(&self, id: &str) -> Option<usize> {
        (0..self.resources.len()).find_map(|r| self.ids.get(&(r, id.to_string())).copied())
    }

    /// The first container (or the element itself) of this class.
    pub fn ancestor(&self, i: usize, class: &str) -> Option<usize> {
        let mut at = Some(i);
        while let Some(e) = at {
            if self.is(e, class) {
                return Some(e);
            }
            at = self.container(e);
        }
        None
    }

    /// EMF's `EcoreUtil.isAncestor(a, b)`: `a` is `b` or contains it.
    pub fn is_ancestor(&self, a: usize, b: usize) -> bool {
        let mut at = Some(b);
        while let Some(e) = at {
            if e == a {
                return true;
            }
            at = self.container(e);
        }
        false
    }

    /// Capella's `EcoreUtil2.getCommonAncestor`.
    pub fn common_ancestor(&self, a: usize, b: usize) -> Option<usize> {
        if self.is_ancestor(a, b) {
            return Some(a);
        }
        if self.is_ancestor(b, a) {
            return Some(b);
        }
        let mut at = Some(a);
        while let Some(e) = at {
            if self.is_ancestor(e, b) {
                return Some(e);
            }
            at = self.container(e);
        }
        None
    }

    /// Every element of a document, depth first (EMF's `getAllContents`).
    pub fn all_contents(&self, i: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut stack: Vec<usize> = self.elements[i].contents.iter().rev().copied().collect();
        while let Some(e) = stack.pop() {
            out.push(e);
            stack.extend(self.elements[e].contents.iter().rev());
        }
        out
    }
}

/// The ids of a reference attribute: `#a #b`, `file#a`; a type written before a
/// reference to another file (`pkg:Kind file#id`) is not one.
fn reference_tokens(value: &str) -> impl Iterator<Item = &str> {
    value.split_whitespace().filter(|t| t.contains('#'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_are_those_of_capella() {
        assert!(is_a("PhysicalFunction", "AbstractFunction"));
        assert!(is_a("Mode", "State"));
        assert!(is_a("PhysicalQuantity", "NumericType"));
        assert!(is_a("FunctionalExchange", "AbstractEvent"));
        assert!(!is_a("BooleanType", "NumericType"));
    }

    #[test]
    fn references_and_their_inverses_are_resolved() {
        let capella = r##"<?xml version="1.0" encoding="UTF-8"?>
<org.polarsys.capella.core.data.capellamodeller:Project xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" id="p" name="pp">
  <ownedFunctions xsi:type="org.polarsys.capella.core.data.pa:PhysicalFunction" id="f" name="F">
    <inputs xsi:type="org.polarsys.capella.core.data.fa:FunctionInputPort" id="i"/>
    <outputs xsi:type="org.polarsys.capella.core.data.fa:FunctionOutputPort" id="o"/>
    <ownedFunctionalExchanges xsi:type="org.polarsys.capella.core.data.fa:FunctionalExchange" id="e" source="#o" target="#i"/>
  </ownedFunctions>
</org.polarsys.capella.core.data.capellamodeller:Project>"##;
        let m = Model::from_strings(&[("pp.capella", capella)]).unwrap();
        let f = m.by_id("f").unwrap();
        let e = m.by_id("e").unwrap();
        assert_eq!(m.class(f), "PhysicalFunction");
        assert_eq!(m.reference(e, "target"), m.by_id("i"));
        assert_eq!(m.referencers(m.by_id("i").unwrap(), "target"), vec![e]);
        assert_eq!(m.common_ancestor(m.by_id("i").unwrap(), m.by_id("o").unwrap()), Some(f));
    }
}
