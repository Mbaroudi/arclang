/**
 * Nested Box Packing Algorithm
 * 
 * Optimal layout for hierarchical containment in Capella LAB/PAB diagrams
 * Ensures child components are properly nested within parent components
 * 
 * LaTeX spec page 24: "Nested box packing algorithm (for LAB/PAB)"
 * 
 * Properties:
 * - Parent boxes contain all child boxes
 * - Minimum padding between parent boundary and children (20px)
 * - Efficient space utilization
 * - Recursive packing for multi-level hierarchies
 * - Automatic parent size calculation
 * 
 * Perfect for:
 * - LAB (Logical Architecture Blank): Functions in Components
 * - PAB (Physical Architecture Blank): Behavioral in Nodes
 */

import { DiagramNode, Point, Size } from '../types/diagram';

export interface PackingConfig {
  minPadding: number;
  childSpacing: number;
  arrangeMode: 'grid' | 'flow' | 'compact';
  aspectRatio: number;
}

export interface PackedNode extends DiagramNode {
  position: Point;
  size: Size;
  children?: PackedNode[];
  level: number;
}

const DEFAULT_CONFIG: PackingConfig = {
  minPadding: 20,
  childSpacing: 15,
  arrangeMode: 'grid',
  aspectRatio: 1.5,
};

/**
 * Apply nested box packing layout
 */
export function applyNestedBoxPacking(
  nodes: DiagramNode[],
  config: Partial<PackingConfig> = {}
): {
  nodes: PackedNode[];
  totalSize: Size;
} {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  const hierarchy = buildHierarchy(nodes);
  
  const packedNodes = packHierarchy(hierarchy, cfg);
  
  const totalSize = calculateTotalSize(packedNodes, cfg);

  return { nodes: packedNodes, totalSize };
}

/**
 * Build hierarchy tree from flat node list
 */
function buildHierarchy(nodes: DiagramNode[]): DiagramNode[] {
  const nodeMap = new Map<string, DiagramNode>();
  const roots: DiagramNode[] = [];

  for (const node of nodes) {
    nodeMap.set(node.id, { ...node, children: [] });
  }

  for (const node of nodes) {
    if (node.children && node.children.length > 0) {
      const parent = nodeMap.get(node.id)!;
      parent.children = node.children
        .map(child => {
          if (typeof child === 'string') {
            return nodeMap.get(child);
          }
          return nodeMap.get((child as any).id);
        })
        .filter(Boolean) as DiagramNode[];
    }
  }

  for (const node of nodeMap.values()) {
    const isChild = Array.from(nodeMap.values()).some(
      n => n.children && n.children.some(c => (c as any).id === node.id)
    );
    if (!isChild) {
      roots.push(node);
    }
  }

  return roots;
}

/**
 * Pack hierarchy recursively
 */
function packHierarchy(
  nodes: DiagramNode[],
  config: PackingConfig,
  level: number = 0,
  offsetX: number = 0,
  offsetY: number = 0
): PackedNode[] {
  const packedNodes: PackedNode[] = [];

  for (const node of nodes) {
    const packed = packNode(node, config, level, offsetX, offsetY);
    packedNodes.push(packed);
  }

  return packedNodes;
}

/**
 * Pack individual node with its children
 */
function packNode(
  node: DiagramNode,
  config: PackingConfig,
  level: number,
  offsetX: number,
  offsetY: number
): PackedNode {
  let packedChildren: PackedNode[] = [];
  let contentSize: Size = { width: 0, height: 0 };

  if (node.children && node.children.length > 0) {
    const childrenArray = node.children as DiagramNode[];
    
    const childLayout = arrangeChildren(childrenArray, config);
    
    contentSize = childLayout.size;
    
    packedChildren = childrenArray.map((child, index) => {
      const childPos = childLayout.positions[index];
      return packNode(
        child,
        config,
        level + 1,
        childPos.x,
        childPos.y
      );
    });
  }

  const nodeSize = calculateNodeSize(node, contentSize, config, level);
  
  const position: Point = {
    x: offsetX,
    y: offsetY,
  };

  return {
    ...node,
    position,
    size: nodeSize,
    children: packedChildren.length > 0 ? packedChildren : undefined,
    level,
  };
}

/**
 * Arrange children within parent
 */
function arrangeChildren(
  children: DiagramNode[],
  config: PackingConfig
): {
  positions: Point[];
  size: Size;
} {
  const positions: Point[] = [];
  
  switch (config.arrangeMode) {
    case 'grid':
      return arrangeChildrenGrid(children, config);
    case 'flow':
      return arrangeChildrenFlow(children, config);
    case 'compact':
      return arrangeChildrenCompact(children, config);
    default:
      return arrangeChildrenGrid(children, config);
  }
}

/**
 * Arrange children in grid layout
 */
function arrangeChildrenGrid(
  children: DiagramNode[],
  config: PackingConfig
): {
  positions: Point[];
  size: Size;
} {
  const positions: Point[] = [];
  
  const cols = Math.ceil(Math.sqrt(children.length * config.aspectRatio));
  const rows = Math.ceil(children.length / cols);
  
  const childWidth = 120;
  const childHeight = 80;
  
  for (let i = 0; i < children.length; i++) {
    const row = Math.floor(i / cols);
    const col = i % cols;
    
    positions.push({
      x: config.minPadding + col * (childWidth + config.childSpacing),
      y: config.minPadding + 30 + row * (childHeight + config.childSpacing),
    });
  }
  
  const width = cols * (childWidth + config.childSpacing) - config.childSpacing;
  const height = rows * (childHeight + config.childSpacing) - config.childSpacing + 30;
  
  return {
    positions,
    size: { width, height },
  };
}

/**
 * Arrange children in flow layout (left-to-right, top-to-bottom)
 */
function arrangeChildrenFlow(
  children: DiagramNode[],
  config: PackingConfig
): {
  positions: Point[];
  size: Size;
} {
  const positions: Point[] = [];
  const maxWidth = 600;
  
  let currentX = config.minPadding;
  let currentY = config.minPadding + 30;
  let rowHeight = 80;
  let maxX = 0;
  
  for (const child of children) {
    const childWidth = child.size?.width || 120;
    const childHeight = child.size?.height || 80;
    
    if (currentX + childWidth > maxWidth && positions.length > 0) {
      currentX = config.minPadding;
      currentY += rowHeight + config.childSpacing;
      rowHeight = childHeight;
    }
    
    positions.push({ x: currentX, y: currentY });
    
    currentX += childWidth + config.childSpacing;
    maxX = Math.max(maxX, currentX);
    rowHeight = Math.max(rowHeight, childHeight);
  }
  
  return {
    positions,
    size: {
      width: maxX - config.childSpacing,
      height: currentY + rowHeight - config.minPadding,
    },
  };
}

/**
 * Arrange children in compact layout (bin packing)
 */
function arrangeChildrenCompact(
  children: DiagramNode[],
  config: PackingConfig
): {
  positions: Point[];
  size: Size;
} {
  const positions: Point[] = [];
  const bins: Array<{ x: number; y: number; width: number; height: number }> = [];
  
  bins.push({
    x: config.minPadding,
    y: config.minPadding + 30,
    width: 800,
    height: 600,
  });
  
  const sortedChildren = [...children].sort((a, b) => {
    const areaA = (a.size?.width || 120) * (a.size?.height || 80);
    const areaB = (b.size?.width || 120) * (b.size?.height || 80);
    return areaB - areaA;
  });
  
  for (const child of sortedChildren) {
    const childWidth = child.size?.width || 120;
    const childHeight = child.size?.height || 80;
    
    let placed = false;
    for (const bin of bins) {
      if (bin.width >= childWidth && bin.height >= childHeight) {
        positions.push({ x: bin.x, y: bin.y });
        
        bins.push({
          x: bin.x + childWidth + config.childSpacing,
          y: bin.y,
          width: bin.width - childWidth - config.childSpacing,
          height: childHeight,
        });
        
        bins.push({
          x: bin.x,
          y: bin.y + childHeight + config.childSpacing,
          width: bin.width,
          height: bin.height - childHeight - config.childSpacing,
        });
        
        bins.splice(bins.indexOf(bin), 1);
        placed = true;
        break;
      }
    }
    
    if (!placed) {
      positions.push({
        x: config.minPadding,
        y: config.minPadding + 30 + positions.length * 100,
      });
    }
  }
  
  let maxX = 0;
  let maxY = 0;
  
  for (let i = 0; i < positions.length; i++) {
    const child = children[i];
    const pos = positions[i];
    const childWidth = child.size?.width || 120;
    const childHeight = child.size?.height || 80;
    
    maxX = Math.max(maxX, pos.x + childWidth);
    maxY = Math.max(maxY, pos.y + childHeight);
  }
  
  return {
    positions,
    size: {
      width: maxX - config.minPadding,
      height: maxY - config.minPadding,
    },
  };
}

/**
 * Calculate node size based on children
 */
function calculateNodeSize(
  node: DiagramNode,
  contentSize: Size,
  config: PackingConfig,
  level: number
): Size {
  if (node.children && node.children.length > 0) {
    const minWidth = Math.max(
      contentSize.width + 2 * config.minPadding,
      200
    );
    
    const minHeight = Math.max(
      contentSize.height + 2 * config.minPadding + 30,
      150
    );
    
    return {
      width: minWidth,
      height: minHeight,
    };
  } else {
    return node.size || { width: 120, height: 80 };
  }
}

/**
 * Calculate total diagram size
 */
function calculateTotalSize(nodes: PackedNode[], config: PackingConfig): Size {
  let maxX = 0;
  let maxY = 0;

  function traverse(node: PackedNode) {
    maxX = Math.max(maxX, node.position.x + node.size.width);
    maxY = Math.max(maxY, node.position.y + node.size.height);

    if (node.children) {
      for (const child of node.children) {
        traverse(child);
      }
    }
  }

  for (const node of nodes) {
    traverse(node);
  }

  return {
    width: maxX + config.minPadding,
    height: maxY + config.minPadding,
  };
}

/**
 * Validate containment (children inside parents)
 */
export function validateContainment(nodes: PackedNode[]): {
  valid: boolean;
  violations: string[];
} {
  const violations: string[] = [];

  function checkNode(parent: PackedNode) {
    if (!parent.children) return;

    for (const child of parent.children) {
      const childRight = child.position.x + child.size.width;
      const childBottom = child.position.y + child.size.height;
      const parentRight = parent.position.x + parent.size.width;
      const parentBottom = parent.position.y + parent.size.height;

      if (
        child.position.x < parent.position.x ||
        child.position.y < parent.position.y ||
        childRight > parentRight ||
        childBottom > parentBottom
      ) {
        violations.push(
          `Child "${child.label}" not fully contained in parent "${parent.label}"`
        );
      }

      checkNode(child);
    }
  }

  for (const node of nodes) {
    checkNode(node);
  }

  return {
    valid: violations.length === 0,
    violations,
  };
}

/**
 * Optimize packing (minimize whitespace)
 */
export function optimizePacking(
  nodes: PackedNode[],
  config: PackingConfig
): PackedNode[] {
  function optimizeNode(node: PackedNode): PackedNode {
    if (!node.children || node.children.length === 0) {
      return node;
    }

    const optimizedChildren = node.children.map(optimizeNode);

    const childLayout = arrangeChildrenCompact(
      optimizedChildren,
      config
    );

    const newSize = calculateNodeSize(node, childLayout.size, config, node.level);

    return {
      ...node,
      size: newSize,
      children: optimizedChildren.map((child, i) => ({
        ...child,
        position: {
          x: node.position.x + childLayout.positions[i].x,
          y: node.position.y + childLayout.positions[i].y,
        },
      })),
    };
  }

  return nodes.map(optimizeNode);
}
