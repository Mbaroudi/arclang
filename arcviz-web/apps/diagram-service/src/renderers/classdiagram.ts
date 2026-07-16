/**
 * Class/Interface Diagram Renderer
 * 
 * Renders UML-style class diagrams with:
 * - Classes with attributes and operations
 * - Interfaces with operations
 * - Data structures with bit-precise fields
 * - Associations, compositions, aggregations
 * - Generalizations (inheritance)
 * - Professional Capella-style design
 */

import {
  ClassModel,
  Class,
  Interface,
  DataStructure,
  Association,
  ClassAttribute,
  Operation,
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
  fontSize: 11,
};

// ============================================================================
// Main Render Function
// ============================================================================

/**
 * Render class diagram
 */
export async function renderClassDiagram(
  model: ClassModel,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg: RenderConfig = { ...DEFAULT_CONFIG, ...config };

  // 1. Convert model to diagram nodes and edges
  const { nodes, edges } = convertToDiagram(model);

  // 2. Apply layout
  const layout = await applyHierarchicalLayout(nodes, edges, {
    direction: 'DOWN',
    nodeSpacing: 80,
    layerSpacing: 120,
  });

  // 3. Render to SVG
  const svg = renderToSvg(model, layout, cfg);

  return {
    svg,
    width: layout.totalSize.width,
    height: layout.totalSize.height,
    metadata: {
      diagramType: 'class',
      classCount: model.classes?.length || 0,
      interfaceCount: model.interfaces?.length || 0,
      dataStructureCount: model.data_structures?.length || 0,
    },
  };
}

// ============================================================================
// Step 1: Convert Model to Diagram
// ============================================================================

function convertToDiagram(model: ClassModel): {
  nodes: DiagramNode[];
  edges: DiagramEdge[];
} {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  // Add classes
  if (model.classes) {
    for (const cls of model.classes) {
      nodes.push(convertClassToNode(cls));
    }
  }

  // Add interfaces
  if (model.interfaces) {
    for (const iface of model.interfaces) {
      nodes.push(convertInterfaceToNode(iface));
    }
  }

  // Add data structures
  if (model.data_structures) {
    for (const ds of model.data_structures) {
      nodes.push(convertDataStructureToNode(ds));
    }
  }

  // Add associations
  if (model.associations) {
    for (let i = 0; i < model.associations.length; i++) {
      const assoc = model.associations[i];
      edges.push({
        id: `assoc-${i}`,
        from: assoc.from,
        to: assoc.to,
        label: assoc.label,
        type: mapAssociationType(assoc.type),
        metadata: {
          multiplicity_from: assoc.multiplicity_from,
          multiplicity_to: assoc.multiplicity_to,
        },
      });
    }
  }

  // Add generalizations
  if (model.generalizations) {
    for (let i = 0; i < model.generalizations.length; i++) {
      const gen = model.generalizations[i];
      edges.push({
        id: `gen-${i}`,
        from: gen.child,
        to: gen.parent,
        type: 'generalization',
      });
    }
  }

  return { nodes, edges };
}

function convertClassToNode(cls: Class): DiagramNode {
  // Calculate dynamic width and height based on content
  const headerHeight = 35;
  const separatorHeight = 2;
  const lineHeight = 14;
  const padding = 12;
  const minWidth = 220;
  
  // Calculate maximum text width
  let maxTextWidth = cls.name.length * 8;
  
  // Check attributes
  const attributeCount = cls.attributes?.length || 0;
  if (cls.attributes) {
    for (const attr of cls.attributes) {
      const visibility = getVisibilitySymbol(attr.visibility);
      const attrText = `${visibility} ${attr.name}: ${attr.type}`;
      const textWidth = attrText.length * 6.5;
      if (textWidth > maxTextWidth) maxTextWidth = textWidth;
    }
  }
  
  // Check operations
  const operationCount = cls.operations?.length || 0;
  if (cls.operations) {
    for (const op of cls.operations) {
      const visibility = getVisibilitySymbol(op.visibility);
      const params = op.parameters?.map((p: any) => `${p.name}: ${p.type}`).join(', ') || '';
      const opText = `${visibility} ${op.name}(${params}): ${op.return_type || 'void'}`;
      const textWidth = opText.length * 6.5;
      if (textWidth > maxTextWidth) maxTextWidth = textWidth;
    }
  }
  
  const calculatedWidth = Math.max(minWidth, maxTextWidth + 20);
  
  const attributeHeight = attributeCount > 0 ? attributeCount * lineHeight + padding * 2 : 0;
  const operationHeight = operationCount > 0 ? operationCount * lineHeight + padding * 2 : 0;
  
  const totalHeight = headerHeight + separatorHeight + 
                      (attributeHeight > 0 ? attributeHeight + separatorHeight : 0) + 
                      (operationHeight > 0 ? operationHeight : 0) + 
                      padding;
  
  return {
    id: cls.name,
    label: cls.name,
    type: 'class',
    size: { width: calculatedWidth, height: Math.max(80, totalHeight) },
    metadata: {
      stereotype: cls.stereotype,
      attributes: cls.attributes,
      operations: cls.operations,
      isAbstract: cls.is_abstract,
    },
  };
}

function convertInterfaceToNode(iface: Interface): DiagramNode {
  const headerHeight = 45;
  const separatorHeight = 2;
  const lineHeight = 14;
  const padding = 12;
  const minWidth = 220;
  
  // Calculate maximum text width
  let maxTextWidth = iface.name.length * 8;
  
  const operationCount = iface.operations?.length || 0;
  if (iface.operations) {
    for (const op of iface.operations) {
      const params = op.parameters?.map((p: any) => `${p.name}: ${p.type}`).join(', ') || '';
      const opText = `+ ${op.name}(${params}): ${op.return_type || 'void'}`;
      const textWidth = opText.length * 6.5;
      if (textWidth > maxTextWidth) maxTextWidth = textWidth;
    }
  }
  
  const calculatedWidth = Math.max(minWidth, maxTextWidth + 20);
  const operationHeight = operationCount > 0 ? operationCount * lineHeight + padding * 2 : 0;
  const totalHeight = headerHeight + separatorHeight + operationHeight + padding;
  
  return {
    id: iface.name,
    label: iface.name,
    type: 'interface',
    size: { width: calculatedWidth, height: Math.max(70, totalHeight) },
    metadata: {
      operations: iface.operations,
    },
  };
}

function convertDataStructureToNode(ds: DataStructure): DiagramNode {
  const headerHeight = 45;
  const separatorHeight = 2;
  const lineHeight = 14;
  const padding = 12;
  const minWidth = 220;
  
  // Calculate maximum text width
  const nameText = ds.bit_size ? `${ds.name} (${ds.bit_size} bits)` : ds.name;
  let maxTextWidth = nameText.length * 8;
  
  const fieldCount = ds.fields?.length || 0;
  if (ds.fields) {
    for (const field of ds.fields) {
      const bitInfo = field.bit_size ? ` [${field.bit_size} bits]` : '';
      const fieldText = `${field.name}: ${field.type}${bitInfo}`;
      const textWidth = fieldText.length * 6.5;
      if (textWidth > maxTextWidth) maxTextWidth = textWidth;
    }
  }
  
  const calculatedWidth = Math.max(minWidth, maxTextWidth + 20);
  const fieldHeight = fieldCount > 0 ? fieldCount * lineHeight + padding * 2 : 0;
  const totalHeight = headerHeight + separatorHeight + fieldHeight + padding;
  
  return {
    id: ds.name,
    label: ds.name,
    type: 'datastructure',
    size: { width: calculatedWidth, height: Math.max(70, totalHeight) },
    metadata: {
      fields: ds.fields,
      bit_size: ds.bit_size,
    },
  };
}

function mapAssociationType(type: string): 'association' | 'composition' | 'aggregation' {
  switch (type) {
    case 'association': return 'association';
    case 'composition': return 'composition';
    case 'aggregation': return 'aggregation';
    default: return 'association';
  }
}

// ============================================================================
// Step 2: Render to SVG
// ============================================================================

function renderToSvg(
  model: ClassModel,
  layout: any,
  config: RenderConfig
): string {
  const elements: SvgElement[] = [];
  const defs: SvgElement[] = [];

  // Create arrow markers
  defs.push(createArrowMarker('arrow-assoc', '#607D8B', 10));
  defs.push(createArrowMarker('arrow-inherit', '#FFFFFF', 12));
  defs.push(createArrowMarker('arrow-diamond', '#607D8B', 10));

  // Background
  elements.push(
    createRect(0, 0, layout.totalSize.width, layout.totalSize.height, {
      fill: config.colorScheme?.background || '#FFFFFF',
      stroke: 'none',
    })
  );

  // Render associations first (behind classes)
  for (const edge of layout.edges) {
    elements.push(renderAssociation(edge, layout.nodes, config));
  }

  // Render classes/interfaces/data structures
  for (const node of layout.nodes) {
    if (node.type === 'class') {
      elements.push(renderClass(node, config));
    } else if (node.type === 'interface') {
      elements.push(renderInterface(node, config));
    } else if (node.type === 'datastructure') {
      elements.push(renderDataStructure(node, config));
    }
  }

  // Title
  if (model.name) {
    elements.push(
      createText(20, 35, model.name, {
        'font-family': 'Arial, sans-serif',
        'font-size': 18,
        'font-weight': 'bold',
        fill: '#212529',
      })
    );
  }

  return createSvgDocument(
    layout.totalSize.width,
    layout.totalSize.height,
    elements,
    defs
  );
}

// ============================================================================
// Render Individual Elements
// ============================================================================

function renderClass(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];
  
  const fillColor = '#E8F5E9';
  const strokeColor = '#388E3C';
  const headerHeight = 35;
  
  // Class box with shadow
  elements.push(
    createRoundedRect(
      node.position.x,
      node.position.y,
      node.size.width,
      node.size.height,
      8,
      {
        fill: fillColor,
        stroke: strokeColor,
        'stroke-width': 2.5,
        'filter': 'drop-shadow(0 2px 4px rgba(0,0,0,0.15))',
      }
    )
  );

  // Stereotype (if present)
  let nameY = node.position.y + headerHeight / 2;
  if (node.metadata?.stereotype) {
    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + 12,
        `«${node.metadata.stereotype}»`,
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
    nameY = node.position.y + 24;
  }

  // Class name
  const fontStyle = node.metadata?.isAbstract ? 'italic' : 'normal';
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      nameY,
      node.label,
      {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 12,
        'font-weight': 'bold',
        'font-style': fontStyle,
        fill: strokeColor,
      }
    )
  );

  // Separator after header
  let currentY = node.position.y + headerHeight;
  elements.push(
    createLine(
      node.position.x,
      currentY,
      node.position.x + node.size.width,
      currentY,
      {
        stroke: strokeColor,
        'stroke-width': 1.5,
        'opacity': '0.7',
      }
    )
  );

  currentY += 12;

  // Attributes
  if (node.metadata?.attributes && node.metadata.attributes.length > 0) {
    for (const attr of node.metadata.attributes) {
      const visibility = getVisibilitySymbol(attr.visibility);
      const attrText = `${visibility} ${attr.name}: ${attr.type}`;
      
      elements.push(
        createText(
          node.position.x + 10,
          currentY,
          attrText,
          {
            'font-family': 'Arial, sans-serif',
            'font-size': 10,
            fill: '#333333',
          }
        )
      );
      currentY += 14;
    }

    // Separator after attributes
    currentY += 4;
    elements.push(
      createLine(
        node.position.x,
        currentY,
        node.position.x + node.size.width,
        currentY,
        {
          stroke: strokeColor,
          'stroke-width': 1.5,
          'opacity': '0.7',
        }
      )
    );
    currentY += 12;
  }

  // Operations
  if (node.metadata?.operations && node.metadata.operations.length > 0) {
    for (const op of node.metadata.operations) {
      const visibility = getVisibilitySymbol(op.visibility);
      const params = op.parameters?.map((p: any) => `${p.name}: ${p.type}`).join(', ') || '';
      const opText = `${visibility} ${op.name}(${params}): ${op.return_type || 'void'}`;
      
      elements.push(
        createText(
          node.position.x + 10,
          currentY,
          opText,
          {
            'font-family': 'Arial, sans-serif',
            'font-size': 10,
            fill: '#333333',
          }
        )
      );
      currentY += 14;
    }
  }

  return createGroup(elements, { id: `class-${node.id}` });
}

function renderInterface(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];
  
  const fillColor = '#E3F2FD';
  const strokeColor = '#1976D2';
  const headerHeight = 45;
  
  // Interface box with shadow
  elements.push(
    createRoundedRect(
      node.position.x,
      node.position.y,
      node.size.width,
      node.size.height,
      8,
      {
        fill: fillColor,
        stroke: strokeColor,
        'stroke-width': 2.5,
        'filter': 'drop-shadow(0 2px 4px rgba(0,0,0,0.15))',
      }
    )
  );

  // «interface» stereotype
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 12,
      '«interface»',
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

  // Interface name
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 28,
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

  // Separator
  let currentY = node.position.y + headerHeight;
  elements.push(
    createLine(
      node.position.x,
      currentY,
      node.position.x + node.size.width,
      currentY,
      {
        stroke: strokeColor,
        'stroke-width': 1.5,
        'opacity': '0.7',
      }
    )
  );

  currentY += 12;

  // Operations
  if (node.metadata?.operations && node.metadata.operations.length > 0) {
    for (const op of node.metadata.operations) {
      const params = op.parameters?.map((p: any) => `${p.name}: ${p.type}`).join(', ') || '';
      const opText = `+ ${op.name}(${params}): ${op.return_type || 'void'}`;
      
      elements.push(
        createText(
          node.position.x + 10,
          currentY,
          opText,
          {
            'font-family': 'Arial, sans-serif',
            'font-size': 10,
            fill: '#333333',
          }
        )
      );
      currentY += 14;
    }
  }

  return createGroup(elements, { id: `interface-${node.id}` });
}

function renderDataStructure(node: any, config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];
  
  const fillColor = '#FFF9C4';
  const strokeColor = '#F57C00';
  const headerHeight = 45;
  
  // Data structure box with shadow
  elements.push(
    createRoundedRect(
      node.position.x,
      node.position.y,
      node.size.width,
      node.size.height,
      8,
      {
        fill: fillColor,
        stroke: strokeColor,
        'stroke-width': 2.5,
        'filter': 'drop-shadow(0 2px 4px rgba(0,0,0,0.15))',
      }
    )
  );

  // «struct» stereotype
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 12,
      '«struct»',
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

  // Structure name
  const nameText = node.metadata?.bit_size 
    ? `${node.label} (${node.metadata.bit_size} bits)`
    : node.label;
  
  elements.push(
    createText(
      node.position.x + node.size.width / 2,
      node.position.y + 28,
      nameText,
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

  // Separator
  let currentY = node.position.y + headerHeight;
  elements.push(
    createLine(
      node.position.x,
      currentY,
      node.position.x + node.size.width,
      currentY,
      {
        stroke: strokeColor,
        'stroke-width': 1.5,
        'opacity': '0.7',
      }
    )
  );

  currentY += 12;

  // Fields with bit precision
  if (node.metadata?.fields && node.metadata.fields.length > 0) {
    for (const field of node.metadata.fields) {
      const bitInfo = field.bit_size ? ` [${field.bit_size} bits]` : '';
      const fieldText = `${field.name}: ${field.type}${bitInfo}`;
      
      elements.push(
        createText(
          node.position.x + 10,
          currentY,
          fieldText,
          {
            'font-family': 'Arial, sans-serif',
            'font-size': 10,
            fill: '#333333',
          }
        )
      );
      currentY += 14;
    }
  }

  return createGroup(elements, { id: `struct-${node.id}` });
}

function renderAssociation(edge: any, nodes: any[], config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];
  const points = edge.points;

  if (!points || points.length < 2) return createGroup([]);

  const start = points[0];
  const end = points[points.length - 1];

  // Determine association style
  let strokeColor = '#607D8B';
  let markerEnd = 'url(#arrow-assoc)';
  let strokeDasharray = undefined;

  if (edge.type === 'generalization') {
    strokeColor = '#607D8B';
    markerEnd = 'url(#arrow-inherit)';
  } else if (edge.type === 'composition') {
    strokeColor = '#D32F2F';
  } else if (edge.type === 'aggregation') {
    strokeDasharray = '5,5';
  }

  // Association line
  elements.push(
    createLine(start.x, start.y, end.x, end.y, {
      stroke: strokeColor,
      'stroke-width': 2,
      'marker-end': markerEnd,
      'stroke-dasharray': strokeDasharray,
    })
  );

  // Label
  if (edge.label && config.showLabels) {
    const midX = (start.x + end.x) / 2;
    const midY = (start.y + end.y) / 2;

    elements.push(
      createRect(midX - 40, midY - 10, 80, 20, {
        fill: '#FFFFFF',
        stroke: strokeColor,
        'stroke-width': 1,
        rx: 3,
        ry: 3,
      })
    );

    elements.push(
      createText(midX, midY, edge.label, {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': 9,
        fill: strokeColor,
      })
    );
  }

  // Multiplicities
  if (edge.metadata?.multiplicity_from) {
    elements.push(
      createText(start.x + 5, start.y - 5, edge.metadata.multiplicity_from, {
        'font-family': 'Arial, sans-serif',
        'font-size': 9,
        fill: '#666666',
      })
    );
  }

  if (edge.metadata?.multiplicity_to) {
    elements.push(
      createText(end.x - 5, end.y - 5, edge.metadata.multiplicity_to, {
        'text-anchor': 'end',
        'font-family': 'Arial, sans-serif',
        'font-size': 9,
        fill: '#666666',
      })
    );
  }

  return createGroup(elements, { id: edge.id });
}

function getVisibilitySymbol(visibility?: string): string {
  switch (visibility) {
    case 'public': return '+';
    case 'private': return '-';
    case 'protected': return '#';
    case 'package': return '~';
    default: return '+';
  }
}
