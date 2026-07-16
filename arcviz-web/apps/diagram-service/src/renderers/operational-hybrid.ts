/**
 * Operational Diagram Renderer with Hybrid ELK+Dagre+D3 Engine
 * 
 * Uses multi-pass optimization for maximum quality:
 * 1. ELK: Hierarchical structure and swimlanes
 * 2. Dagre: Edge crossing minimization
 * 3. D3-Force: Collision detection
 * 4. Capella: MBSE style refinement
 */

import {
  OperationalAnalysis,
  OperationalActivity,
  OperationalExchange,
} from '../types/model';
import {
  DiagramNode,
  DiagramEdge,
  DiagramOutput,
  RenderConfig,
  CAPELLA_COLORS,
} from '../types/diagram';
import { applyHybridLayout } from '../layouts/hybrid-elk-dagre-d3';
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
} from '../utils/svg';
import { SvgElement } from '../types/diagram';
import {
  getSafetyBorderAttributes,
  parseSafetyLevel,
} from '../utils/safety-colors';

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

/**
 * Render operational activity diagram with hybrid engine
 */
export async function renderOperationalActivityHybrid(
  oa: OperationalAnalysis,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  console.log('[Operational-Hybrid] Converting model to diagram...');
  const { nodes, edges } = convertToDiagram(oa);
  console.log(`[Operational-Hybrid] Converted: ${nodes.length} nodes, ${edges.length} edges`);

  console.log('[Operational-Hybrid] Applying hybrid ELK+Dagre+D3 layout...');
  const layout = await applyHybridLayout(nodes, edges, {
    direction: 'RIGHT',
    elkNodeSpacing: 80,
    elkLayerSpacing: 100,
    dagreRankSep: 100,
    dagreNodeSep: 80,
    d3CollisionRadius: 50,
    d3Iterations: 100,
    elkWeight: 0.7,
    dagreWeight: 0.2,
    d3Weight: 0.1,
    minimumSpacing: 40,
  });

  console.log(`[Operational-Hybrid] Layout complete. Quality scores:`);
  console.log(`  ELK: ${layout.metadata.elkScore.toFixed(2)}`);
  console.log(`  Dagre: ${layout.metadata.dagreScore.toFixed(2)}`);
  console.log(`  D3: ${layout.metadata.d3Score.toFixed(2)}`);
  console.log(`  Time: ${layout.metadata.totalOptimizationTime.toFixed(2)}ms`);

  const svg = renderToSvg(oa, layout, cfg);

  return {
    svg,
    width: layout.totalSize.width,
    height: layout.totalSize.height,
    metadata: {
      diagramType: 'operational-activity-hybrid',
      name: oa.name,
      activityCount: oa.activities.length,
      exchangeCount: oa.exchanges.length,
      layoutEngine: 'ELK+Dagre+D3',
      optimizationTime: layout.metadata.totalOptimizationTime,
      qualityScores: {
        elk: layout.metadata.elkScore,
        dagre: layout.metadata.dagreScore,
        d3: layout.metadata.d3Score,
      },
    },
  };
}

function convertToDiagram(oa: OperationalAnalysis): {
  nodes: DiagramNode[];
  edges: DiagramEdge[];
} {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  const actorMap = new Map<string, string>();
  for (const actor of oa.actors) {
    const actorId = (actor.attributes?.id as any)?.String || actor.id || actor.name;
    actorMap.set(actorId, actor.name);
  }
  
  if (oa.entities) {
    for (const entity of oa.entities) {
      const entityId = (entity.attributes?.id as any)?.String || entity.id || entity.name;
      actorMap.set(entityId, entity.name);
    }
  }

  for (const activity of oa.activities) {
    nodes.push(convertActivityToNode(activity, actorMap));
  }

  for (const exchange of oa.exchanges) {
    edges.push(convertExchangeToEdge(exchange));
  }

  return { nodes, edges };
}

function convertActivityToNode(activity: OperationalActivity, actorMap: Map<string, string>): DiagramNode {
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

function renderToSvg(
  oa: OperationalAnalysis,
  layout: any,
  config: RenderConfig
): string {
  const elements: SvgElement[] = [];
  const defs: SvgElement[] = [];

  defs.push(createArrowMarker('arrow-black', '#000000'));
  defs.push(createArrowMarker('arrow-blue', config.colorScheme!.actor));

  elements.push(createRoundedRect(0, 0, layout.totalSize.width, layout.totalSize.height, 0, {
    fill: config.backgroundColor,
    stroke: 'none',
  }));

  // Group nodes by swimlane
  // IMPORTANT: Adjust node positions to avoid actor overlap
  // Swimlane header is 240px wide (0-240), so shift all nodes right by 240px
  const SWIMLANE_HEADER_WIDTH = 240;
  const adjustedNodes = layout.nodes.map((node: any) => ({
    ...node,
    position: {
      x: node.position.x + SWIMLANE_HEADER_WIDTH,
      y: node.position.y,
    },
  }));

  const swimlaneGroups = new Map<string, any[]>();
  for (const node of adjustedNodes) {
    const swimlane = node.metadata?.swimlane || 'default';
    if (!swimlaneGroups.has(swimlane)) {
      swimlaneGroups.set(swimlane, []);
    }
    swimlaneGroups.get(swimlane)!.push(node);
  }

  // Render swimlanes
  let swimlaneY = 60;
  for (const [swimlaneName, swimlaneNodes] of swimlaneGroups.entries()) {
    if (swimlaneNodes.length === 0) continue;

    const minY = Math.min(...swimlaneNodes.map((n: any) => n.position.y));
    const maxY = Math.max(...swimlaneNodes.map((n: any) => n.position.y + n.size.height));
    const swimlaneHeight = maxY - minY + 80;

    elements.push(createRoundedRect(0, swimlaneY, layout.totalSize.width, swimlaneHeight, 5, {
      fill: '#F8F9FA',
      stroke: '#CED4DA',
      'stroke-width': 2,
    }));

    elements.push(createRoundedRect(0, swimlaneY, SWIMLANE_HEADER_WIDTH, swimlaneHeight, 5, {
      fill: '#E9ECEF',
      stroke: '#CED4DA',
      'stroke-width': 2,
    }));

    const figureX = 100;
    const figureY = swimlaneY + 20;
    elements.push(createStickFigure(figureX, figureY, 1.5, config.colorScheme!.actor));

    const actorNameLines = wrapText(swimlaneName, 180);
    elements.push(
      createMultilineText(
        figureX,
        figureY + 80,
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

    swimlaneY += swimlaneHeight + 40;
  }

  // Adjust edge points to match shifted nodes
  const adjustedEdges = layout.edges.map((edge: any) => ({
    ...edge,
    points: edge.points.map((p: any) => ({
      x: p.x + SWIMLANE_HEADER_WIDTH,
      y: p.y,
    })),
  }));

  for (const edge of adjustedEdges) {
    console.log(`[Renderer] Edge ${edge.id}: ${edge.points.length} points`, edge.points);
    elements.push(renderEdge(edge, config));
  }

  for (const node of adjustedNodes) {
    elements.push(renderNode(node, config));
  }

  elements.push(renderTitle(oa.name, config));

  // Add optimization info badge
  elements.push(renderOptimizationBadge(layout.metadata, config));

  return createSvgDocument(
    layout.totalSize.width,
    layout.totalSize.height,
    elements,
    defs
  );
}

function renderNode(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  const safetyData = parseSafetyLevel(node.metadata);
  const safetyLevel = safetyData.level;
  const safetyStandard = safetyData.standard;
  const hasSafetyLevel = safetyLevel !== null;

  const activityAttrs: Record<string, string> = {
    fill: node.color,
  };
  
  if (hasSafetyLevel && safetyLevel) {
    const safetyAttrs = getSafetyBorderAttributes(safetyLevel, safetyStandard || undefined);
    Object.assign(activityAttrs, safetyAttrs);
  } else {
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

  // Use simple line for 2-point paths, rounded path for complex paths
  let pathD: string;
  if (edge.points.length === 2) {
    pathD = `M ${edge.points[0].x} ${edge.points[0].y} L ${edge.points[1].x} ${edge.points[1].y}`;
  } else {
    pathD = createRoundedPath(edge.points, 10);
  }
  
  elements.push(
    createPath(pathD, {
      fill: 'none',
      stroke: '#000000',
      'stroke-width': 2,
      'marker-end': 'url(#arrow-black)',
    })
  );

  if (edge.label && config.showLabels) {
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

    const fontSize = (config.fontSize || 12) - 1;
    const labelWidth = Math.max(edge.label.length * (fontSize * 0.6), 80);
    const labelHeight = 20;
    const padding = 6;
    
    // Position label ABOVE the arrow line to avoid overlap
    const labelY = midPoint.y - 15;
    
    elements.push(
      createRoundedRect(
        midPoint.x - labelWidth / 2 - padding,
        labelY - labelHeight / 2,
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
      createText(midPoint.x, labelY, edge.label, {
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

function renderOptimizationBadge(metadata: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];
  
  const badgeX = 20;
  const badgeY = 50;
  const badgeWidth = 300;
  const badgeHeight = 70;
  
  elements.push(
    createRoundedRect(badgeX, badgeY, badgeWidth, badgeHeight, 5, {
      fill: '#F0F8FF',
      stroke: '#4472C4',
      'stroke-width': 2,
    })
  );
  
  elements.push(
    createText(badgeX + 10, badgeY + 20, 'Hybrid ELK+Dagre+D3 Engine', {
      'font-family': config.fontFamily,
      'font-size': config.fontSize! + 2,
      'font-weight': 'bold',
      fill: '#4472C4',
    })
  );
  
  const infoText = `ELK: ${metadata.elkScore.toFixed(0)} | Dagre: ${metadata.dagreScore.toFixed(0)} | D3: ${metadata.d3Score.toFixed(0)} | ${metadata.totalOptimizationTime.toFixed(0)}ms`;
  elements.push(
    createText(badgeX + 10, badgeY + 45, infoText, {
      'font-family': config.fontFamily,
      'font-size': config.fontSize! - 1,
      fill: '#6C757D',
    })
  );
  
  return createGroup(elements, { id: 'optimization-badge' });
}
