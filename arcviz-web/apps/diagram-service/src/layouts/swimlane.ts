/**
 * Swimlane Layout Algorithm
 * 
 * Lays out operational activities in horizontal swimlanes grouped by actor/entity.
 * Implements automatic lane sizing, activity positioning, and flow routing.
 */

import {
  DiagramNode,
  DiagramEdge,
  Point,
  Size,
  Swimlane,
  SwimlaneLayoutConfig,
  Padding,
} from '../types/diagram';

// ============================================================================
// Configuration
// ============================================================================

const DEFAULT_CONFIG: Required<SwimlaneLayoutConfig> = {
  direction: 'RIGHT',
  swimlaneWidth: 200,
  swimlaneSpacing: 20,
  activityHeight: 80,
  activityWidth: 150,
  nodeSpacing: 40,
  layerSpacing: 100,
  edgeSpacing: 10,
  padding: {
    top: 60,
    right: 40,
    bottom: 40,
    left: 200,
  },
  algorithm: 'swimlane',
};

// ============================================================================
// Types
// ============================================================================

interface LayoutNode extends DiagramNode {
  position: Point;
  size: Size;
  swimlaneId: string;
  layer: number;
}

interface LayoutEdge extends DiagramEdge {
  points: Point[];
}

interface LayoutResult {
  nodes: LayoutNode[];
  edges: LayoutEdge[];
  swimlanes: Swimlane[];
  totalSize: Size;
}

// ============================================================================
// Main Layout Function
// ============================================================================

/**
 * Apply swimlane layout to operational activity nodes and edges
 */
export function applySwimlaneLayout(
  nodes: DiagramNode[],
  edges: DiagramEdge[],
  config: Partial<SwimlaneLayoutConfig> = {}
): LayoutResult {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  // 1. Group nodes by swimlane (performed_by)
  const swimlaneGroups = groupNodesBySwimlane(nodes);

  // 2. Create swimlanes
  const swimlanes = createSwimlanes(swimlaneGroups, cfg);

  // 3. Assign nodes to layers (horizontal positioning)
  const layeredNodes = assignNodesToLayers(nodes, edges);

  // 4. Position nodes within swimlanes
  const positionedNodes = positionNodesInSwimlanes(layeredNodes, swimlanes, cfg);

  // 5. Route edges between nodes
  const routedEdges = routeEdges(edges, positionedNodes, cfg);

  // 6. Calculate total diagram size
  const totalSize = calculateTotalSize(positionedNodes, swimlanes, cfg);

  return {
    nodes: positionedNodes,
    edges: routedEdges,
    swimlanes,
    totalSize,
  };
}

// ============================================================================
// Step 1: Group Nodes by Swimlane
// ============================================================================

interface SwimlaneGroup {
  swimlaneId: string;
  label: string;
  nodes: DiagramNode[];
}

function groupNodesBySwimlane(nodes: DiagramNode[]): SwimlaneGroup[] {
  const groups = new Map<string, SwimlaneGroup>();

  for (const node of nodes) {
    // Extract swimlane ID and label from metadata
    const swimlaneId = node.metadata?.swimlane || node.metadata?.performed_by || 'default';
    const label = node.metadata?.swimlaneLabel || swimlaneId;

    if (!groups.has(swimlaneId)) {
      groups.set(swimlaneId, {
        swimlaneId,
        label,
        nodes: [],
      });
    }

    groups.get(swimlaneId)!.nodes.push(node);
  }

  return Array.from(groups.values());
}

// ============================================================================
// Step 2: Create Swimlanes
// ============================================================================

function createSwimlanes(
  groups: SwimlaneGroup[],
  config: Required<SwimlaneLayoutConfig>
): Swimlane[] {
  const swimlanes: Swimlane[] = [];
  let currentY = config.padding.top;

  for (const group of groups) {
    // Calculate swimlane height based on number of nodes
    const rowCount = Math.ceil(group.nodes.length / 4); // Max 4 activities per row
    const height = Math.max(
      config.activityHeight + 40,
      rowCount * (config.activityHeight + config.nodeSpacing)
    );

    swimlanes.push({
      id: group.swimlaneId,
      label: group.label,
      activityIds: group.nodes.map(n => n.id),
      position: { x: 0, y: currentY },
      size: { width: 0, height }, // Width calculated later
    });

    currentY += height + config.swimlaneSpacing;
  }

  return swimlanes;
}

// ============================================================================
// Step 3: Assign Nodes to Layers
// ============================================================================

interface LayeredNode extends DiagramNode {
  layer: number;
  swimlaneId: string;
}

function assignNodesToLayers(
  nodes: DiagramNode[],
  edges: DiagramEdge[]
): LayeredNode[] {
  const layeredNodes: LayeredNode[] = [];
  const nodeToLayer = new Map<string, number>();

  // Build adjacency list
  const outgoing = new Map<string, string[]>();
  for (const edge of edges) {
    if (!outgoing.has(edge.from)) {
      outgoing.set(edge.from, []);
    }
    outgoing.get(edge.from)!.push(edge.to);
  }

  // Find root nodes (no incoming edges)
  const incoming = new Set(edges.map(e => e.to));
  const rootNodes = nodes.filter(n => !incoming.has(n.id));

  // BFS to assign layers
  const queue: { nodeId: string; layer: number }[] = rootNodes.map(n => ({
    nodeId: n.id,
    layer: 0,
  }));
  const visited = new Set<string>();

  while (queue.length > 0) {
    const { nodeId, layer } = queue.shift()!;

    if (visited.has(nodeId)) continue;
    visited.add(nodeId);

    nodeToLayer.set(nodeId, layer);

    const children = outgoing.get(nodeId) || [];
    for (const childId of children) {
      if (!visited.has(childId)) {
        queue.push({ nodeId: childId, layer: layer + 1 });
      }
    }
  }

  // Handle disconnected nodes
  for (const node of nodes) {
    if (!nodeToLayer.has(node.id)) {
      nodeToLayer.set(node.id, 0);
    }
  }

  // Create layered nodes
  for (const node of nodes) {
    layeredNodes.push({
      ...node,
      layer: nodeToLayer.get(node.id) || 0,
      swimlaneId: node.metadata?.swimlane || node.metadata?.performed_by || 'default',
    });
  }

  return layeredNodes;
}

// ============================================================================
// Step 4: Position Nodes in Swimlanes
// ============================================================================

function positionNodesInSwimlanes(
  layeredNodes: LayeredNode[],
  swimlanes: Swimlane[],
  config: Required<SwimlaneLayoutConfig>
): LayoutNode[] {
  const positionedNodes: LayoutNode[] = [];
  const swimlaneMap = new Map(swimlanes.map(s => [s.id, s]));

  // Group by swimlane and layer
  const groups = new Map<string, Map<number, LayeredNode[]>>();

  for (const node of layeredNodes) {
    if (!groups.has(node.swimlaneId)) {
      groups.set(node.swimlaneId, new Map());
    }
    const swimlaneGroup = groups.get(node.swimlaneId)!;

    if (!swimlaneGroup.has(node.layer)) {
      swimlaneGroup.set(node.layer, []);
    }
    swimlaneGroup.get(node.layer)!.push(node);
  }

  // Track how many nodes already placed in each swimlane
  const swimlaneNodeCounts = new Map<string, number>();
  
  // Sort nodes by swimlane to ensure consistent positioning
  const sortedNodes = [...layeredNodes].sort((a, b) => {
    if (a.swimlaneId !== b.swimlaneId) {
      return a.swimlaneId.localeCompare(b.swimlaneId);
    }
    return a.layer - b.layer;
  });
  
  // Position each node
  for (const node of sortedNodes) {
    const swimlane = swimlaneMap.get(node.swimlaneId);
    if (!swimlane) continue;

    // Get count of nodes already placed in this swimlane (for vertical positioning)
    const indexInSwimlane = swimlaneNodeCounts.get(node.swimlaneId) || 0;
    swimlaneNodeCounts.set(node.swimlaneId, indexInSwimlane + 1);

    // Calculate position
    const x = config.padding.left + node.layer * (config.activityWidth + config.layerSpacing);
    const y =
      swimlane.position!.y +
      30 + // Swimlane header
      indexInSwimlane * (config.activityHeight + config.nodeSpacing);

    positionedNodes.push({
      ...node,
      position: { x, y },
      size: { width: config.activityWidth, height: config.activityHeight },
    });
  }

  // Update swimlane widths based on max layer
  const maxLayer = Math.max(...layeredNodes.map(n => n.layer), 0);
  const totalWidth =
    config.padding.left +
    config.padding.right +
    (maxLayer + 1) * config.activityWidth +
    maxLayer * config.layerSpacing;

  for (const swimlane of swimlanes) {
    swimlane.size!.width = totalWidth;
  }

  return positionedNodes;
}

// ============================================================================
// Step 5: Route Edges
// ============================================================================

function routeEdges(
  edges: DiagramEdge[],
  nodes: LayoutNode[],
  config: Required<SwimlaneLayoutConfig>
): LayoutEdge[] {
  const routedEdges: LayoutEdge[] = [];
  const nodeMap = new Map(nodes.map(n => [n.id, n]));

  // Group edges by source node to detect parallel edges
  const edgesBySource = new Map<string, DiagramEdge[]>();
  for (const edge of edges) {
    if (!edgesBySource.has(edge.from)) {
      edgesBySource.set(edge.from, []);
    }
    edgesBySource.get(edge.from)!.push(edge);
  }

  for (const edge of edges) {
    const fromNode = nodeMap.get(edge.from);
    const toNode = nodeMap.get(edge.to);

    if (!fromNode || !toNode) {
      console.warn(`Edge ${edge.id} references unknown nodes`);
      continue;
    }

    // Calculate connection points
    const fromPoint = {
      x: fromNode.position.x + fromNode.size.width,
      y: fromNode.position.y + fromNode.size.height / 2,
    };

    const toPoint = {
      x: toNode.position.x,
      y: toNode.position.y + toNode.size.height / 2,
    };

    // Calculate midX with offset for parallel edges
    const sourceSiblings = edgesBySource.get(edge.from) || [];
    const siblingIndex = sourceSiblings.indexOf(edge);
    const totalSiblings = sourceSiblings.length;
    
    let midX = (fromPoint.x + toPoint.x) / 2;
    
    // Add horizontal offset for multiple edges from same source
    if (totalSiblings > 1) {
      const offsetRange = 30 * (totalSiblings - 1);
      const offset = (siblingIndex - (totalSiblings - 1) / 2) * 30;
      midX += offset;
    }

    // Simple orthogonal routing
    const points: Point[] = [];

    if (fromNode.swimlaneId === toNode.swimlaneId) {
      // Same swimlane - direct connection with midpoint
      points.push(fromPoint);
      
      points.push({ x: midX, y: fromPoint.y });
      points.push({ x: midX, y: toPoint.y });
      
      points.push(toPoint);
    } else {
      // Different swimlanes - route around
      points.push(fromPoint);
      
      points.push({ x: midX, y: fromPoint.y });
      points.push({ x: midX, y: toPoint.y });
      
      points.push(toPoint);
    }

    routedEdges.push({
      ...edge,
      points,
    });
  }

  return routedEdges;
}

// ============================================================================
// Step 6: Calculate Total Size
// ============================================================================

function calculateTotalSize(
  nodes: LayoutNode[],
  swimlanes: Swimlane[],
  config: Required<SwimlaneLayoutConfig>
): Size {
  const maxX = Math.max(
    ...nodes.map(n => n.position.x + n.size.width),
    0
  );

  const maxY = Math.max(
    ...swimlanes.map(s => s.position!.y + s.size!.height),
    0
  );

  return {
    width: maxX + config.padding.right,
    height: maxY + config.padding.bottom,
  };
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Get node center point
 */
export function getNodeCenter(node: LayoutNode): Point {
  return {
    x: node.position.x + node.size.width / 2,
    y: node.position.y + node.size.height / 2,
  };
}

/**
 * Get node connection point for a given direction
 */
export function getNodeConnectionPoint(
  node: LayoutNode,
  direction: 'TOP' | 'RIGHT' | 'BOTTOM' | 'LEFT'
): Point {
  const center = getNodeCenter(node);

  switch (direction) {
    case 'TOP':
      return { x: center.x, y: node.position.y };
    case 'RIGHT':
      return { x: node.position.x + node.size.width, y: center.y };
    case 'BOTTOM':
      return { x: center.x, y: node.position.y + node.size.height };
    case 'LEFT':
      return { x: node.position.x, y: center.y };
  }
}
