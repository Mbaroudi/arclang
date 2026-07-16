/**
 * Exchange Item Type Visualization
 * 
 * Implements LaTeX Specification Section 5 (Exchange Item Specification)
 * Visualizes different types of data/information flow through interfaces
 * 
 * Exchange Item Types (per Capella metamodel):
 * - Event: Signal with no data payload
 * - Flow: Continuous data stream
 * - Operation: Request-response pattern (function call)
 * - Data: Structured information package
 * - Shared Data: Common data repository access
 * - Unset: Undefined/generic exchange
 */

import { Point } from '../types/diagram';

export type ExchangeItemKind = 
  | 'EVENT'
  | 'FLOW'
  | 'OPERATION'
  | 'DATA'
  | 'SHARED_DATA'
  | 'UNSET';

export interface ExchangeItemStyle {
  arrowType: 'solid' | 'dashed' | 'dotted' | 'double';
  arrowHead: 'standard' | 'open' | 'filled' | 'diamond' | 'none';
  color: string;
  strokeWidth: number;
  pattern: string;
  labelStyle: {
    prefix: string;
    suffix: string;
    fontStyle: string;
    fontWeight: string;
  };
  icon: string;
  description: string;
}

/**
 * Exchange item visual styles per type
 */
const EXCHANGE_ITEM_STYLES: Record<ExchangeItemKind, ExchangeItemStyle> = {
  EVENT: {
    arrowType: 'dashed',
    arrowHead: 'open',
    color: '#FF6B6B',
    strokeWidth: 2,
    pattern: '5,5',
    labelStyle: {
      prefix: '⚡ ',
      suffix: ' (event)',
      fontStyle: 'italic',
      fontWeight: 'normal',
    },
    icon: '⚡',
    description: 'Event signal with no data payload',
  },
  
  FLOW: {
    arrowType: 'solid',
    arrowHead: 'filled',
    color: '#4ECDC4',
    strokeWidth: 3,
    pattern: 'none',
    labelStyle: {
      prefix: '⟿ ',
      suffix: ' (flow)',
      fontStyle: 'normal',
      fontWeight: 'bold',
    },
    icon: '⟿',
    description: 'Continuous data stream',
  },
  
  OPERATION: {
    arrowType: 'double',
    arrowHead: 'standard',
    color: '#95E1D3',
    strokeWidth: 2,
    pattern: 'none',
    labelStyle: {
      prefix: '↔ ',
      suffix: '()',
      fontStyle: 'normal',
      fontWeight: 'normal',
    },
    icon: '↔',
    description: 'Request-response operation (function call)',
  },
  
  DATA: {
    arrowType: 'solid',
    arrowHead: 'standard',
    color: '#808080',  // CORRECTED: Capella spec - exchanges are gray
    strokeWidth: 2,
    pattern: 'none',
    labelStyle: {
      prefix: '📦 ',
      suffix: '',
      fontStyle: 'normal',
      fontWeight: 'normal',
    },
    icon: '📦',
    description: 'Structured data package',
  },
  
  SHARED_DATA: {
    arrowType: 'dotted',
    arrowHead: 'diamond',
    color: '#9B59B6',
    strokeWidth: 2,
    pattern: '2,3',
    labelStyle: {
      prefix: '🗄 ',
      suffix: ' (shared)',
      fontStyle: 'italic',
      fontWeight: 'normal',
    },
    icon: '🗄',
    description: 'Shared data repository access',
  },
  
  UNSET: {
    arrowType: 'solid',
    arrowHead: 'standard',
    color: '#95A5A6',
    strokeWidth: 1.5,
    pattern: 'none',
    labelStyle: {
      prefix: '',
      suffix: '',
      fontStyle: 'normal',
      fontWeight: 'normal',
    },
    icon: '→',
    description: 'Generic exchange (type not specified)',
  },
};

/**
 * Get visual style for exchange item type
 */
export function getExchangeItemStyle(kind: ExchangeItemKind): ExchangeItemStyle {
  return EXCHANGE_ITEM_STYLES[kind] || EXCHANGE_ITEM_STYLES.UNSET;
}

/**
 * Create SVG arrow marker for exchange item type
 */
export function createExchangeItemMarker(kind: ExchangeItemKind): string {
  const style = getExchangeItemStyle(kind);
  const markerId = `arrow-${kind.toLowerCase()}`;
  
  let markerPath: string;
  
  switch (style.arrowHead) {
    case 'open':
      markerPath = `<path d="M 0 0 L 10 5 L 0 10" fill="none" stroke="${style.color}" stroke-width="2"/>`;
      break;
    case 'filled':
      markerPath = `<path d="M 0 0 L 10 5 L 0 10 z" fill="${style.color}"/>`;
      break;
    case 'diamond':
      markerPath = `<path d="M 0 5 L 5 0 L 10 5 L 5 10 z" fill="none" stroke="${style.color}" stroke-width="2"/>`;
      break;
    case 'none':
      markerPath = '';
      break;
    case 'standard':
    default:
      markerPath = `<path d="M 0 0 L 10 5 L 0 10 z" fill="${style.color}"/>`;
  }
  
  return `
    <marker
      id="${markerId}"
      viewBox="0 0 10 10"
      refX="9"
      refY="5"
      markerWidth="6"
      markerHeight="6"
      orient="auto-start-reverse"
    >
      ${markerPath}
    </marker>
  `;
}

/**
 * Create all exchange item markers for SVG defs
 */
export function createAllExchangeItemMarkers(): string {
  const kinds: ExchangeItemKind[] = [
    'EVENT', 'FLOW', 'OPERATION', 'DATA', 'SHARED_DATA', 'UNSET'
  ];
  
  return kinds.map(kind => createExchangeItemMarker(kind)).join('\n');
}

/**
 * Create SVG path for exchange item
 */
export function createExchangeItemPath(
  points: Point[],
  kind: ExchangeItemKind,
  label?: string,
  dataType?: string
): string {
  const style = getExchangeItemStyle(kind);
  
  if (points.length < 2) {
    return '';
  }
  
  let pathD = points.map((p, i) => 
    i === 0 ? `M ${p.x} ${p.y}` : `L ${p.x} ${p.y}`
  ).join(' ');
  
  const strokeDasharray = style.pattern === 'none' ? '' : style.pattern;
  
  let svg = `
    <g class="exchange-item" data-kind="${kind}">
  `;
  
  if (style.arrowType === 'double') {
    svg += `
      <path
        d="${pathD}"
        stroke="${style.color}"
        stroke-width="${style.strokeWidth + 2}"
        fill="none"
        opacity="0.3"
      />
    `;
  }
  
  svg += `
      <path
        d="${pathD}"
        stroke="${style.color}"
        stroke-width="${style.strokeWidth}"
        ${strokeDasharray ? `stroke-dasharray="${strokeDasharray}"` : ''}
        fill="none"
        marker-end="url(#arrow-${kind.toLowerCase()})"
      />
  `;
  
  if (label || dataType) {
    const midIdx = Math.floor(points.length / 2);
    const midPoint = points[midIdx];
    
    const fullLabel = formatExchangeLabel(label, dataType, kind, style);
    
    svg += createExchangeItemLabel(
      midPoint.x,
      midPoint.y,
      fullLabel,
      style
    );
  }
  
  svg += `
    </g>
  `;
  
  return svg;
}

/**
 * Format exchange label with prefix/suffix
 */
function formatExchangeLabel(
  label: string | undefined,
  dataType: string | undefined,
  kind: ExchangeItemKind,
  style: ExchangeItemStyle
): string {
  const parts: string[] = [];
  
  if (style.labelStyle.prefix) {
    parts.push(style.labelStyle.prefix);
  }
  
  if (label) {
    parts.push(label);
  } else if (dataType) {
    parts.push(dataType);
  } else {
    parts.push(kind.toLowerCase());
  }
  
  if (style.labelStyle.suffix) {
    parts.push(style.labelStyle.suffix);
  }
  
  return parts.join('');
}

/**
 * Create label for exchange item
 */
function createExchangeItemLabel(
  x: number,
  y: number,
  text: string,
  style: ExchangeItemStyle
): string {
  const padding = 8;
  const fontSize = 11;
  const approxWidth = text.length * fontSize * 0.6 + padding * 2;
  const height = fontSize + padding;
  
  return `
    <g class="exchange-label">
      <rect
        x="${x - approxWidth / 2}"
        y="${y - height / 2}"
        width="${approxWidth}"
        height="${height}"
        rx="4"
        fill="#FFFFFF"
        stroke="${style.color}"
        stroke-width="1.5"
        opacity="0.95"
      />
      <text
        x="${x}"
        y="${y}"
        text-anchor="middle"
        dominant-baseline="middle"
        font-family="Arial, sans-serif"
        font-size="${fontSize}"
        font-style="${style.labelStyle.fontStyle}"
        font-weight="${style.labelStyle.fontWeight}"
        fill="${style.color}"
      >${text}</text>
    </g>
  `;
}

/**
 * Create exchange item icon badge
 */
export function createExchangeItemBadge(
  x: number,
  y: number,
  kind: ExchangeItemKind,
  size: 'small' | 'medium' | 'large' = 'medium'
): string {
  const style = getExchangeItemStyle(kind);
  
  const sizes = {
    small: { diameter: 24, fontSize: 14 },
    medium: { diameter: 32, fontSize: 18 },
    large: { diameter: 40, fontSize: 22 },
  };
  
  const dims = sizes[size];
  const radius = dims.diameter / 2;
  
  return `
    <g class="exchange-badge" data-kind="${kind}">
      <circle
        cx="${x}"
        cy="${y}"
        r="${radius}"
        fill="${style.color}"
        stroke="#FFFFFF"
        stroke-width="2"
        opacity="0.9"
      />
      <text
        x="${x}"
        y="${y}"
        text-anchor="middle"
        dominant-baseline="middle"
        font-size="${dims.fontSize}"
      >${style.icon}</text>
    </g>
  `;
}

/**
 * Create exchange item legend
 */
export function createExchangeItemLegend(
  x: number,
  y: number
): string {
  const kinds: ExchangeItemKind[] = [
    'EVENT', 'FLOW', 'OPERATION', 'DATA', 'SHARED_DATA'
  ];
  
  let svg = `
    <g class="exchange-legend">
      <rect
        x="${x}"
        y="${y}"
        width="280"
        height="${kinds.length * 40 + 50}"
        rx="8"
        fill="white"
        stroke="#CCC"
        stroke-width="2"
        opacity="0.95"
      />
      <text
        x="${x + 140}"
        y="${y + 25}"
        text-anchor="middle"
        font-family="Arial"
        font-size="14"
        font-weight="bold"
        fill="#333"
      >Exchange Item Types</text>
  `;
  
  kinds.forEach((kind, index) => {
    const style = getExchangeItemStyle(kind);
    const itemY = y + 50 + index * 40;
    
    svg += `
      <line
        x1="${x + 15}"
        y1="${itemY}"
        x2="${x + 60}"
        y2="${itemY}"
        stroke="${style.color}"
        stroke-width="${style.strokeWidth}"
        ${style.pattern !== 'none' ? `stroke-dasharray="${style.pattern}"` : ''}
      />
      <text
        x="${x + 70}"
        y="${itemY}"
        dominant-baseline="middle"
        font-family="Arial"
        font-size="12"
        fill="#333"
      >
        ${style.icon} ${kind}
      </text>
      <text
        x="${x + 70}"
        y="${itemY + 12}"
        dominant-baseline="middle"
        font-family="Arial"
        font-size="9"
        font-style="italic"
        fill="#666"
      >${style.description}</text>
    `;
  });
  
  svg += `</g>`;
  
  return svg;
}

/**
 * Infer exchange item kind from metadata
 */
export function inferExchangeItemKind(
  metadata: any
): ExchangeItemKind {
  if (!metadata) return 'UNSET';
  
  const kindStr = (metadata.exchange_kind || metadata.kind || '').toUpperCase();
  
  if (kindStr.includes('EVENT')) return 'EVENT';
  if (kindStr.includes('FLOW')) return 'FLOW';
  if (kindStr.includes('OPERATION') || kindStr.includes('OP')) return 'OPERATION';
  if (kindStr.includes('DATA')) return 'DATA';
  if (kindStr.includes('SHARED')) return 'SHARED_DATA';
  
  const dataType = (metadata.data_type || metadata.type || '').toLowerCase();
  if (dataType.includes('event')) return 'EVENT';
  if (dataType.includes('stream') || dataType.includes('flow')) return 'FLOW';
  if (dataType.includes('function') || dataType.includes('method')) return 'OPERATION';
  if (dataType.includes('shared') || dataType.includes('global')) return 'SHARED_DATA';
  
  return 'DATA';
}

/**
 * Get exchange item statistics
 */
export function getExchangeItemStatistics(
  exchanges: Array<{ kind?: ExchangeItemKind; metadata?: any }>
): Record<ExchangeItemKind, number> {
  const stats: Record<ExchangeItemKind, number> = {
    EVENT: 0,
    FLOW: 0,
    OPERATION: 0,
    DATA: 0,
    SHARED_DATA: 0,
    UNSET: 0,
  };
  
  for (const exchange of exchanges) {
    const kind = exchange.kind || inferExchangeItemKind(exchange.metadata);
    stats[kind]++;
  }
  
  return stats;
}

/**
 * Create exchange item summary card
 */
export function createExchangeItemSummary(
  exchanges: Array<{ kind?: ExchangeItemKind; metadata?: any }>,
  x: number,
  y: number
): string {
  const stats = getExchangeItemStatistics(exchanges);
  const total = exchanges.length;
  
  let svg = `
    <g class="exchange-summary">
      <rect
        x="${x}"
        y="${y}"
        width="220"
        height="160"
        rx="8"
        fill="white"
        stroke="#CCC"
        stroke-width="2"
        opacity="0.95"
      />
      <text
        x="${x + 110}"
        y="${y + 25}"
        text-anchor="middle"
        font-family="Arial"
        font-size="14"
        font-weight="bold"
        fill="#333"
      >Exchange Summary</text>
      <text
        x="${x + 110}"
        y="${y + 45}"
        text-anchor="middle"
        font-family="Arial"
        font-size="12"
        fill="#666"
      >Total: ${total} exchanges</text>
  `;
  
  const kinds: ExchangeItemKind[] = ['EVENT', 'FLOW', 'OPERATION', 'DATA', 'SHARED_DATA'];
  
  kinds.forEach((kind, index) => {
    if (stats[kind] === 0) return;
    
    const style = getExchangeItemStyle(kind);
    const percentage = ((stats[kind] / total) * 100).toFixed(1);
    const itemY = y + 70 + index * 18;
    
    svg += `
      <text
        x="${x + 15}"
        y="${itemY}"
        font-family="Arial"
        font-size="11"
        fill="${style.color}"
      >
        ${style.icon} ${kind}: ${stats[kind]} (${percentage}%)
      </text>
    `;
  });
  
  svg += `</g>`;
  
  return svg;
}
