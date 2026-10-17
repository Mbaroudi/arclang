//! The transverse content of a Capella model: the data model (types,
//! enumerations, classes, exchange items), the interfaces, the requirements
//! with their attributes and relations, and what Capella attaches to any
//! element: property values (PVMT) and constraints.

use super::{display_name, first_ref, header_attributes, key_of, list_literal, plain, refs, text_attr, Node, PropertyValue, Writer};
use crate::compiler::source_edit::string_literal;
use std::collections::{HashMap, HashSet};

// --- property values and constraints -------------------------------------------

/// Read every property value group and value of the project, resolved to
/// the literal ArcLang will carry.
pub(super) fn collect_property_values(writer: &mut Writer, project: &Node) {
    let literals: HashMap<String, String> = project
        .find_all("capellacore:EnumerationPropertyLiteral")
        .into_iter()
        .map(|literal| (literal.id().to_string(), display_name(literal)))
        .collect();
    let literal_of = |value: &Node| -> Option<String> {
        let key = key_of(value.name());
        let literal = match value.kind.as_str() {
            "capellacore:StringPropertyValue" => string_literal(value.attr("value").unwrap_or("")),
            "capellacore:IntegerPropertyValue" | "capellacore:FloatPropertyValue" => {
                let raw = value.attr("value").unwrap_or("0").trim();
                if raw.parse::<f64>().is_ok() { raw.to_string() } else { string_literal(raw) }
            }
            "capellacore:BooleanPropertyValue" => if value.attr("value") == Some("true") { "true".to_string() } else { "false".to_string() },
            "capellacore:EnumerationPropertyValue" => {
                let name = first_ref(value.attr("value")).and_then(|id| literals.get(&id).cloned()).unwrap_or_default();
                string_literal(&name)
            }
            _ => return None,
        };
        let _ = key;
        Some(literal)
    };
    for group in project.find_all("capellacore:PropertyValueGroup") {
        let prefix = key_of(group.name());
        let mut values = Vec::new();
        for value in group.tagged("ownedPropertyValues") {
            if let Some(literal) = literal_of(value) {
                values.push(PropertyValue { key: format!("{prefix}_{}", key_of(value.name())), literal });
            }
        }
        writer.value_groups.insert(group.id().to_string(), values);
    }
    for value in project.descendants().into_iter().filter(|n| n.kind.ends_with("PropertyValue")) {
        if let Some(literal) = literal_of(value) {
            writer.values.insert(value.id().to_string(), PropertyValue { key: key_of(value.name()), literal });
        }
    }
}

/// The property values applied to (or owned by) `node`, as attributes.
pub(super) fn applied_values(writer: &mut Writer, depth: usize, node: &Node, kind: &str) {
    let mut lines: Vec<PropertyValue> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let groups: Vec<String> = refs(node.attr("appliedPropertyValueGroups"))
        .into_iter()
        .chain(node.tagged("ownedPropertyValueGroups").map(|g| g.id().to_string()))
        .collect();
    for group in groups {
        if seen.insert(format!("g:{group}")) {
            if let Some(values) = writer.value_groups.get(&group) {
                lines.extend(values.iter().cloned());
            }
        }
    }
    let values: Vec<String> = refs(node.attr("appliedPropertyValues"))
        .into_iter()
        .chain(node.tagged("ownedPropertyValues").map(|v| v.id().to_string()))
        .collect();
    for value in values {
        if seen.insert(format!("v:{value}")) {
            if let Some(found) = writer.values.get(&value) {
                lines.push(found.clone());
            }
        }
    }
    // What an add-on viewpoint says about this element, as attributes too.
    if let Some(extensions) = writer.extension_values.get(node.id()).cloned() {
        writer.count_by("viewpoint attributes of the elements they extend", extensions.len());
        lines.extend(extensions);
    }
    if lines.is_empty() {
        return;
    }
    if kind == "Class" {
        // Every attribute of a class is a field: the values would become fields.
        writer.count_by("property values on classes (left out: a class attribute is a field)", lines.len());
        return;
    }
    let mut used: HashSet<String> = HashSet::new();
    for value in lines {
        let reserved = matches!(value.key.as_str(), "id" | "name" | "description" | "summary" | "is" | "type" | "kind") || writer.is_typed(kind, &value.key);
        let mut key = if reserved { format!("pv_{}", value.key) } else { value.key.clone() };
        while !used.insert(key.clone()) {
            key.push('_');
        }
        writer.count("property values");
        writer.line(depth, &format!("{key}: {}", value.literal));
    }
}

/// The constraints owned by `node`: their expression bodies, as text.
pub(super) fn constraints_of(writer: &mut Writer, depth: usize, node: &Node) {
    let mut bodies: Vec<String> = Vec::new();
    for constraint in node.tagged("ownedConstraints").filter(|c| c.is("capellacore:Constraint")) {
        let text = constraint_text(constraint);
        if !text.is_empty() {
            bodies.push(text);
        }
    }
    if !bodies.is_empty() {
        writer.count_by("constraints", bodies.len());
        writer.line(depth, &format!("constraints: {}", list_literal(&bodies)));
    }
}

/// The text of a Capella constraint: its opaque expression's bodies, else its name.
pub(super) fn constraint_text(constraint: &Node) -> String {
    let mut bodies: Vec<String> = Vec::new();
    for specification in constraint.tagged("ownedSpecification") {
        for body in specification.tagged("bodies") {
            let text = body.text.trim();
            if !text.is_empty() {
                bodies.push(plain(text));
            }
        }
        if bodies.is_empty() {
            if let Some(name) = text_attr(specification, "name") {
                bodies.push(name);
            }
        }
    }
    if bodies.is_empty() {
        return constraint.name().trim().to_string();
    }
    bodies.join(" | ")
}

// --- the data model ---------------------------------------------------------------

/// A name for a declaration, unique among those already written in `scope`.
pub(super) fn unique_name(taken: &mut HashSet<String>, name: &str) -> String {
    if taken.insert(name.to_string()) {
        return name.to_string();
    }
    let mut n = 2;
    loop {
        let candidate = format!("{name} ({n})");
        if taken.insert(candidate.clone()) {
            return candidate;
        }
        n += 1;
    }
}

/// Types, enumerations, classes and exchange items of every level. The
/// predefined types Capella repeats at each level are declared once: the
/// copies answer to the first declaration.
pub(super) fn data_model(writer: &mut Writer, engineering: &Node) {
    let mut taken: HashSet<String> = HashSet::new();
    let mut type_names: HashMap<String, String> = HashMap::new();
    let mut first_of_name: HashMap<String, String> = HashMap::new();
    let units: HashMap<String, String> = engineering.find_all("information:Unit").into_iter().map(|u| (u.id().to_string(), display_name(u))).collect();

    // Data types and enumerations.
    for node in engineering.descendants() {
        let (keyword, base) = match node.kind.as_str() {
            "datatype:NumericType" => ("data_type", if node.attr("kind") == Some("FLOAT") { "Real" } else { "Integer" }),
            "datatype:StringType" => ("data_type", "String"),
            "datatype:BooleanType" => ("data_type", "Boolean"),
            "datatype:PhysicalQuantity" => ("data_type", "Real"),
            "datatype:Enumeration" => ("enumeration", ""),
            _ => continue,
        };
        let name = display_name(node);
        if let Some(first) = first_of_name.get(&format!("{keyword}:{name}")) {
            // A copy of a type declared already (Capella's predefined types).
            writer.ids.insert(node.id().to_string(), first.clone());
            type_names.insert(node.id().to_string(), name);
            continue;
        }
        first_of_name.insert(format!("{keyword}:{name}"), node.id().to_string());
        type_names.insert(node.id().to_string(), name.clone());
        let written = unique_name(&mut taken, &name);
        if keyword == "enumeration" {
            writer.count("enumerations");
            writer.remember(node, "Enumeration");
            let values: Vec<String> = node.tagged("ownedLiterals").map(display_name).collect();
            writer.line(0, &format!("enumeration {} {{", string_literal(&written)));
            header_attributes(writer, 1, node, "Enumeration");
            writer.line(1, &format!("values: {}", list_literal(&values)));
            writer.line(0, "}");
        } else {
            writer.count("data types");
            writer.remember(node, "DataType");
            writer.line(0, &format!("data_type {} {{", string_literal(&written)));
            header_attributes(writer, 1, node, "DataType");
            writer.line(1, &format!("base: {}", string_literal(base)));
            if let Some(unit) = first_ref(node.attr("unit")).and_then(|id| units.get(&id)) {
                let key = if KNOWN_UNITS.contains(&unit.as_str()) { "unit" } else { "unit_name" };
                writer.line(1, &format!("{key}: {}", string_literal(unit)));
            }
            writer.line(0, "}");
        }
    }

    // Classes, unions and collections: their fields name their types.
    let classes: Vec<&Node> = engineering
        .descendants()
        .into_iter()
        .filter(|n| matches!(n.kind.as_str(), "information:Class" | "information:Union" | "information:Collection"))
        .collect();
    for class in &classes {
        type_names.insert(class.id().to_string(), display_name(class));
    }
    // Property id -> (class id, field key written): an association names
    // its navigable end so.
    let mut field_keys: HashMap<String, (String, String)> = HashMap::new();
    let mut generalizations = 0;
    for class in &classes {
        let name = unique_name(&mut taken, &display_name(class));
        writer.count(match class.kind.as_str() {
            "information:Union" => "unions (as classes)",
            "information:Collection" => "collections (as classes)",
            _ => "classes",
        });
        writer.remember(class, "Class");
        writer.line(0, &format!("class {} {{", string_literal(&name)));
        header_attributes(writer, 1, class, "Class");
        // The first generalization is `extends:`; a class specializes one class here.
        let supers: Vec<String> = class.tagged("ownedGeneralizations").filter_map(|g| first_ref(g.attr("super"))).collect();
        if let Some(base) = supers.first().and_then(|id| type_names.get(id)) {
            writer.count("class generalizations");
            writer.line(1, &format!("extends: {}", string_literal(base)));
        }
        if supers.len() > 1 {
            writer.count_by("class generalizations beyond the first (left out: a class extends one class)", supers.len() - 1);
        }
        let mut fields: HashSet<String> = HashSet::new();
        let mut index = 0;
        for property in class.children.iter().filter(|c| matches!(c.kind.as_str(), "information:Property" | "information:UnionProperty")) {
            index += 1;
            let raw = property.name().trim();
            let mut key = if raw.is_empty() { format!("field_{index}") } else { key_of(raw) };
            if matches!(key.as_str(), "id" | "description") {
                key = format!("{key}_field");
            }
            while !fields.insert(key.clone()) {
                key.push('_');
            }
            let type_name = first_ref(property.attr("abstractType")).and_then(|id| type_names.get(&id).cloned()).unwrap_or_else(|| "Data".to_string());
            writer.count("class fields");
            field_keys.insert(property.id().to_string(), (class.id().to_string(), key.clone()));
            writer.line(1, &format!("{key}: {}", string_literal(&type_name)));
        }
        if class.is("information:Collection") {
            if let Some(items) = first_ref(class.attr("type")).and_then(|id| type_names.get(&id).cloned()) {
                writer.line(1, &format!("items: {}", string_literal(&items)));
            }
        }
        writer.line(0, "}");
        for realization in class.tagged("ownedInformationRealizations") {
            if let Some(target) = first_ref(realization.attr("targetElement")) {
                writer.traces.push((class.id().to_string(), "realizes", target));
            }
        }
        generalizations += 0;
    }
    let _ = generalizations;
    associations(writer, engineering, &field_keys);

    // Exchange items, with the types of their elements.
    for item in engineering.find_all("information:ExchangeItem") {
        let name = unique_name(&mut taken, &display_name(item));
        writer.count("exchange items");
        writer.remember(item, "ExchangeItem");
        let mechanism = match item.attr("exchangeMechanism") {
            Some("EVENT") => "EVENT",
            Some("OPERATION") => "OPERATION",
            Some("SHARED_DATA") => "SHARED_DATA",
            Some("UNSET") | Some("FLOW") | None => "FLOW",
            Some(other) => other,
        };
        writer.line(0, &format!("exchange_item {} {{", string_literal(&name)));
        header_attributes(writer, 1, item, "ExchangeItem");
        writer.line(1, &format!("mechanism: {}", string_literal(mechanism)));
        let elements: Vec<String> = item
            .tagged("ownedElements")
            .filter_map(|element| first_ref(element.attr("abstractType")))
            .filter(|id| writer.written(id))
            .map(|id| writer.ids[&id].clone())
            .collect();
        if !elements.is_empty() {
            writer.count_by("exchange item elements", elements.len());
            writer.line(1, &format!("elements: {}", list_literal(&elements)));
        }
        writer.line(0, "}");
    }
    writer.line(0, "");
}

/// The associations between classes, as `association` declarations: the
/// class owning the navigable end is `from`, its field is the `role`.
fn associations(writer: &mut Writer, engineering: &Node, field_keys: &HashMap<String, (String, String)>) {
    let by_id: HashMap<&str, &Node> = engineering.descendants().into_iter().filter(|n| !n.id().is_empty()).map(|n| (n.id(), n)).collect();
    let card = |end: &Node, tag: &str| end.tagged(tag).next().and_then(|v| v.attr("value").map(str::to_string));
    for association in engineering.find_all("information:Association") {
        let navigable: Option<&Node> = refs(association.attr("navigableMembers")).first().and_then(|id| by_id.get(id.as_str()).copied());
        let members: Vec<&Node> = association.tagged("ownedMembers").collect();
        let (from, to, role, end) = match navigable {
            Some(property) => match (field_keys.get(property.id()), first_ref(property.attr("abstractType"))) {
                (Some((owner, key)), Some(target)) => (owner.clone(), target, Some(key.clone()), property),
                _ => { writer.count("associations whose navigable end is not a written field (left out)"); continue }
            },
            None => match members.as_slice() {
                [first, second, ..] => match (first_ref(first.attr("abstractType")), first_ref(second.attr("abstractType"))) {
                    (Some(from), Some(to)) => (from, to, None, *second),
                    _ => { writer.count("associations without typed ends (left out)"); continue }
                },
                _ => { writer.count("associations without a navigable end (left out)"); continue }
            },
        };
        if !writer.written(&from) || !writer.written(&to) {
            writer.count("associations of a class not written (left out)");
            continue;
        }
        let name = association.name().trim();
        let name = if name.is_empty() { "association" } else { name };
        writer.count("associations");
        writer.remember(association, "Association");
        writer.line(0, &format!("association {} {{", string_literal(name)));
        header_attributes(writer, 1, association, "Association");
        writer.line(1, &format!("from: {}", string_literal(&writer.ids[&from].clone())));
        writer.line(1, &format!("to: {}", string_literal(&writer.ids[&to].clone())));
        if let Some(role) = role {
            writer.line(1, &format!("role: {}", string_literal(&role)));
        }
        match end.attr("aggregationKind") {
            Some("COMPOSITION") => writer.line(1, "kind: \"composition\""),
            Some("AGGREGATION") => writer.line(1, "kind: \"aggregation\""),
            _ => {}
        }
        if let (Some(min), Some(max)) = (card(end, "ownedMinCard"), card(end, "ownedMaxCard")) {
            let max = if max == "-1" { "*".to_string() } else { max };
            let multiplicity = if min == max { min } else { format!("{min}..{max}") };
            writer.line(1, &format!("multiplicity: {}", string_literal(&multiplicity)));
        }
        writer.line(0, "}");
    }
}

/// Unit symbols the compiler's unit table knows: others are kept as a name.
const KNOWN_UNITS: [&str; 24] = ["s", "ms", "us", "ns", "min", "h", "m", "km", "mm", "cm", "kg", "g", "Hz", "kHz", "MHz", "V", "A", "W", "kW", "J", "N", "K", "C", "%"];

// --- interfaces -----------------------------------------------------------------

/// The interfaces declared under `holder`, written at `depth` in a logical
/// architecture: an interface carries the exchange items allocated to it.
pub(super) fn interfaces(writer: &mut Writer, depth: usize, holder: &Node, level: &str, taken: &mut HashSet<String>) {
    for interface in holder.find_all("cs:Interface") {
        let name = unique_name(taken, &display_name(interface));
        writer.count(match level {
            "logical" => "interfaces",
            "system" => "interfaces of the system level (declared in the logical architecture)",
            _ => "interfaces of the physical level (declared in the logical architecture)",
        });
        writer.remember(interface, "LogicalInterface");
        writer.names.insert(interface.id().to_string(), name.clone());
        writer.line(depth, &format!("interface {} {{", string_literal(&name)));
        header_attributes(writer, depth + 1, interface, "LogicalInterface");
        if level != "logical" {
            writer.line(depth + 1, &format!("level: {}", string_literal(level)));
        }
        let items: Vec<String> = interface
            .tagged("ownedExchangeItemAllocations")
            .filter_map(|allocation| first_ref(allocation.attr("allocatedItem")))
            .filter(|id| writer.written(id))
            .map(|id| writer.ids[&id].clone())
            .collect();
        if !items.is_empty() {
            writer.line(depth + 1, &format!("exchange_items: {}", list_literal(&items)));
        }
        writer.line(depth, "}");
    }
}

/// What a component provides and requires: its interface implementations and uses.
pub(super) fn component_interfaces(writer: &mut Writer, depth: usize, component: &Node) {
    let provided: Vec<String> = component.tagged("ownedInterfaceImplementations").filter_map(|i| first_ref(i.attr("implementedInterface"))).collect();
    let required: Vec<String> = component.tagged("ownedInterfaceUses").filter_map(|u| first_ref(u.attr("usedInterface"))).collect();
    for (keyword, list) in [("provides", provided), ("requires", required)] {
        for id in list {
            let Some(name) = writer.names.get(&id).cloned().filter(|_| writer.kinds.get(&id) == Some(&"LogicalInterface")) else {
                writer.count("interface uses of interfaces not written (left out)");
                continue;
            };
            writer.count("interface uses");
            writer.line(depth, &format!("{keyword} interface {} {{}}", string_literal(&name)));
        }
    }
}

// --- requirements -----------------------------------------------------------------

pub(super) fn requirements(writer: &mut Writer, project: &Node) {
    let requirements: Vec<&Node> = project.find_all("Requirements:Requirement");
    if requirements.is_empty() {
        return;
    }
    let definitions: HashMap<String, String> = project
        .descendants()
        .into_iter()
        .filter(|n| n.kind.starts_with("Requirements:AttributeDefinition"))
        .map(|d| (d.id().to_string(), text_attr(d, "ReqIFLongName").or_else(|| text_attr(d, "ReqIFName")).unwrap_or_else(|| "attribute".to_string())))
        .collect();
    let enum_values: HashMap<String, String> = project.find_all("Requirements:EnumValue").into_iter().map(|v| (v.id().to_string(), text_attr(v, "ReqIFLongName").unwrap_or_else(|| display_name(v)))).collect();
    let types: HashMap<String, String> = project.find_all("Requirements:RequirementType").into_iter().map(|t| (t.id().to_string(), text_attr(t, "ReqIFLongName").unwrap_or_else(|| display_name(t)))).collect();
    let mut taken: HashSet<String> = HashSet::new();
    writer.line(0, "requirements system {");
    for requirement in requirements {
        writer.count("requirements");
        let id = requirement.id().to_string();
        let preferred = text_attr(requirement, "ReqIFIdentifier").unwrap_or_else(|| id.clone());
        let short = if taken.insert(preferred.clone()) { preferred } else { id.clone() };
        let title = text_attr(requirement, "ReqIFLongName").or_else(|| text_attr(requirement, "ReqIFName")).unwrap_or_else(|| short.clone());
        writer.ids.insert(id.clone(), short.clone());
        writer.kinds.insert(id.clone(), "Requirement");
        writer.names.insert(id.clone(), title.clone());
        writer.line(1, &format!("req {} {} {{", string_literal(&short), string_literal(&title)));
        writer.line(2, &format!("id: {}", string_literal(&short)));
        writer.line(2, &format!("capella_id: {}", string_literal(&id)));
        let text = text_attr(requirement, "ReqIFText").map(|t| plain(&t)).or_else(|| text_attr(requirement, "ReqIFDescription")).unwrap_or_else(|| title.clone());
        writer.line(2, &format!("description: {}", string_literal(&text)));
        for (attr, key) in [("ReqIFName", "reqif_name"), ("ReqIFChapterName", "chapter"), ("ReqIFPrefix", "prefix"), ("ReqIFForeignID", "foreign_id")] {
            if let Some(value) = text_attr(requirement, attr) {
                writer.line(2, &format!("{key}: {}", string_literal(&value)));
            }
        }
        if let Some(kind) = first_ref(requirement.attr("requirementType")).and_then(|t| types.get(&t)) {
            writer.line(2, &format!("requirement_type: {}", string_literal(kind)));
        }
        let mut used: HashSet<String> = HashSet::new();
        for attribute in requirement.tagged("ownedAttributes") {
            let Some(definition) = first_ref(attribute.attr("definition")).and_then(|d| definitions.get(&d).cloned()) else { continue };
            let literal = match attribute.kind.as_str() {
                "Requirements:EnumerationValueAttribute" => {
                    let names: Vec<String> = refs(attribute.attr("values")).into_iter().filter_map(|v| enum_values.get(&v).cloned()).collect();
                    if names.len() == 1 { string_literal(&names[0]) } else { list_literal(&names) }
                }
                "Requirements:BooleanValueAttribute" => if attribute.attr("value") == Some("true") { "true".to_string() } else { "false".to_string() },
                "Requirements:IntegerValueAttribute" | "Requirements:RealValueAttribute" => {
                    let raw = attribute.attr("value").unwrap_or("0").trim();
                    if raw.parse::<f64>().is_ok() { raw.to_string() } else { string_literal(raw) }
                }
                _ => string_literal(attribute.attr("value").unwrap_or("")),
            };
            let base = key_of(&definition);
            let reserved = matches!(base.as_str(), "id" | "description" | "capella_id") || writer.is_typed("Requirement", &base);
            let mut key = if reserved { format!("attr_{base}") } else { base };
            while !used.insert(key.clone()) {
                key.push('_');
            }
            writer.count("requirement attributes");
            writer.line(2, &format!("{key}: {literal}"));
        }
        applied_values(writer, 2, requirement, "Requirement");
        writer.line(1, "}");
    }
    writer.line(0, "}");
    writer.line(0, "");
}

/// The relations between requirements and elements, as traces: an element
/// satisfies the requirement it is related to; a requirement refines another.
pub(super) fn requirement_relations(writer: &mut Writer, project: &Node) {
    let relation_types: HashMap<String, String> = project.find_all("Requirements:RelationType").into_iter().map(|t| (t.id().to_string(), text_attr(t, "ReqIFLongName").unwrap_or_else(|| display_name(t)))).collect();
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for relation in project.descendants().into_iter().filter(|n| n.kind.ends_with("Relation") && (n.kind.starts_with("CapellaRequirements:") || n.kind.starts_with("Requirements:"))) {
        let (Some(source), Some(target)) = (first_ref(relation.attr("source")), first_ref(relation.attr("target"))) else { continue };
        if !writer.written(&source) || !writer.written(&target) {
            writer.count("requirement relations to elements not written (left out)");
            continue;
        }
        let is_requirement = |id: &str| writer.kinds.get(id) == Some(&"Requirement");
        let (element, requirement, kind) = match (is_requirement(&source), is_requirement(&target)) {
            (true, true) => (source.clone(), target.clone(), "refines"),
            (true, false) => (target.clone(), source.clone(), "satisfies"),
            (false, true) => (source.clone(), target.clone(), "satisfies"),
            (false, false) => continue,
        };
        if !seen.insert((element.clone(), requirement.clone())) {
            continue;
        }
        writer.count("requirement relations");
        let (from, to) = (writer.ids[&element].clone(), writer.ids[&requirement].clone());
        let relation_name = first_ref(relation.attr("relationType")).and_then(|t| relation_types.get(&t).cloned()).or_else(|| text_attr(relation, "ReqIFLongName"));
        match relation_name {
            Some(name) => writer.line(0, &format!("trace {} {kind} {} {{ relation: {} }}", string_literal(&from), string_literal(&to), string_literal(&name))),
            None => writer.line(0, &format!("trace {} {kind} {}", string_literal(&from), string_literal(&to))),
        }
    }
}
