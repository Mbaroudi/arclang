//! The connector's model-to-model half: Capella elements chosen by their ids
//! become the pivot (`.mlcore`) that the generator turns into MATLAB. A port of
//! `org.eclipse.capella.simulink.capella.m2m` (Thales, EPL 2.0) and of the parts
//! of Kitalpha's Transposer it runs on, done over the Capella XMI itself.
//!
//! As in the connector: an export strategy widens the selection (`scope`), the
//! pre-analysis activities make the project, the Transposer builds the graph of
//! the rules' premises, orders it and applies the rules (`function`, `data`,
//! `sm`), then the dictionaries are linked to those they use.

mod classes;
mod data;
mod diagram;
mod function;
pub mod model;
mod scope;
mod sm;
mod transposer;

use super::pivot::{self, Kind, Obj, Pivot, Value};
use model::Model;
use std::collections::HashMap;

/// The connector's export operations (`capella2simulink.mldescription`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// `FUNCTIONS`: functions, their ports and the states they are available in.
    Functions,
    /// `FUNCTIONAL_CHAINS`: functional chains and their functions.
    FunctionalChains,
    /// `STATE_MACHINES`: state machines, as Stateflow charts.
    StateMachines,
    /// `DATA_PKG`: data packages, as data dictionaries.
    DataPackages,
}

impl Strategy {
    pub fn parse(text: &str) -> Option<Strategy> {
        Some(match text.to_ascii_uppercase().replace('-', "_").as_str() {
            "FUNCTIONS" => Strategy::Functions,
            "FUNCTIONAL_CHAINS" => Strategy::FunctionalChains,
            "STATE_MACHINES" => Strategy::StateMachines,
            "DATA_PKG" | "DATA_PACKAGES" => Strategy::DataPackages,
            _ => return None,
        })
    }
}

/// The operation's options, as the connector's configuration holds them.
#[derive(Debug, Clone)]
pub struct Options {
    /// `leaf.functions.export.option`: `model` (the default) or `subsystem`.
    pub leaf_functions: String,
    /// `port.naming.option`: `ExchangeItem` (the default) or `CapellaElement`.
    pub port_naming: String,
}

impl Default for Options {
    fn default() -> Options {
        Options { leaf_functions: "model".to_string(), port_naming: "ExchangeItem".to_string() }
    }
}

impl Options {
    fn leaf_as_model(&self) -> bool {
        self.leaf_functions.eq_ignore_ascii_case("Model")
    }

    fn leaf_as_subsystem(&self) -> bool {
        self.leaf_functions.eq_ignore_ascii_case("SubSystem")
    }

    fn ports_named_by_exchange(&self) -> bool {
        self.port_naming == "ExchangeItem"
    }
}

/// A key of the Transposer's context: a Capella element, or a string the rules make.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Key {
    E(usize),
    S(String),
}

/// What the context maps a key to.
#[derive(Debug, Clone, PartialEq)]
enum Target {
    /// An element of the pivot.
    P(usize),
    /// The ports a Capella function port became (one per functional exchange).
    Ports(Vec<usize>),
}

/// The state of one transformation: the Capella model, the selection, the pivot
/// being made and the Transposer's context.
struct Run<'m> {
    m: &'m Model,
    options: Options,
    /// The selection after the strategy widened it (`TRANSPOSER_SELECTION`).
    selection: Vec<usize>,
    p: Pivot,
    project: usize,
    targets: HashMap<Key, Target>,
    /// The dictionaries made (`IConstants.DICTIONARIES`).
    dictionaries: Vec<usize>,
    /// The elements of the built-in types library: below this index.
    library: usize,
    /// A port's `ElementReference`: the port it was propagated from.
    reference: HashMap<usize, usize>,
    /// The ports a system reference derives from its system's: (reference, system port) -> port.
    derived: HashMap<(usize, usize), usize>,
    /// A derived port -> the system port it stands for (`ReferencedPort.getSourcePort`).
    source_port: HashMap<usize, usize>,
    /// The geometry of the selected state machine diagrams, by region.
    geometry: HashMap<usize, diagram::View>,
    uuids: usize,
}

impl<'m> Run<'m> {
    fn selected(&self, e: Option<usize>) -> bool {
        e.is_some_and(|e| self.selection.contains(&e))
    }

    fn target(&self, key: &Key) -> Option<usize> {
        match self.targets.get(key) {
            Some(Target::P(i)) => Some(*i),
            _ => None,
        }
    }

    fn main(&self, e: usize) -> Option<usize> {
        self.target(&Key::E(e))
    }

    fn main_opt(&self, e: Option<usize>) -> Option<usize> {
        e.and_then(|e| self.main(e))
    }

    fn set_main(&mut self, key: Key, target: usize) {
        self.targets.insert(key, Target::P(target));
    }

    // ---- Making the pivot -------------------------------------------------------

    fn new_obj(&mut self, kind: Kind, name: Option<String>, originator: Option<String>) -> usize {
        let mut obj = Obj::new(kind);
        obj.name = name;
        obj.originator = originator;
        self.p.add(obj)
    }

    fn set_param(&mut self, i: usize, key: &str, value: &str) {
        self.p[i].set_param(key, Value::Primitive(Some(value.to_string())));
    }

    /// `PropertyHelper.setLayer`.
    fn set_layer(&mut self, i: usize, layer: &str) {
        self.p[i].set_param("capella.layer", Value::Primitive(Some(layer.to_string())));
    }

    fn contain(&mut self, parent: usize, child: usize, list: fn(&mut Obj) -> &mut Vec<usize>) {
        list(&mut self.p[parent]).push(child);
        self.p[child].container = Some(parent);
    }

    /// The layer of an element (`CapellaModelHelper.getElementLayer`): OA, SA, LA, PA or EPBS.
    fn layer(&self, e: usize) -> Option<&'static str> {
        let architecture = self.m.ancestor(e, "BlockArchitecture")?;
        Some(match self.m.class(architecture) {
            c if model::is_a(c, "OperationalAnalysis") => "OA",
            c if model::is_a(c, "SystemAnalysis") => "SA",
            c if model::is_a(c, "LogicalArchitecture") => "LA",
            c if model::is_a(c, "PhysicalArchitecture") => "PA",
            c if model::is_a(c, "EPBSArchitecture") => "EPBS",
            _ => return None,
        })
    }

    /// `DictionaryHelper.createDictionary`.
    fn create_dictionary(&mut self, e: usize, name: Option<String>) -> Result<usize, String> {
        let dictionary = self.new_obj(Kind::Dictionary, name, self.m.id(e));
        self.dictionaries.push(dictionary);
        let Some(layer) = self.layer(e) else { return Err(format!("NullPointerException: no engineering perspective for {}", self.label(e))) };
        self.set_layer(dictionary, layer);
        let project = self.project;
        self.p[project].dictionaries.push(dictionary);
        self.p.roots.push(dictionary);
        Ok(dictionary)
    }

    /// `DictionaryHelper.createDefaultDictionaries`: the layer's dictionary, and the system's own.
    fn create_default_dictionaries(&mut self, e: usize, system: usize) -> Result<(), String> {
        let Some(architecture) = self.m.ancestor(e, "BlockArchitecture") else { return Err(format!("NullPointerException: no engineering perspective for {}", self.label(e))) };
        let layer_dictionary = match self.main(architecture) {
            Some(d) => d,
            None => {
                let d = self.create_dictionary(architecture, self.m.name(architecture))?;
                self.p[d].set_param("capella.layer.root", Value::Primitive(Some("True".to_string())));
                self.set_main(Key::E(architecture), d);
                d
            }
        };
        if self.p[system].dictionary.is_none() {
            let d = self.create_dictionary(e, self.p[system].name.clone())?;
            self.p[system].dictionary = Some(d);
            self.p[d].sub_dictionaries.push(layer_dictionary);
        }
        Ok(())
    }

    /// A built-in MATLAB type of the library (`PredefinedTypesHelper.getType`).
    fn predefined(&self, name: Option<&str>) -> Option<usize> {
        let id = data::predefined_id(name?)?;
        (0..self.library).find(|&i| self.p[i].id.as_deref() == Some(id))
    }

    /// The ports of a holder; a system reference's are those of its system (`getReferencedPorts`).
    fn ports(&mut self, holder: usize) -> Vec<usize> {
        if self.p[holder].kind != Kind::SystemReference {
            return self.p[holder].ports.clone();
        }
        let Some(system) = self.p[holder].system else { return Vec::new() };
        let ports = self.p[system].ports.clone();
        ports.into_iter().map(|port| self.derived_port(holder, port)).collect()
    }

    /// The port a system reference has for one of its system's ports.
    fn derived_port(&mut self, reference: usize, port: usize) -> usize {
        if let Some(&d) = self.derived.get(&(reference, port)) {
            return d;
        }
        let mut obj = Obj::new(Kind::Port);
        obj.container = Some(reference);
        let d = self.p.add(obj);
        self.derived.insert((reference, port), d);
        self.source_port.insert(d, port);
        d
    }

    /// `SystemReference.getReferencedPort(systemPort)`.
    fn referenced_port(&mut self, reference: usize, port: usize) -> Option<usize> {
        self.p[reference].system?;
        self.ports(reference).into_iter().find(|d| self.source_port.get(d) == Some(&port))
    }

    /// A new id for what the connector names with `EcoreUtil.generateUUID()`.
    fn uuid(&mut self, seed: &str) -> String {
        self.uuids += 1;
        uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, format!("arclang-simulink:{seed}:{}", self.uuids).as_bytes()).to_string()
    }

    fn label(&self, e: usize) -> String {
        format!("{} '{}' [{}]", self.m.class(e), self.m.name(e).unwrap_or_default(), self.m.id(e).unwrap_or_default())
    }

    // ---- Finishing --------------------------------------------------------------

    /// `DictionaryDependencyActivity`: a dictionary uses the dictionaries of its bus elements' types.
    fn link_dictionaries(&mut self) {
        for dictionary in self.dictionaries.clone() {
            for i in pivot_contents(&self.p, dictionary) {
                if self.p[i].kind != Kind::BusElement {
                    continue;
                }
                let Some(datatype) = self.p[i].datatype else { continue };
                let Some(container) = self.p[datatype].container else { continue };
                if self.p[container].kind == Kind::Dictionary && container != dictionary && !self.p[dictionary].sub_dictionaries.contains(&container) && container >= self.library {
                    self.p[dictionary].sub_dictionaries.push(container);
                }
            }
        }
    }

    /// A system reference's ports as the saved pivot has them once read again: one per port of
    /// its system, in its order, each the system port's copy; and the system's Capella id.
    fn derive_reference_ports(&mut self) {
        for r in 0..self.p.objs.len() {
            if self.p[r].kind != Kind::SystemReference {
                continue;
            }
            let Some(system) = self.p[r].system else { continue };
            self.p[r].originator = self.p[system].originator.clone();
            let mut ports = Vec::new();
            for source in self.p[system].ports.clone() {
                let d = self.derived_port(r, source);
                let mut port = self.p[source].clone();
                port.container = Some(r);
                port.params = Vec::new();
                port.id = Some(format!("derived-{}--{}", self.p[r].id.clone().unwrap_or_else(|| "null".to_string()), self.p[source].id.clone().unwrap_or_else(|| "null".to_string())));
                self.p[d] = port;
                ports.push(d);
            }
            self.p[r].ports = ports;
        }
    }
}

/// Every element a pivot element contains, depth first.
fn pivot_contents(p: &Pivot, i: usize) -> Vec<usize> {
    let mut out = Vec::new();
    for c in pivot::contents(p, i) {
        out.push(c);
        out.extend(pivot_contents(p, c));
    }
    out
}

/// Transforms the elements with these ids, as the connector's export operation does.
pub fn transform(m: &Model, strategy: Strategy, ids: &[&str], options: &Options) -> Result<Pivot, String> {
    let mut targets = Vec::new();
    for id in ids {
        match m.by_id(id) {
            Some(e) => targets.push(e),
            None => return Err(format!("Cannot find element: {id}")),
        }
    }
    let selection = scope::compute(m, strategy, &targets)?;
    if selection.is_empty() {
        return Err("Nothing to transform: the selection is empty".to_string());
    }
    // The built-in types come first in the arena, as a resource of their own.
    let library = pivot::read(pivot::BUILT_IN_TYPES, std::path::Path::new("."))?;
    let count = library.objs.len();
    let mut p = Pivot { objs: library.objs, roots: Vec::new() };
    // `GenericInitializeTransformation`: the project, named after the system engineering.
    let first = selection.iter().copied().find(|&e| m.is(e, "CapellaElement")).ok_or_else(|| "No object to export!".to_string())?;
    let engineering = m.ancestor(first, "SystemEngineering").ok_or_else(|| "The selection is not in a Capella model".to_string())?;
    let mut project = Obj::new(Kind::Project);
    project.name = m.name(engineering);
    project.originator = m.id(engineering);
    let project = p.add(project);
    p.roots.push(project);
    let mut run = Run {
        m,
        options: options.clone(),
        selection,
        p,
        project,
        targets: HashMap::new(),
        dictionaries: Vec::new(),
        library: count,
        reference: HashMap::new(),
        derived: HashMap::new(),
        source_port: HashMap::new(),
        geometry: HashMap::new(),
        uuids: 0,
    };
    if strategy == Strategy::StateMachines {
        sm::initialize(&mut run)?;
        run.geometry = diagram::collect(m, &run.selection)?;
    }
    transposer::run(&mut run, strategy)?;
    run.link_dictionaries();
    run.derive_reference_ports();
    Ok(run.p)
}
