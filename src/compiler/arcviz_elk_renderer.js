// ============================================================================
// ArcViz ELK Layout Renderer
// Eclipse Layout Kernel integration for Capella-compliant diagrams
// ============================================================================

/**
 * Render diagram using ELK layout engine
 * @param {Object} diagramData - Architecture diagram data
 */
async function renderWithELK(diagramData) {
    console.log('🚀 Using ELK layout engine...');
    console.time('ELK Total');
    
    // Convert ArcLang diagram to ELK graph format
    const elkGraph = convertToELKGraph(diagramData);
    
    // Run ELK layout
    console.time('ELK Layout');
    const layoutGraph = await elk.layout(elkGraph);
    console.timeEnd('ELK Layout');
    
    // Render with D3
    console.time('D3 Render');
    renderELKGraph(layoutGraph, diagramData);
    console.timeEnd('D3 Render');
    
    console.timeEnd('ELK Total');
    console.log('✓ ELK diagram rendered:', diagramData.nodes.length, 'nodes,', (diagramData.edges || []).length, 'edges');
}

/**
 * Convert ArcLang diagram data to ELK graph format
 */
function convertToELKGraph(diagramData) {
    const elkGraph = {
        id: 'root',
        layoutOptions: ARCVIZ_CONFIG.elk,
        children: [],
        edges: []
    };
    
    // Group nodes by layer
    const nodesByLayer = {};
    const layerConfigs = {};
    
    if (diagramData.layers) {
        diagramData.layers.forEach(layer => {
            layerConfigs[layer.name] = layer;
            nodesByLayer[layer.name] = [];
        });
    }
    
    diagramData.nodes.forEach(node => {
        const layer = node.layer || 'Other';
        if (!nodesByLayer[layer]) {
            nodesByLayer[layer] = [];
        }
        nodesByLayer[layer].push(node);
    });
    
    // Create layer nodes
    Object.keys(nodesByLayer).forEach(layerName => {
        const layerConfig = layerConfigs[layerName];
        const layerColor = layerConfig ? layerConfig.color : '#E8F5E9';
        
        const layerNode = {
            id: layerName,
            layoutOptions: {
                'elk.padding': '[top=50,left=30,bottom=30,right=30]',
                'elk.portConstraints': 'FREE',
                'elk.layered.spacing.nodeNodeBetweenLayers': 100
            },
            labels: [{ 
                text: layerName + ' Layer',
                layoutOptions: {
                    'nodeLabels.placement': 'INSIDE V_TOP H_CENTER'
                }
            }],
            properties: {
                color: layerColor,
                isLayer: true
            },
            children: [],
            ports: []
        };
        
        // Add components to layer
        nodesByLayer[layerName].forEach(node => {
            const elkNode = convertNodeToELK(node);
            layerNode.children.push(elkNode);
        });
        
        elkGraph.children.push(layerNode);
    });
    
    // Add edges
    if (diagramData.edges) {
        diagramData.edges.forEach(edge => {
            if (edge.source && edge.target) {
                elkGraph.edges.push({
                    id: `edge_${edge.source}_${edge.target}`,
                    sources: [edge.source],
                    targets: [edge.target],
                    labels: edge.label ? [{ text: edge.label }] : []
                });
            }
        });
    }
    
    return elkGraph;
}

/**
 * Convert single node to ELK format
 */
function convertNodeToELK(node) {
    const width = node.width || ARCVIZ_CONFIG.node.defaultWidth;
    const height = node.height || ARCVIZ_CONFIG.node.defaultHeight;
    
    const elkNode = {
        id: node.id,
        width: width,
        height: height,
        layoutOptions: {
            'elk.portConstraints': 'FIXED_SIDE'
        },
        labels: [{ 
            text: node.label || node.id,
            layoutOptions: {
                'nodeLabels.placement': 'INSIDE V_TOP H_CENTER'
            }
        }],
        properties: {
            originalNode: node,
            safety_level: node.safety_level,
            functions: node.functions || [],
            interfaces_in: node.interfaces_in || [],
            interfaces_out: node.interfaces_out || []
        },
        ports: []
    };
    
    // Add IN ports (left side)
    if (node.interfaces_in) {
        node.interfaces_in.forEach((port, idx) => {
            elkNode.ports.push({
                id: `${node.id}_in_${idx}`,
                properties: {
                    'port.side': 'WEST',
                    'port.index': idx,
                    portType: 'in',
                    portData: port
                },
                width: ARCVIZ_CONFIG.port.size,
                height: ARCVIZ_CONFIG.port.size,
                labels: [{ text: port.name || `IN${idx}` }]
            });
        });
    }
    
    // Add OUT ports (right side)
    if (node.interfaces_out) {
        node.interfaces_out.forEach((port, idx) => {
            elkNode.ports.push({
                id: `${node.id}_out_${idx}`,
                properties: {
                    'port.side': 'EAST',
                    'port.index': idx,
                    portType: 'out',
                    portData: port
                },
                width: ARCVIZ_CONFIG.port.size,
                height: ARCVIZ_CONFIG.port.size,
                labels: [{ text: port.name || `OUT${idx}` }]
            });
        });
    }
    
    return elkNode;
}

/**
 * Render ELK graph with D3
 */
function renderELKGraph(graph, diagramData) {
    const svg = d3.select('#arch-diagram');
    svg.selectAll('*').remove();
    
    // Add arrow marker definition
    const defs = svg.append('defs');
    defs.append('marker')
        .attr('id', 'arrowhead')
        .attr('viewBox', '0 0 10 10')
        .attr('refX', 9)
        .attr('refY', 5)
        .attr('markerUnits', 'strokeWidth')
        .attr('markerWidth', 8)
        .attr('markerHeight', 6)
        .attr('orient', 'auto')
        .append('path')
        .attr('d', 'M 0 0 L 10 5 L 0 10 z')
        .attr('fill', '#607d8b');
    
    const svgGroup = svg.append('g');
    
    // Render layers
    graph.children.forEach(layer => {
        renderLayer(svgGroup, layer);
    });
    
    // Render edges
    if (graph.edges) {
        graph.edges.forEach(edge => {
            renderEdge(svgGroup, edge);
        });
    }
    
    // Setup zoom and pan
    const graphWidth = graph.width || 1000;
    const graphHeight = graph.height || 800;
    setupZoomAndPan(svg, svgGroup, graphWidth, graphHeight);
    
    // Store references
    window.diagramSvg = svg;
    window.diagramSvgGroup = svgGroup;
    window.diagramGraph = graph;
}

/**
 * Render a layer with its components
 */
function renderLayer(svgGroup, layer) {
    const layerGroup = svgGroup.append('g')
        .attr('class', 'layer')
        .attr('id', `layer_${layer.id}`)
        .attr('transform', `translate(${layer.x}, ${layer.y})`);
    
    // Layer background
    const layerColor = layer.properties && layer.properties.color ? layer.properties.color : '#E8F5E9';
    layerGroup.append('rect')
        .attr('class', 'layer-rect')
        .attr('width', layer.width)
        .attr('height', layer.height)
        .attr('rx', ARCVIZ_CONFIG.layer.cornerRadius)
        .attr('ry', ARCVIZ_CONFIG.layer.cornerRadius)
        .style('fill', layerColor)
        .style('fill-opacity', 0.25)
        .style('stroke', '#37474f')
        .style('stroke-width', ARCVIZ_CONFIG.layer.borderWidth)
        .style('stroke-dasharray', ARCVIZ_CONFIG.layer.borderStyle);
    
    // Layer label
    if (layer.labels && layer.labels.length > 0) {
        layerGroup.append('text')
            .attr('class', 'layer-label')
            .attr('x', layer.width / 2)
            .attr('y', 30)
            .attr('text-anchor', 'middle')
            .style('font-size', '18px')
            .style('font-weight', 'bold')
            .style('fill', '#37474f')
            .text(layer.labels[0].text);
    }
    
    // Render components
    if (layer.children) {
        layer.children.forEach(node => {
            renderComponent(layerGroup, node);
        });
    }
}

/**
 * Render a component node with Capella style
 */
function renderComponent(parentGroup, node) {
    const nodeGroup = parentGroup.append('g')
        .attr('class', 'component')
        .attr('id', `node_${node.id}`)
        .attr('transform', `translate(${node.x}, ${node.y})`);
    
    const props = node.properties || {};
    const originalNode = props.originalNode || {};
    
    // Component box
    nodeGroup.append('rect')
        .attr('class', 'component-rect')
        .attr('width', node.width)
        .attr('height', node.height)
        .attr('rx', ARCVIZ_CONFIG.node.borderRadius)
        .attr('ry', ARCVIZ_CONFIG.node.borderRadius)
        .style('fill', 'white')
        .style('stroke', '#1976d2')
        .style('stroke-width', 2);
    
    // Header
    const headerHeight = ARCVIZ_CONFIG.node.headerHeight;
    nodeGroup.append('rect')
        .attr('class', 'component-header')
        .attr('width', node.width)
        .attr('height', headerHeight)
        .attr('rx', ARCVIZ_CONFIG.node.borderRadius)
        .style('fill', '#1976d2')
        .style('fill-opacity', 0.1);
    
    // Title
    if (node.labels && node.labels.length > 0) {
        nodeGroup.append('text')
            .attr('class', 'component-title')
            .attr('x', node.width / 2)
            .attr('y', headerHeight / 2 + 6)
            .attr('text-anchor', 'middle')
            .style('font-size', '14px')
            .style('font-weight', '600')
            .style('fill', '#1565c0')
            .text(node.labels[0].text);
    }
    
    // ASIL badge
    if (props.safety_level) {
        const badgeColor = ARCVIZ_CONFIG.safety.colors[props.safety_level] || '#ff9800';
        const badgeGroup = nodeGroup.append('g')
            .attr('transform', `translate(${node.width - 35}, 10)`);
        
        badgeGroup.append('circle')
            .attr('r', ARCVIZ_CONFIG.safety.badgeSize / 2)
            .style('fill', badgeColor)
            .style('stroke', 'white')
            .style('stroke-width', 2);
        
        badgeGroup.append('text')
            .attr('y', 5)
            .attr('text-anchor', 'middle')
            .style('font-size', '10px')
            .style('font-weight', 'bold')
            .style('fill', 'white')
            .text(props.safety_level.replace('ASIL_', ''));
    }
    
    // Functions list
    if (props.functions && props.functions.length > 0) {
        const funcY = headerHeight + 20;
        const maxFuncs = Math.floor((node.height - headerHeight - 20) / 18);
        const funcsToShow = props.functions.slice(0, maxFuncs);
        
        funcsToShow.forEach((func, idx) => {
            nodeGroup.append('text')
                .attr('class', 'function-text')
                .attr('x', 12)
                .attr('y', funcY + idx * 18)
                .style('font-size', '10px')
                .style('font-weight', '500')
                .style('fill', '#37474f')
                .text(`+ ${func.length > 25 ? func.substring(0, 25) + '...' : func}`);
        });
        
        if (props.functions.length > maxFuncs) {
            nodeGroup.append('text')
                .attr('x', 12)
                .attr('y', funcY + maxFuncs * 18)
                .style('font-size', '9px')
                .style('font-style', 'italic')
                .style('fill', '#999')
                .text(`...${props.functions.length - maxFuncs} more`);
        }
    }
    
    // Render ports
    if (node.ports) {
        node.ports.forEach(port => {
            renderPort(nodeGroup, port, node);
        });
    }
}

/**
 * Render a port with Capella style
 */
function renderPort(nodeGroup, port, node) {
    const portProps = port.properties || {};
    const isInput = portProps.portType === 'in';
    const portData = portProps.portData || {};
    
    const portGroup = nodeGroup.append('g')
        .attr('class', `port port-${portProps.portType}`)
        .attr('transform', `translate(${port.x}, ${port.y})`);
    
    // Port square
    const portColor = isInput ? ARCVIZ_CONFIG.port.colors.inFill : ARCVIZ_CONFIG.port.colors.outFill;
    const portStroke = isInput ? ARCVIZ_CONFIG.port.colors.inStroke : ARCVIZ_CONFIG.port.colors.outStroke;
    
    portGroup.append('rect')
        .attr('x', -ARCVIZ_CONFIG.port.size / 2)
        .attr('y', -ARCVIZ_CONFIG.port.size / 2)
        .attr('width', ARCVIZ_CONFIG.port.size)
        .attr('height', ARCVIZ_CONFIG.port.size)
        .attr('rx', ARCVIZ_CONFIG.port.borderRadius)
        .attr('ry', ARCVIZ_CONFIG.port.borderRadius)
        .style('fill', portColor)
        .style('stroke', portStroke)
        .style('stroke-width', 2);
    
    // Port label (name)
    const labelX = isInput ? -ARCVIZ_CONFIG.port.size - 5 : ARCVIZ_CONFIG.port.size + 5;
    const anchor = isInput ? 'end' : 'start';
    
    if (portData.name) {
        portGroup.append('text')
            .attr('class', 'port-label')
            .attr('x', labelX)
            .attr('y', 4)
            .attr('text-anchor', anchor)
            .style('font-size', '9px')
            .style('font-weight', '600')
            .style('fill', '#263238')
            .text(portData.name.length > 15 ? portData.name.substring(0, 15) + '...' : portData.name);
    }
    
    // Protocol label
    if (portData.protocol) {
        portGroup.append('text')
            .attr('class', 'port-protocol')
            .attr('x', labelX)
            .attr('y', 16)
            .attr('text-anchor', anchor)
            .style('font-size', '7px')
            .style('font-style', 'italic')
            .style('fill', '#546e7a')
            .text(`[${portData.protocol}]`);
    }
}

/**
 * Render an edge with proper routing
 */
function renderEdge(svgGroup, edge) {
    if (!edge.sections || edge.sections.length === 0) return;
    
    const section = edge.sections[0];
    let path = `M ${section.startPoint.x} ${section.startPoint.y} `;
    
    if (section.bendPoints) {
        section.bendPoints.forEach(bp => {
            path += `L ${bp.x} ${bp.y} `;
        });
    }
    
    path += `L ${section.endPoint.x} ${section.endPoint.y}`;
    
    const edgePath = svgGroup.append('path')
        .attr('class', 'edge')
        .attr('id', edge.id)
        .attr('d', path)
        .style('stroke', '#607d8b')
        .style('stroke-width', 2.5)
        .style('fill', 'none')
        .attr('marker-end', 'url(#arrowhead)');
    
    // Edge label
    if (edge.labels && edge.labels.length > 0) {
        const midX = (section.startPoint.x + section.endPoint.x) / 2;
        const midY = (section.startPoint.y + section.endPoint.y) / 2;
        
        const labelGroup = svgGroup.append('g')
            .attr('class', 'edge-label')
            .attr('transform', `translate(${midX}, ${midY})`);
        
        const text = edge.labels[0].text;
        const bbox = { width: text.length * 6, height: 16 };
        
        labelGroup.append('rect')
            .attr('x', -bbox.width / 2 - 4)
            .attr('y', -bbox.height / 2)
            .attr('width', bbox.width + 8)
            .attr('height', bbox.height)
            .attr('rx', 3)
            .style('fill', 'white')
            .style('stroke', '#607d8b')
            .style('stroke-width', 1);
        
        labelGroup.append('text')
            .attr('text-anchor', 'middle')
            .attr('y', 4)
            .style('font-size', '10px')
            .style('font-weight', '500')
            .style('fill', '#37474f')
            .text(text);
    }
}

/**
 * Setup zoom and pan behavior
 */
function setupZoomAndPan(svg, svgGroup, graphWidth, graphHeight) {
    const svgWidth = svg.node().getBoundingClientRect().width;
    const svgHeight = svg.node().getBoundingClientRect().height;
    
    const scaleX = svgWidth / (graphWidth + 100);
    const scaleY = svgHeight / (graphHeight + 100);
    const initialScale = Math.min(scaleX, scaleY, 0.85);
    
    const xCenterOffset = (svgWidth - graphWidth * initialScale) / 2;
    const yCenterOffset = 50;
    
    const zoom = d3.zoom()
        .scaleExtent([0.1, 3])
        .on('zoom', (event) => {
            svgGroup.attr('transform', event.transform);
        });
    
    svg.call(zoom).call(
        zoom.transform,
        d3.zoomIdentity.translate(xCenterOffset, yCenterOffset).scale(initialScale)
    );
    
    window.diagramZoom = zoom;
    window.diagramInitialTransform = { xCenterOffset, yCenterOffset, initialScale };
}
