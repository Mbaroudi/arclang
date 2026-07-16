import ELK, { ElkNode, ElkExtendedEdge } from 'elkjs/lib/elk.bundled.js'
import { ARCVIZ_CONFIG } from '../arcviz-config'

export interface ArchitectureNode {
  id: string
  label: string
  type: 'component' | 'function' | 'requirement' | 'interface' | 'actor'
  layer?: 'operational' | 'system' | 'logical' | 'physical' | 'epbs'
  safetyLevel?: string
  description?: string
  children?: ArchitectureNode[]
  width?: number
  height?: number
  x?: number
  y?: number
}

export interface ArchitectureEdge {
  id: string
  source: string
  target: string
  label?: string
  type?: 'satisfies' | 'implements' | 'realizes' | 'data'
}

export interface ArchitectureGraph {
  nodes: ArchitectureNode[]
  edges: ArchitectureEdge[]
}

export type DiagramType = 
  | 'oab'  // Operational Architecture Blank
  | 'sab'  // System Architecture Blank
  | 'lab'  // Logical Architecture Blank
  | 'pab'  // Physical Architecture Blank
  | 'sdfb' // System Dataflow Blank
  | 'ldfb' // Logical Dataflow Blank
  | 'lcbd' // Logical Component Breakdown
  | 'scbd' // System Component Breakdown
  | 'epbs' // EPBS Architecture

const elk = new ELK()

// Detect dominant layer in graph
function detectDominantLayer(graph: ArchitectureGraph): string {
  const layerCounts: Record<string, number> = {}
  graph.nodes.forEach(node => {
    const layer = node.layer || 'logical'
    layerCounts[layer] = (layerCounts[layer] || 0) + 1
  })
  
  let maxLayer = 'logical'
  let maxCount = 0
  Object.entries(layerCounts).forEach(([layer, count]) => {
    if (count > maxCount) {
      maxCount = count
      maxLayer = layer
    }
  })
  
  return maxLayer
}

// Get layout options based on diagram type
function getDiagramLayoutOptions(diagramType: DiagramType, layer: string) {
  // Dataflow diagrams - emphasize flow
  if (diagramType === 'sdfb' || diagramType === 'ldfb') {
    return {
      'elk.algorithm': 'layered',
      'elk.direction': 'RIGHT',
      'elk.spacing.nodeNode': '70',
      'elk.layered.spacing.nodeNodeBetweenLayers': '120',
      'elk.spacing.edgeNode': '50',
      'elk.spacing.edgeEdge': '30',
      'elk.layered.nodePlacement.strategy': 'LINEAR_SEGMENTS',
      'elk.edgeRouting': 'ORTHOGONAL',
      'elk.layered.thoroughness': '20',
    }
  }
  
  // Breakdown diagrams - tree structure
  if (diagramType === 'lcbd' || diagramType === 'scbd') {
    return {
      'elk.algorithm': 'mrtree',
      'elk.direction': 'DOWN',
      'elk.spacing.nodeNode': '40',
      'elk.mrtree.searchOrder': 'DFS',
    }
  }
  
  // EPBS - product tree
  if (diagramType === 'epbs') {
    return {
      'elk.algorithm': 'mrtree',
      'elk.direction': 'RIGHT',
      'elk.spacing.nodeNode': '50',
    }
  }
  
  // Architecture Blank diagrams - standard Capella style
  switch (layer) {
    case 'operational':
      return {
        'elk.algorithm': 'layered',
        'elk.direction': 'DOWN',
        'elk.spacing.nodeNode': '80',
        'elk.layered.spacing.nodeNodeBetweenLayers': '100',
        'elk.spacing.edgeNode': '50',
      }
    
    case 'system':
      return {
        'elk.algorithm': 'layered',
        'elk.direction': 'DOWN',
        'elk.spacing.nodeNode': '70',
        'elk.layered.spacing.nodeNodeBetweenLayers': '100',
        'elk.spacing.edgeNode': '40',
        'elk.layered.nodePlacement.strategy': 'SIMPLE',
      }
    
    case 'physical':
      return {
        'elk.algorithm': 'layered',
        'elk.direction': 'RIGHT',
        'elk.spacing.nodeNode': '80',
        'elk.layered.spacing.nodeNodeBetweenLayers': '120',
        'elk.spacing.edgeNode': '50',
      }
    
    case 'logical':
    default:
      return {
        'elk.algorithm': 'layered',
        'elk.direction': 'RIGHT',
        'elk.spacing.nodeNode': '60',
        'elk.layered.spacing.nodeNodeBetweenLayers': '90',
        'elk.spacing.edgeNode': '40',
        'elk.spacing.edgeEdge': '25',
        'elk.layered.nodePlacement.strategy': 'NETWORK_SIMPLEX',
        'elk.edgeRouting': 'ORTHOGONAL',
      }
  }
}

// Convert our architecture graph to ELK format
function toElkGraph(graph: ArchitectureGraph, diagramType?: DiagramType): ElkNode {
  const dominantLayer = detectDominantLayer(graph)
  const effectiveDiagramType = diagramType || getDefaultDiagramType(dominantLayer)
  
  console.log('Detected dominant layer:', dominantLayer, 'Diagram type:', effectiveDiagramType)
  
  const elkNodes = graph.nodes.map((node) => {
    const width = node.width || ARCVIZ_CONFIG.node.defaultWidth
    const height = node.height || ARCVIZ_CONFIG.node.defaultHeight
    
    if (!node.width) {
      console.log(`Node ${node.id} missing width, using default: ${width}`)
    }
    
    return {
      id: node.id,
      width,
      height,
      labels: [{ text: node.label }],
      layoutOptions: {
        'nodeLabels.placement': 'INSIDE V_CENTER H_CENTER',
      },
    }
  })

  const elkEdges: ElkExtendedEdge[] = graph.edges.map((edge) => ({
    id: edge.id,
    sources: [edge.source],
    targets: [edge.target],
    labels: edge.label ? [{ text: edge.label }] : [],
  }))

  return {
    id: 'root',
    children: elkNodes,
    edges: elkEdges,
    layoutOptions: getDiagramLayoutOptions(effectiveDiagramType, dominantLayer) as any,
  }
}

// Get default diagram type for layer
function getDefaultDiagramType(layer: string): DiagramType {
  switch (layer) {
    case 'operational': return 'oab'
    case 'system': return 'sab'
    case 'logical': return 'lab'
    case 'physical': return 'pab'
    case 'epbs': return 'epbs'
    default: return 'lab'
  }
}

// Run ELK layout algorithm
export async function layoutGraph(graph: ArchitectureGraph, diagramType?: DiagramType): Promise<ElkNode> {
  console.log('layoutGraph: Input graph has', graph.nodes.length, 'nodes and', graph.edges.length, 'edges', 'Diagram type:', diagramType)
  
  const elkGraph = toElkGraph(graph, diagramType)
  console.log('layoutGraph: Converted to ELK graph with', elkGraph.children?.length, 'children')
  
  try {
    const layoutedGraph = await elk.layout(elkGraph)
    console.log('layoutGraph: ELK layout succeeded')
    return layoutedGraph
  } catch (error) {
    console.error('layoutGraph: ELK layout error:', error)
    throw error
  }
}

// Calculate node dimensions based on content (Capella-style)
export function calculateNodeDimensions(node: ArchitectureNode): { width: number; height: number } {
  const cfg = ARCVIZ_CONFIG.node
  
  if (!ARCVIZ_CONFIG.autoSize.enabled) {
    return {
      width: cfg.defaultWidth,
      height: cfg.defaultHeight,
    }
  }

  // Calculate width based on label length
  const labelWidth = node.label.length * ARCVIZ_CONFIG.autoSize.widthPerChar
  const calculatedWidth = Math.max(
    cfg.minWidth,
    Math.min(
      cfg.maxWidth,
      labelWidth + ARCVIZ_CONFIG.autoSize.minPadding * 2
    )
  )

  // Calculate height based on content
  let calculatedHeight = cfg.headerHeight + cfg.padding * 2
  
  // Add height for functions
  const functionCount = node.children?.length || 0
  calculatedHeight += functionCount * ARCVIZ_CONFIG.autoSize.heightPerFunction
  
  // Add height for ports (if any)
  // Assume 2 ports per interface node as placeholder
  if (node.type === 'interface') {
    calculatedHeight += 2 * ARCVIZ_CONFIG.autoSize.heightPerPort
  }

  calculatedHeight = Math.max(
    cfg.minHeight,
    Math.min(cfg.maxHeight, calculatedHeight)
  )

  return {
    width: calculatedWidth,
    height: calculatedHeight,
  }
}

// Get node color based on type and safety level
export function getNodeColor(node: ArchitectureNode): { fill: string; stroke: string } {
  // Safety level colors (Capella style)
  if (node.safetyLevel) {
    const safetyColors: Record<string, { fill: string; stroke: string }> = {
      ASIL_D: { fill: '#fee2e2', stroke: '#dc2626' },
      ASIL_C: { fill: '#fed7aa', stroke: '#ea580c' },
      ASIL_B: { fill: '#fef3c7', stroke: '#f59e0b' },
      ASIL_A: { fill: '#d1fae5', stroke: '#10b981' },
      DAL_A: { fill: '#fee2e2', stroke: '#dc2626' },
      DAL_B: { fill: '#fed7aa', stroke: '#ea580c' },
      DAL_C: { fill: '#fef3c7', stroke: '#f59e0b' },
      DAL_D: { fill: '#d1fae5', stroke: '#10b981' },
      QM: { fill: '#e0e7ff', stroke: '#6366f1' },
    }
    return safetyColors[node.safetyLevel] || safetyColors.QM
  }

  // Type-based colors
  const typeColors: Record<string, { fill: string; stroke: string }> = {
    actor: { fill: '#fef3c7', stroke: '#f59e0b' },
    component: { fill: '#dbeafe', stroke: '#3b82f6' },
    function: { fill: '#e0e7ff', stroke: '#6366f1' },
    requirement: { fill: '#fce7f3', stroke: '#ec4899' },
    interface: { fill: '#d1fae5', stroke: '#10b981' },
  }

  return typeColors[node.type] || typeColors.component
}

// Get edge color based on type
export function getEdgeColor(edge: ArchitectureEdge): string {
  const edgeColors: Record<string, string> = {
    satisfies: '#10b981',
    implements: '#3b82f6',
    realizes: '#8b5cf6',
    data: '#6b7280',
  }
  return edgeColors[edge.type || 'data'] || edgeColors.data
}

// Generate sample architecture data
export function generateSampleGraph(): ArchitectureGraph {
  return {
    nodes: [
      {
        id: 'REQ-001',
        label: 'System Requirement',
        type: 'requirement',
        description: 'Core system functionality',
        safetyLevel: 'ASIL_B',
      },
      {
        id: 'REQ-002',
        label: 'Safety Requirement',
        type: 'requirement',
        description: 'Emergency stop function',
        safetyLevel: 'ASIL_D',
      },
      {
        id: 'SF-001',
        label: 'Process Data',
        type: 'function',
        description: 'Main processing function',
        safetyLevel: 'ASIL_B',
      },
      {
        id: 'SF-002',
        label: 'Safety Monitor',
        type: 'function',
        description: 'Safety monitoring',
        safetyLevel: 'ASIL_D',
      },
      {
        id: 'LC-001',
        label: 'Sensor Component',
        type: 'component',
        description: 'Data acquisition',
        safetyLevel: 'ASIL_B',
      },
      {
        id: 'LC-002',
        label: 'Controller',
        type: 'component',
        description: 'Main controller',
        safetyLevel: 'ASIL_B',
      },
      {
        id: 'LC-003',
        label: 'Safety Controller',
        type: 'component',
        description: 'Safety-critical control',
        safetyLevel: 'ASIL_D',
      },
      {
        id: 'LC-004',
        label: 'Actuator',
        type: 'component',
        description: 'Output control',
        safetyLevel: 'ASIL_B',
      },
    ],
    edges: [
      { id: 'e1', source: 'SF-001', target: 'REQ-001', label: 'satisfies', type: 'satisfies' },
      { id: 'e2', source: 'SF-002', target: 'REQ-002', label: 'satisfies', type: 'satisfies' },
      { id: 'e3', source: 'LC-001', target: 'SF-001', label: 'implements', type: 'implements' },
      { id: 'e4', source: 'LC-002', target: 'SF-001', label: 'implements', type: 'implements' },
      { id: 'e5', source: 'LC-003', target: 'SF-002', label: 'implements', type: 'implements' },
      { id: 'e6', source: 'LC-001', target: 'LC-002', label: 'data', type: 'data' },
      { id: 'e7', source: 'LC-002', target: 'LC-004', label: 'data', type: 'data' },
      { id: 'e8', source: 'LC-003', target: 'LC-004', label: 'control', type: 'data' },
    ],
  }
}
