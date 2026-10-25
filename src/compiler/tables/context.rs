//! What every kind of table reads: the element graph indexed, the
//! metamodel, the verdicts of the verification cases and the gate.

use super::{CellEdit, Link};
use crate::compiler::ast::AttributeValue;
use crate::compiler::elements::{self, ElementGraph, ElementRecord, RelationshipRecord};
use crate::compiler::metamodel::{AttrType, Layer, Metamodel};
use crate::compiler::verification::{self, Verdict};
use crate::compiler::CompilationResult;
use std::collections::HashMap;

pub struct Context<'a> {
    pub result: &'a CompilationResult,
    pub graph: ElementGraph,
    pub metamodel: Metamodel,
    by_uuid: HashMap<String, usize>,
    by_id: HashMap<String, usize>,
    by_name: HashMap<String, Vec<usize>>,
    /// Test case id -> `pass`, `fail` (with why) or `pending`.
    pub verdicts: HashMap<String, (String, Option<String>)>,
    /// Test case id -> the values its simulation evaluated to.
    pub evidence: HashMap<String, String>,
    pub gate_standard: String,
    pub gate_passed: bool,
    /// The gate's blockers, each with the message naming its element.
    pub gate_blockers: Vec<(String, String)>,
}

/// The layer a scope names: `SA`, `System`, `System Analysis`...
pub fn layer_named(scope: &str) -> Option<Layer> {
    let key: String = scope.to_ascii_lowercase().chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    Some(match key.as_str() {
        "oa" | "operational" | "operationalanalysis" => Layer::Operational,
        "sa" | "system" | "systemanalysis" => Layer::System,
        "la" | "logical" | "logicalarchitecture" => Layer::Logical,
        "pa" | "physical" | "physicalarchitecture" => Layer::Physical,
        "epbs" => Layer::Epbs,
        "data" | "datamodel" => Layer::Data,
        "transverse" => Layer::Transverse,
        _ => return None,
    })
}

/// The method order of the layers: a lower layer realizes an upper one.
pub fn layer_rank(layer: Layer) -> usize {
    match layer {
        Layer::Operational => 0,
        Layer::System => 1,
        Layer::Logical => 2,
        Layer::Physical => 3,
        Layer::Epbs => 4,
        Layer::Data => 5,
        Layer::Transverse => 6,
    }
}

impl<'a> Context<'a> {
    pub fn new(result: &'a CompilationResult, gate_standard: &str) -> Context<'a> {
        let graph = elements::build(&result.ast, &result.semantic_model);
        let mut by_uuid = HashMap::new();
        let mut by_id = HashMap::new();
        let mut by_name: HashMap<String, Vec<usize>> = HashMap::new();
        for (index, element) in graph.elements.iter().enumerate() {
            by_uuid.insert(element.uuid.clone(), index);
            by_id.entry(element.id.clone()).or_insert(index);
            by_name.entry(element.name.clone()).or_default().push(index);
        }
        let verdicts = result
            .ast
            .test_cases
            .iter()
            .map(|tc| {
                let verdict = match verification::verdict(tc, &result.semantic_model) {
                    Verdict::Pass => ("pass".to_string(), None),
                    Verdict::Pending => ("pending".to_string(), None),
                    Verdict::Fail(why) => ("fail".to_string(), Some(why)),
                    Verdict::Inconclusive(why) => ("inconclusive".to_string(), Some(why)),
                };
                (tc.id.clone(), verdict)
            })
            .collect();
        let evidence = result
            .semantic_model
            .simulations
            .iter()
            .filter(|(_, status)| !status.evidence.is_empty())
            .map(|(id, status)| (id.clone(), status.evidence.clone()))
            .collect();
        let standard = normalize_standard(gate_standard);
        let gate = crate::compiler::production_gate::run_gate(&result.ast, &result.semantic_model, &standard);
        let gate_blockers = gate
            .findings
            .iter()
            .filter(|f| matches!(f.severity, crate::compiler::production_gate::Severity::Blocker))
            .map(|f| (f.check.clone(), f.message.clone()))
            .collect();
        Context {
            result,
            metamodel: Metamodel::current(),
            graph,
            by_uuid,
            by_id,
            by_name,
            verdicts,
            evidence,
            gate_standard: standard,
            gate_passed: gate.passed,
            gate_blockers,
        }
    }

    pub fn element(&self, index: usize) -> &ElementRecord {
        &self.graph.elements[index]
    }

    pub fn index_of_uuid(&self, uuid: &str) -> Option<usize> {
        self.by_uuid.get(uuid).copied()
    }

    /// The element a reference names: its id, a dotted path to a port
    /// (`Owner.port`, the owner by id or name), else its unique name.
    pub fn resolve(&self, reference: &str) -> Option<usize> {
        let reference = reference.trim();
        if let Some(&index) = self.by_id.get(reference) {
            return Some(index);
        }
        if let Some((owner, member)) = reference.rsplit_once('.') {
            if let Some(owner) = self.resolve(owner) {
                let owner_uuid = &self.graph.elements[owner].uuid;
                let found = self.graph.elements.iter().position(|e| e.owner.as_deref() == Some(owner_uuid) && (e.name == member || e.id.ends_with(&format!(".{member}"))));
                if found.is_some() {
                    return found;
                }
            }
        }
        match self.by_name.get(reference).map(Vec::as_slice) {
            Some([one]) => Some(*one),
            Some([first, ..]) => Some(*first),
            _ => None,
        }
    }

    pub fn link(&self, index: usize) -> Link {
        let e = &self.graph.elements[index];
        Link { id: e.id.clone(), uuid: e.uuid.clone(), name: if e.name.is_empty() { e.id.clone() } else { e.name.clone() }, kind: e.kind.to_string(), verdict: None }
    }

    /// The layer of an element: its kind's, a table's scope for a table.
    pub fn layer_of(&self, index: usize) -> Layer {
        let element = &self.graph.elements[index];
        if element.kind == "Table" {
            return element.attributes.get("scope").and_then(|v| v.as_string()).and_then(layer_named).unwrap_or(Layer::Transverse);
        }
        self.metamodel.kind(element.kind).map(|k| k.layer).unwrap_or(Layer::Transverse)
    }

    /// The owners of an element, nearest first.
    pub fn owners(&self, index: usize) -> Vec<usize> {
        let mut out = Vec::new();
        let mut current = self.graph.elements[index].owner.clone();
        while let Some(uuid) = current {
            let Some(&owner) = self.by_uuid.get(&uuid) else { break };
            if out.contains(&owner) {
                break;
            }
            out.push(owner);
            current = self.graph.elements[owner].owner.clone();
        }
        out
    }

    /// Whether an element is in a scope: of the layer it names, or under
    /// the element it names. No scope is the whole model.
    pub fn in_scope(&self, index: usize, scope: Option<&str>) -> bool {
        let Some(scope) = scope.map(str::trim).filter(|s| !s.is_empty()) else { return true };
        if let Some(layer) = layer_named(scope) {
            return self.layer_of(index) == layer;
        }
        match self.resolve(scope) {
            Some(container) => container == index || self.owners(index).contains(&container),
            None => false,
        }
    }

    /// Whether a scope means something in this model.
    pub fn check_scope(&self, scope: Option<&str>) -> Result<(), String> {
        match scope.map(str::trim).filter(|s| !s.is_empty()) {
            Some(scope) if layer_named(scope).is_none() && self.resolve(scope).is_none() => {
                Err(format!("scope '{scope}' is neither a layer (OA, SA, LA, PA, EPBS) nor an element of the model"))
            }
            _ => Ok(()),
        }
    }

    /// The elements of the given kinds (or layers: `LA` is every element of
    /// the logical level), in graph order.
    pub fn elements_of(&self, selectors: &[String], scope: Option<&str>) -> Vec<usize> {
        (0..self.graph.elements.len())
            .filter(|&i| {
                let e = &self.graph.elements[i];
                e.kind != "Model"
                    && self.in_scope(i, scope)
                    && selectors.iter().any(|s| s.eq_ignore_ascii_case(e.kind) || layer_named(s).is_some_and(|l| l == self.layer_of(i) && !e.kind.ends_with("Port")))
            })
            .collect()
    }

    pub fn relationships(&self, kind: &str) -> impl Iterator<Item = &RelationshipRecord> {
        let kind = kind.to_string();
        self.graph.relationships.iter().filter(move |r| r.kind == kind)
    }

    /// The edit of one attribute of an element, typed by the metamodel.
    pub fn attribute_edit(&self, index: usize, key: &str) -> Option<CellEdit> {
        let element = &self.graph.elements[index];
        if element.kind == "Model" {
            return None;
        }
        let spec = self.metamodel.kind(element.kind).and_then(|k| k.attributes.iter().find(|a| a.key == key));
        let (value_type, values) = match spec.map(|s| &s.ty) {
            Some(AttrType::Enum(name)) => ("Enum".to_string(), self.metamodel.enums.iter().find(|e| e.name == *name).map(|e| e.values.iter().map(|v| v.to_string()).collect()).unwrap_or_default()),
            Some(AttrType::Quantity(_)) => ("Quantity".to_string(), Vec::new()),
            Some(AttrType::Number) => ("Number".to_string(), Vec::new()),
            Some(AttrType::Boolean) => ("Boolean".to_string(), Vec::new()),
            Some(AttrType::Reference(_)) => ("Reference".to_string(), Vec::new()),
            Some(AttrType::List(_)) => ("List".to_string(), Vec::new()),
            Some(AttrType::Identifier) => return None,
            _ => match element.attributes.get(key) {
                Some(AttributeValue::Quantity(_)) => ("Quantity".to_string(), Vec::new()),
                Some(AttributeValue::Number(_) | AttributeValue::Integer(_)) => ("Number".to_string(), Vec::new()),
                Some(AttributeValue::Boolean(_)) => ("Boolean".to_string(), Vec::new()),
                Some(AttributeValue::List(_)) => ("List".to_string(), Vec::new()),
                _ => ("Text".to_string(), Vec::new()),
            },
        };
        Some(CellEdit { op: "set".to_string(), element: element.id.clone(), key: Some(key.to_string()), value_type: Some(value_type), values })
    }

    pub fn rename_edit(&self, index: usize) -> Option<CellEdit> {
        let element = &self.graph.elements[index];
        (element.kind != "Model").then(|| CellEdit { op: "rename".to_string(), element: element.id.clone(), key: None, value_type: Some("Text".to_string()), values: Vec::new() })
    }

    /// The gate's blockers that name this element (`'REQ-1'`).
    pub fn blockers_of(&self, index: usize) -> Vec<String> {
        let element = &self.graph.elements[index];
        let quoted = [format!("'{}'", element.id), format!("'{}'", element.name)];
        self.gate_blockers.iter().filter(|(_, message)| quoted.iter().any(|q| message.contains(q.as_str()))).map(|(_, message)| message.clone()).collect()
    }

    /// The declared table of that name, as an element.
    pub fn declared(&self, name: &str) -> Option<&ElementRecord> {
        self.graph.elements.iter().find(|e| e.kind == "Table" && (e.name == name || e.id == name))
    }
}

/// The standard as the gate spells it.
pub fn normalize_standard(standard: &str) -> String {
    match standard.to_ascii_uppercase().replace(['-', '_', ' '], "").as_str() {
        "DO178C" => "DO178C".to_string(),
        "IEC61508" => "IEC61508".to_string(),
        _ => "ISO26262".to_string(),
    }
}

/// An attribute value as a cell shows it.
pub fn shown(value: &AttributeValue) -> String {
    match value {
        AttributeValue::String(text) => text.clone(),
        AttributeValue::List(items) => items.iter().map(shown).collect::<Vec<_>>().join(", "),
        other => other.display(),
    }
}
