//! Lookup of model elements by reference, for views whose nodes stand for
//! elements declared elsewhere (scenario lifelines, capability involvements).

use super::builder::identity;
use super::NodeKind;
use crate::compiler::ast::{LogicalComponent, Model, OperationalActivity, SystemFunction};
use std::collections::HashMap;

/// A model element as another view refers to it.
#[derive(Debug, Clone, Copy)]
pub(super) struct ElementRef<'a> {
    pub kind: NodeKind,
    pub id: &'a str,
    pub name: &'a str,
}

/// Serialized name of a node kind (`logical_component`), as the viewer
/// reads it.
pub(super) fn kind_name(kind: NodeKind) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_else(|| "element".to_string())
}

/// Every structural or behavioural element a scenario participant or a
/// capability involvement can name, by id and by name. When a reference
/// matches several elements the first registered wins, and structure is
/// registered before behaviour: a scenario between components is the
/// common case.
pub(super) fn element_index(model: &Model) -> HashMap<&str, ElementRef<'_>> {
    let mut index = HashMap::new();
    for element in all_elements(model) {
        index.entry(element.id).or_insert(element);
        index.entry(element.name).or_insert(element);
    }
    index
}

/// Whether a node kind belongs to the operational level.
pub(super) fn is_operational(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::OperationalActor
            | NodeKind::OperationalEntity
            | NodeKind::OperationalActivity
            | NodeKind::OperationalProcess
    )
}

/// Every element another view can refer to, structure before behaviour.
pub(super) fn all_elements(model: &Model) -> Vec<ElementRef<'_>> {
    let mut found: Vec<ElementRef<'_>> = Vec::new();

    fn components<'a>(component: &'a LogicalComponent, out: &mut Vec<ElementRef<'a>>) {
        out.push(ElementRef {
            kind: NodeKind::LogicalComponent,
            id: identity(&component.attributes, &component.name),
            name: &component.name,
        });
        for sub in &component.sub_components {
            components(sub, out);
        }
    }
    fn functions<'a>(function: &'a SystemFunction, out: &mut Vec<ElementRef<'a>>) {
        out.push(ElementRef {
            kind: NodeKind::Function,
            id: identity(&function.attributes, &function.id),
            name: &function.name,
        });
        for sub in &function.sub_functions {
            functions(sub, out);
        }
    }
    fn activities<'a>(activity: &'a OperationalActivity, out: &mut Vec<ElementRef<'a>>) {
        out.push(ElementRef {
            kind: NodeKind::OperationalActivity,
            id: identity(&activity.attributes, &activity.id),
            name: &activity.name,
        });
        for sub in &activity.sub_activities {
            activities(sub, out);
        }
    }

    for la in &model.logical_architecture {
        for component in &la.components {
            components(component, &mut found);
        }
    }
    for pa in &model.physical_architecture {
        for node in &pa.nodes {
            found.push(ElementRef {
                kind: NodeKind::PhysicalNode,
                id: identity(&node.attributes, &node.name),
                name: &node.name,
            });
            for behavior in &node.behavior_components {
                found.push(ElementRef {
                    kind: NodeKind::BehaviorComponent,
                    id: &behavior.id,
                    name: &behavior.name,
                });
            }
            for hardware in &node.hardware_components {
                found.push(ElementRef {
                    kind: NodeKind::HardwareComponent,
                    id: &hardware.id,
                    name: &hardware.name,
                });
            }
        }
    }
    for sa in &model.system_analysis {
        for actor in &sa.external_actors {
            found.push(ElementRef {
                kind: NodeKind::SystemActor,
                id: identity(&actor.attributes, &actor.id),
                name: &actor.name,
            });
        }
        for component in &sa.components {
            found.push(ElementRef {
                kind: NodeKind::SystemComponent,
                id: identity(&component.attributes, &component.name),
                name: &component.name,
            });
        }
    }
    for oa in &model.operational_analysis {
        for actor in &oa.actors {
            let id = actor
                .id
                .as_deref()
                .filter(|id| !id.is_empty())
                .unwrap_or_else(|| identity(&actor.attributes, &actor.name));
            found.push(ElementRef {
                kind: NodeKind::OperationalActor,
                id,
                name: &actor.name,
            });
        }
        for entity in &oa.entities {
            found.push(ElementRef {
                kind: NodeKind::OperationalEntity,
                id: identity(&entity.attributes, &entity.id),
                name: &entity.name,
            });
            for activity in &entity.activities {
                activities(activity, &mut found);
            }
        }
        for activity in &oa.activities {
            activities(activity, &mut found);
        }
        for process in &oa.processes {
            found.push(ElementRef {
                kind: NodeKind::OperationalProcess,
                id: if process.id.is_empty() {
                    &process.name
                } else {
                    &process.id
                },
                name: &process.name,
            });
        }
    }
    for sa in &model.system_analysis {
        for function in &sa.functions {
            functions(function, &mut found);
        }
    }
    let chains = model
        .system_analysis
        .iter()
        .flat_map(|sa| &sa.functional_chains)
        .chain(
            model
                .logical_architecture
                .iter()
                .flat_map(|la| &la.functional_chains),
        );
    for chain in chains {
        let id = if chain.id.is_empty() {
            &chain.name
        } else {
            &chain.id
        };
        found.push(ElementRef {
            kind: NodeKind::FunctionalChain,
            id,
            name: &chain.name,
        });
    }

    found
}
