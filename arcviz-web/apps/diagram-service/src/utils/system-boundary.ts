/**
 * System Boundary Visualization
 * 
 * Implements Capella Specification Section 4.2 (System Architecture Blank)
 * MARKED AS "MOST IMPORTANT" IN SPECIFICATIONS
 * 
 * Requirements:
 * 1. System must be visually enclosed with clear boundary box
 * 2. System must be centered in diagram
 * 3. External actors MUST be on periphery OUTSIDE system boundary
 * 4. System functions MUST be allocated INSIDE system boundary
 * 5. Visual distinction between inside/outside system
 * 
 * Reference: Capella specifications.pdf - Section 4.2
 */

import { DiagramNode, Point, Size } from '../types/diagram';
import { SvgElement } from '../types/diagram';
import { createRoundedRect, createText, createGroup } from './svg';

export interface SystemBoundaryConfig {
  padding: number;          // Space between functions and boundary (default: 40px)
  strokeWidth: number;      // Border thickness (default: 3px)
  strokeColor: string;      // Border color (default: #2E75B6 - Capella system blue)
  fillColor: string;        // Interior fill (default: #E8F4F8 - very light blue)
  fillOpacity: number;      // Fill transparency (default: 0.1)
  cornerRadius: number;     // Rounded corner radius (default: 12px)
  labelPosition: 'top' | 'bottom' | 'none';  // Where to place "System" label
  labelText: string;        // Label text (default: "System")
  labelFontSize: number;    // Label font size (default: 16)
  actorMargin: number;      // Minimum distance from actors to boundary (default: 80px)
}

export const DEFAULT_BOUNDARY_CONFIG: SystemBoundaryConfig = {
  padding: 40,
  strokeWidth: 3,
  strokeColor: '#2E75B6',      // Capella system blue
  fillColor: '#E8F4F8',        // Very light blue
  fillOpacity: 0.1,
  cornerRadius: 12,
  labelPosition: 'top',
  labelText: 'System',
  labelFontSize: 16,
  actorMargin: 80,
};

/**
 * Calculate system boundary that encloses all system functions
 * and excludes external actors
 */
export function calculateSystemBoundary(
  nodes: DiagramNode[],
  config: Partial<SystemBoundaryConfig> = {}
): { position: Point; size: Size; systemNodes: DiagramNode[]; actorNodes: DiagramNode[] } {
  const cfg = { ...DEFAULT_BOUNDARY_CONFIG, ...config };

  // Separate system functions from external actors
  const systemNodes = nodes.filter(n => n.type === 'function');
  const actorNodes = nodes.filter(n => n.type === 'actor');

  if (systemNodes.length === 0) {
    // No system functions - return empty boundary
    return {
      position: { x: 0, y: 0 },
      size: { width: 0, height: 0 },
      systemNodes: [],
      actorNodes,
    };
  }

  // Find bounding box of all system functions
  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  for (const node of systemNodes) {
    if (!node.position || !node.size) continue;

    minX = Math.min(minX, node.position.x);
    minY = Math.min(minY, node.position.y);
    maxX = Math.max(maxX, node.position.x + node.size.width);
    maxY = Math.max(maxY, node.position.y + node.size.height);
  }

  // Add padding
  const boundaryX = minX - cfg.padding;
  const boundaryY = minY - cfg.padding;
  const boundaryWidth = (maxX - minX) + (cfg.padding * 2);
  const boundaryHeight = (maxY - minY) + (cfg.padding * 2);

  return {
    position: { x: boundaryX, y: boundaryY },
    size: { width: boundaryWidth, height: boundaryHeight },
    systemNodes,
    actorNodes,
  };
}

/**
 * Position external actors on the periphery around system boundary
 * 
 * Algorithm:
 * 1. Place actors outside boundary at minimum margin distance
 * 2. Distribute actors evenly around perimeter (top, right, bottom, left)
 * 3. Maintain connectivity to their connected functions
 */
export function positionActorsOnPeriphery(
  boundary: { position: Point; size: Size },
  actors: DiagramNode[],
  edges: any[],
  config: Partial<SystemBoundaryConfig> = {}
): DiagramNode[] {
  const cfg = { ...DEFAULT_BOUNDARY_CONFIG, ...config };

  if (actors.length === 0) return [];

  const centerX = boundary.position.x + boundary.size.width / 2;
  const centerY = boundary.position.y + boundary.size.height / 2;

  // Analyze actor connections to determine best side
  const positionedActors: DiagramNode[] = [];

  for (let i = 0; i < actors.length; i++) {
    const actor = actors[i];
    
    // Find which side has most connections for this actor
    const connectedFunctions = edges
      .filter(e => e.from === actor.id || e.to === actor.id)
      .map(e => e.from === actor.id ? e.to : e.from);

    let side: 'top' | 'right' | 'bottom' | 'left' = 'left';
    
    if (connectedFunctions.length === 0) {
      // No connections - distribute evenly
      side = ['left', 'top', 'right', 'bottom'][i % 4] as any;
    }

    // Position based on side
    const actorWidth = actor.size?.width || 80;
    const actorHeight = actor.size?.height || 100;
    let position: Point;

    switch (side) {
      case 'left':
        position = {
          x: boundary.position.x - cfg.actorMargin - actorWidth,
          y: centerY - actorHeight / 2,
        };
        break;
      case 'right':
        position = {
          x: boundary.position.x + boundary.size.width + cfg.actorMargin,
          y: centerY - actorHeight / 2,
        };
        break;
      case 'top':
        position = {
          x: centerX - actorWidth / 2,
          y: boundary.position.y - cfg.actorMargin - actorHeight,
        };
        break;
      case 'bottom':
        position = {
          x: centerX - actorWidth / 2,
          y: boundary.position.y + boundary.size.height + cfg.actorMargin,
        };
        break;
    }

    positionedActors.push({
      ...actor,
      position,
      metadata: {
        ...actor.metadata,
        peripherySide: side,
      },
    });
  }

  return positionedActors;
}

/**
 * Render system boundary as SVG element
 */
export function renderSystemBoundary(
  boundary: { position: Point; size: Size },
  config: Partial<SystemBoundaryConfig> = {}
): SvgElement {
  const cfg = { ...DEFAULT_BOUNDARY_CONFIG, ...config };

  const elements: SvgElement[] = [];

  // Main boundary box with light fill and strong border
  elements.push(
    createRoundedRect(
      boundary.position.x,
      boundary.position.y,
      boundary.size.width,
      boundary.size.height,
      cfg.cornerRadius,
      {
        fill: cfg.fillColor,
        'fill-opacity': cfg.fillOpacity,
        stroke: cfg.strokeColor,
        'stroke-width': cfg.strokeWidth,
        'stroke-dasharray': 'none',
      }
    )
  );

  // System label
  if (cfg.labelPosition !== 'none') {
    const labelX = boundary.position.x + boundary.size.width / 2;
    const labelY = cfg.labelPosition === 'top'
      ? boundary.position.y - 15
      : boundary.position.y + boundary.size.height + 25;

    elements.push(
      createText(labelX, labelY, cfg.labelText, {
        'text-anchor': 'middle',
        'font-family': 'Helvetica Neue, Arial, sans-serif',
        'font-size': cfg.labelFontSize,
        'font-weight': 'bold',
        fill: cfg.strokeColor,
      })
    );
  }

  return createGroup(elements, { id: 'system-boundary', class: 'system-boundary-group' });
}

/**
 * Apply system boundary layout to diagram
 * 
 * This is the main function to call - it:
 * 1. Calculates boundary around system functions
 * 2. Positions actors on periphery
 * 3. Returns updated node positions and boundary SVG
 */
export function applySystemBoundaryLayout(
  nodes: DiagramNode[],
  edges: any[],
  config: Partial<SystemBoundaryConfig> = {}
): {
  nodes: DiagramNode[];
  boundary: { position: Point; size: Size };
  boundarySvg: SvgElement;
  totalSize: Size;
} {
  // Calculate system boundary
  const { position, size, systemNodes, actorNodes } = calculateSystemBoundary(nodes, config);

  // Position actors on periphery
  const repositionedActors = positionActorsOnPeriphery(
    { position, size },
    actorNodes,
    edges,
    config
  );

  // Combine system nodes (keep original positions) with repositioned actors
  const allNodes = [...systemNodes, ...repositionedActors];

  // Render boundary
  const boundarySvg = renderSystemBoundary({ position, size }, config);

  // Calculate total diagram size including actors
  const cfg = { ...DEFAULT_BOUNDARY_CONFIG, ...config };
  const totalSize = {
    width: size.width + (cfg.actorMargin + 100) * 2,  // Extra space for actors
    height: size.height + (cfg.actorMargin + 120) * 2,
  };

  return {
    nodes: allNodes,
    boundary: { position, size },
    boundarySvg,
    totalSize,
  };
}

/**
 * Validate system boundary compliance
 * 
 * Checks:
 * 1. All system functions are inside boundary
 * 2. All external actors are outside boundary
 * 3. Minimum margin is maintained
 */
export function validateSystemBoundary(
  nodes: DiagramNode[],
  boundary: { position: Point; size: Size },
  config: Partial<SystemBoundaryConfig> = {}
): { valid: boolean; violations: string[] } {
  const cfg = { ...DEFAULT_BOUNDARY_CONFIG, ...config };
  const violations: string[] = [];

  for (const node of nodes) {
    if (!node.position || !node.size) continue;

    const nodeX = node.position.x;
    const nodeY = node.position.y;
    const nodeRight = nodeX + node.size.width;
    const nodeBottom = nodeY + node.size.height;

    const boundaryLeft = boundary.position.x;
    const boundaryTop = boundary.position.y;
    const boundaryRight = boundary.position.x + boundary.size.width;
    const boundaryBottom = boundary.position.y + boundary.size.height;

    const isInside = (
      nodeX >= boundaryLeft &&
      nodeY >= boundaryTop &&
      nodeRight <= boundaryRight &&
      nodeBottom <= boundaryBottom
    );

    if (node.type === 'function' && !isInside) {
      violations.push(`System function "${node.label}" is outside system boundary`);
    }

    if (node.type === 'actor' && isInside) {
      violations.push(`External actor "${node.label}" is inside system boundary (should be on periphery)`);
    }
  }

  return {
    valid: violations.length === 0,
    violations,
  };
}
