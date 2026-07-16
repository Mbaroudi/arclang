//! Post-Processing Pipeline for SVG Diagram Enhancement
//! 
//! This module provides post-layout refinements to improve visual quality:
//! - Grid snapping for precise alignment
//! - Element alignment (horizontal/vertical)
//! - Spacing distribution for even gaps
//! - Label optimization to avoid overlaps

use serde_json::{json, Value};
use std::collections::HashMap;

/// Post-processing configuration
#[derive(Debug, Clone)]
pub struct PostProcessConfig {
    pub grid_size: f64,
    pub enable_grid_snap: bool,
    pub enable_alignment: bool,
    pub enable_spacing: bool,
    pub enable_label_optimization: bool,
    pub alignment_threshold: f64,
    pub target_gap: f64,
}

impl Default for PostProcessConfig {
    fn default() -> Self {
        PostProcessConfig {
            grid_size: 10.0,
            enable_grid_snap: true,
            enable_alignment: true,
            enable_spacing: true,
            enable_label_optimization: true,
            alignment_threshold: 20.0,
            target_gap: 60.0,
        }
    }
}

/// Element position and bounds
#[derive(Debug, Clone)]
struct ElementBounds {
    id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Post-processor for SVG refinement
pub struct PostProcessor {
    config: PostProcessConfig,
}

impl PostProcessor {
    pub fn new(config: PostProcessConfig) -> Self {
        PostProcessor { config }
    }
    
    /// Main entry point: Process SVG data
    pub fn process(&self, svg_data: Value) -> Value {
        let mut result = svg_data;
        
        // Step 1: Grid Snapping (align to grid)
        if self.config.enable_grid_snap {
            result = self.snap_to_grid(result);
        }
        
        // Step 2: Element Alignment (horizontal/vertical)
        if self.config.enable_alignment {
            result = self.align_elements(result);
        }
        
        // Step 3: Spacing Distribution
        if self.config.enable_spacing {
            result = self.distribute_spacing(result);
        }
        
        // Step 4: Label Optimization (avoid overlaps)
        if self.config.enable_label_optimization {
            result = self.optimize_labels(result);
        }
        
        result
    }
    
    /// Snap all elements to grid
    fn snap_to_grid(&self, mut svg_data: Value) -> Value {
        if let Some(nodes) = svg_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
            for node in nodes {
                if let Some(x) = node.get("x").and_then(|v| v.as_f64()) {
                    let snapped_x = self.snap_value(x);
                    node["x"] = json!(snapped_x);
                }
                
                if let Some(y) = node.get("y").and_then(|v| v.as_f64()) {
                    let snapped_y = self.snap_value(y);
                    node["y"] = json!(snapped_y);
                }
            }
        }
        
        svg_data
    }
    
    /// Snap a single value to grid
    fn snap_value(&self, value: f64) -> f64 {
        (value / self.config.grid_size).round() * self.config.grid_size
    }
    
    /// Align elements that are approximately on the same line
    fn align_elements(&self, mut svg_data: Value) -> Value {
        if let Some(nodes) = svg_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
            // Extract element bounds
            let mut elements: Vec<ElementBounds> = Vec::new();
            
            for node in nodes.iter() {
                if let (Some(id), Some(x), Some(y)) = (
                    node.get("id").and_then(|v| v.as_str()),
                    node.get("x").and_then(|v| v.as_f64()),
                    node.get("y").and_then(|v| v.as_f64()),
                ) {
                    let width = node.get("width").and_then(|v| v.as_f64()).unwrap_or(100.0);
                    let height = node.get("height").and_then(|v| v.as_f64()).unwrap_or(60.0);
                    
                    elements.push(ElementBounds {
                        id: id.to_string(),
                        x,
                        y,
                        width,
                        height,
                    });
                }
            }
            
            // Group by approximate Y coordinate
            let y_groups = self.group_by_y(&elements);
            
            // Align each group to average Y
            let mut y_adjustments: HashMap<String, f64> = HashMap::new();
            for group in y_groups {
                if group.len() > 1 {
                    let avg_y: f64 = group.iter().map(|e| e.y).sum::<f64>() / group.len() as f64;
                    for elem in group {
                        y_adjustments.insert(elem.id.clone(), avg_y);
                    }
                }
            }
            
            // Apply adjustments
            for node in nodes.iter_mut() {
                if let Some(id) = node.get("id").and_then(|v| v.as_str()) {
                    if let Some(&new_y) = y_adjustments.get(id) {
                        node["y"] = json!(new_y);
                    }
                }
            }
        }
        
        svg_data
    }
    
    /// Group elements by approximate Y coordinate
    fn group_by_y<'a>(&self, elements: &'a [ElementBounds]) -> Vec<Vec<&'a ElementBounds>> {
        let mut groups: Vec<Vec<&'a ElementBounds>> = Vec::new();
        
        for elem in elements {
            let mut found_group = false;
            
            for group in &mut groups {
                if let Some(first) = group.first() {
                    if (first.y - elem.y).abs() < self.config.alignment_threshold {
                        group.push(elem);
                        found_group = true;
                        break;
                    }
                }
            }
            
            if !found_group {
                groups.push(vec![elem]);
            }
        }
        
        groups
    }
    
    /// Distribute spacing evenly between elements
    fn distribute_spacing(&self, mut svg_data: Value) -> Value {
        if let Some(nodes) = svg_data.get_mut("nodes").and_then(|n| n.as_array_mut()) {
            // Extract horizontal groups (same Y)
            let mut elements: Vec<ElementBounds> = Vec::new();
            
            for node in nodes.iter() {
                if let (Some(id), Some(x), Some(y)) = (
                    node.get("id").and_then(|v| v.as_str()),
                    node.get("x").and_then(|v| v.as_f64()),
                    node.get("y").and_then(|v| v.as_f64()),
                ) {
                    let width = node.get("width").and_then(|v| v.as_f64()).unwrap_or(100.0);
                    let height = node.get("height").and_then(|v| v.as_f64()).unwrap_or(60.0);
                    
                    elements.push(ElementBounds {
                        id: id.to_string(),
                        x,
                        y,
                        width,
                        height,
                    });
                }
            }
            
            // Group by Y coordinate
            let y_groups = self.group_by_y(&elements);
            
            // For each group, ensure minimum spacing
            let mut x_adjustments: HashMap<String, f64> = HashMap::new();
            
            for group in y_groups {
                if group.len() > 1 {
                    let mut sorted_group = group.clone();
                    sorted_group.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
                    
                    // Check gaps and adjust if needed
                    for i in 1..sorted_group.len() {
                        let prev = sorted_group[i - 1];
                        let curr = sorted_group[i];
                        let gap = curr.x - (prev.x + prev.width);
                        
                        if gap < self.config.target_gap * 0.8 {
                            let new_x = prev.x + prev.width + self.config.target_gap;
                            x_adjustments.insert(curr.id.clone(), new_x);
                        }
                    }
                }
            }
            
            // Apply adjustments
            for node in nodes.iter_mut() {
                if let Some(id) = node.get("id").and_then(|v| v.as_str()) {
                    if let Some(&new_x) = x_adjustments.get(id) {
                        node["x"] = json!(new_x);
                    }
                }
            }
        }
        
        svg_data
    }
    
    /// Optimize label positions to avoid overlaps
    fn optimize_labels(&self, mut svg_data: Value) -> Value {
        // Extract labels and their bounds
        if let Some(edges) = svg_data.get_mut("edges").and_then(|e| e.as_array_mut()) {
            let mut label_positions: Vec<(String, f64, f64)> = Vec::new();
            
            for edge in edges.iter() {
                if let (Some(id), Some(label_x), Some(label_y)) = (
                    edge.get("id").and_then(|v| v.as_str()),
                    edge.get("labelX").and_then(|v| v.as_f64()),
                    edge.get("labelY").and_then(|v| v.as_f64()),
                ) {
                    label_positions.push((id.to_string(), label_x, label_y));
                }
            }
            
            // Detect overlaps and adjust
            let mut adjustments: HashMap<String, (f64, f64)> = HashMap::new();
            
            for i in 0..label_positions.len() {
                for j in (i + 1)..label_positions.len() {
                    let (id1, x1, y1) = &label_positions[i];
                    let (id2, x2, y2) = &label_positions[j];
                    
                    // Check if labels overlap (simple distance check)
                    let distance = ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt();
                    
                    if distance < 40.0 {
                        // Move second label slightly down
                        adjustments.insert(id2.clone(), (*x2, *y2 + 20.0));
                    }
                }
            }
            
            // Apply adjustments
            for edge in edges.iter_mut() {
                if let Some(id) = edge.get("id").and_then(|v| v.as_str()) {
                    if let Some(&(new_x, new_y)) = adjustments.get(id) {
                        edge["labelX"] = json!(new_x);
                        edge["labelY"] = json!(new_y);
                    }
                }
            }
        }
        
        svg_data
    }
}

impl Default for PostProcessor {
    fn default() -> Self {
        PostProcessor::new(PostProcessConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_snap_value() {
        let config = PostProcessConfig {
            grid_size: 10.0,
            ..Default::default()
        };
        let processor = PostProcessor::new(config);
        
        assert_eq!(processor.snap_value(23.0), 20.0);
        assert_eq!(processor.snap_value(27.0), 30.0);
        assert_eq!(processor.snap_value(25.0), 30.0);
    }
    
    #[test]
    fn test_snap_to_grid() {
        let config = PostProcessConfig {
            grid_size: 10.0,
            ..Default::default()
        };
        let processor = PostProcessor::new(config);
        
        let input = json!({
            "nodes": [
                {"id": "n1", "x": 23.0, "y": 47.0},
                {"id": "n2", "x": 156.0, "y": 42.0}
            ]
        });
        
        let output = processor.snap_to_grid(input);
        
        assert_eq!(output["nodes"][0]["x"], json!(20.0));
        assert_eq!(output["nodes"][0]["y"], json!(50.0));
        assert_eq!(output["nodes"][1]["x"], json!(160.0));
        assert_eq!(output["nodes"][1]["y"], json!(40.0));
    }
    
    #[test]
    fn test_group_by_y() {
        let config = PostProcessConfig {
            alignment_threshold: 20.0,
            ..Default::default()
        };
        let processor = PostProcessor::new(config);
        
        let elements = vec![
            ElementBounds {
                id: "e1".to_string(),
                x: 0.0,
                y: 100.0,
                width: 100.0,
                height: 60.0,
            },
            ElementBounds {
                id: "e2".to_string(),
                x: 200.0,
                y: 105.0,
                width: 100.0,
                height: 60.0,
            },
            ElementBounds {
                id: "e3".to_string(),
                x: 0.0,
                y: 300.0,
                width: 100.0,
                height: 60.0,
            },
        ];
        
        let groups = processor.group_by_y(&elements);
        
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].len(), 2); // e1 and e2 grouped
        assert_eq!(groups[1].len(), 1); // e3 alone
    }
    
    #[test]
    fn test_post_processor_default() {
        let processor = PostProcessor::default();
        
        let input = json!({
            "nodes": [
                {"id": "n1", "x": 23.0, "y": 47.0, "width": 100.0, "height": 60.0}
            ]
        });
        
        let output = processor.process(input);
        
        // Should have grid-snapped
        assert_eq!(output["nodes"][0]["x"], json!(20.0));
        assert_eq!(output["nodes"][0]["y"], json!(50.0));
    }
    
    #[test]
    fn test_align_elements() {
        let config = PostProcessConfig {
            alignment_threshold: 20.0,
            enable_grid_snap: false,
            enable_alignment: true,
            enable_spacing: false,
            enable_label_optimization: false,
            ..Default::default()
        };
        let processor = PostProcessor::new(config);
        
        let input = json!({
            "nodes": [
                {"id": "n1", "x": 0.0, "y": 100.0, "width": 100.0, "height": 60.0},
                {"id": "n2", "x": 200.0, "y": 110.0, "width": 100.0, "height": 60.0}
            ]
        });
        
        let output = processor.process(input);
        
        // Both nodes should be aligned to average Y (105.0)
        let y1 = output["nodes"][0]["y"].as_f64().unwrap();
        let y2 = output["nodes"][1]["y"].as_f64().unwrap();
        assert_eq!(y1, y2);
        assert_eq!(y1, 105.0);
    }
}
