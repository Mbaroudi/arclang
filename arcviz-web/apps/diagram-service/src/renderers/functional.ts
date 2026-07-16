/**
 * Functional Dataflow Diagram Renderer
 * 
 * Renders system analysis functional dataflow diagrams matching Capella's visual style.
 * Features:
 * - Green function boxes (#70AD47)
 * - Functional ports (small squares on borders)
 * - Port-to-port connections
 * - Function hierarchy (sub-functions)
 * - External actors
 * - Data type labels on flows
 */

import {
  SystemAnalysis,
  SystemFunction,
  FunctionalExchange,
  ExternalActor,
} from '../types/model';
import {
  DiagramNode,
  DiagramEdge,
  DiagramOutput,
  RenderConfig,
  CAPELLA_COLORS,
  Port,
} from '../types/diagram';
import { applyHierarchicalLayout, assignPortSides } from '../layouts/hierarchical';
import { optimizeDiagram, generateOptimizationReport } from '../layouts/multi-pass-optimizer';
import {
  createSvgDocument,
  createRoundedRect,
  createRect,
  createText,
  createGroup,
  createPath,
  createArrowMarker,
  createMultilineText,
  wrapText,
  createRoundedPath,
  renderSvgElement,
} from '../utils/svg';
import { SvgElement } from '../types/diagram';
import {
  getExchangeItemStyle,
  ExchangeItemKind,
} from '../utils/exchange-item-visualization';
import {
  calculateSystemBoundary,
  renderSystemBoundary,
  validateSystemBoundary,
} from '../utils/system-boundary';
import {
  validatePortPositioning,
  getPortStatistics,
} from '../utils/port-validation';
import {
  validateDiagramQuality,
  generateQualityReport,
} from '../utils/quality-metrics';

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

const PORT_SIZE = 10;
const FUNCTION_DEFAULT_WIDTH = 180;
const FUNCTION_DEFAULT_HEIGHT = 100;

// ============================================================================
// Main Render Function
// ============================================================================

/**
 * Render functional dataflow diagram from ArcLang model
 */
export async function renderFunctionalDataflow(
  sa: SystemAnalysis,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  // 1. Convert model to diagram nodes and edges
  let { nodes, edges } = convertToDiagram(sa);

  // 2. Assign port sides based on connections
  nodes = assignPortSides(nodes, edges);

  // 3. Apply 5-pass optimization pipeline (Phase 3 - Section 8.2)
  console.log('[Functional Renderer] Applying 5-pass optimization pipeline...');
  const optimizationResult = await optimizeDiagram(nodes, edges, {
    enablePass1: true,
    enablePass2: true,
    enablePass3: true,
    enablePass4: true,
    enablePass5: true,
    maxIterations: 5,
    targetCrossings: 10,
    gridSize: 20,
    diagramType: 'functional-dataflow',
    direction: 'RIGHT',
    nodeSpacing: 80,
    layerSpacing: 120,
  });

  const layout = {
    nodes: optimizationResult.nodes as any,
    edges: optimizationResult.edges as any,
    totalSize: optimizationResult.totalSize,
  };

  // Display optimization report
  const optimizationReport = generateOptimizationReport(optimizationResult);
  console.log('\n' + optimizationReport);

  // 4. Calculate system boundary (CRITICAL for SAB compliance - Capella Spec 4.2)
  const boundaryData = calculateSystemBoundary(layout.nodes, {
    labelText: sa.name || 'System',
    strokeColor: '#2E75B6',  // Capella system blue
    fillColor: '#E8F4F8',    // Very light blue
  });

  const boundary = { position: boundaryData.position, size: boundaryData.size };
  const boundarySvg = renderSystemBoundary(boundary, {
    labelText: sa.name || 'System',
    strokeColor: '#2E75B6',
    fillColor: '#E8F4F8',
  });

  // 5. Validate boundary compliance
  const boundaryValidation = validateSystemBoundary(layout.nodes, boundary);
  
  if (!boundaryValidation.valid) {
    console.warn('[SAB Compliance] System boundary violations:', boundaryValidation.violations);
  }

  // Mark nodes with system boundary rendered (for quality metrics)
  layout.nodes.forEach(node => {
    if (!node.metadata) node.metadata = {};
    node.metadata.system_boundary_rendered = true;
  });

  // 6. Validate port positioning (MANDATORY per Capella Spec 5.1)
  const portValidation = validatePortPositioning(layout.nodes);
  const portStats = getPortStatistics(layout.nodes);
  
  if (!portValidation.valid) {
    console.warn('[Port Compliance] Port positioning violations:', portValidation.violations);
  }
  if (portValidation.warnings.length > 0) {
    console.warn('[Port Compliance] Port positioning warnings:', portValidation.warnings);
  }

  // Quality metrics already validated in Pass 5, reuse results
  const qualityMetrics = optimizationResult.qualityMetrics;
  const qualityReport = generateQualityReport(qualityMetrics);
  
  console.log('\n[Quality Report from Pass 5]');
  console.log(qualityReport);
  
  if (qualityMetrics.overallScore < 75) {
    console.warn(`[Quality] Diagram quality (${qualityMetrics.overallScore.toFixed(1)}) below production threshold (75)`);
  }

  // 8. Render to SVG with boundary
  const svg = renderToSvg(sa, layout, boundarySvg, cfg);

  return {
    svg,
    width: layout.totalSize.width,
    height: layout.totalSize.height,
    metadata: {
      diagramType: 'functional-dataflow',
      name: sa.name,
      functionCount: sa.functions.length,
      exchangeCount: sa.functional_exchanges.length,
      systemBoundary: boundaryValidation.valid,
      boundaryViolations: boundaryValidation.violations,
      portPositioning: portValidation.valid,
      portViolations: portValidation.violations.length,
      portWarnings: portValidation.warnings.length,
      portStats,
      qualityScore: qualityMetrics.overallScore,
      qualityLevel: qualityMetrics.qualityLevel,
      regulatoryCompliance: qualityMetrics.regulatoryCompliance,
      qualityReport,
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

  // Convert external actors to nodes
  for (const actor of sa.external_actors) {
    nodes.push(convertActorToNode(actor));
  }

  // Convert functional exchanges to edges
  for (const exchange of sa.functional_exchanges) {
    edges.push(convertExchangeToEdge(exchange));
  }

  return { nodes, edges };
}

function convertFunctionToNode(func: SystemFunction): DiagramNode {
  // Convert function ports
  const ports: Port[] = func.ports.map((p, idx) => ({
    id: p.name,
    name: p.name,
    direction: p.direction === 'In' ? 'IN' : p.direction === 'Out' ? 'OUT' : 'INOUT',
    side: p.direction === 'In' ? 'LEFT' : 'RIGHT',
    position: undefined,
  }));

  return {
    id: func.id,
    label: func.name,
    type: 'function',
    color: func.color || CAPELLA_COLORS.function,
    icon: func.icon || undefined,
    ports,
    children: func.sub_functions.map(convertFunctionToNode),
    metadata: {
      category: func.category,
      sub_functions: func.sub_functions,
    },
    size: {
      width: FUNCTION_DEFAULT_WIDTH,
      height: FUNCTION_DEFAULT_HEIGHT,
    },
  };
}

function convertActorToNode(actor: ExternalActor): DiagramNode {
  return {
    id: actor.id,
    label: actor.name,
    type: 'actor',
    color: actor.color || CAPELLA_COLORS.actor,
    metadata: {
      external: true,
    },
    size: {
      width: 120,
      height: 80,
    },
  };
}

function convertExchangeToEdge(exchange: FunctionalExchange): DiagramEdge {
  // Parse port references (format: "FunctionID.PortName")
  const [fromFunc, fromPort] = exchange.from_port.split('.');
  const [toFunc, toPort] = exchange.to_port.split('.');

  // Detect exchange item kind
  const exchangeKind = ((exchange as any).exchange_item_kind || (exchange as any).kind || 'DATA') as ExchangeItemKind;

  return {
    id: `${exchange.from_port}-${exchange.to_port}`,
    from: fromFunc,
    to: toFunc,
    fromPort: fromPort,
    toPort: toPort,
    label: exchange.label || exchange.data_type,
    type: 'functional-exchange',
    metadata: {
      data_type: exchange.data_type,
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
  boundarySvg: SvgElement,
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

  // Render background
  elements.push(createBackground(layout.totalSize, config));

  // Render system boundary (BEFORE edges and nodes per Capella spec)
  elements.push(boundarySvg);

  // Render edges (draw after boundary, before nodes)
  for (const edge of layout.edges) {
    elements.push(renderEdge(edge, layout.nodes, config));
  }

  // Render nodes (on top of everything)
  for (const node of layout.nodes) {
    elements.push(renderNode(node, config));
  }

  // Render title
  elements.push(renderTitle(sa.name, config));

  return createSvgDocument(
    layout.totalSize.width,
    layout.totalSize.height,
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

  if (node.type === 'function') {
    // Function box (green)
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

    // Function label
    const lines = wrapText(node.label, node.size.width - 40);
    const textY = node.position.y + 30;

    for (let i = 0; i < lines.length; i++) {
      elements.push(
        createText(
          node.position.x + node.size.width / 2,
          textY + i * 16,
          lines[i],
          {
            'text-anchor': 'middle',
            'font-family': config.fontFamily,
            'font-size': config.fontSize,
            'font-weight': 'bold',
            fill: '#000000',
          }
        )
      );
    }

    // Function ID (top-right)
    elements.push(
      createText(
        node.position.x + node.size.width - 5,
        node.position.y + 14,
        node.id,
        {
          'text-anchor': 'end',
          'font-family': config.fontFamily,
          'font-size': config.fontSize! - 2,
          fill: '#2C5F2D',
        }
      )
    );

    // Render ports
    if (node.ports) {
      for (const port of node.ports) {
        elements.push(renderPort(port, node, config));
      }
    }
  } else if (node.type === 'actor') {
    // Actor box (blue, rounded)
    elements.push(
      createRoundedRect(
        node.position.x,
        node.position.y,
        node.size.width,
        node.size.height,
        12,
        {
          fill: node.color,
          stroke: '#000000',
          'stroke-width': 2,
          'stroke-dasharray': '5,5',
        }
      )
    );

    // Actor label
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + node.size.height / 2,
        node.label,
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

    // "External" label
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + node.size.height - 15,
        '«external»',
        {
          'text-anchor': 'middle',
          'font-family': config.fontFamily,
          'font-size': config.fontSize! - 2,
          'font-style': 'italic',
          fill: '#FFFFFF',
        }
      )
    );
  }

  return createGroup(elements, { id: `node-${node.id}` });
}

function renderPort(port: any, node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  // Calculate port position
  let portX: number, portY: number;

  if (port.position) {
    portX = port.position.x;
    portY = port.position.y;
  } else {
    // Calculate based on side
    const centerY = node.position.y + node.size.height / 2;
    const centerX = node.position.x + node.size.width / 2;

    switch (port.side) {
      case 'LEFT':
        portX = node.position.x - PORT_SIZE / 2;
        portY = centerY - PORT_SIZE / 2;
        break;
      case 'RIGHT':
        portX = node.position.x + node.size.width - PORT_SIZE / 2;
        portY = centerY - PORT_SIZE / 2;
        break;
      case 'TOP':
        portX = centerX - PORT_SIZE / 2;
        portY = node.position.y - PORT_SIZE / 2;
        break;
      case 'BOTTOM':
        portX = centerX - PORT_SIZE / 2;
        portY = node.position.y + node.size.height - PORT_SIZE / 2;
        break;
      default:
        portX = node.position.x + node.size.width - PORT_SIZE / 2;
        portY = centerY - PORT_SIZE / 2;
    }
  }

  // Port square
  elements.push(
    createRect(portX, portY, PORT_SIZE, PORT_SIZE, {
      fill: '#FFFFFF',
      stroke: '#000000',
      'stroke-width': 1.5,
    })
  );

  // Port label (small, next to port)
  const labelOffset = 15;
  let labelX = portX;
  let labelY = portY - 3;
  let anchor = 'middle';

  if (port.side === 'LEFT') {
    labelX = portX - labelOffset;
    labelY = portY + PORT_SIZE / 2;
    anchor = 'end';
  } else if (port.side === 'RIGHT') {
    labelX = portX + PORT_SIZE + labelOffset;
    labelY = portY + PORT_SIZE / 2;
    anchor = 'start';
  }

  elements.push(
    createText(labelX, labelY, port.name, {
      'text-anchor': anchor,
      'dominant-baseline': 'middle',
      'font-family': config.fontFamily,
      'font-size': config.fontSize! - 3,
      fill: '#495057',
    })
  );

  return createGroup(elements);
}

function renderEdge(edge: any, nodes: any[], config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  if (edge.points && edge.points.length > 0) {
    // Get exchange item style
    const exchangeKind: ExchangeItemKind = edge.metadata?.exchangeKind || 'DATA';
    const itemStyle = getExchangeItemStyle(exchangeKind);

    // Edge path with exchange item styling
    const pathD = createRoundedPath(edge.points, 8);
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

    // Edge label with exchange item prefix
    if (edge.label && config.showLabels) {
      const midIndex = Math.floor(edge.points.length / 2);
      const midPoint = edge.points[midIndex];

      // Build label with exchange item prefix
      const labelPrefix = itemStyle.labelStyle.prefix;
      const labelSuffix = itemStyle.labelStyle.suffix;
      const fullLabel = `${labelPrefix}${edge.label}${labelSuffix}`;

      // Background for label with exchange item color
      const fontSize = (config.fontSize || 12) - 1;
      const labelWidth = Math.max(fullLabel.length * (fontSize * 0.6), 80);
      const labelHeight = 20;
      const padding = 6;

      elements.push(
        createRoundedRect(
          midPoint.x - labelWidth / 2 - padding,
          midPoint.y - labelHeight / 2,
          labelWidth + padding * 2,
          labelHeight,
          3,
          {
            fill: '#FFFFFF',
            stroke: itemStyle.color,
            'stroke-width': 1,
          }
        )
      );

      elements.push(
        createText(midPoint.x, midPoint.y, fullLabel, {
          'text-anchor': 'middle',
          'dominant-baseline': 'middle',
          'font-family': config.fontFamily,
          'font-size': fontSize,
          fill: itemStyle.color,
          'font-weight': itemStyle.labelStyle.fontWeight as any,
          'font-style': itemStyle.labelStyle.fontStyle as any,
        })
      );
    }
  }

  return createGroup(elements, { id: `edge-${edge.id}` });
}

function renderTitle(title: string, config: RenderConfig): SvgElement {
  return createText(20, 30, title, {
    'font-family': config.fontFamily,
    'font-size': config.fontSize! + 6,
    'font-weight': 'bold',
    fill: '#212529',
  });
}
