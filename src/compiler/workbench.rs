//! What a modelling workbench needs from the compiler, in one place: the
//! snapshot of a model (diagrams as the viewer mounts them, the explorer
//! document, every element with its typed attributes, the findings, the
//! gate) and the edits of its text (set, unset, rename, add, remove, move,
//! traces, fragments), each refused unless the edited model compiles and
//! shows what was asked. The CLI (`arclang workbench`, `arclang edit`), the
//! WebAssembly build of the playground and the VS Code extension all go
//! through here, so they never disagree.

use crate::compiler::elements::{self, ElementGraph};
use crate::compiler::source_edit::{self, Container};
use crate::compiler::{CompilationResult, Compiler, CompilerConfig};
use serde::{Deserialize, Serialize};
use serde_json::json;

mod links;
pub use links::fresh_id;

/// What a snapshot computes besides the model itself.
#[derive(Debug, Clone)]
pub struct SnapshotOptions<'a> {
    /// The standard the production gate is run against (`ISO26262`, `DO178C`, `IEC61508`).
    pub gate_standard: &'a str,
    /// Also render the interop exports (SysML v2, ReqIF, C headers, Protobuf).
    pub exports: bool,
    /// Also render the standalone explorer page (the `arclang explorer` HTML).
    pub explorer_html: bool,
}

impl Default for SnapshotOptions<'_> {
    fn default() -> Self {
        Self { gate_standard: "ISO26262", exports: false, explorer_html: false }
    }
}

impl SnapshotOptions<'_> {
    /// Everything: what the playground's wasm build has always returned.
    pub fn full() -> Self {
        Self { gate_standard: "ISO26262", exports: true, explorer_html: true }
    }
}

/// The snapshot as a JSON string, never failing: a model that does not
/// compile gives `{"success": false, "errors": [...]}`.
pub fn snapshot_json(source: &str, options: &SnapshotOptions) -> String {
    serde_json::to_string(&snapshot(source, options))
        .unwrap_or_else(|e| format!("{{\"success\":false,\"errors\":[\"serialization: {e}\"]}}"))
}

#[derive(Serialize)]
pub struct Stats {
    pub requirements: usize,
    pub components: usize,
    pub functions: usize,
    pub traces: usize,
    pub missions: usize,
    pub capabilities: usize,
    pub functional_chains: usize,
}

#[derive(Serialize)]
pub struct GateFindingOut {
    pub check: String,
    pub severity: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct GateOut {
    pub passed: bool,
    pub requirements_total: usize,
    pub requirements_verified: usize,
    pub findings: Vec<GateFindingOut>,
}

#[derive(Serialize)]
pub struct Snapshot {
    pub success: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub stats: Option<Stats>,
    /// The standalone architecture dossier (the `arclang explorer` page).
    pub explorer_html: Option<String>,
    /// The diagrams of the model, as `ArcViz.mount` takes them: one ELK
    /// graph per Arcadia view, the traces between views, the drawing
    /// diagnostics. The playground mounts the viewer on this directly.
    pub diagrams: Option<serde_json::Value>,
    /// The architecture document behind the dossier: requirements,
    /// components, interfaces, functions, traces, chains, machines,
    /// scenarios and the data model, resolved by the compiler.
    pub document: Option<serde_json::Value>,
    /// Every element of the compiled model with its typed attributes, as
    /// the modelling API lists them: what a properties view edits.
    pub elements: Option<serde_json::Value>,
    /// Every warning, traceability and methodology finding, each named
    /// after its rule (`rules()` lists them) with the element concerned.
    pub findings: Vec<crate::compiler::rules::Finding>,
    pub gate: Option<GateOut>,
    /// OMG SysML v2 textual notation (interop subset).
    pub sysmlv2: Option<String>,
    /// OMG ReqIF 1.0 (requirements exchange).
    pub reqif: Option<String>,
    /// C99 interface contracts.
    pub c_headers: Option<String>,
    /// Protobuf (proto3) interface contracts.
    pub proto: Option<String>,
}

pub fn snapshot(source: &str, options: &SnapshotOptions) -> Snapshot {
    let mut compiler = Compiler::new(CompilerConfig {
        optimization_level: 0,
        target: "json".to_string(),
    });
    snapshot_of(compiler.compile_string(source), options)
}

/// The snapshot of a model file, its imports resolved relative to it.
pub fn snapshot_file_json(path: &std::path::Path, options: &SnapshotOptions) -> String {
    let mut compiler = Compiler::new(CompilerConfig {
        optimization_level: 0,
        target: "json".to_string(),
    });
    serde_json::to_string(&snapshot_of(compiler.compile_file(path), options))
        .unwrap_or_else(|e| format!("{{\"success\":false,\"errors\":[\"serialization: {e}\"]}}"))
}

fn snapshot_of(compiled: Result<crate::CompilationResult, crate::CompilerError>, options: &SnapshotOptions) -> Snapshot {
    match compiled {
        Err(e) => Snapshot {
            success: false,
            errors: vec![e.to_string()],
            warnings: Vec::new(),
            stats: None,
            explorer_html: None,
            diagrams: None,
            document: None,
            elements: None,
            findings: Vec::new(),
            gate: None,
            sysmlv2: None,
            reqif: None,
            c_headers: None,
            proto: None,
        },
        Ok(result) => {
            let model = &result.semantic_model;
            let stats = Stats {
                requirements: model.requirements.len(),
                components: model.components.len(),
                functions: model.functions.len(),
                traces: model.traces.len(),
                missions: model.missions.len(),
                capabilities: model.capabilities.len(),
                functional_chains: model.functional_chains.len(),
            };
            let explorer = crate::compiler::arcviz_explorer::generate_explorer_html(
                model,
                &result.ast,
            )
            .ok();
            let explorer_html = if options.explorer_html { explorer.as_ref().map(|(html, _json)| html.clone()) } else { None };
            let document = explorer
                .as_ref()
                .and_then(|(_html, json)| serde_json::from_str(json).ok());
            // The declared model name, as the dossier titles itself.
            let title = model
                .name
                .clone()
                .unwrap_or_else(|| "System Architecture".to_string());
            let findings = crate::compiler::rules::findings(&result);
            let diagrams = crate::compiler::diagram::build_diagrams(&result.ast);
            let diagrams = crate::compiler::diagram::html::viewer_payload(&diagrams, &title);
            let gate_report = crate::compiler::production_gate::run_gate(
                &result.ast,
                model,
                options.gate_standard,
            );
            let gate = GateOut {
                passed: gate_report.passed,
                requirements_total: gate_report.requirements_total,
                requirements_verified: gate_report.requirements_verified,
                findings: gate_report
                    .findings
                    .iter()
                    .map(|f| GateFindingOut {
                        check: f.check.clone(),
                        severity: format!("{:?}", f.severity),
                        message: f.message.clone(),
                    })
                    .collect(),
            };
            Snapshot {
                success: true,
                errors: Vec::new(),
                warnings: result.warnings,
                stats: Some(stats),
                explorer_html,
                diagrams: Some(diagrams),
                document,
                elements: Some(elements_json(&elements::build(&result.ast, model))),
                findings,
                gate: Some(gate),
                sysmlv2: options.exports.then(|| crate::compiler::sysmlv2_generator::generate_sysmlv2(model, &result.ast)),
                reqif: options.exports.then(|| crate::compiler::reqif::generate_reqif(model, &result.ast)),
                c_headers: options.exports.then(|| crate::compiler::c_header_generator::generate_c_headers(model, &result.ast)),
                proto: options.exports.then(|| crate::compiler::proto_generator::generate_proto(model, &result.ast)),
            }
        }
    }
}

/// Rename an element the way a modelling workbench does: the element keeps
/// its identity and what referred to it still does. An element that
/// writes no `id` is first given its current id in writing; references
/// that used its name are rewritten to the id; then the name changes. The
/// edited model must compile, show the same identities and leave no
/// reference dangling.
pub fn rename_keeping_identity(source: &str, element: &str, name: &str) -> Result<String, String> {
    let before_compiled = compile_text(source).map_err(|e| format!("the model does not compile: {e}"))?;
    let before = elements::build(&before_compiled.ast, &before_compiled.semantic_model);
    let record = before
        .elements
        .iter()
        .find(|e| &e.id == element)
        .or_else(|| before.elements.iter().find(|e| &e.name == element))
        .ok_or_else(|| format!("no element with id or name '{element}'"))?;
    let (id, old_name) = (record.id.clone(), record.name.clone());
    let check = |edited: &str| -> Result<(), String> {
        let after_compiled = compile_text(edited).map_err(|e| format!("edit refused: the renamed model does not compile: {e}"))?;
        let after = elements::build(&after_compiled.ast, &after_compiled.semantic_model);
        let renamed = after.elements.iter().find(|e| e.uuid == record.uuid);
        if renamed.map(|e| e.name.as_str()) != Some(name) {
            return Err(format!("edit refused: '{old_name}' would not read '{name}' once compiled"));
        }
        if identities(&before) != identities(&after) {
            return Err(format!("edit refused: renaming '{old_name}' would change identities"));
        }
        if after.unresolved.len() > before.unresolved.len() {
            return Err(format!("edit refused: renaming '{old_name}' would leave a reference without an end"));
        }
        Ok(())
    };
    // The plain rename, when nothing depends on the name.
    if let Ok(edited) = source_edit::rename_element(source, element, name) {
        if check(&edited).is_ok() {
            return Ok(edited);
        }
    }
    // Otherwise: the id in writing, the references on the id, the new name.
    let written = record.attributes.contains_key("id");
    let mut text = source.to_string();
    if !written {
        text = source_edit::set_attribute(&text, element, "id", &source_edit::string_literal(&id)).map_err(|e| e.to_string())?;
    }
    let (rewritten, _moved) = source_edit::rename_references(&text, &old_name, &id, &id).map_err(|e| e.to_string())?;
    let edited = source_edit::rename_element(&rewritten, &id, name).map_err(|e| e.to_string())?;
    check(&edited)?;
    Ok(edited)
}

/// The element graph as a properties view reads it.
pub fn elements_json(graph: &ElementGraph) -> serde_json::Value {
    json!({
        "elements": graph.elements.iter().map(|element| {
            let mut record = json!({
                "uuid": element.uuid,
                "id": element.id,
                "kind": element.kind,
                "name": element.name,
                "owner": element.owner,
                "attributes": elements::attributes_json(&element.attributes),
            });
            // A port says which way it goes: what a reconnection must respect.
            if let Some(direction) = element.extra.get("direction") {
                record["direction"] = direction.clone();
            }
            record
        }).collect::<Vec<_>>(),
        "relationships": graph.relationships.iter().map(|relationship| json!({
            "uuid": relationship.uuid,
            "kind": relationship.kind,
            "name": relationship.name,
            "source": relationship.source,
            "target": relationship.target,
            "attributes": elements::attributes_json(&relationship.attributes),
        })).collect::<Vec<_>>(),
    })
}

/// One edit of the model text, as `arclang set`, `unset`, `rename`, the
/// modelling API's create and delete, and `trace` write it. Attribute
/// values are ArcLang source (`"text"`, `25 ms`, `[A, B]`).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum EditRequest {
    Set { element: String, key: String, value: String },
    Unset { element: String, key: String },
    Rename { element: String, name: String },
    Add {
        keyword: String,
        name: String,
        #[serde(default)]
        attributes: Vec<(String, String)>,
        /// The element the declaration goes into, else a layer, else (with
        /// `top_level`) the end of the file.
        container: Option<String>,
        layer: Option<String>,
        layer_name: Option<String>,
        #[serde(default)]
        top_level: bool,
    },
    /// One declaration written as ArcLang source (`transition "A" -> "B" {}`,
    /// `message "A" -> "B" "text"`, `provides interface "I" {}`), added as
    /// the last declaration of an element, a layer, or the file.
    AddRaw {
        declaration: String,
        container: Option<String>,
        layer: Option<String>,
        layer_name: Option<String>,
        #[serde(default)]
        top_level: bool,
    },
    Remove { element: String },
    /// Wrap the messages of a scenario, from `first` to `last` (labels), in a
    /// combined fragment; `operands` cut it at the messages named, with guards.
    WrapFragment {
        scenario: String,
        kind: String,
        #[serde(default)]
        label: String,
        first: String,
        last: String,
        #[serde(default)]
        operands: Vec<(String, String)>,
    },
    /// Move an element's declaration into another element (or a layer):
    /// a function reallocated, a component nested or deployed elsewhere.
    Move { element: String, container: Option<String>, layer: Option<String>, layer_name: Option<String> },
    AddTrace { from: String, kind: String, to: String },
    RemoveTrace { from: String, kind: String, to: String },
    /// Reconnect the relationship of `kind` from `from` to `to` (named
    /// `name` when several join them): its source to `new_from`, its target
    /// to `new_to`. Both, swapped, turn it around.
    Reconnect {
        kind: String,
        #[serde(default)]
        name: Option<String>,
        from: String,
        to: String,
        #[serde(default)]
        new_from: Option<String>,
        #[serde(default)]
        new_to: Option<String>,
    },
    /// A copy of an element with what it holds, ids made unique: beside
    /// the original, or into an element or a layer.
    Duplicate { element: String, container: Option<String>, layer: Option<String>, layer_name: Option<String> },
}

pub fn compile_text(source: &str) -> Result<CompilationResult, String> {
    Compiler::new(CompilerConfig { optimization_level: 0, target: "json".to_string() })
        .compile_string(source)
        .map_err(|e| e.to_string())
}

pub fn identities(graph: &ElementGraph) -> Vec<String> {
    let mut all: Vec<String> = graph.elements.iter().map(|e| e.uuid.clone()).collect();
    all.extend(graph.relationships.iter().map(|r| r.uuid.clone()));
    all.sort();
    all
}

/// The designator the text knows for an element: its written id or its name.
/// An id the compiler derived from the name (nothing in the text spells it)
/// is translated to the name; anything else is kept as written.
pub fn written_designator(graph: &ElementGraph, source: &str, designator: &str) -> String {
    if appears_as_token(source, designator) || graph.elements.iter().any(|e| e.name == designator && e.id == designator) {
        return designator.to_string();
    }
    let Some(element) = graph.elements.iter().find(|e| e.id == designator) else {
        return designator.to_string();
    };
    // A port `Owner.port`: kept when the owner writes its id (two owners may
    // both have an `hv` port), the owner as the text knows it otherwise.
    if let Some((owner, port)) = designator.rsplit_once('.') {
        if graph.elements.iter().any(|e| e.id == owner) {
            return if appears_as_token(source, owner) {
                designator.to_string()
            } else {
                format!("{}.{port}", written_designator(graph, source, owner))
            };
        }
    }
    // A name another element also carries (the same actor in two layers) is
    // designated inside its layer block.
    let shared = graph.elements.iter().filter(|e| e.name == element.name).count() > 1;
    match (shared, source_edit::layer_keyword_of_kind(&element.kind)) {
        (true, Some(layer)) => format!("{layer}::{}", element.name),
        _ => element.name.clone(),
    }
}

/// Whether `token` occurs in `text` as a whole identifier (not inside a
/// longer one such as `LC-A` inside `LC-Alpha`).
fn appears_as_token(text: &str, token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    let is_word = |c: char| c.is_alphanumeric() || matches!(c, '_' | '-' | '.');
    text.match_indices(token).any(|(at, _)| {
        let before = text[..at].chars().next_back().is_none_or(|c| !is_word(c));
        let after = text[at + token.len()..].chars().next().is_none_or(|c| !is_word(c));
        before && after
    })
}

impl EditRequest {
    /// The same request with every element designator translated to what
    /// the text writes (see [`written_designator`]).
    pub fn with_written_designators(&self, graph: &ElementGraph, source: &str) -> EditRequest {
        let w = |designator: &str| written_designator(graph, source, designator);
        let opt = |designator: &Option<String>| designator.as_deref().map(w);
        match self {
            EditRequest::Set { element, key, value } => EditRequest::Set { element: w(element), key: key.clone(), value: value.clone() },
            EditRequest::Unset { element, key } => EditRequest::Unset { element: w(element), key: key.clone() },
            EditRequest::Rename { element, name } => EditRequest::Rename { element: w(element), name: name.clone() },
            EditRequest::Add { keyword, name, attributes, container, layer, layer_name, top_level } => EditRequest::Add {
                keyword: keyword.clone(),
                name: name.clone(),
                attributes: attributes.clone(),
                container: opt(container),
                layer: layer.clone(),
                layer_name: layer_name.clone(),
                top_level: *top_level,
            },
            EditRequest::AddRaw { declaration, container, layer, layer_name, top_level } => EditRequest::AddRaw {
                declaration: declaration.clone(),
                container: opt(container),
                layer: layer.clone(),
                layer_name: layer_name.clone(),
                top_level: *top_level,
            },
            EditRequest::Remove { element } => EditRequest::Remove { element: w(element) },
            EditRequest::WrapFragment { scenario, kind, label, first, last, operands } => EditRequest::WrapFragment {
                scenario: w(scenario),
                kind: kind.clone(),
                label: label.clone(),
                first: first.clone(),
                last: last.clone(),
                operands: operands.clone(),
            },
            EditRequest::Move { element, container, layer, layer_name } => EditRequest::Move {
                element: w(element),
                container: opt(container),
                layer: layer.clone(),
                layer_name: layer_name.clone(),
            },
            EditRequest::AddTrace { from, kind, to } => EditRequest::AddTrace { from: w(from), kind: kind.clone(), to: w(to) },
            EditRequest::RemoveTrace { from, kind, to } => EditRequest::RemoveTrace { from: w(from), kind: kind.clone(), to: w(to) },
            // Resolved on the compiled model by the edit itself.
            EditRequest::Reconnect { .. } | EditRequest::Duplicate { .. } => self.clone(),
        }
    }
}

/// The edits checked on relationships and identities (see [`links`]): a
/// rename (of an element, else of a relationship), a port moved with its
/// exchanges, a reconnection, a copy. None for the others.
fn edit_links(source: &str, request: &EditRequest) -> Option<Result<String, String>> {
    match request {
        EditRequest::Rename { element, name } => Some(rename_keeping_identity(source, element, name).or_else(|error| {
            if error.starts_with("no element with id or name") {
                links::rename_relationship(source, element, name)
            } else {
                Err(error)
            }
        })),
        EditRequest::Move { element, container: Some(owner), .. } => {
            let compiled = compile_text(source).ok()?;
            let graph = elements::build(&compiled.ast, &compiled.semantic_model);
            let port = graph.elements.iter().find(|e| &e.id == element || &e.name == element).filter(|e| e.kind.ends_with("Port"))?;
            Some(links::move_port(source, &port.id.clone(), owner))
        }
        EditRequest::Reconnect { kind, name, from, to, new_from, new_to } => Some(links::reconnect(source, kind, name.as_deref(), from, to, new_from.as_deref(), new_to.as_deref())),
        EditRequest::Duplicate { element, container, layer, layer_name } => {
            let target = match (container, layer) {
                (Some(other), _) => Some(Container::Element(other)),
                (None, Some(layer)) => Some(Container::Layer { keyword: layer, name: layer_name.as_deref() }),
                (None, None) => None,
            };
            Some(links::duplicate(source, element, target))
        }
        _ => None,
    }
}

/// Apply one edit written as JSON (`{"op": "set", ...}`); see [`edit`].
pub fn edit_json(source: &str, request: &str) -> Result<String, String> {
    let request: EditRequest = serde_json::from_str(request).map_err(|e| format!("invalid edit request: {e}"))?;
    edit(source, &request)
}

/// Apply one edit to ArcLang source and return the edited text. The edit is
/// refused unless the edited model compiles and shows exactly what was
/// asked: comments, order and every other declaration are kept.
pub fn edit(source: &str, request: &EditRequest) -> Result<String, String> {
    // The snapshot names every element by the id the compiler gives it,
    // written or derived from its name; the text editor only knows what is
    // written. A derived id is translated to the written name first.
    let resolved = match compile_text(source) {
        Ok(compiled) => request.with_written_designators(&elements::build(&compiled.ast, &compiled.semantic_model), source),
        Err(_) => request.clone(),
    };
    let request = &resolved;
    if let Some(outcome) = edit_links(source, request) {
        return outcome;
    }
    let edited = match &request {
        EditRequest::Set { element, key, value } => source_edit::set_attribute(source, element, key, value),
        EditRequest::Unset { element, key } => source_edit::remove_attribute(source, element, key),
        EditRequest::Rename { element, name } => source_edit::rename_element(source, element, name),
        EditRequest::Add { keyword, name, attributes, container, layer, layer_name, top_level } => {
            let container = match (container, layer, top_level) {
                (Some(element), _, _) => Container::Element(element),
                (None, Some(layer), _) => Container::Layer { keyword: layer, name: layer_name.as_deref() },
                (None, None, true) => Container::Document,
                (None, None, false) => return Err("an element is added inside an element, a layer, or at the top level".to_string()),
            };
            source_edit::add_element(source, container, keyword, name, attributes)
        }
        EditRequest::AddRaw { declaration, container, layer, layer_name, top_level } => {
            let container = match (container, layer, top_level) {
                (Some(element), _, _) => Container::Element(element),
                (None, Some(layer), _) => Container::Layer { keyword: layer, name: layer_name.as_deref() },
                (None, None, true) => Container::Document,
                (None, None, false) => return Err("a declaration is added inside an element, a layer, or at the top level".to_string()),
            };
            source_edit::add_declaration(source, container, declaration)
        }
        EditRequest::Remove { element } => source_edit::remove_element(source, element),
        EditRequest::WrapFragment { scenario, kind, label, first, last, operands } => {
            source_edit::wrap_fragment(source, scenario, kind, label, first, last, operands)
        }
        EditRequest::Move { element, container, layer, layer_name } => {
            let container = match (container, layer) {
                (Some(other), _) => Container::Element(other),
                (None, Some(layer)) => Container::Layer { keyword: layer, name: layer_name.as_deref() },
                (None, None) => return Err("an element is moved into an element or a layer".to_string()),
            };
            source_edit::move_element(source, element, container)
        }
        EditRequest::AddTrace { from, kind, to } => source_edit::add_trace(source, from, kind, to),
        EditRequest::RemoveTrace { from, kind, to } => source_edit::remove_trace(source, &[from.as_str()], kind, &[to.as_str()]),
        EditRequest::Reconnect { .. } | EditRequest::Duplicate { .. } => unreachable!("handled by edit_links"),
    }
    .map_err(|e| e.to_string())?;

    let compiled = compile_text(&edited).map_err(|e| format!("edit refused: the edited model does not compile: {e}"))?;
    let after = elements::build(&compiled.ast, &compiled.semantic_model);
    match &request {
        EditRequest::Set { element, key, value } => {
            // A diagram, a scenario or a state machine is no element of the
            // graph: the compile above is its confirmation.
            match source_edit::confirm_effect(&after, element, key, value) {
                Ok(()) | Err(source_edit::EditError::NotFound(_)) => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        EditRequest::Move { element, .. } => {
            // Every element keeps its identity; what referred to the moved one still does.
            let before_compiled = compile_text(source).map_err(|e| format!("the model does not compile: {e}"))?;
            let before = elements::build(&before_compiled.ast, &before_compiled.semantic_model);
            if identities(&before) != identities(&after) {
                return Err(format!(
                    "edit refused: moving '{element}' would change identities or what refers to it; give it an `id` and refer to it by that id first"
                ));
            }
            if after.unresolved.len() > before.unresolved.len() {
                return Err(format!("edit refused: moving '{element}' would leave a reference without an end"));
            }
        }
        _ => {}
    }
    Ok(edited)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL: &str = r#"model M {
    name: "M"
}
logical_architecture LA {
    component "Controller" {
        id: "LC-001"
        description: "Controls" // kept
        function "Compute" {}
    }
    component "Sensor" {}
}
requirements system {
    req "REQ-001" "Shall compute" {
        description: "The system shall compute."
    }
}
"#;

    fn parse(outcome: String) -> serde_json::Value {
        serde_json::from_str(&outcome).expect("json")
    }

    pub(super) fn compile(source: &str) -> String {
        snapshot_json(source, &SnapshotOptions::full())
    }

    pub(super) fn edit(source: &str, request: &str) -> String {
        match edit_json(source, request) {
            Ok(edited) => json!({ "success": true, "source": edited }).to_string(),
            Err(error) => json!({ "success": false, "error": error }).to_string(),
        }
    }

    #[test]
    fn an_element_is_designated_by_the_id_the_compiler_derives_from_its_name() {
        // `Sensor` writes no id: the snapshot lists it as LC-Sensor, and edits accept that id.
        let snapshot = parse(compile(MODEL));
        let sensor = snapshot["elements"]["elements"].as_array().unwrap().iter().find(|e| e["name"] == "Sensor").expect("sensor");
        assert_eq!(sensor["id"], "LC-Sensor");
        let set = parse(edit(MODEL, r#"{"op":"set","element":"LC-Sensor","key":"description","value":"\"Radar\""}"#));
        assert_eq!(set["success"], true, "{set}");
        let text = set["source"].as_str().unwrap();
        assert!(text.contains("component \"Sensor\" {") && text.contains("description: \"Radar\""), "{text}");
        let added = parse(edit(MODEL, r#"{"op":"add","keyword":"function","name":"Sense","container":"LC-Sensor"}"#));
        assert_eq!(added["success"], true, "{added}");
        let removed = parse(edit(MODEL, r#"{"op":"remove","element":"LC-Sensor"}"#));
        assert_eq!(removed["success"], true, "{removed}");
        assert!(!removed["source"].as_str().unwrap().contains("Sensor"));
    }

    #[test]
    fn a_name_two_layers_share_an_arrow_header_and_a_port_of_an_id_less_owner_all_edit() {
        // The same actor in two layers, both without id: the derived ids pick the layer.
        let model = "model T { name: \"t\" }\noperational_analysis \"OA\" {\n    actor \"Driver\" {}\n}\nsystem_analysis \"SA\" {\n    actor \"Driver\" {}\n    function \"Sense\" {\n        port out lane { data_type: \"Lane\" }\n    }\n    function \"Plan\" {\n        port in lane { data_type: \"Lane\" }\n    }\n    functional_exchange \"Sense.lane\" -> \"Plan.lane\" { label: \"lane\" }\n}\n";
        let snapshot = parse(compile(model));
        let ids: Vec<String> = snapshot["elements"]["elements"].as_array().unwrap().iter().filter(|e| e["name"] == "Driver").map(|e| e["id"].as_str().unwrap().to_string()).collect();
        assert_eq!(ids.len(), 2, "{ids:?}");
        for id in &ids {
            let edited = parse(edit(model, &format!(r#"{{"op":"set","element":"{id}","key":"description","value":"\"for {id}\""}}"#)));
            assert_eq!(edited["success"], true, "{id}: {edited}");
            assert_eq!(edited["source"].as_str().unwrap().matches(&format!("for {id}")).count(), 1);
        }
        // `Sense.lane` is the port, not the exchange whose header spells it.
        let port = snapshot["elements"]["elements"].as_array().unwrap().iter().find(|e| e["kind"] == "FunctionPort" && e["id"].as_str().unwrap().ends_with(".lane") && e["id"].as_str().unwrap().contains("Sense")).expect("the port")["id"].as_str().unwrap().to_string();
        let edited = parse(edit(model, &format!(r#"{{"op":"set","element":"{port}","key":"data_type","value":"\"LaneModel\""}}"#)));
        assert_eq!(edited["success"], true, "{edited}");
        assert!(edited["source"].as_str().unwrap().contains("port out lane { data_type: \"LaneModel\" }"), "{}", edited["source"]);
    }

    #[test]
    fn a_port_of_an_owner_with_an_id_is_designated_through_its_owner() {
        // Two components with ids, both with an `hv` port: `LC-A.hv` is the first only.
        let model = "model T { name: \"t\" }\nlogical_architecture \"L\" {\n    component \"A\" {\n        id: \"LC-A\"\n        port inout \"hv\" { protocol: \"x\" }\n    }\n    component \"B\" {\n        id: \"LC-B\"\n        port inout \"hv\" { protocol: \"y\" }\n    }\n}\n";
        let edited = parse(edit(model, r#"{"op":"set","element":"LC-A.hv","key":"protocol","value":"\"z\""}"#));
        assert_eq!(edited["success"], true, "{edited}");
        let source = edited["source"].as_str().unwrap();
        assert!(source.contains("protocol: \"z\"") && source.contains("protocol: \"y\"") && !source.contains("protocol: \"x\""), "{source}");
    }

    #[test]
    fn a_moved_block_takes_the_indentation_of_its_new_container() {
        let model = "model T { name: \"t\" }\nlogical_architecture \"L\" {\n  component \"A\" {\n    id: \"LC-A\"\n    function \"F\" { id: \"LF-F\" }\n  }\n  component \"B\" {\n    id: \"LC-B\"\n    function \"G\" {\n      id: \"LF-G\"\n    }\n  }\n}\n";
        let edited = parse(edit(model, r#"{"op":"move","element":"LC-B","container":"LC-A"}"#));
        assert_eq!(edited["success"], true, "{edited}");
        let source = edited["source"].as_str().unwrap();
        assert!(source.contains("    component \"B\" {\n      id: \"LC-B\"\n      function \"G\" {\n        id: \"LF-G\"\n      }\n    }\n  }"), "{source}");
    }

    #[test]
    fn compile_reports_diagrams_document_and_elements() {
        let output = parse(compile(MODEL));
        assert_eq!(output["success"], true);
        assert!(output["diagrams"]["graphs"].as_array().unwrap().len() >= 1);
        assert_eq!(output["document"]["components"].as_array().unwrap().len(), 2);
        let elements = output["elements"]["elements"].as_array().unwrap();
        let controller = elements.iter().find(|e| e["id"] == "LC-001").expect("controller");
        assert_eq!(controller["kind"], "LogicalComponent");
        assert_eq!(controller["attributes"]["description"], "Controls");
    }

    #[test]
    fn set_keeps_the_comment_and_shows_the_value() {
        let output = parse(edit(MODEL, r#"{"op":"set","element":"LC-001","key":"description","value":"\"Controls braking\""}"#));
        assert_eq!(output["success"], true, "{output}");
        let source = output["source"].as_str().unwrap();
        assert!(source.contains("description: \"Controls braking\" // kept"));
        assert!(source.contains("function \"Compute\" {}"));
    }

    #[test]
    fn add_inside_an_element_and_in_a_layer() {
        let inside = parse(edit(MODEL, r#"{"op":"add","keyword":"function","name":"Filter","container":"LC-001"}"#));
        assert_eq!(inside["success"], true, "{inside}");
        assert!(inside["source"].as_str().unwrap().contains("function \"Filter\" {}"));
        let layer = parse(edit(MODEL, r#"{"op":"add","keyword":"component","name":"Actuator","layer":"logical_architecture","attributes":[["id","\"LC-003\""]]}"#));
        assert_eq!(layer["success"], true, "{layer}");
        let source = layer["source"].as_str().unwrap();
        assert!(source.contains("component \"Actuator\" {"));
        assert!(source.contains("id: \"LC-003\""));
    }

    #[test]
    fn rename_keeps_the_identity_by_writing_the_id_and_moving_the_references() {
        // `Sensor` writes no id and nothing refers to it: its id is written, then it is renamed.
        let renamed = parse(edit(MODEL, r#"{"op":"rename","element":"Sensor","name":"Radar"}"#));
        assert_eq!(renamed["success"], true, "{renamed}");
        let text = renamed["source"].as_str().unwrap();
        assert!(text.contains("component \"Radar\" {"), "{text}");
        assert!(text.contains("id: \"LC-Sensor\""), "{text}");
        let plain = parse(edit(MODEL, r#"{"op":"rename","element":"LC-001","name":"Brake controller"}"#));
        assert_eq!(plain["success"], true, "{plain}");
        assert!(plain["source"].as_str().unwrap().contains("component \"Brake controller\" {"));
        // Known by name in an exchange, a chain and a trace: the references move to the id.
        let model = "model T { name: \"t\" }\nlogical_architecture LA {\n    component Alpha {\n        port out p {}\n    }\n    component Beta {\n        port in q {}\n    }\n    component_exchange \"ab\" { from_port: \"Alpha.p\" to_port: \"Beta.q\" }\n    functional_chain Chain { involves: [Alpha, Beta] }\n}\nrequirements system {\n    req \"REQ-1\" \"R\" { description: \"d\" }\n}\ntrace \"Alpha\" satisfies \"REQ-1\"\n";
        let moved = parse(edit(model, r#"{"op":"rename","element":"Alpha","name":"Alpha prime"}"#));
        assert_eq!(moved["success"], true, "{moved}");
        let text = moved["source"].as_str().unwrap();
        assert!(text.contains("component \"Alpha prime\" {"), "{text}");
        assert!(text.contains("from_port: \"LC-Alpha.p\""), "{text}");
        assert!(text.contains("trace \"LC-Alpha\" satisfies"), "{text}");
        assert!(text.contains("involves: [\"LC-Alpha\", Beta]"), "{text}");
    }

    #[test]
    fn a_diagram_and_a_class_take_attributes() {
        let model = "model T { name: \"t\" }\nlogical_architecture LA {\n    component A { id: \"LC-A\" }\n    component B { id: \"LC-B\" }\n}\nclass Frame {\n    id: \"CL-1\"\n}\ndiagram \"Focus\" {\n    view: lab\n    shows: [\"LC-A\"]\n}\n";
        let shown: serde_json::Value = serde_json::from_str(&edit(model, r#"{"op":"set","element":"Focus","key":"shows","value":"[\"LC-A\", \"LC-B\"]"}"#)).unwrap();
        assert_eq!(shown["success"], true, "{shown}");
        assert!(shown["source"].as_str().unwrap().contains("shows: [\"LC-A\", \"LC-B\"]"));
        let field: serde_json::Value = serde_json::from_str(&edit(model, r#"{"op":"set","element":"CL-1","key":"speed","value":"Float"}"#)).unwrap();
        assert_eq!(field["success"], true, "{field}");
        assert!(field["source"].as_str().unwrap().contains("speed: Float"));
    }

    #[test]
    fn messages_are_wrapped_in_a_fragment_with_operands() {
        let model = "model T { name: \"t\" }\nlogical_architecture LA {\n    component A { id: \"LC-A\" }\n    component B { id: \"LC-B\" }\n}\nscenario Run {\n    participants: [\"A\", \"B\"]\n    message A -> B \"start\"\n    message B -> A \"tick\"\n    message A -> B \"ack\" { type: \"async\" }\n    message B -> A \"done\"\n}\n";
        let looped = parse(edit(model, r#"{"op":"wrap_fragment","scenario":"Run","kind":"loop","label":"while running","first":"tick","last":"ack"}"#));
        assert_eq!(looped["success"], true, "{looped}");
        let text = looped["source"].as_str().unwrap();
        assert!(text.contains("fragment loop \"while running\" {\n        message B -> A \"tick\"\n        message A -> B \"ack\" { type: \"async\" }\n    }"), "{text}");
        let output = parse(compile(text));
        assert_eq!(output["success"], true);
        let alt = parse(edit(model, r#"{"op":"wrap_fragment","scenario":"Run","kind":"alt","first":"tick","last":"ack","operands":[["tick","ok"],["ack","else"]]}"#));
        assert_eq!(alt["success"], true, "{alt}");
        let text = alt["source"].as_str().unwrap();
        assert!(text.contains("fragment alt {\n        operand \"ok\" {\n            message B -> A \"tick\"\n        }\n        operand \"else\" {\n            message A -> B \"ack\" { type: \"async\" }\n        }\n    }"), "{text}");
        assert_eq!(parse(compile(text))["success"], true);
        let bad = parse(edit(model, r#"{"op":"wrap_fragment","scenario":"Run","kind":"loop","first":"ack","last":"tick"}"#));
        assert_eq!(bad["success"], false);
        // A fragment already there is kept whole inside a wider one.
        let text = looped["source"].as_str().unwrap();
        let wider = parse(edit(text, r#"{"op":"wrap_fragment","scenario":"Run","kind":"alt","first":"start","last":"done","operands":[["start","go"],["done","end"]]}"#));
        assert_eq!(wider["success"], true, "{wider}");
        let text = wider["source"].as_str().unwrap();
        assert!(text.contains("operand \"go\" {\n            message A -> B \"start\"\n            fragment loop \"while running\" {\n                message B -> A \"tick\"\n                message A -> B \"ack\" { type: \"async\" }\n            }\n        }\n        operand \"end\" {\n            message B -> A \"done\"\n        }"), "{text}");
        assert_eq!(parse(compile(text))["success"], true);
        // Ends in different blocks are refused.
        let cut = parse(edit(text, r#"{"op":"wrap_fragment","scenario":"Run","kind":"opt","first":"tick","last":"done"}"#));
        assert_eq!(cut["success"], false);
        assert!(cut["error"].as_str().unwrap().contains("same block"), "{cut}");
    }

    #[test]
    fn raw_declarations_and_top_level_additions() {
        let model = "logical_architecture LA {\n    component \"A\" { id: \"LC-A\" }\n}\nstate_machine Modes {\n    initial: \"Off\"\n    state \"Off\" {}\n    state \"On\" {}\n}\n";
        let transition = parse(edit(model, r#"{"op":"add_raw","declaration":"transition \"Off\" -> \"On\" { trigger: \"start\" }","container":"Modes"}"#));
        assert_eq!(transition["success"], true, "{transition}");
        assert!(transition["source"].as_str().unwrap().contains("transition \"Off\" -> \"On\" { trigger: \"start\" }"));
        let provides = parse(edit(model, r#"{"op":"add_raw","declaration":"provides interface \"IData\" {}","container":"LC-A"}"#));
        assert_eq!(provides["success"], true, "{provides}");
        let class = parse(edit(model, r#"{"op":"add","keyword":"class","name":"Frame","top_level":true,"attributes":[["id","\"CL-1\""]]}"#));
        assert_eq!(class["success"], true, "{class}");
        assert!(class["source"].as_str().unwrap().trim_end().ends_with("class \"Frame\" {\n    id: \"CL-1\"\n}"), "{}", class["source"]);
        let bad = parse(edit(model, r#"{"op":"add_raw","declaration":"transition \"Off\" -> \"Nowhere\" {}","container":"Modes"}"#));
        assert_eq!(bad["success"], false);
        let comment = parse(edit(model, r#"{"op":"add_raw","declaration":"state \"X\" {} // no","container":"Modes"}"#));
        assert_eq!(comment["success"], false);
    }

    #[test]
    fn trace_added_then_removed_and_invalid_requests_refused() {
        let added = parse(edit(MODEL, r#"{"op":"add_trace","from":"LC-001","kind":"satisfies","to":"REQ-001"}"#));
        assert_eq!(added["success"], true, "{added}");
        let with_trace = added["source"].as_str().unwrap().to_string();
        assert!(with_trace.contains("trace \"LC-001\" satisfies \"REQ-001\""));
        let removed = parse(edit(&with_trace, r#"{"op":"remove_trace","from":"LC-001","kind":"satisfies","to":"REQ-001"}"#));
        assert_eq!(removed["success"], true, "{removed}");
        assert!(!removed["source"].as_str().unwrap().contains("satisfies"));
        let bad = parse(edit(MODEL, r#"{"op":"set","element":"LC-001","key":"description","value":"not a value"}"#));
        assert_eq!(bad["success"], false);
        let garbage = parse(edit(MODEL, "nope"));
        assert_eq!(garbage["success"], false);
        let unknown = parse(edit(MODEL, r#"{"op":"remove","element":"Nobody"}"#));
        assert_eq!(unknown["success"], false);
        assert!(unknown["error"].as_str().unwrap().contains("Nobody"));
    }
}

#[cfg(test)]
mod connect_tests {
    use super::tests::{compile, edit};
    use super::*;

    #[test]
    fn ports_and_an_exchange_can_be_added_from_the_workbench() {
        let model = "logical_architecture LA {\n    component \"A\" { id: \"LC-A\" }\n    component \"B\" { id: \"LC-B\" }\n}\n";
        let with_out: serde_json::Value = serde_json::from_str(&edit(model, r#"{"op":"add","keyword":"port out","name":"Data","container":"LC-A"}"#)).unwrap();
        assert_eq!(with_out["success"], true, "{with_out}");
        let text = with_out["source"].as_str().unwrap().to_string();
        assert!(text.contains("port out \"Data\""));
        let with_in: serde_json::Value = serde_json::from_str(&edit(&text, r#"{"op":"add","keyword":"port in","name":"Data","container":"LC-B"}"#)).unwrap();
        assert_eq!(with_in["success"], true, "{with_in}");
        let text = with_in["source"].as_str().unwrap().to_string();
        let wired: serde_json::Value = serde_json::from_str(&edit(&text, r#"{"op":"add","keyword":"component_exchange","name":"A to B","layer":"logical_architecture","attributes":[["from_port","\"LC-A.Data\""],["to_port","\"LC-B.Data\""]]}"#)).unwrap();
        assert_eq!(wired["success"], true, "{wired}");
        let text = wired["source"].as_str().unwrap();
        assert!(text.contains("component_exchange \"A to B\""));
        let output: serde_json::Value = serde_json::from_str(&compile(text)).unwrap();
        let lab = output["diagrams"]["graphs"].as_array().unwrap().iter().find(|g| g["id"] == "lab").unwrap();
        assert_eq!(lab["edges"].as_array().unwrap().len(), 1);
        assert!(serde_json::from_str::<serde_json::Value>(&edit(model, r#"{"op":"add","keyword":"port sideways","name":"X","container":"LC-A"}"#)).unwrap()["success"] == false);
    }

    #[test]
    fn an_element_declared_by_identifier_is_found_by_its_written_name() {
        let model = "logical_architecture LA {\n    component A {\n        id: \"LC-A\"\n        function Apply {\n            name: \"Apply braking force\"\n        }\n        function \"Bare one\"\n    }\n    component B { id: \"LC-B\" }\n}\n";
        let moved: serde_json::Value = serde_json::from_str(&edit(model, r#"{"op":"move","element":"Apply braking force","container":"LC-B"}"#)).unwrap();
        assert_eq!(moved["success"], true, "{moved}");
        let text = moved["source"].as_str().unwrap();
        assert!(text.find("function Apply").unwrap() > text.find("component B").unwrap(), "{text}");
        // A declaration with no block moves as its line.
        let bare: serde_json::Value = serde_json::from_str(&edit(model, r#"{"op":"move","element":"Bare one","container":"LC-B"}"#)).unwrap();
        assert_eq!(bare["success"], true, "{bare}");
        let text = bare["source"].as_str().unwrap();
        assert!(text.find("function \"Bare one\"").unwrap() > text.find("component B").unwrap(), "{text}");
        assert_eq!(text.matches("Bare one").count(), 1);
    }

    #[test]
    fn a_function_moves_to_another_component_and_keeps_its_identity() {
        let model = "logical_architecture LA {\n    component \"A\" {\n        id: \"LC-A\"\n        function \"Compute\" { id: \"LF-C\" description: \"kept\" }\n    }\n    component \"B\" { id: \"LC-B\" }\n}\ntrace \"LF-C\" realizes \"LC-B\"\n";
        let moved: serde_json::Value = serde_json::from_str(&edit(model, r#"{"op":"move","element":"LF-C","container":"LC-B"}"#)).unwrap();
        assert_eq!(moved["success"], true, "{moved}");
        let text = moved["source"].as_str().unwrap();
        let a = text.find("component \"A\"").unwrap();
        let b = text.find("component \"B\"").unwrap();
        let function = text.find("function \"Compute\" { id: \"LF-C\" description: \"kept\" }").expect("the declaration, as written");
        assert!(function > b && b > a, "{text}");
        let output: serde_json::Value = serde_json::from_str(&compile(text)).unwrap();
        assert_eq!(output["success"], true);
        let refused: serde_json::Value = serde_json::from_str(&edit(model, r#"{"op":"move","element":"LC-B","container":"LF-C"}"#)).unwrap();
        assert!(refused["success"] == false || refused["source"].as_str().is_some());
        let into_itself: serde_json::Value = serde_json::from_str(&edit(model, r#"{"op":"move","element":"LC-A","container":"LC-A"}"#)).unwrap();
        assert_eq!(into_itself["success"], false);
    }
}

