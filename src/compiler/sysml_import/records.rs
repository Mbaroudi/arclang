//! The elements of a SysML v2 model as the Systems Modeling API serves them:
//! a flat list of JSON records, each with its metaclass (`@type`), its id,
//! its names and the ids it relates to. This reads that shape into a graph
//! of [`Element`]s with what the Arcadia mapping needs: owner, metaclass,
//! names, documentation, typing, specialization, direction, metadata, the
//! bound values of attributes and the two ends of connectors.
//!
//! Three dialects of the same shape are accepted: the pilot's own API
//! (`source`/`target`, `client`/`supplier`, `relatedFeature`), ArcLang's own
//! records (`arclang:sourceElement`, kept beside the standard properties),
//! and a project payload wrapping the elements (`{"elements": [...]}`).
//! What a record says that the mapping does not read is ignored, never a
//! failure: a SysML model holds much that Arcadia has no place for.

use serde_json::Value;
use std::collections::HashMap;

/// One element of the model, by its id.
#[derive(Debug, Clone, Default)]
pub struct Element {
    pub id: String,
    /// KerML / SysML metaclass, without namespace (`PartDefinition`).
    pub kind: String,
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub owner: Option<String>,
    /// Documentation bodies, in order.
    pub docs: Vec<String>,
    /// Names of the metadata definitions applied (`#OA`, `@Arcadia`).
    pub metadata: Vec<String>,
    /// Attribute values of applied metadata: (metadata def, attribute, literal).
    pub metadata_values: Vec<(String, String, String)>,
    /// Ids of the types of a feature, or the generals of a type.
    pub types: Vec<String>,
    pub specializes: Vec<String>,
    pub redefines: Vec<String>,
    pub subsets: Vec<String>,
    /// `in`, `out`, `inout` when the feature declares one.
    pub direction: Option<String>,
    pub is_end: bool,
    pub is_abstract: bool,
    /// `variation part`: a point of variation.
    pub is_variation: bool,
    /// A variant of the variation that owns it (a VariantMembership).
    pub is_variant: bool,
    /// An element of the standard library the file carries along.
    pub is_library: bool,
    /// The literal bound to the feature, rendered (`"text"`, `12`, `true`);
    /// a literal expression element carries its own.
    pub value: Option<String>,
    /// The expression element bound to the feature (`= 30 [ms]`), when any.
    pub value_expression: Option<String>,
    /// Expression pieces: the operator of an `OperatorExpression` (`+`,
    /// `<=`, `[` for a quantity, `.` for a feature chain), its arguments in
    /// order, the referent of a `FeatureReferenceExpression`, the target
    /// feature and written path of a `FeatureChainExpression`, the function
    /// of an `InvocationExpression` (an element id, else its written name).
    pub operator: Option<String>,
    pub arguments: Vec<String>,
    pub referent: Option<String>,
    pub target_feature: Option<String>,
    pub target_path: Option<String>,
    pub function: Option<String>,
    /// The written path of a connector's source end (`Vehicle.MonitorEnvironment`), when the export says it.
    pub source_path: Option<String>,
    /// The expression's text as written (`arclang:expression`), when the records carry it.
    pub source_text: Option<String>,
    /// The features a feature chain goes through (`a.b.port`), in order.
    pub chaining: Vec<String>,
    /// Ends of a connector or relationship: source ids, target ids.
    pub sources: Vec<String>,
    pub targets: Vec<String>,
    /// Transition pieces: trigger, guard and effect, as text.
    pub trigger: Option<String>,
    pub guard: Option<String>,
    pub effect: Option<String>,
    /// Children in document order (owned elements that are not relationships).
    pub children: Vec<String>,
    /// Where the record sat in the file: document order of the model.
    pub order: usize,
    /// What a view exposes: (element, recursive `::**`, namespace `::*`).
    pub exposes: Vec<(String, bool, bool)>,
    /// The metadata a view filters on (`filter @Safety;`): definition ids.
    pub filters: Vec<String>,
}

/// The model: elements by id, in document order.
#[derive(Debug, Default)]
pub struct Graph {
    pub elements: Vec<Element>,
    pub index: HashMap<String, usize>,
}

impl Graph {
    pub fn get(&self, id: &str) -> Option<&Element> {
        self.index.get(id).map(|&i| &self.elements[i])
    }

    /// The metaclass of an element, empty when unknown.
    pub fn kind_of(&self, id: &str) -> &str {
        self.get(id).map(|e| e.kind.as_str()).unwrap_or("")
    }

    /// The nearest ancestor (the element itself included) satisfying `test`.
    pub fn ancestor<'a>(&'a self, id: &str, test: impl Fn(&Element) -> bool) -> Option<&'a Element> {
        let mut current = self.get(id);
        let mut hops = 0;
        while let Some(element) = current {
            if test(element) {
                return Some(element);
            }
            hops += 1;
            if hops > 64 {
                return None;
            }
            current = element.owner.as_deref().and_then(|o| self.get(o));
        }
        None
    }

    /// Elements of the given metaclasses, in document order.
    pub fn of_kinds<'a>(&'a self, kinds: &'a [&str]) -> impl Iterator<Item = &'a Element> + 'a {
        self.elements.iter().filter(move |e| kinds.contains(&e.kind.as_str()))
    }

    /// The root elements: packages and definitions nobody owns.
    pub fn roots(&self) -> Vec<&Element> {
        self.elements.iter().filter(|e| e.owner.as_deref().and_then(|o| self.get(o)).is_none()).collect()
    }
}

/// The records of a file: a JSON array, or an object holding one under
/// `elements`, `members`, `data` or `records`.
pub fn records_of(json: &Value) -> Result<Vec<Value>, String> {
    match json {
        // The pilot's `%export`: items of {identity: {@id}, payload: {...}}.
        Value::Array(items) if items.iter().any(|i| i.get("payload").is_some()) => Ok(items
            .iter()
            .filter_map(|item| {
                let mut payload = item.get("payload")?.clone();
                if let (Some(map), Some(id)) = (payload.as_object_mut(), item.get("identity").and_then(|i| i.get("@id")).cloned()) {
                    map.entry("@id").or_insert(id);
                }
                Some(payload)
            })
            .collect()),
        Value::Array(items) => Ok(items.clone()),
        Value::Object(map) => {
            for key in ["elements", "records", "members", "data", "items"] {
                if let Some(Value::Array(items)) = map.get(key) {
                    return Ok(items.clone());
                }
            }
            if map.contains_key("@type") {
                return Ok(vec![json.clone()]);
            }
            Err("the JSON holds no list of SysML elements (an array, or an object with `elements`)".to_string())
        }
        _ => Err("the JSON is neither an array of elements nor an object".to_string()),
    }
}

const RELATIONSHIPS: &[&str] = &[
    "OwningMembership", "FeatureMembership", "Membership", "ParameterMembership", "ReturnParameterMembership", "ResultExpressionMembership",
    "ObjectiveMembership", "RequirementVerificationMembership", "RequirementConstraintMembership", "SubjectMembership", "ActorMembership",
    "StakeholderMembership", "VariantMembership", "EndFeatureMembership", "ElementFilterMembership", "StateSubactionMembership",
    "TransitionFeatureMembership", "FramedConcernMembership", "ViewRenderingMembership",
    "FeatureTyping", "Subclassification", "Specialization", "Subsetting", "Redefinition", "ReferenceSubsetting", "CrossSubsetting",
    "FeatureValue", "Conjugation", "PortConjugation", "Disjoining", "Intersecting", "Unioning", "Differencing", "FeatureInverting",
    "FeatureChaining", "TypeFeaturing", "Import", "MembershipImport", "NamespaceImport", "Annotation", "TextualRepresentation",
];

fn id_of(value: &Value) -> Option<String> {
    match value {
        Value::Object(map) => map.get("@id").and_then(Value::as_str).map(str::to_string),
        Value::String(text) => Some(text.clone()),
        _ => None,
    }
}

fn ids_of(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items.iter().filter_map(id_of).collect(),
        Some(single) => id_of(single).into_iter().collect(),
        None => Vec::new(),
    }
}

fn first_id(record: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| ids_of(record.get(key)).into_iter().next())
}

fn text(record: &Value, key: &str) -> Option<String> {
    record.get(key).and_then(Value::as_str).filter(|s| !s.is_empty()).map(str::to_string)
}

fn local_kind(record: &Value) -> String {
    let raw = record.get("@type").and_then(Value::as_str).unwrap_or("");
    raw.rsplit(':').next().unwrap_or(raw).to_string()
}

/// A literal element rendered as an ArcLang value.
fn literal(graph_kinds: &HashMap<String, (String, Value)>, id: &str) -> Option<String> {
    let (kind, record) = graph_kinds.get(id)?;
    match kind.as_str() {
        "LiteralString" => Some(crate::compiler::source_edit::string_literal(record.get("value").and_then(Value::as_str).unwrap_or(""))),
        "LiteralInteger" => record.get("value").map(|v| v.to_string()),
        "LiteralRational" => record.get("value").and_then(Value::as_f64).map(crate::compiler::ast::decimal_literal),
        "LiteralBoolean" => record.get("value").and_then(Value::as_bool).map(|b| b.to_string()),
        "LiteralInfinity" => Some("\"*\"".to_string()),
        "FeatureReferenceExpression" => first_id(record, &["referent"]).and_then(|r| graph_kinds.get(&r)).and_then(|(_, target)| {
            text(target, "declaredName").or_else(|| text(target, "name")).map(|n| crate::compiler::source_edit::string_literal(&n))
        }),
        "OperatorExpression" | "InvocationExpression" | "FeatureChainExpression" => {
            // Kept as text when the record carries one; else left out.
            text(record, "textualRepresentation").map(|t| crate::compiler::source_edit::string_literal(&t))
        }
        _ => None,
    }
}

/// Build the graph from the records.
pub fn load(records: &[Value]) -> Result<Graph, String> {
    let mut graph = Graph::default();
    let mut by_id: HashMap<String, (String, Value)> = HashMap::new();
    for record in records {
        if let Some(id) = text(record, "@id") {
            by_id.insert(id, (local_kind(record), record.clone()));
        }
    }
    if by_id.is_empty() {
        return Err("no record carries an `@id`: not a SysML v2 API listing".to_string());
    }
    // Elements first, in document order.
    for (order, record) in records.iter().enumerate() {
        let kind = local_kind(record);
        if RELATIONSHIPS.contains(&kind.as_str()) {
            continue;
        }
        let Some(id) = text(record, "@id") else { continue };
        let owner = first_id(record, &["owner", "owningNamespace", "owningType"]);
        let mut element = Element {
            id: id.clone(),
            kind: kind.clone(),
            name: text(record, "declaredName").or_else(|| text(record, "name")),
            short_name: text(record, "declaredShortName").or_else(|| text(record, "shortName")),
            owner,
            direction: text(record, "direction").map(|d| d.to_ascii_lowercase()),
            is_end: record.get("isEnd").and_then(Value::as_bool).unwrap_or(false),
            is_abstract: record.get("isAbstract").and_then(Value::as_bool).unwrap_or(false),
            is_variation: record.get("isVariation").and_then(Value::as_bool).unwrap_or(false),
            is_library: record.get("isLibraryElement").and_then(Value::as_bool).unwrap_or(false),
            order,
            ..Element::default()
        };
        element.chaining = ids_of(record.get("chainingFeature"));
        if kind.ends_with("Expression") || kind.starts_with("Literal") {
            element.operator = text(record, "operator");
            element.arguments = ids_of(record.get("argument"));
            element.referent = first_id(record, &["referent"]);
            element.target_feature = first_id(record, &["targetFeature"]);
            element.function = first_id(record, &["function", "instantiatedType"]).or_else(|| text(record, "arclang:unresolvedTarget"));
            element.value = literal(&by_id, &id);
        }
        element.source_path = text(record, "arclang:sourcePath");
        element.target_path = text(record, "arclang:targetPath");
        element.source_text = text(record, "arclang:expression").or_else(|| text(record, "textualRepresentation"));
        // Connector and relationship ends, in the dialects' order of trust:
        // ArcLang's export names the feature an end lands on (the port) and
        // the element holding it.
        element.sources = ids_of(record.get("arclang:sourceFeature"));
        element.targets = ids_of(record.get("arclang:targetFeature"));
        if element.sources.is_empty() {
            element.sources = ids_of(record.get("arclang:sourceElement"));
            element.targets = ids_of(record.get("arclang:targetElement"));
        }
        if element.sources.is_empty() {
            // ArcLang's flows of a scenario name their events.
            element.sources = ids_of(record.get("arclang:sourceEvent"));
            element.targets = ids_of(record.get("arclang:targetEvent"));
        }
        if element.sources.is_empty() && !kind.ends_with("Expression") {
            element.sources = ids_of(record.get("sourceFeature"));
            element.targets = ids_of(record.get("targetFeature"));
        }
        if element.sources.is_empty() {
            element.sources = ids_of(record.get("client"));
            element.targets = ids_of(record.get("supplier"));
        }
        if element.sources.is_empty() {
            element.sources = ids_of(record.get("source"));
            element.targets = ids_of(record.get("target"));
        }
        if element.sources.is_empty() {
            let related = ids_of(record.get("relatedFeature"));
            if related.len() >= 2 {
                element.sources = vec![related[0].clone()];
                element.targets = vec![related[1].clone()];
            }
        }
        if kind == "TransitionUsage" {
            element.trigger = first_id(record, &["triggerAction"]).and_then(|t| by_id.get(&t)).and_then(|(_, r)| text(r, "declaredName").or_else(|| text(r, "name")));
            element.guard = first_id(record, &["guardExpression"]).and_then(|g| by_id.get(&g)).and_then(|(_, r)| text(r, "textualRepresentation"));
            element.effect = first_id(record, &["effectAction"]).and_then(|e| by_id.get(&e)).and_then(|(_, r)| text(r, "declaredName").or_else(|| text(r, "name")));
        }
        if kind == "Documentation" || kind == "Comment" {
            // Kept on the owner below.
        }
        graph.index.insert(id, graph.elements.len());
        graph.elements.push(element);
    }
    // Relationships and annotations: typing, specialization, values, docs.
    for record in records {
        let kind = local_kind(record);
        match kind.as_str() {
            "FeatureTyping" => link(&mut graph, first_id(record, &["typedFeature", "specific"]), first_id(record, &["type", "general"]), |e, t| e.types.push(t)),
            "Subclassification" | "Specialization" => link(&mut graph, first_id(record, &["subclassifier", "specific"]), first_id(record, &["superclassifier", "general"]), |e, t| e.specializes.push(t)),
            "Redefinition" => link(&mut graph, first_id(record, &["redefiningFeature", "specific"]), first_id(record, &["redefinedFeature", "general"]), |e, t| e.redefines.push(t)),
            "Subsetting" | "ReferenceSubsetting" => link(&mut graph, first_id(record, &["subsettingFeature", "referencingFeature", "specific"]), first_id(record, &["subsettedFeature", "referencedFeature", "general"]), |e, t| e.subsets.push(t)),
            "VariantMembership" => {
                if let Some(&i) = first_id(record, &["ownedVariantUsage", "ownedMemberElement", "memberElement"]).as_deref().and_then(|v| graph.index.get(v)) {
                    graph.elements[i].is_variant = true;
                }
            }
            "MembershipExpose" | "NamespaceExpose" => {
                let recursive = record.get("isRecursive").and_then(Value::as_bool).unwrap_or(false);
                let namespace = kind == "NamespaceExpose";
                link(&mut graph, first_id(record, &["importOwningNamespace"]), first_id(record, &["importedElement"]), |e, t| e.exposes.push((t, recursive, namespace)));
            }
            "ElementFilterMembership" => {
                // `filter @Meta;`: an `@` expression naming a metadata definition.
                let condition = first_id(record, &["condition", "memberElement"]).and_then(|c| by_id.get(&c)).map(|(_, r)| r);
                let metadata = condition.filter(|c| c.get("operator").and_then(Value::as_str) == Some("@")).and_then(|c| first_id(c, &["arclang:type"]));
                // A filter read as nothing is kept as such: the view is not widened.
                link(&mut graph, first_id(record, &["membershipOwningNamespace"]), Some(metadata.unwrap_or_default()), |e, t| e.filters.push(t));
            }
            "FeatureValue" => {
                let owner = first_id(record, &["featureWithValue", "owner"]);
                let expression = first_id(record, &["value", "memberElement"]);
                if let Some(&i) = owner.as_deref().and_then(|o| graph.index.get(o)) {
                    graph.elements[i].value = expression.as_deref().and_then(|v| literal(&by_id, v));
                    graph.elements[i].value_expression = expression;
                }
            }
            _ => {}
        }
    }
    // Documentation and metadata are elements owned by what they annotate.
    let docs: Vec<(String, String)> = graph
        .elements
        .iter()
        .filter(|e| e.kind == "Documentation" || e.kind == "Comment")
        .filter_map(|e| {
            let body = by_id.get(&e.id).and_then(|(_, r)| text(r, "body"))?;
            let annotated = by_id.get(&e.id).and_then(|(_, r)| first_id(r, &["annotatedElement"])).or_else(|| e.owner.clone())?;
            Some((annotated, body))
        })
        .collect();
    for (owner, body) in docs {
        if let Some(&i) = graph.index.get(&owner) {
            graph.elements[i].docs.push(body);
        }
    }
    let metadata: Vec<(String, String, Vec<(String, String)>)> = graph
        .elements
        .iter()
        .filter(|e| e.kind == "MetadataUsage" || e.kind == "MetadataFeature")
        .filter_map(|e| {
            let record = &by_id.get(&e.id)?.1;
            let definition = first_id(record, &["metadataDefinition"]).or_else(|| e.types.first().cloned())?;
            let name = by_id.get(&definition).and_then(|(_, r)| text(r, "declaredName").or_else(|| text(r, "name")))?;
            let annotated = first_id(record, &["annotatedElement"]).or_else(|| e.owner.clone())?;
            let values: Vec<(String, String)> = e
                .children
                .iter()
                .filter_map(|c| graph.get(c))
                .filter_map(|c| Some((c.name.clone()?, c.value.clone()?)))
                .collect();
            Some((annotated, name, values))
        })
        .collect();
    // Children, in document order, before metadata values are read.
    let mut children: Vec<(String, String)> = Vec::new();
    for element in &graph.elements {
        if let Some(owner) = &element.owner {
            children.push((owner.clone(), element.id.clone()));
        }
    }
    for (owner, child) in children {
        if let Some(&i) = graph.index.get(&owner) {
            graph.elements[i].children.push(child);
        }
    }
    // Metadata values live in the usage's own attribute children.
    let mut applied: Vec<(String, String, Vec<(String, String)>)> = Vec::new();
    for (annotated, name, mut values) in metadata {
        if values.is_empty() {
            if let Some(usage) = graph.elements.iter().find(|e| (e.kind == "MetadataUsage" || e.kind == "MetadataFeature") && e.owner.as_deref() == Some(annotated.as_str()) && e.types.iter().any(|t| by_id.get(t).and_then(|(_, r)| text(r, "declaredName").or_else(|| text(r, "name"))).as_deref() == Some(name.as_str()))) {
                values = usage.children.iter().filter_map(|c| graph.get(c)).filter_map(|c| Some((c.name.clone()?, c.value.clone()?))).collect();
            }
        }
        applied.push((annotated, name, values));
    }
    for (annotated, name, values) in applied {
        if let Some(&i) = graph.index.get(&annotated) {
            graph.elements[i].metadata.push(name.clone());
            for (attribute, value) in values {
                graph.elements[i].metadata_values.push((name.clone(), attribute, value));
            }
        }
    }
    Ok(graph)
}

fn link(graph: &mut Graph, from: Option<String>, to: Option<String>, add: impl Fn(&mut Element, String)) {
    if let (Some(from), Some(to)) = (from, to) {
        if let Some(&i) = graph.index.get(&from) {
            add(&mut graph.elements[i], to);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn elements_relationships_docs_and_values_are_read() {
        let records = vec![
            json!({"@id": "p", "@type": "Package", "declaredName": "Logical Architecture"}),
            json!({"@id": "d", "@type": "PartDefinition", "declaredName": "Ctrl", "declaredShortName": "LC-1", "owner": {"@id": "p"}}),
            json!({"@id": "u", "@type": "PartUsage", "declaredName": "ctrl", "owner": {"@id": "p"}}),
            json!({"@id": "t", "@type": "FeatureTyping", "typedFeature": {"@id": "u"}, "type": {"@id": "d"}}),
            json!({"@id": "a", "@type": "AttributeUsage", "declaredName": "mass", "owner": {"@id": "d"}}),
            json!({"@id": "v", "@type": "FeatureValue", "featureWithValue": {"@id": "a"}, "value": {"@id": "l"}}),
            json!({"@id": "l", "@type": "LiteralRational", "value": 12.0}),
            json!({"@id": "doc", "@type": "Documentation", "body": "The controller", "owner": {"@id": "d"}}),
            json!({"@id": "c", "@type": "ConnectionUsage", "source": [{"@id": "u"}], "target": [{"@id": "d"}], "owner": {"@id": "p"}}),
        ];
        let graph = load(&records).unwrap();
        let def = graph.get("d").unwrap();
        assert_eq!(def.short_name.as_deref(), Some("LC-1"));
        assert_eq!(def.docs, vec!["The controller"]);
        assert_eq!(def.children, vec!["a", "doc"]);
        assert_eq!(graph.get("u").unwrap().types, vec!["d"]);
        assert_eq!(graph.get("a").unwrap().value.as_deref(), Some("12.0"));
        let connection = graph.get("c").unwrap();
        assert_eq!((connection.sources.as_slice(), connection.targets.as_slice()), (&["u".to_string()][..], &["d".to_string()][..]));
        assert_eq!(graph.ancestor("a", |e| e.kind == "Package").unwrap().id, "p");
    }

    #[test]
    fn a_project_payload_and_a_bare_array_are_both_listings() {
        let array = json!([{"@id": "x", "@type": "Package"}]);
        let payload = json!({"elements": [{"@id": "x", "@type": "Package"}]});
        assert_eq!(records_of(&array).unwrap().len(), 1);
        assert_eq!(records_of(&payload).unwrap().len(), 1);
        assert!(records_of(&json!(3)).is_err());
    }
}
