//! LEGACY: Basic ArcViz with diagonal connectors
//! For production use, prefer arcviz_ultimate_routing (arc-viz-ultimate format)

use super::semantic::SemanticModel;
use super::CompilerError;
use std::collections::HashMap;

#[derive(Debug)]
struct LayoutInfo {
    width: u32,
    height: u32,
    components: Vec<ComponentPosition>,
}

#[derive(Debug)]
struct ComponentPosition {
    id: String,
    name: String,
    x: u32,
    y: u32,
    layer: String,
}

fn compute_layout(model: &SemanticModel) -> LayoutInfo {
    let mut components = Vec::new();
    let comp_count = model.components.len();
    
    if comp_count == 0 {
        return LayoutInfo {
            width: 1400,
            height: 800,
            components: vec![],
        };
    }
    
    // Group by level (operational_analysis, system_analysis, logical_architecture, etc.)
    let mut layers: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for comp in &model.components {
        let category = comp.level.clone();
        let id = comp.id.clone();
        let name = comp.name.clone();
        layers.entry(category).or_insert_with(Vec::new).push((id, name));
    }
    
    // Auto-layout: calculate positions
    let mut y_pos = 100;
    let layer_height = 300;
    let comp_width = 400;
    let comp_spacing = 50;
    
    for (layer_name, comps) in layers.iter() {
        let comps_per_row = (comps.len() as f32).sqrt().ceil() as usize;
        let mut x_pos = 100;
        let mut row_idx = 0;
        
        for (idx, (id, name)) in comps.iter().enumerate() {
            if idx > 0 && idx % comps_per_row == 0 {
                row_idx += 1;
                x_pos = 100;
            }
            
            components.push(ComponentPosition {
                id: id.clone(),
                name: name.clone(),
                x: x_pos,
                y: y_pos + (row_idx * layer_height),
                layer: layer_name.clone(),
            });
            
            x_pos += comp_width + comp_spacing;
        }
        
        y_pos += ((comps.len() - 1) / comps_per_row + 1) as u32 * layer_height + 100;
    }
    
    // Calculate canvas size
    let max_x = components.iter().map(|c| c.x).max().unwrap_or(1400) + comp_width + 100;
    let max_y = components.iter().map(|c| c.y).max().unwrap_or(800) + 300;
    
    LayoutInfo {
        width: max_x.max(1800),
        height: max_y.max(1000),
        components,
    }
}

pub struct ArcVizGenerator {
    width: u32,
    height: u32,
    theme: String,
}

impl ArcVizGenerator {
    pub fn new() -> Self {
        Self {
            width: 1400,
            height: 1000,
            theme: "professional".to_string(),
        }
    }
    
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    
    pub fn with_theme(mut self, theme: String) -> Self {
        self.theme = theme;
        self
    }
    
    pub fn generate(&self, model: &SemanticModel, title: &str) -> Result<String, CompilerError> {
        let mut svg = String::new();
        
        // SVG Header
        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" 
     xmlns:xlink="http://www.w3.org/1999/xlink"
     width="{}" height="{}" viewBox="0 0 {} {}">
"#,
            self.width, self.height, self.width, self.height
        ));
        
        // Styles
        svg.push_str(&self.generate_styles());
        
        // Title
        svg.push_str(&format!(
            r#"  <text x="{}" y="40" class="title">{}</text>
"#,
            self.width / 2,
            title
        ));
        
        // Generate requirement boxes
        let categories = self.group_by_category(model);
        let mut y_offset = 80;
        
        for (category, requirements) in &categories {
            svg.push_str(&self.generate_category_group(category, requirements, 50, y_offset)?);
            y_offset += 150 + (requirements.len() as u32 * 60);
        }
        
        // Generate relationships
        svg.push_str(&self.generate_relationships(model, &categories)?);
        
        svg.push_str("</svg>\n");
        
        Ok(svg)
    }
    
    fn generate_styles(&self) -> String {
        String::from(
r###"  <defs>
    <style>
      .title {
        font-family: 'Segoe UI', Arial, sans-serif;
        font-size: 28px;
        font-weight: bold;
        fill: #2c3e50;
        text-anchor: middle;
      }
      .category-title {
        font-family: 'Segoe UI', Arial, sans-serif;
        font-size: 18px;
        font-weight: 600;
        fill: #34495e;
      }
      .req-box {
        fill: #e3f2fd;
        stroke: #1976d2;
        stroke-width: 2.5;
        rx: 6;
        filter: drop-shadow(3px 3px 4px rgba(0,0,0,0.15));
      }
      .req-box-critical {
        fill: #ffebee;
        stroke: #d32f2f;
        stroke-width: 3.5;
        filter: drop-shadow(3px 3px 5px rgba(211,47,47,0.25));
      }
      .req-box-high {
        fill: #fff3e0;
        stroke: #f57c00;
        stroke-width: 3;
        filter: drop-shadow(3px 3px 4px rgba(245,124,0,0.2));
      }
      .req-id {
        font-family: 'Consolas', 'Monaco', monospace;
        font-size: 14px;
        font-weight: bold;
        fill: #2c3e50;
      }
      .req-desc {
        font-family: 'Segoe UI', Arial, sans-serif;
        font-size: 12px;
        fill: #555;
      }
      .req-badge {
        font-family: 'Segoe UI', Arial, sans-serif;
        font-size: 10px;
        fill: white;
        font-weight: 600;
      }
      .trace-line {
        stroke: #95a5a6;
        stroke-width: 2;
        fill: none;
        marker-end: url(#arrowhead);
      }
      .category-box {
        fill: #f8f9fa;
        stroke: #bdc3c7;
        stroke-width: 1;
        rx: 8;
      }
    </style>
    <marker id="arrowhead" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto">
      <polygon points="0 0, 10 3, 0 6" fill="#95a5a6" />
    </marker>
  </defs>
"###)
    }
    
    fn group_by_category<'a>(&self, model: &'a SemanticModel) -> Vec<(String, Vec<&'a super::semantic::RequirementInfo>)> {
        let mut categories: HashMap<String, Vec<&super::semantic::RequirementInfo>> = HashMap::new();
        
        for req in &model.requirements {
            let category = req.category.clone().unwrap_or_else(|| "General".to_string());
            categories.entry(category).or_insert_with(Vec::new).push(req);
        }
        
        let mut sorted_categories: Vec<_> = categories.into_iter().collect();
        sorted_categories.sort_by(|a, b| a.0.cmp(&b.0));
        
        sorted_categories
    }
    
    fn generate_category_group(
        &self,
        category: &str,
        requirements: &[&super::semantic::RequirementInfo],
        x: u32,
        y: u32,
    ) -> Result<String, CompilerError> {
        let mut svg = String::new();
        
        let box_width = self.width - 100;
        let box_height = 80 + (requirements.len() as u32 * 60);
        
        // Category container
        svg.push_str(&format!(
            r#"  <rect x="{}" y="{}" width="{}" height="{}" class="category-box"/>
"#,
            x, y, box_width, box_height
        ));
        
        // Category title
        svg.push_str(&format!(
            r#"  <text x="{}" y="{}" class="category-title">{}</text>
"#,
            x + 20,
            y + 30,
            category
        ));
        
        // Requirements
        let mut req_y = y + 60;
        for req in requirements {
            svg.push_str(&self.generate_requirement_box(req, x + 30, req_y)?);
            req_y += 60;
        }
        
        Ok(svg)
    }
    
    fn generate_requirement_box(
        &self,
        req: &super::semantic::RequirementInfo,
        x: u32,
        y: u32,
    ) -> Result<String, CompilerError> {
        let mut svg = String::new();
        
        let box_class = match req.priority.as_str() {
            "Critical" => "req-box-critical",
            "High" => "req-box-high",
            _ => "req-box",
        };
        
        // Main box
        svg.push_str(&format!(
            r#"  <rect x="{}" y="{}" width="1200" height="50" class="{}"/>
"#,
            x, y, box_class
        ));
        
        // ID
        svg.push_str(&format!(
            r#"  <text x="{}" y="{}" class="req-id">{}</text>
"#,
            x + 15,
            y + 25,
            req.id
        ));
        
        // Description
        let desc = if req.description.len() > 80 {
            format!("{}...", &req.description[..77])
        } else {
            req.description.clone()
        };
        
        svg.push_str(&format!(
            r#"  <text x="{}" y="{}" class="req-desc">{}</text>
"#,
            x + 200,
            y + 25,
            Self::escape_xml(&desc)
        ));
        
        // Priority badge
        let badge_x = x + 1050;
        let badge_color = match req.priority.as_str() {
            "Critical" => "#e74c3c",
            "High" => "#f39c12",
            "Medium" => "#3498db",
            _ => "#95a5a6",
        };
        
        svg.push_str(&format!(
            r#"  <rect x="{}" y="{}" width="80" height="20" rx="10" fill="{}"/>
"#,
            badge_x, y + 15, badge_color
        ));
        
        svg.push_str(&format!(
            r#"  <text x="{}" y="{}" class="req-badge">{}</text>
"#,
            badge_x + 40,
            y + 29,
            req.priority
        ));
        
        Ok(svg)
    }
    
    fn generate_relationships(
        &self,
        model: &SemanticModel,
        categories: &[(String, Vec<&super::semantic::RequirementInfo>)],
    ) -> Result<String, CompilerError> {
        let mut svg = String::new();
        
        // Build position map
        let mut positions: HashMap<String, (u32, u32)> = HashMap::new();
        let mut y_offset = 80;
        
        for (_, requirements) in categories {
            let mut req_y = y_offset + 60;
            for req in requirements {
                positions.insert(req.id.clone(), (50, req_y + 25));
                req_y += 60;
            }
            y_offset += 150 + (requirements.len() as u32 * 60);
        }
        
        // Draw trace lines
        for trace in &model.traces {
            if trace.trace_type == "implements" {
                if let (Some(&(x1, y1)), Some(&(x2, y2))) = 
                    (positions.get(&trace.from), positions.get(&trace.to)) {
                    svg.push_str(&format!(
                        r#"  <path d="M {} {} L {} {}" class="trace-line"/>
"#,
                        x1 + 1230, y1, x2 + 1230, y2
                    ));
                }
            }
        }
        
        Ok(svg)
    }
    
    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

pub fn generate_arcviz(
    model: &SemanticModel,
    title: &str,
) -> Result<String, CompilerError> {
    ArcVizGenerator::new().generate(model, title)
}

pub fn generate_arcviz_architecture(
    model: &SemanticModel,
    title: &str,
) -> Result<String, CompilerError> {
    // Auto-layout components
    let layout = compute_layout(model);
    let (width, height) = (layout.width, layout.height);
    
    let mut svg = String::new();
    
    // SVG Header with dynamic size
    svg.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" 
     xmlns:xlink="http://www.w3.org/1999/xlink"
     width="{}" height="{}" viewBox="0 0 {} {}">
"#,
        width, height, width, height
    ));
    
    // Component architecture styles
    svg.push_str(r###"  <defs>
    <style>
      .title {
        font-family: 'Segoe UI', Arial, sans-serif;
        font-size: 28px;
        font-weight: bold;
        fill: #2c3e50;
        text-anchor: middle;
      }
      .component-box {
        fill: #e3f2fd;
        stroke: #1976d2;
        stroke-width: 3;
        rx: 8;
        filter: drop-shadow(4px 4px 6px rgba(0,0,0,0.2));
      }
      .component-name {
        font-family: 'Segoe UI', Arial, sans-serif;
        font-size: 16px;
        font-weight: 600;
        fill: #1976d2;
        text-anchor: middle;
      }
      .component-id {
        font-family: 'Consolas', monospace;
        font-size: 11px;
        fill: #666;
        text-anchor: middle;
      }
      .function-box {
        fill: #fff3e0;
        stroke: #ff9800;
        stroke-width: 1.5;
        rx: 4;
      }
      .function-text {
        font-family: 'Segoe UI', Arial, sans-serif;
        font-size: 11px;
        fill: #333;
      }
      .port-text {
        font-family: 'Consolas', monospace;
        font-size: 9px;
        fill: #555;
      }
      .connector {
        stroke: #1976d2;
        stroke-width: 2;
        fill: none;
        marker-end: url(#arrow);
      }
      .connector-label {
        font-family: 'Segoe UI', Arial, sans-serif;
        font-size: 10px;
        fill: #666;
      }
      .layer-label {
        font-family: 'Segoe UI', Arial, sans-serif;
        font-size: 18px;
        font-weight: 600;
        fill: #34495e;
      }
    </style>
    <marker id="arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto">
      <polygon points="0 0, 10 3, 0 6" fill="#1976d2" />
    </marker>
  </defs>
"###);
    
    // Title
    svg.push_str(&format!(r#"  <text x="{}" y="40" class="title">{}</text>
"#, width / 2, title));
    
    // Draw components using auto-layout
    let mut current_layer = String::new();
    for comp in &layout.components {
        // Draw layer label if new layer
        if comp.layer != current_layer {
            svg.push_str(&format!(r#"  <text x="50" y="{}" class="layer-label">{}</text>
"#, comp.y - 20, comp.layer));
            current_layer = comp.layer.clone();
        }
        
        // Draw component
        svg.push_str(&draw_component(&comp.id, &comp.name, comp.x, comp.y));
    }
    
    // Draw connectors from traces
    svg.push_str("  <!-- Component Connectors -->\n");
    
    // Build position map
    let mut pos_map: HashMap<String, (u32, u32)> = HashMap::new();
    for comp in &layout.components {
        pos_map.insert(comp.id.clone(), (comp.x + 200, comp.y + 110));
    }
    
    // Draw trace connections if available
    let mut has_traces = false;
    for trace in &model.traces {
        if trace.trace_type == "implements" {
            // Find component positions
            let from_comp = layout.components.iter().find(|c| c.id == trace.from);
            let to_comp = layout.components.iter().find(|c| c.id == trace.to);
            
            if let (Some(from), Some(to)) = (from_comp, to_comp) {
                has_traces = true;
                
                // Start from output port (bottom center of from component)
                let x1 = from.x + 200;  // Center X
                let y1 = from.y + 220;  // Bottom of component (OUT port)
                
                // End at input port (top center of to component)
                let x2 = to.x + 200;    // Center X
                let y2 = to.y;          // Top of component (IN port)
                
                // Detect if components are on same horizontal row (same Y position)
                let same_row = from.y == to.y;
                
                if same_row {
                    // Horizontal routing - use side ports with clearance
                    let x1_right = from.x + 400;     // Right edge OUT
                    let y_mid = from.y + 110;         // Middle height
                    let x2_left = to.x;               // Left edge IN
                    
                    // Add intermediate point if gap is small to avoid overlap
                    let gap = x2_left as i32 - x1_right as i32;
                    if gap < 30 {
                        // Route around if too close
                        let below_y = from.y + 240;
                        let mid_x = (x1_right + x2_left) / 2;
                        svg.push_str(&format!(
                            r#"  <path d="M {} {} L {} {} L {} {} L {} {} L {} {}" class="connector"/>
"#,
                            x1_right, y_mid, x1_right + 20, y_mid, x1_right + 20, below_y, 
                            x2_left - 20, below_y, x2_left, y_mid
                        ));
                    } else {
                        // Direct horizontal line
                        svg.push_str(&format!(
                            r#"  <path d="M {} {} L {} {}" class="connector"/>
"#,
                            x1_right, y_mid, x2_left, y_mid
                        ));
                    }
                } else if y2 > y1 {
                    // Vertical routing downward (component below)
                    let mid_y = (y1 + y2) / 2;
                    svg.push_str(&format!(
                        r#"  <path d="M {} {} L {} {} L {} {}" class="connector"/>
"#,
                        x1, y1, x1, mid_y, x2, y2
                    ));
                } else {
                    // Going upward - route around with minimal clearance
                    // Use bottom edge + small offset instead of going too far down
                    let clear_y = from.y + 240;  // Just below component (220 + 20px)
                    
                    if x1 == x2 {
                        // Same column - simple U-turn with tight clearance
                        svg.push_str(&format!(
                            r#"  <path d="M {} {} L {} {} L {} {}" class="connector"/>
"#,
                            x1, y1, x1, clear_y, x2, y2
                        ));
                    } else {
                        // Different columns - route with smooth corners
                        let mid_x = (x1 + x2) / 2;
                        let approach_y = y2 - 30;  // Approach from above with clearance
                        
                        svg.push_str(&format!(
                            r#"  <path d="M {} {} L {} {} L {} {} L {} {} L {} {}" class="connector"/>
"#,
                            x1, y1, x1, clear_y, mid_x, clear_y, mid_x, approach_y, x2, y2
                        ));
                    }
                }
            }
        }
    }
    
    // If no traces, draw simple sequential connections
    if !has_traces && layout.components.len() > 1 {
        svg.push_str("  <!-- Auto-generated connections (no explicit traces found) -->\n");
        
        for i in 0..layout.components.len() - 1 {
            let comp1 = &layout.components[i];
            let comp2 = &layout.components[i + 1];
            
            // Start from OUT port (bottom center)
            let x1 = comp1.x + 200;
            let y1 = comp1.y + 220;
            
            // End at IN port (top center)  
            let x2 = comp2.x + 200;
            let y2 = comp2.y;
            
            // Different layers (vertical connection)
            if comp1.layer != comp2.layer && y2 > y1 {
                let mid_y = (y1 + y2) / 2;
                svg.push_str(&format!(
                    r#"  <path d="M {} {} L {} {} L {} {}" class="connector" opacity="0.5"/>
"#,
                    x1, y1, x1, mid_y, x2, y2
                ));
            }
            // Same layer (horizontal connection from port to port)
            else if comp1.layer == comp2.layer && comp2.x > comp1.x {
                // Use right side port for OUT, left side port for IN
                let x1_port = comp1.x + 400;  // Right edge
                let y1_port = comp1.y + 110;  // Middle height
                let x2_port = comp2.x;        // Left edge
                let y2_port = comp2.y + 110;  // Middle height
                
                svg.push_str(&format!(
                    r#"  <path d="M {} {} L {} {}" class="connector" opacity="0.5"/>
"#,
                    x1_port, y1_port, x2_port, y2_port
                ));
            }
        }
    }
    
    svg.push_str("</svg>\n");
    Ok(svg)
}

fn draw_component(id: &str, name: &str, x: u32, y: u32) -> String {
    let center_x = x + 200;
    let mut svg = String::new();
    
    svg.push_str(&format!("  <!-- Component: {} -->\n", id));
    svg.push_str(&format!("  <rect x=\"{}\" y=\"{}\" width=\"400\" height=\"220\" class=\"component-box\"/>\n", x, y));
    svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" class=\"component-name\">{}</text>\n", center_x, y + 30, name));
    svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" class=\"component-id\">{}</text>\n", center_x, y + 50, id));
    
    // Input Port (top)
    svg.push_str(&format!("  <rect x=\"{}\" y=\"{}\" width=\"20\" height=\"8\" fill=\"#4caf50\" stroke=\"#2e7d32\" stroke-width=\"1\"/>\n", center_x - 10, y - 2));
    svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" class=\"port-text\" text-anchor=\"middle\">IN</text>\n", center_x, y + 15));
    
    // Output Port (bottom)
    svg.push_str(&format!("  <rect x=\"{}\" y=\"{}\" width=\"20\" height=\"8\" fill=\"#ff9800\" stroke=\"#f57c00\" stroke-width=\"1\"/>\n", center_x - 10, y + 220 - 6));
    svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" class=\"port-text\" text-anchor=\"middle\">OUT</text>\n", center_x, y + 228));
    
    // Functions inside
    svg.push_str(&format!("  <rect x=\"{}\" y=\"{}\" width=\"360\" height=\"120\" fill=\"#f5f5f5\" stroke=\"#bdbdbd\" stroke-width=\"1\" rx=\"4\"/>\n", x + 20, y + 70));
    svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" class=\"function-text\" fill=\"#666\">⚙ Functions &amp; Processing</text>\n", x + 30, y + 90));
    svg.push_str(&format!("  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#e0e0e0\" stroke-width=\"1\"/>\n", x + 30, y + 105, x + 370, y + 105));
    svg.push_str(&format!("  <text x=\"{}\" y=\"{}\" style=\"font-size:10px; fill:#999;\">Data processing logic</text>\n", x + 30, y + 125));
    
    svg
}

pub fn generate_arcviz_html(
    model: &SemanticModel,
    title: &str,
) -> Result<String, CompilerError> {
    // Check if model has components - if so, generate architecture view
    if !model.components.is_empty() {
        let svg = generate_arcviz_architecture(model, title)?;
        return Ok(wrap_in_html(title, &svg));
    }
    
    // Otherwise generate requirements view
    let svg = generate_arcviz(model, title)?;
    Ok(wrap_in_html(title, &svg))
}

fn wrap_in_html(title: &str, svg: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{0} - ArcViz</title>
    <style>
        body {{
            margin: 0;
            padding: 0;
            font-family: 'Segoe UI', Arial, sans-serif;
            background: #f5f6fa;
            overflow: hidden;
        }}
        #container {{
            width: 100vw;
            height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
            overflow: auto;
        }}
        svg {{
            max-width: 95%;
            height: auto;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
            background: white;
            border-radius: 8px;
        }}
        .controls {{
            position: fixed;
            top: 20px;
            right: 20px;
            background: white;
            padding: 15px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        .controls button {{
            margin: 5px;
            padding: 8px 16px;
            border: none;
            border-radius: 4px;
            background: #3498db;
            color: white;
            cursor: pointer;
            font-size: 14px;
        }}
        .controls button:hover {{
            background: #2980b9;
        }}
        .info {{
            position: fixed;
            bottom: 20px;
            left: 20px;
            background: white;
            padding: 15px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
            font-size: 12px;
            color: #7f8c8d;
        }}
    </style>
</head>
<body>
    <div class="controls">
        <button onclick="zoomIn()">🔍 Zoom In</button>
        <button onclick="zoomOut()">🔍 Zoom Out</button>
        <button onclick="resetZoom()">↻ Reset</button>
        <button onclick="exportSVG()">💾 Export SVG</button>
    </div>
    <div id="container">
        {1}
    </div>
    <div class="info">
        🎨 <strong>ArcViz</strong> - ArcLang Native Visualization | Use mouse wheel to zoom | Drag to pan
    </div>
    <script>
        let scale = 1;
        const svg = document.querySelector('svg');
        const container = document.getElementById('container');
        
        function zoomIn() {{
            scale *= 1.2;
            svg.style.transform = `scale(${{scale}})`;
        }}
        
        function zoomOut() {{
            scale /= 1.2;
            svg.style.transform = `scale(${{scale}})`;
        }}
        
        function resetZoom() {{
            scale = 1;
            svg.style.transform = 'scale(1)';
        }}
        
        function exportSVG() {{
            const svgData = svg.outerHTML;
            const blob = new Blob([svgData], {{ type: 'image/svg+xml' }});
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'arcviz-diagram.svg';
            a.click();
        }}
        
        // Mouse wheel zoom
        container.addEventListener('wheel', (e) => {{
            e.preventDefault();
            if (e.deltaY < 0) {{
                zoomIn();
            }} else {{
                zoomOut();
            }}
        }});
        
        // Drag to pan
        let isDragging = false;
        let startX, startY, scrollLeft, scrollTop;
        
        container.addEventListener('mousedown', (e) => {{
            isDragging = true;
            startX = e.pageX - container.offsetLeft;
            startY = e.pageY - container.offsetTop;
            scrollLeft = container.scrollLeft;
            scrollTop = container.scrollTop;
        }});
        
        container.addEventListener('mouseleave', () => {{
            isDragging = false;
        }});
        
        container.addEventListener('mouseup', () => {{
            isDragging = false;
        }});
        
        container.addEventListener('mousemove', (e) => {{
            if (!isDragging) return;
            e.preventDefault();
            const x = e.pageX - container.offsetLeft;
            const y = e.pageY - container.offsetTop;
            const walkX = (x - startX) * 1;
            const walkY = (y - startY) * 1;
            container.scrollLeft = scrollLeft - walkX;
            container.scrollTop = scrollTop - walkY;
        }});
    </script>
</body>
</html>
"#,
        title, svg
    )
}
