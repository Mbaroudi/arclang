'use client';

import { useEffect, useRef, useState } from 'react';
import dagre from 'dagre';
import { Card } from '@/components/ui/card';
import { Loader2, AlertCircle } from 'lucide-react';

interface Capella7DNode {
  id: string;
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
  element_type: string;
  is_actor: boolean;
  is_critical: boolean;
  layer: number;
}

interface Capella7DEdge {
  from: string;
  to: string;
  trace_type: string;
  is_critical: boolean;
}

interface Capella7DLayout {
  nodes: Capella7DNode[];
  edges: Capella7DEdge[];
  dimension: string;
  stats: {
    node_count: number;
    edge_count: number;
    actor_count: number;
    critical_count: number;
  };
}

interface CapellaDagreVisualizerProps {
  code: string;
  dimension?: string;
  width?: number;
  height?: number;
}

// Capella Official Color Palette (Arcadia MBSE Standard)
const CapellaColors = {
  // Actors
  ACTOR: '#FFE0B2',
  ACTOR_BORDER: '#FF6F00',
  
  // Functions
  FUNCTION: '#C8E6C9',
  FUNCTION_BORDER: '#4CAF50',
  
  // Components
  COMPONENT: '#BBDEFB',
  COMPONENT_BORDER: '#1976D2',
  
  // Operational
  OPERATIONAL_ACTIVITY: '#FFE082',
  OPERATIONAL_ACTIVITY_BORDER: '#FFA726',
  
  // System
  SYSTEM: '#FFF9C4',
  SYSTEM_BORDER: '#F57C00',
  
  // Physical
  PHYSICAL: '#FFF59D',
  PHYSICAL_BORDER: '#F9A825',
  
  // Safety Critical
  CRITICAL: '#FFCDD2',
  CRITICAL_BORDER: '#D32F2F',
  
  // Canvas
  CANVAS: '#FAFAFA',
  GRID: '#E0E0E0',
};

export function CapellaDagreVisualizer({ 
  code, 
  dimension = 'logical',
  width = 1200,
  height = 800
}: CapellaDagreVisualizerProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const [layout, setLayout] = useState<Capella7DLayout | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchLayout() {
      try {
        setLoading(true);
        setError(null);
        
        const response = await fetch('http://localhost:5001/api/arcadia-7d/layout', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ code, dimension }),
        });
        
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }
        
        const data = await response.json();
        console.log('[Capella-Dagre] 7D Layout received:', data.stats);
        setLayout(data);
      } catch (err: any) {
        console.error('[Capella-Dagre] Error:', err);
        setError(err.message);
      } finally {
        setLoading(false);
      }
    }
    
    if (code) {
      fetchLayout();
    }
  }, [code, dimension]);

  useEffect(() => {
    if (!layout || !svgRef.current) return;
    
    renderDiagram(layout);
  }, [layout]);

  const renderDiagram = (layoutData: Capella7DLayout) => {
    if (!svgRef.current) return;
    
    // Create Dagre graph for professional edge routing
    const g = new dagre.graphlib.Graph();
    g.setGraph({ 
      rankdir: 'TB',
      nodesep: 180,
      ranksep: 300,
      edgesep: 80,
      ranker: 'longest-path',
    });
    g.setDefaultEdgeLabel(() => ({}));
    
    // Add nodes with 7D Intelligence positions as hints
    layoutData.nodes.forEach(node => {
      g.setNode(node.id, {
        width: node.width,
        height: node.height,
        x: node.x, // 7D position hint
        y: node.y, // 7D position hint
        ...node,
      });
    });
    
    // Add edges
    layoutData.edges.forEach(edge => {
      g.setEdge(edge.from, edge.to, {
        ...edge,
      });
    });
    
    // Run Dagre layout (combines 7D hints with Dagre optimization)
    dagre.layout(g);
    
    // Clear previous diagram
    const svg = svgRef.current;
    svg.innerHTML = '';
    
    // Create SVG groups
    const defs = document.createElementNS('http://www.w3.org/2000/svg', 'defs');
    
    // Arrow marker for edges
    const marker = document.createElementNS('http://www.w3.org/2000/svg', 'marker');
    marker.setAttribute('id', 'arrowhead');
    marker.setAttribute('markerWidth', '10');
    marker.setAttribute('markerHeight', '10');
    marker.setAttribute('refX', '9');
    marker.setAttribute('refY', '3');
    marker.setAttribute('orient', 'auto');
    
    const polygon = document.createElementNS('http://www.w3.org/2000/svg', 'polygon');
    polygon.setAttribute('points', '0 0, 10 3, 0 6');
    polygon.setAttribute('fill', CapellaColors.COMPONENT_BORDER);
    marker.appendChild(polygon);
    defs.appendChild(marker);
    
    // Drop shadow filter
    const filter = document.createElementNS('http://www.w3.org/2000/svg', 'filter');
    filter.setAttribute('id', 'dropshadow');
    filter.setAttribute('height', '130%');
    
    const feGaussian = document.createElementNS('http://www.w3.org/2000/svg', 'feGaussianBlur');
    feGaussian.setAttribute('in', 'SourceAlpha');
    feGaussian.setAttribute('stdDeviation', '3');
    filter.appendChild(feGaussian);
    
    const feOffset = document.createElementNS('http://www.w3.org/2000/svg', 'feOffset');
    feOffset.setAttribute('dx', '2');
    feOffset.setAttribute('dy', '2');
    feOffset.setAttribute('result', 'offsetblur');
    filter.appendChild(feOffset);
    
    const feMerge = document.createElementNS('http://www.w3.org/2000/svg', 'feMerge');
    const feMergeNode1 = document.createElementNS('http://www.w3.org/2000/svg', 'feMergeNode');
    feMergeNode1.setAttribute('in', 'offsetblur');
    const feMergeNode2 = document.createElementNS('http://www.w3.org/2000/svg', 'feMergeNode');
    feMergeNode2.setAttribute('in', 'SourceGraphic');
    feMerge.appendChild(feMergeNode1);
    feMerge.appendChild(feMergeNode2);
    filter.appendChild(feMerge);
    defs.appendChild(filter);
    
    svg.appendChild(defs);
    
    // Background
    const background = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
    background.setAttribute('width', '100%');
    background.setAttribute('height', '100%');
    background.setAttribute('fill', CapellaColors.CANVAS);
    svg.appendChild(background);
    
    // Title
    const title = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    title.setAttribute('x', String(width / 2));
    title.setAttribute('y', '40');
    title.setAttribute('text-anchor', 'middle');
    title.setAttribute('font-size', '24');
    title.setAttribute('font-weight', 'bold');
    title.setAttribute('fill', '#1A237E');
    title.textContent = `${dimension.toUpperCase()} - Capella 7D + Dagre`;
    svg.appendChild(title);
    
    // Render edges first (behind nodes)
    g.edges().forEach(edgeObj => {
      const edge = g.edge(edgeObj);
      const sourceNode = g.node(edgeObj.v);
      const targetNode = g.node(edgeObj.w);
      
      if (!sourceNode || !targetNode) return;
      
      // Calculate edge path with Dagre routing
      const startX = sourceNode.x;
      const startY = sourceNode.y + sourceNode.height / 2;
      const endX = targetNode.x;
      const endY = targetNode.y - targetNode.height / 2;
      
      // Orthogonal routing (Capella standard)
      const midY = (startY + endY) / 2;
      const path = `M ${startX} ${startY} L ${startX} ${midY} L ${endX} ${midY} L ${endX} ${endY}`;
      
      const pathElem = document.createElementNS('http://www.w3.org/2000/svg', 'path');
      pathElem.setAttribute('d', path);
      pathElem.setAttribute('stroke', edge.is_critical ? CapellaColors.CRITICAL_BORDER : CapellaColors.COMPONENT_BORDER);
      pathElem.setAttribute('stroke-width', edge.is_critical ? '4' : '2');
      pathElem.setAttribute('fill', 'none');
      pathElem.setAttribute('marker-end', 'url(#arrowhead)');
      pathElem.style.strokeLinecap = 'round';
      pathElem.style.strokeLinejoin = 'round';
      
      // Add hover effect
      pathElem.style.cursor = 'pointer';
      pathElem.addEventListener('mouseenter', () => {
        pathElem.setAttribute('stroke-width', edge.is_critical ? '6' : '4');
      });
      pathElem.addEventListener('mouseleave', () => {
        pathElem.setAttribute('stroke-width', edge.is_critical ? '4' : '2');
      });
      
      svg.appendChild(pathElem);
      
      // Edge label
      if (edge.trace_type) {
        const label = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        label.setAttribute('x', String((startX + endX) / 2));
        label.setAttribute('y', String(midY - 5));
        label.setAttribute('text-anchor', 'middle');
        label.setAttribute('font-size', '11');
        label.setAttribute('fill', '#546E7A');
        label.setAttribute('font-family', 'monospace');
        label.textContent = edge.trace_type;
        svg.appendChild(label);
      }
    });
    
    // Render nodes with Capella styling
    g.nodes().forEach(nodeId => {
      const node = g.node(nodeId) as Capella7DNode;
      if (!node) return;
      
      // Determine colors based on element type and Capella specs
      let fillColor = CapellaColors.COMPONENT;
      let borderColor = CapellaColors.COMPONENT_BORDER;
      
      if (node.is_actor) {
        fillColor = CapellaColors.ACTOR;
        borderColor = CapellaColors.ACTOR_BORDER;
      } else if (node.element_type.includes('Function') || node.element_type.includes('Activity')) {
        fillColor = node.element_type.includes('Operational') 
          ? CapellaColors.OPERATIONAL_ACTIVITY 
          : CapellaColors.FUNCTION;
        borderColor = node.element_type.includes('Operational')
          ? CapellaColors.OPERATIONAL_ACTIVITY_BORDER
          : CapellaColors.FUNCTION_BORDER;
      } else if (node.element_type.includes('System')) {
        fillColor = CapellaColors.SYSTEM;
        borderColor = CapellaColors.SYSTEM_BORDER;
      } else if (node.element_type.includes('Physical')) {
        fillColor = CapellaColors.PHYSICAL;
        borderColor = CapellaColors.PHYSICAL_BORDER;
      }
      
      if (node.is_critical) {
        borderColor = CapellaColors.CRITICAL_BORDER;
      }
      
      const group = document.createElementNS('http://www.w3.org/2000/svg', 'g');
      group.setAttribute('class', 'capella-node');
      group.style.cursor = 'pointer';
      group.setAttribute('filter', 'url(#dropshadow)');
      
      // Main rectangle
      const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
      rect.setAttribute('x', String(node.x - node.width / 2));
      rect.setAttribute('y', String(node.y - node.height / 2));
      rect.setAttribute('width', String(node.width));
      rect.setAttribute('height', String(node.height));
      rect.setAttribute('rx', '10');
      rect.setAttribute('fill', fillColor);
      rect.setAttribute('stroke', borderColor);
      rect.setAttribute('stroke-width', node.is_critical ? '4' : '3');
      group.appendChild(rect);
      
      // Header bar
      const header = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
      header.setAttribute('x', String(node.x - node.width / 2));
      header.setAttribute('y', String(node.y - node.height / 2));
      header.setAttribute('width', String(node.width));
      header.setAttribute('height', '40');
      header.setAttribute('rx', '10');
      header.setAttribute('fill', borderColor);
      group.appendChild(header);
      
      // Name text
      const nameText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
      nameText.setAttribute('x', String(node.x));
      nameText.setAttribute('y', String(node.y - node.height / 2 + 25));
      nameText.setAttribute('text-anchor', 'middle');
      nameText.setAttribute('font-size', '16');
      nameText.setAttribute('font-weight', 'bold');
      nameText.setAttribute('fill', 'white');
      nameText.setAttribute('font-family', 'Segoe UI, Arial, sans-serif');
      nameText.textContent = node.name;
      group.appendChild(nameText);
      
      // ID text
      const idText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
      idText.setAttribute('x', String(node.x));
      idText.setAttribute('y', String(node.y - node.height / 2 + 60));
      idText.setAttribute('text-anchor', 'middle');
      idText.setAttribute('font-size', '11');
      idText.setAttribute('fill', '#37474F');
      idText.setAttribute('font-family', 'Consolas, monospace');
      idText.textContent = `id: ${node.id}`;
      group.appendChild(idText);
      
      // Type badge
      const typeText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
      typeText.setAttribute('x', String(node.x));
      typeText.setAttribute('y', String(node.y - node.height / 2 + 80));
      typeText.setAttribute('text-anchor', 'middle');
      typeText.setAttribute('font-size', '9');
      typeText.setAttribute('fill', '#78909C');
      typeText.setAttribute('font-family', 'monospace');
      typeText.textContent = node.element_type.replace(/([A-Z])/g, ' $1').trim();
      group.appendChild(typeText);
      
      // Critical badge
      if (node.is_critical) {
        const badge = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
        badge.setAttribute('x', String(node.x + node.width / 2 - 60));
        badge.setAttribute('y', String(node.y - node.height / 2 + 5));
        badge.setAttribute('width', '55');
        badge.setAttribute('height', '18');
        badge.setAttribute('rx', '3');
        badge.setAttribute('fill', CapellaColors.CRITICAL);
        badge.setAttribute('stroke', CapellaColors.CRITICAL_BORDER);
        badge.setAttribute('stroke-width', '1');
        group.appendChild(badge);
        
        const badgeText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        badgeText.setAttribute('x', String(node.x + node.width / 2 - 32));
        badgeText.setAttribute('y', String(node.y - node.height / 2 + 17));
        badgeText.setAttribute('font-size', '9');
        badgeText.setAttribute('font-weight', 'bold');
        badgeText.setAttribute('fill', CapellaColors.CRITICAL_BORDER);
        badgeText.textContent = 'CRITICAL';
        group.appendChild(badgeText);
      }
      
      // Actor badge
      if (node.is_actor) {
        const actorBadge = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
        actorBadge.setAttribute('cx', String(node.x - node.width / 2 + 15));
        actorBadge.setAttribute('cy', String(node.y - node.height / 2 + 15));
        actorBadge.setAttribute('r', '8');
        actorBadge.setAttribute('fill', 'white');
        actorBadge.setAttribute('stroke', borderColor);
        actorBadge.setAttribute('stroke-width', '2');
        group.appendChild(actorBadge);
      }
      
      // Hover effects
      group.addEventListener('mouseenter', () => {
        rect.setAttribute('stroke-width', node.is_critical ? '6' : '5');
        group.style.transform = 'translateY(-2px)';
      });
      group.addEventListener('mouseleave', () => {
        rect.setAttribute('stroke-width', node.is_critical ? '4' : '3');
        group.style.transform = 'translateY(0)';
      });
      
      // Click handler
      group.addEventListener('click', () => {
        console.log('[Capella-Dagre] Node clicked:', node);
      });
      
      svg.appendChild(group);
    });
    
    // Stats overlay
    const statsGroup = document.createElementNS('http://www.w3.org/2000/svg', 'g');
    const statsRect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
    statsRect.setAttribute('x', '20');
    statsRect.setAttribute('y', String(height - 100));
    statsRect.setAttribute('width', '280');
    statsRect.setAttribute('height', '80');
    statsRect.setAttribute('rx', '8');
    statsRect.setAttribute('fill', 'rgba(255, 255, 255, 0.95)');
    statsRect.setAttribute('stroke', CapellaColors.GRID);
    statsRect.setAttribute('stroke-width', '2');
    statsGroup.appendChild(statsRect);
    
    const statsTitle = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    statsTitle.setAttribute('x', '30');
    statsTitle.setAttribute('y', String(height - 75));
    statsTitle.setAttribute('font-size', '12');
    statsTitle.setAttribute('font-weight', 'bold');
    statsTitle.setAttribute('fill', '#1976D2');
    statsTitle.textContent = '📊 7D Intelligence + Dagre';
    statsGroup.appendChild(statsTitle);
    
    const statsText = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    statsText.setAttribute('x', '30');
    statsText.setAttribute('y', String(height - 55));
    statsText.setAttribute('font-size', '11');
    statsText.setAttribute('fill', '#37474F');
    statsText.textContent = `Nodes: ${layoutData.stats.node_count} | Edges: ${layoutData.stats.edge_count}`;
    statsGroup.appendChild(statsText);
    
    const statsText2 = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    statsText2.setAttribute('x', '30');
    statsText2.setAttribute('y', String(height - 35));
    statsText2.setAttribute('font-size', '11');
    statsText2.setAttribute('fill', '#37474F');
    statsText2.textContent = `Actors: ${layoutData.stats.actor_count} | Critical: ${layoutData.stats.critical_count}`;
    statsGroup.appendChild(statsText2);
    
    svg.appendChild(statsGroup);
  };

  if (loading) {
    return (
      <Card className="p-6 bg-slate-800 border-slate-700">
        <div className="flex items-center justify-center gap-3 text-slate-400">
          <Loader2 className="h-5 w-5 animate-spin" />
          <p className="text-sm">Computing 7D Intelligence layout with Dagre routing...</p>
        </div>
      </Card>
    );
  }

  if (error || !layout) {
    return (
      <Card className="p-6 bg-slate-800 border-slate-700">
        <div className="text-center text-red-400">
          <AlertCircle className="h-12 w-12 mx-auto mb-3" />
          <p className="text-lg font-semibold mb-2">Layout Generation Error</p>
          <p className="text-sm mb-4">{error || 'Unable to generate 7D layout'}</p>
          <p className="text-xs text-slate-500">
            Ensure Rust backend is running on port 5001
          </p>
        </div>
      </Card>
    );
  }

  return (
    <Card className="p-4 bg-white border-slate-300">
      <div className="mb-3 text-sm text-slate-600 font-medium">
        🎯 Hybrid Mode: 7D Intelligence (positions + constraints) + Dagre (professional edge routing)
      </div>
      <svg
        ref={svgRef}
        width={width}
        height={height}
        style={{
          border: '1px solid #E0E0E0',
          borderRadius: '8px',
          background: CapellaColors.CANVAS,
        }}
      />
    </Card>
  );
}
