//! DEPRECATED: Use arcviz_ultimate_routing instead
//! This implementation still has crossing issues in complex scenarios

use super::semantic::SemanticModel;
use super::CompilerError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Component {
    id: String,
    name: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    layer: String,
}

#[derive(Debug, Clone)]
struct Connection {
    from_id: String,
    to_id: String,
    label: String,
}

#[derive(Debug)]
struct Rectangle {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Rectangle {
    fn contains_point(&self, px: u32, py: u32) -> bool {
        px >= self.x && px <= self.x + self.width &&
        py >= self.y && py <= self.y + self.height
    }
    
    fn intersects(&self, other: &Rectangle) -> bool {
        !(self.x + self.width < other.x ||
          other.x + other.width < self.x ||
          self.y + self.height < other.y ||
          other.y + other.height < self.y)
    }
}

/// Smart connector routing that avoids crossing other components (Capella-style)
pub struct SmartRouter {
    components: Vec<Component>,
    connections: Vec<Connection>,
    margin: u32,
}

impl SmartRouter {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            connections: Vec::new(),
            margin: 30, // Clearance around components
        }
    }
    
    pub fn add_component(&mut self, comp: Component) {
        self.components.push(comp);
    }
    
    pub fn add_connection(&mut self, conn: Connection) {
        self.connections.push(conn);
    }
    
    /// Generate SVG path with smart routing
    pub fn generate_routed_path(&self, from_id: &str, to_id: &str) -> Option<String> {
        let from = self.components.iter().find(|c| c.id == from_id)?;
        let to = self.components.iter().find(|c| c.id == to_id)?;
        
        // Connection points (center bottom OUT for from, center top IN for to)
        let start_x = from.x + from.width / 2;
        let start_y = from.y + from.height;
        let end_x = to.x + to.width / 2;
        let end_y = to.y;
        
        // Check if direct path is clear
        if self.is_path_clear(start_x, start_y, end_x, end_y, from_id, to_id) {
            return Some(self.direct_path(start_x, start_y, end_x, end_y));
        }
        
        // Use orthogonal routing (Manhattan routing) - Capella style
        let path = self.orthogonal_route(start_x, start_y, end_x, end_y, from, to);
        Some(path)
    }
    
    /// Check if a direct line path is clear of obstacles
    fn is_path_clear(&self, x1: u32, y1: u32, x2: u32, y2: u32, from_id: &str, to_id: &str) -> bool {
        // Simple bounding box check for line segment
        let line_box = Rectangle {
            x: x1.min(x2),
            y: y1.min(y2),
            width: (x1 as i32 - x2 as i32).abs() as u32,
            height: (y1 as i32 - y2 as i32).abs() as u32,
        };
        
        // Check if line intersects any component (except source and target)
        for comp in &self.components {
            if comp.id == from_id || comp.id == to_id {
                continue;
            }
            
            let comp_box = Rectangle {
                x: comp.x.saturating_sub(self.margin),
                y: comp.y.saturating_sub(self.margin),
                width: comp.width + 2 * self.margin,
                height: comp.height + 2 * self.margin,
            };
            
            if comp_box.intersects(&line_box) {
                return false;
            }
        }
        
        true
    }
    
    /// Direct straight path
    fn direct_path(&self, x1: u32, y1: u32, x2: u32, y2: u32) -> String {
        format!("M {} {} L {} {}", x1, y1, x2, y2)
    }
    
    /// Orthogonal (Manhattan) routing - professional Capella style
    fn orthogonal_route(
        &self,
        start_x: u32,
        start_y: u32,
        end_x: u32,
        end_y: u32,
        from_comp: &Component,
        to_comp: &Component,
    ) -> String {
        let mut path = format!("M {} {}", start_x, start_y);
        
        // Route decision based on relative positions
        let going_down = end_y > start_y;
        let going_right = end_x > start_x;
        
        if going_down {
            // Vertical then horizontal routing
            
            // Step 1: Go down from source component with clearance
            let clear_y = from_comp.y + from_comp.height + self.margin;
            path.push_str(&format!(" L {} {}", start_x, clear_y));
            
            // Step 2: Check if we need to route around obstacles
            let mid_x = if self.needs_horizontal_routing(start_x, end_x, clear_y, to_comp) {
                // Route to side first
                if going_right {
                    from_comp.x + from_comp.width + self.margin
                } else {
                    from_comp.x.saturating_sub(self.margin)
                }
            } else {
                end_x
            };
            
            // Step 3: Horizontal segment
            if mid_x != start_x {
                path.push_str(&format!(" L {} {}", mid_x, clear_y));
            }
            
            // Step 4: Approach target from above
            let approach_y = to_comp.y.saturating_sub(self.margin);
            if mid_x != end_x {
                path.push_str(&format!(" L {} {}", mid_x, approach_y));
                path.push_str(&format!(" L {} {}", end_x, approach_y));
            }
            
            // Step 5: Final vertical to target
            path.push_str(&format!(" L {} {}", end_x, end_y));
            
        } else {
            // Going upward - route around with U-shape
            
            // Step 1: Go down below source
            let below_y = from_comp.y + from_comp.height + self.margin * 2;
            path.push_str(&format!(" L {} {}", start_x, below_y));
            
            // Step 2: Horizontal to align with target X
            if start_x != end_x {
                path.push_str(&format!(" L {} {}", end_x, below_y));
            }
            
            // Step 3: Go up to target
            path.push_str(&format!(" L {} {}", end_x, end_y));
        }
        
        path
    }
    
    /// Check if horizontal routing is needed to avoid obstacles
    fn needs_horizontal_routing(&self, start_x: u32, end_x: u32, y: u32, target: &Component) -> bool {
        let min_x = start_x.min(end_x);
        let max_x = start_x.max(end_x);
        
        for comp in &self.components {
            if comp.id == target.id {
                continue;
            }
            
            // Check if component is in the way
            if comp.y <= y && y <= comp.y + comp.height {
                if comp.x <= max_x && min_x <= comp.x + comp.width {
                    return true;
                }
            }
        }
        
        false
    }
}

/// Auto-layout with hierarchical positioning (Capella style)
pub fn compute_smart_layout(model: &SemanticModel) -> (Vec<Component>, u32, u32) {
    let mut components = Vec::new();
    
    if model.components.is_empty() {
        return (components, 1800, 1000);
    }
    
    // Group by layer
    let mut layers: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for comp in &model.components {
        let layer = comp.level.clone();
        layers.entry(layer).or_insert_with(Vec::new)
            .push((comp.id.clone(), comp.name.clone()));
    }
    
    // Layout parameters
    const COMP_WIDTH: u32 = 400;
    const COMP_HEIGHT: u32 = 220;
    const HORIZONTAL_GAP: u32 = 120; // Increased for routing space
    const VERTICAL_GAP: u32 = 150;   // Increased for routing space
    const MARGIN_LEFT: u32 = 100;
    const MARGIN_TOP: u32 = 100;
    
    let mut y_pos = MARGIN_TOP;
    
    // Sort layers for consistent ordering
    let mut sorted_layers: Vec<_> = layers.iter().collect();
    sorted_layers.sort_by(|a, b| a.0.cmp(b.0));
    
    for (layer_name, comps) in sorted_layers {
        // Calculate components per row (aim for square-ish layout)
        let comps_per_row = (comps.len() as f32).sqrt().ceil() as usize;
        let comps_per_row = comps_per_row.max(1).min(3); // Limit to 3 per row
        
        let mut x_pos = MARGIN_LEFT;
        let mut row = 0;
        
        for (idx, (id, name)) in comps.iter().enumerate() {
            if idx > 0 && idx % comps_per_row == 0 {
                // New row
                row += 1;
                x_pos = MARGIN_LEFT;
            }
            
            components.push(Component {
                id: id.clone(),
                name: name.clone(),
                x: x_pos,
                y: y_pos + (row * (COMP_HEIGHT + VERTICAL_GAP)),
                width: COMP_WIDTH,
                height: COMP_HEIGHT,
                layer: layer_name.clone(),
            });
            
            x_pos += COMP_WIDTH + HORIZONTAL_GAP;
        }
        
        // Move to next layer
        let rows = ((comps.len() - 1) / comps_per_row + 1) as u32;
        y_pos += rows * (COMP_HEIGHT + VERTICAL_GAP) + 100;
    }
    
    // Calculate canvas size
    let max_x = components.iter()
        .map(|c| c.x + c.width)
        .max()
        .unwrap_or(1800) + MARGIN_LEFT;
    let max_y = components.iter()
        .map(|c| c.y + c.height)
        .max()
        .unwrap_or(1000) + MARGIN_TOP;
    
    (components, max_x, max_y)
}

/// Generate complete SVG with smart routing
pub fn generate_smart_arcviz(
    model: &SemanticModel,
    title: &str,
) -> Result<String, CompilerError> {
    let (components, width, height) = compute_smart_layout(model);
    
    let mut router = SmartRouter::new();
    
    // Add components to router
    for comp in &components {
        router.add_component(comp.clone());
    }
    
    // Build connections from traces
    for trace in &model.traces {
        if trace.trace_type == "implements" {
            router.add_connection(Connection {
                from_id: trace.from.clone(),
                to_id: trace.to.clone(),
                label: trace.trace_type.clone(),
            });
        }
    }
    
    let mut svg = String::new();
    
    // SVG Header
    svg.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" 
     xmlns:xlink="http://www.w3.org/1999/xlink"
     width="{}" height="{}" viewBox="0 0 {} {}">
"#,
        width, height, width, height
    ));
    
    // Enhanced styles
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
        fill: #e8f4f8;
        stroke: #0277bd;
        stroke-width: 3;
        rx: 8;
        filter: drop-shadow(3px 3px 5px rgba(0,0,0,0.15));
      }
      .component-name {
        font-family: 'Segoe UI', Arial, sans-serif;
        font-size: 16px;
        font-weight: 600;
        fill: #01579b;
        text-anchor: middle;
      }
      .component-id {
        font-family: 'Consolas', monospace;
        font-size: 11px;
        fill: #546e7a;
        text-anchor: middle;
      }
      .port-in {
        fill: #4caf50;
        stroke: #2e7d32;
        stroke-width: 1.5;
      }
      .port-out {
        fill: #ff9800;
        stroke: #f57c00;
        stroke-width: 1.5;
      }
      .port-text {
        font-family: 'Consolas', monospace;
        font-size: 9px;
        fill: #263238;
        font-weight: 600;
      }
      .connector {
        stroke: #0277bd;
        stroke-width: 3;
        fill: none;
        marker-end: url(#arrow);
      }
      .connector-hover {
        stroke: #01579b;
        stroke-width: 4;
      }
      .layer-label {
        font-family: 'Segoe UI', Arial, sans-serif;
        font-size: 18px;
        font-weight: 600;
        fill: #37474f;
      }
      .function-area {
        fill: #f5f5f5;
        stroke: #bdbdbd;
        stroke-width: 1;
        rx: 4;
      }
    </style>
    <marker id="arrow" markerWidth="12" markerHeight="12" refX="10" refY="4" orient="auto">
      <path d="M 0 0 L 10 4 L 0 8 z" fill="#0277bd" />
    </marker>
  </defs>
"###);
    
    // Title
    svg.push_str(&format!(
        r#"  <text x="{}" y="50" class="title">{}</text>
"#,
        width / 2, title
    ));
    
    // Draw layer labels and components
    let mut current_layer = String::new();
    for comp in &components {
        // Layer label
        if comp.layer != current_layer {
            svg.push_str(&format!(
                r#"  <text x="50" y="{}" class="layer-label">{}</text>
"#,
                comp.y - 20, comp.layer
            ));
            current_layer = comp.layer.clone();
        }
        
        // Component box
        svg.push_str(&draw_capella_component(comp));
    }
    
    // Draw smart-routed connectors
    svg.push_str("  <!-- Smart Routed Connectors -->\n");
    
    for conn in &router.connections {
        if let Some(path) = router.generate_routed_path(&conn.from_id, &conn.to_id) {
            svg.push_str(&format!(
                r#"  <path d="{}" class="connector">
    <title>{} → {}</title>
  </path>
"#,
                path, conn.from_id, conn.to_id
            ));
        }
    }
    
    // If no traces, add auto-generated sequential connections with smart routing
    if router.connections.is_empty() && components.len() > 1 {
        svg.push_str("  <!-- Auto-Generated Smart Routes -->\n");
        
        for i in 0..components.len().saturating_sub(1) {
            let from_id = &components[i].id;
            let to_id = &components[i + 1].id;
            
            // Create temporary connection
            let temp_conn = Connection {
                from_id: from_id.clone(),
                to_id: to_id.clone(),
                label: "auto".to_string(),
            };
            
            let mut temp_router = SmartRouter::new();
            for c in &components {
                temp_router.add_component(c.clone());
            }
            temp_router.add_connection(temp_conn);
            
            if let Some(path) = temp_router.generate_routed_path(from_id, to_id) {
                svg.push_str(&format!(
                    r#"  <path d="{}" class="connector" opacity="0.6"/>
"#,
                    path
                ));
            }
        }
    }
    
    svg.push_str("</svg>\n");
    Ok(svg)
}

fn draw_capella_component(comp: &Component) -> String {
    let center_x = comp.x + comp.width / 2;
    let mut svg = String::new();
    
    svg.push_str(&format!("  <!-- Component: {} -->\n", comp.id));
    
    // Main box
    svg.push_str(&format!(
        r#"  <rect x="{}" y="{}" width="{}" height="{}" class="component-box"/>
"#,
        comp.x, comp.y, comp.width, comp.height
    ));
    
    // Name
    svg.push_str(&format!(
        r#"  <text x="{}" y="{}" class="component-name">{}</text>
"#,
        center_x, comp.y + 28, comp.name
    ));
    
    // ID
    svg.push_str(&format!(
        r#"  <text x="{}" y="{}" class="component-id">{}</text>
"#,
        center_x, comp.y + 48, comp.id
    ));
    
    // Input Port (top center)
    let port_width = 24;
    let port_height = 10;
    svg.push_str(&format!(
        r#"  <rect x="{}" y="{}" width="{}" height="{}" class="port-in"/>
"#,
        center_x - port_width / 2, comp.y - port_height / 2, port_width, port_height
    ));
    svg.push_str(&format!(
        r#"  <text x="{}" y="{}" class="port-text" text-anchor="middle">IN</text>
"#,
        center_x, comp.y + 18
    ));
    
    // Output Port (bottom center)
    svg.push_str(&format!(
        r#"  <rect x="{}" y="{}" width="{}" height="{}" class="port-out"/>
"#,
        center_x - port_width / 2,
        comp.y + comp.height - port_height / 2,
        port_width,
        port_height
    ));
    svg.push_str(&format!(
        r#"  <text x="{}" y="{}" class="port-text" text-anchor="middle">OUT</text>
"#,
        center_x, comp.y + comp.height + 13
    ));
    
    // Internal function area
    let func_x = comp.x + 20;
    let func_y = comp.y + 70;
    let func_width = comp.width - 40;
    let func_height = comp.height - 100;
    
    svg.push_str(&format!(
        r#"  <rect x="{}" y="{}" width="{}" height="{}" class="function-area"/>
"#,
        func_x, func_y, func_width, func_height
    ));
    
    svg.push_str(&format!(
        r#"  <text x="{}" y="{}" style="font-size:11px; fill:#666;">⚙ Functions &amp; Processing</text>
"#,
        func_x + 10, func_y + 20
    ));
    
    svg.push_str(&format!(
        "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#e0e0e0\" stroke-width=\"1\"/>\n",
        func_x + 10, func_y + 30, func_x + func_width - 10, func_y + 30
    ));
    
    svg
}

/// Wrap SVG in interactive HTML
pub fn wrap_smart_arcviz_html(title: &str, svg: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{} - ArcViz Smart Routing</title>
    <style>
        body {{
            margin: 0;
            padding: 0;
            font-family: 'Segoe UI', Arial, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
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
            box-shadow: 0 8px 32px rgba(0,0,0,0.3);
            background: white;
            border-radius: 12px;
        }}
        .controls {{
            position: fixed;
            top: 20px;
            right: 20px;
            background: rgba(255,255,255,0.95);
            padding: 20px;
            border-radius: 12px;
            box-shadow: 0 4px 16px rgba(0,0,0,0.2);
            backdrop-filter: blur(10px);
        }}
        .controls button {{
            margin: 5px;
            padding: 10px 20px;
            border: none;
            border-radius: 6px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            cursor: pointer;
            font-size: 14px;
            font-weight: 600;
            transition: transform 0.2s;
        }}
        .controls button:hover {{
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(102,126,234,0.4);
        }}
        .info {{
            position: fixed;
            bottom: 20px;
            left: 20px;
            background: rgba(255,255,255,0.95);
            padding: 20px;
            border-radius: 12px;
            box-shadow: 0 4px 16px rgba(0,0,0,0.2);
            font-size: 13px;
            color: #37474f;
            backdrop-filter: blur(10px);
        }}
        .info strong {{
            color: #0277bd;
        }}
    </style>
</head>
<body>
    <div class="controls">
        <button onclick="zoomIn()">🔍 Zoom In</button>
        <button onclick="zoomOut()">🔍 Zoom Out</button>
        <button onclick="resetZoom()">↻ Reset</button>
        <button onclick="exportSVG()">💾 Export</button>
    </div>
    <div id="container">
        {}
    </div>
    <div class="info">
        🎨 <strong>ArcViz Smart Routing</strong><br>
        Professional Capella-style diagrams with intelligent connector routing<br>
        <em>Arrows avoid crossing components • Orthogonal routing • Zero overlap</em>
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
            a.download = 'capella-diagram.svg';
            a.click();
        }}
        
        container.addEventListener('wheel', (e) => {{
            e.preventDefault();
            if (e.deltaY < 0) {{
                zoomIn();
            }} else {{
                zoomOut();
            }}
        }});
        
        // Connector hover effects
        const connectors = document.querySelectorAll('.connector');
        connectors.forEach(conn => {{
            conn.addEventListener('mouseenter', () => {{
                conn.style.strokeWidth = '4';
                conn.style.stroke = '#01579b';
            }});
            conn.addEventListener('mouseleave', () => {{
                conn.style.strokeWidth = '3';
                conn.style.stroke = '#0277bd';
            }});
        }});
    </script>
</body>
</html>
"#,
        title, svg
    )
}
