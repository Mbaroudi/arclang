/**
 * Periphery Constraint Layout Algorithm
 * 
 * Enforces Capella/Arcadia requirement that actors MUST be positioned
 * on the diagram periphery (edges) for OA and SA layers.
 * 
 * Based on LaTeX specification Section 4 (Layout Rules):
 * - OAB: Actors on periphery, entities as containers
 * - SAB: System centered, actors on periphery outside system boundary
 */

import { DiagramNode, DiagramEdge, Point, Size } from '../types/diagram';

export interface PeripheryConfig {
  diagramType: 'OAB' | 'SAB' | 'other';
  centerNodeId?: string;
  peripheryPadding: number;
  symmetricalLayout: boolean;
}

export interface PeripheryLayoutResult {
  nodes: Array<DiagramNode & { position: Point; size: Size }>;
  edges: DiagramEdge[];
  totalSize: Size;
  actorsOnPeriphery: string[];
  centerNode?: DiagramNode & { position: Point; size: Size };
}

const DEFAULT_CONFIG: PeripheryConfig = {
  diagramType: 'other',
  peripheryPadding: 50,
  symmetricalLayout: true,
};

/**
 * Apply periphery constraint to layout
 * Ensures actors are positioned on diagram edges
 */
export function applyPeripheryConstraint(
  nodes: DiagramNode[],
  edges: DiagramEdge[],
  config: Partial<PeripheryConfig> = {}
): PeripheryLayoutResult {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  const actors: DiagramNode[] = [];
  const entities: DiagramNode[] = [];
  const centerNode = nodes.find(n => n.id === cfg.centerNodeId);

  for (const node of nodes) {
    const isActor = node.type === 'actor' || 
                    node.metadata?.is_actor === true ||
                    (node.metadata?.swimlaneLabel && cfg.diagramType === 'OAB');
    
    if (isActor) {
      actors.push(node);
    } else if (node.id !== cfg.centerNodeId) {
      entities.push(node);
    }
  }

  if (cfg.diagramType === 'OAB') {
    return layoutOAB(actors, entities, edges, cfg);
  } else if (cfg.diagramType === 'SAB') {
    return layoutSAB(actors, entities, centerNode, edges, cfg);
  } else {
    return layoutGeneric(nodes, edges, cfg);
  }
}

/**
 * Layout for Operational Architecture Blank (OAB)
 * - Actors on periphery (symmetrical)
 * - Entities as containers in center
 * - Activities allocated inside entities
 */
function layoutOAB(
  actors: DiagramNode[],
  entities: DiagramNode[],
  edges: DiagramEdge[],
  config: PeripheryConfig
): PeripheryLayoutResult {
  const layoutNodes: Array<DiagramNode & { position: Point; size: Size }> = [];
  
  const centerX = 800;
  const centerY = 600;
  const peripheryRadius = 500;
  const actorSize = { width: 150, height: 100 };
  const entitySize = { width: 300, height: 200 };

  if (config.symmetricalLayout) {
    const angleStep = (2 * Math.PI) / actors.length;
    
    actors.forEach((actor, index) => {
      const angle = index * angleStep - Math.PI / 2;
      const x = centerX + peripheryRadius * Math.cos(angle) - actorSize.width / 2;
      const y = centerY + peripheryRadius * Math.sin(angle) - actorSize.height / 2;
      
      layoutNodes.push({
        ...actor,
        position: { x, y },
        size: actor.size || actorSize,
      });
    });
  } else {
    positionActorsOnEdges(actors, centerX * 2, centerY * 2, actorSize, config.peripheryPadding)
      .forEach(node => layoutNodes.push(node));
  }

  const entitiesPerRow = Math.ceil(Math.sqrt(entities.length));
  const entitySpacing = 50;
  const totalEntityWidth = entitiesPerRow * (entitySize.width + entitySpacing);
  const startX = centerX - totalEntityWidth / 2;
  const startY = centerY - (entities.length / entitiesPerRow) * (entitySize.height + entitySpacing) / 2;

  entities.forEach((entity, index) => {
    const row = Math.floor(index / entitiesPerRow);
    const col = index % entitiesPerRow;
    const x = startX + col * (entitySize.width + entitySpacing);
    const y = startY + row * (entitySize.height + entitySpacing);
    
    layoutNodes.push({
      ...entity,
      position: { x, y },
      size: entity.size || entitySize,
    });
  });

  const totalSize = calculateBoundingBox(layoutNodes, config.peripheryPadding);

  return {
    nodes: layoutNodes,
    edges,
    totalSize,
    actorsOnPeriphery: actors.map(a => a.id),
  };
}

/**
 * Layout for System Architecture Blank (SAB)
 * - System centered with clear boundary
 * - Actors on periphery outside system
 * - Functions inside system boundary
 */
function layoutSAB(
  actors: DiagramNode[],
  entities: DiagramNode[],
  centerNode: DiagramNode | undefined,
  edges: DiagramEdge[],
  config: PeripheryConfig
): PeripheryLayoutResult {
  const layoutNodes: Array<DiagramNode & { position: Point; size: Size }> = [];
  
  const centerX = 800;
  const centerY = 600;
  const systemSize = { width: 800, height: 600 };
  const actorSize = { width: 150, height: 100 };
  const peripheryMargin = 150;

  let systemNode: DiagramNode & { position: Point; size: Size } | undefined;

  if (centerNode) {
    systemNode = {
      ...centerNode,
      position: { 
        x: centerX - systemSize.width / 2, 
        y: centerY - systemSize.height / 2 
      },
      size: centerNode.size || systemSize,
    };
    layoutNodes.push(systemNode);
  }

  entities.forEach((entity, index) => {
    const entitiesPerRow = Math.ceil(Math.sqrt(entities.length));
    const row = Math.floor(index / entitiesPerRow);
    const col = index % entitiesPerRow;
    const spacing = 80;
    const x = centerX - (entitiesPerRow * spacing) / 2 + col * spacing;
    const y = centerY - (Math.ceil(entities.length / entitiesPerRow) * spacing) / 2 + row * spacing;
    
    layoutNodes.push({
      ...entity,
      position: { x: x - 40, y: y - 30 },
      size: entity.size || { width: 80, height: 60 },
    });
  });

  const actorPositions = calculatePeripheryPositions(
    actors.length,
    centerX,
    centerY,
    Math.max(systemSize.width, systemSize.height) / 2 + peripheryMargin,
    config.symmetricalLayout
  );

  actors.forEach((actor, index) => {
    const pos = actorPositions[index];
    layoutNodes.push({
      ...actor,
      position: { 
        x: pos.x - actorSize.width / 2, 
        y: pos.y - actorSize.height / 2 
      },
      size: actor.size || actorSize,
    });
  });

  const totalSize = calculateBoundingBox(layoutNodes, config.peripheryPadding);

  return {
    nodes: layoutNodes,
    edges,
    totalSize,
    actorsOnPeriphery: actors.map(a => a.id),
    centerNode: systemNode,
  };
}

/**
 * Generic layout (no periphery constraint)
 */
function layoutGeneric(
  nodes: DiagramNode[],
  edges: DiagramEdge[],
  config: PeripheryConfig
): PeripheryLayoutResult {
  const layoutNodes: Array<DiagramNode & { position: Point; size: Size }> = [];
  
  const cols = Math.ceil(Math.sqrt(nodes.length));
  const spacing = 200;

  nodes.forEach((node, index) => {
    const row = Math.floor(index / cols);
    const col = index % cols;
    
    layoutNodes.push({
      ...node,
      position: { 
        x: col * spacing + 100, 
        y: row * spacing + 100 
      },
      size: node.size || { width: 150, height: 100 },
    });
  });

  const totalSize = calculateBoundingBox(layoutNodes, config.peripheryPadding);

  return {
    nodes: layoutNodes,
    edges,
    totalSize,
    actorsOnPeriphery: [],
  };
}

/**
 * Calculate positions on periphery (circle or edges)
 */
function calculatePeripheryPositions(
  count: number,
  centerX: number,
  centerY: number,
  radius: number,
  symmetrical: boolean
): Point[] {
  const positions: Point[] = [];
  
  if (symmetrical) {
    const angleStep = (2 * Math.PI) / count;
    
    for (let i = 0; i < count; i++) {
      const angle = i * angleStep - Math.PI / 2;
      positions.push({
        x: centerX + radius * Math.cos(angle),
        y: centerY + radius * Math.sin(angle),
      });
    }
  } else {
    const perSide = Math.ceil(count / 4);
    let index = 0;
    
    for (let side = 0; side < 4; side++) {
      const sideCount = Math.min(perSide, count - index);
      for (let i = 0; i < sideCount; i++) {
        const ratio = (i + 1) / (sideCount + 1);
        let x: number, y: number;
        
        switch (side) {
          case 0:
            x = centerX - radius + ratio * 2 * radius;
            y = centerY - radius;
            break;
          case 1:
            x = centerX + radius;
            y = centerY - radius + ratio * 2 * radius;
            break;
          case 2:
            x = centerX + radius - ratio * 2 * radius;
            y = centerY + radius;
            break;
          case 3:
            x = centerX - radius;
            y = centerY + radius - ratio * 2 * radius;
            break;
          default:
            x = centerX;
            y = centerY;
        }
        
        positions.push({ x, y });
        index++;
      }
    }
  }
  
  return positions;
}

/**
 * Position actors on diagram edges
 */
function positionActorsOnEdges(
  actors: DiagramNode[],
  diagramWidth: number,
  diagramHeight: number,
  actorSize: Size,
  padding: number
): Array<DiagramNode & { position: Point; size: Size }> {
  const layoutNodes: Array<DiagramNode & { position: Point; size: Size }> = [];
  const perSide = Math.ceil(actors.length / 4);

  actors.forEach((actor, index) => {
    const side = Math.floor(index / perSide);
    const sideIndex = index % perSide;
    const sideCount = Math.min(perSide, actors.length - side * perSide);
    const ratio = (sideIndex + 1) / (sideCount + 1);

    let x: number, y: number;

    switch (side) {
      case 0:
        x = ratio * diagramWidth - actorSize.width / 2;
        y = padding;
        break;
      case 1:
        x = diagramWidth - actorSize.width - padding;
        y = ratio * diagramHeight - actorSize.height / 2;
        break;
      case 2:
        x = diagramWidth - ratio * diagramWidth - actorSize.width / 2;
        y = diagramHeight - actorSize.height - padding;
        break;
      case 3:
        x = padding;
        y = diagramHeight - ratio * diagramHeight - actorSize.height / 2;
        break;
      default:
        x = diagramWidth / 2;
        y = diagramHeight / 2;
    }

    layoutNodes.push({
      ...actor,
      position: { x, y },
      size: actor.size || actorSize,
    });
  });

  return layoutNodes;
}

/**
 * Calculate bounding box of all nodes
 */
function calculateBoundingBox(
  nodes: Array<DiagramNode & { position: Point; size: Size }>,
  padding: number
): Size {
  if (nodes.length === 0) {
    return { width: 800, height: 600 };
  }

  let minX = Infinity, minY = Infinity;
  let maxX = -Infinity, maxY = -Infinity;

  for (const node of nodes) {
    minX = Math.min(minX, node.position.x);
    minY = Math.min(minY, node.position.y);
    maxX = Math.max(maxX, node.position.x + node.size.width);
    maxY = Math.max(maxY, node.position.y + node.size.height);
  }

  return {
    width: maxX - minX + 2 * padding,
    height: maxY - minY + 2 * padding,
  };
}

/**
 * Validate periphery constraint
 * Returns percentage of actors correctly positioned on periphery
 */
export function validatePeripheryConstraint(
  layout: PeripheryLayoutResult,
  tolerance: number = 50
): {
  valid: boolean;
  score: number;
  violations: string[];
} {
  const violations: string[] = [];
  let correctCount = 0;

  for (const actorId of layout.actorsOnPeriphery) {
    const node = layout.nodes.find(n => n.id === actorId);
    if (!node) continue;

    const isOnEdge = 
      node.position.x <= tolerance ||
      node.position.y <= tolerance ||
      node.position.x + node.size.width >= layout.totalSize.width - tolerance ||
      node.position.y + node.size.height >= layout.totalSize.height - tolerance;

    if (isOnEdge) {
      correctCount++;
    } else {
      violations.push(`Actor ${actorId} not on periphery: (${Math.round(node.position.x)}, ${Math.round(node.position.y)})`);
    }
  }

  const score = layout.actorsOnPeriphery.length > 0 
    ? (correctCount / layout.actorsOnPeriphery.length) * 100 
    : 100;

  return {
    valid: score === 100,
    score,
    violations,
  };
}
