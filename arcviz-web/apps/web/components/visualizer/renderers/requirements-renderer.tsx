'use client';

import { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { BaseRenderer, type RendererProps } from './base-renderer';
import type { RequirementsModel } from '@/lib/arcadia-parser';

interface RequirementsRendererProps extends RendererProps {
  data: RequirementsModel;
}

export function RequirementsRenderer({ data, width = 800, height = 600, onNodeClick }: RequirementsRendererProps) {
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!svgRef.current || !data) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll('*').remove();

    const g = svg.append('g');

    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        g.attr('transform', event.transform);
      });

    svg.call(zoom);

    const nodes = data.requirements.map((req, i) => ({
      id: req.id,
      name: req.name,
      type: req.type,
      priority: req.priority,
      status: req.status,
      x: 100 + (i % 5) * 150,
      y: 100 + Math.floor(i / 5) * 120,
      color: getRequirementColor(req.type, req.priority)
    }));

    const links: Array<{ source: string; target: string; type: string }> = [];
    
    data.requirements.forEach(req => {
      req.allocatedTo?.forEach(targetId => {
        links.push({ source: req.id, target: targetId, type: 'allocation' });
      });
      req.refinedBy?.forEach(refId => {
        links.push({ source: req.id, target: refId, type: 'refinement' });
      });
    });

    data.traces.forEach(trace => {
      links.push({
        source: trace.from,
        target: trace.to,
        type: 'trace'
      });
    });

    const simulation = d3.forceSimulation(nodes as any)
      .force('link', d3.forceLink(links).id((d: any) => d.id).distance(130))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(60));

    const defs = svg.append('defs');
    
    ['allocation', 'refinement', 'trace'].forEach(type => {
      defs.append('marker')
        .attr('id', `arrow-${type}`)
        .attr('viewBox', '0 -5 10 10')
        .attr('refX', 28)
        .attr('refY', 0)
        .attr('markerWidth', 6)
        .attr('markerHeight', 6)
        .attr('orient', 'auto')
        .append('path')
        .attr('d', 'M0,-5L10,0L0,5')
        .attr('fill', type === 'allocation' ? '#10b981' : type === 'refinement' ? '#8b5cf6' : '#3b82f6');
    });

    const link = g.append('g')
      .selectAll('line')
      .data(links)
      .join('line')
      .attr('stroke', (d: any) => {
        if (d.type === 'allocation') return '#10b981';
        if (d.type === 'refinement') return '#8b5cf6';
        return '#3b82f6';
      })
      .attr('stroke-width', 2)
      .attr('stroke-opacity', 0.6)
      .attr('stroke-dasharray', (d: any) => {
        if (d.type === 'trace') return '5,5';
        if (d.type === 'refinement') return '8,3';
        return 'none';
      })
      .attr('marker-end', (d: any) => `url(#arrow-${d.type})`);

    const node = g.append('g')
      .selectAll('g')
      .data(nodes)
      .join('g')
      .call(d3.drag<any, any>()
        .on('start', dragstarted)
        .on('drag', dragged)
        .on('end', dragended));

    node.append('rect')
      .attr('width', 110)
      .attr('height', 65)
      .attr('x', -55)
      .attr('y', -32.5)
      .attr('rx', 7)
      .attr('fill', (d: any) => d.color)
      .attr('stroke', '#1e293b')
      .attr('stroke-width', 2.5)
      .attr('cursor', 'pointer')
      .on('click', (event, d) => {
        event.stopPropagation();
        onNodeClick?.(d);
      });

    node.append('text')
      .attr('text-anchor', 'middle')
      .attr('dy', '-0.8em')
      .attr('fill', 'white')
      .attr('font-size', '9px')
      .attr('font-weight', '600')
      .attr('pointer-events', 'none')
      .text((d: any) => `${d.type.toUpperCase()} - ${d.priority}`);

    node.append('text')
      .attr('text-anchor', 'middle')
      .attr('dy', '0.5em')
      .attr('fill', 'white')
      .attr('font-size', '10px')
      .attr('font-weight', 'bold')
      .attr('pointer-events', 'none')
      .text((d: any) => d.name.length > 13 ? d.name.substring(0, 11) + '...' : d.name);

    node.append('text')
      .attr('text-anchor', 'middle')
      .attr('dy', '1.6em')
      .attr('fill', 'white')
      .attr('font-size', '8px')
      .attr('pointer-events', 'none')
      .text((d: any) => d.status);

    simulation.on('tick', () => {
      link
        .attr('x1', (d: any) => d.source.x)
        .attr('y1', (d: any) => d.source.y)
        .attr('x2', (d: any) => d.target.x)
        .attr('y2', (d: any) => d.target.y);

      node.attr('transform', (d: any) => `translate(${d.x},${d.y})`);
    });

    function dragstarted(event: any) {
      if (!event.active) simulation.alphaTarget(0.3).restart();
      event.subject.fx = event.subject.x;
      event.subject.fy = event.subject.y;
    }

    function dragged(event: any) {
      event.subject.fx = event.x;
      event.subject.fy = event.y;
    }

    function dragended(event: any) {
      if (!event.active) simulation.alphaTarget(0);
      event.subject.fx = null;
      event.subject.fy = null;
    }

    return () => {
      simulation.stop();
    };
  }, [data, width, height, onNodeClick]);

  return (
    <BaseRenderer width={width} height={height}>
      <svg ref={svgRef} width={width} height={height} className="bg-slate-900" />
    </BaseRenderer>
  );
}

function getRequirementColor(type: string, priority: string): string {
  if (priority === 'critical' || priority === 'high') {
    return type === 'functional' ? '#dc2626' : '#ea580c';
  }
  if (priority === 'medium') {
    return type === 'functional' ? '#f59e0b' : '#eab308';
  }
  return type === 'functional' ? '#10b981' : '#06b6d4';
}
