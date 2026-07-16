/**
 * Functional Chain Diagram Renderer
 * 
 * Renders functional chain diagrams showing execution scenarios:
 * - Sequence of function invocations
 * - Data flow through the chain
 * - Timing and ordering constraints
 * - Similar to sequence diagrams but focused on functions
 */

import {
  SystemFunction,
  FunctionalExchange,
} from '../types/model';
import {
  DiagramNode,
  DiagramEdge,
  Point,
  Size,
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
  createCircle,
} from '../utils/svg';

const DEFAULT_CONFIG: RenderConfig = {
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
  fontSize: 12,
};

export interface FunctionalChainModel {
  name: string;
  functions: SystemFunction[];
  functional_exchanges: FunctionalExchange[];
  execution_order?: string[];
}

export async function renderFunctionalChainDiagram(
  model: FunctionalChainModel,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg: RenderConfig = { ...DEFAULT_CONFIG, ...config };

  const { nodes, edges } = convertToDiagram(model);

  if (nodes.length === 0) {
    return {
      svg: createEmptyDiagram(cfg),
      width: 400,
      height: 200,
      metadata: {
        diagramType: 'functional-chain',
        functionCount: 0,
      },
    };
  }

  const layout = await applyHierarchicalLayout(nodes, edges, {
    direction: 'RIGHT',
    nodeSpacing: 100,
    layerSpacing: 180,
  });

  const svg = renderToSvg(model, layout, cfg);

  return {
    svg,
    width: layout.totalSize.width,
    height: layout.totalSize.height,
    metadata: {
      diagramType: 'functional-chain',
      chainName: model.name,
      functionCount: nodes.length,
      exchangeCount: edges.length,
    },
  };
}

function convertToDiagram(model: FunctionalChainModel): {
  nodes: DiagramNode[];
  edges: DiagramEdge[];
} {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  const functionMap = new Map<string, SystemFunction>();
  
  for (const func of model.functions) {
    functionMap.set(func.id, func);
    
    nodes.push({
      id: func.id,
      label: func.name,
      type: 'function',
      metadata: {
        category: func.category,
        color: func.color,
        icon: func.icon,
        ports: func.ports,
      },
    });
  }

  console.log(`[Functional Chain Renderer] Processing ${model.functional_exchanges.length} exchanges`);
  
  for (const exchange of model.functional_exchanges) {
    console.log(`[Functional Chain Renderer] Exchange:`, exchange);
    const [sourceFunc, sourcePortName] = exchange.from_port.split('.');
    const [targetFunc, targetPortName] = exchange.to_port.split('.');
    
    console.log(`[Functional Chain Renderer] Parsed: ${sourceFunc}.${sourcePortName} -> ${targetFunc}.${targetPortName}`);

    if (sourceFunc && targetFunc) {
      const edge: DiagramEdge = {
        id: `${sourceFunc}-${targetFunc}`,
        from: sourceFunc,
        to: targetFunc,
        label: exchange.label || exchange.data_type,
        type: 'functional-exchange',
        metadata: {
          dataType: exchange.data_type,
          fromPort: sourcePortName || 'out1',
          toPort: targetPortName || 'in1',
        },
      };
      edges.push(edge);
      console.log(`[Functional Chain Renderer] Created edge:`, edge);
    }
  }

  console.log(`[Functional Chain Renderer] Total nodes: ${nodes.length}, Total edges: ${edges.length}`);
  return { nodes, edges };
}

function renderToSvg(
  model: FunctionalChainModel,
  layout: any,
  config: RenderConfig
): string {
  const elements: SvgElement[] = [];

  elements.push(
    createRect(0, 0, layout.totalSize.width, layout.totalSize.height, {
      fill: config.colorScheme?.background || '#FFFFFF',
      stroke: 'none',
    })
  );

  for (const edge of layout.edges) {
    elements.push(renderFunctionalExchange(edge, config));
  }

  for (const node of layout.nodes) {
    const originalFunc = model.functions.find(f => f.id === node.id);
    if (originalFunc) {
      elements.push(renderFunctionNode(node, originalFunc, config));
    }
  }

  elements.push(
    createText(20, 30, `Functional Chain: ${model.name}`, {
      'font-family': 'Arial, sans-serif',
      'font-size': 18,
      'font-weight': 'bold',
      fill: '#212529',
    })
  );

  elements.push(
    createText(20, 55, '→ Data Flow Direction', {
      'font-family': 'Arial, sans-serif',
      'font-size': 10,
      fill: '#666666',
    })
  );

  const defs = [
    createArrowMarker('arrow-blue', '#0066CC', 12),
    createArrowMarker('arrow-green', '#28a745', 12),
  ];

  return createSvgDocument(layout.totalSize.width, layout.totalSize.height, elements, defs);
}

function renderFunctionNode(node: any, func: SystemFunction, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  const nodeColor = getFunctionColor(func.category);

  elements.push(
    createRoundedRect(
      node.position.x,
      node.position.y,
      node.size.width,
      node.size.height,
      10,
      {
        fill: nodeColor,
        stroke: '#2c5282',
        'stroke-width': 2.5,
      }
    )
  );

  if (func.icon) {
    elements.push(
      createText(node.position.x + 15, node.position.y + 30, func.icon, {
        'font-family': 'Arial, sans-serif',
        'font-size': 20,
        fill: '#000000',
      })
    );
  }

  elements.push(
    createText(
      node.position.x + (func.icon ? 45 : node.size.width / 2),
      node.position.y + 35,
      func.name,
      {
        'text-anchor': func.icon ? 'start' : 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 12,
        'font-weight': 'bold',
        fill: '#000000',
      }
    )
  );

  if (func.category) {
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + 55,
        `[${func.category}]`,
        {
          'text-anchor': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 9,
          'font-style': 'italic',
          fill: '#555555',
        }
      )
    );
  }

  if (func.ports && func.ports.length > 0) {
    const inputPorts = func.ports.filter(p => p.direction === 'In' || p.direction === 'InOut');
    const outputPorts = func.ports.filter(p => p.direction === 'Out' || p.direction === 'InOut');

    for (let i = 0; i < inputPorts.length; i++) {
      const port = inputPorts[i];
      const portY = node.position.y + 20 + i * 15;
      
      elements.push(
        createCircle(node.position.x, portY, 4, {
          fill: port.port_type === 'Control' ? '#ff6b6b' : '#4dabf7',
          stroke: '#000000',
          'stroke-width': 1,
        })
      );
    }

    for (let i = 0; i < outputPorts.length; i++) {
      const port = outputPorts[i];
      const portY = node.position.y + 20 + i * 15;
      
      elements.push(
        createCircle(node.position.x + node.size.width, portY, 4, {
          fill: port.port_type === 'Control' ? '#ff6b6b' : '#51cf66',
          stroke: '#000000',
          'stroke-width': 1,
        })
      );
    }
  }

  return createGroup(elements, { id: `function-${node.id}` });
}

function renderFunctionalExchange(edge: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  if (!edge.points || edge.points.length < 2) {
    return createGroup([]);
  }

  const pathD = generateConnectionPath(edge.points);

  elements.push(
    createPath(pathD, {
      fill: 'none',
      stroke: '#0066CC',
      'stroke-width': 2.5,
      'marker-end': 'url(#arrow-blue)',
    })
  );

  if (edge.label && config.showLabels) {
    const midPoint = edge.points[Math.floor(edge.points.length / 2)];
    
    // Improved label background sizing
    const fontSize = (config.fontSize || 12) - 1;
    const labelWidth = Math.max(edge.label.length * (fontSize * 0.6), 80);
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
  }

  if (edge.metadata?.dataType && config.showLabels) {
    const midPoint = edge.points[Math.floor(edge.points.length / 2)];
    
    elements.push(
      createText(midPoint.x, midPoint.y + 15, `{${edge.metadata.dataType}}`, {
        'text-anchor': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 8,
        'font-style': 'italic',
        fill: '#666666',
      })
    );
  }

  return createGroup(elements, { id: edge.id });
}

function generateConnectionPath(points: Point[]): string {
  if (points.length === 0) return '';
  if (points.length === 1) return `M ${points[0].x} ${points[0].y}`;

  let path = `M ${points[0].x} ${points[0].y}`;

  for (let i = 1; i < points.length; i++) {
    const prev = points[i - 1];
    const curr = points[i];
    
    if (Math.abs(curr.x - prev.x) > Math.abs(curr.y - prev.y)) {
      const midX = (prev.x + curr.x) / 2;
      path += ` L ${midX} ${prev.y} L ${midX} ${curr.y}`;
    }
    
    path += ` L ${curr.x} ${curr.y}`;
  }

  return path;
}

function getFunctionColor(category: string): string {
  const colors: Record<string, string> = {
    Environmental: '#C8E6C9',
    System: '#BBDEFB',
    Management: '#FFE0B2',
    Control: '#F8BBD0',
    Interaction: '#E1BEE7',
  };

  return colors[category] || '#E0E0E0';
}

function createEmptyDiagram(config: RenderConfig): string {
  const elements: SvgElement[] = [
    createRect(0, 0, 400, 200, {
      fill: config.colorScheme?.background || '#FFFFFF',
      stroke: 'none',
    }),
    createText(200, 100, 'No functional chain found', {
      'text-anchor': 'middle',
      'dominant-baseline': 'middle',
      'font-family': 'Arial, sans-serif',
      'font-size': 14,
      fill: '#666666',
    }),
  ];

  return createSvgDocument(400, 200, elements, []);
}
