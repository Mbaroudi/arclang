pub fn generate_dagre_html(graph_json: &str, title: &str) -> String {
    format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <script src="https://d3js.org/d3.v7.min.js"></script>
    <script src="https://unpkg.com/dagre@0.8.5/dist/dagre.min.js"></script>
    <style>
        body {{
            margin: 0;
            font-family: 'Segoe UI', Arial, sans-serif;
            background: #f5f7fa;
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
        }}
        #controls button:hover {{
            background: #2980b9;
        }}
        svg {{
            width: 100vw;
            height: calc(100vh - 50px);
            margin-top: 50px;
        }}
        .node rect {{
            fill: white;
            stroke: #2196f3;
            stroke-width: 2px;
            rx: 6px;
        }}
        .node text {{
            font-size: 14px;
            fill: #2c3e50;
            text-anchor: middle;
            font-weight: 600;
        }}
        .edge path {{
            stroke: #607d8b;
            stroke-width: 2px;
            fill: none;
        }}
        .edge path:hover {{
            stroke: #ff5722;
            stroke-width: 3px;
        }}
        .edge-label {{
            font-size: 12px;
            fill: #555;
            font-weight: 500;
        }}
        .arrowhead {{
            fill: #607d8b;
        }}
    </style>
</head>
<body>
    <div id="controls">
        <button id="zoom-fit">Fit to View</button>
        <button id="zoom-in">Zoom In</button>
        <button id="zoom-out">Zoom Out</button>
    </div>
    <svg id="diagram"></svg>
    <script>
        const graphData = {};
        
        // Create a new directed graph
        const g = new dagre.graphlib.Graph();
        g.setGraph({{
            rankdir: 'LR',
            nodesep: 100,
            edgesep: 50,
            ranksep: 150,
            marginx: 50,
            marginy: 50
        }});
        g.setDefaultEdgeLabel(() => ({{}}));
        
        // Add nodes
        graphData.nodes.forEach(node => {{
            g.setNode(node.id, {{
                label: node.label,
                width: 200,
                height: 150
            }});
        }});
        
        // Add edges
        graphData.edges.forEach(edge => {{
            g.setEdge(edge.from, edge.to, {{
                label: edge.label || ''
            }});
        }});
        
        // Run dagre layout
        dagre.layout(g);
        
        // Render the graph
        const svg = d3.select('#diagram');
        const container = svg.append('g');
        
        // Create arrow marker
        svg.append('defs').append('marker')
            .attr('id', 'arrowhead')
            .attr('viewBox', '0 0 10 10')
            .attr('refX', 9)
            .attr('refY', 5)
            .attr('markerWidth', 8)
            .attr('markerHeight', 8)
            .attr('orient', 'auto')
            .append('path')
            .attr('d', 'M 0 0 L 10 5 L 0 10 z')
            .attr('class', 'arrowhead');
        
        // Draw edges
        g.edges().forEach(e => {{
            const edge = g.edge(e);
            const points = edge.points;
            
            const line = d3.line()
                .x(d => d.x)
                .y(d => d.y)
                .curve(d3.curveLinear);
            
            container.append('path')
                .attr('class', 'edge')
                .attr('d', line(points))
                .attr('marker-end', 'url(#arrowhead)');
            
            if (edge.label) {{
                const midpoint = points[Math.floor(points.length / 2)];
                container.append('text')
                    .attr('class', 'edge-label')
                    .attr('x', midpoint.x)
                    .attr('y', midpoint.y - 5)
                    .attr('text-anchor', 'middle')
                    .text(edge.label);
            }}
        }});
        
        // Draw nodes
        g.nodes().forEach(v => {{
            const node = g.node(v);
            const nodeGroup = container.append('g')
                .attr('class', 'node')
                .attr('transform', `translate(${{node.x}},${{node.y}})`);
            
            nodeGroup.append('rect')
                .attr('x', -node.width / 2)
                .attr('y', -node.height / 2)
                .attr('width', node.width)
                .attr('height', node.height);
            
            nodeGroup.append('text')
                .attr('y', 5)
                .text(node.label);
        }});
        
        // Zoom and pan
        const zoom = d3.zoom()
            .scaleExtent([0.1, 4])
            .on('zoom', (event) => {{
                container.attr('transform', event.transform);
            }});
        
        svg.call(zoom);
        
        // Fit to view
        function fitToView() {{
            const bbox = container.node().getBBox();
            const width = svg.node().clientWidth;
            const height = svg.node().clientHeight;
            
            const scale = 0.9 * Math.min(
                width / bbox.width,
                height / bbox.height
            );
            
            const tx = (width - bbox.width * scale) / 2 - bbox.x * scale;
            const ty = (height - bbox.height * scale) / 2 - bbox.y * scale;
            
            svg.transition()
                .duration(750)
                .call(zoom.transform, d3.zoomIdentity.translate(tx, ty).scale(scale));
        }}
        
        document.getElementById('zoom-fit').addEventListener('click', fitToView);
        document.getElementById('zoom-in').addEventListener('click', () => {{
            svg.transition().call(zoom.scaleBy, 1.3);
        }});
        document.getElementById('zoom-out').addEventListener('click', () => {{
            svg.transition().call(zoom.scaleBy, 0.77);
        }});
        
        // Initial fit
        setTimeout(fitToView, 100);
    </script>
</body>
</html>"#, title, graph_json)
}
