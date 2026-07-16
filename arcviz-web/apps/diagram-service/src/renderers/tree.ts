/**
 * Tree Diagram Renderer
 * 
 * Renders hierarchical tree diagrams for function/component breakdowns.
 * Supports both function hierarchies and component hierarchies with
 * collapsible/expandable nodes (visual indicators).
 */

import {
  SystemFunction,
  LogicalComponent,
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
  applyTreeLayout,
  TreeLayoutOptions,
} from '../layouts/tree';
import {
  createSvgDocument,
  createRect,
  createLine,
  createText,
  createGroup,
  createPath,
  createCircle,
  createRoundedRect,
} from '../utils/svg';

const DEFAULT_CONFIG: RenderConfig = {
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
  fontSize: 12,
};

export interface TreeDiagramModel {
  functions?: SystemFunction[];
  components?: LogicalComponent[];
  tree_type: 'function' | 'component';
}

export async function renderTreeDiagram(
  model: TreeDiagramModel,
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
        diagramType: 'tree',
        nodeCount: 0,
      },
    };
  }

  const layoutOptions: TreeLayoutOptions = {
    direction: 'DOWN',
    levelSpacing: 140,
    nodeSpacing: 60,
    rootX: 100,
    rootY: 100,
  };

  const layout = await applyTreeLayout(nodes, edges, layoutOptions);

  const svg = renderToSvg(model, layout, cfg);

  return {
    svg,
    width: layout.totalSize.width,
    height: layout.totalSize.height,
    metadata: {
      diagramType: 'tree',
      treeType: model.tree_type,
      nodeCount: nodes.length,
    },
  };
}

function convertToDiagram(model: TreeDiagramModel): {
  nodes: DiagramNode[];
  edges: DiagramEdge[];
} {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  if (model.tree_type === 'function' && model.functions) {
    for (const func of model.functions) {
      processFunction(func, null, nodes, edges);
    }
  } else if (model.tree_type === 'component' && model.components) {
    for (const comp of model.components) {
      processComponent(comp, null, nodes, edges);
    }
  }

  return { nodes, edges };
}

function processFunction(
  func: SystemFunction,
  parentId: string | null,
  nodes: DiagramNode[],
  edges: DiagramEdge[]
): void {
  const hasChildren = func.sub_functions && func.sub_functions.length > 0;

  nodes.push({
    id: func.id,
    label: func.name,
    type: 'component',
    metadata: {
      nodeType: 'function',
      category: func.category,
      color: func.color,
      icon: func.icon,
      hasChildren,
      expanded: true,
      ports: func.ports,
    },
  });

  if (parentId) {
    edges.push({
      id: `${parentId}-${func.id}`,
      from: parentId,
      to: func.id,
      type: 'hierarchy',
    });
  }

  if (hasChildren) {
    for (const subFunc of func.sub_functions) {
      processFunction(subFunc, func.id, nodes, edges);
    }
  }
}

function processComponent(
  comp: LogicalComponent,
  parentId: string | null,
  nodes: DiagramNode[],
  edges: DiagramEdge[]
): void {
  const hasChildren = comp.sub_components && comp.sub_components.length > 0;

  nodes.push({
    id: comp.id,
    label: comp.name,
    type: 'component',
    metadata: {
      nodeType: 'component',
      componentType: comp.component_type,
      color: comp.color,
      hasChildren,
      expanded: true,
      allocatedFunctions: comp.allocated_functions,
      ports: comp.ports,
    },
  });

  if (parentId) {
    edges.push({
      id: `${parentId}--${comp.id}`,
      from: parentId,
      to: comp.id,
      type: 'hierarchy',
    });
  }

  if (hasChildren) {
    for (const subComp of comp.sub_components) {
      processComponent(subComp, comp.id, nodes, edges);
    }
  }
}

function renderToSvg(
  model: TreeDiagramModel,
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
    elements.push(renderHierarchyEdge(edge, config));
  }

  for (const node of layout.nodes) {
    if (node.metadata?.nodeType === 'function') {
      elements.push(renderFunctionNode(node, config));
    } else if (node.metadata?.nodeType === 'component') {
      elements.push(renderComponentNode(node, config));
    }
  }

  const title = model.tree_type === 'function' ? 'Function Breakdown' : 'Component Breakdown';
  elements.push(
    createText(20, 30, title, {
      'font-family': 'Arial, sans-serif',
      'font-size': 18,
      'font-weight': 'bold',
      fill: '#212529',
    })
  );

  elements.push(
    createText(20, 55, '⊞ = Expandable  ⊟ = Expanded', {
      'font-family': 'Arial, sans-serif',
      'font-size': 10,
      fill: '#666666',
    })
  );

  return createSvgDocument(layout.totalSize.width, layout.totalSize.height, elements, []);
}

function renderFunctionNode(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  const nodeColor = getCategoryColor(node.metadata?.category || 'System');
  const hasChildren = node.metadata?.hasChildren || false;
  const expanded = node.metadata?.expanded || false;

  elements.push(
    createRoundedRect(
      node.position.x,
      node.position.y,
      node.size.width,
      node.size.height,
      10,
      {
        fill: nodeColor,
        stroke: '#333333',
        'stroke-width': 2,
      }
    )
  );

  if (node.metadata?.icon) {
    elements.push(
      createText(node.position.x + 10, node.position.y + 25, node.metadata.icon, {
        'font-family': 'Arial, sans-serif',
        'font-size': 16,
        fill: '#000000',
      })
    );
  }

  elements.push(
    createText(
      node.position.x + (node.metadata?.icon ? 35 : node.size.width / 2),
      node.position.y + 30,
      node.label,
      {
        'text-anchor': node.metadata?.icon ? 'start' : 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 12,
        'font-weight': 'bold',
        fill: '#000000',
      }
    )
  );

  if (node.metadata?.category) {
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + 50,
        `[${node.metadata.category}]`,
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

  if (hasChildren) {
    const expandX = node.position.x + node.size.width - 20;
    const expandY = node.position.y + 10;

    elements.push(
      createRect(expandX, expandY, 16, 16, {
        fill: '#FFFFFF',
        stroke: '#333333',
        'stroke-width': 1.5,
      })
    );

    const symbol = expanded ? '⊟' : '⊞';
    elements.push(
      createText(expandX + 8, expandY + 12, symbol, {
        'text-anchor': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 12,
        'font-weight': 'bold',
        fill: '#333333',
      })
    );
  }

  return createGroup(elements, { id: `function-${node.id}` });
}

function renderComponentNode(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  const nodeColor = getComponentColor(node.metadata?.componentType || 'Logical');
  const hasChildren = node.metadata?.hasChildren || false;
  const expanded = node.metadata?.expanded || false;

  elements.push(
    createRect(node.position.x, node.position.y, node.size.width, node.size.height, {
      fill: nodeColor,
      stroke: '#000000',
      'stroke-width': 2.5,
    })
  );

  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 30,
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

  if (node.metadata?.componentType) {
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + 50,
        `«${node.metadata.componentType}»`,
        {
          'text-anchor': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 9,
          'font-style': 'italic',
          fill: '#666666',
        }
      )
    );
  }

  if (node.metadata?.allocatedFunctions && node.metadata.allocatedFunctions.length > 0) {
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + 68,
        `Functions: ${node.metadata.allocatedFunctions.length}`,
        {
          'text-anchor': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 8,
          fill: '#555555',
        }
      )
    );
  }

  if (hasChildren) {
    const expandX = node.position.x + node.size.width - 20;
    const expandY = node.position.y + 10;

    elements.push(
      createRect(expandX, expandY, 16, 16, {
        fill: '#FFFFFF',
        stroke: '#000000',
        'stroke-width': 1.5,
      })
    );

    const symbol = expanded ? '⊟' : '⊞';
    elements.push(
      createText(expandX + 8, expandY + 12, symbol, {
        'text-anchor': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 12,
        'font-weight': 'bold',
        fill: '#000000',
      })
    );
  }

  return createGroup(elements, { id: `component-${node.id}` });
}

function renderHierarchyEdge(edge: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  if (!edge.points || edge.points.length < 2) {
    return createGroup([]);
  }

  const start = edge.points[0];
  const end = edge.points[edge.points.length - 1];

  const midY = (start.y + end.y) / 2;

  const pathD = `M ${start.x} ${start.y} L ${start.x} ${midY} L ${end.x} ${midY} L ${end.x} ${end.y}`;

  elements.push(
    createPath(pathD, {
      fill: 'none',
      stroke: '#666666',
      'stroke-width': 2,
    })
  );

  return createGroup(elements, { id: edge.id });
}

function getCategoryColor(category: string): string {
  const colors: Record<string, string> = {
    Environmental: '#C8E6C9',
    System: '#BBDEFB',
    Management: '#FFE0B2',
    Control: '#F8BBD0',
    Interaction: '#E1BEE7',
  };

  return colors[category] || '#E0E0E0';
}

function getComponentColor(componentType: string): string {
  const colors: Record<string, string> = {
    Logical: '#FFE6CC',
    Behavioral: '#E1F5FF',
    Hardware: '#F0F0F0',
  };

  return colors[componentType] || '#FFFFFF';
}

function createEmptyDiagram(config: RenderConfig): string {
  const elements: SvgElement[] = [
    createRect(0, 0, 400, 200, {
      fill: config.colorScheme?.background || '#FFFFFF',
      stroke: 'none',
    }),
    createText(200, 100, 'No functions or components found', {
      'text-anchor': 'middle',
      'dominant-baseline': 'middle',
      'font-family': 'Arial, sans-serif',
      'font-size': 14,
      fill: '#666666',
    }),
  ];

  return createSvgDocument(400, 200, elements, []);
}
