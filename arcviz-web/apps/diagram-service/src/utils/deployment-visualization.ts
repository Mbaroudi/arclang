/**
 * Physical Architecture Deployment Visualization
 * 
 * Implements LaTeX Specification Section 4.5 (Physical Architecture), Page 14
 * 
 * Key Requirements:
 * - HW/SW Separation: Node Components (yellow/gold) vs Behavioral (blue)
 * - Deployment: Behavioral components NESTED INSIDE physical nodes
 * - Physical Links: Distinguished from logical connections
 * - Clear visual hierarchy: Hardware contains Software
 * 
 * Achieves Capella-compliant physical deployment diagrams.
 */

import { DiagramNode, Point, Size } from '../types/diagram';
import { SvgElement } from '../types/diagram';
import { createRoundedRect, createText, createLine, createGroup, createPath } from '../utils/svg';

export interface DeployedComponent {
  id: string;
  name: string;
  type: 'behavior' | 'hardware';
  safety_level?: string;
}

export interface PhysicalNodeWithDeployment extends DiagramNode {
  position: Point;
  size: Size;
  metadata: {
    nodeType?: string;
    deployedComponents?: DeployedComponent[];
    processor?: string;
    memory?: string;
    [key: string]: any;
  };
}

/**
 * Render physical node with deployed behavioral components
 * Section 4.5: Behavioral components NESTED inside nodes
 */
export function renderPhysicalNodeWithDeployment(
  node: PhysicalNodeWithDeployment,
  config: {
    nodeColor?: string;
    behaviorColor?: string;
    fontSize?: number;
  } = {}
): SvgElement {
  const elements: SvgElement[] = [];
  
  const nodeColor = config.nodeColor || '#FFD700'; // Capella Physical Node (gold)
  const behaviorColor = config.behaviorColor || '#4169E1'; // Capella Physical Behavioral (royal blue)
  const fontSize = config.fontSize || 12;
  
  const isHardware = node.metadata?.nodeType?.toLowerCase().includes('hardware') ||
                     node.metadata?.nodeType?.toLowerCase().includes('node');
  
  // 1. Render hardware container (3D cube effect)
  if (isHardware) {
    // Main box
    elements.push(
      createRoundedRect(
        node.position.x,
        node.position.y,
        node.size.width,
        node.size.height,
        5,
        {
          fill: nodeColor,
          stroke: '#000000',
          'stroke-width': 2.5,
          filter: 'url(#drop-shadow)',
        }
      )
    );
    
    // 3D effect - top face
    const topPath = `M ${node.position.x} ${node.position.y} 
                     L ${node.position.x + 12} ${node.position.y - 12} 
                     L ${node.position.x + node.size.width + 12} ${node.position.y - 12} 
                     L ${node.position.x + node.size.width} ${node.position.y} Z`;
    elements.push(
      createPath(topPath, {
        fill: adjustBrightness(nodeColor, 1.3),
        stroke: '#000000',
        'stroke-width': 2,
      })
    );
    
    // 3D effect - right face
    const rightPath = `M ${node.position.x + node.size.width} ${node.position.y} 
                       L ${node.position.x + node.size.width + 12} ${node.position.y - 12} 
                       L ${node.position.x + node.size.width + 12} ${node.position.y + node.size.height - 12} 
                       L ${node.position.x + node.size.width} ${node.position.y + node.size.height} Z`;
    elements.push(
      createPath(rightPath, {
        fill: adjustBrightness(nodeColor, 0.7),
        stroke: '#000000',
        'stroke-width': 2,
      })
    );
  }
  
  // 2. Header area with node name
  const headerHeight = 35;
  elements.push(
    createRoundedRect(
      node.position.x + 5,
      node.position.y + 5,
      node.size.width - 10,
      headerHeight,
      4,
      {
        fill: adjustBrightness(nodeColor, 0.9),
        stroke: '#000000',
        'stroke-width': 1,
      }
    )
  );
  
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 23,
      node.label,
      {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': fontSize + 2,
        'font-weight': 'bold',
        fill: '#000000',
      }
    )
  );
  
  // 3. Stereotype
  const stereotype = getPhysicalStereotype(node.metadata?.nodeType);
  if (stereotype) {
    elements.push(
      createText(
        node.position.x + 10,
        node.position.y + 15,
        stereotype,
        {
          'font-family': 'Arial, sans-serif',
          'font-size': 9,
          'font-style': 'italic',
          fill: '#333333',
        }
      )
    );
  }
  
  // 4. Separator line
  elements.push(
    createLine(
      node.position.x + 10,
      node.position.y + headerHeight + 10,
      node.position.x + node.size.width - 10,
      node.position.y + headerHeight + 10,
      {
        stroke: '#000000',
        'stroke-width': 1.5,
        'stroke-dasharray': '5,3',
      }
    )
  );
  
  // 5. Render NESTED behavioral components (KEY FEATURE - Section 4.5)
  const deployedComponents = node.metadata?.deployedComponents || [];
  const behaviorComponents = deployedComponents.filter(c => c.type === 'behavior');
  
  if (behaviorComponents.length > 0) {
    let yOffset = headerHeight + 25;
    const componentWidth = node.size.width - 30;
    const componentHeight = 50;
    const componentSpacing = 10;
    
    behaviorComponents.forEach((component, index) => {
      if (index < 4) { // Limit to 4 visible components
        // Behavioral component box (NESTED inside physical node)
        elements.push(
          createRoundedRect(
            node.position.x + 15,
            node.position.y + yOffset,
            componentWidth,
            componentHeight,
            6,
            {
              fill: behaviorColor,
              stroke: '#000000',
              'stroke-width': 1.5,
              'stroke-dasharray': '3,2', // Dashed to show it's nested
            }
          )
        );
        
        // Component name
        elements.push(
          createText(
            node.position.x + node.size.width / 2,
            node.position.y + yOffset + componentHeight / 2,
            component.name,
            {
              'text-anchor': 'middle',
              'dominant-baseline': 'middle',
              'font-family': 'Arial, sans-serif',
              'font-size': fontSize - 1,
              'font-weight': 'bold',
              fill: '#FFFFFF',
            }
          )
        );
        
        // <<behavior>> stereotype
        elements.push(
          createText(
            node.position.x + node.size.width / 2,
            node.position.y + yOffset + 12,
            '<<behavior>>',
            {
              'text-anchor': 'middle',
              'font-family': 'Arial, sans-serif',
              'font-size': 8,
              'font-style': 'italic',
              fill: '#E0E0E0',
            }
          )
        );
        
        // Safety level if present
        if (component.safety_level) {
          elements.push(
            createRoundedRect(
              node.position.x + node.size.width - 65,
              node.position.y + yOffset + 5,
              50,
              15,
              3,
              {
                fill: getSafetyBadgeColor(component.safety_level),
                stroke: '#000000',
                'stroke-width': 1,
              }
            )
          );
          
          elements.push(
            createText(
              node.position.x + node.size.width - 40,
              node.position.y + yOffset + 12,
              component.safety_level,
              {
                'text-anchor': 'middle',
                'font-family': 'Arial, sans-serif',
                'font-size': 9,
                'font-weight': 'bold',
                fill: '#000000',
              }
            )
          );
        }
        
        yOffset += componentHeight + componentSpacing;
      }
    });
    
    // "..." indicator if more components exist
    if (behaviorComponents.length > 4) {
      elements.push(
        createText(
          node.position.x + node.size.width / 2,
          node.position.y + yOffset,
          `... +${behaviorComponents.length - 4} more`,
          {
            'text-anchor': 'middle',
            'font-family': 'Arial, sans-serif',
            'font-size': 10,
            'font-style': 'italic',
            fill: '#666666',
          }
        )
      );
    }
  } else {
    // No deployed components - show empty state
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + headerHeight + 40,
        '(no deployed components)',
        {
          'text-anchor': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 10,
          'font-style': 'italic',
          fill: '#999999',
        }
      )
    );
  }
  
  // 6. Technical specifications (processor, memory)
  const techSpecs: string[] = [];
  if (node.metadata?.processor) {
    techSpecs.push(`CPU: ${node.metadata.processor}`);
  }
  if (node.metadata?.memory) {
    techSpecs.push(`MEM: ${node.metadata.memory}`);
  }
  
  if (techSpecs.length > 0) {
    let specY = node.position.y + node.size.height - 25;
    techSpecs.forEach((spec, index) => {
      elements.push(
        createText(
          node.position.x + 15,
          specY + (index * 12),
          spec,
          {
            'font-family': 'Courier, monospace',
            'font-size': 9,
            fill: '#555555',
          }
        )
      );
    });
  }
  
  return createGroup(elements, { 
    id: `physical-node-${node.id}`,
    class: 'physical-node-with-deployment'
  });
}

/**
 * Calculate required size for physical node with nested components
 */
export function calculatePhysicalNodeSize(
  deployedComponents: DeployedComponent[],
  baseWidth: number = 220,
  baseHeight: number = 150
): Size {
  const behaviorComponents = deployedComponents.filter(c => c.type === 'behavior');
  const componentHeight = 50;
  const componentSpacing = 10;
  const headerHeight = 35;
  const padding = 50;
  
  const visibleComponents = Math.min(behaviorComponents.length, 4);
  const contentHeight = headerHeight + 
                       (visibleComponents * (componentHeight + componentSpacing)) +
                       padding;
  
  return {
    width: Math.max(baseWidth, 220),
    height: Math.max(baseHeight, contentHeight),
  };
}

/**
 * Render physical link (hardware connection)
 * Distinguished from logical connections with specific styling
 */
export function renderPhysicalLink(
  from: Point,
  to: Point,
  label?: string,
  protocol?: string
): SvgElement[] {
  const elements: SvgElement[] = [];
  
  // Physical link - thick solid line
  const pathD = `M ${from.x} ${from.y} L ${to.x} ${to.y}`;
  elements.push(
    createPath(pathD, {
      fill: 'none',
      stroke: '#8B4513', // Brown for physical/hardware connections
      'stroke-width': 3,
      'marker-end': 'url(#arrow-physical-link)',
    })
  );
  
  // Label with protocol
  if (label || protocol) {
    const midX = (from.x + to.x) / 2;
    const midY = (from.y + to.y) / 2;
    const text = protocol ? `<<${protocol}>>` : label || '';
    
    // Background
    const textWidth = text.length * 6 + 10;
    elements.push(
      createRoundedRect(
        midX - textWidth / 2,
        midY - 10,
        textWidth,
        20,
        3,
        {
          fill: '#FFF8DC', // Cornsilk background
          stroke: '#8B4513',
          'stroke-width': 1.5,
        }
      )
    );
    
    // Text
    elements.push(
      createText(
        midX,
        midY,
        text,
        {
          'text-anchor': 'middle',
          'dominant-baseline': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 10,
          'font-weight': 'bold',
          fill: '#8B4513',
        }
      )
    );
  }
  
  return elements;
}

// ============================================================================
// Helper Functions
// ============================================================================

function adjustBrightness(color: string, factor: number): string {
  // Simple brightness adjustment (assuming hex color)
  const hex = color.replace('#', '');
  const r = Math.min(255, Math.round(parseInt(hex.substr(0, 2), 16) * factor));
  const g = Math.min(255, Math.round(parseInt(hex.substr(2, 2), 16) * factor));
  const b = Math.min(255, Math.round(parseInt(hex.substr(4, 2), 16) * factor));
  return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
}

function getPhysicalStereotype(nodeType?: string): string {
  if (!nodeType) return '<<node>>';
  
  const lower = nodeType.toLowerCase();
  if (lower.includes('hardware')) return '<<hardware>>';
  if (lower.includes('node')) return '<<node>>';
  if (lower.includes('computer')) return '<<computer>>';
  if (lower.includes('ecu')) return '<<ECU>>';
  if (lower.includes('processor')) return '<<processor>>';
  if (lower.includes('network')) return '<<network>>';
  
  return '<<node>>';
}

function getSafetyBadgeColor(level: string): string {
  const upper = level.toUpperCase();
  
  // ASIL levels
  if (upper.includes('ASIL-D')) return '#FF4444';
  if (upper.includes('ASIL-C')) return '#FF8844';
  if (upper.includes('ASIL-B')) return '#FFBB44';
  if (upper.includes('ASIL-A')) return '#FFEE44';
  
  // DAL levels
  if (upper.includes('DAL-A')) return '#FF4444';
  if (upper.includes('DAL-B')) return '#FF8844';
  if (upper.includes('DAL-C')) return '#FFBB44';
  if (upper.includes('DAL-D')) return '#FFEE44';
  
  // SIL levels
  if (upper.includes('SIL-4')) return '#FF4444';
  if (upper.includes('SIL-3')) return '#FF8844';
  if (upper.includes('SIL-2')) return '#FFBB44';
  if (upper.includes('SIL-1')) return '#FFEE44';
  
  return '#CCCCCC'; // QM or unknown
}

/**
 * Create SVG marker for physical links
 */
export function createPhysicalLinkMarker(): string {
  return `
    <marker
      id="arrow-physical-link"
      viewBox="0 0 10 10"
      refX="9"
      refY="5"
      markerWidth="8"
      markerHeight="8"
      orient="auto"
    >
      <path
        d="M 0 0 L 10 5 L 0 10 z"
        fill="#8B4513"
      />
    </marker>
  `;
}
