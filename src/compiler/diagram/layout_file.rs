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
//!       "places": { "LC-1": { "dx": 40.0, "dy": -12.5 } } }
//!   ]
//! }
//! ```
//!
//! `views` says what each view shows when opened. `arrangements` holds the
//! manual placement, one per layout: moving a box in the whole view and in
//! an opened container are two arrangements. Offsets are in sheet units,
//! relative to where the automatic layout puts the box.

use super::{DiagramSet, Node};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Version of the layout file format this build reads and writes.
pub const LAYOUT_VERSION: &str = "1";
/// Sheet units beyond which an offset is not a placement.
const FARTHEST: f64 = 100_000.0;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutFile {
    pub arclang_layout: String,
    /// Per diagram id (`lab`, `msm:OperatingModes`): what it shows.
    #[serde(default)]
    pub views: BTreeMap<String, ViewLayout>,
    #[serde(default)]
    pub arrangements: Vec<Arrangement>,
}

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
    pub places: BTreeMap<String, Offset>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offset {
    pub dx: f64,
    pub dy: f64,
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
        for arrangement in &layout.arrangements {
            for (node, offset) in &arrangement.places {
                let sound = |value: f64| value.is_finite() && value.abs() <= FARTHEST;
                if !sound(offset.dx) || !sound(offset.dy) {
                    return Err(format!(
                        "layout file: '{node}' is placed farther than a sheet can be ({}, {})",
                        offset.dx, offset.dy
                    ));
                }
            }
        }
        Ok(layout.normalized())
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
}

impl<'a> Drawn<'a> {
    fn of(nodes: &'a [Node]) -> Self {
        let mut drawn = Drawn {
            nodes: BTreeSet::new(),
            containers: BTreeSet::new(),
        };
        let mut stack: Vec<&Node> = nodes.iter().collect();
        while let Some(node) = stack.pop() {
            drawn.nodes.insert(&node.id);
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
            .map(|diagram| (diagram.id.as_str(), Drawn::of(&diagram.nodes)))
            .collect();
        let mut warnings: Vec<String> = Vec::new();
        let mut kept = LayoutFile {
            arclang_layout: layout.arclang_layout,
            ..LayoutFile::default()
        };

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
            if !places.is_empty() {
                kept.arrangements.push(Arrangement {
                    view,
                    open: arrangement.open,
                    folded,
                    places,
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
