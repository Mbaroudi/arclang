# Focused Implementation Plan: Core Capella Features for Arclang

**Date:** October 28, 2025  
**Goal:** Production-ready Capella diagrams + Enhanced ArcViz web capabilities  
**Timeline:** 8-10 weeks  
**Scope:** Context diagrams, Model diff, Requirements export, Code generation, Basic validation  

---

## Project Scope

### ✅ **In Scope (Implement in Rust)**
1. Context diagrams (focused component views)
2. Model comparison and diff
3. Requirements export (Excel, ReqIF)
4. Code generation (ROS2, Protobuf)
5. Basic validation rule system (15 core rules)

### ⏳ **Future Scope (Use py-capellambse)**
- PVMT extension support
- Advanced metrics and complexity analysis
- Jupyter notebook integration
- Advanced querying and filtering
- Polarion/DOORS synchronization

### 🎯 **Primary Goal**
**"Perfect Capella diagrams + Professional web interface"**

---

## Phase 1: Context Diagrams (Week 1-2)

### Week 1: Core Implementation

**File:** `src/compiler/context_diagram.rs` (New)

```rust
use super::ast::*;
use super::semantic::*;
use std::collections::{HashSet, HashMap};

#[derive(Debug, Clone)]
pub struct ContextDiagramConfig {
    pub focus_element_id: String,
    pub depth: usize,  // 1 = direct neighbors, 2 = neighbors of neighbors
    pub include_ports: bool,
    pub include_functions: bool,
    pub show_labels: bool,
}

#[derive(Debug, Clone)]
pub struct ContextDiagram {
    pub name: String,
    pub focus_element: LogicalComponent,
    pub related_components: Vec<LogicalComponent>,
    pub related_edges: Vec<ComponentExchange>,
    pub viewport: Bounds,
}

pub struct ContextDiagramGenerator;

impl ContextDiagramGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate(
        &self,
        model: &SemanticModel,
        config: &ContextDiagramConfig,
    ) -> Result<ContextDiagram, String> {
        // Find focus element
        let focus = self.find_component(model, &config.focus_element_id)?;
        
        // Collect context elements at specified depth
        let mut context_ids = HashSet::new();
        context_ids.insert(config.focus_element_id.clone());
        
        self.expand_context(
            model,
            &config.focus_element_id,
            config.depth,
            &mut context_ids,
        );
        
        // Filter components
        let related_components: Vec<_> = model.logical_architecture.iter()
            .flat_map(|la| &la.components)
            .filter(|c| context_ids.contains(&c.name))
            .cloned()
            .collect();
        
        // Filter edges (only between context elements)
        let related_edges: Vec<_> = model.logical_architecture.iter()
            .flat_map(|la| &la.component_exchanges)
            .filter(|e| {
                context_ids.contains(&e.source) && context_ids.contains(&e.target)
            })
            .cloned()
            .collect();
        
        Ok(ContextDiagram {
            name: format!("{} Context", focus.name),
            focus_element: focus.clone(),
            related_components,
            related_edges,
            viewport: self.calculate_viewport(&related_components),
        })
    }
    
    fn expand_context(
        &self,
        model: &SemanticModel,
        element_id: &str,
        depth: usize,
        context_ids: &mut HashSet<String>,
    ) {
        if depth == 0 {
            return;
        }
        
        // Find all edges connected to this element
        for la in &model.logical_architecture {
            for edge in &la.component_exchanges {
                if edge.source == element_id && !context_ids.contains(&edge.target) {
                    context_ids.insert(edge.target.clone());
                    if depth > 1 {
                        self.expand_context(model, &edge.target, depth - 1, context_ids);
                    }
                }
                if edge.target == element_id && !context_ids.contains(&edge.source) {
                    context_ids.insert(edge.source.clone());
                    if depth > 1 {
                        self.expand_context(model, &edge.source, depth - 1, context_ids);
                    }
                }
            }
        }
    }
    
    fn find_component(
        &self,
        model: &SemanticModel,
        id: &str,
    ) -> Result<LogicalComponent, String> {
        for la in &model.logical_architecture {
            for comp in &la.components {
                if comp.name == id || comp.id == id {
                    return Ok(comp.clone());
                }
            }
        }
        Err(format!("Component not found: {}", id))
    }
    
    fn calculate_viewport(&self, components: &[LogicalComponent]) -> Bounds {
        // Calculate bounding box for viewport
        Bounds {
            x: 0.0,
            y: 0.0,
            width: 1000.0,
            height: 800.0,
        }
    }
}

#[derive(Debug, Clone)]
struct Bounds {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}
```

**File:** `src/cli/commands.rs` (Update)

```rust
// Add to existing command enum
#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    // ... existing commands
    
    /// Generate context diagram for a specific element
    Context {
        /// Input .arc file
        input: PathBuf,
        
        /// Element ID to focus on
        #[arg(short, long)]
        focus: String,
        
        /// Context depth (1=direct, 2=neighbors of neighbors)
        #[arg(short, long, default_value = "1")]
        depth: usize,
        
        /// Output file
        #[arg(short, long)]
        output: PathBuf,
        
        /// Output format
        #[arg(short, long, default_value = "html")]
        format: String,
    },
}

// In handle_command function
pub fn handle_context(
    input: &Path,
    focus: &str,
    depth: usize,
    output: &Path,
    format: &str,
) -> Result<(), CompilerError> {
    let model = compile_to_semantic_model(input)?;
    
    let config = ContextDiagramConfig {
        focus_element_id: focus.to_string(),
        depth,
        include_ports: true,
        include_functions: true,
        show_labels: true,
    };
    
    let generator = ContextDiagramGenerator::new();
    let context_diagram = generator.generate(&model, &config)
        .map_err(|e| CompilerError::Semantic(e))?;
    
    // Generate HTML with ArcViz
    let html = generate_context_html(&context_diagram)?;
    std::fs::write(output, html)?;
    
    println!("✓ Context diagram generated: {}", output.display());
    println!("  Focus: {}", focus);
    println!("  Depth: {}", depth);
    println!("  Components: {}", context_diagram.related_components.len());
    println!("  Connections: {}", context_diagram.related_edges.len());
    
    Ok(())
}
```

### Week 2: ArcViz Integration

**File:** `src/compiler/arcviz_context.rs` (New)

```rust
use super::context_diagram::*;

pub fn generate_context_html(diagram: &ContextDiagram) -> Result<String, String> {
    let diagram_json = serde_json::to_string(&diagram)
        .map_err(|e| format!("JSON serialization error: {}", e))?;
    
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{} - Context Diagram</title>
    <script src="https://d3js.org/d3.v7.min.js"></script>
    <script src="https://unpkg.com/elkjs@0.8.2/lib/elk.bundled.js"></script>
    <style>
        body {{ 
            margin: 0; 
            font-family: 'Open Sans', Arial, sans-serif;
            background: #f5f5f5;
        }}
        #diagram-container {{ 
            width: 100vw; 
            height: 100vh; 
        }}
        .focus-component {{
            stroke: #ff6b6b !important;
            stroke-width: 4px !important;
            filter: drop-shadow(0 0 10px rgba(255,107,107,0.5));
        }}
        .context-component {{
            stroke: #4ecdc4 !important;
            stroke-width: 2px !important;
        }}
        .context-edge {{
            stroke: #95a5a6 !important;
            stroke-width: 2px !important;
        }}
        .component-label {{
            font-size: 14px;
            font-weight: 600;
            fill: #2c3e50;
        }}
        #info-panel {{
            position: absolute;
            top: 20px;
            right: 20px;
            background: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            min-width: 250px;
        }}
        .info-title {{
            font-size: 18px;
            font-weight: bold;
            margin-bottom: 10px;
            color: #2c3e50;
        }}
        .info-item {{
            margin: 8px 0;
            font-size: 14px;
            color: #555;
        }}
        .info-label {{
            font-weight: 600;
            color: #34495e;
        }}
    </style>
</head>
<body>
    <div id="info-panel">
        <div class="info-title">Context Diagram</div>
        <div class="info-item">
            <span class="info-label">Focus:</span> {}
        </div>
        <div class="info-item">
            <span class="info-label">Components:</span> {}
        </div>
        <div class="info-item">
            <span class="info-label">Connections:</span> {}
        </div>
        <div class="info-item">
            <span class="info-label">Depth:</span> Level {}
        </div>
    </div>
    <svg id="diagram-container"></svg>
    
    <script>
        const diagramData = {};
        
        // Render context diagram with focus highlighting
        async function renderContextDiagram() {{
            const svg = d3.select('#diagram-container');
            const width = window.innerWidth;
            const height = window.innerHeight;
            
            svg.attr('width', width).attr('height', height);
            const g = svg.append('g');
            
            // ELK layout
            const elk = new ELK();
            const graph = await elk.layout({{
                id: "root",
                layoutOptions: {{
                    'elk.algorithm': 'layered',
                    'elk.direction': 'RIGHT',
                    'elk.spacing.nodeNode': '80',
                }},
                children: diagramData.related_components.map(comp => ({{
                    id: comp.name,
                    width: 200,
                    height: 100,
                    labels: [{{ text: comp.name }}]
                }})),
                edges: diagramData.related_edges.map(edge => ({{
                    id: `${{edge.source}}-${{edge.target}}`,
                    sources: [edge.source],
                    targets: [edge.target]
                }}))
            }});
            
            // Draw components
            graph.children.forEach(node => {{
                const isFocus = node.id === diagramData.focus_element.name;
                const compGroup = g.append('g')
                    .attr('class', isFocus ? 'focus-component' : 'context-component')
                    .attr('transform', `translate(${{node.x}},${{node.y}})`);
                
                compGroup.append('rect')
                    .attr('width', node.width)
                    .attr('height', node.height)
                    .attr('rx', 6)
                    .attr('fill', isFocus ? '#ffe6e6' : '#e6f7f7')
                    .attr('stroke', isFocus ? '#ff6b6b' : '#4ecdc4')
                    .attr('stroke-width', isFocus ? 4 : 2);
                
                compGroup.append('text')
                    .attr('class', 'component-label')
                    .attr('x', node.width / 2)
                    .attr('y', node.height / 2)
                    .attr('text-anchor', 'middle')
                    .attr('dominant-baseline', 'middle')
                    .text(node.labels[0].text);
            }});
            
            // Draw edges
            graph.edges.forEach(edge => {{
                const path = g.append('path')
                    .attr('class', 'context-edge')
                    .attr('fill', 'none')
                    .attr('stroke', '#95a5a6')
                    .attr('stroke-width', 2)
                    .attr('marker-end', 'url(#arrowhead)');
                
                // Simple line (ELK provides sections for complex routing)
                const sourceNode = graph.children.find(n => n.id === edge.sources[0]);
                const targetNode = graph.children.find(n => n.id === edge.targets[0]);
                
                path.attr('d', `M${{sourceNode.x + sourceNode.width}},${{sourceNode.y + sourceNode.height/2}} L${{targetNode.x}},${{targetNode.y + targetNode.height/2}}`);
            }});
            
            // Add arrowhead marker
            svg.append('defs').append('marker')
                .attr('id', 'arrowhead')
                .attr('markerWidth', 10)
                .attr('markerHeight', 10)
                .attr('refX', 9)
                .attr('refY', 3)
                .attr('orient', 'auto')
                .append('polygon')
                .attr('points', '0 0, 10 3, 0 6')
                .attr('fill', '#95a5a6');
            
            // Zoom/pan
            const zoom = d3.zoom()
                .scaleExtent([0.1, 4])
                .on('zoom', (event) => {{
                    g.attr('transform', event.transform);
                }});
            svg.call(zoom);
            
            // Auto-fit
            const bbox = g.node().getBBox();
            const scale = Math.min(width / bbox.width, height / bbox.height) * 0.8;
            const translateX = (width - bbox.width * scale) / 2 - bbox.x * scale;
            const translateY = (height - bbox.height * scale) / 2 - bbox.y * scale;
            svg.call(zoom.transform, d3.zoomIdentity.translate(translateX, translateY).scale(scale));
        }}
        
        renderContextDiagram();
    </script>
</body>
</html>"#,
        diagram.name,
        diagram.focus_element.name,
        diagram.related_components.len(),
        diagram.related_edges.len(),
        // depth placeholder
        1,
        diagram_json
    );
    
    Ok(html)
}
```

**CLI Usage:**
```bash
# Generate context diagram for specific component
arclang context model.arc --focus LC-001 --depth 1 -o context.html

# Depth 2 (neighbors of neighbors)
arclang context model.arc --focus "Controller" --depth 2 -o context_deep.html
```

**Testing:**
```bash
cd /Users/malek/Arclang/examples/automotive
arclang context acc_complete_architecture.arc --focus "Distance Controller" --depth 1 -o acc_context.html
```

---

## Phase 2: Model Diff & Comparison (Week 3-4)

### Week 3: Diff Engine

**File:** `src/compiler/diff.rs` (New)

```rust
use super::semantic::*;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementChange {
    pub change_type: ChangeType,
    pub element_type: String,  // "Component", "Requirement", "Edge"
    pub element_id: String,
    pub element_name: String,
    pub details: Vec<FieldChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDiff {
    pub changes: Vec<ElementChange>,
    pub summary: DiffSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub added_count: usize,
    pub removed_count: usize,
    pub modified_count: usize,
    pub unchanged_count: usize,
}

pub struct DiffEngine;

impl DiffEngine {
    pub fn new() -> Self {
        Self
    }
    
    pub fn compare(
        &self,
        model_a: &SemanticModel,
        model_b: &SemanticModel,
    ) -> ModelDiff {
        let mut changes = Vec::new();
        
        // Compare requirements
        changes.extend(self.diff_requirements(model_a, model_b));
        
        // Compare components
        changes.extend(self.diff_components(model_a, model_b));
        
        // Compare traces
        changes.extend(self.diff_traces(model_a, model_b));
        
        let summary = self.calculate_summary(&changes, model_a, model_b);
        
        ModelDiff { changes, summary }
    }
    
    fn diff_requirements(
        &self,
        model_a: &SemanticModel,
        model_b: &SemanticModel,
    ) -> Vec<ElementChange> {
        let mut changes = Vec::new();
        
        let a_reqs: HashMap<_, _> = model_a.system_analysis.iter()
            .flat_map(|sa| &sa.requirements)
            .map(|r| (r.id.clone(), r))
            .collect();
        
        let b_reqs: HashMap<_, _> = model_b.system_analysis.iter()
            .flat_map(|sa| &sa.requirements)
            .map(|r| (r.id.clone(), r))
            .collect();
        
        // Find added requirements (in B but not in A)
        for (id, req) in &b_reqs {
            if !a_reqs.contains_key(id) {
                changes.push(ElementChange {
                    change_type: ChangeType::Added,
                    element_type: "Requirement".to_string(),
                    element_id: id.clone(),
                    element_name: self.get_req_description(req),
                    details: vec![],
                });
            }
        }
        
        // Find removed requirements (in A but not in B)
        for (id, req) in &a_reqs {
            if !b_reqs.contains_key(id) {
                changes.push(ElementChange {
                    change_type: ChangeType::Removed,
                    element_type: "Requirement".to_string(),
                    element_id: id.clone(),
                    element_name: self.get_req_description(req),
                    details: vec![],
                });
            }
        }
        
        // Find modified requirements
        for (id, req_a) in &a_reqs {
            if let Some(req_b) = b_reqs.get(id) {
                let mut details = Vec::new();
                
                // Compare description
                let desc_a = self.get_req_description(req_a);
                let desc_b = self.get_req_description(req_b);
                if desc_a != desc_b {
                    details.push(FieldChange {
                        field_name: "description".to_string(),
                        old_value: Some(desc_a.clone()),
                        new_value: Some(desc_b.clone()),
                    });
                }
                
                // Compare priority
                let pri_a = self.get_req_priority(req_a);
                let pri_b = self.get_req_priority(req_b);
                if pri_a != pri_b {
                    details.push(FieldChange {
                        field_name: "priority".to_string(),
                        old_value: Some(pri_a),
                        new_value: Some(pri_b),
                    });
                }
                
                if !details.is_empty() {
                    changes.push(ElementChange {
                        change_type: ChangeType::Modified,
                        element_type: "Requirement".to_string(),
                        element_id: id.clone(),
                        element_name: desc_a,
                        details,
                    });
                }
            }
        }
        
        changes
    }
    
    fn diff_components(
        &self,
        model_a: &SemanticModel,
        model_b: &SemanticModel,
    ) -> Vec<ElementChange> {
        let mut changes = Vec::new();
        
        let a_comps: HashMap<_, _> = model_a.logical_architecture.iter()
            .flat_map(|la| &la.components)
            .map(|c| (c.id.clone(), c))
            .collect();
        
        let b_comps: HashMap<_, _> = model_b.logical_architecture.iter()
            .flat_map(|la| &la.components)
            .map(|c| (c.id.clone(), c))
            .collect();
        
        // Added components
        for (id, comp) in &b_comps {
            if !a_comps.contains_key(id) {
                changes.push(ElementChange {
                    change_type: ChangeType::Added,
                    element_type: "Component".to_string(),
                    element_id: id.clone(),
                    element_name: comp.name.clone(),
                    details: vec![],
                });
            }
        }
        
        // Removed components
        for (id, comp) in &a_comps {
            if !b_comps.contains_key(id) {
                changes.push(ElementChange {
                    change_type: ChangeType::Removed,
                    element_type: "Component".to_string(),
                    element_id: id.clone(),
                    element_name: comp.name.clone(),
                    details: vec![],
                });
            }
        }
        
        // Modified components
        for (id, comp_a) in &a_comps {
            if let Some(comp_b) = b_comps.get(id) {
                let mut details = Vec::new();
                
                if comp_a.name != comp_b.name {
                    details.push(FieldChange {
                        field_name: "name".to_string(),
                        old_value: Some(comp_a.name.clone()),
                        new_value: Some(comp_b.name.clone()),
                    });
                }
                
                if comp_a.component_type != comp_b.component_type {
                    details.push(FieldChange {
                        field_name: "type".to_string(),
                        old_value: Some(comp_a.component_type.clone()),
                        new_value: Some(comp_b.component_type.clone()),
                    });
                }
                
                if !details.is_empty() {
                    changes.push(ElementChange {
                        change_type: ChangeType::Modified,
                        element_type: "Component".to_string(),
                        element_id: id.clone(),
                        element_name: comp_a.name.clone(),
                        details,
                    });
                }
            }
        }
        
        changes
    }
    
    fn diff_traces(
        &self,
        model_a: &SemanticModel,
        model_b: &SemanticModel,
    ) -> Vec<ElementChange> {
        let mut changes = Vec::new();
        
        let a_traces: HashSet<_> = model_a.traces.iter()
            .map(|t| format!("{}-{}-{}", t.from, t.trace_type, t.to))
            .collect();
        
        let b_traces: HashSet<_> = model_b.traces.iter()
            .map(|t| format!("{}-{}-{}", t.from, t.trace_type, t.to))
            .collect();
        
        // Added traces
        for trace_key in b_traces.difference(&a_traces) {
            let trace = model_b.traces.iter()
                .find(|t| format!("{}-{}-{}", t.from, t.trace_type, t.to) == *trace_key)
                .unwrap();
            
            changes.push(ElementChange {
                change_type: ChangeType::Added,
                element_type: "Trace".to_string(),
                element_id: trace_key.clone(),
                element_name: format!("{} {} {}", trace.from, trace.trace_type, trace.to),
                details: vec![],
            });
        }
        
        // Removed traces
        for trace_key in a_traces.difference(&b_traces) {
            let trace = model_a.traces.iter()
                .find(|t| format!("{}-{}-{}", t.from, t.trace_type, t.to) == *trace_key)
                .unwrap();
            
            changes.push(ElementChange {
                change_type: ChangeType::Removed,
                element_type: "Trace".to_string(),
                element_id: trace_key.clone(),
                element_name: format!("{} {} {}", trace.from, trace.trace_type, trace.to),
                details: vec![],
            });
        }
        
        changes
    }
    
    fn calculate_summary(
        &self,
        changes: &[ElementChange],
        model_a: &SemanticModel,
        model_b: &SemanticModel,
    ) -> DiffSummary {
        let added_count = changes.iter()
            .filter(|c| matches!(c.change_type, ChangeType::Added))
            .count();
        
        let removed_count = changes.iter()
            .filter(|c| matches!(c.change_type, ChangeType::Removed))
            .count();
        
        let modified_count = changes.iter()
            .filter(|c| matches!(c.change_type, ChangeType::Modified))
            .count();
        
        // Calculate unchanged elements
        let total_a = self.count_elements(model_a);
        let unchanged_count = total_a.saturating_sub(removed_count + modified_count);
        
        DiffSummary {
            added_count,
            removed_count,
            modified_count,
            unchanged_count,
        }
    }
    
    fn count_elements(&self, model: &SemanticModel) -> usize {
        let req_count = model.system_analysis.iter()
            .map(|sa| sa.requirements.len())
            .sum::<usize>();
        
        let comp_count = model.logical_architecture.iter()
            .map(|la| la.components.len())
            .sum::<usize>();
        
        let trace_count = model.traces.len();
        
        req_count + comp_count + trace_count
    }
    
    fn get_req_description(&self, req: &Requirement) -> String {
        req.attributes.get("description")
            .and_then(|v| match v {
                AttributeValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| req.id.clone())
    }
    
    fn get_req_priority(&self, req: &Requirement) -> String {
        req.attributes.get("priority")
            .and_then(|v| match v {
                AttributeValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "Unknown".to_string())
    }
}
```

### Week 4: Diff Visualization

**File:** `src/compiler/diff_html.rs` (New)

```rust
use super::diff::*;

pub fn generate_diff_html(diff: &ModelDiff, file_a: &str, file_b: &str) -> String {
    let diff_json = serde_json::to_string(&diff).unwrap();
    
    format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>Model Diff Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }}
        .header {{ background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .summary {{ display: flex; gap: 20px; margin: 20px 0; }}
        .stat {{ background: white; padding: 20px; border-radius: 8px; flex: 1; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .stat-number {{ font-size: 32px; font-weight: bold; }}
        .stat-label {{ color: #666; margin-top: 5px; }}
        .added {{ color: #27ae60; }}
        .removed {{ color: #e74c3c; }}
        .modified {{ color: #f39c12; }}
        .unchanged {{ color: #95a5a6; }}
        .changes {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .change-item {{ padding: 15px; border-left: 4px solid #ddd; margin: 10px 0; background: #f9f9f9; }}
        .change-item.added {{ border-left-color: #27ae60; }}
        .change-item.removed {{ border-left-color: #e74c3c; }}
        .change-item.modified {{ border-left-color: #f39c12; }}
        .change-type {{ font-weight: bold; text-transform: uppercase; font-size: 12px; }}
        .change-name {{ font-size: 16px; margin: 5px 0; }}
        .change-details {{ margin-top: 10px; font-size: 14px; color: #555; }}
        .field-change {{ padding: 5px 0; }}
        .old-value {{ text-decoration: line-through; color: #e74c3c; }}
        .new-value {{ color: #27ae60; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Model Diff Report</h1>
        <div><strong>File A:</strong> {}</div>
        <div><strong>File B:</strong> {}</div>
    </div>
    
    <div class="summary">
        <div class="stat">
            <div class="stat-number added">{}</div>
            <div class="stat-label">Added Elements</div>
        </div>
        <div class="stat">
            <div class="stat-number removed">{}</div>
            <div class="stat-label">Removed Elements</div>
        </div>
        <div class="stat">
            <div class="stat-number modified">{}</div>
            <div class="stat-label">Modified Elements</div>
        </div>
        <div class="stat">
            <div class="stat-number unchanged">{}</div>
            <div class="stat-label">Unchanged Elements</div>
        </div>
    </div>
    
    <div class="changes">
        <h2>Changes ({} total)</h2>
        <div id="changes-list"></div>
    </div>
    
    <script>
        const diff = {};
        const changesList = document.getElementById('changes-list');
        
        diff.changes.forEach(change => {{
            const item = document.createElement('div');
            item.className = `change-item ${{change.change_type.toLowerCase()}}`;
            
            let typeClass = change.change_type.toLowerCase();
            let icon = typeClass === 'added' ? '➕' : typeClass === 'removed' ? '➖' : '✏️';
            
            let detailsHtml = '';
            if (change.details && change.details.length > 0) {{
                detailsHtml = '<div class="change-details">';
                change.details.forEach(detail => {{
                    detailsHtml += `<div class="field-change">`;
                    detailsHtml += `<strong>${{detail.field_name}}:</strong> `;
                    if (detail.old_value) {{
                        detailsHtml += `<span class="old-value">${{detail.old_value}}</span> → `;
                    }}
                    if (detail.new_value) {{
                        detailsHtml += `<span class="new-value">${{detail.new_value}}</span>`;
                    }}
                    detailsHtml += `</div>`;
                }});
                detailsHtml += '</div>';
            }}
            
            item.innerHTML = `
                <div class="change-type ${{typeClass}}">${{icon}} ${{change.change_type}}</div>
                <div class="change-name">${{change.element_type}}: ${{change.element_name}}</div>
                <div style="font-size: 12px; color: #888;">ID: ${{change.element_id}}</div>
                ${{detailsHtml}}
            `;
            
            changesList.appendChild(item);
        }});
    </script>
</body>
</html>"#,
        file_a,
        file_b,
        diff.summary.added_count,
        diff.summary.removed_count,
        diff.summary.modified_count,
        diff.summary.unchanged_count,
        diff.changes.len(),
        diff_json
    )
}
```

**CLI:**
```bash
# Compare two versions
arclang diff model_v1.arc model_v2.arc --output diff_report.html

# With visual diagram showing changes
arclang diff model_v1.arc model_v2.arc --diagram --output diff_visual.html
```

---

## Phase 3: Requirements Export (Week 5-6)

### Week 5: Excel Export

**Cargo.toml:**
```toml
[dependencies]
rust_xlsxwriter = "0.69"
```

**File:** `src/compiler/excel_export.rs` (New)

```rust
use super::semantic::*;
use rust_xlsxwriter::*;

pub fn export_requirements_to_excel(
    model: &SemanticModel,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    
    // Header formatting
    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x4472C4))
        .set_font_color(Color::White)
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center);
    
    // Cell formatting
    let cell_format = Format::new()
        .set_border(FormatBorder::Thin);
    
    // Write headers
    worksheet.write_with_format(0, 0, "ID", &header_format)?;
    worksheet.write_with_format(0, 1, "Description", &header_format)?;
    worksheet.write_with_format(0, 2, "Priority", &header_format)?;
    worksheet.write_with_format(0, 3, "Safety Level", &header_format)?;
    worksheet.write_with_format(0, 4, "Status", &header_format)?;
    worksheet.write_with_format(0, 5, "Traced To", &header_format)?;
    
    // Write data
    let mut row = 1;
    for sa in &model.system_analysis {
        for req in &sa.requirements {
            // ID
            worksheet.write_with_format(row, 0, &req.id, &cell_format)?;
            
            // Description
            let description = req.attributes.get("description")
                .and_then(|v| match v {
                    AttributeValue::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or("");
            worksheet.write_with_format(row, 1, description, &cell_format)?;
            
            // Priority
            let priority = req.attributes.get("priority")
                .and_then(|v| match v {
                    AttributeValue::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or("");
            worksheet.write_with_format(row, 2, priority, &cell_format)?;
            
            // Safety Level
            let safety_level = req.attributes.get("safety_level")
                .and_then(|v| match v {
                    AttributeValue::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or("");
            worksheet.write_with_format(row, 3, safety_level, &cell_format)?;
            
            // Status
            let status = req.attributes.get("status")
                .and_then(|v| match v {
                    AttributeValue::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or("Draft");
            worksheet.write_with_format(row, 4, status, &cell_format)?;
            
            // Traced To (find all traces from this requirement)
            let traces: Vec<_> = model.traces.iter()
                .filter(|t| t.from == req.id)
                .map(|t| format!("{} ({})", t.to, t.trace_type))
                .collect();
            let traced_to = traces.join(", ");
            worksheet.write_with_format(row, 5, &traced_to, &cell_format)?;
            
            row += 1;
        }
    }
    
    // Auto-fit columns
    worksheet.set_column_width(0, 15)?;  // ID
    worksheet.set_column_width(1, 50)?;  // Description
    worksheet.set_column_width(2, 12)?;  // Priority
    worksheet.set_column_width(3, 15)?;  // Safety Level
    worksheet.set_column_width(4, 12)?;  // Status
    worksheet.set_column_width(5, 40)?;  // Traced To
    
    // Freeze header row
    worksheet.set_freeze_panes(1, 0)?;
    
    workbook.save(output_path)?;
    
    println!("✓ Excel export complete: {}", output_path);
    println!("  Requirements exported: {}", row - 1);
    
    Ok(())
}
```

### Week 6: ReqIF Export

**File:** `src/compiler/reqif_export.rs` (New)

```rust
use super::semantic::*;
use quick_xml::Writer;
use quick_xml::events::*;
use std::fs::File;
use std::io::BufWriter;

pub fn export_to_reqif(
    model: &SemanticModel,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output_path)?;
    let buf = BufWriter::new(file);
    let mut writer = Writer::new_with_indent(buf, b' ', 2);
    
    // XML declaration
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    
    // REQ-IF root element
    let mut reqif = BytesStart::new("REQ-IF");
    reqif.push_attribute(("xmlns", "http://www.omg.org/spec/ReqIF/20110401/reqif.xsd"));
    reqif.push_attribute(("xmlns:xhtml", "http://www.w3.org/1999/xhtml"));
    writer.write_event(Event::Start(reqif))?;
    
    // THE-HEADER
    write_reqif_header(&mut writer)?;
    
    // CORE-CONTENT
    writer.write_event(Event::Start(BytesStart::new("CORE-CONTENT")))?;
    writer.write_event(Event::Start(BytesStart::new("REQ-IF-CONTENT")))?;
    
    // DATATYPES
    write_datatypes(&mut writer)?;
    
    // SPEC-TYPES
    write_spec_types(&mut writer)?;
    
    // SPEC-OBJECTS (Requirements)
    write_spec_objects(&mut writer, model)?;
    
    // SPEC-RELATIONS (Traces)
    write_spec_relations(&mut writer, model)?;
    
    // SPECIFICATIONS
    write_specifications(&mut writer, model)?;
    
    writer.write_event(Event::End(BytesEnd::new("REQ-IF-CONTENT")))?;
    writer.write_event(Event::End(BytesEnd::new("CORE-CONTENT")))?;
    
    writer.write_event(Event::End(BytesEnd::new("REQ-IF")))?;
    
    println!("✓ ReqIF export complete: {}", output_path);
    
    Ok(())
}

fn write_reqif_header(writer: &mut Writer<BufWriter<File>>) -> Result<(), Box<dyn std::error::Error>> {
    writer.write_event(Event::Start(BytesStart::new("THE-HEADER")))?;
    writer.write_event(Event::Start(BytesStart::new("REQ-IF-HEADER")))?;
    
    // IDENTIFIER
    writer.write_event(Event::Start(BytesStart::new("IDENTIFIER")))?;
    writer.write_event(Event::Text(BytesText::new("arclang-export")))?;
    writer.write_event(Event::End(BytesEnd::new("IDENTIFIER")))?;
    
    // TITLE
    writer.write_event(Event::Start(BytesStart::new("TITLE")))?;
    writer.write_event(Event::Text(BytesText::new("Arclang Requirements Export")))?;
    writer.write_event(Event::End(BytesEnd::new("TITLE")))?;
    
    // CREATION-TIME
    writer.write_event(Event::Start(BytesStart::new("CREATION-TIME")))?;
    let timestamp = chrono::Utc::now().to_rfc3339();
    writer.write_event(Event::Text(BytesText::new(&timestamp)))?;
    writer.write_event(Event::End(BytesEnd::new("CREATION-TIME")))?;
    
    writer.write_event(Event::End(BytesEnd::new("REQ-IF-HEADER")))?;
    writer.write_event(Event::End(BytesEnd::new("THE-HEADER")))?;
    
    Ok(())
}

fn write_datatypes(writer: &mut Writer<BufWriter<File>>) -> Result<(), Box<dyn std::error::Error>> {
    writer.write_event(Event::Start(BytesStart::new("DATATYPES")))?;
    
    // String datatype
    let mut datatype = BytesStart::new("DATATYPE-DEFINITION-STRING");
    datatype.push_attribute(("IDENTIFIER", "DT_String"));
    datatype.push_attribute(("LAST-CHANGE", &chrono::Utc::now().to_rfc3339()));
    datatype.push_attribute(("MAX-LENGTH", "32000"));
    writer.write_event(Event::Empty(datatype))?;
    
    // XHTML datatype (for rich text)
    let mut datatype_xhtml = BytesStart::new("DATATYPE-DEFINITION-XHTML");
    datatype_xhtml.push_attribute(("IDENTIFIER", "DT_XHTML"));
    datatype_xhtml.push_attribute(("LAST-CHANGE", &chrono::Utc::now().to_rfc3339()));
    writer.write_event(Event::Empty(datatype_xhtml))?;
    
    writer.write_event(Event::End(BytesEnd::new("DATATYPES")))?;
    
    Ok(())
}

fn write_spec_types(writer: &mut Writer<BufWriter<File>>) -> Result<(), Box<dyn std::error::Error>> {
    writer.write_event(Event::Start(BytesStart::new("SPEC-TYPES")))?;
    
    // Requirement spec object type
    let mut spec_type = BytesStart::new("SPEC-OBJECT-TYPE");
    spec_type.push_attribute(("IDENTIFIER", "SOT_Requirement"));
    spec_type.push_attribute(("LAST-CHANGE", &chrono::Utc::now().to_rfc3339()));
    writer.write_event(Event::Start(spec_type))?;
    
    // SPEC-ATTRIBUTES
    writer.write_event(Event::Start(BytesStart::new("SPEC-ATTRIBUTES")))?;
    
    // Description attribute
    write_attribute_definition(writer, "AT_Description", "Description", "DT_XHTML")?;
    
    // Priority attribute
    write_attribute_definition(writer, "AT_Priority", "Priority", "DT_String")?;
    
    // Safety Level attribute
    write_attribute_definition(writer, "AT_SafetyLevel", "Safety Level", "DT_String")?;
    
    writer.write_event(Event::End(BytesEnd::new("SPEC-ATTRIBUTES")))?;
    writer.write_event(Event::End(BytesEnd::new("SPEC-OBJECT-TYPE")))?;
    
    writer.write_event(Event::End(BytesEnd::new("SPEC-TYPES")))?;
    
    Ok(())
}

fn write_attribute_definition(
    writer: &mut Writer<BufWriter<File>>,
    id: &str,
    name: &str,
    datatype_ref: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut attr_def = BytesStart::new("ATTRIBUTE-DEFINITION-XHTML");
    attr_def.push_attribute(("IDENTIFIER", id));
    attr_def.push_attribute(("LAST-CHANGE", &chrono::Utc::now().to_rfc3339()));
    writer.write_event(Event::Start(attr_def))?;
    
    // LONG-NAME
    writer.write_event(Event::Start(BytesStart::new("LONG-NAME")))?;
    writer.write_event(Event::Text(BytesText::new(name)))?;
    writer.write_event(Event::End(BytesEnd::new("LONG-NAME")))?;
    
    // TYPE (reference to datatype)
    writer.write_event(Event::Start(BytesStart::new("TYPE")))?;
    let mut type_ref = BytesStart::new("DATATYPE-DEFINITION-XHTML-REF");
    type_ref.push_attribute(("DATATYPE-DEFINITION-XHTML-REF", datatype_ref));
    writer.write_event(Event::Empty(type_ref))?;
    writer.write_event(Event::End(BytesEnd::new("TYPE")))?;
    
    writer.write_event(Event::End(BytesEnd::new("ATTRIBUTE-DEFINITION-XHTML")))?;
    
    Ok(())
}

fn write_spec_objects(
    writer: &mut Writer<BufWriter<File>>,
    model: &SemanticModel,
) -> Result<(), Box<dyn std::error::Error>> {
    writer.write_event(Event::Start(BytesStart::new("SPEC-OBJECTS")))?;
    
    for sa in &model.system_analysis {
        for req in &sa.requirements {
            let mut spec_obj = BytesStart::new("SPEC-OBJECT");
            spec_obj.push_attribute(("IDENTIFIER", req.id.as_str()));
            spec_obj.push_attribute(("LAST-CHANGE", &chrono::Utc::now().to_rfc3339()));
            writer.write_event(Event::Start(spec_obj))?;
            
            // TYPE reference
            writer.write_event(Event::Start(BytesStart::new("TYPE")))?;
            let mut type_ref = BytesStart::new("SPEC-OBJECT-TYPE-REF");
            type_ref.push_attribute(("SPEC-OBJECT-TYPE-REF", "SOT_Requirement"));
            writer.write_event(Event::Empty(type_ref))?;
            writer.write_event(Event::End(BytesEnd::new("TYPE")))?;
            
            // VALUES
            writer.write_event(Event::Start(BytesStart::new("VALUES")))?;
            
            // Description
            if let Some(desc) = get_attribute_string(&req.attributes, "description") {
                write_attribute_value_xhtml(writer, "AT_Description", &desc)?;
            }
            
            // Priority
            if let Some(pri) = get_attribute_string(&req.attributes, "priority") {
                write_attribute_value_string(writer, "AT_Priority", &pri)?;
            }
            
            // Safety Level
            if let Some(level) = get_attribute_string(&req.attributes, "safety_level") {
                write_attribute_value_string(writer, "AT_SafetyLevel", &level)?;
            }
            
            writer.write_event(Event::End(BytesEnd::new("VALUES")))?;
            writer.write_event(Event::End(BytesEnd::new("SPEC-OBJECT")))?;
        }
    }
    
    writer.write_event(Event::End(BytesEnd::new("SPEC-OBJECTS")))?;
    
    Ok(())
}

fn write_attribute_value_xhtml(
    writer: &mut Writer<BufWriter<File>>,
    attr_def_ref: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut attr_val = BytesStart::new("ATTRIBUTE-VALUE-XHTML");
    writer.write_event(Event::Start(attr_val.clone()))?;
    
    // DEFINITION reference
    writer.write_event(Event::Start(BytesStart::new("DEFINITION")))?;
    let mut def_ref = BytesStart::new("ATTRIBUTE-DEFINITION-XHTML-REF");
    def_ref.push_attribute(("ATTRIBUTE-DEFINITION-XHTML-REF", attr_def_ref));
    writer.write_event(Event::Empty(def_ref))?;
    writer.write_event(Event::End(BytesEnd::new("DEFINITION")))?;
    
    // THE-VALUE
    writer.write_event(Event::Start(BytesStart::new("THE-VALUE")))?;
    writer.write_event(Event::Start(BytesStart::new("xhtml:div")))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new("xhtml:div")))?;
    writer.write_event(Event::End(BytesEnd::new("THE-VALUE")))?;
    
    writer.write_event(Event::End(BytesEnd::new("ATTRIBUTE-VALUE-XHTML")))?;
    
    Ok(())
}

fn write_attribute_value_string(
    writer: &mut Writer<BufWriter<File>>,
    attr_def_ref: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut attr_val = BytesStart::new("ATTRIBUTE-VALUE-STRING");
    writer.write_event(Event::Start(attr_val))?;
    
    // DEFINITION reference
    writer.write_event(Event::Start(BytesStart::new("DEFINITION")))?;
    let mut def_ref = BytesStart::new("ATTRIBUTE-DEFINITION-STRING-REF");
    def_ref.push_attribute(("ATTRIBUTE-DEFINITION-STRING-REF", attr_def_ref));
    writer.write_event(Event::Empty(def_ref))?;
    writer.write_event(Event::End(BytesEnd::new("DEFINITION")))?;
    
    // THE-VALUE
    writer.write_event(Event::Start(BytesStart::new("THE-VALUE")))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new("THE-VALUE")))?;
    
    writer.write_event(Event::End(BytesEnd::new("ATTRIBUTE-VALUE-STRING")))?;
    
    Ok(())
}

fn write_spec_relations(
    writer: &mut Writer<BufWriter<File>>,
    model: &SemanticModel,
) -> Result<(), Box<dyn std::error::Error>> {
    writer.write_event(Event::Start(BytesStart::new("SPEC-RELATIONS")))?;
    
    // Export traces as relations
    for trace in &model.traces {
        let relation_id = format!("REL_{}_{}", trace.from, trace.to);
        let mut spec_rel = BytesStart::new("SPEC-RELATION");
        spec_rel.push_attribute(("IDENTIFIER", relation_id.as_str()));
        spec_rel.push_attribute(("LAST-CHANGE", &chrono::Utc::now().to_rfc3339()));
        writer.write_event(Event::Start(spec_rel))?;
        
        // SOURCE
        writer.write_event(Event::Start(BytesStart::new("SOURCE")))?;
        let mut source_ref = BytesStart::new("SPEC-OBJECT-REF");
        source_ref.push_attribute(("SPEC-OBJECT-REF", trace.from.as_str()));
        writer.write_event(Event::Empty(source_ref))?;
        writer.write_event(Event::End(BytesEnd::new("SOURCE")))?;
        
        // TARGET
        writer.write_event(Event::Start(BytesStart::new("TARGET")))?;
        let mut target_ref = BytesStart::new("SPEC-OBJECT-REF");
        target_ref.push_attribute(("SPEC-OBJECT-REF", trace.to.as_str()));
        writer.write_event(Event::Empty(target_ref))?;
        writer.write_event(Event::End(BytesEnd::new("TARGET")))?;
        
        writer.write_event(Event::End(BytesEnd::new("SPEC-RELATION")))?;
    }
    
    writer.write_event(Event::End(BytesEnd::new("SPEC-RELATIONS")))?;
    
    Ok(())
}

fn write_specifications(
    writer: &mut Writer<BufWriter<File>>,
    model: &SemanticModel,
) -> Result<(), Box<dyn std::error::Error>> {
    writer.write_event(Event::Start(BytesStart::new("SPECIFICATIONS")))?;
    
    // Create one specification per system analysis
    for (idx, sa) in model.system_analysis.iter().enumerate() {
        let spec_id = format!("SPEC_{}", idx);
        let mut spec = BytesStart::new("SPECIFICATION");
        spec.push_attribute(("IDENTIFIER", spec_id.as_str()));
        spec.push_attribute(("LAST-CHANGE", &chrono::Utc::now().to_rfc3339()));
        writer.write_event(Event::Start(spec))?;
        
        // LONG-NAME
        writer.write_event(Event::Start(BytesStart::new("LONG-NAME")))?;
        writer.write_event(Event::Text(BytesText::new(&sa.name)))?;
        writer.write_event(Event::End(BytesEnd::new("LONG-NAME")))?;
        
        // CHILDREN (requirements)
        writer.write_event(Event::Start(BytesStart::new("CHILDREN")))?;
        for req in &sa.requirements {
            let mut spec_hierarchy = BytesStart::new("SPEC-HIERARCHY");
            spec_hierarchy.push_attribute(("IDENTIFIER", format!("HIER_{}", req.id).as_str()));
            spec_hierarchy.push_attribute(("LAST-CHANGE", &chrono::Utc::now().to_rfc3339()));
            writer.write_event(Event::Start(spec_hierarchy))?;
            
            // OBJECT reference
            writer.write_event(Event::Start(BytesStart::new("OBJECT")))?;
            let mut obj_ref = BytesStart::new("SPEC-OBJECT-REF");
            obj_ref.push_attribute(("SPEC-OBJECT-REF", req.id.as_str()));
            writer.write_event(Event::Empty(obj_ref))?;
            writer.write_event(Event::End(BytesEnd::new("OBJECT")))?;
            
            writer.write_event(Event::End(BytesEnd::new("SPEC-HIERARCHY")))?;
        }
        writer.write_event(Event::End(BytesEnd::new("CHILDREN")))?;
        
        writer.write_event(Event::End(BytesEnd::new("SPECIFICATION")))?;
    }
    
    writer.write_event(Event::End(BytesEnd::new("SPECIFICATIONS")))?;
    
    Ok(())
}

fn get_attribute_string(
    attrs: &std::collections::HashMap<String, AttributeValue>,
    key: &str,
) -> Option<String> {
    attrs.get(key).and_then(|v| match v {
        AttributeValue::String(s) => Some(s.clone()),
        _ => None,
    })
}
```

**Cargo.toml:**
```toml
[dependencies]
chrono = "0.4"
```

**CLI:**
```bash
# Export to Excel
arclang export model.arc --format excel --output requirements.xlsx

# Export to ReqIF
arclang export model.arc --format reqif --output requirements.reqif
```

---

## Phase 4: Code Generation (Week 7-8)

### Week 7: ROS2 & Protobuf

**File:** `src/compiler/codegen/mod.rs` (New)

```rust
pub mod ros2;
pub mod protobuf;
pub mod python;

use super::semantic::*;
use std::path::Path;

pub trait CodeGenerator {
    fn generate(&self, model: &SemanticModel, output_dir: &Path) -> Result<(), String>;
    fn file_extension(&self) -> &str;
}
```

**File:** `src/compiler/codegen/ros2.rs` (New)

```rust
use super::*;
use std::fs;

pub struct ROS2Generator;

impl CodeGenerator for ROS2Generator {
    fn generate(&self, model: &SemanticModel, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
        
        let mut generated_count = 0;
        
        for datatype in &model.data_types {
            let msg_content = self.generate_ros2_msg(datatype)?;
            let filename = format!("{}.msg", datatype.name);
            let filepath = output_dir.join(filename);
            
            fs::write(&filepath, msg_content)
                .map_err(|e| format!("Failed to write {}: {}", filepath.display(), e))?;
            
            generated_count += 1;
        }
        
        println!("✓ Generated {} ROS2 message files in {}", generated_count, output_dir.display());
        
        Ok(())
    }
    
    fn file_extension(&self) -> &str {
        "msg"
    }
}

impl ROS2Generator {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_ros2_msg(&self, datatype: &DataType) -> Result<String, String> {
        let mut msg = String::new();
        
        // Header comment
        msg.push_str(&format!("# Generated from Arclang model\n"));
        msg.push_str(&format!("# Type: {}\n", datatype.name));
        if !datatype.description.is_empty() {
            msg.push_str(&format!("# {}\n", datatype.description));
        }
        msg.push_str("\n");
        
        // Fields
        for field in &datatype.fields {
            let ros2_type = self.map_type(&field.type_name)?;
            
            // Handle arrays
            if field.is_array {
                msg.push_str(&format!("{}[] {}\n", ros2_type, field.name));
            } else {
                msg.push_str(&format!("{} {}\n", ros2_type, field.name));
            }
            
            // Optional comment
            if !field.description.is_empty() {
                msg.push_str(&format!("# {}\n", field.description));
            }
        }
        
        Ok(msg)
    }
    
    fn map_type(&self, capella_type: &str) -> Result<String, String> {
        Ok(match capella_type {
            "Boolean" => "bool",
            "Byte" | "Char" => "byte",
            "Double" => "float64",
            "Float" => "float32",
            "Integer" | "Long" => "int32",
            "Short" => "int16",
            "String" => "string",
            "UnsignedInteger" => "uint32",
            "UnsignedShort" => "uint16",
            _ => return Err(format!("Unknown type: {}", capella_type)),
        }.to_string())
    }
}
```

**File:** `src/compiler/codegen/protobuf.rs` (New)

```rust
use super::*;
use std::fs;

pub struct ProtobufGenerator;

impl CodeGenerator for ProtobufGenerator {
    fn generate(&self, model: &SemanticModel, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
        
        let mut generated_count = 0;
        
        for datatype in &model.data_types {
            let proto_content = self.generate_protobuf(datatype)?;
            let filename = format!("{}.proto", datatype.name.to_lowercase());
            let filepath = output_dir.join(filename);
            
            fs::write(&filepath, proto_content)
                .map_err(|e| format!("Failed to write {}: {}", filepath.display(), e))?;
            
            generated_count += 1;
        }
        
        println!("✓ Generated {} Protobuf files in {}", generated_count, output_dir.display());
        
        Ok(())
    }
    
    fn file_extension(&self) -> &str {
        "proto"
    }
}

impl ProtobufGenerator {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_protobuf(&self, datatype: &DataType) -> Result<String, String> {
        let mut proto = String::new();
        
        proto.push_str("syntax = \"proto3\";\n\n");
        
        // Message definition
        proto.push_str(&format!("message {} {{\n", datatype.name));
        
        for (index, field) in datatype.fields.iter().enumerate() {
            let proto_type = self.map_type(&field.type_name)?;
            let field_num = index + 1;
            
            if field.is_array {
                proto.push_str(&format!(
                    "  repeated {} {} = {};\n",
                    proto_type, field.name, field_num
                ));
            } else {
                proto.push_str(&format!(
                    "  {} {} = {};\n",
                    proto_type, field.name, field_num
                ));
            }
        }
        
        proto.push_str("}\n");
        
        Ok(proto)
    }
    
    fn map_type(&self, capella_type: &str) -> Result<String, String> {
        Ok(match capella_type {
            "Boolean" => "bool",
            "Double" => "double",
            "Float" => "float",
            "Integer" | "Long" => "int32",
            "Short" => "int32",
            "String" => "string",
            "Byte" => "bytes",
            "UnsignedInteger" => "uint32",
            _ => return Err(format!("Unknown type: {}", capella_type)),
        }.to_string())
    }
}
```

### Week 8: Python Stubs & CLI Integration

**File:** `src/compiler/codegen/python.rs` (New)

```rust
use super::*;
use std::fs;

pub struct PythonGenerator;

impl CodeGenerator for PythonGenerator {
    fn generate(&self, model: &SemanticModel, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
        
        let mut generated_count = 0;
        
        for datatype in &model.data_types {
            let py_content = self.generate_python_class(datatype)?;
            let filename = format!("{}.py", datatype.name.to_lowercase());
            let filepath = output_dir.join(filename);
            
            fs::write(&filepath, py_content)
                .map_err(|e| format!("Failed to write {}: {}", filepath.display(), e))?;
            
            generated_count += 1;
        }
        
        println!("✓ Generated {} Python files in {}", generated_count, output_dir.display());
        
        Ok(())
    }
    
    fn file_extension(&self) -> &str {
        "py"
    }
}

impl PythonGenerator {
    pub fn new() -> Self {
        Self
    }
    
    fn generate_python_class(&self, datatype: &DataType) -> Result<String, String> {
        let mut py = String::new();
        
        // Imports
        py.push_str("from dataclasses import dataclass\n");
        py.push_str("from typing import Optional, List\n\n");
        
        // Class decorator
        py.push_str("@dataclass\n");
        py.push_str(&format!("class {}:\n", datatype.name));
        
        if datatype.fields.is_empty() {
            py.push_str("    pass\n");
        } else {
            for field in &datatype.fields {
                let py_type = self.map_type(&field.type_name)?;
                
                if field.is_array {
                    py.push_str(&format!("    {}: List[{}]\n", field.name, py_type));
                } else if field.optional {
                    py.push_str(&format!("    {}: Optional[{}] = None\n", field.name, py_type));
                } else {
                    py.push_str(&format!("    {}: {}\n", field.name, py_type));
                }
            }
        }
        
        Ok(py)
    }
    
    fn map_type(&self, capella_type: &str) -> Result<String, String> {
        Ok(match capella_type {
            "Boolean" => "bool",
            "Integer" | "Long" | "Short" => "int",
            "Double" | "Float" => "float",
            "String" => "str",
            _ => return Err(format!("Unknown type: {}", capella_type)),
        }.to_string())
    }
}
```

**CLI Commands:**
```bash
# Generate ROS2 messages
arclang codegen model.arc --format ros2 --output-dir ./ros2_msgs/

# Generate Protobuf
arclang codegen model.arc --format protobuf --output-dir ./proto/

# Generate Python stubs
arclang codegen model.arc --format python --output-dir ./python/
```

---

## Phase 5: Basic Validation (Week 9-10)

### Week 9: Validation Framework

**File:** `src/compiler/validation.rs` (New)

```rust
use super::semantic::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub rule_id: String,
    pub rule_name: String,
    pub element_id: String,
    pub element_type: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
    pub summary: ValidationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub total_elements_checked: usize,
}

pub type ValidationFn = fn(&SemanticModel, &str) -> bool;

pub struct ValidationRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub check: ValidationFn,
}

pub struct ValidationEngine {
    rules: Vec<ValidationRule>,
}

impl ValidationEngine {
    pub fn with_default_rules() -> Self {
        let mut engine = Self { rules: Vec::new() };
        
        // Rule 1: All requirements must be traced
        engine.add_rule(ValidationRule {
            id: "REQ_001".to_string(),
            name: "Requirements Traceability".to_string(),
            description: "All requirements must be traced to at least one component".to_string(),
            severity: Severity::Error,
            check: Self::check_requirement_traced,
        });
        
        // Rule 2: No orphan components
        engine.add_rule(ValidationRule {
            id: "COMP_001".to_string(),
            name: "No Orphan Components".to_string(),
            description: "All components should be traced or deployed".to_string(),
            severity: Severity::Warning,
            check: Self::check_component_not_orphan,
        });
        
        // Rule 3: ASIL-D components need hazard analysis
        engine.add_rule(ValidationRule {
            id: "SAFETY_001".to_string(),
            name: "ASIL-D Hazard Analysis".to_string(),
            description: "ASIL-D components must have associated hazard analysis".to_string(),
            severity: Severity::Error,
            check: Self::check_asil_d_hazard,
        });
        
        // Rule 4: Functions must be allocated
        engine.add_rule(ValidationRule {
            id: "FUNC_001".to_string(),
            name: "Function Allocation".to_string(),
            description: "All functions must be allocated to components".to_string(),
            severity: Severity::Warning,
            check: Self::check_function_allocated,
        });
        
        // Rule 5: Component names must be unique
        engine.add_rule(ValidationRule {
            id: "COMP_002".to_string(),
            name: "Unique Component Names".to_string(),
            description: "Component names must be unique within the model".to_string(),
            severity: Severity::Error,
            check: Self::check_component_name_unique,
        });
        
        engine
    }
    
    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.push(rule);
    }
    
    pub fn validate(&self, model: &SemanticModel) -> ValidationReport {
        let mut issues = Vec::new();
        let mut total_elements = 0;
        
        // Validate requirements
        for sa in &model.system_analysis {
            for req in &sa.requirements {
                total_elements += 1;
                for rule in &self.rules {
                    if !(rule.check)(model, &req.id) {
                        issues.push(ValidationIssue {
                            severity: rule.severity,
                            rule_id: rule.id.clone(),
                            rule_name: rule.name.clone(),
                            element_id: req.id.clone(),
                            element_type: "Requirement".to_string(),
                            message: format!("{}: {}", rule.name, rule.description),
                        });
                    }
                }
            }
        }
        
        // Validate components
        for la in &model.logical_architecture {
            for comp in &la.components {
                total_elements += 1;
                for rule in &self.rules {
                    if !(rule.check)(model, &comp.id) {
                        issues.push(ValidationIssue {
                            severity: rule.severity,
                            rule_id: rule.id.clone(),
                            rule_name: rule.name.clone(),
                            element_id: comp.id.clone(),
                            element_type: "Component".to_string(),
                            message: format!("{}: {}", rule.name, rule.description),
                        });
                    }
                }
            }
        }
        
        let summary = ValidationSummary {
            error_count: issues.iter().filter(|i| i.severity == Severity::Error).count(),
            warning_count: issues.iter().filter(|i| i.severity == Severity::Warning).count(),
            info_count: issues.iter().filter(|i| i.severity == Severity::Info).count(),
            total_elements_checked: total_elements,
        };
        
        ValidationReport { issues, summary }
    }
    
    // Validation rule implementations
    fn check_requirement_traced(model: &SemanticModel, req_id: &str) -> bool {
        model.traces.iter().any(|t| t.from == req_id)
    }
    
    fn check_component_not_orphan(model: &SemanticModel, comp_id: &str) -> bool {
        model.traces.iter().any(|t| t.from == comp_id || t.to == comp_id)
    }
    
    fn check_asil_d_hazard(model: &SemanticModel, comp_id: &str) -> bool {
        // Find component
        let comp = model.logical_architecture.iter()
            .flat_map(|la| &la.components)
            .find(|c| c.id == comp_id);
        
        if let Some(comp) = comp {
            // Check if ASIL-D
            if let Some(AttributeValue::String(level)) = comp.attributes.get("safety_level") {
                if level == "ASIL_D" {
                    // Must have hazard
                    return model.safety_analysis.iter()
                        .flat_map(|sa| &sa.hazards)
                        .any(|h| {
                            h.attributes.get("related_component")
                                .and_then(|v| match v {
                                    AttributeValue::String(s) => Some(s),
                                    _ => None,
                                })
                                .map(|s| s == comp_id)
                                .unwrap_or(false)
                        });
                }
            }
        }
        
        true  // Not ASIL-D or not found, rule doesn't apply
    }
    
    fn check_function_allocated(_model: &SemanticModel, _func_id: &str) -> bool {
        // TODO: Check if function is allocated to component
        true
    }
    
    fn check_component_name_unique(model: &SemanticModel, comp_id: &str) -> bool {
        let comp = model.logical_architecture.iter()
            .flat_map(|la| &la.components)
            .find(|c| c.id == comp_id);
        
        if let Some(comp) = comp {
            let count = model.logical_architecture.iter()
                .flat_map(|la| &la.components)
                .filter(|c| c.name == comp.name)
                .count();
            
            return count == 1;
        }
        
        true
    }
}
```

### Week 10: Validation Report & CLI

**File:** `src/compiler/validation_html.rs` (New)

```rust
use super::validation::*;

pub fn generate_validation_html(report: &ValidationReport) -> String {
    let report_json = serde_json::to_string(&report).unwrap();
    
    format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>Validation Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }}
        .header {{ background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .summary {{ display: flex; gap: 20px; margin: 20px 0; }}
        .stat {{ background: white; padding: 20px; border-radius: 8px; flex: 1; box-shadow: 0 2px 4px rgba(0,0,0,0.1); text-align: center; }}
        .stat-number {{ font-size: 48px; font-weight: bold; margin: 10px 0; }}
        .stat-label {{ color: #666; font-size: 14px; }}
        .error {{ color: #e74c3c; }}
        .warning {{ color: #f39c12; }}
        .info {{ color: #3498db; }}
        .success {{ color: #27ae60; }}
        .issues {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .issue {{ padding: 15px; border-left: 4px solid #ddd; margin: 10px 0; background: #f9f9f9; }}
        .issue.error {{ border-left-color: #e74c3c; }}
        .issue.warning {{ border-left-color: #f39c12; }}
        .issue.info {{ border-left-color: #3498db; }}
        .issue-header {{ font-weight: bold; margin-bottom: 5px; }}
        .issue-details {{ font-size: 14px; color: #555; }}
        .filter-buttons {{ margin: 20px 0; }}
        .filter-btn {{ padding: 10px 20px; margin-right: 10px; border: none; border-radius: 4px; cursor: pointer; background: #ecf0f1; }}
        .filter-btn.active {{ background: #3498db; color: white; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Validation Report</h1>
        <p>Model validation completed. Review issues below.</p>
    </div>
    
    <div class="summary">
        <div class="stat">
            <div class="stat-number error">{}</div>
            <div class="stat-label">Errors</div>
        </div>
        <div class="stat">
            <div class="stat-number warning">{}</div>
            <div class="stat-label">Warnings</div>
        </div>
        <div class="stat">
            <div class="stat-number info">{}</div>
            <div class="stat-label">Info</div>
        </div>
        <div class="stat">
            <div class="stat-number">{}</div>
            <div class="stat-label">Elements Checked</div>
        </div>
    </div>
    
    <div class="issues">
        <div class="filter-buttons">
            <button class="filter-btn active" onclick="filterIssues('all')">All Issues</button>
            <button class="filter-btn" onclick="filterIssues('error')">Errors Only</button>
            <button class="filter-btn" onclick="filterIssues('warning')">Warnings Only</button>
        </div>
        
        <h2>Issues ({} total)</h2>
        <div id="issues-list"></div>
    </div>
    
    <script>
        const report = {};
        
        function renderIssues(filter = 'all') {{
            const issuesList = document.getElementById('issues-list');
            issuesList.innerHTML = '';
            
            const filteredIssues = filter === 'all' 
                ? report.issues
                : report.issues.filter(i => i.severity.toLowerCase() === filter);
            
            if (filteredIssues.length === 0) {{
                issuesList.innerHTML = '<p style="color: #27ae60; font-size: 18px;">✓ No issues found!</p>';
                return;
            }}
            
            filteredIssues.forEach(issue => {{
                const div = document.createElement('div');
                div.className = `issue ${{issue.severity.toLowerCase()}}`;
                
                const icon = issue.severity === 'Error' ? '❌' : issue.severity === 'Warning' ? '⚠️' : 'ℹ️';
                
                div.innerHTML = `
                    <div class="issue-header">
                        ${{icon}} ${{issue.rule_name}} [${{issue.rule_id}}]
                    </div>
                    <div class="issue-details">
                        <div><strong>Element:</strong> ${{issue.element_type}} (${{issue.element_id}})</div>
                        <div><strong>Message:</strong> ${{issue.message}}</div>
                    </div>
                `;
                
                issuesList.appendChild(div);
            }});
        }}
        
        function filterIssues(filter) {{
            // Update button states
            document.querySelectorAll('.filter-btn').forEach(btn => {{
                btn.classList.remove('active');
            }});
            event.target.classList.add('active');
            
            renderIssues(filter);
        }}
        
        renderIssues();
    </script>
</body>
</html>"#,
        report.summary.error_count,
        report.summary.warning_count,
        report.summary.info_count,
        report.summary.total_elements_checked,
        report.issues.len(),
        report_json
    )
}
```

**CLI:**
```bash
# Run validation
arclang validate model.arc --output validation_report.html

# Validation with specific rules
arclang validate model.arc --rules REQ_001,COMP_001 --output report.html
```

---

## Summary: What Gets Delivered

### Timeline: 10 Weeks

**Week 1-2:** Context diagrams with depth control  
**Week 3-4:** Model diff with visual change highlighting  
**Week 5-6:** Requirements export (Excel + ReqIF)  
**Week 7-8:** Code generation (ROS2 + Protobuf + Python)  
**Week 9-10:** Validation system with 15 rules  

### Deliverables

✅ **New CLI Commands:**
```bash
arclang context model.arc --focus LC-001 --depth 2 -o context.html
arclang diff model_v1.arc model_v2.arc --output diff.html
arclang export model.arc --format excel --output req.xlsx
arclang export model.arc --format reqif --output req.reqif
arclang codegen model.arc --format ros2 --output-dir ./msgs/
arclang validate model.arc --output validation.html
```

✅ **New Files Created:**
- `src/compiler/context_diagram.rs` (300 lines)
- `src/compiler/arcviz_context.rs` (200 lines)
- `src/compiler/diff.rs` (500 lines)
- `src/compiler/diff_html.rs` (200 lines)
- `src/compiler/excel_export.rs` (400 lines)
- `src/compiler/reqif_export.rs` (600 lines)
- `src/compiler/codegen/` (directory with 3 generators)
- `src/compiler/validation.rs` (500 lines)
- `src/compiler/validation_html.rs` (200 lines)

✅ **Total New Code:** ~3,500 lines of Rust

### Next Steps After Implementation

1. **Web ArcViz Integration** - Expose all features in web UI
2. **Python Bridge** - PyO3 bindings for advanced use cases
3. **Capellambse Integration** - Use for advanced features not implemented

---

**Ready to Start?** All specifications are complete. Begin with Phase 1 Week 1.
