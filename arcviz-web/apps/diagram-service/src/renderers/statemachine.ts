/**
 * State Machine Diagram Renderer
 * 
 * Renders UML-inspired state machine diagrams with:
 * - Initial and final states
 * - Regular states with entry/exit actions
 * - Nested states (composite states)
 * - Transitions with guards and actions
 * - Professional Capella-style design
 */

import {
  StateMachine,
  State,
  Transition,
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
  createCircle,
  createLine,
  createText,
  createGroup,
  createPath,
  createArrowMarker,
  createRoundedRect,
} from '../utils/svg';

// ============================================================================
// Configuration
// ============================================================================

const DEFAULT_CONFIG: RenderConfig = {
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
  fontSize: 11,
};

// ============================================================================
// Main Render Function
// ============================================================================

/**
 * Render state machine diagram
 */
export async function renderStateMachine(
  sm: StateMachine,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg: RenderConfig = { ...DEFAULT_CONFIG, ...config };

  // 1. Convert model to diagram nodes and edges
  const { nodes, edges } = convertToDiagram(sm);

  // 2. Apply layout
  const layout = await applyHierarchicalLayout(nodes, edges, {
    direction: 'RIGHT',
    nodeSpacing: 100,
    layerSpacing: 150,
  });

  // 3. Render to SVG
  const svg = renderToSvg(sm, layout, cfg);

  return {
    svg,
    width: layout.totalSize.width,
    height: layout.totalSize.height,
    metadata: {
      diagramType: 'statemachine',
      stateCount: sm.states.length,
      transitionCount: sm.transitions.length,
    },
  };
}

// ============================================================================
// Step 1: Convert Model to Diagram
// ============================================================================

function convertToDiagram(sm: StateMachine): {
  nodes: DiagramNode[];
  edges: DiagramEdge[];
} {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  // Add initial state (pseudo-state)
  nodes.push({
    id: '__initial__',
    label: '',
    type: 'initial-state',
    size: { width: 20, height: 20 },
  });

  // Add regular states
  for (const state of sm.states) {
    nodes.push(convertStateToNode(state));
  }

  // Add transitions
  for (let i = 0; i < sm.transitions.length; i++) {
    const transition = sm.transitions[i];
    edges.push({
      id: `transition-${i}`,
      from: transition.from === 'initial' ? '__initial__' : transition.from,
      to: transition.to === 'final' ? '__final__' : transition.to,
      label: formatTransitionLabel(transition),
      type: 'transition',
      metadata: {
        trigger: transition.trigger,
        guard: transition.guard,
        effect: transition.effect,
      },
    });
  }

  // Add final state if needed
  const hasFinalTransition = sm.transitions.some(t => t.to === 'final');
  if (hasFinalTransition) {
    nodes.push({
      id: '__final__',
      label: '',
      type: 'final-state',
      size: { width: 30, height: 30 },
    });
  }

  return { nodes, edges };
}

function convertStateToNode(state: State): DiagramNode {
  const hasSubStates = state.sub_states && state.sub_states.length > 0;
  
  // Calculate dynamic height based on content
  const headerHeight = 30;
  const lineHeight = 14;
  const padding = 15;
  
  const entryCount = state.entry_actions?.length || 0;
  const exitCount = state.exit_actions?.length || 0;
  const internalCount = state.internal_transitions?.length || 0;
  
  const totalLines = entryCount + exitCount + internalCount;
  const contentHeight = totalLines * lineHeight + padding * 2;
  const calculatedHeight = Math.max(80, headerHeight + contentHeight);
  
  return {
    id: state.name,
    label: state.name,
    type: hasSubStates ? 'composite-state' : 'state',
    size: { width: 180, height: calculatedHeight },
    metadata: {
      entryActions: state.entry_actions,
      exitActions: state.exit_actions,
      internalTransitions: state.internal_transitions,
      hasSubStates,
    },
    children: hasSubStates ? state.sub_states.map(convertStateToNode) : undefined,
  };
}

function formatTransitionLabel(transition: Transition): string {
  const parts: string[] = [];
  
  if (transition.trigger) {
    parts.push(transition.trigger);
  }
  
  if (transition.guard) {
    parts.push(`[${transition.guard}]`);
  }
  
  if (transition.effect) {
    parts.push(`/ ${transition.effect}`);
  }
  
  return parts.join(' ');
}

// ============================================================================
// Step 2: Render to SVG
// ============================================================================

function renderToSvg(
  sm: StateMachine,
  layout: any,
  config: RenderConfig
): string {
  const elements: SvgElement[] = [];
  const defs: SvgElement[] = [];

  // Create arrow markers
  defs.push(createArrowMarker('arrow-transition', '#1976D2', 10));
  defs.push(createArrowMarker('arrow-black', '#000000', 10));

  // Background
  elements.push(
    createRect(0, 0, layout.totalSize.width, layout.totalSize.height, {
      fill: config.colorScheme?.background || '#FFFFFF',
      stroke: 'none',
    })
  );

  // Render transitions first (behind states)
  for (const edge of layout.edges) {
    elements.push(renderTransition(edge, layout.nodes, config));
  }

  // Render states
  for (const node of layout.nodes) {
    elements.push(renderState(node, config));
  }

  // Title
  if (sm.name) {
    elements.push(
      createText(20, 35, sm.name, {
        'font-family': 'Arial, sans-serif',
        'font-size': 18,
        'font-weight': 'bold',
        fill: '#212529',
      })
    );
  }

  return createSvgDocument(
    layout.totalSize.width,
    layout.totalSize.height,
    elements,
    defs
  );
}

// ============================================================================
// Render Individual Elements
// ============================================================================

function renderState(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  // Initial state (filled circle)
  if (node.type === 'initial-state') {
    elements.push(
      createCircle(
        node.position.x + node.size.width / 2,
        node.position.y + node.size.height / 2,
        10,
        {
          fill: '#000000',
          stroke: 'none',
        }
      )
    );
    return createGroup(elements, { id: `state-${node.id}` });
  }

  // Final state (double circle)
  if (node.type === 'final-state') {
    const cx = node.position.x + node.size.width / 2;
    const cy = node.position.y + node.size.height / 2;
    
    elements.push(
      createCircle(cx, cy, 15, {
        fill: 'none',
        stroke: '#000000',
        'stroke-width': 2,
      })
    );
    elements.push(
      createCircle(cx, cy, 10, {
        fill: '#000000',
        stroke: 'none',
      })
    );
    return createGroup(elements, { id: `state-${node.id}` });
  }

  // Regular or composite state with color variety
  const isComposite = node.type === 'composite-state';
  
  // Assign different colors to different states
  const stateColors: Record<string, {fill: string, stroke: string}> = {
    'Idle': { fill: '#E8F5E9', stroke: '#388E3C' },
    'Monitoring': { fill: '#E3F2FD', stroke: '#1976D2' },
    'Warning': { fill: '#FFF9C4', stroke: '#F57C00' },
    'EmergencyBraking': { fill: '#FFEBEE', stroke: '#D32F2F' },
  };
  
  const colors = stateColors[node.label] || { fill: '#F3E5F5', stroke: '#7B1FA2' };
  const fillColor = isComposite ? '#E1F5FE' : colors.fill;
  const strokeColor = colors.stroke;

  // State box with shadow and gradient effect
  elements.push(
    createRoundedRect(
      node.position.x,
      node.position.y,
      node.size.width,
      node.size.height,
      12,
      {
        fill: fillColor,
        stroke: strokeColor,
        'stroke-width': 3,
        'filter': 'drop-shadow(0 2px 4px rgba(0,0,0,0.15))',
      }
    )
  );

  // State name (in header)
  const headerHeight = 30;
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + headerHeight / 2,
      node.label,
      {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 12,
        'font-weight': 'bold',
        fill: strokeColor,
      }
    )
  );

  // Separator line after header with thicker styling
  elements.push(
    createLine(
      node.position.x,
      node.position.y + headerHeight,
      node.position.x + node.size.width,
      node.position.y + headerHeight,
      {
        stroke: strokeColor,
        'stroke-width': 2,
        'opacity': '0.7',
      }
    )
  );

  // Entry actions
  let currentY = node.position.y + headerHeight + 15;
  if (node.metadata?.entryActions && node.metadata.entryActions.length > 0) {
    for (const action of node.metadata.entryActions) {
      elements.push(
        createText(
          node.position.x + 10,
          currentY,
          `entry / ${action}`,
          {
            'font-family': 'Arial, sans-serif',
            'font-size': 10,
            'font-style': 'italic',
            fill: '#666666',
          }
        )
      );
      currentY += 14;
    }
  }

  // Exit actions
  if (node.metadata?.exitActions && node.metadata.exitActions.length > 0) {
    for (const action of node.metadata.exitActions) {
      elements.push(
        createText(
          node.position.x + 10,
          currentY,
          `exit / ${action}`,
          {
            'font-family': 'Arial, sans-serif',
            'font-size': 10,
            'font-style': 'italic',
            fill: '#666666',
          }
        )
      );
      currentY += 14;
    }
  }

  // Internal transitions
  if (node.metadata?.internalTransitions && node.metadata.internalTransitions.length > 0) {
    for (const trans of node.metadata.internalTransitions) {
      elements.push(
        createText(
          node.position.x + 10,
          currentY,
          trans,
          {
            'font-family': 'Arial, sans-serif',
            'font-size': 10,
            fill: '#333333',
          }
        )
      );
      currentY += 14;
    }
  }

  return createGroup(elements, { id: `state-${node.id}` });
}

function renderTransition(edge: any, nodes: any[], config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];
  const points = edge.points;

  if (!points || points.length < 2) return createGroup([]);

  const start = points[0];
  const end = points[points.length - 1];

  // Check for self-transition (4 points = loop)
  const isSelfTransition = points.length === 4;
  
  // Determine transition color based on type
  let transitionColor = '#1976D2'; // Default blue
  if (edge.label.includes('emergency') || edge.label.includes('critical')) {
    transitionColor = '#D32F2F'; // Red for critical
  } else if (edge.label.includes('warning') || edge.label.includes('threat')) {
    transitionColor = '#F57C00'; // Orange for warning
  } else if (edge.label.includes('cleared') || edge.label.includes('avoided')) {
    transitionColor = '#388E3C'; // Green for success
  }

  if (isSelfTransition) {
    // Draw curved loop for self-transition
    const pathD = `M ${points[0].x} ${points[0].y} L ${points[1].x} ${points[1].y} L ${points[2].x} ${points[2].y} L ${points[3].x} ${points[3].y}`;
    elements.push(
      createPath(pathD, {
        fill: 'none',
        stroke: transitionColor,
        'stroke-width': 2.5,
        'marker-end': 'url(#arrow-transition)',
      })
    );
  } else {
    // Regular transition arrow
    elements.push(
      createLine(start.x, start.y, end.x, end.y, {
        stroke: transitionColor,
        'stroke-width': 2.5,
        'marker-end': 'url(#arrow-transition)',
      })
    );
  }

  // Transition label
  if (edge.label && config.showLabels) {
    const midX = isSelfTransition ? points[1].x + 10 : (start.x + end.x) / 2;
    const midY = isSelfTransition ? (points[0].y + points[3].y) / 2 : start.y - 8;

    const fontSize = 10;
    const labelWidth = Math.max(edge.label.length * (fontSize * 0.6), 80);
    const labelHeight = 18;
    const padding = 6;

    // Label background with shadow
    let labelBgColor = '#FFFFFF';
    let labelStrokeColor = transitionColor;
    
    elements.push(
      createRect(
        isSelfTransition ? midX - padding : midX - labelWidth / 2 - padding,
        midY - labelHeight / 2,
        labelWidth + padding * 2,
        labelHeight,
        {
          fill: labelBgColor,
          stroke: labelStrokeColor,
          'stroke-width': 1.5,
          rx: 4,
          ry: 4,
          'filter': 'drop-shadow(0 1px 3px rgba(0,0,0,0.12))',
        }
      )
    );

    // Label text
    elements.push(
      createText(
        isSelfTransition ? midX + labelWidth / 2 : midX,
        midY,
        edge.label,
        {
          'text-anchor': 'middle',
          'dominant-baseline': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': fontSize,
          'font-weight': '600',
          fill: transitionColor,
        }
      )
    );
  }

  return createGroup(elements, { id: edge.id });
}
