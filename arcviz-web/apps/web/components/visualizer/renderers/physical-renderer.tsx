'use client';

import { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { BaseRenderer, type RendererProps } from './base-renderer';
import type { PhysicalArchitecture } from '@/lib/arcadia-parser';

interface PhysicalRendererProps extends RendererProps {
  data: PhysicalArchitecture;
}

export function PhysicalRenderer({ data, width = 800, height = 600, onNodeClick }: PhysicalRendererProps) {
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

    const nodes = data.nodes.map((node, i) => ({
      id: node.id,
      name: node.name,
      type: node.type,
      x: 150 + (i % 4) * 180,
      y: 150 + Math.floor(i / 4) * 150,
      color: node.type === 'hardware' ? '#ef4444' : node.type === 'software' ? '#8b5cf6' : '#06b6d4'
    }));

    const links: Array<{ source: string; target: string; type: string }> = [];
    
    data.links.forEach(link => {
      links.push({
        source: link.from,
        target: link.to,
        type: link.type
      });
    });

    data.deployments.forEach(dep => {
      links.push({
        source: dep.component,
        target: dep.node,
        type: 'deployment'
      });
    });

    const simulation = d3.forceSimulation(nodes as any)
      .force('link', d3.forceLink(links).id((d: any) => d.id).distance(160))
      .force('charge', d3.forceManyBody().strength(-450))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(75));

    const defs = svg.append('defs');
    
    defs.append('marker')
      .attr('id', 'arrow-communication')
      .attr('viewBox', '0 -5 10 10')
      .attr('refX', 35)
      .attr('refY', 0)
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .attr('orient', 'auto')
      .append('path')
      .attr('d', 'M0,-5L10,0L0,5')
      .attr('fill', '#3b82f6');

    defs.append('marker')
      .attr('id', 'arrow-deployment')
      .attr('viewBox', '0 -5 10 10')
      .attr('refX', 35)
      .attr('refY', 0)
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .attr('orient', 'auto')
      .append('path')
      .attr('d', 'M0,-5L10,0L0,5')
      .attr('fill', '#10b981');

    const link = g.append('g')
      .selectAll('line')
      .data(links)
      .join('line')
      .attr('stroke', (d: any) => d.type === 'deployment' ? '#10b981' : '#3b82f6')
      .attr('stroke-width', 2.5)
      .attr('stroke-opacity', 0.7)
      .attr('stroke-dasharray', (d: any) => d.type === 'deployment' ? '8,4' : 'none')
      .attr('marker-end', (d: any) => `url(#arrow-${d.type})`);

    const node = g.append('g')
      .selectAll('g')
      .data(nodes)
      .join('g')
      .call(d3.drag<any, any>()
        .on('start', dragstarted)
        .on('drag', dragged)
        .on('end', dragended));

    node.filter((d: any) => d.type === 'hardware')
      .append('rect')
      .attr('width', 110)
      .attr('height', 80)
      .attr('x', -55)
      .attr('y', -40)
      .attr('rx', 8)
      .attr('fill', (d: any) => d.color)
      .attr('stroke', '#1e293b')
      .attr('stroke-width', 3)
      .attr('cursor', 'pointer')
      .on('click', (event, d) => {
        event.stopPropagation();
        onNodeClick?.(d);
      });

    node.filter((d: any) => d.type === 'software')
      .append('rect')
      .attr('width', 90)
      .attr('height', 60)
      .attr('x', -45)
      .attr('y', -30)
      .attr('rx', 6)
      .attr('fill', (d: any) => d.color)
      .attr('stroke', '#1e293b')
      .attr('stroke-width', 2.5)
      .attr('cursor', 'pointer')
      .on('click', (event, d) => {
        event.stopPropagation();
        onNodeClick?.(d);
      });

    node.filter((d: any) => d.type === 'behavior')
      .append('circle')
      .attr('r', 35)
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
      .attr('dy', '0.35em')
      .attr('fill', 'white')
      .attr('font-size', '11px')
      .attr('font-weight', '600')
      .attr('pointer-events', 'none')
      .each(function(d: any) {
        const text = d3.select(this);
        const words = d.name.split(' ');
        if (words.length > 1) {
          text.append('tspan')
            .attr('x', 0)
            .attr('dy', '-0.6em')
            .text(words[0]);
          text.append('tspan')
            .attr('x', 0)
            .attr('dy', '1.2em')
            .text(words.slice(1).join(' ').substring(0, 10));
        } else {
          text.text(d.name.length > 11 ? d.name.substring(0, 9) + '...' : d.name);
        }
      });

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
