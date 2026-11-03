//! State machine diagrams, in the `.aird`: each top-level region of every
//! machine of the model gets the diagram Capella makes for it, a Mode
//! State Machine (`[MSM]`) on the region, with a box for each of its
//! states and modes (their own regions inside them), a node for each
//! pseudo state, and an edge for each transition. The mappings and styles
//! are those of Capella's own diagrams; the GMF notation stores where each
//! box is and how big (`msm_layout`), so Capella shows the machine drawn
//! when it opens it, and the Simulink connector reads the chart's layout
//! from it as from a diagram drawn in Capella.

use super::context::derived;
use super::msm_layout::{self as layout, Placed, PlacedRegion};
use super::xml::El;
use std::collections::HashMap;

const REPRESENTATION: &str = "platform:/plugin/org.polarsys.capella.core.sirius.analysis/description/common.odesign#//@ownedViewpoints[name='Common']/@ownedRepresentations[name='Mode%20State%20Machine']";
const FONT: (&str, &str) = ("Segoe UI", "8");

fn layer() -> String {
    format!("{REPRESENTATION}/@defaultLayer")
}
fn container_mapping() -> String {
    format!("{}/@containerMappings[name='MSM_ModeState_Container']", layer())
}
fn region_mapping() -> String {
    format!("{}/@subContainerMappings[name='MSM_Region']", container_mapping())
}
fn node_mapping() -> String {
    format!("{}/@nodeMappings[name='MSM_ModeState']", layer())
}
fn edge_mapping() -> String {
    format!("{}/@edgeMappings[name='MSM_Transition']", layer())
}

/// A Sirius element: its feature, kind and uid.
fn sirius(tag: &str, kind: &str, uid: &str) -> El {
    El::root(tag).attr("xmi:type", kind).attr("uid", uid)
}

/// A GMF notation element: its feature, kind and id.
fn notation(tag: &str, kind: &str, id: &str) -> El {
    El::root(tag).attr("xmi:type", kind).attr("xmi:id", id)
}

fn bounds(id: &str, (x, y, w, h): (i32, i32, i32, i32)) -> El {
    notation("layoutConstraint", "notation:Bounds", id).attr("x", &x.to_string()).attr("y", &y.to_string()).attr("width", &w.to_string()).attr("height", &h.to_string())
}

fn shape_style(id: &str) -> El {
    notation("styles", "notation:ShapeStyle", id).attr("fontName", FONT.0).attr("fontHeight", FONT.1)
}

/// How a pseudo state looks: its style, the style's description, the
/// notation kind of the style's figure, its attributes and the node's size
/// in Sirius units (Capella's `MSM_ModeState` conditional styles).
struct PointLook {
    style: &'static str,
    description: &'static str,
    condition: usize,
    figure: &'static str,
    attributes: Vec<(&'static str, String)>,
    size: (&'static str, &'static str),
    resizable: bool,
}

fn image(condition: usize, file: &str, resizable: bool) -> PointLook {
    let path = format!("/org.polarsys.capella.core.sirius.analysis/description/images/{file}");
    PointLook { style: "diagram:WorkspaceImage", description: "style:WorkspaceImageDescription", condition, figure: "3005", attributes: vec![("showIcon", "false".into()), ("workspacePath", path)], size: ("2", "2"), resizable }
}

fn point_look(kind: &str) -> PointLook {
    let border = |mut look: PointLook| {
        look.attributes.splice(1..1, [("borderSize", "1".to_string()), ("borderSizeComputationExpression", "1".to_string())]);
        look
    };
    match kind {
        "ForkPseudoState" | "JoinPseudoState" => border(PointLook {
            style: "diagram:Square",
            description: "style:SquareDescription",
            condition: 5,
            figure: "3003",
            attributes: vec![("showIcon", "false".into()), ("width", "1".into()), ("height", "7".into()), ("color", "0,0,0".into())],
            size: ("1", "7"),
            resizable: true,
        }),
        "ChoicePseudoState" => border(PointLook {
            style: "diagram:Lozenge",
            description: "style:LozengeNodeDescription",
            condition: 6,
            figure: "3017",
            attributes: vec![("showIcon", "false".into()), ("width", "5".into()), ("height", "5".into()), ("color", "168,168,168".into())],
            size: ("5", "5"),
            resizable: true,
        }),
        "FinalState" => image(8, "finalState.svg", false),
        "TerminatePseudoState" => image(9, "terminateState.svg", false),
        "DeepHistoryPseudoState" => image(10, "Pseudostate_deepHistory.svg", true),
        "ShallowHistoryPseudoState" => image(11, "Pseudostate_shallowHistory.svg", true),
        "EntryPointPseudoState" => image(12, "Pseudostate_entryPoint.svg", true),
        "ExitPointPseudoState" => image(13, "Pseudostate_exitPoint.svg", true),
        _ => image(7, "initialState.svg", false),
    }
}

/// One diagram being written: its uid, the model file, the transitions drawn.
struct Writer<'a> {
    uid: String,
    file: &'a str,
    /// Transitions of the diagram's region: (id, source, target).
    transitions: Vec<(String, String, String)>,
    /// The drawn state of each state id: (Sirius uid, notation id).
    drawn: HashMap<String, (String, String)>,
}

impl Writer<'_> {
    fn uid_of(&self, id: &str) -> String {
        format!("{}-{}", self.uid, derived(id))
    }

    fn href(&self, kind: &str, id: &str) -> El {
        El::root("").attr("xmi:type", kind).attr("href", &super::diagrams::uri(self.file, id))
    }

    fn semantic(&self, el: &mut El, kind: &str, id: &str) {
        let mut target = self.href(kind, id);
        target.rename("target");
        let mut semantic = self.href(kind, id);
        semantic.rename("semanticElements");
        el.push(target);
        el.push(semantic);
    }

    /// The edges leaving and reaching a state, as their uids.
    fn edges_of(&self, id: &str) -> (Vec<String>, Vec<String>) {
        let drawn = |t: &&(String, String, String)| self.drawn.contains_key(&t.1) && self.drawn.contains_key(&t.2);
        let outgoing = self.transitions.iter().filter(drawn).filter(|t| t.1 == id).map(|t| self.uid_of(&t.0)).collect();
        let incoming = self.transitions.iter().filter(drawn).filter(|t| t.2 == id).map(|t| self.uid_of(&t.0)).collect();
        (outgoing, incoming)
    }

    fn record(&mut self, placed: &[Placed]) {
        for p in placed {
            let uid = self.uid_of(&p.id);
            self.drawn.insert(p.id.clone(), (uid.clone(), format!("{uid}-view")));
            for r in &p.regions {
                self.record(&r.states);
            }
        }
    }

    /// A state or a pseudo state: its Sirius element and its notation node.
    fn state(&self, p: &Placed, top: bool) -> (El, El) {
        let (uid, view) = self.drawn[&p.id].clone();
        let (outgoing, incoming) = self.edges_of(&p.id);
        let kind = p.kind.as_str();
        let mut el = sirius("ownedDiagramElements", if layout::is_container(kind) { "diagram:DNodeContainer" } else { "diagram:DNode" }, &uid).attr("name", &p.name);
        if !outgoing.is_empty() {
            el.set("outgoingEdges", &outgoing.join(" "));
        }
        if !incoming.is_empty() {
            el.set("incomingEdges", &incoming.join(" "));
        }
        if layout::is_container(kind) {
            return self.container(p, el, &uid, &view, top);
        }
        let look = point_look(kind);
        el.set("width", look.size.0);
        el.set("height", look.size.1);
        if look.resizable {
            el.set("resizeKind", "NSEW");
        }
        self.semantic(&mut el, &p.xsi_type, &p.id);
        let style_uid = format!("{uid}-style");
        let mut style = sirius("ownedStyle", look.style, &style_uid);
        for (k, v) in &look.attributes {
            style.set(k, v);
        }
        el.push(style.with(El::root("description").attr("xmi:type", look.description).attr("href", &format!("{}/@conditionnalStyles.{}/@style", node_mapping(), look.condition))));
        el.push(El::root("actualMapping").attr("xmi:type", "description_1:NodeMapping").attr("href", &node_mapping()));
        let node = notation("children", "notation:Node", &view)
            .attr("type", if top { "2001" } else { "3007" })
            .attr("element", &uid)
            .with(notation("children", "notation:Node", &format!("{view}-label")).attr("type", if top { "5002" } else { "5003" }).with(notation("layoutConstraint", "notation:Location", &format!("{view}-label-at"))))
            .with(
                notation("children", "notation:Node", &format!("{view}-figure"))
                    .attr("type", look.figure)
                    .attr("element", &style_uid)
                    .with(notation("styles", "notation:ShapeStyle", &format!("{view}-figure-style")).attr("fontName", FONT.0))
                    .with(notation("layoutConstraint", "notation:Bounds", &format!("{view}-figure-at"))),
            )
            .with(shape_style(&format!("{view}-style")))
            .with(bounds(&format!("{view}-at"), p.bounds));
        (el, node)
    }

    /// A state or a mode: a container, its regions inside it.
    fn container(&self, p: &Placed, mut el: El, uid: &str, view: &str, top: bool) -> (El, El) {
        let mode = p.kind == "Mode";
        el.set("childrenPresentation", "VerticalStack");
        self.semantic(&mut el, &p.xsi_type, &p.id);
        let (condition, colors) = if mode { (1, "234,239,239") } else { (3, "239,239,239") };
        el.push(container_style(&format!("{uid}-style"), colors, colors, false).with(El::root("description").attr("xmi:type", "style:FlatContainerStyleDescription").attr("href", &format!("{}/@conditionnalStyles.{condition}/@style", container_mapping()))));
        el.push(El::root("actualMapping").attr("xmi:type", "description_1:ContainerMapping").attr("href", &container_mapping()));
        let mut compartment = notation("children", "notation:Node", &format!("{view}-compartment")).attr("type", if top { "7001" } else { "7002" });
        for r in &p.regions {
            let (region, node) = self.region(r, mode);
            el.push(region);
            compartment.push(node);
        }
        compartment.push(notation("styles", "notation:SortingStyle", &format!("{view}-compartment-sorting")));
        compartment.push(notation("styles", "notation:FilteringStyle", &format!("{view}-compartment-filtering")));
        let node = notation("children", "notation:Node", view)
            .attr("type", if top { "2002" } else { "3008" })
            .attr("element", uid)
            .with(notation("children", "notation:Node", &format!("{view}-label")).attr("type", if top { "5006" } else { "5005" }))
            .with(compartment)
            .with(shape_style(&format!("{view}-style")))
            .with(bounds(&format!("{view}-at"), p.bounds));
        (el, node)
    }

    /// A region of a state, with its states.
    fn region(&self, r: &PlacedRegion, in_mode: bool) -> (El, El) {
        let uid = self.uid_of(&r.id);
        let view = format!("{uid}-view");
        let mut el = sirius("ownedDiagramElements", "diagram:DNodeContainer", &uid).attr("name", " [region]");
        self.semantic(&mut el, &r.xsi_type, &r.id);
        let (condition, background, foreground) = if in_mode { (0, "234,239,239", "195,208,208") } else { (1, "239,239,239", "208,208,208") };
        el.push(container_style(&format!("{uid}-style"), background, foreground, true).with(El::root("description").attr("xmi:type", "style:FlatContainerStyleDescription").attr("href", &format!("{}/@conditionnalStyles.{condition}/@style", region_mapping()))));
        el.push(El::root("actualMapping").attr("xmi:type", "description_1:ContainerMapping").attr("href", &region_mapping()));
        let mut compartment = notation("children", "notation:Node", &format!("{view}-compartment")).attr("type", "7002");
        for s in &r.states {
            let (child, node) = self.state(s, false);
            el.push(child);
            compartment.push(node);
        }
        for (style, suffix) in [("notation:SortingStyle", "sorting"), ("notation:FilteringStyle", "filtering"), ("notation:DrawerStyle", "drawer")] {
            compartment.push(notation("styles", style, &format!("{view}-compartment-{suffix}")));
        }
        let node = notation("children", "notation:Node", &view)
            .attr("type", "3008")
            .attr("element", &uid)
            .with(notation("children", "notation:Node", &format!("{view}-label")).attr("type", "5005"))
            .with(compartment)
            .with(shape_style(&format!("{view}-style")))
            .with(bounds(&format!("{view}-at"), r.bounds));
        (el, node)
    }

    /// A transition: its edge and its notation, a straight line between the
    /// centers of its ends (a loop on a state goes round its top right corner).
    fn transition(&self, (id, source, target): &(String, String, String), at: &HashMap<String, (i32, i32, i32, i32)>, label: &str) -> Option<(El, El)> {
        let ((source_uid, source_view), (target_uid, target_view)) = (self.drawn.get(source)?, self.drawn.get(target)?);
        let (a, b) = (at.get(source)?, at.get(target)?);
        let uid = self.uid_of(id);
        let view = format!("{uid}-view");
        let mut el = sirius("ownedDiagramElements", "diagram:DEdge", &uid).attr("name", label).attr("sourceNode", source_uid).attr("targetNode", target_uid);
        self.semantic(&mut el, "org.polarsys.capella.core.data.capellacommon:StateTransition", id);
        el.push(
            sirius("ownedStyle", "diagram:EdgeStyle", &format!("{uid}-style"))
                .attr("strokeColor", "0,0,0")
                .with(El::root("description").attr("xmi:type", "style:EdgeStyleDescription").attr("href", &format!("{}/@style", edge_mapping())))
                .with(sirius("centerLabelStyle", "diagram:CenterLabelStyle", &format!("{uid}-label-style")).attr("showIcon", "false")),
        );
        el.push(El::root("actualMapping").attr("xmi:type", "description_1:EdgeMapping").attr("href", &edge_mapping()));
        let center = |r: &(i32, i32, i32, i32)| (r.0 + r.2 / 2, r.1 + r.3 / 2);
        let (from, to) = (center(a), center(b));
        let points = route(*a, *b, at, self.routed(id, at));
        // GMF's relative bendpoints: each point from the source's reference and from the target's.
        let relative: Vec<String> = points.iter().map(|&(x, y)| format!("[{x}, {y}, {}, {}]", from.0 + x - to.0, from.1 + y - to.1)).collect();
        let mut edge = notation("edges", "notation:Edge", &view).attr("type", "4001").attr("element", &uid).attr("source", source_view).attr("target", target_view);
        for (kind, y) in [("6001", -10), ("6002", 10), ("6003", 10)] {
            edge.push(notation("children", "notation:Node", &format!("{view}-{kind}")).attr("type", kind).with(notation("layoutConstraint", "notation:Bounds", &format!("{view}-{kind}-at")).attr("y", &y.to_string())));
        }
        edge.push(notation("styles", "notation:ConnectorStyle", &format!("{view}-connector")));
        edge.push(notation("styles", "notation:FontStyle", &format!("{view}-font")).attr("fontName", FONT.0).attr("fontHeight", FONT.1));
        edge.push(notation("bendpoints", "notation:RelativeBendpoints", &format!("{view}-bendpoints")).attr("points", &relative.join("$")));
        Some((el, edge))
    }
}

impl Writer<'_> {
    /// How many transitions before this one go round the boxes (to stagger them).
    fn routed(&self, id: &str, at: &HashMap<String, (i32, i32, i32, i32)>) -> i32 {
        let before = self.transitions.iter().take_while(|t| t.0 != id);
        before.filter(|(_, s, t)| s != t && at.get(s).zip(at.get(t)).is_some_and(|(a, b)| goes_round(*a, *b, at))).count() as i32
    }
}

fn center(r: (i32, i32, i32, i32)) -> (i32, i32) {
    (r.0 + r.2 / 2, r.1 + r.3 / 2)
}

fn contains(r: (i32, i32, i32, i32), p: (i32, i32)) -> bool {
    p.0 >= r.0 && p.0 <= r.0 + r.2 && p.1 >= r.1 && p.1 <= r.1 + r.3
}

fn inside(r: (i32, i32, i32, i32), outer: (i32, i32, i32, i32)) -> bool {
    contains(outer, (r.0, r.1)) && contains(outer, (r.0 + r.2, r.1 + r.3))
}

/// The boxes a straight line between two boxes would cross, other than them,
/// those holding them and those they hold.
fn obstacles(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32), at: &HashMap<String, (i32, i32, i32, i32)>) -> Vec<(i32, i32, i32, i32)> {
    let (from, to) = (center(a), center(b));
    let mut found: Vec<(i32, i32, i32, i32)> = at
        .values()
        .copied()
        .filter(|&r| r != a && r != b && !contains(r, from) && !contains(r, to) && !inside(r, a) && !inside(r, b))
        .filter(|&r| {
            // Sampled along the line: enough for boxes tens of pixels wide.
            (1..40).any(|k| {
                let t = k as f64 / 40.0;
                let p = ((from.0 as f64 + t * (to.0 - from.0) as f64) as i32, (from.1 as f64 + t * (to.1 - from.1) as f64) as i32);
                contains(r, p)
            })
        })
        .collect();
    found.sort();
    found
}

/// Whether a transition goes back (right to left) or would cross a box: then it goes round.
fn goes_round(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32), at: &HashMap<String, (i32, i32, i32, i32)>) -> bool {
    center(b).0 < center(a).0 || !obstacles(a, b, at).is_empty()
}

/// The points of a transition from the center of its source, as offsets: a
/// straight line; a loop round the top right corner of its state; under the
/// boxes when it goes back, over them when it would cross one, each next one
/// a little further out.
fn route(a: (i32, i32, i32, i32), b: (i32, i32, i32, i32), at: &HashMap<String, (i32, i32, i32, i32)>, before: i32) -> Vec<(i32, i32)> {
    let (from, to) = (center(a), center(b));
    if a == b {
        let (dx, dy) = (a.2 / 2 + 30, a.3 / 2 + 30);
        return vec![(0, 0), (dx, 0), (dx, -dy), (0, -dy), (0, 0)];
    }
    if !goes_round(a, b, at) {
        return vec![(0, 0), (to.0 - from.0, to.1 - from.1)];
    }
    let mut boxes = obstacles(a, b, at);
    boxes.extend([a, b]);
    let stagger = 12 * (before % 6);
    let y = if to.0 < from.0 { boxes.iter().map(|r| r.1 + r.3).max().unwrap_or(from.1) + 20 + stagger } else { boxes.iter().map(|r| r.1).min().unwrap_or(from.1) - 20 - stagger };
    vec![(0, 0), (0, y - from.1), (to.0 - from.0, y - from.1), (to.0 - from.0, to.1 - from.1)]
}

fn container_style(uid: &str, background: &str, foreground: &str, region: bool) -> El {
    let mut style = sirius("ownedStyle", "diagram:FlatContainerStyle", uid);
    if region {
        style.set("showIcon", "false");
        style.set("labelAlignment", "LEFT");
    }
    style
        .attr("borderSize", "1")
        .attr("borderSizeComputationExpression", "1")
        .attr("borderColor", "69,69,69")
        .attr("backgroundStyle", "GradientTopToBottom")
        .attr("backgroundColor", background)
        .attr("foregroundColor", foreground)
}

/// The label Capella gives a transition: its trigger, then its guard in brackets.
fn labels(region: &El, into: &mut HashMap<String, String>) {
    for child in region.children() {
        if child.tag() == "ownedTransitions" {
            let trigger = child.get("triggerDescription").unwrap_or("");
            let guard = child
                .children()
                .iter()
                .filter(|c| c.tag() == "ownedConstraints")
                .flat_map(|c| c.children())
                .flat_map(|s| s.children())
                .find(|b| b.tag() == "bodies")
                .map(|b| b.content().to_string());
            let label = match guard {
                Some(guard) => format!("{trigger} [{guard}] "),
                None => trigger.to_string(),
            };
            into.insert(child.id().to_string(), label);
        }
        labels(child, into);
    }
}

/// The machines under an element, at any depth.
fn machines<'a>(el: &'a El, into: &mut Vec<&'a El>) {
    for child in el.children() {
        if child.tag() == "ownedStateMachines" {
            into.push(child);
        }
        machines(child, into);
    }
}

/// The descriptor (for the Common view) and the representation of the
/// diagram of each top-level region of every machine under `root`.
pub(super) fn diagrams(root: &El, file: &str) -> Vec<(String, El, El)> {
    let mut found = Vec::new();
    machines(root, &mut found);
    let mut out = Vec::new();
    for machine in found {
        let regions: Vec<&El> = machine.children().iter().filter(|c| c.tag() == "ownedRegions").collect();
        let machine_name = machine.get("name").unwrap_or("");
        for region in &regions {
            let name = if regions.len() > 1 { format!("[MSM] {machine_name} - {}", region.get("name").unwrap_or("")) } else { format!("[MSM] {machine_name}") };
            out.push(diagram(region, &name, file));
        }
    }
    out
}

fn diagram(region: &El, name: &str, file: &str) -> (String, El, El) {
    let uid = format!("_{}", derived(&format!("msm:{}", region.id())));
    let transitions = layout::transitions(region);
    let (placed, _) = layout::region(region, &transitions, true);
    let mut writer = Writer { uid: uid.clone(), file, transitions, drawn: HashMap::new() };
    writer.record(&placed);
    let mut at = HashMap::new();
    layout::absolute(&placed, (0, 0), &mut at);
    let mut names = HashMap::new();
    labels(region, &mut names);

    let mut elements = Vec::new();
    let mut data = notation("data", "notation:Diagram", &format!("{uid}-notation")).attr("type", "Sirius").attr("element", &uid).attr("measurementUnit", "Pixel");
    for p in &placed {
        let (el, node) = writer.state(p, true);
        elements.push(el);
        data.push(node);
    }
    data.push(notation("styles", "notation:DiagramStyle", &format!("{uid}-notation-style")));
    for t in &writer.transitions {
        let label = names.get(&t.0).cloned().unwrap_or_default();
        if let Some((el, edge)) = writer.transition(t, &at, &label) {
            elements.push(el);
            data.push(edge);
        }
    }
    let region_kind = region.kind().to_string();
    let mut representation = El::root("diagram:DSemanticDiagram").attr("uid", &uid);
    representation.push(
        sirius("ownedAnnotationEntries", "description:AnnotationEntry", &format!("{uid}-customization"))
            .attr("source", "DANNOTATION_CUSTOMIZATION_KEY")
            .with(sirius("data", "diagram:ComputedStyleDescriptionRegistry", &format!("{uid}-computed-styles"))),
    );
    representation.push(sirius("ownedAnnotationEntries", "description:AnnotationEntry", &format!("{uid}-gmf")).attr("source", "GMF_DIAGRAMS").with(data));
    for el in elements {
        representation.push(el);
    }
    representation.push(El::root("description").attr("xmi:type", "description_1:DiagramDescription").attr("href", REPRESENTATION));
    for filter in [
        "platform:/plugin/org.polarsys.kitalpha.ad.integration.sirius/description/ad.odesign#//@ownedViewpoints[name='ad']/@ownedRepresentations[name='AD%20diagram']/@filters[name='ModelExtensionFilter']",
        &format!("{REPRESENTATION}/@filters[name='hide.computed.transitions.filter']"),
    ] {
        representation.push(El::root("activatedFilters").attr("xmi:type", "filter:CompositeFilterDescription").attr("href", filter));
    }
    representation.push(sirius("filterVariableHistory", "diagram:FilterVariableHistory", &format!("{uid}-filters")));
    representation.push(El::root("activatedLayers").attr("xmi:type", "description_1:Layer").attr("href", &layer()));
    let mut target = writer.href(&region_kind, region.id());
    target.rename("target");
    representation.push(target);
    let mut descriptor_target = writer.href(&region_kind, region.id());
    descriptor_target.rename("target");
    let descriptor = sirius("ownedRepresentationDescriptors", "viewpoint:DRepresentationDescriptor", &format!("{uid}-descriptor"))
        .attr("name", name)
        .attr("repPath", &format!("#{uid}"))
        .with(El::root("description").attr("xmi:type", "description_1:DiagramDescription").attr("href", REPRESENTATION))
        .with(descriptor_target);
    ("common".to_string(), descriptor, representation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pseudo_state_looks_as_in_capella() {
        assert_eq!(point_look("InitialPseudoState").condition, 7);
        assert_eq!(point_look("ChoicePseudoState").style, "diagram:Lozenge");
        assert_eq!(point_look("JoinPseudoState").figure, "3003");
        assert_eq!(layout::local("org.polarsys.capella.core.data.capellacommon:Mode"), "Mode");
    }
}
