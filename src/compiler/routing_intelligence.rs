use super::constraint_engine::*;
use super::semantic_enhanced::*;
use super::capella_metamodel::*;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeType {
    FunctionalExchange,
    ComponentExchange,
    PhysicalLink,
    Allocation,
    Refinement,
    Generalization,
    Association,
    Dependency,
    ControlFlow,
    DataFlow,
}

impl EdgeType {
    pub fn from_relationship_type(rel_type: &RelationshipType) -> Self {
        match rel_type {
            RelationshipType::FunctionalExchange => EdgeType::FunctionalExchange,
            RelationshipType::ComponentExchange => EdgeType::ComponentExchange,
            RelationshipType::PhysicalLink => EdgeType::PhysicalLink,
            RelationshipType::Allocates => EdgeType::Allocation,
            RelationshipType::Realizes => EdgeType::Refinement,
            RelationshipType::Extends => EdgeType::Generalization,
            RelationshipType::Uses => EdgeType::Association,
            RelationshipType::Implements => EdgeType::Dependency,
            RelationshipType::Contains => EdgeType::Association,
            RelationshipType::Connects => EdgeType::DataFlow,
            _ => EdgeType::Association,
        }
    }
    
    pub fn from_relationship_type_str(rel_type: &str) -> Self {
        match rel_type {
            "functional_exchange" => EdgeType::FunctionalExchange,
            "component_exchange" => EdgeType::ComponentExchange,
            "physical_link" => EdgeType::PhysicalLink,
            "allocation" => EdgeType::Allocation,
            "refinement" => EdgeType::Refinement,
            "generalization" => EdgeType::Generalization,
            "association" => EdgeType::Association,
            "dependency" => EdgeType::Dependency,
            "control_flow" => EdgeType::ControlFlow,
            "data_flow" => EdgeType::DataFlow,
            _ => EdgeType::Association,
        }
    }
    
    pub fn routing_style(&self) -> RoutingStyle {
        match self {
            EdgeType::FunctionalExchange => RoutingStyle::Orthogonal,
            EdgeType::ComponentExchange => RoutingStyle::Orthogonal,
            EdgeType::PhysicalLink => RoutingStyle::Polyline,
            EdgeType::Allocation => RoutingStyle::Straight,
            EdgeType::Refinement => RoutingStyle::Straight,
            EdgeType::Generalization => RoutingStyle::Straight,
            EdgeType::Association => RoutingStyle::Polyline,
            EdgeType::Dependency => RoutingStyle::Polyline,
            EdgeType::ControlFlow => RoutingStyle::Orthogonal,
            EdgeType::DataFlow => RoutingStyle::Polyline,
        }
    }
    
    pub fn visual_style(&self) -> EdgeVisualStyle {
        match self {
            EdgeType::FunctionalExchange => EdgeVisualStyle {
                stroke_width: 2.0,
                stroke_color: "#2196F3".to_string(),
                stroke_dasharray: None,
                arrow_type: ArrowType::Standard,
                label_background: Some("#E3F2FD".to_string()),
            },
            EdgeType::ComponentExchange => EdgeVisualStyle {
                stroke_width: 2.0,
                stroke_color: "#4CAF50".to_string(),
                stroke_dasharray: None,
                arrow_type: ArrowType::Standard,
                label_background: Some("#E8F5E9".to_string()),
            },
            EdgeType::PhysicalLink => EdgeVisualStyle {
                stroke_width: 2.5,
                stroke_color: "#FF9800".to_string(),
                stroke_dasharray: None,
                arrow_type: ArrowType::Standard,
                label_background: Some("#FFF3E0".to_string()),
            },
            EdgeType::Allocation => EdgeVisualStyle {
                stroke_width: 1.5,
                stroke_color: "#9E9E9E".to_string(),
                stroke_dasharray: Some("5,5".to_string()),
                arrow_type: ArrowType::Open,
                label_background: Some("#FAFAFA".to_string()),
            },
            EdgeType::Refinement => EdgeVisualStyle {
                stroke_width: 1.5,
                stroke_color: "#673AB7".to_string(),
                stroke_dasharray: Some("3,3".to_string()),
                arrow_type: ArrowType::Diamond,
                label_background: Some("#EDE7F6".to_string()),
            },
            EdgeType::Generalization => EdgeVisualStyle {
                stroke_width: 2.0,
                stroke_color: "#3F51B5".to_string(),
                stroke_dasharray: None,
                arrow_type: ArrowType::Triangle,
                label_background: Some("#E8EAF6".to_string()),
            },
            EdgeType::Association => EdgeVisualStyle {
                stroke_width: 1.5,
                stroke_color: "#607D8B".to_string(),
                stroke_dasharray: None,
                arrow_type: ArrowType::Standard,
                label_background: Some("#ECEFF1".to_string()),
            },
            EdgeType::Dependency => EdgeVisualStyle {
                stroke_width: 1.5,
                stroke_color: "#795548".to_string(),
                stroke_dasharray: Some("10,5".to_string()),
                arrow_type: ArrowType::Open,
                label_background: Some("#EFEBE9".to_string()),
            },
            EdgeType::ControlFlow => EdgeVisualStyle {
                stroke_width: 2.0,
                stroke_color: "#F44336".to_string(),
                stroke_dasharray: None,
                arrow_type: ArrowType::Standard,
                label_background: Some("#FFEBEE".to_string()),
            },
            EdgeType::DataFlow => EdgeVisualStyle {
                stroke_width: 2.0,
                stroke_color: "#00BCD4".to_string(),
                stroke_dasharray: None,
                arrow_type: ArrowType::Standard,
                label_background: Some("#E0F7FA".to_string()),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingStyle {
    Straight,
    Orthogonal,
    Polyline,
    Spline,
}

#[derive(Debug, Clone)]
pub struct EdgeVisualStyle {
    pub stroke_width: f64,
    pub stroke_color: String,
    pub stroke_dasharray: Option<String>,
    pub arrow_type: ArrowType,
    pub label_background: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowType {
    Standard,
    Open,
    Diamond,
    Triangle,
    Circle,
    None,
}

#[derive(Debug, Clone)]
pub struct RoutedEdge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub waypoints: Vec<Point>,
    pub routing_style: RoutingStyle,
    pub visual_style: EdgeVisualStyle,
    pub label: Option<String>,
    pub label_position: Option<Point>,
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    
    pub fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
    
    pub fn midpoint(&self, other: &Point) -> Point {
        Point {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
        }
    }
}

pub struct IntelligentRouter {
    metamodel: CapellaMetamodel,
    grid_size: f64,
    min_segment_length: f64,
}

impl IntelligentRouter {
    pub fn new(metamodel: CapellaMetamodel) -> Self {
        Self {
            metamodel,
            grid_size: 20.0,
            min_segment_length: 30.0,
        }
    }
    
    pub fn route_edges(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<RoutedEdge> {
        let mut routed_edges = Vec::new();
        
        for relationship in &model.relationships {
            if let (Some(src_bounds), Some(tgt_bounds)) = 
                (layout.get(&relationship.source_id), layout.get(&relationship.target_id)) {
                
                let edge_type = EdgeType::from_relationship_type(&relationship.relationship_type);
                let routing_style = edge_type.routing_style();
                
                let waypoints = self.compute_waypoints(
                    src_bounds,
                    tgt_bounds,
                    routing_style,
                    layout,
                );
                
                let has_label = !relationship.exchange_items.is_empty();
                
                let label_position = if has_label {
                    Some(self.compute_label_position(&waypoints, layout))
                } else {
                    None
                };
                
                let label = if !relationship.exchange_items.is_empty() {
                    Some(relationship.exchange_items.join(", "))
                } else {
                    None
                };
                
                routed_edges.push(RoutedEdge {
                    source_id: relationship.source_id.clone(),
                    target_id: relationship.target_id.clone(),
                    edge_type,
                    waypoints,
                    routing_style,
                    visual_style: edge_type.visual_style(),
                    label,
                    label_position,
                });
            }
        }
        
        self.optimize_routing(&mut routed_edges, layout);
        
        routed_edges
    }
    
    fn compute_waypoints(
        &self,
        src: &ElementBounds,
        tgt: &ElementBounds,
        style: RoutingStyle,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<Point> {
        match style {
            RoutingStyle::Straight => self.route_straight(src, tgt),
            RoutingStyle::Orthogonal => self.route_orthogonal(src, tgt, layout),
            RoutingStyle::Polyline => self.route_polyline(src, tgt, layout),
            RoutingStyle::Spline => self.route_spline(src, tgt, layout),
        }
    }
    
    fn route_straight(&self, src: &ElementBounds, tgt: &ElementBounds) -> Vec<Point> {
        let (src_x, src_y) = src.center();
        let (tgt_x, tgt_y) = tgt.center();
        
        let src_point = self.find_exit_point(src, tgt_x, tgt_y);
        let tgt_point = self.find_entry_point(tgt, src_x, src_y);
        
        vec![src_point, tgt_point]
    }
    
    fn route_orthogonal(
        &self,
        src: &ElementBounds,
        tgt: &ElementBounds,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<Point> {
        let (src_x, src_y) = src.center();
        let (tgt_x, tgt_y) = tgt.center();
        
        let src_point = self.find_exit_point(src, tgt_x, tgt_y);
        let tgt_point = self.find_entry_point(tgt, src_x, src_y);
        
        let mut waypoints = vec![src_point];
        
        let dx = (tgt_point.x - src_point.x).abs();
        let dy = (tgt_point.y - src_point.y).abs();
        
        if dx > dy {
            let mid_x = (src_point.x + tgt_point.x) / 2.0;
            
            if !self.has_obstacle(src_point.x, src_point.y, mid_x, src_point.y, layout) {
                waypoints.push(Point::new(mid_x, src_point.y));
                waypoints.push(Point::new(mid_x, tgt_point.y));
            } else {
                waypoints.extend(self.find_detour_path(src_point, tgt_point, layout));
            }
        } else {
            let mid_y = (src_point.y + tgt_point.y) / 2.0;
            
            if !self.has_obstacle(src_point.x, src_point.y, src_point.x, mid_y, layout) {
                waypoints.push(Point::new(src_point.x, mid_y));
                waypoints.push(Point::new(tgt_point.x, mid_y));
            } else {
                waypoints.extend(self.find_detour_path(src_point, tgt_point, layout));
            }
        }
        
        waypoints.push(tgt_point);
        waypoints
    }
    
    fn route_polyline(
        &self,
        src: &ElementBounds,
        tgt: &ElementBounds,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<Point> {
        let (src_x, src_y) = src.center();
        let (tgt_x, tgt_y) = tgt.center();
        
        let src_point = self.find_exit_point(src, tgt_x, tgt_y);
        let tgt_point = self.find_entry_point(tgt, src_x, src_y);
        
        if !self.has_obstacle(src_point.x, src_point.y, tgt_point.x, tgt_point.y, layout) {
            vec![src_point, tgt_point]
        } else {
            let mut waypoints = vec![src_point];
            waypoints.extend(self.find_detour_path(src_point, tgt_point, layout));
            waypoints.push(tgt_point);
            waypoints
        }
    }
    
    fn route_spline(
        &self,
        src: &ElementBounds,
        tgt: &ElementBounds,
        _layout: &HashMap<String, ElementBounds>,
    ) -> Vec<Point> {
        let (src_x, src_y) = src.center();
        let (tgt_x, tgt_y) = tgt.center();
        
        let src_point = self.find_exit_point(src, tgt_x, tgt_y);
        let tgt_point = self.find_entry_point(tgt, src_x, src_y);
        
        let control1 = Point::new(
            src_point.x + (tgt_point.x - src_point.x) * 0.33,
            src_point.y,
        );
        
        let control2 = Point::new(
            src_point.x + (tgt_point.x - src_point.x) * 0.67,
            tgt_point.y,
        );
        
        vec![src_point, control1, control2, tgt_point]
    }
    
    fn find_exit_point(&self, bounds: &ElementBounds, target_x: f64, target_y: f64) -> Point {
        let (cx, cy) = bounds.center();
        
        let dx = target_x - cx;
        let dy = target_y - cy;
        
        let angle = dy.atan2(dx);
        
        let half_width = bounds.width / 2.0;
        let half_height = bounds.height / 2.0;
        
        if angle.abs() < std::f64::consts::PI / 4.0 {
            Point::new(cx + half_width, cy)
        } else if angle.abs() > 3.0 * std::f64::consts::PI / 4.0 {
            Point::new(cx - half_width, cy)
        } else if angle > 0.0 {
            Point::new(cx, cy + half_height)
        } else {
            Point::new(cx, cy - half_height)
        }
    }
    
    fn find_entry_point(&self, bounds: &ElementBounds, source_x: f64, source_y: f64) -> Point {
        let (cx, cy) = bounds.center();
        
        let dx = source_x - cx;
        let dy = source_y - cy;
        
        let angle = dy.atan2(dx);
        
        let half_width = bounds.width / 2.0;
        let half_height = bounds.height / 2.0;
        
        if angle.abs() < std::f64::consts::PI / 4.0 {
            Point::new(cx + half_width, cy)
        } else if angle.abs() > 3.0 * std::f64::consts::PI / 4.0 {
            Point::new(cx - half_width, cy)
        } else if angle > 0.0 {
            Point::new(cx, cy + half_height)
        } else {
            Point::new(cx, cy - half_height)
        }
    }
    
    fn has_obstacle(
        &self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        layout: &HashMap<String, ElementBounds>,
    ) -> bool {
        let steps = 10;
        
        for i in 1..steps {
            let t = i as f64 / steps as f64;
            let x = x1 + (x2 - x1) * t;
            let y = y1 + (y2 - y1) * t;
            
            for bounds in layout.values() {
                if x >= bounds.x && x <= bounds.x + bounds.width &&
                   y >= bounds.y && y <= bounds.y + bounds.height {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn find_detour_path(
        &self,
        src: Point,
        tgt: Point,
        layout: &HashMap<String, ElementBounds>,
    ) -> Vec<Point> {
        let mut path = Vec::new();
        
        let dx = tgt.x - src.x;
        let dy = tgt.y - src.y;
        
        if dx.abs() > dy.abs() {
            let mid_x = src.x + dx / 2.0;
            let detour_y = if dy > 0.0 {
                self.find_clear_horizontal(src.y, tgt.y, layout)
            } else {
                self.find_clear_horizontal(tgt.y, src.y, layout)
            };
            
            path.push(Point::new(src.x, detour_y));
            path.push(Point::new(tgt.x, detour_y));
        } else {
            let mid_y = src.y + dy / 2.0;
            let detour_x = if dx > 0.0 {
                self.find_clear_vertical(src.x, tgt.x, layout)
            } else {
                self.find_clear_vertical(tgt.x, src.x, layout)
            };
            
            path.push(Point::new(detour_x, src.y));
            path.push(Point::new(detour_x, tgt.y));
        }
        
        path
    }
    
    fn find_clear_horizontal(
        &self,
        y1: f64,
        y2: f64,
        _layout: &HashMap<String, ElementBounds>,
    ) -> f64 {
        (y1 + y2) / 2.0
    }
    
    fn find_clear_vertical(
        &self,
        x1: f64,
        x2: f64,
        _layout: &HashMap<String, ElementBounds>,
    ) -> f64 {
        (x1 + x2) / 2.0
    }
    
    fn compute_label_position(
        &self,
        waypoints: &[Point],
        _layout: &HashMap<String, ElementBounds>,
    ) -> Point {
        if waypoints.len() < 2 {
            return Point::new(0.0, 0.0);
        }
        
        let mid_idx = waypoints.len() / 2;
        
        if waypoints.len() % 2 == 1 {
            waypoints[mid_idx]
        } else {
            waypoints[mid_idx - 1].midpoint(&waypoints[mid_idx])
        }
    }
    
    fn optimize_routing(
        &self,
        edges: &mut Vec<RoutedEdge>,
        _layout: &HashMap<String, ElementBounds>,
    ) {
        for edge in edges.iter_mut() {
            self.simplify_path(&mut edge.waypoints);
            self.snap_to_grid(&mut edge.waypoints);
        }
    }
    
    fn simplify_path(&self, waypoints: &mut Vec<Point>) {
        if waypoints.len() <= 2 {
            return;
        }
        
        let mut simplified = vec![waypoints[0]];
        
        for i in 1..waypoints.len()-1 {
            let prev = &waypoints[i-1];
            let curr = &waypoints[i];
            let next = &waypoints[i+1];
            
            let dx1 = curr.x - prev.x;
            let dy1 = curr.y - prev.y;
            let dx2 = next.x - curr.x;
            let dy2 = next.y - curr.y;
            
            let is_collinear = (dx1 * dy2 - dy1 * dx2).abs() < 0.01;
            
            if !is_collinear {
                simplified.push(*curr);
            }
        }
        
        simplified.push(waypoints[waypoints.len() - 1]);
        *waypoints = simplified;
    }
    
    fn snap_to_grid(&self, waypoints: &mut Vec<Point>) {
        for point in waypoints.iter_mut() {
            point.x = (point.x / self.grid_size).round() * self.grid_size;
            point.y = (point.y / self.grid_size).round() * self.grid_size;
        }
    }
}

impl Default for IntelligentRouter {
    fn default() -> Self {
        Self::new(CapellaMetamodel::new())
    }
}

pub struct PathOptimizer {
    max_iterations: usize,
    convergence_threshold: f64,
}

impl PathOptimizer {
    pub fn new() -> Self {
        Self {
            max_iterations: 100,
            convergence_threshold: 1.0,
        }
    }
    
    pub fn optimize_paths(
        &self,
        edges: &mut Vec<RoutedEdge>,
        layout: &HashMap<String, ElementBounds>,
    ) {
        for _ in 0..self.max_iterations {
            let mut improved = false;
            
            for edge in edges.iter_mut() {
                if self.optimize_single_path(&mut edge.waypoints, layout) {
                    improved = true;
                }
            }
            
            if !improved {
                break;
            }
        }
    }
    
    fn optimize_single_path(
        &self,
        waypoints: &mut Vec<Point>,
        _layout: &HashMap<String, ElementBounds>,
    ) -> bool {
        if waypoints.len() <= 2 {
            return false;
        }
        
        let original_length = self.path_length(waypoints);
        
        for i in 1..waypoints.len()-1 {
            let prev = waypoints[i-1];
            let next = waypoints[i+1];
            
            let new_point = Point::new(
                (prev.x + next.x) / 2.0,
                (prev.y + next.y) / 2.0,
            );
            
            waypoints[i] = new_point;
        }
        
        let new_length = self.path_length(waypoints);
        
        new_length < original_length - self.convergence_threshold
    }
    
    fn path_length(&self, waypoints: &[Point]) -> f64 {
        let mut length = 0.0;
        
        for i in 0..waypoints.len()-1 {
            length += waypoints[i].distance_to(&waypoints[i+1]);
        }
        
        length
    }
}

impl Default for PathOptimizer {
    fn default() -> Self {
        Self::new()
    }
}
