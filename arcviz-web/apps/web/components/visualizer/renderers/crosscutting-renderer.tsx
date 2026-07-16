'use client';

import { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { BaseRenderer, type RendererProps } from './base-renderer';
import type { CrossCuttingConcerns } from '@/lib/arcadia-parser';

interface CrossCuttingRendererProps extends RendererProps {
  data: CrossCuttingConcerns;
}

export function CrossCuttingRenderer({ data, width = 800, height = 600, onNodeClick }: CrossCuttingRendererProps) {
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

    const centerNode = {
      id: 'center',
      name: 'Cross-Cutting',
      type: 'center',
      x: width / 2,
      y: height / 2,
      color: '#6366f1'
    };

    const concernNodes = [
      ...data.securityPolicies.map((policy, i) => ({
        id: policy.id,
        name: policy.name,
        type: 'security',
        x: width / 2 + 200 * Math.cos((i / data.securityPolicies.length) * 2 * Math.PI),
        y: height / 2 + 200 * Math.sin((i / data.securityPolicies.length) * 2 * Math.PI),
        color: '#dc2626'
      })),
      ...data.safetyConstraints.map((constraint, i) => ({
        id: constraint.id,
        name: constraint.name,
        type: 'safety',
        x: width / 2 + 200 * Math.cos((i / data.safetyConstraints.length) * 2 * Math.PI + Math.PI / 3),
        y: height / 2 + 200 * Math.sin((i / data.safetyConstraints.length) * 2 * Math.PI + Math.PI / 3),
        color: '#ea580c'
      })),
      ...data.performanceMetrics.map((metric, i) => ({
        id: metric.id,
        name: metric.name,
        type: 'performance',
        x: width / 2 + 200 * Math.cos((i / data.performanceMetrics.length) * 2 * Math.PI + 2 * Math.PI / 3),
        y: height / 2 + 200 * Math.sin((i / data.performanceMetrics.length) * 2 * Math.PI + 2 * Math.PI / 3),
        color: '#10b981'
      }))
    ];

    const allNodes = [centerNode, ...concernNodes];

    const links = concernNodes.map(node => ({
      source: 'center',
      target: node.id
    }));

    data.dependencies.forEach(dep => {
      links.push({
        source: dep.from,
        target: dep.to
      });
    });

    const simulation = d3.forceSimulation(allNodes as any)
      .force('link', d3.forceLink(links).id((d: any) => d.id).distance(180))
      .force('charge', d3.forceManyBody().strength(-400))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(70));

    const defs = svg.append('defs');
    
    defs.append('marker')
      .attr('id', 'arrow-concern')
      .attr('viewBox', '0 -5 10 10')
      .attr('refX', 32)
      .attr('refY', 0)
      .attr('markerWidth', 6)
      .attr('markerHeight', 6)
      .attr('orient', 'auto')
      .append('path')
      .attr('d', 'M0,-5L10,0L0,5')
      .attr('fill', '#64748b');

    defs.append('radialGradient')
      .attr('id', 'center-gradient')
      .selectAll('stop')
      .data([
        { offset: '0%', color: '#6366f1' },
        { offset: '100%', color: '#4f46e5' }
      ])
      .join('stop')
      .attr('offset', (d: any) => d.offset)
      .attr('stop-color', (d: any) => d.color);

    const link = g.append('g')
      .selectAll('line')
      .data(links)
      .join('line')
      .attr('stroke', '#64748b')
      .attr('stroke-width', 2)
      .attr('stroke-opacity', 0.5)
      .attr('stroke-dasharray', (d: any) => d.source === 'center' ? 'none' : '6,4')
      .attr('marker-end', 'url(#arrow-concern)');

    const node = g.append('g')
      .selectAll('g')
      .data(allNodes)
      .join('g')
      .call(d3.drag<any, any>()
        .on('start', dragstarted)
        .on('drag', dragged)
        .on('end', dragended));

    node.filter((d: any) => d.id === 'center')
      .append('circle')
      .attr('r', 55)
      .attr('fill', 'url(#center-gradient)')
      .attr('stroke', '#1e293b')
      .attr('stroke-width', 4)
      .attr('cursor', 'pointer')
      .on('click', (event, d) => {
        event.stopPropagation();
        onNodeClick?.(d);
      });

    node.filter((d: any) => d.id !== 'center')
      .append('rect')
      .attr('width', 100)
      .attr('height', 60)
      .attr('x', -50)
      .attr('y', -30)
      .attr('rx', 8)
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
      .attr('dy', (d: any) => d.id === 'center' ? '-0.5em' : '-0.7em')
      .attr('fill', 'white')
      .attr('font-size', (d: any) => d.id === 'center' ? '13px' : '9px')
      .attr('font-weight', '600')
      .attr('pointer-events', 'none')
      .text((d: any) => d.type.toUpperCase());

    node.append('text')
      .attr('text-anchor', 'middle')
      .attr('dy', (d: any) => d.id === 'center' ? '0.8em' : '0.6em')
      .attr('fill', 'white')
      .attr('font-size', (d: any) => d.id === 'center' ? '12px' : '10px')
      .attr('font-weight', 'bold')
      .attr('pointer-events', 'none')
      .text((d: any) => {
        if (d.id === 'center') return d.name;
        return d.name.length > 12 ? d.name.substring(0, 10) + '...' : d.name;
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
