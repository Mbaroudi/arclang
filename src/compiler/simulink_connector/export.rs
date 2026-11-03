//! `arclang export <model> -f simulink-connector`: what the connector's export
//! does from Capella, from a `.capella` file or from an ArcLang model (written
//! as Capella first, by the Capella exporter). The elements are chosen by id or
//! by name; without any, the strategy's natural roots are taken.

use super::m2m::{self, model::Model, Options, Strategy};
use super::{mlcore, script};

/// What the export makes: the pivot (`.mlcore`), the MATLAB script, and notes.
pub struct Exported {
    pub pivot: String,
    pub script: String,
    pub notes: Vec<String>,
}

/// The class the elements of a strategy are chosen among.
fn chosen_class(strategy: Strategy) -> &'static str {
    match strategy {
        Strategy::Functions => "AbstractFunction",
        Strategy::FunctionalChains => "FunctionalChain",
        Strategy::StateMachines => "StateMachine",
        Strategy::DataPackages => "AbstractDependenciesPkg",
    }
}

/// The ids of the elements named: an id as it is, else the one element of the strategy's
/// class with that name.
fn resolve(m: &Model, strategy: Strategy, names: &[String]) -> Result<Vec<String>, String> {
    let class = chosen_class(strategy);
    let mut ids = Vec::new();
    for name in names {
        if m.by_id(name).is_some() {
            ids.push(name.clone());
            continue;
        }
        let found: Vec<usize> = (0..m.elements.len()).filter(|&e| m.is(e, class) && m.name(e).as_deref() == Some(name.as_str())).collect();
        match found.as_slice() {
            [e] => ids.push(m.id(*e).unwrap_or_default()),
            [] => return Err(format!("no {class} named or identified '{name}'")),
            several => {
                let list: Vec<String> = several.iter().map(|&e| m.id(e).unwrap_or_default()).collect();
                return Err(format!("several elements are named '{name}': choose by id ({})", list.join(", ")));
            }
        }
    }
    Ok(ids)
}

/// The elements taken when none is named: the functions under each layer's root function, the
/// functional chains, the state machines whose regions all have a diagram, the data packages
/// of the layers.
fn defaults(m: &Model, strategy: Strategy, notes: &mut Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for e in 0..m.elements.len() {
        let take = match strategy {
            Strategy::Functions => m.is(e, "AbstractFunction") && m.container(e).is_some_and(|c| m.is(c, "AbstractFunction") && m.container(c).is_some_and(|p| m.is(p, "FunctionPkg"))),
            Strategy::FunctionalChains => m.is(e, "FunctionalChain"),
            Strategy::StateMachines => {
                let machine = m.is(e, "StateMachine");
                let drawn = m.owned(e, "ownedRegions").iter().all(|&r| m2m::has_diagram(m, r));
                if machine && !drawn {
                    notes.push(format!("the state machine '{}' is left out: a region of it has no diagram, which gives the chart its layout", m.name(e).unwrap_or_default()));
                }
                machine && drawn
            }
            Strategy::DataPackages => m.is(e, "DataPkg") && m.container(e).is_some_and(|c| m.is(c, "BlockArchitecture")),
        };
        if take {
            out.extend(m.id(e));
        }
    }
    out
}

/// Exports the elements of a Capella model.
pub fn export(m: &Model, strategy: Strategy, elements: &[String], options: &Options) -> Result<Exported, String> {
    let mut notes = Vec::new();
    let ids = if elements.is_empty() { defaults(m, strategy, &mut notes) } else { resolve(m, strategy, elements)? };
    if ids.is_empty() {
        return Err(format!("the model has nothing to export with the {strategy:?} strategy"));
    }
    let refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    let pivot = m2m::transform(m, strategy, &refs, options)?;
    let script = script(&pivot)?;
    Ok(Exported { pivot: mlcore::write(&pivot), script, notes })
}
