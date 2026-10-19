//! Relations shared by the levels: what a capability extends, includes,
//! specializes and realizes, and the steps of a chain or process.

use super::context::{derived, list, text, Context};
use super::header::apply;
use super::xml::El;
use crate::compiler::ast::{AttributeValue, ControlKind, FunctionalChain};
use std::collections::HashMap;

/// `extends`, `includes` and `specializes` of a capability, and its
/// `realizes` (the capability of the level above it realizes).
pub(super) fn capability(context: &Context, mut el: El, id: &str, attributes: &HashMap<String, AttributeValue>, realizes: Option<&str>) -> El {
    for (key, tag, kind, reference) in [
        ("extends", "extends", "interaction:AbstractCapabilityExtend", "extended"),
        ("includes", "includes", "interaction:AbstractCapabilityInclude", "included"),
        ("specializes", "superGeneralizations", "interaction:AbstractCapabilityGeneralization", "super"),
    ] {
        for other in list(attributes, key) {
            if let Some(target) = context.resolve(&other) {
                el.push(El::new(tag, kind, &derived(&format!("{id}->{target}:{key}"))).attr(reference, &format!("#{target}")));
            }
        }
    }
    if let Some(target) = realizes.and_then(|r| context.resolve(r)) {
        el.push(
            El::new("ownedAbstractCapabilityRealizations", "interaction:AbstractCapabilityRealization", &derived(&format!("{id}->{target}:realization")))
                .attr("targetElement", &format!("#{target}"))
                .attr("sourceElement", &format!("#{id}")),
        );
    }
    el
}

/// The steps of a chain (or process), in order: a function involvement for
/// each function, a reference for each chain it involves, a link
/// involvement for each exchange whose two ends are steps of the chain;
/// then its control nodes and the sequence links between steps and nodes.
pub(super) fn chain_steps(context: &Context, mut el: El, chain: &FunctionalChain, exchange_ends: &HashMap<String, (String, String)>) -> El {
    let id = chain.id.as_str();
    let mut involvement_of: HashMap<String, String> = HashMap::new();
    let mut links = Vec::new();
    for (index, step) in chain.involves.iter().enumerate() {
        let Some(involved) = context.resolve(step).map(str::to_string) else { continue };
        if let Some(ends) = exchange_ends.get(&involved) {
            links.push((index, involved, ends.clone()));
            continue;
        }
        let involvement = derived(&format!("{id}:{index}"));
        involvement_of.entry(involved.clone()).or_insert_with(|| involvement.clone());
        involvement_of.entry(step.clone()).or_insert_with(|| involvement.clone());
        let kind = if context.chains.contains(&involved) { "fa:FunctionalChainReference" } else { "fa:FunctionalChainInvolvementFunction" };
        el.push(El::new("ownedFunctionalChainInvolvements", kind, &involvement).attr("involved", &format!("#{involved}")));
    }
    for (index, exchange, (from, to)) in links {
        if let (Some(source), Some(target)) = (involvement_of.get(&from), involvement_of.get(&to)) {
            el.push(
                El::new("ownedFunctionalChainInvolvements", "fa:FunctionalChainInvolvementLink", &derived(&format!("{id}:{index}")))
                    .attr("involved", &format!("#{exchange}"))
                    .attr("source", &format!("#{source}"))
                    .attr("target", &format!("#{target}")),
            );
        }
    }
    let mut node_of: HashMap<String, String> = HashMap::new();
    for control in &chain.controls {
        let cid = if control.id.is_empty() { derived(&format!("{id}:control:{}", control.name)) } else { control.id.clone() };
        node_of.insert(control.name.clone(), cid.clone());
        node_of.insert(control.id.clone(), cid.clone());
        let kind = match control.kind {
            ControlKind::And => "AND",
            ControlKind::Or => "OR",
            ControlKind::Iterate => "ITERATE",
        };
        el.push(El::new("ownedSequenceNodes", "fa:ControlNode", &cid).attr("kind", kind));
    }
    let end = |reference: &str| node_of.get(reference).or_else(|| involvement_of.get(reference)).or_else(|| context.resolve(reference).and_then(|r| involvement_of.get(r))).cloned();
    for (index, link) in chain.sequences.iter().enumerate() {
        let (Some(source), Some(target)) = (end(&link.from), end(&link.to)) else { continue };
        let lid = if link.id.is_empty() { derived(&format!("{id}:sequence:{index}")) } else { link.id.clone() };
        let mut sequence = El::new("ownedSequenceLinks", "fa:SequenceLink", &lid).attr("source", &format!("#{source}")).attr("target", &format!("#{target}"));
        if let Some(condition) = link.condition.as_deref().filter(|c| !c.is_empty()) {
            let constraint = derived(&format!("{lid}:condition"));
            sequence.set("condition", &format!("#{constraint}"));
            sequence.push(
                El::new("ownedConstraints", "capellacore:Constraint", &constraint)
                    .with(El::new("ownedSpecification", "datavalue:OpaqueExpression", &derived(&format!("{lid}:condition:expression"))).with(El::text("bodies", condition)).with(El::text("languages", "capella:linkedText"))),
            );
        }
        el.push(sequence);
    }
    el
}

/// What an exchange carries besides its ends: its exchange items, its
/// categories (kept for the function package), its header.
pub(super) fn exchange(context: &mut Context, el: El, attributes: &HashMap<String, AttributeValue>) -> El {
    let mut el = apply(el, attributes, &["from", "to", "from_port", "to_port", "ends", "exchange_items", "category", "categories", "label", "via"]);
    let items: Vec<String> = list(attributes, "exchange_items").iter().filter_map(|i| context.resolve_type(i).or_else(|| context.resolve(i)).map(|t| format!("#{t}"))).collect();
    if !items.is_empty() {
        el.set("exchangedItems", &items.join(" "));
    }
    let mut categories = list(attributes, "categories");
    categories.extend(text(attributes, "category").map(str::to_string));
    for category in categories {
        context.categories.push((category, el.id().to_string()));
    }
    el
}

/// The exchange categories, each naming its exchanges.
pub(super) fn categories(context: &Context, owner: &str) -> Vec<El> {
    let mut names: Vec<&str> = Vec::new();
    for (name, _) in &context.categories {
        if !names.contains(&name.as_str()) {
            names.push(name);
        }
    }
    names
        .into_iter()
        .map(|name| {
            let exchanges: Vec<String> = context.categories.iter().filter(|(n, _)| n == name).map(|(_, e)| format!("#{e}")).collect();
            El::new("ownedCategories", "fa:ExchangeCategory", &derived(&format!("{owner}:category:{name}"))).attr("name", name).attr("exchanges", &exchanges.join(" "))
        })
        .collect()
}
