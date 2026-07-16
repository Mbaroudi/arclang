/**
 * State Machine Diagram Renderer
 * 
 * Renders UML-style state machine diagrams with:
 * - Initial and final states
 * - Regular states with entry/exit/do activities
 * - Composite states (nested states)
 * - Orthogonal regions
 * - State transitions with guards and actions
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
  NodeType,
  SvgElement,
  RenderConfig,
  DiagramOutput,
  CAPELLA_COLORS,
} from '../types/diagram';
import {
  applyStateGraphLayout,
  calculateTransitionLabelPosition,
  isSelfTransition,
} from '../layouts/state-graph';
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
 * Render state machine diagram
 */
export async function renderStateMachine(
  stateMachine: StateMachine,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg: RenderConfig = { ...DEFAULT_CONFIG, ...config };

  // 1. Convert model to diagram nodes and edges
  const { nodes, edges } = convertToDiagram(stateMachine);

  // 2. Apply state graph layout
  const layout = await applyStateGraphLayout(nodes, edges, {
    direction: 'RIGHT',
    nodeSpacing: 80,
    layerSpacing: 120,
  });

  // 3. Render to SVG
  const svg = renderToSvg(stateMachine, layout, cfg);

  // 4. Return result
  const width = layout.totalSize.width;
  const height = layout.totalSize.height;

  return {
    svg,
    width,
    height,
    metadata: {
      diagramType: 'state-machine',
      stateCount: stateMachine.states.length,
      transitionCount: stateMachine.transitions.length,
    },
  };
}

// ============================================================================
// Step 1: Convert Model to Diagram
// ============================================================================

function convertToDiagram(stateMachine: StateMachine): {
  nodes: DiagramNode[];
  edges: DiagramEdge[];
} {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  // Add initial state
  nodes.push({
    id: stateMachine.initial_state,
    label: '',
    type: 'initial-state',
  });

  // Convert states to nodes
  for (const state of stateMachine.states) {
    const isComposite = state.sub_states && state.sub_states.length > 0;
    nodes.push({
      id: state.name,
      label: state.name,
      type: isComposite ? 'composite-state' : 'state',
      metadata: {
        entry: state.entry_actions.length > 0 ? state.entry_actions.join(', ') : undefined,
        exit: state.exit_actions.length > 0 ? state.exit_actions.join(', ') : undefined,
        doActivity: state.internal_transitions.length > 0 ? state.internal_transitions.join(', ') : undefined,
        isComposite,
      },
      children: state.sub_states && state.sub_states.length > 0 ? convertSubStates(state.sub_states) : undefined,
    });
  }

  // Convert transitions to edges
  for (let i = 0; i < stateMachine.transitions.length; i++) {
    const transition = stateMachine.transitions[i];
    
    // Build transition label
    const labelParts = [];
    if (transition.trigger) labelParts.push(transition.trigger);
    if (transition.guard) labelParts.push(`[${transition.guard}]`);
    if (transition.action) labelParts.push(`/ ${transition.action}`);
    const label = labelParts.join(' ');

    edges.push({
      id: `trans-${i}`,
      from: transition.from,
      to: transition.to,
      label: label || undefined,
      type: 'transition',
      metadata: {
        trigger: transition.trigger,
        guard: transition.guard,
        action: transition.action,
        priority: transition.priority,
      },
    });
  }

  return { nodes, edges };
}


function convertSubStates(subStates: State[]): DiagramNode[] {
  return subStates.map(state => ({
    id: state.name,
    label: state.name,
    type: (state.sub_states && state.sub_states.length > 0) ? 'composite-state' : 'state' as NodeType,
    metadata: {
      entry: state.entry_actions.length > 0 ? state.entry_actions.join(', ') : undefined,
      exit: state.exit_actions.length > 0 ? state.exit_actions.join(', ') : undefined,
      doActivity: state.internal_transitions.length > 0 ? state.internal_transitions.join(', ') : undefined,
    },
  }));
}

// ============================================================================
// Step 2: Render to SVG
// ============================================================================

function renderToSvg(
  stateMachine: StateMachine,
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

  // Render transitions first (behind states)
  for (const edge of layout.edges) {
    elements.push(renderTransition(edge, layout.nodes, config));
  }

  // Render states
  for (const node of layout.nodes) {
    elements.push(renderState(node, config));
  }

  // Title
  if (stateMachine.name) {
    elements.push(
      createText(20, 30, stateMachine.name, {
        'font-family': 'Arial, sans-serif',
        'font-size': 18,
        'font-weight': 'bold',
        fill: '#212529',
      })
    );
  }

  // Create arrow markers
  const defs = [
    createArrowMarker('arrow-black', '#000000', 10),
    createArrowMarker('arrow-state', '#000000', 10),
  ];

  return createSvgDocument(layout.totalSize.width, layout.totalSize.height, elements, defs);
}

// ============================================================================
// Render Individual Elements
// ============================================================================

function renderState(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  // Render based on state type
  switch (node.type) {
    case 'initial-state':
      // Small filled circle
      elements.push(
        createCircle(
          node.position.x + node.size.width / 2,
          node.position.y + node.size.height / 2,
          15,
          {
            fill: '#000000',
            stroke: 'none',
          }
        )
      );
      break;

    case 'final-state':
      // Double circle (outer + inner)
      const centerX = node.position.x + node.size.width / 2;
      const centerY = node.position.y + node.size.height / 2;
      elements.push(
        createCircle(centerX, centerY, 15, {
          fill: '#FFFFFF',
          stroke: '#000000',
          'stroke-width': 2,
        })
      );
      elements.push(
        createCircle(centerX, centerY, 10, {
          fill: '#000000',
          stroke: 'none',
        })
      );
      break;

    case 'choice':
    case 'junction':
      // Diamond shape
      const cx = node.position.x + node.size.width / 2;
      const cy = node.position.y + node.size.height / 2;
      const size = 20;
      const pathD = `M ${cx} ${cy - size} L ${cx + size} ${cy} L ${cx} ${cy + size} L ${cx - size} ${cy} Z`;
      elements.push(
        createPath(pathD, {
          fill: node.type === 'choice' ? '#FFFFFF' : '#000000',
          stroke: '#000000',
          'stroke-width': 2,
        })
      );
      break;

    case 'composite-state':
    case 'state':
    default:
      const safetyData = parseSafetyLevel(node.metadata);
      const safetyLevel = safetyData.level;
      const safetyStandard = safetyData.standard;
      const hasSafetyLevel = safetyLevel !== null;

      const stateAttrs: Record<string, string> = {
        fill: config.colorScheme?.state || '#BDD7EE',
      };

      if (hasSafetyLevel && safetyLevel) {
        const safetyAttrs = getSafetyBorderAttributes(safetyLevel, safetyStandard || undefined);
        Object.assign(stateAttrs, safetyAttrs);
      } else {
        stateAttrs.stroke = '#000000';
        stateAttrs['stroke-width'] = '2';
      }

      elements.push(
        createRoundedRect(
          node.position.x,
          node.position.y,
          node.size.width,
          node.size.height,
          10,
          stateAttrs
        )
      );

      // State name
      elements.push(
        createText(
          node.position.x + node.size.width / 2,
          node.position.y + 20,
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

      // State activities (entry, do, exit)
      let yOffset = 40;
      if (node.metadata?.entry) {
        elements.push(
          createText(
            node.position.x + 10,
            node.position.y + yOffset,
            `entry / ${node.metadata.entry}`,
            {
              'font-family': 'Arial, sans-serif',
              'font-size': 9,
              'font-style': 'italic',
              fill: '#495057',
            }
          )
        );
        yOffset += 15;
      }

      if (node.metadata?.doActivity) {
        elements.push(
          createText(
            node.position.x + 10,
            node.position.y + yOffset,
            `do / ${node.metadata.doActivity}`,
            {
              'font-family': 'Arial, sans-serif',
              'font-size': 9,
              'font-style': 'italic',
              fill: '#495057',
            }
          )
        );
        yOffset += 15;
      }

      if (node.metadata?.exit) {
        elements.push(
          createText(
            node.position.x + 10,
            node.position.y + yOffset,
            `exit / ${node.metadata.exit}`,
            {
              'font-family': 'Arial, sans-serif',
              'font-size': 9,
              'font-style': 'italic',
              fill: '#495057',
            }
          )
        );
      }

      // Separator line after state name
      if (node.metadata?.entry || node.metadata?.doActivity || node.metadata?.exit) {
        elements.push(
          createLine(
            node.position.x,
            node.position.y + 30,
            node.position.x + node.size.width,
            node.position.y + 30,
            {
              stroke: '#000000',
              'stroke-width': 1,
            }
          )
        );
      }
      break;
  }

  return createGroup(elements, { id: `state-${node.id}` });
}

function renderTransition(edge: any, nodes: any[], config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  if (!edge.points || edge.points.length < 2) {
    return createGroup([]);
  }

  // Check if self-transition
  const fromNode = nodes.find(n => n.id === edge.from);
  const isSelf = isSelfTransition(edge);

  if (isSelf && fromNode) {
    // Self-transition: arc above the state
    const x = fromNode.position.x + fromNode.size.width / 2;
    const y = fromNode.position.y;
    const loopRadius = 30;

    const pathD = `M ${x - loopRadius} ${y} 
                    Q ${x - loopRadius} ${y - loopRadius * 2} ${x} ${y - loopRadius * 2} 
                    Q ${x + loopRadius} ${y - loopRadius * 2} ${x + loopRadius} ${y}`;

    elements.push(
      createPath(pathD, {
        fill: 'none',
        stroke: '#000000',
        'stroke-width': 2,
        'marker-end': 'url(#arrow-black)',
      })
    );

    // Label above the loop
    if (edge.label && config.showLabels) {
      elements.push(
        createText(x, y - loopRadius * 2 - 10, edge.label, {
          'text-anchor': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 9,
          fill: '#000000',
        })
      );
    }
  } else {
    // Regular transition: curved line through points
    const pathD = generateSmoothPath(edge.points);

    elements.push(
      createPath(pathD, {
        fill: 'none',
        stroke: '#000000',
        'stroke-width': 2,
        'marker-end': 'url(#arrow-black)',
      })
    );

    // Transition label
    if (edge.label && config.showLabels) {
      const labelPos = calculateTransitionLabelPosition(edge.points);
      
      // Improved label background sizing
      const fontSize = (config.fontSize || 12) - 1;
      const labelWidth = Math.max(edge.label.length * (fontSize * 0.6), 80);
      const labelHeight = 20;
      const padding = 6;

      // Background for label
      elements.push(
        createRoundedRect(
          labelPos.x - labelWidth / 2 - padding,
          labelPos.y - labelHeight / 2,
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
        createText(labelPos.x, labelPos.y, edge.label, {
          'text-anchor': 'middle',
          'dominant-baseline': 'middle',
          'font-family': config.fontFamily || 'Arial, sans-serif',
          'font-size': fontSize,
          fill: '#495057',
        })
      );
    }
  }

  return createGroup(elements, { id: edge.id });
}

function generateSmoothPath(points: Point[]): string {
  if (points.length === 0) return '';
  if (points.length === 1) return `M ${points[0].x} ${points[0].y}`;
  if (points.length === 2) {
    return `M ${points[0].x} ${points[0].y} L ${points[1].x} ${points[1].y}`;
  }

  // Smooth curve through multiple points using quadratic bezier
  let path = `M ${points[0].x} ${points[0].y}`;

  for (let i = 1; i < points.length; i++) {
    if (i === points.length - 1) {
      // Last point - straight line
      path += ` L ${points[i].x} ${points[i].y}`;
    } else {
      // Smooth curve
      const current = points[i];
      const next = points[i + 1];
      const controlX = current.x;
      const controlY = current.y;
      const endX = (current.x + next.x) / 2;
      const endY = (current.y + next.y) / 2;
      path += ` Q ${controlX} ${controlY} ${endX} ${endY}`;
    }
  }

  return path;
}

// ============================================================================
// Export
// ============================================================================

export default renderStateMachine;
