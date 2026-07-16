'use client';

import { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { BaseRenderer, type RendererProps } from './base-renderer';
import type { OperationalAnalysis } from '@/lib/arcadia-parser';

interface OperationalRendererProps extends RendererProps {
  data: OperationalAnalysis;
}

export function OperationalRenderer({ data, width = 800, height = 600, onNodeClick }: OperationalRendererProps) {
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

    const nodes = [
      ...data.actors.map((actor, i) => ({
        id: actor.id,
        name: actor.name,
        type: 'actor',
        x: 100 + (i % 4) * 200,
        y: 100,
        color: '#3b82f6'
      })),
      ...data.capabilities.map((cap, i) => ({
        id: cap.id,
        name: cap.name,
        type: 'capability',
        x: 100 + (i % 4) * 200,
        y: 250,
        color: '#10b981'
      })),
      ...data.activities.map((act, i) => ({
        id: act.id,
        name: act.name,
        type: 'activity',
        x: 100 + (i % 4) * 200,
        y: 400,
        color: '#f59e0b'
      }))
    ];

    const links: Array<{ source: string; target: string }> = [];
    
    data.interactions.forEach(interaction => {
      links.push({
        source: interaction.from,
        target: interaction.to
      });
    });

    data.capabilities.forEach(cap => {
      cap.activities?.forEach(actId => {
        links.push({
          source: cap.id,
          target: actId
        });
      });
    });

    const simulation = d3.forceSimulation(nodes as any)
      .force('link', d3.forceLink(links).id((d: any) => d.id).distance(150))
      .force('charge', d3.forceManyBody().strength(-300))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(60));

    const link = g.append('g')
      .selectAll('line')
      .data(links)
      .join('line')
      .attr('stroke', '#94a3b8')
      .attr('stroke-width', 2)
      .attr('stroke-opacity', 0.6)
      .attr('marker-end', 'url(#arrowhead)');

    svg.append('defs').append('marker')
      .attr('id', 'arrowhead')
      .attr('viewBox', '0 -5 10 10')
      .attr('refX', 25)
      .attr('refY', 0)
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .attr('orient', 'auto')
      .append('path')
      .attr('d', 'M0,-5L10,0L0,5')
      .attr('fill', '#94a3b8');

    const node = g.append('g')
      .selectAll('g')
      .data(nodes)
      .join('g')
      .call(d3.drag<any, any>()
        .on('start', dragstarted)
        .on('drag', dragged)
        .on('end', dragended));

    node.append('rect')
      .attr('width', 120)
      .attr('height', 60)
      .attr('x', -60)
      .attr('y', -30)
      .attr('rx', 8)
      .attr('fill', (d: any) => d.color)
      .attr('stroke', '#1e293b')
      .attr('stroke-width', 2)
      .attr('cursor', 'pointer')
      .on('click', (event, d) => {
        event.stopPropagation();
        onNodeClick?.(d);
      });

    node.append('text')
      .attr('text-anchor', 'middle')
      .attr('dy', '0.35em')
      .attr('fill', 'white')
      .attr('font-size', '12px')
      .attr('font-weight', 'bold')
      .attr('pointer-events', 'none')
      .text((d: any) => d.name.length > 15 ? d.name.substring(0, 12) + '...' : d.name);

    node.append('text')
      .attr('text-anchor', 'middle')
      .attr('dy', '-1.2em')
      .attr('fill', 'white')
      .attr('font-size', '10px')
      .attr('pointer-events', 'none')
      .text((d: any) => d.type.toUpperCase());

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
