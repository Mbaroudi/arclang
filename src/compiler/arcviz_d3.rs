//! ArcViz D3 - Professional visualization using D3.js + Dagre
//! 
//! Architecture:
//! - D3.js: DOM/SVG manipulation and rendering
//! - Dagre: Hierarchical graph layout algorithm
//! - Rust: Generate JSON graph data from SemanticModel
//! - Browser: Client-side rendering with automatic text collision avoidance
//! 
//! Benefits:
//! - Zero overlaps guaranteed by Dagre layout engine
//! - Automatic text measurement and positioning
//! - Interactive pan/zoom with D3
//! - Professional hierarchical layout
//! - Scales to large graphs (1000+ nodes)

use super::semantic::SemanticModel;
use super::CompilerError;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Graph node for Dagre layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub layer: String,
    pub stereotype: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asil: Option<String>,
    pub interfaces_in: Vec<InterfacePort>,
    pub interfaces_out: Vec<InterfacePort>,
    pub functions: Vec<String>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfacePort {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Graph edge for Dagre layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub label: String,
}

/// Complete graph structure
#[derive(Debug, Serialize, Deserialize)]
pub struct DagreGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub layers: Vec<LayerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerInfo {
    pub name: String,
    pub color: String,
    pub description: String,
}

impl DagreGraph {
    /// Build graph from semantic model
    pub fn from_model(model: &SemanticModel) -> Result<Self, CompilerError> {
        // Dynamically extract layers from components
        let mut layer_set = std::collections::HashSet::new();
        for comp in &model.components {
            if !comp.level.is_empty() {
                layer_set.insert(comp.level.clone());
            }
        }
        
        // Create layer configs with colors
        let layer_colors: HashMap<&str, (&str, &str)> = [
            ("User", ("#E1F5FE", "User Interface Layer")),
            ("Connectivity", ("#FFF3E0", "Connectivity & Communication Layer")),
            ("Control", ("#F3E5F5", "Control & Logic Layer")),
            ("Vehicle", ("#E8F5E9", "Vehicle Systems Layer")),
            ("Physical", ("#FCE4EC", "Physical Hardware Layer")),
            ("Logical", ("#E3F2FD", "Logical Architecture Layer")),
            ("Application", ("#FFF9C4", "Application Layer")),
            ("Service", ("#E0F2F1", "Service Layer")),
        ].iter().cloned().collect();
        
        let mut layers = Vec::new();
        for layer_name in layer_set {
            let (color, desc) = layer_colors.get(layer_name.as_str())
                .unwrap_or(&("#EFEBE9", "Architecture Layer"));
            layers.push(LayerInfo {
                name: layer_name.clone(),
                color: color.to_string(),
                description: desc.to_string(),
            });
        }
        
        // Sort layers for consistent ordering
        layers.sort_by(|a, b| a.name.cmp(&b.name));
        
        let mut nodes = Vec::new();
        
        // Extract nodes from model
        for comp in &model.components {
            let layer = comp.level.clone();
            
            let interfaces_in: Vec<InterfacePort> = comp.interfaces_in.iter()
                .map(|iface| InterfacePort {
                    name: iface.name.clone(),
                    protocol: iface.protocol.clone(),
                    format: iface.format.clone(),
                })
                .collect();
            
            let interfaces_out: Vec<InterfacePort> = comp.interfaces_out.iter()
                .map(|iface| InterfacePort {
                    name: iface.name.clone(),
                    protocol: iface.protocol.clone(),
                    format: iface.format.clone(),
                })
                .collect();
            
            let functions: Vec<String> = comp.functions.clone();
            
            // Calculate node dimensions based on content
            let width = calculate_node_width(&comp.name, &functions);
            let height = calculate_node_height(&interfaces_in, &interfaces_out, &functions);
            
            nodes.push(GraphNode {
                id: comp.id.clone(),
                label: comp.name.clone(),
                layer,
                stereotype: infer_stereotype(&comp.name),
                safety_level: comp.safety_level.clone(),
                asil: comp.asil.clone(),
                interfaces_in,
                interfaces_out,
                functions,
                width,
                height,
            });
        }
        
        // Extract edges
        let mut edges = Vec::new();
        for interface in &model.interfaces {
            edges.push(GraphEdge {
                source: interface.from.clone(),
                target: interface.to.clone(),
                label: interface.name.clone(),
            });
        }
        
        Ok(DagreGraph {
            nodes,
            edges,
            layers,
        })
    }
    
    /// Generate complete HTML with D3.js visualization
    pub fn generate_html(&self) -> Result<String, CompilerError> {
        let graph_json = serde_json::to_string_pretty(self)
            .map_err(|e| CompilerError::Semantic(format!("JSON serialization error: {}", e)))?;
        
        Ok(format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>ArcViz D3 - Enterprise Data Platform Migration</title>
    <script src="https://d3js.org/d3.v7.min.js"></script>
    <script src="https://unpkg.com/dagre-d3@0.6.4/dist/dagre-d3.min.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/jspdf/2.5.1/jspdf.umd.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/svg2pdf.js@2.2.1/dist/svg2pdf.min.js"></script>
    <style>
        body {{
            margin: 0;
            padding: 0;
            font-family: 'Segoe UI', 'San Francisco', 'Helvetica Neue', Arial, sans-serif;
            background: linear-gradient(135deg, #1a237e 0%, #283593 50%, #3949ab 100%);
            overflow: hidden;
        }}
        
        #container {{
            width: 100vw;
            height: 100vh;
            display: flex;
            flex-direction: column;
            align-items: center;
            padding: 20px;
            box-sizing: border-box;
        }}
        
        .header {{
            color: white;
            text-align: center;
            margin-bottom: 20px;
            z-index: 10;
        }}
        
        .header h1 {{
            font-size: 28px;
            margin: 0 0 5px 0;
            font-weight: 300;
            letter-spacing: 0.5px;
        }}
        
        .header p {{
            font-size: 13px;
            opacity: 0.9;
            margin: 0;
        }}
        
        #viz-container {{
            flex: 1;
            width: 100%;
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 80px rgba(0,0,0,0.6);
            overflow: hidden;
            position: relative;
        }}
        
        svg {{
            width: 100%;
            height: 100%;
        }}
        
        /* Node styles */
        .node {{
            cursor: pointer;
            transition: all 0.3s;
        }}
        
        .node rect {{
            fill: white;
            stroke: #1976d2;
            stroke-width: 2.5px;
            rx: 6px;
            filter: drop-shadow(0px 2px 4px rgba(0,0,0,0.2));
        }}
        
        .node:hover rect {{
            stroke: #0d47a1;
            stroke-width: 3px;
            filter: drop-shadow(0px 4px 8px rgba(0,0,0,0.3));
        }}
        
        .node-header {{
            fill: #1976d2;
        }}
        
        .node-label {{
            fill: white;
            font-size: 12px;
            font-weight: 600;
            text-anchor: middle;
        }}
        
        .node-stereotype {{
            fill: white;
            font-size: 8px;
            font-style: italic;
            text-anchor: middle;
        }}
        
        .node-function {{
            fill: #37474f;
            font-size: 9px;
            font-weight: 500;
        }}
        
        .interface-port {{
            fill: #4caf50;
            stroke: #2e7d32;
            stroke-width: 1.5px;
        }}
        
        .interface-port.out {{
            fill: #ff9800;
            stroke: #e65100;
        }}
        
        .port-label {{
            fill: #01579b;
            font-size: 8px;
            font-weight: 700;
        }}
        
        .port-protocol {{
            fill: #546e7a;
            font-size: 7px;
            font-style: italic;
        }}
        
        /* Edge styles */
        .edgePath path {{
            stroke: #607d8b;
            stroke-width: 2.5px;
            fill: none;
            opacity: 0.8;
        }}
        
        .edgePath:hover path {{
            stroke: #1976d2;
            stroke-width: 3.5px;
            opacity: 1;
        }}
        
        .edgeLabel {{
            background: white;
            border-radius: 4px;
            padding: 4px 8px;
            border: 1px solid #b0bec5;
            font-size: 9px;
            font-weight: 700;
            color: #37474f;
        }}
        
        /* ASIL badges */
        .asil-badge {{
            font-size: 7px;
            font-weight: bold;
            fill: white;
            text-anchor: middle;
        }}
        
        .asil-b {{
            fill: #ff9800;
        }}
        
        .asil-c {{
            fill: #f44336;
        }}
        
        /* Controls */
        .controls {{
            position: absolute;
            top: 20px;
            right: 20px;
            background: white;
            padding: 12px;
            border-radius: 8px;
            box-shadow: 0 4px 16px rgba(0,0,0,0.2);
            z-index: 100;
        }}
        
        .controls button {{
            display: block;
            width: 100%;
            margin: 5px 0;
            padding: 8px 16px;
            border: none;
            background: #1976d2;
            color: white;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
            font-weight: 600;
        }}
        
        .controls button:hover {{
            background: #0d47a1;
        }}
        
        /* Legend */
        .legend {{
            position: absolute;
            bottom: 20px;
            right: 20px;
            background: white;
            padding: 16px;
            border-radius: 8px;
            box-shadow: 0 4px 16px rgba(0,0,0,0.2);
            font-size: 11px;
            z-index: 100;
        }}
        
        .legend-title {{
            font-weight: 600;
            margin-bottom: 8px;
            color: #1976d2;
        }}
        
        .legend-item {{
            margin: 4px 0;
            display: flex;
            align-items: center;
        }}
        
        .legend-color {{
            width: 20px;
            height: 12px;
            margin-right: 8px;
            border-radius: 2px;
        }}
        
        /* Layer backgrounds */
        .layer-bg {{
            fill-opacity: 0.15;
            stroke: #37474f;
            stroke-width: 2;
            stroke-dasharray: 8, 4;
            rx: 12px;
        }}
        
        .layer-label {{
            font-size: 16px;
            font-weight: 600;
            fill: #263238;
        }}
    </style>
</head>
<body>
    <div id="container">
        <div class="header">
            <h1>Enterprise Data Platform Migration</h1>
            <p>Oracle/Snowflake → Databricks | D3.js + Dagre Automatic Layout</p>
        </div>
        <div id="viz-container">
            <svg id="svg-canvas"></svg>
            <div class="controls">
                <button onclick="resetZoom()">🔄 Reset Zoom</button>
                <button onclick="fitToScreen()">📐 Fit to Screen</button>
                <button onclick="exportSVG()">📄 Export SVG</button>
                <button onclick="exportPDF()">📕 Export PDF</button>
                <button onclick="exportPNG()">🖼️ Export PNG</button>
            </div>
            <div class="legend">
                <div class="legend-title">Legend</div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #4caf50;"></div>
                    <span>Input Port</span>
                </div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #ff9800;"></div>
                    <span>Output Port</span>
                </div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #ff9800;"></div>
                    <span>ASIL B</span>
                </div>
                <div class="legend-item">
                    <div class="legend-color" style="background: #f44336;"></div>
                    <span>ASIL C</span>
                </div>
            </div>
        </div>
    </div>

    <script>
        // Graph data from Rust
        const graphData = {graph_json};
        
        // Create Dagre graph
        const g = new dagreD3.graphlib.Graph({{compound: true}})
            .setGraph({{
                rankdir: 'TB',
                nodesep: 100,
                ranksep: 150,
                marginx: 50,
                marginy: 50
            }})
            .setDefaultEdgeLabel(() => ({{}}));
        
        // Group nodes by layer
        const nodesByLayer = {{}};
        graphData.nodes.forEach(node => {{
            if (!nodesByLayer[node.layer]) {{
                nodesByLayer[node.layer] = [];
            }}
            nodesByLayer[node.layer].push(node);
        }});
        
        // Add layer clusters
        graphData.layers.forEach(layer => {{
            if (nodesByLayer[layer.name] && nodesByLayer[layer.name].length > 0) {{
                g.setNode(layer.name, {{
                    label: layer.name + ' Layer',
                    clusterLabelPos: 'top',
                    style: `fill: ${{layer.color}}; stroke: #37474f; stroke-width: 2px; stroke-dasharray: 8, 4;`,
                    rx: 12,
                    ry: 12
                }});
            }}
        }});
        
        // Add nodes to graph
        graphData.nodes.forEach(node => {{
            const label = createNodeLabel(node);
            
            g.setNode(node.id, {{
                labelType: 'html',
                label: label,
                width: node.width,
                height: node.height,
                class: 'node'
            }});
            
            // Assign to layer cluster
            g.setParent(node.id, node.layer);
        }});
        
        // Add edges
        graphData.edges.forEach(edge => {{
            g.setEdge(edge.source, edge.target, {{
                label: edge.label,
                curve: d3.curveBasis,
                arrowheadClass: 'arrowhead'
            }});
        }});
        
        // Create SVG and renderer
        const svg = d3.select('#svg-canvas');
        const svgGroup = svg.append('g');
        
        // Create renderer
        const render = new dagreD3.render();
        
        // Run the renderer
        render(svgGroup, g);
        
        // Center and fit the graph
        const initialScale = 0.75;
        const xCenterOffset = (svg.node().getBoundingClientRect().width - g.graph().width * initialScale) / 2;
        const yCenterOffset = 50;
        
        svg.call(
            d3.zoom()
                .scaleExtent([0.1, 3])
                .on('zoom', (event) => {{
                    svgGroup.attr('transform', event.transform);
                }})
        ).call(
            d3.zoom().transform,
            d3.zoomIdentity.translate(xCenterOffset, yCenterOffset).scale(initialScale)
        );
        
        // Helper function to create node HTML
        function createNodeLabel(node) {{
            let html = `<div style="padding: 0;">`;
            
            // Header
            html += `<div class="node-header" style="background: #1976d2; padding: 8px; border-radius: 6px 6px 0 0;">`;
            html += `<div class="node-stereotype" style="font-size: 8px; color: white; opacity: 0.9;">${{node.stereotype}}</div>`;
            html += `<div class="node-label" style="font-size: 12px; color: white; font-weight: 600; margin-top: 2px;">${{truncate(node.label, 26)}}</div>`;
            
            // ASIL badge
            if (node.asil) {{
                const badgeColor = node.asil.includes('_B') ? '#ff9800' : '#f44336';
                html += `<div style="position: absolute; top: 8px; right: 8px; background: ${{badgeColor}}; color: white; border-radius: 50%; width: 18px; height: 18px; display: flex; align-items: center; justify-content: center; font-size: 7px; font-weight: bold;">${{node.asil.replace('ASIL_', '')}}</div>`;
            }}
            html += `</div>`;
            
            // Functions
            if (node.functions && node.functions.length > 0) {{
                html += `<div style="padding: 8px; border-bottom: 1px solid #e0e0e0;">`;
                const maxFuncs = 4;
                node.functions.slice(0, maxFuncs).forEach(func => {{
                    html += `<div class="node-function" style="font-size: 9px; color: #37474f; margin: 3px 0;">• ${{truncate(func, 24)}}</div>`;
                }});
                if (node.functions.length > maxFuncs) {{
                    html += `<div style="font-size: 9px; color: #607d8b; font-style: italic; margin-top: 3px;">+${{node.functions.length - maxFuncs}} more...</div>`;
                }}
                html += `</div>`;
            }}
            
            // Interface ports
            html += `<div style="padding: 8px; display: flex; justify-content: space-between;">`;
            
            // IN ports
            html += `<div style="flex: 1;">`;
            if (node.interfaces_in && node.interfaces_in.length > 0) {{
                const maxPorts = 3;
                node.interfaces_in.slice(0, maxPorts).forEach(port => {{
                    html += `<div style="margin: 4px 0;">`;
                    html += `<div style="display: inline-block; width: 8px; height: 8px; background: #4caf50; border: 1.5px solid #2e7d32; border-radius: 2px; margin-right: 4px;"></div>`;
                    html += `<span class="port-label" style="font-size: 8px; color: #01579b; font-weight: 700;">${{truncate(port.name, 11)}}</span>`;
                    if (port.protocol) {{
                        html += `<div class="port-protocol" style="font-size: 7px; color: #546e7a; font-style: italic; margin-left: 14px;">${{truncate(port.protocol, 13)}}</div>`;
                    }}
                    html += `</div>`;
                }});
                if (node.interfaces_in.length > maxPorts) {{
                    html += `<div style="font-size: 7px; color: #607d8b; margin-left: 14px;">+${{node.interfaces_in.length - maxPorts}} more</div>`;
                }}
            }}
            html += `</div>`;
            
            // OUT ports
            html += `<div style="flex: 1; text-align: right;">`;
            if (node.interfaces_out && node.interfaces_out.length > 0) {{
                const maxPorts = 3;
                node.interfaces_out.slice(0, maxPorts).forEach(port => {{
                    html += `<div style="margin: 4px 0;">`;
                    html += `<span class="port-label" style="font-size: 8px; color: #01579b; font-weight: 700;">${{truncate(port.name, 11)}}</span>`;
                    html += `<div style="display: inline-block; width: 8px; height: 8px; background: #ff9800; border: 1.5px solid #e65100; border-radius: 2px; margin-left: 4px;"></div>`;
                    if (port.protocol) {{
                        html += `<div class="port-protocol" style="font-size: 7px; color: #546e7a; font-style: italic; margin-right: 14px;">${{truncate(port.protocol, 13)}}</div>`;
                    }}
                    html += `</div>`;
                }});
                if (node.interfaces_out.length > maxPorts) {{
                    html += `<div style="font-size: 7px; color: #607d8b; margin-right: 14px;">+${{node.interfaces_out.length - maxPorts}} more</div>`;
                }}
            }}
            html += `</div>`;
            
            html += `</div>`;
            html += `</div>`;
            
            return html;
        }}
        
        function truncate(text, maxLen) {{
            if (!text) return '';
            return text.length <= maxLen ? text : text.substring(0, maxLen - 3) + '...';
        }}
        
        // Control functions
        function resetZoom() {{
            svg.transition().duration(750).call(
                d3.zoom().transform,
                d3.zoomIdentity.translate(xCenterOffset, yCenterOffset).scale(initialScale)
            );
        }}
        
        function fitToScreen() {{
            const bounds = svgGroup.node().getBBox();
            const parent = svg.node().getBoundingClientRect();
            const fullWidth = bounds.width;
            const fullHeight = bounds.height;
            const width = parent.width;
            const height = parent.height;
            const midX = bounds.x + fullWidth / 2;
            const midY = bounds.y + fullHeight / 2;
            
            const scale = 0.9 / Math.max(fullWidth / width, fullHeight / height);
            const translate = [width / 2 - scale * midX, height / 2 - scale * midY];
            
            svg.transition().duration(750).call(
                d3.zoom().transform,
                d3.zoomIdentity.translate(translate[0], translate[1]).scale(scale)
            );
        }}
        
        function exportSVG() {{
            try {{
                // Clone the SVG
                const svgNode = svg.node();
                const clone = svgNode.cloneNode(true);
                
                // Get the actual dimensions from the graph
                const bbox = svgGroup.node().getBBox();
                const padding = 40;
                
                // Set proper viewBox to include all content
                clone.setAttribute('viewBox', `${{bbox.x - padding}} ${{bbox.y - padding}} ${{bbox.width + padding * 2}} ${{bbox.height + padding * 2}}`);
                clone.setAttribute('width', bbox.width + padding * 2);
                clone.setAttribute('height', bbox.height + padding * 2);
                
                // Embed all styles inline
                const styleSheets = Array.from(document.styleSheets);
                let allStyles = '';
                styleSheets.forEach(sheet => {{
                    try {{
                        const rules = Array.from(sheet.cssRules || []);
                        rules.forEach(rule => {{
                            allStyles += rule.cssText + '\\n';
                        }});
                    }} catch (e) {{
                        // Skip cross-origin stylesheets
                    }}
                }});
                
                // Add style element to SVG
                const styleElement = document.createElementNS('http://www.w3.org/2000/svg', 'style');
                styleElement.textContent = allStyles;
                clone.insertBefore(styleElement, clone.firstChild);
                
                // Serialize
                const serializer = new XMLSerializer();
                const svgString = serializer.serializeToString(clone);
                
                // Add XML declaration
                const svgData = '<?xml version="1.0" encoding="UTF-8"?>\\n' + svgString;
                
                // Download
                const blob = new Blob([svgData], {{type: 'image/svg+xml;charset=utf-8'}});
                const url = URL.createObjectURL(blob);
                const link = document.createElement('a');
                link.href = url;
                link.download = 'data_platform_architecture.svg';
                link.click();
                URL.revokeObjectURL(url);
                
                console.log('SVG exported successfully');
            }} catch (error) {{
                console.error('Error exporting SVG:', error);
                alert('Error exporting SVG: ' + error.message);
            }}
        }}
        
        async function exportPDF() {{
            try {{
                // Get the SVG element
                const svgNode = svg.node();
                const bbox = svgGroup.node().getBBox();
                const padding = 40;
                
                // Create a clean SVG clone
                const clone = svgNode.cloneNode(true);
                clone.setAttribute('viewBox', `${{bbox.x - padding}} ${{bbox.y - padding}} ${{bbox.width + padding * 2}} ${{bbox.height + padding * 2}}`);
                
                // Calculate PDF dimensions (A4 landscape or larger if needed)
                const widthPx = bbox.width + padding * 2;
                const heightPx = bbox.height + padding * 2;
                
                // Convert to points (1px = 0.75pt)
                const widthPt = widthPx * 0.75;
                const heightPt = heightPx * 0.75;
                
                // Use A4 landscape or custom size
                let pdfWidth = 841.89; // A4 landscape width in pt
                let pdfHeight = 595.28; // A4 landscape height in pt
                
                // If diagram is larger than A4, use custom size
                if (widthPt > pdfWidth || heightPt > pdfHeight) {{
                    pdfWidth = widthPt;
                    pdfHeight = heightPt;
                }}
                
                // Initialize jsPDF
                const {{ jsPDF }} = window.jspdf;
                const doc = new jsPDF({{
                    orientation: pdfWidth > pdfHeight ? 'landscape' : 'portrait',
                    unit: 'pt',
                    format: [pdfWidth, pdfHeight]
                }});
                
                // Convert SVG to PDF
                await doc.svg(clone, {{
                    x: 0,
                    y: 0,
                    width: pdfWidth,
                    height: pdfHeight
                }});
                
                // Save PDF
                doc.save('data_platform_architecture.pdf');
                
                console.log('PDF exported successfully');
            }} catch (error) {{
                console.error('Error exporting PDF:', error);
                alert('Error exporting PDF: ' + error.message);
            }}
        }}
        
        function exportPNG() {{
            try {{
                // Get SVG element and dimensions
                const svgNode = svg.node();
                const bbox = svgGroup.node().getBBox();
                const padding = 40;
                
                // Create canvas
                const canvas = document.createElement('canvas');
                const scale = 2; // Higher resolution
                canvas.width = (bbox.width + padding * 2) * scale;
                canvas.height = (bbox.height + padding * 2) * scale;
                const ctx = canvas.getContext('2d');
                
                // Scale for high resolution
                ctx.scale(scale, scale);
                
                // White background
                ctx.fillStyle = 'white';
                ctx.fillRect(0, 0, canvas.width, canvas.height);
                
                // Clone SVG with proper viewBox
                const clone = svgNode.cloneNode(true);
                clone.setAttribute('viewBox', `${{bbox.x - padding}} ${{bbox.y - padding}} ${{bbox.width + padding * 2}} ${{bbox.height + padding * 2}}`);
                clone.setAttribute('width', bbox.width + padding * 2);
                clone.setAttribute('height', bbox.height + padding * 2);
                
                // Embed styles
                const styleSheets = Array.from(document.styleSheets);
                let allStyles = '';
                styleSheets.forEach(sheet => {{
                    try {{
                        const rules = Array.from(sheet.cssRules || []);
                        rules.forEach(rule => {{
                            allStyles += rule.cssText + '\\n';
                        }});
                    }} catch (e) {{}}
                }});
                
                const styleElement = document.createElementNS('http://www.w3.org/2000/svg', 'style');
                styleElement.textContent = allStyles;
                clone.insertBefore(styleElement, clone.firstChild);
                
                // Convert to data URL
                const serializer = new XMLSerializer();
                const svgString = serializer.serializeToString(clone);
                const svgBlob = new Blob([svgString], {{type: 'image/svg+xml;charset=utf-8'}});
                const url = URL.createObjectURL(svgBlob);
                
                // Load image and draw to canvas
                const img = new Image();
                img.onload = function() {{
                    ctx.drawImage(img, 0, 0);
                    URL.revokeObjectURL(url);
                    
                    // Export canvas as PNG
                    canvas.toBlob(function(blob) {{
                        const pngUrl = URL.createObjectURL(blob);
                        const link = document.createElement('a');
                        link.href = pngUrl;
                        link.download = 'data_platform_architecture.png';
                        link.click();
                        URL.revokeObjectURL(pngUrl);
                        console.log('PNG exported successfully');
                    }}, 'image/png');
                }};
                
                img.onerror = function(error) {{
                    console.error('Error loading SVG image:', error);
                    alert('Error exporting PNG. SVG may contain unsupported elements.');
                }};
                
                img.src = url;
            }} catch (error) {{
                console.error('Error exporting PNG:', error);
                alert('Error exporting PNG: ' + error.message);
            }}
        }}
        
        // Initial fit
        setTimeout(() => {{
            fitToScreen();
        }}, 100);
    </script>
</body>
</html>"#, graph_json = graph_json))
    }
}

// Helper functions

fn infer_layer(comp_id: &str) -> String {
    if comp_id.starts_with("LA-SRC") { "Source".to_string() }
    else if comp_id.starts_with("LA-MIG") { "Migration".to_string() }
    else if comp_id.starts_with("LA-TGT") { "Target".to_string() }
    else if comp_id.starts_with("LA-PROC") { "Processing".to_string() }
    else if comp_id.starts_with("LA-GOV") { "Governance".to_string() }
    else if comp_id.starts_with("LA-INT") { "Integration".to_string() }
    else if comp_id.starts_with("LA-ANLZ") { "Analytics".to_string() }
    else if comp_id.starts_with("LA-MON") { "Monitoring".to_string() }
    else { "Other".to_string() }
}

fn infer_stereotype(name: &str) -> String {
    if name.contains("Database") || name.contains("Storage") || name.contains("Warehouse") {
        "<<datastore>>".to_string()
    } else if name.contains("Orchestrator") || name.contains("Engine") || name.contains("ETL") {
        "<<process>>".to_string()
    } else if name.contains("API") || name.contains("Gateway") {
        "<<interface>>".to_string()
    } else if name.contains("Validator") || name.contains("Resolver") || name.contains("Monitor") {
        "<<service>>".to_string()
    } else if name.contains("Registry") || name.contains("Catalog") {
        "<<registry>>".to_string()
    } else {
        "<<component>>".to_string()
    }
}

fn calculate_node_width(name: &str, functions: &[String]) -> u32 {
    let name_width = name.len() as u32 * 7;
    let max_func_width = functions.iter()
        .map(|f| f.len() as u32 * 5)
        .max()
        .unwrap_or(0);
    
    let content_width = name_width.max(max_func_width);
    (content_width + 60).max(220).min(360)
}

fn calculate_node_height(interfaces_in: &[InterfacePort], interfaces_out: &[InterfacePort], functions: &[String]) -> u32 {
    let header_height = 50;
    let func_height = (functions.len().min(4) as u32) * 18 + 20;
    let max_ports = interfaces_in.len().max(interfaces_out.len()).min(3);
    let port_height = if max_ports > 0 {
        (max_ports as u32) * 30 + 20
    } else {
        10
    };
    
    (header_height + func_height + port_height).max(180).min(450)
}

pub fn generate_d3_html(model: &SemanticModel) -> Result<String, CompilerError> {
    let graph = DagreGraph::from_model(model)?;
    graph.generate_html()
}
