/**
 * Component Architecture Diagram Renderer
 * 
 * Renders Logical Architecture (LA layer) component diagrams with:
 * - Logical components (blue boxes)
 * - Component hierarchies (nested components)
 * - Component interfaces (provided/required)
 * - Component exchanges (connections)
 * - Behavioral components
 */

import {
  LogicalArchitecture,
  LogicalComponent,
  ComponentExchange,
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
  assignPortSides,
} from '../layouts/hierarchical';
import { optimizeDiagram, generateOptimizationReport } from '../layouts/multi-pass-optimizer';
import {
  createSvgDocument,
  createRect,
  createCircle,
  createLine,
  createText,
  createGroup,
  createPath,
  createArrowMarker,
  createRoundedRect,
  createShadowFilter,
} from '../utils/svg';
import { wrapText, measureText } from '../utils/text-metrics';
import {
  getSafetyColorConfig,
  getSafetyBorderAttributes,
  createSafetyBadge,
  parseSafetyLevel,
  isSafetyCritical,
} from '../utils/safety-colors';
import {
  getLogicalComponentColor,
  getSafetyBorderColor,
  ComponentColors,
} from '../utils/capella-colors';
import {
  getExchangeItemStyle,
  ExchangeItemKind,
} from '../utils/exchange-item-visualization';
import {
  validateDiagramQuality,
  generateQualityReport,
} from '../utils/quality-metrics';
import {
  createProvidedInterface,
  createRequiredInterface,
  createAllInterfaceMarkers,
} from '../utils/interface-notation';

// ============================================================================
// Configuration
// ============================================================================

const DEFAULT_CONFIG: RenderConfig = {
  showLabels: true,
  colorScheme: CAPELLA_COLORS,
  fontSize: 12,
};

// Interface notation configuration (LaTeX Spec Page 19)
// Precise UML/SysML/Capella ball-and-socket notation
const INTERFACE_SIZE = 12; // Size of provided/required interface symbols
const INTERFACE_RADIUS = INTERFACE_SIZE / 2;
const INTERFACE_LINE_LENGTH = 20; // Length of lollipop stick

// Component type stereotypes and icons
const COMPONENT_STEREOTYPES: Record<string, string> = {
  'sensor': '<<sensor>>',
  'controller': '<<controller>>',
  'actuator': '<<actuator>>',
  'processor': '<<processor>>',
  'gateway': '<<gateway>>',
  'display': '<<display>>',
  'interface': '<<interface>>',
};

function inferComponentStereotype(name: string, metadata?: any): string | null {
  const lowerName = name.toLowerCase();
  if (lowerName.includes('sensor') || lowerName.includes('radar') || lowerName.includes('camera') || lowerName.includes('lidar')) {
    return COMPONENT_STEREOTYPES['sensor'];
  }
  if (lowerName.includes('controller') || lowerName.includes('control')) {
    return COMPONENT_STEREOTYPES['controller'];
  }
  if (lowerName.includes('actuator') || lowerName.includes('brake') || lowerName.includes('motor')) {
    return COMPONENT_STEREOTYPES['actuator'];
  }
  if (lowerName.includes('tracker') || lowerName.includes('processor') || lowerName.includes('assessor')) {
    return COMPONENT_STEREOTYPES['processor'];
  }
  if (lowerName.includes('fusion') || lowerName.includes('gateway')) {
    return COMPONENT_STEREOTYPES['gateway'];
  }
  if (lowerName.includes('interface')) {
    return COMPONENT_STEREOTYPES['interface'];
  }
  return null;
}

function getSemanticColor(stereotype: string | null, isNested: boolean): string {
  // Use official Capella colors per Table 6
  if (!stereotype) {
    // Default: Logical Component color (#6495ED)
    return isNested ? '#7FB3E5' : getLogicalComponentColor();
  }
  
  // Capella-compliant colors for stereotypes
  const baseColors: Record<string, string> = {
    '<<sensor>>': ComponentColors.SENSOR,       // #70AD47 (Green)
    '<<controller>>': getLogicalComponentColor(), // #6495ED (Cornflower Blue)
    '<<actuator>>': ComponentColors.ACTUATOR,   // #ED7D31 (Orange)
    '<<processor>>': getLogicalComponentColor(), // #6495ED (Cornflower Blue)
    '<<gateway>>': getLogicalComponentColor(),  // #6495ED (Cornflower Blue)
    '<<interface>>': getLogicalComponentColor(), // #6495ED (Cornflower Blue)
  };
  
  const nestedColors: Record<string, string> = {
    '<<sensor>>': '#9FD96A',      // Lighter green
    '<<controller>>': '#7FB3E5',  // Lighter blue
    '<<actuator>>': '#F5A55D',    // Lighter orange
    '<<processor>>': '#7FB3E5',   // Lighter blue
    '<<gateway>>': '#7FB3E5',     // Lighter blue
    '<<interface>>': '#7FB3E5',   // Lighter blue
  };
  
  const colorMap = isNested ? nestedColors : baseColors;
  return colorMap[stereotype] || (isNested ? '#7FB3E5' : getLogicalComponentColor());
}

function createComponentIcon(x: number, y: number, stereotype: string): SvgElement[] {
  const elements: SvgElement[] = [];
  const size = 16;
  
  switch (stereotype) {
    case COMPONENT_STEREOTYPES['sensor']:
      // Sensor icon: circle with rays
      elements.push(
        createCircle(x, y, size / 2, {
          fill: 'none',
          stroke: '#FFFFFF',
          'stroke-width': 1.5,
        })
      );
      for (let i = 0; i < 8; i++) {
        const angle = (i * Math.PI) / 4;
        const x1 = x + Math.cos(angle) * (size / 2 + 2);
        const y1 = y + Math.sin(angle) * (size / 2 + 2);
        const x2 = x + Math.cos(angle) * (size / 2 + 5);
        const y2 = y + Math.sin(angle) * (size / 2 + 5);
        elements.push(
          createLine(x1, y1, x2, y2, {
            stroke: '#FFFFFF',
            'stroke-width': 1.5,
          })
        );
      }
      break;
      
    case COMPONENT_STEREOTYPES['controller']:
      // Controller icon: chip/processor
      elements.push(
        createRect(x - size / 2, y - size / 2, size, size, {
          fill: 'none',
          stroke: '#FFFFFF',
          'stroke-width': 1.5,
          rx: 2,
          ry: 2,
        })
      );
      // Add pins
      for (let i = 0; i < 3; i++) {
        const offset = (i - 1) * 5;
        elements.push(
          createLine(x - size / 2 - 3, y + offset, x - size / 2, y + offset, {
            stroke: '#FFFFFF',
            'stroke-width': 1,
          })
        );
        elements.push(
          createLine(x + size / 2, y + offset, x + size / 2 + 3, y + offset, {
            stroke: '#FFFFFF',
            'stroke-width': 1,
          })
        );
      }
      break;
      
    case COMPONENT_STEREOTYPES['actuator']:
      // Actuator icon: gear
      const teeth = 8;
      for (let i = 0; i < teeth; i++) {
        const angle = (i * 2 * Math.PI) / teeth;
        const x1 = x + Math.cos(angle) * (size / 2 - 2);
        const y1 = y + Math.sin(angle) * (size / 2 - 2);
        const x2 = x + Math.cos(angle) * (size / 2 + 1);
        const y2 = y + Math.sin(angle) * (size / 2 + 1);
        elements.push(
          createLine(x1, y1, x2, y2, {
            stroke: '#FFFFFF',
            'stroke-width': 2,
          })
        );
      }
      elements.push(
        createCircle(x, y, size / 3, {
          fill: 'none',
          stroke: '#FFFFFF',
          'stroke-width': 1.5,
        })
      );
      break;
      
    case COMPONENT_STEREOTYPES['processor']:
      // Processor icon: brain/cpu
      elements.push(
        createCircle(x, y, size / 2, {
          fill: 'none',
          stroke: '#FFFFFF',
          'stroke-width': 1.5,
        })
      );
      elements.push(
        createLine(x - size / 4, y - size / 4, x + size / 4, y + size / 4, {
          stroke: '#FFFFFF',
          'stroke-width': 1,
        })
      );
      elements.push(
        createLine(x + size / 4, y - size / 4, x - size / 4, y + size / 4, {
          stroke: '#FFFFFF',
          'stroke-width': 1,
        })
      );
      break;
      
    case COMPONENT_STEREOTYPES['gateway']:
      // Gateway icon: network node
      elements.push(
        createCircle(x, y, 3, {
          fill: '#FFFFFF',
          stroke: 'none',
        })
      );
      for (let i = 0; i < 4; i++) {
        const angle = (i * Math.PI) / 2;
        const x2 = x + Math.cos(angle) * (size / 2);
        const y2 = y + Math.sin(angle) * (size / 2);
        elements.push(
          createLine(x, y, x2, y2, {
            stroke: '#FFFFFF',
            'stroke-width': 1.5,
          })
        );
        elements.push(
          createCircle(x2, y2, 2, {
            fill: '#FFFFFF',
            stroke: 'none',
          })
        );
      }
      break;
      
    case COMPONENT_STEREOTYPES['interface']:
      // Interface icon: connector
      elements.push(
        createRect(x - size / 3, y - size / 2, size / 3 * 2, size, {
          fill: 'none',
          stroke: '#FFFFFF',
          'stroke-width': 1.5,
          rx: 2,
          ry: 2,
        })
      );
      elements.push(
        createLine(x - size / 3, y, x + size / 3, y, {
          stroke: '#FFFFFF',
          'stroke-width': 1,
        })
      );
      break;
  }
  
  return elements;
}

// ============================================================================
// Main Render Function
// ============================================================================

/**
 * Render component architecture diagram
 */
export async function renderComponentArchitecture(
  la: LogicalArchitecture,
  config: Partial<RenderConfig> = {}
): Promise<DiagramOutput> {
  const cfg: RenderConfig = { ...DEFAULT_CONFIG, ...config };

  // 1. Convert model to diagram nodes and edges
  let { nodes, edges } = convertToDiagram(la);
  
  console.log(`[Component Renderer] Converted: ${nodes.length} nodes, ${edges.length} edges`);
  if (edges.length > 0) {
    console.log(`[Component Renderer] Edge examples:`, edges.slice(0, 3).map((e: any) => `${e.from}->${e.to}`));
  }

  // 2. Assign port/interface sides based on connections (MANDATORY per Capella Spec 5.1)
  nodes = assignPortSides(nodes, edges);

  // 3. Apply 5-pass optimization pipeline (Phase 3 - Section 8.2)
  console.log('[Component Renderer] Applying 5-pass optimization pipeline...');
  const optimizationResult = await optimizeDiagram(nodes, edges, {
    enablePass1: true,
    enablePass2: true,
    enablePass3: true,
    enablePass4: true,
    enablePass5: true,
    maxIterations: 5,
    targetCrossings: 10,
    gridSize: 20,
    diagramType: 'component-architecture',
    direction: 'RIGHT',
    nodeSpacing: 100,
    layerSpacing: 150,
  });

  const layout = {
    nodes: optimizationResult.nodes as any,
    edges: optimizationResult.edges as any,
    totalSize: optimizationResult.totalSize,
  };

  // Display optimization report
  const optimizationReport = generateOptimizationReport(optimizationResult);
  console.log('\n' + optimizationReport);
  console.log(`[Component Renderer] After layout: ${layout.nodes.length} nodes, ${layout.edges.length} edges`);

  // 4. Quality metrics already validated in Pass 5, reuse results
  const qualityMetrics = optimizationResult.qualityMetrics;
  const qualityReport = generateQualityReport(qualityMetrics);
  
  console.log('\n[Quality Report from Pass 5]');
  console.log(qualityReport);
  
  if (qualityMetrics.overallScore < 75) {
    console.warn(`[Quality] Diagram quality (${qualityMetrics.overallScore.toFixed(1)}) below production threshold (75)`);
  }

  // 5. Render to SVG
  const svg = renderToSvg(la, layout, cfg);

  // 6. Return result
  const width = layout.totalSize.width;
  const height = layout.totalSize.height;

  return {
    svg,
    width,
    height,
    metadata: {
      diagramType: 'component-architecture',
      componentCount: la.components.length,
      exchangeCount: la.component_exchanges.length,
      interfaceCount: la.interfaces.length,
      qualityScore: qualityMetrics.overallScore,
      qualityLevel: qualityMetrics.qualityLevel,
      regulatoryCompliance: qualityMetrics.regulatoryCompliance,
      qualityReport,
    },
  };
}

// ============================================================================
// Step 1: Convert Model to Diagram
// ============================================================================

function convertToDiagram(la: LogicalArchitecture): {
  nodes: DiagramNode[];
  edges: DiagramEdge[];
} {
  const nodes: DiagramNode[] = [];
  const edges: DiagramEdge[] = [];

  // Convert components to nodes
  for (const component of la.components) {
    nodes.push({
      id: component.id,
      label: component.name,
      type: component.component_type === 'Behavior' ? 'behavior' : 'component',
      metadata: {
        componentType: component.component_type,
        providedInterfaces: component.interfaces_out.map(i => i.name),
        requiredInterfaces: component.interfaces_in.map(i => i.name),
        allocatedFunctions: component.allocated_functions,
        ...component.attributes,  // CRITICAL: Include attributes (safety_level, etc.)
      },
      children: component.sub_components && component.sub_components.length > 0 
        ? convertSubComponents(component.sub_components) 
        : undefined,
      color: component.color || undefined,
    });
  }

  // External actors would be in separate field if present
  // Skipping for now as not in LogicalArchitecture model

  // Convert component exchanges to edges
  console.log(`[convertToDiagram] Processing ${la.component_exchanges.length} component_exchanges`);
  for (let i = 0; i < la.component_exchanges.length; i++) {
    const exchange = la.component_exchanges[i];
    const fromComp = parsePortReference(exchange.from_port).component;
    const toComp = parsePortReference(exchange.to_port).component;
    console.log(`[convertToDiagram] Exchange ${i}: ${exchange.from_port} -> ${exchange.to_port} => ${fromComp} -> ${toComp}`);
    
    edges.push({
      id: `ce-${i}`,
      from: fromComp,
      to: toComp,
      label: exchange.label || undefined,
      type: 'component-exchange',
      metadata: {
        exchangeItem: exchange.exchange_item,
        exchangeKind: ((exchange as any).exchange_item_kind || (exchange as any).kind || 'DATA') as ExchangeItemKind,
        fromPort: parsePortReference(exchange.from_port).port,
        toPort: parsePortReference(exchange.to_port).port,
      },
    });
  }
  
  // Convert logical interfaces (connections) to edges
  console.log(`[convertToDiagram] Processing ${la.interfaces.length} interfaces`);
  for (let i = 0; i < la.interfaces.length; i++) {
    const iface = la.interfaces[i];
    if (iface.from && iface.to) {
      console.log(`[convertToDiagram] Interface ${i}: ${iface.from} -> ${iface.to}`);
      edges.push({
        id: `li-${i}`,
        from: iface.from,
        to: iface.to,
        label: (iface.attributes?.description as any)?.String || undefined,
        type: 'component-exchange',
        metadata: {
          exchangeKind: 'DATA' as ExchangeItemKind, // Default for interfaces
        },
      });
    }
  }
  
  console.log(`[convertToDiagram] Created ${edges.length} edges`);

  return { nodes, edges };
}

function convertSubComponents(subComponents: LogicalComponent[]): DiagramNode[] {
  return subComponents.map(component => ({
    id: component.id,
    label: component.name,
    type: component.component_type === 'Behavior' ? 'behavior' : 'component',
    metadata: {
      componentType: component.component_type,
      providedInterfaces: component.interfaces_out.map(i => i.name),
      requiredInterfaces: component.interfaces_in.map(i => i.name),
    },
    color: component.color || undefined,
  }));
}

function parsePortReference(portRef: string): { component: string; port: string } {
  const parts = portRef.split('.');
  return {
    component: parts[0],
    port: parts.length > 1 ? parts[1] : '',
  };
}

function assignInterfaceSides(nodes: DiagramNode[], edges: DiagramEdge[]): DiagramNode[] {
  // For now, provided interfaces on right, required on left
  // This could be enhanced to analyze edge directions
  return nodes;
}

// ============================================================================
// Step 2: Render to SVG
// ============================================================================

function renderToSvg(
  la: LogicalArchitecture,
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

  // Create layers group for better organization
  const layerElements: SvgElement[] = [];
  
  // Layer 1: Edges
  const edgesLayer: SvgElement[] = [];
  for (const edge of layout.edges) {
    edgesLayer.push(renderExchange(edge, layout.nodes, config));
  }
  layerElements.push(createGroup(edgesLayer, { id: 'layer-edges', class: 'edges-layer' }));

  // Layer 2: Components with nested groups
  const componentsLayer: SvgElement[] = [];
  for (const node of layout.nodes) {
    componentsLayer.push(renderComponent(node, config));
  }
  layerElements.push(createGroup(componentsLayer, { id: 'layer-components', class: 'components-layer' }));

  // Layer 3: Labels
  const labelsLayer: SvgElement[] = [];
  if (la.name) {
    labelsLayer.push(
      createText(20, 30, la.name, {
        'font-family': 'Helvetica Neue, Arial, sans-serif',
        'font-size': 18,
        'font-weight': 'bold',
        fill: '#212529',
      })
    );
  }
  
  // Add metadata text
  labelsLayer.push(
    createText(20, layout.totalSize.height - 10, `Components: ${la.components.length} | Exchanges: ${la.component_exchanges.length}`, {
      'font-family': 'Helvetica Neue, Arial, sans-serif',
      'font-size': 11,
      fill: '#666',
    })
  );
  
  layerElements.push(createGroup(labelsLayer, { id: 'layer-labels', class: 'labels-layer' }));

  elements.push(...layerElements);

  // Create arrow markers and shadow filter
  const defs = [
    createArrowMarker('arrow-component', config.colorScheme?.component || '#5B9BD5', 10),
    createArrowMarker('arrow-black', '#000000', 10),
    createShadowFilter('drop-shadow', 4, 2, 2, 0.15),
  ];
  
  // Add exchange item markers for all types
  const exchangeKinds: ExchangeItemKind[] = ['EVENT', 'FLOW', 'OPERATION', 'DATA', 'SHARED_DATA', 'UNSET'];
  for (const kind of exchangeKinds) {
    const style = getExchangeItemStyle(kind);
    defs.push(createArrowMarker(`arrow-${kind.toLowerCase()}`, style.color, 6));
  }

  return createSvgDocument(layout.totalSize.width, layout.totalSize.height, elements, defs);
}

// ============================================================================
// Render Individual Elements
// ============================================================================

function renderComponent(node: any, config: RenderConfig, isNested: boolean = false): SvgElement {
  const elements: SvgElement[] = [];
  
  // Create nested group for this component
  const componentGroup: SvgElement[] = [];

  if (node.type === 'actor') {
    // Render external actor (dashed box)
    elements.push(
      createRoundedRect(
        node.position.x,
        node.position.y,
        node.size.width,
        node.size.height,
        12,
        {
          fill: config.colorScheme?.actor || '#2E75B6',
          stroke: '#000000',
          'stroke-width': 2,
          'stroke-dasharray': '5,5',
        }
      )
    );

    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + node.size.height / 2,
        node.label,
        {
          'text-anchor': 'middle',
          'dominant-baseline': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 12,
          'font-weight': 'bold',
          fill: '#FFFFFF',
        }
      )
    );

    elements.push(
      createText(
        node.position.x + node.size.width / 2,
        node.position.y + node.size.height - 15,
        '«external»',
        {
          'text-anchor': 'middle',
          'font-family': 'Arial, sans-serif',
          'font-size': 10,
          'font-style': 'italic',
          fill: '#FFFFFF',
        }
      )
    );
  } else {
    // Render component with semantic colors based on stereotype
    const stereotype = inferComponentStereotype(node.label, node.metadata);
    // Use semantic colors (ignore node.color as it's often generic blue from parser)
    let fillColor = getSemanticColor(stereotype, isNested);

    // Check for safety-critical components
    const safetyData = parseSafetyLevel(node.metadata);
    const safetyLevel = safetyData.level;
    const safetyStandard = safetyData.standard;
    const hasSafetyLevel = safetyLevel !== null;
    
    // Build component attributes
    const componentAttrs: Record<string, string> = {
      fill: fillColor,
      class: 'component-node',
    };
    
    if (hasSafetyLevel && safetyLevel) {
      // Apply safety-critical border styling
      const safetyAttrs = getSafetyBorderAttributes(safetyLevel, safetyStandard || undefined);
      Object.assign(componentAttrs, safetyAttrs);
      componentAttrs.filter = 'url(#drop-shadow)';
    } else {
      // Standard component styling
      componentAttrs.stroke = isNested ? '#4A90D9' : '#000000';
      componentAttrs['stroke-width'] = isNested ? '1.5' : '2';
      componentAttrs.filter = isNested ? 'none' : 'url(#drop-shadow)';
    }
    
    elements.push(
      createRoundedRect(
        node.position.x,
        node.position.y,
        node.size.width,
        node.size.height,
        8,
        componentAttrs
      )
    );
    
    // Add safety badge if component is safety-critical
    if (hasSafetyLevel && safetyLevel && isSafetyCritical(node.metadata)) {
      const badgeX = node.position.x + node.size.width - 60;
      const badgeY = node.position.y + 5;
      // createSafetyBadge returns a string, wrap it in a group
      const badgeSvg = createSafetyBadge(badgeX, badgeY, safetyLevel, safetyStandard || undefined);
      // Since createSafetyBadge returns string, we need to skip this for now
      // TODO: Convert safety badge to SvgElement format
    }

    // Component name with multiline wrapping
    const hasChildren = node.children && node.children.length > 0;
    const fontSize = hasChildren ? 14 : 12;
    const maxLabelWidth = node.size.width - 40; // Leave padding on sides
    const labelLines = wrapText(node.label, maxLabelWidth, fontSize, 'bold');
    const lineHeight = fontSize + 4;
    const totalLabelHeight = labelLines.length * lineHeight;
    const startY = hasChildren ? node.position.y + 18 : node.position.y + 20;
    
    // Render each line of the wrapped text
    labelLines.forEach((line: string, index: number) => {
      elements.push(
        createText(
          node.position.x + node.size.width / 2,
          startY + (index * lineHeight),
          line,
          {
            'text-anchor': 'middle',
            'font-family': 'Helvetica Neue, Arial, sans-serif',
            'font-size': fontSize,
            'font-weight': 'bold',
            fill: '#FFFFFF',
            class: 'component-label',
          }
        )
      );
    });
    
    // Add separator line for parent components (dynamic position based on label height)
    if (hasChildren) {
      const separatorY = startY + totalLabelHeight + 8;
      elements.push(
        createLine(
          node.position.x + 10,
          separatorY,
          node.position.x + node.size.width - 10,
          separatorY,
          {
            stroke: '#FFFFFF',
            'stroke-width': 1.5,
            'stroke-opacity': 0.3,
          }
        )
      );
    }

    // Component ID (top-right corner)
    elements.push(
      createText(
        node.position.x + node.size.width - 10,
        node.position.y + 14,
        node.id,
        {
          'text-anchor': 'end',
          'font-family': 'Arial, sans-serif',
          'font-size': 10,
          fill: '#E8F4F8',
        }
      )
    );
    
    // Icon and stereotype (reuse stereotype variable from above)
    if (stereotype) {
      const iconX = node.position.x + 18;
      const iconY = node.position.y + 18;
      const iconElements = createComponentIcon(iconX, iconY, stereotype);
      elements.push(...iconElements);
      
      elements.push(
        createText(
          node.position.x + node.size.width / 2,
          node.position.y + node.size.height - 12,
          stereotype,
          {
            'text-anchor': 'middle',
            'font-family': 'Arial, sans-serif',
            'font-size': 9,
            'font-style': 'italic',
            fill: '#E8F4F8',
          }
        )
      );
    }

    // Stereotype for behavior components
    if (node.metadata?.isBehavior) {
      elements.push(
        createText(
          node.position.x + node.size.width / 2,
          node.position.y + node.size.height - 15,
          '«behavior»',
          {
            'text-anchor': 'middle',
            'font-family': 'Arial, sans-serif',
            'font-size': 10,
            'font-style': 'italic',
            fill: '#E8F4F8',
          }
        )
      );
    }

    // Render provided interfaces (lollipops on right side)
    // Using precise UML/SysML notation geometry
    // Support both metadata.providedInterfaces AND interfaces_out from JSON
    const providedInterfaces = node.metadata?.providedInterfaces || node.metadata?.interfaces_out || node.interfaces_out || [];
    if (providedInterfaces && providedInterfaces.length > 0) {
      const spacing = node.size.height / (providedInterfaces.length + 1);
      providedInterfaces.forEach((iface: any, index: number) => {
        const ifaceName = typeof iface === 'string' ? iface : (iface.name || iface.id || 'Interface');
        const y = node.position.y + spacing * (index + 1);
        const x = node.position.x + node.size.width;
        
        // Lollipop: line + circle (precise UML notation)
        const lineEnd = x + INTERFACE_LINE_LENGTH;
        const circleCenter = lineEnd + INTERFACE_RADIUS;
        
        elements.push(
          createLine(x, y, lineEnd, y, {
            stroke: '#000000',
            'stroke-width': 2,
          })
        );
        elements.push(
          createCircle(circleCenter, y, INTERFACE_RADIUS, {
            fill: '#FFFFFF',
            stroke: '#000000',
            'stroke-width': 2,
          })
        );
        elements.push(
          createText(circleCenter + INTERFACE_RADIUS + 8, y, ifaceName, {
            'dominant-baseline': 'middle',
            'font-family': 'Arial, sans-serif',
            'font-size': 9,
            fill: '#000000',
          })
        );
      });
    }

    // Render required interfaces (sockets on left side)
    // Using precise UML/SysML notation geometry
    // Support both metadata.requiredInterfaces AND interfaces_in from JSON
    const requiredInterfaces = node.metadata?.requiredInterfaces || node.metadata?.interfaces_in || node.interfaces_in || [];
    if (requiredInterfaces && requiredInterfaces.length > 0) {
      const spacing = node.size.height / (requiredInterfaces.length + 1);
      requiredInterfaces.forEach((iface: any, index: number) => {
        const ifaceName = typeof iface === 'string' ? iface : (iface.name || iface.id || 'Interface');
        const y = node.position.y + spacing * (index + 1);
        const x = node.position.x;
        
        // Socket: semicircle arc (precise UML notation)
        // Arc path for LEFT side: vertical semicircle pointing left
        const arcX = x - INTERFACE_LINE_LENGTH;
        const arcPath = `M ${arcX} ${y - INTERFACE_RADIUS} A ${INTERFACE_RADIUS} ${INTERFACE_RADIUS} 0 0 0 ${arcX} ${y + INTERFACE_RADIUS}`;
        
        elements.push(
          createLine(arcX, y, x, y, {
            stroke: '#000000',
            'stroke-width': 2,
          })
        );
        elements.push(
          createPath(arcPath, {
            fill: 'none',
            stroke: '#000000',
            'stroke-width': 2,
          })
        );
        elements.push(
          createText(arcX - 8, y, ifaceName, {
            'text-anchor': 'end',
            'dominant-baseline': 'middle',
            'font-family': 'Arial, sans-serif',
            'font-size': 9,
            fill: '#000000',
          })
        );
      });
    }
    
    // Render nested components (children)
    if (node.children && node.children.length > 0) {
      for (const child of node.children) {
        elements.push(renderComponent(child, config, true));
      }
    }
  }

  return createGroup(elements, { id: `component-${node.id}` });
}

function renderExchange(edge: any, nodes: any[], config: RenderConfig): SvgElement {
  const elements: SvgElement[] = [];

  if (!edge.points || edge.points.length < 2) {
    return createGroup([]);
  }

  // Get exchange item style
  const exchangeKind: ExchangeItemKind = edge.metadata?.exchangeKind || 'DATA';
  const itemStyle = getExchangeItemStyle(exchangeKind);

  // Draw connection path with exchange item styling
  const pathD = generateConnectionPath(edge.points);
  const strokeDasharray = itemStyle.pattern === 'none' ? undefined : itemStyle.pattern;
  
  // For double arrow (OPERATION), render a background path
  if (itemStyle.arrowType === 'double') {
    elements.push(
      createPath(pathD, {
        fill: 'none',
        stroke: itemStyle.color,
        'stroke-width': itemStyle.strokeWidth + 2,
        opacity: '0.3',
      })
    );
  }
  
  elements.push(
    createPath(pathD, {
      fill: 'none',
      stroke: itemStyle.color,
      'stroke-width': itemStyle.strokeWidth,
      'marker-end': `url(#arrow-${exchangeKind.toLowerCase()})`,
      ...(strokeDasharray && { 'stroke-dasharray': strokeDasharray }),
    })
  );

  // Exchange label
  if (edge.label && config.showLabels) {
    // Calculate true geometric midpoint by traversing the path
    let totalLength = 0;
    const segmentLengths: number[] = [];
    
    for (let i = 0; i < edge.points.length - 1; i++) {
      const dx = edge.points[i + 1].x - edge.points[i].x;
      const dy = edge.points[i + 1].y - edge.points[i].y;
      const length = Math.sqrt(dx * dx + dy * dy);
      segmentLengths.push(length);
      totalLength += length;
    }
    
    const halfLength = totalLength / 2;
    let accumulatedLength = 0;
    let midPoint = edge.points[0];
    
    for (let i = 0; i < segmentLengths.length; i++) {
      if (accumulatedLength + segmentLengths[i] >= halfLength) {
        const remainingLength = halfLength - accumulatedLength;
        const ratio = remainingLength / segmentLengths[i];
        midPoint = {
          x: edge.points[i].x + (edge.points[i + 1].x - edge.points[i].x) * ratio,
          y: edge.points[i].y + (edge.points[i + 1].y - edge.points[i].y) * ratio,
        };
        break;
      }
      accumulatedLength += segmentLengths[i];
    }
    
    // Background for label with exchange item styling
    const labelPrefix = itemStyle.labelStyle.prefix;
    const labelSuffix = itemStyle.labelStyle.suffix;
    const fullLabel = `${labelPrefix}${edge.label}${labelSuffix}`;
    
    const fontSize = (config.fontSize || 12) - 3;
    const labelWidth = Math.max(fullLabel.length * (fontSize * 0.6), 80);
    const labelHeight = 20;
    const padding = 6;
    
    elements.push(
      createRect(
        midPoint.x - labelWidth / 2 - padding,
        midPoint.y - labelHeight / 2,
        labelWidth + padding * 2,
        labelHeight,
        {
          fill: '#FFFFFF',
          stroke: itemStyle.color,
          'stroke-width': 1,
          rx: 3,
          ry: 3,
        }
      )
    );

    elements.push(
      createText(midPoint.x, midPoint.y, fullLabel, {
        'text-anchor': 'middle',
        'dominant-baseline': 'middle',
        'font-family': 'Arial, sans-serif',
        'font-size': fontSize,
        fill: itemStyle.color,
        'font-weight': itemStyle.labelStyle.fontWeight as any,
        'font-style': itemStyle.labelStyle.fontStyle as any,
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

