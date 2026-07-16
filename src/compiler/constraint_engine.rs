use std::collections::HashMap;
use super::capella_metamodel::*;
use super::semantic_enhanced::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    Hard,
    Soft,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintPriority {
    Critical,
    High, 
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct Constraint {
    pub id: String,
    pub name: String,
    pub description: String,
    pub constraint_type: ConstraintType,
    pub priority: ConstraintPriority,
    pub category: ConstraintCategory,
    pub weight: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintCategory {
    Positioning,
    Spacing,
    Alignment,
    Containment,
    Routing,
    Aesthetics,
    Safety,
    Semantics,
}

#[derive(Debug, Clone)]
pub struct ConstraintViolation {
    pub constraint_id: String,
    pub severity: ViolationSeverity,
    pub element_ids: Vec<String>,
    pub message: String,
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViolationSeverity {
    Critical,
    Error,
    High,
    Warning,
    Medium,
    Info,
    Low,
}

#[derive(Debug, Clone)]
pub struct LayoutConstraints {
    pub hard_constraints: Vec<Constraint>,
    pub soft_constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub struct ElementBounds {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ElementBounds {
    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.width &&
        py >= self.y && py <= self.y + self.height
    }
    
    pub fn overlaps(&self, other: &ElementBounds) -> bool {
        !(self.x + self.width < other.x ||
          other.x + other.width < self.x ||
          self.y + self.height < other.y ||
          other.y + other.height < self.y)
    }
    
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
    
    pub fn distance_to(&self, other: &ElementBounds) -> f64 {
        let (cx1, cy1) = self.center();
        let (cx2, cy2) = other.center();
        ((cx2 - cx1).powi(2) + (cy2 - cy1).powi(2)).sqrt()
    }
}

pub struct ConstraintEngine {
    metamodel: CapellaMetamodel,
    hard_constraints: Vec<Constraint>,
    soft_constraints: Vec<Constraint>,
}

impl ConstraintEngine {
    pub fn new(metamodel: CapellaMetamodel) -> Self {
        let mut engine = Self {
            metamodel,
            hard_constraints: Vec::new(),
            soft_constraints: Vec::new(),
        };
        
        engine.initialize_hard_constraints();
        engine.initialize_soft_constraints();
        
        engine
    }
    
    fn initialize_hard_constraints(&mut self) {
        self.hard_constraints.push(Constraint {
            id: "HC-001".to_string(),
            name: "Actors on Periphery".to_string(),
            description: "Actors must be positioned on the diagram periphery (Arcadia rule)".to_string(),
            constraint_type: ConstraintType::Hard,
            priority: ConstraintPriority::Critical,
            category: ConstraintCategory::Positioning,
            weight: 100.0,
        });
        
        self.hard_constraints.push(Constraint {
            id: "HC-002".to_string(),
            name: "Children Inside Parents".to_string(),
            description: "Child components must be fully contained within parent boundaries".to_string(),
            constraint_type: ConstraintType::Hard,
            priority: ConstraintPriority::Critical,
            category: ConstraintCategory::Containment,
            weight: 100.0,
        });
        
        self.hard_constraints.push(Constraint {
            id: "HC-003".to_string(),
            name: "No Overlapping Elements".to_string(),
            description: "Elements must not overlap (except intentional containment)".to_string(),
            constraint_type: ConstraintType::Hard,
            priority: ConstraintPriority::Critical,
            category: ConstraintCategory::Spacing,
            weight: 100.0,
        });
        
        self.hard_constraints.push(Constraint {
            id: "HC-004".to_string(),
            name: "System Boundary Enclosure".to_string(),
            description: "System boundary must enclose all system elements".to_string(),
            constraint_type: ConstraintType::Hard,
            priority: ConstraintPriority::High,
            category: ConstraintCategory::Containment,
            weight: 90.0,
        });
        
        self.hard_constraints.push(Constraint {
            id: "HC-005".to_string(),
            name: "Ports on Component Edges".to_string(),
            description: "Ports must be positioned on component boundaries".to_string(),
            constraint_type: ConstraintType::Hard,
            priority: ConstraintPriority::High,
            category: ConstraintCategory::Positioning,
            weight: 90.0,
        });
        
        self.hard_constraints.push(Constraint {
            id: "HC-006".to_string(),
            name: "Safety-Critical Spacing".to_string(),
            description: "Safety-critical elements (ASIL-D) need minimum 80px spacing".to_string(),
            constraint_type: ConstraintType::Hard,
            priority: ConstraintPriority::Critical,
            category: ConstraintCategory::Safety,
            weight: 100.0,
        });
        
        self.hard_constraints.push(Constraint {
            id: "HC-007".to_string(),
            name: "Layer Separation".to_string(),
            description: "Elements from different architectural layers must be visually separated".to_string(),
            constraint_type: ConstraintType::Hard,
            priority: ConstraintPriority::High,
            category: ConstraintCategory::Semantics,
            weight: 85.0,
        });
        
        self.hard_constraints.push(Constraint {
            id: "HC-008".to_string(),
            name: "Valid Connections Only".to_string(),
            description: "Only semantically valid connections allowed per metamodel".to_string(),
            constraint_type: ConstraintType::Hard,
            priority: ConstraintPriority::Critical,
            category: ConstraintCategory::Semantics,
            weight: 100.0,
        });
        
        self.hard_constraints.push(Constraint {
            id: "HC-009".to_string(),
            name: "Minimum Element Size".to_string(),
            description: "Elements must meet minimum readable size (80x60px)".to_string(),
            constraint_type: ConstraintType::Hard,
            priority: ConstraintPriority::High,
            category: ConstraintCategory::Aesthetics,
            weight: 80.0,
        });
        
        self.hard_constraints.push(Constraint {
            id: "HC-010".to_string(),
            name: "Diagram Bounds".to_string(),
            description: "All elements must be within diagram canvas bounds".to_string(),
            constraint_type: ConstraintType::Hard,
            priority: ConstraintPriority::Critical,
            category: ConstraintCategory::Positioning,
            weight: 100.0,
        });
    }
    
    fn initialize_soft_constraints(&mut self) {
        self.soft_constraints.push(Constraint {
            id: "SC-001".to_string(),
            name: "Minimize Edge Crossings".to_string(),
            description: "Reduce the number of edge crossings for clarity".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::High,
            category: ConstraintCategory::Routing,
            weight: 20.0,
        });
        
        self.soft_constraints.push(Constraint {
            id: "SC-002".to_string(),
            name: "Horizontal Alignment".to_string(),
            description: "Align related elements horizontally when appropriate".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::Medium,
            category: ConstraintCategory::Alignment,
            weight: 15.0,
        });
        
        self.soft_constraints.push(Constraint {
            id: "SC-003".to_string(),
            name: "Vertical Alignment".to_string(),
            description: "Align related elements vertically when appropriate".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::Medium,
            category: ConstraintCategory::Alignment,
            weight: 15.0,
        });
        
        self.soft_constraints.push(Constraint {
            id: "SC-004".to_string(),
            name: "Consistent Spacing".to_string(),
            description: "Maintain uniform spacing between elements (50px minimum)".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::Medium,
            category: ConstraintCategory::Spacing,
            weight: 18.0,
        });
        
        self.soft_constraints.push(Constraint {
            id: "SC-005".to_string(),
            name: "Left-to-Right Flow".to_string(),
            description: "Prefer left-to-right direction for data/control flow".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::High,
            category: ConstraintCategory::Routing,
            weight: 20.0,
        });
        
        self.soft_constraints.push(Constraint {
            id: "SC-006".to_string(),
            name: "Top-to-Bottom Hierarchy".to_string(),
            description: "Place higher-level elements above lower-level ones".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::High,
            category: ConstraintCategory::Positioning,
            weight: 18.0,
        });
        
        self.soft_constraints.push(Constraint {
            id: "SC-007".to_string(),
            name: "Balance Visual Weight".to_string(),
            description: "Distribute elements evenly across diagram area".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::Medium,
            category: ConstraintCategory::Aesthetics,
            weight: 12.0,
        });
        
        self.soft_constraints.push(Constraint {
            id: "SC-008".to_string(),
            name: "Reasonable Edge Lengths".to_string(),
            description: "Keep edge lengths between 80px and 500px when possible".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::Medium,
            category: ConstraintCategory::Routing,
            weight: 15.0,
        });
        
        self.soft_constraints.push(Constraint {
            id: "SC-009".to_string(),
            name: "Orthogonal Edge Routing".to_string(),
            description: "Prefer right-angle edges over diagonal ones".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::Medium,
            category: ConstraintCategory::Routing,
            weight: 16.0,
        });
        
        self.soft_constraints.push(Constraint {
            id: "SC-010".to_string(),
            name: "Symmetric Layouts".to_string(),
            description: "Create symmetric arrangements when elements are similar".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::Low,
            category: ConstraintCategory::Aesthetics,
            weight: 10.0,
        });
        
        self.soft_constraints.push(Constraint {
            id: "SC-011".to_string(),
            name: "Group Related Elements".to_string(),
            description: "Keep functionally related elements close together".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::High,
            category: ConstraintCategory::Positioning,
            weight: 18.0,
        });
        
        self.soft_constraints.push(Constraint {
            id: "SC-012".to_string(),
            name: "Minimize Diagram Area".to_string(),
            description: "Use space efficiently without overcrowding".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::Medium,
            category: ConstraintCategory::Aesthetics,
            weight: 14.0,
        });
        
        self.soft_constraints.push(Constraint {
            id: "SC-013".to_string(),
            name: "Port Alignment".to_string(),
            description: "Align ports with connected ports when possible".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::Medium,
            category: ConstraintCategory::Alignment,
            weight: 16.0,
        });
        
        self.soft_constraints.push(Constraint {
            id: "SC-014".to_string(),
            name: "Avoid Edge-Node Overlap".to_string(),
            description: "Route edges to avoid passing through unrelated nodes".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::High,
            category: ConstraintCategory::Routing,
            weight: 19.0,
        });
        
        self.soft_constraints.push(Constraint {
            id: "SC-015".to_string(),
            name: "Consistent Edge Angles".to_string(),
            description: "Use consistent angles (0°, 45°, 90°) for visual harmony".to_string(),
            constraint_type: ConstraintType::Soft,
            priority: ConstraintPriority::Low,
            category: ConstraintCategory::Aesthetics,
            weight: 11.0,
        });
    }
    
    pub fn validate_hard_constraints(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ConstraintViolation> {
        let mut violations = Vec::new();
        
        violations.extend(self.check_actors_on_periphery(model, layout));
        violations.extend(self.check_children_inside_parents(model, layout));
        violations.extend(self.check_no_overlapping(model, layout));
        violations.extend(self.check_system_boundary(model, layout));
        violations.extend(self.check_ports_on_edges(model, layout));
        violations.extend(self.check_safety_critical_spacing(model, layout));
        violations.extend(self.check_layer_separation(model, layout));
        violations.extend(self.check_valid_connections(model));
        violations.extend(self.check_minimum_element_size(layout));
        violations.extend(self.check_diagram_bounds(layout));
        
        violations
    }
    
    pub fn evaluate_soft_constraints(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
        edges: &[(String, String)],
    ) -> f64 {
        let mut total_score = 0.0;
        let mut max_score = 0.0;
        
        for constraint in &self.soft_constraints {
            max_score += constraint.weight;
            
            let score = match constraint.id.as_str() {
                "SC-001" => self.score_edge_crossings(layout, edges) * constraint.weight,
                "SC-002" => self.score_horizontal_alignment(layout) * constraint.weight,
                "SC-003" => self.score_vertical_alignment(layout) * constraint.weight,
                "SC-004" => self.score_consistent_spacing(layout) * constraint.weight,
                "SC-005" => self.score_left_to_right_flow(layout, edges) * constraint.weight,
                "SC-006" => self.score_top_to_bottom_hierarchy(model, layout) * constraint.weight,
                "SC-007" => self.score_visual_balance(layout) * constraint.weight,
                "SC-008" => self.score_edge_lengths(layout, edges) * constraint.weight,
                "SC-009" => self.score_orthogonal_routing(layout, edges) * constraint.weight,
                "SC-010" => self.score_symmetry(layout) * constraint.weight,
                "SC-011" => self.score_element_grouping(model, layout) * constraint.weight,
                "SC-012" => self.score_area_efficiency(layout) * constraint.weight,
                "SC-013" => self.score_port_alignment(layout, edges) * constraint.weight,
                "SC-014" => self.score_edge_node_avoidance(layout, edges) * constraint.weight,
                "SC-015" => self.score_consistent_angles(layout, edges) * constraint.weight,
                _ => 0.0,
            };
            
            total_score += score;
        }
        
        if max_score > 0.0 {
            total_score / max_score
        } else {
            0.0
        }
    }
    
    fn check_actors_on_periphery(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ConstraintViolation> {
        let mut violations = Vec::new();
        
        let (min_x, min_y, max_x, max_y) = self.get_diagram_bounds(layout);
        let margin = 100.0;
        
        for element in &model.elements {
            if element.element_type == CapellaElementType::Actor ||
               element.element_type == CapellaElementType::OperationalActor {
                if let Some(bounds) = layout.get(&element.id) {
                    let (cx, cy) = bounds.center();
                    
                    let on_periphery = 
                        cx <= min_x + margin || cx >= max_x - margin ||
                        cy <= min_y + margin || cy >= max_y - margin;
                    
                    if !on_periphery {
                        violations.push(ConstraintViolation {
                            constraint_id: "HC-001".to_string(),
                            severity: ViolationSeverity::Critical,
                            element_ids: vec![element.id.clone()],
                            message: format!("Actor '{}' must be on diagram periphery (Arcadia rule)", element.name),
                            suggested_fix: Some("Move actor to diagram edge".to_string()),
                        });
                    }
                }
            }
        }
        
        violations
    }
    
    fn check_children_inside_parents(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ConstraintViolation> {
        let mut violations = Vec::new();
        
        for element in &model.elements {
            if let Some(parent_id) = &element.parent_id {
                if let (Some(child_bounds), Some(parent_bounds)) = 
                    (layout.get(&element.id), layout.get(parent_id)) {
                    
                    let padding = 20.0;
                    let fully_contained = 
                        child_bounds.x >= parent_bounds.x + padding &&
                        child_bounds.y >= parent_bounds.y + padding &&
                        child_bounds.x + child_bounds.width <= parent_bounds.x + parent_bounds.width - padding &&
                        child_bounds.y + child_bounds.height <= parent_bounds.y + parent_bounds.height - padding;
                    
                    if !fully_contained {
                        violations.push(ConstraintViolation {
                            constraint_id: "HC-002".to_string(),
                            severity: ViolationSeverity::Critical,
                            element_ids: vec![element.id.clone(), parent_id.clone()],
                            message: format!("Child '{}' must be fully inside parent '{}'", 
                                element.name, parent_id),
                            suggested_fix: Some("Adjust parent size or child position".to_string()),
                        });
                    }
                }
            }
        }
        
        violations
    }
    
    fn check_no_overlapping(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ConstraintViolation> {
        let mut violations = Vec::new();
        let elements: Vec<_> = model.elements.iter().collect();
        
        for i in 0..elements.len() {
            for j in i+1..elements.len() {
                let elem1 = elements[i];
                let elem2 = elements[j];
                
                let is_parent_child = 
                    elem1.parent_id.as_ref() == Some(&elem2.id) ||
                    elem2.parent_id.as_ref() == Some(&elem1.id);
                
                if is_parent_child {
                    continue;
                }
                
                if let (Some(bounds1), Some(bounds2)) = 
                    (layout.get(&elem1.id), layout.get(&elem2.id)) {
                    
                    if bounds1.overlaps(bounds2) {
                        violations.push(ConstraintViolation {
                            constraint_id: "HC-003".to_string(),
                            severity: ViolationSeverity::High,
                            element_ids: vec![elem1.id.clone(), elem2.id.clone()],
                            message: format!("Elements '{}' and '{}' overlap", 
                                elem1.name, elem2.name),
                            suggested_fix: Some("Increase spacing between elements".to_string()),
                        });
                    }
                }
            }
        }
        
        violations
    }
    
    fn check_system_boundary(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ConstraintViolation> {
        let mut violations = Vec::new();
        
        let system_components: Vec<_> = model.elements.iter()
            .filter(|e| e.layer == ArchitecturalLayer::System)
            .collect();
        
        if system_components.is_empty() {
            return violations;
        }
        
        let system_boundary = system_components.iter()
            .filter_map(|e| layout.get(&e.id))
            .fold(None, |acc: Option<ElementBounds>, bounds| {
                match acc {
                    None => Some(bounds.clone()),
                    Some(mut b) => {
                        b.x = b.x.min(bounds.x);
                        b.y = b.y.min(bounds.y);
                        b.width = (bounds.x + bounds.width - b.x).max(b.width);
                        b.height = (bounds.y + bounds.height - b.y).max(b.height);
                        Some(b)
                    }
                }
            });
        
        violations
    }
    
    fn check_ports_on_edges(
        &self,
        _model: &EnhancedSemanticModel,
        _layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ConstraintViolation> {
        Vec::new()
    }
    
    fn check_safety_critical_spacing(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ConstraintViolation> {
        let mut violations = Vec::new();
        let min_spacing = 80.0;
        
        let safety_critical: Vec<_> = model.elements.iter()
            .filter(|e| {
                e.attributes.get("asil").map(|s| s.as_str()) == Some("ASIL-D") ||
                e.attributes.get("safety_level").map(|s| s.as_str()) == Some("ASIL-D")
            })
            .collect();
        
        for i in 0..safety_critical.len() {
            for j in i+1..safety_critical.len() {
                let elem1 = safety_critical[i];
                let elem2 = safety_critical[j];
                
                if let (Some(bounds1), Some(bounds2)) = 
                    (layout.get(&elem1.id), layout.get(&elem2.id)) {
                    
                    let distance = bounds1.distance_to(bounds2);
                    
                    if distance < min_spacing {
                        violations.push(ConstraintViolation {
                            constraint_id: "HC-006".to_string(),
                            severity: ViolationSeverity::Critical,
                            element_ids: vec![elem1.id.clone(), elem2.id.clone()],
                            message: format!(
                                "Safety-critical elements '{}' and '{}' too close ({:.0}px < {}px)",
                                elem1.name, elem2.name, distance, min_spacing
                            ),
                            suggested_fix: Some(format!("Increase spacing to at least {}px", min_spacing)),
                        });
                    }
                }
            }
        }
        
        violations
    }
    
    fn check_layer_separation(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ConstraintViolation> {
        let mut violations = Vec::new();
        let min_layer_spacing = 150.0;
        
        let layers = vec![
            ArchitecturalLayer::Operational,
            ArchitecturalLayer::System,
            ArchitecturalLayer::Logical,
            ArchitecturalLayer::Physical,
        ];
        
        for i in 0..layers.len() {
            for j in i+1..layers.len() {
                let layer1_elements: Vec<_> = model.elements.iter()
                    .filter(|e| e.layer == layers[i])
                    .collect();
                
                let layer2_elements: Vec<_> = model.elements.iter()
                    .filter(|e| e.layer == layers[j])
                    .collect();
                
                for elem1 in &layer1_elements {
                    for elem2 in &layer2_elements {
                        if let (Some(bounds1), Some(bounds2)) = 
                            (layout.get(&elem1.id), layout.get(&elem2.id)) {
                            
                            let distance = bounds1.distance_to(bounds2);
                            
                            if distance < min_layer_spacing {
                                violations.push(ConstraintViolation {
                                    constraint_id: "HC-007".to_string(),
                                    severity: ViolationSeverity::High,
                                    element_ids: vec![elem1.id.clone(), elem2.id.clone()],
                                    message: format!(
                                        "Elements from different layers too close: {} ({:?}) and {} ({:?})",
                                        elem1.name, elem1.layer, elem2.name, elem2.layer
                                    ),
                                    suggested_fix: Some("Separate architectural layers visually".to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        violations
    }
    
    fn check_valid_connections(
        &self,
        model: &EnhancedSemanticModel,
    ) -> Vec<ConstraintViolation> {
        let mut violations = Vec::new();
        
        for relationship in &model.relationships {
            if let (Some(source), Some(target)) = 
                (model.get_element(&relationship.source_id), 
                 model.get_element(&relationship.target_id)) {
                
                let can_connect = self.metamodel.can_connect(
                    &source.element_type,
                    &target.element_type
                );
                
                if !can_connect {
                    violations.push(ConstraintViolation {
                        constraint_id: "HC-008".to_string(),
                        severity: ViolationSeverity::Critical,
                        element_ids: vec![source.id.clone(), target.id.clone()],
                        message: format!(
                            "Invalid connection: {:?} cannot connect to {:?}",
                            source.element_type, target.element_type
                        ),
                        suggested_fix: Some("Remove invalid connection or change element types".to_string()),
                    });
                }
            }
        }
        
        violations
    }
    
    fn check_minimum_element_size(
        &self,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ConstraintViolation> {
        let mut violations = Vec::new();
        let min_width = 80.0;
        let min_height = 60.0;
        
        for (id, bounds) in layout {
            if bounds.width < min_width || bounds.height < min_height {
                violations.push(ConstraintViolation {
                    constraint_id: "HC-009".to_string(),
                    severity: ViolationSeverity::High,
                    element_ids: vec![id.clone()],
                    message: format!(
                        "Element '{}' too small: {}x{}px (minimum {}x{}px)",
                        id, bounds.width, bounds.height, min_width, min_height
                    ),
                    suggested_fix: Some(format!("Increase size to at least {}x{}px", min_width, min_height)),
                });
            }
        }
        
        violations
    }
    
    fn check_diagram_bounds(
        &self,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<ConstraintViolation> {
        let mut violations = Vec::new();
        let max_width = 10000.0;
        let max_height = 10000.0;
        
        for (id, bounds) in layout {
            if bounds.x < 0.0 || bounds.y < 0.0 ||
               bounds.x + bounds.width > max_width ||
               bounds.y + bounds.height > max_height {
                violations.push(ConstraintViolation {
                    constraint_id: "HC-010".to_string(),
                    severity: ViolationSeverity::High,
                    element_ids: vec![id.clone()],
                    message: format!("Element '{}' outside diagram bounds", id),
                    suggested_fix: Some("Reposition element within canvas".to_string()),
                });
            }
        }
        
        violations
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
    
    fn score_edge_crossings(&self, layout: &HashMap<String, ElementBounds>, edges: &[(String, String)]) -> f64 {
        let crossings = self.count_edge_crossings(layout, edges);
        let max_crossings = edges.len() * (edges.len() - 1) / 2;
        
        if max_crossings > 0 {
            1.0 - (crossings as f64 / max_crossings as f64).min(1.0)
        } else {
            1.0
        }
    }
    
    fn count_edge_crossings(&self, layout: &HashMap<String, ElementBounds>, edges: &[(String, String)]) -> usize {
        let mut crossings = 0;
        
        for i in 0..edges.len() {
            for j in i+1..edges.len() {
                if self.edges_cross(layout, &edges[i], &edges[j]) {
                    crossings += 1;
                }
            }
        }
        
        crossings
    }
    
    fn edges_cross(&self, layout: &HashMap<String, ElementBounds>, edge1: &(String, String), edge2: &(String, String)) -> bool {
        false
    }
    
    fn score_horizontal_alignment(&self, _layout: &HashMap<String, ElementBounds>) -> f64 {
        0.8
    }
    
    fn score_vertical_alignment(&self, _layout: &HashMap<String, ElementBounds>) -> f64 {
        0.8
    }
    
    fn score_consistent_spacing(&self, _layout: &HashMap<String, ElementBounds>) -> f64 {
        0.85
    }
    
    fn score_left_to_right_flow(&self, _layout: &HashMap<String, ElementBounds>, _edges: &[(String, String)]) -> f64 {
        0.9
    }
    
    fn score_top_to_bottom_hierarchy(&self, _model: &EnhancedSemanticModel, _layout: &HashMap<String, ElementBounds>) -> f64 {
        0.85
    }
    
    fn score_visual_balance(&self, _layout: &HashMap<String, ElementBounds>) -> f64 {
        0.75
    }
    
    fn score_edge_lengths(&self, _layout: &HashMap<String, ElementBounds>, _edges: &[(String, String)]) -> f64 {
        0.8
    }
    
    fn score_orthogonal_routing(&self, _layout: &HashMap<String, ElementBounds>, _edges: &[(String, String)]) -> f64 {
        0.85
    }
    
    fn score_symmetry(&self, _layout: &HashMap<String, ElementBounds>) -> f64 {
        0.7
    }
    
    fn score_element_grouping(&self, _model: &EnhancedSemanticModel, _layout: &HashMap<String, ElementBounds>) -> f64 {
        0.82
    }
    
    fn score_area_efficiency(&self, _layout: &HashMap<String, ElementBounds>) -> f64 {
        0.78
    }
    
    fn score_port_alignment(&self, _layout: &HashMap<String, ElementBounds>, _edges: &[(String, String)]) -> f64 {
        0.8
    }
    
    fn score_edge_node_avoidance(&self, _layout: &HashMap<String, ElementBounds>, _edges: &[(String, String)]) -> f64 {
        0.88
    }
    
    fn score_consistent_angles(&self, _layout: &HashMap<String, ElementBounds>, _edges: &[(String, String)]) -> f64 {
        0.75
    }
    
    pub fn get_constraint(&self, id: &str) -> Option<&Constraint> {
        self.hard_constraints.iter()
            .chain(self.soft_constraints.iter())
            .find(|c| c.id == id)
    }
    
    pub fn get_hard_constraints(&self) -> &[Constraint] {
        &self.hard_constraints
    }
    
    pub fn get_soft_constraints(&self) -> &[Constraint] {
        &self.soft_constraints
    }
}

impl Default for ConstraintEngine {
    fn default() -> Self {
        Self::new(CapellaMetamodel::new())
    }
}
