#!/usr/bin/env python3
import xml.etree.ElementTree as ET
import sys

xml_file = sys.argv[1]
svg_file = sys.argv[2]

tree = ET.parse(xml_file)
root = tree.getroot()

ns = {'capella': 'http://www.polarsys.org/capella/core/1.4.0'}

requirements = root.findall('.//requirement')
components = root.findall('.//component')
traces = root.findall('.//trace')

width = 1400
height = 1000

svg = f'''<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <defs>
    <marker id="arrowhead" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto">
      <polygon points="0 0, 10 3, 0 6" fill="#666" />
    </marker>
  </defs>
  <style>
    .requirement {{ fill: #E3F2FD; stroke: #1976D2; stroke-width: 2; }}
    .component {{ fill: #FFF3E0; stroke: #F57C00; stroke-width: 2; }}
    .text {{ font-family: Arial, sans-serif; font-size: 12px; text-anchor: middle; }}
    .title {{ font-weight: bold; font-size: 14px; }}
    .trace {{ stroke: #666; stroke-width: 1.5; fill: none; marker-end: url(#arrowhead); }}
    .satisfies {{ stroke: #4CAF50; stroke-dasharray: 5,5; }}
    .implements {{ stroke: #2196F3; }}
  </style>
  
  <rect x="0" y="0" width="{width}" height="{height}" fill="#FAFAFA"/>
  
  <text x="{width//2}" y="30" class="text title" font-size="20">Adaptive Cruise Control - Logical Architecture</text>
'''

req_y = 80
req_x_start = 50
req_width = 280
req_height = 60
req_spacing = 20

positions = {}

svg += '  <!-- Requirements -->\n'
for i, req in enumerate(requirements):
    x = req_x_start + (i % 2) * (req_width + req_spacing)
    y = req_y + (i // 2) * (req_height + req_spacing)
    positions[req.get('id')] = (x + req_width//2, y + req_height//2)
    
    svg += f'  <rect x="{x}" y="{y}" width="{req_width}" height="{req_height}" class="requirement" rx="5"/>\n'
    svg += f'  <text x="{x + req_width//2}" y="{y + 20}" class="text title">{req.get("id")}</text>\n'
    
    desc = req.get('description', '')
    if len(desc) > 40:
        desc = desc[:37] + '...'
    svg += f'  <text x="{x + req_width//2}" y="{y + 38}" class="text" font-size="10">{desc}</text>\n'
    svg += f'  <text x="{x + req_width//2}" y="{y + 52}" class="text" font-size="9" fill="#666">{req.get("priority")}</text>\n'

comp_y = 400
comp_x_start = 50
comp_width = 140
comp_height = 80
comp_spacing = 20

svg += '  <!-- Components -->\n'
for i, comp in enumerate(components):
    x = comp_x_start + (i % 5) * (comp_width + comp_spacing)
    y = comp_y + (i // 5) * (comp_height + comp_spacing)
    positions[comp.get('id')] = (x + comp_width//2, y + comp_height//2)
    
    svg += f'  <rect x="{x}" y="{y}" width="{comp_width}" height="{comp_height}" class="component" rx="5"/>\n'
    svg += f'  <text x="{x + comp_width//2}" y="{y + 25}" class="text title">{comp.get("id")}</text>\n'
    
    name = comp.get('name', '')
    if len(name) > 15:
        words = name.split()
        line1 = ' '.join(words[:len(words)//2])
        line2 = ' '.join(words[len(words)//2:])
        svg += f'  <text x="{x + comp_width//2}" y="{y + 45}" class="text" font-size="10">{line1}</text>\n'
        svg += f'  <text x="{x + comp_width//2}" y="{y + 58}" class="text" font-size="10">{line2}</text>\n'
    else:
        svg += f'  <text x="{x + comp_width//2}" y="{y + 50}" class="text" font-size="10">{name}</text>\n'

svg += '  <!-- Traces -->\n'
for trace in traces[:30]:
    from_id = trace.get('from')
    to_id = trace.get('to')
    trace_type = trace.get('type')
    
    if from_id in positions and to_id in positions:
        x1, y1 = positions[from_id]
        x2, y2 = positions[to_id]
        
        class_name = 'trace satisfies' if trace_type == 'satisfies' else 'trace implements'
        svg += f'  <line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" class="{class_name}"/>\n'

svg += '</svg>'

with open(svg_file, 'w') as f:
    f.write(svg)

print(f"SVG diagram generated: {svg_file}")
