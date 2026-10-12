//! Import of a Capella project (`.capella`, Capella 6/7 XMI) into ArcLang.
//!
//! The model is read level by level — operational analysis, system
//! analysis, logical and physical architectures, EPBS — with its
//! requirements, data, interfaces, state machines, scenarios, property
//! values, constraints and realizations, and written as ArcLang source
//! that the compiler then compiles: an import that does not compile is an
//! import that failed, never a half model. Capella's own identifiers
//! become the `id` of every element, so a model imported twice keeps its
//! identities, and what the language cannot hold yet is counted and named
//! in the notes rather than dropped in silence.
//!
//! The diagrams of the `.aird` beside the file become `diagram`
//! declarations: the elements a Capella diagram shows are what the
//! ArcLang diagram shows, as Capella users expect their diagrams back.

mod behaviour;
mod data;
mod diagrams;
mod levels;

use super::metamodel::Metamodel;
use super::source_edit::string_literal;
use super::{Compiler, CompilerConfig};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

/// One XML element, with its `xsi:type` shortened to `pkg:Kind`.
#[derive(Debug, Clone, Default)]
pub struct Node {
    pub tag: String,
    pub kind: String,
    pub attrs: BTreeMap<String, String>,
    pub children: Vec<Node>,
    /// Text content, for the few elements that carry one (expression bodies).
    pub text: String,
}

impl Node {
    fn attr(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).map(String::as_str)
    }
    fn id(&self) -> &str {
        self.attr("id").unwrap_or("")
    }
    fn name(&self) -> &str {
        self.attr("name").unwrap_or("")
    }
    fn is(&self, kind: &str) -> bool {
        self.kind == kind
    }
    /// Children with this tag, in order.
    fn tagged<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = &'a Node> + 'a {
        self.children.iter().filter(move |child| child.tag == tag)
    }
    /// Every descendant, depth first.
    fn walk<'a>(&'a self, into: &mut Vec<&'a Node>) {
        for child in &self.children {
            into.push(child);
            child.walk(into);
        }
    }
    /// The first descendant of this kind.
    fn find(&self, kind: &str) -> Option<&Node> {
        let mut all = Vec::new();
        self.walk(&mut all);
        all.into_iter().find(|node| node.is(kind))
    }
    fn find_all(&self, kind: &str) -> Vec<&Node> {
        let mut all = Vec::new();
        self.walk(&mut all);
        all.into_iter().filter(|node| node.is(kind)).collect()
    }
    /// Every descendant, in document order.
    fn descendants(&self) -> Vec<&Node> {
        let mut all = Vec::new();
        self.walk(&mut all);
        all
    }
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&#xA;", "\n").replace("&#xD;", "").replace("&#x9;", "\t").replace("&amp;", "&")
}

fn short_kind(xsi: &str) -> String {
    // `org.polarsys.capella.core.data.la:LogicalComponent` -> `la:LogicalComponent`;
    // `Requirements:Requirement` stays.
    match xsi.rsplit_once(':') {
        Some((prefix, kind)) => format!("{}:{}", prefix.rsplit('.').next().unwrap_or(prefix), kind),
        None => xsi.to_string(),
    }
}

/// Parse an XMI document into a tree. Fails on malformed XML.
pub fn parse(xml: &str) -> Result<Node, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut stack: Vec<Node> = vec![Node { tag: "#document".to_string(), ..Default::default() }];
    let mut buffer = Vec::new();
    loop {
        let event = reader.read_event_into(&mut buffer).map_err(|e| format!("XML error at byte {}: {e}", reader.buffer_position()))?;
        let is_empty = matches!(event, Event::Empty(_));
        match event {
            Event::Start(start) | Event::Empty(start) => {
                let mut node = Node { tag: String::from_utf8_lossy(start.name().as_ref()).to_string(), ..Default::default() };
                for attribute in start.attributes().flatten() {
                    let key = String::from_utf8_lossy(attribute.key.as_ref()).to_string();
                    let value = unescape(&String::from_utf8_lossy(&attribute.value));
                    if key == "xsi:type" || key == "xmi:type" {
                        node.kind = short_kind(&value);
                    } else {
                        node.attrs.insert(key, value);
                    }
                }
                if is_empty {
                    stack.last_mut().expect("document").children.push(node);
                } else {
                    stack.push(node);
                }
            }
            Event::Text(text) => {
                let text = unescape(&String::from_utf8_lossy(&text));
                stack.last_mut().expect("document").text.push_str(&text);
            }
            Event::CData(data) => {
                let text = String::from_utf8_lossy(&data);
                stack.last_mut().expect("document").text.push_str(&text);
            }
            Event::End(_) => {
                let done = stack.pop().expect("an open element");
                stack.last_mut().expect("document").children.push(done);
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    stack.pop().ok_or_else(|| "empty document".to_string())
}

/// A reference attribute: `#id`, or `file.capella#id`, several separated by spaces.
fn refs(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or("")
        .split_whitespace()
        .map(|item| item.rsplit('#').next().unwrap_or(item).to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn first_ref(value: Option<&str>) -> Option<String> {
    refs(value).into_iter().next()
}

/// An import that produced no compiling model: the message, and the text
/// written so far so that the reader can see what was refused.
pub struct ImportFailure {
    pub message: String,
    pub rejected: Option<String>,
}

impl From<String> for ImportFailure {
    fn from(message: String) -> Self {
        ImportFailure { message, rejected: None }
    }
}
impl From<&str> for ImportFailure {
    fn from(message: &str) -> Self {
        ImportFailure { message: message.to_string(), rejected: None }
    }
}
impl std::fmt::Display for ImportFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::fmt::Debug for ImportFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// What the import produced.
#[derive(Debug, Clone)]
pub struct Imported {
    pub source: String,
    /// What was read, by kind of element, for the report.
    pub counts: BTreeMap<&'static str, usize>,
    /// What was left out and why.
    pub notes: Vec<String>,
}

/// A property value of Capella's PVMT, resolved to its ArcLang literal.
#[derive(Debug, Clone)]
struct PropertyValue {
    key: String,
    literal: String,
}

pub(crate) struct Writer {
    out: String,
    counts: BTreeMap<&'static str, usize>,
    notes: Vec<String>,
    /// Capella id -> ArcLang id written for it.
    ids: HashMap<String, String>,
    /// Capella id -> ArcLang kind written for it (metamodel name).
    kinds: HashMap<String, &'static str>,
    /// Capella id -> name written for it (what a transition or a path names).
    names: HashMap<String, String>,
    /// Capella id of a port -> id of the element holding it.
    port_owner: HashMap<String, String>,
    /// Capella id of a component port -> `component.port`, as an exchange names it.
    port_ref: HashMap<String, String>,
    /// Capella id of a part -> id of the component it types.
    part_type: HashMap<String, String>,
    traces: Vec<(String, &'static str, String)>,
    /// Property value groups and values, by id, ready to write.
    value_groups: HashMap<String, Vec<PropertyValue>>,
    values: HashMap<String, PropertyValue>,
    /// Names of the exchange categories, by exchange id.
    categories: HashMap<String, Vec<String>>,
    /// The progress status literals (DRAFT, REVIEWED...), by id.
    status_literals: HashMap<String, String>,
    /// The whole SystemEngineering, for writers that look across levels.
    engineering: Node,
    metamodel: Metamodel,
}

impl Writer {
    fn line(&mut self, depth: usize, text: &str) {
        for _ in 0..depth {
            self.out.push_str("    ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }
    fn count(&mut self, what: &'static str) {
        *self.counts.entry(what).or_insert(0) += 1;
    }
    fn count_by(&mut self, what: &'static str, by: usize) {
        if by > 0 {
            *self.counts.entry(what).or_insert(0) += by;
        }
    }
    fn note(&mut self, text: String) {
        if !self.notes.contains(&text) {
            self.notes.push(text);
        }
    }
    /// Register the element as written, under its Capella id and kind.
    fn remember(&mut self, node: &Node, kind: &'static str) -> String {
        let id = node.id().to_string();
        self.ids.insert(id.clone(), id.clone());
        self.kinds.insert(id.clone(), kind);
        self.names.insert(id.clone(), display_name(node));
        id
    }
    fn written(&self, id: &str) -> bool {
        self.ids.contains_key(id)
    }
    /// The written ids among `references`, in order.
    fn known(&self, references: &[String]) -> Vec<String> {
        references.iter().filter(|r| self.written(r)).map(|r| self.ids[r.as_str()].clone()).collect()
    }
    /// The written ids of a given kind among `references`.
    fn known_of(&self, references: &[String], kinds: &[&str]) -> Vec<String> {
        references
            .iter()
            .filter(|r| self.written(r) && self.kinds.get(r.as_str()).is_some_and(|k| kinds.contains(k)))
            .map(|r| self.ids[r.as_str()].clone())
            .collect()
    }
    /// Whether `key` is an attribute the metamodel types on `kind`: a Capella
    /// value written under it would be judged by that type.
    fn is_typed(&self, kind: &str, key: &str) -> bool {
        self.metamodel.kind(kind).is_some_and(|spec| spec.attributes.iter().any(|a| a.key == key))
    }
}

/// The name an element is written under: its name, else its id.
fn display_name(node: &Node) -> String {
    let name = node.name().trim();
    if name.is_empty() {
        node.id().to_string()
    } else {
        name.to_string()
    }
}

fn text_attr(node: &Node, key: &str) -> Option<String> {
    node.attr(key).map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

/// `description` as Capella writes it: HTML; kept as plain text.
fn plain(html: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for c in html.chars() {
        match c {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(c),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ").split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A Capella name as an ArcLang attribute key: lowercase words joined by
/// `_`, starting with a letter.
fn key_of(name: &str) -> String {
    let mut key = String::new();
    for c in name.trim().chars() {
        if c.is_ascii_alphanumeric() {
            key.push(c.to_ascii_lowercase());
        } else if !key.ends_with('_') && !key.is_empty() {
            key.push('_');
        }
    }
    let key = key.trim_end_matches('_').to_string();
    match key.chars().next() {
        Some(c) if c.is_ascii_alphabetic() => key,
        Some(_) => format!("v_{key}"),
        None => "value".to_string(),
    }
}

/// A list attribute of written references: `key: ["a", "b"]`.
fn list_literal(items: &[String]) -> String {
    format!("[{}]", items.iter().map(|item| string_literal(item)).collect::<Vec<_>>().join(", "))
}

/// The usual header of an element: id, description, summary, and what
/// Capella attaches to any element: property values and constraints.
fn header_attributes(writer: &mut Writer, depth: usize, node: &Node, kind: &str) {
    writer.line(depth, &format!("id: {}", string_literal(node.id())));
    if let Some(description) = text_attr(node, "description") {
        writer.line(depth, &format!("description: {}", string_literal(&plain(&description))));
    }
    if let Some(summary) = text_attr(node, "summary") {
        writer.line(depth, &format!("summary: {}", string_literal(&summary)));
    }
    if kind != "Class" {
        if let Some(status) = first_ref(node.attr("status")).and_then(|id| writer.status_literals.get(&id).cloned()) {
            let key = if writer.is_typed(kind, "status") { "capella_status" } else { "status" };
            writer.line(depth, &format!("{key}: {}", string_literal(&status)));
        }
        if let Some(review) = text_attr(node, "review") {
            writer.line(depth, &format!("review: {}", string_literal(&review)));
        }
    }
    data::applied_values(writer, depth, node, kind);
    data::constraints_of(writer, depth, node);
}

// --- the import -----------------------------------------------------------------

/// Import a `.capella` file; the `.aird` beside it, when there is one, gives
/// the diagrams. The result compiles, or the import fails.
pub fn import_file(path: &Path) -> Result<Imported, ImportFailure> {
    let xml = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let aird_path = path.with_extension("aird");
    let aird = match std::fs::read_to_string(&aird_path) {
        Ok(text) => Some(text),
        Err(_) => None,
    };
    import_strings(&xml, aird.as_deref(), &path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default())
}

pub fn import_strings(capella_xml: &str, aird_xml: Option<&str>, file_name: &str) -> Result<Imported, ImportFailure> {
    let document = parse(capella_xml)?;
    let project = document.children.first().ok_or("no root element")?;
    if !project.kind.ends_with(":Project") && !project.tag.ends_with(":Project") {
        return Err(format!("not a Capella project: the root is <{}>", project.tag).into());
    }
    let engineering = project.find("capellamodeller:SystemEngineering").ok_or("no SystemEngineering in the project")?;
    let mut writer = Writer {
        out: String::new(),
        counts: BTreeMap::new(),
        notes: Vec::new(),
        ids: HashMap::new(),
        kinds: HashMap::new(),
        names: HashMap::new(),
        port_owner: HashMap::new(),
        port_ref: HashMap::new(),
        part_type: HashMap::new(),
        traces: Vec::new(),
        value_groups: HashMap::new(),
        values: HashMap::new(),
        categories: HashMap::new(),
        status_literals: project.find_all("capellacore:EnumerationPropertyLiteral").into_iter().map(|l| (l.id().to_string(), display_name(l))).collect(),
        engineering: engineering.clone(),
        metamodel: Metamodel::current(),
    };
    let project_name = project.attr("name").filter(|n| !n.is_empty()).unwrap_or("Capella model");
    writer.line(0, &format!("model {} {{", string_literal(project_name)));
    writer.line(1, &format!("name: {}", string_literal(project_name)));
    writer.line(1, &format!("description: {}", string_literal(&format!("Imported from Capella ({file_name})"))));
    writer.line(0, "}");
    writer.line(0, "");

    // What any element may refer to: parts, property values, categories.
    for part in project.find_all("cs:Part") {
        if let Some(typed) = first_ref(part.attr("abstractType")) {
            writer.part_type.insert(part.id().to_string(), typed);
        }
    }
    data::collect_property_values(&mut writer, project);
    for category in project.find_all("fa:ExchangeCategory") {
        for exchange in refs(category.attr("exchanges")) {
            writer.categories.entry(exchange).or_default().push(display_name(category));
        }
    }

    // The data model first: exchanges and interfaces name its items.
    data::data_model(&mut writer, engineering);
    data::requirements(&mut writer, project);
    for architecture in engineering.tagged("ownedArchitectures") {
        match architecture.kind.as_str() {
            "oa:OperationalAnalysis" => levels::operational(&mut writer, architecture),
            "ctx:SystemAnalysis" => levels::system(&mut writer, architecture),
            "la:LogicalArchitecture" => levels::logical(&mut writer, architecture),
            "pa:PhysicalArchitecture" => levels::physical(&mut writer, architecture),
            "epbs:EPBSArchitecture" => levels::epbs(&mut writer, architecture),
            other => writer.note(format!("architecture '{}' of kind {other} left out", architecture.name())),
        }
    }
    behaviour::state_machines(&mut writer, engineering);
    behaviour::scenarios(&mut writer, engineering);
    data::requirement_relations(&mut writer, project);

    // Realizations between what was written.
    let traces = std::mem::take(&mut writer.traces);
    let mut seen = HashSet::new();
    for (from, kind, to) in traces {
        if writer.written(&from) && writer.written(&to) && seen.insert((from.clone(), kind, to.clone())) {
            writer.count("realization traces");
            // Scenarios and state machines are known by name, the rest by id.
            let by_name = |writer: &Writer, id: &str| matches!(writer.kinds.get(id), Some(&"Scenario" | &"StateMachine"));
            let reference = |writer: &Writer, id: &str| if by_name(writer, id) { writer.names[id].clone() } else { writer.ids[id].clone() };
            let (from, to) = (reference(&writer, &from), reference(&writer, &to));
            writer.line(0, &format!("trace {} {kind} {}", string_literal(&from), string_literal(&to)));
        }
    }
    writer.line(0, "");

    if let Some(aird_xml) = aird_xml {
        match parse(aird_xml) {
            Ok(aird) => diagrams::diagrams(&mut writer, &aird),
            Err(error) => writer.note(format!("the .aird could not be read: {error}")),
        }
    } else {
        writer.note("no .aird beside the file: no diagram imported".to_string());
    }

    let source = writer.out.clone();
    Compiler::new(CompilerConfig { optimization_level: 0, target: "json".to_string() })
        .compile_string(&source)
        .map_err(|e| {
            let error = e.to_string();
            // The line the compiler points at, so the reader sees what was written.
            let shown = error
                .split("line ")
                .nth(1)
                .and_then(|rest| rest.split(|c: char| !c.is_ascii_digit()).next()?.parse::<usize>().ok())
                .and_then(|line| source.lines().nth(line.saturating_sub(1)).map(|text| format!("\n  line {line}: {}", text.trim())))
                .unwrap_or_default();
            ImportFailure { message: format!("the imported model does not compile: {error}{shown}"), rejected: Some(source.clone()) }
        })?;
    Ok(Imported { source, counts: writer.counts, notes: writer.notes })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/capella/test7_0/Model Test 7.0.capella")
    }

    #[test]
    fn the_capella_7_fixture_imports_compiles_and_keeps_its_diagrams() {
        let imported = import_file(&fixture()).expect("imports");
        let counts = &imported.counts;
        assert!(counts.get("logical components").copied().unwrap_or(0) >= 10, "{counts:?}");
        assert!(counts.get("logical functions").copied().unwrap_or(0) >= 10, "{counts:?}");
        assert!(counts.get("physical nodes").copied().unwrap_or(0) >= 1, "{counts:?}");
        assert!(counts.get("requirements").copied().unwrap_or(0) >= 5, "{counts:?}");
        assert!(counts.get("component exchanges").copied().unwrap_or(0) >= 5, "{counts:?}");
        assert!(counts.get("diagrams").copied().unwrap_or(0) >= 3, "{counts:?} {:?}", imported.notes);
        // The behaviour, the data and the reuse of the model come along.
        assert!(counts.get("scenarios").copied().unwrap_or(0) >= 3, "{counts:?}");
        assert!(counts.get("messages").copied().unwrap_or(0) >= 10, "{counts:?}");
        assert!(counts.get("states").copied().unwrap_or(0) >= 10, "{counts:?}");
        assert!(counts.get("transitions").copied().unwrap_or(0) >= 10, "{counts:?}");
        assert!(counts.get("functional chains").copied().unwrap_or(0) >= 1, "{counts:?}");
        assert!(counts.get("physical paths").copied().unwrap_or(0) >= 1, "{counts:?}");
        assert!(counts.get("enumerations").copied().unwrap_or(0) >= 1, "{counts:?}");
        assert!(counts.get("property values").copied().unwrap_or(0) >= 5, "{counts:?}");
        assert!(counts.get("configuration items").copied().unwrap_or(0) >= 1, "{counts:?}");
        // Capella's identifiers are the ids: an import is stable.
        assert!(imported.source.contains("id: \"") && imported.source.contains("component_exchange"));
        let again = import_file(&fixture()).expect("imports again");
        assert_eq!(again.source, imported.source);
    }

    #[test]
    fn a_file_that_is_not_a_project_is_refused() {
        assert!(import_strings("<a/>", None, "a.capella").is_err());
        assert!(import_strings("<not xml", None, "a.capella").is_err());
    }

    #[test]
    fn capella_names_become_attribute_keys() {
        assert_eq!(key_of("Power Level"), "power_level");
        assert_eq!(key_of("cars_defeated"), "cars_defeated");
        assert_eq!(key_of("2nd value"), "v_2nd_value");
        assert_eq!(key_of("Order of Merlin (1st class)"), "order_of_merlin_1st_class");
    }
}
