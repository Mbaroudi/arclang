/**
 * State Graph Layout Algorithm
 * 
 * Hierarchical layout for state machines with nested states and orthogonal regions.
 * Uses ELK for automatic positioning of states and transitions.
 */

import ELK, { ElkNode, ElkExtendedEdge } from 'elkjs/lib/elk.bundled';
import {
  DiagramNode,
  DiagramEdge,
  Point,
  Size,
  LayoutConfig,
} from '../types/diagram';

// ============================================================================
// Configuration
// ============================================================================

const DEFAULT_CONFIG: Required<LayoutConfig> = {
  direction: 'RIGHT',
  nodeSpacing: 80,
  layerSpacing: 100,
  edgeSpacing: 20,
  padding: {
    top: 60,
    right: 60,
    bottom: 60,
    left: 60,
  },
  algorithm: 'elk',
};

// ============================================================================
// Types
// ============================================================================

interface LayoutNode extends DiagramNode {
  position: Point;
  size: Size;
}

interface LayoutEdge extends DiagramEdge {
  points: Point[];
}

interface LayoutResult {
  nodes: LayoutNode[];
  edges: LayoutEdge[];
  totalSize: Size;
}

// ============================================================================
// Main Layout Function
// ============================================================================

/**
 * Apply state graph layout using ELK
 */
export async function applyStateGraphLayout(
  nodes: DiagramNode[],
  edges: DiagramEdge[],
  config: Partial<LayoutConfig> = {}
): Promise<LayoutResult> {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  // 1. Convert to ELK format
  const elkGraph = convertToElkGraph(nodes, edges, cfg);

  // 2. Run ELK layout
  const elk = new ELK();
  const laidOutGraph = await elk.layout(elkGraph);

  // 3. Convert back to our format
  const layoutNodes = extractLayoutNodes(laidOutGraph, nodes);
  const layoutEdges = extractLayoutEdges(laidOutGraph, edges);

  // 4. Calculate total size
  const totalSize = calculateTotalSize(laidOutGraph, cfg);

  return {
    nodes: layoutNodes,
    edges: layoutEdges,
    totalSize,
  };
}

// ============================================================================
// Step 1: Convert to ELK Graph
// ============================================================================

function convertToElkGraph(
  nodes: DiagramNode[],
  edges: DiagramEdge[],
  config: Required<LayoutConfig>
): ElkNode {
  const elkNodes: ElkNode[] = [];
  const elkEdges: ElkExtendedEdge[] = [];

  // Convert nodes
  for (const node of nodes) {
    const width = node.size?.width || (node.type === 'initial-state' || node.type === 'final-state' ? 30 : 150);
    const height = node.size?.height || (node.type === 'initial-state' || node.type === 'final-state' ? 30 : 80);

    const elkNode: ElkNode = {
      id: node.id,
      width,
      height,
      labels: node.label ? [{ text: node.label }] : [],
    };

    // Handle children (nested states)
    if (node.children && node.children.length > 0) {
      const childGraph = convertToElkGraph(node.children, [], config);
      elkNode.children = childGraph.children;
      elkNode.edges = childGraph.edges;
      // Increase size for composite states
      elkNode.width = Math.max(width, 200);
      elkNode.height = Math.max(height, 150);
    }

    elkNodes.push(elkNode);
  }

  // Convert edges (transitions)
  for (const edge of edges) {
    const elkEdge: ElkExtendedEdge = {
      id: edge.id,
      sources: [edge.from],
      targets: [edge.to],
      labels: edge.label ? [{ text: edge.label }] : [],
    };

    elkEdges.push(elkEdge);
  }

  // Create root graph
  return {
    id: 'root',
    children: elkNodes,
    edges: elkEdges,
    layoutOptions: {
      'elk.algorithm': 'layered',
      'elk.direction': config.direction,
      'elk.spacing.nodeNode': String(config.nodeSpacing),
      'elk.layered.spacing.nodeNodeBetweenLayers': String(config.layerSpacing),
      'elk.spacing.edgeNode': String(config.edgeSpacing),
      'elk.padding': `[top=${config.padding.top},left=${config.padding.left},bottom=${config.padding.bottom},right=${config.padding.right}]`,
      'elk.edgeRouting': 'SPLINES',
      'elk.layered.nodePlacement.strategy': 'NETWORK_SIMPLEX',
      'elk.hierarchyHandling': 'INCLUDE_CHILDREN',
      'elk.layered.crossingMinimization.strategy': 'LAYER_SWEEP',
    },
  };
}

// ============================================================================
// Step 2: Extract Layout Nodes
// ============================================================================

function extractLayoutNodes(
  elkGraph: ElkNode,
  originalNodes: DiagramNode[]
): LayoutNode[] {
  const layoutNodes: LayoutNode[] = [];
  const nodeMap = new Map(originalNodes.map(n => [n.id, n]));

  function processNode(elkNode: ElkNode, offsetX: number = 0, offsetY: number = 0) {
    const originalNode = nodeMap.get(elkNode.id);
    if (!originalNode) return;

    const position: Point = {
      x: (elkNode.x || 0) + offsetX,
      y: (elkNode.y || 0) + offsetY,
    };

    const size: Size = {
      width: elkNode.width || 150,
      height: elkNode.height || 80,
    };

    layoutNodes.push({
      ...originalNode,
      position,
      size,
    });

    // Process children recursively
    if (elkNode.children) {
      for (const child of elkNode.children) {
        processNode(child, position.x, position.y);
      }
    }
  }

  if (elkGraph.children) {
    for (const child of elkGraph.children) {
      processNode(child);
    }
  }

  return layoutNodes;
}

// ============================================================================
// Step 3: Extract Layout Edges
// ============================================================================

function extractLayoutEdges(
  elkGraph: ElkNode,
  originalEdges: DiagramEdge[]
): LayoutEdge[] {
  const layoutEdges: LayoutEdge[] = [];
  const edgeMap = new Map(originalEdges.map(e => [e.id, e]));

  if (elkGraph.edges) {
    for (const elkEdge of elkGraph.edges) {
      const originalEdge = edgeMap.get(elkEdge.id);
      if (!originalEdge) continue;

      // Extract edge routing points
      const points: Point[] = [];

      if (elkEdge.sections && elkEdge.sections.length > 0) {
        const section = elkEdge.sections[0];

        // Start point
        points.push({
          x: section.startPoint.x,
          y: section.startPoint.y,
        });

        // Bend points
        if (section.bendPoints) {
          for (const bp of section.bendPoints) {
            points.push({ x: bp.x, y: bp.y });
          }
        }

        // End point
        points.push({
          x: section.endPoint.x,
          y: section.endPoint.y,
        });
      }

      layoutEdges.push({
        ...originalEdge,
        points,
      });
    }
  }

  return layoutEdges;
}

// ============================================================================
// Step 4: Calculate Total Size
// ============================================================================

function calculateTotalSize(
  elkGraph: ElkNode,
  config: Required<LayoutConfig>
): Size {
  return {
    width: (elkGraph.width || 800) + config.padding.left + config.padding.right,
    height: (elkGraph.height || 600) + config.padding.top + config.padding.bottom,
  };
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Calculate transition label position above the curve
 */
export function calculateTransitionLabelPosition(points: Point[]): Point {
  if (points.length < 2) {
    return points[0] || { x: 0, y: 0 };
  }

  // Use midpoint of the transition path
  const midIndex = Math.floor(points.length / 2);
  const midPoint = points[midIndex];

  // Offset label above the line
  return {
    x: midPoint.x,
    y: midPoint.y - 15,
  };
}

/**
 * Identify self-transitions (loops)
 */
export function isSelfTransition(edge: DiagramEdge): boolean {
  return edge.from === edge.to;
}
