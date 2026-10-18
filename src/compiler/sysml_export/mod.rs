//! SysML v2 export in the Arcadia profile (`spec/SYSML_PROFILE.md`): one
//! package per Arcadia level carrying the level's metadata, every element
//! with its id as short name and its name as written, kinds SysML cannot
//! tell by metadata of the `ArcadiaProfile` library, references as
//! relationships. The text is what `sysml_records` reads back and what the
//! OMG pilot validates; `sysml_import` reads it into the same ArcLang model.

mod architecture;
mod attributes;
mod behaviour;
mod data;
mod levels;
mod relations;
mod text;

use crate::compiler::ast::{AttributeValue, Model};
use crate::compiler::metamodel::Metamodel;
use std::collections::{BTreeSet, HashMap};

/// The metadata definitions of the profile: the levels, then the kinds
/// SysML cannot tell by itself.
pub const PROFILE: &[&str] = &[
    "OA", "SA", "LA", "PA", "EPBS", "Actor", "Entity", "Node", "Behaviour", "Hardware", "Mission", "Capability", "Chain", "Link", "Hazard",
    "Mode", "Class", "PhysicalPort", "ExchangeItem", "Path",
];

/// The SysML v2 text of a model.
pub fn export(model: &Model) -> String {
    let mut exporter = Exporter::new(model);
    exporter.run();
    exporter.finish()
}

/// Where an element is declared: its package, and the names that lead to
/// it from there (`'LC-1'`, then `'LC-2'` for a component it holds, then
/// `radar` for a port), as written in the text.
#[derive(Clone)]
struct Target {
    package: String,
    segments: Vec<String>,
}

pub(crate) struct Exporter<'m> {
    model: &'m Model,
    metamodel: Metamodel,
    out: String,
    depth: usize,
    units: BTreeSet<&'static str>,
    /// Reference (id, name, `Owner.port`) -> where it is declared.
    targets: HashMap<String, Target>,
    /// The package being written.
    package: String,
    /// Types ports and exchanges name that the model does not declare.
    undeclared: Vec<String>,
}

impl<'m> Exporter<'m> {
    fn new(model: &'m Model) -> Self {
        Exporter { model, metamodel: Metamodel::current(), out: String::new(), depth: 0, units: BTreeSet::new(), targets: HashMap::new(), package: String::new(), undeclared: Vec::new() }
    }

    fn line(&mut self, text: &str) {
        for _ in 0..self.depth {
            self.out.push_str("    ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    /// `head {`, the body written by `body`, then `}`; `head;` when the body is empty.
    fn block(&mut self, head: &str, body: impl FnOnce(&mut Self)) {
        let mark = self.out.len();
        self.line(&format!("{head} {{"));
        let opened = self.out.len();
        self.depth += 1;
        body(self);
        self.depth -= 1;
        if self.out.len() == opened {
            self.out.truncate(mark);
            self.line(&format!("{head};"));
        } else {
            self.line("}");
        }
    }

    /// A package of the model, its declarations resolved unqualified inside it.
    fn package(&mut self, head: &str, qualified: &str, body: impl FnOnce(&mut Self)) {
        let outer = std::mem::replace(&mut self.package, qualified.to_string());
        self.block(head, body);
        self.package = outer;
    }

    /// The element's documentation and typed attributes.
    fn header(&mut self, kind: &str, attributes: &HashMap<String, AttributeValue>, skip: &[&str]) {
        if let Some(description) = attributes.get("description").and_then(|d| d.as_string()) {
            self.line(&text::doc(description));
        }
        for line in attributes::lines(&self.metamodel, kind, attributes, skip, &mut self.units) {
            self.line(&line);
        }
    }

    /// Declare a top-level element of `package`, known by its short name `id`.
    fn register(&mut self, package: &str, id: &str, references: &[&str]) {
        self.register_at(package, vec![text::name(id)], id, references);
    }

    /// Declare an element reached from `package` by `segments`, known as
    /// `id` and by every one of `references`.
    fn register_at(&mut self, package: &str, segments: Vec<String>, id: &str, references: &[&str]) {
        let target = Target { package: package.to_string(), segments };
        for reference in references.iter().filter(|r| !r.is_empty()) {
            self.targets.entry(reference.to_string()).or_insert_with(|| target.clone());
        }
        if !id.is_empty() {
            self.targets.entry(id.to_string()).or_insert(target);
        }
    }

    /// Let `alias` name what `existing` names (`Owner.function`).
    fn alias(&mut self, alias: &str, existing: &str) {
        if let Some(target) = self.targets.get(existing).cloned() {
            self.targets.entry(alias.to_string()).or_insert(target);
        }
    }

    fn qualify(&self, target: &Target, separator: &str) -> String {
        let path = target.segments.join(separator);
        if target.package == self.package || target.package.is_empty() {
            path
        } else {
            match separator {
                "::" => format!("{}::{path}", target.package),
                // A feature chain starts from a qualified name.
                _ => format!("{}::{}", target.package, path),
            }
        }
    }

    /// How the package being written names the element `reference` names
    /// (`'LC-1'::'LC-2'`, qualified by its package when it is another's).
    fn reference(&self, reference: &str) -> Option<String> {
        self.targets.get(reference).map(|t| self.qualify(t, "::"))
    }

    /// The feature chain to the element `reference` names, the end of a
    /// connector: `'LC-1'.'LC-2'.radar`.
    fn feature(&self, reference: &str) -> Option<String> {
        self.targets.get(reference).map(|t| self.qualify(t, "."))
    }

    fn run(&mut self) {
        let model_name = self.model.attributes.get("name").and_then(|n| n.as_string()).unwrap_or("ArcLang model").to_string();
        self.register_all();
        self.line("// Generated by ArcLang: SysML v2 in the Arcadia profile (spec/SYSML_PROFILE.md)");
        self.block(&format!("package {}", text::name(&model_name)), |e| {
            for import in ["ScalarValues", "MeasurementReferences", "ISQ", "SI"] {
                e.line(&format!("private import {import}::*;"));
            }
            e.profile();
            e.line("private import ArcadiaProfile::*;");
            let model = e.model;
            e.header("Model", &model.attributes, &["name"]);
            data::write(e);
            levels::write(e);
            behaviour::write(e);
            relations::write(e);
            e.units();
        });
    }

    /// The `ArcadiaProfile` library: one metadata definition per level and kind.
    fn profile(&mut self) {
        self.block("library package ArcadiaProfile", |e| {
            for name in PROFILE {
                e.line(&format!("metadata def {name};"));
            }
        });
    }

    fn register_all(&mut self) {
        data::register(self);
        levels::register(self);
        behaviour::register(self);
    }

    /// Units the SI library does not declare, declared locally as the
    /// library declares its own.
    fn units(&mut self) {
        let declarations: Vec<&'static str> = self.units.iter().filter_map(|u| local_unit(u)).collect();
        if declarations.is_empty() {
            return;
        }
        self.block("package ArcLangUnits", |e| {
            for import in ["MeasurementReferences", "ISQ", "SI"] {
                e.line(&format!("private import {import}::*;"));
            }
            for declaration in declarations {
                e.line(declaration);
            }
        });
        self.line("private import ArcLangUnits::*;");
    }

    fn finish(self) -> String {
        self.out
    }
}

/// Whether a flag attribute is set (`actor: true`).
pub(crate) fn flag(attributes: &HashMap<String, AttributeValue>, key: &str) -> bool {
    matches!(attributes.get(key), Some(AttributeValue::Boolean(true))) || attributes.get(key).and_then(|v| v.as_string()) == Some("true")
}

/// The written id of an element: its `id:` attribute, else `fallback`.
pub(crate) fn id_of<'a>(attributes: &'a HashMap<String, AttributeValue>, fallback: &'a str) -> &'a str {
    attributes.get("id").and_then(|v| v.as_string()).filter(|s| !s.is_empty()).unwrap_or(fallback)
}

fn local_unit(symbol: &str) -> Option<&'static str> {
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
