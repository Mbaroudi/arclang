import ELK, { ElkNode, ElkExtendedEdge } from 'elkjs/lib/elk.bundled';
import {
  DiagramNode,
  DiagramEdge,
  Point,
  Size,
  LayoutConfig,
} from '../types/diagram';

const DEFAULT_CONFIG: Required<LayoutConfig> = {
  direction: 'RIGHT',
  nodeSpacing: 80,
  layerSpacing: 100,
  edgeSpacing: 40,
  padding: {
    top: 80,
    right: 80,
    bottom: 80,
    left: 220,
  },
  algorithm: 'elk',
};

interface LayoutNode extends DiagramNode {
  position: Point;
  size: Size;
  swimlaneId: string;
  layer: number;
}

interface LayoutEdge extends DiagramEdge {
  points: Point[];
}

interface Swimlane {
  id: string;
  label: string;
  activityIds: string[];
  position?: Point;
  size?: Size;
}

interface LayoutResult {
  nodes: LayoutNode[];
  edges: LayoutEdge[];
  swimlanes: Swimlane[];
  totalSize: Size;
}

export async function applyElkOperationalLayout(
  nodes: DiagramNode[],
  edges: DiagramEdge[],
  config: Partial<LayoutConfig> = {}
): Promise<LayoutResult> {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  const swimlaneGroups = groupNodesBySwimlane(nodes);
  const swimlaneNames = Array.from(swimlaneGroups.keys());
  
  const elkGraph = convertToElkGraph(nodes, edges, swimlaneNames, cfg);

  const elk = new ELK();
  const laidOutGraph = await elk.layout(elkGraph);

  const layoutNodes = extractLayoutNodes(laidOutGraph, nodes);
  const layoutEdges = extractLayoutEdges(laidOutGraph, edges);
  const swimlanes = createSwimlanes(layoutNodes, swimlaneGroups, cfg);
  const totalSize = calculateTotalSize(swimlanes, cfg);

  return {
    nodes: layoutNodes,
    edges: layoutEdges,
    swimlanes,
    totalSize,
  };
}

function groupNodesBySwimlane(nodes: DiagramNode[]): Map<string, DiagramNode[]> {
  const groups = new Map<string, DiagramNode[]>();
  
  for (const node of nodes) {
    const swimlane = node.metadata?.swimlane || node.metadata?.performed_by || 'default';
    if (!groups.has(swimlane)) {
      groups.set(swimlane, []);
    }
    groups.get(swimlane)!.push(node);
  }
  
  return groups;
}

function convertToElkGraph(
  nodes: DiagramNode[],
  edges: DiagramEdge[],
  swimlaneNames: string[],
  config: Required<LayoutConfig>
): ElkNode {
  const elkNodes: ElkNode[] = [];
  const elkEdges: ElkExtendedEdge[] = [];

  for (const node of nodes) {
    const width = node.size?.width || 180;
    const height = node.size?.height || 100;

    elkNodes.push({
      id: node.id,
      width,
      height,
      labels: node.label ? [{ text: node.label }] : [],
    });
  }

  for (const edge of edges) {
    elkEdges.push({
      id: edge.id,
      sources: [edge.from],
      targets: [edge.to],
      labels: edge.label ? [{ text: edge.label }] : [],
    });
  }

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
      'elk.edgeRouting': 'ORTHOGONAL',
      'elk.layered.nodePlacement.strategy': 'NETWORK_SIMPLEX',
      'elk.layered.crossingMinimization.strategy': 'LAYER_SWEEP',
      'elk.layered.considerModelOrder.strategy': 'NODES_AND_EDGES',
      'elk.separateConnectedComponents': 'false',
    },
  };
}

function extractLayoutNodes(
  elkGraph: ElkNode,
  originalNodes: DiagramNode[]
): LayoutNode[] {
  const layoutNodes: LayoutNode[] = [];
  const nodeMap = new Map(originalNodes.map(n => [n.id, n]));

  if (elkGraph.children) {
    for (const elkNode of elkGraph.children) {
      const originalNode = nodeMap.get(elkNode.id);
      if (!originalNode) continue;

      const swimlane = originalNode.metadata?.swimlane || originalNode.metadata?.performed_by || 'default';
      
      layoutNodes.push({
        ...originalNode,
        position: {
          x: elkNode.x || 0,
          y: elkNode.y || 0,
        },
        size: {
          width: elkNode.width || 180,
          height: elkNode.height || 100,
        },
        swimlaneId: swimlane,
        layer: 0,
      });
    }
  }

  return layoutNodes;
}

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

      const points: Point[] = [];

      if (elkEdge.sections && elkEdge.sections.length > 0) {
        const section = elkEdge.sections[0];

        points.push({
          x: section.startPoint.x,
          y: section.startPoint.y,
        });

        if (section.bendPoints) {
          for (const bp of section.bendPoints) {
            points.push({ x: bp.x, y: bp.y });
          }
        }

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

function createSwimlanes(
  layoutNodes: LayoutNode[],
  swimlaneGroups: Map<string, DiagramNode[]>,
  config: Required<LayoutConfig>
): Swimlane[] {
  const swimlanes: Swimlane[] = [];
  
  const swimlaneNodeMap = new Map<string, LayoutNode[]>();
  for (const node of layoutNodes) {
    const swimlane = node.metadata?.swimlane || node.metadata?.performed_by || 'default';
    if (!swimlaneNodeMap.has(swimlane)) {
      swimlaneNodeMap.set(swimlane, []);
    }
    swimlaneNodeMap.get(swimlane)!.push(node);
  }

  let currentY = config.padding.top;
  
  for (const [swimlaneName, nodes] of swimlaneNodeMap.entries()) {
    if (nodes.length === 0) continue;

    const minY = Math.min(...nodes.map(n => n.position.y));
    const maxY = Math.max(...nodes.map(n => n.position.y + n.size.height));
    const minX = Math.min(...nodes.map(n => n.position.x));
    const maxX = Math.max(...nodes.map(n => n.position.x + n.size.width));

    const swimlaneHeight = maxY - minY + 80;
    
    swimlanes.push({
      id: swimlaneName,
      label: nodes[0].metadata?.swimlaneLabel || swimlaneName,
      activityIds: nodes.map(n => n.id),
      position: {
        x: 0,
        y: currentY,
      },
      size: {
        width: maxX - minX + config.padding.left + config.padding.right,
        height: swimlaneHeight,
      },
    });

    currentY += swimlaneHeight + 40;
  }

  return swimlanes;
}

function calculateTotalSize(
  swimlanes: Swimlane[],
  config: Required<LayoutConfig>
): Size {
  if (swimlanes.length === 0) {
    return { width: 1600, height: 1200 };
  }

  const maxWidth = Math.max(...swimlanes.map(s => s.size?.width || 1600));
  const totalHeight = swimlanes.reduce((sum, s) => sum + (s.size?.height || 400), 0) + 
                      (swimlanes.length - 1) * 40 + 
                      config.padding.top + 
                      config.padding.bottom;

  return {
    width: maxWidth + 100,
    height: totalHeight + 100,
  };
}
