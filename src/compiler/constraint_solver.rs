use super::constraint_engine::*;
use super::semantic_enhanced::*;
use super::capella_metamodel::*;
use std::collections::HashMap;

pub struct ConstraintSolver {
    pub engine: ConstraintEngine,
    max_iterations: usize,
    convergence_threshold: f64,
}

impl ConstraintSolver {
    pub fn new(metamodel: CapellaMetamodel) -> Self {
        Self {
            engine: ConstraintEngine::new(metamodel),
            max_iterations: 1000,
            convergence_threshold: 0.001,
        }
    }
    
    pub fn solve(
        &self,
        model: &EnhancedSemanticModel,
        initial_layout: HashMap<String, ElementBounds>,
    ) -> SolutionResult {
        let mut layout = initial_layout;
        let mut iteration = 0;
        let mut best_score = 0.0;
        let mut best_layout = layout.clone();
        
        let mut violations = self.engine.validate_hard_constraints(model, &layout);
        
        if !violations.is_empty() {
            layout = self.fix_hard_constraints(model, layout, &violations);
            violations = self.engine.validate_hard_constraints(model, &layout);
        }
        
        if !violations.is_empty() {
            return SolutionResult {
                layout,
                score: 0.0,
                iterations: iteration,
                hard_constraint_violations: violations,
                soft_constraint_score: 0.0,
                convergence_achieved: false,
            };
        }
        
        let edges = self.extract_edges(model);
        
        while iteration < self.max_iterations {
            let score = self.engine.evaluate_soft_constraints(model, &layout, &edges);
            
            if score > best_score {
                best_score = score;
                best_layout = layout.clone();
            }
            
            if iteration > 0 && (score - best_score).abs() < self.convergence_threshold {
                break;
            }
            
            layout = self.optimize_soft_constraints(model, layout, &edges);
            
            iteration += 1;
        }
        
        let final_score = self.engine.evaluate_soft_constraints(model, &best_layout, &edges);
        
        SolutionResult {
            layout: best_layout,
            score: final_score,
            iterations: iteration,
            hard_constraint_violations: Vec::new(),
            soft_constraint_score: final_score,
            convergence_achieved: iteration < self.max_iterations,
        }
    }
    
    fn fix_hard_constraints(
        &self,
        model: &EnhancedSemanticModel,
        mut layout: HashMap<String, ElementBounds>,
        violations: &[ConstraintViolation],
    ) -> HashMap<String, ElementBounds> {
        for violation in violations {
            match violation.constraint_id.as_str() {
                "HC-001" => {
                    layout = self.fix_actors_on_periphery(model, layout, violation);
                }
                "HC-002" => {
                    layout = self.fix_children_inside_parents(model, layout, violation);
                }
                "HC-003" => {
                    layout = self.fix_overlapping_elements(layout, violation);
                }
                "HC-006" => {
                    layout = self.fix_safety_critical_spacing(layout, violation);
                }
                "HC-007" => {
                    layout = self.fix_layer_separation(model, layout, violation);
                }
                "HC-009" => {
                    layout = self.fix_minimum_element_size(layout, violation);
                }
                "HC-010" => {
                    layout = self.fix_diagram_bounds(layout, violation);
                }
                _ => {}
            }
        }
        
        layout
    }
    
    fn fix_actors_on_periphery(
        &self,
        _model: &EnhancedSemanticModel,
        mut layout: HashMap<String, ElementBounds>,
        violation: &ConstraintViolation,
    ) -> HashMap<String, ElementBounds> {
        if let Some(actor_id) = violation.element_ids.first() {
            let (min_x, min_y, max_x, max_y) = self.get_diagram_bounds(&layout);
            
            if let Some(bounds) = layout.get(actor_id).cloned() {
                let (cx, cy) = bounds.center();
                
                let distances = vec![
                    (cx - min_x, "left"),
                    (max_x - cx, "right"),
                    (cy - min_y, "top"),
                    (max_y - cy, "bottom"),
                ];
                
                let (_, nearest_edge) = distances.iter()
                    .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
                    .unwrap();
                
                if let Some(actor_bounds) = layout.get_mut(actor_id) {
                    match *nearest_edge {
                        "left" => actor_bounds.x = min_x,
                        "right" => actor_bounds.x = max_x - actor_bounds.width,
                        "top" => actor_bounds.y = min_y,
                        "bottom" => actor_bounds.y = max_y - actor_bounds.height,
                        _ => {}
                    }
                }
            }
        }
        
        layout
    }
    
    fn fix_children_inside_parents(
        &self,
        _model: &EnhancedSemanticModel,
        mut layout: HashMap<String, ElementBounds>,
        violation: &ConstraintViolation,
    ) -> HashMap<String, ElementBounds> {
        if violation.element_ids.len() >= 2 {
            let child_id = &violation.element_ids[0];
            let parent_id = &violation.element_ids[1];
            
            if let (Some(child_bounds), Some(parent_bounds)) = 
                (layout.get(child_id).cloned(), layout.get(parent_id).cloned()) {
                
                let padding = 20.0;
                
                let new_parent_width = (child_bounds.width + padding * 2.0).max(parent_bounds.width);
                let new_parent_height = (child_bounds.height + padding * 2.0).max(parent_bounds.height);
                
                if let Some(parent) = layout.get_mut(parent_id) {
                    parent.width = new_parent_width;
                    parent.height = new_parent_height;
                }
                
                if let Some(child) = layout.get_mut(child_id) {
                    child.x = parent_bounds.x + padding;
                    child.y = parent_bounds.y + padding;
                }
            }
        }
        
        layout
    }
    
    fn fix_overlapping_elements(
        &self,
        mut layout: HashMap<String, ElementBounds>,
        violation: &ConstraintViolation,
    ) -> HashMap<String, ElementBounds> {
        if violation.element_ids.len() >= 2 {
            let elem1_id = &violation.element_ids[0];
            let elem2_id = &violation.element_ids[1];
            
            if let Some(bounds2) = layout.get(elem2_id).cloned() {
                if let Some(bounds1) = layout.get_mut(elem1_id) {
                    bounds1.x = bounds2.x + bounds2.width + 50.0;
                }
            }
        }
        
        layout
    }
    
    fn fix_safety_critical_spacing(
        &self,
        mut layout: HashMap<String, ElementBounds>,
        violation: &ConstraintViolation,
    ) -> HashMap<String, ElementBounds> {
        if violation.element_ids.len() >= 2 {
            let elem1_id = &violation.element_ids[0];
            let elem2_id = &violation.element_ids[1];
            
            let min_spacing = 80.0;
            
            if let (Some(bounds1), Some(bounds2)) = 
                (layout.get(elem1_id).cloned(), layout.get(elem2_id).cloned()) {
                
                let distance = bounds1.distance_to(&bounds2);
                let needed_move = min_spacing - distance;
                
                if needed_move > 0.0 {
                    if let Some(elem2) = layout.get_mut(elem2_id) {
                        elem2.x += needed_move;
                    }
                }
            }
        }
        
        layout
    }
    
    fn fix_layer_separation(
        &self,
        _model: &EnhancedSemanticModel,
        mut layout: HashMap<String, ElementBounds>,
        violation: &ConstraintViolation,
    ) -> HashMap<String, ElementBounds> {
        if violation.element_ids.len() >= 2 {
            let elem2_id = &violation.element_ids[1];
            
            if let Some(bounds2) = layout.get_mut(elem2_id) {
                bounds2.y += 150.0;
            }
        }
        
        layout
    }
    
    fn fix_minimum_element_size(
        &self,
        mut layout: HashMap<String, ElementBounds>,
        violation: &ConstraintViolation,
    ) -> HashMap<String, ElementBounds> {
        if let Some(elem_id) = violation.element_ids.first() {
            if let Some(bounds) = layout.get_mut(elem_id) {
                bounds.width = bounds.width.max(80.0);
                bounds.height = bounds.height.max(60.0);
            }
        }
        
        layout
    }
    
    fn fix_diagram_bounds(
        &self,
        mut layout: HashMap<String, ElementBounds>,
        violation: &ConstraintViolation,
    ) -> HashMap<String, ElementBounds> {
        if let Some(elem_id) = violation.element_ids.first() {
            if let Some(bounds) = layout.get_mut(elem_id) {
                bounds.x = bounds.x.max(0.0).min(10000.0 - bounds.width);
                bounds.y = bounds.y.max(0.0).min(10000.0 - bounds.height);
            }
        }
        
        layout
    }
    
    fn optimize_soft_constraints(
        &self,
        model: &EnhancedSemanticModel,
        layout: HashMap<String, ElementBounds>,
        edges: &[(String, String)],
    ) -> HashMap<String, ElementBounds> {
        let mut optimized = layout.clone();
        
        optimized = self.optimize_alignment(&optimized);
        optimized = self.optimize_spacing(&optimized);
        optimized = self.optimize_flow(model, &optimized, edges);
        
        optimized
    }
    
    fn optimize_alignment(&self, layout: &HashMap<String, ElementBounds>) -> HashMap<String, ElementBounds> {
        let mut optimized = layout.clone();
        let tolerance = 10.0;
        
        let elements: Vec<_> = layout.iter()
            .map(|(id, bounds)| (id.clone(), bounds.clone()))
            .collect();
        
        for i in 0..elements.len() {
            for j in i+1..elements.len() {
                let (_, bounds1) = &elements[i];
                let (id2, bounds2) = &elements[j];
                
                if (bounds1.y - bounds2.y).abs() < tolerance {
                    if let Some(b2) = optimized.get_mut(id2) {
                        b2.y = bounds1.y;
                    }
                }
                
                if (bounds1.x - bounds2.x).abs() < tolerance {
                    if let Some(b2) = optimized.get_mut(id2) {
                        b2.x = bounds1.x;
                    }
                }
            }
        }
        
        optimized
    }
    
    fn optimize_spacing(&self, layout: &HashMap<String, ElementBounds>) -> HashMap<String, ElementBounds> {
        layout.clone()
    }
    
    fn optimize_flow(
        &self,
        _model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
        _edges: &[(String, String)],
    ) -> HashMap<String, ElementBounds> {
        layout.clone()
    }
    
    fn extract_edges(&self, model: &EnhancedSemanticModel) -> Vec<(String, String)> {
        model.relationships.iter()
            .map(|r| (r.source_id.clone(), r.target_id.clone()))
            .collect()
    }
    
    fn get_diagram_bounds(&self, layout: &HashMap<String, ElementBounds>) -> (f64, f64, f64, f64) {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        
        for bounds in layout.values() {
            min_x = min_x.min(bounds.x);
            min_y = min_y.min(bounds.y);
            max_x = max_x.max(bounds.x + bounds.width);
            max_y = max_y.max(bounds.y + bounds.height);
        }
        
        (min_x, min_y, max_x, max_y)
    }
}

#[derive(Debug, Clone)]
pub struct SolutionResult {
    pub layout: HashMap<String, ElementBounds>,
    pub score: f64,
    pub iterations: usize,
    pub hard_constraint_violations: Vec<ConstraintViolation>,
    pub soft_constraint_score: f64,
    pub convergence_achieved: bool,
}

impl SolutionResult {
    pub fn is_valid(&self) -> bool {
        self.hard_constraint_violations.is_empty()
    }
    
    pub fn quality_rating(&self) -> QualityRating {
        if !self.is_valid() {
            return QualityRating::Invalid;
        }
        
        match self.soft_constraint_score {
            s if s >= 0.9 => QualityRating::Excellent,
            s if s >= 0.8 => QualityRating::Good,
            s if s >= 0.7 => QualityRating::Fair,
            s if s >= 0.6 => QualityRating::Poor,
            _ => QualityRating::VeryPoor,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QualityRating {
    Excellent,
    Good,
    Fair,
    Poor,
    VeryPoor,
    Invalid,
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new(CapellaMetamodel::new())
    }
}

#[derive(Debug, Clone)]
pub struct ConflictResolution {
    pub conflicting_constraints: Vec<String>,
    pub resolution_strategy: ResolutionStrategy,
    pub explanation: String,
}

#[derive(Debug, Clone)]
pub enum ResolutionStrategy {
    PrioritizeHard,
    PrioritizeSafety,
    PrioritizeSemantics,
    Compromise,
    UserDecision,
}

pub struct ConflictResolver {
    priority_order: Vec<ConstraintCategory>,
}

impl ConflictResolver {
    pub fn new() -> Self {
        Self {
            priority_order: vec![
                ConstraintCategory::Safety,
                ConstraintCategory::Semantics,
                ConstraintCategory::Containment,
                ConstraintCategory::Positioning,
                ConstraintCategory::Spacing,
                ConstraintCategory::Routing,
                ConstraintCategory::Alignment,
                ConstraintCategory::Aesthetics,
            ],
        }
    }
    
    pub fn resolve_conflict(
        &self,
        constraint1: &Constraint,
        constraint2: &Constraint,
    ) -> ConflictResolution {
        if constraint1.constraint_type == ConstraintType::Hard &&
           constraint2.constraint_type == ConstraintType::Soft {
            return ConflictResolution {
                conflicting_constraints: vec![constraint1.id.clone(), constraint2.id.clone()],
                resolution_strategy: ResolutionStrategy::PrioritizeHard,
                explanation: format!(
                    "Hard constraint '{}' takes precedence over soft constraint '{}'",
                    constraint1.name, constraint2.name
                ),
            };
        }
        
        if constraint1.constraint_type == ConstraintType::Soft &&
           constraint2.constraint_type == ConstraintType::Hard {
            return ConflictResolution {
                conflicting_constraints: vec![constraint1.id.clone(), constraint2.id.clone()],
                resolution_strategy: ResolutionStrategy::PrioritizeHard,
                explanation: format!(
                    "Hard constraint '{}' takes precedence over soft constraint '{}'",
                    constraint2.name, constraint1.name
                ),
            };
        }
        
        if constraint1.category == ConstraintCategory::Safety ||
           constraint2.category == ConstraintCategory::Safety {
            return ConflictResolution {
                conflicting_constraints: vec![constraint1.id.clone(), constraint2.id.clone()],
                resolution_strategy: ResolutionStrategy::PrioritizeSafety,
                explanation: "Safety constraints always take precedence".to_string(),
            };
        }
        
        let cat1_priority = self.priority_order.iter().position(|c| c == &constraint1.category).unwrap_or(999);
        let cat2_priority = self.priority_order.iter().position(|c| c == &constraint2.category).unwrap_or(999);
        
        if cat1_priority < cat2_priority {
            ConflictResolution {
                conflicting_constraints: vec![constraint1.id.clone(), constraint2.id.clone()],
                resolution_strategy: ResolutionStrategy::PrioritizeSemantics,
                explanation: format!(
                    "Constraint '{}' ({:?}) has higher priority than '{}' ({:?})",
                    constraint1.name, constraint1.category,
                    constraint2.name, constraint2.category
                ),
            }
        } else if cat2_priority < cat1_priority {
            ConflictResolution {
                conflicting_constraints: vec![constraint1.id.clone(), constraint2.id.clone()],
                resolution_strategy: ResolutionStrategy::PrioritizeSemantics,
                explanation: format!(
                    "Constraint '{}' ({:?}) has higher priority than '{}' ({:?})",
                    constraint2.name, constraint2.category,
                    constraint1.name, constraint1.category
                ),
            }
        } else {
            ConflictResolution {
                conflicting_constraints: vec![constraint1.id.clone(), constraint2.id.clone()],
                resolution_strategy: ResolutionStrategy::Compromise,
                explanation: format!(
                    "Constraints '{}' and '{}' have equal priority - attempting compromise",
                    constraint1.name, constraint2.name
                ),
            }
        }
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}
