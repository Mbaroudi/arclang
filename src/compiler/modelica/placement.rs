//! The drawing of the package: where each part sits on the diagram of the
//! model that holds it, which side of its box each connector is on, and
//! the right-angled line each `connect` is drawn with.
//!
//! Graphical Modelica editors (Modex, OMEdit) draw from `Placement`,
//! `Line`, `Icon` and `Diagram` annotations; without them the canvas is
//! blank. Nothing here changes what the package means: the annotations are
//! comments to the compiler, and the simulation path leaves them out.
//!
//! The layout is a simple layered one, deterministic and without a
//! dependency: connections give the parts a direction, the parts are put in
//! columns by the longest path from a source (inputs on the left, outputs
//! on the right), ordered in each column by the mean row of their
//! neighbours, and a column holds at most [`MAX_ROWS`] boxes. A
//! `<model>.layout.json` moves a box from there (see [`offsets`]).
//! Y axis up, as Modelica has it.

use super::connector::{Orientation, Physics};
use super::structure::{Connection, Part, Port};
use crate::compiler::diagram::layout_file::LayoutFile;
use std::collections::BTreeMap;

const BOX_W: f64 = 40.0;
const MIN_BOX_H: f64 = 24.0;
const MAX_BOX_H: f64 = 64.0;
const COLUMN_GAP: f64 = 36.0;
const ROW_GAP: f64 = 16.0;
/// Boxes in a column before it wraps to a new one.
const MAX_ROWS: usize = 4;
/// Room between the boxes and the border connectors of the model.
const BORDER: f64 = 30.0;
/// The size (half) of a border connector on the diagram and on an icon.
pub const DIAGRAM_PORT: f64 = 7.0;
const BORDER_PORT_PITCH: f64 = 16.0;
const STUB_CHILD: f64 = 5.0;
const STUB_BORDER: f64 = 16.0;
const LANE_PITCH: f64 = 4.0;
/// One sheet unit of the workbench layout file, in Modelica units.
const SHEET_TO_MODELICA: f64 = 0.1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    fn sign(self) -> f64 {
        match self {
            Side::Left => -1.0,
            Side::Right => 1.0,
        }
    }
}

/// Which side of its box each connector of each part is drawn on.
///
/// Inputs and outputs are on the left and on the right. A connector with no
/// direction (a pin, a flange, a heat port) goes where the flow of the
/// connections as written leaves it: on the right when it is the source of
/// the connections that reach it from outside the part, on the left when it
/// is their target; a tie, or none, falls back on the Modelica Standard
/// Library's own side (`_a` and positive left, `_b` and negative right).
/// One side per class, whatever diagram draws it, since the icon is the
/// class's.
#[derive(Debug, Default)]
pub struct Sides(Vec<Vec<Side>>);

impl Sides {
    pub fn of(parts: &[Part], connections: &[Connection]) -> Sides {
        let sides = parts
            .iter()
            .enumerate()
            .map(|(p, part)| {
                part.ports
                    .iter()
                    .enumerate()
                    .map(|(q, port)| match port.orientation {
                        Orientation::In => Side::Left,
                        Orientation::Out => Side::Right,
                        _ => flow_side(connections, p, q).unwrap_or_else(|| standard_side(port)),
                    })
                    .collect()
            })
            .collect();
        Sides(sides)
    }

    pub fn side(&self, part: usize, port: usize) -> Side {
        self.0[part][port]
    }

    /// The ports of a part on each side, in declaration order.
    pub fn lists(&self, part: usize) -> (Vec<usize>, Vec<usize>) {
        let mut left = Vec::new();
        let mut right = Vec::new();
        for (port, side) in self.0[part].iter().enumerate() {
            match side {
                Side::Left => left.push(port),
                Side::Right => right.push(port),
            }
        }
        (left, right)
    }
}

fn standard_side(port: &Port) -> Side {
    if port.connector.ends_with("_b") || port.connector.ends_with("NegativePin") {
        Side::Right
    } else {
        Side::Left
    }
}

/// The side the connections that reach a port from outside its part give it.
fn flow_side(connections: &[Connection], part: usize, port: usize) -> Option<Side> {
    let (mut source, mut target) = (0, 0);
    for connection in connections {
        let holder = connection.within;
        if holder == Some(part) {
            continue;
        }
        let [from, to] = connection.ends;
        if from == (part, port) {
            source += 1;
        } else if to == (part, port) {
            target += 1;
        }
    }
    match source.cmp(&target) {
        std::cmp::Ordering::Greater => Some(Side::Right),
        std::cmp::Ordering::Less => Some(Side::Left),
        std::cmp::Ordering::Equal => None,
    }
}

/// The colour a connection is drawn in, by the physics it carries.
pub fn color(physics: Physics) -> &'static str {
    match physics {
        Physics::Electrical => "{0,0,255}",
        Physics::Signal => "{0,0,127}",
        Physics::Translational | Physics::Rotational => "{95,95,95}",
        Physics::Thermal => "{191,0,0}",
        Physics::Fluid => "{0,127,255}",
    }
}

/// A number as Modelica text: one decimal at most.
pub fn num(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if rounded == 0.0 {
        "0".to_string()
    } else if rounded.fract() == 0.0 {
        format!("{}", rounded as i64)
    } else {
        format!("{rounded}")
    }
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    left: f64,
    bottom: f64,
    right: f64,
    top: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Placed {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Placed {
    fn rect(&self) -> Rect {
        Rect { left: self.x - self.w / 2.0, bottom: self.y - self.h / 2.0, right: self.x + self.w / 2.0, top: self.y + self.h / 2.0 }
    }
}

/// The drawing of one model.
#[derive(Debug, Default)]
pub struct Plan {
    /// Part index -> where its box sits on this diagram.
    pub boxes: BTreeMap<usize, Placed>,
    /// Port index of the model itself -> its centre on the border.
    pub own_ports: BTreeMap<usize, (f64, f64)>,
    /// Index in the connection list -> the points of its line.
    pub wires: BTreeMap<usize, Vec<(f64, f64)>>,
    /// left, bottom, right, top.
    pub extent: [f64; 4],
}

type Point = (f64, f64);

impl Plan {
    /// Where a connector of the model sits in its own frame, as a fraction of
    /// half the width and half the height from the centre (-1..1). The icon
    /// of a model is its diagram, so a box in a parent shows the connector
    /// there.
    pub fn fraction(&self, port: usize) -> (f64, f64) {
        let [l, b, r, t] = self.extent;
        let (x, y) = self.own_ports.get(&port).copied().unwrap_or(((l + r) / 2.0, (b + t) / 2.0));
        ((x - (l + r) / 2.0) / ((r - l) / 2.0), (y - (b + t) / 2.0) / ((t - b) / 2.0))
    }
}

/// The box a model is drawn as in a parent diagram: as wide as it is for
/// all of them, as high as the aspect of its own diagram says.
pub fn box_size(own: &Plan) -> (f64, f64) {
    let [l, b, r, t] = own.extent;
    let aspect = if r > l { (t - b) / (r - l) } else { 1.0 };
    (BOX_W, (BOX_W * aspect).clamp(MIN_BOX_H, MAX_BOX_H))
}

/// The boxes, the connectors on the border and the lines of a model.
///
/// `holder` is the part whose model is drawn (`None` for a system model),
/// `children` the parts it instantiates, `conns` its connections, and
/// `moves` the offsets (Modelica units, y up) the layout file gives to
/// children.
pub fn plan(parts: &[Part], sides: &Sides, plans: &[Plan], holder: Option<usize>, children: &[usize], conns: &[&Connection], moves: &BTreeMap<usize, Point>) -> Plan {
    let mut plan = Plan::default();
    place_boxes(&mut plan, parts, plans, children, conns, moves);
    let (min_x, min_y, max_x, max_y) = bounds(&plan);
    let left = min_x - BORDER;
    let right = max_x + BORDER;
    let mut bottom = min_y - 10.0;
    let mut top = max_y + 10.0;
    if let Some(holder) = holder {
        let (left_ports, right_ports) = sides.lists(holder);
        let needed = BORDER_PORT_PITCH * left_ports.len().max(right_ports.len()) as f64;
        if top - bottom < needed {
            let middle = (top + bottom) / 2.0;
            bottom = middle - needed / 2.0;
            top = middle + needed / 2.0;
        }
        for (list, x) in [(&left_ports, left + DIAGRAM_PORT + 1.0), (&right_ports, right - DIAGRAM_PORT - 1.0)] {
            for (i, &port) in list.iter().enumerate() {
                let y = top - (i as f64 + 0.5) * (top - bottom) / list.len() as f64;
                plan.own_ports.insert(port, (x, y));
            }
        }
    }
    let (lanes_top, lanes_bottom) = route(&mut plan, parts, sides, plans, holder, conns, (max_y, min_y));
    top = top.max(max_y + 14.0 + LANE_PITCH * lanes_top as f64 + 4.0);
    bottom = bottom.min(min_y - 14.0 - LANE_PITCH * lanes_bottom as f64 - 4.0);
    plan.extent = [left, bottom, right, top];
    plan
}

fn bounds(plan: &Plan) -> (f64, f64, f64, f64) {
    if plan.boxes.is_empty() {
        return (-40.0, -30.0, 40.0, 30.0);
    }
    let mut out = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for placed in plan.boxes.values() {
        let r = placed.rect();
        out = (out.0.min(r.left), out.1.min(r.bottom), out.2.max(r.right), out.3.max(r.top));
    }
    out
}

/// Which end of a connection is the source: a signal goes from its output;
/// the rest as written.
fn directed(parts: &[Part], connection: &Connection) -> [(usize, usize); 2] {
    let [a, b] = connection.ends;
    let orientation = |end: (usize, usize)| parts[end.0].ports[end.1].orientation;
    if orientation(a) == Orientation::In && orientation(b) == Orientation::Out {
        [b, a]
    } else {
        [a, b]
    }
}

/// Columns of children, left to right, by the longest path from a source.
fn columns(parts: &[Part], children: &[usize], conns: &[&Connection]) -> Vec<Vec<usize>> {
    let local = |part: usize| children.iter().position(|&c| c == part);
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for connection in conns {
        let [from, to] = directed(parts, connection);
        if let (Some(a), Some(b)) = (local(from.0), local(to.0)) {
            if a != b && !edges.contains(&(a, b)) {
                edges.push((a, b));
            }
        }
    }
    // Depth-first from the first child; an edge back to a part on the stack
    // closes a cycle and is not kept.
    let n = children.len();
    let mut state = vec![0u8; n];
    let mut order = Vec::new();
    let mut kept = Vec::new();
    for start in 0..n {
        if state[start] == 0 {
            visit(start, &edges, &mut state, &mut order, &mut kept);
        }
    }
    order.reverse();
    let mut layer = vec![0usize; n];
    for &v in &order {
        for &(a, b) in kept.iter().filter(|(a, _)| *a == v) {
            layer[b] = layer[b].max(layer[a] + 1);
        }
    }
    let depth = layer.iter().copied().max().map_or(0, |m| m + 1);
    let mut grouped: Vec<Vec<usize>> = vec![Vec::new(); depth];
    for (v, &l) in layer.iter().enumerate() {
        grouped[l].push(v);
    }
    // Order each column by the mean row of the neighbours already placed.
    let neighbours = |v: usize| -> Vec<usize> {
        edges.iter().filter_map(|&(a, b)| if a == v { Some(b) } else if b == v { Some(a) } else { None }).collect()
    };
    let mut rows: BTreeMap<usize, f64> = BTreeMap::new();
    let mut out: Vec<Vec<usize>> = Vec::new();
    for mut column in grouped {
        let key = |v: usize| -> f64 {
            let known: Vec<f64> = neighbours(v).iter().filter_map(|w| rows.get(w).copied()).collect();
            if known.is_empty() { f64::MAX } else { known.iter().sum::<f64>() / known.len() as f64 }
        };
        let mut keyed: Vec<(f64, usize)> = column.drain(..).map(|v| (key(v), v)).collect();
        keyed.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap_or(std::cmp::Ordering::Equal).then(x.1.cmp(&y.1)));
        for chunk in keyed.chunks(MAX_ROWS) {
            let count = chunk.len() as f64;
            for (row, &(_, v)) in chunk.iter().enumerate() {
                rows.insert(v, row as f64 / count);
            }
            out.push(chunk.iter().map(|&(_, v)| children[v]).collect());
        }
    }
    out
}

fn visit(v: usize, edges: &[(usize, usize)], state: &mut [u8], order: &mut Vec<usize>, kept: &mut Vec<(usize, usize)>) {
    state[v] = 1;
    for &(a, b) in edges.iter().filter(|(a, _)| *a == v) {
        match state[b] {
            0 => {
                kept.push((a, b));
                visit(b, edges, state, order, kept);
            }
            1 => {}
            _ => kept.push((a, b)),
        }
    }
    state[v] = 2;
    order.push(v);
}

fn place_boxes(plan: &mut Plan, parts: &[Part], plans: &[Plan], children: &[usize], conns: &[&Connection], moves: &BTreeMap<usize, Point>) {
    for (c, column) in columns(parts, children, conns).iter().enumerate() {
        let sizes: Vec<(f64, f64)> = column.iter().map(|&p| box_size(&plans[p])).collect();
        let total: f64 = sizes.iter().map(|s| s.1).sum::<f64>() + ROW_GAP * (column.len() as f64 - 1.0);
        let mut y = total / 2.0;
        for (&part, &(w, h)) in column.iter().zip(&sizes) {
            let (dx, dy) = moves.get(&part).copied().unwrap_or((0.0, 0.0));
            plan.boxes.insert(part, Placed { x: c as f64 * (BOX_W + COLUMN_GAP) + dx, y: y - h / 2.0 + dy, w, h });
            y -= h + ROW_GAP;
        }
    }
}

struct Endpoint {
    at: Point,
    /// +1 leaves to the right, -1 to the left.
    leaves: f64,
    stub: f64,
}

fn endpoint(plan: &Plan, sides: &Sides, plans: &[Plan], holder: Option<usize>, end: (usize, usize)) -> Option<Endpoint> {
    if Some(end.0) == holder {
        let at = *plan.own_ports.get(&end.1)?;
        let leaves = -sides.side(end.0, end.1).sign();
        return Some(Endpoint { at, leaves, stub: STUB_BORDER });
    }
    let placed = plan.boxes.get(&end.0)?;
    let (fx, fy) = plans[end.0].fraction(end.1);
    Some(Endpoint { at: (placed.x + fx * placed.w / 2.0, placed.y + fy * placed.h / 2.0), leaves: sides.side(end.0, end.1).sign(), stub: STUB_CHILD })
}

/// Whether a line crosses the inside of a box.
fn crosses(points: &[Point], boxes: &[Rect]) -> bool {
    points.windows(2).any(|w| {
        let (x1, x2) = (w[0].0.min(w[1].0), w[0].0.max(w[1].0));
        let (y1, y2) = (w[0].1.min(w[1].1), w[0].1.max(w[1].1));
        boxes.iter().any(|r| x1 < r.right + 1.5 && x2 > r.left - 1.5 && y1 < r.top + 1.5 && y2 > r.bottom - 1.5)
    })
}

fn squeeze(points: Vec<Point>) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::new();
    for p in points {
        if out.last().is_some_and(|q| (q.0 - p.0).abs() < 0.05 && (q.1 - p.1).abs() < 0.05) {
            continue;
        }
        // The middle of three points on one line is no bend.
        while out.len() >= 2 {
            let (a, b) = (out[out.len() - 2], out[out.len() - 1]);
            let flat = ((a.0 - b.0).abs() < 0.05 && (b.0 - p.0).abs() < 0.05) || ((a.1 - b.1).abs() < 0.05 && (b.1 - p.1).abs() < 0.05);
            if !flat {
                break;
            }
            out.pop();
        }
        out.push(p);
    }
    out
}

/// The lines of the connections; returns how many lanes were used above and
/// below the boxes.
fn route(plan: &mut Plan, parts: &[Part], sides: &Sides, plans: &[Plan], holder: Option<usize>, conns: &[&Connection], span: (f64, f64)) -> (usize, usize) {
    let all: Vec<(usize, Rect)> = plan.boxes.iter().map(|(&part, placed)| (part, placed.rect())).collect();
    let (mut lanes_top, mut lanes_bottom) = (0usize, 0usize);
    let mut channels: BTreeMap<(i64, i64), usize> = BTreeMap::new();
    for (k, connection) in conns.iter().enumerate() {
        let [from, to] = directed(parts, connection);
        let (Some(a), Some(b)) = (endpoint(plan, sides, plans, holder, from), endpoint(plan, sides, plans, holder, to)) else { continue };
        // The boxes the line starts and ends in are not in its way.
        let boxes: Vec<Rect> = all.iter().filter(|(part, _)| *part != from.0 && *part != to.0).map(|(_, r)| *r).collect();
        let reversed = a.leaves < 0.0 && b.leaves > 0.0;
        let (a, b) = if reversed { (b, a) } else { (a, b) };
        let mut points = None;
        if a.leaves > 0.0 && b.leaves < 0.0 && a.at.0 + a.stub < b.at.0 - b.stub {
            // Right to left: a step in the gap between the two.
            let (lo, hi) = (a.at.0 + a.stub, b.at.0 - b.stub);
            let channel = channels.entry(((lo * 10.0) as i64, (hi * 10.0) as i64)).or_insert(0);
            let shift = ((*channel + 1) / 2) as f64 * 3.0 * if *channel % 2 == 1 { 1.0 } else { -1.0 };
            *channel += 1;
            let mx = ((lo + hi) / 2.0 + shift).clamp(lo, hi);
            let candidate = squeeze(vec![a.at, (mx, a.at.1), (mx, b.at.1), b.at]);
            if !crosses(&candidate, &boxes) {
                points = Some(candidate);
            }
        } else if a.leaves == b.leaves {
            // Both leave the same way: a U round the nearer of the two.
            let x = if a.leaves > 0.0 { a.at.0.max(b.at.0) + a.stub.max(b.stub) } else { a.at.0.min(b.at.0) - a.stub.max(b.stub) };
            let candidate = squeeze(vec![a.at, (x, a.at.1), (x, b.at.1), b.at]);
            if !crosses(&candidate, &boxes) {
                points = Some(candidate);
            }
        }
        let points = points.unwrap_or_else(|| {
            let s1 = (a.at.0 + a.leaves * a.stub, a.at.1);
            let s2 = (b.at.0 + b.leaves * b.stub, b.at.1);
            let above = (lanes_top + lanes_bottom) % 2 == 0;
            let y = if above {
                lanes_top += 1;
                span.0 + 14.0 + LANE_PITCH * (lanes_top - 1) as f64
            } else {
                lanes_bottom += 1;
                span.1 - 14.0 - LANE_PITCH * (lanes_bottom - 1) as f64
            };
            squeeze(vec![a.at, s1, (s1.0, y), (s2.0, y), s2, b.at])
        });
        let points = if reversed { points.into_iter().rev().collect() } else { points };
        plan.wires.insert(k, points);
    }
    (lanes_top, lanes_bottom)
}

/// The offsets the layout file gives to boxes, in Modelica units: the
/// arrangement of the physical view (`pab`) opened on `open` (`None`: the
/// whole view). Sheet offsets are relative to the workbench's own layout,
/// so they move a box from where this one puts it, they do not place it.
pub fn offsets(layout: &LayoutFile, open: Option<&str>, parts: &[Part]) -> BTreeMap<usize, Point> {
    let mut out = BTreeMap::new();
    let Some(arrangement) = layout.arrangements.iter().find(|a| a.view == "pab" && a.open.as_deref() == open) else { return out };
    for (index, part) in parts.iter().enumerate() {
        if let Some(offset) = arrangement.places.get(&part.id) {
            out.insert(index, (offset.dx * SHEET_TO_MODELICA, -offset.dy * SHEET_TO_MODELICA));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_short_and_never_negative_zero() {
        assert_eq!(num(-0.04), "0");
        assert_eq!(num(12.0), "12");
        assert_eq!(num(-12.34), "-12.3");
        assert_eq!(num(0.25), "0.3");
    }
}
