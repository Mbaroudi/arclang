use super::ast::*;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct SemanticModel {
    /// Model name from the `model <Name>` header, when present.
    pub name: Option<String>,
    pub requirements: Vec<RequirementInfo>,
    pub components: Vec<ComponentInfo>,
    pub functions: Vec<FunctionInfo>,
    pub traces: Vec<TraceInfo>,
    pub interfaces: Vec<InterfaceInfo>,
    pub all_elements: HashMap<String, ElementInfo>,
}

impl Default for SemanticModel {
    fn default() -> Self {
        Self {
            name: None,
            requirements: Vec::new(),
            components: Vec::new(),
            functions: Vec::new(),
            traces: Vec::new(),
            interfaces: Vec::new(),
            all_elements: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct InterfaceInfo {
    pub name: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequirementInfo {
    pub id: String,
    pub description: String,
    pub priority: String,
    pub category: Option<String>,
    pub safety_level: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentInfo {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub level: String,
    pub safety_level: Option<String>,
    pub asil: Option<String>,
    pub interfaces_in: Vec<InterfacePortInfo>,
    pub interfaces_out: Vec<InterfacePortInfo>,
    pub functions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InterfacePortInfo {
    pub name: String,
    pub protocol: Option<String>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionInfo {
    pub id: String,
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceInfo {
    pub from: String,
    pub to: String,
    pub trace_type: String,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ElementInfo {
    pub id: String,
    pub name: String,
    pub element_type: String,
    /// Deterministic stable identity (UUIDv5 of the element id in the
    /// ArcLang namespace). Same id -> same uuid, everywhere, always.
    pub uuid: String,
}

impl ElementInfo {
    pub fn new(id: impl Into<String>, name: impl Into<String>, element_type: impl Into<String>) -> Self {
        let id = id.into();
        let uuid = super::identity::element_uuid("element", &id);
        Self {
            id,
            name: name.into(),
            element_type: element_type.into(),
            uuid,
        }
    }
}

impl ComponentInfo {
    /// Stable identity; consistent with the `all_elements` registry entry.
    pub fn uuid(&self) -> String {
        super::identity::element_uuid("element", &self.id)
    }
}

impl RequirementInfo {
    pub fn uuid(&self) -> String {
        super::identity::element_uuid("element", &self.id)
    }
}

impl FunctionInfo {
    pub fn uuid(&self) -> String {
        super::identity::element_uuid("element", &self.id)
    }
}

impl TraceInfo {
    /// Stable identity of the trace link itself.
    pub fn uuid(&self) -> String {
        super::identity::element_uuid("trace", &format!("{}|{}|{}", self.from, self.trace_type, self.to))
    }
}


/// Register an element, recording a warning when an id is reused by a
/// DIFFERENT element (identity must be unique across the whole model).
fn register_element(
    elements: &mut HashMap<String, ElementInfo>,
    duplicates: &mut Vec<String>,
    key: String,
    info: ElementInfo,
) {
    if let Some(existing) = elements.get(&key) {
        if existing.name != info.name || existing.element_type != info.element_type {
            duplicates.push(format!(
                "duplicate element id '{}': {} '{}' and {} '{}' share the same identity — give one an explicit unique id",
                key, existing.element_type, existing.name, info.element_type, info.name
            ));
        }
    }
    elements.insert(key, info);
}

/// Arcadia methodology lints: advisory checks on how the model uses the
/// method's layers. Surfaced by `arclang check --lint`, not by every build —
/// partial models (single-layer studies) are legitimate working states.
pub fn arcadia_methodology_lints(ast: &Model) -> Vec<String> {
    let mut lints = Vec::new();

    if !ast.physical_architecture.is_empty() && ast.logical_architecture.is_empty() {
        lints.push(
            "physical architecture without a logical architecture: PA elements have nothing to realize"
                .to_string(),
        );
    }
    if !ast.logical_architecture.is_empty() && ast.system_analysis.is_empty() {
        lints.push(
            "logical architecture without a system analysis: no system functions or requirements to allocate"
                .to_string(),
        );
    }
    if !ast.system_analysis.is_empty() && ast.operational_analysis.is_empty() {
        lints.push(
            "system analysis without an operational analysis: no operational need is captured"
                .to_string(),
        );
    }

    for la in &ast.logical_architecture {
        for comp in &la.components {
            lint_functionless_components(comp, &mut lints);
        }
    }

    lints
}

fn lint_functionless_components(comp: &LogicalComponent, lints: &mut Vec<String>) {
    if comp.functions.is_empty() && comp.sub_components.is_empty() {
        lints.push(format!(
            "logical component '{}' performs no functions — Arcadia expects logical components to realize behavior",
            comp.name
        ));
    }
    for sub in &comp.sub_components {
        lint_functionless_components(sub, lints);
    }
}

pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn analyze(&self, ast: &Model) -> Result<SemanticModel, String> {
        self.analyze_with_warnings(ast).map(|(model, _)| model)
    }

    /// Analyze and also return non-fatal diagnostics (unresolved exchange
    /// endpoints, ...). Dangling trace references remain hard errors.
    pub fn analyze_with_warnings(
        &self,
        ast: &Model,
    ) -> Result<(SemanticModel, Vec<String>), String> {
        let mut requirements = Vec::new();
        let mut components = Vec::new();
        let mut functions = Vec::new();
        let mut traces = Vec::new();
        let mut interfaces = Vec::new();
        let mut all_elements = HashMap::new();
        let mut duplicate_ids: Vec<String> = Vec::new();
        
        // Collect actors from operational analysis
        for oa in &ast.operational_analysis {
            for actor in &oa.actors {
                // Try actor.id first, then check attributes
                let actor_id = actor.id.as_ref().cloned()
                    .or_else(|| actor.attributes.get("id").and_then(|v| v.as_string()).map(|s| s.to_string()))
                    .unwrap_or_else(|| format!("ACT-{}", actor.name.replace(" ", "-")));
                
                let actor_type = actor.attributes.get("category")
                    .and_then(|v| v.as_string())
                    .unwrap_or("Actor")
                    .to_string();
                
                let safety_level = actor.attributes.get("safety_level")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                
                let asil = actor.attributes.get("asil")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                
                components.push(ComponentInfo {
                    id: actor_id.clone(),
                    name: actor.name.clone(),
                    component_type: actor_type,
                    level: "Operational".to_string(),
                    safety_level,
                    asil,
                    interfaces_in: Vec::new(),
                    interfaces_out: Vec::new(),
                    functions: Vec::new(),
                });
                
                register_element(&mut all_elements, &mut duplicate_ids, actor_id.clone(), ElementInfo::new(actor_id.clone(), actor.name.clone(), "Actor"));
            }
            
            // Collect traces from operational_analysis
            for trace in &oa.traces {
                traces.push(TraceInfo {
                    from: trace.from.clone(),
                    to: trace.to.clone(),
                    trace_type: trace.trace_type.clone(),
                    rationale: trace.attributes.get("rationale").and_then(|v| v.as_string()).map(|s| s.to_string()),
                });
            }
            
            // Collect operational entities with their nested activities
            for entity in &oa.entities {
                let entity_type = match entity.entity_type {
                    EntityType::Actor => "Actor",
                    EntityType::System => "System",
                    EntityType::Environment => "Environment",
                }.to_string();
                
                let safety_level = entity.attributes.get("safety_level")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                
                let asil = entity.attributes.get("asil")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                
                // Collect activity IDs that belong to this entity
                let mut entity_function_ids = Vec::new();
                for activity in &entity.activities {
                    let activity_id = activity.attributes.get("id")
                        .and_then(|v| v.as_string())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| activity.id.clone());
                    entity_function_ids.push(activity_id.clone());
                    
                    // Register activity as function
                    let inputs = activity.attributes.get("inputs")
                        .and_then(|v| match v {
                            AttributeValue::List(list) => Some(list),
                            _ => None,
                        })
                        .map(|arr| arr.iter().filter_map(|v| v.as_string().map(|s| s.to_string())).collect())
                        .unwrap_or_else(Vec::new);
                    
                    let outputs = activity.attributes.get("outputs")
                        .and_then(|v| match v {
                            AttributeValue::List(list) => Some(list),
                            _ => None,
                        })
                        .map(|arr| arr.iter().filter_map(|v| v.as_string().map(|s| s.to_string())).collect())
                        .unwrap_or_else(Vec::new);
                    
                    functions.push(FunctionInfo {
                        id: activity_id.clone(),
                        name: activity.name.clone(),
                        inputs,
                        outputs,
                    });
                    
                    register_element(&mut all_elements, &mut duplicate_ids, activity_id.clone(), ElementInfo::new(activity_id, activity.name.clone(), "Activity"));
                }
                
                components.push(ComponentInfo {
                    id: entity.id.clone(),
                    name: entity.name.clone(),
                    component_type: entity_type,
                    level: "Operational".to_string(),
                    safety_level,
                    asil,
                    interfaces_in: Vec::new(),
                    interfaces_out: Vec::new(),
                    functions: entity_function_ids,
                });
                
                register_element(&mut all_elements, &mut duplicate_ids, entity.id.clone(), ElementInfo::new(entity.id.clone(), entity.name.clone(), "Entity"));
            }
            
            // Collect operational activities (recursively handle sub-activities)
            fn collect_activities_recursive(
                activity: &OperationalActivity,
                components: &mut Vec<ComponentInfo>,
                all_elements: &mut HashMap<String, ElementInfo>,
                duplicates: &mut Vec<String>,
            ) {
                // Use ID from attributes if available, otherwise use the struct id
                let activity_id = activity.attributes.get("id")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| activity.id.clone());
                
                let safety_level = activity.attributes.get("safety_level")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                
                let asil = activity.attributes.get("asil")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                
                components.push(ComponentInfo {
                    id: activity_id.clone(),
                    name: activity.name.clone(),
                    component_type: "OperationalActivity".to_string(),
                    level: "Operational".to_string(),
                    safety_level,
                    asil,
                    interfaces_in: Vec::new(),
                    interfaces_out: Vec::new(),
                    functions: Vec::new(),
                });
                
                register_element(all_elements, duplicates, activity_id.clone(), ElementInfo::new(activity_id.clone(), activity.name.clone(), "OperationalActivity"));
                
                // Recursively collect sub-activities
                for sub_activity in &activity.sub_activities {
                    collect_activities_recursive(sub_activity, components, all_elements, duplicates);
                }
            }
            
            for activity in &oa.activities {
                let attr_id = activity.attributes.get("id")
                    .and_then(|v| v.as_string())
                    .unwrap_or(&activity.id);
                collect_activities_recursive(activity, &mut components, &mut all_elements, &mut duplicate_ids);
            }
        }
        
        // Collect requirements from system analysis
        for sa in &ast.system_analysis {
            // Functional exchanges are data flows of the canonical model,
            // same as component and physical exchanges.
            for exchange in &sa.functional_exchanges {
                interfaces.push(InterfaceInfo {
                    name: exchange
                        .label
                        .clone()
                        .unwrap_or_else(|| format!("{} -> {}", exchange.from_port, exchange.to_port)),
                    from: exchange.from_port.clone(),
                    to: exchange.to_port.clone(),
                });
            }

            for req in &sa.requirements {
                // Use ID from attributes if available, otherwise use struct id
                let req_id = req.attributes.get("id")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| req.id.clone());
                
                let description = req.attributes.get("description")
                    .and_then(|v| v.as_string())
                    .unwrap_or("")
                    .to_string();
                
                let priority = req.attributes.get("priority")
                    .and_then(|v| v.as_string())
                    .unwrap_or("Medium")
                    .to_string();
                
                let category = req.attributes.get("category")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                
                let safety_level = req.attributes.get("safety_level")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                
                requirements.push(RequirementInfo {
                    id: req_id.clone(),
                    description,
                    priority,
                    category,
                    safety_level,
                });
                
                register_element(&mut all_elements, &mut duplicate_ids, req_id.clone(), ElementInfo::new(req_id.clone(), req_id.clone(), "Requirement"));
            }
            
            // Collect system components
            for comp in &sa.components {
                let comp_id = comp.attributes.get("id")
                    .and_then(|v| v.as_string())
                    .unwrap_or(&comp.name)
                    .to_string();
                
                let comp_type = comp.attributes.get("type")
                    .and_then(|v| v.as_string())
                    .unwrap_or("System")
                    .to_string();
                
                let safety_level = comp.attributes.get("safety_level")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                
                let asil = comp.attributes.get("asil")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                
                components.push(ComponentInfo {
                    id: comp_id.clone(),
                    name: comp.name.clone(),
                    component_type: comp_type,
                    level: "System".to_string(),
                    safety_level,
                    asil,
                    interfaces_in: Vec::new(),
                    interfaces_out: Vec::new(),
                    functions: Vec::new(),
                });
                
                register_element(&mut all_elements, &mut duplicate_ids, comp_id.clone(), ElementInfo::new(comp_id.clone(), comp.name.clone(), "SystemComponent"));
            }
            
            // Collect system functions (recursively handle sub-functions)
            fn collect_system_functions_recursive(
                func: &SystemFunction,
                all_elements: &mut HashMap<String, ElementInfo>,
                duplicates: &mut Vec<String>,
            ) {
                let func_id = func.attributes.get("id")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| func.id.clone());

                register_element(all_elements, duplicates, func_id.clone(), ElementInfo::new(func_id.clone(), func.name.clone(), "SystemFunction"));

                // Recursively collect sub-functions
                for sub_func in &func.sub_functions {
                    collect_system_functions_recursive(sub_func, all_elements, duplicates);
                }
            }

            for func in &sa.functions {
                collect_system_functions_recursive(func, &mut all_elements, &mut duplicate_ids);
            }
        }
        
        // Collect components and interfaces from logical architecture
        for la in &ast.logical_architecture {
            // Collect interfaces
            for interface in &la.interfaces {
                interfaces.push(InterfaceInfo {
                    name: interface.name.clone(),
                    from: interface.from.clone(),
                    to: interface.to.clone(),
                });
            }
            
            // Collect component_exchanges as interfaces
            for exchange in &la.component_exchanges {
                interfaces.push(InterfaceInfo {
                    name: exchange.label.clone().unwrap_or_else(|| format!("{} -> {}", exchange.from_port, exchange.to_port)),
                    from: exchange.from_port.clone(),
                    to: exchange.to_port.clone(),
                });
            }
            
            // Collect logical components recursively: nested sub-components are
            // model elements in their own right (identity, trace/exchange targets).
            fn collect_logical_component(
                comp: &LogicalComponent,
                components: &mut Vec<ComponentInfo>,
                functions: &mut Vec<FunctionInfo>,
                all_elements: &mut HashMap<String, ElementInfo>,
                duplicates: &mut Vec<String>,
            ) {
                let comp_id = comp.attributes.get("id")
                    .and_then(|v| v.as_string())
                    .unwrap_or(&comp.name)
                    .to_string();

                let comp_type = comp.attributes.get("type")
                    .and_then(|v| v.as_string())
                    .unwrap_or("Logical")
                    .to_string();

                let safety_level = comp.attributes.get("safety_level")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());

                let asil = comp.attributes.get("asil")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());

                let interfaces_in: Vec<InterfacePortInfo> = comp.interfaces_in.iter()
                    .map(|iface| InterfacePortInfo {
                        name: iface.name.clone(),
                        protocol: iface.protocol.clone(),
                        format: iface.format.clone(),
                    })
                    .collect();

                let interfaces_out: Vec<InterfacePortInfo> = comp.interfaces_out.iter()
                    .map(|iface| InterfacePortInfo {
                        name: iface.name.clone(),
                        protocol: iface.protocol.clone(),
                        format: iface.format.clone(),
                    })
                    .collect();

                let comp_functions: Vec<String> = comp.functions.iter()
                    .map(|f| f.name.clone())
                    .collect();

                let layer = comp.attributes.get("layer")
                    .and_then(|v| v.as_string())
                    .unwrap_or("Logical")
                    .to_string();

                components.push(ComponentInfo {
                    id: comp_id.clone(),
                    name: comp.name.clone(),
                    component_type: comp_type,
                    level: layer,
                    safety_level,
                    asil,
                    interfaces_in,
                    interfaces_out,
                    functions: comp_functions,
                });

                register_element(all_elements, duplicates, comp_id.clone(), ElementInfo::new(comp_id.clone(), comp.name.clone(), "Component"));

                for interface_def in &comp.interfaces_in {
                    register_element(all_elements, duplicates, 
                        format!("{}_{}", comp_id, interface_def.name),
                        ElementInfo::new(
                            format!("{}_{}", comp_id, interface_def.name),
                            format!("{} IN", interface_def.name),
                            "InterfaceIn",
                        ),
                    );
                }

                for interface_def in &comp.interfaces_out {
                    register_element(all_elements, duplicates, 
                        format!("{}_{}", comp_id, interface_def.name),
                        ElementInfo::new(
                            format!("{}_{}", comp_id, interface_def.name),
                            format!("{} OUT", interface_def.name),
                            "InterfaceOut",
                        ),
                    );
                }

                for func in &comp.functions {
                    let func_id = func.attributes.get("id")
                        .and_then(|v| v.as_string())
                        .unwrap_or(&func.name)
                        .to_string();

                    let inputs = func.attributes.get("inputs")
                        .and_then(|v| {
                            if let AttributeValue::List(list) = v {
                                Some(list.iter()
                                    .filter_map(|v| v.as_string().map(|s| s.to_string()))
                                    .collect())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();

                    let outputs = func.attributes.get("outputs")
                        .and_then(|v| {
                            if let AttributeValue::List(list) = v {
                                Some(list.iter()
                                    .filter_map(|v| v.as_string().map(|s| s.to_string()))
                                    .collect())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();

                    functions.push(FunctionInfo {
                        id: func_id.clone(),
                        name: func.name.clone(),
                        inputs,
                        outputs,
                    });

                    register_element(all_elements, duplicates, func_id.clone(), ElementInfo::new(func_id.clone(), func.name.clone(), "Function"));
                }

                for sub in &comp.sub_components {
                    collect_logical_component(sub, components, functions, all_elements, duplicates);
                }
            }

            for comp in &la.components {
                collect_logical_component(comp, &mut components, &mut functions, &mut all_elements, &mut duplicate_ids);
            }
        }
        
        // Collect components from physical architecture (nodes)
        for pa in &ast.physical_architecture {
            // Collect physical exchanges as interfaces so the canonical model
            // carries every cross-component data flow (logical and physical)
            for exchange in &pa.physical_exchanges {
                interfaces.push(InterfaceInfo {
                    name: exchange.label.clone().unwrap_or_else(|| format!("{} -> {}", exchange.from, exchange.to)),
                    from: exchange.from.clone(),
                    to: exchange.to.clone(),
                });
            }

            for node in &pa.nodes {
                let node_id = node.attributes.get("id")
                    .and_then(|v| v.as_string())
                    .unwrap_or(&node.name)
                    .to_string();
                
                let node_type = node.attributes.get("type")
                    .and_then(|v| v.as_string())
                    .unwrap_or("Physical")
                    .to_string();
                
                let safety_level = node.attributes.get("safety_level")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                
                let asil = node.attributes.get("asil")
                    .and_then(|v| v.as_string())
                    .map(|s| s.to_string());
                
                let layer = node.attributes.get("layer")
                    .and_then(|v| v.as_string())
                    .unwrap_or("Physical")
                    .to_string();
                
                components.push(ComponentInfo {
                    id: node_id.clone(),
                    name: node.name.clone(),
                    component_type: node_type,
                    level: layer,
                    safety_level,
                    asil,
                    interfaces_in: Vec::new(),
                    interfaces_out: Vec::new(),
                    functions: Vec::new(),
                });
                
                register_element(&mut all_elements, &mut duplicate_ids, node_id.clone(), ElementInfo::new(node_id.clone(), node.name.clone(), "Component"));
            }
        }
        
        // Collect traces
        for trace in &ast.traces {
            let rationale = trace.attributes.get("rationale")
                .and_then(|v| v.as_string())
                .map(|s| s.to_string());
            
            traces.push(TraceInfo {
                from: trace.from.clone(),
                to: trace.to.clone(),
                trace_type: trace.trace_type.clone(),
                rationale,
            });
        }
        
        // Resolve trace endpoints. Dangling references are compile errors:
        // a trace that points at nothing must never be silently dropped.
        let resolved_traces = Self::resolve_traces(traces, &all_elements)?;

        // Exchange endpoints are checked but only warned about for now:
        // port paths (Component.Port) are not first-class elements yet.
        let mut warnings = duplicate_ids;
        warnings.extend(Self::check_exchange_endpoints(ast, &all_elements));

        let name = ast
            .attributes
            .get("name")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());

        Ok((
            SemanticModel {
                name,
                requirements,
                components,
                functions,
                traces: resolved_traces,
                interfaces,
                all_elements,
            },
            warnings,
        ))
    }

    /// Check that exchange/link endpoints reference known elements.
    /// A reference resolves if it matches an element id, an element name, or
    /// a dotted path whose first segment matches (Component.Port).
    fn check_exchange_endpoints(
        ast: &Model,
        elements: &HashMap<String, ElementInfo>,
    ) -> Vec<String> {
        let names: std::collections::HashSet<&str> =
            elements.values().map(|e| e.name.as_str()).collect();

        let resolves = |reference: &str| -> bool {
            if reference.is_empty() {
                return true; // absence is a parser-level concern
            }
            let root = reference.split('.').next().unwrap_or(reference);
            elements.contains_key(reference)
                || names.contains(reference)
                || elements.contains_key(root)
                || names.contains(root)
        };

        let mut warnings = Vec::new();
        let mut check = |kind: &str, label: &str, endpoint: &str, role: &str| {
            if !resolves(endpoint) {
                warnings.push(format!(
                    "{} '{}': '{}' ({}) does not match any declared element",
                    kind, label, endpoint, role
                ));
            }
        };

        for sa in &ast.system_analysis {
            for ex in &sa.functional_exchanges {
                let label = ex.label.as_deref().unwrap_or("");
                check("functional_exchange", label, &ex.from_port, "from");
                check("functional_exchange", label, &ex.to_port, "to");
            }
        }
        for la in &ast.logical_architecture {
            for ex in &la.component_exchanges {
                let label = ex.label.as_deref().unwrap_or("");
                check("component_exchange", label, &ex.from_port, "from");
                check("component_exchange", label, &ex.to_port, "to");
            }
        }
        for pa in &ast.physical_architecture {
            for ex in &pa.physical_exchanges {
                let label = ex.label.as_deref().unwrap_or("");
                check("physical_exchange", label, &ex.from, "from");
                check("physical_exchange", label, &ex.to, "to");
            }
            for link in &pa.links {
                check("link", "", &link.from, "from");
                check("link", "", &link.to, "to");
            }
            // Arcadia allocation rules: deployments and behavior-component
            // allocations must point at declared elements.
            for node in &pa.nodes {
                for deployment in &node.deployments {
                    check("deployment", &node.name, &deployment.component, "component");
                }
                for behavior in &node.behavior_components {
                    for function in &behavior.allocated_functions {
                        check("behavior_component", &behavior.name, function, "allocated");
                    }
                }
            }
        }

        warnings
    }

    /// Resolve each trace endpoint against the element registry, by id first
    /// and by (unambiguous) name second, normalizing endpoints to element ids.
    /// Unresolved or ambiguous references are errors.
    fn resolve_traces(
        traces: Vec<TraceInfo>,
        elements: &HashMap<String, ElementInfo>,
    ) -> Result<Vec<TraceInfo>, String> {
        // Name index: name -> ids (a name may be ambiguous)
        let mut by_name: HashMap<&str, Vec<&str>> = HashMap::new();
        for element in elements.values() {
            by_name.entry(element.name.as_str()).or_default().push(element.id.as_str());
        }

        let resolve = |reference: &str, role: &str, trace: &TraceInfo| -> Result<String, String> {
            if elements.contains_key(reference) {
                return Ok(reference.to_string());
            }
            match by_name.get(reference).map(Vec::as_slice) {
                Some([single]) => Ok((*single).to_string()),
                Some(candidates) => Err(format!(
                    "trace '{}' {} '{}': ambiguous name, matches ids {:?} — use an id",
                    trace.trace_type, role, reference, candidates
                )),
                None => Err(format!(
                    "trace '{} {} {}': unknown element '{}' ({}) — declare it or fix the reference",
                    trace.from, trace.trace_type, trace.to, reference, role
                )),
            }
        };

        let mut errors = Vec::new();
        let mut resolved = Vec::new();
        for mut trace in traces {
            match (
                resolve(&trace.from, "from", &trace),
                resolve(&trace.to, "to", &trace),
            ) {
                (Ok(from), Ok(to)) => {
                    trace.from = from;
                    trace.to = to;
                    resolved.push(trace);
                }
                (from_result, to_result) => {
                    errors.extend(from_result.err());
                    errors.extend(to_result.err());
                }
            }
        }

        if errors.is_empty() {
            Ok(resolved)
        } else {
            Err(format!(
                "{} unresolved trace reference(s):\n  {}",
                errors.len(),
                errors.join("\n  ")
            ))
        }
    }
}

impl SemanticModel {
    pub fn get_requirement(&self, id: &str) -> Option<&RequirementInfo> {
        self.requirements.iter().find(|r| r.id == id)
    }
    
    pub fn get_component(&self, id: &str) -> Option<&ComponentInfo> {
        self.components.iter().find(|c| c.id == id)
    }
    
    pub fn get_traces_from(&self, element_id: &str) -> Vec<&TraceInfo> {
        self.traces.iter()
            .filter(|t| t.from == element_id)
            .collect()
    }

    /// Change-impact analysis: everything transitively connected to the given
    /// element through traces (both directions), exchanges/interfaces,
    /// component-function allocations, and containment. BFS with the relation
    /// that reached each element and its distance from the change.
    pub fn impact_of(&self, element: &str) -> Option<Vec<ImpactEntry>> {
        // Resolve by id first, then by unambiguous name.
        let start_id = if self.all_elements.contains_key(element) {
            element.to_string()
        } else {
            let matches: Vec<&ElementInfo> = self
                .all_elements
                .values()
                .filter(|e| e.name == element)
                .collect();
            match matches.as_slice() {
                [single] => single.id.clone(),
                _ => return None,
            }
        };

        // Adjacency: element id -> [(neighbor id, relation description)]
        let mut graph: HashMap<String, Vec<(String, String)>> = HashMap::new();
        let mut link = |graph: &mut HashMap<String, Vec<(String, String)>>,
                        a: &str,
                        b: &str,
                        forward: &str,
                        backward: &str| {
            graph.entry(a.to_string()).or_default().push((b.to_string(), forward.to_string()));
            graph.entry(b.to_string()).or_default().push((a.to_string(), backward.to_string()));
        };

        for trace in &self.traces {
            link(
                &mut graph,
                &trace.from,
                &trace.to,
                &format!("{} (trace)", trace.trace_type),
                &format!("is {}d by (trace)", trace.trace_type.trim_end_matches('s')),
            );
        }
        let resolve_endpoint = |endpoint: &str| -> Option<String> {
            if self.all_elements.contains_key(endpoint) {
                return Some(endpoint.to_string());
            }
            let root = endpoint.split('.').next().unwrap_or(endpoint);
            if self.all_elements.contains_key(root) {
                return Some(root.to_string());
            }
            self.all_elements
                .values()
                .find(|e| e.name == endpoint || e.name == root)
                .map(|e| e.id.clone())
        };
        for interface in &self.interfaces {
            if let (Some(from), Some(to)) =
                (resolve_endpoint(&interface.from), resolve_endpoint(&interface.to))
            {
                let label = format!("exchange '{}'", interface.name);
                link(&mut graph, &from, &to, &label, &label);
            }
        }
        for component in &self.components {
            for function_ref in &component.functions {
                if let Some(function_id) = resolve_endpoint(function_ref) {
                    link(
                        &mut graph,
                        &component.id,
                        &function_id,
                        "performs (allocation)",
                        "is performed by (allocation)",
                    );
                }
            }
        }

        // BFS
        let mut visited: HashMap<String, (u32, String, String)> = HashMap::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start_id.clone(), 0u32));
        visited.insert(start_id.clone(), (0, String::new(), String::new()));
        while let Some((current, depth)) = queue.pop_front() {
            if let Some(neighbors) = graph.get(&current) {
                for (neighbor, relation) in neighbors {
                    if !visited.contains_key(neighbor) {
                        visited.insert(
                            neighbor.clone(),
                            (depth + 1, relation.clone(), current.clone()),
                        );
                        queue.push_back((neighbor.clone(), depth + 1));
                    }
                }
            }
        }

        let mut entries: Vec<ImpactEntry> = visited
            .into_iter()
            .filter(|(id, _)| *id != start_id)
            .map(|(id, (depth, via, from))| {
                let element = self.all_elements.get(&id);
                ImpactEntry {
                    name: element.map(|e| e.name.clone()).unwrap_or_else(|| id.clone()),
                    element_type: element
                        .map(|e| e.element_type.clone())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    id,
                    via,
                    via_element: from,
                    depth,
                }
            })
            .collect();
        entries.sort_by(|a, b| a.depth.cmp(&b.depth).then(a.name.cmp(&b.name)));
        Some(entries)
    }
    
    pub fn get_traces_to(&self, element_id: &str) -> Vec<&TraceInfo> {
        self.traces.iter()
            .filter(|t| t.to == element_id)
            .collect()
    }
    
    pub fn validate_traceability(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Check for requirements without downstream traces
        for req in &self.requirements {
            if self.get_traces_from(&req.id).is_empty() {
                issues.push(format!("Requirement {} has no downstream traces", req.id));
            }
        }
        
        // Check for components without upstream traces
        for comp in &self.components {
            if self.get_traces_to(&comp.id).is_empty() {
                issues.push(format!("Component {} has no upstream traces", comp.id));
            }
        }
        
        issues
    }
    
    pub fn compute_metrics(&self) -> ModelMetrics {
        let total_elements = self.requirements.len() + self.components.len() + self.functions.len();
        
        // A requirement is covered when any trace touches it — usually as the
        // TARGET ("component satisfies requirement"), sometimes as the source.
        let traced_requirements = self.requirements.iter()
            .filter(|r| self.traces.iter().any(|t| t.to == r.id || t.from == r.id))
            .count();
        
        let traceability_coverage = if !self.requirements.is_empty() {
            (traced_requirements as f64 / self.requirements.len() as f64) * 100.0
        } else {
            0.0
        };
        
        ModelMetrics {
            total_elements,
            requirements_count: self.requirements.len(),
            components_count: self.components.len(),
            functions_count: self.functions.len(),
            traces_count: self.traces.len(),
            traceability_coverage,
        }
    }
}

/// One element reached by change-impact analysis.
#[derive(Debug, Clone, Serialize)]
pub struct ImpactEntry {
    pub id: String,
    pub name: String,
    pub element_type: String,
    /// The relation through which the impact propagates.
    pub via: String,
    /// The already-impacted element this one was reached from.
    pub via_element: String,
    /// Distance from the changed element (1 = directly related).
    pub depth: u32,
}

#[derive(Debug, Clone)]
pub struct ModelMetrics {
    pub total_elements: usize,
    pub requirements_count: usize,
    pub components_count: usize,
    pub functions_count: usize,
    pub traces_count: usize,
    pub traceability_coverage: f64,
}
