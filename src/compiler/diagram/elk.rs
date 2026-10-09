//! ELK graph export: the layout-engine input for one diagram.
//!
//! Produces the [ELK JSON format](https://eclipse.dev/elk/documentation/tooldevelopers/graphdatastructure/jsonformat.html):
//! hierarchical nodes, ports pinned to a side by direction, and edges whose
//! endpoints are port ids wherever the model binds a port. Every element
//! carries an `arc` object with what a renderer needs to draw the Arcadia
//! notation (kind, properties, direction), so the laid-out graph returned by
//! ELK is self-sufficient. (ELK reports each edge route relative to the node
//! containing both its ends, named in the edge's `container`.)
//!
//! Sizes are estimates from label lengths: layout must not depend on a
//! browser measuring text, or it could not run headless and be tested.

use super::{Diagram, Edge, Node, NodeKind, Port, PortDirection, ViewKind};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

// Text metrics of the viewer's font (Arial); keep in step with
// viewer/arcviz-render.js.
const CHAR_WIDTH: f64 = 7.2;
const LABEL_HEIGHT: f64 = 16.0;
const SMALL_CHAR_WIDTH: f64 = 5.6;
const SMALL_LABEL_HEIGHT: f64 = 12.0;
const NODE_MIN_WIDTH: f64 = 112.0;
const NODE_MIN_HEIGHT: f64 = 42.0;
/// Room for the kind glyph on the left and the same margin on the right.
const NODE_HORIZONTAL_PADDING: f64 = 44.0;
/// Extra line under the name: `realizes <component>`.
const REALIZES_LINE_HEIGHT: f64 = 13.0;
/// A box listing lines under its name (class fields, enumeration values):
/// where the list starts, the pitch of its lines and its side inset. Keep in
/// step with COMPARTMENT in viewer/arcviz-render.js.
const COMPARTMENT_TOP: f64 = 34.0;
const COMPARTMENT_LINE_HEIGHT: f64 = 14.0;
const COMPARTMENT_INSET: f64 = 10.0;
const COMPARTMENT_CHAR_WIDTH: f64 = 6.2;
/// Diameter of the filled disc marking a machine's initial state.
const INITIAL_STATE_SIZE: f64 = 18.0;
const COMPONENT_PORT_SIZE: f64 = 12.0;
const FUNCTION_PORT_SIZE: f64 = 8.0;
const PORT_PITCH: f64 = 20.0;
/// Container padding: the name band on top, the safety badge at the bottom.
const CONTAINER_PADDING_TOP: f64 = 34.0;
const CONTAINER_PADDING_SIDE: f64 = 18.0;
const CONTAINER_PADDING_BOTTOM: f64 = 20.0;
/// Wrap long left-to-right flows onto several rows beyond this width:height.
const TARGET_ASPECT_RATIO: &str = "1.9";

/// Convert a diagram to an ELK graph.
///
/// Every edge is laid out, including one between a node and its own
/// container: ELK routes it from the container's border to the node.
///
/// A scenario is not a graph to lay out: its lifelines are columns and its
/// messages rows, in declared order. It is exported in the same shape with
/// `arc.layout = "sequence"`, and the viewer draws it without calling ELK.
pub fn to_elk(diagram: &Diagram) -> Value {
    if diagram.kind == ViewKind::Es {
        return sequence_graph(diagram);
    }
    let mut ancestors: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in &diagram.nodes {
        record_ancestors(node, &mut Vec::new(), &mut ancestors);
    }
    let laid_out: Vec<&Edge> = diagram.edges.iter().collect();

    // Nodes an edge reaches, with all their ancestors. A container outside
    // this set holds nothing to route, so its children are packed in rows
    // instead of being stacked in a single layer.
    let mut wired: HashSet<&str> = HashSet::new();
    for edge in &laid_out {
        for end in [edge.source.as_str(), edge.target.as_str()] {
            wired.insert(end);
            wired.extend(ancestors.get(end).into_iter().flatten().copied());
        }
    }
    // Edges may cross container borders only when there are containers.
    // On a flat graph, laying the hierarchy out as one stops ELK from
    // packing unconnected groups side by side: they pile up in one column.
    let nested = diagram.nodes.iter().any(|node| !node.children.is_empty());
    let hierarchy = if nested {
        "INCLUDE_CHILDREN"
    } else {
        "SEPARATE_CHILDREN"
    };
    // A breakdown reads top-down; everything else is a left-to-right flow.
    let direction = if diagram.kind == ViewKind::Pbs {
        "DOWN"
    } else {
        "RIGHT"
    };
    let mut layout = if laid_out.is_empty() {
        packing_options()
    } else {
        json!({
            "elk.algorithm": "layered",
            "elk.direction": direction,
            "elk.hierarchyHandling": hierarchy,
            "elk.edgeRouting": "ORTHOGONAL",
            "elk.layered.spacing.nodeNodeBetweenLayers": "46",
            "elk.spacing.nodeNode": "28",
            "elk.spacing.edgeNode": "16",
            "elk.spacing.edgeEdge": "12",
            "elk.layered.spacing.edgeNodeBetweenLayers": "16",
        })
    };
    // Long left-to-right dataflows are wrapped onto several rows. A state
    // machine is a cycle and a breakdown is a tree: wrapping either only
    // adds detours.
    if !laid_out.is_empty() && !matches!(diagram.kind, ViewKind::Msm | ViewKind::Pbs) {
        layout["elk.layered.wrapping.strategy"] = json!("MULTI_EDGE");
        layout["elk.aspectRatio"] = json!(TARGET_ASPECT_RATIO);
    }

    json!({
        "id": diagram.id,
        "layoutOptions": layout,
        "children": diagram.nodes.iter().map(|node| elk_node(node, &wired)).collect::<Vec<_>>(),
        "edges": laid_out.into_iter().map(elk_edge).collect::<Vec<_>>(),
        "arc": {
            "view": diagram.kind,
            "layout": "elk",
            "title": diagram.title,
            "chains": diagram.chains,
        },
    })
}

fn record_ancestors<'a>(
    node: &'a Node,
    path: &mut Vec<&'a str>,
    out: &mut HashMap<&'a str, Vec<&'a str>>,
) {
    out.insert(node.id.as_str(), path.clone());
    path.push(node.id.as_str());
    for child in &node.children {
        record_ancestors(child, path, out);
    }
    path.pop();
}

fn sequence_graph(diagram: &Diagram) -> Value {
    let nothing_wired = HashSet::new();
    json!({
        "id": diagram.id,
        "children": diagram.nodes.iter().map(|node| elk_node(node, &nothing_wired)).collect::<Vec<_>>(),
        "edges": diagram.edges.iter().map(elk_edge).collect::<Vec<_>>(),
        "arc": {
            "view": diagram.kind,
            "layout": "sequence",
            "title": diagram.title,
            "chains": diagram.chains,
        },
    })
}

/// Layout of a group with no edges to route: rows of boxes.
fn packing_options() -> Value {
    json!({
        "elk.algorithm": "rectpacking",
        "elk.aspectRatio": TARGET_ASPECT_RATIO,
        "elk.spacing.nodeNode": "22",
    })
}

fn elk_node(node: &Node, wired: &HashSet<&str>) -> Value {
    let on_function = node.kind == NodeKind::Function;
    let port_size = if on_function {
        FUNCTION_PORT_SIZE
    } else {
        COMPONENT_PORT_SIZE
    };
    let is_container = !node.children.is_empty();
    let has_safety = ["safety_level", "asil"]
        .iter()
        .any(|key| node.properties.contains_key(*key));

    let label_width = text_width(&node.name, CHAR_WIDTH);
    let realizes_width = if node.realizes.is_empty() {
        0.0
    } else {
        text_width(
            &format!("realizes {}", node.realizes.join(", ")),
            SMALL_CHAR_WIDTH,
        )
    };
    let realizes_height = if node.realizes.is_empty() {
        0.0
    } else {
        REALIZES_LINE_HEIGHT
    };
    let side_count = |direction: PortDirection| {
        node.ports
            .iter()
            .filter(|p| p.direction == direction)
            .count()
    };
    let busiest_side = side_count(PortDirection::In)
        .max(side_count(PortDirection::Out))
        .max(1) as f64;

    let badge_room = if has_safety && !is_container {
        10.0
    } else {
        0.0
    };
    let (width, height) = if node.kind == NodeKind::InitialState {
        (INITIAL_STATE_SIZE, INITIAL_STATE_SIZE)
    } else if !node.compartment.is_empty() {
        let widest_line = node
            .compartment
            .iter()
            .map(|line| text_width(line, COMPARTMENT_CHAR_WIDTH))
            .fold(0.0, f64::max);
        (
            (label_width + NODE_HORIZONTAL_PADDING)
                .max(widest_line + 2.0 * COMPARTMENT_INSET)
                .max(NODE_MIN_WIDTH),
            COMPARTMENT_TOP
                + node.compartment.len() as f64 * COMPARTMENT_LINE_HEIGHT
                + COMPARTMENT_INSET,
        )
    } else {
        (
            (label_width.max(realizes_width) + NODE_HORIZONTAL_PADDING).max(NODE_MIN_WIDTH),
            (LABEL_HEIGHT + 10.0 + busiest_side * PORT_PITCH)
                .max(NODE_MIN_HEIGHT + realizes_height + badge_room),
        )
    };
    let oriented = node
        .ports
        .iter()
        .any(|p| p.direction != PortDirection::Undirected);

    let mut layout = json!({
        "elk.portConstraints": if oriented { "FIXED_SIDE" } else { "FREE" },
        "elk.spacing.portPort": PORT_PITCH.to_string(),
    });
    let mut out = json!({
        "id": node.id,
        "labels": [{ "text": node.name }],
        "ports": node.ports.iter().map(|port| elk_port(port, port_size)).collect::<Vec<_>>(),
        "arc": {
            "kind": node.kind,
            "uuid": node.uuid,
            "realizes": node.realizes,
            "compartment": node.compartment,
            "properties": node.properties,
        },
    });
    if is_container {
        let top = CONTAINER_PADDING_TOP + realizes_height;
        layout["elk.padding"] = json!(format!(
            "[top={top},left={CONTAINER_PADDING_SIDE},bottom={CONTAINER_PADDING_BOTTOM},right={CONTAINER_PADDING_SIDE}]"
        ));
        layout["elk.nodeSize.constraints"] = json!("MINIMUM_SIZE PORTS");
        layout["elk.nodeSize.minimum"] = json!(format!("({width},{height})"));
        if !wired.contains(node.id.as_str()) {
            for (key, value) in packing_options().as_object().into_iter().flatten() {
                layout[key] = value.clone();
            }
        }
        out["children"] = node
            .children
            .iter()
            .map(|child| elk_node(child, wired))
            .collect();
    } else {
        out["width"] = json!(width);
        out["height"] = json!(height);
    }
    out["layoutOptions"] = layout;
    out
}

/// Ports carry no layout label: names would cost more width than the
/// diagram can afford, so the viewer shows them on hover.
fn elk_port(port: &Port, size: f64) -> Value {
    let mut out = json!({
        "id": port.id,
        "width": size,
        "height": size,
        "arc": {
            "name": port.name,
            "direction": port.direction,
            "interface": port.interface,
            "protocol": port.protocol,
            "synthesized": port.synthesized,
        },
    });
    let side = match port.direction {
        PortDirection::In => Some("WEST"),
        PortDirection::Out => Some("EAST"),
        PortDirection::InOut | PortDirection::Undirected => None,
    };
    if let Some(side) = side {
        out["layoutOptions"] = json!({ "elk.port.side": side });
    }
    out
}

fn elk_edge(edge: &Edge) -> Value {
    json!({
        "id": edge.id,
        "sources": [edge.source_port.as_ref().unwrap_or(&edge.source)],
        "targets": [edge.target_port.as_ref().unwrap_or(&edge.target)],
        "labels": [{
            "text": edge.label,
            "width": text_width(&edge.label, SMALL_CHAR_WIDTH),
            "height": SMALL_LABEL_HEIGHT,
        }],
        "arc": {
            "kind": edge.kind,
            "source_node": edge.source,
            "target_node": edge.target,
            "exchange_item": edge.exchange_item,
            "properties": edge.properties,
        },
    })
}

fn text_width(text: &str, char_width: f64) -> f64 {
    (text.chars().count() as f64 * char_width).ceil()
}

/// Edge endpoints of an ELK graph that name no node or port of that graph.
/// Always empty for graphs produced by [`to_elk`]; exposed so tests (and
/// consumers) can assert it.
pub fn dangling_references(graph: &Value) -> Vec<String> {
    fn collect_ids<'a>(node: &'a Value, ids: &mut HashSet<&'a str>) {
        for child in node["children"].as_array().into_iter().flatten() {
            ids.extend(child["id"].as_str());
            for port in child["ports"].as_array().into_iter().flatten() {
                ids.extend(port["id"].as_str());
            }
            collect_ids(child, ids);
        }
    }
    let mut ids = HashSet::new();
    collect_ids(graph, &mut ids);

    let mut dangling = Vec::new();
    for edge in graph["edges"].as_array().into_iter().flatten() {
        for role in ["sources", "targets"] {
            for endpoint in edge[role].as_array().into_iter().flatten() {
                match endpoint.as_str() {
                    Some(id) if ids.contains(id) => {}
                    other => dangling.push(format!("edge {}: {role} {:?}", edge["id"], other)),
                }
            }
        }
    }
    dangling
}
