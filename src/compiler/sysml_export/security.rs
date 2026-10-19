//! The cybersecurity analysis: each `security_analysis` a `#Security`
//! package (its level an attribute), each primary asset a `#PrimaryAsset`
//! part, each threat a `#Threat` part, and what they name (`members`,
//! `applies_to`, `involves`) dependencies named by their key. The scales
//! are a `#SecurityScales` package holding one enumeration per scale, its
//! literals in order.

use super::relations::dependency;
use super::{id_of, text, Exporter};
use crate::compiler::ast::*;

/// The package of an analysis: its id, else `Security <level>`.
/// As written in a qualified name (`'SEC-SA'::'AS-PLAN'`).
fn package_name(analysis: &SecurityAnalysis, rank: usize) -> String {
    let name = if !analysis.id.is_empty() {
        analysis.id.clone()
    } else {
        match analysis.level() {
            Some(level) => format!("Security {level}"),
            None => format!("Security {}", rank + 1),
        }
    };
    text::name(&name)
}

pub(super) fn register(e: &mut Exporter<'_>) {
    let model = e.model;
    for (rank, analysis) in model.security_analysis.iter().enumerate() {
        let package = package_name(analysis, rank);
        for asset in &analysis.assets {
            e.register(&package, id_of(&asset.attributes, &asset.id), &[&asset.name]);
        }
        for threat in &analysis.threats {
            e.register(&package, id_of(&threat.attributes, &threat.id), &[&threat.name]);
        }
    }
}

pub(super) fn write(e: &mut Exporter<'_>) {
    let model = e.model;
    if let Some(scales) = &model.security_scales {
        // No short name when none is written: it would hide the metadata.
        let head = if scales.id.is_empty() { text::name("Security scales") } else { text::declared(&scales.id, "Security scales") };
        e.block(&format!("#SecurityScales package {head}"), |e| {
            // The scales are lists of literals; anything else is an attribute.
            let is_list = |k: &String| matches!(scales.attributes.get(k), Some(AttributeValue::List(_)));
            let others: Vec<&str> = scales.attributes.keys().filter(|k| is_list(k)).map(String::as_str).collect();
            e.header("SecurityScales", &scales.attributes, &others);
            let mut keys: Vec<&String> = scales.attributes.keys().filter(|k| k.as_str() != "id" && is_list(k)).collect();
            keys.sort();
            for key in keys {
                let literals = scales.scale(key).unwrap_or_default();
                e.block(&format!("enum def {}", text::name(key)), |e| {
                    for literal in &literals {
                        e.line(&format!("{};", text::name(literal)));
                    }
                });
            }
        });
    }
    for (rank, analysis) in model.security_analysis.iter().enumerate() {
        let package = package_name(analysis, rank);
        e.package(&format!("#Security package {package}"), &package, |e| {
            e.header("SecurityAnalysis", &analysis.attributes, &[]);
            for asset in &analysis.assets {
                let id = id_of(&asset.attributes, &asset.id).to_string();
                e.block(&format!("#PrimaryAsset part {}", text::declared(&id, &asset.name)), |e| {
                    kind(e, &asset.attributes, Some(asset.kind.as_str()));
                    e.header("PrimaryAsset", &asset.attributes, &["members", "kind"]);
                });
                for member in &asset.members {
                    dependency(e, "members", &id, member);
                }
            }
            for threat in &analysis.threats {
                let id = id_of(&threat.attributes, &threat.id).to_string();
                e.block(&format!("#Threat part {}", text::declared(&id, &threat.name)), |e| {
                    kind(e, &threat.attributes, threat.kind.as_deref());
                    e.header("Threat", &threat.attributes, &["applies_to", "involves", "kind"]);
                });
                for asset in &threat.applies_to {
                    dependency(e, "applies_to", &id, asset);
                }
                for involved in &threat.involves {
                    dependency(e, "involves", &id, involved);
                }
            }
        });
    }
}

/// `kind`, which the parser keeps apart from the attributes.
fn kind(e: &mut Exporter<'_>, attributes: &std::collections::HashMap<String, AttributeValue>, kind: Option<&str>) {
    let written = attributes.get("kind").and_then(|k| k.as_string()).or(kind);
    if let Some(kind) = written {
        e.line(&format!("attribute kind : String = {};", text::string(kind)));
    }
}
