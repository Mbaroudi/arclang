//! Relations shared by the levels: what a capability extends, includes,
//! specializes and realizes, and the steps of a chain or process.

use super::context::{derived, list, Context};
use super::xml::El;
use crate::compiler::ast::AttributeValue;
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
/// each function, a link involvement for each exchange whose two ends are
/// steps of the chain.
pub(super) fn chain_steps(context: &Context, mut el: El, chain: &str, steps: &[String], exchange_ends: &HashMap<String, (String, String)>) -> El {
    let mut involvement_of: HashMap<String, String> = HashMap::new();
    let mut links = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        let Some(involved) = context.resolve(step).map(str::to_string) else { continue };
        if let Some(ends) = exchange_ends.get(&involved) {
            links.push((index, involved, ends.clone()));
            continue;
        }
        let id = derived(&format!("{chain}:{index}"));
        involvement_of.entry(involved.clone()).or_insert_with(|| id.clone());
        el.push(El::new("ownedFunctionalChainInvolvements", "fa:FunctionalChainInvolvementFunction", &id).attr("involved", &format!("#{involved}")));
    }
    for (index, exchange, (from, to)) in links {
        if let (Some(source), Some(target)) = (involvement_of.get(&from), involvement_of.get(&to)) {
            el.push(
                El::new("ownedFunctionalChainInvolvements", "fa:FunctionalChainInvolvementLink", &derived(&format!("{chain}:{index}")))
                    .attr("involved", &format!("#{exchange}"))
                    .attr("source", &format!("#{source}"))
                    .attr("target", &format!("#{target}")),
            );
        }
    }
    el
}
