/**
 * Precise Interface Notation
 * 
 * Implements LaTeX Specification Section 5 (Interface Notation), Page 19
 * 
 * Interface Types:
 * - Provided Interface: Semi-circle on component boundary (component offers)
 * - Required Interface: Semi-circle pointing inward (component needs)
 * - Port: Small rectangle on boundary
 * - Ball-and-Socket: Combined provided/required notation
 * 
 * Precise UML/SysML/Capella notation with correct geometric shapes
 */

import { Point } from '../types/diagram';

export type InterfaceType = 'PROVIDED' | 'REQUIRED' | 'PORT' | 'BALL_AND_SOCKET';

export type InterfaceSide = 'TOP' | 'RIGHT' | 'BOTTOM' | 'LEFT';

export interface InterfaceNotationConfig {
  radius: number;
  portSize: number;
  strokeWidth: number;
  color: string;
  labelOffset: number;
}

const DEFAULT_CONFIG: InterfaceNotationConfig = {
  radius: 12,
  portSize: 10,
  strokeWidth: 2,
  color: '#000000',
  labelOffset: 20,
};

/**
 * Create provided interface (lollipop)
 * Semi-circle protruding from component boundary
 */
export function createProvidedInterface(
  x: number,
  y: number,
  side: InterfaceSide,
  label: string,
  config: Partial<InterfaceNotationConfig> = {}
): string {
  const cfg = { ...DEFAULT_CONFIG, ...config };
  
  const { lineStart, lineEnd, circleCenter } = calculateProvidedGeometry(
    x, y, side, cfg.radius
  );
  
  return `
    <g class="provided-interface">
      <line
        x1="${lineStart.x}"
        y1="${lineStart.y}"
        x2="${lineEnd.x}"
        y2="${lineEnd.y}"
        stroke="${cfg.color}"
        stroke-width="${cfg.strokeWidth}"
      />
      <circle
        cx="${circleCenter.x}"
        cy="${circleCenter.y}"
        r="${cfg.radius}"
        fill="white"
        stroke="${cfg.color}"
        stroke-width="${cfg.strokeWidth}"
      />
      ${createInterfaceLabel(circleCenter.x, circleCenter.y, side, label, cfg)}
    </g>
  `;
}

/**
 * Create required interface (socket)
 * Semi-circle/arc pointing inward
 */
export function createRequiredInterface(
  x: number,
  y: number,
  side: InterfaceSide,
  label: string,
  config: Partial<InterfaceNotationConfig> = {}
): string {
  const cfg = { ...DEFAULT_CONFIG, ...config };
  
  const { arcPath, arcCenter } = calculateRequiredGeometry(
    x, y, side, cfg.radius
  );
  
  return `
    <g class="required-interface">
      <path
        d="${arcPath}"
        fill="none"
        stroke="${cfg.color}"
        stroke-width="${cfg.strokeWidth}"
      />
      ${createInterfaceLabel(arcCenter.x, arcCenter.y, side, label, cfg)}
    </g>
  `;
}

/**
 * Create port (small rectangle)
 */
export function createPort(
  x: number,
  y: number,
  side: InterfaceSide,
  direction: 'IN' | 'OUT' | 'INOUT',
  label: string,
  config: Partial<InterfaceNotationConfig> = {}
): string {
  const cfg = { ...DEFAULT_CONFIG, ...config };
  
  const { rectX, rectY } = calculatePortPosition(x, y, side, cfg.portSize);
  
  const fillColor = direction === 'IN' ? '#4caf50' : 
                    direction === 'OUT' ? '#ff9800' : '#2196f3';
  
  return `
    <g class="port" data-direction="${direction}">
      <rect
        x="${rectX}"
        y="${rectY}"
        width="${cfg.portSize}"
        height="${cfg.portSize}"
        fill="${fillColor}"
        stroke="${cfg.color}"
        stroke-width="${cfg.strokeWidth}"
        rx="2"
      />
      ${createInterfaceLabel(x, y, side, label, cfg)}
    </g>
  `;
}

/**
 * Create ball-and-socket notation (provided + required)
 * Used for interface connections between components
 */
export function createBallAndSocket(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  label: string,
  config: Partial<InterfaceNotationConfig> = {}
): string {
  const cfg = { ...DEFAULT_CONFIG, ...config };
  
  const midX = (x1 + x2) / 2;
  const midY = (y1 + y2) / 2;
  
  const angle = Math.atan2(y2 - y1, x2 - x1);
  
  const ballOffset = cfg.radius + 5;
  const socketOffset = cfg.radius + 5;
  
  const ballX = x1 + Math.cos(angle) * ballOffset;
  const ballY = y1 + Math.sin(angle) * ballOffset;
  
  const socketX = x2 - Math.cos(angle) * socketOffset;
  const socketY = y2 - Math.sin(angle) * socketOffset;
  
  const socketArc = createSocketArc(socketX, socketY, angle, cfg.radius);
  
  return `
    <g class="ball-and-socket">
      <line
        x1="${x1}"
        y1="${y1}"
        x2="${ballX}"
        y2="${ballY}"
        stroke="${cfg.color}"
        stroke-width="${cfg.strokeWidth}"
      />
      
      <circle
        cx="${ballX}"
        cy="${ballY}"
        r="${cfg.radius}"
        fill="white"
        stroke="${cfg.color}"
        stroke-width="${cfg.strokeWidth}"
      />
      
      <line
        x1="${ballX + Math.cos(angle) * cfg.radius}"
        y1="${ballY + Math.sin(angle) * cfg.radius}"
        x2="${socketX - Math.cos(angle) * cfg.radius}"
        y2="${socketY - Math.sin(angle) * cfg.radius}"
        stroke="${cfg.color}"
        stroke-width="${cfg.strokeWidth}"
      />
      
      <path
        d="${socketArc}"
        fill="none"
        stroke="${cfg.color}"
        stroke-width="${cfg.strokeWidth}"
      />
      
      <line
        x1="${socketX}"
        y1="${socketY}"
        x2="${x2}"
        y2="${y2}"
        stroke="${cfg.color}"
        stroke-width="${cfg.strokeWidth}"
      />
      
      ${label ? `
        <text
          x="${midX}"
          y="${midY - 10}"
          text-anchor="middle"
          font-family="Arial"
          font-size="11"
          fill="${cfg.color}"
        >${label}</text>
      ` : ''}
    </g>
  `;
}

/**
 * Calculate provided interface geometry
 */
function calculateProvidedGeometry(
  x: number,
  y: number,
  side: InterfaceSide,
  radius: number
): {
  lineStart: Point;
  lineEnd: Point;
  circleCenter: Point;
} {
  const lineLength = radius;
  const totalLength = radius * 2 + lineLength;
  
  switch (side) {
    case 'TOP':
      return {
        lineStart: { x, y },
        lineEnd: { x, y: y - lineLength },
        circleCenter: { x, y: y - lineLength - radius },
      };
    case 'RIGHT':
      return {
        lineStart: { x, y },
        lineEnd: { x: x + lineLength, y },
        circleCenter: { x: x + lineLength + radius, y },
      };
    case 'BOTTOM':
      return {
        lineStart: { x, y },
        lineEnd: { x, y: y + lineLength },
        circleCenter: { x, y: y + lineLength + radius },
      };
    case 'LEFT':
      return {
        lineStart: { x, y },
        lineEnd: { x: x - lineLength, y },
        circleCenter: { x: x - lineLength - radius, y },
      };
  }
}

/**
 * Calculate required interface geometry (socket arc)
 */
function calculateRequiredGeometry(
  x: number,
  y: number,
  side: InterfaceSide,
  radius: number
): {
  arcPath: string;
  arcCenter: Point;
} {
  const r = radius;
  
  switch (side) {
    case 'TOP':
      return {
        arcPath: `M ${x - r} ${y} A ${r} ${r} 0 0 1 ${x + r} ${y}`,
        arcCenter: { x, y: y - r },
      };
    case 'RIGHT':
      return {
        arcPath: `M ${x} ${y - r} A ${r} ${r} 0 0 1 ${x} ${y + r}`,
        arcCenter: { x: x + r, y },
      };
    case 'BOTTOM':
      return {
        arcPath: `M ${x - r} ${y} A ${r} ${r} 0 0 0 ${x + r} ${y}`,
        arcCenter: { x, y: y + r },
      };
    case 'LEFT':
      return {
        arcPath: `M ${x} ${y - r} A ${r} ${r} 0 0 0 ${x} ${y + r}`,
        arcCenter: { x: x - r, y },
      };
  }
}

/**
 * Calculate port position
 */
function calculatePortPosition(
  x: number,
  y: number,
  side: InterfaceSide,
  portSize: number
): {
  rectX: number;
  rectY: number;
} {
  const half = portSize / 2;
  
  switch (side) {
    case 'TOP':
      return { rectX: x - half, rectY: y - portSize };
    case 'RIGHT':
      return { rectX: x, rectY: y - half };
    case 'BOTTOM':
      return { rectX: x - half, rectY: y };
    case 'LEFT':
      return { rectX: x - portSize, rectY: y - half };
  }
}

/**
 * Create socket arc for ball-and-socket
 */
function createSocketArc(
  x: number,
  y: number,
  angle: number,
  radius: number
): string {
  const perpAngle1 = angle + Math.PI / 2;
  const perpAngle2 = angle - Math.PI / 2;
  
  const x1 = x + Math.cos(perpAngle1) * radius;
  const y1 = y + Math.sin(perpAngle1) * radius;
  const x2 = x + Math.cos(perpAngle2) * radius;
  const y2 = y + Math.sin(perpAngle2) * radius;
  
  return `M ${x1} ${y1} A ${radius} ${radius} 0 0 1 ${x2} ${y2}`;
}

/**
 * Create interface label
 */
function createInterfaceLabel(
  x: number,
  y: number,
  side: InterfaceSide,
  label: string,
  config: InterfaceNotationConfig
): string {
  if (!label) return '';
  
  let labelX = x;
  let labelY = y;
  let anchor = 'middle';
  
  switch (side) {
    case 'TOP':
      labelY = y - config.labelOffset;
      break;
    case 'RIGHT':
      labelX = x + config.labelOffset;
      anchor = 'start';
      break;
    case 'BOTTOM':
      labelY = y + config.labelOffset;
      break;
    case 'LEFT':
      labelX = x - config.labelOffset;
      anchor = 'end';
      break;
  }
  
  return `
    <text
      x="${labelX}"
      y="${labelY}"
      text-anchor="${anchor}"
      dominant-baseline="middle"
      font-family="Arial"
      font-size="10"
      font-style="italic"
      fill="${config.color}"
    >${label}</text>
  `;
}

/**
 * Create interface with assembly connector
 * Shows connection between provided and required interfaces
 */
export function createAssemblyConnector(
  providedX: number,
  providedY: number,
  requiredX: number,
  requiredY: number,
  label: string,
  config: Partial<InterfaceNotationConfig> = {}
): string {
  return createBallAndSocket(
    providedX,
    providedY,
    requiredX,
    requiredY,
    label,
    config
  );
}

/**
 * Create interface delegation (port to internal interface)
 */
export function createDelegationConnector(
  portX: number,
  portY: number,
  interfaceX: number,
  interfaceY: number,
  config: Partial<InterfaceNotationConfig> = {}
): string {
  const cfg = { ...DEFAULT_CONFIG, ...config };
  
  return `
    <line
      x1="${portX}"
      y1="${portY}"
      x2="${interfaceX}"
      y2="${interfaceY}"
      stroke="${cfg.color}"
      stroke-width="${cfg.strokeWidth}"
      stroke-dasharray="4,2"
    />
  `;
}

/**
 * Create interface realization (component implements interface)
 */
export function createRealizationConnector(
  componentX: number,
  componentY: number,
  interfaceX: number,
  interfaceY: number,
  config: Partial<InterfaceNotationConfig> = {}
): string {
  const cfg = { ...DEFAULT_CONFIG, ...config };
  
  return `
    <line
      x1="${componentX}"
      y1="${componentY}"
      x2="${interfaceX}"
      y2="${interfaceY}"
      stroke="${cfg.color}"
      stroke-width="${cfg.strokeWidth}"
      stroke-dasharray="8,4"
      marker-end="url(#arrow-realization)"
    />
  `;
}

/**
 * Create all interface markers for SVG defs
 */
export function createAllInterfaceMarkers(): string {
  return `
    <marker
      id="arrow-realization"
      viewBox="0 0 10 10"
      refX="9"
      refY="5"
      markerWidth="6"
      markerHeight="6"
      orient="auto"
    >
      <path d="M 0 0 L 10 5 L 0 10" fill="none" stroke="#000000" stroke-width="2"/>
    </marker>
  `;
}

/**
 * Calculate interface position on component boundary
 */
export function calculateInterfacePosition(
  componentX: number,
  componentY: number,
  componentWidth: number,
  componentHeight: number,
  side: InterfaceSide,
  index: number,
  total: number
): Point {
  const spacing = 1 / (total + 1);
  const position = (index + 1) * spacing;
  
  switch (side) {
    case 'TOP':
      return {
        x: componentX + componentWidth * position,
        y: componentY,
      };
    case 'RIGHT':
      return {
        x: componentX + componentWidth,
        y: componentY + componentHeight * position,
      };
    case 'BOTTOM':
      return {
        x: componentX + componentWidth * position,
        y: componentY + componentHeight,
      };
    case 'LEFT':
      return {
        x: componentX,
        y: componentY + componentHeight * position,
      };
  }
}
