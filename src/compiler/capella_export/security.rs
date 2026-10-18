//! The Cybersecurity viewpoint's own elements: the configuration that holds
//! the scales of the security needs and the kinds of threat, and per level a
//! CybersecurityPkg with its threats and primary assets. A security need or
//! a threat kind is a literal of its scale, known by a deterministic id
//! ([`literal_id`]) so that an element's needs can be written before the
//! configuration is.

use super::context::{derived, Context};
use super::header;
use super::xml::El;
use crate::compiler::ast::{AssetKind, Model, SecurityAnalysis};
use std::collections::BTreeSet;

/// The scales: `(ArcLang key, Capella attribute of the configuration, name of its enumeration)`.
pub(super) const SCALES: &[(&str, &str, &str)] = &[
    ("confidentiality", "confidentiality", "Security.Confidentiality"),
    ("integrity", "integrity", "Security.Integrity"),
    ("traceability", "traceability", "Security.Traceability"),
    ("availability", "availability", "Security.Availability"),
    ("threat_kinds", "threatKind", "Threat.Kind"),
];

/// The id of the literal `name` of a scale.
pub(super) fn literal_id(scale: &str, name: &str) -> String {
    derived(&format!("security-scale:{scale}:{name}"))
}

/// Whether the model says anything of the viewpoint's own elements.
pub(super) fn is_used(model: &Model) -> bool {
    model.security_scales.is_some() || !model.security_analysis.is_empty() || needs(model).values().any(|v| !v.is_empty())
}

/// The literal names each need takes on any element of the model: a model
/// without `security_scales` gets scales made of them.
fn needs(model: &Model) -> std::collections::BTreeMap<&'static str, BTreeSet<String>> {
    let mut used: std::collections::BTreeMap<&'static str, BTreeSet<String>> = SCALES.iter().map(|(k, _, _)| (*k, BTreeSet::new())).collect();
    let json = serde_json::to_value(model).unwrap_or_default();
    fn walk(value: &serde_json::Value, used: &mut std::collections::BTreeMap<&'static str, BTreeSet<String>>) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    if let (Some(need), Some(text)) = (key.strip_prefix("security_"), value.get("String").and_then(|s| s.as_str())) {
                        let need = need.trim_end_matches('_');
                        if let Some((scale, _, _)) = SCALES.iter().find(|(k, _, _)| *k == need) {
                            if !text.starts_with('#') {
                                used.get_mut(scale).expect("a scale").insert(text.to_string());
                            }
                        }
                    }
                    walk(value, used);
                }
            }
            serde_json::Value::Array(items) => items.iter().for_each(|i| walk(i, used)),
            _ => {}
        }
    }
    walk(&json, &mut used);
    for analysis in &model.security_analysis {
        for threat in &analysis.threats {
            if let Some(kind) = &threat.kind {
                used.get_mut("threat_kinds").expect("threat kinds").insert(kind.clone());
            }
        }
    }
    used
}

/// The CybersecurityConfiguration of the project.
pub(super) fn configuration(model: &Model, project_id: &str) -> El {
    let id = model.security_scales.as_ref().map(|s| s.id.clone()).filter(|i| !i.is_empty()).unwrap_or_else(|| derived(&format!("{project_id}:cybersecurity")));
    let used = needs(model);
    let mut el = El::new("ownedExtensions", "cybersecurity:CybersecurityConfiguration", &id).attr("name", "Cybersecurity Configuration");
    let mut types = Vec::new();
    for (key, attribute, name) in SCALES {
        let declared = model.security_scales.as_ref().and_then(|s| s.scale(key));
        let literals: Vec<String> = declared.unwrap_or_else(|| used[key].iter().cloned().collect());
        let type_id = derived(&format!("{id}:{key}"));
        el.set(attribute, &format!("#{type_id}"));
        let mut scale = El::new("ownedEnumerationPropertyTypes", "capellacore:EnumerationPropertyType", &type_id).attr("name", name);
        for literal in &literals {
            scale.push(El::new("ownedLiterals", "capellacore:EnumerationPropertyLiteral", &literal_id(key, literal)).attr("name", literal));
        }
        types.push(scale);
    }
    for scale in types {
        el.push(scale);
    }
    el
}

/// Register the threats and assets: scenarios, traces and chains name them.
pub(super) fn register(context: &mut Context, model: &Model) {
    for analysis in &model.security_analysis {
        for threat in &analysis.threats {
            context.register(&threat.name, &threat.id);
            context.register(&threat.id, &threat.id);
        }
        for asset in &analysis.assets {
            context.register(&asset.name, &asset.id);
            context.register(&asset.id, &asset.id);
        }
    }
}

/// The analyses of a level (`oa`, `sa`, `la`, `pa`) as its CybersecurityPkgs.
pub(super) fn packages(context: &mut Context, model: &Model, level: &str) -> Vec<El> {
    model.security_analysis.iter().filter(|a| a.level() == Some(level)).map(|a| package(context, a)).collect()
}

fn package(context: &mut Context, analysis: &SecurityAnalysis) -> El {
    let id = if analysis.id.is_empty() { derived(&format!("security-analysis:{}", analysis.level().unwrap_or(""))) } else { analysis.id.clone() };
    let mut pkg = El::new("ownedExtensions", "cybersecurity:CybersecurityPkg", &id).attr("name", "Cybersecurity");
    let mut unresolved = 0;
    let mut reference = |context: &Context, r: &str| match context.resolve(r).or_else(|| context.resolve_type(r)) {
        Some(id) => Some(format!("#{id}")),
        None => {
            unresolved += 1;
            None
        }
    };
    for threat in &analysis.threats {
        let mut el = El::new("ownedThreats", "cybersecurity:Threat", &threat.id).attr("name", &threat.name);
        if let Some(kind) = &threat.kind {
            el.set("kind", &format!("#{}", literal_id("threat_kinds", kind)));
        }
        if let Some(level) = threat.attributes.get("level").and_then(|l| l.as_integer()) {
            el.set("level", &level.to_string());
        }
        if let Some(rationale) = threat.attributes.get("rationale").and_then(|r| r.as_string()) {
            el.set("rationale", rationale);
        }
        let mut el = header::apply(el, &threat.attributes, &["kind", "level", "rationale", "applies_to", "involves"]);
        for (n, asset) in threat.applies_to.iter().enumerate() {
            if let Some(asset) = reference(context, asset) {
                el.push(El::new("ownedThreatApplications", "cybersecurity:ThreatApplication", &derived(&format!("{}:application:{n}", threat.id))).attr("asset", &asset));
            }
        }
        for (n, component) in threat.involves.iter().enumerate() {
            if let Some(component) = reference(context, component) {
                el.push(El::new("ownedThreatInvolvements", "cybersecurity:ThreatInvolvement", &derived(&format!("{}:involvement:{n}", threat.id))).attr("component", &component));
            }
        }
        pkg.push(el);
    }
    for asset in &analysis.assets {
        let kind = match asset.kind {
            AssetKind::Functional => "cybersecurity:FunctionalPrimaryAsset",
            AssetKind::Information => "cybersecurity:InformationPrimaryAsset",
            AssetKind::Enterprise => "cybersecurity:EnterprisePrimaryAsset",
        };
        let mut el = header::apply(El::new("ownedPrimaryAssets", kind, &asset.id).attr("name", &asset.name), &asset.attributes, &["kind", "members"]);
        for (n, member) in asset.members.iter().enumerate() {
            if let Some(member) = reference(context, member) {
                el.push(El::new("ownedMembers", "cybersecurity:PrimaryAssetMember", &derived(&format!("{}:member:{n}", asset.id))).attr("member", &member));
            }
        }
        pkg.push(el);
    }
    if unresolved > 0 {
        context.note(format!("{unresolved} references of the security analysis left out: they name no exported element"));
    }
    pkg
}
