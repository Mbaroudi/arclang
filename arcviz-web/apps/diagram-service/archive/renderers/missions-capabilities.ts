/**
 * Missions & Capabilities Blank (MCB) Renderer
 * 
 * Implements LaTeX Specification Section 2 (System Analysis Diagram Types)
 * MCB: System missions and capabilities definition
 * 
 * Shows:
 * - Mission hierarchy
 * - Capability realization
 * - Mission-capability relationships
 * - Operational capability refinement
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
import { Mission, CapabilityRealization } from '../types/model';
import { applyHierarchicalLayout } from '../layouts/hierarchical';
import {
  createSvgDocument,
  createRoundedRect,
  createText,
  createGroup,
  createPath,
  createArrowMarker,
  createMultilineText,
  wrapText,
} from '../utils/svg';
import {
  getTraceabilityStyle,
  createTraceabilityLabel,
  createAllTraceabilityMarkers,
} from '../utils/traceability-styles';

const DEFAULT_CONFIG: RenderConfig = {
  width: 1600,
  height: 1200,
  backgroundColor: '#FFFFFF',
  fontSize: 12,
  fontFamily: 'Arial, sans-serif',
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
};

export interface MissionsCapabilitiesModel {
  missions: Mission[];
  capability_realizations: CapabilityRealization[];
}

/**
 * Render Missions & Capabilities Blank diagram
 */
export async function renderMissionsCapabilities(
  model: MissionsCapabilitiesModel,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  const { nodes, edges } = convertToDiagram(model);

  const layout = await applyHierarchicalLayout(nodes, edges, {
    direction: 'DOWN',
    nodeSpacing: 80,
    layerSpacing: 150,
  });

  const svg = renderToSvg(layout, cfg);

  return {
    svg,
    width: layout.totalSize.width,
    height: layout.totalSize.height,
    metadata: {
      diagramType: 'MCB',
      missionCount: model.missions.length,
      capabilityCount: model.capability_realizations.length,
    },
  };
}

function convertToDiagram(
  model: MissionsCapabilitiesModel
): { nodes: DiagramNode[]; edges: DiagramEdge[] } {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  for (const mission of model.missions) {
    nodes.push({
      id: mission.id,
      label: mission.name,
      type: 'mission',
      color: '#FFC000',
      metadata: {
        description: mission.description,
        capabilities: mission.capabilities,
      },
    });
  }

  for (const capReal of model.capability_realizations) {
    nodes.push({
      id: capReal.id,
      label: capReal.name,
      type: 'capability-realization',
      color: '#ADD8E6',
      metadata: {
        description: capReal.description,
        realized_capability_id: capReal.realized_capability_id,
        components: capReal.involved_components,
        functions: capReal.involved_functions,
      },
    });

    const mission = model.missions.find(m => 
      m.capabilities.includes(capReal.realized_capability_id)
    );
    
    if (mission) {
      edges.push({
        id: `${mission.id}-${capReal.id}`,
        from: mission.id,
        to: capReal.id,
        type: 'realizes',
        label: 'realizes',
      });
    }
  }

  return { nodes, edges };
}

function renderToSvg(layout: any, config: RenderConfig): string {
  const elements: SvgElement[] = [];
  const defs: SvgElement[] = [];

  const realizesStyle = getTraceabilityStyle('realizes');
  defs.push(createArrowMarker(realizesStyle.markerEnd, realizesStyle.strokeColor, 10));

  elements.push(
    createRoundedRect(0, 0, layout.totalSize.width, layout.totalSize.height, 0, {
      fill: config.backgroundColor,
      stroke: 'none',
    })
  );

  for (const edge of layout.edges) {
    elements.push(renderEdge(edge, config));
  }

  for (const node of layout.nodes) {
    elements.push(renderNode(node, config));
  }

  elements.push(
    createText(20, 30, 'Missions & Capabilities Blank (MCB)', {
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

function renderNode(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  const isMission = node.type === 'mission';
  const width = isMission ? 250 : 200;
  const height = isMission ? 100 : 80;

  elements.push(
    createRoundedRect(
      node.position.x,
      node.position.y,
      width,
      height,
      10,
      {
        fill: node.color,
        stroke: isMission ? '#000000' : '#333333',
        'stroke-width': isMission ? 3 : 2,
        'filter': 'drop-shadow(0 3px 6px rgba(0,0,0,0.15))',
      }
    )
  );

  const stereotype = isMission ? '«mission»' : '«capability»';
  elements.push(
    createText(
      node.position.x + width / 2,
      node.position.y + 20,
      stereotype,
      {
        'text-anchor': 'middle',
        'font-family': config.fontFamily,
        'font-size': 10,
        'font-style': 'italic',
        fill: '#666666',
      }
    )
  );

  const lines = wrapText(node.label, width - 20);
  const textStartY = node.position.y + 40;

  for (let i = 0; i < lines.length; i++) {
    elements.push(
      createText(
        node.position.x + width / 2,
        textStartY + i * 16,
        lines[i],
        {
          'text-anchor': 'middle',
          'font-family': config.fontFamily,
          'font-size': config.fontSize,
          'font-weight': isMission ? 'bold' : 'normal',
          fill: '#000000',
        }
      )
    );
  }

  return createGroup(elements, { id: `node-${node.id}` });
}

function renderEdge(edge: any, config: RenderConfig): SvgElement {
  if (!edge.points || edge.points.length < 2) {
    return createGroup([]);
  }

  const elements: SvgElement[] = [];

  const pathD = edge.points.map((p: Point, i: number) =>
    i === 0 ? `M ${p.x} ${p.y}` : `L ${p.x} ${p.y}`
  ).join(' ');

  const traceStyle = getTraceabilityStyle(edge.type || 'realizes');

  elements.push(
    createPath(pathD, {
      fill: 'none',
      stroke: traceStyle.strokeColor,
      'stroke-width': traceStyle.strokeWidth,
      'stroke-dasharray': traceStyle.strokeDasharray,
      'marker-end': `url(#${traceStyle.markerEnd})`,
    })
  );

  if (edge.label && config.showLabels) {
    const midIdx = Math.floor(edge.points.length / 2);
    const midPoint = edge.points[midIdx];

    const labelStyle = traceStyle.labelStyle;
    const padding = 6;
    const labelWidth = Math.max(edge.label.length * labelStyle.fontSize * 0.6, 70);
    const labelHeight = 20;

    elements.push(
      createRoundedRect(
        midPoint.x - labelWidth / 2 - padding,
        midPoint.y - labelHeight / 2,
        labelWidth + padding * 2,
        labelHeight,
        4,
        {
          fill: labelStyle.backgroundColor,
          stroke: labelStyle.border,
          'stroke-width': 1,
        }
      )
    );

    elements.push(
      createText(midPoint.x, midPoint.y, `«${edge.label}»`, {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': config.fontFamily,
        'font-size': labelStyle.fontSize,
        'font-style': labelStyle.fontStyle,
        'font-weight': labelStyle.fontWeight,
        fill: labelStyle.fill,
      })
    );
  }

  return createGroup(elements);
}
