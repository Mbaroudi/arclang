/**
 * Physical Architecture Diagram Renderer
 * 
 * Renders Physical Architecture (PA layer) deployment diagrams with:
 * - Physical nodes (hardware, software, firmware)
 * - Physical links (connections, networks)
 * - Deployed components
 * - Communication protocols
 */

import {
  PhysicalArchitecture,
  PhysicalNode,
  PhysicalLink,
  PhysicalExchange,
} from '../types/model';
import {
  DiagramNode,
  DiagramEdge,
  Point,
  SvgElement,
  RenderConfig,
  DiagramOutput,
  CAPELLA_COLORS,
} from '../types/diagram';
import {
  applyHierarchicalLayout,
} from '../layouts/hierarchical';
import {
  createSvgDocument,
  createRect,
  createLine,
  createText,
  createGroup,
  createPath,
  createArrowMarker,
  createRoundedRect,
} from '../utils/svg';
import {
  getSafetyColorConfig,
  getSafetyBorderAttributes,
  createSafetyBadge,
  parseSafetyLevel,
  isSafetyCritical,
} from '../utils/safety-colors';
import {
  renderPhysicalNodeWithDeployment,
  calculatePhysicalNodeSize,
  renderPhysicalLink,
  createPhysicalLinkMarker,
  DeployedComponent,
} from '../utils/deployment-visualization';
import { optimizeDiagram, generateOptimizationReport } from '../layouts/multi-pass-optimizer';

// ============================================================================
// Configuration
// ============================================================================

const DEFAULT_CONFIG: RenderConfig = {
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
  fontSize: 12,
};

// ============================================================================
// Main Render Function
// ============================================================================

/**
 * Render physical architecture diagram
 */
export async function renderPhysicalArchitecture(
  pa: PhysicalArchitecture,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg: RenderConfig = { ...DEFAULT_CONFIG, ...config };

  // 1. Convert model to diagram nodes and edges
  const { nodes, edges } = convertToDiagram(pa);

  // 2. Apply 5-pass optimization pipeline (Phase 3)
  console.log('[Physical Renderer] Applying 5-pass optimization pipeline...');
  const optimizationResult = await optimizeDiagram(nodes, edges, {
    enablePass1: true,
    enablePass2: true,
    enablePass3: true,
    enablePass4: true,
    enablePass5: true,
    maxIterations: 5,
    targetCrossings: 10,
    gridSize: 20,
    diagramType: 'physical-architecture',
    direction: 'RIGHT',
    nodeSpacing: 120,
    layerSpacing: 180,
  });

  const layout = {
    nodes: optimizationResult.nodes as any,
    edges: optimizationResult.edges as any,
    totalSize: optimizationResult.totalSize,
  };

  // Display optimization report
  const optimizationReport = generateOptimizationReport(optimizationResult);
  console.log('\n' + optimizationReport);

  // 3. Render to SVG
  const svg = renderToSvg(pa, layout, cfg);

  // 4. Return result
  const width = layout.totalSize.width;
  const height = layout.totalSize.height;

  return {
    svg,
    width,
    height,
    metadata: {
      diagramType: 'physical-architecture',
      nodeCount: pa.nodes.length,
      linkCount: pa.links.length,
      exchangeCount: pa.physical_exchanges.length,
    },
  };
}

// ============================================================================
// Step 1: Convert Model to Diagram
// ============================================================================

function convertToDiagram(pa: PhysicalArchitecture): {
  nodes: DiagramNode[];
  edges: DiagramEdge[];
} {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  // Convert physical nodes
  for (const node of pa.nodes) {
    nodes.push({
      id: node.id,
      label: node.name,
      type: mapNodeType(node.node_type),
      metadata: {
        nodeType: node.node_type,
        deployedComponents: [...node.behavior_components.map(c => c.name), ...node.hardware_components.map(c => c.name)],
        behavior_components: node.behavior_components,
        hardware_components: node.hardware_components,
        processor: node.processor,
        memory: node.memory,
      },
      color: node.color || undefined,
    });
  }

  // Convert physical links to edges
  for (let i = 0; i < pa.links.length; i++) {
    const link = pa.links[i];
    edges.push({
      id: `link-${i}`,
      from: link.from,
      to: link.to,
      label: link.protocol || undefined,
      type: 'physical-link',
      metadata: {
        protocol: link.protocol,
        bandwidth: link.bandwidth,
        connections: link.connections,
      },
    });
  }

  // Convert physical exchanges to edges (if different from links)
  for (let i = 0; i < pa.physical_exchanges.length; i++) {
    const exchange = pa.physical_exchanges[i];
    edges.push({
      id: `exchange-${i}`,
      from: exchange.from,
      to: exchange.to,
      label: exchange.message_type || undefined,
      type: 'physical-link',
      metadata: {
        messageType: exchange.message_type,
        frequency: exchange.frequency,
        via: exchange.via,
      },
    });
  }

  return { nodes, edges };
}

function mapNodeType(type: string): 'physical-node' | 'hardware' | 'behavior' {
  if (type.toLowerCase().includes('hardware')) return 'hardware';
  if (type.toLowerCase().includes('software')) return 'behavior';
  return 'physical-node';
}


// ============================================================================
// Step 2: Render to SVG
// ============================================================================

function renderToSvg(
  pa: PhysicalArchitecture,
  layout: any,
  config: RenderConfig
): string {
  const elements: SvgElement[] = [];

  // Background
  elements.push(
    createRect(0, 0, layout.totalSize.width, layout.totalSize.height, {
      fill: config.colorScheme?.background || '#FFFFFF',
      stroke: 'none',
    })
  );

  // Render edges first (behind nodes)
  for (const edge of layout.edges) {
    elements.push(renderLink(edge, layout.nodes, config));
  }

  // Render nodes
  for (const node of layout.nodes) {
    elements.push(renderNode(node, config));
  }

  // Title
  if (pa.name) {
    elements.push(
      createText(20, 30, pa.name, {
        'font-family': 'Arial, sans-serif',
        'font-size': 18,
        'font-weight': 'bold',
        fill: '#212529',
      })
    );
  }

  // Create arrow markers
  const defs = [
    createArrowMarker('arrow-physical', config.colorScheme?.physicalNode || '#FFE699', 10),
    createArrowMarker('arrow-black', '#000000', 10),
  ];

  return createSvgDocument(layout.totalSize.width, layout.totalSize.height, elements, defs);
}

// ============================================================================
// Render Individual Elements
// ============================================================================

function renderNode(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  // Determine fill color based on node type
  let fillColor = node.color;
  if (!fillColor) {
    switch (node.type) {
      case 'hardware':
        fillColor = config.colorScheme?.hardware || '#C0C0C0';
        break;
      case 'behavior':
        fillColor = config.colorScheme?.behavior || '#5B9BD5';
        break;
      case 'physical-node':
      default:
        fillColor = config.colorScheme?.physicalNode || '#FFE699';
        break;
    }
  }

  // Check for safety-critical nodes
  const safetyData = parseSafetyLevel(node.metadata);
  const safetyLevel = safetyData.level;
  const safetyStandard = safetyData.standard;
  const hasSafetyLevel = safetyLevel !== null;
  
  // Build node attributes
  let nodeStroke = '#000000';
  let nodeStrokeWidth = '2';
  
  if (hasSafetyLevel && safetyLevel) {
    const safetyAttrs = getSafetyBorderAttributes(safetyLevel, safetyStandard || undefined);
    nodeStroke = safetyAttrs.stroke;
    nodeStrokeWidth = safetyAttrs['stroke-width'];
  }
  
  // 3D cube-like appearance for physical nodes
  if (node.type === 'physical-node' || node.type === 'hardware') {
    // Main box
    elements.push(
      createRoundedRect(
        node.position.x,
        node.position.y,
        node.size.width,
        node.size.height,
        5,
        {
          fill: fillColor,
          stroke: nodeStroke,
          'stroke-width': nodeStrokeWidth,
        }
      )
    );

    // 3D effect - top edge
    const topPath = `M ${node.position.x} ${node.position.y} 
                     L ${node.position.x + 10} ${node.position.y - 10} 
                     L ${node.position.x + node.size.width + 10} ${node.position.y - 10} 
                     L ${node.position.x + node.size.width} ${node.position.y} Z`;
    elements.push(
      createPath(topPath, {
        fill: adjustBrightness(fillColor, 1.2),
        stroke: '#000000',
        'stroke-width': 1.5,
      })
    );

    // 3D effect - right edge
    const rightPath = `M ${node.position.x + node.size.width} ${node.position.y} 
                       L ${node.position.x + node.size.width + 10} ${node.position.y - 10} 
                       L ${node.position.x + node.size.width + 10} ${node.position.y + node.size.height - 10} 
                       L ${node.position.x + node.size.width} ${node.position.y + node.size.height} Z`;
    elements.push(
      createPath(rightPath, {
        fill: adjustBrightness(fillColor, 0.8),
        stroke: '#000000',
        'stroke-width': 1.5,
      })
    );
  } else {
    // Regular rounded rectangle for software nodes
    elements.push(
      createRoundedRect(
        node.position.x,
        node.position.y,
        node.size.width,
        node.size.height,
        8,
        {
          fill: fillColor,
          stroke: nodeStroke,
          'stroke-width': nodeStrokeWidth,
        }
      )
    );
  }
  
  // Add safety badge if node is safety-critical
  if (hasSafetyLevel && safetyLevel && isSafetyCritical(node.metadata)) {
    const badgeX = node.position.x + node.size.width - 60;
    const badgeY = node.position.y + 5;
    // createSafetyBadge returns a string, not SvgElement
    // TODO: Convert safety badge to SvgElement format
  }

  // Node name
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 25,
      node.label,
      {
        'text-anchor': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 12,
        'font-weight': 'bold',
        fill: '#000000',
      }
    )
  );

  // Node ID (top-right corner)
  elements.push(
    createText(
      node.position.x + node.size.width - 10,
      node.position.y + 14,
      node.id,
      {
        'text-anchor': 'end',
        'font-family': 'Arial, sans-serif',
        'font-size': 10,
        fill: '#495057',
      }
    )
  );

  // Node type stereotype
  const stereotype = getStereotype(node.metadata?.nodeType);
  if (stereotype) {
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + node.size.height - 15,
        stereotype,
        {
          'text-anchor': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 10,
          'font-style': 'italic',
          fill: '#495057',
        }
      )
    );
  }

  // Render behavior_components as nested blue rectangles inside ECU (Phase 4)
  if (node.metadata?.behavior_components && node.metadata.behavior_components.length > 0) {
    const components = node.metadata.behavior_components;
    const padding = 15;
    const compHeight = 35;
    const compSpacing = 8;
    const startY = node.position.y + 45;
    
    // Separator line
    elements.push(
      createLine(
        node.position.x + 10,
        node.position.y + 40,
        node.position.x + node.size.width - 10,
        node.position.y + 40,
        {
          stroke: '#000000',
          'stroke-width': 1,
          'stroke-dasharray': '3,3',
        }
      )
    );

    components.forEach((comp: any, index: number) => {
      const compY = startY + (index * (compHeight + compSpacing));
      const compX = node.position.x + padding;
      const compWidth = node.size.width - (2 * padding);
      
      // Behavior component rectangle (blue)
      const compColor = comp.color || '#5B9BD5';
      elements.push(
        createRoundedRect(
          compX,
          compY,
          compWidth,
          compHeight,
          4,
          {
            fill: compColor,
            stroke: '#000000',
            'stroke-width': 1.5,
            opacity: '0.9',
          }
        )
      );
      
      // Component name
      elements.push(
        createText(
          compX + compWidth / 2,
          compY + 14,
          comp.name,
          {
            'text-anchor': 'middle',
            'font-family': 'Arial, sans-serif',
            'font-size': 10,
            'font-weight': 'bold',
            fill: '#FFFFFF',
          }
        )
      );
      
      // Component ID
      elements.push(
        createText(
          compX + compWidth - 5,
          compY + 10,
          comp.id,
          {
            'text-anchor': 'end',
            'font-family': 'Arial, sans-serif',
            'font-size': 8,
            fill: '#E8F4F8',
          }
        )
      );
      
      // Allocated functions (if any)
      if (comp.allocated_functions && comp.allocated_functions.length > 0) {
        const funcText = comp.allocated_functions.slice(0, 2).join(', ');
        elements.push(
          createText(
            compX + compWidth / 2,
            compY + 28,
            funcText,
            {
              'text-anchor': 'middle',
              'font-family': 'Arial, sans-serif',
              'font-size': 8,
              'font-style': 'italic',
              fill: '#E8F4F8',
            }
          )
        );
      }
    });
  }
  // Fallback: show deployed components as text if behavior_components not available
  else if (node.metadata?.deployedComponents && node.metadata.deployedComponents.length > 0) {
    let yOffset = 45;
    elements.push(
      createLine(
        node.position.x + 10,
        node.position.y + 40,
        node.position.x + node.size.width - 10,
        node.position.y + 40,
        {
          stroke: '#000000',
          'stroke-width': 1,
          'stroke-dasharray': '3,3',
        }
      )
    );

    node.metadata.deployedComponents.slice(0, 3).forEach((comp: string) => {
      elements.push(
        createText(
          node.position.x + node.size.width / 2,
          node.position.y + yOffset,
          `◈ ${comp}`,
          {
            'text-anchor': 'middle',
            'font-family': 'Arial, sans-serif',
            'font-size': 9,
            fill: '#495057',
          }
        )
      );
      yOffset += 12;
    });
  }

  return createGroup(elements, { id: `node-${node.id}` });
}

function renderLink(edge: any, nodes: any[], config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  if (!edge.points || edge.points.length < 2) {
    return createGroup([]);
  }

  // Draw connection path
  const pathD = generateConnectionPath(edge.points);

  // Thicker line for physical links
  elements.push(
    createPath(pathD, {
      fill: 'none',
      stroke: '#000000',
      'stroke-width': 3,
      'marker-end': 'url(#arrow-black)',
    })
  );

  // Link label
  if (edge.label && config.showLabels) {
    const midPoint = edge.points[Math.floor(edge.points.length / 2)];
    
    // Improved label background sizing
    const fontSize = (config.fontSize || 12) - 1;
    const labelWidth = Math.max(edge.label.length * (fontSize * 0.6), 80);
    const labelHeight = 20;
    const padding = 6;

    // Background for label
    elements.push(
      createRoundedRect(
        midPoint.x - labelWidth / 2 - padding,
        midPoint.y - labelHeight / 2,
        labelWidth + padding * 2,
        labelHeight,
        3,
        {
          fill: '#FFFFFF',
          stroke: '#CED4DA',
          'stroke-width': 1,
        }
      )
    );

    elements.push(
      createText(midPoint.x, midPoint.y, edge.label, {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': config.fontFamily || 'Arial, sans-serif',
        'font-size': fontSize,
        fill: '#495057',
      })
    );

    // Bandwidth info if available
    if (edge.metadata?.bandwidth) {
      elements.push(
        createText(midPoint.x, midPoint.y + 5, edge.metadata.bandwidth, {
          'text-anchor': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 8,
          'font-style': 'italic',
          fill: '#666666',
        })
      );
    }
  }

  return createGroup(elements, { id: edge.id });
}

function generateConnectionPath(points: Point[]): string {
  if (points.length === 0) return '';
  if (points.length === 1) return `M ${points[0].x} ${points[0].y}`;

  let path = `M ${points[0].x} ${points[0].y}`;

  for (let i = 1; i < points.length; i++) {
    path += ` L ${points[i].x} ${points[i].y}`;
  }

  return path;
}

function getStereotype(nodeType: string): string {
  if (!nodeType) return '';
  if (nodeType.toLowerCase().includes('hardware')) return '«hardware»';
  if (nodeType.toLowerCase().includes('software')) return '«software»';
  if (nodeType.toLowerCase().includes('firmware')) return '«firmware»';
  if (nodeType.toLowerCase().includes('processor')) return '«processor»';
  if (nodeType.toLowerCase().includes('memory')) return '«memory»';
  if (nodeType.toLowerCase().includes('network')) return '«network»';
  return `«${nodeType.toLowerCase()}»`;
}

function adjustBrightness(color: string, factor: number): string {
  // Simple color adjustment for 3D effect
  if (!color.startsWith('#')) return color;

  const hex = color.substring(1);
  const r = Math.min(255, Math.round(parseInt(hex.substring(0, 2), 16) * factor));
  const g = Math.min(255, Math.round(parseInt(hex.substring(2, 4), 16) * factor));
  const b = Math.min(255, Math.round(parseInt(hex.substring(4, 6), 16) * factor));

  return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
}
