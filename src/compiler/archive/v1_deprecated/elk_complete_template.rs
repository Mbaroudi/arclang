pub fn generate_elk_complete_html(elk_json: &str, title: &str, use_dagre_preprocessing: bool) -> String {
    let dagre_script = if use_dagre_preprocessing {
        r#"<script src="https://unpkg.com/dagre@0.8.5/dist/dagre.min.js"></script>"#
    } else {
        ""
    };
    
    let preprocessing_badge = if use_dagre_preprocessing {
        r#"<span class="badge badge-dagre">Dagre Pre-process</span>"#
    } else {
        ""
    };
    
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <script src="https://d3js.org/d3.v7.min.js"></script>
    <script src="https://unpkg.com/elkjs@0.8.2/lib/elk.bundled.js"></script>
    {}
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{ 
            font-family: 'Segoe UI', 'Open Sans', 'Helvetica Neue', Arial, sans-serif;
            background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
            overflow: hidden;
        }}
        
        #controls {{
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            padding: 14px 24px;
            box-shadow: 0 4px 20px rgba(0,0,0,0.2);
            z-index: 1000;
            display: flex;
            align-items: center;
            gap: 12px;
        }}
        
        #controls button {{
            background: white;
            color: #667eea;
            border: none;
            padding: 9px 18px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 13px;
            font-weight: 600;
            transition: all 0.2s ease;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        
        #controls button:hover {{
            background: #f0f0f0;
            transform: translateY(-1px);
            box-shadow: 0 4px 12px rgba(0,0,0,0.15);
        }}
        
        #controls button:active {{
            transform: translateY(0);
        }}
        
        #controls label {{
            color: white;
            font-size: 13px;
            font-weight: 500;
            display: flex;
            align-items: center;
            gap: 8px;
            cursor: pointer;
            background: rgba(255,255,255,0.15);
            padding: 8px 14px;
            border-radius: 6px;
            transition: background 0.2s;
        }}
        
        #controls label:hover {{
            background: rgba(255,255,255,0.25);
        }}
        
        #controls input[type="checkbox"] {{
            cursor: pointer;
            width: 16px;
            height: 16px;
        }}
        
        .badge {{
            background: rgba(255,255,255,0.2);
            color: white;
            padding: 6px 14px;
            border-radius: 20px;
            font-size: 11px;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            border: 1px solid rgba(255,255,255,0.3);
        }}
        
        .badge-dagre {{
            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
        }}
        
        #diagram {{ 
            width: 100vw; 
            height: calc(100vh - 56px);
            margin-top: 56px;
            display: block;
            background: white;
        }}
        
        .layer {{
            fill: #f8f9fa;
            fill-opacity: 0.6;
            stroke: #495057;
            stroke-width: 2.5;
            stroke-dasharray: 8,4;
            rx: 10;
        }}
        
        .component {{
            fill: white;
            stroke: url(#component-gradient);
            stroke-width: 3;
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            filter: drop-shadow(0 3px 8px rgba(102,126,234,0.15));
        }}
        
        .component:hover {{
            fill: linear-gradient(135deg, #ffffff 0%, #f0f4ff 100%);
            stroke: url(#component-gradient-hover);
            stroke-width: 4;
            filter: drop-shadow(0 6px 20px rgba(102,126,234,0.35));
            transform: scale(1.03);
            cursor: pointer;
        }}
        
        .port {{
            fill: #4caf50;
            stroke: #2e7d32;
            stroke-width: 2;
            transition: all 0.2s;
        }}
        
        .port:hover {{
            fill: #66bb6a;
            transform: scale(1.4);
        }}
        
        .port-out {{
            fill: #ff9800;
            stroke: #e65100;
        }}
        
        .port-out:hover {{
            fill: #ffb74d;
        }}
        
        .port-north {{
            fill: #9c27b0;
            stroke: #6a1b9a;
        }}
        
        .port-south {{
            fill: #f44336;
            stroke: #c62828;
        }}
        
        .edge {{
            stroke: #546e7a;
            stroke-width: 3;
            fill: none;
            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            stroke-linecap: round;
            stroke-linejoin: round;
        }}
        
        .edge:hover {{
            stroke: #ff6b35;
            stroke-width: 4.5;
            filter: drop-shadow(0 0 8px rgba(255,107,53,0.6));
        }}
        
        .edge-animated {{
            stroke-dasharray: 8, 4;
            animation: dash 20s linear infinite;
        }}
        
        @keyframes dash {{
            to {{
                stroke-dashoffset: -1000;
            }}
        }}
        
        .label {{
            font-size: 14px;
            font-weight: 600;
            fill: #2c3e50;
            text-anchor: middle;
            dominant-baseline: middle;
            pointer-events: none;
        }}
        
        .layer-label {{
            font-size: 18px;
            font-weight: 700;
            fill: #495057;
            text-anchor: middle;
        }}
        
        .port-label {{
            font-size: 10px;
            fill: #555;
            font-weight: 500;
        }}
        
        .edge-label {{
            font-size: 12px;
            fill: #2c3e50;
            font-weight: 600;
            font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
            letter-spacing: 0.3px;
        }}
        
        .edge-label-bg {{
            fill: white;
            stroke: #667eea;
            stroke-width: 1.5px;
            rx: 6;
            filter: drop-shadow(0 2px 8px rgba(102,126,234,0.2));
            opacity: 0.98;
        }}
        
        .arrowhead {{
            fill: #546e7a;
        }}
        
        .arrowhead-hover {{
            fill: #ff6b35;
        }}
        
        #info {{
            position: fixed;
            bottom: 16px;
            right: 16px;
            background: rgba(255, 255, 255, 0.95);
            color: #2c3e50;
            padding: 16px 20px;
            border-radius: 10px;
            font-size: 12px;
            max-width: 320px;
            box-shadow: 0 8px 24px rgba(0,0,0,0.15);
            border: 1px solid rgba(0,0,0,0.1);
        }}
        
        #info strong {{
            color: #667eea;
            font-size: 14px;
            display: block;
            margin-bottom: 8px;
        }}
        
        #info .info-line {{
            margin: 4px 0;
            color: #6c757d;
        }}
        
        .loading {{
            position: fixed;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            background: white;
            padding: 30px 40px;
            border-radius: 12px;
            box-shadow: 0 10px 40px rgba(0,0,0,0.2);
            z-index: 2000;
            text-align: center;
        }}
        
        .spinner {{
            border: 4px solid #f3f3f3;
            border-top: 4px solid #667eea;
            border-radius: 50%;
            width: 40px;
            height: 40px;
            animation: spin 1s linear infinite;
            margin: 0 auto 16px;
        }}
        
        @keyframes spin {{
            0% {{ transform: rotate(0deg); }}
            100% {{ transform: rotate(360deg); }}
        }}
    </style>
</head>
<body>
    <div id="controls">
        <span class="badge">ELK Layout Engine</span>
        {}
        <button id="relayout">Re-layout</button>
        <button id="zoom-fit">Fit to View</button>
        <label>
            <input type="checkbox" id="show-ports" checked>
            Show Ports
        </label>
        <label>
            <input type="checkbox" id="show-labels" checked>
            Show Labels
        </label>
    </div>
    
    <div id="loading" class="loading">
        <div class="spinner"></div>
        <div>Computing layout...</div>
    </div>
    
    <svg id="diagram"></svg>
    
    <div id="info">
        <strong>Capella-style MBSE Diagram</strong>
        <div class="info-line">Algorithm: <span id="algo-name">ELK Layered</span></div>
        <div class="info-line">Zoom: Mouse wheel | Pan: Drag</div>
    </div>
    
    <script>
        const elkGraph = {};
        let showPorts = true;
        let showLabels = true;
        let currentTransform = d3.zoomIdentity;
        
        async function layoutAndRender() {{
            console.log('Starting layout...');
            console.time('Total Layout');
            
            let layoutedGraph;
            
            // Check if this is hybrid format (has dagreConfig)
            if (elkGraph.dagreConfig) {{
                console.log('Using Dagre + ELK hybrid layout');
                layoutedGraph = await hybridLayout(elkGraph);
            }} else {{
                console.log('Using pure ELK layout');
                const elk = new ELK();
                try {{
                    layoutedGraph = await elk.layout(elkGraph);
                }} catch (error) {{
                    console.error('ELK Layout Error:', error);
                    console.timeEnd('Total Layout');
                    document.getElementById('loading').innerHTML = 
                        '<div style="color: #e74c3c;">Layout failed. Check console for details.</div>';
                    return;
                }}
            }}
            
            console.timeEnd('Total Layout');
            console.log('Layout complete:', layoutedGraph);
            
            document.getElementById('loading').style.display = 'none';
            renderDiagram(layoutedGraph);
        }}
        
        async function hybridLayout(graphData) {{
            console.time('Dagre Layout');
            
            // Use pure Dagre for everything - positioning AND edges
            const dagreGraph = new dagre.graphlib.Graph({{ multigraph: true }});
            dagreGraph.setGraph(graphData.dagreConfig);
            dagreGraph.setDefaultEdgeLabel(() => ({{}}));
            
            // Add nodes
            graphData.nodes.forEach(node => {{
                dagreGraph.setNode(node.id, {{
                    label: node.label,
                    width: node.width,
                    height: node.height
                }});
            }});
            
            // Add edges with labels
            graphData.edges.forEach(edge => {{
                dagreGraph.setEdge(edge.source, edge.target, {{
                    label: edge.label,
                    width: edge.label ? edge.label.length * 8 + 20 : 0,
                    height: edge.label ? 30 : 0,
                    labeloffset: 15,
                    curve: d3.curveBasis
                }}, edge.id);
            }});
            
            // Run Dagre layout
            dagre.layout(dagreGraph);
            console.timeEnd('Dagre Layout');
            
            // Convert to display format
            const result = {{
                id: "root",
                children: [],
                edges: []
            }};
            
            // Add positioned nodes
            graphData.nodes.forEach(nodeData => {{
                const dagreNode = dagreGraph.node(nodeData.id);
                result.children.push({{
                    id: nodeData.id,
                    x: dagreNode.x - dagreNode.width / 2,
                    y: dagreNode.y - dagreNode.height / 2,
                    width: dagreNode.width,
                    height: dagreNode.height,
                    labels: [{{ text: nodeData.label }}]
                }});
            }});
            
            // Add edges with Dagre's routing
            graphData.edges.forEach(edgeData => {{
                const dagreEdge = dagreGraph.edge(edgeData.source, edgeData.target, edgeData.id);
                
                if (dagreEdge && dagreEdge.points) {{
                    result.edges.push({{
                        id: edgeData.id,
                        sources: [edgeData.source],
                        targets: [edgeData.target],
                        labels: edgeData.label ? [{{ text: edgeData.label }}] : [],
                        sections: [{{
                            startPoint: dagreEdge.points[0],
                            endPoint: dagreEdge.points[dagreEdge.points.length - 1],
                            bendPoints: dagreEdge.points.slice(1, -1)
                        }}]
                    }});
                }}
            }});
            
            return result;
        }}
        
        function renderDiagram(graph) {{
            const svg = d3.select('#diagram');
            svg.selectAll('*').remove();
            
            const width = window.innerWidth;
            const height = window.innerHeight - 56;
            svg.attr('width', width).attr('height', height);
            
            const defs = svg.append('defs');
            
            // Enhanced arrow marker (normal)
            defs.append('marker')
                .attr('id', 'arrowhead')
                .attr('markerWidth', 12)
                .attr('markerHeight', 12)
                .attr('refX', 11)
                .attr('refY', 6)
                .attr('orient', 'auto')
                .attr('markerUnits', 'userSpaceOnUse')
                .append('path')
                .attr('d', 'M 0 0 L 12 6 L 0 12 L 3 6 Z')
                .attr('class', 'arrowhead');
            
            // Hover arrow marker
            defs.append('marker')
                .attr('id', 'arrowhead-hover')
                .attr('markerWidth', 14)
                .attr('markerHeight', 14)
                .attr('refX', 12)
                .attr('refY', 7)
                .attr('orient', 'auto')
                .attr('markerUnits', 'userSpaceOnUse')
                .append('path')
                .attr('d', 'M 0 0 L 14 7 L 0 14 L 3 7 Z')
                .attr('class', 'arrowhead-hover');
            
            // Component gradient (normal)
            const compGradient = defs.append('linearGradient')
                .attr('id', 'component-gradient')
                .attr('x1', '0%')
                .attr('y1', '0%')
                .attr('x2', '100%')
                .attr('y2', '100%');
            
            compGradient.append('stop')
                .attr('offset', '0%')
                .attr('stop-color', '#667eea');
            
            compGradient.append('stop')
                .attr('offset', '100%')
                .attr('stop-color', '#764ba2');
            
            // Component gradient (hover)
            const compGradientHover = defs.append('linearGradient')
                .attr('id', 'component-gradient-hover')
                .attr('x1', '0%')
                .attr('y1', '0%')
                .attr('x2', '100%')
                .attr('y2', '100%');
            
            compGradientHover.append('stop')
                .attr('offset', '0%')
                .attr('stop-color', '#f093fb');
            
            compGradientHover.append('stop')
                .attr('offset', '100%')
                .attr('stop-color', '#f5576c');
            
            const g = svg.append('g').attr('class', 'main-group');
            
            renderNode(g, graph, 0, 0, true);
            renderEdges(g, graph);
            
            const zoom = d3.zoom()
                .scaleExtent([0.1, 4])
                .on('zoom', (event) => {{
                    currentTransform = event.transform;
                    g.attr('transform', event.transform);
                }});
            
            svg.call(zoom);
            
            autoFit(svg, g, zoom);
        }}
        
        function renderNode(parent, node, offsetX, offsetY, isRoot) {{
            if (!node) return;
            
            const x = (node.x || 0) + offsetX;
            const y = (node.y || 0) + offsetY;
            
            const nodeGroup = parent.append('g')
                .attr('transform', `translate(${{x}},${{y}})`)
                .attr('data-node-id', node.id);
            
            nodeGroup.append('rect')
                .attr('class', isRoot ? 'layer' : (node.children ? 'layer' : 'component'))
                .attr('width', node.width)
                .attr('height', node.height)
                .attr('rx', isRoot ? 10 : 8);
            
            if (showLabels && node.labels && node.labels.length > 0) {{
                nodeGroup.append('text')
                    .attr('class', isRoot ? 'layer-label' : 'label')
                    .attr('x', node.width / 2)
                    .attr('y', isRoot ? 35 : node.height / 2)
                    .text(node.labels[0].text);
            }}
            
            if (showPorts && node.ports) {{
                node.ports.forEach(port => {{
                    renderPort(nodeGroup, port);
                }});
            }}
            
            if (node.children) {{
                node.children.forEach(child => {{
                    renderNode(nodeGroup, child, 0, 0, false);
                }});
            }}
        }}
        
        function renderPort(parent, port) {{
            const portGroup = parent.append('g')
                .attr('transform', `translate(${{port.x || 0}},${{port.y || 0}})`)
                .attr('data-port-id', port.id);
            
            const side = port.properties?.['port.side'] || 'WEST';
            let portClass = 'port';
            
            if (side === 'EAST') portClass = 'port port-out';
            else if (side === 'NORTH') portClass = 'port port-north';
            else if (side === 'SOUTH') portClass = 'port port-south';
            
            portGroup.append('rect')
                .attr('class', portClass)
                .attr('x', -5)
                .attr('y', -5)
                .attr('width', 10)
                .attr('height', 10)
                .attr('rx', 2);
            
            if (showLabels && port.labels && port.labels.length > 0) {{
                const isEast = side === 'EAST';
                const isNorth = side === 'NORTH';
                const isSouth = side === 'SOUTH';
                
                portGroup.append('text')
                    .attr('class', 'port-label')
                    .attr('x', isEast ? 15 : (isNorth || isSouth ? 0 : -15))
                    .attr('y', isNorth ? -10 : (isSouth ? 20 : 4))
                    .attr('text-anchor', isEast ? 'start' : (isNorth || isSouth ? 'middle' : 'end'))
                    .text(port.labels[0].text);
            }}
        }}
        
        function renderEdges(parent, graph) {{
            if (!graph.edges) return;
            
            const edgeGroup = parent.append('g').attr('class', 'edges');
            
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
                    
                    const edgeElem = edgeGroup.append('g')
                        .attr('class', 'edge')
                        .attr('data-edge-id', edge.id);
                    
                    // Main edge path
                    const edgePath = edgeElem.append('path')
                        .attr('d', pathData)
                        .attr('marker-end', 'url(#arrowhead)')
                        .style('stroke-width', '3px');
                    
                    // Add label if present
                    if (showLabels && edge.labels && edge.labels.length > 0) {{
                        const label = edge.labels[0].text;
                        
                        // Calculate label position - prefer middle of path with offset
                        let labelX, labelY;
                        let offsetY = -15; // Offset label above the edge
                        
                        if (section.bendPoints && section.bendPoints.length > 0) {{
                            const midIdx = Math.floor(section.bendPoints.length / 2);
                            const midPoint = section.bendPoints[midIdx];
                            labelX = midPoint.x;
                            labelY = midPoint.y + offsetY;
                            
                            // Check if this is a vertical segment and adjust
                            if (midIdx > 0 && midIdx < section.bendPoints.length - 1) {{
                                const prevPoint = section.bendPoints[midIdx - 1];
                                const nextPoint = section.bendPoints[midIdx + 1];
                                const isVertical = Math.abs(prevPoint.x - nextPoint.x) < 10;
                                if (isVertical) {{
                                    offsetY = 0;
                                    const offsetX = 40; // Offset to the right for vertical edges
                                    labelX = midPoint.x + offsetX;
                                    labelY = midPoint.y;
                                }}
                            }}
                        }} else {{
                            labelX = (section.startPoint.x + section.endPoint.x) / 2;
                            labelY = (section.startPoint.y + section.endPoint.y) / 2 + offsetY;
                        }}
                        
                        // Create text element first to measure
                        const textElem = edgeElem.append('text')
                            .attr('class', 'edge-label')
                            .attr('x', labelX)
                            .attr('y', labelY)
                            .attr('text-anchor', 'middle')
                            .attr('dominant-baseline', 'middle')
                            .text(label);
                        
                        // Get bounding box for background
                        const bbox = textElem.node().getBBox();
                        const padding = 10;
                        
                        // Insert background rectangle before text
                        edgeElem.insert('rect', 'text')
                            .attr('class', 'edge-label-bg')
                            .attr('x', bbox.x - padding)
                            .attr('y', bbox.y - padding / 2)
                            .attr('width', bbox.width + padding * 2)
                            .attr('height', bbox.height + padding)
                            .attr('rx', 8);
                    }}
                    
                    // Enhanced hover effects
                    edgeElem.on('mouseenter', function() {{
                        d3.select(this).select('path')
                            .attr('marker-end', 'url(#arrowhead-hover)')
                            .style('stroke-width', '4.5px');
                        
                        d3.select(this).select('.edge-label-bg')
                            .style('fill', '#fff5f0')
                            .style('stroke', '#ff6b35')
                            .style('stroke-width', '2px');
                            
                        d3.select(this).select('.edge-label')
                            .style('fill', '#ff6b35')
                            .style('font-weight', '700');
                    }}).on('mouseleave', function() {{
                        d3.select(this).select('path')
                            .attr('marker-end', 'url(#arrowhead)')
                            .style('stroke-width', '3px');
                        
                        d3.select(this).select('.edge-label-bg')
                            .style('fill', 'white')
                            .style('stroke', '#667eea')
                            .style('stroke-width', '1.5px');
                            
                        d3.select(this).select('.edge-label')
                            .style('fill', '#2c3e50')
                            .style('font-weight', '600');
                    }});
                }});
            }});
        }}
        
        function autoFit(svg, g, zoom) {{
            try {{
                const bounds = g.node().getBBox();
                const width = parseFloat(svg.attr('width'));
                const height = parseFloat(svg.attr('height'));
                
                const scale = 0.85 * Math.min(
                    width / bounds.width,
                    height / bounds.height
                );
                
                const translate = [
                    (width - bounds.width * scale) / 2 - bounds.x * scale,
                    (height - bounds.height * scale) / 2 - bounds.y * scale
                ];
                
                svg.transition()
                    .duration(750)
                    .call(zoom.transform, d3.zoomIdentity.translate(translate[0], translate[1]).scale(scale));
            }} catch (error) {{
                console.warn('Auto-fit failed:', error);
            }}
        }}
        
        document.getElementById('relayout').addEventListener('click', () => {{
            document.getElementById('loading').style.display = 'block';
            layoutAndRender();
        }});
        
        document.getElementById('zoom-fit').addEventListener('click', () => {{
            const svg = d3.select('#diagram');
            const g = svg.select('.main-group');
            const zoom = d3.zoom().scaleExtent([0.1, 4]);
            svg.call(zoom);
            autoFit(svg, g, zoom);
        }});
        
        document.getElementById('show-ports').addEventListener('change', (e) => {{
            showPorts = e.target.checked;
            layoutAndRender();
        }});
        
        document.getElementById('show-labels').addEventListener('change', (e) => {{
            showLabels = e.target.checked;
            layoutAndRender();
        }});
        
        layoutAndRender();
    </script>
</body>
</html>"#, title, dagre_script, preprocessing_badge, elk_json)
}
