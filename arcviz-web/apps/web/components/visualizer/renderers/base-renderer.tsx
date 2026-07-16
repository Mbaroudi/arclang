'use client';

import { useEffect, useRef } from 'react';
import { Card } from '@/components/ui/card';

export interface RendererProps {
  data: any;
  width?: number;
  height?: number;
  onNodeClick?: (node: any) => void;
  onError?: (error: Error) => void;
}

export interface RenderNode {
  id: string;
  label: string;
  type: string;
  x?: number;
  y?: number;
  data?: any;
}

export interface RenderEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
  type?: string;
}

export function BaseRenderer({ 
  data, 
  width = 800, 
  height = 600,
  onNodeClick,
  onError 
}: RendererProps) {
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!svgRef.current || !data) return;

    try {
      renderGraph(svgRef.current, data, width, height, onNodeClick);
    } catch (error) {
      console.error('Render error:', error);
      onError?.(error as Error);
    }
  }, [data, width, height, onNodeClick, onError]);

  if (!data || (!data.nodes && !data.elements)) {
    return (
      <Card className="p-8 text-center text-gray-500">
        <p>No data to visualize</p>
        <p className="text-sm mt-2">Add elements in the editor to see them here</p>
      </Card>
    );
  }

  return (
    <div className="w-full h-full overflow-auto bg-gray-50 rounded-lg border">
      <svg
        ref={svgRef}
        width={width}
        height={height}
        className="bg-white"
      >
        <defs>
          <marker
            id="arrowhead"
            markerWidth="10"
            markerHeight="10"
            refX="9"
            refY="3"
            orient="auto"
          >
            <polygon points="0 0, 10 3, 0 6" fill="#666" />
          </marker>
          
          <filter id="shadow">
            <feDropShadow dx="0" dy="2" stdDeviation="2" floodOpacity="0.2"/>
          </filter>
        </defs>
      </svg>
    </div>
  );
}

function renderGraph(
  svg: SVGSVGElement,
  data: any,
  width: number,
  height: number,
  onNodeClick?: (node: any) => void
) {
  // Clear previous content
  while (svg.firstChild) {
    svg.removeChild(svg.firstChild);
  }

  const nodes: RenderNode[] = data.nodes || [];
  const edges: RenderEdge[] = data.edges || [];

  // Simple force-directed layout
  const positions = layoutNodes(nodes, width, height);

  // Create groups
  const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
  svg.appendChild(g);

  // Render edges first (so they appear behind nodes)
  edges.forEach((edge) => {
    const source = positions.get(edge.source);
    const target = positions.get(edge.target);
    
    if (!source || !target) return;

    const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    line.setAttribute('x1', source.x.toString());
    line.setAttribute('y1', source.y.toString());
    line.setAttribute('x2', target.x.toString());
    line.setAttribute('y2', target.y.toString());
    line.setAttribute('stroke', '#999');
    line.setAttribute('stroke-width', '2');
    line.setAttribute('marker-end', 'url(#arrowhead)');
    g.appendChild(line);

    // Edge label
    if (edge.label) {
      const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
      text.setAttribute('x', ((source.x + target.x) / 2).toString());
      text.setAttribute('y', ((source.y + target.y) / 2 - 5).toString());
      text.setAttribute('text-anchor', 'middle');
      text.setAttribute('font-size', '12');
      text.setAttribute('fill', '#666');
      text.textContent = edge.label;
      g.appendChild(text);
    }
  });

  // Render nodes
  nodes.forEach((node) => {
    const pos = positions.get(node.id);
    if (!pos) return;

    // Node group
    const nodeG = document.createElementNS('http://www.w3.org/2000/svg', 'g');
    nodeG.setAttribute('transform', `translate(${pos.x},${pos.y})`);
    nodeG.setAttribute('cursor', 'pointer');
    nodeG.setAttribute('filter', 'url(#shadow)');
    
    // Node circle/rect
    const shape = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
    shape.setAttribute('x', '-60');
    shape.setAttribute('y', '-25');
    shape.setAttribute('width', '120');
    shape.setAttribute('height', '50');
    shape.setAttribute('rx', '8');
    shape.setAttribute('fill', getNodeColor(node.type));
    shape.setAttribute('stroke', '#333');
    shape.setAttribute('stroke-width', '2');
    nodeG.appendChild(shape);

    // Node label
    const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    text.setAttribute('text-anchor', 'middle');
    text.setAttribute('dy', '5');
    text.setAttribute('font-size', '14');
    text.setAttribute('font-weight', 'bold');
    text.setAttribute('fill', '#fff');
    text.textContent = truncate(node.label, 15);
    nodeG.appendChild(text);

    // Click handler
    if (onNodeClick) {
      nodeG.addEventListener('click', () => onNodeClick(node));
    }

    // Hover effect
    nodeG.addEventListener('mouseenter', () => {
      shape.setAttribute('stroke-width', '3');
      shape.setAttribute('stroke', '#000');
    });
    nodeG.addEventListener('mouseleave', () => {
      shape.setAttribute('stroke-width', '2');
      shape.setAttribute('stroke', '#333');
    });

    g.appendChild(nodeG);
  });
}

function layoutNodes(nodes: RenderNode[], width: number, height: number): Map<string, {x: number, y: number}> {
  const positions = new Map<string, {x: number, y: number}>();
  
  // Simple grid layout
  const cols = Math.ceil(Math.sqrt(nodes.length));
  const rows = Math.ceil(nodes.length / cols);
  const cellWidth = (width - 100) / cols;
  const cellHeight = (height - 100) / rows;

  nodes.forEach((node, index) => {
    const col = index % cols;
    const row = Math.floor(index / cols);
    
    positions.set(node.id, {
      x: 50 + col * cellWidth + cellWidth / 2,
      y: 50 + row * cellHeight + cellHeight / 2,
    });
  });

  return positions;
}

function getNodeColor(type: string): string {
  const colors: Record<string, string> = {
    actor: '#3b82f6',
    component: '#8b5cf6',
    function: '#10b981',
    requirement: '#f59e0b',
    node: '#ef4444',
    activity: '#06b6d4',
    capability: '#ec4899',
    default: '#6b7280',
  };
  
  return colors[type] || colors.default;
}

function truncate(str: string, maxLen: number): string {
  return str.length > maxLen ? str.substring(0, maxLen - 3) + '...' : str;
}
