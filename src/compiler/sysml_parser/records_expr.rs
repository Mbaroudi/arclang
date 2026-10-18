//! Expressions as KerML elements, and the parts of an element its detail
//! holds: connector ends, dependency ends, annotations, transitions, the
//! parameters of action nodes.

use super::records::{expression_text, reference, references, Builder};
use super::syntax::*;
use crate::compiler::sysml_library::Role;
use serde_json::{json, Map, Value};

impl<'n> Builder<'n> {
    /// Render `expr` as an element owned by `owner_id`; its identity.
    pub(super) fn expression(&mut self, expr: &Expr, context: Option<usize>, owner_id: &str, key: &str, owning_relationship: Option<&str>) -> String {
        let id = self.extra_id("sysml-expression", owner_id, key);
        let mut object = Map::new();
        let mut arguments: Vec<String> = Vec::new();
        let mut operand = |builder: &mut Self, rank: usize, e: &Expr, arguments: &mut Vec<String>| {
            let child = builder.expression(e, context, &id, &rank.to_string(), None);
            arguments.push(child);
        };
        let metaclass = match expr {
            Expr::Null => "NullExpression",
            Expr::Literal(literal) => {
                let (kind, value) = match literal {
                    Literal::Boolean(b) => ("LiteralBoolean", json!(b)),
                    Literal::String(s) => ("LiteralString", json!(s)),
                    Literal::Integer(n) => ("LiteralInteger", n.parse::<i64>().map(|v| json!(v)).unwrap_or(json!(n))),
                    Literal::Real(n) => ("LiteralRational", n.parse::<f64>().map(|v| json!(v)).unwrap_or(json!(n))),
                    Literal::Infinity => ("LiteralInfinity", Value::Null),
                };
                if !value.is_null() {
                    object.insert("value".into(), value);
                }
                kind
            }
            Expr::Reference(name) => {
                self.referent(&mut object, name, context);
                "FeatureReferenceExpression"
            }
            Expr::MetadataAccess(name) => {
                let target = self.designate(name, context, Role::Value);
                object.insert("referencedElement".into(), target.as_deref().map(reference).unwrap_or(Value::Null));
                "MetadataAccessExpression"
            }
            Expr::Invocation { function, arguments: args } | Expr::Constructor { ty: function, arguments: args } => {
                for (rank, argument) in args.iter().enumerate() {
                    operand(self, rank, &argument.value, &mut arguments);
                    if let Some(name) = &argument.name {
                        if let Some(Value::Object(last)) = self.extras.last_mut() {
                            last.insert("arclang:parameter".into(), json!(name.text()));
                        }
                    }
                }
                let target = self.designate_target(function, context, Role::Function, &id, "function");
                let target_value = target.as_deref().map(reference).unwrap_or(Value::Null);
                object.insert("function".into(), target_value.clone());
                object.insert("instantiatedType".into(), target_value);
                if target.is_none() {
                    object.insert("arclang:unresolvedTarget".into(), json!(function.text()));
                }
                if matches!(expr, Expr::Constructor { .. }) { "ConstructorExpression" } else { "InvocationExpression" }
            }
            Expr::Body { members } => {
                self.body_members(members, context, &id);
                "Expression"
            }
            Expr::Sequence(items) => {
                for (rank, item) in items.iter().enumerate() {
                    operand(self, rank, item, &mut arguments);
                }
                object.insert("operator".into(), json!(","));
                "OperatorExpression"
            }
            Expr::Operator { operator, operands } => {
                for (rank, item) in operands.iter().enumerate() {
                    operand(self, rank, item, &mut arguments);
                }
                object.insert("operator".into(), json!(operator));
                "OperatorExpression"
            }
            Expr::Conditional { condition, then, otherwise } => {
                for (rank, item) in [condition, then, otherwise].into_iter().enumerate() {
                    operand(self, rank, item, &mut arguments);
                }
                object.insert("operator".into(), json!("if"));
                "OperatorExpression"
            }
            Expr::TypeTest { operand: base, operator, ty } | Expr::Cast { operand: base, operator, ty } => {
                if let Some(base) = base {
                    operand(self, 0, base, &mut arguments);
                }
                object.insert("operator".into(), json!(operator));
                let target = self.designate(ty, context, Role::Type);
                object.insert("arclang:type".into(), target.as_deref().map(reference).unwrap_or(json!(ty.text())));
                "OperatorExpression"
            }
            Expr::Extent(ty) => {
                object.insert("operator".into(), json!("all"));
                let target = self.designate(ty, context, Role::Type);
                object.insert("arclang:type".into(), target.as_deref().map(reference).unwrap_or(json!(ty.text())));
                "OperatorExpression"
            }
            Expr::Chain { base, chain } => {
                operand(self, 0, base, &mut arguments);
                object.insert("operator".into(), json!("."));
                object.insert("arclang:targetPath".into(), json!(chain.text()));
                let target = self.chain_target(base, chain, context);
                object.insert("targetFeature".into(), target.as_deref().map(reference).unwrap_or(Value::Null));
                if target.is_none() {
                    object.insert("arclang:unresolvedTarget".into(), json!(chain.text()));
                }
                "FeatureChainExpression"
            }
            Expr::Bracket { base, index } => {
                operand(self, 0, base, &mut arguments);
                operand(self, 1, index, &mut arguments);
                object.insert("operator".into(), json!("["));
                "OperatorExpression"
            }
            Expr::Index { base, index } => {
                operand(self, 0, base, &mut arguments);
                operand(self, 1, index, &mut arguments);
                object.insert("operator".into(), json!("#"));
                "IndexExpression"
            }
            Expr::Arrow { base, ty, argument } => {
                operand(self, 0, base, &mut arguments);
                match argument {
                    ArrowArgument::Body(body) => operand(self, 1, body, &mut arguments),
                    ArrowArgument::Function(f) => operand(self, 1, &Expr::Reference(f.clone()), &mut arguments),
                    ArrowArgument::Arguments(args) => {
                        for (rank, a) in args.iter().enumerate() {
                            operand(self, rank + 1, &a.value, &mut arguments);
                        }
                    }
                }
                let target = self.designate_target(ty, context, Role::Function, &id, "function");
                let target_value = target.as_deref().map(reference).unwrap_or(Value::Null);
                object.insert("function".into(), target_value.clone());
                object.insert("instantiatedType".into(), target_value);
                if target.is_none() {
                    object.insert("arclang:unresolvedTarget".into(), json!(ty.text()));
                }
                "InvocationExpression"
            }
            Expr::Collect { base, body } => {
                operand(self, 0, base, &mut arguments);
                operand(self, 1, body, &mut arguments);
                "CollectExpression"
            }
            Expr::Select { base, body } => {
                operand(self, 0, base, &mut arguments);
                operand(self, 1, body, &mut arguments);
                "SelectExpression"
            }
        };
        object.insert("@id".into(), json!(id));
        object.insert("@type".into(), json!(metaclass));
        object.insert("elementId".into(), json!(id));
        object.insert("owner".into(), reference(owner_id));
        object.insert("owningRelationship".into(), owning_relationship.map(reference).unwrap_or(Value::Null));
        object.insert("ownedElement".into(), references(&arguments));
        if !arguments.is_empty() {
            object.insert("argument".into(), references(&arguments));
        }
        object.insert("isImpliedIncluded".into(), json!(false));
        object.insert("isLibraryElement".into(), json!(false));
        self.extras.push(Value::Object(object));
        id
    }

    fn referent(&mut self, object: &mut Map<String, Value>, name: &QualifiedName, context: Option<usize>) {
        match self.designate(name, context, Role::Value) {
            Some(target) => {
                object.insert("referent".into(), reference(&target));
            }
            None => {
                object.insert("referent".into(), Value::Null);
                object.insert("arclang:unresolvedTarget".into(), json!(name.text()));
            }
        }
    }

    /// The feature a chain expression designates: the chain resolved from
    /// the base's referent.
    fn chain_target(&mut self, base: &Expr, chain: &Target, context: Option<usize>) -> Option<String> {
        let mut steps: Vec<QualifiedName> = match base {
            Expr::Reference(name) => vec![name.clone()],
            Expr::Chain { base: inner, chain: inner_chain } => {
                let Expr::Reference(name) = &**inner else { return None };
                let mut steps = vec![name.clone()];
                match inner_chain {
                    Target::Name(n) => steps.push(n.clone()),
                    Target::Chain(more) => steps.extend(more.iter().cloned()),
                }
                steps
            }
            _ => return None,
        };
        match chain {
            Target::Name(n) => steps.push(n.clone()),
            Target::Chain(more) => steps.extend(more.iter().cloned()),
        }
        let resolved = self.scope.resolve_chain(&steps, context)?;
        resolved.last().map(|i| self.id(*i))
    }

    /// The members of a body expression: parameters, members, a result.
    fn body_members(&mut self, members: &[Node], context: Option<usize>, body_id: &str) {
        for (rank, member) in members.iter().enumerate() {
            match &member.detail {
                Detail::Result(expr) => {
                    let expression = self.expression(expr, context, body_id, "result", None);
                    let mut extra = Map::new();
                    extra.insert("ownedRelatedElement".into(), json!([reference(&expression)]));
                    extra.insert("memberElement".into(), reference(&expression));
                    extra.insert("ownedResultExpression".into(), reference(&expression));
                    self.relationship("ResultExpressionMembership", None, body_id, "result", Some(&expression), extra);
                }
                _ => {
                    let param_id = self.extra_id("sysml-parameter", body_id, &format!("{rank}:{}", member.name.as_deref().unwrap_or("")));
                    let mut extra = Map::new();
                    extra.insert("memberElement".into(), reference(&param_id));
                    extra.insert("ownedRelatedElement".into(), json!([reference(&param_id)]));
                    extra.insert("memberName".into(), json!(member.name));
                    let membership = self.relationship(if member.prefixes.direction.is_some() { "ParameterMembership" } else { "FeatureMembership" }, None, body_id, &format!("member{rank}"), Some(&param_id), extra);
                    self.extras.push(json!({
                        "@id": param_id, "@type": member.kind, "elementId": param_id, "declaredName": member.name, "declaredShortName": member.short_name,
                        "direction": member.prefixes.direction, "owner": reference(body_id), "owningMembership": reference(&membership),
                        "arclang:expression": member.value.as_ref().map(|v| expression_text(&v.expr)),
                        "isLibraryElement": false, "isImpliedIncluded": false,
                    }));
                }
            }
        }
    }

    // --- details ------------------------------------------------------------

    pub(super) fn detail(&mut self, index: usize) {
        let node = self.scope.node(index);
        let owner = self.scope.owner(index);
        let context = owner;
        let id = self.id(index);
        match &node.detail {
            Detail::Connector { ends } => self.connector(index, ends),
            Detail::Dependency { clients, suppliers } => {
                let mut sides = Vec::new();
                for (role, names, side) in [("client", clients, "source"), ("supplier", suppliers, "target")] {
                    let mut resolved = Vec::new();
                    for name in names {
                        match self.designate(name, context, Role::Value) {
                            Some(found) => resolved.push(found),
                            None => self.note_unresolved("dependency end", &name.text()),
                        }
                    }
                    self.elements[index].insert(role.into(), references(&resolved));
                    self.elements[index].insert(side.into(), references(&resolved));
                    if let Some(first) = resolved.first() {
                        self.elements[index].insert(format!("arclang:{side}Element"), reference(first));
                        self.elements[index].insert(format!("arclang:{side}Feature"), reference(first));
                    }
                    self.elements[index].insert(format!("arclang:{side}Path"), json!(names.iter().map(QualifiedName::text).collect::<Vec<_>>().join(", ")));
                    sides.extend(resolved);
                }
                self.elements[index].insert("relatedElement".into(), references(&sides));
            }
            Detail::Annotation { about, .. } => {
                let mut annotated = Vec::new();
                for name in about {
                    match self.designate(name, context, Role::Value) {
                        Some(found) => annotated.push(found),
                        None => self.note_unresolved("about", &name.text()),
                    }
                }
                if annotated.is_empty() {
                    if let Some(owner) = owner {
                        annotated.push(self.id(owner));
                    }
                }
                self.elements[index].insert("annotatedElement".into(), references(&annotated));
                // An annotation of named elements is a relationship each.
                for target in &annotated {
                    if owner.map(|o| self.id(o)).as_deref() != Some(target.as_str()) {
                        let mut extra = Map::new();
                        extra.insert("annotatedElement".into(), reference(target));
                        extra.insert("annotatingElement".into(), reference(&id));
                        self.relationship("Annotation", Some(index), &id, target, Some(target), extra);
                    }
                }
            }
            Detail::Transition { source, trigger, guard, effect, target, is_else } => {
                let source_id = match source {
                    Some(target) => self.designate_target(target, context, Role::Value, &id, "source"),
                    None => owner.and_then(|o| self.previous_member(o, index)).map(|p| self.id(p)),
                };
                if source.is_some() && source_id.is_none() {
                    self.note_unresolved("transition source", &source.as_ref().map(Target::text).unwrap_or_default());
                }
                let target_id = target.target.as_ref().and_then(|t| self.designate_target(t, context, Role::Value, &id, "target"));
                if target_id.is_none() {
                    self.note_unresolved("transition target", &target.target.as_ref().map(Target::text).unwrap_or_default());
                }
                let element = &mut self.elements[index];
                element.insert("source".into(), source_id.as_deref().map(|s| json!([reference(s)])).unwrap_or(json!([])));
                element.insert("target".into(), target_id.as_deref().map(|t| json!([reference(t)])).unwrap_or(json!([])));
                element.insert("arclang:sourceFeature".into(), source_id.as_deref().map(reference).unwrap_or(Value::Null));
                element.insert("arclang:targetFeature".into(), target_id.as_deref().map(reference).unwrap_or(Value::Null));
                element.insert("arclang:sourcePath".into(), json!(source.as_ref().map(Target::text)));
                element.insert("arclang:targetPath".into(), json!(target.target.as_ref().map(Target::text)));
                element.insert("arclang:isElse".into(), json!(is_else));
                // Trigger and effect are children (flattened); the guard an expression.
                let children: Vec<usize> = self.scope.children(index).to_vec();
                for child in children {
                    let child_node = self.scope.node(child);
                    let child_id = self.id(child);
                    match child_node.membership_kind {
                        Some("trigger") => {
                            self.elements[index].insert("triggerAction".into(), json!([reference(&child_id)]));
                            // The trigger as text: what the accept accepts.
                            let text = self.accept_text(child_node);
                            self.elements[child].insert("name".into(), json!(text));
                        }
                        Some("effect") => {
                            self.elements[index].insert("effectAction".into(), json!([reference(&child_id)]));
                            let text = child_node.name.clone().or_else(|| child_node.specializations.first().map(|s| s.target.text()));
                            self.elements[child].insert("name".into(), json!(text));
                        }
                        _ => {}
                    }
                }
                let _ = trigger;
                let _ = effect;
                if let Some(guard) = guard {
                    let expression = self.expression(guard, Some(index), &id, "guard", None);
                    let mut extra = Map::new();
                    extra.insert("kind".into(), json!("guard"));
                    extra.insert("memberElement".into(), reference(&expression));
                    extra.insert("ownedRelatedElement".into(), json!([reference(&expression)]));
                    self.relationship("TransitionFeatureMembership", Some(index), &id, "guard", Some(&expression), extra);
                    self.elements[index].insert("guardExpression".into(), json!([reference(&expression)]));
                    if let Some(Value::Object(record)) = self.extras.iter_mut().rev().find(|r| r.get("@id").and_then(Value::as_str) == Some(expression.as_str())) {
                        record.insert("textualRepresentation".into(), json!(expression_text(guard)));
                        record.insert("arclang:expression".into(), json!(expression_text(guard)));
                    }
                }
                // The succession the transition owns.
                let mut fields = Map::new();
                fields.insert("declaredName".into(), Value::Null);
                fields.insert("source".into(), source_id.as_deref().map(|s| json!([reference(s)])).unwrap_or(json!([])));
                fields.insert("target".into(), target_id.as_deref().map(|t| json!([reference(t)])).unwrap_or(json!([])));
                fields.insert("isEnd".into(), json!(false));
                self.extra_element(index, "SuccessionAsUsage", "OwningMembership", "succession", fields);
            }
            Detail::Accept { payload, via } => {
                let _ = payload; // flattened as a child
                if let Some(via) = via {
                    self.expression_parameter(index, "via", via, "ParameterMembership");
                }
                let text = self.accept_text(node);
                self.elements[index].insert("arclang:accepts".into(), json!(text));
            }
            Detail::Send { payload, via, to } => {
                for (key, expr) in [("payload", payload), ("via", via), ("to", to)] {
                    if let Some(expr) = expr {
                        self.expression_parameter(index, key, expr, "ParameterMembership");
                    }
                }
            }
            Detail::Assign { target, value } => {
                self.expression_parameter(index, "target", target, "ParameterMembership");
                self.expression_parameter(index, "value", value, "ParameterMembership");
            }
            Detail::If { condition, .. } => {
                self.expression_parameter(index, "condition", condition, "ParameterMembership");
            }
            Detail::While { condition, until, .. } => {
                if let Some(condition) = condition {
                    self.expression_parameter(index, "condition", condition, "ParameterMembership");
                }
                if let Some(until) = until {
                    self.expression_parameter(index, "until", until, "ParameterMembership");
                }
            }
            Detail::For { collection, .. } => {
                self.expression_parameter(index, "collection", collection, "ParameterMembership");
            }
            Detail::Terminate(target) => {
                if let Some(target) = target {
                    self.expression_parameter(index, "terminated", target, "ParameterMembership");
                }
            }
            Detail::Satisfy { by } => {
                // The satisfying feature and the satisfied requirement.
                let requirement = node.specializations.iter().find(|s| s.kind == SpecKind::Reference || s.kind == SpecKind::Typing).and_then(|s| self.designate_target(&s.target, context, Role::Type, &id, "requirement"));
                let satisfying = by.as_ref().and_then(|e| match e {
                    Expr::Reference(name) => self.designate(name, context, Role::Value),
                    Expr::Chain { base, chain } => self.chain_target(base, chain, context),
                    _ => None,
                });
                if let Some(by) = by {
                    self.expression_parameter(index, "by", by, "SubjectMembership");
                }
                let element = &mut self.elements[index];
                element.insert("satisfiedRequirement".into(), requirement.as_deref().map(reference).unwrap_or(Value::Null));
                element.insert("satisfyingFeature".into(), satisfying.as_deref().map(reference).unwrap_or(Value::Null));
                element.insert("source".into(), satisfying.as_deref().map(|s| json!([reference(s)])).unwrap_or(json!([])));
                element.insert("target".into(), requirement.as_deref().map(|r| json!([reference(r)])).unwrap_or(json!([])));
                element.insert("arclang:sourceFeature".into(), satisfying.as_deref().map(reference).unwrap_or(Value::Null));
                element.insert("arclang:targetFeature".into(), requirement.as_deref().map(reference).unwrap_or(Value::Null));
            }
            Detail::Trigger(trigger) => {
                let expression = self.expression(&trigger.expr, context, &id, "trigger", None);
                let trigger_id = self.extra_id("sysml-expression", &id, "trigger-invocation");
                self.extras.push(json!({
                    "@id": trigger_id, "@type": "TriggerInvocationExpression", "elementId": trigger_id, "kind": trigger.kind,
                    "owner": reference(&id), "argument": [reference(&expression)], "ownedElement": [reference(&expression)],
                    "isLibraryElement": false, "isImpliedIncluded": false,
                }));
                let mut extra = Map::new();
                extra.insert("value".into(), reference(&trigger_id));
                extra.insert("featureWithValue".into(), reference(&id));
                extra.insert("memberElement".into(), reference(&trigger_id));
                self.relationship("FeatureValue", Some(index), &id, "trigger", Some(&trigger_id), extra);
                self.elements[index].insert("arclang:expression".into(), json!(format!("{} {}", trigger.kind, expression_text(&trigger.expr))));
            }
            _ => {}
        }
    }

    /// What an accept accepts, as text: its payload's name or type, or its trigger.
    fn accept_text(&self, node: &Node) -> Option<String> {
        let Detail::Accept { payload, .. } = &node.detail else { return node.name.clone() };
        if let Detail::Trigger(trigger) = &payload.detail {
            return Some(format!("{} {}", trigger.kind, expression_text(&trigger.expr)));
        }
        payload.specializations.first().map(|s| s.target.text()).or_else(|| payload.name.clone()).or_else(|| payload.value.as_ref().map(|v| expression_text(&v.expr)))
    }

    /// A parameter bound to an expression (`via x`, `send e`).
    fn expression_parameter(&mut self, owner: usize, key: &str, expr: &Expr, membership: &str) {
        let owner_id = self.id(owner);
        let mut fields = Map::new();
        fields.insert("declaredName".into(), Value::Null);
        fields.insert("arclang:parameter".into(), json!(key));
        fields.insert("arclang:expression".into(), json!(expression_text(expr)));
        let param_id = self.extra_element(owner, "ReferenceUsage", membership, key, fields);
        let expression = self.expression(expr, Some(owner), &param_id, "value", None);
        let mut extra = Map::new();
        extra.insert("value".into(), reference(&expression));
        extra.insert("featureWithValue".into(), reference(&param_id));
        extra.insert("memberElement".into(), reference(&expression));
        self.relationship("FeatureValue", None, &param_id, "value", Some(&expression), extra);
        let _ = owner_id;
    }

    /// The ends of a connector, flow, message, binding or succession.
    fn connector(&mut self, index: usize, ends: &[End]) {
        let node = self.scope.node(index);
        let owner = self.scope.owner(index);
        let context = owner;
        let id = self.id(index);
        let is_flow = matches!(node.kind, "FlowUsage" | "SuccessionFlowUsage");
        let is_message = node.membership_kind == Some("message");
        let mut end_ids = Vec::new();
        let mut related: Vec<String> = Vec::new();
        let mut sides: Vec<(Option<String>, Option<String>, Option<String>, String)> = Vec::new(); // (related, element, feature, path)
        for (rank, end) in ends.iter().enumerate() {
            let key = format!("end{rank}");
            let path = end.target.as_ref().map(Target::text).unwrap_or_default();
            // What the end designates: the base feature, the first element,
            // and the feature a flow end names after the last `.`.
            let (base_ids, feature_id, element_id) = match &end.target {
                Some(target) => {
                    let resolved = self.scope.resolve_target(target, context);
                    let base_ids: Option<Vec<String>> = resolved.as_ref().map(|r| r.iter().map(|i| self.id(*i)).collect());
                    let element_id = base_ids.as_ref().and_then(|b| b.first().cloned());
                    let mut feature_id = base_ids.as_ref().and_then(|b| b.last().cloned());
                    if let (Some(flow_feature), Some(resolved)) = (&end.flow_feature, &resolved) {
                        if let Some(last) = resolved.last() {
                            match self.scope.member(*last, flow_feature.last()) {
                                Some(found) => feature_id = Some(self.id(found)),
                                None => self.note_unresolved("flow end feature", &format!("{path}.{}", flow_feature.text())),
                            }
                        }
                    }
                    if resolved.is_none() {
                        self.note_unresolved("connector end", &path);
                    }
                    (base_ids, feature_id, element_id)
                }
                None if end.flow_feature.is_some() => {
                    // `flow from x to y`: the feature itself.
                    let name = end.flow_feature.clone().unwrap();
                    let found = self.designate(&name, context, Role::Value);
                    if found.is_none() {
                        self.note_unresolved("flow end", &name.text());
                    }
                    (found.clone().map(|f| vec![f]), found.clone(), found)
                }
                None => {
                    // An empty source end: the previous member.
                    let previous = owner.and_then(|o| self.previous_member(o, index)).map(|p| self.id(p));
                    (previous.clone().map(|p| vec![p]), previous.clone(), previous)
                }
            };
            // The end feature: a reference usage subsetting what it designates.
            let referenced = match (&end.target, &base_ids) {
                (Some(target), Some(_)) => self.designate_target(target, context, Role::Value, &id, &key),
                _ => base_ids.as_ref().and_then(|b| b.last().cloned()),
            };
            let (end_kind, membership) = if is_message { ("EventOccurrenceUsage", "ParameterMembership") } else if is_flow { ("FlowEnd", "EndFeatureMembership") } else { ("ReferenceUsage", "EndFeatureMembership") };
            let mut fields = Map::new();
            fields.insert("declaredName".into(), json!(end.name));
            fields.insert("isEnd".into(), json!(!is_message));
            fields.insert("arclang:path".into(), json!(path));
            let end_id = self.extra_element(index, end_kind, membership, &key, fields);
            let mut extra = Map::new();
            extra.insert("referencingFeature".into(), reference(&end_id));
            extra.insert("specific".into(), reference(&end_id));
            match &referenced {
                Some(target) => {
                    extra.insert("referencedFeature".into(), reference(target));
                    extra.insert("general".into(), reference(target));
                }
                None => {
                    extra.insert("arclang:unresolvedTarget".into(), json!(path));
                }
            }
            self.relationship("ReferenceSubsetting", None, &end_id, "references", referenced.as_deref(), extra);
            if let (true, Some(feature), Some(flow_feature)) = (is_flow, &feature_id, &end.flow_feature) {
                // The flow's feature: a redefinition of the port or item.
                let flow_feature_id = self.extra_id("sysml-extra", &end_id, "feature");
                let mut extra = Map::new();
                extra.insert("redefiningFeature".into(), reference(&flow_feature_id));
                extra.insert("redefinedFeature".into(), reference(feature));
                extra.insert("general".into(), reference(feature));
                let redefinition = self.relationship("Redefinition", None, &flow_feature_id, "redefines", Some(feature), extra);
                self.extras.push(json!({
                    "@id": flow_feature_id, "@type": "ReferenceUsage", "elementId": flow_feature_id, "declaredName": Value::Null,
                    "name": flow_feature.last(), "owner": reference(&end_id), "ownedRelationship": [reference(&redefinition)],
                    "isLibraryElement": false, "isImpliedIncluded": false,
                }));
            }
            end_ids.push(end_id);
            // In KerML a flow relates the features owning its ports; a
            // connector its ends' referenced features.
            let related_here = if is_flow { base_ids.as_ref().and_then(|b| b.last().cloned()) } else { referenced.clone() };
            if let Some(r) = &related_here {
                related.push(r.clone());
            }
            sides.push((related_here, element_id, feature_id, path));
        }
        let element = &mut self.elements[index];
        element.insert("connectorEnd".into(), references(&end_ids));
        element.insert("relatedFeature".into(), references(&related));
        element.insert("relatedElement".into(), references(&related));
        let source: Vec<String> = sides.first().and_then(|s| s.0.clone()).into_iter().collect();
        let target: Vec<String> = sides.iter().skip(1).filter_map(|s| s.0.clone()).collect();
        element.insert("source".into(), references(&source));
        element.insert("target".into(), references(&target));
        for (side, (_, element_id, feature_id, path)) in [("source", sides.first()), ("target", sides.get(1))].into_iter().filter_map(|(s, v)| v.map(|v| (s, v))) {
            element.insert(format!("arclang:{side}Path"), json!(path));
            element.insert(format!("arclang:{side}Element"), element_id.as_deref().map(reference).unwrap_or(Value::Null));
            element.insert(format!("arclang:{side}Feature"), feature_id.as_deref().map(reference).unwrap_or(Value::Null));
        }
    }
}
