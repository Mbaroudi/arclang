//! The STPA viewpoint (labs4capella): hazards that carry an `stpa_id` go
//! back as STPA hazards, nested by `sub_hazard_of`, with the losses they
//! lead to and the system constraints (requirements with an `stpa_id`)
//! that mitigate them, in an STPA analysis of the model.

use super::context::{derived, list, text, Context};
use super::xml::El;
use crate::compiler::ast::{Hazard, Model, Requirement};
use std::collections::HashMap;

fn stpa_hazards(model: &Model) -> Vec<&Hazard> {
    model.safety_analysis.iter().flat_map(|s| &s.hazards).filter(|h| text(&h.attributes, "stpa_id").is_some()).collect()
}

/// The requirements that are STPA system constraints.
pub(super) fn constraints(model: &Model) -> Vec<&Requirement> {
    if stpa_hazards(model).is_empty() {
        return Vec::new();
    }
    model.system_analysis.iter().flat_map(|sa| &sa.requirements).filter(|r| text(&r.attributes, "stpa_id").is_some()).collect()
}

pub(super) fn is_used(model: &Model) -> bool {
    !stpa_hazards(model).is_empty()
}

fn hazard_id(h: &Hazard) -> String {
    text(&h.attributes, "id").map(str::to_string).unwrap_or_else(|| derived(&format!("stpa:hazard:{}", h.name)))
}

pub(super) fn register(context: &mut Context, model: &Model) {
    for h in stpa_hazards(model) {
        let id = hazard_id(h);
        context.register(&id, &id);
        context.register(&h.name, &id);
    }
}

/// `L-01 Loss of life` -> (`L-01`, `Loss of life`).
fn split_loss(label: &str) -> (Option<&str>, &str) {
    match label.split_once(' ') {
        Some((first, rest)) if first.chars().any(|c| c.is_ascii_digit()) => (Some(first), rest),
        _ => (None, label),
    }
}

/// The STPA analysis, for the system engineering's `ownedExtensions`.
pub(super) fn write(context: &Context, model: &Model, engineering_id: &str) -> Option<El> {
    let hazards = stpa_hazards(model);
    if hazards.is_empty() {
        return None;
    }
    let id = derived(&format!("{engineering_id}:stpa"));
    let constraint_of = |r: &Requirement| text(&r.attributes, "capella_id").map(str::to_string).unwrap_or_else(|| r.id.clone());
    let constraints = constraints(model);
    // Losses, by label, with the hazards leading to them.
    let mut losses: Vec<(String, Vec<String>)> = Vec::new();
    for h in &hazards {
        for label in list(&h.attributes, "losses") {
            match losses.iter_mut().find(|(l, _)| *l == label) {
                Some((_, hs)) => hs.push(hazard_id(h)),
                None => losses.push((label, vec![hazard_id(h)])),
            }
        }
    }
    let loss_id = |label: &str| derived(&format!("{id}:loss:{label}"));
    let mut loss_pkg = El::new("lossPackage", "stpa:LossPackage", &derived(&format!("{id}:losses"))).attr("name", "Losses");
    for (label, hs) in &losses {
        let (user_id, name) = split_loss(label);
        loss_pkg.push(El::new("losses", "stpa:Loss", &loss_id(label)).attr("name", name).opt("userID", user_id).attr("hazards", &hs.iter().map(|h| format!("#{h}")).collect::<Vec<_>>().join(" ")));
    }
    // Hazards, nested by `sub_hazard_of`.
    let mut children: HashMap<Option<String>, Vec<&Hazard>> = HashMap::new();
    for h in &hazards {
        let parent = text(&h.attributes, "sub_hazard_of").and_then(|p| context.resolve(p)).map(str::to_string);
        children.entry(parent).or_default().push(h);
    }
    let constraint_ids: Vec<String> = constraints.iter().map(|r| constraint_of(r)).collect();
    fn hazard(tag: &str, h: &Hazard, children: &HashMap<Option<String>, Vec<&Hazard>>, context: &Context, constraint_ids: &[String], loss_id: &dyn Fn(&str) -> String) -> El {
        let hid = hazard_id(h);
        let mut el = El::new(tag, "stpa:Hazard", &hid).attr("name", &h.name).opt("userID", text(&h.attributes, "stpa_id")).opt("description", text(&h.attributes, "description"));
        let losses: Vec<String> = list(&h.attributes, "losses").iter().map(|l| format!("#{}", loss_id(l))).collect();
        if !losses.is_empty() {
            el.set("losses", &losses.join(" "));
        }
        let mitigated: Vec<String> = list(&h.attributes, "mitigated_by").iter().filter_map(|c| context.resolve(c)).filter(|c| constraint_ids.iter().any(|i| i == c)).map(|c| format!("#{c}")).collect();
        if !mitigated.is_empty() {
            el.set("systemConstraints", &mitigated.join(" "));
        }
        for sub in children.get(&Some(hid.clone())).into_iter().flatten() {
            el.push(hazard("subHazards", sub, children, context, constraint_ids, loss_id));
        }
        el
    }
    let mut hazard_pkg = El::new("hazardConstraintPackage", "stpa:HazardConstraintPackage", &derived(&format!("{id}:hazards"))).attr("name", "Hazards and Constraints");
    for h in children.get(&None).into_iter().flatten() {
        hazard_pkg.push(hazard("rootHazards", h, &children, context, &constraint_ids, &loss_id));
    }
    for r in &constraints {
        let cid = constraint_of(r);
        let mitigates: Vec<String> = hazards.iter().filter(|h| list(&h.attributes, "mitigated_by").iter().any(|m| context.resolve(m) == Some(cid.as_str()))).map(|h| format!("#{}", hazard_id(h))).collect();
        let name = text(&r.attributes, "title").or_else(|| text(&r.attributes, "description")).unwrap_or(&r.id);
        let mut el = El::new("rootSystemConstraints", "stpa:SystemConstraint", &cid).attr("name", name).opt("userID", text(&r.attributes, "stpa_id"));
        if !mitigates.is_empty() {
            el.set("hazards", &mitigates.join(" "));
        }
        hazard_pkg.push(el);
    }
    Some(El::new("ownedExtensions", "stpa:StpaAnalysis", &id).attr("name", "STPA").with(loss_pkg).with(hazard_pkg))
}
