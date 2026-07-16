/**
 * Timeline Layout Algorithm
 * 
 * Lays out sequence diagrams with vertical lifelines and chronological message ordering.
 * Supports combined fragments (PAR, OPT, LOOP, ALT) and activation bars.
 */

import {
  DiagramNode,
  DiagramEdge,
  Point,
  Size,
  TimelineLayoutConfig,
  DiagramFragment,
  DiagramFragmentOperand,
} from '../types/diagram';

// ============================================================================
// Configuration
// ============================================================================

const DEFAULT_CONFIG: Required<TimelineLayoutConfig> = {
  direction: 'DOWN',
  lifelineSpacing: 200,
  messageSpacing: 60,
  nodeSpacing: 40,
  layerSpacing: 100,
  edgeSpacing: 10,
  fragmentPadding: {
    top: 30,
    right: 20,
    bottom: 20,
    left: 20,
  },
  activationWidth: 20,
  padding: {
    top: 100,
    right: 40,
    bottom: 40,
    left: 40,
  },
  algorithm: 'timeline',
};

// ============================================================================
// Types
// ============================================================================

interface LayoutNode extends DiagramNode {
  position: Point;
  size: Size;
  lifelineX?: number;
}

interface LayoutEdge extends DiagramEdge {
  points: Point[];
  yLevel?: number;
}

interface LayoutFragment extends DiagramFragment {
  position: Point;
  size: Size;
}

interface LayoutResult {
  nodes: LayoutNode[];
  edges: LayoutEdge[];
  fragments: LayoutFragment[];
  totalSize: Size;
}

// ============================================================================
// Main Layout Function
// ============================================================================

/**
 * Apply timeline layout for sequence diagrams
 */
export function applyTimelineLayout(
  nodes: DiagramNode[],
  edges: DiagramEdge[],
  fragments: DiagramFragment[] = [],
  config: Partial<TimelineLayoutConfig> = {}
): LayoutResult {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  // 1. Position participants (lifelines)
  const layoutNodes = positionParticipants(nodes, cfg);

  // 2. Order messages chronologically and assign Y positions
  const layoutEdges = positionMessages(edges, layoutNodes, cfg);

  // 3. Position combined fragments
  const layoutFragments = positionFragments(fragments, layoutEdges, layoutNodes, cfg);

  // 4. Calculate total size
  const totalSize = calculateTotalSize(layoutNodes, layoutEdges, layoutFragments, cfg);

  return {
    nodes: layoutNodes,
    edges: layoutEdges,
    fragments: layoutFragments,
    totalSize,
  };
}

// ============================================================================
// Step 1: Position Participants (Lifelines)
// ============================================================================

function positionParticipants(
  nodes: DiagramNode[],
  config: Required<TimelineLayoutConfig>
): LayoutNode[] {
  const layoutNodes: LayoutNode[] = [];
  
  let currentX = config.padding.left;

  for (const node of nodes) {
    const width = node.size?.width || 120;
    const height = node.size?.height || 60;

    // Center the participant box above the lifeline
    const lifelineX = currentX + width / 2;

    layoutNodes.push({
      ...node,
      position: {
        x: currentX,
        y: config.padding.top,
      },
      size: {
        width,
        height,
      },
      lifelineX,
    });

    currentX += width + config.lifelineSpacing;
  }

  return layoutNodes;
}

// ============================================================================
// Step 2: Position Messages
// ============================================================================

function positionMessages(
  edges: DiagramEdge[],
  nodes: LayoutNode[],
  config: Required<TimelineLayoutConfig>
): LayoutEdge[] {
  const layoutEdges: LayoutEdge[] = [];
  const nodeMap = new Map(nodes.map(n => [n.id, n]));

  // Start messages below participant boxes
  let currentY = config.padding.top + 60 + config.nodeSpacing;

  for (const edge of edges) {
    const fromNode = nodeMap.get(edge.from);
    const toNode = nodeMap.get(edge.to);

    if (!fromNode || !toNode) {
      console.warn(`Edge ${edge.id} references unknown nodes`);
      continue;
    }

    const fromX = fromNode.lifelineX!;
    const toX = toNode.lifelineX!;

    // Check if this is a self-call (same participant)
    const isSelfCall = edge.from === edge.to;

    let points: Point[];
    
    if (isSelfCall) {
      // Create a loop arrow for self-calls
      const loopWidth = 60;
      const loopHeight = 40;
      
      points = [
        { x: fromX, y: currentY },
        { x: fromX + loopWidth, y: currentY },
        { x: fromX + loopWidth, y: currentY + loopHeight },
        { x: fromX, y: currentY + loopHeight },
      ];
      
      // Self-calls need more vertical space
      currentY += loopHeight + config.messageSpacing;
    } else {
      // Regular message line
      points = [
        { x: fromX, y: currentY },
        { x: toX, y: currentY },
      ];
      
      currentY += config.messageSpacing;
    }

    layoutEdges.push({
      ...edge,
      points,
      yLevel: currentY - (isSelfCall ? config.messageSpacing : 0),
    });
  }

  return layoutEdges;
}

// ============================================================================
// Step 3: Position Combined Fragments
// ============================================================================

function positionFragments(
  fragments: DiagramFragment[],
  edges: LayoutEdge[],
  nodes: LayoutNode[],
  config: Required<TimelineLayoutConfig>
): LayoutFragment[] {
  const layoutFragments: LayoutFragment[] = [];

  for (const fragment of fragments) {
    // Find messages within this fragment
    const fragmentMessageIds = fragment.operands.flatMap(op => op.messageIds);
    const fragmentEdges = edges.filter(e => fragmentMessageIds.includes(e.id));

    if (fragmentEdges.length === 0) continue;

    // Calculate fragment bounds
    const minY = Math.min(...fragmentEdges.map(e => e.yLevel!));
    const maxY = Math.max(...fragmentEdges.map(e => e.yLevel!));

    // Find leftmost and rightmost participants involved
    const involvedNodeIds = new Set<string>();
    for (const edge of fragmentEdges) {
      involvedNodeIds.add(edge.from);
      involvedNodeIds.add(edge.to);
    }

    const involvedNodes = nodes.filter(n => involvedNodeIds.has(n.id));
    const minX = Math.min(...involvedNodes.map(n => n.position.x));
    const maxX = Math.max(...involvedNodes.map(n => n.position.x + n.size.width));

    layoutFragments.push({
      ...fragment,
      position: {
        x: minX - config.fragmentPadding.left,
        y: minY - config.fragmentPadding.top,
      },
      size: {
        width: maxX - minX + config.fragmentPadding.left + config.fragmentPadding.right,
        height: maxY - minY + config.fragmentPadding.top + config.fragmentPadding.bottom + config.messageSpacing,
      },
    });
  }

  return layoutFragments;
}

// ============================================================================
// Step 4: Calculate Total Size
// ============================================================================

function calculateTotalSize(
  nodes: LayoutNode[],
  edges: LayoutEdge[],
  fragments: LayoutFragment[],
  config: Required<TimelineLayoutConfig>
): Size {
  const maxX = Math.max(
    ...nodes.map(n => n.position.x + n.size.width),
    ...fragments.map(f => f.position.x + f.size.width),
    0
  );

  const maxY = Math.max(
    ...edges.map(e => e.yLevel || 0),
    ...fragments.map(f => f.position.y + f.size.height),
    config.padding.top + 200
  );

  return {
    width: maxX + config.padding.right,
    height: maxY + config.padding.bottom + 100, // Extra space for lifeline endings
  };
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Calculate activation bar positions for a participant
 */
export function calculateActivations(
  participantId: string,
  edges: LayoutEdge[],
  participantX: number,
  activationWidth: number
): Array<{ x: number; y: number; height: number }> {
  const activations: Array<{ x: number; y: number; height: number }> = [];
  
  // Find messages where this participant is the target
  const incomingMessages = edges
    .filter(e => e.to === participantId && e.metadata?.activation)
    .sort((a, b) => (a.yLevel || 0) - (b.yLevel || 0));

  for (const msg of incomingMessages) {
    // Find corresponding return message or next message
    const msgIndex = edges.indexOf(msg);
    const returnMsg = edges
      .slice(msgIndex + 1)
      .find(e => e.from === participantId && e.type === 'message-return');

    const startY = msg.yLevel!;
    const endY = returnMsg?.yLevel || (msg.yLevel! + 100);

    activations.push({
      x: participantX - activationWidth / 2,
      y: startY,
      height: endY - startY,
    });
  }

  return activations;
}

/**
 * Get arrow style based on message type
 */
export function getMessageArrowStyle(messageType: string): {
  strokeDasharray?: string;
  markerEnd: string;
} {
  switch (messageType) {
    case 'message-async':
      return {
        markerEnd: 'url(#arrow-open)',
      };
    case 'message-return':
      return {
        strokeDasharray: '5,5',
        markerEnd: 'url(#arrow-black)',
      };
    case 'message-sync':
    default:
      return {
        markerEnd: 'url(#arrow-filled)',
      };
  }
}

/**
 * Calculate fragment operand divider positions
 */
export function calculateOperandDividers(
  fragment: LayoutFragment,
  operandCount: number
): Point[][] {
  const dividers: Point[][] = [];
  
  if (operandCount <= 1) return dividers;

  const operandHeight = fragment.size.height / operandCount;
  
  for (let i = 1; i < operandCount; i++) {
    const y = fragment.position.y + i * operandHeight;
    dividers.push([
      { x: fragment.position.x, y },
      { x: fragment.position.x + fragment.size.width, y },
    ]);
  }

  return dividers;
}
