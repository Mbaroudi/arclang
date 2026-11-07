//! The MATLAB script of a pivot: what the connector's toolbox runs when it
//! imports an `.mlcore` (`CapSLBridge.import`). It makes the Simulink
//! project (`Cap2SL2.sltx`), fills the data dictionaries, builds each model
//! (blocks, ports, lines, Stateflow charts) and saves it in the project, the
//! Capella id of each element in its `Tag` or `Description`.
//!
//! A port of the connector's generator (`ProjectGenerator`,
//! `DictionaryGenerator`, `SystemGenerator`, `StateFlowGenerator`): the
//! statements and their order are its own, to the character, Java's hash
//! order included. What fails says what the connector says.

use super::java;
use super::pivot::{self, Kind, Pivot, Value};

const BOUNDS: &str = "bounds";
const SOURCE_END: &str = "position.source";
const DESTINATION_END: &str = "position.destination";
const LAYER: &str = "capella.layer";
const ROOT_OF_LAYER: &str = "capella.layer.root";
/// The parameters of the connector itself, never passed to MATLAB.
const BRIDGE_KEYS: [&str; 4] = [BOUNDS, SOURCE_END, DESTINATION_END, LAYER];

/// The script of the document, or what the connector says when it cannot write it.
pub fn script(pivot: &Pivot) -> Result<String, String> {
    let errors = pivot::unresolved(pivot);
    if !errors.is_empty() {
        return Err(std::iter::once("Model validation failed:".to_string()).chain(errors).collect::<Vec<_>>().join("\n"));
    }
    let mut generator = Generator { p: pivot.clone(), sb: String::new(), visited: Vec::new(), used: Vec::new() };
    for root in pivot.roots.clone() {
        match generator.p[root].kind {
            Kind::Project => generator.project(root)?,
            Kind::Dictionary => generator.project_dictionary(root)?,
            Kind::System => generator.project_system(root)?,
            _ => {}
        }
    }
    Ok(generator.sb)
}

struct Generator {
    /// The pivot, which the generation completes (a model reference's `ModelFile`).
    p: Pivot,
    sb: String,
    visited: Vec<usize>,
    /// The variables put in the base workspace, as a Java `HashSet` keeps them.
    used: Vec<String>,
}

fn quote(text: &str) -> String {
    format!("'{text}'")
}

/// Java's `"" + value` of a string that may be null.
fn text(value: &Option<String>) -> String {
    value.clone().unwrap_or_else(|| "null".to_string())
}

fn empty(value: &Option<String>) -> bool {
    value.as_deref().is_none_or(str::is_empty)
}

fn primitive(value: &Value) -> Option<String> {
    match value {
        Value::Primitive(spec) => spec.clone(),
        _ => None,
    }
}

/// `PropertyHelper.toString(MultipleValue)`: `[a b c]`, `?` for what is not a scalar.
fn multiple(values: &[Value]) -> String {
    let inner: Vec<String> = values.iter().map(|v| match v {
        Value::Primitive(spec) => text(spec),
        _ => "?".to_string(),
    }).collect();
    format!("[{}]", inner.join(" "))
}

impl Generator {
    fn say(&mut self, statement: impl AsRef<str>) {
        self.sb.push_str(statement.as_ref());
        self.sb.push('\n');
    }

    // ---- Names ----------------------------------------------------------------

    fn layer(&self, i: usize) -> Option<String> {
        self.p[i].param(LAYER).and_then(primitive)
    }

    fn layer_root(&self, i: usize) -> bool {
        matches!(self.p[i].param(ROOT_OF_LAYER).and_then(primitive).as_deref(), Some("True" | "true"))
    }

    fn prefixed(&self, i: usize) -> Result<String, String> {
        let name = java::normalize(self.p[i].name.as_deref());
        let Some(layer) = self.layer(i) else { return Ok(name) };
        match name.chars().next() {
            None => Err("Index 0 out of bounds for length 0".to_string()),
            Some('_') => Ok(format!("{layer}{name}")),
            Some(_) => Ok(format!("{layer}_{name}")),
        }
    }

    /// `NameHelper.getName`.
    fn name(&self, i: usize) -> Result<String, String> {
        match self.p[i].kind {
            Kind::Dictionary if self.layer_root(i) => Ok(format!("{}_FunctionalExchanges_Categories", text(&self.layer(i)))),
            Kind::Dictionary => self.prefixed(i),
            Kind::System => Ok(self.prefixed(i)?.chars().take(55).collect()),
            _ => Ok(java::normalize(self.p[i].name.as_deref())),
        }
    }

    /// `NameHelper.getFullName`: the path from the system (or the root) down to the element.
    fn full_name(&self, i: usize) -> Result<String, String> {
        let mut path = self.name(i)?;
        let mut at = i;
        while let Some(container) = self.p[at].container {
            if self.p[at].kind == Kind::System {
                break;
            }
            path = format!("{}/{path}", self.name(container)?);
            at = container;
        }
        Ok(path)
    }

    fn file_path(&self, i: usize) -> Result<String, String> {
        let (folder, extension) = if self.p[i].kind == Kind::Dictionary { ("data/", ".sldd") } else { ("models/", ".slx") };
        let mut path = folder.to_string();
        if let Some(layer) = self.layer(i) {
            path.push_str(&format!("{layer}/"));
        }
        Ok(format!("{path}{}{extension}", self.name(i)?))
    }

    fn file_name(&self, dictionary: usize) -> Result<String, String> {
        Ok(format!("{}.sldd", self.name(dictionary)?))
    }

    fn modeler_id(&self, i: usize) -> Option<String> {
        self.p[i].originator.as_ref().filter(|o| !o.is_empty()).map(|o| format!("ModelerId:{o}"))
    }

    fn add_modeler_id(&mut self, i: usize, full: &str) {
        let Some(id) = self.modeler_id(i) else { return };
        if self.p[i].kind.is_datatype() || self.p[i].kind == Kind::BusElement {
            self.say(format!("{full}.Description = '{id}';"));
        } else {
            self.say(format!("set_param('{full}', 'Tag', '{id}');"));
        }
    }

    fn add_labels(&mut self, target: &str, i: usize) {
        if let Some(layer) = self.layer(i) {
            self.say(format!("addLabel({target}, 'Capella Layer', '{layer}');"));
        }
    }

    // ---- Data types -----------------------------------------------------------

    fn has_datatype(&self, i: usize) -> bool {
        if !self.p[i].kind.holds_datatype() {
            return false;
        }
        match self.p[i].datatype {
            Some(dt) if self.p[dt].kind.holds_datatype() => self.has_datatype(dt),
            dt => dt.is_some(),
        }
    }

    fn is_bus(&self, i: usize) -> bool {
        if !self.p[i].kind.holds_datatype() {
            return false;
        }
        match self.p[i].datatype {
            Some(dt) if self.p[dt].kind.holds_datatype() => self.is_bus(dt),
            Some(dt) => self.p[dt].kind == Kind::Bus,
            None => false,
        }
    }

    fn type_text(&self, dt: usize) -> Result<String, String> {
        Ok(match self.p[dt].kind {
            Kind::Bus => quote(&format!("Bus: {}", self.name(dt)?)),
            Kind::BuiltIntype => quote(&text(&self.p[dt].name)),
            Kind::CustomType => quote(&self.name(dt)?),
            Kind::Enumeration => quote(&format!("Enum: {}", self.name(dt)?)),
            Kind::Signal => format!("{}.DataType", self.name(dt)?),
            other => quote(&format!("Unexpected type: {}", other.class())),
        })
    }

    /// `PropertyHelper.getMatlabParameters`: the element's MATLAB parameters, in Java's hash order.
    fn matlab_parameters(&self, i: usize) -> Vec<(String, String)> {
        let entries: Vec<(String, String)> = self.p[i]
            .params
            .iter()
            .filter(|(k, _)| !BRIDGE_KEYS.contains(&k.as_str()))
            .filter_map(|(k, v)| {
                let value = match v {
                    Value::Primitive(spec) => spec.clone(),
                    Value::Multiple(values) => Some(multiple(values)),
                    Value::Other => None,
                }?;
                Some((k.clone(), value))
            })
            .collect();
        let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_str()).collect();
        java::hash_order(&keys).into_iter().map(|j| entries[j].clone()).collect()
    }

    fn bounds(&self, i: usize) -> Result<Vec<String>, String> {
        let o = &self.p[i];
        let Some(value) = o.param(BOUNDS) else {
            return Err(format!("There is no key '{BOUNDS}' on object {} / {}", text(&o.name), text(&o.id)));
        };
        let Value::Multiple(values) = value else {
            return Err(format!("class org.eclipse.capella.simulink.core.model.mlinfra.impl.PrimitiveValueImpl cannot be cast to class org.eclipse.capella.simulink.core.model.mlinfra.MultipleValue (id:{})", text(&o.id)));
        };
        if values.len() != 4 {
            return Err(format!("Bounds must have 4 scalars. id:{}", text(&o.id)));
        }
        Ok(values.iter().map(|v| match v {
            Value::Primitive(spec) => text(spec),
            _ => "?".to_string(),
        }).collect())
    }

    fn ints(&self, i: usize) -> Result<Vec<i32>, String> {
        self.bounds(i)?.iter().map(|v| v.parse::<i32>().map_err(|_| format!("For input string: \"{v}\""))).collect()
    }

    /// `[x y x+width y+height]`.
    fn simulink_bounds(&self, i: usize) -> Result<String, String> {
        let b = self.ints(i)?;
        Ok(format!("[{} {} {} {}]", b[0], b[1], b[0] + b[2], b[1] + b[3]))
    }

    fn location(&self, i: usize, key: &str) -> Result<String, String> {
        let o = &self.p[i];
        let Some(value) = o.param(key) else {
            return Err(format!("There is no key '{key}' on object {} / {}", text(&o.name), text(&o.id)));
        };
        match value {
            Value::Primitive(spec) => Ok(text(spec)),
            Value::Multiple(values) if values.len() == 1 => Ok(values.first().and_then(primitive).unwrap_or_else(|| "null".to_string())),
            Value::Multiple(values) if values.len() == 2 => Ok(format!("{} {}", values[0].clone_text(), values[1].clone_text())),
            _ => Err(format!("Location must have 1 or 2 scalars. id:{}", text(&o.id))),
        }
    }

    // ---- The project ----------------------------------------------------------

    fn project(&mut self, project: usize) -> Result<(), String> {
        let name = self.name(project)?;
        match self.p[project].param("Folder").and_then(primitive) {
            None => {
                self.say("if not(exist('project_path', 'var'))");
                self.say("\tproject_path = userpath;");
                self.say(format!("\tproject_path =  fullfile(project_path, '{name}');"));
                self.say("end");
            }
            Some(path) => self.say(format!("project_path = '{path}';")),
        }
        self.say("if isfolder(project_path)");
        self.say("\tproj = slproject.loadProject(project_path);");
        self.say("else");
        self.say(format!("\tproj = Simulink.createFromTemplate('Cap2SL2.sltx', 'Name', '{name}', 'Folder', project_path);"));
        self.say("end");
        self.say("cd(project_path);");
        for dictionary in self.p[project].dictionaries.clone() {
            if self.p[dictionary].kind == Kind::Dictionary {
                self.project_dictionary(dictionary)?;
            }
        }
        if !self.used.is_empty() {
            let used = java::hash_set(&self.used);
            self.say(format!("evalin('base', \"clear {}\");", used.join(" ")));
        }
        for system in self.systems(project)? {
            self.project_system(system)?;
            let file = self.file_path(system)?;
            self.say(format!("file = proj.addFile('{file}');"));
            self.add_labels("file", system);
        }
        self.say("cd(proj.Information.ProjectStartupFolder);");
        self.say("clear proj file project_path;");
        Ok(())
    }

    /// `SystemCollector`: every system the project's systems use, the used ones first.
    fn systems(&self, project: usize) -> Result<Vec<usize>, String> {
        fn system(g: &Generator, project: usize, sys: usize, out: &mut Vec<usize>) -> Result<(), String> {
            out.retain(|&s| s != sys);
            out.insert(0, sys);
            if let Some(dictionary) = g.p[sys].dictionary {
                if !g.p[project].dictionaries.contains(&dictionary) {
                    return Err(format!("The dict '{}' must be referenced by the project.", text(&g.p[dictionary].name)));
                }
            }
            diagram(g, project, sys, out)
        }
        fn diagram(g: &Generator, project: usize, d: usize, out: &mut Vec<usize>) -> Result<(), String> {
            for &block in &g.p[d].blocks {
                match g.p[block].kind {
                    Kind::Subsystem => diagram(g, project, block, out)?,
                    Kind::SystemReference => {
                        let Some(sys) = g.p[block].system else {
                            return Err(format!("SystemReference '{}' has no system.", text(&g.p[block].id)));
                        };
                        system(g, project, sys, out)?;
                    }
                    _ => {}
                }
            }
            Ok(())
        }
        let mut out = Vec::new();
        for sys in self.p[project].systems.clone() {
            system(self, project, sys, &mut out)?;
        }
        Ok(out)
    }

    fn project_dictionary(&mut self, dictionary: usize) -> Result<(), String> {
        for sub in self.p[dictionary].sub_dictionaries.clone() {
            self.project_dictionary(sub)?;
        }
        if !self.visited.contains(&dictionary) {
            self.visited.push(dictionary);
            self.dictionary(dictionary)?;
            let file = self.file_path(dictionary)?;
            self.say(format!("file = proj.addFile('{file}');"));
            self.add_labels("file", dictionary);
        }
        Ok(())
    }

    fn project_system(&mut self, sys: usize) -> Result<(), String> {
        self.system(sys)?;
        let name = self.name(sys)?;
        let file = self.file_path(sys)?;
        self.say(format!("save_system('{name}', fullfile(project_path, '{file}'));"));
        Ok(())
    }

    // ---- Dictionaries ---------------------------------------------------------

    fn dictionary(&mut self, dictionary: usize) -> Result<(), String> {
        let name = self.name(dictionary)?;
        let mut variables = Vec::new();
        for dt in self.p[dictionary].datatypes.clone() {
            let variable = match self.p[dt].kind {
                Kind::Bus => Some(self.bus(dt)?),
                Kind::Signal => Some(self.signal(dt)?),
                Kind::Enumeration => Some(self.enumeration(dt)?),
                Kind::CustomType => Some(self.custom_type(dt)?),
                _ => None,
            };
            variables.push(format!("'{}'", text(&variable)));
        }
        let path = self.file_path(dictionary)?;
        self.say(format!("if isfile('{path}')"));
        self.say(format!("\t{name} = Simulink.data.dictionary.open('{path}');"));
        self.say("else");
        self.say(format!("\t{name} = Simulink.data.dictionary.create('{path}');"));
        self.say("end");
        if !variables.is_empty() {
            self.say(format!("importFromBaseWorkspace({name}, 'varList', {{{}}}, 'existingVarsAction','overwrite');", variables.join(", ")));
        }
        for sub in self.p[dictionary].sub_dictionaries.clone() {
            let file = self.file_name(sub)?;
            self.say(format!("addDataSource({name}, '{file}');"));
        }
        self.say(format!("saveChanges({name});"));
        self.say(format!("clear {name};"));
        self.used.extend(variables);
        Ok(())
    }

    fn bus(&mut self, bus: usize) -> Result<String, String> {
        self.say("clear elems;");
        for (n, element) in self.p[bus].bus_elements.clone().into_iter().enumerate() {
            let target = format!("elems({})", n + 1);
            self.say(format!("{target} = Simulink.BusElement;"));
            let name = self.name(element)?;
            self.say(format!("{target}.Name = '{name}';"));
            self.add_modeler_id(element, &target);
            if self.has_datatype(element) {
                let dt = self.type_text(self.p[element].datatype.expect("a datatype"))?;
                self.say(format!("{target}.DataType = {dt};"));
            }
        }
        let name = self.name(bus)?;
        self.say(format!("{name} = Simulink.Bus;"));
        self.add_modeler_id(bus, &name);
        if !self.p[bus].bus_elements.is_empty() {
            self.say(format!("{name}.Elements = elems;"));
        }
        self.say(format!("assignin('base', '{name}', {name});"));
        self.say("clear elems;");
        Ok(name)
    }

    fn fields(&mut self, i: usize, name: &str, quoted: &[&str]) {
        for (key, value) in self.matlab_parameters(i) {
            let value = if quoted.contains(&key.as_str()) { quote(&value) } else { value };
            self.say(format!("{name}.{key} = {value};"));
        }
    }

    fn signal(&mut self, signal: usize) -> Result<String, String> {
        let name = self.name(signal)?;
        self.say(format!("{name} = Simulink.Signal;"));
        if self.has_datatype(signal) {
            let dt = self.type_text(self.p[signal].datatype.expect("a datatype"))?;
            self.say(format!("{name}.DataType = {dt};"));
        }
        if !empty(&self.p[signal].initial_value) {
            self.say(format!("{name}.InitialValue = '{}';", text(&self.p[signal].initial_value)));
        }
        if !empty(&self.p[signal].unit) {
            self.say(format!("{name}.Unit = '{}';", text(&self.p[signal].unit)));
        }
        self.add_modeler_id(signal, &name);
        self.fields(signal, &name, &["Description"]);
        self.say(format!("assignin('base', '{name}', {name});"));
        Ok(name)
    }

    fn enumeration(&mut self, enumeration: usize) -> Result<String, String> {
        let name = self.name(enumeration)?;
        self.say(format!("{name} = Simulink.data.dictionary.EnumTypeDefinition;"));
        self.fields(enumeration, &name, &["DataScope", "DefaultValue", "Description", "HeaderFile", "StorageType"]);
        for literal in self.p[enumeration].literals.clone() {
            let literal_name = self.name(literal)?;
            self.say(format!("appendEnumeral({name}, '{literal_name}', {}, '');", self.p[literal].value));
        }
        self.say(format!("removeEnumeral({name},1);"));
        self.say(format!("assignin('base', '{name}', {name});"));
        Ok(name)
    }

    fn custom_type(&mut self, custom: usize) -> Result<String, String> {
        let name = self.name(custom)?;
        self.say(format!("{name} = Simulink.NumericType;"));
        self.fields(custom, &name, &["DataScope", "DataTypeMode", "DataTypeOverride", "HeaderFile", "Description", "Signedness"]);
        self.say(format!("assignin('base', '{name}', {name});"));
        Ok(name)
    }

    // ---- Systems --------------------------------------------------------------

    fn system(&mut self, sys: usize) -> Result<(), String> {
        let name = self.name(sys)?;
        self.say(format!("new_system('{name}');"));
        self.add_modeler_id(sys, &name);
        self.add_description(sys)?;
        if let Some(dictionary) = self.p[sys].dictionary {
            let file = self.file_name(dictionary)?;
            self.say(format!("set_param('{name}', 'DataDictionary', '{file}');"));
        }
        self.block_ports(sys)?;
        self.diagram_elements(sys)?;
        self.layout(sys, &name)
    }

    fn add_description(&mut self, i: usize) -> Result<(), String> {
        if !empty(&self.p[i].description) {
            let name = self.full_name(i)?;
            self.say(format!("set_param('{name}', 'Description', '{}');", text(&self.p[i].description)));
        }
        Ok(())
    }

    fn add_datatype(&mut self, i: usize) -> Result<(), String> {
        if self.has_datatype(i) {
            let name = self.full_name(i)?;
            let dt = self.type_text(self.p[i].datatype.expect("a datatype"))?;
            self.say(format!("set_param('{name}', 'OutDataTypeStr', {dt});"));
        }
        Ok(())
    }

    fn has_bounds(&self, i: usize) -> bool {
        self.p[i].param(BOUNDS).is_some()
    }

    fn layout(&mut self, diagram: usize, name: &str) -> Result<(), String> {
        let bounded = self.has_bounds(diagram) || pivot::contents(&self.p, diagram).into_iter().any(|c| self.has_bounds(c));
        if bounded {
            return Ok(());
        }
        let used: Vec<usize> = self.p[diagram].lines.iter().flat_map(|&l| [self.p[l].line_source, self.p[l].line_target]).flatten().collect();
        for port in self.p[diagram].ports.clone() {
            if !used.contains(&port) {
                self.port_decoration(diagram, port)?;
            }
        }
        self.say("try");
        self.say(format!("\tSimulink.BlockDiagram.arrangeSystem('{name}');"));
        self.say("catch");
        self.say("end");
        Ok(())
    }

    fn port_decoration(&mut self, diagram: usize, port: usize) -> Result<(), String> {
        let diagram_name = self.full_name(diagram)?;
        let port_name = self.full_name(port)?;
        match self.p[port].port_kind.as_str() {
            "Inport" => {
                self.say(format!("t_Block = add_block('simulink/Sinks/Terminator', '{diagram_name}/Terminator', 'MakeNameUnique', 'on' );"));
                self.say("t_handles = get_param(t_Block, 'PortHandles');");
                self.say(format!("p_Block = get_param('{port_name}', 'handle');"));
                self.say("p_handles = get_param(p_Block, 'PortHandles');");
                self.say("terminator = t_handles.Inport(1);");
                self.say("port = p_handles.Outport(1);");
                self.say(format!("add_line('{diagram_name}', port, terminator);"));
                self.say("clear t_Block t_handles p_Block p_handles terminator port");
            }
            "Outport" => {
                self.say(format!("g_Block = add_block('simulink/Sources/Ground', '{diagram_name}/Ground', 'MakeNameUnique', 'on' );"));
                self.say("g_handles = get_param(g_Block, 'PortHandles');");
                self.say(format!("p_Block = get_param('{port_name}', 'handle');"));
                self.say("p_handles = get_param(p_Block, 'PortHandles');");
                self.say("ground = g_handles.Outport(1);");
                self.say("port = p_handles.Inport(1);");
                self.say(format!("add_line('{diagram_name}', ground, port);"));
                self.say("clear g_Block g_handles p_Block p_handles ground port");
            }
            _ => {}
        }
        Ok(())
    }

    fn diagram_elements(&mut self, diagram: usize) -> Result<(), String> {
        for block in self.p[diagram].blocks.clone() {
            match self.p[block].kind {
                Kind::Subsystem => self.subsystem(block)?,
                Kind::Chart => self.chart(block)?,
                Kind::SystemReference => {
                    let model = match self.p[block].system {
                        Some(sys) => Some(self.name(sys)?),
                        None => None,
                    };
                    self.p[block].set_param("ModelFile", Value::Primitive(model));
                    self.block(block)?;
                }
                _ => self.block(block)?,
            }
        }
        for line in self.p[diagram].lines.clone() {
            self.line(line)?;
        }
        Ok(())
    }

    fn block(&mut self, block: usize) -> Result<(), String> {
        self.block_properties(block)?;
        self.block_ports(block)
    }

    fn block_properties(&mut self, block: usize) -> Result<(), String> {
        let full = self.full_name(block)?;
        self.say(format!("add_block('{}', '{full}');", text(&self.p[block].source)));
        if self.has_bounds(block) {
            let bounds = self.simulink_bounds(block)?;
            self.say(format!("set_param('{full}', 'Position', '{bounds}')"));
        }
        self.add_datatype(block)?;
        self.add_description(block)?;
        self.add_modeler_id(block, &full);
        for (key, value) in self.matlab_parameters(block) {
            self.say(format!("set_param('{full}', '{key}', '{value}')"));
        }
        Ok(())
    }

    fn block_ports(&mut self, holder: usize) -> Result<(), String> {
        if self.p[holder].kind == Kind::Subsystem {
            let full = self.full_name(holder)?;
            self.say(format!("CapSLBridge.internal.cleanSubsystem('{full}');"));
        }
        let mut ports = self.p[holder].ports.clone();
        ports.sort_by_key(|&p| self.p[p].index);
        for port in ports {
            self.port(port)?;
        }
        Ok(())
    }

    fn subsystem(&mut self, subsystem: usize) -> Result<(), String> {
        self.block(subsystem)?;
        self.diagram_elements(subsystem)?;
        let full = self.full_name(subsystem)?;
        self.layout(subsystem, &full)
    }

    fn port(&mut self, port: usize) -> Result<(), String> {
        let holder = self.p[port].container.expect("a port is held");
        let kind = self.p[port].port_kind.clone();
        let index = self.p[port].index;
        if self.p[holder].kind.is_diagram() {
            let full = self.full_name(port)?;
            let library = match kind.as_str() {
                "Inport" => "simulink/Ports & Subsystems/In1",
                "Outport" => "simulink/Ports & Subsystems/Out1",
                "Enable" => "simulink/Ports & Subsystems/Enable",
                "Trigger" => "simulink/Ports & Subsystems/Trigger",
                _ => return Err("java.lang.UnsupportedOperationException".to_string()),
            };
            self.say(format!("add_block('{library}', '{full}');"));
            if kind != "Enable" {
                self.say(format!("set_param('{full}', 'Port', '{index}');"));
            }
            self.add_datatype(port)?;
            if kind == "Outport" && self.is_bus(port) {
                self.say(format!("set_param('{full}', 'BusOutputAsStruct', 'on');"));
            }
            if self.has_bounds(port) {
                let bounds = self.simulink_bounds(port)?;
                self.say(format!("set_param('{full}', 'Position', '{bounds}');"));
            }
            self.add_modeler_id(port, &full);
            self.add_description(port)?;
            for (key, value) in self.matlab_parameters(port) {
                self.say(format!("set_param('{full}', '{key}', '{value}')"));
            }
            return Ok(());
        }
        if kind == "Outport" {
            let holder_name = self.full_name(holder)?;
            self.say(format!("handles = get_param('{holder_name}', 'porthandles');"));
            if let Some(id) = self.modeler_id(port) {
                self.say(format!("set_param(handles.{kind}({index}), 'Tag', '{id}');"));
            }
            self.say("clear handles;");
        }
        Ok(())
    }

    fn line(&mut self, line: usize) -> Result<(), String> {
        let diagram = self.p[line].container.expect("a line is in a diagram");
        let container = self.full_name(diagram)?;
        let source = self.p[line].line_source;
        let target = self.p[line].line_target;
        self.port_handler(diagram, source, "source")?;
        self.port_handler(diagram, target, "target")?;
        self.say(format!("line = add_line('{container}', source, target, 'autorouting','smart');"));
        let name = self.name(line)?;
        if !name.is_empty() {
            self.say(format!("set_param(line, 'Name', '{name}');"));
        }
        if let Some(id) = self.modeler_id(line) {
            self.say(format!("set_param( line, 'Tag', '{id}');"));
        }
        if !empty(&self.p[line].description) {
            self.say(format!("set_param(line, 'Description', '{}');", text(&self.p[line].description)));
        }
        for (key, value) in self.matlab_parameters(line) {
            self.say(format!("set_param(line, '{key}', '{value}')"));
        }
        self.say("clear line handles source target;");
        Ok(())
    }

    fn port_handler(&mut self, diagram: usize, port: Option<usize>, variable: &str) -> Result<(), String> {
        let port = port.ok_or("java.lang.NullPointerException")?;
        let holder = self.p[port].container.expect("a port is held");
        let holder_name = if holder == diagram && self.p[holder].kind.is_diagram() { self.full_name(port)? } else { self.full_name(holder)? };
        self.say(format!("handles = get_param('{holder_name}', 'PortHandles');"));
        if holder == diagram {
            let method = match variable {
                "target" => "Inport",
                _ => "Outport",
            };
            self.say(format!("{variable} = handles.{method}(1);"));
        } else {
            self.say(format!("{variable} = handles.{}({});", self.p[port].port_kind, self.p[port].index));
        }
        Ok(())
    }

    // ---- Stateflow ------------------------------------------------------------

    fn chart(&mut self, chart: usize) -> Result<(), String> {
        self.say("states = containers.Map;");
        self.block_properties(chart)?;
        self.say("rt = sfroot;");
        let full = self.full_name(chart)?;
        self.say(format!("ch = rt.find('-isa','Stateflow.Chart','Path', '{full}');"));
        for symbol in self.p[chart].symbols.clone() {
            self.symbol(symbol);
        }
        self.block_ports(chart)?;
        for state in self.p[chart].states.clone() {
            self.state(state)?;
        }
        for transition in self.p[chart].transitions.clone() {
            self.transition(transition)?;
        }
        self.say("clear states rt ch parent transition destination junction myVar source state;");
        Ok(())
    }

    /// A state's key in the chart's map: its Capella id, else its own.
    fn state_id(&self, i: usize) -> String {
        let o = &self.p[i];
        if empty(&o.originator) { text(&o.id) } else { text(&o.originator) }
    }

    fn state(&mut self, state: usize) -> Result<(), String> {
        if self.p[state].kind == Kind::Junction {
            return self.junction(state);
        }
        if self.p[state].kind != Kind::State {
            return Ok(());
        }
        match self.p[state].container.filter(|&c| self.p[c].kind == Kind::State) {
            None => self.say("parent = ch;"),
            Some(parent) => self.say(format!("parent = states('{}');", self.state_id(parent))),
        }
        self.say("state = Stateflow.State(parent);");
        let bounds = self.bounds(state)?;
        self.say(format!("state.Position = [{}];", bounds.join(" ")));
        if let Some(id) = self.modeler_id(state) {
            self.say(format!("state.Tag = '{id}';"));
        }
        let label = self.state_label(state)?;
        self.say(format!("state.labelString = {label};"));
        self.say(format!("state.Decomposition = '{}';", self.p[state].decomposition));
        self.say(format!("states('{}') = state;", self.state_id(state)));
        for sub in self.p[state].sub_states.clone() {
            self.state(sub)?;
        }
        Ok(())
    }

    fn state_label(&self, state: usize) -> Result<String, String> {
        let o = &self.p[state];
        let mut label = format!("'{}'", self.name(state)?);
        if !empty(&o.entry) {
            label.push_str(&format!(", 10, 'en: {}'", text(&o.entry)));
        }
        if !empty(&o.during) {
            label.push_str(&format!(", 10, 'du: {}'", text(&o.during)));
        }
        if !empty(&o.exit) {
            label.push_str(&format!(", 10, 'ex: {}'", text(&o.exit)));
        }
        if empty(&o.entry) && empty(&o.during) && empty(&o.exit) {
            return Ok(label);
        }
        Ok(format!("[{label}]"))
    }

    fn junction(&mut self, junction: usize) -> Result<(), String> {
        self.say("junction = Stateflow.Junction(ch);");
        self.say(format!("junction.type = '{}';", self.p[junction].junction));
        if let Some(id) = self.modeler_id(junction) {
            self.say(format!("junction.Tag = '{id}';"));
        }
        let b = self.ints(junction)?;
        self.say(format!("junction.Position.Center = [{} {}];", b[0] + b[2] / 2, b[1] + b[3] / 2));
        self.say(format!("junction.Position.Radius = {};", (b[3] + b[2]) / 4));
        self.say(format!("states('{}') = junction;", self.state_id(junction)));
        Ok(())
    }

    fn transition(&mut self, transition: usize) -> Result<(), String> {
        self.say("transition = Stateflow.Transition(ch);");
        if let Some(id) = self.modeler_id(transition) {
            self.say(format!("transition.Tag = '{id}';"));
        }
        if let Some(source) = self.p[transition].state_source {
            self.say(format!("source = states('{}');", self.state_id(source)));
            self.say("transition.source = source;");
        }
        let location = self.location(transition, SOURCE_END)?;
        if location.trim().contains(' ') {
            self.say(format!("transition.SourceEndPoint = [{location}];"));
        } else {
            self.say(format!("transition.SourceOClock = {location};"));
        }
        match self.p[transition].destination {
            None => self.say(format!("echo ('Error transition {} has no destination.');", text(&self.p[transition].id))),
            Some(destination) => {
                self.say(format!("destination = states('{}');", self.state_id(destination)));
                self.say("transition.Destination = destination;");
                let location = self.location(transition, DESTINATION_END)?;
                if location.trim().contains(' ') {
                    self.say(format!("transition.DestinationEndPoint = [{location}];"));
                } else {
                    self.say(format!("transition.DestinationOClock = {location};"));
                }
            }
        }
        let o = &self.p[transition];
        let mut label = String::new();
        if !empty(&o.event) {
            label.push_str(&format!("'{}'", text(&o.event)));
        }
        if !empty(&o.comment) {
            if !label.is_empty() {
                label.push_str(" newline");
            }
            label.push_str(&format!(" '...Trigger:{}'", text(&o.comment)));
        }
        if !empty(&o.condition) {
            if !label.is_empty() {
                label.push_str(" newline");
            }
            label.push_str(&format!(" '...Guard:[{}]'", text(&o.condition)));
        }
        if !label.is_empty() {
            self.say(format!("transition.LabelString = [{label}];"));
        }
        Ok(())
    }

    fn symbol(&mut self, symbol: usize) {
        let kind = self.p[symbol].kind;
        let class = match kind {
            Kind::Data => "Data",
            Kind::Event => "Event",
            Kind::Message => "Message",
            _ => return,
        };
        self.say(format!("myVar = Stateflow.{class}(ch);"));
        let o = self.p[symbol].clone();
        self.say(format!("myVar.Name = '{}';", text(&o.name)));
        self.say(format!("myVar.Scope = '{}';", o.scope));
        if !empty(&o.description) {
            self.say(format!("myVar.Description = '{}';", text(&o.description)));
        }
        if let Some(port) = o.underlying_port {
            self.say(format!("myVar.Port = {};", self.p[port].index));
        }
        if let Some(id) = self.modeler_id(symbol) {
            self.say(format!("myVar.Tag = '{id}';"));
        }
        for (key, value) in self.matlab_parameters(symbol) {
            self.say(format!("myVar.{key} = '{value}';"));
        }
        let symbol_datatype = |g: &Generator| -> Option<String> {
            let port = o.underlying_port?;
            let dt = g.p[port].datatype?;
            g.type_text(dt).ok()
        };
        match kind {
            Kind::Message => {
                if let Some(dt) = symbol_datatype(self) {
                    self.say(format!("myVar.DataType = {dt};"));
                }
                self.say("v = version('-release');");
                self.say("minor = v(5);");
                self.say("vers = str2num(v(1:4));");
                self.say("if (vers == 2019 && minor == 'b' ) | vers >= 2020;");
                self.say("\tmyVar.UseInternalQueue = 0;");
                self.say("end");
                self.say("clear v minor vers;");
            }
            Kind::Data => {
                if let Some(dt) = symbol_datatype(self) {
                    self.say(format!("myVar.DataType = {dt};"));
                }
            }
            _ => {}
        }
    }
}

trait CloneText {
    fn clone_text(&self) -> String;
}

impl CloneText for Value {
    fn clone_text(&self) -> String {
        match self {
            Value::Primitive(spec) => text(spec),
            _ => "null".to_string(),
        }
    }
}
