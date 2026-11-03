//! The geometry of the state machine diagrams: where the connector reads the
//! positions of states and the ends of transitions (`CollectGraphicalDataForTransformation`,
//! `CoordinatesCalculator`, `ViewGraphicalInfomation`, `addGraphicalParameters`).
//!
//! The connector draws each diagram with GMF and Sirius and reads the figures'
//! bounds. Here the figures are laid out from the `.aird`'s notation: a node at
//! its stored position, with its stored size or, when GMF sizes it (`-1`), the
//! size Sirius 7.4 gives it on the connector's reference platform (Linux GTK,
//! DejaVu Sans 8 pt, `font`). The positions are relative to the diagram image
//! (its figures' bounds less a 10 px margin), as `PartPositionInfoGenerator`
//! gives them; the ends of a transition come from GMF's slidable anchors on the
//! figures' handle bounds (without the containers' 2 px drop shadow), as a clock
//! position around the state (`computeOClock`).
//!
//! What is exact: positions, the sizes of states holding regions (their label,
//! their regions' contents), the ends of straight transitions. The sizes of a
//! state's regions themselves and of the diagram are estimates; they are used
//! only for states with several regions and machines with several regions.

use super::font::text_width;
use super::model::Model;
use super::Run;
use super::super::pivot::{Kind as PivotKind, Value};
use std::collections::HashMap;

/// `IConstants`/`ViewGraphicalInfomation.DEFAULT_TRANSLATION`.
const TRANSLATION: f64 = 10.0;
/// `DiagramGenerator.DEFAULT_IMAGE_MARGIN_PIXELS`.
const IMAGE_MARGIN: i32 = 10;
/// The drop shadow of a container (`AlphaDropShadowBorder`): right and bottom.
const SHADOW: i32 = 2;
/// A state holding regions: its width past its label's, its height with an empty region,
/// where its regions' contents start, and the room after them.
const STATE_LABEL_MARGIN: i32 = 13;
const STATE_EMPTY_HEIGHT: i32 = 67;
const STATE_CLIENT: (i32, i32) = (6, 34);
const STATE_CONTENT_MARGIN: (i32, i32) = (12, 9);
/// The height of a label line.
const LABEL_HEIGHT: i32 = 13;
/// `AbstractCapellaSMToPivot`: a junction's size, the shift of twin regions.
const JUNCTION_SIZE: i32 = 16;
const TRANSLATE_DELTA: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Node,
    Edge,
    Diagram,
}

/// `ViewGraphicalInfomation`: an origin and an end (a node's size, an edge's clock positions).
#[derive(Debug, Clone)]
pub struct View {
    pub kind: Kind,
    pub origin: (f64, f64),
    pub end: (f64, f64),
    /// An edge whose source end is a clock position: it does not move.
    pub source_oclock: bool,
    /// The views of the diagram's elements, by Capella element, in order.
    pub children: Vec<(usize, View)>,
}

impl View {
    fn new(kind: Kind, origin: (f64, f64), end: (f64, f64)) -> View {
        View { kind, origin, end, source_oclock: false, children: Vec::new() }
    }

    fn origin_translate(&mut self, dx: f64, dy: f64) {
        if self.kind == Kind::Edge && self.source_oclock {
            return;
        }
        self.origin = (self.origin.0 + dx, self.origin.1 + dy);
        for (_, child) in &mut self.children {
            child.origin_translate(dx, dy);
        }
    }

    fn end_translate(&mut self, dx: f64, dy: f64) {
        if self.kind == Kind::Edge && self.source_oclock {
            return;
        }
        self.end = (self.end.0 + dx, self.end.1 + dy);
    }

    fn child(&mut self, e: usize) -> Option<&mut View> {
        self.children.iter_mut().find(|(k, _)| *k == e).map(|(_, v)| v)
    }
}

/// draw2d's `PrecisionPoint` to int.
fn int(v: f64) -> i32 {
    (v + 1e-9).floor() as i32
}

/// The first diagram whose target is this element (`DialectManager.getRepresentations`).
pub fn diagram_of(m: &Model, target: usize) -> Option<usize> {
    m.referencers(target, "target").into_iter().find(|&d| m.class(d) == "DSemanticDiagram")
}

/// The geometry of the diagrams of the selection, by the region each shows.
pub fn collect(m: &Model, selection: &[usize]) -> Result<HashMap<usize, View>, String> {
    let mut out = HashMap::new();
    for &d in selection {
        if m.class(d) != "DSemanticDiagram" {
            continue;
        }
        let Some(target) = m.reference(d, "target") else { continue };
        if out.contains_key(&target) {
            continue;
        }
        if let Some(view) = Layout::new(m, d).and_then(|mut l| l.view()) {
            out.insert(target, view);
        }
    }
    Ok(out)
}

// ---- Laying the notation out ------------------------------------------------------------

/// A figure: its notation node, its diagram element, its Capella element, its absolute bounds.
#[derive(Debug, Clone)]
struct Figure {
    element: Option<usize>,
    target: Option<usize>,
    bounds: (i32, i32, i32, i32),
    /// Whether it has the containers' drop shadow.
    shadow: bool,
    top: bool,
}

struct Layout<'m> {
    m: &'m Model,
    diagram: usize,
    resource: usize,
    figures: Vec<Figure>,
    by_node: HashMap<usize, usize>,
}

impl<'m> Layout<'m> {
    fn new(m: &'m Model, diagram: usize) -> Option<Layout<'m>> {
        let entry = m.owned(diagram, "ownedAnnotationEntries").into_iter().find(|&a| m.attr(a, "source") == Some("GMF_DIAGRAMS"))?;
        let notation = m.owned(entry, "data").into_iter().next()?;
        Some(Layout { m, diagram: notation, resource: m.elements[diagram].resource, figures: Vec::new(), by_node: HashMap::new() })
    }

    fn element_of(&self, node: usize) -> Option<usize> {
        self.m.attr(node, "element").and_then(|id| self.m.by_id_in(self.resource, id))
    }

    fn bounds_of(&self, node: usize) -> (i32, i32, i32, i32) {
        let constraint = self.m.owned(node, "layoutConstraint").into_iter().next();
        let get = |k: &str, default: i32| constraint.and_then(|c| self.m.attr(c, k)).and_then(|v| v.parse().ok()).unwrap_or(default);
        (get("x", 0), get("y", 0), get("width", -1), get("height", -1))
    }

    fn kind_of(&self, node: usize) -> String {
        self.m.attr(node, "type").unwrap_or("").to_string()
    }

    /// The notation nodes a node contains, through its compartments.
    fn inner_nodes(&self, node: usize) -> Vec<usize> {
        let mut out = Vec::new();
        for child in self.m.owned(node, "children") {
            if self.element_of(child).is_some() && !self.kind_of(child).starts_with('5') {
                out.push(child);
            } else if self.kind_of(child).starts_with('7') {
                out.extend(self.inner_nodes(child));
            }
        }
        out
    }

    fn label_name(&self, node: usize) -> String {
        self.element_of(node).and_then(|e| self.m.name(e)).unwrap_or_default()
    }

    /// The size of a figure: its stored size, else the size Sirius gives it.
    fn size(&self, node: usize) -> (i32, i32) {
        let (_, _, w, h) = self.bounds_of(node);
        if w > 0 && h > 0 {
            return (w, h);
        }
        let element = self.element_of(node);
        let container = element.is_some_and(|e| self.m.class(e) == "DNodeContainer");
        if !container {
            let style = element.and_then(|e| self.m.owned(e, "ownedStyle").into_iter().next());
            let dim = |k: &str| style.and_then(|s| self.m.attr(s, k)).and_then(|v| v.parse::<i32>().ok()).map(|v| v * 10).unwrap_or(10);
            return (if w > 0 { w } else { dim("width") }, if h > 0 { h } else { dim("height") });
        }
        // A state: its label, and what its regions hold.
        let mut right = 0;
        let mut bottom = 0;
        for region in self.inner_nodes(node) {
            for child in self.inner_nodes(region) {
                let (x, y, _, _) = self.bounds_of(child);
                let (cw, ch) = self.size(child);
                right = right.max(x + cw);
                bottom = bottom.max(y + ch);
                for label in self.m.owned(child, "children").into_iter().filter(|&l| self.kind_of(l).starts_with('5')) {
                    let Some(at) = self.m.owned(label, "layoutConstraint").into_iter().next() else { continue };
                    if self.m.class(at) != "Location" {
                        continue;
                    }
                    let lx: i32 = self.m.attr(at, "x").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let ly: i32 = self.m.attr(at, "y").and_then(|v| v.parse().ok()).unwrap_or(0);
                    right = right.max(x + lx + text_width(&self.label_name(child)));
                    bottom = bottom.max(y + ly + LABEL_HEIGHT);
                }
            }
        }
        let width = (text_width(&self.label_name(node)) + STATE_LABEL_MARGIN).max(if right > 0 { STATE_CLIENT.0 + right + STATE_CONTENT_MARGIN.0 } else { 0 });
        let height = STATE_EMPTY_HEIGHT.max(if bottom > 0 { STATE_CLIENT.1 + bottom + STATE_CONTENT_MARGIN.1 } else { 0 });
        (if w > 0 { w } else { width }, if h > 0 { h } else { height })
    }

    /// Lays out a node at an absolute origin, then what it holds.
    fn place(&mut self, node: usize, origin: (i32, i32), top: bool) {
        let (x, y, _, _) = self.bounds_of(node);
        let (w, h) = self.size(node);
        let (ax, ay) = (origin.0 + x, origin.1 + y);
        let element = self.element_of(node);
        let container = element.is_some_and(|e| self.m.class(e) == "DNodeContainer");
        let target = element.and_then(|e| self.m.reference(e, "target"));
        let region = container && target.is_some_and(|t| self.m.is(t, "Region"));
        self.by_node.insert(node, self.figures.len());
        self.figures.push(Figure { element, target, bounds: (ax, ay, w, h), shadow: container && !region, top });
        if !container {
            return;
        }
        if region {
            for child in self.inner_nodes(node) {
                self.place(child, (ax + STATE_CLIENT.0 - 1, ay + STATE_CLIENT.1 - 17), false);
            }
            return;
        }
        // A state's regions, one under the other (an estimate past the first).
        let mut offset = 0;
        for region in self.inner_nodes(node) {
            let rx = ax + 1;
            let ry = ay + 17 + offset;
            let rh = (h - 20 - offset).max(LABEL_HEIGHT);
            self.by_node.insert(region, self.figures.len());
            let re = self.element_of(region);
            self.figures.push(Figure { element: re, target: re.and_then(|e| self.m.reference(e, "target")), bounds: (rx, ry, w - 4, rh), shadow: false, top: false });
            for child in self.inner_nodes(region) {
                self.place(child, (ax + STATE_CLIENT.0, ay + STATE_CLIENT.1 + offset), false);
            }
            offset += rh;
        }
    }

    /// The figure's handle bounds: its bounds without the drop shadow.
    fn handle(&self, f: usize) -> (f64, f64, f64, f64) {
        let (x, y, w, h) = self.figures[f].bounds;
        let s = if self.figures[f].shadow { SHADOW } else { 0 };
        (x as f64, y as f64, (w - s) as f64, (h - s) as f64)
    }

    fn view(&mut self) -> Option<View> {
        let m = self.m;
        for node in m.owned(self.diagram, "children") {
            if self.element_of(node).is_some() {
                self.place(node, (0, 0), true);
            }
        }
        let edges: Vec<usize> = m.owned(self.diagram, "edges");
        let mut ends = Vec::new();
        for &edge in &edges {
            ends.push(self.edge_points(edge));
        }
        // The image: the bounds of the top figures and of the edges, less the margin.
        let mut min = (i32::MAX, i32::MAX);
        let mut max = (0, 0);
        for f in self.figures.iter().filter(|f| f.top) {
            let (x, y, w, h) = f.bounds;
            min = (min.0.min(x), min.1.min(y));
            max = (max.0.max(x + w), max.1.max(y + h));
        }
        for &(a, b) in ends.iter().flatten() {
            for p in [a, b] {
                min = (min.0.min(p.0), min.1.min(p.1));
            }
        }
        if min.0 == i32::MAX {
            return None;
        }
        let origin = (min.0 - IMAGE_MARGIN, min.1 - IMAGE_MARGIN);
        let mut views: Vec<(usize, View)> = Vec::new();
        for f in &self.figures {
            let Some(target) = f.target else { continue };
            if views.iter().any(|(k, _)| *k == target) {
                continue;
            }
            let (x, y, w, h) = f.bounds;
            views.push((target, View::new(Kind::Node, ((x - origin.0) as f64, (y - origin.1) as f64), (w as f64, h as f64))));
        }
        let mut edge_views = Vec::new();
        for (k, &edge) in edges.iter().enumerate() {
            let Some((source, target)) = ends[k] else { continue };
            let Some(element) = self.element_of(edge) else { continue };
            let Some(transition) = m.reference(element, "target") else { continue };
            let rect = |node: Option<usize>| {
                let target = node.and_then(|n| self.by_node.get(&n)).and_then(|&f| self.figures[f].target)?;
                views.iter().find(|(e, _)| *e == target).map(|(_, v)| (int(v.origin.0), int(v.origin.1), int(v.end.0), int(v.end.1)))
            };
            let source_node = m.attr(edge, "source").and_then(|id| m.by_id_in(self.resource, id));
            let target_node = m.attr(edge, "target").and_then(|id| m.by_id_in(self.resource, id));
            let src = (source.0 - origin.0, source.1 - origin.1);
            let dst = (target.0 - origin.0, target.1 - origin.1);
            let initial = m.class(transition) == "StateTransition" && m.reference(transition, "source").is_some_and(|s| m.is(s, "InitialPseudoState"));
            let origin_point = if initial { (src.0 as f64, src.1 as f64) } else { (rect(source_node).map(|r| oclock(r, src)).unwrap_or(-1.0), -1.0) };
            let end_point = (rect(target_node).map(|r| oclock(r, dst)).unwrap_or(-1.0), -1.0);
            let mut view = View::new(Kind::Edge, origin_point, end_point);
            view.source_oclock = !initial;
            edge_views.push((transition, view));
        }
        for (k, v) in edge_views {
            if !views.iter().any(|(e, _)| *e == k) {
                views.push((k, v));
            }
        }
        let mut diagram = View::new(Kind::Diagram, (0.0, 0.0), ((max.0 - origin.0 + IMAGE_MARGIN) as f64, (max.1 - origin.1 + IMAGE_MARGIN) as f64));
        diagram.children = views;
        diagram.origin_translate(2.0 * TRANSLATION, 2.0 * TRANSLATION);
        diagram.end_translate(3.0 * TRANSLATION, 3.0 * TRANSLATION);
        Some(diagram)
    }

    /// The two ends of an edge, as the connection's first and last points (GMF's oblique router
    /// over the stored bendpoints, its slidable anchors on the handle bounds).
    fn edge_points(&self, edge: usize) -> Option<((i32, i32), (i32, i32))> {
        let m = self.m;
        let node = |k: &str| m.attr(edge, k).and_then(|id| m.by_id_in(self.resource, id)).and_then(|n| self.by_node.get(&n).copied());
        let (s, t) = (node("source")?, node("target")?);
        let anchor = |k: &str| m.owned(edge, k).into_iter().next().and_then(|a| m.attr(a, "id")).and_then(terminal);
        let (sb, tb) = (self.handle(s), self.handle(t));
        let sref = reference(sb, anchor("sourceAnchor"));
        let tref = reference(tb, anchor("targetAnchor"));
        let bends = m.owned(edge, "bendpoints").into_iter().next().and_then(|b| m.attr(b, "points")).map(relative_bendpoints).unwrap_or_default();
        let n = bends.len();
        let inner: Vec<(f64, f64)> = if n > 2 {
            (1..n - 1)
                .map(|i| {
                    let w = i as f64 / (n - 1) as f64;
                    let (a, b, c, d) = bends[i];
                    ((1.0 - w) * (sref.0 + a) + w * (tref.0 + c), (1.0 - w) * (sref.1 + b) + w * (tref.1 + d))
                })
                .map(|(x, y)| (int(x) as f64, int(y) as f64))
                .collect()
        } else {
            Vec::new()
        };
        let (source, target) = if inner.is_empty() {
            let source = location(sb, sref, tref);
            (source, location(tb, tref, source))
        } else {
            (location(sb, sref, inner[0]), location(tb, tref, *inner.last().expect("a bendpoint")))
        };
        Some(((int(source.0), int(source.1)), (int(target.0), int(target.1))))
    }
}

/// `BaseSlidableAnchor.parseTerminalString`: `(x,y)`, relative to the box.
fn terminal(text: &str) -> Option<(f64, f64)> {
    let inner = text.trim().strip_prefix('(')?.strip_suffix(')')?;
    let (x, y) = inner.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

/// GMF's `RelativeBendpoints`: `[sx, sy, tx, ty]$…`, offsets from the two ends.
fn relative_bendpoints(text: &str) -> Vec<(f64, f64, f64, f64)> {
    text.split('$')
        .filter_map(|p| {
            let v: Vec<f64> = p.trim().trim_start_matches('[').trim_end_matches(']').split(',').filter_map(|n| n.trim().parse().ok()).collect();
            (v.len() == 4).then(|| (v[0], v[1], v[2], v[3]))
        })
        .collect()
}

/// An anchor's reference point: its relative position in the box, or the box's center.
fn reference(b: (f64, f64, f64, f64), relative: Option<(f64, f64)>) -> (f64, f64) {
    match relative {
        Some((rx, ry)) => (rx * b.2 + b.0, ry * b.3 + b.1),
        None => (b.0 + b.2 / 2.0, b.1 + b.3 / 2.0),
    }
}

/// `BaseSlidableAnchor.getLocation`: where the line from the anchor's reference (straightened
/// within 10 px) to the foreign one crosses the box, the crossing nearest the foreign one.
fn location(b: (f64, f64, f64, f64), own: (f64, f64), foreign: (f64, f64)) -> (f64, f64) {
    let mut own = own;
    if (foreign.0 - own.0).abs() < 10.0 {
        own.0 = foreign.0;
    } else if (foreign.1 - own.1).abs() < 10.0 {
        own.1 = foreign.1;
    }
    let center = (b.0 + b.2 / 2.0, b.1 + b.3 / 2.0);
    crossing(b, own, foreign).or_else(|| crossing(b, center, foreign)).unwrap_or(center)
}

fn crossing(b: (f64, f64, f64, f64), a: (f64, f64), f: (f64, f64)) -> Option<(f64, f64)> {
    let corners = [(b.0, b.1), (b.0 + b.2, b.1), (b.0 + b.2, b.1 + b.3), (b.0, b.1 + b.3), (b.0, b.1)];
    let mut found: Vec<(f64, f64)> = Vec::new();
    for w in corners.windows(2) {
        let (p, q) = (w[0], w[1]);
        let den = (a.0 - f.0) * (p.1 - q.1) - (a.1 - f.1) * (p.0 - q.0);
        if den.abs() < 1e-12 {
            continue;
        }
        let t = ((a.0 - p.0) * (p.1 - q.1) - (a.1 - p.1) * (p.0 - q.0)) / den;
        let point = (a.0 + t * (f.0 - a.0), a.1 + t * (f.1 - a.1));
        let within = |v: f64, l: f64, r: f64| v >= l.min(r) - 1e-9 && v <= l.max(r) + 1e-9;
        if within(point.0, p.0, q.0) && within(point.1, p.1, q.1) {
            found.push(point);
        }
    }
    found.into_iter().min_by(|p, q| {
        let d = |r: &(f64, f64)| (r.0 - f.0).powi(2) + (r.1 - f.1).powi(2);
        d(p).total_cmp(&d(q))
    })
}

/// `CoordinatesCalculator.computeOClock`: the clock position of a point around a box's center.
fn oclock(r: (i32, i32, i32, i32), p: (i32, i32)) -> f64 {
    let center = (r.0 + r.2 / 2, r.1 + r.3 / 2);
    let (dx, dy) = (p.0 - center.0, p.1 - center.1);
    (std::f64::consts::PI + f64::atan2(-dx as f64, dy as f64)) / std::f64::consts::PI * 6.0
}

// ---- Putting the geometry on the pivot ----------------------------------------------------

/// The region of the machine an element is drawn in (`identifyRootParentRegion`).
fn root_region(m: &Model, e: usize) -> Option<usize> {
    let mut at = Some(e);
    while let Some(x) = at {
        if m.is(x, "Region") && m.container(x).is_some_and(|c| m.is(c, "StateMachine")) {
            return Some(x);
        }
        at = m.container(x);
    }
    None
}

/// `DiagramSemanticHelper.getTwins`: the regions beside a region, with it (the selected ones
/// when asked), if its owner has several.
fn twins(run: &Run, e: usize, selected_only: bool) -> Vec<usize> {
    let m = run.m;
    if !m.is(e, "Region") {
        return Vec::new();
    }
    let Some(parent) = m.container(e) else { return Vec::new() };
    let regions = m.owned(parent, "ownedRegions");
    if regions.len() < 2 {
        return Vec::new();
    }
    regions.into_iter().filter(|r| !selected_only || run.selection.contains(r)).collect()
}

/// `shouldTransformRegion`: another region of the region's owner is selected.
fn other_region_selected(run: &Run, region: usize) -> bool {
    let m = run.m;
    let Some(parent) = m.container(region) else { return false };
    m.owned(parent, "ownedRegions").into_iter().any(|r| r != region && run.selection.contains(&r))
}

fn multiple(values: &[String]) -> Value {
    Value::Multiple(values.iter().map(|v| Value::Primitive(Some(v.clone()))).collect())
}

/// `addGraphicalParameters`: the bounds of the element's view, or its ends, on the pivot element.
pub fn graphical(run: &mut Run, pivot: usize, capella: usize, edge_oclock: bool) {
    let m = run.m;
    let Some(root) = root_region(m, capella) else { return };
    if !run.geometry.contains_key(&root) {
        return;
    }
    let y = if root == capella {
        let mut y = -1.0;
        for twin in twins(run, capella, false) {
            if twin == capella {
                break;
            }
            if let Some(v) = run.geometry.get(&twin) {
                y += v.end.1.floor() + 11.0;
            }
        }
        y
    } else {
        -1.0
    };
    let twin_count = twins(run, capella, true).len();
    let in_state = m.is(capella, "Region") && m.container(capella).is_some_and(|c| m.is(c, "State"));
    let parallel = m.is(capella, "State") && m.owned(capella, "ownedRegions").into_iter().any(|r| other_region_selected(run, r));
    let junction = run.p[pivot].kind == PivotKind::Junction;
    let diagram = run.geometry.get_mut(&root).expect("the diagram's view");
    let view = if root == capella { Some(diagram) } else { diagram.child(capella) };
    let Some(view) = view else { return };
    if root == capella && y != -1.0 {
        view.origin_translate(0.0, y);
    }
    if in_state && twin_count > 1 {
        view.end_translate(0.0, -TRANSLATE_DELTA);
    }
    if parallel {
        view.origin_translate(-TRANSLATE_DELTA, -TRANSLATE_DELTA);
        view.end_translate(TRANSLATE_DELTA, TRANSLATE_DELTA);
    }
    let (x1, y1, x2, y2) = (int(view.origin.0), int(view.origin.1), int(view.end.0), int(view.end.1));
    let (kind, source_clock, destination_clock) = (view.kind, view.origin.0, view.end.0);
    let o = &mut run.p[pivot];
    match kind {
        Kind::Diagram | Kind::Node => {
            let (w, h) = if kind == Kind::Node && junction { (JUNCTION_SIZE, JUNCTION_SIZE) } else { (x2, y2) };
            o.set_param("bounds", multiple(&[x1.to_string(), y1.to_string(), w.to_string(), h.to_string()]));
        }
        Kind::Edge => {
            let source = if edge_oclock { vec![super::super::java::double(source_clock)] } else { vec![x1.to_string(), y1.to_string()] };
            o.set_param("position.source", multiple(&source));
            o.set_param("position.destination", multiple(&[super::super::java::double(destination_clock)]));
        }
    }
}
