//! A layout for a state machine region that has no diagram: ArcLang's, not the
//! connector's (which takes every position from the `.aird`). The Capella export
//! of ArcLang draws every machine (`capella_export::msm`); a Capella model whose
//! machine was never drawn, or a model read without its `.aird`, is laid out here.
//!
//! The states of a region are put on a grid, a state holding states draws them
//! in a row inside it; the figures are sized as `diagram` sizes them, and the
//! transitions run from center to center, their ends read as the connector
//! reads a diagram's (clock positions on the handle bounds).

use super::diagram::{int, location, oclock, Kind, View, IMAGE_MARGIN, SHADOW, STATE_CLIENT, STATE_CONTENT_MARGIN, STATE_EMPTY_HEIGHT, STATE_LABEL_MARGIN, TRANSLATION};
use super::font::text_width;
use super::model::Model;

/// The room between two states of a grid or of a row.
const GAP: i32 = 60;
/// A pseudo state or a final state.
const POINT_SIZE: i32 = 20;

struct Placed {
    element: usize,
    bounds: (i32, i32, i32, i32),
    shadow: bool,
}

fn is_container(m: &Model, e: usize) -> bool {
    m.is(e, "State") && !m.is(e, "FinalState")
}

fn sub_states(m: &Model, state: usize) -> Vec<usize> {
    m.owned(state, "ownedRegions").into_iter().flat_map(|r| m.owned(r, "ownedStates")).collect()
}

fn size(m: &Model, e: usize) -> (i32, i32) {
    if !is_container(m, e) {
        return (POINT_SIZE, POINT_SIZE);
    }
    let label = text_width(&m.name(e).unwrap_or_default()) + STATE_LABEL_MARGIN;
    let inner: Vec<(i32, i32)> = sub_states(m, e).into_iter().map(|s| size(m, s)).collect();
    if inner.is_empty() {
        return (label, STATE_EMPTY_HEIGHT);
    }
    let row: i32 = inner.iter().map(|s| s.0).sum::<i32>() + GAP / 3 * (inner.len() as i32 - 1);
    let tallest = inner.iter().map(|s| s.1).max().unwrap_or(0);
    (label.max(STATE_CLIENT.0 + row + STATE_CONTENT_MARGIN.0), STATE_EMPTY_HEIGHT.max(STATE_CLIENT.1 + tallest + STATE_CONTENT_MARGIN.1))
}

fn place(m: &Model, e: usize, at: (i32, i32), out: &mut Vec<Placed>) {
    let (w, h) = size(m, e);
    out.push(Placed { element: e, bounds: (at.0, at.1, w, h), shadow: is_container(m, e) });
    let mut x = at.0 + STATE_CLIENT.0;
    for s in sub_states(m, e) {
        place(m, s, (x, at.1 + STATE_CLIENT.1), out);
        x += size(m, s).0 + GAP / 3;
    }
}

/// The transitions of a region and of the regions inside its states.
fn transitions(m: &Model, region: usize) -> Vec<usize> {
    let mut out = m.owned(region, "ownedTransitions");
    for state in m.owned(region, "ownedStates") {
        for inner in m.owned(state, "ownedRegions") {
            out.extend(transitions(m, inner));
        }
    }
    out
}

/// The view of a region laid out on a grid, as `diagram` gives a drawn one.
pub fn lay_out(m: &Model, region: usize) -> View {
    let states = m.owned(region, "ownedStates");
    let columns = (states.len() as f64).sqrt().ceil().max(1.0) as usize;
    let cell_w = states.iter().map(|&s| size(m, s).0).max().unwrap_or(0) + GAP;
    let cell_h = states.iter().map(|&s| size(m, s).1).max().unwrap_or(0) + GAP;
    let mut placed = Vec::new();
    for (k, &s) in states.iter().enumerate() {
        let at = ((k % columns) as i32 * cell_w, (k / columns) as i32 * cell_h);
        place(m, s, at, &mut placed);
    }
    let origin = (-IMAGE_MARGIN, -IMAGE_MARGIN);
    let mut children: Vec<(usize, View)> = placed
        .iter()
        .map(|p| {
            let (x, y, w, h) = p.bounds;
            (p.element, View::new(Kind::Node, ((x - origin.0) as f64, (y - origin.1) as f64), (w as f64, h as f64)))
        })
        .collect();
    let find = |e: Option<usize>| e.and_then(|e| placed.iter().find(|p| p.element == e));
    for t in transitions(m, region) {
        let (Some(s), Some(d)) = (find(m.reference(t, "source")), find(m.reference(t, "target"))) else { continue };
        let handle = |p: &Placed| {
            let (x, y, w, h) = p.bounds;
            let shadow = if p.shadow { SHADOW } else { 0 };
            (x as f64, y as f64, (w - shadow) as f64, (h - shadow) as f64)
        };
        let center = |b: (f64, f64, f64, f64)| (b.0 + b.2 / 2.0, b.1 + b.3 / 2.0);
        let (sb, db) = (handle(s), handle(d));
        let from = location(sb, center(sb), center(db));
        let to = location(db, center(db), from);
        let part = |p: &Placed| (p.bounds.0 - origin.0, p.bounds.1 - origin.1, p.bounds.2, p.bounds.3);
        let src = (int(from.0) - origin.0, int(from.1) - origin.1);
        let dst = (int(to.0) - origin.0, int(to.1) - origin.1);
        let initial = m.is(s.element, "InitialPseudoState");
        let start = if initial { (src.0 as f64, src.1 as f64) } else { (oclock(part(s), src), -1.0) };
        let mut view = View::new(Kind::Edge, start, (oclock(part(d), dst), -1.0));
        view.source_oclock = !initial;
        children.push((t, view));
    }
    let right = placed.iter().map(|p| p.bounds.0 + p.bounds.2).max().unwrap_or(0);
    let bottom = placed.iter().map(|p| p.bounds.1 + p.bounds.3).max().unwrap_or(0);
    let mut diagram = View::new(Kind::Diagram, (0.0, 0.0), ((right - origin.0 + IMAGE_MARGIN) as f64, (bottom - origin.1 + IMAGE_MARGIN) as f64));
    diagram.children = children;
    diagram.origin_translate(2.0 * TRANSLATION, 2.0 * TRANSLATION);
    diagram.end_translate(3.0 * TRANSLATION, 3.0 * TRANSLATION);
    diagram
}
