//! Product lines, as SysML v2 has them (`variation` / `variant`).
//!
//! ```text
//! component "Engine" {
//!     variation: true
//!     component "Four cylinders" { id: "LC-4CYL" variant: true }
//!     component "Six cylinders" { id: "LC-6CYL" variant: true }
//! }
//! configuration "City" { selects: ["LC-4CYL", "LC-MANUAL"] }
//! ```
//!
//! A component `variation: true` is a point of variation; the components
//! it holds marked `variant: true` are its variants. A configuration is one
//! product: it selects one variant of every variation.
//!
//! Errors: a variant outside a variation, anything else inside one; a configuration selecting what is
//! not a variant, or two variants of one variation. Warnings: a variation
//! with fewer than two variants; a configuration leaving a variation open.

use super::ast::*;
use std::collections::{HashMap, HashSet};

/// A point of variation and its variants, by written id.
#[derive(Debug, Clone)]
pub struct Variation {
    pub id: String,
    pub variants: Vec<(String, String)>,
    /// The components holding it, outermost first: a variation inside a
    /// variant is open only in a product that selects that variant.
    pub within: Vec<String>,
}

fn flag(attributes: &HashMap<String, AttributeValue>, key: &str) -> bool {
    match attributes.get(key) {
        Some(AttributeValue::Boolean(on)) => *on,
        Some(other) => other.as_string() == Some("true"),
        None => false,
    }
}

/// Whether an element is a point of variation (`variation: true`).
pub fn is_variation(attributes: &HashMap<String, AttributeValue>) -> bool {
    flag(attributes, "variation")
}

/// Whether an element is one variant of its variation (`variant: true`).
pub fn is_variant(attributes: &HashMap<String, AttributeValue>) -> bool {
    flag(attributes, "variant")
}

fn written_id(component: &LogicalComponent) -> String {
    component.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&component.id).to_string()
}

/// Every variation of the logical architecture, with its variants.
pub fn variations(ast: &Model) -> Vec<Variation> {
    fn walk(components: &[LogicalComponent], within: &mut Vec<String>, out: &mut Vec<Variation>) {
        for component in components {
            if is_variation(&component.attributes) {
                let variants = component.sub_components.iter().filter(|c| is_variant(&c.attributes)).map(|c| (written_id(c), c.name.clone())).collect();
                out.push(Variation { id: written_id(component), variants, within: within.clone() });
            }
            within.push(written_id(component));
            walk(&component.sub_components, within, out);
            within.pop();
        }
    }
    let mut out = Vec::new();
    for la in &ast.logical_architecture {
        walk(&la.components, &mut Vec::new(), &mut out);
    }
    out
}

/// The variant a selection names: by id, else by a name only one variant
/// bears. `Err` lists the variations whose variants bear the name.
fn resolve(variations: &[Variation], selected: &str) -> Result<Option<(String, String)>, Vec<String>> {
    for variation in variations {
        if let Some((id, _)) = variation.variants.iter().find(|(id, _)| id == selected) {
            return Ok(Some((variation.id.clone(), id.clone())));
        }
    }
    let named: Vec<(String, String)> = variations
        .iter()
        .flat_map(|v| v.variants.iter().filter(|(_, name)| name == selected).map(move |(id, _)| (v.id.clone(), id.clone())))
        .collect();
    match named.as_slice() {
        [] => Ok(None),
        [one] => Ok(Some(one.clone())),
        many => Err(many.iter().map(|(variation, _)| variation.clone()).collect()),
    }
}

/// The variants a configuration leaves out, by id: those of its variations
/// it does not select.
pub fn excluded_by(ast: &Model, configuration: &str) -> HashSet<String> {
    let Some(config) = ast.configurations.iter().find(|c| c.id == configuration || c.name == configuration) else { return HashSet::new() };
    let variations = variations(ast);
    let selected: HashSet<String> = config.selects.iter().filter_map(|s| resolve(&variations, s).ok().flatten()).map(|(_, id)| id).collect();
    variations.iter().flat_map(|v| v.variants.iter().map(|(id, _)| id.clone())).filter(|id| !selected.contains(id)).collect()
}

/// Check variations, variants and configurations: `Err` for what cannot be
/// a product line, the warnings otherwise.
pub fn check(ast: &Model) -> Result<Vec<String>, String> {
    fn misplaced(components: &[LogicalComponent], in_variation: bool) -> Option<String> {
        for component in components {
            if is_variant(&component.attributes) && !in_variation {
                return Some(format!("'{}' is a variant outside a variation: put it in a component `variation: true`", written_id(component)));
            }
            if !is_variant(&component.attributes) && in_variation {
                return Some(format!("'{}' is in a variation but is not a variant: a variation holds only its variants (`variant: true`)", written_id(component)));
            }
            if let Some(found) = misplaced(&component.sub_components, is_variation(&component.attributes)) {
                return Some(found);
            }
        }
        None
    }
    for la in &ast.logical_architecture {
        if let Some(error) = misplaced(&la.components, false) {
            return Err(error);
        }
    }
    let variations = variations(ast);
    let mut warnings = Vec::new();
    for variation in &variations {
        if variation.variants.len() < 2 {
            warnings.push(format!("variation '{}' has {} variant(s): a variation chooses between two or more", variation.id, variation.variants.len()));
        }
    }
    for config in &ast.configurations {
        let mut chosen: HashMap<String, String> = HashMap::new();
        for selected in &config.selects {
            let (variation, _) = resolve(&variations, selected)
                .map_err(|holders| format!("configuration '{}': '{selected}' names variants of several variations ({}): select it by id", config.name, holders.join(", ")))?
                .ok_or_else(|| format!("configuration '{}': '{selected}' is not a variant of any variation", config.name))?;
            if let Some(previous) = chosen.insert(variation.clone(), selected.clone()).filter(|previous| previous != selected) {
                return Err(format!("configuration '{}' selects '{previous}' and '{selected}', two variants of variation '{variation}'", config.name));
            }
        }
        let excluded = excluded_by(ast, &config.id);
        for variation in &variations {
            if !chosen.contains_key(&variation.id) && !variation.within.iter().any(|holder| excluded.contains(holder)) {
                warnings.push(format!("configuration '{}' leaves variation '{}' open", config.name, variation.id));
            }
        }
    }
    Ok(warnings)
}
