# True ELK Nested Hierarchical Layouts for ArcLang

**Problem:** Your current `arcviz_elk.rs` is a custom Rust implementation, not real ELK  
**Solution:** Integrate actual ELK.js for true nested hierarchical layouts with mixed directions  
**Benefit:** Support complex Capella-style diagrams with nested containers and different layout directions  

---

## Why You Need True ELK

### Your Current Approach (Custom Rust)
```rust
// src/compiler/arcviz_elk.rs (current)
pub struct ElkLayout {
    pub layers: Vec<LayoutedLayer>,  // Flat layer structure
    pub connections: Vec<LayoutedConnection>,
}

// Problem: Can't handle:
// - Nested components inside components
// - Different layout directions per container
// - Mixed algorithms (layered inside one, stress inside another)
```

### What True ELK Enables
```javascript
// Real ELK hierarchical graph
{
  id: "root",
  layoutOptions: { 'elk.algorithm': 'layered', 'elk.direction': 'RIGHT' },
  children: [
    {
      id: "container1",
      layoutOptions: { 'elk.direction': 'DOWN' },  // Different direction!
      children: [
        { id: "n1" },
        { id: "n2" }
      ]
    },
    {
      id: "container2",
      layoutOptions: { 'elk.algorithm': 'stress' },  // Different algorithm!
      children: [...]
    }
  ]
}
```

---

## Architecture: Rust + ELK.js Integration

### Option 1: Generate ELK JSON in Rust, Layout in Browser (Recommended)

**Flow:**
```
Arclang Rust Compiler
    ↓ Generate ELK JSON
Browser HTML/JavaScript
    ↓ ELK.js layout()
    ↓ D3.js render()
SVG Diagram
```

**Advantages:**
- No Node.js dependency in Rust compiler
- Fast Rust JSON generation
- Full ELK features available
- Works in arcviz-web

### Option 2: Call Node.js ELK from Rust

**Flow:**
```
Arclang Rust Compiler
    ↓ Generate ELK JSON
    ↓ Call Node.js via subprocess
Node.js + elkjs
    ↓ Return layouted JSON
Rust renders SVG
```

**Advantages:**
- Server-side rendering
- No browser required

---

## Implementation: Option 1 (Browser-based ELK)

### Step 1: Update Rust to Generate Nested ELK JSON

**File:** `src/compiler/elk_json_generator.rs` (New)

```rust
use super::semantic::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Generate true ELK hierarchical JSON from semantic model
pub struct ELKJsonGenerator {
    pub config: ELKGlobalConfig,
}

#[derive(Debug, Clone)]
pub struct ELKGlobalConfig {
    pub default_direction: String,      // "RIGHT", "DOWN", "UP", "LEFT"
    pub default_algorithm: String,      // "layered", "stress", "force"
    pub port_constraints: String,       // "FIXED_SIDE", "FREE"
    pub edge_routing: String,           // "ORTHOGONAL", "SPLINES"
    pub hierarchy_handling: String,     // "INCLUDE_CHILDREN", "SEPARATE_CHILDREN"
}

impl Default for ELKGlobalConfig {
    fn default() -> Self {
        Self {
            default_direction: "RIGHT".to_string(),
            default_algorithm: "layered".to_string(),
            port_constraints: "FIXED_SIDE".to_string(),
            edge_routing: "ORTHOGONAL".to_string(),
            hierarchy_handling: "INCLUDE_CHILDREN".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ELKNode {
    pub id: String,
    pub label: String,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub children: Vec<ELKNode>,
    pub ports: Vec<ELKPort>,
    pub layout_options: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ELKPort {
    pub id: String,
    pub label: String,
    pub side: String,  // "NORTH", "SOUTH", "EAST", "WEST"
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct ELKEdge {
    pub id: String,
    pub sources: Vec<String>,  // Can be port IDs
    pub targets: Vec<String>,
    pub labels: Vec<String>,
}

impl ELKJsonGenerator {
    pub fn new() -> Self {
        Self {
            config: ELKGlobalConfig::default(),
        }
    }
    
    pub fn generate(&self, model: &SemanticModel) -> Value {
        let mut root_children = Vec::new();
        
        // Group components by layer
        let layers = self.group_by_layer(model);
        
        for (layer_name, components) in layers {
            let layer_node = self.create_layer_container(
                &layer_name,
                components,
                &self.get_layer_config(&layer_name),
            );
            root_children.push(layer_node);
        }
        
        // Generate edges
        let edges = self.generate_edges(model);
        
        json!({
            "id": "root",
            "layoutOptions": {
                "elk.algorithm": self.config.default_algorithm,
                "elk.direction": self.config.default_direction,
                "elk.hierarchyHandling": self.config.hierarchy_handling,
                "elk.portConstraints": self.config.port_constraints,
                "elk.edgeRouting": self.config.edge_routing,
                "elk.spacing.nodeNode": "80",
                "elk.layered.spacing.nodeNodeBetweenLayers": "100",
            },
            "children": root_children,
            "edges": edges,
        })
    }
    
    fn create_layer_container(
        &self,
        layer_name: &str,
        components: Vec<&LogicalComponent>,
        config: &ELKLayerConfig,
    ) -> Value {
        let mut component_nodes = Vec::new();
        
        for comp in components {
            let node = self.create_component_node(comp);
            component_nodes.push(node);
        }
        
        json!({
            "id": format!("layer_{}", layer_name),
            "layoutOptions": {
                "elk.padding": "[top=50,left=30,bottom=30,right=30]",
                "elk.direction": config.direction,
                "elk.algorithm": config.algorithm,
                // Allow different direction than parent
                "elk.hierarchyHandling": config.hierarchy_handling,
            },
            "labels": [{
                "text": format!("{} Layer", layer_name),
                "layoutOptions": {
                    "elk.label.anchor": "TOP_CENTER"
                }
            }],
            "children": component_nodes,
        })
    }
    
    fn create_component_node(&self, comp: &LogicalComponent) -> Value {
        let mut ports = Vec::new();
        
        // Input ports on WEST
        for (idx, interface) in comp.interfaces_in.iter().enumerate() {
            ports.push(json!({
                "id": format!("{}_port_in_{}", comp.id, idx),
                "properties": {
                    "port.side": "WEST",
                    "port.index": idx,
                },
                "width": 10,
                "height": 10,
                "labels": [{
                    "text": interface,
                    "layoutOptions": {
                        "elk.label.side": "WEST"
                    }
                }]
            }));
        }
        
        // Output ports on EAST
        for (idx, interface) in comp.interfaces_out.iter().enumerate() {
            ports.push(json!({
                "id": format!("{}_port_out_{}", comp.id, idx),
                "properties": {
                    "port.side": "EAST",
                    "port.index": idx,
                },
                "width": 10,
                "height": 10,
                "labels": [{
                    "text": interface,
                    "layoutOptions": {
                        "elk.label.side": "EAST"
                    }
                }]
            }));
        }
        
        // Check if component has sub-components (nested hierarchy)
        let mut children = Vec::new();
        if !comp.sub_components.is_empty() {
            for sub_comp in &comp.sub_components {
                children.push(self.create_component_node(sub_comp));
            }
        }
        
        let mut node = json!({
            "id": comp.id,
            "width": 220,
            "height": 180,
            "labels": [{"text": comp.name}],
            "ports": ports,
        });
        
        // Add children if nested component
        if !children.is_empty() {
            node["children"] = json!(children);
            // Nested components may have different layout direction
            node["layoutOptions"] = json!({
                "elk.direction": "DOWN",  // Inner layout goes down
                "elk.padding": "[top=40,left=20,bottom=20,right=20]",
            });
        }
        
        node
    }
    
    fn generate_edges(&self, model: &SemanticModel) -> Vec<Value> {
        let mut edges = Vec::new();
        
        for la in &model.logical_architecture {
            for exchange in &la.component_exchanges {
                edges.push(json!({
                    "id": format!("edge_{}_{}", exchange.source, exchange.target),
                    "sources": [exchange.source],
                    "targets": [exchange.target],
                    "labels": exchange.exchange_items.iter()
                        .map(|item| json!({"text": item}))
                        .collect::<Vec<_>>(),
                }));
            }
        }
        
        edges
    }
    
    fn group_by_layer(&self, model: &SemanticModel) -> HashMap<String, Vec<&LogicalComponent>> {
        let mut layers: HashMap<String, Vec<&LogicalComponent>> = HashMap::new();
        
        for la in &model.logical_architecture {
            for comp in &la.components {
                // Get layer from component attributes or default to "LA"
                let layer = comp.attributes.get("layer")
                    .and_then(|v| match v {
                        AttributeValue::String(s) => Some(s.as_str()),
                        _ => None,
                    })
                    .unwrap_or("LA");
                
                layers.entry(layer.to_string())
                    .or_insert_with(Vec::new)
                    .push(comp);
            }
        }
        
        layers
    }
    
    fn get_layer_config(&self, layer_name: &str) -> ELKLayerConfig {
        // Different layers can have different configurations
        match layer_name {
            "OA" => ELKLayerConfig {
                direction: "DOWN".to_string(),  // Operational flows downward
                algorithm: "layered".to_string(),
                hierarchy_handling: "INCLUDE_CHILDREN".to_string(),
            },
            "SA" | "LA" => ELKLayerConfig {
                direction: "RIGHT".to_string(),  // System/Logical flows left-to-right
                algorithm: "layered".to_string(),
                hierarchy_handling: "INCLUDE_CHILDREN".to_string(),
            },
            "PA" => ELKLayerConfig {
                direction: "DOWN".to_string(),  // Physical deployment top-down
                algorithm: "layered".to_string(),
                hierarchy_handling: "INCLUDE_CHILDREN".to_string(),
            },
            _ => ELKLayerConfig {
                direction: self.config.default_direction.clone(),
                algorithm: self.config.default_algorithm.clone(),
                hierarchy_handling: self.config.hierarchy_handling.clone(),
            }
        }
    }
}

#[derive(Debug, Clone)]
struct ELKLayerConfig {
    direction: String,
    algorithm: String,
    hierarchy_handling: String,
}
```

### Step 2: Generate HTML with ELK.js Layout

**File:** `src/compiler/elk_html_template.rs` (New)

```rust
pub fn generate_elk_html(elk_json: &str, title: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <script src="https://d3js.org/d3.v7.min.js"></script>
    <script src="https://unpkg.com/elkjs@0.8.2/lib/elk.bundled.js"></script>
    <style>
        body {{ 
            margin: 0; 
            font-family: 'Open Sans', Arial, sans-serif;
            background: #f5f5f5;
        }}
        #diagram {{ 
            width: 100vw; 
            height: 100vh; 
        }}
        .layer {{
            fill: #E8F5E9;
            fill-opacity: 0.3;
            stroke: #37474f;
            stroke-width: 3;
            stroke-dasharray: 10,5;
        }}
        .component {{
            fill: white;
            stroke: #2196f3;
            stroke-width: 2;
        }}
        .nested-component {{
            fill: #e3f2fd;
            stroke: #1976d2;
            stroke-width: 2;
        }}
        .port {{
            fill: #4caf50;
            stroke: #2e7d32;
            stroke-width: 2;
        }}
        .port-out {{
            fill: #ff9800;
            stroke: #e65100;
        }}
        .edge {{
            stroke: #607d8b;
            stroke-width: 2;
            fill: none;
        }}
        .label {{
            font-size: 14px;
            font-weight: 600;
            fill: #2c3e50;
            text-anchor: middle;
            dominant-baseline: middle;
        }}
        .layer-label {{
            font-size: 18px;
            font-weight: bold;
            fill: #37474f;
        }}
    </style>
</head>
<body>
    <svg id="diagram"></svg>
    
    <script>
        const elkGraph = {};
        
        async function layoutAndRender() {{
            console.log('ELK Graph:', elkGraph);
            
            // Create ELK instance
            const elk = new ELK();
            
            // Perform layout
            console.time('ELK Layout');
            const layoutedGraph = await elk.layout(elkGraph);
            console.timeEnd('ELK Layout');
            console.log('Layouted Graph:', layoutedGraph);
            
            // Render with D3
            renderDiagram(layoutedGraph);
        }}
        
        function renderDiagram(graph) {{
            const svg = d3.select('#diagram');
            const width = window.innerWidth;
            const height = window.innerHeight;
            
            svg.attr('width', width).attr('height', height);
            
            const g = svg.append('g');
            
            // Render recursively (handles nested children)
            renderNode(g, graph, 0, 0, true);
            
            // Render edges
            renderEdges(g, graph);
            
            // Add zoom/pan
            const zoom = d3.zoom()
                .scaleExtent([0.1, 4])
                .on('zoom', (event) => {{
                    g.attr('transform', event.transform);
                }});
            svg.call(zoom);
            
            // Auto-fit
            const bbox = g.node().getBBox();
            const scale = Math.min(width / bbox.width, height / bbox.height) * 0.9;
            const translateX = (width - bbox.width * scale) / 2 - bbox.x * scale;
            const translateY = (height - bbox.height * scale) / 2 - bbox.y * scale;
            svg.call(zoom.transform, d3.zoomIdentity.translate(translateX, translateY).scale(scale));
        }}
        
        function renderNode(parent, node, offsetX, offsetY, isLayer) {{
            if (!node) return;
            
            const x = (node.x || 0) + offsetX;
            const y = (node.y || 0) + offsetY;
            
            const nodeGroup = parent.append('g')
                .attr('transform', `translate(${{x}},${{y}})`);
            
            // Draw node rectangle
            nodeGroup.append('rect')
                .attr('class', isLayer ? 'layer' : (node.children ? 'nested-component' : 'component'))
                .attr('width', node.width)
                .attr('height', node.height)
                .attr('rx', isLayer ? 16 : 6);
            
            // Draw label
            if (node.labels && node.labels.length > 0) {{
                nodeGroup.append('text')
                    .attr('class', isLayer ? 'layer-label' : 'label')
                    .attr('x', node.width / 2)
                    .attr('y', isLayer ? 30 : node.height / 2)
                    .text(node.labels[0].text);
            }}
            
            // Draw ports
            if (node.ports) {{
                node.ports.forEach(port => {{
                    const portGroup = nodeGroup.append('g')
                        .attr('transform', `translate(${{port.x || 0}},${{port.y || 0}})`);
                    
                    const isOut = port.properties && port.properties['port.side'] === 'EAST';
                    
                    portGroup.append('rect')
                        .attr('class', isOut ? 'port port-out' : 'port')
                        .attr('x', -5)
                        .attr('y', -5)
                        .attr('width', 10)
                        .attr('height', 10)
                        .attr('rx', 2);
                    
                    // Port label
                    if (port.labels && port.labels.length > 0) {{
                        portGroup.append('text')
                            .attr('x', isOut ? 15 : -15)
                            .attr('y', 4)
                            .attr('text-anchor', isOut ? 'start' : 'end')
                            .attr('font-size', '10px')
                            .attr('fill', '#555')
                            .text(port.labels[0].text);
                    }}
                }});
            }}
            
            // Recursively render children (nested components)
            if (node.children) {{
                node.children.forEach(child => {{
                    renderNode(nodeGroup, child, 0, 0, false);
                }});
            }}
        }}
        
        function renderEdges(parent, graph) {{
            if (!graph.edges) return;
            
            graph.edges.forEach(edge => {{
                if (!edge.sections || edge.sections.length === 0) return;
                
                edge.sections.forEach(section => {{
                    let pathData = `M ${{section.startPoint.x}} ${{section.startPoint.y}}`;
                    
                    if (section.bendPoints) {{
                        section.bendPoints.forEach(bp => {{
                            pathData += ` L ${{bp.x}} ${{bp.y}}`;
                        }});
                    }}
                    
                    pathData += ` L ${{section.endPoint.x}} ${{section.endPoint.y}}`;
                    
                    parent.append('path')
                        .attr('class', 'edge')
                        .attr('d', pathData)
                        .attr('marker-end', 'url(#arrowhead)');
                }});
                
                // Edge label
                if (edge.labels && edge.labels.length > 0) {{
                    const section = edge.sections[0];
                    const midX = (section.startPoint.x + section.endPoint.x) / 2;
                    const midY = (section.startPoint.y + section.endPoint.y) / 2;
                    
                    parent.append('text')
                        .attr('x', midX)
                        .attr('y', midY)
                        .attr('font-size', '12px')
                        .attr('fill', '#555')
                        .text(edge.labels[0].text);
                }}
            }});
            
            // Arrowhead marker
            parent.append('defs').append('marker')
                .attr('id', 'arrowhead')
                .attr('markerWidth', 10)
                .attr('markerHeight', 10)
                .attr('refX', 9)
                .attr('refY', 3)
                .attr('orient', 'auto')
                .append('polygon')
                .attr('points', '0 0, 10 3, 0 6')
                .attr('fill', '#607d8b');
        }}
        
        layoutAndRender();
    </script>
</body>
</html>"#, title, elk_json)
}
```

### Step 3: Update CLI Command

**File:** `src/cli/commands.rs`

```rust
use crate::compiler::elk_json_generator::ELKJsonGenerator;
use crate::compiler::elk_html_template::generate_elk_html;

pub fn handle_export_elk(input: &Path, output: &Path) -> Result<(), CompilerError> {
    // Compile to semantic model
    let model = compile_to_semantic_model(input)?;
    
    // Generate ELK JSON
    let elk_generator = ELKJsonGenerator::new();
    let elk_json = elk_generator.generate(&model);
    let elk_json_str = serde_json::to_string_pretty(&elk_json)?;
    
    // Generate HTML with embedded ELK JSON
    let html = generate_elk_html(&elk_json_str, "ArcLang Diagram");
    
    std::fs::write(output, html)?;
    
    println!("✓ ELK diagram generated: {}", output.display());
    
    Ok(())
}
```

### Step 4: Usage Example

**ArcLang with Nested Components:**

```arc
logical_architecture "Nested System" {
    component "OuterContainer" {
        id: "OUTER-001"
        type: "System"
        layout_direction: "RIGHT"  // Container flows right
        
        component "InnerModule1" {
            id: "INNER-001"
            type: "Module"
            layout_direction: "DOWN"  // Inner flows down!
            
            function "Process" {
                id: "FUNC-001"
            }
            
            function "Validate" {
                id: "FUNC-002"
            }
        }
        
        component "InnerModule2" {
            id: "INNER-002"
            type: "Module"
            layout_algorithm: "stress"  // Different algorithm!
            
            function "Analyze" {
                id: "FUNC-003"
            }
        }
    }
}
```

**CLI:**
```bash
arclang export model.arc --format elk --output diagram.html
open diagram.html
```

---

## Advanced Features

### Mixed Layout Algorithms

```rust
impl ELKJsonGenerator {
    fn create_component_with_custom_layout(
        &self,
        comp: &LogicalComponent,
        algorithm: &str,  // "layered", "stress", "force", "mrtree"
    ) -> Value {
        json!({
            "id": comp.id,
            "layoutOptions": {
                "elk.algorithm": algorithm,
                // Stress-specific options
                "elk.stress.desiredEdgeLength": "100",
                "elk.stress.epsilon": "0.01",
                // Force-specific options
                "elk.force.repulsion": "5.0",
                "elk.force.temperature": "0.001",
            },
            "children": [...]
        })
    }
}
```

### Interactive Layout Options

```rust
// Allow user to specify per-component layout in .arc file
component "SpecialComponent" {
    id: "COMP-001"
    
    // Layout metadata
    layout {
        algorithm: "stress"
        direction: "DOWN"
        port_constraints: "FREE"
        edge_routing: "SPLINES"
    }
    
    // ... rest of component
}
```

---

## Testing

Create test cases with nested layouts:

```arc
// tests/nested_layout_test.arc
logical_architecture "Test Nested" {
    component "Root" {
        id: "ROOT"
        layout_direction: "RIGHT"
        
        component "Child1" {
            id: "CHILD1"
            layout_direction: "DOWN"  // Different!
            
            function "F1" { id: "F1" }
            function "F2" { id: "F2" }
        }
        
        component "Child2" {
            id: "CHILD2"
            layout_algorithm: "stress"  // Different algorithm!
            
            function "F3" { id: "F3" }
            function "F4" { id: "F4" }
        }
    }
}

trace "CHILD1" -> "CHILD2" {
    exchange_items: ["data"]
}
```

Test:
```bash
arclang export tests/nested_layout_test.arc --format elk -o test_nested.html
```

---

## Advanced Features: Interactive Layout & Mixed Algorithms

### Interactive Layout Support

Based on your ELK examples showing `interactiveLayout: true` with constraints:

**Extended ELK JSON Generator:**

```rust
// Add to src/compiler/elk_json_generator.rs

#[derive(Debug, Clone)]
pub struct ELKInteractiveConfig {
    pub enabled: bool,
    pub expandNodes: bool,
    pub aspectRatio: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ELKNodeConstraints {
    pub position_choice: Option<i32>,  // positionChoiceConstraint
    pub layer_choice: Option<i32>,     // layerChoiceConstraint
}

impl ELKJsonGenerator {
    /// Create node with mixed algorithms and interactive layout
    fn create_interactive_node(
        &self,
        comp: &LogicalComponent,
        algorithm: &str,              // "rectpacking", "layered", "stress"
        interactive: bool,
        constraints: Option<ELKNodeConstraints>,
    ) -> Value {
        let mut layout_options = json!({
            "elk.algorithm": algorithm,
            "interactiveLayout": interactive,
        });
        
        // Algorithm-specific options
        match algorithm {
            "rectpacking" => {
                layout_options["elk.aspectRatio"] = json!("1.908321167883212");
                layout_options["expandNodes"] = json!(false);
            },
            "layered" => {
                layout_options["elk.direction"] = json!("RIGHT");
                layout_options["separateConnectedComponents"] = json!(false);
                
                // Add constraints if provided
                if let Some(c) = constraints {
                    if let Some(layer) = c.layer_choice {
                        layout_options["layering.layerChoiceConstraint"] = json!(layer);
                    }
                    if let Some(pos) = c.position_choice {
                        layout_options["crossingMinimization.positionChoiceConstraint"] = json!(pos);
                    }
                }
            },
            "stress" => {
                layout_options["elk.stress.desiredEdgeLength"] = json!("100");
                layout_options["elk.stress.epsilon"] = json!("0.01");
            },
            "force" => {
                layout_options["elk.force.repulsion"] = json!("5.0");
                layout_options["elk.force.temperature"] = json!("0.001");
            },
            _ => {}
        }
        
        json!({
            "id": comp.id,
            "layoutOptions": layout_options,
            "labels": [{"text": comp.name}],
            "children": self.create_children_nodes(&comp.sub_components),
        })
    }
    
    /// Generate mixed-algorithm graph (like your example)
    /// Root uses rectpacking, nested nodes use layered
    pub fn generate_mixed_algorithm(&self, model: &SemanticModel) -> Value {
        let mut root_children = Vec::new();
        
        // Simple leaf nodes at root level
        for simple_comp in &model.simple_components {
            root_children.push(json!({
                "id": simple_comp.id,
                "labels": [{"text": simple_comp.name}],
                "width": 80,
                "height": 60,
            }));
        }
        
        // Complex nested components with different algorithm
        for nested_comp in &model.nested_components {
            let mut nested_children = Vec::new();
            
            for sub in &nested_comp.sub_components {
                // Apply constraints to specific nodes
                let constraints = if sub.id == "n2" {
                    Some(ELKNodeConstraints {
                        layer_choice: Some(0),      // Can set to 1 for effect
                        position_choice: Some(0),
                    })
                } else {
                    None
                };
                
                nested_children.push(json!({
                    "id": sub.id,
                    "labels": [{"text": sub.name}],
                    "layoutOptions": {
                        "layering.layerChoiceConstraint": 
                            constraints.as_ref().and_then(|c| c.layer_choice).unwrap_or(0),
                        "crossingMinimization.positionChoiceConstraint":
                            constraints.as_ref().and_then(|c| c.position_choice).unwrap_or(0),
                    },
                    "width": 80,
                    "height": 60,
                }));
            }
            
            // Nested container uses layered + interactive
            root_children.push(json!({
                "id": nested_comp.id,
                "layoutOptions": {
                    "interactiveLayout": true,
                    "elk.algorithm": "layered",
                    "elk.direction": "RIGHT",
                    "separateConnectedComponents": false,
                },
                "labels": [{"text": nested_comp.name}],
                "children": nested_children,
            }));
        }
        
        // Root uses rectpacking
        json!({
            "id": "root",
            "layoutOptions": {
                "interactiveLayout": true,
                "elk.algorithm": "rectpacking",
                "expandNodes": false,
                "aspectRatio": 1.908321167883212,
            },
            "children": root_children,
            "edges": self.generate_edges(model),
        })
    }
}
```

### ArcLang Syntax for Interactive Layouts

**Proposed `.arc` syntax to support these features:**

```arc
logical_architecture "Interactive System" {
    // Root level uses rectpacking
    layout {
        algorithm: "rectpacking"
        interactive: true
        aspectRatio: 1.908
        expandNodes: false
    }
    
    // Simple components at root
    component "n11" {
        id: "n11"
    }
    
    component "n12" {
        id: "n12"
    }
    
    // Nested component with different algorithm
    component "TestContainer" {
        id: "n13"
        
        // Override layout for this container
        layout {
            algorithm: "layered"
            direction: "RIGHT"
            interactive: true
            separateConnectedComponents: false
        }
        
        component "n1" {
            id: "n1"
        }
        
        component "n2" {
            id: "n2"
            
            // Apply interactive constraints
            constraints {
                layerChoice: 0        // Set to 1 for different effect
                positionChoice: 0
            }
        }
    }
}
```

### Parser Updates Required

**File:** `src/compiler/parser.rs` (additions)

```rust
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub algorithm: String,           // "rectpacking", "layered", "stress", "force"
    pub direction: Option<String>,   // "RIGHT", "DOWN", "UP", "LEFT"
    pub interactive: bool,
    pub aspect_ratio: Option<f64>,
    pub expand_nodes: Option<bool>,
    pub separate_connected_components: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct LayoutConstraints {
    pub layer_choice: Option<i32>,
    pub position_choice: Option<i32>,
}

// Add to LogicalComponent struct
pub struct LogicalComponent {
    pub id: String,
    pub name: String,
    pub layout: Option<LayoutConfig>,
    pub constraints: Option<LayoutConstraints>,
    // ... existing fields
}

// Parser rule for layout block
fn parse_layout_block(input: &str) -> IResult<&str, LayoutConfig> {
    map(
        delimited(
            tag("layout"),
            delimited(ws(tag("{")), many0(parse_layout_property), ws(tag("}"))),
            opt(ws(tag(","))),
        ),
        |props| build_layout_config(props),
    )(input)
}

fn parse_layout_property(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(
        alpha1,
        ws(tag(":")),
        terminated(take_until_either("\n,}"), opt(ws(tag(",")))),
    )(input)
}
```

### Updated HTML Template with Interactive Support

**File:** `src/compiler/elk_html_template.rs` (extended)

```rust
pub fn generate_elk_html_with_interactive(elk_json: &str, title: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <script src="https://d3js.org/d3.v7.min.js"></script>
    <script src="https://unpkg.com/elkjs@0.8.2/lib/elk.bundled.js"></script>
    <style>
        /* ... existing styles ... */
        
        .interactive-node {{
            cursor: move;
        }}
        
        .constraint-indicator {{
            fill: #ff5722;
            stroke: #d32f2f;
            stroke-width: 2;
        }}
    </style>
</head>
<body>
    <div id="controls">
        <button id="relayout">Re-layout</button>
        <label>
            <input type="checkbox" id="interactive-mode" checked> Interactive Mode
        </label>
    </div>
    <svg id="diagram"></svg>
    
    <script>
        const elkGraph = {};
        let interactiveMode = true;
        
        async function layoutAndRender() {{
            const elk = new ELK();
            
            console.log('ELK Graph (Mixed Algorithms):', elkGraph);
            console.log('Root algorithm:', elkGraph.layoutOptions['elk.algorithm']);
            console.log('Interactive:', elkGraph.layoutOptions['interactiveLayout']);
            
            console.time('ELK Layout');
            const layoutedGraph = await elk.layout(elkGraph);
            console.timeEnd('ELK Layout');
            
            renderDiagram(layoutedGraph);
        }}
        
        function renderDiagram(graph) {{
            const svg = d3.select('#diagram');
            svg.selectAll('*').remove();
            
            const width = window.innerWidth;
            const height = window.innerHeight - 50;
            svg.attr('width', width).attr('height', height);
            
            const g = svg.append('g');
            
            // Render with algorithm detection
            renderNode(g, graph, 0, 0, true);
            renderEdges(g, graph);
            
            // Add zoom/pan
            const zoom = d3.zoom()
                .scaleExtent([0.1, 4])
                .on('zoom', (event) => {{
                    g.attr('transform', event.transform);
                }});
            svg.call(zoom);
            
            // Auto-fit
            const bbox = g.node().getBBox();
            const scale = Math.min(width / bbox.width, height / bbox.height) * 0.9;
            const translateX = (width - bbox.width * scale) / 2 - bbox.x * scale;
            const translateY = (height - bbox.height * scale) / 2 - bbox.y * scale;
            svg.call(zoom.transform, d3.zoomIdentity.translate(translateX, translateY).scale(scale));
            
            // Interactive drag support
            if (interactiveMode) {{
                enableInteractiveDrag(g);
            }}
        }}
        
        function renderNode(parent, node, offsetX, offsetY, isRoot) {{
            if (!node) return;
            
            const x = (node.x || 0) + offsetX;
            const y = (node.y || 0) + offsetY;
            
            const nodeGroup = parent.append('g')
                .attr('transform', `translate(${{x}},${{y}})`)
                .attr('class', node.layoutOptions?.interactiveLayout ? 'interactive-node' : '');
            
            // Visual indicator for algorithm type
            const algorithm = node.layoutOptions?.['elk.algorithm'] || 'layered';
            const algorithmColor = {{
                'rectpacking': '#e3f2fd',
                'layered': '#fff3e0',
                'stress': '#f3e5f5',
                'force': '#e8f5e9'
            }}[algorithm] || 'white';
            
            nodeGroup.append('rect')
                .attr('class', isRoot ? 'layer' : 'component')
                .attr('width', node.width)
                .attr('height', node.height)
                .attr('fill', algorithmColor)
                .attr('rx', 6);
            
            // Show constraints indicator
            if (node.layoutOptions?.['layering.layerChoiceConstraint'] !== undefined ||
                node.layoutOptions?.['crossingMinimization.positionChoiceConstraint'] !== undefined) {{
                nodeGroup.append('circle')
                    .attr('class', 'constraint-indicator')
                    .attr('cx', 10)
                    .attr('cy', 10)
                    .attr('r', 5);
            }}
            
            // Label with algorithm name
            if (node.labels && node.labels.length > 0) {{
                nodeGroup.append('text')
                    .attr('class', 'label')
                    .attr('x', node.width / 2)
                    .attr('y', node.height / 2)
                    .text(node.labels[0].text);
                    
                // Show algorithm name
                nodeGroup.append('text')
                    .attr('x', node.width / 2)
                    .attr('y', node.height - 10)
                    .attr('font-size', '10px')
                    .attr('fill', '#666')
                    .attr('text-anchor', 'middle')
                    .text(`[${{algorithm}}]`);
            }}
            
            // Recursively render children
            if (node.children) {{
                node.children.forEach(child => {{
                    renderNode(nodeGroup, child, 0, 0, false);
                }});
            }}
        }}
        
        function enableInteractiveDrag(g) {{
            const nodes = g.selectAll('.interactive-node');
            
            const drag = d3.drag()
                .on('start', function(event) {{
                    d3.select(this).raise().attr('stroke', '#ff5722');
                }})
                .on('drag', function(event) {{
                    const transform = d3.select(this).attr('transform');
                    const translate = transform.match(/translate\\(([^,]+),([^)]+)\\)/);
                    if (translate) {{
                        const baseX = parseFloat(translate[1]);
                        const baseY = parseFloat(translate[2]);
                        d3.select(this).attr('transform', 
                            `translate(${{baseX + event.dx}},${{baseY + event.dy}})`);
                    }}
                }})
                .on('end', function(event) {{
                    d3.select(this).attr('stroke', null);
                }});
            
            nodes.call(drag);
        }}
        
        function renderEdges(parent, graph) {{
            // ... existing edge rendering ...
        }}
        
        // Control buttons
        document.getElementById('relayout').addEventListener('click', layoutAndRender);
        document.getElementById('interactive-mode').addEventListener('change', (e) => {{
            interactiveMode = e.target.checked;
            layoutAndRender();
        }});
        
        layoutAndRender();
    </script>
</body>
</html>"#, title, elk_json)
}
```

## Summary

**What You Need:**
1. ✅ Generate true ELK JSON with nested children
2. ✅ Support `layoutOptions` per node
3. ✅ Allow mixed directions (RIGHT/DOWN/UP/LEFT)
4. ✅ Allow mixed algorithms (layered/stress/force/rectpacking)
5. ✅ Interactive layout with constraints (positionChoiceConstraint, layerChoiceConstraint)
6. ✅ Per-container algorithm override
7. ✅ Render hierarchically in browser with D3.js
8. ✅ Interactive drag support for nodes

**Why This Works:**
- Rust generates JSON (fast)
- ELK.js does complex layout with full algorithm support (correct)
- Browser renders SVG with interactivity (beautiful)
- No Node.js dependency in Rust
- Full feature parity with ELK examples you provided

**Next Step:** Implement `elk_json_generator.rs` with mixed algorithm and interactive constraint support!
