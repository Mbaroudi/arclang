//! Data: packages become data dictionaries, classes and exchange items buses,
//! numeric types custom types (or MATLAB's own), physical quantities signals,
//! enumerations enumerations (`DataPackageRule`, `InterfacePackageRule`,
//! `ClassRule`, `ExchangeItemRule`, `NumericTypeRule`, `PhysicalQuantityRule`,
//! `EnumerationRule` and their helpers).

use super::super::pivot::Kind;
use super::model::Model;
use super::transposer::{Premise, Rule};
use super::{Key, Run, Target};

/// `PredefinedTypesHelper`: the Capella predefined types MATLAB has, by name.
pub fn predefined_id(name: &str) -> Option<&'static str> {
    Some(match name {
        "Boolean" => "895807ab-63ff-4779-8bed-8ff8180aa13e",
        "Byte" | "Short" => "73495737-7776-4b32-bd60-679298ddd36b",
        "Char" => "d386bdaf-8107-4db5-aada-ecdf042d8d1f",
        "Double" => "af51b9eb-672c-4506-ac0a-3faefd6e01fc",
        "Float" => "6a8cb80e-a3a8-4bce-b9f8-49dd1371a1c0",
        "Integer" => "15c85f46-ad7f-4fc6-af4c-4b91579177b1",
        "Long" => "00d92ac8-3b5d-44bb-89a8-cb71d44c1283",
        "LongLong" => "2741be9b-8e19-48c0-baef-b109fdbd243c",
        "String" => "6a508238-c18c-4e99-b021-21ef65851886",
        "UnsignedInteger" => "cf2b84ed-a056-4d16-88f8-d79695dc36c0",
        "UnsignedLong" => "dcb6921d-bf28-487f-a818-b47afcafae0b",
        "UnsignedLongLong" => "24c1e089-50e4-4602-9325-4473125c0263",
        "UnsignedShort" => "c429e0fa-20a6-40c9-ad40-9c443ff3e639",
        _ => return None,
    })
}

/// `DataNameHelper.getName`.
pub fn bus_name(m: &Model, e: usize) -> String {
    let name = m.name(e).unwrap_or_else(|| "null".to_string());
    if m.is(e, "FunctionalExchange") {
        format!("busFE_{name}")
    } else if m.is(e, "ExchangeItem") {
        format!("busEI_{name}")
    } else if m.is(e, "Class") {
        format!("busClass_{name}")
    } else {
        name
    }
}

/// `Classifier.getContainedProperties`: the owned features that are properties.
pub fn contained_properties(m: &Model, classifier: usize) -> Vec<usize> {
    m.owned(classifier, "ownedFeatures").into_iter().filter(|&f| m.class(f) == "Property").collect()
}

fn type_of(m: &Model, typed: usize) -> Option<usize> {
    m.reference(typed, "abstractType")
}

/// `GeneralizableElement.getSuper`.
fn supers(m: &Model, e: usize) -> Vec<usize> {
    m.owned(e, "ownedGeneralizations").into_iter().filter_map(|g| m.reference(g, "super")).collect()
}

pub fn applicable(run: &Run, rule: Rule, e: usize) -> bool {
    let m = run.m;
    match rule {
        Rule::DataPkg => {
            m.name(e).as_deref() != Some("Predefined Types") && ["ownedClasses", "ownedDataTypes", "ownedExchangeItems"].iter().any(|f| !m.owned(e, f).is_empty())
        }
        Rule::InterfacePkg => !m.owned(e, "ownedExchangeItems").is_empty(),
        _ => true,
    }
}

fn data_package_ancestor(m: &Model, e: Option<usize>) -> Option<usize> {
    m.container(e?).and_then(|c| m.ancestor(c, "DataPkg"))
}

/// `DataPkgHelper.getDependentDataPkg`: the packages of what a package's types use. (The
/// connector keeps them in a `HashSet`, in no order Java promises; they are kept here in the
/// order they are found.)
fn dependent_packages(m: &Model, pkg: usize) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    let mut add = |p: Option<usize>| {
        if let Some(p) = p {
            if !out.contains(&p) {
                out.push(p);
            }
        }
    };
    for t in m.owned(pkg, "ownedDataTypes").into_iter().filter(|&t| m.is(t, "Enumeration")) {
        for literal in m.owned(t, "ownedLiterals") {
            for value in m.owned(literal, "domainValue").into_iter().chain(m.refs(literal, "domainValue")) {
                add(data_package_ancestor(m, m.reference(value, "referencedProperty")));
            }
        }
    }
    for class in m.owned(pkg, "ownedClasses") {
        for property in contained_properties(m, class) {
            add(data_package_ancestor(m, type_of(m, property)));
        }
        for s in supers(m, class) {
            add(data_package_ancestor(m, Some(s)));
        }
    }
    for ei in m.owned(pkg, "ownedExchangeItems") {
        add(m.container(ei).filter(|&c| m.is(c, "AbstractDependenciesPkg")));
        for element in m.owned(ei, "ownedElements") {
            add(data_package_ancestor(m, type_of(m, element)));
        }
    }
    out.retain(|&p| p != pkg);
    out
}

pub fn premises(run: &Run, rule: Rule, e: usize) -> Vec<Premise> {
    let m = run.m;
    let mut out: Vec<Premise> = Vec::new();
    match rule {
        Rule::DataPkg => out.extend(dependent_packages(m, e).into_iter().map(|p| (Some(p), false))),
        Rule::Class => {
            for s in supers(m, e).into_iter().filter(|&s| m.is(s, "Classifier")) {
                out.extend(contained_properties(m, s).into_iter().filter_map(|p| type_of(m, p)).map(|t| (Some(t), false)));
            }
            out.extend(contained_properties(m, e).into_iter().filter_map(|p| type_of(m, p)).map(|t| (Some(t), false)));
            out.push((m.container(e), true));
        }
        Rule::NumericType | Rule::Enumeration => out.push((m.container(e), true)),
        Rule::PhysicalQuantity => {
            out.push((m.container(e), true));
            out.extend(supers(m, e).into_iter().map(|s| (Some(s), false)));
        }
        Rule::ExchangeItem => {
            out.push((m.container(e), true));
            out.extend(m.owned(e, "ownedElements").into_iter().filter_map(|x| type_of(m, x)).map(|t| (Some(t), false)));
        }
        _ => {}
    }
    out
}

pub fn create(run: &mut Run, rule: Rule, e: usize) -> Result<Option<Target>, String> {
    let m = run.m;
    let made = match rule {
        Rule::DataPkg | Rule::InterfacePkg => run.create_dictionary(e, m.name(e))?,
        Rule::Class => create_class(run, e),
        Rule::ExchangeItem => create_exchange_item(run, e),
        Rule::NumericType => match run.predefined(m.name(e).as_deref()) {
            Some(t) => t,
            None => create_numeric(run, e),
        },
        Rule::Enumeration => match run.predefined(m.name(e).as_deref()) {
            Some(t) => t,
            None => create_enumeration(run, e),
        },
        Rule::PhysicalQuantity => create_quantity(run, e),
        _ => return Ok(None),
    };
    run.set_main(Key::E(e), made);
    Ok(Some(Target::P(made)))
}

/// Puts a type in the dictionary of its Capella container, if it has one.
fn add_to_container(run: &mut Run, e: usize, datatype: usize) {
    if let Some(parent) = run.main_opt(run.m.container(e)) {
        if run.p[parent].kind == Kind::Dictionary {
            run.contain(parent, datatype, |o| &mut o.datatypes);
        }
    }
}

fn create_class(run: &mut Run, class: usize) -> usize {
    let m = run.m;
    let bus = run.new_obj(Kind::Bus, Some(bus_name(m, class)), m.id(class));
    add_to_container(run, class, bus);
    run.set_main(Key::E(class), bus);
    class_properties(run, bus, class);
    bus
}

/// A bus element for each property of the classifier, then of its super classifiers.
fn class_properties(run: &mut Run, bus: usize, classifier: usize) {
    let m = run.m;
    for property in contained_properties(m, classifier) {
        let element = run.new_obj(Kind::BusElement, m.name(property), m.id(property));
        let t = type_of(m, property);
        let mut datatype = run.main_opt(t);
        if datatype.is_none() && t.is_some() {
            datatype = run.predefined(t.and_then(|t| m.name(t)).as_deref());
        }
        if let Some(d) = datatype.filter(|&d| run.p[d].kind.is_datatype()) {
            run.p[element].datatype = Some(d);
        }
        run.contain(bus, element, |o| &mut o.bus_elements);
    }
    for s in supers(m, classifier) {
        if m.is(s, "Classifier") {
            class_properties(run, bus, s);
        }
    }
}

fn create_exchange_item(run: &mut Run, ei: usize) -> usize {
    let m = run.m;
    let bus = run.new_obj(Kind::Bus, Some(bus_name(m, ei)), m.id(ei));
    add_to_container(run, ei, bus);
    for x in m.owned(ei, "ownedElements") {
        let element = run.new_obj(Kind::BusElement, m.name(x), m.id(x));
        if let Some(t) = type_of(m, x) {
            run.p[element].datatype = run.main(t);
        }
        run.contain(bus, element, |o| &mut o.bus_elements);
    }
    bus
}

fn create_numeric(run: &mut Run, numeric: usize) -> usize {
    let m = run.m;
    let custom = run.new_obj(Kind::CustomType, m.name(numeric), m.id(numeric));
    match m.attr(numeric, "kind").unwrap_or("INTEGER") {
        "FLOAT" => run.set_param(custom, "DataTypeMode", "Double"),
        "INTEGER" => {
            run.set_param(custom, "DataTypeMode", "Fixed-point: slope and bias scaling");
            run.set_param(custom, "Bias", "0");
            run.set_param(custom, "Signedness", "Signed");
            run.set_param(custom, "WordLength", "64");
        }
        _ => {}
    }
    run.set_param(custom, "IsAlias", "true");
    add_to_container(run, numeric, custom);
    custom
}

fn create_enumeration(run: &mut Run, enumeration: usize) -> usize {
    let m = run.m;
    let custom = run.new_obj(Kind::Enumeration, m.name(enumeration), m.id(enumeration));
    add_to_container(run, enumeration, custom);
    for (value, literal) in m.owned(enumeration, "ownedLiterals").into_iter().enumerate() {
        let l = run.new_obj(Kind::Literal, m.name(literal), m.id(literal));
        run.p[l].value = value as i32;
        run.contain(custom, l, |o| &mut o.literals);
    }
    custom
}

fn create_quantity(run: &mut Run, quantity: usize) -> usize {
    let m = run.m;
    let signal = run.new_obj(Kind::Signal, m.name(quantity), m.id(quantity));
    if let Some(unit) = m.reference(quantity, "unit") {
        run.p[signal].unit = m.name(unit);
    }
    if let Some(value) = m.owned(quantity, "ownedDefaultValue").into_iter().next() {
        if m.class(value) == "LiteralNumericValue" {
            run.p[signal].initial_value = m.attr(value, "value").map(str::to_string);
        }
    }
    if let Some(&first) = supers(m, quantity).first() {
        if let Some(d) = run.main(first).filter(|&d| run.p[d].kind.is_datatype()) {
            run.p[signal].datatype = Some(d);
        }
    }
    add_to_container(run, quantity, signal);
    signal
}
