//! The declarations of an exported SysML v2 text as Systems Modeling API
//! records, with the property names of the KerML / SysML v2 metamodel:
//! elements, the memberships that own them, their specializations, and the
//! ends of their connectors.
//!
//! Only what the text declares is rendered. Derived properties that would
//! need the full KerML semantics (inherited members, implied
//! specializations, connector end features) are absent, not empty.
//!
//! Expressions are rendered as their KerML elements (literals, operator,
//! feature-reference, feature-chain and invocation expressions), bound by
//! `FeatureValue` or `ResultExpressionMembership`. The parameter features
//! through which KerML passes arguments are not rendered: an argument is
//! owned by the expression it belongs to and listed in `argument`.
//!
//! A standard-library element the model refers to (`String`, `kg`) is
//! rendered as a stub: its identity, metaclass and names, flagged
//! `isLibraryElement`, without its content. A name that designates neither
//! a model element nor an indexed library element has no target and is
//! kept in `arclang:unresolvedTarget`.

use super::identity::element_uuid;
use super::sysml_library::{self, LibraryElement, Role};
use super::sysml_syntax::{Declaration, Expr, Scope};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

/// Records of one model in the SysML v2 vocabulary.
pub struct Records {
    /// Declared elements in document order, then expressions, then the
    /// library elements referred to, then relationships.
    pub records: Vec<Value>,
    /// Indexes of the root elements (the model package).
    pub roots: Vec<usize>,
    /// (record index, source id, target id) of every relationship.
    pub ends: Vec<(usize, String, String)>,
}

fn reference(id: &str) -> Value {
    json!({ "@id": id })
}

fn references(ids: &[String]) -> Value {
    Value::Array(ids.iter().map(|id| reference(id)).collect())
}

fn is_type(metaclass: &str) -> bool {
    metaclass.ends_with("Definition")
}

fn is_feature(metaclass: &str) -> bool {
    metaclass.ends_with("Usage")
}

/// Deterministic identity of each declaration: its place in the model.
/// A declaration is known by its name, a connector also by what it
/// connects; what is still indistinguishable among siblings (two unnamed
/// attributes) is told apart by rank.
fn identities(declarations: &[Declaration]) -> Vec<String> {
    let mut paths: Vec<String> = Vec::with_capacity(declarations.len());
    let mut ranks: std::collections::HashMap<(Option<usize>, String), usize> = std::collections::HashMap::new();
    for declaration in declarations {
        let mut own = format!("{}:{}", declaration.metaclass, declaration.name.as_deref().unwrap_or(""));
        if let Some((source, target)) = declaration.ends.as_ref().or(declaration.events.as_ref()) {
            own.push_str(&format!("({}->{})", source, target));
        }
        let rank = ranks.entry((declaration.owner, own.clone())).or_insert(0);
        *rank += 1;
        if *rank > 1 {
            own.push_str(&format!("#{}", rank));
        }
        paths.push(match declaration.owner {
            Some(owner) => format!("{}/{}", paths[owner], own),
            None => own,
        });
    }
    paths.iter().map(|path| element_uuid("sysml", path)).collect()
}

struct Builder<'d> {
    declarations: &'d [Declaration],
    scope: Scope<'d>,
    ids: Vec<String>,
    relationships: Vec<Value>,
    ends: Vec<(usize, String, String)>,
    /// Per declaration: ids of the relationships it owns.
    owned_relationships: Vec<Vec<String>>,
    expressions: Vec<Value>,
    /// Library elements referred to, by qualified name.
    library: BTreeMap<&'static str, &'static LibraryElement>,
}

fn library_id(element: &LibraryElement) -> String {
    element_uuid("sysml-library", &element.qualified_name)
}

impl<'d> Builder<'d> {
    /// A relationship owned by `source`, towards `target` when it resolved.
    fn relationship(&mut self, metaclass: &str, key: &str, source: usize, target: Option<String>, mut extra: Map<String, Value>) -> String {
        let source_id = self.ids[source].clone();
        let id = element_uuid("sysml-relationship", &format!("{}|{}|{}", metaclass, source_id, key));
        let targets: Vec<String> = target.into_iter().collect();
        let mut object = Map::new();
        object.insert("@id".into(), json!(id));
        object.insert("@type".into(), json!(metaclass));
        object.insert("elementId".into(), json!(id));
        object.insert("owningRelatedElement".into(), reference(&source_id));
        object.insert("owner".into(), reference(&source_id));
        object.insert("source".into(), json!([reference(&source_id)]));
        object.insert("target".into(), references(&targets));
        let mut related = vec![source_id.clone()];
        related.extend(targets.iter().cloned());
        object.insert("relatedElement".into(), references(&related));
        object.insert("isImplied".into(), json!(false));
        object.insert("isImpliedIncluded".into(), json!(false));
        object.insert("isLibraryElement".into(), json!(false));
        object.append(&mut extra);
        if let Some(target) = targets.first() {
            // Indexes are fixed up once elements are laid out before relationships.
            self.ends.push((self.relationships.len(), source_id, target.clone()));
        }
        self.owned_relationships[source].push(id.clone());
        self.relationships.push(Value::Object(object));
        id
    }

    /// The identity of what `name` designates, written in position `role`
    /// inside `context`: a model element, else a library element.
    fn designated(&mut self, name: &str, context: Option<usize>, role: Role) -> Option<String> {
        if let Some(index) = self.scope.resolve(name, context) {
            return Some(self.ids[index].clone());
        }
        let element = sysml_library::lookup(name, role)?;
        self.library.insert(element.qualified_name.as_str(), element);
        Some(library_id(element))
    }

    /// `: T`, `:> S`, `:>> R` and references, as relationships of `index`.
    fn specializations(&mut self, index: usize) {
        let declaration = &self.declarations[index];
        let general_kind = if is_type(declaration.metaclass) { "Subclassification" } else { "Subsetting" };
        let groups: [(&str, &Vec<String>, (&str, &str), Role); 4] = [
            ("FeatureTyping", &declaration.typed_by, ("typedFeature", "type"), Role::Type),
            (
                general_kind,
                &declaration.specializes,
                if is_type(declaration.metaclass) { ("subclassifier", "superclassifier") } else { ("subsettingFeature", "subsettedFeature") },
                Role::Type,
            ),
            ("Redefinition", &declaration.redefines, ("redefiningFeature", "redefinedFeature"), Role::Redefinition),
            ("ReferenceSubsetting", &declaration.references, ("referencingFeature", "referencedFeature"), Role::Value),
        ];
        for (metaclass, names, (specific_role, general_role), role) in groups {
            for name in names {
                // A redefined feature is one the owner inherits, never a
                // namesake found further out.
                let target = match (role, declaration.owner) {
                    (Role::Redefinition, Some(owner)) => self
                        .scope
                        .inherited(owner, name)
                        .map(|found| self.ids[found].clone())
                        .or_else(|| self.designated_in_library(name, role)),
                    _ => self.designated(name, declaration.owner, role),
                };
                let mut extra = Map::new();
                extra.insert(specific_role.into(), reference(&self.ids[index]));
                extra.insert("specific".into(), reference(&self.ids[index]));
                match &target {
                    Some(target) => {
                        extra.insert(general_role.into(), reference(target));
                        extra.insert("general".into(), reference(target));
                    }
                    None => {
                        extra.insert("arclang:unresolvedTarget".into(), json!(name));
                    }
                }
                self.relationship(metaclass, name, index, target, extra);
            }
        }
    }

    fn designated_in_library(&mut self, name: &str, role: Role) -> Option<String> {
        let element = sysml_library::lookup(name, role)?;
        self.library.insert(element.qualified_name.as_str(), element);
        Some(library_id(element))
    }

    /// Render `expression` and what it is made of; returns its identity.
    /// `key` places it under `owner_id`: its rank among the arguments.
    fn expression(&mut self, expression: &Expr, context: Option<usize>, owner_id: &str, key: &str, owning_relationship: Option<&str>) -> String {
        let id = element_uuid("sysml-expression", &format!("{}|{}", owner_id, key));
        let mut object = Map::new();
        let mut arguments: Vec<String> = Vec::new();
        let metaclass = match expression {
            Expr::Integer(value) => {
                object.insert("value".into(), json!(value));
                "LiteralInteger"
            }
            Expr::Rational(value) => {
                object.insert("value".into(), json!(value));
                "LiteralRational"
            }
            Expr::Text(value) => {
                object.insert("value".into(), json!(value));
                "LiteralString"
            }
            Expr::Boolean(value) => {
                object.insert("value".into(), json!(value));
                "LiteralBoolean"
            }
            Expr::Reference(name) => {
                self.referent(&mut object, name, context);
                "FeatureReferenceExpression"
            }
            Expr::Chain(base, features) => {
                arguments.push(self.expression(base, context, &id, "0", None));
                let path = match base.as_ref() {
                    Expr::Reference(first) => format!("{}.{}", first, features),
                    _ => features.clone(),
                };
                let target = self.scope.resolve(&path, context).map(|found| self.ids[found].clone());
                object.insert("operator".into(), json!("."));
                object.insert("targetFeature".into(), target.as_deref().map(reference).unwrap_or(Value::Null));
                // The feature path after the base, as written: KerML holds a
                // multi-step path in an anonymous chain feature.
                object.insert("arclang:targetPath".into(), json!(features));
                "FeatureChainExpression"
            }
            Expr::Operator(operator, operands) => {
                for (rank, operand) in operands.iter().enumerate() {
                    arguments.push(self.expression(operand, context, &id, &rank.to_string(), None));
                }
                object.insert("operator".into(), json!(operator));
                "OperatorExpression"
            }
            Expr::Invocation(function, operands) => {
                for (rank, operand) in operands.iter().enumerate() {
                    arguments.push(self.expression(operand, context, &id, &rank.to_string(), None));
                }
                let target = self.designated(function, context, Role::Function);
                let target_value = target.as_deref().map(reference).unwrap_or(Value::Null);
                object.insert("function".into(), target_value.clone());
                object.insert("instantiatedType".into(), target_value);
                if target.is_none() {
                    object.insert("arclang:unresolvedTarget".into(), json!(function));
                }
                "InvocationExpression"
            }
        };
        object.insert("@id".into(), json!(id));
        object.insert("@type".into(), json!(metaclass));
        object.insert("elementId".into(), json!(id));
        object.insert("owner".into(), reference(owner_id));
        object.insert("owningRelationship".into(), owning_relationship.map(reference).unwrap_or(Value::Null));
        object.insert("ownedElement".into(), references(&arguments));
        if !matches!(expression, Expr::Integer(_) | Expr::Rational(_) | Expr::Text(_) | Expr::Boolean(_) | Expr::Reference(_)) {
            object.insert("argument".into(), references(&arguments));
        }
        object.insert("isImpliedIncluded".into(), json!(false));
        object.insert("isLibraryElement".into(), json!(false));
        self.expressions.push(Value::Object(object));
        id
    }

    fn referent(&mut self, object: &mut Map<String, Value>, name: &str, context: Option<usize>) {
        match self.designated(name, context, Role::Value) {
            Some(target) => {
                object.insert("referent".into(), reference(&target));
            }
            None => {
                object.insert("referent".into(), Value::Null);
                object.insert("arclang:unresolvedTarget".into(), json!(name));
            }
        }
    }

    /// The bounds of a multiplicity range, as the literals it owns.
    fn bounds(&mut self, index: usize, object: &mut Map<String, Value>) {
        let Some((lower, upper)) = self.declarations[index].bounds else { return };
        let range_id = self.ids[index].clone();
        let mut literal = |builder: &mut Builder, role: &str, value: Option<u64>| -> String {
            let id = element_uuid("sysml-expression", &format!("{}|{}", range_id, role));
            let membership = element_uuid("sysml-relationship", &format!("OwningMembership|{}|{}", range_id, role));
            let (metaclass, value) = match value {
                Some(value) => ("LiteralInteger", json!(value)),
                None => ("LiteralInfinity", Value::Null),
            };
            builder.expressions.push(json!({
                "@id": id,
                "@type": metaclass,
                "elementId": id,
                "value": value,
                "owner": reference(&range_id),
                "owningRelationship": reference(&membership),
                "owningMembership": reference(&membership),
                "ownedElement": [],
                "isImpliedIncluded": false,
                "isLibraryElement": false,
            }));
            let mut extra = Map::new();
            extra.insert("membershipOwningNamespace".into(), reference(&range_id));
            extra.insert("memberElement".into(), reference(&id));
            extra.insert("ownedMemberElement".into(), reference(&id));
            extra.insert("ownedRelatedElement".into(), json!([reference(&id)]));
            extra.insert("visibility".into(), json!("public"));
            let created = builder.relationship("OwningMembership", role, index, Some(id.clone()), extra);
            debug_assert_eq!(created, membership);
            id
        };
        let lower_id = lower.map(|value| literal(self, "lower", Some(value)));
        let upper_id = literal(self, "upper", upper);
        let bound: Vec<String> = lower_id.iter().cloned().chain([upper_id.clone()]).collect();
        object.insert("lowerBound".into(), lower_id.as_deref().map(reference).unwrap_or(Value::Null));
        object.insert("upperBound".into(), reference(&upper_id));
        object.insert("bound".into(), references(&bound));
    }

    /// Bind the parsed value of `index`: a feature's value, or the result
    /// expression of a constraint.
    fn value(&mut self, index: usize) {
        let declaration = &self.declarations[index];
        let Some(value) = &declaration.value else { return };
        let owner_id = self.ids[index].clone();
        let is_constraint = declaration.metaclass.contains("Constraint");
        let metaclass = if is_constraint { "ResultExpressionMembership" } else { "FeatureValue" };
        let relationship_id = element_uuid("sysml-relationship", &format!("{}|{}|value", metaclass, owner_id));
        // A feature's value is read where the feature is declared; a
        // constraint's body inside the constraint.
        let context = if is_constraint { Some(index) } else { declaration.owner };
        let expression = self.expression(value, context, &owner_id, "value", Some(&relationship_id));
        let mut extra = Map::new();
        extra.insert("ownedRelatedElement".into(), json!([reference(&expression)]));
        extra.insert("memberElement".into(), reference(&expression));
        if is_constraint {
            extra.insert("ownedResultExpression".into(), reference(&expression));
            extra.insert("membershipOwningNamespace".into(), reference(&owner_id));
        } else {
            extra.insert("value".into(), reference(&expression));
            extra.insert("featureWithValue".into(), reference(&owner_id));
            extra.insert("isDefault".into(), json!(declaration.is_default));
            extra.insert("isInitial".into(), json!(false));
        }
        let created = self.relationship(metaclass, "value", index, Some(expression), extra);
        debug_assert_eq!(created, relationship_id);
    }

    fn membership(&mut self, owner: usize, child: usize) -> String {
        let declaration = &self.declarations[child];
        let child_id = self.ids[child].clone();
        let mut extra = Map::new();
        extra.insert("membershipOwningNamespace".into(), reference(&self.ids[owner]));
        extra.insert("memberElement".into(), reference(&child_id));
        extra.insert("ownedMemberElement".into(), reference(&child_id));
        extra.insert("ownedRelatedElement".into(), json!([reference(&child_id)]));
        extra.insert("memberName".into(), json!(declaration.name));
        extra.insert("memberShortName".into(), json!(declaration.short_name));
        extra.insert("visibility".into(), json!("public"));
        self.relationship(declaration.membership, &child_id, owner, Some(child_id.clone()), extra)
    }
}

/// One end of a connector.
struct ConnectorEnd {
    path: String,
    /// What the connector relates at this end, in KerML terms: the element
    /// itself, or nothing servable when the end is a feature chain (KerML
    /// then relates an anonymous chain feature, which is not rendered).
    related: Vec<String>,
    /// The declared element the path starts from.
    element: Option<String>,
    /// The feature the whole path designates.
    feature: Option<String>,
}

fn connector_end(builder: &Builder, declaration: &Declaration, path: &str) -> ConnectorEnd {
    let first = path.split('.').next().unwrap_or(path);
    let element = builder.scope.resolve(first, declaration.owner).map(|e| builder.ids[e].clone());
    let feature = builder.scope.resolve(path, declaration.owner).map(|f| builder.ids[f].clone());
    let is_chain = path.contains('.');
    // A flow relates the features that own its output and input.
    let related = match (is_chain, declaration.metaclass) {
        (false, _) | (true, "FlowUsage") => element.clone().into_iter().collect(),
        (true, _) => Vec::new(),
    };
    ConnectorEnd { path: path.to_string(), related, element, feature }
}

/// Render declarations (owners first, as `sysml_syntax::read` yields them).
pub fn build(declarations: &[Declaration]) -> Records {
    let ids = identities(declarations);
    let mut builder = Builder {
        declarations,
        scope: Scope::new(declarations),
        ids,
        relationships: Vec::new(),
        ends: Vec::new(),
        owned_relationships: vec![Vec::new(); declarations.len()],
        expressions: Vec::new(),
        library: BTreeMap::new(),
    };

    let mut children: Vec<Vec<usize>> = vec![Vec::new(); declarations.len()];
    let mut memberships: Vec<Option<String>> = vec![None; declarations.len()];
    let mut owned_memberships: Vec<Vec<String>> = vec![Vec::new(); declarations.len()];
    for (index, declaration) in declarations.iter().enumerate() {
        if let Some(owner) = declaration.owner {
            children[owner].push(index);
            let membership = builder.membership(owner, index);
            owned_memberships[owner].push(membership.clone());
            memberships[index] = Some(membership);
        }
    }
    for index in 0..declarations.len() {
        builder.specializations(index);
        builder.value(index);
    }

    let mut records = Vec::with_capacity(declarations.len() + builder.relationships.len());
    let mut roots = Vec::new();
    let mut connector_ends = Vec::new();
    for (index, declaration) in declarations.iter().enumerate() {
        let id = &builder.ids[index];
        let child_ids: Vec<String> = children[index].iter().map(|c| builder.ids[*c].clone()).collect();
        let documentation: Vec<String> = children[index]
            .iter()
            .filter(|c| declarations[**c].metaclass == "Documentation")
            .map(|c| builder.ids[*c].clone())
            .collect();
        let mut object = Map::new();
        object.insert("@id".into(), json!(id));
        object.insert("@type".into(), json!(declaration.metaclass));
        object.insert("elementId".into(), json!(id));
        object.insert("declaredName".into(), json!(declaration.name));
        object.insert("declaredShortName".into(), json!(declaration.short_name));
        object.insert("name".into(), json!(Scope::effective_name(declaration)));
        object.insert("shortName".into(), json!(declaration.short_name));
        object.insert("qualifiedName".into(), json!(builder.scope.qualified_name(index)));
        let owner = declaration.owner.map(|o| reference(&builder.ids[o])).unwrap_or(Value::Null);
        object.insert("owner".into(), owner.clone());
        object.insert("owningNamespace".into(), owner);
        let membership = memberships[index].as_deref().map(reference).unwrap_or(Value::Null);
        object.insert("owningMembership".into(), membership.clone());
        object.insert("owningRelationship".into(), membership);
        object.insert("ownedRelationship".into(), references(&builder.owned_relationships[index]));
        object.insert("ownedMembership".into(), references(&owned_memberships[index]));
        object.insert("ownedMember".into(), references(&child_ids));
        object.insert("ownedElement".into(), references(&child_ids));
        object.insert("documentation".into(), references(&documentation));
        object.insert("aliasIds".into(), json!([]));
        object.insert("isImpliedIncluded".into(), json!(false));
        object.insert("isLibraryElement".into(), json!(false));
        if is_type(declaration.metaclass) || is_feature(declaration.metaclass) {
            object.insert("isAbstract".into(), json!(declaration.is_abstract));
        }
        if is_feature(declaration.metaclass) {
            object.insert("direction".into(), json!(declaration.direction));
        }
        if let Some(body) = &declaration.body {
            object.insert("body".into(), json!(body));
        }
        if declaration.metaclass == "MultiplicityRange" {
            builder.bounds(index, &mut object);
            object.insert("ownedRelationship".into(), references(&builder.owned_relationships[index]));
        }
        let range = children[index].iter().find(|child| declarations[**child].metaclass == "MultiplicityRange");
        if let Some(range) = range {
            object.insert("multiplicity".into(), reference(&builder.ids[*range]));
        }
        if let Some(expression) = &declaration.expression {
            object.insert("arclang:expression".into(), json!(expression));
        }
        if let Some((source_path, target_path)) = &declaration.ends {
            let ends = [connector_end(&builder, declaration, source_path), connector_end(&builder, declaration, target_path)];
            let related: Vec<String> = ends.iter().flat_map(|end| end.related.clone()).collect();
            let (source_role, target_role) = match declaration.metaclass {
                "Dependency" => ("client", "supplier"),
                _ => ("source", "target"),
            };
            for (end, role, side) in [(&ends[0], source_role, "source"), (&ends[1], target_role, "target")] {
                object.insert(role.into(), references(&end.related));
                if role != side {
                    object.insert(side.into(), references(&end.related));
                }
                // The end as written, the element it starts from and the
                // feature it designates: `p_A.out` is port `out` of `p_A`.
                object.insert(format!("arclang:{}Path", side), json!(end.path));
                object.insert(format!("arclang:{}Element", side), end.element.as_deref().map(reference).unwrap_or(Value::Null));
                object.insert(format!("arclang:{}Feature", side), end.feature.as_deref().map(reference).unwrap_or(Value::Null));
            }
            if declaration.metaclass == "SatisfyRequirementUsage" {
                object.insert("satisfyingFeature".into(), ends[0].feature.as_deref().map(reference).unwrap_or(Value::Null));
                object.insert("satisfiedRequirement".into(), ends[1].element.as_deref().map(reference).unwrap_or(Value::Null));
            }
            object.insert("relatedElement".into(), references(&related));
            if let (Some(source), Some(target)) = (&ends[0].element, &ends[1].element) {
                connector_ends.push((records.len(), source.clone(), target.clone()));
            }
        }
        if let Some((sent, received)) = &declaration.events {
            for (path, key) in [(sent, "arclang:sourceEvent"), (received, "arclang:targetEvent")] {
                let event = builder.scope.resolve(path, declaration.owner);
                object.insert(key.into(), event.map(|e| reference(&builder.ids[e])).unwrap_or(Value::Null));
                object.insert(format!("{}Path", key), json!(path));
            }
        }
        if declaration.metaclass == "TransitionUsage" {
            // A transition's source and target are those of its succession.
            let succession = children[index].iter().find_map(|child| declarations[*child].ends.as_ref());
            if let Some((source, target)) = succession {
                for (path, key) in [(source, "source"), (target, "target")] {
                    let state = builder.scope.resolve(path, Some(index));
                    object.insert(key.into(), state.map(|s| reference(&builder.ids[s])).unwrap_or(Value::Null));
                }
            }
        }
        if declaration.owner.is_none() {
            roots.push(records.len());
        }
        records.push(Value::Object(object));
    }

    records.append(&mut builder.expressions);
    for element in builder.library.values() {
        let id = library_id(element);
        records.push(json!({
            "@id": id,
            "@type": element.metaclass,
            "elementId": id,
            "declaredName": element.name,
            "declaredShortName": element.short_name,
            "name": element.name,
            "shortName": element.short_name,
            "qualifiedName": element.qualified_name,
            "owner": Value::Null,
            "ownedRelationship": [],
            "ownedMember": [],
            "ownedElement": [],
            "isImpliedIncluded": false,
            "isLibraryElement": true,
        }));
    }
    let offset = records.len();
    let mut ends = connector_ends;
    ends.extend(builder.ends.into_iter().map(|(index, source, target)| (index + offset, source, target)));
    records.extend(builder.relationships);
    Records { records, roots, ends }
}

/// Read an exported SysML v2 text and render its abstract syntax.
pub fn from_text(text: &str) -> Result<Records, String> {
    Ok(build(&super::sysml_syntax::read(text)?))
}
