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
    #[serde(default)]
    pub missions: Vec<MissionInfo>,
    #[serde(default)]
    pub capabilities: Vec<CapabilityInfo>,
    #[serde(default)]
    pub functional_chains: Vec<FunctionalChainInfo>,
    pub all_elements: HashMap<String, ElementInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MissionInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CapabilityInfo {
    pub id: String,
    pub name: String,
    /// Resolved element ids involved in this capability.
    pub involves: Vec<String>,
    /// Resolved id of the realized higher-level capability, if any.
    pub realizes: Option<String>,
    /// Resolved id of the mission fulfilled, if any.
    pub mission: Option<String>,
    /// "System" (SA) or "Realization" (LA/PA).
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionalChainInfo {
    pub id: String,
    pub name: String,
    /// Resolved, ordered element ids (functions/exchanges) of the chain.
    pub involves: Vec<String>,
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
            missions: Vec::new(),
            capabilities: Vec::new(),
            functional_chains: Vec::new(),
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

    // Arcadia transport rule: a behavioural exchange between components
    // deployed on DIFFERENT nodes must be supported by a physical link or
    // physical path between those nodes (MetaModel p.16).
    let mut node_of_component: HashMap<&str, &str> = HashMap::new();
    let mut node_id_by_name: HashMap<&str, &str> = HashMap::new();
    for pa in &ast.physical_architecture {
        for node in &pa.nodes {
            node_id_by_name.insert(node.name.as_str(), node.id.as_str());
            node_id_by_name.insert(node.id.as_str(), node.id.as_str());
            for deployment in &node.deployments {
                node_of_component.insert(deployment.component.as_str(), node.id.as_str());
            }
            for behavior in &node.behavior_components {
                for allocated in &behavior.allocated_functions {
                    node_of_component.insert(allocated.as_str(), node.id.as_str());
                }
            }
        }
    }
    // Link endpoints may use node NAMES; normalize to node ids.
    let normalize = |endpoint: &str| -> String {
        node_id_by_name.get(endpoint).map(|id| id.to_string()).unwrap_or_else(|| endpoint.to_string())
    };
    let mut linked_pairs: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    for pa in &ast.physical_architecture {
        for link in &pa.links {
            let mut pair = [normalize(&link.from), normalize(&link.to)];
            pair.sort();
            linked_pairs.insert((pair[0].clone(), pair[1].clone()));
        }
        for exchange in &pa.physical_exchanges {
            let mut pair = [normalize(&exchange.from), normalize(&exchange.to)];
            pair.sort();
            linked_pairs.insert((pair[0].clone(), pair[1].clone()));
        }
    }
    if !node_of_component.is_empty() {
        for la in &ast.logical_architecture {
            for exchange in &la.component_exchanges {
                let from_root = exchange.from_port.split('.').next().unwrap_or(&exchange.from_port);
                let to_root = exchange.to_port.split('.').next().unwrap_or(&exchange.to_port);
                if let (Some(node_a), Some(node_b)) =
                    (node_of_component.get(from_root), node_of_component.get(to_root))
                {
                    if node_a != node_b {
                        let mut pair = [node_a.to_string(), node_b.to_string()];
                        pair.sort();
                        if !linked_pairs.contains(&(pair[0].clone(), pair[1].clone())) {
                            lints.push(format!(
                                "component exchange '{}' crosses nodes '{}' and '{}' but no physical link or exchange connects them — behavioural exchanges must be transported (Arcadia)",
                                exchange.label.as_deref().unwrap_or(""),
                                node_a, node_b
                            ));
                        }
                    }
                }
            }
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
            
            // Register operational capabilities (realization targets for SA)
            for capability in &oa.capabilities {
                register_element(
                    &mut all_elements,
                    &mut duplicate_ids,
                    capability.id.clone(),
                    ElementInfo::new(capability.id.clone(), capability.name.clone(), "OperationalCapability"),
                );
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

                for port in &func.ports {
                    let port_id = format!("{}.{}", func_id, port.name);
                    register_element(
                        all_elements,
                        duplicates,
                        port_id.clone(),
                        ElementInfo::new(port_id, port.name.clone(), "FunctionPort"),
                    );
                }

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

                for port in &comp.ports {
                    let port_id = format!("{}.{}", comp_id, port.name);
                    register_element(
                        all_elements,
                        duplicates,
                        port_id.clone(),
                        ElementInfo::new(port_id, port.name.clone(), "ComponentPort"),
                    );
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

                for port in &node.ports {
                    let port_id = format!("{}.{}", node_id, port.name);
                    register_element(
                        &mut all_elements,
                        &mut duplicate_ids,
                        port_id.clone(),
                        ElementInfo::new(port_id, port.name.clone(), "PhysicalPort"),
                    );
                }
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
        
        // Collect missions, capabilities, and functional chains (SA + LA),
        // registering their identities BEFORE resolving their references.
        let mut missions_info = Vec::new();
        let mut capabilities_info = Vec::new();
        let mut chains_info = Vec::new();

        for sa in &ast.system_analysis {
            for mission in &sa.missions {
                register_element(
                    &mut all_elements,
                    &mut duplicate_ids,
                    mission.id.clone(),
                    ElementInfo::new(mission.id.clone(), mission.name.clone(), "Mission"),
                );
                missions_info.push(MissionInfo { id: mission.id.clone(), name: mission.name.clone() });
            }
        }
        let capability_sources: Vec<(&Capability, &str)> = ast
            .system_analysis
            .iter()
            .flat_map(|sa| sa.capabilities.iter().map(|c| (c, "System")))
            .chain(
                ast.logical_architecture
                    .iter()
                    .flat_map(|la| la.capability_realizations.iter().map(|c| (c, "Realization"))),
            )
            .collect();
        for (capability, kind) in &capability_sources {
            register_element(
                &mut all_elements,
                &mut duplicate_ids,
                capability.id.clone(),
                ElementInfo::new(capability.id.clone(), capability.name.clone(), format!("{}Capability", kind)),
            );
        }
        let chain_sources: Vec<&FunctionalChain> = ast
            .system_analysis
            .iter()
            .flat_map(|sa| sa.functional_chains.iter())
            .chain(ast.logical_architecture.iter().flat_map(|la| la.functional_chains.iter()))
            .collect();
        for chain in &chain_sources {
            register_element(
                &mut all_elements,
                &mut duplicate_ids,
                chain.id.clone(),
                ElementInfo::new(chain.id.clone(), chain.name.clone(), "FunctionalChain"),
            );
        }

        // Resolve involves/realizes/mission references (dangling = error, like traces)
        let mut reference_errors = Vec::new();
        {
            let resolve = |reference: &str, context: String, errors: &mut Vec<String>| -> Option<String> {
                if all_elements.contains_key(reference) {
                    return Some(reference.to_string());
                }
                let matches: Vec<&ElementInfo> =
                    all_elements.values().filter(|e| e.name == reference).collect();
                match matches.as_slice() {
                    [single] => Some(single.id.clone()),
                    [] => {
                        errors.push(format!("{}: unknown element '{}'", context, reference));
                        None
                    }
                    _ => {
                        errors.push(format!("{}: ambiguous name '{}' — use an id", context, reference));
                        None
                    }
                }
            };

            for (capability, kind) in &capability_sources {
                let involves = capability
                    .involves
                    .iter()
                    .filter_map(|r| resolve(r, format!("capability '{}' involves", capability.name), &mut reference_errors))
                    .collect();
                let realizes = capability.realizes.as_ref().and_then(|r| {
                    resolve(r, format!("capability '{}' realizes", capability.name), &mut reference_errors)
                });
                let mission = capability.mission.as_ref().and_then(|r| {
                    resolve(r, format!("capability '{}' mission", capability.name), &mut reference_errors)
                });
                capabilities_info.push(CapabilityInfo {
                    id: capability.id.clone(),
                    name: capability.name.clone(),
                    involves,
                    realizes,
                    mission,
                    kind: kind.to_string(),
                });
            }
            for chain in &chain_sources {
                let involves = chain
                    .involves
                    .iter()
                    .filter_map(|r| resolve(r, format!("functional_chain '{}' involves", chain.name), &mut reference_errors))
                    .collect();
                chains_info.push(FunctionalChainInfo {
                    id: chain.id.clone(),
                    name: chain.name.clone(),
                    involves,
                });
            }
        }
        let mut deferred_warnings: Vec<String> = Vec::new();

        // Data model: classes, data types, exchange items — with identity.
        for class_def in &ast.classes {
            register_element(
                &mut all_elements,
                &mut duplicate_ids,
                class_def.id.clone(),
                ElementInfo::new(class_def.id.clone(), class_def.name.clone(), "Class"),
            );
        }
        for data_type in &ast.data_types {
            let kind = if data_type.enumeration_values.is_some() { "Enumeration" } else { "DataType" };
            register_element(
                &mut all_elements,
                &mut duplicate_ids,
                data_type.id.clone(),
                ElementInfo::new(data_type.id.clone(), data_type.name.clone(), kind),
            );
        }
        for item in &ast.exchange_items {
            register_element(
                &mut all_elements,
                &mut duplicate_ids,
                item.id.clone(),
                ElementInfo::new(item.id.clone(), item.name.clone(), "ExchangeItem"),
            );
        }
        for item in &ast.exchange_items {
            for element in &item.elements {
                if !all_elements.contains_key(element)
                    && !all_elements.values().any(|e| e.name == *element)
                {
                    reference_errors.push(format!(
                        "exchange_item '{}': element '{}' does not match any declared class or data type",
                        item.name, element
                    ));
                }
            }
        }

        // Physical links get identity by name; paths must reference them.
        for pa in &ast.physical_architecture {
            for link in &pa.links {
                if !link.name.is_empty() {
                    register_element(
                        &mut all_elements,
                        &mut duplicate_ids,
                        link.name.clone(),
                        ElementInfo::new(link.name.clone(), link.name.clone(), "PhysicalLink"),
                    );
                }
            }
            for path in &pa.paths {
                register_element(
                    &mut all_elements,
                    &mut duplicate_ids,
                    path.id.clone(),
                    ElementInfo::new(path.id.clone(), path.name.clone(), "PhysicalPath"),
                );
                for link_ref in &path.involves {
                    let is_link = all_elements
                        .get(link_ref)
                        .map(|e| e.element_type == "PhysicalLink")
                        .unwrap_or(false)
                        || all_elements
                            .values()
                            .any(|e| e.element_type == "PhysicalLink" && e.name == *link_ref);
                    if !is_link {
                        reference_errors.push(format!(
                            "physical_path '{}': '{}' does not match any declared physical link",
                            path.name, link_ref
                        ));
                    }
                }
            }
        }

        // When exchange items are declared, exchanges referencing unknown
        // items get a warning (the data-model feature is in use).
        if !ast.exchange_items.is_empty() {
            let item_names: std::collections::HashSet<&str> = ast
                .exchange_items
                .iter()
                .flat_map(|i| [i.id.as_str(), i.name.as_str()])
                .collect();
            for sa in &ast.system_analysis {
                for exchange in &sa.functional_exchanges {
                    if !item_names.contains(exchange.data_type.as_str()) {
                        deferred_warnings.push(format!(
                            "functional_exchange '{}': exchange item '{}' is not declared as an exchange_item",
                            exchange.label.as_deref().unwrap_or(""),
                            exchange.data_type
                        ));
                    }
                }
            }
        }

        // State machines and scenarios: register identities and validate
        // their internal references (declared states, participants).
        for machine in &ast.state_machines {
            register_element(
                &mut all_elements,
                &mut duplicate_ids,
                machine.name.clone(),
                ElementInfo::new(machine.name.clone(), machine.name.clone(), "StateMachine"),
            );
            let state_names: std::collections::HashSet<&str> =
                machine.states.iter().map(|s| s.name.as_str()).collect();
            if !machine.initial_state.is_empty() && !state_names.contains(machine.initial_state.as_str()) {
                reference_errors.push(format!(
                    "state_machine '{}': initial state '{}' is not declared",
                    machine.name, machine.initial_state
                ));
            }
            for transition in &machine.transitions {
                for (endpoint, role) in [(&transition.from, "from"), (&transition.to, "to")] {
                    if !state_names.contains(endpoint.as_str()) {
                        reference_errors.push(format!(
                            "state_machine '{}': transition {} '{}' is not a declared state or mode",
                            machine.name, role, endpoint
                        ));
                    }
                }
                if !transition.trigger.is_empty()
                    && !all_elements.contains_key(&transition.trigger)
                    && !all_elements.values().any(|e| e.name == transition.trigger)
                {
                    deferred_warnings.push(format!(
                        "state_machine '{}': trigger '{}' does not match any declared element (Arcadia: transitions are commanded by functional dataflow)",
                        machine.name, transition.trigger
                    ));
                }
            }
        }
        for scenario in &ast.scenarios {
            register_element(
                &mut all_elements,
                &mut duplicate_ids,
                scenario.name.clone(),
                ElementInfo::new(scenario.name.clone(), scenario.name.clone(), "Scenario"),
            );
            let mut participant_ids: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            for participant in &scenario.participants {
                let resolved = if all_elements.contains_key(&participant.id) {
                    Some(participant.id.clone())
                } else {
                    let matches: Vec<&ElementInfo> = all_elements
                        .values()
                        .filter(|e| e.name == participant.id)
                        .collect();
                    match matches.as_slice() {
                        [single] => Some(single.id.clone()),
                        _ => None,
                    }
                };
                match resolved {
                    Some(id) => {
                        participant_ids.insert(id);
                        participant_ids.insert(participant.id.clone());
                    }
                    None => reference_errors.push(format!(
                        "scenario '{}': participant '{}' does not match any declared element",
                        scenario.name, participant.id
                    )),
                }
            }
            for message in &scenario.messages {
                for (endpoint, role) in [(&message.from, "from"), (&message.to, "to")] {
                    if !participant_ids.contains(endpoint.as_str()) {
                        reference_errors.push(format!(
                            "scenario '{}': message {} '{}' is not one of the scenario participants",
                            scenario.name, role, endpoint
                        ));
                    }
                }
            }
        }

        if !reference_errors.is_empty() {
            return Err(format!(
                "{} unresolved reference(s):\n  {}",
                reference_errors.len(),
                reference_errors.join("\n  ")
            ));
        }

        // Resolve trace endpoints. Dangling references are compile errors:
        // a trace that points at nothing must never be silently dropped.
        let resolved_traces = Self::resolve_traces(traces, &all_elements)?;

        // Exchange endpoints are checked but only warned about for now:
        // port paths (Component.Port) are not first-class elements yet.
        let mut warnings = duplicate_ids;
        warnings.extend(deferred_warnings);
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
                missions: missions_info,
                capabilities: capabilities_info,
                functional_chains: chains_info,
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
        for chain in &self.functional_chains {
            for involved in &chain.involves {
                link(
                    &mut graph,
                    &chain.id,
                    involved,
                    "involves (functional chain)",
                    "is involved in (functional chain)",
                );
            }
        }
        for capability in &self.capabilities {
            for involved in &capability.involves {
                link(
                    &mut graph,
                    &capability.id,
                    involved,
                    "involves (capability)",
                    "is involved in (capability)",
                );
            }
            if let Some(realized) = &capability.realizes {
                link(&mut graph, &capability.id, realized, "realizes (capability)", "is realized by (capability)");
            }
            if let Some(mission) = &capability.mission {
                link(&mut graph, &capability.id, mission, "fulfills (mission)", "requires (capability)");
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
