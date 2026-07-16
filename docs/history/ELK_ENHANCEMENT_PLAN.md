# ELK Enhancement Plan for ArcLang Compiler

**Date:** October 28, 2025  
**Goal:** Perfect Capella-quality diagrams using Eclipse Layout Kernel (ELK)  
**Focus:** Improve diagram rendering, not add new features  

---

## Current State Analysis

### What You Already Have

From `/Users/malek/Arclang/src/compiler/`:
- ✅ `arcviz_elk.rs` (987 lines) - ELK integration exists
- ✅ `arcviz_ultimate_routing.rs` (515 lines) - Custom routing
- ✅ `arcviz_smart_routing.rs` (710 lines) - Smart routing
- ✅ `arcviz_perfect_routing.rs` (544 lines) - Perfect routing
- ✅ `arcviz_capella_routing.rs` (405 lines) - Capella-style routing
- ✅ Multiple layout engines already implemented

### Current Gaps (From Capellambse Study)

**Missing from your ELK implementation:**
1. ❌ **Port constraints** - Native port positioning (FIXED_SIDE)
2. ❌ **Manhattan routing** - True orthogonal edges (90° only)
3. ❌ **Hierarchical ports** - Ports on swimlane boundaries
4. ❌ **Layer hierarchy** - Proper nested swimlanes
5. ❌ **Smart port distribution** - Even spacing, collision avoidance
6. ❌ **Edge label positioning** - Centered on edge paths

---

## Learnings from Capellambse Diagram System

### Key Algorithms to Port

#### 1. Vector Snapping System

**Capellambse Pattern:**
```python
# Three routing strategies
class RoutingStyle(enum.Enum):
    OBLIQUE = enum.auto()      # Direct diagonal
    MANHATTAN = enum.auto()    # Orthogonal 90°
    TREE = enum.auto()         # Hierarchical top-down

def vector_snap(point, source, style):
    if style == MANHATTAN:
        # Snap to box edge maintaining one axis
        axis = (point - source).closest_axis()
        if axis.x:
            return Vector2D(box.edge_x, point.y)
        else:
            return Vector2D(point.x, box.edge_y)
```

**Your Rust Implementation:**
```rust
// In src/compiler/arcviz_elk.rs

#[derive(Debug, Clone, Copy)]
pub enum RoutingStyle {
    Oblique,    // Direct line
    Manhattan,  // Orthogonal 90°
    Tree,       // Hierarchical
}

impl ComponentBox {
    /// Snap point to box edge based on routing style
    pub fn vector_snap(
        &self,
        point: Vector2D,
        source: Vector2D,
        style: RoutingStyle,
    ) -> Vector2D {
        match style {
            RoutingStyle::Oblique => self.snap_oblique(point, source),
            RoutingStyle::Manhattan => self.snap_manhattan(point, source),
            RoutingStyle::Tree => self.snap_tree(point, source),
        }
    }
    
    fn snap_manhattan(&self, point: Vector2D, source: Vector2D) -> Vector2D {
        let direction = point - source;
        let axis = direction.closest_axis();
        
        if axis.x.abs() > axis.y.abs() {
            // Horizontal primary
            if point.y < self.pos.y {
                // Top edge
                Vector2D::new(self.pos.x + self.size.x / 2.0, self.pos.y)
            } else if point.y > self.pos.y + self.size.y {
                // Bottom edge
                Vector2D::new(self.pos.x + self.size.x / 2.0, self.pos.y + self.size.y)
            } else {
                // Left or right edge
                if source.x < self.pos.x {
                    Vector2D::new(self.pos.x, point.y)
                } else {
                    Vector2D::new(self.pos.x + self.size.x, point.y)
                }
            }
        } else {
            // Vertical primary
            if point.x < self.pos.x {
                // Left edge
                Vector2D::new(self.pos.x, self.pos.y + self.size.y / 2.0)
            } else if point.x > self.pos.x + self.size.x {
                // Right edge
                Vector2D::new(self.pos.x + self.size.x, self.pos.y + self.size.y / 2.0)
            } else {
                // Top or bottom edge
                if source.y < self.pos.y {
                    Vector2D::new(point.x, self.pos.y)
                } else {
                    Vector2D::new(point.x, self.pos.y + self.size.y)
                }
            }
        }
    }
}

impl Vector2D {
    /// Return axis-aligned vector in primary direction
    pub fn closest_axis(&self) -> Vector2D {
        if self.x.abs() >= self.y.abs() {
            Vector2D::new(self.x.signum(), 0.0)
        } else {
            Vector2D::new(0.0, self.y.signum())
        }
    }
}
```

#### 2. Port Positioning with ELK

**Capellambse Pattern:**
```python
# ELK port configuration
elkNode.ports.push({
    id: f"{node.id}_in_{idx}",
    properties: {
        'port.side': 'WEST',      # Fixed to left side
        'port.index': idx,         # Ordering
    },
    labels: [{ text: port.name }]
})
```

**Your Rust Implementation:**
```rust
// In src/compiler/arcviz_elk.rs

pub struct ELKPort {
    pub id: String,
    pub side: PortSide,
    pub index: usize,
    pub label: String,
    pub size: Vector2D,
}

#[derive(Debug, Clone, Copy)]
pub enum PortSide {
    North,
    South,
    East,
    West,
}

impl PortSide {
    pub fn to_elk_string(&self) -> &'static str {
        match self {
            PortSide::North => "NORTH",
            PortSide::South => "SOUTH",
            PortSide::East => "EAST",
            PortSide::West => "WEST",
        }
    }
}

pub fn generate_elk_graph_with_ports(
    components: &[LogicalComponent],
    edges: &[ComponentExchange],
) -> serde_json::Value {
    let mut elk_nodes = Vec::new();
    
    for comp in components {
        let mut ports = Vec::new();
        
        // IN ports on left (WEST)
        for (idx, port_in) in comp.interfaces_in.iter().enumerate() {
            ports.push(json!({
                "id": format!("{}_{}_in", comp.id, idx),
                "properties": {
                    "port.side": "WEST",
                    "port.index": idx,
                },
                "width": 10,
                "height": 10,
                "labels": [{"text": port_in}]
            }));
        }
        
        // OUT ports on right (EAST)
        for (idx, port_out) in comp.interfaces_out.iter().enumerate() {
            ports.push(json!({
                "id": format!("{}_{}_out", comp.id, idx),
                "properties": {
                    "port.side": "EAST",
                    "port.index": idx,
                },
                "width": 10,
                "height": 10,
                "labels": [{"text": port_out}]
            }));
        }
        
        elk_nodes.push(json!({
            "id": comp.id,
            "width": 200,
            "height": 100,
            "labels": [{"text": comp.name}],
            "ports": ports,
        }));
    }
    
    json!({
        "id": "root",
        "layoutOptions": {
            "elk.algorithm": "layered",
            "elk.direction": "RIGHT",
            "elk.portConstraints": "FIXED_SIDE",
            "elk.edgeRouting": "ORTHOGONAL",
        },
        "children": elk_nodes,
        "edges": generate_elk_edges(edges),
    })
}
```

#### 3. Hierarchical Swimlanes

**Capellambse Pattern:**
```python
# Nested containers for layers
elk_graph = {
    "children": [
        {
            "id": "oa_layer",
            "layoutOptions": {
                "elk.padding": "[top=50,left=30,bottom=30,right=30]",
            },
            "children": [
                {"id": "actor1", ...},
                {"id": "actor2", ...},
            ]
        },
        {
            "id": "sa_layer",
            "children": [...],
        }
    ]
}
```

**Your Rust Implementation:**
```rust
// In src/compiler/arcviz_elk.rs

pub struct LayerContainer {
    pub name: String,
    pub color: String,
    pub padding: Padding,
    pub components: Vec<LogicalComponent>,
}

pub struct Padding {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Padding {
    pub fn to_elk_string(&self) -> String {
        format!(
            "[top={},right={},bottom={},left={}]",
            self.top, self.right, self.bottom, self.left
        )
    }
}

pub fn generate_layered_elk_graph(
    layers: &[LayerContainer],
    edges: &[ComponentExchange],
) -> serde_json::Value {
    let mut layer_nodes = Vec::new();
    
    for layer in layers {
        let mut layer_components = Vec::new();
        
        for comp in &layer.components {
            layer_components.push(json!({
                "id": comp.id,
                "width": 200,
                "height": 100,
                "labels": [{"text": comp.name}],
            }));
        }
        
        layer_nodes.push(json!({
            "id": format!("layer_{}", layer.name),
            "layoutOptions": {
                "elk.padding": layer.padding.to_elk_string(),
                "elk.portConstraints": "FREE",
            },
            "labels": [{"text": format!("{} Layer", layer.name)}],
            "children": layer_components,
        }));
    }
    
    json!({
        "id": "root",
        "layoutOptions": {
            "elk.algorithm": "layered",
            "elk.direction": "DOWN",
            "elk.spacing.nodeNode": "80",
            "elk.layered.spacing.nodeNodeBetweenLayers": "100",
        },
        "children": layer_nodes,
        "edges": generate_elk_edges(edges),
    })
}
```

---

## Priority Enhancements

### Priority 1: Perfect Port Positioning (Week 1)

**Goal:** Match Capella's port layout exactly

**Current Problem:**
Your `arcviz_elk.rs` likely doesn't use ELK's native port system properly.

**Solution:**
```rust
// Update src/compiler/arcviz_elk.rs

pub struct ELKLayoutEngine {
    pub config: ELKConfig,
}

#[derive(Debug, Clone)]
pub struct ELKConfig {
    pub algorithm: String,          // "layered"
    pub direction: String,          // "RIGHT", "DOWN"
    pub port_constraints: String,   // "FIXED_SIDE", "FIXED_ORDER"
    pub edge_routing: String,       // "ORTHOGONAL", "SPLINES"
    pub node_spacing: f64,          // 80.0
    pub layer_spacing: f64,         // 100.0
    pub port_spacing: f64,          // 20.0
}

impl Default for ELKConfig {
    fn default() -> Self {
        Self {
            algorithm: "layered".to_string(),
            direction: "RIGHT".to_string(),
            port_constraints: "FIXED_SIDE".to_string(),
            edge_routing: "ORTHOGONAL".to_string(),
            node_spacing: 80.0,
            layer_spacing: 100.0,
            port_spacing: 20.0,
        }
    }
}

impl ELKLayoutEngine {
    pub fn layout(&self, model: &SemanticModel) -> Result<ELKGraph, String> {
        // Build ELK graph with proper port configuration
        let mut elk_graph = json!({
            "id": "root",
            "layoutOptions": {
                "elk.algorithm": self.config.algorithm,
                "elk.direction": self.config.direction,
                "elk.portConstraints": self.config.port_constraints,
                "elk.edgeRouting": self.config.edge_routing,
                "elk.spacing.nodeNode": self.config.node_spacing.to_string(),
                "elk.layered.spacing.nodeNodeBetweenLayers": self.config.layer_spacing.to_string(),
            },
            "children": [],
            "edges": [],
        });
        
        // Add components with ports
        for la in &model.logical_architecture {
            for comp in &la.components {
                let elk_node = self.component_to_elk_node(comp)?;
                elk_graph["children"].as_array_mut().unwrap().push(elk_node);
            }
        }
        
        Ok(serde_json::from_value(elk_graph)?)
    }
    
    fn component_to_elk_node(&self, comp: &LogicalComponent) -> Result<serde_json::Value, String> {
        let mut ports = Vec::new();
        
        // Input ports (left side)
        for (idx, interface) in comp.interfaces_in.iter().enumerate() {
            ports.push(json!({
                "id": format!("{}_port_in_{}", comp.id, idx),
                "properties": {
                    "port.side": "WEST",
                    "port.index": idx,
                    "port.borderOffset": -5.0,  // Slight overhang
                },
                "width": 10,
                "height": 10,
                "labels": [{
                    "text": interface,
                    "layoutOptions": {
                        "elk.label.side": "WEST",
                    }
                }]
            }));
        }
        
        // Output ports (right side)
        for (idx, interface) in comp.interfaces_out.iter().enumerate() {
            ports.push(json!({
                "id": format!("{}_port_out_{}", comp.id, idx),
                "properties": {
                    "port.side": "EAST",
                    "port.index": idx,
                    "port.borderOffset": -5.0,
                },
                "width": 10,
                "height": 10,
                "labels": [{
                    "text": interface,
                    "layoutOptions": {
                        "elk.label.side": "EAST",
                    }
                }]
            }));
        }
        
        Ok(json!({
            "id": comp.id,
            "width": 220,
            "height": 180,
            "labels": [{"text": comp.name}],
            "ports": ports,
            "layoutOptions": {
                "elk.portAlignment.default": "CENTER",
            }
        }))
    }
}
```

### Priority 2: True Orthogonal Routing (Week 2)

**Goal:** Pure 90° angles, no diagonal lines

**Current Problem:**
Your routing algorithms may create diagonal segments.

**Solution:**
```rust
// In src/compiler/arcviz_elk.rs

pub fn enforce_orthogonal_edges(elk_result: &mut ELKGraph) {
    for edge in &mut elk_result.edges {
        if let Some(sections) = &mut edge.sections {
            for section in sections {
                // Ensure each segment is perfectly horizontal or vertical
                let mut points = section.points.clone();
                
                for i in 1..points.len() {
                    let prev = points[i - 1];
                    let curr = points[i];
                    
                    // Force to axis-aligned
                    if (curr.x - prev.x).abs() > (curr.y - prev.y).abs() {
                        // Horizontal segment
                        points[i].y = prev.y;
                    } else {
                        // Vertical segment
                        points[i].x = prev.x;
                    }
                }
                
                section.points = points;
            }
        }
    }
}
```

### Priority 3: Dynamic Component Sizing (Week 3)

**Goal:** Auto-size components based on content

**Capellambse Pattern:**
```python
@property
def size(self) -> Vector2D:
    if self._size.x <= 0 or self._size.y <= 0:
        # Calculate from label + children
        label_extent = get_text_extent(self.label)
        width = max(self.minsize.x, label_extent.x + padding)
        
        for child in self.children:
            width = max(width, child.bounds.width)
        
        return Vector2D(width, height)
    return self._size
```

**Your Rust Implementation:**
```rust
// In src/compiler/arcviz_elk.rs

pub fn calculate_component_size(
    comp: &LogicalComponent,
    font_size: f64,
) -> (f64, f64) {
    let base_padding = 20.0;
    let function_height = 25.0;
    let port_spacing = 15.0;
    
    // Calculate width from label
    let label_width = estimate_text_width(&comp.name, font_size);
    let mut width = (label_width + base_padding * 2.0).max(180.0);
    
    // Add width for functions if present
    if !comp.functions.is_empty() {
        let max_func_width = comp.functions.iter()
            .map(|f| estimate_text_width(&f.name, font_size - 2.0))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        width = width.max(max_func_width + base_padding * 2.0);
    }
    
    // Calculate height from functions
    let function_count = comp.functions.len();
    let base_height = 60.0;  // Header + padding
    let functions_height = function_count as f64 * function_height;
    let mut height = base_height + functions_height;
    
    // Ensure enough height for ports
    let max_ports = comp.interfaces_in.len().max(comp.interfaces_out.len());
    let min_height_for_ports = (max_ports as f64 * port_spacing) + base_padding * 2.0;
    height = height.max(min_height_for_ports).max(100.0);
    
    (width, height)
}

fn estimate_text_width(text: &str, font_size: f64) -> f64 {
    // Rough estimate: 0.6 * font_size per character
    text.len() as f64 * font_size * 0.6
}
```

### Priority 4: Swimlane Visualization (Week 4)

**Goal:** Visual layer separation like Capella

**Current Problem:**
Layers might not have visual swimlane backgrounds.

**Solution:**
```rust
// In src/compiler/arcviz_generator.rs or arcviz_elk.rs

pub struct SwimlaneConfig {
    pub show_lanes: bool,
    pub lane_colors: HashMap<String, String>,
    pub lane_opacity: f64,
    pub lane_border: bool,
}

impl Default for SwimlaneConfig {
    fn default() -> Self {
        let mut colors = HashMap::new();
        colors.insert("OA".to_string(), "#E8F5E9".to_string());  // Light green
        colors.insert("SA".to_string(), "#E3F2FD".to_string());  // Light blue
        colors.insert("LA".to_string(), "#FFF3E0".to_string());  // Light orange
        colors.insert("PA".to_string(), "#F3E5F5".to_string());  // Light purple
        
        Self {
            show_lanes: true,
            lane_colors: colors,
            lane_opacity: 0.25,
            lane_border: true,
        }
    }
}

pub fn render_swimlanes_svg(
    layers: &[LayerContainer],
    svg: &mut String,
    config: &SwimlaneConfig,
) {
    for layer in layers {
        if !config.show_lanes {
            continue;
        }
        
        let color = config.lane_colors.get(&layer.name)
            .unwrap_or(&"#E0E0E0".to_string());
        
        svg.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" 
               fill="{}" fill-opacity="{}" 
               stroke="{}" stroke-width="{}" 
               stroke-dasharray="{}" rx="8"/>"#,
            layer.bounds.x,
            layer.bounds.y,
            layer.bounds.width,
            layer.bounds.height,
            color,
            config.lane_opacity,
            if config.lane_border { "#37474f" } else { "none" },
            if config.lane_border { 2 } else { 0 },
            "10,5",  // Dashed border
        ));
        
        // Layer label
        svg.push_str(&format!(
            r#"<text x="{}" y="{}" font-size="18" font-weight="bold" fill="#37474f" text-anchor="middle">{} Layer</text>"#,
            layer.bounds.x + layer.bounds.width / 2.0,
            layer.bounds.y + 30.0,
            layer.name,
        ));
    }
}
```

---

## Configuration Optimization

### Recommended ELK Settings for Capella Style

```rust
// In src/compiler/arcviz_elk.rs

pub fn get_capella_elk_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    
    // Core algorithm
    config.insert("elk.algorithm".to_string(), "layered".to_string());
    config.insert("elk.direction".to_string(), "RIGHT".to_string());
    
    // Port handling
    config.insert("elk.portConstraints".to_string(), "FIXED_SIDE".to_string());
    config.insert("elk.portAlignment.default".to_string(), "CENTER".to_string());
    
    // Edge routing
    config.insert("elk.edgeRouting".to_string(), "ORTHOGONAL".to_string());
    config.insert("elk.layered.edgeRouting.sloppySplineRouting".to_string(), "false".to_string());
    
    // Spacing
    config.insert("elk.spacing.nodeNode".to_string(), "80".to_string());
    config.insert("elk.spacing.edgeNode".to_string(), "40".to_string());
    config.insert("elk.spacing.edgeEdge".to_string(), "20".to_string());
    config.insert("elk.layered.spacing.nodeNodeBetweenLayers".to_string(), "100".to_string());
    
    // Node placement
    config.insert("elk.layered.nodePlacement.strategy".to_string(), "NETWORK_SIMPLEX".to_string());
    config.insert("elk.layered.layering.strategy".to_string(), "NETWORK_SIMPLEX".to_string());
    
    // Compaction
    config.insert("elk.layered.compaction.postCompaction.strategy".to_string(), "EDGE_LENGTH".to_string());
    
    // Hierarchy
    config.insert("elk.hierarchyHandling".to_string(), "INCLUDE_CHILDREN".to_string());
    
    config
}
```

---

## Testing & Validation

### Test Cases

Create test examples to verify improvements:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_manhattan_routing() {
        let box1 = ComponentBox::new(Vector2D::new(0.0, 0.0), Vector2D::new(100.0, 100.0));
        let box2 = ComponentBox::new(Vector2D::new(200.0, 150.0), Vector2D::new(100.0, 100.0));
        
        let snap_point = box1.vector_snap(
            box2.center(),
            box1.center(),
            RoutingStyle::Manhattan,
        );
        
        // Should snap to right edge, maintaining Y coordinate
        assert_eq!(snap_point.x, 100.0);
        assert!((snap_point.y - 50.0).abs() < 0.1);
    }
    
    #[test]
    fn test_port_positioning() {
        let comp = create_test_component_with_ports();
        let elk_node = component_to_elk_with_ports(&comp);
        
        // Verify ports are on correct sides
        let west_ports = elk_node.ports.iter()
            .filter(|p| p.side == PortSide::West)
            .count();
        let east_ports = elk_node.ports.iter()
            .filter(|p| p.side == PortSide::East)
            .count();
        
        assert_eq!(west_ports, comp.interfaces_in.len());
        assert_eq!(east_ports, comp.interfaces_out.len());
    }
}
```

---

## Next Steps

### Immediate Actions

1. **Read your current `arcviz_elk.rs`** to understand existing implementation
2. **Identify gaps** compared to Capellambse patterns
3. **Implement Priority 1** (Port positioning) first
4. **Test with ACC example** to see improvements
5. **Iterate** on other priorities

### Files to Focus On

Primary files to enhance:
- `src/compiler/arcviz_elk.rs` - Main ELK integration
- `src/compiler/arcviz_generator.rs` - SVG generation
- `src/compiler/semantic.rs` - Add port/layer metadata if needed

### Success Criteria

✅ Ports align perfectly on left/right edges  
✅ All edges are pure orthogonal (90° only)  
✅ Components auto-size to fit content  
✅ Swimlanes visible with layer colors  
✅ Matches Capella visual quality  

---

## Summary

**Focus:** Improve diagram quality using ELK properly  
**Not adding:** New features, just better diagrams  
**Timeline:** 4 weeks for all priorities  
**Outcome:** Production-ready Capella-quality diagrams  

All patterns extracted from Capellambse are included above. Start with Priority 1 (ports) for immediate visual improvement.
