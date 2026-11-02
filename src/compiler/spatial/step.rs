//! The model to CAD: STEP, ISO 10303-21, application protocol AP242
//! (ISO 10303-242, managed model-based 3D engineering), which CATIA,
//! SOLIDWORKS, NX, Creo, Inventor, Fusion and FreeCAD read.
//!
//! The product structure is the system's: the model, each architecture
//! level, its physical nodes and the behavior components they hold, and
//! the EPBS breakdown (systems, subsystems, items). An element with a
//! `shape` is a solid (a box or a cylinder, as a B-rep), placed by its
//! `position` and `orientation` in its owner; an element whose children
//! have shapes is an assembly. Lengths are millimetres.
//!
//! A top-level element with a `location` and no `position` is placed by
//! its real distance from the first located element (east, north, up on
//! the local tangent plane, within 0.1 % over tens of kilometres), so a
//! site keeps its layout in CAD. What has no shape anywhere below it is
//! not exported: CAD gets the geometry the model states, nothing guessed.

use super::{location, orientation, position, shape, GeoPoint, Offset, Rotation, Shape, EARTH_RADIUS};
use crate::compiler::ast::{AttributeValue, Model};
use std::collections::HashMap;

/// One product of the structure.
#[derive(Debug, Clone)]
struct Item {
    id: String,
    name: String,
    description: String,
    shape: Option<Shape>,
    offset: Offset,
    rotation: Rotation,
    location: Option<GeoPoint>,
    placed: bool,
    children: Vec<Item>,
}

impl Item {
    fn new(id: &str, name: &str, attributes: &HashMap<String, AttributeValue>) -> Item {
        let get = |key: &str| attributes.get(key);
        let offset = get("position").and_then(|v| position(v).ok());
        Item {
            id: id.to_string(),
            name: name.to_string(),
            description: get("description").and_then(AttributeValue::as_string).unwrap_or_default().to_string(),
            shape: get("shape").and_then(|v| shape(v).ok()),
            placed: offset.is_some(),
            offset: offset.unwrap_or_default(),
            rotation: get("orientation").and_then(|v| orientation(v).ok()).unwrap_or_default(),
            location: get("location").and_then(|v| location(v).ok()),
            children: Vec::new(),
        }
    }

    fn group(id: &str, name: &str, children: Vec<Item>) -> Item {
        Item { id: id.to_string(), name: name.to_string(), description: String::new(), shape: None, offset: Offset::default(), rotation: Rotation::default(), location: None, placed: true, children }
    }

    /// Kept when it, or something below it, has a shape.
    fn pruned(mut self) -> Option<Item> {
        self.children = self.children.into_iter().filter_map(Item::pruned).collect();
        (self.shape.is_some() || !self.children.is_empty()).then_some(self)
    }

    fn solids(&self) -> usize {
        usize::from(self.shape.is_some()) + self.children.iter().map(Item::solids).sum::<usize>()
    }
}

/// The product structure of a model: one group per level with geometry.
fn structure(ast: &Model) -> Vec<Item> {
    let mut levels = Vec::new();
    for pa in &ast.physical_architecture {
        let nodes = pa
            .nodes
            .iter()
            .map(|node| {
                let mut item = Item::new(&node.id, &node.name, &node.attributes);
                item.children = node.behavior_components.iter().map(|b| Item::new(&b.id, &b.name, &b.attributes)).collect();
                item
            })
            .collect();
        let id = if pa.id.is_empty() { pa.name.clone() } else { pa.id.clone() };
        levels.push(Item::group(&id, &pa.name, nodes));
    }
    for epbs in &ast.epbs {
        let systems = epbs
            .systems
            .iter()
            .map(|system| {
                let mut s = Item::new(&system.name, &system.name, &system.attributes);
                s.children = system
                    .subsystems
                    .iter()
                    .map(|subsystem| {
                        let mut sub = Item::new(&subsystem.name, &subsystem.name, &subsystem.attributes);
                        sub.children = subsystem.items.iter().map(|item| Item::new(&item.name, &item.name, &item.attributes)).collect();
                        sub
                    })
                    .collect();
                s
            })
            .collect();
        let id = if epbs.id.is_empty() { epbs.name.clone() } else { epbs.id.clone() };
        levels.push(Item::group(&id, &epbs.name, systems));
    }
    levels.into_iter().filter_map(Item::pruned).collect()
}

/// East, north, up of `point` from `origin`, metres, on the local tangent plane.
fn enu(origin: GeoPoint, point: GeoPoint) -> Offset {
    let lat0 = origin.lat.to_radians();
    Offset {
        x: EARTH_RADIUS * (point.lon - origin.lon).to_radians() * lat0.cos(),
        y: EARTH_RADIUS * (point.lat - origin.lat).to_radians(),
        z: point.alt.unwrap_or(0.0) - origin.alt.unwrap_or(0.0),
    }
}

/// Top-level elements with a location and no position are placed by their distance from the first located one.
fn place_sites(levels: &mut [Item]) -> Option<GeoPoint> {
    let origin = levels.iter().flat_map(|l| l.children.iter()).find_map(|i| i.location)?;
    for level in levels.iter_mut() {
        for item in level.children.iter_mut().filter(|i| !i.placed) {
            if let Some(at) = item.location {
                item.offset = enu(origin, at);
                item.placed = true;
            }
        }
    }
    Some(origin)
}

/// An ISO 10303-21 string: quotes doubled, what is not printable ASCII as `\X2\hhhh\X0\`.
fn string(text: &str) -> String {
    let mut out = String::from("'");
    let mut wide = String::new();
    let flush = |out: &mut String, wide: &mut String| {
        if !wide.is_empty() {
            out.push_str("\\X2\\");
            out.push_str(wide);
            out.push_str("\\X0\\");
            wide.clear();
        }
    };
    for c in text.chars() {
        if (' '..='~').contains(&c) {
            flush(&mut out, &mut wide);
            match c {
                '\'' => out.push_str("''"),
                '\\' => out.push_str("\\\\"),
                _ => out.push(c),
            }
        } else {
            let mut units = [0u16; 2];
            for unit in c.encode_utf16(&mut units) {
                wide.push_str(&format!("{unit:04X}"));
            }
        }
    }
    flush(&mut out, &mut wide);
    out.push('\'');
    out
}

/// An ISO 10303-21 real: always with its decimal point.
fn real(value: f64) -> String {
    let value = if value.abs() < 1e-12 { 0.0 } else { value };
    let text = format!("{}", (value * 1e9).round() / 1e9);
    if text.contains('.') || text.contains('e') || text.contains('E') { text } else { format!("{text}.") }
}

fn mm(metres: f64) -> f64 {
    metres * 1000.0
}

/// The rotation matrix columns (x axis, z axis) of yaw about z, pitch about y, roll about x.
fn axes(r: Rotation) -> ([f64; 3], [f64; 3]) {
    let (cy, sy) = (r.yaw.cos(), r.yaw.sin());
    let (cp, sp) = (r.pitch.cos(), r.pitch.sin());
    let (cr, sr) = (r.roll.cos(), r.roll.sin());
    // R = Rz(yaw) * Ry(pitch) * Rx(roll)
    let x = [cy * cp, sy * cp, -sp];
    let z = [cy * sp * cr + sy * sr, sy * sp * cr - cy * sr, cp * cr];
    (x, z)
}

struct Writer {
    lines: Vec<String>,
    context: usize,
    product_context: usize,
    definition_context: usize,
}

impl Writer {
    fn add(&mut self, entity: String) -> usize {
        self.lines.push(entity);
        self.lines.len()
    }

    fn point(&mut self, p: [f64; 3]) -> usize {
        self.add(format!("CARTESIAN_POINT('',({},{},{}))", real(p[0]), real(p[1]), real(p[2])))
    }

    fn direction(&mut self, d: [f64; 3]) -> usize {
        self.add(format!("DIRECTION('',({},{},{}))", real(d[0]), real(d[1]), real(d[2])))
    }

    fn axis(&mut self, origin: [f64; 3], z: [f64; 3], x: [f64; 3]) -> usize {
        let (o, z, x) = (self.point(origin), self.direction(z), self.direction(x));
        self.add(format!("AXIS2_PLACEMENT_3D('',#{o},#{z},#{x})"))
    }

    fn identity(&mut self) -> usize {
        self.axis([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0])
    }

    /// PRODUCT ... PRODUCT_DEFINITION_SHAPE for an item: (definition, definition shape).
    fn product(&mut self, id: &str, name: &str, description: &str) -> (usize, usize) {
        let product = self.add(format!("PRODUCT({},{},{},(#{}))", string(id), string(name), string(description), self.product_context));
        self.add(format!("PRODUCT_RELATED_PRODUCT_CATEGORY('part',$,(#{product}))"));
        let formation = self.add(format!("PRODUCT_DEFINITION_FORMATION('','',#{product})"));
        let definition = self.add(format!("PRODUCT_DEFINITION('design','',#{formation},#{})", self.definition_context));
        let shape = self.add(format!("PRODUCT_DEFINITION_SHAPE('','',#{definition})"));
        (definition, shape)
    }

    /// A closed B-rep solid of `shape`, base on the origin of its product.
    fn solid(&mut self, shape: Shape) -> usize {
        match shape {
            Shape::Box { length, width, height } => self.brep_box(mm(length), mm(width), mm(height)),
            Shape::Cylinder { diameter, height } => self.brep_cylinder(mm(diameter) / 2.0, mm(height)),
        }
    }

    fn brep_box(&mut self, l: f64, w: f64, h: f64) -> usize {
        let corner = |i: usize| [if i & 1 == 1 { l } else { 0.0 }, if i & 2 == 2 { w } else { 0.0 }, if i & 4 == 4 { h } else { 0.0 }];
        let vertices: Vec<usize> = (0..8)
            .map(|i| {
                let p = self.point(corner(i));
                self.add(format!("VERTEX_POINT('',#{p})"))
            })
            .collect();
        // Faces: corners counter-clockwise seen from outside, and the outward normal.
        let faces: [([usize; 4], [f64; 3], [f64; 3]); 6] = [
            ([0, 2, 3, 1], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]),
            ([4, 5, 7, 6], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
            ([0, 1, 5, 4], [0.0, -1.0, 0.0], [1.0, 0.0, 0.0]),
            ([2, 6, 7, 3], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
            ([0, 4, 6, 2], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
            ([1, 3, 7, 5], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        ];
        let mut edges: HashMap<(usize, usize), usize> = HashMap::new();
        let mut face_ids = Vec::new();
        for (loop_corners, normal, reference) in faces {
            let mut oriented = Vec::new();
            for k in 0..4 {
                let (a, b) = (loop_corners[k], loop_corners[(k + 1) % 4]);
                let key = (a.min(b), a.max(b));
                let edge = match edges.get(&key) {
                    Some(edge) => *edge,
                    None => {
                        let (p, q) = (corner(key.0), corner(key.1));
                        let start = self.point(p);
                        let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
                        let length = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                        let dir = self.direction([d[0] / length, d[1] / length, d[2] / length]);
                        let vector = self.add(format!("VECTOR('',#{dir},{})", real(length)));
                        let line = self.add(format!("LINE('',#{start},#{vector})"));
                        let edge = self.add(format!("EDGE_CURVE('',#{},#{},#{line},.T.)", vertices[key.0], vertices[key.1]));
                        edges.insert(key, edge);
                        edge
                    }
                };
                let forward = if a < b { ".T." } else { ".F." };
                oriented.push(self.add(format!("ORIENTED_EDGE('',*,*,#{edge},{forward})")));
            }
            let edge_loop = self.add(format!("EDGE_LOOP('',({}))", refs(&oriented)));
            let bound = self.add(format!("FACE_OUTER_BOUND('',#{edge_loop},.T.)"));
            let placement = self.axis(corner(loop_corners[0]), normal, reference);
            let plane = self.add(format!("PLANE('',#{placement})"));
            face_ids.push(self.add(format!("ADVANCED_FACE('',(#{bound}),#{plane},.T.)")));
        }
        let shell = self.add(format!("CLOSED_SHELL('',({}))", refs(&face_ids)));
        self.add(format!("MANIFOLD_SOLID_BREP('',#{shell})"))
    }

    fn brep_cylinder(&mut self, r: f64, h: f64) -> usize {
        let (up, east) = ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        let bottom_axis = self.axis([0.0; 3], up, east);
        let top_axis = self.axis([0.0, 0.0, h], up, east);
        let v0 = {
            let p = self.point([r, 0.0, 0.0]);
            self.add(format!("VERTEX_POINT('',#{p})"))
        };
        let v1 = {
            let p = self.point([r, 0.0, h]);
            self.add(format!("VERTEX_POINT('',#{p})"))
        };
        let bottom_circle = self.add(format!("CIRCLE('',#{bottom_axis},{})", real(r)));
        let top_circle = self.add(format!("CIRCLE('',#{top_axis},{})", real(r)));
        let bottom = self.add(format!("EDGE_CURVE('',#{v0},#{v0},#{bottom_circle},.T.)"));
        let top = self.add(format!("EDGE_CURVE('',#{v1},#{v1},#{top_circle},.T.)"));
        let seam = {
            let start = self.point([r, 0.0, 0.0]);
            let dir = self.direction(up);
            let vector = self.add(format!("VECTOR('',#{dir},{})", real(h)));
            let line = self.add(format!("LINE('',#{start},#{vector})"));
            self.add(format!("EDGE_CURVE('',#{v0},#{v1},#{line},.T.)"))
        };
        let oriented = |w: &mut Writer, edge: usize, forward: bool| w.add(format!("ORIENTED_EDGE('',*,*,#{edge},{})", if forward { ".T." } else { ".F." }));
        // Side: around counter-clockwise at the bottom, up the seam, back at the top, down the seam.
        let side_edges = [oriented(self, bottom, true), oriented(self, seam, true), oriented(self, top, false), oriented(self, seam, false)];
        let side_loop = self.add(format!("EDGE_LOOP('',({}))", refs(&side_edges)));
        let side_bound = self.add(format!("FACE_OUTER_BOUND('',#{side_loop},.T.)"));
        let side_axis = self.axis([0.0; 3], up, east);
        let surface = self.add(format!("CYLINDRICAL_SURFACE('',#{side_axis},{})", real(r)));
        let side = self.add(format!("ADVANCED_FACE('',(#{side_bound}),#{surface},.T.)"));
        // Top: its circle counter-clockwise seen from above, the outward side.
        let top_edge = oriented(self, top, true);
        let top_loop = self.add(format!("EDGE_LOOP('',(#{top_edge}))"));
        let top_bound = self.add(format!("FACE_OUTER_BOUND('',#{top_loop},.T.)"));
        let top_plane_axis = self.axis([0.0, 0.0, h], up, east);
        let top_plane = self.add(format!("PLANE('',#{top_plane_axis})"));
        let top_face = self.add(format!("ADVANCED_FACE('',(#{top_bound}),#{top_plane},.T.)"));
        // Bottom: the plane faces up, the face down; its circle runs the other way.
        let bottom_edge = oriented(self, bottom, false);
        let bottom_loop = self.add(format!("EDGE_LOOP('',(#{bottom_edge}))"));
        let bottom_bound = self.add(format!("FACE_OUTER_BOUND('',#{bottom_loop},.T.)"));
        let bottom_plane_axis = self.axis([0.0; 3], up, east);
        let bottom_plane = self.add(format!("PLANE('',#{bottom_plane_axis})"));
        let bottom_face = self.add(format!("ADVANCED_FACE('',(#{bottom_bound}),#{bottom_plane},.F.)"));
        let shell = self.add(format!("CLOSED_SHELL('',(#{side},#{top_face},#{bottom_face}))"));
        self.add(format!("MANIFOLD_SOLID_BREP('',#{shell})"))
    }

    /// Place `child` in `parent`: the assembly link and its transformation.
    fn link(&mut self, parent: (usize, usize), child: (usize, usize, usize), placement: usize, id: usize, name: &str) {
        let (parent_definition, parent_representation) = parent;
        let (child_definition, child_representation, child_origin) = child;
        let usage = self.add(format!("NEXT_ASSEMBLY_USAGE_OCCURRENCE('{id}',{},'',#{parent_definition},#{child_definition},$)", string(name)));
        let shape = self.add(format!("PRODUCT_DEFINITION_SHAPE('Placement','Placement of an item',#{usage})"));
        let transformation = self.add(format!("ITEM_DEFINED_TRANSFORMATION('','',#{child_origin},#{placement})"));
        let relationship = self.add(format!(
            "( REPRESENTATION_RELATIONSHIP('','',#{child_representation},#{parent_representation}) REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#{transformation}) SHAPE_REPRESENTATION_RELATIONSHIP() )"
        ));
        self.add(format!("CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#{relationship},#{shape})"));
    }

    /// An item and everything below it: (definition, representation, origin).
    fn item(&mut self, item: &Item, usages: &mut usize) -> (usize, usize, usize) {
        let (definition, definition_shape) = self.product(&item.id, &item.name, &item.description);
        let origin = self.identity();
        if item.children.is_empty() {
            let solid = self.solid(item.shape.expect("a kept leaf has a shape"));
            let representation = self.add(format!("ADVANCED_BREP_SHAPE_REPRESENTATION({},(#{origin},#{solid}),#{})", string(&item.name), self.context));
            self.add(format!("SHAPE_DEFINITION_REPRESENTATION(#{definition_shape},#{representation})"));
            return (definition, representation, origin);
        }
        // An assembly: its own body (when it has a shape) and its children, each placed.
        let mut placed: Vec<(Item, usize)> = Vec::new();
        let mut items = vec![origin];
        if let Some(shape) = item.shape {
            let body = Item { id: format!("{} body", item.id), name: format!("{} (body)", item.name), description: String::new(), shape: Some(shape), offset: Offset::default(), rotation: Rotation::default(), location: None, placed: true, children: Vec::new() };
            let axis = self.identity();
            items.push(axis);
            placed.push((body, axis));
        }
        for child in &item.children {
            let (x, z) = axes(child.rotation);
            let axis = self.axis([mm(child.offset.x), mm(child.offset.y), mm(child.offset.z)], z, x);
            items.push(axis);
            placed.push((child.clone(), axis));
        }
        let representation = self.add(format!("SHAPE_REPRESENTATION({},({}),#{})", string(&item.name), refs(&items), self.context));
        self.add(format!("SHAPE_DEFINITION_REPRESENTATION(#{definition_shape},#{representation})"));
        for (child, axis) in placed {
            let built = self.item(&child, usages);
            *usages += 1;
            self.link((definition, representation), built, axis, *usages, &child.name);
        }
        (definition, representation, origin)
    }
}

fn refs(ids: &[usize]) -> String {
    ids.iter().map(|id| format!("#{id}")).collect::<Vec<_>>().join(",")
}

/// What an export wrote, for the reader.
#[derive(Debug, Clone, PartialEq)]
pub struct Exported {
    pub text: String,
    pub solids: usize,
    pub notes: Vec<String>,
}

/// The STEP AP242 file of a model, stamped `timestamp` (ISO 8601). An error
/// when no element has a shape: there would be nothing to give CAD.
pub fn export(ast: &Model, timestamp: &str) -> Result<Exported, String> {
    let mut levels = structure(ast);
    if levels.is_empty() {
        return Err("no element has a `shape`: there is no geometry to export to CAD (give a node, a behavior component or an EPBS item `shape: { box: [l, w, h] }`)".to_string());
    }
    let mut notes = Vec::new();
    if let Some(origin) = place_sites(&mut levels) {
        notes.push(format!("located elements without a position are placed by their distance from {:.5}°, {:.5}° (east = x, north = y, up = z)", origin.lat, origin.lon));
    }
    let name = ast.attributes.get("name").and_then(AttributeValue::as_string).unwrap_or(&ast.id).to_string();
    let mut w = Writer { lines: Vec::new(), context: 0, product_context: 0, definition_context: 0 };
    let application = w.add("APPLICATION_CONTEXT('managed model based 3d engineering')".to_string());
    w.add(format!("APPLICATION_PROTOCOL_DEFINITION('international standard','ap242_managed_model_based_3d_engineering',2014,#{application})"));
    w.product_context = w.add(format!("PRODUCT_CONTEXT('',#{application},'mechanical')"));
    w.definition_context = w.add(format!("PRODUCT_DEFINITION_CONTEXT('part definition',#{application},'design')"));
    let length = w.add("( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) )".to_string());
    let angle = w.add("( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) )".to_string());
    let solid_angle = w.add("( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() )".to_string());
    let uncertainty = w.add(format!("UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-07),#{length},'distance_accuracy_value','confusion accuracy')"));
    w.context = w.add(format!(
        "( GEOMETRIC_REPRESENTATION_CONTEXT(3) GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#{uncertainty})) GLOBAL_UNIT_ASSIGNED_CONTEXT((#{length},#{angle},#{solid_angle})) REPRESENTATION_CONTEXT('3D','ArcLang model') )"
    ));
    let root = Item::group(&ast.id, &name, levels);
    let solids = root.solids();
    let mut usages = 0;
    w.item(&root, &mut usages);

    let mut text = String::from("ISO-10303-21;\nHEADER;\n");
    text.push_str(&format!("FILE_DESCRIPTION(({}),'2;1');\n", string(&format!("ArcLang model {name}"))));
    text.push_str(&format!(
        "FILE_NAME({},{},(''),(''),{},'ArcLang','');\n",
        string(&format!("{name}.step")),
        string(timestamp),
        string(&format!("ArcLang {}", env!("CARGO_PKG_VERSION")))
    ));
    text.push_str("FILE_SCHEMA(('AP242_MANAGED_MODEL_BASED_3D_ENGINEERING_MIM_LF { 1 0 10303 442 1 1 4 }'));\nENDSEC;\nDATA;\n");
    for (i, line) in w.lines.iter().enumerate() {
        text.push_str(&format!("#{} = {line};\n", i + 1));
    }
    text.push_str("ENDSEC;\nEND-ISO-10303-21;\n");
    Ok(Exported { text, solids, notes })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Compiler, CompilerConfig};
    use std::collections::HashSet;

    fn exported(body: &str) -> Result<Exported, String> {
        let model = format!("model Site {{ name: \"Site\" }}\n{body}");
        export(&Compiler::new(CompilerConfig::default()).compile_string(&model).expect("compiles").ast, "2026-10-20T00:00:00")
    }

    /// Every `#n` referenced is defined, once, and parentheses balance.
    fn well_formed(text: &str) {
        let data = text.split("DATA;\n").nth(1).unwrap().split("ENDSEC;").next().unwrap();
        let mut defined = HashSet::new();
        for line in data.lines() {
            let (id, body) = line.split_once(" = ").unwrap_or_else(|| panic!("{line}"));
            assert!(defined.insert(id.to_string()), "{id} twice");
            assert!(body.ends_with(';'), "{line}");
            assert_eq!(body.matches('(').count(), body.matches(')').count(), "{line}");
        }
        for reference in data.split('#').skip(1) {
            let id: String = reference.chars().take_while(char::is_ascii_digit).collect();
            if !id.is_empty() {
                assert!(defined.contains(&format!("#{id}")), "#{id} used, not defined");
            }
        }
    }

    #[test]
    fn a_node_with_a_shape_and_a_placed_component_is_an_assembly_of_two_solids() {
        let out = exported("physical_architecture \"PA\" {\n  node \"Cabinet\" {\n    id: \"PN-CAB\"\n    shape: { box: [2 m, 1 m, 0.6 m] }\n    behavior_component \"Controller\" { id: \"PB-CTL\" position: { x: 0.5 m y: 0.2 m z: 0.1 m } orientation: { yaw: 90 deg } shape: { cylinder: [200 mm, 0.4 m] } }\n  }\n  node \"Nothing\" { id: \"PN-N\" }\n}\n").unwrap();
        well_formed(&out.text);
        assert_eq!(out.solids, 2);
        assert!(out.text.contains("FILE_SCHEMA(('AP242_MANAGED_MODEL_BASED_3D_ENGINEERING_MIM_LF"));
        assert!(out.text.contains("PRODUCT('PN-CAB','Cabinet'"));
        assert!(out.text.contains("PRODUCT('PB-CTL','Controller'"));
        assert!(!out.text.contains("'PN-N'"), "nothing without geometry is exported");
        assert_eq!(out.text.matches("MANIFOLD_SOLID_BREP").count(), 2);
        assert!(out.text.contains("CYLINDRICAL_SURFACE('',") && out.text.contains(",100.)"), "radius in mm");
        assert!(out.text.contains("CARTESIAN_POINT('',(500.,200.,100.))"), "the component placed in mm");
        assert!(out.text.contains("DIRECTION('',(0.,1.,0.))"), "turned 90° about z: x becomes y");
    }

    #[test]
    fn located_nodes_are_laid_out_by_their_distance() {
        let out = exported("physical_architecture \"PA\" {\n  node \"A\" { id: \"PN-A\" location: { lat: 45 deg lon: 5 deg } shape: { box: [1 m, 1 m, 1 m] } }\n  node \"B\" { id: \"PN-B\" location: { lat: 45 deg lon: 5.001 deg } shape: { box: [1 m, 1 m, 1 m] } }\n}\n").unwrap();
        well_formed(&out.text);
        // 0.001° of longitude at 45° north is 78.63 m east.
        assert!(out.text.contains("CARTESIAN_POINT('',(78626.795"), "{}", out.text.lines().filter(|l| l.contains("CARTESIAN_POINT")).take(12).collect::<Vec<_>>().join("\n"));
        assert_eq!(out.notes.len(), 1);
    }

    #[test]
    fn epbs_items_are_products_and_text_is_iso_encoded() {
        let out = exported("epbs \"Product\" {\n  system \"Borne\" {\n    subsystem \"Électronique\" {\n      item \"Carte d'alimentation\" { shape: { box: [200 mm, 150 mm, 20 mm] } }\n    }\n  }\n}\n").unwrap();
        well_formed(&out.text);
        assert_eq!(out.solids, 1);
        assert!(out.text.contains("'Carte d''alimentation'"));
        assert!(out.text.contains("'\\X2\\00C9\\X0\\lectronique'"));
    }

    #[test]
    fn a_model_without_shapes_is_refused_with_the_reason() {
        assert!(exported("physical_architecture \"PA\" { node \"A\" { id: \"PN-A\" } }\n").unwrap_err().contains("no element has a `shape`"));
    }

    #[test]
    fn reals_always_carry_their_point() {
        assert_eq!(real(1.0), "1.");
        assert_eq!(real(-0.0000000000001), "0.");
        assert_eq!(real(2.5), "2.5");
    }
}
