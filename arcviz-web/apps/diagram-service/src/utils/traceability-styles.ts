/**
 * Traceability Link Styles
 * 
 * Implements LaTeX Specification Section 6 (Traceability Visualization), Pages 20-21
 * 
 * Traceability link types across the 5 Arcadia layers:
 * - Realizes: Higher-level element realized by lower-level (dashed arrow)
 * - Refines: More detailed specification (dotted arrow)
 * - Allocates: Function allocated to component (solid arrow)
 * - Implements: Component implements requirement (dashed blue arrow)
 * - Satisfies: Architecture satisfies requirement (dashed green arrow)
 * 
 * Vertical traceability across layers:
 * OA (Operational) → SA (System) → LA (Logical) → PA (Physical) → EPBS
 */

import { Point } from '../types/diagram';

export type TraceabilityLinkType = 
  | 'realizes'      // OA → SA, SA → LA
  | 'refines'       // SA → LA, LA → PA
  | 'allocates'     // Function → Component
  | 'implements'    // Component → Requirement
  | 'satisfies'     // Architecture → Requirement
  | 'derives'       // Requirement → Requirement
  | 'justifies'     // Design decision → Requirement
  | 'verifies'      // Test → Requirement
  | 'traces'        // Generic traceability

export interface TraceabilityStyle {
  strokeColor: string;
  strokeWidth: number;
  strokeDasharray: string;
  markerEnd: string;
  opacity: number;
  labelStyle: LabelStyle;
  description: string;
}

export interface LabelStyle {
  fontSize: number;
  fontStyle: 'normal' | 'italic' | 'oblique';
  fontWeight: 'normal' | 'bold';
  fill: string;
  backgroundColor: string;
  border: string;
}

/**
 * Traceability link style specifications
 * LaTeX spec page 21, Table: Traceability Link Notation
 */
const TRACEABILITY_STYLES: Record<TraceabilityLinkType, TraceabilityStyle> = {
  realizes: {
    strokeColor: '#607D8B',
    strokeWidth: 2,
    strokeDasharray: '8,4',
    markerEnd: 'arrow-realizes',
    opacity: 0.8,
    labelStyle: {
      fontSize: 11,
      fontStyle: 'italic',
      fontWeight: 'normal',
      fill: '#607D8B',
      backgroundColor: '#FFFFFF',
      border: '#607D8B',
    },
    description: 'Higher-level element realized by lower-level',
  },
  
  refines: {
    strokeColor: '#9C27B0',
    strokeWidth: 2,
    strokeDasharray: '4,4',
    markerEnd: 'arrow-refines',
    opacity: 0.7,
    labelStyle: {
      fontSize: 11,
      fontStyle: 'italic',
      fontWeight: 'normal',
      fill: '#9C27B0',
      backgroundColor: '#FFFFFF',
      border: '#9C27B0',
    },
    description: 'More detailed specification',
  },
  
  allocates: {
    strokeColor: '#FF9800',
    strokeWidth: 2.5,
    strokeDasharray: 'none',
    markerEnd: 'arrow-allocates',
    opacity: 0.9,
    labelStyle: {
      fontSize: 12,
      fontStyle: 'normal',
      fontWeight: 'bold',
      fill: '#FF9800',
      backgroundColor: '#FFF3E0',
      border: '#FF9800',
    },
    description: 'Function allocated to component',
  },
  
  implements: {
    strokeColor: '#2196F3',
    strokeWidth: 2,
    strokeDasharray: '10,5',
    markerEnd: 'arrow-implements',
    opacity: 0.8,
    labelStyle: {
      fontSize: 11,
      fontStyle: 'italic',
      fontWeight: 'normal',
      fill: '#2196F3',
      backgroundColor: '#E3F2FD',
      border: '#2196F3',
    },
    description: 'Component implements requirement',
  },
  
  satisfies: {
    strokeColor: '#4CAF50',
    strokeWidth: 2,
    strokeDasharray: '8,4',
    markerEnd: 'arrow-satisfies',
    opacity: 0.8,
    labelStyle: {
      fontSize: 11,
      fontStyle: 'italic',
      fontWeight: 'normal',
      fill: '#4CAF50',
      backgroundColor: '#E8F5E9',
      border: '#4CAF50',
    },
    description: 'Architecture satisfies requirement',
  },
  
  derives: {
    strokeColor: '#795548',
    strokeWidth: 1.5,
    strokeDasharray: '6,3',
    markerEnd: 'arrow-derives',
    opacity: 0.7,
    labelStyle: {
      fontSize: 10,
      fontStyle: 'italic',
      fontWeight: 'normal',
      fill: '#795548',
      backgroundColor: '#FFFFFF',
      border: '#795548',
    },
    description: 'Requirement derived from requirement',
  },
  
  justifies: {
    strokeColor: '#E91E63',
    strokeWidth: 1.5,
    strokeDasharray: '5,5',
    markerEnd: 'arrow-justifies',
    opacity: 0.7,
    labelStyle: {
      fontSize: 10,
      fontStyle: 'italic',
      fontWeight: 'normal',
      fill: '#E91E63',
      backgroundColor: '#FCE4EC',
      border: '#E91E63',
    },
    description: 'Design decision justifies requirement',
  },
  
  verifies: {
    strokeColor: '#00BCD4',
    strokeWidth: 2,
    strokeDasharray: '12,3',
    markerEnd: 'arrow-verifies',
    opacity: 0.8,
    labelStyle: {
      fontSize: 11,
      fontStyle: 'normal',
      fontWeight: 'bold',
      fill: '#00BCD4',
      backgroundColor: '#E0F7FA',
      border: '#00BCD4',
    },
    description: 'Test verifies requirement',
  },
  
  traces: {
    strokeColor: '#9E9E9E',
    strokeWidth: 1.5,
    strokeDasharray: '6,6',
    markerEnd: 'arrow-traces',
    opacity: 0.6,
    labelStyle: {
      fontSize: 10,
      fontStyle: 'italic',
      fontWeight: 'normal',
      fill: '#9E9E9E',
      backgroundColor: '#FFFFFF',
      border: '#9E9E9E',
    },
    description: 'Generic traceability link',
  },
};

/**
 * Get traceability style for a given link type
 */
export function getTraceabilityStyle(linkType: TraceabilityLinkType): TraceabilityStyle {
  return TRACEABILITY_STYLES[linkType] || TRACEABILITY_STYLES.traces;
}

/**
 * Create SVG arrow marker for traceability link
 */
export function createTraceabilityMarker(linkType: TraceabilityLinkType): string {
  const style = getTraceabilityStyle(linkType);
  const markerId = `arrow-${linkType}`;
  
  return `
    <marker
      id="${markerId}"
      viewBox="0 0 10 10"
      refX="9"
      refY="5"
      markerWidth="6"
      markerHeight="6"
      orient="auto-start-reverse"
      markerUnits="strokeWidth"
    >
      <path
        d="M 0 0 L 10 5 L 0 10 z"
        fill="${style.strokeColor}"
        opacity="${style.opacity}"
      />
    </marker>
  `;
}

/**
 * Create all traceability markers for SVG defs
 */
export function createAllTraceabilityMarkers(): string {
  const linkTypes: TraceabilityLinkType[] = [
    'realizes', 'refines', 'allocates', 'implements', 'satisfies',
    'derives', 'justifies', 'verifies', 'traces'
  ];
  
  return linkTypes.map(type => createTraceabilityMarker(type)).join('\n');
}

/**
 * Generate SVG path for traceability link
 */
export function createTraceabilityLink(
  points: Point[],
  linkType: TraceabilityLinkType,
  label?: string
): string {
  const style = getTraceabilityStyle(linkType);
  
  if (points.length < 2) {
    return '';
  }
  
  const pathD = points.map((p, i) => 
    i === 0 ? `M ${p.x} ${p.y}` : `L ${p.x} ${p.y}`
  ).join(' ');
  
  const strokeDasharray = style.strokeDasharray === 'none' ? '' : style.strokeDasharray;
  
  let svg = `
    <g class="traceability-link" data-type="${linkType}">
      <path
        d="${pathD}"
        stroke="${style.strokeColor}"
        stroke-width="${style.strokeWidth}"
        ${strokeDasharray ? `stroke-dasharray="${strokeDasharray}"` : ''}
        fill="none"
        opacity="${style.opacity}"
        marker-end="url(#${style.markerEnd})"
      />
  `;
  
  if (label) {
    const midIdx = Math.floor(points.length / 2);
    const midPoint = points[midIdx];
    
    svg += createTraceabilityLabel(
      midPoint.x,
      midPoint.y,
      label,
      linkType
    );
  }
  
  svg += `
    </g>
  `;
  
  return svg;
}

/**
 * Create label for traceability link
 */
export function createTraceabilityLabel(
  x: number,
  y: number,
  text: string,
  linkType: TraceabilityLinkType
): string {
  const style = getTraceabilityStyle(linkType);
  const labelStyle = style.labelStyle;
  
  const padding = 6;
  const approxWidth = text.length * labelStyle.fontSize * 0.6 + padding * 2;
  const height = labelStyle.fontSize + padding;
  
  return `
    <g class="traceability-label">
      <rect
        x="${x - approxWidth / 2}"
        y="${y - height / 2}"
        width="${approxWidth}"
        height="${height}"
        rx="3"
        fill="${labelStyle.backgroundColor}"
        stroke="${labelStyle.border}"
        stroke-width="1"
        opacity="0.95"
      />
      <text
        x="${x}"
        y="${y}"
        text-anchor="middle"
        dominant-baseline="middle"
        font-family="Arial, sans-serif"
        font-size="${labelStyle.fontSize}"
        font-style="${labelStyle.fontStyle}"
        font-weight="${labelStyle.fontWeight}"
        fill="${labelStyle.fill}"
      >«${text}»</text>
    </g>
  `;
}

/**
 * Determine traceability link type from layer transition
 */
export function inferTraceabilityType(
  fromLayer: 'OA' | 'SA' | 'LA' | 'PA' | 'EPBS' | 'REQ',
  toLayer: 'OA' | 'SA' | 'LA' | 'PA' | 'EPBS' | 'REQ',
  fromElementType?: string,
  toElementType?: string
): TraceabilityLinkType {
  if (toLayer === 'REQ') {
    if (fromElementType === 'test') {
      return 'verifies';
    } else if (fromElementType === 'decision') {
      return 'justifies';
    } else if (fromLayer === 'OA' || fromLayer === 'SA') {
      return 'satisfies';
    } else {
      return 'implements';
    }
  }
  
  if (fromLayer === 'REQ') {
    return 'derives';
  }
  
  if (fromElementType === 'function' && toElementType === 'component') {
    return 'allocates';
  }
  
  if (fromLayer === 'OA' && toLayer === 'SA') {
    return 'realizes';
  }
  
  if (fromLayer === 'SA' && toLayer === 'LA') {
    return 'realizes';
  }
  
  if (fromLayer === 'LA' && toLayer === 'PA') {
    return 'refines';
  }
  
  if (fromLayer === 'PA' && toLayer === 'EPBS') {
    return 'refines';
  }
  
  return 'traces';
}

/**
 * Generate vertical traceability diagram
 * Shows links across all 5 Arcadia layers
 */
export function createVerticalTraceabilityDiagram(
  traces: Array<{
    from: { id: string; label: string; layer: string };
    to: { id: string; label: string; layer: string };
    type?: TraceabilityLinkType;
  }>
): string {
  const layers = ['OA', 'SA', 'LA', 'PA', 'EPBS'];
  const layerY: Record<string, number> = {
    OA: 100,
    SA: 250,
    LA: 400,
    PA: 550,
    EPBS: 700,
  };
  
  const layerColors: Record<string, string> = {
    OA: '#FFD966',
    SA: '#ADD8E6',
    LA: '#6495ED',
    PA: '#4169E1',
    EPBS: '#90EE90',
  };
  
  let svg = `
    <svg width="800" height="850" xmlns="http://www.w3.org/2000/svg">
      <defs>
        ${createAllTraceabilityMarkers()}
      </defs>
  `;
  
  svg += `<rect width="800" height="850" fill="#FAFAFA"/>`;
  
  svg += `
    <text x="400" y="40" text-anchor="middle" font-size="20" font-weight="bold" fill="#333">
      Vertical Traceability - Arcadia Layers
    </text>
  `;
  
  for (const layer of layers) {
    const y = layerY[layer];
    svg += `
      <rect x="50" y="${y - 30}" width="700" height="80" rx="8" 
            fill="${layerColors[layer]}" stroke="#333" stroke-width="2" opacity="0.3"/>
      <text x="80" y="${y}" font-size="16" font-weight="bold" fill="#333">${layer} Layer</text>
    `;
  }
  
  const elementPositions: Map<string, Point> = new Map();
  let xOffset = 200;
  
  for (const trace of traces) {
    if (!elementPositions.has(trace.from.id)) {
      elementPositions.set(trace.from.id, {
        x: xOffset,
        y: layerY[trace.from.layer],
      });
      xOffset += 150;
    }
    
    if (!elementPositions.has(trace.to.id)) {
      elementPositions.set(trace.to.id, {
        x: xOffset,
        y: layerY[trace.to.layer],
      });
    }
  }
  
  for (const trace of traces) {
    const fromPos = elementPositions.get(trace.from.id);
    const toPos = elementPositions.get(trace.to.id);
    
    if (fromPos && toPos) {
      const linkType = trace.type || inferTraceabilityType(
        trace.from.layer as any,
        trace.to.layer as any
      );
      
      svg += createTraceabilityLink(
        [fromPos, toPos],
        linkType,
        linkType
      );
    }
  }
  
  for (const [id, pos] of elementPositions.entries()) {
    const trace = traces.find(t => t.from.id === id || t.to.id === id);
    const label = trace?.from.id === id ? trace.from.label : trace?.to.label || id;
    
    svg += `
      <circle cx="${pos.x}" cy="${pos.y}" r="8" fill="#333" stroke="#FFF" stroke-width="2"/>
      <text x="${pos.x}" y="${pos.y + 25}" text-anchor="middle" font-size="10" fill="#333">
        ${label}
      </text>
    `;
  }
  
  svg += `</svg>`;
  
  return svg;
}

/**
 * Generate traceability matrix (Requirements Traceability Matrix - RTM)
 */
export function generateTraceabilityMatrix(
  requirements: Array<{ id: string; name: string }>,
  components: Array<{ id: string; name: string }>,
  traces: Array<{ from: string; to: string; type: TraceabilityLinkType }>
): string {
  const matrix: string[][] = [];
  
  const header = ['Requirement', ...components.map(c => c.name)];
  matrix.push(header);
  
  for (const req of requirements) {
    const row: string[] = [req.name];
    
    for (const comp of components) {
      const trace = traces.find(t => 
        t.from === req.id && t.to === comp.id
      );
      
      if (trace) {
        const style = getTraceabilityStyle(trace.type);
        row.push(`✓ ${trace.type}`);
      } else {
        row.push('—');
      }
    }
    
    matrix.push(row);
  }
  
  let html = '<table class="traceability-matrix" style="border-collapse: collapse; width: 100%;">';
  
  for (let i = 0; i < matrix.length; i++) {
    html += '<tr>';
    for (let j = 0; j < matrix[i].length; j++) {
      const isHeader = i === 0 || j === 0;
      const tag = isHeader ? 'th' : 'td';
      const style = isHeader 
        ? 'background: #E0E0E0; font-weight: bold; padding: 8px; border: 1px solid #999;'
        : 'padding: 8px; border: 1px solid #CCC;';
      
      html += `<${tag} style="${style}">${matrix[i][j]}</${tag}>`;
    }
    html += '</tr>';
  }
  
  html += '</table>';
  
  return html;
}

/**
 * Generate traceability coverage report
 */
export function generateTraceabilityCoverageReport(
  traces: Array<{ from: string; to: string; type: TraceabilityLinkType }>,
  requirements: string[],
  components: string[]
): {
  coverage: number;
  implementedRequirements: number;
  unimplementedRequirements: string[];
  orphanedComponents: string[];
} {
  const implementedReqs = new Set<string>();
  const connectedComps = new Set<string>();
  
  for (const trace of traces) {
    if (trace.type === 'implements' || trace.type === 'satisfies') {
      implementedReqs.add(trace.from);
      connectedComps.add(trace.to);
    }
  }
  
  const unimplementedRequirements = requirements.filter(r => !implementedReqs.has(r));
  const orphanedComponents = components.filter(c => !connectedComps.has(c));
  
  const coverage = requirements.length > 0 
    ? (implementedReqs.size / requirements.length) * 100 
    : 100;
  
  return {
    coverage,
    implementedRequirements: implementedReqs.size,
    unimplementedRequirements,
    orphanedComponents,
  };
}

/**
 * Export all traceability link types
 */
export const TRACEABILITY_LINK_TYPES: TraceabilityLinkType[] = [
  'realizes', 'refines', 'allocates', 'implements', 'satisfies',
  'derives', 'justifies', 'verifies', 'traces'
];

/**
 * Get traceability icon
 */
export function getTraceabilityIcon(linkType: TraceabilityLinkType): string {
  const icons: Record<TraceabilityLinkType, string> = {
    realizes: '⬇',
    refines: '⤵',
    allocates: '➜',
    implements: '✓',
    satisfies: '✔',
    derives: '↳',
    justifies: '∵',
    verifies: '⊗',
    traces: '⤴',
  };
  
  return icons[linkType] || '→';
}
