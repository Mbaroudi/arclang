'use client';

import { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { BaseRenderer, type RendererProps } from './base-renderer';
import type { LogicalArchitecture } from '@/lib/arcadia-parser';

interface LogicalRendererProps extends RendererProps {
  data: LogicalArchitecture;
}

export function LogicalRenderer({ data, width = 800, height = 600, onNodeClick }: LogicalRendererProps) {
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
      ...data.components.map((comp, i) => ({
        id: comp.id,
        name: comp.name,
        type: 'component',
        x: 100 + (i % 4) * 180,
        y: 150 + Math.floor(i / 4) * 150,
        color: '#ec4899'
      })),
      ...data.interfaces.map((iface, i) => ({
        id: iface.id,
        name: iface.name,
        type: 'interface',
        x: 150 + (i % 3) * 220,
        y: 450,
        color: '#06b6d4'
      }))
    ];

    const links: Array<{ source: string; target: string; type: string }> = [];
    
    data.components.forEach(comp => {
      comp.provides?.forEach(ifaceId => {
        links.push({ source: comp.id, target: ifaceId, type: 'provides' });
      });
      comp.requires?.forEach(ifaceId => {
        links.push({ source: ifaceId, target: comp.id, type: 'requires' });
      });
    });

    data.dataFlows.forEach(flow => {
      links.push({ source: flow.from, target: flow.to, type: 'dataflow' });
    });

    const simulation = d3.forceSimulation(nodes as any)
      .force('link', d3.forceLink(links).id((d: any) => d.id).distance(140))
      .force('charge', d3.forceManyBody().strength(-350))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(65));

    const defs = svg.append('defs');
    
    defs.append('marker')
      .attr('id', 'arrow-provides')
      .attr('viewBox', '0 -5 10 10')
      .attr('refX', 30)
      .attr('refY', 0)
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .attr('orient', 'auto')
      .append('path')
      .attr('d', 'M0,-5L10,0L0,5')
      .attr('fill', '#10b981');

    defs.append('marker')
      .attr('id', 'arrow-requires')
      .attr('viewBox', '0 -5 10 10')
      .attr('refX', 30)
      .attr('refY', 0)
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .attr('orient', 'auto')
      .append('path')
      .attr('d', 'M0,-5L10,0L0,5')
      .attr('fill', '#f59e0b');

    defs.append('marker')
      .attr('id', 'arrow-dataflow')
      .attr('viewBox', '0 -5 10 10')
      .attr('refX', 30)
      .attr('refY', 0)
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .attr('orient', 'auto')
      .append('path')
      .attr('d', 'M0,-5L10,0L0,5')
      .attr('fill', '#8b5cf6');

    const link = g.append('g')
      .selectAll('line')
      .data(links)
      .join('line')
      .attr('stroke', (d: any) => {
        if (d.type === 'provides') return '#10b981';
        if (d.type === 'requires') return '#f59e0b';
        return '#8b5cf6';
      })
      .attr('stroke-width', 2)
      .attr('stroke-opacity', 0.7)
      .attr('stroke-dasharray', (d: any) => d.type === 'dataflow' ? '5,5' : 'none')
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
      .attr('width', (d: any) => d.type === 'component' ? 100 : 80)
      .attr('height', (d: any) => d.type === 'component' ? 70 : 50)
      .attr('x', (d: any) => d.type === 'component' ? -50 : -40)
      .attr('y', (d: any) => d.type === 'component' ? -35 : -25)
      .attr('rx', 6)
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
      .text((d: any) => d.name.length > 12 ? d.name.substring(0, 10) + '...' : d.name);

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
