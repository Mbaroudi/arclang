//! Where a state machine diagram puts its states: a layered layout of each
//! region, deterministic and without a dependency (the ELK layout of the
//! workbench runs in the browser). The transitions give the states of a
//! region a direction; the states are put in columns by the longest path
//! from the initial state (transitions that go back are left out of it),
//! ordered in each column by the mean row of the states they come from. A
//! state holding states is sized around its region, laid out the same way.
//!
//! The geometry is the one Capella draws a Mode State Machine with: a state
//! is a container whose region starts 1 px in and 17 px down (its label),
//! and the region's contents start 5 px in and 17 px down again (the
//! region's label), so a sub-state at `(x, y)` in its region is at
//! `(6, 34) + (x, y)` from its state, which is how the Simulink connector
//! reads a diagram (`simulink_connector::m2m::diagram`). Every size is
//! written: nothing depends on the fonts of the platform that opens it.

use super::xml::El;
use crate::compiler::simulink_connector::m2m::font::text_width;
use std::collections::HashMap;

/// Where a region's contents start in its state (`m2m::diagram::STATE_CLIENT`).
pub(super) const STATE_CLIENT: (i32, i32) = (6, 34);
/// The room around the states of the diagram, and inside a region.
const DIAGRAM_MARGIN: i32 = 20;
const REGION_MARGIN: i32 = 12;
/// Between two columns (room for the transitions' labels) and two rows.
const COLUMN_GAP: i32 = 80;
/// Beside a transition's label, in the gap it is drawn in.
const LABEL_GAP_ROOM: i32 = 30;
const ROW_GAP: i32 = 40;
/// The columns of a band, and the room between two bands (for the transitions going back).
const BAND_COLUMNS: usize = 8;
const BAND_GAP: i32 = 80;
/// A state holding nothing: its height, the room beside its label, its least width.
const STATE_HEIGHT: i32 = 50;
const LABEL_ROOM: i32 = 40;
const STATE_MIN_WIDTH: i32 = 90;
/// A state holding states is at least as tall as Capella's empty one (`STATE_EMPTY_HEIGHT`).
const COMPOSITE_MIN_HEIGHT: i32 = 67;
/// Room under the regions of a state: the state's label is 17 px with
/// Capella's Windows font, 26 px with the DejaVu font of Capella on Linux,
/// and a region left shorter than what it holds shows scroll bars.
const COMPOSITE_SLACK: i32 = 12;
/// A transition going back or past a box runs under the boxes of its region:
/// this far under them, then one lane further for each next one, more than a
/// label's height apart (`msm::route`). The region keeps the room for them.
pub(super) const ROUTE_DROP: i32 = 20;
pub(super) const ROUTE_STAGGER: i32 = 24;
/// The room above a state with a loop (`msm::LOOP_HEIGHT`, and its label).
const LOOP_ROOM: i32 = 40;
/// The label of a pseudo state or a final state is drawn above it, in this
/// band (`msm::Writer::state`), so that its transitions do not cross it.
pub(super) const POINT_LABEL_HEIGHT: i32 = 18;

/// A state drawn in a region: its Capella id, kind (`Mode`, `InitialPseudoState`...),
/// name, bounds in its region, and the region it holds.
pub(super) struct Placed {
    pub(super) id: String,
    pub(super) kind: String,
    pub(super) xsi_type: String,
    pub(super) name: String,
    pub(super) bounds: (i32, i32, i32, i32),
    pub(super) regions: Vec<PlacedRegion>,
}

/// A region drawn in its state: its bounds in the state's compartment, its states.
pub(super) struct PlacedRegion {
    pub(super) id: String,
    pub(super) xsi_type: String,
    pub(super) bounds: (i32, i32, i32, i32),
    pub(super) states: Vec<Placed>,
}

/// The local kind of an element: `org...capellacommon:Mode` -> `Mode`.
pub(super) fn local(kind: &str) -> &str {
    kind.rsplit(':').next().unwrap_or(kind)
}

/// Whether a state is drawn as a container (a state or a mode) or as a node (a pseudo state).
pub(super) fn is_container(kind: &str) -> bool {
    matches!(kind, "State" | "Mode")
}

/// The size of a pseudo state or a final state, as Capella's styles give it.
pub(super) fn point_size(kind: &str) -> (i32, i32) {
    match kind {
        "ChoicePseudoState" => (50, 50),
        "ForkPseudoState" | "JoinPseudoState" => (10, 70),
        _ => (20, 20),
    }
}

fn reference(value: Option<&str>) -> String {
    value.unwrap_or("").trim_start_matches('#').to_string()
}

/// Every transition under an element, at any depth: (id, source, target).
pub(super) fn transitions(el: &El) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for child in el.children() {
        if child.tag() == "ownedTransitions" {
            out.push((child.id().to_string(), reference(child.get("source")), reference(child.get("target"))));
        }
        out.extend(transitions(child));
    }
    out
}

/// The states of a region, in order.
fn states(region: &El) -> Vec<&El> {
    region.children().iter().filter(|c| c.tag() == "ownedStates").collect()
}

fn ids_within(state: &El, into: &mut Vec<String>) {
    into.push(state.id().to_string());
    for region in state.children().iter().filter(|c| c.tag() == "ownedRegions") {
        for s in states(region) {
            ids_within(s, into);
        }
    }
}

/// The states of a region laid out, with the extent of what they cover (right, bottom).
pub(super) fn region(region: &El, all: &[(String, String, String)], labels: &HashMap<String, String>, top: bool) -> (Vec<Placed>, (i32, i32)) {
    let margin = if top { DIAGRAM_MARGIN } else { REGION_MARGIN };
    let mut placed: Vec<Placed> = states(region).into_iter().map(|s| state(s, all, labels)).collect();
    if placed.is_empty() {
        return (placed, (0, 0));
    }
    // Each state of the machine, by the state of this region it is in.
    let mut at: HashMap<String, usize> = HashMap::new();
    for (k, s) in states(region).into_iter().enumerate() {
        let mut inside = Vec::new();
        ids_within(s, &mut inside);
        for id in inside {
            at.insert(id, k);
        }
    }
    let edges: Vec<(usize, usize)> = all.iter().filter_map(|(_, a, b)| Some((*at.get(a)?, *at.get(b)?))).filter(|(a, b)| a != b).collect();
    let columns = columns(&placed, &edges);
    // Between two columns, room for the labels of the transitions from one to the next.
    let column_of: HashMap<usize, usize> = columns.iter().enumerate().flat_map(|(c, members)| members.iter().map(move |&k| (k, c))).collect();
    let own: HashMap<&str, usize> = placed.iter().enumerate().map(|(k, p)| (p.id.as_str(), k)).collect();
    let mut gaps = vec![COLUMN_GAP; columns.len()];
    for (id, a, b) in all {
        let (Some(&a), Some(&b)) = (own.get(a.as_str()), own.get(b.as_str())) else { continue };
        let (ca, cb) = (column_of[&a], column_of[&b]);
        if cb == ca + 1 {
            let width = labels.get(id).map_or(0, |l| text_width(l.trim_end()));
            gaps[ca] = gaps[ca].max(width + LABEL_GAP_ROOM);
        }
    }
    // What each state takes in its column: a pseudo state its label too.
    let sizes: Vec<(i32, i32)> = placed.iter().map(room).collect();
    let width = |c: &Vec<usize>| c.iter().map(|&k| sizes[k].0).max().unwrap_or(0);
    let height = |c: &Vec<usize>| c.iter().map(|&k| sizes[k].1).sum::<i32>() + ROW_GAP * (c.len() as i32 - 1).max(0);
    // A loop goes over its state: room above the states for it.
    let loops = all.iter().any(|(_, a, b)| a == b && own.contains_key(a.as_str()));
    // A long sequence of columns wraps onto bands, one under the other.
    let mut top_of_band = margin + if loops { LOOP_ROOM } else { 0 };
    for (b, band) in columns.chunks(BAND_COLUMNS).enumerate() {
        let tallest = band.iter().map(height).max().unwrap_or(0);
        let mut x = margin;
        for (i, column) in band.iter().enumerate() {
            let w = width(column);
            let mut y = top_of_band + (tallest - height(column)) / 2;
            for &k in column {
                let (_, _, sw, sh) = placed[k].bounds;
                let above = sizes[k].1 - sh;
                placed[k].bounds = (x + (w - sw) / 2, y + above, sw, sh);
                y += sizes[k].1 + ROW_GAP;
            }
            x += w + gaps[b * BAND_COLUMNS + i];
        }
        top_of_band += tallest + BAND_GAP;
    }
    let right = placed.iter().map(|p| p.bounds.0 + p.bounds.2 + (room(p).0 - p.bounds.2) / 2).max().unwrap_or(0);
    let bottom = placed.iter().map(|p| p.bounds.1 + p.bounds.3).max().unwrap_or(0);
    (placed, (right, bottom + round_room(&edges, &column_of)))
}

/// The room under a region's states for the transitions that go round them:
/// every one but those from a column to the next may (`msm::route`).
fn round_room(edges: &[(usize, usize)], column_of: &HashMap<usize, usize>) -> i32 {
    let round = edges.iter().filter(|(a, b)| column_of[b] != column_of[a] + 1).count() as i32;
    if round == 0 {
        0
    } else {
        ROUTE_DROP + ROUTE_STAGGER * (round - 1) + POINT_LABEL_HEIGHT
    }
}

/// The room a state takes in its region: its box, and for a pseudo state or a
/// final state the label above it, centered on it.
fn room(p: &Placed) -> (i32, i32) {
    let (w, h) = (p.bounds.2, p.bounds.3);
    if is_container(&p.kind) {
        return (w, h);
    }
    (w.max(point_label_width(&p.name)), h + POINT_LABEL_HEIGHT)
}

/// The width of the label of a pseudo state or a final state.
pub(super) fn point_label_width(name: &str) -> i32 {
    if name.is_empty() {
        0
    } else {
        text_width(name) + 4
    }
}

/// A state with what it holds, sized; its position is set by its region.
fn state(s: &El, all: &[(String, String, String)], labels: &HashMap<String, String>) -> Placed {
    let kind = local(s.kind()).to_string();
    let name = s.get("name").unwrap_or("").to_string();
    let mut placed = Placed { id: s.id().to_string(), xsi_type: s.kind().to_string(), name, bounds: (0, 0, 0, 0), regions: Vec::new(), kind };
    if !is_container(&placed.kind) {
        let (w, h) = point_size(&placed.kind);
        placed.bounds = (0, 0, w, h);
        return placed;
    }
    let label = text_width(&placed.name) + LABEL_ROOM;
    let regions: Vec<&El> = s.children().iter().filter(|c| c.tag() == "ownedRegions").collect();
    if regions.is_empty() {
        placed.bounds = (0, 0, label.max(STATE_MIN_WIDTH), STATE_HEIGHT);
        return placed;
    }
    // The regions one under the other, each as tall as what it holds.
    let inner: Vec<(Vec<Placed>, (i32, i32))> = regions.iter().map(|r| region(r, all, labels, false)).collect();
    let right = inner.iter().map(|(_, (r, _))| *r).max().unwrap_or(0);
    let width = label.max(STATE_MIN_WIDTH).max(STATE_CLIENT.0 + right + REGION_MARGIN + STATE_CLIENT.0);
    let mut offset = 0;
    for (r, (states, (_, bottom))) in regions.iter().zip(inner) {
        let h = (STATE_CLIENT.1 - 17) + bottom.max(REGION_MARGIN) + REGION_MARGIN;
        placed.regions.push(PlacedRegion { id: r.id().to_string(), xsi_type: r.kind().to_string(), bounds: (0, offset, width - 4, h), states });
        offset += h;
    }
    placed.bounds = (0, 0, width, (17 + offset + 3 + COMPOSITE_SLACK).max(COMPOSITE_MIN_HEIGHT));
    placed
}

/// The columns of a region's states: by the longest path from the initial
/// state over the transitions that do not go back, each column ordered by the
/// mean row of the states its states come from.
fn columns(placed: &[Placed], edges: &[(usize, usize)]) -> Vec<Vec<usize>> {
    let n = placed.len();
    // Initial states first, then the others in order: the roots of the walk.
    let mut roots: Vec<usize> = (0..n).filter(|&k| placed[k].kind == "InitialPseudoState").collect();
    roots.extend((0..n).filter(|&k| placed[k].kind != "InitialPseudoState"));
    // 0 unseen, 1 on the path, 2 done; a transition to a state on the path goes back.
    let mut state = vec![0u8; n];
    let mut order = Vec::new();
    let mut forward: Vec<(usize, usize)> = Vec::new();
    fn visit(v: usize, edges: &[(usize, usize)], state: &mut [u8], order: &mut Vec<usize>, forward: &mut Vec<(usize, usize)>) {
        state[v] = 1;
        for &(a, b) in edges.iter().filter(|(a, _)| *a == v) {
            match state[b] {
                0 => {
                    forward.push((a, b));
                    visit(b, edges, state, order, forward);
                }
                2 => forward.push((a, b)),
                _ => {}
            }
        }
        state[v] = 2;
        order.push(v);
    }
    for &r in &roots {
        if state[r] == 0 {
            visit(r, edges, &mut state, &mut order, &mut forward);
        }
    }
    let mut layer = vec![0usize; n];
    for &v in order.iter().rev() {
        for &(_, b) in forward.iter().filter(|(a, _)| *a == v) {
            layer[b] = layer[b].max(layer[v] + 1);
        }
    }
    let count = layer.iter().max().map_or(0, |m| m + 1);
    let mut columns: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut row: Vec<f64> = vec![0.0; n];
    for (c, column) in columns.iter_mut().enumerate() {
        let mut members: Vec<(f64, usize)> = (0..n)
            .filter(|&k| layer[k] == c)
            .map(|k| {
                let from: Vec<f64> = forward.iter().filter(|(_, b)| *b == k).map(|(a, _)| row[*a]).collect();
                let mean = if from.is_empty() { k as f64 } else { from.iter().sum::<f64>() / from.len() as f64 };
                (mean, k)
            })
            .collect();
        members.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        for (r, &(_, k)) in members.iter().enumerate() {
            row[k] = r as f64;
        }
        *column = members.into_iter().map(|(_, k)| k).collect();
    }
    columns
}

/// The absolute bounds of every state of a diagram, by id, as the diagram draws them.
pub(super) fn absolute(placed: &[Placed], origin: (i32, i32), into: &mut HashMap<String, (i32, i32, i32, i32)>) {
    for p in placed {
        let (x, y, w, h) = p.bounds;
        let (ax, ay) = (origin.0 + x, origin.1 + y);
        into.insert(p.id.clone(), (ax, ay, w, h));
        for r in &p.regions {
            absolute(&r.states, (ax + STATE_CLIENT.0, ay + STATE_CLIENT.1 + r.bounds.1), into);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(kind: &str, id: &str) -> El {
        El::new("ownedStates", kind, id).attr("name", id)
    }

    fn tr(id: &str, from: &str, to: &str) -> El {
        El::new("ownedTransitions", "capellacommon:StateTransition", id).attr("source", &format!("#{from}")).attr("target", &format!("#{to}"))
    }

    #[test]
    fn states_follow_their_transitions_from_left_to_right() {
        let region = El::new("ownedRegions", "capellacommon:Region", "r")
            .with(st("capellacommon:InitialPseudoState", "i"))
            .with(st("capellacommon:Mode", "a"))
            .with(st("capellacommon:Mode", "b"))
            .with(tr("t0", "i", "a"))
            .with(tr("t1", "a", "b"))
            .with(tr("t2", "b", "a"));
        let all = transitions(&region);
        let (placed, (right, bottom)) = super::region(&region, &all, &HashMap::new(), true);
        let x = |id: &str| placed.iter().find(|p| p.id == id).unwrap().bounds.0;
        assert!(x("i") < x("a") && x("a") < x("b"), "{:?}", placed.iter().map(|p| (&p.id, p.bounds)).collect::<Vec<_>>());
        assert!(right > x("b") && bottom > 0);
    }

    #[test]
    fn a_state_is_sized_around_its_region() {
        let inner = El::new("ownedRegions", "capellacommon:Region", "r2").with(st("capellacommon:State", "x")).with(st("capellacommon:State", "y")).with(tr("t", "x", "y"));
        let outer = st("capellacommon:State", "s").with(inner);
        let region = El::new("ownedRegions", "capellacommon:Region", "r").with(outer);
        let all = transitions(&region);
        let (placed, _) = super::region(&region, &all, &HashMap::new(), true);
        let s = &placed[0];
        let mut at = HashMap::new();
        absolute(&placed, (0, 0), &mut at);
        for id in ["x", "y"] {
            let (x, y, w, h) = at[id];
            assert!(x + w <= s.bounds.0 + s.bounds.2 && y + h <= s.bounds.1 + s.bounds.3, "{id} is inside its state");
            assert!(x >= s.bounds.0 + STATE_CLIENT.0 && y >= s.bounds.1 + STATE_CLIENT.1);
        }
    }
}
