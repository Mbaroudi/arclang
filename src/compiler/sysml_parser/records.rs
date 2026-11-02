//! The syntax tree as Systems Modeling API records: one record per element
//! (with the property names of the KerML / SysML v2 metamodel), one per
//! relationship (memberships, specializations, imports, feature values), the
//! expressions as elements of their own, and stubs for the library elements
//! the model refers to. What a name designates is resolved by `Scope`; a
//! name designating nothing known is kept in `arclang:unresolvedTarget`.
//!
//! Besides the standard properties, connectors carry `arclang:sourceFeature`
//! and friends, the pieces `sysml_import` reads an end from.

use super::scope::Scope;
use super::syntax::*;
use crate::compiler::identity::element_uuid;
use crate::compiler::sysml_library::{self, LibraryElement, Role};
use serde_json::{json, Map, Value};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

pub struct Built {
    pub records: Vec<Value>,
    /// What could not be resolved, for the notes.
    pub unresolved: Vec<String>,
}

pub(super) struct Builder<'n> {
    pub(super) scope: Scope<'n>,
    /// Element id per flat index.
    pub(super) ids: Vec<String>,
    /// Records of the flat elements, filled progressively.
    pub(super) elements: Vec<Map<String, Value>>,
    /// Extra elements: ends, chains, expressions, parameters, metadata.
    pub(super) extras: Vec<Value>,
    pub(super) relationships: Vec<Value>,
    pub(super) owned_relationships: Vec<Vec<String>>,
    pub(super) owned_memberships: Vec<Vec<String>>,
    pub(super) owned_members: Vec<Vec<String>>,
    pub(super) library: BTreeMap<String, &'static LibraryElement>,
    /// Standard-library elements of the scope the records refer to.
    pub(super) used_library: RefCell<BTreeSet<usize>>,
    pub(super) unresolved: Vec<String>,
}

pub(super) fn reference(id: &str) -> Value {
    json!({ "@id": id })
}

pub(super) fn references(ids: &[String]) -> Value {
    Value::Array(ids.iter().map(|id| reference(id)).collect())
}

/// Metaclasses that are not elements of the model but relationships or
/// memberships the parser kept as nodes.
fn is_relationship_node(node: &Node) -> bool {
    matches!(node.detail, Detail::Import { .. } | Detail::Alias { .. } | Detail::Filter(_) | Detail::Initial(_) | Detail::Result(_))
}

fn is_type(kind: &str) -> bool {
    kind.ends_with("Definition") || matches!(kind, "Package" | "LibraryPackage")
}

fn is_feature(kind: &str) -> bool {
    kind.ends_with("Usage") || matches!(kind, "ForkNode" | "JoinNode" | "DecisionNode" | "MergeNode" | "Feature" | "Expression")
}

/// Deterministic identity of each element: its place in the model.
fn identities(scope: &Scope) -> Vec<String> {
    let mut paths: Vec<String> = Vec::with_capacity(scope.len());
    let mut ranks: std::collections::HashMap<(Option<usize>, String), usize> = std::collections::HashMap::new();
    for index in 0..scope.text_len() {
        let node = scope.node(index);
        let mut own = format!("{}:{}", node.kind, node.name.as_deref().or(node.short_name.as_deref()).unwrap_or(""));
        if let Detail::Connector { ends } = &node.detail {
            let texts: Vec<String> = ends.iter().map(|e| e.target.as_ref().map(Target::text).unwrap_or_default()).collect();
            own.push_str(&format!("({})", texts.join("->")));
        }
        let owner = scope.owner(index);
        let rank = ranks.entry((owner, own.clone())).or_insert(0);
        *rank += 1;
        if *rank > 1 {
            own.push_str(&format!("#{rank}"));
        }
        paths.push(match owner {
            Some(owner) => format!("{}/{}", paths[owner], own),
            None => own,
        });
    }
    // A library element is known by its qualified name, the id the library
    // index gives it; one without a name of its own, or sharing its name
    // with a sibling, by its place in the skeleton.
    let mut taken: std::collections::HashSet<String> = std::collections::HashSet::new();
    let library: Vec<String> = (scope.text_len()..scope.len())
        .map(|index| match scope.qualified_name(index).filter(|name| taken.insert(name.clone())) {
            Some(name) => element_uuid("sysml-library", &name),
            None => element_uuid("sysml-library", &format!("#{}", index - scope.text_len())),
        })
        .collect();
    paths.iter().map(|path| element_uuid("sysml", path)).chain(library).collect()
}

/// Build the records of a parsed text.
pub fn build(roots: &[Node]) -> Built {
    let scope = Scope::new(roots, super::library::roots());
    let ids = identities(&scope);
    let n = scope.text_len();
    let mut builder = Builder {
        scope,
        ids,
        elements: vec![Map::new(); n],
        extras: Vec::new(),
        relationships: Vec::new(),
        owned_relationships: vec![Vec::new(); n],
        owned_memberships: vec![Vec::new(); n],
        owned_members: vec![Vec::new(); n],
        used_library: RefCell::new(BTreeSet::new()),
        library: BTreeMap::new(),
        unresolved: Vec::new(),
    };
    for index in 0..n {
        builder.element(index);
    }
    for index in 0..n {
        builder.relationships_of(index);
    }
    let mut records = Vec::new();
    for index in 0..n {
        let node = builder.scope.node(index);
        if is_relationship_node(node) {
            continue;
        }
        let mut object = std::mem::take(&mut builder.elements[index]);
        object.insert("ownedRelationship".into(), references(&builder.owned_relationships[index]));
        object.insert("ownedMembership".into(), references(&builder.owned_memberships[index]));
        object.insert("ownedMember".into(), references(&builder.owned_members[index]));
        object.insert("ownedElement".into(), references(&builder.owned_members[index]));
        records.push(Value::Object(object));
    }
    records.append(&mut builder.extras);
    let mut stubbed: BTreeSet<String> = BTreeSet::new();
    for index in builder.used_library.borrow().iter().copied() {
        let node = builder.scope.node(index);
        let id = builder.ids[index].clone();
        stubbed.insert(id.clone());
        records.push(json!({
            "@id": id, "@type": node.kind, "elementId": id,
            "declaredName": node.name, "declaredShortName": node.short_name, "name": node.name, "shortName": node.short_name,
            "qualifiedName": builder.scope.qualified_name(index), "owner": Value::Null, "isLibraryElement": true,
        }));
    }
    for element in builder.library.values().filter(|e| !stubbed.contains(&library_id(e))) {
        records.push(json!({
            "@id": library_id(element),
            "@type": element.metaclass,
            "elementId": library_id(element),
            "declaredName": element.name,
            "declaredShortName": element.short_name,
            "name": element.name,
            "shortName": element.short_name,
            "qualifiedName": element.qualified_name,
            "owner": Value::Null,
            "isLibraryElement": true,
        }));
    }
    records.append(&mut builder.relationships);
    Built { records, unresolved: builder.unresolved }
}

pub(super) fn library_id(element: &LibraryElement) -> String {
    element_uuid("sysml-library", &element.qualified_name)
}

impl<'n> Builder<'n> {
    // --- identities and resolution ----------------------------------------

    /// The id of an element; a library element is recorded as referred to.
    pub(super) fn id(&self, index: usize) -> String {
        if self.scope.is_library(index) {
            self.used_library.borrow_mut().insert(index);
        }
        self.ids[index].clone()
    }

    /// A fresh id for an extra element under `owner_id`.
    pub(super) fn extra_id(&self, kind: &str, owner_id: &str, key: &str) -> String {
        element_uuid(kind, &format!("{owner_id}|{key}"))
    }

    /// The id a qualified name designates from `context`: a model element,
    /// else a library element in `role`; `None` when nothing, noted.
    pub(super) fn designate(&mut self, name: &QualifiedName, context: Option<usize>, role: Role) -> Option<String> {
        if let Some(found) = self.scope.resolve_name(name, context) {
            return Some(self.id(found));
        }
        let text = name.text();
        let element = sysml_library::lookup(&text, role).or_else(|| sysml_library::lookup(name.last(), role))?;
        self.library.insert(element.qualified_name.clone(), element);
        Some(library_id(element))
    }

    /// The id a target designates: a name, or a feature chain rendered as
    /// an anonymous feature chaining its steps (owned by `owner_id`).
    pub(super) fn designate_target(&mut self, target: &Target, context: Option<usize>, role: Role, owner_id: &str, key: &str) -> Option<String> {
        match target {
            Target::Name(name) => self.designate(name, context, role),
            Target::Chain(steps) => {
                let resolved = self.scope.resolve_chain(steps, context)?;
                let chaining: Vec<String> = resolved.iter().map(|i| self.id(*i)).collect();
                let id = self.extra_id("sysml-feature-chain", owner_id, key);
                self.extras.push(json!({
                    "@id": id, "@type": "Feature", "elementId": id, "owner": reference(owner_id),
                    "chainingFeature": references(&chaining), "isLibraryElement": false, "isImpliedIncluded": false,
                }));
                Some(id)
            }
        }
    }

    pub(super) fn note_unresolved(&mut self, what: &str, text: &str) {
        if self.unresolved.len() < 200 {
            self.unresolved.push(format!("{what} '{text}' designates nothing known"));
        }
    }

    // --- relationships ----------------------------------------------------

    /// A relationship owned by `owner_id` towards `target` (when any).
    pub(super) fn relationship(&mut self, metaclass: &str, owner_index: Option<usize>, owner_id: &str, key: &str, target: Option<&str>, mut extra: Map<String, Value>) -> String {
        let id = element_uuid("sysml-relationship", &format!("{metaclass}|{owner_id}|{key}"));
        let targets: Vec<String> = target.map(str::to_string).into_iter().collect();
        let mut object = Map::new();
        object.insert("@id".into(), json!(id));
        object.insert("@type".into(), json!(metaclass));
        object.insert("elementId".into(), json!(id));
        object.insert("owningRelatedElement".into(), reference(owner_id));
        object.insert("owner".into(), reference(owner_id));
        object.insert("source".into(), json!([reference(owner_id)]));
        object.insert("target".into(), references(&targets));
        let mut related = vec![owner_id.to_string()];
        related.extend(targets.iter().cloned());
        object.insert("relatedElement".into(), references(&related));
        object.insert("isImplied".into(), json!(false));
        object.insert("isImpliedIncluded".into(), json!(false));
        object.insert("isLibraryElement".into(), json!(false));
        object.append(&mut extra);
        if let Some(owner_index) = owner_index {
            self.owned_relationships[owner_index].push(id.clone());
        }
        self.relationships.push(Value::Object(object));
        id
    }

    /// A membership of `child_id` in `owner`.
    pub(super) fn membership(&mut self, metaclass: &str, owner: usize, child_id: &str, name: Option<&str>, short_name: Option<&str>, visibility: Option<&str>, kind: Option<&str>) -> String {
        let owner_id = self.id(owner);
        let mut extra = Map::new();
        extra.insert("membershipOwningNamespace".into(), reference(&owner_id));
        extra.insert("memberElement".into(), reference(child_id));
        extra.insert("ownedMemberElement".into(), reference(child_id));
        extra.insert("ownedRelatedElement".into(), json!([reference(child_id)]));
        extra.insert("memberName".into(), json!(name));
        extra.insert("memberShortName".into(), json!(short_name));
        extra.insert("visibility".into(), json!(visibility.unwrap_or("public")));
        if let Some(kind) = kind {
            extra.insert("kind".into(), json!(kind));
        }
        let id = self.relationship(metaclass, Some(owner), &owner_id, child_id, Some(child_id), extra);
        self.owned_memberships[owner].push(id.clone());
        self.owned_members[owner].push(child_id.to_string());
        id
    }

    /// An extra element (end, parameter, expression) with a membership in `owner`.
    pub(super) fn extra_element(&mut self, owner: usize, metaclass: &str, membership: &str, key: &str, mut fields: Map<String, Value>) -> String {
        let owner_id = self.id(owner);
        let id = self.extra_id("sysml-extra", &owner_id, key);
        let membership_id = self.membership(membership, owner, &id, fields.get("declaredName").and_then(Value::as_str), None, None, None);
        fields.insert("@id".into(), json!(id));
        fields.insert("@type".into(), json!(metaclass));
        fields.insert("elementId".into(), json!(id));
        fields.insert("owner".into(), reference(&owner_id));
        fields.insert("owningNamespace".into(), reference(&owner_id));
        fields.insert("owningMembership".into(), reference(&membership_id));
        fields.insert("owningRelationship".into(), reference(&membership_id));
        fields.entry("isLibraryElement".to_string()).or_insert(json!(false));
        fields.entry("isImpliedIncluded".to_string()).or_insert(json!(false));
        self.extras.push(Value::Object(fields));
        id
    }

    // --- elements ---------------------------------------------------------

    /// The record of element `index`: its own properties and its membership.
    fn element(&mut self, index: usize) {
        let node = self.scope.node(index);
        let owner = self.scope.owner(index);
        if is_relationship_node(node) {
            return;
        }
        let id = self.id(index);
        let mut object = Map::new();
        object.insert("@id".into(), json!(id));
        object.insert("@type".into(), json!(node.kind));
        object.insert("elementId".into(), json!(id));
        object.insert("declaredName".into(), json!(node.name));
        object.insert("declaredShortName".into(), json!(node.short_name));
        object.insert("name".into(), json!(node.name.clone().or_else(|| self.redefined_name(index))));
        object.insert("shortName".into(), json!(node.short_name));
        object.insert("qualifiedName".into(), json!(self.scope.qualified_name(index)));
        object.insert("owner".into(), owner.map(|o| reference(&self.id(o))).unwrap_or(Value::Null));
        object.insert("owningNamespace".into(), owner.map(|o| reference(&self.id(o))).unwrap_or(Value::Null));
        object.insert("aliasIds".into(), json!([]));
        object.insert("isImpliedIncluded".into(), json!(false));
        object.insert("isLibraryElement".into(), json!(false));
        let p = &node.prefixes;
        if is_type(node.kind) || is_feature(node.kind) {
            object.insert("isAbstract".into(), json!(p.is_abstract));
        }
        if is_feature(node.kind) {
            object.insert("direction".into(), json!(p.direction));
            object.insert("isEnd".into(), json!(p.is_end));
            object.insert("isDerived".into(), json!(p.is_derived));
            object.insert("isConstant".into(), json!(p.is_constant));
            object.insert("isReference".into(), json!(p.is_reference || node.kind == "ReferenceUsage"));
            object.insert("isVariation".into(), json!(p.is_variation));
            object.insert("isIndividual".into(), json!(p.is_individual));
            object.insert("portionKind".into(), json!(p.portion));
            if let Some(m) = &node.multiplicity {
                object.insert("isOrdered".into(), json!(m.ordered));
                object.insert("isUnique".into(), json!(!m.nonunique));
            }
        }
        if node.kind == "StateDefinition" || node.kind == "StateUsage" || node.kind == "ExhibitStateUsage" {
            object.insert("isParallel".into(), json!(p.is_parallel));
        }
        if node.kind == "LibraryPackage" {
            object.insert("isStandard".into(), json!(p.is_standard));
        }
        if node.kind == "AssertConstraintUsage" || node.kind == "SatisfyRequirementUsage" {
            object.insert("isNegated".into(), json!(p.is_negated));
        }
        if let Detail::Annotation { locale, language, body, .. } = &node.detail {
            object.insert("body".into(), json!(body));
            if let Some(locale) = locale {
                object.insert("locale".into(), json!(locale));
            }
            if let Some(language) = language {
                object.insert("language".into(), json!(language));
            }
        }
        self.elements[index] = object;
        // Membership in the owner.
        if let Some(owner) = owner {
            let owner_node = self.scope.node(owner);
            let membership_kind = self.membership_kind(index, owner);
            if owner_node.kind == "Dependency" || membership_kind == "Annotation" {
                self.annotation_membership(owner, index);
            } else {
                let kind_text = node.membership_kind.filter(|k| !matches!(*k, "after-then" | "message" | "cross"));
                let visibility = node.prefixes.visibility;
                let membership_id = self.membership(membership_kind, owner, &id, node.name.as_deref(), node.short_name.as_deref(), visibility, kind_text);
                self.elements[index].insert("owningMembership".into(), reference(&membership_id));
                self.elements[index].insert("owningRelationship".into(), reference(&membership_id));
            }
        }
    }

    /// The name a redefining feature takes from what it redefines.
    fn redefined_name(&self, index: usize) -> Option<String> {
        let node = self.scope.node(index);
        let context = self.scope.owner(index);
        node.specializations.iter().filter(|s| s.kind == SpecKind::Redefinition).find_map(|s| {
            let last = match &s.target {
                Target::Name(n) => n.last().to_string(),
                Target::Chain(steps) => steps.last()?.last().to_string(),
            };
            let _ = context;
            Some(last)
        })
    }

    /// The membership metaclass of `index` in `owner`: what the parser said,
    /// refined by the owner (parameters of behaviors, variants, annotations).
    fn membership_kind(&self, index: usize, owner: usize) -> &'static str {
        let node = self.scope.node(index);
        let owner_kind = self.scope.node(owner).kind;
        if matches!(node.kind, "Comment" | "Documentation" | "TextualRepresentation") || (node.kind == "MetadataUsage" && !is_type(owner_kind) && !is_feature(owner_kind)) {
            return if is_type(owner_kind) || is_feature(owner_kind) { "OwningMembership" } else { "Annotation" };
        }
        if node.membership == "FeatureMembership" && node.prefixes.direction.is_some() && is_behavior(owner_kind) {
            return "ParameterMembership";
        }
        if node.membership == "FeatureMembership" && node.prefixes.is_end {
            return "EndFeatureMembership";
        }
        node.membership
    }

    /// Metadata of a relationship (`#Trace dependency`): an annotation.
    fn annotation_membership(&mut self, owner: usize, index: usize) {
        let owner_id = self.id(owner);
        let child_id = self.id(index);
        let mut extra = Map::new();
        extra.insert("annotatedElement".into(), reference(&owner_id));
        extra.insert("owningAnnotatedElement".into(), reference(&owner_id));
        extra.insert("annotatingElement".into(), reference(&child_id));
        extra.insert("ownedAnnotatingElement".into(), reference(&child_id));
        extra.insert("ownedRelatedElement".into(), json!([reference(&child_id)]));
        let annotation = self.relationship("Annotation", Some(owner), &owner_id, &child_id, Some(&child_id), extra);
        self.elements[index].insert("owningRelationship".into(), reference(&annotation));
        self.elements[index].insert("owningNamespace".into(), Value::Null);
        self.elements[index].insert("annotatedElement".into(), json!([reference(&owner_id)]));
    }

    // --- what an element owns ---------------------------------------------

    fn relationships_of(&mut self, index: usize) {
        let node = self.scope.node(index);
        if is_relationship_node(node) {
            self.relationship_node(index);
            return;
        }
        self.prefix_metadata(index);
        self.specializations(index);
        self.multiplicity(index);
        self.value(index);
        self.detail(index);
        if node.membership_kind == Some("after-then") {
            self.implied_succession(index);
        }
    }

    /// `#Meta` prefixes: metadata usages annotating the element.
    fn prefix_metadata(&mut self, index: usize) {
        let node = self.scope.node(index);
        let id = self.id(index);
        for (rank, metadata) in node.metadata.iter().enumerate() {
            let Some(typing) = metadata.specializations.first() else { continue };
            let Target::Name(name) = &typing.target else { continue };
            let def = self.designate(name, self.scope.owner(index), Role::Type);
            if def.is_none() {
                self.note_unresolved("metadata", &name.text());
            }
            let usage_id = self.extra_id("sysml-metadata", &id, &format!("{rank}:{}", name.text()));
            let is_relationship = node.kind == "Dependency";
            let membership_id = if is_relationship {
                let mut extra = Map::new();
                extra.insert("annotatedElement".into(), reference(&id));
                extra.insert("owningAnnotatedElement".into(), reference(&id));
                extra.insert("annotatingElement".into(), reference(&usage_id));
                extra.insert("ownedAnnotatingElement".into(), reference(&usage_id));
                extra.insert("ownedRelatedElement".into(), json!([reference(&usage_id)]));
                self.relationship("Annotation", Some(index), &id, &usage_id, Some(&usage_id), extra)
            } else {
                self.membership("OwningMembership", index, &usage_id, None, None, None, None)
            };
            let mut typing_extra = Map::new();
            typing_extra.insert("typedFeature".into(), reference(&usage_id));
            typing_extra.insert("specific".into(), reference(&usage_id));
            match &def {
                Some(def) => {
                    typing_extra.insert("type".into(), reference(def));
                    typing_extra.insert("general".into(), reference(def));
                }
                None => {
                    typing_extra.insert("arclang:unresolvedTarget".into(), json!(name.text()));
                }
            }
            let typing_id = self.relationship("FeatureTyping", None, &usage_id, "type", def.as_deref(), typing_extra);
            self.extras.push(json!({
                "@id": usage_id, "@type": "MetadataUsage", "elementId": usage_id,
                "declaredName": Value::Null, "declaredShortName": Value::Null,
                "owner": reference(&id), "owningRelationship": reference(&membership_id),
                "owningMembership": if is_relationship { Value::Null } else { reference(&membership_id) },
                "annotatedElement": [reference(&id)],
                "metadataDefinition": def.as_deref().map(reference).unwrap_or(Value::Null),
                "ownedRelationship": [reference(&typing_id)],
                "isLibraryElement": false, "isImpliedIncluded": false,
            }));
        }
    }

    /// `: T`, `:> S`, `::> r`, `=> c`, `:>> r` as relationships.
    fn specializations(&mut self, index: usize) {
        let node = self.scope.node(index);
        let id = self.id(index);
        let context = self.scope.owner(index);
        for (rank, specialization) in node.specializations.iter().enumerate() {
            let (metaclass, specific_role, general_role, role): (&str, &str, &str, Role) = match specialization.kind {
                SpecKind::Typing if specialization.conjugated => ("ConjugatedPortTyping", "typedFeature", "type", Role::Type),
                SpecKind::Typing => ("FeatureTyping", "typedFeature", "type", Role::Type),
                SpecKind::Subclassification => ("Subclassification", "subclassifier", "superclassifier", Role::Type),
                SpecKind::Subsetting if is_type(node.kind) => ("Subclassification", "subclassifier", "superclassifier", Role::Type),
                SpecKind::Subsetting => ("Subsetting", "subsettingFeature", "subsettedFeature", Role::Type),
                SpecKind::Reference => ("ReferenceSubsetting", "referencingFeature", "referencedFeature", Role::Value),
                SpecKind::Crossing => ("CrossSubsetting", "crossingFeature", "crossedFeature", Role::Value),
                SpecKind::Redefinition => ("Redefinition", "redefiningFeature", "redefinedFeature", Role::Redefinition),
            };
            // A redefined feature is one the owner inherits: looked up from the
            // owner's generals before anywhere else.
            let key = format!("{rank}:{}", specialization.target.text());
            // A redefinition is looked up in the owner; other targets from
            // where their name does not designate the element itself.
            let context = if matches!(specialization.kind, SpecKind::Redefinition | SpecKind::Typing) { context } else { self.scope.specialization_context(index, &specialization.target) };
            let target = match (specialization.kind, &specialization.target, context) {
                (SpecKind::Redefinition, Target::Name(name), Some(owner)) if name.segments.len() == 1 => {
                    let inherited = self.scope.redefined(index, name.last()).or_else(|| self.scope.member(owner, name.last()).filter(|&m| m != index)).map(|m| self.id(m));
                    inherited.or_else(|| self.designate(name, context, role))
                }
                _ => self.designate_target(&specialization.target, context, role, &id, &key),
            };
            let mut extra = Map::new();
            extra.insert(specific_role.into(), reference(&id));
            extra.insert("specific".into(), reference(&id));
            match &target {
                Some(target) => {
                    extra.insert(general_role.into(), reference(target));
                    extra.insert("general".into(), reference(target));
                    if metaclass == "ConjugatedPortTyping" {
                        extra.insert("conjugatedPortDefinition".into(), reference(target));
                        extra.insert("arclang:conjugated".into(), json!(true));
                    }
                }
                None => {
                    extra.insert("arclang:unresolvedTarget".into(), json!(specialization.target.text()));
                    self.note_unresolved(metaclass, &specialization.target.text());
                }
            }
            let metaclass = if metaclass == "ConjugatedPortTyping" { "FeatureTyping" } else { metaclass };
            self.relationship(metaclass, Some(index), &id, &key, target.as_deref(), extra);
        }
    }

    /// `[l..u]`: a MultiplicityRange element with its bounds.
    fn multiplicity(&mut self, index: usize) {
        let node = self.scope.node(index);
        let Some(multiplicity) = &node.multiplicity else { return };
        if multiplicity.lower.is_none() && multiplicity.upper.is_none() {
            return;
        }
        let id = self.id(index);
        let range_id = self.extra_id("sysml-multiplicity", &id, "multiplicity");
        let membership_id = self.membership("OwningMembership", index, &range_id, None, None, None, None);
        let mut bounds = Vec::new();
        let mut fields = Map::new();
        for (role, bound) in [("lower", &multiplicity.lower), ("upper", &multiplicity.upper)] {
            let Some(expr) = bound else { continue };
            let bound_id = self.expression(expr, Some(index), &range_id, role, None);
            fields.insert(format!("{role}Bound"), reference(&bound_id));
            bounds.push(bound_id);
        }
        fields.insert("bound".into(), references(&bounds));
        fields.insert("@id".into(), json!(range_id));
        fields.insert("@type".into(), json!("MultiplicityRange"));
        fields.insert("elementId".into(), json!(range_id));
        fields.insert("owner".into(), reference(&id));
        fields.insert("owningMembership".into(), reference(&membership_id));
        fields.insert("owningRelationship".into(), reference(&membership_id));
        fields.insert("isLibraryElement".into(), json!(false));
        self.extras.push(Value::Object(fields));
        self.elements[index].insert("multiplicity".into(), reference(&range_id));
    }

    /// `= e`: a FeatureValue binding the expression; a constraint's body
    /// is its result expression.
    fn value(&mut self, index: usize) {
        let node = self.scope.node(index);
        let Some(value) = &node.value else { return };
        let id = self.id(index);
        let context = Some(index);
        let expression = self.expression(&value.expr, context, &id, "value", None);
        let mut extra = Map::new();
        extra.insert("ownedRelatedElement".into(), json!([reference(&expression)]));
        extra.insert("memberElement".into(), reference(&expression));
        extra.insert("value".into(), reference(&expression));
        extra.insert("featureWithValue".into(), reference(&id));
        extra.insert("isDefault".into(), json!(matches!(value.kind, ValueKind::Default | ValueKind::DefaultInitial)));
        extra.insert("isInitial".into(), json!(matches!(value.kind, ValueKind::Initial | ValueKind::DefaultInitial)));
        self.relationship("FeatureValue", Some(index), &id, "value", Some(&expression), extra);
        self.elements[index].insert("arclang:expression".into(), json!(expression_text(&value.expr)));
    }

    /// A result expression of a body (`{ ...; x + y }`).
    pub(super) fn result_expression(&mut self, owner: usize, expr: &Expr, key: &str) -> String {
        let owner_id = self.id(owner);
        let expression = self.expression(expr, Some(owner), &owner_id, key, None);
        let mut extra = Map::new();
        extra.insert("ownedRelatedElement".into(), json!([reference(&expression)]));
        extra.insert("memberElement".into(), reference(&expression));
        extra.insert("ownedResultExpression".into(), reference(&expression));
        extra.insert("membershipOwningNamespace".into(), reference(&owner_id));
        let id = self.relationship("ResultExpressionMembership", Some(owner), &owner_id, key, Some(&expression), extra);
        self.owned_memberships[owner].push(id);
        self.owned_members[owner].push(expression.clone());
        self.elements[owner].insert("arclang:expression".into(), json!(expression_text(expr)));
        expression
    }

    /// Imports, aliases, filters, initial nodes and result expressions: the
    /// nodes that are relationships of their owner.
    fn relationship_node(&mut self, index: usize) {
        let node = self.scope.node(index);
        let Some(owner) = self.scope.owner(index) else { return };
        let owner_id = self.id(owner);
        let context = Some(owner);
        match &node.detail {
            Detail::Import { target, namespace, recursive, all, filters } => {
                let resolved = self.designate(target, context, Role::Type);
                if resolved.is_none() {
                    self.note_unresolved("import", &target.text());
                }
                let mut extra = Map::new();
                extra.insert(if *namespace { "importedNamespace" } else { "importedMembership" }.into(), resolved.as_deref().map(reference).unwrap_or(Value::Null));
                extra.insert("importedElement".into(), resolved.as_deref().map(reference).unwrap_or(Value::Null));
                extra.insert("importOwningNamespace".into(), reference(&owner_id));
                extra.insert("isRecursive".into(), json!(recursive));
                extra.insert("isImportAll".into(), json!(all));
                extra.insert("visibility".into(), json!(node.prefixes.visibility.unwrap_or("private")));
                extra.insert("arclang:importedName".into(), json!(target.text()));
                let import_id = self.relationship(node.kind, Some(owner), &owner_id, &format!("import:{}", target.text()), resolved.as_deref(), extra);
                for (rank, filter) in filters.iter().enumerate() {
                    let expression = self.expression(filter, context, &import_id, &format!("filter{rank}"), None);
                    let mut extra = Map::new();
                    extra.insert("condition".into(), reference(&expression));
                    extra.insert("ownedRelatedElement".into(), json!([reference(&expression)]));
                    self.relationship("ElementFilterMembership", None, &import_id, &format!("filter{rank}"), Some(&expression), extra);
                }
            }
            Detail::Alias { target } => {
                let resolved = self.designate(target, context, Role::Value);
                if resolved.is_none() {
                    self.note_unresolved("alias", &target.text());
                }
                let mut extra = Map::new();
                extra.insert("membershipOwningNamespace".into(), reference(&owner_id));
                extra.insert("memberElement".into(), resolved.as_deref().map(reference).unwrap_or(Value::Null));
                extra.insert("memberName".into(), json!(node.name));
                extra.insert("memberShortName".into(), json!(node.short_name));
                extra.insert("visibility".into(), json!(node.prefixes.visibility.unwrap_or("public")));
                let id = self.relationship("Membership", Some(owner), &owner_id, &format!("alias:{}", node.name.as_deref().unwrap_or("")), resolved.as_deref(), extra);
                self.owned_memberships[owner].push(id);
            }
            Detail::Filter(expr) => {
                let expression = self.expression(expr, context, &owner_id, "filter", None);
                let mut extra = Map::new();
                extra.insert("condition".into(), reference(&expression));
                extra.insert("membershipOwningNamespace".into(), reference(&owner_id));
                extra.insert("memberElement".into(), reference(&expression));
                extra.insert("ownedRelatedElement".into(), json!([reference(&expression)]));
                let id = self.relationship("ElementFilterMembership", Some(owner), &owner_id, &format!("filter:{}", self.ids[index]), Some(&expression), extra);
                self.owned_memberships[owner].push(id);
            }
            Detail::Initial(target) => {
                let resolved = self.designate_target(target, context, Role::Value, &owner_id, "first");
                if resolved.is_none() {
                    self.note_unresolved("first", &target.text());
                }
                let mut extra = Map::new();
                extra.insert("membershipOwningNamespace".into(), reference(&owner_id));
                extra.insert("memberElement".into(), resolved.as_deref().map(reference).unwrap_or(Value::Null));
                extra.insert("arclang:initialNode".into(), json!(true));
                let id = self.relationship("Membership", Some(owner), &owner_id, "first", resolved.as_deref(), extra);
                self.owned_memberships[owner].push(id);
            }
            Detail::Result(expr) => {
                self.result_expression(owner, expr, &format!("result:{}", self.ids[index]));
            }
            _ => {}
        }
    }

    /// `then action b;`: the implied succession from the previous member.
    fn implied_succession(&mut self, index: usize) {
        let Some(owner) = self.scope.owner(index) else { return };
        let Some(previous) = self.previous_member(owner, index) else { return };
        let owner_id = self.id(owner);
        let (source_id, target_id) = (self.id(previous), self.id(index));
        let fields: Map<String, Value> = [
            ("declaredName".to_string(), Value::Null),
            ("source".to_string(), json!([reference(&source_id)])),
            ("target".to_string(), json!([reference(&target_id)])),
            ("relatedFeature".to_string(), json!([reference(&source_id), reference(&target_id)])),
            ("arclang:sourceFeature".to_string(), reference(&source_id)),
            ("arclang:targetFeature".to_string(), reference(&target_id)),
            ("arclang:sourceElement".to_string(), reference(&source_id)),
            ("arclang:targetElement".to_string(), reference(&target_id)),
            ("isEnd".to_string(), json!(false)),
            ("direction".to_string(), Value::Null),
        ]
        .into_iter()
        .collect();
        let _ = owner_id;
        self.extra_element(owner, "SuccessionAsUsage", "FeatureMembership", &format!("then:{}", self.ids[index]), fields);
    }

    /// The member declared before `index` in `owner` that a succession or
    /// transition starts from: a usage, not an annotation or relationship.
    pub(super) fn previous_member(&self, owner: usize, index: usize) -> Option<usize> {
        let children = self.scope.children(owner);
        let position = children.iter().position(|&c| c == index)?;
        children[..position].iter().rev().copied().find(|&c| {
            let node = self.scope.node(c);
            !is_relationship_node(node) && !matches!(node.kind, "Comment" | "Documentation" | "TextualRepresentation" | "MetadataUsage" | "Dependency") && !matches!(node.detail, Detail::Transition { .. })
        })
    }
}

/// Behaviors: their directed features are parameters.
fn is_behavior(kind: &str) -> bool {
    matches!(
        kind,
        "ActionDefinition" | "ActionUsage" | "PerformActionUsage" | "CalculationDefinition" | "CalculationUsage" | "ConstraintDefinition" | "ConstraintUsage"
            | "AssertConstraintUsage" | "RequirementDefinition" | "RequirementUsage" | "SatisfyRequirementUsage" | "ConcernDefinition" | "ConcernUsage"
            | "CaseDefinition" | "CaseUsage" | "AnalysisCaseDefinition" | "AnalysisCaseUsage" | "VerificationCaseDefinition" | "VerificationCaseUsage"
            | "UseCaseDefinition" | "UseCaseUsage" | "IncludeUseCaseUsage" | "StateDefinition" | "StateUsage" | "ExhibitStateUsage" | "TransitionUsage"
            | "AcceptActionUsage" | "SendActionUsage" | "AssignmentActionUsage" | "IfActionUsage" | "WhileLoopActionUsage" | "ForLoopActionUsage"
            | "TerminateActionUsage" | "ViewpointDefinition" | "ViewpointUsage" | "FlowDefinition" | "FlowUsage" | "SuccessionFlowUsage"
    )
}

/// The text of an expression, as the notation writes it.
pub(super) fn expression_text(expr: &Expr) -> String {
    match expr {
        Expr::Null => "null".to_string(),
        Expr::Literal(Literal::Boolean(b)) => b.to_string(),
        Expr::Literal(Literal::String(s)) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        Expr::Literal(Literal::Integer(n)) | Expr::Literal(Literal::Real(n)) => n.clone(),
        Expr::Literal(Literal::Infinity) => "*".to_string(),
        Expr::Reference(name) => name.text(),
        Expr::MetadataAccess(name) => format!("{}.metadata", name.text()),
        Expr::Invocation { function, arguments } => format!("{}({})", function.text(), arguments_text(arguments)),
        Expr::Constructor { ty, arguments } => format!("new {}({})", ty.text(), arguments_text(arguments)),
        Expr::Body { members } => {
            let result = members.iter().find_map(|m| if let Detail::Result(e) = &m.detail { Some(expression_text(e)) } else { None }).unwrap_or_default();
            format!("{{ {result} }}")
        }
        Expr::Sequence(items) => format!("({})", items.iter().map(expression_text).collect::<Vec<_>>().join(", ")),
        Expr::Operator { operator, operands } if operands.len() == 1 => format!("{operator}{}", expression_text(&operands[0])),
        Expr::Operator { operator, operands } => operands.iter().map(expression_text).collect::<Vec<_>>().join(&format!(" {operator} ")),
        Expr::Conditional { condition, then, otherwise } => format!("if {} ? {} else {}", expression_text(condition), expression_text(then), expression_text(otherwise)),
        Expr::TypeTest { operand, operator, ty } | Expr::Cast { operand, operator, ty } => match operand {
            Some(operand) => format!("{} {operator} {}", expression_text(operand), ty.text()),
            None => format!("{operator} {}", ty.text()),
        },
        Expr::Extent(ty) => format!("all {}", ty.text()),
        Expr::Chain { base, chain } => format!("{}.{}", expression_text(base), chain.text()),
        Expr::Bracket { base, index } => format!("{} [{}]", expression_text(base), expression_text(index)),
        Expr::Index { base, index } => format!("{}#({})", expression_text(base), expression_text(index)),
        Expr::Arrow { base, ty, argument } => {
            let argument = match argument {
                ArrowArgument::Body(body) => expression_text(body),
                ArrowArgument::Function(f) => f.text(),
                ArrowArgument::Arguments(args) => format!("({})", arguments_text(args)),
            };
            format!("{}->{}{argument}", expression_text(base), ty.text())
        }
        Expr::Collect { base, body } => format!("{}.{}", expression_text(base), expression_text(body)),
        Expr::Select { base, body } => format!("{}.?{}", expression_text(base), expression_text(body)),
    }
}

fn arguments_text(arguments: &[Argument]) -> String {
    arguments
        .iter()
        .map(|a| match &a.name {
            Some(name) => format!("{} = {}", name.text(), expression_text(&a.value)),
            None => expression_text(&a.value),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
