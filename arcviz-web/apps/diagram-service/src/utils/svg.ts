/**
 * SVG Generation Utilities
 * 
 * Helper functions for creating SVG elements and paths
 */

import { Point, SvgElement } from '../types/diagram';

// ============================================================================
// SVG Element Creation
// ============================================================================

/**
 * Create SVG rectangle
 */
export function createRect(
  x: number,
  y: number,
  width: number,
  height: number,
  attrs: Record<string, any> = {}
): SvgElement {
  return {
    type: 'rect',
    attributes: {
      x,
      y,
      width,
      height,
      ...attrs,
    },
  };
}

/**
 * Create SVG circle
 */
export function createCircle(
  cx: number,
  cy: number,
  r: number,
  attrs: Record<string, any> = {}
): SvgElement {
  return {
    type: 'circle',
    attributes: {
      cx,
      cy,
      r,
      ...attrs,
    },
  };
}

/**
 * Create SVG text
 */
export function createText(
  x: number,
  y: number,
  text: string,
  attrs: Record<string, any> = {}
): SvgElement {
  return {
    type: 'text',
    attributes: {
      x,
      y,
      ...attrs,
    },
    text,
  };
}

/**
 * Create SVG line
 */
export function createLine(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  attrs: Record<string, any> = {}
): SvgElement {
  return {
    type: 'line',
    attributes: {
      x1,
      y1,
      x2,
      y2,
      ...attrs,
    },
  };
}

/**
 * Create SVG polyline
 */
export function createPolyline(
  points: Point[],
  attrs: Record<string, any> = {}
): SvgElement {
  const pointsStr = points.map(p => `${p.x},${p.y}`).join(' ');
  
  return {
    type: 'polyline',
    attributes: {
      points: pointsStr,
      ...attrs,
    },
  };
}

/**
 * Create SVG path
 */
export function createPath(
  d: string,
  attrs: Record<string, any> = {}
): SvgElement {
  return {
    type: 'path',
    attributes: {
      d,
      ...attrs,
    },
  };
}

/**
 * Create SVG group
 */
export function createGroup(
  children: SvgElement[],
  attrs: Record<string, any> = {}
): SvgElement {
  return {
    type: 'g',
    attributes: attrs,
    children,
  };
}

// ============================================================================
// Path Generation
// ============================================================================

/**
 * Create orthogonal path from points
 */
export function createOrthogonalPath(points: Point[]): string {
  if (points.length < 2) return '';

  let path = `M ${points[0].x} ${points[0].y}`;

  for (let i = 1; i < points.length; i++) {
    path += ` L ${points[i].x} ${points[i].y}`;
  }

  return path;
}

/**
 * Create smooth curve path from points
 */
export function createSmoothPath(points: Point[]): string {
  if (points.length < 2) return '';

  let path = `M ${points[0].x} ${points[0].y}`;

  for (let i = 1; i < points.length; i++) {
    const prev = points[i - 1];
    const curr = points[i];
    
    // Calculate control points for curve
    const dx = curr.x - prev.x;
    const dy = curr.y - prev.y;
    
    const cp1x = prev.x + dx * 0.5;
    const cp1y = prev.y;
    const cp2x = prev.x + dx * 0.5;
    const cp2y = curr.y;
    
    path += ` C ${cp1x} ${cp1y}, ${cp2x} ${cp2y}, ${curr.x} ${curr.y}`;
  }

  return path;
}

/**
 * Create rounded orthogonal path
 */
export function createRoundedPath(points: Point[], radius: number = 10): string {
  if (points.length < 2) return '';

  let path = `M ${points[0].x} ${points[0].y}`;

  for (let i = 1; i < points.length - 1; i++) {
    const prev = points[i - 1];
    const curr = points[i];
    const next = points[i + 1];

    // Calculate direction vectors
    const dx1 = curr.x - prev.x;
    const dy1 = curr.y - prev.y;
    const dx2 = next.x - curr.x;
    const dy2 = next.y - curr.y;

    // Calculate corner points
    const len1 = Math.sqrt(dx1 * dx1 + dy1 * dy1);
    const len2 = Math.sqrt(dx2 * dx2 + dy2 * dy2);
    
    // Avoid division by zero
    if (len1 < 0.01 || len2 < 0.01) {
      path += ` L ${curr.x} ${curr.y}`;
      continue;
    }
    
    const offset1 = Math.min(radius, len1 / 2);
    const offset2 = Math.min(radius, len2 / 2);

    const corner1x = curr.x - (dx1 / len1) * offset1;
    const corner1y = curr.y - (dy1 / len1) * offset1;
    const corner2x = curr.x + (dx2 / len2) * offset2;
    const corner2y = curr.y + (dy2 / len2) * offset2;

    // Line to first corner
    path += ` L ${corner1x} ${corner1y}`;
    
    // Quadratic curve around corner
    path += ` Q ${curr.x} ${curr.y}, ${corner2x} ${corner2y}`;
  }

  // Line to last point
  path += ` L ${points[points.length - 1].x} ${points[points.length - 1].y}`;

  return path;
}

// ============================================================================
// Arrow Markers
// ============================================================================

/**
 * Create SVG marker for arrow
 */
export function createArrowMarker(
  id: string,
  color: string = '#000000',
  size: number = 10
): SvgElement {
  return {
    type: 'marker',
    attributes: {
      id,
      viewBox: '0 0 10 10',
      refX: '9',
      refY: '5',
      markerWidth: size,
      markerHeight: size,
      orient: 'auto-start-reverse',
    },
    children: [
      {
        type: 'path',
        attributes: {
          d: 'M 0 0 L 10 5 L 0 10 z',
          fill: color,
        },
      },
    ],
  };
}

/**
 * Create diamond marker (for associations)
 */
export function createDiamondMarker(
  id: string,
  color: string = '#000000',
  filled: boolean = false
): SvgElement {
  return {
    type: 'marker',
    attributes: {
      id,
      viewBox: '0 0 10 10',
      refX: '5',
      refY: '5',
      markerWidth: '8',
      markerHeight: '8',
      orient: 'auto',
    },
    children: [
      {
        type: 'path',
        attributes: {
          d: 'M 5 0 L 10 5 L 5 10 L 0 5 Z',
          fill: filled ? color : 'white',
          stroke: color,
          'stroke-width': '1',
        },
      },
    ],
  };
}

/**
 * Create shadow filter for depth
 */
export function createShadowFilter(
  id: string = 'drop-shadow',
  blur: number = 4,
  offsetX: number = 2,
  offsetY: number = 2,
  opacity: number = 0.15
): SvgElement {
  return {
    type: 'filter',
    attributes: {
      id,
      x: '-50%',
      y: '-50%',
      width: '200%',
      height: '200%',
    },
    children: [
      {
        type: 'feGaussianBlur',
        attributes: {
          in: 'SourceAlpha',
          stdDeviation: blur,
        },
      },
      {
        type: 'feOffset',
        attributes: {
          dx: offsetX,
          dy: offsetY,
          result: 'offsetblur',
        },
      },
      {
        type: 'feComponentTransfer',
        attributes: {},
        children: [
          {
            type: 'feFuncA',
            attributes: {
              type: 'linear',
              slope: opacity,
            },
          },
        ],
      },
      {
        type: 'feMerge',
        attributes: {},
        children: [
          {
            type: 'feMergeNode',
            attributes: {},
          },
          {
            type: 'feMergeNode',
            attributes: {
              in: 'SourceGraphic',
            },
          },
        ],
      },
    ],
  };
}

// ============================================================================
// Text Utilities
// ============================================================================

/**
 * Wrap text to fit width
 */
export function wrapText(text: string, maxWidth: number, charWidth: number = 7): string[] {
  const maxChars = Math.floor(maxWidth / charWidth);
  const words = text.split(' ');
  const lines: string[] = [];
  let currentLine = '';

  for (const word of words) {
    const testLine = currentLine ? `${currentLine} ${word}` : word;
    
    if (testLine.length <= maxChars) {
      currentLine = testLine;
    } else {
      if (currentLine) {
        lines.push(currentLine);
      }
      currentLine = word;
    }
  }

  if (currentLine) {
    lines.push(currentLine);
  }

  return lines;
}

/**
 * Create multiline text element
 */
export function createMultilineText(
  x: number,
  y: number,
  lines: string[],
  lineHeight: number = 16,
  attrs: Record<string, any> = {}
): SvgElement {
  const children: SvgElement[] = lines.map((line, i) => ({
    type: 'text',
    attributes: {
      x,
      y: y + i * lineHeight,
      ...attrs,
    },
    text: line,
  }));

  return createGroup(children);
}

// ============================================================================
// Rendering
// ============================================================================

/**
 * Render SVG element to string
 */
export function renderSvgElement(element: SvgElement): string {
  const { type, attributes, children, text } = element;

  // Build attributes string
  const attrsStr = Object.entries(attributes)
    .map(([key, value]) => `${key}="${value}"`)
    .join(' ');

  // Self-closing tags
  if (!children && !text && ['circle', 'rect', 'line', 'polyline'].includes(type)) {
    return `<${type} ${attrsStr} />`;
  }

  // Tags with children or text
  const opening = `<${type} ${attrsStr}>`;
  const content = text || (children ? children.map(renderSvgElement).join('\n') : '');
  const closing = `</${type}>`;

  return `${opening}${content}${closing}`;
}

/**
 * Create complete SVG document
 */
export function createSvgDocument(
  width: number,
  height: number,
  children: SvgElement[],
  defs: SvgElement[] = []
): string {
  let svg = `<?xml version="1.0" encoding="UTF-8"?>
<svg width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">`;

  // Add defs (markers, patterns, filters, etc.)
  if (defs.length > 0) {
    svg += '\n<defs>\n';
    svg += defs.map(renderSvgElement).join('\n');
    svg += '\n</defs>\n';
  }

  // Add children
  svg += '\n';
  svg += children.map(renderSvgElement).join('\n');
  svg += '\n</svg>';

  return svg;
}

// ============================================================================
// Common Shapes
// ============================================================================

/**
 * Create rounded rectangle
 */
export function createRoundedRect(
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number = 5,
  attrs: Record<string, any> = {}
): SvgElement {
  return createRect(x, y, width, height, {
    rx: radius,
    ry: radius,
    ...attrs,
  });
}

/**
 * Create stick figure for actor
 */
export function createStickFigure(
  x: number,
  y: number,
  scale: number = 1,
  color: string = '#2E75B6'
): SvgElement {
  const headRadius = 8 * scale;
  const bodyHeight = 20 * scale;
  const armWidth = 12 * scale;
  const legWidth = 10 * scale;

  const centerX = x;
  const headY = y + headRadius;
  const bodyStartY = headY + headRadius;
  const bodyEndY = bodyStartY + bodyHeight;
  const armY = bodyStartY + bodyHeight * 0.3;

  return createGroup([
    // Head
    createCircle(centerX, headY, headRadius, {
      fill: 'none',
      stroke: color,
      'stroke-width': 2,
    }),
    // Body
    createLine(centerX, bodyStartY, centerX, bodyEndY, {
      stroke: color,
      'stroke-width': 2,
    }),
    // Arms
    createLine(centerX - armWidth, armY, centerX + armWidth, armY, {
      stroke: color,
      'stroke-width': 2,
    }),
    // Left leg
    createLine(centerX, bodyEndY, centerX - legWidth, bodyEndY + bodyHeight * 0.5, {
      stroke: color,
      'stroke-width': 2,
    }),
    // Right leg
    createLine(centerX, bodyEndY, centerX + legWidth, bodyEndY + bodyHeight * 0.5, {
      stroke: color,
      'stroke-width': 2,
    }),
  ]);
}
