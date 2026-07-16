/**
 * Sequence Diagram Renderer
 * 
 * Renders UML-style sequence diagrams with:
 * - Vertical lifelines for participants
 * - Horizontal message arrows (sync, async, return)
 * - Combined fragments (PAR, OPT, LOOP, ALT)
 * - Activation bars
 * - Timing constraints
 */

import {
  Scenario,
  Message,
  CombinedFragment,
  TimingConstraint,
} from '../types/model';
import {
  DiagramNode,
  DiagramEdge,
  DiagramFragment,
  DiagramFragmentOperand,
  Point,
  Size,
  SvgElement,
  RenderConfig,
  DiagramOutput,
  CAPELLA_COLORS,
} from '../types/diagram';
import {
  applyTimelineLayout,
  calculateActivations,
  getMessageArrowStyle,
  calculateOperandDividers,
} from '../layouts/timeline';
import {
  createSvgDocument,
  createRect,
  createLine,
  createText,
  createGroup,
  createPath,
  createArrowMarker,
} from '../utils/svg';
import {
  getSafetyBorderAttributes,
  parseSafetyLevel,
} from '../utils/safety-colors';

// ============================================================================
// Configuration
// ============================================================================

const DEFAULT_CONFIG: RenderConfig = {
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
  fontSize: 12,
};

// ============================================================================
// Main Render Function
// ============================================================================

/**
 * Render sequence diagram from scenario
 */
export async function renderSequenceDiagram(
  scenario: Scenario,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg: RenderConfig = { ...DEFAULT_CONFIG, ...config };

  // 1. Convert model to diagram nodes and edges
  const { nodes, edges, fragments } = convertToDiagram(scenario);

  // 2. Apply timeline layout
  const layout = applyTimelineLayout(nodes, edges, fragments, {
    lifelineSpacing: 200,
    messageSpacing: 60,
    padding: { top: 100, right: 40, bottom: 40, left: 40 },
  });

  // 3. Render to SVG
  const svg = renderToSvg(scenario, layout, cfg);

  // 4. Return result
  const width = layout.totalSize.width;
  const height = layout.totalSize.height;

  return {
    svg,
    width,
    height,
    metadata: {
      diagramType: 'sequence',
      participantCount: scenario.participants.length,
      messageCount: scenario.messages.length,
      fragmentCount: scenario.fragments.length,
    },
  };
}

// ============================================================================
// Step 1: Convert Model to Diagram
// ============================================================================

function convertToDiagram(scenario: Scenario): {
  nodes: DiagramNode[];
  edges: DiagramEdge[];
  fragments: DiagramFragment[];
} {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];
  const fragments: DiagramFragment[] = [];

  // Convert participants to nodes
  for (const participant of scenario.participants) {
    nodes.push({
      id: participant.id,
      label: participant.name,
      type: participant.participant_type === 'Actor' ? 'actor' : 'component',
      metadata: {
        participantType: participant.participant_type,
      },
    });
  }

  // Convert messages to edges
  for (let i = 0; i < scenario.messages.length; i++) {
    const message = scenario.messages[i];
    edges.push({
      id: `msg-${i}`,
      from: message.from,
      to: message.to,
      label: message.label,
      type: mapMessageType(message.message_type),
      metadata: {
        activation: message.activation,
        timing: message.timing,
        params: message.params,
      },
    });
  }

  // Convert combined fragments
  for (let i = 0; i < scenario.fragments.length; i++) {
    const fragment = scenario.fragments[i];
    fragments.push({
      id: `fragment-${i}`,
      type: mapFragmentType(fragment.fragment_type),
      label: fragment.label,
      condition: fragment.condition || undefined,
      operands: fragment.operands.map(op => ({
        label: op.label || '',
        messageIds: op.messages.map((_, idx) => `msg-${idx}`),
      })),
    });
  }

  return { nodes, edges, fragments };
}

function mapMessageType(type: 'Synchronous' | 'Asynchronous' | 'Return'): 'message-sync' | 'message-async' | 'message-return' {
  switch (type) {
    case 'Synchronous': return 'message-sync';
    case 'Asynchronous': return 'message-async';
    case 'Return': return 'message-return';
  }
}

function mapFragmentType(type: 'Par' | 'Opt' | 'Loop' | 'Alt'): 'PAR' | 'OPT' | 'LOOP' | 'ALT' {
  return type.toUpperCase() as 'PAR' | 'OPT' | 'LOOP' | 'ALT';
}

// ============================================================================
// Step 2: Render to SVG
// ============================================================================

function renderToSvg(
  scenario: Scenario,
  layout: any,
  config: RenderConfig
): string {
  const elements: SvgElement[] = [];

  // Background
  elements.push(
    createRect(0, 0, layout.totalSize.width, layout.totalSize.height, {
      fill: config.colorScheme?.background || '#FFFFFF',
      stroke: 'none',
    })
  );

  // Render combined fragments first (behind everything)
  for (const fragment of layout.fragments) {
    elements.push(renderFragment(fragment, config));
  }

  // Render lifelines
  for (const node of layout.nodes) {
    elements.push(renderLifeline(node, layout, config));
  }

  // Render messages
  for (const edge of layout.edges) {
    elements.push(renderMessage(edge, layout.nodes, config));
  }

  // Render participants (boxes at top)
  for (const node of layout.nodes) {
    elements.push(renderParticipant(node, config));
  }

  // Render activation bars
  for (const node of layout.nodes) {
    const activations = calculateActivations(
      node.id,
      layout.edges,
      node.lifelineX,
      20
    );
    for (const activation of activations) {
      elements.push(renderActivation(activation, config));
    }
  }

  // Title
  if (scenario.name) {
    elements.push(
      createText(20, 30, scenario.name, {
        'font-family': 'Arial, sans-serif',
        'font-size': 18,
        'font-weight': 'bold',
        fill: '#212529',
      })
    );
  }

  // Create arrow markers
  const defs = [
    createArrowMarker('arrow-filled', '#000000', 10),
    createArrowMarker('arrow-open', '#000000', 10),
    createArrowMarker('arrow-black', '#000000', 10),
  ];

  return createSvgDocument(layout.totalSize.width, layout.totalSize.height, elements, defs);
}

// ============================================================================
// Render Individual Elements
// ============================================================================

function renderParticipant(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  // Determine color based on participant type
  let fillColor = '#E3F2FD'; // Default light blue
  let strokeColor = '#1976D2';
  
  if (node.type === 'actor') {
    fillColor = '#FFF9C4'; // Light yellow for actors
    strokeColor = '#F57C00';
  } else if (node.metadata?.participantType === 'Function') {
    fillColor = '#E8F5E9'; // Light green for functions
    strokeColor = '#388E3C';
  } else if (node.metadata?.participantType === 'Component') {
    fillColor = '#F3E5F5'; // Light purple for components
    strokeColor = '#7B1FA2';
  }

  // Participant box with gradient
  elements.push(
    createRect(node.position.x, node.position.y, node.size.width, node.size.height, {
      fill: fillColor,
      stroke: strokeColor,
      'stroke-width': 2.5,
      rx: 8,
      ry: 8,
    })
  );

  // Participant name with word wrapping
  const lines = node.label.split('\n');
  const lineHeight = 14;
  const startY = node.position.y + node.size.height / 2 - ((lines.length - 1) * lineHeight) / 2;
  
  lines.forEach((line: string, i: number) => {
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        startY + i * lineHeight,
        line,
        {
          'text-anchor': 'middle',
          'dominant-baseline': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 11,
          'font-weight': 'bold',
          fill: strokeColor,
        }
      )
    );
  });

  return createGroup(elements, { id: `participant-${node.id}` });
}

function renderLifeline(node: any, layout: any, config: RenderConfig): SvgElement {
  const startY = node.position.y + node.size.height;
  const endY = layout.totalSize.height - 40;

  return createLine(node.lifelineX, startY, node.lifelineX, endY, {
    stroke: '#90A4AE',
    'stroke-width': 2,
    'stroke-dasharray': '6,4',
  });
}

function renderMessage(edge: any, nodes: any[], config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];
  const points = edge.points;

  if (points.length < 2) return createGroup([]);

  const start = points[0];
  const end = points[points.length - 1];

  // Get arrow style
  const arrowStyle = getMessageArrowStyle(edge.type);

  const safetyData = parseSafetyLevel(edge.metadata);
  const safetyLevel = safetyData.level;
  const safetyStandard = safetyData.standard;
  const hasSafetyLevel = safetyLevel !== null;

  let messageColor = '#1976D2';
  let strokeWidth = 2.5;
  
  if (hasSafetyLevel && safetyLevel) {
    const safetyAttrs = getSafetyBorderAttributes(safetyLevel, safetyStandard || undefined);
    messageColor = safetyAttrs.stroke;
    strokeWidth = parseFloat(safetyAttrs['stroke-width']) || 2.5;
  } else if (edge.type === 'message-async') {
    messageColor = '#388E3C';
  } else if (edge.type === 'message-return') {
    messageColor = '#666666';
  }

  const lineAttrs: Record<string, any> = {
    stroke: messageColor,
    'stroke-width': strokeWidth,
    'marker-end': arrowStyle.markerEnd,
    fill: 'none',
  };

  if (arrowStyle.strokeDasharray) {
    lineAttrs['stroke-dasharray'] = arrowStyle.strokeDasharray;
  }

  // Check if self-call (4 points = rectangular loop)
  const isSelfCall = points.length === 4;
  
  if (isSelfCall) {
    // Draw rectangular loop for self-call
    const pathD = `M ${points[0].x} ${points[0].y} L ${points[1].x} ${points[1].y} L ${points[2].x} ${points[2].y} L ${points[3].x} ${points[3].y}`;
    elements.push(createPath(pathD, lineAttrs));
  } else {
    // Regular straight line
    elements.push(createLine(start.x, start.y, end.x, end.y, lineAttrs));
  }

  // Message label
  if (edge.label && config.showLabels) {
    let midX: number;
    let midY: number;
    
    if (isSelfCall) {
      // Position label to the right of the loop
      midX = points[1].x + 10;
      midY = (points[0].y + points[3].y) / 2;
    } else {
      // Center label above line
      midX = (start.x + end.x) / 2;
      midY = start.y - 10;
    }

    // Calculate label width based on actual text
    const fontSize = 10;
    const charWidth = fontSize * 0.65;
    const labelWidth = Math.max(edge.label.length * charWidth + 20, 100);
    const labelHeight = 20;
    const padding = 8;

    // Label background with subtle shadow
    elements.push(
      createRect(
        isSelfCall ? midX - padding : midX - labelWidth / 2 - padding,
        midY - labelHeight / 2,
        labelWidth + padding * 2,
        labelHeight,
        {
          fill: '#FFFFFF',
          stroke: messageColor,
          'stroke-width': 1.5,
          rx: 4,
          ry: 4,
          'filter': 'drop-shadow(0 1px 2px rgba(0,0,0,0.1))',
        }
      )
    );

    elements.push(
      createText(isSelfCall ? midX + labelWidth / 2 : midX, midY, edge.label, {
        'text-anchor': isSelfCall ? 'middle' : 'middle',
        'dominant-baseline': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': fontSize,
        'font-weight': '600',
        fill: messageColor,
      })
    );
  }

  return createGroup(elements, { id: edge.id });
}

function renderActivation(
  activation: { x: number; y: number; height: number },
  config: RenderConfig
): SvgElement {
  return createRect(activation.x, activation.y, 16, activation.height, {
    fill: '#E3F2FD',
    stroke: '#1976D2',
    'stroke-width': 2,
    rx: 2,
    ry: 2,
  });
}

function renderFragment(fragment: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  // Fragment box with subtle fill
  elements.push(
    createRect(fragment.position.x, fragment.position.y, fragment.size.width, fragment.size.height, {
      fill: '#FFF8E1',
      'fill-opacity': '0.3',
      stroke: '#F57C00',
      'stroke-width': 2.5,
      'stroke-dasharray': '10,5',
      rx: 4,
      ry: 4,
    })
  );

  // Fragment label box (top-left corner)
  const labelWidth = 70;
  const labelHeight = 28;

  elements.push(
    createRect(fragment.position.x, fragment.position.y, labelWidth, labelHeight, {
      fill: '#FFE082',
      stroke: '#F57C00',
      'stroke-width': 2.5,
      rx: 4,
      ry: 4,
    })
  );

  elements.push(
    createText(
      fragment.position.x + labelWidth / 2,
      fragment.position.y + labelHeight / 2,
      fragment.label,
      {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 11,
        'font-weight': 'bold',
        fill: '#E65100',
      }
    )
  );

  // Condition (if present)
  if (fragment.condition) {
    elements.push(
      createText(
        fragment.position.x + labelWidth + 12,
        fragment.position.y + labelHeight / 2,
        `[${fragment.condition}]`,
        {
          'dominant-baseline': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 10,
          'font-style': 'italic',
          'font-weight': '600',
          fill: '#F57C00',
        }
      )
    );
  }

  // Operand dividers
  const dividers = calculateOperandDividers(fragment, fragment.operands.length);
  for (const divider of dividers) {
    elements.push(
      createLine(divider[0].x, divider[0].y, divider[1].x, divider[1].y, {
        stroke: '#F57C00',
        'stroke-width': 2,
        'stroke-dasharray': '8,4',
      })
    );
  }

  return createGroup(elements, { id: fragment.id });
}

