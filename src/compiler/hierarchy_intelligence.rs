use super::constraint_engine::*;
use super::semantic_enhanced::*;
use super::capella_metamodel::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct HierarchyIntelligence {
    metamodel: CapellaMetamodel,
    config: HierarchyConfig,
}

#[derive(Debug, Clone)]
pub struct HierarchyConfig {
    pub min_container_padding: f64,
    pub level_spacing: f64,
    pub sibling_spacing: f64,
    pub enable_auto_nesting: bool,
    pub max_depth: usize,
    pub enable_subgraph_breaking: bool,
    pub subgraph_max_elements: usize,
}

impl HierarchyConfig {
    pub fn default() -> Self {
        Self {
            min_container_padding: 40.0,
            level_spacing: 100.0,
            sibling_spacing: 60.0,
            enable_auto_nesting: true,
            max_depth: 10,
            enable_subgraph_breaking: true,
            subgraph_max_elements: 50,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructuralPattern {
    pub pattern_type: StructuralPatternType,
    pub root_element_id: String,
    pub participating_elements: Vec<String>,
    pub depth: usize,
    pub complexity_score: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuralPatternType {
    ContainmentHierarchy,
    FunctionalChain,
    InterfaceCluster,
    AllocationPattern,
    LayeredArchitecture,
    ComponentDecomposition,
}

#[derive(Debug, Clone)]
pub struct HierarchyNode {
    pub element_id: String,
    pub parent_id: Option<String>,
    pub children: Vec<String>,
    pub level: usize,
    pub bounds: ElementBounds,
    pub allocated_bounds: ElementBounds,
    pub is_container: bool,
    pub depth_in_tree: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionalChain {
    pub id: String,
    pub name: String,
    pub sequence: Vec<String>,
    pub involved_elements: HashSet<String>,
    pub chain_type: ChainType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainType {
    Sequential,
    Parallel,
    Conditional,
    Loop,
    Mixed,
}

#[derive(Debug, Clone)]
pub struct InterfaceCluster {
    pub id: String,
    pub center_element_id: String,
    pub connected_elements: Vec<String>,
    pub interface_count: usize,
    pub cluster_density: f64,
}

#[derive(Debug, Clone)]
pub struct AllocationPattern {
    pub function_id: String,
    pub component_id: String,
    pub allocation_level: ArchitecturalLayer,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct Subgraph {
    pub id: String,
    pub elements: Vec<String>,
    pub internal_edges: Vec<(String, String)>,
    pub boundary_edges: Vec<(String, String)>,
    pub subgraph_type: SubgraphType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubgraphType {
    Layer,
    Component,
    Feature,
    Capability,
}

impl HierarchyIntelligence {
    pub fn new(metamodel: CapellaMetamodel) -> Self {
        Self {
            metamodel,
            config: HierarchyConfig::default(),
        }
    }
    
    pub fn with_config(mut self, config: HierarchyConfig) -> Self {
        self.config = config;
        self
    }
    
    pub fn detect_structural_patterns(
        &self,
        model: &EnhancedSemanticModel,
    ) -> Vec<StructuralPattern> {
        let mut patterns = Vec::new();
        
        patterns.extend(self.detect_containment_hierarchies(model));
        patterns.extend(self.detect_functional_chains(model));
        patterns.extend(self.detect_interface_clusters(model));
        patterns.extend(self.detect_allocation_patterns(model));
        
        patterns
    }
    
    fn detect_containment_hierarchies(
        &self,
        model: &EnhancedSemanticModel,
    ) -> Vec<StructuralPattern> {
        let mut patterns = Vec::new();
        
        let roots: Vec<_> = model.elements.iter()
            .filter(|e| e.parent_id.is_none())
            .collect();
        
        for root in roots {
            let mut participating = Vec::new();
            let depth = self.collect_hierarchy(&root.id, model, &mut participating, 0);
            
            if participating.len() > 1 {
                patterns.push(StructuralPattern {
                    pattern_type: StructuralPatternType::ContainmentHierarchy,
                    root_element_id: root.id.clone(),
                    participating_elements: participating.clone(),
                    depth,
                    complexity_score: self.compute_hierarchy_complexity(&participating, model),
                });
            }
        }
        
        patterns
    }
    
    fn collect_hierarchy(
        &self,
        element_id: &str,
        model: &EnhancedSemanticModel,
        participating: &mut Vec<String>,
        current_depth: usize,
    ) -> usize {
        participating.push(element_id.to_string());
        
        if let Some(element) = model.elements.iter().find(|e| e.id == element_id) {
            let mut max_child_depth = current_depth;
            
            for child_id in &element.children {
                let child_depth = self.collect_hierarchy(
                    child_id,
                    model,
                    participating,
                    current_depth + 1,
                );
                max_child_depth = max_child_depth.max(child_depth);
            }
            
            max_child_depth
        } else {
            current_depth
        }
    }
    
    fn compute_hierarchy_complexity(&self, elements: &[String], model: &EnhancedSemanticModel) -> f64 {
        let element_count = elements.len() as f64;
        
        let relationship_count = model.relationships.iter()
            .filter(|r| elements.contains(&r.source_id) && elements.contains(&r.target_id))
            .count() as f64;
        
        let avg_children = elements.iter()
            .filter_map(|id| model.elements.iter().find(|e| &e.id == id))
            .map(|e| e.children.len())
            .sum::<usize>() as f64 / element_count.max(1.0);
        
        (element_count * 0.3 + relationship_count * 0.5 + avg_children * 0.2).min(100.0)
    }
    
    fn detect_functional_chains(
        &self,
        model: &EnhancedSemanticModel,
    ) -> Vec<StructuralPattern> {
        let mut patterns = Vec::new();
        
        let functions: Vec<_> = model.elements.iter()
            .filter(|e| {
                e.element_type == CapellaElementType::Function ||
                e.element_type == CapellaElementType::SystemFunction ||
                e.element_type == CapellaElementType::LogicalFunction
            })
            .collect();
        
        let mut visited = HashSet::new();
        
        for func in &functions {
            if visited.contains(&func.id) {
                continue;
            }
            
            let mut chain = Vec::new();
            self.trace_functional_chain(&func.id, model, &mut chain, &mut visited);
            
            if chain.len() >= 2 {
                patterns.push(StructuralPattern {
                    pattern_type: StructuralPatternType::FunctionalChain,
                    root_element_id: chain[0].clone(),
                    participating_elements: chain.clone(),
                    depth: 1,
                    complexity_score: (chain.len() as f64 * 5.0).min(100.0),
                });
            }
        }
        
        patterns
    }
    
    fn trace_functional_chain(
        &self,
        func_id: &str,
        model: &EnhancedSemanticModel,
        chain: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) {
        if visited.contains(func_id) {
            return;
        }
        
        visited.insert(func_id.to_string());
        chain.push(func_id.to_string());
        
        let outgoing: Vec<_> = model.relationships.iter()
            .filter(|r| r.source_id == func_id && 
                       r.relationship_type == RelationshipType::FunctionalExchange)
            .collect();
        
        for rel in outgoing {
            if let Some(target) = model.elements.iter().find(|e| e.id == rel.target_id) {
                if target.element_type == CapellaElementType::Function ||
                   target.element_type == CapellaElementType::SystemFunction ||
                   target.element_type == CapellaElementType::LogicalFunction {
                    self.trace_functional_chain(&target.id, model, chain, visited);
                }
            }
        }
    }
    
    fn detect_interface_clusters(
        &self,
        model: &EnhancedSemanticModel,
    ) -> Vec<StructuralPattern> {
        let mut patterns = Vec::new();
        
        for element in &model.elements {
            let connected: Vec<_> = model.relationships.iter()
                .filter(|r| r.source_id == element.id || r.target_id == element.id)
                .map(|r| if r.source_id == element.id { &r.target_id } else { &r.source_id })
                .collect();
            
            if connected.len() >= 3 {
                let mut participating = vec![element.id.clone()];
                participating.extend(connected.iter().map(|s| s.to_string()));
                
                patterns.push(StructuralPattern {
                    pattern_type: StructuralPatternType::InterfaceCluster,
                    root_element_id: element.id.clone(),
                    participating_elements: participating,
                    depth: 1,
                    complexity_score: (connected.len() as f64 * 8.0).min(100.0),
                });
            }
        }
        
        patterns
    }
    
    fn detect_allocation_patterns(
        &self,
        model: &EnhancedSemanticModel,
    ) -> Vec<StructuralPattern> {
        let mut patterns = Vec::new();
        
        for element in &model.elements {
            if !element.allocated_to.is_empty() {
                let mut participating = vec![element.id.clone()];
                participating.extend(element.allocated_to.iter().cloned());
                
                patterns.push(StructuralPattern {
                    pattern_type: StructuralPatternType::AllocationPattern,
                    root_element_id: element.id.clone(),
                    participating_elements: participating,
                    depth: 1,
                    complexity_score: (element.allocated_to.len() as f64 * 10.0).min(100.0),
                });
            }
        }
        
        patterns
    }
    
    pub fn apply_hierarchical_layout(
        &self,
        model: &EnhancedSemanticModel,
        patterns: &[StructuralPattern],
    ) -> HashMap<String, ElementBounds> {
        let mut layout = HashMap::new();
        
        let hierarchy_tree = self.build_hierarchy_tree(model);
        
        let roots: Vec<_> = hierarchy_tree.values()
            .filter(|node| node.parent_id.is_none())
            .collect();
        
        let mut y_offset = 50.0;
        
        for root in roots {
            let root_bounds = self.layout_hierarchy_subtree(
                &root.element_id,
                &hierarchy_tree,
                model,
                50.0,
                y_offset,
                &mut layout,
            );
            
            y_offset = root_bounds.y + root_bounds.height + self.config.level_spacing;
        }
        
        self.adjust_container_sizes(&mut layout, &hierarchy_tree);
        
        layout
    }
    
    fn build_hierarchy_tree(&self, model: &EnhancedSemanticModel) -> HashMap<String, HierarchyNode> {
        let mut tree = HashMap::new();
        
        for element in &model.elements {
            let is_container = !element.children.is_empty();
            
            let depth = self.compute_depth(&element.id, model, 0);
            
            let metadata = self.metamodel.get_metadata(&element.element_type);
            
            let base_size = metadata.map(|m| match m.shape {
                DiagramShape::Rectangle => (150.0, 100.0),
                DiagramShape::RoundedRectangle => (160.0, 100.0),
                DiagramShape::Diamond => (140.0, 140.0),
                DiagramShape::Hexagon => (150.0, 110.0),
                DiagramShape::Ellipse => (160.0, 100.0),
                _ => (150.0, 100.0),
            }).unwrap_or((150.0, 100.0));
            
            tree.insert(element.id.clone(), HierarchyNode {
                element_id: element.id.clone(),
                parent_id: element.parent_id.clone(),
                children: element.children.clone(),
                level: match element.layer {
                    ArchitecturalLayer::Operational => 0,
                    ArchitecturalLayer::System => 1,
                    ArchitecturalLayer::Logical => 2,
                    ArchitecturalLayer::Physical => 3,
                    ArchitecturalLayer::EPBS => 4,
                    ArchitecturalLayer::CrossLayer => 5,
                },
                bounds: ElementBounds {
                    id: element.id.clone(),
                    x: 0.0,
                    y: 0.0,
                    width: base_size.0,
                    height: base_size.1,
                },
                allocated_bounds: ElementBounds {
                    id: format!("{}_allocated", element.id),
                    x: 0.0,
                    y: 0.0,
                    width: base_size.0,
                    height: base_size.1,
                },
                is_container,
                depth_in_tree: depth,
            });
        }
        
        tree
    }
    
    fn compute_depth(&self, element_id: &str, model: &EnhancedSemanticModel, current: usize) -> usize {
        if current > self.config.max_depth {
            return current;
        }
        
        if let Some(element) = model.elements.iter().find(|e| e.id == element_id) {
            if element.children.is_empty() {
                return current;
            }
            
            element.children.iter()
                .map(|child_id| self.compute_depth(child_id, model, current + 1))
                .max()
                .unwrap_or(current)
        } else {
            current
        }
    }
    
    fn layout_hierarchy_subtree(
        &self,
        element_id: &str,
        tree: &HashMap<String, HierarchyNode>,
        model: &EnhancedSemanticModel,
        x: f64,
        y: f64,
        layout: &mut HashMap<String, ElementBounds>,
    ) -> ElementBounds {
        let node = match tree.get(element_id) {
            Some(n) => n,
            None => return ElementBounds { id: "default".to_string(), x, y, width: 100.0, height: 80.0 },
        };
        
        if node.children.is_empty() {
            let bounds = ElementBounds {
                id: element_id.to_string(),
                x,
                y,
                width: node.bounds.width,
                height: node.bounds.height,
            };
            layout.insert(element_id.to_string(), bounds.clone());
            return bounds;
        }
        
        let padding = self.config.min_container_padding;
        let mut child_x = x + padding;
        let mut child_y = y + padding + 30.0;
        let mut max_child_width = 0.0_f64;
        let mut total_height = 0.0;
        
        for child_id in &node.children {
            let child_bounds = self.layout_hierarchy_subtree(
                child_id,
                tree,
                model,
                child_x,
                child_y,
                layout,
            );
            
            child_y += child_bounds.height + self.config.sibling_spacing;
            total_height += child_bounds.height + self.config.sibling_spacing;
            max_child_width = max_child_width.max(child_bounds.width);
        }
        
        let container_width = max_child_width + 2.0 * padding;
        let container_height = total_height + padding + 30.0;
        
        let bounds = ElementBounds {
            id: element_id.to_string(),
            x,
            y,
            width: container_width.max(node.bounds.width),
            height: container_height.max(node.bounds.height),
        };
        
        layout.insert(element_id.to_string(), bounds.clone());
        bounds
    }
    
    fn adjust_container_sizes(
        &self,
        layout: &mut HashMap<String, ElementBounds>,
        tree: &HashMap<String, HierarchyNode>,
    ) {
        let containers: Vec<_> = tree.values()
            .filter(|n| n.is_container)
            .map(|n| n.element_id.clone())
            .collect();
        
        for container_id in containers {
            if let Some(node) = tree.get(&container_id) {
                let mut min_x = f64::MAX;
                let mut min_y = f64::MAX;
                let mut max_x = f64::MIN;
                let mut max_y = f64::MIN;
                
                for child_id in &node.children {
                    if let Some(child_bounds) = layout.get(child_id) {
                        min_x = min_x.min(child_bounds.x);
                        min_y = min_y.min(child_bounds.y);
                        max_x = max_x.max(child_bounds.x + child_bounds.width);
                        max_y = max_y.max(child_bounds.y + child_bounds.height);
                    }
                }
                
                if min_x < f64::MAX {
                    let padding = self.config.min_container_padding;
                    
                    if let Some(container_bounds) = layout.get_mut(&container_id) {
                        container_bounds.x = (min_x - padding).min(container_bounds.x);
                        container_bounds.y = (min_y - padding - 30.0).min(container_bounds.y);
                        container_bounds.width = (max_x - min_x + 2.0 * padding).max(container_bounds.width);
                        container_bounds.height = (max_y - min_y + 2.0 * padding + 30.0).max(container_bounds.height);
                    }
                }
            }
        }
    }
    
    pub fn break_into_subgraphs(
        &self,
        model: &EnhancedSemanticModel,
    ) -> Vec<Subgraph> {
        if !self.config.enable_subgraph_breaking || 
           model.elements.len() <= self.config.subgraph_max_elements {
            return vec![self.create_full_graph_subgraph(model)];
        }
        
        let mut subgraphs = Vec::new();
        
        subgraphs.extend(self.break_by_layers(model));
        
        if subgraphs.is_empty() {
            subgraphs.extend(self.break_by_components(model));
        }
        
        if subgraphs.is_empty() {
            subgraphs.push(self.create_full_graph_subgraph(model));
        }
        
        subgraphs
    }
    
    fn create_full_graph_subgraph(&self, model: &EnhancedSemanticModel) -> Subgraph {
        let elements: Vec<_> = model.elements.iter().map(|e| e.id.clone()).collect();
        
        let internal_edges: Vec<_> = model.relationships.iter()
            .map(|r| (r.source_id.clone(), r.target_id.clone()))
            .collect();
        
        Subgraph {
            id: "full_graph".to_string(),
            elements,
            internal_edges,
            boundary_edges: Vec::new(),
            subgraph_type: SubgraphType::Layer,
        }
    }
    
    fn break_by_layers(&self, model: &EnhancedSemanticModel) -> Vec<Subgraph> {
        let mut subgraphs = Vec::new();
        
        let layers = vec![
            (ArchitecturalLayer::Operational, "operational"),
            (ArchitecturalLayer::System, "system"),
            (ArchitecturalLayer::Logical, "logical"),
            (ArchitecturalLayer::Physical, "physical"),
            (ArchitecturalLayer::EPBS, "epbs"),
        ];
        
        for (layer, layer_name) in layers {
            let layer_elements: Vec<_> = model.elements.iter()
                .filter(|e| e.layer == layer)
                .map(|e| e.id.clone())
                .collect();
            
            if layer_elements.is_empty() {
                continue;
            }
            
            let element_set: HashSet<_> = layer_elements.iter().cloned().collect();
            
            let internal_edges: Vec<_> = model.relationships.iter()
                .filter(|r| element_set.contains(&r.source_id) && element_set.contains(&r.target_id))
                .map(|r| (r.source_id.clone(), r.target_id.clone()))
                .collect();
            
            let boundary_edges: Vec<_> = model.relationships.iter()
                .filter(|r| {
                    (element_set.contains(&r.source_id) && !element_set.contains(&r.target_id)) ||
                    (!element_set.contains(&r.source_id) && element_set.contains(&r.target_id))
                })
                .map(|r| (r.source_id.clone(), r.target_id.clone()))
                .collect();
            
            subgraphs.push(Subgraph {
                id: format!("layer_{}", layer_name),
                elements: layer_elements,
                internal_edges,
                boundary_edges,
                subgraph_type: SubgraphType::Layer,
            });
        }
        
        subgraphs
    }
    
    fn break_by_components(&self, model: &EnhancedSemanticModel) -> Vec<Subgraph> {
        let mut subgraphs = Vec::new();
        
        let components: Vec<_> = model.elements.iter()
            .filter(|e| {
                e.element_type == CapellaElementType::Component ||
                e.element_type == CapellaElementType::SystemComponent ||
                e.element_type == CapellaElementType::LogicalComponent ||
                e.element_type == CapellaElementType::PhysicalComponent
            })
            .filter(|e| e.parent_id.is_none())
            .collect();
        
        for comp in components {
            let mut component_elements = Vec::new();
            self.collect_component_subtree(&comp.id, model, &mut component_elements);
            
            if component_elements.is_empty() {
                continue;
            }
            
            let element_set: HashSet<_> = component_elements.iter().cloned().collect();
            
            let internal_edges: Vec<_> = model.relationships.iter()
                .filter(|r| element_set.contains(&r.source_id) && element_set.contains(&r.target_id))
                .map(|r| (r.source_id.clone(), r.target_id.clone()))
                .collect();
            
            let boundary_edges: Vec<_> = model.relationships.iter()
                .filter(|r| {
                    (element_set.contains(&r.source_id) && !element_set.contains(&r.target_id)) ||
                    (!element_set.contains(&r.source_id) && element_set.contains(&r.target_id))
                })
                .map(|r| (r.source_id.clone(), r.target_id.clone()))
                .collect();
            
            subgraphs.push(Subgraph {
                id: format!("component_{}", comp.id),
                elements: component_elements,
                internal_edges,
                boundary_edges,
                subgraph_type: SubgraphType::Component,
            });
        }
        
        subgraphs
    }
    
    fn collect_component_subtree(
        &self,
        element_id: &str,
        model: &EnhancedSemanticModel,
        result: &mut Vec<String>,
    ) {
        result.push(element_id.to_string());
        
        if let Some(element) = model.elements.iter().find(|e| e.id == element_id) {
            for child_id in &element.children {
                self.collect_component_subtree(child_id, model, result);
            }
        }
    }
    
    pub fn create_level_of_detail_views(
        &self,
        model: &EnhancedSemanticModel,
    ) -> HashMap<usize, Vec<String>> {
        let mut lod_views = HashMap::new();
        
        lod_views.insert(0, self.get_level_0_elements(model));
        lod_views.insert(1, self.get_level_1_elements(model));
        lod_views.insert(2, self.get_level_2_elements(model));
        lod_views.insert(3, self.get_all_elements(model));
        
        lod_views
    }
    
    fn get_level_0_elements(&self, model: &EnhancedSemanticModel) -> Vec<String> {
        model.elements.iter()
            .filter(|e| {
                e.parent_id.is_none() &&
                (e.element_type == CapellaElementType::SystemComponent ||
                 e.element_type == CapellaElementType::Actor)
            })
            .map(|e| e.id.clone())
            .collect()
    }
    
    fn get_level_1_elements(&self, model: &EnhancedSemanticModel) -> Vec<String> {
        let level_0 = self.get_level_0_elements(model);
        let level_0_set: HashSet<_> = level_0.iter().cloned().collect();
        
        let mut level_1 = level_0.clone();
        
        for element in &model.elements {
            if element.parent_id.as_ref().map(|p| level_0_set.contains(p)).unwrap_or(false) {
                level_1.push(element.id.clone());
            }
        }
        
        level_1
    }
    
    fn get_level_2_elements(&self, model: &EnhancedSemanticModel) -> Vec<String> {
        let level_1 = self.get_level_1_elements(model);
        let level_1_set: HashSet<_> = level_1.iter().cloned().collect();
        
        let mut level_2 = level_1.clone();
        
        for element in &model.elements {
            if element.parent_id.as_ref().map(|p| level_1_set.contains(p)).unwrap_or(false) {
                level_2.push(element.id.clone());
            }
        }
        
        level_2
    }
    
    fn get_all_elements(&self, model: &EnhancedSemanticModel) -> Vec<String> {
        model.elements.iter().map(|e| e.id.clone()).collect()
    }
    
    pub fn filter_by_element_types(
        &self,
        model: &EnhancedSemanticModel,
        allowed_types: &[CapellaElementType],
    ) -> Vec<String> {
        let allowed_set: HashSet<_> = allowed_types.iter().cloned().collect();
        
        model.elements.iter()
            .filter(|e| allowed_set.contains(&e.element_type))
            .map(|e| e.id.clone())
            .collect()
    }
}

impl Default for HierarchyIntelligence {
    fn default() -> Self {
        Self::new(CapellaMetamodel::new())
    }
}
