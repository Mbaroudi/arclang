'use client';

import { useEffect, useRef } from 'react';
import * as d3 from 'd3';
import { BaseRenderer, type RendererProps } from './base-renderer';
import type { EPBSStructure } from '@/lib/arcadia-parser';

interface EPBSRendererProps extends RendererProps {
  data: EPBSStructure;
}

export function EPBSRenderer({ data, width = 800, height = 600, onNodeClick }: EPBSRendererProps) {
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

    const root = d3.hierarchy(buildTree(data));
    const treeLayout = d3.tree<any>().size([width - 100, height - 100]);
    const treeData = treeLayout(root);

    const links = treeData.links();
    const nodes = treeData.descendants();

    const link = g.append('g')
      .selectAll('path')
      .data(links)
      .join('path')
      .attr('d', d3.linkVertical<any, any>()
        .x((d: any) => d.x + 50)
        .y((d: any) => d.y + 50))
      .attr('fill', 'none')
      .attr('stroke', '#64748b')
      .attr('stroke-width', 2.5)
      .attr('stroke-opacity', 0.6);

    const node = g.append('g')
      .selectAll('g')
      .data(nodes)
      .join('g')
      .attr('transform', (d: any) => `translate(${d.x + 50},${d.y + 50})`);

    node.append('rect')
      .attr('width', 120)
      .attr('height', 70)
      .attr('x', -60)
      .attr('y', -35)
      .attr('rx', 8)
      .attr('fill', (d: any) => {
        if (d.depth === 0) return '#dc2626';
        if (d.data.type === 'subsystem') return '#ea580c';
        if (d.data.type === 'assembly') return '#f59e0b';
        return '#84cc16';
      })
      .attr('stroke', '#1e293b')
      .attr('stroke-width', 2.5)
      .attr('cursor', 'pointer')
      .on('click', (event, d) => {
        event.stopPropagation();
        onNodeClick?.(d.data);
      });

    node.append('text')
      .attr('text-anchor', 'middle')
      .attr('dy', '-0.5em')
      .attr('fill', 'white')
      .attr('font-size', '10px')
      .attr('font-weight', '600')
      .attr('pointer-events', 'none')
      .text((d: any) => d.data.type?.toUpperCase() || 'ROOT');

    node.append('text')
      .attr('text-anchor', 'middle')
      .attr('dy', '1em')
      .attr('fill', 'white')
      .attr('font-size', '11px')
      .attr('font-weight', 'bold')
      .attr('pointer-events', 'none')
      .text((d: any) => {
        const name = d.data.name || d.data.id;
        return name.length > 14 ? name.substring(0, 12) + '...' : name;
      });

    return () => {};
  }, [data, width, height, onNodeClick]);

  return (
    <BaseRenderer width={width} height={height}>
      <svg ref={svgRef} width={width} height={height} className="bg-slate-900" />
    </BaseRenderer>
  );
}

function buildTree(data: EPBSStructure): any {
  const root: any = {
    id: 'root',
    name: 'System',
    type: 'system',
    children: []
  };

  data.subsystems.forEach(subsystem => {
    const subsystemNode: any = {
      id: subsystem.id,
      name: subsystem.name,
      type: 'subsystem',
      children: []
    };

    const assemblies = data.assemblies.filter(a => a.parent === subsystem.id);
    assemblies.forEach(assembly => {
      const assemblyNode: any = {
        id: assembly.id,
        name: assembly.name,
        type: 'assembly',
        children: []
      };

      const components = data.components.filter(c => c.parent === assembly.id);
      components.forEach(component => {
        assemblyNode.children.push({
          id: component.id,
          name: component.name,
          type: 'component'
        });
      });

      subsystemNode.children.push(assemblyNode);
    });

    root.children.push(subsystemNode);
  });

  return root;
}
