/**
 * Accurate text measurement utilities
 * Uses character-based width calculation for SVG text sizing
 */

// Character widths for common fonts at 1px font-size (normalized)
const CHAR_WIDTHS: Record<string, number> = {
  // Uppercase
  'A': 0.722, 'B': 0.722, 'C': 0.722, 'D': 0.722, 'E': 0.667, 'F': 0.611,
  'G': 0.778, 'H': 0.722, 'I': 0.278, 'J': 0.556, 'K': 0.722, 'L': 0.611,
  'M': 0.833, 'N': 0.722, 'O': 0.778, 'P': 0.667, 'Q': 0.778, 'R': 0.722,
  'S': 0.667, 'T': 0.611, 'U': 0.722, 'V': 0.667, 'W': 0.944, 'X': 0.667,
  'Y': 0.667, 'Z': 0.611,
  // Lowercase
  'a': 0.556, 'b': 0.611, 'c': 0.556, 'd': 0.611, 'e': 0.556, 'f': 0.333,
  'g': 0.611, 'h': 0.611, 'i': 0.278, 'j': 0.278, 'k': 0.556, 'l': 0.278,
  'm': 0.889, 'n': 0.611, 'o': 0.611, 'p': 0.611, 'q': 0.611, 'r': 0.389,
  's': 0.556, 't': 0.333, 'u': 0.611, 'v': 0.556, 'w': 0.778, 'x': 0.556,
  'y': 0.556, 'z': 0.500,
  // Numbers
  '0': 0.556, '1': 0.556, '2': 0.556, '3': 0.556, '4': 0.556,
  '5': 0.556, '6': 0.556, '7': 0.556, '8': 0.556, '9': 0.556,
  // Common punctuation
  ' ': 0.278, '.': 0.278, ',': 0.278, ':': 0.278, ';': 0.278,
  '-': 0.333, '_': 0.556, '/': 0.278, '\\': 0.278,
  '(': 0.333, ')': 0.333, '[': 0.333, ']': 0.333, '{': 0.389, '}': 0.389,
  '&': 0.722, '@': 1.000, '#': 0.556, '$': 0.556, '%': 0.889,
  '*': 0.389, '+': 0.584, '=': 0.584, '<': 0.584, '>': 0.584,
  '!': 0.333, '?': 0.611, '\'': 0.278, '"': 0.474,
};

const DEFAULT_CHAR_WIDTH = 0.556; // Average character width

/**
 * Calculate the pixel width of text for a given font size
 */
export function measureText(text: string, fontSize: number, fontWeight: 'normal' | 'bold' = 'normal'): number {
  let width = 0;
  
  for (let i = 0; i < text.length; i++) {
    const char = text[i];
    const charWidth = CHAR_WIDTHS[char] || DEFAULT_CHAR_WIDTH;
    width += charWidth;
  }
  
  // Apply font size multiplier
  width *= fontSize;
  
  // Bold text is approximately 1.1x wider
  if (fontWeight === 'bold') {
    width *= 1.1;
  }
  
  return Math.ceil(width);
}

/**
 * Calculate required box width to contain text with padding
 */
export function calculateBoxWidth(text: string, fontSize: number, fontWeight: 'normal' | 'bold' = 'normal', padding: number = 40): number {
  const textWidth = measureText(text, fontSize, fontWeight);
  return textWidth + padding;
}

/**
 * Truncate text to fit within a maximum width, adding ellipsis if needed
 */
export function truncateText(text: string, maxWidth: number, fontSize: number, fontWeight: 'normal' | 'bold' = 'normal'): string {
  const ellipsis = '...';
  const ellipsisWidth = measureText(ellipsis, fontSize, fontWeight);
  
  if (measureText(text, fontSize, fontWeight) <= maxWidth) {
    return text;
  }
  
  let truncated = text;
  while (truncated.length > 0 && measureText(truncated + ellipsis, fontSize, fontWeight) > maxWidth) {
    truncated = truncated.slice(0, -1);
  }
  
  return truncated + ellipsis;
}

/**
 * Split long text into multiple lines to fit within max width
 */
export function wrapText(text: string, maxWidth: number, fontSize: number, fontWeight: 'normal' | 'bold' = 'normal'): string[] {
  const words = text.split(' ');
  const lines: string[] = [];
  let currentLine = '';
  
  for (const word of words) {
    const testLine = currentLine ? `${currentLine} ${word}` : word;
    const testWidth = measureText(testLine, fontSize, fontWeight);
    
    if (testWidth <= maxWidth) {
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
 * Calculate component dimensions based on content
 */
export interface ComponentDimensions {
  width: number;
  height: number;
  labelLines: string[];
  labelHeight: number;
}

export function calculateComponentDimensions(
  label: string,
  hasChildren: boolean,
  childCount: number = 0,
  childHeight: number = 80,
  fontSize: number = 12
): ComponentDimensions {
  const fontWeight = hasChildren ? 'bold' : 'bold';
  const effectiveFontSize = hasChildren ? 14 : fontSize;
  
  // Calculate label dimensions
  const maxLabelWidth = hasChildren ? 180 : 140;
  const labelLines = wrapText(label, maxLabelWidth, effectiveFontSize, fontWeight);
  const labelHeight = labelLines.length * (effectiveFontSize + 4); // 4px line spacing
  
  // Calculate box width based on longest label line
  let maxLineWidth = 0;
  for (const line of labelLines) {
    const lineWidth = measureText(line, effectiveFontSize, fontWeight);
    maxLineWidth = Math.max(maxLineWidth, lineWidth);
  }
  
  const minWidth = hasChildren ? 200 : 160;
  const width = Math.max(minWidth, maxLineWidth + 40);
  
  // Calculate height
  let height: number;
  if (hasChildren && childCount > 0) {
    const headerHeight = 60; // Space for label, separator, padding
    const childSpacing = 20;
    const totalChildHeight = childCount * childHeight + (childCount - 1) * childSpacing;
    const bottomPadding = 20;
    height = headerHeight + totalChildHeight + bottomPadding;
  } else {
    height = Math.max(80, labelHeight + 40);
  }
  
  return {
    width: Math.ceil(width),
    height: Math.ceil(height),
    labelLines,
    labelHeight: Math.ceil(labelHeight),
  };
}
