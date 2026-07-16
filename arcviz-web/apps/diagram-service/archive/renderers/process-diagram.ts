/**
 * Operational Process Diagram (OPD) Renderer
 * 
 * Implements LaTeX Specification Section 2 (Operational Analysis Diagram Types)
 * OPD: Visualization of operational processes with BPMN-like notation
 * 
 * Shows:
 * - Operational processes
 * - Activity sequences
 * - Process flow (sequential, parallel, conditional)
 * - Pre/post conditions
 * - Decision points
 */

import {
  DiagramNode,
  DiagramEdge,
  DiagramOutput,
  RenderConfig,
  CAPELLA_COLORS,
  SvgElement,
  Point,
} from '../types/diagram';
import { OperationalProcess, OperationalActivity } from '../types/model';
import { applyHierarchicalLayout } from '../layouts/hierarchical';
import {
  createSvgDocument,
  createRoundedRect,
  createText,
  createGroup,
  createPath,
  createArrowMarker,
  createCircle,
} from '../utils/svg';
import {
  getSafetyBorderAttributes,
  parseSafetyLevel,
} from '../utils/safety-colors';

const DEFAULT_CONFIG: RenderConfig = {
  width: 1600,
  height: 1000,
  backgroundColor: '#FFFFFF',
  fontSize: 12,
  fontFamily: 'Arial, sans-serif',
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
};

export interface ProcessModel {
  processes: OperationalProcess[];
  activities: OperationalActivity[];
  flows: Array<{
    from: string;
    to: string;
    condition?: string;
    flow_type: 'sequence' | 'conditional' | 'parallel';
  }>;
}

/**
 * Render Operational Process Diagram
 */
export async function renderProcessDiagram(
  model: ProcessModel,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  const { nodes, edges } = convertToDiagram(model);

  const layout = await applyHierarchicalLayout(nodes, edges, {
    direction: 'RIGHT',
    nodeSpacing: 80,
    layerSpacing: 150,
  });

  const svg = renderToSvg(layout, cfg);

  return {
    svg,
    width: layout.totalSize.width,
    height: layout.totalSize.height,
    metadata: {
      diagramType: 'OPD',
      processCount: model.processes.length,
      activityCount: model.activities.length,
    },
  };
}

function convertToDiagram(
  model: ProcessModel
): { nodes: DiagramNode[]; edges: DiagramEdge[] } {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  nodes.push({
    id: 'start',
    label: 'Start',
    type: 'start-event',
    color: '#90EE90',
  });

  for (const process of model.processes) {
    nodes.push({
      id: process.id,
      label: process.name,
      type: 'process',
      color: '#FFB266',
      metadata: {
        description: process.description,
        pre_condition: process.pre_condition,
        post_condition: process.post_condition,
        activities: process.activities,
      },
    });
  }

  for (const activity of model.activities) {
    nodes.push({
      id: activity.id,
      label: activity.name,
      type: 'activity',
      color: '#FFD966',
      metadata: {
        category: activity.category,
        performed_by: activity.performed_by,
      },
    });
  }

  nodes.push({
    id: 'end',
    label: 'End',
    type: 'end-event',
    color: '#FFB6C1',
  });

  for (const flow of model.flows) {
    edges.push({
      id: `${flow.from}-${flow.to}`,
      from: flow.from,
      to: flow.to,
      type: flow.flow_type,
      label: flow.condition,
    });
  }

  return { nodes, edges };
}

function renderToSvg(layout: any, config: RenderConfig): string {
  const elements: SvgElement[] = [];
  const defs: SvgElement[] = [];

  defs.push(createArrowMarker('arrow-flow', '#000000'));
  defs.push(createArrowMarker('arrow-conditional', '#FF6600'));

  elements.push(
    createRoundedRect(0, 0, layout.totalSize.width, layout.totalSize.height, 0, {
      fill: config.backgroundColor,
      stroke: 'none',
    })
  );

  for (const edge of layout.edges) {
    elements.push(renderFlowEdge(edge, config));
  }

  for (const node of layout.nodes) {
    elements.push(renderProcessNode(node, config));
  }

  elements.push(
    createText(20, 30, 'Operational Process Diagram (OPD)', {
      'font-family': config.fontFamily,
      'font-size': config.fontSize! + 6,
      'font-weight': 'bold',
      fill: '#212529',
    })
  );

  return createSvgDocument(
    layout.totalSize.width,
    layout.totalSize.height,
    elements,
    defs
  );
}

function renderProcessNode(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  if (node.type === 'start-event' || node.type === 'end-event') {
    const radius = 30;
    elements.push(
      createCircle(node.position.x + radius, node.position.y + radius, radius, {
        fill: node.color,
        stroke: '#000000',
        'stroke-width': node.type === 'end-event' ? 4 : 2,
      })
    );

    elements.push(
      createText(
        node.position.x + radius,
        node.position.y + radius,
        node.label,
        {
          'text-anchor': 'middle',
          'dominant-baseline': 'middle',
          'font-family': config.fontFamily,
          'font-size': 10,
          'font-weight': 'bold',
          fill: '#000000',
        }
      )
    );

    return createGroup(elements, { id: `node-${node.id}` });
  }

  const isProcess = node.type === 'process';
  const width = isProcess ? 200 : 160;
  const height = isProcess ? 100 : 80;

  const safetyData = parseSafetyLevel(node.metadata);
  const safetyLevel = safetyData.level;
  const safetyStandard = safetyData.standard;
  const hasSafetyLevel = safetyLevel !== null;

  const nodeAttrs: Record<string, string> = {
    fill: node.color,
    'filter': 'drop-shadow(0 2px 4px rgba(0,0,0,0.15))',
  };

  if (hasSafetyLevel && safetyLevel) {
    const safetyAttrs = getSafetyBorderAttributes(safetyLevel, safetyStandard || undefined);
    Object.assign(nodeAttrs, safetyAttrs);
  } else {
    nodeAttrs.stroke = '#000000';
    nodeAttrs['stroke-width'] = isProcess ? '3' : '2';
  }

  elements.push(
    createRoundedRect(
      node.position.x,
      node.position.y,
      width,
      height,
      isProcess ? 12 : 8,
      nodeAttrs
    )
  );

  if (isProcess) {
    elements.push(
      createText(
        node.position.x + width / 2,
        node.position.y + 20,
        '«process»',
        {
          'text-anchor': 'middle',
          'font-family': config.fontFamily,
          'font-size': 10,
          'font-style': 'italic',
          fill: '#666666',
        }
      )
    );
  }

  const lines = wrapText(node.label, width - 20);
  const textStartY = node.position.y + (isProcess ? 45 : height / 2 - lines.length * 8);

  for (let i = 0; i < lines.length; i++) {
    elements.push(
      createText(
        node.position.x + width / 2,
        textStartY + i * 16,
        lines[i],
        {
          'text-anchor': 'middle',
          'dominant-baseline': 'middle',
          'font-family': config.fontFamily,
          'font-size': config.fontSize,
          'font-weight': isProcess ? 'bold' : 'normal',
          fill: '#000000',
        }
      )
    );
  }

  return createGroup(elements, { id: `node-${node.id}` });
}

function renderFlowEdge(edge: any, config: RenderConfig): SvgElement {
  if (!edge.points || edge.points.length < 2) {
    return createGroup([]);
  }

  const elements: SvgElement[] = [];
  const isConditional = edge.type === 'conditional';

  const pathD = edge.points.map((p: Point, i: number) =>
    i === 0 ? `M ${p.x} ${p.y}` : `L ${p.x} ${p.y}`
  ).join(' ');

  elements.push(
    createPath(pathD, {
      fill: 'none',
      stroke: isConditional ? '#FF6600' : '#000000',
      'stroke-width': 2,
      'stroke-dasharray': isConditional ? '5,5' : 'none',
      'marker-end': isConditional ? 'url(#arrow-conditional)' : 'url(#arrow-flow)',
    })
  );

  if (edge.label && config.showLabels) {
    const midIdx = Math.floor(edge.points.length / 2);
    const midPoint = edge.points[midIdx];

    elements.push(
      createRoundedRect(
        midPoint.x - 40,
        midPoint.y - 12,
        80,
        24,
        4,
        {
          fill: '#FFFFFF',
          stroke: isConditional ? '#FF6600' : '#999999',
          'stroke-width': 1,
        }
      )
    );

    elements.push(
      createText(midPoint.x, midPoint.y, `[${edge.label}]`, {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': config.fontFamily,
        'font-size': 10,
        'font-style': 'italic',
        fill: isConditional ? '#FF6600' : '#666666',
      })
    );
  }

  return createGroup(elements);
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
      if (currentLine) lines.push(currentLine);
      currentLine = word;
    }
  }

  if (currentLine) lines.push(currentLine);
  return lines;
}
