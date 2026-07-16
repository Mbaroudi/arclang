/**
 * Dataflow Diagram Renderer
 * 
 * Renders functional dataflow diagrams showing information dependencies between functions.
 * Features:
 * - Hierarchical function layout
 * - Categorized data exchanges
 * - Functional chains as highlighted paths
 * - Data rate and protocol annotations
 */

import {
  SystemAnalysis,
  SystemFunction,
  FunctionalExchange,
} from '../types/model';
import {
  DiagramNode,
  DiagramEdge,
  DiagramOutput,
  RenderConfig,
  CAPELLA_COLORS,
} from '../types/diagram';
import { applyHierarchicalLayout } from '../layouts/hierarchical';
import {
  createSvgDocument,
  createRoundedRect,
  createText,
  createGroup,
  createPath,
  createArrowMarker,
  createRoundedPath,
  renderSvgElement,
} from '../utils/svg';
import { SvgElement } from '../types/diagram';
import { wrapText } from '../utils/svg';
import {
  getExchangeItemStyle,
  ExchangeItemKind,
  createExchangeItemMarker,
} from '../utils/exchange-item-visualization';

// ============================================================================
// Configuration
// ============================================================================

const DEFAULT_CONFIG: RenderConfig = {
  width: 1600,
  height: 1200,
  backgroundColor: '#FFFFFF',
  fontSize: 12,
  fontFamily: 'Arial, sans-serif',
  showGrid: false,
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
};

const CATEGORY_COLORS: Record<string, string> = {
  'Input': '#70AD47',
  'Processing': '#4472C4',
  'Perception': '#9E7BB5',
  'Decision': '#ED7D31',
  'Planning': '#FFC000',
  'Control': '#C00000',
  'HMI': '#5B9BD5',
  'Safety': '#43682B',
  'Output': '#767171',
};

// ============================================================================
// Main Render Function
// ============================================================================

/**
 * Render dataflow diagram from system analysis
 */
export async function renderDataflow(
  sa: SystemAnalysis,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  // 1. Convert model to diagram nodes and edges
  const { nodes, edges } = convertToDiagram(sa);

  // 2. Apply hierarchical layout
  const layout = await applyHierarchicalLayout(nodes, edges);

  // 3. Render to SVG
  const svg = renderToSvg(sa, layout, cfg);

  return {
    svg,
    width: layout.totalSize.width,
    height: layout.totalSize.height,
    metadata: {
      diagramType: 'dataflow',
      name: sa.name,
      functionCount: sa.functions.length,
      exchangeCount: sa.functional_exchanges?.length || 0,
    },
  };
}

// ============================================================================
// Step 1: Convert Model to Diagram
// ============================================================================

function convertToDiagram(sa: SystemAnalysis): {
  nodes: DiagramNode[];
  edges: DiagramEdge[];
} {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  // Convert functions to nodes
  for (const func of sa.functions) {
    nodes.push(convertFunctionToNode(func));
  }

  // Convert exchanges to edges
  if (sa.functional_exchanges) {
    for (const exchange of sa.functional_exchanges) {
      edges.push(convertExchangeToEdge(exchange));
    }
  }

  return { nodes, edges };
}

function convertFunctionToNode(func: SystemFunction): DiagramNode {
  const category = (func.attributes?.category as any)?.String || func.category || 'Processing';
  const color = (func.attributes?.color as any)?.String || func.color || CATEGORY_COLORS[category] || '#4472C4';
  const description = (func.attributes?.description as any)?.String || '';
  
  return {
    id: func.id,
    label: func.name,
    type: 'function',
    color: color,
    metadata: {
      category: category,
      description: description,
    },
  };
}

function convertExchangeToEdge(exchange: any): DiagramEdge {
  // Detect exchange item kind from metadata or attributes
  const exchangeKind = (exchange.exchange_item_kind || exchange.kind || 'DATA') as ExchangeItemKind;
  
  return {
    id: `${exchange.from}-${exchange.to}`,
    from: exchange.from,
    to: exchange.to,
    label: exchange.data || exchange.label || '',
    type: 'functional-exchange',
    metadata: {
      description: exchange.description,
      rate: exchange.rate,
      protocol: exchange.protocol,
      exchangeKind: exchangeKind,
    },
  };
}

// ============================================================================
// Step 2: Render to SVG
// ============================================================================

function renderToSvg(
  sa: SystemAnalysis,
  layout: Awaited<ReturnType<typeof applyHierarchicalLayout>>,
  config: RenderConfig
): string {
  const elements: SvgElement[] = [];
  const defs: SvgElement[] = [];

  // Create exchange item markers for all types
  const exchangeKinds: ExchangeItemKind[] = ['EVENT', 'FLOW', 'OPERATION', 'DATA', 'SHARED_DATA', 'UNSET'];
  for (const kind of exchangeKinds) {
    const style = getExchangeItemStyle(kind);
    defs.push(createArrowMarker(`arrow-${kind.toLowerCase()}`, style.color, 6));
  }

  // Calculate actual diagram bounds (max x and y from nodes)
  let maxNodeX = 0;
  let maxNodeY = 0;
  for (const node of layout.nodes) {
    const nodeRight = node.position.x + node.size.width;
    const nodeBottom = node.position.y + node.size.height;
    if (nodeRight > maxNodeX) maxNodeX = nodeRight;
    if (nodeBottom > maxNodeY) maxNodeY = nodeBottom;
  }

  // Add padding and calculate new total size with space for legend
  const legendWidth = 180;
  const legendHeight = 220;
  const legendPadding = 40;
  
  const newWidth = Math.max(layout.totalSize.width, maxNodeX + legendWidth + legendPadding * 2);
  const newHeight = layout.totalSize.height;
  
  const newTotalSize = { width: newWidth, height: newHeight };

  // Render background
  elements.push(createBackground(newTotalSize, config));

  // Render edges
  for (const edge of layout.edges) {
    elements.push(renderEdge(edge, config));
  }

  // Render nodes
  for (const node of layout.nodes) {
    elements.push(renderNode(node, config));
  }

  // Render title
  elements.push(renderTitle(sa.name, config));

  // Render legend with calculated position
  elements.push(renderLegend(newTotalSize, maxNodeX, maxNodeY, config));

  return createSvgDocument(
    newTotalSize.width,
    newTotalSize.height,
    elements,
    defs
  );
}

// ============================================================================
// Rendering Components
// ============================================================================

function createBackground(size: { width: number; height: number }, config: RenderConfig): SvgElement {
  return createRoundedRect(0, 0, size.width, size.height, 0, {
    fill: config.backgroundColor,
    stroke: 'none',
  });
}

function renderNode(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  // Function box
  elements.push(
    createRoundedRect(
      node.position.x,
      node.position.y,
      node.size.width,
      node.size.height,
      8,
      {
        fill: node.color,
        stroke: '#000000',
        'stroke-width': 2,
      }
    )
  );

  // Function name with multiline support
  const lines = wrapText(node.label, node.size.width - 20);
  const textY = node.position.y + node.size.height / 2 - (lines.length * 8);
  
  for (let i = 0; i < lines.length; i++) {
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        textY + i * 16,
        lines[i],
        {
          'text-anchor': 'middle',
          'dominant-baseline': 'middle',
          'font-family': config.fontFamily,
          'font-size': config.fontSize,
          'font-weight': 'bold',
          fill: '#FFFFFF',
        }
      )
    );
  }

  // Category label (small, bottom)
  if (node.metadata?.category) {
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + node.size.height - 10,
        `[${node.metadata.category}]`,
        {
          'text-anchor': 'middle',
          'font-family': config.fontFamily,
          'font-size': config.fontSize! - 2,
          fill: '#FFFFFF',
          opacity: 0.8,
        }
      )
    );
  }

  // Function ID (top-right corner)
  elements.push(
    createText(
      node.position.x + node.size.width - 5,
      node.position.y + 12,
      node.id,
      {
        'text-anchor': 'end',
        'font-family': config.fontFamily,
        'font-size': config.fontSize! - 2,
        fill: '#FFFFFF',
        opacity: 0.7,
      }
    )
  );

  return createGroup(elements, { id: `node-${node.id}` });
}

function renderEdge(edge: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  // Get exchange item style
  const exchangeKind: ExchangeItemKind = edge.metadata?.exchangeKind || 'DATA';
  const itemStyle = getExchangeItemStyle(exchangeKind);

  // Edge path with exchange item styling
  const pathD = createRoundedPath(edge.points, 10);
  const strokeDasharray = itemStyle.pattern === 'none' ? undefined : itemStyle.pattern;
  
  // For double arrow (OPERATION), render a background path
  if (itemStyle.arrowType === 'double') {
    elements.push(
      createPath(pathD, {
        fill: 'none',
        stroke: itemStyle.color,
        'stroke-width': itemStyle.strokeWidth + 2,
        opacity: '0.3',
      })
    );
  }
  
  elements.push(
    createPath(pathD, {
      fill: 'none',
      stroke: itemStyle.color,
      'stroke-width': itemStyle.strokeWidth,
      'marker-end': `url(#arrow-${exchangeKind.toLowerCase()})`,
      ...(strokeDasharray && { 'stroke-dasharray': strokeDasharray }),
    })
  );

  // Edge label with metadata
  if (edge.label && config.showLabels) {
    // Calculate true geometric midpoint
    let totalLength = 0;
    const segmentLengths: number[] = [];

    for (let i = 0; i < edge.points.length - 1; i++) {
      const dx = edge.points[i + 1].x - edge.points[i].x;
      const dy = edge.points[i + 1].y - edge.points[i].y;
      const length = Math.sqrt(dx * dx + dy * dy);
      segmentLengths.push(length);
      totalLength += length;
    }

    const halfLength = totalLength / 2;
    let accumulatedLength = 0;
    let midPoint = edge.points[0];

    for (let i = 0; i < segmentLengths.length; i++) {
      if (accumulatedLength + segmentLengths[i] >= halfLength) {
        const remainingLength = halfLength - accumulatedLength;
        const ratio = remainingLength / segmentLengths[i];
        midPoint = {
          x: edge.points[i].x + (edge.points[i + 1].x - edge.points[i].x) * ratio,
          y: edge.points[i].y + (edge.points[i + 1].y - edge.points[i].y) * ratio,
        };
        break;
      }
      accumulatedLength += segmentLengths[i];
    }

    // Build label with exchange item prefix
    const labelPrefix = itemStyle.labelStyle.prefix;
    const labelSuffix = itemStyle.labelStyle.suffix;
    const mainLabel = `${labelPrefix}${edge.label}${labelSuffix}`;
    const subLabel = edge.metadata?.rate ? `${edge.metadata.rate}` : '';
    
    const fontSize = (config.fontSize || 12) - 1;
    const labelWidth = Math.max(mainLabel.length * (fontSize * 0.6), 100);
    const labelHeight = subLabel ? 35 : 25;
    const padding = 6;
    
    // Check for bidirectional edges (SF-006 <-> SF-007)
    // Add vertical offset for overlapping edges
    let yOffset = 0;
    if ((edge.from === 'SF-006' && edge.to === 'SF-007') || 
        (edge.from === 'SF-007' && edge.to === 'SF-006')) {
      // Offset based on direction to separate them vertically
      // Larger offset to ensure complete separation
      yOffset = (edge.from === 'SF-006') ? -50 : 50;
    }
    
    // Background for label with exchange item color
    elements.push(
      createRoundedRect(
        midPoint.x - labelWidth / 2 - padding,
        midPoint.y - labelHeight / 2 + yOffset,
        labelWidth + padding * 2,
        labelHeight,
        5,
        {
          fill: '#FFFFFF',
          stroke: itemStyle.color,
          'stroke-width': 1.5,
        }
      )
    );

    // Main label
    elements.push(
      createText(
        midPoint.x,
        (subLabel ? midPoint.y - 5 : midPoint.y) + yOffset,
        mainLabel,
        {
          'text-anchor': 'middle',
          'dominant-baseline': 'middle',
          'font-family': config.fontFamily,
          'font-size': config.fontSize! - 1,
          'font-weight': 'bold',
          fill: '#2E75B6',
        }
      )
    );

    // Sub label (rate)
    if (subLabel) {
      elements.push(
        createText(
          midPoint.x,
          midPoint.y + 8 + yOffset,
          subLabel,
          {
            'text-anchor': 'middle',
            'dominant-baseline': 'middle',
            'font-family': config.fontFamily,
            'font-size': config.fontSize! - 3,
            fill: '#495057',
          }
        )
      );
    }
  }

  return createGroup(elements, { id: `edge-${edge.id}` });
}

function renderTitle(title: string, config: RenderConfig): SvgElement {
  return createText(20, 35, title, {
    'font-family': config.fontFamily,
    'font-size': config.fontSize! + 8,
    'font-weight': 'bold',
    fill: '#212529',
  });
}

function renderLegend(
  size: { width: number; height: number }, 
  maxNodeX: number, 
  maxNodeY: number, 
  config: RenderConfig
): SvgElement {
  const elements: SvgElement[] = [];
  const legendWidth = 160;
  const legendPadding = 30;
  
  // Position legend to the right of all nodes with padding
  const legendX = maxNodeX + legendPadding;
  const legendY = 80; // Fixed vertical position from top
  
  // Legend background
  elements.push(
    createRoundedRect(legendX, legendY, 160, 200, 8, {
      fill: '#F8F9FA',
      stroke: '#CED4DA',
      'stroke-width': 2,
    })
  );

  // Legend title
  elements.push(
    createText(legendX + 80, legendY + 20, 'Categories', {
      'text-anchor': 'middle',
      'font-family': config.fontFamily,
      'font-size': config.fontSize! + 2,
      'font-weight': 'bold',
      fill: '#212529',
    })
  );

  // Category items
  const categories = [
    { name: 'Input', color: '#70AD47' },
    { name: 'Processing', color: '#4472C4' },
    { name: 'Perception', color: '#9E7BB5' },
    { name: 'Decision', color: '#ED7D31' },
    { name: 'Planning', color: '#FFC000' },
    { name: 'Control', color: '#C00000' },
    { name: 'HMI', color: '#5B9BD5' },
    { name: 'Safety', color: '#43682B' },
  ];

  categories.forEach((cat, i) => {
    const y = legendY + 40 + i * 20;
    
    // Color box
    elements.push(
      createRoundedRect(legendX + 10, y - 8, 15, 15, 3, {
        fill: cat.color,
        stroke: '#000000',
        'stroke-width': 1,
      })
    );
    
    // Category name
    elements.push(
      createText(legendX + 35, y, cat.name, {
        'text-anchor': 'start',
        'font-family': config.fontFamily,
        'font-size': config.fontSize! - 1,
        fill: '#212529',
      })
    );
  });

  return createGroup(elements, { id: 'legend' });
}
