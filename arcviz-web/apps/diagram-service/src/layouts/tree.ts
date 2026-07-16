/**
 * Tree Layout Algorithm
 * 
 * Implements hierarchical tree layout for function/component breakdowns.
 * Uses Reingold-Tilford algorithm for optimal node positioning.
 */

import { DiagramNode, DiagramEdge, Point, Size } from '../types/diagram';

export interface TreeLayoutOptions {
  direction?: 'DOWN' | 'RIGHT';
  levelSpacing?: number;
  nodeSpacing?: number;
  rootX?: number;
  rootY?: number;
}

export interface LayoutResult {
  nodes: LayoutNode[];
  edges: LayoutEdge[];
  totalSize: Size;
}

export interface LayoutNode {
  id: string;
  position: Point;
  size: Size;
  metadata?: any;
  children?: LayoutNode[];
}

export interface LayoutEdge {
  id: string;
  from: string;
  to: string;
  points: Point[];
  label?: string;
  metadata?: any;
}

const DEFAULT_OPTIONS: Required<TreeLayoutOptions> = {
  direction: 'DOWN',
  levelSpacing: 120,
  nodeSpacing: 80,
  rootX: 0,
  rootY: 0,
};

export async function applyTreeLayout(
  nodes: DiagramNode[],
  edges: DiagramEdge[],
  options: TreeLayoutOptions = {}
): Promise<LayoutResult> {
  const opts = { ...DEFAULT_OPTIONS, ...options };

  if (nodes.length === 0) {
    return {
      nodes: [],
      edges: [],
      totalSize: { width: 0, height: 0 },
    };
  }

  const tree = buildTree(nodes, edges);
  
  const positioned = positionTree(tree, opts);
  
  const layoutEdges = computeEdgePoints(positioned, edges, opts);
  
  const bounds = calculateBounds(positioned);

  return {
    nodes: flattenTree(positioned),
    edges: layoutEdges,
    totalSize: bounds,
  };
}

interface TreeNode {
  id: string;
  node: DiagramNode;
  children: TreeNode[];
  parent: TreeNode | null;
  x: number;
  y: number;
  mod: number;
  thread?: TreeNode;
  ancestor: TreeNode;
  change: number;
  shift: number;
  prelim: number;
}

function buildTree(nodes: DiagramNode[], edges: DiagramEdge[]): TreeNode {
  const nodeMap = new Map<string, TreeNode>();

  for (const node of nodes) {
    nodeMap.set(node.id, {
      id: node.id,
      node: node,
      children: [],
      parent: null,
      x: 0,
      y: 0,
      mod: 0,
      ancestor: null as any,
      change: 0,
      shift: 0,
      prelim: 0,
    });
  }

  for (const [_, treeNode] of nodeMap) {
    treeNode.ancestor = treeNode;
  }

  const childrenMap = new Map<string, string[]>();
  for (const edge of edges) {
    if (!childrenMap.has(edge.from)) {
      childrenMap.set(edge.from, []);
    }
    childrenMap.get(edge.from)!.push(edge.to);
  }

  for (const [parentId, childIds] of childrenMap) {
    const parent = nodeMap.get(parentId);
    if (parent) {
      for (const childId of childIds) {
        const child = nodeMap.get(childId);
        if (child) {
          parent.children.push(child);
          child.parent = parent;
        }
      }
    }
  }

  let root = nodes.length > 0 ? nodeMap.get(nodes[0].id) : null;
  
  for (const [_, node] of nodeMap) {
    if (node.parent === null) {
      root = node;
      break;
    }
  }

  return root!;
}

function positionTree(root: TreeNode, options: Required<TreeLayoutOptions>): LayoutNode {
  firstWalk(root, options);
  secondWalk(root, -root.prelim, 0, options);
  
  return convertToLayoutNode(root, options);
}

function firstWalk(node: TreeNode, options: Required<TreeLayoutOptions>): void {
  if (node.children.length === 0) {
    if (node.parent && node.parent.children[0] !== node) {
      const leftSibling = getLeftSibling(node);
      node.prelim = leftSibling ? leftSibling.prelim + options.nodeSpacing : 0;
    } else {
      node.prelim = 0;
    }
  } else {
    for (const child of node.children) {
      firstWalk(child, options);
    }

    const midpoint = (node.children[0].prelim + node.children[node.children.length - 1].prelim) / 2;

    const leftSibling = getLeftSibling(node);
    if (leftSibling) {
      node.prelim = leftSibling.prelim + options.nodeSpacing;
      node.mod = node.prelim - midpoint;
    } else {
      node.prelim = midpoint;
    }
  }
}

function secondWalk(
  node: TreeNode,
  modSum: number,
  depth: number,
  options: Required<TreeLayoutOptions>
): void {
  node.x = node.prelim + modSum;
  node.y = depth;

  for (const child of node.children) {
    secondWalk(child, modSum + node.mod, depth + 1, options);
  }
}

function getLeftSibling(node: TreeNode): TreeNode | null {
  if (!node.parent) return null;

  const siblings = node.parent.children;
  const index = siblings.indexOf(node);

  return index > 0 ? siblings[index - 1] : null;
}

function convertToLayoutNode(treeNode: TreeNode, options: Required<TreeLayoutOptions>): LayoutNode {
  const nodeWidth = 200;
  const nodeHeight = 80;

  let x: number, y: number;

  if (options.direction === 'DOWN') {
    x = options.rootX + treeNode.x * options.nodeSpacing;
    y = options.rootY + treeNode.y * options.levelSpacing;
  } else {
    x = options.rootX + treeNode.y * options.levelSpacing;
    y = options.rootY + treeNode.x * options.nodeSpacing;
  }

  const layoutNode: LayoutNode = {
    id: treeNode.id,
    position: { x, y },
    size: { width: nodeWidth, height: nodeHeight },
    metadata: treeNode.node.metadata,
    children: treeNode.children.map(child => convertToLayoutNode(child, options)),
  };

  return layoutNode;
}

function flattenTree(root: LayoutNode): LayoutNode[] {
  const result: LayoutNode[] = [];

  function traverse(node: LayoutNode): void {
    const { children, ...rest } = node;
    result.push(rest);

    if (children) {
      for (const child of children) {
        traverse(child);
      }
    }
  }

  traverse(root);
  return result;
}

function computeEdgePoints(
  root: LayoutNode,
  edges: DiagramEdge[],
  options: Required<TreeLayoutOptions>
): LayoutEdge[] {
  const nodeMap = new Map<string, LayoutNode>();

  function buildMap(node: LayoutNode): void {
    nodeMap.set(node.id, node);
    if (node.children) {
      for (const child of node.children) {
        buildMap(child);
      }
    }
  }

  buildMap(root);

  const layoutEdges: LayoutEdge[] = [];

  for (const edge of edges) {
    const fromNode = nodeMap.get(edge.from);
    const toNode = nodeMap.get(edge.to);

    if (!fromNode || !toNode) continue;

    const points: Point[] = [];

    if (options.direction === 'DOWN') {
      const fromX = fromNode.position.x + fromNode.size.width / 2;
      const fromY = fromNode.position.y + fromNode.size.height;
      const toX = toNode.position.x + toNode.size.width / 2;
      const toY = toNode.position.y;

      points.push({ x: fromX, y: fromY });
      points.push({ x: toX, y: toY });
    } else {
      const fromX = fromNode.position.x + fromNode.size.width;
      const fromY = fromNode.position.y + fromNode.size.height / 2;
      const toX = toNode.position.x;
      const toY = toNode.position.y + toNode.size.height / 2;

      points.push({ x: fromX, y: fromY });
      points.push({ x: toX, y: toY });
    }

    layoutEdges.push({
      id: edge.id,
      from: edge.from,
      to: edge.to,
      points,
      label: edge.label,
      metadata: edge.metadata,
    });
  }

  return layoutEdges;
}

function calculateBounds(root: LayoutNode): Size {
  let minX = Infinity,
    minY = Infinity,
    maxX = -Infinity,
    maxY = -Infinity;

  function traverse(node: LayoutNode): void {
    minX = Math.min(minX, node.position.x);
    minY = Math.min(minY, node.position.y);
    maxX = Math.max(maxX, node.position.x + node.size.width);
    maxY = Math.max(maxY, node.position.y + node.size.height);

    if (node.children) {
      for (const child of node.children) {
        traverse(child);
      }
    }
  }

  traverse(root);

  return {
    width: maxX - minX + 100,
    height: maxY - minY + 100,
  };
}
