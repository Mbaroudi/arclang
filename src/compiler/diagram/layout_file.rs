//! The layout file — how a reader arranged the diagrams of a model.
//!
//! Layout is automatic, but a reader may fold containers, open one as a
//! diagram of its own and move boxes by hand. That arrangement is kept in
//! `<model>.layout.json`, next to the model, so it is shared and versioned
//! with it. The file never changes what the model means: it names elements
//! the diagrams draw, and whatever it names that is not drawn is reported
//! and left out, never guessed.
//!
//! ```json
//! {
//!   "arclang_layout": "1",
//!   "views": { "lab": { "folded": ["LC-2"], "open": null } },
//!   "arrangements": [
//!     { "view": "lab", "open": null, "folded": ["LC-2"],
//!       "places": { "LC-1": { "dx": 40.0, "dy": -12.5, "dw": 60.0 } } }
//!   ]
//! }
//! ```
//!
//! `views` says what each view shows when opened. `arrangements` holds the
//! manual placement, one per layout: moving a box in the whole view and in
//! an opened container are two arrangements. Offsets are in sheet units,
//! relative to where the automatic layout puts the box; `dw` and `dh`, when
//! written, are how much wider and taller the reader made it. `edges`,
//! when written, gives an exchange the waypoints the reader bent it
//! through, in sheet units: `{ "ce:Flow": { "points": [{ "x": 120, "y": 40 }] } }`,
//! and, when set, how it is drawn (`"style": "straight"` or `"curved"`;
//! right angles otherwise) and how far its name was dragged off it
//! (`"label": { "dx": 12, "dy": -8 }`). `ports`, when written, moves a
//! port along the border of its box, from where the automatic layout puts
//! it: `{ "LC-1::out": { "dx": 0, "dy": 24 } }`. `order`, when written,
//! stacks boxes among the boxes beside them: above 0 in front, below 0
//! behind: `{ "LC-1": 1 }`. Each is written only when used, so a file
//! without them reads in earlier builds.
//!
//! Three sections hold draw.io's Arrange state, also only when used:
//! `layers` (layers made of the elements' tags, the first tag of an
//! element being its layer, `""` the elements without one: their `order`
//! top first, those `hidden` and those `locked`), `groups` (per view,
//! frames around sibling boxes: `{ "id": "g1", "members": ["LC-1",
//! "group:g2"], "label": "Sensing", "collapsed": true }`) and `locked`
//! (elements locked on the canvas, by drawing id).

use super::{DiagramSet, Node};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Version of the layout file format this build reads and writes.
pub const LAYOUT_VERSION: &str = "1";
/// Sheet units beyond which an offset is not a placement.
const FARTHEST: f64 = 100_000.0;
/// The most arrangements a file may hold (views times opened containers).
pub const MAX_ARRANGEMENTS: usize = 2_000;
/// The most entries one map of a layout may hold (places, edges, ports, order, folded).
pub const MAX_ENTRIES: usize = 50_000;
/// The most points one bent exchange may pass through.
pub const MAX_BENDS: usize = 1_000;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutFile {
    pub arclang_layout: String,
    /// Per diagram id (`lab`, `msm:OperatingModes`): what it shows.
    #[serde(default)]
    pub views: BTreeMap<String, ViewLayout>,
    #[serde(default)]
    pub arrangements: Vec<Arrangement>,
    /// Layers made of tags: order (top first), hidden, locked.
    #[serde(default, skip_serializing_if = "Layers::is_empty")]
    pub layers: Layers,
    /// Per diagram id: groups of sibling boxes, framed and moved together.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub groups: BTreeMap<String, Vec<Group>>,
    /// Elements locked on the canvas, by drawing id.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locked: Vec<String>,
}

/// The layers of the drawing: each a tag (`""`: the elements without one).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layers {
    /// Drawn top first; empty, no layer is drawn above another.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locked: Vec<String>,
}

impl Layers {
    fn is_empty(&self) -> bool {
        self.order.is_empty() && self.hidden.is_empty() && self.locked.is_empty()
    }
}

/// A frame around sibling boxes of one view; a member is a drawn box or
/// another group (`group:<id>`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: String,
    pub members: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub collapsed: bool,
}

/// The most layers, groups of a view and members of a group a file may hold.
pub const MAX_LAYERS: usize = 1_000;
pub const MAX_GROUPS: usize = 2_000;
pub const MAX_MEMBERS: usize = 5_000;
/// The longest layer name, group id or group label.
pub const MAX_NAME: usize = 200;
/// How a group is named among the members of another.
pub const GROUP_PREFIX: &str = "group:";

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewLayout {
    /// Containers shown as a single box.
    #[serde(default)]
    pub folded: Vec<String>,
    /// The container opened as a diagram of its own, if any.
    #[serde(default)]
    pub open: Option<String>,
}

/// Manual placement of one layout of a view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arrangement {
    pub view: String,
    #[serde(default)]
    pub open: Option<String>,
    #[serde(default)]
    pub folded: Vec<String>,
    #[serde(default)]
    pub places: BTreeMap<String, Offset>,
    /// Exchanges bent, styled or with their name moved by hand, by edge id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub edges: BTreeMap<String, EdgeRoute>,
    /// Ports moved along the border of their box, by port id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ports: BTreeMap<String, Shift>,
    /// Boxes brought to the front (above 0) or sent to the back (below 0).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub order: BTreeMap<String, i64>,
}

/// How one exchange was arranged: the waypoints it passes through, from
/// its source to its target, its line style and the offset of its name.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EdgeRoute {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<Point>,
    /// `straight` or `curved`; right angles when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// How far the name of the exchange was dragged from where it sits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<Shift>,
}

impl EdgeRoute {
    fn is_empty(&self) -> bool {
        self.points.is_empty() && self.style.is_none() && self.label.is_none()
    }
}

/// Line styles an exchange may be drawn in besides right angles.
pub const EDGE_STYLES: [&str; 2] = ["straight", "curved"];
/// The furthest a box may be stacked in front of or behind the others.
const DEEPEST: i64 = 100_000;

/// A move in sheet units.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shift {
    pub dx: f64,
    pub dy: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offset {
    pub dx: f64,
    pub dy: f64,
    /// How much wider the reader made the box than the automatic layout did.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub dw: f64,
    /// How much taller the reader made the box.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub dh: f64,
}

fn is_zero(value: &f64) -> bool {
    *value == 0.0
}

/// Where the layout of a model is kept: `braking.arc` -> `braking.layout.json`.
pub fn sidecar_path(model: &Path) -> PathBuf {
    model.with_extension("layout.json")
}

impl LayoutFile {
    /// Read a layout file. Anything that is not one is refused with the
    /// reason: a layout is never half-read.
    pub fn parse(text: &str) -> Result<Self, String> {
        let layout: LayoutFile = serde_json::from_str(text)
            .map_err(|error| format!("layout file is not valid JSON for a layout: {error}"))?;
        if layout.arclang_layout != LAYOUT_VERSION {
            return Err(format!(
                "layout file version '{}' is not supported (this build reads version '{LAYOUT_VERSION}')",
                layout.arclang_layout
            ));
        }
        if layout.arrangements.len() > MAX_ARRANGEMENTS || layout.views.len() > MAX_ENTRIES {
            return Err(format!(
                "layout file: {} arrangements and {} views, more than a layout holds ({MAX_ARRANGEMENTS} and {MAX_ENTRIES})",
                layout.arrangements.len(),
                layout.views.len()
            ));
        }
        if let Some((view, _)) = layout.views.iter().find(|(_, shown)| shown.folded.len() > MAX_ENTRIES) {
            return Err(format!("layout file: view '{view}' folds more than {MAX_ENTRIES} containers"));
        }
        layout.check_arrange_sections()?;
        for arrangement in &layout.arrangements {
            let sizes = [
                ("placed boxes", arrangement.places.len()),
                ("arranged exchanges", arrangement.edges.len()),
                ("moved ports", arrangement.ports.len()),
                ("stacked boxes", arrangement.order.len()),
                ("folded containers", arrangement.folded.len()),
            ];
            if let Some((what, size)) = sizes.iter().find(|(_, size)| *size > MAX_ENTRIES) {
                return Err(format!(
                    "layout file: an arrangement of '{}' has {size} {what}, more than {MAX_ENTRIES}",
                    arrangement.view
                ));
            }
            if let Some((edge, route)) = arrangement.edges.iter().find(|(_, route)| route.points.len() > MAX_BENDS) {
                return Err(format!(
                    "layout file: '{edge}' bends through {} points, more than {MAX_BENDS}",
                    route.points.len()
                ));
            }
            for (node, offset) in &arrangement.places {
                let sound = |value: f64| value.is_finite() && value.abs() <= FARTHEST;
                if !sound(offset.dw) || !sound(offset.dh) {
                    return Err(format!(
                        "layout file: '{node}' is resized beyond what a sheet can hold ({}, {})",
                        offset.dw, offset.dh
                    ));
                }
                if !sound(offset.dx) || !sound(offset.dy) {
                    return Err(format!(
                        "layout file: '{node}' is placed farther than a sheet can be ({}, {})",
                        offset.dx, offset.dy
                    ));
                }
            }
            let sound = |value: f64| value.is_finite() && value.abs() <= FARTHEST;
            for (edge, route) in &arrangement.edges {
                if let Some(point) = route.points.iter().find(|point| !sound(point.x) || !sound(point.y)) {
                    return Err(format!(
                        "layout file: '{edge}' bends farther than a sheet can be ({}, {})",
                        point.x, point.y
                    ));
                }
                if let Some(label) = route.label.filter(|label| !sound(label.dx) || !sound(label.dy)) {
                    return Err(format!(
                        "layout file: the name of '{edge}' is moved farther than a sheet can be ({}, {})",
                        label.dx, label.dy
                    ));
                }
            }
            for (port, shift) in &arrangement.ports {
                if !sound(shift.dx) || !sound(shift.dy) {
                    return Err(format!(
                        "layout file: port '{port}' is moved farther than a sheet can be ({}, {})",
                        shift.dx, shift.dy
                    ));
                }
            }
            for (node, z) in &arrangement.order {
                if z.abs() > DEEPEST {
                    return Err(format!("layout file: '{node}' is stacked deeper than {DEEPEST} ({z})"));
                }
            }
        }
        Ok(layout.normalized())
    }

    /// Bounds of the layers, groups and locks: a file past them is refused.
    fn check_arrange_sections(&self) -> Result<(), String> {
        let layers = &self.layers;
        for (what, list) in [("ordered", &layers.order), ("hidden", &layers.hidden), ("locked", &layers.locked)] {
            if list.len() > MAX_LAYERS {
                return Err(format!("layout file: {} {what} layers, more than {MAX_LAYERS}", list.len()));
            }
            if let Some(name) = list.iter().find(|name| name.chars().count() > MAX_NAME) {
                return Err(format!("layout file: the layer name '{}…' is longer than {MAX_NAME} characters", name.chars().take(20).collect::<String>()));
            }
        }
        if self.groups.len() > MAX_ENTRIES || self.locked.len() > MAX_ENTRIES {
            return Err(format!("layout file: groups in {} views and {} locked elements, more than {MAX_ENTRIES}", self.groups.len(), self.locked.len()));
        }
        for (view, groups) in &self.groups {
            if groups.len() > MAX_GROUPS {
                return Err(format!("layout file: view '{view}' has {} groups, more than {MAX_GROUPS}", groups.len()));
            }
            for group in groups {
                if group.members.len() > MAX_MEMBERS {
                    return Err(format!("layout file: group '{}' of '{view}' has {} members, more than {MAX_MEMBERS}", group.id, group.members.len()));
                }
                let long = |text: &str| text.chars().count() > MAX_NAME;
                if group.id.is_empty() || long(&group.id) || group.label.as_deref().is_some_and(long) {
                    return Err(format!("layout file: a group of '{view}' has an empty or too long id or label (at most {MAX_NAME} characters)"));
                }
            }
        }
        Ok(())
    }

    /// The file as text: sorted, so the same arrangement is always the same
    /// bytes.
    pub fn to_json(&self) -> String {
        let text = serde_json::to_string_pretty(&self.clone().normalized())
            .expect("a layout is plain data");
        format!("{text}\n")
    }

    /// Lists sorted and without repeats; arrangements in a fixed order.
    fn normalized(mut self) -> Self {
        let sorted = |ids: &mut Vec<String>| {
            *ids = ids.drain(..).collect::<BTreeSet<_>>().into_iter().collect();
        };
        for view in self.views.values_mut() {
            sorted(&mut view.folded);
        }
        for arrangement in &mut self.arrangements {
            sorted(&mut arrangement.folded);
        }
        sorted(&mut self.layers.hidden);
        sorted(&mut self.layers.locked);
        sorted(&mut self.locked);
        // The order is the reader's: repeats go, the first place stays.
        let mut seen = BTreeSet::new();
        self.layers.order.retain(|name| seen.insert(name.clone()));
        self.arrangements.sort_by(|a, b| {
            (&a.view, &a.open, &a.folded).cmp(&(&b.view, &b.open, &b.folded))
        });
        self
    }
}

/// What a view draws, as a layout can refer to it.
struct Drawn<'a> {
    nodes: BTreeSet<&'a str>,
    containers: BTreeSet<&'a str>,
    edges: BTreeSet<&'a str>,
    ports: BTreeSet<&'a str>,
}

impl<'a> Drawn<'a> {
    fn of(nodes: &'a [Node], edges: &'a [super::Edge]) -> Self {
        let mut drawn = Drawn {
            nodes: BTreeSet::new(),
            containers: BTreeSet::new(),
            edges: edges.iter().map(|edge| edge.id.as_str()).collect(),
            ports: BTreeSet::new(),
        };
        let mut stack: Vec<&Node> = nodes.iter().collect();
        while let Some(node) = stack.pop() {
            drawn.nodes.insert(&node.id);
            drawn.ports.extend(node.ports.iter().map(|port| port.id.as_str()));
            if !node.children.is_empty() {
                drawn.containers.insert(&node.id);
            }
            stack.extend(node.children.iter());
        }
        drawn
    }
}

impl DiagramSet {
    /// Attach a layout to these diagrams. What it names that the diagrams
    /// do not draw is left out, returned as warnings and added to the
    /// diagnostics, where the viewer shows them: `[lab] ...` under that
    /// view, `[layout] ...` under every view. Saving the layout from the
    /// viewer then writes it without them, knowingly.
    pub fn apply_layout(&mut self, layout: LayoutFile) -> Vec<String> {
        let drawn: BTreeMap<&str, Drawn<'_>> = self
            .diagrams
            .iter()
            .map(|diagram| (diagram.id.as_str(), Drawn::of(&diagram.nodes, &diagram.edges)))
            .collect();
        let mut warnings: Vec<String> = Vec::new();
        let mut kept = LayoutFile {
            arclang_layout: layout.arclang_layout,
            // Layers are tags, which an element may carry without being drawn: kept as written.
            layers: layout.layers,
            ..LayoutFile::default()
        };
        kept.groups = keep_groups(layout.groups, &drawn, &mut warnings);
        kept.locked = layout
            .locked
            .into_iter()
            .filter(|id| {
                let is_drawn = drawn.values().any(|view| view.nodes.contains(id.as_str()) || view.edges.contains(id.as_str()) || view.ports.contains(id.as_str()));
                if !is_drawn {
                    warnings.push(format!("[layout] layout file: locked '{id}' is not drawn by this model"));
                }
                is_drawn
            })
            .collect();

        let keep_containers = |view: &str, what: &str, ids: Vec<String>, warnings: &mut Vec<String>| {
            ids.into_iter()
                .filter(|id| {
                    let known = drawn[view].containers.contains(id.as_str());
                    if !known {
                        warnings.push(format!(
                            "[{view}] layout file: {what} '{id}' is not a container of this view"
                        ));
                    }
                    known
                })
                .collect::<Vec<_>>()
        };

        for (view, shown) in layout.views {
            if !drawn.contains_key(view.as_str()) {
                warnings.push(format!("[layout] view '{view}' is not drawn by this model"));
                continue;
            }
            let folded = keep_containers(&view, "folded", shown.folded, &mut warnings);
            let open = shown
                .open
                .and_then(|id| keep_containers(&view, "open", vec![id], &mut warnings).pop());
            kept.views.insert(view, ViewLayout { folded, open });
        }
        for arrangement in layout.arrangements {
            let view = arrangement.view;
            let Some(known) = drawn.get(view.as_str()) else {
                warnings.push(format!("[layout] view '{view}' is not drawn by this model"));
                continue;
            };
            // An arrangement belongs to one layout: if the container it was
            // made in is gone, it has nothing to apply to.
            if let Some(open) = arrangement.open.as_deref() {
                if !known.containers.contains(open) {
                    warnings.push(format!(
                        "[{view}] layout file: an arrangement is for the open container '{open}', \
                         which is not a container of this view"
                    ));
                    continue;
                }
            }
            // The open container is not folded in its own diagram.
            let folded: Vec<String> = keep_containers(&view, "folded", arrangement.folded, &mut warnings)
                .into_iter()
                .filter(|id| Some(id.as_str()) != arrangement.open.as_deref())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            // Two arrangements may turn out to be for one layout once what
            // is stale is left out: the reader must know one is not applied.
            let same_layout = |other: &Arrangement| {
                other.view == view && other.open == arrangement.open && other.folded == folded
            };
            if kept.arrangements.iter().any(same_layout) {
                warnings.push(format!(
                    "[{view}] layout file: two arrangements are for the same layout — only the first is kept"
                ));
                continue;
            }
            let places: BTreeMap<String, Offset> = arrangement
                .places
                .into_iter()
                .filter(|(node, _)| {
                    let is_drawn = known.nodes.contains(node.as_str());
                    if !is_drawn {
                        warnings.push(format!(
                            "[{view}] layout file: placed '{node}' is not drawn in this view"
                        ));
                    }
                    is_drawn
                })
                .collect();
            let edges: BTreeMap<String, EdgeRoute> = arrangement
                .edges
                .into_iter()
                .filter_map(|(edge, mut route)| {
                    if !known.edges.contains(edge.as_str()) {
                        warnings.push(format!(
                            "[{view}] layout file: bent '{edge}' is not drawn in this view"
                        ));
                        return None;
                    }
                    if let Some(style) = route.style.as_deref().filter(|style| !EDGE_STYLES.contains(style)) {
                        warnings.push(format!(
                            "[{view}] layout file: '{edge}' has the line style '{style}', which is not one of {}",
                            EDGE_STYLES.join(", ")
                        ));
                        route.style = None;
                    }
                    (!route.is_empty()).then_some((edge, route))
                })
                .collect();
            let ports: BTreeMap<String, Shift> = arrangement
                .ports
                .into_iter()
                .filter(|(port, _)| {
                    let is_drawn = known.ports.contains(port.as_str());
                    if !is_drawn {
                        warnings.push(format!(
                            "[{view}] layout file: moved port '{port}' is not drawn in this view"
                        ));
                    }
                    is_drawn
                })
                .collect();
            let order: BTreeMap<String, i64> = arrangement
                .order
                .into_iter()
                .filter(|(node, z)| {
                    let is_drawn = known.nodes.contains(node.as_str());
                    if !is_drawn {
                        warnings.push(format!(
                            "[{view}] layout file: stacked '{node}' is not drawn in this view"
                        ));
                    }
                    is_drawn && *z != 0
                })
                .collect();
            if !places.is_empty() || !edges.is_empty() || !ports.is_empty() || !order.is_empty() {
                kept.arrangements.push(Arrangement {
                    view,
                    open: arrangement.open,
                    folded,
                    places,
                    edges,
                    ports,
                    order,
                });
            }
        }
        // Each thing left out is said once, in the order it was met.
        let mut said = BTreeSet::new();
        warnings.retain(|warning| said.insert(warning.clone()));
        self.diagnostics.extend(warnings.iter().cloned());
        self.layout = Some(kept.normalized());
        warnings
    }
}

/// The groups that still frame what a view draws: members not drawn, groups
/// of no drawn view, repeated ids and groups held in a circle are left out
/// and said; a group left with no member goes.
fn keep_groups(groups: BTreeMap<String, Vec<Group>>, drawn: &BTreeMap<&str, Drawn<'_>>, warnings: &mut Vec<String>) -> BTreeMap<String, Vec<Group>> {
    let mut out = BTreeMap::new();
    for (view, list) in groups {
        let Some(known) = drawn.get(view.as_str()) else {
            warnings.push(format!("[layout] layout file: groups of view '{view}', which this model does not draw"));
            continue;
        };
        let mut seen = BTreeSet::new();
        let list: Vec<Group> = list
            .into_iter()
            .filter(|group| {
                let fresh = seen.insert(group.id.clone());
                if !fresh {
                    warnings.push(format!("[{view}] layout file: two groups are named '{}' — only the first is kept", group.id));
                }
                fresh
            })
            .collect();
        let ids: BTreeSet<String> = list.iter().map(|group| group.id.clone()).collect();
        // A group inside itself, directly or through others, frames nothing.
        let inner = |group: &Group| -> Vec<String> { group.members.iter().filter_map(|m| m.strip_prefix(GROUP_PREFIX)).map(str::to_string).collect() };
        let by_id: BTreeMap<String, Vec<String>> = list.iter().map(|group| (group.id.clone(), inner(group))).collect();
        let circular = |start: &str| {
            let mut stack: Vec<String> = by_id.get(start).cloned().unwrap_or_default();
            let mut met = BTreeSet::new();
            while let Some(at) = stack.pop() {
                if at == start {
                    return true;
                }
                if met.insert(at.clone()) {
                    stack.extend(by_id.get(&at).cloned().unwrap_or_default());
                }
            }
            false
        };
        let mut kept = Vec::new();
        for mut group in list {
            if circular(&group.id) {
                warnings.push(format!("[{view}] layout file: group '{}' is inside itself", group.id));
                continue;
            }
            group.members.retain(|member| {
                let ok = match member.strip_prefix(GROUP_PREFIX) {
                    Some(id) => ids.contains(id),
                    None => known.nodes.contains(member.as_str()),
                };
                if !ok {
                    warnings.push(format!("[{view}] layout file: '{member}' of group '{}' is not drawn in this view", group.id));
                }
                ok
            });
            if !group.members.is_empty() {
                kept.push(group);
            }
        }
        if !kept.is_empty() {
            out.insert(view, kept);
        }
    }
    out
}

/// Read the layout of a model and attach it to its diagrams.
///
/// `explicit` is a file the user named: it must exist. Without it, the file
/// next to the model is used when there is one. Returns the warnings of
/// [`DiagramSet::apply_layout`]. The name to save the layout under is set
/// either way, so a first arrangement lands next to the model.
pub fn arrange(
    set: &mut DiagramSet,
    model: &Path,
    explicit: Option<&Path>,
) -> Result<Vec<String>, String> {
    let sidecar = sidecar_path(model);
    let path = match explicit {
        Some(path) => Some(path.to_path_buf()),
        None => sidecar.is_file().then(|| sidecar.clone()),
    };
    set.layout_name = path
        .as_deref()
        .unwrap_or(&sidecar)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned());
    let Some(path) = path else {
        return Ok(Vec::new());
    };
    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("cannot read layout file {}: {error}", path.display()))?;
    let layout = LayoutFile::parse(&text).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(set.apply_layout(layout))
}
