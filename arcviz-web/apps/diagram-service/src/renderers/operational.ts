/**
 * Operational Activity Diagram Renderer
 * 
 * Renders operational activity diagrams with swimlanes, matching Capella's visual style.
 * Features:
 * - Horizontal swimlanes for each actor/entity
 * - Yellow activity boxes with icons
 * - Stick figures for actors
 * - Data flow arrows with labels
 * - Hierarchical activity decomposition
 */

import {
  OperationalAnalysis,
  OperationalActivity,
  OperationalExchange,
  Entity,
} from '../types/model';
import {
  DiagramNode,
  DiagramEdge,
  DiagramOutput,
  RenderConfig,
  CAPELLA_COLORS,
} from '../types/diagram';
import { applySwimlaneLayout } from '../layouts/swimlane';
import { applyElkOperationalLayout } from '../layouts/elk-operational';
import {
  createSvgDocument,
  createRoundedRect,
  createText,
  createGroup,
  createPath,
  createArrowMarker,
  createStickFigure,
  createMultilineText,
  wrapText,
  createRoundedPath,
  renderSvgElement,
} from '../utils/svg';
import { SvgElement } from '../types/diagram';
import {
  getSafetyBorderAttributes,
  parseSafetyLevel,
  isSafetyCritical,
} from '../utils/safety-colors';

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

// ============================================================================
// Main Render Function
// ============================================================================

/**
 * Render operational activity diagram from ArcLang model
 */
export async function renderOperationalActivity(
  oa: OperationalAnalysis,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  const { nodes, edges } = convertToDiagram(oa);

  const layout = await applyElkOperationalLayout(nodes, edges, {
    direction: 'RIGHT',
    nodeSpacing: 80,
    layerSpacing: 100,
    edgeSpacing: 40,
  });

  const svg = renderToSvg(oa, layout, cfg);

  return {
    svg,
    width: layout.totalSize.width,
    height: layout.totalSize.height,
    metadata: {
      diagramType: 'operational-activity',
      name: oa.name,
      activityCount: oa.activities.length,
      exchangeCount: oa.exchanges.length,
    },
  };
}

// ============================================================================
// Step 1: Convert Model to Diagram
// ============================================================================

function convertToDiagram(oa: OperationalAnalysis): {
  nodes: DiagramNode[];
  edges: DiagramEdge[];
} {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  // Build actor ID to name mapping
  const actorMap = new Map<string, string>();
  for (const actor of oa.actors) {
    const actorId = (actor.attributes?.id as any)?.String || actor.id || actor.name;
    actorMap.set(actorId, actor.name);
  }
  
  // Also add entities
  if (oa.entities) {
    for (const entity of oa.entities) {
      const entityId = (entity.attributes?.id as any)?.String || entity.id || entity.name;
      actorMap.set(entityId, entity.name);
    }
  }

  // Convert activities to nodes
  for (const activity of oa.activities) {
    nodes.push(convertActivityToNode(activity, actorMap));
  }

  // Convert exchanges to edges
  for (const exchange of oa.exchanges) {
    edges.push(convertExchangeToEdge(exchange));
  }

  return { nodes, edges };
}

function convertActivityToNode(activity: OperationalActivity, actorMap: Map<string, string>): DiagramNode {
  // Resolve actor name from ID
  const performedById = (activity.attributes?.performed_by as any)?.String || activity.performed_by;
  const actorName = actorMap.get(performedById) || performedById || 'default';
  
  return {
    id: activity.id,
    label: activity.name,
    type: 'activity',
    color: activity.color || CAPELLA_COLORS.activity,
    icon: activity.icon,
    metadata: {
      performed_by: performedById,
      swimlane: actorName,
      swimlaneLabel: actorName,
      category: activity.category,
      sub_activities: activity.sub_activities,
    },
  };
}

function convertExchangeToEdge(exchange: OperationalExchange): DiagramEdge {
  return {
    id: `${exchange.from}-${exchange.to}`,
    from: exchange.from,
    to: exchange.to,
    label: exchange.label || exchange.data_type,
    type: 'operational-exchange',
    metadata: {
      data_type: exchange.data_type,
      protocol: exchange.protocol,
    },
  };
}

// ============================================================================
// Step 2: Render to SVG
// ============================================================================

function renderToSvg(
  oa: OperationalAnalysis,
  layout: ReturnType<typeof applySwimlaneLayout>,
  config: RenderConfig
): string {
  const elements: SvgElement[] = [];
  const defs: SvgElement[] = [];

  // Create arrow markers
  defs.push(createArrowMarker('arrow-black', '#000000'));
  defs.push(createArrowMarker('arrow-blue', config.colorScheme!.actor));

  // Render background
  elements.push(createBackground(layout.totalSize, config));

  // Render swimlanes
  for (const swimlane of layout.swimlanes) {
    elements.push(renderSwimlane(swimlane, oa, config));
  }

  // Render edges
  for (const edge of layout.edges) {
    elements.push(renderEdge(edge, config));
  }

  // Render nodes
  for (const node of layout.nodes) {
    elements.push(renderNode(node, config));
  }

  // Render title
  elements.push(renderTitle(oa.name, config));

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

function renderSwimlane(
  swimlane: any,
  oa: OperationalAnalysis,
  config: RenderConfig
): SvgElement {
  const elements: SvgElement[] = [];

  // Swimlane background
  elements.push(
    createRoundedRect(
      swimlane.position.x,
      swimlane.position.y,
      swimlane.size.width,
      swimlane.size.height,
      5,
      {
        fill: '#F8F9FA',
        stroke: '#CED4DA',
        'stroke-width': 2,
      }
    )
  );

  // Swimlane header area
  elements.push(
    createRoundedRect(
      swimlane.position.x,
      swimlane.position.y,
      200,
      swimlane.size.height,
      5,
      {
        fill: '#E9ECEF',
        stroke: '#CED4DA',
        'stroke-width': 2,
      }
    )
  );

  // Find entity/actor for this swimlane
  const entity = oa.entities.find(e => e.id === swimlane.id || e.name === swimlane.label);
  const actor = oa.actors.find(a => a.name === swimlane.label);

  // Render actor stick figure or entity icon
  if (entity || actor) {
    const figureX = swimlane.position.x + 100;
    const figureY = swimlane.position.y + 20;
    const figureScale = 1.5;
    
    // Calculate stick figure height: head + body + legs
    const headRadius = 8 * figureScale;
    const bodyHeight = 20 * figureScale;
    const legHeight = bodyHeight * 0.5;
    const totalFigureHeight = headRadius * 2 + bodyHeight + legHeight;

    elements.push(createStickFigure(figureX, figureY, figureScale, config.colorScheme!.actor));

    // Actor/Entity name below the figure with spacing
    const actorNameLines = wrapText(swimlane.label, 180);
    const textStartY = figureY + totalFigureHeight + 15;
    
    elements.push(
      createMultilineText(
        figureX,
        textStartY,
        actorNameLines,
        14,
        {
          'text-anchor': 'middle',
          'font-family': config.fontFamily,
          'font-size': config.fontSize,
          'font-weight': 'bold',
          fill: '#212529',
        }
      )
    );
  } else {
    // Just text label
    const lines = wrapText(swimlane.label, 180);
    elements.push(
      createMultilineText(
        swimlane.position.x + 100,
        swimlane.position.y + swimlane.size.height / 2,
        lines,
        16,
        {
          'text-anchor': 'middle',
          'dominant-baseline': 'middle',
          'font-family': config.fontFamily,
          'font-size': config.fontSize! + 2,
          'font-weight': 'bold',
          fill: '#212529',
        }
      )
    );
  }

  return createGroup(elements, { id: `swimlane-${swimlane.id}` });
}

function renderNode(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  // Check for safety-critical activities
  const safetyData = parseSafetyLevel(node.metadata);
  const safetyLevel = safetyData.level;
  const safetyStandard = safetyData.standard;
  const hasSafetyLevel = safetyLevel !== null;

  // Activity box with safety border if applicable
  const activityAttrs: Record<string, string> = {
    fill: node.color,
  };
  
  if (hasSafetyLevel && safetyLevel) {
    // Apply safety-critical border styling
    const safetyAttrs = getSafetyBorderAttributes(safetyLevel, safetyStandard || undefined);
    Object.assign(activityAttrs, safetyAttrs);
  } else {
    // Standard activity styling
    activityAttrs.stroke = '#000000';
    activityAttrs['stroke-width'] = '2';
  }
  
  elements.push(
    createRoundedRect(
      node.position.x,
      node.position.y,
      node.size.width,
      node.size.height,
      8,
      activityAttrs
    )
  );

  // Activity icon (if present)
  if (node.icon) {
    elements.push(renderIcon(node.icon, node.position.x + 10, node.position.y + 10, 20));
  }

  // Activity label
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
          fill: '#000000',
        }
      )
    );
  }

  // Activity ID (small, top-right corner)
  elements.push(
    createText(
      node.position.x + node.size.width - 5,
      node.position.y + 12,
      node.id,
      {
        'text-anchor': 'end',
        'font-family': config.fontFamily,
        'font-size': config.fontSize! - 2,
        fill: '#6C757D',
      }
    )
  );

  return createGroup(elements, { id: `node-${node.id}` });
}

function renderEdge(edge: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  // Edge path
  const pathD = createRoundedPath(edge.points, 10);
  elements.push(
    createPath(pathD, {
      fill: 'none',
      stroke: '#000000',
      'stroke-width': 2,
      'marker-end': 'url(#arrow-black)',
    })
  );

  // Edge label
  if (edge.label && config.showLabels) {
    // Calculate true geometric midpoint by traversing the path
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

    // Background for label - better width calculation
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
        'font-family': config.fontFamily,
        'font-size': config.fontSize! - 1,
        fill: '#495057',
      })
    );
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

function renderIcon(iconName: string, x: number, y: number, size: number): SvgElement {
  // Simple icon rendering - could be replaced with icon library
  const iconMap: Record<string, string> = {
    circle: 'M 10,10 m -8,0 a 8,8 0 1,0 16,0 a 8,8 0 1,0 -16,0',
    headphones: 'M 5,8 C 5,4 8,2 10,2 C 12,2 15,4 15,8 M 5,8 L 5,14 M 15,8 L 15,14',
    entertainment: 'M 4,6 L 16,6 L 16,14 L 4,14 Z M 8,10 L 12,8 L 12,12 Z',
    general: 'M 6,6 L 14,6 L 14,14 L 6,14 Z',
  };

  const pathD = iconMap[iconName] || iconMap.general;

  return createPath(pathD, {
    transform: `translate(${x}, ${y}) scale(${size / 20})`,
    fill: 'none',
    stroke: '#000000',
    'stroke-width': 1.5,
    'stroke-linecap': 'round',
    'stroke-linejoin': 'round',
  });
}
