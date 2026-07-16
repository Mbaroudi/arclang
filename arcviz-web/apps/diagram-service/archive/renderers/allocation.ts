/**
 * Function-to-Component Allocation Diagram Renderer
 * 
 * Shows allocation relationships between:
 * - System Functions (from System Analysis)
 * - Logical Components (from Logical Architecture)
 * - Unallocated functions highlighted
 * - Allocation links as visual connections
 * - Professional Capella-style design
 */

import {
  LogicalArchitecture,
  LogicalComponent,
  SystemAnalysis,
  SystemFunction,
} from '../types/model';
import {
  DiagramNode,
  DiagramEdge,
  Point,
  SvgElement,
  RenderConfig,
  DiagramOutput,
  CAPELLA_COLORS,
} from '../types/diagram';
import {
  applyHierarchicalLayout,
} from '../layouts/hierarchical';
import {
  createSvgDocument,
  createRect,
  createLine,
  createText,
  createGroup,
  createArrowMarker,
  createRoundedRect,
} from '../utils/svg';
// Traceability styles (simplified inline)
const getTraceabilityStyle = () => ({ stroke: '#9B59B6', 'stroke-width': 2, 'stroke-dasharray': '5,5' });
const createTraceabilityLabel = (text: string) => text;
const createAllTraceabilityMarkers = () => [];

// ============================================================================
// Configuration
// ============================================================================

const DEFAULT_CONFIG: RenderConfig = {
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
  fontSize: 11,
};

// ============================================================================
// Main Render Function
// ============================================================================

export interface AllocationModel {
  system_analysis: SystemAnalysis;
  logical_architecture: LogicalArchitecture;
}

/**
 * Render function-to-component allocation diagram
 */
export async function renderAllocationDiagram(
  model: AllocationModel,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg: RenderConfig = { ...DEFAULT_CONFIG, ...config };

  // 1. Convert model to diagram nodes and edges
  const { nodes, edges } = convertToDiagram(model);

  // 2. Apply layout (functions on left, components on right)
  const layout = await applyHierarchicalLayout(nodes, edges, {
    direction: 'RIGHT',
    nodeSpacing: 80,
    layerSpacing: 300,
    padding: { top: 120, right: 60, bottom: 180, left: 60 },
  });

  // 3. Render to SVG
  const svg = renderToSvg(model, layout, cfg);

  const allocationCount = edges.filter(e => e.type === 'allocation').length;
  const unallocatedCount = model.logical_architecture.unallocated_functions?.length || 0;

  return {
    svg,
    width: layout.totalSize.width,
    height: layout.totalSize.height,
    metadata: {
      diagramType: 'allocation',
      functionCount: model.system_analysis.functions?.length || 0,
      componentCount: model.logical_architecture.components?.length || 0,
      allocationCount,
      unallocatedCount,
    },
  };
}

// ============================================================================
// Model Conversion
// ============================================================================

function convertToDiagram(model: AllocationModel): {
  nodes: DiagramNode[];
  edges: DiagramEdge[];
} {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  const sa = model.system_analysis;
  const la = model.logical_architecture;

  // Add function nodes (left side)
  if (sa.functions) {
    for (const func of sa.functions) {
      nodes.push(convertFunctionToNode(func, la.unallocated_functions || []));
    }
  }

  // Add component nodes (right side)
  if (la.components) {
    for (const comp of la.components) {
      nodes.push(convertComponentToNode(comp));

      // Create allocation edges
      if (comp.allocated_functions) {
        for (const funcId of comp.allocated_functions) {
          edges.push({
            id: `alloc-${funcId}-${comp.id}`,
            from: funcId,
            to: comp.id,
            type: 'allocation',
            label: 'allocated to',
          });
        }
      }
    }
  }

  return { nodes, edges };
}

function convertFunctionToNode(
  func: SystemFunction,
  unallocatedFunctions: string[]
): DiagramNode {
  const isUnallocated = unallocatedFunctions.includes(func.id);

  return {
    id: func.id,
    label: func.name,
    type: 'function',
    size: { width: 180, height: 80 },
    metadata: {
      category: func.category,
      isUnallocated,
    },
  };
}

function convertComponentToNode(comp: LogicalComponent): DiagramNode {
  const functionCount = comp.allocated_functions?.length || 0;

  return {
    id: comp.id,
    label: comp.name,
    type: 'component',
    size: { width: 200, height: 100 },
    metadata: {
      componentType: comp.component_type,
      allocatedFunctionCount: functionCount,
      allocatedFunctions: comp.allocated_functions || [],
    },
  };
}

// ============================================================================
// SVG Rendering
// ============================================================================

function renderToSvg(
  model: AllocationModel,
  layout: any,
  config: RenderConfig
): string {
  const elements: SvgElement[] = [];
  const defs: SvgElement[] = [];

  // Create traceability marker for allocates relationship
  const allocatesStyle = getTraceabilityStyle('allocates');
  defs.push(createArrowMarker(allocatesStyle.markerEnd, allocatesStyle.strokeColor, 10));
  defs.push(createArrowMarker('arrow-unallocated', '#D32F2F', 10));

  // Background
  elements.push(
    createRect(0, 0, layout.totalSize.width, layout.totalSize.height, {
      fill: config.colorScheme?.background || '#FFFFFF',
      stroke: 'none',
    })
  );

  // Layer labels
  elements.push(
    createText(100, 80, 'System Functions', {
      'font-family': 'Arial, sans-serif',
      'font-size': 16,
      'font-weight': 'bold',
      fill: '#388E3C',
    })
  );

  elements.push(
    createText(layout.totalSize.width - 200, 80, 'Logical Components', {
      'font-family': 'Arial, sans-serif',
      'font-size': 16,
      'font-weight': 'bold',
      fill: '#7B1FA2',
    })
  );

  // Render allocation edges first (behind nodes)
  for (const edge of layout.edges) {
    elements.push(renderAllocation(edge, layout.nodes, config));
  }

  // Render nodes
  for (const node of layout.nodes) {
    if (node.type === 'function') {
      elements.push(renderFunction(node, config));
    } else if (node.type === 'component') {
      elements.push(renderComponent(node, config));
    }
  }

  // Title
  elements.push(
    createText(20, 35, 'Function-to-Component Allocation', {
      'font-family': 'Arial, sans-serif',
      'font-size': 18,
      'font-weight': 'bold',
      fill: '#212529',
    })
  );

  // Legend (below diagram)
  const legendX = (layout.totalSize.width - 230) / 2;
  const legendY = layout.totalSize.height - 160;
  elements.push(renderLegend(legendX, legendY, config));

  return createSvgDocument(
    layout.totalSize.width,
    layout.totalSize.height,
    elements,
    defs
  );
}

function renderFunction(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  const isUnallocated = node.metadata?.isUnallocated || false;
  const fillColor = isUnallocated ? '#FFEBEE' : '#E8F5E9';
  const strokeColor = isUnallocated ? '#D32F2F' : '#388E3C';
  const strokeWidth = isUnallocated ? 4 : 2.5;

  // Function box
  elements.push(
    createRoundedRect(
      node.position.x,
      node.position.y,
      node.size.width,
      node.size.height,
      10,
      {
        fill: fillColor,
        stroke: strokeColor,
        'stroke-width': strokeWidth,
        'filter': 'drop-shadow(0 2px 4px rgba(0,0,0,0.15))',
      }
    )
  );

  // «function» stereotype
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 18,
      '«function»',
      {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 9,
        'font-style': 'italic',
        fill: strokeColor,
      }
    )
  );

  // Function name (with wrapping)
  const lines = node.label.split('\n');
  const lineHeight = 14;
  const startY = node.position.y + 38;

  lines.forEach((line: string, i: number) => {
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        startY + i * lineHeight,
        line,
        {
          'text-anchor': 'middle',
          'dominant-baseline': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 11,
          'font-weight': 'bold',
          fill: strokeColor,
        }
      )
    );
  });

  // Unallocated warning
  if (isUnallocated) {
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + node.size.height - 12,
        '⚠ Not Allocated',
        {
          'text-anchor': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 9,
          'font-weight': 'bold',
          fill: '#D32F2F',
        }
      )
    );
  }

  return createGroup(elements, { id: `func-${node.id}` });
}

function renderComponent(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  const fillColor = '#F3E5F5';
  const strokeColor = '#7B1FA2';

  // Component box
  elements.push(
    createRoundedRect(
      node.position.x,
      node.position.y,
      node.size.width,
      node.size.height,
      10,
      {
        fill: fillColor,
        stroke: strokeColor,
        'stroke-width': 3,
        'filter': 'drop-shadow(0 2px 4px rgba(0,0,0,0.15))',
      }
    )
  );

  // «component» stereotype
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 20,
      '«component»',
      {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 10,
        'font-style': 'italic',
        fill: strokeColor,
      }
    )
  );

  // Component name
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 45,
      node.label,
      {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 12,
        'font-weight': 'bold',
        fill: strokeColor,
      }
    )
  );

  // Function count
  const functionCount = node.metadata?.allocatedFunctionCount || 0;
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 70,
      `${functionCount} allocated function${functionCount !== 1 ? 's' : ''}`,
      {
        'text-anchor': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 10,
        fill: '#666666',
      }
    )
  );

  return createGroup(elements, { id: `comp-${node.id}` });
}

function renderAllocation(edge: any, nodes: any[], config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];
  const points = edge.points;

  if (!points || points.length < 2) return createGroup([]);

  const start = points[0];
  const end = points[points.length - 1];

  // Get traceability style for 'allocates' relationship
  const traceStyle = getTraceabilityStyle('allocates');

  // Allocation line with traceability styling
  elements.push(
    createLine(start.x, start.y, end.x, end.y, {
      stroke: traceStyle.strokeColor,
      'stroke-width': traceStyle.strokeWidth,
      'marker-end': `url(#${traceStyle.markerEnd})`,
      'stroke-dasharray': traceStyle.strokeDasharray,
    })
  );

  // Label with traceability styling
  if (config.showLabels && edge.label) {
    const midX = (start.x + end.x) / 2;
    const midY = (start.y + end.y) / 2;

    const labelStyle = traceStyle.labelStyle;
    const padding = 6;
    const labelWidth = Math.max(edge.label.length * labelStyle.fontSize * 0.6, 80);
    const labelHeight = 20;

    elements.push(
      createRect(midX - labelWidth / 2 - padding, midY - labelHeight / 2, labelWidth + padding * 2, labelHeight, {
        fill: labelStyle.backgroundColor,
        stroke: labelStyle.border,
        'stroke-width': 1,
        rx: 4,
        ry: 4,
        'filter': 'drop-shadow(0 1px 2px rgba(0,0,0,0.1))',
      })
    );

    elements.push(
      createText(midX, midY, `«${edge.label}»`, {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': labelStyle.fontSize,
        'font-style': labelStyle.fontStyle,
        'font-weight': labelStyle.fontWeight,
        fill: labelStyle.fill,
      })
    );
  }

  return createGroup(elements, { id: edge.id });
}

function renderLegend(x: number, y: number, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  // Legend box
  elements.push(
    createRoundedRect(x, y, 230, 130, 8, {
      fill: '#FFFFFF',
      stroke: '#CCCCCC',
      'stroke-width': 1.5,
    })
  );

  // Title
  elements.push(
    createText(x + 115, y + 20, 'Legend', {
      'text-anchor': 'middle',
      'font-family': 'Arial, sans-serif',
      'font-size': 12,
      'font-weight': 'bold',
      fill: '#212529',
    })
  );

  // Allocated function
  elements.push(
    createRoundedRect(x + 10, y + 35, 40, 20, 4, {
      fill: '#E8F5E9',
      stroke: '#388E3C',
      'stroke-width': 2,
    })
  );
  elements.push(
    createText(x + 60, y + 45, 'Allocated Function', {
      'font-family': 'Arial, sans-serif',
      'font-size': 10,
      fill: '#333333',
    })
  );

  // Unallocated function
  elements.push(
    createRoundedRect(x + 10, y + 65, 40, 20, 4, {
      fill: '#FFEBEE',
      stroke: '#D32F2F',
      'stroke-width': 3,
    })
  );
  elements.push(
    createText(x + 60, y + 75, 'Unallocated Function', {
      'font-family': 'Arial, sans-serif',
      'font-size': 10,
      fill: '#333333',
    })
  );

  // Component
  elements.push(
    createRoundedRect(x + 10, y + 95, 40, 20, 4, {
      fill: '#F3E5F5',
      stroke: '#7B1FA2',
      'stroke-width': 2,
    })
  );
  elements.push(
    createText(x + 60, y + 105, 'Component', {
      'font-family': 'Arial, sans-serif',
      'font-size': 10,
      fill: '#333333',
    })
  );

  return createGroup(elements, { id: 'legend' });
}
