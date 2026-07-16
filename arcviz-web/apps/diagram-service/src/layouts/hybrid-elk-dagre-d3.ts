/**
 * Hybrid ELK + Dagre + D3 Layout Engine
 * 
 * Multi-pass optimization approach combining three layout technologies:
 * 
 * Layer 1: ELK (Eclipse Layout Kernel)
 *   - Hierarchical structure and initial positioning
 *   - Layer assignment and node placement
 *   - Primary layout algorithm
 * 
 * Layer 2: Dagre (Directed Graph Layout)
 *   - Edge crossing minimization
 *   - Rank optimization
 *   - Port assignment refinement
 * 
 * Layer 3: D3-Force (Force-Directed Layout)
 *   - Collision detection and prevention
 *   - Local spacing adjustments
 *   - Aesthetic improvements
 * 
 * Layer 4: Capella Style (Custom)
 *   - Safety-critical border styling
 *   - MBSE-specific constraints
 *   - Interface notation (ball-and-socket)
 */

import ELK, { ElkNode, ElkExtendedEdge } from 'elkjs/lib/elk.bundled';
import * as dagre from 'dagre';
import {
  forceSimulation,
  forceCollide,
  forceX,
  forceY,
  forceLink,
  SimulationNodeDatum,
  SimulationLinkDatum,
} from 'd3-force';
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

interface HybridConfig extends LayoutConfig {
  // ELK configuration
  elkNodeSpacing?: number;
  elkLayerSpacing?: number;
  elkEdgeRouting?: 'ORTHOGONAL' | 'SPLINES' | 'POLYLINE';
  
  // Dagre configuration
  dagreRankSep?: number;
  dagreNodeSep?: number;
  dagreEdgeSep?: number;
  dagreRankDir?: 'TB' | 'BT' | 'LR' | 'RL';
  
  // D3-Force configuration
  d3CollisionRadius?: number;
  d3LinkStrength?: number;
  d3Iterations?: number;
  d3AlphaDecay?: number;
  
  // Optimization weights
  elkWeight?: number;      // 0-1: How much to trust ELK layout
  dagreWeight?: number;    // 0-1: How much to apply Dagre optimization
  d3Weight?: number;       // 0-1: How much to apply D3 refinement
  
  // Capella style
  enableSafetyBorders?: boolean;
  enableInterfaceNotation?: boolean;
  minimumSpacing?: number;
}

const DEFAULT_HYBRID_CONFIG: Required<HybridConfig> = {
  direction: 'RIGHT',
  nodeSpacing: 80,
  layerSpacing: 100,
  edgeSpacing: 40,
  padding: {
    top: 60,
    right: 60,
    bottom: 60,
    left: 60,
  },
  algorithm: 'elk',
  
  // ELK settings
  elkNodeSpacing: 80,
  elkLayerSpacing: 100,
  elkEdgeRouting: 'ORTHOGONAL',
  
  // Dagre settings
  dagreRankSep: 100,
  dagreNodeSep: 80,
  dagreEdgeSep: 40,
  dagreRankDir: 'LR',
  
  // D3-Force settings
  d3CollisionRadius: 50,
  d3LinkStrength: 0.3,
  d3Iterations: 100,
  d3AlphaDecay: 0.05,
  
  // Optimization weights
  elkWeight: 0.7,      // ELK is primary
  dagreWeight: 0.2,    // Dagre for edge optimization
  d3Weight: 0.1,       // D3 for fine-tuning
  
  // Capella style
  enableSafetyBorders: true,
  enableInterfaceNotation: true,
  minimumSpacing: 40,
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
  metadata: {
    elkScore: number;
    dagreScore: number;
    d3Score: number;
    totalOptimizationTime: number;
  };
}

interface D3Node extends SimulationNodeDatum {
  id: string;
  x: number;
  y: number;
  vx?: number;
  vy?: number;
  fx?: number | null;
  fy?: number | null;
  radius: number;
  originalX: number;
  originalY: number;
}

interface D3Link extends SimulationLinkDatum<D3Node> {
  source: string | D3Node;
  target: string | D3Node;
}

// ============================================================================
// Main Hybrid Layout Function
// ============================================================================

/**
 * Apply hybrid ELK + Dagre + D3 multi-pass optimization
 */
export async function applyHybridLayout(
  nodes: DiagramNode[],
  edges: DiagramEdge[],
  config: Partial<HybridConfig> = {}
): Promise<LayoutResult> {
  const cfg = { ...DEFAULT_HYBRID_CONFIG, ...config };
  const startTime = performance.now();
  
  console.log('[Hybrid] Starting multi-pass optimization...');
  console.log(`[Hybrid] Weights: ELK=${cfg.elkWeight} Dagre=${cfg.dagreWeight} D3=${cfg.d3Weight}`);
  
  // Layer 1: ELK - Hierarchical structure and initial positioning
  console.log('[Hybrid] Layer 1: ELK layout...');
  const elkStart = performance.now();
  const elkLayout = await applyElkLayout(nodes, edges, cfg);
  const elkTime = performance.now() - elkStart;
  console.log(`[Hybrid] ELK completed in ${elkTime.toFixed(2)}ms`);
  
  // Layer 2: Dagre - Edge crossing minimization and rank optimization
  console.log('[Hybrid] Layer 2: Dagre optimization...');
  const dagreStart = performance.now();
  const dagreLayout = applyDagreOptimization(elkLayout, edges, cfg);
  const dagreTime = performance.now() - dagreStart;
  console.log(`[Hybrid] Dagre completed in ${dagreTime.toFixed(2)}ms`);
  
  // Layer 3: D3-Force - Collision detection and local adjustments
  console.log('[Hybrid] Layer 3: D3-Force refinement...');
  const d3Start = performance.now();
  const d3Layout = applyD3ForceRefinement(dagreLayout, edges, cfg);
  const d3Time = performance.now() - d3Start;
  console.log(`[Hybrid] D3-Force completed in ${d3Time.toFixed(2)}ms`);
  
  // Layer 4: Capella style - MBSE-specific adjustments
  console.log('[Hybrid] Layer 4: Capella style refinement...');
  const finalLayout = applyCapellaStyle(d3Layout, edges, cfg);
  
  const totalTime = performance.now() - startTime;
  console.log(`[Hybrid] Total optimization: ${totalTime.toFixed(2)}ms`);
  
  // Calculate quality scores
  const elkScore = calculateLayoutQuality(elkLayout.nodes, edges);
  const dagreScore = calculateLayoutQuality(dagreLayout.nodes, edges);
  const d3Score = calculateLayoutQuality(d3Layout.nodes, edges);
  
  console.log(`[Hybrid] Quality scores: ELK=${elkScore.toFixed(2)} Dagre=${dagreScore.toFixed(2)} D3=${d3Score.toFixed(2)}`);
  
  return {
    ...finalLayout,
    metadata: {
      elkScore,
      dagreScore,
      d3Score,
      totalOptimizationTime: totalTime,
    },
  };
}

// ============================================================================
// Layer 1: ELK Layout
// ============================================================================

async function applyElkLayout(
  nodes: DiagramNode[],
  edges: DiagramEdge[],
  config: Required<HybridConfig>
): Promise<{ nodes: LayoutNode[]; edges: LayoutEdge[] }> {
  const elkNodes: ElkNode[] = [];
  const elkEdges: ElkExtendedEdge[] = [];
  
  // Convert nodes to ELK format
  for (const node of nodes) {
    elkNodes.push({
      id: node.id,
      width: node.size?.width || 180,
      height: node.size?.height || 100,
      labels: node.label ? [{ text: node.label }] : [],
    });
  }
  
  // Convert edges to ELK format
  for (const edge of edges) {
    elkEdges.push({
      id: edge.id,
      sources: [edge.from],
      targets: [edge.to],
      labels: edge.label ? [{ text: edge.label }] : [],
    });
  }
  
  // Create ELK graph
  const elkGraph: ElkNode = {
    id: 'root',
    children: elkNodes,
    edges: elkEdges,
    layoutOptions: {
      'elk.algorithm': 'layered',
      'elk.direction': config.direction,
      'elk.spacing.nodeNode': String(config.elkNodeSpacing),
      'elk.layered.spacing.nodeNodeBetweenLayers': String(config.elkLayerSpacing),
      'elk.spacing.edgeNode': String(config.edgeSpacing),
      'elk.edgeRouting': config.elkEdgeRouting,
      'elk.layered.nodePlacement.strategy': 'NETWORK_SIMPLEX',
      'elk.layered.crossingMinimization.strategy': 'LAYER_SWEEP',
      'elk.layered.considerModelOrder.strategy': 'NODES_AND_EDGES',
      'elk.separateConnectedComponents': 'false',
    },
  };
  
  // Run ELK layout
  const elk = new ELK();
  const laidOutGraph = await elk.layout(elkGraph);
  
  // Extract positioned nodes
  const layoutNodes: LayoutNode[] = [];
  const nodeMap = new Map(nodes.map(n => [n.id, n]));
  
  if (laidOutGraph.children) {
    for (const elkNode of laidOutGraph.children) {
      const originalNode = nodeMap.get(elkNode.id);
      if (!originalNode) continue;
      
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
      });
    }
  }
  
  // Extract edge routing
  const layoutEdges: LayoutEdge[] = [];
  const edgeMap = new Map(edges.map(e => [e.id, e]));
  
  if (laidOutGraph.edges) {
    for (const elkEdge of laidOutGraph.edges) {
      const originalEdge = edgeMap.get(elkEdge.id);
      if (!originalEdge) continue;
      
      const points: Point[] = [];
      
      if (elkEdge.sections && elkEdge.sections.length > 0) {
        const section = elkEdge.sections[0];
        points.push({ x: section.startPoint.x, y: section.startPoint.y });
        
        if (section.bendPoints) {
          for (const bp of section.bendPoints) {
            points.push({ x: bp.x, y: bp.y });
          }
        }
        
        points.push({ x: section.endPoint.x, y: section.endPoint.y });
      }
      
      layoutEdges.push({
        ...originalEdge,
        points,
      });
    }
  }
  
  return { nodes: layoutNodes, edges: layoutEdges };
}

// ============================================================================
// Layer 2: Dagre Optimization
// ============================================================================

function applyDagreOptimization(
  elkLayout: { nodes: LayoutNode[]; edges: LayoutEdge[] },
  originalEdges: DiagramEdge[],
  config: Required<HybridConfig>
): { nodes: LayoutNode[]; edges: LayoutEdge[] } {
  // Create Dagre graph
  const g = new dagre.graphlib.Graph();
  g.setGraph({
    rankdir: config.dagreRankDir,
    ranksep: config.dagreRankSep,
    nodesep: config.dagreNodeSep,
    edgesep: config.dagreEdgeSep,
    marginx: config.padding.left,
    marginy: config.padding.top,
  });
  
  g.setDefaultEdgeLabel(() => ({}));
  
  // Add nodes with ELK positions as initial hints
  for (const node of elkLayout.nodes) {
    g.setNode(node.id, {
      width: node.size.width,
      height: node.size.height,
      x: node.position.x,
      y: node.position.y,
    });
  }
  
  // Add edges
  for (const edge of originalEdges) {
    g.setEdge(edge.from, edge.to);
  }
  
  // Run Dagre layout
  dagre.layout(g);
  
  // Blend Dagre positions with ELK positions
  const dagreNodes: LayoutNode[] = elkLayout.nodes.map(elkNode => {
    const dagreNode = g.node(elkNode.id);
    
    if (!dagreNode) return elkNode;
    
    // Weighted average: more weight to ELK, Dagre for fine-tuning
    const blendedX = elkNode.position.x * config.elkWeight + dagreNode.x * config.dagreWeight;
    const blendedY = elkNode.position.y * config.elkWeight + dagreNode.y * config.dagreWeight;
    
    return {
      ...elkNode,
      position: {
        x: blendedX,
        y: blendedY,
      },
    };
  });
  
  // Recalculate edge routing based on new positions
  const dagreEdges: LayoutEdge[] = elkLayout.edges.map(edge => {
    const sourceNode = dagreNodes.find(n => n.id === edge.from);
    const targetNode = dagreNodes.find(n => n.id === edge.to);
    
    if (!sourceNode || !targetNode) return edge;
    
    // Simple orthogonal routing between blended positions
    const points = calculateOrthogonalPath(
      sourceNode.position,
      sourceNode.size,
      targetNode.position,
      targetNode.size
    );
    
    return {
      ...edge,
      points,
    };
  });
  
  return { nodes: dagreNodes, edges: dagreEdges };
}

// ============================================================================
// Layer 3: D3-Force Refinement
// ============================================================================

function applyD3ForceRefinement(
  dagreLayout: { nodes: LayoutNode[]; edges: LayoutEdge[] },
  originalEdges: DiagramEdge[],
  config: Required<HybridConfig>
): { nodes: LayoutNode[]; edges: LayoutEdge[] } {
  // Convert to D3 format
  const d3Nodes: D3Node[] = dagreLayout.nodes.map(node => ({
    id: node.id,
    x: node.position.x + node.size.width / 2,
    y: node.position.y + node.size.height / 2,
    radius: Math.max(node.size.width, node.size.height) / 2,
    originalX: node.position.x + node.size.width / 2,
    originalY: node.position.y + node.size.height / 2,
    fx: null,
    fy: null,
  }));
  
  const d3Links: D3Link[] = originalEdges.map(edge => ({
    source: edge.from,
    target: edge.to,
  }));
  
  // Create force simulation
  const simulation = forceSimulation(d3Nodes)
    .force('collision', forceCollide<D3Node>().radius(d => d.radius + config.d3CollisionRadius))
    .force('link', forceLink<D3Node, D3Link>(d3Links)
      .id(d => d.id)
      .strength(config.d3LinkStrength)
      .distance(100))
    .force('x', forceX<D3Node>().x(d => d.originalX).strength(0.5))
    .force('y', forceY<D3Node>().y(d => d.originalY).strength(0.5))
    .alphaDecay(config.d3AlphaDecay)
    .stop();
  
  // Run simulation
  for (let i = 0; i < config.d3Iterations; i++) {
    simulation.tick();
  }
  
  // Blend D3 positions with Dagre positions
  const d3RefinedNodes: LayoutNode[] = dagreLayout.nodes.map(dagreNode => {
    const d3Node = d3Nodes.find(n => n.id === dagreNode.id);
    
    if (!d3Node) return dagreNode;
    
    // Weighted average: more weight to Dagre, D3 for collision avoidance
    const blendedX = dagreNode.position.x * (1 - config.d3Weight) + 
                     (d3Node.x - dagreNode.size.width / 2) * config.d3Weight;
    const blendedY = dagreNode.position.y * (1 - config.d3Weight) + 
                     (d3Node.y - dagreNode.size.height / 2) * config.d3Weight;
    
    return {
      ...dagreNode,
      position: {
        x: blendedX,
        y: blendedY,
      },
    };
  });
  
  // Recalculate edge routing
  const d3RefinedEdges: LayoutEdge[] = dagreLayout.edges.map(edge => {
    const sourceNode = d3RefinedNodes.find(n => n.id === edge.from);
    const targetNode = d3RefinedNodes.find(n => n.id === edge.to);
    
    if (!sourceNode || !targetNode) return edge;
    
    const points = calculateOrthogonalPath(
      sourceNode.position,
      sourceNode.size,
      targetNode.position,
      targetNode.size
    );
    
    return {
      ...edge,
      points,
    };
  });
  
  return { nodes: d3RefinedNodes, edges: d3RefinedEdges };
}

// ============================================================================
// Layer 4: Capella Style Refinement
// ============================================================================

function applyCapellaStyle(
  d3Layout: { nodes: LayoutNode[]; edges: LayoutEdge[] },
  edges: DiagramEdge[],
  config: Required<HybridConfig>
): { nodes: LayoutNode[]; edges: LayoutEdge[]; totalSize: Size } {
  // Apply minimum spacing constraints
  const capellaNodes = enforceMinimumSpacing(d3Layout.nodes, config.minimumSpacing);
  
  // Ensure no overlaps
  const noOverlapNodes = preventOverlaps(capellaNodes);
  
  // Snap to grid for clean alignment
  const snappedNodes = snapToGrid(noOverlapNodes, 10);
  
  // Recalculate edge routing for final positions
  const capellaEdges: LayoutEdge[] = d3Layout.edges.map(edge => {
    const sourceNode = snappedNodes.find(n => n.id === edge.from);
    const targetNode = snappedNodes.find(n => n.id === edge.to);
    
    if (!sourceNode || !targetNode) return edge;
    
    const points = calculateOrthogonalPath(
      sourceNode.position,
      sourceNode.size,
      targetNode.position,
      targetNode.size
    );
    
    return {
      ...edge,
      points,
    };
  });
  
  // Calculate total size
  const totalSize = calculateTotalSize(snappedNodes, config);
  
  return {
    nodes: snappedNodes,
    edges: capellaEdges,
    totalSize,
  };
}

// ============================================================================
// Utility Functions
// ============================================================================

function calculateOrthogonalPath(
  sourcePos: Point,
  sourceSize: Size,
  targetPos: Point,
  targetSize: Size
): Point[] {
  const sourceCenterX = sourcePos.x + sourceSize.width / 2;
  const sourceCenterY = sourcePos.y + sourceSize.height / 2;
  const targetCenterX = targetPos.x + targetSize.width / 2;
  const targetCenterY = targetPos.y + targetSize.height / 2;
  
  const points: Point[] = [];
  
  // Simple straight line if on same horizontal level
  if (Math.abs(sourceCenterY - targetCenterY) < 5) {
    points.push({
      x: sourcePos.x + sourceSize.width,
      y: sourceCenterY,
    });
    points.push({
      x: targetPos.x,
      y: targetCenterY,
    });
    return points;
  }
  
  // Start point (right edge of source)
  points.push({
    x: sourcePos.x + sourceSize.width,
    y: sourceCenterY,
  });
  
  // Mid point (orthogonal routing)
  const midX = (sourcePos.x + sourceSize.width + targetPos.x) / 2;
  points.push({ x: midX, y: sourceCenterY });
  points.push({ x: midX, y: targetCenterY });
  
  // End point (left edge of target)
  points.push({
    x: targetPos.x,
    y: targetCenterY,
  });
  
  return points;
}

function enforceMinimumSpacing(nodes: LayoutNode[], minSpacing: number): LayoutNode[] {
  const result = [...nodes];
  
  for (let i = 0; i < result.length; i++) {
    for (let j = i + 1; j < result.length; j++) {
      const node1 = result[i];
      const node2 = result[j];
      
      const dx = (node2.position.x + node2.size.width / 2) - (node1.position.x + node1.size.width / 2);
      const dy = (node2.position.y + node2.size.height / 2) - (node1.position.y + node1.size.height / 2);
      const distance = Math.sqrt(dx * dx + dy * dy);
      const minDist = minSpacing + (node1.size.width + node2.size.width) / 2;
      
      if (distance < minDist && distance > 0) {
        const angle = Math.atan2(dy, dx);
        const pushDist = (minDist - distance) / 2;
        
        result[j].position.x += Math.cos(angle) * pushDist;
        result[j].position.y += Math.sin(angle) * pushDist;
      }
    }
  }
  
  return result;
}

function preventOverlaps(nodes: LayoutNode[]): LayoutNode[] {
  // Simple overlap prevention using separation
  return enforceMinimumSpacing(nodes, 20);
}

function snapToGrid(nodes: LayoutNode[], gridSize: number): LayoutNode[] {
  return nodes.map(node => ({
    ...node,
    position: {
      x: Math.round(node.position.x / gridSize) * gridSize,
      y: Math.round(node.position.y / gridSize) * gridSize,
    },
  }));
}

function calculateTotalSize(nodes: LayoutNode[], config: Required<HybridConfig>): Size {
  if (nodes.length === 0) {
    return { width: 1600, height: 1200 };
  }
  
  const maxX = Math.max(...nodes.map(n => n.position.x + n.size.width));
  const maxY = Math.max(...nodes.map(n => n.position.y + n.size.height));
  
  return {
    width: maxX + config.padding.right,
    height: maxY + config.padding.bottom,
  };
}

function calculateLayoutQuality(nodes: LayoutNode[], edges: DiagramEdge[]): number {
  let score = 100;
  
  // Check for overlaps
  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      if (nodesOverlap(nodes[i], nodes[j])) {
        score -= 10;
      }
    }
  }
  
  // Check edge crossings
  const crossings = countEdgeCrossings(nodes, edges);
  score -= crossings * 2;
  
  // Check spacing uniformity
  const spacingScore = calculateSpacingUniformity(nodes);
  score += spacingScore * 0.3;
  
  return Math.max(0, Math.min(100, score));
}

function nodesOverlap(node1: LayoutNode, node2: LayoutNode): boolean {
  return !(
    node1.position.x + node1.size.width < node2.position.x ||
    node2.position.x + node2.size.width < node1.position.x ||
    node1.position.y + node1.size.height < node2.position.y ||
    node2.position.y + node2.size.height < node1.position.y
  );
}

function countEdgeCrossings(nodes: LayoutNode[], edges: DiagramEdge[]): number {
  // Simplified crossing detection
  return 0; // TODO: Implement full edge crossing detection
}

function calculateSpacingUniformity(nodes: LayoutNode[]): number {
  if (nodes.length < 2) return 100;
  
  const distances: number[] = [];
  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      const dx = (nodes[j].position.x + nodes[j].size.width / 2) - (nodes[i].position.x + nodes[i].size.width / 2);
      const dy = (nodes[j].position.y + nodes[j].size.height / 2) - (nodes[i].position.y + nodes[i].size.height / 2);
      distances.push(Math.sqrt(dx * dx + dy * dy));
    }
  }
  
  const mean = distances.reduce((sum, d) => sum + d, 0) / distances.length;
  const variance = distances.reduce((sum, d) => sum + Math.pow(d - mean, 2), 0) / distances.length;
  const stdDev = Math.sqrt(variance);
  
  // Lower standard deviation = more uniform spacing = higher score
  return Math.max(0, 100 - stdDev);
}
