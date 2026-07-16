pub fn generate_elk_dagre_hybrid_html(graph_json: &str, title: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <script src="https://d3js.org/d3.v7.min.js"></script>
    <script src="https://unpkg.com/dagre@0.8.5/dist/dagre.min.js"></script>
    <script src="https://unpkg.com/elkjs@0.8.2/lib/elk.bundled.js"></script>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: 'Segoe UI', 'Open Sans', Arial, sans-serif;
            background: #f5f7fa;
            overflow: hidden;
        }}
        #controls {{
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            background: #2c3e50;
            padding: 12px 20px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.15);
            z-index: 1000;
            display: flex;
            gap: 15px;
            align-items: center;
        }}
        #controls button {{
            background: #3498db;
            color: white;
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            transition: background 0.2s;
        }}
        #controls button:hover {{
            background: #2980b9;
        }}
        #controls .badge {{
            background: #27ae60;
            color: white;
            padding: 4px 12px;
            border-radius: 12px;
            font-size: 12px;
            font-weight: 600;
        }}
        svg {{
            width: 100vw;
            height: calc(100vh - 50px);
            margin-top: 50px;
        }}
        .node rect {{
            fill: white;
            stroke: #2196f3;
            stroke-width: 2.5px;
            rx: 8px;
            transition: all 0.2s;
        }}
        .node:hover rect {{
            fill: #e3f2fd;
            stroke: #1565c0;
            stroke-width: 3px;
        }}
        .node text {{
            font-size: 14px;
            fill: #2c3e50;
            text-anchor: middle;
            font-weight: 600;
            pointer-events: none;
        }}
        .edge path {{
            stroke: #607d8b;
            stroke-width: 2.5px;
            fill: none;
            transition: all 0.2s;
        }}
        .edge:hover path {{
            stroke: #ff5722;
            stroke-width: 3.5px;
        }}
        .edge-label {{
            font-size: 11px;
            fill: #555;
            font-weight: 500;
            background: white;
            padding: 2px 6px;
            border-radius: 3px;
        }}
        .edge-label-bg {{
            fill: white;
            stroke: #ddd;
            stroke-width: 1px;
            rx: 3px;
        }}
        .arrowhead {{
            fill: #607d8b;
        }}
        .arrowhead-hover {{
            fill: #ff5722;
        }}
        #info {{
            position: fixed;
            bottom: 10px;
            right: 10px;
            background: rgba(44, 62, 80, 0.95);
            color: white;
            padding: 12px 16px;
            border-radius: 6px;
            font-size: 12px;
            max-width: 350px;
            box-shadow: 0 4px 12px rgba(0,0,0,0.3);
        }}
        #info strong {{
            color: #3498db;
        }}
    </style>
</head>
<body>
    <div id="controls">
        <span class="badge">Hybrid: Dagre + ELK</span>
        <button id="zoom-fit">Fit to View</button>
        <button id="zoom-in">Zoom In (+)</button>
        <button id="zoom-out">Zoom Out (-)</button>
        <button id="relayout">Re-layout</button>
    </div>
    <svg id="diagram"></svg>
    <div id="info">
        <strong>Hybrid Layout</strong><br>
        Node Placement: Dagre (optimal positioning)<br>
        Edge Routing: ELK (orthogonal routing)<br>
        Zoom: Mouse wheel | Pan: Drag
    </div>
    <script>
        const graphData = {};
        let currentTransform = d3.zoomIdentity;
        
        async function layoutAndRender() {{
            console.log('Starting hybrid layout...');
            console.time('Dagre Layout');
            console.time('ELK Routing');
            
            // Step 1: Use Dagre for node positioning
            const dagreGraph = new dagre.graphlib.Graph();
            dagreGraph.setGraph(graphData.dagreConfig);
            dagreGraph.setDefaultEdgeLabel(() => ({{}}));
            
            // Add nodes to Dagre
            graphData.nodes.forEach(node => {{
                dagreGraph.setNode(node.id, {{
                    label: node.label,
                    width: node.width,
                    height: node.height
                }});
            }});
            
            // Add edges to Dagre
            graphData.edges.forEach(edge => {{
                dagreGraph.setEdge(edge.source, edge.target);
            }});
            
            // Run Dagre layout
            dagre.layout(dagreGraph);
            console.timeEnd('Dagre Layout');
            
            // Step 2: Convert Dagre positions to ELK format with fixed positions
            const elkGraph = {{
                id: "root",
                layoutOptions: {{
                    "elk.algorithm": "layered",
                    "elk.direction": "RIGHT",
                    "elk.edgeRouting": "ORTHOGONAL",
                    "elk.spacing.nodeNode": String(graphData.elkConfig.spacing.nodeNode),
                    "elk.spacing.edgeNode": String(graphData.elkConfig.spacing.edgeNode),
                    "elk.spacing.edgeEdge": String(graphData.elkConfig.spacing.edgeEdge),
                    "elk.layered.spacing.nodeNodeBetweenLayers": String(graphData.elkConfig.spacing.nodeNodeBetweenLayers),
                    "elk.layered.nodePlacement.strategy": "NETWORK_SIMPLEX",
                    "elk.layered.crossingMinimization.strategy": "LAYER_SWEEP",
                    "elk.portConstraints": "FREE",
                    "elk.separateConnectedComponents": "false",
                }},
                children: [],
                edges: []
            }};
            
            // Add nodes with Dagre positions
            graphData.nodes.forEach(nodeData => {{
                const dagreNode = dagreGraph.node(nodeData.id);
                elkGraph.children.push({{
                    id: nodeData.id,
                    x: dagreNode.x - dagreNode.width / 2,
                    y: dagreNode.y - dagreNode.height / 2,
                    width: dagreNode.width,
                    height: dagreNode.height,
                    labels: [{{ text: nodeData.label }}],
                    layoutOptions: {{
                        "elk.portConstraints": "FREE"
                    }}
                }});
            }});
            
            // Add edges for ELK routing
            graphData.edges.forEach(edge => {{
                elkGraph.edges.push({{
                    id: edge.id,
                    sources: [edge.source],
                    targets: [edge.target],
                    labels: edge.label ? [{{ text: edge.label }}] : []
                }});
            }});
            
            // Step 3: Use ELK for edge routing only
            const elk = new ELK();
            const layoutedGraph = await elk.layout(elkGraph);
            console.timeEnd('ELK Routing');
            
            // Render the final result
            renderDiagram(layoutedGraph);
        }}
        
        function renderDiagram(graph) {{
            const svg = d3.select('#diagram');
            svg.selectAll('*').remove();
            
            const width = window.innerWidth;
            const height = window.innerHeight - 50;
            svg.attr('width', width).attr('height', height);
            
            const defs = svg.append('defs');
            
            // Arrow marker (normal)
            defs.append('marker')
                .attr('id', 'arrowhead')
                .attr('viewBox', '0 0 10 10')
                .attr('refX', 9)
                .attr('refY', 5)
                .attr('markerWidth', 10)
                .attr('markerHeight', 10)
                .attr('orient', 'auto')
                .append('path')
                .attr('d', 'M 0 0 L 10 5 L 0 10 z')
                .attr('class', 'arrowhead');
            
            // Arrow marker (hover)
            defs.append('marker')
                .attr('id', 'arrowhead-hover')
                .attr('viewBox', '0 0 10 10')
                .attr('refX', 9)
                .attr('refY', 5)
                .attr('markerWidth', 10)
                .attr('markerHeight', 10)
                .attr('orient', 'auto')
                .append('path')
                .attr('d', 'M 0 0 L 10 5 L 0 10 z')
                .attr('class', 'arrowhead-hover');
            
            const container = svg.append('g').attr('class', 'main-group');
            
            // Draw edges first (so they appear behind nodes)
            const edgeGroup = container.append('g').attr('class', 'edges');
            
            if (graph.edges) {{
                graph.edges.forEach(edge => {{
                    if (!edge.sections || edge.sections.length === 0) return;
                    
                    edge.sections.forEach(section => {{
                        // Build path
                        let pathData = `M ${{section.startPoint.x}} ${{section.startPoint.y}}`;
                        
                        if (section.bendPoints && section.bendPoints.length > 0) {{
                            section.bendPoints.forEach(bp => {{
                                pathData += ` L ${{bp.x}} ${{bp.y}}`;
                            }});
                        }}
                        
                        pathData += ` L ${{section.endPoint.x}} ${{section.endPoint.y}}`;
                        
                        const edgeElem = edgeGroup.append('g')
                            .attr('class', 'edge');
                        
                        edgeElem.append('path')
                            .attr('d', pathData)
                            .attr('marker-end', 'url(#arrowhead)');
                        
                        // Add label with background
                        if (edge.labels && edge.labels.length > 0) {{
                            const label = edge.labels[0].text;
                            const midIdx = Math.floor((section.bendPoints?.length || 0) / 2);
                            let labelX, labelY;
                            
                            if (section.bendPoints && section.bendPoints.length > 0) {{
                                const midPoint = section.bendPoints[midIdx] || section.bendPoints[0];
                                labelX = midPoint.x;
                                labelY = midPoint.y;
                            }} else {{
                                labelX = (section.startPoint.x + section.endPoint.x) / 2;
                                labelY = (section.startPoint.y + section.endPoint.y) / 2;
                            }}
                            
                            // Measure text
                            const textElem = edgeElem.append('text')
                                .attr('class', 'edge-label')
                                .attr('x', labelX)
                                .attr('y', labelY - 2)
                                .attr('text-anchor', 'middle')
                                .text(label);
                            
                            const bbox = textElem.node().getBBox();
                            
                            // Add background rectangle
                            edgeElem.insert('rect', 'text')
                                .attr('class', 'edge-label-bg')
                                .attr('x', bbox.x - 4)
                                .attr('y', bbox.y - 2)
                                .attr('width', bbox.width + 8)
                                .attr('height', bbox.height + 4);
                        }}
                        
                        // Hover effects
                        edgeElem.on('mouseenter', function() {{
                            d3.select(this).select('path')
                                .attr('marker-end', 'url(#arrowhead-hover)');
                        }}).on('mouseleave', function() {{
                            d3.select(this).select('path')
                                .attr('marker-end', 'url(#arrowhead)');
                        }});
                    }});
                }});
            }}
            
            // Draw nodes
            const nodeGroup = container.append('g').attr('class', 'nodes');
            
            graph.children.forEach(node => {{
                const nodeElem = nodeGroup.append('g')
                    .attr('class', 'node')
                    .attr('transform', `translate(${{node.x}},${{node.y}})`);
                
                nodeElem.append('rect')
                    .attr('width', node.width)
                    .attr('height', node.height);
                
                nodeElem.append('text')
                    .attr('x', node.width / 2)
                    .attr('y', node.height / 2 + 5)
                    .text(node.labels[0].text);
            }});
            
            // Zoom and pan
            const zoom = d3.zoom()
                .scaleExtent([0.1, 4])
                .on('zoom', (event) => {{
                    currentTransform = event.transform;
                    container.attr('transform', event.transform);
                }});
            
            svg.call(zoom);
            
            // Auto-fit to view
            fitToView(svg, container, zoom);
        }}
        
        function fitToView(svg, container, zoom) {{
            try {{
                const bbox = container.node().getBBox();
                const width = parseFloat(svg.attr('width'));
                const height = parseFloat(svg.attr('height'));
                
                const scale = 0.85 * Math.min(
                    width / bbox.width,
                    height / bbox.height
                );
                
                const tx = (width - bbox.width * scale) / 2 - bbox.x * scale;
                const ty = (height - bbox.height * scale) / 2 - bbox.y * scale;
                
                svg.transition()
                    .duration(750)
                    .call(zoom.transform, d3.zoomIdentity.translate(tx, ty).scale(scale));
            }} catch (error) {{
                console.warn('Auto-fit failed:', error);
            }}
        }}
        
        // Controls
        document.getElementById('zoom-fit').addEventListener('click', () => {{
            const svg = d3.select('#diagram');
            const container = svg.select('.main-group');
            const zoom = d3.zoom().scaleExtent([0.1, 4]);
            svg.call(zoom);
            fitToView(svg, container, zoom);
        }});
        
        document.getElementById('zoom-in').addEventListener('click', () => {{
            d3.select('#diagram').transition().call(
                d3.zoom().scaleBy, 1.3
            );
        }});
        
        document.getElementById('zoom-out').addEventListener('click', () => {{
            d3.select('#diagram').transition().call(
                d3.zoom().scaleBy, 0.77
            );
        }});
        
        document.getElementById('relayout').addEventListener('click', () => {{
            layoutAndRender();
        }});
        
        // Initial layout
        layoutAndRender();
    </script>
</body>
</html>"#, title, graph_json)
}
