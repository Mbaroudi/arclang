//! Quality Metrics System for Diagram Assessment
//! 
//! This module provides metrics to evaluate diagram quality:
//! - Edge crossings count
//! - Node overlaps detection
//! - Whitespace balance analysis
//! - Alignment scoring
//! - Arcadia compliance checking

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::compiler::semantic_analyzer::{ArcadiaPhase, SemanticContext};

/// Quality assessment report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub edge_crossings: usize,
    pub node_overlaps: usize,
    pub whitespace_balance: f64,
    pub alignment_score: f64,
    pub arcadia_compliance: f64,
    pub overall_score: f64,
    pub warnings: Vec<String>,
}

/// Individual Arcadia rule
#[derive(Debug, Clone)]
pub struct ArcadiaRule {
    pub name: String,
    pub description: String,
    pub weight: f64,
    pub check_fn: fn(&Value, &SemanticContext) -> bool,
}

/// Quality metrics calculator
pub struct QualityMetrics;

impl QualityMetrics {
    pub fn new() -> Self {
        QualityMetrics
    }
    
    /// Main entry point: Calculate quality metrics
    pub fn calculate(&self, svg_data: &Value, semantic: &SemanticContext) -> QualityReport {
        let edge_crossings = self.count_edge_crossings(svg_data);
        let node_overlaps = self.detect_node_overlaps(svg_data);
        let whitespace_balance = self.assess_whitespace(svg_data);
        let alignment_score = self.score_alignment(svg_data);
        let arcadia_compliance = self.check_arcadia_compliance(svg_data, semantic);
        
        let overall_score = self.calculate_overall_score(
            edge_crossings,
            node_overlaps,
            whitespace_balance,
            alignment_score,
            arcadia_compliance,
        );
        
        let warnings = self.generate_warnings(
            edge_crossings,
            node_overlaps,
            alignment_score,
            arcadia_compliance,
        );
        
        QualityReport {
            edge_crossings,
            node_overlaps,
            whitespace_balance,
            alignment_score,
            arcadia_compliance,
            overall_score,
            warnings,
        }
    }
    
    /// Count edge crossings
    fn count_edge_crossings(&self, svg_data: &Value) -> usize {
        let edges = match svg_data.get("edges").and_then(|e| e.as_array()) {
            Some(e) => e,
            None => return 0,
        };
        
        let mut crossings = 0;
        
        // Extract edge segments
        let segments: Vec<_> = edges
            .iter()
            .filter_map(|edge| {
                let x1 = edge.get("x1").and_then(|v| v.as_f64())?;
                let y1 = edge.get("y1").and_then(|v| v.as_f64())?;
                let x2 = edge.get("x2").and_then(|v| v.as_f64())?;
                let y2 = edge.get("y2").and_then(|v| v.as_f64())?;
                Some((x1, y1, x2, y2))
            })
            .collect();
        
        // Check each pair for intersection
        for i in 0..segments.len() {
            for j in (i + 1)..segments.len() {
                if self.segments_intersect(segments[i], segments[j]) {
                    crossings += 1;
                }
            }
        }
        
        crossings
    }
    
    /// Check if two line segments intersect
    fn segments_intersect(&self, seg1: (f64, f64, f64, f64), seg2: (f64, f64, f64, f64)) -> bool {
        let (x1, y1, x2, y2) = seg1;
        let (x3, y3, x4, y4) = seg2;
        
        let denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
        
        if denom.abs() < 0.0001 {
            return false; // Parallel or coincident
        }
        
        let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;
        let u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denom;
        
        t > 0.0 && t < 1.0 && u > 0.0 && u < 1.0
    }
    
    /// Detect node overlaps
    fn detect_node_overlaps(&self, svg_data: &Value) -> usize {
        let nodes = match svg_data.get("nodes").and_then(|n| n.as_array()) {
            Some(n) => n,
            None => return 0,
        };
        
        let mut overlaps = 0;
        
        // Extract node bounds
        let bounds: Vec<_> = nodes
            .iter()
            .filter_map(|node| {
                let x = node.get("x").and_then(|v| v.as_f64())?;
                let y = node.get("y").and_then(|v| v.as_f64())?;
                let width = node.get("width").and_then(|v| v.as_f64()).unwrap_or(100.0);
                let height = node.get("height").and_then(|v| v.as_f64()).unwrap_or(60.0);
                Some((x, y, width, height))
            })
            .collect();
        
        // Check each pair for overlap
        for i in 0..bounds.len() {
            for j in (i + 1)..bounds.len() {
                if self.rectangles_overlap(bounds[i], bounds[j]) {
                    overlaps += 1;
                }
            }
        }
        
        overlaps
    }
    
    /// Check if two rectangles overlap
    fn rectangles_overlap(&self, rect1: (f64, f64, f64, f64), rect2: (f64, f64, f64, f64)) -> bool {
        let (x1, y1, w1, h1) = rect1;
        let (x2, y2, w2, h2) = rect2;
        
        !(x1 + w1 < x2 || x2 + w2 < x1 || y1 + h1 < y2 || y2 + h2 < y1)
    }
    
    /// Assess whitespace balance (0.0 = all edges, 1.0 = all whitespace)
    fn assess_whitespace(&self, svg_data: &Value) -> f64 {
        let nodes = match svg_data.get("nodes").and_then(|n| n.as_array()) {
            Some(n) => n,
            None => return 0.5,
        };
        
        if nodes.is_empty() {
            return 0.5;
        }
        
        // Calculate total area covered by nodes
        let total_node_area: f64 = nodes
            .iter()
            .filter_map(|node| {
                let width = node.get("width").and_then(|v| v.as_f64()).unwrap_or(100.0);
                let height = node.get("height").and_then(|v| v.as_f64()).unwrap_or(60.0);
                Some(width * height)
            })
            .sum();
        
        // Calculate bounding box area
        let (min_x, min_y, max_x, max_y) = self.calculate_bounding_box(svg_data);
        let total_area = (max_x - min_x) * (max_y - min_y);
        
        if total_area <= 0.0 {
            return 0.5;
        }
        
        // Whitespace ratio
        1.0 - (total_node_area / total_area).min(1.0)
    }
    
    /// Calculate bounding box of all nodes
    fn calculate_bounding_box(&self, svg_data: &Value) -> (f64, f64, f64, f64) {
        let nodes = match svg_data.get("nodes").and_then(|n| n.as_array()) {
            Some(n) => n,
            None => return (0.0, 0.0, 800.0, 600.0),
        };
        
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        
        for node in nodes {
            if let (Some(x), Some(y)) = (
                node.get("x").and_then(|v| v.as_f64()),
                node.get("y").and_then(|v| v.as_f64()),
            ) {
                let width = node.get("width").and_then(|v| v.as_f64()).unwrap_or(100.0);
                let height = node.get("height").and_then(|v| v.as_f64()).unwrap_or(60.0);
                
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x + width);
                max_y = max_y.max(y + height);
            }
        }
        
        (min_x, min_y, max_x, max_y)
    }
    
    /// Score alignment (0.0 = no alignment, 1.0 = perfect alignment)
    fn score_alignment(&self, svg_data: &Value) -> f64 {
        let nodes = match svg_data.get("nodes").and_then(|n| n.as_array()) {
            Some(n) => n,
            None => return 1.0,
        };
        
        if nodes.len() < 2 {
            return 1.0;
        }
        
        // Extract Y coordinates
        let y_coords: Vec<f64> = nodes
            .iter()
            .filter_map(|node| node.get("y").and_then(|v| v.as_f64()))
            .collect();
        
        if y_coords.is_empty() {
            return 1.0;
        }
        
        // Count how many nodes are aligned (within 5px)
        let mut aligned_count = 0;
        let threshold = 5.0;
        
        for i in 0..y_coords.len() {
            for j in (i + 1)..y_coords.len() {
                if (y_coords[i] - y_coords[j]).abs() < threshold {
                    aligned_count += 1;
                }
            }
        }
        
        // Normalize to 0-1 range
        let max_possible_pairs = (y_coords.len() * (y_coords.len() - 1)) / 2;
        if max_possible_pairs == 0 {
            1.0
        } else {
            aligned_count as f64 / max_possible_pairs as f64
        }
    }
    
    /// Check Arcadia compliance
    fn check_arcadia_compliance(&self, svg_data: &Value, semantic: &SemanticContext) -> f64 {
        let rules = self.get_arcadia_rules(&semantic.phase);
        
        if rules.is_empty() {
            return 100.0;
        }
        
        let mut score = 100.0;
        
        for rule in &rules {
            if !(rule.check_fn)(svg_data, semantic) {
                score -= rule.weight;
            }
        }
        
        score.max(0.0)
    }
    
    /// Get Arcadia rules for a specific phase
    fn get_arcadia_rules(&self, phase: &ArcadiaPhase) -> Vec<ArcadiaRule> {
        match phase {
            ArcadiaPhase::Operational => vec![
                ArcadiaRule {
                    name: "actors_at_boundary".to_string(),
                    description: "Actors must be positioned at system boundaries".to_string(),
                    weight: 20.0,
                    check_fn: |_svg, semantic| semantic.has_actors,
                },
            ],
            ArcadiaPhase::Logical => vec![
                ArcadiaRule {
                    name: "interfaces_visible".to_string(),
                    description: "Components must show interface notation".to_string(),
                    weight: 15.0,
                    check_fn: |_svg, semantic| {
                        semantic.elements.iter().any(|e| !e.interfaces_in.is_empty() || !e.interfaces_out.is_empty())
                    },
                },
                ArcadiaRule {
                    name: "safety_borders".to_string(),
                    description: "Safety-critical components must have visual indicators".to_string(),
                    weight: 15.0,
                    check_fn: |_svg, semantic| semantic.has_safety_critical,
                },
            ],
            ArcadiaPhase::Physical => vec![
                ArcadiaRule {
                    name: "nested_deployment".to_string(),
                    description: "Behavior components must nest inside physical nodes".to_string(),
                    weight: 20.0,
                    check_fn: |_svg, semantic| semantic.has_hierarchy,
                },
            ],
            _ => vec![],
        }
    }
    
    /// Calculate overall quality score (0-10 scale)
    fn calculate_overall_score(
        &self,
        edge_crossings: usize,
        node_overlaps: usize,
        whitespace_balance: f64,
        alignment_score: f64,
        arcadia_compliance: f64,
    ) -> f64 {
        // Start with 10.0
        let mut score = 10.0;
        
        // Penalize edge crossings (max -3 points)
        score -= (edge_crossings as f64 * 0.5).min(3.0);
        
        // Penalize node overlaps (max -3 points)
        score -= (node_overlaps as f64 * 1.0).min(3.0);
        
        // Penalize poor whitespace balance (max -1 point)
        let ideal_whitespace = 0.5;
        score -= ((whitespace_balance - ideal_whitespace).abs() * 2.0).min(1.0);
        
        // Penalize poor alignment (max -1 point)
        score -= (1.0 - alignment_score).min(1.0);
        
        // Penalize Arcadia non-compliance (max -2 points)
        score -= ((100.0 - arcadia_compliance) / 50.0).min(2.0);
        
        score.max(0.0)
    }
    
    /// Generate warnings based on metrics
    fn generate_warnings(
        &self,
        edge_crossings: usize,
        node_overlaps: usize,
        alignment_score: f64,
        arcadia_compliance: f64,
    ) -> Vec<String> {
        let mut warnings = Vec::new();
        
        if edge_crossings > 5 {
            warnings.push(format!(
                "High edge crossing count ({}) - consider different layout algorithm",
                edge_crossings
            ));
        }
        
        if node_overlaps > 0 {
            warnings.push(format!(
                "Node overlaps detected ({}) - increase spacing",
                node_overlaps
            ));
        }
        
        if alignment_score < 0.3 {
            warnings.push("Low alignment score - consider grid snap and alignment post-processing".to_string());
        }
        
        if arcadia_compliance < 70.0 {
            warnings.push(format!(
                "Low Arcadia compliance ({:.0}%) - check phase-specific rules",
                arcadia_compliance
            ));
        }
        
        warnings
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::compiler::semantic_analyzer::{
        ComplexityMetrics, ElementClassification, ElementType, ElementStereotype,
        RelationshipAnalysis, RecommendedStrategy,
    };
    
    fn create_test_semantic() -> SemanticContext {
        SemanticContext {
            phase: ArcadiaPhase::Logical,
            diagram_type: "component".to_string(),
            elements: vec![],
            relationships: RelationshipAnalysis {
                containment: vec![],
                connections: vec![],
                allocations: vec![],
                traces: vec![],
            },
            complexity: ComplexityMetrics {
                total_elements: 3,
                depth: 1,
                branching_factor: 1.5,
                has_cycles: false,
            },
            recommended_strategy: RecommendedStrategy::Hierarchy,
            has_actors: false,
            has_hierarchy: true,
            has_data_flow: true,
            has_safety_critical: false,
        }
    }
    
    #[test]
    fn test_segments_intersect() {
        let metrics = QualityMetrics::new();
        
        // Crossing segments
        assert!(metrics.segments_intersect((0.0, 0.0, 10.0, 10.0), (0.0, 10.0, 10.0, 0.0)));
        
        // Non-crossing segments
        assert!(!metrics.segments_intersect((0.0, 0.0, 10.0, 0.0), (0.0, 10.0, 10.0, 10.0)));
    }
    
    #[test]
    fn test_rectangles_overlap() {
        let metrics = QualityMetrics::new();
        
        // Overlapping rectangles
        assert!(metrics.rectangles_overlap((0.0, 0.0, 100.0, 60.0), (50.0, 30.0, 100.0, 60.0)));
        
        // Non-overlapping rectangles
        assert!(!metrics.rectangles_overlap((0.0, 0.0, 100.0, 60.0), (200.0, 200.0, 100.0, 60.0)));
    }
    
    #[test]
    fn test_count_edge_crossings() {
        let metrics = QualityMetrics::new();
        
        let svg_data = json!({
            "edges": [
                {"x1": 0.0, "y1": 0.0, "x2": 10.0, "y2": 10.0},
                {"x1": 0.0, "y1": 10.0, "x2": 10.0, "y2": 0.0}
            ]
        });
        
        assert_eq!(metrics.count_edge_crossings(&svg_data), 1);
    }
    
    #[test]
    fn test_detect_node_overlaps() {
        let metrics = QualityMetrics::new();
        
        let svg_data = json!({
            "nodes": [
                {"x": 0.0, "y": 0.0, "width": 100.0, "height": 60.0},
                {"x": 50.0, "y": 30.0, "width": 100.0, "height": 60.0}
            ]
        });
        
        assert_eq!(metrics.detect_node_overlaps(&svg_data), 1);
    }
    
    #[test]
    fn test_calculate_overall_score() {
        let metrics = QualityMetrics::new();
        
        let semantic = create_test_semantic();
        let svg_data = json!({
            "nodes": [
                {"x": 0.0, "y": 100.0, "width": 100.0, "height": 60.0},
                {"x": 200.0, "y": 100.0, "width": 100.0, "height": 60.0}
            ]
        });
        
        let report = metrics.calculate(&svg_data, &semantic);
        
        assert!(report.overall_score >= 0.0);
        assert!(report.overall_score <= 10.0);
        assert_eq!(report.edge_crossings, 0);
        assert_eq!(report.node_overlaps, 0);
    }
}
