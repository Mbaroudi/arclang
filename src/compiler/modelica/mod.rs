//! `arclang export -f modelica`: the physical architecture as a Modelica
//! package that OpenModelica loads and checks.
//!
//! The generated package holds the STRUCTURE, never the physics:
//!
//! - `Behavior.<Node>.<Component>`: a `partial model` per behavior
//!   component, with its connectors and parameters;
//! - `Physical.<Node>`: a `partial model` per physical node, with its
//!   connectors, its parameters, its behavior components as `replaceable`
//!   instances and the `connect` equations between them;
//! - `Systems.<Configuration>`: a `partial model` per configuration (one
//!   `System` without any) instantiating the nodes the configuration keeps,
//!   each `replaceable`, with the parameters its variants set in the
//!   `constrainedby` clause, and the `connect` equations of the links.
//!
//! The physics is hand-written in a package of its own that `extends` these
//! models and redeclares their parts, so regenerating never overwrites it.
//! Every class, component, connector and connection carries the ArcLang id
//! it comes from in a vendor annotation, `__ArcLang(id="...")`.

pub mod connector;
pub mod names;
pub mod placement;
pub mod structure;
pub mod units;

use crate::compiler::ast::Model;
use crate::compiler::diagram::layout_file::LayoutFile;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use structure::{Connection, Part, Structure};

/// The Modelica Standard Library version the package `uses`.
pub const MSL_VERSION: &str = "4.1.0";

#[derive(Debug, Clone, Default)]
pub struct Export {
    /// The package name (`EVRangeSimulation`).
    pub package: String,
    /// `<package>.mo`.
    pub file_name: String,
    pub text: String,
    /// The same package, one file per class, as (path relative to the output
    /// directory, text): the structured layout `--split` writes.
    pub files: Vec<(String, String)>,
    /// What was not exported, and why.
    pub notes: Vec<String>,
    /// ArcLang id (and name) of each part -> its path in a system model
    /// (`PN-BATT` -> `batteryPack`, `BC-SPD` -> `vehicleControlUnit.speedController`).
    pub instances: BTreeMap<String, String>,
    /// ArcLang id of each part -> its path (the ids alone, without the names of `instances`).
    pub parts: BTreeMap<String, String>,
    /// Configuration id (empty without configurations) -> qualified system model.
    pub systems: BTreeMap<String, String>,
}

/// The name of the model, for the package.
pub fn model_name(ast: &Model) -> String {
    ast.attributes.get("name").and_then(|v| v.as_string()).map(str::to_string).filter(|n| !n.is_empty()).unwrap_or_else(|| if ast.id.is_empty() { "ArcLangModel".to_string() } else { ast.id.clone() })
}

/// Connections between ports that declare physics must join compatible
/// ends: checked on every compilation, not only on export.
pub fn check(ast: &Model) -> Result<(), String> {
    if ast.physical_architecture.is_empty() {
        return Ok(());
    }
    structure::read(ast).map(|_| ()).map_err(|e| format!("modelica: {e}"))
}

/// What the package is drawn with. The default draws nothing: the
/// structure only, which is what the simulation hashes and runs.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Write `Placement`, `Line`, `Icon` and `Diagram` annotations, so a
    /// graphical editor (Modex, OMEdit) draws the architecture.
    pub placement: bool,
    /// The workbench layout (`<model>.layout.json`), moving boxes from where
    /// the automatic layout puts them. Only read with `placement`.
    pub layout: Option<LayoutFile>,
}

/// The structure only, with no drawing: what `arclang verify` simulates.
pub fn export(ast: &Model) -> Result<Export, String> {
    export_with(ast, &Options::default())
}

pub fn export_with(ast: &Model, options: &Options) -> Result<Export, String> {
    if ast.physical_architecture.iter().all(|pa| pa.nodes.is_empty()) {
        return Err("Modelica export: the model has no physical node to export".to_string());
    }
    let structure = structure::read(ast)?;
    let package = names::class_name(&model_name(ast));
    let mut writer = Writer { package: &package, structure: &structure, ast, options, sides: placement::Sides::of(&structure.parts, &structure.connections), plans: Vec::new() };
    if options.placement {
        writer.plans = writer.draw_parts();
    }
    let text = writer.package();
    let files = writer.tree();
    let mut instances = BTreeMap::new();
    let mut by_name = Vec::new();
    for part in &structure.parts {
        let path = match part.node {
            Some(node) => format!("{}.{}", structure.parts[node].instance, part.instance),
            None => part.instance.clone(),
        };
        instances.insert(part.id.clone(), path.clone());
        by_name.push((part.name.clone(), path));
    }
    let parts = instances.clone();
    for (name, path) in by_name {
        instances.entry(name).or_insert(path);
    }
    let systems = structure.systems.iter().map(|s| (s.id.clone(), format!("{package}.Systems.{}", s.class))).collect();
    Ok(Export { file_name: format!("{package}.mo"), package, text, files, notes: structure.notes.clone(), instances, parts, systems })
}

struct Writer<'a> {
    package: &'a str,
    structure: &'a Structure,
    ast: &'a Model,
    options: &'a Options,
    sides: placement::Sides,
    /// The drawing of each part's own model (empty without placement).
    plans: Vec<placement::Plan>,
}

fn annotation(id: &str) -> String {
    format!("annotation(__ArcLang(id=\"{}\"))", names::string(id))
}

/// `drawing` is the `Icon(...)` and `Diagram(...)` of the model, with a
/// trailing `, ` when not empty.
fn documentation(paragraphs: &[String], id: &str, kind: &str, drawing: &str) -> String {
    let html: String = paragraphs.iter().map(|p| format!("<p>{}</p>", names::html(p))).collect();
    format!("annotation({drawing}Documentation(info=\"<html>{html}</html>\"), __ArcLang(id=\"{}\", kind=\"{kind}\"));", names::string(id))
}

/// `Placement(transformation(...))` of a component centered on (x, y).
fn component_placement(placed: &placement::Placed) -> String {
    let (hw, hh) = (placed.w / 2.0, placed.h / 2.0);
    format!(
        "Placement(transformation(origin={{{},{}}}, extent={{{{{},{}}},{{{},{}}}}}))",
        placement::num(placed.x), placement::num(placed.y),
        placement::num(-hw), placement::num(-hh), placement::num(hw), placement::num(hh)
    )
}

/// A connector of a model, on the border of its diagram. The icon of the
/// model is its diagram, so there is no `iconTransformation`: the connector
/// sits at the same place on both.
fn port_annotation(at: (f64, f64), half: f64, id: &str) -> String {
    let r = half;
    format!(
        "annotation(Placement(transformation(origin={{{},{}}}, extent={{{{{},{}}},{{{},{}}}}})), __ArcLang(id=\"{}\"))",
        placement::num(at.0), placement::num(at.1), placement::num(-r), placement::num(-r), placement::num(r), placement::num(r),
        names::string(id)
    )
}

fn diagram(extent: [f64; 4]) -> String {
    let [l, b, r, t] = extent;
    format!("Diagram(coordinateSystem(extent={{{{{},{}}},{{{},{}}}}}))", l.floor(), b.floor(), r.ceil(), t.ceil())
}

/// A rectangle with the name of the model, over the extent of its diagram:
/// the box in a parent diagram.
fn icon(name: &str, extent: [f64; 4]) -> String {
    let [l, b, r, t] = extent;
    let (l, b, r, t) = (l.floor(), b.floor(), r.ceil(), t.ceil());
    let (w, h) = (r - l, t - b);
    let (cx, cy) = ((l + r) / 2.0, (b + t) / 2.0);
    let rectangle = format!("{{{{{},{}}},{{{},{}}}}}", l, b, r, t);
    let label = format!("{{{{{},{}}},{{{},{}}}}}", placement::num(l + w * 0.27), placement::num(cy - h * 0.2), placement::num(r - w * 0.27), placement::num(cy + h * 0.2));
    let instance = format!("{{{{{},{}}},{{{},{}}}}}", l, placement::num(t + 2.0), r, placement::num(t + h * 0.25));
    let _ = cx;
    format!(
        "Icon(coordinateSystem(extent={rectangle}), graphics={{Rectangle(extent={rectangle}, lineColor={{0,0,0}}, fillColor={{255,255,255}}, fillPattern=FillPattern.Solid), Text(extent={label}, textString=\"{}\", textColor={{0,0,0}}), Text(extent={instance}, textString=\"%name\", textColor={{0,0,255}})}})",
        names::string(name)
    )
}

/// The first words of every file the export writes: how a file is known
/// to be ArcLang's, hence safe to replace.
pub const GENERATED_HEADER: &str = "// Generated by ArcLang (arclang export -f modelica";

impl Writer<'_> {
    fn package(&self) -> String {
        let mut out = String::new();
        let name = model_name(self.ast);
        let _ = writeln!(out, "// Generated by ArcLang (arclang export -f modelica). Do not edit: regenerate it.");
        let _ = writeln!(out, "// The physics lives in a package of its own that extends these models.");
        let _ = writeln!(out, "within ;");
        let _ = writeln!(out, "package {} \"{}\"", self.package, names::string(&name));
        let behaviors: Vec<&Part> = self.structure.parts.iter().filter(|p| p.node.is_none() && !p.behaviors.is_empty()).collect();
        if !behaviors.is_empty() {
            let _ = writeln!(out, "  package Behavior \"Behavior components, by the node they are deployed on\"");
            for node in behaviors {
                let _ = writeln!(out, "    package {} \"Behavior components of {}\"", node.class, names::string(&node.name));
                for &child in &node.behaviors {
                    self.part_model(&mut out, &self.structure.parts[child], "      ", "BehaviorComponent");
                }
                let _ = writeln!(out, "    end {};", node.class);
            }
            let _ = writeln!(out, "  end Behavior;");
        }
        let _ = writeln!(out, "  package Physical \"Physical nodes\"");
        for part in self.structure.parts.iter().filter(|p| p.node.is_none()) {
            self.part_model(&mut out, part, "    ", "PhysicalNode");
        }
        let _ = writeln!(out, "  end Physical;");
        self.systems(&mut out);
        let _ = writeln!(out, "  {}", self.package_annotation());
        let _ = writeln!(out, "end {};", self.package);
        out
    }

    fn part_model(&self, out: &mut String, part: &Part, indent: &str, kind: &str) {
        let _ = writeln!(out, "{indent}partial model {} \"{}\"", part.class, names::string(&part.name));
        for param in &part.params {
            let declaration = units::declaration(&param.ty, &param.name);
            let value = param.value.map(|v| format!(" = {}", names::real(v))).unwrap_or_default();
            let comment = if param.comment.is_empty() { String::new() } else { format!(" \"{}\"", names::string(&param.comment)) };
            let _ = writeln!(out, "{indent}  parameter {declaration}{value}{comment};");
        }
        let index = self.structure.parts.iter().position(|p| p.id == part.id && p.node == part.node);
        let connections: Vec<&Connection> = self.structure.connections.iter().filter(|c| c.within.is_some() && c.within == index).collect();
        let drawn = self.options.placement.then(|| &self.plans[index.unwrap_or(0)]);
        for (number, port) in part.ports.iter().enumerate() {
            let note = match &drawn {
                Some(plan) => port_annotation(plan.own_ports.get(&number).copied().unwrap_or((0.0, 0.0)), plan.port_half, &port.id),
                None => annotation(&port.id),
            };
            let _ = writeln!(out, "{indent}  {} {} \"{} port {}\" {note};", port.connector, port.member, port.physics.label(), names::string(&port.name));
        }
        for &child in &part.behaviors {
            let child_part = &self.structure.parts[child];
            let class = format!("{}.Behavior.{}.{}", self.package, part.class, child_part.class);
            let note = self.instance_annotation(drawn, child, &child_part.id);
            let _ = writeln!(out, "{indent}  replaceable {class} {} constrainedby {class} \"{}\" {note};", child_part.instance, names::string(&child_part.name));
        }
        if !connections.is_empty() {
            let _ = writeln!(out, "{indent}equation");
            for (number, connection) in connections.iter().enumerate() {
                let ends: Vec<String> = connection.ends.iter().map(|&(p, port)| self.reference_within(index, p, port)).collect();
                let note = self.connect_annotation(drawn, number, connection);
                let _ = writeln!(out, "{indent}  connect({}, {}) {note};", ends[0], ends[1]);
            }
        }
        let mut info: Vec<String> = part.description.iter().cloned().collect();
        info.push(format!("{} '{}' ({}).", if kind == "PhysicalNode" { "Physical node" } else { "Behavior component" }, part.name, part.id));
        if !part.realizes.is_empty() {
            info.push(format!("Realizes {}.", part.realizes.join(", ")));
        }
        let drawing = drawn.map(|plan| format!("{}, {}, ", icon(&part.name, plan.extent), diagram(plan.extent))).unwrap_or_default();
        let _ = writeln!(out, "{indent}  {}", documentation(&info, &part.id, kind, &drawing));
        let _ = writeln!(out, "{indent}end {};", part.class);
    }

    /// The drawing of the model of every part, behavior components first:
    /// a node's boxes are sized by theirs.
    fn draw_parts(&self) -> Vec<placement::Plan> {
        let parts = &self.structure.parts;
        let mut plans: Vec<placement::Plan> = (0..parts.len()).map(|_| placement::Plan::default()).collect();
        let order: Vec<usize> = (0..parts.len()).filter(|&i| parts[i].node.is_some()).chain((0..parts.len()).filter(|&i| parts[i].node.is_none())).collect();
        for index in order {
            let connections: Vec<&Connection> = self.structure.connections.iter().filter(|c| c.within == Some(index)).collect();
            let moves = self.moves(Some(&parts[index].id));
            plans[index] = placement::plan(parts, &self.sides, &plans, Some(index), &parts[index].behaviors, &connections, &moves);
        }
        plans
    }

    /// The offsets the layout file gives to the boxes of a model (`open`:
    /// the node whose model it is, `None` for a system model).
    fn moves(&self, open: Option<&str>) -> BTreeMap<usize, (f64, f64)> {
        self.options.layout.as_ref().map(|layout| placement::offsets(layout, open, &self.structure.parts)).unwrap_or_default()
    }

    /// `annotation(Placement(...), __ArcLang(id=...))` of an instance.
    fn instance_annotation(&self, drawn: Option<&placement::Plan>, part: usize, id: &str) -> String {
        match drawn.and_then(|plan| plan.boxes.get(&part)) {
            Some(placed) => format!("annotation({}, __ArcLang(id=\"{}\"))", component_placement(placed), names::string(id)),
            None => annotation(id),
        }
    }

    fn connect_annotation(&self, drawn: Option<&placement::Plan>, number: usize, connection: &Connection) -> String {
        let id = names::string(&connection.id);
        let Some(points) = drawn.and_then(|plan| plan.wires.get(&number)) else { return annotation(&connection.id) };
        let (part, port) = connection.ends[0];
        let color = placement::color(self.structure.parts[part].ports[port].physics);
        let points: Vec<String> = points.iter().map(|&(x, y)| format!("{{{},{}}}", placement::num(x), placement::num(y))).collect();
        format!("annotation(Line(points={{{}}}, color={color}), __ArcLang(id=\"{id}\"))", points.join(","))
    }

    fn reference_within(&self, holder: Option<usize>, part: usize, port: usize) -> String {
        let member = &self.structure.parts[part].ports[port].member;
        if Some(part) == holder {
            member.clone()
        } else {
            format!("{}.{member}", self.structure.parts[part].instance)
        }
    }

    fn systems(&self, out: &mut String) {
        let _ = writeln!(out, "  package Systems \"One system model per configuration\"");
        for system in &self.structure.systems {
            self.system_model(out, system, "    ");
        }
        let _ = writeln!(out, "  end Systems;");
    }

    fn system_model(&self, out: &mut String, system: &structure::System, indent: &str) {
        let comment = if system.id.is_empty() { "The whole physical architecture".to_string() } else { format!("Configuration {} ({})", system.name, system.id) };
        let _ = writeln!(out, "{indent}partial model {} \"{}\"", system.class, names::string(&comment));
        let kept: Vec<usize> = system.nodes.iter().map(|(i, _)| *i).collect();
        let connections: Vec<&Connection> = self.structure.connections.iter().filter(|c| c.within.is_none() && c.ends.iter().all(|(p, _)| kept.contains(p))).collect();
        let drawn_owned = self.options.placement.then(|| placement::plan(&self.structure.parts, &self.sides, &self.plans, None, &kept, &connections, &self.moves(None)));
        let drawn = drawn_owned.as_ref();
        for (index, modifiers) in &system.nodes {
            let part = &self.structure.parts[*index];
            let class = format!("{}.Physical.{}", self.package, part.class);
            let modifiers = if modifiers.is_empty() { String::new() } else { format!("({})", modifiers.join(", ")) };
            let note = self.instance_annotation(drawn, *index, &part.id);
            let _ = writeln!(out, "{indent}  replaceable {class} {} constrainedby {class}{modifiers} \"{}\" {note};", part.instance, names::string(&part.name));
        }
        if !connections.is_empty() {
            let _ = writeln!(out, "{indent}equation");
            for (number, connection) in connections.iter().enumerate() {
                let ends: Vec<String> = connection.ends.iter().map(|&(p, port)| self.reference_within(None, p, port)).collect();
                let note = self.connect_annotation(drawn, number, connection);
                let _ = writeln!(out, "{indent}  connect({}, {}) {note};", ends[0], ends[1]);
            }
        }
        let mut info = vec![comment.clone()];
        if !system.selects.is_empty() {
            info.push(format!("Selects {}.", system.selects.join(", ")));
        }
        info.push("Extend it, redeclaring each part with the model of its physics, to simulate it.".to_string());
        let drawing = drawn.map(|plan| format!("{}, ", diagram(plan.extent))).unwrap_or_default();
        let id = if system.id.is_empty() { "system".to_string() } else { system.id.clone() };
        let _ = writeln!(out, "{indent}  {}", documentation(&info, &id, "Configuration", &drawing));
        let _ = writeln!(out, "{indent}end {};", system.class);
    }

    /// The package annotation: the library it uses and its description.
    fn package_annotation(&self) -> String {
        let name = model_name(self.ast);
        let mut info = Vec::new();
        if let Some(description) = self.ast.attributes.get("description") {
            info.push(description.display());
        }
        info.push("Generated by ArcLang from the physical architecture: the structure only. Every model is partial: a hand-written package extends them with the physics and redeclares their parts, so regenerating never overwrites it.".to_string());
        let html: String = info.iter().map(|p| format!("<p>{}</p>", names::html(p))).collect();
        format!("annotation(uses(Modelica(version=\"{MSL_VERSION}\")), Documentation(info=\"<html>{html}</html>\"), __ArcLang(name=\"{}\"));", names::string(&name))
    }

    /// The same package as one file per class, in the structured layout of a
    /// Modelica library (`<Package>/package.mo`, a folder per sub-package).
    /// Modex browses a package by its folders, and OpenModelica loads the
    /// folder as it loads the file.
    fn tree(&self) -> Vec<(String, String)> {
        let header = "// Generated by ArcLang (arclang export -f modelica --split). Do not edit: regenerate it.\n";
        let root = &self.package;
        let mut files: Vec<(String, String)> = Vec::new();
        let package_file = |path: &str, within: &str, name: &str, comment: &str, order: &[String], annotation: &str| -> Vec<(String, String)> {
            let mut text = format!("{header}within {within};\npackage {name} \"{}\"\n", names::string(comment));
            if !annotation.is_empty() {
                let _ = writeln!(text, "  {annotation}");
            }
            let _ = writeln!(text, "end {name};");
            vec![(format!("{path}/package.mo"), text), (format!("{path}/package.order"), order.iter().map(|n| format!("{n}\n")).collect())]
        };
        let nodes: Vec<usize> = (0..self.structure.parts.len()).filter(|&i| self.structure.parts[i].node.is_none()).collect();
        let with_behaviours: Vec<usize> = nodes.iter().copied().filter(|&i| !self.structure.parts[i].behaviors.is_empty()).collect();
        let mut top = Vec::new();
        if !with_behaviours.is_empty() {
            top.push("Behavior".to_string());
        }
        top.extend(["Physical".to_string(), "Systems".to_string()]);
        files.extend(package_file(root, "", root, &model_name(self.ast), &top, &self.package_annotation()));
        if !with_behaviours.is_empty() {
            let order: Vec<String> = with_behaviours.iter().map(|&i| self.structure.parts[i].class.clone()).collect();
            files.extend(package_file(&format!("{root}/Behavior"), root, "Behavior", "Behavior components, by the node they are deployed on", &order, ""));
            for &i in &with_behaviours {
                let node = &self.structure.parts[i];
                let dir = format!("{root}/Behavior/{}", node.class);
                let within = format!("{root}.Behavior");
                let order: Vec<String> = node.behaviors.iter().map(|&c| self.structure.parts[c].class.clone()).collect();
                files.extend(package_file(&dir, &within, &node.class, &format!("Behavior components of {}", node.name), &order, ""));
                for &child in &node.behaviors {
                    let part = &self.structure.parts[child];
                    let mut text = format!("{header}within {within}.{};\n", node.class);
                    self.part_model(&mut text, part, "", "BehaviorComponent");
                    files.push((format!("{dir}/{}.mo", part.class), text));
                }
            }
        }
        let order: Vec<String> = nodes.iter().map(|&i| self.structure.parts[i].class.clone()).collect();
        files.extend(package_file(&format!("{root}/Physical"), root, "Physical", "Physical nodes", &order, ""));
        for &i in &nodes {
            let part = &self.structure.parts[i];
            let mut text = format!("{header}within {root}.Physical;\n");
            self.part_model(&mut text, part, "", "PhysicalNode");
            files.push((format!("{root}/Physical/{}.mo", part.class), text));
        }
        let order: Vec<String> = self.structure.systems.iter().map(|s| s.class.clone()).collect();
        files.extend(package_file(&format!("{root}/Systems"), root, "Systems", "One system model per configuration", &order, ""));
        for system in &self.structure.systems {
            let mut text = format!("{header}within {root}.Systems;\n");
            self.system_model(&mut text, system, "");
            files.push((format!("{root}/Systems/{}.mo", system.class), text));
        }
        files
    }
}
