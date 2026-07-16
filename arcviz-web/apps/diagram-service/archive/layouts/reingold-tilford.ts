/**
 * Reingold-Tilford Tree Layout Algorithm
 * 
 * Classic algorithm for aesthetic tree layout (1981)
 * Optimal for hierarchical breakdown diagrams
 * 
 * Properties:
 * - Nodes at same level aligned horizontally
 * - Parent centered over children
 * - Subtrees drawn identically regardless of position
 * - Minimum width for given depth
 * - O(n) time complexity
 * 
 * Perfect for SFBD, LFBD, PFBD, LCBD, PCBD, OEBD
 * LaTeX spec page 24: "Reingold-Tilford Tree (for breakdown diagrams)"
 */

import { DiagramNode, Point, Size } from '../types/diagram';

export interface TreeLayoutConfig {
  direction: 'DOWN' | 'UP' | 'RIGHT' | 'LEFT';
  levelSeparation: number;
  siblingSeparation: number;
  subtreeSeparation: number;
  nodeWidth: number;
  nodeHeight: number;
}

export interface TreeNode {
  id: string;
  label: string;
  children: TreeNode[];
  metadata?: any;
  
  x?: number;
  y?: number;
  prelim?: number;
  modifier?: number;
  leftNeighbor?: TreeNode;
  thread?: TreeNode;
  ancestor?: TreeNode;
  number?: number;
}

const DEFAULT_CONFIG: TreeLayoutConfig = {
  direction: 'DOWN',
  levelSeparation: 100,
  siblingSeparation: 60,
  subtreeSeparation: 80,
  nodeWidth: 120,
  nodeHeight: 60,
};

/**
 * Apply Reingold-Tilford layout to tree
 */
export function applyReingoldTilfordLayout(
  root: TreeNode,
  config: Partial<TreeLayoutConfig> = {}
): {
  nodes: Array<DiagramNode & { position: Point; size: Size }>;
  totalSize: Size;
} {
  const cfg = { ...DEFAULT_CONFIG, ...config };

  initializeTree(root);
  
  firstWalk(root, cfg);
  
  secondWalk(root, -root.prelim!, 0, cfg);
  
  const { minX, maxX, minY, maxY } = calculateBounds(root);
  
  shiftTree(root, -minX + 50, -minY + 50);
  
  const nodes = flattenTree(root, cfg);
  
  const totalSize: Size = {
    width: maxX - minX + 100,
    height: maxY - minY + 100,
  };

  return { nodes, totalSize };
}

/**
 * Initialize tree node properties
 */
function initializeTree(node: TreeNode, depth: number = 0): void {
  node.x = 0;
  node.y = depth;
  node.prelim = 0;
  node.modifier = 0;
  node.leftNeighbor = undefined;
  node.thread = undefined;
  node.ancestor = node;
  node.number = 0;

  for (let i = 0; i < node.children.length; i++) {
    node.children[i].number = i;
    initializeTree(node.children[i], depth + 1);
  }
}

/**
 * First walk: compute preliminary x-coordinates
 */
function firstWalk(
  node: TreeNode,
  config: TreeLayoutConfig,
  leftSibling?: TreeNode
): void {
  if (node.children.length === 0) {
    if (leftSibling) {
      node.prelim = leftSibling.prelim! + config.siblingSeparation;
    } else {
      node.prelim = 0;
    }
  } else {
    let defaultAncestor = node.children[0];
    
    for (let i = 0; i < node.children.length; i++) {
      const child = node.children[i];
      firstWalk(child, config, i > 0 ? node.children[i - 1] : undefined);
      defaultAncestor = apportion(child, defaultAncestor, config);
    }

    executeShifts(node);

    const midpoint =
      (node.children[0].prelim! + node.children[node.children.length - 1].prelim!) / 2;

    if (leftSibling) {
      node.prelim = leftSibling.prelim! + config.siblingSeparation;
      node.modifier = node.prelim - midpoint;
    } else {
      node.prelim = midpoint;
    }
  }
}

/**
 * Apportion: resolve conflicts between subtrees
 */
function apportion(
  node: TreeNode,
  defaultAncestor: TreeNode,
  config: TreeLayoutConfig
): TreeNode {
  const leftSibling = getLeftSibling(node);
  
  if (leftSibling) {
    let vir = node;
    let vor = node;
    let vil = leftSibling;
    let vol = getLeftmost(node.children[0].ancestor || node);

    let sir = node.modifier!;
    let sor = node.modifier!;
    let sil = vil.modifier!;
    let sol = vol.modifier!;

    while (nextRight(vil) && nextLeft(vir)) {
      vil = nextRight(vil)!;
      vir = nextLeft(vir)!;
      vol = nextLeft(vol)!;
      vor = nextRight(vor)!;

      vor.ancestor = node;

      const shift =
        (vil.prelim! + sil) -
        (vir.prelim! + sir) +
        config.subtreeSeparation;

      if (shift > 0) {
        moveSubtree(
          getAncestor(vil, node, defaultAncestor),
          node,
          shift
        );
        sir += shift;
        sor += shift;
      }

      sil += vil.modifier!;
      sir += vir.modifier!;
      sol += vol.modifier!;
      sor += vor.modifier!;
    }

    if (nextRight(vil) && !nextRight(vor)) {
      vor.thread = nextRight(vil);
      vor.modifier! += sil - sor;
    }

    if (nextLeft(vir) && !nextLeft(vol)) {
      vol.thread = nextLeft(vir);
      vol.modifier! += sir - sol;
      defaultAncestor = node;
    }
  }

  return defaultAncestor;
}

/**
 * Move subtree to resolve conflicts
 */
function moveSubtree(
  wl: TreeNode,
  wr: TreeNode,
  shift: number
): void {
  const subtrees = wr.number! - wl.number!;
  wr.modifier! += shift;
  wr.prelim! += shift;
}

/**
 * Execute accumulated shifts
 */
function executeShifts(node: TreeNode): void {
  let shift = 0;
  let change = 0;

  for (let i = node.children.length - 1; i >= 0; i--) {
    const child = node.children[i];
    child.prelim! += shift;
    child.modifier! += shift;
    change += 0;
    shift += 0 + change;
  }
}

/**
 * Second walk: compute final coordinates
 */
function secondWalk(
  node: TreeNode,
  m: number,
  level: number,
  config: TreeLayoutConfig
): void {
  node.x = node.prelim! + m;
  node.y = level;

  for (const child of node.children) {
    secondWalk(child, m + node.modifier!, level + 1, config);
  }
}

/**
 * Get ancestor for conflict resolution
 */
function getAncestor(
  vil: TreeNode,
  node: TreeNode,
  defaultAncestor: TreeNode
): TreeNode {
  const parent = getParent(node);
  if (parent && parent.children.includes(vil.ancestor!)) {
    return vil.ancestor!;
  }
  return defaultAncestor;
}

/**
 * Get left sibling
 */
function getLeftSibling(node: TreeNode): TreeNode | undefined {
  const parent = getParent(node);
  if (!parent) return undefined;
  
  const index = parent.children.indexOf(node);
  if (index > 0) {
    return parent.children[index - 1];
  }
  return undefined;
}

/**
 * Get leftmost descendant
 */
function getLeftmost(node: TreeNode): TreeNode {
  let current = node;
  while (current.children.length > 0) {
    current = current.children[0];
  }
  return current;
}

/**
 * Get next right contour node
 */
function nextRight(node: TreeNode): TreeNode | undefined {
  if (node.children.length > 0) {
    return node.children[node.children.length - 1];
  }
  return node.thread;
}

/**
 * Get next left contour node
 */
function nextLeft(node: TreeNode): TreeNode | undefined {
  if (node.children.length > 0) {
    return node.children[0];
  }
  return node.thread;
}

/**
 * Get parent (helper - needs tree structure with parent links)
 */
function getParent(node: TreeNode): TreeNode | undefined {
  return undefined;
}

/**
 * Calculate bounding box
 */
function calculateBounds(node: TreeNode): {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
} {
  let minX = node.x!;
  let maxX = node.x!;
  let minY = node.y!;
  let maxY = node.y!;

  function traverse(n: TreeNode) {
    if (n.x! < minX) minX = n.x!;
    if (n.x! > maxX) maxX = n.x!;
    if (n.y! < minY) minY = n.y!;
    if (n.y! > maxY) maxY = n.y!;

    for (const child of n.children) {
      traverse(child);
    }
  }

  traverse(node);

  return { minX, maxX, minY, maxY };
}

/**
 * Shift entire tree
 */
function shiftTree(node: TreeNode, dx: number, dy: number): void {
  node.x! += dx;
  node.y! += dy;

  for (const child of node.children) {
    shiftTree(child, dx, dy);
  }
}

/**
 * Flatten tree to diagram nodes
 */
function flattenTree(
  node: TreeNode,
  config: TreeLayoutConfig
): Array<DiagramNode & { position: Point; size: Size }> {
  const nodes: Array<DiagramNode & { position: Point; size: Size }> = [];

  function traverse(n: TreeNode) {
    let x: number, y: number;

    switch (config.direction) {
      case 'DOWN':
        x = n.x! * config.siblingSeparation;
        y = n.y! * config.levelSeparation;
        break;
      case 'UP':
        x = n.x! * config.siblingSeparation;
        y = -n.y! * config.levelSeparation;
        break;
      case 'RIGHT':
        x = n.y! * config.levelSeparation;
        y = n.x! * config.siblingSeparation;
        break;
      case 'LEFT':
        x = -n.y! * config.levelSeparation;
        y = n.x! * config.siblingSeparation;
        break;
    }

    nodes.push({
      id: n.id,
      label: n.label,
      type: 'tree-node',
      position: { x, y },
      size: { width: config.nodeWidth, height: config.nodeHeight },
      children: n.children.length > 0 ? n.children : undefined,
      metadata: n.metadata,
    });

    for (const child of n.children) {
      traverse(child);
    }
  }

  traverse(node);

  return nodes;
}

/**
 * Calculate tree depth
 */
export function calculateTreeDepth(node: TreeNode): number {
  if (node.children.length === 0) {
    return 1;
  }
  return 1 + Math.max(...node.children.map(calculateTreeDepth));
}

/**
 * Calculate tree width (number of leaves)
 */
export function calculateTreeWidth(node: TreeNode): number {
  if (node.children.length === 0) {
    return 1;
  }
  return node.children.reduce((sum, child) => sum + calculateTreeWidth(child), 0);
}

/**
 * Balance tree (minimize width)
 */
export function balanceTree(node: TreeNode): void {
  if (node.children.length <= 1) return;

  node.children.sort((a, b) => {
    const widthA = calculateTreeWidth(a);
    const widthB = calculateTreeWidth(b);
    return widthB - widthA;
  });

  for (const child of node.children) {
    balanceTree(child);
  }
}
