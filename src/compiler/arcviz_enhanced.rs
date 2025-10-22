//! ArcViz Enhanced - Capella-level visualization engine
//! 
//! Features:
//! - Interface ports (IN/OUT) with protocols
//! - Function compartments inside components
//! - Hierarchical layer layout
//! - UML/SysML stereotypes
//! - Safety badges (ASIL)
//! - Interactive tooltips
//! - Layer filtering
//! - Smart orthogonal routing

use super::semantic::{SemanticModel, InterfacePortInfo};
use super::arcviz_capella_routing::RoutingGrid;
use super::CompilerError;
use std::collections::HashMap;

/// Enhanced component with full details
#[derive(Debug, Clone)]
pub struct EnhancedComponent {
    pub id: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub layer: String,
    pub stereotype: String,
    pub safety_level: Option<String>,
    pub asil: Option<String>,
    pub interfaces_in: Vec<InterfacePort>,
    pub interfaces_out: Vec<InterfacePort>,
    pub functions: Vec<FunctionInfo>,
}

#[derive(Debug, Clone)]
pub struct InterfacePort {
    pub name: String,
    pub protocol: Option<String>,
    pub format: Option<String>,
    pub direction: PortDirection,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub enum PortDirection {
    In,
    Out,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct EnhancedConnection {
    pub from_id: String,
    pub to_id: String,
    pub from_port: Option<String>,
    pub to_port: Option<String>,
    pub path: String,
    pub label: Option<String>,
}

#[derive(Debug)]
pub struct LayerGroup {
    pub name: String,
    pub y_offset: f64,
    pub height: f64,
    pub components: Vec<EnhancedComponent>,
}

pub struct ArcVizEnhanced {
    pub layers: Vec<LayerGroup>,
    pub connections: Vec<EnhancedConnection>,
    pub width: f64,
    pub height: f64,
}

impl ArcVizEnhanced {
    pub fn from_model(model: &SemanticModel) -> Result<Self, CompilerError> {
        let mut layers = Vec::new();
        
        // Define layer order and styling
        let layer_definitions = vec![
            ("Source", 0.0, "#E3F2FD"),
            ("Migration", 1.0, "#FFF3E0"),
            ("Target", 2.0, "#E8F5E9"),
            ("Processing", 3.0, "#F3E5F5"),
            ("Governance", 4.0, "#FCE4EC"),
            ("Integration", 5.0, "#E0F2F1"),
            ("Analytics", 6.0, "#FFF9C4"),
            ("Monitoring", 7.0, "#EFEBE9"),
        ];
        
        let layer_height = 700.0;
        let layer_spacing = 150.0;
        let margin_top = 100.0;
        
        for (layer_name, index, _color) in &layer_definitions {
            let y_offset = margin_top + index * (layer_height + layer_spacing) + 50.0;
            
            let layer_components = Self::extract_layer_components(
                model, 
                layer_name, 
                y_offset
            );
            
            if !layer_components.is_empty() {
                layers.push(LayerGroup {
                    name: layer_name.to_string(),
                    y_offset,
                    height: layer_height,
                    components: layer_components,
                });
            }
        }
        
        let connections = Self::compute_connections(model, &layers);
        
        let width = 1800.0;
        let height = margin_top + (layers.len() as f64) * (layer_height + layer_spacing) + 150.0;
        
        Ok(ArcVizEnhanced {
            layers,
            connections,
            width,
            height,
        })
    }
    
    fn extract_layer_components(
        model: &SemanticModel,
        layer_name: &str,
        y_offset: f64,
    ) -> Vec<EnhancedComponent> {
        let mut components = Vec::new();
        
        // Dynamic sizing based on content
        let base_width = 220.0;
        let h_spacing = 250.0;
        let margin_left = 120.0;
        
        let layer_components: Vec<_> = model.components.iter()
            .filter(|c| Self::component_belongs_to_layer(&c.id, layer_name))
            .collect();
        
        for (idx, comp_info) in layer_components.iter().enumerate() {
            let col = idx % 3;
            let row = idx / 3;
            
            // Extract interfaces and functions from semantic model
            let (interfaces_in, interfaces_out, functions) = 
                Self::extract_component_details(model, &comp_info.id);
            
            // Calculate dynamic size based on content
            let comp_width = Self::calculate_component_width(&comp_info.name, &functions);
            let comp_height = Self::calculate_component_height(&interfaces_in, &interfaces_out, &functions);
            
            let x = margin_left + (col as f64) * (base_width + h_spacing);
            let y = y_offset + 80.0 + (row as f64) * (comp_height + 80.0);
            
            components.push(EnhancedComponent {
                id: comp_info.id.clone(),
                name: comp_info.name.clone(),
                x,
                y,
                width: comp_width,
                height: comp_height,
                layer: layer_name.to_string(),
                stereotype: Self::infer_stereotype(&comp_info.name),
                safety_level: comp_info.safety_level.clone(),
                asil: comp_info.asil.clone(),
                interfaces_in,
                interfaces_out,
                functions,
            });
        }
        
        components
    }
    
    fn component_belongs_to_layer(comp_id: &str, layer_name: &str) -> bool {
        match layer_name {
            "Source" => comp_id.starts_with("LA-SRC"),
            "Migration" => comp_id.starts_with("LA-MIG"),
            "Target" => comp_id.starts_with("LA-TGT"),
            "Processing" => comp_id.starts_with("LA-PROC"),
            "Governance" => comp_id.starts_with("LA-GOV"),
            "Integration" => comp_id.starts_with("LA-INT"),
            "Analytics" => comp_id.starts_with("LA-ANLZ"),
            "Monitoring" => comp_id.starts_with("LA-MON"),
            _ => false,
        }
    }
    
    fn extract_component_details(
        model: &SemanticModel,
        comp_id: &str,
    ) -> (Vec<InterfacePort>, Vec<InterfacePort>, Vec<FunctionInfo>) {
        // Find the component in the semantic model
        let comp_info = model.components.iter()
            .find(|c| c.id == comp_id);
        
        if let Some(comp) = comp_info {
            // Extract interface_in ports
            let interfaces_in: Vec<InterfacePort> = comp.interfaces_in.iter()
                .enumerate()
                .map(|(idx, iface)| InterfacePort {
                    name: iface.name.clone(),
                    protocol: iface.protocol.clone(),
                    format: iface.format.clone(),
                    direction: PortDirection::In,
                    x: 0.0, // Will be positioned during rendering
                    y: (idx as f64) * 25.0,
                })
                .collect();
            
            // Extract interface_out ports
            let interfaces_out: Vec<InterfacePort> = comp.interfaces_out.iter()
                .enumerate()
                .map(|(idx, iface)| InterfacePort {
                    name: iface.name.clone(),
                    protocol: iface.protocol.clone(),
                    format: iface.format.clone(),
                    direction: PortDirection::Out,
                    x: 0.0,
                    y: (idx as f64) * 25.0,
                })
                .collect();
            
            // Extract functions from component
            let functions: Vec<FunctionInfo> = comp.functions.iter()
                .map(|fname| {
                    // Try to find full function info
                    let func_info = model.functions.iter()
                        .find(|f| f.name == *fname || f.id.contains(fname));
                    
                    FunctionInfo {
                        id: func_info.map(|f| f.id.clone()).unwrap_or_else(|| fname.clone()),
                        name: fname.clone(),
                        description: "".to_string(),
                    }
                })
                .collect();
            
            (interfaces_in, interfaces_out, functions)
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        }
    }
    
    fn infer_stereotype(name: &str) -> String {
        if name.contains("Database") || name.contains("Storage") {
            "<<datastore>>".to_string()
        } else if name.contains("Orchestrator") || name.contains("Engine") {
            "<<process>>".to_string()
        } else if name.contains("API") || name.contains("Gateway") {
            "<<interface>>".to_string()
        } else if name.contains("Validator") || name.contains("Resolver") {
            "<<service>>".to_string()
        } else {
            "<<component>>".to_string()
        }
    }
    
    fn compute_connections(
        model: &SemanticModel,
        layers: &[LayerGroup],
    ) -> Vec<EnhancedConnection> {
        let mut connections = Vec::new();
        
        // Build component lookup with layer indices
        let mut comp_map = HashMap::new();
        for (layer_idx, layer) in layers.iter().enumerate() {
            for comp in &layer.components {
                comp_map.insert(comp.id.clone(), (comp.clone(), layer_idx));
            }
        }
        
        // Create routing grid - with proper spacing for routing
        let width = 1800.0;
        let height = 100.0 + (layers.len() as f64) * 850.0;
        let mut routing_grid = RoutingGrid::new(width, height, layers.len());
        
        // Register all components in routing grid
        for (layer_idx, layer) in layers.iter().enumerate() {
            for comp in &layer.components {
                routing_grid.add_component(
                    comp.id.clone(),
                    comp.x,
                    comp.y,
                    comp.width,
                    comp.height,
                    layer_idx,
                );
            }
        }
        
        // Create connections from interfaces using professional routing
        for interface in &model.interfaces {
            if let Some(routed) = routing_grid.route_connection(
                &interface.from,
                &interface.to,
                None,
                None,
            ) {
                connections.push(EnhancedConnection {
                    from_id: interface.from.clone(),
                    to_id: interface.to.clone(),
                    from_port: None,
                    to_port: None,
                    path: routed.path,
                    label: Some(interface.name.clone()),
                });
            }
        }
        
        connections
    }
    
    fn compute_orthogonal_path(from: &EnhancedComponent, to: &EnhancedComponent) -> String {
        // Professional routing with side channels to avoid overlaps
        let start_x = from.x + from.width / 2.0;
        let start_y = from.y + from.height;
        let end_x = to.x + to.width / 2.0;
        let end_y = to.y;
        
        // Check if routing vertically (same column)
        if (start_x - end_x).abs() < 50.0 {
            // Direct vertical connection
            format!("M {} {} L {} {}", start_x, start_y, end_x, end_y)
        } else if start_y < end_y {
            // Going down - use vertical then horizontal then vertical
            let exit_y = start_y + 60.0;
            let entry_y = end_y - 60.0;
            format!(
                "M {} {} L {} {} L {} {} L {} {}",
                start_x, start_y,
                start_x, exit_y,
                end_x, entry_y,
                end_x, end_y
            )
        } else {
            // Going up - use side channel routing
            let side_channel_x = if start_x < end_x {
                from.x + from.width + 100.0
            } else {
                from.x - 100.0
            };
            
            let exit_y = start_y + 40.0;
            let entry_y = end_y - 40.0;
            
            format!(
                "M {} {} L {} {} L {} {} L {} {} L {} {} L {} {}",
                start_x, start_y,
                start_x, exit_y,
                side_channel_x, exit_y,
                side_channel_x, entry_y,
                end_x, entry_y,
                end_x, end_y
            )
        }
    }
    
    pub fn generate_html(&self) -> String {
        let mut html = String::new();
        
        html.push_str(&self.generate_html_header());
        html.push_str(&self.generate_svg_content());
        html.push_str(&self.generate_html_footer());
        
        html
    }
    
    fn generate_html_header(&self) -> String {
        format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>ArcViz Enhanced - Capella-Level Visualization</title>
    <style>
        body {{
            margin: 0;
            padding: 0;
            font-family: 'Segoe UI', Arial, sans-serif;
            background: linear-gradient(135deg, #1a237e 0%, #283593 50%, #3949ab 100%);
            overflow: auto;
        }}
        #container {{
            width: 100vw;
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: flex-start;
            padding: 40px;
            box-sizing: border-box;
        }}
        svg {{
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 80px rgba(0,0,0,0.6);
        }}
        .component {{
            cursor: pointer;
            transition: all 0.3s;
        }}
        .component:hover {{
            filter: drop-shadow(0 8px 16px rgba(0,0,0,0.3));
        }}
        .component-rect {{
            fill: white;
            stroke: #1976d2;
            stroke-width: 2;
        }}
        .component-header {{
            fill: #1976d2;
        }}
        .component-name {{
            fill: white;
            font-size: 14px;
            font-weight: bold;
        }}
        .stereotype {{
            fill: white;
            font-size: 10px;
            font-style: italic;
        }}
        .interface-port {{
            fill: #4caf50;
            stroke: #2e7d32;
            stroke-width: 1.5;
        }}
        .interface-port.out {{
            fill: #ff9800;
            stroke: #e65100;
        }}
        .interface-label {{
            font-size: 10px;
            fill: #0d47a1;
            font-weight: 600;
        }}
        .port-protocol {{
            font-size: 8px;
            fill: #607d8b;
            font-style: italic;
        }}
        .function-text {{
            font-size: 11px;
            fill: #37474f;
            font-weight: 500;
        }}
        .function-separator {{
            stroke: #e0e0e0;
            stroke-width: 1;
        }}
        .connection {{
            fill: none;
            stroke: #666;
            stroke-width: 2;
            marker-end: url(#arrowhead);
        }}
        .connection-label {{
            font-size: 10px;
            fill: #666;
            text-anchor: middle;
        }}
        .layer-group {{
            fill: none;
            stroke: #90a4ae;
            stroke-width: 2;
            stroke-dasharray: 10, 5;
            rx: 8;
        }}
        .layer-label {{
            font-size: 20px;
            font-weight: bold;
            fill: #37474f;
        }}
        .safety-badge {{
            font-size: 10px;
            font-weight: bold;
        }}
        .asil-b {{
            fill: #ff9800;
        }}
        .asil-c {{
            fill: #f44336;
        }}
        .tooltip {{
            position: absolute;
            background: rgba(33, 33, 33, 0.95);
            color: white;
            padding: 12px 16px;
            border-radius: 8px;
            font-size: 12px;
            pointer-events: none;
            display: none;
            z-index: 1000;
            max-width: 300px;
            box-shadow: 0 4px 16px rgba(0,0,0,0.4);
        }}
        .controls {{
            position: fixed;
            top: 20px;
            right: 20px;
            background: white;
            padding: 20px;
            border-radius: 12px;
            box-shadow: 0 8px 32px rgba(0,0,0,0.3);
            z-index: 100;
        }}
        .controls h3 {{
            margin: 0 0 12px 0;
            font-size: 16px;
            color: #1976d2;
        }}
        .controls label {{
            display: block;
            margin: 8px 0;
            font-size: 13px;
            cursor: pointer;
        }}
        .controls input[type="checkbox"] {{
            margin-right: 8px;
        }}
    </style>
</head>
<body>
    <div class="controls">
        <h3>🔧 Layer Controls</h3>
        <label><input type="checkbox" checked onchange="toggleLayer('Source')"> Source Layer</label>
        <label><input type="checkbox" checked onchange="toggleLayer('Migration')"> Migration Layer</label>
        <label><input type="checkbox" checked onchange="toggleLayer('Target')"> Target Layer</label>
        <label><input type="checkbox" checked onchange="toggleLayer('Processing')"> Processing Layer</label>
        <label><input type="checkbox" checked onchange="toggleLayer('Governance')"> Governance Layer</label>
        <label><input type="checkbox" checked onchange="toggleLayer('Integration')"> Integration Layer</label>
        <label><input type="checkbox" checked onchange="toggleLayer('Analytics')"> Analytics Layer</label>
        <label><input type="checkbox" checked onchange="toggleLayer('Monitoring')"> Monitoring Layer</label>
    </div>
    <div id="container">
        <div id="tooltip" class="tooltip"></div>
"#)
    }
    
    fn generate_svg_content(&self) -> String {
        let mut svg = format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
            self.width, self.height
        );
        
        svg.push_str("
    <defs>
        <marker id=\"arrowhead\" markerWidth=\"10\" markerHeight=\"10\" refX=\"9\" refY=\"3\" orient=\"auto\">
            <polygon points=\"0 0, 10 3, 0 6\" fill=\"#666\" />
        </marker>
    </defs>
");
        
        // Render layers
        for layer in &self.layers {
            svg.push_str(&self.render_layer(layer));
        }
        
        // Render connections
        svg.push_str(r#"<g id="connections">"#);
        for conn in &self.connections {
            svg.push_str(&self.render_connection(conn));
        }
        svg.push_str("</g>");
        
        svg.push_str("</svg>");
        svg
    }
    
    fn render_layer(&self, layer: &LayerGroup) -> String {
        let mut svg = format!(
            r#"<g id="layer-{}" class="layer">"#,
            layer.name.replace(" ", "-")
        );
        
        // Layer background - more compact
        svg.push_str(&format!(
            r#"<rect class="layer-group" x="30" y="{}" width="{}" height="{}" />"#,
            layer.y_offset - 40.0,
            self.width - 60.0,
            layer.height + 80.0
        ));
        
        // Layer label
        svg.push_str(&format!(
            r#"<text class="layer-label" x="70" y="{}">{} Layer</text>"#,
            layer.y_offset - 20.0,
            layer.name
        ));
        
        // Render components
        for comp in &layer.components {
            svg.push_str(&self.render_enhanced_component(comp));
        }
        
        svg.push_str("</g>");
        svg
    }
    
    fn render_enhanced_component(&self, comp: &EnhancedComponent) -> String {
        let mut svg = format!(
            r#"<g class="component" data-id="{}" onmouseover="showTooltip(evt, '{}')" onmouseout="hideTooltip()">"#,
            comp.id, comp.id
        );
        
        // Main component rectangle
        svg.push_str(&format!(
            r#"<rect class="component-rect" x="{}" y="{}" width="{}" height="{}" rx="4" />"#,
            comp.x, comp.y, comp.width, comp.height
        ));
        
        // Header background
        let header_height = 60.0;
        svg.push_str(&format!(
            r#"<rect class="component-header" x="{}" y="{}" width="{}" height="{}" rx="4" />"#,
            comp.x, comp.y, comp.width, header_height
        ));
        
        // Stereotype
        svg.push_str(&format!(
            r#"<text class="stereotype" x="{}" y="{}" text-anchor="middle">{}</text>"#,
            comp.x + comp.width / 2.0,
            comp.y + 20.0,
            comp.stereotype
        ));
        
        // Component name
        svg.push_str(&format!(
            r#"<text class="component-name" x="{}" y="{}" text-anchor="middle">{}</text>"#,
            comp.x + comp.width / 2.0,
            comp.y + 42.0,
            Self::truncate_text(&comp.name, 28)
        ));
        
        // ASIL badge if present
        if let Some(asil) = &comp.asil {
            let badge_class = if asil.contains("_B") { "asil-b" } else { "asil-c" };
            svg.push_str(&format!(
                r#"<circle class="safety-badge {}" cx="{}" cy="{}" r="12" />"#,
                badge_class,
                comp.x + comp.width - 25.0,
                comp.y + 20.0
            ));
            svg.push_str(&format!(
                r#"<text class="safety-badge" x="{}" y="{}" text-anchor="middle" fill="white">{}</text>"#,
                comp.x + comp.width - 25.0,
                comp.y + 24.0,
                asil.replace("ASIL_", "")
            ));
        }
        
        // Function compartment
        let func_y_start = comp.y + header_height + 10.0;
        svg.push_str(&format!(
            r#"<line class="function-separator" x1="{}" y1="{}" x2="{}" y2="{}" />"#,
            comp.x, func_y_start - 5.0,
            comp.x + comp.width, func_y_start - 5.0
        ));
        
        // Render functions (max 6 for compact display)
        let max_funcs = 6;
        for (idx, func) in comp.functions.iter().take(max_funcs).enumerate() {
            let y = func_y_start + (idx as f64) * 18.0;
            svg.push_str(&format!(
                r#"<text class="function-text" x="{}" y="{}"  >+ {}</text>"#,
                comp.x + 12.0,
                y,
                Self::truncate_text(&func.name, 28)
            ));
        }
        
        if comp.functions.len() > max_funcs {
            let y = func_y_start + (max_funcs as f64) * 18.0;
            svg.push_str(&format!(
                r#"<text class="function-text" x="{}" y="{}" font-style="italic" opacity="0.7">+{} more</text>"#,
                comp.x + 12.0,
                y,
                comp.functions.len() - max_funcs
            ));
        }
        
        // Interface ports - left side (IN) - More compact
        let port_size = 14.0;
        let port_spacing = 50.0;
        let port_start_y = comp.y + header_height + 140.0;
        let max_ports_display = 3; // Limit ports displayed
        
        for (idx, port) in comp.interfaces_in.iter().take(max_ports_display).enumerate() {
            let py = port_start_y + (idx as f64) * port_spacing;
            svg.push_str(&format!(
                r#"<rect class="interface-port" x="{}" y="{}" width="{}" height="{}" rx="2" />"#,
                comp.x - port_size / 2.0,
                py - port_size / 2.0,
                port_size,
                port_size
            ));
            
            // Port name - positioned well away from component
            svg.push_str(&format!(
                r#"<text class="interface-label" x="{}" y="{}" text-anchor="end">{}</text>"#,
                comp.x - 22.0,
                py + 3.0,
                Self::truncate_text(&port.name, 14)
            ));
            
            // Protocol label - smaller, below name
            if let Some(protocol) = &port.protocol {
                svg.push_str(&format!(
                    r#"<text class="port-protocol" x="{}" y="{}" text-anchor="end">{}</text>"#,
                    comp.x - 22.0,
                    py + 14.0,
                    Self::truncate_text(protocol, 16)
                ));
            }
        }
        
        // Show ellipsis if more ports
        if comp.interfaces_in.len() > max_ports_display {
            let py = port_start_y + (max_ports_display as f64) * port_spacing;
            svg.push_str(&format!(
                r#"<text class="port-protocol" x="{}" y="{}" text-anchor="end">+{} more</text>"#,
                comp.x - 22.0,
                py,
                comp.interfaces_in.len() - max_ports_display
            ));
        }
        
        // Interface ports - right side (OUT)
        for (idx, port) in comp.interfaces_out.iter().take(max_ports_display).enumerate() {
            let py = port_start_y + (idx as f64) * port_spacing;
            svg.push_str(&format!(
                r#"<rect class="interface-port out" x="{}" y="{}" width="{}" height="{}" rx="2" />"#,
                comp.x + comp.width - port_size / 2.0,
                py - port_size / 2.0,
                port_size,
                port_size
            ));
            
            // Port name
            svg.push_str(&format!(
                r#"<text class="interface-label" x="{}" y="{}">{}</text>"#,
                comp.x + comp.width + 22.0,
                py + 3.0,
                Self::truncate_text(&port.name, 14)
            ));
            
            // Protocol label
            if let Some(protocol) = &port.protocol {
                svg.push_str(&format!(
                    r#"<text class="port-protocol" x="{}" y="{}">{}</text>"#,
                    comp.x + comp.width + 22.0,
                    py + 14.0,
                    Self::truncate_text(protocol, 16)
                ));
            }
        }
        
        // Show ellipsis if more ports
        if comp.interfaces_out.len() > max_ports_display {
            let py = port_start_y + (max_ports_display as f64) * port_spacing;
            svg.push_str(&format!(
                r#"<text class="port-protocol" x="{}" y="{}"  >+{} more</text>"#,
                comp.x + comp.width + 22.0,
                py,
                comp.interfaces_out.len() - max_ports_display
            ));
        }
        
        svg.push_str("</g>");
        svg
    }
    
    fn render_connection(&self, conn: &EnhancedConnection) -> String {
        let mut svg = format!(
            r#"<path class="connection" d="{}" />"#,
            conn.path
        );
        
        if let Some(label) = &conn.label {
            // Parse path to find horizontal segment for label placement
            let path_parts: Vec<&str> = conn.path.split_whitespace().collect();
            
            // Find a horizontal segment (consecutive points with same Y)
            let mut label_placed = false;
            for i in (1..path_parts.len()).step_by(3) {
                if i + 4 < path_parts.len() && path_parts[i] == "L" {
                    if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                        path_parts.get(i + 1).and_then(|s| s.parse::<f64>().ok()),
                        path_parts.get(i + 2).and_then(|s| s.parse::<f64>().ok()),
                        path_parts.get(i + 4).and_then(|s| s.parse::<f64>().ok()),
                        path_parts.get(i + 5).and_then(|s| s.parse::<f64>().ok()),
                    ) {
                        // Check if horizontal segment
                        if (y1 - y2).abs() < 5.0 && (x2 - x1).abs() > 50.0 {
                            let label_x = (x1 + x2) / 2.0;
                            let label_y = y1 - 8.0;
                            
                            svg.push_str(&format!(
                                r#"<rect x="{}" y="{}" width="{}" height="18" fill="white" opacity="0.9" rx="3"/>"#,
                                label_x - 40.0,
                                label_y - 12.0,
                                80.0
                            ));
                            
                            svg.push_str(&format!(
                                r#"<text class="connection-label" x="{}" y="{}">{}</text>"#,
                                label_x,
                                label_y,
                                Self::truncate_text(label, 18)
                            ));
                            
                            label_placed = true;
                            break;
                        }
                    }
                }
            }
            
            // If no horizontal segment found, place at midpoint
            if !label_placed && path_parts.len() >= 6 {
                let mid_idx = path_parts.len() / 2;
                if let (Some(x), Some(y)) = (
                    path_parts.get(mid_idx).and_then(|s| s.parse::<f64>().ok()),
                    path_parts.get(mid_idx + 1).and_then(|s| s.parse::<f64>().ok()),
                ) {
                    svg.push_str(&format!(
                        r#"<text class="connection-label" x="{}" y="{}">{}</text>"#,
                        x, y - 8.0,
                        Self::truncate_text(label, 18)
                    ));
                }
            }
        }
        
        svg
    }
    
    fn generate_html_footer(&self) -> String {
        // Generate JavaScript metadata for all components
        let mut metadata_js = String::from("const componentMetadata = {\n");
        
        for layer in &self.layers {
            for comp in &layer.components {
                let asil_str = comp.asil.as_ref()
                    .map(|a| format!("'{}'", a))
                    .unwrap_or_else(|| "null".to_string());
                
                metadata_js.push_str(&format!(
                    "    '{}': {{ id: '{}', name: '{}', stereotype: '{}', layer: '{}', asil: {}, functionCount: {}, interfacesIn: {}, interfacesOut: {} }},\n",
                    comp.id,
                    comp.id,
                    comp.name.replace("'", "\\'"),
                    comp.stereotype.replace("'", "\\'"),
                    comp.layer,
                    asil_str,
                    comp.functions.len(),
                    comp.interfaces_in.len(),
                    comp.interfaces_out.len()
                ));
            }
        }
        
        metadata_js.push_str("};\n");
        
        format!(r#"
    </div>
    <script>
        {}
        
        function toggleLayer(layerName) {{
            const layer = document.getElementById('layer-' + layerName);
            if (layer) {{
                layer.style.display = layer.style.display === 'none' ? 'block' : 'none';
            }}
        }}
        
        function showTooltip(evt, compId) {{
            const tooltip = document.getElementById('tooltip');
            const metadata = componentMetadata[compId];
            
            if (!metadata) return;
            
            tooltip.innerHTML = `
                <strong>${{metadata.name}}</strong><br/>
                <em>${{metadata.stereotype}}</em><br/>
                <br/>
                <strong>ID:</strong> ${{metadata.id}}<br/>
                <strong>Layer:</strong> ${{metadata.layer}}<br/>
                ${{metadata.asil ? `<strong>Safety:</strong> ${{metadata.asil}}<br/>` : ''}}
                <br/>
                <strong>Functions:</strong> ${{metadata.functionCount}}<br/>
                <strong>Interfaces IN:</strong> ${{metadata.interfacesIn}}<br/>
                <strong>Interfaces OUT:</strong> ${{metadata.interfacesOut}}
            `;
            
            tooltip.style.display = 'block';
            tooltip.style.left = (evt.pageX + 15) + 'px';
            tooltip.style.top = (evt.pageY + 15) + 'px';
        }}
        
        function hideTooltip() {{
            const tooltip = document.getElementById('tooltip');
            tooltip.style.display = 'none';
        }}
        
        // Zoom and pan functionality
        let scale = 1;
        const svg = document.querySelector('svg');
        const container = document.getElementById('container');
        
        container.addEventListener('wheel', (e) => {{
            e.preventDefault();
            const delta = e.deltaY > 0 ? 0.9 : 1.1;
            scale *= delta;
            scale = Math.max(0.1, Math.min(scale, 5));
            svg.style.transform = `scale(${{scale}})`;
        }});
    </script>
</body>
</html>
"#, metadata_js)
    }
    
    fn truncate_text(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            format!("{}...", &text[0..max_len - 3])
        }
    }
    
    fn calculate_component_width(name: &str, functions: &[FunctionInfo]) -> f64 {
        let name_width = name.len() as f64 * 7.0;
        
        let max_func_width = functions.iter()
            .map(|f| f.name.len() as f64 * 5.5)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        let content_width = name_width.max(max_func_width);
        (content_width + 50.0).max(180.0).min(320.0)
    }
    
    fn calculate_component_height(
        interfaces_in: &[InterfacePort],
        interfaces_out: &[InterfacePort],
        functions: &[FunctionInfo]
    ) -> f64 {
        let header_height = 60.0;
        let max_funcs_displayed = 6;
        let func_height = (functions.len().min(max_funcs_displayed) as f64) * 18.0 + 20.0;
        
        let max_ports = interfaces_in.len().max(interfaces_out.len()).min(3);
        let port_height = if max_ports > 0 {
            (max_ports as f64) * 50.0 + 140.0
        } else {
            20.0
        };
        
        (header_height + func_height + port_height).max(200.0).min(500.0)
    }
}

pub fn generate_enhanced_html(model: &SemanticModel) -> Result<String, CompilerError> {
    let viz = ArcVizEnhanced::from_model(model)?;
    Ok(viz.generate_html())
}
