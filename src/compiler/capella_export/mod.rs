//! Export of an ArcLang model as a Capella 7 project: the `.capella` model,
//! with the `.aird` (its Sirius session, no diagram yet) and the `.afm`
//! (the viewpoints it uses) Capella needs to open it.
//!
//! The Capella ids are the ArcLang ids: a model imported from Capella goes
//! back with its identities, so Capella and the tools around it (diff,
//! merge, links from requirement tools) see the same elements. What
//! Capella needs and ArcLang does not write (packages, parts, involvements,
//! realization links) gets ids made up from what identifies it, so that an
//! export is the same every time. Attributes Capella has no place for go as
//! property values of their element, as Capella users do.
//!
//! The import reads the export back: Capella -> ArcLang -> Capella keeps
//! what ArcLang holds.

mod behaviour;
mod context;
mod data;
mod diagram_drawer;
mod diagrams;
mod epbs;
mod functions;
mod header;
mod logical;
mod operational;
mod physical;
mod relations;
mod requirements;
mod scenarios;
mod stpa;
mod system;
mod traces;
mod xml;

use crate::compiler::ast::Model;
use context::{derived, Context};
use xml::{document, El, NAMESPACES};

/// The files of an exported Capella project, by role.
pub struct Exported {
    pub capella: String,
    pub aird: String,
    pub afm: String,
    /// What the export could not write, and why.
    pub notes: Vec<String>,
}

/// The name of the model, as Capella shows it.
fn model_name(model: &Model) -> String {
    context::text(&model.attributes, "name").map(str::to_string).unwrap_or_else(|| "ArcLang model".to_string())
}

/// Export `model`; `file_stem` names the files (`<stem>.capella`, `<stem>.afm`).
pub fn export(model: &Model, file_stem: &str) -> Exported {
    let mut context = Context::default();
    let name = model_name(model);
    let project_id = context::id_of(&model.attributes, &format!("project:{name}"));
    let mut project = El::root("org.polarsys.capella.core.data.capellamodeller:Project").attr("xmi:version", "2.0");
    for (key, value) in NAMESPACES {
        project.set(key, value);
    }
    if stpa::is_used(model) {
        project.set("xmlns:stpa", "http://www.thalesgroup.com/capella/stpa");
    }
    project.set("id", &project_id);
    project.set("name", &name);
    project.push(El::new("ownedExtensions", "libraries:ModelInformation", &derived(&format!("{project_id}:information"))));
    project.push(header::progress_status(&project_id, &statuses(model)));
    project.push(El::new("keyValuePairs", "capellacore:KeyValue", &derived(&format!("{project_id}:approach"))).attr("key", "projectApproach").attr("value", "SingletonComponents"));

    // Every level registers what it declares before any level refers to it.
    data::register(&mut context, model);
    requirements::register(&mut context, model);
    stpa::register(&mut context, model);
    for r in stpa::constraints(model) {
        let id = context::text(&r.attributes, "capella_id").unwrap_or(&r.id).to_string();
        context.register(&r.id, &id);
        context.register(&id, &id);
    }
    for machine in &model.state_machines {
        behaviour::register(&mut context, machine);
    }
    for oa in &model.operational_analysis {
        operational::register(&mut context, oa);
    }
    let main_sa = main_system(model);
    for sa in &model.system_analysis {
        system::register(&mut context, sa);
    }
    for la in &model.logical_architecture {
        logical::register(&mut context, la);
    }
    for pa in &model.physical_architecture {
        physical::register(&mut context, pa);
    }
    for e in &model.epbs {
        epbs::register(&mut context, e);
    }

    let engineering_id = derived(&format!("{project_id}:engineering"));
    let mut engineering = El::new("ownedModelRoots", "capellamodeller:SystemEngineering", &engineering_id).attr("name", &name);
    if let Some(analysis) = stpa::write(&context, model, &engineering_id) {
        engineering.push(analysis);
    }
    let oa_el = match model.operational_analysis.first() {
        Some(oa) => operational::write(&mut context, oa),
        None => operational::empty(&project_id),
    };
    let oa_id = oa_el.id().to_string();
    let mut sa_el = match main_sa {
        Some(sa) => system::write(&mut context, sa, &derived(&format!("{oa_id}:root-activity"))),
        None => system::empty(&project_id, &derived(&format!("{oa_id}:root-activity"))),
    };
    let sa_id = sa_el.id().to_string();
    for extension in requirements::write(&context, model, &sa_id) {
        sa_el.push(extension);
    }
    if let Some(pkg) = data::package(&context, model, &sa_id) {
        sa_el.push(pkg);
    }
    if let Some(pkg) = data::interfaces(&context, model, "system", &sa_id) {
        sa_el.push(pkg);
    }
    let mut la_el = match model.logical_architecture.first() {
        Some(la) => logical::write(&mut context, la, &derived(&format!("{sa_id}:root-function"))),
        None => logical::empty(&project_id, &derived(&format!("{sa_id}:root-function"))),
    };
    let la_id = la_el.id().to_string();
    if let Some(pkg) = data::interfaces(&context, model, "logical", &la_id) {
        la_el.push(pkg);
    }
    let mut pa_el = match model.physical_architecture.first() {
        Some(pa) => physical::write(&mut context, pa, &derived(&format!("{la_id}:root-function"))),
        None => physical::empty(&project_id, &derived(&format!("{la_id}:root-function"))),
    };
    let pa_id = pa_el.id().to_string();
    if let Some(pkg) = data::interfaces(&context, model, "physical", &pa_id) {
        pa_el.push(pkg);
    }
    let epbs_el = match model.epbs.first() {
        Some(e) => epbs::write(&context, e, &project_id),
        None => El::new("ownedArchitectures", "epbs:EPBSArchitecture", &derived(&format!("{project_id}:epbs"))).attr("name", "EPBS Architecture"),
    };
    let epbs_id = epbs_el.id().to_string();
    engineering.push(oa_el);
    engineering.push(sa_el.with(realization("ownedOperationalAnalysisRealizations", "ctx:OperationalAnalysisRealization", &sa_id, &oa_id)));
    engineering.push(la_el.with(realization("ownedSystemAnalysisRealizations", "la:SystemAnalysisRealization", &la_id, &sa_id)));
    engineering.push(pa_el.with(realization("ownedLogicalArchitectureRealizations", "pa:LogicalArchitectureRealization", &pa_id, &la_id)));
    engineering.push(epbs_el.with(realization("ownedPhysicalArchitectureRealizations", "epbs:PhysicalArchitectureRealization", &epbs_id, &pa_id)));
    if model.operational_analysis.len() > 1 {
        context.note(format!("{} operational analyses: Capella has one, the first is exported", model.operational_analysis.len()));
    }
    if model.system_analysis.iter().filter(|sa| !(sa.functions.is_empty() && sa.components.is_empty() && sa.external_actors.is_empty())).count() > 1 {
        context.note(format!("{} system analyses: Capella has one, the main one is exported", model.system_analysis.len()));
    }
    // State machines go to their component.
    let fallback = main_sa
        .and_then(|sa| sa.components.first().map(|c| context::id_of(&c.attributes, &format!("system-component:{}", c.name))));
    for machine in &model.state_machines {
        let (owner, name) = behaviour::owner_and_name(&context, machine, fallback.as_deref());
        let name = own_name(&engineering, owner.as_deref(), name);
        let el = behaviour::state_machine(&context, machine, name);
        match owner.and_then(|o| engineering.find_mut(&o)) {
            Some(owner) => owner.push(el),
            None => context.note(format!("state machine '{}' left out: no component to own it", machine.name)),
        }
    }
    for scenario in &model.scenarios {
        let (owner, name) = scenarios::owner(&context, &engineering, scenario);
        let name = own_name(&engineering, owner.as_deref(), name);
        let Some(el) = scenarios::scenario(&mut context, &engineering, scenario, name) else { continue };
        match owner.and_then(|o| engineering.find_mut(&o)) {
            Some(owner) => owner.push(el),
            None => context.note(format!("scenario '{}' left out: no capability to own it", scenario.name)),
        }
    }
    // Functions available only in some states or modes.
    for (function, states) in available_in_states(model) {
        let refs: Vec<String> = states.iter().filter_map(|s| context.resolve(s)).map(|s| format!("#{s}")).collect();
        if let (false, Some(el)) = (refs.is_empty(), engineering.find_mut(&function)) {
            el.set("availableInStates", &refs.join(" "));
        }
    }
    traces::write(&mut context, &mut engineering, &model.traces);
    requirements::relations(&mut context, &mut engineering, model, &sa_id);
    // Exchange categories go to the function package of the system analysis.
    let functions = derived(&format!("{sa_id}:functions"));
    let categories = relations::categories(&context, &functions);
    if let Some(pkg) = engineering.find_mut(&functions) {
        for category in categories {
            pkg.push(category);
        }
    }
    project.push(engineering);

    let afm = afm(&project_id, !requirements::all(model).is_empty(), stpa::is_used(model));
    let index = diagrams::index(project.children().last().expect("the system engineering"));
    let drawn = diagrams::diagrams(&mut context, &index, &model.diagrams, &format!("{file_stem}.capella"));
    let aird = aird(&project_id, file_stem, drawn);
    Exported { capella: document(&project), aird, afm, notes: context.notes }
}

/// A machine's or scenario's own name: the import names one `Owner / Name`
/// when several share a name; the owner says it again, so the prefix goes.
fn own_name<'a>(engineering: &El, owner: Option<&str>, name: &'a str) -> &'a str {
    let owner_name = owner.and_then(|o| engineering.find(o)).and_then(|o| o.get("name"));
    match (name.split_once(" / "), owner_name) {
        // `Owner / Name (2)`: the import numbers a prefixed name that two share.
        (Some((prefix, rest)), Some(owner_name)) if prefix == owner_name => match rest.rsplit_once(" (") {
            Some((base, n)) if n.strip_suffix(')').is_some_and(|n| n.parse::<u32>().is_ok()) => base,
            _ => rest,
        },
        _ => name,
    }
}

/// The system analysis that holds the model: the one with Capella's id,
/// else the fullest (a `requirements` block makes one of its own).
fn main_system(model: &Model) -> Option<&crate::compiler::ast::SystemAnalysis> {
    let content = |sa: &crate::compiler::ast::SystemAnalysis| sa.functions.len() + sa.components.len() + sa.external_actors.len() + sa.capabilities.len() + sa.missions.len();
    model.system_analysis.iter().find(|sa| !sa.id.is_empty()).or_else(|| model.system_analysis.iter().max_by_key(|sa| content(sa)))
}

/// `available_in_states` of the functions and activities of every level.
fn available_in_states(model: &Model) -> Vec<(String, Vec<String>)> {
    use crate::compiler::ast::{AttributeValue, LogicalFunction, OperationalActivity, SystemFunction};
    use std::collections::HashMap;
    let states = |attributes: &HashMap<String, AttributeValue>| context::list(attributes, "available_in_states");
    let mut out = Vec::new();
    fn system(f: &SystemFunction, out: &mut Vec<(String, Vec<String>)>, states: &dyn Fn(&HashMap<String, AttributeValue>) -> Vec<String>) {
        out.push((f.id.clone(), states(&f.attributes)));
        f.sub_functions.iter().for_each(|s| system(s, out, states));
    }
    fn logical(f: &LogicalFunction, out: &mut Vec<(String, Vec<String>)>, states: &dyn Fn(&HashMap<String, AttributeValue>) -> Vec<String>) {
        out.push((f.id.clone(), states(&f.attributes)));
        f.sub_functions.iter().for_each(|s| logical(s, out, states));
    }
    fn activity(a: &OperationalActivity, out: &mut Vec<(String, Vec<String>)>, states: &dyn Fn(&HashMap<String, AttributeValue>) -> Vec<String>) {
        out.push((a.id.clone(), states(&a.attributes)));
        a.sub_activities.iter().for_each(|s| activity(s, out, states));
    }
    model.operational_analysis.iter().flat_map(|oa| &oa.activities).for_each(|a| activity(a, &mut out, &states));
    model.system_analysis.iter().flat_map(|sa| &sa.functions).for_each(|f| system(f, &mut out, &states));
    for la in &model.logical_architecture {
        la.functions.iter().for_each(|f| logical(f, &mut out, &states));
        fn components(list: &[crate::compiler::ast::LogicalComponent], out: &mut Vec<(String, Vec<String>)>, states: &dyn Fn(&HashMap<String, AttributeValue>) -> Vec<String>) {
            for c in list {
                c.functions.iter().for_each(|f| logical(f, out, states));
                components(&c.sub_components, out, states);
            }
        }
        components(&la.components, &mut out, &states);
    }
    for pa in &model.physical_architecture {
        pa.functions.iter().for_each(|f| logical(f, &mut out, &states));
        for n in &pa.nodes {
            n.functions.iter().for_each(|f| logical(f, &mut out, &states));
            n.behavior_components.iter().flat_map(|b| &b.functions).for_each(|f| logical(f, &mut out, &states));
        }
    }
    out.retain(|(_, s)| !s.is_empty());
    out
}

/// The progress statuses the model uses, beyond Capella's own.
fn statuses(model: &Model) -> Vec<String> {
    fn walk(value: &serde_json::Value, found: &mut Vec<String>) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    if key == "status" || key == "capella_status" {
                        if let Some(text) = value.get("String").and_then(|v| v.as_str()) {
                            if !found.iter().any(|f| f == text) {
                                found.push(text.to_string());
                            }
                        }
                    }
                    walk(value, found);
                }
            }
            serde_json::Value::Array(items) => items.iter().for_each(|item| walk(item, found)),
            _ => {}
        }
    }
    let mut found = Vec::new();
    if let Ok(json) = model.to_json_value() {
        walk(&json, &mut found);
    }
    found
}

/// A realization link, from the element that owns it to the one it realizes.
pub(super) fn realization(tag: &str, kind: &str, source: &str, target: &str) -> El {
    El::new(tag, kind, &derived(&format!("{source}->{target}:{kind}"))).attr("targetElement", &format!("#{target}")).attr("sourceElement", &format!("#{source}"))
}

/// The viewpoints the model uses: Capella's own.
fn afm(project_id: &str, with_requirements: bool, with_stpa: bool) -> String {
    let mut root = El::root("metadata:Metadata")
        .attr("xmi:version", "2.0")
        .attr("xmlns:xmi", "http://www.omg.org/XMI")
        .attr("xmlns:metadata", "http://www.polarsys.org/kitalpha/ad/metadata/1.0.0")
        .attr("id", &derived(&format!("{project_id}:afm")))
        .with(El::root("viewpointReferences").attr("id", &derived(&format!("{project_id}:afm:capella"))).attr("vpId", "org.polarsys.capella.core.viewpoint").attr("version", "7.1.0"));
    if with_stpa {
        root.push(El::root("viewpointReferences").attr("id", &derived(&format!("{project_id}:afm:stpa"))).attr("vpId", "com.thalesgroup.mde.capella.stpa").attr("version", "0.1.0.qualifier"));
    }
    if with_requirements {
        for vp in ["org.polarsys.kitalpha.vp.requirements", "org.polarsys.capella.vp.requirements"] {
            root.push(El::root("viewpointReferences").attr("id", &derived(&format!("{project_id}:afm:{vp}"))).attr("vpId", vp).attr("version", "0.14.0"));
        }
    }
    document(&root)
}

/// The Sirius session of the model: its resources and the Capella viewpoints.
fn aird(project_id: &str, file_stem: &str, drawn: Vec<(String, El, El)>) -> String {
    const VIEWPOINTS: &[&str] = &[
        "platform:/plugin/org.polarsys.kitalpha.ad.integration.sirius/description/ad.odesign#//@ownedViewpoints[name='ad']",
        "platform:/plugin/org.polarsys.capella.core.sirius.analysis/description/oa.odesign#//@ownedViewpoints[name='Operational%20Analysis']",
        "platform:/plugin/org.polarsys.capella.core.sirius.analysis/description/context.odesign#//@ownedViewpoints[name='System%20Analysis']",
        "platform:/plugin/org.polarsys.capella.core.sirius.analysis/description/logical.odesign#//@ownedViewpoints[name='Logical%20Architecture']",
        "platform:/plugin/org.polarsys.capella.core.sirius.analysis/description/physical.odesign#//@ownedViewpoints[name='Physical%20Architecture']",
        "platform:/plugin/org.polarsys.capella.core.sirius.analysis/description/EPBS.odesign#//@ownedViewpoints[name='EPBS%20Architecture']",
        "platform:/plugin/org.polarsys.capella.core.sirius.analysis/description/common.odesign#//@ownedViewpoints[name='Common']",
    ];
    let views: Vec<String> = (0..VIEWPOINTS.len()).map(|i| format!("_{}", derived(&format!("{project_id}:view:{i}")))).collect();
    let mut root = El::root("viewpoint:DAnalysis")
        .attr("uid", &format!("_{}", derived(&format!("{project_id}:analysis"))))
        .attr("selectedViews", &views.join(" "))
        .attr("version", "15.4.1.202403191723");
    root.push(El::text("semanticResources", &format!("{file_stem}.afm")));
    root.push(El::text("semanticResources", &format!("{file_stem}.capella")));
    let mut representations = Vec::new();
    let mut by_view: Vec<Vec<El>> = vec![Vec::new(); VIEWPOINTS.len()];
    for (odesign, descriptor, representation) in drawn {
        let view = VIEWPOINTS.iter().position(|v| v.contains(&format!("/description/{odesign}.odesign"))).unwrap_or(VIEWPOINTS.len() - 1);
        by_view[view].push(descriptor);
        representations.push(representation);
    }
    for ((uid, viewpoint), descriptors) in views.iter().zip(VIEWPOINTS).zip(by_view) {
        let mut view = El::root("ownedViews").attr("xmi:type", "viewpoint:DView").attr("uid", uid);
        for descriptor in descriptors {
            view.push(descriptor);
        }
        root.push(view.with(El::root("viewpoint").attr("xmi:type", "description:Viewpoint").attr("href", viewpoint)));
    }
    // One XMI resource: the session, then the diagrams.
    let mut resource = El::root("xmi:XMI")
        .attr("xmi:version", "2.0")
        .attr("xmlns:xmi", "http://www.omg.org/XMI")
        .attr("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance")
        .attr("xmlns:description", "http://www.eclipse.org/sirius/description/1.1.0")
        .attr("xmlns:description_1", "http://www.eclipse.org/sirius/diagram/description/1.1.0")
        .attr("xmlns:diagram", "http://www.eclipse.org/sirius/diagram/1.1.0")
        .attr("xmlns:style", "http://www.eclipse.org/sirius/diagram/description/style/1.1.0")
        .attr("xmlns:viewpoint", "http://www.eclipse.org/sirius/1.1.0");
    for (key, value) in NAMESPACES.iter().filter(|(k, _)| k.starts_with("xmlns:org.") || *k == "xmlns:libraries") {
        resource.set(key, value);
    }
    resource.push(root);
    for representation in representations {
        resource.push(representation);
    }
    document(&resource)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{capella_xmi, Compiler, CompilerConfig};
    use std::collections::HashSet;

    fn fixture() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/capella/test7_0/Model Test 7.0.capella")
    }

    fn compiled(source: &str) -> Model {
        Compiler::new(CompilerConfig { optimization_level: 0, target: "json".to_string() }).compile_string(source).expect("compiles").ast
    }

    /// The ids written in an ArcLang source.
    fn ids(source: &str) -> HashSet<String> {
        source.lines().flat_map(|l| l.match_indices("id: \"").map(move |(i, _)| l[i + 5..].split('"').next().unwrap_or("").to_string())).collect()
    }

    fn export_fixture() -> (capella_xmi::Imported, Exported) {
        let imported = capella_xmi::import_file(&fixture()).expect("imports");
        let exported = export(&compiled(&imported.source), "model");
        (imported, exported)
    }

    #[test]
    fn the_capella_7_fixture_comes_back_with_its_elements() {
        let (first, exported) = export_fixture();
        let dir = std::env::temp_dir().join(format!("arclang-export-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("model.capella"), &exported.capella).unwrap();
        std::fs::write(dir.join("model.aird"), &exported.aird).unwrap();
        let second = capella_xmi::import_file(&dir.join("model.capella")).unwrap_or_else(|f| panic!("{}", f.message));
        let (before, after) = (ids(&first.source), ids(&second.source));
        let kept = before.intersection(&after).count();
        assert!(kept * 100 >= before.len() * 99, "{kept} of {} ids come back", before.len());
        let traces = |s: &str| s.lines().filter(|l| l.starts_with("trace ")).map(str::to_string).collect::<HashSet<_>>();
        assert_eq!(traces(&first.source), traces(&second.source));
        assert!(second.counts.get("diagrams").copied().unwrap_or(0) >= 10, "{:?}", second.counts);
    }

    #[test]
    fn the_export_has_unique_ids_and_no_dangling_reference() {
        let (_, exported) = export_fixture();
        let mut seen = HashSet::new();
        for (i, _) in exported.capella.match_indices(" id=\"") {
            let id = exported.capella[i + 5..].split('"').next().unwrap();
            assert!(seen.insert(id.to_string()), "id {id} twice");
        }
        for (i, _) in exported.capella.match_indices("=\"#") {
            for reference in exported.capella[i + 2..].split('"').next().unwrap().split_whitespace() {
                let id = reference.trim_start_matches('#');
                assert!(seen.contains(id), "dangling reference {reference}");
            }
        }
        // Diagrams name elements of the model.
        for (i, _) in exported.aird.match_indices("href=\"model.capella#") {
            let id = exported.aird[i + 20..].split('"').next().unwrap();
            assert!(seen.contains(id), "a diagram names {id}, not in the model");
        }
    }

    #[test]
    fn an_export_is_the_same_every_time() {
        let (_, a) = export_fixture();
        let (_, b) = export_fixture();
        assert_eq!(a.capella, b.capella);
        assert_eq!(a.aird, b.aird);
    }
}
