//! SysML v2 textual notation export.
//!
//! Emits an OMG SysML v2 textual model from the compiled ArcLang model.
//! The mapping follows the `sysml` column of the metamodel
//! (`spec/METAMODEL.md`):
//!
//! | ArcLang                         | SysML v2                                   |
//! |---------------------------------|--------------------------------------------|
//! | requirement                     | `requirement` usage (short name = id)      |
//! | function / activity             | `action def` with `in`/`out` items, usage  |
//! | functional chain                | `action def` with `then` successions       |
//! | component / node / actor        | `part def` + `part` usage, nested parts    |
//! | component port                  | `port def` + `port`                        |
//! | logical interface               | `interface def`                            |
//! | exchange (component, physical)  | `connect`                                  |
//! | functional exchange             | `flow`                                     |
//! | deployment                      | `allocate`                                 |
//! | typed attribute (quantity)      | `attribute x : DurationValue = 25 [ms];`   |
//! | enum / text attribute           | `attribute x : String = "...";`            |
//! | class / exchange item / enum    | `item def` / `enum def`                    |
//! | capability / mission            | `use case def`                             |
//! | state machine                   | `state def` with transitions               |
//! | scenario                        | `occurrence def` with events and messages  |
//! | test case                       | `verification def` with `verify`           |
//! | constraint                      | `assert constraint { <expression> }`       |
//! | type / `extends` / `is:`        | `abstract part def` / `action def`, `:>`,  |
//! |                                 | redefined attributes as `:>>`              |
//! | trace satisfies / realizes      | `satisfy ... by` / `dependency`            |
//! | hazard                          | `requirement` tagged as hazard in `doc`    |
//!
//! Identifiers: every ArcLang name becomes a SysML basic identifier
//! (`Sensor Fusion` → `Sensor_Fusion`) with the original text kept in
//! `doc`; element ids become short names (`<'LC-001'>`). Output is
//! deterministic for a given model.
//!
//! Quantities use the standard library: `private import ISQ::*;` and
//! `SI::*` make `DurationValue`, `[ms]`, `['Mbit/s']` resolve.

use super::ast::*;
use super::metamodel::{AttrType, Metamodel};
use super::quantity::Quantity;
use super::multiplicity::Multiplicity;
use super::types::EffectiveType;
use super::semantic::SemanticModel;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// A SysML v2 basic identifier: letter or underscore, then letters, digits,
/// underscores. Anything else must use the unrestricted-name form '...'.
pub fn sysml_name(raw: &str) -> String {
    let mut chars = raw.chars();
    let valid = match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    };
    if valid {
        raw.to_string()
    } else {
        format!("'{}'", raw.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}

/// Basic identifier derived from any text (`Sensor Fusion` → `Sensor_Fusion`,
/// `REQ-001` → `REQ_001`, `2nd` → `_2nd`).
fn ident(raw: &str) -> String {
    let mut out = String::new();
    for c in raw.chars() {
        out.push(if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' });
    }
    if out.is_empty() {
        out.push('_');
    }
    if out.chars().next().map_or(false, |c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    // A reserved word is only usable as a name in its quoted form.
    if SYSML_KEYWORDS.contains(&out.as_str()) {
        return format!("'{}'", out);
    }
    out
}

/// Reserved words of the SysML v2 textual notation.
const SYSML_KEYWORDS: &[&str] = &[
    "about", "abstract", "accept", "action", "actor", "after", "alias", "all", "allocate",
    "allocation", "analysis", "and", "as", "assert", "assign", "assume", "at", "attribute",
    "bind", "binding", "by", "calc", "case", "comment", "concern", "connect", "connection",
    "constant", "constraint", "crosses", "decide", "def", "default", "defined", "dependency",
    "derived", "do", "doc", "else", "end", "entry", "enum", "event", "exhibit", "exit", "expose",
    "false", "feature", "filter", "first", "flow", "for", "fork", "frame", "from", "hastype",
    "if", "implies", "import", "in", "include", "individual", "inout", "interface", "istype",
    "item", "join", "language", "library", "locale", "loop", "merge", "message", "meta",
    "metadata", "nonunique", "not", "null", "objective", "occurrence", "of", "or", "ordered",
    "out", "package", "parallel", "part", "perform", "port", "private", "protected", "public",
    "redefines", "ref", "references", "render", "rendering", "rep", "require", "requirement",
    "return", "satisfy", "send", "snapshot", "specializes", "stakeholder", "standard", "state",
    "subject", "subsets", "succession", "terminate", "then", "timeslice", "to", "transition",
    "true", "until", "use", "variant", "variation", "verification", "verify", "via", "view",
    "viewpoint", "when", "while", "xor",
];

fn string_literal(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

fn doc_line(indent: &str, text: &str) -> String {
    format!("{indent}doc /* {} */\n", text.replace("*/", "* /"))
}

/// Unique identifier allocator: the same key always maps to the same
/// identifier, two different keys never collide.
#[derive(Default)]
struct Names {
    by_key: HashMap<String, String>,
    taken: BTreeSet<String>,
}

impl Names {
    fn get(&mut self, key: &str, wanted: &str) -> String {
        if let Some(existing) = self.by_key.get(key) {
            return existing.clone();
        }
        let base = ident(wanted);
        let mut candidate = base.clone();
        let mut n = 2;
        while self.taken.contains(&candidate) {
            candidate = format!("{}_{}", base, n);
            n += 1;
        }
        self.taken.insert(candidate.clone());
        self.by_key.insert(key.to_string(), candidate.clone());
        candidate
    }
}

struct Generator<'a> {
    ast: &'a Model,
    semantic: &'a SemanticModel,
    metamodel: Metamodel,
    out: String,
    defs: Names,
    usages: Names,
    /// Element id or name → usage path (`p_LC_001`, `p_LC_001.p_LC_002`).
    usage_path: BTreeMap<String, String>,
    /// Function id or name → action usage name.
    action_usage: BTreeMap<String, String>,
    /// Function id or name → action def name.
    action_def: BTreeMap<String, String>,
    item_defs: BTreeSet<String>,
    port_defs: BTreeSet<String>,
    requirement_usage: BTreeMap<String, String>,
    /// User-defined types with inheritance folded in.
    types: BTreeMap<String, EffectiveType>,
    /// True while emitting a type's own attributes: a definition gives
    /// DEFAULT values (`default =`), which specializations may override; an
    /// element binds its values (`=`).
    emitting_defaults: bool,
    /// Type definitions the export needs: a type is emitted as a part def
    /// and/or an action def depending on what it types.
    type_requests: BTreeSet<(String, Category)>,
    /// Part id or name → part def name (dependency ends cannot be feature chains).
    part_def: BTreeMap<String, String>,
    /// Unit symbols (SysML spelling) used by exported quantities.
    units_used: BTreeSet<&'static str>,
}

pub fn generate_sysmlv2(semantic: &SemanticModel, ast: &Model) -> String {
    let mut generator = Generator {
        ast,
        semantic,
        metamodel: Metamodel::current(),
        out: String::new(),
        defs: Names::default(),
        usages: Names::default(),
        usage_path: BTreeMap::new(),
        action_usage: BTreeMap::new(),
        action_def: BTreeMap::new(),
        item_defs: BTreeSet::new(),
        port_defs: BTreeSet::new(),
        requirement_usage: BTreeMap::new(),
        types: super::types::effective_types(&ast.types).unwrap_or_default(),
        type_requests: BTreeSet::new(),
        emitting_defaults: false,
        part_def: BTreeMap::new(),
        units_used: BTreeSet::new(),
    };
    generator.run();
    generator.out
}

impl<'a> Generator<'a> {
    fn run(&mut self) {
        let package = self.defs.get("package", self.semantic.name.as_deref().unwrap_or("ArcLangModel"));
        self.out.push_str("// Generated by ArcLang — SysML v2 textual notation export\n");
        self.out.push_str("// Mapping: see spec/METAMODEL.md (column \"SysML v2\")\n");
        self.out.push_str(&format!("package {} {{\n", package));
        self.out.push_str("    private import ScalarValues::*;\n");
        self.out.push_str("    private import MeasurementReferences::*;\n");
        self.out.push_str("    private import ISQ::*;\n");
        self.out.push_str("    private import SI::*;\n\n");

        self.collect_item_types();
        self.emit_data();
        self.emit_requirements();
        self.emit_hazards();
        self.emit_use_cases();
        self.emit_functions();
        self.emit_chains();
        self.emit_ports();
        self.emit_components();
        self.emit_physical();
        self.emit_exchanges();
        self.emit_state_machines();
        self.emit_scenarios();
        self.emit_verification();
        self.emit_constraints();
        self.emit_traces();
        self.emit_types();
        self.emit_units();

        self.out.push_str("}\n");
    }

    // ---- attributes ------------------------------------------------------

    /// Emit typed `attribute` lines for an element of `kind`. Attributes the
    /// metamodel types are emitted with that type (quantities with their ISQ
    /// value type and unit); other scalar attributes as `String`. `id`,
    /// `name` and `description` are carried by short name / name / doc.
    fn emit_attributes(&mut self, indent: &str, kind: &str, attributes: &HashMap<String, AttributeValue>) {
        let mut keys: Vec<&String> = attributes.keys().collect();
        keys.sort();
        // For an element whose definition specializes a type (`:> T`), an
        // attribute inherited unchanged is NOT repeated, and one the element
        // overrides is a redefinition (`:>>`).
        let inherited = self.inherited_attributes(kind, attributes);
        let mut lines = Vec::new();
        for key in keys {
            if matches!(key.as_str(), "id" | "name" | "title" | "description" | "required" | "from" | "to" | "involves" | "verifies" | "mitigated_by" | "participants" | "elements" | "values") {
                continue;
            }
            // A valid multiplicity is written on the usage (`part p : D [4];`).
            if key == "multiplicity" && Multiplicity::of(kind, attributes).is_some() {
                continue;
            }
            // Capability relations are written as relations (`:>`, `include`,
            // `dependency`), not repeated as text attributes.
            let is_capability = matches!(kind, "OperationalCapability" | "Capability" | "CapabilityRealization");
            if is_capability && matches!(key.as_str(), "extends" | "includes" | "specializes") {
                continue;
            }
            let value = &attributes[key];
            let mut redefines = false;
            if let Some(inherited) = &inherited {
                if key == "is" {
                    continue;
                }
                match inherited.get(key.as_str()) {
                    Some(same) if same == value => continue,
                    Some(_) => redefines = true,
                    None => {}
                }
            }
            let declared = self
                .metamodel
                .kind(kind)
                .and_then(|k| k.attributes.iter().find(|a| a.key == key.as_str()))
                .map(|a| a.ty.clone());
            let name = ident(key);
            let line = match (declared, value) {
                (Some(AttrType::Quantity(dimension)), value) => match value.as_quantity() {
                    Some(Ok(quantity)) if quantity.dimension() == dimension => {
                        self.units_used.insert(quantity.spec().sysml);
                        format!("attribute {} : {} = {};", name, dimension.sysml_value_type(), quantity_literal(&quantity))
                    }
                    _ => format!("attribute {} : String = {};  // not a valid {} quantity", name, string_literal(&value.display()), dimension.label()),
                },
                (Some(AttrType::Enum(enumeration)), value) => {
                    let raw = value.display();
                    let canonical = self.metamodel.normalize_enum(enumeration, &raw).map(str::to_string).unwrap_or(raw);
                    format!("attribute {} : String = {};", name, string_literal(&canonical))
                }
                (Some(AttrType::Number), AttributeValue::Number(n)) => format!("attribute {} : Real = {};", name, number_literal(*n)),
                (Some(AttrType::Boolean), AttributeValue::Boolean(b)) => format!("attribute {} : Boolean = {};", name, b),
                (_, AttributeValue::Number(n)) => format!("attribute {} : Real = {};", name, number_literal(*n)),
                (_, AttributeValue::Boolean(b)) => format!("attribute {} : Boolean = {};", name, b),
                (_, AttributeValue::Quantity(quantity)) => {
                    self.units_used.insert(quantity.spec().sysml);
                    format!("attribute {} : {} = {};", name, quantity.dimension().sysml_value_type(), quantity_literal(quantity))
                }
                (_, AttributeValue::String(s)) => format!("attribute {} : String = {};", name, string_literal(s)),
                (_, AttributeValue::List(_)) | (_, AttributeValue::Map(_)) => {
                    format!("attribute {} : String = {};", name, string_literal(&value.display()))
                }
            };
            let line = match (redefines, line.split_once(" = ")) {
                (true, Some((_, literal))) => format!("attribute :>> {} = {}", name, literal),
                _ => line,
            };
            let line = if self.emitting_defaults { line.replacen(" = ", " default = ", 1) } else { line };
            lines.push(line);
        }
        for line in lines {
            self.out.push_str(indent);
            self.out.push_str(&line);
            self.out.push('\n');
        }
    }

    // ---- data ------------------------------------------------------------

    fn collect_item_types(&mut self) {
        for sa in &self.ast.system_analysis {
            for function in &sa.functions {
                collect_function_item_types(function, &mut self.item_defs);
            }
        }
        for la in &self.ast.logical_architecture {
            for component in &la.components {
                collect_component_item_types(component, &mut self.item_defs);
            }
            for exchange in &la.component_exchanges {
                if !exchange.exchange_item.is_empty() {
                    self.item_defs.insert(exchange.exchange_item.clone());
                }
            }
        }
        for item in &self.ast.exchange_items {
            self.item_defs.insert(item.name.clone());
        }
        for class in &self.ast.classes {
            self.item_defs.insert(class.name.clone());
        }
        // An empty data type is an untyped port: it carries `Data`, like a
        // port that states no type at all.
        if self.item_defs.remove("") {
            self.item_defs.insert(UNTYPED_ITEM.to_string());
        }
        self.item_defs.retain(|name| !is_scalar_type(name));
    }

    fn emit_data(&mut self) {
        let enumerations: Vec<&DataType> = self.ast.data_types.iter().filter(|d| d.enumeration_values.is_some()).collect();
        if self.item_defs.is_empty() && enumerations.is_empty() {
            return;
        }
        self.out.push_str("    // ---- Data ----\n");
        for data_type in enumerations {
            let name = self.defs.get(&format!("enum:{}", data_type.name), &data_type.name);
            self.out.push_str(&format!("    enum def {} {{\n", name));
            for value in data_type.enumeration_values.as_deref().unwrap_or(&[]) {
                self.out.push_str(&format!("        {};\n", ident(&value.name)));
            }
            self.out.push_str("    }\n");
        }
        let classes: HashMap<&str, &ClassDef> = self.ast.classes.iter().map(|c| (c.name.as_str(), c)).collect();
        let items: Vec<String> = self.item_defs.iter().cloned().collect();
        for item in items {
            let name = self.defs.get(&format!("item:{}", item), &item);
            match classes.get(item.as_str()) {
                Some(class) if !class.fields.is_empty() => {
                    self.out.push_str(&format!("    item def {} {{\n", name));
                    for field in &class.fields {
                        let ty = scalar_type(&field.attr_type).unwrap_or("String");
                        self.out.push_str(&format!("        attribute {} : {};\n", ident(&field.name), ty));
                    }
                    self.out.push_str("    }\n");
                }
                _ => {
                    if name != item {
                        self.out.push_str(&format!("    item def {} {{ {} }}\n", name, doc_line("", &item).trim_end()));
                    } else {
                        self.out.push_str(&format!("    item def {};\n", name));
                    }
                }
            }
        }
        self.out.push('\n');
    }

    // ---- requirements ----------------------------------------------------

    fn emit_requirements(&mut self) {
        let requirements: Vec<&Requirement> = self.ast.system_analysis.iter().flat_map(|sa| sa.requirements.iter()).collect();
        if requirements.is_empty() {
            return;
        }
        self.out.push_str("    // ---- Requirements ----\n");
        for requirement in requirements {
            let id = requirement
                .attributes
                .get("id")
                .and_then(|v| v.as_string())
                .unwrap_or(&requirement.id)
                .to_string();
            let name = self.usages.get(&format!("req:{}", id), &format!("req_{}", id));
            self.requirement_usage.insert(id.clone(), name.clone());
            self.out.push_str(&format!("    requirement <{}> {} {{\n", sysml_name(&id), name));
            let title = requirement.attributes.get("title").or_else(|| requirement.attributes.get("name")).and_then(|v| v.as_string());
            let description = requirement.attributes.get("description").and_then(|v| v.as_string());
            match (title, description) {
                (Some(t), Some(d)) => self.out.push_str(&doc_line("        ", &format!("{} — {}", t, d))),
                (Some(t), None) => self.out.push_str(&doc_line("        ", t)),
                (None, Some(d)) => self.out.push_str(&doc_line("        ", d)),
                (None, None) => {}
            }
            self.emit_attributes("        ", "Requirement", &requirement.attributes);
            self.out.push_str("    }\n");
        }
        self.out.push('\n');
    }

    fn emit_hazards(&mut self) {
        let hazards: Vec<&Hazard> = self.ast.safety_analysis.iter().flat_map(|s| s.hazards.iter()).collect();
        if hazards.is_empty() {
            return;
        }
        self.out.push_str("    // ---- Hazards (HARA) — SysML v2 has no hazard construct; kept as tagged requirements ----\n");
        for hazard in hazards {
            let name = self.usages.get(&format!("hazard:{}", hazard.name), &format!("hazard_{}", hazard.name));
            self.out.push_str(&format!("    requirement <{}> {} {{\n", sysml_name(&hazard.name), name));
            let description = hazard.attributes.get("description").and_then(|v| v.as_string()).unwrap_or("");
            self.out.push_str(&doc_line("        ", &format!("HAZARD: {}", description)));
            self.emit_attributes("        ", "Hazard", &hazard.attributes);
            if let Some(AttributeValue::List(items)) = hazard.attributes.get("mitigated_by") {
                for item in items {
                    if let Some(requirement) = item.as_string().and_then(|r| self.requirement_usage.get(r)) {
                        self.out.push_str(&format!("        // mitigated by {}\n", requirement));
                    }
                }
            }
            self.out.push_str("    }\n");
        }
        self.out.push('\n');
    }

    fn emit_use_cases(&mut self) {
        // (id, name, kind, attributes); capabilities of every level, nested
        // ones included: the parser lists them after their parent.
        let mut entries: Vec<(String, String, &'static str, &HashMap<String, AttributeValue>)> = Vec::new();
        for oa in &self.ast.operational_analysis {
            for capability in &oa.capabilities {
                entries.push((capability.id.clone(), capability.name.clone(), "OperationalCapability", &capability.attributes));
            }
        }
        for sa in &self.ast.system_analysis {
            for mission in &sa.missions {
                entries.push((mission.id.clone(), mission.name.clone(), "Mission", &mission.attributes));
            }
            for capability in &sa.capabilities {
                entries.push((capability.id.clone(), capability.name.clone(), "Capability", &capability.attributes));
            }
        }
        for la in &self.ast.logical_architecture {
            for capability in &la.capability_realizations {
                entries.push((capability.id.clone(), capability.name.clone(), "CapabilityRealization", &capability.attributes));
            }
        }
        if entries.is_empty() {
            return;
        }
        // Every definition is named before any is written: a capability may
        // specialize or include one declared after it.
        for (id, name, _, _) in &entries {
            self.defs.get(&format!("usecase:{}", id), name);
        }
        let def_of = |defs: &mut Names, id: &str| defs.get(&format!("usecase:{}", id), id);
        let semantic = self.semantic;
        let resolved = |id: &str| semantic.capabilities.iter().find(|c| c.id == id);

        self.out.push_str("    // ---- Missions & capabilities ----\n");
        let mut extensions: Vec<(String, String)> = Vec::new();
        for (id, name, kind, attributes) in &entries {
            let def = def_of(&mut self.defs, id);
            let relations = resolved(id).map(|c| c.relations.as_slice()).unwrap_or(&[]);
            let generals: Vec<String> = relations
                .iter()
                .filter(|r| r.kind == "specializes")
                .map(|r| def_of(&mut self.defs, &r.target))
                .collect();
            let special = if generals.is_empty() { String::new() } else { format!(" :> {}", generals.join(", ")) };
            self.out.push_str(&format!("    use case def <{}> {}{} {{\n", sysml_name(id), def, special));
            self.out.push_str(&doc_line("        ", &format!("{}: {}", kind, name)));
            self.emit_attributes("        ", kind, attributes);
            // What a capability involves has no SysML v2 counterpart: kept
            // as the ids involved, like `realizes` and `mission`.
            let involved = resolved(id).map(|c| c.involves.join(", ")).unwrap_or_default();
            if !involved.is_empty() {
                self.out.push_str(&format!("        attribute involves : String = {};\n", string_literal(&involved)));
            }
            for relation in relations {
                let target = def_of(&mut self.defs, &relation.target);
                match relation.kind.as_str() {
                    "includes" => {
                        let usage = self.usages.get(&format!("usecase-include:{}:{}", id, relation.target), &format!("uc_{}", relation.target));
                        self.out.push_str(&format!("        include use case {} : {};\n", usage, target));
                    }
                    // SysML v2 has no «extend»: kept as a named dependency.
                    "extends" => extensions.push((def.clone(), target)),
                    _ => {}
                }
            }
            // A capability declared inside this one is a part of it.
            let nested: Vec<&str> = semantic
                .capabilities
                .iter()
                .filter(|c| c.parent.as_deref() == Some(id.as_str()))
                .map(|c| c.id.as_str())
                .collect();
            for child in nested {
                let usage = self.usages.get(&format!("usecase-part:{}", child), &format!("uc_{}", child));
                let child_def = def_of(&mut self.defs, child);
                self.out.push_str(&format!("        use case {} : {};\n", usage, child_def));
            }
            self.out.push_str("    }\n");
        }
        for (source, target) in extensions {
            self.out.push_str(&format!("    dependency extends from {} to {};\n", source, target));
        }
        self.out.push('\n');
    }

    // ---- functions -------------------------------------------------------

    fn emit_functions(&mut self) {
        let mut any = false;
        let mut activities: Vec<&OperationalActivity> = Vec::new();
        for oa in &self.ast.operational_analysis {
            for entity in &oa.entities {
                for activity in &entity.activities {
                    collect_activities(activity, &mut activities);
                }
            }
            for activity in &oa.activities {
                collect_activities(activity, &mut activities);
            }
        }
        for activity in activities {
            if !any {
                self.out.push_str("    // ---- Functions ----\n");
                any = true;
            }
            let id = activity.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&activity.id).to_string();
            let def = self.defs.get(&format!("action:{}", id), &activity.name);
            let usage = self.usages.get(&format!("action-usage:{}", id), &format!("a_{}", id));
            self.action_def.insert(id.clone(), def.clone());
            self.action_def.entry(activity.name.clone()).or_insert_with(|| def.clone());
            self.action_usage.insert(id.clone(), usage.clone());
            self.action_usage.entry(activity.name.clone()).or_insert_with(|| usage.clone());
            let special = self.specialization("OperationalActivity", &activity.attributes);
            self.out.push_str(&format!("    action def <{}> {}{} {{\n", sysml_name(&id), def, special));
            self.out.push_str(&doc_line("        ", &format!("{} (operational activity)", activity.name)));
            self.emit_attributes("        ", "OperationalActivity", &activity.attributes);
            self.out.push_str("    }\n");
            self.out.push_str(&format!("    action {} : {};\n", usage, def));
        }
        for sa in &self.ast.system_analysis {
            if !sa.functions.is_empty() && !any {
                self.out.push_str("    // ---- Functions ----\n");
                any = true;
            }
            for function in &sa.functions {
                self.emit_system_function(function);
            }
        }
        let mut logical: Vec<(&LogicalFunction, String)> = Vec::new();
        for la in &self.ast.logical_architecture {
            for component in &la.components {
                collect_logical_functions(component, &mut logical);
            }
        }
        for (function, owner) in logical {
            if !any {
                self.out.push_str("    // ---- Functions ----\n");
                any = true;
            }
            let id = function.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&function.name).to_string();
            if self.action_def.contains_key(&id) || self.action_def.contains_key(&function.name) {
                continue;
            }
            let def = self.defs.get(&format!("action:{}", id), &function.name);
            self.action_def.insert(id.clone(), def.clone());
            self.action_def.insert(function.name.clone(), def.clone());
            let special = self.specialization("LogicalFunction", &function.attributes);
            self.out.push_str(&format!("    action def <{}> {}{} {{\n", sysml_name(&id), def, special));
            self.out.push_str(&doc_line("        ", &format!("{} (logical function of {})", function.name, owner)));
            self.emit_attributes("        ", "LogicalFunction", &function.attributes);
            self.out.push_str("    }\n");
        }
        if any {
            self.out.push('\n');
        }
    }

    fn emit_system_function(&mut self, function: &SystemFunction) {
        let id = function.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&function.id).to_string();
        let def = self.defs.get(&format!("action:{}", id), &function.name);
        self.action_def.insert(id.clone(), def.clone());
        self.action_def.insert(function.name.clone(), def.clone());
        let usage = self.usages.get(&format!("action-usage:{}", id), &format!("a_{}", id));
        self.action_usage.insert(id.clone(), usage.clone());
        self.action_usage.insert(function.name.clone(), usage.clone());

        let special = self.specialization("SystemFunction", &function.attributes);
        self.out.push_str(&format!("    action def <{}> {}{} {{\n", sysml_name(&id), def, special));
        self.out.push_str(&doc_line("        ", &function.name));
        for port in &function.ports {
            let direction = match port.direction {
                PortDirection::In => "in",
                PortDirection::Out => "out",
                PortDirection::InOut => "inout",
            };
            let item_type = item_type_name(&port.data_type);
            self.out.push_str(&format!("        {} item {} : {};\n", direction, ident(&port.name), item_type));
        }
        self.emit_attributes("        ", "SystemFunction", &function.attributes);
        for sub in &function.sub_functions {
            let sub_id = sub.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&sub.id).to_string();
            let sub_def = self.defs.get(&format!("action:{}", sub_id), &sub.name);
            self.out.push_str(&format!("        action {} : {};\n", ident(&format!("sub_{}", sub_id)), sub_def));
        }
        self.out.push_str("    }\n");
        self.out.push_str(&format!("    action {} : {};\n", usage, def));
        for sub in &function.sub_functions {
            self.emit_system_function(sub);
        }
    }

    fn emit_chains(&mut self) {
        let chains: Vec<(&FunctionalChain, &str)> = self
            .ast
            .system_analysis
            .iter()
            .flat_map(|sa| sa.functional_chains.iter().map(|c| (c, "FunctionalChain")))
            .chain(self.ast.logical_architecture.iter().flat_map(|la| la.functional_chains.iter().map(|c| (c, "FunctionalChain"))))
            .chain(self.ast.operational_analysis.iter().flat_map(|oa| oa.processes.iter().map(|c| (c, "OperationalProcess"))))
            .collect();
        if chains.is_empty() {
            return;
        }
        self.out.push_str("    // ---- Functional chains ----\n");
        for (chain, kind) in chains {
            let def = self.defs.get(&format!("chain:{}", chain.id), &format!("chain_{}", chain.name));
            self.out.push_str(&format!("    action def <{}> {} {{\n", sysml_name(&chain.id), def));
            self.out.push_str(&doc_line("        ", &chain.name));
            let mut step = 0;
            for involved in &chain.involves {
                if let Some(action) = self.action_def.get(involved).cloned() {
                    step += 1;
                    let keyword = if step == 1 { "" } else { "then " };
                    self.out.push_str(&format!("        {}action step{} : {};\n", keyword, step, action));
                } else {
                    self.out.push_str(&format!("        // involves {} (exchange)\n", involved));
                }
            }
            self.emit_attributes("        ", kind, &chain.attributes);
            self.out.push_str("    }\n");
            // A usage, so constraints can reference the chain's attributes.
            let usage = self.usages.get(&format!("action-usage:{}", chain.id), &format!("a_{}", chain.id));
            self.out.push_str(&format!("    action {} : {};\n", usage, def));
            self.action_usage.entry(chain.id.clone()).or_insert_with(|| usage.clone());
            self.action_usage.entry(chain.name.clone()).or_insert_with(|| usage.clone());
        }
        self.out.push('\n');
    }

    // ---- ports & components ----------------------------------------------

    fn emit_ports(&mut self) {
        for la in &self.ast.logical_architecture {
            for component in &la.components {
                collect_port_defs(component, &mut self.port_defs);
            }
        }
        for ty in &self.ast.types {
            for port in &ty.ports {
                self.port_defs.insert(format!("{}|{}", direction_keyword(&port.direction), port.interface_type));
            }
        }
        // Untyped ports: `interface_in`/`interface_out` blocks and physical
        // node ports use one generic port def per direction.
        let mut generic: BTreeSet<&str> = BTreeSet::new();
        for la in &self.ast.logical_architecture {
            for component in &la.components {
                collect_generic_port_directions(component, &mut generic);
            }
        }
        if self.ast.physical_architecture.iter().any(|pa| pa.nodes.iter().any(|n| !n.ports.is_empty()))
            || self.ast.logical_architecture.iter().any(|la| !la.interfaces.is_empty())
        {
            generic.insert("inout");
        }
        if self.port_defs.is_empty() && generic.is_empty() {
            return;
        }
        self.out.push_str("    // ---- Port definitions ----\n");
        let ports: Vec<String> = self.port_defs.iter().cloned().collect();
        for key in ports {
            let (direction, item_type) = key.split_once('|').unwrap_or(("inout", &key));
            let def = self.defs.get(&format!("port:{}", key), &port_def_name(direction, item_type));
            let item = item_type_name(item_type);
            self.out.push_str(&format!("    port def {} {{ {} item data : {}; }}\n", def, direction, item));
        }
        for direction in generic {
            let def = self.generic_port_def(direction);
            self.out.push_str(&format!("    port def {};  // untyped {} port\n", def, direction));
        }
        self.out.push('\n');
    }

    fn emit_components(&mut self) {
        let mut any = false;
        for oa in &self.ast.operational_analysis {
            for actor in &oa.actors {
                if !any {
                    self.out.push_str("    // ---- Actors & components ----\n");
                    any = true;
                }
                let id = actor
                    .id
                    .clone()
                    .or_else(|| actor.attributes.get("id").and_then(|v| v.as_string()).map(str::to_string))
                    .unwrap_or_else(|| format!("ACT-{}", actor.name.replace(' ', "-")));
                self.emit_simple_part(&id, &actor.name, "Actor", &actor.attributes, "actor");
            }
            for entity in &oa.entities {
                if !any {
                    self.out.push_str("    // ---- Actors & components ----\n");
                    any = true;
                }
                self.emit_simple_part(&entity.id, &entity.name, "OperationalEntity", &entity.attributes, "operational entity");
            }
        }
        for sa in &self.ast.system_analysis {
            for component in &sa.components {
                if !any {
                    self.out.push_str("    // ---- Actors & components ----\n");
                    any = true;
                }
                let id = component.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&component.name).to_string();
                self.emit_simple_part(&id, &component.name, "SystemComponent", &component.attributes, "system component");
            }
            for actor in &sa.external_actors {
                if !any {
                    self.out.push_str("    // ---- Actors & components ----\n");
                    any = true;
                }
                self.emit_simple_part(&actor.id, &actor.name, "SystemActor", &actor.attributes, "actor");
            }
        }
        for la in &self.ast.logical_architecture {
            for interface in &la.interfaces {
                if !any {
                    self.out.push_str("    // ---- Actors & components ----\n");
                    any = true;
                }
                let def = self.defs.get(&format!("interface:{}", interface.name), &interface.name);
                let end_port = self.generic_port_def("inout");
                self.out.push_str(&format!("    interface def {} {{\n", def));
                self.out.push_str(&doc_line("        ", &format!("{}: {} -> {}", interface.name, interface.from, interface.to)));
                self.out.push_str(&format!("        end provider : {};\n        end consumer : {};\n", end_port, end_port));
                self.emit_attributes("        ", "LogicalInterface", &interface.attributes);
                self.out.push_str("    }\n");
            }
            for component in &la.components {
                if !any {
                    self.out.push_str("    // ---- Actors & components ----\n");
                    any = true;
                }
                self.emit_logical_component(component, None);
            }
        }
        if any {
            self.out.push('\n');
        }
    }

    fn emit_simple_part(&mut self, id: &str, name: &str, kind: &str, attributes: &HashMap<String, AttributeValue>, role: &str) {
        let def = self.defs.get(&format!("part:{}", id), name);
        let usage = self.usages.get(&format!("part-usage:{}", id), &format!("p_{}", id));
        self.usage_path.insert(id.to_string(), usage.clone());
        self.usage_path.entry(name.to_string()).or_insert_with(|| usage.clone());
        self.part_def.insert(id.to_string(), def.clone());
        self.part_def.entry(name.to_string()).or_insert_with(|| def.clone());
        let special = self.specialization(kind, attributes);
        self.out.push_str(&format!("    part def <{}> {}{} {{\n", sysml_name(id), def, special));
        self.out.push_str(&doc_line("        ", &format!("{} ({})", name, role)));
        self.emit_attributes("        ", kind, attributes);
        self.out.push_str("    }\n");
        self.out.push_str(&format!("    part {} : {};\n", usage, def));
    }

    /// Definitions are emitted at package level (flat); usages are nested
    /// so that paths like `p_parent.p_child.port` resolve.
    fn emit_logical_component(&mut self, component: &LogicalComponent, parent_path: Option<&str>) {
        let id = component.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&component.name).to_string();
        let def = self.defs.get(&format!("part:{}", id), &component.name);
        let usage = self.usages.get(&format!("part-usage:{}", id), &format!("p_{}", id));
        let path = match parent_path {
            Some(parent) => format!("{}.{}", parent, usage),
            None => usage.clone(),
        };
        self.part_def.insert(id.clone(), def.clone());
        self.part_def.entry(component.name.clone()).or_insert_with(|| def.clone());
        self.usage_path.insert(id.clone(), path.clone());
        self.usage_path.entry(component.name.clone()).or_insert_with(|| path.clone());

        // Sub-component definitions first (a part def references them).
        for sub in &component.sub_components {
            self.emit_logical_component(sub, Some(&path));
        }

        let special = self.specialization("LogicalComponent", &component.attributes);
        self.out.push_str(&format!("    part def <{}> {}{} {{\n", sysml_name(&id), def, special));
        self.out.push_str(&doc_line("        ", &format!("{} (logical component)", component.name)));
        self.emit_attributes("        ", "LogicalComponent", &component.attributes);
        // Ports inherited as declared by the type(s) are not repeated; a port
        // the component declares differently is its own.
        let inherited_ports: Vec<ComponentPort> = super::types::declared_types(&component.attributes)
            .ok()
            .and_then(|names| super::types::inherit(&self.types, &names).ok())
            .map(|inherited| inherited.ports)
            .unwrap_or_default();
        for port in &component.ports {
            if inherited_ports
                .iter()
                .any(|p| p.name == port.name && p.direction == port.direction && p.interface_type == port.interface_type)
            {
                continue;
            }
            let direction = match port.direction {
                PortDirection::In => "in",
                PortDirection::Out => "out",
                PortDirection::InOut => "inout",
            };
            let key = format!("{}|{}", direction, port.interface_type);
            let port_def = self.defs.get(&format!("port:{}", key), &port_def_name(direction, &port.interface_type));
            self.out.push_str(&format!("        port {} : {};\n", ident(&port.name), port_def));
        }
        // Interface ports keep their ArcLang name so that exchange endpoints
        // (`LC-001.RadarDataProvider`) resolve to `p_LC_001.RadarDataProvider`.
        let in_names: BTreeSet<String> = component.interfaces_in.iter().map(|i| ident(&i.name)).collect();
        let out_names: BTreeSet<String> = component.interfaces_out.iter().map(|i| ident(&i.name)).collect();
        for interface in &component.interfaces_in {
            let port_def = self.generic_port_def("in");
            let name = ident(&interface.name);
            let name = if out_names.contains(&name) { format!("{}_in", name) } else { name };
            self.out.push_str(&format!("        port {} : {};  // interface_in{}\n", name, port_def, protocol_note(interface)));
        }
        for interface in &component.interfaces_out {
            let port_def = self.generic_port_def("out");
            let name = ident(&interface.name);
            let name = if in_names.contains(&name) { format!("{}_out", name) } else { name };
            self.out.push_str(&format!("        port {} : {};  // interface_out{}\n", name, port_def, protocol_note(interface)));
        }
        for function in &component.functions {
            let function_id = function.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&function.name).to_string();
            if let Some(action) = self.action_def.get(&function_id).or_else(|| self.action_def.get(&function.name)).cloned() {
                let performed = ident(&format!("perform_{}", function.name));
                self.out.push_str(&format!("        perform action {} : {};\n", performed, action));
                let performed_path = format!("{}.{}", path, performed);
                self.action_usage.entry(function_id.clone()).or_insert_with(|| performed_path.clone());
                self.action_usage.entry(function.name.clone()).or_insert_with(|| performed_path.clone());
            }
        }
        for sub in &component.sub_components {
            let sub_id = sub.attributes.get("id").and_then(|v| v.as_string()).unwrap_or(&sub.name).to_string();
            let sub_def = self.defs.get(&format!("part:{}", sub_id), &sub.name);
            let sub_usage = self.usages.get(&format!("part-usage:{}", sub_id), &format!("p_{}", sub_id));
            let multiplicity = multiplicity_suffix("LogicalComponent", &sub.attributes);
            self.out.push_str(&format!("        part {} : {}{};\n", sub_usage, sub_def, multiplicity));
        }
        self.out.push_str("    }\n");
        if parent_path.is_none() {
            let multiplicity = multiplicity_suffix("LogicalComponent", &component.attributes);
            self.out.push_str(&format!("    part {} : {}{};\n", usage, def, multiplicity));
        }
    }

    fn generic_port_def(&mut self, direction: &str) -> String {
        self.defs.get(&format!("port:{}|*", direction), &format!("{}_Port", capitalize(direction)))
    }

    // ---- physical --------------------------------------------------------

    fn emit_physical(&mut self) {
        let has_nodes = self.ast.physical_architecture.iter().any(|pa| !pa.nodes.is_empty());
        if !has_nodes {
            return;
        }
        // Generic port defs may have been requested by logical interfaces.
        self.out.push_str("    // ---- Physical nodes ----\n");
        let mut allocations: Vec<(String, String)> = Vec::new();
        for pa in &self.ast.physical_architecture {
            for node in &pa.nodes {
                let def = self.defs.get(&format!("part:{}", node.id), &node.name);
                let usage = self.usages.get(&format!("part-usage:{}", node.id), &format!("p_{}", node.id));
                self.usage_path.insert(node.id.clone(), usage.clone());
                self.usage_path.entry(node.name.clone()).or_insert_with(|| usage.clone());
                self.part_def.insert(node.id.clone(), def.clone());
                self.part_def.entry(node.name.clone()).or_insert_with(|| def.clone());
                let special = self.specialization("PhysicalNode", &node.attributes);
                self.out.push_str(&format!("    part def <{}> {}{} {{\n", sysml_name(&node.id), def, special));
                self.out.push_str(&doc_line("        ", &format!("{} (physical node)", node.name)));
                self.emit_attributes("        ", "PhysicalNode", &node.attributes);
                for port in &node.ports {
                    let port_def = self.generic_port_def("inout");
                    self.out.push_str(&format!("        port {} : {};\n", ident(&port.name), port_def));
                }
                for hardware in &node.hardware_components {
                    let hw_def = self.defs.get(&format!("part:{}", hardware.id), &hardware.name);
                    self.out.push_str(&format!("        part {} : {};  // hardware: {}\n", ident(&format!("hw_{}", hardware.id)), hw_def, hardware.hw_type));
                }
                for behavior in &node.behavior_components {
                    let bc_def = self.defs.get(&format!("part:{}", behavior.id), &behavior.name);
                    self.out.push_str(&format!("        part {} : {};  // behaviour component\n", ident(&format!("bc_{}", behavior.id)), bc_def));
                }
                self.out.push_str("    }\n");
                let multiplicity = multiplicity_suffix("PhysicalNode", &node.attributes);
                self.out.push_str(&format!("    part {} : {}{};\n", usage, def, multiplicity));
                for hardware in &node.hardware_components {
                    let hw_def = self.defs.get(&format!("part:{}", hardware.id), &hardware.name);
                    self.out.push_str(&format!("    part def <{}> {} {{ {} }}\n", sysml_name(&hardware.id), hw_def, doc_line("", &format!("{} ({})", hardware.name, hardware.hw_type)).trim_end()));
                }
                for behavior in &node.behavior_components {
                    let bc_def = self.defs.get(&format!("part:{}", behavior.id), &behavior.name);
                    self.out.push_str(&format!("    part def <{}> {} {{ {} }}\n", sysml_name(&behavior.id), bc_def, doc_line("", &behavior.name).trim_end()));
                }
                for deployment in &node.deployments {
                    allocations.push((deployment.component.clone(), node.id.clone()));
                }
            }
        }
        for (component, node) in allocations {
            if let (Some(source), Some(target)) = (self.resolve_usage(&component), self.usage_path.get(&node).cloned()) {
                self.out.push_str(&format!("    allocate {} to {};\n", source, target));
            } else {
                self.out.push_str(&format!("    // deployment {} -> {} (unresolved)\n", component, node));
            }
        }
        self.out.push('\n');
    }

    // ---- exchanges -------------------------------------------------------

    fn emit_exchanges(&mut self) {
        let mut lines: Vec<String> = Vec::new();
        for sa in &self.ast.system_analysis {
            for exchange in &sa.functional_exchanges {
                let from = self.resolve_action_endpoint(&exchange.from_port);
                let to = self.resolve_action_endpoint(&exchange.to_port);
                match (from, to) {
                    (Some(from), Some(to)) => {
                        let label = exchange.label.clone().unwrap_or_default();
                        lines.push(flow_or_connect(&from, &to, &label));
                    }
                    _ => lines.push(format!("    // functional exchange {} -> {} (unresolved endpoint)", exchange.from_port, exchange.to_port)),
                }
            }
        }
        for la in &self.ast.logical_architecture {
            for interface in &la.interfaces {
                match (self.resolve_usage(&interface.from), self.resolve_usage(&interface.to)) {
                    (Some(from), Some(to)) => lines.push(format!("    connect {} to {};{}", from, to, comment(&format!("interface {}", interface.name)))),
                    _ => lines.push(format!("    // interface {}: {} -> {} (unresolved endpoint)", interface.name, interface.from, interface.to)),
                }
            }
            for exchange in &la.component_exchanges {
                let from = self.resolve_port_endpoint(&exchange.from_port);
                let to = self.resolve_port_endpoint(&exchange.to_port);
                match (from, to) {
                    (Some(from), Some(to)) => {
                        let label = exchange.label.clone().unwrap_or_default();
                        let item = if exchange.exchange_item.is_empty() { String::new() } else { format!(" [{}]", exchange.exchange_item) };
                        lines.push(format!("    connect {} to {};{}", from, to, comment(&format!("{}{}", label, item))));
                    }
                    _ => lines.push(format!("    // component exchange {} -> {} (unresolved endpoint)", exchange.from_port, exchange.to_port)),
                }
            }
        }
        for pa in &self.ast.physical_architecture {
            for link in &pa.links {
                let from = self.resolve_usage(&link.from);
                let to = self.resolve_usage(&link.to);
                match (from, to) {
                    // A named link is a connection usage carrying its typed
                    // attributes, so constraints can reference them.
                    (Some(from), Some(to)) if !link.name.is_empty() => {
                        let usage = self.usages.get(&format!("link:{}", link.name), &link.name);
                        self.usage_path.entry(link.name.clone()).or_insert_with(|| usage.clone());
                        let mut attributes = link.attributes.clone();
                        if let Some(bandwidth) = &link.bandwidth {
                            attributes.entry("bandwidth".to_string()).or_insert_with(|| AttributeValue::String(bandwidth.clone()));
                        }
                        attributes.entry("protocol".to_string()).or_insert_with(|| AttributeValue::String(link.protocol.clone()));
                        // A usage is TYPED by its definition (`: Def`).
                        let typing = self.specialization("PhysicalLink", &attributes).replace(" :> ", " : ");
                        let before = self.out.len();
                        self.emit_attributes("        ", "PhysicalLink", &attributes);
                        let body = self.out.split_off(before);
                        lines.push(format!("    connection {}{} connect {} to {} {{\n{}    }}", usage, typing, from, to, body));
                    }
                    (Some(from), Some(to)) => {
                        let note = format!("link [{}{}]", link.protocol, link.bandwidth.as_deref().map(|b| format!(", {}", b)).unwrap_or_default());
                        lines.push(format!("    connect {} to {};{}", from, to, comment(&note)));
                    }
                    _ => lines.push(format!("    // physical link {} -> {} (unresolved endpoint)", link.from, link.to)),
                }
            }
            for exchange in &pa.physical_exchanges {
                let from = self.resolve_usage(&exchange.from);
                let to = self.resolve_usage(&exchange.to);
                match (from, to) {
                    (Some(from), Some(to)) => {
                        let note = format!("{} [{}{}]", exchange.label.clone().unwrap_or_default(), exchange.message_type, exchange.via.as_deref().map(|v| format!(" via {}", v)).unwrap_or_default());
                        lines.push(flow_or_connect(&from, &to, &note));
                    }
                    _ => lines.push(format!("    // physical exchange {} -> {} (unresolved endpoint)", exchange.from, exchange.to)),
                }
            }
        }
        if lines.is_empty() {
            return;
        }
        self.out.push_str("    // ---- Exchanges ----\n");
        for line in lines {
            self.out.push_str(&line);
            self.out.push('\n');
        }
        self.out.push('\n');
    }

    /// `Comp.port` → `p_comp.port`; `Comp` → `p_comp`.
    fn resolve_port_endpoint(&self, endpoint: &str) -> Option<String> {
        if let Some(path) = self.usage_path.get(endpoint) {
            return Some(path.clone());
        }
        let (owner, port) = endpoint.rsplit_once('.')?;
        let base = self.resolve_usage(owner)?;
        Some(format!("{}.{}", base, ident(port)))
    }

    fn resolve_action_endpoint(&self, endpoint: &str) -> Option<String> {
        if let Some(usage) = self.action_usage.get(endpoint) {
            return Some(usage.clone());
        }
        let (owner, port) = endpoint.rsplit_once('.')?;
        let usage = self.action_usage.get(owner)?;
        Some(format!("{}.{}", usage, ident(port)))
    }

    /// Resolve an element id, name, or dotted path to a part usage path.
    fn resolve_usage(&self, endpoint: &str) -> Option<String> {
        let root = endpoint.split('.').next().unwrap_or(endpoint);
        for candidate in [endpoint, root] {
            if let Some(path) = self.usage_path.get(candidate) {
                return Some(path.clone());
            }
            if let Some(component) = self.semantic.components.iter().find(|c| c.name == candidate) {
                if let Some(path) = self.usage_path.get(&component.id) {
                    return Some(path.clone());
                }
            }
        }
        None
    }

    // ---- behaviour, verification, traces ---------------------------------

    fn emit_state_machines(&mut self) {
        if self.ast.state_machines.is_empty() {
            return;
        }
        self.out.push_str("    // ---- Modes & states ----\n");
        for machine in &self.ast.state_machines {
            let def = self.defs.get(&format!("state:{}", machine.name), &machine.name);
            self.out.push_str(&format!("    state def {} {{\n", def));
            let mut state_names: BTreeMap<&str, String> = BTreeMap::new();
            for state in &machine.states {
                state_names.insert(state.name.as_str(), ident(&state.name));
            }
            if let Some(initial) = state_names.get(machine.initial_state.as_str()) {
                self.out.push_str(&format!("        entry; then {};\n", initial));
            }
            for state in &machine.states {
                let kind = match state.kind { StateKind::Mode => "mode", StateKind::State => "state" };
                self.out.push_str(&format!("        state {};  // {}\n", state_names[state.name.as_str()], kind));
            }
            for (index, transition) in machine.transitions.iter().enumerate() {
                let (Some(from), Some(to)) = (state_names.get(transition.from.as_str()), state_names.get(transition.to.as_str())) else { continue };
                let mut note = format!("trigger: {}", transition.trigger);
                if let Some(guard) = &transition.guard { note.push_str(&format!("; guard: {}", guard)); }
                if let Some(action) = &transition.action { note.push_str(&format!("; action: {}", action)); }
                if let Some(timing) = &transition.timing { note.push_str(&format!("; timing: {}", timing)); }
                self.out.push_str(&format!("        transition t{} first {} then {};{}\n", index + 1, from, to, comment(&note)));
            }
            self.out.push_str("    }\n");
        }
        self.out.push('\n');
    }

    /// A scenario is an occurrence definition: each participant is a
    /// referenced part with one event per message end, and each message
    /// connects a send event to a receive event, in declaration order.
    fn emit_scenarios(&mut self) {
        if self.ast.scenarios.is_empty() {
            return;
        }
        self.out.push_str("    // ---- Scenarios ----\n");
        for scenario in &self.ast.scenarios {
            let def = self.defs.get(&format!("scenario:{}", scenario.name), &format!("scenario_{}", scenario.name));
            self.out.push_str(&format!("    occurrence def {} {{\n", def));
            self.out.push_str(&doc_line("        ", &scenario.name));

            let mut lifelines: BTreeMap<String, String> = BTreeMap::new();
            let mut local = Names::default();
            for participant in &scenario.participants {
                for key in [&participant.id, &participant.name] {
                    if !lifelines.contains_key(key) {
                        // Prefixed: a lifeline named like its definition would shadow it.
                        let lifeline = local.get(&participant.id, &format!("l_{}", participant.name));
                        lifelines.insert(key.clone(), lifeline);
                    }
                }
            }
            // Events per lifeline, in message order.
            let mut events: BTreeMap<String, Vec<String>> = BTreeMap::new();
            let mut messages = Vec::new();
            for (index, message) in scenario.messages.iter().enumerate() {
                let (Some(from), Some(to)) = (lifelines.get(&message.from), lifelines.get(&message.to)) else {
                    messages.push(format!("        // message {} -> {} (participant not declared)", message.from, message.to));
                    continue;
                };
                let n = index + 1;
                let sent = format!("m{}_sent", n);
                let received = format!("m{}_received", n);
                events.entry(from.clone()).or_default().push(sent.clone());
                events.entry(to.clone()).or_default().push(received.clone());
                let kind = match message.message_type {
                    MessageType::Synchronous => "sync",
                    MessageType::Asynchronous => "async",
                    MessageType::Return => "return",
                };
                let mut note = format!("{} [{}]", message.label, kind);
                if let Some(timing) = &message.timing {
                    note.push_str(&format!(" timing: {}", timing));
                }
                messages.push(format!("        message m{} from {}.{} to {}.{};{}", n, from, sent, to, received, comment(&note)));
            }
            let mut emitted: BTreeSet<&String> = BTreeSet::new();
            for participant in &scenario.participants {
                let lifeline = &lifelines[&participant.id];
                if !emitted.insert(lifeline) {
                    continue;
                }
                let typing = self
                    .part_def
                    .get(&participant.id)
                    .or_else(|| self.part_def.get(&participant.name))
                    .map(|def| format!(" : {}", def))
                    .unwrap_or_default();
                match events.get(lifeline) {
                    Some(lifeline_events) => {
                        self.out.push_str(&format!("        ref part {}{} {{\n", lifeline, typing));
                        for (index, event) in lifeline_events.iter().enumerate() {
                            let keyword = if index == 0 { "" } else { "then " };
                            self.out.push_str(&format!("            {}event occurrence {};\n", keyword, event));
                        }
                        self.out.push_str("        }\n");
                    }
                    None => self.out.push_str(&format!("        ref part {}{};\n", lifeline, typing)),
                }
            }
            for line in messages {
                self.out.push_str(&line);
                self.out.push('\n');
            }
            self.out.push_str("    }\n");
        }
        self.out.push('\n');
    }

    fn emit_verification(&mut self) {
        if self.ast.test_cases.is_empty() {
            return;
        }
        self.out.push_str("    // ---- Verification ----\n");
        for test_case in &self.ast.test_cases {
            let id = if test_case.id.is_empty() { test_case.name.clone() } else { test_case.id.clone() };
            let def = self.defs.get(&format!("verification:{}", id), &format!("verify_{}", test_case.name));
            self.out.push_str(&format!("    verification def <{}> {} {{\n", sysml_name(&id), def));
            self.out.push_str(&doc_line("        ", &format!("{} (method: {})", test_case.name, test_case.method)));
            self.emit_attributes("        ", "TestCase", &test_case.attributes);
            self.out.push_str("        objective {\n");
            for requirement in &test_case.verifies {
                match self.requirement_usage.get(requirement) {
                    Some(usage) => self.out.push_str(&format!("            verify {};\n", usage)),
                    None => self.out.push_str(&format!("            // verifies unknown requirement {}\n", requirement)),
                }
            }
            self.out.push_str("        }\n");
            self.out.push_str("    }\n");
        }
        self.out.push('\n');
    }

    fn emit_constraints(&mut self) {
        if self.ast.constraints.is_empty() {
            return;
        }
        let scope = super::constraint::Scope::from_model(self.ast);
        self.out.push_str("    // ---- Constraints ----\n");
        for constraint in &self.ast.constraints {
            let name = self.usages.get(&format!("constraint:{}", constraint.id), &constraint.name);
            let description = constraint.attributes.get("description").and_then(|v| v.as_string());
            let title = match description {
                Some(description) => format!("{} — {}", constraint.name, description),
                None => constraint.name.clone(),
            };
            match self.expression(&constraint.expression, &scope, true) {
                Ok(expression) => {
                    self.out.push_str(&format!("    assert constraint <{}> {} {{\n", sysml_name(&constraint.id), name));
                    self.out.push_str(&doc_line("        ", &title));
                    self.out.push_str(&format!("        {}\n", expression));
                    self.out.push_str("    }\n");
                }
                Err(reason) => {
                    self.out.push_str(&format!("    constraint def <{}> {} {{\n", sysml_name(&constraint.id), name));
                    self.out.push_str(&doc_line("        ", &format!("{}: {} (expression not exported: {})", title, constraint.expression, reason)));
                    self.out.push_str("    }\n");
                }
            }
        }
        self.out.push('\n');
    }

    /// Effective attributes of the type an element's definition specializes,
    /// when that element kind is exported with a specialization.
    fn inherited_attributes(&self, kind: &str, attributes: &HashMap<String, AttributeValue>) -> Option<HashMap<String, AttributeValue>> {
        Category::of(kind)?;
        let names = super::types::declared_types(attributes).ok()?;
        if names.is_empty() {
            return None;
        }
        // Attributes the types disagree on are always redefined by the
        // element (the compiler requires it), so any inherited value works
        // as "differs from the element's".
        super::types::inherit(&self.types, &names).ok().map(|inherited| inherited.attributes)
    }

    fn type_def_name(&mut self, type_name: &str, category: Category) -> String {
        self.defs.get(&format!("type:{:?}:{}", category, type_name), type_name)
    }

    /// ` :> TypeDef` for an element typed with `is:`, or nothing.
    fn specialization(&mut self, kind: &str, attributes: &HashMap<String, AttributeValue>) -> String {
        let Some(category) = Category::of(kind) else { return String::new() };
        let names: Vec<String> = match super::types::declared_types(attributes) {
            Ok(names) => names.into_iter().filter(|name| self.types.contains_key(*name)).map(str::to_string).collect(),
            Err(_) => Vec::new(),
        };
        if names.is_empty() {
            return String::new();
        }
        let mut defs = Vec::with_capacity(names.len());
        for name in names {
            defs.push(self.type_def_name(&name, category));
            self.type_requests.insert((name, category));
        }
        format!(" :> {}", defs.join(", "))
    }

    /// Emit the user-defined types as abstract definitions. A type is a
    /// part def when it types parts, an action def when it types functions
    /// (both if both), a connection def when it types links; a type nothing
    /// uses is still exported, as a part def.
    fn emit_types(&mut self) {
        if self.ast.types.is_empty() {
            return;
        }
        let mut requests = self.type_requests.clone();
        // A specialization needs its bases in the same category; a type
        // nothing uses (directly or through a subtype) defaults to a part def.
        for pass in 0..2 {
            for (name, category) in requests.clone() {
                if let Some(effective) = self.types.get(&name) {
                    for ancestor in &effective.ancestors {
                        requests.insert((ancestor.clone(), category));
                    }
                }
            }
            if pass == 0 {
                for ty in &self.ast.types {
                    if !requests.iter().any(|(name, _)| name == &ty.name) {
                        requests.insert((ty.name.clone(), Category::Part));
                    }
                }
            }
        }
        self.out.push_str("    // ---- Types ----\n");
        let declared: BTreeMap<&str, &TypeDef> = self.ast.types.iter().map(|t| (t.name.as_str(), t)).collect();
        for (name, category) in requests {
            let Some(ty) = declared.get(name.as_str()).copied() else { continue };
            let def = self.type_def_name(&name, category);
            let base = match &ty.extends {
                Some(base) => format!(" :> {}", self.type_def_name(base, category)),
                None => String::new(),
            };
            let (keyword, kind) = match category {
                Category::Part => ("part", "LogicalComponent"),
                Category::Action => ("action", "SystemFunction"),
                Category::Connection => ("connection", "PhysicalLink"),
            };
            self.out.push_str(&format!("    abstract {} def {}{} {{\n", keyword, def, base));
            let required = if ty.required.is_empty() {
                String::new()
            } else {
                format!(" — instances must provide: {}", ty.required.join(", "))
            };
            self.out.push_str(&doc_line("        ", &format!("type {}{}", ty.name, required)));
            // Own attributes; `is` stands for the base so that overridden
            // inherited attributes come out as redefinitions.
            let mut attributes = ty.attributes.clone();
            if let Some(base) = &ty.extends {
                attributes.insert("is".to_string(), AttributeValue::String(base.clone()));
            }
            if category == Category::Connection && ty.extends.is_none() {
                self.out.push_str("        end source;\n        end target;\n");
            }
            self.emitting_defaults = true;
            self.emit_attributes("        ", kind, &attributes);
            self.emitting_defaults = false;
            if category == Category::Part {
                let inherited_ports: BTreeSet<String> = ty
                    .extends
                    .as_ref()
                    .and_then(|base| self.types.get(base))
                    .map(|base| base.ports.iter().map(|p| p.name.clone()).collect())
                    .unwrap_or_default();
                for port in &ty.ports {
                    let direction = direction_keyword(&port.direction);
                    let key = format!("{}|{}", direction, port.interface_type);
                    let port_def = self.defs.get(&format!("port:{}", key), &port_def_name(direction, &port.interface_type));
                    let redefinition = if inherited_ports.contains(&port.name) { ":>> " } else { "" };
                    self.out.push_str(&format!("        port {}{} : {};\n", redefinition, ident(&port.name), port_def));
                }
            }
            self.out.push_str("    }\n");
        }
        self.out.push('\n');
    }

    fn dependency_end(&self, element: &str, path: String) -> Option<String> {
        if !path.contains('.') {
            return Some(path);
        }
        self.action_def.get(element).or_else(|| self.part_def.get(element)).cloned()
    }

    /// Feature path of an element usage, for attribute references.
    fn element_path(&self, element: &str) -> Result<String, String> {
        self.action_usage
            .get(element)
            .cloned()
            .or_else(|| self.resolve_usage(element))
            .or_else(|| self.requirement_usage.get(element).cloned())
            .ok_or_else(|| format!("no exported usage for '{}'", element))
    }

    /// ArcLang expression → SysML v2 expression over the exported usages.
    /// Aggregates over a chain are expanded to their members.
    fn expression(&mut self, expr: &Expr, scope: &super::constraint::Scope, top: bool) -> Result<String, String> {
        Ok(match expr {
            Expr::Number(n) => number_literal(*n),
            Expr::Quantity(quantity) => {
                self.units_used.insert(quantity.spec().sysml);
                quantity_literal(quantity)
            }
            Expr::Attr { element, attribute } => format!("{}.{}", self.element_path(element)?, ident(attribute)),
            Expr::Ref(name) => return Err(format!("'{}' is not a value", name)),
            Expr::Neg(inner) => format!("-({})", self.expression(inner, scope, false)?),
            Expr::Binary { op, lhs, rhs } => {
                let text = format!(
                    "{} {} {}",
                    self.expression(lhs, scope, false)?,
                    op.symbol(),
                    self.expression(rhs, scope, false)?
                );
                if top { text } else { format!("({})", text) }
            }
            Expr::Call { function, args } => {
                let collection = match args.first() {
                    Some(Expr::Ref(name)) => name,
                    _ => return Err(format!("{}() without a collection", function)),
                };
                let members = scope.operand_members(collection)?;
                if function == "count" {
                    return Ok(members.len().to_string());
                }
                let attribute = match args.get(1) {
                    Some(Expr::Ref(name)) => ident(name),
                    _ => return Err(format!("{}() without an attribute", function)),
                };
                let mut paths = Vec::new();
                for member in members {
                    paths.push(format!("{}.{}", self.element_path(member)?, attribute));
                }
                match function.as_str() {
                    "sum" => format!("({})", paths.join(" + ")),
                    "min" | "max" => {
                        let mut folded = paths[0].clone();
                        for path in &paths[1..] {
                            folded = format!("DataFunctions::{}({}, {})", function, folded, path);
                        }
                        folded
                    }
                    other => return Err(format!("unknown function '{}'", other)),
                }
            }
        })
    }

    fn emit_traces(&mut self) {
        let traces: Vec<&Trace> = self
            .ast
            .traces
            .iter()
            .chain(self.ast.operational_analysis.iter().flat_map(|oa| oa.traces.iter()))
            .collect();
        if traces.is_empty() {
            return;
        }
        self.out.push_str("    // ---- Traceability ----\n");
        for trace in traces {
            let rationale = trace.attributes.get("rationale").and_then(|v| v.as_string()).unwrap_or("");
            let source = self.resolve_usage(&trace.from).or_else(|| self.action_usage.get(&trace.from).cloned());
            match trace.trace_type.as_str() {
                "satisfies" => match (self.requirement_usage.get(&trace.to), source) {
                    (Some(requirement), Some(source)) => {
                        self.out.push_str(&format!("    satisfy {} by {};{}\n", requirement, source, comment(rationale)));
                    }
                    _ => self.out.push_str(&format!("    // {} satisfies {} (unresolved)\n", trace.from, trace.to)),
                },
                kind => {
                    let target = self
                        .resolve_usage(&trace.to)
                        .or_else(|| self.action_usage.get(&trace.to).cloned())
                        .or_else(|| self.requirement_usage.get(&trace.to).cloned());
                    // A dependency relates named elements, not feature
                    // chains: a nested usage is represented by its definition.
                    let source = source.and_then(|path| self.dependency_end(&trace.from, path));
                    let target = target.and_then(|path| self.dependency_end(&trace.to, path));
                    match (source, target) {
                        (Some(source), Some(target)) => {
                            self.out.push_str(&format!("    dependency {} from {} to {};{}\n", ident(kind), source, target, comment(rationale)));
                        }
                        _ => self.out.push_str(&format!("    // {} {} {} (unresolved)\n", trace.from, kind, trace.to)),
                    }
                }
            }
        }
    }
}

impl<'a> Generator<'a> {
    /// Units of the ArcLang table that the SysML v2 `SI` library does not
    /// declare (most prefixed units) are declared locally, the way `SI`
    /// itself declares `km` or `kg`, so every `[unit]` resolves.
    fn emit_units(&mut self) {
        let declarations: Vec<&'static str> = self
            .units_used
            .iter()
            .filter_map(|symbol| local_unit_declaration(symbol))
            .collect();
        if declarations.is_empty() {
            return;
        }
        self.out.push_str("\n    // ---- Units not predeclared by the SI library ----\n");
        self.out.push_str("    package ArcLangUnits {\n");
        self.out.push_str("        private import MeasurementReferences::*;\n");
        self.out.push_str("        private import ISQ::*;\n");
        self.out.push_str("        private import SI::*;\n");
        for declaration in declarations {
            self.out.push_str("        ");
            self.out.push_str(declaration);
            self.out.push('\n');
        }
        self.out.push_str("    }\n");
        self.out.push_str("    private import ArcLangUnits::*;\n");
    }
}

/// A SysML v2 package that uses every standard-library element the exporter
/// can write: scalar and quantity value types, units, the unit declarations
/// of `ArcLangUnits`, and the aggregate functions. It exists to be given to
/// the OMG pilot implementation, which says what each name designates
/// (`tools/sysml_library_index.py` → `spec/sysml_library_index.json`).
pub fn library_probe() -> String {
    let mut out = String::from("package ArcLangLibraryProbe {\n");
    for import in ["ScalarValues", "MeasurementReferences", "ISQ", "SI"] {
        out.push_str(&format!("    private import {}::*;\n", import));
    }
    let mut types: BTreeSet<&str> = ["String", "Integer", "Real", "Boolean"].into_iter().collect();
    let mut units: BTreeSet<&str> = BTreeSet::new();
    for unit in super::quantity::UNITS {
        types.insert(unit.dimension.sysml_value_type());
        units.insert(unit.sysml);
    }
    for (index, name) in types.iter().enumerate() {
        out.push_str(&format!("    attribute type{} : {};\n", index + 1, name));
    }
    for (index, unit) in units.iter().filter(|unit| local_unit_declaration(unit).is_none()).enumerate() {
        out.push_str(&format!("    attribute unit{} = 1 [{}];\n", index + 1, unit));
    }
    for (index, function) in ["max", "min"].iter().enumerate() {
        out.push_str(&format!("    attribute call{} = DataFunctions::{}(1, 2);\n", index + 1, function));
    }
    out.push_str("    package ArcLangUnits {\n");
    for import in ["MeasurementReferences", "ISQ", "SI"] {
        out.push_str(&format!("        private import {}::*;\n", import));
    }
    for declaration in units.iter().filter_map(|unit| local_unit_declaration(unit)) {
        out.push_str(&format!("        {}\n", declaration));
    }
    out.push_str("    }\n    private import ArcLangUnits::*;\n}\n");
    out
}

/// SysML v2 declaration of a unit the `SI` library lacks, in the library's
/// own style (`ConversionByPrefix` / `ConversionByConvention`).
fn local_unit_declaration(symbol: &str) -> Option<&'static str> {
    Some(match symbol {
        "ns" => "attribute <ns> nanosecond : DurationUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = nano; :>> referenceUnit = s; } }",
        "us" => "attribute <us> microsecond : DurationUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = micro; :>> referenceUnit = s; } }",
        "ms" => "attribute <ms> millisecond : DurationUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = milli; :>> referenceUnit = s; } }",
        "kHz" => "attribute <kHz> kilohertz : FrequencyUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = kilo; :>> referenceUnit = Hz; } }",
        "MHz" => "attribute <MHz> megahertz : FrequencyUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = mega; :>> referenceUnit = Hz; } }",
        "GHz" => "attribute <GHz> gigahertz : FrequencyUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = giga; :>> referenceUnit = Hz; } }",
        "'kbit/s'" => "attribute <'kbit/s'> 'kilobit per second' : BinaryDigitRateUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = kilo; :>> referenceUnit = 'bit/s'; } }",
        "'Mbit/s'" => "attribute <'Mbit/s'> 'megabit per second' : BinaryDigitRateUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = mega; :>> referenceUnit = 'bit/s'; } }",
        "'Gbit/s'" => "attribute <'Gbit/s'> 'gigabit per second' : BinaryDigitRateUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = giga; :>> referenceUnit = 'bit/s'; } }",
        "kB" => "attribute <kB> kilobyte : StorageCapacityUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = kilo; :>> referenceUnit = B; } }",
        "MB" => "attribute <MB> megabyte : StorageCapacityUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = mega; :>> referenceUnit = B; } }",
        "GB" => "attribute <GB> gigabyte : StorageCapacityUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = giga; :>> referenceUnit = B; } }",
        "KiB" => "attribute <KiB> kibibyte : StorageCapacityUnit { :>> unitConversion: ConversionByConvention { :>> referenceUnit = B; :>> conversionFactor = 1024; } }",
        "MiB" => "attribute <MiB> mebibyte : StorageCapacityUnit { :>> unitConversion: ConversionByConvention { :>> referenceUnit = B; :>> conversionFactor = 1048576; } }",
        "GiB" => "attribute <GiB> gibibyte : StorageCapacityUnit { :>> unitConversion: ConversionByConvention { :>> referenceUnit = B; :>> conversionFactor = 1073741824; } }",
        "t" => "attribute <t> tonne : MassUnit { :>> unitConversion: ConversionByConvention { :>> referenceUnit = kg; :>> conversionFactor = 1000; } }",
        "kN" => "attribute <kN> kilonewton : ForceUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = kilo; :>> referenceUnit = N; } }",
        "mV" => "attribute <mV> millivolt : ElectricPotentialUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = milli; :>> referenceUnit = V; } }",
        "kV" => "attribute <kV> kilovolt : ElectricPotentialUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = kilo; :>> referenceUnit = V; } }",
        "mA" => "attribute <mA> milliampere : ElectricCurrentUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = milli; :>> referenceUnit = A; } }",
        "mW" => "attribute <mW> milliwatt : PowerUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = milli; :>> referenceUnit = W; } }",
        "'W*h'" => "attribute <'W*h'> 'watt hour' : EnergyUnit { :>> unitConversion: ConversionByConvention { :>> referenceUnit = J; :>> conversionFactor = 3600; } }",
        "'kW*h'" => "attribute <'kW*h'> 'kilowatt hour' : EnergyUnit { :>> unitConversion: ConversionByConvention { :>> referenceUnit = J; :>> conversionFactor = 3600000; } }",
        "kPa" => "attribute <kPa> kilopascal : PressureUnit { :>> unitConversion: ConversionByPrefix { :>> prefix = kilo; :>> referenceUnit = Pa; } }",
        "bar" => "attribute <bar> bar : PressureUnit { :>> unitConversion: ConversionByConvention { :>> referenceUnit = Pa; :>> conversionFactor = 100000; } }",
        "'%'" => "attribute <'%'> percent : DimensionOneUnit { :>> unitConversion: ConversionByConvention { :>> referenceUnit = one; :>> conversionFactor = 0.01; } }",
        _ => return None,
    })
}

// ---- helpers ---------------------------------------------------------------

/// Which SysML definition family an element kind is exported as, for the
/// kinds whose definition can specialize a user-defined type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Category {
    Part,
    Action,
    Connection,
}

impl Category {
    fn of(kind: &str) -> Option<Category> {
        match kind {
            "Actor" | "OperationalEntity" | "SystemComponent" | "SystemActor" | "LogicalComponent" | "PhysicalNode" => Some(Category::Part),
            "OperationalActivity" | "SystemFunction" | "LogicalFunction" => Some(Category::Action),
            "PhysicalLink" => Some(Category::Connection),
            _ => None,
        }
    }
}

fn direction_keyword(direction: &PortDirection) -> &'static str {
    match direction {
        PortDirection::In => "in",
        PortDirection::Out => "out",
        PortDirection::InOut => "inout",
    }
}

/// A `flow` needs feature ends in dot notation (`a.port`); an exchange
/// declared between whole elements becomes a plain `connect`.
fn flow_or_connect(from: &str, to: &str, note: &str) -> String {
    if from.contains('.') && to.contains('.') {
        format!("    flow from {} to {};{}", from, to, comment(note))
    } else {
        format!("    connect {} to {};{}", from, to, comment(&format!("{} (exchange without ports)", note).trim_start_matches(' ').to_string()))
    }
}

fn comment(text: &str) -> String {
    if text.trim().is_empty() { String::new() } else { format!("  // {}", text.replace('\n', " ")) }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn number_literal(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 { format!("{}", n as i64) } else { n.to_string() }
}

fn quantity_literal(quantity: &Quantity) -> String {
    format!("{} [{}]", number_literal(quantity.value), quantity.spec().sysml)
}

fn protocol_note(interface: &InterfaceDefinition) -> String {
    match &interface.protocol {
        Some(protocol) => format!(" ({})", protocol),
        None => String::new(),
    }
}

fn is_scalar_type(name: &str) -> bool {
    scalar_type(name).is_some()
}

fn scalar_type(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "string" | "str" | "text" => Some("String"),
        "int" | "integer" | "int32" | "int64" | "uint8" | "uint16" | "uint32" | "uint64" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" => Some("Integer"),
        "float" | "double" | "real" | "f32" | "f64" | "number" => Some("Real"),
        "bool" | "boolean" => Some("Boolean"),
        _ => None,
    }
}

/// ` [4]`, ` [1..*]`: the multiplicity of a usage, when its element states one.
fn multiplicity_suffix(kind: &str, attributes: &HashMap<String, AttributeValue>) -> String {
    Multiplicity::of(kind, attributes).map(|multiplicity| format!(" [{}]", multiplicity.canonical())).unwrap_or_default()
}

/// Item definition carried by a port that states no data type.
const UNTYPED_ITEM: &str = "Data";

fn item_type_name(data_type: &str) -> String {
    if data_type.is_empty() {
        return UNTYPED_ITEM.to_string();
    }
    scalar_type(data_type).map(str::to_string).unwrap_or_else(|| ident(data_type))
}

fn port_def_name(direction: &str, item_type: &str) -> String {
    if item_type.is_empty() {
        format!("{}_Port", capitalize(direction))
    } else {
        format!("{}_{}_Port", capitalize(direction), ident(item_type))
    }
}

fn collect_function_item_types(function: &SystemFunction, out: &mut BTreeSet<String>) {
    for port in &function.ports {
        out.insert(port.data_type.clone());
    }
    for sub in &function.sub_functions {
        collect_function_item_types(sub, out);
    }
}

fn collect_component_item_types(component: &LogicalComponent, out: &mut BTreeSet<String>) {
    for port in &component.ports {
        out.insert(port.interface_type.clone());
    }
    for sub in &component.sub_components {
        collect_component_item_types(sub, out);
    }
}

fn collect_port_defs(component: &LogicalComponent, out: &mut BTreeSet<String>) {
    for port in &component.ports {
        let direction = match port.direction {
            PortDirection::In => "in",
            PortDirection::Out => "out",
            PortDirection::InOut => "inout",
        };
        out.insert(format!("{}|{}", direction, port.interface_type));
    }
    for sub in &component.sub_components {
        collect_port_defs(sub, out);
    }
}

fn collect_activities<'m>(activity: &'m OperationalActivity, out: &mut Vec<&'m OperationalActivity>) {
    out.push(activity);
    for sub in &activity.sub_activities {
        collect_activities(sub, out);
    }
}

fn collect_generic_port_directions<'m>(component: &'m LogicalComponent, out: &mut BTreeSet<&'m str>) {
    if !component.interfaces_in.is_empty() {
        out.insert("in");
    }
    if !component.interfaces_out.is_empty() {
        out.insert("out");
    }
    for sub in &component.sub_components {
        collect_generic_port_directions(sub, out);
    }
}

fn collect_logical_functions<'m>(component: &'m LogicalComponent, out: &mut Vec<(&'m LogicalFunction, String)>) {
    for function in &component.functions {
        out.push((function, component.name.clone()));
    }
    for sub in &component.sub_components {
        collect_logical_functions(sub, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{Compiler, CompilerConfig};

    fn export(source: &str) -> String {
        let result = Compiler::new(CompilerConfig::default()).compile_string(source).expect("compiles");
        generate_sysmlv2(&result.semantic_model, &result.ast)
    }

    #[test]
    fn plain_names_stay_basic_identifiers() {
        assert_eq!(sysml_name("BrakeController"), "BrakeController");
        assert_eq!(ident("Sensor Fusion"), "Sensor_Fusion");
        assert_eq!(ident("REQ-001"), "REQ_001");
        assert_eq!(ident("2nd"), "_2nd");
    }

    #[test]
    fn names_with_spaces_or_symbols_become_unrestricted() {
        assert_eq!(sysml_name("Brake Controller"), "'Brake Controller'");
        assert_eq!(sysml_name("REQ-001"), "'REQ-001'");
        assert_eq!(sysml_name("O'Brien"), "'O\\'Brien'");
    }

    #[test]
    fn export_contains_core_constructs() {
        let sysml = export(
            r#"
model Demo {
}

requirements safety {
    req "REQ-001" "Braking" { description: "Brake on demand" priority: "High" safety_level: ASIL_D }
}

architecture logical {
    component "Brake Controller" {
        id: "LC-001"
        asil: "ASIL-D"
        function "Compute force" { latency: 25 ms }
    }
}

trace "LC-001" satisfies "REQ-001" { rationale: "direct" }
"#,
        );
        assert!(sysml.contains("package Demo {"), "{sysml}");
        assert!(sysml.contains("private import ISQ::*;"), "{sysml}");
        assert!(sysml.contains("requirement <'REQ-001'> req_REQ_001 {"), "{sysml}");
        assert!(sysml.contains("doc /* Braking — Brake on demand */"), "{sysml}");
        assert!(sysml.contains("attribute priority : String = \"High\";"), "{sysml}");
        assert!(sysml.contains("attribute safety_level : String = \"ASIL-D\";"), "normalized enum: {sysml}");
        assert!(sysml.contains("part def <'LC-001'> Brake_Controller {"), "{sysml}");
        assert!(sysml.contains("attribute asil : String = \"ASIL-D\";"), "{sysml}");
        assert!(sysml.contains("perform action perform_Compute_force : Compute_force;"), "{sysml}");
        assert!(sysml.contains("attribute latency : DurationValue = 25 [ms];"), "{sysml}");
        assert!(sysml.contains("part p_LC_001 : Brake_Controller;"), "{sysml}");
        assert!(sysml.contains("satisfy req_REQ_001 by p_LC_001;  // direct"), "{sysml}");
    }

    #[test]
    fn ports_exchanges_nesting_deployment_states_and_verification_are_exported() {
        let sysml = export(
            r#"
model Full {}
system_analysis "SA" {
    requirement "REQ-1" { description: "detect" }
    function "Detect" { id: "SF-1" port out objects { data_type: "ObjectList" } latency: 10 ms }
    function "Decide" { id: "SF-2" port in objects { data_type: "ObjectList" } latency: 0.02 s }
    functional_exchange "SF-1.objects" -> "SF-2.objects" { label: "objects" }
    functional_chain "Chain" { id: "FC-1" involves: ["SF-1", "SF-2"] latency_budget: 100 ms }
}
architecture logical {
    component "Perception" {
        id: "LC-1"
        port out objects { interface: "ObjectList" }
        component "Radar" { id: "LC-1-1" function "Scan" }
    }
    component "Planner" { id: "LC-2" port in objects { interface: "ObjectList" } }
    component_exchange "ObjectsFlow" { from_port: "LC-1.objects" to_port: "LC-2.objects" }
}
architecture physical {
    node "ECU" { id: "PN-1" deploys "LC-2" }
}
state_machine "Modes" {
    initial: "Off"
    mode "Off"
    mode "On"
    transition "Off" -> "On" { trigger: "power" }
}
test_case "TC-1" { verifies: ["REQ-1"] method: "test" }
trace "LC-2" satisfies "REQ-1"
trace "LC-1" realizes "SF-1"
"#,
        );
        assert!(sysml.contains("item def ObjectList;"), "{sysml}");
        assert!(sysml.contains("action def <'SF-1'> Detect {"), "{sysml}");
        assert!(sysml.contains("out item objects : ObjectList;"), "{sysml}");
        assert!(sysml.contains("attribute latency : DurationValue = 0.02 [s];"), "{sysml}");
        assert!(sysml.contains("action a_SF_1 : Detect;"), "{sysml}");
        assert!(sysml.contains("flow from a_SF_1.objects to a_SF_2.objects;  // objects"), "{sysml}");
        assert!(sysml.contains("action def <'FC-1'> chain_Chain {"), "{sysml}");
        assert!(sysml.contains("action step1 : Detect;\n        then action step2 : Decide;"), "{sysml}");
        assert!(sysml.contains("attribute latency_budget : DurationValue = 100 [ms];"), "{sysml}");
        assert!(sysml.contains("port def Out_ObjectList_Port { out item data : ObjectList; }"), "{sysml}");
        assert!(sysml.contains("port objects : Out_ObjectList_Port;"), "{sysml}");
        assert!(sysml.contains("part def <'LC-1-1'> Radar {"), "{sysml}");
        assert!(sysml.contains("part p_LC_1_1 : Radar;"), "nested usage: {sysml}");
        assert!(sysml.contains("connect p_LC_1.objects to p_LC_2.objects;  // ObjectsFlow"), "{sysml}");
        assert!(sysml.contains("allocate p_LC_2 to p_PN_1;"), "{sysml}");
        assert!(sysml.contains("state def Modes {"), "{sysml}");
        assert!(sysml.contains("entry; then Off;"), "{sysml}");
        assert!(sysml.contains("transition t1 first Off then On;  // trigger: power"), "{sysml}");
        assert!(sysml.contains("verification def <'TC-1'> verify_TC_1 {"), "{sysml}");
        assert!(sysml.contains("verify req_REQ_1;"), "{sysml}");
        assert!(sysml.contains("dependency realizes from p_LC_1 to a_SF_1;"), "{sysml}");
    }

    #[test]
    fn reserved_words_are_quoted_and_interfaces_have_two_ends() {
        assert_eq!(ident("standard"), "'standard'");
        assert_eq!(ident("port"), "'port'");
        let sysml = export(
            r#"
model K {}
architecture logical {
    component "A" { id: "LC-1" standard: "FIPS 140-3" }
    component "B" { id: "LC-2" }
    interface "Bus" { from: "LC-1" to: "LC-2" }
}
"#,
        );
        assert!(sysml.contains("attribute 'standard' : String = \"FIPS 140-3\";"), "{sysml}");
        assert!(sysml.contains("port def Inout_Port;"), "{sysml}");
        assert!(sysml.contains("end provider : Inout_Port;\n        end consumer : Inout_Port;"), "{sysml}");
        assert!(sysml.contains("connect p_LC_1 to p_LC_2;  // interface Bus"), "{sysml}");
    }

    #[test]
    fn constraints_are_exported_as_asserted_expressions_over_usages() {
        let sysml = export(
            r#"
model C {}
system_analysis "SA" {
    function "Detect" { id: "SF-1" latency: 40 ms }
    function "Decide" { id: "SF-2" latency: 50 ms }
    functional_chain "Braking" { id: "FC-1" involves: ["SF-1", "SF-2"] latency_budget: 100 ms }
}
architecture logical {
    component "Controller" { id: "LC-1" function "Regulate" { latency: 5 ms } }
}
constraint "Chain budget" {
    id: "CST-1"
    description: "end to end"
    assert: sum("FC-1", latency) + Regulate.latency <= "FC-1".latency_budget * 0.99
}
constraint "Ghost" { id: "CST-2" assert: count("FC-1") == 2 }
"#,
        );
        assert!(sysml.contains("action a_FC_1 : chain_Braking;"), "{sysml}");
        assert!(sysml.contains("assert constraint <'CST-1'> Chain_budget {"), "{sysml}");
        assert!(sysml.contains("doc /* Chain budget — end to end */"), "{sysml}");
        assert!(
            sysml.contains("((a_SF_1.latency + a_SF_2.latency) + p_LC_1.perform_Regulate.latency) <= (a_FC_1.latency_budget * 0.99)"),
            "{sysml}"
        );
        assert!(sysml.contains("        2 == 2\n"), "{sysml}");
    }

    #[test]
    fn links_are_named_connections_and_aggregates_use_library_functions() {
        let sysml = export(
            r#"
model L {}
system_analysis "SA" {
    function "A" { id: "SF-1" wcet: 3 ms }
    function "B" { id: "SF-2" wcet: 4 ms }
    functional_chain "Chain" { id: "FC-1" involves: ["SF-1", "SF-2"] }
}
architecture physical {
    node "N1" { id: "PN-1" }
    node "N2" { id: "PN-2" }
    link "Bus" { from: "PN-1" to: "PN-2" protocol: "CAN" bandwidth: 2 Mbps load: 600 kbps }
}
constraint "Load" { assert: Bus.load / Bus.bandwidth <= 0.4 }
constraint "Slowest" { assert: max("FC-1", wcet) <= 5 ms }
"#,
        );
        assert!(sysml.contains("connection Bus connect p_PN_1 to p_PN_2 {"), "{sysml}");
        assert!(sysml.contains("attribute bandwidth : BinaryDigitRateValue = 2 ['Mbit/s'];"), "{sysml}");
        assert!(sysml.contains("(Bus.load / Bus.bandwidth) <= 0.4"), "{sysml}");
        assert!(sysml.contains("DataFunctions::max(a_SF_1.wcet, a_SF_2.wcet) <= 5 [ms]"), "{sysml}");
    }

    #[test]
    fn scenarios_are_exported_as_occurrences_with_ordered_events() {
        let sysml = export(
            r#"
model Scn {}
architecture logical {
    component "Controller" { id: "LC-1" }
    component "Brake" { id: "LC-2" }
}
scenario "Stop" {
    participants: ["LC-1", "LC-2"]
    message "LC-1" -> "LC-2" "apply brake" { type: async }
    message "LC-2" -> "LC-1" "pressure reached"
}
"#,
        );
        assert!(sysml.contains("occurrence def scenario_Stop {"), "{sysml}");
        assert!(sysml.contains("ref part l_LC_1 : Controller {\n            event occurrence m1_sent;\n            then event occurrence m2_received;"), "{sysml}");
        assert!(sysml.contains("message m1 from l_LC_1.m1_sent to l_LC_2.m1_received;  // apply brake [async]"), "{sysml}");
        assert!(sysml.contains("message m2 from l_LC_2.m2_sent to l_LC_1.m2_received;"), "{sysml}");
    }

    #[test]
    fn types_become_abstract_definitions_and_instances_specialize_them() {
        let sysml = export(
            r#"
model Ty {}
type "ECU" { required: ["ram"] voltage: 12 V port in power { interface: "Power12V" } }
type "Safety ECU" extends "ECU" { safety_level: "ASIL-D" voltage: 24 V }
type "Periodic task" { period: 10 ms }
type "Unused" { note: "kept" }
system_analysis "SA" {
    function "Sample" { id: "SF-1" is: "Periodic task" wcet: 2 ms }
}
architecture logical {
    component "Brake ECU" { id: "LC-1" is: "Safety ECU" ram: 128 MB voltage: 48 V port out torque { interface: "Torque" } }
}
"#,
        );
        assert!(sysml.contains("abstract part def ECU {"), "{sysml}");
        assert!(sysml.contains("doc /* type ECU — instances must provide: ram */"), "{sysml}");
        assert!(sysml.contains("attribute voltage : ElectricPotentialValue default = 12 [V];"), "{sysml}");
        assert!(sysml.contains("        port power : In_Power12V_Port;"), "{sysml}");
        assert!(sysml.contains("abstract part def Safety_ECU :> ECU {"), "{sysml}");
        assert!(sysml.contains("attribute :>> voltage default = 24 [V];"), "type redefinition: {sysml}");
        assert!(sysml.contains("abstract action def Periodic_task {"), "{sysml}");
        assert!(sysml.contains("abstract part def Unused {"), "{sysml}");
        assert!(sysml.contains("action def <'SF-1'> Sample :> Periodic_task {"), "{sysml}");
        assert!(sysml.contains("part def <'LC-1'> Brake_ECU :> Safety_ECU {"), "{sysml}");
        assert!(sysml.contains("attribute :>> voltage = 48 [V];"), "instance redefinition: {sysml}");
        assert!(sysml.contains("attribute ram : StorageCapacityValue = 128 [MB];"), "{sysml}");
        let brake = &sysml[sysml.find("part def <'LC-1'>").unwrap()..];
        let brake = &brake[..brake.find("    }\n").unwrap()];
        assert!(!brake.contains("safety_level"), "inherited unchanged attributes are not repeated: {brake}");
        assert!(!brake.contains("port power"), "inherited ports are not redeclared: {brake}");
        assert!(brake.contains("port torque : Out_Torque_Port;"), "{brake}");
        assert!(!brake.contains("attribute is "), "{brake}");
    }

    #[test]
    fn multiple_typing_specializes_every_type() {
        let sysml = export(
            r#"
model Mt {}
type "ECU" { voltage: 12 V }
type "Redundant" { redundancy: "dual" }
architecture logical {
    component "Brake ECU" { id: "LC-1" is: ["ECU", "Redundant"] }
}
"#,
        );
        assert!(sysml.contains("part def <'LC-1'> Brake_ECU :> ECU, Redundant {"), "{sysml}");
        assert!(sysml.contains("abstract part def Redundant {"), "{sysml}");
        assert!(!sysml.contains("attribute is "), "{sysml}");
    }

    #[test]
    fn typed_links_are_connections_typed_by_a_connection_definition() {
        let sysml = export(
            r#"
model Ln {}
type "Bus" { protocol: "CAN" bandwidth: 500 kbps }
type "Fast bus" extends "Bus" { bandwidth: 2 Mbps }
architecture physical {
    node "A" { id: "PN-1" }
    node "B" { id: "PN-2" }
    link "Chassis" { from: "PN-1" to: "PN-2" is: "Fast bus" load: 600 kbps }
}
"#,
        );
        assert!(sysml.contains("abstract connection def Bus {\n        doc /* type Bus */\n        end source;\n        end target;"), "{sysml}");
        assert!(sysml.contains("attribute bandwidth : BinaryDigitRateValue default = 500 ['kbit/s'];"), "{sysml}");
        assert!(sysml.contains("abstract connection def Fast_bus :> Bus {"), "{sysml}");
        assert!(sysml.contains("attribute :>> bandwidth default = 2 ['Mbit/s'];"), "{sysml}");
        assert!(sysml.contains("connection Chassis : Fast_bus connect p_PN_1 to p_PN_2 {\n        attribute load : BinaryDigitRateValue = 600 ['kbit/s'];\n    }"), "{sysml}");
    }

    #[test]
    fn an_empty_data_type_is_an_untyped_port_and_its_item_is_declared() {
        let sysml = export("model T {}\nsystem_analysis \"S\" {\n  function \"F\" { id: \"SF-1\" port out x { data_type: \"\" } }\n}\n");
        assert!(sysml.contains("item def Data;"), "{sysml}");
        assert!(sysml.contains("out item x : Data;"), "{sysml}");
        assert!(!sysml.contains("Anything"), "{sysml}");
    }

    #[test]
    fn export_is_deterministic() {
        let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/complete_emergency_braking_simple.arc")).unwrap();
        assert_eq!(export(&source), export(&source));
    }
}
