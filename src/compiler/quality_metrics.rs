use super::constraint_engine::*;
use super::semantic_enhanced::*;
use super::capella_metamodel::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub edge_crossing_count: usize,
    pub edge_crossing_score: f64,
    
    pub average_edge_length: f64,
    pub edge_length_score: f64,
    
    pub alignment_score: f64,
    pub horizontal_alignment: f64,
    pub vertical_alignment: f64,
    
    pub whitespace_distribution: f64,
    pub whitespace_balance: f64,
    
    pub arcadia_compliance_score: f64,
    pub actors_periphery_score: f64,
    pub layer_separation_score: f64,
    pub flow_direction_score: f64,
    
    pub safety_emphasis_score: f64,
    pub safety_spacing_score: f64,
    pub safety_visibility_score: f64,
    
    pub aesthetic_balance: f64,
    pub symmetry_score: f64,
    pub compactness_score: f64,
    pub visual_hierarchy_score: f64,
    
    pub overall_score: f64,
}

impl QualityMetrics {
    pub fn new() -> Self {
        Self {
            edge_crossing_count: 0,
            edge_crossing_score: 0.0,
            average_edge_length: 0.0,
            edge_length_score: 0.0,
            alignment_score: 0.0,
            horizontal_alignment: 0.0,
            vertical_alignment: 0.0,
            whitespace_distribution: 0.0,
            whitespace_balance: 0.0,
            arcadia_compliance_score: 0.0,
            actors_periphery_score: 0.0,
            layer_separation_score: 0.0,
            flow_direction_score: 0.0,
            safety_emphasis_score: 0.0,
            safety_spacing_score: 0.0,
            safety_visibility_score: 0.0,
            aesthetic_balance: 0.0,
            symmetry_score: 0.0,
            compactness_score: 0.0,
            visual_hierarchy_score: 0.0,
            overall_score: 0.0,
        }
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self::new()
    }
}

pub struct QualityEvaluator {
    metamodel: CapellaMetamodel,
    pub weights: MetricWeights,
}

#[derive(Debug, Clone)]
pub struct MetricWeights {
    pub edge_crossings: f64,
    pub edge_length: f64,
    pub alignment: f64,
    pub whitespace: f64,
    pub arcadia_compliance: f64,
    pub safety_emphasis: f64,
    pub aesthetics: f64,
}

impl MetricWeights {
    pub fn default_weights() -> Self {
        Self {
            edge_crossings: 0.20,
            edge_length: 0.15,
            alignment: 0.15,
            whitespace: 0.10,
            arcadia_compliance: 0.20,
            safety_emphasis: 0.12,
            aesthetics: 0.08,
        }
    }
    
    pub fn normalize(&mut self) {
        let total = self.edge_crossings + self.edge_length + self.alignment + 
                   self.whitespace + self.arcadia_compliance + self.safety_emphasis + 
                   self.aesthetics;
        
        if total > 0.0 {
            self.edge_crossings /= total;
            self.edge_length /= total;
            self.alignment /= total;
            self.whitespace /= total;
            self.arcadia_compliance /= total;
            self.safety_emphasis /= total;
            self.aesthetics /= total;
        }
    }
}

impl QualityEvaluator {
    pub fn new(metamodel: CapellaMetamodel) -> Self {
        Self {
            metamodel,
            weights: MetricWeights::default_weights(),
        }
    }
    
    pub fn with_weights(mut self, weights: MetricWeights) -> Self {
        self.weights = weights;
        self
    }
    
    pub fn evaluate(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
        edges: &[(String, String)],
    ) -> QualityMetrics {
        let mut metrics = QualityMetrics::new();
        
        self.evaluate_edge_crossings(&mut metrics, layout, edges);
        self.evaluate_edge_lengths(&mut metrics, layout, edges);
        self.evaluate_alignment(&mut metrics, layout);
        self.evaluate_whitespace(&mut metrics, layout);
        self.evaluate_arcadia_compliance(&mut metrics, model, layout);
        self.evaluate_safety_emphasis(&mut metrics, model, layout);
        self.evaluate_aesthetics(&mut metrics, layout);
        
        metrics.overall_score = self.compute_overall_score(&metrics);
        
        metrics
    }
    
    fn evaluate_edge_crossings(
        &self,
        metrics: &mut QualityMetrics,
        layout: &HashMap<String, ElementBounds>,
        edges: &[(String, String)],
    ) {
        let crossings = self.count_edge_crossings(layout, edges);
        metrics.edge_crossing_count = crossings;
        
        let max_possible = edges.len() * (edges.len() - 1) / 2;
        
        if max_possible > 0 {
            metrics.edge_crossing_score = 1.0 - (crossings as f64 / max_possible as f64).min(1.0);
        } else {
            metrics.edge_crossing_score = 1.0;
        }
    }
    
    fn count_edge_crossings(
        &self,
        layout: &HashMap<String, ElementBounds>,
        edges: &[(String, String)],
    ) -> usize {
        let mut crossings = 0;
        
        for i in 0..edges.len() {
            for j in i+1..edges.len() {
                if self.edges_intersect(layout, &edges[i], &edges[j]) {
                    crossings += 1;
                }
            }
        }
        
        crossings
    }
    
    fn edges_intersect(
        &self,
        layout: &HashMap<String, ElementBounds>,
        edge1: &(String, String),
        edge2: &(String, String),
    ) -> bool {
        let (src1, tgt1) = edge1;
        let (src2, tgt2) = edge2;
        
        if src1 == src2 || src1 == tgt2 || tgt1 == src2 || tgt1 == tgt2 {
            return false;
        }
        
        if let (Some(b1), Some(b2), Some(b3), Some(b4)) = 
            (layout.get(src1), layout.get(tgt1), layout.get(src2), layout.get(tgt2)) {
            
            let (x1, y1) = b1.center();
            let (x2, y2) = b2.center();
            let (x3, y3) = b3.center();
            let (x4, y4) = b4.center();
            
            self.line_segments_intersect((x1, y1), (x2, y2), (x3, y3), (x4, y4))
        } else {
            false
        }
    }
    
    fn line_segments_intersect(
        &self,
        p1: (f64, f64),
        p2: (f64, f64),
        p3: (f64, f64),
        p4: (f64, f64),
    ) -> bool {
        let ccw = |a: (f64, f64), b: (f64, f64), c: (f64, f64)| -> bool {
            (c.1 - a.1) * (b.0 - a.0) > (b.1 - a.1) * (c.0 - a.0)
        };
        
        ccw(p1, p3, p4) != ccw(p2, p3, p4) && ccw(p1, p2, p3) != ccw(p1, p2, p4)
    }
    
    fn evaluate_edge_lengths(
        &self,
        metrics: &mut QualityMetrics,
        layout: &HashMap<String, ElementBounds>,
        edges: &[(String, String)],
    ) {
        if edges.is_empty() {
            metrics.average_edge_length = 0.0;
            metrics.edge_length_score = 1.0;
            return;
        }
        
        let mut total_length = 0.0;
        let mut count = 0;
        
        for (src, tgt) in edges {
            if let (Some(b1), Some(b2)) = (layout.get(src), layout.get(tgt)) {
                total_length += b1.distance_to(b2);
                count += 1;
            }
        }
        
        if count > 0 {
            metrics.average_edge_length = total_length / count as f64;
            
            let optimal_min = 80.0;
            let optimal_max = 500.0;
            
            if metrics.average_edge_length >= optimal_min && metrics.average_edge_length <= optimal_max {
                metrics.edge_length_score = 1.0;
            } else if metrics.average_edge_length < optimal_min {
                metrics.edge_length_score = metrics.average_edge_length / optimal_min;
            } else {
                metrics.edge_length_score = optimal_max / metrics.average_edge_length;
            }
        } else {
            metrics.average_edge_length = 0.0;
            metrics.edge_length_score = 1.0;
        }
    }
    
    fn evaluate_alignment(
        &self,
        metrics: &mut QualityMetrics,
        layout: &HashMap<String, ElementBounds>,
    ) {
        let tolerance = 10.0;
        
        let mut horizontal_aligned = 0;
        let mut vertical_aligned = 0;
        let mut total_pairs = 0;
        
        let elements: Vec<_> = layout.values().collect();
        
        for i in 0..elements.len() {
            for j in i+1..elements.len() {
                total_pairs += 1;
                
                let (_, y1) = elements[i].center();
                let (_, y2) = elements[j].center();
                
                if (y1 - y2).abs() < tolerance {
                    horizontal_aligned += 1;
                }
                
                let (x1, _) = elements[i].center();
                let (x2, _) = elements[j].center();
                
                if (x1 - x2).abs() < tolerance {
                    vertical_aligned += 1;
                }
            }
        }
        
        if total_pairs > 0 {
            metrics.horizontal_alignment = horizontal_aligned as f64 / total_pairs as f64;
            metrics.vertical_alignment = vertical_aligned as f64 / total_pairs as f64;
            metrics.alignment_score = (metrics.horizontal_alignment + metrics.vertical_alignment) / 2.0;
        } else {
            metrics.horizontal_alignment = 1.0;
            metrics.vertical_alignment = 1.0;
            metrics.alignment_score = 1.0;
        }
    }
    
    fn evaluate_whitespace(
        &self,
        metrics: &mut QualityMetrics,
        layout: &HashMap<String, ElementBounds>,
    ) {
        if layout.is_empty() {
            metrics.whitespace_distribution = 1.0;
            metrics.whitespace_balance = 1.0;
            return;
        }
        
        let (min_x, min_y, max_x, max_y) = self.get_bounds(layout);
        
        let grid_size = 4;
        let cell_width = (max_x - min_x) / grid_size as f64;
        let cell_height = (max_y - min_y) / grid_size as f64;
        
        let mut cell_occupancy = vec![vec![0.0; grid_size]; grid_size];
        
        for bounds in layout.values() {
            let cell_x = ((bounds.x + bounds.width / 2.0 - min_x) / cell_width).floor() as usize;
            let cell_y = ((bounds.y + bounds.height / 2.0 - min_y) / cell_height).floor() as usize;
            
            let cell_x = cell_x.min(grid_size - 1);
            let cell_y = cell_y.min(grid_size - 1);
            
            cell_occupancy[cell_y][cell_x] += bounds.width * bounds.height;
        }
        
        let total_area: f64 = cell_occupancy.iter()
            .flat_map(|row| row.iter())
            .sum();
        
        if total_area > 0.0 {
            let densities: Vec<f64> = cell_occupancy.iter()
                .flat_map(|row| row.iter())
                .map(|&area| area / total_area)
                .collect();
            
            let mean = densities.iter().sum::<f64>() / densities.len() as f64;
            let variance = densities.iter()
                .map(|d| (d - mean).powi(2))
                .sum::<f64>() / densities.len() as f64;
            
            metrics.whitespace_balance = 1.0 - variance.sqrt().min(1.0);
        } else {
            metrics.whitespace_balance = 1.0;
        }
        
        let total_element_area: f64 = layout.values()
            .map(|b| b.width * b.height)
            .sum();
        
        let diagram_area = (max_x - min_x) * (max_y - min_y);
        
        if diagram_area > 0.0 {
            let density = total_element_area / diagram_area;
            
            let optimal_density = 0.4;
            metrics.whitespace_distribution = 1.0 - (density - optimal_density).abs() / optimal_density;
            metrics.whitespace_distribution = metrics.whitespace_distribution.max(0.0);
        } else {
            metrics.whitespace_distribution = 1.0;
        }
    }
    
    fn evaluate_arcadia_compliance(
        &self,
        metrics: &mut QualityMetrics,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) {
        metrics.actors_periphery_score = self.score_actors_on_periphery(model, layout);
        
        metrics.layer_separation_score = self.score_layer_separation(model, layout);
        
        metrics.flow_direction_score = self.score_flow_direction(model, layout);
        
        metrics.arcadia_compliance_score = 
            (metrics.actors_periphery_score + 
             metrics.layer_separation_score + 
             metrics.flow_direction_score) / 3.0;
    }
    
    fn score_actors_on_periphery(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> f64 {
        let actors: Vec<_> = model.elements.iter()
            .filter(|e| e.element_type == CapellaElementType::Actor || 
                       e.element_type == CapellaElementType::OperationalActor)
            .collect();
        
        if actors.is_empty() {
            return 1.0;
        }
        
        let (min_x, min_y, max_x, max_y) = self.get_bounds(layout);
        let margin = 100.0;
        
        let mut on_periphery = 0;
        
        for actor in &actors {
            if let Some(bounds) = layout.get(&actor.id) {
                let (cx, cy) = bounds.center();
                
                if cx <= min_x + margin || cx >= max_x - margin ||
                   cy <= min_y + margin || cy >= max_y - margin {
                    on_periphery += 1;
                }
            }
        }
        
        on_periphery as f64 / actors.len() as f64
    }
    
    fn score_layer_separation(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> f64 {
        let layers = vec![
            ArchitecturalLayer::Operational,
            ArchitecturalLayer::System,
            ArchitecturalLayer::Logical,
            ArchitecturalLayer::Physical,
        ];
        
        let mut layer_centers: HashMap<ArchitecturalLayer, (f64, f64)> = HashMap::new();
        let mut layer_counts: HashMap<ArchitecturalLayer, usize> = HashMap::new();
        
        for element in &model.elements {
            if let Some(bounds) = layout.get(&element.id) {
                let (cx, cy) = bounds.center();
                
                let entry = layer_centers.entry(element.layer).or_insert((0.0, 0.0));
                entry.0 += cx;
                entry.1 += cy;
                
                *layer_counts.entry(element.layer).or_insert(0) += 1;
            }
        }
        
        for (layer, center) in layer_centers.iter_mut() {
            if let Some(&count) = layer_counts.get(layer) {
                if count > 0 {
                    center.0 /= count as f64;
                    center.1 /= count as f64;
                }
            }
        }
        
        let mut separation_score = 1.0;
        let min_separation = 150.0;
        
        for i in 0..layers.len() {
            for j in i+1..layers.len() {
                if let (Some(c1), Some(c2)) = 
                    (layer_centers.get(&layers[i]), layer_centers.get(&layers[j])) {
                    
                    let distance = ((c2.0 - c1.0).powi(2) + (c2.1 - c1.1).powi(2)).sqrt();
                    
                    if distance < min_separation {
                        separation_score *= distance / min_separation;
                    }
                }
            }
        }
        
        separation_score
    }
    
    fn score_flow_direction(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> f64 {
        let mut left_to_right = 0;
        let mut total = 0;
        
        for relationship in &model.relationships {
            if let (Some(src_bounds), Some(tgt_bounds)) = 
                (layout.get(&relationship.source_id), layout.get(&relationship.target_id)) {
                
                let (src_x, _) = src_bounds.center();
                let (tgt_x, _) = tgt_bounds.center();
                
                total += 1;
                
                if tgt_x >= src_x {
                    left_to_right += 1;
                }
            }
        }
        
        if total > 0 {
            left_to_right as f64 / total as f64
        } else {
            1.0
        }
    }
    
    fn evaluate_safety_emphasis(
        &self,
        metrics: &mut QualityMetrics,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) {
        metrics.safety_spacing_score = self.score_safety_spacing(model, layout);
        
        metrics.safety_visibility_score = self.score_safety_visibility(model, layout);
        
        metrics.safety_emphasis_score = 
            (metrics.safety_spacing_score + metrics.safety_visibility_score) / 2.0;
    }
    
    fn score_safety_spacing(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> f64 {
        let safety_critical: Vec<_> = model.elements.iter()
            .filter(|e| {
                e.attributes.get("asil").map(|s| s.as_str()) == Some("ASIL-D") ||
                e.attributes.get("safety_level").map(|s| s.as_str()) == Some("ASIL-D")
            })
            .collect();
        
        if safety_critical.len() < 2 {
            return 1.0;
        }
        
        let min_spacing = 80.0;
        let mut score = 1.0;
        let mut pairs = 0;
        
        for i in 0..safety_critical.len() {
            for j in i+1..safety_critical.len() {
                if let (Some(b1), Some(b2)) = 
                    (layout.get(&safety_critical[i].id), layout.get(&safety_critical[j].id)) {
                    
                    pairs += 1;
                    let distance = b1.distance_to(b2);
                    
                    if distance < min_spacing {
                        score *= distance / min_spacing;
                    }
                }
            }
        }
        
        score
    }
    
    fn score_safety_visibility(
        &self,
        model: &EnhancedSemanticModel,
        layout: &HashMap<String, ElementBounds>,
    ) -> f64 {
        let safety_critical: Vec<_> = model.elements.iter()
            .filter(|e| {
                e.attributes.get("asil").map(|s| s.as_str()) == Some("ASIL-D") ||
                e.attributes.get("safety_level").map(|s| s.as_str()) == Some("ASIL-D")
            })
            .collect();
        
        if safety_critical.is_empty() {
            return 1.0;
        }
        
        let total_elements = layout.len();
        let safety_count = safety_critical.len();
        
        if total_elements > 0 {
            (safety_count as f64 / total_elements as f64).min(1.0) * 0.5 + 0.5
        } else {
            1.0
        }
    }
    
    fn evaluate_aesthetics(
        &self,
        metrics: &mut QualityMetrics,
        layout: &HashMap<String, ElementBounds>,
    ) {
        metrics.symmetry_score = self.score_symmetry(layout);
        metrics.compactness_score = self.score_compactness(layout);
        metrics.visual_hierarchy_score = self.score_visual_hierarchy(layout);
        
        metrics.aesthetic_balance = 
            (metrics.symmetry_score + metrics.compactness_score + metrics.visual_hierarchy_score) / 3.0;
    }
    
    fn score_symmetry(&self, layout: &HashMap<String, ElementBounds>) -> f64 {
        if layout.len() < 2 {
            return 1.0;
        }
        
        let (min_x, _, max_x, _) = self.get_bounds(layout);
        let center_x = (min_x + max_x) / 2.0;
        
        let elements: Vec<_> = layout.values().collect();
        
        let mut symmetry_score = 0.0;
        let mut count = 0;
        
        for elem in &elements {
            let (cx, _) = elem.center();
            let distance_from_center = (cx - center_x).abs();
            
            let mut best_match: f64 = 0.0;
            
            for other in &elements {
                if std::ptr::eq(*elem, *other) {
                    continue;
                }
                
                let (other_cx, _) = other.center();
                let other_distance = (other_cx - center_x).abs();
                
                if (distance_from_center - other_distance).abs() < 50.0 {
                    let match_score = 1.0 - ((distance_from_center - other_distance).abs() / 50.0);
                    best_match = best_match.max(match_score as f64);
                }
            }
            
            symmetry_score += best_match;
            count += 1;
        }
        
        if count > 0 {
            symmetry_score / count as f64
        } else {
            1.0
        }
    }
    
    fn score_compactness(&self, layout: &HashMap<String, ElementBounds>) -> f64 {
        if layout.is_empty() {
            return 1.0;
        }
        
        let total_element_area: f64 = layout.values()
            .map(|b| b.width * b.height)
            .sum();
        
        let (min_x, min_y, max_x, max_y) = self.get_bounds(layout);
        let bounding_box_area = (max_x - min_x) * (max_y - min_y);
        
        if bounding_box_area > 0.0 {
            (total_element_area / bounding_box_area).min(1.0)
        } else {
            1.0
        }
    }
    
    fn score_visual_hierarchy(&self, layout: &HashMap<String, ElementBounds>) -> f64 {
        0.8
    }
    
    fn compute_overall_score(&self, metrics: &QualityMetrics) -> f64 {
        self.weights.edge_crossings * metrics.edge_crossing_score +
        self.weights.edge_length * metrics.edge_length_score +
        self.weights.alignment * metrics.alignment_score +
        self.weights.whitespace * metrics.whitespace_distribution +
        self.weights.arcadia_compliance * metrics.arcadia_compliance_score +
        self.weights.safety_emphasis * metrics.safety_emphasis_score +
        self.weights.aesthetics * metrics.aesthetic_balance
    }
    
    fn get_bounds(&self, layout: &HashMap<String, ElementBounds>) -> (f64, f64, f64, f64) {
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

impl Default for QualityEvaluator {
    fn default() -> Self {
        Self::new(CapellaMetamodel::new())
    }
}
