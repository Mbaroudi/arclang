//! The CESAM grid of a model following CESAM (`method: cesam`), as SysML v2
//! views: one `view` per view of the grid that holds something, exposing
//! the exported elements placed in it, with its vision and aspect as
//! attributes. A tool reading the SysML (Cameo, Enterprise Architect,
//! Rhapsody) gets the model organized the CESAM way, as CESAMES maps the
//! framework onto each tool.

use super::{text, Exporter};
use crate::compiler::cesam::{self, Classifier, Placement, ASPECTS, VISIONS};
use crate::compiler::{cesam_rules, elements, semantic};
use std::collections::BTreeMap;

pub(super) fn write(e: &mut Exporter<'_>) {
    let model = e.model;
    if !cesam_rules::declared(model) {
        return;
    }
    let Ok(semantic) = semantic::SemanticAnalyzer::new().analyze(model) else {
        e.line("// CESAM views left out: the model does not analyze");
        return;
    };
    let graph = elements::build(model, &semantic);
    let classifier = Classifier::new(&graph, model);
    // Placement -> the references of its exported elements, in graph order, once each.
    let mut cells: BTreeMap<(usize, usize), Vec<String>> = BTreeMap::new();
    for element in &graph.elements {
        let Some(Placement { vision, aspect }) = classifier.element(element) else { continue };
        let Some(reference) = e.reference(&element.id).or_else(|| e.reference(&element.name)) else { continue };
        let key = (VISIONS.iter().position(|v| *v == vision).unwrap_or(0), ASPECTS.iter().position(|a| *a == aspect).unwrap_or(0));
        let exposed = cells.entry(key).or_default();
        if !exposed.contains(&reference) {
            exposed.push(reference);
        }
    }
    for ((v, a), exposed) in cells {
        let view = cesam::view(VISIONS[v], ASPECTS[a]);
        e.block(&format!("view {}", text::name(&format!("CESAM {}", view.name))), |e| {
            e.line(&format!("attribute vision : String = {};", text::string(VISIONS[v].key())));
            e.line(&format!("attribute aspect : String = {};", text::string(ASPECTS[a].key())));
            for reference in &exposed {
                e.line(&format!("expose {reference};"));
            }
        });
    }
}
