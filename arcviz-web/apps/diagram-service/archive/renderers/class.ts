/**
 * Class/Interface Diagram Renderer
 * 
 * Renders UML-style class diagrams for data modeling with:
 * - Exchange Items (classes with attributes)
 * - Data Types (enumerations, primitives)
 * - Interface definitions
 * - Relationships (associations, generalizations)
 * - Stereotypes and annotations
 */

import {
  ExchangeItem,
  DataType,
  DataAttribute,
  InterfaceDefinition,
} from '../types/model';
import {
  DiagramNode,
  DiagramEdge,
  Point,
  Size,
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
  createPath,
  createArrowMarker,
  createRoundedRect,
} from '../utils/svg';

// ============================================================================
// Configuration
// ============================================================================

const DEFAULT_CONFIG: RenderConfig = {
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
  fontSize: 12,
};

interface ClassDiagramModel {
  exchange_items: ExchangeItem[];
  data_types: DataType[];
  interfaces?: InterfaceDefinition[];
}

// ============================================================================
// Main Render Function
// ============================================================================

/**
 * Render class/interface diagram
 */
export async function renderClassDiagram(
  model: ClassDiagramModel,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg: RenderConfig = { ...DEFAULT_CONFIG, ...config };

  // 1. Convert model to diagram nodes and edges
  const { nodes, edges } = convertToDiagram(model);

  if (nodes.length === 0) {
    return {
      svg: createEmptyDiagram(cfg),
      width: 400,
      height: 200,
      metadata: {
        diagramType: 'class',
        classCount: 0,
        interfaceCount: 0,
      },
    };
  }

  // 2. Apply hierarchical layout
  const layout = await applyHierarchicalLayout(nodes, edges, {
    direction: 'DOWN',
    nodeSpacing: 80,
    layerSpacing: 150,
  });

  // 3. Render to SVG
  const svg = renderToSvg(model, layout, cfg);

  // 4. Return result
  const width = layout.totalSize.width;
  const height = layout.totalSize.height;

  return {
    svg,
    width,
    height,
    metadata: {
      diagramType: 'class',
      classCount: model.exchange_items.length,
      dataTypeCount: model.data_types.length,
      interfaceCount: model.interfaces?.length || 0,
    },
  };
}

// ============================================================================
// Step 1: Convert Model to Diagram
// ============================================================================

function convertToDiagram(model: ClassDiagramModel): {
  nodes: DiagramNode[];
  edges: DiagramEdge[];
} {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  // Convert Exchange Items to class nodes
  for (const item of model.exchange_items) {
    nodes.push({
      id: item.name,
      label: item.name,
      type: 'component',
      metadata: {
        nodeType: 'class',
        stereotype: item.stereotype,
        attributes: item.attributes,
      },
    });

    // Create edges for attribute types (associations)
    for (const attr of item.attributes) {
      const referencedType = model.exchange_items.find(ei => ei.name === attr.attr_type);
      if (referencedType) {
        edges.push({
          id: `${item.name}-${attr.name}-${attr.attr_type}`,
          from: item.name,
          to: attr.attr_type,
          label: attr.name,
          type: 'association',
        });
      }
    }
  }

  // Convert Data Types to nodes
  for (const dataType of model.data_types) {
    nodes.push({
      id: dataType.name,
      label: dataType.name,
      type: 'component',
      metadata: {
        nodeType: 'datatype',
        baseType: dataType.base_type,
        enumerationValues: dataType.enumeration_values,
      },
    });
  }

  // Create inheritance edges (only if both nodes exist)
  for (const dataType of model.data_types) {
    if (dataType.base_type) {
      const baseExists = model.data_types.some(dt => dt.name === dataType.base_type);
      if (baseExists) {
        edges.push({
          id: `${dataType.name}-extends-${dataType.base_type}`,
          from: dataType.name,
          to: dataType.base_type,
          type: 'association',
          metadata: {
            relationship: 'generalization',
          },
        });
      }
    }
  }

  // Convert Interfaces to nodes
  if (model.interfaces) {
    for (const iface of model.interfaces) {
      nodes.push({
        id: iface.name,
        label: iface.name,
        type: 'component',
        metadata: {
          nodeType: 'interface',
          protocol: iface.protocol,
          format: iface.format,
        },
      });
    }
  }

  return { nodes, edges };
}

// ============================================================================
// Step 2: Render to SVG
// ============================================================================

function renderToSvg(
  model: ClassDiagramModel,
  layout: any,
  config: RenderConfig
): string {
  const elements: SvgElement[] = [];

  // Background
  elements.push(
    createRect(0, 0, layout.totalSize.width, layout.totalSize.height, {
      fill: config.colorScheme?.background || '#FFFFFF',
      stroke: 'none',
    })
  );

  // Render edges first (behind classes)
  for (const edge of layout.edges) {
    elements.push(renderRelationship(edge, layout.nodes, config));
  }

  // Render class nodes
  for (const node of layout.nodes) {
    const originalItem = model.exchange_items.find(ei => ei.name === node.id);
    const originalType = model.data_types.find(dt => dt.name === node.id);
    const originalInterface = model.interfaces?.find(i => i.name === node.id);

    if (originalItem) {
      elements.push(renderClass(node, originalItem, config));
    } else if (originalType) {
      elements.push(renderDataType(node, originalType, config));
    } else if (originalInterface) {
      elements.push(renderInterface(node, originalInterface, config));
    }
  }

  // Title
  elements.push(
    createText(20, 30, 'Data Model', {
      'font-family': 'Arial, sans-serif',
      'font-size': 18,
      'font-weight': 'bold',
      fill: '#212529',
    })
  );

  // Create arrow markers
  const defs = [
    createArrowMarker('arrow-black', '#000000', 10),
    createArrowMarker('arrow-white', '#FFFFFF', 10),
  ];

  // Add generalization marker (hollow triangle)
  defs.push({
    type: 'marker',
    attributes: {
      id: 'generalization',
      viewBox: '0 0 10 10',
      refX: '9',
      refY: '5',
      markerWidth: 10,
      markerHeight: 10,
      orient: 'auto-start-reverse',
    },
    children: [
      {
        type: 'path',
        attributes: {
          d: 'M 0 0 L 10 5 L 0 10 Z',
          fill: '#FFFFFF',
          stroke: '#000000',
          'stroke-width': 2,
        },
      },
    ],
  });

  return createSvgDocument(layout.totalSize.width, layout.totalSize.height, elements, defs);
}

// ============================================================================
// Render Individual Elements
// ============================================================================

function renderClass(node: any, item: ExchangeItem, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  const headerHeight = 50;
  const attrSectionHeight = Math.max(40, item.attributes.length * 20 + 20);
  const totalHeight = headerHeight + attrSectionHeight;

  // Adjust node size
  node.size.height = totalHeight;

  // Class box
  elements.push(
    createRect(node.position.x, node.position.y, node.size.width, node.size.height, {
      fill: '#FFE6CC',
      stroke: '#000000',
      'stroke-width': 2,
    })
  );

  // Header section background
  elements.push(
    createRect(node.position.x, node.position.y, node.size.width, headerHeight, {
      fill: '#FFD699',
      stroke: 'none',
    })
  );

  // Stereotype
  if (item.stereotype) {
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + 15,
        `«${item.stereotype}»`,
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

  // Class name
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + (item.stereotype ? 35 : 25),
      item.name,
      {
        'text-anchor': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 12,
        'font-weight': 'bold',
        fill: '#000000',
      }
    )
  );

  // Separator line
  elements.push(
    createLine(
      node.position.x,
      node.position.y + headerHeight,
      node.position.x + node.size.width,
      node.position.y + headerHeight,
      {
        stroke: '#000000',
        'stroke-width': 2,
      }
    )
  );

  // Attributes
  let yOffset = headerHeight + 20;
  for (const attr of item.attributes) {
    const attrText = `${attr.name}: ${attr.attr_type}${attr.default_value ? ` = ${attr.default_value}` : ''}`;
    elements.push(
      createText(node.position.x + 10, node.position.y + yOffset, attrText, {
        'font-family': 'Arial, sans-serif',
        'font-size': 10,
        fill: '#000000',
      })
    );
    yOffset += 20;
  }

  return createGroup(elements, { id: `class-${node.id}` });
}

function renderDataType(node: any, dataType: DataType, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  const isEnum = dataType.enumeration_values && dataType.enumeration_values.length > 0;
  const headerHeight = 50;
  const valueHeight = isEnum ? dataType.enumeration_values!.length * 18 + 20 : 30;
  const totalHeight = headerHeight + valueHeight;

  node.size.height = totalHeight;

  // Data type box
  elements.push(
    createRect(node.position.x, node.position.y, node.size.width, node.size.height, {
      fill: isEnum ? '#E1F5FF' : '#F0F0F0',
      stroke: '#000000',
      'stroke-width': 2,
    })
  );

  // Header background
  elements.push(
    createRect(node.position.x, node.position.y, node.size.width, headerHeight, {
      fill: isEnum ? '#B3E5FC' : '#D0D0D0',
      stroke: 'none',
    })
  );

  // Stereotype
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 15,
      isEnum ? '«enumeration»' : '«datatype»',
      {
        'text-anchor': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 10,
        'font-style': 'italic',
        fill: '#666666',
      }
    )
  );

  // Type name
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 35,
      dataType.name,
      {
        'text-anchor': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 12,
        'font-weight': 'bold',
        fill: '#000000',
      }
    )
  );

  // Separator
  elements.push(
    createLine(
      node.position.x,
      node.position.y + headerHeight,
      node.position.x + node.size.width,
      node.position.y + headerHeight,
      {
        stroke: '#000000',
        'stroke-width': 2,
      }
    )
  );

  // Enumeration values or base type
  if (isEnum) {
    let yOffset = headerHeight + 18;
    for (const enumVal of dataType.enumeration_values!) {
      const valText = enumVal.value ? `${enumVal.name} = ${enumVal.value}` : enumVal.name;
      elements.push(
        createText(node.position.x + 10, node.position.y + yOffset, valText, {
          'font-family': 'Arial, sans-serif',
          'font-size': 10,
          fill: '#000000',
        })
      );
      yOffset += 18;
    }
  } else if (dataType.base_type) {
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + headerHeight + 20,
        `extends ${dataType.base_type}`,
        {
          'text-anchor': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 9,
          'font-style': 'italic',
          fill: '#666666',
        }
      )
    );
  }

  return createGroup(elements, { id: `datatype-${node.id}` });
}

function renderInterface(node: any, iface: InterfaceDefinition, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  const totalHeight = 80;
  node.size.height = totalHeight;

  // Interface box
  elements.push(
    createRect(node.position.x, node.position.y, node.size.width, node.size.height, {
      fill: '#E8F5E9',
      stroke: '#000000',
      'stroke-width': 2,
    })
  );

  // Header background
  elements.push(
    createRect(node.position.x, node.position.y, node.size.width, 50, {
      fill: '#C8E6C9',
      stroke: 'none',
    })
  );

  // Stereotype
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 15,
      '«interface»',
      {
        'text-anchor': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 10,
        'font-style': 'italic',
        fill: '#666666',
      }
    )
  );

  // Interface name
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 35,
      iface.name,
      {
        'text-anchor': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 12,
        'font-weight': 'bold',
        fill: '#000000',
      }
    )
  );

  // Separator
  elements.push(
    createLine(
      node.position.x,
      node.position.y + 50,
      node.position.x + node.size.width,
      node.position.y + 50,
      {
        stroke: '#000000',
        'stroke-width': 2,
      }
    )
  );

  // Protocol/Format info
  if (iface.protocol || iface.format) {
    const info = [iface.protocol, iface.format].filter(Boolean).join(' / ');
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + 68,
        info,
        {
          'text-anchor': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 9,
          'font-style': 'italic',
          fill: '#666666',
        }
      )
    );
  }

  return createGroup(elements, { id: `interface-${node.id}` });
}

function renderRelationship(edge: any, nodes: any[], config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  if (!edge.points || edge.points.length < 2) {
    return createGroup([]);
  }

  const isGeneralization = edge.metadata?.relationship === 'generalization';
  const pathD = generateConnectionPath(edge.points);

  // Draw connection
  elements.push(
    createPath(pathD, {
      fill: 'none',
      stroke: '#000000',
      'stroke-width': isGeneralization ? 2 : 1.5,
      'marker-end': isGeneralization ? 'url(#generalization)' : 'url(#arrow-black)',
      'stroke-dasharray': isGeneralization ? 'none' : '5,5',
    })
  );

  // Relationship label
  if (edge.label && config.showLabels && !isGeneralization) {
    const midPoint = edge.points[Math.floor(edge.points.length / 2)];
    
    // Improved label background sizing
    const fontSize = (config.fontSize || 12) - 1;
    const labelWidth = Math.max(edge.label.length * (fontSize * 0.6), 80);
    const labelHeight = 20;
    const padding = 6;

    elements.push(
      createRoundedRect(
        midPoint.x - labelWidth / 2 - padding,
        midPoint.y - labelHeight / 2,
        labelWidth + padding * 2,
        labelHeight,
        3,
        {
          fill: '#FFFFFF',
          stroke: '#CED4DA',
          'stroke-width': 1,
        }
      )
    );

    elements.push(
      createText(midPoint.x, midPoint.y, edge.label, {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': config.fontFamily || 'Arial, sans-serif',
        'font-size': fontSize,
        fill: '#495057',
      })
    );
  }

  return createGroup(elements, { id: edge.id });
}

function generateConnectionPath(points: Point[]): string {
  if (points.length === 0) return '';
  if (points.length === 1) return `M ${points[0].x} ${points[0].y}`;

  let path = `M ${points[0].x} ${points[0].y}`;

  for (let i = 1; i < points.length; i++) {
    path += ` L ${points[i].x} ${points[i].y}`;
  }

  return path;
}

function createEmptyDiagram(config: RenderConfig): string {
  const elements: SvgElement[] = [
    createRect(0, 0, 400, 200, {
      fill: config.colorScheme?.background || '#FFFFFF',
      stroke: 'none',
    }),
    createText(200, 100, 'No data types or exchange items found', {
      'text-anchor': 'middle',
      'dominant-baseline': 'middle',
      'font-family': 'Arial, sans-serif',
      'font-size': 14,
      fill: '#666666',
    }),
  ];

  return createSvgDocument(400, 200, elements, []);
}
