'use client';

import { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { BaseRenderer, type RendererProps } from './base-renderer';
import type { SystemAnalysis } from '@/lib/arcadia-parser';

interface SystemRendererProps extends RendererProps {
  data: SystemAnalysis;
}

export function SystemRenderer({ data, width = 800, height = 600, onNodeClick }: SystemRendererProps) {
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
      { id: data.system.id, name: data.system.name, type: 'system', x: width / 2, y: 100, color: '#8b5cf6' },
      ...data.actors.map((actor, i) => ({
        id: actor.id,
        name: actor.name,
        type: 'actor',
        x: 150 + (i % 3) * 250,
        y: 250,
        color: '#3b82f6'
      })),
      ...data.functions.map((func, i) => ({
        id: func.id,
        name: func.name,
        type: 'function',
        x: 150 + (i % 4) * 200,
        y: 400,
        color: '#10b981'
      }))
    ];

    const links: Array<{ source: string; target: string }> = [];
    
    data.actors.forEach(actor => {
      links.push({ source: actor.id, target: data.system.id });
    });

    data.functions.forEach(func => {
      func.allocatedTo?.forEach(targetId => {
        links.push({ source: func.id, target: targetId });
      });
    });

    data.interactions.forEach(interaction => {
      links.push({
        source: interaction.from,
        target: interaction.to
      });
    });

    const simulation = d3.forceSimulation(nodes as any)
      .force('link', d3.forceLink(links).id((d: any) => d.id).distance(120))
      .force('charge', d3.forceManyBody().strength(-400))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(70));

    const link = g.append('g')
      .selectAll('line')
      .data(links)
      .join('line')
      .attr('stroke', '#64748b')
      .attr('stroke-width', 2)
      .attr('stroke-opacity', 0.6);

    const node = g.append('g')
      .selectAll('g')
      .data(nodes)
      .join('g')
      .call(d3.drag<any, any>()
        .on('start', dragstarted)
        .on('drag', dragged)
        .on('end', dragended));

    node.append('circle')
      .attr('r', (d: any) => d.type === 'system' ? 50 : 40)
      .attr('fill', (d: any) => d.color)
      .attr('stroke', '#1e293b')
      .attr('stroke-width', 3)
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
      .attr('font-weight', 'bold')
      .attr('pointer-events', 'none')
      .each(function(d: any) {
        const text = d3.select(this);
        const words = d.name.split(' ');
        if (words.length > 2) {
          text.append('tspan')
            .attr('x', 0)
            .attr('dy', '-0.6em')
            .text(words.slice(0, 2).join(' '));
          text.append('tspan')
            .attr('x', 0)
            .attr('dy', '1.2em')
            .text(words.slice(2).join(' '));
        } else {
          text.text(d.name.length > 12 ? d.name.substring(0, 10) + '...' : d.name);
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
