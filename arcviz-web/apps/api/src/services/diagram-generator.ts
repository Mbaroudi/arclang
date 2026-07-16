import {
  renderOperationalActivity,
  renderFunctionalDataflow,
  renderComponentArchitecture,
  renderSequenceDiagram,
  renderStateMachine,
  renderPhysicalArchitecture,
  renderClassDiagram,
  renderTreeDiagram,
  renderCapabilityDiagram,
  renderFunctionalChainDiagram,
  type OperationalAnalysis,
  type SystemAnalysis,
  type LogicalArchitecture,
  type Scenario,
  type StateMachine,
  type PhysicalArchitecture,
  type FunctionalChainModel,
} from '@arcviz/diagram-service'

export type DiagramType = 
  | 'operational'
  | 'functional'
  | 'component'
  | 'sequence'
  | 'state-machine'
  | 'physical'
  | 'class'
  | 'tree'
  | 'capability'
  | 'functional-chain'

export interface DiagramGenerationResult {
  success: boolean
  svg?: string
  error?: string
  diagramType: DiagramType
  outputPath?: string
  size?: string
  elementCount?: string
  features?: string[]
}

/**
 * Generate a diagram from model data
 */
export async function generateDiagram(
  diagramType: DiagramType,
  modelData: any
): Promise<DiagramGenerationResult> {
  try {
    let result: { svg: string; width: number; height: number }

    switch (diagramType) {
      case 'operational':
        result = await renderOperationalActivity(
          modelData as OperationalAnalysis
        )
        break

      case 'functional':
        result = await renderFunctionalDataflow(
          modelData as SystemAnalysis
        )
        break

      case 'component':
        result = await renderComponentArchitecture(
          modelData as LogicalArchitecture
        )
        break

      case 'sequence':
        result = await renderSequenceDiagram(
          modelData as Scenario
        )
        break

      case 'state-machine':
        result = await renderStateMachine(
          modelData as StateMachine
        )
        break

      case 'physical':
        result = await renderPhysicalArchitecture(
          modelData as PhysicalArchitecture
        )
        break

      case 'class':
        result = await renderClassDiagram(
          modelData as any
        )
        break

      case 'tree':
        result = await renderTreeDiagram(
          modelData as any
        )
        break

      case 'capability':
        result = await renderCapabilityDiagram(
          modelData as any
        )
        break

      case 'functional-chain':
        result = await renderFunctionalChainDiagram(
          modelData as FunctionalChainModel
        )
        break

      default:
        return {
          success: false,
          error: `Unknown diagram type: ${diagramType}`,
          diagramType,
        }
    }

    const sizeKB = (result.svg.length / 1024).toFixed(1)
    const elementCount = countElements(result.svg)

    return {
      success: true,
      svg: result.svg,
      diagramType,
      size: `${sizeKB}KB`,
      elementCount: `${elementCount} elements`,
      features: getDiagramFeatures(diagramType),
    }
  } catch (error: any) {
    return {
      success: false,
      error: error.message || 'Unknown error during diagram generation',
      diagramType,
    }
  }
}

/**
 * Generate all 10 diagram types
 */
export async function generateAllDiagrams(
  modelData: Record<DiagramType, any>
): Promise<Record<DiagramType, DiagramGenerationResult>> {
  const types: DiagramType[] = [
    'operational',
    'functional',
    'component',
    'sequence',
    'state-machine',
    'physical',
    'class',
    'tree',
    'capability',
    'functional-chain',
  ]

  const results: Record<string, DiagramGenerationResult> = {}

  for (const type of types) {
    if (modelData[type]) {
      results[type] = await generateDiagram(type, modelData[type])
    } else {
      results[type] = {
        success: false,
        error: 'Model data not provided for this diagram type',
        diagramType: type,
      }
    }
  }

  return results
}

/**
 * Count SVG elements (rough estimate)
 */
function countElements(svg: string): number {
  const rectCount = (svg.match(/<rect/g) || []).length
  const circleCount = (svg.match(/<circle/g) || []).length
  const pathCount = (svg.match(/<path/g) || []).length
  const textCount = (svg.match(/<text/g) || []).length
  const lineCount = (svg.match(/<line/g) || []).length

  return rectCount + circleCount + pathCount + textCount + lineCount
}

/**
 * Get feature list for diagram type
 */
function getDiagramFeatures(diagramType: DiagramType): string[] {
  const features: Record<DiagramType, string[]> = {
    operational: [
      'Swimlane layout by actor',
      'Stick figures for human actors',
      'System boxes for components',
      'Activity symbols (⊕)',
      'Protocol labels',
    ],
    functional: [
      'Data flow visualization',
      'Port-based connections',
      'Category coloring',
      'External actor boundaries',
    ],
    component: [
      'Hierarchical structure',
      'Interface protocols',
      'Port visualization',
      'Sub-component nesting',
    ],
    sequence: [
      'Time-ordered messages',
      'Participant lifelines',
      'Fragment blocks',
      'Synchronous/asynchronous calls',
    ],
    'state-machine': [
      'State visualization',
      'Transition arrows',
      'Guard conditions',
      'Entry/exit actions',
    ],
    physical: [
      'Hardware node representation',
      'Deployment links',
      'Communication buses',
      'Physical interfaces',
    ],
    class: [
      'UML class notation',
      'Inheritance hierarchies',
      'Associations',
      'Attributes and operations',
    ],
    tree: [
      'Reingold-Tilford layout',
      'Hierarchical breakdown',
      'Expand/collapse indicators',
      'Category icons',
    ],
    capability: [
      '3-level hierarchy',
      'Mission/Capability/Operational',
      'Capability associations',
      'Color-coded levels',
    ],
    'functional-chain': [
      'Left-to-right execution flow',
      'Function sequence',
      'Data exchange labels',
      'Port connections',
    ],
  }

  return features[diagramType] || []
}
