//! The Cybersecurity viewpoint (eclipse-capella/capella-cybersecurity): the
//! data a function stores, the trust boundary of a component and the actor a
//! component is used as a threat source by are extensions of that element
//! (`ownedExtensions`), not property values. The import writes them as
//! attributes of the element ([`crate::compiler::capella_xmi`]'s viewpoints);
//! here they go back to their extension.

use super::context::{derived, Context};
use super::xml::El;
use crate::compiler::ast::AttributeValue;
use std::collections::{HashMap, HashSet};

pub(super) const NAMESPACE: (&str, &str) = ("xmlns:cybersecurity", "http://www.polarsys.org/capella/cybersecurity/1.0");

/// How an attribute is written in its extension.
#[derive(Clone, Copy)]
enum Shape {
    /// A boolean, a number or a text, as it is.
    Plain,
    /// References to other elements.
    Refs,
    /// A reference, or `true` for an extension without one.
    RefOrPresence,
}

/// An extension: its kind, the kinds of element it extends (by the end of
/// their kind name), and its attributes as `(ArcLang key, Capella attribute, shape)`.
struct Extension {
    kind: &'static str,
    extends: &'static [&'static str],
    attributes: &'static [(&'static str, &'static str, Shape)],
}

const EXTENSIONS: &[Extension] = &[
    Extension {
        kind: "cybersecurity:FunctionStorage",
        extends: &["Function", "OperationalActivity"],
        attributes: &[("stored_items", "exchangedItems", Shape::Refs), ("data_storage", "dataStorage", Shape::Plain), ("remanent_data", "remanentData", Shape::Plain)],
    },
    Extension {
        kind: "cybersecurity:TrustBoundaryStorage",
        extends: &["Component", "Entity"],
        attributes: &[
            ("trusted", "trusted", Shape::Plain),
            ("threat_source", "threatSource", Shape::Plain),
            ("threat_source_profile", "threatSourceProfile", Shape::Plain),
            ("trust_rationale", "rationale", Shape::Plain),
        ],
    },
    Extension { kind: "cybersecurity:ThreatSourceUse", extends: &["Component"], attributes: &[("threat_source_use", "usedActor", Shape::RefOrPresence)] },
];

/// The reference attributes of the extensions, which [`resolve`] rewrites.
const REFERENCES: &[&str] = &["exchangedItems", "usedActor"];

fn extends(extension: &Extension, el: &El) -> bool {
    let name = el.kind().rsplit(':').next().unwrap_or("");
    extension.extends.iter().any(|end| name.ends_with(end))
}

fn texts(value: &AttributeValue) -> Vec<String> {
    match value {
        AttributeValue::List(items) => items.iter().map(AttributeValue::display).collect(),
        other => other.display().split_whitespace().map(str::to_string).collect(),
    }
}

/// `el` with the extensions its attributes describe, and the attributes they
/// took (which are then not property values). The import writes the keys of
/// a second extension of a kind with a `_` more (`trusted_`): each suffix is
/// one extension. References are written as given; [`resolve`] makes them
/// ids once every element is known.
pub(super) fn extend(mut el: El, attributes: &HashMap<String, AttributeValue>) -> (El, HashSet<String>) {
    let mut taken = HashSet::new();
    let fitting: Vec<&Extension> = EXTENSIONS.iter().filter(|e| extends(e, &el)).collect();
    for extension in fitting {
        let name = extension.kind.rsplit(':').next().unwrap_or(extension.kind);
        for nth in 0.. {
            let suffix = "_".repeat(nth);
            let present: Vec<_> = extension.attributes.iter().filter_map(|(key, attribute, shape)| attributes.get(&format!("{key}{suffix}")).map(|v| (format!("{key}{suffix}"), *attribute, *shape, v))).collect();
            if present.is_empty() {
                break;
            }
            let seed = if nth == 0 { format!("{}:{name}", el.id()) } else { format!("{}:{name}:{nth}", el.id()) };
            let mut ext = El::new("ownedExtensions", extension.kind, &derived(&seed));
            for (key, attribute, shape, value) in present {
                match shape {
                    Shape::Plain => ext.set(attribute, &value.display()),
                    Shape::Refs => ext.set(attribute, &texts(value).iter().map(|r| format!("#{}", r.trim_start_matches('#'))).collect::<Vec<_>>().join(" ")),
                    Shape::RefOrPresence => match value {
                        AttributeValue::String(flag) if flag == "true" => {}
                        AttributeValue::Boolean(true) => {}
                        other => ext.set(attribute, &format!("#{}", other.display().trim_start_matches('#'))),
                    },
                }
                taken.insert(key);
            }
            el.push(ext);
        }
    }
    (el, taken)
}

/// Rewrite the references of the extensions under `root` to ids: an id of
/// the export stays, any other name is resolved; one that resolves to
/// nothing is left out, with a note.
pub(super) fn resolve(context: &mut Context, root: &mut El) {
    let mut ids = HashSet::new();
    collect_ids(root, &mut ids);
    let mut notes = Vec::new();
    rewrite(context, root, &ids, &mut notes);
    for note in notes {
        context.note(note);
    }
}

fn collect_ids(el: &El, ids: &mut HashSet<String>) {
    ids.insert(el.id().to_string());
    for child in el.children() {
        collect_ids(child, ids);
    }
}

fn rewrite(context: &Context, el: &mut El, ids: &HashSet<String>, notes: &mut Vec<String>) {
    if el.kind().starts_with("cybersecurity:") {
        for attribute in REFERENCES {
            let Some(value) = el.get(attribute).map(str::to_string) else { continue };
            let mut kept = Vec::new();
            for reference in value.split_whitespace().map(|r| r.trim_start_matches('#')) {
                match ids.get(reference).map(String::as_str).or_else(|| context.resolve(reference).or_else(|| context.resolve_type(reference))) {
                    Some(id) => kept.push(format!("#{id}")),
                    None => notes.push(format!("{attribute} '{reference}' of {} left out: it names no exported element", el.id())),
                }
            }
            if kept.is_empty() {
                el.remove(attribute);
            } else {
                el.set(attribute, &kept.join(" "));
            }
        }
    }
    for child in el.children_mut() {
        rewrite(context, child, ids, notes);
    }
}

/// Whether the export holds an element of the viewpoint.
pub(super) fn is_used(el: &El) -> bool {
    el.kind().starts_with("cybersecurity:") || el.children().iter().any(is_used)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rendered(el: &El) -> String {
        let mut out = String::new();
        el.render(&mut out, 0);
        out
    }

    fn attributes(pairs: &[(&str, AttributeValue)]) -> HashMap<String, AttributeValue> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
    }

    #[test]
    fn the_security_attributes_of_a_function_become_its_extensions() {
        let function = El::new("ownedFunctions", "pa:PhysicalFunction", "pf1");
        let given = attributes(&[
            ("stored_items", AttributeValue::List(vec![AttributeValue::String("ei1".into())])),
            ("remanent_data", AttributeValue::String("true".into())),
            ("mass", AttributeValue::Number(2.0)),
        ]);
        let (el, taken) = extend(function, &given);
        let out = rendered(&el);
        assert!(out.contains("cybersecurity:FunctionStorage") && out.contains("exchangedItems=\"#ei1\"") && out.contains("remanentData=\"true\""), "{out}");
        assert_eq!(taken, ["stored_items", "remanent_data"].into_iter().map(String::from).collect());
        assert!(!out.contains("mass"), "a property value is the header's: {out}");
    }

    #[test]
    fn a_component_has_its_trust_boundary_and_threat_source_use() {
        let component = El::new("ownedPhysicalComponents", "pa:PhysicalComponent", "pc1");
        let given = attributes(&[("trusted", AttributeValue::String("false".into())), ("threat_source_use", AttributeValue::String("true".into()))]);
        let (el, _) = extend(component, &given);
        let out = rendered(&el);
        assert!(out.contains("cybersecurity:TrustBoundaryStorage") && out.contains("trusted=\"false\""), "{out}");
        assert!(out.contains("cybersecurity:ThreatSourceUse") && !out.contains("usedActor"), "{out}");
    }

    #[test]
    fn a_second_extension_of_a_kind_has_its_keys_suffixed_on_import() {
        let component = El::new("ownedActors", "ctx:SystemComponent", "sa2");
        let given = attributes(&[
            ("trusted", AttributeValue::String("false".into())),
            ("threat_source_profile", AttributeValue::Number(3.0)),
            ("trusted_", AttributeValue::String("false".into())),
            ("threat_source_profile_", AttributeValue::Number(2.0)),
        ]);
        let (el, taken) = extend(component, &given);
        let out = rendered(&el);
        assert_eq!(out.matches("cybersecurity:TrustBoundaryStorage").count(), 2, "{out}");
        let first = out.find("threatSourceProfile=\"3\"").expect("first");
        let second = out.find("threatSourceProfile=\"2\"").expect("second");
        assert!(first < second, "{out}");
        assert_eq!(taken.len(), 4);
    }

    #[test]
    fn an_attribute_on_an_element_the_extension_does_not_extend_stays_a_property() {
        let exchange = El::new("ownedComponentExchanges", "fa:ComponentExchange", "ce1");
        let (el, taken) = extend(exchange, &attributes(&[("trusted", AttributeValue::String("true".into()))]));
        assert!(taken.is_empty() && el.is_empty());
    }

    #[test]
    fn references_resolve_to_ids_and_unknown_ones_are_noted() {
        let mut context = Context::default();
        context.register("Video", "ei-video");
        let (function, _) = extend(
            El::new("ownedFunctions", "pa:PhysicalFunction", "pf1"),
            &attributes(&[("stored_items", AttributeValue::List(vec![AttributeValue::String("Video".into()), AttributeValue::String("Ghost".into())]))]),
        );
        let mut root = El::new("ownedModelRoots", "capellamodeller:SystemEngineering", "se").with(function);
        resolve(&mut context, &mut root);
        let out = rendered(&root);
        assert!(out.contains("exchangedItems=\"#ei-video\""), "{out}");
        assert!(context.notes.iter().any(|n| n.contains("'Ghost'")), "{:?}", context.notes);
        assert!(is_used(&root));
    }
}
