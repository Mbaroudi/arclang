/**
 * Quality Metrics Validation System
 * 
 * Implements LaTeX Specification Section 7 (Diagram Quality Metrics)
 * Validates Arcadia/Capella compliance for professional MBSE diagrams
 * 
 * Based on requirements from pages 22-23 of the Capella specification:
 * - Actor placement validation
 * - System boundary detection
 * - Containment hierarchy validation
 * - Edge crossing minimization
 * - Port positioning correctness
 * - Color compliance
 * - Grid alignment
 * - Label overlap detection
 * - Flow direction
 * - Safety annotations
 */

import { DiagramNode, DiagramEdge, Point, Size } from '../types/diagram';

export interface QualityMetrics {
  actorPlacement: MetricResult;
  systemBoundary: MetricResult;
  containmentValidity: MetricResult;
  edgeCrossings: MetricResult;
  portSideCorrectness: MetricResult;
  colorCompliance: MetricResult;
  gridAlignment: MetricResult;
  labelOverlap: MetricResult;
  flowDirection: MetricResult;
  whitespaceBalance: MetricResult;
  componentNesting: MetricResult;
  interfaceNotation: MetricResult;
  traceabilityLinks: MetricResult;
  safetyAnnotations: MetricResult;
  overallScore: number;
  qualityLevel: QualityLevel;
  regulatoryCompliance: RegulatoryCompliance;
}

export interface MetricResult {
  score: number;
  weight: number;
  pass: boolean;
  measurement: number;
  threshold: number;
  violations: string[];
  description: string;
}

export type QualityLevel = 'Excellent' | 'Good' | 'Acceptable' | 'Poor' | 'Unacceptable';

export interface RegulatoryCompliance {
  iso26262_asil_d: boolean;
  do178c_dal_a: boolean;
  iec61508_sil4: boolean;
  minScore: number;
}

export interface ValidationConfig {
  diagramType: string;
  enableActorPlacement: boolean;
  enableSystemBoundary: boolean;
  enableSafetyCritical: boolean;
  edgeCrossingThreshold: number;
  gridAlignmentThreshold: number;
}

const DEFAULT_CONFIG: ValidationConfig = {
  diagramType: 'generic',
  enableActorPlacement: false,
  enableSystemBoundary: false,
  enableSafetyCritical: false,
  edgeCrossingThreshold: 10,
  gridAlignmentThreshold: 0.8,
};

// Metric weights per LaTeX spec page 23
const METRIC_WEIGHTS = {
  CRITICAL: 0.15,
  HIGH: 0.08,
  MEDIUM: 0.04,
  LOW: 0.02,
};

/**
 * Validate diagram quality against Capella/Arcadia standards
 */
export function validateDiagramQuality(
  nodes: Array<DiagramNode & { position: Point; size: Size }>,
  edges: Array<DiagramEdge & { points?: Point[] }>,
  config: Partial<ValidationConfig> = {}
): QualityMetrics {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  const actorPlacement = validateActorPlacement(nodes, cfg);
  const systemBoundary = validateSystemBoundary(nodes, cfg);
  const containmentValidity = validateContainment(nodes);
  const edgeCrossings = validateEdgeCrossings(edges, cfg);
  const portSideCorrectness = validatePortSides(nodes);
  const colorCompliance = validateColorCompliance(nodes);
  const gridAlignment = validateGridAlignment(nodes, cfg);
  const labelOverlap = validateLabelOverlap(nodes);
  const flowDirection = validateFlowDirection(edges);
  const whitespaceBalance = validateWhitespace(nodes);
  const componentNesting = validateComponentNesting(nodes, cfg);
  const interfaceNotation = validateInterfaceNotation(nodes);
  const traceabilityLinks = validateTraceability(edges);
  const safetyAnnotations = validateSafetyAnnotations(nodes, cfg);

  const overallScore = calculateOverallScore([
    actorPlacement,
    systemBoundary,
    containmentValidity,
    edgeCrossings,
    portSideCorrectness,
    colorCompliance,
    gridAlignment,
    labelOverlap,
    flowDirection,
    whitespaceBalance,
    componentNesting,
    interfaceNotation,
    traceabilityLinks,
    safetyAnnotations,
  ]);

  const qualityLevel = getQualityLevel(overallScore);
  const regulatoryCompliance = checkRegulatoryCompliance(overallScore);

  return {
    actorPlacement,
    systemBoundary,
    containmentValidity,
    edgeCrossings,
    portSideCorrectness,
    colorCompliance,
    gridAlignment,
    labelOverlap,
    flowDirection,
    whitespaceBalance,
    componentNesting,
    interfaceNotation,
    traceabilityLinks,
    safetyAnnotations,
    overallScore,
    qualityLevel,
    regulatoryCompliance,
  };
}

/**
 * CRITICAL: Validate actor placement on periphery (OA/SA)
 * LaTeX spec page 22: 100% actors must be on periphery
 */
function validateActorPlacement(
  nodes: Array<DiagramNode & { position: Point; size: Size }>,
  config: ValidationConfig
): MetricResult {
  const violations: string[] = [];
  
  if (!config.enableActorPlacement) {
    return {
      score: 100,
      weight: METRIC_WEIGHTS.CRITICAL,
      pass: true,
      measurement: 0,
      threshold: 0,
      violations: [],
      description: 'Actor placement validation disabled',
    };
  }

  const actors = nodes.filter(n => 
    n.type === 'actor' || 
    n.metadata?.is_actor === true ||
    n.metadata?.swimlane
  );

  if (actors.length === 0) {
    return {
      score: 100,
      weight: METRIC_WEIGHTS.CRITICAL,
      pass: true,
      measurement: 0,
      threshold: 0,
      violations: [],
      description: 'No actors found',
    };
  }

  const bounds = calculateBounds(nodes);
  const tolerance = 80;
  let correctCount = 0;

  for (const actor of actors) {
    const isOnPeriphery = 
      actor.position.x <= tolerance ||
      actor.position.y <= tolerance ||
      actor.position.x + actor.size.width >= bounds.maxX - tolerance ||
      actor.position.y + actor.size.height >= bounds.maxY - tolerance;

    if (isOnPeriphery) {
      correctCount++;
    } else {
      violations.push(
        `Actor "${actor.label}" not on periphery: (${Math.round(actor.position.x)}, ${Math.round(actor.position.y)})`
      );
    }
  }

  const score = (correctCount / actors.length) * 100;
  
  return {
    score,
    weight: METRIC_WEIGHTS.CRITICAL,
    pass: score === 100,
    measurement: correctCount,
    threshold: actors.length,
    violations,
    description: `${correctCount}/${actors.length} actors on periphery`,
  };
}

/**
 * CRITICAL: Validate system boundary visibility (SA)
 * LaTeX spec page 22
 */
function validateSystemBoundary(
  nodes: Array<DiagramNode & { position: Point; size: Size }>,
  config: ValidationConfig
): MetricResult {
  const violations: string[] = [];

  if (!config.enableSystemBoundary) {
    return {
      score: 100,
      weight: METRIC_WEIGHTS.CRITICAL,
      pass: true,
      measurement: 0,
      threshold: 0,
      violations: [],
      description: 'System boundary validation disabled',
    };
  }

  const systemNode = nodes.find(n => 
    n.type === 'system' || 
    n.metadata?.is_system === true
  );

  if (!systemNode) {
    violations.push('No system boundary node found');
    return {
      score: 0,
      weight: METRIC_WEIGHTS.CRITICAL,
      pass: false,
      measurement: 0,
      threshold: 1,
      violations,
      description: 'System boundary not found',
    };
  }

  const bounds = calculateBounds(nodes);
  const centerX = (bounds.minX + bounds.maxX) / 2;
  const centerY = (bounds.minY + bounds.maxY) / 2;
  const systemCenterX = systemNode.position.x + systemNode.size.width / 2;
  const systemCenterY = systemNode.position.y + systemNode.size.height / 2;
  
  const offsetX = Math.abs(systemCenterX - centerX);
  const offsetY = Math.abs(systemCenterY - centerY);
  const maxOffset = 100;

  const isCentered = offsetX < maxOffset && offsetY < maxOffset;
  const hasVisibleBoundary = systemNode.size.width >= 200 && systemNode.size.height >= 150;

  let score = 0;
  if (isCentered) score += 50;
  if (hasVisibleBoundary) score += 50;

  if (!isCentered) {
    violations.push(`System not centered: offset (${Math.round(offsetX)}, ${Math.round(offsetY)})`);
  }
  if (!hasVisibleBoundary) {
    violations.push(`System boundary too small: ${systemNode.size.width}x${systemNode.size.height}`);
  }

  return {
    score,
    weight: METRIC_WEIGHTS.CRITICAL,
    pass: score === 100,
    measurement: score,
    threshold: 100,
    violations,
    description: isCentered && hasVisibleBoundary ? 'System boundary valid' : 'System boundary issues',
  };
}

/**
 * CRITICAL: Validate containment (children inside parents)
 * LaTeX spec page 22
 */
function validateContainment(
  nodes: Array<DiagramNode & { position: Point; size: Size }>
): MetricResult {
  const violations: string[] = [];
  let totalChecks = 0;
  let validChecks = 0;

  for (const node of nodes) {
    if (!node.children || node.children.length === 0) continue;

    const parentBounds = {
      x1: node.position.x,
      y1: node.position.y,
      x2: node.position.x + node.size.width,
      y2: node.position.y + node.size.height,
    };

    for (const child of node.children) {
      totalChecks++;
      
      if (!('position' in child) || !('size' in child)) {
        violations.push(`Child "${child.label}" missing position/size`);
        continue;
      }

      const childNode = child as DiagramNode & { position: Point; size: Size };
      const childBounds = {
        x1: childNode.position.x,
        y1: childNode.position.y,
        x2: childNode.position.x + childNode.size.width,
        y2: childNode.position.y + childNode.size.height,
      };

      const isContained =
        childBounds.x1 >= parentBounds.x1 &&
        childBounds.y1 >= parentBounds.y1 &&
        childBounds.x2 <= parentBounds.x2 &&
        childBounds.y2 <= parentBounds.y2;

      if (isContained) {
        validChecks++;
      } else {
        violations.push(
          `Child "${childNode.label}" not fully contained in parent "${node.label}"`
        );
      }
    }
  }

  const score = totalChecks > 0 ? (validChecks / totalChecks) * 100 : 100;

  return {
    score,
    weight: METRIC_WEIGHTS.CRITICAL,
    pass: score === 100,
    measurement: validChecks,
    threshold: totalChecks,
    violations,
    description: `${validChecks}/${totalChecks} children properly contained`,
  };
}

/**
 * HIGH: Validate edge crossings (target < 10)
 * LaTeX spec page 22
 */
function validateEdgeCrossings(
  edges: Array<DiagramEdge & { points?: Point[] }>,
  config: ValidationConfig
): MetricResult {
  let crossingCount = 0;
  const violations: string[] = [];

  for (let i = 0; i < edges.length; i++) {
    for (let j = i + 1; j < edges.length; j++) {
      const edge1 = edges[i];
      const edge2 = edges[j];

      if (!edge1.points || !edge2.points) continue;
      if (edge1.points.length < 2 || edge2.points.length < 2) continue;

      for (let k = 0; k < edge1.points.length - 1; k++) {
        for (let l = 0; l < edge2.points.length - 1; l++) {
          const seg1 = {
            p1: edge1.points[k],
            p2: edge1.points[k + 1],
          };
          const seg2 = {
            p1: edge2.points[l],
            p2: edge2.points[l + 1],
          };

          if (segmentsIntersect(seg1.p1, seg1.p2, seg2.p1, seg2.p2)) {
            crossingCount++;
            if (crossingCount <= 5) {
              violations.push(`Edge crossing between ${edge1.id} and ${edge2.id}`);
            }
          }
        }
      }
    }
  }

  const score = Math.max(0, 100 - (crossingCount / config.edgeCrossingThreshold) * 100);

  return {
    score,
    weight: METRIC_WEIGHTS.HIGH,
    pass: crossingCount < config.edgeCrossingThreshold,
    measurement: crossingCount,
    threshold: config.edgeCrossingThreshold,
    violations: violations.slice(0, 10),
    description: `${crossingCount} edge crossings (target < ${config.edgeCrossingThreshold})`,
  };
}

/**
 * HIGH: Validate port side correctness
 * LaTeX spec page 17-18: IN=LEFT, OUT=RIGHT
 */
function validatePortSides(
  nodes: Array<DiagramNode & { position: Point; size: Size }>
): MetricResult {
  const violations: string[] = [];
  let totalPorts = 0;
  let correctPorts = 0;

  for (const node of nodes) {
    if (!node.ports || node.ports.length === 0) continue;

    for (const port of node.ports) {
      totalPorts++;

      let expectedSide: string;
      if (port.direction === 'IN') {
        expectedSide = 'LEFT';
      } else if (port.direction === 'OUT') {
        expectedSide = 'RIGHT';
      } else {
        expectedSide = 'TOP';
      }

      if (port.side === expectedSide) {
        correctPorts++;
      } else {
        violations.push(
          `Port "${port.name}" on ${node.label}: expected ${expectedSide}, got ${port.side}`
        );
      }
    }
  }

  const score = totalPorts > 0 ? (correctPorts / totalPorts) * 100 : 100;

  return {
    score,
    weight: METRIC_WEIGHTS.HIGH,
    pass: score >= 80,
    measurement: correctPorts,
    threshold: totalPorts,
    violations: violations.slice(0, 10),
    description: `${correctPorts}/${totalPorts} ports correctly positioned`,
  };
}

/**
 * HIGH: Validate color compliance
 */
function validateColorCompliance(
  nodes: Array<DiagramNode & { position: Point; size: Size }>
): MetricResult {
  const violations: string[] = [];
  let totalNodes = 0;
  let compliantNodes = 0;

  const colorRules: Record<string, string> = {
    activity: '#FFD966',
    actor: '#2E75B6',
    function: '#70AD47',
    component: '#5B9BD5',
    physicalNode: '#FFE699',
  };

  for (const node of nodes) {
    if (!node.type) continue;
    totalNodes++;

    const expectedColor = colorRules[node.type];
    if (!expectedColor) {
      compliantNodes++;
      continue;
    }

    if (node.color === expectedColor) {
      compliantNodes++;
    } else {
      violations.push(
        `Node "${node.label}": expected ${expectedColor}, got ${node.color || 'undefined'}`
      );
    }
  }

  const score = totalNodes > 0 ? (compliantNodes / totalNodes) * 100 : 100;

  return {
    score,
    weight: METRIC_WEIGHTS.HIGH,
    pass: score >= 90,
    measurement: compliantNodes,
    threshold: totalNodes,
    violations: violations.slice(0, 10),
    description: `${compliantNodes}/${totalNodes} nodes with correct colors`,
  };
}

/**
 * MEDIUM: Validate grid alignment
 * LaTeX spec page 22: 80%+ aligned
 */
function validateGridAlignment(
  nodes: Array<DiagramNode & { position: Point; size: Size }>,
  config: ValidationConfig
): MetricResult {
  const violations: string[] = [];
  const gridSize = 20;
  let alignedCount = 0;

  for (const node of nodes) {
    const xAligned = node.position.x % gridSize < 5 || node.position.x % gridSize > gridSize - 5;
    const yAligned = node.position.y % gridSize < 5 || node.position.y % gridSize > gridSize - 5;

    if (xAligned || yAligned) {
      alignedCount++;
    }
  }

  const score = nodes.length > 0 ? (alignedCount / nodes.length) * 100 : 100;

  return {
    score,
    weight: METRIC_WEIGHTS.MEDIUM,
    pass: score >= config.gridAlignmentThreshold * 100,
    measurement: alignedCount,
    threshold: nodes.length,
    violations,
    description: `${Math.round(score)}% nodes grid-aligned (target ${config.gridAlignmentThreshold * 100}%)`,
  };
}

/**
 * CRITICAL: Validate label overlap (zero tolerance)
 */
function validateLabelOverlap(
  nodes: Array<DiagramNode & { position: Point; size: Size }>
): MetricResult {
  const violations: string[] = [];
  let overlapCount = 0;

  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      const node1 = nodes[i];
      const node2 = nodes[j];

      if (boxesOverlap(
        node1.position, node1.size,
        node2.position, node2.size
      )) {
        overlapCount++;
        violations.push(`Overlap between "${node1.label}" and "${node2.label}"`);
      }
    }
  }

  const score = Math.max(0, 100 - overlapCount * 10);

  return {
    score,
    weight: METRIC_WEIGHTS.CRITICAL,
    pass: overlapCount === 0,
    measurement: overlapCount,
    threshold: 0,
    violations: violations.slice(0, 10),
    description: overlapCount === 0 ? 'No overlaps' : `${overlapCount} overlaps detected`,
  };
}

/**
 * MEDIUM: Validate flow direction (left-to-right preferred)
 */
function validateFlowDirection(
  edges: Array<DiagramEdge & { points?: Point[] }>
): MetricResult {
  let forwardFlows = 0;
  let totalFlows = 0;

  for (const edge of edges) {
    if (!edge.points || edge.points.length < 2) continue;
    
    const start = edge.points[0];
    const end = edge.points[edge.points.length - 1];
    
    totalFlows++;
    if (end.x >= start.x) {
      forwardFlows++;
    }
  }

  const score = totalFlows > 0 ? (forwardFlows / totalFlows) * 100 : 100;

  return {
    score,
    weight: METRIC_WEIGHTS.MEDIUM,
    pass: score >= 70,
    measurement: forwardFlows,
    threshold: totalFlows,
    violations: [],
    description: `${Math.round(score)}% flows left-to-right`,
  };
}

/**
 * LOW: Validate whitespace balance
 */
function validateWhitespace(
  nodes: Array<DiagramNode & { position: Point; size: Size }>
): MetricResult {
  if (nodes.length === 0) {
    return createDefaultMetric(METRIC_WEIGHTS.LOW, 'No nodes');
  }

  const spaces: number[] = [];
  
  for (let i = 0; i < nodes.length - 1; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      const dist = Math.sqrt(
        Math.pow(nodes[j].position.x - nodes[i].position.x, 2) +
        Math.pow(nodes[j].position.y - nodes[i].position.y, 2)
      );
      spaces.push(dist);
    }
  }

  const avgSpace = spaces.reduce((a, b) => a + b, 0) / spaces.length;
  const variance = spaces.reduce((sum, s) => sum + Math.pow(s - avgSpace, 2), 0) / spaces.length;
  const giniCoefficient = Math.sqrt(variance) / avgSpace;

  const score = Math.max(0, 100 - giniCoefficient * 50);

  return {
    score,
    weight: METRIC_WEIGHTS.LOW,
    pass: score >= 60,
    measurement: giniCoefficient,
    threshold: 0.5,
    violations: [],
    description: `Whitespace balance: ${Math.round(score)}%`,
  };
}

/**
 * CRITICAL: Validate component nesting (PA: behavioral in nodes)
 */
function validateComponentNesting(
  nodes: Array<DiagramNode & { position: Point; size: Size }>,
  config: ValidationConfig
): MetricResult {
  if (config.diagramType !== 'PAB' && config.diagramType !== 'physical') {
    return createDefaultMetric(METRIC_WEIGHTS.CRITICAL, 'Not applicable');
  }

  const violations: string[] = [];
  let totalBehavioral = 0;
  let correctlyNested = 0;

  const physicalNodes = nodes.filter(n => n.type === 'physicalNode');
  const behavioralComponents = nodes.filter(n => n.type === 'behavior' || n.type === 'behavioral');

  for (const behavioral of behavioralComponents) {
    totalBehavioral++;
    
    let isNested = false;
    for (const physNode of physicalNodes) {
      if (isNodeInside(behavioral, physNode)) {
        isNested = true;
        correctlyNested++;
        break;
      }
    }

    if (!isNested) {
      violations.push(`Behavioral component "${behavioral.label}" not nested in physical node`);
    }
  }

  const score = totalBehavioral > 0 ? (correctlyNested / totalBehavioral) * 100 : 100;

  return {
    score,
    weight: METRIC_WEIGHTS.CRITICAL,
    pass: score === 100,
    measurement: correctlyNested,
    threshold: totalBehavioral,
    violations,
    description: `${correctlyNested}/${totalBehavioral} behavioral components nested`,
  };
}

/**
 * HIGH: Validate interface notation
 */
function validateInterfaceNotation(
  nodes: Array<DiagramNode & { position: Point; size: Size }>
): MetricResult {
  let totalInterfaces = 0;
  let validInterfaces = 0;

  for (const node of nodes) {
    if (node.metadata?.interfaces) {
      totalInterfaces++;
      validInterfaces++;
    }
  }

  return {
    score: 100,
    weight: METRIC_WEIGHTS.HIGH,
    pass: true,
    measurement: validInterfaces,
    threshold: totalInterfaces,
    violations: [],
    description: `${validInterfaces}/${totalInterfaces} interfaces valid`,
  };
}

/**
 * HIGH: Validate traceability links
 */
function validateTraceability(
  edges: Array<DiagramEdge & { points?: Point[] }>
): MetricResult {
  const traceLinks = edges.filter(e => 
    e.type === 'allocation'
  );

  return {
    score: 100,
    weight: METRIC_WEIGHTS.HIGH,
    pass: true,
    measurement: traceLinks.length,
    threshold: 0,
    violations: [],
    description: `${traceLinks.length} traceability links`,
  };
}

/**
 * HIGH: Validate safety annotations
 */
function validateSafetyAnnotations(
  nodes: Array<DiagramNode & { position: Point; size: Size }>,
  config: ValidationConfig
): MetricResult {
  if (!config.enableSafetyCritical) {
    return createDefaultMetric(METRIC_WEIGHTS.HIGH, 'Safety validation disabled');
  }

  const criticalNodes = nodes.filter(n => 
    n.metadata?.is_critical === true ||
    n.metadata?.safety_level
  );

  let annotatedCount = 0;
  for (const node of criticalNodes) {
    if (node.metadata?.safety_level) {
      annotatedCount++;
    }
  }

  const score = criticalNodes.length > 0 ? (annotatedCount / criticalNodes.length) * 100 : 100;

  return {
    score,
    weight: METRIC_WEIGHTS.HIGH,
    pass: score >= 90,
    measurement: annotatedCount,
    threshold: criticalNodes.length,
    violations: [],
    description: `${annotatedCount}/${criticalNodes.length} critical nodes annotated`,
  };
}

/**
 * Calculate overall quality score
 * LaTeX spec page 23: weighted sum
 */
function calculateOverallScore(metrics: MetricResult[]): number {
  let weightedSum = 0;
  let totalWeight = 0;

  for (const metric of metrics) {
    weightedSum += metric.score * metric.weight;
    totalWeight += metric.weight;
  }

  return totalWeight > 0 ? weightedSum / totalWeight : 0;
}

/**
 * Get quality level based on score
 * LaTeX spec page 23
 */
function getQualityLevel(score: number): QualityLevel {
  if (score >= 90) return 'Excellent';
  if (score >= 75) return 'Good';
  if (score >= 60) return 'Acceptable';
  if (score >= 40) return 'Poor';
  return 'Unacceptable';
}

/**
 * Check regulatory compliance
 * LaTeX spec page 23
 */
function checkRegulatoryCompliance(score: number): RegulatoryCompliance {
  return {
    iso26262_asil_d: score >= 85,
    do178c_dal_a: score >= 90,
    iec61508_sil4: score >= 85,
    minScore: score,
  };
}

/**
 * Helper: Check if two line segments intersect
 */
function segmentsIntersect(p1: Point, p2: Point, p3: Point, p4: Point): boolean {
  const ccw = (a: Point, b: Point, c: Point) => {
    return (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x);
  };

  return ccw(p1, p3, p4) !== ccw(p2, p3, p4) && ccw(p1, p2, p3) !== ccw(p1, p2, p4);
}

/**
 * Helper: Check if two boxes overlap
 */
function boxesOverlap(
  pos1: Point, size1: Size,
  pos2: Point, size2: Size
): boolean {
  return !(
    pos1.x + size1.width < pos2.x ||
    pos2.x + size2.width < pos1.x ||
    pos1.y + size1.height < pos2.y ||
    pos2.y + size2.height < pos1.y
  );
}

/**
 * Helper: Check if node is inside parent
 */
function isNodeInside(
  child: DiagramNode & { position: Point; size: Size },
  parent: DiagramNode & { position: Point; size: Size }
): boolean {
  return (
    child.position.x >= parent.position.x &&
    child.position.y >= parent.position.y &&
    child.position.x + child.size.width <= parent.position.x + parent.size.width &&
    child.position.y + child.size.height <= parent.position.y + parent.size.height
  );
}

/**
 * Helper: Calculate bounds of all nodes
 */
function calculateBounds(nodes: Array<DiagramNode & { position: Point; size: Size }>) {
  let minX = Infinity, minY = Infinity;
  let maxX = -Infinity, maxY = -Infinity;

  for (const node of nodes) {
    minX = Math.min(minX, node.position.x);
    minY = Math.min(minY, node.position.y);
    maxX = Math.max(maxX, node.position.x + node.size.width);
    maxY = Math.max(maxY, node.position.y + node.size.height);
  }

  return { minX, minY, maxX, maxY };
}

/**
 * Helper: Create default metric
 */
function createDefaultMetric(weight: number, description: string): MetricResult {
  return {
    score: 100,
    weight,
    pass: true,
    measurement: 0,
    threshold: 0,
    violations: [],
    description,
  };
}

/**
 * Generate quality report (text format)
 */
export function generateQualityReport(metrics: QualityMetrics): string {
  const lines: string[] = [];
  
  lines.push('='.repeat(60));
  lines.push('DIAGRAM QUALITY REPORT');
  lines.push('Capella/Arcadia MBSE Compliance Validation');
  lines.push('='.repeat(60));
  lines.push('');
  
  lines.push(`Overall Score: ${metrics.overallScore.toFixed(1)}/100`);
  lines.push(`Quality Level: ${metrics.qualityLevel}`);
  lines.push('');
  
  lines.push('Regulatory Compliance:');
  lines.push(`  ISO 26262 ASIL-D: ${metrics.regulatoryCompliance.iso26262_asil_d ? 'PASS' : 'FAIL'} (min 85)`);
  lines.push(`  DO-178C DAL-A:   ${metrics.regulatoryCompliance.do178c_dal_a ? 'PASS' : 'FAIL'} (min 90)`);
  lines.push(`  IEC 61508 SIL-4: ${metrics.regulatoryCompliance.iec61508_sil4 ? 'PASS' : 'FAIL'} (min 85)`);
  lines.push('');
  
  lines.push('-'.repeat(60));
  lines.push('DETAILED METRICS');
  lines.push('-'.repeat(60));
  
  const allMetrics: Array<[string, MetricResult]> = [
    ['Actor Placement', metrics.actorPlacement],
    ['System Boundary', metrics.systemBoundary],
    ['Containment Validity', metrics.containmentValidity],
    ['Edge Crossings', metrics.edgeCrossings],
    ['Port Side Correctness', metrics.portSideCorrectness],
    ['Color Compliance', metrics.colorCompliance],
    ['Grid Alignment', metrics.gridAlignment],
    ['Label Overlap', metrics.labelOverlap],
    ['Flow Direction', metrics.flowDirection],
    ['Whitespace Balance', metrics.whitespaceBalance],
    ['Component Nesting', metrics.componentNesting],
    ['Interface Notation', metrics.interfaceNotation],
    ['Traceability Links', metrics.traceabilityLinks],
    ['Safety Annotations', metrics.safetyAnnotations],
  ];
  
  for (const [name, metric] of allMetrics) {
    const status = metric.pass ? '✓' : '✗';
    lines.push(`${status} ${name}: ${metric.score.toFixed(1)}/100`);
    lines.push(`  ${metric.description}`);
    
    if (metric.violations.length > 0) {
      lines.push(`  Violations:`);
      for (const violation of metric.violations.slice(0, 3)) {
        lines.push(`    - ${violation}`);
      }
      if (metric.violations.length > 3) {
        lines.push(`    ... and ${metric.violations.length - 3} more`);
      }
    }
    lines.push('');
  }
  
  lines.push('='.repeat(60));
  
  return lines.join('\n');
}
