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
mod epbs;
mod functions;
mod header;
mod logical;
mod operational;
mod physical;
mod relations;
mod scenarios;
mod system;
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
    let name = model_name(&model);
    let project_id = context::id_of(&model.attributes, &format!("project:{name}"));
    let mut project = El::root("org.polarsys.capella.core.data.capellamodeller:Project").attr("xmi:version", "2.0");
    for (key, value) in NAMESPACES {
        project.set(key, value);
    }
    project.set("id", &project_id);
    project.set("name", &name);
    project.push(El::new("ownedExtensions", "libraries:ModelInformation", &derived(&format!("{project_id}:information"))));
    project.push(header::progress_status(&project_id, &statuses(model)));
    project.push(El::new("keyValuePairs", "capellacore:KeyValue", &derived(&format!("{project_id}:approach"))).attr("key", "projectApproach").attr("value", "SingletonComponents"));

    // Every level registers what it declares before any level refers to it.
    data::register(&mut context, model);
    for machine in &model.state_machines {
        behaviour::register(&mut context, machine);
    }
    for oa in &model.operational_analysis {
        operational::register(&mut context, oa);
    }
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
    let oa_el = match model.operational_analysis.first() {
        Some(oa) => operational::write(&mut context, oa),
        None => operational::empty(&project_id),
    };
    let oa_id = oa_el.id().to_string();
    let mut sa_el = match model.system_analysis.first() {
        Some(sa) => system::write(&mut context, sa, &derived(&format!("{oa_id}:root-activity"))),
        None => system::empty(&project_id, &derived(&format!("{oa_id}:root-activity"))),
    };
    let sa_id = sa_el.id().to_string();
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
    if model.system_analysis.len() > 1 {
        context.note(format!("{} system analyses: Capella has one, the first is exported", model.system_analysis.len()));
    }
    // State machines go to their component.
    let fallback = model
        .system_analysis
        .first()
        .and_then(|sa| sa.components.first().map(|c| context::id_of(&c.attributes, &format!("system-component:{}", c.name))));
    for machine in &model.state_machines {
        let (owner, name) = behaviour::owner_and_name(&context, machine, fallback.as_deref());
        let el = behaviour::state_machine(machine, name);
        match owner.and_then(|o| engineering.find_mut(&o)) {
            Some(owner) => owner.push(el),
            None => context.note(format!("state machine '{}' left out: no component to own it", machine.name)),
        }
    }
    for scenario in &model.scenarios {
        let (owner, name) = scenarios::owner(&context, &engineering, scenario);
        let Some(el) = scenarios::scenario(&mut context, &engineering, scenario, name) else { continue };
        match owner.and_then(|o| engineering.find_mut(&o)) {
            Some(owner) => owner.push(el),
            None => context.note(format!("scenario '{}' left out: no capability to own it", scenario.name)),
        }
    }
    project.push(engineering);

    let afm = afm(&project_id);
    let aird = aird(&project_id, file_stem);
    Exported { capella: document(&project), aird, afm, notes: context.notes }
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
fn afm(project_id: &str) -> String {
    let root = El::root("metadata:Metadata")
        .attr("xmi:version", "2.0")
        .attr("xmlns:xmi", "http://www.omg.org/XMI")
        .attr("xmlns:metadata", "http://www.polarsys.org/kitalpha/ad/metadata/1.0.0")
        .attr("id", &derived(&format!("{project_id}:afm")))
        .with(El::root("viewpointReferences").attr("id", &derived(&format!("{project_id}:afm:capella"))).attr("vpId", "org.polarsys.capella.core.viewpoint").attr("version", "7.1.0"));
    document(&root)
}

/// The Sirius session of the model: its resources and the Capella viewpoints.
fn aird(project_id: &str, file_stem: &str) -> String {
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
        .attr("xmi:version", "2.0")
        .attr("xmlns:xmi", "http://www.omg.org/XMI")
        .attr("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance")
        .attr("xmlns:description", "http://www.eclipse.org/sirius/description/1.1.0")
        .attr("xmlns:viewpoint", "http://www.eclipse.org/sirius/1.1.0")
        .attr("xsi:schemaLocation", "http://www.eclipse.org/sirius/description/1.1.0 http://www.eclipse.org/sirius/1.1.0#//description")
        .attr("uid", &format!("_{}", derived(&format!("{project_id}:analysis"))))
        .attr("selectedViews", &views.join(" "))
        .attr("version", "15.4.1.202403191723");
    root.push(El::text("semanticResources", &format!("{file_stem}.afm")));
    root.push(El::text("semanticResources", &format!("{file_stem}.capella")));
    for (uid, viewpoint) in views.iter().zip(VIEWPOINTS) {
        root.push(El::root("ownedViews").attr("xmi:type", "viewpoint:DView").attr("uid", uid).with(El::root("viewpoint").attr("xmi:type", "description:Viewpoint").attr("href", viewpoint)));
    }
    document(&root)
}
