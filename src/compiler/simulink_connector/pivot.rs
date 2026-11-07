//! The connector's pivot model (`.mlcore`): what Capella hands to MATLAB.
//! A project, its Simulink systems (blocks, ports, lines, Stateflow charts)
//! and its data dictionaries (buses, signals, enumerations, types), each
//! element with the Capella id it comes from (`originatorId`) and its
//! parameters (`bounds`, `capella.layer`, MATLAB parameters).
//!
//! The model is an arena: an element is an index. A reference to a file that
//! cannot be read stays a proxy and is reported as EMF reports it.

use super::super::capella_xmi::{self, Node};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The library of MATLAB's built-in types the connector ships (`library:/model/builtInTypes.mlcore`).
pub const BUILT_IN_TYPES: &str = include_str!("builtInTypes.mlcore");
const LIBRARY: &str = "library:/model/builtInTypes.mlcore";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Project,
    System,
    Subsystem,
    Block,
    SystemReference,
    Chart,
    Line,
    Port,
    State,
    Junction,
    Transition,
    Data,
    Message,
    Event,
    Dictionary,
    Bus,
    BusElement,
    BuiltIntype,
    CustomType,
    Enumeration,
    Literal,
    Signal,
    /// A reference that could not be resolved; its class is in `name`.
    Proxy,
}

impl Kind {
    pub fn of(class: &str) -> Option<Kind> {
        Some(match class {
            "Project" => Kind::Project,
            "System" => Kind::System,
            "Subsystem" => Kind::Subsystem,
            "Block" => Kind::Block,
            "SystemReference" => Kind::SystemReference,
            "Chart" => Kind::Chart,
            "Line" => Kind::Line,
            "Port" => Kind::Port,
            "State" => Kind::State,
            "Junction" => Kind::Junction,
            "Transition" => Kind::Transition,
            "Data" => Kind::Data,
            "Message" => Kind::Message,
            "Event" => Kind::Event,
            "Dictionary" => Kind::Dictionary,
            "Bus" => Kind::Bus,
            "BusElement" => Kind::BusElement,
            "BuiltIntype" => Kind::BuiltIntype,
            "CustomType" => Kind::CustomType,
            "Enumeration" => Kind::Enumeration,
            "Literal" => Kind::Literal,
            "Signal" => Kind::Signal,
            _ => return None,
        })
    }

    pub fn class(self) -> &'static str {
        match self {
            Kind::Project => "Project",
            Kind::System => "System",
            Kind::Subsystem => "Subsystem",
            Kind::Block => "Block",
            Kind::SystemReference => "SystemReference",
            Kind::Chart => "Chart",
            Kind::Line => "Line",
            Kind::Port => "Port",
            Kind::State => "State",
            Kind::Junction => "Junction",
            Kind::Transition => "Transition",
            Kind::Data => "Data",
            Kind::Message => "Message",
            Kind::Event => "Event",
            Kind::Dictionary => "Dictionary",
            Kind::Bus => "Bus",
            Kind::BusElement => "BusElement",
            Kind::BuiltIntype => "BuiltIntype",
            Kind::CustomType => "CustomType",
            Kind::Enumeration => "Enumeration",
            Kind::Literal => "Literal",
            Kind::Signal => "Signal",
            Kind::Proxy => "Proxy",
        }
    }

    /// A Simulink diagram: it holds blocks and lines.
    pub fn is_diagram(self) -> bool {
        matches!(self, Kind::System | Kind::Subsystem)
    }

    pub fn is_block(self) -> bool {
        matches!(self, Kind::Block | Kind::Subsystem | Kind::SystemReference | Kind::Chart)
    }

    pub fn is_datatype(self) -> bool {
        matches!(self, Kind::Bus | Kind::BuiltIntype | Kind::CustomType | Kind::Enumeration | Kind::Signal)
    }

    /// The classes that hold a `datatype`.
    pub fn holds_datatype(self) -> bool {
        self.is_block() || matches!(self, Kind::Port | Kind::BusElement | Kind::Signal)
    }

    pub fn is_symbol(self) -> bool {
        matches!(self, Kind::Data | Kind::Message | Kind::Event)
    }
}

/// A parameter's value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Primitive(Option<String>),
    Multiple(Vec<Value>),
    /// A reference to an element: no MATLAB value.
    Other,
}

/// An element of the pivot. The features of every class are here; each class uses its own.
#[derive(Debug, Clone)]
pub struct Obj {
    pub kind: Kind,
    pub id: Option<String>,
    pub originator: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    /// In the order written (an EMF map keeps its order).
    pub params: Vec<(String, Value)>,
    pub container: Option<usize>,
    // Containment.
    pub systems: Vec<usize>,
    pub blocks: Vec<usize>,
    pub lines: Vec<usize>,
    pub ports: Vec<usize>,
    pub referenced_ports: Vec<usize>,
    pub datatypes: Vec<usize>,
    pub bus_elements: Vec<usize>,
    pub literals: Vec<usize>,
    pub transitions: Vec<usize>,
    pub states: Vec<usize>,
    pub sub_states: Vec<usize>,
    pub symbols: Vec<usize>,
    // References.
    pub dictionaries: Vec<usize>,
    pub sub_dictionaries: Vec<usize>,
    pub dictionary: Option<usize>,
    pub datatype: Option<usize>,
    pub system: Option<usize>,
    pub line_source: Option<usize>,
    pub line_target: Option<usize>,
    pub underlying_port: Option<usize>,
    pub state_source: Option<usize>,
    pub destination: Option<usize>,
    // Attributes.
    /// A block's library path (`sflib/Chart`).
    pub source: Option<String>,
    pub file_name: Option<String>,
    pub is_library: bool,
    /// A port's kind: `Inport`, `Outport`, `Enable`…
    pub port_kind: String,
    pub index: i32,
    pub scope: String,
    /// A state's decomposition, a junction's type.
    pub decomposition: String,
    pub junction: String,
    pub event: Option<String>,
    pub condition: Option<String>,
    pub action: Option<String>,
    pub comment: Option<String>,
    pub entry: Option<String>,
    pub during: Option<String>,
    pub exit: Option<String>,
    pub value: i32,
    pub initial_value: Option<String>,
    pub unit: Option<String>,
}

impl Obj {
    pub fn new(kind: Kind) -> Obj {
        Obj {
            kind,
            id: None,
            originator: None,
            name: None,
            description: None,
            params: Vec::new(),
            container: None,
            systems: Vec::new(),
            blocks: Vec::new(),
            lines: Vec::new(),
            ports: Vec::new(),
            referenced_ports: Vec::new(),
            datatypes: Vec::new(),
            bus_elements: Vec::new(),
            literals: Vec::new(),
            transitions: Vec::new(),
            states: Vec::new(),
            sub_states: Vec::new(),
            symbols: Vec::new(),
            dictionaries: Vec::new(),
            sub_dictionaries: Vec::new(),
            dictionary: None,
            datatype: None,
            system: None,
            line_source: None,
            line_target: None,
            underlying_port: None,
            state_source: None,
            destination: None,
            source: None,
            file_name: None,
            is_library: false,
            port_kind: "Inport".to_string(),
            index: 1,
            scope: "Input".to_string(),
            decomposition: "EXCLUSIVE_OR".to_string(),
            junction: "CONNECTIVE".to_string(),
            event: None,
            condition: None,
            action: None,
            comment: None,
            entry: None,
            during: None,
            exit: None,
            value: 0,
            initial_value: None,
            unit: None,
        }
    }

    pub fn param(&self, key: &str) -> Option<&Value> {
        self.params.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Puts a parameter: replaced where it is, else added at the end (an EMF map).
    pub fn set_param(&mut self, key: &str, value: Value) {
        match self.params.iter_mut().find(|(k, _)| k == key) {
            Some(entry) => entry.1 = value,
            None => self.params.push((key.to_string(), value)),
        }
    }
}

/// Elements, and the roots of the document the generation starts from.
#[derive(Debug, Clone, Default)]
pub struct Pivot {
    pub objs: Vec<Obj>,
    pub roots: Vec<usize>,
}

impl Pivot {
    pub fn add(&mut self, obj: Obj) -> usize {
        self.objs.push(obj);
        self.objs.len() - 1
    }
}

impl std::ops::Index<usize> for Pivot {
    type Output = Obj;
    fn index(&self, i: usize) -> &Obj {
        &self.objs[i]
    }
}

impl std::ops::IndexMut<usize> for Pivot {
    fn index_mut(&mut self, i: usize) -> &mut Obj {
        &mut self.objs[i]
    }
}

// ---- Reading ----------------------------------------------------------------------

/// The class of a contained element written without `xsi:type`: the type of its feature.
fn default_class(feature: &str) -> &'static str {
    match feature {
        "systems" => "System",
        "blocks" => "Block",
        "lines" => "Line",
        "ports" | "referencedPorts" => "Port",
        "transitions" => "Transition",
        "busElements" => "BusElement",
        "literals" => "Literal",
        _ => "",
    }
}

/// The class a reference feature points to (a proxy's class).
fn reference_class(feature: &str) -> &'static str {
    match feature {
        "dictionaries" | "subDictionaries" | "dictionary" => "Dictionary",
        "datatype" => "Datatype",
        "system" => "System",
        "source" | "target" | "underlyingPort" => "Port",
        "destination" => "AState",
        _ => "EObject",
    }
}

struct Loader {
    pivot: Pivot,
    /// Resources read: their ids.
    resources: HashMap<String, HashMap<String, usize>>,
    /// References to resolve once every resource is read: (element, feature, resource, id, class).
    pending: Vec<(usize, String, String, String, String)>,
    /// References to a model reference's derived port, resolved once those ports are made.
    derived: Vec<(usize, String, String, String, String)>,
    /// The resource each element was read from.
    resource_of: HashMap<usize, String>,
    base: PathBuf,
}

fn value_of(node: &Node) -> Value {
    match node.kind.rsplit(':').next().unwrap_or("") {
        "PrimitiveValue" => Value::Primitive(node.attrs.get("specification").cloned()),
        "MultipleValue" => Value::Multiple(node.children.iter().filter(|c| c.tag == "values").map(value_of).collect()),
        _ => Value::Other,
    }
}

impl Loader {
    /// Reads a resource (a file, or the bundled library) once.
    fn resource(&mut self, key: &str) -> Result<(), String> {
        if self.resources.contains_key(key) {
            return Ok(());
        }
        self.resources.insert(key.to_string(), HashMap::new());
        let text = if key == LIBRARY {
            BUILT_IN_TYPES.to_string()
        } else {
            match std::fs::read_to_string(self.base.join(key)) {
                Ok(text) => text,
                // An unreadable resource leaves its references unresolved.
                Err(_) => return Ok(()),
            }
        };
        let document = capella_xmi::parse(&text)?;
        let roots: Vec<&Node> = match document.children.first() {
            Some(root) if root.tag == "xmi:XMI" => root.children.iter().collect(),
            Some(root) => vec![root],
            None => Vec::new(),
        };
        for root in roots {
            let class = root.tag.rsplit(':').next().unwrap_or("");
            self.element(root, class, None, key)?;
        }
        Ok(())
    }

    fn element(&mut self, node: &Node, class: &str, container: Option<usize>, resource: &str) -> Result<Option<usize>, String> {
        let class = if node.kind.is_empty() { class } else { node.kind.rsplit(':').next().unwrap_or(class) };
        let Some(kind) = Kind::of(class) else { return Ok(None) };
        let mut obj = Obj::new(kind);
        obj.container = container;
        let attr = |name: &str| node.attrs.get(name).cloned();
        obj.id = attr("id");
        obj.originator = attr("originatorId");
        obj.name = attr("name");
        obj.description = attr("description");
        obj.file_name = attr("fileName");
        obj.is_library = attr("isLibrary").as_deref() == Some("true");
        if let Some(kind) = attr("kind") {
            obj.port_kind = kind;
        }
        // A port's index is 1 unless written (the metamodel's default).
        obj.index = attr("index").and_then(|v| v.parse().ok()).unwrap_or(1);
        if let Some(scope) = attr("scope") {
            obj.scope = scope;
        }
        if let Some(t) = attr("type") {
            if kind == Kind::Junction {
                obj.junction = t;
            } else {
                obj.decomposition = t;
            }
        }
        obj.event = attr("event");
        obj.condition = attr("condition");
        obj.action = attr("action");
        obj.comment = attr("comment");
        obj.entry = attr("entry");
        obj.during = attr("during");
        obj.exit = attr("exit");
        obj.value = attr("value").and_then(|v| v.parse().ok()).unwrap_or(0);
        obj.initial_value = attr("initialValue");
        obj.unit = attr("unit");
        if kind.is_block() {
            obj.source = attr("source");
        }
        let me = self.pivot.add(obj);
        self.resource_of.insert(me, resource.to_string());
        if let Some(id) = node.attrs.get("id") {
            self.resources.get_mut(resource).expect("the resource being read").insert(id.clone(), me);
        }
        // References written as attributes: ids of this resource.
        let references: &[&str] = match kind {
            Kind::Project => &["dictionaries"],
            Kind::Dictionary => &["subDictionaries"],
            Kind::System => &["dictionary"],
            Kind::Line => &["source", "target"],
            Kind::Transition => &["source", "destination"],
            _ => &[],
        };
        let mut features: Vec<&str> = references.to_vec();
        if kind.holds_datatype() {
            features.push("datatype");
        }
        if kind == Kind::SystemReference {
            features.push("system");
        }
        if kind.is_symbol() {
            features.push("underlyingPort");
        }
        for feature in features {
            if let Some(ids) = node.attrs.get(feature) {
                for id in ids.split_whitespace() {
                    self.pending.push((me, feature.to_string(), resource.to_string(), id.to_string(), reference_class(feature).to_string()));
                }
            }
        }
        for child in &node.children {
            let feature = child.tag.as_str();
            if let Some(href) = child.attrs.get("href") {
                let (file, id) = href.split_once('#').unwrap_or((href, ""));
                let class = if child.kind.is_empty() { reference_class(feature).to_string() } else { child.kind.rsplit(':').next().unwrap_or("").to_string() };
                self.pending.push((me, feature.to_string(), file.to_string(), id.to_string(), class));
                continue;
            }
            match feature {
                "parameters" => {
                    let key = child.attrs.get("key").cloned().unwrap_or_default();
                    let value = child.children.iter().find(|c| c.tag == "value").map(value_of).unwrap_or(Value::Primitive(None));
                    self.pivot[me].params.push((key, value));
                }
                "systems" | "blocks" | "lines" | "ports" | "referencedPorts" | "datatypes" | "busElements" | "literals" | "transitions" | "states" | "subStates" | "symbols" => {
                    let Some(i) = self.element(child, default_class(feature), Some(me), resource)? else { continue };
                    let list = match feature {
                        "systems" => &mut self.pivot[me].systems,
                        "blocks" => &mut self.pivot[me].blocks,
                        "lines" => &mut self.pivot[me].lines,
                        "ports" => &mut self.pivot[me].ports,
                        "referencedPorts" => &mut self.pivot[me].referenced_ports,
                        "datatypes" => &mut self.pivot[me].datatypes,
                        "busElements" => &mut self.pivot[me].bus_elements,
                        "literals" => &mut self.pivot[me].literals,
                        "transitions" => &mut self.pivot[me].transitions,
                        "states" => &mut self.pivot[me].states,
                        "subStates" => &mut self.pivot[me].sub_states,
                        _ => &mut self.pivot[me].symbols,
                    };
                    list.push(i);
                }
                _ => {}
            }
        }
        Ok(Some(me))
    }

    fn resolve(&mut self) -> Result<(), String> {
        // Resolving may read more resources, whose references come in turn.
        while !self.pending.is_empty() {
            let pending = std::mem::take(&mut self.pending);
            for (me, feature, resource, id, class) in pending {
                if !resource.is_empty() {
                    self.resource(&resource)?;
                }
                let target = match self.resources.get(&resource).and_then(|ids| ids.get(&id)) {
                    Some(&i) => i,
                    None if id.starts_with("derived-") => {
                        self.derived.push((me, feature, resource, id, class));
                        continue;
                    }
                    None => {
                        let mut proxy = Obj::new(Kind::Proxy);
                        proxy.name = Some(class);
                        self.pivot.add(proxy)
                    }
                };
                self.set(me, &feature, target);
            }
        }
        Ok(())
    }

    /// A model reference has no ports of its own: one per port of the system it references,
    /// `derived-<reference>--<port>`, with the port's kind, index, name and type; and the
    /// system's Capella id.
    fn derive_ports(&mut self) {
        for r in 0..self.pivot.objs.len() {
            if self.pivot[r].kind != Kind::SystemReference {
                continue;
            }
            let Some(system) = self.pivot[r].system.filter(|&s| self.pivot[s].kind == Kind::System) else { continue };
            self.pivot[r].originator = self.pivot[system].originator.clone();
            let resource = self.resource_of.get(&r).cloned().unwrap_or_default();
            let mut ports = Vec::new();
            for source in self.pivot[system].ports.clone() {
                let mut port = self.pivot[source].clone();
                port.container = Some(r);
                port.params = Vec::new();
                port.id = Some(format!("derived-{}--{}", self.pivot[r].id.clone().unwrap_or_default(), self.pivot[source].id.clone().unwrap_or_default()));
                let id = port.id.clone().unwrap_or_default();
                let i = self.pivot.add(port);
                self.resources.entry(resource.clone()).or_default().insert(id, i);
                ports.push(i);
            }
            self.pivot[r].ports = ports;
        }
        for (me, feature, resource, id, class) in std::mem::take(&mut self.derived) {
            let target = match self.resources.get(&resource).and_then(|ids| ids.get(&id)) {
                Some(&i) => i,
                None => {
                    let mut proxy = Obj::new(Kind::Proxy);
                    proxy.name = Some(class);
                    self.pivot.add(proxy)
                }
            };
            self.set(me, &feature, target);
        }
    }

    fn set(&mut self, me: usize, feature: &str, target: usize) {
        {
            {
                let obj = &mut self.pivot[me];
                match feature {
                    "dictionaries" => obj.dictionaries.push(target),
                    "subDictionaries" => obj.sub_dictionaries.push(target),
                    "dictionary" => obj.dictionary = Some(target),
                    "datatype" => obj.datatype = Some(target),
                    "system" => obj.system = Some(target),
                    "underlyingPort" => obj.underlying_port = Some(target),
                    "destination" => obj.destination = Some(target),
                    "target" => obj.line_target = Some(target),
                    "source" if obj.kind == Kind::Transition => obj.state_source = Some(target),
                    "source" => obj.line_source = Some(target),
                    _ => {}
                }
            }
        }
    }
}

/// Reads an `.mlcore` document; the files it refers to are read beside it.
pub fn load(path: &Path) -> Result<Pivot, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    read(&text, path.parent().unwrap_or(Path::new(".")))
}

/// Reads an `.mlcore` document; the files it refers to are read in `base`.
pub fn read(text: &str, base: &Path) -> Result<Pivot, String> {
    let mut loader = Loader { pivot: Pivot::default(), resources: HashMap::new(), pending: Vec::new(), derived: Vec::new(), resource_of: HashMap::new(), base: base.to_path_buf() };
    // The document itself is the resource "".
    loader.resources.insert(String::new(), HashMap::new());
    let document = capella_xmi::parse(text)?;
    let roots: Vec<&Node> = match document.children.first() {
        Some(root) if root.tag == "xmi:XMI" => root.children.iter().collect(),
        Some(root) => vec![root],
        None => Vec::new(),
    };
    for root in roots {
        let class = root.tag.rsplit(':').next().unwrap_or("");
        if let Some(i) = loader.element(root, class, None, "")? {
            loader.pivot.roots.push(i);
        }
    }
    loader.resolve()?;
    loader.derive_ports();
    Ok(loader.pivot)
}

/// Every element of the document, its own contents after it (the order EMF validates in).
pub fn tree(pivot: &Pivot) -> Vec<usize> {
    fn walk(pivot: &Pivot, i: usize, out: &mut Vec<usize>) {
        out.push(i);
        for c in contents(pivot, i) {
            walk(pivot, c, out);
        }
    }
    let mut out = Vec::new();
    for &root in &pivot.roots {
        walk(pivot, root, &mut out);
    }
    out
}

/// An element's contents, feature by feature in the metamodel's order.
pub fn contents(pivot: &Pivot, i: usize) -> Vec<usize> {
    let o = &pivot[i];
    let mut out = Vec::new();
    // PortHolder.ports first (a supertype of SLDiagram and Block), then the diagram's, the chart's…
    out.extend(&o.ports);
    out.extend(&o.blocks);
    out.extend(&o.lines);
    out.extend(&o.referenced_ports);
    out.extend(&o.transitions);
    out.extend(&o.states);
    out.extend(&o.symbols);
    out.extend(&o.sub_states);
    out.extend(&o.systems);
    out.extend(&o.bus_elements);
    out.extend(&o.datatypes);
    out.extend(&o.literals);
    out
}

/// The validation of the generator: an unresolved reference is an error, as EMF says it.
pub fn unresolved(pivot: &Pivot) -> Vec<String> {
    let label = |i: usize| {
        let o = &pivot[i];
        let mut text = o.kind.class().to_string();
        if let Some(name) = &o.name {
            text.push_str(&format!(" ({name})"));
        }
        if let Some(id) = &o.id {
            text.push_str(&format!(" [{id}]"));
        }
        text
    };
    let mut out = Vec::new();
    for i in tree(pivot) {
        let o = &pivot[i];
        // The references of each class, in the order of its features.
        let mut refs: Vec<(&str, Vec<usize>)> = Vec::new();
        if o.kind.holds_datatype() {
            refs.push(("datatype", o.datatype.into_iter().collect()));
        }
        match o.kind {
            Kind::Project => refs.push(("dictionaries", o.dictionaries.clone())),
            Kind::System => refs.push(("dictionary", o.dictionary.into_iter().collect())),
            Kind::SystemReference => refs.push(("system", o.system.into_iter().collect())),
            Kind::Line => {
                refs.push(("source", o.line_source.into_iter().collect()));
                refs.push(("target", o.line_target.into_iter().collect()));
            }
            Kind::Transition => {
                refs.push(("destination", o.destination.into_iter().collect()));
                refs.push(("source", o.state_source.into_iter().collect()));
            }
            Kind::Dictionary => refs.push(("subDictionaries", o.sub_dictionaries.clone())),
            k if k.is_symbol() => refs.push(("underlyingPort", o.underlying_port.into_iter().collect())),
            _ => {}
        }
        for (feature, targets) in refs {
            for t in targets {
                if pivot[t].kind == Kind::Proxy {
                    out.push(format!("The feature '{feature}' of '{}' contains an unresolved proxy '{}'", label(i), pivot[t].name.clone().unwrap_or_default()));
                }
            }
        }
    }
    out
}
