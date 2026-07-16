/**
 * Breakdown Tree Diagram Renderer
 * 
 * Renders hierarchical breakdown diagrams for Capella/Arcadia:
 * - OEBD (Operational Entity Breakdown Diagram)
 * - SFBD (System Functional Breakdown Diagram)
 * - LFBD (Logical Functional Breakdown Diagram)
 * - LCBD (Logical Component Breakdown Diagram)
 * - PFBD (Physical Functional Breakdown Diagram)
 * - PCBD (Physical Component Breakdown Diagram)
 * 
 * Uses Reingold-Tilford tree layout algorithm (LaTeX spec page 24)
 */

import {
  DiagramNode,
  DiagramEdge,
  DiagramOutput,
  RenderConfig,
  CAPELLA_COLORS,
  SvgElement,
  Point,
  Size,
} from '../types/diagram';
import { applyTreeLayout } from '../layouts/tree';
import {
  createSvgDocument,
  createRoundedRect,
  createText,
  createGroup,
  createPath,
  createArrowMarker,
} from '../utils/svg';
import {
  getSafetyBorderAttributes,
  parseSafetyLevel,
} from '../utils/safety-colors';

export type BreakdownType = 'OEBD' | 'SFBD' | 'LFBD' | 'LCBD' | 'PFBD' | 'PCBD';

const DEFAULT_CONFIG: RenderConfig = {
  width: 1400,
  height: 1000,
  backgroundColor: '#FFFFFF',
  fontSize: 12,
  fontFamily: 'Arial, sans-serif',
  showGrid: false,
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
};

const BREAKDOWN_COLORS: Record<BreakdownType, string> = {
  OEBD: '#FFFF99',
  SFBD: '#ADD8E6',
  LFBD: '#4682B4',
  LCBD: '#6495ED',
  PFBD: '#4169E1',
  PCBD: '#FFD700',
};

interface BreakdownNode {
  id: string;
  label: string;
  level: number;
  children: BreakdownNode[];
  metadata?: any;
}

/**
 * Render breakdown tree diagram
 */
export async function renderBreakdownTree(
  rootNode: BreakdownNode,
  breakdownType: BreakdownType,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  const { nodes, edges } = convertTreeToDiagram(rootNode, breakdownType);

  const layout = applyTreeLayout(nodes, edges, {
    direction: 'DOWN',
    nodeSpacing: 100,
    layerSpacing: 150,
  });

  const svg = renderToSvg(layout, breakdownType, cfg);

  return {
    svg,
    width: layout.totalSize.width,
    height: layout.totalSize.height,
    metadata: {
      diagramType: breakdownType,
      nodeCount: nodes.length,
      maxDepth: calculateMaxDepth(rootNode),
    },
  };
}

function convertTreeToDiagram(
  root: BreakdownNode,
  breakdownType: BreakdownType
): { nodes: DiagramNode[]; edges: DiagramEdge[] } {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];
  const color = BREAKDOWN_COLORS[breakdownType];

  function traverse(node: BreakdownNode, level: number) {
    nodes.push({
      id: node.id,
      label: node.label,
      type: 'tree-node',
      color,
      metadata: {
        level,
        ...node.metadata,
      },
    });

    for (const child of node.children) {
      traverse(child, level + 1);
      edges.push({
        id: `${node.id}-${child.id}`,
        from: node.id,
        to: child.id,
        type: 'hierarchy',
      });
    }
  }

  traverse(root, 0);

  return { nodes, edges };
}

function renderToSvg(
  layout: any,
  breakdownType: BreakdownType,
  config: RenderConfig
): string {
  const elements: SvgElement[] = [];
  const defs: SvgElement[] = [];

  defs.push(createArrowMarker('arrow-hierarchy', '#333333'));

  elements.push(
    createRoundedRect(0, 0, layout.totalSize.width, layout.totalSize.height, 0, {
      fill: config.backgroundColor,
      stroke: 'none',
    })
  );

  for (const edge of layout.edges) {
    elements.push(renderTreeEdge(edge, config));
  }

  for (const node of layout.nodes) {
    elements.push(renderTreeNode(node, breakdownType, config));
  }

  elements.push(renderTitle(breakdownType, config));

  return createSvgDocument(
    layout.totalSize.width,
    layout.totalSize.height,
    elements,
    defs
  );
}

function renderTreeNode(
  node: any,
  breakdownType: BreakdownType,
  config: RenderConfig
): SvgElement {
  const elements: SvgElement[] = [];
  const level = node.metadata?.level || 0;

  const width = 180 - level * 10;
  const height = 80;

  const x = node.position.x - width / 2;
  const y = node.position.y - height / 2;

  const safetyData = parseSafetyLevel(node.metadata);
  const safetyLevel = safetyData.level;
  const safetyStandard = safetyData.standard;
  const hasSafetyLevel = safetyLevel !== null;

  const nodeAttrs: Record<string, string> = {
    fill: node.color,
    'filter': 'drop-shadow(0 2px 4px rgba(0,0,0,0.1))',
  };

  if (hasSafetyLevel && safetyLevel) {
    const safetyAttrs = getSafetyBorderAttributes(safetyLevel, safetyStandard || undefined);
    Object.assign(nodeAttrs, safetyAttrs);
  } else {
    nodeAttrs.stroke = '#333333';
    nodeAttrs['stroke-width'] = level === 0 ? '3' : '2';
  }

  elements.push(
    createRoundedRect(x, y, width, height, 8, nodeAttrs)
  );

  const lines = wrapText(node.label, width - 20);
  const textY = y + height / 2 - (lines.length * 8);

  for (let i = 0; i < lines.length; i++) {
    elements.push(
      createText(
        node.position.x,
        textY + i * 16,
        lines[i],
        {
          'text-anchor': 'middle',
          'dominant-baseline': 'middle',
          'font-family': config.fontFamily,
          'font-size': level === 0 ? 14 : 12,
          'font-weight': level === 0 ? 'bold' : 'normal',
          fill: '#000000',
        }
      )
    );
  }

  if (node.children && node.children.length > 0) {
    elements.push(
      createText(
        x + width - 10,
        y + height - 10,
        `(${node.children.length})`,
        {
          'text-anchor': 'end',
          'font-family': config.fontFamily,
          'font-size': 10,
          fill: '#666666',
        }
      )
    );
  }

  return createGroup(elements, { id: `node-${node.id}` });
}

function renderTreeEdge(edge: any, config: RenderConfig): SvgElement {
  if (!edge.points || edge.points.length < 2) {
    return createGroup([]);
  }

  const pathD = edge.points.map((p: Point, i: number) =>
    i === 0 ? `M ${p.x} ${p.y}` : `L ${p.x} ${p.y}`
  ).join(' ');

  return createPath(pathD, {
    fill: 'none',
    stroke: '#333333',
    'stroke-width': 2,
    'marker-end': 'url(#arrow-hierarchy)',
  });
}

function renderTitle(breakdownType: BreakdownType, config: RenderConfig): SvgElement {
  const titles: Record<BreakdownType, string> = {
    OEBD: 'Operational Entity Breakdown Diagram',
    SFBD: 'System Functional Breakdown Diagram',
    LFBD: 'Logical Functional Breakdown Diagram',
    LCBD: 'Logical Component Breakdown Diagram',
    PFBD: 'Physical Functional Breakdown Diagram',
    PCBD: 'Physical Component Breakdown Diagram',
  };

  return createText(20, 30, titles[breakdownType], {
    'font-family': config.fontFamily,
    'font-size': config.fontSize! + 6,
    'font-weight': 'bold',
    fill: '#212529',
  });
}

function calculateMaxDepth(node: BreakdownNode): number {
  if (!node.children || node.children.length === 0) {
    return 1;
  }

  return 1 + Math.max(...node.children.map(calculateMaxDepth));
}

function wrapText(text: string, maxWidth: number): string[] {
  const words = text.split(' ');
  const lines: string[] = [];
  let currentLine = '';

  for (const word of words) {
    const testLine = currentLine ? `${currentLine} ${word}` : word;
    const approxWidth = testLine.length * 7;

    if (approxWidth <= maxWidth) {
      currentLine = testLine;
    } else {
      if (currentLine) {
        lines.push(currentLine);
      }
      currentLine = word;
    }
  }

  if (currentLine) {
    lines.push(currentLine);
  }

  return lines;
}
