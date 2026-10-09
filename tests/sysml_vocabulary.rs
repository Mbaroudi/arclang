//! The SysML v2 vocabulary: what ArcLang reads back from its own SysML v2
//! export, checked against listings frozen from the OMG pilot
//! implementation (`tools/sysml_abstract_syntax_check.py --freeze`), and
//! served under /api/sysml-v2.

use arclang::compiler::sysml_records;
use arclang::compiler::sysmlv2_generator::generate_sysmlv2;
use arclang::web_server::systems_modeling::Workspace;
use arclang::{Compiler, CompilerConfig};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tower::util::ServiceExt;

/// An expression as `tools/sysml_abstract_syntax_check.py` spells it.
fn expression(by: &HashMap<&str, &Value>, reference: &Value) -> String {
    let node = by[reference["@id"].as_str().unwrap()];
    let qualified = |reference: &Value| by[reference["@id"].as_str().unwrap()]["qualifiedName"].as_str().unwrap().to_string();
    let arguments: Vec<String> = node["argument"].as_array().map(|a| a.iter().map(|r| expression(by, r)).collect()).unwrap_or_default();
    let metaclass = node["@type"].as_str().unwrap();
    match metaclass {
        "FeatureReferenceExpression" => match node["referent"].is_null() {
            true => format!("ref(?{})", node["arclang:unresolvedTarget"].as_str().unwrap_or_default()),
            false => format!("ref({})", qualified(&node["referent"])),
        },
        "FeatureChainExpression" => {
            let written = node["arclang:targetPath"].as_str().unwrap_or_default();
            let path = match (written.contains('.'), node["targetFeature"].is_null()) {
                (true, _) => written.to_string(),
                (false, false) => qualified(&node["targetFeature"]),
                (false, true) => format!("?{written}"),
            };
            format!("chain({}, {path})", arguments[0])
        }
        "InvocationExpression" => {
            let function = if node["instantiatedType"].is_null() { "?".to_string() } else { qualified(&node["instantiatedType"]) };
            format!("call({})", [vec![function], arguments].concat().join(", "))
        }
        "OperatorExpression" => {
            let operator = node["operator"].as_str().unwrap().to_string();
            format!("op({})", [vec![operator], arguments].concat().join(", "))
        }
        literal => format!("{}({})", literal.trim_start_matches("Literal"), node["value"]),
    }
}

/// The listing `tools/sysml_abstract_syntax_check.py` computes, for ArcLang
/// records: one line per declared element, in ownership order.
fn canonical(records: &[Value]) -> Vec<String> {
    let by: HashMap<&str, &Value> = records.iter().map(|r| (r["@id"].as_str().unwrap(), r)).collect();
    let get = |reference: &Value| by[reference["@id"].as_str().unwrap()];
    let qualified = |reference: &Value| {
        let element = get(reference);
        element["qualifiedName"].as_str().map(str::to_string).unwrap_or(format!("<{}>", element["@type"].as_str().unwrap()))
    };
    let end = |element: &Value, role: &str| {
        let related: Vec<String> = element[role].as_array().map(|r| r.iter().map(&qualified).collect()).unwrap_or_default();
        if related.is_empty() {
            element[format!("arclang:{role}Path")].as_str().unwrap_or_default().to_string()
        } else {
            related.join(",")
        }
    };
    let line = |element: &Value, is_root: bool| {
        let metaclass = element["@type"].as_str().unwrap();
        let membership = if is_root { "-" } else { get(&element["owningMembership"])["@type"].as_str().unwrap() };
        let mut parts = vec![
            metaclass.to_string(),
            format!("name={}", element["declaredName"].as_str().unwrap_or("-")),
            format!("short={}", element["declaredShortName"].as_str().unwrap_or("-")),
            format!("via={membership}"),
        ];
        if let Some(direction) = element["direction"].as_str() {
            parts.push(format!("dir={direction}"));
        }
        if element["isAbstract"] == true {
            parts.push("abstract".to_string());
        }
        let mut relationships = Vec::new();
        for reference in element["ownedRelationship"].as_array().unwrap() {
            let relationship = get(reference);
            let symbol = match relationship["@type"].as_str().unwrap() {
                "FeatureTyping" => ":",
                "Subclassification" | "Subsetting" => ":>",
                "Redefinition" => ":>>",
                "ReferenceSubsetting" => "references",
                _ => continue,
            };
            let target = match relationship["arclang:unresolvedTarget"].as_str() {
                Some(name) => format!("?{name}"),
                None => relationship["target"].as_array().unwrap().iter().map(&qualified).collect::<Vec<_>>().join(","),
            };
            relationships.push(format!("{symbol} {target}"));
        }
        relationships.sort();
        parts.extend(relationships);
        for reference in element["ownedRelationship"].as_array().unwrap() {
            let relationship = get(reference);
            match relationship["@type"].as_str().unwrap() {
                "FeatureValue" => {
                    let binding = if relationship["isDefault"] == true { "default=" } else { "=" };
                    parts.push(format!("{binding} {}", expression(&by, &relationship["value"])));
                }
                "ResultExpressionMembership" => {
                    parts.push(format!("result {}", expression(&by, &relationship["ownedResultExpression"])));
                }
                _ => {}
            }
        }
        match metaclass {
            "SatisfyRequirementUsage" => parts.push(format!(
                "req={} by={}",
                qualified(&element["satisfiedRequirement"]),
                qualified(&element["satisfyingFeature"])
            )),
            "Dependency" => parts.push(format!("client={} supplier={}", end(element, "source"), end(element, "target"))),
            _ if element.get("arclang:sourcePath").is_some() => {
                parts.push(format!("src={} tgt={}", end(element, "source"), end(element, "target")))
            }
            _ => {}
        }
        if metaclass == "MultiplicityRange" {
            let bounds: Vec<String> = element["bound"].as_array().unwrap().iter().map(|b| expression(&by, b)).collect();
            parts.push(format!("bounds={}", bounds.join(",")));
        }
        if metaclass == "Documentation" {
            let body: Vec<&str> = element["body"].as_str().unwrap_or_default().split_whitespace().collect();
            parts.push(format!("body={}", json!(body.join(" "))));
        }
        parts.join(" ")
    };

    fn show<'a>(
        element: &'a Value,
        depth: usize,
        lines: &mut Vec<String>,
        get: &dyn Fn(&Value) -> &'a Value,
        line: &dyn Fn(&Value, bool) -> String,
    ) {
        let unnamed_reference = element["@type"] == "ReferenceUsage" && element["declaredName"].is_null();
        if unnamed_reference {
            return;
        }
        lines.push(format!("{}{}", "  ".repeat(depth), line(element, depth == 0)));
        let members: Vec<&'a Value> = element["ownedMember"].as_array().unwrap().iter().map(|m| get(m)).collect();
        for member in members.iter().filter(|m| m["@type"] != "Dependency") {
            show(member, depth + 1, lines, get, line);
        }
        let start = lines.len();
        for member in members.iter().filter(|m| m["@type"] == "Dependency") {
            show(member, depth + 1, lines, get, line);
        }
        lines[start..].sort();
    }

    let mut lines = Vec::new();
    for root in records.iter().filter(|r| r["@type"] == "Package" && r["owner"].is_null()) {
        show(root, 0, &mut lines, &|reference| get(reference), &line);
    }
    lines
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sysml_abstract_syntax")
}

#[test]
fn the_abstract_syntax_agrees_with_the_omg_pilot_on_the_frozen_references() {
    let mut checked = 0;
    for name in ["expression_forms", "timing_constraints", "complete_emergency_braking_simple", "level_crossing_protection"] {
        let text = std::fs::read_to_string(fixtures().join(format!("{name}.sysml"))).unwrap();
        let pilot = std::fs::read_to_string(fixtures().join(format!("{name}.pilot.txt"))).unwrap();
        let rendered = sysml_records::from_text(&text).unwrap_or_else(|e| panic!("{name}: {e}"));

        let ours = canonical(&rendered.records);
        let theirs: Vec<&str> = pilot.lines().collect();

        assert_eq!(ours.len(), theirs.len(), "{name}: number of declared elements");
        for (number, (ours, theirs)) in ours.iter().zip(&theirs).enumerate() {
            assert_eq!(ours.as_str(), *theirs, "{name}, declared element {}", number + 1);
        }
        checked += theirs.len();
    }
    assert!(checked > 600, "only {checked} elements compared");
}

fn arc_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            arc_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "arc") {
            files.push(path);
        }
    }
}

#[test]
fn every_example_that_compiles_has_a_consistent_sysml_view() {
    let mut files = Vec::new();
    arc_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("examples"), &mut files);
    files.sort();
    let mut read = 0;
    for path in files {
        let Ok(result) = Compiler::new(CompilerConfig::default()).compile_file(&path) else { continue };
        let text = generate_sysmlv2(&result.semantic_model, &result.ast);
        let rendered = sysml_records::from_text(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));

        // Identities are unique, and every reference leads to a record.
        let ids: HashSet<&str> = rendered.records.iter().map(|r| r["@id"].as_str().unwrap()).collect();
        assert_eq!(ids.len(), rendered.records.len(), "{}: duplicate identities", path.display());
        fn references<'v>(value: &'v Value, found: &mut Vec<&'v str>) {
            match value {
                Value::Object(object) => {
                    if let (1, Some(id)) = (object.len(), object.get("@id").and_then(Value::as_str)) {
                        found.push(id);
                    }
                    object.values().for_each(|inner| references(inner, found));
                }
                Value::Array(items) => items.iter().for_each(|inner| references(inner, found)),
                _ => {}
            }
        }
        let mut referenced = Vec::new();
        rendered.records.iter().for_each(|record| references(record, &mut referenced));
        for id in referenced {
            assert!(ids.contains(id), "{}: dangling reference {id}", path.display());
        }
        // One root, and every relationship end is a record.
        assert_eq!(rendered.roots.len(), 1, "{}", path.display());
        for (index, source, target) in &rendered.ends {
            assert!(*index < rendered.records.len() && ids.contains(source.as_str()) && ids.contains(target.as_str()));
        }
        // Same model, same records.
        assert_eq!(rendered.records, sysml_records::from_text(&text).unwrap().records);
        read += 1;
    }
    assert!(read >= 15, "only {read} examples compile");
}

const MODEL: &str = r#"
model Brakes {}
system_analysis "Braking" {
    requirement "REQ-1" { description: "Stop in time" priority: "High" }
    function "Decide" { id: "SF-1" latency: 10 ms }
}
logical_architecture "Logical" {
    component "Controller" { id: "LC-1" safety_level: "ASIL-B" }
    component "Actuator" { id: "LC-2" }
}
physical_architecture "Physical" {
    node "ECU" { id: "PN-1" }
}
trace "LC-1" satisfies "REQ-1" {}
"#;

async fn get(base: &str, path: &str) -> (StatusCode, Value) {
    let mut workspace = Workspace::default();
    workspace.add_source(MODEL, chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap()).unwrap();
    let app = arclang::web_server::build_router_with(workspace);
    let response = app.oneshot(Request::builder().uri(format!("{base}{path}")).body(Body::empty()).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

const SYSML: &str = "/api/sysml-v2";
const ARCLANG: &str = "/api/systems-modeling";

async fn head(base: &str) -> String {
    let (_, projects) = get(base, "/projects").await;
    let project = projects[0]["@id"].as_str().unwrap().to_string();
    let (_, commits) = get(base, &format!("/projects/{project}/commits")).await;
    format!("/projects/{project}/commits/{}", commits[0]["@id"].as_str().unwrap())
}

#[tokio::test]
async fn both_vocabularies_serve_the_same_projects_and_commits() {
    assert_eq!(head(SYSML).await, head(ARCLANG).await);
}

#[tokio::test]
async fn the_sysml_base_path_serves_sysml_metaclasses() {
    let commit = head(SYSML).await;
    let (status, elements) = get(SYSML, &format!("{commit}/elements")).await;
    assert_eq!(status, StatusCode::OK);
    let elements = elements.as_array().unwrap();
    let find = |metaclass: &str, short: &str| {
        elements
            .iter()
            .find(|e| e["@type"] == metaclass && e["declaredShortName"] == short)
            .unwrap_or_else(|| panic!("no {metaclass} <{short}>"))
    };

    let controller = find("PartDefinition", "LC-1");
    let requirement = find("RequirementUsage", "REQ-1");
    let function = find("ActionDefinition", "SF-1");
    assert_eq!(controller["qualifiedName"], "Brakes::Controller");
    assert_eq!(function["declaredName"], "Decide");
    assert_eq!(requirement["name"], "req_REQ_1");

    // The ArcLang vocabulary has none of these metaclasses.
    let (_, arclang) = get(ARCLANG, &format!("{commit}/elements")).await;
    assert!(arclang.as_array().unwrap().iter().all(|e| e["@type"] != "PartDefinition"));

    // The root is the model package, and it is the only root.
    let (_, roots) = get(SYSML, &format!("{commit}/roots")).await;
    assert_eq!(roots.as_array().unwrap().len(), 1);
    assert_eq!(roots[0]["@type"], "Package");
    assert_eq!(roots[0]["declaredName"], "Brakes");
}

#[tokio::test]
async fn a_sysml_element_links_back_to_the_arclang_element_it_comes_from() {
    let commit = head(SYSML).await;
    let (_, elements) = get(SYSML, &format!("{commit}/elements")).await;
    let controller = elements.as_array().unwrap().iter().find(|e| e["declaredShortName"] == "LC-1").unwrap();
    let source = controller["arclang:element"]["@id"].as_str().expect("link to the ArcLang element");

    let (status, arclang) = get(ARCLANG, &format!("{commit}/elements/{source}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(arclang["@type"], "LogicalComponent");
    assert_eq!(arclang["shortName"], "LC-1");
}

#[tokio::test]
async fn ownership_typing_and_values_are_navigable() {
    let commit = head(SYSML).await;
    let (_, elements) = get(SYSML, &format!("{commit}/elements")).await;
    let elements = elements.as_array().unwrap();
    let by_id = |id: &Value| elements.iter().find(|e| e["@id"] == id["@id"]).unwrap();

    let function = elements.iter().find(|e| e["declaredShortName"] == "SF-1").unwrap();
    let latency = function["ownedMember"].as_array().unwrap().iter().map(by_id).find(|m| m["declaredName"] == "latency").unwrap();
    assert_eq!(latency["@type"], "AttributeUsage");
    assert_eq!(latency["owner"], json!({ "@id": function["@id"] }));
    assert_eq!(latency["arclang:expression"], "10 [ms]");
    assert_eq!(by_id(&latency["owningMembership"])["@type"], "FeatureMembership");

    // `: DurationValue` is a standard-library type, served as a stub.
    let typing = latency["ownedRelationship"].as_array().unwrap().iter().map(by_id).find(|r| r["@type"] == "FeatureTyping").unwrap();
    let duration = by_id(&typing["target"][0]);
    assert_eq!(duration["qualifiedName"], "ISQBase::DurationValue");
    assert_eq!(duration["@type"], "AttributeDefinition");
    assert_eq!(duration["isLibraryElement"], true);

    // `= 10 [ms]` is a quantity expression: 10, in the unit the model declares.
    let binding = latency["ownedRelationship"].as_array().unwrap().iter().map(by_id).find(|r| r["@type"] == "FeatureValue").unwrap();
    let quantity = by_id(&binding["value"]);
    assert_eq!((&quantity["@type"], &quantity["operator"]), (&json!("OperatorExpression"), &json!("[")));
    let operands: Vec<&Value> = quantity["argument"].as_array().unwrap().iter().map(by_id).collect();
    assert_eq!((&operands[0]["@type"], &operands[0]["value"]), (&json!("LiteralInteger"), &json!(10)));
    assert_eq!(operands[1]["@type"], "FeatureReferenceExpression");
    assert_eq!(by_id(&operands[1]["referent"])["qualifiedName"], "Brakes::ArcLangUnits::millisecond");

    // The usage is typed by the definition, inside the model.
    let usage = elements.iter().find(|e| e["@type"] == "ActionUsage" && e["declaredName"] == "a_SF_1").unwrap();
    let typing = usage["ownedRelationship"].as_array().unwrap().iter().map(by_id).find(|r| r["@type"] == "FeatureTyping").unwrap();
    assert_eq!(typing["target"], json!([{ "@id": function["@id"] }]));
}

#[test]
fn every_library_name_the_exporter_can_write_is_in_the_index() {
    // The probe uses them all; whatever it does not declare itself must be
    // a library element the pilot has indexed.
    let probe = arclang::compiler::sysmlv2_generator::library_probe();
    let rendered = sysml_records::from_text(&probe).expect("the probe reads");
    let unresolved: Vec<&str> = rendered.records.iter().filter_map(|r| r["arclang:unresolvedTarget"].as_str()).collect();
    assert!(
        unresolved.is_empty(),
        "not in spec/sysml_library_index.json: {unresolved:?}; regenerate it with tools/sysml_library_index.py"
    );
    let library = rendered.records.iter().filter(|r| r["isLibraryElement"] == true).count();
    assert!(library >= 60, "only {library} library elements referred to by the probe");
}

#[tokio::test]
async fn relationships_of_an_element_include_what_satisfies_it() {
    let commit = head(SYSML).await;
    let (_, elements) = get(SYSML, &format!("{commit}/elements")).await;
    let elements = elements.as_array().unwrap();
    let requirement = elements.iter().find(|e| e["declaredShortName"] == "REQ-1").unwrap();
    let id = requirement["@id"].as_str().unwrap();

    let (status, incoming) = get(SYSML, &format!("{commit}/elements/{id}/relationships?direction=in")).await;

    assert_eq!(status, StatusCode::OK);
    let satisfy = incoming.as_array().unwrap().iter().find(|r| r["@type"] == "SatisfyRequirementUsage").expect("a satisfy");
    assert_eq!(satisfy["satisfiedRequirement"], json!({ "@id": id }));
    let by = elements.iter().find(|e| e["@id"] == satisfy["satisfyingFeature"]["@id"]).unwrap();
    assert_eq!(by["declaredName"], "p_LC_1");
}

#[tokio::test]
async fn the_sysml_view_is_read_only() {
    let commit = head(SYSML).await;
    let project = commit.split("/commits/").next().unwrap().to_string();
    let mut workspace = Workspace::default();
    workspace.add_source(MODEL, chrono::Utc::now()).unwrap();
    workspace.require_token("0123456789abcdef0123456789abcdef").unwrap();
    workspace.allow_writes().unwrap();
    let app = arclang::web_server::build_router_with(workspace);

    let request = Request::builder()
        .method("POST")
        .uri(format!("{SYSML}{project}/commits"))
        .header("authorization", "Bearer 0123456789abcdef0123456789abcdef")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "change": [] }).to_string()))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}
