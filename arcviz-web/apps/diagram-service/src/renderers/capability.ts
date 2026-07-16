/**
 * Capability Diagram Renderer
 * 
 * Renders capability decomposition diagrams showing:
 * - Missions, Capabilities, and Sub-Capabilities in hierarchy
 * - Capability associations (includes, extends, generalizes)
 * - Links to scenarios, functional chains, and activities
 */

import {
  OperationalCapability,
  CapabilityAssociation,
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
} from '../utils/svg';

const DEFAULT_CONFIG: RenderConfig = {
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
  fontSize: 12,
};

export interface CapabilityDiagramModel {
  capabilities: OperationalCapability[];
  capability_associations?: CapabilityAssociation[];
}

export async function renderCapabilityDiagram(
  model: CapabilityDiagramModel,
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
        diagramType: 'capability',
        capabilityCount: 0,
      },
    };
  }

  const layout = await applyHierarchicalLayout(nodes, edges, {
    direction: 'DOWN',
    nodeSpacing: 100,
    layerSpacing: 140,
  });

  const svg = renderToSvg(model, layout, cfg);

  return {
    svg,
    width: layout.totalSize.width,
    height: layout.totalSize.height,
    metadata: {
      diagramType: 'capability',
      capabilityCount: nodes.length,
      associationCount: edges.length,
    },
  };
}

function convertToDiagram(model: CapabilityDiagramModel): {
  nodes: DiagramNode[];
  edges: DiagramEdge[];
} {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  for (const capability of model.capabilities) {
    processCapability(capability, null, nodes, edges);
  }

  if (model.capability_associations) {
    for (const assoc of model.capability_associations) {
      edges.push({
        id: `${assoc.from}-${assoc.to}`,
        from: assoc.from,
        to: assoc.to,
        label: assoc.label || undefined,
        type: 'association',
        metadata: {
          associationType: assoc.association_type,
        },
      });
    }
  }

  return { nodes, edges };
}

function processCapability(
  capability: OperationalCapability,
  parentId: string | null,
  nodes: DiagramNode[],
  edges: DiagramEdge[]
): void {
  nodes.push({
    id: capability.id,
    label: capability.name,
    type: 'capability',
    metadata: {
      level: capability.level,
      color: capability.color,
      stereotype: capability.stereotype,
      hasChildren: capability.children && capability.children.length > 0,
    },
  });

  if (parentId) {
    edges.push({
      id: `${parentId}-contains-${capability.id}`,
      from: parentId,
      to: capability.id,
      type: 'hierarchy',
      metadata: {
        relationship: 'contains',
      },
    });
  }

  if (capability.children && capability.children.length > 0) {
    for (const child of capability.children) {
      processCapability(child, capability.id, nodes, edges);
    }
  }
}

function renderToSvg(
  model: CapabilityDiagramModel,
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
    elements.push(renderCapabilityEdge(edge, config));
  }

  for (const node of layout.nodes) {
    const originalCap = findCapability(model.capabilities, node.id);
    if (originalCap) {
      elements.push(renderCapabilityNode(node, originalCap, config));
    }
  }

  elements.push(
    createText(20, 30, 'Capability Diagram', {
      'font-family': 'Arial, sans-serif',
      'font-size': 18,
      'font-weight': 'bold',
      fill: '#212529',
    })
  );

  const defs = [
    createArrowMarker('arrow-black', '#000000', 10),
    createArrowMarker('arrow-dashed', '#666666', 8),
  ];

  defs.push({
    type: 'marker',
    attributes: {
      id: 'diamond',
      viewBox: '0 0 10 10',
      refX: '5',
      refY: '5',
      markerWidth: 10,
      markerHeight: 10,
      orient: 'auto-start-reverse',
    },
    children: [
      {
        type: 'path',
        attributes: {
          d: 'M 0 5 L 5 0 L 10 5 L 5 10 Z',
          fill: '#FFFFFF',
          stroke: '#000000',
          'stroke-width': 1.5,
        },
      },
    ],
  });

  return createSvgDocument(layout.totalSize.width, layout.totalSize.height, elements, defs);
}

function renderCapabilityNode(node: any, capability: OperationalCapability, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  const nodeColor = getLevelColor(capability.level);
  const borderWidth = capability.level === 'Mission' ? 3 : 2;

  elements.push(
    createRoundedRect(
      node.position.x,
      node.position.y,
      node.size.width,
      node.size.height,
      12,
      {
        fill: capability.color || nodeColor,
        stroke: '#000000',
        'stroke-width': borderWidth,
      }
    )
  );

  const levelLabel = `[${capability.level}]`;
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 18,
      levelLabel,
      {
        'text-anchor': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 9,
        'font-style': 'italic',
        'font-weight': 'bold',
        fill: '#555555',
      }
    )
  );

  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 40,
      capability.name,
      {
        'text-anchor': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 13,
        'font-weight': 'bold',
        fill: '#000000',
      }
    )
  );

  if (capability.stereotype) {
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + 60,
        `«${capability.stereotype}»`,
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

  return createGroup(elements, { id: `capability-${node.id}` });
}

function renderCapabilityEdge(edge: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  if (!edge.points || edge.points.length < 2) {
    return createGroup([]);
  }

  const isHierarchy = edge.metadata?.relationship === 'contains';
  const assocType = edge.metadata?.associationType;

  const pathD = generateConnectionPath(edge.points);

  let strokeStyle: Record<string, any> = {
    fill: 'none',
    stroke: isHierarchy ? '#000000' : '#666666',
    'stroke-width': isHierarchy ? 2 : 1.5,
  };

  if (assocType === 'includes' || assocType === 'extends') {
    strokeStyle['stroke-dasharray'] = '5,5';
    strokeStyle['marker-end'] = 'url(#arrow-dashed)';
  } else if (assocType === 'generalizes') {
    strokeStyle['marker-end'] = 'url(#diamond)';
  } else if (isHierarchy) {
    strokeStyle['marker-end'] = 'url(#arrow-black)';
  }

  elements.push(createPath(pathD, strokeStyle));

  if (edge.label && config.showLabels && !isHierarchy) {
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

  if (assocType && config.showLabels) {
    const midPoint = edge.points[Math.floor(edge.points.length / 2)];
    
    elements.push(
      createText(midPoint.x, midPoint.y + 15, `«${assocType}»`, {
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
    path += ` L ${points[i].x} ${points[i].y}`;
  }

  return path;
}

function getLevelColor(level: string): string {
  const colors: Record<string, string> = {
    Mission: '#FFD699',
    Capability: '#BBDEFB',
    SubCapability: '#C8E6C9',
  };

  return colors[level] || '#E0E0E0';
}

function findCapability(capabilities: OperationalCapability[], id: string): OperationalCapability | null {
  for (const cap of capabilities) {
    if (cap.id === id) return cap;

    if (cap.children && cap.children.length > 0) {
      const found = findCapability(cap.children, id);
      if (found) return found;
    }
  }

  return null;
}

function createEmptyDiagram(config: RenderConfig): string {
  const elements: SvgElement[] = [
    createRect(0, 0, 400, 200, {
      fill: config.colorScheme?.background || '#FFFFFF',
      stroke: 'none',
    }),
    createText(200, 100, 'No capabilities found', {
      'text-anchor': 'middle',
      'dominant-baseline': 'middle',
      'font-family': 'Arial, sans-serif',
      'font-size': 14,
      fill: '#666666',
    }),
  ];

  return createSvgDocument(400, 200, elements, []);
}
