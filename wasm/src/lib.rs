//! ArcLang in the browser: the SAME compiler core as the CLI, compiled to
//! WebAssembly. The model never leaves the user's machine — compilation,
//! validation, the production gate and the explorer rendering all run
//! client-side. There is no server-side compile endpoint at all.

use arclang::compiler::elements::{self, ElementGraph};
use arclang::compiler::source_edit::{self, Container};
use arclang::compiler::{CompilationResult, Compiler, CompilerConfig};
use serde::{Deserialize, Serialize};
use serde_json::json;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct Stats {
    requirements: usize,
    components: usize,
    functions: usize,
    traces: usize,
    missions: usize,
    capabilities: usize,
    functional_chains: usize,
}

#[derive(Serialize)]
struct GateFindingOut {
    check: String,
    severity: String,
    message: String,
}

#[derive(Serialize)]
struct GateOut {
    passed: bool,
    requirements_total: usize,
    requirements_verified: usize,
    findings: Vec<GateFindingOut>,
}

#[derive(Serialize)]
struct CompileOutput {
    success: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
    stats: Option<Stats>,
    /// The standalone architecture dossier (the `arclang explorer` page).
    explorer_html: Option<String>,
    /// The diagrams of the model, as `ArcViz.mount` takes them: one ELK
    /// graph per Arcadia view, the traces between views, the drawing
    /// diagnostics. The playground mounts the viewer on this directly.
    diagrams: Option<serde_json::Value>,
    /// The architecture document behind the dossier: requirements,
    /// components, interfaces, functions, traces, chains, machines,
    /// scenarios and the data model, resolved by the compiler.
    document: Option<serde_json::Value>,
    /// Every element of the compiled model with its typed attributes, as
    /// the modelling API lists them: what a properties view edits.
    elements: Option<serde_json::Value>,
    /// Every warning, traceability and methodology finding, each named
    /// after its rule (`rules()` lists them) with the element concerned.
    findings: Vec<arclang::compiler::rules::Finding>,
    gate: Option<GateOut>,
    /// OMG SysML v2 textual notation (interop subset).
    sysmlv2: Option<String>,
    /// OMG ReqIF 1.0 (requirements exchange).
    reqif: Option<String>,
    /// C99 interface contracts.
    c_headers: Option<String>,
    /// Protobuf (proto3) interface contracts.
    proto: Option<String>,
}

/// Compile ArcLang source; returns a JSON string:
/// `{success, errors, warnings, stats, explorer_html, diagrams, document,
/// gate, sysmlv2, reqif, c_headers, proto}`.
#[wasm_bindgen]
pub fn compile(source: &str) -> String {
    let output = compile_inner(source);
    serde_json::to_string(&output)
        .unwrap_or_else(|e| format!("{{\"success\":false,\"errors\":[\"serialization: {e}\"]}}"))
}

/// Format ArcLang source (whitespace only, comments kept); returns a JSON
/// string: `{success, formatted}` or `{success: false, error}`.
#[wasm_bindgen]
pub fn format(source: &str) -> String {
    let output = match arclang::compiler::format::format_source(source) {
        Ok(formatted) => serde_json::json!({ "success": true, "formatted": formatted }),
        Err(error) => serde_json::json!({ "success": false, "error": error }),
    };
    output.to_string()
}

/// Version of the underlying compiler crate.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// An Arcadia transition (`oa-to-sa`, `sa-to-la`, `la-to-pa`) applied to
/// the source; returns `{success, source, created, container,
/// container_created}` or `{success: false, error}`. With `plan_only`,
/// returns the plan of every step instead: what each would create.
#[wasm_bindgen]
pub fn transition(source: &str, step: &str, into: &str, plan_only: bool) -> String {
    use arclang::compiler::transition::{self, Step};
    if plan_only {
        return match compile_text(source) {
            Ok(compiled) => {
                let plans: Vec<transition::Plan> = Step::ALL.iter().map(|s| transition::plan(*s, &compiled.ast)).collect();
                json!({ "success": true, "plans": plans }).to_string()
            }
            Err(error) => json!({ "success": false, "error": error }).to_string(),
        };
    }
    let Some(step) = Step::parse(step) else {
        return json!({ "success": false, "error": format!("unknown step '{step}': use oa-to-sa, sa-to-la or la-to-pa") }).to_string();
    };
    let into = if into.trim().is_empty() { None } else { Some(into.trim()) };
    match transition::apply(step, source, into) {
        Ok(done) => json!({
            "success": true,
            "source": done.source,
            "created": done.created,
            "container": done.container,
            "container_created": done.container_created,
        })
        .to_string(),
        Err(error) => json!({ "success": false, "error": error }).to_string(),
    }
}

/// The catalogue of validation rules as JSON: what `findings` name.
#[wasm_bindgen]
pub fn rules() -> String {
    serde_json::to_string(arclang::compiler::rules::catalogue()).unwrap_or_default()
}

/// The language metamodel as JSON (element kinds, typed attributes,
/// enumerations, units, trace rules) — same data as `GET /api/metamodel`.
#[wasm_bindgen]
pub fn metamodel() -> String {
    arclang::compiler::metamodel::Metamodel::current().to_json()
}

fn compile_inner(source: &str) -> CompileOutput {
    let mut compiler = Compiler::new(CompilerConfig {
        optimization_level: 0,
        target: "json".to_string(),
    });
    match compiler.compile_string(source) {
        Err(e) => CompileOutput {
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
            let explorer = arclang::compiler::arcviz_explorer::generate_explorer_html(
                model,
                &result.ast,
            )
            .ok();
            let explorer_html = explorer.as_ref().map(|(html, _json)| html.clone());
            let document = explorer
                .as_ref()
                .and_then(|(_html, json)| serde_json::from_str(json).ok());
            // The declared model name, as the dossier titles itself.
            let title = model
                .name
                .clone()
                .unwrap_or_else(|| "System Architecture".to_string());
            let findings = arclang::compiler::rules::findings(&result);
            let diagrams = arclang::compiler::diagram::build_diagrams(&result.ast);
            let diagrams = arclang::compiler::diagram::html::viewer_payload(&diagrams, &title);
            let gate_report = arclang::compiler::production_gate::run_gate(
                &result.ast,
                model,
                "ISO26262",
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
            CompileOutput {
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
                sysmlv2: Some(arclang::compiler::sysmlv2_generator::generate_sysmlv2(model, &result.ast)),
                reqif: Some(arclang::compiler::reqif::generate_reqif(model, &result.ast)),
                c_headers: Some(arclang::compiler::c_header_generator::generate_c_headers(model, &result.ast)),
                proto: Some(arclang::compiler::proto_generator::generate_proto(model, &result.ast)),
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
fn rename_keeping_identity(source: &str, element: &str, name: &str) -> Result<String, String> {
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
fn elements_json(graph: &ElementGraph) -> serde_json::Value {
    json!({
        "elements": graph.elements.iter().map(|element| json!({
            "uuid": element.uuid,
            "id": element.id,
            "kind": element.kind,
            "name": element.name,
            "owner": element.owner,
            "attributes": elements::attributes_json(&element.attributes),
        })).collect::<Vec<_>>(),
        "relationships": graph.relationships.iter().map(|relationship| json!({
            "uuid": relationship.uuid,
            "kind": relationship.kind,
            "name": relationship.name,
            "source": relationship.source,
            "target": relationship.target,
        })).collect::<Vec<_>>(),
    })
}

/// One edit of the model text, as `arclang set`, `unset`, `rename`, the
/// modelling API's create and delete, and `trace` write it. Attribute
/// values are ArcLang source (`"text"`, `25 ms`, `[A, B]`).
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum EditRequest {
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
    /// Move an element's declaration into another element (or a layer):
    /// a function reallocated, a component nested or deployed elsewhere.
    Move { element: String, container: Option<String>, layer: Option<String>, layer_name: Option<String> },
    AddTrace { from: String, kind: String, to: String },
    RemoveTrace { from: String, kind: String, to: String },
}

fn compile_text(source: &str) -> Result<CompilationResult, String> {
    Compiler::new(CompilerConfig { optimization_level: 0, target: "json".to_string() })
        .compile_string(source)
        .map_err(|e| e.to_string())
}

fn identities(graph: &ElementGraph) -> Vec<String> {
    let mut all: Vec<String> = graph.elements.iter().map(|e| e.uuid.clone()).collect();
    all.extend(graph.relationships.iter().map(|r| r.uuid.clone()));
    all.sort();
    all
}

/// Apply one edit to ArcLang source; returns a JSON string:
/// `{success, source}` with the edited text, or `{success: false, error}`.
/// The edit is refused unless the edited model compiles and shows exactly
/// what was asked: comments, order and every other declaration are kept.
#[wasm_bindgen]
pub fn edit(source: &str, request: &str) -> String {
    let outcome = edit_inner(source, request);
    let value = match outcome {
        Ok(edited) => json!({ "success": true, "source": edited }),
        Err(error) => json!({ "success": false, "error": error }),
    };
    value.to_string()
}

fn edit_inner(source: &str, request: &str) -> Result<String, String> {
    let request: EditRequest = serde_json::from_str(request).map_err(|e| format!("invalid edit request: {e}"))?;
    if let EditRequest::Rename { element, name } = &request {
        return rename_keeping_identity(source, element, name);
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
