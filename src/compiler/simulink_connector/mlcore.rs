//! Writes a pivot as the `.mlcore` file the connector saves (EMF XMI): the
//! project and its dictionaries as roots, each element with its id, its Capella
//! id and its parameters, a reference to the built-in types by `href` into the
//! connector's library. What is not written is what EMF does not save: the
//! default values, and the ports a model reference derives from its model
//! (lines name them `derived-<reference>--<port>`, as `pivot::read` expects).
//!
//! An element made without an id gets one: a UUID (version 5) of its place in
//! the document, so that the same pivot is always written the same.

use super::pivot::{Kind, Obj, Pivot, Value};
use std::collections::{HashMap, HashSet};

const CORE: &str = "org.eclipse.capella.simulink.core";
const INFRA: &str = "org.eclipse.capella.simulink.infra";
const LIBRARY: &str = "library:/model/builtInTypes.mlcore";

fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\n' => out.push_str("&#xA;"),
            '\r' => out.push_str("&#xD;"),
            '\t' => out.push_str("&#x9;"),
            c => out.push(c),
        }
    }
    out
}

struct Writer<'p> {
    p: &'p Pivot,
    /// The elements written in this document.
    local: HashSet<usize>,
    ids: HashMap<usize, String>,
    out: String,
}

/// Every element written, its contents after it (a model reference's ports are not).
fn document(p: &Pivot) -> Vec<usize> {
    fn walk(p: &Pivot, i: usize, out: &mut Vec<usize>) {
        out.push(i);
        for c in super::pivot::contents(p, i) {
            if !(p[i].kind == Kind::SystemReference && p[i].ports.contains(&c)) {
                walk(p, c, out);
            }
        }
    }
    let mut out = Vec::new();
    for &root in &p.roots {
        walk(p, root, &mut out);
    }
    out
}

/// The built-in type an element of the library is, by its id.
fn is_library(p: &Pivot, i: usize, local: &HashSet<usize>) -> bool {
    !local.contains(&i) && p[i].id.is_some()
}

/// Writes the pivot.
pub fn write(p: &Pivot) -> String {
    let order = document(p);
    let local: HashSet<usize> = order.iter().copied().collect();
    let mut ids = HashMap::new();
    let mut seen = HashSet::new();
    let mut path: HashMap<usize, String> = HashMap::new();
    for &i in &order {
        let place = match p[i].container {
            Some(c) => format!("{}/{}", path.get(&c).cloned().unwrap_or_default(), position(p, c, i)),
            None => format!("{}", p.roots.iter().position(|&r| r == i).unwrap_or(0)),
        };
        path.insert(i, place.clone());
        let id = match &p[i].id {
            Some(id) if !id.contains("null") && seen.insert(id.clone()) => id.clone(),
            _ => {
                let seed = format!("{place}:{}:{}", p[i].kind.class(), p[i].originator.clone().unwrap_or_default());
                uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, seed.as_bytes()).to_string()
            }
        };
        ids.insert(i, id);
    }
    let mut w = Writer { p, local, ids, out: String::new() };
    w.out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let namespaces = format!("xmi:version=\"2.0\" xmlns:xmi=\"http://www.omg.org/XMI\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:{CORE}=\"http://www.eclipse.org/capella/simulink/core/1.0\" xmlns:{INFRA}=\"http://www.eclipse.org/capella/simulink/infra/1.0\"");
    if p.roots.len() == 1 {
        let root = p.roots[0];
        w.element(root, &format!("{CORE}:{}", p[root].kind.class()), None, 0, Some(&namespaces));
    } else {
        w.out.push_str(&format!("<xmi:XMI {namespaces}>\n"));
        for &root in &p.roots {
            w.element(root, &format!("{CORE}:{}", p[root].kind.class()), None, 1, None);
        }
        w.out.push_str("</xmi:XMI>\n");
    }
    w.out
}

/// The index of a child among its container's contents.
fn position(p: &Pivot, container: usize, child: usize) -> usize {
    super::pivot::contents(p, container).iter().position(|&c| c == child).unwrap_or(0)
}

impl Writer<'_> {
    fn id(&self, i: usize) -> String {
        self.ids.get(&i).cloned().unwrap_or_default()
    }

    /// How a reference to an element of this document is written: its id; a model reference's
    /// derived port as `pivot::read` derives it.
    fn reference(&self, i: usize) -> Option<String> {
        if self.local.contains(&i) {
            return Some(self.id(i));
        }
        let o = &self.p[i];
        let r = o.container?;
        if self.p[r].kind == Kind::SystemReference {
            let k = self.p[r].ports.iter().position(|&d| d == i)?;
            let system = self.p[r].system?;
            let source = *self.p[system].ports.get(k)?;
            return Some(format!("derived-{}--{}", self.id(r), self.id(source)));
        }
        None
    }

    fn element(&mut self, i: usize, tag: &str, xsi: Option<&str>, depth: usize, namespaces: Option<&str>) {
        let o: &Obj = &self.p[i];
        let pad = "  ".repeat(depth);
        let mut attrs: Vec<(String, String)> = Vec::new();
        if let Some(ns) = namespaces {
            attrs.push((String::new(), ns.to_string()));
        }
        if let Some(t) = xsi {
            attrs.push(("xsi:type".to_string(), format!("{CORE}:{t}")));
        }
        attrs.push(("id".to_string(), self.id(i)));
        let mut text = |k: &str, v: &Option<String>| {
            if let Some(v) = v {
                attrs.push((k.to_string(), escape(v)));
            }
        };
        text("originatorId", &o.originator);
        text("name", &o.name);
        text("description", &o.description);
        let mut children_href: Vec<(String, String, String)> = Vec::new();
        let mut refs: Vec<(&str, Vec<usize>)> = Vec::new();
        match o.kind {
            Kind::Project => refs.push(("dictionaries", o.dictionaries.clone())),
            Kind::System => {
                if let Some(f) = &o.file_name {
                    attrs.push(("fileName".to_string(), escape(f)));
                }
                if o.is_library {
                    attrs.push(("isLibrary".to_string(), "true".to_string()));
                }
                refs.push(("dictionary", o.dictionary.into_iter().collect()));
            }
            Kind::Dictionary => {
                if let Some(f) = &o.file_name {
                    attrs.push(("fileName".to_string(), escape(f)));
                }
                refs.push(("subDictionaries", o.sub_dictionaries.iter().copied().filter(|d| self.local.contains(d)).collect()));
            }
            Kind::Line => {
                refs.push(("source", o.line_source.into_iter().collect()));
                refs.push(("target", o.line_target.into_iter().collect()));
            }
            Kind::Transition => {
                for (k, v) in [("event", &o.event), ("condition", &o.condition), ("action", &o.action), ("comment", &o.comment)] {
                    if let Some(v) = v {
                        attrs.push((k.to_string(), escape(v)));
                    }
                }
                refs.push(("destination", o.destination.into_iter().collect()));
                refs.push(("source", o.state_source.into_iter().collect()));
            }
            Kind::State => {
                for (k, v) in [("entry", &o.entry), ("during", &o.during), ("exit", &o.exit)] {
                    if let Some(v) = v {
                        attrs.push((k.to_string(), escape(v)));
                    }
                }
                if o.decomposition != "EXCLUSIVE_OR" {
                    attrs.push(("type".to_string(), o.decomposition.clone()));
                }
            }
            Kind::Junction => {
                if o.junction != "CONNECTIVE" {
                    attrs.push(("type".to_string(), o.junction.clone()));
                }
            }
            Kind::Port => {
                if o.port_kind != "Inport" {
                    attrs.push(("kind".to_string(), o.port_kind.clone()));
                }
                if o.index != 1 {
                    attrs.push(("index".to_string(), o.index.to_string()));
                }
            }
            Kind::Literal => {
                if o.value != 0 {
                    attrs.push(("value".to_string(), o.value.to_string()));
                }
            }
            Kind::Signal => {
                if let Some(v) = &o.initial_value {
                    attrs.push(("initialValue".to_string(), escape(v)));
                }
                if let Some(v) = &o.unit {
                    attrs.push(("unit".to_string(), escape(v)));
                }
            }
            k if k.is_symbol() => {
                if o.scope != "Input" {
                    attrs.push(("scope".to_string(), o.scope.clone()));
                }
                refs.push(("underlyingPort", o.underlying_port.into_iter().collect()));
            }
            _ => {}
        }
        if o.kind.is_block() {
            if let Some(s) = &o.source {
                attrs.push(("source".to_string(), escape(s)));
            }
        }
        if o.kind == Kind::SystemReference {
            refs.push(("system", o.system.into_iter().collect()));
        }
        if o.kind.holds_datatype() {
            if let Some(d) = o.datatype {
                if is_library(self.p, d, &self.local) {
                    let class = self.p[d].kind.class().to_string();
                    children_href.push(("datatype".to_string(), class, format!("{LIBRARY}#{}", self.p[d].id.clone().unwrap_or_default())));
                } else {
                    refs.push(("datatype", vec![d]));
                }
            }
        }
        for (feature, targets) in refs {
            let written: Vec<String> = targets.iter().filter_map(|&t| self.reference(t)).collect();
            if !written.is_empty() {
                attrs.push((feature.to_string(), written.join(" ")));
            }
        }
        let head: Vec<String> = attrs.iter().map(|(k, v)| if k.is_empty() { v.clone() } else { format!("{k}=\"{v}\"") }).collect();
        let children = self.children(i);
        if children.is_empty() && o.params.is_empty() && children_href.is_empty() {
            self.out.push_str(&format!("{pad}<{tag} {}/>\n", head.join(" ")));
            return;
        }
        self.out.push_str(&format!("{pad}<{tag} {}>\n", head.join(" ")));
        let params = o.params.clone();
        for (n, (key, value)) in params.iter().enumerate() {
            let pid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, format!("{}:parameter:{n}", self.id(i)).as_bytes());
            self.out.push_str(&format!("{pad}  <parameters id=\"{pid}\" key=\"{}\">\n", escape(key)));
            self.value(value, "value", &format!("{pid}"), depth + 2);
            self.out.push_str(&format!("{pad}  </parameters>\n"));
        }
        for (feature, class, href) in children_href {
            self.out.push_str(&format!("{pad}  <{feature} xsi:type=\"{CORE}:{class}\" href=\"{href}\"/>\n"));
        }
        for (feature, child) in children {
            let class = self.p[child].kind.class();
            let xsi = match (feature, self.p[child].kind) {
                ("systems", Kind::System) | ("ports", Kind::Port) | ("lines", Kind::Line) | ("transitions", Kind::Transition) | ("busElements", Kind::BusElement) | ("literals", Kind::Literal) => None,
                _ => Some(class),
            };
            self.element(child, feature, xsi, depth + 1, None);
        }
        self.out.push_str(&format!("{pad}</{tag}>\n"));
    }

    fn value(&mut self, value: &Value, tag: &str, seed: &str, depth: usize) {
        let pad = "  ".repeat(depth);
        let vid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, format!("{seed}:{tag}").as_bytes());
        match value {
            Value::Primitive(spec) => {
                let spec = spec.as_ref().map(|s| format!(" specification=\"{}\"", escape(s))).unwrap_or_default();
                self.out.push_str(&format!("{pad}<{tag} xsi:type=\"{INFRA}:PrimitiveValue\" id=\"{vid}\"{spec}/>\n"));
            }
            Value::Multiple(values) => {
                self.out.push_str(&format!("{pad}<{tag} xsi:type=\"{INFRA}:MultipleValue\" id=\"{vid}\">\n"));
                for (n, v) in values.iter().enumerate() {
                    self.value(v, "values", &format!("{vid}:{n}"), depth + 1);
                }
                self.out.push_str(&format!("{pad}</{tag}>\n"));
            }
            Value::Other => {}
        }
    }

    /// The contained elements to write, with their feature, in the metamodel's order; a model
    /// reference's ports are derived, not saved.
    fn children(&self, i: usize) -> Vec<(&'static str, usize)> {
        let o = &self.p[i];
        let mut out: Vec<(&'static str, usize)> = Vec::new();
        if o.kind != Kind::SystemReference {
            out.extend(o.ports.iter().map(|&c| ("ports", c)));
        }
        out.extend(o.blocks.iter().map(|&c| ("blocks", c)));
        out.extend(o.lines.iter().map(|&c| ("lines", c)));
        out.extend(o.transitions.iter().map(|&c| ("transitions", c)));
        out.extend(o.states.iter().map(|&c| ("states", c)));
        out.extend(o.symbols.iter().map(|&c| ("symbols", c)));
        out.extend(o.sub_states.iter().map(|&c| ("subStates", c)));
        out.extend(o.systems.iter().map(|&c| ("systems", c)));
        out.extend(o.bus_elements.iter().map(|&c| ("busElements", c)));
        out.extend(o.datatypes.iter().map(|&c| ("datatypes", c)));
        out.extend(o.literals.iter().map(|&c| ("literals", c)));
        out
    }
}
