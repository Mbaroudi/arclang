//! Arcadia transitions: carry the elements of one level to the next.
//!
//! In Capella, the transition commands create, on the next level, the
//! elements that realize those of the level above, with the realization
//! links. ArcLang writes them into the source the way the modeling API
//! writes anything: as declarations added to the text, each one verified by
//! the compiler, nothing else touched.
//!
//! What a transition creates is what the next level does not realize yet:
//! an operational activity no system function realizes, a system function
//! no logical function realizes, a logical component no physical component
//! realizes. What is realized already is left alone, so a transition can be
//! run again after the model grew.

use super::ast::{AttributeValue, LogicalComponent, LogicalFunction, Model, OperationalActivity, SystemFunction};
use super::source_edit::{self, string_literal, Container};
use super::{Compiler, CompilerConfig};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Step {
    /// Operational activities -> system functions.
    OaToSa,
    /// System functions -> logical functions, allocated to a logical component.
    SaToLa,
    /// Logical components -> behavior components deployed on a physical node.
    LaToPa,
}

impl Step {
    pub const ALL: [Step; 3] = [Step::OaToSa, Step::SaToLa, Step::LaToPa];

    pub fn parse(text: &str) -> Option<Step> {
        match text.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "oa-to-sa" | "oa2sa" | "oa" => Some(Step::OaToSa),
            "sa-to-la" | "sa2la" | "sa" => Some(Step::SaToLa),
            "la-to-pa" | "la2pa" | "la" => Some(Step::LaToPa),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Step::OaToSa => "oa-to-sa",
            Step::SaToLa => "sa-to-la",
            Step::LaToPa => "la-to-pa",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Step::OaToSa => "Operational activities to system functions",
            Step::SaToLa => "System functions to logical functions",
            Step::LaToPa => "Logical components to physical components",
        }
    }
}

/// One element of the upper level and whether the next level realizes it.
#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    pub id: String,
    pub name: String,
    pub kind: &'static str,
    /// Ids of the elements that realize it; empty means the transition creates one.
    pub realized_by: Vec<String>,
}

/// What a step would do, before it is applied.
#[derive(Debug, Clone, Serialize)]
pub struct Plan {
    pub step: Step,
    pub label: &'static str,
    pub candidates: Vec<Candidate>,
    /// Why the step cannot run at all, when it cannot.
    pub blocked: Option<String>,
}

impl Plan {
    pub fn pending(&self) -> impl Iterator<Item = &Candidate> {
        self.candidates.iter().filter(|candidate| candidate.realized_by.is_empty())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Created {
    pub kind: &'static str,
    pub id: String,
    pub name: String,
    pub realizes: String,
    pub inside: String,
    /// Why the element was not given the name of what it realizes, when
    /// it was not: the model refers to that name and a second element
    /// carrying it would make the reference ambiguous.
    pub note: Option<String>,
}

/// The outcome of an applied step: the edited source and what was declared.
#[derive(Debug, Clone, Serialize)]
pub struct Transition {
    pub step: Step,
    pub source: String,
    pub created: Vec<Created>,
    /// The container the created elements went into, created by the
    /// transition when `true`.
    pub container: String,
    pub container_created: bool,
}

/// Every realization written in the model: `(realizing id, realized reference)`.
fn realizations(model: &Model) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = model
        .traces
        .iter()
        .chain(model.operational_analysis.iter().flat_map(|oa| oa.traces.iter()))
        .filter(|trace| trace.trace_type == "realizes")
        .map(|trace| (trace.from.clone(), trace.to.clone()))
        .collect();
    let attribute = |id: &str, attributes: &std::collections::HashMap<String, AttributeValue>, into: &mut Vec<(String, String)>| {
        match attributes.get("realizes") {
            Some(AttributeValue::String(one)) => into.push((id.to_string(), one.clone())),
            Some(AttributeValue::List(items)) => {
                for item in items {
                    if let Some(text) = item.as_string() {
                        into.push((id.to_string(), text.to_string()));
                    }
                }
            }
            _ => {}
        }
    };
    fn walk_level_functions(functions: &[LogicalFunction], attribute: &dyn Fn(&str, &std::collections::HashMap<String, AttributeValue>, &mut Vec<(String, String)>), into: &mut Vec<(String, String)>) {
        for function in functions {
            attribute(&function.id, &function.attributes, into);
            walk_level_functions(&function.sub_functions, attribute, into);
        }
    }
    fn walk_components(components: &[LogicalComponent], attribute: &dyn Fn(&str, &std::collections::HashMap<String, AttributeValue>, &mut Vec<(String, String)>), into: &mut Vec<(String, String)>) {
        for component in components {
            attribute(&component.id, &component.attributes, into);
            walk_level_functions(&component.functions, attribute, into);
            walk_components(&component.sub_components, attribute, into);
        }
    }
    for la in &model.logical_architecture {
        walk_components(&la.components, &attribute, &mut found);
        walk_level_functions(&la.functions, &attribute, &mut found);
    }
    for pa in &model.physical_architecture {
        for node in &pa.nodes {
            attribute(&node.id, &node.attributes, &mut found);
            // A behavior component realizes the logical component it is
            // allocated (`allocated_component`), kept by the parser in one
            // list with the functions it performs.
            walk_level_functions(&node.functions, &attribute, &mut found);
            for behavior in &node.behavior_components {
                for reference in behavior.allocated_functions.iter().chain(behavior.allocated_component.iter()) {
                    found.push((behavior.id.clone(), reference.clone()));
                }
                walk_level_functions(&behavior.functions, &attribute, &mut found);
            }
        }
        walk_level_functions(&pa.functions, &attribute, &mut found);
    }
    for sa in &model.system_analysis {
        fn walk_functions(functions: &[SystemFunction], attribute: &dyn Fn(&str, &std::collections::HashMap<String, AttributeValue>, &mut Vec<(String, String)>), into: &mut Vec<(String, String)>) {
            for function in functions {
                attribute(&function.id, &function.attributes, into);
                walk_functions(&function.sub_functions, attribute, into);
            }
        }
        walk_functions(&sa.functions, &attribute, &mut found);
    }
    found
}

fn realized_by(realizations: &[(String, String)], id: &str, name: &str) -> Vec<String> {
    realizations
        .iter()
        .filter(|(_, realized)| realized == id || realized == name)
        .map(|(by, _)| by.clone())
        .collect()
}

fn activities(model: &Model) -> Vec<&OperationalActivity> {
    fn walk<'a>(list: &'a [OperationalActivity], into: &mut Vec<&'a OperationalActivity>) {
        for activity in list {
            into.push(activity);
            walk(&activity.sub_activities, into);
        }
    }
    let mut all = Vec::new();
    for oa in &model.operational_analysis {
        walk(&oa.activities, &mut all);
        for entity in &oa.entities {
            walk(&entity.activities, &mut all);
        }
    }
    all
}

fn system_functions(model: &Model) -> Vec<&SystemFunction> {
    fn walk<'a>(list: &'a [SystemFunction], into: &mut Vec<&'a SystemFunction>) {
        for function in list {
            into.push(function);
            walk(&function.sub_functions, into);
        }
    }
    let mut all = Vec::new();
    for sa in &model.system_analysis {
        walk(&sa.functions, &mut all);
    }
    all
}

fn logical_components(model: &Model) -> Vec<&LogicalComponent> {
    fn walk<'a>(list: &'a [LogicalComponent], into: &mut Vec<&'a LogicalComponent>) {
        for component in list {
            into.push(component);
            walk(&component.sub_components, into);
        }
    }
    let mut all = Vec::new();
    for la in &model.logical_architecture {
        walk(&la.components, &mut all);
    }
    all
}

/// What `step` would create on `model`.
pub fn plan(step: Step, model: &Model) -> Plan {
    let realizations = realizations(model);
    let candidate = |id: &str, name: &str, kind: &'static str| Candidate {
        id: id.to_string(),
        name: name.to_string(),
        kind,
        realized_by: realized_by(&realizations, id, name),
    };
    let (candidates, blocked) = match step {
        Step::OaToSa => (
            activities(model).iter().map(|a| candidate(&a.id, &a.name, "operational_activity")).collect(),
            model.system_analysis.is_empty().then(|| "the model has no system_analysis block to create system functions in".to_string()),
        ),
        Step::SaToLa => (
            system_functions(model).iter().map(|f| candidate(&f.id, &f.name, "function")).collect(),
            model.logical_architecture.is_empty().then(|| "the model has no logical_architecture block to create logical functions in".to_string()),
        ),
        Step::LaToPa => (
            logical_components(model).iter().map(|c| candidate(&c.id, &c.name, "component")).collect(),
            model.physical_architecture.is_empty().then(|| "the model has no physical_architecture block to create physical components in".to_string()),
        ),
    };
    Plan { step, label: step.label(), candidates, blocked }
}

/// A short identifier made of a prefix and the name.
fn ident(prefix: &str, name: &str) -> String {
    let mut out = String::from(prefix);
    out.push('-');
    let mut last_dash = false;
    for character in name.chars() {
        if character.is_alphanumeric() {
            out.push(character);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_end_matches('-').to_string()
}

fn compile(source: &str) -> Result<super::CompilationResult, String> {
    Compiler::new(CompilerConfig { optimization_level: 0, target: "json".to_string() })
        .compile_string(source)
        .map_err(|e| e.to_string())
}

/// Apply `step` to `source`. `into` names the component (SA→LA) or node
/// (LA→PA) that receives the created elements; without it, one is created
/// when none exists. Every declaration goes through the source editor and
/// the result must compile.
pub fn apply(step: Step, source: &str, into: Option<&str>) -> Result<Transition, String> {
    let compiled = compile(source).map_err(|e| format!("the model does not compile: {e}"))?;
    let model = &compiled.ast;
    let plan = plan(step, model);
    if let Some(reason) = plan.blocked {
        return Err(reason);
    }
    let existing: HashSet<String> = compiled.semantic_model.all_elements.keys().cloned().collect();
    let mut text = source.to_string();
    let mut created = Vec::new();

    // Where the created elements go: the ways the text may know the
    // container (its id, its name), or a container created now.
    let (layer, container, container_created): (&str, Option<Vec<String>>, bool) = match step {
        Step::OaToSa => ("system_analysis", None, false),
        Step::SaToLa => {
            let name = into.map(str::to_string).unwrap_or_else(|| "Logical System".to_string());
            let known = logical_components(model).into_iter().find(|c| c.id == name || c.name == name);
            match known {
                Some(component) => ("logical_architecture", Some(vec![component.id.clone(), component.name.clone()]), false),
                None if into.is_some() => return Err(format!("no logical component '{name}' to allocate the functions to")),
                None => {
                    let id = ident("LC", &name);
                    text = source_edit::add_element(
                        &text,
                        Container::Layer { keyword: "logical_architecture", name: None },
                        "component",
                        &name,
                        &[("id".to_string(), string_literal(&id))],
                    )
                    .map_err(|e| e.to_string())?;
                    ("logical_architecture", Some(vec![id, name]), true)
                }
            }
        }
        Step::LaToPa => {
            let name = into.map(str::to_string).unwrap_or_else(|| "Physical System".to_string());
            let known = model.physical_architecture.iter().flat_map(|pa| pa.nodes.iter()).find(|n| n.id == name || n.name == name);
            match known {
                Some(node) => ("physical_architecture", Some(vec![node.id.clone(), node.name.clone()]), false),
                None if into.is_some() => return Err(format!("no physical node '{name}' to deploy the components on")),
                None => {
                    let id = ident("PN", &name);
                    text = source_edit::add_element(
                        &text,
                        Container::Layer { keyword: "physical_architecture", name: None },
                        "node",
                        &name,
                        &[("id".to_string(), string_literal(&id))],
                    )
                    .map_err(|e| e.to_string())?;
                    ("physical_architecture", Some(vec![id, name]), true)
                }
            }
        }
    };

    /// Try an edit with each way the text may know the container.
    fn add_into(text: &str, layer: &str, container: &Option<Vec<String>>, keyword: &str, name: &str, attributes: &[(String, String)]) -> Result<String, source_edit::EditError> {
        let Some(ways) = container else {
            return source_edit::add_element(text, Container::Layer { keyword: layer, name: None }, keyword, name, attributes);
        };
        let mut outcome = Err(source_edit::EditError::NotFound(ways.join(" / ")));
        for way in ways {
            outcome = source_edit::add_element(text, Container::Element(way), keyword, name, attributes);
            if !matches!(outcome, Err(source_edit::EditError::NotFound(_))) {
                break;
            }
        }
        outcome
    }

    let mut taken: HashSet<String> = existing.clone();
    let unique = |wanted: String, taken: &mut HashSet<String>| -> String {
        let mut id = wanted.clone();
        let mut n = 2;
        while taken.contains(&id) {
            id = format!("{wanted}-{n}");
            n += 1;
        }
        taken.insert(id.clone());
        id
    };

    for pending in plan.pending() {
        let (keyword, prefix) = match step {
            Step::OaToSa => ("function", "SF"),
            Step::SaToLa => ("function", "LF"),
            Step::LaToPa => ("behavior_component", "PC"),
        };
        let id = unique(ident(prefix, &pending.name), &mut taken);
        let mut attributes = vec![("id".to_string(), string_literal(&id))];
        if step == Step::LaToPa {
            // What the physical view draws as the realization.
            attributes.push(("allocated_component".to_string(), string_literal(&pending.id)));
        }
        let declare = |text: &str, name: &str| -> Result<String, String> {
            let mut next = add_into(text, layer, &container, keyword, name, &attributes)
                .map_err(|e| format!("cannot declare {keyword} '{name}': {e}"))?;
            // A behavior component's realization is its `allocated_component`;
            // functions get a realization trace.
            if step != Step::LaToPa {
                next = source_edit::add_trace(&next, &id, "realizes", &pending.id)
                    .map_err(|e| format!("cannot trace '{id}' realizes '{}': {e}", pending.id))?;
            }
            Ok(next)
        };
        // The realizing element takes the name of what it realizes, as in
        // Capella. When the model refers to that name, a second element
        // carrying it would make the reference ambiguous: the new one is
        // then named apart, and the note says why.
        let same = declare(&text, &pending.name)?;
        let (next, name, note) = match compile(&same) {
            Ok(_) => (same, pending.name.clone(), None),
            Err(error) if error.contains(&format!("ambiguous name '{}'", pending.name)) => {
                let apart = format!("{} ({})", pending.name, prefix);
                let text = declare(&text, &apart)?;
                let note = format!(
                    "named '{apart}' because the model refers to '{}' by name; refer to it by its id '{}' instead and both can carry the name",
                    pending.name, pending.id
                );
                (text, apart, Some(note))
            }
            Err(error) => return Err(format!("declaring '{}' would leave a model that does not compile: {error}", pending.name)),
        };
        text = next;
        created.push(Created {
            kind: keyword,
            id,
            name,
            realizes: pending.id.clone(),
            inside: container.as_ref().map(|ways| ways[0].clone()).unwrap_or_else(|| layer.to_string()),
            note,
        });
    }

    let after = compile(&text).map_err(|e| format!("the transition would leave a model that does not compile: {e}"))?;
    let graph = super::elements::build(&after.ast, &after.semantic_model);
    let missing: Vec<&str> = created
        .iter()
        .filter(|c| !graph.elements.iter().any(|element| element.id == c.id))
        .map(|c| c.id.as_str())
        .collect();
    if !missing.is_empty() {
        return Err(format!("the compiled model does not show the created elements {missing:?}"));
    }
    Ok(Transition {
        step,
        source: text,
        created,
        container: container.map(|ways| ways[0].clone()).unwrap_or_else(|| layer.to_string()),
        container_created,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL: &str = r#"operational_analysis "OA" {
    actor "Driver" { id: "OA-DRV" }
    operational_activity "Brake" { id: "OA-BRK" performed_by: "Driver" }
    operational_activity "Watch the road" { id: "OA-WTC" performed_by: "Driver" }
}
system_analysis SA {
    function "Watch the road" { id: "SF-WTC" }
}
logical_architecture LA {
    component "Controller" { id: "LC-CTL" }
}
physical_architecture PA {
    node "ECU" { id: "PN-ECU" }
}
trace "SF-WTC" realizes "OA-WTC"
"#;

    #[test]
    fn the_plan_tells_what_is_realized_and_what_is_not() {
        let model = compile(MODEL).expect("compiles").ast;
        let first = plan(Step::OaToSa, &model);
        let pending: Vec<&str> = first.pending().map(|c| c.name.as_str()).collect();
        assert_eq!(pending, ["Brake"]);
        let watched = first.candidates.iter().find(|c| c.id == "OA-WTC").unwrap();
        assert_eq!(watched.realized_by, ["SF-WTC"]);
        assert!(first.blocked.is_none());
        assert!(plan(Step::SaToLa, &model).blocked.is_none());
    }

    #[test]
    fn oa_to_sa_declares_a_function_and_its_realization() {
        let done = apply(Step::OaToSa, MODEL, None).expect("applies");
        assert_eq!(done.created.len(), 1);
        assert_eq!((done.created[0].id.as_str(), done.created[0].realizes.as_str()), ("SF-Brake", "OA-BRK"));
        assert!(done.source.contains("function \"Brake\""));
        assert!(done.source.contains("trace \"SF-Brake\" realizes \"OA-BRK\""));
        // Running it again creates nothing more.
        let again = apply(Step::OaToSa, &done.source, None).expect("applies again");
        assert!(again.created.is_empty());
        assert_eq!(again.source, done.source);
    }

    #[test]
    fn sa_to_la_allocates_to_the_named_component_or_creates_one() {
        let named = apply(Step::SaToLa, MODEL, Some("Controller")).expect("applies");
        assert!(!named.container_created);
        assert!(named.source.contains("function \"Watch the road\""), "{}", named.source);
        assert!(named.source.contains("id: \"LF-Watch-the-road\""), "{}", named.source);
        assert!(named.source.contains("trace \"LF-Watch-the-road\" realizes \"SF-WTC\""));
        let created = apply(Step::SaToLa, MODEL, None).expect("applies");
        assert!(created.container_created);
        assert!(created.source.contains("component \"Logical System\""));
        assert!(apply(Step::SaToLa, MODEL, Some("Nobody")).is_err());
    }

    #[test]
    fn la_to_pa_deploys_a_realizing_component_on_the_node() {
        let done = apply(Step::LaToPa, MODEL, Some("ECU")).expect("applies");
        assert_eq!(done.created[0].id, "PC-Controller");
        assert!(done.source.contains("behavior_component \"Controller\""), "{}", done.source);
        assert!(done.source.contains("allocated_component: \"LC-CTL\""));
        let after = compile(&done.source).expect("compiles");
        let graph = super::super::elements::build(&after.ast, &after.semantic_model);
        assert!(graph.elements.iter().any(|e| e.id == "PC-Controller"));
        // The physical view draws the realization.
        let set = super::super::diagram::build_diagrams(&after.ast);
        let pab = set.diagrams.iter().find(|d| d.id == "pab").expect("physical view");
        let node = pab.nodes.iter().find(|n| n.id == "PN-ECU").expect("node");
        assert_eq!(node.children[0].realizes, ["LC-CTL"]);
        // Running it again creates nothing more.
        assert!(apply(Step::LaToPa, &done.source, Some("ECU")).expect("again").created.is_empty());
    }

    #[test]
    fn a_missing_layer_blocks_the_step() {
        let blocked = plan(Step::LaToPa, &compile("logical_architecture LA { component \"A\" {} }").unwrap().ast);
        assert!(blocked.blocked.is_some());
        assert!(apply(Step::LaToPa, "logical_architecture LA { component \"A\" {} }", None).is_err());
    }
}
