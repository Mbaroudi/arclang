//! ArcLang -> SysML v2 text -> ArcLang: what comes back. The text is read
//! by the complete parser (`sysml_parser`), as `arclang import -f sysml`
//! reads any SysML v2 text, and rendered as API records for the import. The
//! element graph of the model compiled before and after is compared by
//! (kind, id) for elements and (kind, source, target) for relationships,
//! kind by kind, so a loss names what the export or the import drops.

use arclang::compiler::elements;
use arclang::compiler::{sysml_import, sysml_parser, sysmlv2_generator, Compiler, CompilerConfig};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn compile(source: &str) -> arclang::CompilationResult {
    Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles")
}

/// (kind, id) of every element, (kind, source id, target id) of every relationship.
fn shape(result: &arclang::CompilationResult) -> (BTreeSet<(String, String)>, BTreeSet<(String, String, String)>) {
    let graph = elements::build(&result.ast, &result.semantic_model);
    let by_uuid: BTreeMap<&str, &str> = graph.elements.iter().map(|e| (e.uuid.as_str(), e.id.as_str())).collect();
    let elements = graph.elements.iter().map(|e| (e.kind.to_string(), e.id.clone())).collect();
    let relationships = graph
        .relationships
        .iter()
        .filter_map(|r| Some((r.kind.to_string(), by_uuid.get(r.source.as_str())?.to_string(), by_uuid.get(r.target.as_str())?.to_string())))
        .collect();
    (elements, relationships)
}

struct Measure {
    kept_elements: usize,
    total_elements: usize,
    kept_relationships: usize,
    total_relationships: usize,
    lost_by_kind: BTreeMap<String, usize>,
    /// Lost relationships by kind: their (source, target) pairs, to name them.
    lost_relationships_by_kind: BTreeMap<String, Vec<(String, String)>>,
    /// What an element or a relationship that comes back no longer says, or says
    /// otherwise: by `Kind.key`, the ids it is lost on.
    lost_values: BTreeMap<String, Vec<String>>,
}

/// What each element and relationship says: its attributes and its kind-specific data
/// (a transition's guard, a machine's initial state), by (kind, id) and by key.
fn values(result: &arclang::CompilationResult) -> BTreeMap<(String, String), BTreeMap<String, serde_json::Value>> {
    let graph = elements::build(&result.ast, &result.semantic_model);
    let by_uuid: BTreeMap<&str, &str> = graph.elements.iter().map(|e| (e.uuid.as_str(), e.id.as_str())).collect();
    let said = |attributes: &std::collections::HashMap<String, arclang::compiler::ast::AttributeValue>, extra: &BTreeMap<&'static str, serde_json::Value>| {
        let mut said: BTreeMap<String, serde_json::Value> = attributes.iter().map(|(key, value)| (key.clone(), elements::attribute_json(value))).collect();
        said.extend(extra.iter().map(|(key, value)| (key.to_string(), value.clone())));
        said
    };
    let mut all = BTreeMap::new();
    for element in &graph.elements {
        let mut said = said(&element.attributes, &element.extra);
        said.insert("(name)".to_string(), serde_json::json!(element.name));
        all.insert((element.kind.to_string(), element.id.clone()), said);
    }
    for relationship in &graph.relationships {
        let (Some(source), Some(target)) = (by_uuid.get(relationship.source.as_str()), by_uuid.get(relationship.target.as_str())) else { continue };
        // Two transitions between the same states are told apart by what they say: they are compared together.
        let entry: &mut BTreeMap<String, serde_json::Value> = all.entry((relationship.kind.to_string(), format!("{source} -> {target}"))).or_default();
        for (key, value) in said(&relationship.attributes, &relationship.extra) {
            match entry.get_mut(&key) {
                Some(serde_json::Value::Array(list)) if list.first().is_some_and(|first| first.get("@several").is_some()) => list.push(serde_json::json!({ "@several": value })),
                Some(other) => *other = serde_json::json!([{ "@several": other.clone() }, { "@several": value }]),
                None => {
                    entry.insert(key, value);
                }
            }
        }
    }
    all
}

/// The model of a file, and the model its SysML v2 export is imported as.
fn there_and_back(path: &Path) -> (arclang::CompilationResult, arclang::CompilationResult) {
    let source = std::fs::read_to_string(path).unwrap();
    let before = compile(&source);
    let text = sysmlv2_generator::generate_sysmlv2(&before.semantic_model, &before.ast);
    let built = sysml_parser::records(&text).unwrap_or_else(|e| panic!("{}: the export does not parse: {e}", path.display()));
    let imported = sysml_import::import_records(&built.records).unwrap_or_else(|f| panic!("{}: {}\n{}", path.display(), f.message, f.rejected.unwrap_or_default()));
    let after = compile(&imported.source);
    (before, after)
}

fn round_trip(path: &Path) -> Measure {
    let (before, after) = there_and_back(path);
    let (elements_before, relationships_before) = shape(&before);
    let (elements_after, relationships_after) = shape(&after);
    let mut lost_by_kind: BTreeMap<String, usize> = BTreeMap::new();
    for (kind, id) in elements_before.difference(&elements_after) {
        // The same id under another kind is a kind confusion, not a loss of the element.
        let kept_elsewhere = elements_after.iter().any(|(_, other)| other == id);
        *lost_by_kind.entry(if kept_elsewhere { format!("{kind} (kind changed)") } else { kind.clone() }).or_insert(0) += 1;
    }
    let mut lost_relationships_by_kind: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (kind, source, target) in relationships_before.difference(&relationships_after) {
        lost_relationships_by_kind.entry(kind.clone()).or_default().push((source.clone(), target.clone()));
    }
    let values_after = values(&after);
    let ids = ids_by_name(&before);
    let mut lost_values: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for ((kind, id), said) in values(&before) {
        let Some(back) = values_after.get(&(kind.clone(), id.clone())) else { continue };
        for (key, value) in said {
            // The id and the name are what the element is compared by; an order is the importer's.
            // An element named by its name comes back named by its id: the same element.
            let kept = |other: &serde_json::Value| if key == "(name)" { other == &value } else { same(&by_id(other, &ids), &by_id(&value, &ids)) };
            if matches!(key.as_str(), "id" | "name") || back.get(&key).is_some_and(kept) {
                continue;
            }
            lost_values.entry(format!("{kind}.{key}")).or_default().push(id.clone());
        }
    }
    Measure {
        lost_values,
        lost_relationships_by_kind,
        kept_elements: elements_before.intersection(&elements_after).count(),
        total_elements: elements_before.len(),
        kept_relationships: relationships_before.intersection(&relationships_after).count(),
        total_relationships: relationships_before.len(),
        lost_by_kind,
    }
}

/// The id of each element that one name names, and no other element.
fn ids_by_name(result: &arclang::CompilationResult) -> BTreeMap<String, String> {
    let graph = elements::build(&result.ast, &result.semantic_model);
    let mut ids: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for element in graph.elements.iter().filter(|e| !e.name.is_empty()) {
        ids.entry(element.name.clone()).or_default().insert(element.id.clone());
    }
    ids.into_iter().filter(|(_, ids)| ids.len() == 1).map(|(name, ids)| (name, ids.into_iter().next().unwrap())).collect()
}

/// The value with each element it names written by its id.
fn by_id(value: &serde_json::Value, ids: &BTreeMap<String, String>) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::String(text) => Value::String(ids.get(text).cloned().unwrap_or_else(|| text.clone())),
        Value::Array(list) => Value::Array(list.iter().map(|one| by_id(one, ids)).collect()),
        Value::Object(map) => Value::Object(map.iter().map(|(key, one)| (key.clone(), by_id(one, ids))).collect()),
        other => other.clone(),
    }
}

/// The same value: numbers within what a text keeps of them, lists and blocks member by member.
/// A quantity is the same in another unit (`-10 °C` comes back as `263.15 K`, `50 mΩ` as `50 mOhm`).
fn same(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    use serde_json::Value;
    let quantity = |value: &Value| (value.get("@type")? == "Quantity").then(|| (value.get("dimension").cloned(), value.get("canonicalValue").cloned()));
    if let (Some((dimension, x)), Some((other, y))) = (quantity(a), quantity(b)) {
        return dimension == other && matches!((x, y), (Some(x), Some(y)) if same(&x, &y));
    }
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(x), Some(y)) => (x - y).abs() <= 1e-9 * x.abs().max(y.abs()),
            _ => x == y,
        },
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(x, y)| same(x, y)),
        (Value::Object(x), Value::Object(y)) => x.len() == y.len() && x.iter().all(|(key, x)| y.get(key).is_some_and(|y| same(x, y))),
        _ => a == b,
    }
}

fn report(name: &str, m: &Measure) {
    println!(
        "ROUNDTRIP {name}: elements {}/{} ({}%), relationships {}/{} ({}%)",
        m.kept_elements,
        m.total_elements,
        100 * m.kept_elements / m.total_elements.max(1),
        m.kept_relationships,
        m.total_relationships,
        100 * m.kept_relationships / m.total_relationships.max(1)
    );
    for (kind, n) in &m.lost_by_kind {
        println!("ROUNDTRIP   lost {n:4} {kind}");
    }
    for (kind, pairs) in &m.lost_relationships_by_kind {
        let shown: Vec<String> = pairs.iter().take(3).map(|(s, t)| format!("{s} -> {t}")).collect();
        println!("ROUNDTRIP   lost relationship {:4} {kind}: {}{}", pairs.len(), shown.join(", "), if pairs.len() > 3 { ", ..." } else { "" });
    }
    for (key, ids) in &m.lost_values {
        println!("ROUNDTRIP   lost value {:4} {key}: {}{}", ids.len(), ids.iter().take(3).cloned().collect::<Vec<_>>().join(", "), if ids.len() > 3 { ", ..." } else { "" });
    }
}

#[test]
fn the_flagship_example_comes_back_from_sysml() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/automotive/emergency-braking/complete_emergency_braking_simple.arc");
    let m = round_trip(&path);
    report("complete_emergency_braking_simple", &m);
    assert!(m.kept_elements == m.total_elements, "{} of {} elements come back", m.kept_elements, m.total_elements);
    assert!(m.kept_relationships == m.total_relationships, "{} of {} relationships come back", m.kept_relationships, m.total_relationships);
}

#[test]
fn the_capella_ife_example_comes_back_whole() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/capella/in-flight-entertainment/in_flight_entertainment.arc");
    let m = round_trip(&path);
    report("in_flight_entertainment", &m);
    assert!(m.kept_elements == m.total_elements, "{} of {} elements come back", m.kept_elements, m.total_elements);
    assert!(m.kept_relationships == m.total_relationships, "{} of {} relationships come back", m.kept_relationships, m.total_relationships);
}

/// What the behaviour of a model says and no longer does after the round trip.
fn lost_behaviour(m: &Measure) -> Vec<String> {
    m.lost_values
        .iter()
        .filter(|(key, _)| ["StateMachine.", "State.", "Transition.", "TestCase.execute", "TestCase.expects"].iter().any(|kind| key.starts_with(kind)))
        .map(|(key, ids)| format!("{key} on {}", ids.join(", ")))
        .collect()
}

#[test]
fn the_machines_of_a_model_come_back_with_what_they_compute() {
    // A block without states (its equations), a machine with guards on its
    // variables, and the cases that execute them: what `arclang simulate`
    // and the embedded stages run.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/automotive/speed-monitor/speed_monitor.arc");
    let m = round_trip(&path);
    report("speed_monitor", &m);
    assert!(m.kept_elements == m.total_elements, "{} of {} elements come back: {:?}", m.kept_elements, m.total_elements, m.lost_by_kind);
    assert!(m.kept_relationships == m.total_relationships, "{} of {} relationships come back: {:?}", m.kept_relationships, m.total_relationships, m.lost_relationships_by_kind);
    assert_eq!(lost_behaviour(&m), Vec::<String>::new());
}

#[test]
fn the_actions_of_states_and_transitions_come_back() {
    // Entry and exit actions, triggers, guards and actions of transitions, nested states.
    for example in ["automotive/body-controller/body_controller.arc", "automotive/power-window/power_window.arc", "energy/battery-charger/battery_charger.arc"] {
        let m = round_trip(&Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(example));
        report(example, &m);
        assert_eq!(lost_behaviour(&m), Vec::<String>::new(), "{example}");
        assert!(m.lost_by_kind.keys().all(|kind| !kind.starts_with("State")), "{example}: {:?}", m.lost_by_kind);
        assert!(!m.lost_relationships_by_kind.contains_key("Transition"), "{example}: {:?}", m.lost_relationships_by_kind.get("Transition"));
    }
}

/// The regions of every machine and of every state: where each is written, the state it is
/// entered by, its states and how many transitions it holds.
fn regions(result: &arclang::CompilationResult) -> BTreeSet<(String, String, String, Vec<String>, usize)> {
    use arclang::compiler::ast::{Region, State};
    fn of(owner: &str, regions: &[Region], states: &[State], all: &mut BTreeSet<(String, String, String, Vec<String>, usize)>) {
        for region in regions {
            all.insert((owner.to_string(), region.name.clone(), region.initial.clone(), region.states.iter().map(|state| state.name.clone()).collect(), region.transitions.len()));
        }
        for state in states {
            of(&format!("{owner}.{}", state.name), &state.regions, &state.sub_states, all);
        }
    }
    let mut all = BTreeSet::new();
    for machine in &result.ast.state_machines {
        of(&machine.name, &machine.regions, &machine.states, &mut all);
    }
    all
}

#[test]
fn the_regions_of_a_machine_come_back_and_its_cases_read_the_same() {
    // A machine is in each of its regions at once, entered by the initial state of each.
    // Flattened, the fan of the charger never leaves the state it is not entered by, and
    // the case that verifies it fails on a model that says the same thing.
    for example in ["energy/battery-charger/battery_charger.arc", "automotive/power-window/power_window.arc"] {
        let (before, after) = there_and_back(&Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(example));
        assert!(!regions(&before).is_empty(), "{example} has regions");
        assert_eq!(regions(&before), regions(&after), "{example}");
        let verdicts = |result: &arclang::CompilationResult| result.warnings.iter().filter(|warning| warning.contains("verification:")).cloned().collect::<BTreeSet<String>>();
        assert_eq!(verdicts(&before), verdicts(&after), "{example}");
    }
}

/// The transitions and the successions a SysML v2 text writes directly in a `parallel` state,
/// which the language does not allow there.
fn written_in_a_parallel_state(text: &str) -> Vec<String> {
    let mut open: Vec<bool> = Vec::new();
    let mut found = Vec::new();
    for line in text.lines().map(str::trim) {
        if open.last() == Some(&true) && (line.starts_with("transition ") || line.starts_with("entry;") || line.starts_with("first ")) {
            found.push(line.to_string());
        }
        if line.ends_with('{') {
            open.push(line.starts_with("state ") && line.contains(" parallel"));
        } else if line == "}" {
            open.pop();
        }
    }
    found
}

#[test]
fn a_transition_written_beside_regions_is_valid_sysml_and_comes_back_beside_them() {
    // A machine, and a state, whose transition joins a state of one region to a state of the
    // other and is written beside both.
    let source = r#"model T { name: "t" }
requirements system { req "R" "r" { description: "d" } }
state_machine "Two" {
  region "A" { initial: "A1" state A1 {} state A2 {} transition A1 -> A2 { trigger: "go" } }
  region "B" { initial: "B1" state B1 {} state B2 {} transition B1 -> B2 { trigger: "go" } }
  transition B2 -> A1 { id: "T-JUMP" trigger: "jump" }
}
state_machine "Held" {
  initial: "Off"
  state Off {}
  state On {
    region "C" { initial: "C1" state C1 {} state C2 {} transition C1 -> C2 { trigger: "go" } }
    region "D" { initial: "D1" state D1 {} state D2 {} }
    transition C2 -> D2 { trigger: "cross" }
    transition D2 -> C1 { trigger: "back" }
  }
  transition Off -> On { trigger: "start" }
}
test_case "TC" { verifies: ["R"] execute: { machine: "Two" stop_time: 40 ms inputs: ["10 ms: go", "20 ms: jump"] }
  expects: ["at(10 ms, \"Two\".A2) == 1", "at(20 ms, \"Two\".A1) == 1", "at(20 ms, \"Two\".B1) == 1"] }
"#;
    let before = compile(source);
    let text = sysmlv2_generator::generate_sysmlv2(&before.semantic_model, &before.ast);
    assert_eq!(written_in_a_parallel_state(&text), Vec::<String>::new(), "{text}");
    assert_eq!(text.matches("@Beside;").count(), 3, "{text}");
    let built = sysml_parser::records(&text).unwrap_or_else(|e| panic!("the export does not parse: {e}\n{text}"));
    let imported = sysml_import::import_records(&built.records).unwrap_or_else(|f| panic!("{}\n{}", f.message, f.rejected.unwrap_or_default()));
    let after = compile(&imported.source);
    assert_eq!(regions(&before), regions(&after), "{}", imported.source);
    let transitions = |result: &arclang::CompilationResult| -> Vec<Vec<(String, String, String, String)>> {
        result.ast.state_machines.iter().map(|machine| machine.transitions.iter().map(|t| (t.id.clone(), t.from.clone(), t.to.clone(), t.trigger.clone())).collect()).collect()
    };
    assert_eq!(transitions(&before), transitions(&after), "{}", imported.source);
    // Beside the regions of the state, as written: the state's own.
    let on = |result: &arclang::CompilationResult| result.ast.state_machines[1].states.iter().find(|state| state.name == "On").map(|state| state.transitions.len());
    assert_eq!((on(&before), on(&after)), (Some(2), Some(2)), "{}", imported.source);
    let verdicts = |result: &arclang::CompilationResult| result.warnings.iter().filter(|warning| warning.contains("verification:")).cloned().collect::<BTreeSet<String>>();
    assert_eq!(verdicts(&before), verdicts(&after), "{}", imported.source);
}

/// What a model declares outside its architecture, which the element graph does not carry:
/// by keyword and id, the name and the attributes of each I/O binding, twin and rule.
fn declarations(result: &arclang::CompilationResult) -> BTreeMap<(&'static str, String), BTreeMap<String, serde_json::Value>> {
    type Attributes = std::collections::HashMap<String, arclang::compiler::ast::AttributeValue>;
    let said = |name: &str, attributes: &Attributes| {
        let mut said: BTreeMap<String, serde_json::Value> = attributes.iter().filter(|(key, _)| key.as_str() != "id").map(|(key, value)| (key.clone(), elements::attribute_json(value))).collect();
        said.insert("name".to_string(), serde_json::json!(name));
        said
    };
    let ast = &result.ast;
    let mut all = BTreeMap::new();
    for binding in &ast.io_bindings {
        all.insert(("io_binding", binding.id.clone()), said(&binding.name, &binding.attributes));
    }
    for twin in &ast.twins {
        all.insert(("twin", twin.id.clone()), said(&twin.name, &twin.attributes));
    }
    for rule in &ast.rules {
        let id = rule.attributes.get("id").and_then(|id| id.as_string()).unwrap_or(&rule.name).to_string();
        all.insert(("rule", id), said(&rule.name, &rule.attributes));
    }
    all
}

#[test]
fn what_sysml_has_no_notion_of_comes_back_as_declared() {
    // The binding a case executes on, the twin `arclang twin` follows, the rules that block the
    // gate: lost on the way, the model that comes back no longer compiles, or passes a gate it
    // did not pass.
    for (example, expected) in [
        ("automotive/power-window/power_window.arc", "io_binding"),
        ("automotive/body-controller/body_controller.arc", "io_binding"),
        ("building/heat-pump/heat_pump_heating.arc", "twin"),
        ("aerospace/drone-endurance/drone_endurance.arc", "rule"),
    ] {
        let (before, after) = there_and_back(&Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(example));
        let declared = declarations(&before);
        assert!(declared.keys().any(|(keyword, _)| *keyword == expected), "{example} declares a {expected}");
        let back = declarations(&after);
        assert_eq!(back.keys().collect::<Vec<_>>(), declared.keys().collect::<Vec<_>>(), "{example}");
        for (key, said) in &declared {
            for (attribute, value) in said {
                assert!(back[key].get(attribute).is_some_and(|other| same(other, value)), "{example}: {} {}: {attribute} was {value}, is {:?}", key.0, key.1, back[key].get(attribute));
            }
            assert_eq!(back[key].len(), said.len(), "{example}: {} {} says more than it did: {:?}", key.0, key.1, back[key]);
        }
    }
}

#[test]
fn a_trace_comes_back_with_what_it_says_and_a_function_with_what_it_realizes() {
    // Why a component satisfies a requirement, the change a suspect link was reviewed against,
    // the function of the level above a function realizes: none is said by the ends of a
    // `satisfy` or by a `perform`.
    for (example, expected) in [
        ("aerospace/flight-control/flight_control_system.arc", "rationale"),
        ("aerospace/drone-endurance/drone_endurance.arc", "reviewed"),
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples").join(example);
        let (before, _) = there_and_back(&path);
        assert!(before.ast.traces.iter().any(|trace| trace.attributes.contains_key(expected)), "{example}: a trace says its {expected}");
        let m = round_trip(&path);
        report(example, &m);
        let lost: Vec<&String> = m.lost_values.keys().filter(|key| key.starts_with("Trace.") || key.ends_with("Function.realizes") || key.ends_with("Function.parent")).collect();
        assert!(lost.is_empty(), "{example}: lost {lost:?}");
        assert_eq!(m.kept_relationships, m.total_relationships, "{example}: {:?}", m.lost_relationships_by_kind);
    }
}

#[test]
fn a_reviewed_link_is_still_reviewed_when_the_model_comes_back() {
    // A suspect link blocks the gate: a model that only went to SysML and back has changed
    // nothing, whatever the units of its quantities are called there (`85 %`, `85 percent`).
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/aerospace/drone-endurance/drone_endurance.arc");
    let (before, after) = there_and_back(&path);
    let reviewed = |result: &arclang::CompilationResult| result.ast.traces.iter().filter(|t| t.attributes.contains_key("reviewed")).count() + result.ast.test_cases.iter().filter(|c| c.attributes.contains_key("reviewed")).count();
    assert!(reviewed(&before) >= 8, "the example reviews its traces and its verifications");
    assert_eq!(reviewed(&after), reviewed(&before));
    let suspects = |result: &arclang::CompilationResult| -> Vec<String> { result.warnings.iter().filter(|w| w.starts_with("suspect:")).cloned().collect() };
    assert!(suspects(&before).is_empty(), "{:?}", suspects(&before));
    assert!(suspects(&after).is_empty(), "{:?}", suspects(&after));
}

#[test]
#[ignore = "survey: prints the round trip of every example, run with --ignored --nocapture"]
fn every_example_round_trip_is_measured() {
    fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == "archive" || n == "multifile") {
                    continue;
                }
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "arc") {
                out.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(&Path::new(env!("CARGO_MANIFEST_DIR")).join("examples"), &mut files);
    files.sort();
    let (mut kept, mut total) = (0, 0);
    for file in files {
        let source = std::fs::read_to_string(&file).unwrap();
        if Compiler::new(CompilerConfig::default()).compile_string(&source).is_err() {
            continue;
        }
        // A model that does not come back is reported, and the survey goes on.
        let name = file.file_name().unwrap().to_str().unwrap().to_string();
        match std::panic::catch_unwind(|| round_trip(&file)) {
            Ok(m) => {
                report(&name, &m);
                kept += m.kept_elements;
                total += m.total_elements;
            }
            Err(_) => {
                println!("ROUNDTRIP {name}: does not come back (see the panic above)");
                total += shape(&compile(&source)).0.len();
            }
        }
    }
    println!("ROUNDTRIP total elements {kept}/{total} ({}%)", 100 * kept / total.max(1));
}

#[test]
fn a_note_goes_to_sysml_as_a_comment_about_its_elements_and_comes_back() {
    let source = "model T { name: \"t\" }\nlogical_architecture \"L\" {\n    component \"Camera\" { id: \"LC-CAM\" description: \"Sees the lane\" }\n    component \"Controller\" { id: \"LC-CTL\" }\n}\nnote \"Runs at 30 Hz; */ is harmless\" { id: \"N-1\" about: [\"LC-CAM\", \"LC-CTL\"] }\nnote \"Plans within a frame\" { about: [\"LC-CTL\"] }\n";
    let before = compile(source);
    let text = sysmlv2_generator::generate_sysmlv2(&before.semantic_model, &before.ast);
    assert!(text.contains("comment 'N-1' about "), "{text}");
    let built = sysml_parser::records(&text).unwrap_or_else(|e| panic!("the export does not parse: {e}\n{text}"));
    let imported = sysml_import::import_records(&built.records).unwrap_or_else(|f| panic!("{}", f.message));
    let after = compile(&imported.source);
    let notes: Vec<(String, String, Vec<String>)> = after.ast.notes.iter().map(|n| (n.id.clone(), n.name.clone(), n.about.clone())).collect();
    assert_eq!(notes.len(), 2, "{}", imported.source);
    assert!(notes.iter().any(|(id, text, about)| id == "N-1" && text == "Runs at 30 Hz; * / is harmless" && about.len() == 2), "{notes:?}");
    assert!(notes.iter().any(|(id, text, about)| id.is_empty() && text == "Plans within a frame" && about.len() == 1), "{notes:?}");
    // A doc stays the description of its element, not a note.
    assert!(imported.source.contains("Sees the lane"), "{}", imported.source);
    assert_eq!(shape(&before).1.iter().filter(|r| r.0 == "Annotation").count(), shape(&after).1.iter().filter(|r| r.0 == "Annotation").count());
}
