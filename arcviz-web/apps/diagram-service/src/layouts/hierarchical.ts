/**
 * Hierarchical Layout Algorithm
 * 
 * Uses ELK (Eclipse Layout Kernel) for hierarchical graph layout.
 * Ideal for functional dataflow diagrams and component hierarchies.
 */

import ELK, { ElkNode, ElkExtendedEdge, ElkPort } from 'elkjs/lib/elk.bundled';
import {
  DiagramNode,
  DiagramEdge,
  Point,
  Size,
  LayoutConfig,
  Port,
} from '../types/diagram';
import { wrapText, measureText, calculateComponentDimensions } from '../utils/text-metrics';

// ============================================================================
// Text Measurement Utility
// ============================================================================

function measureTextApprox(text: string, fontSize: number, fontWeight: string): number {
  let width = 0;
  for (let i = 0; i < text.length; i++) {
    const char = text[i];
    if (char === char.toUpperCase() && char !== char.toLowerCase()) {
      width += 0.7; 
    } else if (char === ' ') {
      width += 0.3;
    } else {
      width += 0.6;
    }
  }
  return Math.ceil(width * fontSize * (fontWeight === 'bold' ? 1.1 : 1));
}

// ============================================================================
// Configuration
// ============================================================================

const DEFAULT_CONFIG: Required<LayoutConfig> = {
  direction: 'RIGHT',
  nodeSpacing: 60,
  layerSpacing: 100,
  edgeSpacing: 20,
  padding: {
    top: 40,
    right: 40,
    bottom: 40,
    left: 40,
  },
  algorithm: 'elk',
};

// ============================================================================
// Types
// ============================================================================

interface LayoutNode extends DiagramNode {
  position: Point;
  size: Size;
  ports?: Array<Port & { position: Point }>;
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
 * Apply hierarchical layout using ELK
 */
export async function applyHierarchicalLayout(
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
    const hasChildren = !!(node.children && node.children.length > 0);
    const childCount = hasChildren ? (node.children?.length || 0) : 0;
    
    // Calculate dimensions using professional text wrapping
    const dimensions = calculateComponentDimensions(
      node.label || '',
      hasChildren,
      childCount,
      80, // child height
      12  // base font size
    );
    
    let width = node.size?.width || dimensions.width;
    let height = node.size?.height || dimensions.height;
    
    const elkNode: ElkNode = {
      id: node.id,
      width: width,
      height: height,
      labels: node.label ? [{ 
        text: node.label,
        width: dimensions.width,
        height: dimensions.labelHeight,
      }] : [],
    };

    // Add ports if present
    if (node.ports && node.ports.length > 0) {
      elkNode.ports = node.ports.map(port => ({
        id: `${node.id}.${port.name}`,
        width: 10,
        height: 10,
        labels: [{ text: port.name }],
        properties: {
          'port.side': mapPortSide(port.side),
          'port.index': port.id,
        },
      }));
    }

    // Handle children (nested nodes)
    if (node.children && node.children.length > 0) {
      // Calculate required space for parent label
      const parentLabelHeight = 30; // space for label + separator
      const childNodeSpacing = 20;
      const sidePadding = 20;
      const bottomPadding = 20;
      
      // Convert children first to get their sizes
      const childGraph = convertToElkGraph(node.children, [], config);
      elkNode.children = childGraph.children;
      elkNode.edges = childGraph.edges;
      
      // Calculate total child height
      let totalChildHeight = 0;
      let maxChildWidth = 0;
      if (childGraph.children) {
        for (const child of childGraph.children) {
          totalChildHeight += (child.height || 80) + childNodeSpacing;
          maxChildWidth = Math.max(maxChildWidth, child.width || 150);
        }
        totalChildHeight -= childNodeSpacing; // remove last spacing
      }
      
      // Calculate minimum parent size to contain children + labels
      const minHeight = parentLabelHeight + totalChildHeight + bottomPadding;
      const minWidth = maxChildWidth + (sidePadding * 2);
      const parentLabelWidth = measureTextApprox(node.label || '', 14, 'bold');
      
      elkNode.height = Math.max(elkNode.height || 80, minHeight);
      elkNode.width = Math.max(elkNode.width || 150, minWidth, parentLabelWidth + 60);
      
      // Add layout options for parent node
      elkNode.layoutOptions = {
        'elk.algorithm': 'box',
        'elk.direction': 'DOWN',
        'elk.padding': '[top=50,left=20,bottom=20,right=20]',
        'elk.spacing.nodeNode': String(childNodeSpacing),
        'elk.contentAlignment': 'V_TOP H_CENTER',
      };
    }

    elkNodes.push(elkNode);
  }

  // Convert edges
  for (const edge of edges) {
    // Calculate edge label dimensions
    const edgeLabelWidth = edge.label ? edge.label.length * 6 + 10 : 0;
    const edgeLabelHeight = edge.label ? 20 : 0;
    
    const elkEdge: ElkExtendedEdge = {
      id: edge.id,
      sources: [edge.fromPort ? `${edge.from}.${edge.fromPort}` : edge.from],
      targets: [edge.toPort ? `${edge.to}.${edge.toPort}` : edge.to],
      labels: edge.label ? [{ 
        text: edge.label,
        width: edgeLabelWidth,
        height: edgeLabelHeight,
      }] : [],
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
      'elk.spacing.edgeLabel': '10',
      'elk.padding': `[top=${config.padding.top},left=${config.padding.left},bottom=${config.padding.bottom},right=${config.padding.right}]`,
      'elk.edgeRouting': 'ORTHOGONAL',
      'elk.port.side': 'SIDES_ONLY',
      'elk.layered.nodePlacement.strategy': 'NETWORK_SIMPLEX',
      'elk.hierarchyHandling': 'INCLUDE_CHILDREN',
      'elk.edgeLabels.placement': 'CENTER',
      'elk.edgeLabels.inline': 'false',
      'elk.considerModelOrder.strategy': 'PREFER_EDGES',
    },
  };
}

function mapPortSide(side: 'TOP' | 'RIGHT' | 'BOTTOM' | 'LEFT'): string {
  const mapping: Record<string, string> = {
    TOP: 'NORTH',
    RIGHT: 'EAST',
    BOTTOM: 'SOUTH',
    LEFT: 'WEST',
  };
  return mapping[side] || 'EAST';
}

// ============================================================================
// Step 2: Extract Layout Nodes
// ============================================================================

function extractLayoutNodes(
  elkGraph: ElkNode,
  originalNodes: DiagramNode[]
): LayoutNode[] {
  const layoutNodes: LayoutNode[] = [];
  
  // Recursively collect all nodes including nested ones
  function collectAllNodes(nodes: DiagramNode[]): Map<string, DiagramNode> {
    const map = new Map<string, DiagramNode>();
    for (const node of nodes) {
      map.set(node.id, node);
      if (node.children && node.children.length > 0) {
        const childMap = collectAllNodes(node.children);
        for (const [id, childNode] of childMap) {
          map.set(id, childNode);
        }
      }
    }
    return map;
  }
  
  const nodeMap = collectAllNodes(originalNodes);

  function processNode(elkNode: ElkNode, offsetX: number = 0, offsetY: number = 0): LayoutNode | null {
    const originalNode = nodeMap.get(elkNode.id);
    if (!originalNode) return null;

    const position: Point = {
      x: (elkNode.x || 0) + offsetX,
      y: (elkNode.y || 0) + offsetY,
    };

    const size: Size = {
      width: elkNode.width || 150,
      height: elkNode.height || 80,
    };

    // Process ports
    const ports: Array<Port & { position: Point }> = [];
    if (elkNode.ports) {
      for (const elkPort of elkNode.ports) {
        const originalPort = originalNode.ports?.find(p => `${elkNode.id}.${p.name}` === elkPort.id);
        if (originalPort) {
          ports.push({
            ...originalPort,
            position: {
              x: position.x + (elkPort.x || 0),
              y: position.y + (elkPort.y || 0),
            },
          });
        }
      }
    }

    // Process children recursively
    const children: LayoutNode[] = [];
    if (elkNode.children) {
      for (const child of elkNode.children) {
        const childNode = processNode(child, position.x, position.y);
        if (childNode) {
          children.push(childNode);
        }
      }
    }

    return {
      ...originalNode,
      position,
      size,
      ports: ports.length > 0 ? ports : undefined,
      children: children.length > 0 ? children : undefined,
    };
  }

  if (elkGraph.children) {
    for (const child of elkGraph.children) {
      const node = processNode(child);
      if (node) {
        layoutNodes.push(node);
      }
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
 * Calculate port position on node border
 */
export function calculatePortPosition(
  node: LayoutNode,
  port: Port
): Point {
  const center = {
    x: node.position.x + node.size.width / 2,
    y: node.position.y + node.size.height / 2,
  };

  switch (port.side) {
    case 'TOP':
      return {
        x: center.x,
        y: node.position.y,
      };
    case 'RIGHT':
      return {
        x: node.position.x + node.size.width,
        y: center.y,
      };
    case 'BOTTOM':
      return {
        x: center.x,
        y: node.position.y + node.size.height,
      };
    case 'LEFT':
      return {
        x: node.position.x,
        y: center.y,
      };
  }
}

/**
 * Assign ports to optimal sides based on edge directions
 * 
 * LaTeX Spec Section 5 (Port Positioning Rules):
 * - IN ports → LEFT side
 * - OUT ports → RIGHT side
 * - BIDIRECTIONAL/INOUT ports → TOP or BOTTOM side
 * - CONTROL/Management ports → TOP side
 * - POWER/Ground ports → BOTTOM side (physical architecture)
 */
export function assignPortSides(
  nodes: DiagramNode[],
  edges: DiagramEdge[]
): DiagramNode[] {
  const nodeMap = new Map(nodes.map(n => [n.id, n]));

  for (const node of nodes) {
    if (!node.ports) continue;

    for (const port of node.ports) {
      // Find edges connected to this port
      const outgoing = edges.filter(e => e.from === node.id && e.fromPort === port.name);
      const incoming = edges.filter(e => e.to === node.id && e.toPort === port.name);

      // Determine port type from metadata
      const portType = port.metadata?.port_type || port.metadata?.type || '';
      const portName = port.name.toLowerCase();

      // CRITICAL: Assign side based on Capella specification
      if (portType === 'control' || portName.includes('control') || portName.includes('cmd')) {
        // Control/Management ports → TOP
        port.side = 'TOP';
      } else if (portType === 'power' || portName.includes('power') || portName.includes('vcc') || 
                 portName.includes('gnd') || portName.includes('ground')) {
        // Power/Ground ports → BOTTOM
        port.side = 'BOTTOM';
      } else if (port.direction === 'INOUT') {
        // Bidirectional ports → TOP or BOTTOM (prefer TOP)
        port.side = 'TOP';
      } else if (port.direction === 'IN') {
        // Input ports → LEFT
        port.side = 'LEFT';
      } else if (port.direction === 'OUT') {
        // Output ports → RIGHT
        port.side = 'RIGHT';
      } else {
        // Default based on edge direction
        if (outgoing.length > incoming.length) {
          port.side = 'RIGHT';
        } else if (incoming.length > 0) {
          port.side = 'LEFT';
        } else {
          port.side = 'TOP';
        }
      }
    }
  }

  return nodes;
}
