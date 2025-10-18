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
    row: usize,
    col: usize,
}

/// ULTIMATE routing - absolute ZERO crossings
/// Uses ONLY safe vertical spaces and dedicated side channels
/// NO horizontal movement through component space AT ALL
pub struct UltimateRouter {
    components: Vec<Component>,
    comp_width: u32,
    comp_height: u32,
    h_gap: u32,
    v_gap: u32,
    margin_left: u32,
    margin_top: u32,
    side_channel_x: u32,  // Dedicated side channel X coordinate
}

impl UltimateRouter {
    pub fn new() -> Self {
        let comp_width = 380;
        let comp_height = 200;
        let h_gap = 180;
        let margin_left = 100;
        
        // Side channel is FAR to the right, beyond ALL components
        let side_channel_x = margin_left + 3 * (comp_width + h_gap) + 50;
        
        Self {
            components: Vec::new(),
            comp_width,
            comp_height,
            h_gap,
            v_gap: 200,
            margin_left,
            margin_top: 120,
            side_channel_x,
        }
    }
    
    /// Generate path that GUARANTEES zero crossings
    /// Strategy: ALL paths use side channel for ANY horizontal movement
    pub fn generate_ultimate_path(&self, from_id: &str, to_id: &str) -> Option<String> {
        let from = self.components.iter().find(|c| c.id == from_id)?;
        let to = self.components.iter().find(|c| c.id == to_id)?;
        
        let start_x = from.x + from.width / 2;
        let start_y = from.y + from.height;  // OUT port at bottom
        let end_x = to.x + to.width / 2;
        let end_y = to.y;  // IN port at top
        
        let mut path = format!("M {} {}", start_x, start_y);
        
        // RULE: Never move horizontally except in side channel!
        // ALL paths: vertical down -> side channel -> vertical in channel -> horizontal -> vertical to target
        
        if from.row == to.row && from.col == to.col {
            // Same component (shouldn't happen)
            return None;
        }
        
        // Step 1: Go down from source
        let exit_y = start_y + 40;
        path.push_str(&format!(" L {} {}", start_x, exit_y));
        
        // Step 2: Go to side channel (vertical column, then horizontal in safe space)
        path.push_str(&format!(" L {} {}", self.side_channel_x, exit_y));
        
        // Step 3: Move vertically in side channel to target row level
        let approach_y = end_y - 40;
        path.push_str(&format!(" L {} {}", self.side_channel_x, approach_y));
        
        // Step 4: Come back horizontally to target (safe, above target)
        path.push_str(&format!(" L {} {}", end_x, approach_y));
        
        // Step 5: Final vertical approach to target
        path.push_str(&format!(" L {} {}", end_x, end_y));
        
        Some(path)
    }
}

/// Layout with extra space for side routing
pub fn compute_ultimate_layout(model: &SemanticModel) -> (Vec<Component>, u32, u32) {
    let mut components = Vec::new();
    
    if model.components.is_empty() {
        return (components, 1800, 1000);
    }
    
    const COMP_WIDTH: u32 = 380;
    const COMP_HEIGHT: u32 = 200;
    const HORIZONTAL_GAP: u32 = 180;
    const VERTICAL_GAP: u32 = 200;
    const MARGIN_LEFT: u32 = 100;
    const MARGIN_TOP: u32 = 120;
    const PER_ROW: usize = 3;
    
    let mut layers: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for comp in &model.components {
        layers.entry(comp.level.clone())
            .or_insert_with(Vec::new)
            .push((comp.id.clone(), comp.name.clone()));
    }
    
    let mut sorted_layers: Vec<_> = layers.iter().collect();
    sorted_layers.sort_by(|a, b| a.0.cmp(b.0));
    
    let mut global_row = 0;
    
    for (_layer, comps) in sorted_layers {
        let rows = (comps.len() + PER_ROW - 1) / PER_ROW;
        
        for row_in_layer in 0..rows {
            let start = row_in_layer * PER_ROW;
            let end = (start + PER_ROW).min(comps.len());
            
            for (col, idx) in (start..end).enumerate() {
                let (id, name) = &comps[idx];
                
                components.push(Component {
                    id: id.clone(),
                    name: name.clone(),
                    x: MARGIN_LEFT + (col as u32) * (COMP_WIDTH + HORIZONTAL_GAP),
                    y: MARGIN_TOP + (global_row as u32) * (COMP_HEIGHT + VERTICAL_GAP),
                    width: COMP_WIDTH,
                    height: COMP_HEIGHT,
                    row: global_row,
                    col,
                });
            }
            global_row += 1;
        }
    }
    
    // Extra space for side channel
    let max_x = MARGIN_LEFT + 3 * (COMP_WIDTH + HORIZONTAL_GAP) + 200;
    let max_y = components.iter()
        .map(|c| c.y + c.height)
        .max()
        .unwrap_or(1000) + 150;
    
    (components, max_x, max_y)
}

pub fn generate_ultimate_arcviz(model: &SemanticModel, title: &str) -> Result<String, CompilerError> {
    let (components, width, height) = compute_ultimate_layout(model);
    
    let mut router = UltimateRouter::new();
    router.components = components.clone();
    
    let mut svg = String::new();
    
    svg.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
"#,
        width, height, width, height
    ));
    
    svg.push_str(r###"  <defs>
    <style>
      .title {
        font-family: 'Segoe UI', Roboto, sans-serif;
        font-size: 36px;
        font-weight: 800;
        fill: #0d47a1;
        text-anchor: middle;
      }
      .component-box {
        fill: url(#grad1);
        stroke: #01579b;
        stroke-width: 4;
        rx: 12;
        filter: drop-shadow(5px 5px 10px rgba(0,0,0,0.25));
      }
      .component-name {
        font-family: 'Segoe UI', sans-serif;
        font-size: 18px;
        font-weight: 700;
        fill: #01579b;
        text-anchor: middle;
      }
      .component-id {
        font-family: 'Consolas', 'Courier New', monospace;
        font-size: 13px;
        fill: #37474f;
        font-weight: 700;
        text-anchor: middle;
      }
      .port-in {
        fill: #43a047;
        stroke: #1b5e20;
        stroke-width: 2.5;
        rx: 4;
      }
      .port-out {
        fill: #fb8c00;
        stroke: #e65100;
        stroke-width: 2.5;
        rx: 4;
      }
      .port-text {
        font-family: 'Consolas', monospace;
        font-size: 11px;
        fill: white;
        font-weight: 900;
      }
      .connector {
        stroke: #0277bd;
        stroke-width: 1.5;
        fill: none;
        marker-end: url(#arrowhead);
        stroke-linecap: round;
        stroke-linejoin: round;
        opacity: 0.7;
      }
      .connector:hover {
        stroke: #01579b;
        stroke-width: 2.5;
        opacity: 1;
      }
      .func-box {
        fill: #fafafa;
        stroke: #90a4ae;
        stroke-width: 1.5;
        rx: 8;
      }
      .side-channel {
        stroke: #bbdefb;
        stroke-width: 40;
        stroke-dasharray: 8,8;
        opacity: 0.2;
      }
    </style>
    <linearGradient id="grad1" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" style="stop-color:#e1f5fe;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#b3e5fc;stop-opacity:1" />
    </linearGradient>
    <marker id="arrowhead" markerWidth="10" markerHeight="10" refX="9" refY="5" orient="auto">
      <path d="M 0 0 L 9 5 L 0 10 z" fill="#0277bd" />
    </marker>
  </defs>
"###);
    
    // Title
    svg.push_str(&format!(
        r#"  <text x="{}" y="70" class="title">{}</text>
"#,
        width / 2, title
    ));
    
    // Visual side channel indicator
    svg.push_str(&format!(
        r#"  <!-- Side Channel (visualization) -->
  <line x1="{}" y1="100" x2="{}" y2="{}" class="side-channel"/>
"#,
        router.side_channel_x, router.side_channel_x, height - 100
    ));
    
    // Components
    for comp in &components {
        svg.push_str(&draw_ultimate_component(comp));
    }
    
    // Ultimate connectors - ZERO crossings guaranteed!
    svg.push_str("  <!-- ULTIMATE Connectors - ABSOLUTE ZERO Crossings! -->\\n");
    
    for trace in &model.traces {
        if trace.trace_type == "implements" {
            if let Some(path) = router.generate_ultimate_path(&trace.from, &trace.to) {
                svg.push_str(&format!(
                    r#"  <path d="{}" class="connector">
    <title>{} → {}</title>
  </path>
"#,
                    path, trace.from, trace.to
                ));
            }
        }
    }
    
    svg.push_str("</svg>\\n");
    Ok(svg)
}

fn draw_ultimate_component(comp: &Component) -> String {
    let cx = comp.x + comp.width / 2;
    let mut svg = String::new();
    
    svg.push_str(&format!("  <!-- {} -->\\n", comp.id));
    svg.push_str(&format!(
        r#"  <rect x="{}" y="{}" width="{}" height="{}" class="component-box"/>
"#,
        comp.x, comp.y, comp.width, comp.height
    ));
    
    svg.push_str(&format!(
        r#"  <text x="{}" y="{}" class="component-name">{}</text>
"#,
        cx, comp.y + 35, comp.name
    ));
    
    svg.push_str(&format!(
        r#"  <text x="{}" y="{}" class="component-id">{}</text>
"#,
        cx, comp.y + 55, comp.id
    ));
    
    // Ports
    let pw = 32;
    let ph = 14;
    svg.push_str(&format!(
        r#"  <rect x="{}" y="{}" width="{}" height="{}" class="port-in"/>
"#,
        cx - pw / 2, comp.y - ph / 2, pw, ph
    ));
    svg.push_str(&format!(
        r#"  <text x="{}" y="{}" class="port-text" text-anchor="middle">IN</text>
"#,
        cx, comp.y + 4
    ));
    
    svg.push_str(&format!(
        r#"  <rect x="{}" y="{}" width="{}" height="{}" class="port-out"/>
"#,
        cx - pw / 2, comp.y + comp.height - ph / 2, pw, ph
    ));
    svg.push_str(&format!(
        r#"  <text x="{}" y="{}" class="port-text" text-anchor="middle">OUT</text>
"#,
        cx, comp.y + comp.height + 4
    ));
    
    // Function area
    svg.push_str(&format!(
        r#"  <rect x="{}" y="{}" width="{}" height="{}" class="func-box"/>
"#,
        comp.x + 20, comp.y + 75, comp.width - 40, comp.height - 100
    ));
    
    svg.push_str(&format!(
        r#"  <text x="{}" y="{}" style="font-size:13px;fill:#607d8b;font-weight:700;">⚙ Functions</text>
"#,
        comp.x + 30, comp.y + 95
    ));
    
    svg
}

pub fn wrap_ultimate_html(title: &str, svg: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{} - ULTIMATE Zero-Crossing</title>
    <style>
        body {{
            margin: 0;
            padding: 0;
            font-family: 'Segoe UI', Roboto, Arial, sans-serif;
            background: linear-gradient(135deg, #0d47a1 0%, #1976d2 50%, #42a5f5 100%);
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
            max-width: 97%;
            height: auto;
            box-shadow: 0 16px 64px rgba(0,0,0,0.5);
            background: white;
            border-radius: 20px;
        }}
        .controls {{
            position: fixed;
            top: 30px;
            right: 30px;
            background: rgba(255,255,255,0.98);
            padding: 28px;
            border-radius: 20px;
            box-shadow: 0 12px 48px rgba(0,0,0,0.4);
        }}
        .controls button {{
            display: block;
            width: 180px;
            margin: 10px 0;
            padding: 14px 24px;
            border: none;
            border-radius: 10px;
            background: linear-gradient(135deg, #01579b 0%, #0277bd 100%);
            color: white;
            cursor: pointer;
            font-size: 16px;
            font-weight: 800;
            transition: all 0.3s;
            box-shadow: 0 6px 16px rgba(1,87,155,0.4);
        }}
        .controls button:hover {{
            transform: translateY(-3px);
            box-shadow: 0 8px 24px rgba(1,87,155,0.6);
        }}
        .badge {{
            position: fixed;
            top: 30px;
            left: 30px;
            background: rgba(67,160,71,0.95);
            padding: 28px;
            border-radius: 20px;
            box-shadow: 0 12px 48px rgba(0,0,0,0.4);
            font-size: 16px;
            line-height: 1.8;
            color: white;
        }}
        .badge-title {{
            font-size: 22px;
            font-weight: 900;
            margin-bottom: 12px;
        }}
        .badge-check {{
            font-weight: 700;
            font-size: 15px;
        }}
    </style>
</head>
<body>
    <div class="controls">
        <button onclick="zoomIn()">🔍 Zoom In</button>
        <button onclick="zoomOut()">🔎 Zoom Out</button>
        <button onclick="resetZoom()">↻ Reset</button>
        <button onclick="exportSVG()">💾 Export SVG</button>
    </div>
    <div class="badge">
        <div class="badge-title">✨ ULTIMATE Routing</div>
        <div class="badge-check">✓ ABSOLUTE ZERO Crossings</div>
        <div class="badge-check">✓ Side Channel Only</div>
        <div class="badge-check">✓ Capella Professional</div>
        <div class="badge-check">✓ Certification Ready</div>
    </div>
    <div id="container">
        {}
    </div>
    <script>
        let scale = 1;
        const svg = document.querySelector('svg');
        const container = document.getElementById('container');
        
        function zoomIn() {{
            scale *= 1.25;
            svg.style.transform = `scale(${{scale}})`;
        }}
        
        function zoomOut() {{
            scale /= 1.25;
            svg.style.transform = `scale(${{scale}})`;
        }}
        
        function resetZoom() {{
            scale = 1;
            svg.style.transform = 'scale(1)';
        }}
        
        function exportSVG() {{
            const blob = new Blob([svg.outerHTML], {{ type: 'image/svg+xml' }});
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'ultimate-diagram.svg';
            a.click();
            URL.revokeObjectURL(url);
        }}
        
        container.addEventListener('wheel', (e) => {{
            e.preventDefault();
            e.deltaY < 0 ? zoomIn() : zoomOut();
        }});
    </script>
</body>
</html>
"#,
        title, svg
    )
}
