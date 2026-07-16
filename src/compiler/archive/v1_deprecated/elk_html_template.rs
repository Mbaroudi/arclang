pub fn generate_elk_html(elk_json: &str, title: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <script src="https://d3js.org/d3.v7.min.js"></script>
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
            align-items: center;
            gap: 15px;
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
        
        #controls label {{
            color: #ecf0f1;
            font-size: 14px;
            display: flex;
            align-items: center;
            gap: 6px;
            cursor: pointer;
        }}
        
        #controls input[type="checkbox"] {{
            cursor: pointer;
        }}
        
        #diagram {{ 
            width: 100vw; 
            height: calc(100vh - 50px);
            margin-top: 50px;
            display: block;
        }}
        
        .layer {{
            fill: #E8F5E9;
            fill-opacity: 0.3;
            stroke: #37474f;
            stroke-width: 3;
            stroke-dasharray: 10,5;
            rx: 12;
        }}
        
        .component {{
            fill: white;
            stroke: #2196f3;
            stroke-width: 2;
            transition: all 0.2s;
        }}
        
        .component:hover {{
            fill: #e3f2fd;
            stroke: #1565c0;
            stroke-width: 3;
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
            transition: all 0.2s;
        }}
        
        .port:hover {{
            fill: #66bb6a;
            transform: scale(1.3);
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
            stroke: #607d8b;
            stroke-width: 2;
            fill: none;
            transition: all 0.2s;
        }}
        
        .edge:hover {{
            stroke: #ff5722;
            stroke-width: 3;
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
            font-weight: bold;
            fill: #37474f;
            text-anchor: middle;
        }}
        
        .port-label {{
            font-size: 10px;
            fill: #555;
            font-weight: 500;
        }}
        
        .algorithm-label {{
            font-size: 10px;
            fill: #666;
            text-anchor: middle;
            font-style: italic;
        }}
        
        .interactive-node {{
            cursor: move;
        }}
        
        .interactive-node.dragging {{
            opacity: 0.7;
        }}
        
        .constraint-indicator {{
            fill: #ff5722;
            stroke: #d32f2f;
            stroke-width: 2;
            animation: pulse 2s infinite;
        }}
        
        @keyframes pulse {{
            0%, 100% {{ opacity: 1; }}
            50% {{ opacity: 0.5; }}
        }}
        
        .algorithm-rectpacking {{ fill: #e3f2fd; }}
        .algorithm-layered {{ fill: #fff3e0; }}
        .algorithm-stress {{ fill: #f3e5f5; }}
        .algorithm-force {{ fill: #e8f5e9; }}
        
        #info {{
            position: fixed;
            bottom: 10px;
            right: 10px;
            background: rgba(44, 62, 80, 0.9);
            color: white;
            padding: 10px 15px;
            border-radius: 4px;
            font-size: 12px;
            max-width: 300px;
        }}
    </style>
</head>
<body>
    <div id="controls">
        <button id="relayout">Re-layout</button>
        <button id="zoom-fit">Fit to View</button>
        <label>
            <input type="checkbox" id="interactive-mode" checked>
            Interactive Mode
        </label>
        <label>
            <input type="checkbox" id="show-ports" checked>
            Show Ports
        </label>
        <label>
            <input type="checkbox" id="show-labels" checked>
            Show Labels
        </label>
    </div>
    
    <svg id="diagram"></svg>
    
    <div id="info">
        <strong>ArcLang Diagram</strong><br>
        Zoom: Mouse wheel | Pan: Drag | Move nodes: Drag in interactive mode
    </div>
    
    <script>
        const elkGraph = {};
        let interactiveMode = true;
        let showPorts = true;
        let showLabels = true;
        let currentTransform = d3.zoomIdentity;
        
        async function layoutAndRender() {{
            const elk = new ELK();
            
            console.log('ELK Graph Configuration:', elkGraph);
            console.log('Root Algorithm:', elkGraph.layoutOptions?.['elk.algorithm']);
            console.log('Interactive:', elkGraph.layoutOptions?.interactiveLayout);
            
            console.time('ELK Layout');
            try {{
                const layoutedGraph = await elk.layout(elkGraph);
                console.timeEnd('ELK Layout');
                console.log('Layouted Graph:', layoutedGraph);
                renderDiagram(layoutedGraph);
            }} catch (error) {{
                console.error('ELK Layout Error:', error);
                console.timeEnd('ELK Layout');
            }}
        }}
        
        function renderDiagram(graph) {{
            const svg = d3.select('#diagram');
            svg.selectAll('*').remove();
            
            const width = window.innerWidth;
            const height = window.innerHeight - 50;
            svg.attr('width', width).attr('height', height);
            
            const defs = svg.append('defs');
            
            defs.append('marker')
                .attr('id', 'arrowhead')
                .attr('markerWidth', 10)
                .attr('markerHeight', 10)
                .attr('refX', 9)
                .attr('refY', 3)
                .attr('orient', 'auto')
                .append('polygon')
                .attr('points', '0 0, 10 3, 0 6')
                .attr('fill', '#607d8b');
            
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
                .attr('data-node-id', node.id);
            
            if (node.layoutOptions?.interactiveLayout) {{
                nodeGroup.attr('class', 'interactive-node');
            }}
            
            const algorithm = node.layoutOptions?.['elk.algorithm'] || 'layered';
            const algorithmClass = `algorithm-${{algorithm}}`;
            
            nodeGroup.append('rect')
                .attr('class', isRoot ? 'layer' : (node.children ? `nested-component ${{algorithmClass}}` : `component ${{algorithmClass}}`))
                .attr('width', node.width)
                .attr('height', node.height)
                .attr('rx', isRoot ? 12 : 6);
            
            if (node.layoutOptions?.['layering.layerChoiceConstraint'] !== undefined ||
                node.layoutOptions?.['crossingMinimization.positionChoiceConstraint'] !== undefined) {{
                nodeGroup.append('circle')
                    .attr('class', 'constraint-indicator')
                    .attr('cx', 10)
                    .attr('cy', 10)
                    .attr('r', 5);
            }}
            
            if (showLabels && node.labels && node.labels.length > 0) {{
                nodeGroup.append('text')
                    .attr('class', isRoot ? 'layer-label' : 'label')
                    .attr('x', node.width / 2)
                    .attr('y', isRoot ? 30 : node.height / 2)
                    .text(node.labels[0].text);
                
                if (!isRoot) {{
                    nodeGroup.append('text')
                        .attr('class', 'algorithm-label')
                        .attr('x', node.width / 2)
                        .attr('y', node.height - 10)
                        .text(`[${{algorithm}}]`);
                }}
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
                    
                    edgeGroup.append('path')
                        .attr('class', 'edge')
                        .attr('d', pathData)
                        .attr('marker-end', 'url(#arrowhead)');
                }});
                
                if (showLabels && edge.labels && edge.labels.length > 0) {{
                    const section = edge.sections[0];
                    const midX = (section.startPoint.x + section.endPoint.x) / 2;
                    const midY = (section.startPoint.y + section.endPoint.y) / 2;
                    
                    edgeGroup.append('text')
                        .attr('class', 'label')
                        .attr('x', midX)
                        .attr('y', midY - 5)
                        .attr('font-size', '12px')
                        .attr('fill', '#555')
                        .text(edge.labels[0].text);
                }}
            }});
        }}
        
        function enableInteractiveDrag(g) {{
            const nodes = g.selectAll('.interactive-node');
            
            const drag = d3.drag()
                .on('start', function(event) {{
                    d3.select(this).classed('dragging', true)
                        .raise();
                }})
                .on('drag', function(event) {{
                    const transform = d3.select(this).attr('transform');
                    const match = transform.match(/translate\(([^,]+),([^)]+)\)/);
                    if (match) {{
                        const x = parseFloat(match[1]) + event.dx / currentTransform.k;
                        const y = parseFloat(match[2]) + event.dy / currentTransform.k;
                        d3.select(this).attr('transform', `translate(${{x}},${{y}})`);
                    }}
                }})
                .on('end', function(event) {{
                    d3.select(this).classed('dragging', false);
                }});
            
            nodes.call(drag);
        }}
        
        function autoFit(svg, g, zoom) {{
            try {{
                const bounds = g.node().getBBox();
                const width = parseFloat(svg.attr('width'));
                const height = parseFloat(svg.attr('height'));
                
                const scale = 0.9 * Math.min(
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
            layoutAndRender();
        }});
        
        document.getElementById('zoom-fit').addEventListener('click', () => {{
            const svg = d3.select('#diagram');
            const g = svg.select('.main-group');
            const zoom = d3.zoom().scaleExtent([0.1, 4]);
            svg.call(zoom);
            autoFit(svg, g, zoom);
        }});
        
        document.getElementById('interactive-mode').addEventListener('change', (e) => {{
            interactiveMode = e.target.checked;
            layoutAndRender();
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
</html>"#, title, elk_json)
}
