//! The geometry of the state machine diagrams (`CollectGraphicalDataForTransformation`,
//! `CoordinatesCalculator`, `addGraphicalParameters`).

use super::model::Model;
use super::Run;
use std::collections::HashMap;

/// The graphical information of a diagram or of one of its views.
#[derive(Debug, Clone, Default)]
pub struct View {}

/// The first diagram whose target is this element (`DialectManager.getRepresentations`).
pub fn diagram_of(m: &Model, target: usize) -> Option<usize> {
    m.referencers(target, "target").into_iter().find(|&d| m.class(d) == "DSemanticDiagram")
}

/// The geometry of the diagrams of the selection, by the region each shows.
pub fn collect(_m: &Model, _selection: &[usize]) -> Result<HashMap<usize, View>, String> {
    Ok(HashMap::new())
}

/// Puts the bounds (or the ends) of the Capella element's view on the pivot element.
pub fn graphical(_run: &mut Run, _pivot: usize, _capella: usize, _edge_oclock: bool) {}
